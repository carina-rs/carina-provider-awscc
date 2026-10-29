//! organization schema definition for AWS Cloud Control
//!
//! Auto-generated from CloudFormation schema: AWS::Organizations::Organization
//!
//! DO NOT EDIT MANUALLY - regenerate with carina-codegen

use crate::schemas::config::AwsccSchemaConfig;
use carina_core::resource::{ConcreteValue, Value};
use carina_core::schema::{AttributeSchema, AttributeType, ResourceSchema, legacy_validator};
use regex::Regex;

pub fn arn() -> AttributeType {
    AttributeType::refined_string_with_validator(
        carina_aws_types::provider_type("organizations", "Organization", "Arn"),
        Some("^arn:(aws|aws-cn|aws-us-gov):organizations:.*$".to_string()),
        None,
        legacy_validator(|value| {
            if let Value::Concrete(ConcreteValue::String(s)) = value {
                let s = s.expose_plaintext();
                carina_aws_types::validate_service_arn(s, "organizations", None)
                    .map_err(|reason| format!("Invalid organizations ARN '{}': {}", s, reason))
            } else {
                Err("Expected string".to_string())
            }
        }),
        None,
    )
}

const VALID_FEATURE_SET: &[&str] = &["ALL", "CONSOLIDATED_BILLING"];

/// Returns the schema config for organizations_organization (AWS::Organizations::Organization)
#[rustfmt::skip]
pub fn organizations_organization_config() -> AwsccSchemaConfig {
    let mut schema = ResourceSchema::new("organizations.Organization")
        .with_description("Resource schema for AWS::Organizations::Organization");
    #[inline(never)]
    fn build_arn_attribute() -> AttributeSchema {
        AttributeSchema::provider_populated("arn", self::arn())
                .with_description("The Amazon Resource Name (ARN) of an organization. (read-only)")
                .with_provider_name("Arn")
            .build()
    }
    schema = schema.attribute(build_arn_attribute());
    #[inline(never)]
    fn build_feature_set_attribute() -> AttributeSchema {
        AttributeSchema::optional("feature_set", AttributeType::enum_(carina_core::schema::enum_identity("FeatureSet", Some("aws.organizations.Organization")), Some(vec!["ALL".to_string(), "CONSOLIDATED_BILLING".to_string()]), vec![("ALL".to_string(), "all".to_string()), ("CONSOLIDATED_BILLING".to_string(), "consolidated_billing".to_string())], None, None))
                .with_description("Specifies the feature set supported by the new organization. Each feature set supports different levels of functionality.")
                .with_provider_name("FeatureSet")
                .with_default(Value::Concrete(ConcreteValue::String("ALL".to_string().into())))
            .build()
    }
    schema = schema.attribute(build_feature_set_attribute());
    #[inline(never)]
    fn build_id_attribute() -> AttributeSchema {
        AttributeSchema::provider_populated("id", AttributeType::refined_string(None, Some("^o-[a-z0-9]{10,32}$".to_string()), None, None))
                .with_description("The unique identifier (ID) of an organization. (read-only)")
                .with_provider_name("Id")
            .build()
    }
    schema = schema.attribute(build_id_attribute());
    #[inline(never)]
    fn build_management_account_arn_attribute() -> AttributeSchema {
        AttributeSchema::provider_populated("management_account_arn", carina_aws_types::arn())
                .with_description("The Amazon Resource Name (ARN) of the account that is designated as the management account for the organization. (read-only)")
                .with_provider_name("ManagementAccountArn")
            .build()
    }
    schema = schema.attribute(build_management_account_arn_attribute());
    #[inline(never)]
    fn build_management_account_email_attribute() -> AttributeSchema {
        AttributeSchema::provider_populated("management_account_email", AttributeType::refined_string(None, Some("[^\\s@]+@[^\\s@]+\\.[^\\s@]+".to_string()), Some((Some(6), Some(64))), None))
                .with_description("The email address that is associated with the AWS account that is designated as the management account for the organization. (read-only)")
                .with_provider_name("ManagementAccountEmail")
            .build()
    }
    schema = schema.attribute(build_management_account_email_attribute());
    #[inline(never)]
    fn build_management_account_id_attribute() -> AttributeSchema {
        AttributeSchema::provider_populated("management_account_id", carina_aws_types::aws_account_id())
                .with_description("The unique identifier (ID) of the management account of an organization. (read-only)")
                .with_provider_name("ManagementAccountId")
            .build()
    }
    schema = schema.attribute(build_management_account_id_attribute());
    #[inline(never)]
    fn build_root_id_attribute() -> AttributeSchema {
        AttributeSchema::provider_populated("root_id", AttributeType::refined_string(None, Some("^r-[0-9a-z]{4,32}$".to_string()), Some((None, Some(64))), None))
                .with_description("The unique identifier (ID) for the root. (read-only)")
                .with_provider_name("RootId")
            .build()
    }
    schema = schema.attribute(build_root_id_attribute());
    schema = schema
        .with_conflicting_replacement();
    AwsccSchemaConfig {
        aws_type_name: "AWS::Organizations::Organization",
        resource_type_name: "organizations.Organization",
        primary_identifier: &[crate::schemas::config::PrimaryIdentifierAttribute { provider_name: "Id", dsl_name: "id" }],
        tags: None,
        schema,
    }
}

#[allow(dead_code)]
fn validate_string_pattern_ec4d9bee0dcd262b_len_6_64(value: &Value) -> Result<(), String> {
    if let Value::Concrete(ConcreteValue::String(s)) = value {
        let s = s.expose_plaintext();
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
    if let Value::Concrete(ConcreteValue::String(s)) = value {
        let s = s.expose_plaintext();
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

#[allow(dead_code)]
fn validate_string_pattern_2fd01fd52b67fc75(value: &Value) -> Result<(), String> {
    if let Value::Concrete(ConcreteValue::String(s)) = value {
        let s = s.expose_plaintext();
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

/// Returns the resource type name and all enum valid values for this module
pub fn enum_valid_values() -> (
    &'static str,
    &'static [(&'static str, &'static [&'static str])],
) {
    (
        "organizations.Organization",
        &[("feature_set", VALID_FEATURE_SET)],
    )
}

/// Returns the IAM permissions declared by the CloudFormation handler for this operation.
pub fn required_permissions(op: carina_core::effect::PlanOp) -> &'static [&'static str] {
    match op {
        carina_core::effect::PlanOp::Create => &[
            "organizations:CreateOrganization",
            "organizations:DescribeOrganization",
            "iam:CreateServiceLinkedRole",
            "organizations:ListRoots",
        ],
        carina_core::effect::PlanOp::Read => &[
            "organizations:DescribeOrganization",
            "organizations:ListRoots",
        ],
        carina_core::effect::PlanOp::Update => &["organizations:DescribeOrganization"],
        carina_core::effect::PlanOp::Delete => &[
            "organizations:DeleteOrganization",
            "organizations:DescribeOrganization",
        ],
    }
}
