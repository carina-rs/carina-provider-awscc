//! account schema definition for AWS Cloud Control
//!
//! Auto-generated from CloudFormation schema: AWS::Organizations::Account
//!
//! DO NOT EDIT MANUALLY - regenerate with carina-codegen

use super::AwsccSchemaConfig;
use super::tags_type;
use carina_core::resource::Value;
use carina_core::schema::{AttributeSchema, AttributeType, ResourceSchema};
use regex::Regex;

const VALID_JOINED_METHOD: &[&str] = &["INVITED", "CREATED"];

const VALID_STATE: &[&str] = &[
    "PENDING_ACTIVATION",
    "ACTIVE",
    "SUSPENDED",
    "PENDING_CLOSURE",
    "CLOSED",
];

const VALID_STATUS: &[&str] = &["ACTIVE", "SUSPENDED", "PENDING_CLOSURE"];

#[allow(dead_code)]
fn validate_string_pattern_6fa92970742ee8e6(value: &Value) -> Result<(), String> {
    if let Value::String(s) = value {
        static RE: std::sync::LazyLock<Regex> = std::sync::LazyLock::new(|| {
            Regex::new("^(r-[0-9a-z]{4,32})|(ou-[0-9a-z]{4,32}-[a-z0-9]{8,32})$")
                .expect("invalid pattern regex")
        });
        if RE.is_match(s) {
            Ok(())
        } else {
            Err(format!(
                "Value '{}' does not match pattern ^(r-[0-9a-z]{{4,32}})|(ou-[0-9a-z]{{4,32}}-[a-z0-9]{{8,32}})$",
                s
            ))
        }
    } else {
        Err("Expected string".to_string())
    }
}

