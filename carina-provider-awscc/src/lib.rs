//! Carina AWS Cloud Control Provider
//!
//! AWS Cloud Control API Provider implementation.
//!
//! ## Module Structure
//!
//! - `resources` - Schema configuration tests
//! - `provider` - AwsccProvider implementation
//! - `schemas` - Auto-generated resource schemas

pub mod provider;
pub mod resources;
pub mod schemas;

// Re-export main types
pub use provider::AwsccProvider;

use std::collections::HashMap;

use indexmap::IndexMap;

use carina_core::provider::{
    BoxFuture, CreateRequest, DeleteRequest, Provider, ProviderError, ProviderFactory,
    ProviderNormalizer, ProviderResult, ReadRequest, SavedAttrs, UpdateRequest,
    merge_default_tags_for_provider,
};
use carina_core::resource::{ConcreteValue, Resource, ResourceId, State, Value};
use carina_core::schema::SchemaRegistry;

use crate::provider::AwsccProviderConfig;

/// Schema extension for the AWSCC provider.
///
/// Handles plan-time normalization of enum identifiers and hydration of
/// unreturned attributes from saved state.
pub struct AwsccNormalizer;

impl ProviderNormalizer for AwsccNormalizer {
    fn normalize_desired(&self, resources: &mut [Resource]) {
        crate::provider::resolve_enum_identifiers_impl(resources);
    }

    fn normalize_state(&self, current_states: &mut HashMap<ResourceId, State>) {
        crate::provider::normalize_state_enums_impl(current_states);
        // Canonicalize `Union[String, list(String)]` typed values
        // (IAM-style `string_or_list_of_strings`) so AWS's silent
        // scalar normalization no longer leaks past the provider
        // boundary. See carina-rs/carina#2481, sub-issue 5.
        crate::provider::canonicalize_string_or_list_states_impl(current_states);
    }

    fn hydrate_read_state(
        &self,
        current_states: &mut HashMap<ResourceId, State>,
        saved_attrs: &SavedAttrs,
    ) {
        crate::provider::restore_unreturned_attrs_impl(current_states, saved_attrs);
    }

    fn merge_default_tags(
        &self,
        resources: &mut [Resource],
        default_tags: &IndexMap<String, Value>,
        registry: &SchemaRegistry,
    ) {
        merge_default_tags_for_provider("awscc", resources, default_tags, registry);
    }
}

/// Factory for creating and configuring the AWSCC Provider
pub struct AwsccProviderFactory;

impl ProviderFactory for AwsccProviderFactory {
    fn name(&self) -> &str {
        "awscc"
    }

    fn display_name(&self) -> &str {
        "AWS Cloud Control provider"
    }

    fn provider_config_attribute_types(
        &self,
    ) -> HashMap<String, carina_core::schema::AttributeType> {
        use carina_core::schema::AttributeType;
        let mut types = HashMap::new();
        types.insert(
            "region".to_string(),
            AttributeType::StringEnum {
                name: "Region".to_string(),
                values: carina_aws_types::REGIONS
                    .iter()
                    .map(|(code, _)| code.to_string())
                    .collect(),
                namespace: Some("awscc".to_string()),
                // Region API spellings carry hyphens (`ap-northeast-1`)
                // but the DSL spelling uses underscores
                // (`ap_northeast_1`). Materialize the alias table so it
                // is data, not a `fn` pointer — the latter cannot cross
                // the WASM boundary (carina#2831).
                dsl_aliases: carina_aws_types::region_dsl_aliases(),
            },
        );
        types.insert(
            "allowed_account_ids".to_string(),
            AttributeType::List {
                inner: Box::new(AttributeType::String),
                ordered: false,
            },
        );
        types.insert(
            "forbidden_account_ids".to_string(),
            AttributeType::List {
                inner: Box::new(AttributeType::String),
                ordered: false,
            },
        );
        types
    }

    fn validate_config(&self, _attributes: &IndexMap<String, Value>) -> Result<(), String> {
        // Region format/value validation is handled by the host via
        // `provider_config_attribute_types`. No provider-specific semantic
        // checks are needed beyond that for now.
        Ok(())
    }

