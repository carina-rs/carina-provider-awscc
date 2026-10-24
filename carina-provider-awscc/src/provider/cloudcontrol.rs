//! Cloud Control API interaction layer.
//!
//! This module contains low-level methods for interacting with the AWS Cloud Control
//! API: creating, reading, updating, and deleting resources, as well as polling for
//! operation completion and retry/error handling logic.

use aws_sdk_cloudcontrol::types::{HandlerErrorCode, OperationStatus, ProgressEvent};
use std::future::Future;

use aws_smithy_runtime_api::client::{orchestrator::HttpResponse, result::SdkError};
use aws_smithy_types::error::metadata::ProvideErrorMetadata;
use carina_core::provider::{ProviderError, ProviderResult};
use carina_provider_protocol::OperationStep;
use serde::{Deserialize, Serialize};

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

/// Opaque CloudControl continuation identifier returned by a mutating request.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct RequestToken(String);

impl RequestToken {
    pub(crate) fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<&str> for RequestToken {
    fn from(value: &str) -> Self {
        Self(value.to_string())
    }
}

impl From<String> for RequestToken {
    fn from(value: String) -> Self {
        Self(value)
    }
}

/// Shared continuation state for one CloudControl mutating request.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "phase", rename_all = "snake_case")]
pub enum CcRequestPhase {
    NotIssued { attempt: u32 },
    InFlight { request: RequestToken, attempt: u32 },
}

/// Continuation state for a CloudControl create operation.
///
/// CloudControl can disclose the physical identifier before the request is
/// terminal. Keep it beside the request token so every later poll or resume
/// can preserve both pieces of evidence. The client token is generated once
/// per logical create and retained across every resend for idempotency.
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "phase", rename_all = "snake_case")]
pub enum CcCreateToken {
    NotIssued {
        attempt: u32,
        #[serde(default = "new_create_client_token")]
        client_token: String,
    },
    InFlight {
        request: RequestToken,
        attempt: u32,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        identifier: Option<String>,
        #[serde(default = "new_create_client_token")]
        client_token: String,
    },
}

impl std::fmt::Debug for CcCreateToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotIssued { attempt, .. } => f
                .debug_struct("NotIssued")
                .field("attempt", attempt)
                .finish(),
            Self::InFlight {
                request,
                attempt,
                identifier,
                ..
            } => f
                .debug_struct("InFlight")
                .field("request", request)
                .field("attempt", attempt)
                .field("identifier", identifier)
                .finish(),
        }
    }
}

impl CcCreateToken {
    pub(crate) fn identifier(&self) -> Option<&str> {
        match self {
            Self::NotIssued { .. } => None,
            Self::InFlight { identifier, .. } => identifier.as_deref(),
        }
    }

    pub(crate) fn attempt(&self) -> u32 {
        match self {
            Self::NotIssued { attempt, .. } | Self::InFlight { attempt, .. } => *attempt,
        }
    }

    pub(crate) fn client_token(&self) -> &str {
        match self {
            Self::NotIssued { client_token, .. } | Self::InFlight { client_token, .. } => {
                client_token
            }
        }
    }

    pub(crate) fn retry_not_issued(&self, attempt: u32) -> Self {
        Self::NotIssued {
            attempt,
            client_token: self.client_token().to_string(),
        }
    }

    pub(crate) fn with_identifier(self, identifier: Option<String>) -> Self {
        match self {
            Self::NotIssued {
                attempt,
                client_token,
            } => Self::NotIssued {
                attempt,
                client_token,
            },
            Self::InFlight {
                request,
                attempt,
                identifier: _,
                client_token,
            } => Self::InFlight {
                request,
                attempt,
                identifier,
                client_token,
            },
        }
    }
}

fn new_create_client_token() -> String {
    uuid::Uuid::new_v4().to_string()
}

/// Continuation state for a CloudControl update operation.
pub type CcUpdateToken = CcRequestPhase;

/// Continuation state for a CloudControl delete operation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "phase", content = "request", rename_all = "snake_case")]
pub enum CcDeleteToken {
    DetachIgw(CcRequestPhase),
    Delete(CcRequestPhase),
}

