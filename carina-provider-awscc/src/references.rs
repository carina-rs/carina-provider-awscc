//! Resolve comparison-only KMS inputs in the initialized AWSCC instance's scope.

use crate::{AwsccProvider, BoundedSdkConfig};
use carina_aws_types::kms_reference::{
    DescribeKeyClient, DescribeKeyFailure, describe_key_failure, resolve_reference_batch,
};
use carina_aws_types::{Idempotency, ResendGuard};
use carina_core::provider::BoxFuture;
use carina_provider_protocol::types::{ReferenceError, ReferenceOutcome, ReferenceRequest};

/// Regional clients retain the effective credentials provider, including assumed roles
/// and refresh, as well as the provider's timeout, retry and HTTP configuration.
pub(crate) struct KmsClient {
    config: BoundedSdkConfig,
}

impl KmsClient {
    pub(crate) fn from_bounded_config(config: &BoundedSdkConfig) -> Self {
        Self {
            config: config.clone(),
        }
    }

    #[allow(clippy::disallowed_methods)]
    fn client_in_region(&self, region: &str) -> aws_sdk_kms::Client {
        let config = self.config.in_region(region);
        aws_sdk_kms::Client::from_conf(
            aws_sdk_kms::config::Builder::from(config.sdk_config())
                .interceptor(ResendGuard::new(|operation| match operation {
                    // DescribeKey may initialize a managed key; repeating it is safe.
                    "DescribeKey" => Idempotency::ReadOnly,
                    _ => Idempotency::NotIdempotent,
                }))
                .build(),
        )
    }

    #[cfg(test)]
    pub(crate) fn config(&self, region: &str) -> aws_sdk_kms::Config {
        self.client_in_region(region).config().clone()
    }
}

impl DescribeKeyClient for KmsClient {
    fn describe_key<'a>(
        &'a self,
        region: &'a str,
        raw: &'a str,
    ) -> BoxFuture<'a, Result<Option<String>, DescribeKeyFailure>> {
        Box::pin(async move {
            let output = self
                .client_in_region(region)
                .describe_key()
                .key_id(raw)
                .send()
                .await
                .map_err(describe_key_failure)?;
            Ok(output
                .key_metadata()
                .and_then(|metadata| metadata.arn())
                .map(str::to_owned))
        })
    }
}

impl AwsccProvider {
    /// Resolve one positional batch without changing authored or persisted values.
    pub async fn resolve_references(
        &self,
        requests: Vec<ReferenceRequest>,
    ) -> Vec<Result<ReferenceOutcome, ReferenceError>> {
        resolve_reference_batch(&self.kms_client, "awscc", &self.region, requests).await
    }
}
