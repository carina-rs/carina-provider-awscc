//! oidc_provider schema definition for AWS Cloud Control
//!
//! Auto-generated from CloudFormation schema: AWS::IAM::OIDCProvider
//!
//! DO NOT EDIT MANUALLY - regenerate with carina-codegen

use super::AwsccSchemaConfig;
use super::tags_type;
use super::validate_tags_map;
use carina_core::resource::Value;
use carina_core::schema::{AttributeSchema, AttributeType, ResourceSchema};
use regex::Regex;

#[allow(dead_code)]
fn validate_list_items_max_5(value: &Value) -> Result<(), String> {
    if let Value::List(items) = value {
        let len = items.len();
        if len > 5 {
            Err(format!("List has {} items, expected ..=5", len))
        } else {
            Ok(())
        }
    } else {
        Err("Expected list".to_string())
    }
}

fn validate_string_length_1_255(value: &Value) -> Result<(), String> {
    if let Value::String(s) = value {
        let len = s.chars().count();
        if !(1..=255).contains(&len) {
            Err(format!("String length {} is out of range 1..=255", len))
        } else {
            Ok(())
        }
    } else {
        Ok(())
    }
}

#[allow(dead_code)]
fn validate_string_pattern_57ee0c44b504b839_len_40_40(value: &Value) -> Result<(), String> {
    if let Value::String(s) = value {
        static RE: std::sync::LazyLock<Regex> = std::sync::LazyLock::new(|| {
            Regex::new("[0-9A-Fa-f]{40}").expect("invalid pattern regex")
        });
        if !RE.is_match(s) {
            return Err(format!(
                "Value '{}' does not match pattern [0-9A-Fa-f]{{40}}",
                s
            ));
        }
        let len = s.chars().count();
        if !(40..=40).contains(&len) {
            return Err(format!("String length {} is out of range 40..=40", len));
        }
        Ok(())
    } else {
        Err("Expected string".to_string())
    }
}

/// Returns the schema config for iam_oidc_provider (AWS::IAM::OIDCProvider)
pub fn iam_oidc_provider_config() -> AwsccSchemaConfig {
    AwsccSchemaConfig {
        aws_type_name: "AWS::IAM::OIDCProvider",
        resource_type_name: "iam.oidc_provider",
        has_tags: true,
        schema: ResourceSchema::new("awscc.iam.oidc_provider")
            .with_description("Resource Type definition for AWS::IAM::OIDCProvider")
            .attribute(
                AttributeSchema::new("arn", super::arn())
                    .read_only()
                    .with_description("Amazon Resource Name (ARN) of the OIDC provider (read-only)")
                    .with_provider_name("Arn"),
            )
            .attribute(
                AttributeSchema::new(
                    "client_id_list",
                    AttributeType::unordered_list(AttributeType::Custom {
                        name: "String(len: 1..=255)".to_string(),
                        base: Box::new(AttributeType::String),
                        validate: validate_string_length_1_255,
                        namespace: None,
                        to_dsl: None,
                    }),
                )
                .with_provider_name("ClientIdList"),
            )
            .attribute(AttributeSchema::new("tags", tags_type()).with_provider_name("Tags"))
            .attribute(
                AttributeSchema::new(
                    "thumbprint_list",
                    AttributeType::Custom {
                        name: "List(..=5)".to_string(),
                        base: Box::new(AttributeType::unordered_list(AttributeType::Custom {
                            name: "String(pattern, len: 40..=40)".to_string(),
                            base: Box::new(AttributeType::String),
                            validate: validate_string_pattern_57ee0c44b504b839_len_40_40,
                            namespace: None,
                            to_dsl: None,
                        })),
                        validate: validate_list_items_max_5,
                        namespace: None,
                        to_dsl: None,
                    },
                )
                .with_provider_name("ThumbprintList"),
            )
            .attribute(
                AttributeSchema::new(
                    "url",
                    AttributeType::Custom {
                        name: "String(len: 1..=255)".to_string(),
                        base: Box::new(AttributeType::String),
                        validate: validate_string_length_1_255,
                        namespace: None,
                        to_dsl: None,
                    },
                )
                .create_only()
                .with_provider_name("Url"),
            )
            .with_validator(|attrs| {
                let mut errors = Vec::new();
                if let Err(mut e) = validate_tags_map(attrs) {
                    errors.append(&mut e);
                }
                if errors.is_empty() {
                    Ok(())
                } else {
                    Err(errors)
                }
            }),
    }
}

/// Returns the resource type name and all enum valid values for this module
pub fn enum_valid_values() -> (
    &'static str,
    &'static [(&'static str, &'static [&'static str])],
) {
    ("iam.oidc_provider", &[])
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
