//! Cloud Control API interaction layer.
//!
//! This module contains low-level methods for interacting with the AWS Cloud Control
//! API: creating, reading, updating, and deleting resources, as well as polling for
//! operation completion and retry/error handling logic.

use std::time::Duration;

use aws_sdk_cloudcontrol::types::{OperationStatus, ProgressEvent};
use aws_smithy_runtime_api::client::result::SdkError;
use aws_smithy_types::error::metadata::ProvideErrorMetadata;
use carina_core::provider::{ProviderError, ProviderResult};
use thiserror::Error;

use super::{
    AwsccProvider, CREATE_RETRY_INITIAL_DELAY_SECS, CREATE_RETRY_MAX_ATTEMPTS,
    CREATE_RETRY_MAX_DELAY_SECS, DELETE_RETRY_INITIAL_DELAY_SECS, DELETE_RETRY_MAX_ATTEMPTS,
    DELETE_RETRY_MAX_DELAY_SECS,
    error_helpers::{CloudControlOperationFailure, api_error_with_meta_and_subject},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WaitOutcome {
    Success {
        identifier: String,
    },
    PartialOrFailed {
        identifier: String,
        status_message: String,
    },
}

#[derive(Debug)]
struct CloudControlResourceSubject {
    type_name: String,
    identifier: Option<String>,
}

impl CloudControlResourceSubject {
    fn type_only(type_name: &str) -> Self {
        Self {
            type_name: type_name.to_string(),
            identifier: None,
        }
    }

    fn identified(type_name: &str, identifier: &str) -> Self {
        Self {
            type_name: type_name.to_string(),
            identifier: Some(identifier.to_string()),
        }
    }
}

impl std::fmt::Display for CloudControlResourceSubject {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "resource type {}", self.type_name)?;
        if let Some(identifier) = &self.identifier {
            write!(f, ", identifier {identifier}")?;
        }
        Ok(())
    }
}

#[derive(Debug)]
struct CloudControlRequestToken(String);

impl std::fmt::Display for CloudControlRequestToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "request token {}", self.0)
    }
}

#[derive(Debug, Error)]
#[error("resource type {type_name} after {attempts} retry attempts")]
struct RetryAttemptsExhausted {
    type_name: String,
    attempts: u32,
}

impl AwsccProvider {
    /// Get a resource by identifier using Cloud Control API
    pub async fn cc_get_resource(
        &self,
        type_name: &str,
        identifier: &str,
    ) -> ProviderResult<Option<serde_json::Value>> {
        let result = self
            .cloudcontrol_client
            .get_resource()
            .type_name(type_name)
            .identifier(identifier)
            .send()
            .await;

        match result {
            Ok(response) => {
                if let Some(desc) = response.resource_description()
                    && let Some(props_str) = desc.properties()
                {
                    let props = super::parse_resource_properties(props_str)?;
                    Ok(Some(props))
                } else {
                    Ok(None)
                }
            }
            Err(e) => {
                if Self::is_not_found_error(&e) {
                    Ok(None)
                } else {
                    Err(api_error_with_meta_and_subject(
                        "Failed to get resource",
                        "cloudcontrol.GetResource",
                        CloudControlResourceSubject::identified(type_name, identifier),
                        e,
                    ))
                }
            }
        }
    }

    /// Create a resource using Cloud Control API, with retry logic for retryable errors.
    ///
    /// Some operations fail transiently due to eventual consistency in AWS
    /// (e.g., IPAM Pool CIDR propagation delays cause "missing a source resource"
    /// errors when creating subnets). This method retries with exponential backoff
    /// for such errors.
    pub async fn cc_create_resource(
        &self,
        type_name: &str,
        desired_state: serde_json::Value,
        operation_config: Option<&carina_core::schema::OperationConfig>,
    ) -> ProviderResult<WaitOutcome> {
        let mut delay_secs = CREATE_RETRY_INITIAL_DELAY_SECS;
        let max_retry_attempts = operation_config
            .and_then(|c| c.create_max_retries)
            .unwrap_or(CREATE_RETRY_MAX_ATTEMPTS);
        let max_polling_attempts = operation_config
            .and_then(|c| c.create_timeout_secs)
            .map(|secs| (secs / 5) as u32)
            .unwrap_or(Self::default_polling_attempts(type_name, "create"));

        for attempt in 0..=max_retry_attempts {
            let result = self
                .cloudcontrol_client
                .create_resource()
                .type_name(type_name)
                .desired_state(desired_state.to_string())
                .send()
                .await;

            match result {
                Ok(response) => {
                    let request_token =
                        response
                            .progress_event()
                            .and_then(|p| p.request_token())
                            .ok_or_else(|| ProviderError::internal("No request token returned"))?;

                    match self
                        .wait_for_operation_with_attempts(request_token, max_polling_attempts)
                        .await
                    {
                        Ok(outcome) => return Ok(outcome),
                        Err(e)
                            if Self::is_retryable_operation_error(&e)
                                && attempt < max_retry_attempts =>
                        {
                            log::warn!(
                                "Retryable error creating {} (attempt {}/{}): {}. Retrying in {}s...",
                                type_name,
                                attempt + 1,
                                max_retry_attempts,
                                e,
                                delay_secs,
                            );
                            tokio::time::sleep(Duration::from_secs(delay_secs)).await;
                            delay_secs = (delay_secs * 2).min(CREATE_RETRY_MAX_DELAY_SECS);
                            continue;
                        }
                        Err(e) => return Err(e),
                    }
                }
                Err(e) => {
                    if Self::is_retryable_sdk_error(&e) && attempt < max_retry_attempts {
                        log::warn!(
                            "Retryable error creating {} (attempt {}/{}): {:?}. Retrying in {}s...",
                            type_name,
                            attempt + 1,
                            max_retry_attempts,
                            e,
                            delay_secs,
                        );
                        tokio::time::sleep(Duration::from_secs(delay_secs)).await;
                        delay_secs = (delay_secs * 2).min(CREATE_RETRY_MAX_DELAY_SECS);
                        continue;
                    }
                    return Err(api_error_with_meta_and_subject(
                        "Failed to create resource",
                        "cloudcontrol.CreateResource",
                        CloudControlResourceSubject::type_only(type_name),
                        e,
                    ));
                }
            }
        }

        Err(ProviderError::api_error_caused_by(
            "Failed to create resource",
            RetryAttemptsExhausted {
                type_name: type_name.to_string(),
                attempts: max_retry_attempts,
            },
        ))
    }

