//! topic schema definition for AWS Cloud Control
//!
//! Auto-generated from CloudFormation schema: AWS::SNS::Topic
//!
//! DO NOT EDIT MANUALLY - regenerate with carina-codegen

use crate::schemas::config::{AwsccSchemaConfig, TagsProperty, TagsWire};
use carina_core::schema::{AttributeSchema, AttributeType, ResourceSchema, StructField};

const VALID_LOGGING_CONFIG_PROTOCOL: &[&str] =
    &["http/s", "sqs", "lambda", "firehose", "application"];

/// Returns the schema config for sns_topic (AWS::SNS::Topic)
#[rustfmt::skip]
pub fn sns_topic_config() -> AwsccSchemaConfig {
    let mut schema = ResourceSchema::new("sns.Topic")
        .with_description("The ``AWS::SNS::Topic`` resource creates a topic to which notifications can be published.   One account can create a maximum of 100,000 standard topics and 1,000 FIFO topics. For more information, see [endpoints and quotas](https://docs.aws.amazon.com/general/latest/gr/sns.html) in the *General Reference*.    The structure of ``AUTHPARAMS`` depends on the .signature of the API request. For more information, see [Examples of the complete Signature Version 4 signing process](https://docs.aws.amazon.com/general/latest/gr/sigv4-signed-request-examples.html) in the *General Reference*.");
    #[inline(never)]
    fn build_archive_policy_attribute() -> AttributeSchema {
        AttributeSchema::optional("archive_policy", crate::schemas::awscc_types::sns_topic_archive_policy())
                .with_description("The ``ArchivePolicy`` determines the number of days SNS retains messages in FIFO topics. You can set a retention period ranging from 1 to 365 days. This property is only applicable to FIFO topics; attempting to use it with standard topics will result in a creation failure.")
                .with_provider_name("ArchivePolicy")
            .build()
    }
    schema = schema.attribute(build_archive_policy_attribute());
    #[inline(never)]
    fn build_content_based_deduplication_attribute() -> AttributeSchema {
        AttributeSchema::optional("content_based_deduplication", AttributeType::bool())
                .with_description("``ContentBasedDeduplication`` enables deduplication of messages based on their content for FIFO topics. By default, this property is set to false. If you create a FIFO topic with ``ContentBasedDeduplication`` set to false, you must provide a ``MessageDeduplicationId`` for each ``Publish`` action. When set to true, SNS automatically generates a ``MessageDeduplicationId`` using a SHA-256 hash of the message body (excluding message attributes). You can optionally override this generated value by specifying a ``MessageDeduplicationId`` in the ``Publish`` action. Note that this property only applies to FIFO topics; using it with standard topics will cause the creation to fail.")
                .with_provider_name("ContentBasedDeduplication")
            .build()
    }
    schema = schema.attribute(build_content_based_deduplication_attribute());
    #[inline(never)]
    fn build_delivery_status_logging_attribute() -> AttributeSchema {
        AttributeSchema::optional("delivery_status_logging", AttributeType::unordered_list(AttributeType::struct_("LoggingConfig".to_string(), vec![StructField::optional("failure_feedback_role_arn", carina_aws_types::arn()).with_description("The IAM role ARN to be used when logging failed message deliveries in Amazon CloudWatch.").with_provider_name("FailureFeedbackRoleArn").build(),
                    StructField::required("protocol", AttributeType::enum_(carina_core::schema::enum_identity("Protocol", Some("aws.sns.Topic.LoggingConfig")), Some(vec!["http/s".to_string(), "sqs".to_string(), "lambda".to_string(), "firehose".to_string(), "application".to_string()]), vec![("http/s".to_string(), "http_s".to_string()), ("sqs".to_string(), "sqs".to_string()), ("lambda".to_string(), "lambda".to_string()), ("firehose".to_string(), "firehose".to_string()), ("application".to_string(), "application".to_string())], None, None)).with_description("Indicates one of the supported protocols for the Amazon SNS topic. At least one of the other three ``LoggingConfig`` properties is recommend along with ``Protocol``.").with_provider_name("Protocol").build(),
                    StructField::optional("success_feedback_role_arn", carina_aws_types::arn()).with_description("The IAM role ARN to be used when logging successful message deliveries in Amazon CloudWatch.").with_provider_name("SuccessFeedbackRoleArn").build(),
                    StructField::optional("success_feedback_sample_rate", AttributeType::string()).with_description("The percentage of successful message deliveries to be logged in Amazon CloudWatch. Valid percentage values range from 0 to 100.").with_provider_name("SuccessFeedbackSampleRate").build()])))
                .with_description("The ``DeliveryStatusLogging`` configuration enables you to log the delivery status of messages sent from your Amazon SNS topic to subscribed endpoints with the following supported delivery protocols: + HTTP + Amazon Kinesis Data Firehose + AWS Lambda + Platform application endpoint + Amazon Simple Queue Service Once configured, log entries are sent to Amazon CloudWatch Logs.")
                .with_provider_name("DeliveryStatusLogging")
                .with_block_name("delivery_status_logging")
            .build()
    }
    schema = schema.attribute(build_delivery_status_logging_attribute());
    #[inline(never)]
    fn build_display_name_attribute() -> AttributeSchema {
        AttributeSchema::optional("display_name", AttributeType::string())
                .with_description("The display name to use for an SNS topic with SMS subscriptions. The display name must be maximum 100 characters long, including hyphens (-), underscores (_), spaces, and tabs.")
                .with_provider_name("DisplayName")
            .build()
    }
    schema = schema.attribute(build_display_name_attribute());
    #[inline(never)]
    fn build_fifo_throughput_scope_attribute() -> AttributeSchema {
        AttributeSchema::optional("fifo_throughput_scope", AttributeType::string())
                .with_description("Specifies the throughput quota and deduplication behavior to apply for the FIFO topic. Valid values are ``Topic`` or ``MessageGroup``.")
                .with_provider_name("FifoThroughputScope")
            .build()
    }
    schema = schema.attribute(build_fifo_throughput_scope_attribute());
    #[inline(never)]
    fn build_fifo_topic_attribute() -> AttributeSchema {
        AttributeSchema::optional("fifo_topic", AttributeType::bool())
                .create_only()
                .with_description("Set to true to create a FIFO topic.")
                .with_provider_name("FifoTopic")
            .build()
    }
    schema = schema.attribute(build_fifo_topic_attribute());
    #[inline(never)]
    fn build_kms_master_key_id_attribute() -> AttributeSchema {
        AttributeSchema::optional("kms_master_key_id", AttributeType::string())
                .with_description("The ID of an AWS managed customer master key (CMK) for SNS or a custom CMK. For more information, see [Key terms](https://docs.aws.amazon.com/sns/latest/dg/sns-server-side-encryption.html#sse-key-terms). For more examples, see ``KeyId`` in the *API Reference*. This property applies only to [server-side-encryption](https://docs.aws.amazon.com/sns/latest/dg/sns-server-side-encryption.html).")
                .with_provider_name("KmsMasterKeyId")
            .build()
    }
    schema = schema.attribute(build_kms_master_key_id_attribute());
    #[inline(never)]
    fn build_maximum_message_size_attribute() -> AttributeSchema {
        AttributeSchema::optional("maximum_message_size", AttributeType::refined_int(None, Some((Some(1024), Some(1048576)))))
                .with_description("The maximum payload size, in bytes, for messages published to the topic. Valid values are ``1024`` to ``1048576`` (1 MiB). When this property is not specified, the topic uses the default SNS message size of ``262144`` bytes (256 KiB). A topic with ``MaximumMessageSize`` above ``262144`` supports a maximum of 100 subscriptions, all of which must use the SQS, AWS Lambda, or Amazon Data Firehose protocols. For more information, see [Publishing large messages with](https://docs.aws.amazon.com/sns/latest/dg/large-message-payloads.html) in the *Developer Guide*.")
                .with_provider_name("MaximumMessageSize")
            .build()
    }
    schema = schema.attribute(build_maximum_message_size_attribute());
    #[inline(never)]
    fn build_signature_version_attribute() -> AttributeSchema {
        AttributeSchema::optional("signature_version", AttributeType::string())
                .with_description("The signature version corresponds to the hashing algorithm used while creating the signature of the notifications, subscription confirmations, or unsubscribe confirmation messages sent by Amazon SNS. By default, ``SignatureVersion`` is set to ``1``.")
                .with_provider_name("SignatureVersion")
            .build()
    }
    schema = schema.attribute(build_signature_version_attribute());
    #[inline(never)]
    fn build_tags_attribute() -> AttributeSchema {
        AttributeSchema::optional("tags", carina_aws_types::tags_type())
                .with_description("The list of tags to add to a new topic. To be able to tag a topic on creation, you must have the ``sns:CreateTopic`` and ``sns:TagResource`` permissions.")
                .with_provider_name("Tags")
                .with_block_name("tag")
            .build()
    }
    schema = schema.attribute(build_tags_attribute());
    #[inline(never)]
    fn build_topic_arn_attribute() -> AttributeSchema {
        AttributeSchema::provider_populated("topic_arn", carina_aws_types::arn())
                .with_description(" (read-only)")
                .with_provider_name("TopicArn")
            .build()
    }
    schema = schema.attribute(build_topic_arn_attribute());
    #[inline(never)]
    fn build_topic_name_attribute() -> AttributeSchema {
        AttributeSchema::optional("topic_name", AttributeType::string())
                .create_only()
                .with_description("The name of the topic you want to create. Topic names must include only uppercase and lowercase ASCII letters, numbers, underscores, and hyphens, and must be between 1 and 256 characters long. FIFO topic names must end with ``.fifo``. If you don't specify a name, CFN generates a unique physical ID and uses that ID for the topic name. For more information, see [Name type](https://docs.aws.amazon.com/AWSCloudFormation/latest/UserGuide/aws-properties-name.html). If you specify a name, you can't perform updates that require replacement of this resource. You can perform updates that require no or some interruption. If you must replace the resource, specify a new name.")
                .with_provider_name("TopicName")
            .build()
    }
    schema = schema.attribute(build_topic_name_attribute());
    #[inline(never)]
    fn build_tracing_config_attribute() -> AttributeSchema {
        AttributeSchema::optional("tracing_config", AttributeType::string())
                .with_description("Tracing mode of an SNS topic. By default ``TracingConfig`` is set to ``PassThrough``, and the topic passes through the tracing header it receives from an SNS publisher to its subscriptions. If set to ``Active``, SNS will vend X-Ray segment data to topic owner account if the sampled flag in the tracing header is true.")
                .with_provider_name("TracingConfig")
            .build()
    }
    schema = schema.attribute(build_tracing_config_attribute());
    schema = schema
        .with_conflicting_replacement();
    schema = schema.with_declared_validators(vec![carina_provider_protocol::types::ValidatorType::TagsKeyValueCheck]);
    AwsccSchemaConfig {
        aws_type_name: "AWS::SNS::Topic",
        resource_type_name: "sns.Topic",
        primary_identifier: &[crate::schemas::config::PrimaryIdentifierAttribute { provider_name: "TopicArn", dsl_name: "topic_arn" }],
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
    ("sns.Topic", &[("protocol", VALID_LOGGING_CONFIG_PROTOCOL)])
}

/// Returns the IAM permissions declared by the CloudFormation handler for this operation.
pub fn required_permissions(op: carina_core::effect::PlanOp) -> &'static [&'static str] {
    match op {
        carina_core::effect::PlanOp::Create => &[
            "sns:CreateTopic",
            "sns:TagResource",
            "sns:Subscribe",
            "sns:GetTopicAttributes",
            "sns:PutDataProtectionPolicy",
            "iam:GetRole",
            "iam:PassRole",
        ],
        carina_core::effect::PlanOp::Read => &[
            "sns:GetTopicAttributes",
            "sns:ListTagsForResource",
            "sns:ListSubscriptionsByTopic",
            "sns:GetDataProtectionPolicy",
        ],
        carina_core::effect::PlanOp::Update => &[
            "sns:SetTopicAttributes",
            "sns:TagResource",
            "sns:UntagResource",
            "sns:Subscribe",
            "sns:Unsubscribe",
            "sns:GetTopicAttributes",
            "sns:ListTagsForResource",
            "sns:ListSubscriptionsByTopic",
            "sns:GetDataProtectionPolicy",
            "sns:PutDataProtectionPolicy",
            "iam:GetRole",
            "iam:PassRole",
        ],
        carina_core::effect::PlanOp::Delete => &["sns:GetTopicAttributes", "sns:DeleteTopic"],
    }
}
