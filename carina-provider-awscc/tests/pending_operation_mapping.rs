//! Exact state-machine coverage for host-driven CloudControl operations.

#![cfg(not(target_arch = "wasm32"))]

mod common;

use std::collections::{HashMap, VecDeque};
use std::future::Future;
use std::pin::Pin;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

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
    PatchOp, PendingToken, Poll, ProviderDirectiveSchema, ProviderError, ReadOutcome,
    ResumeCreateRequest, ResumeFailureKind, Sent, UpdateOutcome, UpdateOutcomeKind, UpdatePatch,
};
use carina_core::resource::{ConcreteValue, Directives, Resource, ResourceId, State, Value};
use carina_plugin_sdk::PhasedToken;
use carina_provider_awscc::provider::{
    AwsccProviderConfig, CcCreateAttempt, CcCreatePoll, CcCreateResume, CcIssuedCreate,
    CcPollAttempt, CcResumeAttempt, CcResumedCreate, NotAppliedCreate,
};
use carina_provider_awscc::{
    AwsccProvider, BoundedSdkConfig, CcCreateOperation, CcCreateToken, CcDeleteToken,
    CcRequestPhase, CcUpdateToken, InFlightCreate, RequestToken,
};
use carina_provider_protocol::{OperationStep, PendingPhase, PollFinality, PollRequest};
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

fn delete_request() -> DeleteRequest {
    DeleteRequest::resolve(&Directives::default(), ProviderDirectiveSchema::Unavailable)
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
        attributes: HashMap<String, Value>,
        reason: String,
        missing_attributes: Vec<String>,
    },
    Completed {
        identifier: AppliedIdentifier,
        state: AppliedState,
    },
}

#[derive(Debug)]
enum TestCreateVerdict {
    NotCreated(Box<ProviderError>),
    InFlight {
        token: CcCreateToken,
        identifier: Option<AppliedIdentifier>,
        error: Box<ProviderError>,
    },
    IdentifierReported {
        identifier: AppliedIdentifier,
        observation: Box<Observation>,
        error: Box<ProviderError>,
    },
    NoResource(Box<ProviderError>),
}

struct TestCreateAttempt;

struct TestIssuedCreate {
    identifier: AppliedIdentifier,
}

impl CcCreateAttempt for TestCreateAttempt {
    type Step = TestCreateStep;
    type Verdict = TestCreateVerdict;
    type Issued = TestIssuedCreate;

    fn not_created(self, failure: NotAppliedCreate) -> Self::Verdict {
        TestCreateVerdict::NotCreated(Box::new(failure.into_error()))
    }

    fn preserve(self, token: CcCreateOperation, error: ProviderError) -> Self::Verdict {
        TestCreateVerdict::InFlight {
            token: token.into(),
            identifier: None,
            error: Box::new(error),
        }
    }

    fn issued(self, identifier: AppliedIdentifier, _sent: Sent) -> Self::Issued {
        TestIssuedCreate { identifier }
    }

    fn pending(
        self,
        token: CcCreateOperation,
        _sent: Sent,
        poll_after_secs: u32,
    ) -> Result<Self::Step, Self::Verdict> {
        let token = token.with_poll_after_secs(poll_after_secs);
        Ok(TestCreateStep::Pending {
            token: token.into(),
            poll_after_secs,
            issued: None,
        })
    }
}

impl CcPollAttempt for TestCreateAttempt {
    type Step = TestCreateStep;
    type Verdict = TestCreateVerdict;
    type Issued = TestIssuedCreate;

    fn not_created(self, failure: NotAppliedCreate) -> Self::Verdict {
        TestCreateVerdict::NotCreated(Box::new(failure.into_error()))
    }

    fn preserve(self, token: CcCreateOperation, failure: InFlightCreate) -> Self::Verdict {
        let error = failure.into_error();
        TestCreateVerdict::InFlight {
            token: token.into(),
            identifier: None,
            error: Box::new(error),
        }
    }

    fn issued(self, identifier: AppliedIdentifier) -> Self::Issued {
        TestIssuedCreate { identifier }
    }

