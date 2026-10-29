//! vpc schema definition for AWS Cloud Control
//!
//! Auto-generated from CloudFormation schema: AWS::EC2::VPC
//!
//! DO NOT EDIT MANUALLY - regenerate with carina-codegen

use crate::schemas::config::{AwsccSchemaConfig, TagsProperty, TagsWire};
use carina_core::schema::{AttributeSchema, AttributeType, ResourceSchema, types};

const VALID_INSTANCE_TENANCY: &[&str] = &["default", "dedicated", "host"];

/// Returns the schema config for ec2_vpc (AWS::EC2::VPC)
#[rustfmt::skip]
pub fn ec2_vpc_config() -> AwsccSchemaConfig {
    let mut schema = ResourceSchema::new("ec2.Vpc")
        .with_description("Specifies a virtual private cloud (VPC).  To add an IPv6 CIDR block to the VPC, see [AWS::EC2::VPCCidrBlock](https://docs.aws.amazon.com/AWSCloudFormation/latest/UserGuide/aws-resource-ec2-vpccidrblock.html).  For more information, see [Virtual private clouds (VPC)](https://docs.aws.amazon.com/vpc/latest/userguide/configure-your-vpc.html) in the *Amazon VPC User Guide*.");
    #[inline(never)]
    fn build_cidr_block_attribute() -> AttributeSchema {
        AttributeSchema::optional("cidr_block", types::ipv4_cidr())
                .create_only()
                .with_description("The IPv4 network range for the VPC, in CIDR notation. For example, ``10.0.0.0/16``. We modify the specified CIDR block to its canonical form; for example, if you specify ``100.68.0.18/18``, we modify it to ``100.68.0.0/18``. You must specify either``CidrBlock`` or ``Ipv4IpamPoolId``.")
                .with_provider_name("CidrBlock")
            .build()
    }
    schema = schema.attribute(build_cidr_block_attribute());
    #[inline(never)]
    fn build_cidr_block_associations_attribute() -> AttributeSchema {
        AttributeSchema::provider_populated("cidr_block_associations", AttributeType::unordered_list(carina_aws_types::vpc_cidr_block_association_id()))
                .with_description(" (read-only)")
                .with_provider_name("CidrBlockAssociations")
            .build()
    }
    schema = schema.attribute(build_cidr_block_associations_attribute());
    #[inline(never)]
    fn build_default_network_acl_attribute() -> AttributeSchema {
        AttributeSchema::provider_populated("default_network_acl", carina_aws_types::network_acl_id())
                .with_description(" (read-only)")
                .with_provider_name("DefaultNetworkAcl")
            .build()
    }
    schema = schema.attribute(build_default_network_acl_attribute());
    #[inline(never)]
    fn build_default_security_group_attribute() -> AttributeSchema {
        AttributeSchema::provider_populated("default_security_group", carina_aws_types::security_group_id())
                .with_description(" (read-only)")
                .with_provider_name("DefaultSecurityGroup")
            .build()
    }
    schema = schema.attribute(build_default_security_group_attribute());
    #[inline(never)]
    fn build_enable_dns_hostnames_attribute() -> AttributeSchema {
        AttributeSchema::optional("enable_dns_hostnames", AttributeType::bool())
                .with_description("Indicates whether the instances launched in the VPC get DNS hostnames. If enabled, instances in the VPC get DNS hostnames; otherwise, they do not. Disabled by default for nondefault VPCs. For more information, see [DNS attributes in your VPC](https://docs.aws.amazon.com/vpc/latest/userguide/vpc-dns.html#vpc-dns-support). You can only enable DNS hostnames if you've enabled DNS support.")
                .with_provider_name("EnableDnsHostnames")
            .build()
    }
    schema = schema.attribute(build_enable_dns_hostnames_attribute());
    #[inline(never)]
    fn build_enable_dns_support_attribute() -> AttributeSchema {
        AttributeSchema::optional("enable_dns_support", AttributeType::bool())
                .with_description("Indicates whether the DNS resolution is supported for the VPC. If enabled, queries to the Amazon provided DNS server at the 169.254.169.253 IP address, or the reserved IP address at the base of the VPC network range \"plus two\" succeed. If disabled, the Amazon provided DNS service in the VPC that resolves public DNS hostnames to IP addresses is not enabled. Enabled by default. For more information, see [DNS attributes in your VPC](https://docs.aws.amazon.com/vpc/latest/userguide/vpc-dns.html#vpc-dns-support).")
                .with_provider_name("EnableDnsSupport")
            .build()
    }
    schema = schema.attribute(build_enable_dns_support_attribute());
    #[inline(never)]
    fn build_instance_tenancy_attribute() -> AttributeSchema {
        AttributeSchema::optional("instance_tenancy", AttributeType::enum_(carina_core::schema::enum_identity("InstanceTenancy", Some("aws.ec2.Vpc")), Some(vec!["default".to_string(), "dedicated".to_string(), "host".to_string()]), vec![("default".to_string(), "default".to_string()), ("dedicated".to_string(), "dedicated".to_string()), ("host".to_string(), "host".to_string())], None, None))
                .with_description("The allowed tenancy of instances launched into the VPC. + ``default``: An instance launched into the VPC runs on shared hardware by default, unless you explicitly specify a different tenancy during instance launch. + ``dedicated``: An instance launched into the VPC runs on dedicated hardware by default, unless you explicitly specify a tenancy of ``host`` during instance launch. You cannot specify a tenancy of ``default`` during instance launch. Updating ``InstanceTenancy`` requires no replacement only if you are updating its value from ``dedicated`` to ``default``. Updating ``InstanceTenancy`` from ``default`` to ``dedicated`` requires replacement.")
                .with_provider_name("InstanceTenancy")
            .build()
    }
    schema = schema.attribute(build_instance_tenancy_attribute());
    #[inline(never)]
    fn build_ipv4_ipam_pool_id_attribute() -> AttributeSchema {
        AttributeSchema::optional("ipv4_ipam_pool_id", carina_aws_types::ipam_pool_id())
                .create_only()
                .write_only()
                .with_description("The ID of an IPv4 IPAM pool you want to use for allocating this VPC's CIDR. For more information, see [What is IPAM?](https://docs.aws.amazon.com//vpc/latest/ipam/what-is-it-ipam.html) in the *Amazon VPC IPAM User Guide*. You must specify either``CidrBlock`` or ``Ipv4IpamPoolId``.")
                .with_provider_name("Ipv4IpamPoolId")
            .build()
    }
    schema = schema.attribute(build_ipv4_ipam_pool_id_attribute());
    #[inline(never)]
    fn build_ipv4_netmask_length_attribute() -> AttributeSchema {
        AttributeSchema::optional("ipv4_netmask_length", AttributeType::refined_int(None, Some((Some(0), Some(32)))))
                .create_only()
                .write_only()
                .with_description("The netmask length of the IPv4 CIDR you want to allocate to this VPC from an Amazon VPC IP Address Manager (IPAM) pool. For more information about IPAM, see [What is IPAM?](https://docs.aws.amazon.com//vpc/latest/ipam/what-is-it-ipam.html) in the *Amazon VPC IPAM User Guide*.")
                .with_provider_name("Ipv4NetmaskLength")
            .build()
    }
    schema = schema.attribute(build_ipv4_netmask_length_attribute());
    #[inline(never)]
    fn build_ipv6_cidr_blocks_attribute() -> AttributeSchema {
        AttributeSchema::provider_populated("ipv6_cidr_blocks", AttributeType::unordered_list(types::ipv6_cidr()))
                .with_description(" (read-only)")
                .with_provider_name("Ipv6CidrBlocks")
            .build()
    }
    schema = schema.attribute(build_ipv6_cidr_blocks_attribute());
    #[inline(never)]
    fn build_tags_attribute() -> AttributeSchema {
        AttributeSchema::optional("tags", carina_aws_types::tags_type())
                .with_description("The tags for the VPC.")
                .with_provider_name("Tags")
                .with_block_name("tag")
            .build()
    }
    schema = schema.attribute(build_tags_attribute());
    #[inline(never)]
    fn build_vpc_id_attribute() -> AttributeSchema {
        AttributeSchema::provider_populated("vpc_id", carina_aws_types::vpc_id())
                .with_description(" (read-only)")
                .with_provider_name("VpcId")
            .build()
    }
    schema = schema.attribute(build_vpc_id_attribute());
    schema = schema
        .with_coexisting_replacement();
    schema = schema.exclusive_required(&["cidr_block", "ipv4_ipam_pool_id"]);
    schema = schema.with_declared_validators(vec![carina_provider_protocol::types::ValidatorType::TagsKeyValueCheck]);
    AwsccSchemaConfig {
        aws_type_name: "AWS::EC2::VPC",
        resource_type_name: "ec2.Vpc",
        primary_identifier: &[crate::schemas::config::PrimaryIdentifierAttribute { provider_name: "VpcId", dsl_name: "vpc_id" }],
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
    ("ec2.Vpc", &[("instance_tenancy", VALID_INSTANCE_TENANCY)])
}

/// Returns the IAM permissions declared by the CloudFormation handler for this operation.
pub fn required_permissions(op: carina_core::effect::PlanOp) -> &'static [&'static str] {
    match op {
        carina_core::effect::PlanOp::Create => &[
            "ec2:CreateVpc",
            "ec2:DescribeVpcs",
            "ec2:DescribeVpcAttribute",
            "ec2:ModifyVpcAttribute",
            "ec2:CreateTags",
        ],
        carina_core::effect::PlanOp::Read => &[
            "ec2:DescribeVpcs",
            "ec2:DescribeSecurityGroups",
            "ec2:DescribeNetworkAcls",
            "ec2:DescribeVpcAttribute",
        ],
        carina_core::effect::PlanOp::Update => &[
            "ec2:CreateTags",
            "ec2:ModifyVpcAttribute",
            "ec2:DescribeVpcAttribute",
            "ec2:DeleteTags",
            "ec2:ModifyVpcTenancy",
        ],
        carina_core::effect::PlanOp::Delete => &["ec2:DeleteVpc", "ec2:DescribeVpcs"],
    }
}
