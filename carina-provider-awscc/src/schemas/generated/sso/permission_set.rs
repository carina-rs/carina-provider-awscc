//! permission_set schema definition for AWS Cloud Control
//!
//! Auto-generated from CloudFormation schema: AWS::SSO::PermissionSet
//!
//! DO NOT EDIT MANUALLY - regenerate with carina-codegen

use super::AwsccSchemaConfig;
use super::tags_type;
use super::validate_tags_map;
use carina_core::resource::Value;
use carina_core::schema::{AttributeSchema, AttributeType, ResourceSchema, StructField};
use regex::Regex;

#[allow(dead_code)]
fn validate_list_items_max_20(value: &Value) -> Result<(), String> {
    if let Value::List(items) = value {
        let len = items.len();
        if len > 20 {
            Err(format!("List has {} items, expected ..=20", len))
        } else {
            Ok(())
        }
    } else {
        Err("Expected list".to_string())
    }
}

#[allow(dead_code)]
fn validate_list_items_max_50(value: &Value) -> Result<(), String> {
    if let Value::List(items) = value {
        let len = items.len();
        if len > 50 {
            Err(format!("List has {} items, expected ..=50", len))
        } else {
            Ok(())
        }
    } else {
        Err("Expected list".to_string())
    }
}

#[allow(dead_code)]
fn validate_string_pattern_b84fa12576539ca9_len_1_512(value: &Value) -> Result<(), String> {
    if let Value::String(s) = value {
        static RE: std::sync::LazyLock<Regex> = std::sync::LazyLock::new(|| {
            Regex::new("((/[A-Za-z0-9\\.,\\+@=_-]+)*)/").expect("invalid pattern regex")
        });
        if !RE.is_match(s) {
            return Err(format!(
                "Value '{}' does not match pattern ((/[A-Za-z0-9\\.,\\+@=_-]+)*)/",
                s
            ));
        }
        let len = s.chars().count();
        if !(1..=512).contains(&len) {
            return Err(format!("String length {} is out of range 1..=512", len));
        }
        Ok(())
    } else {
        Err("Expected string".to_string())
    }
}

#[allow(dead_code)]
fn validate_string_pattern_9863be410e005e12_len_1_700(value: &Value) -> Result<(), String> {
    if let Value::String(s) = value {
        static RE: std::sync::LazyLock<Regex> = std::sync::LazyLock::new(|| {
            Regex::new("[\\u0009\\u000A\\u000D\\u0020-\\u007E\\u00A1-\\u00FF]*")
                .expect("invalid pattern regex")
        });
        if !RE.is_match(s) {
            return Err(format!(
                "Value '{}' does not match pattern [\\u0009\\u000A\\u000D\\u0020-\\u007E\\u00A1-\\u00FF]*",
                s
            ));
        }
        let len = s.chars().count();
        if !(1..=700).contains(&len) {
            return Err(format!("String length {} is out of range 1..=700", len));
        }
        Ok(())
    } else {
        Err("Expected string".to_string())
    }
}

#[allow(dead_code)]
fn validate_string_pattern_9b83f4f8f3673df5_len_max_256(value: &Value) -> Result<(), String> {
    if let Value::String(s) = value {
        static RE: std::sync::LazyLock<Regex> =
            std::sync::LazyLock::new(|| Regex::new("[\\w+=,.@-]+").expect("invalid pattern regex"));
        if !RE.is_match(s) {
            return Err(format!("Value '{}' does not match pattern [\\w+=,.@-]+", s));
        }
        let len = s.chars().count();
        if len > 256 {
            return Err(format!("String length {} is out of range ..=256", len));
        }
        Ok(())
    } else {
        Err("Expected string".to_string())
    }
}

#[allow(dead_code)]
fn validate_string_pattern_9b83f4f8f3673df5_len_1_32(value: &Value) -> Result<(), String> {
    if let Value::String(s) = value {
        static RE: std::sync::LazyLock<Regex> =
            std::sync::LazyLock::new(|| Regex::new("[\\w+=,.@-]+").expect("invalid pattern regex"));
        if !RE.is_match(s) {
            return Err(format!("Value '{}' does not match pattern [\\w+=,.@-]+", s));
        }
        let len = s.chars().count();
        if !(1..=32).contains(&len) {
            return Err(format!("String length {} is out of range 1..=32", len));
        }
        Ok(())
    } else {
        Err("Expected string".to_string())
    }
}

