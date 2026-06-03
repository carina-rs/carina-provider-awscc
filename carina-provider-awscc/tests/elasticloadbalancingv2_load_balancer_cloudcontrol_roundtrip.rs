//! Integration coverage for the awscc provider's real Provider-trait apply path
//! against an in-process winterbaume CloudControl mock.
//!
//! This test proves that create -> read wiring round-trips Elastic Load
//! Balancing v2 load balancer structured list serialization through
//! CloudControl. In particular, `subnets` survives as a list of strings instead
//! of being flattened into a string.
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
async fn load_balancer_create_then_read_round_trips_structured_list_fields() {
    let provider = winterbaume_provider().await;
    let resource = load_balancer_resource();
    let id = resource.id.clone();

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
    assert_eq!(read.attributes.get("name"), Some(&string("registry-alb")));
    let expected_subnets = list([string("subnet-aaaa1111"), string("subnet-bbbb2222")]);
    assert_string_list(&read.attributes, "subnets", expected_subnets);

    let security_groups = read
        .attributes
        .get("security_groups")
        .expect("read-back state must include security_groups");
    let security_group_items = match security_groups {
        Value::Concrete(ConcreteValue::List(items)) => items,
        other => panic!("security_groups must round-trip as List(String), got {other:?}"),
    };
    assert_eq!(
        security_group_items.len(),
        1,
        "security_groups must contain one structured string element"
    );
    assert_eq!(security_groups, &list([string("sg-cccc3333")]));
    assert!(
        security_group_items
            .iter()
            .all(|item| matches!(item, Value::Concrete(ConcreteValue::String(_)))),
        "security_groups must contain only string elements"
    );

    assert_eq!(
        read.attributes.get("tags"),
        Some(&map([
            ("Environment", string("test")),
            ("Workload", string("registry")),
        ]))
    );
}
