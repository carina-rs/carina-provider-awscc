//! transit_gateway schema definition for AWS Cloud Control
//!
//! Auto-generated from CloudFormation schema: AWS::EC2::TransitGateway
//!
//! DO NOT EDIT MANUALLY - regenerate with carina-codegen

use crate::schemas::config::{AwsccSchemaConfig, TagsProperty, TagsWire};
use carina_core::schema::{AttributeSchema, AttributeType, OperationConfig, ResourceSchema, types};

const VALID_AUTO_ACCEPT_SHARED_ATTACHMENTS: &[&str] = &["enable", "disable"];

const VALID_DEFAULT_ROUTE_TABLE_ASSOCIATION: &[&str] = &["enable", "disable"];

const VALID_DEFAULT_ROUTE_TABLE_PROPAGATION: &[&str] = &["enable", "disable"];

const VALID_DNS_SUPPORT: &[&str] = &["enable", "disable"];

const VALID_ENCRYPTION_SUPPORT: &[&str] = &["disable", "enable"];

const VALID_ENCRYPTION_SUPPORT_STATE: &[&str] = &["disable", "enable"];

const VALID_MULTICAST_SUPPORT: &[&str] = &["enable", "disable"];

const VALID_SECURITY_GROUP_REFERENCING_SUPPORT: &[&str] = &["enable", "disable"];

const VALID_VPN_ECMP_SUPPORT: &[&str] = &["enable", "disable"];

