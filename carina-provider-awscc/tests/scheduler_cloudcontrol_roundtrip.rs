//! Integration coverage for Scheduler resources through the provider's real
//! Provider-trait apply path against an in-process winterbaume CloudControl
//! mock. The ScheduleGroup is independent; the schedule uses the only group
//! that Cloud Control can read by Schedule.Name: `default`.

mod common;

use std::sync::Arc;

use aws_config::{BehaviorVersion, Region};
use carina_core::provider::{AppliedState, CreateOutcomeKind, Provider, ReadRequest};
use carina_core::resource::{ConcreteValue, Resource, Value};
use carina_core::schema::{AttributeType, RawShape, StructField};
use carina_provider_awscc::AwsccProvider;
use carina_provider_awscc::provider::AwsccProviderConfig;
use carina_provider_awscc::schemas::generated::scheduler::schedule::scheduler_schedule_config;
use carina_provider_awscc::schemas::generated::scheduler::schedule_group::scheduler_schedule_group_config;
use indexmap::IndexMap;
use winterbaume_cloudcontrol::CloudControlService;
use winterbaume_core::{MockAws, StatefulService};

const SCHEDULE_TYPE_NAME: &str = "AWS::Scheduler::Schedule";
const SCHEDULE_GROUP_TYPE_NAME: &str = "AWS::Scheduler::ScheduleGroup";
const ACCOUNT_ID: &str = "123456789012";
const REGION: &str = "us-east-1";
const GROUP_NAME: &str = "carina-registry-publish-api";
const DEFAULT_GROUP_NAME: &str = "default";
const SCHEDULE_NAME: &str = "carina-registry-publish-refresh";
const TARGET_ARN: &str =
    "arn:aws:lambda:us-east-1:123456789012:function:carina-registry-publish-api:live";
const ROLE_ARN: &str = "arn:aws:iam::123456789012:role/carina-registry-publish-scheduler";
const TARGET_INPUT: &str = r#"{"operation":"refresh"}"#;

fn string(value: &str) -> Value {
    Value::Concrete(ConcreteValue::String(value.to_string().into()))
}

fn enum_identifier(value: &str) -> Value {
    Value::Concrete(ConcreteValue::enum_identifier(value))
}

fn map(entries: impl IntoIterator<Item = (&'static str, Value)>) -> Value {
    Value::Concrete(ConcreteValue::Map(
        entries
            .into_iter()
            .map(|(key, value)| (key.to_string(), value))
            .collect::<IndexMap<_, _>>(),
    ))
}

fn find_field<'a>(attribute_type: &'a AttributeType, field_name: &str) -> &'a StructField {
    let RawShape::Struct { fields, .. } = attribute_type.raw_shape() else {
        panic!(
            "expected {field_name} parent to be a struct, got {:?}",
            attribute_type.raw_shape()
        );
    };
    fields
        .iter()
        .find(|field| field.name() == field_name)
        .unwrap_or_else(|| panic!("expected struct field {field_name}"))
}

fn schedule_group_resource() -> Resource {
    Resource::with_provider("awscc", "scheduler.ScheduleGroup", "publish_api", None)
        .with_attribute("name", string(GROUP_NAME))
        .with_attribute(
            "tags",
            map([
                ("Project", string("carina-registry")),
                ("Purpose", string("publish-api")),
            ]),
        )
}

