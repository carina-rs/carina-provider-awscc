//! group_membership schema definition for AWS Cloud Control
//!
//! Auto-generated from CloudFormation schema: AWS::IdentityStore::GroupMembership
//!
//! DO NOT EDIT MANUALLY - regenerate with carina-codegen

use super::AwsccSchemaConfig;
use carina_core::resource::Value;
use carina_core::schema::{AttributeSchema, AttributeType, ResourceSchema, StructField};
use regex::Regex;

#[allow(dead_code)]
fn validate_string_pattern_2a77a2e32f71b5f3_len_1_47(value: &Value) -> Result<(), String> {
    if let Value::String(s) = value {
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

#[allow(dead_code)]
fn validate_string_pattern_135f0b126ef95449_len_1_36(value: &Value) -> Result<(), String> {
    if let Value::String(s) = value {
        static RE: std::sync::LazyLock<Regex> = std::sync::LazyLock::new(|| {
            Regex::new(
                "^d-[0-9a-f]{10}$|^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$",
            )
            .expect("invalid pattern regex")
        });
        if !RE.is_match(s) {
            return Err(format!(
                "Value '{}' does not match pattern ^d-[0-9a-f]{{10}}$|^[0-9a-f]{{8}}-[0-9a-f]{{4}}-[0-9a-f]{{4}}-[0-9a-f]{{4}}-[0-9a-f]{{12}}$",
                s
            ));
        }
        let len = s.chars().count();
        if !(1..=36).contains(&len) {
            return Err(format!("String length {} is out of range 1..=36", len));
        }
        Ok(())
    } else {
        Err("Expected string".to_string())
    }
}

/// Returns the schema config for identitystore_group_membership (AWS::IdentityStore::GroupMembership)
pub fn identitystore_group_membership_config() -> AwsccSchemaConfig {
    AwsccSchemaConfig {
        aws_type_name: "AWS::IdentityStore::GroupMembership",
        resource_type_name: "identitystore.group_membership",
        has_tags: false,
        schema: ResourceSchema::new("awscc.identitystore.group_membership")
        .with_description("Resource Type Definition for AWS:IdentityStore::GroupMembership")
        .attribute(
            AttributeSchema::new("group_id", AttributeType::Custom {
                semantic_name: None,
                pattern: Some("^([0-9a-f]{10}-|)[A-Fa-f0-9]{8}-[A-Fa-f0-9]{4}-[A-Fa-f0-9]{4}-[A-Fa-f0-9]{4}-[A-Fa-f0-9]{12}$".to_string()),
                length: Some((Some(1), Some(47))),
                base: Box::new(AttributeType::String),
                validate: validate_string_pattern_2a77a2e32f71b5f3_len_1_47,
                namespace: None,
                to_dsl: None,
            })
                .required()
                .create_only()
                .with_description("The unique identifier for a group in the identity store.")
                .with_provider_name("GroupId"),
        )
        .attribute(
            AttributeSchema::new("identity_store_id", AttributeType::Custom {
                semantic_name: None,
                pattern: Some("^d-[0-9a-f]{10}$|^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$".to_string()),
                length: Some((Some(1), Some(36))),
                base: Box::new(AttributeType::String),
                validate: validate_string_pattern_135f0b126ef95449_len_1_36,
                namespace: None,
                to_dsl: None,
            })
                .required()
                .create_only()
                .with_description("The globally unique identifier for the identity store.")
                .with_provider_name("IdentityStoreId"),
        )
        .attribute(
            AttributeSchema::new("member_id", AttributeType::Struct {
                    name: "MemberId".to_string(),
                    fields: vec![
                    StructField::new("user_id", AttributeType::Custom {
                semantic_name: None,
                pattern: Some("^([0-9a-f]{10}-|)[A-Fa-f0-9]{8}-[A-Fa-f0-9]{4}-[A-Fa-f0-9]{4}-[A-Fa-f0-9]{4}-[A-Fa-f0-9]{12}$".to_string()),
                length: Some((Some(1), Some(47))),
                base: Box::new(AttributeType::String),
                validate: validate_string_pattern_2a77a2e32f71b5f3_len_1_47,
                namespace: None,
                to_dsl: None,
            }).required().with_description("The identifier for a user in the identity store.").with_provider_name("UserId")
                    ],
                })
                .required()
                .create_only()
                .with_description("An object containing the identifier of a group member.")
                .with_provider_name("MemberId"),
        )
        .attribute(
            AttributeSchema::new("membership_id", AttributeType::Custom {
                semantic_name: None,
                pattern: Some("^([0-9a-f]{10}-|)[A-Fa-f0-9]{8}-[A-Fa-f0-9]{4}-[A-Fa-f0-9]{4}-[A-Fa-f0-9]{4}-[A-Fa-f0-9]{12}$".to_string()),
                length: Some((Some(1), Some(47))),
                base: Box::new(AttributeType::String),
                validate: validate_string_pattern_2a77a2e32f71b5f3_len_1_47,
                namespace: None,
                to_dsl: None,
            })
                .read_only()
                .with_description("The identifier for a GroupMembership in the identity store. (read-only)")
                .with_provider_name("MembershipId"),
        )
    }
}

/// Returns the resource type name and all enum valid values for this module
pub fn enum_valid_values() -> (
    &'static str,
    &'static [(&'static str, &'static [&'static str])],
) {
    ("identitystore.group_membership", &[])
}

/// Maps DSL alias values back to canonical AWS values for this module.
/// e.g., ("ip_protocol", "all") -> Some("-1")
pub fn enum_alias_reverse(attr_name: &str, value: &str) -> Option<&'static str> {
    let _ = (attr_name, value);
    None
}

/// Returns all enum alias entries as (attr_name, alias, canonical) tuples.
pub fn enum_alias_entries() -> &'static [(&'static str, &'static str, &'static str)] {
    &[]
}
