//! Integration coverage for Budgets Budget through the provider's real
//! Provider-trait apply path against an in-process winterbaume CloudControl
//! mock.
//!
//! winterbaume-cloudcontrol 1.0.1 does not register a shaper for
//! `AWS::Budgets::Budget`, so the mock returns the create-time DesiredState
//! verbatim. This test therefore asserts structural preservation only. In
//! particular, the recursive `Expression.And` and `Expression.Not` branches,
//! decimal money/notification fields, subscribers, and resource tags must all
//! survive as structured values. Upgrade this to full shaped equality once
//! winterbaume registers this resource type.

mod common;

use std::sync::Arc;

use aws_config::{BehaviorVersion, Region};
use carina_core::provider::{CreateOutcomeKind, Provider, ReadRequest};
use carina_core::resource::{ConcreteValue, Resource, Value};
use carina_core::schema::RawShape;
use carina_provider_awscc::AwsccProvider;
use carina_provider_awscc::provider::AwsccProviderConfig;
use carina_provider_awscc::schemas::generated::budgets::budget::budgets_budget_config;
use indexmap::IndexMap;
use winterbaume_cloudcontrol::CloudControlService;
use winterbaume_core::{MockAws, StatefulService};

const TYPE_NAME: &str = "AWS::Budgets::Budget";
const ACCOUNT_ID: &str = "123456789012";
const REGION: &str = "us-east-1";
const BUDGET_NAME: &str = "carina-registry-monthly-cost";

fn string(value: &str) -> Value {
    Value::Concrete(ConcreteValue::String(value.to_string().into()))
}

fn float(value: f64) -> Value {
    Value::Concrete(ConcreteValue::Float(value))
}

