//! internet_gateway schema definition for AWS Cloud Control
//!
//! Auto-generated from CloudFormation schema: AWS::EC2::InternetGateway
//!
//! DO NOT EDIT MANUALLY - regenerate with carina-codegen

use crate::schemas::config::AwsccSchemaConfig;
use carina_core::schema::{AttributeSchema, ResourceSchema};

/// Returns the schema config for ec2_internet_gateway (AWS::EC2::InternetGateway)
pub fn ec2_internet_gateway_config() -> AwsccSchemaConfig {
    AwsccSchemaConfig {
        aws_type_name: "AWS::EC2::InternetGateway",
        resource_type_name: "ec2.InternetGateway",
        primary_identifier: &[crate::schemas::config::PrimaryIdentifierAttribute { provider_name: "InternetGatewayId", dsl_name: "internet_gateway_id" }],
        has_tags: true,
        schema: ResourceSchema::new("ec2.InternetGateway")
	        .with_description("Allocates an internet gateway for use with a VPC. After creating the Internet gateway, you then attach it to a VPC.")
        .attribute(
            AttributeSchema::new("internet_gateway_id", carina_aws_types::internet_gateway_id())
                .read_only()
                .with_description(" (read-only)")
                .with_provider_name("InternetGatewayId"),
        )
        .attribute(
            AttributeSchema::new("tags", carina_aws_types::tags_type())
                .with_description("Any tags to assign to the internet gateway.")
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

/// Returns the resource type name and all enum valid values for this module
pub fn enum_valid_values() -> (
    &'static str,
    &'static [(&'static str, &'static [&'static str])],
) {
    ("ec2.InternetGateway", &[])
}

/// Returns the IAM permissions declared by the CloudFormation handler for this operation.
pub fn required_permissions(op: carina_core::effect::PlanOp) -> &'static [&'static str] {
    match op {
        carina_core::effect::PlanOp::Create => &[
            "ec2:CreateInternetGateway",
            "ec2:CreateTags",
            "ec2:DescribeInternetGateways",
        ],
        carina_core::effect::PlanOp::Read => &["ec2:DescribeInternetGateways"],
        carina_core::effect::PlanOp::Update => &[
            "ec2:DeleteTags",
            "ec2:CreateTags",
            "ec2:DescribeInternetGateways",
        ],
        carina_core::effect::PlanOp::Delete => {
            &["ec2:DeleteInternetGateway", "ec2:DescribeInternetGateways"]
        }
    }
}