type CcWaitStep = OperationStep<WaitOutcome, ProviderError, CcRequestPhase>;
pub(crate) type CcUpdateWaitStep = CcWaitStep;
pub(crate) type CcDeleteWaitStep = OperationStep<(), ProviderError, CcRequestPhase>;

const FIRST_POLL_AFTER_SECS: u32 = 1;
const POLL_AFTER_SECS: u32 = 5;

#[derive(Debug)]
enum RequestStatusStep {
    Pending,
    Done(WaitOutcome),
    RetryableFailure { error: ProviderError },
}

#[derive(Debug)]
pub(crate) struct CcCreatePending {
    pub(crate) token: CcCreateToken,
    pub(crate) poll_after_secs: u32,
}

#[derive(Debug)]
pub(crate) enum CcCreateStatus {
    Pending(CcCreatePending),
    Success {
        token: CcCreateToken,
        identifier: Option<String>,
    },
    Failed {
        token: CcCreateToken,
        status: OperationStatus,
        identifier: Option<String>,
        error_code: Option<HandlerErrorCode>,
        status_message: String,
    },
    RequestTokenNotFound {
        token: CcCreateToken,
        error: ProviderError,
    },
    StatusError {
        token: CcCreateToken,
        error: ProviderError,
        poll_after_secs: u32,
    },
}

#[derive(Debug, Clone, Copy)]
struct RetryPolicy {
    action: &'static str,
    max_attempts: u32,
    initial_delay_secs: u64,
    max_delay_secs: u64,
}

impl RetryPolicy {
    fn delay_secs(self, attempt: u32) -> u32 {
        let multiplier = 2_u64.saturating_pow(attempt);
        u32::try_from(
            self.initial_delay_secs
                .saturating_mul(multiplier)
                .min(self.max_delay_secs),
        )
        .expect("CloudControl retry delay constants fit in u32")
    }
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

    /// Issue one CloudControl create request.
    pub(crate) async fn cc_create_step(
        &self,
        type_name: &str,
        desired_state: serde_json::Value,
        operation_config: Option<&carina_core::schema::OperationConfig>,
    ) -> ProviderResult<CcCreatePending> {
        self.issue_create(
            type_name,
            desired_state,
            0,
            new_create_client_token(),
            Self::request_retry_policy(operation_config, "creating"),
        )
        .await
    }

    /// Re-issue a create whose previous send did not return a request token.
    pub(crate) async fn cc_reissue_create_step(
        &self,
        type_name: &str,
        desired_state: serde_json::Value,
        operation_config: Option<&carina_core::schema::OperationConfig>,
        token: CcCreateToken,
    ) -> ProviderResult<CcCreatePending> {
        let retry_policy = Self::request_retry_policy(operation_config, "creating");
        let CcCreateToken::NotIssued {
            attempt,
            client_token,
        } = token
        else {
            return Err(ProviderError::internal(
                "cannot re-issue an in-flight Cloud Control create request",
            ));
        };
        self.issue_create(
            type_name,
            desired_state,
            attempt,
            client_token,
            retry_policy,
        )
        .await
    }

    async fn issue_create(
        &self,
        type_name: &str,
        desired_state: serde_json::Value,
        attempt: u32,
        client_token: String,
        retry_policy: RetryPolicy,
    ) -> ProviderResult<CcCreatePending> {
        let result = self
            .cloudcontrol_client
            .create_resource()
            .type_name(type_name)
            .desired_state(desired_state.to_string())
            .client_token(&client_token)
            .send()
            .await;

        match result {
            Ok(response) => {
                let progress = response.progress_event();
                let request = progress
                    .and_then(ProgressEvent::request_token)
                    .filter(|request| !request.is_empty())
                    .map(RequestToken::from)
                    .ok_or_else(|| ProviderError::internal("No request token returned"))?;
                let identifier = progress
                    .and_then(ProgressEvent::identifier)
                    .filter(|identifier| !identifier.is_empty())
                    .map(ToString::to_string);
                Ok(CcCreatePending {
                    token: CcCreateToken::InFlight {
                        request,
                        attempt,
                        identifier,
                        client_token,
                    },
                    poll_after_secs: FIRST_POLL_AFTER_SECS,
                })
            }
            Err(error)
                if Self::is_retryable_sdk_error(&error) && attempt < retry_policy.max_attempts =>
            {
                let delay_secs = retry_policy.delay_secs(attempt);
                log::warn!(
                    "Retryable error creating {type_name} (attempt {}/{}): {error:?}. Retrying in {delay_secs}s...",
                    attempt + 1,
                    retry_policy.max_attempts,
                );
                Ok(CcCreatePending {
                    token: CcCreateToken::NotIssued {
                        attempt: attempt + 1,
                        client_token,
                    },
                    poll_after_secs: delay_secs,
                })
            }
            Err(error) => Err(api_error_with_meta_and_subject(
                "Failed to create resource",
                "cloudcontrol.CreateResource",
                CloudControlResourceSubject::type_only(type_name),
                error,
            )),
        }
    }

