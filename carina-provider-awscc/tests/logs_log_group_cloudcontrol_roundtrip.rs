//! Cloud Control round-trip coverage for the typed CloudWatch Logs policy
//! documents measured with carina-test-008 in ap-northeast-1 on 2026-10-31.

mod common;

use std::collections::HashMap;
use std::sync::Arc;

use aws_config::{BehaviorVersion, Region};
use carina_core::differ::{Diff, diff};
use carina_core::provider::{CreateOutcomeKind, Provider, RawSavedAttrs, ReadRequest};
use carina_core::resource::{ConcreteValue, Resource, Value};
use carina_core::schema::SchemaRegistry;
use carina_provider_awscc::provider::AwsccProviderConfig;
use carina_provider_awscc::schemas::generated::logs::log_group::logs_log_group_config;
use carina_provider_awscc::{AwsccNormalizer, AwsccProvider};
use indexmap::IndexMap;
use winterbaume_cloudcontrol::CloudControlService;
use winterbaume_core::{MockAws, StatefulService};

const TYPE_NAME: &str = "AWS::Logs::LogGroup";
const ACCOUNT_ID: &str = "123456789012";
const REGION: &str = "ap-northeast-1";

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

fn empty_map() -> Value {
    Value::Concrete(ConcreteValue::Map(IndexMap::new()))
}

fn list(items: impl IntoIterator<Item = Value>) -> Value {
    Value::Concrete(ConcreteValue::List(items.into_iter().collect()))
}

fn measured_data_protection_policy() -> Value {
    map([
        ("name", string("carina-measured-policy")),
        ("description", string("Measured policy grammar")),
        ("version", enum_identifier("v2021_06_01")),
        (
            "configuration",
            map([(
                "custom_data_identifier",
                list([map([
                    ("name", string("EmployeeId")),
                    ("regex", string("EMP-[0-9]{6}")),
                ])]),
            )]),
        ),
        (
            "statement",
            list([
                map([
                    ("sid", string("audit-empty-destination")),
                    (
                        "data_identifier",
                        list([string(
                            "arn:aws:dataprotection::aws:data-identifier/EmailAddress",
                        )]),
                    ),
                    (
                        "operation",
                        map([("audit", map([("findings_destination", empty_map())]))]),
                    ),
                ]),
                map([
                    ("sid", string("audit-and-mask")),
                    (
                        "data_identifier",
                        list([string(
                            "arn:aws:dataprotection::aws:data-identifier/DriversLicense-US",
                        )]),
                    ),
                    (
                        "operation",
                        map([
                            (
                                "audit",
                                map([(
                                    "findings_destination",
                                    map([
                                        (
                                            "cloud_watch_logs",
                                            map([("log_group", string("/carina/audit-findings"))]),
                                        ),
                                        (
                                            "firehose",
                                            map([(
                                                "delivery_stream",
                                                string("carina-audit-findings"),
                                            )]),
                                        ),
                                        ("s3", map([("bucket", string("carina-audit-findings"))])),
                                    ]),
                                )]),
                            ),
                            ("deidentify", map([("mask_config", empty_map())])),
                        ]),
                    ),
                ]),
            ]),
        ),
    ])
}

fn policies_resource() -> Resource {
    Resource::with_provider("awscc", "logs.LogGroup", "policies", None)
        .with_attribute("log_group_name", string("/carina/test/policies"))
        .with_attribute("data_protection_policy", measured_data_protection_policy())
        .with_attribute(
            "field_index_policies",
            list([map([("fields", list([string("requestId")]))])]),
        )
}

fn no_field_indexes_resource() -> Resource {
    Resource::with_provider("awscc", "logs.LogGroup", "no_indexes", None)
        .with_attribute("log_group_name", string("/carina/test/no-indexes"))
}

