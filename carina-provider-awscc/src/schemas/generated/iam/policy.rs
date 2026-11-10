//! Auto-generated helper schema module for AWSCC IAM Policy identifiers
//!
//! DO NOT EDIT MANUALLY - regenerate with carina-codegen

use carina_core::schema::AttributeType;

pub fn arn() -> AttributeType {
    AttributeType::string_member(
        crate::schemas::string_identity::StringIdentity::IamPolicyArn.identity_layer(),
        carina_core::schema::StringMemberConstraints::new(None, None),
    )
}
