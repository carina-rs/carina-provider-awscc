//! vpn_gateway schema definition for AWS Cloud Control
//!
//! Auto-generated from CloudFormation schema: AWS::EC2::VPNGateway
//!
//! DO NOT EDIT MANUALLY - regenerate with carina-codegen

use crate::schemas::config::AwsccSchemaConfig;
use carina_core::resource::{ConcreteValue, Value};
use carina_core::schema::{AttributeSchema, AttributeType, ResourceSchema};

const VALID_TYPE: &[&str] = &["ipsec.1"];

#[allow(dead_code)]
fn validate_amazon_side_asn_range(value: &Value) -> Result<(), String> {
    if let Value::Concrete(ConcreteValue::Int(n)) = value {
        if *n < 1 || *n > 4294967294 {
            Err(format!("Value {} is out of range 1..=4294967294", n))
        } else {
            Ok(())
        }
    } else {
        Err("Expected integer".to_string())
    }
}

/// Returns the schema config for ec2_vpn_gateway (AWS::EC2::VPNGateway)
pub fn ec2_vpn_gateway_config() -> AwsccSchemaConfig {
    AwsccSchemaConfig {
        aws_type_name: "AWS::EC2::VPNGateway",
        resource_type_name: "ec2.VpnGateway",
        primary_identifier: &["VPNGatewayId"],
        has_tags: true,
        schema: ResourceSchema::new("ec2.VpnGateway")
	        .with_description("Specifies a virtual private gateway. A virtual private gateway is the endpoint on the VPC side of your VPN connection. You can create a virtual private gateway before creating the VPC itself.  For more information, see [](https://docs.aws.amazon.com/vpn/latest/s2svpn/VPC_VPN.html) in the *User Guide*.")
        .attribute(
            AttributeSchema::new("amazon_side_asn", AttributeType::refined_int(None, Some((Some(1), Some(4294967294)))))
                .create_only()
                .with_description("The private Autonomous System Number (ASN) for the Amazon side of a BGP session.")
                .with_provider_name("AmazonSideAsn"),
        )
        .attribute(
            AttributeSchema::new("tags", carina_aws_types::tags_type())
                .with_description("Any tags assigned to the virtual private gateway.")
                .with_provider_name("Tags")
                .with_block_name("tag"),
        )
        .attribute(
            AttributeSchema::new("type", AttributeType::enum_(carina_core::schema::enum_identity("Type", Some("aws.ec2.VpnGateway")), Some(vec!["ipsec.1".to_string()]), vec![("ipsec.1".to_string(), "ipsec_1".to_string())], None, None))
                .required()
                .create_only()
                .with_description("The type of VPN connection the virtual private gateway supports.")
                .with_provider_name("Type"),
        )
        .attribute(
            AttributeSchema::new("vpn_gateway_id", carina_aws_types::vpn_gateway_id())
                .read_only()
                .with_description(" (read-only)")
                .with_provider_name("VPNGatewayId"),
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
