//! AWS Cloud Control-specific reusable attribute shapes.
//!
//! These types live here, instead of in `carina-aws-types`, until another
//! provider needs the same Cloud Control document shapes.

use carina_core::schema::{
    AttributeSchema, AttributeType, FieldGroup, ListElements, OptionalAttributeSchemaBuilder,
    OptionalStructFieldBuilder, ProviderPopulatedAttributeSchemaBuilder,
    ProviderPopulatedStructFieldBuilder, RawShape, RequiredAttributeSchemaBuilder,
    RequiredStructFieldBuilder, StructField, enum_identity,
};

/// The emitter applies the documented grammar to a freshly constructed builder,
/// before emitting any CFN annotations. The shared seed retains the established
/// constructor spelling and supplies the existing enum identities. There is no
/// dependency alias or hidden shared-type import.
pub(crate) trait DocumentedIamGrammar: Sized {
    fn with_documented_iam_grammar(self) -> Self;
}

macro_rules! documented_iam_builder {
    ($builder:ty, $owner:ident, $constructor:ident, $type_getter:ident) => {
        impl DocumentedIamGrammar for $builder {
            fn with_documented_iam_grammar(self) -> Self {
                let seed = self.build();
                $owner::$constructor(
                    seed.name(),
                    iam_policy_document_from_seed(seed.$type_getter()),
                )
            }
        }
    };
}

documented_iam_builder!(
    OptionalAttributeSchemaBuilder,
    AttributeSchema,
    optional,
    attr_type
);
documented_iam_builder!(
    RequiredAttributeSchemaBuilder,
    AttributeSchema,
    required,
    attr_type
);
documented_iam_builder!(
    ProviderPopulatedAttributeSchemaBuilder,
    AttributeSchema,
    provider_populated,
    attr_type
);
documented_iam_builder!(
    OptionalStructFieldBuilder,
    StructField,
    optional,
    field_type
);
documented_iam_builder!(
    RequiredStructFieldBuilder,
    StructField,
    required,
    field_type
);
documented_iam_builder!(
    ProviderPopulatedStructFieldBuilder,
    StructField,
    provider_populated,
    field_type
);

/// IAM JSON grammar, retaining the established DSL field and enum identities.
/// Checked 2026-11-07:
/// https://docs.aws.amazon.com/IAM/latest/UserGuide/reference_policies_grammar.html
/// https://docs.aws.amazon.com/IAM/latest/UserGuide/reference_policies_elements_statement.html
/// https://docs.aws.amazon.com/IAM/latest/UserGuide/reference_policies_elements_condition.html
pub fn iam_policy_document() -> AttributeType {
    iam_policy_document_from_seed(&carina_aws_types::iam_policy_document())
}

