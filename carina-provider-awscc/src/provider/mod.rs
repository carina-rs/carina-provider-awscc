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
mod error_helpers;
mod normalizer;
mod operations;
mod s3;
mod special_cases;
mod tags;
pub(crate) mod update;

use aws_sdk_cloudcontrol::Client as CloudControlClient;
use aws_sdk_sts::Client as StsClient;
use carina_core::provider::{ProviderError, ProviderResult};
use tokio::sync::OnceCell;

use crate::schemas::config::AwsccSchemaConfig;
use crate::sdk_config::{BoundedSdkConfig, SdkClients};
use error_helpers::api_error_with_meta;

// Re-export public API
pub use cloudcontrol::{
    CcCreateToken, CcDeleteToken, CcRequestPhase, CcUpdateToken, ClientToken,
    InFlightCreateRequest, InFlightRequest, NotAppliedCreate, NotIssuedCreate, NotIssuedRequest,
    RequestToken, UnconfirmedCreate, UnconfirmedRequest, UndeterminedCreate,
};
pub use normalizer::{
    canonicalize_string_or_list_states_impl, normalize_state_string_dsl_transforms_impl,
    restore_unreturned_attrs_impl,
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
/// `assume_role` block that layers an STS `AssumeRoleProvider` on top
/// of the ambient credential chain.
#[derive(Debug, Default, Clone)]
pub struct AwsccProviderConfig {
    pub allowed_account_ids: Vec<String>,
    pub forbidden_account_ids: Vec<String>,
    pub assume_role: Option<assume_role::AssumeRoleConfig>,
}

/// AWS Cloud Control Provider
pub struct AwsccProvider {
    cloudcontrol_client: CloudControlClient,
    s3_client: aws_sdk_s3::Client,
    sts_client: StsClient,
    /// Set when provider initialization rejected the caller's AWS account
    /// against `allowed_account_ids` / `forbidden_account_ids`. Every
    /// `Provider` trait method short-circuits to this error before
    /// touching CloudControl, so a wrong-account `apply` aborts before
    /// any read/refresh/mutation.
    init_error: Option<String>,
    /// Caller's AWS account id, fetched lazily via `sts:GetCallerIdentity`
    /// the first time a synthesized attribute (e.g.
    /// `cloudfront.Distribution.arn`) needs it, then cached for the
    /// lifetime of the provider instance.
    account_id: OnceCell<String>,
}

impl AwsccProvider {
    /// Create a new AwsccProvider for the specified region with no
    /// account guard. Convenience wrapper for tests and other callers
    /// that do not need `allowed_account_ids` / `forbidden_account_ids`.
    pub async fn new(region: &str) -> Self {
        Self::new_with_config(region, &AwsccProviderConfig::default()).await
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
    pub async fn new_with_config(region: &str, cfg: &AwsccProviderConfig) -> Self {
        let config = BoundedSdkConfig::load(region, cfg.assume_role.as_ref()).await;
        Self::from_sdk_config(config, cfg).await
    }

    /// Create a new AwsccProvider from a bounded AWS SDK configuration.
    pub async fn from_sdk_config(config: BoundedSdkConfig, cfg: &AwsccProviderConfig) -> Self {
        let SdkClients {
            cloudcontrol,
            s3,
            sts,
        } = config.clients();
        let init_error =
            if cfg.allowed_account_ids.is_empty() && cfg.forbidden_account_ids.is_empty() {
                None
            } else {
                Self::check_account_guard(&sts, cfg).await.err()
            };

        Self {
            cloudcontrol_client: cloudcontrol,
            s3_client: s3,
            sts_client: sts,
            init_error,
            account_id: OnceCell::new(),
        }
    }

    /// Return the caller's AWS account id, fetching it via
    /// `sts:GetCallerIdentity` on first call and caching the result on
    /// the provider for subsequent reads.
    pub(crate) async fn account_id(&self) -> ProviderResult<&str> {
        self.account_id
            .get_or_try_init(|| async {
                let identity = self
                    .sts_client
                    .get_caller_identity()
                    .send()
                    .await
                    .map_err(|e| {
                        api_error_with_meta(
                            "Failed to call sts:GetCallerIdentity",
                            "sts.GetCallerIdentity",
                            e,
                        )
                    })?;
                identity.account().map(|s| s.to_string()).ok_or_else(|| {
                    ProviderError::api_error("sts:GetCallerIdentity returned no account id")
                })
            })
            .await
            .map(String::as_str)
    }

    /// Look up the caller's AWS account ID via STS and validate it
    /// against the provider's allowed/forbidden lists.
    ///
    /// Returns `Ok(())` if the account is allowed; otherwise an error
    /// message naming both the offending list and the actual account ID.
    async fn check_account_guard(sts: &StsClient, cfg: &AwsccProviderConfig) -> Result<(), String> {
        let identity = sts.get_caller_identity().send().await.map_err(|e| {
            format!(
                "Failed to call sts:GetCallerIdentity to check \
                 allowed_account_ids / forbidden_account_ids: {e}"
            )
        })?;
        let account_id = identity.account().ok_or_else(|| {
            "sts:GetCallerIdentity returned no account ID; \
             cannot evaluate allowed_account_ids / forbidden_account_ids"
                .to_string()
        })?;

        account_guard::validate_account_against_lists(
            account_id,
            &cfg.allowed_account_ids,
            &cfg.forbidden_account_ids,
        )
    }

    /// Returns the deferred initialization error, if the account guard
    /// rejected the caller during `new_with_config`.
    pub fn init_error(&self) -> Option<&str> {
        self.init_error.as_deref()
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
    async fn from_sdk_config_applies_policy_to_all_client_config_sources() {
        let mock = MockAws::builder().build();
        let input = aws_config::SdkConfig::builder()
            .http_client(mock.http_client())
            .credentials_provider(mock.credentials_provider())
            .timeout_config(TimeoutConfig::disabled())
            .retry_config(RetryConfig::standard().with_max_attempts(10))
            .build();
        let provider = AwsccProvider::from_sdk_config(
            BoundedSdkConfig::new(input),
            &AwsccProviderConfig::default(),
        )
        .await;

        assert_bounded_policy(
            provider.cloudcontrol_client.config().timeout_config(),
            provider.cloudcontrol_client.config().retry_config(),
        );

        assert_bounded_policy(
            provider.s3_client.config().timeout_config(),
            provider.s3_client.config().retry_config(),
        );
        assert_bounded_policy(
            provider.sts_client.config().timeout_config(),
            provider.sts_client.config().retry_config(),
        );
    }
}
