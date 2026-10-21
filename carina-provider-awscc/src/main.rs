use std::collections::HashMap;
use std::sync::LazyLock;

mod convert;
use carina_plugin_sdk::{
    CarinaProvider, CreateOutcome as SdkCreateOutcome, DeleteOutcome as SdkDeleteOutcome, Pending,
    UpdateOutcome as SdkUpdateOutcome,
};
use carina_provider_protocol::OperationStep;
use carina_provider_protocol::types as proto;

use carina_core::executor::normalized::NormalizedResource;
use carina_core::provider::{
    DeleteRequest as CoreDeleteRequest, LiftedSavedAttrs, PersistedAttributes, Provider,
    ProviderError as CoreProviderError, ProviderNormalizer, RawSavedAttrs,
    ReadRequest as CoreReadRequest,
};
use carina_core::resource::{
    ConcreteValue, ResourceId as CoreResourceId, State as CoreState, Value as CoreValue,
};
use carina_core::schema::SchemaRegistry;

use carina_provider_awscc::AwsccNormalizer;
use carina_provider_awscc::provider::{
    AwsccProvider, AwsccProviderConfig, CcCreateToken, CcDeleteToken, CcUpdateToken,
};
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
        e.to_wire()
    }

    fn provider(&self) -> &AwsccProvider {
        self.provider
            .as_ref()
            .expect("Provider not initialized; call initialize() first")
    }

    fn schema_registry() -> &'static SchemaRegistry {
        static REGISTRY: LazyLock<SchemaRegistry> = LazyLock::new(|| {
            let mut registry = SchemaRegistry::new();
            for config in schemas::generated::configs() {
                registry.insert("awscc", config.schema.clone());
            }
            registry
        });
        &REGISTRY
    }

    fn normalize_create_resource(
        &self,
        resource: &proto::Resource,
    ) -> Result<NormalizedResource, CoreProviderError> {
        let core_resource = convert::proto_to_core_resource(resource);
        self.runtime.block_on(
            carina_core::executor::normalized::apply_desired_normalization(
                core_resource,
                &[],
                &self.normalizer,
                Self::schema_registry(),
            ),
        )
    }

    fn map_step<T, Token, Outcome>(
        result: Result<OperationStep<T, Token>, CoreProviderError>,
        done: impl FnOnce(T) -> Outcome,
        pending: impl FnOnce(Pending<Token>) -> Outcome,
    ) -> Result<Outcome, CoreProviderError> {
        match result {
            Ok(OperationStep::Done(value)) => Ok(done(value)),
            Ok(OperationStep::Pending {
                pending: token,
                poll_after_secs,
            }) => Ok(pending(Pending {
                token,
                poll_after_secs,
            })),
            Err(error) => Err(error),
        }
    }

    fn core_delete_request(request: proto::DeleteRequest) -> CoreDeleteRequest {
        CoreDeleteRequest {
            directives: carina_core::resource::Directives {
                force_delete: request.directives.force_delete,
                create_before_destroy: request.directives.create_before_destroy,
                prevent_destroy: request.directives.prevent_destroy,
                depends_on: Vec::new(),
                provider_instance: None,
            },
        }
    }
}

