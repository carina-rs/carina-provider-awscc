//! Integration coverage for the awscc provider's real Provider-trait apply path
//! against an in-process winterbaume CloudControl mock.
//!
//! This test proves that create -> read wiring round-trips ECS cluster
//! structured list serialization through CloudControl. In particular,
//! `capacity_providers` survives as a list of strings and `cluster_settings`
//! survives as a list of maps instead of being flattened into a string.
//!
//! This test deliberately does not assert write-only stripping, read-only
//! synthesis such as `arn`, schema-default fill-in, or normalization. Those are
//! real AWS CloudControl behaviours driven by the CloudFormation resource-type
//! schema; the generic winterbaume mock returns the desired state verbatim and
//! does not reproduce them because it does not consult CFN schemas. That
//! winterbaume behavior is tracked upstream as moriyoshi/winterbaume issue #6.
//! The AWS behaviours should be verified separately against live AWS, not here.

use aws_config::{BehaviorVersion, Region};
use carina_core::provider::{CreateRequest, Provider, ReadRequest};
use carina_core::resource::{ConcreteValue, Resource, Value};
use carina_provider_awscc::AwsccProvider;
use carina_provider_awscc::provider::AwsccProviderConfig;
use indexmap::IndexMap;
use std::collections::HashMap;
use winterbaume_cloudcontrol::CloudControlService;
use winterbaume_core::MockAws;

fn string(value: &str) -> Value {
    Value::Concrete(ConcreteValue::String(value.to_string()))
}

fn map(entries: impl IntoIterator<Item = (&'static str, Value)>) -> Value {
    Value::Concrete(ConcreteValue::Map(
        entries
            .into_iter()
            .map(|(key, value)| (key.to_string(), value))
            .collect::<IndexMap<_, _>>(),
    ))
}

fn list(items: impl IntoIterator<Item = Value>) -> Value {
    Value::Concrete(ConcreteValue::List(items.into_iter().collect()))
}

fn ecs_cluster_resource() -> Resource {
    Resource::with_provider("awscc", "ecs.Cluster", "registry_fargate", None)
        .with_attribute("cluster_name", string("registry-fargate"))
        .with_attribute(
            "capacity_providers",
            list([string("FARGATE"), string("FARGATE_SPOT")]),
        )
        .with_attribute(
            "cluster_settings",
            list([map([
                ("name", string("containerInsights")),
                ("value", string("enabled")),
            ])]),
        )
        .with_attribute(
            "tags",
            map([
                ("Environment", string("test")),
                ("Workload", string("registry")),
            ]),
        )
}

async fn winterbaume_provider() -> AwsccProvider {
    let mock = MockAws::builder()
        .with_service(CloudControlService::new())
        .build();
    let config = aws_config::defaults(BehaviorVersion::latest())
        .http_client(mock.http_client())
        .credentials_provider(mock.credentials_provider())
        .region(Region::new("us-east-1"))
        .load()
        .await;

    AwsccProvider::from_sdk_config(config, &AwsccProviderConfig::default()).await
}

fn assert_single_map_list(
    attributes: &HashMap<String, Value>,
    attribute_name: &str,
    expected_entries: &[(&str, Value)],
) {
    let value = attributes
        .get(attribute_name)
        .unwrap_or_else(|| panic!("read-back state must include {attribute_name}"));
    let items = match value {
        Value::Concrete(ConcreteValue::List(items)) => items,
        other => panic!("{attribute_name} must round-trip as List(Map), got {other:?}"),
    };
    assert_eq!(
        items.len(),
        1,
        "{attribute_name} must contain one structured element"
    );
    let item = match &items[0] {
        Value::Concrete(ConcreteValue::Map(item)) => item,
        other => panic!("{attribute_name}[0] must round-trip as Map, got {other:?}"),
    };

    for (key, expected_value) in expected_entries {
        assert_eq!(
            item.get(*key),
            Some(expected_value),
            "{attribute_name}[0].{key} must round-trip"
        );
    }
}

fn assert_string_list(attributes: &HashMap<String, Value>, attribute_name: &str, expected: Value) {
    let value = attributes
        .get(attribute_name)
        .unwrap_or_else(|| panic!("read-back state must include {attribute_name}"));
    let items = match value {
        Value::Concrete(ConcreteValue::List(items)) => items,
        other => panic!("{attribute_name} must round-trip as List(String), got {other:?}"),
    };

    assert_eq!(
        items.len(),
        2,
        "{attribute_name} must contain two structured string elements"
    );
    assert_eq!(value, &expected);
    assert!(
        items
            .iter()
            .all(|item| matches!(item, Value::Concrete(ConcreteValue::String(_)))),
        "{attribute_name} must contain only string elements"
    );
}

#[tokio::test]
async fn ecs_cluster_create_then_read_round_trips_structured_list_fields() {
    let provider = winterbaume_provider().await;
    let resource = ecs_cluster_resource();
    let id = resource.id.clone();

    let created = Provider::create(&provider, &id, CreateRequest { resource })
        .await
        .expect("ecs.Cluster create through Provider::create should succeed");
    let identifier = created
        .identifier
        .as_deref()
        .expect("CloudControl create must return a stable identifier");

    let read = Provider::read(&provider, &id, Some(identifier), ReadRequest)
        .await
        .expect("ecs.Cluster read through Provider::read should succeed");

    assert!(read.exists, "read-back state must exist");
    assert_eq!(read.identifier.as_deref(), Some(identifier));
    assert_eq!(
        read.attributes.get("cluster_name"),
        Some(&string("registry-fargate"))
    );
    let expected_capacity_providers = list([string("FARGATE"), string("FARGATE_SPOT")]);
    assert_string_list(
        &read.attributes,
        "capacity_providers",
        expected_capacity_providers,
    );
    assert_single_map_list(
        &read.attributes,
        "cluster_settings",
        &[
            ("name", string("containerInsights")),
            ("value", string("enabled")),
        ],
    );
    assert_eq!(
        read.attributes.get("tags"),
        Some(&map([
            ("Environment", string("test")),
            ("Workload", string("registry")),
        ]))
    );
}
