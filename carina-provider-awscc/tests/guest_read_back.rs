//! Exercise the actual guest adapters through the SDK's sealed dispatch boundary.
#![cfg(not(target_arch = "wasm32"))]

use std::collections::BTreeMap;
use std::future::{Future, ready};
use std::pin::Pin;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use carina_plugin_sdk::{CarinaProvider, Pending, UpdateOutcome};
use carina_provider_awscc::{AwsccProvider, BoundedSdkConfig};
use carina_provider_protocol::{PendingPhase, PollFinality, PollRequest, types as proto};
use serde_json::json;
use winterbaume_core::{MockAws, MockRequest, MockResponse, MockService, json_error_response};

// Compile the guest entry point itself so these tests exercise its adapters and
// finality forwarding. Filter on guest_read_back_ when running this test target.
#[allow(dead_code)]
mod guest {
    pub(super) fn with_provider(
        provider: AwsccProvider,
    ) -> impl CarinaProvider<CreateToken = CcCreateToken, UpdateToken = CcUpdateToken> {
        let mut guest = AwsccProcessProvider::new();
        guest.provider = Some(provider);
        guest
    }

    include!("../src/main.rs");
}

const VPC: &str = "vpc-0123456789abcdef0";
#[derive(Default)]
struct MissingRead {
    creates: AtomicUsize,
    updates: AtomicUsize,
    statuses: AtomicUsize,
    reads: AtomicUsize,
}
impl MockService for MissingRead {
    fn service_name(&self) -> &str {
        "cloudcontrolapi"
    }
    fn url_patterns(&self) -> Vec<&str> {
        vec![r"^https://cloudcontrolapi\.[^/]+\.amazonaws\.com"]
    }
    fn handle(
        &self,
        request: MockRequest,
    ) -> Pin<Box<dyn Future<Output = MockResponse> + Send + '_>> {
        let action = request
            .headers
            .get("x-amz-target")
            .unwrap()
            .to_str()
            .unwrap()
            .split('.')
            .next_back()
            .unwrap();
        let status = match action {
            "CreateResource" => {
                self.creates.fetch_add(1, Ordering::Relaxed);
                "IN_PROGRESS"
            }
            "UpdateResource" => {
                self.updates.fetch_add(1, Ordering::Relaxed);
                "IN_PROGRESS"
            }
            "GetResourceRequestStatus" => {
                self.statuses.fetch_add(1, Ordering::Relaxed);
                "SUCCESS"
            }
            "GetResource" => {
                self.reads.fetch_add(1, Ordering::Relaxed);
                return Box::pin(ready(json_error_response(
                    404,
                    "ResourceNotFoundException",
                    "not yet visible",
                )));
            }
            other => panic!("unexpected operation {other}"),
        };
        Box::pin(ready(MockResponse::json(200, json!({"ProgressEvent":{"OperationStatus":status,"RequestToken":"guest-request","Identifier":VPC}}).to_string())))
    }
}

#[test]
fn guest_read_back_phase_and_finality_reach_create_resume_and_update() {
    std::thread::Builder::new()
        .stack_size(8 * 1024 * 1024)
        .spawn(run_guest_read_back)
        .unwrap()
        .join()
        .unwrap();
}

