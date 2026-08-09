//! Integration coverage for the awscc provider's real Provider-trait apply path
//! against an in-process winterbaume CloudControl mock.
//!
//! winterbaume-cloudcontrol 1.0.1 does not register a shaper for
//! AWS::ECR::Repository, so the mock returns the create-time DesiredState
//! verbatim on read. This test therefore asserts only structural preservation:
//! the nested struct fields (`image_scanning_configuration`,
//! `lifecycle_policy`) and the `tags` map survive the apply path as structured
//! `Map` values instead of being flattened or stringified. It must NOT assert
//! full shaped equality, because that would lock in a mock artifact rather
//! than real AWS behaviour. Upgrade to full shaped equality once winterbaume
//! registers a shaper for this type.
//!
//! Live AWS reconciliation (CloudControl against a real account) showed the
//! real `GetResource` differs from the verbatim mock read in exactly the ways
//! shaping would cover: it synthesizes the read-only `Arn` and
//! `RepositoryUri`, fills the schema default
//! `EncryptionConfiguration.EncryptionType = AES256`, backfills
//! `LifecyclePolicy.RegistryId` with the account id, and omits unset
//! collection properties (`ImageTagMutabilityExclusionFilters`,
//! `RepositoryPolicyText`) that the provider's desired-state normalization
//! sends as empty collections. The create-time identifier is the repository
//! name on both real AWS and the mock.

mod common;

use aws_config::{BehaviorVersion, Region};
use carina_core::provider::{CreateOutcome, CreateRequest, Provider, ReadRequest};
use carina_core::resource::{ConcreteValue, Resource, Value};
use carina_provider_awscc::AwsccProvider;
use carina_provider_awscc::provider::AwsccProviderConfig;
use indexmap::IndexMap;
use std::collections::HashMap;
use std::sync::Arc;
use winterbaume_cloudcontrol::CloudControlService;
use winterbaume_core::{MockAws, StatefulService};

const TYPE_NAME: &str = "AWS::ECR::Repository";
const ACCOUNT_ID: &str = "123456789012";
const REGION: &str = "us-east-1";
const REPOSITORY_NAME: &str = "carina-registry-publish-api";
const LIFECYCLE_POLICY_TEXT: &str = r#"{"rules":[{"rulePriority":1,"description":"keep last 10 images","selection":{"tagStatus":"any","countType":"imageCountMoreThan","countNumber":10},"action":{"type":"expire"}}]}"#;

fn string(value: &str) -> Value {
    Value::Concrete(ConcreteValue::String(value.to_string()))
}

fn bool_(value: bool) -> Value {
    Value::Concrete(ConcreteValue::Bool(value))
}

fn map(entries: impl IntoIterator<Item = (&'static str, Value)>) -> Value {
    Value::Concrete(ConcreteValue::Map(
        entries
            .into_iter()
            .map(|(key, value)| (key.to_string(), value))
            .collect::<IndexMap<_, _>>(),
    ))
}

fn ecr_repository_resource() -> Resource {
    Resource::with_provider("awscc", "ecr.Repository", "publish_api_repo", None)
        .with_attribute("repository_name", string(REPOSITORY_NAME))
        .with_attribute("image_tag_mutability", string("IMMUTABLE"))
        .with_attribute(
            "image_scanning_configuration",
            map([("scan_on_push", bool_(true))]),
        )
        .with_attribute(
            "lifecycle_policy",
            map([("lifecycle_policy_text", string(LIFECYCLE_POLICY_TEXT))]),
        )
        .with_attribute("tags", map([("Environment", string("test"))]))
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
        AwsccProvider::from_sdk_config(config, &AwsccProviderConfig::default()).await,
        cloudcontrol,
    )
}

fn attributes(entries: impl IntoIterator<Item = (&'static str, Value)>) -> HashMap<String, Value> {
    entries
        .into_iter()
        .map(|(key, value)| (key.to_string(), value))
        .collect()
}

async fn stored_repository_identifier(cloudcontrol: &CloudControlService) -> String {
    let view = cloudcontrol.snapshot(ACCOUNT_ID, REGION).await;
    let resources = view
        .resources
        .values()
        .filter(|resource| resource.type_name == TYPE_NAME)
        .collect::<Vec<_>>();
    assert_eq!(
        resources.len(),
        1,
        "winterbaume should store exactly one ECR Repository resource"
    );
    resources[0].identifier.clone()
}

#[tokio::test]
async fn ecr_repository_create_then_read_preserves_struct_and_map_fields() {
    let (provider, cloudcontrol) = winterbaume_provider().await;
    let resource = ecr_repository_resource();
    let id = resource.id.clone();
    let resource = common::normalize_resource(resource).await;

    let created = Provider::create(&provider, &id, CreateRequest { resource })
        .await
        .expect("ecr.Repository create through Provider::create should succeed");
    let created = match created {
        CreateOutcome::Success { state } => state,
        CreateOutcome::PartialSuccess { diagnostic, .. } => {
            panic!(
                "roundtrip create should be full success, got partial: {:?}",
                diagnostic
            )
        }
    };
    let identifier = created
        .identifier
        .as_deref()
        .expect("CloudControl create must return a stable identifier");

    assert_eq!(
        identifier, REPOSITORY_NAME,
        "primary identifier for AWS::ECR::Repository is the repository name"
    );
    let stored_identifier = stored_repository_identifier(&cloudcontrol).await;
    assert_ne!(stored_identifier, identifier);
    assert!(
        common::is_uuidish(&stored_identifier),
        "unregistered winterbaume identifier should be a synthetic UUID: {stored_identifier}"
    );

    let read = Provider::read(&provider, &id, Some(&stored_identifier), ReadRequest)
        .await
        .expect("ecr.Repository read through Provider::read should succeed");

    assert!(read.exists, "read-back state must exist");
    assert_eq!(read.identifier.as_deref(), Some(stored_identifier.as_str()));
    assert_eq!(
        read.attributes,
        attributes([
            ("repository_name", string(REPOSITORY_NAME)),
            ("image_tag_mutability", string("IMMUTABLE")),
            (
                "image_scanning_configuration",
                map([("scan_on_push", bool_(true))]),
            ),
            (
                "lifecycle_policy",
                map([("lifecycle_policy_text", string(LIFECYCLE_POLICY_TEXT))]),
            ),
            ("tags", map([("Environment", string("test"))])),
            // Unset collection attributes are backfilled as empty values by
            // desired-state normalization before the create request, so the
            // verbatim mock read returns them too.
            (
                "image_tag_mutability_exclusion_filters",
                Value::Concrete(ConcreteValue::List(vec![])),
            ),
            ("repository_policy_text", map([])),
        ]),
        "unregistered winterbaume type must preserve struct and map fields as structured values"
    );
}
