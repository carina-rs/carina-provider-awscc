use std::collections::BTreeSet;
use std::path::Path;

use carina_core::resource::{ConcreteValue, Value};
use carina_core::schema::{
    RawShape, ShapeWalkBudget, StringIdentityParts, ValidatedResourceSchema,
};
use serde_json::json;

use crate::provider::conversion::{aws_value_to_dsl_with_defs, dsl_value_to_aws_with_defs};
use crate::test_support::{
    AssignmentContext, enum_identifier, field, float, int, list, string, try_validate_fixtures,
    validate_fixtures,
};

const NOTIFICATIONS: [&str; 3] = [
    "notification_configuration.lambda_configurations",
    "notification_configuration.queue_configurations",
    "notification_configuration.topic_configurations",
];

const EVENTS: &[(&str, &str)] = &[
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
];

fn schema() -> ValidatedResourceSchema {
    super::generated::get_config_by_type("s3.Bucket")
        .unwrap()
        .schema
        .clone()
}

fn map(entries: impl IntoIterator<Item = (&'static str, Value)>) -> Value {
    Value::Concrete(ConcreteValue::Map(
        entries
            .into_iter()
            .map(|(key, value)| (key.to_string(), value))
            .collect(),
    ))
}

fn destination(format: &str) -> Value {
    map([
        ("bucket_arn", string("arn:aws:s3:::reports")),
        ("format", enum_identifier(format)),
    ])
}

#[test]
fn integer_boundaries_apply_at_every_lifecycle_replication_tiering_and_metadata_sink() {
    let schema = schema();
    for (path, minimum, maximum) in [
        (
            "lifecycle_configuration.rules.expiration_in_days",
            1,
            i64::MAX,
        ),
        (
            "lifecycle_configuration.rules.noncurrent_version_expiration.noncurrent_days",
            1,
            i64::MAX,
        ),
        (
            "lifecycle_configuration.rules.noncurrent_version_expiration.newer_noncurrent_versions",
            i64::MIN,
            100,
        ),
        (
            "lifecycle_configuration.rules.noncurrent_version_transitions.newer_noncurrent_versions",
            i64::MIN,
            100,
        ),
        (
            "lifecycle_configuration.rules.transitions.transition_in_days",
            0,
            i64::MAX,
        ),
        (
            "replication_configuration.rules.destination.metrics.event_threshold.minutes",
            15,
            15,
        ),
        (
            "replication_configuration.rules.destination.replication_time.time.minutes",
            15,
            15,
        ),
        ("intelligent_tiering_configurations.tierings.days", 90, 730),
        (
            "metadata_configuration.journal_table_configuration.record_expiration.days",
            7,
            2147483647,
        ),
    ] {
        let attr = field(&schema, path);
        for value in [minimum, maximum] {
            attr.validate(&int(value))
                .unwrap_or_else(|error| panic!("{path}: {error}"));
        }
        for value in [minimum.checked_sub(1), maximum.checked_add(1)]
            .into_iter()
            .flatten()
        {
            assert!(attr.validate(&int(value)).is_err(), "{path}: {value}");
        }
        assert!(attr.validate(&float(minimum as f64)).is_err(), "{path}");
        assert!(
            attr.validate(&string(&minimum.to_string())).is_err(),
            "{path}"
        );
    }
}

#[test]
fn lifecycle_dates_validate_the_calendar_and_preserve_both_midnight_wire_forms() {
    let schema = schema();
    for path in [
        "lifecycle_configuration.rules.expiration_date",
        "lifecycle_configuration.rules.transitions.transition_date",
    ] {
        let attr = field(&schema, path);
        assert!(matches!(
            attr.as_type().raw_shape(),
            RawShape::String {
                identity_layer: StringIdentityParts {
                    identity: Some(_),
                    validate: Some(_),
                    ..
                },
                ..
            }
        ));
        for value in [
            "2030-01-01T00:00:00Z",     // Cloud Control read, 2026-11-04.
            "2030-01-01T00:00:00.000Z", // Existing CFN-compatible milliseconds.
            "2000-02-29T00:00:00Z",
            "2028-02-29T00:00:00.000Z",
            "1900-03-01T00:00:00Z",
            "9999-12-31T00:00:00Z",
        ] {
            attr.validate(&string(value))
                .unwrap_or_else(|error| panic!("{path}: {error}"));
            let read = aws_value_to_dsl_with_defs(
                &crate::test_support::diagnostics(),
                path,
                &json!(value),
                attr.as_type(),
                "s3.Bucket",
                schema.as_schema().defs(),
            )
            .unwrap();
            let canonical = attr.canonicalize_state(read).into_present().unwrap();
            assert_eq!(
                dsl_value_to_aws_with_defs(
                    &crate::test_support::diagnostics(),
                    &canonical,
                    attr.as_type(),
                    "s3.Bucket",
                    path,
                    schema.as_schema().defs(),
                ),
                Some(json!(value)),
                "{path}"
            );
        }
        for value in [
            "1900-02-29T00:00:00Z",
            "2100-02-29T00:00:00Z",
            "2027-02-29T00:00:00Z",
            "2030-04-31T00:00:00Z",
            "2030-00-01T00:00:00Z",
            "2030-13-01T00:00:00Z",
            "2030-01-00T00:00:00Z",
            "2030-01-32T00:00:00Z",
            "2030-01-01T01:00:00Z",
            "2030-01-01T00:01:00Z",
            "2030-01-01T00:00:01Z",
            "2030-01-01T24:00:00Z",
            "2030-01-01T00:00:60Z",
            "2030-01-01T00:00:00.001Z",
            "2030-01-01T00:00:00.00Z",
            "2030-01-01T00:00:00.0000Z",
            "2030-01-01T00:00:00+00:00",
            "2030-01-01",
            "",
            "2030-01-01T00:00:00Z\n",
            "prefix2030-01-01T00:00:00Z",
            "２０３０-01-01T00:00:00Z",
        ] {
            assert!(attr.validate(&string(value)).is_err(), "{path}: {value:?}");
        }
        assert!(attr.validate(&int(20300101)).is_err());
    }
    let assignment = AssignmentContext::new();
    let expiration = field(&schema, "lifecycle_configuration.rules.expiration_date");
    let transition = field(
        &schema,
        "lifecycle_configuration.rules.transitions.transition_date",
    );
    assert!(
        schema
            .as_schema()
            .type_in_schema(expiration.as_type())
            .is_assignable_to(assignment.sink(&schema, transition.as_type()))
    );
}

#[test]
fn lifecycle_size_thresholds_are_canonical_decimal_strings_with_the_cfn_cap() {
    let schema = schema();
    for name in ["object_size_greater_than", "object_size_less_than"] {
        let path = format!("lifecycle_configuration.rules.{name}");
        let attr = field(&schema, &path);
        for value in ["0", "1", "128", "1048576", "99999999999999999999"] {
            attr.validate(&string(value)).unwrap();
            let read = aws_value_to_dsl_with_defs(
                &crate::test_support::diagnostics(),
                name,
                &json!(value),
                attr.as_type(),
                "s3.Bucket",
                schema.as_schema().defs(),
            )
            .unwrap();
            assert_eq!(read, string(value), "Cloud Control returns strings: {path}");
            let canonical = attr.canonicalize_state(read).into_present().unwrap();
            assert_eq!(
                dsl_value_to_aws_with_defs(
                    &crate::test_support::diagnostics(),
                    &canonical,
                    attr.as_type(),
                    "s3.Bucket",
                    name,
                    schema.as_schema().defs(),
                ),
                Some(json!(value))
            );
        }
        for value in [
            "",
            "00",
            "01",
            "0128",
            "-1",
            "+1",
            "1.0",
            "1e3",
            " 1",
            "1 ",
            "1\n",
            "x1",
            "１",
            "100000000000000000000",
        ] {
            assert!(attr.validate(&string(value)).is_err(), "{path}: {value:?}");
        }
        assert!(attr.validate(&int(128)).is_err());
    }
}

fn counted_item(path: &str, index: usize) -> Value {
    let id = string(&format!("rule-{index}"));
    match path {
        "lifecycle_configuration.rules" => {
            map([("id", id), ("status", enum_identifier("enabled"))])
        }
        "replication_configuration.rules" => map([
            ("id", id),
            ("status", enum_identifier("enabled")),
            (
                "destination",
                map([("bucket", string("arn:aws:s3:::replica"))]),
            ),
        ]),
        "cors_configuration.cors_rules" => map([
            ("id", id),
            ("allowed_methods", list(vec![enum_identifier("get")])),
            ("allowed_origins", list(vec![string("*")])),
        ]),
        "analytics_configurations" => map([
            ("id", id),
            (
                "storage_class_analysis",
                map([(
                    "data_export",
                    map([
                        ("destination", destination("csv")),
                        ("output_schema_version", enum_identifier("v_1")),
                    ]),
                )]),
            ),
        ]),
        "inventory_configurations" => map([
            ("id", id),
            ("destination", destination("parquet")),
            ("enabled", Value::Concrete(ConcreteValue::Bool(true))),
            ("included_object_versions", enum_identifier("current")),
            ("schedule_frequency", enum_identifier("daily")),
        ]),
        "metrics_configurations" => map([("id", id)]),
        "intelligent_tiering_configurations" => map([
            ("id", id),
            ("status", enum_identifier("enabled")),
            (
                "tierings",
                list(vec![map([
                    ("days", int(90)),
                    ("access_tier", enum_identifier("archive_access")),
                ])]),
            ),
        ]),
        "website_configuration.routing_rules" => map([(
            "redirect_rule",
            map([("replace_key_with", string(&format!("page-{index}.html")))]),
        )]),
        _ => string(&index.to_string()),
    }
}

#[test]
fn configuration_counts_preserve_item_shapes_order_and_uniqueness() {
    let schema = schema();
    for (path, minimum, maximum, unique) in [
        ("lifecycle_configuration.rules", 1, Some(1000), true),
        ("replication_configuration.rules", 1, Some(1000), true),
        ("cors_configuration.cors_rules", 0, Some(100), true),
        (
            "cors_configuration.cors_rules.allowed_origins",
            1,
            None,
            true,
        ),
        (
            "cors_configuration.cors_rules.exposed_headers",
            0,
            None,
            true,
        ),
        ("analytics_configurations", 0, Some(1000), true),
        ("inventory_configurations", 0, Some(1000), true),
        ("metrics_configurations", 0, Some(1000), true),
        ("intelligent_tiering_configurations", 0, Some(1000), true),
        // Cached CFN definitions.WebsiteConfiguration.properties.RoutingRules
        // is a direct ordered array with no uniqueItems, $ref or oneOf wrapper.
        ("website_configuration.routing_rules", 0, Some(50), false),
    ] {
        let attr = field(&schema, path);
        for count in [minimum, maximum.unwrap_or(101)] {
            attr.validate(&list(
                (0..count).map(|index| counted_item(path, index)).collect(),
            ))
            .unwrap_or_else(|error| panic!("{path} ({count}): {error}"));
        }
        for count in [minimum.checked_sub(1), maximum.map(|maximum| maximum + 1)]
            .into_iter()
            .flatten()
        {
            assert!(
                attr.validate(&list(
                    (0..count).map(|index| counted_item(path, index)).collect()
                ))
                .is_err(),
                "{path}: {count}"
            );
        }
        assert!(
            matches!(
                attr.as_type().raw_shape(),
                RawShape::List { ordered: true, .. }
            ),
            "{path}"
        );
        assert_eq!(
            attr.validate(&list(vec![counted_item(path, 0), counted_item(path, 0)]))
                .is_err(),
            unique,
            "{path}: preserve CFN duplicate policy"
        );
    }
    for path in [
        "cors_configuration.cors_rules.allowed_origins",
        "cors_configuration.cors_rules.exposed_headers",
    ] {
        // Preserve unconstrained origin/header strings.
        field(&schema, path)
            .validate(&list(vec![string("")]))
            .unwrap();
    }
}

#[test]
fn cors_rules_accept_empty_exposed_headers_but_reject_empty_allowed_origins() {
    let schema = schema();
    let cors_rules = field(&schema, "cors_configuration.cors_rules");
    let rules = |allowed_origins| {
        list(vec![map([
            ("allowed_methods", list(vec![enum_identifier("get")])),
            ("allowed_origins", list(allowed_origins)),
            ("exposed_headers", list(vec![])),
        ])])
    };
    cors_rules
        .validate(&rules(vec![string("*")]))
        .expect("a CORS rule must accept exposed_headers = []");
    assert!(
        cors_rules.validate(&rules(vec![])).is_err(),
        "a CORS rule must reject allowed_origins = []"
    );
}

#[test]
fn metrics_ids_and_website_documents_enforce_only_the_documented_grammar() {
    let schema = schema();
    let id = field(&schema, "metrics_configurations.id");
    for value in ["m-1.x_y".to_string(), "A".repeat(64), ".-_".to_string()] {
        id.validate(&string(&value)).unwrap();
    }
    for value in [
        "".to_string(),
        "a".repeat(65),
        "a/b".to_string(),
        "é".to_string(),
        "m\n".to_string(),
    ] {
        assert!(id.validate(&string(&value)).is_err(), "{value:?}");
    }
    let index = field(&schema, "website_configuration.index_document");
    for value in ["index.html", "é", "a\nb", " "] {
        index.validate(&string(value)).unwrap();
    }
    for value in ["", "/index.html", "dir/index.html", "index.html/"] {
        assert!(index.validate(&string(value)).is_err(), "{value:?}");
    }
    let error = field(&schema, "website_configuration.error_document");
    for value in ["error.html", "errors/404.html", "\n"] {
        error.validate(&string(value)).unwrap();
    }
    assert!(error.validate(&string("")).is_err());
    for attr in [id, index, error] {
        assert!(attr.validate(&int(1)).is_err());
    }
}

#[test]
fn analytics_is_csv_only_while_inventory_retains_all_three_formats() {
    let schema = schema();
    for (path, expected) in [
        (
            "analytics_configurations.storage_class_analysis.data_export.destination.format",
            &["CSV"][..],
        ),
        (
            "inventory_configurations.destination.format",
            &["CSV", "ORC", "Parquet"][..],
        ),
    ] {
        let attr = field(&schema, path);
        let (_, values, _, _, aliases) = attr.as_type().enum_parts().unwrap();
        assert_eq!(
            values
                .unwrap()
                .iter()
                .map(String::as_str)
                .collect::<Vec<_>>(),
            expected,
            "{path}"
        );
        for (wire, dsl) in [("CSV", "csv"), ("ORC", "orc"), ("Parquet", "parquet")] {
            assert_eq!(
                attr.validate(&enum_identifier(dsl)).is_ok(),
                expected.contains(&wire),
                "{path}: {wire}"
            );
            if expected.contains(&wire) {
                assert_eq!(aliases.api_for(dsl), wire);
                let read = aws_value_to_dsl_with_defs(
                    &crate::test_support::diagnostics(),
                    "format",
                    &json!(wire),
                    attr.as_type(),
                    "s3.Bucket",
                    schema.as_schema().defs(),
                )
                .unwrap();
                let canonical = attr.canonicalize_state(read).into_present().unwrap();
                assert_eq!(
                    dsl_value_to_aws_with_defs(
                        &crate::test_support::diagnostics(),
                        &canonical,
                        attr.as_type(),
                        "s3.Bucket",
                        "format",
                        schema.as_schema().defs()
                    ),
                    Some(json!(wire))
                );
            }
            assert!(attr.validate(&string(wire)).is_err());
        }
    }
    // Exercise the context-specific constructor's complete provider-name mapping.
    let path = "analytics_configurations.storage_class_analysis.data_export.destination";
    let attr = field(&schema, path);
    let wire = json!({"BucketArn": "arn:aws:s3:::reports", "BucketAccountId": "123456789012", "Format": "CSV", "Prefix": "reports/"});
    let read = aws_value_to_dsl_with_defs(
        &crate::test_support::diagnostics(),
        "destination",
        &wire,
        attr.as_type(),
        "s3.Bucket",
        schema.as_schema().defs(),
    )
    .unwrap();
    let canonical = attr.canonicalize_state(read).into_present().unwrap();
    assert_eq!(
        dsl_value_to_aws_with_defs(
            &crate::test_support::diagnostics(),
            &canonical,
            attr.as_type(),
            "s3.Bucket",
            "destination",
            schema.as_schema().defs()
        ),
        Some(wire)
    );
}

#[test]
fn inventory_format_preserves_main_identifiers_without_colliding_with_analytics() {
    let schema = schema();
    let inventory = field(&schema, "inventory_configurations.destination.format");
    let analytics = field(
        &schema,
        "analytics_configurations.storage_class_analysis.data_export.destination.format",
    );
    let main_identity =
        "aws.s3.Bucket.AnalyticsConfiguration.StorageClassAnalysis.DataExport.Destination.Format";
    assert_eq!(
        inventory.as_type().enum_parts().unwrap().0.to_string(),
        main_identity
    );
    assert_ne!(
        inventory.as_type().enum_parts().unwrap().0,
        analytics.as_type().enum_parts().unwrap().0
    );
    for alias in ["csv", "orc", "parquet"] {
        let qualified = enum_identifier(&format!("{main_identity}.{alias}"));
        inventory.validate(&qualified).unwrap();
        assert!(analytics.validate(&qualified).is_err());
    }
    analytics
        .validate(&enum_identifier(
            "aws.s3.Bucket.AnalyticsDestination.Format.csv",
        ))
        .unwrap();
}

#[test]
fn notification_events_and_filter_names_have_exact_wire_strings_and_legal_aliases() {
    let schema = schema();
    let event_def = schema
        .as_schema()
        .defs()
        .get("S3NotificationEvent")
        .expect("notification events must have one shared definition");
    assert_eq!(
        event_def.enum_parts().unwrap().0.to_string(),
        "aws.s3.Bucket.NotificationConfiguration.Event"
    );
    assert_eq!(EVENTS.len(), 31);
    assert_eq!(
        EVENTS
            .iter()
            .map(|(_, alias)| alias)
            .collect::<BTreeSet<_>>()
            .len(),
        EVENTS.len()
    );
    for notification in NOTIFICATIONS {
        for (suffix, expected) in [
            ("event", EVENTS),
            (
                "filter.s3_key.rules.name",
                &[("prefix", "prefix"), ("suffix", "suffix")][..],
            ),
        ] {
            let path = format!("{notification}.{suffix}");
            let attr = field(&schema, &path);
            if suffix == "event" {
                assert!(
                    matches!(
                        attr.as_type().raw_shape(),
                        RawShape::Ref("S3NotificationEvent")
                    ),
                    "{path}: reference the shared event declaration"
                );
            }
            let resolved = schema.as_schema().resolve_of(attr.as_type());
            let (_, values, _, _, aliases) = resolved.as_attr().enum_parts().unwrap();
            assert_eq!(
                values
                    .unwrap()
                    .iter()
                    .map(String::as_str)
                    .collect::<Vec<_>>(),
                expected.iter().map(|(wire, _)| *wire).collect::<Vec<_>>(),
                "{path}"
            );
            for (wire, dsl) in expected {
                assert!(
                    dsl.bytes()
                        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == b'_')
                );
                assert_eq!(aliases.dsl_for(wire), *dsl, "{path}");
                assert_eq!(aliases.api_for(dsl), *wire, "{path}");
                attr.validate(&enum_identifier(dsl)).unwrap();
                if wire != dsl {
                    assert!(attr.validate(&enum_identifier(wire)).is_err(), "{path}");
                }
                assert!(attr.validate(&string(wire)).is_err(), "{path}");
                let read = aws_value_to_dsl_with_defs(
                    &crate::test_support::diagnostics(),
                    suffix,
                    &json!(wire),
                    attr.as_type(),
                    "s3.Bucket",
                    schema.as_schema().defs(),
                )
                .unwrap();
                let canonical = attr.canonicalize_state(read).into_present().unwrap();
                assert_eq!(
                    dsl_value_to_aws_with_defs(
                        &crate::test_support::diagnostics(),
                        &canonical,
                        attr.as_type(),
                        "s3.Bucket",
                        suffix,
                        schema.as_schema().defs()
                    ),
                    Some(json!(wire)),
                    "{path}"
                );
            }
            assert!(
                attr.validate(&enum_identifier("unknown")).is_err(),
                "{path}"
            );
        }
    }
}

