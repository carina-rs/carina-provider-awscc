//! Cloud Control API interaction layer.
//!
//! This module contains low-level methods for interacting with the AWS Cloud Control
//! API: creating, reading, updating, and deleting resources, as well as polling for
//! operation completion and retry/error handling logic.

use aws_sdk_cloudcontrol::{
    operation::{
        create_resource::{CreateResourceError, CreateResourceOutput},
        delete_resource::DeleteResourceError,
        update_resource::UpdateResourceError,
    },
    types::{HandlerErrorCode, OperationStatus, ProgressEvent},
};

use aws_smithy_runtime_api::client::{orchestrator::HttpResponse, result::SdkError};
use aws_smithy_types::error::metadata::ProvideErrorMetadata;
use aws_smithy_types::retry::ProvideErrorKind;
use carina_aws_types::SendOutcome;
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

mod client_token {
    use super::{OperationStatus, ProviderError, RequestToken};
    use serde::{Deserialize, Serialize};

    /// Opaque idempotency token for one logical CloudControl request.
    #[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
    #[serde(try_from = "String")]
    pub struct ClientToken(String);

    impl ClientToken {
        pub(super) fn as_str(&self) -> &str {
            &self.0
        }
    }

    impl std::fmt::Debug for ClientToken {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.write_str("ClientToken(<redacted>)")
        }
    }

    impl TryFrom<String> for ClientToken {
        type Error = String;

        fn try_from(value: String) -> Result<Self, Self::Error> {
            if !(1..=128).contains(&value.len()) {
                return Err("CloudControl ClientToken length must be between 1 and 128".into());
            }
            if !value.bytes().all(|byte| {
                byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'+' | b'/' | b'=')
            }) {
                return Err(
                    "CloudControl ClientToken must match the pattern [-A-Za-z0-9+/=]+".into(),
                );
            }
            Ok(Self(value))
        }
    }

    fn fresh_client_token() -> ClientToken {
        ClientToken(uuid::Uuid::new_v4().to_string())
    }

    /// A non-create mutation that has not been sent.
    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    pub struct NotIssuedRequest {
        attempt: u32,
        client_token: ClientToken,
    }

    impl NotIssuedRequest {
        pub fn attempt(&self) -> u32 {
            self.attempt
        }

        pub(super) fn start() -> Self {
            Self {
                attempt: 0,
                client_token: fresh_client_token(),
            }
        }

        pub(super) fn client_token(&self) -> &ClientToken {
            &self.client_token
        }

        pub(super) fn into_in_flight(self, request: RequestToken) -> InFlightRequest {
            InFlightRequest::new(request, self.attempt)
        }
    }

    /// A non-create mutation whose previous send may have been accepted.
    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    pub struct UnconfirmedRequest {
        attempt: u32,
        resends: u32,
        client_token: ClientToken,
    }

    impl UnconfirmedRequest {
        pub fn attempt(&self) -> u32 {
            self.attempt
        }

        pub fn resends(&self) -> u32 {
            self.resends
        }

        pub(super) fn client_token(&self) -> &ClientToken {
            &self.client_token
        }

        pub(super) fn into_in_flight(self, request: RequestToken) -> InFlightRequest {
            InFlightRequest::new(request, self.attempt)
        }
    }

    /// A non-create mutation accepted by CloudControl.
    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    pub struct InFlightRequest {
        request: RequestToken,
        attempt: u32,
    }

    impl InFlightRequest {
        pub fn request(&self) -> &RequestToken {
            &self.request
        }

        pub fn attempt(&self) -> u32 {
            self.attempt
        }

        fn new(request: RequestToken, attempt: u32) -> Self {
            Self { request, attempt }
        }
    }

    /// Proof that an accepted non-create mutation reached a terminal failure.
    #[derive(Debug)]
    pub(crate) struct TerminallyFailedRequest {
        request: InFlightRequest,
    }

    impl TerminallyFailedRequest {
        pub(super) fn from_status(
            request: InFlightRequest,
            status: OperationStatus,
        ) -> Option<Self> {
            matches!(
                status,
                OperationStatus::Failed | OperationStatus::CancelComplete
            )
            .then_some(Self { request })
        }

        pub(crate) fn attempt(&self) -> u32 {
            self.request.attempt
        }

        pub(crate) fn retry_after_terminal_failure(self) -> NotIssuedRequest {
            NotIssuedRequest {
                attempt: self.request.attempt + 1,
                client_token: fresh_client_token(),
            }
        }
    }

    /// Shared continuation state for one CloudControl mutating request.
    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    #[serde(tag = "phase", rename_all = "snake_case")]
    pub enum CcRequestPhase {
        /// No send could have been accepted by CloudControl.
        NotIssued(NotIssuedRequest),
        /// A send may have been accepted; only the same request and token may be resent.
        Unconfirmed(UnconfirmedRequest),
        /// CloudControl accepted the request and returned a request token.
        InFlight(InFlightRequest),
    }

    impl CcRequestPhase {
        pub(super) fn start() -> Self {
            NotIssuedRequest::start().into()
        }
    }

    impl From<NotIssuedRequest> for CcRequestPhase {
        fn from(request: NotIssuedRequest) -> Self {
            Self::NotIssued(request)
        }
    }

    impl From<UnconfirmedRequest> for CcRequestPhase {
        fn from(request: UnconfirmedRequest) -> Self {
            Self::Unconfirmed(request)
        }
    }

    impl From<InFlightRequest> for CcRequestPhase {
        fn from(request: InFlightRequest) -> Self {
            Self::InFlight(request)
        }
    }

    /// A create request that has not been sent.
    #[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
    pub struct NotIssuedCreate {
        attempt: u32,
        #[serde(default = "fresh_client_token")]
        client_token: ClientToken,
    }

    impl NotIssuedCreate {
        pub fn attempt(&self) -> u32 {
            self.attempt
        }

        pub(crate) fn start() -> Self {
            Self {
                attempt: 0,
                client_token: fresh_client_token(),
            }
        }

        pub(super) fn client_token(&self) -> &ClientToken {
            &self.client_token
        }

        pub(super) fn into_in_flight(
            self,
            request: RequestToken,
            identifier: Option<String>,
        ) -> InFlightCreateRequest {
            InFlightCreateRequest::new(request, self.attempt, identifier)
        }

        pub(crate) fn into_not_applied(self, error: ProviderError) -> NotAppliedCreate {
            NotAppliedCreate(error)
        }
    }

    /// A create request whose previous send may have been accepted.
    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    pub struct UnconfirmedCreate {
        attempt: u32,
        resends: u32,
        client_token: ClientToken,
    }

    impl UnconfirmedCreate {
        pub fn attempt(&self) -> u32 {
            self.attempt
        }

        pub fn resends(&self) -> u32 {
            self.resends
        }

        pub(super) fn client_token(&self) -> &ClientToken {
            &self.client_token
        }

        pub(super) fn to_in_flight(
            &self,
            request: RequestToken,
            identifier: Option<String>,
        ) -> InFlightCreateRequest {
            InFlightCreateRequest::new(request, self.attempt, identifier)
        }
    }

    /// A create request accepted by CloudControl.
    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    pub struct InFlightCreateRequest {
        request: RequestToken,
        attempt: u32,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        identifier: Option<String>,
    }

    impl InFlightCreateRequest {
        pub fn request(&self) -> &RequestToken {
            &self.request
        }

        pub fn attempt(&self) -> u32 {
            self.attempt
        }

        pub fn identifier(&self) -> Option<&str> {
            self.identifier.as_deref()
        }

        fn new(request: RequestToken, attempt: u32, identifier: Option<String>) -> Self {
            Self {
                request,
                attempt,
                identifier,
            }
        }

        pub(super) fn with_identifier(self, identifier: Option<String>) -> Self {
            Self { identifier, ..self }
        }
    }

    /// Proof that an accepted create reached a terminal failure.
    #[derive(Debug)]
    pub(crate) struct TerminallyFailedCreate {
        request: InFlightCreateRequest,
        status: OperationStatus,
    }

    impl TerminallyFailedCreate {
        pub(super) fn from_status(
            request: InFlightCreateRequest,
            status: OperationStatus,
        ) -> Option<Self> {
            matches!(
                status,
                OperationStatus::Failed | OperationStatus::CancelComplete
            )
            .then_some(Self { request, status })
        }

        pub(crate) fn status(&self) -> &OperationStatus {
            &self.status
        }

        pub(crate) fn attempt(&self) -> u32 {
            self.request.attempt
        }

        pub(crate) fn into_in_flight(self) -> InFlightCreateRequest {
            self.request
        }

        pub(crate) fn retry_after_terminal_failure(self) -> NotIssuedCreate {
            NotIssuedCreate {
                attempt: self.request.attempt + 1,
                client_token: fresh_client_token(),
            }
        }
    }

    /// Proof that a create failed before any send could have been accepted.
    #[derive(Debug)]
    #[doc(hidden)]
    pub struct NotAppliedCreate(ProviderError);

    impl NotAppliedCreate {
        pub fn into_error(self) -> ProviderError {
            self.0
        }

        pub(crate) fn map_error(self, map: impl FnOnce(ProviderError) -> ProviderError) -> Self {
            Self(map(self.0))
        }
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
        /// No create send could have been accepted by CloudControl.
        NotIssued(NotIssuedCreate),
        /// A create send may have been accepted; only the same request and token may be resent.
        Unconfirmed(UnconfirmedCreate),
        /// CloudControl accepted the create and returned a request token.
        InFlight(InFlightCreateRequest),
    }

    impl std::fmt::Debug for CcCreateToken {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                Self::NotIssued(create) => f
                    .debug_struct("NotIssued")
                    .field("attempt", &create.attempt)
                    .finish(),
                Self::Unconfirmed(create) => f
                    .debug_struct("Unconfirmed")
                    .field("attempt", &create.attempt)
                    .field("resends", &create.resends)
                    .finish(),
                Self::InFlight(create) => f
                    .debug_struct("InFlight")
                    .field("request", &create.request)
                    .field("attempt", &create.attempt)
                    .field("identifier", &create.identifier)
                    .finish(),
            }
        }
    }

    impl CcCreateToken {
        pub fn identifier(&self) -> Option<&str> {
            match self {
                Self::NotIssued(_) | Self::Unconfirmed(_) => None,
                Self::InFlight(create) => create.identifier(),
            }
        }

        pub fn attempt(&self) -> u32 {
            match self {
                Self::NotIssued(create) => create.attempt(),
                Self::Unconfirmed(create) => create.attempt(),
                Self::InFlight(create) => create.attempt(),
            }
        }

        pub(crate) fn with_identifier(self, identifier: Option<String>) -> Self {
            match self {
                Self::InFlight(create) => Self::InFlight(create.with_identifier(identifier)),
                token @ (Self::NotIssued(_) | Self::Unconfirmed(_)) => token,
            }
        }
    }

    impl From<NotIssuedCreate> for CcCreateToken {
        fn from(create: NotIssuedCreate) -> Self {
            Self::NotIssued(create)
        }
    }

    impl From<UnconfirmedCreate> for CcCreateToken {
        fn from(create: UnconfirmedCreate) -> Self {
            Self::Unconfirmed(create)
        }
    }

    impl From<InFlightCreateRequest> for CcCreateToken {
        fn from(create: InFlightCreateRequest) -> Self {
            Self::InFlight(create)
        }
    }

    pub(super) trait NotIssuedPhase: Sized {
        type Unconfirmed;

        fn attempt(&self) -> u32;
        fn retry_rejected(self) -> Self;
        fn into_unconfirmed(self, resends: u32) -> Self::Unconfirmed;
    }

    pub(super) trait UnconfirmedPhase: Sized {
        fn resends(&self) -> u32;
        fn resend(&self, resends: u32) -> Self;
    }

    impl NotIssuedPhase for NotIssuedCreate {
        type Unconfirmed = UnconfirmedCreate;

        fn attempt(&self) -> u32 {
            self.attempt
        }

        fn retry_rejected(mut self) -> Self {
            self.attempt += 1;
            self
        }

        fn into_unconfirmed(self, resends: u32) -> Self::Unconfirmed {
            UnconfirmedCreate {
                attempt: self.attempt,
                resends,
                client_token: self.client_token,
            }
        }
    }

    impl UnconfirmedPhase for UnconfirmedCreate {
        fn resends(&self) -> u32 {
            self.resends
        }

        fn resend(&self, resends: u32) -> Self {
            Self {
                attempt: self.attempt,
                resends,
                client_token: self.client_token.clone(),
            }
        }
    }

    impl NotIssuedPhase for NotIssuedRequest {
        type Unconfirmed = UnconfirmedRequest;

        fn attempt(&self) -> u32 {
            self.attempt
        }

        fn retry_rejected(mut self) -> Self {
            self.attempt += 1;
            self
        }

        fn into_unconfirmed(self, resends: u32) -> Self::Unconfirmed {
            UnconfirmedRequest {
                attempt: self.attempt,
                resends,
                client_token: self.client_token,
            }
        }
    }

    impl UnconfirmedPhase for UnconfirmedRequest {
        fn resends(&self) -> u32 {
            self.resends
        }

        fn resend(&self, resends: u32) -> Self {
            Self {
                attempt: self.attempt,
                resends,
                client_token: self.client_token.clone(),
            }
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        fn in_flight_request() -> InFlightRequest {
            InFlightRequest::new(RequestToken::from("request"), 3)
        }

        fn in_flight_create() -> InFlightCreateRequest {
            InFlightCreateRequest::new(RequestToken::from("create"), 3, None)
        }

        #[test]
        fn terminal_failure_witnesses_accept_only_failed_and_cancel_complete() {
            for status in [
                OperationStatus::Pending,
                OperationStatus::InProgress,
                OperationStatus::CancelInProgress,
                OperationStatus::Success,
            ] {
                assert!(
                    TerminallyFailedRequest::from_status(in_flight_request(), status.clone())
                        .is_none(),
                    "{} must not prove terminal request failure",
                    status.as_str()
                );
                assert!(
                    TerminallyFailedCreate::from_status(in_flight_create(), status.clone())
                        .is_none(),
                    "{} must not prove terminal create failure",
                    status.as_str()
                );
            }

            for status in [OperationStatus::Failed, OperationStatus::CancelComplete] {
                assert!(
                    TerminallyFailedRequest::from_status(in_flight_request(), status.clone())
                        .is_some(),
                    "{} must prove terminal request failure",
                    status.as_str()
                );
                assert!(
                    TerminallyFailedCreate::from_status(in_flight_create(), status.clone())
                        .is_some(),
                    "{} must prove terminal create failure",
                    status.as_str()
                );
            }
        }
    }
}

