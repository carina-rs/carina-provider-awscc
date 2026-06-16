mod common;

use aws_config::{BehaviorVersion, Region};
use carina_core::provider::{CreateOutcome, CreateRequest, Provider, ReadRequest};
use carina_core::resource::{ConcreteValue, Resource, Value};
use carina_provider_awscc::AwsccProvider;
use carina_provider_awscc::provider::AwsccProviderConfig;
use indexmap::IndexMap;
use serde_json::json;
use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use winterbaume_core::{MockAws, MockRequest, MockResponse, MockService, json_error_response};

const ROLE_NAME: &str = "carina-acc-test-role";
const ROLE_PATH: &str = "/foo/";
const PATH_QUALIFIED_IDENTIFIER: &str = "/foo/carina-acc-test-role";
const REQUEST_TOKEN: &str = "iam-role-create-token";

#[derive(Clone, Default)]
struct IamRoleIdentifierCloudControl {
    get_identifiers: Arc<Mutex<Vec<String>>>,
}

impl IamRoleIdentifierCloudControl {
    fn get_identifiers(&self) -> Vec<String> {
        self.get_identifiers.lock().unwrap().clone()
    }
}

impl MockService for IamRoleIdentifierCloudControl {
    fn service_name(&self) -> &str {
        "cloudcontrolapi"
    }

    fn url_patterns(&self) -> Vec<&str> {
        vec![
            r"https?://cloudcontrolapi\..*\.amazonaws\.com",
            r"https?://cloudcontrolapi\.amazonaws\.com",
        ]
    }

    fn handle(
        &self,
        request: MockRequest,
    ) -> Pin<Box<dyn Future<Output = MockResponse> + Send + '_>> {
        Box::pin(async move {
            let action = request
                .headers
                .get("x-amz-target")
                .and_then(|value| value.to_str().ok())
                .and_then(|value| value.split('.').next_back())
                .unwrap_or("");
            match action {
                "CreateResource" => create_resource_response(),
                "GetResourceRequestStatus" => create_status_response(),
                "GetResource" => self.get_resource_response(&request.body),
                _ => json_error_response(
                    501,
                    "NotImplementedError",
                    &format!("{action} is not implemented in test mock"),
                ),
            }
        })
    }
}

fn create_resource_response() -> MockResponse {
    MockResponse::json(
        200,
        json!({
            "ProgressEvent": {
                "TypeName": "AWS::IAM::Role",
                "Identifier": PATH_QUALIFIED_IDENTIFIER,
                "RequestToken": REQUEST_TOKEN,
                "Operation": "CREATE",
                "OperationStatus": "IN_PROGRESS"
            }
        })
        .to_string(),
    )
}

fn create_status_response() -> MockResponse {
    MockResponse::json(
        200,
        json!({
            "ProgressEvent": {
                "TypeName": "AWS::IAM::Role",
                "Identifier": PATH_QUALIFIED_IDENTIFIER,
                "RequestToken": REQUEST_TOKEN,
                "Operation": "CREATE",
                "OperationStatus": "SUCCESS"
            }
        })
        .to_string(),
    )
}

impl IamRoleIdentifierCloudControl {
    fn get_resource_response(&self, body: &[u8]) -> MockResponse {
        let input: serde_json::Value = serde_json::from_slice(body).unwrap();
        let identifier = input
            .get("Identifier")
            .and_then(|value| value.as_str())
            .unwrap_or("")
            .to_string();
        self.get_identifiers
            .lock()
            .unwrap()
            .push(identifier.clone());
        if identifier != ROLE_NAME {
            return json_error_response(
                404,
                "ResourceNotFoundException",
                &format!("AWS::IAM::Role {identifier} not found"),
            );
        }

        let properties = json!({
            "Arn": format!("arn:aws:iam::123456789012:role{ROLE_PATH}{ROLE_NAME}"),
            "AssumeRolePolicyDocument": assume_role_policy_json(),
            "Path": ROLE_PATH,
            "RoleId": "AROATESTROLEID",
            "RoleName": ROLE_NAME
        });
        MockResponse::json(
            200,
            json!({
                "TypeName": "AWS::IAM::Role",
                "ResourceDescription": {
                    "Identifier": ROLE_NAME,
                    "Properties": properties.to_string()
                }
            })
            .to_string(),
        )
    }
}

fn string(value: &str) -> Value {
    Value::Concrete(ConcreteValue::String(value.to_string()))
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

fn assume_role_policy_json() -> serde_json::Value {
    json!({
        "Version": "2012-10-17",
        "Statement": [{
            "Effect": "Allow",
            "Principal": { "Service": "ec2.amazonaws.com" },
            "Action": "sts:AssumeRole"
        }]
    })
}

fn assume_role_policy_value() -> Value {
    map([
        ("version", string("2012-10-17")),
        (
            "statement",
            list([map([
                ("effect", string("Allow")),
                ("principal", map([("Service", string("ec2.amazonaws.com"))])),
                ("action", string("sts:AssumeRole")),
            ])]),
        ),
    ])
}

fn iam_role_resource() -> Resource {
    Resource::with_provider("awscc", "iam.Role", "role", None)
        .with_attribute("role_name", string(ROLE_NAME))
        .with_attribute("path", string(ROLE_PATH))
        .with_attribute("assume_role_policy_document", assume_role_policy_value())
}

async fn provider_with_mock(service: IamRoleIdentifierCloudControl) -> AwsccProvider {
    let mock = MockAws::builder().with_service(service).build();
    let config = aws_config::defaults(BehaviorVersion::latest())
        .http_client(mock.http_client())
        .credentials_provider(mock.credentials_provider())
        .region(Region::new("us-east-1"))
        .load()
        .await;

    AwsccProvider::from_sdk_config(config, &AwsccProviderConfig::default()).await
}

#[tokio::test]
async fn iam_role_create_canonicalizes_path_qualified_progress_identifier_to_role_name() {
    let service = IamRoleIdentifierCloudControl::default();
    let provider = provider_with_mock(service.clone()).await;
    let resource = iam_role_resource();
    let id = resource.id.clone();
    let resource = common::normalize_resource(resource).await;

    let created = Provider::create(&provider, &id, CreateRequest { resource })
        .await
        .expect("iam.Role create should succeed with canonical RoleName identifier");
    let created = match created {
        CreateOutcome::Success { state } => state,
        CreateOutcome::PartialSuccess { diagnostic, .. } => {
            panic!("iam.Role create should be full success, got partial: {diagnostic:?}")
        }
    };

    assert_eq!(created.identifier.as_deref(), Some(ROLE_NAME));
    assert_eq!(service.get_identifiers(), vec![ROLE_NAME.to_string()]);

    let read = Provider::read(&provider, &id, created.identifier.as_deref(), ReadRequest)
        .await
        .expect("iam.Role read should use the stored RoleName identifier");

    assert!(read.exists);
    assert_eq!(read.identifier.as_deref(), Some(ROLE_NAME));
    assert_eq!(
        service.get_identifiers(),
        vec![ROLE_NAME.to_string(), ROLE_NAME.to_string()]
    );
}
