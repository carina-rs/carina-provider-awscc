//! Resolve comparison-only KMS inputs in the initialized AWSCC instance's scope.

use crate::provider::{AwsccProviderConfig, account_guard};
use crate::{AwsccProvider, BoundedSdkConfig};
use carina_aws_types::kms_reference::{
    DescribeKeyClient, DescribeKeyFailure, describe_key_failure, resolve_reference_batch,
};
use carina_aws_types::{Idempotency, ResendGuard};
use carina_core::provider::BoxFuture;
use carina_provider_protocol::types::{ReferenceError, ReferenceOutcome, ReferenceRequest};
use std::sync::OnceLock;

/// Only this module can create the witness, after the account policy succeeds.
/// The client is lazy so its constructor can require the initialized state itself.
/// Private fields prevent crate siblings from forging this state; `KmsClient`'s
/// private constructor cannot accept an unchecked `BoundedSdkConfig`.
pub(crate) struct InitializedState {
    config: BoundedSdkConfig,
    kms_client: OnceLock<KmsClient>,
}

impl InitializedState {
    pub(crate) async fn initialize(
        config: BoundedSdkConfig,
        cfg: &AwsccProviderConfig,
    ) -> (crate::sdk_config::SdkClients, Result<Self, String>) {
        let clients = config.clients();
        let guard = if cfg.allowed_account_ids.is_empty() && cfg.forbidden_account_ids.is_empty() {
            Ok(())
        } else {
            Self::check_account_guard(&clients.sts, cfg).await
        };
        let initialized = guard.map(|()| Self {
            config,
            kms_client: OnceLock::new(),
        });
        (clients, initialized)
    }

    pub(crate) fn kms_client(&self) -> &KmsClient {
        self.kms_client.get_or_init(|| KmsClient::new(self))
    }

    /// Look up the caller's AWS account ID via STS and validate it
    /// against the provider's allowed/forbidden lists.
    ///
    /// Returns `Ok(())` if the account is allowed; otherwise an error
    /// message naming both the offending list and the actual account ID.
    async fn check_account_guard(
        sts: &aws_sdk_sts::Client,
        cfg: &AwsccProviderConfig,
    ) -> Result<(), String> {
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
}

/// Regional clients retain the effective credentials provider, including assumed roles
/// and refresh, as well as the provider's timeout, retry and HTTP configuration.
pub(crate) struct KmsClient {
    client: aws_sdk_kms::Client,
}

impl KmsClient {
    #[allow(clippy::disallowed_methods)]
    fn new(initialized: &InitializedState) -> Self {
        let client = aws_sdk_kms::Client::from_conf(
            aws_sdk_kms::config::Builder::from(initialized.config.sdk_config())
                .interceptor(ResendGuard::new(|operation| match operation {
                    // DescribeKey may initialize a managed key; repeating it is safe.
                    "DescribeKey" => Idempotency::ReadOnly,
                    _ => Idempotency::NotIdempotent,
                }))
                .build(),
        );
        Self { client }
    }

    pub(crate) fn instance_region(&self) -> Result<&str, &str> {
        self.client
            .config()
            .region()
            .map(|region| region.as_ref())
            .ok_or("AWS region is not configured")
    }

