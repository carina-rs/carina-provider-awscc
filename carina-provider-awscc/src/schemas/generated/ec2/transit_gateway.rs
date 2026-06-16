//! transit_gateway schema definition for AWS Cloud Control
//!
//! Auto-generated from CloudFormation schema: AWS::EC2::TransitGateway
//!
//! DO NOT EDIT MANUALLY - regenerate with carina-codegen

use crate::schemas::config::AwsccSchemaConfig;
use carina_core::resource::{ConcreteValue, Value};
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

#[allow(dead_code)]
fn validate_amazon_side_asn_range(value: &Value) -> Result<(), String> {
    if let Value::Concrete(ConcreteValue::Int(n)) = value {
        if *n < 1 || *n > 4294967294 {
            Err(format!("Value {} is out of range 1..=4294967294", n))
        } else {
            Ok(())
        }
    } else {
        Err("Expected integer".to_string())
    }
}

/// Returns the schema config for ec2_transit_gateway (AWS::EC2::TransitGateway)
pub fn ec2_transit_gateway_config() -> AwsccSchemaConfig {
    AwsccSchemaConfig {
        aws_type_name: "AWS::EC2::TransitGateway",
        resource_type_name: "ec2.TransitGateway",
        primary_identifier: &["Id"],
        has_tags: true,
        schema: ResourceSchema::new("ec2.TransitGateway")
            .with_description("Resource Type definition for AWS::EC2::TransitGateway")
            .attribute(
                AttributeSchema::new(
                    "amazon_side_asn",
                    AttributeType::refined_int(None, Some((Some(1), Some(4294967294)))),
                )
                .create_only()
                .with_provider_name("AmazonSideAsn"),
            )
            .attribute(
                AttributeSchema::new(
                    "association_default_route_table_id",
                    carina_aws_types::tgw_route_table_id(),
                )
                .with_provider_name("AssociationDefaultRouteTableId"),
            )
            .attribute(
                AttributeSchema::new(
                    "auto_accept_shared_attachments",
                    AttributeType::enum_(
                        carina_core::schema::enum_identity(
                            "AutoAcceptSharedAttachments",
                            Some("aws.ec2.TransitGateway"),
                        ),
                        Some(vec!["enable".to_string(), "disable".to_string()]),
                        vec![
                            ("enable".to_string(), "enable".to_string()),
                            ("disable".to_string(), "disable".to_string()),
                        ],
                        None,
                        None,
                    ),
                )
                .with_provider_name("AutoAcceptSharedAttachments"),
            )
            .attribute(
                AttributeSchema::new(
                    "default_route_table_association",
                    AttributeType::enum_(
                        carina_core::schema::enum_identity(
                            "DefaultRouteTableAssociation",
                            Some("aws.ec2.TransitGateway"),
                        ),
                        Some(vec!["enable".to_string(), "disable".to_string()]),
                        vec![
                            ("enable".to_string(), "enable".to_string()),
                            ("disable".to_string(), "disable".to_string()),
                        ],
                        None,
                        None,
                    ),
                )
                .with_provider_name("DefaultRouteTableAssociation"),
            )
            .attribute(
                AttributeSchema::new(
                    "default_route_table_propagation",
                    AttributeType::enum_(
                        carina_core::schema::enum_identity(
                            "DefaultRouteTablePropagation",
                            Some("aws.ec2.TransitGateway"),
                        ),
                        Some(vec!["enable".to_string(), "disable".to_string()]),
                        vec![
                            ("enable".to_string(), "enable".to_string()),
                            ("disable".to_string(), "disable".to_string()),
                        ],
                        None,
                        None,
                    ),
                )
                .with_provider_name("DefaultRouteTablePropagation"),
            )
            .attribute(
                AttributeSchema::new("description", AttributeType::string())
                    .with_provider_name("Description"),
            )
            .attribute(
                AttributeSchema::new(
                    "dns_support",
                    AttributeType::enum_(
                        carina_core::schema::enum_identity(
                            "DnsSupport",
                            Some("aws.ec2.TransitGateway"),
                        ),
                        Some(vec!["enable".to_string(), "disable".to_string()]),
                        vec![
                            ("enable".to_string(), "enable".to_string()),
                            ("disable".to_string(), "disable".to_string()),
                        ],
                        None,
                        None,
                    ),
                )
                .with_provider_name("DnsSupport"),
            )
            .attribute(
                AttributeSchema::new(
                    "encryption_support",
                    AttributeType::enum_(
                        carina_core::schema::enum_identity(
                            "EncryptionSupport",
                            Some("aws.ec2.TransitGateway"),
                        ),
                        Some(vec!["disable".to_string(), "enable".to_string()]),
                        vec![
                            ("disable".to_string(), "disable".to_string()),
                            ("enable".to_string(), "enable".to_string()),
                        ],
                        None,
                        None,
                    ),
                )
                .write_only()
                .with_provider_name("EncryptionSupport"),
            )
            .attribute(
                AttributeSchema::new(
                    "encryption_support_state",
                    AttributeType::enum_(
                        carina_core::schema::enum_identity(
                            "EncryptionSupportState",
                            Some("aws.ec2.TransitGateway"),
                        ),
                        Some(vec!["disable".to_string(), "enable".to_string()]),
                        vec![
                            ("disable".to_string(), "disable".to_string()),
                            ("enable".to_string(), "enable".to_string()),
                        ],
                        None,
                        None,
                    ),
                )
                .read_only()
                .with_provider_name("EncryptionSupportState"),
            )
            .attribute(
                AttributeSchema::new("id", carina_aws_types::transit_gateway_id())
                    .read_only()
                    .with_provider_name("Id"),
            )
            .attribute(
                AttributeSchema::new(
                    "multicast_support",
                    AttributeType::enum_(
                        carina_core::schema::enum_identity(
                            "MulticastSupport",
                            Some("aws.ec2.TransitGateway"),
                        ),
                        Some(vec!["enable".to_string(), "disable".to_string()]),
                        vec![
                            ("enable".to_string(), "enable".to_string()),
                            ("disable".to_string(), "disable".to_string()),
                        ],
                        None,
                        None,
                    ),
                )
                .create_only()
                .with_provider_name("MulticastSupport"),
            )
            .attribute(
                AttributeSchema::new(
                    "propagation_default_route_table_id",
                    carina_aws_types::tgw_route_table_id(),
                )
                .with_provider_name("PropagationDefaultRouteTableId"),
            )
            .attribute(
                AttributeSchema::new(
                    "security_group_referencing_support",
                    AttributeType::enum_(
                        carina_core::schema::enum_identity(
                            "SecurityGroupReferencingSupport",
                            Some("aws.ec2.TransitGateway"),
                        ),
                        Some(vec!["enable".to_string(), "disable".to_string()]),
                        vec![
                            ("enable".to_string(), "enable".to_string()),
                            ("disable".to_string(), "disable".to_string()),
                        ],
                        None,
                        None,
                    ),
                )
                .with_provider_name("SecurityGroupReferencingSupport"),
            )
            .attribute(
                AttributeSchema::new("tags", carina_aws_types::tags_type())
                    .with_provider_name("Tags")
                    .with_block_name("tag"),
            )
            .attribute(
                AttributeSchema::new("transit_gateway_arn", carina_aws_types::arn())
                    .read_only()
                    .with_provider_name("TransitGatewayArn"),
            )
            .attribute(
                AttributeSchema::new(
                    "transit_gateway_cidr_blocks",
                    AttributeType::list(types::cidr()),
                )
                .with_provider_name("TransitGatewayCidrBlocks"),
            )
            .attribute(
                AttributeSchema::new(
                    "vpn_ecmp_support",
                    AttributeType::enum_(
                        carina_core::schema::enum_identity(
                            "VpnEcmpSupport",
                            Some("aws.ec2.TransitGateway"),
                        ),
                        Some(vec!["enable".to_string(), "disable".to_string()]),
                        vec![
                            ("enable".to_string(), "enable".to_string()),
                            ("disable".to_string(), "disable".to_string()),
                        ],
                        None,
                        None,
                    ),
                )
                .with_provider_name("VpnEcmpSupport"),
            )
            .with_operation_config(OperationConfig {
                delete_timeout_secs: Some(1800),
                delete_max_retries: Some(24),
                create_timeout_secs: None,
                create_max_retries: None,
            })
            .with_validator(|attrs| {
                let mut errors = Vec::new();
                if let Err(mut e) = carina_aws_types::validate_tags_map(attrs) {
                    errors.append(&mut e);
                }
                if errors.is_empty() {
                    Ok(())
                } else {
                    Err(errors)
                }
            }),
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
