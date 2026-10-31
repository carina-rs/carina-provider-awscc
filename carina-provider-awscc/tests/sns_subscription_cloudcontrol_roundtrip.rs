//! Integration coverage for SNS Subscription through the provider's real
//! Provider-trait apply path against an in-process winterbaume CloudControl
//! mock. winterbaume-cloudcontrol 1.0.1 does not register a shaper for
//! `AWS::SNS::Subscription`, so this test explicitly seeds the measured
//! object-shaped policy read-back. Typed policies must survive structurally,
//! FilterPolicy must remain omitted, and Cloud Control's added
//! `guaranteed: false` must not produce a diff.

mod common;

use std::collections::HashMap;
use std::sync::Arc;

use aws_config::{BehaviorVersion, Region};
use carina_core::differ::{Diff, diff};
use carina_core::provider::{CreateOutcomeKind, Provider, RawSavedAttrs, ReadRequest};
use carina_core::resource::{ConcreteValue, Resource, Value};
use carina_core::schema::SchemaRegistry;
use carina_provider_awscc::provider::AwsccProviderConfig;
use carina_provider_awscc::schemas::generated::sns::subscription::sns_subscription_config;
use carina_provider_awscc::{AwsccNormalizer, AwsccProvider};
use indexmap::IndexMap;
use winterbaume_cloudcontrol::CloudControlService;
use winterbaume_core::{MockAws, StatefulService};

const TYPE_NAME: &str = "AWS::SNS::Subscription";
const ACCOUNT_ID: &str = "123456789012";
const REGION: &str = "ap-northeast-1";
const TOPIC_ARN: &str = "arn:aws:sns:ap-northeast-1:123456789012:carina-registry-alerts";
const ENDPOINT_ARN: &str = "arn:aws:sqs:ap-northeast-1:123456789012:carina-registry-alerts";
const DEAD_LETTER_ARN: &str = "arn:aws:sqs:ap-northeast-1:123456789012:carina-registry-alerts-dlq";

fn string(value: &str) -> Value {
    Value::Concrete(ConcreteValue::String(value.to_string().into()))
}

fn bool_value(value: bool) -> Value {
    Value::Concrete(ConcreteValue::Bool(value))
}

