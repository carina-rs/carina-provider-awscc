use carina_core::schema::{RawShape, ValidatedResourceSchema};
use serde::Deserialize;

use crate::test_support::{field, int, string, try_validate_fixtures, validate_fixtures};

fn schema(resource: &str) -> ValidatedResourceSchema {
    super::generated::get_config_by_type(resource)
        .unwrap()
        .schema
        .clone()
}

#[derive(Deserialize)]
struct LiveExpression {
    probe: u8,
    line: usize,
    expression: String,
    outcome: String,
}

impl LiveExpression {
    fn accepted(&self) -> bool {
        match self.outcome.as_str() {
            "ACCEPT" | "NEVER" => true,
            "REJECT" | "OTHER" => false,
            outcome => panic!("unknown probe outcome {outcome}"),
        }
    }
}

fn live_expressions() -> Vec<LiveExpression> {
    // CreateSchedule probes, 2026-11-05, ap-northeast-1, carina-test-008.
    // https://github.com/carina-rs/carina-provider-awscc/issues/525#issuecomment-6655771279
    // Every row of sched-probe2/3/4.out, including duplicates. The original
    // scripts disambiguate padding from expression whitespace. Probe2's three
    // truncated OTHER year errors are NEVER, as confirmed by probe3. The one
    // remaining OTHER (L in hour) is rejected: L belongs only in DOM/DOW.
    serde_json::from_str(include_str!(
        "../../tests/fixtures/scheduler_live_expressions.json"
    ))
    .unwrap()
}

