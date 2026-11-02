//! Builds AWS SDK configurations with provider-wide timeout and retry bounds.

use std::time::Duration;

use aws_config::Region;
use aws_credential_types::provider::SharedCredentialsProvider;
use aws_smithy_types::retry::RetryConfig;
use aws_smithy_types::timeout::TimeoutConfig;

use crate::provider::assume_role::AssumeRoleConfig;
use crate::resend_safety::cloudcontrol_idempotency;

/// An AWS SDK configuration with the provider's timeout and retry policy applied.
#[derive(Clone)]
pub struct BoundedSdkConfig(aws_config::SdkConfig);

pub(crate) struct SdkClients {
    pub(crate) cloudcontrol: aws_sdk_cloudcontrol::Client,
    pub(crate) s3: aws_sdk_s3::Client,
    pub(crate) sts: aws_sdk_sts::Client,
    pub(crate) kms: crate::references::KmsClient,
}

impl BoundedSdkConfig {
    pub(crate) fn sdk_config(&self) -> &aws_config::SdkConfig {
        &self.0
    }

    /// Apply the provider's timeout and retry policy to an SDK configuration.
    fn new(config: aws_config::SdkConfig) -> Self {
        Self(
            config
                .into_builder()
                .timeout_config(timeout_config())
                .retry_config(retry_config())
                .build(),
        )
    }

    /// Build a bounded configuration for native tests with explicit credentials.
    #[cfg(any(test, feature = "test-utils"))]
    #[cfg(not(target_arch = "wasm32"))]
    pub fn for_test(config: aws_config::SdkConfig) -> Self {
        Self::new(config)
    }

    /// Load a bounded SDK configuration for the region and optional assumed role.
    pub(crate) async fn load(region: &str, assume_role: Option<&AssumeRoleConfig>) -> Self {
        Self::load_with_overrides(region, assume_role, None, None).await
    }

    async fn load_with_overrides(
        region: &str,
        assume_role: Option<&AssumeRoleConfig>,
        http_client: Option<aws_smithy_runtime_api::client::http::SharedHttpClient>,
        credentials_provider: Option<SharedCredentialsProvider>,
    ) -> Self {
        // Give the loader the policy too so invalid AWS retry settings cannot panic first.
        // This is the sole SDK loader; wasm32 always installs host credentials below.
        #[allow(clippy::disallowed_methods)]
        let loader = aws_config::defaults(aws_config::BehaviorVersion::latest())
            .region(Region::new(region.to_string()))
            .timeout_config(timeout_config())
            .retry_config(retry_config());
        let loader = match http_client {
            Some(http_client) => loader.http_client(http_client),
            None => loader,
        };
        #[cfg(not(target_arch = "wasm32"))]
        let loader = match base_credentials_provider(credentials_provider) {
            Some(credentials_provider) => loader.credentials_provider(credentials_provider),
            None => loader,
        };
        #[cfg(target_arch = "wasm32")]
        let loader = loader.credentials_provider(base_credentials_provider(credentials_provider));
        #[cfg(target_arch = "wasm32")]
        let loader = loader.http_client(carina_plugin_sdk::wasi_http::WasiHttpClient::new());
        let base = Self::new(loader.load().await);

        match assume_role {
            None => base,
            Some(assume_role) => base.with_assume_role(assume_role).await,
        }
    }

    async fn with_assume_role(self, assume_role: &AssumeRoleConfig) -> Self {
        let mut builder =
            aws_config::sts::AssumeRoleProvider::builder(assume_role.role_arn.clone())
                .configure(&self.0);
        if let Some(name) = assume_role.session_name.as_deref() {
            builder = builder.session_name(name);
        }
        if let Some(external_id) = assume_role.external_id.as_deref() {
            builder = builder.external_id(external_id);
        }
        if let Some(duration) = assume_role.duration {
            builder = builder.session_length(duration);
        }

        let provider = builder.build().await;
        let shared = aws_credential_types::provider::SharedCredentialsProvider::new(provider);
        Self::new(self.0.into_builder().credentials_provider(shared).build())
    }