fn map(entries: impl IntoIterator<Item = (&'static str, Value)>) -> Value {
    Value::Concrete(ConcreteValue::Map(
        entries
            .into_iter()
            .map(|(key, value)| (key.to_string(), value))
            .collect::<IndexMap<_, _>>(),
    ))
}

fn list(items: impl IntoIterator<Item = Value>) -> Value {
    Value::Concrete(ConcreteValue::List(items.into_iter().collect()))
}

fn filter_expression() -> Value {
    map([(
        "and",
        list([
            map([(
                "dimensions",
                map([
                    ("key", string("REGION")),
                    ("match_options", list([string("EQUALS")])),
                    ("values", list([string("ap-northeast-1")])),
                ]),
            )]),
            map([(
                "not",
                map([(
                    "tags",
                    map([
                        ("key", string("Environment")),
                        ("match_options", list([string("EQUALS")])),
                        ("values", list([string("production")])),
                    ]),
                )]),
            )]),
        ]),
    )])
}

fn budget_resource() -> Resource {
    Resource::with_provider("awscc", "budgets.Budget", "monthly_cost", None)
        .with_attribute(
            "budget",
            map([
                (
                    "budget_limit",
                    map([("amount", float(5.25)), ("unit", string("USD"))]),
                ),
                ("budget_name", string(BUDGET_NAME)),
                ("budget_type", string("COST")),
                ("filter_expression", filter_expression()),
                ("time_unit", string("MONTHLY")),
            ]),
        )
        .with_attribute(
            "notifications_with_subscribers",
            list([map([
                (
                    "notification",
                    map([
                        ("comparison_operator", string("GREATER_THAN")),
                        ("notification_type", string("ACTUAL")),
                        ("threshold", float(80.5)),
                        ("threshold_type", string("PERCENTAGE")),
                    ]),
                ),
                (
                    "subscribers",
                    list([map([
                        ("address", string("budget@example.com")),
                        ("subscription_type", string("EMAIL")),
                    ])]),
                ),
            ])]),
        )
        .with_attribute(
            "resource_tags",
            list([map([
                ("key", string("Project")),
                ("value", string("carina-registry")),
            ])]),
        )
}

async fn winterbaume_provider() -> (AwsccProvider, Arc<CloudControlService>) {
    let cloudcontrol = Arc::new(CloudControlService::new());
    let mock = MockAws::builder()
        .with_service(Arc::clone(&cloudcontrol))
        .build();
    let config = aws_config::defaults(BehaviorVersion::latest())
        .http_client(mock.http_client())
        .credentials_provider(mock.credentials_provider())
        .region(Region::new(REGION))
        .load()
        .await;

    (
        AwsccProvider::from_sdk_config(
            carina_provider_awscc::BoundedSdkConfig::new(config),
            &AwsccProviderConfig::default(),
        )
        .await,
        cloudcontrol,
    )
}

async fn stored_budget(cloudcontrol: &CloudControlService) -> (String, serde_json::Value) {
    let view = cloudcontrol.snapshot(ACCOUNT_ID, REGION).await;
    let resources = view
        .resources
        .values()
        .filter(|resource| resource.type_name == TYPE_NAME)
        .collect::<Vec<_>>();
    assert_eq!(resources.len(), 1, "expected exactly one stored budget");
    let resource = resources[0];
    (
        resource.identifier.clone(),
        serde_json::from_str(&resource.resource_model).expect("stored budget model should be JSON"),
    )
}

#[test]
fn budget_schema_admits_recursive_expressions_and_float_amounts() {
    let config = budgets_budget_config();
    config
        .schema
        .clone()
        .validate()
        .expect("Budgets Budget schema must admit recursive refs");

    let budget = config
        .schema
        .attributes()
        .get("budget")
        .expect("Budget.budget should exist");
    let RawShape::Struct { fields, .. } = budget.attr_type().raw_shape() else {
        panic!("Budget.budget should be a struct");
    };
    let budget_limit = fields
        .iter()
        .find(|field| field.name() == "budget_limit")
        .expect("Budget.budget_limit should exist");
    let RawShape::Struct { fields, .. } = budget_limit.field_type().raw_shape() else {
        panic!("Budget.budget_limit should be a struct");
    };
    let amount = fields
        .iter()
        .find(|field| field.name() == "amount")
        .expect("Budget.budget_limit.amount should exist");
    assert!(matches!(
        amount.field_type().raw_shape(),
        RawShape::Float { .. }
    ));

    let expression = config
        .schema
        .defs()
        .get("Expression")
        .expect("recursive Expression definition should be emitted");
    let RawShape::Struct { fields, .. } = expression.raw_shape() else {
        panic!("Expression should be a struct");
    };
    let and = fields
        .iter()
        .find(|field| field.name() == "and")
        .expect("Expression.and should exist");
    assert!(matches!(
        and.field_type().raw_shape(),
        RawShape::List { element_type, .. }
            if matches!(element_type.raw_shape(), RawShape::Ref(name) if name == "Expression")
    ));
    let not = fields
        .iter()
        .find(|field| field.name() == "not")
        .expect("Expression.not should exist");
    assert!(matches!(
        not.field_type().raw_shape(),
        RawShape::Ref(name) if name == "Expression"
    ));
}

#[tokio::test(start_paused = true)]
async fn budgets_budget_create_then_read_preserves_recursive_expression_and_notifications() {
    let (provider, cloudcontrol) = winterbaume_provider().await;
    let resource = budget_resource();
    let id = resource.id.clone();
    let request = common::create_request(resource).await;

    let created = common::dispatch_create(&provider, &id, request)
        .await
        .expect("budgets.Budget create_resource should succeed");
    let created = match created.into_kind() {
        CreateOutcomeKind::Success { state } | CreateOutcomeKind::PartialSuccess { state, .. } => {
            state
        }
    };

    let (stored_identifier, desired_state) = stored_budget(&cloudcontrol).await;
    assert_eq!(created.identifier().as_str(), stored_identifier);
    assert_eq!(desired_state["Budget"]["BudgetName"], BUDGET_NAME);
    assert_eq!(desired_state["Budget"]["BudgetLimit"]["Amount"], 5.25);
    assert_eq!(
        desired_state["Budget"]["FilterExpression"],
        serde_json::json!({
            "And": [
                {
                    "Dimensions": {
                        "Key": "REGION",
                        "MatchOptions": ["EQUALS"],
                        "Values": ["ap-northeast-1"]
                    }
                },
                {
                    "Not": {
                        "Tags": {
                            "Key": "Environment",
                            "MatchOptions": ["EQUALS"],
                            "Values": ["production"]
                        }
                    }
                }
            ]
        })
    );
    assert_eq!(
        desired_state["NotificationsWithSubscribers"],
        serde_json::json!([{
            "Notification": {
                "ComparisonOperator": "GREATER_THAN",
                "NotificationType": "ACTUAL",
                "Threshold": 80.5,
                "ThresholdType": "PERCENTAGE"
            },
            "Subscribers": [{
                "Address": "budget@example.com",
                "SubscriptionType": "EMAIL"
            }]
        }])
    );
    assert_eq!(
        desired_state["ResourceTags"],
        serde_json::json!([{"Key": "Project", "Value": "carina-registry"}])
    );

    let stored_identifier = common::applied_identifier(stored_identifier);
    let read = Provider::read(&provider, &id, &stored_identifier, ReadRequest)
        .await
        .expect("budgets.Budget read through Provider::read should succeed");
    let read = common::found_state(read);

    let Some(Value::Concrete(ConcreteValue::Map(budget))) = read.attributes().get("budget") else {
        panic!("budget must round-trip as a map");
    };
    let Some(Value::Concrete(ConcreteValue::Map(limit))) = budget.get("budget_limit") else {
        panic!("budget_limit must round-trip as a map");
    };
    assert_eq!(limit.get("amount"), Some(&float(5.25)));
    assert_eq!(limit.get("unit"), Some(&string("USD")));
    assert_eq!(budget.get("filter_expression"), Some(&filter_expression()));

    assert_eq!(
        read.attributes().get("notifications_with_subscribers"),
        Some(&list([map([
            (
                "notification",
                map([
                    ("comparison_operator", string("GREATER_THAN")),
                    ("notification_type", string("ACTUAL")),
                    ("threshold", float(80.5)),
                    ("threshold_type", string("PERCENTAGE")),
                ]),
            ),
            (
                "subscribers",
                list([map([
                    ("address", string("budget@example.com")),
                    ("subscription_type", string("EMAIL")),
                ])]),
            ),
        ])]))
    );
    assert_eq!(
        read.attributes().get("resource_tags"),
        Some(&list([map([
            ("key", string("Project")),
            ("value", string("carina-registry")),
        ])]))
    );
}