#[test]
fn v1_metadata_namespace_is_a_read_only_singleton_and_v2_stays_a_string() {
    let schema = schema();
    assert!(
        field(
            &schema,
            "metadata_table_configuration.s3_tables_destination"
        )
        .find_field_with_budget("table_namespace", &mut ShapeWalkBudget::new(64))
        .unwrap()
        .is_read_only()
    );
    let v1 = field(
        &schema,
        "metadata_table_configuration.s3_tables_destination.table_namespace",
    );
    v1.validate(&enum_identifier("aws_s3_metadata")).unwrap();
    assert!(v1.validate(&enum_identifier("custom")).is_err());
    assert!(v1.validate(&string("aws_s3_metadata")).is_err());
    field(
        &schema,
        "metadata_configuration.destination.table_namespace",
    )
    .validate(&string("custom"))
    .unwrap();
}

#[test]
fn deferred_fields_and_legacy_bucket_outputs_keep_their_existing_types() {
    let schema = schema();
    for (path, value) in [
        (
            "domain_name",
            format!("{}.s3.amazonaws.com", "OLD_bucket_".repeat(25)),
        ),
        (
            "regional_domain_name",
            "OLD_bucket.s3.us-east-1.amazonaws.com".to_string(),
        ),
        (
            "dual_stack_domain_name",
            "OLD_bucket.s3.dualstack.us-east-1.amazonaws.com".to_string(),
        ),
        (
            "website_url",
            "http://OLD_bucket.s3-website-us-east-1.amazonaws.com".to_string(),
        ),
    ] {
        assert!(schema.effective_attribute(path).unwrap().is_read_only());
        let output = field(&schema, path);
        assert!(
            matches!(
                output.as_type().raw_shape(),
                RawShape::String {
                    identity_layer: StringIdentityParts {
                        identity: None,
                        pattern: None,
                        length: None,
                        ..
                    },
                    ..
                }
            ),
            "{path} must stay String for legacy bucket reads"
        );
        output.validate(&string(&value)).unwrap();
    }
    for path in [
        "bucket_name",
        "replication_configuration.rules.destination.bucket",
        "lifecycle_configuration.rules.prefix",
        "analytics_configurations.id",
        "inventory_configurations.id",
        "intelligent_tiering_configurations.id",
        "analytics_configurations.prefix",
        "inventory_configurations.prefix",
        "metrics_configurations.prefix",
        "intelligent_tiering_configurations.prefix",
        "analytics_configurations.storage_class_analysis.data_export.destination.prefix",
        "inventory_configurations.destination.prefix",
        "website_configuration.redirect_all_requests_to.host_name",
        "website_configuration.routing_rules.redirect_rule.host_name",
        "website_configuration.routing_rules.redirect_rule.http_redirect_code",
        "website_configuration.routing_rules.routing_rule_condition.http_error_code_returned_equals",
    ] {
        let attr = field(&schema, path);
        for value in ["", "free form / _ : ?", &"x".repeat(2048)] {
            attr.validate(&string(value))
                .unwrap_or_else(|error| panic!("{path}: {error}"));
        }
    }
    for notification in NOTIFICATIONS {
        field(
            &schema,
            &format!("{notification}.filter.s3_key.rules.value"),
        )
        .validate(&string(&"x".repeat(2048)))
        .unwrap();
    }
    for path in [
        "lifecycle_configuration.rules.noncurrent_version_transitions.transition_in_days",
        "replication_configuration.rules.priority",
        "object_lock_configuration.rule.default_retention.days",
        "object_lock_configuration.rule.default_retention.years",
    ] {
        for value in [i64::MIN, -1, 0, i64::MAX] {
            field(&schema, path).validate(&int(value)).unwrap();
        }
    }
    let assignment = AssignmentContext::new();
    let source_schema = super::generated::sns::topic::sns_topic_config().schema;
    let source = field(&source_schema, "topic_arn");
    for path in [
        "bucket_name",
        "metrics_configurations.id",
        "website_configuration.index_document",
    ] {
        let sink = field(&schema, path);
        assert!(
            source_schema
                .as_schema()
                .type_in_schema(source.as_type())
                .is_assignable_to(assignment.sink(&schema, sink.as_type())),
            "{path}"
        );
    }
}

