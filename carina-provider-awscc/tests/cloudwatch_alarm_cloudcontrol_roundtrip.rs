//! Integration coverage for CloudWatch Alarm through the provider's real
//! Provider-trait apply path against an in-process winterbaume CloudControl
//! mock.
//!
//! winterbaume-cloudcontrol 1.0.1 does not register a shaper for
//! `AWS::CloudWatch::Alarm`, so the mock returns the create-time DesiredState
//! verbatim. This test therefore asserts structural preservation only: integer
//! periods remain integers, the decimal threshold remains a float, nested
//! dimensions and the evaluation window remain structured, and resource tags
//! round-trip through the provider's map/list wire conversion. Upgrade this to
//! full shaped equality once winterbaume registers this resource type.

mod common;

use std::sync::Arc;

use aws_config::{BehaviorVersion, Region};
use carina_core::provider::{CreateOutcomeKind, Provider, ReadRequest};
use carina_core::resource::{ConcreteValue, Resource, Value};
use carina_core::schema::RawShape;
use carina_provider_awscc::AwsccProvider;
use carina_provider_awscc::provider::AwsccProviderConfig;
use carina_provider_awscc::schemas::generated::cloudwatch::alarm::cloudwatch_alarm_config;
use indexmap::IndexMap;
use winterbaume_cloudcontrol::CloudControlService;
use winterbaume_core::{MockAws, StatefulService};

const TYPE_NAME: &str = "AWS::CloudWatch::Alarm";
const ACCOUNT_ID: &str = "123456789012";
const REGION: &str = "ap-northeast-1";
const ALARM_NAME: &str = "carina-registry-api-errors";

fn string(value: &str) -> Value {
    Value::Concrete(ConcreteValue::String(value.to_string().into()))
}

fn int(value: i64) -> Value {
    Value::Concrete(ConcreteValue::Int(value))
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

fn alarm_resource() -> Resource {
    Resource::with_provider("awscc", "cloudwatch.Alarm", "api_errors", None)
        .with_attribute("alarm_name", string(ALARM_NAME))
        .with_attribute("alarm_description", string("Registry API errors"))
        .with_attribute("comparison_operator", string("GreaterThanThreshold"))
        .with_attribute("evaluation_periods", int(2))
        .with_attribute("datapoints_to_alarm", int(1))
        .with_attribute("metric_name", string("Errors"))
        .with_attribute("namespace", string("Carina/Registry"))
        .with_attribute("period", int(60))
        .with_attribute("statistic", string("Sum"))
        .with_attribute("threshold", float(0.5))
        .with_attribute(
            "dimensions",
            list([map([
                ("name", string("Service")),
                ("value", string("registry-api")),
            ])]),
        )
        .with_attribute(
            "evaluation_window",
            map([(
                "wall_clock_window",
                map([("timezone", string("Asia/Tokyo"))]),
            )]),
        )
        .with_attribute(
            "tags",
            map([
                ("Environment", string("test")),
                ("Project", string("carina-registry")),
            ]),
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

async fn stored_alarm(cloudcontrol: &CloudControlService) -> (String, serde_json::Value) {
    let view = cloudcontrol.snapshot(ACCOUNT_ID, REGION).await;
    let resources = view
        .resources
        .values()
        .filter(|resource| resource.type_name == TYPE_NAME)
        .collect::<Vec<_>>();
    assert_eq!(resources.len(), 1, "expected exactly one stored alarm");
    let resource = resources[0];
    (
        resource.identifier.clone(),
        serde_json::from_str(&resource.resource_model).expect("stored alarm model should be JSON"),
    )
}

#[test]
fn alarm_numeric_fields_keep_their_dsl_shapes() {
    let config = cloudwatch_alarm_config();
    config
        .schema
        .clone()
        .validate()
        .expect("CloudWatch Alarm schema must pass admission");

    for field in [
        "evaluation_periods",
        "datapoints_to_alarm",
        "evaluation_interval",
        "period",
    ] {
        let attribute = config
            .schema
            .attributes()
            .get(field)
            .unwrap_or_else(|| panic!("Alarm.{field} should exist"));
        assert!(
            matches!(attribute.attr_type().raw_shape(), RawShape::Int { .. }),
            "Alarm.{field} must use Int"
        );
    }

    let threshold = config
        .schema
        .attributes()
        .get("threshold")
        .expect("Alarm.threshold should exist");
    assert!(
        matches!(threshold.attr_type().raw_shape(), RawShape::Float { .. }),
        "Alarm.threshold must remain Float"
    );
}

#[tokio::test(start_paused = true)]
async fn cloudwatch_alarm_create_then_read_preserves_structures_and_numeric_shapes() {
    let (provider, cloudcontrol) = winterbaume_provider().await;
    let resource = alarm_resource();
    let id = resource.id.clone();
    let request = common::create_request(resource).await;

    let created = common::dispatch_create(&provider, &id, request)
        .await
        .expect("cloudwatch.Alarm create_resource should succeed");
    let created = match created.into_kind() {
        CreateOutcomeKind::Success { state } | CreateOutcomeKind::PartialSuccess { state, .. } => {
            state
        }
    };

    let (stored_identifier, desired_state) = stored_alarm(&cloudcontrol).await;
    assert_eq!(created.identifier().as_str(), ALARM_NAME);
    assert_eq!(desired_state["AlarmName"], ALARM_NAME);
    assert_eq!(desired_state["EvaluationPeriods"], 2);
    assert_eq!(desired_state["DatapointsToAlarm"], 1);
    assert_eq!(desired_state["Period"], 60);
    assert_eq!(desired_state["Threshold"], 0.5);
    assert_eq!(
        desired_state["Dimensions"],
        serde_json::json!([{"Name": "Service", "Value": "registry-api"}])
    );
    assert_eq!(
        desired_state["EvaluationWindow"],
        serde_json::json!({"WallClockWindow": {"Timezone": "Asia/Tokyo"}})
    );
    assert_eq!(
        desired_state["Tags"],
        serde_json::json!([
            {"Key": "Environment", "Value": "test"},
            {"Key": "Project", "Value": "carina-registry"}
        ])
    );

    let stored_identifier = common::applied_identifier(stored_identifier);
    let read = Provider::read(&provider, &id, &stored_identifier, ReadRequest)
        .await
        .expect("cloudwatch.Alarm read through Provider::read should succeed");
    let read = common::found_state(read);

    assert_eq!(read.attributes().get("evaluation_periods"), Some(&int(2)));
    assert_eq!(read.attributes().get("datapoints_to_alarm"), Some(&int(1)));
    assert_eq!(read.attributes().get("period"), Some(&int(60)));
    assert_eq!(read.attributes().get("threshold"), Some(&float(0.5)));
    assert_eq!(
        read.attributes().get("dimensions"),
        Some(&list([map([
            ("name", string("Service")),
            ("value", string("registry-api")),
        ])]))
    );
    assert_eq!(
        read.attributes().get("evaluation_window"),
        Some(&map([(
            "wall_clock_window",
            map([("timezone", string("Asia/Tokyo"))]),
        )]))
    );
    assert_eq!(
        read.attributes().get("tags"),
        Some(&map([
            ("Environment", string("test")),
            ("Project", string("carina-registry")),
        ]))
    );
}
