use std::collections::HashMap;

mod convert;
use carina_plugin_sdk::CarinaProvider;
use carina_provider_protocol::types as proto;

use carina_core::provider::{
    CreateRequest as CoreCreateRequest, DeleteRequest as CoreDeleteRequest, Provider,
    ProviderError as CoreProviderError, ProviderNormalizer, ReadRequest as CoreReadRequest,
    SavedAttrs, UpdateRequest as CoreUpdateRequest,
};
use carina_core::resource::{
    ConcreteValue, ResourceId as CoreResourceId, State as CoreState, Value as CoreValue,
};

use carina_provider_awscc::AwsccNormalizer;
use carina_provider_awscc::provider::{AwsccProvider, AwsccProviderConfig};
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
        convert::core_to_proto_provider_error(e)
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
                // Region API spellings carry hyphens (`ap-northeast-1`)
                // but the DSL spelling uses underscores. Materialize
                // the alias pairs as data; a `fn` pointer would not
                // cross the WASM boundary (carina#2831).
                dsl_aliases: carina_aws_types::region_dsl_aliases(),
            },
        );
        types.insert(
            "allowed_account_ids".to_string(),
            proto::AttributeType::List {
                inner: Box::new(proto::AttributeType::String),
                ordered: false,
            },
        );
        types.insert(
            "forbidden_account_ids".to_string(),
            proto::AttributeType::List {
                inner: Box::new(proto::AttributeType::String),
                ordered: false,
            },
        );
        types.insert("assume_role".to_string(), assume_role_attribute_type());
        types
    }

    fn validate_config(&self, attrs: &HashMap<String, proto::Value>) -> Result<(), String> {
        // Cross-account guardrail: when `assume_role.role_arn` parses to
        // an account id that is not in `allowed_account_ids` (when that
        // list is configured), refuse the configuration. Avoids the
        // foot-gun of an assume-role silently landing in the wrong AWS
        // account (awscc#260).
        use carina_provider_awscc::provider::assume_role::{
            check_cross_account, extract_assume_role,
        };
        let core_attrs = convert::proto_to_core_value_map(attrs);
        let assume_role = extract_assume_role(core_attrs.get("assume_role"))?;
        if let Some(ar) = &assume_role {
            let allowed = extract_string_list(&core_attrs, "allowed_account_ids");
            check_cross_account(&ar.role_arn, &allowed)?;
        }
        Ok(())
    }

    fn initialize(&mut self, attrs: &HashMap<String, proto::Value>) -> Result<(), String> {
        let core_attrs = convert::proto_to_core_value_map(attrs);
        let region = if let Some(CoreValue::Concrete(ConcreteValue::String(region))) =
            core_attrs.get("region")
        {
            carina_core::utils::convert_region_value(region)
        } else {
            "ap-northeast-1".to_string()
        };
        let cfg = extract_account_guard_config(&core_attrs)?;
        let provider = self
            .runtime
            .block_on(AwsccProvider::new_with_config(&region, &cfg));
        // Surface the account-guard rejection eagerly: if the caller's
        // AWS account violates allowed_account_ids / forbidden_account_ids,
        // initialize() must fail so the host aborts before any plan,
        // refresh, or apply step.
        if let Some(err) = provider.init_error() {
            return Err(err.to_string());
        }
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

    // `enum_aliases` is no longer overridden. After awscc#220 / #223,
    // DSL → API canonical conversion goes through `DslMap::api_for`
    // against each `StringEnum`'s exhaustive `dsl_aliases` table
    // (sourced from `carina-core` schema). The host-side dispatch
    // table this method used to feed has been removed.

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
        _request: proto::ReadRequest,
    ) -> Result<proto::State, proto::ProviderError> {
        let core_id = convert::proto_to_core_resource_id(id);
        let result =
            self.runtime
                .block_on(self.provider().read(&core_id, identifier, CoreReadRequest));
        match result {
            Ok(state) => Ok(convert::core_to_proto_state(&state)),
            Err(e) => Err(Self::convert_error(e)),
        }
    }

    fn read_data_source(
        &self,
        resource: &proto::Resource,
    ) -> Result<proto::State, proto::ProviderError> {
        let core_resource = convert::proto_to_core_resource(resource);
        let result = self
            .runtime
            .block_on(self.provider().read_data_source(&core_resource));
        match result {
            Ok(state) => Ok(convert::core_to_proto_state(&state)),
            Err(e) => Err(Self::convert_error(e)),
        }
    }

    fn create(
        &self,
        id: &proto::ResourceId,
        request: proto::CreateRequest,
    ) -> Result<proto::State, proto::ProviderError> {
        let core_id = convert::proto_to_core_resource_id(id);
        let core_resource = convert::proto_to_core_resource(&request.resource);
        let result = self.runtime.block_on(self.provider().create(
            &core_id,
            CoreCreateRequest {
                resource: core_resource,
            },
        ));
        match result {
            Ok(state) => Ok(convert::core_to_proto_state(&state)),
            Err(e) => Err(Self::convert_error(e)),
        }
    }

    fn update(
        &self,
        id: &proto::ResourceId,
        identifier: &str,
        request: proto::UpdateRequest,
    ) -> Result<proto::State, proto::ProviderError> {
        let core_id = convert::proto_to_core_resource_id(id);
        let core_from = convert::proto_to_core_state(&request.from);
        let core_patch = convert::proto_to_core_update_patch(&request.patch);
        let result = self.runtime.block_on(self.provider().update(
            &core_id,
            identifier,
            CoreUpdateRequest {
                from: core_from,
                patch: core_patch,
            },
        ));
        match result {
            Ok(state) => Ok(convert::core_to_proto_state(&state)),
            Err(e) => Err(Self::convert_error(e)),
        }
    }

    fn delete(
        &self,
        id: &proto::ResourceId,
        identifier: &str,
        request: proto::DeleteRequest,
    ) -> Result<(), proto::ProviderError> {
        let core_id = convert::proto_to_core_resource_id(id);
        let core_directives = carina_core::resource::Directives {
            force_delete: request.directives.force_delete,
            create_before_destroy: request.directives.create_before_destroy,
            prevent_destroy: request.directives.prevent_destroy,
            depends_on: Vec::new(),
            provider_instance: None,
        };
        let result = self.runtime.block_on(self.provider().delete(
            &core_id,
            identifier,
            CoreDeleteRequest {
                directives: core_directives,
            },
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
        // Guest-side: drive the now-async normalizer on the guest's own
        // outermost runtime — same pattern as the guest's `Provider`
        // CRUD methods. Not a nested runtime: the host drives the WASM
        // call, the guest drives its internal async (carina#3112).
        self.runtime
            .block_on(self.normalizer.normalize_desired(&mut core_resources));
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
        self.runtime
            .block_on(self.normalizer.normalize_state(&mut core_states));
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
        self.runtime.block_on(
            self.normalizer
                .hydrate_read_state(&mut core_states, &core_saved),
        );
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
        self.runtime.block_on(self.normalizer.merge_default_tags(
            &mut core_resources,
            &core_tags,
            &registry,
        ));
        *resources = core_resources
            .iter()
            .map(convert::core_to_proto_resource)
            .collect();
    }
}

/// Pull `allowed_account_ids` / `forbidden_account_ids` / `assume_role`
/// out of the provider's configured attributes. Both lists unset/empty
/// means "no account check"; `assume_role` absent means "do not chain
/// sts:AssumeRole". Mirrors the in-process Provider wiring in
/// `lib.rs::extract_provider_config`.
fn extract_account_guard_config(
    core_attrs: &HashMap<String, CoreValue>,
) -> Result<AwsccProviderConfig, String> {
    let assume_role = carina_provider_awscc::provider::assume_role::extract_assume_role(
        core_attrs.get("assume_role"),
    )?;
    Ok(AwsccProviderConfig {
        allowed_account_ids: extract_string_list(core_attrs, "allowed_account_ids"),
        forbidden_account_ids: extract_string_list(core_attrs, "forbidden_account_ids"),
        assume_role,
    })
}

/// Shared `list(string)` extractor used by both `validate_config` and
/// `extract_account_guard_config`. Drops non-string elements silently —
/// the host enforces the declared `list(string)` shape before we see
/// the value.
fn extract_string_list(attrs: &HashMap<String, CoreValue>, key: &str) -> Vec<String> {
    match attrs.get(key) {
        Some(CoreValue::Concrete(ConcreteValue::List(items))) => items
            .iter()
            .filter_map(|v| match v {
                CoreValue::Concrete(ConcreteValue::String(s)) => Some(s.clone()),
                _ => None,
            })
            .collect(),
        _ => Vec::new(),
    }
}

/// Schema for the provider-level `assume_role` block. Mirrors the
/// Terraform AWS provider's `assume_role` (MVP field set: `role_arn`,
/// `session_name`, `external_id`, `duration`). When present, the
/// provider chains an `sts:AssumeRole` call on top of the ambient
/// credential chain (awscc#260).
fn assume_role_attribute_type() -> proto::AttributeType {
    proto::AttributeType::Struct {
        name: "AssumeRole".to_string(),
        fields: vec![
            proto::StructField {
                name: "role_arn".to_string(),
                field_type: proto::AttributeType::String,
                required: true,
                description: Some("IAM role ARN to assume.".to_string()),
                block_name: None,
                provider_name: None,
            },
            proto::StructField {
                name: "session_name".to_string(),
                field_type: proto::AttributeType::String,
                required: false,
                description: Some(
                    "STS session name to associate with the assumed-role session.".to_string(),
                ),
                block_name: None,
                provider_name: None,
            },
            proto::StructField {
                name: "external_id".to_string(),
                field_type: proto::AttributeType::String,
                required: false,
                description: Some(
                    "External ID required by the trust policy of the assumed role.".to_string(),
                ),
                block_name: None,
                provider_name: None,
            },
            proto::StructField {
                name: "duration".to_string(),
                field_type: proto::AttributeType::Duration,
                required: false,
                description: Some(
                    "Assumed-role session duration (e.g., 30min, 1h, 15s).".to_string(),
                ),
                block_name: None,
                provider_name: None,
            },
        ],
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

    #[test]
    fn provider_config_attribute_types_declares_assume_role_struct() {
        // Mirrors the in-process Provider's assume_role declaration over
        // the WASM/proto boundary. If the proto declaration goes missing
        // or its required field changes, the host stops surfacing
        // schema-level errors for malformed assume_role blocks
        // (awscc#260).
        let provider = AwsccProcessProvider::new();
        let types = provider.provider_config_attribute_types();
        let ty = types
            .get("assume_role")
            .expect("assume_role must be declared as a provider config attribute");
        match ty {
            proto::AttributeType::Struct { name, fields } => {
                assert_eq!(name, "AssumeRole");
                let role_arn = fields
                    .iter()
                    .find(|f| f.name == "role_arn")
                    .expect("assume_role.role_arn must be declared");
                assert!(role_arn.required, "role_arn must be required");
                for opt in ["session_name", "external_id"] {
                    let f = fields
                        .iter()
                        .find(|f| f.name == opt)
                        .unwrap_or_else(|| panic!("assume_role.{opt} must be declared"));
                    assert!(!f.required, "assume_role.{opt} must be optional");
                    assert!(matches!(f.field_type, proto::AttributeType::String));
                }
                let duration = fields
                    .iter()
                    .find(|f| f.name == "duration")
                    .expect("assume_role.duration must be declared");
                assert!(!duration.required, "duration must be optional");
                assert!(
                    matches!(duration.field_type, proto::AttributeType::Duration),
                    "duration must be a Duration so DSL literals like `30min` are accepted; \
                     was {:?}",
                    duration.field_type
                );
            }
            other => panic!("assume_role must be a Struct, was {other:?}"),
        }
    }

    fn proto_str(s: &str) -> proto::Value {
        proto::Value::String(s.to_string())
    }

    fn proto_list_str(items: &[&str]) -> proto::Value {
        proto::Value::List(items.iter().copied().map(proto_str).collect())
    }

    fn proto_map(items: &[(&str, proto::Value)]) -> proto::Value {
        let mut m = std::collections::HashMap::new();
        for (k, v) in items {
            m.insert((*k).to_string(), v.clone());
        }
        proto::Value::Map(m)
    }

    #[test]
    fn validate_config_accepts_assume_role_in_allowed_account() {
        let provider = AwsccProcessProvider::new();
        let attrs = HashMap::from([
            (
                "allowed_account_ids".to_string(),
                proto_list_str(&["412038850359"]),
            ),
            (
                "assume_role".to_string(),
                proto_map(&[(
                    "role_arn",
                    proto_str("arn:aws:iam::412038850359:role/delegation"),
                )]),
            ),
        ]);
        provider
            .validate_config(&attrs)
            .expect("matching allowed_account_ids must validate");
    }

    #[test]
    fn validate_config_rejects_assume_role_outside_allowed_account() {
        let provider = AwsccProcessProvider::new();
        let attrs = HashMap::from([
            (
                "allowed_account_ids".to_string(),
                proto_list_str(&["111111111111"]),
            ),
            (
                "assume_role".to_string(),
                proto_map(&[(
                    "role_arn",
                    proto_str("arn:aws:iam::412038850359:role/delegation"),
                )]),
            ),
        ]);
        let err = provider
            .validate_config(&attrs)
            .expect_err("cross-account role outside allowed_account_ids must fail");
        assert!(err.contains("412038850359"), "must name target: {err}");
        assert!(err.contains("111111111111"), "must name allow list: {err}");
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
