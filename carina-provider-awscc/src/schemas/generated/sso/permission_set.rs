//! permission_set schema definition for AWS Cloud Control
//!
//! Auto-generated from CloudFormation schema: AWS::SSO::PermissionSet
//!
//! DO NOT EDIT MANUALLY - regenerate with carina-codegen

use crate::schemas::config::{AwsccSchemaConfig, TagsProperty, TagsWire};
use carina_core::resource::{ConcreteValue, Value};
use carina_core::schema::{AttributeSchema, AttributeType, ResourceSchema, StructField};
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

/// Returns the schema config for sso_permission_set (AWS::SSO::PermissionSet)
pub fn sso_permission_set_config() -> AwsccSchemaConfig {
    AwsccSchemaConfig {
        aws_type_name: "AWS::SSO::PermissionSet",
        resource_type_name: "sso.PermissionSet",
        primary_identifier: &[crate::schemas::config::PrimaryIdentifierAttribute { provider_name: "InstanceArn", dsl_name: "instance_arn" }, crate::schemas::config::PrimaryIdentifierAttribute { provider_name: "PermissionSetArn", dsl_name: "permission_set_arn" }],
        tags: Some(TagsProperty { provider_name: "Tags", wire: TagsWire::KeyValueList }),
        schema: ResourceSchema::new("sso.PermissionSet")
	        .with_description("Resource Type definition for SSO PermissionSet")
        .attribute(
            AttributeSchema::optional("customer_managed_policy_references", AttributeType::refined_list(AttributeType::struct_("CustomerManagedPolicyReference".to_string(), vec![StructField::required("name", AttributeType::refined_string(None, Some("[\\w+=,.@-]+".to_string()), Some((Some(1), Some(128))), None)).with_provider_name("Name").build(),
                    StructField::optional("path", AttributeType::refined_string(None, Some("((/[A-Za-z0-9\\.,\\+@=_-]+)*)/".to_string()), Some((Some(1), Some(512))), None)).with_provider_name("Path").build()]), false, Some((None, Some(20)))))
                .with_provider_name("CustomerManagedPolicyReferences")
                .with_block_name("customer_managed_policy_reference")
                .build(),
        )
        .attribute(
            AttributeSchema::optional("description", AttributeType::refined_string(None, Some("[\\u0009\\u000A\\u000D\\u0020-\\u007E\\u00A1-\\u00FF]*".to_string()), Some((Some(1), Some(700))), None))
                .with_description("The permission set description.")
                .with_provider_name("Description")
                .build(),
        )
        .attribute(
            AttributeSchema::optional("inline_policy", AttributeType::map(AttributeType::string()))
                .with_description("The inline policy to put in permission set.")
                .with_provider_name("InlinePolicy")
                .build(),
        )
        .attribute(
            AttributeSchema::required("instance_arn", carina_aws_types::sso_instance_arn())
                .create_only()
                .with_description("The sso instance arn that the permission set is owned.")
                .with_provider_name("InstanceArn")
                .build(),
        )
        .attribute(
            AttributeSchema::optional("managed_policies", AttributeType::refined_list(AttributeType::string(), false, Some((None, Some(20)))))
                .with_provider_name("ManagedPolicies")
                .build(),
        )
        .attribute(
            AttributeSchema::required("name", AttributeType::refined_string(None, Some("[\\w+=,.@-]+".to_string()), Some((Some(1), Some(32))), None))
                .create_only()
                .with_description("The name you want to assign to this permission set.")
                .with_provider_name("Name")
                .build(),
        )
        .attribute(
            AttributeSchema::provider_populated("permission_set_arn", carina_aws_types::sso_permission_set_arn())
                .with_description("The permission set that the policy will be attached to (read-only)")
                .with_provider_name("PermissionSetArn")
                .build(),
        )
        .attribute(
            AttributeSchema::optional("permissions_boundary", AttributeType::struct_("PermissionsBoundary".to_string(), vec![StructField::optional("customer_managed_policy_reference", AttributeType::struct_("CustomerManagedPolicyReference".to_string(), vec![StructField::required("name", AttributeType::refined_string(None, Some("[\\w+=,.@-]+".to_string()), Some((Some(1), Some(128))), None)).with_provider_name("Name").build(),
                    StructField::optional("path", AttributeType::refined_string(None, Some("((/[A-Za-z0-9\\.,\\+@=_-]+)*)/".to_string()), Some((Some(1), Some(512))), None)).with_provider_name("Path").build()])).with_provider_name("CustomerManagedPolicyReference").build(),
                    StructField::optional("managed_policy_arn", carina_aws_types::arn()).with_provider_name("ManagedPolicyArn").build()]))
                .with_provider_name("PermissionsBoundary")
                .build(),
        )
        .attribute(
            AttributeSchema::optional("relay_state_type", AttributeType::refined_string(None, Some("[a-zA-Z0-9&amp;$@#\\/%?=~\\-_'&quot;|!:,.;*+\\[\\]\\ \\(\\)\\{\\}]+".to_string()), Some((Some(1), Some(240))), None))
                .with_description("The relay state URL that redirect links to any service in the AWS Management Console.")
                .with_provider_name("RelayStateType")
                .build(),
        )
        .attribute(
            AttributeSchema::optional("session_duration", AttributeType::refined_string(None, Some("^(-?)P(?=\\d|T\\d)(?:(\\d+)Y)?(?:(\\d+)M)?(?:(\\d+)([DW]))?(?:T(?:(\\d+)H)?(?:(\\d+)M)?(?:(\\d+(?:\\.\\d+)?)S)?)?$".to_string()), Some((Some(1), Some(100))), None))
                .with_description("The length of time that a user can be signed in to an AWS account.")
                .with_provider_name("SessionDuration")
                .build(),
        )
        .attribute(
            AttributeSchema::optional("tags", carina_aws_types::tags_type())
                .with_provider_name("Tags")
                .with_block_name("tag")
                .build(),
        )
        .with_unique_name_attribute("name")
        .with_declared_validators(vec![carina_provider_protocol::types::ValidatorType::TagsKeyValueCheck])
        .with_def("CustomerManagedPolicyReference", AttributeType::struct_("CustomerManagedPolicyReference".to_string(), vec![StructField::required("name", AttributeType::refined_string(None, Some("[\\w+=,.@-]+".to_string()), Some((Some(1), Some(128))), None)).with_provider_name("Name").build(),
                    StructField::optional("path", AttributeType::refined_string(None, Some("((/[A-Za-z0-9\\.,\\+@=_-]+)*)/".to_string()), Some((Some(1), Some(512))), None)).with_provider_name("Path").build()]))
        .with_def("PermissionsBoundary", AttributeType::struct_("PermissionsBoundary".to_string(), vec![StructField::optional("customer_managed_policy_reference", AttributeType::struct_("CustomerManagedPolicyReference".to_string(), vec![StructField::required("name", AttributeType::refined_string(None, Some("[\\w+=,.@-]+".to_string()), Some((Some(1), Some(128))), None)).with_provider_name("Name").build(),
                    StructField::optional("path", AttributeType::refined_string(None, Some("((/[A-Za-z0-9\\.,\\+@=_-]+)*)/".to_string()), Some((Some(1), Some(512))), None)).with_provider_name("Path").build()])).with_provider_name("CustomerManagedPolicyReference").build(),
                    StructField::optional("managed_policy_arn", carina_aws_types::arn()).with_provider_name("ManagedPolicyArn").build()]))
    }
}