impl CarinaProvider for AwsccProcessProvider {
    type CreateToken = CcCreateToken;
    type UpdateToken = CcUpdateToken;
    type DeleteToken = CcDeleteToken;

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
                // proto's StringEnum still carries the flat namespace
                // string for the wire form; only the core form was
                // restructured in carina#3222.
                namespace: Some("aws".to_string()),
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
                element_type: Box::new(proto_string_attribute_type()),
                ordered: false,
                length: None,
                validate: None,
            },
        );
        types.insert(
            "forbidden_account_ids".to_string(),
            proto::AttributeType::List {
                element_type: Box::new(proto_string_attribute_type()),
                ordered: false,
                length: None,
                validate: None,
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
            region.expose_plaintext().to_string()
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

    // Protocol v6 resolves enum aliases in the host. The guest receives API
    // spellings as wire strings, so it does not expose an enum-alias callback.

    fn validate_custom_type(
        &self,
        identity: &carina_plugin_sdk::types::TypeIdentity,
        value: &str,
    ) -> Result<(), String> {
        use carina_core::parser::ValidatorFn;
        use std::sync::OnceLock;
        static VALIDATORS: OnceLock<HashMap<String, ValidatorFn>> = OnceLock::new();
        let validators = VALIDATORS.get_or_init(schemas::config::awscc_validators);
        // Inner map is still snake-cased; project the kind once at the
        // boundary. Provider-axis collisions are filtered by the host
        // before the call reaches us.
        let key = carina_core::parser::pascal_to_snake(&identity.kind);
        if let Some(validator) = validators.get(&key) {
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
        let core_data_source = convert::proto_to_core_data_source(resource);
        let result = self
            .runtime
            .block_on(self.provider().read_resource(&core_data_source.id, None));
        match result {
            Ok(state) => Ok(convert::core_to_proto_state(&state)),
            Err(e) => Err(Self::convert_error(e)),
        }
    }

    fn create(
        &self,
        _id: &proto::ResourceId,
        request: proto::CreateRequest,
    ) -> Result<SdkCreateOutcome<Self::CreateToken>, proto::ProviderError> {
        let normalized_resource = self
            .normalize_create_resource(&request.resource)
            .map_err(Self::convert_error)?;
        let result = self.runtime.block_on(
            self.provider()
                .create_step(normalized_resource.as_resource()),
        );
        Self::map_step(
            result,
            |outcome| SdkCreateOutcome::Done(convert::core_to_proto_create_outcome(outcome)),
            SdkCreateOutcome::Pending,
        )
        .map_err(Self::convert_error)
    }

    fn poll_create(
        &self,
        _id: &proto::ResourceId,
        request: proto::CreateRequest,
        pending: Pending<Self::CreateToken>,
    ) -> Result<SdkCreateOutcome<Self::CreateToken>, proto::ProviderError> {
        let normalized_resource = self
            .normalize_create_resource(&request.resource)
            .map_err(Self::convert_error)?;
        let result = self.runtime.block_on(
            self.provider()
                .poll_create_step(normalized_resource.as_resource(), pending.token),
        );
        Self::map_step(
            result,
            |outcome| SdkCreateOutcome::Done(convert::core_to_proto_create_outcome(outcome)),
            SdkCreateOutcome::Pending,
        )
        .map_err(Self::convert_error)
    }

    fn update(
        &self,
        id: &proto::ResourceId,
        identifier: &str,
        request: proto::UpdateRequest,
    ) -> Result<SdkUpdateOutcome<Self::UpdateToken>, proto::ProviderError> {
        let core_id = convert::proto_to_core_resource_id(id);
        let core_from = convert::proto_to_core_persisted_state(&request.from).map_err(|error| {
            Self::convert_error(
                CoreProviderError::internal_caused_by(
                    "failed to decode persisted update state",
                    error,
                )
                .for_resource(core_id.clone()),
            )
        })?;
        let core_patch = convert::proto_to_core_update_patch(&request.patch);
        let result = self.runtime.block_on(self.provider().update_step(
            &core_id,
            identifier,
            &core_from,
            &core_patch,
        ));
        Self::map_step(
            result,
            |outcome| SdkUpdateOutcome::Done(convert::core_to_proto_update_outcome(outcome)),
            SdkUpdateOutcome::Pending,
        )
        .map_err(Self::convert_error)
    }

    fn poll_update(
        &self,
        id: &proto::ResourceId,
        identifier: &str,
        request: proto::UpdateRequest,
        pending: Pending<Self::UpdateToken>,
    ) -> Result<SdkUpdateOutcome<Self::UpdateToken>, proto::ProviderError> {
        let core_id = convert::proto_to_core_resource_id(id);
        let core_from = convert::proto_to_core_persisted_state(&request.from).map_err(|error| {
            Self::convert_error(
                CoreProviderError::internal_caused_by(
                    "failed to decode persisted update state",
                    error,
                )
                .for_resource(core_id.clone()),
            )
        })?;
        let core_patch = convert::proto_to_core_update_patch(&request.patch);
        let result = self.runtime.block_on(self.provider().poll_update_step(
            &core_id,
            identifier,
            &core_from,
            &core_patch,
            pending.token,
        ));
        Self::map_step(
            result,
            |outcome| SdkUpdateOutcome::Done(convert::core_to_proto_update_outcome(outcome)),
            SdkUpdateOutcome::Pending,
        )
        .map_err(Self::convert_error)
    }

    fn delete(
        &self,
        id: &proto::ResourceId,
        identifier: &str,
        request: proto::DeleteRequest,
    ) -> Result<SdkDeleteOutcome<Self::DeleteToken>, proto::ProviderError> {
        let core_id = convert::proto_to_core_resource_id(id);
        let core_request = Self::core_delete_request(request);
        let result = self.runtime.block_on(self.provider().delete_step(
            &core_id,
            identifier,
            &core_request,
        ));
        Self::map_step(
            result,
            |()| SdkDeleteOutcome::Deleted,
            SdkDeleteOutcome::Pending,
        )
        .map_err(Self::convert_error)
    }

    fn poll_delete(
        &self,
        id: &proto::ResourceId,
        identifier: &str,
        request: proto::DeleteRequest,
        pending: Pending<Self::DeleteToken>,
    ) -> Result<SdkDeleteOutcome<Self::DeleteToken>, proto::ProviderError> {
        let core_id = convert::proto_to_core_resource_id(id);
        let core_request = Self::core_delete_request(request);
        let result = self.runtime.block_on(self.provider().poll_delete_step(
            &core_id,
            identifier,
            &core_request,
            pending.token,
        ));
        Self::map_step(
            result,
            |()| SdkDeleteOutcome::Deleted,
            SdkDeleteOutcome::Pending,
        )
        .map_err(Self::convert_error)
    }

    fn required_permissions(
        &self,
        id: &proto::ResourceId,
        op: carina_plugin_sdk::PlanOp,
    ) -> Result<Vec<String>, proto::ProviderError> {
        let op = match op {
            carina_plugin_sdk::PlanOp::Create => carina_core::effect::PlanOp::Create,
            carina_plugin_sdk::PlanOp::Read => carina_core::effect::PlanOp::Read,
            carina_plugin_sdk::PlanOp::Update => carina_core::effect::PlanOp::Update,
            carina_plugin_sdk::PlanOp::Delete => carina_core::effect::PlanOp::Delete,
        };
        Ok(
            schemas::generated::required_permissions(&id.resource_type, op)
                .iter()
                .map(|permission| (*permission).to_string())
                .collect(),
        )
    }

    fn normalize_desired(
        &self,
        resources: Vec<proto::Resource>,
    ) -> Result<Vec<proto::Resource>, proto::ProviderError> {
        let mut core_resources: Vec<_> = resources
            .iter()
            .map(convert::proto_to_core_resource)
            .collect();
        // Guest-side: drive the now-async normalizer on the guest's own
        // outermost runtime — same pattern as the guest's `Provider`
        // CRUD methods. Not a nested runtime: the host drives the WASM
        // call, the guest drives its internal async (carina#3112).
        self.runtime
            .block_on(self.normalizer.normalize_desired(&mut core_resources))
            .map_err(Self::convert_error)?;
        Ok(core_resources
            .iter()
            .map(convert::core_to_proto_resource)
            .collect())
    }

    fn normalize_state(
        &self,
        states: HashMap<String, proto::PersistedState>,
    ) -> Result<HashMap<String, proto::PersistedState>, proto::ProviderError> {
        let mut core_states: HashMap<CoreResourceId, CoreState> = states
            .iter()
            .map(|(key, state)| {
                convert::proto_to_core_persisted_state(state)
                    .map(|core_state| (core_state.id.clone(), core_state))
                    .map_err(|error| {
                        CoreProviderError::internal(format!(
                            "normalize_state could not decode persisted state `{key}`: {error}"
                        ))
                    })
            })
            .collect::<Result<_, _>>()
            .map_err(Self::convert_error)?;
        self.runtime
            .block_on(self.normalizer.normalize_state(&mut core_states))
            .map_err(Self::convert_error)?;
        Ok(core_states
            .iter()
            .map(|(id, state)| {
                (
                    id.to_string(),
                    convert::core_to_proto_persisted_state(state),
                )
            })
            .collect())
    }

    fn hydrate_read_state(
        &self,
        states: &mut HashMap<String, proto::PersistedState>,
        saved_attrs: &HashMap<String, HashMap<String, proto::PersistedValue>>,
    ) -> Result<(), proto::ProviderError> {
        // Build a key-to-CoreResourceId lookup from states (which contain structured IDs)
        let key_to_id: HashMap<&str, CoreResourceId> = states
            .iter()
            .map(|(k, s)| (k.as_str(), convert::proto_to_core_resource_id(&s.id)))
            .collect();

        let mut core_states: HashMap<CoreResourceId, CoreState> = states
            .iter()
            .map(|(key, state)| {
                convert::proto_to_core_persisted_state(state)
                    .map(|core_state| (core_state.id.clone(), core_state))
                    .map_err(|error| {
                        CoreProviderError::internal(format!(
                            "hydrate_read_state could not decode persisted state `{key}`: {error}"
                        ))
                    })
            })
            .collect::<Result<_, _>>()
            .map_err(Self::convert_error)?;
        let core_saved =
            decode_persisted_saved_attrs(saved_attrs, &key_to_id, Self::schema_registry())
                .map_err(Self::convert_error)?;
        self.runtime
            .block_on(
                self.normalizer
                    .hydrate_read_state(&mut core_states, core_saved.as_provider_saved_attrs()),
            )
            .map_err(Self::convert_error)?;
        *states = core_states
            .iter()
            .map(|(id, state)| {
                (
                    id.to_string(),
                    convert::core_to_proto_persisted_state(state),
                )
            })
            .collect();
        Ok(())
    }
}

fn decode_persisted_saved_attrs(
    saved_attrs: &HashMap<String, HashMap<String, proto::PersistedValue>>,
    key_to_id: &HashMap<&str, CoreResourceId>,
    registry: &SchemaRegistry,
) -> Result<LiftedSavedAttrs, CoreProviderError> {
    let persisted_saved: HashMap<CoreResourceId, PersistedAttributes> = saved_attrs
        .iter()
        .map(|(key, values)| {
            let id = key_to_id.get(key.as_str()).cloned().ok_or_else(|| {
                CoreProviderError::internal(format!(
                    "hydrate_read_state received saved attributes for unknown state key `{key}`"
                ))
            })?;
            let typed_values =
                convert::proto_to_core_persisted_value_map(values).map_err(|error| {
                    CoreProviderError::internal(format!(
                        "hydrate_read_state could not decode persisted attributes for `{key}`: \
                         {error}"
                    ))
                })?;
            let attrs = PersistedAttributes::from_typed_persisted_wire_values(typed_values)
                .map_err(|error| {
                    CoreProviderError::internal(format!(
                        "hydrate_read_state could not encode persisted attributes for `{key}`: \
                         {error}"
                    ))
                })?;
            Ok((id, attrs))
        })
        .collect::<Result<_, CoreProviderError>>()?;
    Ok(RawSavedAttrs::from_persisted(persisted_saved).lift(registry))
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
                CoreValue::Concrete(ConcreteValue::String(s)) => {
                    Some(s.expose_plaintext().to_string())
                }
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
                field_type: proto_string_attribute_type(),
                required: true,
                description: Some("IAM role ARN to assume.".to_string()),
                block_name: None,
                provider_name: None,
                read_only: false,
                deferred_populate: false,
            },
            proto::StructField {
                name: "session_name".to_string(),
                field_type: proto_string_attribute_type(),
                required: false,
                description: Some(
                    "STS session name to associate with the assumed-role session.".to_string(),
                ),
                block_name: None,
                provider_name: None,
                read_only: false,
                deferred_populate: false,
            },
            proto::StructField {
                name: "external_id".to_string(),
                field_type: proto_string_attribute_type(),
                required: false,
                description: Some(
                    "External ID required by the trust policy of the assumed role.".to_string(),
                ),
                block_name: None,
                provider_name: None,
                read_only: false,
                deferred_populate: false,
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
                read_only: false,
                deferred_populate: false,
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

fn proto_string_attribute_type() -> proto::AttributeType {
    proto::AttributeType::String {
        pattern: None,
        length: None,
        validate: None,
        to_dsl: None,
        identity: None,
    }
}

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
                    assert!(matches!(f.field_type, proto::AttributeType::String { .. }));
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

    fn persisted_str(s: &str) -> proto::PersistedValue {
        proto::PersistedValue::String(s.to_string())
    }

    fn persisted_map(items: &[(&str, proto::PersistedValue)]) -> proto::PersistedValue {
        let mut m = std::collections::HashMap::new();
        for (k, v) in items {
            m.insert((*k).to_string(), v.clone());
        }
        proto::PersistedValue::Map(m)
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

    #[test]
    fn hydrate_read_state_rejects_saved_attrs_without_matching_state() {
        let provider = AwsccProcessProvider::new();
        let mut states = HashMap::new();
        let saved_attrs = HashMap::from([(
            "awscc.ec2.Vpc.orphan".to_string(),
            HashMap::<String, proto::PersistedValue>::new(),
        )]);

        let error = provider
            .hydrate_read_state(&mut states, &saved_attrs)
            .expect_err("an unknown saved-attribute key must not be silently discarded");

        assert!(matches!(error.kind, proto::ProviderErrorKind::Internal));
        assert!(
            error.message.contains("unknown state key"),
            "unexpected error: {}",
            error.message
        );
    }

    #[test]
    fn hydrate_read_state_round_trips_saved_write_only_redacted_secret() {
        let provider = AwsccProcessProvider::new();
        let digest = concat!(
            "$argon2id$v=19$m=19456,t=2,p=1$MDEyMzQ1Njc4OWFiY2RlZg$",
            "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA"
        )
        .to_string();
        let state_key = "awscc.ec2.Eip.example".to_string();
        let mut states = HashMap::from([(
            state_key.clone(),
            proto::PersistedState {
                id: proto::ResourceId {
                    provider: "awscc".to_string(),
                    resource_type: "ec2.Eip".to_string(),
                    identity: "example".to_string(),
                },
                identifier: Some("203.0.113.10".to_string()),
                attributes: HashMap::new(),
                exists: true,
            },
        )]);
        let saved_attrs = HashMap::from([(
            state_key,
            HashMap::from([(
                "address".to_string(),
                proto::PersistedValue::RedactedSecret(digest.clone()),
            )]),
        )]);

        provider
            .hydrate_read_state(&mut states, &saved_attrs)
            .expect("saved write-only redacted secrets must survive hydration");

        let hydrated = states
            .values()
            .next()
            .expect("hydrated state must remain present");
        assert_eq!(
            hydrated.attributes.get("address"),
            Some(&proto::PersistedValue::RedactedSecret(digest)),
            "state-side serialization must preserve the structural redacted secret"
        );
    }

    #[test]
    fn persisted_saved_attr_decode_preserves_non_secret_wire_values() {
        let distribution_id = CoreResourceId::with_provider_identity(
            "awscc",
            "cloudfront.Distribution",
            "example",
            None,
        );
        let distribution_key = distribution_id.to_string();
        let distribution_wire_attrs = HashMap::from([(
            "distribution_config".to_string(),
            persisted_map(&[
                (
                    "aliases",
                    proto::PersistedValue::List(vec![persisted_str("cdn.example.com")]),
                ),
                ("comment", persisted_str("shape parity")),
                ("enabled", proto::PersistedValue::Bool(true)),
                (
                    "default_cache_behavior",
                    persisted_map(&[("default_ttl", proto::PersistedValue::Float(60.5))]),
                ),
                (
                    "origins",
                    proto::PersistedValue::List(vec![persisted_map(&[
                        ("connection_attempts", proto::PersistedValue::Int(3)),
                        ("id", persisted_str("primary")),
                    ])]),
                ),
            ]),
        )]);
        let expected_distribution =
            convert::proto_to_core_persisted_value_map(&distribution_wire_attrs)
                .expect("ordinary persisted values should decode");

        let policy_id =
            CoreResourceId::with_provider_identity("awscc", "iam.RolePolicy", "example", None);
        let policy_key = policy_id.to_string();
        let policy_wire_attrs = HashMap::from([(
            "policy_document".to_string(),
            persisted_map(&[(
                "statement",
                proto::PersistedValue::List(vec![persisted_map(&[(
                    "action",
                    proto::PersistedValue::StringList(vec!["s3:GetObject".to_string()]),
                )])]),
            )]),
        )]);
        let expected_policy = convert::proto_to_core_persisted_value_map(&policy_wire_attrs)
            .expect("ordinary persisted values should decode");

        let saved_attrs = HashMap::from([
            (distribution_key.clone(), distribution_wire_attrs),
            (policy_key.clone(), policy_wire_attrs),
        ]);
        let key_to_id = HashMap::from([
            (distribution_key.as_str(), distribution_id.clone()),
            (policy_key.as_str(), policy_id.clone()),
        ]);

        let decoded = decode_persisted_saved_attrs(
            &saved_attrs,
            &key_to_id,
            AwsccProcessProvider::schema_registry(),
        )
        .expect("ordinary persisted values should decode");

        assert_eq!(
            decoded.as_provider_saved_attrs().get(&distribution_id),
            Some(&expected_distribution),
            "typed persisted decoding plus schema lifting must preserve strings, ints, floats, \
             bools, lists, and real-schema nested structs"
        );
        assert_eq!(
            decoded.as_provider_saved_attrs().get(&policy_id),
            Some(&expected_policy),
            "schema lifting must reconstruct the StringList wire shape for a real \
             Union[String, list(String)] field"
        );
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
