//! assignment schema definition for AWS Cloud Control
//!
//! Auto-generated from CloudFormation schema: AWS::SSO::Assignment
//!
//! DO NOT EDIT MANUALLY - regenerate with carina-codegen

use crate::schemas::config::AwsccSchemaConfig;
use carina_core::schema::{AttributeSchema, AttributeType, ResourceSchema};

const VALID_PRINCIPAL_TYPE: &[&str] = &["USER", "GROUP"];

const VALID_TARGET_TYPE: &[&str] = &["AWS_ACCOUNT"];

/// Returns the schema config for sso_assignment (AWS::SSO::Assignment)
pub fn sso_assignment_config() -> AwsccSchemaConfig {
    AwsccSchemaConfig {
        aws_type_name: "AWS::SSO::Assignment",
        resource_type_name: "sso.Assignment",
        primary_identifier: &[
            "InstanceArn",
            "TargetId",
            "TargetType",
            "PermissionSetArn",
            "PrincipalType",
            "PrincipalId",
        ],
        has_tags: false,
        schema: ResourceSchema::new("sso.Assignment")
            .with_description("Resource Type definition for SSO assignmet")
            .attribute(
                AttributeSchema::new("instance_arn", carina_aws_types::sso_instance_arn())
                    .required()
                    .create_only()
                    .with_description("The sso instance that the permission set is owned.")
                    .with_provider_name("InstanceArn"),
            )
            .attribute(
                AttributeSchema::new(
                    "permission_set_arn",
                    carina_aws_types::sso_permission_set_arn(),
                )
                .required()
                .create_only()
                .with_description("The permission set that the assignment will be assigned")
                .with_provider_name("PermissionSetArn"),
            )
            .attribute(
                AttributeSchema::new("principal_id", carina_aws_types::sso_principal_id())
                    .required()
                    .create_only()
                    .with_description("The assignee's identifier, user id/group id")
                    .with_provider_name("PrincipalId"),
            )
            .attribute(
                AttributeSchema::new(
                    "principal_type",
                    AttributeType::enum_(
                        carina_core::schema::enum_identity(
                            "PrincipalType",
                            Some("aws.sso.Assignment"),
                        ),
                        Some(vec!["USER".to_string(), "GROUP".to_string()]),
                        vec![
                            ("USER".to_string(), "user".to_string()),
                            ("GROUP".to_string(), "group".to_string()),
                        ],
                        None,
                        None,
                    ),
                )
                .required()
                .create_only()
                .with_description("The assignee's type, user/group")
                .with_provider_name("PrincipalType"),
            )
            .attribute(
                AttributeSchema::new("target_id", carina_aws_types::aws_account_id())
                    .required()
                    .create_only()
                    .with_description("The account id to be provisioned.")
                    .with_provider_name("TargetId"),
            )
            .attribute(
                AttributeSchema::new(
                    "target_type",
                    AttributeType::enum_(
                        carina_core::schema::enum_identity(
                            "TargetType",
                            Some("aws.sso.Assignment"),
                        ),
                        Some(vec!["AWS_ACCOUNT".to_string()]),
                        vec![("AWS_ACCOUNT".to_string(), "aws_account".to_string())],
                        None,
                        None,
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

/// Returns the IAM permissions declared by the CloudFormation handler for this operation.
pub fn required_permissions(op: carina_core::effect::PlanOp) -> &'static [&'static str] {
    match op {
        carina_core::effect::PlanOp::Create => &[
            "sso:CreateAccountAssignment",
            "sso:DescribeAccountAssignmentCreationStatus",
            "sso:ListAccountAssignments",
            "iam:GetSAMLProvider",
            "iam:CreateSAMLProvider",
            "iam:AttachRolePolicy",
            "iam:PutRolePolicy",
            "iam:CreateRole",
            "iam:ListRolePolicies",
        ],
        carina_core::effect::PlanOp::Read => &[
            "sso:ListAccountAssignments",
            "iam:GetSAMLProvider",
            "iam:ListRolePolicies",
        ],
        carina_core::effect::PlanOp::Update => &[],
        carina_core::effect::PlanOp::Delete => &[
            "sso:ListAccountAssignments",
            "sso:DeleteAccountAssignment",
            "sso:DescribeAccountAssignmentDeletionStatus",
            "iam:GetSAMLProvider",
            "iam:ListRolePolicies",
        ],
    }
}
