//! oidc_provider schema definition for AWS Cloud Control
//!
//! Auto-generated from CloudFormation schema: AWS::IAM::OIDCProvider
//!
//! DO NOT EDIT MANUALLY - regenerate with carina-codegen

use super::AwsccSchemaConfig;
use super::tags_type;
use super::validate_tags_map;
use carina_core::resource::{ConcreteValue, Value};
use carina_core::schema::{AttributeSchema, AttributeType, ResourceSchema, legacy_validator};
use regex::Regex;

pub fn arn() -> AttributeType {
    super::iam_oidc_provider_arn()
}

#[allow(dead_code)]
fn validate_list_items_max_5(value: &Value) -> Result<(), String> {
    if let Value::Concrete(ConcreteValue::List(items)) = value {
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
    if let Value::Concrete(ConcreteValue::String(s)) = value {
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
    if let Value::Concrete(ConcreteValue::String(s)) = value {
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
        resource_type_name: "iam.OidcProvider",
        has_tags: true,
        schema: ResourceSchema::new("iam.OidcProvider")
            .with_description("Resource Type definition for AWS::IAM::OIDCProvider")
            .attribute(
                AttributeSchema::new("arn", self::arn())
                    .read_only()
                    .with_description("Amazon Resource Name (ARN) of the OIDC provider (read-only)")
                    .with_provider_name("Arn"),
            )
            .attribute(
                AttributeSchema::new(
                    "client_id_list",
                    AttributeType::unordered_list(AttributeType::custom(
                        None,
                        AttributeType::string(),
                        None,
                        Some((Some(1), Some(255))),
                        legacy_validator(validate_string_length_1_255),
                        None,
                    )),
                )
                .with_provider_name("ClientIdList"),
            )
            .attribute(
                AttributeSchema::new("tags", tags_type())
                    .with_provider_name("Tags")
                    .with_block_name("tag"),
            )
            .attribute(
                AttributeSchema::new(
                    "thumbprint_list",
                    AttributeType::custom(
                        None,
                        AttributeType::unordered_list(AttributeType::custom(
                            None,
                            AttributeType::string(),
                            Some("[0-9A-Fa-f]{40}".to_string()),
                            Some((Some(40), Some(40))),
                            legacy_validator(validate_string_pattern_57ee0c44b504b839_len_40_40),
                            None,
                        )),
                        None,
                        None,
                        legacy_validator(validate_list_items_max_5),
                        None,
                    ),
                )
                .with_provider_name("ThumbprintList"),
            )
            .attribute(
                AttributeSchema::new(
                    "url",
                    AttributeType::custom(
                        None,
                        AttributeType::string(),
                        None,
                        Some((Some(1), Some(255))),
                        legacy_validator(validate_string_length_1_255),
                        None,
                    ),
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
    ("iam.OidcProvider", &[])
}