fn run_guest_read_back() {
    let service = Arc::new(MissingRead::default());
    let mock = MockAws::builder()
        .with_service(Arc::clone(&service))
        .build();
    let runtime = tokio::runtime::Runtime::new().unwrap();
    let provider = runtime.block_on(async {
        AwsccProvider::from_sdk_config(
            BoundedSdkConfig::for_test(mock.sdk_config("us-guest-read-back-1").await),
            &carina_provider_awscc::provider::AwsccProviderConfig::default(),
        )
        .await
    });
    drop(runtime);
    let provider = guest::with_provider(provider);
    let id = proto::ResourceId {
        provider: "awscc".into(),
        resource_type: "ec2.Vpc".into(),
        identity: "guest".into(),
    };
    let request = proto::CreateRequest {
        resource: proto::Resource {
            id: id.clone(),
            attributes: BTreeMap::from([(
                "cidr_block".into(),
                proto::Value::String("10.0.0.0/16".into()),
            )]),
            directives: Default::default(),
        },
    };
    let step = carina_plugin_sdk::dispatch_create(&provider, &id, request.clone()).unwrap();
    let proto::CreateStep::Pending(pending) = step else {
        panic!("expected operation pending")
    };
    assert_eq!(pending.progress.phase(), PendingPhase::Operation);
    let step = carina_plugin_sdk::dispatch_poll_create(
        &provider,
        &id,
        request.clone(),
        Some(proto::AppliedIdentifier::new(VPC).unwrap()),
        pending.sent.clone(),
        pending.token,
        PollRequest {
            finality: PollFinality::Continue,
        },
    )
    .unwrap();
    let proto::CreateStep::Pending(pending) = step else {
        panic!("expected read-back pending")
    };
    assert_eq!(pending.progress.phase(), PendingPhase::ReadBack);
    let step = carina_plugin_sdk::dispatch_resume_create(
        &provider,
        &id,
        request.clone(),
        Some(proto::AppliedIdentifier::new(VPC).unwrap()),
        pending.token.clone(),
        PollRequest {
            finality: PollFinality::Continue,
        },
    )
    .unwrap();
    let proto::ResumeStep::Pending(resumed) = step else {
        panic!("expected resumed read-back pending")
    };
    assert_eq!(resumed.progress.phase(), PendingPhase::ReadBack);
    let step = carina_plugin_sdk::dispatch_resume_create(
        &provider,
        &id,
        request.clone(),
        Some(proto::AppliedIdentifier::new(VPC).unwrap()),
        resumed.token,
        PollRequest {
            finality: PollFinality::FinalAttempt,
        },
    )
    .unwrap();
    assert!(matches!(step, proto::ResumeStep::PartialSuccess { .. }));
    let step = carina_plugin_sdk::dispatch_poll_create(
        &provider,
        &id,
        request.clone(),
        Some(proto::AppliedIdentifier::new(VPC).unwrap()),
        pending.sent,
        pending.token,
        PollRequest {
            finality: PollFinality::FinalAttempt,
        },
    )
    .unwrap();
    let proto::CreateStep::PartialSuccess { diagnostic, .. } = step else {
        panic!("expected final partial create")
    };
    assert!(
        diagnostic
            .missing_attributes
            .contains(&"vpc_id".to_string())
    );
    assert!(!diagnostic.missing_attributes.contains(&"state".to_string()));

    let from: proto::PersistedState =
        serde_json::from_value(json!({"id":id, "identifier":VPC, "attributes":{}, "exists":true}))
            .unwrap();
    let update = proto::UpdateRequest {
        from,
        patch: proto::UpdatePatch {
            ops: vec![proto::PatchOp::Add {
                key: "enable_dns_support".into(),
                value: proto::Value::Bool(true),
            }],
        },
    };
    let UpdateOutcome::Pending(pending) = provider
        .update(
            &id,
            &proto::AppliedIdentifier::new(VPC).unwrap(),
            update.clone(),
        )
        .unwrap()
    else {
        panic!("expected operation pending")
    };
    let UpdateOutcome::Pending(pending) = provider
        .poll_update(
            &id,
            &proto::AppliedIdentifier::new(VPC).unwrap(),
            update.clone(),
            pending,
            PollRequest {
                finality: PollFinality::Continue,
            },
        )
        .unwrap()
    else {
        panic!("expected read-back pending")
    };
    assert_eq!(pending.phase, PendingPhase::ReadBack);
    let UpdateOutcome::Done(outcome) = provider
        .poll_update(
            &id,
            &proto::AppliedIdentifier::new(VPC).unwrap(),
            update,
            Pending::read_back(pending.token, pending.poll_after_secs),
            PollRequest {
                finality: PollFinality::FinalAttempt,
            },
        )
        .unwrap()
    else {
        panic!("expected terminal update")
    };
    assert!(matches!(
        outcome,
        proto::UpdateOutcome::PartialSuccess { .. }
    ));
    assert_eq!(service.creates.load(Ordering::Relaxed), 1);
    assert_eq!(service.updates.load(Ordering::Relaxed), 1);
    assert_eq!(service.statuses.load(Ordering::Relaxed), 2);
    assert_eq!(service.reads.load(Ordering::Relaxed), 6);
}
