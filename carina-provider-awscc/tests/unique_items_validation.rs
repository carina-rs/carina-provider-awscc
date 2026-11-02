use std::collections::HashSet;
use std::sync::Arc;

use carina_core::parser::{ParsedFile, ProviderContext};
use carina_core::provider::{
    ProviderFactory, collect_schemas, provider_custom_types_from_factories,
};
use carina_core::resource::{AuthoredResource as Resource, AuthoredValue as Value, ConcreteValue};
use carina_core::schema::Shape;
use carina_core::validation::validate_resources;
use carina_provider_awscc::AwsccProviderFactory;

fn string(value: &str) -> Value {
    Value::Concrete(ConcreteValue::String(value.to_string().into()))
}

fn list(items: impl IntoIterator<Item = Value>) -> Value {
    Value::Concrete(ConcreteValue::List(items.into_iter().collect()))
}

fn map(entries: impl IntoIterator<Item = (&'static str, Value)>) -> Value {
    Value::Concrete(ConcreteValue::Map(
        entries
            .into_iter()
            .map(|(key, value)| (key.to_string(), value))
            .collect(),
    ))
}

fn resource_with(resource_type: &str, attribute: &str, value: Value) -> Resource {
    Resource::with_provider("awscc", resource_type, "unique_items", None)
        .with_attribute(attribute, value)
}

fn validate(resource: Resource) -> Result<(), String> {
    let factories: Arc<Vec<Box<dyn ProviderFactory>>> =
        Arc::new(vec![Box::new(AwsccProviderFactory)]);
    let schemas = collect_schemas(&factories);
    let context = ProviderContext {
        custom_types: provider_custom_types_from_factories(factories),
        resource_types: ProviderContext::resource_types_from_schema_registry(&schemas),
        ..ProviderContext::default()
    };
    let mut parsed = ParsedFile::default();
    parsed.resources = vec![resource];
    validate_resources(
        &parsed,
        &schemas,
        &HashSet::from(["awscc".to_string()]),
        &context,
    )
}

fn assert_duplicate(resource: Resource, attribute: &str) {
    let error = validate(resource).expect_err("duplicate authored list elements must be rejected");
    assert!(
        error.contains("Duplicate list element")
            && error.contains(attribute)
            && error.contains("index 1 (first at index 0)"),
        "expected a duplicate error tied to {attribute}, got: {error}"
    );
}

#[test]
fn unique_string_list_rejects_duplicates_and_accepts_distinct_values() {
    let load_balancer = |subnets| {
        resource_with(
            "elasticloadbalancingv2.LoadBalancer",
            "subnets",
            list(subnets),
        )
    };
    let first = string("subnet-0123456789abcdef0");
    let second = string("subnet-0123456789abcdef1");
    validate(load_balancer([first.clone(), second])).expect("distinct subnets must validate");
    assert_duplicate(load_balancer([first.clone(), first]), "subnets");
}

#[test]
fn unique_struct_list_compares_whole_values_independently_of_field_order() {
    let target_group = |attributes| {
        resource_with(
            "elasticloadbalancingv2.TargetGroup",
            "target_group_attributes",
            list(attributes),
        )
    };
    let first = map([
        ("key", string("deregistration_delay.timeout_seconds")),
        ("value", string("30")),
    ]);
    // Equality covers the entire object, rather than a provider-specific key.
    let distinct = map([
        ("key", string("deregistration_delay.timeout_seconds")),
        ("value", string("60")),
    ]);
    validate(target_group([first.clone(), distinct])).expect("distinct structs must validate");
    let reordered_duplicate = map([
        ("value", string("30")),
        ("key", string("deregistration_delay.timeout_seconds")),
    ]);
    assert_duplicate(
        target_group([first, reordered_duplicate]),
        "target_group_attributes",
    );
}

#[test]
fn nested_refined_unique_list_preserves_bounds_and_rejects_duplicates() {
    let alarm = |keys: Vec<Value>| {
        resource_with(
            "cloudwatch.Alarm",
            "action_configuration",
            map([(
                "alarm_tags_propagation",
                map([
                    ("enabled", Value::Concrete(ConcreteValue::Bool(true))),
                    ("included_tag_keys", list(keys)),
                ]),
            )]),
        )
    };
    validate(alarm(vec![string("team"), string("environment")]))
        .expect("distinct tag keys within the item bounds must validate");
    assert_duplicate(
        alarm(vec![string("team"), string("team")]),
        "included_tag_keys",
    );
    let error = validate(alarm(vec![])).expect_err("the existing minItems must still be enforced");
    assert!(error.contains("included_tag_keys"), "{error}");
}

#[test]
fn nested_unique_enum_list_rejects_duplicates_and_accepts_distinct_values() {
    let bucket =
        |methods: [&str; 2]| {
            resource_with(
                "s3.Bucket",
                "cors_configuration",
                map([(
                    "cors_rules",
                    list([map([
                        (
                            "allowed_methods",
                            list(methods.map(|method| {
                                Value::Concrete(ConcreteValue::enum_identifier(method))
                            })),
                        ),
                        ("allowed_origins", list([string("https://example.com")])),
                    ])]),
                )]),
            )
        };
    validate(bucket(["get", "put"])).expect("distinct enum elements must validate");
    assert_duplicate(bucket(["get", "get"]), "allowed_methods");
}

#[test]
fn unique_list_with_overridden_struct_items_rejects_duplicates() {
    let log_group =
        |policies| resource_with("logs.LogGroup", "field_index_policies", list(policies));
    let first = map([("fields", list([string("requestId")]))]);
    let second = map([("fields", list([string("accountId")]))]);
    validate(log_group([first.clone(), second])).expect("distinct field index policies validate");
    assert_duplicate(log_group([first.clone(), first]), "field_index_policies");
}

#[test]
fn lists_without_unique_items_still_accept_duplicates() {
    for (resource_type, attribute, value) in [
        ("ecs.Cluster", "capacity_providers", "FARGATE"),
        ("iam.OidcProvider", "client_id_list", "sts.amazonaws.com"),
    ] {
        validate(resource_with(
            resource_type,
            attribute,
            list([string(value), string(value)]),
        ))
        .unwrap_or_else(|error| panic!("{resource_type}.{attribute} allows duplicates: {error}"));
    }
}

#[test]
fn seven_unique_tag_lists_remain_string_maps() {
    let schemas = carina_provider_awscc::schemas::schema_registry();
    for resource_type in [
        "cloudwatch.Alarm",
        "ec2.Ipam",
        "ec2.IpamPool",
        "ecr.Repository",
        "kms.Key",
        "logs.LogGroup",
        "organizations.Account",
    ] {
        let resource = Resource::with_provider("awscc", resource_type, "tag_map", None);
        let schema = schemas
            .get_for(&resource)
            .expect("real generated schema")
            .schema()
            .as_schema();
        let tags = schema.attributes().get("tags").expect("generated tags");
        let Shape::Map { key, value } = schema.shape_of(tags.attr_type()) else {
            panic!("{resource_type}.tags must retain its map shape");
        };
        assert!(matches!(schema.shape_of(key), Shape::String { .. }));
        assert!(matches!(schema.shape_of(value), Shape::String { .. }));
    }
}
