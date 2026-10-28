//! Exact state-machine coverage for host-driven CloudControl operations.

#![cfg(not(target_arch = "wasm32"))]

mod common;

use std::collections::{HashMap, VecDeque};
use std::future::Future;
use std::pin::Pin;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use aws_config::{BehaviorVersion, Region};
use aws_sdk_cloudcontrol::operation::create_resource::CreateResourceError;
use aws_smithy_runtime_api::client::{
    http::{
        HttpClient, HttpConnector, HttpConnectorFuture, SharedHttpClient, SharedHttpConnector,
        http_client_fn,
    },
    result::ConnectorError,
};
use carina_core::provider::{
    AppliedIdentifier, AppliedState, CreateAttempt, CreateFailureKind, DeleteRequest, Observation,
    PatchOp, PendingToken, Poll, ProviderError, ReadOutcome, ResumeCreateRequest,
    ResumeFailureKind, Sent, UpdateOutcome, UpdatePatch,
};
use carina_core::resource::{ConcreteValue, Resource, ResourceId, State, Value};
use carina_provider_awscc::provider::{
    AwsccProviderConfig, CcCreateAttempt, CcCreatePoll, CcCreateResume, CcIssuedCreate,
    CcPollAttempt, CcResumeAttempt, CcResumedCreate,
};
use carina_provider_awscc::{
    AwsccProvider, BoundedSdkConfig, CcCreateToken, CcDeleteToken, CcRequestPhase, CcUpdateToken,
    InFlightCreateRequest, RequestToken, UndeterminedCreate,
};
use carina_provider_protocol::OperationStep;
use indexmap::IndexMap;
use serde_json::{Value as JsonValue, json};
use winterbaume_core::{MockAws, MockRequest, MockResponse, MockService, json_error_response};

static NEXT_TEST_REGION: AtomicUsize = AtomicUsize::new(1);
const VPC_TYPE: &str = "AWS::EC2::VPC";
const VPC_ID: &str = "vpc-0123456789abcdef0";
const TEST_CLIENT_TOKEN: &str = "00000000-0000-4000-8000-000000000000";
const IGW_TYPE: &str = "AWS::EC2::InternetGateway";
const IGW_ID: &str = "igw-0123456789abcdef0";
const TRANSIT_GATEWAY_ID: &str = "tgw-0123456789abcdef0";

fn vpc_identifier() -> carina_core::provider::AppliedIdentifier {
    common::applied_identifier(VPC_ID)
}

fn internet_gateway_identifier() -> carina_core::provider::AppliedIdentifier {
    common::applied_identifier(IGW_ID)
}

fn transit_gateway_identifier() -> carina_core::provider::AppliedIdentifier {
    common::applied_identifier(TRANSIT_GATEWAY_ID)
}

#[derive(Clone)]
enum Reply {
    Json(JsonValue),
    Error {
        status: u16,
        code: &'static str,
        message: &'static str,
    },
}

impl Reply {
    fn into_response(self) -> MockResponse {
        match self {
            Self::Json(body) => MockResponse::json(200, body.to_string()),
            Self::Error {
                status,
                code,
                message,
            } => json_error_response(status, code, message),
        }
    }
}

struct ScriptedCloudControl {
    replies: Mutex<HashMap<String, VecDeque<Reply>>>,
    calls: Mutex<HashMap<String, usize>>,
    bodies: Mutex<HashMap<String, Vec<String>>>,
    actions: Mutex<Vec<String>>,
}

impl ScriptedCloudControl {
    fn new(entries: impl IntoIterator<Item = (&'static str, Vec<Reply>)>) -> Self {
        Self {
            replies: Mutex::new(
                entries
                    .into_iter()
                    .map(|(action, replies)| (action.to_string(), replies.into()))
                    .collect(),
            ),
            calls: Mutex::new(HashMap::new()),
            bodies: Mutex::new(HashMap::new()),
            actions: Mutex::new(Vec::new()),
        }
    }

    fn calls(&self, action: &str) -> usize {
        self.calls
            .lock()
            .expect("scripted call-count lock")
            .get(action)
            .copied()
            .unwrap_or(0)
    }

    fn bodies(&self, action: &str) -> Vec<String> {
        self.bodies
            .lock()
            .expect("scripted request-body lock")
            .get(action)
            .cloned()
            .unwrap_or_default()
    }

    fn actions(&self) -> Vec<String> {
        self.actions
            .lock()
            .expect("scripted action-order lock")
            .clone()
    }

    fn next_reply(&self, action: &str) -> Reply {
        let mut replies = self.replies.lock().expect("scripted reply lock");
        let replies = replies
            .get_mut(action)
            .unwrap_or_else(|| panic!("no replies configured for {action}"));
        if replies.len() > 1 {
            replies.pop_front().expect("non-empty reply script")
        } else {
            replies.front().expect("non-empty reply script").clone()
        }
    }
}

impl MockService for ScriptedCloudControl {
    fn service_name(&self) -> &str {
        "cloudcontrolapi"
    }

    fn url_patterns(&self) -> Vec<&str> {
        vec![
            r"https?://cloudcontrolapi\..*\.amazonaws\.com",
            r"https?://cloudcontrolapi\.amazonaws\.com",
        ]
    }

    fn handle(
        &self,
        request: MockRequest,
    ) -> Pin<Box<dyn Future<Output = MockResponse> + Send + '_>> {
        let action = request
            .headers
            .get("x-amz-target")
            .and_then(|value| value.to_str().ok())
            .and_then(|value| value.split('.').next_back())
            .expect("CloudControl request should carry x-amz-target")
            .to_string();
        *self
            .calls
            .lock()
            .expect("scripted call-count lock")
            .entry(action.clone())
            .or_default() += 1;
        self.actions
            .lock()
            .expect("scripted action-order lock")
            .push(action.clone());
        self.bodies
            .lock()
            .expect("scripted request-body lock")
            .entry(action.clone())
            .or_default()
            .push(String::from_utf8_lossy(&request.body).into_owned());
        let response = self.next_reply(&action).into_response();
        Box::pin(async move { response })
    }
}

struct DeniedSts;

impl MockService for DeniedSts {
    fn service_name(&self) -> &str {
        "sts"
    }

    fn url_patterns(&self) -> Vec<&str> {
        vec![
            r"https?://sts\..*\.amazonaws\.com",
            r"https?://sts\.amazonaws\.com",
        ]
    }

    fn handle(
        &self,
        _request: MockRequest,
    ) -> Pin<Box<dyn Future<Output = MockResponse> + Send + '_>> {
        Box::pin(async {
            MockResponse::error(
                403,
                "AccessDeniedException",
                "injected provider initialization failure",
            )
        })
    }
}

#[derive(Clone, Debug)]
struct PostSendFailureConnector {
    inner: SharedHttpConnector,
    failures: Arc<Mutex<HashMap<String, PostSendFailureScript>>>,
}

#[derive(Debug)]
struct PostSendFailureScript {
    outcomes: VecDeque<bool>,
    invocation_id: Option<String>,
    current_outcome: bool,
}

impl PostSendFailureScript {
    fn new(outcomes: Vec<bool>) -> Self {
        Self {
            outcomes: outcomes.into(),
            invocation_id: None,
            current_outcome: false,
        }
    }

    fn outcome_for(&mut self, invocation_id: String) -> bool {
        if self.invocation_id.as_deref() != Some(invocation_id.as_str()) {
            self.current_outcome = if self.outcomes.len() > 1 {
                self.outcomes.pop_front().expect("non-empty failure script")
            } else {
                self.outcomes.front().copied().unwrap_or(false)
            };
            self.invocation_id = Some(invocation_id);
        }
        self.current_outcome
    }
}

impl HttpConnector for PostSendFailureConnector {
    fn call(
        &self,
        request: aws_smithy_runtime_api::client::orchestrator::HttpRequest,
    ) -> HttpConnectorFuture {
        let action = request
            .headers()
            .get("x-amz-target")
            .and_then(|value| value.split('.').next_back())
            .map(ToString::to_string);
        let invocation_id = request
            .headers()
            .get("amz-sdk-invocation-id")
            .map(ToString::to_string);
        let inner = self.inner.clone();
        let failures = Arc::clone(&self.failures);

        HttpConnectorFuture::new(async move {
            let response = inner.call(request).await?;
            let should_fail = action
                .zip(invocation_id)
                .is_some_and(|(action, invocation_id)| {
                    let mut failures = failures.lock().expect("post-send failure script lock");
                    let Some(script) = failures.get_mut(&action) else {
                        return false;
                    };
                    script.outcome_for(invocation_id)
                });
            if should_fail {
                Err(ConnectorError::timeout(Box::new(std::io::Error::new(
                    std::io::ErrorKind::TimedOut,
                    "injected post-send timeout",
                ))))
            } else {
                Ok(response)
            }
        })
    }
}

fn mutation_reply(token: &str, identifier: &str) -> Reply {
    Reply::Json(json!({
        "ProgressEvent": {
            "Identifier": identifier,
            "OperationStatus": "IN_PROGRESS",
            "RequestToken": token,
        }
    }))
}

fn mutation_reply_without_identifier(token: &str) -> Reply {
    Reply::Json(json!({
        "ProgressEvent": {
            "OperationStatus": "IN_PROGRESS",
            "RequestToken": token,
        }
    }))
}

fn mutation_reply_without_request_token(request_token: Option<&str>) -> Reply {
    let mut progress = serde_json::Map::from_iter([(
        "OperationStatus".to_string(),
        JsonValue::String("IN_PROGRESS".to_string()),
    )]);
    if let Some(request_token) = request_token {
        progress.insert(
            "RequestToken".to_string(),
            JsonValue::String(request_token.to_string()),
        );
    }
    Reply::Json(json!({"ProgressEvent": progress}))
}

fn status_reply(status: &str, identifier: Option<&str>, message: Option<&str>) -> Reply {
    status_reply_with_error_code(status, identifier, None, message)
}

fn status_reply_with_error_code(
    status: &str,
    identifier: Option<&str>,
    error_code: Option<&str>,
    message: Option<&str>,
) -> Reply {
    let mut progress = serde_json::Map::from_iter([(
        "OperationStatus".to_string(),
        JsonValue::String(status.to_string()),
    )]);
    if let Some(identifier) = identifier {
        progress.insert(
            "Identifier".to_string(),
            JsonValue::String(identifier.to_string()),
        );
    }
    if let Some(error_code) = error_code {
        progress.insert(
            "ErrorCode".to_string(),
            JsonValue::String(error_code.to_string()),
        );
    }
    if let Some(message) = message {
        progress.insert(
            "StatusMessage".to_string(),
            JsonValue::String(message.to_string()),
        );
    }
    Reply::Json(json!({"ProgressEvent": progress}))
}

fn get_resource_reply(type_name: &str, identifier: &str, properties: JsonValue) -> Reply {
    Reply::Json(json!({
        "TypeName": type_name,
        "ResourceDescription": {
            "Identifier": identifier,
            "Properties": properties.to_string(),
        }
    }))
}

fn throttled() -> Reply {
    Reply::Error {
        status: 400,
        code: "ThrottlingException",
        message: "Rate exceeded",
    }
}

fn invalid_request() -> Reply {
    Reply::Error {
        status: 400,
        code: "InvalidRequestException",
        message: "invalid create request",
    }
}

fn client_token_conflict() -> Reply {
    Reply::Error {
        status: 400,
        code: "ClientTokenConflictException",
        message: "client token reused with different desired state",
    }
}

fn service_internal_error() -> Reply {
    Reply::Error {
        status: 500,
        code: "ServiceInternalErrorException",
        message: "Internal error",
    }
}

fn not_found() -> Reply {
    Reply::Error {
        status: 404,
        code: "ResourceNotFoundException",
        message: "resource not found",
    }
}

fn string(value: &str) -> Value {
    Value::Concrete(ConcreteValue::String(value.to_string().into()))
}

fn bool_(value: bool) -> Value {
    Value::Concrete(ConcreteValue::Bool(value))
}

#[derive(Debug)]
enum TestCreateStep {
    Pending {
        token: CcCreateToken,
        poll_after_secs: u32,
        issued: Option<AppliedIdentifier>,
    },
    Succeeded {
        identifier: AppliedIdentifier,
        attributes: HashMap<String, Value>,
    },
    Partial {
        identifier: AppliedIdentifier,
        reason: String,
    },
    Completed {
        identifier: AppliedIdentifier,
        state: AppliedState,
    },
}

#[derive(Debug)]
enum TestCreateVerdict {
    Untracked(Box<ProviderError>),
    Undetermined {
        token: CcCreateToken,
        error: Box<ProviderError>,
    },
    Created {
        identifier: AppliedIdentifier,
        observation: Box<Observation>,
        error: Box<ProviderError>,
    },
    Preserved {
        token: CcCreateToken,
        identifier: AppliedIdentifier,
        error: Box<ProviderError>,
    },
    ResumePreserved {
        identifier: Option<AppliedIdentifier>,
        error: Box<ProviderError>,
    },
    Discarded(Box<ProviderError>),
}

struct TestCreateAttempt;

struct TestIssuedCreate {
    identifier: AppliedIdentifier,
}

impl CcCreateAttempt for TestCreateAttempt {
    type Step = TestCreateStep;
    type Verdict = TestCreateVerdict;
    type Issued = TestIssuedCreate;

    fn untracked(self, error: ProviderError) -> Self::Verdict {
        TestCreateVerdict::Untracked(Box::new(error))
    }

    fn issued(self, identifier: AppliedIdentifier, _sent: Sent) -> Self::Issued {
        TestIssuedCreate { identifier }
    }

    fn pending(
        self,
        token: CcCreateToken,
        _sent: Sent,
        poll_after_secs: u32,
    ) -> Result<Self::Step, Self::Verdict> {
        Ok(TestCreateStep::Pending {
            token,
            poll_after_secs,
            issued: None,
        })
    }
}

impl CcPollAttempt for TestCreateAttempt {
    type Step = TestCreateStep;
    type Verdict = TestCreateVerdict;
    type Issued = TestIssuedCreate;

    fn untracked(self, error: ProviderError) -> Self::Verdict {
        TestCreateVerdict::Untracked(Box::new(error))
    }

    fn undetermined(self, failure: UndeterminedCreate) -> Self::Verdict {
        let (token, error) = failure.into_token_and_error();
        TestCreateVerdict::Undetermined {
            token,
            error: Box::new(error),
        }
    }

    fn issued(self, identifier: AppliedIdentifier) -> Self::Issued {
        TestIssuedCreate { identifier }
    }

    fn pending(
        self,
        token: CcCreateToken,
        poll_after_secs: u32,
    ) -> Result<Self::Step, Self::Verdict> {
        Ok(TestCreateStep::Pending {
            token,
            poll_after_secs,
            issued: None,
        })
    }
}

impl CcIssuedCreate for TestIssuedCreate {
    type Step = TestCreateStep;
    type Verdict = TestCreateVerdict;

    fn identifier(&self) -> &AppliedIdentifier {
        &self.identifier
    }

