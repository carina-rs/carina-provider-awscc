//! instance schema definition for AWS Cloud Control
//!
//! Auto-generated from CloudFormation schema: AWS::SSO::Instance
//!
//! DO NOT EDIT MANUALLY - regenerate with carina-codegen

use crate::schemas::config::AwsccSchemaConfig;
use carina_core::resource::{ConcreteValue, Value};
use carina_core::schema::{AttributeSchema, AttributeType, ResourceSchema};
use regex::Regex;

const VALID_STATUS: &[&str] = &["CREATE_IN_PROGRESS", "DELETE_IN_PROGRESS", "ACTIVE"];

#[allow(dead_code)]
fn validate_list_items_max_75(value: &Value) -> Result<(), String> {
    if let Value::Concrete(ConcreteValue::List(items)) = value {
        let len = items.len();
        if len > 75 {
            Err(format!("List has {} items, expected ..=75", len))
        } else {
            Ok(())
        }
    } else {
        Err("Expected list".to_string())
    }
}

/// Returns the schema config for sso_instance (AWS::SSO::Instance)
pub fn sso_instance_config() -> AwsccSchemaConfig {
    AwsccSchemaConfig {
        aws_type_name: "AWS::SSO::Instance",
        resource_type_name: "sso.Instance",
        primary_identifier: &["InstanceArn"],
        has_tags: true,
        schema: ResourceSchema::new("sso.Instance")
	        .with_description("Resource Type definition for Identity Center (SSO) Instance")
        .attribute(
            AttributeSchema::new("identity_store_id", carina_aws_types::identity_store_id())
                .read_only()
                .with_description("The ID of the identity store associated with the created Identity Center (SSO) Instance (read-only)")
                .with_provider_name("IdentityStoreId"),
        )
        .attribute(
            AttributeSchema::new("instance_arn", carina_aws_types::sso_instance_arn())
                .read_only()
                .with_description("The SSO Instance ARN that is returned upon creation of the Identity Center (SSO) Instance (read-only)")
                .with_provider_name("InstanceArn"),
        )
        .attribute(
            AttributeSchema::new("name", AttributeType::refined_string(None, Some("^[\\w+=,.@-]+$".to_string()), Some((Some(1), Some(32))), None))
                .with_description("The name you want to assign to this Identity Center (SSO) Instance")
                .with_provider_name("Name"),
        )
        .attribute(
            AttributeSchema::new("owner_account_id", carina_aws_types::aws_account_id())
                .read_only()
                .with_description("The AWS accountId of the owner of the Identity Center (SSO) Instance (read-only)")
                .with_provider_name("OwnerAccountId"),
        )
        .attribute(
            AttributeSchema::new("status", AttributeType::enum_(carina_core::schema::enum_identity("Status", Some("aws.sso.Instance")), Some(vec!["CREATE_IN_PROGRESS".to_string(), "DELETE_IN_PROGRESS".to_string(), "ACTIVE".to_string()]), vec![("CREATE_IN_PROGRESS".to_string(), "create_in_progress".to_string()), ("DELETE_IN_PROGRESS".to_string(), "delete_in_progress".to_string()), ("ACTIVE".to_string(), "active".to_string())], None, None))
                .read_only()
                .with_description("The status of the Identity Center (SSO) Instance, create_in_progress/delete_in_progress/active (read-only)")
                .with_provider_name("Status"),
        )
        .attribute(
            AttributeSchema::new("tags", carina_aws_types::tags_type())
                .with_provider_name("Tags")
                .with_block_name("tag"),
        )
        .with_validator(|attrs| {
            let mut errors = Vec::new();
            if let Err(mut e) = carina_aws_types::validate_tags_map(attrs) {
                errors.append(&mut e);
            }
            if errors.is_empty() { Ok(()) } else { Err(errors) }
        })
    }
}

#[allow(dead_code)]
fn validate_string_pattern_9b83f4f8f3673df5_len_max_256(value: &Value) -> Result<(), String> {
    if let Value::Concrete(ConcreteValue::String(s)) = value {
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
fn validate_string_pattern_9b83f4f8f3673df5_len_1_128(value: &Value) -> Result<(), String> {
    if let Value::Concrete(ConcreteValue::String(s)) = value {
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
fn validate_string_pattern_5a2bd7daee6344f1_len_1_32(value: &Value) -> Result<(), String> {
    if let Value::Concrete(ConcreteValue::String(s)) = value {
        static RE: std::sync::LazyLock<Regex> = std::sync::LazyLock::new(|| {
            Regex::new("^[\\w+=,.@-]+$").expect("invalid pattern regex")
        });
        if !RE.is_match(s) {
            return Err(format!(
                "Value '{}' does not match pattern ^[\\w+=,.@-]+$",
                s
            ));
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

/// Returns the resource type name and all enum valid values for this module
pub fn enum_valid_values() -> (
    &'static str,
    &'static [(&'static str, &'static [&'static str])],
) {
    ("sso.Instance", &[("status", VALID_STATUS)])
}

/// Returns the IAM permissions declared by the CloudFormation handler for this operation.
pub fn required_permissions(op: carina_core::effect::PlanOp) -> &'static [&'static str] {
    match op {
        carina_core::effect::PlanOp::Create => &[
            "sso:CreateInstance",
            "sso:DescribeInstance",
            "sso:TagResource",
            "iam:CreateServiceLinkedRole",
            "sso:TagInstance",
            "sso:ListTagsForResource",
            "identitystore:CreateIdentityStore",
        ],
        carina_core::effect::PlanOp::Read => &["sso:DescribeInstance", "sso:ListTagsForResource"],
        carina_core::effect::PlanOp::Update => &[
            "sso:UpdateInstance",
            "sso:TagResource",
            "sso:UntagResource",
            "sso:ListTagsForResource",
            "sso:TagInstance",
            "sso:DescribeInstance",
        ],
        carina_core::effect::PlanOp::Delete => {
            &["sso:DeleteInstance", "identitystore:DeleteIdentityStore"]
        }
    }
}
