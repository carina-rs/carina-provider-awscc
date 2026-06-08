//! ipam_pool schema definition for AWS Cloud Control
//!
//! Auto-generated from CloudFormation schema: AWS::EC2::IPAMPool
//!
//! DO NOT EDIT MANUALLY - regenerate with carina-codegen

use crate::schemas::config::AwsccSchemaConfig;
use carina_core::resource::{ConcreteValue, Value};
use carina_core::schema::{
    AttributeSchema, AttributeType, OperationConfig, ResourceSchema, StructField, legacy_validator,
    types,
};

pub fn arn() -> AttributeType {
    AttributeType::custom(
        Some(carina_aws_types::provider_type("ec2", "IpamPool", "Arn")),
        carina_aws_types::arn(),
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

const VALID_ADDRESS_FAMILY: &[&str] = &["IPv4", "IPv6"];

const VALID_AWS_SERVICE: &[&str] = &["ec2", "global-services"];

const VALID_IPAM_SCOPE_TYPE: &[&str] = &["public", "private"];

const VALID_PUBLIC_IP_SOURCE: &[&str] = &["byoip", "amazon"];

const VALID_STATE: &[&str] = &[
    "create-in-progress",
    "create-complete",
    "modify-in-progress",
    "modify-complete",
    "delete-in-progress",
    "delete-complete",
];

/// Returns the schema config for ec2_ipam_pool (AWS::EC2::IPAMPool)
pub fn ec2_ipam_pool_config() -> AwsccSchemaConfig {
    AwsccSchemaConfig {
        aws_type_name: "AWS::EC2::IPAMPool",
        resource_type_name: "ec2.IpamPool",
        has_tags: true,
        schema: ResourceSchema::new("ec2.IpamPool")
        .with_description("Resource Schema of AWS::EC2::IPAMPool Type")
        .attribute(
            AttributeSchema::new("address_family", AttributeType::enum_(carina_core::schema::enum_identity("AddressFamily", Some("aws.ec2.IpamPool")), Some(vec!["IPv4".to_string(), "IPv6".to_string()]), vec![("IPv4".to_string(), "ipv4".to_string()), ("IPv6".to_string(), "ipv6".to_string())], None, None))
                .required()
                .create_only()
                .with_description("The address family of the address space in this pool. Either IPv4 or IPv6.")
                .with_provider_name("AddressFamily"),
        )
        .attribute(
            AttributeSchema::new("allocation_default_netmask_length", AttributeType::int())
                .with_description("The default netmask length for allocations made from this pool. This value is used when the netmask length of an allocation isn't specified.")
                .with_provider_name("AllocationDefaultNetmaskLength"),
        )
        .attribute(
            AttributeSchema::new("allocation_max_netmask_length", AttributeType::int())
                .with_description("The maximum allowed netmask length for allocations made from this pool.")
                .with_provider_name("AllocationMaxNetmaskLength"),
        )
        .attribute(
            AttributeSchema::new("allocation_min_netmask_length", AttributeType::int())
                .with_description("The minimum allowed netmask length for allocations made from this pool.")
                .with_provider_name("AllocationMinNetmaskLength"),
        )
        .attribute(
            AttributeSchema::new("allocation_resource_tags", AttributeType::unordered_list(carina_aws_types::tags_type()))
                .with_description("When specified, an allocation will not be allowed unless a resource has a matching set of tags.")
                .with_provider_name("AllocationResourceTags")
                .with_block_name("allocation_resource_tag"),
        )
        .attribute(
            AttributeSchema::new("arn", self::arn())
                .read_only()
                .with_description("The Amazon Resource Name (ARN) of the IPAM Pool. (read-only)")
                .with_provider_name("Arn"),
        )
        .attribute(
            AttributeSchema::new("auto_import", AttributeType::bool())
                .with_description("Determines what to do if IPAM discovers resources that haven't been assigned an allocation. If set to true, an allocation will be made automatically.")
                .with_provider_name("AutoImport"),
        )
        .attribute(
            AttributeSchema::new("aws_service", AttributeType::enum_(carina_core::schema::enum_identity("AwsService", Some("aws.ec2.IpamPool")), Some(vec!["ec2".to_string(), "global-services".to_string()]), vec![("ec2".to_string(), "ec2".to_string()), ("global-services".to_string(), "global_services".to_string())], None, None))
                .create_only()
                .with_description("Limits which service in Amazon Web Services that the pool can be used in.")
                .with_provider_name("AwsService"),
        )
        .attribute(
            AttributeSchema::new("description", AttributeType::string())
                .with_provider_name("Description"),
        )
        .attribute(
            AttributeSchema::new("ipam_arn", carina_aws_types::arn())
                .read_only()
                .with_description("The Amazon Resource Name (ARN) of the IPAM this pool is a part of. (read-only)")
                .with_provider_name("IpamArn"),
        )
        .attribute(
            AttributeSchema::new("ipam_pool_id", carina_aws_types::ipam_pool_id())
                .read_only()
                .with_description("Id of the IPAM Pool. (read-only)")
                .with_provider_name("IpamPoolId"),
        )
        .attribute(
            AttributeSchema::new("ipam_scope_arn", carina_aws_types::arn())
                .read_only()
                .with_description("The Amazon Resource Name (ARN) of the scope this pool is a part of. (read-only)")
                .with_provider_name("IpamScopeArn"),
        )
        .attribute(
            AttributeSchema::new("ipam_scope_id", AttributeType::string())
                .required()
                .create_only()
                .with_description("The Id of the scope this pool is a part of.")
                .with_provider_name("IpamScopeId"),
        )
        .attribute(
            AttributeSchema::new("ipam_scope_type", AttributeType::enum_(carina_core::schema::enum_identity("IpamScopeType", Some("aws.ec2.IpamPool")), Some(vec!["public".to_string(), "private".to_string()]), vec![("public".to_string(), "public".to_string()), ("private".to_string(), "private".to_string())], None, None))
                .read_only()
                .with_description("Determines whether this scope contains publicly routable space or space for a private network (read-only)")
                .with_provider_name("IpamScopeType"),
        )
        .attribute(
            AttributeSchema::new("locale", carina_aws_types::aws_region())
                .create_only()
                .with_description("The region of this pool. If not set, this will default to \"None\" which will disable non-custom allocations. If the locale has been specified for the source pool, this value must match.")
                .with_provider_name("Locale"),
        )
        .attribute(
            AttributeSchema::new("pool_depth", AttributeType::int())
                .read_only()
                .with_description("The depth of this pool in the source pool hierarchy. (read-only)")
                .with_provider_name("PoolDepth"),
        )
        .attribute(
            AttributeSchema::new("provisioned_cidrs", AttributeType::unordered_list(AttributeType::struct_("ProvisionedCidr".to_string(), vec![StructField::new("cidr", types::cidr()).required().with_provider_name("Cidr")])))
                .with_description("A list of cidrs representing the address space available for allocation in this pool.")
                .with_provider_name("ProvisionedCidrs")
                .with_block_name("provisioned_cidr"),
        )
        .attribute(
            AttributeSchema::new("public_ip_source", AttributeType::enum_(carina_core::schema::enum_identity("PublicIpSource", Some("aws.ec2.IpamPool")), Some(vec!["byoip".to_string(), "amazon".to_string()]), vec![("byoip".to_string(), "byoip".to_string()), ("amazon".to_string(), "amazon".to_string())], None, None))
                .create_only()
                .with_description("The IP address source for pools in the public scope. Only used for provisioning IP address CIDRs to pools in the public scope. Default is `byoip`.")
                .with_provider_name("PublicIpSource"),
        )
        .attribute(
            AttributeSchema::new("publicly_advertisable", AttributeType::bool())
                .create_only()
                .with_description("Determines whether or not address space from this pool is publicly advertised. Must be set if and only if the pool is IPv6.")
                .with_provider_name("PubliclyAdvertisable"),
        )
        .attribute(
            AttributeSchema::new("source_ipam_pool_id", carina_aws_types::ipam_pool_id())
                .create_only()
                .with_description("The Id of this pool's source. If set, all space provisioned in this pool must be free space provisioned in the parent pool.")
                .with_provider_name("SourceIpamPoolId"),
        )
        .attribute(
            AttributeSchema::new("source_resource", AttributeType::struct_("SourceResource".to_string(), vec![StructField::new("resource_id", AttributeType::string()).required().with_provider_name("ResourceId"),
                    StructField::new("resource_owner", AttributeType::string()).required().with_provider_name("ResourceOwner"),
                    StructField::new("resource_region", carina_aws_types::aws_region()).required().with_provider_name("ResourceRegion"),
                    StructField::new("resource_type", AttributeType::string()).required().with_provider_name("ResourceType")]))
                .create_only()
                .with_provider_name("SourceResource"),
        )
        .attribute(
            AttributeSchema::new("state", AttributeType::enum_(carina_core::schema::enum_identity("State", Some("aws.ec2.IpamPool")), Some(vec!["create-in-progress".to_string(), "create-complete".to_string(), "modify-in-progress".to_string(), "modify-complete".to_string(), "delete-in-progress".to_string(), "delete-complete".to_string()]), vec![("create-in-progress".to_string(), "create_in_progress".to_string()), ("create-complete".to_string(), "create_complete".to_string()), ("modify-in-progress".to_string(), "modify_in_progress".to_string()), ("modify-complete".to_string(), "modify_complete".to_string()), ("delete-in-progress".to_string(), "delete_in_progress".to_string()), ("delete-complete".to_string(), "delete_complete".to_string())], None, None))
                .read_only()
                .with_description("The state of this pool. This can be one of the following values: \"create-in-progress\", \"create-complete\", \"modify-in-progress\", \"modify-complete\", \"delete-in-progress\", or \"delete-complete\" (read-only)")
                .with_provider_name("State"),
        )
        .attribute(
            AttributeSchema::new("state_message", AttributeType::string())
                .read_only()
                .with_description("An explanation of how the pool arrived at it current state. (read-only)")
                .with_provider_name("StateMessage"),
        )
        .attribute(
            AttributeSchema::new("tags", carina_aws_types::tags_type())
                .with_description("An array of key-value pairs to apply to this resource.")
                .with_provider_name("Tags")
                .with_block_name("tag"),
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
        .with_def("SourceResource", AttributeType::struct_("SourceResource".to_string(), vec![StructField::new("resource_id", AttributeType::string()).required().with_provider_name("ResourceId"),
                    StructField::new("resource_owner", AttributeType::string()).required().with_provider_name("ResourceOwner"),
                    StructField::new("resource_region", carina_aws_types::aws_region()).required().with_provider_name("ResourceRegion"),
                    StructField::new("resource_type", AttributeType::string()).required().with_provider_name("ResourceType")]))
    }
}

/// Returns the resource type name and all enum valid values for this module
pub fn enum_valid_values() -> (
    &'static str,
    &'static [(&'static str, &'static [&'static str])],
) {
    (
        "ec2.IpamPool",
        &[
            ("address_family", VALID_ADDRESS_FAMILY),
            ("aws_service", VALID_AWS_SERVICE),
            ("ipam_scope_type", VALID_IPAM_SCOPE_TYPE),
            ("public_ip_source", VALID_PUBLIC_IP_SOURCE),
            ("state", VALID_STATE),
        ],
    )
}
