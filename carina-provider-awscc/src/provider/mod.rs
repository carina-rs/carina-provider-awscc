//! AWS Cloud Control Provider implementation
//!
//! This module contains the main provider implementation that communicates
//! with AWS Cloud Control API to manage resources.
//!
//! ## Sub-modules
//!
//! - `cloudcontrol` - Low-level Cloud Control API methods and retry/error handling
//! - `conversion` - Value conversion between DSL and AWS JSON formats
//! - `normalizer` - State normalization and hydration
//! - `operations` - High-level resource operations (read, create, update, delete)
//! - `s3` - S3-specific operations (empty bucket for force_delete)
//! - `special_cases` - Resource-type-specific attribute handling
//! - `tags` - Tag conversion between DSL and CloudFormation formats
//! - `update` - Update patch building and resource property parsing

pub(crate) mod account_guard;
mod arn_synthesis;
pub mod assume_role;
mod cloudcontrol;
pub(crate) mod conversion;
pub(crate) mod error_helpers;
mod normalizer;
mod operations;
pub mod read_back;
mod s3;
mod special_cases;
mod tags;
pub(crate) mod update;

use carina_aws_types::sdk::{cloudcontrol::CloudControlClient, s3::S3Client, sts::StsClient};
use carina_core::provider::ProviderError;
use tokio::sync::OnceCell;

use crate::references::InitializedState;
use crate::schemas::config::AwsccSchemaConfig;
use crate::sdk_config::{BoundedSdkConfig, SdkClients};
use error_helpers::api_error_with_meta;

// Re-export public API
pub use cloudcontrol::{
    CcCreateOperation, CcCreateToken, CcDeleteToken, CcRequestPhase, CcUpdateToken, ClientToken,
    InFlightCreate, InFlightCreateRequest, InFlightRequest, NotAppliedCreate, NotIssuedCreate,
    NotIssuedRequest, RequestToken, UnconfirmedCreate, UnconfirmedRequest,
};
pub use normalizer::{
    canonicalize_string_or_list_states_impl, normalize_state_string_dsl_transforms_impl,
};
#[doc(hidden)]
pub use operations::{
    CcCreateAttempt, CcCreatePoll, CcCreateResume, CcIssuedCreate, CcPollAttempt, CcResumeAttempt,
    CcResumedCreate,
};
pub(crate) use update::parse_resource_properties;

/// Maximum number of retry attempts for retryable create errors
const CREATE_RETRY_MAX_ATTEMPTS: u32 = 12;

/// Initial delay in seconds before retrying a failed create operation
const CREATE_RETRY_INITIAL_DELAY_SECS: u64 = 10;

/// Maximum delay in seconds between create retry attempts
const CREATE_RETRY_MAX_DELAY_SECS: u64 = 120;

/// Maximum number of retry attempts for retryable delete errors
const DELETE_RETRY_MAX_ATTEMPTS: u32 = 12;

/// Initial delay in seconds before retrying a failed delete operation
const DELETE_RETRY_INITIAL_DELAY_SECS: u64 = 10;

/// Maximum delay in seconds between delete retry attempts
const DELETE_RETRY_MAX_DELAY_SECS: u64 = 120;

/// Provider-level configuration that affects AwsccProvider construction.
///
/// Carries the `allowed_account_ids` / `forbidden_account_ids` guard
/// lists (both empty = "no check") and, when present, a parsed
/// `assume_role` block that layers a guarded STS credentials provider on top
/// of base credentials (host-supplied on wasm32, the default chain natively).
#[derive(Debug, Default, Clone)]
pub struct AwsccProviderConfig {
    pub allowed_account_ids: Vec<String>,
    pub forbidden_account_ids: Vec<String>,
    pub assume_role: Option<assume_role::AssumeRoleConfig>,
}

/// AWS Cloud Control Provider
pub struct AwsccProvider {
    cloudcontrol_client: CloudControlClient,
    s3_client: S3Client,
    sts_client: StsClient,
    /// KMS is reachable only after the shared account guard has succeeded.
    initialized: Result<InitializedState, ProviderError>,
    diagnostics: carina_plugin_sdk::ProviderDiagnostics,
    /// Caller's AWS account id, fetched lazily via `sts:GetCallerIdentity`
    /// the first time a synthesized attribute (e.g.
    /// `cloudfront.Distribution.arn`) needs it, then cached for the
    /// lifetime of the provider instance.
    account_id: OnceCell<String>,
    #[cfg(any(test, feature = "test-utils"))]
    operation_config_overrides:
        std::collections::HashMap<String, carina_core::schema::OperationConfig>,
}

impl AwsccProvider {
    /// Create a new AwsccProvider for the specified region with no
    /// account guard. Convenience wrapper for tests and other callers
    /// that do not need `allowed_account_ids` / `forbidden_account_ids`.
    pub async fn new(region: &str, diagnostics: carina_plugin_sdk::ProviderDiagnostics) -> Self {
        Self::new_with_config(region, &AwsccProviderConfig::default(), diagnostics).await
    }

    /// Create a new AwsccProvider for the specified region, applying
    /// the account-guard policy in `cfg`.
    ///
    /// When either `allowed_account_ids` or `forbidden_account_ids` is
    /// non-empty, this call invokes `sts:GetCallerIdentity` once and
    /// records a deferred error if the caller's account violates the
    /// policy. The provider is still returned (so the host's wiring
    /// stays infallible), but every `Provider` trait method on the
    /// returned instance will surface that error before any
    /// CloudControl call.
    pub async fn new_with_config(
        region: &str,
        cfg: &AwsccProviderConfig,
        diagnostics: carina_plugin_sdk::ProviderDiagnostics,
    ) -> Self {
        let config = BoundedSdkConfig::load(region, cfg.assume_role.as_ref(), diagnostics).await;
        Self::from_sdk_config(config, cfg).await
    }

