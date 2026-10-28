//! Integration coverage for the awscc provider's real Provider-trait apply path
//! against an in-process winterbaume CloudControl mock.
//!
//! `AWS::CloudFront::Function` is not registered with
//! winterbaume-cloudcontrol 1.0.1, so the mock returns the create-time
//! `DesiredState` verbatim. This test therefore asserts structural preservation
//! only: `FunctionConfig` must remain a structured map, and the DSL runtime
//! alias `cloudfront_js_2_0` must reach CloudControl as `cloudfront-js-2.0` and
//! map back to the same alias. It intentionally does not assert full shaped
//! equality. Upgrade the test once winterbaume registers a shaper for this type.
//!
//! Real AWS reconciliation in us-east-1 on 2026-10-27 (`carina-test-000`):
//! create with `AutoPublish=true` returns `FunctionConfig`, `FunctionARN`,
//! `FunctionMetadata.FunctionARN`, `FunctionCode`, `Stage="LIVE"`, `Tags=[]`,
//! and `Name`, while omitting the write-only `AutoPublish`. Updating
//! `FunctionCode` together with `AutoPublish=true` publishes the new code to
//! LIVE; updating `FunctionCode` alone changes only DEVELOPMENT even though
//! `GetResource` reports the new code with `Stage="LIVE"`.

mod common;

use std::collections::HashMap;
use std::sync::Arc;

use aws_config::{BehaviorVersion, Region};
use carina_core::provider::{CreateOutcomeKind, Provider, ReadRequest};
use carina_core::resource::{ConcreteValue, Resource, Value};
use carina_core::schema::{AttributeType, RawShape, StructField, TypeError};
use carina_provider_awscc::AwsccProvider;
use carina_provider_awscc::provider::AwsccProviderConfig;
use carina_provider_awscc::schemas::generated::cloudfront::distribution::cloudfront_distribution_config;
use carina_provider_awscc::schemas::generated::cloudfront::function::cloudfront_function_config;
use indexmap::IndexMap;
use winterbaume_cloudcontrol::CloudControlService;
use winterbaume_core::{MockAws, StatefulService};

const TYPE_NAME: &str = "AWS::CloudFront::Function";
const ACCOUNT_ID: &str = "123456789012";
const REGION: &str = "us-east-1";
const FUNCTION_NAME: &str = "carina-registry-viewer-request";
const FUNCTION_CODE: &str = "function handler(event) { return event.request; }";

fn string(value: &str) -> Value {
    Value::Concrete(ConcreteValue::String(value.to_string().into()))
}

fn bool_(value: bool) -> Value {
    Value::Concrete(ConcreteValue::Bool(value))
}

fn enum_identifier(value: &str) -> Value {
    Value::Concrete(ConcreteValue::enum_identifier(value))
}

fn map(entries: impl IntoIterator<Item = (&'static str, Value)>) -> Value {
    Value::Concrete(ConcreteValue::Map(
        entries
            .into_iter()
            .map(|(key, value)| (key.to_string(), value))
            .collect::<IndexMap<_, _>>(),
    ))
}

fn find_field<'a>(attribute_type: &'a AttributeType, field_name: &str) -> &'a StructField {
    let RawShape::Struct { fields, .. } = attribute_type.raw_shape() else {
        panic!(
            "expected {field_name} parent to be a struct, got {:?}",
            attribute_type.raw_shape()
        );
    };
    fields
        .iter()
        .find(|field| field.name() == field_name)
        .unwrap_or_else(|| panic!("expected struct field {field_name}"))
}

fn list_element(attribute_type: &AttributeType) -> &AttributeType {
    let RawShape::List { element_type, .. } = attribute_type.raw_shape() else {
        panic!("expected a list, got {:?}", attribute_type.raw_shape());
    };
    element_type
}

fn cloudfront_function_resource() -> Resource {
    Resource::with_provider(
        "awscc",
        "cloudfront.Function",
        "registry_viewer_request",
        None,
    )
    .with_attribute("name", string(FUNCTION_NAME))
    .with_attribute("function_code", string(FUNCTION_CODE))
    .with_attribute(
        "function_config",
        map([
            ("comment", string("Rewrite the registry root request")),
            ("runtime", enum_identifier("cloudfront_js_2_0")),
        ]),
    )
    .with_attribute("auto_publish", bool_(true))
}

async fn winterbaume_provider() -> (AwsccProvider, Arc<CloudControlService>) {
    let cloudcontrol = Arc::new(CloudControlService::new());
    let mock = MockAws::builder()
        .with_service(Arc::clone(&cloudcontrol))
        .build();
    let config = aws_config::defaults(BehaviorVersion::latest())
        .http_client(mock.http_client())
        .credentials_provider(mock.credentials_provider())
        .region(Region::new(REGION))
        .load()
        .await;

    (
        AwsccProvider::from_sdk_config(
            carina_provider_awscc::BoundedSdkConfig::new(config),
            &AwsccProviderConfig::default(),
        )
        .await,
        cloudcontrol,
    )
}

async fn stored_function(cloudcontrol: &CloudControlService) -> (String, serde_json::Value) {
    let view = cloudcontrol.snapshot(ACCOUNT_ID, REGION).await;
    let resources = view
        .resources
        .values()
        .filter(|resource| resource.type_name == TYPE_NAME)
        .collect::<Vec<_>>();
    assert_eq!(
        resources.len(),
        1,
        "winterbaume should store exactly one CloudFront Function resource"
    );
    let resource = resources[0];
    (
        resource.identifier.clone(),
        serde_json::from_str(&resource.resource_model)
            .expect("stored resource model should be JSON"),
    )
}

