//! log_group schema definition for AWS Cloud Control
//!
//! Auto-generated from CloudFormation schema: AWS::Logs::LogGroup
//!
//! DO NOT EDIT MANUALLY - regenerate with carina-codegen

use crate::schemas::config::AwsccSchemaConfig;
use carina_core::resource::{ConcreteValue, Value};
use carina_core::schema::{AttributeSchema, AttributeType, ResourceSchema, legacy_validator};
use regex::Regex;

pub fn arn() -> AttributeType {
    AttributeType::refined_string_with_validator(
        Some(carina_aws_types::provider_type("logs", "LogGroup", "Arn")),
        Some("^arn:(aws|aws-cn|aws-us-gov):logs:[^:]*:[^:]*:log-group:.+$".to_string()),
        None,
        legacy_validator(|value| {
            if let Value::Concrete(ConcreteValue::String(s)) = value {
                carina_aws_types::validate_service_arn(s, "logs", Some("log-group:"))
                    .map_err(|reason| format!("Invalid logs ARN '{}': {}", s, reason))
            } else {
                Err("Expected string".to_string())
            }
        }),
        None,
    )
}

const VALID_LOG_GROUP_CLASS: &[&str] = &["STANDARD", "INFREQUENT_ACCESS", "DELIVERY"];

#[allow(dead_code)]
const VALID_RETENTION_IN_DAYS_VALUES: &[i64] = &[
    1, 3, 5, 7, 14, 30, 60, 90, 120, 150, 180, 365, 400, 545, 731, 1096, 1827, 2192, 2557, 2922,
    3288, 3653,
];

#[allow(dead_code)]
fn validate_retention_in_days_int_enum(value: &Value) -> Result<(), String> {
    if let Value::Concrete(ConcreteValue::Int(n)) = value {
        if VALID_RETENTION_IN_DAYS_VALUES.contains(n) {
            Ok(())
        } else {
            Err(format!("Value {} is not a valid value", n))
        }
    } else {
        Err("Expected integer".to_string())
    }
}