    fn validate_custom_type(&self, type_name: &str, value: &str) -> Result<(), String> {
        use carina_core::parser::ValidatorFn;
        use std::sync::OnceLock;
        static VALIDATORS: OnceLock<HashMap<String, ValidatorFn>> = OnceLock::new();
        let validators = VALIDATORS.get_or_init(schemas::awscc_types::awscc_validators);
        if let Some(validator) = validators.get(type_name) {
            validator(value)
        } else {
            Ok(())
        }
    }

    fn extract_region(&self, attributes: &IndexMap<String, Value>) -> String {
        if let Some(Value::Concrete(ConcreteValue::String(region))) = attributes.get("region") {
            return carina_core::utils::convert_region_value(region);
        }
        "ap-northeast-1".to_string()
    }

    fn create_provider(
        &self,
        _binding: Option<&str>,
        attributes: &IndexMap<String, Value>,
    ) -> BoxFuture<'_, Result<Box<dyn Provider>, carina_core::provider::ProviderError>> {
        // `_binding` is intentionally unused: the AWS Cloud Control
        // factory does not cache instances, so each call already
        // produces an independent `AwsccProvider`. The host uses the
        // binding name as a cache key in `WasmProviderFactory`; for
        // in-process factories the constructed-fresh shape is enough.
        let region = self.extract_region(attributes);
        let cfg = extract_provider_config(attributes);
        Box::pin(async move {
            Ok(Box::new(AwsccProvider::new_with_config(&region, &cfg).await) as Box<dyn Provider>)
        })
    }

    fn create_normalizer(
        &self,
        _binding: Option<&str>,
        _attributes: &IndexMap<String, Value>,
    ) -> BoxFuture<'_, Box<dyn ProviderNormalizer>> {
        Box::pin(async { Box::new(AwsccNormalizer) as Box<dyn ProviderNormalizer> })
    }

    fn schemas(&self) -> Vec<carina_core::schema::ResourceSchema> {
        schemas::all_schemas()
    }

    fn identity_attributes(&self) -> Vec<&str> {
        vec!["region"]
    }

    fn config_completions(
        &self,
    ) -> std::collections::HashMap<String, Vec<carina_core::schema::CompletionValue>> {
        std::collections::HashMap::from([(
            "region".to_string(),
            carina_aws_types::region_completions("awscc"),
        )])
    }

    // The default `Provider::get_enum_alias_reverse` returning `None`
    // is fine — DSL → API canonical lookup is done via
    // `DslMap::api_for` in `dsl_value_to_aws`, sourced from the
    // exhaustive `dsl_aliases` table on each StringEnum. The legacy
    // string-keyed dispatch (`(resource_type, attr_name)`) was
    // removed in awscc#223 / awscc#220.
}

// =============================================================================
// Provider Trait Implementation
// =============================================================================

/// Extract the account-guard policy from a provider's configuration
/// attributes. Treats unset / non-list / non-string-element values as
/// "absent", consistent with the schema declared in
/// `provider_config_attribute_types` — the host enforces the declared
/// types before reaching the provider, so this is a defensive parse.
pub(crate) fn extract_provider_config(attributes: &IndexMap<String, Value>) -> AwsccProviderConfig {
    fn extract_string_list(attributes: &IndexMap<String, Value>, key: &str) -> Vec<String> {
        match attributes.get(key) {
            Some(Value::Concrete(ConcreteValue::List(items))) => items
                .iter()
                .filter_map(|v| match v {
                    Value::Concrete(ConcreteValue::String(s)) => Some(s.clone()),
                    _ => None,
                })
                .collect(),
            _ => Vec::new(),
        }
    }
    AwsccProviderConfig {
        allowed_account_ids: extract_string_list(attributes, "allowed_account_ids"),
        forbidden_account_ids: extract_string_list(attributes, "forbidden_account_ids"),
    }
}

impl Provider for AwsccProvider {
    fn name(&self) -> &str {
        "awscc"
    }

