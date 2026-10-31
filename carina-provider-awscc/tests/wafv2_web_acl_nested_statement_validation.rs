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
use carina_core::schema::{
    ReferenceForms, SchemaRegistry, ValidatedConfigSchema, ValidatedProviderSchemaWitness,
};
use carina_core::validation::validate_resources;
use carina_provider_awscc::schemas::generated::wafv2::web_acl::wafv2_web_acl_config;
use indexmap::IndexMap;

type Resource = GenericResource<ModuleSource, ParserPlaceholder>;
type Value = GenericValue<ParserPlaceholder>;

struct Wafv2SchemaFactory;

impl ProviderFactory for Wafv2SchemaFactory {
    fn name(&self) -> &str {
        "awscc"
    }

    fn display_name(&self) -> &str {
        "AWS Cloud Control WAFv2 schema test factory"
    }

    fn provider_config_attribute_types(&self) -> carina_core::schema::ValidatedConfigSchema {
        carina_core::schema::ValidatedConfigSchema::default()
    }

    fn validate_config(
        &self,
        _attributes: &carina_core::provider::CanonicalProviderConfigAttributes,
    ) -> carina_core::provider::ProviderResult<()> {
        Ok(())
    }

    fn extract_region(
        &self,
        _attributes: &IndexMap<String, carina_core::resource::Value>,
    ) -> String {
        "ap-northeast-1".to_string()
    }

    fn create_provider(
        &self,
        _binding: Option<&str>,
        _config: &carina_core::provider::ProviderReadyConfig,
        _initialize_outcome: carina_core::provider::InitializeOutcomeMint,
    ) -> carina_core::provider::BoxFuture<
        '_,
        carina_core::provider::ProviderResult<carina_core::provider::ProviderInitialization>,
    > {
        Box::pin(async {
            Err(carina_core::provider::ProviderError::internal(
                "schema-only test factory cannot create providers",
            ))
        })
    }

    fn schemas(&self) -> Vec<carina_core::schema::ValidatedResourceSchema> {
        vec![
            wafv2_web_acl_config()
                .schema
                .validate()
                .expect("WebACL schema must be valid"),
        ]
    }

    fn schema_witness(&self) -> ValidatedProviderSchemaWitness {
        ValidatedProviderSchemaWitness::new(
            self.schemas(),
            ValidatedConfigSchema::default(),
            ReferenceForms::new(),
        )
        .expect("WebACL schema-only witness must be valid")
    }
}

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

fn visibility_config(metric_name: &str) -> Value {
    map([
        ("cloud_watch_metrics_enabled", bool_(false)),
        ("metric_name", string(metric_name)),
        ("sampled_requests_enabled", bool_(false)),
    ])
}

fn byte_match_statement(search_string: &str) -> Value {
    map([
        (
            "field_to_match",
            map([("single_header", map([("name", string("user-agent"))]))]),
        ),
        ("positional_constraint", enum_identifier("contains")),
        ("search_string", string(search_string)),
        (
            "text_transformations",
            list([map([
                ("priority", int(0)),
                ("type", enum_identifier("none")),
            ])]),
        ),
    ])
}

fn nested_statement(geo_match_statement: Value) -> Value {
    map([(
        "and_statement",
        map([(
            "statements",
            list([
                map([("byte_match_statement", byte_match_statement("outer-bot"))]),
                map([(
                    "not_statement",
                    map([(
                        "statement",
                        map([(
                            "or_statement",
                            map([(
                                "statements",
                                list([
                                    map([("geo_match_statement", geo_match_statement)]),
                                    map([(
                                        "byte_match_statement",
                                        byte_match_statement("inner-bot"),
                                    )]),
                                ]),
                            )]),
                        )]),
                    )]),
                )]),
            ]),
        )]),
    )])
}

fn web_acl(geo_match_statement: Value) -> Resource {
    Resource::with_provider("awscc", "wafv2.WebAcl", "nested_statements", None)
        .with_attribute("name", string("carina-nested-statements"))
        .with_attribute("default_action", map([("allow", empty_map())]))
        .with_attribute(
            "rules",
            list([map([
                ("action", map([("count", empty_map())])),
                ("name", string("nested-statements")),
                ("priority", int(0)),
                ("statement", nested_statement(geo_match_statement)),
                ("visibility_config", visibility_config("nestedRule")),
            ])]),
        )
        .with_attribute("scope", enum_identifier("regional"))
        .with_attribute("visibility_config", visibility_config("nestedWebAcl"))
}

fn validate_web_acl(resource: Resource) -> Result<(), String> {
    let factories: Arc<Vec<Box<dyn ProviderFactory>>> =
        Arc::new(vec![Box::new(Wafv2SchemaFactory)]);
    let schemas: SchemaRegistry = collect_schemas(&factories);
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

#[test]
fn nested_waf_statements_validate_through_the_user_resource_path() {
    let valid_geo_match = map([("country_codes", list([string("US")]))]);
    validate_web_acl(web_acl(valid_geo_match))
        .expect("nested and/not/or statements must pass whole-resource validation");

    let unknown_field = "unknown_three_levels_deep";
    let unknown_geo_match = map([
        ("country_codes", list([string("US")])),
        (unknown_field, bool_(true)),
    ]);
    let unknown_error = validate_web_acl(web_acl(unknown_geo_match))
        .expect_err("an unknown field inside the nested geo_match_statement must be rejected");
    assert!(
        unknown_error.contains(unknown_field),
        "the nested unknown field should be identified: {unknown_error}"
    );

    let wrong_type_geo_match = map([("country_codes", string("US"))]);
    let wrong_type_error = validate_web_acl(web_acl(wrong_type_geo_match))
        .expect_err("a wrong-typed field inside the nested geo_match_statement must be rejected");
    assert!(
        wrong_type_error.contains("country_codes"),
        "the nested wrong-typed field should be identified: {wrong_type_error}"
    );
}
