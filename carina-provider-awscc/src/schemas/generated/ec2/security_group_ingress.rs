//! security_group_ingress schema definition for AWS Cloud Control
//!
//! Auto-generated from CloudFormation schema: AWS::EC2::SecurityGroupIngress
//!
//! DO NOT EDIT MANUALLY - regenerate with carina-codegen

use crate::schemas::config::AwsccSchemaConfig;
use carina_core::resource::{ConcreteValue, Value};
use carina_core::schema::{AttributeSchema, AttributeType, ResourceSchema, types};

const VALID_IP_PROTOCOL: &[&str] = &["tcp", "udp", "icmp", "icmpv6", "-1"];

#[allow(dead_code)]
fn validate_from_port_range(value: &Value) -> Result<(), String> {
    if let Value::Concrete(ConcreteValue::Int(n)) = value {
        if *n < -1 || *n > 65535 {
            Err(format!("Value {} is out of range -1..=65535", n))
        } else {
            Ok(())
        }
    } else {
        Err("Expected integer".to_string())
    }
}

#[allow(dead_code)]
fn validate_to_port_range(value: &Value) -> Result<(), String> {
    if let Value::Concrete(ConcreteValue::Int(n)) = value {
        if *n < -1 || *n > 65535 {
            Err(format!("Value {} is out of range -1..=65535", n))
        } else {
            Ok(())
        }
    } else {
        Err("Expected integer".to_string())
    }
}

