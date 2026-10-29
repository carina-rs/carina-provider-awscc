//! security_group schema definition for AWS Cloud Control
//!
//! Auto-generated from CloudFormation schema: AWS::EC2::SecurityGroup
//!
//! DO NOT EDIT MANUALLY - regenerate with carina-codegen

use crate::schemas::config::{AwsccSchemaConfig, TagsProperty, TagsWire};
use carina_core::resource::{ConcreteValue, Value};
use carina_core::schema::{AttributeSchema, AttributeType, ResourceSchema, StructField, types};

const VALID_EGRESS_IP_PROTOCOL: &[&str] = &["tcp", "udp", "icmp", "icmpv6", "-1"];

const VALID_INGRESS_IP_PROTOCOL: &[&str] = &["tcp", "udp", "icmp", "icmpv6", "-1"];

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

/// Returns the schema config for ec2_security_group (AWS::EC2::SecurityGroup)
pub fn ec2_security_group_config() -> AwsccSchemaConfig {
    AwsccSchemaConfig {
        aws_type_name: "AWS::EC2::SecurityGroup",
        resource_type_name: "ec2.SecurityGroup",
        primary_identifier: &[crate::schemas::config::PrimaryIdentifierAttribute { provider_name: "Id", dsl_name: "id" }],
        tags: Some(TagsProperty { provider_name: "Tags", wire: TagsWire::KeyValueList }),
        schema: ResourceSchema::new("ec2.SecurityGroup")
	        .with_description("Resource Type definition for AWS::EC2::SecurityGroup")
        .attribute(
            AttributeSchema::required("group_description", AttributeType::string())
                .create_only()
                .with_description("A description for the security group.")
                .with_provider_name("GroupDescription")
                .build(),
        )
        .attribute(
            AttributeSchema::provider_populated("group_id", carina_aws_types::security_group_id())
                .with_description("The group ID of the specified security group. (read-only)")
                .with_provider_name("GroupId")
                .build(),
        )
        .attribute(
            AttributeSchema::optional("group_name", AttributeType::string())
                .create_only()
                .with_description("The name of the security group.")
                .with_provider_name("GroupName")
                .build(),
        )
        .attribute(
            AttributeSchema::provider_populated("id", carina_aws_types::security_group_id())
                .with_description("The group name or group ID depending on whether the SG is created in default or specific VPC (read-only)")
                .with_provider_name("Id")
                .build(),
        )
        .attribute(
            AttributeSchema::optional("security_group_egress", AttributeType::unordered_list(AttributeType::struct_("Egress".to_string(), vec![StructField::optional("cidr_ip", types::ipv4_cidr()).with_provider_name("CidrIp").build(),
                    StructField::optional("cidr_ipv6", types::ipv6_cidr()).with_provider_name("CidrIpv6").build(),
                    StructField::optional("description", AttributeType::string()).with_provider_name("Description").build(),
                    StructField::optional("destination_prefix_list_id", carina_aws_types::prefix_list_id()).with_provider_name("DestinationPrefixListId").build(),
                    StructField::optional("destination_security_group_id", carina_aws_types::security_group_id()).with_provider_name("DestinationSecurityGroupId").build(),
                    StructField::optional("from_port", AttributeType::refined_int(None, Some((Some(-1), Some(65535))))).with_provider_name("FromPort").build(),
                    StructField::required("ip_protocol", AttributeType::enum_(carina_core::schema::enum_identity("IpProtocol", Some("aws.ec2.SecurityGroup.Egress")), Some(vec!["tcp".to_string(), "udp".to_string(), "icmp".to_string(), "icmpv6".to_string(), "-1".to_string()]), vec![("tcp".to_string(), "tcp".to_string()), ("udp".to_string(), "udp".to_string()), ("icmp".to_string(), "icmp".to_string()), ("icmpv6".to_string(), "icmpv6".to_string()), ("-1".to_string(), "all".to_string())], None, None)).with_provider_name("IpProtocol").build(),
                    StructField::optional("to_port", AttributeType::refined_int(None, Some((Some(-1), Some(65535))))).with_provider_name("ToPort").build()])))
                .with_description("[VPC only] The outbound rules associated with the security group. There is a short interruption during which you cannot connect to the security group.")
                .with_provider_name("SecurityGroupEgress")
                .with_block_name("security_group_egress")
                .build(),
        )
        .attribute(
            AttributeSchema::optional("security_group_ingress", AttributeType::unordered_list(AttributeType::struct_("Ingress".to_string(), vec![StructField::optional("cidr_ip", types::ipv4_cidr()).with_provider_name("CidrIp").build(),
                    StructField::optional("cidr_ipv6", types::ipv6_cidr()).with_provider_name("CidrIpv6").build(),
                    StructField::optional("description", AttributeType::string()).with_provider_name("Description").build(),
                    StructField::optional("from_port", AttributeType::refined_int(None, Some((Some(-1), Some(65535))))).with_provider_name("FromPort").build(),
                    StructField::required("ip_protocol", AttributeType::enum_(carina_core::schema::enum_identity("IpProtocol", Some("aws.ec2.SecurityGroup.Ingress")), Some(vec!["tcp".to_string(), "udp".to_string(), "icmp".to_string(), "icmpv6".to_string(), "-1".to_string()]), vec![("tcp".to_string(), "tcp".to_string()), ("udp".to_string(), "udp".to_string()), ("icmp".to_string(), "icmp".to_string()), ("icmpv6".to_string(), "icmpv6".to_string()), ("-1".to_string(), "all".to_string())], None, None)).with_provider_name("IpProtocol").build(),
                    StructField::optional("source_prefix_list_id", carina_aws_types::prefix_list_id()).with_provider_name("SourcePrefixListId").build(),
                    StructField::optional("source_security_group_id", carina_aws_types::security_group_id()).with_provider_name("SourceSecurityGroupId").build(),
                    StructField::optional("source_security_group_name", AttributeType::string()).write_only().with_provider_name("SourceSecurityGroupName").build(),
                    StructField::optional("source_security_group_owner_id", carina_aws_types::aws_account_id()).with_provider_name("SourceSecurityGroupOwnerId").build(),
                    StructField::optional("to_port", AttributeType::refined_int(None, Some((Some(-1), Some(65535))))).with_provider_name("ToPort").build()])))
                .with_description("The inbound rules associated with the security group. There is a short interruption during which you cannot connect to the security group.")
                .with_provider_name("SecurityGroupIngress")
                .with_block_name("security_group_ingress")
                .build(),
        )
        .attribute(
            AttributeSchema::optional("tags", carina_aws_types::tags_type())
                .with_description("Any tags assigned to the security group.")
                .with_provider_name("Tags")
                .with_block_name("tag")
                .build(),
        )
        .attribute(
            AttributeSchema::optional("vpc_id", carina_aws_types::vpc_id())
                .create_only()
                .with_description("The ID of the VPC for the security group.")
                .with_provider_name("VpcId")
                .build(),
        )
        .with_unique_name_attribute("group_name")
        .with_declared_validators(vec![carina_provider_protocol::types::ValidatorType::TagsKeyValueCheck])
    }
}

