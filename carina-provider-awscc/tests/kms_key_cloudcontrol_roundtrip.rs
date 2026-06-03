//! Integration coverage for the awscc provider's real Provider-trait apply path
//! against an in-process winterbaume CloudControl mock.
//!
//! This test proves that create -> read wiring round-trips the provider's
//! serialization and conversion with no network. In particular, `key_policy`
//! survives as a structured policy document, which is PR #313's core
//! `key_policy` = `iam_policy_document` design.
//!
//! This test deliberately does not assert write-only stripping, read-only
//! synthesis, or schema-default fill-in. Those are real AWS CloudControl
//! behaviours driven by the CloudFormation resource-type schema; the generic
//! winterbaume mock returns the desired state verbatim and does not reproduce
//! them because it does not consult CFN schemas. That winterbaume behavior is
//! tracked upstream as moriyoshi/winterbaume issue #6. The AWS behaviours were
//! verified separately against live AWS, not here.

use aws_config::{BehaviorVersion, Region};
use carina_core::provider::{CreateRequest, Provider, ReadRequest};
use carina_core::resource::{ConcreteValue, Resource, Value};
use carina_provider_awscc::AwsccProvider;
use carina_provider_awscc::provider::AwsccProviderConfig;
use indexmap::IndexMap;
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

fn key_policy() -> Value {
    map([
        ("version", string("2012-10-17")),
        (
            "statement",
            list([map([
                ("sid", string("AllowRootAccountAccess")),
                ("effect", string("Allow")),
                (
                    "principal",
                    map([("aws", string("arn:aws:iam::111122223333:root"))]),
                ),
                ("action", string("kms:*")),
                ("resource", string("*")),
            ])]),
        ),
    ])
}

fn kms_key_resource() -> Resource {
    Resource::with_provider("awscc", "kms.Key", "signing_key", None)
        .with_attribute("description", string("Winterbaume signing key"))
        .with_attribute("key_policy", key_policy())
        .with_attribute("key_usage", string("SIGN_VERIFY"))
        .with_attribute("key_spec", string("ECC_NIST_P256"))
        .with_attribute("enable_key_rotation", bool_(false))
        .with_attribute("pending_window_in_days", int(7))
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

#[tokio::test]
async fn kms_key_create_then_read_round_trips_structured_key_policy() {
    let provider = winterbaume_provider().await;
    let resource = kms_key_resource();
    let id = resource.id.clone();
    let desired_policy = resource
        .attributes
        .get("key_policy")
        .expect("test resource has key_policy")
        .clone();

    let created = Provider::create(&provider, &id, CreateRequest { resource })
        .await
        .expect("kms.Key create through Provider::create should succeed");
    let identifier = created
        .identifier
        .as_deref()
        .expect("CloudControl create must return a stable identifier");

    let read = Provider::read(&provider, &id, Some(identifier), ReadRequest)
        .await
        .expect("kms.Key read through Provider::read should succeed");

    assert!(read.exists, "read-back state must exist");
    assert_eq!(read.identifier.as_deref(), Some(identifier));
    assert_eq!(
        read.attributes.get("description"),
        Some(&string("Winterbaume signing key"))
    );
    assert_eq!(
        read.attributes.get("key_usage"),
        Some(&string("SIGN_VERIFY"))
    );
    assert_eq!(
        read.attributes.get("key_spec"),
        Some(&string("ECC_NIST_P256"))
    );
    assert_eq!(
        read.attributes.get("enable_key_rotation"),
        Some(&bool_(false))
    );
    assert_eq!(read.attributes.get("key_policy"), Some(&desired_policy));
    assert!(
        matches!(
            read.attributes.get("key_policy"),
            Some(Value::Concrete(ConcreteValue::Map(_)))
        ),
        "key_policy must remain a structured policy document"
    );
    assert_eq!(
        read.attributes.get("tags"),
        Some(&map([("Environment", string("test"))]))
    );
}
