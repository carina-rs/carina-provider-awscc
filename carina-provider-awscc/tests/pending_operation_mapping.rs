//! Exact state-machine coverage for host-driven CloudControl operations.

#![cfg(not(target_arch = "wasm32"))]

mod common;

use std::collections::{HashMap, VecDeque};
use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Mutex};

use aws_config::{BehaviorVersion, Region};
use aws_sdk_cloudcontrol::operation::create_resource::CreateResourceError;
use carina_core::provider::{
    CreateOutcome, DeleteRequest, PatchOp, PatchOpKind, UpdateOutcome, UpdatePatch,
};
use carina_core::resource::{ConcreteValue, Resource, ResourceId, State, Value};
use carina_provider_awscc::provider::AwsccProviderConfig;
use carina_provider_awscc::{
    AwsccProvider, CcCreateToken, CcDeleteToken, CcRequestPhase, CcUpdateToken, RequestToken,
};
use carina_provider_protocol::OperationStep;
use serde_json::{Value as JsonValue, json};
use winterbaume_core::{MockAws, MockRequest, MockResponse, MockService, json_error_response};

const REGION: &str = "us-east-1";
const VPC_TYPE: &str = "AWS::EC2::VPC";
const VPC_ID: &str = "vpc-0123456789abcdef0";
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

fn mutation_reply(token: &str, identifier: &str) -> Reply {
    Reply::Json(json!({
        "ProgressEvent": {
            "Identifier": identifier,
            "OperationStatus": "IN_PROGRESS",
            "RequestToken": token,
        }
    }))
}

fn status_reply(status: &str, identifier: Option<&str>, message: Option<&str>) -> Reply {
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
        ops: vec![PatchOp {
            kind: PatchOpKind::Replace,
            key: "enable_dns_support".to_string(),
            value: Some(bool_(true)),
        }],
    };
    (id, from, patch)
}

async fn provider_with_script(
    entries: impl IntoIterator<Item = (&'static str, Vec<Reply>)>,
) -> (AwsccProvider, Arc<ScriptedCloudControl>) {
    let service = Arc::new(ScriptedCloudControl::new(entries));
    let mock = MockAws::builder()
        .with_service(Arc::clone(&service))
        .build();
    let config = aws_config::defaults(BehaviorVersion::latest())
        .http_client(mock.http_client())
        .credentials_provider(mock.credentials_provider())
        .region(Region::new(REGION))
        .retry_config(aws_config::retry::RetryConfig::disabled())
        .load()
        .await;
    (
        AwsccProvider::from_sdk_config(config, &AwsccProviderConfig::default()).await,
        service,
    )
}

fn expect_create_pending(
    step: OperationStep<CreateOutcome, CcCreateToken>,
) -> (CcCreateToken, u32) {
    match step {
        OperationStep::Pending {
            pending,
            poll_after_secs,
        } => (pending, poll_after_secs),
        OperationStep::Done(outcome) => panic!("expected pending create, got {outcome:?}"),
    }
}

fn expect_delete_pending(step: OperationStep<(), CcDeleteToken>) -> (CcDeleteToken, u32) {
    match step {
        OperationStep::Pending {
            pending,
            poll_after_secs,
        } => (pending, poll_after_secs),
        OperationStep::Done(()) => panic!("expected pending delete"),
    }
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
            .create_step(resource.as_resource())
            .await
            .expect("issue create"),
    );
    assert_eq!(delay, 1);
    let expected = CcCreateToken::InFlight {
        request: RequestToken::from("create-0"),
        attempt: 0,
    };
    assert_eq!(token, expected);

    for expected_status in ["PENDING", "IN_PROGRESS"] {
        let (next, delay) = expect_create_pending(
            provider
                .poll_create_step(resource.as_resource(), token)
                .await
                .unwrap_or_else(|error| panic!("poll {expected_status}: {error}")),
        );
        assert_eq!(next, expected);
        assert_eq!(delay, 5);
        token = next;
    }

    let state = match provider
        .poll_create_step(resource.as_resource(), token)
        .await
        .expect("poll successful create")
    {
        OperationStep::Done(CreateOutcome::Success { state }) => state,
        other => panic!("expected successful create, got {other:?}"),
    };
    assert_eq!(state.identifier().as_str(), VPC_ID);
    assert_eq!(service.calls("CreateResource"), 1);
    assert_eq!(service.calls("GetResourceRequestStatus"), 3);
    assert_eq!(service.calls("GetResource"), 1);
}