fn schedule_resource() -> Resource {
    Resource::with_provider("awscc", "scheduler.Schedule", "publish_refresh", None)
        .with_attribute("name", string(SCHEDULE_NAME))
        .with_attribute("group_name", string(DEFAULT_GROUP_NAME))
        .with_attribute(
            "flexible_time_window",
            map([("mode", enum_identifier("off"))]),
        )
        .with_attribute("schedule_expression", string("rate(6 hours)"))
        .with_attribute("state", enum_identifier("enabled"))
        .with_attribute(
            "target",
            map([
                ("arn", string(TARGET_ARN)),
                ("role_arn", string(ROLE_ARN)),
                ("input", string(TARGET_INPUT)),
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

async fn stored_resource(
    cloudcontrol: &CloudControlService,
    type_name: &str,
) -> (String, serde_json::Value) {
    let view = cloudcontrol.snapshot(ACCOUNT_ID, REGION).await;
    let resources = view
        .resources
        .values()
        .filter(|resource| resource.type_name == type_name)
        .collect::<Vec<_>>();
    assert_eq!(
        resources.len(),
        1,
        "winterbaume should store exactly one {type_name} resource"
    );
    let resource = resources[0];
    (
        resource.identifier.clone(),
        serde_json::from_str(&resource.resource_model)
            .expect("stored resource model should be JSON"),
    )
}

async fn create(provider: &AwsccProvider, resource: Resource) -> AppliedState {
    let id = resource.id.clone();
    let request = common::create_request(resource).await;
    let created = carina_core::executor::dispatch_create(provider, &id, request)
        .await
        .expect("Scheduler create_resource should succeed");
    match created.into_kind() {
        CreateOutcomeKind::Success { state } => state,
        CreateOutcomeKind::PartialSuccess { diagnostic, .. } => {
            panic!("roundtrip create should be full success, got partial: {diagnostic:?}")
        }
    }
}

#[test]
fn maximum_window_in_minutes_uses_the_integer_dsl_shape() {
    let schedule = scheduler_schedule_config();
    let flexible_time_window = &schedule
        .schema
        .attributes()
        .get("flexible_time_window")
        .expect("Schedule.flexible_time_window should exist")
        .attr_type();
    let maximum_window = find_field(flexible_time_window, "maximum_window_in_minutes").field_type();

    assert!(
        matches!(maximum_window.raw_shape(), RawShape::Int { .. }),
        "MaximumWindowInMinutes must accept an integer DSL value, got {:?}",
        maximum_window.raw_shape()
    );
}

#[test]
fn schedule_group_exposes_tags_but_schedule_does_not() {
    let group_config = scheduler_schedule_group_config();
    assert!(
        group_config.schema.attributes().contains_key("tags"),
        "ScheduleGroup must expose Tags"
    );
    assert!(
        !scheduler_schedule_config()
            .schema
            .attributes()
            .contains_key("tags"),
        "Schedule must not expose Tags"
    );
}

#[tokio::test(start_paused = true)]
async fn schedule_group_and_default_group_schedule_round_trip() {
    let (provider, cloudcontrol) = winterbaume_provider().await;

    let group = schedule_group_resource();
    let group_id = group.id.clone();
    let group_state = create(&provider, group).await;
    let (stored_group_identifier, stored_group) =
        stored_resource(&cloudcontrol, SCHEDULE_GROUP_TYPE_NAME).await;
    assert_eq!(group_state.identifier().as_str(), stored_group_identifier);
    assert_eq!(stored_group["Name"], GROUP_NAME);
    assert_eq!(
        stored_group["Tags"],
        serde_json::json!([
            {"Key": "Project", "Value": "carina-registry"},
            {"Key": "Purpose", "Value": "publish-api"}
        ])
    );

    let schedule = schedule_resource();
    let schedule_id = schedule.id.clone();
    let schedule_state = create(&provider, schedule).await;
    let (stored_schedule_identifier, stored_schedule) =
        stored_resource(&cloudcontrol, SCHEDULE_TYPE_NAME).await;
    assert_eq!(
        schedule_state.identifier().as_str(),
        stored_schedule_identifier
    );
    assert_eq!(stored_schedule["Name"], SCHEDULE_NAME);
    assert_eq!(stored_schedule["GroupName"], DEFAULT_GROUP_NAME);
    assert_eq!(
        stored_schedule["FlexibleTimeWindow"],
        serde_json::json!({"Mode": "OFF"})
    );
    assert_eq!(stored_schedule["ScheduleExpression"], "rate(6 hours)");
    assert_eq!(stored_schedule["State"], "ENABLED");
    assert_eq!(
        stored_schedule["Target"],
        serde_json::json!({
            "Arn": TARGET_ARN,
            "RoleArn": ROLE_ARN,
            "Input": TARGET_INPUT
        })
    );

    let group_identifier = common::applied_identifier(stored_group_identifier);
    let read_group = Provider::read(&provider, &group_id, &group_identifier, ReadRequest)
        .await
        .expect("scheduler.ScheduleGroup read should succeed");
    let read_group = common::found_state(read_group);
    assert_eq!(
        read_group.attributes().get("tags"),
        Some(&map([
            ("Project", string("carina-registry")),
            ("Purpose", string("publish-api")),
        ]))
    );

    let schedule_identifier = common::applied_identifier(stored_schedule_identifier);
    let read_schedule = Provider::read(&provider, &schedule_id, &schedule_identifier, ReadRequest)
        .await
        .expect("scheduler.Schedule read should succeed");
    let read_schedule = common::found_state(read_schedule);
    assert_eq!(
        read_schedule.attributes().get("group_name"),
        Some(&string(DEFAULT_GROUP_NAME))
    );
    assert_eq!(
        read_schedule.attributes().get("schedule_expression"),
        Some(&string("rate(6 hours)"))
    );
    let Some(Value::Concrete(ConcreteValue::Map(target))) =
        read_schedule.attributes().get("target")
    else {
        panic!("Schedule.target must round-trip as a map")
    };
    assert_eq!(target.get("arn"), Some(&string(TARGET_ARN)));
    assert_eq!(target.get("role_arn"), Some(&string(ROLE_ARN)));
    assert_eq!(target.get("input"), Some(&string(TARGET_INPUT)));
}
