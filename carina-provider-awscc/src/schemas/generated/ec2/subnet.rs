//! subnet schema definition for AWS Cloud Control
//!
//! Auto-generated from CloudFormation schema: AWS::EC2::Subnet
//!
//! DO NOT EDIT MANUALLY - regenerate with carina-codegen

use crate::schemas::config::{AwsccSchemaConfig, TagsProperty, TagsWire};
use carina_core::resource::{ConcreteValue, Value};
use carina_core::schema::{AttributeSchema, AttributeType, ResourceSchema, StructField, types};

#[allow(dead_code)]
fn validate_ipv4_netmask_length_range(value: &Value) -> Result<(), String> {
    if let Value::Concrete(ConcreteValue::Int(n)) = value {
        if *n < 0 || *n > 32 {
            Err(format!("Value {} is out of range 0..=32", n))
        } else {
            Ok(())
        }
    } else {
        Err("Expected integer".to_string())
    }
}

#[allow(dead_code)]
fn validate_ipv6_netmask_length_range(value: &Value) -> Result<(), String> {
    if let Value::Concrete(ConcreteValue::Int(n)) = value {
        if *n < 0 || *n > 128 {
            Err(format!("Value {} is out of range 0..=128", n))
        } else {
            Ok(())
        }
    } else {
        Err("Expected integer".to_string())
    }
}