#[allow(dead_code)]
fn validate_string_pattern_b84fa12576539ca9_len_1_512(value: &Value) -> Result<(), String> {
    if let Value::Concrete(ConcreteValue::String(s)) = value {
        let s = s.expose_plaintext();
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
        let s = s.expose_plaintext();
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
        let s = s.expose_plaintext();
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
        let s = s.expose_plaintext();
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
        let s = s.expose_plaintext();
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
        let s = s.expose_plaintext();
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
        let s = s.expose_plaintext();
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

/// Returns the resource type name and all enum valid values for this module
pub fn enum_valid_values() -> (
    &'static str,
    &'static [(&'static str, &'static [&'static str])],
) {
    ("sso.PermissionSet", &[])
}

/// Returns the IAM permissions declared by the CloudFormation handler for this operation.
pub fn required_permissions(op: carina_core::effect::PlanOp) -> &'static [&'static str] {
    match op {
        carina_core::effect::PlanOp::Create => &[
            "sso:CreatePermissionSet",
            "sso:PutInlinePolicyToPermissionSet",
            "sso:AttachManagedPolicyToPermissionSet",
            "sso:AttachCustomerManagedPolicyReferenceToPermissionSet",
            "sso:PutPermissionsBoundaryToPermissionSet",
            "sso:TagResource",
            "sso:DescribePermissionSet",
            "sso:ListTagsForResource",
            "sso:ListManagedPoliciesInPermissionSet",
            "sso:ListCustomerManagedPolicyReferencesInPermissionSet",
            "sso:GetInlinePolicyForPermissionSet",
            "sso:GetPermissionsBoundaryForPermissionSet",
        ],
        carina_core::effect::PlanOp::Read => &[
            "sso:DescribePermissionSet",
            "sso:ListTagsForResource",
            "sso:ListManagedPoliciesInPermissionSet",
            "sso:ListCustomerManagedPolicyReferencesInPermissionSet",
            "sso:GetInlinePolicyForPermissionSet",
            "sso:GetPermissionsBoundaryForPermissionSet",
        ],
        carina_core::effect::PlanOp::Update => &[
            "sso:UpdatePermissionSet",
            "sso:TagResource",
            "sso:UntagResource",
            "sso:ListTagsForResource",
            "sso:AttachManagedPolicyToPermissionSet",
            "sso:AttachCustomerManagedPolicyReferenceToPermissionSet",
            "sso:DetachManagedPolicyFromPermissionSet",
            "sso:DetachCustomerManagedPolicyReferenceFromPermissionSet",
            "sso:ListManagedPoliciesInPermissionSet",
            "sso:ListCustomerManagedPolicyReferencesInPermissionSet",
            "sso:PutInlinePolicyToPermissionSet",
            "sso:GetPermissionsBoundaryForPermissionSet",
            "sso:DeletePermissionsBoundaryFromPermissionSet",
            "sso:PutPermissionsBoundaryToPermissionSet",
            "sso:DeleteInlinePolicyFromPermissionSet",
            "sso:ProvisionPermissionSet",
            "sso:DescribePermissionSet",
            "sso:GetInlinePolicyForPermissionSet",
            "sso:DescribePermissionSetProvisioningStatus",
        ],
        carina_core::effect::PlanOp::Delete => &["sso:DeletePermissionSet"],
    }
}
