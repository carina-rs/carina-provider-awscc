//! Provider-level retry regressions using the real CloudControl SDK and resend guard.

#![cfg(not(target_arch = "wasm32"))]

mod common;

use std::collections::VecDeque;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use aws_config::Region;
use aws_credential_types::Credentials;
use aws_credential_types::provider::error::CredentialsError;
use aws_credential_types::provider::{ProvideCredentials, SharedCredentialsProvider};
use aws_smithy_runtime_api::client::http::{
    HttpConnector, HttpConnectorFuture, SharedHttpConnector, http_client_fn,
};
use aws_smithy_runtime_api::client::orchestrator::HttpRequest;
use aws_smithy_runtime_api::client::result::ConnectorError;
use carina_core::provider::CreateFailureKind;
use carina_core::resource::{ConcreteValue, Resource, Value};
use carina_plugin_sdk::wasi_http::{SendPhase, transport_connector_error_for_test};
use carina_provider_awscc::provider::AwsccProviderConfig;
use carina_provider_awscc::{AwsccProvider, BoundedSdkConfig};

#[derive(Debug)]
struct ExpiringCredentials {
    successful_loads: usize,
    calls: Arc<AtomicUsize>,
}

impl ProvideCredentials for ExpiringCredentials {
    fn provide_credentials<'a>(
        &'a self,
    ) -> aws_credential_types::provider::future::ProvideCredentials<'a>
    where
        Self: 'a,
    {
        let call = self.calls.fetch_add(1, Ordering::SeqCst);
        let result = if call < self.successful_loads {
            Ok(Credentials::new(
                "test-key",
                "test-secret",
                None,
                None,
                "test",
            ))
        } else {
            Err(CredentialsError::not_loaded(std::io::Error::other(
                "test credentials unavailable",
            )))
        };
        aws_credential_types::provider::future::ProvideCredentials::ready(result)
    }
}

#[derive(Debug)]
enum Failure {
    MaybeSent,
    Timeout,
}

#[derive(Debug, Default)]
struct Script {
    failures: VecDeque<Failure>,
    client_tokens: Vec<String>,
}

#[derive(Clone, Debug)]
struct Connector(Arc<Mutex<Script>>);

impl HttpConnector for Connector {
    fn call(&self, request: HttpRequest) -> HttpConnectorFuture {
        let mut script = self.0.lock().expect("script lock");
        let body: serde_json::Value =
            serde_json::from_slice(request.body().bytes().expect("request bytes"))
                .expect("CloudControl request JSON");
        script.client_tokens.push(
            body["ClientToken"]
                .as_str()
                .expect("explicit ClientToken")
                .to_string(),
        );
        let failure = script.failures.pop_front().expect("unexpected HTTP call");
        HttpConnectorFuture::ready(Err(match failure {
            Failure::MaybeSent => transport_connector_error_for_test(SendPhase::MaybeSent),
            Failure::Timeout => {
                ConnectorError::timeout(std::io::Error::other("post-send timeout").into())
            }
        }))
    }
}

async fn provider(
    successful_loads: usize,
    failures: impl IntoIterator<Item = Failure>,
) -> (AwsccProvider, Arc<AtomicUsize>, Arc<Mutex<Script>>) {
    let calls = Arc::new(AtomicUsize::new(0));
    let script = Arc::new(Mutex::new(Script {
        failures: failures.into_iter().collect(),
        ..Script::default()
    }));
    let connector = Connector(Arc::clone(&script));
    let config = aws_config::SdkConfig::builder()
        .region(Region::new("us-east-1"))
        .credentials_provider(SharedCredentialsProvider::new(ExpiringCredentials {
            successful_loads,
            calls: Arc::clone(&calls),
        }))
        .identity_cache(aws_sdk_cloudcontrol::config::IdentityCache::no_cache())
        .http_client(http_client_fn(move |_, _| {
            SharedHttpConnector::new(connector.clone())
        }))
        .sleep_impl(aws_smithy_async::rt::sleep::TokioSleep::new())
        .build();
    let provider = AwsccProvider::from_sdk_config(
        BoundedSdkConfig::new(config),
        &AwsccProviderConfig::default(),
    )
    .await;
    (provider, calls, script)
}