    /// Fetch and classify one status for an accepted create request.
    pub(crate) async fn cc_create_request_status(&self, token: CcCreateToken) -> CcCreateStatus {
        let CcCreateToken::InFlight {
            request,
            attempt,
            identifier: recorded_identifier,
            client_token,
        } = token
        else {
            return CcCreateStatus::StatusError {
                token,
                error: ProviderError::internal(
                    "cannot poll a Cloud Control create request before it is issued",
                ),
                poll_after_secs: POLL_AFTER_SECS,
            };
        };

        let result = self
            .cloudcontrol_client
            .get_resource_request_status()
            .request_token(request.as_str())
            .send()
            .await;
        let token = CcCreateToken::InFlight {
            request: request.clone(),
            attempt,
            identifier: recorded_identifier.clone(),
            client_token,
        };
        let response = match result {
            Ok(response) => response,
            Err(error) if Self::is_request_token_not_found_error(&error) => {
                return CcCreateStatus::RequestTokenNotFound {
                    token,
                    error: api_error_with_meta_and_subject(
                        "Cloud Control request token is no longer known",
                        "cloudcontrol.GetResourceRequestStatus",
                        CloudControlRequestToken(request.as_str().to_string()),
                        error,
                    ),
                };
            }
            Err(error) => {
                return CcCreateStatus::StatusError {
                    token,
                    error: api_error_with_meta_and_subject(
                        "Failed to get operation status",
                        "cloudcontrol.GetResourceRequestStatus",
                        CloudControlRequestToken(request.as_str().to_string()),
                        error,
                    ),
                    poll_after_secs: POLL_AFTER_SECS,
                };
            }
        };

        let Some(progress) = response.progress_event() else {
            return CcCreateStatus::Pending(CcCreatePending {
                token,
                poll_after_secs: POLL_AFTER_SECS,
            });
        };
        let identifier = progress
            .identifier()
            .filter(|identifier| !identifier.is_empty())
            .map(ToString::to_string)
            .or(recorded_identifier);
        let token = token.with_identifier(identifier.clone());
        match progress.operation_status() {
            None
            | Some(OperationStatus::Pending)
            | Some(OperationStatus::InProgress)
            | Some(OperationStatus::CancelInProgress) => CcCreateStatus::Pending(CcCreatePending {
                token,
                poll_after_secs: POLL_AFTER_SECS,
            }),
            Some(OperationStatus::Success) => CcCreateStatus::Success { token, identifier },
            Some(status @ (OperationStatus::Failed | OperationStatus::CancelComplete)) => {
                CcCreateStatus::Failed {
                    token,
                    status: status.clone(),
                    identifier,
                    error_code: progress.error_code().cloned(),
                    status_message: progress
                        .status_message()
                        .unwrap_or("Unknown error")
                        .to_string(),
                }
            }
            Some(future_status) => {
                log::warn!(
                    "Cloud Control returned unknown create operation status {}; preserving the request as pending",
                    future_status.as_str()
                );
                CcCreateStatus::Pending(CcCreatePending {
                    token,
                    poll_after_secs: POLL_AFTER_SECS,
                })
            }
        }
    }