fn explicit_field_indexes_resource(field_index_policies: Value) -> Resource {
    no_field_indexes_resource().with_attribute("field_index_policies", field_index_policies)
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

async fn stored_log_group(cloudcontrol: &CloudControlService) -> (String, serde_json::Value) {
    let view = cloudcontrol.snapshot(ACCOUNT_ID, REGION).await;
    let resources = view
        .resources
        .values()
        .filter(|resource| resource.type_name == TYPE_NAME)
        .collect::<Vec<_>>();
    assert_eq!(resources.len(), 1, "expected exactly one stored log group");
    let resource = resources[0];
    (
        resource.identifier.clone(),
        serde_json::from_str(&resource.resource_model)
            .expect("stored log group model should be JSON"),
    )
}

async fn seed_empty_field_index_readback(cloudcontrol: &CloudControlService) {
    let mut view = cloudcontrol.snapshot(ACCOUNT_ID, REGION).await;
    let resource = view
        .resources
        .values_mut()
        .find(|resource| resource.type_name == TYPE_NAME)
        .expect("stored log group to seed");
    let mut model: serde_json::Value = serde_json::from_str(&resource.resource_model)
        .expect("stored log group model should be JSON");
    model
        .as_object_mut()
        .expect("stored log group model should be an object")
        .insert("FieldIndexPolicies".to_string(), serde_json::json!([]));
    resource.resource_model = serde_json::to_string(&model).expect("seeded log group model JSON");
    cloudcontrol
        .restore(ACCOUNT_ID, REGION, view)
        .await
        .expect("seed stored log group read model");
}

#[tokio::test(start_paused = true)]
async fn logs_policy_documents_round_trip_as_native_json_without_a_diff() {
    let (provider, cloudcontrol) = winterbaume_provider().await;
    let resource = policies_resource();
    let id = resource.id.clone();
    let request = common::create_request(resource.clone()).await;

    let created = carina_core::executor::dispatch_create(&provider, &id, request)
        .await
        .expect("logs.LogGroup create_resource should succeed");
    let created = match created.into_kind() {
        CreateOutcomeKind::Success { state } | CreateOutcomeKind::PartialSuccess { state, .. } => {
            state
        }
    };

    assert_eq!(created.identifier().as_str(), "/carina/test/policies");
    let (stored_identifier, desired_state) = stored_log_group(&cloudcontrol).await;
    assert_eq!(
        desired_state["DataProtectionPolicy"],
        serde_json::json!({
            "Name": "carina-measured-policy",
            "Description": "Measured policy grammar",
            "Version": "2021-06-01",
            "Configuration": {
                "CustomDataIdentifier": [{"Name": "EmployeeId", "Regex": "EMP-[0-9]{6}"}]
            },
            "Statement": [
                {
                    "Sid": "audit-empty-destination",
                    "DataIdentifier": ["arn:aws:dataprotection::aws:data-identifier/EmailAddress"],
                    "Operation": {"Audit": {"FindingsDestination": {}}}
                },
                {
                    "Sid": "audit-and-mask",
                    "DataIdentifier": ["arn:aws:dataprotection::aws:data-identifier/DriversLicense-US"],
                    "Operation": {
                        "Audit": {"FindingsDestination": {
                            "CloudWatchLogs": {"LogGroup": "/carina/audit-findings"},
                            "Firehose": {"DeliveryStream": "carina-audit-findings"},
                            "S3": {"Bucket": "carina-audit-findings"}
                        }},
                        "Deidentify": {"MaskConfig": {}}
                    }
                }
            ]
        })
    );
    assert_eq!(
        desired_state["FieldIndexPolicies"],
        serde_json::json!([{"Fields": ["requestId"]}])
    );

    let read = Provider::read(
        &provider,
        &id,
        &common::applied_identifier(stored_identifier),
        ReadRequest,
    )
    .await
    .expect("logs.LogGroup read should succeed");
    let read = common::found_state(read);
    assert!(read.attributes().contains_key("data_protection_policy"));
    assert!(read.attributes().contains_key("field_index_policies"));

    let mut current_states = HashMap::from([(id.clone(), read.into_state_for_writeback())]);
    carina_core::provider::normalize_provider_states(&AwsccNormalizer, &mut current_states)
        .await
        .expect("read state normalization should succeed");
    let desired = common::normalized_resolved_resource(resource).await;
    let saved = RawSavedAttrs::default().lift(&SchemaRegistry::new());
    let config = logs_log_group_config();
    let result = diff(
        &desired,
        &current_states[&id],
        &saved,
        None,
        Some(&config.schema),
    );
    assert!(
        matches!(result, Diff::NoChange(_)),
        "measured native policy objects must round-trip without a diff: {result:?}"
    );
}

#[tokio::test(start_paused = true)]
async fn omitted_field_index_policies_and_read_back_empty_list_have_no_diff() {
    let (provider, cloudcontrol) = winterbaume_provider().await;
    let resource = no_field_indexes_resource();
    let id = resource.id.clone();
    let request = common::create_request(resource.clone()).await;
    carina_core::executor::dispatch_create(&provider, &id, request)
        .await
        .expect("logs.LogGroup create_resource should succeed");

    seed_empty_field_index_readback(&cloudcontrol).await;
    let (stored_identifier, seeded_state) = stored_log_group(&cloudcontrol).await;
    assert_eq!(seeded_state["FieldIndexPolicies"], serde_json::json!([]));

    let read = Provider::read(
        &provider,
        &id,
        &common::applied_identifier(stored_identifier),
        ReadRequest,
    )
    .await
    .expect("logs.LogGroup read should succeed");
    let read = common::found_state(read);
    assert_eq!(
        read.attributes().get("field_index_policies"),
        Some(&list([])),
        "measured Cloud Control read-back supplies an empty list"
    );

    let mut current_states = HashMap::from([(id.clone(), read.into_state_for_writeback())]);
    carina_core::provider::normalize_provider_states(&AwsccNormalizer, &mut current_states)
        .await
        .expect("read state normalization should succeed");
    assert_eq!(
        current_states[&id].attributes.get("field_index_policies"),
        Some(&list([])),
        "read normalization must preserve an explicit server value"
    );

    let desired = common::normalized_resolved_resource(resource).await;
    let saved = RawSavedAttrs::default().lift(&SchemaRegistry::new());
    let config = logs_log_group_config();
    let result = diff(
        &desired,
        &current_states[&id],
        &saved,
        None,
        Some(&config.schema),
    );
    assert!(
        matches!(result, Diff::NoChange(_)),
        "omitted FieldIndexPolicies versus measured [] must have no diff: {result:?}"
    );

    let explicit_empty =
        common::normalized_resolved_resource(explicit_field_indexes_resource(list([]))).await;
    let result = diff(
        &explicit_empty,
        &current_states[&id],
        &saved,
        None,
        Some(&config.schema),
    );
    assert!(
        matches!(result, Diff::NoChange(_)),
        "explicit FieldIndexPolicies=[] versus measured [] must have no diff: {result:?}"
    );

    let explicit_non_empty = common::normalized_resolved_resource(explicit_field_indexes_resource(
        list([map([("fields", list([string("requestId")]))])]),
    ))
    .await;
    let result = diff(
        &explicit_non_empty,
        &current_states[&id],
        &saved,
        None,
        Some(&config.schema),
    );
    assert!(
        !matches!(result, Diff::NoChange(_)),
        "explicit non-empty FieldIndexPolicies must differ from measured []"
    );
}