pub(crate) use client_token::TerminallyFailedCreate;
pub use client_token::{
    CcCreateToken, CcRequestPhase, ClientToken, InFlightCreateRequest, InFlightRequest,
    NotAppliedCreate, NotIssuedCreate, NotIssuedRequest, UnconfirmedCreate, UnconfirmedRequest,
};
use client_token::{NotIssuedPhase, TerminallyFailedRequest, UnconfirmedPhase};

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
    Pending(InFlightRequest),
    Done(WaitOutcome),
    RetryableFailure {
        failure: TerminallyFailedRequest,
        error: ProviderError,
    },
}

#[derive(Debug)]
pub(crate) struct CcCreatePending {
    pub(crate) token: CcCreateToken,
    pub(crate) poll_after_secs: u32,
}

#[derive(Debug)]
#[doc(hidden)]
pub struct UndeterminedCreate(ProviderError);

impl UndeterminedCreate {
    pub(crate) fn new(error: ProviderError) -> Self {
        Self(error)
    }

    pub fn into_error(self) -> ProviderError {
        self.0
    }

    pub(crate) fn map_error(self, map: impl FnOnce(ProviderError) -> ProviderError) -> Self {
        Self(map(self.0))
    }
}

#[derive(Debug)]
pub(crate) enum CcCreateStatus {
    Pending {
        token: InFlightCreateRequest,
        poll_after_secs: u32,
    },
    Success {
        token: InFlightCreateRequest,
        identifier: Option<String>,
    },
    Failed {
        failure: TerminallyFailedCreate,
        identifier: Option<String>,
        error_code: Option<HandlerErrorCode>,
        status_message: String,
    },
    RequestTokenNotFound {
        token: InFlightCreateRequest,
        error: ProviderError,
    },
    StatusError {
        token: InFlightCreateRequest,
        error: ProviderError,
        poll_after_secs: u32,
    },
}