    /// Issue one CloudControl update request.
    pub(crate) async fn cc_update_step(
        &self,
        type_name: &str,
        identifier: &str,
        patch_ops: Vec<serde_json::Value>,
        operation_config: Option<&carina_core::schema::OperationConfig>,
    ) -> ProviderResult<CcUpdateWaitStep> {
        if patch_ops.is_empty() {
            return Ok(OperationStep::Done(WaitOutcome::Success {
                identifier: identifier.to_string(),
            }));
        }
        self.issue_update(
            type_name,
            identifier,
            patch_ops,
            0,
            Self::request_retry_policy(operation_config, "updating"),
        )
        .await
    }

    /// Perform one poll or re-issue step for a pending update.
    pub(crate) async fn cc_poll_update_step(
        &self,
        type_name: &str,
        identifier: &str,
        patch_ops: Vec<serde_json::Value>,
        operation_config: Option<&carina_core::schema::OperationConfig>,
        token: CcUpdateToken,
    ) -> ProviderResult<CcUpdateWaitStep> {
        let retry_policy = Self::request_retry_policy(operation_config, "updating");
        let step = self
            .poll_phase(type_name, token, retry_policy, |attempt| {
                self.issue_update(type_name, identifier, patch_ops, attempt, retry_policy)
            })
            .await?;
        Ok(match step {
            OperationStep::Done(outcome) => {
                OperationStep::Done(Self::outcome_with_identifier(outcome, identifier))
            }
            OperationStep::Verdict(error) => OperationStep::Verdict(error),
            OperationStep::Pending {
                pending,
                poll_after_secs,
            } => OperationStep::Pending {
                pending,
                poll_after_secs,
            },
        })
    }

    async fn issue_update(
        &self,
        type_name: &str,
        identifier: &str,
        patch_ops: Vec<serde_json::Value>,
        attempt: u32,
        retry_policy: RetryPolicy,
    ) -> ProviderResult<CcUpdateWaitStep> {
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
            .map(|response| {
                response
                    .progress_event()
                    .and_then(|progress| progress.request_token())
                    .map(RequestToken::from)
            });
        Self::finish_issue(
            result,
            type_name,
            attempt,
            retry_policy,
            "Failed to update resource",
            "cloudcontrol.UpdateResource",
            CloudControlResourceSubject::identified(type_name, identifier),
            || {
                Ok(WaitOutcome::Success {
                    identifier: identifier.to_string(),
                })
            },
        )
    }

    /// Issue one CloudControl delete request.
    pub(crate) async fn cc_delete_step(
        &self,
        type_name: &str,
        identifier: &str,
        operation_config: Option<&carina_core::schema::OperationConfig>,
    ) -> ProviderResult<CcDeleteWaitStep> {
        let step = self
            .issue_delete(
                type_name,
                identifier,
                0,
                Self::delete_retry_policy(operation_config),
            )
            .await?;
        Self::finish_delete_step(step)
    }

    /// Perform one poll or re-issue step for a pending delete.
    pub(crate) async fn cc_poll_delete_step(
        &self,
        type_name: &str,
        identifier: &str,
        operation_config: Option<&carina_core::schema::OperationConfig>,
        phase: CcRequestPhase,
    ) -> ProviderResult<CcDeleteWaitStep> {
        let retry_policy = Self::delete_retry_policy(operation_config);
        let step = self
            .poll_phase(type_name, phase, retry_policy, |attempt| {
                self.issue_delete(type_name, identifier, attempt, retry_policy)
            })
            .await?;
        Self::finish_delete_step(step)
    }

    async fn issue_delete(
        &self,
        type_name: &str,
        identifier: &str,
        attempt: u32,
        retry_policy: RetryPolicy,
    ) -> ProviderResult<CcWaitStep> {
        let result = self
            .cloudcontrol_client
            .delete_resource()
            .type_name(type_name)
            .identifier(identifier)
            .send()
            .await
            .map(|response| {
                response
                    .progress_event()
                    .and_then(|progress| progress.request_token())
                    .map(RequestToken::from)
            });
        Self::finish_issue(
            result,
            type_name,
            attempt,
            retry_policy,
            "Failed to delete resource",
            "cloudcontrol.DeleteResource",
            CloudControlResourceSubject::identified(type_name, identifier),
            || {
                Ok(WaitOutcome::Success {
                    identifier: identifier.to_string(),
                })
            },
        )
    }

