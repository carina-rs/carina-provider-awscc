//! Integration tests for service resend guards using the real SDK loop.

#[cfg(test)]
use carina_core::test_utils::provider_text::BindProviderText as _;
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

use aws_credential_types::Credentials;
use aws_credential_types::provider::SharedCredentialsProvider;
use aws_smithy_runtime_api::client::http::{
    HttpConnector, HttpConnectorFuture, SharedHttpConnector, http_client_fn,
};
use aws_smithy_runtime_api::client::interceptors::Intercept;
use aws_smithy_runtime_api::client::orchestrator::{HttpRequest, HttpResponse};
use aws_smithy_runtime_api::client::result::ConnectorError;
use aws_smithy_types::body::SdkBody;
use aws_smithy_types::error::display::DisplayErrorContext;
use aws_smithy_types::retry::RetryConfig;
use aws_smithy_types::timeout::TimeoutConfig;
use aws_types::region::Region;
use carina_aws_types::sdk::cloudcontrol::{
    CloudControlClient,
    error::SdkError,
    operation::create_resource::{CreateResourceError, CreateResourceOutput},
};
use carina_aws_types::{FinalAttempt, SendClassification};
use carina_plugin_sdk::wasi_http::{SendPhase, transport_connector_error_for_test};

use crate::provider::{AwsccProviderConfig, NotIssuedCreate};
use crate::sdk_config::assert_bounded_policy;
use crate::{AwsccProvider, BoundedSdkConfig};

const VALIDATION_ERROR: &str =
    r#"{"__type":"ValidationException","message":"desired state is invalid"}"#;
const THROTTLING_ERROR: &str = r#"{"__type":"ThrottlingException","message":"rate exceeded"}"#;
const CREATE_OK: &str = r#"{"ProgressEvent":{"RequestToken":"request-token"}}"#;
const GET_RESOURCE_OK: &str = r#"{
    "TypeName":"AWS::S3::Bucket",
    "ResourceDescription":{"Identifier":"partial-bucket","Properties":"{}"}
}"#;

#[derive(Debug)]
enum Action {
    Timeout,
    TransientNotSent,
    Transport(SendPhase),
    LegacyOther,
    Response { status: u16, body: &'static str },
}

#[derive(Debug, Default)]
struct ScriptState {
    actions: VecDeque<Action>,
    requests: usize,
}

#[derive(Clone, Debug)]
struct ScriptedConnector(Arc<Mutex<ScriptState>>);

impl ScriptedConnector {
    fn new(actions: impl IntoIterator<Item = Action>) -> Self {
        Self(Arc::new(Mutex::new(ScriptState {
            actions: actions.into_iter().collect(),
            requests: 0,
        })))
    }

