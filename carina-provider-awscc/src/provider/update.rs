//! Update patch building and resource property parsing.
//!
//! This module maps `carina_core::provider::UpdatePatch` ops directly to
//! CloudControl JSON Patch operations. The patch carries only the
//! attributes the user explicitly specified or removed — fields the user
//! has never specified do not appear in the patch and therefore generate
//! no JSON Patch op, leaving CloudControl-managed defaults and
//! sibling-resource state alone (the actual fix for
//! `carina-rs/carina#2559`).

use carina_core::provider::{PatchOp, ProviderError, ProviderResult, UpdatePatch};
use serde_json::json;

use super::conversion::dsl_value_to_aws_with_defs;
use crate::schemas::config::AwsccSchemaConfig;

/// Parse a JSON string from CloudControl API response into a `serde_json::Value`.
///
/// Returns an error instead of silently returning an empty object when the JSON is malformed.
pub(crate) fn parse_resource_properties(props_str: &str) -> ProviderResult<serde_json::Value> {
    serde_json::from_str(props_str)
        .map_err(|e| ProviderError::internal_caused_by("Failed to parse resource properties", e))
}

/// A type-tied update plan containing the JSON Patch sent to Cloud Control, the DSL
/// attributes whose requested values need presence confirmation, and every schema-known
/// attribute touched by the patch for the case where no read-back is available.
#[derive(Debug, Default)]
pub(crate) struct UpdatePatchPlan {
    pub(crate) ops: Vec<serde_json::Value>,
    pub(crate) requires_confirmation: Vec<String>,
    pub(crate) unsent: Vec<String>,
    pub(crate) touched: Vec<String>,
}

impl UpdatePatchPlan {
    fn touch(&mut self, key: &str) {
        if !self.touched.iter().any(|existing| existing == key) {
            self.touched.push(key.to_string());
        }
    }

    fn require_confirmation(&mut self, key: &str) {
        if !self
            .requires_confirmation
            .iter()
            .any(|existing| existing == key)
        {
            self.requires_confirmation.push(key.to_string());
        }
    }

    fn mark_unsent(&mut self, key: &str) {
        if !self.unsent.iter().any(|existing| existing == key) {
            self.unsent.push(key.to_string());
        }
    }
}

