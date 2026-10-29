//! parameter schema definition for AWS Cloud Control
//!
//! Auto-generated from CloudFormation schema: AWS::SSM::Parameter
//!
//! DO NOT EDIT MANUALLY - regenerate with carina-codegen

use crate::schemas::config::{AwsccSchemaConfig, TagsProperty, TagsWire};
use carina_core::schema::{AttributeSchema, AttributeType, ResourceSchema};

pub fn arn() -> AttributeType {
    carina_aws_types::arn()
}

const VALID_DATA_TYPE: &[&str] = &["text", "aws:ec2:image"];

const VALID_TIER: &[&str] = &["Standard", "Advanced", "Intelligent-Tiering"];

const VALID_TYPE: &[&str] = &["String", "StringList"];

/// Returns the schema config for ssm_parameter (AWS::SSM::Parameter)
#[rustfmt::skip]
pub fn ssm_parameter_config() -> AwsccSchemaConfig {
    let mut schema = ResourceSchema::new("ssm.Parameter")
        .with_description("The ``AWS::SSM::Parameter`` resource creates an SSM parameter in SYSlong Parameter Store.   To create an SSM parameter, you must have the IAMlong (IAM) permissions ``ssm:PutParameter`` and ``ssm:AddTagsToResource``. On stack creation, CFNlong adds the following three tags to the parameter: ``aws:cloudformation:stack-name``, ``aws:cloudformation:logical-id``, and ``aws:cloudformation:stack-id``, in addition to any custom tags you specify.  To add, update, or remove tags during stack update, you must have IAM permissions for both ``ssm:AddTagsToResource`` and ``ssm:RemoveTagsFromResource``. For more information, see [Managing access using policies](https://docs.aws.amazon.com/systems-manager/latest/userguide/security-iam.html#security_iam_access-manage) in the *User Guide*.   For information about valid values for parameters, see [About requirements and constraints for parameter names](https://docs.aws.amazon.com/systems-manager/latest/userguide/sysman-paramstore-su-create.html#sysman-parameter-name-constraints) in the *User Guide* and [PutParameter](https://docs.aws.amazon.com/systems-manager/latest/APIReference/API_PutParameter.html) in the *API Reference*.");
    #[inline(never)]
    fn build_allowed_pattern_attribute() -> AttributeSchema {
        AttributeSchema::optional("allowed_pattern", AttributeType::string())
                .write_only()
                .with_description("A regular expression used to validate the parameter value. For example, for ``String`` types with values restricted to numbers, you can specify the following: ``AllowedPattern=^\\d+$``")
                .with_provider_name("AllowedPattern")
            .build()
    }
    schema = schema.attribute(build_allowed_pattern_attribute());
    #[inline(never)]
    fn build_arn_attribute() -> AttributeSchema {
        AttributeSchema::provider_populated("arn", self::arn())
                .with_description(" (read-only)")
                .with_provider_name("Arn")
            .build()
    }
    schema = schema.attribute(build_arn_attribute());
    #[inline(never)]
    fn build_data_type_attribute() -> AttributeSchema {
        AttributeSchema::optional("data_type", AttributeType::enum_(carina_core::schema::enum_identity("DataType", Some("aws.ssm.Parameter")), Some(vec!["text".to_string(), "aws:ec2:image".to_string()]), vec![("text".to_string(), "text".to_string()), ("aws:ec2:image".to_string(), "aws_ec2_image".to_string())], None, None))
                .with_description("The data type of the parameter, such as ``text`` or ``aws:ec2:image``. The default is ``text``.")
                .with_provider_name("DataType")
            .build()
    }
    schema = schema.attribute(build_data_type_attribute());
    #[inline(never)]
    fn build_description_attribute() -> AttributeSchema {
        AttributeSchema::optional("description", AttributeType::string())
                .write_only()
                .with_description("Information about the parameter.")
                .with_provider_name("Description")
            .build()
    }
    schema = schema.attribute(build_description_attribute());
    #[inline(never)]
    fn build_name_attribute() -> AttributeSchema {
        AttributeSchema::optional("name", AttributeType::string())
                .create_only()
                .with_description("The name of the parameter. The reported maximum length of 2048 characters for a parameter name includes 1037 characters that are reserved for internal use by SYS. The maximum length for a parameter name that you specify is 1011 characters. This count of 1011 characters includes the characters in the ARN that precede the name you specify. This ARN length will vary depending on your partition and Region. For example, the following 45 characters count toward the 1011 character maximum for a parameter created in the US East (Ohio) Region: ``arn:aws:ssm:us-east-2:111122223333:parameter/``.")
                .with_provider_name("Name")
            .build()
    }
    schema = schema.attribute(build_name_attribute());
    #[inline(never)]
    fn build_policies_attribute() -> AttributeSchema {
        AttributeSchema::optional("policies", AttributeType::string())
                .write_only()
                .with_description("Information about the policies assigned to a parameter. [Assigning parameter policies](https://docs.aws.amazon.com/systems-manager/latest/userguide/parameter-store-policies.html) in the *User Guide*.")
                .with_provider_name("Policies")
            .build()
    }
    schema = schema.attribute(build_policies_attribute());
    #[inline(never)]
    fn build_tags_attribute() -> AttributeSchema {
        AttributeSchema::optional("tags", carina_aws_types::tags_type())
                .with_description("Optional metadata that you assign to a resource in the form of an arbitrary set of tags (key-value pairs). Tags enable you to categorize a resource in different ways, such as by purpose, owner, or environment. For example, you might want to tag a SYS parameter to identify the type of resource to which it applies, the environment, or the type of configuration data referenced by the parameter.")
                .with_provider_name("Tags")
            .build()
    }
    schema = schema.attribute(build_tags_attribute());
    #[inline(never)]
    fn build_tier_attribute() -> AttributeSchema {
        AttributeSchema::optional("tier", AttributeType::enum_(carina_core::schema::enum_identity("Tier", Some("aws.ssm.Parameter")), Some(vec!["Standard".to_string(), "Advanced".to_string(), "Intelligent-Tiering".to_string()]), vec![("Standard".to_string(), "standard".to_string()), ("Advanced".to_string(), "advanced".to_string()), ("Intelligent-Tiering".to_string(), "intelligent_tiering".to_string())], None, None))
                .write_only()
                .with_description("The parameter tier.")
                .with_provider_name("Tier")
            .build()
    }
    schema = schema.attribute(build_tier_attribute());
    #[inline(never)]
    fn build_type_attribute() -> AttributeSchema {
        AttributeSchema::required("type", AttributeType::enum_(carina_core::schema::enum_identity("Type", Some("aws.ssm.Parameter")), Some(vec!["String".to_string(), "StringList".to_string()]), vec![("String".to_string(), "string".to_string()), ("StringList".to_string(), "string_list".to_string())], None, None))
                .with_description("The type of parameter. Parameters of type ``SecureString`` are not supported by CFNlong.")
                .with_provider_name("Type")
            .build()
    }
    schema = schema.attribute(build_type_attribute());
    #[inline(never)]
    fn build_value_attribute() -> AttributeSchema {
        AttributeSchema::required("value", AttributeType::string())
                .with_description("The parameter value. If type is ``StringList``, the system returns a comma-separated string with no spaces between commas in the ``Value`` field.")
                .with_provider_name("Value")
            .build()
    }
    schema = schema.attribute(build_value_attribute());
    schema = schema
        .with_unique_name_attribute("name");
    schema = schema.with_declared_validators(vec![carina_provider_protocol::types::ValidatorType::TagsKeyValueCheck]);
    AwsccSchemaConfig {
        aws_type_name: "AWS::SSM::Parameter",
        resource_type_name: "ssm.Parameter",
        primary_identifier: &[crate::schemas::config::PrimaryIdentifierAttribute { provider_name: "Name", dsl_name: "name" }],
        tags: Some(TagsProperty { provider_name: "Tags", wire: TagsWire::Map }),
        schema,
    }
}

/// Returns the resource type name and all enum valid values for this module
pub fn enum_valid_values() -> (
    &'static str,
    &'static [(&'static str, &'static [&'static str])],
) {
    (
        "ssm.Parameter",
        &[
            ("data_type", VALID_DATA_TYPE),
            ("tier", VALID_TIER),
            ("type", VALID_TYPE),
        ],
    )
}

/// Returns the IAM permissions declared by the CloudFormation handler for this operation.
pub fn required_permissions(op: carina_core::effect::PlanOp) -> &'static [&'static str] {
    match op {
        carina_core::effect::PlanOp::Create => &[
            "ssm:PutParameter",
            "ssm:AddTagsToResource",
            "ssm:GetParameters",
        ],
        carina_core::effect::PlanOp::Read => &["ssm:GetParameters", "ssm:ListTagsForResource"],
        carina_core::effect::PlanOp::Update => &[
            "ssm:PutParameter",
            "ssm:AddTagsToResource",
            "ssm:RemoveTagsFromResource",
            "ssm:GetParameters",
        ],
        carina_core::effect::PlanOp::Delete => &["ssm:DeleteParameter"],
    }
}