#[allow(dead_code)]
fn validate_string_pattern_9b83f4f8f3673df5_len_1_128(value: &Value) -> Result<(), String> {
    if let Value::String(s) = value {
        static RE: std::sync::LazyLock<Regex> =
            std::sync::LazyLock::new(|| Regex::new("[\\w+=,.@-]+").expect("invalid pattern regex"));
        if !RE.is_match(s) {
            return Err(format!("Value '{}' does not match pattern [\\w+=,.@-]+", s));
        }
        let len = s.chars().count();
        if !(1..=128).contains(&len) {
            return Err(format!("String length {} is out of range 1..=128", len));
        }
        Ok(())
    } else {
        Err("Expected string".to_string())
    }
}

#[allow(dead_code)]
fn validate_string_pattern_4d6d630589930649_len_1_240(value: &Value) -> Result<(), String> {
    if let Value::String(s) = value {
        static RE: std::sync::LazyLock<Regex> = std::sync::LazyLock::new(|| {
            Regex::new("[a-zA-Z0-9&amp;$@#\\/%?=~\\-_'&quot;|!:,.;*+\\[\\]\\ \\(\\)\\{\\}]+")
                .expect("invalid pattern regex")
        });
        if !RE.is_match(s) {
            return Err(format!(
                "Value '{}' does not match pattern [a-zA-Z0-9&amp;$@#\\/%?=~\\-_'&quot;|!:,.;*+\\[\\]\\ \\(\\)\\{{\\}}]+",
                s
            ));
        }
        let len = s.chars().count();
        if !(1..=240).contains(&len) {
            return Err(format!("String length {} is out of range 1..=240", len));
        }
        Ok(())
    } else {
        Err("Expected string".to_string())
    }
}

#[allow(dead_code)]
fn validate_string_pattern_1e58d8243b46a2f1_len_1_100(value: &Value) -> Result<(), String> {
    if let Value::String(s) = value {
        static RE: std::sync::LazyLock<Regex> =
            std::sync::LazyLock::new(|| Regex::new(".*").expect("invalid pattern regex"));
        if !RE.is_match(s) {
            return Err(format!("Value '{}' does not match pattern .*", s));
        }
        let len = s.chars().count();
        if !(1..=100).contains(&len) {
            return Err(format!("String length {} is out of range 1..=100", len));
        }
        Ok(())
    } else {
        Err("Expected string".to_string())
    }
}

