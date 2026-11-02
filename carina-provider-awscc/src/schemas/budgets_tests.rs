use std::collections::HashSet;
use std::path::Path;
use std::sync::Arc;

use carina_core::parser::{ProviderContext, parse_file};
use carina_core::provider::{ProviderFactory, provider_custom_types_from_factories};
use carina_core::resource::{ConcreteValue, Value};
use carina_core::schema::{EffectiveType, ShapeWalkBudget, ValidatedResourceSchema};
use carina_core::validation::validate_resources;
use serde_json::json;

use crate::provider::conversion::{aws_value_to_dsl_with_defs, dsl_value_to_aws_with_defs};

fn budget_schema() -> ValidatedResourceSchema {
    super::generated::budgets::budget::budgets_budget_config().schema
}

fn field<'a>(schema: &'a ValidatedResourceSchema, path: &str) -> EffectiveType<'a> {
    let mut segments = path.split('.');
    let mut attr_type = schema
        .effective_attribute(segments.next().unwrap())
        .unwrap()
        .value_type();
    for segment in segments {
        attr_type = attr_type.map_value().unwrap_or(attr_type);
        attr_type = attr_type
            .find_field_with_budget(segment, &mut ShapeWalkBudget::new(64))
            .unwrap_or_else(|| panic!("missing Budget field {path}"))
            .value_type();
    }
    attr_type
}

fn string(value: &str) -> Value {
    Value::Concrete(ConcreteValue::String(value.into()))
}

fn enum_identifier(value: &str) -> Value {
    Value::Concrete(ConcreteValue::enum_identifier(value))
}

fn float(value: f64) -> Value {
    Value::Concrete(ConcreteValue::Float(value))
}

fn int(value: i64) -> Value {
    Value::Concrete(ConcreteValue::Int(value))
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
fn metrics_validate_item_enum() {
    let schema = budget_schema();
    let metrics = field(&schema, "budget.metrics");
    let list = |items| Value::Concrete(ConcreteValue::List(items));
    metrics.validate(&list(vec![])).unwrap();
    metrics
        .validate(&list(vec![enum_identifier("blended_cost")]))
        .unwrap();
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
fn budget_names_validate_length_and_supported_pattern() {
    let schema = budget_schema();
    let name = field(&schema, "budget.budget_name");
    for value in ["a".to_string(), "x".repeat(100), "monthly-cost".to_string()] {
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
}

#[test]
fn budgets_example_and_acceptance_fixture_validate() {
    let factories: Arc<Vec<Box<dyn ProviderFactory>>> =
        Arc::new(vec![Box::new(crate::AwsccProviderFactory)]);
    let schemas = super::schema_registry();
    let context = ProviderContext {
        custom_types: provider_custom_types_from_factories(factories),
        resource_types: ProviderContext::resource_types_from_schema_registry(&schemas),
        ..ProviderContext::default()
    };
    for (path, source) in [
        (
            "examples/budgets_budget/main.crn",
            include_str!("../../../examples/budgets_budget/main.crn"),
        ),
        (
            "acceptance-tests/budgets_budget/basic.crn",
            include_str!("../../../acceptance-tests/budgets_budget/basic.crn"),
        ),
    ] {
        let parsed = parse_file(source, Path::new(path), &context)
            .unwrap_or_else(|error| panic!("{path}: {error}"));
        validate_resources(
            &parsed,
            &schemas,
            &HashSet::from(["awscc".to_string()]),
            &context,
        )
        .unwrap_or_else(|error| panic!("{path}: {error}"));
    }
}
