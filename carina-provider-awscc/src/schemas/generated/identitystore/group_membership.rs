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
#[rustfmt::skip]
pub fn identitystore_group_membership_config() -> AwsccSchemaConfig {
    let mut schema = ResourceSchema::new("identitystore.GroupMembership")
        .with_description("Resource Type Definition for AWS:IdentityStore::GroupMembership");
    #[inline(never)]
    fn build_group_id_attribute() -> AttributeSchema {
        AttributeSchema::required("group_id", carina_aws_types::sso_principal_id())
                .create_only()
                .with_description("The unique identifier for a group in the identity store.")
                .with_provider_name("GroupId")
            .build()
    }
    schema = schema.attribute(build_group_id_attribute());
    #[inline(never)]
    fn build_identity_store_id_attribute() -> AttributeSchema {
        AttributeSchema::required("identity_store_id", carina_aws_types::identity_store_id())
                .create_only()
                .with_description("The globally unique identifier for the identity store.")
                .with_provider_name("IdentityStoreId")
            .build()
    }
    schema = schema.attribute(build_identity_store_id_attribute());
    #[inline(never)]
    fn build_member_id_attribute() -> AttributeSchema {
        AttributeSchema::required("member_id", AttributeType::struct_("MemberId".to_string(), vec![StructField::required("user_id", AttributeType::refined_string(None, Some("^([0-9a-f]{10}-|)[A-Fa-f0-9]{8}-[A-Fa-f0-9]{4}-[A-Fa-f0-9]{4}-[A-Fa-f0-9]{4}-[A-Fa-f0-9]{12}$".to_string()), Some((Some(1), Some(47))), None)).with_description("The identifier for a user in the identity store.").with_provider_name("UserId").build()]))
                .create_only()
                .with_description("An object containing the identifier of a group member.")
                .with_provider_name("MemberId")
            .build()
    }
    schema = schema.attribute(build_member_id_attribute());
    #[inline(never)]
    fn build_membership_id_attribute() -> AttributeSchema {
        AttributeSchema::provider_populated("membership_id", AttributeType::refined_string(None, Some("^([0-9a-f]{10}-|)[A-Fa-f0-9]{8}-[A-Fa-f0-9]{4}-[A-Fa-f0-9]{4}-[A-Fa-f0-9]{4}-[A-Fa-f0-9]{12}$".to_string()), Some((Some(1), Some(47))), None))
                .with_description("The identifier for a GroupMembership in the identity store. (read-only)")
                .with_provider_name("MembershipId")
            .build()
    }
    schema = schema.attribute(build_membership_id_attribute());
    schema = schema
        .with_coexisting_replacement();
    #[inline(never)]
    fn build_schema_def_0() -> AttributeType { AttributeType::struct_("MemberId".to_string(), vec![StructField::required("user_id", AttributeType::refined_string(None, Some("^([0-9a-f]{10}-|)[A-Fa-f0-9]{8}-[A-Fa-f0-9]{4}-[A-Fa-f0-9]{4}-[A-Fa-f0-9]{4}-[A-Fa-f0-9]{12}$".to_string()), Some((Some(1), Some(47))), None)).with_description("The identifier for a user in the identity store.").with_provider_name("UserId").build()]) }
    schema = schema.with_def("MemberId", build_schema_def_0());
    AwsccSchemaConfig {
        aws_type_name: "AWS::IdentityStore::GroupMembership",
        resource_type_name: "identitystore.GroupMembership",
        primary_identifier: &[crate::schemas::config::PrimaryIdentifierAttribute { provider_name: "MembershipId", dsl_name: "membership_id" }, crate::schemas::config::PrimaryIdentifierAttribute { provider_name: "IdentityStoreId", dsl_name: "identity_store_id" }],
        tags: None,
        schema,
    }
}

#[allow(dead_code)]
fn validate_string_pattern_2a77a2e32f71b5f3_len_1_47(value: &Value) -> Result<(), String> {
    if let Value::Concrete(ConcreteValue::String(s)) = value {
        let s = s.expose_plaintext();
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