#[derive(Debug, Clone, Copy)]
struct RetryPolicy {
    action: &'static str,
    unknown_outcome_context: &'static str,
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
enum NotIssuedDecision<NotIssued, Unconfirmed> {
    RetryRejected {
        pending: NotIssued,
        delay_secs: u32,
    },
    Unconfirmed {
        pending: Unconfirmed,
        delay_secs: u32,
    },
    Fail {
        not_issued: NotIssued,
    },
}

#[derive(Debug)]
enum UnconfirmedDecision<Unconfirmed> {
    Resend {
        pending: Unconfirmed,
        delay_secs: u32,
    },
    Fail,
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

    /// Send a create that is proven not to have been issued.
    pub(crate) async fn send_not_issued_create(
        &self,
        type_name: &str,
        desired_state: serde_json::Value,
        operation_config: Option<&carina_core::schema::OperationConfig>,
        token: NotIssuedCreate,
    ) -> Result<CcCreatePending, NotAppliedCreate> {
        let retry_policy = Self::request_retry_policy(
            operation_config,
            "creating",
            "Failed to resend create resource; the outcome of the earlier send is unknown",
        );
        self.issue_not_issued_create(type_name, desired_state, token, retry_policy)
            .await
    }

    /// Resend a create whose previous send may have been accepted.
    pub(crate) async fn send_unconfirmed_create(
        &self,
        type_name: &str,
        desired_state: serde_json::Value,
        operation_config: Option<&carina_core::schema::OperationConfig>,
        token: &UnconfirmedCreate,
    ) -> Result<CcCreatePending, UndeterminedCreate> {
        let retry_policy = Self::request_retry_policy(
            operation_config,
            "creating",
            "Failed to resend create resource; the outcome of the earlier send is unknown",
        );
        self.issue_unconfirmed_create(type_name, desired_state, token, retry_policy)
            .await
            .map_err(UndeterminedCreate::new)
    }