#[test]
fn scheduler_expression_matches_every_live_probe_row() {
    let schema = schema("scheduler.Schedule");
    let expression = field(&schema, "schedule_expression");
    let cases = live_expressions();
    assert_eq!(cases.len(), 108);
    assert_eq!(cases.iter().filter(|case| case.accepted()).count(), 63);
    assert_eq!(cases[57].expression, " rate(5 minutes)");
    assert_eq!(cases[58].expression, "rate(5 minutes) ");
    assert_eq!(cases[104].expression, "cron(0 12 * * ? *) ");
    let failures = cases
        .iter()
        .filter_map(|case| {
            let result = expression.validate(&string(&case.expression));
            (result.is_ok() != case.accepted()).then(|| {
                format!(
                    "probe{}:{} {:?} {}: {result:?}",
                    case.probe, case.line, case.expression, case.outcome
                )
            })
        })
        .collect::<Vec<_>>();
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn scheduler_expression_catalog_callback_keeps_identity_bounds_and_all_vectors() {
    let row = super::catalog::AwsccType::ALL
        .iter()
        .find(|row| row.identity().to_string() == "aws.scheduler.Schedule.ScheduleExpression")
        .expect("ScheduleExpression must be an awscc-local catalog row under aws");
    assert_eq!(row.refines(), None);
    let attribute_type = row.attribute_type();
    let RawShape::String {
        identity: Some(identity),
        length,
        validate: Some(callback),
        ..
    } = attribute_type.raw_shape()
    else {
        panic!("ScheduleExpression must retain its string identity and callback");
    };
    assert_eq!(identity.provider.as_deref(), Some("aws"));
    assert_eq!(identity.segments, ["scheduler", "Schedule"]);
    assert_eq!(identity.kind, "ScheduleExpression");
    assert_eq!(length, Some((Some(1), Some(256))));
    for case in live_expressions() {
        assert_eq!(
            callback(&string(&case.expression)).is_ok(),
            case.accepted(),
            "callback probe{}:{} {:?}",
            case.probe,
            case.line,
            case.expression
        );
    }
    let schema = schema("scheduler.Schedule");
    let expression = field(&schema, "schedule_expression");
    // The rate number has no fixed integer bound; only the whole-string bound.
    for (value, valid) in [
        (format!("rate({} minutes)", "9".repeat(242)), true),
        (format!("rate({} minutes)", "9".repeat(243)), false),
        ("".to_string(), false),
        ("rate(000000 minutes)".to_string(), false),
        ("rate(000001minutes)".to_string(), true),
    ] {
        assert_eq!(
            expression.validate(&string(&value)).is_ok(),
            valid,
            "{value:?}"
        );
        assert_eq!(
            callback(&string(&value)).is_ok(),
            valid,
            "callback: {value:?}"
        );
    }
    assert!(callback(&int(1)).is_err());
    assert!(expression.validate(&int(1)).is_err());
}

#[test]
fn scheduler_expression_unprobed_shapes_follow_the_observed_grammar() {
    let schema = schema("scheduler.Schedule");
    let expression = field(&schema, "schedule_expression");
    // These are extrapolations, not additional live AWS findings. Keep them
    // separate from the exact probe vectors so they can be re-measured later.
    for (value, valid) in [
        ("rate(0001MINUTES)", true),
        ("rate(000 minutes)", false),
        ("rate(1\tminutes)", false),
        ("rate(1 minutes )", false),
        ("rate(١ minutes)", false),
        ("weekly(1 days)", false),
        ("rate(1 day))", false),
        ("cron(0\t12\n* * ? *)", true),
        ("cron(0 12 * * ? * )", true),
        ("cron(0\u{a0}12 * * ? *)", true),
        ("cron(0 12 * * ? 0000)", true),
        ("cron(0 12 * * ? 999999999999999999999999999999999)", true),
        ("cron(0 12 * * ? 0-999999999999999999999999)", true),
        ("cron(0 12 * * ? 999999999999999999999999/2)", true),
        ("cron(0 12 * * ? ?)", false),
        ("cron(0 12 * * ? MON)", false),
        ("cron(0 12 * ? ? *)", false),
        ("cron(0 12 ? * L *)", true),
        ("cron(0 12 ? * monl *)", true),
        ("cron(0 12 ? * 2W *)", true),
        ("cron(0 12 l-0 * ? *)", true),
        ("cron(0 12 L-999 * ? *)", true),
        ("cron(0 12 1W,15W,L * ? *)", true),
        ("cron(0 12 ? * MON#1,FRI#5 *)", true),
        ("cron(0 12 ? * MON#0 *)", false),
        ("cron(0 12 ? * 8L *)", false),
        ("cron(0 12 0W * ? *)", false),
        ("cron(0 12 * JANUARY ? *)", false),
        ("cron(0 12 * 0 ? *)", false),
        ("cron(L 12 * * ? *)", false),
        ("cron(0, 12 * * ? *)", false),
        ("cron(0,,30 12 * * ? *)", false),
        ("cron(0/0 12 * * ? *)", false),
        ("cron(*/0001 12 * * ? *)", true),
        ("cron(*/999999999999999999999999 12 * * ? *)", true),
        ("cron(0/2/3 12 * * ? *)", false),
        ("cron(59-0 23-0 ? DEC-JAN SAT-SUN *)", true),
        ("cron(00 012 ? jan mon *)", true),
        ("cron(0 12 L/2 * ? *)", false),
        ("at(2030-01-01T24:00)", true),
        ("at(0000-01-01T00:00)", true),
        ("at(2030-01-01t00:00)", false),
        ("at(2030-01-01T24:00:01)", false),
        ("at(2030-01-01T00:0:00)", false),
        ("at(2030-01-01T00:00:0)", false),
    ] {
        assert_eq!(
            expression.validate(&string(value)).is_ok(),
            valid,
            "{value:?}"
        );
    }
}

#[test]
fn scheduler_dates_follow_cloud_control_input_and_read_back_formats() {
    let schedule = schema("scheduler.Schedule");
    for name in ["start_date", "end_date"] {
        let date = field(&schedule, name);
        assert!(matches!(
            date.as_type().raw_shape(),
            RawShape::String {
                identity: None,
                validate: None,
                pattern: Some(_),
                ..
            }
        ));
        for value in ["2030-01-01T00:00:00.123Z", "2030-01-01T00:00:00.000Z"] {
            assert!(date.validate(&string(value)).is_ok(), "{name}: {value}");
        }
        for value in [
            "2030-01-01T00:00:00Z",
            "2030-01-01T00:00:00+09:00",
            "2030-01-01T00:00:00.000+09:00",
            "2030-01-01T00:00:00.1Z",
            "2030-01-01T00:00:00",
            "2030-01-01T00:00:00.0000Z",
            "2030-01-01T00:00:00.000Z ",
        ] {
            assert!(date.validate(&string(value)).is_err(), "{name}: {value}");
        }
        assert!(date.validate(&int(1)).is_err());
    }

    let group = schema("scheduler.ScheduleGroup");
    for name in ["creation_date", "last_modification_date"] {
        assert!(group.effective_attribute(name).unwrap().is_read_only());
        let date = field(&group, name);
        assert!(matches!(
            date.as_type().raw_shape(),
            RawShape::String {
                identity: None,
                validate: None,
                pattern: Some(_),
                ..
            }
        ));
        for value in [
            "2026-11-05T10:32:55Z",
            "2030-01-01T00:00:00.123Z",
            "2030-01-01T00:00:00+09:00",
            "2030-01-01T00:00:00.000-05:30",
            "2030-01-01T00:00:00-00:00",
            "2030-01-01t00:00:00.1z",
        ] {
            assert!(date.validate(&string(value)).is_ok(), "{name}: {value}");
        }
        for value in [
            "2030-01-01T00:00:00",
            "2030-01-01T00:00Z",
            "2030-01-01T00:00:00.Z",
            "2030-01-01T00:00:00+24:00",
            "2030-01-01T00:00:00+09:60",
            "2030-13-01T00:00:00Z",
            "2030-01-01T25:00:00Z",
            "2030-01-01T00:00:00Z\n",
        ] {
            assert!(date.validate(&string(value)).is_err(), "{name}: {value}");
        }
    }
    let timezone = field(&schedule, "schedule_expression_timezone");
    assert!(matches!(
        timezone.as_type().raw_shape(),
        RawShape::String {
            identity: None,
            pattern: None,
            length: Some((Some(1), Some(50))),
            validate: None,
            ..
        }
    ));
}

fn fixture(expression: &str, attributes: &str) -> String {
    format!(
        r#"awscc.scheduler.Schedule {{
  name = 'scheduler-validation'
  schedule_expression = {expression}
  {attributes}
  flexible_time_window = {{ mode = off }}
  target = {{
    arn = 'arn:aws:lambda:ap-northeast-1:123456789012:function:example:live'
    role_arn = 'arn:aws:iam::123456789012:role/example-scheduler'
  }}
}}"#
    )
}