#[tokio::test]
async fn retryable_initial_sdk_error_returns_not_issued_and_poll_resends_same_request() {
    let (provider, service) = provider_with_script([(
        "CreateResource",
        vec![throttled(), mutation_reply("create-1", VPC_ID)],
    )])
    .await;
    let resource = common::normalize_resource(vpc_resource()).await;

    let (token, delay) = expect_create_pending(
        provider
            .create_step(resource.as_resource())
            .await
            .expect("retryable initial create"),
    );
    assert_eq!(token, CcCreateToken::NotIssued { attempt: 1 });
    assert_eq!(delay, 10);
    assert_eq!(service.calls("CreateResource"), 1);

    let (token, delay) = expect_create_pending(
        provider
            .poll_create_step(resource.as_resource(), token)
            .await
            .expect("re-send create"),
    );
    assert_eq!(
        token,
        CcCreateToken::InFlight {
            request: RequestToken::from("create-1"),
            attempt: 1,
        }
    );
    assert_eq!(delay, 1);
    assert_eq!(service.calls("CreateResource"), 2);

    let bodies = service.bodies("CreateResource");
    let desired_states = bodies
        .iter()
        .map(|body| {
            serde_json::from_str::<JsonValue>(body).unwrap()["DesiredState"]
                .as_str()
                .unwrap()
                .to_string()
        })
        .collect::<Vec<_>>();
    assert_eq!(desired_states.len(), 2);
    assert_eq!(desired_states[0], desired_states[1]);
}

