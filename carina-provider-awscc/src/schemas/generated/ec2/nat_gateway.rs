//! nat_gateway schema definition for AWS Cloud Control
//!
//! Auto-generated from CloudFormation schema: AWS::EC2::NatGateway
//!
//! DO NOT EDIT MANUALLY - regenerate with carina-codegen

use crate::schemas::config::{AwsccSchemaConfig, TagsProperty, TagsWire};
use carina_core::schema::{
    AttributeSchema, AttributeType, OperationConfig, ResourceSchema, StructField, types,
};

const VALID_AVAILABILITY_MODE: &[&str] = &["zonal", "regional"];

const VALID_CONNECTIVITY_TYPE: &[&str] = &["public", "private"];

/// Returns the schema config for ec2_nat_gateway (AWS::EC2::NatGateway)
#[rustfmt::skip]
pub fn ec2_nat_gateway_config() -> AwsccSchemaConfig {
    let mut schema = ResourceSchema::new("ec2.NatGateway")
        .with_description("Specifies a network address translation (NAT) gateway in the specified subnet. You can create either a public NAT gateway or a private NAT gateway. The default is a public NAT gateway. If you create a public NAT gateway, you must specify an elastic IP address.  With a NAT gateway, instances in a private subnet can connect to the internet, other AWS services, or an on-premises network using the IP address of the NAT gateway. For more information, see [NAT gateways](https://docs.aws.amazon.com/vpc/latest/userguide/vpc-nat-gateway.html) in the *Amazon VPC User Guide*.  If you add a default route (``AWS::EC2::Route`` resource) that points to a NAT gateway, specify the NAT gateway ID for the route's ``NatGatewayId`` property.   When you associate an Elastic IP address or secondary Elastic IP address with a public NAT gateway, the network border group of the Elastic IP address must match the network border group of the Availability Zone (AZ) that the public NAT gateway is in. Otherwise, the NAT gateway fails to launch. You can see the network border group for the AZ by viewing the details of the subnet. Similarly, you can view the network border group for the Elastic IP address by viewing its details. For more information, see [Allocate an Elastic IP address](https://docs.aws.amazon.com/vpc/latest/userguide/vpc-eips.html#allocate-eip) in the *Amazon VPC User Guide*.");
    #[inline(never)]
    fn build_allocation_id_attribute() -> AttributeSchema {
        AttributeSchema::optional("allocation_id", carina_aws_types::allocation_id())
                .create_only()
                .with_description("[Public NAT gateway only] The allocation ID of the Elastic IP address that's associated with the NAT gateway. This property is required for a public NAT gateway and cannot be specified with a private NAT gateway.")
                .with_provider_name("AllocationId")
            .build()
    }
    schema = schema.attribute(build_allocation_id_attribute());
    #[inline(never)]
    fn build_auto_provision_zones_attribute() -> AttributeSchema {
        AttributeSchema::provider_populated("auto_provision_zones", AttributeType::string())
                .with_description(" (read-only)")
                .with_provider_name("AutoProvisionZones")
            .build()
    }
    schema = schema.attribute(build_auto_provision_zones_attribute());
    #[inline(never)]
    fn build_auto_scaling_ips_attribute() -> AttributeSchema {
        AttributeSchema::provider_populated("auto_scaling_ips", AttributeType::string())
                .with_description(" (read-only)")
                .with_provider_name("AutoScalingIps")
            .build()
    }
    schema = schema.attribute(build_auto_scaling_ips_attribute());
    #[inline(never)]
    fn build_availability_mode_attribute() -> AttributeSchema {
        AttributeSchema::optional("availability_mode", AttributeType::enum_(carina_core::schema::enum_identity("AvailabilityMode", Some("aws.ec2.NatGateway")), Some(vec!["zonal".to_string(), "regional".to_string()]), vec![("zonal".to_string(), "zonal".to_string()), ("regional".to_string(), "regional".to_string())], None, None))
                .create_only()
                .with_description("Indicates whether this is a zonal (single-AZ) or regional (multi-AZ) NAT gateway. A zonal NAT gateway is a NAT Gateway that provides redundancy and scalability within a single availability zone. A regional NAT gateway is a single NAT Gateway that works across multiple availability zones (AZs) in your VPC, providing redundancy, scalability and availability across all the AZs in a Region. For more information, see [Regional NAT gateways for automatic multi-AZ expansion](https://docs.aws.amazon.com/vpc/latest/userguide/nat-gateways-regional.html) in the *Amazon VPC User Guide*.")
                .with_provider_name("AvailabilityMode")
            .build()
    }
    schema = schema.attribute(build_availability_mode_attribute());
    #[inline(never)]
    fn build_availability_zone_addresses_attribute() -> AttributeSchema {
        AttributeSchema::optional("availability_zone_addresses", AttributeType::unordered_list(AttributeType::struct_("AvailabilityZoneAddress".to_string(), vec![StructField::required("allocation_ids", AttributeType::unordered_list(carina_aws_types::allocation_id())).with_description("The allocation IDs of the Elastic IP addresses (EIPs) to be used for handling outbound NAT traffic in this specific Availability Zone.").with_provider_name("AllocationIds").build(),
                    StructField::optional("availability_zone", carina_aws_types::availability_zone()).with_description("For regional NAT gateways only: The Availability Zone where this specific NAT gateway configuration will be active. Each AZ in a regional NAT gateway has its own configuration to handle outbound NAT traffic from that AZ. A regional NAT gateway is a single NAT Gateway that works across multiple availability zones (AZs) in your VPC, providing redundancy, scalability and availability across all the AZs in a Region.").with_provider_name("AvailabilityZone").build(),
                    StructField::optional("availability_zone_id", carina_aws_types::availability_zone_id()).with_description("For regional NAT gateways only: The ID of the Availability Zone where this specific NAT gateway configuration will be active. Each AZ in a regional NAT gateway has its own configuration to handle outbound NAT traffic from that AZ. Use this instead of AvailabilityZone for consistent identification of AZs across AWS Regions. A regional NAT gateway is a single NAT Gateway that works across multiple availability zones (AZs) in your VPC, providing redundancy, scalability and availability across all the AZs in a Region.").with_provider_name("AvailabilityZoneId").build()])))
                .with_description("For regional NAT gateways only: Specifies which Availability Zones you want the NAT gateway to support and the Elastic IP addresses (EIPs) to use in each AZ. The regional NAT gateway uses these EIPs to handle outbound NAT traffic from their respective AZs. If not specified, the NAT gateway will automatically expand to new AZs and associate EIPs upon detection of an elastic network interface. If you specify this parameter, auto-expansion is disabled and you must manually manage AZ coverage. A regional NAT gateway is a single NAT Gateway that works across multiple availability zones (AZs) in your VPC, providing redundancy, scalability and availability across all the AZs in a Region. For more information, see [Regional NAT gateways for automatic multi-AZ expansion](https://docs.aws.amazon.com/vpc/latest/userguide/nat-gateways-regional.html) in the *Amazon VPC User Guide*.")
                .with_provider_name("AvailabilityZoneAddresses")
                .with_block_name("availability_zone_address")
            .build()
    }
    schema = schema.attribute(build_availability_zone_addresses_attribute());
    #[inline(never)]
    fn build_connectivity_type_attribute() -> AttributeSchema {
        AttributeSchema::optional("connectivity_type", AttributeType::enum_(carina_core::schema::enum_identity("ConnectivityType", Some("aws.ec2.NatGateway")), Some(vec!["public".to_string(), "private".to_string()]), vec![("public".to_string(), "public".to_string()), ("private".to_string(), "private".to_string())], None, None))
                .create_only()
                .with_description("Indicates whether the NAT gateway supports public or private connectivity. The default is public connectivity.")
                .with_provider_name("ConnectivityType")
            .build()
    }
    schema = schema.attribute(build_connectivity_type_attribute());
    #[inline(never)]
    fn build_eni_id_attribute() -> AttributeSchema {
        AttributeSchema::provider_populated("eni_id", carina_aws_types::network_interface_id())
                .with_description(" (read-only)")
                .with_provider_name("EniId")
            .build()
    }
    schema = schema.attribute(build_eni_id_attribute());
    #[inline(never)]
    fn build_max_drain_duration_seconds_attribute() -> AttributeSchema {
        AttributeSchema::optional("max_drain_duration_seconds", AttributeType::int())
                .write_only()
                .with_description("The maximum amount of time to wait (in seconds) before forcibly releasing the IP addresses if connections are still in progress. Default value is 350 seconds.")
                .with_provider_name("MaxDrainDurationSeconds")
            .build()
    }
    schema = schema.attribute(build_max_drain_duration_seconds_attribute());
    #[inline(never)]
    fn build_nat_gateway_id_attribute() -> AttributeSchema {
        AttributeSchema::provider_populated("nat_gateway_id", carina_aws_types::nat_gateway_id())
                .with_description(" (read-only)")
                .with_provider_name("NatGatewayId")
            .build()
    }
    schema = schema.attribute(build_nat_gateway_id_attribute());
    #[inline(never)]
    fn build_private_ip_address_attribute() -> AttributeSchema {
        AttributeSchema::optional("private_ip_address", types::ipv4_address())
                .create_only()
                .with_description("The private IPv4 address to assign to the NAT gateway. If you don't provide an address, a private IPv4 address will be automatically assigned.")
                .with_provider_name("PrivateIpAddress")
            .build()
    }
    schema = schema.attribute(build_private_ip_address_attribute());
    #[inline(never)]
    fn build_route_table_id_attribute() -> AttributeSchema {
        AttributeSchema::provider_populated("route_table_id", carina_aws_types::route_table_id())
                .with_description(" (read-only)")
                .with_provider_name("RouteTableId")
            .build()
    }
    schema = schema.attribute(build_route_table_id_attribute());
    #[inline(never)]
    fn build_secondary_allocation_ids_attribute() -> AttributeSchema {
        AttributeSchema::optional("secondary_allocation_ids", AttributeType::list(carina_aws_types::allocation_id()))
                .with_description("Secondary EIP allocation IDs. For more information, see [Create a NAT gateway](https://docs.aws.amazon.com/vpc/latest/userguide/nat-gateway-working-with.html) in the *Amazon VPC User Guide*.")
                .with_provider_name("SecondaryAllocationIds")
            .build()
    }
    schema = schema.attribute(build_secondary_allocation_ids_attribute());
    #[inline(never)]
    fn build_secondary_private_ip_address_count_attribute() -> AttributeSchema {
        AttributeSchema::optional("secondary_private_ip_address_count", AttributeType::refined_int(None, Some((Some(1), None))))
                .with_description("[Private NAT gateway only] The number of secondary private IPv4 addresses you want to assign to the NAT gateway. For more information about secondary addresses, see [Create a NAT gateway](https://docs.aws.amazon.com/vpc/latest/userguide/vpc-nat-gateway.html#nat-gateway-creating) in the *Amazon Virtual Private Cloud User Guide*. ``SecondaryPrivateIpAddressCount`` and ``SecondaryPrivateIpAddresses`` cannot be set at the same time.")
                .with_provider_name("SecondaryPrivateIpAddressCount")
            .build()
    }
    schema = schema.attribute(build_secondary_private_ip_address_count_attribute());
    #[inline(never)]
    fn build_secondary_private_ip_addresses_attribute() -> AttributeSchema {
        AttributeSchema::optional("secondary_private_ip_addresses", AttributeType::list(types::ipv4_address()))
                .with_description("Secondary private IPv4 addresses. For more information about secondary addresses, see [Create a NAT gateway](https://docs.aws.amazon.com/vpc/latest/userguide/vpc-nat-gateway.html#nat-gateway-creating) in the *Amazon Virtual Private Cloud User Guide*. ``SecondaryPrivateIpAddressCount`` and ``SecondaryPrivateIpAddresses`` cannot be set at the same time.")
                .with_provider_name("SecondaryPrivateIpAddresses")
            .build()
    }
    schema = schema.attribute(build_secondary_private_ip_addresses_attribute());
    #[inline(never)]
    fn build_subnet_id_attribute() -> AttributeSchema {
        AttributeSchema::optional("subnet_id", carina_aws_types::subnet_id())
                .create_only()
                .with_description("The ID of the subnet in which the NAT gateway is located.")
                .with_provider_name("SubnetId")
            .build()
    }
    schema = schema.attribute(build_subnet_id_attribute());
    #[inline(never)]
    fn build_tags_attribute() -> AttributeSchema {
        AttributeSchema::optional("tags", carina_aws_types::tags_type())
                .with_description("The tags for the NAT gateway.")
                .with_provider_name("Tags")
                .with_block_name("tag")
            .build()
    }
    schema = schema.attribute(build_tags_attribute());
    #[inline(never)]
    fn build_vpc_id_attribute() -> AttributeSchema {
        AttributeSchema::optional("vpc_id", carina_aws_types::vpc_id())
                .create_only()
                .with_description("The ID of the VPC in which the NAT gateway is located.")
                .with_provider_name("VpcId")
            .build()
    }
    schema = schema.attribute(build_vpc_id_attribute());
    schema = schema
        .with_conflicting_replacement();
    schema = schema.with_operation_config(OperationConfig {
            delete_timeout_secs: Some(1200),
            delete_max_retries: None,
            create_timeout_secs: None,
            create_max_retries: None,
            update_timeout_secs: None,
    });
    schema = schema.with_declared_validators(vec![carina_provider_protocol::types::ValidatorType::TagsKeyValueCheck]);
    AwsccSchemaConfig {
        aws_type_name: "AWS::EC2::NatGateway",
        resource_type_name: "ec2.NatGateway",
        primary_identifier: &[crate::schemas::config::PrimaryIdentifierAttribute { provider_name: "NatGatewayId", dsl_name: "nat_gateway_id" }],
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
    (
        "ec2.NatGateway",
        &[
            ("availability_mode", VALID_AVAILABILITY_MODE),
            ("connectivity_type", VALID_CONNECTIVITY_TYPE),
        ],
    )
}

/// Returns the IAM permissions declared by the CloudFormation handler for this operation.
pub fn required_permissions(op: carina_core::effect::PlanOp) -> &'static [&'static str] {
    match op {
        carina_core::effect::PlanOp::Create => &[
            "ec2:CreateNatGateway",
            "ec2:DescribeNatGateways",
            "ec2:CreateTags",
        ],
        carina_core::effect::PlanOp::Read => &["ec2:DescribeNatGateways"],
        carina_core::effect::PlanOp::Update => &[
            "ec2:DescribeNatGateways",
            "ec2:CreateTags",
            "ec2:DeleteTags",
            "ec2:AssociateNatGatewayAddress",
            "ec2:DisassociateNatGatewayAddress",
            "ec2:AssignPrivateNatGatewayAddress",
            "ec2:UnassignPrivateNatGatewayAddress",
        ],
        carina_core::effect::PlanOp::Delete => &["ec2:DeleteNatGateway", "ec2:DescribeNatGateways"],
    }
}