fn vpc() -> Resource {
    let mut resource = Resource::with_provider("awscc", "ec2.Vpc", "test", None);
    resource.set_attr(
        "cidr_block".to_string(),
        Value::Concrete(ConcreteValue::String("10.0.0.0/16".into())),
    );
    resource
}

#[tokio::test(start_paused = true)]
async fn credentials_not_loaded_fails_promptly_without_resends_or_http_calls() {
    let (provider, calls, script) = provider(0, []).await;
    let resource = vpc();
    let id = resource.id.clone();
    let request = common::create_request(resource).await;

    let failure = tokio::time::timeout(
        Duration::from_secs(5),
        common::dispatch_create(&provider, &id, request),
    )
    .await
    .expect("missing credentials must not enter the provider resend loop")
    .expect_err("create without credentials must fail");
    let CreateFailureKind::Untracked(error) = failure.into_kind() else {
        panic!("a create that never reached transmit must fail as untracked");
    };
    assert!(error.to_string().contains("test credentials unavailable"));
    assert_eq!(calls.load(Ordering::SeqCst), 1);
    assert!(script.lock().expect("script lock").client_tokens.is_empty());
}

#[tokio::test(start_paused = true)]
async fn permanent_not_sent_resend_preserves_the_earlier_maybe_sent_create() {
    assert_preserved_promptly(1, [Failure::MaybeSent], 20).await;
}

#[tokio::test(start_paused = true)]
async fn maybe_sent_then_permanent_not_sent_in_initial_sdk_call_preserves_create_promptly() {
    assert_preserved_promptly(1, [Failure::Timeout], 5).await;
}

#[tokio::test(start_paused = true)]
async fn maybe_sent_then_permanent_not_sent_on_resend_preserves_create_promptly() {
    // The first provider send becomes unconfirmed. Its resend reaches transmit,
    // then loses credentials on the SDK's second attempt. That resend still has
    // an aggregate MaybeSent outcome, but another resend cannot succeed.
    assert_preserved_promptly(2, [Failure::MaybeSent, Failure::Timeout], 20).await;
}

async fn assert_preserved_promptly(
    successful_loads: usize,
    failures: impl IntoIterator<Item = Failure>,
    deadline_secs: u64,
) {
    let (provider, calls, script) = provider(successful_loads, failures).await;
    let resource = vpc();
    let id = resource.id.clone();
    let request = common::create_request(resource).await;

    let failure = tokio::time::timeout(
        Duration::from_secs(deadline_secs),
        common::dispatch_create(&provider, &id, request),
    )
    .await
    .expect("permanent final pre-send failure must stop provider resends promptly")
    .expect_err("a possibly applied create must remain in flight");
    let CreateFailureKind::InFlight { pending, error } = failure.into_kind() else {
        panic!("the earlier possible send must never be forgotten");
    };
    assert!(error.to_string().contains("test credentials unavailable"));
    let token: serde_json::Value =
        serde_json::from_str(pending.token().expose_for_provider_call()).expect("continuation");
    assert_eq!(token["phase"], "unconfirmed");
    assert_eq!(token["attempt"], 0);
    assert_eq!(token["resends"], 1);
    let script = script.lock().expect("script lock");
    assert_eq!(script.client_tokens.len(), successful_loads);
    assert!(
        script
            .client_tokens
            .iter()
            .all(|t| t == &script.client_tokens[0])
    );
    assert_eq!(token["client_token"], script.client_tokens[0]);
    assert_eq!(calls.load(Ordering::SeqCst), successful_loads + 1);
}