#[test]
fn optional_fields_remain_limited_to_the_cached_cfn_enum() {
    let cached: serde_json::Value = serde_json::from_str(include_str!(
        "../../../cfn-schema-cache/AWS__S3__Bucket.json"
    ))
    .unwrap();
    let expected = cached["definitions"]["InventoryConfiguration"]["properties"]["OptionalFields"]
        ["items"]["enum"]
        .as_array()
        .unwrap();
    let schema = schema();
    let optional = field(&schema, "inventory_configurations.optional_fields");
    let item = optional.list_element().unwrap();
    let (_, values, _, _, aliases) = item.as_type().enum_parts().unwrap();
    assert_eq!(
        values
            .unwrap()
            .iter()
            .map(String::as_str)
            .collect::<Vec<_>>(),
        expected
            .iter()
            .map(|value| value.as_str().unwrap())
            .collect::<Vec<_>>()
    );
    for value in expected {
        let value = value.as_str().unwrap();
        item.validate(&enum_identifier(&aliases.dsl_for(value)))
            .unwrap();
    }
    for alias in [
        "object_lock_event_hold_status",
        "object_lock_event_hold_duration",
    ] {
        assert!(item.validate(&enum_identifier(alias)).is_err());
    }
}

#[test]
fn every_notification_alias_resolves_in_complete_dsl_resources() {
    let cases = EVENTS.iter().map(|(_, alias)| {
        (format!("s3_notification_{alias}.crn"), format!(r#"
provider awscc {{ region = awscc.Region.ap_northeast_1 }}
awscc.s3.Bucket {{
  notification_configuration = {{
    lambda_configurations = [{{
      event = {alias}
      function = 'arn:aws:lambda:ap-northeast-1:123456789012:function:handler'
      filter = {{ s3_key = {{ rules = [{{ name = prefix, value = 'in/' }}, {{ name = suffix, value = '.jpg' }}] }} }}
    }}]
    queue_configurations = [{{ event = {alias}, queue = 'arn:aws:sqs:ap-northeast-1:123456789012:queue' }}]
    topic_configurations = [{{ event = {alias}, topic = 'arn:aws:sns:ap-northeast-1:123456789012:topic' }}]
  }}
}}
"#))
    }).collect::<Vec<_>>();
    let cases = cases
        .iter()
        .map(|(path, source)| (path.as_str(), source.as_str()))
        .collect::<Vec<_>>();
    validate_fixtures(&cases);
}

#[test]
fn live_lifecycle_forms_and_contextual_export_formats_validate_in_complete_dsl() {
    let source = r#"
provider awscc { region = awscc.Region.ap_northeast_1 }
awscc.s3.Bucket {
  lifecycle_configuration = {
    rules = [{
      status = enabled
      expiration_date = '2030-01-01T00:00:00Z'
      object_size_greater_than = '128'
      object_size_less_than = '1048576'
      transitions = [{ storage_class = glacier, transition_date = '2028-02-29T00:00:00.000Z' }]
    }]
  }
  analytics_configurations = [{
    id = 'analytics'
    storage_class_analysis = { data_export = {
      output_schema_version = v_1
      destination = { bucket_arn = 'arn:aws:s3:::reports', format = csv }
    } }
  }]
  inventory_configurations = [{
    id = 'inventory'
    enabled = true
    included_object_versions = current
    schedule_frequency = daily
    destination = { bucket_arn = 'arn:aws:s3:::reports', format = parquet }
  }]
}
"#;
    validate_fixtures(&[("s3_live_forms.crn", source)]);
    let qualified_inventory = source.replace(
        "format = parquet",
        "format = aws.s3.Bucket.AnalyticsConfiguration.StorageClassAnalysis.DataExport.Destination.Format.orc",
    );
    validate_fixtures(&[("s3_qualified_inventory_format.crn", &qualified_inventory)]);
    for (from, to) in [
        ("format = csv", "format = orc"),
        ("2030-01-01T00:00:00Z", "2030-02-30T00:00:00Z"),
        ("2028-02-29T00:00:00.000Z", "2028-02-29T00:00:00.001Z"),
        (
            "object_size_greater_than = '128'",
            "object_size_greater_than = '0128'",
        ),
        (
            "object_size_less_than = '1048576'",
            "object_size_less_than = 1048576",
        ),
    ] {
        let invalid = source.replace(from, to);
        assert!(
            try_validate_fixtures(&[("s3_invalid_forms.crn", &invalid)]).is_err(),
            "{to}"
        );
    }
}

#[test]
fn every_example_and_acceptance_fixture_declaring_s3_resources_validates() {
    fn collect(directory: &Path, root: &Path, fixtures: &mut Vec<(String, String)>) {
        let mut entries = std::fs::read_dir(directory)
            .unwrap()
            .map(|entry| entry.unwrap().path())
            .collect::<Vec<_>>();
        entries.sort();
        for path in entries {
            if path.is_dir() {
                collect(&path, root, fixtures);
            } else if path.extension().is_some_and(|extension| extension == "crn") {
                let source = std::fs::read_to_string(&path).unwrap();
                if source.contains("awscc.s3.") {
                    fixtures.push((
                        path.strip_prefix(root)
                            .unwrap()
                            .to_string_lossy()
                            .into_owned(),
                        source,
                    ));
                }
            }
        }
    }
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
    let mut fixtures = Vec::new();
    for directory in ["examples", "acceptance-tests"] {
        collect(&root.join(directory), root, &mut fixtures);
    }
    assert!(
        !fixtures.is_empty(),
        "the S3 fixture inventory must not be empty"
    );
    let cases = fixtures
        .iter()
        .map(|(path, source)| (path.as_str(), source.as_str()))
        .collect::<Vec<_>>();
    validate_fixtures(&cases);
}
