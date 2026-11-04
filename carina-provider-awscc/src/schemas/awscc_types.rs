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

/// Organizations parent IDs remain anonymous until identity work in #422.
pub fn organizations_account_parent_id() -> AttributeType {
    // Group the alternatives so both anchors apply to root and OU IDs alike.
    // The 100-character cap is an item limit, not a list count.
    // https://docs.aws.amazon.com/organizations/latest/APIReference/API_Parent.html#organizations-Type-Parent-Id
    AttributeType::refined_string(
        None,
        Some(r"^(?:r-[0-9a-z]{4,32}|ou-[0-9a-z]{4,32}-[a-z0-9]{8,32})\z".to_string()),
        Some((None, Some(100))),
        None,
    )
}

/// S3 lifecycle size thresholds retain CloudFormation's decimal-string wire type.
pub fn s3_object_size() -> AttributeType {
    // The API Long is represented as a string by CFN, with a 20-character cap.
    // Cloud Control GetResource, carina-test-001, ap-northeast-1, 2026-11-04,
    // returned ObjectSizeGreaterThan/LessThan as "128" / "1048576".
    // https://docs.aws.amazon.com/AmazonS3/latest/API/API_LifecycleRuleFilter.html
    AttributeType::refined_string(
        None,
        Some(r"^(?:0|[1-9][0-9]*)\z".to_string()),
        Some((None, Some(20))),
        None,
    )
}

/// S3 notification events, shared locally by Lambda, SQS and SNS destinations.
/// The generator declares this body once as `S3NotificationEvent`; destination
/// fields reference that definition instead of redeclaring the enum.
pub fn s3_notification_event() -> AttributeType {
    // Explicit aliases preserve the s3: prefix, case and wildcards on the wire.
    // https://docs.aws.amazon.com/AmazonS3/latest/API/API_LambdaFunctionConfiguration.html
    // https://docs.aws.amazon.com/AmazonS3/latest/API/API_QueueConfiguration.html
    // https://docs.aws.amazon.com/AmazonS3/latest/API/API_TopicConfiguration.html
    string_enum(
        "Event",
        "aws.s3.Bucket.NotificationConfiguration",
        &[
            ("s3:ObjectCreated:*", "object_created_all"),
            ("s3:ObjectCreated:Put", "object_created_put"),
            ("s3:ObjectCreated:Post", "object_created_post"),
            ("s3:ObjectCreated:Copy", "object_created_copy"),
            (
                "s3:ObjectCreated:CompleteMultipartUpload",
                "object_created_complete_multipart_upload",
            ),
            ("s3:ObjectRemoved:*", "object_removed_all"),
            ("s3:ObjectRemoved:Delete", "object_removed_delete"),
            (
                "s3:ObjectRemoved:DeleteMarkerCreated",
                "object_removed_delete_marker_created",
            ),
            ("s3:ObjectRestore:*", "object_restore_all"),
            ("s3:ObjectRestore:Post", "object_restore_post"),
            ("s3:ObjectRestore:Completed", "object_restore_completed"),
            ("s3:ObjectRestore:Delete", "object_restore_delete"),
            ("s3:Replication:*", "replication_all"),
            (
                "s3:Replication:OperationFailedReplication",
                "replication_operation_failed_replication",
            ),
            (
                "s3:Replication:OperationNotTracked",
                "replication_operation_not_tracked",
            ),
            (
                "s3:Replication:OperationMissedThreshold",
                "replication_operation_missed_threshold",
            ),
            (
                "s3:Replication:OperationReplicatedAfterThreshold",
                "replication_operation_replicated_after_threshold",
            ),
            ("s3:LifecycleTransition", "lifecycle_transition"),
            ("s3:IntelligentTiering", "intelligent_tiering"),
            ("s3:ObjectAcl:Put", "object_acl_put"),
            ("s3:LifecycleExpiration:*", "lifecycle_expiration_all"),
            (
                "s3:LifecycleExpiration:Delete",
                "lifecycle_expiration_delete",
            ),
            (
                "s3:LifecycleExpiration:DeleteMarkerCreated",
                "lifecycle_expiration_delete_marker_created",
            ),
            ("s3:ObjectTagging:*", "object_tagging_all"),
            ("s3:ObjectTagging:Put", "object_tagging_put"),
            ("s3:ObjectTagging:Delete", "object_tagging_delete"),
            ("s3:ObjectAnnotation:*", "object_annotation_all"),
            ("s3:ObjectAnnotation:Put", "object_annotation_put"),
            ("s3:ObjectAnnotation:Delete", "object_annotation_delete"),
            ("s3:ObjectRetention:Put", "object_retention_put"),
            (
                "s3:ReducedRedundancyLostObject",
                "reduced_redundancy_lost_object",
            ),
        ],
    )
}

/// Analytics exports only CSV; inventory retains the cached Destination shape.
pub fn s3_analytics_destination() -> AttributeType {
    // Give the contextual shape its own name so it cannot redefine inventory's
    // Destination.Format enum under the same identity in the provider witness.
    // https://docs.aws.amazon.com/AmazonS3/latest/API/API_AnalyticsS3BucketDestination.html
    // https://docs.aws.amazon.com/AmazonS3/latest/API/API_InventoryS3BucketDestination.html
    AttributeType::struct_(
        "AnalyticsDestination".to_string(),
        vec![
            StructField::required("bucket_arn", carina_aws_types::arn())
                .with_description("ARN of the bucket to which analytics data is exported.")
                .with_provider_name("BucketArn")
                .build(),
            StructField::optional("bucket_account_id", carina_aws_types::aws_account_id())
                .with_description("Account ID that owns the destination bucket.")
                .with_provider_name("BucketAccountId")
                .build(),
            StructField::required(
                "format",
                string_enum(
                    "Format",
                    "aws.s3.Bucket.AnalyticsDestination",
                    &[("CSV", "csv")],
                ),
            )
            .with_description("Analytics export format (CSV).")
            .with_provider_name("Format")
            .build(),
            StructField::optional("prefix", AttributeType::string())
                .with_description("Prefix prepended to all analytics results.")
                .with_provider_name("Prefix")
                .build(),
        ],
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

/// One CloudWatch Logs field index policy; codegen supplies the CFN list constraints.
pub fn logs_field_index_policy() -> AttributeType {
    AttributeType::struct_(
        "FieldIndexPolicy".to_string(),
        vec![
            StructField::required("fields", AttributeType::list(AttributeType::string()))
                .with_description("Log-event fields for which CloudWatch Logs creates indexes.")
                .with_provider_name("Fields")
                .build(),
        ],
    )
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
