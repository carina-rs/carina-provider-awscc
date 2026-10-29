//! vpn_gateway schema definition for AWS Cloud Control
//!
//! Auto-generated from CloudFormation schema: AWS::EC2::VPNGateway
//!
//! DO NOT EDIT MANUALLY - regenerate with carina-codegen

use crate::schemas::config::{AwsccSchemaConfig, TagsProperty, TagsWire};
use carina_core::schema::{AttributeSchema, AttributeType, ResourceSchema};

const VALID_TYPE: &[&str] = &["ipsec.1"];

/// Returns the schema config for ec2_vpn_gateway (AWS::EC2::VPNGateway)
#[rustfmt::skip]
pub fn ec2_vpn_gateway_config() -> AwsccSchemaConfig {
    let mut schema = ResourceSchema::new("ec2.VpnGateway")
        .with_description("Specifies a virtual private gateway. A virtual private gateway is the endpoint on the VPC side of your VPN connection. You can create a virtual private gateway before creating the VPC itself.  For more information, see [](https://docs.aws.amazon.com/vpn/latest/s2svpn/VPC_VPN.html) in the *User Guide*.");
    #[inline(never)]
    fn build_amazon_side_asn_attribute() -> AttributeSchema {
        AttributeSchema::optional("amazon_side_asn", AttributeType::refined_int(None, Some((Some(1), Some(4294967294)))))
                .create_only()
                .with_description("The private Autonomous System Number (ASN) for the Amazon side of a BGP session.")
                .with_provider_name("AmazonSideAsn")
            .build()
    }
    schema = schema.attribute(build_amazon_side_asn_attribute());
    #[inline(never)]
    fn build_tags_attribute() -> AttributeSchema {
        AttributeSchema::optional("tags", carina_aws_types::tags_type())
                .with_description("Any tags assigned to the virtual private gateway.")
                .with_provider_name("Tags")
                .with_block_name("tag")
            .build()
    }
    schema = schema.attribute(build_tags_attribute());
    #[inline(never)]
    fn build_type_attribute() -> AttributeSchema {
        AttributeSchema::required("type", AttributeType::enum_(carina_core::schema::enum_identity("Type", Some("aws.ec2.VpnGateway")), Some(vec!["ipsec.1".to_string()]), vec![("ipsec.1".to_string(), "ipsec_1".to_string())], None, None))
                .create_only()
                .with_description("The type of VPN connection the virtual private gateway supports.")
                .with_provider_name("Type")
            .build()
    }
    schema = schema.attribute(build_type_attribute());
    #[inline(never)]
    fn build_vpn_gateway_id_attribute() -> AttributeSchema {
        AttributeSchema::provider_populated("vpn_gateway_id", carina_aws_types::vpn_gateway_id())
                .with_description(" (read-only)")
                .with_provider_name("VPNGatewayId")
            .build()
    }
    schema = schema.attribute(build_vpn_gateway_id_attribute());
    schema = schema
        .with_coexisting_replacement();
    schema = schema.with_declared_validators(vec![carina_provider_protocol::types::ValidatorType::TagsKeyValueCheck]);
    AwsccSchemaConfig {
        aws_type_name: "AWS::EC2::VPNGateway",
        resource_type_name: "ec2.VpnGateway",
        primary_identifier: &[crate::schemas::config::PrimaryIdentifierAttribute { provider_name: "VPNGatewayId", dsl_name: "vpn_gateway_id" }],
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
    ("ec2.VpnGateway", &[("type", VALID_TYPE)])
}

/// Returns the IAM permissions declared by the CloudFormation handler for this operation.
pub fn required_permissions(op: carina_core::effect::PlanOp) -> &'static [&'static str] {
    match op {
        carina_core::effect::PlanOp::Create => &[
            "ec2:CreateVpnGateway",
            "ec2:DescribeVpnGateways",
            "ec2:CreateTags",
        ],
        carina_core::effect::PlanOp::Read => &["ec2:DescribeVpnGateways"],
        carina_core::effect::PlanOp::Update => &[
            "ec2:DescribeVpnGateways",
            "ec2:CreateTags",
            "ec2:DeleteTags",
        ],
        carina_core::effect::PlanOp::Delete => &["ec2:DeleteVpnGateway", "ec2:DescribeVpnGateways"],
    }
}
