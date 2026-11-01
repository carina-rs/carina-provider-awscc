//! AWS Cloud Control-specific reusable attribute shapes.
//!
//! These types live here, instead of in `carina_aws_types`, until another
//! provider needs the same Cloud Control document shapes.

use carina_core::schema::{AttributeType, StructField, enum_identity};

fn string_enum(name: &str, namespace: &str, values_and_aliases: &[(&str, &str)]) -> AttributeType {
    AttributeType::enum_(
        enum_identity(name, Some(namespace)),
        Some(
            values_and_aliases
                .iter()
                .map(|(value, _)| (*value).to_string())
                .collect(),
        ),
        values_and_aliases
            .iter()
            .map(|(value, alias)| ((*value).to_string(), (*alias).to_string()))
            .collect(),
        None,
        None,
    )
}

/// Amazon SNS subscription dead-letter queue policy.
pub fn sns_subscription_redrive_policy() -> AttributeType {
    AttributeType::struct_(
        "SubscriptionRedrivePolicy".to_string(),
        vec![
            StructField::required("dead_letter_target_arn", AttributeType::string())
                .with_description(
                    "ARN of the Amazon SQS queue that receives undeliverable messages.",
                )
                .with_provider_name("deadLetterTargetArn")
                .build(),
        ],
    )
}

/// Amazon SNS subscription delivery retry and request policy.
pub fn sns_subscription_delivery_policy() -> AttributeType {
    // Cloud Control create/get in both directions with carina-test-008 in
    // ap-northeast-1 on 2026-10-31 accepted healthyRetryPolicy and
    // throttlePolicy for an SQS subscription, rejected requestPolicy because
    // it is HTTP/S-only, and added undocumented `guaranteed: false` on read.
    let backoff_function = string_enum(
        "BackoffFunction",
        "aws.sns.Subscription.DeliveryPolicy.HealthyRetryPolicy",
        &[
            ("arithmetic", "arithmetic"),
            ("exponential", "exponential"),
            ("geometric", "geometric"),
            ("linear", "linear"),
        ],
    );
    let header_content_type = string_enum(
        "HeaderContentType",
        "aws.sns.Subscription.DeliveryPolicy.RequestPolicy",
        &[
            ("text/css", "text_css"),
            ("text/csv", "text_csv"),
            ("text/html", "text_html"),
            ("text/plain", "text_plain"),
            ("text/xml", "text_xml"),
            ("application/atom+xml", "application_atom_xml"),
            ("application/json", "application_json"),
            ("application/octet-stream", "application_octet_stream"),
            ("application/soap+xml", "application_soap_xml"),
            (
                "application/x-www-form-urlencoded",
                "application_x_www_form_urlencoded",
            ),
            ("application/xhtml+xml", "application_xhtml_xml"),
            ("application/xml", "application_xml"),
        ],
    );

    AttributeType::struct_(
        "SubscriptionDeliveryPolicy".to_string(),
        vec![
            StructField::optional(
                "healthy_retry_policy",
                AttributeType::struct_(
                    "HealthyRetryPolicy".to_string(),
                    vec![
                        StructField::optional(
                            "min_delay_target",
                            AttributeType::refined_int(None, Some((Some(1), Some(3600)))),
                        )
                        .with_description("Minimum retry delay, in seconds.")
                        .with_provider_name("minDelayTarget")
                        .build(),
                        StructField::optional(
                            "max_delay_target",
                            AttributeType::refined_int(None, Some((Some(1), Some(3600)))),
                        )
                        .with_description("Maximum retry delay, in seconds.")
                        .with_provider_name("maxDelayTarget")
                        .build(),
                        StructField::optional(
                            "num_retries",
                            AttributeType::refined_int(None, Some((Some(0), Some(100)))),
                        )
                        .with_description("Total number of delivery retries.")
                        .with_provider_name("numRetries")
                        .build(),
                        StructField::optional(
                            "num_no_delay_retries",
                            AttributeType::refined_int(None, Some((Some(0), None))),
                        )
                        .with_description("Number of retries attempted without delay.")
                        .with_provider_name("numNoDelayRetries")
                        .build(),
                        StructField::optional(
                            "num_min_delay_retries",
                            AttributeType::refined_int(None, Some((Some(0), None))),
                        )
                        .with_description("Number of retries attempted at the minimum delay.")
                        .with_provider_name("numMinDelayRetries")
                        .build(),
                        StructField::optional(
                            "num_max_delay_retries",
                            AttributeType::refined_int(None, Some((Some(0), None))),
                        )
                        .with_description("Number of retries attempted at the maximum delay.")
                        .with_provider_name("numMaxDelayRetries")
                        .build(),
                        StructField::optional("backoff_function", backoff_function)
                            .with_description("Function used to calculate retry delays.")
                            .with_provider_name("backoffFunction")
                            .build(),
                    ],
                ),
            )
            .with_description("Retry timing and backoff settings for delivery failures.")
            .with_provider_name("healthyRetryPolicy")
            .build(),
            StructField::optional(
                "throttle_policy",
                AttributeType::struct_(
                    "ThrottlePolicy".to_string(),
                    vec![
                        StructField::optional(
                            "max_receives_per_second",
                            AttributeType::refined_int(None, Some((Some(1), None))),
                        )
                        .with_description("Maximum delivery attempts per second.")
                        .with_provider_name("maxReceivesPerSecond")
                        .build(),
                    ],
                ),
            )
            .with_description("Maximum delivery rate for the subscription.")
            .with_provider_name("throttlePolicy")
            .build(),
            StructField::optional(
                "request_policy",
                AttributeType::struct_(
                    "RequestPolicy".to_string(),
                    vec![
                        StructField::optional("header_content_type", header_content_type)
                            .with_description(
                                "Content-Type header used for HTTP/S notification delivery.",
                            )
                            .with_provider_name("headerContentType")
                            .build(),
                    ],
                ),
            )
            .with_description("HTTP/S request settings for notification delivery.")
            .with_provider_name("requestPolicy")
            .build(),
            StructField::optional("guaranteed", AttributeType::bool())
                .with_description(
                    "Whether delivery uses guaranteed retries; Cloud Control may return false when omitted.",
                )
                .with_provider_name("guaranteed")
                .build(),
        ],
    )
}

