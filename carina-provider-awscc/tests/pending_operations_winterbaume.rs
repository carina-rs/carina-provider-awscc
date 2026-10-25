//! Host-driven CloudControl step coverage against winterbaume.

#![cfg(not(target_arch = "wasm32"))]

mod common;

use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Mutex};

use aws_config::{BehaviorVersion, Region};
use carina_core::provider::{CreateOutcomeKind, DeleteRequest, Provider};
use carina_core::resource::{ConcreteValue, Resource, Value};
use carina_provider_awscc::provider::AwsccProviderConfig;
use carina_provider_awscc::{
    AwsccProvider, CcCreateToken, CcDeleteToken, CcRequestPhase, CcUpdateToken, RequestToken,
};
use carina_provider_protocol::OperationStep;
use serde::{Serialize, de::DeserializeOwned};
use winterbaume_cloudcontrol::CloudControlService;
use winterbaume_core::{MockAws, MockRequest, MockResponse, MockService, StatefulService};

const ACCOUNT_ID: &str = "123456789012";
const REGION: &str = "us-east-1";
const TYPE_NAME: &str = "AWS::ECR::Repository";
const REPOSITORY_NAME: &str = "pending-operation-test";
const TEST_CLIENT_TOKEN: &str = "00000000-0000-4000-8000-000000000001";

struct CountingCloudControl {
    inner: CloudControlService,
    calls: Mutex<HashMap<String, usize>>,
}

impl CountingCloudControl {
    fn new() -> Self {
        Self {
            inner: CloudControlService::new(),
            calls: Mutex::new(HashMap::new()),
        }
    }

    fn calls(&self, action: &str) -> usize {
        self.calls
            .lock()
            .expect("CloudControl call-count lock")
            .get(action)
            .copied()
            .unwrap_or(0)
    }

    async fn stored_identifier(&self) -> String {
        let view = self.inner.snapshot(ACCOUNT_ID, REGION).await;
        let resources = view
            .resources
            .values()
            .filter(|resource| resource.type_name == TYPE_NAME)
            .collect::<Vec<_>>();
        assert_eq!(resources.len(), 1);
        resources[0].identifier.clone()
    }
}

impl MockService for CountingCloudControl {
    fn service_name(&self) -> &str {
        self.inner.service_name()
    }

    fn url_patterns(&self) -> Vec<&str> {
        self.inner.url_patterns()
    }

    fn handle(
        &self,
        request: MockRequest,
    ) -> Pin<Box<dyn Future<Output = MockResponse> + Send + '_>> {
        let action = request
            .headers
            .get("x-amz-target")
            .and_then(|value| value.to_str().ok())
            .and_then(|value| value.split('.').next_back())
            .expect("CloudControl request should carry x-amz-target")
            .to_string();
        *self
            .calls
            .lock()
            .expect("CloudControl call-count lock")
            .entry(action)
            .or_default() += 1;
        self.inner.handle(request)
    }
}

fn string(value: &str) -> Value {
    Value::Concrete(ConcreteValue::String(value.to_string().into()))
}

fn repository_resource() -> Resource {
    Resource::with_provider("awscc", "ecr.Repository", "pending", None)
        .with_attribute("repository_name", string(REPOSITORY_NAME))
}

async fn provider() -> (AwsccProvider, Arc<CountingCloudControl>) {
    let cloudcontrol = Arc::new(CountingCloudControl::new());
    let mock = MockAws::builder()
        .with_service(Arc::clone(&cloudcontrol))
        .build();
    let config = aws_config::defaults(BehaviorVersion::latest())
        .http_client(mock.http_client())
        .credentials_provider(mock.credentials_provider())
        .region(Region::new(REGION))
        .load()
        .await;
    (
        AwsccProvider::from_sdk_config(config, &AwsccProviderConfig::default()).await,
        cloudcontrol,
    )
}

fn assert_token_round_trip<T>(token: T)
where
    T: Serialize + DeserializeOwned + std::fmt::Debug + PartialEq,
{
    let json = serde_json::to_string(&token).expect("serialize pending token");
    assert_eq!(
        serde_json::from_str::<T>(&json).expect("deserialize pending token"),
        token
    );
}