#[test]
fn function_arn_is_assignable_to_distribution_function_association() {
    let function = cloudfront_function_config();
    let function_arn = function
        .schema
        .attributes()
        .get("function_arn")
        .expect("Function.function_arn should exist")
        .attr_type();
    let distribution = cloudfront_distribution_config();
    let distribution_config = distribution
        .schema
        .attributes()
        .get("distribution_config")
        .expect("Distribution.distribution_config should exist")
        .attr_type();
    let default_cache_behavior =
        find_field(distribution_config, "default_cache_behavior").field_type();
    let cache_behavior =
        list_element(find_field(distribution_config, "cache_behaviors").field_type());

    for (path, behavior) in [
        ("default_cache_behavior", default_cache_behavior),
        ("cache_behaviors[]", cache_behavior),
    ] {
        let associations = list_element(find_field(behavior, "function_associations").field_type());
        let association_arn = find_field(associations, "function_arn").field_type();

        assert!(
            function
                .schema
                .type_in_schema(function_arn)
                .is_assignable_to(distribution.schema.type_in_schema(association_arn)),
            "Function.function_arn must be reference-compatible with Distribution.{path}.function_associations[].function_arn"
        );
    }
}

#[test]
fn function_metadata_is_provider_populated_and_rejects_assignment() {
    let config = cloudfront_function_config();
    let function_metadata = config
        .schema
        .attributes()
        .get("function_metadata")
        .expect("Function.function_metadata should exist");
    assert!(
        function_metadata.is_read_only(),
        "FunctionMetadata contains only provider-populated fields"
    );

    let attributes = HashMap::from([
        ("name".to_string(), string(FUNCTION_NAME)),
        ("function_code".to_string(), string(FUNCTION_CODE)),
        (
            "function_config".to_string(),
            map([
                ("comment", string("Rewrite the registry root request")),
                ("runtime", enum_identifier("cloudfront_js_2_0")),
            ]),
        ),
        (
            "function_metadata".to_string(),
            map([(
                "function_arn",
                string("arn:aws:cloudfront::123456789012:function/user-supplied"),
            )]),
        ),
    ]);
    let errors = config
        .schema
        .validate(&attributes)
        .expect_err("assigning FunctionMetadata must fail validation");
    assert!(
        matches!(
            errors.as_slice(),
            [TypeError::ReadOnlyAttribute { name }] if name == "function_metadata"
        ),
        "expected a provider-populated function_metadata error, got {errors:?}"
    );
}

#[tokio::test(start_paused = true)]
async fn cloudfront_function_create_then_read_preserves_config_and_runtime_alias() {
    let (provider, cloudcontrol) = winterbaume_provider().await;
    let resource = cloudfront_function_resource();
    let id = resource.id.clone();
    let request = common::create_request(resource).await;

    let created = carina_core::executor::dispatch_create(&provider, &id, request)
        .await
        .expect("cloudfront.Function create_resource should succeed");
    let created = match created.into_kind() {
        CreateOutcomeKind::Success { state } | CreateOutcomeKind::PartialSuccess { state, .. } => {
            state
        }
    };

    let (stored_identifier, desired_state) = stored_function(&cloudcontrol).await;
    assert_eq!(created.identifier().as_str(), stored_identifier);
    assert_eq!(desired_state["Name"], FUNCTION_NAME);
    assert_eq!(desired_state["FunctionCode"], FUNCTION_CODE);
    assert_eq!(desired_state["AutoPublish"], true);
    assert_eq!(
        desired_state["FunctionConfig"],
        serde_json::json!({
            "Comment": "Rewrite the registry root request",
            "Runtime": "cloudfront-js-2.0"
        }),
        "FunctionConfig must stay structured and use the API runtime spelling"
    );

    let stored_identifier = common::applied_identifier(stored_identifier);
    let read = Provider::read(&provider, &id, &stored_identifier, ReadRequest)
        .await
        .expect("cloudfront.Function read through Provider::read should succeed");
    let read = common::found_state(read);

    let Some(Value::Concrete(ConcreteValue::Map(function_config))) =
        read.attributes().get("function_config")
    else {
        panic!("function_config must round-trip as a map");
    };
    assert_eq!(
        function_config.get("comment"),
        Some(&string("Rewrite the registry root request"))
    );
    let runtime = function_config
        .get("runtime")
        .expect("function_config.runtime must round-trip");
    let Value::Concrete(ConcreteValue::String(runtime)) = runtime else {
        panic!("runtime must read back as the API string, got {runtime:?}");
    };

    let config = cloudfront_function_config();
    let function_config_type = config
        .schema
        .attributes()
        .get("function_config")
        .expect("Function.function_config should exist")
        .attr_type();
    let runtime_type = find_field(function_config_type, "runtime").field_type();
    let (_, _, _, _, runtime_dsl_map) = runtime_type
        .enum_parts()
        .expect("FunctionConfig.runtime should be an enum");
    assert_eq!(
        runtime_dsl_map.dsl_for(runtime.expose_plaintext()),
        "cloudfront_js_2_0",
        "the API runtime must map back to its dotted-value DSL alias"
    );
}
