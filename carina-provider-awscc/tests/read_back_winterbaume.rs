//! Native driver and executor read-back coverage using in-process CloudControl.
#![cfg(not(target_arch = "wasm32"))]
#![recursion_limit = "256"]

use std::collections::HashMap;
use std::future::ready;
use std::num::NonZeroUsize;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use carina_core::binding_index::{BindingValueSource, ResolvedBindings};
use carina_core::data_source_value::ExecutableDataSourceValues;
use carina_core::effect::{Effect, PlanOp};
use carina_core::executor::ProviderCheckInputs;
use carina_core::executor::{
    self, Applied, ExecutionCheckpointSink, ExecutionEvent, ExecutionObserver, ExecutionOutcome,
    ExecutionResult, LiveExecutionInput,
};
use carina_core::plan::Plan;
use carina_core::provider::{
    AppliedIdentifier, BoxFuture, CreateAttempt, CreateFailure, CreateOutcome, CreateRequest,
    DeleteRequest, Provider, ProviderReadyDataSource, ProviderResult, ReadBackEnd, ReadOutcome,
    ReadRequest, ResumeCreateRequest, ResumeFailure, ResumeOutcome, Sent, UpdateOutcome,
    UpdateRequest,
};
use carina_core::resource::{
    ConcreteValue, Current, ResolvedResourceId, Resource, ResourceId, State, Value,
};
use carina_core::schema::{AttributeSchema, AttributeType, ResourceSchema, SchemaRegistry};
use carina_core::shutdown::ShutdownToken;
use carina_provider_awscc::{
    AwsccNormalizer, AwsccProvider, AwsccProviderFactory, BoundedSdkConfig,
};
use carina_provider_protocol::types::{OperationConfig, ReadBackBudget};
use serde_json::json;
use std::collections::VecDeque;
use std::future::Future;
use std::pin::Pin;
use winterbaume_core::{MockAws, MockRequest, MockResponse, MockService};

const ROLE: &str = "read-back-role";
const ROLE_ID: &str = "AROAREADBACKTEST000001";
const ARN: &str = "arn:aws:iam::123456789012:role/read-back-role";

struct CloudControlService {
    populated: Mutex<VecDeque<bool>>,
    requests: AtomicUsize,
    reads: AtomicUsize,
    statuses: AtomicUsize,
}
impl MockService for CloudControlService {
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
        let body = match action {
            "CreateResource" => {
                self.requests.fetch_add(1, Ordering::Relaxed);
                json!({"ProgressEvent":{"OperationStatus":"IN_PROGRESS", "RequestToken":"read-back-create", "Identifier":ROLE}})
            }
            "GetResourceRequestStatus" => {
                self.statuses.fetch_add(1, Ordering::Relaxed);
                json!({"ProgressEvent":{"OperationStatus":"SUCCESS", "RequestToken":"read-back-create", "Identifier":ROLE}})
            }
            "GetResource" => {
                self.reads.fetch_add(1, Ordering::Relaxed);
                let body: serde_json::Value = serde_json::from_slice(&request.body).unwrap();
                assert_eq!(body["Identifier"], ROLE);
                let mut script = self.populated.lock().unwrap();
                let populated = if script.len() > 1 {
                    script.pop_front().unwrap()
                } else {
                    *script.front().unwrap()
                };
                let props = if populated {
                    json!({"RoleName":ROLE, "RoleId":ROLE_ID, "Arn":ARN})
                } else {
                    json!({"RoleId":ROLE_ID})
                };
                json!({"ResourceDescription":{"Identifier":ROLE, "Properties":props.to_string()}})
            }
            other => panic!("unexpected CloudControl operation {other}"),
        };
        Box::pin(ready(MockResponse::json(200, body.to_string())))
    }
}

