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
    BoxFuture, Provider, ProviderError, ProviderFactory, ProviderNormalizer, ProviderResult,
    SavedAttrs, merge_default_tags_for_provider,
};
use carina_core::resource::{LifecycleConfig, Resource, ResourceId, State, Value};
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
                to_dsl: Some(|s| s.replace('-', "_")),
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
        if let Some(Value::String(region)) = attributes.get("region") {
            return carina_core::utils::convert_region_value(region);
        }
        "ap-northeast-1".to_string()
    }

    fn create_provider(
        &self,
        attributes: &IndexMap<String, Value>,
    ) -> BoxFuture<'_, Box<dyn Provider>> {
        let region = self.extract_region(attributes);
        let cfg = extract_provider_config(attributes);
        Box::pin(async move {
            Box::new(AwsccProvider::new_with_config(&region, &cfg).await) as Box<dyn Provider>
        })
    }

    fn create_normalizer(
        &self,
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

    fn get_enum_alias_reverse(
        &self,
        resource_type: &str,
        attr_name: &str,
        value: &str,
    ) -> Option<String> {
        schemas::generated::get_enum_alias_reverse(resource_type, attr_name, value)
            .map(|s| s.to_string())
    }
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
            Some(Value::List(items)) => items
                .iter()
                .filter_map(|v| match v {
                    Value::String(s) => Some(s.clone()),
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
    ) -> BoxFuture<'_, ProviderResult<State>> {
        if let Some(err) = self.init_error() {
            let err = err.to_string();
            let id = id.clone();
            return Box::pin(async move { Err(ProviderError::new(err).for_resource(id)) });
        }
        let id = id.clone();
        let identifier = identifier.map(|s| s.to_string());
        Box::pin(async move {
            self.read_resource(&id.resource_type, id.name_str(), identifier.as_deref())
                .await
        })
    }

    fn read_data_source(&self, resource: &Resource) -> BoxFuture<'_, ProviderResult<State>> {
        self.read(&resource.id, None)
    }

    fn create(&self, resource: &Resource) -> BoxFuture<'_, ProviderResult<State>> {
        if let Some(err) = self.init_error() {
            let err = err.to_string();
            let id = resource.id.clone();
            return Box::pin(async move { Err(ProviderError::new(err).for_resource(id)) });
        }
        let resource = resource.clone();
        Box::pin(async move { self.create_resource(resource).await })
    }

    fn update(
        &self,
        id: &ResourceId,
        identifier: &str,
        from: &State,
        to: &Resource,
    ) -> BoxFuture<'_, ProviderResult<State>> {
        if let Some(err) = self.init_error() {
            let err = err.to_string();
            let id = id.clone();
            return Box::pin(async move { Err(ProviderError::new(err).for_resource(id)) });
        }
        let id = id.clone();
        let identifier = identifier.to_string();
        let from = from.clone();
        let to = to.clone();
        Box::pin(async move { self.update_resource(id, &identifier, &from, to).await })
    }

    fn delete(
        &self,
        id: &ResourceId,
        identifier: &str,
        lifecycle: &LifecycleConfig,
    ) -> BoxFuture<'_, ProviderResult<()>> {
        if let Some(err) = self.init_error() {
            let err = err.to_string();
            let id = id.clone();
            return Box::pin(async move { Err(ProviderError::new(err).for_resource(id)) });
        }
        let id = id.clone();
        let identifier = identifier.to_string();
        let lifecycle = lifecycle.clone();
        Box::pin(async move { self.delete_resource(&id, &identifier, &lifecycle).await })
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
            Value::List(vec![
                Value::String("111111111111".to_string()),
                Value::String("222222222222".to_string()),
            ]),
        );
        attrs.insert(
            "forbidden_account_ids".to_string(),
            Value::List(vec![Value::String("999999999999".to_string())]),
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

    #[test]
    fn test_merge_default_tags_resource_tags_win() {
        let schemas = build_schemas();
        let normalizer = AwsccNormalizer;

        let mut resource = Resource::with_provider("awscc", "ec2.Vpc", "test-vpc");
        resource.set_attr(
            "cidr_block".to_string(),
            Value::String("10.0.0.0/16".to_string()),
        );
        let mut resource_tags: IndexMap<String, Value> = IndexMap::new();
        resource_tags.insert("Name".to_string(), Value::String("my-vpc".to_string()));
        resource_tags.insert(
            "Environment".to_string(),
            Value::String("staging".to_string()),
        );
        resource.set_attr("tags".to_string(), Value::Map(resource_tags));

        let mut default_tags: IndexMap<String, Value> = IndexMap::new();
        default_tags.insert(
            "Environment".to_string(),
            Value::String("production".to_string()),
        );
        default_tags.insert("Team".to_string(), Value::String("platform".to_string()));

        let mut resources = vec![resource];
        normalizer.merge_default_tags(&mut resources, &default_tags, &schemas);

        if let Some(Value::Map(tags)) = resources[0].get_attr("tags") {
            assert_eq!(
                tags.get("Environment"),
                Some(&Value::String("staging".to_string()))
            );
            assert_eq!(tags.get("Name"), Some(&Value::String("my-vpc".to_string())));
            assert_eq!(
                tags.get("Team"),
                Some(&Value::String("platform".to_string()))
            );
        } else {
            panic!("Expected tags to be a Map");
        }

        if let Some(Value::List(keys)) = resources[0].get_attr("_default_tag_keys") {
            let key_strs: Vec<&str> = keys
                .iter()
                .filter_map(|v| match v {
                    Value::String(s) => Some(s.as_str()),
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

        let mut resource = Resource::with_provider("awscc", "ec2.Vpc", "test-vpc");
        resource.set_attr(
            "cidr_block".to_string(),
            Value::String("10.0.0.0/16".to_string()),
        );

        let mut default_tags: IndexMap<String, Value> = IndexMap::new();
        default_tags.insert(
            "Environment".to_string(),
            Value::String("production".to_string()),
        );

        let mut resources = vec![resource];
        normalizer.merge_default_tags(&mut resources, &default_tags, &schemas);

        if let Some(Value::Map(tags)) = resources[0].get_attr("tags") {
            assert_eq!(
                tags.get("Environment"),
                Some(&Value::String("production".to_string()))
            );
        } else {
            panic!("Expected tags to be set from default_tags");
        }

        if let Some(Value::List(keys)) = resources[0].get_attr("_default_tag_keys") {
            let key_strs: Vec<&str> = keys
                .iter()
                .filter_map(|v| match v {
                    Value::String(s) => Some(s.as_str()),
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

        let mut resource = Resource::with_provider("awscc", "ec2.Route", "test-route");
        resource.set_attr(
            "route_table_id".to_string(),
            Value::String("rtb-123".to_string()),
        );

        let mut default_tags: IndexMap<String, Value> = IndexMap::new();
        default_tags.insert(
            "Environment".to_string(),
            Value::String("production".to_string()),
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

        let mut resource = Resource::with_provider("awscc", "ec2.Vpc", "test-vpc");
        resource.set_attr(
            "cidr_block".to_string(),
            Value::String("10.0.0.0/16".to_string()),
        );
        let mut resource_tags: IndexMap<String, Value> = IndexMap::new();
        resource_tags.insert("Name".to_string(), Value::String("my-vpc".to_string()));
        resource.set_attr("tags".to_string(), Value::Map(resource_tags));

        let default_tags: IndexMap<String, Value> = IndexMap::new();

        let mut resources = vec![resource];
        normalizer.merge_default_tags(&mut resources, &default_tags, &schemas);

        if let Some(Value::Map(tags)) = resources[0].get_attr("tags") {
            assert_eq!(tags.len(), 1);
            assert_eq!(tags.get("Name"), Some(&Value::String("my-vpc".to_string())));
        } else {
            panic!("Expected tags to be unchanged");
        }
        assert!(!resources[0].attributes.contains_key("_default_tag_keys"));
    }
}
