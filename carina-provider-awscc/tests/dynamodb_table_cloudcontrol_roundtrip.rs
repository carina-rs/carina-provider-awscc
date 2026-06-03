//! Integration coverage for the awscc provider's real Provider-trait apply path
//! against an in-process winterbaume CloudControl mock.
//!
//! This test proves that create -> read wiring round-trips DynamoDB table
//! list-of-struct serialization through CloudControl. In particular,
//! `key_schema`, whose generated type is selected through oneOf resolution,
//! survives the real apply path as a structured list of maps instead of being
//! flattened into a string.
//!
//! This test deliberately does not assert write-only stripping, read-only
//! synthesis, or schema-default fill-in. Those are real AWS CloudControl
//! behaviours driven by the CloudFormation resource-type schema; the generic
//! winterbaume mock returns the desired state verbatim and does not reproduce
//! them because it does not consult CFN schemas. That winterbaume behavior is
//! tracked upstream as moriyoshi/winterbaume issue #6. The AWS behaviours
//! should be verified separately against live AWS, not here.

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

#[allow(dead_code)]
fn int(value: i64) -> Value {
    Value::Concrete(ConcreteValue::Int(value))
}

fn bool_(value: bool) -> Value {
    Value::Concrete(ConcreteValue::Bool(value))
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

fn dynamodb_table_resource() -> Resource {
    Resource::with_provider("awscc", "dynamodb.Table", "jti_store", None)
        .with_attribute("table_name", string("jti-store"))
        .with_attribute("billing_mode", string("PAY_PER_REQUEST"))
        .with_attribute(
            "attribute_definitions",
            list([map([
                ("attribute_name", string("jti")),
                ("attribute_type", string("S")),
            ])]),
        )
        .with_attribute(
            "key_schema",
            list([map([
                ("attribute_name", string("jti")),
                ("key_type", string("HASH")),
            ])]),
        )
        .with_attribute(
            "time_to_live_specification",
            map([("enabled", bool_(true)), ("attribute_name", string("ttl"))]),
        )
        .with_attribute(
            "point_in_time_recovery_specification",
            map([("point_in_time_recovery_enabled", bool_(true))]),
        )
        .with_attribute("tags", map([("Environment", string("test"))]))
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

#[tokio::test]
async fn dynamodb_table_create_then_read_round_trips_list_of_struct_fields() {
    let provider = winterbaume_provider().await;
    let resource = dynamodb_table_resource();
    let id = resource.id.clone();

    let created = Provider::create(&provider, &id, CreateRequest { resource })
        .await
        .expect("dynamodb.Table create through Provider::create should succeed");
    let identifier = created
        .identifier
        .as_deref()
        .expect("CloudControl create must return a stable identifier");

    let read = Provider::read(&provider, &id, Some(identifier), ReadRequest)
        .await
        .expect("dynamodb.Table read through Provider::read should succeed");

    assert!(read.exists, "read-back state must exist");
    assert_eq!(read.identifier.as_deref(), Some(identifier));
    assert_eq!(
        read.attributes.get("table_name"),
        Some(&string("jti-store"))
    );
    assert_eq!(
        read.attributes.get("billing_mode"),
        Some(&string("PAY_PER_REQUEST"))
    );
    assert_single_map_list(
        &read.attributes,
        "key_schema",
        &[
            ("attribute_name", string("jti")),
            ("key_type", string("HASH")),
        ],
    );
    assert_single_map_list(
        &read.attributes,
        "attribute_definitions",
        &[
            ("attribute_name", string("jti")),
            ("attribute_type", string("S")),
        ],
    );
    assert_eq!(
        read.attributes.get("tags"),
        Some(&map([("Environment", string("test"))]))
    );
}
