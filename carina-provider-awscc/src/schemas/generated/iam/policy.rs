//! Auto-generated helper schema module for AWSCC IAM Policy identifiers
//!
//! DO NOT EDIT MANUALLY - regenerate with carina-codegen

use carina_core::schema::AttributeType;

pub fn arn() -> AttributeType {
    AttributeType::refined_string_with_validator(
        crate::schemas::string_identity::StringIdentity::IamPolicyArn.identity(),
        None,
        None,
        crate::schemas::string_identity::StringIdentity::IamPolicyArn
            .check()
            .expect("shared string check"),
        None,
    )
}
