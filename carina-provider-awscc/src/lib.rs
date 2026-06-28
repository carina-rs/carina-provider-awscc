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

use carina_core::effect::PlanOp;
use carina_core::provider::{
    BoxFuture, CreateOutcome, CreateRequest, DeleteRequest, Provider, ProviderError,
    ProviderFactory, ProviderNormalizer, ProviderResult, ReadRequest, SavedAttrs, UpdateOutcome,
    UpdateRequest, merge_default_tags_for_provider, ready_noop,
};
use carina_core::resource::{ConcreteValue, DataSource, Resource, ResourceId, State, Value};
use carina_core::schema::SchemaRegistry;

use crate::provider::AwsccProviderConfig;

/// Schema extension for the AWSCC provider.
///
/// Handles provider-local state normalization and hydration of unreturned
/// attributes from saved state.
pub struct AwsccNormalizer;

impl ProviderNormalizer for AwsccNormalizer {
    // Desired-side enum canonicalization is owned by the host. Keeping
    // provider-side enum re-derivation here can re-namespace host-canonical
    // open-enum API values and leak DSL strings to AWS.
    fn normalize_desired<'a>(&'a self, _resources: &'a mut [Resource]) -> BoxFuture<'a, ()> {
        ready_noop()
    }

    fn normalize_state<'a>(
        &'a self,
        current_states: &'a mut HashMap<ResourceId, State>,
    ) -> BoxFuture<'a, ()> {
        Box::pin(async move {
            crate::provider::normalize_state_string_dsl_transforms_impl(current_states);
            // Canonicalize `Union[String, list(String)]` typed values
            // (IAM-style `string_or_list_of_strings`) so AWS's silent
            // scalar normalization no longer leaks past the provider
            // boundary. See carina-rs/carina#2481, sub-issue 5.
            crate::provider::canonicalize_string_or_list_states_impl(current_states);
        })
    }

    fn hydrate_read_state<'a>(
        &'a self,
        current_states: &'a mut HashMap<ResourceId, State>,
        saved_attrs: &'a SavedAttrs,
    ) -> BoxFuture<'a, ()> {
        Box::pin(async move {
            crate::provider::restore_unreturned_attrs_impl(current_states, saved_attrs);
        })
    }

    fn merge_default_tags<'a>(
        &'a self,
        resources: &'a mut [Resource],
        default_tags: &'a IndexMap<String, Value>,
        registry: &'a SchemaRegistry,
    ) -> BoxFuture<'a, ()> {
        Box::pin(async move {
            merge_default_tags_for_provider("awscc", resources, default_tags, registry);
        })
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
            AttributeType::enum_(
                carina_core::schema::enum_identity("Region", Some("aws")),
                Some(
                    carina_aws_types::REGIONS
                        .iter()
                        .map(|(code, _)| code.to_string())
                        .collect(),
                ),
                carina_aws_types::region_dsl_aliases(),
                None,
                None,
            ),
        );
        types.insert(
            "allowed_account_ids".to_string(),
            AttributeType::unordered_list(AttributeType::string()),
        );
        types.insert(
            "forbidden_account_ids".to_string(),
            AttributeType::unordered_list(AttributeType::string()),
        );
        types.insert("assume_role".to_string(), assume_role_attribute_type());
        types
    }

    fn validate_config(&self, attributes: &IndexMap<String, Value>) -> Result<(), String> {
        // Cross-account guardrail: when `assume_role.role_arn` parses to
        // an account id that is not in `allowed_account_ids` (when that
        // list is configured), refuse the configuration. Avoids the
        // foot-gun of an assume-role silently landing in the wrong AWS
        // account (awscc#260).
        use crate::provider::assume_role::{check_cross_account, extract_assume_role};
        let assume_role = extract_assume_role(attributes.get("assume_role"))?;
        if let Some(ar) = &assume_role {
            let allowed = extract_string_list_attr(attributes, "allowed_account_ids");
            check_cross_account(&ar.role_arn, &allowed)?;
        }
        Ok(())
    }

    fn validate_custom_type(
        &self,
        identity: &carina_core::schema::TypeIdentity,
        value: &str,
    ) -> Result<(), String> {
        use carina_core::parser::ValidatorFn;
        use std::sync::OnceLock;
        static VALIDATORS: OnceLock<HashMap<String, ValidatorFn>> = OnceLock::new();
        let validators = VALIDATORS.get_or_init(schemas::config::awscc_validators);
        // The inner map is still keyed on snake-cased semantic names —
        // project the identity's kind through `pascal_to_snake` at this
        // single boundary. Provider-axis collisions are already
        // filtered by the host before the call reaches us.
        let key = carina_core::parser::pascal_to_snake(&identity.kind);
        if let Some(validator) = validators.get(&key) {
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
    // `assume_role` is parsed here without surfacing parse errors —
    // `validate_config` runs first on the host side and would have
    // rejected a malformed block before reaching this code path. Fall
    // back to `None` defensively if it somehow slips through.
    let assume_role =
        crate::provider::assume_role::extract_assume_role(attributes.get("assume_role"))
            .ok()
            .flatten();
    AwsccProviderConfig {
        allowed_account_ids: extract_string_list_attr(attributes, "allowed_account_ids"),
        forbidden_account_ids: extract_string_list_attr(attributes, "forbidden_account_ids"),
        assume_role,
    }
}

/// Schema for the provider-level `assume_role` block. Mirrors the
/// Terraform AWS provider's `assume_role` (MVP field set: `role_arn`,
/// `session_name`, `external_id`, `duration`). When present, the
/// provider chains an `sts:AssumeRole` call on top of the ambient
/// credential chain (awscc#260).
fn assume_role_attribute_type() -> carina_core::schema::AttributeType {
    use carina_core::schema::{AttributeType, StructField};
    AttributeType::struct_(
        "AssumeRole".to_string(),
        vec![
            StructField::new("role_arn", AttributeType::string())
                .required()
                .with_description("IAM role ARN to assume."),
            StructField::new("session_name", AttributeType::string())
                .with_description("STS session name to associate with the assumed-role session."),
            StructField::new("external_id", AttributeType::string())
                .with_description("External ID required by the trust policy of the assumed role."),
            StructField::new("duration", AttributeType::duration())
                .with_description("Assumed-role session duration (e.g., 30min, 1h, 15s)."),
        ],
    )
}

/// Extract a `list(string)` provider config attribute. Non-string
/// elements are dropped silently — the host enforces type before we
/// see the value here. Used by both `extract_provider_config` and the
/// cross-account guardrail in `validate_config`.
fn extract_string_list_attr(attributes: &IndexMap<String, Value>, key: &str) -> Vec<String> {
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

impl Provider for AwsccProvider {
    fn name(&self) -> &str {
        "awscc"
    }

    fn required_permissions(&self, id: &ResourceId, op: PlanOp) -> Vec<String> {
        schemas::generated::required_permissions(&id.resource_type, op)
            .iter()
            .map(|permission| (*permission).to_string())
            .collect()
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
            self.read_resource(
                &id.resource_type,
                id.identity_or_empty(),
                identifier.as_deref(),
            )
            .await
        })
    }

    fn read_data_source(&self, resource: &DataSource) -> BoxFuture<'_, ProviderResult<State>> {
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
            self.read_resource(&id.resource_type, id.identity_or_empty(), None)
                .await
        })
    }

    fn create(
        &self,
        _id: &ResourceId,
        request: CreateRequest,
    ) -> BoxFuture<'_, ProviderResult<CreateOutcome>> {
        if let Some(err) = self.init_error() {
            let err = err.to_string();
            let id = request.resource.as_resource().id.clone();
            return Box::pin(async move {
                Err(ProviderError::invalid_input(err)
                    .for_provider("awscc")
                    .for_resource(id))
            });
        }
        Box::pin(async move { self.create_resource(request.resource.as_resource()).await })
    }

    fn update(
        &self,
        id: &ResourceId,
        identifier: &str,
        request: UpdateRequest,
    ) -> BoxFuture<'_, ProviderResult<UpdateOutcome>> {
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
            types.get("allowed_account_ids").map(|t| t
                .shape_ref_free()
                .expect("provider config types are Ref-free")),
            Some(carina_core::schema::Shape::List { .. })
        ));
        assert!(matches!(
            types.get("forbidden_account_ids").map(|t| t
                .shape_ref_free()
                .expect("provider config types are Ref-free")),
            Some(carina_core::schema::Shape::List { .. })
        ));
    }

    #[test]
    fn provider_config_attribute_types_declares_assume_role_struct() {
        // The host validates the assume_role block's shape against this
        // Struct declaration before calling `initialize`. If the
        // declaration disappears or its required field changes, the
        // host stops surfacing schema-level errors for malformed
        // assume_role blocks — this test prevents that regression
        // (awscc#260).
        let factory = AwsccProviderFactory;
        let types = factory.provider_config_attribute_types();
        let ty = types
            .get("assume_role")
            .expect("assume_role must be declared as a provider config attribute");
        match ty
            .shape_ref_free()
            .expect("provider config types are Ref-free")
        {
            carina_core::schema::Shape::Struct { name } => {
                assert_eq!(name, "AssumeRole");
                let fields = ty
                    .struct_fields_ref_free_with_budget(
                        &mut carina_core::schema::ShapeWalkBudget::new(16),
                    )
                    .expect("provider config types are Ref-free")
                    .expect("assume_role should expose struct fields");
                let role_arn = fields
                    .iter()
                    .find(|f| f.name == "role_arn")
                    .expect("assume_role.role_arn must be declared");
                assert!(role_arn.required, "role_arn must be required");
                for opt in ["session_name", "external_id", "duration"] {
                    let f = fields
                        .iter()
                        .find(|f| f.name == opt)
                        .unwrap_or_else(|| panic!("assume_role.{opt} must be declared"));
                    assert!(!f.required, "assume_role.{opt} must be optional");
                }
            }
            other => panic!("assume_role must be a Struct, was {other:?}"),
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    async fn provider_for_required_permissions_tests() -> AwsccProvider {
        use aws_config::BehaviorVersion;
        use aws_config::Region;
        use winterbaume_cloudcontrol::CloudControlService;
        use winterbaume_core::MockAws;

        let mock = MockAws::builder()
            .with_service(CloudControlService::new())
            .build();
        let config = aws_config::defaults(BehaviorVersion::latest())
            .http_client(mock.http_client())
            .credentials_provider(mock.credentials_provider())
            .region(Region::new("us-east-1"))
            .load()
            .await;

        AwsccProvider::from_sdk_config(config, &AwsccProviderConfig::default()).await
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[tokio::test]
    async fn required_permissions_returns_cfn_handler_actions_for_known_resource() {
        let provider = provider_for_required_permissions_tests().await;
        let id = ResourceId::with_provider_identity(
            "awscc",
            "elasticloadbalancingv2.LoadBalancer",
            "test",
            None,
        );

        let permissions = Provider::required_permissions(&provider, &id, PlanOp::Create);

        assert!(!permissions.is_empty());
        assert!(
            permissions
                .iter()
                .any(|p| p == "elasticloadbalancing:CreateLoadBalancer"),
            "expected LoadBalancer create permissions to include \
             elasticloadbalancing:CreateLoadBalancer, got {permissions:?}"
        );
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[tokio::test]
    async fn required_permissions_returns_empty_vec_for_unknown_resource() {
        let provider = provider_for_required_permissions_tests().await;
        let id = ResourceId::with_provider_identity("awscc", "example.Unknown", "test", None);

        let permissions = Provider::required_permissions(&provider, &id, PlanOp::Create);

        assert!(permissions.is_empty());
    }

    fn list_of(items: &[&str]) -> Value {
        Value::Concrete(ConcreteValue::List(
            items
                .iter()
                .map(|s| Value::Concrete(ConcreteValue::String((*s).to_string())))
                .collect(),
        ))
    }

    fn map_of(items: &[(&str, Value)]) -> Value {
        let mut m = IndexMap::new();
        for (k, v) in items {
            m.insert((*k).to_string(), v.clone());
        }
        Value::Concrete(ConcreteValue::Map(m))
    }

    #[test]
    fn validate_config_accepts_assume_role_in_allowed_account() {
        let factory = AwsccProviderFactory;
        let mut attrs: IndexMap<String, Value> = IndexMap::new();
        attrs.insert(
            "allowed_account_ids".to_string(),
            list_of(&["412038850359"]),
        );
        attrs.insert(
            "assume_role".to_string(),
            map_of(&[(
                "role_arn",
                Value::Concrete(ConcreteValue::String(
                    "arn:aws:iam::412038850359:role/delegation".to_string(),
                )),
            )]),
        );
        factory
            .validate_config(&attrs)
            .expect("matching allowed_account_ids must validate");
    }

    #[test]
    fn validate_config_rejects_assume_role_outside_allowed_account() {
        let factory = AwsccProviderFactory;
        let mut attrs: IndexMap<String, Value> = IndexMap::new();
        attrs.insert(
            "allowed_account_ids".to_string(),
            list_of(&["111111111111"]),
        );
        attrs.insert(
            "assume_role".to_string(),
            map_of(&[(
                "role_arn",
                Value::Concrete(ConcreteValue::String(
                    "arn:aws:iam::412038850359:role/delegation".to_string(),
                )),
            )]),
        );
        let err = factory
            .validate_config(&attrs)
            .expect_err("cross-account role outside allowed_account_ids must fail");
        assert!(err.contains("412038850359"), "must name target: {err}");
        assert!(err.contains("111111111111"), "must name allow list: {err}");
    }

    #[test]
    fn validate_config_no_assume_role_is_noop() {
        let factory = AwsccProviderFactory;
        let mut attrs: IndexMap<String, Value> = IndexMap::new();
        attrs.insert(
            "allowed_account_ids".to_string(),
            list_of(&["412038850359"]),
        );
        factory
            .validate_config(&attrs)
            .expect("validate_config without assume_role must be a no-op");
    }

    #[test]
    fn extract_provider_config_reads_assume_role() {
        let mut attrs: IndexMap<String, Value> = IndexMap::new();
        attrs.insert(
            "assume_role".to_string(),
            map_of(&[
                (
                    "role_arn",
                    Value::Concrete(ConcreteValue::String(
                        "arn:aws:iam::412038850359:role/delegation".to_string(),
                    )),
                ),
                (
                    "session_name",
                    Value::Concrete(ConcreteValue::String("carina".to_string())),
                ),
                (
                    "duration",
                    Value::Concrete(ConcreteValue::Duration(std::time::Duration::from_secs(
                        1800,
                    ))),
                ),
            ]),
        );
        let cfg = extract_provider_config(&attrs);
        let ar = cfg.assume_role.expect("assume_role must be parsed");
        assert_eq!(ar.role_arn, "arn:aws:iam::412038850359:role/delegation");
        assert_eq!(ar.session_name.as_deref(), Some("carina"));
        assert_eq!(ar.duration, Some(std::time::Duration::from_secs(1800)));
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
        use carina_core::schema::{AttributeType, RawShape};

        let factory = AwsccProviderFactory;
        let schema = factory
            .schemas()
            .into_iter()
            .find(|s| s.resource_type == "cloudfront.Distribution")
            .expect("cloudfront.Distribution schema must be present");

        // Recursively collect the `ordered` flag of every List whose
        // owning struct field is named allowed_methods/cached_methods.
        // carina#3340: this walker resolves `AttributeType::Ref` against
        // `schema.defs` so cycle-broken / shared struct subtrees stay
        // reachable. carina#3349 + #3352: use `raw_shape()` (the
        // Ref-preserving projection) so we can short-circuit on the
        // second visit; `shape(defs)` would auto-resolve and infinite-loop
        // on a self-referential `Ref("X") -> Struct { ..Ref("X").. }`.
        fn collect(
            at: &AttributeType,
            field: Option<&str>,
            defs: &std::collections::BTreeMap<String, AttributeType>,
            seen: &mut std::collections::HashSet<String>,
            out: &mut Vec<(String, bool)>,
        ) {
            match at.raw_shape() {
                RawShape::Ref(name) => {
                    if seen.insert(name.to_string())
                        && let Some(target) = defs.get(name)
                    {
                        collect(target, field, defs, seen, out);
                    }
                }
                RawShape::List {
                    element_type: inner,
                    ordered,
                    ..
                } => {
                    if matches!(field, Some("allowed_methods" | "cached_methods")) {
                        out.push((field.unwrap().to_string(), ordered));
                    }
                    collect(inner, None, defs, seen, out);
                }
                RawShape::Struct { fields, .. } => {
                    for f in fields {
                        collect(&f.field_type, Some(f.name.as_str()), defs, seen, out);
                    }
                }
                RawShape::Map { value, .. } => collect(value, None, defs, seen, out),
                _ => {}
            }
        }

        let mut found = Vec::new();
        for attr in schema.attributes.values() {
            let mut seen = std::collections::HashSet::new();
            collect(&attr.attr_type, None, &schema.defs, &mut seen, &mut found);
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

    #[tokio::test]
    async fn test_merge_default_tags_resource_tags_win() {
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
        normalizer
            .merge_default_tags(&mut resources, &default_tags, &schemas)
            .await;

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

    #[tokio::test]
    async fn test_merge_default_tags_no_explicit_tags() {
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
        normalizer
            .merge_default_tags(&mut resources, &default_tags, &schemas)
            .await;

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

    #[tokio::test]
    async fn test_merge_default_tags_skips_no_tag_schema() {
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
        normalizer
            .merge_default_tags(&mut resources, &default_tags, &schemas)
            .await;

        assert!(!resources[0].attributes.contains_key("tags"));
        assert!(!resources[0].attributes.contains_key("_default_tag_keys"));
    }

    #[tokio::test]
    async fn test_merge_default_tags_no_default_tags() {
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
        normalizer
            .merge_default_tags(&mut resources, &default_tags, &schemas)
            .await;

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
