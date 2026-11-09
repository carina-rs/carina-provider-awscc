//! Builds AWS SDK configurations with provider-wide timeout and retry bounds.

use std::time::Duration;

use aws_smithy_types::retry::RetryConfig;
use aws_smithy_types::timeout::TimeoutConfig;
use aws_types::region::Region;
use carina_aws_types::sdk::{cloudcontrol::CloudControlClient, s3::S3Client, sts::StsClient};

use crate::provider::assume_role::AssumeRoleConfig;
use crate::resend_safety::{cloudcontrol_idempotency, s3_idempotency, sts_idempotency};

/// An AWS SDK configuration with the provider's timeout and retry policy applied.
#[derive(Clone)]
pub struct BoundedSdkConfig(carina_aws_types::AwsSdkConfig);

pub(crate) struct SdkClients {
    pub(crate) cloudcontrol: CloudControlClient,
    pub(crate) s3: S3Client,
    pub(crate) sts: StsClient,
}

impl BoundedSdkConfig {
    pub(crate) fn sdk_config(&self) -> &carina_aws_types::AwsSdkConfig {
        &self.0
    }

    pub(crate) fn diagnostics(&self) -> &carina_plugin_sdk::ProviderDiagnostics {
        self.0.diagnostics()
    }

    /// Retain effective credentials, diagnostics and bounds for regional routes.
    pub(crate) fn in_region(&self, region: &str) -> Self {
        Self(self.0.clone().in_region(Region::new(region.to_owned())))
    }

    /// Apply the provider's timeout and retry policy to a session-bound config.
    fn new(config: carina_aws_types::AwsSdkConfig) -> Self {
        Self(config.with_policy(timeout_config(), retry_config()))
    }

    /// Build a bounded configuration for native tests with explicit credentials.
    #[cfg(any(test, feature = "test-utils"))]
    #[cfg(not(target_arch = "wasm32"))]
    pub fn for_test(config: aws_types::SdkConfig) -> Self {
        Self::new(carina_aws_types::AwsSdkConfig::for_test(config))
    }

    /// Load through the shared credential-registration and diagnostics path.
    pub(crate) async fn load(
        region: &str,
        assume_role: Option<&AssumeRoleConfig>,
        diagnostics: carina_plugin_sdk::ProviderDiagnostics,
    ) -> Self {
        let base = Self::new(
            carina_aws_types::load_sdk_config(
                Region::new(region.to_owned()),
                timeout_config(),
                retry_config(),
                diagnostics,
                None,
            )
            .await,
        );
        match assume_role {
            None => base,
            Some(assume_role) => base.with_assume_role(assume_role),
        }
    }

    fn with_assume_role(self, assume_role: &AssumeRoleConfig) -> Self {
        let provider = carina_aws_types::GuardedAssumeRoleProvider::new(
            &self.0,
            assume_role.role_arn.clone(),
            assume_role.session_name.clone(),
            assume_role.external_id.clone(),
            assume_role.duration,
        );
        Self::new(self.0.with_source(provider))
    }

    // Service clients share the bounded config, including host credentials on wasm32.
    // KMS requires InitializedState. Every service handle installs the resend guard.
    pub(crate) fn clients(&self) -> SdkClients {
        SdkClients {
            cloudcontrol: CloudControlClient::new(&self.0, cloudcontrol_idempotency),
            s3: S3Client::new(&self.0, s3_idempotency),
            sts: StsClient::new(&self.0, sts_idempotency),
        }
    }
}

/// Sets read and per-attempt timeouts to the host's 30-second `wasi:http` per-request
/// cap (`HTTP_API_REQUEST_TIMEOUT` in `carina-plugin-host`), so WASM requests get no
/// shorter deadline than the host already enforces. The 3.1-second connect timeout is
/// the AWS SDK's default connect timeout, now set explicitly. The operation timeout
/// caps one SDK call at 60 seconds natively. In WASM the HTTP client sends synchronously,
/// so connect/read timeouts reach `wasi:http`, but attempt/operation timeouts are
/// checked only between attempts and one call can overrun 60 seconds by up to one attempt.
fn timeout_config() -> TimeoutConfig {
    TimeoutConfig::builder()
        .connect_timeout(Duration::from_millis(3_100))
        .read_timeout(Duration::from_secs(30))
        .operation_attempt_timeout(Duration::from_secs(30))
        .operation_timeout(Duration::from_secs(60))
        .build()
}