/// Build a Cloud Control update plan from an [`UpdatePatch`].
///
/// Each [`PatchOp`] in `patch.ops` corresponds to a key the user explicitly added,
/// replaced, or removed in the desired state, or an unchanged write-only key that
/// must be reasserted alongside another update. Attributes the user has never specified
/// do not appear in `patch.ops` and therefore generate no JSON Patch op — CloudControl
/// leaves them untouched.
///
/// Mapping rules:
/// - [`PatchOp::Add`], [`PatchOp::Replace`], and [`PatchOp::Reassert`] →
///   `{"op": "add", "path": "/<aws_name>", "value": <serialized>}`
///   (`add` is used for both because CloudControl's JSON Patch
///   `replace` fails when the property does not exist in the resource
///   model, while `add` works for both create-and-set and replace-existing
///   cases — matches the Terraform AWSCC provider's behavior).
/// - [`PatchOp::Remove`] →
///   `{"op": "remove", "path": "/<aws_name>"}`.
///
/// Read-only and create-only attributes are filtered out: CloudControl rejects
/// patches that touch them. Add/Replace operations that are filtered out remain in
/// the plan's confirmation metadata so their desired values cannot later be fabricated
/// into state. Reassert operations never require read-back confirmation because their
/// write-only values cannot appear in the read model.
///
/// `tags` is converted from the per-key DSL `Map<String, String>` through
/// the schema-declared CloudControl wire shape.
pub(crate) fn build_update_patches(
    config: &AwsccSchemaConfig,
    resource_type: &str,
    patch: &UpdatePatch,
) -> UpdatePatchPlan {
    let mut plan = UpdatePatchPlan::default();

    for op in &patch.ops {
        let key = op.key();
        let attr_schema = match config.schema.attributes().get(key) {
            Some(s) => s,
            // Unknown keys (not in the schema) are silently dropped:
            // CloudControl will reject anything we don't have an AWS
            // path for, and surfacing this as an error here would mask
            // schema bugs as user errors.
            None => continue,
        };
        plan.touch(key);

        let requires_confirmation = match op {
            PatchOp::Add { .. } | PatchOp::Replace { .. } => !attr_schema.is_write_only(),
            PatchOp::Reassert { .. } | PatchOp::Remove { .. } => false,
        };
        if requires_confirmation {
            plan.require_confirmation(key);
        }

        // CloudControl rejects patches that touch read-only or create-only
        // properties. Drop them even if they appear in the patch.
        if attr_schema.is_read_only() || attr_schema.is_create_only() {
            if requires_confirmation {
                plan.mark_unsent(key);
            }
            continue;
        }

        if key == "tags" {
            let Some(tags_property) = config.tags.as_ref() else {
                if requires_confirmation {
                    plan.mark_unsent(key);
                }
                continue;
            };
            if let Some(patch_op) = tags_property.update_patch(op) {
                plan.ops.push(patch_op);
            } else if requires_confirmation {
                plan.mark_unsent(key);
            }
            continue;
        }

        let provider_name = attr_schema.provider_name();
        let aws_name = match provider_name {
            Some(name) => name,
            None => {
                if requires_confirmation {
                    plan.mark_unsent(key);
                }
                continue;
            }
        };

        match op {
            PatchOp::Add { value, .. }
            | PatchOp::Replace { value, .. }
            | PatchOp::Reassert { value, .. } => {
                if let Some(aws_value) = dsl_value_to_aws_with_defs(
                    value,
                    attr_schema.attr_type(),
                    resource_type,
                    key,
                    config.schema.defs(),
                ) {
                    plan.ops.push(json!({
                        "op": "add",
                        "path": format!("/{}", aws_name),
                        "value": aws_value,
                    }));
                } else if requires_confirmation {
                    plan.mark_unsent(key);
                }
            }
            PatchOp::Remove { .. } => {
                plan.ops.push(json!({
                    "op": "remove",
                    "path": format!("/{}", aws_name),
                }));
            }
        }
    }

    plan
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use carina_core::binding_index::ResolvedBindings;
    use carina_core::executor::{
        ModuleConstraintGate, ProviderPreparationContext, prepare_provider_ready_resource,
    };
    use carina_core::provider::{PatchOp, build_update_patch as build_core_update_patch};
    use carina_core::resource::{
        ConcreteValue, DeferredValue, Resource, State, UnknownReason, Value,
    };
    use carina_core::schema::{AttributeSchema, AttributeType, ResourceSchema, SchemaRegistry};
    use indexmap::IndexMap;

    use super::*;
    use crate::AwsccNormalizer;

    fn get_schema_config(resource_type: &str) -> Option<&'static AwsccSchemaConfig> {
        super::super::get_schema_config(resource_type)
    }

    fn get_vpc_config() -> &'static AwsccSchemaConfig {
        get_schema_config("ec2.Vpc").expect("ec2.vpc schema should exist")
    }

    fn replace(key: &str, value: Value) -> PatchOp {
        PatchOp::Replace {
            key: key.to_string(),
            value,
        }
    }

    fn add(key: &str, value: Value) -> PatchOp {
        PatchOp::Add {
            key: key.to_string(),
            value,
        }
    }

    fn reassert(key: &str, value: Value) -> PatchOp {
        PatchOp::Reassert {
            key: key.to_string(),
            value,
        }
    }

    fn remove(key: &str) -> PatchOp {
        PatchOp::Remove {
            key: key.to_string(),
        }
    }

    fn string(value: &str) -> Value {
        Value::Concrete(ConcreteValue::String(value.to_string().into()))
    }

    fn bool_(value: bool) -> Value {
        Value::Concrete(ConcreteValue::Bool(value))
    }

    fn map(entries: impl IntoIterator<Item = (&'static str, Value)>) -> Value {
        Value::Concrete(ConcreteValue::Map(
            entries
                .into_iter()
                .map(|(key, value)| (key.to_string(), value))
                .collect(),
        ))
    }

    #[test]
    fn test_parse_resource_properties_valid_json() {
        let json_str = r#"{"VpcId": "vpc-123", "CidrBlock": "10.0.0.0/16"}"#;
        let result = parse_resource_properties(json_str);
        assert!(result.is_ok());
        let value = result.unwrap();
        assert_eq!(value["VpcId"], "vpc-123");
        assert_eq!(value["CidrBlock"], "10.0.0.0/16");
    }

    #[test]
    fn test_parse_resource_properties_malformed_json_returns_error() {
        let malformed = r#"{"VpcId": "vpc-123", invalid"#;
        let result = parse_resource_properties(malformed);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(
            err.detail()
                .context()
                .contains("Failed to parse resource properties"),
            "Expected error message about parsing, got: {}",
            err
        );
        assert!(
            matches!(err, ProviderError::Internal(_)),
            "Expected Internal variant, got: {err:?}"
        );
    }

    #[test]
    fn test_parse_resource_properties_empty_string_returns_error() {
        let result = parse_resource_properties("");
        assert!(result.is_err());
    }

    /// Regression test for `carina-rs/carina#2559`: a user `.crn` that
    /// changes only `tags` on `aws.iam.Role` must produce exactly one
    /// JSON Patch op (`add /Tags`). It must NOT also emit a
    /// `remove /Policies` (or any other `remove`) for fields the user
    /// never specified, because that clobbers sibling
    /// `aws.iam.RolePolicy` resources.
    #[test]
    fn issue_2559_tags_only_patch_does_not_touch_other_fields() {
        let config = get_schema_config("iam.Role").expect("iam.role schema should exist");

        let mut tags = IndexMap::new();
        tags.insert(
            "Env".to_string(),
            Value::Concrete(ConcreteValue::String("staging".to_string().into())),
        );
        let patch = UpdatePatch {
            ops: vec![replace("tags", Value::Concrete(ConcreteValue::Map(tags)))],
        };

        let patches = build_update_patches(config, "iam.Role", &patch).ops;

        assert_eq!(
            patches.len(),
            1,
            "Tags-only patch should produce exactly one JSON Patch op, got: {patches:?}"
        );
        let op = &patches[0];
        assert_eq!(op.get("op").and_then(|v| v.as_str()), Some("add"));
        assert_eq!(op.get("path").and_then(|v| v.as_str()), Some("/Tags"));

        // Defensive: no `remove` op should be present.
        assert!(
            !patches
                .iter()
                .any(|p| p.get("op").and_then(|v| v.as_str()) == Some("remove")),
            "Tags-only patch must not emit any /remove op (caused #2559 by clobbering /Policies); got: {patches:?}"
        );
    }

    #[test]
    fn test_empty_patch_produces_no_ops() {
        let config = get_vpc_config();
        let patch = UpdatePatch::default();
        let patches = build_update_patches(config, "ec2.Vpc", &patch).ops;
        assert!(
            patches.is_empty(),
            "Empty patch must produce no JSON Patch ops, got: {patches:?}"
        );
    }

    #[test]
    fn test_replace_op_emits_add_json_patch() {
        // CloudControl JSON Patch: "add" works whether the property exists or not,
        // while "replace" fails if the property doesn't exist in the model.
        // We always emit "add" for set operations, matching Terraform AWSCC.
        let config = get_vpc_config();
        let patch = UpdatePatch {
            ops: vec![replace(
                "instance_tenancy",
                Value::Concrete(ConcreteValue::String(
                    "aws.ec2.Vpc.InstanceTenancy.dedicated".to_string().into(),
                )),
            )],
        };
        let patches = build_update_patches(config, "ec2.Vpc", &patch).ops;

        assert_eq!(patches.len(), 1, "got: {patches:?}");
        let op = &patches[0];
        assert_eq!(op.get("op").and_then(|v| v.as_str()), Some("add"));
        assert_eq!(
            op.get("path").and_then(|v| v.as_str()),
            Some("/InstanceTenancy")
        );
    }

    #[test]
    fn test_add_op_emits_add_json_patch() {
        let config =
            get_schema_config("logs.LogGroup").expect("logs.log_group schema should exist");
        let patch = UpdatePatch {
            ops: vec![add(
                "retention_in_days",
                Value::Concrete(ConcreteValue::Int(14)),
            )],
        };
        let patches = build_update_patches(config, "logs.LogGroup", &patch).ops;

        assert_eq!(patches.len(), 1, "got: {patches:?}");
        let op = &patches[0];
        assert_eq!(op.get("op").and_then(|v| v.as_str()), Some("add"));
        assert_eq!(
            op.get("path").and_then(|v| v.as_str()),
            Some("/RetentionInDays")
        );
        assert_eq!(op.get("value"), Some(&serde_json::json!(14)));
    }

    #[test]
    fn test_reassert_op_emits_converted_add_without_confirmation() {
        let config = get_schema_config("ssm.Parameter").expect("ssm.Parameter schema should exist");
        let patch = UpdatePatch {
            ops: vec![reassert("tier", string("Advanced"))],
        };

        let plan = build_update_patches(config, "ssm.Parameter", &patch);

        assert_eq!(
            plan.ops,
            [json!({"op": "add", "path": "/Tier", "value": "Advanced"})]
        );
        assert!(plan.requires_confirmation.is_empty());
        assert!(plan.unsent.is_empty());
        assert_eq!(plan.touched, ["tier"]);
    }

    #[tokio::test]
    async fn cloudfront_function_code_update_reasserts_auto_publish_without_confirmation() {
        let config = get_schema_config("cloudfront.Function")
            .expect("cloudfront.Function schema should exist");
        let desired =
            Resource::with_provider("awscc", "cloudfront.Function", "publish_update", None)
                .with_attribute("name", string("carina-acc-test-fn-publish"))
                .with_attribute(
                    "function_code",
                    string("function handler(event) { return event.request; }"),
                )
                .with_attribute(
                    "function_config",
                    map([
                        ("comment", string("Publish update test")),
                        (
                            "runtime",
                            Value::Concrete(ConcreteValue::enum_identifier("cloudfront_js_2_0")),
                        ),
                    ]),
                )
                .with_attribute("auto_publish", bool_(true));
        let from = State::existing(
            desired.id.clone(),
            HashMap::from([(
                "function_code".to_string(),
                string("function handler(event) { return event.request.uri; }"),
            )]),
        );

        let bindings = ResolvedBindings::default();
        let module_gate = ModuleConstraintGate::new(&[]);
        let mut schemas = SchemaRegistry::new();
        for schema in crate::schemas::all_schemas() {
            schemas.insert("awscc", schema);
        }
        let preparation = ProviderPreparationContext::new(
            &bindings,
            &module_gate,
            &[],
            &AwsccNormalizer,
            &[],
            &schemas,
        );
        let desired = prepare_provider_ready_resource(desired, &preparation)
            .await
            .expect("CloudFront Function should pass provider preparation");
        let patch = build_core_update_patch(
            &["function_code".to_string()],
            &desired,
            &from,
            &config.schema,
        );

        assert!(matches!(
            patch.ops.as_slice(),
            [
                PatchOp::Replace { key: function_code, .. },
                PatchOp::Reassert { key: auto_publish, value }
            ] if function_code == "function_code"
                && auto_publish == "auto_publish"
                && value == &bool_(true)
        ));

        let plan = build_update_patches(config, "cloudfront.Function", &patch);

        assert_eq!(
            plan.ops,
            [
                json!({
                    "op": "add",
                    "path": "/FunctionCode",
                    "value": "function handler(event) { return event.request; }",
                }),
                json!({"op": "add", "path": "/AutoPublish", "value": true}),
            ]
        );
        assert_eq!(
            plan.requires_confirmation,
            ["function_code"],
            "the write-only AutoPublish reassertion must not require read-back confirmation"
        );
        assert!(plan.unsent.is_empty());
        assert_eq!(plan.touched, ["function_code", "auto_publish"]);
    }

    #[test]
    fn test_remove_op_emits_remove_json_patch() {
        let config = get_vpc_config();
        let patch = UpdatePatch {
            ops: vec![remove("instance_tenancy")],
        };
        let patches = build_update_patches(config, "ec2.Vpc", &patch).ops;

        assert_eq!(patches.len(), 1, "got: {patches:?}");
        let op = &patches[0];
        assert_eq!(op.get("op").and_then(|v| v.as_str()), Some("remove"));
        assert_eq!(
            op.get("path").and_then(|v| v.as_str()),
            Some("/InstanceTenancy")
        );
        assert!(op.get("value").is_none());
    }

    #[test]
    fn test_read_only_attribute_is_dropped() {
        // Regression for issue #806: update patch must never include
        // read-only properties (e.g. Arn). CloudControl rejects patches
        // that touch them. The new mapper drops them silently rather
        // than erroring out, on the theory that the diff layer
        // shouldn't have generated the op in the first place — this is
        // a defensive filter, not a primary check.
        let config =
            get_schema_config("logs.LogGroup").expect("logs.log_group schema should exist");
        let patch = UpdatePatch {
            ops: vec![replace(
                "arn",
                Value::Concrete(ConcreteValue::String("arn:aws:logs:::*".to_string().into())),
            )],
        };
        let plan = build_update_patches(config, "logs.LogGroup", &patch);
        assert!(
            plan.ops.is_empty(),
            "read-only attribute must be dropped, got: {:?}",
            plan.ops
        );
        assert_eq!(plan.requires_confirmation, ["arn"]);
        assert_eq!(plan.unsent, ["arn"]);
    }

    #[test]
    fn test_reassert_read_only_and_create_only_attributes_are_dropped() {
        let config = AwsccSchemaConfig {
            aws_type_name: "AWS::Test::Resource",
            resource_type_name: "test.Resource",
            primary_identifier: &[],
            tags: None,
            schema: ResourceSchema::new("test.Resource")
                .attribute(
                    AttributeSchema::new("read_only", AttributeType::string())
                        .read_only()
                        .with_provider_name("ReadOnly"),
                )
                .attribute(
                    AttributeSchema::new("create_only", AttributeType::string())
                        .create_only()
                        .with_provider_name("CreateOnly"),
                ),
        };
        let patch = UpdatePatch {
            ops: vec![
                reassert("read_only", string("read")),
                reassert("create_only", string("create")),
            ],
        };

        let plan = build_update_patches(&config, "test.Resource", &patch);

        assert!(plan.ops.is_empty());
        assert!(plan.requires_confirmation.is_empty());
        assert!(plan.unsent.is_empty());
        assert_eq!(plan.touched, ["read_only", "create_only"]);
    }

    #[test]
    fn test_attribute_without_provider_name_is_tracked_as_unsent() {
        let config = AwsccSchemaConfig {
            aws_type_name: "AWS::Test::Resource",
            resource_type_name: "test.Resource",
            primary_identifier: &[],
            tags: None,
            schema: ResourceSchema::new("test.Resource").attribute(AttributeSchema::new(
                "unmapped_attribute",
                AttributeType::string(),
            )),
        };
        let patch = UpdatePatch {
            ops: vec![replace(
                "unmapped_attribute",
                Value::Concrete(ConcreteValue::String("requested".to_string().into())),
            )],
        };

        let plan = build_update_patches(&config, "test.Resource", &patch);

        assert!(plan.ops.is_empty());
        assert_eq!(plan.requires_confirmation, ["unmapped_attribute"]);
        assert_eq!(plan.unsent, ["unmapped_attribute"]);
    }

    #[test]
    fn test_unconvertible_value_is_tracked_as_unsent() {
        let config = get_vpc_config();
        let patch = UpdatePatch {
            ops: vec![replace(
                "instance_tenancy",
                Value::Deferred(DeferredValue::Unknown(UnknownReason::ForValue)),
            )],
        };

        let plan = build_update_patches(config, "ec2.Vpc", &patch);

        assert!(plan.ops.is_empty());
        assert_eq!(plan.requires_confirmation, ["instance_tenancy"]);
        assert_eq!(plan.unsent, ["instance_tenancy"]);
    }

    #[test]
    fn test_unconvertible_tags_value_is_tracked_as_unsent_without_remove_op() {
        let config = get_vpc_config();
        let patch = UpdatePatch {
            ops: vec![add(
                "tags",
                Value::Deferred(DeferredValue::Unknown(UnknownReason::ForValue)),
            )],
        };

        let plan = build_update_patches(config, "ec2.Vpc", &patch);

        assert!(
            plan.ops.is_empty(),
            "an unresolved tags value must not emit a destructive remove"
        );
        assert_eq!(plan.requires_confirmation, ["tags"]);
        assert_eq!(plan.unsent, ["tags"]);
    }

    #[test]
    fn test_partially_unconvertible_tags_map_is_tracked_as_unsent_without_op() {
        let config = get_vpc_config();
        let mut tags = IndexMap::new();
        tags.insert(
            "projectable".to_string(),
            Value::Concrete(ConcreteValue::String("kept".to_string().into())),
        );
        tags.insert(
            "unresolved".to_string(),
            Value::Deferred(DeferredValue::Unknown(UnknownReason::ForValue)),
        );
        let patch = UpdatePatch {
            ops: vec![replace("tags", Value::Concrete(ConcreteValue::Map(tags)))],
        };

        let plan = build_update_patches(config, "ec2.Vpc", &patch);

        assert!(
            plan.ops.is_empty(),
            "a partially unresolved tags map must not emit a lossy replacement"
        );
        assert_eq!(plan.requires_confirmation, ["tags"]);
        assert_eq!(plan.unsent, ["tags"]);
    }

    #[test]
    fn test_remove_tags_emits_tags_remove() {
        let config = get_vpc_config();
        let patch = UpdatePatch {
            ops: vec![remove("tags")],
        };
        let plan = build_update_patches(config, "ec2.Vpc", &patch);

        assert_eq!(plan.ops.len(), 1);
        assert!(plan.requires_confirmation.is_empty());
        assert!(plan.unsent.is_empty());
        let op = &plan.ops[0];
        assert_eq!(op.get("op").and_then(|v| v.as_str()), Some("remove"));
        assert_eq!(op.get("path").and_then(|v| v.as_str()), Some("/Tags"));
    }

    #[test]
    fn test_replace_tags_projects_to_aws_shape() {
        let config = get_vpc_config();
        let mut tags = IndexMap::new();
        tags.insert(
            "Name".to_string(),
            Value::Concrete(ConcreteValue::String("my-vpc".to_string().into())),
        );
        tags.insert(
            "Env".to_string(),
            Value::Concrete(ConcreteValue::String("prod".to_string().into())),
        );
        let patch = UpdatePatch {
            ops: vec![replace("tags", Value::Concrete(ConcreteValue::Map(tags)))],
        };
        let plan = build_update_patches(config, "ec2.Vpc", &patch);

        assert_eq!(plan.ops.len(), 1);
        assert_eq!(plan.requires_confirmation, ["tags"]);
        assert!(plan.unsent.is_empty());
        let op = &plan.ops[0];
        assert_eq!(op.get("op").and_then(|v| v.as_str()), Some("add"));
        assert_eq!(op.get("path").and_then(|v| v.as_str()), Some("/Tags"));
        let arr = op
            .get("value")
            .and_then(|v| v.as_array())
            .expect("Tags should be array");
        assert_eq!(arr.len(), 2);
        assert!(
            arr.iter()
                .any(|t| t.get("Key").and_then(|v| v.as_str()) == Some("Name")
                    && t.get("Value").and_then(|v| v.as_str()) == Some("my-vpc"))
        );
    }

    #[test]
    fn test_ssm_parameter_replace_tags_projects_to_map_shape() {
        let config = get_schema_config("ssm.Parameter").expect("ssm.Parameter schema should exist");
        let tags = IndexMap::from([(
            "Env".to_string(),
            Value::Concrete(ConcreteValue::String("prod".to_string().into())),
        )]);
        let patch = UpdatePatch {
            ops: vec![replace("tags", Value::Concrete(ConcreteValue::Map(tags)))],
        };

        let plan = build_update_patches(config, "ssm.Parameter", &patch);

        assert_eq!(
            plan.ops,
            [json!({
                "op": "add",
                "path": "/Tags",
                "value": {"Env": "prod"},
            })]
        );
        assert_eq!(plan.requires_confirmation, ["tags"]);
        assert!(plan.unsent.is_empty());
    }

    #[test]
    fn test_ssm_parameter_value_update_is_in_place() {
        let config = get_schema_config("ssm.Parameter").expect("ssm.Parameter schema should exist");
        let patch = UpdatePatch {
            ops: vec![replace(
                "value",
                Value::Concrete(ConcreteValue::String("v2".to_string().into())),
            )],
        };

        let plan = build_update_patches(config, "ssm.Parameter", &patch);

        assert_eq!(
            plan.ops,
            [json!({"op": "add", "path": "/Value", "value": "v2"})]
        );
        assert_eq!(plan.requires_confirmation, ["value"]);
        assert!(plan.unsent.is_empty());
    }

    #[test]
    fn test_cloudfront_response_headers_policy_security_headers_update_is_in_place() {
        let config = get_schema_config("cloudfront.ResponseHeadersPolicy")
            .expect("cloudfront.ResponseHeadersPolicy schema should exist");
        let attr = config
            .schema
            .attributes()
            .get("response_headers_policy_config")
            .expect("response_headers_policy_config should exist");
        assert!(!attr.is_create_only());

        let patch = UpdatePatch {
            ops: vec![replace(
                "response_headers_policy_config",
                map([
                    ("name", string("carina-registry-security-headers")),
                    (
                        "security_headers_config",
                        map([
                            (
                                "content_security_policy",
                                map([
                                    ("content_security_policy", string("default-src 'self'")),
                                    ("override", bool_(true)),
                                ]),
                            ),
                            ("content_type_options", map([("override", bool_(true))])),
                        ]),
                    ),
                ]),
            )],
        };

        let plan = build_update_patches(config, "cloudfront.ResponseHeadersPolicy", &patch);

        assert_eq!(
            plan.ops,
            [json!({
                "op": "add",
                "path": "/ResponseHeadersPolicyConfig",
                "value": {
                    "Name": "carina-registry-security-headers",
                    "SecurityHeadersConfig": {
                        "ContentSecurityPolicy": {
                            "ContentSecurityPolicy": "default-src 'self'",
                            "Override": true,
                        },
                        "ContentTypeOptions": {
                            "Override": true,
                        },
                    },
                },
            })]
        );
        assert_eq!(
            plan.requires_confirmation,
            ["response_headers_policy_config"]
        );
        assert!(plan.unsent.is_empty());
    }

    #[test]
    fn test_tags_patch_uses_schema_provider_name_for_path() {
        let config = AwsccSchemaConfig {
            aws_type_name: "AWS::Test::Resource",
            resource_type_name: "test.Resource",
            primary_identifier: &[],
            tags: Some(crate::schemas::config::TagsProperty {
                provider_name: "ResourceTags",
                wire: crate::schemas::config::TagsWire::KeyValueList,
            }),
            schema: ResourceSchema::new("test.Resource").attribute(
                AttributeSchema::new("tags", carina_aws_types::tags_type())
                    .with_provider_name("ResourceTags"),
            ),
        };
        let mut tags = IndexMap::new();
        tags.insert(
            "Env".to_string(),
            Value::Concrete(ConcreteValue::String("test".to_string().into())),
        );
        let patch = UpdatePatch {
            ops: vec![replace("tags", Value::Concrete(ConcreteValue::Map(tags)))],
        };

        assert_eq!(
            config.tags.as_ref().map(|tags| tags.provider_name),
            Some("ResourceTags")
        );
        let plan = build_update_patches(&config, "test.Resource", &patch);

        assert_eq!(plan.ops.len(), 1);
        assert_eq!(
            plan.ops[0].get("path").and_then(|value| value.as_str()),
            Some("/ResourceTags")
        );
    }

    #[test]
    fn test_unknown_attribute_is_dropped() {
        let config = get_vpc_config();
        let patch = UpdatePatch {
            ops: vec![replace(
                "not_a_real_attr",
                Value::Concrete(ConcreteValue::String("x".to_string().into())),
            )],
        };
        let patches = build_update_patches(config, "ec2.Vpc", &patch).ops;
        assert!(
            patches.is_empty(),
            "unknown attributes must be dropped, got: {patches:?}"
        );
    }

    #[test]
    fn test_delete_retry_constants() {
        use super::super::{
            CREATE_RETRY_INITIAL_DELAY_SECS, CREATE_RETRY_MAX_ATTEMPTS,
            CREATE_RETRY_MAX_DELAY_SECS, DELETE_RETRY_INITIAL_DELAY_SECS,
            DELETE_RETRY_MAX_ATTEMPTS, DELETE_RETRY_MAX_DELAY_SECS,
        };
        assert_eq!(DELETE_RETRY_MAX_ATTEMPTS, 12);
        assert_eq!(DELETE_RETRY_INITIAL_DELAY_SECS, 10);
        assert_eq!(DELETE_RETRY_MAX_DELAY_SECS, 120);
        assert_eq!(DELETE_RETRY_MAX_ATTEMPTS, CREATE_RETRY_MAX_ATTEMPTS);
        assert_eq!(
            DELETE_RETRY_INITIAL_DELAY_SECS,
            CREATE_RETRY_INITIAL_DELAY_SECS
        );
        assert_eq!(DELETE_RETRY_MAX_DELAY_SECS, CREATE_RETRY_MAX_DELAY_SECS);
    }
}
