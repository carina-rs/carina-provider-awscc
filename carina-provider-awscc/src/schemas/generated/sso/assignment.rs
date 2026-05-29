//! assignment schema definition for AWS Cloud Control
//!
//! Auto-generated from CloudFormation schema: AWS::SSO::Assignment
//!
//! DO NOT EDIT MANUALLY - regenerate with carina-codegen

use super::AwsccSchemaConfig;
use carina_core::schema::{AttributeSchema, AttributeType, ResourceSchema};

const VALID_PRINCIPAL_TYPE: &[&str] = &["USER", "GROUP"];

const VALID_TARGET_TYPE: &[&str] = &["AWS_ACCOUNT"];

/// Returns the schema config for sso_assignment (AWS::SSO::Assignment)
pub fn sso_assignment_config() -> AwsccSchemaConfig {
    AwsccSchemaConfig {
        aws_type_name: "AWS::SSO::Assignment",
        resource_type_name: "sso.Assignment",
        has_tags: false,
        schema: ResourceSchema::new("sso.Assignment")
            .with_description("Resource Type definition for SSO assignmet")
            .attribute(
                AttributeSchema::new("instance_arn", super::sso_instance_arn())
                    .required()
                    .create_only()
                    .with_description("The sso instance that the permission set is owned.")
                    .with_provider_name("InstanceArn"),
            )
            .attribute(
                AttributeSchema::new("permission_set_arn", super::sso_permission_set_arn())
                    .required()
                    .create_only()
                    .with_description("The permission set that the assignment will be assigned")
                    .with_provider_name("PermissionSetArn"),
            )
            .attribute(
                AttributeSchema::new("principal_id", super::sso_principal_id())
                    .required()
                    .create_only()
                    .with_description("The assignee's identifier, user id/group id")
                    .with_provider_name("PrincipalId"),
            )
            .attribute(
                AttributeSchema::new(
                    "principal_type",
                    AttributeType::string_enum(
                        "PrincipalType".to_string(),
                        vec!["USER".to_string(), "GROUP".to_string()],
                        Some(carina_core::schema::string_enum_identity(
                            "PrincipalType",
                            Some("awscc.sso.Assignment"),
                        )),
                        vec![
                            ("USER".to_string(), "user".to_string()),
                            ("GROUP".to_string(), "group".to_string()),
                        ],
                    ),
                )
                .required()
                .create_only()
                .with_description("The assignee's type, user/group")
                .with_provider_name("PrincipalType"),
            )
            .attribute(
                AttributeSchema::new("target_id", super::aws_account_id())
                    .required()
                    .create_only()
                    .with_description("The account id to be provisioned.")
                    .with_provider_name("TargetId"),
            )
            .attribute(
                AttributeSchema::new(
                    "target_type",
                    AttributeType::string_enum(
                        "TargetType".to_string(),
                        vec!["AWS_ACCOUNT".to_string()],
                        Some(carina_core::schema::string_enum_identity(
                            "TargetType",
                            Some("awscc.sso.Assignment"),
                        )),
                        vec![("AWS_ACCOUNT".to_string(), "aws_account".to_string())],
                    ),
                )
                .required()
                .create_only()
                .with_description("The type of resource to be provisioned to, only aws account now")
                .with_provider_name("TargetType"),
            ),
    }
}

/// Returns the resource type name and all enum valid values for this module
pub fn enum_valid_values() -> (
    &'static str,
    &'static [(&'static str, &'static [&'static str])],
) {
    (
        "sso.Assignment",
        &[
            ("principal_type", VALID_PRINCIPAL_TYPE),
            ("target_type", VALID_TARGET_TYPE),
        ],
    )
}