/// Returns the schema config for ec2_transit_gateway (AWS::EC2::TransitGateway)
#[rustfmt::skip]
pub fn ec2_transit_gateway_config() -> AwsccSchemaConfig {
    let mut schema = ResourceSchema::new("ec2.TransitGateway")
        .with_description("Resource Type definition for AWS::EC2::TransitGateway");
    #[inline(never)]
    fn build_amazon_side_asn_attribute() -> AttributeSchema {
        AttributeSchema::optional("amazon_side_asn", AttributeType::refined_int(None, Some((Some(1), Some(4294967294)))))
                .create_only()
                .with_provider_name("AmazonSideAsn")
            .build()
    }
    schema = schema.attribute(build_amazon_side_asn_attribute());
    #[inline(never)]
    fn build_association_default_route_table_id_attribute() -> AttributeSchema {
        AttributeSchema::optional("association_default_route_table_id", carina_aws_types::tgw_route_table_id())
                .with_provider_name("AssociationDefaultRouteTableId")
            .build()
    }
    schema = schema.attribute(build_association_default_route_table_id_attribute());
    #[inline(never)]
    fn build_auto_accept_shared_attachments_attribute() -> AttributeSchema {
        AttributeSchema::optional("auto_accept_shared_attachments", AttributeType::enum_(carina_core::schema::enum_identity("AutoAcceptSharedAttachments", Some("aws.ec2.TransitGateway")), Some(vec!["enable".to_string(), "disable".to_string()]), vec![("enable".to_string(), "enable".to_string()), ("disable".to_string(), "disable".to_string())], None, None))
                .with_provider_name("AutoAcceptSharedAttachments")
            .build()
    }
    schema = schema.attribute(build_auto_accept_shared_attachments_attribute());
    #[inline(never)]
    fn build_default_route_table_association_attribute() -> AttributeSchema {
        AttributeSchema::optional("default_route_table_association", AttributeType::enum_(carina_core::schema::enum_identity("DefaultRouteTableAssociation", Some("aws.ec2.TransitGateway")), Some(vec!["enable".to_string(), "disable".to_string()]), vec![("enable".to_string(), "enable".to_string()), ("disable".to_string(), "disable".to_string())], None, None))
                .with_provider_name("DefaultRouteTableAssociation")
            .build()
    }
    schema = schema.attribute(build_default_route_table_association_attribute());
    #[inline(never)]
    fn build_default_route_table_propagation_attribute() -> AttributeSchema {
        AttributeSchema::optional("default_route_table_propagation", AttributeType::enum_(carina_core::schema::enum_identity("DefaultRouteTablePropagation", Some("aws.ec2.TransitGateway")), Some(vec!["enable".to_string(), "disable".to_string()]), vec![("enable".to_string(), "enable".to_string()), ("disable".to_string(), "disable".to_string())], None, None))
                .with_provider_name("DefaultRouteTablePropagation")
            .build()
    }
    schema = schema.attribute(build_default_route_table_propagation_attribute());
    #[inline(never)]
    fn build_description_attribute() -> AttributeSchema {
        AttributeSchema::optional("description", AttributeType::string())
                .with_provider_name("Description")
            .build()
    }
    schema = schema.attribute(build_description_attribute());
    #[inline(never)]
    fn build_dns_support_attribute() -> AttributeSchema {
        AttributeSchema::optional("dns_support", AttributeType::enum_(carina_core::schema::enum_identity("DnsSupport", Some("aws.ec2.TransitGateway")), Some(vec!["enable".to_string(), "disable".to_string()]), vec![("enable".to_string(), "enable".to_string()), ("disable".to_string(), "disable".to_string())], None, None))
                .with_provider_name("DnsSupport")
            .build()
    }
    schema = schema.attribute(build_dns_support_attribute());
    #[inline(never)]
    fn build_encryption_support_attribute() -> AttributeSchema {
        AttributeSchema::optional("encryption_support", AttributeType::enum_(carina_core::schema::enum_identity("EncryptionSupport", Some("aws.ec2.TransitGateway")), Some(vec!["disable".to_string(), "enable".to_string()]), vec![("disable".to_string(), "disable".to_string()), ("enable".to_string(), "enable".to_string())], None, None))
                .write_only()
                .with_provider_name("EncryptionSupport")
            .build()
    }
    schema = schema.attribute(build_encryption_support_attribute());
    #[inline(never)]
    fn build_encryption_support_state_attribute() -> AttributeSchema {
        AttributeSchema::provider_populated("encryption_support_state", AttributeType::enum_(carina_core::schema::enum_identity("EncryptionSupportState", Some("aws.ec2.TransitGateway")), Some(vec!["disable".to_string(), "enable".to_string()]), vec![("disable".to_string(), "disable".to_string()), ("enable".to_string(), "enable".to_string())], None, None))
                .with_provider_name("EncryptionSupportState")
            .build()
    }
    schema = schema.attribute(build_encryption_support_state_attribute());
    #[inline(never)]
    fn build_id_attribute() -> AttributeSchema {
        AttributeSchema::provider_populated("id", carina_aws_types::transit_gateway_id())
                .with_provider_name("Id")
            .build()
    }
    schema = schema.attribute(build_id_attribute());
    #[inline(never)]
    fn build_multicast_support_attribute() -> AttributeSchema {
        AttributeSchema::optional("multicast_support", AttributeType::enum_(carina_core::schema::enum_identity("MulticastSupport", Some("aws.ec2.TransitGateway")), Some(vec!["enable".to_string(), "disable".to_string()]), vec![("enable".to_string(), "enable".to_string()), ("disable".to_string(), "disable".to_string())], None, None))
                .create_only()
                .with_provider_name("MulticastSupport")
            .build()
    }
    schema = schema.attribute(build_multicast_support_attribute());
    #[inline(never)]
    fn build_propagation_default_route_table_id_attribute() -> AttributeSchema {
        AttributeSchema::optional("propagation_default_route_table_id", carina_aws_types::tgw_route_table_id())
                .with_provider_name("PropagationDefaultRouteTableId")
            .build()
    }
    schema = schema.attribute(build_propagation_default_route_table_id_attribute());
    #[inline(never)]
    fn build_security_group_referencing_support_attribute() -> AttributeSchema {
        AttributeSchema::optional("security_group_referencing_support", AttributeType::enum_(carina_core::schema::enum_identity("SecurityGroupReferencingSupport", Some("aws.ec2.TransitGateway")), Some(vec!["enable".to_string(), "disable".to_string()]), vec![("enable".to_string(), "enable".to_string()), ("disable".to_string(), "disable".to_string())], None, None))
                .with_provider_name("SecurityGroupReferencingSupport")
            .build()
    }
    schema = schema.attribute(build_security_group_referencing_support_attribute());
    #[inline(never)]
    fn build_tags_attribute() -> AttributeSchema {
        AttributeSchema::optional("tags", carina_aws_types::tags_type())
                .with_provider_name("Tags")
                .with_block_name("tag")
            .build()
    }
    schema = schema.attribute(build_tags_attribute());
    #[inline(never)]
    fn build_transit_gateway_arn_attribute() -> AttributeSchema {
        AttributeSchema::provider_populated("transit_gateway_arn", carina_aws_types::arn())
                .with_provider_name("TransitGatewayArn")
            .build()
    }
    schema = schema.attribute(build_transit_gateway_arn_attribute());
    #[inline(never)]
    fn build_transit_gateway_cidr_blocks_attribute() -> AttributeSchema {
        AttributeSchema::optional("transit_gateway_cidr_blocks", AttributeType::list(types::cidr()))
                .with_provider_name("TransitGatewayCidrBlocks")
            .build()
    }
    schema = schema.attribute(build_transit_gateway_cidr_blocks_attribute());
    #[inline(never)]
    fn build_vpn_ecmp_support_attribute() -> AttributeSchema {
        AttributeSchema::optional("vpn_ecmp_support", AttributeType::enum_(carina_core::schema::enum_identity("VpnEcmpSupport", Some("aws.ec2.TransitGateway")), Some(vec!["enable".to_string(), "disable".to_string()]), vec![("enable".to_string(), "enable".to_string()), ("disable".to_string(), "disable".to_string())], None, None))
                .with_provider_name("VpnEcmpSupport")
            .build()
    }
    schema = schema.attribute(build_vpn_ecmp_support_attribute());
    schema = schema
        .with_coexisting_replacement();
    schema = schema.with_operation_config(OperationConfig {
            delete_timeout_secs: Some(1800),
            delete_max_retries: Some(24),
            create_timeout_secs: None,
            create_max_retries: None,
            update_timeout_secs: None,
    });
    schema = schema.with_declared_validators(vec![carina_provider_protocol::types::ValidatorType::TagsKeyValueCheck]);
    AwsccSchemaConfig {
        aws_type_name: "AWS::EC2::TransitGateway",
        resource_type_name: "ec2.TransitGateway",
        primary_identifier: &[crate::schemas::config::PrimaryIdentifierAttribute { provider_name: "Id", dsl_name: "id" }],
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
        "ec2.TransitGateway",
        &[
            (
                "auto_accept_shared_attachments",
                VALID_AUTO_ACCEPT_SHARED_ATTACHMENTS,
            ),
            (
                "default_route_table_association",
                VALID_DEFAULT_ROUTE_TABLE_ASSOCIATION,
            ),
            (
                "default_route_table_propagation",
                VALID_DEFAULT_ROUTE_TABLE_PROPAGATION,
            ),
            ("dns_support", VALID_DNS_SUPPORT),
            ("encryption_support", VALID_ENCRYPTION_SUPPORT),
            ("encryption_support_state", VALID_ENCRYPTION_SUPPORT_STATE),
            ("multicast_support", VALID_MULTICAST_SUPPORT),
            (
                "security_group_referencing_support",
                VALID_SECURITY_GROUP_REFERENCING_SUPPORT,
            ),
            ("vpn_ecmp_support", VALID_VPN_ECMP_SUPPORT),
        ],
    )
}

/// Returns the IAM permissions declared by the CloudFormation handler for this operation.
pub fn required_permissions(op: carina_core::effect::PlanOp) -> &'static [&'static str] {
    match op {
        carina_core::effect::PlanOp::Create => &[
            "ec2:CreateTransitGateway",
            "ec2:CreateTags",
            "ec2:DescribeTransitGateways",
            "ec2:DescribeTags",
            "ec2:ModifyTransitGateway",
            "ec2:ModifyTransitGatewayOptions",
        ],
        carina_core::effect::PlanOp::Read => &["ec2:DescribeTransitGateways", "ec2:DescribeTags"],
        carina_core::effect::PlanOp::Update => &[
            "ec2:CreateTransitGateway",
            "ec2:CreateTags",
            "ec2:DescribeTransitGateways",
            "ec2:DescribeTags",
            "ec2:DeleteTransitGateway",
            "ec2:DeleteTags",
            "ec2:ModifyTransitGateway",
            "ec2:ModifyTransitGatewayOptions",
        ],
        carina_core::effect::PlanOp::Delete => &[
            "ec2:DescribeTransitGateways",
            "ec2:DescribeTags",
            "ec2:DeleteTransitGateway",
            "ec2:DeleteTags",
        ],
    }
}
