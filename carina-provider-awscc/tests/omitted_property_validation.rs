use std::collections::HashSet;
use std::sync::Arc;

use carina_core::parser::{ParsedFile, ProviderContext};
use carina_core::provider::{
    ProviderFactory, collect_schemas, provider_custom_types_from_factories,
};
use carina_core::resource::{
    ConcreteValue, ModuleSource, ParserPlaceholder, Resource as GenericResource,
    Value as GenericValue,
};
use carina_core::schema::SchemaRegistry;
use carina_core::validation::validate_resources;
use carina_provider_awscc::AwsccProviderFactory;
use indexmap::IndexMap;

type Resource = GenericResource<ModuleSource, ParserPlaceholder>;
type Value = GenericValue<ParserPlaceholder>;

fn string(value: &str) -> Value {
    Value::Concrete(ConcreteValue::String(value.to_string().into()))
}

fn bool_(value: bool) -> Value {
    Value::Concrete(ConcreteValue::Bool(value))
}

fn int(value: i64) -> Value {
    Value::Concrete(ConcreteValue::Int(value))
}

fn enum_identifier(value: &str) -> Value {
    Value::Concrete(ConcreteValue::enum_identifier(value))
}

fn map(entries: impl IntoIterator<Item = (&'static str, Value)>) -> Value {
    Value::Concrete(ConcreteValue::Map(
        entries
            .into_iter()
            .map(|(key, value)| (key.to_string(), value))
            .collect(),
    ))
}

fn empty_map() -> Value {
    Value::Concrete(ConcreteValue::Map(IndexMap::new()))
}

fn list(items: impl IntoIterator<Item = Value>) -> Value {
    Value::Concrete(ConcreteValue::List(items.into_iter().collect()))
}

fn validation_context() -> (SchemaRegistry, ProviderContext) {
    let factories: Arc<Vec<Box<dyn ProviderFactory>>> =
        Arc::new(vec![Box::new(AwsccProviderFactory)]);
    let schemas = collect_schemas(&factories);
    let context = ProviderContext {
        custom_types: provider_custom_types_from_factories(factories),
        resource_types: ProviderContext::resource_types_from_schema_registry(&schemas),
        ..ProviderContext::default()
    };
    (schemas, context)
}

fn assert_omitted_attribute_is_rejected(
    schemas: &SchemaRegistry,
    context: &ProviderContext,
    resource: Resource,
    omitted_attribute: &str,
) {
    let schema = schemas
        .get_for(&resource)
        .unwrap_or_else(|| panic!("missing generated schema for {}", resource.id.resource_type));
    assert!(
        schema.effective_attribute(omitted_attribute).is_none(),
        "{} must not expose omitted attribute/struct field {omitted_attribute}",
        resource.id.resource_type
    );

    let resource_id = resource.id.to_string();
    let mut parsed = ParsedFile::default();
    parsed.resources = vec![resource];
    let error = match validate_resources(
        &parsed,
        schemas,
        &HashSet::from(["awscc".to_string()]),
        context,
    ) {
        Err(error) => error,
        Ok(()) => {
            panic!("{resource_id}.{omitted_attribute} must be rejected, not silently dropped")
        }
    };
    let expected = format!("Unknown attribute '{omitted_attribute}'");
    assert!(
        error.contains(&expected),
        "{resource_id}.{omitted_attribute} must report {expected:?}, got: {error}"
    );
}

fn subscription_with(attribute: &'static str, value: Value) -> Resource {
    Resource::with_provider("awscc", "sns.Subscription", attribute, None)
        .with_attribute(
            "topic_arn",
            string("arn:aws:sns:ap-northeast-1:123456789012:alerts"),
        )
        .with_attribute("protocol", string("email"))
        .with_attribute("endpoint", string("alerts@example.com"))
        .with_attribute(attribute, value)
}

fn resource_with(resource_type: &'static str, attribute: &'static str, value: Value) -> Resource {
    Resource::with_provider("awscc", resource_type, attribute, None)
        .with_attribute(attribute, value)
}

#[test]
fn omitted_properties_are_unknown_attribute_errors_at_host_validation() {
    let (schemas, context) = validation_context();

    for attribute in [
        "delivery_policy",
        "filter_policy",
        "redrive_policy",
        "replay_policy",
    ] {
        assert_omitted_attribute_is_rejected(
            &schemas,
            &context,
            subscription_with(attribute, string(r#"{"event":["created"]}"#)),
            attribute,
        );
    }

    for attribute in ["archive_policy", "data_protection_policy", "subscription"] {
        assert_omitted_attribute_is_rejected(
            &schemas,
            &context,
            resource_with("sns.Topic", attribute, string(r#"{"Name":"example"}"#)),
            attribute,
        );
    }

    for attribute in ["data_protection_policy", "field_index_policies"] {
        assert_omitted_attribute_is_rejected(
            &schemas,
            &context,
            resource_with(
                "logs.LogGroup",
                attribute,
                string(r#"[{"Fields":["requestId"]}]"#),
            ),
            attribute,
        );
    }
}

fn visibility_config(metric_name: &str) -> Value {
    map([
        ("cloud_watch_metrics_enabled", bool_(false)),
        ("metric_name", string(metric_name)),
        ("sampled_requests_enabled", bool_(false)),
    ])
}

fn web_acl_with_uri_path(uri_path: Value) -> Resource {
    Resource::with_provider("awscc", "wafv2.WebAcl", "uri_path_marker", None)
        .with_attribute("name", string("carina-marker-validation"))
        .with_attribute("default_action", map([("allow", empty_map())]))
        .with_attribute(
            "rules",
            list([map([
                ("action", map([("count", empty_map())])),
                ("name", string("uri-path")),
                ("priority", int(0)),
                (
                    "statement",
                    map([(
                        "byte_match_statement",
                        map([
                            ("field_to_match", map([("uri_path", uri_path)])),
                            ("positional_constraint", enum_identifier("contains")),
                            ("search_string", string("health")),
                            (
                                "text_transformations",
                                list([map([
                                    ("priority", int(0)),
                                    ("type", enum_identifier("none")),
                                ])]),
                            ),
                        ]),
                    )]),
                ),
                ("visibility_config", visibility_config("uriPathRule")),
            ])]),
        )
        .with_attribute("scope", enum_identifier("regional"))
        .with_attribute("visibility_config", visibility_config("uriPathWebAcl"))
}

fn validate_resource(
    schemas: &SchemaRegistry,
    context: &ProviderContext,
    resource: Resource,
) -> Result<(), String> {
    let mut parsed = ParsedFile::default();
    parsed.resources = vec![resource];
    validate_resources(
        &parsed,
        schemas,
        &HashSet::from(["awscc".to_string()]),
        context,
    )
}

#[test]
fn waf_empty_marker_accepts_only_the_empty_struct() {
    let (schemas, context) = validation_context();

    validate_resource(&schemas, &context, web_acl_with_uri_path(empty_map()))
        .expect("uri_path = {} must validate as an empty marker struct");

    let error = validate_resource(
        &schemas,
        &context,
        web_acl_with_uri_path(map([("foo", string("x"))])),
    )
    .expect_err("a non-empty uri_path marker must be rejected");
    assert!(
        error.contains("foo"),
        "unknown field must be named: {error}"
    );
    assert!(
        error.to_lowercase().contains("unknown"),
        "marker must fail as an unknown field: {error}"
    );
}

fn web_acl_with_custom_response_bodies(custom_response_bodies: Value) -> Resource {
    Resource::with_provider("awscc", "wafv2.WebAcl", "custom_response_bodies", None)
        .with_attribute("name", string("carina-custom-response-validation"))
        .with_attribute("custom_response_bodies", custom_response_bodies)
        .with_attribute("default_action", map([("allow", empty_map())]))
        .with_attribute("scope", enum_identifier("regional"))
        .with_attribute(
            "visibility_config",
            visibility_config("customResponseBodiesWebAcl"),
        )
}

#[test]
fn waf_custom_response_bodies_accepts_struct_values_and_rejects_strings() {
    let (schemas, context) = validation_context();
    let valid = map([(
        "marker_response",
        map([
            ("content", string("blocked by the marker fixture")),
            ("content_type", enum_identifier("text_plain")),
        ]),
    )]);
    validate_resource(
        &schemas,
        &context,
        web_acl_with_custom_response_bodies(valid),
    )
    .expect("custom_response_bodies must accept Map<String, CustomResponseBody>");

    let error = validate_resource(
        &schemas,
        &context,
        web_acl_with_custom_response_bodies(string("not a response-body map")),
    )
    .expect_err("custom_response_bodies must reject a scalar string");
    assert!(
        error.contains("custom_response_bodies"),
        "validation error must name custom_response_bodies: {error}"
    );
}
