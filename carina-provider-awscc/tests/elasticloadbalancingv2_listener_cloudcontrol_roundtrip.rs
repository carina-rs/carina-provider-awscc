//! Integration coverage for the awscc provider's real Provider-trait apply path
//! against an in-process winterbaume CloudControl mock.
//!
//! winterbaume-cloudcontrol 1.0.1 fixes moriyoshi/winterbaume issue #11: the
//! mock now reproduces CloudFormation-schema shaping for
//! AWS::ElasticLoadBalancingV2::Listener, including write-only stripping,
//! read-only synthesis (for example `ListenerArn`), schema-default fill-in, and
//! enriched sub-structures. This test sends one create request and asserts the
//! full CFN-schema-shaped read state round-trips through the awscc provider's
//! serialization and conversion.

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

fn key_value(key: &'static str, value: &'static str) -> Value {
    map([("key", string(key)), ("value", string(value))])
}

fn listener_resource() -> Resource {
    Resource::with_provider(
        "awscc",
        "elasticloadbalancingv2.Listener",
        "registry_listener",
        None,
    )
    .with_attribute(
        "load_balancer_arn",
        string(
            "arn:aws:elasticloadbalancing:ap-northeast-1:123456789012:loadbalancer/app/registry-alb/abc123",
        ),
    )
    .with_attribute("port", int(443))
    .with_attribute("protocol", string("HTTPS"))
    .with_attribute(
        "certificates",
        list([map([(
            "certificate_arn",
            string("arn:aws:acm:ap-northeast-1:123456789012:certificate/aaaa-bbbb"),
        )])]),
    )
    .with_attribute(
        "default_actions",
        list([map([
            ("type", string("forward")),
            (
                "target_group_arn",
                string(
                    "arn:aws:elasticloadbalancing:ap-northeast-1:123456789012:targetgroup/registry-tg/def456",
                ),
            ),
        ])]),
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

fn listener_attributes() -> Value {
    list([
        key_value("routing.http.response.server.enabled", "true"),
        key_value(
            "routing.http.response.access_control_allow_headers.header_value",
            "",
        ),
        key_value("routing.http.response.x_frame_options.header_value", ""),
        key_value(
            "routing.http.response.access_control_allow_methods.header_value",
            "",
        ),
        key_value(
            "routing.http.response.access_control_allow_origin.header_value",
            "",
        ),
        key_value(
            "routing.http.response.access_control_allow_credentials.header_value",
            "",
        ),
        key_value(
            "routing.http.response.x_content_type_options.header_value",
            "",
        ),
        key_value(
            "routing.http.response.content_security_policy.header_value",
            "",
        ),
        key_value(
            "routing.http.response.access_control_expose_headers.header_value",
            "",
        ),
        key_value(
            "routing.http.response.strict_transport_security.header_value",
            "",
        ),
        key_value(
            "routing.http.response.access_control_max_age.header_value",
            "",
        ),
    ])
}

#[tokio::test]
async fn listener_create_then_read_round_trips_full_shaped_state() {
    let provider = winterbaume_provider().await;
    let resource = listener_resource();
    let id = resource.id.clone();

    let created = Provider::create(&provider, &id, CreateRequest { resource })
        .await
        .expect("elasticloadbalancingv2.Listener create through Provider::create should succeed");
    let identifier = created
        .identifier
        .as_deref()
        .expect("CloudControl create must return a stable identifier");

    let read = Provider::read(&provider, &id, Some(identifier), ReadRequest)
        .await
        .expect("elasticloadbalancingv2.Listener read through Provider::read should succeed");

    assert!(read.exists, "read-back state must exist");
    assert_eq!(read.identifier.as_deref(), Some(identifier));

    let listener_arn = concrete_string_attribute(&read.attributes, "listener_arn");
    let suffix = listener_arn
        .strip_prefix(
            "arn:aws:elasticloadbalancing:ap-northeast-1:123456789012:listener/app/registry-alb/abc123/",
        )
        .expect("listener_arn must derive from load_balancer_arn and append a listener suffix");
    assert!(
        is_lower_hex16(suffix),
        "listener_arn suffix must be 16 hex characters: {suffix}"
    );

    let target_group_arn =
        "arn:aws:elasticloadbalancing:ap-northeast-1:123456789012:targetgroup/registry-tg/def456";
    let expected = attributes([
        (
            "load_balancer_arn",
            string(
                "arn:aws:elasticloadbalancing:ap-northeast-1:123456789012:loadbalancer/app/registry-alb/abc123",
            ),
        ),
        ("port", int(443)),
        ("protocol", string("HTTPS")),
        (
            "certificates",
            list([map([(
                "certificate_arn",
                string("arn:aws:acm:ap-northeast-1:123456789012:certificate/aaaa-bbbb"),
            )])]),
        ),
        ("alpn_policy", list([])),
        (
            "default_actions",
            list([map([
                ("type", string("forward")),
                ("target_group_arn", string(target_group_arn)),
                (
                    "forward_config",
                    map([
                        (
                            "target_group_stickiness_config",
                            map([("enabled", bool_(false))]),
                        ),
                        (
                            "target_groups",
                            list([map([
                                ("target_group_arn", string(target_group_arn)),
                                ("weight", int(1)),
                            ])]),
                        ),
                    ]),
                ),
            ])]),
        ),
        ("listener_arn", string(listener_arn)),
        ("listener_attributes", listener_attributes()),
    ]);
    assert_eq!(
        &read.attributes, &expected,
        "read-back attributes must exactly match the full shaped ELBv2 listener state"
    );
}
