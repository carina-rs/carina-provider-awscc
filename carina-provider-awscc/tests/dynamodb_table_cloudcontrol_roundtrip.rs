//! Integration coverage for the awscc provider's real Provider-trait apply path
//! against an in-process winterbaume CloudControl mock.
//!
//! winterbaume-cloudcontrol 1.0.0 fixes moriyoshi/winterbaume issue #7: the
//! mock now reproduces CloudFormation-schema shaping, including write-only
//! stripping, read-only synthesis, and schema-default fill-in. This test sends
//! one create request and asserts the full CFN-schema-shaped read state
//! round-trips through the awscc provider's serialization and conversion.
//!
//! In particular, `key_schema`, whose generated type is selected through oneOf
//! resolution, survives the real apply path as a structured list of maps instead
//! of being flattened into a string.

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

fn attributes(entries: impl IntoIterator<Item = (&'static str, Value)>) -> HashMap<String, Value> {
    entries
        .into_iter()
        .map(|(key, value)| (key.to_string(), value))
        .collect()
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
    let expected = attributes([
        ("table_name", string("jti-store")),
        ("billing_mode", string("PAY_PER_REQUEST")),
        (
            "attribute_definitions",
            list([map([
                ("attribute_name", string("jti")),
                ("attribute_type", string("S")),
            ])]),
        ),
        (
            "key_schema",
            list([map([
                ("attribute_name", string("jti")),
                ("key_type", string("HASH")),
            ])]),
        ),
        (
            "time_to_live_specification",
            map([("attribute_name", string("ttl")), ("enabled", bool_(true))]),
        ),
        (
            "point_in_time_recovery_specification",
            map([("point_in_time_recovery_enabled", bool_(true))]),
        ),
        ("tags", map([("Environment", string("test"))])),
        ("sse_specification", map([("sse_enabled", bool_(false))])),
        (
            "contributor_insights_specification",
            map([("enabled", bool_(false))]),
        ),
        ("deletion_protection_enabled", bool_(false)),
        ("local_secondary_indexes", list([])),
        ("global_secondary_indexes", list([])),
        (
            "warm_throughput",
            map([
                ("read_units_per_second", int(12000)),
                ("write_units_per_second", int(4000)),
            ]),
        ),
        (
            "arn",
            string("arn:aws:dynamodb:us-east-1:123456789012:table/jti-store"),
        ),
    ]);
    assert_eq!(
        &read.attributes, &expected,
        "read-back attributes must exactly match the full shaped DynamoDB table state"
    );
}
