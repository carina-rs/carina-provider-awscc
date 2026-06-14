//! group schema definition for AWS Cloud Control
//!
//! Auto-generated from CloudFormation schema: AWS::IdentityStore::Group
//!
//! DO NOT EDIT MANUALLY - regenerate with carina-codegen

use crate::schemas::config::AwsccSchemaConfig;
use carina_core::resource::{ConcreteValue, Value};
use carina_core::schema::{AttributeSchema, AttributeType, ResourceSchema};
use regex::Regex;

/// Returns the schema config for identitystore_group (AWS::IdentityStore::Group)
pub fn identitystore_group_config() -> AwsccSchemaConfig {
    AwsccSchemaConfig {
        aws_type_name: "AWS::IdentityStore::Group",
        resource_type_name: "identitystore.Group",
        has_tags: false,
        schema: ResourceSchema::new("identitystore.Group")
        .with_description("Resource Type definition for AWS::IdentityStore::Group")
        .attribute(
            AttributeSchema::new("description", AttributeType::refined_string(None, Some("^[\\p{L}\\p{M}\\p{S}\\p{N}\\p{P}\\t\\n\\r  　]+$".to_string()), Some((Some(1), Some(1024))), None))
                .with_description("A string containing the description of the group.")
                .with_provider_name("Description"),
        )
        .attribute(
            AttributeSchema::new("display_name", AttributeType::refined_string(None, Some("^[\\p{L}\\p{M}\\p{S}\\p{N}\\p{P}\\t\\n\\r  ]+$".to_string()), Some((Some(1), Some(1024))), None))
                .required()
                .with_description("A string containing the name of the group. This value is commonly displayed when the group is referenced.")
                .with_provider_name("DisplayName"),
        )
        .attribute(
            AttributeSchema::new("group_id", carina_aws_types::sso_principal_id())
                .read_only()
                .with_description("The unique identifier for a group in the identity store. (read-only)")
                .with_provider_name("GroupId"),
        )
        .attribute(
            AttributeSchema::new("identity_store_id", carina_aws_types::identity_store_id())
                .required()
                .create_only()
                .with_description("The globally unique identifier for the identity store.")
                .with_provider_name("IdentityStoreId"),
        )
    }
}

#[allow(dead_code)]
fn validate_string_pattern_a301e45ae2f7df12_len_1_1024(value: &Value) -> Result<(), String> {
    if let Value::Concrete(ConcreteValue::String(s)) = value {
        static RE: std::sync::LazyLock<Regex> = std::sync::LazyLock::new(|| {
            Regex::new("^[\\p{L}\\p{M}\\p{S}\\p{N}\\p{P}\\t\\n\\r  ]+$")
                .expect("invalid pattern regex")
        });
        if !RE.is_match(s) {
            return Err(format!(
                "Value '{}' does not match pattern ^[\\p{{L}}\\p{{M}}\\p{{S}}\\p{{N}}\\p{{P}}\\t\\n\\r  ]+$",
                s
            ));
        }
        let len = s.chars().count();
        if !(1..=1024).contains(&len) {
            return Err(format!("String length {} is out of range 1..=1024", len));
        }
        Ok(())
    } else {
        Err("Expected string".to_string())
    }
}

#[allow(dead_code)]
fn validate_string_pattern_3e29f1c0497511f3_len_1_1024(value: &Value) -> Result<(), String> {
    if let Value::Concrete(ConcreteValue::String(s)) = value {
        static RE: std::sync::LazyLock<Regex> = std::sync::LazyLock::new(|| {
            Regex::new("^[\\p{L}\\p{M}\\p{S}\\p{N}\\p{P}\\t\\n\\r  　]+$")
                .expect("invalid pattern regex")
        });
        if !RE.is_match(s) {
            return Err(format!(
                "Value '{}' does not match pattern ^[\\p{{L}}\\p{{M}}\\p{{S}}\\p{{N}}\\p{{P}}\\t\\n\\r  　]+$",
                s
            ));
        }
        let len = s.chars().count();
        if !(1..=1024).contains(&len) {
            return Err(format!("String length {} is out of range 1..=1024", len));
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
    ("identitystore.Group", &[])
}

/// Returns the IAM permissions declared by the CloudFormation handler for this operation.
pub fn required_permissions(op: carina_core::effect::PlanOp) -> &'static [&'static str] {
    match op {
        carina_core::effect::PlanOp::Create => {
            &["identitystore:CreateGroup", "identitystore:DescribeGroup"]
        }
        carina_core::effect::PlanOp::Read => &["identitystore:DescribeGroup"],
        carina_core::effect::PlanOp::Update => {
            &["identitystore:DescribeGroup", "identitystore:UpdateGroup"]
        }
        carina_core::effect::PlanOp::Delete => {
            &["identitystore:DescribeGroup", "identitystore:DeleteGroup"]
        }
    }
}
