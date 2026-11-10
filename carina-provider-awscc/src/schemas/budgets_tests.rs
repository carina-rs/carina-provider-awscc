use carina_core::resource::{ConcreteValue, Value};
use carina_core::schema::{RawShape, StringIdentityParts, ValidatedResourceSchema};
use serde_json::json;

use crate::provider::conversion::{aws_value_to_dsl_with_defs, dsl_value_to_aws_with_defs};
use crate::test_support::{
    AssignmentContext, enum_identifier, field, float, int, list, string, validate_fixtures,
};

fn budget_schema() -> ValidatedResourceSchema {
    super::generated::budgets::budget::budgets_budget_config().schema
}

#[test]
fn documented_enums_validate_and_round_trip_aws_wire_strings() {
    let schema = budget_schema();
    for path in [
        "budget.budget_type",
        "budget.time_unit",
        "budget.metrics",
        "budget.auto_adjust_data.auto_adjust_type",
        "notifications_with_subscribers.notification.comparison_operator",
        "notifications_with_subscribers.notification.notification_type",
        "notifications_with_subscribers.notification.threshold_type",
        "notifications_with_subscribers.subscribers.subscription_type",
        "budget.filter_expression.dimensions.key",
        "budget.filter_expression.dimensions.match_options",
        "budget.filter_expression.tags.match_options",
        "budget.filter_expression.cost_categories.match_options",
        "budget.filter_expression.not.and.dimensions.key",
        "budget.filter_expression.or.tags.match_options",
    ] {
        let attr_type = field(&schema, path);
        let attr_type = attr_type.list_element().unwrap_or(attr_type);
        let (_, values, _, _, spellings) = attr_type
            .as_type()
            .enum_parts()
            .unwrap_or_else(|| panic!("{path} must be an enum"));
        for wire in values.unwrap() {
            let dsl = spellings.dsl_for(wire);
            attr_type
                .validate(&enum_identifier(&dsl))
                .unwrap_or_else(|error| panic!("{path}: {dsl}: {error}"));
            assert_eq!(spellings.api_for(&dsl), *wire, "{path}");
            assert!(
                attr_type.validate(&enum_identifier(wire)).is_err(),
                "{path}: wire spelling is not a DSL identifier"
            );
            assert!(
                attr_type.validate(&string(wire)).is_err(),
                "{path}: quoted enum strings must fail"
            );

            let read = aws_value_to_dsl_with_defs(
                &crate::test_support::diagnostics(),
                path,
                &json!(wire),
                attr_type.as_type(),
                "budgets.Budget",
                schema.as_schema().defs(),
            )
            .expect("AWS enum state must decode");
            let canonical = attr_type
                .canonicalize_state(read)
                .into_present()
                .expect("AWS enum state must canonicalize");
            assert_eq!(
                dsl_value_to_aws_with_defs(
                    &crate::test_support::diagnostics(),
                    &canonical,
                    attr_type.as_type(),
                    "budgets.Budget",
                    path,
                    schema.as_schema().defs(),
                ),
                Some(json!(wire)),
                "{path}: AWS wire spelling must survive the round trip"
            );
        }
        assert!(
            attr_type
                .validate(&enum_identifier("invalid_budget_enum"))
                .is_err(),
            "{path}"
        );
    }
}

#[test]
fn numeric_bounds_accept_endpoints_and_reject_outside_values() {
    let schema = budget_schema();
    for (path, accepted, rejected) in [
        (
            "notifications_with_subscribers.notification.threshold",
            vec![0.0, 0.5, 15000000000000.0],
            vec![-0.5, 15000000000000.5],
        ),
        (
            "budget.budget_limit.amount",
            vec![0.0, 0.5, 15000000000001.0],
            vec![-0.5],
        ),
        (
            "budget.planned_budget_limits.amount",
            vec![0.0, 0.5, 15000000000001.0],
            vec![-0.5],
        ),
    ] {
        let attr_type = field(&schema, path);
        for value in accepted {
            assert!(attr_type.validate(&float(value)).is_ok(), "{path}: {value}");
        }
        for value in rejected {
            assert!(
                attr_type.validate(&float(value)).is_err(),
                "{path}: {value}"
            );
        }
    }
    let period = field(
        &schema,
        "budget.auto_adjust_data.historical_options.budget_adjustment_period",
    );
    for value in [1, 60] {
        period.validate(&int(value)).unwrap();
    }
    for value in [0, 61] {
        assert!(period.validate(&int(value)).is_err());
    }
    assert!(period.validate(&float(1.5)).is_err());
}

#[test]
fn metrics_validate_item_enum_and_count() {
    let schema = budget_schema();
    let metrics = field(&schema, "budget.metrics");
    metrics.validate(&list(vec![])).unwrap();
    metrics
        .validate(&list(vec![enum_identifier("blended_cost")]))
        .unwrap();
    assert!(
        metrics
            .validate(&list(vec![
                enum_identifier("blended_cost"),
                enum_identifier("unblended_cost"),
            ]))
            .is_err(),
        "Metrics must contain at most one item"
    );
    assert!(
        metrics
            .validate(&list(vec![enum_identifier("invalid_metric")]))
            .is_err()
    );
    assert!(
        metrics
            .validate(&list(vec![string("BlendedCost")]))
            .is_err()
    );
}

