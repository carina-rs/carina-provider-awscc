//! Integration coverage for the awscc provider's real Provider-trait apply path
//! against an in-process winterbaume CloudControl mock.
//!
//! `AWS::CloudFront::ResponseHeadersPolicy` is not registered with
//! winterbaume-cloudcontrol 1.0.1, so the mock returns the create-time
//! `DesiredState` verbatim. This test therefore asserts structural preservation
//! only: the nested `cors_config`, `security_headers_config`,
//! `content_security_policy`, and `content_type_options` values must survive as
//! structured values. It intentionally does not assert full shaped equality.
//! Upgrade the test once winterbaume registers a shaper for this type.
//!
//! Real AWS reconciliation in us-east-1 on 2026-10-27 (carina-test-000):
//! create accepts `Name`, `Comment`, and `SecurityHeadersConfig` with only
//! `ContentSecurityPolicy` and `ContentTypeOptions`. `GetResource` returns `Id`
//! and `LastModifiedTime`, and returns every unset `SecurityHeadersConfig` member
//! (`FrameOptions`, `ReferrerPolicy`, `StrictTransportSecurity`, `XSSProtection`)
//! as an empty object `{}`. Because CloudControl applies a patch on top of that
//! read model, a JSON patch on a nested path fails model validation (the empty
//! members lack their required keys), while replacing the whole
//! `/ResponseHeadersPolicyConfig` succeeds; the provider's update path sends the
//! whole top-level attribute, so a `Comment`, CSP, and `Name` change all update
//! in place with the same `Id`. A carina apply -> plan cycle shows no diff for
//! the empty members, and an in-place update through carina apply leaves the
//! next plan clean.

mod common;

use std::sync::Arc;

use aws_config::{BehaviorVersion, Region};
use carina_core::provider::{CreateOutcomeKind, Provider, ReadRequest};
use carina_core::resource::{ConcreteValue, Resource, Value};
use carina_core::schema::{AttributeType, RawShape, StructField};
use carina_provider_awscc::AwsccProvider;
use carina_provider_awscc::provider::AwsccProviderConfig;
use carina_provider_awscc::schemas::generated::cloudfront::distribution::cloudfront_distribution_config;
use carina_provider_awscc::schemas::generated::cloudfront::origin_access_control::cloudfront_origin_access_control_config;
use carina_provider_awscc::schemas::generated::cloudfront::response_headers_policy::cloudfront_response_headers_policy_config;
use indexmap::IndexMap;
use winterbaume_cloudcontrol::CloudControlService;
use winterbaume_core::{MockAws, StatefulService};

const TYPE_NAME: &str = "AWS::CloudFront::ResponseHeadersPolicy";
const ACCOUNT_ID: &str = "123456789012";
const REGION: &str = "us-east-1";
const POLICY_NAME: &str = "carina-registry-security-headers";
const POLICY_COMMENT: &str = "Security headers for the registry read plane";
const CONTENT_SECURITY_POLICY: &str = "default-src 'self'";

fn string(value: &str) -> Value {
    Value::Concrete(ConcreteValue::String(value.to_string().into()))
}

fn bool_(value: bool) -> Value {
    Value::Concrete(ConcreteValue::Bool(value))
}

fn enum_identifier(value: &str) -> Value {
    Value::Concrete(ConcreteValue::enum_identifier(value))
}

