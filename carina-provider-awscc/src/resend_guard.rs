//! Integration tests for the CloudControl resend guard using the real SDK loop.

use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

use aws_config::Region;
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
use carina_aws_types::SendOutcome;
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
    let config = aws_config::SdkConfig::builder()
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
    BoundedSdkConfig::new(config)
}

fn cloudcontrol_client(connector: ScriptedConnector) -> aws_sdk_cloudcontrol::Client {
    bounded_config(connector).clients().cloudcontrol
}

fn assert_resend_guard(client: &aws_sdk_cloudcontrol::Client) {
    assert!(
        client
            .config()
            .interceptors()
            .any(|interceptor| interceptor.name() == "carina.ResendGuard"),
        "production CloudControl client has no carina.ResendGuard interceptor"
    );
}

async fn send_create(
    client: &aws_sdk_cloudcontrol::Client,
) -> Result<
    aws_sdk_cloudcontrol::operation::create_resource::CreateResourceOutput,
    aws_sdk_cloudcontrol::error::SdkError<
        aws_sdk_cloudcontrol::operation::create_resource::CreateResourceError,
    >,
> {
    client
        .create_resource()
        .type_name("AWS::Test::Resource")
        .desired_state("{}")
        .client_token("test-client-token")
        .send()
        .await
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
        error.into_error().detail().code.as_deref(),
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
    assert_eq!(SendOutcome::classify(&error), SendOutcome::Rejected);
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
    assert_eq!(SendOutcome::classify(&error), SendOutcome::MaybeSent);
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
        .get_resource()
        .type_name("AWS::S3::Bucket")
        .identifier("partial-bucket")
        .send()
        .await
        .expect_err("three maybe-sent failures should exhaust the production retry budget");

    assert_eq!(connector.requests(), 3);
    assert_eq!(SendOutcome::classify(&error), SendOutcome::MaybeSent);
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
    assert_eq!(SendOutcome::classify(&error), SendOutcome::MaybeSent);
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
    let rendered = error.to_string();

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
        client.config().timeout_config(),
        client.config().retry_config(),
    );
    assert_resend_guard(&client);
}
