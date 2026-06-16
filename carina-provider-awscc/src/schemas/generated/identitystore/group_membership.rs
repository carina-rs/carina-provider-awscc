//! group_membership schema definition for AWS Cloud Control
//!
//! Auto-generated from CloudFormation schema: AWS::IdentityStore::GroupMembership
//!
//! DO NOT EDIT MANUALLY - regenerate with carina-codegen

use crate::schemas::config::AwsccSchemaConfig;
use carina_core::resource::{ConcreteValue, Value};
use carina_core::schema::{AttributeSchema, AttributeType, ResourceSchema, StructField};
use regex::Regex;

/// Returns the schema config for identitystore_group_membership (AWS::IdentityStore::GroupMembership)
pub fn identitystore_group_membership_config() -> AwsccSchemaConfig {
    AwsccSchemaConfig {
        aws_type_name: "AWS::IdentityStore::GroupMembership",
        resource_type_name: "identitystore.GroupMembership",
        primary_identifier: &["MembershipId", "IdentityStoreId"],
        has_tags: false,
        schema: ResourceSchema::new("identitystore.GroupMembership")
	        .with_description("Resource Type Definition for AWS:IdentityStore::GroupMembership")
        .attribute(
            AttributeSchema::new("group_id", carina_aws_types::sso_principal_id())
                .required()
                .create_only()
                .with_description("The unique identifier for a group in the identity store.")
                .with_provider_name("GroupId"),
        )
        .attribute(
            AttributeSchema::new("identity_store_id", carina_aws_types::identity_store_id())
                .required()
                .create_only()
                .with_description("The globally unique identifier for the identity store.")
                .with_provider_name("IdentityStoreId"),
        )
        .attribute(
            AttributeSchema::new("member_id", AttributeType::struct_("MemberId".to_string(), vec![StructField::new("user_id", AttributeType::refined_string(None, Some("^([0-9a-f]{10}-|)[A-Fa-f0-9]{8}-[A-Fa-f0-9]{4}-[A-Fa-f0-9]{4}-[A-Fa-f0-9]{4}-[A-Fa-f0-9]{12}$".to_string()), Some((Some(1), Some(47))), None)).required().with_description("The identifier for a user in the identity store.").with_provider_name("UserId")]))
                .required()
                .create_only()
                .with_description("An object containing the identifier of a group member.")
                .with_provider_name("MemberId"),
        )
        .attribute(
            AttributeSchema::new("membership_id", AttributeType::refined_string(None, Some("^([0-9a-f]{10}-|)[A-Fa-f0-9]{8}-[A-Fa-f0-9]{4}-[A-Fa-f0-9]{4}-[A-Fa-f0-9]{4}-[A-Fa-f0-9]{12}$".to_string()), Some((Some(1), Some(47))), None))
                .read_only()
                .with_description("The identifier for a GroupMembership in the identity store. (read-only)")
                .with_provider_name("MembershipId"),
        )
        .with_def("MemberId", AttributeType::struct_("MemberId".to_string(), vec![StructField::new("user_id", AttributeType::refined_string(None, Some("^([0-9a-f]{10}-|)[A-Fa-f0-9]{8}-[A-Fa-f0-9]{4}-[A-Fa-f0-9]{4}-[A-Fa-f0-9]{4}-[A-Fa-f0-9]{12}$".to_string()), Some((Some(1), Some(47))), None)).required().with_description("The identifier for a user in the identity store.").with_provider_name("UserId")]))
    }
}

#[allow(dead_code)]
fn validate_string_pattern_2a77a2e32f71b5f3_len_1_47(value: &Value) -> Result<(), String> {
    if let Value::Concrete(ConcreteValue::String(s)) = value {
        static RE: std::sync::LazyLock<Regex> = std::sync::LazyLock::new(|| {
            Regex::new("^([0-9a-f]{10}-|)[A-Fa-f0-9]{8}-[A-Fa-f0-9]{4}-[A-Fa-f0-9]{4}-[A-Fa-f0-9]{4}-[A-Fa-f0-9]{12}$").expect("invalid pattern regex")
        });
        if !RE.is_match(s) {
            return Err(format!(
                "Value '{}' does not match pattern ^([0-9a-f]{{10}}-|)[A-Fa-f0-9]{{8}}-[A-Fa-f0-9]{{4}}-[A-Fa-f0-9]{{4}}-[A-Fa-f0-9]{{4}}-[A-Fa-f0-9]{{12}}$",
                s
            ));
        }
        let len = s.chars().count();
        if !(1..=47).contains(&len) {
            return Err(format!("String length {} is out of range 1..=47", len));
        }
        Ok(())
    } else {
        Err("Expected string".to_string())
    }
}

/// Returns the resource type name and all enum valid values for this module
pub fn enum_valid_values() -> (
    &'static str,
    &'static [(&'static str, &'static [&'static str])],
) {
    ("identitystore.GroupMembership", &[])
}

/// Returns the IAM permissions declared by the CloudFormation handler for this operation.
pub fn required_permissions(op: carina_core::effect::PlanOp) -> &'static [&'static str] {
    match op {
        carina_core::effect::PlanOp::Create => &[
            "identitystore:CreateGroupMembership",
            "identitystore:DescribeGroupMembership",
        ],
        carina_core::effect::PlanOp::Read => &["identitystore:DescribeGroupMembership"],
        carina_core::effect::PlanOp::Update => &[],
        carina_core::effect::PlanOp::Delete => &[
            "identitystore:DeleteGroupMembership",
            "identitystore:DescribeGroupMembership",
        ],
    }
}
