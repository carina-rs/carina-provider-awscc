use carina_core::schema::ValidatedResourceSchema;
use serde_json::json;

use crate::provider::conversion::{aws_value_to_dsl_with_defs, dsl_value_to_aws_with_defs};
use crate::test_support::{enum_identifier, field, float, int, list, string, validate_fixtures};

fn alarm_schema() -> ValidatedResourceSchema {
    super::generated::cloudwatch::alarm::cloudwatch_alarm_config().schema
}

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
fn plain_strings_accept_identity_typed_references_at_every_deferred_constraint() {
    let schema = alarm_schema();
    let topic = super::generated::sns::topic::sns_topic_config().schema;
    let vpc = super::generated::ec2::vpc::ec2_vpc_config().schema;
    // Exercise both an aws.Arn and a resource ID: anonymous string length or
    // pattern refinements reject these references until carina-rs/carina#4230.
    for path in [
        "alarm_name",
        "alarm_description",
        "metric_name",
        "namespace",
        "threshold_metric_id",
        "metrics.metric_stat.metric.metric_name",
        "metrics.metric_stat.metric.namespace",
        "dimensions.name",
        "dimensions.value",
        "metrics.metric_stat.metric.dimensions.name",
        "metrics.metric_stat.metric.dimensions.value",
        "metrics.id",
        "metrics.expression",
        "evaluation_criteria.prom_ql_criteria.query",
        "evaluation_window.wall_clock_window.timezone",
        "extended_statistic",
        "metrics.metric_stat.stat",
    ] {
        let sink = field(&schema, path);
        assert!(
            matches!(
                sink.as_type().raw_shape(),
                carina_core::schema::RawShape::String {
                    identity: None,
                    pattern: None,
                    length: None,
                    validate: None,
                    to_dsl: None,
                }
            ),
            "{path} must remain plain String"
        );
        for (source_schema, source_path) in [(&topic, "topic_arn"), (&vpc, "vpc_id")] {
            let source = field(source_schema, source_path);
            assert!(
                source_schema
                    .as_schema()
                    .type_in_schema(source.as_type())
                    .is_assignable_to(schema.as_schema().type_in_schema(sink.as_type())),
                "{source_path} must be assignable to {path}"
            );
        }
    }
}

#[test]
fn action_arrays_validate_arns_count_and_allow_duplicates() {
    let schema = alarm_schema();
    let topic = super::generated::sns::topic::sns_topic_config().schema;
    let source = field(&topic, "topic_arn");
    let source = topic.as_schema().type_in_schema(source.as_type());
    for path in ["alarm_actions", "ok_actions", "insufficient_data_actions"] {
        let actions = field(&schema, path);
        let item = actions.list_element().unwrap();
        assert!(
            source.is_assignable_to(schema.as_schema().type_in_schema(item.as_type())),
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