fn iam_policy_document_from_seed(legacy: &AttributeType) -> AttributeType {
    // Reuse the public declaration's nominal enums; changing their namespaces
    // would break existing aws.iam.PolicyDocument.Version/Statement expressions.
    let RawShape::Struct { fields, .. } = legacy.raw_shape() else {
        unreachable!("IAM policy declaration is a struct")
    };
    let version = fields
        .iter()
        .find(|f| f.name() == "version")
        .unwrap()
        .clone();
    let id = fields.iter().find(|f| f.name() == "id").unwrap().clone();
    let statement = fields.iter().find(|f| f.name() == "statement").unwrap();
    let RawShape::List { element_type, .. } = statement.field_type().raw_shape() else {
        unreachable!("legacy Statement is a list")
    };
    let RawShape::Struct { fields, .. } = element_type.raw_shape() else {
        unreachable!("IAM statement is a struct")
    };
    let condition = fields.iter().find(|f| f.name() == "condition").unwrap();
    let RawShape::Map { key, .. } = condition.field_type().raw_shape() else {
        unreachable!("IAM condition is an operator map")
    };
    let scalar = AttributeType::union(vec![
        AttributeType::string(),
        AttributeType::int(),
        AttributeType::float(),
        AttributeType::bool(),
    ]);
    let strings = || {
        AttributeType::union(vec![
            AttributeType::string(),
            AttributeType::refined_list(
                AttributeType::string(),
                false,
                Some((Some(1), None)),
                ListElements::AllowDuplicates,
            ),
        ])
    };
    let condition = AttributeType::map_with_key(
        key.clone(),
        AttributeType::map(AttributeType::union(vec![
            // Keep the established String-or-StringList branch intact so
            // scalar string conditions still canonicalize like Action and
            // Resource. Native numeric/Boolean values are separate branches.
            strings(),
            AttributeType::int(),
            AttributeType::float(),
            AttributeType::bool(),
            AttributeType::refined_list(
                scalar,
                false,
                Some((Some(1), None)),
                ListElements::AllowDuplicates,
            ),
        ])),
    );
    let principal = AttributeType::union(vec![
        AttributeType::struct_with_field_groups(
            "Principal",
            vec![
                StructField::optional("service", strings())
                    .with_provider_name("Service")
                    .build(),
                StructField::optional("aws", strings())
                    .with_provider_name("AWS")
                    .build(),
                StructField::optional("federated", strings())
                    .with_provider_name("Federated")
                    .build(),
                StructField::optional("canonical_user", strings())
                    .with_provider_name("CanonicalUser")
                    .build(),
            ],
            vec![
                FieldGroup::at_least_one(["service", "aws", "federated", "canonical_user"])
                    .unwrap(),
            ],
        ),
        AttributeType::refined_string(None, Some(r"^\*\z".to_string()), None, None),
    ]);
    let effect = fields
        .iter()
        .find(|f| f.name() == "effect")
        .unwrap()
        .field_type()
        .clone();
    let statement = AttributeType::struct_with_field_groups(
        "Statement",
        vec![
            StructField::optional("sid", AttributeType::string())
                .with_provider_name("Sid")
                .build(),
            StructField::required("effect", effect)
                .with_provider_name("Effect")
                .build(),
            StructField::optional("action", strings())
                .with_provider_name("Action")
                .build(),
            StructField::optional("not_action", strings())
                .with_provider_name("NotAction")
                .build(),
            // Trust policies obtain their resource from the attached role.
            StructField::optional("resource", strings())
                .with_provider_name("Resource")
                .build(),
            StructField::optional("not_resource", strings())
                .with_provider_name("NotResource")
                .build(),
            StructField::optional("principal", principal.clone())
                .with_provider_name("Principal")
                .build(),
            StructField::optional("not_principal", principal)
                .with_provider_name("NotPrincipal")
                .build(),
            StructField::optional("condition", condition)
                .with_provider_name("Condition")
                .build(),
        ],
        vec![
            FieldGroup::exactly_one(["action", "not_action"]).unwrap(),
            FieldGroup::at_most_one(["resource", "not_resource"]).unwrap(),
            FieldGroup::at_most_one(["principal", "not_principal"]).unwrap(),
        ],
    );
    AttributeType::struct_(
        "PolicyDocument",
        vec![
            version,
            id,
            StructField::required(
                "statement",
                AttributeType::union(vec![
                    AttributeType::refined_list(
                        statement.clone(),
                        true,
                        Some((Some(1), None)),
                        ListElements::AllowDuplicates,
                    ),
                    statement,
                ]),
            )
            .with_provider_name("Statement")
            .with_block_name("statement")
            .build(),
        ],
    )
}

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
            StructField::required("bucket_arn", AttributeType::string())
                .with_description("ARN of the bucket to which analytics data is exported.")
                .with_provider_name("BucketArn")
                .build(),
            StructField::optional("bucket_account_id", AttributeType::string())
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
            // A measured Cloud Control output also participates in comparison.
            // This field is not an AWS input constraint.
            StructField::optional("guaranteed", AttributeType::bool())
                .with_description("Whether delivery uses guaranteed retries; Cloud Control may return false when omitted.")
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
    // AWS documents an empty JSON object for deactivating archiving:
    // https://docs.aws.amazon.com/sns/latest/dg/message-archiving-and-replay-topic-owner.html
    // Cloud Control create/get in both directions with carina-test-008 in
    // ap-northeast-1 on 2026-10-31 round-tripped MessageRetentionPeriod 1 as
    // the JSON number 1, not a string.
    AttributeType::struct_(
        "TopicArchivePolicy".to_string(),
        vec![
            StructField::optional(
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
    AttributeType::struct_with_field_groups(
        "FieldIndexPolicy".to_string(),
        vec![
            StructField::optional("fields", AttributeType::list(AttributeType::string()))
                .with_description("Log-event fields for which CloudWatch Logs creates indexes.")
                .with_provider_name("Fields")
                .build(),
            StructField::optional(
                "fields_v2",
                AttributeType::map(AttributeType::struct_(
                    "FieldIndexConfiguration",
                    vec![
                        StructField::required(
                            "type",
                            string_enum(
                                "Type",
                                "aws.logs.LogGroup.FieldIndexPolicy.FieldIndexConfiguration",
                                &[("FIELD_INDEX", "field_index"), ("FACET", "facet")],
                            ),
                        )
                        .with_description("Whether this field is an index or a facet.")
                        .with_provider_name("type")
                        .build(),
                    ],
                )),
            )
            .with_description("Log-event fields and their index or facet types.")
            .with_provider_name("FieldsV2")
            .build(),
        ],
        vec![FieldGroup::at_least_one(["fields", "fields_v2"]).unwrap()],
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

    let operation = AttributeType::struct_with_field_groups(
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
        vec![FieldGroup::at_least_one(["audit", "deidentify"]).unwrap()],
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
                            AttributeType::refined_list(AttributeType::struct_(
                                "CustomDataIdentifier".to_string(),
                                vec![
                                    StructField::required("name", AttributeType::string())
                                        .with_description("Name of the custom data identifier.")
                                        .with_provider_name("Name")
                                        .build(),
                                    StructField::required("regex", AttributeType::refined_string(None, None, Some((None, Some(200))), None))
                                        .with_description(
                                            "Regular expression matched by the custom data identifier.",
                                        )
                                        .with_provider_name("Regex")
                                        .build(),
                                ],
                            ), true, Some((None, Some(10))), ListElements::AllowDuplicates),
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
                AttributeType::refined_list(AttributeType::struct_(
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
                ), true, Some((Some(2), Some(2))), ListElements::AllowDuplicates),
            )
            .with_description("Data protection policy statements.")
            .with_provider_name("Statement")
            .with_block_name("statement")
            .build(),
        ],
    )
}