/// Returns the schema config for sso_permission_set (AWS::SSO::PermissionSet)
pub fn sso_permission_set_config() -> AwsccSchemaConfig {
    AwsccSchemaConfig {
        aws_type_name: "AWS::SSO::PermissionSet",
        resource_type_name: "sso.permission_set",
        has_tags: true,
        schema: ResourceSchema::new("awscc.sso.permission_set")
        .with_description("Resource Type definition for SSO PermissionSet")
        .attribute(
            AttributeSchema::new("customer_managed_policy_references", AttributeType::Custom {
                name: "List(..=20)".to_string(),
                base: Box::new(AttributeType::unordered_list(AttributeType::Struct {
                    name: "CustomerManagedPolicyReference".to_string(),
                    fields: vec![
                    StructField::new("name", AttributeType::Custom {
                name: "String(pattern, len: 1..=128)".to_string(),
                base: Box::new(AttributeType::String),
                validate: validate_string_pattern_9b83f4f8f3673df5_len_1_128,
                namespace: None,
                to_dsl: None,
            }).required().with_provider_name("Name"),
                    StructField::new("path", AttributeType::Custom {
                name: "String(pattern, len: 1..=512)".to_string(),
                base: Box::new(AttributeType::String),
                validate: validate_string_pattern_b84fa12576539ca9_len_1_512,
                namespace: None,
                to_dsl: None,
            }).with_provider_name("Path")
                    ],
                })),
                validate: validate_list_items_max_20,
                namespace: None,
                to_dsl: None,
            })
                .with_provider_name("CustomerManagedPolicyReferences")
                .with_block_name("customer_managed_policy_reference"),
        )
        .attribute(
            AttributeSchema::new("description", AttributeType::Custom {
                name: "String(pattern, len: 1..=700)".to_string(),
                base: Box::new(AttributeType::String),
                validate: validate_string_pattern_9863be410e005e12_len_1_700,
                namespace: None,
                to_dsl: None,
            })
                .with_description("The permission set description.")
                .with_provider_name("Description"),
        )
        .attribute(
            AttributeSchema::new("inline_policy", AttributeType::Map(Box::new(AttributeType::String)))
                .with_description("The inline policy to put in permission set.")
                .with_provider_name("InlinePolicy"),
        )
        .attribute(
            AttributeSchema::new("instance_arn", super::arn())
                .required()
                .create_only()
                .with_description("The sso instance arn that the permission set is owned.")
                .with_provider_name("InstanceArn"),
        )
        .attribute(
            AttributeSchema::new("managed_policies", AttributeType::Custom {
                name: "List(..=20)".to_string(),
                base: Box::new(AttributeType::unordered_list(AttributeType::String)),
                validate: validate_list_items_max_20,
                namespace: None,
                to_dsl: None,
            })
                .with_provider_name("ManagedPolicies"),
        )
        .attribute(
            AttributeSchema::new("name", AttributeType::Custom {
                name: "String(pattern, len: 1..=32)".to_string(),
                base: Box::new(AttributeType::String),
                validate: validate_string_pattern_9b83f4f8f3673df5_len_1_32,
                namespace: None,
                to_dsl: None,
            })
                .required()
                .create_only()
                .with_description("The name you want to assign to this permission set.")
                .with_provider_name("Name"),
        )
        .attribute(
            AttributeSchema::new("permission_set_arn", super::arn())
                .read_only()
                .with_description("The permission set that the policy will be attached to (read-only)")
                .with_provider_name("PermissionSetArn"),
        )
        .attribute(
            AttributeSchema::new("permissions_boundary", AttributeType::Struct {
                    name: "PermissionsBoundary".to_string(),
                    fields: vec![
                    StructField::new("customer_managed_policy_reference", AttributeType::Struct {
                    name: "CustomerManagedPolicyReference".to_string(),
                    fields: vec![
                    StructField::new("name", AttributeType::Custom {
                name: "String(pattern, len: 1..=128)".to_string(),
                base: Box::new(AttributeType::String),
                validate: validate_string_pattern_9b83f4f8f3673df5_len_1_128,
                namespace: None,
                to_dsl: None,
            }).required().with_provider_name("Name"),
                    StructField::new("path", AttributeType::Custom {
                name: "String(pattern, len: 1..=512)".to_string(),
                base: Box::new(AttributeType::String),
                validate: validate_string_pattern_b84fa12576539ca9_len_1_512,
                namespace: None,
                to_dsl: None,
            }).with_provider_name("Path")
                    ],
                }).with_provider_name("CustomerManagedPolicyReference"),
                    StructField::new("managed_policy_arn", super::arn()).with_provider_name("ManagedPolicyArn")
                    ],
                })
                .with_provider_name("PermissionsBoundary"),
        )
        .attribute(
            AttributeSchema::new("relay_state_type", AttributeType::Custom {
                name: "String(pattern, len: 1..=240)".to_string(),
                base: Box::new(AttributeType::String),
                validate: validate_string_pattern_4d6d630589930649_len_1_240,
                namespace: None,
                to_dsl: None,
            })
                .with_description("The relay state URL that redirect links to any service in the AWS Management Console.")
                .with_provider_name("RelayStateType"),
        )
        .attribute(
            AttributeSchema::new("session_duration", AttributeType::Custom {
                name: "String(pattern, len: 1..=100)".to_string(),
                base: Box::new(AttributeType::String),
                validate: validate_string_pattern_1e58d8243b46a2f1_len_1_100,
                namespace: None,
                to_dsl: None,
            })
                .with_description("The length of time that a user can be signed in to an AWS account.")
                .with_provider_name("SessionDuration"),
        )
        .attribute(
            AttributeSchema::new("tags", tags_type())
                .with_provider_name("Tags"),
        )
        .with_validator(|attrs| {
            let mut errors = Vec::new();
            if let Err(mut e) = validate_tags_map(attrs) {
                errors.append(&mut e);
            }
            if errors.is_empty() { Ok(()) } else { Err(errors) }
        })
    }
}

/// Returns the resource type name and all enum valid values for this module
pub fn enum_valid_values() -> (
    &'static str,
    &'static [(&'static str, &'static [&'static str])],
) {
    ("sso.permission_set", &[])
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
