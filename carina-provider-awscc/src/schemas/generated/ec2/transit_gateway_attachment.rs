//! transit_gateway_attachment schema definition for AWS Cloud Control
//!
//! Auto-generated from CloudFormation schema: AWS::EC2::TransitGatewayAttachment
//!
//! DO NOT EDIT MANUALLY - regenerate with carina-codegen

use crate::schemas::config::AwsccSchemaConfig;
use carina_core::schema::{
    AttributeSchema, AttributeType, OperationConfig, ResourceSchema, StructField,
};

/// Returns the schema config for ec2_transit_gateway_attachment (AWS::EC2::TransitGatewayAttachment)
pub fn ec2_transit_gateway_attachment_config() -> AwsccSchemaConfig {
    AwsccSchemaConfig {
        aws_type_name: "AWS::EC2::TransitGatewayAttachment",
        resource_type_name: "ec2.TransitGatewayAttachment",
        primary_identifier: &["Id"],
        has_tags: true,
        schema: ResourceSchema::new("ec2.TransitGatewayAttachment")
	        .with_description("Resource Type definition for AWS::EC2::TransitGatewayAttachment")
        .attribute(
            AttributeSchema::new("id", carina_aws_types::transit_gateway_attachment_id())
                .read_only()
                .with_provider_name("Id"),
        )
        .attribute(
            AttributeSchema::new("options", AttributeType::struct_("Options".to_string(), vec![StructField::new("appliance_mode_support", AttributeType::enum_(carina_core::schema::enum_identity("ApplianceModeSupport", Some("aws.ec2.TransitGatewayAttachment.Options")), Some(vec!["enable".to_string(), "disable".to_string()]), vec![("enable".to_string(), "enable".to_string()), ("disable".to_string(), "disable".to_string())], None, None)).with_description("Indicates whether to enable Ipv6 Support for Vpc Attachment. Valid Values: enable | disable").with_provider_name("ApplianceModeSupport"),
                    StructField::new("dns_support", AttributeType::enum_(carina_core::schema::enum_identity("DnsSupport", Some("aws.ec2.TransitGatewayAttachment.Options")), Some(vec!["enable".to_string(), "disable".to_string()]), vec![("enable".to_string(), "enable".to_string()), ("disable".to_string(), "disable".to_string())], None, None)).with_description("Indicates whether to enable DNS Support for Vpc Attachment. Valid Values: enable | disable").with_provider_name("DnsSupport"),
                    StructField::new("ipv6_support", AttributeType::enum_(carina_core::schema::enum_identity("Ipv6Support", Some("aws.ec2.TransitGatewayAttachment.Options")), Some(vec!["enable".to_string(), "disable".to_string()]), vec![("enable".to_string(), "enable".to_string()), ("disable".to_string(), "disable".to_string())], None, None)).with_description("Indicates whether to enable Ipv6 Support for Vpc Attachment. Valid Values: enable | disable").with_provider_name("Ipv6Support"),
                    StructField::new("security_group_referencing_support", AttributeType::enum_(carina_core::schema::enum_identity("SecurityGroupReferencingSupport", Some("aws.ec2.TransitGatewayAttachment.Options")), Some(vec!["enable".to_string(), "disable".to_string()]), vec![("enable".to_string(), "enable".to_string()), ("disable".to_string(), "disable".to_string())], None, None)).with_description("Indicates whether to enable Security Group referencing support for Vpc Attachment. Valid Values: enable | disable").with_provider_name("SecurityGroupReferencingSupport")]))
                .with_description("The options for the transit gateway vpc attachment.")
                .with_provider_name("Options"),
        )
        .attribute(
            AttributeSchema::new("subnet_ids", AttributeType::unordered_list(carina_aws_types::subnet_id()))
                .required()
                .with_provider_name("SubnetIds"),
        )
        .attribute(
            AttributeSchema::new("tags", carina_aws_types::tags_type())
                .with_provider_name("Tags")
                .with_block_name("tag"),
        )
        .attribute(
            AttributeSchema::new("transit_gateway_id", carina_aws_types::transit_gateway_id())
                .required()
                .create_only()
                .with_provider_name("TransitGatewayId"),
        )
        .attribute(
            AttributeSchema::new("vpc_id", carina_aws_types::vpc_id())
                .required()
                .create_only()
                .with_provider_name("VpcId"),
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
            if errors.is_empty() { Ok(()) } else { Err(errors) }
        })
    }
}

/// Returns the resource type name and all enum valid values for this module
pub fn enum_valid_values() -> (
    &'static str,
    &'static [(&'static str, &'static [&'static str])],
) {
    ("ec2.TransitGatewayAttachment", &[])
}

/// Returns the IAM permissions declared by the CloudFormation handler for this operation.
pub fn required_permissions(op: carina_core::effect::PlanOp) -> &'static [&'static str] {
    match op {
        carina_core::effect::PlanOp::Create => &[
            "ec2:DescribeTransitGatewayVpcAttachments",
            "ec2:CreateTransitGatewayVpcAttachment",
            "ec2:CreateTags",
            "ec2:DescribeTags",
        ],
        carina_core::effect::PlanOp::Read => &[
            "ec2:DescribeTransitGatewayVpcAttachments",
            "ec2:DescribeTags",
        ],
        carina_core::effect::PlanOp::Update => &[
            "ec2:DescribeTransitGatewayVpcAttachments",
            "ec2:DescribeTags",
            "ec2:CreateTransitGatewayVpcAttachment",
            "ec2:CreateTags",
            "ec2:DeleteTransitGatewayVpcAttachment",
            "ec2:DeleteTags",
            "ec2:ModifyTransitGatewayVpcAttachment",
        ],
        carina_core::effect::PlanOp::Delete => &[
            "ec2:DescribeTransitGatewayVpcAttachments",
            "ec2:DeleteTransitGatewayVpcAttachment",
            "ec2:DeleteTags",
            "ec2:DescribeTags",
        ],
    }
}