/// Returns the schema config for ec2_subnet (AWS::EC2::Subnet)
#[rustfmt::skip]
pub fn ec2_subnet_config() -> AwsccSchemaConfig {
    let mut schema = ResourceSchema::new("ec2.Subnet")
        .with_description("Specifies a subnet for the specified VPC.  For an IPv4 only subnet, specify an IPv4 CIDR block. If the VPC has an IPv6 CIDR block, you can create an IPv6 only subnet or a dual stack subnet instead. For an IPv6 only subnet, specify an IPv6 CIDR block. For a dual stack subnet, specify both an IPv4 CIDR block and an IPv6 CIDR block.  For more information, see [Subnets for your VPC](https://docs.aws.amazon.com/vpc/latest/userguide/configure-subnets.html) in the *Amazon VPC User Guide*.");
    #[inline(never)]
    fn build_assign_ipv6_address_on_creation_attribute() -> AttributeSchema {
        AttributeSchema::optional("assign_ipv6_address_on_creation", AttributeType::bool())
                .with_description("Indicates whether a network interface created in this subnet receives an IPv6 address. The default value is ``false``. If you specify ``AssignIpv6AddressOnCreation``, you must also specify an IPv6 CIDR block.")
                .with_provider_name("AssignIpv6AddressOnCreation")
            .build()
    }
    schema = schema.attribute(build_assign_ipv6_address_on_creation_attribute());
    #[inline(never)]
    fn build_availability_zone_attribute() -> AttributeSchema {
        AttributeSchema::optional("availability_zone", carina_aws_types::availability_zone())
                .create_only()
                .with_description("The Availability Zone of the subnet. If you update this property, you must also update the ``CidrBlock`` property.")
                .with_provider_name("AvailabilityZone")
            .build()
    }
    schema = schema.attribute(build_availability_zone_attribute());
    #[inline(never)]
    fn build_availability_zone_id_attribute() -> AttributeSchema {
        AttributeSchema::optional("availability_zone_id", carina_aws_types::availability_zone_id())
                .create_only()
                .with_description("The AZ ID of the subnet.")
                .with_provider_name("AvailabilityZoneId")
            .build()
    }
    schema = schema.attribute(build_availability_zone_id_attribute());
    #[inline(never)]
    fn build_block_public_access_states_attribute() -> AttributeSchema {
        AttributeSchema::provider_populated("block_public_access_states", AttributeType::struct_("BlockPublicAccessStates".to_string(), vec![StructField::optional("internet_gateway_block_mode", AttributeType::enum_(carina_core::schema::enum_identity("InternetGatewayBlockMode", Some("aws.ec2.Subnet.BlockPublicAccessStates")), Some(vec!["off".to_string(), "block-bidirectional".to_string(), "block-ingress".to_string()]), vec![("off".to_string(), "off".to_string()), ("block-bidirectional".to_string(), "block_bidirectional".to_string()), ("block-ingress".to_string(), "block_ingress".to_string())], None, None)).with_description("The mode of VPC BPA. Options here are off, block-bidirectional, block-ingress ").with_provider_name("InternetGatewayBlockMode").build()]))
                .with_description(" (read-only)")
                .with_provider_name("BlockPublicAccessStates")
            .build()
    }
    schema = schema.attribute(build_block_public_access_states_attribute());
    #[inline(never)]
    fn build_cidr_block_attribute() -> AttributeSchema {
        AttributeSchema::optional("cidr_block", types::ipv4_cidr())
                .create_only()
                .with_description("The IPv4 CIDR block assigned to the subnet. If you update this property, we create a new subnet, and then delete the existing one.")
                .with_provider_name("CidrBlock")
            .build()
    }
    schema = schema.attribute(build_cidr_block_attribute());
    #[inline(never)]
    fn build_enable_dns64_attribute() -> AttributeSchema {
        AttributeSchema::optional("enable_dns64", AttributeType::bool())
                .with_description("Indicates whether DNS queries made to the Amazon-provided DNS Resolver in this subnet should return synthetic IPv6 addresses for IPv4-only destinations. You must first configure a NAT gateway in a public subnet (separate from the subnet containing the IPv6-only workloads). For example, the subnet containing the NAT gateway should have a ``0.0.0.0/0`` route pointing to the internet gateway. For more information, see [Configure DNS64 and NAT64](https://docs.aws.amazon.com/vpc/latest/userguide/nat-gateway-nat64-dns64.html#nat-gateway-nat64-dns64-walkthrough) in the *User Guide*.")
                .with_provider_name("EnableDns64")
            .build()
    }
    schema = schema.attribute(build_enable_dns64_attribute());
    #[inline(never)]
    fn build_enable_lni_at_device_index_attribute() -> AttributeSchema {
        AttributeSchema::optional("enable_lni_at_device_index", AttributeType::int())
                .write_only()
                .with_description("Indicates the device position for local network interfaces in this subnet. For example, ``1`` indicates local network interfaces in this subnet are the secondary network interface (eth1).")
                .with_provider_name("EnableLniAtDeviceIndex")
            .build()
    }
    schema = schema.attribute(build_enable_lni_at_device_index_attribute());
    #[inline(never)]
    fn build_ipv4_ipam_pool_id_attribute() -> AttributeSchema {
        AttributeSchema::optional("ipv4_ipam_pool_id", carina_aws_types::ipam_pool_id())
                .create_only()
                .write_only()
                .with_description("An IPv4 IPAM pool ID for the subnet.")
                .with_provider_name("Ipv4IpamPoolId")
            .build()
    }
    schema = schema.attribute(build_ipv4_ipam_pool_id_attribute());
    #[inline(never)]
    fn build_ipv4_netmask_length_attribute() -> AttributeSchema {
        AttributeSchema::optional("ipv4_netmask_length", AttributeType::refined_int(None, Some((Some(0), Some(32)))))
                .create_only()
                .write_only()
                .with_description("An IPv4 netmask length for the subnet.")
                .with_provider_name("Ipv4NetmaskLength")
            .build()
    }
    schema = schema.attribute(build_ipv4_netmask_length_attribute());
    #[inline(never)]
    fn build_ipv6_cidr_block_attribute() -> AttributeSchema {
        AttributeSchema::optional("ipv6_cidr_block", types::ipv6_cidr())
                .with_description("The IPv6 CIDR block. If you specify ``AssignIpv6AddressOnCreation``, you must also specify an IPv6 CIDR block.")
                .with_provider_name("Ipv6CidrBlock")
            .build()
    }
    schema = schema.attribute(build_ipv6_cidr_block_attribute());
    #[inline(never)]
    fn build_ipv6_cidr_blocks_attribute() -> AttributeSchema {
        AttributeSchema::provider_populated("ipv6_cidr_blocks", AttributeType::list(types::ipv6_cidr()))
                .with_description(" (read-only)")
                .with_provider_name("Ipv6CidrBlocks")
            .build()
    }
    schema = schema.attribute(build_ipv6_cidr_blocks_attribute());
    #[inline(never)]
    fn build_ipv6_ipam_pool_id_attribute() -> AttributeSchema {
        AttributeSchema::optional("ipv6_ipam_pool_id", carina_aws_types::ipam_pool_id())
                .create_only()
                .write_only()
                .with_description("An IPv6 IPAM pool ID for the subnet.")
                .with_provider_name("Ipv6IpamPoolId")
            .build()
    }
    schema = schema.attribute(build_ipv6_ipam_pool_id_attribute());
    #[inline(never)]
    fn build_ipv6_native_attribute() -> AttributeSchema {
        AttributeSchema::optional("ipv6_native", AttributeType::bool())
                .create_only()
                .with_description("Indicates whether this is an IPv6 only subnet. For more information, see [Subnet basics](https://docs.aws.amazon.com/vpc/latest/userguide/VPC_Subnets.html#subnet-basics) in the *User Guide*.")
                .with_provider_name("Ipv6Native")
            .build()
    }
    schema = schema.attribute(build_ipv6_native_attribute());
    #[inline(never)]
    fn build_ipv6_netmask_length_attribute() -> AttributeSchema {
        AttributeSchema::optional("ipv6_netmask_length", AttributeType::refined_int(None, Some((Some(0), Some(128)))))
                .create_only()
                .write_only()
                .with_description("An IPv6 netmask length for the subnet.")
                .with_provider_name("Ipv6NetmaskLength")
            .build()
    }
    schema = schema.attribute(build_ipv6_netmask_length_attribute());
    #[inline(never)]
    fn build_map_public_ip_on_launch_attribute() -> AttributeSchema {
        AttributeSchema::optional("map_public_ip_on_launch", AttributeType::bool())
                .with_description("Indicates whether instances launched in this subnet receive a public IPv4 address. The default value is ``false``. AWS charges for all public IPv4 addresses, including public IPv4 addresses associated with running instances and Elastic IP addresses. For more information, see the *Public IPv4 Address* tab on the [VPC pricing page](https://docs.aws.amazon.com/vpc/pricing/).")
                .with_provider_name("MapPublicIpOnLaunch")
            .build()
    }
    schema = schema.attribute(build_map_public_ip_on_launch_attribute());
    #[inline(never)]
    fn build_network_acl_association_id_attribute() -> AttributeSchema {
        AttributeSchema::provider_populated("network_acl_association_id", AttributeType::string())
                .with_description(" (read-only)")
                .with_provider_name("NetworkAclAssociationId")
            .build()
    }
    schema = schema.attribute(build_network_acl_association_id_attribute());
    #[inline(never)]
    fn build_outpost_arn_attribute() -> AttributeSchema {
        AttributeSchema::optional("outpost_arn", carina_aws_types::arn())
                .create_only()
                .with_description("The Amazon Resource Name (ARN) of the Outpost.")
                .with_provider_name("OutpostArn")
            .build()
    }
    schema = schema.attribute(build_outpost_arn_attribute());
    #[inline(never)]
    fn build_private_dns_name_options_on_launch_attribute() -> AttributeSchema {
        AttributeSchema::optional("private_dns_name_options_on_launch", AttributeType::struct_("PrivateDnsNameOptionsOnLaunch".to_string(), vec![StructField::optional("enable_resource_name_dns_aaaa_record", AttributeType::bool()).with_provider_name("EnableResourceNameDnsAAAARecord").build(),
                    StructField::optional("enable_resource_name_dns_a_record", AttributeType::bool()).with_provider_name("EnableResourceNameDnsARecord").build(),
                    StructField::optional("hostname_type", AttributeType::enum_(carina_core::schema::enum_identity("HostnameType", Some("aws.ec2.Subnet.PrivateDnsNameOptionsOnLaunch")), Some(vec!["ip-name".to_string(), "resource-name".to_string()]), vec![("ip-name".to_string(), "ip_name".to_string()), ("resource-name".to_string(), "resource_name".to_string())], None, None)).with_provider_name("HostnameType").build()]))
                .with_description("The hostname type for EC2 instances launched into this subnet and how DNS A and AAAA record queries to the instances should be handled. For more information, see [Amazon EC2 instance hostname types](https://docs.aws.amazon.com/AWSEC2/latest/UserGuide/ec2-instance-naming.html) in the *User Guide*. Available options: + EnableResourceNameDnsAAAARecord (true | false) + EnableResourceNameDnsARecord (true | false) + HostnameType (ip-name | resource-name)")
                .with_provider_name("PrivateDnsNameOptionsOnLaunch")
            .build()
    }
    schema = schema.attribute(build_private_dns_name_options_on_launch_attribute());
    #[inline(never)]
    fn build_subnet_id_attribute() -> AttributeSchema {
        AttributeSchema::provider_populated("subnet_id", carina_aws_types::subnet_id())
                .with_description(" (read-only)")
                .with_provider_name("SubnetId")
            .build()
    }
    schema = schema.attribute(build_subnet_id_attribute());
    #[inline(never)]
    fn build_tags_attribute() -> AttributeSchema {
        AttributeSchema::optional("tags", carina_aws_types::tags_type())
                .with_description("Any tags assigned to the subnet.")
                .with_provider_name("Tags")
                .with_block_name("tag")
            .build()
    }
    schema = schema.attribute(build_tags_attribute());
    #[inline(never)]
    fn build_vpc_id_attribute() -> AttributeSchema {
        AttributeSchema::required("vpc_id", carina_aws_types::vpc_id())
                .create_only()
                .with_description("The ID of the VPC the subnet is in. If you update this property, you must also update the ``CidrBlock`` property.")
                .with_provider_name("VpcId")
            .build()
    }
    schema = schema.attribute(build_vpc_id_attribute());
    schema = schema
        .with_conflicting_replacement();
    schema = schema.with_declared_validators(vec![carina_provider_protocol::types::ValidatorType::TagsKeyValueCheck]);
    AwsccSchemaConfig {
        aws_type_name: "AWS::EC2::Subnet",
        resource_type_name: "ec2.Subnet",
        primary_identifier: &[crate::schemas::config::PrimaryIdentifierAttribute { provider_name: "SubnetId", dsl_name: "subnet_id" }],
        tags: Some(TagsProperty { provider_name: "Tags", wire: TagsWire::KeyValueList }),
        schema,
    }
}

/// Returns the resource type name and all enum valid values for this module
pub fn enum_valid_values() -> (
    &'static str,
    &'static [(&'static str, &'static [&'static str])],
) {
    ("ec2.Subnet", &[])
}

/// Returns the IAM permissions declared by the CloudFormation handler for this operation.
pub fn required_permissions(op: carina_core::effect::PlanOp) -> &'static [&'static str] {
    match op {
        carina_core::effect::PlanOp::Create => &[
            "ec2:DescribeSubnets",
            "ec2:CreateSubnet",
            "ec2:CreateTags",
            "ec2:ModifySubnetAttribute",
        ],
        carina_core::effect::PlanOp::Read => &["ec2:DescribeSubnets", "ec2:DescribeNetworkAcls"],
        carina_core::effect::PlanOp::Update => &[
            "ec2:DescribeSubnets",
            "ec2:ModifySubnetAttribute",
            "ec2:CreateTags",
            "ec2:DeleteTags",
            "ec2:AssociateSubnetCidrBlock",
            "ec2:DisassociateSubnetCidrBlock",
        ],
        carina_core::effect::PlanOp::Delete => &["ec2:DescribeSubnets", "ec2:DeleteSubnet"],
    }
}
