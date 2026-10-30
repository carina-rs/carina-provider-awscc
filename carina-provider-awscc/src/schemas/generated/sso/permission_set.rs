//! permission_set schema definition for AWS Cloud Control
//!
//! Auto-generated from CloudFormation schema: AWS::SSO::PermissionSet
//!
//! DO NOT EDIT MANUALLY - regenerate with carina-codegen

use crate::schemas::config::{AwsccSchemaConfig, TagsProperty, TagsWire};
use carina_core::schema::{AttributeSchema, AttributeType, ResourceSchema, StructField};

/// Returns the schema config for sso_permission_set (AWS::SSO::PermissionSet)
#[rustfmt::skip]
pub fn sso_permission_set_config() -> AwsccSchemaConfig {
    let mut schema = ResourceSchema::new("sso.PermissionSet")
        .with_description("Resource Type definition for SSO PermissionSet");
    #[inline(never)]
    fn build_customer_managed_policy_references_attribute() -> AttributeSchema {
        AttributeSchema::optional("customer_managed_policy_references", AttributeType::refined_list(AttributeType::struct_("CustomerManagedPolicyReference".to_string(), vec![StructField::required("name", AttributeType::refined_string(None, Some("[\\w+=,.@-]+".to_string()), Some((Some(1), Some(128))), None)).with_provider_name("Name").build(),
                    StructField::optional("path", AttributeType::refined_string(None, Some("((/[A-Za-z0-9\\.,\\+@=_-]+)*)/".to_string()), Some((Some(1), Some(512))), None)).with_provider_name("Path").build()]), false, Some((None, Some(20)))))
                .with_provider_name("CustomerManagedPolicyReferences")
                .with_block_name("customer_managed_policy_reference")
            .build()
    }
    schema = schema.attribute(build_customer_managed_policy_references_attribute());
    #[inline(never)]
    fn build_description_attribute() -> AttributeSchema {
        AttributeSchema::optional("description", AttributeType::refined_string(None, Some("[\\u0009\\u000A\\u000D\\u0020-\\u007E\\u00A1-\\u00FF]*".to_string()), Some((Some(1), Some(700))), None))
                .with_description("The permission set description.")
                .with_provider_name("Description")
            .build()
    }
    schema = schema.attribute(build_description_attribute());
    #[inline(never)]
    fn build_inline_policy_attribute() -> AttributeSchema {
        AttributeSchema::optional("inline_policy", carina_aws_types::iam_policy_document())
                .with_description("The inline policy to put in permission set.")
                .with_provider_name("InlinePolicy")
            .build()
    }
    schema = schema.attribute(build_inline_policy_attribute());
    #[inline(never)]
    fn build_instance_arn_attribute() -> AttributeSchema {
        AttributeSchema::required("instance_arn", carina_aws_types::sso_instance_arn())
                .create_only()
                .with_description("The sso instance arn that the permission set is owned.")
                .with_provider_name("InstanceArn")
            .build()
    }
    schema = schema.attribute(build_instance_arn_attribute());
    #[inline(never)]
    fn build_managed_policies_attribute() -> AttributeSchema {
        AttributeSchema::optional("managed_policies", AttributeType::refined_list(AttributeType::string(), false, Some((None, Some(20)))))
                .with_provider_name("ManagedPolicies")
            .build()
    }
    schema = schema.attribute(build_managed_policies_attribute());
    #[inline(never)]
    fn build_name_attribute() -> AttributeSchema {
        AttributeSchema::required("name", AttributeType::refined_string(None, Some("[\\w+=,.@-]+".to_string()), Some((Some(1), Some(32))), None))
                .create_only()
                .with_description("The name you want to assign to this permission set.")
                .with_provider_name("Name")
            .build()
    }
    schema = schema.attribute(build_name_attribute());
    #[inline(never)]
    fn build_permission_set_arn_attribute() -> AttributeSchema {
        AttributeSchema::provider_populated("permission_set_arn", carina_aws_types::sso_permission_set_arn())
                .with_description("The permission set that the policy will be attached to (read-only)")
                .with_provider_name("PermissionSetArn")
            .build()
    }
    schema = schema.attribute(build_permission_set_arn_attribute());
    #[inline(never)]
    fn build_permissions_boundary_attribute() -> AttributeSchema {
        AttributeSchema::optional("permissions_boundary", AttributeType::struct_("PermissionsBoundary".to_string(), vec![StructField::optional("customer_managed_policy_reference", AttributeType::struct_("CustomerManagedPolicyReference".to_string(), vec![StructField::required("name", AttributeType::refined_string(None, Some("[\\w+=,.@-]+".to_string()), Some((Some(1), Some(128))), None)).with_provider_name("Name").build(),
                    StructField::optional("path", AttributeType::refined_string(None, Some("((/[A-Za-z0-9\\.,\\+@=_-]+)*)/".to_string()), Some((Some(1), Some(512))), None)).with_provider_name("Path").build()])).with_provider_name("CustomerManagedPolicyReference").build(),
                    StructField::optional("managed_policy_arn", carina_aws_types::arn()).with_provider_name("ManagedPolicyArn").build()]))
                .with_provider_name("PermissionsBoundary")
            .build()
    }
    schema = schema.attribute(build_permissions_boundary_attribute());
    #[inline(never)]
    fn build_relay_state_type_attribute() -> AttributeSchema {
        AttributeSchema::optional("relay_state_type", AttributeType::refined_string(None, Some("[a-zA-Z0-9&amp;$@#\\/%?=~\\-_'&quot;|!:,.;*+\\[\\]\\ \\(\\)\\{\\}]+".to_string()), Some((Some(1), Some(240))), None))
                .with_description("The relay state URL that redirect links to any service in the AWS Management Console.")
                .with_provider_name("RelayStateType")
            .build()
    }
    schema = schema.attribute(build_relay_state_type_attribute());
    #[inline(never)]
    fn build_session_duration_attribute() -> AttributeSchema {
        AttributeSchema::optional("session_duration", AttributeType::refined_string(None, Some("^(-?)P(?=\\d|T\\d)(?:(\\d+)Y)?(?:(\\d+)M)?(?:(\\d+)([DW]))?(?:T(?:(\\d+)H)?(?:(\\d+)M)?(?:(\\d+(?:\\.\\d+)?)S)?)?$".to_string()), Some((Some(1), Some(100))), None))
                .with_description("The length of time that a user can be signed in to an AWS account.")
                .with_provider_name("SessionDuration")
            .build()
    }
    schema = schema.attribute(build_session_duration_attribute());
    #[inline(never)]
    fn build_tags_attribute() -> AttributeSchema {
        AttributeSchema::optional("tags", carina_aws_types::tags_type())
                .with_provider_name("Tags")
                .with_block_name("tag")
            .build()
    }
    schema = schema.attribute(build_tags_attribute());
    schema = schema
        .with_unique_name_attribute("name");
    schema = schema.with_declared_validators(vec![carina_provider_protocol::types::ValidatorType::TagsKeyValueCheck]);
    #[inline(never)]
    fn build_schema_def_0() -> AttributeType { AttributeType::struct_("CustomerManagedPolicyReference".to_string(), vec![StructField::required("name", AttributeType::refined_string(None, Some("[\\w+=,.@-]+".to_string()), Some((Some(1), Some(128))), None)).with_provider_name("Name").build(),
                    StructField::optional("path", AttributeType::refined_string(None, Some("((/[A-Za-z0-9\\.,\\+@=_-]+)*)/".to_string()), Some((Some(1), Some(512))), None)).with_provider_name("Path").build()]) }
    schema = schema.with_def("CustomerManagedPolicyReference", build_schema_def_0());
    #[inline(never)]
    fn build_schema_def_1() -> AttributeType { AttributeType::struct_("PermissionsBoundary".to_string(), vec![StructField::optional("customer_managed_policy_reference", AttributeType::struct_("CustomerManagedPolicyReference".to_string(), vec![StructField::required("name", AttributeType::refined_string(None, Some("[\\w+=,.@-]+".to_string()), Some((Some(1), Some(128))), None)).with_provider_name("Name").build(),
                    StructField::optional("path", AttributeType::refined_string(None, Some("((/[A-Za-z0-9\\.,\\+@=_-]+)*)/".to_string()), Some((Some(1), Some(512))), None)).with_provider_name("Path").build()])).with_provider_name("CustomerManagedPolicyReference").build(),
                    StructField::optional("managed_policy_arn", carina_aws_types::arn()).with_provider_name("ManagedPolicyArn").build()]) }
    schema = schema.with_def("PermissionsBoundary", build_schema_def_1());
    AwsccSchemaConfig {
        aws_type_name: "AWS::SSO::PermissionSet",
        resource_type_name: "sso.PermissionSet",
        primary_identifier: &[crate::schemas::config::PrimaryIdentifierAttribute { provider_name: "InstanceArn", dsl_name: "instance_arn" }, crate::schemas::config::PrimaryIdentifierAttribute { provider_name: "PermissionSetArn", dsl_name: "permission_set_arn" }],
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
    ("sso.PermissionSet", &[])
}

/// Returns the IAM permissions declared by the CloudFormation handler for this operation.
pub fn required_permissions(op: carina_core::effect::PlanOp) -> &'static [&'static str] {
    match op {
        carina_core::effect::PlanOp::Create => &[
            "sso:CreatePermissionSet",
            "sso:PutInlinePolicyToPermissionSet",
            "sso:AttachManagedPolicyToPermissionSet",
            "sso:AttachCustomerManagedPolicyReferenceToPermissionSet",
            "sso:PutPermissionsBoundaryToPermissionSet",
            "sso:TagResource",
            "sso:DescribePermissionSet",
            "sso:ListTagsForResource",
            "sso:ListManagedPoliciesInPermissionSet",
            "sso:ListCustomerManagedPolicyReferencesInPermissionSet",
            "sso:GetInlinePolicyForPermissionSet",
            "sso:GetPermissionsBoundaryForPermissionSet",
        ],
        carina_core::effect::PlanOp::Read => &[
            "sso:DescribePermissionSet",
            "sso:ListTagsForResource",
            "sso:ListManagedPoliciesInPermissionSet",
            "sso:ListCustomerManagedPolicyReferencesInPermissionSet",
            "sso:GetInlinePolicyForPermissionSet",
            "sso:GetPermissionsBoundaryForPermissionSet",
        ],
        carina_core::effect::PlanOp::Update => &[
            "sso:UpdatePermissionSet",
            "sso:TagResource",
            "sso:UntagResource",
            "sso:ListTagsForResource",
            "sso:AttachManagedPolicyToPermissionSet",
            "sso:AttachCustomerManagedPolicyReferenceToPermissionSet",
            "sso:DetachManagedPolicyFromPermissionSet",
            "sso:DetachCustomerManagedPolicyReferenceFromPermissionSet",
            "sso:ListManagedPoliciesInPermissionSet",
            "sso:ListCustomerManagedPolicyReferencesInPermissionSet",
            "sso:PutInlinePolicyToPermissionSet",
            "sso:GetPermissionsBoundaryForPermissionSet",
            "sso:DeletePermissionsBoundaryFromPermissionSet",
            "sso:PutPermissionsBoundaryToPermissionSet",
            "sso:DeleteInlinePolicyFromPermissionSet",
            "sso:ProvisionPermissionSet",
            "sso:DescribePermissionSet",
            "sso:GetInlinePolicyForPermissionSet",
            "sso:DescribePermissionSetProvisioningStatus",
        ],
        carina_core::effect::PlanOp::Delete => &["sso:DeletePermissionSet"],
    }
}
