//! Structured AWS error construction for provider errors.

use aws_smithy_types::error::metadata::ProvideErrorMetadata;
use aws_smithy_types::retry::ProvideErrorKind;
use carina_aws_types::Transience;
use carina_aws_types::sdk::cloudcontrol::types::HandlerErrorCode;
use carina_core::provider::ProviderError;
use carina_core::provider_text::Retryability;
#[cfg(test)]
use carina_core::test_utils::provider_text::NativeWireFixture as _;
use carina_provider_protocol::OperationKind;
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
    E: std::error::Error + ProvideErrorMetadata + ProvideErrorKind + Send + Sync + 'static,
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
    E: std::error::Error + ProvideErrorMetadata + ProvideErrorKind + Send + Sync + 'static,
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
    E: std::error::Error + ProvideErrorMetadata + ProvideErrorKind + Send + Sync + 'static,
{
    let retryability = match super::AwsccProvider::read_error_transience(&err) {
        Transience::Transient => Retryability::Transient,
        Transience::Permanent => Retryability::Unclassified,
    };
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
        .with_operation(operation)
        .with_retryability(retryability);
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

/// Classify handler failures while their structured code and status message are available.
pub(crate) fn cloudcontrol_failure_retryability(
    operation: OperationKind,
    error_code: Option<&HandlerErrorCode>,
    status_message: &str,
) -> Retryability {
    if matches!(error_code, Some(HandlerErrorCode::Throttling))
        || ["Throttling", "Rate exceeded", "RequestLimitExceeded"]
            .iter()
            .any(|marker| status_message.contains(marker))
    {
        return Retryability::Transient;
    }
    if operation == OperationKind::Delete
        && (error_code.is_some_and(|code| {
            // The pinned SDK models ResourceConflict; as_str also accepts a
            // DependencyViolation code from a newer CloudControl handler.
            matches!(code.as_str(), "ResourceConflict" | "DependencyViolation")
        }) || [
            "DependencyViolation",
            "has dependent object",
            "has a dependent object",
            "resource has dependencies",
            "mapped public address",
        ]
        .iter()
        .any(|marker| status_message.contains(marker)))
    {
        return Retryability::DependencyBlocked;
    }
    Retryability::Unclassified
}

/// Terminal CloudControl operation status returned after a successful API call.
#[derive(Debug, Error)]
pub(crate) enum CloudControlOperationFailure {
    #[error("{status_message}")]
    Generic { status_message: String },
    #[error(
        "{status_message}\n  remediation: remove `archive_policy` from the configuration, run `carina apply`, then run `carina destroy`"
    )]
    SnsTopicArchivePolicy { status_message: String },
}

impl CloudControlOperationFailure {
    pub(crate) fn into_provider_error(
        self,
        operation: OperationKind,
        error_code: Option<&HandlerErrorCode>,
    ) -> ProviderError {
        let (Self::Generic { status_message } | Self::SnsTopicArchivePolicy { status_message }) =
            &self;
        let retryability = cloudcontrol_failure_retryability(operation, error_code, status_message);
        let mut error = ProviderError::api_error_caused_by("Operation failed", self)
            .with_retryability(retryability);
        if let Some(code) = error_code {
            error = error.with_code(code.as_str());
        }
        error
    }

    pub(crate) fn new(status_message: impl Into<String>) -> Self {
        Self::Generic {
            status_message: status_message.into(),
        }
    }