    fn requests(&self) -> usize {
        self.0.lock().expect("script lock").requests
    }
}

impl HttpConnector for ScriptedConnector {
    fn call(&self, _request: HttpRequest) -> HttpConnectorFuture {
        let action = {
            let mut state = self.0.lock().expect("script lock");
            state.requests += 1;
            state.actions.pop_front().expect("script exhausted")
        };

        HttpConnectorFuture::new(async move {
            match action {
                Action::Timeout => Err(ConnectorError::timeout(
                    std::io::Error::other("post-send timeout").into(),
                )),
                // Retain the transport's pre-send witness inside a typed
                // connector timeout, which is a transient NotSent failure.
                Action::TransientNotSent => Err(ConnectorError::timeout(
                    transport_connector_error_for_test(SendPhase::NotSent).into_source(),
                )),
                Action::Transport(phase) => Err(transport_connector_error_for_test(phase)),
                Action::LegacyOther => Err(ConnectorError::other(
                    std::io::Error::other("HTTP error: ErrorCode::ConnectionTimeout").into(),
                    None,
                )),
                Action::Response { status, body } => {
                    let mut response = HttpResponse::new(
                        status.try_into().expect("valid test status"),
                        SdkBody::from(body),
                    );
                    response
                        .headers_mut()
                        .insert("content-type", "application/x-amz-json-1.0");
                    Ok(response)
                }
            }
        })
    }
}

fn bounded_config(connector: ScriptedConnector) -> BoundedSdkConfig {
    let selected = connector.clone();
    let http_client = http_client_fn(move |_, _| SharedHttpConnector::new(selected.clone()));
    let config = aws_types::SdkConfig::builder()
        .http_client(http_client)
        .credentials_provider(SharedCredentialsProvider::new(Credentials::new(
            "test-access-key",
            "test-secret-key",
            None,
            None,
            "resend-guard-test",
        )))
        .region(Region::new("us-east-1"))
        .timeout_config(TimeoutConfig::disabled())
        .retry_config(RetryConfig::standard().with_max_attempts(10))
        .build();
    BoundedSdkConfig::for_test(config)
}

fn cloudcontrol_client(connector: ScriptedConnector) -> CloudControlClient {
    bounded_config(connector).clients().cloudcontrol
}

fn assert_resend_guard(client: &CloudControlClient) {
    assert!(
        client
            .inner()
            .config()
            .interceptors()
            .any(|interceptor| interceptor.name() == "carina.ResendGuard"),
        "production CloudControl client has no carina.ResendGuard interceptor"
    );
}

#[allow(clippy::result_large_err)]
async fn send_create(
    client: &CloudControlClient,
) -> Result<CreateResourceOutput, SdkError<CreateResourceError>> {
    client
        .inner()
        .create_resource()
        .type_name("AWS::Test::Resource")
        .desired_state("{}")
        .client_token("test-client-token")
        .send()
        .await
}

#[tokio::test(start_paused = true)]
async fn readonly_s3_list_object_versions_retries_after_maybe_sent_transport_error() {
    let connector = ScriptedConnector::new([
        Action::Transport(SendPhase::MaybeSent),
        Action::Response {
            status: 200,
            body: "<ListVersionsResult xmlns=\"http://s3.amazonaws.com/doc/2006-03-01/\"><Name>test-bucket</Name><IsTruncated>false</IsTruncated></ListVersionsResult>",
        },
    ]);
    let client = bounded_config(connector.clone()).clients().s3;

    let output = client
        .inner()
        .list_object_versions()
        .bucket("test-bucket")
        .send()
        .await
        .expect("the read-only guard should promote the transport failure to retryable");

    assert_eq!(output.name(), Some("test-bucket"));
    assert_eq!(connector.requests(), 2);
}

#[tokio::test(start_paused = true)]
async fn s3_cleanup_rejects_entries_without_version_id_before_deleting() {
    for body in [
        r#"<ListVersionsResult xmlns="http://s3.amazonaws.com/doc/2006-03-01/">
            <Name>test-bucket</Name>
            <IsTruncated>false</IsTruncated>
            <Version><Key>valid-key</Key><VersionId>test-version</VersionId></Version>
            <Version><Key>missing-version-id-key</Key></Version>
        </ListVersionsResult>"#,
        r#"<ListVersionsResult xmlns="http://s3.amazonaws.com/doc/2006-03-01/">
            <Name>test-bucket</Name>
            <IsTruncated>false</IsTruncated>
            <Version><Key>valid-key</Key><VersionId>test-version</VersionId></Version>
            <DeleteMarker><Key>missing-version-id-key</Key></DeleteMarker>
        </ListVersionsResult>"#,
    ] {
        let connector = ScriptedConnector::new([
            Action::Response { status: 200, body },
            Action::Response {
                status: 200,
                body: "<DeleteResult xmlns=\"http://s3.amazonaws.com/doc/2006-03-01/\"/>",
            },
        ]);
        let provider = AwsccProvider::from_sdk_config(
            bounded_config(connector.clone()),
            &AwsccProviderConfig::default(),
        )
        .await;

        let result = provider.empty_s3_bucket("test-bucket").await;

        assert_eq!(
            connector.requests(),
            1,
            "cleanup must send only ListObjectVersions and no DeleteObjects"
        );
        let error = result.expect_err("cleanup must reject an entry without VersionId");
        assert_eq!(
            error
                .clone()
                .into_host_for_test()
                .detail()
                .context()
                .to_string(),
            "ListObjectVersions returned no VersionId for key \"missing-version-id-key\""
        );
    }
}

#[tokio::test(start_paused = true)]
async fn idempotent_s3_delete_objects_retries_after_maybe_sent_timeout() {
    let connector = ScriptedConnector::new([
        Action::Timeout,
        Action::Response {
            status: 200,
            body: "<DeleteResult xmlns=\"http://s3.amazonaws.com/doc/2006-03-01/\"/>",
        },
    ]);
    let client = bounded_config(connector.clone()).clients().s3;
    let object = carina_aws_types::sdk::s3::types::ObjectIdentifier::builder()
        .key("test-key")
        .version_id("test-version")
        .build()
        .unwrap();
    let delete = carina_aws_types::sdk::s3::types::Delete::builder()
        .objects(object)
        .build()
        .unwrap();

    client
        .inner()
        .delete_objects()
        .bucket("test-bucket")
        .delete(delete)
        .send()
        .await
        .expect("deleting explicit versions should remain retryable after a timeout");

    assert_eq!(connector.requests(), 2);
}

#[tokio::test(start_paused = true)]
async fn readonly_sts_get_caller_identity_retries_after_maybe_sent_transport_error() {
    let connector = ScriptedConnector::new([
        Action::Transport(SendPhase::MaybeSent),
        Action::Response {
            status: 200,
            body: "<GetCallerIdentityResponse xmlns=\"https://sts.amazonaws.com/doc/2011-06-15/\"><GetCallerIdentityResult><Account>123456789012</Account></GetCallerIdentityResult><ResponseMetadata><RequestId>test</RequestId></ResponseMetadata></GetCallerIdentityResponse>",
        },
    ]);
    let client = bounded_config(connector.clone()).clients().sts;

    let output = client
        .inner()
        .get_caller_identity()
        .send()
        .await
        .expect("the read-only guard should promote the transport failure to retryable");

    assert_eq!(output.account(), Some("123456789012"));
    assert_eq!(connector.requests(), 2);
}

#[tokio::test]
async fn permanent_create_rejection_is_rejected_and_fails_not_issued_create() {
    let provider_connector = ScriptedConnector::new([Action::Response {
        status: 400,
        body: VALIDATION_ERROR,
    }]);
    let provider = AwsccProvider::from_sdk_config(
        bounded_config(provider_connector.clone()),
        &AwsccProviderConfig::default(),
    )
    .await;

    let result = provider
        .send_not_issued_create(
            "AWS::Test::Resource",
            serde_json::json!({}),
            None,
            NotIssuedCreate::start(),
        )
        .await;
    let Err(error) = result else {
        panic!("a permanent rejection must fail instead of becoming unconfirmed");
    };
    assert_eq!(
        error
            .into_error()
            .into_host_for_test()
            .detail()
            .code()
            .map(|code| code.to_string())
            .as_deref(),
        Some("ValidationException")
    );
    assert_eq!(provider_connector.requests(), 1);

    let connector = ScriptedConnector::new([Action::Response {
        status: 400,
        body: VALIDATION_ERROR,
    }]);
    let client = cloudcontrol_client(connector.clone());
    let error = send_create(&client)
        .await
        .expect_err("the service should reject invalid desired state");

    assert_eq!(connector.requests(), 1);
    assert_eq!(SendClassification::of(&error), SendClassification::Rejected);
    assert_resend_guard(&client);
}

#[tokio::test(start_paused = true)]
async fn throttling_rejection_is_retried_and_then_succeeds() {
    let connector = ScriptedConnector::new([
        Action::Response {
            status: 429,
            body: THROTTLING_ERROR,
        },
        Action::Response {
            status: 200,
            body: CREATE_OK,
        },
    ]);
    let client = cloudcontrol_client(connector.clone());

    let output = send_create(&client)
        .await
        .expect("the SDK should retry a rejected throttling response");

    assert_eq!(
        output
            .progress_event()
            .and_then(|event| event.request_token()),
        Some("request-token")
    );
    assert_eq!(connector.requests(), 2);
    assert_resend_guard(&client);
}

#[tokio::test(start_paused = true)]
async fn timeout_before_final_rejection_keeps_operation_maybe_sent() {
    let connector = ScriptedConnector::new([
        Action::Timeout,
        Action::Response {
            status: 400,
            body: VALIDATION_ERROR,
        },
    ]);
    let client = cloudcontrol_client(connector.clone());

    let error = send_create(&client)
        .await
        .expect_err("the second attempt should surface the validation error");

    assert_eq!(connector.requests(), 2);
    assert_eq!(
        SendClassification::of(&error),
        SendClassification::MaybeSent {
            final_attempt: FinalAttempt::Sent,
        }
    );
    assert_resend_guard(&client);
}

#[tokio::test(start_paused = true)]
async fn readonly_get_resource_retries_after_maybe_sent_timeout() {
    let connector = ScriptedConnector::new([
        Action::Timeout,
        Action::Response {
            status: 200,
            body: GET_RESOURCE_OK,
        },
    ]);
    let client = cloudcontrol_client(connector.clone());

    let output = client
        .inner()
        .get_resource()
        .type_name("AWS::S3::Bucket")
        .identifier("partial-bucket")
        .send()
        .await
        .expect("the SDK should retry a read-only operation after a timeout");

    assert_eq!(output.type_name(), Some("AWS::S3::Bucket"));
    assert_eq!(
        output
            .resource_description()
            .and_then(|description| description.identifier()),
        Some("partial-bucket")
    );
    assert_eq!(connector.requests(), 2);
    assert_resend_guard(&client);
}

#[tokio::test(start_paused = true)]
async fn readonly_get_resource_retries_after_not_sent_transport_error() {
    let connector = ScriptedConnector::new([
        Action::Transport(SendPhase::NotSent),
        Action::Response {
            status: 200,
            body: GET_RESOURCE_OK,
        },
    ]);
    let client = cloudcontrol_client(connector.clone());

    let output = client
        .inner()
        .get_resource()
        .type_name("AWS::S3::Bucket")
        .identifier("partial-bucket")
        .send()
        .await
        .expect("the SDK should retry a read-only operation after a not-sent transport error");

    assert_eq!(output.type_name(), Some("AWS::S3::Bucket"));
    assert_eq!(connector.requests(), 2);
    assert_resend_guard(&client);
}

#[tokio::test(start_paused = true)]
async fn readonly_get_resource_retries_after_maybe_sent_transport_error() {
    let connector = ScriptedConnector::new([
        Action::Transport(SendPhase::MaybeSent),
        Action::Response {
            status: 200,
            body: GET_RESOURCE_OK,
        },
    ]);
    let client = cloudcontrol_client(connector.clone());

    let output = client
        .inner()
        .get_resource()
        .type_name("AWS::S3::Bucket")
        .identifier("partial-bucket")
        .send()
        .await
        .expect("the read-only guard should make a maybe-sent transport error retryable");

    assert_eq!(output.type_name(), Some("AWS::S3::Bucket"));
    assert_eq!(connector.requests(), 2);
    assert_resend_guard(&client);
}

#[tokio::test(start_paused = true)]
async fn readonly_get_resource_exhausts_maybe_sent_transport_retries_as_maybe_sent() {
    let connector = ScriptedConnector::new([
        Action::Transport(SendPhase::MaybeSent),
        Action::Transport(SendPhase::MaybeSent),
        Action::Transport(SendPhase::MaybeSent),
    ]);
    let client = cloudcontrol_client(connector.clone());

    let error = client
        .inner()
        .get_resource()
        .type_name("AWS::S3::Bucket")
        .identifier("partial-bucket")
        .send()
        .await
        .expect_err("three maybe-sent failures should exhaust the production retry budget");

    assert_eq!(connector.requests(), 3);
    assert_eq!(
        SendClassification::of(&error),
        SendClassification::MaybeSent {
            final_attempt: FinalAttempt::Sent,
        }
    );
    assert_resend_guard(&client);
}

#[tokio::test(start_paused = true)]
async fn create_resource_retries_after_not_sent_transport_error() {
    let connector = ScriptedConnector::new([
        Action::Transport(SendPhase::NotSent),
        Action::Response {
            status: 200,
            body: CREATE_OK,
        },
    ]);
    let client = cloudcontrol_client(connector.clone());

    let output = send_create(&client)
        .await
        .expect("the SDK should retry a token-bearing mutation after a not-sent transport error");

    assert_eq!(
        output
            .progress_event()
            .and_then(|event| event.request_token()),
        Some("request-token")
    );
    assert_eq!(connector.requests(), 2);
    assert_resend_guard(&client);
}

#[tokio::test(start_paused = true)]
async fn create_resource_does_not_retry_after_maybe_sent_transport_error() {
    let connector = ScriptedConnector::new([
        Action::Transport(SendPhase::MaybeSent),
        Action::Response {
            status: 200,
            body: CREATE_OK,
        },
    ]);
    let client = cloudcontrol_client(connector.clone());

    let error = send_create(&client)
        .await
        .expect_err("an untyped other connector error is not classified as retryable");
    let rendered = DisplayErrorContext(&error).to_string();

    assert_eq!(connector.requests(), 1);
    assert_eq!(
        SendClassification::of(&error),
        SendClassification::MaybeSent {
            final_attempt: FinalAttempt::Sent,
        }
    );
    assert!(rendered.contains("dispatch failure: other"), "{rendered}");
    assert_resend_guard(&client);
}

#[tokio::test(start_paused = true)]
async fn exhausted_not_sent_transport_errors_render_as_io() {
    let connector = ScriptedConnector::new([
        Action::Transport(SendPhase::NotSent),
        Action::Transport(SendPhase::NotSent),
        Action::Transport(SendPhase::NotSent),
    ]);
    let provider = AwsccProvider::from_sdk_config(
        bounded_config(connector.clone()),
        &AwsccProviderConfig::default(),
    )
    .await;

    let error = provider
        .cc_get_resource("AWS::S3::Bucket", "partial-bucket")
        .await
        .expect_err("three not-sent failures should exhaust the production retry budget");
    let rendered = error.into_error().into_host_for_test().to_string();

    assert_eq!(connector.requests(), 3);
    assert!(
        rendered.contains("AWS SDK request failed: dispatch failure: io"),
        "{rendered}"
    );
    assert!(!rendered.contains("dispatch failure: other"), "{rendered}");
}

#[tokio::test(start_paused = true)]
async fn readonly_get_resource_retries_legacy_other_transport_error() {
    // This is the exact ConnectorError::other(_, None) shape that caused the
    // transient GetResource failure in the awscc 35a1878 infra run from #518.
    let connector = ScriptedConnector::new([
        Action::LegacyOther,
        Action::Response {
            status: 200,
            body: GET_RESOURCE_OK,
        },
    ]);
    let client = cloudcontrol_client(connector.clone());

    let output = client
        .inner()
        .get_resource()
        .type_name("AWS::S3::Bucket")
        .identifier("partial-bucket")
        .send()
        .await
        .expect("the read-only guard should retry the legacy unclassified transport error");

    assert_eq!(output.type_name(), Some("AWS::S3::Bucket"));
    assert_eq!(connector.requests(), 2);
    assert_resend_guard(&client);
}

#[test]
fn production_cloudcontrol_client_keeps_bounded_policy_and_installs_guard() {
    let connector = ScriptedConnector::new([]);
    let client = cloudcontrol_client(connector);

    assert_bounded_policy(
        client.inner().config().timeout_config(),
        client.inner().config().retry_config(),
    );
    assert_resend_guard(&client);
}

#[tokio::test(start_paused = true)]
async fn transient_not_sent_retries_with_backoff_only_within_not_issued_budget() {
    use crate::CcRequestPhase;
    use carina_provider_protocol::{OperationStep, PendingPhase};

    let connector = ScriptedConnector::new((0..9).map(|_| Action::TransientNotSent));
    let provider = AwsccProvider::from_sdk_config(
        bounded_config(connector.clone()),
        &AwsccProviderConfig::default(),
    )
    .await;
    let config = carina_core::schema::OperationConfig {
        create_max_retries: Some(2),
        ..Default::default()
    };
    let mut token: CcRequestPhase = serde_json::from_value(serde_json::json!({
        "phase":"not_issued", "attempt":0, "client_token":"same-token"
    }))
    .expect("not-issued continuation");
    for attempt in 1..=2 {
        let previous = serde_json::to_value(&token).expect("encode prior token");
        let step = provider
            .cc_poll_update_step(
                "AWS::Test::Resource",
                "physical-id",
                vec![serde_json::json!({"op":"add", "path":"/Name", "value":"test"})],
                Some(&config),
                token,
            )
            .await
            .expect("transient NotSent must consume the bounded not-issued retry budget");
        let OperationStep::Pending {
            pending,
            poll_after_secs,
            phase: PendingPhase::Operation,
        } = step
        else {
            panic!("transient NotSent must back off before retrying");
        };
        let encoded = serde_json::to_value(&pending).expect("encode pending token");
        assert_eq!(encoded["phase"], "not_issued");
        assert_eq!(encoded["attempt"], attempt);
        assert_eq!(encoded["client_token"], previous["client_token"]);
        assert_eq!(poll_after_secs, 10 * 2_u32.pow(attempt - 1));
        token = pending;
    }
    provider
        .cc_poll_update_step(
            "AWS::Test::Resource",
            "physical-id",
            vec![serde_json::json!({"op":"add", "path":"/Name", "value":"test"})],
            Some(&config),
            token,
        )
        .await
        .expect_err("transient NotSent must stop when the not-issued budget is exhausted");
    assert_eq!(connector.requests(), 9);
}

#[tokio::test(start_paused = true)]
async fn unconfirmed_transient_not_sent_keeps_same_token_and_capped_backoff() {
    use crate::CcRequestPhase;
    use carina_provider_protocol::{OperationStep, PendingPhase};

    let connector = ScriptedConnector::new((0..39).map(|_| Action::TransientNotSent));
    let provider = AwsccProvider::from_sdk_config(
        bounded_config(connector.clone()),
        &AwsccProviderConfig::default(),
    )
    .await;
    let mut token: CcRequestPhase = serde_json::from_value(serde_json::json!({
        "phase":"unconfirmed", "attempt":0, "resends":1, "client_token":"same-token"
    }))
    .expect("unconfirmed continuation");
    for resends in 2..=14 {
        let step = provider
            .cc_poll_update_step(
                "AWS::Test::Resource",
                "physical-id",
                vec![serde_json::json!({"op":"add", "path":"/Name", "value":"test"})],
                None,
                token,
            )
            .await
            .expect("a transient pre-send failure must keep the earlier send unconfirmed");
        let OperationStep::Pending {
            pending,
            poll_after_secs,
            phase: PendingPhase::Operation,
        } = step
        else {
            panic!("an earlier send may still be accepted");
        };
        let encoded = serde_json::to_value(&pending).expect("encode continuation");
        assert_eq!(encoded["phase"], "unconfirmed");
        assert_eq!(encoded["attempt"], 0);
        assert_eq!(encoded["resends"], resends);
        assert_eq!(encoded["client_token"], "same-token");
        assert_eq!(poll_after_secs, (10 * 2_u32.pow(resends - 1)).min(120));
        token = pending;
    }
    assert_eq!(connector.requests(), 39);
}