    // Contract checked against aws-sdk-cloudcontrol 1.103.0 on 2026-10-28 for
    // every create/update/delete ClientToken member: "A unique identifier to
    // ensure the idempotency of the resource request. As a best practice,
    // specify this token to ensure idempotency, so that Amazon Web Services
    // Cloud Control API can accurately distinguish between request retries and
    // new resource requests. You might retry a resource request to ensure that
    // it was successfully received." Its ClientTokenConflictException docs
    // add, "However, client token expire after 36 hours." Within that documented
    // window, resending any CloudControl mutation with the same ClientToken
    // cannot apply it twice.
    async fn send_create_request(
        &self,
        type_name: &str,
        desired_state: &serde_json::Value,
        client_token: &ClientToken,
    ) -> Result<CreateResourceOutput, Box<SdkError<CreateResourceError, HttpResponse>>> {
        self.cloudcontrol_client
            .create_resource()
            .type_name(type_name)
            .desired_state(desired_state.to_string())
            .client_token(client_token.as_str())
            .send()
            .await
            .map_err(Box::new)
    }

    async fn issue_not_issued_create(
        &self,
        type_name: &str,
        desired_state: serde_json::Value,
        token: NotIssuedCreate,
        retry_policy: RetryPolicy,
    ) -> Result<CcCreatePending, NotAppliedCreate> {
        let result = self
            .send_create_request(type_name, &desired_state, token.client_token())
            .await;

        match result {
            Ok(response) => {
                let progress = response.progress_event();
                let request = progress
                    .and_then(ProgressEvent::request_token)
                    .filter(|request| !request.is_empty())
                    .map(RequestToken::from);
                let identifier = progress
                    .and_then(ProgressEvent::identifier)
                    .filter(|identifier| !identifier.is_empty())
                    .map(ToString::to_string);
                match request {
                    Some(request) => Ok(CcCreatePending {
                        token: token.into_in_flight(request, identifier).into(),
                        poll_after_secs: FIRST_POLL_AFTER_SECS,
                    }),
                    None => {
                        log::warn!(
                            "Cloud Control accepted create {type_name} without returning a request token; resending with the same client token"
                        );
                        Ok(CcCreatePending {
                            token: token.into_unconfirmed(1).into(),
                            poll_after_secs: FIRST_POLL_AFTER_SECS,
                        })
                    }
                }
            }
            Err(error) => {
                match Self::not_issued_retry_decision(
                    error.as_ref(),
                    type_name,
                    token,
                    retry_policy,
                ) {
                    NotIssuedDecision::RetryRejected {
                        pending,
                        delay_secs,
                    } => Ok(CcCreatePending {
                        token: pending.into(),
                        poll_after_secs: delay_secs,
                    }),
                    NotIssuedDecision::Unconfirmed {
                        pending,
                        delay_secs,
                    } => Ok(CcCreatePending {
                        token: pending.into(),
                        poll_after_secs: delay_secs,
                    }),
                    NotIssuedDecision::Fail { not_issued } => {
                        Err(not_issued.into_not_applied(api_error_with_meta_and_subject(
                            "Failed to create resource",
                            "cloudcontrol.CreateResource",
                            CloudControlResourceSubject::type_only(type_name),
                            *error,
                        )))
                    }
                }
            }
        }
    }