    /// Update a resource using Cloud Control API
    pub async fn cc_update_resource(
        &self,
        type_name: &str,
        identifier: &str,
        patch_ops: Vec<serde_json::Value>,
    ) -> ProviderResult<WaitOutcome> {
        if patch_ops.is_empty() {
            return Ok(WaitOutcome::Success {
                identifier: identifier.to_string(),
            });
        }

        let patch_document = serde_json::to_string(&patch_ops)
            .map_err(|e| ProviderError::internal_caused_by("Failed to build patch", e))?;

        let result = self
            .cloudcontrol_client
            .update_resource()
            .type_name(type_name)
            .identifier(identifier)
            .patch_document(patch_document)
            .send()
            .await
            .map_err(|e| {
                api_error_with_meta_and_subject(
                    "Failed to update resource",
                    "cloudcontrol.UpdateResource",
                    CloudControlResourceSubject::identified(type_name, identifier),
                    e,
                )
            })?;

        if let Some(request_token) = result.progress_event().and_then(|p| p.request_token()) {
            return self
                .wait_for_operation_with_attempts(request_token, 120)
                .await;
        }

        Ok(WaitOutcome::Success {
            identifier: identifier.to_string(),
        })
    }

    /// Delete a resource using Cloud Control API, with retry logic for retryable errors.
    ///
    /// Uses resource-type-specific polling timeouts. IPAM-related resources
    /// get a longer timeout since their deletion via CloudControl API can
    /// take 15-30 minutes. Retries with exponential backoff on transient errors
    /// such as throttling or service unavailability.
    pub async fn cc_delete_resource(
        &self,
        type_name: &str,
        identifier: &str,
        operation_config: Option<&carina_core::schema::OperationConfig>,
    ) -> ProviderResult<()> {
        let mut delay_secs = DELETE_RETRY_INITIAL_DELAY_SECS;
        let max_polling_attempts = operation_config
            .and_then(|c| c.delete_timeout_secs)
            .map(|secs| (secs / 5) as u32)
            .unwrap_or(Self::default_polling_attempts(type_name, "delete"));
        let max_retry_attempts = operation_config
            .and_then(|c| c.delete_max_retries)
            .unwrap_or(DELETE_RETRY_MAX_ATTEMPTS);

        for attempt in 0..=max_retry_attempts {
            let result = self
                .cloudcontrol_client
                .delete_resource()
                .type_name(type_name)
                .identifier(identifier)
                .send()
                .await;

            match result {
                Ok(response) => {
                    if let Some(request_token) =
                        response.progress_event().and_then(|p| p.request_token())
                    {
                        match self
                            .wait_for_operation_with_attempts(request_token, max_polling_attempts)
                            .await
                        {
                            Ok(WaitOutcome::Success { .. }) => return Ok(()),
                            Ok(WaitOutcome::PartialOrFailed { status_message, .. }) => {
                                return Err(ProviderError::api_error_caused_by(
                                    "Operation failed",
                                    CloudControlOperationFailure::new(status_message),
                                ));
                            }
                            Err(e)
                                if Self::is_retryable_operation_error(&e)
                                    && attempt < max_retry_attempts =>
                            {
                                log::warn!(
                                    "Retryable error deleting {} (attempt {}/{}): {}. Retrying in {}s...",
                                    type_name,
                                    attempt + 1,
                                    max_retry_attempts,
                                    e,
                                    delay_secs,
                                );
                                tokio::time::sleep(Duration::from_secs(delay_secs)).await;
                                delay_secs = (delay_secs * 2).min(DELETE_RETRY_MAX_DELAY_SECS);
                                continue;
                            }
                            Err(e) => return Err(e),
                        }
                    }
                    return Ok(());
                }
                Err(e) => {
                    if Self::is_retryable_sdk_error(&e) && attempt < max_retry_attempts {
                        log::warn!(
                            "Retryable error deleting {} (attempt {}/{}): {:?}. Retrying in {}s...",
                            type_name,
                            attempt + 1,
                            max_retry_attempts,
                            e,
                            delay_secs,
                        );
                        tokio::time::sleep(Duration::from_secs(delay_secs)).await;
                        delay_secs = (delay_secs * 2).min(DELETE_RETRY_MAX_DELAY_SECS);
                        continue;
                    }
                    return Err(api_error_with_meta_and_subject(
                        "Failed to delete resource",
                        "cloudcontrol.DeleteResource",
                        CloudControlResourceSubject::identified(type_name, identifier),
                        e,
                    ));
                }
            }
        }

        Err(ProviderError::api_error_caused_by(
            "Failed to delete resource",
            RetryAttemptsExhausted {
                type_name: type_name.to_string(),
                attempts: max_retry_attempts,
            },
        ))
    }