    /// Create a new AwsccProvider from a bounded AWS SDK configuration.
    pub async fn from_sdk_config(config: BoundedSdkConfig, cfg: &AwsccProviderConfig) -> Self {
        let diagnostics = config.diagnostics().clone();
        let (
            SdkClients {
                cloudcontrol,
                s3,
                sts,
            },
            initialized,
        ) = InitializedState::initialize(config, cfg).await;

        Self {
            cloudcontrol_client: cloudcontrol,
            s3_client: s3,
            sts_client: sts,
            initialized,
            diagnostics,
            account_id: OnceCell::new(),
            #[cfg(any(test, feature = "test-utils"))]
            operation_config_overrides: Default::default(),
        }
    }

    #[cfg(any(test, feature = "test-utils"))]
    #[doc(hidden)]
    pub fn with_operation_config_for_test(
        mut self,
        resource_type: &str,
        config: carina_core::schema::OperationConfig,
    ) -> Self {
        self.operation_config_overrides
            .insert(resource_type.to_string(), config);
        self
    }

    /// Return the caller's AWS account id, fetching it via
    /// `sts:GetCallerIdentity` on first call and caching the result on
    /// the provider for subsequent reads.
    pub(crate) async fn account_id(&self) -> Result<&str, read_back::ReadFailure> {
        self.account_id
            .get_or_try_init(|| async {
                let identity = self
                    .sts_client
                    .inner()
                    .get_caller_identity()
                    .send()
                    .await
                    .map_err(|e| {
                        let transience = Self::read_error_transience(&e);
                        read_back::ReadFailure::new(
                            api_error_with_meta(
                                "Failed to call sts:GetCallerIdentity",
                                "sts.GetCallerIdentity",
                                e,
                            ),
                            transience,
                        )
                    })?;
                identity.account().map(|s| s.to_string()).ok_or_else(|| {
                    read_back::ReadFailure::from(ProviderError::api_error(
                        "sts:GetCallerIdentity returned no account id",
                    ))
                })
            })
            .await
            .map(String::as_str)
    }

    /// Returns the deferred initialization error, if the account guard
    /// rejected the caller during `new_with_config`.
    pub fn init_error(&self) -> Option<&ProviderError> {
        self.initialized.as_ref().err()
    }

    pub(crate) fn kms_client(&self) -> Result<&crate::references::KmsClient, ProviderError> {
        self.initialized
            .as_ref()
            .map(InitializedState::kms_client)
            .map_err(Clone::clone)
    }
}

/// Get the AwsccSchemaConfig for a resource type. O(1) via cached HashMap.
fn get_schema_config(resource_type: &str) -> Option<&'static AwsccSchemaConfig> {
    crate::schemas::generated::get_config_by_type(resource_type)
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use aws_smithy_types::retry::RetryConfig;
    use aws_smithy_types::timeout::TimeoutConfig;
    use winterbaume_core::MockAws;

    use super::{AwsccProvider, AwsccProviderConfig};
    use crate::BoundedSdkConfig;
    use crate::sdk_config::assert_bounded_policy;

    #[tokio::test]
    async fn every_provider_client_has_the_resend_guard() {
        use aws_smithy_runtime_api::client::interceptors::Intercept;

        let mock = MockAws::builder().build();
        let provider = AwsccProvider::from_sdk_config(
            BoundedSdkConfig::for_test(mock.sdk_config("us-east-1").await),
            &AwsccProviderConfig::default(),
        )
        .await;

        macro_rules! assert_guard {
            ($config:expr) => {
                assert!(
                    $config
                        .interceptors()
                        .any(|interceptor| interceptor.name() == "carina.ResendGuard")
                );
            };
        }
        assert_guard!(provider.cloudcontrol_client.inner().config());
        assert_guard!(provider.s3_client.inner().config());
        assert_guard!(provider.sts_client.inner().config());
        let kms = provider.kms_client().unwrap();
        assert_guard!(kms.guarded_client("us-east-1").inner().config());
        assert_guard!(kms.guarded_client("eu-west-1").inner().config());
    }

    #[tokio::test]
    async fn from_sdk_config_applies_policy_to_all_client_config_sources() {
        let mock = MockAws::builder().build();
        let input = aws_types::SdkConfig::builder()
            .http_client(mock.http_client())
            .credentials_provider(mock.credentials_provider())
            .timeout_config(TimeoutConfig::disabled())
            .retry_config(RetryConfig::standard().with_max_attempts(10))
            .build();
        let provider = AwsccProvider::from_sdk_config(
            BoundedSdkConfig::for_test(input),
            &AwsccProviderConfig::default(),
        )
        .await;

        assert_bounded_policy(
            provider
                .cloudcontrol_client
                .inner()
                .config()
                .timeout_config(),
            provider.cloudcontrol_client.inner().config().retry_config(),
        );

        assert_bounded_policy(
            provider.s3_client.inner().config().timeout_config(),
            provider.s3_client.inner().config().retry_config(),
        );
        assert_bounded_policy(
            provider.sts_client.inner().config().timeout_config(),
            provider.sts_client.inner().config().retry_config(),
        );
    }
}
