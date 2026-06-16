use aws_config::{BehaviorVersion, Region};
use carina_core::provider::{Provider, ReadRequest};
use carina_core::resource::{ConcreteValue, ResourceId, Value};
use carina_provider_awscc::AwsccProvider;
use carina_provider_awscc::provider::AwsccProviderConfig;
use serde_json::json;
use std::future::Future;
use std::pin::Pin;
use winterbaume_core::{MockAws, MockRequest, MockResponse, MockService, json_error_response};

const VPC_ENDPOINT_ID: &str = "vpce-0123456789abcdef0";

#[derive(Clone)]
struct VpcEndpointPolicyCloudControl {
    policy_document: serde_json::Value,
}

impl MockService for VpcEndpointPolicyCloudControl {
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
                "GetResource" => get_resource_response(&request.body, &self.policy_document),
                _ => json_error_response(
                    501,
                    "NotImplementedError",
                    &format!("{action} is not implemented in test mock"),
                ),
            }
        })
    }
}

fn get_resource_response(body: &[u8], policy_document: &serde_json::Value) -> MockResponse {
    let input: serde_json::Value = serde_json::from_slice(body).unwrap();
    let identifier = input
        .get("Identifier")
        .and_then(|value| value.as_str())
        .unwrap_or("");
    if identifier != VPC_ENDPOINT_ID {
        return json_error_response(
            404,
            "ResourceNotFoundException",
            &format!("AWS::EC2::VPCEndpoint {identifier} not found"),
        );
    }

    let properties = json!({
        "Id": VPC_ENDPOINT_ID,
        "VpcId": "vpc-0123456789abcdef0",
        "ServiceName": "com.amazonaws.us-east-1.s3",
        "VpcEndpointType": "Gateway",
        "PolicyDocument": policy_document
    });
    let properties = properties.to_string();

    MockResponse::json(
        200,
        json!({
            "TypeName": "AWS::EC2::VPCEndpoint",
            "ResourceDescription": {
                "Identifier": VPC_ENDPOINT_ID,
                "Properties": properties
            }
        })
        .to_string(),
    )
}

async fn provider_with_mock(policy_document: serde_json::Value) -> AwsccProvider {
    let mock = MockAws::builder()
        .with_service(VpcEndpointPolicyCloudControl { policy_document })
        .build();
    let config = aws_config::defaults(BehaviorVersion::latest())
        .http_client(mock.http_client())
        .credentials_provider(mock.credentials_provider())
        .region(Region::new("us-east-1"))
        .load()
        .await;

    AwsccProvider::from_sdk_config(config, &AwsccProviderConfig::default()).await
}

async fn read_vpc_endpoint(
    policy_document: serde_json::Value,
) -> indexmap::IndexMap<String, Value> {
    let provider = provider_with_mock(policy_document).await;
    let id = ResourceId::with_provider("awscc", "ec2.VpcEndpoint", "gateway", None);

    let read = Provider::read(&provider, &id, Some(VPC_ENDPOINT_ID), ReadRequest)
        .await
        .expect("ec2.VpcEndpoint read through Provider::read should succeed");

    assert!(read.exists, "read-back state must exist");
    assert_eq!(read.identifier.as_deref(), Some(VPC_ENDPOINT_ID));

    first_policy_statement(&read.attributes).clone()
}

fn first_policy_statement(
    attributes: &std::collections::HashMap<String, Value>,
) -> &indexmap::IndexMap<String, Value> {
    let Some(Value::Concrete(ConcreteValue::Map(policy_document))) =
        attributes.get("policy_document")
    else {
        panic!("policy_document must be a Map: {attributes:?}");
    };
    let Some(Value::Concrete(ConcreteValue::List(statements))) = policy_document.get("statement")
    else {
        panic!("policy_document.statement must be a List: {policy_document:?}");
    };
    let Some(Value::Concrete(ConcreteValue::Map(statement))) = statements.first() else {
        panic!("policy_document.statement[0] must be a Map: {statements:?}");
    };
    statement
}

#[tokio::test]
async fn ec2_vpc_endpoint_read_canonicalizes_scalar_policy_action_and_resource() {
    let statement = read_vpc_endpoint(json!({
        "Version": "2012-10-17",
        "Statement": [{
            "Action": "s3:GetObject",
            "Resource": "arn:aws:s3:::*",
            "Effect": "Allow",
            "Principal": "*"
        }]
    }))
    .await;

    assert_eq!(
        statement.get("action"),
        Some(&Value::Concrete(ConcreteValue::StringList(vec![
            "s3:GetObject".to_string()
        ]))),
        "policy_document.statement[0].action must be canonicalized"
    );
    assert_eq!(
        statement.get("resource"),
        Some(&Value::Concrete(ConcreteValue::StringList(vec![
            "arn:aws:s3:::*".to_string()
        ]))),
        "policy_document.statement[0].resource must be canonicalized"
    );
}

#[tokio::test]
async fn ec2_vpc_endpoint_read_canonicalizes_list_policy_action_and_resource() {
    let statement = read_vpc_endpoint(json!({
        "Version": "2012-10-17",
        "Statement": [{
            "Action": ["s3:GetObject", "s3:PutObject"],
            "Resource": ["arn:aws:s3:::*", "arn:aws:s3:::example/*"],
            "Effect": "Allow",
            "Principal": "*"
        }]
    }))
    .await;

    assert_eq!(
        statement.get("action"),
        Some(&Value::Concrete(ConcreteValue::StringList(vec![
            "s3:GetObject".to_string(),
            "s3:PutObject".to_string()
        ]))),
        "policy_document.statement[0].action must be canonicalized"
    );
    assert_eq!(
        statement.get("resource"),
        Some(&Value::Concrete(ConcreteValue::StringList(vec![
            "arn:aws:s3:::*".to_string(),
            "arn:aws:s3:::example/*".to_string()
        ]))),
        "policy_document.statement[0].resource must be canonicalized"
    );
}