    pub(crate) fn for_delete(type_name: &str, status_message: impl Into<String>) -> Self {
        let status_message = status_message.into();
        // Cloud Control delete was measured with carina-test-008 in
        // ap-northeast-1 on 2026-10-31: deleting AWS::SNS::Topic with an
        // ArchivePolicy fails with this status. Do not clear the policy
        // automatically because it protects archived messages for replay.
        if type_name == "AWS::SNS::Topic"
            && status_message.contains("Cannot delete a topic with an ArchivePolicy")
        {
            Self::SnsTopicArchivePolicy { status_message }
        } else {
            Self::Generic { status_message }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use aws_smithy_runtime_api::http::{Response, StatusCode};
    use aws_smithy_types::body::SdkBody;
    use aws_smithy_types::error::metadata::ErrorMetadata;
    use carina_aws_types::sdk::cloudcontrol::error::SdkError;
    use carina_aws_types::sdk::cloudcontrol::operation::create_resource::CreateResourceError;
    use carina_aws_types::sdk::cloudcontrol::types::error::GeneralServiceException;
    use carina_core::test_utils::provider_text::BindProviderText as _;
    use carina_provider_protocol::types::ProviderErrorKind;

    #[test]
    fn typed_retryability_sdk_errors_survive_host_binding() {
        for (status, code, expected) in [
            (400, "ThrottlingException", Retryability::Transient),
            (400, "RequestTimeout", Retryability::Transient),
            (400, "SlowDown", Retryability::Transient),
            (409, "OperationAborted", Retryability::Unclassified),
            (429, "Unmodeled", Retryability::Transient),
            (500, "Unmodeled", Retryability::Transient),
            (503, "Unmodeled", Retryability::Transient),
            (403, "AccessDeniedException", Retryability::Unclassified),
            (400, "InvalidRequestException", Retryability::Unclassified),
        ] {
            let sdk_error = SdkError::service_error(
                aws_smithy_types::error::ErrorMetadata::builder()
                    .code(code)
                    .message("Throttling text alone must not decide SDK retryability")
                    .build(),
                Response::new(StatusCode::try_from(status).unwrap(), SdkBody::empty()),
            );
            let error = api_error_with_meta(
                "Failed to get resource",
                "cloudcontrol.GetResource",
                sdk_error,
            )
            .into_host_for_test();
            assert_eq!(error.retryability(), expected, "{status} {code}");
            assert_eq!(error.is_transient(), expected == Retryability::Transient);
            assert!(!error.is_delete_dependency());
        }
    }

    #[test]
    fn typed_retryability_sdk_io_retains_awscc_transport_policy() {
        use carina_plugin_sdk::wasi_http::{SendPhase, transport_connector_error_for_test};

        // The shared AWS classifier treats this code-less WASI I/O failure as
        // permanent. AWSCC's existing read policy treats unannotated I/O
        // failures as transient.
        let connector = transport_connector_error_for_test(SendPhase::NotSent);
        assert!(connector.is_io());
        let sdk_error: SdkError<aws_smithy_types::error::ErrorMetadata> =
            SdkError::dispatch_failure(connector);
        let error = api_error_with_meta(
            "Failed to get resource",
            "cloudcontrol.GetResource",
            sdk_error,
        )
        .into_host_for_test();
        assert_eq!(error.retryability(), Retryability::Transient);
    }

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
        assert_eq!(
            error
                .clone()
                .into_host_for_test()
                .detail()
                .context()
                .to_string(),
            "Failed to create resource"
        );
        assert_eq!(
            error
                .clone()
                .into_host_for_test()
                .detail()
                .operation
                .as_ref()
                .map(ToString::to_string)
                .as_deref(),
            Some("cloudcontrol.CreateResource")
        );
        assert_eq!(
            error.clone().into_host_for_test().detail().status,
            Some(500)
        );
        assert_eq!(
            error
                .clone()
                .into_host_for_test()
                .detail()
                .code()
                .map(|code| code.to_string())
                .as_deref(),
            Some("GeneralServiceException")
        );
        assert_eq!(
            error
                .clone()
                .into_host_for_test()
                .detail()
                .request_id
                .as_ref()
                .map(ToString::to_string)
                .as_deref(),
            Some("test-request-id")
        );

        let rendered = error.clone().into_host_for_test().to_string();
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
        assert_eq!(
            wire.request_id.as_ref().map(ToString::to_string).as_deref(),
            Some("test-request-id")
        );
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

    #[test]
    fn sns_topic_archive_policy_delete_failure_keeps_aws_reason_and_adds_remediation() {
        const AWS_MESSAGE: &str = "Invalid state: Cannot delete a topic with an ArchivePolicy";

        let cause = CloudControlOperationFailure::for_delete("AWS::SNS::Topic", AWS_MESSAGE);
        assert!(matches!(
            cause,
            CloudControlOperationFailure::SnsTopicArchivePolicy { .. }
        ));

        let rendered = cause.to_string();
        assert_eq!(rendered.matches(AWS_MESSAGE).count(), 1);
        assert!(rendered.contains("remove `archive_policy` from the configuration"));
        assert!(rendered.contains("run `carina apply`"));
        assert!(rendered.contains("run `carina destroy`"));

        let unrelated =
            CloudControlOperationFailure::for_delete("AWS::SNS::Subscription", AWS_MESSAGE);
        assert!(matches!(
            unrelated,
            CloudControlOperationFailure::Generic { .. }
        ));
        assert_eq!(unrelated.to_string(), AWS_MESSAGE);
    }
}