/// Uses standard retry mode with three total attempts, overriding `AWS_MAX_ATTEMPTS`,
/// `AWS_RETRY_MODE`, and profile settings.
fn retry_config() -> RetryConfig {
    RetryConfig::standard().with_max_attempts(3)
}

#[cfg(test)]
pub(crate) fn assert_bounded_policy(timeout: Option<&TimeoutConfig>, retry: Option<&RetryConfig>) {
    use aws_smithy_types::retry::RetryMode;

    let timeout = timeout.expect("timeout config should be set");
    assert_eq!(
        timeout.connect_timeout(),
        Some(Duration::from_millis(3_100))
    );
    assert_eq!(timeout.read_timeout(), Some(Duration::from_secs(30)));
    assert_eq!(
        timeout.operation_attempt_timeout(),
        Some(Duration::from_secs(30))
    );
    assert_eq!(timeout.operation_timeout(), Some(Duration::from_secs(60)));

    let retry = retry.expect("retry config should be set");
    assert_eq!(retry.mode(), RetryMode::Standard);
    assert_eq!(retry.max_attempts(), 3);
}

#[cfg(test)]
mod tests {
    use aws_smithy_types::retry::RetryConfig;
    use aws_smithy_types::timeout::TimeoutConfig;
    use carina_core::test_utils::provider_text::BindProviderText as _;

    use super::{BoundedSdkConfig, assert_bounded_policy};