    #[allow(clippy::disallowed_methods)]
    fn client_in_region(&self, region: &str) -> aws_sdk_kms::Client {
        if self
            .client
            .config()
            .region()
            .is_some_and(|instance| instance.as_ref() == region)
        {
            return self.client.clone();
        }
        aws_sdk_kms::Client::from_conf(
            self.client
                .config()
                .to_builder()
                .region(aws_config::Region::new(region.to_owned()))
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
        let scope = self
            .kms_client()
            .and_then(|client| client.instance_region().map(|region| (client, region)));
        match scope {
            Ok((client, region)) => {
                resolve_reference_batch(client, "awscc", region, requests).await
            }
            Err(reason) => requests
                .into_iter()
                .map(|_| {
                    Err(ReferenceError::InvalidReference(Box::new(
                        carina_core::provider::ProviderError::invalid_input(reason.to_owned())
                            .for_provider("awscc")
                            .to_wire(),
                    )))
                })
                .collect(),
        }
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use super::*;
    use crate::provider::AwsccProviderConfig;
    use std::sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    };
    use winterbaume_core::{MockAws, MockRequest, MockResponse, MockService};

    struct IdentityAndKms(Arc<AtomicUsize>);

    impl MockService for IdentityAndKms {
        fn service_name(&self) -> &str {
            "reference-init"
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
                MockResponse::xml(
                    200,
                    "<GetCallerIdentityResponse xmlns=\"https://sts.amazonaws.com/doc/2011-06-15/\"><GetCallerIdentityResult><Account>123456789012</Account><Arn>arn:aws:iam::123456789012:user/test</Arn><UserId>test</UserId></GetCallerIdentityResult><ResponseMetadata><RequestId>test</RequestId></ResponseMetadata></GetCallerIdentityResponse>",
                )
            } else {
                self.0.fetch_add(1, Ordering::SeqCst);
                MockResponse::json(
                    200,
                    r#"{"KeyMetadata":{"Arn":"arn:aws:kms:us-east-1:123456789012:key/11111111-1111-1111-1111-111111111111"}}"#,
                )
            };
            Box::pin(async move { response })
        }
    }

    fn request(value: &str) -> ReferenceRequest {
        ReferenceRequest {
            ty: carina_provider_protocol::types::TypeIdentity {
                provider: "aws".into(),
                segments: vec!["kms".into(), "Key".into()],
                kind: "Reference".into(),
            },
            value: value.into(),
        }
    }

    #[tokio::test]
    async fn native_batch_rejects_providerless_identity_at_its_original_position() {
        use carina_core::provider::Provider;
        use carina_core::references::{
            ReferenceError as CoreError, ReferenceRequest as CoreRequest,
        };

        let calls = Arc::new(AtomicUsize::new(0));
        let mock = MockAws::builder()
            .with_service(IdentityAndKms(calls.clone()))
            .build();
        let config = aws_config::SdkConfig::builder()
            .region(aws_config::Region::new("us-east-1"))
            .credentials_provider(mock.credentials_provider())
            .http_client(mock.http_client())
            .build();
        let provider =
            AwsccProvider::from_sdk_config(BoundedSdkConfig::for_test(config), &Default::default())
                .await;
        let identity = carina_aws_types::AwsType::KmsKeyReference.identity();
        let mut providerless = identity.clone();
        providerless.provider = None;
        let results = Provider::resolve_references(
            &provider,
            &[
                CoreRequest {
                    ty: identity.clone(),
                    value: "alias/customer".into(),
                },
                CoreRequest {
                    ty: providerless,
                    value: "alias/invalid".into(),
                },
                CoreRequest {
                    ty: identity,
                    value: "alias/other".into(),
                },
            ],
        )
        .await
        .unwrap();
        assert_eq!(results.len(), 3);
        assert!(results[0].is_ok());
        let Err(CoreError::InvalidReference(error)) = &results[1] else {
            panic!("a providerless identity must be rejected at its original position");
        };
        assert!(
            error
                .to_string()
                .contains("reference identity has no provider"),
            "{error}"
        );
        assert!(results[2].is_ok());
        assert_eq!(calls.load(Ordering::SeqCst), 2);
    }

    #[tokio::test]
    async fn guarded_initialization_is_the_only_path_to_the_lazy_client() {
        let mock = MockAws::builder().build();
        let config = aws_config::SdkConfig::builder()
            .region(aws_config::Region::new("us-east-1"))
            .credentials_provider(mock.credentials_provider())
            .http_client(mock.http_client())
            .build();
        let (_, initialized) = InitializedState::initialize(
            BoundedSdkConfig::for_test(config),
            &AwsccProviderConfig::default(),
        )
        .await;
        let initialized = initialized.unwrap();
        assert!(initialized.kms_client.get().is_none());
        let client = initialized.kms_client();
        assert_eq!(client.instance_region().unwrap(), "us-east-1");
        assert!(std::ptr::eq(client, initialized.kms_client()));
    }

