//! route schema definition for AWS Cloud Control
//!
//! Auto-generated from CloudFormation schema: AWS::EC2::Route
//!
//! DO NOT EDIT MANUALLY - regenerate with carina-codegen

use crate::schemas::config::{AwsccSchemaConfig, ExclusiveGroup};
use carina_core::schema::{AttributeSchema, ResourceSchema, types};

/// Returns the schema config for ec2_route (AWS::EC2::Route)
#[rustfmt::skip]
pub fn ec2_route_config() -> AwsccSchemaConfig {
    let mut schema = ResourceSchema::new("ec2.Route")
        .with_description("Specifies a route in a route table. For more information, see [Routes](https://docs.aws.amazon.com/vpc/latest/userguide/VPC_Route_Tables.html#route-table-routes) in the *Amazon VPC User Guide*.  You must specify either a destination CIDR block or prefix list ID. You must also specify exactly one of the resources as the target.  If you create a route that references a transit gateway in the same template where you create the transit gateway, you must declare a dependency on the transit gateway attachment. The route table cannot use the transit gateway until it has successfully attached to the VPC. Add a [DependsOn Attribute](https://docs.aws.amazon.com/AWSCloudFormation/latest/UserGuide/aws-attribute-dependson.html) in the ``AWS::EC2::Route`` resource to explicitly declare a dependency on the ``AWS::EC2::TransitGatewayAttachment`` resource.");
    #[inline(never)]
    fn build_carrier_gateway_id_attribute() -> AttributeSchema {
        AttributeSchema::optional("carrier_gateway_id", carina_aws_types::carrier_gateway_id())
                .with_description("The ID of the carrier gateway. You can only use this option when the VPC contains a subnet which is associated with a Wavelength Zone.")
                .with_provider_name("CarrierGatewayId")
            .build()
    }
    schema = schema.attribute(build_carrier_gateway_id_attribute());
    #[inline(never)]
    fn build_cidr_block_attribute() -> AttributeSchema {
        AttributeSchema::provider_populated("cidr_block", types::ipv4_cidr())
                .with_description(" (read-only)")
                .with_provider_name("CidrBlock")
            .build()
    }
    schema = schema.attribute(build_cidr_block_attribute());
    #[inline(never)]
    fn build_core_network_arn_attribute() -> AttributeSchema {
        AttributeSchema::optional("core_network_arn", carina_aws_types::arn())
                .with_description("The Amazon Resource Name (ARN) of the core network.")
                .with_provider_name("CoreNetworkArn")
            .build()
    }
    schema = schema.attribute(build_core_network_arn_attribute());
    #[inline(never)]
    fn build_destination_cidr_block_attribute() -> AttributeSchema {
        AttributeSchema::optional("destination_cidr_block", types::ipv4_cidr())
                .create_only()
                .with_description("The IPv4 CIDR address block used for the destination match. Routing decisions are based on the most specific match. We modify the specified CIDR block to its canonical form; for example, if you specify ``100.68.0.18/18``, we modify it to ``100.68.0.0/18``.")
                .with_provider_name("DestinationCidrBlock")
            .build()
    }
    schema = schema.attribute(build_destination_cidr_block_attribute());
    #[inline(never)]
    fn build_destination_ipv6_cidr_block_attribute() -> AttributeSchema {
        AttributeSchema::optional("destination_ipv6_cidr_block", types::ipv6_cidr())
                .create_only()
                .with_description("The IPv6 CIDR block used for the destination match. Routing decisions are based on the most specific match.")
                .with_provider_name("DestinationIpv6CidrBlock")
            .build()
    }
    schema = schema.attribute(build_destination_ipv6_cidr_block_attribute());
    #[inline(never)]
    fn build_destination_prefix_list_id_attribute() -> AttributeSchema {
        AttributeSchema::optional("destination_prefix_list_id", carina_aws_types::prefix_list_id())
                .create_only()
                .with_description("The ID of a prefix list used for the destination match.")
                .with_provider_name("DestinationPrefixListId")
            .build()
    }
    schema = schema.attribute(build_destination_prefix_list_id_attribute());
    #[inline(never)]
    fn build_egress_only_internet_gateway_id_attribute() -> AttributeSchema {
        AttributeSchema::optional("egress_only_internet_gateway_id", carina_aws_types::egress_only_internet_gateway_id())
                .with_description("[IPv6 traffic only] The ID of an egress-only internet gateway.")
                .with_provider_name("EgressOnlyInternetGatewayId")
            .build()
    }
    schema = schema.attribute(build_egress_only_internet_gateway_id_attribute());
    #[inline(never)]
    fn build_gateway_id_attribute() -> AttributeSchema {
        AttributeSchema::optional("gateway_id", carina_aws_types::gateway_id())
                .with_description("The ID of an internet gateway or virtual private gateway attached to your VPC.")
                .with_provider_name("GatewayId")
            .build()
    }
    schema = schema.attribute(build_gateway_id_attribute());
    #[inline(never)]
    fn build_instance_id_attribute() -> AttributeSchema {
        AttributeSchema::optional("instance_id", carina_aws_types::instance_id())
                .with_description("The ID of a NAT instance in your VPC. The operation fails if you specify an instance ID unless exactly one network interface is attached.")
                .with_provider_name("InstanceId")
            .build()
    }
    schema = schema.attribute(build_instance_id_attribute());
    #[inline(never)]
    fn build_local_gateway_id_attribute() -> AttributeSchema {
        AttributeSchema::optional("local_gateway_id", carina_aws_types::local_gateway_id())
                .with_description("The ID of the local gateway.")
                .with_provider_name("LocalGatewayId")
            .build()
    }
    schema = schema.attribute(build_local_gateway_id_attribute());
    #[inline(never)]
    fn build_nat_gateway_id_attribute() -> AttributeSchema {
        AttributeSchema::optional("nat_gateway_id", carina_aws_types::nat_gateway_id())
                .with_description("[IPv4 traffic only] The ID of a NAT gateway.")
                .with_provider_name("NatGatewayId")
            .build()
    }
    schema = schema.attribute(build_nat_gateway_id_attribute());
    #[inline(never)]
    fn build_network_interface_id_attribute() -> AttributeSchema {
        AttributeSchema::optional("network_interface_id", carina_aws_types::network_interface_id())
                .with_description("The ID of a network interface.")
                .with_provider_name("NetworkInterfaceId")
            .build()
    }
    schema = schema.attribute(build_network_interface_id_attribute());
    #[inline(never)]
    fn build_route_table_id_attribute() -> AttributeSchema {
        AttributeSchema::required("route_table_id", carina_aws_types::route_table_id())
                .create_only()
                .with_description("The ID of the route table for the route.")
                .with_provider_name("RouteTableId")
            .build()
    }
    schema = schema.attribute(build_route_table_id_attribute());
    #[inline(never)]
    fn build_transit_gateway_id_attribute() -> AttributeSchema {
        AttributeSchema::optional("transit_gateway_id", carina_aws_types::transit_gateway_id())
                .with_description("The ID of a transit gateway.")
                .with_provider_name("TransitGatewayId")
            .build()
    }
    schema = schema.attribute(build_transit_gateway_id_attribute());
    #[inline(never)]
    fn build_vpc_endpoint_id_attribute() -> AttributeSchema {
        AttributeSchema::optional("vpc_endpoint_id", carina_aws_types::vpc_endpoint_id())
                .with_description("The ID of a VPC endpoint. Supported for Gateway Load Balancer endpoints only.")
                .with_provider_name("VpcEndpointId")
            .build()
    }
    schema = schema.attribute(build_vpc_endpoint_id_attribute());
    #[inline(never)]
    fn build_vpc_peering_connection_id_attribute() -> AttributeSchema {
        AttributeSchema::optional("vpc_peering_connection_id", carina_aws_types::vpc_peering_connection_id())
                .with_description("The ID of a VPC peering connection.")
                .with_provider_name("VpcPeeringConnectionId")
            .build()
    }
    schema = schema.attribute(build_vpc_peering_connection_id_attribute());
    schema = schema
        .with_coexisting_replacement();
    AwsccSchemaConfig {
        aws_type_name: "AWS::EC2::Route",
        resource_type_name: "ec2.Route",
        primary_identifier: &[crate::schemas::config::PrimaryIdentifierAttribute { provider_name: "RouteTableId", dsl_name: "route_table_id" }, crate::schemas::config::PrimaryIdentifierAttribute { provider_name: "CidrBlock", dsl_name: "cidr_block" }],
        exclusive_groups: &[ExclusiveGroup { attributes: &["carrier_gateway_id", "core_network_arn", "egress_only_internet_gateway_id", "gateway_id", "instance_id", "local_gateway_id", "nat_gateway_id", "network_interface_id", "transit_gateway_id", "vpc_endpoint_id", "vpc_peering_connection_id"] }],
        tags: None,
        schema,
    }
}

/// Returns the resource type name and all enum valid values for this module
pub fn enum_valid_values() -> (
    &'static str,
    &'static [(&'static str, &'static [&'static str])],
) {
    ("ec2.Route", &[])
}

/// Returns the IAM permissions declared by the CloudFormation handler for this operation.
pub fn required_permissions(op: carina_core::effect::PlanOp) -> &'static [&'static str] {
    match op {
        carina_core::effect::PlanOp::Create => &[
            "ec2:CreateRoute",
            "ec2:DescribeRouteTables",
            "ec2:DescribeNetworkInterfaces",
        ],
        carina_core::effect::PlanOp::Read => &["ec2:DescribeRouteTables"],
        carina_core::effect::PlanOp::Update => &[
            "ec2:ReplaceRoute",
            "ec2:DescribeRouteTables",
            "ec2:DescribeNetworkInterfaces",
        ],
        carina_core::effect::PlanOp::Delete => &["ec2:DeleteRoute", "ec2:DescribeRouteTables"],
    }
}
