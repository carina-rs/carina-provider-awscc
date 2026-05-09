//! origin_access_control schema definition for AWS Cloud Control
//!
//! Auto-generated from CloudFormation schema: AWS::CloudFront::OriginAccessControl
//!
//! DO NOT EDIT MANUALLY - regenerate with carina-codegen

use super::AwsccSchemaConfig;
use carina_core::resource::Value;
use carina_core::schema::{
    AttributeSchema, AttributeType, ResourceSchema, StructField, legacy_validator,
};
use regex::Regex;

const VALID_ORIGIN_ACCESS_CONTROL_CONFIG_SIGNING_BEHAVIOR: &[&str] =
    &["always", "never", "no-override"];

#[allow(dead_code)]
fn validate_string_pattern_597c12a2d8028697(value: &Value) -> Result<(), String> {
    if let Value::String(s) = value {
        static RE: std::sync::LazyLock<Regex> = std::sync::LazyLock::new(|| {
            Regex::new("^(s3|mediastore|lambda|mediapackagev2)$").expect("invalid pattern regex")
        });
        if RE.is_match(s) {
            Ok(())
        } else {
            Err(format!(
                "Value '{}' does not match pattern ^(s3|mediastore|lambda|mediapackagev2)$",
                s
            ))
        }
    } else {
        Err("Expected string".to_string())
    }
}

#[allow(dead_code)]
fn validate_string_pattern_e0706e45c974b71e(value: &Value) -> Result<(), String> {
    if let Value::String(s) = value {
        static RE: std::sync::LazyLock<Regex> =
            std::sync::LazyLock::new(|| Regex::new("^(sigv4)$").expect("invalid pattern regex"));
        if RE.is_match(s) {
            Ok(())
        } else {
            Err(format!("Value '{}' does not match pattern ^(sigv4)$", s))
        }
    } else {
        Err("Expected string".to_string())
    }
}

/// Returns the schema config for cloudfront_origin_access_control (AWS::CloudFront::OriginAccessControl)
pub fn cloudfront_origin_access_control_config() -> AwsccSchemaConfig {
    AwsccSchemaConfig {
        aws_type_name: "AWS::CloudFront::OriginAccessControl",
        resource_type_name: "cloudfront.OriginAccessControl",
        has_tags: false,
        schema: ResourceSchema::new("cloudfront.OriginAccessControl")
        .with_description("Creates a new origin access control in CloudFront. After you create an origin access control, you can add it to an origin in a CloudFront distribution so that CloudFront sends authenticated (signed) requests to the origin.  This makes it possible to block public access to the origin, allowing viewers (users) to access the origin's content only through CloudFront.  For more information about using a CloudFront origin access control, see [Restricting access to an origin](https://docs.aws.amazon.com/AmazonCloudFront/latest/DeveloperGuide/private-content-restricting-access-to-origin.html) in the *Amazon CloudFront Developer Guide*.")
        .attribute(
            AttributeSchema::new("id", AttributeType::String)
                .read_only()
                .with_description(" (read-only)")
                .with_provider_name("Id"),
        )
        .attribute(
            AttributeSchema::new("origin_access_control_config", AttributeType::Struct {
                    name: "OriginAccessControlConfig".to_string(),
                    fields: vec![
                    StructField::new("description", AttributeType::String).with_description("A description of the origin access control.").with_provider_name("Description"),
                    StructField::new("name", AttributeType::String).required().with_description("A name to identify the origin access control. You can specify up to 64 characters.").with_provider_name("Name"),
                    StructField::new("origin_access_control_origin_type", AttributeType::Custom {
                semantic_name: None,
                pattern: Some("^(s3|mediastore|lambda|mediapackagev2)$".to_string()),
                length: None,
                base: Box::new(AttributeType::String),
                validate: legacy_validator(validate_string_pattern_597c12a2d8028697),
                namespace: None,
                to_dsl: None,
            }).required().with_description("The type of origin that this origin access control is for.").with_provider_name("OriginAccessControlOriginType"),
                    StructField::new("signing_behavior", AttributeType::StringEnum {
                name: "SigningBehavior".to_string(),
                values: vec!["always".to_string(), "never".to_string(), "no-override".to_string()],
                namespace: Some("awscc.cloudfront.OriginAccessControl".to_string()),
                to_dsl: Some(|s: &str| s.replace('-', "_")),
            }).required().with_description("Specifies which requests CloudFront signs (adds authentication information to). Specify ``always`` for the most common use case. For more information, see [origin access control advanced settings](https://docs.aws.amazon.com/AmazonCloudFront/latest/DeveloperGuide/private-content-restricting-access-to-s3.html#oac-advanced-settings) in the *Amazon CloudFront Developer Guide*. This field can have one of the following values: + ``always`` ? CloudFront signs all origin requests, overwriting the ``Authorization`` header from the viewer request if one exists. + ``never`` ? CloudFront doesn't sign any origin requests. This value turns off origin access control for all origins in all distributions that use this origin access control. + ``no-override`` ? If the viewer request doesn't contain the ``Authorization`` header, then CloudFront signs the origin request. If the viewer request contains the ``Authorization`` header, then CloudFront doesn't sign the origin request and instead passes along the ``Authorization`` header from the viewer request. *WARNING: To pass along the Authorization header from the viewer request, you must add the Authorization header to a cache policy for all cache behaviors that use origins associated with this origin access control.*").with_provider_name("SigningBehavior"),
                    StructField::new("signing_protocol", AttributeType::Custom {
                semantic_name: None,
                pattern: Some("^(sigv4)$".to_string()),
                length: None,
                base: Box::new(AttributeType::String),
                validate: legacy_validator(validate_string_pattern_e0706e45c974b71e),
                namespace: None,
                to_dsl: None,
            }).required().with_description("The signing protocol of the origin access control, which determines how CloudFront signs (authenticates) requests. The only valid value is ``sigv4``.").with_provider_name("SigningProtocol")
                    ],
                })
                .required()
                .with_description("The origin access control.")
                .with_provider_name("OriginAccessControlConfig"),
        )
    }
}

/// Returns the resource type name and all enum valid values for this module
pub fn enum_valid_values() -> (
    &'static str,
    &'static [(&'static str, &'static [&'static str])],
) {
    (
        "cloudfront.OriginAccessControl",
        &[(
            "signing_behavior",
            VALID_ORIGIN_ACCESS_CONTROL_CONFIG_SIGNING_BEHAVIOR,
        )],
    )
}

/// Maps DSL alias values back to canonical AWS values for this module.
/// e.g., ("ip_protocol", "all") -> Some("-1")
pub fn enum_alias_reverse(attr_name: &str, value: &str) -> Option<&'static str> {
    match (attr_name, value) {
        ("signing_behavior", "no_override") => Some("no-override"),
        _ => None,
    }
}

/// Returns all enum alias entries as (attr_name, alias, canonical) tuples.
pub fn enum_alias_entries() -> &'static [(&'static str, &'static str, &'static str)] {
    &[("signing_behavior", "no_override", "no-override")]
}
