//! route_table schema definition for AWS Cloud Control
//!
//! Auto-generated from CloudFormation schema: AWS::EC2::RouteTable
//!
//! DO NOT EDIT MANUALLY - regenerate with carina-codegen

use crate::schemas::config::{AwsccSchemaConfig, TagsProperty, TagsWire};
use carina_core::schema::{AttributeSchema, ResourceSchema};

/// Returns the schema config for ec2_route_table (AWS::EC2::RouteTable)
#[rustfmt::skip]
pub fn ec2_route_table_config() -> AwsccSchemaConfig {
    let mut schema = ResourceSchema::new("ec2.RouteTable")
        .with_description("Specifies a route table for the specified VPC. After you create a route table, you can add routes and associate the table with a subnet.  For more information, see [Route tables](https://docs.aws.amazon.com/vpc/latest/userguide/VPC_Route_Tables.html) in the *Amazon VPC User Guide*.");
    #[inline(never)]
    fn build_route_table_id_attribute() -> AttributeSchema {
        AttributeSchema::provider_populated("route_table_id", carina_aws_types::route_table_id())
                .with_description(" (read-only)")
                .with_provider_name("RouteTableId")
            .build()
    }
    schema = schema.attribute(build_route_table_id_attribute());
    #[inline(never)]
    fn build_tags_attribute() -> AttributeSchema {
        AttributeSchema::optional("tags", carina_aws_types::tags_type())
                .with_description("Any tags assigned to the route table.")
                .with_provider_name("Tags")
                .with_block_name("tag")
            .build()
    }
    schema = schema.attribute(build_tags_attribute());
    #[inline(never)]
    fn build_vpc_id_attribute() -> AttributeSchema {
        AttributeSchema::required("vpc_id", carina_aws_types::vpc_id())
                .create_only()
                .with_description("The ID of the VPC.")
                .with_provider_name("VpcId")
            .build()
    }
    schema = schema.attribute(build_vpc_id_attribute());
    schema = schema
        .with_coexisting_replacement();
    schema = schema.with_declared_validators(vec![carina_provider_protocol::types::ValidatorType::TagsKeyValueCheck]);
    AwsccSchemaConfig {
        aws_type_name: "AWS::EC2::RouteTable",
        resource_type_name: "ec2.RouteTable",
        primary_identifier: &[crate::schemas::config::PrimaryIdentifierAttribute { provider_name: "RouteTableId", dsl_name: "route_table_id" }],
        tags: Some(TagsProperty { provider_name: "Tags", wire: TagsWire::KeyValueList }),
        schema,
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
