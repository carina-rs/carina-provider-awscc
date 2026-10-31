//! Integration coverage for SNS Subscription through the provider's real
//! Provider-trait apply path against an in-process winterbaume CloudControl
//! mock. winterbaume-cloudcontrol 1.0.1 does not register a shaper for
//! `AWS::SNS::Subscription`, so this test explicitly seeds the stored read
//! model with the object-shaped FilterPolicy observed from Cloud Control. The
//! provider must drop that omitted property and produce no diff against the
//! desired Subscription.

mod common;

use std::sync::Arc;

use aws_config::{BehaviorVersion, Region};
use carina_core::differ::{Diff, diff};
use carina_core::provider::{CreateOutcomeKind, Provider, RawSavedAttrs, ReadRequest};
use carina_core::resource::{ConcreteValue, ResolvedResource, Resource, Value};
use carina_core::schema::SchemaRegistry;
use carina_provider_awscc::AwsccProvider;
use carina_provider_awscc::provider::AwsccProviderConfig;
use carina_provider_awscc::schemas::generated::sns::subscription::sns_subscription_config;
use winterbaume_cloudcontrol::CloudControlService;
use winterbaume_core::{MockAws, StatefulService};

const TYPE_NAME: &str = "AWS::SNS::Subscription";
const ACCOUNT_ID: &str = "123456789012";
const REGION: &str = "ap-northeast-1";
const TOPIC_ARN: &str = "arn:aws:sns:ap-northeast-1:123456789012:carina-registry-alerts";
const ENDPOINT_ARN: &str = "arn:aws:sqs:ap-northeast-1:123456789012:carina-registry-alerts";

fn string(value: &str) -> Value {
    Value::Concrete(ConcreteValue::String(value.to_string().into()))
}

fn bool_value(value: bool) -> Value {
    Value::Concrete(ConcreteValue::Bool(value))
}

fn subscription_resource() -> Resource {
    Resource::with_provider("awscc", "sns.Subscription", "alerts", None)
        .with_attribute("topic_arn", string(TOPIC_ARN))
        .with_attribute("protocol", string("sqs"))
        .with_attribute("endpoint", string(ENDPOINT_ARN))
        .with_attribute("raw_message_delivery", bool_value(true))
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

async fn seed_filter_policy_readback(cloudcontrol: &CloudControlService) {
    let mut view = cloudcontrol.snapshot(ACCOUNT_ID, REGION).await;
    let resource = view
        .resources
        .values_mut()
        .find(|resource| resource.type_name == TYPE_NAME)
        .expect("stored subscription to seed");
    let mut model: serde_json::Value = serde_json::from_str(&resource.resource_model)
        .expect("stored subscription model should be JSON");
    model
        .as_object_mut()
        .expect("stored subscription model should be an object")
        .insert(
            "FilterPolicy".to_string(),
            serde_json::json!({"event": ["created"]}),
        );
    resource.resource_model =
        serde_json::to_string(&model).expect("seeded subscription model JSON");
    cloudcontrol
        .restore(ACCOUNT_ID, REGION, view)
        .await
        .expect("seed stored subscription read model");
}

#[test]
fn subscription_schema_omits_free_form_json_policies() {
    let config = sns_subscription_config();
    config
        .schema
        .clone()
        .validate()
        .expect("SNS Subscription schema must pass admission");

    for attribute in [
        "delivery_policy",
        "filter_policy",
        "redrive_policy",
        "replay_policy",
    ] {
        assert!(!config.schema.attributes().contains_key(attribute));
    }
}

#[tokio::test(start_paused = true)]
async fn sns_subscription_create_then_read_omits_filter_policy_without_a_diff() {
    let (provider, cloudcontrol) = winterbaume_provider().await;
    let resource = subscription_resource();
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

    seed_filter_policy_readback(&cloudcontrol).await;
    let (_, seeded_state) = stored_subscription(&cloudcontrol).await;
    assert_eq!(
        seeded_state["FilterPolicy"],
        serde_json::json!({"event": ["created"]})
    );

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
    assert_eq!(
        read.attributes().get("raw_message_delivery"),
        Some(&bool_value(true))
    );

    let current = read.into_state_for_writeback();
    let desired = ResolvedResource::new(resource);
    let saved = RawSavedAttrs::default().lift(&SchemaRegistry::new());
    let config = sns_subscription_config();
    let result = diff(&desired, &current, &saved, None, Some(&config.schema));
    assert!(
        matches!(result, Diff::NoChange(_)),
        "object-shaped FilterPolicy read-back must not produce a diff: {result:?}"
    );
}
