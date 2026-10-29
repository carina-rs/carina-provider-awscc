//! vpc_peering_connection schema definition for AWS Cloud Control
//!
//! Auto-generated from CloudFormation schema: AWS::EC2::VPCPeeringConnection
//!
//! DO NOT EDIT MANUALLY - regenerate with carina-codegen

use crate::schemas::config::{AwsccSchemaConfig, TagsProperty, TagsWire};
use carina_core::schema::{AttributeSchema, ResourceSchema};

/// Returns the schema config for ec2_vpc_peering_connection (AWS::EC2::VPCPeeringConnection)
#[rustfmt::skip]
pub fn ec2_vpc_peering_connection_config() -> AwsccSchemaConfig {
    let mut schema = ResourceSchema::new("ec2.VpcPeeringConnection")
        .with_description("Resource Type definition for AWS::EC2::VPCPeeringConnection");
    #[inline(never)]
    fn build_assume_role_region_attribute() -> AttributeSchema {
        AttributeSchema::optional("assume_role_region", carina_aws_types::aws_region())
                .create_only()
                .write_only()
                .with_description("The Region code to use when calling Security Token Service (STS) to assume the PeerRoleArn, if provided.")
                .with_provider_name("AssumeRoleRegion")
            .build()
    }
    schema = schema.attribute(build_assume_role_region_attribute());
    #[inline(never)]
    fn build_id_attribute() -> AttributeSchema {
        AttributeSchema::provider_populated("id", carina_aws_types::vpc_peering_connection_id())
                .with_provider_name("Id")
            .build()
    }
    schema = schema.attribute(build_id_attribute());
    #[inline(never)]
    fn build_peer_owner_id_attribute() -> AttributeSchema {
        AttributeSchema::optional("peer_owner_id", carina_aws_types::aws_account_id())
                .create_only()
                .with_description("The AWS account ID of the owner of the accepter VPC.")
                .with_provider_name("PeerOwnerId")
            .build()
    }
    schema = schema.attribute(build_peer_owner_id_attribute());
    #[inline(never)]
    fn build_peer_region_attribute() -> AttributeSchema {
        AttributeSchema::optional("peer_region", carina_aws_types::aws_region())
                .create_only()
                .with_description("The Region code for the accepter VPC, if the accepter VPC is located in a Region other than the Region in which you make the request.")
                .with_provider_name("PeerRegion")
            .build()
    }
    schema = schema.attribute(build_peer_region_attribute());
    #[inline(never)]
    fn build_peer_role_arn_attribute() -> AttributeSchema {
        AttributeSchema::optional("peer_role_arn", super::super::iam::role::arn())
                .create_only()
                .write_only()
                .with_description("The Amazon Resource Name (ARN) of the VPC peer role for the peering connection in another AWS account.")
                .with_provider_name("PeerRoleArn")
            .build()
    }
    schema = schema.attribute(build_peer_role_arn_attribute());
    #[inline(never)]
    fn build_peer_vpc_id_attribute() -> AttributeSchema {
        AttributeSchema::required("peer_vpc_id", carina_aws_types::vpc_id())
                .create_only()
                .with_description("The ID of the VPC with which you are creating the VPC peering connection. You must specify this parameter in the request.")
                .with_provider_name("PeerVpcId")
            .build()
    }
    schema = schema.attribute(build_peer_vpc_id_attribute());
    #[inline(never)]
    fn build_tags_attribute() -> AttributeSchema {
        AttributeSchema::optional("tags", carina_aws_types::tags_type())
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
        .with_conflicting_replacement();
    schema = schema.with_declared_validators(vec![carina_provider_protocol::types::ValidatorType::TagsKeyValueCheck]);
    AwsccSchemaConfig {
        aws_type_name: "AWS::EC2::VPCPeeringConnection",
        resource_type_name: "ec2.VpcPeeringConnection",
        primary_identifier: &[crate::schemas::config::PrimaryIdentifierAttribute { provider_name: "Id", dsl_name: "id" }],
        tags: Some(TagsProperty { provider_name: "Tags", wire: TagsWire::KeyValueList }),
        schema,
    }
}

/// Returns the resource type name and all enum valid values for this module
pub fn enum_valid_values() -> (
    &'static str,
    &'static [(&'static str, &'static [&'static str])],
) {
    ("ec2.VpcPeeringConnection", &[])
}

/// Returns the IAM permissions declared by the CloudFormation handler for this operation.
pub fn required_permissions(op: carina_core::effect::PlanOp) -> &'static [&'static str] {
    match op {
        carina_core::effect::PlanOp::Create => &[
            "ec2:CreateVpcPeeringConnection",
            "ec2:DescribeVpcPeeringConnections",
            "ec2:AcceptVpcPeeringConnection",
            "ec2:CreateTags",
            "sts:AssumeRole",
        ],
        carina_core::effect::PlanOp::Read => &["ec2:DescribeVpcPeeringConnections"],
        carina_core::effect::PlanOp::Update => &[
            "ec2:CreateTags",
            "ec2:DeleteTags",
            "ec2:DescribeVpcPeeringConnections",
        ],
        carina_core::effect::PlanOp::Delete => &[
            "ec2:DeleteVpcPeeringConnection",
            "ec2:DescribeVpcPeeringConnections",
        ],
    }
}
