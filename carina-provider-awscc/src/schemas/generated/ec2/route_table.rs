//! route_table schema definition for AWS Cloud Control
//!
//! Auto-generated from CloudFormation schema: AWS::EC2::RouteTable
//!
//! DO NOT EDIT MANUALLY - regenerate with carina-codegen

use crate::schemas::config::AwsccSchemaConfig;
use carina_core::schema::{AttributeSchema, ResourceSchema};

/// Returns the schema config for ec2_route_table (AWS::EC2::RouteTable)
pub fn ec2_route_table_config() -> AwsccSchemaConfig {
    AwsccSchemaConfig {
        aws_type_name: "AWS::EC2::RouteTable",
        resource_type_name: "ec2.RouteTable",
        primary_identifier: &["RouteTableId"],
        has_tags: true,
        schema: ResourceSchema::new("ec2.RouteTable")
	        .with_description("Specifies a route table for the specified VPC. After you create a route table, you can add routes and associate the table with a subnet.  For more information, see [Route tables](https://docs.aws.amazon.com/vpc/latest/userguide/VPC_Route_Tables.html) in the *Amazon VPC User Guide*.")
        .attribute(
            AttributeSchema::new("route_table_id", carina_aws_types::route_table_id())
                .read_only()
                .with_description(" (read-only)")
                .with_provider_name("RouteTableId"),
        )
        .attribute(
            AttributeSchema::new("tags", carina_aws_types::tags_type())
                .with_description("Any tags assigned to the route table.")
                .with_provider_name("Tags")
                .with_block_name("tag"),
        )
        .attribute(
            AttributeSchema::new("vpc_id", carina_aws_types::vpc_id())
                .required()
                .create_only()
                .with_description("The ID of the VPC.")
                .with_provider_name("VpcId"),
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
    ("ec2.RouteTable", &[])
}

/// Returns the IAM permissions declared by the CloudFormation handler for this operation.
pub fn required_permissions(op: carina_core::effect::PlanOp) -> &'static [&'static str] {
    match op {
        carina_core::effect::PlanOp::Create => &[
            "ec2:CreateRouteTable",
            "ec2:CreateTags",
            "ec2:DescribeRouteTables",
        ],
        carina_core::effect::PlanOp::Read => &["ec2:DescribeRouteTables"],
        carina_core::effect::PlanOp::Update => &[
            "ec2:CreateTags",
            "ec2:DeleteTags",
            "ec2:DescribeRouteTables",
        ],
        carina_core::effect::PlanOp::Delete => &["ec2:DescribeRouteTables", "ec2:DeleteRouteTable"],
    }
}
