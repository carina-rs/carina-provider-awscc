//! Integration coverage for the awscc provider's real Provider-trait apply path
//! against an in-process winterbaume CloudControl mock.
//!
//! winterbaume-cloudcontrol 1.0.1 fixes moriyoshi/winterbaume issue #10: the
//! mock now reproduces CloudFormation-schema shaping for
//! AWS::ElasticLoadBalancingV2::LoadBalancer, including write-only stripping,
//! read-only synthesis (for example `LoadBalancerArn`), schema-default fill-in,
//! and derived sub-structures. This test sends one create request and asserts
//! the full CFN-schema-shaped read state round-trips through the awscc
//! provider's serialization and conversion.

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

fn key_value(key: &'static str, value: &'static str) -> Value {
    map([("key", string(key)), ("value", string(value))])
}

fn load_balancer_resource() -> Resource {
    Resource::with_provider(
        "awscc",
        "elasticloadbalancingv2.LoadBalancer",
        "registry_alb",
        None,
    )
    .with_attribute("name", string("registry-alb"))
    .with_attribute("type", string("application"))
    .with_attribute("scheme", string("internet-facing"))
    .with_attribute(
        "subnets",
        list([string("subnet-aaaa1111"), string("subnet-bbbb2222")]),
    )
    .with_attribute("security_groups", list([string("sg-cccc3333")]))
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

fn is_decimal10(value: &str) -> bool {
    value.len() == 10 && value.chars().all(|ch| ch.is_ascii_digit())
}

fn load_balancer_attributes() -> Value {
    list([
        key_value("access_logs.s3.prefix", ""),
        key_value("routing.http.xff_header_processing.mode", "append"),
        key_value("routing.http2.enabled", "true"),
        key_value("waf.fail_open.enabled", "false"),
        key_value("connection_logs.s3.bucket", ""),
        key_value("access_logs.s3.enabled", "false"),
        key_value("zonal_shift.config.enabled", "false"),
        key_value("routing.http.desync_mitigation_mode", "defensive"),
        key_value("connection_logs.s3.prefix", ""),
        key_value("health_check_logs.s3.prefix", ""),
        key_value(
            "routing.http.x_amzn_tls_version_and_cipher_suite.enabled",
            "false",
        ),
        key_value("routing.http.preserve_host_header.enabled", "false"),
        key_value("load_balancing.cross_zone.enabled", "true"),
        key_value("health_check_logs.s3.enabled", "false"),
        key_value("health_check_logs.s3.bucket", ""),
        key_value("routing.http.xff_client_port.enabled", "false"),
        key_value("access_logs.s3.bucket", ""),
        key_value("deletion_protection.enabled", "false"),
        key_value("client_keep_alive.seconds", "3600"),
        key_value("routing.http.drop_invalid_header_fields.enabled", "false"),
        key_value("connection_logs.s3.enabled", "false"),
        key_value("idle_timeout.timeout_seconds", "60"),
    ])
}

#[tokio::test]
async fn load_balancer_create_then_read_round_trips_full_shaped_state() {
    let provider = winterbaume_provider().await;
    let resource = load_balancer_resource();
    let id = resource.id.clone();
    let resource = common::normalize_resource(resource).await;

    let created = Provider::create(&provider, &id, CreateRequest { resource })
        .await
        .expect(
            "elasticloadbalancingv2.LoadBalancer create through Provider::create should succeed",
        );
    let identifier = created
        .identifier
        .as_deref()
        .expect("CloudControl create must return a stable identifier");

    let read = Provider::read(&provider, &id, Some(identifier), ReadRequest)
        .await
        .expect("elasticloadbalancingv2.LoadBalancer read through Provider::read should succeed");

    assert!(read.exists, "read-back state must exist");
    assert_eq!(read.identifier.as_deref(), Some(identifier));

    let load_balancer_arn = concrete_string_attribute(&read.attributes, "load_balancer_arn");
    let load_balancer_full_name =
        concrete_string_attribute(&read.attributes, "load_balancer_full_name");
    let suffix = load_balancer_full_name
        .strip_prefix("app/registry-alb/")
        .expect("load_balancer_full_name must use app/<name>/<suffix>");
    assert!(
        is_lower_hex16(suffix),
        "load_balancer_full_name suffix must be 16 hex characters: {suffix}"
    );
    assert_eq!(
        load_balancer_arn,
        format!(
            "arn:aws:elasticloadbalancing:us-east-1:123456789012:loadbalancer/{load_balancer_full_name}"
        )
    );

    let dns_name = concrete_string_attribute(&read.attributes, "dns_name");
    let dns_suffix = dns_name
        .strip_prefix("registry-alb-")
        .and_then(|rest| rest.strip_suffix(".us-east-1.elb.amazonaws.com"))
        .expect("dns_name must use <name>-<10digits>.<region>.elb.amazonaws.com");
    assert!(
        is_decimal10(dns_suffix),
        "dns_name suffix must be 10 decimal digits: {dns_suffix}"
    );

    let expected = attributes([
        ("name", string("registry-alb")),
        ("type", string("application")),
        ("scheme", string("internet-facing")),
        (
            "subnets",
            list([string("subnet-aaaa1111"), string("subnet-bbbb2222")]),
        ),
        ("security_groups", list([string("sg-cccc3333")])),
        (
            "tags",
            map([
                ("Environment", string("test")),
                ("Workload", string("registry")),
            ]),
        ),
        ("load_balancer_name", string("registry-alb")),
        ("load_balancer_full_name", string(load_balancer_full_name)),
        ("load_balancer_arn", string(load_balancer_arn)),
        ("dns_name", string(dns_name)),
        ("canonical_hosted_zone_id", string("Z35SXDOTRQ7X7K")),
        ("ip_address_type", string("ipv4")),
        ("enable_prefix_for_ipv6_source_nat", string("off")),
        ("load_balancer_attributes", load_balancer_attributes()),
        (
            "subnet_mappings",
            list([
                map([("subnet_id", string("subnet-aaaa1111"))]),
                map([("subnet_id", string("subnet-bbbb2222"))]),
            ]),
        ),
    ]);
    assert_eq!(
        &read.attributes, &expected,
        "read-back attributes must exactly match the full shaped ELBv2 load balancer state"
    );
}
