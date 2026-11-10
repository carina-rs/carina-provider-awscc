use carina_core::schema::{RawShape, StringIdentityParts, ValidatedResourceSchema};
use serde_json::json;

use crate::provider::conversion::{aws_value_to_dsl_with_defs, dsl_value_to_aws_with_defs};
use crate::test_support::{
    AssignmentContext, enum_identifier, field, float, int, list, string, validate_fixtures,
};

fn alarm_schema() -> ValidatedResourceSchema {
    super::generated::cloudwatch::alarm::cloudwatch_alarm_config().schema
}

const REFINED_STRING_BOUNDS: &[(&str, u64, Option<u64>)] = &[
    ("alarm_name", 1, Some(255)),
    ("alarm_description", 0, Some(1024)),
    ("metric_name", 1, Some(255)),
    ("namespace", 1, Some(255)),
    ("threshold_metric_id", 1, Some(255)),
    ("metrics.metric_stat.metric.metric_name", 1, Some(255)),
    ("metrics.metric_stat.metric.namespace", 1, Some(255)),
    ("dimensions.name", 1, Some(255)),
    ("dimensions.value", 1, Some(1024)),
    ("metrics.metric_stat.metric.dimensions.name", 1, Some(255)),
    ("metrics.metric_stat.metric.dimensions.value", 1, Some(1024)),
    ("metrics.id", 1, Some(255)),
    ("metrics.expression", 1, Some(2048)),
    ("evaluation_criteria.prom_ql_criteria.query", 1, Some(10000)),
    ("evaluation_window.wall_clock_window.timezone", 1, Some(50)),
    ("extended_statistic", 1, None),
    ("metrics.metric_stat.stat", 1, None),
];

#[test]
fn documented_enums_validate_and_round_trip_exact_aws_wire_strings() {
    let schema = alarm_schema();
    for path in [
        "comparison_operator",
        "statistic",
        "unit",
        "metrics.metric_stat.unit",
        "evaluate_low_sample_count_percentile",
        "treat_missing_data",
    ] {
        let attr_type = field(&schema, path);
        let (_, values, _, _, spellings) = attr_type.as_type().enum_parts().unwrap();
        for wire in values.unwrap() {
            let dsl = spellings.dsl_for(wire);
            attr_type.validate(&enum_identifier(&dsl)).unwrap();
            assert_eq!(spellings.api_for(&dsl), *wire, "{path}");
            if dsl != *wire {
                assert!(
                    attr_type.validate(&enum_identifier(wire)).is_err(),
                    "{path}"
                );
            }
            assert!(
                attr_type.validate(&string(wire)).is_err(),
                "{path}: quoted enum"
            );
            let read = aws_value_to_dsl_with_defs(
                &crate::test_support::diagnostics(),
                path,
                &json!(wire),
                attr_type.as_type(),
                "cloudwatch.Alarm",
                schema.as_schema().defs(),
            )
            .unwrap();
            let canonical = attr_type.canonicalize_state(read).into_present().unwrap();
            assert_eq!(
                dsl_value_to_aws_with_defs(
                    &crate::test_support::diagnostics(),
                    &canonical,
                    attr_type.as_type(),
                    "cloudwatch.Alarm",
                    path,
                    schema.as_schema().defs(),
                ),
                Some(json!(wire)),
                "{path}: AWS read-back must survive canonicalization"
            );
        }
        assert!(
            attr_type
                .validate(&enum_identifier("invalid_cloudwatch_enum"))
                .is_err()
        );
        if path.ends_with("unit") {
            assert_eq!(values.unwrap().len(), 27);
            assert_eq!(spellings.dsl_for("Bytes/Second"), "bytes_second");
            assert_eq!(spellings.dsl_for("Count/Second"), "count_second");
        }
    }
}