    #[tokio::test]
    async fn instance_region_reuses_the_initialized_kms_client() {
        let mock = MockAws::builder().build();
        let config = aws_config::SdkConfig::builder()
            .region(aws_config::Region::new("us-east-1"))
            .credentials_provider(mock.credentials_provider())
            .http_client(mock.http_client())
            .build();
        let provider =
            AwsccProvider::from_sdk_config(BoundedSdkConfig::for_test(config), &Default::default())
                .await;
        let client = provider.kms_client().unwrap();
        let local = client.client_in_region("us-east-1");
        assert!(std::ptr::eq(client.client.config(), local.config()));
        let remote = client.client_in_region("eu-west-1");
        assert_eq!(remote.config().region().unwrap().as_ref(), "eu-west-1");
        assert_eq!(client.instance_region().unwrap(), "us-east-1");
    }

    #[tokio::test]
    async fn rejected_accounts_cannot_resolve_references_through_native_or_guest_route() {
        for (list, cfg) in [
            (
                "allowed_account_ids",
                AwsccProviderConfig {
                    allowed_account_ids: vec!["000000000000".into()],
                    ..Default::default()
                },
            ),
            (
                "forbidden_account_ids",
                AwsccProviderConfig {
                    forbidden_account_ids: vec!["123456789012".into()],
                    ..Default::default()
                },
            ),
        ] {
            let calls = Arc::new(AtomicUsize::new(0));
            let mock = MockAws::builder()
                .with_service(IdentityAndKms(calls.clone()))
                .build();
            let config = aws_config::SdkConfig::builder()
                .region(aws_config::Region::new("us-east-1"))
                .credentials_provider(mock.credentials_provider())
                .http_client(mock.http_client())
                .build();
            let provider =
                AwsccProvider::from_sdk_config(BoundedSdkConfig::for_test(config), &cfg).await;
            assert!(provider.init_error().unwrap().contains(list));
            let results = provider
                .resolve_references(vec![request("alias/customer"), request("alias/other")])
                .await;
            assert_eq!(results.len(), 2);
            for result in results {
                let Err(ReferenceError::InvalidReference(error)) = result else {
                    panic!("rejected account must return its initialization error")
                };
                assert!(error.message.contains(list));
            }
            // The native Provider trait and WASM Guest delegate to the same method.
            use carina_core::provider::Provider;
            let core_request = carina_core::references::ReferenceRequest {
                ty: carina_aws_types::AwsType::KmsKeyReference.identity(),
                value: "alias/customer".into(),
            };
            let results = Provider::resolve_references(&provider, &[core_request])
                .await
                .unwrap();
            assert!(
                matches!(&results[0], Err(carina_core::references::ReferenceError::InvalidReference(error)) if error.to_string().contains(list))
            );
            assert_eq!(
                calls.load(Ordering::SeqCst),
                0,
                "account guard must precede DescribeKey"
            );
        }
    }

    #[tokio::test]
    async fn missing_region_errors_without_describe_key_or_region_fallback() {
        let calls = Arc::new(AtomicUsize::new(0));
        let mock = MockAws::builder()
            .with_service(IdentityAndKms(calls.clone()))
            .build();
        let config = aws_config::SdkConfig::builder()
            .credentials_provider(mock.credentials_provider())
            .http_client(mock.http_client())
            .build();
        let provider =
            AwsccProvider::from_sdk_config(BoundedSdkConfig::for_test(config), &Default::default())
                .await;
        let results = provider
            .resolve_references(vec![
                request("alias/customer"),
                request("arn:aws:kms:eu-west-1:123456789012:alias/customer"),
            ])
            .await;
        for result in results {
            let Err(ReferenceError::InvalidReference(error)) = result else {
                panic!("a missing instance region must fail instead of selecting a default")
            };
            assert!(error.message.to_ascii_lowercase().contains("region"));
        }
        assert_eq!(calls.load(Ordering::SeqCst), 0);
    }
}
