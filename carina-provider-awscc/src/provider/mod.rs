//! AWS Cloud Control Provider implementation
//!
//! This module contains the main provider implementation that communicates
//! with AWS Cloud Control API to manage resources.
//!
//! ## Sub-modules
//!
//! - `cloudcontrol` - Low-level Cloud Control API methods and retry/error handling
//! - `conversion` - Value conversion between DSL and AWS JSON formats
//! - `normalizer` - Plan-time enum resolution and state hydration
//! - `operations` - High-level resource operations (read, create, update, delete)
//! - `s3` - S3-specific operations (empty bucket for force_delete)
//! - `special_cases` - Resource-type-specific attribute handling
//! - `tags` - Tag conversion between DSL and CloudFormation formats
//! - `update` - Update patch building and resource property parsing

pub(crate) mod account_guard;
mod cloudcontrol;
pub(crate) mod conversion;
mod normalizer;
mod operations;
mod s3;
mod special_cases;
mod tags;
pub(crate) mod update;

use aws_config::Region;
use aws_sdk_cloudcontrol::Client as CloudControlClient;
use aws_sdk_sts::Client as StsClient;

use crate::schemas::generated::AwsccSchemaConfig;

// Re-export public API
pub use normalizer::{
    normalize_state_enums_impl, resolve_enum_identifiers_impl, restore_unreturned_attrs_impl,
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
/// Currently carries the `allowed_account_ids` / `forbidden_account_ids`
/// guard lists. Both empty means "no check", matching the
/// pre-allowed-account-ids behavior.
#[derive(Debug, Default, Clone)]
pub struct AwsccProviderConfig {
    pub allowed_account_ids: Vec<String>,
    pub forbidden_account_ids: Vec<String>,
}

/// AWS Cloud Control Provider
pub struct AwsccProvider {
    cloudcontrol_client: CloudControlClient,
    aws_config: aws_config::SdkConfig,
    /// Set when provider initialization rejected the caller's AWS account
    /// against `allowed_account_ids` / `forbidden_account_ids`. Every
    /// `Provider` trait method short-circuits to this error before
    /// touching CloudControl, so a wrong-account `apply` aborts before
    /// any read/refresh/mutation.
    init_error: Option<String>,
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
        let config = Self::build_config(region).await;

        let init_error =
            if cfg.allowed_account_ids.is_empty() && cfg.forbidden_account_ids.is_empty() {
                None
            } else {
                Self::check_account_guard(&config, cfg).await.err()
            };

        Self {
            cloudcontrol_client: CloudControlClient::new(&config),
            aws_config: config,
            init_error,
        }
    }

    /// Look up the caller's AWS account ID via STS and validate it
    /// against the provider's allowed/forbidden lists.
    ///
    /// Returns `Ok(())` if the account is allowed; otherwise an error
    /// message naming both the offending list and the actual account ID.
    async fn check_account_guard(
        sdk_config: &aws_config::SdkConfig,
        cfg: &AwsccProviderConfig,
    ) -> Result<(), String> {
        let sts = StsClient::new(sdk_config);
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

    #[cfg(not(target_arch = "wasm32"))]
    async fn build_config(region: &str) -> aws_config::SdkConfig {
        aws_config::defaults(aws_config::BehaviorVersion::latest())
            .region(Region::new(region.to_string()))
            .load()
            .await
    }

    #[cfg(target_arch = "wasm32")]
    async fn build_config(region: &str) -> aws_config::SdkConfig {
        use carina_plugin_sdk::wasi_http::WasiHttpClient;
        aws_config::defaults(aws_config::BehaviorVersion::latest())
            .region(Region::new(region.to_string()))
            .http_client(WasiHttpClient::new())
            .load()
            .await
    }
}

/// Get the AwsccSchemaConfig for a resource type. O(1) via cached HashMap.
fn get_schema_config(resource_type: &str) -> Option<&'static AwsccSchemaConfig> {
    crate::schemas::generated::get_config_by_type(resource_type)
}