    /// Returns the max polling attempts for a given resource type and operation.
    ///
    /// Some resource types (e.g., IPAM Pool, VPCGatewayAttachment, NatGateway) take
    /// significantly longer to delete via the CloudControl API than the default timeout allows.
    pub(crate) fn default_polling_attempts(type_name: &str, operation: &str) -> u32 {
        if operation == "delete" {
            // IPAM Pool deletions can take 15-30 minutes via CloudControl API
            if type_name.contains("IPAMPool") || type_name.contains("IPAM") {
                return 360; // 30 minutes (360 * 5s)
            }
            // VPCGatewayAttachment deletion via CloudControl can be slow when
            // dependent resources (e.g., NAT gateways) are still being cleaned up.
            // The default 10-minute timeout is often insufficient. (issue #1066)
            if type_name.contains("VPCGatewayAttachment") {
                return 360; // 30 minutes (360 * 5s)
            }
            // NatGateway deletion via CloudControl API can take 10-15 minutes.
            // The default 10-minute timeout is insufficient. (issue #1443)
            if type_name == "AWS::EC2::NatGateway" {
                return 240; // 20 minutes (240 * 5s)
            }
            // TransitGateway deletion can take 10-20 minutes, especially during
            // create_before_destroy when VPC attachments are still being detached.
            if type_name.contains("TransitGateway") {
                return 360; // 30 minutes (360 * 5s)
            }
        }
        120 // Default: 10 minutes (120 * 5s)
    }

    /// Returns true if the SDK error represents a "not found" condition.
    ///
    /// Uses structured error metadata (`ProvideErrorMetadata::code()`) instead of
    /// fragile string matching against Debug-formatted output.
    ///
    /// Not-found error codes:
    /// - `ResourceNotFoundException`: The resource does not exist
    /// - `HandlerNotFoundException`: The resource handler was not found
    pub(crate) fn is_not_found_error<E, R>(error: &SdkError<E, R>) -> bool
    where
        E: ProvideErrorMetadata,
    {
        const NOT_FOUND_ERROR_CODES: &[&str] =
            &["ResourceNotFoundException", "HandlerNotFoundException"];

        match error {
            SdkError::ServiceError(service_error) => {
                let err = service_error.err();
                if let Some(code) = err.code() {
                    NOT_FOUND_ERROR_CODES.contains(&code)
                } else {
                    false
                }
            }
            _ => false,
        }
    }

    /// Returns true if an AWS SDK error represents a retryable condition.
    ///
    /// Uses structured error types from the AWS SDK rather than string matching.
    /// This detects retryable conditions based on the error variant or error code,
    /// which are part of the AWS API contract and more stable than error messages.
    ///
    /// Retryable error types:
    /// - `ThrottlingException`: Request rate exceeded (covers "Throttling", "Rate exceeded")
    /// - `ServiceInternalErrorException`: AWS internal server error
    /// - `HandlerFailureException`: Resource handler failed (may be transient)
    /// - `HandlerInternalFailureException`: Internal handler error
    /// - `NetworkFailureException`: Network connectivity issues
    /// - `ConcurrentOperationException`: Another operation is in progress
    /// - `NotStabilizedException`: Resource not yet stabilized
    /// - `SdkError::TimeoutError`: Connection timeout
    /// - `SdkError::DispatchFailure`: HTTP dispatch failure
    pub(crate) fn is_retryable_sdk_error<E, R>(error: &SdkError<E, R>) -> bool
    where
        E: ProvideErrorMetadata,
    {
        /// Error codes from the CloudControl API that indicate retryable conditions.
        const RETRYABLE_ERROR_CODES: &[&str] = &[
            "ThrottlingException",
            "ServiceInternalErrorException",
            "HandlerFailureException",
            "HandlerInternalFailureException",
            "NetworkFailureException",
            "ConcurrentOperationException",
            "NotStabilizedException",
            "ClientTokenConflictException",
        ];

        /// Error message patterns that indicate retryable conditions regardless of error code.
        /// These are checked against the error message when the error code alone is not
        /// sufficient to determine retryability.
        const RETRYABLE_MESSAGE_PATTERNS: &[&str] = &["non-deleted VPC Attachments"];

        match error {
            SdkError::TimeoutError(_) | SdkError::DispatchFailure(_) => true,
            SdkError::ServiceError(service_error) => {
                let err = service_error.err();
                if err
                    .code()
                    .is_some_and(|code| RETRYABLE_ERROR_CODES.contains(&code))
                {
                    return true;
                }
                if let Some(message) = err.message() {
                    RETRYABLE_MESSAGE_PATTERNS
                        .iter()
                        .any(|pattern| message.contains(pattern))
                } else {
                    false
                }
            }
            _ => false,
        }
    }