/// Only the dependent sink is synthetic. CloudControl and polling use the
/// production provider; the executor prepares and resolves every request.
struct FixtureProvider {
    aws: AwsccProvider,
    sleeps: Mutex<Vec<Duration>>,
    ends: Mutex<Vec<ReadBackEnd>>,
    sinks: Mutex<HashMap<String, Value>>,
}
impl Provider for FixtureProvider {
    fn name(&self) -> &str {
        "awscc"
    }
    fn read(
        &self,
        id: &ResourceId,
        identifier: &AppliedIdentifier,
        request: ReadRequest,
    ) -> BoxFuture<'_, ProviderResult<ReadOutcome>> {
        self.aws.read(id, identifier, request)
    }
    fn read_data_source(
        &self,
        resource: &ProviderReadyDataSource,
    ) -> BoxFuture<'_, ProviderResult<State>> {
        self.aws.read_data_source(resource)
    }
    fn create(
        &self,
        id: &ResourceId,
        request: CreateRequest,
        attempt: CreateAttempt,
    ) -> BoxFuture<'_, Result<CreateOutcome, CreateFailure>> {
        let id = id.clone();
        Box::pin(async move {
            if id.resource_type.as_str() == "test.Dependent" {
                let attrs = request.resource().as_resource().resolved_attributes();
                self.sinks
                    .lock()
                    .unwrap()
                    .insert(id.identity().unwrap().to_string(), attrs["value"].clone());
                return Ok(attempt
                    .issued(
                        AppliedIdentifier::new(format!("sink-{}", id.identity().unwrap())).unwrap(),
                        Sent::AllRequestAttributes,
                    )
                    .succeeded(attrs));
            }
            let outcome = Box::pin(self.aws.create_with_poll_sleep_for_test(
                request.resource().as_resource(),
                attempt,
                |duration| {
                    self.sleeps.lock().unwrap().push(duration);
                    ready(())
                },
            ))
            .await?;
            self.ends.lock().unwrap().push(outcome.read_back_end());
            Ok(outcome)
        })
    }
    fn resume_create(
        &self,
        request: ResumeCreateRequest,
    ) -> BoxFuture<'_, Result<ResumeOutcome, ResumeFailure>> {
        self.aws.resume_create(request)
    }
    fn update(
        &self,
        id: &ResourceId,
        identifier: &AppliedIdentifier,
        request: UpdateRequest,
    ) -> BoxFuture<'_, ProviderResult<UpdateOutcome>> {
        self.aws.update(id, identifier, request)
    }
    fn delete(
        &self,
        id: &ResourceId,
        identifier: &AppliedIdentifier,
        request: DeleteRequest,
    ) -> BoxFuture<'_, ProviderResult<()>> {
        self.aws.delete(id, identifier, request)
    }
    fn required_permissions(&self, _id: &ResourceId, _op: PlanOp) -> ProviderResult<Vec<String>> {
        Ok(vec![])
    }
}

#[derive(Default)]
struct Observer(Mutex<Vec<String>>);
impl ExecutionObserver for Observer {
    fn on_event(&self, event: &ExecutionEvent<'_>) {
        if let ExecutionEvent::EffectFailed { failure, .. } = event {
            self.0.lock().unwrap().push(failure.to_string());
        }
    }
}

