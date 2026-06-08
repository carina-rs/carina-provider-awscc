//! Auto-generated helper schema module for AWSCC IAM Policy identifiers
//!
//! DO NOT EDIT MANUALLY - regenerate with carina-codegen

use carina_core::resource::{ConcreteValue, Value};
use carina_core::schema::{AttributeType, legacy_validator};

pub fn arn() -> AttributeType {
    AttributeType::custom(
        Some(carina_aws_types::provider_type("iam", "Policy", "Arn")),
        carina_aws_types::arn(),
        Some("^arn:(aws|aws-cn|aws-us-gov):iam::[^:]*:policy/.+$".to_string()),
        None,
        legacy_validator(|value| {
            if let Value::Concrete(ConcreteValue::String(s)) = value {
                carina_aws_types::validate_iam_arn(s, "policy/")
                    .map_err(|reason| format!("Invalid IAM Policy ARN '{}': {}", s, reason))
            } else {
                Err("Expected string".to_string())
            }
        }),
        None,
    )
}