    fn read(
        &self,
        id: &ResourceId,
        identifier: Option<&str>,
        _request: ReadRequest,
    ) -> BoxFuture<'_, ProviderResult<State>> {
        if let Some(err) = self.init_error() {
            let err = err.to_string();
            let id = id.clone();
            return Box::pin(async move {
                Err(ProviderError::invalid_input(err)
                    .for_provider("awscc")
                    .for_resource(id))
            });
        }
        let id = id.clone();
        let identifier = identifier.map(|s| s.to_string());
        Box::pin(async move {
            self.read_resource(&id.resource_type, id.name_str(), identifier.as_deref())
                .await
        })
    }

    fn read_data_source(&self, resource: &Resource) -> BoxFuture<'_, ProviderResult<State>> {
        if let Some(err) = self.init_error() {
            let err = err.to_string();
            let id = resource.id.clone();
            return Box::pin(async move {
                Err(ProviderError::invalid_input(err)
                    .for_provider("awscc")
                    .for_resource(id))
            });
        }
        let id = resource.id.clone();
        Box::pin(async move {
            self.read_resource(&id.resource_type, id.name_str(), None)
                .await
        })
    }

    fn create(
        &self,
        _id: &ResourceId,
        request: CreateRequest,
    ) -> BoxFuture<'_, ProviderResult<State>> {
        if let Some(err) = self.init_error() {
            let err = err.to_string();
            let id = request.resource.id.clone();
            return Box::pin(async move {
                Err(ProviderError::invalid_input(err)
                    .for_provider("awscc")
                    .for_resource(id))
            });
        }
        let resource = request.resource;
        Box::pin(async move { self.create_resource(resource).await })
    }

    fn update(
        &self,
        id: &ResourceId,
        identifier: &str,
        request: UpdateRequest,
    ) -> BoxFuture<'_, ProviderResult<State>> {
        if let Some(err) = self.init_error() {
            let err = err.to_string();
            let id = id.clone();
            return Box::pin(async move {
                Err(ProviderError::invalid_input(err)
                    .for_provider("awscc")
                    .for_resource(id))
            });
        }
        let id = id.clone();
        let identifier = identifier.to_string();
        Box::pin(async move {
            self.update_resource(id, &identifier, &request.from, &request.patch)
                .await
        })
    }

    fn delete(
        &self,
        id: &ResourceId,
        identifier: &str,
        request: DeleteRequest,
    ) -> BoxFuture<'_, ProviderResult<()>> {
        if let Some(err) = self.init_error() {
            let err = err.to_string();
            let id = id.clone();
            return Box::pin(async move {
                Err(ProviderError::invalid_input(err)
                    .for_provider("awscc")
                    .for_resource(id))
            });
        }
        let id = id.clone();
        let identifier = identifier.to_string();
        Box::pin(async move {
            self.delete_resource(&id, &identifier, &request.directives)
                .await
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn build_schemas() -> SchemaRegistry {
        let factory = AwsccProviderFactory;
        let mut registry = SchemaRegistry::new();
        for schema in factory.schemas() {
            registry.insert("awscc", schema);
        }
        registry
    }

    #[test]
    fn extract_provider_config_reads_both_lists() {
        let mut attrs: IndexMap<String, Value> = IndexMap::new();
        attrs.insert(
            "allowed_account_ids".to_string(),
            Value::Concrete(ConcreteValue::List(vec![
                Value::Concrete(ConcreteValue::String("111111111111".to_string())),
                Value::Concrete(ConcreteValue::String("222222222222".to_string())),
            ])),
        );
        attrs.insert(
            "forbidden_account_ids".to_string(),
            Value::Concrete(ConcreteValue::List(vec![Value::Concrete(
                ConcreteValue::String("999999999999".to_string()),
            )])),
        );

        let cfg = extract_provider_config(&attrs);
        assert_eq!(
            cfg.allowed_account_ids,
            vec!["111111111111".to_string(), "222222222222".to_string()]
        );
        assert_eq!(cfg.forbidden_account_ids, vec!["999999999999".to_string()]);
    }

    #[test]
    fn extract_provider_config_defaults_to_empty_when_unset() {
        let attrs: IndexMap<String, Value> = IndexMap::new();
        let cfg = extract_provider_config(&attrs);
        assert!(cfg.allowed_account_ids.is_empty());
        assert!(cfg.forbidden_account_ids.is_empty());
    }

    #[test]
    fn provider_config_attribute_types_declares_account_id_lists() {
        let factory = AwsccProviderFactory;
        let types = factory.provider_config_attribute_types();
        assert!(matches!(
            types.get("allowed_account_ids"),
            Some(carina_core::schema::AttributeType::List { .. })
        ));
        assert!(matches!(
            types.get("forbidden_account_ids"),
            Some(carina_core::schema::AttributeType::List { .. })
        ));
    }

    /// carina#3093 acceptance: the *generated* CloudFront Distribution
    /// schema must type every `allowed_methods` / `cached_methods`
    /// (nested under `distribution_config.{default_cache_behavior,
    /// cache_behaviors[]}`) as an **unordered** list. With
    /// `ordered: false`, carina-core's `type_aware_equal` compares
    /// these as multisets, so a provider-read order differing from the
    /// DSL-authored order is no longer a never-converging phantom diff.
    /// Walking the real schema (not just asserting the override kind)
    /// is the apply-path-faithful check — it proves the override
    /// actually reached the emitted `AttributeType`.
    #[test]
    fn cloudfront_distribution_methods_are_unordered_lists() {
        use carina_core::schema::AttributeType;

        let factory = AwsccProviderFactory;
        let schema = factory
            .schemas()
            .into_iter()
            .find(|s| s.resource_type == "cloudfront.Distribution")
            .expect("cloudfront.Distribution schema must be present");

        // Recursively collect the `ordered` flag of every List whose
        // owning struct field is named allowed_methods/cached_methods.
        fn collect(at: &AttributeType, field: Option<&str>, out: &mut Vec<(String, bool)>) {
            match at {
                AttributeType::List { inner, ordered } => {
                    if matches!(field, Some("allowed_methods" | "cached_methods")) {
                        out.push((field.unwrap().to_string(), *ordered));
                    }
                    collect(inner, None, out);
                }
                AttributeType::Struct { fields, .. } => {
                    for f in fields {
                        collect(&f.field_type, Some(f.name.as_str()), out);
                    }
                }
                AttributeType::Map { value, .. } => collect(value, None, out),
                _ => {}
            }
        }

        let mut found = Vec::new();
        for attr in schema.attributes.values() {
            collect(&attr.attr_type, None, &mut found);
        }

        assert!(
            !found.is_empty(),
            "expected to find allowed_methods/cached_methods list fields in the schema"
        );
        for (name, ordered) in &found {
            assert!(
                !ordered,
                "carina#3093: `{name}` must be an unordered_list \
                 (ordered=false); got ordered={ordered}"
            );
        }
        // Sanity: both field names are actually present (default +
        // per-behavior cache configs each carry both).
        assert!(found.iter().any(|(n, _)| n == "allowed_methods"));
        assert!(found.iter().any(|(n, _)| n == "cached_methods"));
    }

    #[test]
    fn test_merge_default_tags_resource_tags_win() {
        let schemas = build_schemas();
        let normalizer = AwsccNormalizer;

        let mut resource = Resource::with_provider("awscc", "ec2.Vpc", "test-vpc", None);
        resource.set_attr(
            "cidr_block".to_string(),
            Value::Concrete(ConcreteValue::String("10.0.0.0/16".to_string())),
        );
        let mut resource_tags: IndexMap<String, Value> = IndexMap::new();
        resource_tags.insert(
            "Name".to_string(),
            Value::Concrete(ConcreteValue::String("my-vpc".to_string())),
        );
        resource_tags.insert(
            "Environment".to_string(),
            Value::Concrete(ConcreteValue::String("staging".to_string())),
        );
        resource.set_attr(
            "tags".to_string(),
            Value::Concrete(ConcreteValue::Map(resource_tags)),
        );

        let mut default_tags: IndexMap<String, Value> = IndexMap::new();
        default_tags.insert(
            "Environment".to_string(),
            Value::Concrete(ConcreteValue::String("production".to_string())),
        );
        default_tags.insert(
            "Team".to_string(),
            Value::Concrete(ConcreteValue::String("platform".to_string())),
        );

        let mut resources = vec![resource];
        normalizer.merge_default_tags(&mut resources, &default_tags, &schemas);

        if let Some(Value::Concrete(ConcreteValue::Map(tags))) = resources[0].get_attr("tags") {
            assert_eq!(
                tags.get("Environment"),
                Some(&Value::Concrete(ConcreteValue::String(
                    "staging".to_string()
                )))
            );
            assert_eq!(
                tags.get("Name"),
                Some(&Value::Concrete(ConcreteValue::String(
                    "my-vpc".to_string()
                )))
            );
            assert_eq!(
                tags.get("Team"),
                Some(&Value::Concrete(ConcreteValue::String(
                    "platform".to_string()
                )))
            );
        } else {
            panic!("Expected tags to be a Map");
        }

        if let Some(Value::Concrete(ConcreteValue::List(keys))) =
            resources[0].get_attr("_default_tag_keys")
        {
            let key_strs: Vec<&str> = keys
                .iter()
                .filter_map(|v| match v {
                    Value::Concrete(ConcreteValue::String(s)) => Some(s.as_str()),
                    _ => None,
                })
                .collect();
            assert_eq!(key_strs, vec!["Team"]);
        } else {
            panic!("Expected _default_tag_keys to be set");
        }
    }

    #[test]
    fn test_merge_default_tags_no_explicit_tags() {
        let schemas = build_schemas();
        let normalizer = AwsccNormalizer;

        let mut resource = Resource::with_provider("awscc", "ec2.Vpc", "test-vpc", None);
        resource.set_attr(
            "cidr_block".to_string(),
            Value::Concrete(ConcreteValue::String("10.0.0.0/16".to_string())),
        );

        let mut default_tags: IndexMap<String, Value> = IndexMap::new();
        default_tags.insert(
            "Environment".to_string(),
            Value::Concrete(ConcreteValue::String("production".to_string())),
        );

        let mut resources = vec![resource];
        normalizer.merge_default_tags(&mut resources, &default_tags, &schemas);

        if let Some(Value::Concrete(ConcreteValue::Map(tags))) = resources[0].get_attr("tags") {
            assert_eq!(
                tags.get("Environment"),
                Some(&Value::Concrete(ConcreteValue::String(
                    "production".to_string()
                )))
            );
        } else {
            panic!("Expected tags to be set from default_tags");
        }

        if let Some(Value::Concrete(ConcreteValue::List(keys))) =
            resources[0].get_attr("_default_tag_keys")
        {
            let key_strs: Vec<&str> = keys
                .iter()
                .filter_map(|v| match v {
                    Value::Concrete(ConcreteValue::String(s)) => Some(s.as_str()),
                    _ => None,
                })
                .collect();
            assert_eq!(key_strs, vec!["Environment"]);
        } else {
            panic!("Expected _default_tag_keys to be set");
        }
    }

    #[test]
    fn test_merge_default_tags_skips_no_tag_schema() {
        let schemas = build_schemas();
        let normalizer = AwsccNormalizer;

        let mut resource = Resource::with_provider("awscc", "ec2.Route", "test-route", None);
        resource.set_attr(
            "route_table_id".to_string(),
            Value::Concrete(ConcreteValue::String("rtb-123".to_string())),
        );

        let mut default_tags: IndexMap<String, Value> = IndexMap::new();
        default_tags.insert(
            "Environment".to_string(),
            Value::Concrete(ConcreteValue::String("production".to_string())),
        );

        let mut resources = vec![resource];
        normalizer.merge_default_tags(&mut resources, &default_tags, &schemas);

        assert!(!resources[0].attributes.contains_key("tags"));
        assert!(!resources[0].attributes.contains_key("_default_tag_keys"));
    }

    #[test]
    fn test_merge_default_tags_no_default_tags() {
        let schemas = build_schemas();
        let normalizer = AwsccNormalizer;

        let mut resource = Resource::with_provider("awscc", "ec2.Vpc", "test-vpc", None);
        resource.set_attr(
            "cidr_block".to_string(),
            Value::Concrete(ConcreteValue::String("10.0.0.0/16".to_string())),
        );
        let mut resource_tags: IndexMap<String, Value> = IndexMap::new();
        resource_tags.insert(
            "Name".to_string(),
            Value::Concrete(ConcreteValue::String("my-vpc".to_string())),
        );
        resource.set_attr(
            "tags".to_string(),
            Value::Concrete(ConcreteValue::Map(resource_tags)),
        );

        let default_tags: IndexMap<String, Value> = IndexMap::new();

        let mut resources = vec![resource];
        normalizer.merge_default_tags(&mut resources, &default_tags, &schemas);

        if let Some(Value::Concrete(ConcreteValue::Map(tags))) = resources[0].get_attr("tags") {
            assert_eq!(tags.len(), 1);
            assert_eq!(
                tags.get("Name"),
                Some(&Value::Concrete(ConcreteValue::String(
                    "my-vpc".to_string()
                )))
            );
        } else {
            panic!("Expected tags to be unchanged");
        }
        assert!(!resources[0].attributes.contains_key("_default_tag_keys"));
    }
}
