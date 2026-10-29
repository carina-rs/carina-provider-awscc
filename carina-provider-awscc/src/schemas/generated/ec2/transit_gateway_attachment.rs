//! transit_gateway_attachment schema definition for AWS Cloud Control
//!
//! Auto-generated from CloudFormation schema: AWS::EC2::TransitGatewayAttachment
//!
//! DO NOT EDIT MANUALLY - regenerate with carina-codegen

use crate::schemas::config::{AwsccSchemaConfig, TagsProperty, TagsWire};
use carina_core::schema::{
    AttributeSchema, AttributeType, OperationConfig, ResourceSchema, StructField,
};

/// Returns the schema config for ec2_transit_gateway_attachment (AWS::EC2::TransitGatewayAttachment)
pub fn ec2_transit_gateway_attachment_config() -> AwsccSchemaConfig {
    AwsccSchemaConfig {
        aws_type_name: "AWS::EC2::TransitGatewayAttachment",
        resource_type_name: "ec2.TransitGatewayAttachment",
        primary_identifier: &[crate::schemas::config::PrimaryIdentifierAttribute { provider_name: "Id", dsl_name: "id" }],
        tags: Some(TagsProperty { provider_name: "Tags", wire: TagsWire::KeyValueList }),
        schema: ResourceSchema::new("ec2.TransitGatewayAttachment")
	        .with_description("Resource Type definition for AWS::EC2::TransitGatewayAttachment")
        .attribute(
            AttributeSchema::provider_populated("id", carina_aws_types::transit_gateway_attachment_id())
                .with_provider_name("Id")
                .build(),
        )
        .attribute(
            AttributeSchema::optional("options", AttributeType::struct_("Options".to_string(), vec![StructField::optional("appliance_mode_support", AttributeType::enum_(carina_core::schema::enum_identity("ApplianceModeSupport", Some("aws.ec2.TransitGatewayAttachment.Options")), Some(vec!["enable".to_string(), "disable".to_string()]), vec![("enable".to_string(), "enable".to_string()), ("disable".to_string(), "disable".to_string())], None, None)).with_description("Indicates whether to enable Ipv6 Support for Vpc Attachment. Valid Values: enable | disable").with_provider_name("ApplianceModeSupport").build(),
                    StructField::optional("dns_support", AttributeType::enum_(carina_core::schema::enum_identity("DnsSupport", Some("aws.ec2.TransitGatewayAttachment.Options")), Some(vec!["enable".to_string(), "disable".to_string()]), vec![("enable".to_string(), "enable".to_string()), ("disable".to_string(), "disable".to_string())], None, None)).with_description("Indicates whether to enable DNS Support for Vpc Attachment. Valid Values: enable | disable").with_provider_name("DnsSupport").build(),
                    StructField::optional("ipv6_support", AttributeType::enum_(carina_core::schema::enum_identity("Ipv6Support", Some("aws.ec2.TransitGatewayAttachment.Options")), Some(vec!["enable".to_string(), "disable".to_string()]), vec![("enable".to_string(), "enable".to_string()), ("disable".to_string(), "disable".to_string())], None, None)).with_description("Indicates whether to enable Ipv6 Support for Vpc Attachment. Valid Values: enable | disable").with_provider_name("Ipv6Support").build(),
                    StructField::optional("security_group_referencing_support", AttributeType::enum_(carina_core::schema::enum_identity("SecurityGroupReferencingSupport", Some("aws.ec2.TransitGatewayAttachment.Options")), Some(vec!["enable".to_string(), "disable".to_string()]), vec![("enable".to_string(), "enable".to_string()), ("disable".to_string(), "disable".to_string())], None, None)).with_description("Indicates whether to enable Security Group referencing support for Vpc Attachment. Valid Values: enable | disable").with_provider_name("SecurityGroupReferencingSupport").build()]))
                .with_description("The options for the transit gateway vpc attachment.")
                .with_provider_name("Options")
                .build(),
        )
        .attribute(
            AttributeSchema::required("subnet_ids", AttributeType::unordered_list(carina_aws_types::subnet_id()))
                .with_provider_name("SubnetIds")
                .build(),
        )
        .attribute(
            AttributeSchema::optional("tags", carina_aws_types::tags_type())
                .with_provider_name("Tags")
                .with_block_name("tag")
                .build(),
        )
        .attribute(
            AttributeSchema::required("transit_gateway_id", carina_aws_types::transit_gateway_id())
                .create_only()
                .with_provider_name("TransitGatewayId")
                .build(),
        )
        .attribute(
            AttributeSchema::required("vpc_id", carina_aws_types::vpc_id())
                .create_only()
                .with_provider_name("VpcId")
                .build(),
        )
        .with_coexisting_replacement()
        .with_operation_config(OperationConfig {
            delete_timeout_secs: Some(1800),
            delete_max_retries: Some(24),
            create_timeout_secs: None,
            create_max_retries: None,
            update_timeout_secs: None,
        })
        .with_declared_validators(vec![carina_provider_protocol::types::ValidatorType::TagsKeyValueCheck])
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