#[test]
fn budget_filter_values_enforce_item_lengths_and_validate_count() {
    let schema = budget_schema();
    for prefix in [
        "budget.filter_expression",
        "budget.filter_expression.not.and",
        "budget.filter_expression.or",
    ] {
        for filter in ["tags", "cost_categories", "dimensions"] {
            let path = format!("{prefix}.{filter}.values");
            let values = field(&schema, &path);
            for length in [0, 1, 1024] {
                assert!(
                    values
                        .validate(&list(vec![string(&"x".repeat(length))]))
                        .is_ok(),
                    "{path}: item length {length} must be accepted"
                );
            }
            assert!(
                values
                    .validate(&list(vec![string("ok"), string(&"x".repeat(1025))]))
                    .is_err(),
                "{path}: every item must be at most 1024 characters"
            );
            values
                .validate(&list(vec![string(&"é".repeat(1024)), string("\n")]))
                .unwrap();
            assert!(
                values.validate(&list(vec![])).is_err(),
                "{path}: at least one item is required"
            );
            assert!(
                values.validate(&list(vec![string(""), string("")])).is_ok(),
                "{path}: multiple items and duplicates must remain allowed"
            );
        }
    }
}

#[test]
fn budget_dimension_match_options_validate_count_in_nested_and_expression() {
    let schema = budget_schema();
    let expression = field(&schema, "budget.filter_expression");
    let map = |key: &str, value| {
        Value::Concrete(ConcreteValue::Map(
            [(key.to_string(), value)].into_iter().collect(),
        ))
    };
    for filter in ["dimensions", "tags", "cost_categories"] {
        for count in [0, 1, 2] {
            let options = [enum_identifier("equals"), enum_identifier("case_sensitive")];
            let nested = map(
                "and",
                list(vec![map(
                    filter,
                    map("match_options", list(options[..count].to_vec())),
                )]),
            );
            assert_eq!(
                expression.validate(&nested).is_ok(),
                filter != "dimensions" || count <= 1,
                "nested And {filter}.MatchOptions with {count} items"
            );
        }
    }
}

#[test]
fn budget_names_enforce_length_and_supported_character_exclusions() {
    let schema = budget_schema();
    let name = field(&schema, "budget.budget_name");
    for value in [
        "a".to_string(),
        "x".repeat(100),
        "monthly-cost".to_string(),
        "é".repeat(100),
    ] {
        name.validate(&string(&value)).unwrap();
    }
    for value in [
        "".to_string(),
        "x".repeat(101),
        "monthly:cost".to_string(),
        r"monthly\cost".to_string(),
    ] {
        assert!(name.validate(&string(&value)).is_err(), "{value:?}");
    }
    // The API's lookahead exclusions cannot be compiled by core's regex engine.
    for value in ["monthly/action/cost", "<script>alert(1)</script>"] {
        name.validate(&string(value)).unwrap();
    }
}

#[test]
fn budget_tag_keys_enforce_documented_character_lengths() {
    let schema = budget_schema();
    for path in [
        "budget.filter_expression.tags.key",
        "budget.filter_expression.not.and.tags.key",
        "budget.filter_expression.or.tags.key",
    ] {
        let key = field(&schema, path);
        for length in [0, 1, 1024] {
            assert!(
                key.validate(&string(&"x".repeat(length))).is_ok(),
                "{path}: length {length} must be accepted"
            );
        }
        assert!(key.validate(&string(&"x".repeat(1025))).is_err(), "{path}");
        key.validate(&string(&"é".repeat(1024))).unwrap();
        key.validate(&string("\n")).unwrap();
    }
}

#[test]
fn identity_typed_outputs_are_assignable_to_every_refined_budget_string() {
    let schema = budget_schema();
    let topic = super::generated::sns::topic::sns_topic_config().schema;
    let vpc = super::generated::ec2::vpc::ec2_vpc_config().schema;

    // carina#4230 defers anonymous length/pattern checks until references resolve.
    let assignment = AssignmentContext::new();
    let assert_assignable = |path: &str| {
        let sink = field(&schema, path);
        let sink = sink.list_element().unwrap_or(sink);
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
                "{source_path} must be reference-compatible with Budget.{path}"
            );
        }
    };
    assert_assignable("budget.budget_name");
    for prefix in [
        "budget.filter_expression",
        "budget.filter_expression.not.and",
        "budget.filter_expression.or",
    ] {
        assert_assignable(&format!("{prefix}.tags.key"));
        for filter in ["tags", "cost_categories", "dimensions"] {
            assert_assignable(&format!("{prefix}.{filter}.values"));
        }
    }
}

#[test]
fn budgets_example_and_acceptance_fixture_validate() {
    validate_fixtures(&[
        (
            "examples/budgets_budget/main.crn",
            include_str!("../../../examples/budgets_budget/main.crn"),
        ),
        (
            "acceptance-tests/budgets_budget/basic.crn",
            include_str!("../../../acceptance-tests/budgets_budget/basic.crn"),
        ),
    ]);
}
