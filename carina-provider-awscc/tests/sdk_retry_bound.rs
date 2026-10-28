//! Verifies the provider boundary caps AWS SDK retry attempts.

#![cfg(not(target_arch = "wasm32"))]

use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use aws_config::{BehaviorVersion, Region};
use aws_smithy_types::retry::RetryConfig;
use carina_provider_awscc::provider::AwsccProviderConfig;
use carina_provider_awscc::{AwsccProvider, BoundedSdkConfig};
use winterbaume_core::{MockAws, MockRequest, MockResponse, MockService};

const REGION: &str = "us-east-1";

#[derive(Default)]
struct AlwaysUnavailableSts {
    calls: AtomicUsize,
}

impl AlwaysUnavailableSts {
    fn calls(&self) -> usize {
        self.calls.load(Ordering::SeqCst)
    }
}

impl MockService for AlwaysUnavailableSts {
    fn service_name(&self) -> &str {
        "sts"
    }

    fn url_patterns(&self) -> Vec<&str> {
        vec![
            r"https?://sts\..*\.amazonaws\.com",
            r"https?://sts\.amazonaws\.com",
        ]
    }

    fn handle(
        &self,
        _request: MockRequest,
    ) -> Pin<Box<dyn Future<Output = MockResponse> + Send + '_>> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        Box::pin(async {
            MockResponse::error(503, "ServiceUnavailable", "service temporarily unavailable")
        })
    }
}

#[tokio::test(start_paused = true)]
async fn provider_boundary_limits_sts_call_to_three_attempts() {
    let sts = Arc::new(AlwaysUnavailableSts::default());
    let mock = MockAws::builder().with_service(Arc::clone(&sts)).build();
    let input = aws_config::defaults(BehaviorVersion::latest())
        .http_client(mock.http_client())
        .credentials_provider(mock.credentials_provider())
        .region(Region::new(REGION))
        .retry_config(RetryConfig::standard().with_max_attempts(10))
        .load()
        .await;
    let config = BoundedSdkConfig::new(input);
    let provider = AwsccProvider::from_sdk_config(
        config,
        &AwsccProviderConfig {
            allowed_account_ids: vec!["123456789012".into()],
            ..Default::default()
        },
    )
    .await;

    assert!(provider.init_error().is_some());
    assert_eq!(sts.calls(), 3);
}
