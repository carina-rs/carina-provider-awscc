//! egress_only_internet_gateway schema definition for AWS Cloud Control
//!
//! Auto-generated from CloudFormation schema: AWS::EC2::EgressOnlyInternetGateway
//!
//! DO NOT EDIT MANUALLY - regenerate with carina-codegen

use crate::schemas::config::{AwsccSchemaConfig, TagsProperty, TagsWire};
use carina_core::schema::{AttributeSchema, ResourceSchema};

/// Returns the schema config for ec2_egress_only_internet_gateway (AWS::EC2::EgressOnlyInternetGateway)
#[rustfmt::skip]
pub fn ec2_egress_only_internet_gateway_config() -> AwsccSchemaConfig {
    let mut schema = ResourceSchema::new("ec2.EgressOnlyInternetGateway")
        .with_description("Resource Type definition for AWS::EC2::EgressOnlyInternetGateway");
    #[inline(never)]
    fn build_id_attribute() -> AttributeSchema {
        AttributeSchema::provider_populated("id", carina_aws_types::egress_only_internet_gateway_id())
                .with_description("Service Generated ID of the EgressOnlyInternetGateway (read-only)")
                .with_provider_name("Id")
            .build()
    }
    schema = schema.attribute(build_id_attribute());
    #[inline(never)]
    fn build_tags_attribute() -> AttributeSchema {
        AttributeSchema::optional("tags", carina_aws_types::tags_type())
                .with_description("Any tags assigned to the egress only internet gateway.")
                .with_provider_name("Tags")
                .with_block_name("tag")
            .build()
    }
    schema = schema.attribute(build_tags_attribute());
    #[inline(never)]
    fn build_vpc_id_attribute() -> AttributeSchema {
        AttributeSchema::required("vpc_id", carina_aws_types::vpc_id())
                .create_only()
                .with_description("The ID of the VPC for which to create the egress-only internet gateway.")
                .with_provider_name("VpcId")
            .build()
    }
    schema = schema.attribute(build_vpc_id_attribute());
    schema = schema
        .with_coexisting_replacement();
    schema = schema.with_declared_validators(vec![carina_provider_protocol::types::ValidatorType::TagsKeyValueCheck]);
    AwsccSchemaConfig {
        aws_type_name: "AWS::EC2::EgressOnlyInternetGateway",
        resource_type_name: "ec2.EgressOnlyInternetGateway",
        primary_identifier: &[crate::schemas::config::PrimaryIdentifierAttribute { provider_name: "Id", dsl_name: "id" }],
        exclusive_groups: &[],
        tags: Some(TagsProperty { provider_name: "Tags", wire: TagsWire::KeyValueList }),
        schema,
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