    async fn poll_phase<Issue, IssueFuture>(
        &self,
        type_name: &str,
        phase: CcRequestPhase,
        retry_policy: RetryPolicy,
        issue: Issue,
    ) -> ProviderResult<CcWaitStep>
    where
        Issue: FnOnce(u32) -> IssueFuture,
        IssueFuture: Future<Output = ProviderResult<CcWaitStep>>,
    {
        match phase {
            CcRequestPhase::NotIssued { attempt } => issue(attempt).await,
            CcRequestPhase::InFlight { request, attempt } => {
                match self.poll_request_status(&request).await? {
                    RequestStatusStep::Pending => Ok(OperationStep::Pending {
                        pending: CcRequestPhase::InFlight { request, attempt },
                        poll_after_secs: POLL_AFTER_SECS,
                    }),
                    RequestStatusStep::Done(outcome) => Ok(OperationStep::Done(outcome)),
                    RequestStatusStep::RetryableFailure { error } => {
                        if attempt >= retry_policy.max_attempts {
                            return Err(error);
                        }
                        let delay_secs = retry_policy.delay_secs(attempt);
                        log::warn!(
                            "Retryable error {} {type_name} (attempt {}/{}): {}. Retrying in {}s...",
                            retry_policy.action,
                            attempt + 1,
                            retry_policy.max_attempts,
                            error,
                            delay_secs,
                        );
                        Ok(OperationStep::Pending {
                            pending: CcRequestPhase::NotIssued {
                                attempt: attempt + 1,
                            },
                            poll_after_secs: delay_secs,
                        })
                    }
                }
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn finish_issue<E, Subject, NoRequest>(
        result: Result<Option<RequestToken>, SdkError<E, HttpResponse>>,
        type_name: &str,
        attempt: u32,
        retry_policy: RetryPolicy,
        context: &'static str,
        operation: &'static str,
        subject: Subject,
        no_request: NoRequest,
    ) -> ProviderResult<CcWaitStep>
    where
        E: std::error::Error + ProvideErrorMetadata + Send + Sync + 'static,
        Subject: std::fmt::Debug + std::fmt::Display + Send + Sync + 'static,
        NoRequest: FnOnce() -> ProviderResult<WaitOutcome>,
    {
        match result {
            Ok(Some(request)) => Ok(OperationStep::Pending {
                pending: CcRequestPhase::InFlight { request, attempt },
                poll_after_secs: FIRST_POLL_AFTER_SECS,
            }),
            Ok(None) => no_request().map(OperationStep::Done),
            Err(error)
                if Self::is_retryable_sdk_error(&error) && attempt < retry_policy.max_attempts =>
            {
                let delay_secs = retry_policy.delay_secs(attempt);
                log::warn!(
                    "Retryable error {} {type_name} (attempt {}/{}): {error:?}. Retrying in {delay_secs}s...",
                    retry_policy.action,
                    attempt + 1,
                    retry_policy.max_attempts,
                );
                Ok(OperationStep::Pending {
                    pending: CcRequestPhase::NotIssued {
                        attempt: attempt + 1,
                    },
                    poll_after_secs: delay_secs,
                })
            }
            Err(error) => Err(api_error_with_meta_and_subject(
                context, operation, subject, error,
            )),
        }
    }

    fn finish_delete_step(step: CcWaitStep) -> ProviderResult<CcDeleteWaitStep> {
        match step {
            OperationStep::Done(WaitOutcome::Success { .. }) => Ok(OperationStep::Done(())),
            OperationStep::Done(WaitOutcome::PartialOrFailed { status_message, .. }) => {
                Err(ProviderError::api_error_caused_by(
                    "Operation failed",
                    CloudControlOperationFailure::new(status_message),
                ))
            }
            OperationStep::Verdict(error) => Ok(OperationStep::Verdict(error)),
            OperationStep::Pending {
                pending,
                poll_after_secs,
            } => Ok(OperationStep::Pending {
                pending,
                poll_after_secs,
            }),
        }
    }

    fn request_retry_policy(
        operation_config: Option<&carina_core::schema::OperationConfig>,
        action: &'static str,
    ) -> RetryPolicy {
        RetryPolicy {
            action,
            max_attempts: operation_config
                .and_then(|config| config.create_max_retries)
                .unwrap_or(CREATE_RETRY_MAX_ATTEMPTS),
            initial_delay_secs: CREATE_RETRY_INITIAL_DELAY_SECS,
            max_delay_secs: CREATE_RETRY_MAX_DELAY_SECS,
        }
    }

    fn delete_retry_policy(
        operation_config: Option<&carina_core::schema::OperationConfig>,
    ) -> RetryPolicy {
        RetryPolicy {
            action: "deleting",
            max_attempts: operation_config
                .and_then(|config| config.delete_max_retries)
                .unwrap_or(DELETE_RETRY_MAX_ATTEMPTS),
            initial_delay_secs: DELETE_RETRY_INITIAL_DELAY_SECS,
            max_delay_secs: DELETE_RETRY_MAX_DELAY_SECS,
        }
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

    fn is_request_token_not_found_error<E, R>(error: &SdkError<E, R>) -> bool
    where
        E: ProvideErrorMetadata,
    {
        matches!(
            error,
            SdkError::ServiceError(service_error)
                if service_error.err().code() == Some("RequestTokenNotFoundException")
        )
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

    /// Fetch and classify exactly one CloudControl request status.
    async fn poll_request_status(
        &self,
        request: &RequestToken,
    ) -> ProviderResult<RequestStatusStep> {
        let status = self
            .cloudcontrol_client
            .get_resource_request_status()
            .request_token(request.as_str())
            .send()
            .await
            .map_err(|error| {
                api_error_with_meta_and_subject(
                    "Failed to get operation status",
                    "cloudcontrol.GetResourceRequestStatus",
                    CloudControlRequestToken(request.as_str().to_string()),
                    error,
                )
            })?;

        let Some(progress) = status.progress_event() else {
            return Ok(RequestStatusStep::Pending);
        };
        match Self::classify_progress_event(progress) {
            Ok(Some(outcome)) => Ok(RequestStatusStep::Done(outcome)),
            Ok(None) => Ok(RequestStatusStep::Pending),
            Err(error)
                if matches!(
                    progress.operation_status(),
                    Some(OperationStatus::Failed | OperationStatus::CancelComplete)
                ) && progress
                    .status_message()
                    .is_some_and(Self::is_retryable_status_message) =>
            {
                Ok(RequestStatusStep::RetryableFailure { error })
            }
            Err(error) => Err(error),
        }
    }

    fn outcome_with_identifier(outcome: WaitOutcome, fallback: &str) -> WaitOutcome {
        match outcome {
            WaitOutcome::Success { identifier } if identifier.is_empty() => WaitOutcome::Success {
                identifier: fallback.to_string(),
            },
            WaitOutcome::PartialOrFailed {
                identifier,
                status_message,
            } if identifier.is_empty() => WaitOutcome::PartialOrFailed {
                identifier: fallback.to_string(),
                status_message,
            },
            other => other,
        }
    }

    pub(crate) fn classify_progress_event(
        progress: &ProgressEvent,
    ) -> ProviderResult<Option<WaitOutcome>> {
        match progress.operation_status() {
            None
            | Some(OperationStatus::Pending)
            | Some(OperationStatus::InProgress)
            | Some(OperationStatus::CancelInProgress) => Ok(None),
            Some(OperationStatus::Success) => Ok(Some(WaitOutcome::Success {
                identifier: progress.identifier().unwrap_or("").to_string(),
            })),
            Some(OperationStatus::Failed) => Self::terminal_failure_outcome(progress),
            Some(OperationStatus::CancelComplete) => Self::terminal_failure_outcome(progress),
            Some(future_status) => {
                log::warn!(
                    "Cloud Control returned unknown operation status {}; preserving the request as pending",
                    future_status.as_str()
                );
                Ok(None)
            }
        }
    }

    fn terminal_failure_outcome(progress: &ProgressEvent) -> ProviderResult<Option<WaitOutcome>> {
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
}
