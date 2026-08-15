//! Update patch building and resource property parsing.
//!
//! This module maps `carina_core::provider::UpdatePatch` ops directly to
//! CloudControl JSON Patch operations. The patch carries only the
//! attributes the user explicitly specified or removed — fields the user
//! has never specified do not appear in the patch and therefore generate
//! no JSON Patch op, leaving CloudControl-managed defaults and
//! sibling-resource state alone (the actual fix for
//! `carina-rs/carina#2559`).

use carina_core::provider::{PatchOp, PatchOpKind, ProviderError, ProviderResult, UpdatePatch};
use carina_core::resource::{ConcreteValue, Value};
use serde_json::json;

use super::conversion::dsl_value_to_aws_with_defs;
use super::tags::tags_provider_name;
use crate::schemas::config::AwsccSchemaConfig;

/// Parse a JSON string from CloudControl API response into a `serde_json::Value`.
///
/// Returns an error instead of silently returning an empty object when the JSON is malformed.
pub(crate) fn parse_resource_properties(props_str: &str) -> ProviderResult<serde_json::Value> {
    serde_json::from_str(props_str)
        .map_err(|e| ProviderError::internal("Failed to parse resource properties").with_cause(e))
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
/// Each [`PatchOp`] in `patch.ops` corresponds to a key the user
/// explicitly added, replaced, or removed in the desired state.
/// Attributes the user has never specified do not appear in `patch.ops`
/// and therefore generate no JSON Patch op — CloudControl leaves them
/// untouched.
///
/// Mapping rules:
/// - [`PatchOpKind::Add`] / [`PatchOpKind::Replace`] with a value →
///   `{"op": "add", "path": "/<aws_name>", "value": <serialized>}`
///   (`add` is used for both because CloudControl's JSON Patch
///   `replace` fails when the property does not exist in the resource
///   model, while `add` works for both create-and-set and replace-existing
///   cases — matches the Terraform AWSCC provider's behavior).
/// - [`PatchOpKind::Remove`] (or Add/Replace with `value: None`) →
///   `{"op": "remove", "path": "/<aws_name>"}`.
///
/// Read-only and create-only attributes are filtered out: CloudControl rejects
/// patches that touch them. Add/Replace operations with concrete values that are
/// filtered out remain in the plan's confirmation metadata so their desired values
/// cannot later be fabricated into state.
///
/// `tags` is special-cased to project the per-key DSL `Map<String,
/// String>` into CloudControl's `[{"Key": ..., "Value": ...}]` shape.
pub(crate) fn build_update_patches(
    config: &AwsccSchemaConfig,
    resource_type: &str,
    patch: &UpdatePatch,
) -> UpdatePatchPlan {
    let mut plan = UpdatePatchPlan::default();

    for op in &patch.ops {
        let attr_schema = match config.schema.attributes.get(&op.key) {
            Some(s) => s,
            // Unknown keys (not in the schema) are silently dropped:
            // CloudControl will reject anything we don't have an AWS
            // path for, and surfacing this as an error here would mask
            // schema bugs as user errors.
            None => continue,
        };
        plan.touch(&op.key);

        let requires_confirmation = matches!(op.kind, PatchOpKind::Add | PatchOpKind::Replace)
            && op.value.is_some()
            && !attr_schema.write_only;
        if requires_confirmation {
            plan.require_confirmation(&op.key);
        }

        // CloudControl rejects patches that touch read-only or create-only
        // properties. Drop them even if they appear in the patch.
        if attr_schema.read_only || attr_schema.create_only {
            if requires_confirmation {
                plan.mark_unsent(&op.key);
            }
            continue;
        }

        let provider_name = if op.key == "tags" {
            tags_provider_name(config)
        } else {
            attr_schema.provider_name.as_deref()
        };
        let aws_name = match provider_name {
            Some(name) => name,
            None => {
                if requires_confirmation {
                    plan.mark_unsent(&op.key);
                }
                continue;
            }
        };

        if op.key == "tags" {
            if !push_tags_op(&mut plan.ops, op, aws_name) && requires_confirmation {
                plan.mark_unsent(&op.key);
            }
            continue;
        }

        match (op.kind, &op.value) {
            (PatchOpKind::Add | PatchOpKind::Replace, Some(value)) => {
                if let Some(aws_value) = dsl_value_to_aws_with_defs(
                    value,
                    &attr_schema.attr_type,
                    resource_type,
                    &op.key,
                    &config.schema.defs,
                ) {
                    plan.ops.push(json!({
                        "op": "add",
                        "path": format!("/{}", aws_name),
                        "value": aws_value,
                    }));
                } else if requires_confirmation {
                    plan.mark_unsent(&op.key);
                }
            }
            (PatchOpKind::Remove, _) | (PatchOpKind::Add | PatchOpKind::Replace, None) => {
                plan.ops.push(json!({
                    "op": "remove",
                    "path": format!("/{}", aws_name),
                }));
            }
        }
    }

    plan
}

/// Project a `tags` patch op (DSL `Map<String, String>`) into the
/// CloudControl `[{"Key": ..., "Value": ...}]` shape. Projection is
/// all-or-nothing because the resulting array replaces the remote tag set.
fn push_tags_op(patch_ops: &mut Vec<serde_json::Value>, op: &PatchOp, aws_name: &str) -> bool {
    match (op.kind, &op.value) {
        (
            PatchOpKind::Add | PatchOpKind::Replace,
            Some(Value::Concrete(ConcreteValue::Map(user_tags))),
        ) => {
            let Some(tags) = user_tags
                .iter()
                .map(|(key, value)| match value {
                    Value::Concrete(ConcreteValue::String(v)) => {
                        Some(json!({"Key": key, "Value": v}))
                    }
                    _ => None,
                })
                .collect::<Option<Vec<_>>>()
            else {
                return false;
            };
            if tags.is_empty() {
                // An empty map means clear all tags: treat it as remove so we
                // don't push an empty Tags array (CloudControl rejects
                // some resource types' empty Tags).
                patch_ops.push(json!({"op": "remove", "path": format!("/{aws_name}")}));
            } else {
                patch_ops.push(json!({
                    "op": "add",
                    "path": format!("/{aws_name}"),
                    "value": tags,
                }));
            }
            true
        }
        (PatchOpKind::Add | PatchOpKind::Replace, Some(_)) => false,
        (PatchOpKind::Remove, _) | (PatchOpKind::Add | PatchOpKind::Replace, None) => {
            patch_ops.push(json!({"op": "remove", "path": format!("/{aws_name}")}));
            true
        }
    }
}

#[cfg(test)]
mod tests {
    use carina_core::resource::{DeferredValue, UnknownReason};
    use carina_core::schema::{AttributeSchema, AttributeType, ResourceSchema};
    use indexmap::IndexMap;

    use super::*;

    fn get_schema_config(resource_type: &str) -> Option<&'static AwsccSchemaConfig> {
        super::super::get_schema_config(resource_type)
    }

    fn get_vpc_config() -> &'static AwsccSchemaConfig {
        get_schema_config("ec2.Vpc").expect("ec2.vpc schema should exist")
    }

    fn replace(key: &str, value: Value) -> PatchOp {
        PatchOp {
            kind: PatchOpKind::Replace,
            key: key.to_string(),
            value: Some(value),
        }
    }

    fn add(key: &str, value: Value) -> PatchOp {
        PatchOp {
            kind: PatchOpKind::Add,
            key: key.to_string(),
            value: Some(value),
        }
    }

    fn remove(key: &str) -> PatchOp {
        PatchOp {
            kind: PatchOpKind::Remove,
            key: key.to_string(),
            value: None,
        }
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
            err.message()
                .contains("Failed to parse resource properties"),
            "Expected error message about parsing, got: {}",
            err.message()
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
            Value::Concrete(ConcreteValue::String("staging".to_string())),
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
                    "aws.ec2.Vpc.InstanceTenancy.dedicated".to_string(),
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
                Value::Concrete(ConcreteValue::String("arn:aws:logs:::*".to_string())),
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
    fn test_attribute_without_provider_name_is_tracked_as_unsent() {
        let config = AwsccSchemaConfig {
            aws_type_name: "AWS::Test::Resource",
            resource_type_name: "test.Resource",
            primary_identifier: &[],
            has_tags: false,
            schema: ResourceSchema::new("test.Resource").attribute(AttributeSchema::new(
                "unmapped_attribute",
                AttributeType::string(),
            )),
        };
        let patch = UpdatePatch {
            ops: vec![replace(
                "unmapped_attribute",
                Value::Concrete(ConcreteValue::String("requested".to_string())),
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
            Value::Concrete(ConcreteValue::String("kept".to_string())),
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
            Value::Concrete(ConcreteValue::String("my-vpc".to_string())),
        );
        tags.insert(
            "Env".to_string(),
            Value::Concrete(ConcreteValue::String("prod".to_string())),
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
    fn test_tags_patch_uses_schema_provider_name_for_path() {
        let config = AwsccSchemaConfig {
            aws_type_name: "AWS::Test::Resource",
            resource_type_name: "test.Resource",
            primary_identifier: &[],
            has_tags: true,
            schema: ResourceSchema::new("test.Resource").attribute(
                AttributeSchema::new("tags", carina_aws_types::tags_type())
                    .with_provider_name("ResourceTags"),
            ),
        };
        let mut tags = IndexMap::new();
        tags.insert(
            "Env".to_string(),
            Value::Concrete(ConcreteValue::String("test".to_string())),
        );
        let patch = UpdatePatch {
            ops: vec![replace("tags", Value::Concrete(ConcreteValue::Map(tags)))],
        };

        assert_eq!(tags_provider_name(&config), Some("ResourceTags"));
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
                Value::Concrete(ConcreteValue::String("x".to_string())),
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