    async fn issue_unconfirmed_create(
        &self,
        type_name: &str,
        desired_state: serde_json::Value,
        token: &UnconfirmedCreate,
        retry_policy: RetryPolicy,
    ) -> ProviderResult<CcCreatePending> {
        let result = self
            .send_create_request(type_name, &desired_state, token.client_token())
            .await;

        match result {
            Ok(response) => {
                let progress = response.progress_event();
                let request = progress
                    .and_then(ProgressEvent::request_token)
                    .filter(|request| !request.is_empty())
                    .map(RequestToken::from);
                let identifier = progress
                    .and_then(ProgressEvent::identifier)
                    .filter(|identifier| !identifier.is_empty())
                    .map(ToString::to_string);
                match request {
                    Some(request) => Ok(CcCreatePending {
                        token: token.to_in_flight(request, identifier).into(),
                        poll_after_secs: FIRST_POLL_AFTER_SECS,
                    }),
                    None => {
                        let delay_secs = retry_policy.delay_secs(token.resends());
                        let resends = token.resends().saturating_add(1);
                        log::warn!(
                            "Cloud Control accepted create {type_name} without returning a request token; resending with the same client token in {delay_secs}s"
                        );
                        Ok(CcCreatePending {
                            token: token.resend(resends).into(),
                            poll_after_secs: delay_secs,
                        })
                    }
                }
            }
            Err(error) => {
                match Self::unconfirmed_retry_decision(
                    error.as_ref(),
                    type_name,
                    token,
                    retry_policy,
                ) {
                    UnconfirmedDecision::Resend {
                        pending,
                        delay_secs,
                    } => Ok(CcCreatePending {
                        token: pending.into(),
                        poll_after_secs: delay_secs,
                    }),
                    UnconfirmedDecision::Fail => Err(api_error_with_meta_and_subject(
                        retry_policy.unknown_outcome_context,
                        "cloudcontrol.CreateResource",
                        CloudControlResourceSubject::type_only(type_name),
                        *error,
                    )),
                }
            }
        }
    }