/// Amazon SNS FIFO subscription replay policy.
pub fn sns_subscription_replay_policy() -> AttributeType {
    AttributeType::struct_(
        "SubscriptionReplayPolicy".to_string(),
        vec![
            StructField::required(
                "point_type",
                string_enum(
                    "PointType",
                    "aws.sns.Subscription.ReplayPolicy",
                    &[("Timestamp", "timestamp")],
                ),
            )
            .with_description("Kind of replay start point.")
            .with_provider_name("PointType")
            .build(),
            StructField::required("starting_point", AttributeType::string())
                .with_description("ISO-8601 timestamp at which message replay starts.")
                .with_provider_name("StartingPoint")
                .build(),
            StructField::optional("ending_point", AttributeType::string())
                .with_description("ISO-8601 timestamp at which message replay ends.")
                .with_provider_name("EndingPoint")
                .build(),
        ],
    )
}

/// Amazon SNS FIFO topic message archive policy.
pub fn sns_topic_archive_policy() -> AttributeType {
    // Cloud Control create/get in both directions with carina-test-008 in
    // ap-northeast-1 on 2026-10-31 round-tripped MessageRetentionPeriod 1 as
    // the JSON number 1, not a string.
    AttributeType::struct_(
        "TopicArchivePolicy".to_string(),
        vec![
            StructField::required(
                "message_retention_period",
                AttributeType::refined_int(None, Some((Some(1), Some(365)))),
            )
            .with_description("Number of days that the FIFO topic retains messages for replay.")
            .with_provider_name("MessageRetentionPeriod")
            .build(),
        ],
    )
}

/// CloudWatch Logs field index policies.
pub fn logs_field_index_policies() -> AttributeType {
    AttributeType::unordered_list(AttributeType::struct_(
        "FieldIndexPolicy".to_string(),
        vec![
            StructField::required("fields", AttributeType::list(AttributeType::string()))
                .with_description("Log-event fields for which CloudWatch Logs creates indexes.")
                .with_provider_name("Fields")
                .build(),
        ],
    ))
}