/// Returns the schema config for logs_log_group (AWS::Logs::LogGroup)
pub fn logs_log_group_config() -> AwsccSchemaConfig {
    AwsccSchemaConfig {
        aws_type_name: "AWS::Logs::LogGroup",
        resource_type_name: "logs.LogGroup",
        primary_identifier: &[crate::schemas::config::PrimaryIdentifierAttribute { provider_name: "LogGroupName", dsl_name: "log_group_name" }],
        has_tags: true,
        schema: ResourceSchema::new("logs.LogGroup")
	        .with_description("The ``AWS::Logs::LogGroup`` resource specifies a log group. A log group defines common properties for log streams, such as their retention and access control rules. Each log stream must belong to one log group.  You can create up to 1,000,000 log groups per Region per account. You must use the following guidelines when naming a log group:   +  Log group names must be unique within a Region for an AWS account.   +  Log group names can be between 1 and 512 characters long.   +  Log group names consist of the following characters: a-z, A-Z, 0-9, '_' (underscore), '-' (hyphen), '/' (forward slash), and '.' (period).")
        .attribute(
            AttributeSchema::new("arn", self::arn())
                .read_only()
                .with_description(" (read-only)")
                .with_provider_name("Arn"),
        )
        .attribute(
            AttributeSchema::new("data_protection_policy", AttributeType::map(AttributeType::string()))
                .with_description("Creates a data protection policy and assigns it to the log group. A data protection policy can help safeguard sensitive data that's ingested by the log group by auditing and masking the sensitive log data. When a user who does not have permission to view masked data views a log event that includes masked data, the sensitive data is replaced by asterisks.")
                .with_provider_name("DataProtectionPolicy"),
        )
        .attribute(
            AttributeSchema::new("deletion_protection_enabled", AttributeType::bool())
                .with_description("Indicates whether deletion protection is enabled for this log group. When enabled, deletion protection blocks all deletion operations until it is explicitly disabled.")
                .with_provider_name("DeletionProtectionEnabled")
                .with_default(Value::Concrete(ConcreteValue::Bool(false))),
        )
        .attribute(
            AttributeSchema::new("field_index_policies", AttributeType::unordered_list(AttributeType::map(AttributeType::string())))
                .with_description("Creates or updates a *field index policy* for the specified log group. Only log groups in the Standard log class support field index policies. For more information about log classes, see [Log classes](https://docs.aws.amazon.com/AmazonCloudWatch/latest/logs/CloudWatch_Logs_Log_Classes.html). You can use field index policies to create *field indexes* on fields found in log events in the log group. Creating field indexes lowers the costs for CWL Insights queries that reference those field indexes, because these queries attempt to skip the processing of log events that are known to not match the indexed field. Good fields to index are fields that you often need to query for and fields that have high cardinality of values Common examples of indexes include request ID, session ID, userID, and instance IDs. For more information, see [Create field indexes to improve query performance and reduce costs](https://docs.aws.amazon.com/AmazonCloudWatch/latest/logs/CloudWatchLogs-Field-Indexing.html). Currently, this array supports only one field index policy object.")
                .with_provider_name("FieldIndexPolicies"),
        )
        .attribute(
            AttributeSchema::new("kms_key_id", super::super::kms::key::arn())
                .with_description("The Amazon Resource Name (ARN) of the KMS key to use when encrypting log data. To associate an KMS key with the log group, specify the ARN of that KMS key here. If you do so, ingested data is encrypted using this key. This association is stored as long as the data encrypted with the KMS key is still within CWL. This enables CWL to decrypt this data whenever it is requested. If you attempt to associate a KMS key with the log group but the KMS key doesn't exist or is deactivated, you will receive an ``InvalidParameterException`` error. Log group data is always encrypted in CWL. If you omit this key, the encryption does not use KMS. For more information, see [Encrypt log data in using](https://docs.aws.amazon.com/AmazonCloudWatch/latest/logs/encrypt-log-data-kms.html)")
                .with_provider_name("KmsKeyId"),
        )
        .attribute(
            AttributeSchema::new("log_group_class", AttributeType::enum_(carina_core::schema::enum_identity("LogGroupClass", Some("aws.logs.LogGroup")), Some(vec!["STANDARD".to_string(), "INFREQUENT_ACCESS".to_string(), "DELIVERY".to_string()]), vec![("STANDARD".to_string(), "standard".to_string()), ("INFREQUENT_ACCESS".to_string(), "infrequent_access".to_string()), ("DELIVERY".to_string(), "delivery".to_string())], None, None))
                .with_description("Specifies the log group class for this log group. There are two classes: + The ``Standard`` log class supports all CWL features. + The ``Infrequent Access`` log class supports a subset of CWL features and incurs lower costs. For details about the features supported by each class, see [Log classes](https://docs.aws.amazon.com/AmazonCloudWatch/latest/logs/CloudWatch_Logs_Log_Classes.html)")
                .with_provider_name("LogGroupClass")
                .with_default(Value::Concrete(ConcreteValue::String("STANDARD".to_string()))),
        )
        .attribute(
            AttributeSchema::new("log_group_name", AttributeType::refined_string(None, Some("^[.\\-_/#A-Za-z0-9]{1,512}\\Z".to_string()), Some((Some(1), Some(512))), None))
                .create_only()
                .with_description("The name of the log group. If you don't specify a name, CFNlong generates a unique ID for the log group.")
                .with_provider_name("LogGroupName"),
        )
        .attribute(
            AttributeSchema::new("resource_policy_document", carina_aws_types::iam_policy_document())
                .with_description("Creates or updates a resource policy for the specified log group that allows other services to put log events to this account. A LogGroup can have 1 resource policy.")
                .with_provider_name("ResourcePolicyDocument"),
        )
        .attribute(
            AttributeSchema::new("retention_in_days", AttributeType::refined_int_with_validator(None, None, legacy_validator(validate_retention_in_days_int_enum)))
                .with_description("The number of days to retain the log events in the specified log group. Possible values are: 1, 3, 5, 7, 14, 30, 60, 90, 120, 150, 180, 365, 400, 545, 731, 1096, 1827, 2192, 2557, 2922, 3288, and 3653. To set a log group so that its log events do not expire, do not specify this property.")
                .with_provider_name("RetentionInDays"),
        )
        .attribute(
            AttributeSchema::new("tags", carina_aws_types::tags_type())
                .with_description("An array of key-value pairs to apply to the log group. For more information, see [Tag](https://docs.aws.amazon.com/AWSCloudFormation/latest/UserGuide/aws-properties-resource-tags.html).")
                .with_provider_name("Tags")
                .with_block_name("tag"),
        )
        .with_unique_name_attribute("log_group_name")
        .with_validator(|attrs| {
            let mut errors = Vec::new();
            if let Err(mut e) = carina_aws_types::validate_tags_map(attrs) {
                errors.append(&mut e);
            }
            if errors.is_empty() { Ok(()) } else { Err(errors) }
        })
    }
}