    fn succeeded(
        self,
        identifier: AppliedIdentifier,
        attributes: HashMap<String, Value>,
    ) -> Self::Step {
        TestCreateStep::Succeeded {
            identifier,
            attributes,
        }
    }

    fn partial(
        self,
        identifier: AppliedIdentifier,
        _attributes: HashMap<String, Value>,
        reason: String,
        _missing_attributes: Vec<String>,
    ) -> Self::Step {
        TestCreateStep::Partial { identifier, reason }
    }

    fn failed(
        self,
        identifier: AppliedIdentifier,
        observation: Observation,
        error: ProviderError,
    ) -> Self::Verdict {
        TestCreateVerdict::Created {
            identifier,
            observation: Box::new(observation),
            error: Box::new(error),
        }
    }

    fn discarded(self, error: ProviderError) -> Self::Verdict {
        TestCreateVerdict::Discarded(Box::new(error))
    }

    fn preserve(self, token: CcCreateToken, error: ProviderError) -> Self::Verdict {
        TestCreateVerdict::Preserved {
            token,
            identifier: self.identifier,
            error: Box::new(error),
        }
    }

    fn pending(
        self,
        token: CcCreateToken,
        poll_after_secs: u32,
    ) -> Result<Self::Step, Self::Verdict> {
        Ok(TestCreateStep::Pending {
            token,
            poll_after_secs,
            issued: Some(self.identifier),
        })
    }
}

impl CcResumeAttempt for TestCreateAttempt {
    type Step = TestCreateStep;
    type Verdict = TestCreateVerdict;
    type Resumed = TestIssuedCreate;
    type Resent = TestCreateAttempt;

    fn untracked(self, error: ProviderError) -> Self::Verdict {
        TestCreateVerdict::Untracked(Box::new(error))
    }

    fn undetermined(self, failure: UndeterminedCreate) -> Self::Verdict {
        let (token, error) = failure.into_token_and_error();
        TestCreateVerdict::Undetermined {
            token,
            error: Box::new(error),
        }
    }

    fn forgotten(self, token: InFlightCreateRequest, error: ProviderError) -> Self::Verdict {
        TestCreateVerdict::Undetermined {
            token: token.into(),
            error: Box::new(error),
        }
    }

    fn preserve(self, error: ProviderError) -> Self::Verdict {
        TestCreateVerdict::ResumePreserved {
            identifier: None,
            error: Box::new(error),
        }
    }

    fn issued(self, identifier: AppliedIdentifier) -> Self::Resumed {
        TestIssuedCreate { identifier }
    }

    fn pending(
        self,
        token: CcCreateToken,
        poll_after_secs: u32,
    ) -> Result<Self::Step, Self::Verdict> {
        Ok(TestCreateStep::Pending {
            token,
            poll_after_secs,
            issued: None,
        })
    }

    fn resend(self) -> Self::Resent {
        TestCreateAttempt
    }
}

impl CcResumedCreate for TestIssuedCreate {
    type Step = TestCreateStep;
    type Verdict = TestCreateVerdict;

    fn identifier(&self) -> &AppliedIdentifier {
        &self.identifier
    }

    fn completed(self, identifier: AppliedIdentifier, state: AppliedState) -> Self::Step {
        TestCreateStep::Completed { identifier, state }
    }

    fn failed(
        self,
        identifier: AppliedIdentifier,
        observation: Observation,
        error: ProviderError,
    ) -> Self::Verdict {
        TestCreateVerdict::Created {
            identifier,
            observation: Box::new(observation),
            error: Box::new(error),
        }
    }

    fn discarded(self, error: ProviderError) -> Self::Verdict {
        TestCreateVerdict::Discarded(Box::new(error))
    }

    fn preserve(self, error: ProviderError) -> Self::Verdict {
        TestCreateVerdict::ResumePreserved {
            identifier: Some(self.identifier),
            error: Box::new(error),
        }
    }

    fn pending(
        self,
        token: CcCreateToken,
        poll_after_secs: u32,
    ) -> Result<Self::Step, Self::Verdict> {
        Ok(TestCreateStep::Pending {
            token,
            poll_after_secs,
            issued: Some(self.identifier),
        })
    }
}

fn vpc_resource() -> Resource {
    Resource::with_provider("awscc", "ec2.Vpc", "pending", None)
        .with_attribute("cidr_block", string("10.0.0.0/16"))
}

fn vpc_properties() -> JsonValue {
    json!({
        "CidrBlock": "10.0.0.0/16",
        "VpcId": VPC_ID,
        "EnableDnsSupport": true,
    })
}

fn vpc_update() -> (ResourceId, State, UpdatePatch) {
    let id = ResourceId::with_provider_identity("awscc", "ec2.Vpc", "pending", None);
    let from = State::existing(
        id.clone(),
        HashMap::from([
            ("cidr_block".to_string(), string("10.0.0.0/16")),
            ("enable_dns_support".to_string(), bool_(false)),
        ]),
    )
    .with_identifier(VPC_ID);
    let patch = UpdatePatch {
        ops: vec![PatchOp::Replace {
            key: "enable_dns_support".to_string(),
            value: bool_(true),
        }],
    };
    (id, from, patch)
}