#[allow(dead_code)]
fn validate_string_pattern_9329bdea96a93739_len_max_256(value: &Value) -> Result<(), String> {
    if let Value::String(s) = value {
        static RE: std::sync::LazyLock<Regex> =
            std::sync::LazyLock::new(|| Regex::new("[\\s\\S]*").expect("invalid pattern regex"));
        if !RE.is_match(s) {
            return Err(format!("Value '{}' does not match pattern [\\s\\S]*", s));
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
fn validate_string_pattern_9329bdea96a93739_len_1_128(value: &Value) -> Result<(), String> {
    if let Value::String(s) = value {
        static RE: std::sync::LazyLock<Regex> =
            std::sync::LazyLock::new(|| Regex::new("[\\s\\S]*").expect("invalid pattern regex"));
        if !RE.is_match(s) {
            return Err(format!("Value '{}' does not match pattern [\\s\\S]*", s));
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
fn validate_string_pattern_3af299ea99241fab_len_1_50(value: &Value) -> Result<(), String> {
    if let Value::String(s) = value {
        static RE: std::sync::LazyLock<Regex> = std::sync::LazyLock::new(|| {
            Regex::new("[\\u0020-\\u007E]+").expect("invalid pattern regex")
        });
        if !RE.is_match(s) {
            return Err(format!(
                "Value '{}' does not match pattern [\\u0020-\\u007E]+",
                s
            ));
        }
        let len = s.chars().count();
        if !(1..=50).contains(&len) {
            return Err(format!("String length {} is out of range 1..=50", len));
        }
        Ok(())
    } else {
        Err("Expected string".to_string())
    }
}

#[allow(dead_code)]
fn validate_string_pattern_253e7eb79a4beec5_len_1_64(value: &Value) -> Result<(), String> {
    if let Value::String(s) = value {
        static RE: std::sync::LazyLock<Regex> = std::sync::LazyLock::new(|| {
            Regex::new("[\\w+=,.@-]{1,64}").expect("invalid pattern regex")
        });
        if !RE.is_match(s) {
            return Err(format!(
                "Value '{}' does not match pattern [\\w+=,.@-]{{1,64}}",
                s
            ));
        }
        let len = s.chars().count();
        if !(1..=64).contains(&len) {
            return Err(format!("String length {} is out of range 1..=64", len));
        }
        Ok(())
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

/// Returns the schema config for organizations_account (AWS::Organizations::Account)
pub fn organizations_account_config() -> AwsccSchemaConfig {
    AwsccSchemaConfig {
        aws_type_name: "AWS::Organizations::Account",
        resource_type_name: "organizations.account",
        has_tags: true,
        schema: ResourceSchema::new("awscc.organizations.account")
        .with_description("You can use AWS::Organizations::Account to manage accounts in organization.")
        .attribute(
            AttributeSchema::new("account_id", super::aws_account_id())
                .read_only()
                .with_description("If the account was created successfully, the unique identifier (ID) of the new account. (read-only)")
                .with_provider_name("AccountId"),
        )
        .attribute(
            AttributeSchema::new("account_name", AttributeType::Custom {
                name: "String(pattern, len: 1..=50)".to_string(),
                base: Box::new(AttributeType::String),
                validate: validate_string_pattern_3af299ea99241fab_len_1_50,
                namespace: None,
                to_dsl: None,
            })
                .required()
                .with_description("The friendly name of the member account.")
                .with_provider_name("AccountName"),
        )
        .attribute(
            AttributeSchema::new("arn", super::arn())
                .read_only()
                .with_description("The Amazon Resource Name (ARN) of the account. (read-only)")
                .with_provider_name("Arn"),
        )
        .attribute(
            AttributeSchema::new("email", AttributeType::Custom {
                name: "String(pattern, len: 6..=64)".to_string(),
                base: Box::new(AttributeType::String),
                validate: validate_string_pattern_ec4d9bee0dcd262b_len_6_64,
                namespace: None,
                to_dsl: None,
            })
                .required()
                .with_description("The email address of the owner to assign to the new member account.")
                .with_provider_name("Email"),
        )
        .attribute(
            AttributeSchema::new("joined_method", AttributeType::StringEnum {
                name: "JoinedMethod".to_string(),
                values: vec!["INVITED".to_string(), "CREATED".to_string()],
                namespace: Some("awscc.organizations.account".to_string()),
                to_dsl: None,
            })
                .read_only()
                .with_description("The method by which the account joined the organization. (read-only)")
                .with_provider_name("JoinedMethod"),
        )
        .attribute(
            AttributeSchema::new("joined_timestamp", AttributeType::String)
                .read_only()
                .with_description("The date the account became a part of the organization. (read-only)")
                .with_provider_name("JoinedTimestamp"),
        )
        .attribute(
            AttributeSchema::new("parent_ids", AttributeType::unordered_list(AttributeType::Custom {
                name: "String(pattern)".to_string(),
                base: Box::new(AttributeType::String),
                validate: validate_string_pattern_6fa92970742ee8e6,
                namespace: None,
                to_dsl: None,
            }))
                .with_description("List of parent nodes for the member account. Currently only one parent at a time is supported. Default is root.")
                .with_provider_name("ParentIds"),
        )
        .attribute(
            AttributeSchema::new("role_name", AttributeType::Custom {
                name: "String(pattern, len: 1..=64)".to_string(),
                base: Box::new(AttributeType::String),
                validate: validate_string_pattern_253e7eb79a4beec5_len_1_64,
                namespace: None,
                to_dsl: None,
            })
                .write_only()
                .with_description("The name of an IAM role that AWS Organizations automatically preconfigures in the new member account. Default name is OrganizationAccountAccessRole if not specified.")
                .with_provider_name("RoleName")
                .with_default(Value::String("OrganizationAccountAccessRole".to_string())),
        )
        .attribute(
            AttributeSchema::new("state", AttributeType::StringEnum {
                name: "State".to_string(),
                values: vec!["PENDING_ACTIVATION".to_string(), "ACTIVE".to_string(), "SUSPENDED".to_string(), "PENDING_CLOSURE".to_string(), "CLOSED".to_string()],
                namespace: Some("awscc.organizations.account".to_string()),
                to_dsl: None,
            })
                .read_only()
                .with_description("The state of the account in the organization. (read-only)")
                .with_provider_name("State"),
        )
        .attribute(
            AttributeSchema::new("status", AttributeType::StringEnum {
                name: "Status".to_string(),
                values: vec!["ACTIVE".to_string(), "SUSPENDED".to_string(), "PENDING_CLOSURE".to_string()],
                namespace: Some("awscc.organizations.account".to_string()),
                to_dsl: None,
            })
                .read_only()
                .with_description("The status of the account in the organization. (read-only)")
                .with_provider_name("Status"),
        )
        .attribute(
            AttributeSchema::new("tags", tags_type())
                .with_description("A list of tags that you want to attach to the newly created account. For each tag in the list, you must specify both a tag key and a value.")
                .with_provider_name("Tags"),
        )
    }
}

/// Returns the resource type name and all enum valid values for this module
pub fn enum_valid_values() -> (
    &'static str,
    &'static [(&'static str, &'static [&'static str])],
) {
    (
        "organizations.account",
        &[
            ("joined_method", VALID_JOINED_METHOD),
            ("state", VALID_STATE),
            ("status", VALID_STATUS),
        ],
    )
}

/// Maps DSL alias values back to canonical AWS values for this module.
/// e.g., ("ip_protocol", "all") -> Some("-1")
pub fn enum_alias_reverse(attr_name: &str, value: &str) -> Option<&'static str> {
    let _ = (attr_name, value);
    None
}
