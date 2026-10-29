//! eip schema definition for AWS Cloud Control
//!
//! Auto-generated from CloudFormation schema: AWS::EC2::EIP
//!
//! DO NOT EDIT MANUALLY - regenerate with carina-codegen

use crate::schemas::config::{AwsccSchemaConfig, TagsProperty, TagsWire};
use carina_core::schema::{AttributeSchema, AttributeType, ResourceSchema, types};

const VALID_DOMAIN: &[&str] = &["vpc", "standard"];

/// Returns the schema config for ec2_eip (AWS::EC2::EIP)
#[rustfmt::skip]
pub fn ec2_eip_config() -> AwsccSchemaConfig {
    let mut schema = ResourceSchema::new("ec2.Eip")
        .with_description("Specifies an Elastic IP (EIP) address and can, optionally, associate it with an Amazon EC2 instance.  You can allocate an Elastic IP address from an address pool owned by AWS or from an address pool created from a public IPv4 address range that you have brought to AWS for use with your AWS resources using bring your own IP addresses (BYOIP). For more information, see [Bring Your Own IP Addresses (BYOIP)](https://docs.aws.amazon.com/AWSEC2/latest/UserGuide/ec2-byoip.html) in the *Amazon EC2 User Guide*.  For more information, see [Elastic IP Addresses](https://docs.aws.amazon.com/AWSEC2/latest/UserGuide/elastic-ip-addresses-eip.html) in the *Amazon EC2 User Guide*.");
    #[inline(never)]
    fn build_address_attribute() -> AttributeSchema {
        AttributeSchema::optional("address", types::ipv4_address())
                .create_only()
                .write_only()
                .with_description("")
                .with_provider_name("Address")
            .build()
    }
    schema = schema.attribute(build_address_attribute());
    #[inline(never)]
    fn build_allocation_id_attribute() -> AttributeSchema {
        AttributeSchema::provider_populated("allocation_id", carina_aws_types::allocation_id())
                .with_description(" (read-only)")
                .with_provider_name("AllocationId")
            .build()
    }
    schema = schema.attribute(build_allocation_id_attribute());
    #[inline(never)]
    fn build_domain_attribute() -> AttributeSchema {
        AttributeSchema::optional("domain", AttributeType::enum_(carina_core::schema::enum_identity("Domain", Some("aws.ec2.Eip")), Some(vec!["vpc".to_string(), "standard".to_string()]), vec![("vpc".to_string(), "vpc".to_string()), ("standard".to_string(), "standard".to_string())], None, None))
                .with_description("The network (``vpc``). If you define an Elastic IP address and associate it with a VPC that is defined in the same template, you must declare a dependency on the VPC-gateway attachment by using the [DependsOn Attribute](https://docs.aws.amazon.com/AWSCloudFormation/latest/UserGuide/aws-attribute-dependson.html) on this resource.")
                .with_provider_name("Domain")
            .build()
    }
    schema = schema.attribute(build_domain_attribute());
    #[inline(never)]
    fn build_instance_id_attribute() -> AttributeSchema {
        AttributeSchema::optional("instance_id", carina_aws_types::instance_id())
                .with_description("The ID of the instance. Updates to the ``InstanceId`` property may require *some interruptions*. Updates on an EIP reassociates the address on its associated resource.")
                .with_provider_name("InstanceId")
            .build()
    }
    schema = schema.attribute(build_instance_id_attribute());
    #[inline(never)]
    fn build_ipam_pool_id_attribute() -> AttributeSchema {
        AttributeSchema::optional("ipam_pool_id", carina_aws_types::ipam_pool_id())
                .create_only()
                .write_only()
                .with_description("")
                .with_provider_name("IpamPoolId")
            .build()
    }
    schema = schema.attribute(build_ipam_pool_id_attribute());
    #[inline(never)]
    fn build_network_border_group_attribute() -> AttributeSchema {
        AttributeSchema::optional("network_border_group", AttributeType::string())
                .create_only()
                .with_description("A unique set of Availability Zones, Local Zones, or Wavelength Zones from which AWS advertises IP addresses. Use this parameter to limit the IP address to this location. IP addresses cannot move between network border groups. Use [DescribeAvailabilityZones](https://docs.aws.amazon.com/AWSEC2/latest/APIReference/API_DescribeAvailabilityZones.html) to view the network border groups.")
                .with_provider_name("NetworkBorderGroup")
            .build()
    }
    schema = schema.attribute(build_network_border_group_attribute());
    #[inline(never)]
    fn build_public_ip_attribute() -> AttributeSchema {
        AttributeSchema::provider_populated("public_ip", types::ipv4_address())
                .with_description(" (read-only)")
                .with_provider_name("PublicIp")
            .build()
    }
    schema = schema.attribute(build_public_ip_attribute());
    #[inline(never)]
    fn build_public_ipv4_pool_attribute() -> AttributeSchema {
        AttributeSchema::optional("public_ipv4_pool", AttributeType::string())
                .with_description("The ID of an address pool that you own. Use this parameter to let Amazon EC2 select an address from the address pool. Updates to the ``PublicIpv4Pool`` property may require *some interruptions*. Updates on an EIP reassociates the address on its associated resource.")
                .with_provider_name("PublicIpv4Pool")
            .build()
    }
    schema = schema.attribute(build_public_ipv4_pool_attribute());
    #[inline(never)]
    fn build_tags_attribute() -> AttributeSchema {
        AttributeSchema::optional("tags", carina_aws_types::tags_type())
                .with_description("Any tags assigned to the Elastic IP address. Updates to the ``Tags`` property may require *some interruptions*. Updates on an EIP reassociates the address on its associated resource.")
                .with_provider_name("Tags")
                .with_block_name("tag")
            .build()
    }
    schema = schema.attribute(build_tags_attribute());
    #[inline(never)]
    fn build_transfer_address_attribute() -> AttributeSchema {
        AttributeSchema::optional("transfer_address", types::ipv4_address())
                .create_only()
                .write_only()
                .with_description("The Elastic IP address you are accepting for transfer. You can only accept one transferred address. For more information on Elastic IP address transfers, see [Transfer Elastic IP addresses](https://docs.aws.amazon.com/vpc/latest/userguide/vpc-eips.html#transfer-EIPs-intro) in the *Amazon Virtual Private Cloud User Guide*.")
                .with_provider_name("TransferAddress")
            .build()
    }
    schema = schema.attribute(build_transfer_address_attribute());
    schema = schema
        .with_conflicting_replacement();
    schema = schema.with_declared_validators(vec![carina_provider_protocol::types::ValidatorType::TagsKeyValueCheck]);
    AwsccSchemaConfig {
        aws_type_name: "AWS::EC2::EIP",
        resource_type_name: "ec2.Eip",
        primary_identifier: &[crate::schemas::config::PrimaryIdentifierAttribute { provider_name: "PublicIp", dsl_name: "public_ip" }, crate::schemas::config::PrimaryIdentifierAttribute { provider_name: "AllocationId", dsl_name: "allocation_id" }],
        tags: Some(TagsProperty { provider_name: "Tags", wire: TagsWire::KeyValueList }),
        schema,
    }
}

/// Returns the resource type name and all enum valid values for this module
pub fn enum_valid_values() -> (
    &'static str,
    &'static [(&'static str, &'static [&'static str])],
) {
    ("ec2.Eip", &[("domain", VALID_DOMAIN)])
}

/// Returns the IAM permissions declared by the CloudFormation handler for this operation.
pub fn required_permissions(op: carina_core::effect::PlanOp) -> &'static [&'static str] {
    match op {
        carina_core::effect::PlanOp::Create => &[
            "ec2:AllocateAddress",
            "ec2:AcceptAddressTransfer",
            "ec2:DescribeAddresses",
            "ec2:AssociateAddress",
            "ec2:CreateTags",
        ],
        carina_core::effect::PlanOp::Read => &["ec2:DescribeAddresses"],
        carina_core::effect::PlanOp::Update => &[
            "ec2:DescribeAddresses",
            "ec2:DisassociateAddress",
            "ec2:DeleteTags",
            "ec2:CreateTags",
            "ec2:AssociateAddress",
        ],
        carina_core::effect::PlanOp::Delete => &[
            "ec2:ReleaseAddress",
            "ec2:DescribeAddresses",
            "ec2:DisassociateAddress",
        ],
    }
}
