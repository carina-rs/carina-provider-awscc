//! function schema definition for AWS Cloud Control
//!
//! Auto-generated from CloudFormation schema: AWS::CloudFront::Function
//!
//! DO NOT EDIT MANUALLY - regenerate with carina-codegen

use crate::schemas::config::{AwsccSchemaConfig, TagsProperty, TagsWire};
use carina_core::resource::{ConcreteValue, Value};
use carina_core::schema::{AttributeSchema, AttributeType, ResourceSchema, StructField};
use regex::Regex;

const VALID_FUNCTION_CONFIG_RUNTIME: &[&str] = &["cloudfront-js-1.0", "cloudfront-js-2.0"];

/// Returns the schema config for cloudfront_function (AWS::CloudFront::Function)
pub fn cloudfront_function_config() -> AwsccSchemaConfig {
    AwsccSchemaConfig {
        aws_type_name: "AWS::CloudFront::Function",
        resource_type_name: "cloudfront.Function",
        primary_identifier: &[crate::schemas::config::PrimaryIdentifierAttribute { provider_name: "FunctionARN", dsl_name: "function_arn" }],
        tags: Some(TagsProperty { provider_name: "Tags", wire: TagsWire::KeyValueList }),
        schema: ResourceSchema::new("cloudfront.Function")
	        .with_description("Creates a CF function.  To create a function, you provide the function code and some configuration information about the function. The response contains an Amazon Resource Name (ARN) that uniquely identifies the function, and the function?s stage.  By default, when you create a function, it?s in the ``DEVELOPMENT`` stage. In this stage, you can [test the function](https://docs.aws.amazon.com/AmazonCloudFront/latest/DeveloperGuide/test-function.html) in the CF console (or with ``TestFunction`` in the CF API).  When you?re ready to use your function with a CF distribution, publish the function to the ``LIVE`` stage. You can do this in the CF console, with ``PublishFunction`` in the CF API, or by updating the ``AWS::CloudFront::Function`` resource with the ``AutoPublish`` property set to ``true``. When the function is published to the ``LIVE`` stage, you can attach it to a distribution?s cache behavior, using the function?s ARN.  To automatically publish the function to the ``LIVE`` stage when it?s created, set the ``AutoPublish`` property to ``true``.")
        .attribute(
            AttributeSchema::new("auto_publish", AttributeType::bool())
                .write_only()
                .with_description("A flag that determines whether to automatically publish the function to the ``LIVE`` stage when it?s created. To automatically publish to the ``LIVE`` stage, set this property to ``true``.")
                .with_provider_name("AutoPublish"),
        )
        .attribute(
            AttributeSchema::new("function_arn", carina_aws_types::arn())
                .read_only()
                .with_description(" (read-only)")
                .with_provider_name("FunctionARN"),
        )
        .attribute(
            AttributeSchema::new("function_code", AttributeType::string())
                .required()
                .with_description("The function code. For more information about writing a CloudFront function, see [Writing function code for CloudFront Functions](https://docs.aws.amazon.com/AmazonCloudFront/latest/DeveloperGuide/writing-function-code.html) in the *Amazon CloudFront Developer Guide*.")
                .with_provider_name("FunctionCode"),
        )
        .attribute(
            AttributeSchema::new("function_config", AttributeType::struct_("FunctionConfig".to_string(), vec![StructField::new("comment", AttributeType::string()).required().with_description("A comment to describe the function. The comment cannot be longer than 128 characters.").with_provider_name("Comment"),
                    StructField::new("key_value_store_associations", AttributeType::list(AttributeType::struct_("KeyValueStoreAssociation".to_string(), vec![StructField::new("key_value_store_arn", carina_aws_types::arn()).required().with_description("The Amazon Resource Name (ARN) of the key value store association.").with_provider_name("KeyValueStoreARN")]))).with_description("The configuration for the key value store associations.").with_provider_name("KeyValueStoreAssociations").with_block_name("key_value_store_association"),
                    StructField::new("runtime", AttributeType::enum_(carina_core::schema::enum_identity("Runtime", Some("aws.cloudfront.Function.FunctionConfig")), Some(vec!["cloudfront-js-1.0".to_string(), "cloudfront-js-2.0".to_string()]), vec![("cloudfront-js-1.0".to_string(), "cloudfront_js_1_0".to_string()), ("cloudfront-js-2.0".to_string(), "cloudfront_js_2_0".to_string())], None, None)).required().with_description("The function's runtime environment version.").with_provider_name("Runtime")]))
                .required()
                .with_description("Contains configuration information about a CloudFront function.")
                .with_provider_name("FunctionConfig")
                .with_block_name("function_config"),
        )
        .attribute(
            AttributeSchema::new("function_metadata", AttributeType::struct_("FunctionMetadata".to_string(), vec![StructField::new("function_arn", carina_aws_types::arn()).with_description("The Amazon Resource Name (ARN) of the function. The ARN uniquely identifies the function.").with_provider_name("FunctionARN")]))
                .with_description("Contains metadata about a CloudFront function.")
                .with_provider_name("FunctionMetadata"),
        )
        .attribute(
            AttributeSchema::new("name", AttributeType::string())
                .required()
                .create_only()
                .with_description("A name to identify the function.")
                .with_provider_name("Name"),
        )
        .attribute(
            AttributeSchema::new("stage", AttributeType::string())
                .read_only()
                .with_description(" (read-only)")
                .with_provider_name("Stage"),
        )
        .attribute(
            AttributeSchema::new("tags", carina_aws_types::tags_type())
                .with_description("A complex type that contains zero or more ``Tag`` elements.")
                .with_provider_name("Tags")
                .with_block_name("tag"),
        )
        .with_unique_name_attribute("name")
        .with_validator(|attrs| {
            let mut errors = Vec::new();
            if let Err(mut e) = carina_aws_types::validate_tags_map(attrs) {
                errors.append(&mut e);
            }
            if errors.is_empty() { Ok(()) } else { Err(errors) }
        })
        .with_def("FunctionConfig", AttributeType::struct_("FunctionConfig".to_string(), vec![StructField::new("comment", AttributeType::string()).required().with_description("A comment to describe the function. The comment cannot be longer than 128 characters.").with_provider_name("Comment"),
                    StructField::new("key_value_store_associations", AttributeType::list(AttributeType::struct_("KeyValueStoreAssociation".to_string(), vec![StructField::new("key_value_store_arn", carina_aws_types::arn()).required().with_description("The Amazon Resource Name (ARN) of the key value store association.").with_provider_name("KeyValueStoreARN")]))).with_description("The configuration for the key value store associations.").with_provider_name("KeyValueStoreAssociations").with_block_name("key_value_store_association"),
                    StructField::new("runtime", AttributeType::enum_(carina_core::schema::enum_identity("Runtime", Some("aws.cloudfront.Function.FunctionConfig")), Some(vec!["cloudfront-js-1.0".to_string(), "cloudfront-js-2.0".to_string()]), vec![("cloudfront-js-1.0".to_string(), "cloudfront_js_1_0".to_string()), ("cloudfront-js-2.0".to_string(), "cloudfront_js_2_0".to_string())], None, None)).required().with_description("The function's runtime environment version.").with_provider_name("Runtime")]))
        .with_def("FunctionMetadata", AttributeType::struct_("FunctionMetadata".to_string(), vec![StructField::new("function_arn", carina_aws_types::arn()).with_description("The Amazon Resource Name (ARN) of the function. The ARN uniquely identifies the function.").with_provider_name("FunctionARN")]))
    }
}