fn int(value: i64) -> Value {
    Value::Concrete(ConcreteValue::Int(value))
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

fn delivery_policy(guaranteed: Option<bool>) -> Value {
    let mut entries = vec![
        (
            "healthy_retry_policy",
            map([
                ("min_delay_target", int(1)),
                ("max_delay_target", int(20)),
                ("num_retries", int(3)),
                ("num_no_delay_retries", int(0)),
                ("num_min_delay_retries", int(1)),
                ("num_max_delay_retries", int(2)),
                ("backoff_function", enum_identifier("linear")),
            ]),
        ),
        (
            "throttle_policy",
            map([("max_receives_per_second", int(10))]),
        ),
    ];
    if let Some(guaranteed) = guaranteed {
        entries.push(("guaranteed", bool_value(guaranteed)));
    }
    map(entries)
}

fn subscription_resource(guaranteed: Option<bool>) -> Resource {
    Resource::with_provider("awscc", "sns.Subscription", "alerts", None)
        .with_attribute("topic_arn", string(TOPIC_ARN))
        .with_attribute("protocol", string("sqs"))
        .with_attribute("endpoint", string(ENDPOINT_ARN))
        .with_attribute("raw_message_delivery", bool_value(true))
        .with_attribute(
            "redrive_policy",
            map([("dead_letter_target_arn", string(DEAD_LETTER_ARN))]),
        )
        .with_attribute("delivery_policy", delivery_policy(guaranteed))
        .with_attribute(
            "replay_policy",
            map([
                ("point_type", enum_identifier("timestamp")),
                ("starting_point", string("2026-10-31T00:00:00Z")),
                ("ending_point", string("2026-10-31T01:00:00Z")),
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

async fn stored_subscription(cloudcontrol: &CloudControlService) -> (String, serde_json::Value) {
    let view = cloudcontrol.snapshot(ACCOUNT_ID, REGION).await;
    let resources = view
        .resources
        .values()
        .filter(|resource| resource.type_name == TYPE_NAME)
        .collect::<Vec<_>>();
    assert_eq!(
        resources.len(),
        1,
        "expected exactly one stored subscription"
    );
    let resource = resources[0];
    (
        resource.identifier.clone(),
        serde_json::from_str(&resource.resource_model)
            .expect("stored subscription model should be JSON"),
    )
}

async fn seed_measured_policy_readback(cloudcontrol: &CloudControlService) {
    let mut view = cloudcontrol.snapshot(ACCOUNT_ID, REGION).await;
    let resource = view
        .resources
        .values_mut()
        .find(|resource| resource.type_name == TYPE_NAME)
        .expect("stored subscription to seed");
    let mut model: serde_json::Value = serde_json::from_str(&resource.resource_model)
        .expect("stored subscription model should be JSON");
    let model = model
        .as_object_mut()
        .expect("stored subscription model should be an object");
    model.insert(
        "FilterPolicy".to_string(),
        serde_json::json!({"event": ["created"]}),
    );
    model
        .get_mut("DeliveryPolicy")
        .and_then(serde_json::Value::as_object_mut)
        .expect("stored DeliveryPolicy should be a native JSON object")
        .insert("guaranteed".to_string(), serde_json::json!(false));
    resource.resource_model =
        serde_json::to_string(&model).expect("seeded subscription model JSON");
    cloudcontrol
        .restore(ACCOUNT_ID, REGION, view)
        .await
        .expect("seed stored subscription read model");
}

#[test]
fn subscription_schema_types_documented_policies_and_omits_only_filter_policy() {
    let config = sns_subscription_config();
    config
        .schema
        .clone()
        .validate()
        .expect("SNS Subscription schema must pass admission");

    for attribute in ["delivery_policy", "redrive_policy", "replay_policy"] {
        assert!(config.schema.attributes().contains_key(attribute));
    }
    assert!(!config.schema.attributes().contains_key("filter_policy"));
}

#[tokio::test(start_paused = true)]
async fn sns_subscription_typed_policies_round_trip_and_server_default_has_no_diff() {
    let (provider, cloudcontrol) = winterbaume_provider().await;
    let resource = subscription_resource(None);
    let id = resource.id.clone();
    let request = common::create_request(resource.clone()).await;

    let created = carina_core::executor::dispatch_create(&provider, &id, request)
        .await
        .expect("sns.Subscription create_resource should succeed");
    let created = match created.into_kind() {
        CreateOutcomeKind::Success { state } | CreateOutcomeKind::PartialSuccess { state, .. } => {
            state
        }
    };

    let (stored_identifier, desired_state) = stored_subscription(&cloudcontrol).await;
    assert_eq!(created.identifier().as_str(), stored_identifier);
    assert_eq!(desired_state["TopicArn"], TOPIC_ARN);
    assert_eq!(desired_state["Protocol"], "sqs");
    assert_eq!(desired_state["Endpoint"], ENDPOINT_ARN);
    assert!(desired_state.get("FilterPolicy").is_none());
    assert_eq!(desired_state["RawMessageDelivery"], true);
    assert_eq!(
        desired_state["RedrivePolicy"],
        serde_json::json!({"deadLetterTargetArn": DEAD_LETTER_ARN})
    );
    assert_eq!(
        desired_state["DeliveryPolicy"],
        serde_json::json!({
            "healthyRetryPolicy": {
                "minDelayTarget": 1,
                "maxDelayTarget": 20,
                "numRetries": 3,
                "numNoDelayRetries": 0,
                "numMinDelayRetries": 1,
                "numMaxDelayRetries": 2,
                "backoffFunction": "linear"
            },
            "throttlePolicy": {"maxReceivesPerSecond": 10}
        })
    );
    assert_eq!(
        desired_state["ReplayPolicy"],
        serde_json::json!({
            "PointType": "Timestamp",
            "StartingPoint": "2026-10-31T00:00:00Z",
            "EndingPoint": "2026-10-31T01:00:00Z"
        })
    );

    seed_measured_policy_readback(&cloudcontrol).await;
    let (_, seeded_state) = stored_subscription(&cloudcontrol).await;
    assert_eq!(
        seeded_state["FilterPolicy"],
        serde_json::json!({"event": ["created"]})
    );
    assert_eq!(seeded_state["DeliveryPolicy"]["guaranteed"], false);

    let stored_identifier = common::applied_identifier(stored_identifier);
    let read = Provider::read(&provider, &id, &stored_identifier, ReadRequest)
        .await
        .expect("sns.Subscription read through Provider::read should succeed");
    let read = common::found_state(read);

    assert_eq!(read.attributes().get("topic_arn"), Some(&string(TOPIC_ARN)));
    assert_eq!(read.attributes().get("protocol"), Some(&string("sqs")));
    assert_eq!(
        read.attributes().get("endpoint"),
        Some(&string(ENDPOINT_ARN))
    );
    assert!(!read.attributes().contains_key("filter_policy"));
    assert!(read.attributes().contains_key("redrive_policy"));
    assert!(read.attributes().contains_key("replay_policy"));
    let Value::Concrete(ConcreteValue::Map(delivery_read)) = read
        .attributes()
        .get("delivery_policy")
        .expect("typed delivery_policy must survive read conversion")
    else {
        panic!("delivery_policy must read as a struct map");
    };
    assert_eq!(
        delivery_read.get("guaranteed"),
        Some(&bool_value(false)),
        "measured Cloud Control read-back adds guaranteed=false"
    );
    assert_eq!(
        read.attributes().get("raw_message_delivery"),
        Some(&bool_value(true))
    );

    let unnormalized_current = read.into_state_for_writeback();
    let desired_without_guaranteed = common::normalized_resolved_resource(resource).await;
    let desired_guaranteed_false =
        common::normalized_resolved_resource(subscription_resource(Some(false))).await;
    let desired_guaranteed_true =
        common::normalized_resolved_resource(subscription_resource(Some(true))).await;
    let saved = RawSavedAttrs::default().lift(&SchemaRegistry::new());
    let config = sns_subscription_config();

    // Exercise the repository's real differ before any provider read-state
    // normalization. This establishes whether nested fields absent from the
    // desired struct need provider-side server-default handling at all.
    let result = diff(
        &desired_without_guaranteed,
        &unnormalized_current,
        &saved,
        None,
        Some(&config.schema),
    );
    assert!(
        matches!(result, Diff::NoChange(_)),
        "(a) omitted guaranteed versus read guaranteed=false should be NoChange: {result:?}"
    );

    let result = diff(
        &desired_guaranteed_false,
        &unnormalized_current,
        &saved,
        None,
        Some(&config.schema),
    );
    assert!(
        matches!(result, Diff::NoChange(_)),
        "(b) explicit guaranteed=false versus read false should be NoChange: {result:?}"
    );

    let result = diff(
        &desired_guaranteed_true,
        &unnormalized_current,
        &saved,
        None,
        Some(&config.schema),
    );
    assert!(
        !matches!(result, Diff::NoChange(_)),
        "(c) explicit guaranteed=true versus read false must differ"
    );

    let mut current_states = HashMap::from([(id.clone(), unnormalized_current)]);
    carina_core::provider::normalize_provider_states(&AwsccNormalizer, &mut current_states)
        .await
        .expect("read state normalization should succeed");
    let current = current_states
        .get(&id)
        .expect("normalized subscription state must remain present");
    let result = diff(
        &desired_without_guaranteed,
        current,
        &saved,
        None,
        Some(&config.schema),
    );
    assert!(
        matches!(result, Diff::NoChange(_)),
        "typed policy read-back plus guaranteed=false must not produce a diff: {result:?}"
    );

    let result = diff(
        &desired_guaranteed_true,
        current,
        &saved,
        None,
        Some(&config.schema),
    );
    assert!(
        !matches!(result, Diff::NoChange(_)),
        "explicit guaranteed=true must still differ from a read of false"
    );
}