async fn provider_with_script(
    entries: impl IntoIterator<Item = (&'static str, Vec<Reply>)>,
) -> (AwsccProvider, Arc<ScriptedCloudControl>) {
    provider_with_post_send_failures(entries, []).await
}

async fn provider_with_post_send_failures(
    entries: impl IntoIterator<Item = (&'static str, Vec<Reply>)>,
    failure_scripts: impl IntoIterator<Item = (&'static str, Vec<bool>)>,
) -> (AwsccProvider, Arc<ScriptedCloudControl>) {
    let service = Arc::new(ScriptedCloudControl::new(entries));
    let mock = MockAws::builder()
        .with_service(Arc::clone(&service))
        .build();
    let inner = mock.http_client();
    let failures = Arc::new(Mutex::new(
        failure_scripts
            .into_iter()
            .map(|(action, script)| (action.to_string(), PostSendFailureScript::new(script)))
            .collect::<HashMap<_, _>>(),
    ));
    let http_client: SharedHttpClient = http_client_fn(move |settings, components| {
        SharedHttpConnector::new(PostSendFailureConnector {
            inner: inner.http_connector(settings, components),
            failures: Arc::clone(&failures),
        })
    });
    // The SDK's standard retry quota is process-global for a service/region retry
    // partition. Give each provider a unique region, and therefore a unique default
    // CloudControl RetryPartition, so timeout-heavy parallel tests cannot drain one
    // another's retry tokens.
    let region = format!(
        "us-test-{}",
        NEXT_TEST_REGION.fetch_add(1, Ordering::Relaxed)
    );
    let config = aws_config::defaults(BehaviorVersion::latest())
        .http_client(http_client)
        .credentials_provider(mock.credentials_provider())
        .region(Region::new(region))
        .load()
        .await;
    (
        AwsccProvider::from_sdk_config(
            carina_provider_awscc::BoundedSdkConfig::new(config),
            &AwsccProviderConfig::default(),
        )
        .await,
        service,
    )
}

async fn provider_with_initialization_error() -> AwsccProvider {
    let cloudcontrol = Arc::new(ScriptedCloudControl::new([
        (
            "CreateResource",
            vec![mutation_reply_without_identifier("unexpected-create")],
        ),
        (
            "GetResourceRequestStatus",
            vec![status_reply("IN_PROGRESS", Some(VPC_ID), None)],
        ),
    ]));
    let mock = MockAws::builder()
        .with_service(cloudcontrol)
        .with_service(Arc::new(DeniedSts))
        .build();
    let config = aws_config::defaults(BehaviorVersion::latest())
        .http_client(mock.http_client())
        .credentials_provider(mock.credentials_provider())
        .region(Region::new("us-init-error-1"))
        .load()
        .await;
    let provider = AwsccProvider::from_sdk_config(
        BoundedSdkConfig::new(config),
        &AwsccProviderConfig {
            allowed_account_ids: vec!["123456789012".to_string()],
            ..AwsccProviderConfig::default()
        },
    )
    .await;
    assert!(provider.init_error().is_some());
    provider
}

fn request_client_tokens(service: &ScriptedCloudControl, action: &str) -> Vec<String> {
    service
        .bodies(action)
        .iter()
        .map(|body| {
            serde_json::from_str::<JsonValue>(body).expect("request body should be valid JSON")
                ["ClientToken"]
                .as_str()
                .expect("mutating request should include ClientToken")
                .to_string()
        })
        .collect()
}

fn create_token(value: JsonValue) -> CcCreateToken {
    serde_json::from_value(value).expect("deserialize test create token")
}

fn pending_create_for_token(
    resource: &Resource,
    token: &CcCreateToken,
) -> carina_core::provider::PendingCreate {
    let pending_token = PendingToken::new(
        serde_json::to_string(token).expect("serialize test Cloud Control create token"),
    );
    let attempt = CreateAttempt::for_test(resource.id.clone());
    match token.identifier() {
        Some(identifier) => attempt
            .issued(
                common::applied_identifier(identifier),
                Sent::AllRequestAttributes,
            )
            .pending(pending_token, 5),
        None => attempt.pending(pending_token, Sent::AllRequestAttributes, 5),
    }
}

async fn resume_request_for_token(
    resource: Resource,
    token: &CcCreateToken,
) -> ResumeCreateRequest {
    let pending = pending_create_for_token(&resource, token);
    let failure = pending.preserve(ProviderError::internal("construct test in-flight create"));
    let CreateFailureKind::InFlight { pending, error } = failure.into_kind() else {
        panic!("preserving a pending create must produce an in-flight create");
    };
    assert_eq!(error.to_string(), "construct test in-flight create");
    ResumeCreateRequest::for_test(pending, common::create_request(resource).await)
        .expect("test resume request must preserve its resource route")
}

#[test]
fn unconfirmed_create_without_client_token_is_rejected() {
    let error = serde_json::from_value::<CcCreateToken>(json!({
        "phase": "unconfirmed",
        "attempt": 0,
        "resends": 1,
    }))
    .expect_err("an unconfirmed create must never mint a replacement client token");

    assert_eq!(error.to_string(), "missing field `client_token`");
}

#[test]
fn legacy_in_flight_tokens_ignore_dead_client_token_data() {
    let request_phase = serde_json::from_value::<CcRequestPhase>(json!({
        "phase": "in_flight",
        "request": "update-request",
        "attempt": 3,
        "client_token": TEST_CLIENT_TOKEN,
    }))
    .expect("legacy update continuation must deserialize");
    assert_eq!(
        serde_json::to_value(request_phase).expect("serialize update continuation"),
        json!({
            "phase": "in_flight",
            "request": "update-request",
            "attempt": 3,
        })
    );

    let create_token = serde_json::from_value::<CcCreateToken>(json!({
        "phase": "in_flight",
        "request": "create-request",
        "attempt": 4,
        "identifier": VPC_ID,
        "client_token": TEST_CLIENT_TOKEN,
    }))
    .expect("legacy create continuation must deserialize");
    assert_eq!(
        serde_json::to_value(create_token).expect("serialize create continuation"),
        json!({
            "phase": "in_flight",
            "request": "create-request",
            "attempt": 4,
            "identifier": VPC_ID,
        })
    );
}

fn expect_update_pending(
    step: OperationStep<UpdateOutcome, ProviderError, CcUpdateToken>,
) -> (CcUpdateToken, u32) {
    match step {
        OperationStep::Pending {
            pending,
            poll_after_secs,
        } => (pending, poll_after_secs),
        OperationStep::Done(outcome) => panic!("expected pending update, got {outcome:?}"),
        OperationStep::Verdict(error) => panic!("expected pending update, got {error}"),
    }
}

fn expect_create_pending(step: TestCreateStep) -> (CcCreateToken, u32) {
    match step {
        TestCreateStep::Pending {
            token,
            poll_after_secs,
            issued,
        } => {
            let _ = issued.as_ref();
            (token, poll_after_secs)
        }
        TestCreateStep::Partial { identifier, reason } => {
            panic!("expected pending create, got partial {identifier:?}: {reason}")
        }
        TestCreateStep::Completed { identifier, state } => {
            panic!("expected pending create, got completed {identifier:?}: {state:?}")
        }
        other => panic!("expected pending create, got {other:?}"),
    }
}

fn test_create_poll(token: &CcCreateToken) -> CcCreatePoll<TestCreateAttempt, TestIssuedCreate> {
    match token {
        CcCreateToken::InFlight(create) if create.identifier().is_some() => {
            CcCreatePoll::Issued(TestIssuedCreate {
                identifier: common::applied_identifier(
                    create
                        .identifier()
                        .expect("guard requires an in-flight identifier"),
                ),
            })
        }
        CcCreateToken::NotIssued(_)
        | CcCreateToken::Unconfirmed(_)
        | CcCreateToken::InFlight(_) => CcCreatePoll::Unissued(TestCreateAttempt),
    }
}

fn in_flight_create_token(identifier: Option<&str>) -> CcCreateToken {
    let mut value = json!({
        "phase": "in_flight",
        "request": "create-resume",
        "attempt": 0,
    });
    if let Some(identifier) = identifier {
        value["identifier"] = json!(identifier);
    }
    create_token(value)
}

async fn poll_test_create(
    provider: &AwsccProvider,
    resource: &Resource,
    token: CcCreateToken,
) -> Result<TestCreateStep, TestCreateVerdict> {
    let poll = test_create_poll(&token);
    provider.poll_create_step(resource, token, poll).await
}

fn expect_delete_pending(
    step: OperationStep<(), ProviderError, CcDeleteToken>,
) -> (CcDeleteToken, u32) {
    match step {
        OperationStep::Pending {
            pending,
            poll_after_secs,
        } => (pending, poll_after_secs),
        OperationStep::Done(()) => panic!("expected pending delete"),
        OperationStep::Verdict(error) => panic!("expected pending delete, got {error}"),
    }
}

#[tokio::test]
async fn create_invalid_request_rejection_is_terminal_and_untracked() {
    let (provider, service) = provider_with_script([(
        "CreateResource",
        vec![Reply::Error {
            status: 400,
            code: "InvalidRequestException",
            message: "invalid create request",
        }],
    )])
    .await;
    let resource = common::normalize_resource(vpc_resource()).await;

    let failure = provider
        .create_step(resource.as_resource(), TestCreateAttempt)
        .await
        .expect_err("a rejected CreateResource call has no accepted create to track");

    let TestCreateVerdict::Untracked(error) = failure else {
        panic!("CreateResource rejection must be untracked");
    };
    assert!(error.to_string().contains("invalid create request"));
    assert_eq!(service.calls("CreateResource"), 1);
    assert_eq!(service.calls("GetResourceRequestStatus"), 0);
}

#[tokio::test]
async fn accepted_create_without_request_token_becomes_unconfirmed_and_resends_same_token() {
    for (label, request_token) in [("missing", None), ("empty", Some(""))] {
        let (provider, service) = provider_with_script([(
            "CreateResource",
            vec![
                mutation_reply_without_request_token(request_token),
                mutation_reply_without_identifier("create-after-missing-token"),
            ],
        )])
        .await;
        let resource = common::normalize_resource(vpc_resource()).await;

        let initial = provider
            .create_step(resource.as_resource(), TestCreateAttempt)
            .await
            .unwrap_or_else(|error| {
                panic!("accepted create with {label} request token must stay tracked: {error:?}")
            });
        let (token, poll_after_secs) = expect_create_pending(initial);
        let encoded = serde_json::to_value(&token).expect("serialize unconfirmed create token");
        assert_eq!(
            encoded["phase"], "unconfirmed",
            "{label} request token must produce an unconfirmed create"
        );
        assert_eq!(encoded["attempt"], 0);
        assert_eq!(encoded["resends"], 1);
        assert_eq!(poll_after_secs, 1);

        let resent = provider
            .poll_create_step(
                resource.as_resource(),
                token,
                CcCreatePoll::Unissued(TestCreateAttempt),
            )
            .await
            .unwrap_or_else(|error| panic!("same-token resend must recover tracking: {error:?}"));
        let (resent_token, resent_poll_after_secs) = expect_create_pending(resent);
        assert!(matches!(
            resent_token,
            CcCreateToken::InFlight(ref create)
                if create.request() == &RequestToken::from("create-after-missing-token")
                    && create.attempt() == 0
                    && create.identifier().is_none()
        ));
        assert_eq!(resent_poll_after_secs, 1);

        let client_tokens = request_client_tokens(&service, "CreateResource");
        assert_eq!(client_tokens.len(), 2);
        assert_eq!(client_tokens[0], client_tokens[1]);
        assert_eq!(encoded["client_token"], client_tokens[0]);
        assert_eq!(service.calls("GetResourceRequestStatus"), 0);
    }
}

#[tokio::test(start_paused = true)]
async fn unconfirmed_create_rejection_from_native_drive_preserves_in_flight_token() {
    let (provider, service) = provider_with_post_send_failures(
        [(
            "CreateResource",
            vec![
                mutation_reply_without_identifier("accepted-but-hidden"),
                mutation_reply_without_identifier("accepted-but-hidden"),
                mutation_reply_without_identifier("accepted-but-hidden"),
                invalid_request(),
            ],
        )],
        [("CreateResource", vec![true, false])],
    )
    .await;
    let resource = vpc_resource();
    let id = resource.id.clone();
    let request = common::create_request(resource).await;

    let failure = carina_core::executor::dispatch_create(&provider, &id, request)
        .await
        .expect_err("a rejected resend cannot disprove the earlier create send");
    let CreateFailureKind::InFlight { pending, error } = failure.into_kind() else {
        panic!("an unconfirmed create rejection must remain in flight");
    };
    assert_eq!(error.variant_name(), "api_error");
    assert_eq!(
        error.detail().context(),
        "Failed to resend create resource; the outcome of the earlier send is unknown"
    );

    let persisted: JsonValue = serde_json::from_str(pending.token().expose_for_provider_call())
        .expect("persisted Cloud Control create token must be valid JSON");
    let client_tokens = request_client_tokens(&service, "CreateResource");
    assert_eq!(client_tokens.len(), 4);
    assert_eq!(client_tokens, vec![client_tokens[0].clone(); 4]);
    assert_eq!(persisted["phase"], "unconfirmed");
    assert_eq!(persisted["attempt"], 0);
    assert_eq!(persisted["resends"], 1);
    assert_eq!(persisted["client_token"], client_tokens[0]);
    assert_eq!(service.calls("GetResourceRequestStatus"), 0);
}

#[tokio::test]
async fn unconfirmed_create_rejection_from_poll_preserves_in_flight_token() {
    let (provider, service) =
        provider_with_script([("CreateResource", vec![invalid_request()])]).await;
    let resource = common::normalize_resource(vpc_resource()).await;
    let token = serde_json::from_value::<CcCreateToken>(json!({
        "phase": "unconfirmed",
        "attempt": 2,
        "resends": 7,
        "client_token": TEST_CLIENT_TOKEN,
    }))
    .expect("deserialize unconfirmed create continuation");
    let pending = pending_create_for_token(resource.as_resource(), &token);
    let Poll::Unissued(poll) = pending.into_poll() else {
        panic!("an identifier-less pending create must produce an unissued poll");
    };

    let failure = provider
        .poll_create_step(
            resource.as_resource(),
            token.clone(),
            CcCreatePoll::Unissued(poll),
        )
        .await
        .expect_err("a rejected resend cannot disprove the earlier create send");
    let CreateFailureKind::InFlight { pending, error } = failure.into_kind() else {
        panic!("an unconfirmed poll rejection must remain in flight");
    };
    assert_eq!(error.variant_name(), "api_error");
    assert_eq!(
        error.detail().context(),
        "Failed to resend create resource; the outcome of the earlier send is unknown"
    );
    assert_eq!(
        pending.token().expose_for_provider_call(),
        serde_json::to_string(&token).expect("serialize expected create continuation")
    );
    assert_eq!(
        request_client_tokens(&service, "CreateResource"),
        [TEST_CLIENT_TOKEN]
    );
}

#[tokio::test]
async fn unconfirmed_create_rejection_from_resume_preserves_in_flight_token() {
    let (provider, service) =
        provider_with_script([("CreateResource", vec![client_token_conflict()])]).await;
    let resource = vpc_resource();
    let token = serde_json::from_value::<CcCreateToken>(json!({
        "phase": "unconfirmed",
        "attempt": 2,
        "resends": 7,
        "client_token": TEST_CLIENT_TOKEN,
    }))
    .expect("deserialize unconfirmed create continuation");
    let request = resume_request_for_token(resource, &token).await;

    let failure = carina_core::executor::dispatch_resume_create(&provider, request)
        .await
        .expect_err("a rejected resumed send cannot disprove the earlier create send");
    let ResumeFailureKind::InFlight { pending, error } = failure.into_kind() else {
        panic!("an unconfirmed resume rejection must remain in flight");
    };
    assert_eq!(error.variant_name(), "api_error");
    assert_eq!(
        error.detail().context(),
        "Failed to resend create resource; the outcome of the earlier send is unknown"
    );
    assert_eq!(
        pending.token().expose_for_provider_call(),
        serde_json::to_string(&token).expect("serialize expected create continuation")
    );
    assert_eq!(
        request_client_tokens(&service, "CreateResource"),
        [TEST_CLIENT_TOKEN]
    );
}

#[tokio::test]
async fn unconfirmed_create_pre_send_failure_from_resume_is_undetermined() {
    let (provider, service) =
        provider_with_script(std::iter::empty::<(&'static str, Vec<Reply>)>()).await;
    let resource = Resource::with_provider("awscc", "s3.Bucket", "invalid_resume", None)
        .with_attribute("bucket_name", string("carina-invalid-resume"))
        .with_attribute(
            "tags",
            Value::Concrete(ConcreteValue::Map(IndexMap::from([(
                "env".to_string(),
                bool_(true),
            )]))),
        );
    let token = serde_json::from_value::<CcCreateToken>(json!({
        "phase": "unconfirmed",
        "attempt": 2,
        "resends": 7,
        "client_token": TEST_CLIENT_TOKEN,
    }))
    .expect("deserialize unconfirmed create continuation");

    let failure = provider
        .resume_create_step(
            &resource,
            token.clone(),
            CcCreateResume::<TestCreateAttempt, TestIssuedCreate>::Unissued(TestCreateAttempt),
        )
        .await
        .expect_err("a pre-send failure cannot disprove the earlier create send");
    let TestCreateVerdict::Undetermined {
        token: preserved,
        error,
    } = failure
    else {
        panic!("an unconfirmed pre-send failure must be undetermined");
    };
    assert_eq!(preserved, token);
    assert_eq!(error.variant_name(), "invalid_input");
    assert_eq!(service.calls("CreateResource"), 0);
}

#[tokio::test]
async fn issued_poll_unconfirmed_resend_failure_preserves_token_and_error() {
    let (provider, service) =
        provider_with_script([("CreateResource", vec![client_token_conflict()])]).await;
    let resource = common::normalize_resource(vpc_resource()).await;
    let token = create_token(json!({
        "phase": "unconfirmed",
        "attempt": 2,
        "resends": 7,
        "client_token": TEST_CLIENT_TOKEN,
    }));
    let issued_identifier = vpc_identifier();

    let failure = provider
        .poll_create_step::<TestCreateAttempt>(
            resource.as_resource(),
            token.clone(),
            CcCreatePoll::Issued(TestIssuedCreate {
                identifier: issued_identifier.clone(),
            }),
        )
        .await
        .expect_err("a non-retryable resend failure must preserve the issued create");

    let TestCreateVerdict::Preserved {
        token: preserved,
        identifier,
        error,
    } = failure
    else {
        panic!("an issued create resend failure must use the preserving verdict");
    };
    assert_eq!(preserved, token);
    assert_eq!(identifier, issued_identifier);
    assert_eq!(error.variant_name(), "api_error");
    assert_eq!(
        error.detail().context(),
        "Failed to resend create resource; the outcome of the earlier send is unknown"
    );
    assert!(error.to_string().contains("client token reused"));
    assert_eq!(
        request_client_tokens(&service, "CreateResource"),
        [TEST_CLIENT_TOKEN]
    );
}

#[tokio::test(start_paused = true)]
async fn resume_initialization_error_preserves_unissued_and_issued_continuations() {
    let provider = provider_with_initialization_error().await;
    let initialization_error = provider
        .init_error()
        .expect("test provider must retain its initialization error")
        .to_string();
    let cases = [
        (
            "unissued",
            create_token(json!({
                "phase": "not_issued",
                "attempt": 0,
                "client_token": TEST_CLIENT_TOKEN,
            })),
            None,
        ),
        (
            "issued",
            create_token(json!({
                "phase": "in_flight",
                "request": "create-before-init-error",
                "attempt": 0,
                "identifier": VPC_ID,
            })),
            Some(vpc_identifier()),
        ),
    ];

    for (label, token, expected_identifier) in cases {
        let request = resume_request_for_token(vpc_resource(), &token).await;
        let failure = match carina_core::executor::dispatch_resume_create(&provider, request).await
        {
            Err(failure) => failure,
            Ok(outcome) => {
                panic!("{label} resume must preserve its initialization error, got {outcome:?}")
            }
        };
        let ResumeFailureKind::InFlight { pending, error } = failure.into_kind() else {
            panic!("{label} resume initialization failure must remain in flight");
        };
        assert_eq!(
            pending.token().expose_for_provider_call(),
            serde_json::to_string(&token).expect("serialize expected preserved token")
        );
        assert_eq!(pending.identifier(), expected_identifier.as_ref());
        assert_eq!(error.variant_name(), "invalid_input");
        assert_eq!(error.detail().context(), initialization_error.as_str());
        assert_eq!(error.detail().provider_name.as_deref(), Some("awscc"));
        assert_eq!(error.detail().resource_id.as_deref(), Some(pending.id()));
    }
}

#[tokio::test]
async fn resume_unknown_resource_type_preserves_unissued_and_issued_witnesses() {
    let (provider, service) =
        provider_with_script(std::iter::empty::<(&'static str, Vec<Reply>)>()).await;
    let resource = Resource::with_provider("awscc", "unknown.Resource", "pending", None);
    let issued_identifier = common::applied_identifier("unknown-identifier");
    let cases = [
        (
            "unissued",
            create_token(json!({
                "phase": "not_issued",
                "attempt": 0,
                "client_token": TEST_CLIENT_TOKEN,
            })),
            CcCreateResume::Unissued(TestCreateAttempt),
            None,
        ),
        (
            "issued",
            create_token(json!({
                "phase": "in_flight",
                "request": "unknown-resource-create",
                "attempt": 0,
                "identifier": issued_identifier.as_str(),
            })),
            CcCreateResume::Issued(TestIssuedCreate {
                identifier: issued_identifier.clone(),
            }),
            Some(issued_identifier.clone()),
        ),
    ];

    for (label, token, resume, expected_identifier) in cases {
        let failure = match provider.resume_create_step(&resource, token, resume).await {
            Err(failure) => failure,
            Ok(step) => {
                panic!("{label} unknown-type resume must preserve its error, got {step:?}")
            }
        };
        let TestCreateVerdict::ResumePreserved { identifier, error } = failure else {
            panic!("{label} unknown-type resume must use the preserving verdict");
        };
        assert_eq!(identifier, expected_identifier);
        assert_eq!(error.variant_name(), "internal");
        assert_eq!(
            error.detail().context(),
            "Unknown resource type: unknown.Resource"
        );
        assert_eq!(error.detail().resource_id.as_deref(), Some(&resource.id));
    }
    assert_eq!(service.actions(), Vec::<String>::new());
}

#[tokio::test]
async fn progress_identifier_transitions_pending_create_to_issued() {
    let (provider, service) = provider_with_script([
        (
            "CreateResource",
            vec![mutation_reply_without_identifier("create-0")],
        ),
        (
            "GetResourceRequestStatus",
            vec![status_reply("IN_PROGRESS", Some(VPC_ID), None)],
        ),
    ])
    .await;
    let resource = common::normalize_resource(vpc_resource()).await;

    let initial = provider
        .create_step(resource.as_resource(), TestCreateAttempt)
        .await
        .expect("accepted create without an identifier stays pending");
    let TestCreateStep::Pending {
        token,
        issued: None,
        ..
    } = initial
    else {
        panic!("the initial response has no identifier and must stay unissued");
    };

    let next = poll_test_create(&provider, resource.as_resource(), token)
        .await
        .expect("in-progress status with an identifier remains pending");
    let TestCreateStep::Pending {
        token: CcCreateToken::InFlight(create),
        issued: Some(issued_identifier),
        ..
    } = next
    else {
        panic!("the first progress identifier must consume the issued transition");
    };
    assert_eq!(create.identifier(), Some(VPC_ID));
    assert_eq!(issued_identifier.as_str(), VPC_ID);
    assert_eq!(
        service.actions(),
        ["CreateResource", "GetResourceRequestStatus"]
    );
}

#[tokio::test(start_paused = true)]
async fn accepted_in_progress_create_over_budget_preserves_token_and_identifier() {
    let (provider, service) = provider_with_script([
        (
            "CreateResource",
            vec![mutation_reply("create-budget", VPC_ID)],
        ),
        (
            "GetResourceRequestStatus",
            vec![status_reply("IN_PROGRESS", Some(VPC_ID), None)],
        ),
    ])
    .await;
    let resource = vpc_resource();
    let id = resource.id.clone();
    let request = common::create_request(resource).await;

    let create = tokio::spawn(async move {
        carina_core::executor::dispatch_create(&provider, &id, request).await
    });
    for _ in 0..32 {
        if service.calls("CreateResource") == 1 {
            break;
        }
        tokio::task::yield_now().await;
    }
    assert_eq!(service.calls("CreateResource"), 1);
    for _ in 0..1024 {
        if create.is_finished() {
            break;
        }
        tokio::time::advance(std::time::Duration::from_secs(60)).await;
        for _ in 0..8 {
            tokio::task::yield_now().await;
        }
    }
    assert!(
        create.is_finished(),
        "budgeted create should have timed out; actions: {:?}",
        service.actions()
    );
    let failure = create
        .await
        .expect("create task should not panic")
        .expect_err("an accepted request that exceeds the budget remains in flight");
    let CreateFailureKind::InFlight { pending, error } = failure.into_kind() else {
        panic!("budget exhaustion must preserve the accepted create");
    };
    assert_eq!(
        pending.identifier().map(AppliedIdentifier::as_str),
        Some(VPC_ID)
    );
    let token: CcCreateToken = serde_json::from_str(pending.token().expose_for_provider_call())
        .expect("persisted CloudControl token should decode");
    assert!(matches!(
        token,
        CcCreateToken::InFlight(create)
            if create.request() == &RequestToken::from("create-budget")
                && create.attempt() == 0
                && create.identifier() == Some(VPC_ID)
    ));
    assert_eq!(error.variant_name(), "timeout");
    assert_eq!(service.calls("CreateResource"), 1);
    assert_eq!(service.calls("GetResourceRequestStatus"), 15);
    assert_eq!(service.calls("GetResource"), 0);
}

#[tokio::test(start_paused = true)]
async fn status_sdk_error_re_pends_the_issued_create() {
    let (provider, service) = provider_with_script([(
        "GetResourceRequestStatus",
        vec![Reply::Error {
            status: 500,
            code: "InternalFailure",
            message: "status temporarily unavailable",
        }],
    )])
    .await;
    let resource = common::normalize_resource(vpc_resource()).await;
    let token = in_flight_create_token(Some(VPC_ID));

    let next = poll_test_create(&provider, resource.as_resource(), token.clone())
        .await
        .expect("status SDK errors must keep an issued request in flight");
    let TestCreateStep::Pending {
        token: next_token,
        issued: Some(identifier),
        poll_after_secs,
    } = next
    else {
        panic!("status SDK error must produce an issued pending step");
    };
    assert_eq!(next_token, token);
    assert_eq!(identifier.as_str(), VPC_ID);
    assert_eq!(poll_after_secs, 5);
    assert_eq!(service.calls("GetResource"), 0);
    assert_eq!(service.calls("GetResourceRequestStatus"), 3);
}

#[tokio::test]
async fn resume_in_flight_polls_status_before_any_read() {
    let (provider, service) = provider_with_script([
        (
            "GetResourceRequestStatus",
            vec![status_reply("IN_PROGRESS", Some(VPC_ID), None)],
        ),
        (
            "GetResource",
            vec![get_resource_reply(VPC_TYPE, VPC_ID, vpc_properties())],
        ),
    ])
    .await;
    let resource = common::normalize_resource(vpc_resource()).await;
    let token = in_flight_create_token(Some(VPC_ID));

    let step = provider
        .resume_create_step(
            resource.as_resource(),
            token.clone(),
            CcCreateResume::<TestCreateAttempt, TestIssuedCreate>::Issued(TestIssuedCreate {
                identifier: vpc_identifier(),
            }),
        )
        .await
        .expect("in-progress resume must remain pending");

    let TestCreateStep::Pending {
        token: next_token,
        issued: Some(identifier),
        ..
    } = step
    else {
        panic!("in-progress resume must remain issued and pending");
    };
    assert_eq!(next_token, token);
    assert_eq!(identifier.as_str(), VPC_ID);
    assert_eq!(service.actions(), ["GetResourceRequestStatus"]);
}

#[tokio::test]
async fn resume_success_then_found_completes_after_status_poll() {
    let (provider, service) = provider_with_script([
        (
            "GetResourceRequestStatus",
            vec![status_reply("SUCCESS", Some(VPC_ID), None)],
        ),
        (
            "GetResource",
            vec![get_resource_reply(
                VPC_TYPE,
                VPC_ID,
                json!({"VpcId": VPC_ID}),
            )],
        ),
    ])
    .await;
    let resource = common::normalize_resource(vpc_resource()).await;

    let step = provider
        .resume_create_step(
            resource.as_resource(),
            in_flight_create_token(Some(VPC_ID)),
            CcCreateResume::<TestCreateAttempt, TestIssuedCreate>::Issued(TestIssuedCreate {
                identifier: vpc_identifier(),
            }),
        )
        .await
        .expect("terminal success with a found resource completes resume");

    let TestCreateStep::Completed { identifier, state } = step else {
        panic!("resume success plus found read must complete");
    };
    assert_eq!(identifier.as_str(), VPC_ID);
    assert_eq!(state.identifier().as_str(), VPC_ID);
    assert!(
        !state.attributes().contains_key("cidr_block"),
        "completed resume must carry the read observation, not desired values"
    );
    assert_eq!(
        service.actions(),
        ["GetResourceRequestStatus", "GetResource"]
    );
}

#[tokio::test]
async fn resume_success_then_not_found_is_created_failure_not_completion() {
    let (provider, service) = provider_with_script([
        (
            "GetResourceRequestStatus",
            vec![status_reply("SUCCESS", Some(VPC_ID), None)],
        ),
        ("GetResource", vec![not_found()]),
    ])
    .await;
    let resource = common::normalize_resource(vpc_resource()).await;

    let failure = provider
        .resume_create_step(
            resource.as_resource(),
            in_flight_create_token(Some(VPC_ID)),
            CcCreateResume::<TestCreateAttempt, TestIssuedCreate>::Issued(TestIssuedCreate {
                identifier: vpc_identifier(),
            }),
        )
        .await
        .expect_err("terminal success plus NotFound must not be reported completed");

    let TestCreateVerdict::Created {
        identifier,
        observation,
        ..
    } = failure
    else {
        panic!("terminal success plus NotFound must preserve the identifier");
    };
    assert_eq!(identifier.as_str(), VPC_ID);
    assert!(matches!(
        observation.as_ref(),
        Observation::Read(ReadOutcome::NotFound { .. })
    ));
    assert_eq!(
        service.actions(),
        ["GetResourceRequestStatus", "GetResource"]
    );
}

#[tokio::test]
async fn resume_failed_with_identifier_returns_created_failure_with_observation() {
    let (provider, service) = provider_with_script([
        (
            "GetResourceRequestStatus",
            vec![status_reply(
                "FAILED",
                Some(VPC_ID),
                Some("handler rejected stabilization"),
            )],
        ),
        (
            "GetResource",
            vec![get_resource_reply(VPC_TYPE, VPC_ID, vpc_properties())],
        ),
    ])
    .await;
    let resource = common::normalize_resource(vpc_resource()).await;

    let failure = provider
        .resume_create_step(
            resource.as_resource(),
            in_flight_create_token(Some(VPC_ID)),
            CcCreateResume::<TestCreateAttempt, TestIssuedCreate>::Issued(TestIssuedCreate {
                identifier: vpc_identifier(),
            }),
        )
        .await
        .expect_err("FAILED with an identifier must never become success");

    let TestCreateVerdict::Created {
        identifier,
        observation,
        error,
    } = failure
    else {
        panic!("FAILED with an identifier must preserve a created verdict");
    };
    assert_eq!(identifier.as_str(), VPC_ID);
    let Observation::Read(ReadOutcome::Found { state }) = observation.as_ref() else {
        panic!("FAILED read-back must be retained as the created observation");
    };
    assert_eq!(state.identifier().as_str(), VPC_ID);
    assert!(error.to_string().contains("handler rejected stabilization"));
    assert_eq!(
        service.actions(),
        ["GetResourceRequestStatus", "GetResource"]
    );
}

#[tokio::test]
async fn resume_expired_token_with_identifier_and_not_found_is_discarded() {
    let (provider, service) = provider_with_script([
        (
            "GetResourceRequestStatus",
            vec![Reply::Error {
                status: 400,
                code: "RequestTokenNotFoundException",
                message: "request token expired",
            }],
        ),
        ("GetResource", vec![not_found()]),
    ])
    .await;
    let resource = common::normalize_resource(vpc_resource()).await;
    let token = in_flight_create_token(Some(VPC_ID));

    let failure = provider
        .resume_create_step(
            resource.as_resource(),
            token,
            CcCreateResume::<TestCreateAttempt, TestIssuedCreate>::Issued(TestIssuedCreate {
                identifier: vpc_identifier(),
            }),
        )
        .await
        .expect_err("an expired token plus NotFound proves there is no create to keep");

    let TestCreateVerdict::Discarded(error) = failure else {
        panic!("expired token plus NotFound must discard the in-flight create");
    };
    assert!(error.to_string().contains("no longer known"));
    assert_eq!(
        service.actions(),
        ["GetResourceRequestStatus", "GetResource"]
    );
}

#[tokio::test]
async fn poll_expired_token_with_identifier_and_not_found_is_discarded() {
    let (provider, service) = provider_with_script([
        (
            "GetResourceRequestStatus",
            vec![Reply::Error {
                status: 400,
                code: "RequestTokenNotFoundException",
                message: "request token expired",
            }],
        ),
        ("GetResource", vec![not_found()]),
    ])
    .await;
    let resource = common::normalize_resource(vpc_resource()).await;

    let failure = poll_test_create(
        &provider,
        resource.as_resource(),
        in_flight_create_token(Some(VPC_ID)),
    )
    .await
    .expect_err("an expired token plus NotFound must not be polled forever");

    let TestCreateVerdict::Discarded(error) = failure else {
        panic!("expired token plus NotFound must discard the pending create");
    };
    assert!(error.to_string().contains("no longer known"));
    assert_eq!(
        service.actions(),
        ["GetResourceRequestStatus", "GetResource"]
    );
}

#[tokio::test]
async fn resume_expired_token_without_identifier_is_undetermined() {
    let (provider, service) = provider_with_script([(
        "GetResourceRequestStatus",
        vec![Reply::Error {
            status: 400,
            code: "RequestTokenNotFoundException",
            message: "request token expired",
        }],
    )])
    .await;
    let resource = common::normalize_resource(vpc_resource()).await;

    let failure = provider
        .resume_create_step(
            resource.as_resource(),
            in_flight_create_token(None),
            CcCreateResume::<TestCreateAttempt, TestIssuedCreate>::Unissued(TestCreateAttempt),
        )
        .await
        .expect_err("an unknown request with no identifier is undetermined");

    let TestCreateVerdict::Undetermined { token, error } = failure else {
        panic!("expired token without an identifier must be undetermined");
    };
    assert_eq!(token, in_flight_create_token(None));
    assert!(error.to_string().contains("no longer known"));
    assert_eq!(service.calls("GetResource"), 0);
}

#[tokio::test(start_paused = true)]
async fn resume_status_sdk_error_without_identifier_re_pends_with_backoff() {
    let (provider, service) = provider_with_script([(
        "GetResourceRequestStatus",
        vec![Reply::Error {
            status: 500,
            code: "InternalFailure",
            message: "status temporarily unavailable",
        }],
    )])
    .await;
    let resource = common::normalize_resource(vpc_resource()).await;

    let token = in_flight_create_token(None);
    let step = provider
        .resume_create_step(
            resource.as_resource(),
            token.clone(),
            CcCreateResume::<TestCreateAttempt, TestIssuedCreate>::Unissued(TestCreateAttempt),
        )
        .await
        .expect("status SDK errors must keep an accepted request pending");

    let TestCreateStep::Pending {
        token: next_token,
        poll_after_secs,
        issued: None,
    } = step
    else {
        panic!("unissued resume status error must produce an unissued pending step");
    };
    assert_eq!(next_token, token);
    assert_eq!(poll_after_secs, 5);
    assert_eq!(service.calls("GetResource"), 0);
    assert_eq!(service.calls("GetResourceRequestStatus"), 3);
}

#[tokio::test]
async fn resume_not_issued_uses_resend_attempt() {
    let (provider, service) = provider_with_script([(
        "CreateResource",
        vec![mutation_reply("create-resent", VPC_ID)],
    )])
    .await;
    let resource = common::normalize_resource(vpc_resource()).await;

    let step = provider
        .resume_create_step(
            resource.as_resource(),
            create_token(json!({
                "phase": "not_issued",
                "attempt": 1,
                "client_token": TEST_CLIENT_TOKEN,
            })),
            CcCreateResume::<TestCreateAttempt, TestIssuedCreate>::Unissued(TestCreateAttempt),
        )
        .await
        .expect("NotIssued resume must resend the desired state");

    let TestCreateStep::Pending {
        token: CcCreateToken::InFlight(create),
        issued: Some(issued),
        ..
    } = step
    else {
        panic!("resent accepted create must become issued pending");
    };
    assert_eq!(create.request(), &RequestToken::from("create-resent"));
    assert_eq!(create.attempt(), 1);
    assert_eq!(create.identifier(), Some(VPC_ID));
    assert_eq!(issued.as_str(), VPC_ID);
    assert_eq!(service.calls("CreateResource"), 1);
    assert_eq!(service.calls("GetResourceRequestStatus"), 0);
}

#[tokio::test]
async fn resume_cancel_complete_with_identifier_reads_back_created_failure() {
    let (provider, service) = provider_with_script([
        (
            "GetResourceRequestStatus",
            vec![status_reply(
                "CANCEL_COMPLETE",
                Some(VPC_ID),
                Some("request canceled"),
            )],
        ),
        (
            "GetResource",
            vec![get_resource_reply(VPC_TYPE, VPC_ID, vpc_properties())],
        ),
    ])
    .await;
    let resource = common::normalize_resource(vpc_resource()).await;

    let failure = provider
        .resume_create_step(
            resource.as_resource(),
            in_flight_create_token(Some(VPC_ID)),
            CcCreateResume::<TestCreateAttempt, TestIssuedCreate>::Issued(TestIssuedCreate {
                identifier: vpc_identifier(),
            }),
        )
        .await
        .expect_err("a canceled create with an identifier may have created a resource");

    let TestCreateVerdict::Created {
        identifier,
        observation,
        error,
    } = failure
    else {
        panic!("CANCEL_COMPLETE with an identifier must preserve a created verdict");
    };
    assert_eq!(identifier.as_str(), VPC_ID);
    let Observation::Read(ReadOutcome::Found { state }) = observation.as_ref() else {
        panic!("CANCEL_COMPLETE read-back must be retained as the created observation");
    };
    assert_eq!(state.identifier().as_str(), VPC_ID);
    assert!(error.to_string().contains("request canceled"));
    assert_eq!(
        service.actions(),
        ["GetResourceRequestStatus", "GetResource"]
    );
}

#[tokio::test]
async fn pending_statuses_keep_the_same_create_token_and_success_reads_back() {
    let (provider, service) = provider_with_script([
        ("CreateResource", vec![mutation_reply("create-0", VPC_ID)]),
        (
            "GetResourceRequestStatus",
            vec![
                status_reply("PENDING", Some(VPC_ID), None),
                status_reply("IN_PROGRESS", Some(VPC_ID), None),
                status_reply("SUCCESS", Some(VPC_ID), None),
            ],
        ),
        (
            "GetResource",
            vec![get_resource_reply(VPC_TYPE, VPC_ID, vpc_properties())],
        ),
    ])
    .await;
    let resource = common::normalize_resource(vpc_resource()).await;

    let (mut token, delay) = expect_create_pending(
        provider
            .create_step(resource.as_resource(), TestCreateAttempt)
            .await
            .expect("issue create"),
    );
    assert_eq!(delay, 1);
    assert!(matches!(
        &token,
        CcCreateToken::InFlight(create)
            if create.request() == &RequestToken::from("create-0")
                && create.attempt() == 0
                && create.identifier() == Some(VPC_ID)
    ));
    let expected = token.clone();

    for expected_status in ["PENDING", "IN_PROGRESS"] {
        let (next, delay) = expect_create_pending(
            poll_test_create(&provider, resource.as_resource(), token)
                .await
                .unwrap_or_else(|error| panic!("poll {expected_status}: {error:?}")),
        );
        assert_eq!(next, expected);
        assert_eq!(delay, 5);
        token = next;
    }

    let state = match poll_test_create(&provider, resource.as_resource(), token)
        .await
        .expect("poll successful create")
    {
        TestCreateStep::Succeeded {
            identifier,
            attributes,
        } => AppliedState::from_identifier(resource.id.clone(), identifier, attributes),
        other => panic!("expected successful create, got {other:?}"),
    };
    assert_eq!(state.identifier().as_str(), VPC_ID);
    assert_eq!(service.calls("CreateResource"), 1);
    assert_eq!(service.calls("GetResourceRequestStatus"), 3);
    assert_eq!(service.calls("GetResource"), 1);
}

#[tokio::test(start_paused = true)]
async fn create_throttling_retries_with_the_same_client_token() {
    let (provider, service) = provider_with_script([(
        "CreateResource",
        vec![
            throttled(),
            throttled(),
            throttled(),
            mutation_reply("create-1", VPC_ID),
        ],
    )])
    .await;
    let resource = common::normalize_resource(vpc_resource()).await;

    let (token, delay) = expect_create_pending(
        provider
            .create_step(resource.as_resource(), TestCreateAttempt)
            .await
            .expect("retryable initial create"),
    );
    assert!(matches!(
        &token,
        CcCreateToken::NotIssued(create) if create.attempt() == 1
    ));
    assert_eq!(delay, 10);
    assert_eq!(service.calls("CreateResource"), 3);

    let (token, delay) = expect_create_pending(
        poll_test_create(&provider, resource.as_resource(), token)
            .await
            .expect("re-send create"),
    );
    assert!(matches!(
        token,
        CcCreateToken::InFlight(create)
            if create.request() == &RequestToken::from("create-1")
                && create.attempt() == 1
                && create.identifier() == Some(VPC_ID)
    ));
    assert_eq!(delay, 1);
    assert_eq!(service.calls("CreateResource"), 4);

    let bodies = service.bodies("CreateResource");
    let requests = bodies
        .iter()
        .map(|body| serde_json::from_str::<JsonValue>(body).unwrap())
        .collect::<Vec<_>>();
    assert_eq!(requests.len(), 4);
    assert_eq!(requests[0]["DesiredState"], requests[3]["DesiredState"]);
    let first_client_token = requests[0]["ClientToken"]
        .as_str()
        .expect("the first create send must include a ClientToken");
    let retried_client_token = requests[3]["ClientToken"]
        .as_str()
        .expect("an in-run resend must include a ClientToken");
    assert!(common::is_uuidish(first_client_token));
    assert_eq!(retried_client_token, first_client_token);
}

#[tokio::test(start_paused = true)]
async fn resume_not_issued_reuses_original_create_client_token() {
    let (provider, service) = provider_with_script([(
        "CreateResource",
        vec![
            throttled(),
            throttled(),
            throttled(),
            mutation_reply("create-resent", VPC_ID),
        ],
    )])
    .await;
    let resource = common::normalize_resource(vpc_resource()).await;

    let (token, _) = expect_create_pending(
        provider
            .create_step(resource.as_resource(), TestCreateAttempt)
            .await
            .expect("retryable initial create"),
    );
    assert!(matches!(
        &token,
        CcCreateToken::NotIssued(create) if create.attempt() == 1
    ));

    let step = provider
        .resume_create_step(
            resource.as_resource(),
            token,
            CcCreateResume::<TestCreateAttempt, TestIssuedCreate>::Unissued(TestCreateAttempt),
        )
        .await
        .expect("NotIssued resume must resend the desired state");
    assert!(matches!(step, TestCreateStep::Pending { .. }));

    let requests = service
        .bodies("CreateResource")
        .iter()
        .map(|body| serde_json::from_str::<JsonValue>(body).unwrap())
        .collect::<Vec<_>>();
    assert_eq!(requests.len(), 4);
    let first_client_token = requests[0]["ClientToken"]
        .as_str()
        .expect("the first create send must include a ClientToken");
    let resumed_client_token = requests[3]["ClientToken"]
        .as_str()
        .expect("a cross-run resend must include a ClientToken");
    assert!(common::is_uuidish(first_client_token));
    assert_eq!(resumed_client_token, first_client_token);
}

#[tokio::test]
async fn retryable_failed_status_reissues_with_incremented_attempt_and_backoff() {
    let (provider, service) = provider_with_script([
        (
            "CreateResource",
            vec![
                mutation_reply_without_identifier("create-0"),
                mutation_reply("create-1", VPC_ID),
            ],
        ),
        (
            "GetResourceRequestStatus",
            vec![status_reply(
                "FAILED",
                None,
                Some("IpamPool is missing a source resource"),
            )],
        ),
    ])
    .await;
    let resource = common::normalize_resource(vpc_resource()).await;
    let (token, _) = expect_create_pending(
        provider
            .create_step(resource.as_resource(), TestCreateAttempt)
            .await
            .expect("issue create"),
    );

    let (token, delay) = expect_create_pending(
        poll_test_create(&provider, resource.as_resource(), token)
            .await
            .expect("retry failed create"),
    );
    assert!(matches!(
        &token,
        CcCreateToken::NotIssued(create) if create.attempt() == 1
    ));
    assert_eq!(delay, 10);
    assert_eq!(service.calls("GetResourceRequestStatus"), 1);
    assert_eq!(service.calls("CreateResource"), 1);

    let (token, delay) = expect_create_pending(
        poll_test_create(&provider, resource.as_resource(), token)
            .await
            .expect("re-issue failed create after backoff"),
    );
    assert!(matches!(
        token,
        CcCreateToken::InFlight(create)
            if create.request() == &RequestToken::from("create-1")
                && create.attempt() == 1
                && create.identifier() == Some(VPC_ID)
    ));
    assert_eq!(delay, 1);
    assert_eq!(service.calls("CreateResource"), 2);
    let client_tokens = request_client_tokens(&service, "CreateResource");
    assert_eq!(client_tokens.len(), 2);
    assert_ne!(client_tokens[0], client_tokens[1]);
}

#[tokio::test(start_paused = true)]
async fn create_one_maybe_sent_then_uncapped_throttling_stays_pending() {
    let (provider, service) = provider_with_post_send_failures(
        [(
            "CreateResource",
            vec![
                mutation_reply_without_identifier("never-returned"),
                mutation_reply_without_identifier("never-returned"),
                mutation_reply_without_identifier("never-returned"),
                throttled(),
            ],
        )],
        [("CreateResource", vec![true, false])],
    )
    .await;
    let resource = common::normalize_resource(vpc_resource()).await;

    let (mut token, delay) = expect_create_pending(
        provider
            .create_step(resource.as_resource(), TestCreateAttempt)
            .await
            .expect("first outcome-unknown create should remain pending"),
    );
    let encoded = serde_json::to_value(&token).expect("serialize create continuation");
    assert_eq!(encoded["phase"], "unconfirmed");
    assert_eq!(encoded["attempt"], 0);
    assert_eq!(encoded["resends"], 1);
    assert_eq!(delay, 10);

    for expected_resends in 2..=14 {
        let (next, delay) = expect_create_pending(
            poll_test_create(&provider, resource.as_resource(), token)
                .await
                .unwrap_or_else(|error| {
                    panic!(
                        "throttled unconfirmed create resend {expected_resends} was lost: {error:?}"
                    )
                }),
        );
        let encoded = serde_json::to_value(&next).expect("serialize create continuation");
        assert_eq!(encoded["phase"], "unconfirmed");
        assert_eq!(encoded["attempt"], 0);
        assert_eq!(encoded["resends"], expected_resends);
        let expected_delay = match expected_resends {
            2 => 20,
            3 => 40,
            4 => 80,
            5..=14 => 120,
            _ => unreachable!("test resend range is exact"),
        };
        assert_eq!(delay, expected_delay);
        token = next;
    }

    let client_tokens = request_client_tokens(&service, "CreateResource");
    assert_eq!(client_tokens.len(), 42);
    assert_eq!(client_tokens, vec![client_tokens[0].clone(); 42]);
}

#[tokio::test(start_paused = true)]
async fn create_repeated_post_send_timeouts_past_max_attempts_stay_pending() {
    let (provider, service) = provider_with_post_send_failures(
        [(
            "CreateResource",
            vec![mutation_reply_without_identifier("never-returned")],
        )],
        [("CreateResource", vec![true])],
    )
    .await;
    let resource = common::normalize_resource(vpc_resource()).await;

    let (mut token, delay) = expect_create_pending(
        provider
            .create_step(resource.as_resource(), TestCreateAttempt)
            .await
            .expect("first post-send timeout must remain pending"),
    );
    let encoded = serde_json::to_value(&token).expect("serialize create continuation");
    assert_eq!(encoded["phase"], "unconfirmed");
    assert_eq!(encoded["attempt"], 0);
    assert_eq!(encoded["resends"], 1);
    assert_eq!(delay, 10);

    for expected_resends in 2..=14 {
        let (next, delay) = expect_create_pending(
            poll_test_create(&provider, resource.as_resource(), token)
                .await
                .unwrap_or_else(|error| {
                    panic!(
                        "post-send timeout on create resend {expected_resends} was lost: {error:?}"
                    )
                }),
        );
        let encoded = serde_json::to_value(&next).expect("serialize create continuation");
        assert_eq!(encoded["phase"], "unconfirmed");
        assert_eq!(encoded["attempt"], 0);
        assert_eq!(encoded["resends"], expected_resends);
        let expected_delay = match expected_resends {
            2 => 20,
            3 => 40,
            4 => 80,
            5..=14 => 120,
            _ => unreachable!("test resend range is exact"),
        };
        assert_eq!(delay, expected_delay);
        token = next;
    }

    let client_tokens = request_client_tokens(&service, "CreateResource");
    assert_eq!(client_tokens.len(), 42);
    assert_eq!(client_tokens, vec![client_tokens[0].clone(); 42]);
}

#[tokio::test(start_paused = true)]
async fn unconfirmed_resends_do_not_consume_failed_status_retry_budget() {
    let mut create_replies = vec![
        mutation_reply_without_identifier("accepted-but-hidden"),
        mutation_reply_without_identifier("accepted-but-hidden"),
        mutation_reply_without_identifier("accepted-but-hidden"),
    ];
    create_replies.extend((0..39).map(|_| throttled()));
    create_replies.push(mutation_reply_without_identifier("accepted-after-resends"));
    let (provider, service) = provider_with_post_send_failures(
        [
            ("CreateResource", create_replies),
            (
                "GetResourceRequestStatus",
                vec![status_reply(
                    "FAILED",
                    None,
                    Some("IpamPool is missing a source resource"),
                )],
            ),
        ],
        [("CreateResource", vec![true, false])],
    )
    .await;
    let resource = common::normalize_resource(vpc_resource()).await;

    let (mut token, delay) = expect_create_pending(
        provider
            .create_step(resource.as_resource(), TestCreateAttempt)
            .await
            .expect("first outcome-unknown create must remain pending"),
    );
    assert_eq!(delay, 10);
    for expected_resends in 2..=14 {
        let (next, _) = expect_create_pending(
            poll_test_create(&provider, resource.as_resource(), token)
                .await
                .unwrap_or_else(|error| {
                    panic!("unconfirmed create resend {expected_resends} was lost: {error:?}")
                }),
        );
        let encoded = serde_json::to_value(&next).expect("serialize create continuation");
        assert_eq!(encoded["attempt"], 0);
        assert_eq!(encoded["resends"], expected_resends);
        token = next;
    }

    let (token, delay) = expect_create_pending(
        poll_test_create(&provider, resource.as_resource(), token)
            .await
            .expect("the accepted resend must become an in-flight create"),
    );
    assert_eq!(delay, 1);
    assert!(matches!(
        &token,
        CcCreateToken::InFlight(create)
            if create.request() == &RequestToken::from("accepted-after-resends")
                && create.attempt() == 0
                && create.identifier().is_none()
    ));

    let (token, delay) = expect_create_pending(
        poll_test_create(&provider, resource.as_resource(), token)
            .await
            .expect("the first retryable FAILED status must retain the full retry budget"),
    );
    assert!(matches!(
        token,
        CcCreateToken::NotIssued(create) if create.attempt() == 1
    ));
    assert_eq!(delay, 10);
    assert_eq!(service.calls("CreateResource"), 43);
    assert_eq!(service.calls("GetResourceRequestStatus"), 1);
}

#[tokio::test]
async fn failed_status_retry_follows_message_not_handler_error_code() {
    let (provider, service) = provider_with_script([(
        "GetResourceRequestStatus",
        vec![status_reply_with_error_code(
            "FAILED",
            None,
            Some("NotStabilized"),
            Some("invalid configuration"),
        )],
    )])
    .await;
    let resource = common::normalize_resource(vpc_resource()).await;

    let failure = poll_test_create(
        &provider,
        resource.as_resource(),
        in_flight_create_token(None),
    )
    .await
    .expect_err("a retryable-looking code must not override a permanent message");
    assert!(matches!(failure, TestCreateVerdict::Untracked(_)));
    assert_eq!(service.calls("CreateResource"), 0);

    let (provider, service) = provider_with_script([(
        "GetResourceRequestStatus",
        vec![status_reply_with_error_code(
            "FAILED",
            None,
            Some("InvalidRequest"),
            Some("IpamPool is missing a source resource"),
        )],
    )])
    .await;
    let resource = common::normalize_resource(vpc_resource()).await;

    let (token, delay) = expect_create_pending(
        poll_test_create(
            &provider,
            resource.as_resource(),
            in_flight_create_token(None),
        )
        .await
        .expect("a known retryable message must override a permanent-looking code"),
    );
    assert!(matches!(
        token,
        CcCreateToken::NotIssued(create) if create.attempt() == 1
    ));
    assert_eq!(delay, 10);
    assert_eq!(service.calls("CreateResource"), 0);
}

#[tokio::test]
async fn retryable_failed_create_with_identifier_is_created_without_reissue() {
    let (provider, service) = provider_with_script([
        ("CreateResource", vec![mutation_reply("create-0", VPC_ID)]),
        (
            "GetResourceRequestStatus",
            vec![status_reply("FAILED", Some(VPC_ID), Some("Throttling"))],
        ),
        ("GetResource", vec![not_found()]),
    ])
    .await;
    let resource = common::normalize_resource(vpc_resource()).await;
    let (token, _) = expect_create_pending(
        provider
            .create_step(resource.as_resource(), TestCreateAttempt)
            .await
            .expect("issue create"),
    );

    match poll_test_create(&provider, resource.as_resource(), token)
        .await
        .expect_err("identifier-bearing failure must remain a failure")
    {
        TestCreateVerdict::Created {
            identifier,
            observation,
            error,
        } => {
            assert_eq!(identifier.as_str(), VPC_ID);
            assert!(matches!(
                observation.as_ref(),
                Observation::Read(ReadOutcome::NotFound { .. })
            ));
            assert!(error.to_string().contains("Throttling"));
        }
        other => panic!("expected created failure, got {other:?}"),
    }
    assert_eq!(service.calls("CreateResource"), 1);
    assert_eq!(service.calls("GetResourceRequestStatus"), 1);
}

#[tokio::test]
async fn cancel_complete_without_identifier_is_untracked_without_reissue() {
    let (provider, service) = provider_with_script([
        (
            "CreateResource",
            vec![mutation_reply_without_identifier("create-0")],
        ),
        (
            "GetResourceRequestStatus",
            vec![status_reply("CANCEL_COMPLETE", None, Some("Throttling"))],
        ),
    ])
    .await;
    let resource = common::normalize_resource(vpc_resource()).await;
    let (token, _) = expect_create_pending(
        provider
            .create_step(resource.as_resource(), TestCreateAttempt)
            .await
            .expect("issue create"),
    );

    let failure = poll_test_create(&provider, resource.as_resource(), token)
        .await
        .expect_err("a deliberately canceled create must not be re-issued");
    let TestCreateVerdict::Untracked(error) = failure else {
        panic!("identifier-less CANCEL_COMPLETE must be untracked");
    };
    assert!(error.to_string().contains("Throttling"));
    assert_eq!(service.calls("CreateResource"), 1);
}

#[tokio::test]
async fn retryable_failed_delete_schedules_reissue_after_backoff() {
    let (provider, service) = provider_with_script([
        ("DeleteResource", vec![mutation_reply("delete-0", VPC_ID)]),
        (
            "GetResourceRequestStatus",
            vec![status_reply("FAILED", None, Some("Throttling"))],
        ),
    ])
    .await;
    let id = ResourceId::with_provider_identity("awscc", "ec2.Vpc", "pending", None);
    let request = DeleteRequest::default();
    let (token, delay) = expect_delete_pending(
        provider
            .delete_step(&id, &vpc_identifier(), &request)
            .await
            .expect("issue delete"),
    );
    assert_eq!(delay, 1);

    let (token, delay) = expect_delete_pending(
        provider
            .poll_delete_step(&id, &vpc_identifier(), &request, token)
            .await
            .expect("schedule delete retry"),
    );
    assert!(matches!(
        token,
        CcDeleteToken::Delete(CcRequestPhase::NotIssued(request))
            if request.attempt() == 1
    ));
    assert_eq!(delay, 10);
    assert_eq!(service.calls("DeleteResource"), 1);
}

#[tokio::test(start_paused = true)]
async fn delete_post_send_timeout_resends_with_same_client_token() {
    let (provider, service) = provider_with_post_send_failures(
        [(
            "DeleteResource",
            vec![mutation_reply("delete-timeout", VPC_ID)],
        )],
        [("DeleteResource", vec![true, false])],
    )
    .await;
    let id = ResourceId::with_provider_identity("awscc", "ec2.Vpc", "pending", None);
    let request = DeleteRequest::default();

    let (token, delay) = expect_delete_pending(
        provider
            .delete_step(&id, &vpc_identifier(), &request)
            .await
            .expect("an outcome-unknown delete should remain pending"),
    );
    assert!(matches!(
        &token,
        CcDeleteToken::Delete(CcRequestPhase::Unconfirmed(request))
            if request.attempt() == 0
    ));
    assert_eq!(delay, 10);

    let (token, delay) = expect_delete_pending(
        provider
            .poll_delete_step(&id, &vpc_identifier(), &request, token)
            .await
            .expect("resending with the same token should recover the request"),
    );
    assert!(matches!(
        token,
        CcDeleteToken::Delete(CcRequestPhase::InFlight(request))
            if request.request() == &RequestToken::from("delete-timeout")
                && request.attempt() == 0
    ));
    assert_eq!(delay, 1);

    let client_tokens = request_client_tokens(&service, "DeleteResource");
    assert_eq!(client_tokens.len(), 4);
    assert!(common::is_uuidish(&client_tokens[0]));
    assert!(client_tokens.iter().all(|token| token == &client_tokens[0]));
}

#[tokio::test(start_paused = true)]
async fn unconfirmed_delete_rejection_reports_unknown_earlier_outcome() {
    let (provider, service) = provider_with_post_send_failures(
        [(
            "DeleteResource",
            vec![
                mutation_reply("accepted-but-hidden", VPC_ID),
                mutation_reply("accepted-but-hidden", VPC_ID),
                mutation_reply("accepted-but-hidden", VPC_ID),
                invalid_request(),
            ],
        )],
        [("DeleteResource", vec![true, false])],
    )
    .await;
    let id = ResourceId::with_provider_identity("awscc", "ec2.Vpc", "pending", None);
    let request = DeleteRequest::default();
    let (token, delay) = expect_delete_pending(
        provider
            .delete_step(&id, &vpc_identifier(), &request)
            .await
            .expect("an outcome-unknown delete must remain pending"),
    );
    assert_eq!(delay, 10);

    let error = provider
        .poll_delete_step(&id, &vpc_identifier(), &request, token)
        .await
        .expect_err("a permanent rejection must stop the delete resend loop");
    assert_eq!(error.variant_name(), "api_error");
    assert_eq!(
        error.detail().context(),
        "Failed to resend delete resource; the outcome of the earlier send is unknown"
    );
    let client_tokens = request_client_tokens(&service, "DeleteResource");
    assert_eq!(client_tokens.len(), 4);
    assert_eq!(client_tokens, vec![client_tokens[0].clone(); 4]);
}

#[tokio::test(start_paused = true)]
async fn delete_post_send_timeout_is_not_capped_by_max_attempts() {
    let (provider, service) = provider_with_post_send_failures(
        [(
            "DeleteResource",
            vec![
                mutation_reply("never-returned", VPC_ID),
                mutation_reply("never-returned", VPC_ID),
                mutation_reply("never-returned", VPC_ID),
                throttled(),
            ],
        )],
        [("DeleteResource", vec![true, false])],
    )
    .await;
    let id = ResourceId::with_provider_identity("awscc", "ec2.Vpc", "pending", None);
    let request = DeleteRequest::default();

    let (mut token, delay) = expect_delete_pending(
        provider
            .delete_step(&id, &vpc_identifier(), &request)
            .await
            .expect("first outcome-unknown delete should remain pending"),
    );
    let encoded = serde_json::to_value(&token).expect("serialize delete continuation");
    assert_eq!(encoded["request"]["phase"], "unconfirmed");
    assert_eq!(encoded["request"]["attempt"], 0);
    assert_eq!(encoded["request"]["resends"], 1);
    assert_eq!(delay, 10);

    for expected_resends in 2..=14 {
        let (next, delay) = expect_delete_pending(
            provider
                .poll_delete_step(&id, &vpc_identifier(), &request, token)
                .await
                .unwrap_or_else(|error| {
                    panic!(
                        "throttled unconfirmed delete resend {expected_resends} was capped: {error}"
                    )
                }),
        );
        let encoded = serde_json::to_value(&next).expect("serialize delete continuation");
        assert_eq!(encoded["request"]["phase"], "unconfirmed");
        assert_eq!(encoded["request"]["attempt"], 0);
        assert_eq!(encoded["request"]["resends"], expected_resends);
        let expected_delay = match expected_resends {
            2 => 20,
            3 => 40,
            4 => 80,
            5..=14 => 120,
            _ => unreachable!("test resend range is exact"),
        };
        assert_eq!(delay, expected_delay);
        token = next;
    }

    let client_tokens = request_client_tokens(&service, "DeleteResource");
    assert_eq!(client_tokens.len(), 42);
    assert_eq!(client_tokens, vec![client_tokens[0].clone(); 42]);
}

#[tokio::test(start_paused = true)]
async fn delete_throttling_reuses_client_token_until_retry_cap() {
    let (provider, service) = provider_with_script([("DeleteResource", vec![throttled()])]).await;
    let id = ResourceId::with_provider_identity("awscc", "ec2.Vpc", "pending", None);
    let request = DeleteRequest::default();

    let (mut token, _) = expect_delete_pending(
        provider
            .delete_step(&id, &vpc_identifier(), &request)
            .await
            .expect("first throttled delete should schedule a retry"),
    );
    for expected_attempt in 2..=12 {
        let (next, _) = expect_delete_pending(
            provider
                .poll_delete_step(&id, &vpc_identifier(), &request, token)
                .await
                .unwrap_or_else(|error| {
                    panic!("throttled delete attempt {expected_attempt} failed early: {error}")
                }),
        );
        token = next;
    }

    provider
        .poll_delete_step(&id, &vpc_identifier(), &request, token)
        .await
        .expect_err("retryable service rejection should still honor the retry cap");
    let client_tokens = request_client_tokens(&service, "DeleteResource");
    assert_eq!(client_tokens.len(), 39);
    assert!(client_tokens.iter().all(|token| token == &client_tokens[0]));
}

#[tokio::test(start_paused = true)]
async fn terminal_failed_delete_rotates_client_token_before_reissue() {
    let (provider, service) = provider_with_post_send_failures(
        [
            (
                "DeleteResource",
                vec![
                    mutation_reply("delete-failed", VPC_ID),
                    mutation_reply("delete-reissued", VPC_ID),
                ],
            ),
            (
                "GetResourceRequestStatus",
                vec![status_reply(
                    "FAILED",
                    None,
                    Some("resource has non-deleted VPC Attachments"),
                )],
            ),
        ],
        [("DeleteResource", vec![false, true, false])],
    )
    .await;
    let id = ResourceId::with_provider_identity("awscc", "ec2.Vpc", "pending", None);
    let request = DeleteRequest::default();

    let (token, _) = expect_delete_pending(
        provider
            .delete_step(&id, &vpc_identifier(), &request)
            .await
            .expect("issue original delete"),
    );
    let (token, _) = expect_delete_pending(
        provider
            .poll_delete_step(&id, &vpc_identifier(), &request, token)
            .await
            .expect("terminal retryable failure should schedule a new request"),
    );
    let (token, _) = expect_delete_pending(
        provider
            .poll_delete_step(&id, &vpc_identifier(), &request, token)
            .await
            .expect("outcome-unknown resend should preserve the new request"),
    );
    expect_delete_pending(
        provider
            .poll_delete_step(&id, &vpc_identifier(), &request, token)
            .await
            .expect("second resend should recover the new request"),
    );

    let client_tokens = request_client_tokens(&service, "DeleteResource");
    assert_eq!(client_tokens.len(), 5);
    assert_ne!(client_tokens[0], client_tokens[1]);
    assert!(
        client_tokens[1..]
            .iter()
            .all(|token| token == &client_tokens[1])
    );
}

#[tokio::test]
async fn transit_gateway_delete_uses_per_type_retry_override_past_attempt_twelve() {
    let (provider, service) = provider_with_script([
        (
            "DeleteResource",
            vec![mutation_reply("delete-tgw", TRANSIT_GATEWAY_ID)],
        ),
        (
            "GetResourceRequestStatus",
            vec![status_reply("FAILED", None, Some("Throttling"))],
        ),
    ])
    .await;
    let id = ResourceId::with_provider_identity("awscc", "ec2.TransitGateway", "pending", None);
    let request = DeleteRequest::default();
    let (mut token, _) = expect_delete_pending(
        provider
            .delete_step(&id, &transit_gateway_identifier(), &request)
            .await
            .expect("issue TransitGateway delete"),
    );

    for attempt in 1_u32..=13 {
        let (not_issued, _) = expect_delete_pending(
            provider
                .poll_delete_step(&id, &transit_gateway_identifier(), &request, token)
                .await
                .unwrap_or_else(|error| {
                    panic!("TransitGateway retry {attempt} exhausted too early: {error}")
                }),
        );
        assert!(matches!(
            &not_issued,
            CcDeleteToken::Delete(CcRequestPhase::NotIssued(request))
                if request.attempt() == attempt
        ));

        let (in_flight, delay) = expect_delete_pending(
            provider
                .poll_delete_step(&id, &transit_gateway_identifier(), &request, not_issued)
                .await
                .expect("re-issue TransitGateway delete"),
        );
        assert!(matches!(
            &in_flight,
            CcDeleteToken::Delete(CcRequestPhase::InFlight(request))
                if request.request() == &RequestToken::from("delete-tgw")
                    && request.attempt() == attempt
        ));
        assert_eq!(delay, 1);
        token = in_flight;
    }

    assert_eq!(service.calls("GetResourceRequestStatus"), 13);
    assert_eq!(service.calls("DeleteResource"), 14);
}

#[tokio::test]
async fn retryable_failed_delete_with_identifier_errors_without_reissue() {
    let (provider, service) = provider_with_script([
        ("DeleteResource", vec![mutation_reply("delete-0", VPC_ID)]),
        (
            "GetResourceRequestStatus",
            vec![status_reply("FAILED", Some(VPC_ID), Some("Throttling"))],
        ),
    ])
    .await;
    let id = ResourceId::with_provider_identity("awscc", "ec2.Vpc", "pending", None);
    let request = DeleteRequest::default();
    let (token, _) = expect_delete_pending(
        provider
            .delete_step(&id, &vpc_identifier(), &request)
            .await
            .expect("issue delete"),
    );

    let error = provider
        .poll_delete_step(&id, &vpc_identifier(), &request, token)
        .await
        .expect_err("identifier-bearing delete failure should not retry");
    assert_eq!(error.detail().context(), "Operation failed");
    assert!(error.detail().cause().is_some());
    assert_eq!(service.calls("DeleteResource"), 1);
}

#[tokio::test]
async fn non_retryable_failed_status_preserves_untracked_and_created_outcomes() {
    let (provider, _) = provider_with_script([
        (
            "CreateResource",
            vec![mutation_reply_without_identifier("create-0")],
        ),
        (
            "GetResourceRequestStatus",
            vec![status_reply(
                "FAILED",
                None,
                Some("InvalidParameterValue: invalid CIDR"),
            )],
        ),
    ])
    .await;
    let resource = common::normalize_resource(vpc_resource()).await;
    let (token, _) = expect_create_pending(
        provider
            .create_step(resource.as_resource(), TestCreateAttempt)
            .await
            .expect("issue create"),
    );
    let error = poll_test_create(&provider, resource.as_resource(), token)
        .await
        .expect_err("failed create without identifier must error");
    let TestCreateVerdict::Untracked(error) = error else {
        panic!("failure without identifier must be untracked");
    };
    assert_eq!(error.variant_name(), "api_error");
    assert!(error.to_string().contains("InvalidParameterValue"));

    let (provider, _) = provider_with_script([
        ("CreateResource", vec![mutation_reply("create-0", VPC_ID)]),
        (
            "GetResourceRequestStatus",
            vec![status_reply(
                "FAILED",
                Some(VPC_ID),
                Some("post-create read denied"),
            )],
        ),
        ("GetResource", vec![not_found()]),
    ])
    .await;
    let resource = common::normalize_resource(vpc_resource()).await;
    let (token, _) = expect_create_pending(
        provider
            .create_step(resource.as_resource(), TestCreateAttempt)
            .await
            .expect("issue partial create"),
    );
    match poll_test_create(&provider, resource.as_resource(), token)
        .await
        .expect_err("terminal failure must not become partial success")
    {
        TestCreateVerdict::Created { error, .. } => {
            assert!(error.to_string().contains("post-create read denied"));
        }
        other => panic!("expected created failure, got {other:?}"),
    }
}

#[tokio::test(start_paused = true)]
async fn retry_attempts_exhausted_preserves_the_last_sdk_error_source() {
    let (provider, service) = provider_with_script([("CreateResource", vec![throttled()])]).await;
    let resource = common::normalize_resource(vpc_resource()).await;
    let (mut token, delay) = expect_create_pending(
        provider
            .create_step(resource.as_resource(), TestCreateAttempt)
            .await
            .expect("first retryable create"),
    );
    assert_eq!(delay, 10);

    let expected_delays = [20, 40, 80, 120, 120, 120, 120, 120, 120, 120, 120];
    for (attempt, expected_delay) in (1..12).zip(expected_delays) {
        let (next, delay) = expect_create_pending(
            poll_test_create(&provider, resource.as_resource(), token)
                .await
                .unwrap_or_else(|error| panic!("retry {attempt} failed early: {error:?}")),
        );
        assert!(matches!(
            &next,
            CcCreateToken::NotIssued(create) if create.attempt() == attempt + 1
        ));
        assert_eq!(delay, expected_delay);
        token = next;
    }

    let error = poll_test_create(&provider, resource.as_resource(), token)
        .await
        .expect_err("the final configured retry must be exhausted");
    let TestCreateVerdict::Untracked(error) = error else {
        panic!("exhausted unissued retry must be untracked");
    };
    let cause = error
        .detail()
        .cause()
        .expect("final retryable SDK error should remain attached");
    let sdk_error = cause.source().and_then(|source| {
        source.downcast_ref::<aws_sdk_cloudcontrol::error::SdkError<CreateResourceError>>()
    });
    assert!(
        sdk_error.is_some(),
        "cause chain should retain CreateResource SdkError"
    );
    assert_eq!(service.calls("CreateResource"), 39);
}

#[tokio::test]
async fn status_retry_exhaustion_preserves_the_last_operation_failure() {
    let (provider, service) = provider_with_script([(
        "GetResourceRequestStatus",
        vec![status_reply("FAILED", None, Some("Throttling"))],
    )])
    .await;
    let resource = common::normalize_resource(vpc_resource()).await;

    let error = poll_test_create(
        &provider,
        resource.as_resource(),
        create_token(json!({
            "phase": "in_flight",
            "request": "create-final",
            "attempt": 12,
        })),
    )
    .await
    .expect_err("final retryable status should return its real failure");

    let TestCreateVerdict::Untracked(error) = error else {
        panic!("exhausted status retry without an identifier must be untracked");
    };
    assert_eq!(
        error.detail().context(),
        "Cloud Control create ended with FAILED: Throttling"
    );
    assert_eq!(service.calls("GetResourceRequestStatus"), 1);
    assert_eq!(service.calls("CreateResource"), 0);
}

#[tokio::test]
async fn update_success_reads_back_after_one_status_call() {
    let (provider, service) = provider_with_script([
        ("UpdateResource", vec![mutation_reply("update-0", VPC_ID)]),
        (
            "GetResourceRequestStatus",
            vec![status_reply("SUCCESS", Some(VPC_ID), None)],
        ),
        (
            "GetResource",
            vec![get_resource_reply(VPC_TYPE, VPC_ID, vpc_properties())],
        ),
    ])
    .await;
    let (id, from, patch) = vpc_update();

    let token = match provider
        .update_step(&id, &vpc_identifier(), &from, &patch)
        .await
        .expect("issue update")
    {
        OperationStep::Pending {
            pending,
            poll_after_secs,
        } => {
            assert_eq!(poll_after_secs, 1);
            pending
        }
        other => panic!("expected pending update, got {other:?}"),
    };
    assert!(matches!(
        &token,
        CcUpdateToken::InFlight(request)
            if request.request() == &RequestToken::from("update-0")
                && request.attempt() == 0
    ));
    match provider
        .poll_update_step(&id, &vpc_identifier(), &from, &patch, token)
        .await
        .expect("poll update")
    {
        OperationStep::Done(UpdateOutcome::Success { state }) => {
            assert_eq!(state.identifier().as_str(), VPC_ID);
        }
        other => panic!("expected successful update, got {other:?}"),
    }
    assert_eq!(service.calls("UpdateResource"), 1);
    assert_eq!(service.calls("GetResourceRequestStatus"), 1);
    assert_eq!(service.calls("GetResource"), 1);
}

#[tokio::test(start_paused = true)]
async fn update_not_issued_resends_the_original_patch() {
    let (provider, service) = provider_with_script([(
        "UpdateResource",
        vec![
            throttled(),
            throttled(),
            throttled(),
            mutation_reply("update-1", VPC_ID),
        ],
    )])
    .await;
    let (id, from, patch) = vpc_update();

    let (token, delay) = match provider
        .update_step(&id, &vpc_identifier(), &from, &patch)
        .await
        .expect("retryable initial update")
    {
        OperationStep::Pending {
            pending,
            poll_after_secs,
        } => (pending, poll_after_secs),
        other => panic!("expected pending update, got {other:?}"),
    };
    assert!(matches!(
        &token,
        CcUpdateToken::NotIssued(request) if request.attempt() == 1
    ));
    assert_eq!(delay, 10);
    assert_eq!(service.calls("UpdateResource"), 3);

    let (token, delay) = expect_update_pending(
        provider
            .poll_update_step(&id, &vpc_identifier(), &from, &patch, token)
            .await
            .expect("re-send update"),
    );
    assert!(matches!(
        token,
        CcUpdateToken::InFlight(request)
            if request.request() == &RequestToken::from("update-1")
                && request.attempt() == 1
    ));
    assert_eq!(delay, 1);
    assert_eq!(service.calls("UpdateResource"), 4);
    let bodies = service.bodies("UpdateResource");
    assert_eq!(bodies.len(), 4);
    let patches = bodies
        .iter()
        .map(|body| serde_json::from_str::<JsonValue>(body).unwrap()["PatchDocument"].clone())
        .collect::<Vec<_>>();
    assert_eq!(patches[0], patches[3]);
    let client_tokens = request_client_tokens(&service, "UpdateResource");
    assert!(client_tokens.iter().all(|token| token == &client_tokens[0]));
}

#[tokio::test(start_paused = true)]
async fn update_post_send_timeout_resends_with_same_client_token() {
    let (provider, service) = provider_with_post_send_failures(
        [(
            "UpdateResource",
            vec![mutation_reply("update-timeout", VPC_ID)],
        )],
        [("UpdateResource", vec![true, false])],
    )
    .await;
    let (id, from, patch) = vpc_update();

    let (token, delay) = expect_update_pending(
        provider
            .update_step(&id, &vpc_identifier(), &from, &patch)
            .await
            .expect("an outcome-unknown update should remain pending"),
    );
    assert!(matches!(
        &token,
        CcRequestPhase::Unconfirmed(request) if request.attempt() == 0
    ));
    assert_eq!(delay, 10);

    let (token, delay) = expect_update_pending(
        provider
            .poll_update_step(&id, &vpc_identifier(), &from, &patch, token)
            .await
            .expect("resending with the same token should recover the request"),
    );
    assert!(matches!(
        token,
        CcRequestPhase::InFlight(request)
            if request.request() == &RequestToken::from("update-timeout")
                && request.attempt() == 0
    ));
    assert_eq!(delay, 1);

    let client_tokens = request_client_tokens(&service, "UpdateResource");
    assert_eq!(client_tokens.len(), 4);
    assert!(common::is_uuidish(&client_tokens[0]));
    assert!(client_tokens.iter().all(|token| token == &client_tokens[0]));
}

#[tokio::test(start_paused = true)]
async fn unconfirmed_update_rejection_reports_unknown_earlier_outcome() {
    let (provider, service) = provider_with_post_send_failures(
        [(
            "UpdateResource",
            vec![
                mutation_reply("accepted-but-hidden", VPC_ID),
                mutation_reply("accepted-but-hidden", VPC_ID),
                mutation_reply("accepted-but-hidden", VPC_ID),
                invalid_request(),
            ],
        )],
        [("UpdateResource", vec![true, false])],
    )
    .await;
    let (id, from, patch) = vpc_update();
    let (token, delay) = expect_update_pending(
        provider
            .update_step(&id, &vpc_identifier(), &from, &patch)
            .await
            .expect("an outcome-unknown update must remain pending"),
    );
    assert_eq!(delay, 10);

    let error = provider
        .poll_update_step(&id, &vpc_identifier(), &from, &patch, token)
        .await
        .expect_err("a permanent rejection must stop the update resend loop");
    assert_eq!(error.variant_name(), "api_error");
    assert_eq!(
        error.detail().context(),
        "Failed to resend update resource; the outcome of the earlier send is unknown"
    );
    let client_tokens = request_client_tokens(&service, "UpdateResource");
    assert_eq!(client_tokens.len(), 4);
    assert_eq!(client_tokens, vec![client_tokens[0].clone(); 4]);
}

#[tokio::test(start_paused = true)]
async fn update_post_send_timeout_is_not_capped_by_max_attempts() {
    let (provider, service) = provider_with_post_send_failures(
        [(
            "UpdateResource",
            vec![
                mutation_reply("never-returned", VPC_ID),
                mutation_reply("never-returned", VPC_ID),
                mutation_reply("never-returned", VPC_ID),
                throttled(),
            ],
        )],
        [("UpdateResource", vec![true, false])],
    )
    .await;
    let (id, from, patch) = vpc_update();

    let (mut token, delay) = expect_update_pending(
        provider
            .update_step(&id, &vpc_identifier(), &from, &patch)
            .await
            .expect("first outcome-unknown update should remain pending"),
    );
    let encoded = serde_json::to_value(&token).expect("serialize update continuation");
    assert_eq!(encoded["phase"], "unconfirmed");
    assert_eq!(encoded["attempt"], 0);
    assert_eq!(encoded["resends"], 1);
    assert_eq!(delay, 10);

    for expected_resends in 2..=14 {
        let (next, delay) = expect_update_pending(
            provider
                .poll_update_step(&id, &vpc_identifier(), &from, &patch, token)
                .await
                .unwrap_or_else(|error| {
                    panic!(
                        "throttled unconfirmed update resend {expected_resends} was capped: {error}"
                    )
                }),
        );
        let encoded = serde_json::to_value(&next).expect("serialize update continuation");
        assert_eq!(encoded["phase"], "unconfirmed");
        assert_eq!(encoded["attempt"], 0);
        assert_eq!(encoded["resends"], expected_resends);
        let expected_delay = match expected_resends {
            2 => 20,
            3 => 40,
            4 => 80,
            5..=14 => 120,
            _ => unreachable!("test resend range is exact"),
        };
        assert_eq!(delay, expected_delay);
        token = next;
    }

    let client_tokens = request_client_tokens(&service, "UpdateResource");
    assert_eq!(client_tokens.len(), 42);
    assert_eq!(client_tokens, vec![client_tokens[0].clone(); 42]);
}

#[tokio::test(start_paused = true)]
async fn update_5xx_service_internal_error_is_maybe_sent_and_uncapped() {
    let (provider, service) =
        provider_with_script([("UpdateResource", vec![service_internal_error()])]).await;
    let (id, from, patch) = vpc_update();

    let (mut token, delay) = expect_update_pending(
        provider
            .update_step(&id, &vpc_identifier(), &from, &patch)
            .await
            .expect("a 5xx service error may follow an accepted update"),
    );
    let encoded = serde_json::to_value(&token).expect("serialize update continuation");
    assert_eq!(encoded["phase"], "unconfirmed");
    assert_eq!(encoded["attempt"], 0);
    assert_eq!(encoded["resends"], 1);
    assert_eq!(delay, 10);

    for expected_resends in 2..=13 {
        let (next, delay) = expect_update_pending(
            provider
                .poll_update_step(&id, &vpc_identifier(), &from, &patch, token)
                .await
                .unwrap_or_else(|error| {
                    panic!("maybe-sent update resend {expected_resends} was capped: {error}")
                }),
        );
        let encoded = serde_json::to_value(&next).expect("serialize update continuation");
        assert_eq!(encoded["phase"], "unconfirmed");
        assert_eq!(encoded["attempt"], 0);
        assert_eq!(encoded["resends"], expected_resends);
        let expected_delay = match expected_resends {
            2 => 20,
            3 => 40,
            4 => 80,
            5..=13 => 120,
            _ => unreachable!("test resend range is exact"),
        };
        assert_eq!(delay, expected_delay);
        token = next;
    }

    let client_tokens = request_client_tokens(&service, "UpdateResource");
    assert_eq!(client_tokens.len(), 39);
    assert_eq!(client_tokens, vec![client_tokens[0].clone(); 39]);
}

#[tokio::test(start_paused = true)]
async fn update_throttling_reuses_client_token_until_retry_cap() {
    let (provider, service) = provider_with_script([("UpdateResource", vec![throttled()])]).await;
    let (id, from, patch) = vpc_update();

    let (mut token, _) = expect_update_pending(
        provider
            .update_step(&id, &vpc_identifier(), &from, &patch)
            .await
            .expect("first throttled update should schedule a retry"),
    );
    for expected_attempt in 2..=12 {
        let (next, _) = expect_update_pending(
            provider
                .poll_update_step(&id, &vpc_identifier(), &from, &patch, token)
                .await
                .unwrap_or_else(|error| {
                    panic!("throttled update attempt {expected_attempt} failed early: {error}")
                }),
        );
        token = next;
    }

    provider
        .poll_update_step(&id, &vpc_identifier(), &from, &patch, token)
        .await
        .expect_err("retryable service rejection should still honor the retry cap");
    let client_tokens = request_client_tokens(&service, "UpdateResource");
    assert_eq!(client_tokens.len(), 39);
    assert!(client_tokens.iter().all(|token| token == &client_tokens[0]));
}

#[tokio::test]
async fn separate_update_steps_use_different_client_tokens() {
    let (provider, service) = provider_with_script([(
        "UpdateResource",
        vec![
            mutation_reply("update-logical-1", VPC_ID),
            mutation_reply("update-logical-2", VPC_ID),
        ],
    )])
    .await;
    let (id, from, patch) = vpc_update();

    expect_update_pending(
        provider
            .update_step(&id, &vpc_identifier(), &from, &patch)
            .await
            .expect("issue first logical update"),
    );
    expect_update_pending(
        provider
            .update_step(&id, &vpc_identifier(), &from, &patch)
            .await
            .expect("issue second logical update"),
    );

    let client_tokens = request_client_tokens(&service, "UpdateResource");
    assert_eq!(client_tokens.len(), 2);
    assert_ne!(client_tokens[0], client_tokens[1]);
}

#[tokio::test]
async fn update_and_delete_non_retryable_service_errors_are_terminal() {
    let denied = || Reply::Error {
        status: 403,
        code: "AccessDeniedException",
        message: "mutation denied",
    };
    let (provider, service) = provider_with_script([
        ("UpdateResource", vec![denied()]),
        ("DeleteResource", vec![denied()]),
    ])
    .await;
    let (id, from, patch) = vpc_update();

    provider
        .update_step(&id, &vpc_identifier(), &from, &patch)
        .await
        .expect_err("non-retryable update rejection should be terminal");
    provider
        .delete_step(&id, &vpc_identifier(), &DeleteRequest::default())
        .await
        .expect_err("non-retryable delete rejection should be terminal");
    assert_eq!(service.calls("UpdateResource"), 1);
    assert_eq!(service.calls("DeleteResource"), 1);
}

#[tokio::test]
async fn update_without_request_token_settles_and_reads_back() {
    let (provider, service) = provider_with_script([
        (
            "UpdateResource",
            vec![Reply::Json(json!({
                "ProgressEvent": {
                    "Identifier": VPC_ID,
                    "OperationStatus": "SUCCESS"
                }
            }))],
        ),
        (
            "GetResource",
            vec![get_resource_reply(VPC_TYPE, VPC_ID, vpc_properties())],
        ),
    ])
    .await;
    let (id, from, patch) = vpc_update();

    match provider
        .update_step(&id, &vpc_identifier(), &from, &patch)
        .await
        .expect("update without request token")
    {
        OperationStep::Done(UpdateOutcome::Success { state }) => {
            assert_eq!(state.identifier().as_str(), VPC_ID);
        }
        other => panic!("expected settled update, got {other:?}"),
    }
    assert_eq!(service.calls("UpdateResource"), 1);
    assert_eq!(service.calls("GetResourceRequestStatus"), 0);
    assert_eq!(service.calls("GetResource"), 1);
}

#[tokio::test]
async fn empty_update_patch_settles_without_update_request() {
    let (provider, service) = provider_with_script([(
        "GetResource",
        vec![get_resource_reply(VPC_TYPE, VPC_ID, vpc_properties())],
    )])
    .await;
    let (id, from, _) = vpc_update();
    let patch = UpdatePatch { ops: Vec::new() };

    assert!(matches!(
        provider
            .update_step(&id, &vpc_identifier(), &from, &patch)
            .await
            .expect("empty update"),
        OperationStep::Done(UpdateOutcome::Success { .. })
    ));
    assert_eq!(service.calls("UpdateResource"), 0);
    assert_eq!(service.calls("GetResource"), 1);
}

#[tokio::test]
async fn internet_gateway_delete_detaches_then_deletes() {
    let (provider, service) = provider_with_script([
        (
            "GetResource",
            vec![get_resource_reply(
                IGW_TYPE,
                IGW_ID,
                json!({
                    "InternetGatewayId": IGW_ID,
                    "Attachments": [{"VpcId": VPC_ID}],
                }),
            )],
        ),
        ("UpdateResource", vec![mutation_reply("detach-0", IGW_ID)]),
        ("DeleteResource", vec![mutation_reply("delete-0", IGW_ID)]),
        (
            "GetResourceRequestStatus",
            vec![
                status_reply("SUCCESS", Some(IGW_ID), None),
                status_reply("SUCCESS", Some(IGW_ID), None),
            ],
        ),
    ])
    .await;
    let id = ResourceId::with_provider_identity("awscc", "ec2.InternetGateway", "pending", None);
    let request = DeleteRequest::default();

    let (token, delay) = expect_delete_pending(
        provider
            .delete_step(&id, &internet_gateway_identifier(), &request)
            .await
            .expect("issue IGW detach"),
    );
    assert_eq!(delay, 1);
    assert!(matches!(
        &token,
        CcDeleteToken::DetachIgw(CcRequestPhase::InFlight(request))
            if request.request() == &RequestToken::from("detach-0")
                && request.attempt() == 0
    ));
    assert_eq!(service.calls("UpdateResource"), 1);
    assert_eq!(service.calls("DeleteResource"), 0);

    let (token, delay) = expect_delete_pending(
        provider
            .poll_delete_step(&id, &internet_gateway_identifier(), &request, token)
            .await
            .expect("finish detach and issue delete"),
    );
    assert_eq!(delay, 1);
    assert!(matches!(
        &token,
        CcDeleteToken::Delete(CcRequestPhase::InFlight(request))
            if request.request() == &RequestToken::from("delete-0")
                && request.attempt() == 0
    ));
    assert_eq!(service.calls("DeleteResource"), 1);

    assert!(matches!(
        provider
            .poll_delete_step(&id, &internet_gateway_identifier(), &request, token)
            .await
            .expect("finish delete"),
        OperationStep::Done(())
    ));
    assert_eq!(service.calls("GetResourceRequestStatus"), 2);
}

#[tokio::test(start_paused = true)]
async fn internet_gateway_detach_not_issued_rechecks_then_resets_delete_attempt() {
    let (provider, service) = provider_with_script([
        (
            "GetResource",
            vec![get_resource_reply(
                IGW_TYPE,
                IGW_ID,
                json!({
                    "InternetGatewayId": IGW_ID,
                    "Attachments": [{"VpcId": VPC_ID}],
                }),
            )],
        ),
        (
            "UpdateResource",
            vec![
                throttled(),
                throttled(),
                throttled(),
                mutation_reply("detach-1", IGW_ID),
            ],
        ),
        ("DeleteResource", vec![mutation_reply("delete-0", IGW_ID)]),
        (
            "GetResourceRequestStatus",
            vec![status_reply("SUCCESS", Some(IGW_ID), None)],
        ),
    ])
    .await;
    let id = ResourceId::with_provider_identity("awscc", "ec2.InternetGateway", "pending", None);
    let request = DeleteRequest::default();

    let (token, delay) = expect_delete_pending(
        provider
            .delete_step(&id, &internet_gateway_identifier(), &request)
            .await
            .expect("schedule IGW detach resend"),
    );
    assert!(matches!(
        &token,
        CcDeleteToken::DetachIgw(CcRequestPhase::NotIssued(request))
            if request.attempt() == 1
    ));
    assert_eq!(delay, 10);
    assert_eq!(service.calls("UpdateResource"), 3);

    let (token, delay) = expect_delete_pending(
        provider
            .poll_delete_step(&id, &internet_gateway_identifier(), &request, token)
            .await
            .expect("re-send IGW detach"),
    );
    assert!(matches!(
        &token,
        CcDeleteToken::DetachIgw(CcRequestPhase::InFlight(request))
            if request.request() == &RequestToken::from("detach-1")
                && request.attempt() == 1
    ));
    assert_eq!(delay, 1);
    assert_eq!(service.calls("GetResource"), 2);
    assert_eq!(service.calls("UpdateResource"), 4);
    assert_eq!(service.calls("GetResourceRequestStatus"), 0);
    assert_eq!(service.calls("DeleteResource"), 0);

    let (token, delay) = expect_delete_pending(
        provider
            .poll_delete_step(&id, &internet_gateway_identifier(), &request, token)
            .await
            .expect("finish retried detach and start delete"),
    );
    assert!(matches!(
        token,
        CcDeleteToken::Delete(CcRequestPhase::InFlight(request))
            if request.request() == &RequestToken::from("delete-0")
                && request.attempt() == 0
    ));
    assert_eq!(delay, 1);
    assert_eq!(service.calls("GetResourceRequestStatus"), 1);
    assert_eq!(service.calls("DeleteResource"), 1);
}

#[tokio::test(start_paused = true)]
async fn internet_gateway_detach_not_issued_skips_resend_when_already_detached() {
    let attached = get_resource_reply(
        IGW_TYPE,
        IGW_ID,
        json!({
            "InternetGatewayId": IGW_ID,
            "Attachments": [{"VpcId": VPC_ID}],
        }),
    );
    let detached = get_resource_reply(
        IGW_TYPE,
        IGW_ID,
        json!({
            "InternetGatewayId": IGW_ID,
            "Attachments": [],
        }),
    );
    let (provider, service) = provider_with_script([
        ("GetResource", vec![attached, detached]),
        ("UpdateResource", vec![throttled()]),
        ("DeleteResource", vec![mutation_reply("delete-0", IGW_ID)]),
    ])
    .await;
    let id = ResourceId::with_provider_identity("awscc", "ec2.InternetGateway", "pending", None);
    let request = DeleteRequest::default();

    let (token, _) = expect_delete_pending(
        provider
            .delete_step(&id, &internet_gateway_identifier(), &request)
            .await
            .expect("schedule IGW detach retry"),
    );
    assert!(matches!(
        &token,
        CcDeleteToken::DetachIgw(CcRequestPhase::NotIssued(request))
            if request.attempt() == 1
    ));

    let (token, delay) = expect_delete_pending(
        provider
            .poll_delete_step(&id, &internet_gateway_identifier(), &request, token)
            .await
            .expect("skip redundant detach and issue delete"),
    );
    assert!(matches!(
        token,
        CcDeleteToken::Delete(CcRequestPhase::InFlight(request))
            if request.request() == &RequestToken::from("delete-0")
                && request.attempt() == 0
    ));
    assert_eq!(delay, 1);
    assert_eq!(service.calls("GetResource"), 2);
    assert_eq!(service.calls("UpdateResource"), 3);
    assert_eq!(service.calls("GetResourceRequestStatus"), 0);
    assert_eq!(service.calls("DeleteResource"), 1);
}

#[tokio::test]
async fn internet_gateway_unconfirmed_detach_resends_without_rechecking_attachment() {
    let (provider, service) = provider_with_script([
        (
            "GetResource",
            vec![get_resource_reply(
                IGW_TYPE,
                IGW_ID,
                json!({
                    "InternetGatewayId": IGW_ID,
                    "Attachments": [],
                }),
            )],
        ),
        (
            "UpdateResource",
            vec![mutation_reply("detach-resend", IGW_ID)],
        ),
        (
            "DeleteResource",
            vec![mutation_reply("must-not-delete", IGW_ID)],
        ),
    ])
    .await;
    let id = ResourceId::with_provider_identity("awscc", "ec2.InternetGateway", "pending", None);
    let request = DeleteRequest::default();
    let token: CcDeleteToken = serde_json::from_value(json!({
        "phase": "detach_igw",
        "request": {
            "phase": "unconfirmed",
            "attempt": 13,
            "resends": 14,
            "client_token": TEST_CLIENT_TOKEN,
        },
    }))
    .expect("deserialize an unconfirmed detach continuation");

    let (token, delay) = expect_delete_pending(
        provider
            .poll_delete_step(&id, &internet_gateway_identifier(), &request, token)
            .await
            .expect("an unconfirmed detach must be resent with the same request"),
    );

    assert!(matches!(
        token,
        CcDeleteToken::DetachIgw(CcRequestPhase::InFlight(request))
            if request.request() == &RequestToken::from("detach-resend")
                && request.attempt() == 13
    ));
    assert_eq!(delay, 1);
    assert_eq!(service.calls("GetResource"), 0);
    assert_eq!(service.calls("UpdateResource"), 1);
    assert_eq!(service.calls("DeleteResource"), 0);

    let bodies = service.bodies("UpdateResource");
    assert_eq!(bodies.len(), 1);
    let body: JsonValue = serde_json::from_str(&bodies[0]).expect("valid update request body");
    let patch: JsonValue = serde_json::from_str(
        body["PatchDocument"]
            .as_str()
            .expect("detach request should contain a patch document"),
    )
    .expect("valid detach patch document");
    assert_eq!(patch, json!([{"op": "remove", "path": "/Attachments"}]));
    assert_eq!(body["ClientToken"], TEST_CLIENT_TOKEN);
}

#[tokio::test]
async fn internet_gateway_detach_partial_failure_does_not_reissue_or_delete() {
    let (provider, service) = provider_with_script([
        (
            "GetResource",
            vec![get_resource_reply(
                IGW_TYPE,
                IGW_ID,
                json!({
                    "InternetGatewayId": IGW_ID,
                    "Attachments": [{"VpcId": VPC_ID}],
                }),
            )],
        ),
        ("UpdateResource", vec![mutation_reply("detach-0", IGW_ID)]),
        (
            "GetResourceRequestStatus",
            vec![status_reply("FAILED", Some(IGW_ID), Some("Throttling"))],
        ),
    ])
    .await;
    let id = ResourceId::with_provider_identity("awscc", "ec2.InternetGateway", "pending", None);
    let request = DeleteRequest::default();
    let (token, _) = expect_delete_pending(
        provider
            .delete_step(&id, &internet_gateway_identifier(), &request)
            .await
            .expect("issue IGW detach"),
    );

    let error = provider
        .poll_delete_step(&id, &internet_gateway_identifier(), &request, token)
        .await
        .expect_err("partial detach must stop delete");
    assert_eq!(
        error.detail().context(),
        "Failed to detach Internet Gateway from VPC before deletion"
    );
    assert_eq!(
        error.to_string(),
        "[ec2.InternetGateway.pending] Failed to detach Internet Gateway from VPC before deletion: Throttling"
    );
    assert!(error.detail().cause().is_some());
    assert_eq!(service.calls("UpdateResource"), 1);
    assert_eq!(service.calls("DeleteResource"), 0);
}

#[tokio::test]
async fn unattached_internet_gateway_goes_straight_to_delete() {
    let (provider, service) = provider_with_script([
        (
            "GetResource",
            vec![get_resource_reply(
                IGW_TYPE,
                IGW_ID,
                json!({
                    "InternetGatewayId": IGW_ID,
                    "Attachments": [],
                }),
            )],
        ),
        ("DeleteResource", vec![mutation_reply("delete-0", IGW_ID)]),
    ])
    .await;
    let id = ResourceId::with_provider_identity("awscc", "ec2.InternetGateway", "pending", None);

    let (token, delay) = expect_delete_pending(
        provider
            .delete_step(
                &id,
                &internet_gateway_identifier(),
                &DeleteRequest::default(),
            )
            .await
            .expect("issue direct IGW delete"),
    );
    assert!(matches!(
        token,
        CcDeleteToken::Delete(CcRequestPhase::InFlight(request))
            if request.request() == &RequestToken::from("delete-0")
                && request.attempt() == 0
    ));
    assert_eq!(delay, 1);
    assert_eq!(service.calls("UpdateResource"), 0);
    assert_eq!(service.calls("DeleteResource"), 1);
}
