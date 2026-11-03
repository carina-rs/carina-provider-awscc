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

    // Inject a recoverable transport failure at the real adapters. SDK dispatch
    // still owns the sealed handles, phase checks, and resulting wire verdicts.
    pub(super) struct TransientPollFailure;

    #[allow(unused_variables)]
    impl CarinaProvider for TransientPollFailure {
        type CreateToken = CcCreateToken;
        type UpdateToken = CcUpdateToken;
        type DeleteToken = CcDeleteToken;
        fn info(&self) -> proto::ProviderInfo {
            unreachable!("this fixture only polls read-back continuations")
        }
        fn schemas(&self) -> Vec<ValidatedResourceSchema> {
            unreachable!("this fixture only polls read-back continuations")
        }
        fn reference_forms(&self) -> ReferenceForms {
            unreachable!("this fixture only polls read-back continuations")
        }
        fn assignability_edges(&self) -> Vec<AssignabilityEdge> {
            schemas::assignability_edges()
        }
        fn provider_config_attribute_types(&self) -> ValidatedConfigSchema {
            unreachable!("this fixture only polls read-back continuations")
        }
        fn validate_config(&self, attrs: &BTreeMap<String, proto::Value>) -> Result<(), String> {
            unreachable!("this fixture only polls read-back continuations")
        }
        fn initialize(
            &mut self,
            attrs: &BTreeMap<String, proto::Value>,
        ) -> Result<proto::ReferenceScope, String> {
            unreachable!("this fixture only polls read-back continuations")
        }
        fn read(
            &self,
            id: &proto::ResourceId,
            identifier: &proto::AppliedIdentifier,
            _request: proto::ReadRequest,
        ) -> Result<proto::ReadOutcome, proto::ProviderError> {
            unreachable!("this fixture only polls read-back continuations")
        }
        fn read_data_source(
            &self,
            resource: &proto::Resource,
        ) -> Result<proto::State, proto::ProviderError> {
            unreachable!("this fixture only polls read-back continuations")
        }
        fn create(
            &self,
            _id: &proto::ResourceId,
            request: proto::CreateRequest,
            attempt: SdkCreateAttempt,
        ) -> Result<SdkCreateStep<Self::CreateToken>, SdkCreateVerdict<Self::CreateToken>> {
            unreachable!("this fixture only polls read-back continuations")
        }
        fn poll_create(
            &self,
            _id: &proto::ResourceId,
            request: proto::CreateRequest,
            poll: SdkPoll<Self::CreateToken>,
            poll_request: PollRequest,
        ) -> Result<SdkCreateStep<Self::CreateToken>, SdkPollVerdict<Self::CreateToken>> {
            let SdkPoll::Issued(issued, token) = poll else {
                unreachable!()
            };
            Err(GuestPolledIssuedCreate(issued).preserve(
                token,
                CoreProviderError::timeout("temporary polling failure"),
            ))
        }
        fn resume_create(
            &self,
            _id: &proto::ResourceId,
            request: proto::CreateRequest,
            resume: SdkResume<Self::CreateToken>,
            poll_request: PollRequest,
        ) -> Result<SdkResumeStep<Self::CreateToken>, SdkResumeVerdict<Self::CreateToken>> {
            let SdkResume::Issued(resumed, token) = resume else {
                unreachable!()
            };
            Err(GuestResumedCreate(resumed).preserve(
                token,
                CoreProviderError::timeout("temporary polling failure"),
            ))
        }
        fn update(
            &self,
            id: &proto::ResourceId,
            identifier: &proto::AppliedIdentifier,
            request: proto::UpdateRequest,
        ) -> Result<SdkUpdateOutcome<Self::UpdateToken>, proto::ProviderError> {
            unreachable!("this fixture only polls read-back continuations")
        }
        fn poll_update(
            &self,
            id: &proto::ResourceId,
            identifier: &proto::AppliedIdentifier,
            request: proto::UpdateRequest,
            pending: Pending<Self::UpdateToken>,
            poll_request: PollRequest,
        ) -> Result<SdkUpdateOutcome<Self::UpdateToken>, proto::ProviderError> {
            unreachable!("this fixture only polls read-back continuations")
        }
        fn delete(
            &self,
            id: &proto::ResourceId,
            identifier: &proto::AppliedIdentifier,
            request: proto::DeleteRequest,
        ) -> Result<SdkDeleteOutcome<Self::DeleteToken>, proto::ProviderError> {
            unreachable!("this fixture only polls read-back continuations")
        }
        fn poll_delete(
            &self,
            id: &proto::ResourceId,
            identifier: &proto::AppliedIdentifier,
            request: proto::DeleteRequest,
            pending: PendingDelete<Self::DeleteToken>,
        ) -> Result<SdkDeleteOutcome<Self::DeleteToken>, proto::ProviderError> {
            unreachable!("this fixture only polls read-back continuations")
        }
        fn required_permissions(
            &self,
            id: &proto::ResourceId,
            op: carina_plugin_sdk::PlanOp,
        ) -> Result<Vec<String>, proto::ProviderError> {
            unreachable!("this fixture only polls read-back continuations")
        }
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
        pending.progress.clone(),
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
        pending.progress.clone(),
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
        resumed.progress.clone(),
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
        pending.progress.clone(),
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
    assert_eq!(pending.phase(), PendingPhase::ReadBack);
    let UpdateOutcome::Done(outcome) = provider
        .poll_update(
            &id,
            &proto::AppliedIdentifier::new(VPC).unwrap(),
            update,
            Pending::new(pending.token, pending.poll_after_secs),
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

#[test]
fn guest_read_back_poll_transient_error_preserves_continuation() {
    let provider = guest::TransientPollFailure;
    let id = proto::ResourceId {
        provider: "awscc".into(),
        resource_type: "ec2.Vpc".into(),
        identity: "guest".into(),
    };
    let request = proto::CreateRequest {
        resource: proto::Resource {
            id: id.clone(),
            attributes: BTreeMap::new(),
            directives: Default::default(),
        },
    };
    let identifier = proto::AppliedIdentifier::new(VPC).unwrap();
    let token: carina_provider_awscc::CcCreateToken = serde_json::from_value(
        json!({"phase":"read_back", "identifier":VPC, "resource_type":"ec2.Vpc"}),
    )
    .unwrap();
    let sent = proto::Sent::only(["cidr_block"]);
    let progress = proto::CreateProgress::ReadingBack {
        identifier: identifier.clone(),
    };
    let poll_request = PollRequest {
        finality: PollFinality::Continue,
    };
    let error = carina_plugin_sdk::dispatch_poll_create(
        &provider,
        &id,
        request.clone(),
        progress.clone(),
        sent.clone(),
        token.clone(),
        poll_request,
    )
    .unwrap_err();
    let proto::PollVerdict::InFlight(in_flight) = *error else {
        panic!("a transient read-back poll failure must keep the continuation: {error:?}")
    };
    assert_eq!(in_flight.pending.progress, progress);
    assert_eq!(in_flight.pending.token, token);
    assert_eq!(in_flight.pending.sent, sent);
    assert_eq!(in_flight.pending.poll_after_secs, 5);
    assert_eq!(in_flight.error.kind, proto::ProviderErrorKind::Timeout);
    assert_eq!(in_flight.error.message, "temporary polling failure");

    // Persisted continuations initially arrive with operation progress. The
    // same read-back token must advance it even when the poll reports an error.
    for host_progress in [
        progress.clone(),
        proto::CreateProgress::Operating {
            identifier: Some(identifier),
        },
    ] {
        let error = carina_plugin_sdk::dispatch_resume_create(
            &provider,
            &id,
            request.clone(),
            host_progress,
            token.clone(),
            poll_request,
        )
        .unwrap_err();
        let proto::ResumeVerdict::InFlight(in_flight) = *error else {
            panic!("a transient resumed read-back failure must keep the continuation: {error:?}")
        };
        assert_eq!(in_flight.pending.progress, progress);
        assert_eq!(in_flight.pending.token, token);
        assert_eq!(in_flight.pending.poll_after_secs, 5);
        assert_eq!(in_flight.error.kind, proto::ProviderErrorKind::Timeout);
        assert_eq!(in_flight.error.message, "temporary polling failure");
    }
}