async fn fixture(
    populated: &[bool],
    read_back_seconds: u64,
) -> (FixtureProvider, Arc<CloudControlService>) {
    let service = Arc::new(CloudControlService {
        populated: Mutex::new(populated.iter().copied().collect()),
        requests: AtomicUsize::new(0),
        reads: AtomicUsize::new(0),
        statuses: AtomicUsize::new(0),
    });
    let mock = MockAws::builder()
        .with_service(Arc::clone(&service))
        .build();
    let config = BoundedSdkConfig::for_test(mock.sdk_config("us-east-1").await);
    let aws = AwsccProvider::from_sdk_config(
        config,
        &carina_provider_awscc::provider::AwsccProviderConfig::default(),
    )
    .await
    .with_operation_config_for_test(
        "iam.Role",
        OperationConfig {
            read_back: ReadBackBudget::from_secs(read_back_seconds),
            ..Default::default()
        },
    );
    (
        FixtureProvider {
            aws,
            sleeps: Mutex::new(vec![]),
            ends: Mutex::new(vec![]),
            sinks: Mutex::new(HashMap::new()),
        },
        service,
    )
}
fn string(value: &str) -> Value {
    Value::Concrete(ConcreteValue::String(value.into()))
}
fn role() -> Resource {
    let mut resource: Resource = Resource::with_provider("awscc", "iam.Role", "role", None);
    resource.binding = Some(carina_core::resource::absolute_binding_address_for_test(
        "role",
    ));
    resource.set_attr("role_name", string(ROLE));
    resource.set_attr(
        "assume_role_policy_document",
        Value::Concrete(ConcreteValue::Map(indexmap::IndexMap::from([
            (
                "version".to_string(),
                Value::Concrete(ConcreteValue::enum_identifier("2012_10_17")),
            ),
            (
                "statement".to_string(),
                Value::Concrete(ConcreteValue::List(vec![Value::Concrete(
                    ConcreteValue::Map(indexmap::IndexMap::from([
                        (
                            "effect".to_string(),
                            Value::Concrete(ConcreteValue::enum_identifier("allow")),
                        ),
                        ("action".to_string(), string("sts:AssumeRole")),
                        (
                            "principal".to_string(),
                            Value::Concrete(ConcreteValue::Map(indexmap::IndexMap::from([(
                                "service".to_string(),
                                string("ec2.amazonaws.com"),
                            )]))),
                        ),
                    ])),
                )])),
            ),
        ]))),
    );
    resource
}
fn dependent(name: &'static str, attribute: &str) -> Resource {
    let mut resource: Resource = Resource::with_provider("awscc", "test.Dependent", name, None);
    resource.binding = Some(carina_core::resource::absolute_binding_address_for_test(
        name,
    ));
    resource.dependencies.value_bindings.insert(
        carina_core::resource::absolute_binding_address_for_test("role"),
    );
    resource.set_attr(
        "value",
        Value::resource_ref("role", attribute, vec![]).resolve_root_for_test(),
    );
    resource
}
fn schemas() -> SchemaRegistry {
    let mut registry = carina_provider_awscc::schemas::schema_registry();
    registry.insert(
        "awscc",
        ResourceSchema::new("test.Dependent")
            .attribute(AttributeSchema::required("value", AttributeType::string()).build())
            .validate()
            .unwrap(),
    );
    registry
}
async fn execute(
    provider: &FixtureProvider,
    resources: Vec<Resource>,
    bindings: ResolvedBindings,
    current: HashMap<ResourceId, Current>,
    observer: &Observer,
) -> ExecutionResult {
    let mut plan = Plan::new();
    let schemas = schemas();
    // A refreshed role may be outside this plan while its dependents run.
    let mut graph_resources = resources.clone();
    let source = role();
    if !graph_resources
        .iter()
        .any(|resource| resource.id == source.id)
    {
        graph_resources.push(source);
    }
    let graph =
        carina_core::resolved_graph::ExpandedBindingIndex::for_test_resources(&graph_resources)
            .expect("test resource bindings should form a valid graph");
    let resource_origins = graph_resources
        .iter()
        .map(|resource| {
            (
                ResolvedResourceId::new(resource.id.clone()),
                executor::UnresolvedResource::from_pre_resolve(resource.clone()),
            )
        })
        .collect();
    for resource in resources {
        let resource = executor::normalized::apply_desired_normalization(
            resource,
            &[],
            &AwsccNormalizer,
            &schemas,
        )
        .await
        .expect("normalize fixture values before constructing the plan");
        let resource = graph
            .resolve_resource(resource.as_resource().clone(), &schemas)
            .expect("resolve fixture dependencies before constructing the plan");
        plan.add(Effect::Create(resource));
    }
    let factories: Vec<Box<dyn carina_core::provider::ProviderFactory>> =
        vec![Box::new(AwsccProviderFactory)];
    let input = LiveExecutionInput {
        provider_context: carina_core::parser::ProviderContext::clone(
            &carina_core::parser::LoadedProviderContext::empty_for_test(),
        ),
        plan: &plan,
        provider_check_inputs: ProviderCheckInputs::PlanNormalized {
            resource_origins: &resource_origins,
            data_source_origins: &[],
        },
        compositions: &[],
        bindings,
        managed_current: current,
        data_source_values: ExecutableDataSourceValues::empty(),
        normalizer: &AwsccNormalizer,
        provider_configs: &[],
        factories: &factories,
        schemas: &schemas,
        parallelism: NonZeroUsize::new(1).unwrap(),
        precompleted_success_count: 0,
        checkpoint_sink: ExecutionCheckpointSink::noop(),
    };
    match Box::pin(executor::execute_live_plan_for_test(
        provider,
        input,
        observer,
        ShutdownToken::running(),
    ))
    .await
    {
        ExecutionOutcome::Completed(result) => result,
        ExecutionOutcome::Cancelled(_) => panic!("test did not cancel"),
    }
}