    fn pending(
        self,
        token: CcCreateOperation,
        poll_after_secs: u32,
    ) -> Result<Self::Step, Self::Verdict> {
        let token = token.with_poll_after_secs(poll_after_secs);
        Ok(TestCreateStep::Pending {
            token: token.into(),
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
    ) -> Result<Self::Step, Self::Verdict> {
        Ok(TestCreateStep::Succeeded {
            identifier,
            attributes,
        })
    }

    fn partial(
        self,
        identifier: AppliedIdentifier,
        attributes: HashMap<String, Value>,
        reason: ProviderError,
        missing_attributes: carina_provider_awscc::provider::read_back::NonEmptyAttributeNames,
    ) -> Result<Self::Step, Self::Verdict> {
        Ok(TestCreateStep::Partial {
            identifier,
            attributes,
            reason: reason.to_string(),
            missing_attributes: missing_attributes.into(),
        })
    }

    fn identifier_reported(
        self,
        identifier: AppliedIdentifier,
        observation: Observation,
        error: ProviderError,
    ) -> Self::Verdict {
        TestCreateVerdict::IdentifierReported {
            identifier,
            observation: Box::new(observation),
            error: Box::new(error),
        }
    }

    fn no_resource(self, error: ProviderError) -> Self::Verdict {
        TestCreateVerdict::NoResource(Box::new(error))
    }

    fn preserve(self, token: CcCreateToken, error: ProviderError) -> Self::Verdict {
        TestCreateVerdict::InFlight {
            token,
            identifier: Some(self.identifier),
            error: Box::new(error),
        }
    }

    fn pending(
        self,
        token: CcCreateToken,
        poll_after_secs: u32,
    ) -> Result<Self::Step, Self::Verdict> {
        let token = token.with_poll_after_secs(poll_after_secs);
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

    fn not_created(self, failure: NotAppliedCreate) -> Self::Verdict {
        TestCreateVerdict::NotCreated(Box::new(failure.into_error()))
    }

    fn preserve(self, token: CcCreateOperation, error: ProviderError) -> Self::Verdict {
        TestCreateVerdict::InFlight {
            token: token.into(),
            identifier: None,
            error: Box::new(error),
        }
    }

    fn issued(self, identifier: AppliedIdentifier) -> Self::Resumed {
        TestIssuedCreate { identifier }
    }

    fn pending(
        self,
        token: CcCreateOperation,
        poll_after_secs: u32,
    ) -> Result<Self::Step, Self::Verdict> {
        let token = token.with_poll_after_secs(poll_after_secs);
        Ok(TestCreateStep::Pending {
            token: token.into(),
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

    fn completed(
        self,
        identifier: AppliedIdentifier,
        state: AppliedState,
    ) -> Result<Self::Step, Self::Verdict> {
        Ok(TestCreateStep::Completed { identifier, state })
    }

    fn partial(
        self,
        identifier: AppliedIdentifier,
        attributes: HashMap<String, Value>,
        reason: ProviderError,
        missing_attributes: carina_provider_awscc::provider::read_back::NonEmptyAttributeNames,
    ) -> Result<Self::Step, Self::Verdict> {
        Ok(TestCreateStep::Partial {
            identifier,
            attributes,
            reason: reason.to_string(),
            missing_attributes: missing_attributes.into(),
        })
    }

    fn identifier_reported(
        self,
        identifier: AppliedIdentifier,
        observation: Observation,
        error: ProviderError,
    ) -> Self::Verdict {
        TestCreateVerdict::IdentifierReported {
            identifier,
            observation: Box::new(observation),
            error: Box::new(error),
        }
    }

    fn no_resource(self, error: ProviderError) -> Self::Verdict {
        TestCreateVerdict::NoResource(Box::new(error))
    }

    fn preserve(self, token: CcCreateToken, error: ProviderError) -> Self::Verdict {
        TestCreateVerdict::InFlight {
            token,
            identifier: Some(self.identifier),
            error: Box::new(error),
        }
    }

    fn pending(
        self,
        token: CcCreateToken,
        poll_after_secs: u32,
    ) -> Result<Self::Step, Self::Verdict> {
        let token = token.with_poll_after_secs(poll_after_secs);
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
    let from: State = State::existing(
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
    let config = mock
        .sdk_config(&region)
        .await
        .into_builder()
        .http_client(http_client)
        .build();
    (
        AwsccProvider::from_sdk_config(
            carina_provider_awscc::BoundedSdkConfig::for_test(config),
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
    let config = mock.sdk_config("us-init-error-1").await;
    let provider = AwsccProvider::from_sdk_config(
        BoundedSdkConfig::for_test(config),
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

fn assert_cloudcontrol_api_error(
    error: &ProviderError,
    message: &str,
    operation: &str,
    code: &str,
    cause: &str,
) {
    assert_eq!(error.variant_name(), "api_error");
    assert_eq!(error.detail().context(), message);
    assert_eq!(error.detail().operation.as_deref(), Some(operation));
    assert_eq!(error.detail().status, Some(400));
    assert_eq!(error.detail().code(), Some(code));
    assert_eq!(error.to_wire().cause.as_deref(), Some(cause));
}

fn create_token(value: JsonValue) -> CcCreateToken {
    serde_json::from_value(value).expect("deserialize test create token")
}

fn pending_create_for_token(
    resource: &Resource<
        impl Sized,
        carina_core::resource::NoPlaceholder,
        carina_core::resource::AbsoluteBindingAddress,
        impl Sized,
    >,
    token: &CcCreateToken,
) -> carina_core::provider::PendingCreate {
    let pending_token = PendingToken::new(
        serde_json::to_string(token).expect("serialize test Cloud Control create token"),
    );
    let attempt = CreateAttempt::for_test(resource.id.clone());
    match token.identifier() {
        Some(identifier) => {
            let issued = attempt.issued(
                common::applied_identifier(identifier),
                Sent::AllRequestAttributes,
            );
            if token.phase() == PendingPhase::ReadBack {
                issued.pending_read_back(pending_token, 5)
            } else {
                issued.pending(pending_token, 5)
            }
        }
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
fn not_issued_request_without_client_token_is_rejected() {
    let error = serde_json::from_value::<CcRequestPhase>(json!({
        "phase": "not_issued",
        "attempt": 0,
    }))
    .expect_err("ephemeral update/delete continuations need no legacy token fallback");

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
            "poll_after_secs": 5,
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
            phase,
        } => {
            assert_eq!(phase, PendingPhase::Operation);
            (pending, poll_after_secs)
        }
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
            assert_eq!(
                token.poll_after_secs(),
                poll_after_secs,
                "the continuation token must retain the exact host poll cadence"
            );
            (token, poll_after_secs)
        }
        TestCreateStep::Partial {
            identifier, reason, ..
        } => {
            panic!("expected pending create, got partial {identifier:?}: {reason}")
        }
        TestCreateStep::Completed { identifier, state } => {
            panic!("expected pending create, got completed {identifier:?}: {state:?}")
        }
        other => panic!("expected pending create, got {other:?}"),
    }
}

fn test_create_poll(token: &CcCreateToken) -> CcCreatePoll<TestCreateAttempt, TestIssuedCreate> {
    match token.identifier() {
        Some(identifier) => CcCreatePoll::Issued(
            TestIssuedCreate {
                identifier: common::applied_identifier(identifier),
            },
            token.clone(),
        ),
        None => CcCreatePoll::Unissued(TestCreateAttempt, token.clone().into_operation().unwrap()),
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
    resource: &Resource<
        impl Sized,
        carina_core::resource::NoPlaceholder,
        carina_core::resource::AbsoluteBindingAddress,
        impl Sized,
    >,
    token: CcCreateToken,
) -> Result<TestCreateStep, TestCreateVerdict> {
    let poll = test_create_poll(&token);
    provider
        .poll_create_step(
            resource,
            poll,
            carina_provider_protocol::PollRequest {
                finality: carina_provider_protocol::PollFinality::Continue,
            },
        )
        .await
}

#[tokio::test]
async fn create_read_back_success_then_get_not_found_then_complete() {
    assert_create_read_back_recovers(not_found()).await;
}

#[tokio::test]
async fn create_read_back_error_then_complete() {
    assert_create_read_back_recovers(throttled()).await;
}

async fn assert_create_read_back_recovers(first_read: Reply) {
    let (provider, service) = provider_with_script([
        (
            "CreateResource",
            vec![mutation_reply("create-read-back", VPC_ID)],
        ),
        (
            "GetResourceRequestStatus",
            vec![status_reply("SUCCESS", Some(VPC_ID), None)],
        ),
        (
            "GetResource",
            vec![first_read; 3]
                .into_iter()
                .chain([get_resource_reply(VPC_TYPE, VPC_ID, vpc_properties())])
                .collect(),
        ),
    ])
    .await;
    let resource = common::normalize_resource(vpc_resource()).await;
    let (operation, _) = expect_create_pending(
        provider
            .create_step(resource.as_resource(), TestCreateAttempt)
            .await
            .unwrap(),
    );
    let (read_back, _) = expect_create_pending(
        poll_test_create(&provider, resource.as_resource(), operation)
            .await
            .unwrap(),
    );
    assert_eq!(read_back.identifier(), Some(VPC_ID));
    assert_eq!(
        serde_json::to_value(&read_back).unwrap()["phase"],
        "read_back"
    );
    let mut next = read_back;
    for attempt in 0..3 {
        match poll_test_create(&provider, resource.as_resource(), next)
            .await
            .unwrap()
        {
            TestCreateStep::Succeeded { .. } => break,
            TestCreateStep::Pending { token, .. } if attempt < 2 => next = token,
            other => panic!("read-back should recover: {other:?}"),
        }
    }
    assert_eq!(service.calls("CreateResource"), 1);
    assert_eq!(service.calls("GetResourceRequestStatus"), 1);
    assert_eq!(service.calls("GetResource"), 4);
}

#[tokio::test]
async fn update_read_back_after_terminal_status_then_not_found_then_complete() {
    let (provider, service) = provider_with_script([
        (
            "UpdateResource",
            vec![mutation_reply("update-read-back", VPC_ID)],
        ),
        (
            "GetResourceRequestStatus",
            vec![status_reply("SUCCESS", Some(VPC_ID), None)],
        ),
        (
            "GetResource",
            vec![
                not_found(),
                get_resource_reply(VPC_TYPE, VPC_ID, vpc_properties()),
            ],
        ),
    ])
    .await;
    let (id, from, patch) = vpc_update();
    let (operation, _) = expect_update_pending(
        provider
            .update_step(&id, &vpc_identifier(), &from, &patch)
            .await
            .unwrap(),
    );
    let read_back = match provider
        .poll_update_step(
            &id,
            &vpc_identifier(),
            &from,
            &patch,
            operation,
            carina_provider_protocol::PollRequest {
                finality: carina_provider_protocol::PollFinality::Continue,
            },
        )
        .await
        .unwrap()
    {
        OperationStep::Pending {
            pending,
            phase: PendingPhase::ReadBack,
            ..
        } => pending,
        other => panic!("expected read-back pending, got {other:?}"),
    };
    assert!(matches!(
        provider
            .poll_update_step(
                &id,
                &vpc_identifier(),
                &from,
                &patch,
                read_back,
                carina_provider_protocol::PollRequest {
                    finality: carina_provider_protocol::PollFinality::Continue
                }
            )
            .await
            .unwrap(),
        OperationStep::Done(_)
    ));
    assert_eq!(service.calls("UpdateResource"), 1);
    assert_eq!(service.calls("GetResourceRequestStatus"), 1);
    assert_eq!(service.calls("GetResource"), 2);
}

fn read_back_token(resource_type: &str, identifier: &str, _missing: &[&str]) -> CcCreateToken {
    create_token(json!({
        "phase": "read_back", "identifier": identifier, "resource_type": resource_type,
    }))
}

#[tokio::test]
async fn create_read_back_declared_synthesized_attribute_then_complete() {
    let role = "read-back-role";
    let arn = "arn:aws:iam::123456789012:role/read-back-role";
    let resource = Resource::with_provider("awscc", "iam.Role", "pending", None)
        .with_attribute("role_name", string(role));
    let (provider, service) = provider_with_script([
        ("CreateResource", vec![mutation_reply("create-role", role)]),
        (
            "GetResourceRequestStatus",
            vec![status_reply("SUCCESS", Some(role), None)],
        ),
        (
            "GetResource",
            vec![
                get_resource_reply("AWS::IAM::Role", role, json!({})),
                get_resource_reply("AWS::IAM::Role", role, json!({"RoleName":role, "Arn":arn})),
            ],
        ),
    ])
    .await;
    let (operation, _) = expect_create_pending(
        provider
            .create_step(&resource, TestCreateAttempt)
            .await
            .unwrap(),
    );
    let (pending, _) = expect_create_pending(
        poll_test_create(&provider, &resource, operation)
            .await
            .unwrap(),
    );
    let CcCreateToken::ReadBack(continuation) = &pending else {
        panic!("expected read-back")
    };
    assert_eq!(continuation.identifier().as_str(), role);
    let TestCreateStep::Succeeded { attributes, .. } =
        poll_test_create(&provider, &resource, pending)
            .await
            .unwrap()
    else {
        panic!("expected success")
    };
    assert_eq!(attributes.get("arn"), Some(&string(arn)));
    assert_eq!(service.calls("CreateResource"), 1);
    assert_eq!(service.calls("GetResourceRequestStatus"), 1);
    assert_eq!(service.calls("GetResource"), 2);
}

#[tokio::test]
async fn read_back_final_attempt_returns_partial_with_exact_missing_attributes() {
    for (resource, identifier, first_read, expected_missing) in [
        (
            vpc_resource(),
            VPC_ID,
            not_found(),
            carina_provider_awscc::schemas::generated::get_config_by_type("ec2.Vpc")
                .unwrap()
                .schema
                .as_schema()
                .attributes()
                .keys()
                .cloned()
                .collect::<Vec<_>>(),
        ),
        (
            vpc_resource(),
            VPC_ID,
            Reply::Error {
                status: 400,
                code: "AccessDeniedException",
                message: "injected read failure",
            },
            carina_provider_awscc::schemas::generated::get_config_by_type("ec2.Vpc")
                .unwrap()
                .schema
                .as_schema()
                .attributes()
                .keys()
                .cloned()
                .collect::<Vec<_>>(),
        ),
        (
            Resource::with_provider("awscc", "iam.Role", "pending", None),
            "read-back-role",
            get_resource_reply("AWS::IAM::Role", "read-back-role", json!({})),
            vec!["arn".into()],
        ),
    ] {
        for boundary in ["create", "resume", "update"] {
            let (provider, service) =
                provider_with_script([("GetResource", vec![first_read.clone()])]).await;
            let token = read_back_token(resource.id.resource_type.as_str(), identifier, &["arn"]);
            // VPC has no arn declaration; its previous missing list must still be a schema name.
            let token = if resource.id.resource_type.as_str() == "ec2.Vpc" {
                read_back_token("ec2.Vpc", identifier, &["vpc_id"])
            } else {
                token
            };
            let final_poll = PollRequest {
                finality: PollFinality::FinalAttempt,
            };
            let identifier = common::applied_identifier(identifier);
            let missing = match boundary {
                "create" => {
                    let step = provider
                        .poll_create_step(
                            &resource,
                            CcCreatePoll::<TestCreateAttempt, TestIssuedCreate>::Issued(
                                TestIssuedCreate {
                                    identifier: identifier.clone(),
                                },
                                token,
                            ),
                            final_poll,
                        )
                        .await
                        .unwrap();
                    let TestCreateStep::Partial {
                        identifier: actual,
                        attributes,
                        missing_attributes,
                        ..
                    } = step
                    else {
                        panic!("expected partial create")
                    };
                    assert_eq!(actual, identifier);
                    if resource.id.resource_type.as_str() == "ec2.Vpc" {
                        assert_eq!(attributes, resource.resolved_attributes());
                    } else {
                        assert!(!attributes.contains_key("arn"));
                    }
                    missing_attributes
                }
                "resume" => {
                    let step = provider
                        .resume_create_step(
                            &resource,
                            CcCreateResume::<TestCreateAttempt, TestIssuedCreate>::Issued(
                                TestIssuedCreate {
                                    identifier: identifier.clone(),
                                },
                                token,
                            ),
                            final_poll,
                        )
                        .await
                        .unwrap();
                    let TestCreateStep::Partial {
                        identifier: actual,
                        missing_attributes,
                        ..
                    } = step
                    else {
                        panic!("expected partial resume")
                    };
                    assert_eq!(actual, identifier);
                    missing_attributes
                }
                _ => {
                    let CcCreateToken::ReadBack(continuation) = token else {
                        unreachable!()
                    };
                    let from: State = State::existing(resource.id.clone(), HashMap::new())
                        .with_identifier(identifier.as_str());
                    let step = provider
                        .poll_update_step(
                            &resource.id,
                            &identifier,
                            &from,
                            &UpdatePatch { ops: vec![] },
                            CcUpdateToken::ReadBack(continuation.into()),
                            final_poll,
                        )
                        .await
                        .unwrap();
                    let OperationStep::Done(outcome) = step else {
                        panic!("final update must be terminal")
                    };
                    let UpdateOutcomeKind::PartialSuccess { state, diagnostic } = outcome.kind()
                    else {
                        panic!("expected partial update")
                    };
                    assert_eq!(state.identifier(), &identifier);
                    diagnostic
                        .missing_attributes()
                        .iter()
                        .map(|name| name.as_str().to_string())
                        .collect::<Vec<_>>()
                }
            };
            assert_eq!(missing, expected_missing, "{boundary}");
            assert_eq!(service.actions(), ["GetResource"]);
        }
    }
}

#[tokio::test]
async fn read_back_complete_final_read_succeeds_without_waiting_for_undeclared_outputs() {
    let (provider, service) = provider_with_script([(
        "GetResource",
        vec![get_resource_reply(
            VPC_TYPE,
            VPC_ID,
            json!({"VpcId":VPC_ID}),
        )],
    )])
    .await;
    let resource = vpc_resource();
    let token = read_back_token("ec2.Vpc", VPC_ID, &["vpc_id"]);
    let step = provider
        .poll_create_step(
            &resource,
            CcCreatePoll::<TestCreateAttempt, TestIssuedCreate>::Issued(
                TestIssuedCreate {
                    identifier: vpc_identifier(),
                },
                token,
            ),
            PollRequest {
                finality: PollFinality::FinalAttempt,
            },
        )
        .await
        .unwrap();
    assert!(matches!(step, TestCreateStep::Succeeded { .. }));
    assert_eq!(service.actions(), ["GetResource"]);
}

#[tokio::test]
async fn read_back_finality_applies_when_operation_first_reaches_success() {
    for boundary in ["create", "resume", "update"] {
        let (provider, service) = provider_with_script([
            (
                "UpdateResource",
                vec![mutation_reply("update-final", VPC_ID)],
            ),
            (
                "GetResourceRequestStatus",
                vec![status_reply("SUCCESS", Some(VPC_ID), None)],
            ),
            ("GetResource", vec![not_found()]),
        ])
        .await;
        let final_poll = PollRequest {
            finality: PollFinality::FinalAttempt,
        };
        match boundary {
            "create" => {
                let step = provider
                    .poll_create_step(
                        &vpc_resource(),
                        CcCreatePoll::<TestCreateAttempt, TestIssuedCreate>::Issued(
                            TestIssuedCreate {
                                identifier: vpc_identifier(),
                            },
                            in_flight_create_token(Some(VPC_ID)),
                        ),
                        final_poll,
                    )
                    .await
                    .unwrap();
                assert!(matches!(step, TestCreateStep::Partial { .. }));
            }
            "resume" => {
                let step = provider
                    .resume_create_step(
                        &vpc_resource(),
                        CcCreateResume::<TestCreateAttempt, TestIssuedCreate>::Issued(
                            TestIssuedCreate {
                                identifier: vpc_identifier(),
                            },
                            in_flight_create_token(Some(VPC_ID)),
                        ),
                        final_poll,
                    )
                    .await
                    .unwrap();
                assert!(matches!(step, TestCreateStep::Partial { .. }));
            }
            _ => {
                let (id, from, patch) = vpc_update();
                let (token, _) = expect_update_pending(
                    provider
                        .update_step(&id, &vpc_identifier(), &from, &patch)
                        .await
                        .unwrap(),
                );
                let step = provider
                    .poll_update_step(&id, &vpc_identifier(), &from, &patch, token, final_poll)
                    .await
                    .unwrap();
                let OperationStep::Done(outcome) = step else {
                    panic!("final update must be terminal")
                };
                assert!(matches!(
                    outcome.kind(),
                    UpdateOutcomeKind::PartialSuccess { .. }
                ));
            }
        }
        assert_eq!(service.calls("GetResourceRequestStatus"), 1);
        assert_eq!(service.calls("GetResource"), 1);
    }
}

#[tokio::test]
async fn read_back_rejects_mismatched_resource_and_identifier_before_reading() {
    let (provider, service) = provider_with_script([]).await;
    for (resource_type, identifier) in [("s3.Bucket", VPC_ID), ("ec2.Vpc", "vpc-other")] {
        let token = create_token(
            json!({"phase":"read_back", "identifier":identifier, "resource_type":resource_type}),
        );
        let failure = provider
            .poll_create_step(
                &vpc_resource(),
                CcCreatePoll::<TestCreateAttempt, TestIssuedCreate>::Issued(
                    TestIssuedCreate {
                        identifier: vpc_identifier(),
                    },
                    token,
                ),
                PollRequest {
                    finality: PollFinality::Continue,
                },
            )
            .await
            .unwrap_err();
        assert!(matches!(
            failure,
            TestCreateVerdict::InFlight {
                identifier: Some(_),
                ..
            }
        ));
    }
    assert!(service.actions().is_empty());
}

#[tokio::test]
async fn resume_read_back_in_fresh_provider_reads_by_identifier() {
    let (provider, service) = provider_with_script([
        (
            "CreateResource",
            vec![mutation_reply("create-fresh-resume", VPC_ID)],
        ),
        (
            "GetResourceRequestStatus",
            vec![status_reply("SUCCESS", Some(VPC_ID), None)],
        ),
        (
            "GetResource",
            vec![
                not_found(),
                get_resource_reply(VPC_TYPE, VPC_ID, vpc_properties()),
            ],
        ),
    ])
    .await;
    let resource = common::normalize_resource(vpc_resource()).await;
    let (operation, _) = expect_create_pending(
        provider
            .create_step(resource.as_resource(), TestCreateAttempt)
            .await
            .unwrap(),
    );
    let (pending, _) = expect_create_pending(
        poll_test_create(&provider, resource.as_resource(), operation)
            .await
            .unwrap(),
    );
    let encoded = serde_json::to_string(&pending).unwrap();
    drop(provider);
    let mock = MockAws::builder()
        .with_service(Arc::clone(&service))
        .build();
    let fresh = AwsccProvider::from_sdk_config(
        BoundedSdkConfig::for_test(mock.sdk_config("us-fresh-resume-1").await),
        &AwsccProviderConfig::default(),
    )
    .await;
    let token: CcCreateToken = serde_json::from_str(&encoded).unwrap();
    let request = resume_request_for_token(vpc_resource(), &token).await;
    let outcome = carina_core::provider::Provider::resume_create(&fresh, request)
        .await
        .unwrap();
    assert!(matches!(
        outcome.kind(),
        carina_core::provider::ResumeOutcomeKind::Resumed(_)
    ));
    assert_eq!(service.calls("CreateResource"), 1);
    assert_eq!(service.calls("GetResourceRequestStatus"), 1);
    assert_eq!(service.calls("GetResource"), 2);
    for body in service.bodies("GetResource") {
        assert_eq!(
            serde_json::from_str::<JsonValue>(&body).unwrap()["Identifier"],
            VPC_ID
        );
    }
}

#[tokio::test(start_paused = true)]
async fn native_resume_read_back_uses_schema_budget_without_reissuing_create() {
    let (provider, service) = provider_with_script([("GetResource", vec![not_found()])]).await;
    let provider = provider.with_operation_config_for_test(
        "ec2.Vpc",
        carina_core::schema::OperationConfig {
            read_back: carina_provider_protocol::types::ReadBackBudget::from_secs(1),
            ..Default::default()
        },
    );
    let token = read_back_token("ec2.Vpc", VPC_ID, &["vpc_id"]);
    let request = resume_request_for_token(vpc_resource(), &token).await;
    let outcome = carina_core::provider::Provider::resume_create(&provider, request)
        .await
        .unwrap();
    let carina_core::provider::ResumeOutcomeKind::PartialSuccess(partial) = outcome.kind() else {
        panic!("final resume must be partial")
    };
    assert_eq!(partial.identifier().as_str(), VPC_ID);
    let expected: Vec<_> = carina_provider_awscc::schemas::generated::get_config_by_type("ec2.Vpc")
        .unwrap()
        .schema
        .as_schema()
        .attributes()
        .keys()
        .cloned()
        .collect();
    assert_eq!(partial.diagnostic().missing_attributes(), expected);
    assert_eq!(service.actions(), ["GetResource", "GetResource"]);
}

#[tokio::test(start_paused = true)]
async fn native_update_read_back_uses_schema_budget_and_retains_host_deadline() {
    let (provider, service) = provider_with_script([
        (
            "UpdateResource",
            vec![mutation_reply("native-update", VPC_ID)],
        ),
        (
            "GetResourceRequestStatus",
            vec![status_reply("SUCCESS", Some(VPC_ID), None)],
        ),
        ("GetResource", vec![not_found()]),
    ])
    .await;
    let provider = provider.with_operation_config_for_test(
        "ec2.Vpc",
        carina_core::schema::OperationConfig {
            read_back: carina_provider_protocol::types::ReadBackBudget::from_secs(1),
            ..Default::default()
        },
    );
    let (id, from, patch) = vpc_update();
    let outcome = provider
        .update_resource(id, &vpc_identifier(), &from, &patch)
        .await
        .unwrap();
    assert_eq!(
        outcome.read_back_end(),
        carina_provider_protocol::ReadBackEnd::HostDeadline
    );
    assert_eq!(service.calls("UpdateResource"), 1);
    assert_eq!(service.calls("GetResourceRequestStatus"), 1);
    assert_eq!(service.calls("GetResource"), 2);
}

fn expect_delete_pending(
    step: OperationStep<(), ProviderError, CcDeleteToken>,
) -> (CcDeleteToken, u32) {
    match step {
        OperationStep::Pending {
            pending,
            poll_after_secs,
            phase,
        } => {
            assert_eq!(phase, PendingPhase::Operation);
            (pending, poll_after_secs)
        }
        OperationStep::Done(()) => panic!("expected pending delete"),
        OperationStep::Verdict(error) => panic!("expected pending delete, got {error}"),
    }
}

#[tokio::test]
async fn create_invalid_request_rejection_is_terminal_and_not_created() {
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

    let TestCreateVerdict::NotCreated(error) = failure else {
        panic!("CreateResource rejection must be not-created");
    };
    assert_cloudcontrol_api_error(
        &error,
        "Failed to create resource",
        "cloudcontrol.CreateResource",
        "InvalidRequestException",
        "resource type AWS::EC2::VPC: AWS SDK request failed: service error: InvalidRequestException: invalid create request: InvalidRequestException: invalid create request",
    );
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
                CcCreatePoll::Unissued(TestCreateAttempt, (token).into_operation().unwrap()),
                carina_provider_protocol::PollRequest {
                    finality: carina_provider_protocol::PollFinality::Continue,
                },
            )
            .await
            .unwrap_or_else(|error| panic!("same-token resend must recover tracking: {error:?}"));
        let (resent_token, resent_poll_after_secs) = expect_create_pending(resent);
        assert!(matches!(
            resent_token,
            CcCreateToken::Operation(CcCreateOperation::InFlight(ref create))
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

#[tokio::test]
async fn unconfirmed_create_without_request_token_uses_resend_backoff() {
    let (provider, service) = provider_with_script([(
        "CreateResource",
        vec![
            mutation_reply_without_request_token(None),
            mutation_reply_without_request_token(None),
        ],
    )])
    .await;
    let resource = common::normalize_resource(vpc_resource()).await;

    let initial = provider
        .create_step(resource.as_resource(), TestCreateAttempt)
        .await
        .expect("accepted create without a request token must stay tracked");
    let (token, initial_delay) = expect_create_pending(initial);
    assert_eq!(initial_delay, 1);

    let resent = provider
        .poll_create_step(
            resource.as_resource(),
            CcCreatePoll::Unissued(TestCreateAttempt, (token).into_operation().unwrap()),
            carina_provider_protocol::PollRequest {
                finality: carina_provider_protocol::PollFinality::Continue,
            },
        )
        .await
        .expect("an unconfirmed missing-token reply must stay tracked");
    let (token, delay) = expect_create_pending(resent);
    let encoded = serde_json::to_value(&token).expect("serialize unconfirmed create token");
    assert_eq!(encoded["phase"], "unconfirmed");
    assert_eq!(encoded["attempt"], 0);
    assert_eq!(encoded["resends"], 2);
    assert_eq!(delay, 20);

    let client_tokens = request_client_tokens(&service, "CreateResource");
    assert_eq!(client_tokens.len(), 2);
    assert_eq!(client_tokens[0], client_tokens[1]);
    assert_eq!(encoded["client_token"], client_tokens[0]);
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

    let failure = common::dispatch_create(&provider, &id, request)
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
            CcCreatePoll::Unissued(poll, (token.clone()).into_operation().unwrap()),
            carina_provider_protocol::PollRequest {
                finality: carina_provider_protocol::PollFinality::Continue,
            },
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
async fn unissued_poll_unconfirmed_rejection_is_in_flight_with_same_token() {
    let (provider, service) =
        provider_with_script([("CreateResource", vec![invalid_request()])]).await;
    let resource = common::normalize_resource(vpc_resource()).await;
    let token = create_token(json!({
        "phase": "unconfirmed",
        "attempt": 2,
        "resends": 7,
        "client_token": TEST_CLIENT_TOKEN,
    }));

    let failure = provider
        .poll_create_step(
            resource.as_resource(),
            CcCreatePoll::Unissued(TestCreateAttempt, (token.clone()).into_operation().unwrap()),
            carina_provider_protocol::PollRequest {
                finality: carina_provider_protocol::PollFinality::Continue,
            },
        )
        .await
        .expect_err("a rejected unconfirmed resend must remain in flight");

    let TestCreateVerdict::InFlight {
        token: preserved,
        identifier: None,
        error,
    } = failure
    else {
        panic!("an unissued unconfirmed rejection must remain in flight");
    };
    assert_eq!(preserved, token);
    assert_cloudcontrol_api_error(
        &error,
        "Failed to resend create resource; the outcome of the earlier send is unknown",
        "cloudcontrol.CreateResource",
        "InvalidRequestException",
        "resource type AWS::EC2::VPC: AWS SDK request failed: service error: InvalidRequestException: invalid create request: InvalidRequestException: invalid create request",
    );
    assert_eq!(
        request_client_tokens(&service, "CreateResource"),
        [TEST_CLIENT_TOKEN]
    );
}

#[tokio::test]
async fn unconfirmed_create_rejection_from_resume_preserves_in_flight_token() {
    let (provider, service) =
        provider_with_script([("CreateResource", vec![invalid_request()])]).await;
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
async fn invalid_resume_tags_are_rejected_before_dispatch() {
    let (_provider, service) =
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
    let preparation = common::try_create_request(resource.clone()).await;
    let error = preparation.expect_err("invalid tags must fail real create preparation");
    let carina_core::executor::ProviderPreparationError::ValueConstraint {
        resource: rejected,
        errors,
    } = error
    else {
        panic!("invalid tags must fail schema validation: {error:?}");
    };
    assert_eq!(rejected.as_ref(), &resource.id);
    let [carina_core::schema::TypeError::MapValueError { key, inner }] = errors.errors() else {
        panic!("expected one invalid tag value: {errors}");
    };
    assert_eq!(key, "env");
    assert!(matches!(
        inner.as_ref(),
        carina_core::schema::TypeError::TypeMismatch { expected, got }
            if expected == "String" && got == "Bool"
    ));
    assert_eq!(service.calls("CreateResource"), 0);
}

#[derive(Debug)]
struct UnavailableCredentials;

impl aws_credential_types::provider::ProvideCredentials for UnavailableCredentials {
    fn provide_credentials<'a>(
        &'a self,
    ) -> aws_credential_types::provider::future::ProvideCredentials<'a>
    where
        Self: 'a,
    {
        aws_credential_types::provider::future::ProvideCredentials::ready(Err(
            aws_credential_types::provider::error::CredentialsError::not_loaded(
                std::io::Error::other("test credentials unavailable"),
            ),
        ))
    }
}

#[tokio::test]
async fn unconfirmed_create_pre_send_failure_from_resume_preserves_pending_token() {
    let service = Arc::new(ScriptedCloudControl::new(std::iter::empty::<(
        &'static str,
        Vec<Reply>,
    )>()));
    let mock = MockAws::builder()
        .with_service(Arc::clone(&service))
        .build();
    let config = mock
        .sdk_config("us-east-1")
        .await
        .into_builder()
        .credentials_provider(
            aws_credential_types::provider::SharedCredentialsProvider::new(UnavailableCredentials),
        )
        .identity_cache(aws_sdk_cloudcontrol::config::IdentityCache::no_cache())
        .build();
    let provider = AwsccProvider::from_sdk_config(
        BoundedSdkConfig::for_test(config),
        &AwsccProviderConfig::default(),
    )
    .await;
    let resource = Resource::with_provider("awscc", "s3.Bucket", "resume", None)
        .with_attribute("bucket_name", string("carina-resume"))
        .with_attribute(
            "tags",
            Value::Concrete(ConcreteValue::Map(IndexMap::from([(
                "env".to_string(),
                string("test"),
            )]))),
        );
    let token = create_token(json!({
        "phase": "unconfirmed", "attempt": 2, "resends": 7,
        "client_token": TEST_CLIENT_TOKEN,
    }));
    let request = resume_request_for_token(resource, &token).await;
    let failure = tokio::time::timeout(
        std::time::Duration::from_secs(5),
        carina_core::executor::dispatch_resume_create(&provider, request),
    )
    .await
    .expect("permanent pre-send failure must settle promptly")
    .expect_err("a pre-send failure cannot disprove the earlier create send");
    let ResumeFailureKind::InFlight { pending, error } = failure.into_kind() else {
        panic!("an unconfirmed pre-send failure must remain in flight");
    };
    assert_eq!(
        pending.token().expose_for_provider_call(),
        serde_json::to_string(&token).expect("serialize expected create continuation"),
    );
    assert_eq!(pending.identifier(), None);
    assert_eq!(error.variant_name(), "api_error");
    assert_eq!(
        error.detail().context(),
        "Failed to resend create resource; the outcome of the earlier send is unknown"
    );
    assert!(error.to_string().contains("test credentials unavailable"));
    assert_eq!(service.calls("CreateResource"), 0);
    assert!(service.actions().is_empty(), "no HTTP request may be sent");
}

#[tokio::test]
async fn issued_poll_unconfirmed_client_token_conflict_retries_with_same_token() {
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

    let step = provider
        .poll_create_step::<TestCreateAttempt>(
            resource.as_resource(),
            CcCreatePoll::Issued(
                TestIssuedCreate {
                    identifier: issued_identifier.clone(),
                },
                token,
            ),
            PollRequest {
                finality: PollFinality::Continue,
            },
        )
        .await
        .expect("ClientTokenConflictException must retry an unconfirmed create");

    let TestCreateStep::Pending {
        token,
        poll_after_secs,
        issued,
    } = step
    else {
        panic!("a retryable client-token conflict must remain pending");
    };
    let encoded = serde_json::to_value(&token).expect("serialize retried create token");
    assert_eq!(encoded["phase"], "unconfirmed");
    assert_eq!(encoded["attempt"], 2);
    assert_eq!(encoded["resends"], 8);
    assert_eq!(encoded["client_token"], TEST_CLIENT_TOKEN);
    assert_eq!(poll_after_secs, 120);
    assert_eq!(issued, Some(issued_identifier));
    assert_eq!(
        request_client_tokens(&service, "CreateResource"),
        [TEST_CLIENT_TOKEN]
    );
}

#[tokio::test]
async fn issued_poll_unconfirmed_non_retryable_rejection_preserves_create() {
    let (provider, service) =
        provider_with_script([("CreateResource", vec![invalid_request()])]).await;
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
            CcCreatePoll::Issued(
                TestIssuedCreate {
                    identifier: issued_identifier.clone(),
                },
                token.clone(),
            ),
            PollRequest {
                finality: PollFinality::Continue,
            },
        )
        .await
        .expect_err("a permanent rejection cannot disprove an earlier create send");

    let TestCreateVerdict::InFlight {
        token: preserved,
        identifier: Some(identifier),
        error,
    } = failure
    else {
        panic!("an issued unconfirmed rejection must preserve the create");
    };
    assert_eq!(preserved, token);
    assert_eq!(identifier, issued_identifier);
    assert_cloudcontrol_api_error(
        &error,
        "Failed to resend create resource; the outcome of the earlier send is unknown",
        "cloudcontrol.CreateResource",
        "InvalidRequestException",
        "resource type AWS::EC2::VPC: AWS SDK request failed: service error: InvalidRequestException: invalid create request: InvalidRequestException: invalid create request",
    );
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
        assert_eq!(
            error
                .detail()
                .resource_id
                .as_deref()
                .map(|id| id.as_resource_id()),
            Some(pending.id())
        );
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
            Some(issued_identifier.clone()),
        ),
    ];

    for (label, token, expected_identifier) in cases {
        let resume = match &expected_identifier {
            Some(identifier) => CcCreateResume::Issued(
                TestIssuedCreate {
                    identifier: identifier.clone(),
                },
                token.clone(),
            ),
            None => {
                CcCreateResume::Unissued(TestCreateAttempt, token.clone().into_operation().unwrap())
            }
        };
        let failure = match provider
            .resume_create_step(
                &resource,
                resume,
                carina_provider_protocol::PollRequest {
                    finality: carina_provider_protocol::PollFinality::Continue,
                },
            )
            .await
        {
            Err(failure) => failure,
            Ok(step) => {
                panic!("{label} unknown-type resume must preserve its error, got {step:?}")
            }
        };
        let TestCreateVerdict::InFlight {
            token: preserved,
            identifier,
            error,
        } = failure
        else {
            panic!("{label} unknown-type resume must use the preserving verdict");
        };
        assert_eq!(preserved, token);
        assert_eq!(identifier, expected_identifier);
        assert_eq!(error.variant_name(), "internal");
        assert_eq!(
            error.detail().context(),
            "Unknown resource type: unknown.Resource"
        );
        assert_eq!(
            error
                .detail()
                .resource_id
                .as_deref()
                .map(|id| id.as_resource_id()),
            Some(&resource.id)
        );
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
        token: CcCreateToken::Operation(CcCreateOperation::InFlight(create)),
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
    let provider = provider.with_operation_config_for_test(
        "ec2.Vpc",
        carina_core::schema::OperationConfig {
            create_timeout_secs: Some(0),
            ..Default::default()
        },
    );
    let resource = vpc_resource();
    let id = resource.id.clone();
    let request = common::create_request(resource).await;

    // The protocol driver owns a std::time deadline. A zero schema budget
    // reaches its final operation poll without pretending Tokio time advances it.
    let failure = common::dispatch_create(&provider, &id, request)
        .await
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
        CcCreateToken::Operation(CcCreateOperation::InFlight(create))
            if create.request() == &RequestToken::from("create-budget")
                && create.attempt() == 0
                && create.identifier() == Some(VPC_ID)
    ));
    assert_eq!(error.variant_name(), "timeout");
    assert_eq!(service.calls("CreateResource"), 1);
    assert_eq!(service.calls("GetResourceRequestStatus"), 1);
    assert_eq!(service.calls("GetResource"), 0);
}

#[tokio::test(start_paused = true)]
async fn status_sdk_error_preserves_the_issued_create_in_flight() {
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

    let failure = poll_test_create(&provider, resource.as_resource(), token.clone())
        .await
        .expect_err("status SDK errors must report an in-flight verdict");
    let TestCreateVerdict::InFlight {
        token: next_token,
        identifier: Some(identifier),
        error,
    } = failure
    else {
        panic!("status SDK error must preserve the issued create");
    };
    assert_eq!(next_token, token);
    assert_eq!(identifier.as_str(), VPC_ID);
    assert_eq!(next_token.poll_after_secs(), 5);
    assert_eq!(error.detail().context(), "Failed to get operation status");
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
            CcCreateResume::<TestCreateAttempt, TestIssuedCreate>::Issued(
                TestIssuedCreate {
                    identifier: vpc_identifier(),
                },
                token.clone(),
            ),
            carina_provider_protocol::PollRequest {
                finality: carina_provider_protocol::PollFinality::Continue,
            },
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
            CcCreateResume::<TestCreateAttempt, TestIssuedCreate>::Issued(
                TestIssuedCreate {
                    identifier: vpc_identifier(),
                },
                in_flight_create_token(Some(VPC_ID)),
            ),
            carina_provider_protocol::PollRequest {
                finality: carina_provider_protocol::PollFinality::Continue,
            },
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
async fn resume_success_then_not_found_enters_read_back() {
    let (provider, service) = provider_with_script([
        (
            "GetResourceRequestStatus",
            vec![status_reply("SUCCESS", Some(VPC_ID), None)],
        ),
        ("GetResource", vec![not_found()]),
    ])
    .await;
    let resource = common::normalize_resource(vpc_resource()).await;

    let step = provider
        .resume_create_step(
            resource.as_resource(),
            CcCreateResume::<TestCreateAttempt, TestIssuedCreate>::Issued(
                TestIssuedCreate {
                    identifier: vpc_identifier(),
                },
                in_flight_create_token(Some(VPC_ID)),
            ),
            PollRequest {
                finality: PollFinality::Continue,
            },
        )
        .await
        .expect("successful mutation with a missing read must remain pending");
    let (token, _) = expect_create_pending(step);
    assert_eq!(token.identifier(), Some(VPC_ID));
    assert_eq!(token.phase(), PendingPhase::ReadBack);
    assert_eq!(
        service.actions(),
        ["GetResourceRequestStatus", "GetResource"]
    );
}

#[tokio::test]
async fn resume_failed_with_identifier_returns_identifier_reported_failure_with_observation() {
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
            CcCreateResume::<TestCreateAttempt, TestIssuedCreate>::Issued(
                TestIssuedCreate {
                    identifier: vpc_identifier(),
                },
                in_flight_create_token(Some(VPC_ID)),
            ),
            carina_provider_protocol::PollRequest {
                finality: carina_provider_protocol::PollFinality::Continue,
            },
        )
        .await
        .expect_err("FAILED with an identifier must never become success");

    let TestCreateVerdict::IdentifierReported {
        identifier,
        observation,
        error,
    } = failure
    else {
        panic!("FAILED with an identifier must preserve an identifier-reported verdict");
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
async fn resume_expired_token_with_identifier_and_not_found_leaves_no_resource() {
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
            CcCreateResume::<TestCreateAttempt, TestIssuedCreate>::Issued(
                TestIssuedCreate {
                    identifier: vpc_identifier(),
                },
                token,
            ),
            carina_provider_protocol::PollRequest {
                finality: carina_provider_protocol::PollFinality::Continue,
            },
        )
        .await
        .expect_err("an expired token plus NotFound proves there is no create to keep");

    let TestCreateVerdict::NoResource(error) = failure else {
        panic!("expired token plus NotFound must discard the in-flight create");
    };
    assert!(error.to_string().contains("no longer known"));
    assert_eq!(
        service.actions(),
        ["GetResourceRequestStatus", "GetResource"]
    );
}

#[tokio::test]
async fn poll_expired_token_with_identifier_and_not_found_leaves_no_resource() {
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

    let TestCreateVerdict::NoResource(error) = failure else {
        panic!("expired token plus NotFound must discard the pending create");
    };
    assert!(error.to_string().contains("no longer known"));
    assert_eq!(
        service.actions(),
        ["GetResourceRequestStatus", "GetResource"]
    );
}

#[tokio::test]
async fn resume_expired_token_without_identifier_preserves_pending_token() {
    let (provider, service) = provider_with_script([(
        "GetResourceRequestStatus",
        vec![Reply::Error {
            status: 400,
            code: "RequestTokenNotFoundException",
            message: "request token expired",
        }],
    )])
    .await;
    let token = in_flight_create_token(None);
    let request = resume_request_for_token(vpc_resource(), &token).await;
    let failure = carina_core::executor::dispatch_resume_create(&provider, request)
        .await
        .expect_err("an unknown request with no identifier remains in flight");

    let ResumeFailureKind::InFlight { pending, error } = failure.into_kind() else {
        panic!("expired token without an identifier must remain in flight");
    };
    assert_eq!(
        pending.token().expose_for_provider_call(),
        serde_json::to_string(&token).expect("serialize expected create continuation")
    );
    assert_eq!(pending.identifier(), None);
    assert_eq!(error.variant_name(), "api_error");
    assert_eq!(
        error.detail().context(),
        "Cloud Control request token is no longer known"
    );
    assert_eq!(service.calls("GetResource"), 0);
}

#[tokio::test(start_paused = true)]
async fn resume_status_sdk_error_without_identifier_stays_in_flight_with_backoff() {
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
    let failure = provider
        .resume_create_step(
            resource.as_resource(),
            CcCreateResume::<TestCreateAttempt, TestIssuedCreate>::Unissued(
                TestCreateAttempt,
                (token.clone()).into_operation().unwrap(),
            ),
            carina_provider_protocol::PollRequest {
                finality: carina_provider_protocol::PollFinality::Continue,
            },
        )
        .await
        .expect_err("status SDK errors must report an in-flight verdict");

    let TestCreateVerdict::InFlight {
        token: next_token,
        identifier: None,
        error,
    } = failure
    else {
        panic!("unissued resume status error must preserve the accepted create");
    };
    assert_eq!(next_token, token);
    assert_eq!(next_token.poll_after_secs(), 5);
    assert_eq!(error.detail().context(), "Failed to get operation status");
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
            CcCreateResume::<TestCreateAttempt, TestIssuedCreate>::Unissued(
                TestCreateAttempt,
                (create_token(json!({
                    "phase": "not_issued",
                    "attempt": 1,
                    "client_token": TEST_CLIENT_TOKEN,
                })))
                .into_operation()
                .unwrap(),
            ),
            carina_provider_protocol::PollRequest {
                finality: carina_provider_protocol::PollFinality::Continue,
            },
        )
        .await
        .expect("NotIssued resume must resend the desired state");

    let TestCreateStep::Pending {
        token: CcCreateToken::Operation(CcCreateOperation::InFlight(create)),
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
async fn resume_cancel_complete_with_identifier_reads_back_identifier_reported_failure() {
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
            CcCreateResume::<TestCreateAttempt, TestIssuedCreate>::Issued(
                TestIssuedCreate {
                    identifier: vpc_identifier(),
                },
                in_flight_create_token(Some(VPC_ID)),
            ),
            carina_provider_protocol::PollRequest {
                finality: carina_provider_protocol::PollFinality::Continue,
            },
        )
        .await
        .expect_err("a canceled create with an identifier may have created a resource");

    let TestCreateVerdict::IdentifierReported {
        identifier,
        observation,
        error,
    } = failure
    else {
        panic!("CANCEL_COMPLETE with an identifier must preserve an identifier-reported verdict");
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
        CcCreateToken::Operation(CcCreateOperation::InFlight(create))
            if create.request() == &RequestToken::from("create-0")
                && create.attempt() == 0
                && create.identifier() == Some(VPC_ID)
    ));
    let expected = token.clone().with_poll_after_secs(5);

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
        CcCreateToken::Operation(CcCreateOperation::NotIssued(create)) if create.attempt() == 1
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
        CcCreateToken::Operation(CcCreateOperation::InFlight(create))
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

#[tokio::test]
async fn not_issued_client_token_conflict_retries_with_the_same_client_token() {
    let (provider, service) = provider_with_script([(
        "CreateResource",
        vec![
            client_token_conflict(),
            mutation_reply("create-after-conflict", VPC_ID),
        ],
    )])
    .await;
    let resource = common::normalize_resource(vpc_resource()).await;

    let (token, delay) = expect_create_pending(
        provider
            .create_step(resource.as_resource(), TestCreateAttempt)
            .await
            .expect("a not-issued client-token conflict must schedule a bounded retry"),
    );
    assert!(matches!(
        &token,
        CcCreateToken::Operation(CcCreateOperation::NotIssued(create)) if create.attempt() == 1
    ));
    assert_eq!(delay, 10);

    let (token, delay) = expect_create_pending(
        poll_test_create(&provider, resource.as_resource(), token)
            .await
            .expect("the bounded retry must reuse the original client token"),
    );
    assert!(matches!(
        token,
        CcCreateToken::Operation(CcCreateOperation::InFlight(create))
            if create.request() == &RequestToken::from("create-after-conflict")
                && create.attempt() == 1
                && create.identifier() == Some(VPC_ID)
    ));
    assert_eq!(delay, 1);
    assert_eq!(service.calls("CreateResource"), 2);
    let client_tokens = request_client_tokens(&service, "CreateResource");
    assert_eq!(client_tokens.len(), 2);
    assert!(common::is_uuidish(&client_tokens[0]));
    assert_eq!(client_tokens[1], client_tokens[0]);
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
        CcCreateToken::Operation(CcCreateOperation::NotIssued(create)) if create.attempt() == 1
    ));

    let step = provider
        .resume_create_step(
            resource.as_resource(),
            CcCreateResume::<TestCreateAttempt, TestIssuedCreate>::Unissued(
                TestCreateAttempt,
                (token).into_operation().unwrap(),
            ),
            carina_provider_protocol::PollRequest {
                finality: carina_provider_protocol::PollFinality::Continue,
            },
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
        CcCreateToken::Operation(CcCreateOperation::NotIssued(create)) if create.attempt() == 1
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
        CcCreateToken::Operation(CcCreateOperation::InFlight(create))
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
        CcCreateToken::Operation(CcCreateOperation::InFlight(create))
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
        CcCreateToken::Operation(CcCreateOperation::NotIssued(create)) if create.attempt() == 1
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

    let token = in_flight_create_token(None);
    let failure = poll_test_create(&provider, resource.as_resource(), token.clone())
        .await
        .expect_err("a retryable-looking code must not override a permanent message");
    let TestCreateVerdict::InFlight {
        token: preserved,
        identifier: None,
        error,
    } = failure
    else {
        panic!("an identifier-less accepted create must remain tracked");
    };
    assert_eq!(preserved, token);
    assert_eq!(
        error.detail().context(),
        "Cloud Control create ended with FAILED: invalid configuration"
    );
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
        CcCreateToken::Operation(CcCreateOperation::NotIssued(create)) if create.attempt() == 1
    ));
    assert_eq!(delay, 10);
    assert_eq!(service.calls("CreateResource"), 0);
}

#[tokio::test]
async fn retryable_failed_create_with_identifier_is_identifier_reported_without_reissue() {
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
        TestCreateVerdict::IdentifierReported {
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
        other => panic!("expected identifier-reported failure, got {other:?}"),
    }
    assert_eq!(service.calls("CreateResource"), 1);
    assert_eq!(service.calls("GetResourceRequestStatus"), 1);
}

#[tokio::test]
async fn cancel_complete_without_identifier_remains_in_flight_without_reissue() {
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

    let expected = token.clone();
    let failure = poll_test_create(&provider, resource.as_resource(), token)
        .await
        .expect_err("a deliberately canceled create must not be re-issued");
    let TestCreateVerdict::InFlight {
        token: preserved,
        identifier: None,
        error,
    } = failure
    else {
        panic!("identifier-less CANCEL_COMPLETE must remain tracked");
    };
    assert_eq!(preserved, expected);
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
    let request = delete_request();
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
    let request = delete_request();

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
    let request = delete_request();
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
async fn delete_one_post_send_timeout_then_uncapped_throttling_stays_pending() {
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
    let request = delete_request();

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
async fn delete_repeated_post_send_timeouts_past_max_attempts_stay_pending() {
    let (provider, service) = provider_with_post_send_failures(
        [(
            "DeleteResource",
            vec![mutation_reply("never-returned", VPC_ID)],
        )],
        [("DeleteResource", vec![true])],
    )
    .await;
    let id = ResourceId::with_provider_identity("awscc", "ec2.Vpc", "pending", None);
    let request = delete_request();

    let (mut token, delay) = expect_delete_pending(
        provider
            .delete_step(&id, &vpc_identifier(), &request)
            .await
            .expect("first post-send timeout must remain pending"),
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
                        "post-send timeout on delete resend {expected_resends} was capped: {error}"
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
    let request = delete_request();

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

    let error = provider
        .poll_delete_step(&id, &vpc_identifier(), &request, token)
        .await
        .expect_err("retryable service rejection should still honor the retry cap");
    assert_cloudcontrol_api_error(
        &error,
        "Failed to delete resource",
        "cloudcontrol.DeleteResource",
        "ThrottlingException",
        "resource type AWS::EC2::VPC, identifier vpc-0123456789abcdef0: AWS SDK request failed: service error: ThrottlingException: Rate exceeded: ThrottlingException: Rate exceeded",
    );
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
    let request = delete_request();

    let (token, delay) = expect_delete_pending(
        provider
            .delete_step(&id, &vpc_identifier(), &request)
            .await
            .expect("issue original delete"),
    );
    assert!(matches!(
        &token,
        CcDeleteToken::Delete(CcRequestPhase::InFlight(request))
            if request.request() == &RequestToken::from("delete-failed")
                && request.attempt() == 0
    ));
    assert_eq!(delay, 1);

    let (token, delay) = expect_delete_pending(
        provider
            .poll_delete_step(&id, &vpc_identifier(), &request, token)
            .await
            .expect("terminal retryable failure should schedule a new request"),
    );
    assert!(matches!(
        &token,
        CcDeleteToken::Delete(CcRequestPhase::NotIssued(request))
            if request.attempt() == 1
    ));
    assert_eq!(delay, 10);

    let (token, delay) = expect_delete_pending(
        provider
            .poll_delete_step(&id, &vpc_identifier(), &request, token)
            .await
            .expect("outcome-unknown resend should preserve the new request"),
    );
    assert!(matches!(
        &token,
        CcDeleteToken::Delete(CcRequestPhase::Unconfirmed(request))
            if request.attempt() == 1 && request.resends() == 1
    ));
    assert_eq!(delay, 10);

    let (token, delay) = expect_delete_pending(
        provider
            .poll_delete_step(&id, &vpc_identifier(), &request, token)
            .await
            .expect("second resend should recover the new request"),
    );
    assert!(matches!(
        token,
        CcDeleteToken::Delete(CcRequestPhase::InFlight(request))
            if request.request() == &RequestToken::from("delete-reissued")
                && request.attempt() == 1
    ));
    assert_eq!(delay, 1);

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
    let request = delete_request();
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
    let request = delete_request();
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
async fn non_retryable_failed_status_preserves_in_flight_and_identifier_reported_outcomes() {
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
    let expected = token.clone();
    let error = poll_test_create(&provider, resource.as_resource(), token)
        .await
        .expect_err("failed create without identifier must error");
    let TestCreateVerdict::InFlight {
        token: preserved,
        identifier: None,
        error,
    } = error
    else {
        panic!("failure without identifier must remain tracked");
    };
    assert_eq!(preserved, expected);
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
        TestCreateVerdict::IdentifierReported { error, .. } => {
            assert!(error.to_string().contains("post-create read denied"));
        }
        other => panic!("expected identifier-reported failure, got {other:?}"),
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
            CcCreateToken::Operation(CcCreateOperation::NotIssued(create)) if create.attempt() == attempt + 1
        ));
        assert_eq!(delay, expected_delay);
        token = next;
    }

    let error = poll_test_create(&provider, resource.as_resource(), token)
        .await
        .expect_err("the final configured retry must be exhausted");
    let TestCreateVerdict::NotCreated(error) = error else {
        panic!("exhausted unissued retry must be not-created");
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

    let token = create_token(json!({
        "phase": "in_flight",
        "request": "create-final",
        "attempt": 12,
    }));
    let error = poll_test_create(&provider, resource.as_resource(), token.clone())
        .await
        .expect_err("final retryable status should return its real failure");

    let TestCreateVerdict::InFlight {
        token: preserved,
        identifier: None,
        error,
    } = error
    else {
        panic!("exhausted status retry without an identifier must remain tracked");
    };
    assert_eq!(preserved, token);
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
            ..
        } => {
            assert_eq!(poll_after_secs, 1);
            pending
        }
        other => panic!("expected pending update, got {other:?}"),
    };
    assert!(matches!(
        &token,
        CcUpdateToken::Operation(CcRequestPhase::InFlight(request))
            if request.request() == &RequestToken::from("update-0")
                && request.attempt() == 0
    ));
    match provider
        .poll_update_step(
            &id,
            &vpc_identifier(),
            &from,
            &patch,
            token,
            carina_provider_protocol::PollRequest {
                finality: carina_provider_protocol::PollFinality::Continue,
            },
        )
        .await
        .expect("poll update")
    {
        OperationStep::Done(outcome) => {
            let UpdateOutcomeKind::Success { state } = outcome.kind() else {
                panic!("expected successful update, got {outcome:?}");
            };
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
            ..
        } => (pending, poll_after_secs),
        other => panic!("expected pending update, got {other:?}"),
    };
    assert!(matches!(
        &token,
        CcUpdateToken::Operation(CcRequestPhase::NotIssued(request)) if request.attempt() == 1
    ));
    assert_eq!(delay, 10);
    assert_eq!(service.calls("UpdateResource"), 3);

    let (token, delay) = expect_update_pending(
        provider
            .poll_update_step(
                &id,
                &vpc_identifier(),
                &from,
                &patch,
                token,
                carina_provider_protocol::PollRequest {
                    finality: carina_provider_protocol::PollFinality::Continue,
                },
            )
            .await
            .expect("re-send update"),
    );
    assert!(matches!(
        token,
        CcUpdateToken::Operation(CcRequestPhase::InFlight(request))
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
        CcUpdateToken::Operation(CcRequestPhase::Unconfirmed(request)) if request.attempt() == 0
    ));
    assert_eq!(delay, 10);

    let (token, delay) = expect_update_pending(
        provider
            .poll_update_step(
                &id,
                &vpc_identifier(),
                &from,
                &patch,
                token,
                carina_provider_protocol::PollRequest {
                    finality: carina_provider_protocol::PollFinality::Continue,
                },
            )
            .await
            .expect("resending with the same token should recover the request"),
    );
    assert!(matches!(
        token,
        CcUpdateToken::Operation(CcRequestPhase::InFlight(request))
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
        .poll_update_step(
            &id,
            &vpc_identifier(),
            &from,
            &patch,
            token,
            carina_provider_protocol::PollRequest {
                finality: carina_provider_protocol::PollFinality::Continue,
            },
        )
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
async fn update_one_post_send_timeout_then_uncapped_throttling_stays_pending() {
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
                .poll_update_step(
                    &id,
                    &vpc_identifier(),
                    &from,
                    &patch,
                    token,
                    carina_provider_protocol::PollRequest {
                        finality: carina_provider_protocol::PollFinality::Continue,
                    },
                )
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
async fn update_repeated_post_send_timeouts_past_max_attempts_stay_pending() {
    let (provider, service) = provider_with_post_send_failures(
        [(
            "UpdateResource",
            vec![mutation_reply("never-returned", VPC_ID)],
        )],
        [("UpdateResource", vec![true])],
    )
    .await;
    let (id, from, patch) = vpc_update();

    let (mut token, delay) = expect_update_pending(
        provider
            .update_step(&id, &vpc_identifier(), &from, &patch)
            .await
            .expect("first post-send timeout must remain pending"),
    );
    let encoded = serde_json::to_value(&token).expect("serialize update continuation");
    assert_eq!(encoded["phase"], "unconfirmed");
    assert_eq!(encoded["attempt"], 0);
    assert_eq!(encoded["resends"], 1);
    assert_eq!(delay, 10);

    for expected_resends in 2..=14 {
        let (next, delay) = expect_update_pending(
            provider
                .poll_update_step(
                    &id,
                    &vpc_identifier(),
                    &from,
                    &patch,
                    token,
                    carina_provider_protocol::PollRequest {
                        finality: carina_provider_protocol::PollFinality::Continue,
                    },
                )
                .await
                .unwrap_or_else(|error| {
                    panic!(
                        "post-send timeout on update resend {expected_resends} was capped: {error}"
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
                .poll_update_step(
                    &id,
                    &vpc_identifier(),
                    &from,
                    &patch,
                    token,
                    carina_provider_protocol::PollRequest {
                        finality: carina_provider_protocol::PollFinality::Continue,
                    },
                )
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
                .poll_update_step(
                    &id,
                    &vpc_identifier(),
                    &from,
                    &patch,
                    token,
                    carina_provider_protocol::PollRequest {
                        finality: carina_provider_protocol::PollFinality::Continue,
                    },
                )
                .await
                .unwrap_or_else(|error| {
                    panic!("throttled update attempt {expected_attempt} failed early: {error}")
                }),
        );
        token = next;
    }

    let error = provider
        .poll_update_step(
            &id,
            &vpc_identifier(),
            &from,
            &patch,
            token,
            carina_provider_protocol::PollRequest {
                finality: carina_provider_protocol::PollFinality::Continue,
            },
        )
        .await
        .expect_err("retryable service rejection should still honor the retry cap");
    assert_cloudcontrol_api_error(
        &error,
        "Failed to update resource",
        "cloudcontrol.UpdateResource",
        "ThrottlingException",
        "resource type AWS::EC2::VPC, identifier vpc-0123456789abcdef0: AWS SDK request failed: service error: ThrottlingException: Rate exceeded: ThrottlingException: Rate exceeded",
    );
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
        .delete_step(&id, &vpc_identifier(), &delete_request())
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
        OperationStep::Done(outcome) => {
            let UpdateOutcomeKind::Success { state } = outcome.kind() else {
                panic!("expected successful update, got {outcome:?}");
            };
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
        OperationStep::Done(outcome) if matches!(outcome.kind(), UpdateOutcomeKind::Success { .. })
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
    let request = delete_request();

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
    let request = delete_request();

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
    let request = delete_request();

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
    let request = delete_request();
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
    let request = delete_request();
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
            .delete_step(&id, &internet_gateway_identifier(), &delete_request())
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

fn read_back_continuation_token(resource_type: &str, identifier: &str) -> CcCreateToken {
    read_back_token(
        resource_type,
        identifier,
        if resource_type == "iam.Role" {
            &["arn"]
        } else {
            &["vpc_id"]
        },
    )
}

async fn poll_read_back_create(
    provider: &AwsccProvider,
    resource: &Resource,
    token: CcCreateToken,
    finality: PollFinality,
) -> TestCreateStep {
    let poll = test_create_poll(&token);
    provider
        .poll_create_step(resource, poll, PollRequest { finality })
        .await
        .unwrap()
}

#[tokio::test]
async fn permanent_read_errors_finish_partial_on_initial_read_and_poll() {
    let bad_properties = Reply::Json(json!({"ResourceDescription":{"Properties":"{broken json"}}));
    for reply in [
        Reply::Error {
            status: 403,
            code: "AccessDeniedException",
            message: "read denied",
        },
        Reply::Error {
            status: 400,
            code: "ValidationException",
            message: "invalid identifier",
        },
        bad_properties,
    ] {
        for already_pending in [false, true] {
            let (provider, service) = provider_with_script([
                (
                    "GetResourceRequestStatus",
                    vec![status_reply("SUCCESS", Some(VPC_ID), None)],
                ),
                ("GetResource", vec![reply.clone()]),
            ])
            .await;
            let resource = vpc_resource();
            let token = if already_pending {
                read_back_continuation_token("ec2.Vpc", VPC_ID)
            } else {
                in_flight_create_token(Some(VPC_ID))
            };
            let step =
                poll_read_back_create(&provider, &resource, token, PollFinality::Continue).await;
            let TestCreateStep::Partial {
                identifier, reason, ..
            } = step
            else {
                panic!("permanent read error must finish immediately: {step:?}")
            };
            assert_eq!(identifier, vpc_identifier());
            assert!(!reason.is_empty());
            assert_eq!(service.calls("GetResource"), 1);

            let (provider, service) = provider_with_script([
                (
                    "UpdateResource",
                    vec![mutation_reply("update-review", VPC_ID)],
                ),
                (
                    "GetResourceRequestStatus",
                    vec![status_reply("SUCCESS", Some(VPC_ID), None)],
                ),
                ("GetResource", vec![reply.clone()]),
            ])
            .await;
            let (id, from, patch) = vpc_update();
            let token = if already_pending {
                let CcCreateToken::ReadBack(continuation) =
                    read_back_continuation_token("ec2.Vpc", VPC_ID)
                else {
                    panic!("read token")
                };
                CcUpdateToken::ReadBack(continuation.into())
            } else {
                expect_update_pending(
                    provider
                        .update_step(&id, &vpc_identifier(), &from, &patch)
                        .await
                        .unwrap(),
                )
                .0
            };
            let step = provider
                .poll_update_step(
                    &id,
                    &vpc_identifier(),
                    &from,
                    &patch,
                    token,
                    PollRequest {
                        finality: PollFinality::Continue,
                    },
                )
                .await
                .unwrap();
            let OperationStep::Done(outcome) = step else {
                panic!("permanent update read error must finish immediately: {step:?}")
            };
            assert!(matches!(
                outcome.kind(),
                UpdateOutcomeKind::PartialSuccess { .. }
            ));
            assert_eq!(service.calls("GetResource"), 1);
        }
    }
}

#[tokio::test]
async fn transient_read_errors_wait_then_complete() {
    for (reply, dispatch) in [
        (throttled(), false),
        (service_internal_error(), false),
        (get_resource_reply(VPC_TYPE, VPC_ID, vpc_properties()), true),
    ] {
        let reads = vec![reply.clone(); 3]
            .into_iter()
            .chain([get_resource_reply(VPC_TYPE, VPC_ID, vpc_properties())])
            .collect();
        let (provider, service) = provider_with_post_send_failures(
            [("GetResource", reads)],
            if dispatch {
                vec![("GetResource", vec![true, false])]
            } else {
                vec![]
            },
        )
        .await;
        let resource = vpc_resource();
        let first = poll_read_back_create(
            &provider,
            &resource,
            read_back_continuation_token("ec2.Vpc", VPC_ID),
            PollFinality::Continue,
        )
        .await;
        let (token, _) = expect_create_pending(first);
        assert!(matches!(
            poll_read_back_create(&provider, &resource, token, PollFinality::Continue).await,
            TestCreateStep::Succeeded { .. }
        ));
        assert_eq!(service.calls("GetResource"), 4);
        let reads = vec![reply; 3]
            .into_iter()
            .chain([get_resource_reply(VPC_TYPE, VPC_ID, vpc_properties())])
            .collect();
        let (provider, service) = provider_with_post_send_failures(
            [("GetResource", reads)],
            if dispatch {
                vec![("GetResource", vec![true, false])]
            } else {
                vec![]
            },
        )
        .await;
        let (id, from, patch) = vpc_update();
        let CcCreateToken::ReadBack(continuation) = read_back_continuation_token("ec2.Vpc", VPC_ID)
        else {
            panic!("read token")
        };
        let OperationStep::Pending {
            pending,
            phase: PendingPhase::ReadBack,
            ..
        } = provider
            .poll_update_step(
                &id,
                &vpc_identifier(),
                &from,
                &patch,
                CcUpdateToken::ReadBack(continuation.into()),
                PollRequest {
                    finality: PollFinality::Continue,
                },
            )
            .await
            .unwrap()
        else {
            panic!("transient update read must wait")
        };
        let OperationStep::Done(outcome) = provider
            .poll_update_step(
                &id,
                &vpc_identifier(),
                &from,
                &patch,
                pending,
                PollRequest {
                    finality: PollFinality::Continue,
                },
            )
            .await
            .unwrap()
        else {
            panic!("transient update read must recover")
        };
        assert!(matches!(outcome.kind(), UpdateOutcomeKind::Success { .. }));
        assert_eq!(service.calls("GetResource"), 4);
    }
}

#[tokio::test]
async fn final_partial_retains_create_request_attributes_after_poll() {
    let (provider, _) = provider_with_script([("GetResource", vec![not_found()])]).await;
    let resource = vpc_resource();
    let first = poll_read_back_create(
        &provider,
        &resource,
        read_back_continuation_token("ec2.Vpc", VPC_ID),
        PollFinality::Continue,
    )
    .await;
    let (token, _) = expect_create_pending(first);
    let TestCreateStep::Partial {
        attributes,
        missing_attributes,
        ..
    } = poll_read_back_create(&provider, &resource, token, PollFinality::FinalAttempt).await
    else {
        panic!("final partial")
    };
    assert_eq!(attributes, resource.resolved_attributes());
    let mut expected = carina_provider_awscc::schemas::generated::get_config_by_type("ec2.Vpc")
        .unwrap()
        .schema
        .as_schema()
        .attributes()
        .keys()
        .cloned()
        .collect::<Vec<_>>();
    expected.sort();
    assert_eq!(missing_attributes, expected);
}

#[tokio::test]
async fn final_partial_retains_post_update_attributes_after_poll() {
    let (provider, _) = provider_with_script([("GetResource", vec![not_found()])]).await;
    let (id, from, mut patch) = vpc_update();
    patch.ops.push(PatchOp::Remove {
        key: "cidr_block".into(),
    });
    let CcCreateToken::ReadBack(continuation) = read_back_continuation_token("ec2.Vpc", VPC_ID)
    else {
        panic!("read token")
    };
    let OperationStep::Pending { pending, .. } = provider
        .poll_update_step(
            &id,
            &vpc_identifier(),
            &from,
            &patch,
            CcUpdateToken::ReadBack(continuation.into()),
            PollRequest {
                finality: PollFinality::Continue,
            },
        )
        .await
        .unwrap()
    else {
        panic!("pending")
    };
    let OperationStep::Done(outcome) = provider
        .poll_update_step(
            &id,
            &vpc_identifier(),
            &from,
            &patch,
            pending,
            PollRequest {
                finality: PollFinality::FinalAttempt,
            },
        )
        .await
        .unwrap()
    else {
        panic!("done")
    };
    let UpdateOutcomeKind::PartialSuccess { state, diagnostic } = outcome.kind() else {
        panic!("partial")
    };
    assert_eq!(
        state.attributes(),
        &HashMap::from([("enable_dns_support".into(), bool_(true).to_snapshot())])
    );
    assert!(
        diagnostic
            .missing_attributes()
            .contains(&"enable_dns_support".into())
    );
    assert!(diagnostic.missing_attributes().iter().all(|name| {
        name.as_str()
            .bytes()
            .all(|ch| ch.is_ascii_lowercase() || ch == b'_' || ch.is_ascii_digit())
    }));
}

#[tokio::test]
async fn saved_identifier_only_token_resumes_and_rederives_missing() {
    let token: CcCreateToken = serde_json::from_value(
        json!({"phase":"read_back", "identifier":"review-role", "resource_type":"iam.Role"}),
    )
    .expect("identifier-only saved tokens must resume");
    let (provider, service) = provider_with_script([(
        "GetResource",
        vec![get_resource_reply(
            "AWS::IAM::Role",
            "review-role",
            json!({"RoleId":"AROAREVIEW000000000001"}),
        )],
    )])
    .await;
    let resource = Resource::with_provider("awscc", "iam.Role", "review", None);
    let step = provider
        .resume_create_step(
            &resource,
            CcCreateResume::<TestCreateAttempt, TestIssuedCreate>::Issued(
                TestIssuedCreate {
                    identifier: common::applied_identifier("review-role"),
                },
                token,
            ),
            PollRequest {
                finality: PollFinality::FinalAttempt,
            },
        )
        .await
        .unwrap();
    let TestCreateStep::Partial {
        missing_attributes, ..
    } = step
    else {
        panic!("re-derived arn declaration")
    };
    assert_eq!(missing_attributes, ["arn"]);
    assert_eq!(service.actions(), ["GetResource"]);
    let token: CcUpdateToken = serde_json::from_value(json!({"phase":"read_back", "continuation":{"identifier":"review-role", "resource_type":"iam.Role"}})).expect("identifier-only update token must resume");
    let from: State =
        State::existing(resource.id.clone(), HashMap::new()).with_identifier("review-role");
    let OperationStep::Done(outcome) = provider
        .poll_update_step(
            &resource.id,
            &common::applied_identifier("review-role"),
            &from,
            &UpdatePatch { ops: vec![] },
            token,
            PollRequest {
                finality: PollFinality::FinalAttempt,
            },
        )
        .await
        .unwrap()
    else {
        panic!("final update must finish")
    };
    let UpdateOutcomeKind::PartialSuccess { diagnostic, .. } = outcome.kind() else {
        panic!("re-derived update arn declaration")
    };
    assert_eq!(diagnostic.missing_attributes(), ["arn"]);
    assert_eq!(service.actions(), ["GetResource", "GetResource"]);
}

async fn update_missing_role_attribute() -> UpdateOutcome {
    let role = "review-role";
    let (provider, service) = provider_with_script([
        (
            "UpdateResource",
            vec![mutation_reply("review-update", role)],
        ),
        (
            "GetResourceRequestStatus",
            vec![status_reply(
                "FAILED",
                Some(role),
                Some("handler rejected description"),
            )],
        ),
        (
            "GetResource",
            vec![get_resource_reply(
                "AWS::IAM::Role",
                role,
                json!({"RoleId":"AROAREVIEW000000000001"}),
            )],
        ),
    ])
    .await;
    let id = ResourceId::with_provider_identity("awscc", "iam.Role", "review", None);
    let identifier = common::applied_identifier(role);
    let from: State = State::existing(id.clone(), HashMap::new()).with_identifier(role);
    let patch = UpdatePatch {
        ops: vec![PatchOp::Add {
            key: "description".into(),
            value: string("new"),
        }],
    };
    let (operation, _) = expect_update_pending(
        provider
            .update_step(&id, &identifier, &from, &patch)
            .await
            .unwrap(),
    );
    let OperationStep::Pending {
        pending,
        phase: PendingPhase::ReadBack,
        ..
    } = provider
        .poll_update_step(
            &id,
            &identifier,
            &from,
            &patch,
            operation,
            PollRequest {
                finality: PollFinality::Continue,
            },
        )
        .await
        .unwrap()
    else {
        panic!("declared arn missing must wait")
    };
    let pending = serde_json::from_value(serde_json::to_value(&pending).unwrap()).unwrap();
    let OperationStep::Pending { pending, .. } = provider
        .poll_update_step(
            &id,
            &identifier,
            &from,
            &patch,
            pending,
            PollRequest {
                finality: PollFinality::Continue,
            },
        )
        .await
        .unwrap()
    else {
        panic!("still pending")
    };
    let pending = serde_json::from_value(serde_json::to_value(&pending).unwrap()).unwrap();
    let OperationStep::Done(outcome) = provider
        .poll_update_step(
            &id,
            &identifier,
            &from,
            &patch,
            pending,
            PollRequest {
                finality: PollFinality::FinalAttempt,
            },
        )
        .await
        .unwrap()
    else {
        panic!("final partial")
    };
    assert_eq!(service.calls("UpdateResource"), 1);
    assert_eq!(service.calls("GetResourceRequestStatus"), 1);
    assert_eq!(service.calls("GetResource"), 3);
    outcome
}

#[tokio::test]
async fn update_retains_handler_failure_across_polls() {
    let outcome = update_missing_role_attribute().await;
    let UpdateOutcomeKind::PartialSuccess { diagnostic, .. } = outcome.kind() else {
        panic!("partial")
    };
    assert!(
        diagnostic.reason().contains("handler rejected description"),
        "{}",
        diagnostic.reason()
    );
}

#[tokio::test]
async fn update_checks_patch_confirmation_when_declared_attribute_missing_at_deadline() {
    let outcome = update_missing_role_attribute().await;
    let UpdateOutcomeKind::PartialSuccess { diagnostic, .. } = outcome.kind() else {
        panic!("partial")
    };
    assert_eq!(diagnostic.missing_attributes(), ["arn", "description"]);
    assert!(diagnostic.reason().contains("description"));
}

#[test]
fn native_read_back_poll_transient_error_preserves_continuation() {
    let resource = vpc_resource();
    let token = read_back_token("ec2.Vpc", VPC_ID, &["vpc_id"]);
    let pending = pending_create_for_token(&resource, &token);
    let sent = pending.sent().clone();
    let Poll::Issued(issued) = pending.into_poll() else {
        panic!("read-back must have an issued identifier")
    };
    let failure = CcIssuedCreate::preserve(
        issued,
        token.clone(),
        ProviderError::timeout("temporary polling failure"),
    );
    let CreateFailureKind::InFlight { pending, error } = failure.into_kind() else {
        panic!("a transient read-back poll failure must keep the continuation")
    };
    let pending = pending.into_fresh().unwrap();
    assert_eq!(pending.phase(), PendingPhase::ReadBack);
    assert_eq!(pending.identifier(), Some(&vpc_identifier()));
    assert_eq!(pending.sent(), &sent);
    assert_eq!(pending.poll_after_secs(), 5);
    assert_eq!(
        pending.token().expose_for_provider_call(),
        serde_json::to_string(&token).unwrap()
    );
    assert_eq!(error.variant_name(), "timeout");
    assert_eq!(error.to_string(), "temporary polling failure");
}

#[test]
fn phased_tokens_keep_existing_json_and_derive_pending_phase() {
    fn check<T>(encoded: JsonValue, phase: PendingPhase)
    where
        T: PhasedToken
            + serde::Serialize
            + serde::de::DeserializeOwned
            + Clone
            + PartialEq
            + std::fmt::Debug,
    {
        let token: T = serde_json::from_value(encoded.clone()).unwrap();
        assert_eq!(serde_json::to_value(&token).unwrap(), encoded);
        assert_eq!(token.phase(), phase);
        assert_eq!(
            token.operation().is_some(),
            phase == PendingPhase::Operation
        );
        match token.clone().into_operation() {
            Ok(operation) => assert_eq!(T::from_operation(operation), token),
            Err(read_back) => {
                assert_eq!(phase, PendingPhase::ReadBack);
                assert_eq!(read_back, token);
            }
        }
        let pending = carina_plugin_sdk::Pending::new(token, 17);
        assert_eq!(pending.phase(), phase);
        assert_eq!(pending.poll_after_secs, 17);
        assert_eq!(serde_json::to_value(pending.token).unwrap(), encoded);
    }
    for encoded in [
        json!({"phase":"not_issued", "attempt":2, "client_token":TEST_CLIENT_TOKEN, "poll_after_secs":7}),
        json!({"phase":"unconfirmed", "attempt":2, "resends":3, "client_token":TEST_CLIENT_TOKEN, "poll_after_secs":11}),
        json!({"phase":"in_flight", "request":"accepted-create", "attempt":2, "poll_after_secs":13}),
        json!({"phase":"in_flight", "request":"accepted-create", "attempt":2, "identifier":VPC_ID, "poll_after_secs":13}),
    ] {
        check::<CcCreateToken>(encoded, PendingPhase::Operation);
    }
    check::<CcCreateToken>(
        json!({"phase":"read_back", "identifier":VPC_ID, "resource_type":"ec2.Vpc"}),
        PendingPhase::ReadBack,
    );
    for encoded in [
        json!({"phase":"not_issued", "attempt":2, "client_token":TEST_CLIENT_TOKEN}),
        json!({"phase":"unconfirmed", "attempt":2, "resends":3, "client_token":TEST_CLIENT_TOKEN}),
        json!({"phase":"in_flight", "request":"accepted-update", "attempt":2}),
    ] {
        check::<CcUpdateToken>(encoded, PendingPhase::Operation);
    }
    check::<CcUpdateToken>(
        json!({"phase":"read_back", "continuation":{"identifier":VPC_ID, "resource_type":"ec2.Vpc"}}),
        PendingPhase::ReadBack,
    );
}
