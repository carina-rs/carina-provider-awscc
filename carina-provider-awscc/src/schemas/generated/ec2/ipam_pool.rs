//! ipam_pool schema definition for AWS Cloud Control
//!
//! Auto-generated from CloudFormation schema: AWS::EC2::IPAMPool
//!
//! DO NOT EDIT MANUALLY - regenerate with carina-codegen

use crate::schemas::config::{AwsccSchemaConfig, TagsProperty, TagsWire};
use carina_core::resource::{ConcreteValue, Value};
use carina_core::schema::{
    AttributeSchema, AttributeType, OperationConfig, ResourceSchema, StructField, legacy_validator,
    types,
};

pub fn arn() -> AttributeType {
    AttributeType::refined_string_with_validator(
        carina_aws_types::provider_type("ec2", "IpamPool", "Arn"),
        Some("^arn:(aws|aws-cn|aws-us-gov):ec2:.*$".to_string()),
        None,
        legacy_validator(|value| {
            if let Value::Concrete(ConcreteValue::String(s)) = value {
                let s = s.expose_plaintext();
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
#[rustfmt::skip]
pub fn ec2_ipam_pool_config() -> AwsccSchemaConfig {
    let mut schema = ResourceSchema::new("ec2.IpamPool")
        .with_description("Resource Schema of AWS::EC2::IPAMPool Type");
    #[inline(never)]
    fn build_address_family_attribute() -> AttributeSchema {
        AttributeSchema::required("address_family", AttributeType::enum_(carina_core::schema::enum_identity("AddressFamily", Some("aws.ec2.IpamPool")), Some(vec!["IPv4".to_string(), "IPv6".to_string()]), vec![("IPv4".to_string(), "ipv4".to_string()), ("IPv6".to_string(), "ipv6".to_string())], None, None))
                .create_only()
                .with_description("The address family of the address space in this pool. Either IPv4 or IPv6.")
                .with_provider_name("AddressFamily")
            .build()
    }
    schema = schema.attribute(build_address_family_attribute());
    #[inline(never)]
    fn build_allocation_default_netmask_length_attribute() -> AttributeSchema {
        AttributeSchema::optional("allocation_default_netmask_length", AttributeType::int())
                .with_description("The default netmask length for allocations made from this pool. This value is used when the netmask length of an allocation isn't specified.")
                .with_provider_name("AllocationDefaultNetmaskLength")
            .build()
    }
    schema = schema.attribute(build_allocation_default_netmask_length_attribute());
    #[inline(never)]
    fn build_allocation_max_netmask_length_attribute() -> AttributeSchema {
        AttributeSchema::optional("allocation_max_netmask_length", AttributeType::int())
                .with_description("The maximum allowed netmask length for allocations made from this pool.")
                .with_provider_name("AllocationMaxNetmaskLength")
            .build()
    }
    schema = schema.attribute(build_allocation_max_netmask_length_attribute());
    #[inline(never)]
    fn build_allocation_min_netmask_length_attribute() -> AttributeSchema {
        AttributeSchema::optional("allocation_min_netmask_length", AttributeType::int())
                .with_description("The minimum allowed netmask length for allocations made from this pool.")
                .with_provider_name("AllocationMinNetmaskLength")
            .build()
    }
    schema = schema.attribute(build_allocation_min_netmask_length_attribute());
    #[inline(never)]
    fn build_allocation_resource_tags_attribute() -> AttributeSchema {
        AttributeSchema::optional("allocation_resource_tags", AttributeType::unordered_list(carina_aws_types::tags_type()))
                .with_description("When specified, an allocation will not be allowed unless a resource has a matching set of tags.")
                .with_provider_name("AllocationResourceTags")
                .with_block_name("allocation_resource_tag")
            .build()
    }
    schema = schema.attribute(build_allocation_resource_tags_attribute());
    #[inline(never)]
    fn build_arn_attribute() -> AttributeSchema {
        AttributeSchema::provider_populated("arn", self::arn())
                .with_description("The Amazon Resource Name (ARN) of the IPAM Pool. (read-only)")
                .with_provider_name("Arn")
            .build()
    }
    schema = schema.attribute(build_arn_attribute());
    #[inline(never)]
    fn build_auto_import_attribute() -> AttributeSchema {
        AttributeSchema::optional("auto_import", AttributeType::bool())
                .with_description("Determines what to do if IPAM discovers resources that haven't been assigned an allocation. If set to true, an allocation will be made automatically.")
                .with_provider_name("AutoImport")
            .build()
    }
    schema = schema.attribute(build_auto_import_attribute());
    #[inline(never)]
    fn build_aws_service_attribute() -> AttributeSchema {
        AttributeSchema::optional("aws_service", AttributeType::enum_(carina_core::schema::enum_identity("AwsService", Some("aws.ec2.IpamPool")), Some(vec!["ec2".to_string(), "global-services".to_string()]), vec![("ec2".to_string(), "ec2".to_string()), ("global-services".to_string(), "global_services".to_string())], None, None))
                .create_only()
                .with_description("Limits which service in Amazon Web Services that the pool can be used in.")
                .with_provider_name("AwsService")
            .build()
    }
    schema = schema.attribute(build_aws_service_attribute());
    #[inline(never)]
    fn build_description_attribute() -> AttributeSchema {
        AttributeSchema::optional("description", AttributeType::string())
                .with_provider_name("Description")
            .build()
    }
    schema = schema.attribute(build_description_attribute());
    #[inline(never)]
    fn build_ipam_arn_attribute() -> AttributeSchema {
        AttributeSchema::provider_populated("ipam_arn", carina_aws_types::arn())
                .with_description("The Amazon Resource Name (ARN) of the IPAM this pool is a part of. (read-only)")
                .with_provider_name("IpamArn")
            .build()
    }
    schema = schema.attribute(build_ipam_arn_attribute());
    #[inline(never)]
    fn build_ipam_pool_id_attribute() -> AttributeSchema {
        AttributeSchema::provider_populated("ipam_pool_id", carina_aws_types::ipam_pool_id())
                .with_description("Id of the IPAM Pool. (read-only)")
                .with_provider_name("IpamPoolId")
            .build()
    }
    schema = schema.attribute(build_ipam_pool_id_attribute());
    #[inline(never)]
    fn build_ipam_scope_arn_attribute() -> AttributeSchema {
        AttributeSchema::provider_populated("ipam_scope_arn", carina_aws_types::arn())
                .with_description("The Amazon Resource Name (ARN) of the scope this pool is a part of. (read-only)")
                .with_provider_name("IpamScopeArn")
            .build()
    }
    schema = schema.attribute(build_ipam_scope_arn_attribute());
    #[inline(never)]
    fn build_ipam_scope_id_attribute() -> AttributeSchema {
        AttributeSchema::required("ipam_scope_id", AttributeType::string())
                .create_only()
                .with_description("The Id of the scope this pool is a part of.")
                .with_provider_name("IpamScopeId")
            .build()
    }
    schema = schema.attribute(build_ipam_scope_id_attribute());
    #[inline(never)]
    fn build_ipam_scope_type_attribute() -> AttributeSchema {
        AttributeSchema::provider_populated("ipam_scope_type", AttributeType::enum_(carina_core::schema::enum_identity("IpamScopeType", Some("aws.ec2.IpamPool")), Some(vec!["public".to_string(), "private".to_string()]), vec![("public".to_string(), "public".to_string()), ("private".to_string(), "private".to_string())], None, None))
                .with_description("Determines whether this scope contains publicly routable space or space for a private network (read-only)")
                .with_provider_name("IpamScopeType")
            .build()
    }
    schema = schema.attribute(build_ipam_scope_type_attribute());
    #[inline(never)]
    fn build_locale_attribute() -> AttributeSchema {
        AttributeSchema::optional("locale", carina_aws_types::aws_region())
                .create_only()
                .with_description("The region of this pool. If not set, this will default to \"None\" which will disable non-custom allocations. If the locale has been specified for the source pool, this value must match.")
                .with_provider_name("Locale")
            .build()
    }
    schema = schema.attribute(build_locale_attribute());
    #[inline(never)]
    fn build_pool_depth_attribute() -> AttributeSchema {
        AttributeSchema::provider_populated("pool_depth", AttributeType::int())
                .with_description("The depth of this pool in the source pool hierarchy. (read-only)")
                .with_provider_name("PoolDepth")
            .build()
    }
    schema = schema.attribute(build_pool_depth_attribute());
    #[inline(never)]
    fn build_provisioned_cidrs_attribute() -> AttributeSchema {
        AttributeSchema::optional("provisioned_cidrs", AttributeType::unordered_list(AttributeType::struct_("ProvisionedCidr".to_string(), vec![StructField::required("cidr", types::cidr()).with_provider_name("Cidr").build()])))
                .with_description("A list of cidrs representing the address space available for allocation in this pool.")
                .with_provider_name("ProvisionedCidrs")
                .with_block_name("provisioned_cidr")
            .build()
    }
    schema = schema.attribute(build_provisioned_cidrs_attribute());
    #[inline(never)]
    fn build_public_ip_source_attribute() -> AttributeSchema {
        AttributeSchema::optional("public_ip_source", AttributeType::enum_(carina_core::schema::enum_identity("PublicIpSource", Some("aws.ec2.IpamPool")), Some(vec!["byoip".to_string(), "amazon".to_string()]), vec![("byoip".to_string(), "byoip".to_string()), ("amazon".to_string(), "amazon".to_string())], None, None))
                .create_only()
                .with_description("The IP address source for pools in the public scope. Only used for provisioning IP address CIDRs to pools in the public scope. Default is `byoip`.")
                .with_provider_name("PublicIpSource")
            .build()
    }
    schema = schema.attribute(build_public_ip_source_attribute());
    #[inline(never)]
    fn build_publicly_advertisable_attribute() -> AttributeSchema {
        AttributeSchema::optional("publicly_advertisable", AttributeType::bool())
                .create_only()
                .with_description("Determines whether or not address space from this pool is publicly advertised. Must be set if and only if the pool is IPv6.")
                .with_provider_name("PubliclyAdvertisable")
            .build()
    }
    schema = schema.attribute(build_publicly_advertisable_attribute());
    #[inline(never)]
    fn build_source_ipam_pool_id_attribute() -> AttributeSchema {
        AttributeSchema::optional("source_ipam_pool_id", carina_aws_types::ipam_pool_id())
                .create_only()
                .with_description("The Id of this pool's source. If set, all space provisioned in this pool must be free space provisioned in the parent pool.")
                .with_provider_name("SourceIpamPoolId")
            .build()
    }
    schema = schema.attribute(build_source_ipam_pool_id_attribute());
    #[inline(never)]
    fn build_source_resource_attribute() -> AttributeSchema {
        AttributeSchema::optional("source_resource", AttributeType::struct_("SourceResource".to_string(), vec![StructField::required("resource_id", AttributeType::string()).with_provider_name("ResourceId").build(),
                    StructField::required("resource_owner", AttributeType::string()).with_provider_name("ResourceOwner").build(),
                    StructField::required("resource_region", carina_aws_types::aws_region()).with_provider_name("ResourceRegion").build(),
                    StructField::required("resource_type", AttributeType::string()).with_provider_name("ResourceType").build()]))
                .create_only()
                .with_provider_name("SourceResource")
            .build()
    }
    schema = schema.attribute(build_source_resource_attribute());
    #[inline(never)]
    fn build_state_attribute() -> AttributeSchema {
        AttributeSchema::provider_populated("state", AttributeType::enum_(carina_core::schema::enum_identity("State", Some("aws.ec2.IpamPool")), Some(vec!["create-in-progress".to_string(), "create-complete".to_string(), "modify-in-progress".to_string(), "modify-complete".to_string(), "delete-in-progress".to_string(), "delete-complete".to_string()]), vec![("create-in-progress".to_string(), "create_in_progress".to_string()), ("create-complete".to_string(), "create_complete".to_string()), ("modify-in-progress".to_string(), "modify_in_progress".to_string()), ("modify-complete".to_string(), "modify_complete".to_string()), ("delete-in-progress".to_string(), "delete_in_progress".to_string()), ("delete-complete".to_string(), "delete_complete".to_string())], None, None))
                .with_description("The state of this pool. This can be one of the following values: \"create-in-progress\", \"create-complete\", \"modify-in-progress\", \"modify-complete\", \"delete-in-progress\", or \"delete-complete\" (read-only)")
                .with_provider_name("State")
            .build()
    }
    schema = schema.attribute(build_state_attribute());
    #[inline(never)]
    fn build_state_message_attribute() -> AttributeSchema {
        AttributeSchema::provider_populated("state_message", AttributeType::string())
                .with_description("An explanation of how the pool arrived at it current state. (read-only)")
                .with_provider_name("StateMessage")
            .build()
    }
    schema = schema.attribute(build_state_message_attribute());
    #[inline(never)]
    fn build_tags_attribute() -> AttributeSchema {
        AttributeSchema::optional("tags", carina_aws_types::tags_type())
                .with_description("An array of key-value pairs to apply to this resource.")
                .with_provider_name("Tags")
                .with_block_name("tag")
            .build()
    }
    schema = schema.attribute(build_tags_attribute());
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
    #[inline(never)]
    fn build_schema_def_0() -> AttributeType { AttributeType::struct_("SourceResource".to_string(), vec![StructField::required("resource_id", AttributeType::string()).with_provider_name("ResourceId").build(),
                    StructField::required("resource_owner", AttributeType::string()).with_provider_name("ResourceOwner").build(),
                    StructField::required("resource_region", carina_aws_types::aws_region()).with_provider_name("ResourceRegion").build(),
                    StructField::required("resource_type", AttributeType::string()).with_provider_name("ResourceType").build()]) }
    schema = schema.with_def("SourceResource", build_schema_def_0());
    AwsccSchemaConfig {
        aws_type_name: "AWS::EC2::IPAMPool",
        resource_type_name: "ec2.IpamPool",
        primary_identifier: &[crate::schemas::config::PrimaryIdentifierAttribute { provider_name: "IpamPoolId", dsl_name: "ipam_pool_id" }],
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

/// Returns the IAM permissions declared by the CloudFormation handler for this operation.
pub fn required_permissions(op: carina_core::effect::PlanOp) -> &'static [&'static str] {
    match op {
        carina_core::effect::PlanOp::Create => &[
            "ec2:CreateIpamPool",
            "ec2:DescribeIpamPools",
            "ec2:ProvisionIpamPoolCidr",
            "ec2:GetIpamPoolCidrs",
            "ec2:CreateTags",
        ],
        carina_core::effect::PlanOp::Read => &["ec2:DescribeIpamPools", "ec2:GetIpamPoolCidrs"],
        carina_core::effect::PlanOp::Update => &[
            "ec2:ModifyIpamPool",
            "ec2:DescribeIpamPools",
            "ec2:GetIpamPoolCidrs",
            "ec2:ProvisionIpamPoolCidr",
            "ec2:DeprovisionIpamPoolCidr",
            "ec2:CreateTags",
            "ec2:DeleteTags",
        ],
        carina_core::effect::PlanOp::Delete => &[
            "ec2:DeleteIpamPool",
            "ec2:DescribeIpamPools",
            "ec2:GetIpamPoolCidrs",
            "ec2:DeprovisionIpamPoolCidr",
            "ec2:DeleteTags",
        ],
    }
}
