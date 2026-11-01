//! Builds AWS SDK configurations with provider-wide timeout and retry bounds.

use std::time::Duration;

use aws_config::Region;
use aws_smithy_types::retry::RetryConfig;
use aws_smithy_types::timeout::TimeoutConfig;

use crate::provider::assume_role::AssumeRoleConfig;
use crate::resend_safety::cloudcontrol_idempotency;

/// An AWS SDK configuration with the provider's timeout and retry policy applied.
pub struct BoundedSdkConfig(aws_config::SdkConfig);

pub(crate) struct SdkClients {
    pub(crate) cloudcontrol: aws_sdk_cloudcontrol::Client,
    pub(crate) s3: aws_sdk_s3::Client,
    pub(crate) sts: aws_sdk_sts::Client,
}

impl BoundedSdkConfig {
    /// Apply the provider's timeout and retry policy to an SDK configuration.
    pub fn new(config: aws_config::SdkConfig) -> Self {
        Self(
            config
                .into_builder()
                .timeout_config(timeout_config())
                .retry_config(retry_config())
                .build(),
        )
    }

    /// Load a bounded SDK configuration for the region and optional assumed role.
    pub(crate) async fn load(region: &str, assume_role: Option<&AssumeRoleConfig>) -> Self {
        Self::load_with_overrides(region, assume_role, None, None).await
    }

    async fn load_with_overrides(
        region: &str,
        assume_role: Option<&AssumeRoleConfig>,
        http_client: Option<aws_smithy_runtime_api::client::http::SharedHttpClient>,
        credentials_provider: Option<aws_credential_types::provider::SharedCredentialsProvider>,
    ) -> Self {
        // Give the loader the policy too so invalid AWS retry settings cannot panic first.
        let loader = aws_config::defaults(aws_config::BehaviorVersion::latest())
            .region(Region::new(region.to_string()))
            .timeout_config(timeout_config())
            .retry_config(retry_config());
        let loader = match http_client {
            Some(http_client) => loader.http_client(http_client),
            None => loader,
        };
        let loader = match credentials_provider {
            Some(credentials_provider) => loader.credentials_provider(credentials_provider),
            None => loader,
        };
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

    use super::{BoundedSdkConfig, assert_bounded_policy};

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
}