    #[test]
    fn new_overrides_input_timeout_and_retry_config() {
        let input = aws_types::SdkConfig::builder()
            .timeout_config(TimeoutConfig::disabled())
            .retry_config(RetryConfig::standard().with_max_attempts(10))
            .build();

        let bounded = BoundedSdkConfig::for_test(input);
        assert_bounded_policy(
            bounded.0.sdk_config().timeout_config(),
            bounded.0.sdk_config().retry_config(),
        );
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[tokio::test]
    async fn load_applies_bounded_policy() {
        use aws_credential_types::provider::ProvideCredentials;

        let mock = winterbaume_core::MockAws::builder().build();
        let session =
            carina_core::provider::ProviderPlan::state_only().session("awscc", None, None);
        let expected = mock
            .credentials_provider()
            .provide_credentials()
            .await
            .unwrap();
        let bounded = BoundedSdkConfig::new(
            carina_aws_types::load_sdk_config(
                aws_types::region::Region::new("us-east-1"),
                super::timeout_config(),
                super::retry_config(),
                carina_plugin_sdk::ProviderDiagnostics::from_session(&session),
                Some(expected.clone().into()),
            )
            .await,
        );
        assert_bounded_policy(
            bounded.0.sdk_config().timeout_config(),
            bounded.0.sdk_config().retry_config(),
        );
        assert_eq!(
            bounded.0.sdk_config().region().unwrap().as_ref(),
            "us-east-1"
        );
        let actual = bounded
            .0
            .sdk_config()
            .credentials_provider()
            .unwrap()
            .provide_credentials()
            .await
            .unwrap();
        assert_eq!(actual.access_key_id(), expected.access_key_id());
        assert_eq!(actual.secret_access_key(), expected.secret_access_key());
        assert_eq!(actual.session_token(), expected.session_token());
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[tokio::test(start_paused = true)]
    async fn assumed_credentials_do_not_resend_after_a_possibly_sent_timeout() {
        use std::error::Error;
        use std::sync::Arc;
        use std::sync::atomic::{AtomicUsize, Ordering};

        use aws_credential_types::Credentials;
        use aws_credential_types::provider::ProvideCredentials;
        use aws_smithy_runtime_api::client::http::{
            HttpConnector, HttpConnectorFuture, SharedHttpConnector, http_client_fn,
        };
        use aws_smithy_runtime_api::client::orchestrator::HttpRequest;
        use aws_smithy_runtime_api::client::result::ConnectorError;

        #[derive(Clone, Debug)]
        struct PostSendTimeout(Arc<AtomicUsize>);

        impl HttpConnector for PostSendTimeout {
            fn call(&self, _request: HttpRequest) -> HttpConnectorFuture {
                self.0.fetch_add(1, Ordering::SeqCst);
                HttpConnectorFuture::new(async {
                    Err(ConnectorError::timeout(
                        std::io::Error::other("AssumeRole response timed out").into(),
                    ))
                })
            }
        }

        let requests = Arc::new(AtomicUsize::new(0));
        let connector = PostSendTimeout(requests.clone());
        let http_client = http_client_fn(move |_, _| SharedHttpConnector::new(connector.clone()));
        let role = crate::provider::assume_role::AssumeRoleConfig {
            role_arn: "arn:aws:iam::123456789012:role/test".into(),
            session_name: Some("guard-test".into()),
            external_id: Some("test-external-id".into()),
            duration: Some(std::time::Duration::from_secs(900)),
        };
        let config = BoundedSdkConfig::for_test(
            aws_types::SdkConfig::builder()
                .region(aws_types::region::Region::new("us-east-1"))
                .http_client(http_client)
                .sleep_impl(aws_smithy_async::rt::sleep::TokioSleep::new())
                .credentials_provider(
                    aws_credential_types::provider::SharedCredentialsProvider::new(
                        Credentials::new(
                            "test-access-key",
                            "test-secret-key",
                            None,
                            None,
                            "assume-role-guard-test",
                        ),
                    ),
                )
                .build(),
        )
        .with_assume_role(&role);

        let error = config
            .0
            .sdk_config()
            .credentials_provider()
            .unwrap()
            .provide_credentials()
            .await
            .expect_err("a possibly-sent AssumeRole must fail without issuing another request");

        assert_eq!(requests.load(Ordering::SeqCst), 1);
        let failure = error
            .source()
            .unwrap()
            .downcast_ref::<carina_aws_types::AssumeRoleFailure>()
            .expect("the guarded provider should retain the send classification");
        assert!(matches!(
            failure.classification(),
            carina_aws_types::SendClassification::MaybeSent { .. }
        ));
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[tokio::test]
    async fn kms_routes_use_assumed_credentials_and_instance_or_arn_region() {
        use carina_plugin_sdk::types::{ReferenceError, ReferenceOutcome};
        use carina_provider_protocol::types::{ReferenceRequest, TypeIdentity};
        use std::sync::{Arc, Mutex};
        use winterbaume_core::{MockAws, MockRequest, MockResponse, MockService};

        const INSTANCE_KEY: &str =
            "arn:aws:kms:us-east-1:222222222222:key/11111111-1111-1111-1111-111111111111";
        const REMOTE_KEY: &str =
            "arn:aws:kms:eu-west-1:999999999999:key/22222222-2222-2222-2222-222222222222";
        #[derive(Default)]
        struct Calls {
            sts: usize,
            kms: Vec<(String, String)>,
        }
        struct ScopedKms(Arc<Mutex<Calls>>);
        impl MockService for ScopedKms {
            fn service_name(&self) -> &str {
                "kms-scope-test"
            }
            fn url_patterns(&self) -> Vec<&str> {
                vec![r"^https://(kms|sts)\."]
            }
            fn handle(
                &self,
                request: MockRequest,
            ) -> std::pin::Pin<Box<dyn std::future::Future<Output = MockResponse> + Send + '_>>
            {
                let response = if request.uri.contains("sts.") {
                    self.0.lock().unwrap().sts += 1;
                    MockResponse::xml(
                        200,
                        "<AssumeRoleResponse xmlns=\"https://sts.amazonaws.com/doc/2011-06-15/\"><AssumeRoleResult><Credentials><AccessKeyId>ASSUMEDKMSKEY</AccessKeyId><SecretAccessKey>fake-assumed-secret</SecretAccessKey><SessionToken>fake-assumed-token</SessionToken><Expiration>2099-01-01T00:00:00Z</Expiration></Credentials><AssumedRoleUser><AssumedRoleId>ROLE:session</AssumedRoleId><Arn>arn:aws:sts::222222222222:assumed-role/test/session</Arn></AssumedRoleUser></AssumeRoleResult><ResponseMetadata><RequestId>sts-test</RequestId></ResponseMetadata></AssumeRoleResponse>",
                    )
                } else {
                    assert_eq!(
                        request.headers.get("x-amz-target").unwrap(),
                        "TrentService.DescribeKey"
                    );
                    assert!(
                        request
                            .headers
                            .get("authorization")
                            .unwrap()
                            .to_str()
                            .unwrap()
                            .contains("Credential=ASSUMEDKMSKEY/")
                    );
                    assert_eq!(
                        request.headers.get("x-amz-security-token").unwrap(),
                        "fake-assumed-token"
                    );
                    let body: serde_json::Value = serde_json::from_slice(&request.body).unwrap();
                    let raw = body["KeyId"].as_str().unwrap();
                    self.0
                        .lock()
                        .unwrap()
                        .kms
                        .push((request.uri.clone(), raw.to_string()));
                    if raw == "alias/denied" {
                        MockResponse::json(
                            400,
                            r#"{"__type":"AccessDeniedException","message":"kms:DescribeKey denied"}"#,
                        )
                    } else {
                        let arn = if raw == "alias/aws/ecr" {
                            assert_eq!(request.uri, "https://kms.us-east-1.amazonaws.com/");
                            INSTANCE_KEY
                        } else {
                            assert_eq!(raw, "arn:aws:kms:eu-west-1:999999999999:alias/customer");
                            assert_eq!(request.uri, "https://kms.eu-west-1.amazonaws.com/");
                            REMOTE_KEY
                        };
                        MockResponse::json(
                            200,
                            format!(r#"{{"KeyMetadata":{{"KeyId":"test","Arn":"{arn}"}}}}"#),
                        )
                    }
                };
                Box::pin(async move { response })
            }
        }

        let calls = Arc::new(Mutex::new(Calls::default()));
        let mock = MockAws::builder()
            .with_service(ScopedKms(calls.clone()))
            .build();
        let role = crate::provider::assume_role::AssumeRoleConfig {
            role_arn: "arn:aws:iam::222222222222:role/test".into(),
            session_name: Some("kms-test".into()),
            external_id: None,
            duration: None,
        };
        let config =
            BoundedSdkConfig::for_test(mock.sdk_config("us-east-1").await).with_assume_role(&role);
        let provider = crate::AwsccProvider::from_sdk_config(config, &Default::default()).await;
        let client = provider.kms_client().unwrap().guarded_client("eu-west-1");
        let client_config = client.inner().config();
        assert_bounded_policy(client_config.timeout_config(), client_config.retry_config());
        use aws_smithy_runtime_api::client::interceptors::Intercept;
        assert!(
            client_config
                .interceptors()
                .any(|interceptor| interceptor.name() == "carina.ResendGuard")
        );

        let request = |value: &str| ReferenceRequest {
            ty: TypeIdentity {
                provider: "aws".into(),
                segments: vec!["kms".into(), "Key".into()],
                kind: "Reference".into(),
            },
            value: value.into(),
        };
        let results = provider
            .resolve_references(vec![
                request("alias/aws/ecr"),
                request("alias/aws/ecr"),
                request(INSTANCE_KEY),
                request("arn:aws:kms:eu-west-1:999999999999:alias/customer"),
                request("alias/denied"),
            ])
            .await;
        assert_eq!(results.len(), 5);
        for result in &results[..3] {
            assert!(matches!(result, Ok(ReferenceOutcome::Resolved(arn)) if arn == INSTANCE_KEY));
        }
        assert!(matches!(&results[3], Ok(ReferenceOutcome::Resolved(arn)) if arn == REMOTE_KEY));
        let Err(ReferenceError::AccessDenied(error)) = &results[4] else {
            panic!("expected typed access denial");
        };
        let error = error.as_ref().clone().into_host_for_test().to_wire();
        assert_eq!(
            error
                .provider_name
                .as_ref()
                .map(ToString::to_string)
                .as_deref(),
            Some("awscc")
        );
        assert_eq!(error.code.as_deref(), Some("AccessDeniedException"));
        assert_eq!(error.status, Some(400));
        let rendered = format!("{error:?}");
        for secret in ["ASSUMEDKMSKEY", "fake-assumed-secret", "fake-assumed-token"] {
            assert!(
                !rendered.contains(secret),
                "reference errors must not include credentials"
            );
        }
        assert_eq!(
            calls.lock().unwrap().kms.len(),
            4,
            "the shared resolver preserves batch positions and short-circuits key ARNs"
        );

        let native = carina_core::provider::Provider::resolve_references(
            &provider,
            &[
                carina_core::references::ReferenceRequest {
                    ty: carina_aws_types::AwsType::KmsKeyReference.identity(),
                    value: "alias/aws/ecr".into(),
                },
                carina_core::references::ReferenceRequest {
                    ty: carina_aws_types::AwsType::KmsKeyReference.identity(),
                    value: REMOTE_KEY.into(),
                },
                carina_core::references::ReferenceRequest {
                    ty: carina_aws_types::AwsType::KmsKeyArn.identity(),
                    value: "alias/unsupported".into(),
                },
            ],
        )
        .await
        .unwrap();
        let canonical =
            |value: &str| match carina_core::references::ReferenceOutcome::from_guest(value.into())
                .unwrap()
            {
                carina_core::references::ReferenceOutcome::Resolved(value) => value,
                carina_core::references::ReferenceOutcome::NotFound(_) => unreachable!(),
            };
        assert!(
            matches!(&native[0], Ok(carina_core::references::ReferenceOutcome::Resolved(arn)) if arn == &canonical(INSTANCE_KEY))
        );
        assert!(
            matches!(&native[1], Ok(carina_core::references::ReferenceOutcome::Resolved(arn)) if arn == &canonical(REMOTE_KEY))
        );
        assert!(matches!(
            &native[2],
            Err(carina_core::references::ReferenceError::InvalidReference(_))
        ));
        let calls = calls.lock().unwrap();
        assert_eq!(calls.kms.len(), 5);
        assert_eq!(
            calls.sts, 1,
            "all regional clients must reuse the effective assumed-role session"
        );
    }
    #[tokio::test]
    async fn terminal_retry_emits_a_typed_redacted_failure_on_the_config_session() {
        use carina_core::provider_text::present_event;
        use carina_core::test_utils::provider_text::{captured_session, context};
        use carina_plugin_sdk::{EventLevel, ProviderDiagnostics};
        use winterbaume_core::{MockAws, MockRequest, MockResponse, MockService};

        struct RetryableStatus;
        impl MockService for RetryableStatus {
            fn service_name(&self) -> &str {
                "retry-diagnostics"
            }
            fn url_patterns(&self) -> Vec<&str> {
                vec![r"^https://cloudcontrolapi\."]
            }
            fn handle(
                &self,
                _: MockRequest,
            ) -> std::pin::Pin<Box<dyn std::future::Future<Output = MockResponse> + Send + '_>>
            {
                Box::pin(async {
                    MockResponse::json(
                        200,
                        r#"{"ProgressEvent":{"OperationStatus":"FAILED","StatusMessage":"Throttling session-retry-canary DIAGNOSTICKEY","RequestToken":"diagnostic-request"}}"#,
                    )
                })
            }
        }
        let mock = MockAws::builder().with_service(RetryableStatus).build();
        let fixture = context(&["session-retry-canary"]);
        let (session, events) = captured_session(fixture.session_builder());
        let source = aws_credential_types::Credentials::new(
            "DIAGNOSTICKEY",
            "diagnostic-secret",
            Some("diagnostic-token".into()),
            None,
            "retry-diagnostics-test",
        );
        let config = BoundedSdkConfig::new(carina_aws_types::AwsSdkConfig::from_source(
            mock.sdk_config("us-east-1").await,
            source,
            ProviderDiagnostics::from_session(&session),
        ));
        let provider = crate::AwsccProvider::from_sdk_config(config, &Default::default()).await;
        let token = serde_json::from_value(
            serde_json::json!({"phase":"in_flight", "request":"diagnostic-request", "attempt":0}),
        )
        .unwrap();
        let step = provider
            .cc_poll_update_step("AWS::Test::Resource", "diagnostic-id", vec![], None, token)
            .await
            .unwrap();
        assert!(matches!(
            step,
            carina_provider_protocol::OperationStep::Pending { .. }
        ));
        let events = events.lock().unwrap();
        let failures: Vec<_> = events
            .iter()
            .filter(|event| event.failure().is_some())
            .collect();
        assert_eq!(failures.len(), 1);
        let event = failures[0];
        assert_eq!(event.severity(), EventLevel::Warn);
        let failure = event.failure().unwrap();
        assert!(
            failure
                .detail()
                .cause()
                .unwrap()
                .is::<crate::provider::error_helpers::CloudControlOperationFailure>()
        );
        let rendered = present_event(event).to_string();
        assert!(rendered.contains("Throttling"));
        assert!(!rendered.contains("session-retry-canary"));
        assert!(!rendered.contains("DIAGNOSTICKEY"));
    }
}