    /// Returns true if a CloudControl operation status message indicates a retryable condition.
    ///
    /// When a CloudControl operation (create/delete) succeeds at the API level but the
    /// async operation fails, the error details come as a plain-text status message from
    /// `progress_event.status_message()`. These messages don't have structured error codes,
    /// so string pattern matching is the only option.
    ///
    /// **Fragility note**: These patterns depend on AWS error message wording. If AWS
    /// changes the message format, retries may silently stop working. The patterns below
    /// are based on observed CloudControl API behavior as of 2025:
    ///
    /// - `"missing a source resource"`: IPAM Pool CIDR propagation delay causes subnet
    ///   creation to fail transiently while the pool is still provisioning.
    /// - `"Throttling"` / `"Rate exceeded"` / `"RequestLimitExceeded"`: Downstream service
    ///   throttling reported through CloudControl operation status.
    /// - `"ServiceUnavailable"` / `"InternalError"`: Transient downstream service errors
    ///   reported through CloudControl operation status.
    /// - `"non-deleted VPC Attachments"`: Transit Gateway deletion fails while VPC
    ///   attachments are still being detached asynchronously (e.g., during
    ///   `create_before_destroy` replacement).
    pub(crate) fn is_retryable_status_message(status_message: &str) -> bool {
        const RETRYABLE_STATUS_PATTERNS: &[&str] = &[
            "missing a source resource",
            "non-deleted VPC Attachments",
            "Throttling",
            "Rate exceeded",
            "RequestLimitExceeded",
            "ServiceUnavailable",
            "InternalError",
        ];
        RETRYABLE_STATUS_PATTERNS
            .iter()
            .any(|pattern| status_message.contains(pattern))
    }

    /// Classify a failed asynchronous CloudControl operation using the typed
    /// status cause created by [`Self::classify_progress_event`].
    fn is_retryable_operation_error(error: &ProviderError) -> bool {
        error
            .detail()
            .cause()
            .and_then(|cause| cause.downcast_ref::<CloudControlOperationFailure>())
            .is_some_and(|cause| Self::is_retryable_status_message(cause.status_message()))
    }

    /// Wait for a Cloud Control operation to complete with a configurable number of attempts
    pub(crate) async fn wait_for_operation_with_attempts(
        &self,
        request_token: &str,
        max_attempts: u32,
    ) -> ProviderResult<WaitOutcome> {
        let delay = Duration::from_secs(5);

        for _ in 0..max_attempts {
            let status = self
                .cloudcontrol_client
                .get_resource_request_status()
                .request_token(request_token)
                .send()
                .await
                .map_err(|e| {
                    api_error_with_meta_and_subject(
                        "Failed to get operation status",
                        "cloudcontrol.GetResourceRequestStatus",
                        CloudControlRequestToken(request_token.to_string()),
                        e,
                    )
                })?;

            if let Some(progress) = status.progress_event()
                && let Some(outcome) = Self::classify_progress_event(progress)?
            {
                return Ok(outcome);
            }
            tokio::time::sleep(delay).await;
        }

        Err(ProviderError::timeout("Operation timed out"))
    }

