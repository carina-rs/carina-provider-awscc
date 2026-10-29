//! oidc_provider schema definition for AWS Cloud Control
//!
//! Auto-generated from CloudFormation schema: AWS::IAM::OIDCProvider
//!
//! DO NOT EDIT MANUALLY - regenerate with carina-codegen

use crate::schemas::config::{AwsccSchemaConfig, TagsProperty, TagsWire};
use carina_core::schema::{AttributeSchema, AttributeType, ResourceSchema};

pub fn arn() -> AttributeType {
    crate::schemas::catalog::iam_oidc_provider_arn()
}

/// Returns the schema config for iam_oidc_provider (AWS::IAM::OIDCProvider)
#[rustfmt::skip]
pub fn iam_oidc_provider_config() -> AwsccSchemaConfig {
    let mut schema = ResourceSchema::new("iam.OidcProvider")
        .with_description("Resource Type definition for AWS::IAM::OIDCProvider");
    #[inline(never)]
    fn build_arn_attribute() -> AttributeSchema {
        AttributeSchema::provider_populated("arn", self::arn())
                .with_description("Amazon Resource Name (ARN) of the OIDC provider (read-only)")
                .with_provider_name("Arn")
            .build()
    }
    schema = schema.attribute(build_arn_attribute());
    #[inline(never)]
    fn build_client_id_list_attribute() -> AttributeSchema {
        AttributeSchema::optional("client_id_list", AttributeType::unordered_list(AttributeType::refined_string(None, None, Some((Some(1), Some(255))), None)))
                .with_provider_name("ClientIdList")
            .build()
    }
    schema = schema.attribute(build_client_id_list_attribute());
    #[inline(never)]
    fn build_tags_attribute() -> AttributeSchema {
        AttributeSchema::optional("tags", carina_aws_types::tags_type())
                .with_provider_name("Tags")
                .with_block_name("tag")
            .build()
    }
    schema = schema.attribute(build_tags_attribute());
    #[inline(never)]
    fn build_thumbprint_list_attribute() -> AttributeSchema {
        AttributeSchema::optional("thumbprint_list", AttributeType::refined_list(AttributeType::refined_string(None, Some("[0-9A-Fa-f]{40}".to_string()), Some((Some(40), Some(40))), None), false, Some((None, Some(5)))))
                .with_provider_name("ThumbprintList")
            .build()
    }
    schema = schema.attribute(build_thumbprint_list_attribute());
    #[inline(never)]
    fn build_url_attribute() -> AttributeSchema {
        AttributeSchema::optional("url", AttributeType::refined_string(None, None, Some((Some(1), Some(255))), None))
                .create_only()
                .with_provider_name("Url")
            .build()
    }
    schema = schema.attribute(build_url_attribute());
    schema = schema
        .with_coexisting_replacement();
    schema = schema.with_declared_validators(vec![carina_provider_protocol::types::ValidatorType::TagsKeyValueCheck]);
    AwsccSchemaConfig {
        aws_type_name: "AWS::IAM::OIDCProvider",
        resource_type_name: "iam.OidcProvider",
        primary_identifier: &[crate::schemas::config::PrimaryIdentifierAttribute { provider_name: "Arn", dsl_name: "arn" }],
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
    ("iam.OidcProvider", &[])
}

/// Returns the IAM permissions declared by the CloudFormation handler for this operation.
pub fn required_permissions(op: carina_core::effect::PlanOp) -> &'static [&'static str] {
    match op {
        carina_core::effect::PlanOp::Create => &[
            "iam:CreateOpenIDConnectProvider",
            "iam:TagOpenIDConnectProvider",
            "iam:GetOpenIDConnectProvider",
        ],
        carina_core::effect::PlanOp::Read => &["iam:GetOpenIDConnectProvider"],
        carina_core::effect::PlanOp::Update => &[
            "iam:UpdateOpenIDConnectProviderThumbprint",
            "iam:RemoveClientIDFromOpenIDConnectProvider",
            "iam:AddClientIDToOpenIDConnectProvider",
            "iam:GetOpenIDConnectProvider",
            "iam:TagOpenIDConnectProvider",
            "iam:UntagOpenIDConnectProvider",
            "iam:ListOpenIDConnectProviderTags",
        ],
        carina_core::effect::PlanOp::Delete => &["iam:DeleteOpenIDConnectProvider"],
    }
}
