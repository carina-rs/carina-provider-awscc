//! ipam schema definition for AWS Cloud Control
//!
//! Auto-generated from CloudFormation schema: AWS::EC2::IPAM
//!
//! DO NOT EDIT MANUALLY - regenerate with carina-codegen

use crate::schemas::config::AwsccSchemaConfig;
use carina_core::resource::{ConcreteValue, Value};
use carina_core::schema::{
    AttributeSchema, AttributeType, OperationConfig, ResourceSchema, StructField, legacy_validator,
};

pub fn arn() -> AttributeType {
    AttributeType::refined_string_with_validator(
        Some(carina_aws_types::provider_type("ec2", "Ipam", "Arn")),
        Some("^arn:(aws|aws-cn|aws-us-gov):ec2:.*$".to_string()),
        None,
        legacy_validator(|value| {
            if let Value::Concrete(ConcreteValue::String(s)) = value {
                carina_aws_types::validate_service_arn(s, "ec2", None)
                    .map_err(|reason| format!("Invalid ec2 ARN '{}': {}", s, reason))
            } else {
                Err("Expected string".to_string())
            }
        }),
        None,
    )
}

const VALID_METERED_ACCOUNT: &[&str] = &["ipam-owner", "resource-owner"];

const VALID_TIER: &[&str] = &["free", "advanced"];

#[allow(dead_code)]
fn validate_string_length_min_1(value: &Value) -> Result<(), String> {
    if let Value::Concrete(ConcreteValue::String(s)) = value {
        let len = s.chars().count();
        if len < 1 {
            Err(format!("String length {} is out of range 1..", len))
        } else {
            Ok(())
        }
    } else {
        Ok(())
    }
}

#[allow(dead_code)]
fn validate_string_length_max_255(value: &Value) -> Result<(), String> {
    if let Value::Concrete(ConcreteValue::String(s)) = value {
        let len = s.chars().count();
        if len > 255 {
            Err(format!("String length {} is out of range ..=255", len))
        } else {
            Ok(())
        }
    } else {
        Ok(())
    }
}

