//! Integration coverage for the awscc provider's real Provider-trait apply path
//! against an in-process winterbaume CloudControl mock.
//!
//! winterbaume-cloudcontrol 1.0.1 does not register a shaper for
//! AWS::WAFv2::WebACLAssociation. Its unregistered fallback stores the model
//! under a synthetic UUID because the desired state has no generic
//! `Id`/`Name`/`Arn` field, then returns the create-time DesiredState verbatim.
//!
//! Live AWS reconciliation showed the same read shape for this type:
//! `GetResource` returned only `ResourceArn` and `WebACLArn`, with no read-only
//! synthesis or defaults. The mock's verbatim fallback is therefore faithful for
//! this resource; there is no later shaped-equality upgrade needed.
//!
//! Live AWS also returned the CloudControl identifier as
//! `<ResourceArn>|<WebACLArn>`. The create-side assertion pins that separator
//! and segment order; it does not prove mock storage preservation because
//! `merge_desired_attributes` backfills missing authored attributes before
//! canonicalization. The standalone read by the mock's UUID key exercises stored
//! state preservation and provider-name mapping.

mod common;

use aws_config::{BehaviorVersion, Region};
use carina_core::provider::{CreateOutcome, CreateRequest, Provider, ReadRequest};
use carina_core::resource::{ConcreteValue, Resource, Value};
use carina_provider_awscc::AwsccProvider;
use carina_provider_awscc::provider::AwsccProviderConfig;
use std::collections::HashMap;
use std::sync::Arc;
use winterbaume_cloudcontrol::CloudControlService;
use winterbaume_core::{MockAws, StatefulService};

const TYPE_NAME: &str = "AWS::WAFv2::WebACLAssociation";
const ACCOUNT_ID: &str = "123456789012";
const REGION: &str = "us-east-1";
const RESOURCE_ARN: &str =
    "arn:aws:elasticloadbalancing:ap-northeast-1:123456789012:loadbalancer/app/publish-api/abc123";
const WEB_ACL_ARN: &str =
    "arn:aws:wafv2:ap-northeast-1:123456789012:regional/webacl/publish-api/def456";

fn string(value: &str) -> Value {
    Value::Concrete(ConcreteValue::String(value.to_string()))
}

fn web_acl_association_resource() -> Resource {
    Resource::with_provider("awscc", "wafv2.WebAclAssociation", "publish_api_waf", None)
        .with_attribute("resource_arn", string(RESOURCE_ARN))
        .with_attribute("web_acl_arn", string(WEB_ACL_ARN))
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

async fn stored_web_acl_association_identifier(cloudcontrol: &CloudControlService) -> String {
    let view = cloudcontrol.snapshot(ACCOUNT_ID, REGION).await;
    let resources = view
        .resources
        .values()
        .filter(|resource| resource.type_name == TYPE_NAME)
        .collect::<Vec<_>>();
    assert_eq!(
        resources.len(),
        1,
        "winterbaume should store exactly one WebACLAssociation resource"
    );
    resources[0].identifier.clone()
}

#[tokio::test]
async fn web_acl_association_create_then_read_preserves_plain_string_properties() {
    let (provider, cloudcontrol) = winterbaume_provider().await;
    let resource = web_acl_association_resource();
    let id = resource.id.clone();
    let resource = common::normalize_resource(resource).await;

    let created = Provider::create(&provider, &id, CreateRequest { resource })
        .await
        .expect("wafv2.WebAclAssociation create through Provider::create should succeed");
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
    let expected_identifier = format!("{RESOURCE_ARN}|{WEB_ACL_ARN}");

    assert_eq!(identifier, expected_identifier);
    let stored_identifier = stored_web_acl_association_identifier(&cloudcontrol).await;
    assert_ne!(stored_identifier, expected_identifier);
    assert!(
        common::is_uuidish(&stored_identifier),
        "unregistered winterbaume identifier should be a synthetic UUID: {stored_identifier}"
    );

    let read = Provider::read(&provider, &id, Some(&stored_identifier), ReadRequest)
        .await
        .expect("wafv2.WebAclAssociation read through Provider::read should succeed");

    assert!(read.exists, "read-back state must exist");
    assert_eq!(read.identifier.as_deref(), Some(stored_identifier.as_str()));
    assert_eq!(
        read.attributes,
        attributes([
            ("resource_arn", string(RESOURCE_ARN)),
            ("web_acl_arn", string(WEB_ACL_ARN)),
        ]),
        "unregistered winterbaume type must preserve both composite identifier attributes"
    );
}