    // Service clients share the bounded config, including host credentials on wasm32.
    // CloudControl gets its guard here; the KMS wrapper guards regional clients.
    #[allow(clippy::disallowed_methods)]
    pub(crate) fn clients(&self) -> SdkClients {
        let cloudcontrol = aws_sdk_cloudcontrol::Client::from_conf(
            aws_sdk_cloudcontrol::config::Builder::from(&self.0)
                .interceptor(carina_aws_types::ResendGuard::new(cloudcontrol_idempotency))
                .build(),
        );
        SdkClients {
            cloudcontrol,
            s3: aws_sdk_s3::Client::new(&self.0),
            sts: aws_sdk_sts::Client::new(&self.0),
            kms: crate::references::KmsClient::from_bounded_config(self),
        }
    }
}

#[cfg(target_arch = "wasm32")]
fn base_credentials_provider(
    _override_provider: Option<SharedCredentialsProvider>,
) -> SharedCredentialsProvider {
    SharedCredentialsProvider::new(carina_aws_types::HostCredentialsProvider::default())
}

#[cfg(not(target_arch = "wasm32"))]
fn base_credentials_provider(
    override_provider: Option<SharedCredentialsProvider>,
) -> Option<SharedCredentialsProvider> {
    override_provider
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

    use super::{BoundedSdkConfig, assert_bounded_policy};

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn native_base_keeps_default_credentials_chain() {
        assert!(super::base_credentials_provider(None).is_none());
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[tokio::test]
    async fn native_base_keeps_credentials_override() {
        use aws_credential_types::{Credentials, provider::ProvideCredentials};

        let credentials = Credentials::new(
            "override-access-key",
            "override-secret",
            Some("override-token".into()),
            None,
            "sdk-config-test",
        );
        let provider = super::base_credentials_provider(Some(
            super::SharedCredentialsProvider::new(credentials),
        ))
        .expect("the explicit credentials override must be retained");
        let credentials = provider.provide_credentials().await.unwrap();

        assert_eq!(credentials.access_key_id(), "override-access-key");
        assert_eq!(credentials.secret_access_key(), "override-secret");
        assert_eq!(credentials.session_token(), Some("override-token"));
    }

    #[test]
    fn new_overrides_input_timeout_and_retry_config() {
        let input = aws_config::SdkConfig::builder()
            .timeout_config(TimeoutConfig::disabled())
            .retry_config(RetryConfig::standard().with_max_attempts(10))
            .build();

        let bounded = BoundedSdkConfig::new(input);
        assert_bounded_policy(bounded.0.timeout_config(), bounded.0.retry_config());
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[tokio::test]
    async fn load_applies_bounded_policy() {
        let mock = winterbaume_core::MockAws::builder().build();
        let bounded = BoundedSdkConfig::load_with_overrides(
            "us-east-1",
            None,
            Some(mock.http_client()),
            Some(mock.credentials_provider()),
        )
        .await;
        assert_bounded_policy(bounded.0.timeout_config(), bounded.0.retry_config());
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[tokio::test]
    async fn kms_routes_use_assumed_credentials_and_instance_or_arn_region() {
        use carina_aws_types::AwsType;
        use carina_provider_protocol::types::{
            ReferenceError, ReferenceOutcome, ReferenceRequest, TypeIdentity,
        };
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
        let config = BoundedSdkConfig::load_with_overrides(
            "us-east-1",
            Some(&role),
            Some(mock.http_client()),
            Some(mock.credentials_provider()),
        )
        .await;
        let provider = crate::AwsccProvider::from_sdk_config(config, &Default::default()).await;
        let client_config = provider.kms_client().unwrap().config("eu-west-1");
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
        assert_eq!(error.provider_name.as_deref(), Some("awscc"));
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
                    ty: AwsType::KmsKeyReference.identity(),
                    value: "alias/aws/ecr".into(),
                },
                carina_core::references::ReferenceRequest {
                    ty: AwsType::KmsKeyReference.identity(),
                    value: REMOTE_KEY.into(),
                },
                carina_core::references::ReferenceRequest {
                    ty: AwsType::KmsKeyArn.identity(),
                    value: "alias/unsupported".into(),
                },
            ],
        )
        .await
        .unwrap();
        let canonical = |value: &str| match carina_core::references::ReferenceOutcome::from_guest(
            ReferenceOutcome::Resolved(value.into()),
        )
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
}