/// Returns the schema config for ec2_ipam (AWS::EC2::IPAM)
pub fn ec2_ipam_config() -> AwsccSchemaConfig {
    AwsccSchemaConfig {
        aws_type_name: "AWS::EC2::IPAM",
        resource_type_name: "ec2.Ipam",
        primary_identifier: &[crate::schemas::config::PrimaryIdentifierAttribute { provider_name: "IpamId", dsl_name: "ipam_id" }],
        has_tags: true,
        schema: ResourceSchema::new("ec2.Ipam")
	        .with_description("Resource Schema of AWS::EC2::IPAM Type")
        .attribute(
            AttributeSchema::new("arn", self::arn())
                .read_only()
                .with_description("The Amazon Resource Name (ARN) of the IPAM. (read-only)")
                .with_provider_name("Arn"),
        )
        .attribute(
            AttributeSchema::new("default_resource_discovery_association_id", AttributeType::string())
                .read_only()
                .with_description("The Id of the default association to the default resource discovery, created with this IPAM. (read-only)")
                .with_provider_name("DefaultResourceDiscoveryAssociationId"),
        )
        .attribute(
            AttributeSchema::new("default_resource_discovery_id", AttributeType::string())
                .read_only()
                .with_description("The Id of the default resource discovery, created with this IPAM. (read-only)")
                .with_provider_name("DefaultResourceDiscoveryId"),
        )
        .attribute(
            AttributeSchema::new("default_resource_discovery_organizational_unit_exclusions", AttributeType::unordered_list(AttributeType::struct_("IpamOrganizationalUnitExclusion".to_string(), vec![StructField::new("organizations_entity_path", AttributeType::refined_string(None, None, Some((Some(1), None)), None)).required().with_description("An AWS Organizations entity path. Build the path for the OU(s) using AWS Organizations IDs separated by a '/'. Include all child OUs by ending the path with '/*'.").with_provider_name("OrganizationsEntityPath")])))
                .with_description("A set of organizational unit (OU) exclusions for the default resource discovery, created with this IPAM.")
                .with_provider_name("DefaultResourceDiscoveryOrganizationalUnitExclusions")
                .with_block_name("default_resource_discovery_organizational_unit_exclusion"),
        )
        .attribute(
            AttributeSchema::new("description", AttributeType::string())
                .with_provider_name("Description"),
        )
        .attribute(
            AttributeSchema::new("enable_private_gua", AttributeType::bool())
                .with_description("Enable provisioning of GUA space in private pools.")
                .with_provider_name("EnablePrivateGua"),
        )
        .attribute(
            AttributeSchema::new("ipam_id", carina_aws_types::ipam_id())
                .read_only()
                .with_description("Id of the IPAM. (read-only)")
                .with_provider_name("IpamId"),
        )
        .attribute(
            AttributeSchema::new("metered_account", AttributeType::enum_(carina_core::schema::enum_identity("MeteredAccount", Some("aws.ec2.Ipam")), Some(vec!["ipam-owner".to_string(), "resource-owner".to_string()]), vec![("ipam-owner".to_string(), "ipam_owner".to_string()), ("resource-owner".to_string(), "resource_owner".to_string())], None, None))
                .with_description("A metered account is an account that is charged for active IP addresses managed in IPAM")
                .with_provider_name("MeteredAccount"),
        )
        .attribute(
            AttributeSchema::new("operating_regions", AttributeType::unordered_list(AttributeType::struct_("IpamOperatingRegion".to_string(), vec![StructField::new("region_name", carina_aws_types::aws_region()).required().with_description("The name of the region.").with_provider_name("RegionName")])))
                .with_description("The regions IPAM is enabled for. Allows pools to be created in these regions, as well as enabling monitoring")
                .with_provider_name("OperatingRegions")
                .with_block_name("operating_region"),
        )
        .attribute(
            AttributeSchema::new("private_default_scope_id", AttributeType::string())
                .read_only()
                .with_description("The Id of the default scope for publicly routable IP space, created with this IPAM. (read-only)")
                .with_provider_name("PrivateDefaultScopeId"),
        )
        .attribute(
            AttributeSchema::new("public_default_scope_id", AttributeType::refined_string(None, None, Some((None, Some(255))), None))
                .read_only()
                .with_description("The Id of the default scope for publicly routable IP space, created with this IPAM. (read-only)")
                .with_provider_name("PublicDefaultScopeId"),
        )
        .attribute(
            AttributeSchema::new("resource_discovery_association_count", AttributeType::int())
                .read_only()
                .with_description("The count of resource discoveries associated with this IPAM. (read-only)")
                .with_provider_name("ResourceDiscoveryAssociationCount"),
        )
        .attribute(
            AttributeSchema::new("scope_count", AttributeType::int())
                .read_only()
                .with_description("The number of scopes that currently exist in this IPAM. (read-only)")
                .with_provider_name("ScopeCount"),
        )
        .attribute(
            AttributeSchema::new("tags", carina_aws_types::tags_type())
                .with_description("An array of key-value pairs to apply to this resource.")
                .with_provider_name("Tags")
                .with_block_name("tag"),
        )
        .attribute(
            AttributeSchema::new("tier", AttributeType::enum_(carina_core::schema::enum_identity("Tier", Some("aws.ec2.Ipam")), Some(vec!["free".to_string(), "advanced".to_string()]), vec![("free".to_string(), "free".to_string()), ("advanced".to_string(), "advanced".to_string())], None, None))
                .with_description("The tier of the IPAM.")
                .with_provider_name("Tier"),
        )
        .with_operation_config(OperationConfig {
            delete_timeout_secs: Some(1800),
            delete_max_retries: None,
            create_timeout_secs: None,
            create_max_retries: None,
        })
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