#[test]
fn numeric_bounds_accept_endpoints_and_reject_outside_values() {
    let schema = alarm_schema();
    for (path, min, max) in [
        ("period", 10, None),
        ("metrics.period", 1, None),
        ("metrics.metric_stat.period", 1, None),
        ("evaluation_periods", 1, None),
        ("datapoints_to_alarm", 1, None),
        (
            "evaluation_criteria.prom_ql_criteria.pending_period",
            0,
            Some(86400),
        ),
        (
            "evaluation_criteria.prom_ql_criteria.recovery_period",
            0,
            Some(86400),
        ),
        (
            "warm_up_configuration.warm_up_period_duration_in_minutes",
            1,
            Some(2880),
        ),
    ] {
        let attr_type = field(&schema, path);
        for value in [min, max.unwrap_or(i64::MAX)] {
            assert!(attr_type.validate(&int(value)).is_ok(), "{path}: {value}");
        }
        assert!(attr_type.validate(&int(min - 1)).is_err(), "{path}");
        if let Some(max) = max {
            assert!(attr_type.validate(&int(max + 1)).is_err(), "{path}");
        }
        assert!(
            attr_type.validate(&float(min as f64)).is_err(),
            "{path} must remain Int"
        );
    }
    // The pinned core only supports ranges on anonymous Ints, so the three
    // unbounded Period sets cannot enforce the documented modulus rule.
    // Keep this limitation explicit until core gains a portable per-field rule.
    for (path, value) in [
        ("period", 11),
        ("metrics.period", 2),
        ("metrics.metric_stat.period", 2),
    ] {
        assert!(field(&schema, path).validate(&int(value)).is_ok(), "{path}");
    }
}

#[test]
fn evaluation_interval_enforces_its_complete_bounded_period_set() {
    let schema = alarm_schema();
    let interval = field(&schema, "evaluation_interval");
    for value in -1..=3601 {
        let allowed =
            (10..=3600).contains(&value) && (matches!(value, 10 | 20 | 30) || value % 60 == 0);
        assert_eq!(interval.validate(&int(value)).is_ok(), allowed, "{value}");
    }
    assert!(interval.validate(&int(i64::MAX)).is_err());
    assert!(interval.validate(&float(60.0)).is_err());
}

#[test]
fn refined_strings_enforce_each_documented_length_boundary() {
    let schema = alarm_schema();
    for &(path, min, max) in REFINED_STRING_BOUNDS {
        let attr_type = field(&schema, path);
        assert!(
            matches!(
                attr_type.as_type().raw_shape(),
                RawShape::String { identity_layer: StringIdentityParts { identity: None, length: Some(bounds), .. }, .. }
                    if bounds == (Some(min), max)
            ),
            "{path}"
        );
        for length in [min, max.unwrap_or(10001)] {
            attr_type
                .validate(&string(&"a".repeat(length as usize)))
                .unwrap();
        }
        if min > 0 {
            assert!(
                attr_type
                    .validate(&string(&"a".repeat((min - 1) as usize)))
                    .is_err(),
                "{path}"
            );
        }
        if let Some(max) = max {
            assert!(
                attr_type
                    .validate(&string(&"a".repeat((max + 1) as usize)))
                    .is_err(),
                "{path}"
            );
        }
        assert!(
            attr_type.validate(&int(1)).is_err(),
            "{path} must remain String"
        );
    }
}

#[test]
fn alarm_names_reject_ascii_controls_without_restricting_unicode_or_spaces() {
    let schema = alarm_schema();
    let name = field(&schema, "alarm_name");
    for value in [
        " ".to_string(),
        "é".repeat(255),
        "alarm\u{0085}name".to_string(),
    ] {
        name.validate(&string(&value)).unwrap();
    }
    for control in (0u8..=31).chain([127]) {
        assert!(
            name.validate(&string(&format!("alarm{}", char::from(control))))
                .is_err()
        );
    }
}

#[test]
fn namespaces_reject_a_leading_colon_at_top_level_and_in_metrics() {
    let schema = alarm_schema();
    for path in ["namespace", "metrics.metric_stat.metric.namespace"] {
        let namespace = field(&schema, path);
        for value in ["AWS/EC2", "a:b", " :namespace", "名前空間"] {
            namespace.validate(&string(value)).unwrap();
        }
        for value in [":", ":AWS/EC2", "a\nb"] {
            assert!(
                namespace.validate(&string(value)).is_err(),
                "{path}: {value:?}"
            );
        }
    }
}