#[test]
fn continuation_tokens_round_trip() {
    assert_token_round_trip(CcCreateToken::NotIssued {
        attempt: 1,
        client_token: TEST_CLIENT_TOKEN.to_string(),
    });
    assert_token_round_trip(CcCreateToken::InFlight {
        request: RequestToken::from("create-request"),
        attempt: 2,
        identifier: Some(REPOSITORY_NAME.to_string()),
        client_token: "00000000-0000-4000-8000-000000000002".to_string(),
    });
    assert_token_round_trip(CcUpdateToken::NotIssued { attempt: 3 });
    assert_token_round_trip(CcUpdateToken::InFlight {
        request: RequestToken::from("update-request"),
        attempt: 4,
    });
    assert_token_round_trip(CcDeleteToken::DetachIgw(CcRequestPhase::InFlight {
        request: RequestToken::from("detach-request"),
        attempt: 5,
    }));
    assert_token_round_trip(CcDeleteToken::Delete(CcRequestPhase::NotIssued {
        attempt: 6,
    }));
    assert_token_round_trip(CcDeleteToken::Delete(CcRequestPhase::InFlight {
        request: RequestToken::from("delete-request"),
        attempt: 7,
    }));
}

#[test]
fn create_continuation_debug_omits_client_token() {
    let debug = format!(
        "{:?}",
        CcCreateToken::NotIssued {
            attempt: 1,
            client_token: TEST_CLIENT_TOKEN.to_string(),
        }
    );

    assert_eq!(debug, "NotIssued { attempt: 1 }");
    assert!(!debug.contains(TEST_CLIENT_TOKEN));
}

#[tokio::test(start_paused = true)]
async fn create_dispatch_and_delete_steps_make_one_issue_and_one_status_call_each() {
    let (provider, cloudcontrol) = provider().await;
    let resource = repository_resource();
    let id = resource.id.clone();
    let request = common::create_request(resource).await;

    let created = match carina_core::executor::dispatch_create(&provider, &id, request)
        .await
        .expect("dispatch create")
        .into_kind()
    {
        CreateOutcomeKind::Success { state } => state,
        other => panic!("create poll must settle, got {other:?}"),
    };
    assert_eq!(cloudcontrol.calls("CreateResource"), 1);
    assert_eq!(cloudcontrol.calls("GetResourceRequestStatus"), 1);
    assert_eq!(cloudcontrol.calls("GetResource"), 1);

    let stored_identifier = common::applied_identifier(cloudcontrol.stored_identifier().await);
    let delete_token = match provider
        .delete_step(&id, &stored_identifier, &DeleteRequest::default())
        .await
        .expect("issue delete")
    {
        OperationStep::Pending {
            pending,
            poll_after_secs,
        } => {
            assert_eq!(poll_after_secs, 1);
            pending
        }
        OperationStep::Done(()) => panic!("delete must defer"),
        OperationStep::Verdict(error) => panic!("delete failed: {error}"),
    };
    assert_eq!(created.identifier().as_str(), REPOSITORY_NAME);
    assert_eq!(cloudcontrol.calls("DeleteResource"), 1);

    assert!(matches!(
        provider
            .poll_delete_step(
                &id,
                &stored_identifier,
                &DeleteRequest::default(),
                delete_token,
            )
            .await
            .expect("poll delete"),
        OperationStep::Done(())
    ));
    assert_eq!(cloudcontrol.calls("DeleteResource"), 1);
    assert_eq!(cloudcontrol.calls("GetResourceRequestStatus"), 2);
}

#[tokio::test(start_paused = true)]
async fn native_provider_drives_create_and_delete_to_completion() {
    let (provider, cloudcontrol) = provider().await;
    let resource = repository_resource();
    let id = resource.id.clone();
    let request = common::create_request(resource).await;

    let created = carina_core::executor::complete_create_for_writeback(
        carina_core::executor::dispatch_create(&provider, &id, request)
            .await
            .expect("native create should drive pending steps"),
    )
    .0;
    assert_eq!(created.identifier.as_deref(), Some(REPOSITORY_NAME));
    assert_eq!(cloudcontrol.calls("CreateResource"), 1);
    assert_eq!(cloudcontrol.calls("GetResourceRequestStatus"), 1);

    let stored_identifier = common::applied_identifier(cloudcontrol.stored_identifier().await);
    Provider::delete(&provider, &id, &stored_identifier, DeleteRequest::default())
        .await
        .expect("native delete should drive pending steps");
    assert_eq!(cloudcontrol.calls("DeleteResource"), 1);
    assert_eq!(cloudcontrol.calls("GetResourceRequestStatus"), 2);
}
