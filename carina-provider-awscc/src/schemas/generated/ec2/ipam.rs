//! ipam schema definition for AWS Cloud Control
//!
//! Auto-generated from CloudFormation schema: AWS::EC2::IPAM
//!
//! DO NOT EDIT MANUALLY - regenerate with carina-codegen

use crate::schemas::config::{AwsccSchemaConfig, TagsProperty, TagsWire};
use carina_core::schema::{
    AttributeSchema, AttributeType, OperationConfig, ResourceSchema, StructField,
};

pub fn arn() -> AttributeType {
    crate::schemas::catalog::ec2_ipam_arn()
}

const VALID_METERED_ACCOUNT: &[&str] = &["ipam-owner", "resource-owner"];

const VALID_TIER: &[&str] = &["free", "advanced"];

/// Returns the schema config for ec2_ipam (AWS::EC2::IPAM)
#[rustfmt::skip]
pub fn ec2_ipam_config() -> AwsccSchemaConfig {
    let mut schema = ResourceSchema::new("ec2.Ipam")
        .with_description("Resource Schema of AWS::EC2::IPAM Type");
    #[inline(never)]
    fn build_arn_attribute() -> AttributeSchema {
        AttributeSchema::provider_populated("arn", self::arn())
                .with_description("The Amazon Resource Name (ARN) of the IPAM. (read-only)")
                .with_provider_name("Arn")
            .build()
    }
    schema = schema.attribute(build_arn_attribute());
    #[inline(never)]
    fn build_default_resource_discovery_association_id_attribute() -> AttributeSchema {
        AttributeSchema::provider_populated("default_resource_discovery_association_id", AttributeType::string())
                .with_description("The Id of the default association to the default resource discovery, created with this IPAM. (read-only)")
                .with_provider_name("DefaultResourceDiscoveryAssociationId")
            .build()
    }
    schema = schema.attribute(build_default_resource_discovery_association_id_attribute());
    #[inline(never)]
    fn build_default_resource_discovery_id_attribute() -> AttributeSchema {
        AttributeSchema::provider_populated("default_resource_discovery_id", AttributeType::string())
                .with_description("The Id of the default resource discovery, created with this IPAM. (read-only)")
                .with_provider_name("DefaultResourceDiscoveryId")
            .build()
    }
    schema = schema.attribute(build_default_resource_discovery_id_attribute());
    #[inline(never)]
    fn build_default_resource_discovery_organizational_unit_exclusions_attribute() -> AttributeSchema {
        AttributeSchema::optional("default_resource_discovery_organizational_unit_exclusions", AttributeType::unordered_list(AttributeType::struct_("IpamOrganizationalUnitExclusion".to_string(), vec![StructField::required("organizations_entity_path", AttributeType::refined_string(None, None, Some((Some(1), None)), None)).with_description("An AWS Organizations entity path. Build the path for the OU(s) using AWS Organizations IDs separated by a '/'. Include all child OUs by ending the path with '/*'.").with_provider_name("OrganizationsEntityPath").build()])))
                .with_description("A set of organizational unit (OU) exclusions for the default resource discovery, created with this IPAM.")
                .with_provider_name("DefaultResourceDiscoveryOrganizationalUnitExclusions")
                .with_block_name("default_resource_discovery_organizational_unit_exclusion")
            .build()
    }
    schema = schema.attribute(build_default_resource_discovery_organizational_unit_exclusions_attribute());
    #[inline(never)]
    fn build_description_attribute() -> AttributeSchema {
        AttributeSchema::optional("description", AttributeType::string())
                .with_provider_name("Description")
            .build()
    }
    schema = schema.attribute(build_description_attribute());
    #[inline(never)]
    fn build_enable_private_gua_attribute() -> AttributeSchema {
        AttributeSchema::optional("enable_private_gua", AttributeType::bool())
                .with_description("Enable provisioning of GUA space in private pools.")
                .with_provider_name("EnablePrivateGua")
            .build()
    }
    schema = schema.attribute(build_enable_private_gua_attribute());
    #[inline(never)]
    fn build_ipam_id_attribute() -> AttributeSchema {
        AttributeSchema::provider_populated("ipam_id", carina_aws_types::ipam_id())
                .with_description("Id of the IPAM. (read-only)")
                .with_provider_name("IpamId")
            .build()
    }
    schema = schema.attribute(build_ipam_id_attribute());
    #[inline(never)]
    fn build_metered_account_attribute() -> AttributeSchema {
        AttributeSchema::optional("metered_account", AttributeType::enum_(carina_core::schema::enum_identity("MeteredAccount", Some("aws.ec2.Ipam")), Some(vec!["ipam-owner".to_string(), "resource-owner".to_string()]), vec![("ipam-owner".to_string(), "ipam_owner".to_string()), ("resource-owner".to_string(), "resource_owner".to_string())], None, None))
                .with_description("A metered account is an account that is charged for active IP addresses managed in IPAM")
                .with_provider_name("MeteredAccount")
            .build()
    }
    schema = schema.attribute(build_metered_account_attribute());
    #[inline(never)]
    fn build_operating_regions_attribute() -> AttributeSchema {
        AttributeSchema::optional("operating_regions", AttributeType::unordered_list(AttributeType::struct_("IpamOperatingRegion".to_string(), vec![StructField::required("region_name", carina_aws_types::aws_region()).with_description("The name of the region.").with_provider_name("RegionName").build()])))
                .with_description("The regions IPAM is enabled for. Allows pools to be created in these regions, as well as enabling monitoring")
                .with_provider_name("OperatingRegions")
                .with_block_name("operating_region")
            .build()
    }
    schema = schema.attribute(build_operating_regions_attribute());
    #[inline(never)]
    fn build_private_default_scope_id_attribute() -> AttributeSchema {
        AttributeSchema::provider_populated("private_default_scope_id", AttributeType::string())
                .with_description("The Id of the default scope for publicly routable IP space, created with this IPAM. (read-only)")
                .with_provider_name("PrivateDefaultScopeId")
            .build()
    }
    schema = schema.attribute(build_private_default_scope_id_attribute());
    #[inline(never)]
    fn build_public_default_scope_id_attribute() -> AttributeSchema {
        AttributeSchema::provider_populated("public_default_scope_id", AttributeType::refined_string(None, None, Some((None, Some(255))), None))
                .with_description("The Id of the default scope for publicly routable IP space, created with this IPAM. (read-only)")
                .with_provider_name("PublicDefaultScopeId")
            .build()
    }
    schema = schema.attribute(build_public_default_scope_id_attribute());
    #[inline(never)]
    fn build_resource_discovery_association_count_attribute() -> AttributeSchema {
        AttributeSchema::provider_populated("resource_discovery_association_count", AttributeType::int())
                .with_description("The count of resource discoveries associated with this IPAM. (read-only)")
                .with_provider_name("ResourceDiscoveryAssociationCount")
            .build()
    }
    schema = schema.attribute(build_resource_discovery_association_count_attribute());
    #[inline(never)]
    fn build_scope_count_attribute() -> AttributeSchema {
        AttributeSchema::provider_populated("scope_count", AttributeType::int())
                .with_description("The number of scopes that currently exist in this IPAM. (read-only)")
                .with_provider_name("ScopeCount")
            .build()
    }
    schema = schema.attribute(build_scope_count_attribute());
    #[inline(never)]
    fn build_tags_attribute() -> AttributeSchema {
        AttributeSchema::optional("tags", carina_aws_types::tags_type())
                .with_description("An array of key-value pairs to apply to this resource.")
                .with_provider_name("Tags")
                .with_block_name("tag")
            .build()
    }
    schema = schema.attribute(build_tags_attribute());
    #[inline(never)]
    fn build_tier_attribute() -> AttributeSchema {
        AttributeSchema::optional("tier", AttributeType::enum_(carina_core::schema::enum_identity("Tier", Some("aws.ec2.Ipam")), Some(vec!["free".to_string(), "advanced".to_string()]), vec![("free".to_string(), "free".to_string()), ("advanced".to_string(), "advanced".to_string())], None, None))
                .with_description("The tier of the IPAM.")
                .with_provider_name("Tier")
            .build()
    }
    schema = schema.attribute(build_tier_attribute());
    schema = schema
        .with_conflicting_replacement();
    schema = schema.with_operation_config(OperationConfig {
            delete_timeout_secs: Some(3600),
            delete_max_retries: None,
            create_timeout_secs: None,
            create_max_retries: None,
            update_timeout_secs: None,
    });
    schema = schema.with_declared_validators(vec![carina_provider_protocol::types::ValidatorType::TagsKeyValueCheck]);
    AwsccSchemaConfig {
        aws_type_name: "AWS::EC2::IPAM",
        resource_type_name: "ec2.Ipam",
        primary_identifier: &[crate::schemas::config::PrimaryIdentifierAttribute { provider_name: "IpamId", dsl_name: "ipam_id" }],
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
        "ec2.Ipam",
        &[
            ("metered_account", VALID_METERED_ACCOUNT),
            ("tier", VALID_TIER),
        ],
    )
}

/// Returns the IAM permissions declared by the CloudFormation handler for this operation.
pub fn required_permissions(op: carina_core::effect::PlanOp) -> &'static [&'static str] {
    match op {
        carina_core::effect::PlanOp::Create => &[
            "ec2:CreateIpam",
            "iam:CreateServiceLinkedRole",
            "ec2:CreateTags",
            "ec2:DescribeIpams",
            "ec2:DescribeIpamResourceDiscoveries",
            "ec2:ModifyIpamResourceDiscovery",
        ],
        carina_core::effect::PlanOp::Read => {
            &["ec2:DescribeIpams", "ec2:DescribeIpamResourceDiscoveries"]
        }
        carina_core::effect::PlanOp::Update => &[
            "ec2:ModifyIpam",
            "ec2:CreateTags",
            "ec2:DeleteTags",
            "ec2:DescribeIpams",
            "ec2:DescribeIpamResourceDiscoveries",
            "ec2:ModifyIpamResourceDiscovery",
        ],
        carina_core::effect::PlanOp::Delete => {
            &["ec2:DeleteIpam", "ec2:DeleteTags", "ec2:DescribeIpams"]
        }
    }
}
