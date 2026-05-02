use std::collections::HashMap;

mod convert;
use carina_plugin_sdk::CarinaProvider;
use carina_provider_protocol::types as proto;

use carina_core::provider::{
    Provider, ProviderError as CoreProviderError, ProviderNormalizer, SavedAttrs,
};
use carina_core::resource::{ResourceId as CoreResourceId, State as CoreState, Value as CoreValue};

use carina_provider_awscc::AwsccNormalizer;
use carina_provider_awscc::provider::AwsccProvider;
use carina_provider_awscc::schemas;

struct AwsccProcessProvider {
    runtime: tokio::runtime::Runtime,
    provider: Option<AwsccProvider>,
    normalizer: AwsccNormalizer,
}

impl Default for AwsccProcessProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl AwsccProcessProvider {
    fn new() -> Self {
        #[cfg(not(target_arch = "wasm32"))]
        let runtime = tokio::runtime::Runtime::new().expect("Failed to create tokio runtime");
        #[cfg(target_arch = "wasm32")]
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_time()
            .build()
            .expect("Failed to create tokio runtime");
        Self {
            runtime,
            provider: None,
            normalizer: AwsccNormalizer,
        }
    }

    fn convert_error(e: CoreProviderError) -> proto::ProviderError {
        proto::ProviderError {
            message: e.to_string(),
            resource_id: e
                .resource_id
                .as_ref()
                .map(convert::core_to_proto_resource_id),
            is_timeout: e.is_timeout,
        }
    }

    fn provider(&self) -> &AwsccProvider {
        self.provider
            .as_ref()
            .expect("Provider not initialized; call initialize() first")
    }
}

impl CarinaProvider for AwsccProcessProvider {
    fn info(&self) -> proto::ProviderInfo {
        proto::ProviderInfo {
            name: "awscc".into(),
            display_name: "AWS Cloud Control provider".into(),
            capabilities: vec![],
            version: env!("CARGO_PKG_VERSION").into(),
        }
    }

    fn schemas(&self) -> Vec<proto::ResourceSchema> {
        schemas::generated::configs()
            .iter()
            .map(|config| {
                let mut schema = convert::core_to_proto_schema(&config.schema);
                if config.has_tags {
                    schema
                        .validators
                        .push(proto::ValidatorType::TagsKeyValueCheck);
                }
                schema
            })
            .collect()
    }

    fn provider_config_attribute_types(&self) -> HashMap<String, proto::AttributeType> {
        let mut types = HashMap::new();
        types.insert(
            "region".to_string(),
            proto::AttributeType::StringEnum {
                name: "Region".to_string(),
                values: carina_aws_types::REGIONS
                    .iter()
                    .map(|(code, _)| code.to_string())
                    .collect(),
                namespace: Some("awscc".to_string()),
            },
        );
        types
    }

    fn validate_config(&self, _attrs: &HashMap<String, proto::Value>) -> Result<(), String> {
        // Region format/value validation is handled by the host via
        // `provider_config_attribute_types`. No provider-specific semantic
        // checks are needed beyond that for now.
        Ok(())
    }

    fn initialize(&mut self, attrs: &HashMap<String, proto::Value>) -> Result<(), String> {
        let core_attrs = convert::proto_to_core_value_map(attrs);
        let region = if let Some(CoreValue::String(region)) = core_attrs.get("region") {
            carina_core::utils::convert_region_value(region)
        } else {
            "ap-northeast-1".to_string()
        };
        let provider = self.runtime.block_on(AwsccProvider::new(&region));
        self.provider = Some(provider);
        Ok(())
    }

    fn config_completions(&self) -> HashMap<String, Vec<proto::CompletionValue>> {
        HashMap::from([(
            "region".to_string(),
            carina_aws_types::region_completions("awscc")
                .into_iter()
                .map(|c| proto::CompletionValue {
                    value: c.value,
                    description: c.description,
                })
                .collect(),
        )])
    }

    fn identity_attributes(&self) -> Vec<String> {
        vec!["region".to_string()]
    }

