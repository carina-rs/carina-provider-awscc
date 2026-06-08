//! permission_set schema definition for AWS Cloud Control
//!
//! Auto-generated from CloudFormation schema: AWS::SSO::PermissionSet
//!
//! DO NOT EDIT MANUALLY - regenerate with carina-codegen

use crate::schemas::config::AwsccSchemaConfig;
use carina_core::resource::{ConcreteValue, Value};
use carina_core::schema::{
    AttributeSchema, AttributeType, ResourceSchema, StructField, legacy_validator,
};
use regex::Regex;

#[allow(dead_code)]
fn validate_list_items_max_20(value: &Value) -> Result<(), String> {
    if let Value::Concrete(ConcreteValue::List(items)) = value {
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
    if let Value::Concrete(ConcreteValue::List(items)) = value {
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
    if let Value::Concrete(ConcreteValue::String(s)) = value {
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
    if let Value::Concrete(ConcreteValue::String(s)) = value {
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
fn validate_string_pattern_9b83f4f8f3673df5_len_1_32(value: &Value) -> Result<(), String> {
    if let Value::Concrete(ConcreteValue::String(s)) = value {
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
fn validate_string_pattern_4d6d630589930649_len_1_240(value: &Value) -> Result<(), String> {
    if let Value::Concrete(ConcreteValue::String(s)) = value {
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
    if let Value::Concrete(ConcreteValue::String(s)) = value {
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
        resource_type_name: "sso.PermissionSet",
        has_tags: true,
        schema: ResourceSchema::new("sso.PermissionSet")
        .with_description("Resource Type definition for SSO PermissionSet")
        .attribute(
            AttributeSchema::new("customer_managed_policy_references", AttributeType::custom(None, AttributeType::unordered_list(AttributeType::struct_("CustomerManagedPolicyReference".to_string(), vec![StructField::new("name", AttributeType::custom(None, AttributeType::string(), Some("[\\w+=,.@-]+".to_string()), Some((Some(1), Some(128))), legacy_validator(validate_string_pattern_9b83f4f8f3673df5_len_1_128), None)).required().with_provider_name("Name"),
                    StructField::new("path", AttributeType::custom(None, AttributeType::string(), Some("((/[A-Za-z0-9\\.,\\+@=_-]+)*)/".to_string()), Some((Some(1), Some(512))), legacy_validator(validate_string_pattern_b84fa12576539ca9_len_1_512), None)).with_provider_name("Path")])), None, None, legacy_validator(validate_list_items_max_20), None))
                .with_provider_name("CustomerManagedPolicyReferences")
                .with_block_name("customer_managed_policy_reference"),
        )
        .attribute(
            AttributeSchema::new("description", AttributeType::custom(None, AttributeType::string(), Some("[\\u0009\\u000A\\u000D\\u0020-\\u007E\\u00A1-\\u00FF]*".to_string()), Some((Some(1), Some(700))), legacy_validator(validate_string_pattern_9863be410e005e12_len_1_700), None))
                .with_description("The permission set description.")
                .with_provider_name("Description"),
        )
        .attribute(
            AttributeSchema::new("inline_policy", AttributeType::map(AttributeType::string()))
                .with_description("The inline policy to put in permission set.")
                .with_provider_name("InlinePolicy"),
        )
        .attribute(
            AttributeSchema::new("instance_arn", carina_aws_types::sso_instance_arn())
                .required()
                .create_only()
                .with_description("The sso instance arn that the permission set is owned.")
                .with_provider_name("InstanceArn"),
        )
        .attribute(
            AttributeSchema::new("managed_policies", AttributeType::custom(None, AttributeType::unordered_list(AttributeType::string()), None, None, legacy_validator(validate_list_items_max_20), None))
                .with_provider_name("ManagedPolicies"),
        )
        .attribute(
            AttributeSchema::new("name", AttributeType::custom(None, AttributeType::string(), Some("[\\w+=,.@-]+".to_string()), Some((Some(1), Some(32))), legacy_validator(validate_string_pattern_9b83f4f8f3673df5_len_1_32), None))
                .required()
                .create_only()
                .with_description("The name you want to assign to this permission set.")
                .with_provider_name("Name"),
        )
        .attribute(
            AttributeSchema::new("permission_set_arn", carina_aws_types::sso_permission_set_arn())
                .read_only()
                .with_description("The permission set that the policy will be attached to (read-only)")
                .with_provider_name("PermissionSetArn"),
        )
        .attribute(
            AttributeSchema::new("permissions_boundary", AttributeType::struct_("PermissionsBoundary".to_string(), vec![StructField::new("customer_managed_policy_reference", AttributeType::struct_("CustomerManagedPolicyReference".to_string(), vec![StructField::new("name", AttributeType::custom(None, AttributeType::string(), Some("[\\w+=,.@-]+".to_string()), Some((Some(1), Some(128))), legacy_validator(validate_string_pattern_9b83f4f8f3673df5_len_1_128), None)).required().with_provider_name("Name"),
                    StructField::new("path", AttributeType::custom(None, AttributeType::string(), Some("((/[A-Za-z0-9\\.,\\+@=_-]+)*)/".to_string()), Some((Some(1), Some(512))), legacy_validator(validate_string_pattern_b84fa12576539ca9_len_1_512), None)).with_provider_name("Path")])).with_provider_name("CustomerManagedPolicyReference"),
                    StructField::new("managed_policy_arn", carina_aws_types::arn()).with_provider_name("ManagedPolicyArn")]))
                .with_provider_name("PermissionsBoundary"),
        )
        .attribute(
            AttributeSchema::new("relay_state_type", AttributeType::custom(None, AttributeType::string(), Some("[a-zA-Z0-9&amp;$@#\\/%?=~\\-_'&quot;|!:,.;*+\\[\\]\\ \\(\\)\\{\\}]+".to_string()), Some((Some(1), Some(240))), legacy_validator(validate_string_pattern_4d6d630589930649_len_1_240), None))
                .with_description("The relay state URL that redirect links to any service in the AWS Management Console.")
                .with_provider_name("RelayStateType"),
        )
        .attribute(
            AttributeSchema::new("session_duration", AttributeType::custom(None, AttributeType::string(), Some("^(-?)P(?=\\d|T\\d)(?:(\\d+)Y)?(?:(\\d+)M)?(?:(\\d+)([DW]))?(?:T(?:(\\d+)H)?(?:(\\d+)M)?(?:(\\d+(?:\\.\\d+)?)S)?)?$".to_string()), Some((Some(1), Some(100))), legacy_validator(validate_string_pattern_1e58d8243b46a2f1_len_1_100), None))
                .with_description("The length of time that a user can be signed in to an AWS account.")
                .with_provider_name("SessionDuration"),
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
        .with_def("CustomerManagedPolicyReference", AttributeType::struct_("CustomerManagedPolicyReference".to_string(), vec![StructField::new("name", AttributeType::custom(None, AttributeType::string(), Some("[\\w+=,.@-]+".to_string()), Some((Some(1), Some(128))), legacy_validator(validate_string_pattern_9b83f4f8f3673df5_len_1_128), None)).required().with_provider_name("Name"),
                    StructField::new("path", AttributeType::custom(None, AttributeType::string(), Some("((/[A-Za-z0-9\\.,\\+@=_-]+)*)/".to_string()), Some((Some(1), Some(512))), legacy_validator(validate_string_pattern_b84fa12576539ca9_len_1_512), None)).with_provider_name("Path")]))
        .with_def("PermissionsBoundary", AttributeType::struct_("PermissionsBoundary".to_string(), vec![StructField::new("customer_managed_policy_reference", AttributeType::struct_("CustomerManagedPolicyReference".to_string(), vec![StructField::new("name", AttributeType::custom(None, AttributeType::string(), Some("[\\w+=,.@-]+".to_string()), Some((Some(1), Some(128))), legacy_validator(validate_string_pattern_9b83f4f8f3673df5_len_1_128), None)).required().with_provider_name("Name"),
                    StructField::new("path", AttributeType::custom(None, AttributeType::string(), Some("((/[A-Za-z0-9\\.,\\+@=_-]+)*)/".to_string()), Some((Some(1), Some(512))), legacy_validator(validate_string_pattern_b84fa12576539ca9_len_1_512), None)).with_provider_name("Path")])).with_provider_name("CustomerManagedPolicyReference"),
                    StructField::new("managed_policy_arn", carina_aws_types::arn()).with_provider_name("ManagedPolicyArn")]))
    }
}

/// Returns the resource type name and all enum valid values for this module
pub fn enum_valid_values() -> (
    &'static str,
    &'static [(&'static str, &'static [&'static str])],
) {
    ("sso.PermissionSet", &[])
}
