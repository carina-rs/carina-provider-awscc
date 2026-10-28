//! Integration coverage for the awscc provider's real Provider-trait apply path
//! against an in-process winterbaume CloudControl mock.
//!
//! `AWS::SSM::Parameter` is not registered with winterbaume-cloudcontrol 1.0.1,
//! so the mock returns the create-time `DesiredState` verbatim. This test
//! therefore asserts structural preservation only, especially that CloudControl
//! receives `Tags` as a JSON object and the provider reads it back as the DSL
//! `Map<String, String>`. It intentionally does not assert full shaped equality.
//!
//! Real AWS reconciliation in us-east-1 on 2026-10-26 showed that create accepts
//! `Name`, `Type`, `Value`, `Description`, and object-map `Tags`; `GetResource`
//! strips the write-only `Description`, defaults `DataType` to `text`, synthesizes
//! `Arn`, and returns `Tags` as an object map. Replacing `Value` and `Tags` updates
//! in place with the same `Name`/`Arn`; an out-of-band `ssm put-parameter
//! --overwrite` is visible through `GetResource`; removing `/Tags` omits the key;
//! and `StringList` with value `a,b` follows the same behavior.

mod common;

use std::sync::Arc;

use aws_config::{BehaviorVersion, Region};
use carina_core::provider::{CreateOutcomeKind, Provider, ReadRequest};
use carina_core::resource::{ConcreteValue, Resource, Value};
use carina_provider_awscc::AwsccProvider;
use carina_provider_awscc::provider::AwsccProviderConfig;
use indexmap::IndexMap;
use winterbaume_cloudcontrol::CloudControlService;
use winterbaume_core::{MockAws, StatefulService};

const TYPE_NAME: &str = "AWS::SSM::Parameter";
const ACCOUNT_ID: &str = "123456789012";
const REGION: &str = "us-east-1";
const PARAMETER_NAME: &str = "/carina-registry/dev/publish/task_role_arn";
const PARAMETER_VALUE: &str = "arn:aws:iam::123456789012:role/carina-registry-publish-task";

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

fn ssm_parameter_resource() -> Resource {
    Resource::with_provider("awscc", "ssm.Parameter", "publish_task_role_arn", None)
        .with_attribute("name", string(PARAMETER_NAME))
        .with_attribute("type", enum_identifier("string"))
        .with_attribute("value", string(PARAMETER_VALUE))
        .with_attribute("description", string("Registry publisher task role"))
        .with_attribute(
            "tags",
            map([
                ("Environment", string("dev")),
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

async fn stored_parameter(cloudcontrol: &CloudControlService) -> (String, serde_json::Value) {
    let view = cloudcontrol.snapshot(ACCOUNT_ID, REGION).await;
    let resources = view
        .resources
        .values()
        .filter(|resource| resource.type_name == TYPE_NAME)
        .collect::<Vec<_>>();
    assert_eq!(
        resources.len(),
        1,
        "winterbaume should store exactly one SSM Parameter resource"
    );
    let resource = resources[0];
    (
        resource.identifier.clone(),
        serde_json::from_str(&resource.resource_model)
            .expect("stored resource model should be JSON"),
    )
}

#[tokio::test(start_paused = true)]
async fn ssm_parameter_create_then_read_preserves_map_tags() {
    let (provider, cloudcontrol) = winterbaume_provider().await;
    let resource = ssm_parameter_resource();
    let id = resource.id.clone();
    let request = common::create_request(resource).await;

    let created = carina_core::executor::dispatch_create(&provider, &id, request)
        .await
        .expect("ssm.Parameter create_resource should succeed");
    let created = match created.into_kind() {
        CreateOutcomeKind::Success { state } => state,
        CreateOutcomeKind::PartialSuccess { diagnostic, .. } => {
            panic!(
                "roundtrip create should be full success, got partial: {:?}",
                diagnostic
            )
        }
    };
    assert_eq!(
        created.identifier().as_str(),
        PARAMETER_NAME,
        "primary identifier should be the parameter name"
    );

    let (stored_identifier, desired_state) = stored_parameter(&cloudcontrol).await;
    assert_eq!(
        stored_identifier, PARAMETER_NAME,
        "winterbaume should use the Name primary identifier"
    );
    assert_eq!(
        desired_state.get("Tags"),
        Some(&serde_json::json!({
            "Environment": "dev",
            "Project": "carina-registry"
        })),
        "CloudControl DesiredState must carry Tags as an object map, not a Key/Value array"
    );
    assert!(desired_state["Tags"].is_object());

    let stored_identifier = common::applied_identifier(stored_identifier);
    let read = Provider::read(&provider, &id, &stored_identifier, ReadRequest)
        .await
        .expect("ssm.Parameter read through Provider::read should succeed");
    let read = common::found_state(read);

    assert_eq!(read.identifier(), &stored_identifier);
    assert_eq!(
        read.attributes().get("value"),
        Some(&string(PARAMETER_VALUE))
    );
    assert_eq!(
        read.attributes().get("tags"),
        Some(&map([
            ("Environment", string("dev")),
            ("Project", string("carina-registry")),
        ])),
        "object-map CloudControl Tags must read back as the DSL map"
    );
}