#[allow(dead_code)]
fn validate_string_pattern_b6dfbc56753dfe38_len_1_512(value: &Value) -> Result<(), String> {
    if let Value::Concrete(ConcreteValue::String(s)) = value {
        static RE: std::sync::LazyLock<Regex> = std::sync::LazyLock::new(|| {
            Regex::new("^[.\\-_/#A-Za-z0-9]{1,512}\\z").expect("invalid pattern regex")
        });
        if !RE.is_match(s) {
            return Err(format!(
                "Value '{}' does not match pattern ^[.\\-_/#A-Za-z0-9]{{1,512}}\\z",
                s
            ));
        }
        let len = s.chars().count();
        if !(1..=512).contains(&len) {
            return Err(format!("String length {} is out of range 1..=512", len));
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
        "logs.LogGroup",
        &[("log_group_class", VALID_LOG_GROUP_CLASS)],
    )
}

/// Returns the IAM permissions declared by the CloudFormation handler for this operation.
pub fn required_permissions(op: carina_core::effect::PlanOp) -> &'static [&'static str] {
    match op {
        carina_core::effect::PlanOp::Create => &[
            "logs:DescribeLogGroups",
            "logs:CreateLogGroup",
            "logs:PutRetentionPolicy",
            "logs:TagResource",
            "logs:GetDataProtectionPolicy",
            "logs:PutDataProtectionPolicy",
            "logs:CreateLogDelivery",
            "s3:REST.PUT.OBJECT",
            "firehose:TagDeliveryStream",
            "logs:PutResourcePolicy",
            "logs:DescribeResourcePolicies",
            "logs:PutIndexPolicy",
            "logs:DescribeIndexPolicies",
        ],
        carina_core::effect::PlanOp::Read => &[
            "logs:DescribeLogGroups",
            "logs:ListTagsForResource",
            "logs:GetDataProtectionPolicy",
            "logs:DescribeIndexPolicies",
            "logs:DescribeResourcePolicies",
        ],
        carina_core::effect::PlanOp::Update => &[
            "logs:DescribeLogGroups",
            "logs:AssociateKmsKey",
            "logs:DisassociateKmsKey",
            "logs:PutRetentionPolicy",
            "logs:DeleteRetentionPolicy",
            "logs:TagResource",
            "logs:UntagResource",
            "logs:ListTagsForResource",
            "logs:GetDataProtectionPolicy",
            "logs:PutDataProtectionPolicy",
            "logs:CreateLogDelivery",
            "s3:REST.PUT.OBJECT",
            "firehose:TagDeliveryStream",
            "logs:PutIndexPolicy",
            "logs:DeleteIndexPolicy",
            "logs:PutResourcePolicy",
            "logs:DescribeResourcePolicies",
            "logs:DeleteResourcePolicy",
            "logs:PutLogGroupDeletionProtection",
        ],
        carina_core::effect::PlanOp::Delete => &[
            "logs:DescribeLogGroups",
            "logs:DeleteLogGroup",
            "logs:DeleteDataProtectionPolicy",
        ],
    }
}