/// Returns the schema config for ec2_security_group_ingress (AWS::EC2::SecurityGroupIngress)
pub fn ec2_security_group_ingress_config() -> AwsccSchemaConfig {
    AwsccSchemaConfig {
        aws_type_name: "AWS::EC2::SecurityGroupIngress",
        resource_type_name: "ec2.SecurityGroupIngress",
        primary_identifier: &["Id"],
        has_tags: false,
        schema: ResourceSchema::new("ec2.SecurityGroupIngress")
        .with_description("Resource Type definition for AWS::EC2::SecurityGroupIngress")
        .attribute(
            AttributeSchema::new("cidr_ip", types::ipv4_cidr())
                .create_only()
                .with_description("The IPv4 ranges")
                .with_provider_name("CidrIp"),
        )
        .attribute(
            AttributeSchema::new("cidr_ipv6", types::ipv6_cidr())
                .create_only()
                .with_description("[VPC only] The IPv6 ranges")
                .with_provider_name("CidrIpv6"),
        )
        .attribute(
            AttributeSchema::new("description", AttributeType::string())
                .with_description("Updates the description of an ingress (inbound) security group rule. You can replace an existing description, or add a description to a rule that did not have one previously")
                .with_provider_name("Description"),
        )
        .attribute(
            AttributeSchema::new("from_port", AttributeType::refined_int(None, Some((Some(-1), Some(65535)))))
                .create_only()
                .with_description("The start of port range for the TCP and UDP protocols, or an ICMP/ICMPv6 type number. A value of -1 indicates all ICMP/ICMPv6 types. If you specify all ICMP/ICMPv6 types, you must specify all codes. Use this for ICMP and any protocol that uses ports.")
                .with_provider_name("FromPort"),
        )
        .attribute(
            AttributeSchema::new("group_id", carina_aws_types::security_group_id())
                .create_only()
                .with_description("The ID of the security group. You must specify either the security group ID or the security group name in the request. For security groups in a nondefault VPC, you must specify the security group ID. You must specify the GroupName property or the GroupId property. For security groups that are in a VPC, you must use the GroupId property.")
                .with_provider_name("GroupId"),
        )
        .attribute(
            AttributeSchema::new("group_name", AttributeType::string())
                .create_only()
                .with_description("The name of the security group.")
                .with_provider_name("GroupName"),
        )
        .attribute(
            AttributeSchema::new("id", AttributeType::string())
                .read_only()
                .with_description("The Security Group Rule Id (read-only)")
                .with_provider_name("Id"),
        )
        .attribute(
            AttributeSchema::new("ip_protocol", AttributeType::enum_(carina_core::schema::enum_identity("IpProtocol", Some("aws.ec2.SecurityGroupIngress")), Some(vec!["tcp".to_string(), "udp".to_string(), "icmp".to_string(), "icmpv6".to_string(), "-1".to_string()]), vec![("tcp".to_string(), "tcp".to_string()), ("udp".to_string(), "udp".to_string()), ("icmp".to_string(), "icmp".to_string()), ("icmpv6".to_string(), "icmpv6".to_string()), ("-1".to_string(), "all".to_string())], None, None))
                .required()
                .create_only()
                .with_description("The IP protocol name (tcp, udp, icmp, icmpv6) or number (see Protocol Numbers). [VPC only] Use -1 to specify all protocols. When authorizing security group rules, specifying -1 or a protocol number other than tcp, udp, icmp, or icmpv6 allows traffic on all ports, regardless of any port range you specify. For tcp, udp, and icmp, you must specify a port range. For icmpv6, the port range is optional; if you omit the port range, traffic for all types and codes is allowed.")
                .with_provider_name("IpProtocol"),
        )
        .attribute(
            AttributeSchema::new("source_prefix_list_id", carina_aws_types::prefix_list_id())
                .create_only()
                .with_description("[EC2-VPC only] The ID of a prefix list. ")
                .with_provider_name("SourcePrefixListId"),
        )
        .attribute(
            AttributeSchema::new("source_security_group_id", carina_aws_types::security_group_id())
                .create_only()
                .with_description("The ID of the security group. You must specify either the security group ID or the security group name. For security groups in a nondefault VPC, you must specify the security group ID.")
                .with_provider_name("SourceSecurityGroupId"),
        )
        .attribute(
            AttributeSchema::new("source_security_group_name", AttributeType::string())
                .create_only()
                .with_description("[EC2-Classic, default VPC] The name of the source security group. You must specify the GroupName property or the GroupId property. For security groups that are in a VPC, you must use the GroupId property.")
                .with_provider_name("SourceSecurityGroupName"),
        )
        .attribute(
            AttributeSchema::new("source_security_group_owner_id", carina_aws_types::aws_account_id())
                .create_only()
                .with_description("[nondefault VPC] The AWS account ID that owns the source security group. You can't specify this property with an IP address range. If you specify SourceSecurityGroupName or SourceSecurityGroupId and that security group is owned by a different account than the account creating the stack, you must specify the SourceSecurityGroupOwnerId; otherwise, this property is optional.")
                .with_provider_name("SourceSecurityGroupOwnerId"),
        )
        .attribute(
            AttributeSchema::new("to_port", AttributeType::refined_int(None, Some((Some(-1), Some(65535)))))
                .create_only()
                .with_description("The end of port range for the TCP and UDP protocols, or an ICMP/ICMPv6 code. A value of -1 indicates all ICMP/ICMPv6 codes for the specified ICMP type. If you specify all ICMP/ICMPv6 types, you must specify all codes. Use this for ICMP and any protocol that uses ports.")
                .with_provider_name("ToPort"),
        )
    }
}

/// Returns the resource type name and all enum valid values for this module
pub fn enum_valid_values() -> (
    &'static str,
    &'static [(&'static str, &'static [&'static str])],
) {
    (
        "ec2.SecurityGroupIngress",
        &[("ip_protocol", VALID_IP_PROTOCOL)],
    )
}

/// Returns the IAM permissions declared by the CloudFormation handler for this operation.
pub fn required_permissions(op: carina_core::effect::PlanOp) -> &'static [&'static str] {
    match op {
        carina_core::effect::PlanOp::Create => &[
            "ec2:DescribeSecurityGroupRules",
            "ec2:AuthorizeSecurityGroupIngress",
        ],
        carina_core::effect::PlanOp::Read => &[
            "ec2:DescribeSecurityGroups",
            "ec2:DescribeSecurityGroupRules",
        ],
        carina_core::effect::PlanOp::Update => &["ec2:UpdateSecurityGroupRuleDescriptionsIngress"],
        carina_core::effect::PlanOp::Delete => &[
            "ec2:DescribeSecurityGroupRules",
            "ec2:RevokeSecurityGroupIngress",
        ],
    }
}
