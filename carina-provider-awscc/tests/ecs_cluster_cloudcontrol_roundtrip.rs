//! Integration coverage for the awscc provider's real Provider-trait apply path
//! against an in-process winterbaume CloudControl mock.
//!
//! winterbaume-cloudcontrol 1.0.0 fixes moriyoshi/winterbaume issue #8: the
//! mock now reproduces CloudFormation-schema shaping, including write-only
//! stripping, read-only synthesis, and schema-default fill-in. This test sends
//! one create request and asserts the full CFN-schema-shaped read state
//! round-trips through the awscc provider's serialization and conversion.
//!
//! In particular, `capacity_providers` survives as a list of strings and
//! `cluster_settings` survives as a list of maps instead of being flattened into
//! a string.

mod common;

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

fn attributes(entries: impl IntoIterator<Item = (&'static str, Value)>) -> HashMap<String, Value> {
    entries
        .into_iter()
        .map(|(key, value)| (key.to_string(), value))
        .collect()
}

#[tokio::test]
async fn ecs_cluster_create_then_read_round_trips_structured_list_fields() {
    let provider = winterbaume_provider().await;
    let resource = ecs_cluster_resource();
    let id = resource.id.clone();
    let resource = common::normalize_resource(resource).await;

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
    let expected = attributes([
        ("cluster_name", string("registry-fargate")),
        (
            "capacity_providers",
            list([string("FARGATE"), string("FARGATE_SPOT")]),
        ),
        (
            "cluster_settings",
            list([map([
                ("name", string("containerInsights")),
                ("value", string("enabled")),
            ])]),
        ),
        (
            "tags",
            map([
                ("Environment", string("test")),
                ("Workload", string("registry")),
            ]),
        ),
        ("default_capacity_provider_strategy", list([])),
        (
            "arn",
            string("arn:aws:ecs:us-east-1:123456789012:cluster/registry-fargate"),
        ),
    ]);
    assert_eq!(
        &read.attributes, &expected,
        "read-back attributes must exactly match the full shaped ECS cluster state"
    );
}