    pub(crate) fn classify_progress_event(
        progress: &ProgressEvent,
    ) -> ProviderResult<Option<WaitOutcome>> {
        match progress.operation_status() {
            Some(OperationStatus::Success) => Ok(Some(WaitOutcome::Success {
                identifier: progress.identifier().unwrap_or("").to_string(),
            })),
            Some(OperationStatus::Failed) | Some(OperationStatus::CancelComplete) => {
                let msg = progress
                    .status_message()
                    .unwrap_or("Unknown error")
                    .to_string();
                if let Some(identifier) = progress.identifier()
                    && !identifier.is_empty()
                {
                    return Ok(Some(WaitOutcome::PartialOrFailed {
                        identifier: identifier.to_string(),
                        status_message: msg,
                    }));
                }
                Err(ProviderError::api_error_caused_by(
                    "Operation failed",
                    CloudControlOperationFailure::new(msg),
                ))
            }
            _ => Ok(None),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // =========================================================================
    // is_retryable_status_message tests
    // =========================================================================

    #[test]
    fn test_is_retryable_status_message_ipam_source_resource() {
        assert!(AwsccProvider::is_retryable_status_message(
            "Operation failed: IpamPool 'ipam-pool-xxx' is missing a source resource"
        ));
    }

    #[test]
    fn test_is_retryable_status_message_throttling() {
        assert!(AwsccProvider::is_retryable_status_message(
            "Throttling: Rate exceeded"
        ));
    }

    #[test]
    fn test_is_retryable_status_message_request_limit() {
        assert!(AwsccProvider::is_retryable_status_message(
            "RequestLimitExceeded: too many requests"
        ));
    }

    #[test]
    fn test_is_retryable_status_message_service_unavailable() {
        assert!(AwsccProvider::is_retryable_status_message(
            "ServiceUnavailable: try again later"
        ));
    }

    #[test]
    fn test_is_retryable_status_message_internal_error() {
        assert!(AwsccProvider::is_retryable_status_message(
            "InternalError: something went wrong"
        ));
    }

    #[test]
    fn test_is_retryable_status_message_non_deleted_vpc_attachments() {
        assert!(AwsccProvider::is_retryable_status_message(
            "tgw-0abc123def456 has non-deleted VPC Attachments: tgw-attach-0abc123def456"
        ));
    }

    #[test]
    fn test_is_not_retryable_status_message() {
        assert!(!AwsccProvider::is_retryable_status_message(
            "InvalidParameterValue: invalid CIDR"
        ));
        assert!(!AwsccProvider::is_retryable_status_message(
            "ResourceNotFoundException: not found"
        ));
        assert!(!AwsccProvider::is_retryable_status_message(
            "AccessDeniedException: not authorized"
        ));
    }

    #[test]
    fn operation_error_retry_classification_uses_typed_status_cause() {
        let retryable = AwsccProvider::classify_progress_event(&progress(
            OperationStatus::Failed,
            None,
            Some("Throttling: Rate exceeded"),
        ))
        .expect_err("failed progress must produce an error");
        let non_retryable = AwsccProvider::classify_progress_event(&progress(
            OperationStatus::Failed,
            None,
            Some("InvalidParameterValue: invalid CIDR"),
        ))
        .expect_err("failed progress must produce an error");

        assert!(AwsccProvider::is_retryable_operation_error(&retryable));
        assert!(!AwsccProvider::is_retryable_operation_error(&non_retryable));
    }

    // =========================================================================
    // is_retryable_sdk_error tests
    // =========================================================================

    fn error_meta(code: &str) -> aws_smithy_types::error::ErrorMetadata {
        aws_smithy_types::error::ErrorMetadata::builder()
            .code(code)
            .build()
    }

    fn progress(
        status: OperationStatus,
        identifier: Option<&str>,
        message: Option<&str>,
    ) -> ProgressEvent {
        let mut builder = ProgressEvent::builder().operation_status(status);
        if let Some(identifier) = identifier {
            builder = builder.identifier(identifier);
        }
        if let Some(message) = message {
            builder = builder.status_message(message);
        }
        builder.build()
    }

    #[test]
    fn classify_failed_progress_with_identifier_as_partial_or_failed() {
        let progress = progress(
            OperationStatus::Failed,
            Some("arn:aws:test:resource/created"),
            Some("post-create read denied"),
        );

        let outcome = AwsccProvider::classify_progress_event(&progress)
            .expect("classification should not fail")
            .expect("terminal failed progress must produce an outcome");

        assert_eq!(
            outcome,
            WaitOutcome::PartialOrFailed {
                identifier: "arn:aws:test:resource/created".to_string(),
                status_message: "post-create read denied".to_string(),
            }
        );
    }

    #[test]
    fn classify_failed_progress_without_identifier_as_error() {
        let progress = progress(
            OperationStatus::Failed,
            None,
            Some("handler failed before identifier"),
        );

        let err = AwsccProvider::classify_progress_event(&progress)
            .expect_err("failed progress without identifier must remain an error");

        assert_eq!(err.detail().context(), "Operation failed");
        assert_eq!(
            err.to_string(),
            "Operation failed: handler failed before identifier"
        );
    }

    #[test]
    fn classify_failed_progress_with_empty_identifier_as_error() {
        let progress = progress(
            OperationStatus::Failed,
            Some(""),
            Some("handler failed before useful identifier"),
        );

        let err = AwsccProvider::classify_progress_event(&progress)
            .expect_err("empty identifier must not be treated as partial success");

        assert_eq!(err.detail().context(), "Operation failed");
        assert_eq!(
            err.to_string(),
            "Operation failed: handler failed before useful identifier"
        );
    }

    #[test]
    fn classify_cancel_complete_with_identifier_as_partial_or_failed() {
        let progress = progress(
            OperationStatus::CancelComplete,
            Some("created-before-cancel"),
            Some("operation cancelled after identifier"),
        );

        let outcome = AwsccProvider::classify_progress_event(&progress)
            .expect("classification should not fail")
            .expect("terminal cancelled progress must produce an outcome");

        assert_eq!(
            outcome,
            WaitOutcome::PartialOrFailed {
                identifier: "created-before-cancel".to_string(),
                status_message: "operation cancelled after identifier".to_string(),
            }
        );
    }

    #[test]
    fn classify_cancel_complete_without_identifier_as_error() {
        let progress = progress(
            OperationStatus::CancelComplete,
            None,
            Some("operation cancelled before identifier"),
        );

        let err = AwsccProvider::classify_progress_event(&progress)
            .expect_err("cancel without identifier must remain an error");

        assert_eq!(err.detail().context(), "Operation failed");
        assert_eq!(
            err.to_string(),
            "Operation failed: operation cancelled before identifier"
        );
    }

    #[test]
    fn test_is_retryable_sdk_error_throttling() {
        use aws_sdk_cloudcontrol::operation::create_resource::CreateResourceError;
        use aws_sdk_cloudcontrol::types::error::ThrottlingException;
        use aws_smithy_runtime_api::client::result::SdkError;

        let err = CreateResourceError::ThrottlingException(
            ThrottlingException::builder()
                .message("Rate exceeded")
                .meta(error_meta("ThrottlingException"))
                .build(),
        );
        let sdk_err = SdkError::service_error(err, http::Response::new(""));
        assert!(AwsccProvider::is_retryable_sdk_error(&sdk_err));
    }

    #[test]
    fn test_is_retryable_sdk_error_service_internal() {
        use aws_sdk_cloudcontrol::operation::create_resource::CreateResourceError;
        use aws_sdk_cloudcontrol::types::error::ServiceInternalErrorException;
        use aws_smithy_runtime_api::client::result::SdkError;

        let err = CreateResourceError::ServiceInternalErrorException(
            ServiceInternalErrorException::builder()
                .message("Internal error")
                .meta(error_meta("ServiceInternalErrorException"))
                .build(),
        );
        let sdk_err = SdkError::service_error(err, http::Response::new(""));
        assert!(AwsccProvider::is_retryable_sdk_error(&sdk_err));
    }

    #[test]
    fn test_is_retryable_sdk_error_handler_failure() {
        use aws_sdk_cloudcontrol::operation::create_resource::CreateResourceError;
        use aws_sdk_cloudcontrol::types::error::HandlerFailureException;
        use aws_smithy_runtime_api::client::result::SdkError;

        let err = CreateResourceError::HandlerFailureException(
            HandlerFailureException::builder()
                .message("Handler failed")
                .meta(error_meta("HandlerFailureException"))
                .build(),
        );
        let sdk_err = SdkError::service_error(err, http::Response::new(""));
        assert!(AwsccProvider::is_retryable_sdk_error(&sdk_err));
    }

    #[test]
    fn test_is_retryable_sdk_error_handler_internal_failure() {
        use aws_sdk_cloudcontrol::operation::create_resource::CreateResourceError;
        use aws_sdk_cloudcontrol::types::error::HandlerInternalFailureException;
        use aws_smithy_runtime_api::client::result::SdkError;

        let err = CreateResourceError::HandlerInternalFailureException(
            HandlerInternalFailureException::builder()
                .message("Internal failure")
                .meta(error_meta("HandlerInternalFailureException"))
                .build(),
        );
        let sdk_err = SdkError::service_error(err, http::Response::new(""));
        assert!(AwsccProvider::is_retryable_sdk_error(&sdk_err));
    }

    #[test]
    fn test_is_retryable_sdk_error_network_failure() {
        use aws_sdk_cloudcontrol::operation::create_resource::CreateResourceError;
        use aws_sdk_cloudcontrol::types::error::NetworkFailureException;
        use aws_smithy_runtime_api::client::result::SdkError;

        let err = CreateResourceError::NetworkFailureException(
            NetworkFailureException::builder()
                .message("Network error")
                .meta(error_meta("NetworkFailureException"))
                .build(),
        );
        let sdk_err = SdkError::service_error(err, http::Response::new(""));
        assert!(AwsccProvider::is_retryable_sdk_error(&sdk_err));
    }

    #[test]
    fn test_is_retryable_sdk_error_concurrent_operation() {
        use aws_sdk_cloudcontrol::operation::create_resource::CreateResourceError;
        use aws_sdk_cloudcontrol::types::error::ConcurrentOperationException;
        use aws_smithy_runtime_api::client::result::SdkError;

        let err = CreateResourceError::ConcurrentOperationException(
            ConcurrentOperationException::builder()
                .message("Concurrent operation")
                .meta(error_meta("ConcurrentOperationException"))
                .build(),
        );
        let sdk_err = SdkError::service_error(err, http::Response::new(""));
        assert!(AwsccProvider::is_retryable_sdk_error(&sdk_err));
    }

    #[test]
    fn test_is_retryable_sdk_error_not_stabilized() {
        use aws_sdk_cloudcontrol::operation::create_resource::CreateResourceError;
        use aws_sdk_cloudcontrol::types::error::NotStabilizedException;
        use aws_smithy_runtime_api::client::result::SdkError;

        let err = CreateResourceError::NotStabilizedException(
            NotStabilizedException::builder()
                .message("Not stabilized")
                .meta(error_meta("NotStabilizedException"))
                .build(),
        );
        let sdk_err = SdkError::service_error(err, http::Response::new(""));
        assert!(AwsccProvider::is_retryable_sdk_error(&sdk_err));
    }

    #[test]
    fn test_is_not_retryable_sdk_error_invalid_request() {
        use aws_sdk_cloudcontrol::operation::create_resource::CreateResourceError;
        use aws_sdk_cloudcontrol::types::error::InvalidRequestException;
        use aws_smithy_runtime_api::client::result::SdkError;

        let err = CreateResourceError::InvalidRequestException(
            InvalidRequestException::builder()
                .message("Invalid request")
                .meta(error_meta("InvalidRequestException"))
                .build(),
        );
        let sdk_err = SdkError::service_error(err, http::Response::new(""));
        assert!(!AwsccProvider::is_retryable_sdk_error(&sdk_err));
    }

    #[test]
    fn test_is_not_retryable_sdk_error_already_exists() {
        use aws_sdk_cloudcontrol::operation::create_resource::CreateResourceError;
        use aws_sdk_cloudcontrol::types::error::AlreadyExistsException;
        use aws_smithy_runtime_api::client::result::SdkError;

        let err = CreateResourceError::AlreadyExistsException(
            AlreadyExistsException::builder()
                .message("Already exists")
                .meta(error_meta("AlreadyExistsException"))
                .build(),
        );
        let sdk_err = SdkError::service_error(err, http::Response::new(""));
        assert!(!AwsccProvider::is_retryable_sdk_error(&sdk_err));
    }

    #[test]
    fn test_is_retryable_sdk_error_timeout() {
        use aws_sdk_cloudcontrol::operation::create_resource::CreateResourceError;
        use aws_smithy_runtime_api::client::result::SdkError;

        let sdk_err: SdkError<CreateResourceError, http::Response<&str>> =
            SdkError::timeout_error("connection timed out");
        assert!(AwsccProvider::is_retryable_sdk_error(&sdk_err));
    }

    #[test]
    fn test_is_retryable_sdk_error_delete_throttling() {
        use aws_sdk_cloudcontrol::operation::delete_resource::DeleteResourceError;
        use aws_sdk_cloudcontrol::types::error::ThrottlingException;
        use aws_smithy_runtime_api::client::result::SdkError;

        let err = DeleteResourceError::ThrottlingException(
            ThrottlingException::builder()
                .message("Rate exceeded")
                .meta(error_meta("ThrottlingException"))
                .build(),
        );
        let sdk_err = SdkError::service_error(err, http::Response::new(""));
        assert!(AwsccProvider::is_retryable_sdk_error(&sdk_err));
    }

    #[test]
    fn test_is_not_retryable_sdk_error_delete_type_not_found() {
        use aws_sdk_cloudcontrol::operation::delete_resource::DeleteResourceError;
        use aws_sdk_cloudcontrol::types::error::TypeNotFoundException;
        use aws_smithy_runtime_api::client::result::SdkError;

        let err = DeleteResourceError::TypeNotFoundException(
            TypeNotFoundException::builder()
                .message("Type not found")
                .meta(error_meta("TypeNotFoundException"))
                .build(),
        );
        let sdk_err = SdkError::service_error(err, http::Response::new(""));
        assert!(!AwsccProvider::is_retryable_sdk_error(&sdk_err));
    }

    #[test]
    fn test_is_retryable_sdk_error_client_token_conflict() {
        use aws_sdk_cloudcontrol::operation::delete_resource::DeleteResourceError;
        use aws_sdk_cloudcontrol::types::error::ClientTokenConflictException;
        use aws_smithy_runtime_api::client::result::SdkError;

        let err = DeleteResourceError::ClientTokenConflictException(
            ClientTokenConflictException::builder()
                .message("ClientToken is already associated with an existing operation")
                .meta(error_meta("ClientTokenConflictException"))
                .build(),
        );
        let sdk_err = SdkError::service_error(err, http::Response::new(""));
        assert!(AwsccProvider::is_retryable_sdk_error(&sdk_err));
    }

    #[test]
    fn test_is_retryable_sdk_error_non_deleted_vpc_attachments() {
        use aws_sdk_cloudcontrol::operation::delete_resource::DeleteResourceError;
        use aws_sdk_cloudcontrol::types::error::GeneralServiceException;
        use aws_smithy_runtime_api::client::result::SdkError;

        let meta = aws_smithy_types::error::ErrorMetadata::builder()
            .code("GeneralServiceException")
            .message("tgw-0abc123def456 has non-deleted VPC Attachments: tgw-attach-0abc123def456")
            .build();
        let err = DeleteResourceError::GeneralServiceException(
            GeneralServiceException::builder()
                .message(
                    "tgw-0abc123def456 has non-deleted VPC Attachments: tgw-attach-0abc123def456",
                )
                .meta(meta)
                .build(),
        );
        let sdk_err = SdkError::service_error(err, http::Response::new(""));
        assert!(AwsccProvider::is_retryable_sdk_error(&sdk_err));
    }

    // =========================================================================
    // is_not_found_error tests
    // =========================================================================

    #[test]
    fn test_is_not_found_error_resource_not_found() {
        use aws_sdk_cloudcontrol::operation::get_resource::GetResourceError;
        use aws_sdk_cloudcontrol::types::error::ResourceNotFoundException;
        use aws_smithy_runtime_api::client::result::SdkError;

        let err = GetResourceError::ResourceNotFoundException(
            ResourceNotFoundException::builder()
                .message("Resource not found")
                .meta(error_meta("ResourceNotFoundException"))
                .build(),
        );
        let sdk_err = SdkError::service_error(err, http::Response::new(""));
        assert!(AwsccProvider::is_not_found_error(&sdk_err));
    }

    #[test]
    fn test_is_not_found_error_handler_not_found_code() {
        use aws_sdk_cloudcontrol::operation::get_resource::GetResourceError;
        use aws_smithy_runtime_api::client::result::SdkError;

        let err = GetResourceError::generic(
            aws_smithy_types::error::ErrorMetadata::builder()
                .code("HandlerNotFoundException")
                .message("Handler not found")
                .build(),
        );
        let sdk_err = SdkError::service_error(err, http::Response::new(""));
        assert!(AwsccProvider::is_not_found_error(&sdk_err));
    }

    #[test]
    fn test_is_not_found_error_false_for_throttling() {
        use aws_sdk_cloudcontrol::operation::get_resource::GetResourceError;
        use aws_sdk_cloudcontrol::types::error::ThrottlingException;
        use aws_smithy_runtime_api::client::result::SdkError;

        let err = GetResourceError::ThrottlingException(
            ThrottlingException::builder()
                .message("Rate exceeded")
                .meta(error_meta("ThrottlingException"))
                .build(),
        );
        let sdk_err = SdkError::service_error(err, http::Response::new(""));
        assert!(!AwsccProvider::is_not_found_error(&sdk_err));
    }

    #[test]
    fn test_is_not_found_error_false_for_timeout() {
        use aws_sdk_cloudcontrol::operation::get_resource::GetResourceError;
        use aws_smithy_runtime_api::client::result::SdkError;

        let sdk_err: SdkError<GetResourceError, http::Response<&str>> =
            SdkError::timeout_error("connection timed out");
        assert!(!AwsccProvider::is_not_found_error(&sdk_err));
    }

    #[test]
    fn test_max_polling_attempts_ipam_pool_delete() {
        assert_eq!(
            AwsccProvider::default_polling_attempts("AWS::EC2::IPAMPool", "delete"),
            360
        );
    }

    #[test]
    fn test_max_polling_attempts_ipam_delete() {
        assert_eq!(
            AwsccProvider::default_polling_attempts("AWS::EC2::IPAM", "delete"),
            360
        );
    }

    #[test]
    fn test_max_polling_attempts_default_delete() {
        assert_eq!(
            AwsccProvider::default_polling_attempts("AWS::EC2::VPC", "delete"),
            120
        );
    }

    #[test]
    fn test_max_polling_attempts_ipam_create() {
        assert_eq!(
            AwsccProvider::default_polling_attempts("AWS::EC2::IPAMPool", "create"),
            120
        );
    }

    #[test]
    fn test_max_polling_attempts_vpc_gateway_attachment_delete() {
        // VPCGatewayAttachment deletion via CloudControl API can be slow,
        // especially when dependent resources (NAT gateways) are still
        // being cleaned up. Extended timeout prevents premature timeout
        // failures. (issue #1066)
        assert_eq!(
            AwsccProvider::default_polling_attempts("AWS::EC2::VPCGatewayAttachment", "delete"),
            360
        );
    }

    #[test]
    fn test_max_polling_attempts_vpc_gateway_attachment_create() {
        // Create operations use the default timeout
        assert_eq!(
            AwsccProvider::default_polling_attempts("AWS::EC2::VPCGatewayAttachment", "create"),
            120
        );
    }

    #[test]
    fn test_max_polling_attempts_transit_gateway_delete() {
        // TransitGateway deletion can take 10-20 minutes during
        // create_before_destroy while VPC attachments are being detached.
        assert_eq!(
            AwsccProvider::default_polling_attempts("AWS::EC2::TransitGateway", "delete"),
            360
        );
    }

    #[test]
    fn test_max_polling_attempts_transit_gateway_attachment_delete() {
        assert_eq!(
            AwsccProvider::default_polling_attempts("AWS::EC2::TransitGatewayAttachment", "delete"),
            360
        );
    }

    #[test]
    fn test_max_polling_attempts_transit_gateway_create() {
        // Create operations use the default timeout
        assert_eq!(
            AwsccProvider::default_polling_attempts("AWS::EC2::TransitGateway", "create"),
            120
        );
    }

    #[test]
    fn test_max_polling_attempts_nat_gateway_delete() {
        // NatGateway deletion via CloudControl API can take 10-15 minutes.
        // Extended timeout prevents premature timeout failures. (issue #1443)
        assert_eq!(
            AwsccProvider::default_polling_attempts("AWS::EC2::NatGateway", "delete"),
            240
        );
    }

    #[test]
    fn test_max_polling_attempts_nat_gateway_create() {
        // Create operations use the default timeout
        assert_eq!(
            AwsccProvider::default_polling_attempts("AWS::EC2::NatGateway", "create"),
            120
        );
    }
}