    /// Fetch and classify one status for an accepted create request.
    pub(crate) async fn cc_create_request_status(
        &self,
        token: InFlightCreateRequest,
    ) -> CcCreateStatus {
        let request = token.request().clone();
        let recorded_identifier = token.identifier().map(ToString::to_string);

        let result = self
            .cloudcontrol_client
            .get_resource_request_status()
            .request_token(request.as_str())
            .send()
            .await;
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
            return CcCreateStatus::Pending {
                token,
                poll_after_secs: POLL_AFTER_SECS,
            };
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
            | Some(OperationStatus::CancelInProgress) => CcCreateStatus::Pending {
                token,
                poll_after_secs: POLL_AFTER_SECS,
            },
            Some(OperationStatus::Success) => CcCreateStatus::Success { token, identifier },
            Some(status @ (OperationStatus::Failed | OperationStatus::CancelComplete)) => {
                let failure = TerminallyFailedCreate::from_status(token, status.clone())
                    .expect("matched terminal create status constructs a failure witness");
                CcCreateStatus::Failed {
                    failure,
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
                CcCreateStatus::Pending {
                    token,
                    poll_after_secs: POLL_AFTER_SECS,
                }
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
        self.cc_poll_update_step(
            type_name,
            identifier,
            patch_ops,
            operation_config,
            CcRequestPhase::start(),
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
        let retry_policy = Self::request_retry_policy(
            operation_config,
            "updating",
            "Failed to resend update resource; the outcome of the earlier send is unknown",
        );
        let step = match token {
            CcRequestPhase::NotIssued(token) => {
                self.issue_not_issued_update(type_name, identifier, patch_ops, token, retry_policy)
                    .await?
            }
            CcRequestPhase::Unconfirmed(token) => {
                self.issue_unconfirmed_update(type_name, identifier, patch_ops, token, retry_policy)
                    .await?
            }
            CcRequestPhase::InFlight(token) => {
                self.poll_in_flight_phase(type_name, token, retry_policy)
                    .await?
            }
        };
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

    async fn send_update_request(
        &self,
        type_name: &str,
        identifier: &str,
        patch_ops: Vec<serde_json::Value>,
        client_token: &ClientToken,
    ) -> ProviderResult<
        Result<Option<RequestToken>, Box<SdkError<UpdateResourceError, HttpResponse>>>,
    > {
        let patch_document = serde_json::to_string(&patch_ops)
            .map_err(|e| ProviderError::internal_caused_by("Failed to build patch", e))?;

        Ok(self
            .cloudcontrol_client
            .update_resource()
            .type_name(type_name)
            .identifier(identifier)
            .patch_document(patch_document)
            .client_token(client_token.as_str())
            .send()
            .await
            .map(|response| {
                response
                    .progress_event()
                    .and_then(|progress| progress.request_token())
                    .map(RequestToken::from)
            })
            .map_err(Box::new))
    }

    async fn issue_not_issued_update(
        &self,
        type_name: &str,
        identifier: &str,
        patch_ops: Vec<serde_json::Value>,
        token: NotIssuedRequest,
        retry_policy: RetryPolicy,
    ) -> ProviderResult<CcUpdateWaitStep> {
        let result = self
            .send_update_request(type_name, identifier, patch_ops, token.client_token())
            .await?;
        Self::finish_not_issued(
            result,
            type_name,
            token,
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

    async fn issue_unconfirmed_update(
        &self,
        type_name: &str,
        identifier: &str,
        patch_ops: Vec<serde_json::Value>,
        token: UnconfirmedRequest,
        retry_policy: RetryPolicy,
    ) -> ProviderResult<CcUpdateWaitStep> {
        let result = self
            .send_update_request(type_name, identifier, patch_ops, token.client_token())
            .await?;
        Self::finish_unconfirmed(
            result,
            type_name,
            token,
            retry_policy,
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
        self.cc_poll_delete_step(
            type_name,
            identifier,
            operation_config,
            CcRequestPhase::start(),
        )
        .await
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
        let step = match phase {
            CcRequestPhase::NotIssued(token) => {
                self.issue_not_issued_delete(type_name, identifier, token, retry_policy)
                    .await?
            }
            CcRequestPhase::Unconfirmed(token) => {
                self.issue_unconfirmed_delete(type_name, identifier, token, retry_policy)
                    .await?
            }
            CcRequestPhase::InFlight(token) => {
                self.poll_in_flight_phase(type_name, token, retry_policy)
                    .await?
            }
        };
        Self::finish_delete_step(type_name, step)
    }

    async fn send_delete_request(
        &self,
        type_name: &str,
        identifier: &str,
        client_token: &ClientToken,
    ) -> Result<Option<RequestToken>, Box<SdkError<DeleteResourceError, HttpResponse>>> {
        self.cloudcontrol_client
            .delete_resource()
            .type_name(type_name)
            .identifier(identifier)
            .client_token(client_token.as_str())
            .send()
            .await
            .map(|response| {
                response
                    .progress_event()
                    .and_then(|progress| progress.request_token())
                    .map(RequestToken::from)
            })
            .map_err(Box::new)
    }

    async fn issue_not_issued_delete(
        &self,
        type_name: &str,
        identifier: &str,
        token: NotIssuedRequest,
        retry_policy: RetryPolicy,
    ) -> ProviderResult<CcWaitStep> {
        let result = self
            .send_delete_request(type_name, identifier, token.client_token())
            .await;
        Self::finish_not_issued(
            result,
            type_name,
            token,
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

    async fn issue_unconfirmed_delete(
        &self,
        type_name: &str,
        identifier: &str,
        token: UnconfirmedRequest,
        retry_policy: RetryPolicy,
    ) -> ProviderResult<CcWaitStep> {
        let result = self
            .send_delete_request(type_name, identifier, token.client_token())
            .await;
        Self::finish_unconfirmed(
            result,
            type_name,
            token,
            retry_policy,
            "cloudcontrol.DeleteResource",
            CloudControlResourceSubject::identified(type_name, identifier),
            || {
                Ok(WaitOutcome::Success {
                    identifier: identifier.to_string(),
                })
            },
        )
    }

    async fn poll_in_flight_phase(
        &self,
        type_name: &str,
        token: InFlightRequest,
        retry_policy: RetryPolicy,
    ) -> ProviderResult<CcWaitStep> {
        match self.poll_request_status(token).await? {
            RequestStatusStep::Pending(token) => Ok(OperationStep::Pending {
                pending: token.into(),
                poll_after_secs: POLL_AFTER_SECS,
            }),
            RequestStatusStep::Done(outcome) => Ok(OperationStep::Done(outcome)),
            RequestStatusStep::RetryableFailure { failure, error } => {
                let attempt = failure.attempt();
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
                    pending: failure.retry_after_terminal_failure().into(),
                    poll_after_secs: delay_secs,
                })
            }
        }
    }

    /// Decides whether and how to resend one failed mutating request.
    ///
    /// A `MaybeSent` outcome is never capped by `max_attempts`: create continuations are
    /// persisted as an in-flight create when the host drive budget expires, while update
    /// and delete remain bounded by that drive budget and are reconciled on the next plan.
    /// The rejection `attempt` value advances only before any send may have been accepted.
    /// Once unconfirmed, `resends` advances independently to provide capped backoff without
    /// consuming a later terminal-status rejection budget.
    fn not_issued_retry_decision<E, Phase>(
        error: &SdkError<E, HttpResponse>,
        type_name: &str,
        pending: Phase,
        retry_policy: RetryPolicy,
    ) -> NotIssuedDecision<Phase, Phase::Unconfirmed>
    where
        E: std::fmt::Debug + ProvideErrorKind + ProvideErrorMetadata,
        Phase: NotIssuedPhase,
    {
        let attempt = pending.attempt();
        match SendOutcome::classify(error) {
            SendOutcome::MaybeSent => {
                let delay_secs = retry_policy.delay_secs(0);
                log::warn!(
                    "Outcome unknown {} {type_name} after 1 send(s): {error:?}. Retrying with the same client token in {delay_secs}s...",
                    retry_policy.action,
                );
                NotIssuedDecision::Unconfirmed {
                    pending: pending.into_unconfirmed(1),
                    delay_secs,
                }
            }
            SendOutcome::Rejected
                if Self::is_retryable_rejection(error) && attempt < retry_policy.max_attempts =>
            {
                let delay_secs = retry_policy.delay_secs(attempt);
                log::warn!(
                    "Retryable rejection {} {type_name} (attempt {}/{}): {error:?}. Retrying in {delay_secs}s...",
                    retry_policy.action,
                    attempt + 1,
                    retry_policy.max_attempts,
                );
                NotIssuedDecision::RetryRejected {
                    pending: pending.retry_rejected(),
                    delay_secs,
                }
            }
            SendOutcome::NotSent | SendOutcome::Rejected => NotIssuedDecision::Fail {
                not_issued: pending,
            },
        }
    }

    fn unconfirmed_retry_decision<E, Phase>(
        error: &SdkError<E, HttpResponse>,
        type_name: &str,
        pending: &Phase,
        retry_policy: RetryPolicy,
    ) -> UnconfirmedDecision<Phase>
    where
        E: std::fmt::Debug + ProvideErrorKind + ProvideErrorMetadata,
        Phase: UnconfirmedPhase,
    {
        let resends = pending.resends();
        match SendOutcome::classify(error) {
            SendOutcome::MaybeSent => {
                let delay_secs = retry_policy.delay_secs(resends);
                let resends = resends.saturating_add(1);
                log::warn!(
                    "Outcome unknown {} {type_name} after {resends} send(s): {error:?}. Retrying with the same client token in {delay_secs}s...",
                    retry_policy.action,
                );
                UnconfirmedDecision::Resend {
                    pending: pending.resend(resends),
                    delay_secs,
                }
            }
            SendOutcome::Rejected if Self::is_retryable_rejection(error) => {
                let delay_secs = retry_policy.delay_secs(resends);
                let resends = resends.saturating_add(1);
                log::warn!(
                    "Retryable rejection resending unconfirmed {} {type_name} (resend {resends}): {error:?}. Retrying with the same client token in {delay_secs}s...",
                    retry_policy.action,
                );
                UnconfirmedDecision::Resend {
                    pending: pending.resend(resends),
                    delay_secs,
                }
            }
            SendOutcome::NotSent | SendOutcome::Rejected => UnconfirmedDecision::Fail,
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn finish_not_issued<E, Subject, NoRequest>(
        result: Result<Option<RequestToken>, Box<SdkError<E, HttpResponse>>>,
        type_name: &str,
        token: NotIssuedRequest,
        retry_policy: RetryPolicy,
        context: &'static str,
        operation: &'static str,
        subject: Subject,
        no_request: NoRequest,
    ) -> ProviderResult<CcWaitStep>
    where
        E: std::error::Error + ProvideErrorKind + ProvideErrorMetadata + Send + Sync + 'static,
        Subject: std::fmt::Debug + std::fmt::Display + Send + Sync + 'static,
        NoRequest: FnOnce() -> ProviderResult<WaitOutcome>,
    {
        match result {
            Ok(Some(request)) => Ok(OperationStep::Pending {
                pending: token.into_in_flight(request).into(),
                poll_after_secs: FIRST_POLL_AFTER_SECS,
            }),
            Ok(None) => no_request().map(OperationStep::Done),
            Err(error) => {
                match Self::not_issued_retry_decision(
                    error.as_ref(),
                    type_name,
                    token,
                    retry_policy,
                ) {
                    NotIssuedDecision::RetryRejected {
                        pending,
                        delay_secs,
                    } => Ok(OperationStep::Pending {
                        pending: pending.into(),
                        poll_after_secs: delay_secs,
                    }),
                    NotIssuedDecision::Unconfirmed {
                        pending,
                        delay_secs,
                    } => Ok(OperationStep::Pending {
                        pending: pending.into(),
                        poll_after_secs: delay_secs,
                    }),
                    NotIssuedDecision::Fail { .. } => Err(api_error_with_meta_and_subject(
                        context, operation, subject, *error,
                    )),
                }
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn finish_unconfirmed<E, Subject, NoRequest>(
        result: Result<Option<RequestToken>, Box<SdkError<E, HttpResponse>>>,
        type_name: &str,
        token: UnconfirmedRequest,
        retry_policy: RetryPolicy,
        operation: &'static str,
        subject: Subject,
        no_request: NoRequest,
    ) -> ProviderResult<CcWaitStep>
    where
        E: std::error::Error + ProvideErrorKind + ProvideErrorMetadata + Send + Sync + 'static,
        Subject: std::fmt::Debug + std::fmt::Display + Send + Sync + 'static,
        NoRequest: FnOnce() -> ProviderResult<WaitOutcome>,
    {
        match result {
            Ok(Some(request)) => Ok(OperationStep::Pending {
                pending: token.into_in_flight(request).into(),
                poll_after_secs: FIRST_POLL_AFTER_SECS,
            }),
            Ok(None) => no_request().map(OperationStep::Done),
            Err(error) => {
                match Self::unconfirmed_retry_decision(
                    error.as_ref(),
                    type_name,
                    &token,
                    retry_policy,
                ) {
                    UnconfirmedDecision::Resend {
                        pending,
                        delay_secs,
                    } => Ok(OperationStep::Pending {
                        pending: pending.into(),
                        poll_after_secs: delay_secs,
                    }),
                    UnconfirmedDecision::Fail => Err(api_error_with_meta_and_subject(
                        retry_policy.unknown_outcome_context,
                        operation,
                        subject,
                        *error,
                    )),
                }
            }
        }
    }

    fn finish_delete_step(type_name: &str, step: CcWaitStep) -> ProviderResult<CcDeleteWaitStep> {
        match step {
            OperationStep::Done(WaitOutcome::Success { .. }) => Ok(OperationStep::Done(())),
            OperationStep::Done(WaitOutcome::PartialOrFailed { status_message, .. }) => {
                Err(ProviderError::api_error_caused_by(
                    "Operation failed",
                    CloudControlOperationFailure::for_delete(type_name, status_message),
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
        unknown_outcome_context: &'static str,
    ) -> RetryPolicy {
        RetryPolicy {
            action,
            unknown_outcome_context,
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
            unknown_outcome_context: "Failed to resend delete resource; the outcome of the earlier send is unknown",
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

    /// Returns whether CloudControl's proven rejection is worth retrying.
    ///
    /// Entries that CloudControl normally returns with a 5xx status (for
    /// example, `ServiceInternalErrorException`) are classified as
    /// `SendOutcome::MaybeSent` before this proven-rejection list is consulted.
    /// Keep them here as a defensive classification for model- or
    /// transport-dependent responses that reach this function as rejections.
    fn is_retryable_rejection<E, R>(error: &SdkError<E, R>) -> bool
    where
        E: ProvideErrorMetadata,
    {
        /// Error codes from the CloudControl API that indicate retryable conditions.
        const RETRYABLE_ERROR_CODES: &[&str] = &[
            "ThrottlingException",
            "ClientTokenConflictException",
            "ServiceInternalErrorException",
            "HandlerFailureException",
            "HandlerInternalFailureException",
            "NetworkFailureException",
            "ConcurrentOperationException",
            "NotStabilizedException",
        ];

        /// Error message patterns that indicate retryable conditions regardless of error code.
        /// These are checked against the error message when the error code alone is not
        /// sufficient to determine retryability.
        const RETRYABLE_MESSAGE_PATTERNS: &[&str] = &["non-deleted VPC Attachments"];

        match error {
            SdkError::ServiceError(service_error) => {
                let err = service_error.err();
                err.code()
                    .is_some_and(|code| RETRYABLE_ERROR_CODES.contains(&code))
                    || err.message().is_some_and(|message| {
                        RETRYABLE_MESSAGE_PATTERNS
                            .iter()
                            .any(|pattern| message.contains(pattern))
                    })
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
        token: InFlightRequest,
    ) -> ProviderResult<RequestStatusStep> {
        let request = token.request().clone();
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
            return Ok(RequestStatusStep::Pending(token));
        };
        match Self::classify_progress_event(progress) {
            Ok(Some(outcome)) => Ok(RequestStatusStep::Done(outcome)),
            Ok(None) => Ok(RequestStatusStep::Pending(token)),
            Err(error)
                if matches!(
                    progress.operation_status(),
                    Some(OperationStatus::Failed | OperationStatus::CancelComplete)
                ) && progress
                    .status_message()
                    .is_some_and(Self::is_retryable_status_message) =>
            {
                let observed_status = progress
                    .operation_status()
                    .cloned()
                    .expect("retryable terminal branch has an operation status");
                let failure = TerminallyFailedRequest::from_status(token, observed_status)
                    .expect("matched terminal request status constructs a failure witness");
                Ok(RequestStatusStep::RetryableFailure { failure, error })
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
