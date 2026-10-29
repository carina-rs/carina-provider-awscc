//! instance schema definition for AWS Cloud Control
//!
//! Auto-generated from CloudFormation schema: AWS::SSO::Instance
//!
//! DO NOT EDIT MANUALLY - regenerate with carina-codegen

use crate::schemas::config::{AwsccSchemaConfig, TagsProperty, TagsWire};
use carina_core::schema::{AttributeSchema, AttributeType, ResourceSchema};

const VALID_STATUS: &[&str] = &["CREATE_IN_PROGRESS", "DELETE_IN_PROGRESS", "ACTIVE"];

/// Returns the schema config for sso_instance (AWS::SSO::Instance)
#[rustfmt::skip]
pub fn sso_instance_config() -> AwsccSchemaConfig {
    let mut schema = ResourceSchema::new("sso.Instance")
        .with_description("Resource Type definition for Identity Center (SSO) Instance");
    #[inline(never)]
    fn build_identity_store_id_attribute() -> AttributeSchema {
        AttributeSchema::provider_populated("identity_store_id", carina_aws_types::identity_store_id())
                .with_description("The ID of the identity store associated with the created Identity Center (SSO) Instance (read-only)")
                .with_provider_name("IdentityStoreId")
            .build()
    }
    schema = schema.attribute(build_identity_store_id_attribute());
    #[inline(never)]
    fn build_instance_arn_attribute() -> AttributeSchema {
        AttributeSchema::provider_populated("instance_arn", carina_aws_types::sso_instance_arn())
                .with_description("The SSO Instance ARN that is returned upon creation of the Identity Center (SSO) Instance (read-only)")
                .with_provider_name("InstanceArn")
            .build()
    }
    schema = schema.attribute(build_instance_arn_attribute());
    #[inline(never)]
    fn build_name_attribute() -> AttributeSchema {
        AttributeSchema::optional("name", AttributeType::refined_string(None, Some("^[\\w+=,.@-]+$".to_string()), Some((Some(1), Some(32))), None))
                .with_description("The name you want to assign to this Identity Center (SSO) Instance")
                .with_provider_name("Name")
            .build()
    }
    schema = schema.attribute(build_name_attribute());
    #[inline(never)]
    fn build_owner_account_id_attribute() -> AttributeSchema {
        AttributeSchema::provider_populated("owner_account_id", carina_aws_types::aws_account_id())
                .with_description("The AWS accountId of the owner of the Identity Center (SSO) Instance (read-only)")
                .with_provider_name("OwnerAccountId")
            .build()
    }
    schema = schema.attribute(build_owner_account_id_attribute());
    #[inline(never)]
    fn build_status_attribute() -> AttributeSchema {
        AttributeSchema::provider_populated("status", AttributeType::enum_(carina_core::schema::enum_identity("Status", Some("aws.sso.Instance")), Some(vec!["CREATE_IN_PROGRESS".to_string(), "DELETE_IN_PROGRESS".to_string(), "ACTIVE".to_string()]), vec![("CREATE_IN_PROGRESS".to_string(), "create_in_progress".to_string()), ("DELETE_IN_PROGRESS".to_string(), "delete_in_progress".to_string()), ("ACTIVE".to_string(), "active".to_string())], None, None))
                .with_description("The status of the Identity Center (SSO) Instance, create_in_progress/delete_in_progress/active (read-only)")
                .with_provider_name("Status")
            .build()
    }
    schema = schema.attribute(build_status_attribute());
    #[inline(never)]
    fn build_tags_attribute() -> AttributeSchema {
        AttributeSchema::optional("tags", carina_aws_types::tags_type())
                .with_provider_name("Tags")
                .with_block_name("tag")
            .build()
    }
    schema = schema.attribute(build_tags_attribute());
    schema = schema
        .with_conflicting_replacement();
    schema = schema.with_declared_validators(vec![carina_provider_protocol::types::ValidatorType::TagsKeyValueCheck]);
    AwsccSchemaConfig {
        aws_type_name: "AWS::SSO::Instance",
        resource_type_name: "sso.Instance",
        primary_identifier: &[crate::schemas::config::PrimaryIdentifierAttribute { provider_name: "InstanceArn", dsl_name: "instance_arn" }],
        tags: Some(TagsProperty { provider_name: "Tags", wire: TagsWire::KeyValueList }),
        schema,
    }
}

/// Returns the resource type name and all enum valid values for this module
pub fn enum_valid_values() -> (
    &'static str,
    &'static [(&'static str, &'static [&'static str])],
) {
    ("sso.Instance", &[("status", VALID_STATUS)])
}

/// Returns the IAM permissions declared by the CloudFormation handler for this operation.
pub fn required_permissions(op: carina_core::effect::PlanOp) -> &'static [&'static str] {
    match op {
        carina_core::effect::PlanOp::Create => &[
            "sso:CreateInstance",
            "sso:DescribeInstance",
            "sso:TagResource",
            "iam:CreateServiceLinkedRole",
            "sso:TagInstance",
            "sso:ListTagsForResource",
            "identitystore:CreateIdentityStore",
        ],
        carina_core::effect::PlanOp::Read => &["sso:DescribeInstance", "sso:ListTagsForResource"],
        carina_core::effect::PlanOp::Update => &[
            "sso:UpdateInstance",
            "sso:TagResource",
            "sso:UntagResource",
            "sso:ListTagsForResource",
            "sso:TagInstance",
            "sso:DescribeInstance",
        ],
        carina_core::effect::PlanOp::Delete => {
            &["sso:DeleteInstance", "identitystore:DeleteIdentityStore"]
        }
    }
}