/// Returns the resource type name and all enum valid values for this module
pub fn enum_valid_values() -> (
    &'static str,
    &'static [(&'static str, &'static [&'static str])],
) {
    (
        "ec2.SecurityGroup",
        &[
            ("ip_protocol", VALID_EGRESS_IP_PROTOCOL),
            ("ip_protocol", VALID_INGRESS_IP_PROTOCOL),
        ],
    )
}

/// Returns the IAM permissions declared by the CloudFormation handler for this operation.
pub fn required_permissions(op: carina_core::effect::PlanOp) -> &'static [&'static str] {
    match op {
        carina_core::effect::PlanOp::Create => &[
            "ec2:CreateSecurityGroup",
            "ec2:DescribeSecurityGroups",
            "ec2:RevokeSecurityGroupEgress",
            "ec2:AuthorizeSecurityGroupEgress",
            "ec2:AuthorizeSecurityGroupIngress",
            "ec2:CreateTags",
        ],
        carina_core::effect::PlanOp::Read => &["ec2:DescribeSecurityGroups"],
        carina_core::effect::PlanOp::Update => &[
            "ec2:RevokeSecurityGroupEgress",
            "ec2:RevokeSecurityGroupIngress",
            "ec2:DescribeSecurityGroups",
            "ec2:AuthorizeSecurityGroupEgress",
            "ec2:AuthorizeSecurityGroupIngress",
            "ec2:UpdateSecurityGroupRuleDescriptionsIngress",
            "ec2:UpdateSecurityGroupRuleDescriptionsEgress",
            "ec2:CreateTags",
            "ec2:DeleteTags",
        ],
        carina_core::effect::PlanOp::Delete => &[
            "ec2:DescribeSecurityGroups",
            "ec2:DeleteSecurityGroup",
            "ec2:DescribeInstances",
        ],
    }
}