#[tokio::test]
async fn retryable_failed_status_reissues_with_incremented_attempt_and_backoff() {
    let (provider, service) = provider_with_script([
        (
            "CreateResource",
            vec![
                mutation_reply("create-0", VPC_ID),
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
            .create_step(resource.as_resource())
            .await
            .expect("issue create"),
    );

    let (token, delay) = expect_create_pending(
        provider
            .poll_create_step(resource.as_resource(), token)
            .await
            .expect("retry failed create"),
    );
    assert_eq!(token, CcCreateToken::NotIssued { attempt: 1 });
    assert_eq!(delay, 10);
    assert_eq!(service.calls("GetResourceRequestStatus"), 1);
    assert_eq!(service.calls("CreateResource"), 1);

    let (token, delay) = expect_create_pending(
        provider
            .poll_create_step(resource.as_resource(), token)
            .await
            .expect("re-issue failed create after backoff"),
    );
    assert_eq!(
        token,
        CcCreateToken::InFlight {
            request: RequestToken::from("create-1"),
            attempt: 1,
        }
    );
    assert_eq!(delay, 1);
    assert_eq!(service.calls("CreateResource"), 2);
}

#[tokio::test]
async fn retryable_failed_create_with_identifier_uses_partial_path_without_reissue() {
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
            .create_step(resource.as_resource())
            .await
            .expect("issue create"),
    );

    match provider
        .poll_create_step(resource.as_resource(), token)
        .await
        .expect("identifier-bearing failure should settle through read-back")
    {
        OperationStep::Done(CreateOutcome::PartialSuccess { diagnostic, .. }) => {
            assert!(diagnostic.reason().contains("Throttling"));
        }
        other => panic!("expected partial create, got {other:?}"),
    }
    assert_eq!(service.calls("CreateResource"), 1);
    assert_eq!(service.calls("GetResourceRequestStatus"), 1);
}

#[tokio::test]
async fn retryable_cancel_complete_without_identifier_schedules_reissue() {
    let (provider, service) = provider_with_script([
        ("CreateResource", vec![mutation_reply("create-0", VPC_ID)]),
        (
            "GetResourceRequestStatus",
            vec![status_reply("CANCEL_COMPLETE", None, Some("Throttling"))],
        ),
    ])
    .await;
    let resource = common::normalize_resource(vpc_resource()).await;
    let (token, _) = expect_create_pending(
        provider
            .create_step(resource.as_resource())
            .await
            .expect("issue create"),
    );

    let (token, delay) = expect_create_pending(
        provider
            .poll_create_step(resource.as_resource(), token)
            .await
            .expect("retry canceled create"),
    );
    assert_eq!(token, CcCreateToken::NotIssued { attempt: 1 });
    assert_eq!(delay, 10);
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
    assert_eq!(
        token,
        CcDeleteToken::Delete(CcRequestPhase::NotIssued { attempt: 1 })
    );
    assert_eq!(delay, 10);
    assert_eq!(service.calls("DeleteResource"), 1);
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
        assert_eq!(
            not_issued,
            CcDeleteToken::Delete(CcRequestPhase::NotIssued { attempt })
        );

        let (in_flight, delay) = expect_delete_pending(
            provider
                .poll_delete_step(&id, &transit_gateway_identifier(), &request, not_issued)
                .await
                .expect("re-issue TransitGateway delete"),
        );
        assert_eq!(
            in_flight,
            CcDeleteToken::Delete(CcRequestPhase::InFlight {
                request: RequestToken::from("delete-tgw"),
                attempt,
            })
        );
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
async fn non_retryable_failed_status_preserves_error_and_partial_outcomes() {
    let (provider, _) = provider_with_script([
        ("CreateResource", vec![mutation_reply("create-0", VPC_ID)]),
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
            .create_step(resource.as_resource())
            .await
            .expect("issue create"),
    );
    let error = provider
        .poll_create_step(resource.as_resource(), token)
        .await
        .expect_err("failed create without identifier must error");
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
            .create_step(resource.as_resource())
            .await
            .expect("issue partial create"),
    );
    match provider
        .poll_create_step(resource.as_resource(), token)
        .await
        .expect("partial create")
    {
        OperationStep::Done(CreateOutcome::PartialSuccess { diagnostic, .. }) => {
            assert!(diagnostic.reason().contains("post-create read denied"));
        }
        other => panic!("expected partial create, got {other:?}"),
    }
}

#[tokio::test]
async fn retry_attempts_exhausted_preserves_the_last_sdk_error_source() {
    let (provider, service) = provider_with_script([("CreateResource", vec![throttled()])]).await;
    let resource = common::normalize_resource(vpc_resource()).await;
    let (mut token, delay) = expect_create_pending(
        provider
            .create_step(resource.as_resource())
            .await
            .expect("first retryable create"),
    );
    assert_eq!(delay, 10);

    let expected_delays = [20, 40, 80, 120, 120, 120, 120, 120, 120, 120, 120];
    for (attempt, expected_delay) in (1..12).zip(expected_delays) {
        let (next, delay) = expect_create_pending(
            provider
                .poll_create_step(resource.as_resource(), token)
                .await
                .unwrap_or_else(|error| panic!("retry {attempt} failed early: {error}")),
        );
        assert_eq!(
            next,
            CcCreateToken::NotIssued {
                attempt: attempt + 1
            }
        );
        assert_eq!(delay, expected_delay);
        token = next;
    }

    let error = provider
        .poll_create_step(resource.as_resource(), token)
        .await
        .expect_err("the final configured retry must be exhausted");
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
    assert_eq!(service.calls("CreateResource"), 13);
}

#[tokio::test]
async fn status_retry_exhaustion_preserves_the_last_operation_failure() {
    let (provider, service) = provider_with_script([(
        "GetResourceRequestStatus",
        vec![status_reply("FAILED", None, Some("Throttling"))],
    )])
    .await;
    let resource = common::normalize_resource(vpc_resource()).await;

    let error = provider
        .poll_create_step(
            resource.as_resource(),
            CcCreateToken::InFlight {
                request: RequestToken::from("create-final"),
                attempt: 12,
            },
        )
        .await
        .expect_err("final retryable status should return its real failure");

    assert_eq!(error.detail().context(), "Operation failed");
    assert!(error.detail().cause().is_some());
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
    assert_eq!(
        token,
        CcUpdateToken::InFlight {
            request: RequestToken::from("update-0"),
            attempt: 0,
        }
    );
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

#[tokio::test]
async fn update_not_issued_resends_the_original_patch() {
    let (provider, service) = provider_with_script([(
        "UpdateResource",
        vec![throttled(), mutation_reply("update-1", VPC_ID)],
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
    assert_eq!(token, CcUpdateToken::NotIssued { attempt: 1 });
    assert_eq!(delay, 10);
    assert_eq!(service.calls("UpdateResource"), 1);

    let (token, delay) = match provider
        .poll_update_step(&id, &vpc_identifier(), &from, &patch, token)
        .await
        .expect("re-send update")
    {
        OperationStep::Pending {
            pending,
            poll_after_secs,
        } => (pending, poll_after_secs),
        other => panic!("expected pending update, got {other:?}"),
    };
    assert_eq!(
        token,
        CcUpdateToken::InFlight {
            request: RequestToken::from("update-1"),
            attempt: 1,
        }
    );
    assert_eq!(delay, 1);
    assert_eq!(service.calls("UpdateResource"), 2);
    let bodies = service.bodies("UpdateResource");
    assert_eq!(bodies.len(), 2);
    let patches = bodies
        .iter()
        .map(|body| serde_json::from_str::<JsonValue>(body).unwrap()["PatchDocument"].clone())
        .collect::<Vec<_>>();
    assert_eq!(patches[0], patches[1]);
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
    assert_eq!(
        token,
        CcDeleteToken::DetachIgw(CcRequestPhase::InFlight {
            request: RequestToken::from("detach-0"),
            attempt: 0,
        })
    );
    assert_eq!(service.calls("UpdateResource"), 1);
    assert_eq!(service.calls("DeleteResource"), 0);

    let (token, delay) = expect_delete_pending(
        provider
            .poll_delete_step(&id, &internet_gateway_identifier(), &request, token)
            .await
            .expect("finish detach and issue delete"),
    );
    assert_eq!(delay, 1);
    assert_eq!(
        token,
        CcDeleteToken::Delete(CcRequestPhase::InFlight {
            request: RequestToken::from("delete-0"),
            attempt: 0,
        })
    );
    assert_eq!(service.calls("DeleteResource"), 1);

    assert_eq!(
        provider
            .poll_delete_step(&id, &internet_gateway_identifier(), &request, token)
            .await
            .expect("finish delete"),
        OperationStep::Done(())
    );
    assert_eq!(service.calls("GetResourceRequestStatus"), 2);
}

#[tokio::test]
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
            vec![throttled(), mutation_reply("detach-1", IGW_ID)],
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
    assert_eq!(
        token,
        CcDeleteToken::DetachIgw(CcRequestPhase::NotIssued { attempt: 1 })
    );
    assert_eq!(delay, 10);
    assert_eq!(service.calls("UpdateResource"), 1);

    let (token, delay) = expect_delete_pending(
        provider
            .poll_delete_step(&id, &internet_gateway_identifier(), &request, token)
            .await
            .expect("re-send IGW detach"),
    );
    assert_eq!(
        token,
        CcDeleteToken::DetachIgw(CcRequestPhase::InFlight {
            request: RequestToken::from("detach-1"),
            attempt: 1,
        })
    );
    assert_eq!(delay, 1);
    assert_eq!(service.calls("GetResource"), 2);
    assert_eq!(service.calls("UpdateResource"), 2);
    assert_eq!(service.calls("GetResourceRequestStatus"), 0);
    assert_eq!(service.calls("DeleteResource"), 0);

    let (token, delay) = expect_delete_pending(
        provider
            .poll_delete_step(&id, &internet_gateway_identifier(), &request, token)
            .await
            .expect("finish retried detach and start delete"),
    );
    assert_eq!(
        token,
        CcDeleteToken::Delete(CcRequestPhase::InFlight {
            request: RequestToken::from("delete-0"),
            attempt: 0,
        })
    );
    assert_eq!(delay, 1);
    assert_eq!(service.calls("GetResourceRequestStatus"), 1);
    assert_eq!(service.calls("DeleteResource"), 1);
}

#[tokio::test]
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
    assert_eq!(
        token,
        CcDeleteToken::DetachIgw(CcRequestPhase::NotIssued { attempt: 1 })
    );

    let (token, delay) = expect_delete_pending(
        provider
            .poll_delete_step(&id, &internet_gateway_identifier(), &request, token)
            .await
            .expect("skip redundant detach and issue delete"),
    );
    assert_eq!(
        token,
        CcDeleteToken::Delete(CcRequestPhase::InFlight {
            request: RequestToken::from("delete-0"),
            attempt: 0,
        })
    );
    assert_eq!(delay, 1);
    assert_eq!(service.calls("GetResource"), 2);
    assert_eq!(service.calls("UpdateResource"), 1);
    assert_eq!(service.calls("GetResourceRequestStatus"), 0);
    assert_eq!(service.calls("DeleteResource"), 1);
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
    assert_eq!(
        token,
        CcDeleteToken::Delete(CcRequestPhase::InFlight {
            request: RequestToken::from("delete-0"),
            attempt: 0,
        })
    );
    assert_eq!(delay, 1);
    assert_eq!(service.calls("UpdateResource"), 0);
    assert_eq!(service.calls("DeleteResource"), 1);
}
