//! function schema definition for AWS Cloud Control
//!
//! Auto-generated from CloudFormation schema: AWS::CloudFront::Function
//!
//! DO NOT EDIT MANUALLY - regenerate with carina-codegen

use crate::schemas::config::{AwsccSchemaConfig, TagsProperty, TagsWire};
use carina_core::schema::{AttributeSchema, AttributeType, ResourceSchema, StructField};

const VALID_FUNCTION_CONFIG_RUNTIME: &[&str] = &["cloudfront-js-1.0", "cloudfront-js-2.0"];

/// Returns the schema config for cloudfront_function (AWS::CloudFront::Function)
#[rustfmt::skip]
pub fn cloudfront_function_config() -> AwsccSchemaConfig {
    let mut schema = ResourceSchema::new("cloudfront.Function")
        .with_description("Creates a CF function.  To create a function, you provide the function code and some configuration information about the function. The response contains an Amazon Resource Name (ARN) that uniquely identifies the function, and the function?s stage.  By default, when you create a function, it?s in the ``DEVELOPMENT`` stage. In this stage, you can [test the function](https://docs.aws.amazon.com/AmazonCloudFront/latest/DeveloperGuide/test-function.html) in the CF console (or with ``TestFunction`` in the CF API).  When you?re ready to use your function with a CF distribution, publish the function to the ``LIVE`` stage. You can do this in the CF console, with ``PublishFunction`` in the CF API, or by updating the ``AWS::CloudFront::Function`` resource with the ``AutoPublish`` property set to ``true``. When the function is published to the ``LIVE`` stage, you can attach it to a distribution?s cache behavior, using the function?s ARN.  To automatically publish the function to the ``LIVE`` stage when it?s created, set the ``AutoPublish`` property to ``true``.");
    #[inline(never)]
    fn build_auto_publish_attribute() -> AttributeSchema {
        AttributeSchema::optional("auto_publish", AttributeType::bool())
                .write_only()
                .with_description("A flag that determines whether to automatically publish the function to the ``LIVE`` stage when it?s created. To automatically publish to the ``LIVE`` stage, set this property to ``true``.")
                .with_provider_name("AutoPublish")
            .build()
    }
    schema = schema.attribute(build_auto_publish_attribute());
    #[inline(never)]
    fn build_function_arn_attribute() -> AttributeSchema {
        AttributeSchema::provider_populated("function_arn", carina_aws_types::arn())
                .with_description(" (read-only)")
                .with_provider_name("FunctionARN")
            .build()
    }
    schema = schema.attribute(build_function_arn_attribute());
    #[inline(never)]
    fn build_function_code_attribute() -> AttributeSchema {
        AttributeSchema::required("function_code", AttributeType::string())
                .with_description("The function code. For more information about writing a CloudFront function, see [Writing function code for CloudFront Functions](https://docs.aws.amazon.com/AmazonCloudFront/latest/DeveloperGuide/writing-function-code.html) in the *Amazon CloudFront Developer Guide*.")
                .with_provider_name("FunctionCode")
            .build()
    }
    schema = schema.attribute(build_function_code_attribute());
    #[inline(never)]
    fn build_function_config_attribute() -> AttributeSchema {
        AttributeSchema::required("function_config", AttributeType::struct_("FunctionConfig".to_string(), vec![StructField::required("comment", AttributeType::string()).with_description("A comment to describe the function. The comment cannot be longer than 128 characters.").with_provider_name("Comment").build(),
                    StructField::optional("key_value_store_associations", AttributeType::list(AttributeType::struct_("KeyValueStoreAssociation".to_string(), vec![StructField::required("key_value_store_arn", carina_aws_types::arn()).with_description("The Amazon Resource Name (ARN) of the key value store association.").with_provider_name("KeyValueStoreARN").build()]))).with_description("The configuration for the key value store associations.").with_provider_name("KeyValueStoreAssociations").with_block_name("key_value_store_association").build(),
                    StructField::required("runtime", AttributeType::enum_(carina_core::schema::enum_identity("Runtime", Some("aws.cloudfront.Function.FunctionConfig")), Some(vec!["cloudfront-js-1.0".to_string(), "cloudfront-js-2.0".to_string()]), vec![("cloudfront-js-1.0".to_string(), "cloudfront_js_1_0".to_string()), ("cloudfront-js-2.0".to_string(), "cloudfront_js_2_0".to_string())], None, None)).with_description("The function's runtime environment version.").with_provider_name("Runtime").build()]))
                .with_description("Contains configuration information about a CloudFront function.")
                .with_provider_name("FunctionConfig")
                .with_block_name("function_config")
            .build()
    }
    schema = schema.attribute(build_function_config_attribute());
    #[inline(never)]
    fn build_function_metadata_attribute() -> AttributeSchema {
        AttributeSchema::provider_populated("function_metadata", AttributeType::struct_("FunctionMetadata".to_string(), vec![StructField::provider_populated("function_arn", carina_aws_types::arn()).with_description("The Amazon Resource Name (ARN) of the function. The ARN uniquely identifies the function.").with_provider_name("FunctionARN").build()]))
                .with_description("Contains metadata about a CloudFront function. (read-only)")
                .with_provider_name("FunctionMetadata")
            .build()
    }
    schema = schema.attribute(build_function_metadata_attribute());
    #[inline(never)]
    fn build_name_attribute() -> AttributeSchema {
        AttributeSchema::required("name", AttributeType::string())
                .create_only()
                .with_description("A name to identify the function.")
                .with_provider_name("Name")
            .build()
    }
    schema = schema.attribute(build_name_attribute());
    #[inline(never)]
    fn build_stage_attribute() -> AttributeSchema {
        AttributeSchema::provider_populated("stage", AttributeType::string())
                .with_description(" (read-only)")
                .with_provider_name("Stage")
            .build()
    }
    schema = schema.attribute(build_stage_attribute());
    #[inline(never)]
    fn build_tags_attribute() -> AttributeSchema {
        AttributeSchema::optional("tags", carina_aws_types::tags_type())
                .with_description("A complex type that contains zero or more ``Tag`` elements.")
                .with_provider_name("Tags")
                .with_block_name("tag")
            .build()
    }
    schema = schema.attribute(build_tags_attribute());
    schema = schema
        .with_unique_name_attribute("name");
    schema = schema.with_declared_validators(vec![carina_provider_protocol::types::ValidatorType::TagsKeyValueCheck]);
    #[inline(never)]
    fn build_schema_def_0() -> AttributeType { AttributeType::struct_("FunctionConfig".to_string(), vec![StructField::required("comment", AttributeType::string()).with_description("A comment to describe the function. The comment cannot be longer than 128 characters.").with_provider_name("Comment").build(),
                    StructField::optional("key_value_store_associations", AttributeType::list(AttributeType::struct_("KeyValueStoreAssociation".to_string(), vec![StructField::required("key_value_store_arn", carina_aws_types::arn()).with_description("The Amazon Resource Name (ARN) of the key value store association.").with_provider_name("KeyValueStoreARN").build()]))).with_description("The configuration for the key value store associations.").with_provider_name("KeyValueStoreAssociations").with_block_name("key_value_store_association").build(),
                    StructField::required("runtime", AttributeType::enum_(carina_core::schema::enum_identity("Runtime", Some("aws.cloudfront.Function.FunctionConfig")), Some(vec!["cloudfront-js-1.0".to_string(), "cloudfront-js-2.0".to_string()]), vec![("cloudfront-js-1.0".to_string(), "cloudfront_js_1_0".to_string()), ("cloudfront-js-2.0".to_string(), "cloudfront_js_2_0".to_string())], None, None)).with_description("The function's runtime environment version.").with_provider_name("Runtime").build()]) }
    schema = schema.with_def("FunctionConfig", build_schema_def_0());
    #[inline(never)]
    fn build_schema_def_1() -> AttributeType { AttributeType::struct_("FunctionMetadata".to_string(), vec![StructField::provider_populated("function_arn", carina_aws_types::arn()).with_description("The Amazon Resource Name (ARN) of the function. The ARN uniquely identifies the function.").with_provider_name("FunctionARN").build()]) }
    schema = schema.with_def("FunctionMetadata", build_schema_def_1());
    AwsccSchemaConfig {
        aws_type_name: "AWS::CloudFront::Function",
        resource_type_name: "cloudfront.Function",
        primary_identifier: &[crate::schemas::config::PrimaryIdentifierAttribute { provider_name: "FunctionARN", dsl_name: "function_arn" }],
        tags: Some(TagsProperty { provider_name: "Tags", wire: TagsWire::KeyValueList }),
        schema,
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