#[tokio::test]
async fn read_back_completion_reaches_same_apply_dependent() {
    let (provider, service) = fixture(&[false, true], 120).await;
    let dependent = dependent("record", "arn");
    let observer = Observer::default();
    let result = execute(
        &provider,
        vec![role(), dependent],
        ResolvedBindings::default(),
        HashMap::new(),
        &observer,
    )
    .await;
    assert_eq!(
        result.failure_count(),
        0,
        "{:?}",
        observer.0.lock().unwrap()
    );
    assert_eq!(result.partial_count, 0);
    assert_eq!(result.success_count, 2);
    assert_eq!(provider.sinks.lock().unwrap()["record"], string(ARN));
    assert_eq!(service.requests.load(Ordering::Relaxed), 1);
    assert_eq!(service.reads.load(Ordering::Relaxed), 2);
    assert_eq!(
        *provider.sleeps.lock().unwrap(),
        [Duration::from_secs(1), Duration::from_secs(5)]
    );
    assert_eq!(service.statuses.load(Ordering::Relaxed), 1);
}

#[tokio::test]
async fn exhausted_read_back_is_retried_by_next_apply() {
    let (provider, service) = fixture(&[false, false, true], 1).await;
    let source = role();
    let id = source.id.clone();
    let observer = Observer::default();
    let result = execute(
        &provider,
        vec![
            source,
            dependent("present", "role_id"),
            dependent("missing", "arn"),
        ],
        ResolvedBindings::default(),
        HashMap::new(),
        &observer,
    )
    .await;
    assert_eq!(result.partial_count, 1);
    assert_eq!(result.success_count, 1);
    assert_eq!(
        result.failure_count(),
        1,
        "{:?}",
        observer.0.lock().unwrap()
    );
    let Applied::Complete { state, .. } = &result.applied_states[&id] else {
        panic!("partial create must retain applied state")
    };
    assert_eq!(state.identifier.as_deref(), Some(ROLE));
    let marker = state
        .partial_read
        .as_ref()
        .expect("partial marker must persist");
    assert_eq!(
        marker
            .missing_attributes()
            .iter()
            .map(String::as_str)
            .collect::<Vec<_>>(),
        ["arn"]
    );
    let errors = observer.0.lock().unwrap().join("\n");
    assert!(errors.contains("configured budget"), "{errors}");
    assert!(!errors.contains("wait"), "{errors}");
    assert_eq!(*provider.ends.lock().unwrap(), [ReadBackEnd::HostDeadline]);

    {
        let sleeps = provider.sleeps.lock().unwrap();
        assert_eq!(sleeps.len(), 2);
        assert_eq!(sleeps[0], Duration::from_secs(1));
        assert!(sleeps[1] > Duration::ZERO && sleeps[1] <= Duration::from_secs(1));
    }
    assert_eq!(service.statuses.load(Ordering::Relaxed), 1);
    assert_eq!(service.reads.load(Ordering::Relaxed), 2);

    // The next apply refreshes the saved identifier before planning dependents.
    let fresh = provider
        .read(&id, &AppliedIdentifier::new(ROLE).unwrap(), ReadRequest)
        .await
        .unwrap()
        .into_state();
    assert!(fresh.partial_read.is_none());
    let mut bindings = ResolvedBindings::default();
    let arn = fresh
        .attributes
        .get("arn")
        .expect("refreshed role has an ARN")
        .to_value_at("role.arn")
        .expect("refreshed ARN is available");
    bindings.set(
        "role",
        HashMap::from([("arn".into(), arn)]),
        BindingValueSource::Local,
    );
    let next = execute(
        &provider,
        vec![dependent("missing", "arn")],
        bindings,
        HashMap::from([(id, Current::from_snapshot(fresh))]),
        &Observer::default(),
    )
    .await;
    assert_eq!(next.success_count, 1);
    assert_eq!(next.failure_count(), 0);
    assert_eq!(next.partial_count, 0);
    assert_eq!(provider.sinks.lock().unwrap()["missing"], string(ARN));
    assert_eq!(
        service.requests.load(Ordering::Relaxed),
        1,
        "refresh must not recreate the role"
    );
}