#[allow(dead_code)]
fn validate_string_pattern_6d706baa1fcf763c_len_max_256(value: &Value) -> Result<(), String> {
    if let Value::Concrete(ConcreteValue::String(s)) = value {
        let s = s.expose_plaintext();
        static RE: std::sync::LazyLock<Regex> = std::sync::LazyLock::new(|| {
            Regex::new("^([\\p{L}\\p{Z}\\p{N}_.:/=+\\-@]*)$").expect("invalid pattern regex")
        });
        if !RE.is_match(s) {
            return Err(format!(
                "Value '{}' does not match pattern ^([\\p{{L}}\\p{{Z}}\\p{{N}}_.:/=+\\-@]*)$",
                s
            ));
        }
        let len = s.chars().count();
        if len > 256 {
            return Err(format!("String length {} is out of range ..=256", len));
        }
        Ok(())
    } else {
        Err("Expected string".to_string())
    }
}

#[allow(dead_code)]
fn validate_string_pattern_6d706baa1fcf763c_len_1_128(value: &Value) -> Result<(), String> {
    if let Value::Concrete(ConcreteValue::String(s)) = value {
        let s = s.expose_plaintext();
        static RE: std::sync::LazyLock<Regex> = std::sync::LazyLock::new(|| {
            Regex::new("^([\\p{L}\\p{Z}\\p{N}_.:/=+\\-@]*)$").expect("invalid pattern regex")
        });
        if !RE.is_match(s) {
            return Err(format!(
                "Value '{}' does not match pattern ^([\\p{{L}}\\p{{Z}}\\p{{N}}_.:/=+\\-@]*)$",
                s
            ));
        }
        let len = s.chars().count();
        if !(1..=128).contains(&len) {
            return Err(format!("String length {} is out of range 1..=128", len));
        }
        Ok(())
    } else {
        Err("Expected string".to_string())
    }
}

/// Returns the resource type name and all enum valid values for this module
pub fn enum_valid_values() -> (
    &'static str,
    &'static [(&'static str, &'static [&'static str])],
) {
    (
        "cloudfront.Function",
        &[("runtime", VALID_FUNCTION_CONFIG_RUNTIME)],
    )
}

/// Returns the IAM permissions declared by the CloudFormation handler for this operation.
pub fn required_permissions(op: carina_core::effect::PlanOp) -> &'static [&'static str] {
    match op {
        carina_core::effect::PlanOp::Create => &[
            "cloudfront:CreateFunction",
            "cloudfront:PublishFunction",
            "cloudfront:DescribeFunction",
            "cloudfront:TagResource",
        ],
        carina_core::effect::PlanOp::Read => &[
            "cloudfront:DescribeFunction",
            "cloudfront:GetFunction",
            "cloudfront:ListTagsForResource",
        ],
        carina_core::effect::PlanOp::Update => &[
            "cloudfront:UpdateFunction",
            "cloudfront:PublishFunction",
            "cloudfront:DescribeFunction",
            "cloudfront:TagResource",
            "cloudfront:UntagResource",
        ],
        carina_core::effect::PlanOp::Delete => {
            &["cloudfront:DeleteFunction", "cloudfront:DescribeFunction"]
        }
    }
}
