//! Integration coverage for SNS Topic through the provider's real
//! Provider-trait apply path against an in-process winterbaume CloudControl
//! mock. winterbaume-cloudcontrol 1.0.1 does not register a shaper for
//! `AWS::SNS::Topic`, so this test explicitly seeds Cloud Control's stored
//! read model with the inline `Subscription` aggregate observed from AWS. The
//! provider must drop that omitted property while preserving the native-object
//! ArchivePolicy read-back (including its numeric retention period), and the
//! resulting state must produce no diff against the desired Topic.

mod common;

use std::collections::HashMap;
use std::sync::Arc;

use carina_core::differ::{Diff, diff};
use carina_core::provider::{
    CreateOutcomeKind, Provider, RawSavedAttrs, ReadRequest, normalize_provider_states,
};
use carina_core::resource::{ConcreteValue, ResolvedResource, Resource, Value};
use carina_provider_awscc::AwsccProvider;
use carina_provider_awscc::provider::AwsccProviderConfig;
use carina_provider_awscc::schemas::generated::sns::topic::sns_topic_config;
use indexmap::IndexMap;
use winterbaume_cloudcontrol::CloudControlService;
use winterbaume_core::{MockAws, StatefulService};

const TYPE_NAME: &str = "AWS::SNS::Topic";
const ACCOUNT_ID: &str = "123456789012";
const REGION: &str = "ap-northeast-1";
const TOPIC_NAME: &str = "carina-registry-alerts.fifo";

fn string(value: &str) -> Value {
    Value::Concrete(ConcreteValue::String(value.to_string().into()))
}

fn bool_value(value: bool) -> Value {
    Value::Concrete(ConcreteValue::Bool(value))
}

fn int(value: i64) -> Value {
    Value::Concrete(ConcreteValue::Int(value))
}