    fn enum_aliases(&self) -> HashMap<String, HashMap<String, HashMap<String, String>>> {
        schemas::generated::build_enum_aliases_map()
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

    fn read(
        &self,
        id: &proto::ResourceId,
        identifier: Option<&str>,
    ) -> Result<proto::State, proto::ProviderError> {
        let core_id = convert::proto_to_core_resource_id(id);
        let result = self
            .runtime
            .block_on(self.provider().read(&core_id, identifier));
        match result {
            Ok(state) => Ok(convert::core_to_proto_state(&state)),
            Err(e) => Err(Self::convert_error(e)),
        }
    }

    fn read_data_source(
        &self,
        resource: &proto::Resource,
    ) -> Result<proto::State, proto::ProviderError> {
        self.read(&resource.id, None)
    }

    fn create(&self, resource: &proto::Resource) -> Result<proto::State, proto::ProviderError> {
        let core_resource = convert::proto_to_core_resource(resource);
        let result = self
            .runtime
            .block_on(self.provider().create(&core_resource));
        match result {
            Ok(state) => Ok(convert::core_to_proto_state(&state)),
            Err(e) => Err(Self::convert_error(e)),
        }
    }

    fn update(
        &self,
        id: &proto::ResourceId,
        identifier: &str,
        from: &proto::State,
        to: &proto::Resource,
    ) -> Result<proto::State, proto::ProviderError> {
        let core_id = convert::proto_to_core_resource_id(id);
        let core_from = convert::proto_to_core_state(from);
        let core_to = convert::proto_to_core_resource(to);
        let result = self.runtime.block_on(
            self.provider()
                .update(&core_id, identifier, &core_from, &core_to),
        );
        match result {
            Ok(state) => Ok(convert::core_to_proto_state(&state)),
            Err(e) => Err(Self::convert_error(e)),
        }
    }

    fn delete(
        &self,
        id: &proto::ResourceId,
        identifier: &str,
        lifecycle: &proto::LifecycleConfig,
    ) -> Result<(), proto::ProviderError> {
        let core_id = convert::proto_to_core_resource_id(id);
        let core_lifecycle = carina_core::resource::LifecycleConfig {
            force_delete: lifecycle.force_delete,
            create_before_destroy: lifecycle.create_before_destroy,
            prevent_destroy: lifecycle.prevent_destroy,
        };
        let result = self.runtime.block_on(self.provider().delete(
            &core_id,
            identifier,
            &core_lifecycle,
        ));
        match result {
            Ok(()) => Ok(()),
            Err(e) => Err(Self::convert_error(e)),
        }
    }

    fn normalize_desired(&self, resources: Vec<proto::Resource>) -> Vec<proto::Resource> {
        let mut core_resources: Vec<_> = resources
            .iter()
            .map(convert::proto_to_core_resource)
            .collect();
        self.normalizer.normalize_desired(&mut core_resources);
        core_resources
            .iter()
            .map(convert::core_to_proto_resource)
            .collect()
    }

    fn normalize_state(
        &self,
        states: HashMap<String, proto::State>,
    ) -> HashMap<String, proto::State> {
        let mut core_states: HashMap<CoreResourceId, CoreState> = states
            .values()
            .map(|s| {
                let core_state = convert::proto_to_core_state(s);
                (core_state.id.clone(), core_state)
            })
            .collect();
        self.normalizer.normalize_state(&mut core_states);
        core_states
            .iter()
            .map(|(id, state)| (id.to_string(), convert::core_to_proto_state(state)))
            .collect()
    }

    fn hydrate_read_state(
        &self,
        states: &mut HashMap<String, proto::State>,
        saved_attrs: &HashMap<String, HashMap<String, proto::Value>>,
    ) {
        // Build a key-to-CoreResourceId lookup from states (which contain structured IDs)
        let key_to_id: HashMap<&str, CoreResourceId> = states
            .iter()
            .map(|(k, s)| (k.as_str(), convert::proto_to_core_resource_id(&s.id)))
            .collect();

        let mut core_states: HashMap<CoreResourceId, CoreState> = states
            .values()
            .map(|s| {
                let core_state = convert::proto_to_core_state(s);
                (core_state.id.clone(), core_state)
            })
            .collect();
        let core_saved: SavedAttrs = saved_attrs
            .iter()
            .filter_map(|(k, v)| {
                let id = key_to_id.get(k.as_str())?.clone();
                let attrs = convert::proto_to_core_value_map(v);
                Some((id, attrs))
            })
            .collect();
        self.normalizer
            .hydrate_read_state(&mut core_states, &core_saved);
        *states = core_states
            .iter()
            .map(|(id, state)| (id.to_string(), convert::core_to_proto_state(state)))
            .collect();
    }

    fn merge_default_tags(
        &self,
        resources: &mut Vec<proto::Resource>,
        default_tags: &HashMap<String, proto::Value>,
        proto_schemas: &Vec<proto::ResourceSchema>,
    ) {
        let mut core_resources: Vec<_> = resources
            .iter()
            .map(convert::proto_to_core_resource)
            .collect();
        let core_tags: indexmap::IndexMap<String, _> = default_tags
            .iter()
            .map(|(k, v)| (k.clone(), convert::proto_to_core_value(v)))
            .collect();
        let mut registry = carina_core::schema::SchemaRegistry::new();
        for s in proto_schemas {
            registry.insert("awscc", convert::proto_to_core_schema(s));
        }
        self.normalizer
            .merge_default_tags(&mut core_resources, &core_tags, &registry);
        *resources = core_resources
            .iter()
            .map(convert::core_to_proto_resource)
            .collect();
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn main() {
    carina_plugin_sdk::run(AwsccProcessProvider::new());
}

#[cfg(target_arch = "wasm32")]
carina_plugin_sdk::export_provider!(AwsccProcessProvider, http);

#[cfg(target_arch = "wasm32")]
fn main() {}

#[cfg(test)]
mod tests {
    use super::*;
    use carina_plugin_sdk::types::ValidatorType;

    #[test]
    fn schemas_include_tags_validator_for_tagged_resources() {
        let provider = AwsccProcessProvider::new();
        let schemas = provider.schemas();
        let bucket = schemas
            .iter()
            .find(|s| s.resource_type == "s3.Bucket")
            .expect("s3.bucket schema should exist");
        assert!(
            bucket
                .validators
                .contains(&ValidatorType::TagsKeyValueCheck),
            "s3.bucket should have TagsKeyValueCheck validator"
        );
    }

    #[test]
    fn schemas_exclude_tags_validator_for_untagged_resources() {
        let provider = AwsccProcessProvider::new();
        let schemas = provider.schemas();
        let configs = schemas::generated::configs();
        if let Some(untagged) = configs.iter().find(|c| !c.has_tags) {
            let schema = schemas
                .iter()
                .find(|s| s.resource_type == untagged.resource_type_name)
                .expect("untagged schema should exist");
            assert!(
                !schema
                    .validators
                    .contains(&ValidatorType::TagsKeyValueCheck),
                "untagged resource should not have TagsKeyValueCheck"
            );
        }
    }

    /// Regression for carina-rs/carina#2025: the VPC schema must carry the
    /// `cidr_block` / `ipv4_ipam_pool_id` oneOf constraint as serializable
    /// data, so it survives the WASM plugin boundary and validate/plan can
    /// reject `awscc.ec2.Vpc {}` before any provider call.
    #[test]
    fn vpc_schema_carries_cidr_block_exclusive_group_across_proto() {
        let provider = AwsccProcessProvider::new();
        let schemas = provider.schemas();
        let vpc = schemas
            .iter()
            .find(|s| s.resource_type == "ec2.Vpc")
            .expect("ec2.vpc schema should exist");
        assert!(
            vpc.exclusive_required
                .iter()
                .any(|g| g.contains(&"cidr_block".to_string())
                    && g.contains(&"ipv4_ipam_pool_id".to_string())),
            "vpc schema should expose cidr_block/ipv4_ipam_pool_id as a declarative \
             exclusive_required group, got: {:?}",
            vpc.exclusive_required
        );
    }
}