/// CloudWatch Logs data protection policy document.
pub fn logs_data_protection_policy() -> AttributeType {
    // Cloud Control create/get in both directions with carina-test-008 in
    // ap-northeast-1 on 2026-10-31 round-tripped this complete grammar,
    // including custom identifiers, Sid, all findings destinations, the empty
    // FindingsDestination marker, and the empty MaskConfig marker.
    let findings_destination = AttributeType::struct_(
        "FindingsDestination".to_string(),
        vec![
            StructField::optional(
                "cloud_watch_logs",
                AttributeType::struct_(
                    "CloudWatchLogsDestination".to_string(),
                    vec![
                        StructField::required("log_group", AttributeType::string())
                            .with_description(
                                "CloudWatch Logs log group that receives audit findings.",
                            )
                            .with_provider_name("LogGroup")
                            .build(),
                    ],
                ),
            )
            .with_description("CloudWatch Logs audit findings destination.")
            .with_provider_name("CloudWatchLogs")
            .build(),
            StructField::optional(
                "firehose",
                AttributeType::struct_(
                    "FirehoseDestination".to_string(),
                    vec![
                        StructField::required("delivery_stream", AttributeType::string())
                            .with_description(
                                "Amazon Data Firehose delivery stream that receives audit findings.",
                            )
                            .with_provider_name("DeliveryStream")
                            .build(),
                    ],
                ),
            )
            .with_description("Amazon Data Firehose audit findings destination.")
            .with_provider_name("Firehose")
            .build(),
            StructField::optional(
                "s3",
                AttributeType::struct_(
                    "S3Destination".to_string(),
                    vec![
                        StructField::required("bucket", AttributeType::string())
                            .with_description("Amazon S3 bucket that receives audit findings.")
                            .with_provider_name("Bucket")
                            .build(),
                    ],
                ),
            )
            .with_description("Amazon S3 audit findings destination.")
            .with_provider_name("S3")
            .build(),
        ],
    );

    let operation = AttributeType::struct_(
        "DataProtectionOperation".to_string(),
        vec![
            StructField::optional(
                "audit",
                AttributeType::struct_(
                    "AuditOperation".to_string(),
                    vec![
                        StructField::required("findings_destination", findings_destination)
                            .with_description("Destinations for sensitive-data audit findings.")
                            .with_provider_name("FindingsDestination")
                            .build(),
                    ],
                ),
            )
            .with_description("Audits sensitive data found in log events.")
            .with_provider_name("Audit")
            .build(),
            StructField::optional(
                "deidentify",
                AttributeType::struct_(
                    "DeidentifyOperation".to_string(),
                    vec![
                        StructField::required(
                            "mask_config",
                            AttributeType::struct_("MaskConfig".to_string(), vec![]),
                        )
                        .with_description("Empty marker selecting the masking operation.")
                        .with_provider_name("MaskConfig")
                        .build(),
                    ],
                ),
            )
            .with_description("Masks sensitive data found in log events.")
            .with_provider_name("Deidentify")
            .build(),
        ],
    );

    AttributeType::struct_(
        "LogGroupDataProtectionPolicy".to_string(),
        vec![
            StructField::required("name", AttributeType::string())
                .with_description("Name of the data protection policy.")
                .with_provider_name("Name")
                .build(),
            StructField::optional("description", AttributeType::string())
                .with_description("Description of the data protection policy.")
                .with_provider_name("Description")
                .build(),
            StructField::required(
                "version",
                string_enum(
                    "Version",
                    "aws.logs.LogGroup.DataProtectionPolicy",
                    &[("2021-06-01", "v2021_06_01")],
                ),
            )
            .with_description("Data protection policy grammar version.")
            .with_provider_name("Version")
            .build(),
            StructField::optional(
                "configuration",
                AttributeType::struct_(
                    "DataProtectionConfiguration".to_string(),
                    vec![
                        StructField::optional(
                            "custom_data_identifier",
                            AttributeType::list(AttributeType::struct_(
                                "CustomDataIdentifier".to_string(),
                                vec![
                                    StructField::required("name", AttributeType::string())
                                        .with_description("Name of the custom data identifier.")
                                        .with_provider_name("Name")
                                        .build(),
                                    StructField::required("regex", AttributeType::string())
                                        .with_description(
                                            "Regular expression matched by the custom data identifier.",
                                        )
                                        .with_provider_name("Regex")
                                        .build(),
                                ],
                            )),
                        )
                        .with_description("Custom patterns recognized as sensitive data.")
                        .with_provider_name("CustomDataIdentifier")
                        .with_block_name("custom_data_identifier")
                        .build(),
                    ],
                ),
            )
            .with_description("Custom data identifiers used by policy statements.")
            .with_provider_name("Configuration")
            .build(),
            StructField::required(
                "statement",
                AttributeType::list(AttributeType::struct_(
                    "DataProtectionStatement".to_string(),
                    vec![
                        StructField::optional("sid", AttributeType::string())
                            .with_description("Optional identifier for the policy statement.")
                            .with_provider_name("Sid")
                            .build(),
                        StructField::required(
                            "data_identifier",
                            AttributeType::list(AttributeType::string()),
                        )
                        .with_description("Sensitive-data identifiers matched by the statement.")
                        .with_provider_name("DataIdentifier")
                        .build(),
                        StructField::required("operation", operation)
                            .with_description(
                                "Audit and deidentification actions performed by the statement.",
                            )
                            .with_provider_name("Operation")
                            .build(),
                    ],
                )),
            )
            .with_description("Data protection policy statements.")
            .with_provider_name("Statement")
            .with_block_name("statement")
            .build(),
        ],
    )
}
