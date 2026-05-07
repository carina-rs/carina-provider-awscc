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
use carina_core::resource::Value;
use serde_json::json;

use super::conversion::dsl_value_to_aws;
use crate::schemas::generated::AwsccSchemaConfig;

/// Parse a JSON string from CloudControl API response into a `serde_json::Value`.
///
/// Returns an error instead of silently returning an empty object when the JSON is malformed.
pub(crate) fn parse_resource_properties(props_str: &str) -> ProviderResult<serde_json::Value> {
    serde_json::from_str(props_str)
        .map_err(|e| ProviderError::internal("Failed to parse resource properties").with_cause(e))
}

/// Build CloudControl JSON Patch operations from an [`UpdatePatch`].
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
/// Read-only and create-only attributes are filtered out: CloudControl
/// rejects patches that touch them, so even if the user changed the
/// value in their `.crn` (typically because the schema was wrong) the
/// op is dropped before the API call.
///
/// `tags` is special-cased to project the per-key DSL `Map<String,
/// String>` into CloudControl's `[{"Key": ..., "Value": ...}]` shape.
pub(crate) fn build_update_patches(
    config: &AwsccSchemaConfig,
    resource_type: &str,
    patch: &UpdatePatch,
) -> Vec<serde_json::Value> {
    let mut patch_ops = Vec::new();

    for op in &patch.ops {
        // tags handled below
        if op.key == "tags" {
            push_tags_op(&mut patch_ops, op);
            continue;
        }

        let attr_schema = match config.schema.attributes.get(&op.key) {
            Some(s) => s,
            // Unknown keys (not in the schema) are silently dropped:
            // CloudControl will reject anything we don't have an AWS
            // path for, and surfacing this as an error here would mask
            // schema bugs as user errors.
            None => continue,
        };

        // CloudControl rejects patches that touch read-only or create-only
        // properties. Drop them even if they appear in the patch.
        if attr_schema.read_only || attr_schema.create_only {
            continue;
        }

        let aws_name = match &attr_schema.provider_name {
            Some(name) => name,
            None => continue,
        };

        match (op.kind, &op.value) {
            (PatchOpKind::Add | PatchOpKind::Replace, Some(value)) => {
                if let Some(aws_value) =
                    dsl_value_to_aws(value, &attr_schema.attr_type, resource_type, &op.key)
                {
                    patch_ops.push(json!({
                        "op": "add",
                        "path": format!("/{}", aws_name),
                        "value": aws_value,
                    }));
                }
            }
            (PatchOpKind::Remove, _) | (PatchOpKind::Add | PatchOpKind::Replace, None) => {
                patch_ops.push(json!({
                    "op": "remove",
                    "path": format!("/{}", aws_name),
                }));
            }
        }
    }

    patch_ops
}

/// Project a `tags` patch op (DSL `Map<String, String>`) into the
/// CloudControl `[{"Key": ..., "Value": ...}]` shape.
fn push_tags_op(patch_ops: &mut Vec<serde_json::Value>, op: &PatchOp) {
    match (op.kind, &op.value) {
        (PatchOpKind::Add | PatchOpKind::Replace, Some(Value::Map(user_tags))) => {
            let mut tags = Vec::new();
            for (key, value) in user_tags {
                if let Value::String(v) = value {
                    tags.push(json!({"Key": key, "Value": v}));
                }
            }
            if tags.is_empty() {
                // Empty map after projection: treat as remove so we
                // don't push an empty Tags array (CloudControl rejects
                // some resource types' empty Tags).
                patch_ops.push(json!({"op": "remove", "path": "/Tags"}));
            } else {
                patch_ops.push(json!({"op": "add", "path": "/Tags", "value": tags}));
            }
        }
        (PatchOpKind::Remove, _) | (PatchOpKind::Add | PatchOpKind::Replace, _) => {
            patch_ops.push(json!({"op": "remove", "path": "/Tags"}));
        }
    }
}

#[cfg(test)]
mod tests {
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
    /// changes only `tags` on `awscc.iam.Role` must produce exactly one
    /// JSON Patch op (`add /Tags`). It must NOT also emit a
    /// `remove /Policies` (or any other `remove`) for fields the user
    /// never specified, because that clobbers sibling
    /// `awscc.iam.RolePolicy` resources.
    #[test]
    fn issue_2559_tags_only_patch_does_not_touch_other_fields() {
        let config = get_schema_config("iam.Role").expect("iam.role schema should exist");

        let mut tags = IndexMap::new();
        tags.insert("Env".to_string(), Value::String("staging".to_string()));
        let patch = UpdatePatch {
            ops: vec![replace("tags", Value::Map(tags))],
        };

        let patches = build_update_patches(config, "iam.Role", &patch);

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
        let patches = build_update_patches(config, "ec2.Vpc", &patch);
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
                Value::String("awscc.ec2.Vpc.InstanceTenancy.dedicated".to_string()),
            )],
        };
        let patches = build_update_patches(config, "ec2.Vpc", &patch);

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
            ops: vec![add("retention_in_days", Value::Int(14))],
        };
        let patches = build_update_patches(config, "logs.LogGroup", &patch);

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
        let patches = build_update_patches(config, "ec2.Vpc", &patch);

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
                Value::String("arn:aws:logs:::*".to_string()),
            )],
        };
        let patches = build_update_patches(config, "logs.LogGroup", &patch);
        assert!(
            patches.is_empty(),
            "read-only attribute must be dropped, got: {patches:?}"
        );
    }

    #[test]
    fn test_remove_tags_emits_tags_remove() {
        let config = get_vpc_config();
        let patch = UpdatePatch {
            ops: vec![remove("tags")],
        };
        let patches = build_update_patches(config, "ec2.Vpc", &patch);

        assert_eq!(patches.len(), 1);
        let op = &patches[0];
        assert_eq!(op.get("op").and_then(|v| v.as_str()), Some("remove"));
        assert_eq!(op.get("path").and_then(|v| v.as_str()), Some("/Tags"));
    }

    #[test]
    fn test_replace_tags_projects_to_aws_shape() {
        let config = get_vpc_config();
        let mut tags = IndexMap::new();
        tags.insert("Name".to_string(), Value::String("my-vpc".to_string()));
        tags.insert("Env".to_string(), Value::String("prod".to_string()));
        let patch = UpdatePatch {
            ops: vec![replace("tags", Value::Map(tags))],
        };
        let patches = build_update_patches(config, "ec2.Vpc", &patch);

        assert_eq!(patches.len(), 1);
        let op = &patches[0];
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
    fn test_unknown_attribute_is_dropped() {
        let config = get_vpc_config();
        let patch = UpdatePatch {
            ops: vec![replace("not_a_real_attr", Value::String("x".to_string()))],
        };
        let patches = build_update_patches(config, "ec2.Vpc", &patch);
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