fn list(values: impl IntoIterator<Item = Value>) -> Value {
    Value::Concrete(ConcreteValue::List(values.into_iter().collect()))
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

fn response_headers_policy_config(allow_methods: &[&str]) -> Value {
    map([
        ("name", string(POLICY_NAME)),
        ("comment", string(POLICY_COMMENT)),
        (
            "cors_config",
            map([
                ("access_control_allow_credentials", bool_(false)),
                (
                    "access_control_allow_headers",
                    map([("items", list([string("*")]))]),
                ),
                (
                    "access_control_allow_methods",
                    map([(
                        "items",
                        list(allow_methods.iter().map(|method| enum_identifier(method))),
                    )]),
                ),
                (
                    "access_control_allow_origins",
                    map([("items", list([string("https://example.com")]))]),
                ),
                ("origin_override", bool_(true)),
            ]),
        ),
        (
            "security_headers_config",
            map([
                (
                    "content_security_policy",
                    map([
                        ("content_security_policy", string(CONTENT_SECURITY_POLICY)),
                        ("override", bool_(true)),
                    ]),
                ),
                ("content_type_options", map([("override", bool_(true))])),
            ]),
        ),
    ])
}

fn response_headers_policy_resource(allow_methods: &[&str]) -> Resource {
    Resource::with_provider(
        "awscc",
        "cloudfront.ResponseHeadersPolicy",
        "registry_security_headers",
        None,
    )
    .with_attribute(
        "response_headers_policy_config",
        response_headers_policy_config(allow_methods),
    )
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

async fn stored_policy(cloudcontrol: &CloudControlService) -> (String, serde_json::Value) {
    let view = cloudcontrol.snapshot(ACCOUNT_ID, REGION).await;
    let resources = view
        .resources
        .values()
        .filter(|resource| resource.type_name == TYPE_NAME)
        .collect::<Vec<_>>();
    assert_eq!(
        resources.len(),
        1,
        "winterbaume should store exactly one ResponseHeadersPolicy resource"
    );
    let resource = resources[0];
    (
        resource.identifier.clone(),
        serde_json::from_str(&resource.resource_model)
            .expect("stored resource model should be JSON"),
    )
}

#[test]
fn response_headers_policy_id_matches_existing_cloudfront_id_reference_pattern() {
    let response_headers_policy = cloudfront_response_headers_policy_config();
    let response_headers_policy_id = response_headers_policy
        .schema
        .attributes()
        .get("id")
        .expect("ResponseHeadersPolicy.id should exist");
    assert!(response_headers_policy_id.is_read_only());

    let origin_access_control = cloudfront_origin_access_control_config();
    let origin_access_control_id = origin_access_control
        .schema
        .attributes()
        .get("id")
        .expect("OriginAccessControl.id should exist")
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
    let origin = list_element(find_field(distribution_config, "origins").field_type());

    for (label, sink) in [
        (
            "default_cache_behavior.response_headers_policy_id",
            find_field(default_cache_behavior, "response_headers_policy_id").field_type(),
        ),
        (
            "cache_behaviors[].response_headers_policy_id",
            find_field(cache_behavior, "response_headers_policy_id").field_type(),
        ),
    ] {
        assert!(
            response_headers_policy
                .schema
                .type_in_schema(response_headers_policy_id.attr_type())
                .is_assignable_to(distribution.schema.type_in_schema(sink)),
            "ResponseHeadersPolicy.id must be assignable to Distribution.{label}"
        );
    }

    let origin_access_control_sink = find_field(origin, "origin_access_control_id").field_type();
    assert!(
        origin_access_control
            .schema
            .type_in_schema(origin_access_control_id)
            .is_assignable_to(
                distribution
                    .schema
                    .type_in_schema(origin_access_control_sink)
            ),
        "the existing OriginAccessControl.id reference pattern should remain assignable"
    );
}

#[test]
fn response_headers_policy_schema_validates_nested_access_control_allow_methods() {
    let config = cloudfront_response_headers_policy_config();
    let schema = config
        .schema
        .validate()
        .expect("CloudFront response headers policy schema must pass admission");
    let policy_config = schema
        .effective_attribute("response_headers_policy_config")
        .expect("ResponseHeadersPolicy.response_headers_policy_config should exist");
    let valid = response_headers_policy_resource(&["get"]);
    let invalid = response_headers_policy_resource(&["GTE"]);
    let valid = valid.resolved_attributes();
    let invalid = invalid.resolved_attributes();

    assert!(
        policy_config
            .validate(
                valid
                    .get("response_headers_policy_config")
                    .expect("valid resource should contain response_headers_policy_config"),
            )
            .is_ok(),
        "whole-resource schema validation should accept the nested DSL enum value `get`"
    );
    assert!(
        policy_config
            .validate(
                invalid
                    .get("response_headers_policy_config")
                    .expect("invalid resource should contain response_headers_policy_config"),
            )
            .is_err(),
        "whole-resource schema validation should reject the nested unknown HTTP method `GTE`"
    );
}

#[tokio::test(start_paused = true)]
async fn response_headers_policy_create_then_read_preserves_nested_headers() {
    let (provider, cloudcontrol) = winterbaume_provider().await;
    let resource = response_headers_policy_resource(&["get", "head"]);
    let id = resource.id.clone();
    let request = common::create_request(resource).await;

    let created = carina_core::executor::dispatch_create(&provider, &id, request)
        .await
        .expect("cloudfront.ResponseHeadersPolicy create_resource should succeed");
    let created = match created.into_kind() {
        CreateOutcomeKind::Success { state } | CreateOutcomeKind::PartialSuccess { state, .. } => {
            state
        }
    };

    let (stored_identifier, desired_state) = stored_policy(&cloudcontrol).await;
    assert!(
        common::is_uuidish(&stored_identifier),
        "unregistered winterbaume identifier should be a synthetic UUID: {stored_identifier}"
    );
    assert_eq!(created.identifier().as_str(), stored_identifier);
    let stored_methods = desired_state["ResponseHeadersPolicyConfig"]["CorsConfig"]
        ["AccessControlAllowMethods"]["Items"]
        .as_array()
        .expect("CloudControl DesiredState should contain CORS allow methods");
    let mut stored_methods = stored_methods
        .iter()
        .map(|method| {
            method
                .as_str()
                .expect("a stored CORS allow method should be a string")
        })
        .collect::<Vec<_>>();
    stored_methods.sort_unstable();
    assert_eq!(
        stored_methods,
        vec!["GET", "HEAD"],
        "CloudControl DesiredState must use API-canonical HTTP method names"
    );
    assert_eq!(
        desired_state["ResponseHeadersPolicyConfig"]["SecurityHeadersConfig"],
        serde_json::json!({
            "ContentSecurityPolicy": {
                "ContentSecurityPolicy": CONTENT_SECURITY_POLICY,
                "Override": true
            },
            "ContentTypeOptions": {
                "Override": true
            }
        }),
        "CloudControl DesiredState must preserve both nested security-header structs"
    );
    assert!(
        desired_state["ResponseHeadersPolicyConfig"]["SecurityHeadersConfig"]
            ["ContentSecurityPolicy"]
            .is_object()
    );
    assert!(
        desired_state["ResponseHeadersPolicyConfig"]["SecurityHeadersConfig"]["ContentTypeOptions"]
            .is_object()
    );

    let stored_identifier = common::applied_identifier(stored_identifier);
    let read = Provider::read(&provider, &id, &stored_identifier, ReadRequest)
        .await
        .expect("cloudfront.ResponseHeadersPolicy read through Provider::read should succeed");
    let read = common::found_state(read);

    let Some(Value::Concrete(ConcreteValue::Map(policy_config))) =
        read.attributes().get("response_headers_policy_config")
    else {
        panic!("response_headers_policy_config must round-trip as a map");
    };
    assert_eq!(policy_config.get("name"), Some(&string(POLICY_NAME)));
    assert_eq!(policy_config.get("comment"), Some(&string(POLICY_COMMENT)));

    let Some(Value::Concrete(ConcreteValue::Map(cors_config))) = policy_config.get("cors_config")
    else {
        panic!("cors_config must round-trip as a map");
    };
    assert_eq!(
        cors_config.get("access_control_allow_credentials"),
        Some(&bool_(false))
    );
    assert_eq!(cors_config.get("origin_override"), Some(&bool_(true)));

    let Some(Value::Concrete(ConcreteValue::Map(allow_headers))) =
        cors_config.get("access_control_allow_headers")
    else {
        panic!("access_control_allow_headers must round-trip as a map");
    };
    assert_eq!(allow_headers.get("items"), Some(&list([string("*")])));

    let Some(Value::Concrete(ConcreteValue::Map(allow_origins))) =
        cors_config.get("access_control_allow_origins")
    else {
        panic!("access_control_allow_origins must round-trip as a map");
    };
    assert_eq!(
        allow_origins.get("items"),
        Some(&list([string("https://example.com")]))
    );

    let Some(Value::Concrete(ConcreteValue::Map(allow_methods))) =
        cors_config.get("access_control_allow_methods")
    else {
        panic!("access_control_allow_methods must round-trip as a map");
    };
    let Some(Value::Concrete(ConcreteValue::List(read_methods))) = allow_methods.get("items")
    else {
        panic!("access_control_allow_methods.items must round-trip as a list");
    };
    let response_headers_policy = cloudfront_response_headers_policy_config();
    let policy_config_type = response_headers_policy
        .schema
        .attributes()
        .get("response_headers_policy_config")
        .expect("ResponseHeadersPolicy.response_headers_policy_config should exist")
        .attr_type();
    let cors_config_type = find_field(policy_config_type, "cors_config").field_type();
    let allow_methods_type =
        find_field(cors_config_type, "access_control_allow_methods").field_type();
    let method_type = list_element(find_field(allow_methods_type, "items").field_type());
    let (_, _, _, _, method_dsl_map) = method_type
        .enum_parts()
        .expect("access_control_allow_methods.items elements should be enums");
    let mut read_methods = read_methods
        .iter()
        .map(|method| {
            let Value::Concrete(ConcreteValue::String(method)) = method else {
                panic!("a read-back CORS allow method should be a string, got {method:?}");
            };
            method_dsl_map
                .dsl_for(method.expose_plaintext())
                .into_owned()
        })
        .collect::<Vec<_>>();
    read_methods.sort_unstable();
    assert_eq!(
        read_methods,
        vec!["get", "head"],
        "read-back CORS allow methods should map to the DSL enum forms"
    );

    let Some(Value::Concrete(ConcreteValue::Map(security_headers))) =
        policy_config.get("security_headers_config")
    else {
        panic!("security_headers_config must round-trip as a map");
    };
    let Some(Value::Concrete(ConcreteValue::Map(content_security_policy))) =
        security_headers.get("content_security_policy")
    else {
        panic!("content_security_policy must round-trip as a map");
    };
    assert_eq!(
        content_security_policy.get("content_security_policy"),
        Some(&string(CONTENT_SECURITY_POLICY))
    );
    assert_eq!(content_security_policy.get("override"), Some(&bool_(true)));

    let Some(Value::Concrete(ConcreteValue::Map(content_type_options))) =
        security_headers.get("content_type_options")
    else {
        panic!("content_type_options must round-trip as a map");
    };
    assert_eq!(content_type_options.get("override"), Some(&bool_(true)));
}