#[test]
fn scheduler_live_expressions_and_dates_validate_through_test_support() {
    for case in live_expressions() {
        let expression = serde_json::to_string(&case.expression).unwrap();
        let source = fixture(&expression, "");
        let result = try_validate_fixtures(&[("scheduler_probe.crn", &source)]);
        assert_eq!(
            result.is_ok(),
            case.accepted(),
            "fixture probe{}:{} {:?}: {result:?}",
            case.probe,
            case.line,
            case.expression
        );
    }
    let source = fixture(
        "'rate(6 hours)'",
        "start_date = '2030-01-01T00:00:00.123Z'\nend_date = '2031-01-01T00:00:00.000Z'",
    );
    validate_fixtures(&[("scheduler_dates.crn", &source)]);
    for from in ["2030-01-01T00:00:00.123Z", "2031-01-01T00:00:00.000Z"] {
        for to in ["2030-01-01T00:00:00Z", "2030-01-01T00:00:00.000+09:00"] {
            let invalid = source.replace(from, to);
            assert!(try_validate_fixtures(&[("scheduler_bad_date.crn", &invalid)]).is_err());
        }
    }
    let interpolated = format!(
        "let hours = 6\n{}",
        fixture(r#""rate(${hours} hours)""#, "")
    );
    validate_fixtures(&[("scheduler_interpolation.crn", &interpolated)]);
}

#[test]
fn scheduler_examples_and_acceptance_fixtures_keep_cfn_reference_limits() {
    crate::test_support::validate_fixtures_with_cfn_reference_limits(&[
        (
            "examples/scheduler_schedule/main.crn",
            include_str!("../../../examples/scheduler_schedule/main.crn"),
        ),
        (
            "examples/scheduler_schedule_group/main.crn",
            include_str!("../../../examples/scheduler_schedule_group/main.crn"),
        ),
        (
            "acceptance-tests/scheduler/basic.crn",
            include_str!("../../../acceptance-tests/scheduler/basic.crn"),
        ),
        (
            "acceptance-tests/scheduler/group.crn",
            include_str!("../../../acceptance-tests/scheduler/group.crn"),
        ),
    ]);
}

#[test]
fn role_references_retain_the_scheduler_sinks_cfn_length() {
    let role = super::generated::iam::role::iam_role_config().schema;
    let schedule = schema("scheduler.Schedule");
    assert!(matches!(
        field(&role, "arn").as_type().raw_shape(),
        RawShape::String { length: None, .. }
    ));
    assert!(matches!(
        field(&schedule, "target.role_arn").as_type().raw_shape(),
        RawShape::String {
            length: Some((Some(1), Some(1600))),
            ..
        }
    ));
}
