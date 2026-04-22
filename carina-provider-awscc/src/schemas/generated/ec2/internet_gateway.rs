//! internet_gateway schema definition for AWS Cloud Control
//!
//! Auto-generated from CloudFormation schema: AWS::EC2::InternetGateway
//!
//! DO NOT EDIT MANUALLY - regenerate with carina-codegen

use super::AwsccSchemaConfig;
use super::tags_type;
use super::validate_tags_map;
use carina_core::schema::{AttributeSchema, ResourceSchema};

/// Returns the schema config for ec2_internet_gateway (AWS::EC2::InternetGateway)
pub fn ec2_internet_gateway_config() -> AwsccSchemaConfig {
    AwsccSchemaConfig {
        aws_type_name: "AWS::EC2::InternetGateway",
        resource_type_name: "ec2.InternetGateway",
        has_tags: true,
        schema: ResourceSchema::new("awscc.ec2.InternetGateway")
        .with_description("Allocates an internet gateway for use with a VPC. After creating the Internet gateway, you then attach it to a VPC.")
        .attribute(
            AttributeSchema::new("internet_gateway_id", super::internet_gateway_id())
                .read_only()
                .with_description(" (read-only)")
                .with_provider_name("InternetGatewayId"),
        )
        .attribute(
            AttributeSchema::new("tags", tags_type())
                .with_description("Any tags to assign to the internet gateway.")
                .with_provider_name("Tags"),
        )
        .force_replace()
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
    ("ec2.InternetGateway", &[])
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
