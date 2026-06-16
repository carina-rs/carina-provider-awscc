//! egress_only_internet_gateway schema definition for AWS Cloud Control
//!
//! Auto-generated from CloudFormation schema: AWS::EC2::EgressOnlyInternetGateway
//!
//! DO NOT EDIT MANUALLY - regenerate with carina-codegen

use crate::schemas::config::AwsccSchemaConfig;
use carina_core::schema::{AttributeSchema, ResourceSchema};

/// Returns the schema config for ec2_egress_only_internet_gateway (AWS::EC2::EgressOnlyInternetGateway)
pub fn ec2_egress_only_internet_gateway_config() -> AwsccSchemaConfig {
    AwsccSchemaConfig {
        aws_type_name: "AWS::EC2::EgressOnlyInternetGateway",
        resource_type_name: "ec2.EgressOnlyInternetGateway",
        primary_identifier: &["Id"],
        has_tags: true,
        schema: ResourceSchema::new("ec2.EgressOnlyInternetGateway")
            .with_description("Resource Type definition for AWS::EC2::EgressOnlyInternetGateway")
            .attribute(
                AttributeSchema::new("id", carina_aws_types::egress_only_internet_gateway_id())
                    .read_only()
                    .with_description(
                        "Service Generated ID of the EgressOnlyInternetGateway (read-only)",
                    )
                    .with_provider_name("Id"),
            )
            .attribute(
                AttributeSchema::new("tags", carina_aws_types::tags_type())
                    .with_description("Any tags assigned to the egress only internet gateway.")
                    .with_provider_name("Tags")
                    .with_block_name("tag"),
            )
            .attribute(
                AttributeSchema::new("vpc_id", carina_aws_types::vpc_id())
                    .required()
                    .create_only()
                    .with_description(
                        "The ID of the VPC for which to create the egress-only internet gateway.",
                    )
                    .with_provider_name("VpcId"),
            )
            .with_validator(|attrs| {
                let mut errors = Vec::new();
                if let Err(mut e) = carina_aws_types::validate_tags_map(attrs) {
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
    ("ec2.EgressOnlyInternetGateway", &[])
}

/// Returns the IAM permissions declared by the CloudFormation handler for this operation.
pub fn required_permissions(op: carina_core::effect::PlanOp) -> &'static [&'static str] {
    match op {
        carina_core::effect::PlanOp::Create => &[
            "ec2:CreateEgressOnlyInternetGateway",
            "ec2:CreateTags",
            "ec2:DescribeEgressOnlyInternetGateways",
        ],
        carina_core::effect::PlanOp::Read => {
            &["ec2:DescribeEgressOnlyInternetGateways", "ec2:DescribeTags"]
        }
        carina_core::effect::PlanOp::Update => &[
            "ec2:DeleteTags",
            "ec2:CreateTags",
            "ec2:DescribeEgressOnlyInternetGateways",
        ],
        carina_core::effect::PlanOp::Delete => &[
            "ec2:DeleteEgressOnlyInternetGateway",
            "ec2:DescribeEgressOnlyInternetGateways",
            "ec2:DescribeVpcs",
            "ec2:DeleteTags",
        ],
    }
}
