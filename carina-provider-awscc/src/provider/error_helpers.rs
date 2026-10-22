//! Structured AWS error construction for provider errors.

use aws_smithy_types::error::metadata::ProvideErrorMetadata;
use carina_core::provider::ProviderError;
use thiserror::Error;

type AwsSdkError<E> = aws_smithy_runtime_api::client::result::SdkError<
    E,
    aws_smithy_runtime_api::client::orchestrator::HttpResponse,
>;

#[derive(Debug)]
struct AwsServiceMessage(Option<String>);

impl AwsServiceMessage {
    fn new<E>(message: Option<String>, source: &AwsSdkError<E>) -> Self
    where
        E: std::error::Error + 'static,
    {
        Self(message.filter(|message| !error_chain_contains(source, message)))
    }
}

impl std::fmt::Display for AwsServiceMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.0 {
            Some(message) => f.write_str(message),
            None => f.write_str("AWS SDK request failed"),
        }
    }
}

fn error_chain_contains(mut error: &(dyn std::error::Error + 'static), needle: &str) -> bool {
    loop {
        if error.to_string().contains(needle) {
            return true;
        }
        match error.source() {
            Some(source) => error = source,
            None => return false,
        }
    }
}

/// AWS diagnostic attached beneath the operation-specific provider context.
///
/// The SDK error remains a source because transport and modeled service details
/// live in its source chain. The wrapper renders the AWS-side message only when
/// that chain does not already render it; `ProviderError` owns the traversal.
#[derive(Debug, Error)]
#[error("{message}")]
struct AwsServiceErrorCause<E>
where
    E: std::error::Error + 'static,
{
    message: AwsServiceMessage,
    #[source]
    source: AwsSdkError<E>,
}

/// AWS diagnostic with dynamic typed subject data below a static context.
#[derive(Debug, Error)]
#[error("{subject}: {message}")]
struct AwsServiceErrorWithSubjectCause<E, Subject>
where
    E: std::error::Error + 'static,
    Subject: std::fmt::Debug + std::fmt::Display,
{
    subject: Subject,
    message: AwsServiceMessage,
    #[source]
    source: AwsSdkError<E>,
}

pub(crate) fn api_error_with_meta<E, S>(
    context: &'static str,
    operation: S,
    err: AwsSdkError<E>,
) -> ProviderError
where
    S: Into<String>,
    E: std::error::Error + ProvideErrorMetadata + Send + Sync + 'static,
{
    api_error_with_meta_cause(context, operation, err, |message, source| {
        AwsServiceErrorCause { message, source }
    })
}

pub(crate) fn api_error_with_meta_and_subject<E, S, Subject>(
    context: &'static str,
    operation: S,
    subject: Subject,
    err: AwsSdkError<E>,
) -> ProviderError
where
    S: Into<String>,
    Subject: std::fmt::Debug + std::fmt::Display + Send + Sync + 'static,
    E: std::error::Error + ProvideErrorMetadata + Send + Sync + 'static,
{
    api_error_with_meta_cause(context, operation, err, |message, source| {
        AwsServiceErrorWithSubjectCause {
            subject,
            message,
            source,
        }
    })
}

fn api_error_with_meta_cause<E, S, Cause>(
    context: &'static str,
    operation: S,
    err: AwsSdkError<E>,
    make_cause: impl FnOnce(AwsServiceMessage, AwsSdkError<E>) -> Cause,
) -> ProviderError
where
    S: Into<String>,
    Cause: std::error::Error + Send + Sync + 'static,
    E: std::error::Error + ProvideErrorMetadata + Send + Sync + 'static,
{
    let status = err
        .raw_response()
        .map(|response| response.status().as_u16());
    let code = err.code().map(str::to_owned);
    let aws_message = AwsServiceMessage::new(err.message().map(str::to_owned), &err);
    let request_id = err.raw_response().and_then(|response| {
        response
            .headers()
            .get("x-amzn-requestid")
            .or_else(|| response.headers().get("x-amz-request-id"))
            .map(str::to_owned)
    });
    let cause = make_cause(aws_message, err);

    let mut error =
        if code.as_deref().is_some_and(is_not_found_error_code) || status == Some(404) {
            ProviderError::not_found_caused_by(context, cause)
        } else {
            ProviderError::api_error_caused_by(context, cause)
        }
        .with_operation(operation);
    if let Some(status) = status {
        error = error.with_status(status);
    }
    if let Some(code) = code {
        error = error.with_code(code);
    }
    if let Some(request_id) = request_id {
        error = error.with_request_id(request_id);
    }
    error
}

fn is_not_found_error_code(code: &str) -> bool {
    matches!(
        code,
        "ResourceNotFoundException"
            | "NoSuchEntity"
            | "NoSuchBucket"
            | "NoSuchEntityException"
            | "InvalidGroup.NotFound"
            | "InvalidGroupId.NotFound"
            | "InvalidRoute.NotFound"
            | "InvalidRouteTableID.NotFound"
            | "InvalidVpcID.NotFound"
            | "InvalidSubnetID.NotFound"
            | "InvalidInternetGatewayID.NotFound"
            | "InvalidNatGatewayID.NotFound"
            | "InvalidAllocationID.NotFound"
            | "InvalidVpcPeeringConnectionID.NotFound"
            | "InvalidVpnGatewayID.NotFound"
            | "InvalidTransitGatewayID.NotFound"
            | "InvalidPrefixListID.NotFound"
            | "InvalidIpamID.NotFound"
            | "InvalidIpamPoolId.NotFound"
            | "InvalidVpcEndpointId.NotFound"
            | "InvalidNetworkInterfaceID.NotFound"
            | "InvalidGatewayID.NotFound"
            | "InvalidFlowLogId.NotFound"
            | "ResourceNotFoundFault"
            | "ResourceNotFoundError"
    )
}

/// Terminal CloudControl operation status returned after a successful API call.
#[derive(Debug, Error)]
#[error("{status_message}")]
pub(crate) struct CloudControlOperationFailure {
    status_message: String,
}

impl CloudControlOperationFailure {
    pub(crate) fn new(status_message: impl Into<String>) -> Self {
        Self {
            status_message: status_message.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use aws_sdk_cloudcontrol::error::SdkError;
    use aws_sdk_cloudcontrol::operation::create_resource::CreateResourceError;
    use aws_sdk_cloudcontrol::types::error::GeneralServiceException;
    use aws_smithy_runtime_api::http::{Response, StatusCode};
    use aws_smithy_types::body::SdkBody;
    use aws_smithy_types::error::metadata::ErrorMetadata;
    use carina_provider_protocol::types::ProviderErrorKind;

    #[test]
    fn cloudcontrol_error_renders_reason_once_and_preserves_wire_metadata() {
        const AWS_MESSAGE: &str = "CloudControl handler returned status FAILED";

        let inner = CreateResourceError::GeneralServiceException(
            GeneralServiceException::builder()
                .meta(
                    ErrorMetadata::builder()
                        .code("GeneralServiceException")
                        .message(AWS_MESSAGE)
                        .build(),
                )
                .build(),
        );
        let mut raw = Response::new(
            StatusCode::try_from(500).expect("valid HTTP status"),
            SdkBody::empty(),
        );
        raw.headers_mut()
            .insert("x-amzn-requestid", "test-request-id");
        let sdk_error = SdkError::service_error(inner, raw);

        let error = api_error_with_meta(
            "Failed to create resource",
            "cloudcontrol.CreateResource",
            sdk_error,
        );

        assert!(matches!(&error, ProviderError::ApiError(_)));
        assert_eq!(error.detail().context(), "Failed to create resource");
        assert_eq!(
            error.detail().operation.as_deref(),
            Some("cloudcontrol.CreateResource")
        );
        assert_eq!(error.detail().status, Some(500));
        assert_eq!(
            error.detail().code.as_deref(),
            Some("GeneralServiceException")
        );
        assert_eq!(
            error.detail().request_id.as_deref(),
            Some("test-request-id")
        );

        let rendered = error.to_string();
        assert_eq!(
            rendered.matches(AWS_MESSAGE).count(),
            1,
            "display must render the AWS reason exactly once: {rendered}"
        );

        let wire = error.to_wire();
        assert_eq!(wire.kind, ProviderErrorKind::ApiError);
        assert_eq!(wire.message, "Failed to create resource");
        assert_eq!(wire.status, Some(500));
        assert_eq!(wire.code.as_deref(), Some("GeneralServiceException"));
        assert_eq!(wire.request_id.as_deref(), Some("test-request-id"));
        let wire_rendered = format!(
            "{}: {}",
            wire.message,
            wire.cause.as_deref().unwrap_or_default()
        );
        assert_eq!(
            wire_rendered.matches(AWS_MESSAGE).count(),
            1,
            "wire conversion must render the AWS reason exactly once: {wire_rendered}"
        );
    }
}
