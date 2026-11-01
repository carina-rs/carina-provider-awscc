//! vpc_gateway_attachment schema definition for AWS Cloud Control
//!
//! Auto-generated from CloudFormation schema: AWS::EC2::VPCGatewayAttachment
//!
//! DO NOT EDIT MANUALLY - regenerate with carina-codegen

use crate::schemas::config::AwsccSchemaConfig;
use carina_core::schema::{
    AttributeSchema, AttributeType, FieldGroup, OperationConfig, ResourceSchema,
};

/// Returns the schema config for ec2_vpc_gateway_attachment (AWS::EC2::VPCGatewayAttachment)
#[rustfmt::skip]
pub fn ec2_vpc_gateway_attachment_config() -> AwsccSchemaConfig {
    let mut schema = ResourceSchema::new("ec2.VpcGatewayAttachment")
        .with_description("Resource Type definition for AWS::EC2::VPCGatewayAttachment");
    #[inline(never)]
    fn build_attachment_type_attribute() -> AttributeSchema {
        AttributeSchema::provider_populated("attachment_type", AttributeType::string())
                .with_description("Used to identify if this resource is an Internet Gateway or Vpn Gateway Attachment  (read-only)")
                .with_provider_name("AttachmentType")
            .build()
    }
    schema = schema.attribute(build_attachment_type_attribute());
    #[inline(never)]
    fn build_internet_gateway_id_attribute() -> AttributeSchema {
        AttributeSchema::optional("internet_gateway_id", carina_aws_types::internet_gateway_id())
                .with_description("The ID of the internet gateway. You must specify either InternetGatewayId or VpnGatewayId, but not both.")
                .with_provider_name("InternetGatewayId")
            .build()
    }
    schema = schema.attribute(build_internet_gateway_id_attribute());
    #[inline(never)]
    fn build_vpc_id_attribute() -> AttributeSchema {
        AttributeSchema::required("vpc_id", carina_aws_types::vpc_id())
                .create_only()
                .with_description("The ID of the VPC.")
                .with_provider_name("VpcId")
            .build()
    }
    schema = schema.attribute(build_vpc_id_attribute());
    #[inline(never)]
    fn build_vpn_gateway_id_attribute() -> AttributeSchema {
        AttributeSchema::optional("vpn_gateway_id", carina_aws_types::vpn_gateway_id())
                .with_description("The ID of the virtual private gateway. You must specify either InternetGatewayId or VpnGatewayId, but not both.")
                .with_provider_name("VpnGatewayId")
            .build()
    }
    schema = schema.attribute(build_vpn_gateway_id_attribute());
    schema = schema
        .with_conflicting_replacement();
    schema = schema.with_operation_config(OperationConfig {
            delete_timeout_secs: Some(1800),
            delete_max_retries: None,
            create_timeout_secs: None,
            create_max_retries: None,
            update_timeout_secs: None,
    });
    schema = schema.field_group(FieldGroup::exactly_one(["internet_gateway_id", "vpn_gateway_id"]).expect("generated exactly-one field group must be valid"));
    AwsccSchemaConfig {
        aws_type_name: "AWS::EC2::VPCGatewayAttachment",
        resource_type_name: "ec2.VpcGatewayAttachment",
        primary_identifier: &[crate::schemas::config::PrimaryIdentifierAttribute { provider_name: "AttachmentType", dsl_name: "attachment_type" }, crate::schemas::config::PrimaryIdentifierAttribute { provider_name: "VpcId", dsl_name: "vpc_id" }],
        exclusive_groups: &[],
        tags: None,
        schema,
    }
}

/// Returns the resource type name and all enum valid values for this module
pub fn enum_valid_values() -> (
    &'static str,
    &'static [(&'static str, &'static [&'static str])],
) {
    ("ec2.VpcGatewayAttachment", &[])
}

/// Returns the IAM permissions declared by the CloudFormation handler for this operation.
pub fn required_permissions(op: carina_core::effect::PlanOp) -> &'static [&'static str] {
    match op {
        carina_core::effect::PlanOp::Create => &[
            "ec2:AttachInternetGateway",
            "ec2:AttachVpnGateway",
            "ec2:DescribeInternetGateways",
            "ec2:DescribeVpnGateways",
        ],
        carina_core::effect::PlanOp::Read => {
            &["ec2:DescribeInternetGateways", "ec2:DescribeVpnGateways"]
        }
        carina_core::effect::PlanOp::Update => &[
            "ec2:AttachInternetGateway",
            "ec2:AttachVpnGateway",
            "ec2:DetachInternetGateway",
            "ec2:DetachVpnGateway",
            "ec2:DescribeInternetGateways",
            "ec2:DescribeVpnGateways",
        ],
        carina_core::effect::PlanOp::Delete => &[
            "ec2:DetachInternetGateway",
            "ec2:DetachVpnGateway",
            "ec2:DescribeInternetGateways",
            "ec2:DescribeVpnGateways",
        ],
    }
}