fn map(entries: impl IntoIterator<Item = (&'static str, Value)>) -> Value {
    Value::Concrete(ConcreteValue::Map(
        entries
            .into_iter()
            .map(|(key, value)| (key.to_string(), value))
            .collect::<IndexMap<_, _>>(),
    ))
}

fn topic_resource_without_archive_policy() -> Resource {
    Resource::with_provider("awscc", "sns.Topic", "alerts", None)
        .with_attribute("topic_name", string(TOPIC_NAME))
        .with_attribute("fifo_topic", bool_value(true))
        .with_attribute("content_based_deduplication", bool_value(true))
        .with_attribute(
            "tags",
            map([
                ("Environment", string("test")),
                ("Project", string("carina-registry")),
            ]),
        )
}

fn topic_resource() -> Resource {
    topic_resource_without_archive_policy().with_attribute(
        "archive_policy",
        map([("message_retention_period", int(1))]),
    )
}

async fn winterbaume_provider() -> (AwsccProvider, Arc<CloudControlService>) {
    let cloudcontrol = Arc::new(CloudControlService::new());
    let mock = MockAws::builder()
        .with_service(Arc::clone(&cloudcontrol))
        .build();
    let config = mock.sdk_config(REGION).await;

    (
        AwsccProvider::from_sdk_config(
            carina_provider_awscc::BoundedSdkConfig::for_test(config),
            &AwsccProviderConfig::default(),
        )
        .await,
        cloudcontrol,
    )
}

async fn stored_topic(cloudcontrol: &CloudControlService) -> (String, serde_json::Value) {
    let view = cloudcontrol.snapshot(ACCOUNT_ID, REGION).await;
    let resources = view
        .resources
        .values()
        .filter(|resource| resource.type_name == TYPE_NAME)
        .collect::<Vec<_>>();
    assert_eq!(resources.len(), 1, "expected exactly one stored topic");
    let resource = resources[0];
    (
        resource.identifier.clone(),
        serde_json::from_str(&resource.resource_model).expect("stored topic model should be JSON"),
    )
}

async fn seed_inline_subscription_readback(cloudcontrol: &CloudControlService) {
    let mut view = cloudcontrol.snapshot(ACCOUNT_ID, REGION).await;
    let resource = view
        .resources
        .values_mut()
        .find(|resource| resource.type_name == TYPE_NAME)
        .expect("stored topic to seed");
    let mut model: serde_json::Value =
        serde_json::from_str(&resource.resource_model).expect("stored topic model should be JSON");
    model
        .as_object_mut()
        .expect("stored topic model should be an object")
        .insert(
            "Subscription".to_string(),
            serde_json::json!([{
                "Endpoint": "x@example.com",
                "Protocol": "email"
            }]),
        );
    resource.resource_model = serde_json::to_string(&model).expect("seeded topic model JSON");
    cloudcontrol
        .restore(ACCOUNT_ID, REGION, view)
        .await
        .expect("seed stored topic read model");
}

async fn seed_archive_policy_readback(
    cloudcontrol: &CloudControlService,
    archive_policy: serde_json::Value,
) {
    let mut view = cloudcontrol.snapshot(ACCOUNT_ID, REGION).await;
    let resource = view
        .resources
        .values_mut()
        .find(|resource| resource.type_name == TYPE_NAME)
        .expect("stored topic to seed");
    let mut model: serde_json::Value =
        serde_json::from_str(&resource.resource_model).expect("stored topic model should be JSON");
    model
        .as_object_mut()
        .expect("stored topic model should be an object")
        .insert("ArchivePolicy".to_string(), archive_policy);
    resource.resource_model = serde_json::to_string(&model).expect("seeded topic model JSON");
    cloudcontrol
        .restore(ACCOUNT_ID, REGION, view)
        .await
        .expect("seed stored topic ArchivePolicy read model");
}

#[test]
fn topic_schema_types_archive_policy_and_omits_unsupported_documents() {
    let config = sns_topic_config();
    assert!(
        !config
            .schema
            .as_schema()
            .attributes()
            .contains_key("subscription")
    );
    assert!(
        !config
            .schema
            .as_schema()
            .attributes()
            .contains_key("data_protection_policy")
    );
    assert!(
        config
            .schema
            .as_schema()
            .attributes()
            .contains_key("archive_policy")
    );
}

#[tokio::test(start_paused = true)]
async fn sns_topic_create_then_read_omits_inline_subscriptions_and_preserves_tags() {
    let (provider, cloudcontrol) = winterbaume_provider().await;
    let resource = topic_resource();
    let id = resource.id.clone();
    let request = common::create_request(resource.clone()).await;

    let created = common::dispatch_create(&provider, &id, request)
        .await
        .expect("sns.Topic create_resource should succeed");
    let created = match created.into_kind() {
        CreateOutcomeKind::Success { state } | CreateOutcomeKind::PartialSuccess { state, .. } => {
            state
        }
    };

    let (stored_identifier, desired_state) = stored_topic(&cloudcontrol).await;
    assert_eq!(created.identifier().as_str(), stored_identifier);
    assert_eq!(desired_state["TopicName"], TOPIC_NAME);
    assert!(desired_state.get("Subscription").is_none());
    assert_eq!(
        desired_state["ArchivePolicy"],
        serde_json::json!({"MessageRetentionPeriod": 1})
    );
    assert_eq!(
        desired_state["Tags"],
        serde_json::json!([
            {"Key": "Environment", "Value": "test"},
            {"Key": "Project", "Value": "carina-registry"}
        ])
    );

    seed_inline_subscription_readback(&cloudcontrol).await;
    let (_, seeded_state) = stored_topic(&cloudcontrol).await;
    assert_eq!(
        seeded_state["Subscription"],
        serde_json::json!([{
            "Endpoint": "x@example.com",
            "Protocol": "email"
        }])
    );

    let stored_identifier = common::applied_identifier(stored_identifier);
    let read = Provider::read(&provider, &id, &stored_identifier, ReadRequest)
        .await
        .expect("sns.Topic read through Provider::read should succeed");
    let read = common::found_state(read);

    assert!(!read.attributes().contains_key("subscription"));
    assert_eq!(
        read.attributes().get("archive_policy"),
        Some(&map([("message_retention_period", int(1))]))
    );
    assert_eq!(
        read.attributes().get("tags"),
        Some(&map([
            ("Environment", string("test")),
            ("Project", string("carina-registry")),
        ]))
    );

    let current = read.into_state_for_writeback();
    let desired = ResolvedResource::new(resource.into_resolved_dependency_phase_for_test());
    let registry = carina_provider_awscc::schemas::schema_registry();
    let saved = RawSavedAttrs::default().lift(&registry);
    let router = carina_provider_awscc::test_comparison_router();
    let comparison = router
        .comparison_for(&registry, &desired.id)
        .expect("SNS Topic comparison context");
    let result = comparison
        .context()
        .with_references(
            &carina_core::references::prepare_empty_for_tests(),
            &desired.id,
            |context| diff(&desired, &current, &saved, None, comparison, context),
        )
        .expect("AWSCC fixtures must not require reference resolution")
        .expect("schema-bound diff should succeed");
    assert!(
        matches!(result, Diff::NoChange(_)),
        "inline Subscription read-back must not produce a diff: {result:?}"
    );
}

#[tokio::test(start_paused = true)]
async fn sns_topic_empty_archive_policy_readback_is_absent_and_no_change() {
    let (provider, cloudcontrol) = winterbaume_provider().await;
    let resource = topic_resource_without_archive_policy();
    let id = resource.id.clone();
    let request = common::create_request(resource.clone()).await;

    common::dispatch_create(&provider, &id, request)
        .await
        .expect("sns.Topic create_resource should succeed");

    let (stored_identifier, _) = stored_topic(&cloudcontrol).await;
    seed_archive_policy_readback(&cloudcontrol, serde_json::json!({})).await;

    let stored_identifier = common::applied_identifier(stored_identifier);
    let read = Provider::read(&provider, &id, &stored_identifier, ReadRequest)
        .await
        .expect("sns.Topic read through Provider::read should succeed");
    let read = common::found_state(read);
    let mut current_states = HashMap::from([(id.clone(), read.into_state_for_writeback())]);
    normalize_provider_states(&carina_provider_awscc::AwsccNormalizer, &mut current_states)
        .await
        .expect("SNS Topic read-state normalization should succeed");
    let current = current_states
        .remove(&id)
        .expect("normalized SNS Topic state must remain present");
    assert!(
        !current.attributes.contains_key("archive_policy"),
        "an empty ArchivePolicy read-back means archiving is disabled"
    );

    let desired = ResolvedResource::new(resource.into_resolved_dependency_phase_for_test());
    let registry = carina_provider_awscc::schemas::schema_registry();
    let saved = RawSavedAttrs::default().lift(&registry);
    let router = carina_provider_awscc::test_comparison_router();
    let comparison = router
        .comparison_for(&registry, &desired.id)
        .expect("SNS Topic comparison context");
    let result = comparison
        .context()
        .with_references(
            &carina_core::references::prepare_empty_for_tests(),
            &desired.id,
            |context| diff(&desired, &current, &saved, None, comparison, context),
        )
        .expect("AWSCC fixtures must not require reference resolution")
        .expect("schema-bound diff should succeed");
    assert!(
        matches!(result, Diff::NoChange(_)),
        "empty ArchivePolicy read-back must not produce a diff: {result:?}"
    );
}