#[test]
fn dimensions_enforce_printable_ascii_non_whitespace_and_name_prefix() {
    let schema = alarm_schema();
    for prefix in ["dimensions", "metrics.metric_stat.metric.dimensions"] {
        for property in ["name", "value"] {
            let path = format!("{prefix}.{property}");
            let attr_type = field(&schema, &path);
            for value in ["a", " a ", " :a", "a:b"] {
                attr_type.validate(&string(value)).unwrap();
            }
            for value in [" ", "   ", "é", "\u{0085}"] {
                assert!(
                    attr_type.validate(&string(value)).is_err(),
                    "{path}: {value:?}"
                );
            }
            for control in (0u8..=31).chain([127]) {
                let value = format!("a{}b", char::from(control));
                assert!(
                    attr_type.validate(&string(&value)).is_err(),
                    "{path}: {value:?}"
                );
            }
            assert_eq!(
                attr_type.validate(&string(":a")).is_ok(),
                property == "value",
                "{path}"
            );
        }
    }
}

#[test]
fn metric_query_ids_require_a_lowercase_ascii_initial_and_ascii_word_characters() {
    let schema = alarm_schema();
    let id = field(&schema, "metrics.id");
    for value in ["a", "m1", "aA0_Z9"] {
        id.validate(&string(value)).unwrap();
    }
    for value in ["A", "1m", "_m", "a-b", "a b", "a\n", "aé", "a١"] {
        assert!(id.validate(&string(value)).is_err(), "{value:?}");
    }
}

#[test]
fn length_only_text_constraints_count_characters_and_allow_newlines() {
    let schema = alarm_schema();
    for (path, max) in [
        ("alarm_description", 1024),
        ("metric_name", 255),
        ("metrics.metric_stat.metric.metric_name", 255),
        ("threshold_metric_id", 255),
        ("metrics.expression", 2048),
        ("evaluation_criteria.prom_ql_criteria.query", 10000),
    ] {
        let attr_type = field(&schema, path);
        attr_type.validate(&string(&"é".repeat(max))).unwrap();
        attr_type.validate(&string("\n")).unwrap();
        assert!(
            attr_type.validate(&string(&"é".repeat(max + 1))).is_err(),
            "{path}"
        );
    }
}

#[test]
fn statistic_constraints_enforce_nonempty_without_guessing_a_complete_grammar() {
    let schema = alarm_schema();
    for path in ["extended_statistic", "metrics.metric_stat.stat"] {
        let stat = field(&schema, path);
        assert!(matches!(
            stat.as_type().raw_shape(),
            RawShape::String {
                identity_layer: StringIdentityParts {
                    pattern: None,
                    length: Some((Some(1), None)),
                    ..
                },
                ..
            }
        ));
        for value in [
            "p0",
            "p0.5",
            "p99.99",
            "p100",
            "p100.0",
            "tm90",
            "tc90",
            "ts90",
            "wm90",
            "IQM",
            "tm10:90",
            "tc10:90",
            "ts10:90",
            "wm10:90",
            "PR(:300)",
            "PR(100:2000)",
            "PR(10:)",
            "TM(2%:98%)",
            "TC(0.005:0.030)",
            "TS(80%:)",
            "WM(10%:90%)",
        ] {
            stat.validate(&string(value)).unwrap();
        }
        assert!(stat.validate(&string("")).is_err());
    }
    let stat = field(&schema, "metrics.metric_stat.stat");
    for value in ["SampleCount", "Average", "Sum", "Minimum", "Maximum"] {
        stat.validate(&string(value)).unwrap();
    }
}

#[test]
fn timezone_is_length_only_because_iana_names_are_an_open_list() {
    let schema = alarm_schema();
    let timezone = field(&schema, "evaluation_window.wall_clock_window.timezone");
    assert!(matches!(
        timezone.as_type().raw_shape(),
        RawShape::String {
            identity_layer: StringIdentityParts { pattern: None, .. },
            ..
        }
    ));
    for value in [
        "UTC",
        "GMT",
        "Z",
        "Asia/Tokyo",
        "Etc/GMT+5",
        "+05:30",
        "UTC+05:30",
        "Future/Zone",
    ] {
        timezone.validate(&string(value)).unwrap();
    }
}

