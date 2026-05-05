//! role_policy schema definition for AWS Cloud Control
//!
//! Auto-generated from CloudFormation schema: AWS::IAM::RolePolicy
//!
//! DO NOT EDIT MANUALLY - regenerate with carina-codegen

use super::AwsccSchemaConfig;
use carina_core::schema::{AttributeSchema, AttributeType, ResourceSchema};

/// Returns the schema config for iam_role_policy (AWS::IAM::RolePolicy)
pub fn iam_role_policy_config() -> AwsccSchemaConfig {
    AwsccSchemaConfig {
        aws_type_name: "AWS::IAM::RolePolicy",
        resource_type_name: "iam.RolePolicy",
        has_tags: false,
        schema: ResourceSchema::new("iam.RolePolicy")
        .with_description("Adds or updates an inline policy document that is embedded in the specified IAM role.  When you embed an inline policy in a role, the inline policy is used as part of the role's access (permissions) policy. The role's trust policy is created at the same time as the role, using [CreateRole](https://docs.aws.amazon.com/IAM/latest/APIReference/API_CreateRole.html). You can update a role's trust policy using [UpdateAssumeRolePolicy](https://docs.aws.amazon.com/IAM/latest/APIReference/API_UpdateAssumeRolePolicy.html). For information about roles, see [roles](https://docs.aws.amazon.com/IAM/latest/UserGuide/roles-toplevel.html) in the *IAM User Guide*.  A role can also have a managed policy attached to it. To attach a managed policy to a role, use [AWS::IAM::Role](https://docs.aws.amazon.com/AWSCloudFormation/latest/UserGuide/aws-resource-iam-role.html). To create a new managed policy, use [AWS::IAM::ManagedPolicy](https://docs.aws.amazon.com/AWSCloudFormation/latest/UserGuide/aws-resource-iam-managedpolicy.html). For information about policies, see [Managed policies and inline policies](https://docs.aws.amazon.com/IAM/latest/UserGuide/policies-managed-vs-inline.html) in the *IAM User Guide*.  For information about the maximum number of inline policies that you can embed with a role, see [IAM and quotas](https://docs.aws.amazon.com/IAM/latest/UserGuide/reference_iam-quotas.html) in the *IAM User Guide*.")
        .attribute(
            AttributeSchema::new("policy_document", super::iam_policy_document())
                .with_description("The policy document. You must provide policies in JSON format in IAM. However, for CFN templates formatted in YAML, you can provide the policy in JSON or YAML format. CFN always converts a YAML policy to JSON format before submitting it to IAM. The [regex pattern](https://docs.aws.amazon.com/http://wikipedia.org/wiki/regex) used to validate this parameter is a string of characters consisting of the following: + Any printable ASCII character ranging from the space character (``\\u0020``) through the end of the ASCII character range + The printable characters in the Basic Latin and Latin-1 Supplement character set (through ``\\u00FF``) + The special characters tab (``\\u0009``), line feed (``\\u000A``), and carriage return (``\\u000D``)")
                .with_provider_name("PolicyDocument"),
        )
        .attribute(
            AttributeSchema::new("policy_name", AttributeType::String)
                .required()
                .create_only()
                .with_description("The name of the policy document. This parameter allows (through its [regex pattern](https://docs.aws.amazon.com/http://wikipedia.org/wiki/regex)) a string of characters consisting of upper and lowercase alphanumeric characters with no spaces. You can also include any of the following characters: _+=,.@-")
                .with_provider_name("PolicyName"),
        )
        .attribute(
            AttributeSchema::new("role_name", AttributeType::String)
                .required()
                .create_only()
                .with_description("The name of the role to associate the policy with. This parameter allows (through its [regex pattern](https://docs.aws.amazon.com/http://wikipedia.org/wiki/regex)) a string of characters consisting of upper and lowercase alphanumeric characters with no spaces. You can also include any of the following characters: _+=,.@-")
                .with_provider_name("RoleName"),
        )
    }
}

/// Returns the resource type name and all enum valid values for this module
pub fn enum_valid_values() -> (
    &'static str,
    &'static [(&'static str, &'static [&'static str])],
) {
    ("iam.RolePolicy", &[])
}

/// Maps DSL alias values back to canonical AWS values for this module.
/// e.g., ("ip_protocol", "all") -> Some("-1")
pub fn enum_alias_reverse(attr_name: &str, value: &str) -> Option<&'static str> {
    let _ = (attr_name, value);
    None
}

/// Returns all enum alias entries as (attr_name, alias, canonical) tuples.
pub fn enum_alias_entries() -> &'static [(&'static str, &'static str, &'static str)] {
    &[]
}
