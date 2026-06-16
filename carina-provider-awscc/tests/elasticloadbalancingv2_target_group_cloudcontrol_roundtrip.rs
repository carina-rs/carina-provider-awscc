//! Integration coverage for the awscc provider's real Provider-trait apply path
//! against an in-process winterbaume CloudControl mock.
//!
//! winterbaume-cloudcontrol 1.0.1 fixes moriyoshi/winterbaume issue #9: the
//! mock now reproduces CloudFormation-schema shaping for
//! AWS::ElasticLoadBalancingV2::TargetGroup, including read-only synthesis
//! (for example `TargetGroupArn`) and schema-default fill-in. This test sends
//! one create request and asserts the full CFN-schema-shaped read state
//! round-trips through the awscc provider's serialization and conversion.

mod common;

use aws_config::{BehaviorVersion, Region};
use carina_core::provider::{CreateOutcome, CreateRequest, Provider, ReadRequest};
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

fn key_value(key: &'static str, value: &'static str) -> Value {
    map([("key", string(key)), ("value", string(value))])
}

fn target_group_resource() -> Resource {
    Resource::with_provider(
        "awscc",
        "elasticloadbalancingv2.TargetGroup",
        "registry_tg",
        None,
    )
    .with_attribute("name", string("registry-tg"))
    .with_attribute("protocol", string("HTTP"))
    .with_attribute("port", int(8080))
    .with_attribute("vpc_id", string("vpc-dddd4444"))
    .with_attribute("target_type", string("ip"))
    .with_attribute("health_check_path", string("/health"))
    .with_attribute(
        "targets",
        list([map([("id", string("10.0.1.10")), ("port", int(8080))])]),
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

fn concrete_string_attribute<'a>(
    attributes: &'a HashMap<String, Value>,
    attribute_name: &str,
) -> &'a str {
    match attributes.get(attribute_name) {
        Some(Value::Concrete(ConcreteValue::String(value))) => value,
        other => panic!("{attribute_name} must be a String, got {other:?}"),
    }
}

fn is_lower_hex16(value: &str) -> bool {
    value.len() == 16 && value.chars().all(|ch| ch.is_ascii_hexdigit())
}

fn target_group_attributes() -> Value {
    list([
        key_value("stickiness.type", "lb_cookie"),
        key_value("stickiness.app_cookie.duration_seconds", "86400"),
        key_value(
            "target_group_health.dns_failover.minimum_healthy_targets.count",
            "1",
        ),
        key_value(
            "load_balancing.cross_zone.enabled",
            "use_load_balancer_configuration",
        ),
        key_value("stickiness.lb_cookie.duration_seconds", "86400"),
        key_value(
            "target_group_health.dns_failover.minimum_healthy_targets.percentage",
            "off",
        ),
        key_value("stickiness.enabled", "false"),
        key_value(
            "target_group_health.unhealthy_state_routing.minimum_healthy_targets.percentage",
            "off",
        ),
        key_value("slow_start.duration_seconds", "0"),
        key_value("deregistration_delay.timeout_seconds", "300"),
        key_value(
            "target_group_health.unhealthy_state_routing.minimum_healthy_targets.count",
            "1",
        ),
        key_value("load_balancing.algorithm.anomaly_mitigation", "off"),
        key_value("stickiness.app_cookie.cookie_name", ""),
        key_value("load_balancing.algorithm.type", "round_robin"),
    ])
}

#[tokio::test]
async fn target_group_create_then_read_round_trips_full_shaped_state() {
    let provider = winterbaume_provider().await;
    let resource = target_group_resource();
    let id = resource.id.clone();
    let resource = common::normalize_resource(resource).await;

    let created = Provider::create(&provider, &id, CreateRequest { resource })
        .await
        .expect(
            "elasticloadbalancingv2.TargetGroup create through Provider::create should succeed",
        );
    let created = match created {
        CreateOutcome::Success { state } => state,
        CreateOutcome::PartialSuccess { diagnostic, .. } => {
            panic!(
                "roundtrip create should be full success, got partial: {:?}",
                diagnostic
            )
        }
    };
    let identifier = created
        .identifier
        .as_deref()
        .expect("CloudControl create must return a stable identifier");

    let read = Provider::read(&provider, &id, Some(identifier), ReadRequest)
        .await
        .expect("elasticloadbalancingv2.TargetGroup read through Provider::read should succeed");

    assert!(read.exists, "read-back state must exist");
    assert_eq!(read.identifier.as_deref(), Some(identifier));

    let target_group_arn = concrete_string_attribute(&read.attributes, "target_group_arn");
    let target_group_full_name =
        concrete_string_attribute(&read.attributes, "target_group_full_name");
    let suffix = target_group_full_name
        .strip_prefix("targetgroup/registry-tg/")
        .expect("target_group_full_name must use targetgroup/<name>/<suffix>");
    assert!(
        is_lower_hex16(suffix),
        "target_group_full_name suffix must be 16 hex characters: {suffix}"
    );
    assert_eq!(
        target_group_arn,
        format!("arn:aws:elasticloadbalancing:us-east-1:123456789012:{target_group_full_name}")
    );

    let expected = attributes([
        ("name", string("registry-tg")),
        ("protocol", string("HTTP")),
        ("port", int(8080)),
        ("vpc_id", string("vpc-dddd4444")),
        ("target_type", string("ip")),
        ("health_check_path", string("/health")),
        (
            "targets",
            list([map([("id", string("10.0.1.10")), ("port", int(8080))])]),
        ),
        (
            "tags",
            map([
                ("Environment", string("test")),
                ("Workload", string("registry")),
            ]),
        ),
        ("target_group_name", string("registry-tg")),
        ("target_group_full_name", string(target_group_full_name)),
        ("target_group_arn", string(target_group_arn)),
        ("load_balancer_arns", list([])),
        ("ip_address_type", string("ipv4")),
        ("health_check_enabled", bool_(true)),
        ("health_check_interval_seconds", int(30)),
        ("health_check_protocol", string("HTTP")),
        ("health_check_port", string("traffic-port")),
        ("health_check_timeout_seconds", int(5)),
        ("healthy_threshold_count", int(5)),
        ("unhealthy_threshold_count", int(2)),
        ("protocol_version", string("HTTP1")),
        ("matcher", map([("http_code", string("200"))])),
        ("target_group_attributes", target_group_attributes()),
    ]);
    assert_eq!(
        &read.attributes, &expected,
        "read-back attributes must exactly match the full shaped ELBv2 target group state"
    );
}