#[test]
fn refined_strings_accept_identity_typed_references_at_every_tightened_sink() {
    let assignment = AssignmentContext::new();
    let schema = alarm_schema();
    let topic = super::generated::sns::topic::sns_topic_config().schema;
    let vpc = super::generated::ec2::vpc::ec2_vpc_config().schema;
    // carina#4230 defers anonymous pattern/length checks until references resolve.
    for &(path, _, _) in REFINED_STRING_BOUNDS {
        let sink = field(&schema, path);
        assert!(
            matches!(
                sink.as_type().raw_shape(),
                RawShape::String {
                    identity_layer: StringIdentityParts {
                        identity: None,
                        length: Some(_),
                        validate: None,
                        to_dsl: None,
                        ..
                    },
                    ..
                }
            ),
            "{path} must have an anonymous refinement"
        );
        for (source_schema, source_path) in [(&topic, "topic_arn"), (&vpc, "vpc_id")] {
            let source = field(source_schema, source_path);
            assert!(matches!(
                source.as_type().raw_shape(),
                RawShape::String {
                    identity_layer: StringIdentityParts {
                        identity: Some(_),
                        ..
                    },
                    ..
                }
            ));
            assert!(
                source_schema
                    .as_schema()
                    .type_in_schema(source.as_type())
                    .is_assignable_to(assignment.sink(&schema, sink.as_type())),
                "{source_path} must be assignable to {path}"
            );
        }
    }
}

#[test]
fn action_arrays_validate_arns_count_and_allow_duplicates() {
    let assignment = AssignmentContext::new();
    let schema = alarm_schema();
    let topic = super::generated::sns::topic::sns_topic_config().schema;
    let source = field(&topic, "topic_arn");
    let source = topic.as_schema().type_in_schema(source.as_type());
    for path in ["alarm_actions", "ok_actions", "insufficient_data_actions"] {
        let actions = field(&schema, path);
        let item = actions.list_element().unwrap();
        assert!(
            source.is_assignable_to(assignment.sink(&schema, item.as_type())),
            "{path} must accept aws.Arn-typed topic_arn references"
        );
        let arn = string("arn:aws:sns:ap-northeast-1:123456789012:alarm-topic");
        for count in [0, 1, 5] {
            actions.validate(&list(vec![arn.clone(); count])).unwrap();
        }
        assert!(actions.validate(&list(vec![arn; 6])).is_err(), "{path}");
        for value in [
            "",
            "not-an-arn",
            "arn:aws:sns",
            "arn:aws::ap-northeast-1:123456789012:topic",
        ] {
            assert!(
                actions.validate(&list(vec![string(value)])).is_err(),
                "{path}: {value:?}"
            );
        }
    }
}

#[test]
fn existing_tag_key_item_bounds_remain_enforced() {
    let schema = alarm_schema();
    for path in [
        "action_configuration.alarm_tags_propagation.included_tag_keys",
        "action_configuration.alarm_tags_propagation.excluded_tag_keys",
    ] {
        let keys = field(&schema, path);
        for length in [1, 128] {
            keys.validate(&list(vec![string(&"a".repeat(length))]))
                .unwrap();
        }
        for length in [0, 129] {
            assert!(
                keys.validate(&list(vec![string(&"a".repeat(length))]))
                    .is_err(),
                "{path}"
            );
        }
    }
}

#[test]
fn cloudwatch_fixtures_validate_arn_references_in_all_action_lists_and_metric_math_enums() {
    validate_fixtures(&[
        (
            "examples/cloudwatch_alarm/main.crn",
            include_str!("../../../examples/cloudwatch_alarm/main.crn"),
        ),
        (
            "acceptance-tests/cloudwatch_alarm/basic.crn",
            include_str!("../../../acceptance-tests/cloudwatch_alarm/basic.crn"),
        ),
        (
            "metric_math.crn",
            r#"
provider awscc {
  region = awscc.Region.ap_northeast_1
}
awscc.cloudwatch.Alarm {
  alarm_name = 'metric-math'
  comparison_operator = greater_than_threshold
  evaluation_periods = 1
  threshold = 0.5
  metrics = [{
    id = 'm1'
    metric_stat = {
      metric = {
        metric_name = 'NetworkIn'
        namespace = 'AWS/EC2'
        dimensions = [{ name = 'InstanceId', value = 'i-1234567890abcdef0' }]
      }
      period = 60
      stat = 'Average'
      unit = bytes_second
    }
    return_data = true
  }]
}
"#,
        ),
    ]);
}
