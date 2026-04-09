//! organization schema definition for AWS Cloud Control
//!
//! Auto-generated from CloudFormation schema: AWS::Organizations::Organization
//!
//! DO NOT EDIT MANUALLY - regenerate with carina-codegen

use super::AwsccSchemaConfig;
use carina_core::resource::Value;
use carina_core::schema::{AttributeSchema, AttributeType, ResourceSchema};
use regex::Regex;

const VALID_FEATURE_SET: &[&str] = &["ALL", "CONSOLIDATED_BILLING"];

#[allow(dead_code)]
fn validate_string_pattern_2fd01fd52b67fc75(value: &Value) -> Result<(), String> {
    if let Value::String(s) = value {
        static RE: std::sync::LazyLock<Regex> = std::sync::LazyLock::new(|| {
            Regex::new("^o-[a-z0-9]{10,32}$").expect("invalid pattern regex")
        });
        if RE.is_match(s) {
            Ok(())
        } else {
            Err(format!(
                "Value '{}' does not match pattern ^o-[a-z0-9]{{10,32}}$",
                s
            ))
        }
    } else {
        Err("Expected string".to_string())
    }
}

#[allow(dead_code)]
fn validate_string_pattern_ec4d9bee0dcd262b_len_6_64(value: &Value) -> Result<(), String> {
    if let Value::String(s) = value {
        static RE: std::sync::LazyLock<Regex> = std::sync::LazyLock::new(|| {
            Regex::new("[^\\s@]+@[^\\s@]+\\.[^\\s@]+").expect("invalid pattern regex")
        });
        if !RE.is_match(s) {
            return Err(format!(
                "Value '{}' does not match pattern [^\\s@]+@[^\\s@]+\\.[^\\s@]+",
                s
            ));
        }
        let len = s.chars().count();
        if !(6..=64).contains(&len) {
            return Err(format!("String length {} is out of range 6..=64", len));
        }
        Ok(())
    } else {
        Err("Expected string".to_string())
    }
}

#[allow(dead_code)]
fn validate_string_pattern_0cb01cbc89d38ae3_len_max_64(value: &Value) -> Result<(), String> {
    if let Value::String(s) = value {
        static RE: std::sync::LazyLock<Regex> = std::sync::LazyLock::new(|| {
            Regex::new("^r-[0-9a-z]{4,32}$").expect("invalid pattern regex")
        });
        if !RE.is_match(s) {
            return Err(format!(
                "Value '{}' does not match pattern ^r-[0-9a-z]{{4,32}}$",
                s
            ));
        }
        let len = s.chars().count();
        if len > 64 {
            return Err(format!("String length {} is out of range ..=64", len));
        }
        Ok(())
    } else {
        Err("Expected string".to_string())
    }
}

/// Returns the schema config for organizations_organization (AWS::Organizations::Organization)
pub fn organizations_organization_config() -> AwsccSchemaConfig {
    AwsccSchemaConfig {
        aws_type_name: "AWS::Organizations::Organization",
        resource_type_name: "organizations.organization",
        has_tags: false,
        schema: ResourceSchema::new("awscc.organizations.organization")
        .with_description("Resource schema for AWS::Organizations::Organization")
        .attribute(
            AttributeSchema::new("arn", super::arn())
                .read_only()
                .with_description("The Amazon Resource Name (ARN) of an organization. (read-only)")
                .with_provider_name("Arn"),
        )
        .attribute(
            AttributeSchema::new("feature_set", AttributeType::StringEnum {
                name: "FeatureSet".to_string(),
                values: vec!["ALL".to_string(), "CONSOLIDATED_BILLING".to_string()],
                namespace: Some("awscc.organizations.organization".to_string()),
                to_dsl: None,
            })
                .with_description("Specifies the feature set supported by the new organization. Each feature set supports different levels of functionality.")
                .with_provider_name("FeatureSet")
                .with_default(Value::String("ALL".to_string())),
        )
        .attribute(
            AttributeSchema::new("id", AttributeType::Custom {
                name: "String(pattern)".to_string(),
                base: Box::new(AttributeType::String),
                validate: validate_string_pattern_2fd01fd52b67fc75,
                namespace: None,
                to_dsl: None,
            })
                .read_only()
                .with_description("The unique identifier (ID) of an organization. (read-only)")
                .with_provider_name("Id"),
        )
        .attribute(
            AttributeSchema::new("management_account_arn", super::arn())
                .read_only()
                .with_description("The Amazon Resource Name (ARN) of the account that is designated as the management account for the organization. (read-only)")
                .with_provider_name("ManagementAccountArn"),
        )
        .attribute(
            AttributeSchema::new("management_account_email", AttributeType::Custom {
                name: "String(pattern, len: 6..=64)".to_string(),
                base: Box::new(AttributeType::String),
                validate: validate_string_pattern_ec4d9bee0dcd262b_len_6_64,
                namespace: None,
                to_dsl: None,
            })
                .read_only()
                .with_description("The email address that is associated with the AWS account that is designated as the management account for the organization. (read-only)")
                .with_provider_name("ManagementAccountEmail"),
        )
        .attribute(
            AttributeSchema::new("management_account_id", super::aws_account_id())
                .read_only()
                .with_description("The unique identifier (ID) of the management account of an organization. (read-only)")
                .with_provider_name("ManagementAccountId"),
        )
        .attribute(
            AttributeSchema::new("root_id", AttributeType::Custom {
                name: "String(pattern, len: ..=64)".to_string(),
                base: Box::new(AttributeType::String),
                validate: validate_string_pattern_0cb01cbc89d38ae3_len_max_64,
                namespace: None,
                to_dsl: None,
            })
                .read_only()
                .with_description("The unique identifier (ID) for the root. (read-only)")
                .with_provider_name("RootId"),
        )
    }
}

/// Returns the resource type name and all enum valid values for this module
pub fn enum_valid_values() -> (
    &'static str,
    &'static [(&'static str, &'static [&'static str])],
) {
    (
        "organizations.organization",
        &[("feature_set", VALID_FEATURE_SET)],
    )
}

/// Maps DSL alias values back to canonical AWS values for this module.
/// e.g., ("ip_protocol", "all") -> Some("-1")
pub fn enum_alias_reverse(attr_name: &str, value: &str) -> Option<&'static str> {
    let _ = (attr_name, value);
    None
}
