//! High-level resource operations (read, create, update, delete).
//!
//! This module implements the main resource lifecycle operations that bridge
//! between DSL resources and the Cloud Control API. It handles attribute mapping,
//! tags, special cases, and default values.

use std::collections::HashMap;
use std::future::Future;
use std::time::Duration;

use aws_sdk_cloudcontrol::types::{HandlerErrorCode, OperationStatus};
use carina_core::provider::{
    AppliedIdentifier, AppliedState, CreateAttempt, CreateFailure, CreateOutcome, DeleteRequest,
    IssuedCreate, Observation, PartialReadDiagnostic, PendingCreate, PendingToken, Poll,
    PollAttempt, ProviderError, ProviderResult, ReadOutcome, ResentCreateAttempt,
    ResentIssuedCreate, ResentPendingCreate, ResentPoll, ResentPollAttempt, Resume, ResumeAttempt,
    ResumeCreateRequest, ResumeFailure, ResumeOutcome, ResumedCreate, Sent, UpdateOutcome,
    UpdatePatch,
};
use carina_core::resource::{ConcreteValue, Directives, Resource, ResourceId, State, Value};
use carina_core::schema::{AttributeSchema, AttributeType, Schema, Shape};
use carina_provider_protocol::{
    DEFAULT_OPERATION_BUDGET, DriveError, DriveOutcome, OperationKind, OperationStep, drive_pending,
};
use indexmap::IndexMap;

use super::conversion::{aws_value_to_dsl_with_defs, dsl_value_to_aws_with_defs};
use super::tags::wire_tags_are_empty;
use super::update::{UpdatePatchPlan, build_update_patches};
use super::{
    AwsccProvider, CcCreateToken, CcDeleteToken, CcRequestPhase, CcUpdateToken, get_schema_config,
};
use crate::provider::arn_synthesis::SynthesisStatus;
use crate::provider::cloudcontrol::{CcCreatePending, CcCreateStatus, WaitOutcome};
use crate::provider::special_cases::internet_gateway_detach_patch;
use crate::schemas::config::AwsccSchemaConfig;

pub type CcUpdateStep = OperationStep<UpdateOutcome, ProviderError, CcUpdateToken>;
pub type CcDeleteStep = OperationStep<(), ProviderError, CcDeleteToken>;

const POLL_AFTER_SECS: u32 = 5;

struct CreateTerminalFailure {
    token: CcCreateToken,
    status: OperationStatus,
    identifier: Option<String>,
    error_code: Option<HandlerErrorCode>,
    status_message: String,
}

async fn drive_operation<T, Token, Initial, Poll, PollFuture, Sleep, SleepFuture>(
    id: &ResourceId,
    kind: OperationKind,
    budget: Duration,
    initial: Initial,
    poll: Poll,
    sleep: Sleep,
) -> ProviderResult<T>
where
    Token: Clone,
    Initial: Future<Output = ProviderResult<OperationStep<T, ProviderError, Token>>>,
    Poll: FnMut(Token) -> PollFuture,
    PollFuture: Future<Output = ProviderResult<OperationStep<T, ProviderError, Token>>>,
    Sleep: FnMut(Duration) -> SleepFuture,
    SleepFuture: Future<Output = ()>,
{
    match drive_pending(budget, initial, poll, sleep).await {
        Ok(DriveOutcome::Done(value)) => Ok(value),
        Ok(DriveOutcome::Verdict(error)) => Err(error),
        Ok(DriveOutcome::OverBudget {
            last_pending: _last_pending,
        }) => {
            let operation = match kind {
                OperationKind::Create => "create",
                OperationKind::Update => "update",
                OperationKind::Delete => "delete",
            };
            Err(ProviderError::timeout(format!(
                "{operation} of {} did not finish within {budget:?}",
                id.resource_type
            ))
            .for_resource(id.clone()))
        }
        Err(DriveError::Initial(error)) | Err(DriveError::Poll { error, .. }) => Err(error),
    }
}

/// Create typestate adapter shared by the native provider and WASM guest.
#[doc(hidden)]
pub trait CcIssuedCreate {
    type Step;
    type Verdict;

    fn identifier(&self) -> &AppliedIdentifier;
    fn succeeded(
        self,
        identifier: AppliedIdentifier,
        attributes: HashMap<String, Value>,
    ) -> Self::Step;
    fn partial(
        self,
        identifier: AppliedIdentifier,
        attributes: HashMap<String, Value>,
        reason: String,
        missing_attributes: Vec<String>,
    ) -> Self::Step;
    fn failed(
        self,
        identifier: AppliedIdentifier,
        observation: Observation,
        error: ProviderError,
    ) -> Self::Verdict;
    fn discarded(self, error: ProviderError) -> Self::Verdict;
    fn pending(
        self,
        token: CcCreateToken,
        poll_after_secs: u32,
    ) -> Result<Self::Step, Self::Verdict>;
}

#[doc(hidden)]
pub trait CcCreateAttempt {
    type Step;
    type Verdict;
    type Issued: CcIssuedCreate<Step = Self::Step, Verdict = Self::Verdict>;

    fn untracked(self, error: ProviderError) -> Self::Verdict;
    fn issued(self, identifier: AppliedIdentifier, sent: Sent) -> Self::Issued;
    fn pending(
        self,
        token: CcCreateToken,
        sent: Sent,
        poll_after_secs: u32,
    ) -> Result<Self::Step, Self::Verdict>;
}

#[doc(hidden)]
pub trait CcPollAttempt {
    type Step;
    type Verdict;
    type Issued: CcIssuedCreate<Step = Self::Step, Verdict = Self::Verdict>;

    fn untracked(self, error: ProviderError) -> Self::Verdict;
    fn issued(self, identifier: AppliedIdentifier) -> Self::Issued;
    fn pending(
        self,
        token: CcCreateToken,
        poll_after_secs: u32,
    ) -> Result<Self::Step, Self::Verdict>;
}

#[doc(hidden)]
pub enum CcCreatePoll<U, I> {
    Unissued(U),
    Issued(I),
}

#[doc(hidden)]
pub trait CcResumedCreate {
    type Step;
    type Verdict;

    fn identifier(&self) -> &AppliedIdentifier;
    fn completed(self, identifier: AppliedIdentifier, state: AppliedState) -> Self::Step;
    fn failed(
        self,
        identifier: AppliedIdentifier,
        observation: Observation,
        error: ProviderError,
    ) -> Self::Verdict;
    fn discarded(self, error: ProviderError) -> Self::Verdict;
    fn pending(
        self,
        token: CcCreateToken,
        poll_after_secs: u32,
    ) -> Result<Self::Step, Self::Verdict>;
}

#[doc(hidden)]
pub trait CcResumeAttempt {
    type Step;
    type Verdict;
    type Resumed: CcResumedCreate<Step = Self::Step, Verdict = Self::Verdict>;
    type Resent: CcCreateAttempt<Step = Self::Step, Verdict = Self::Verdict>;

    fn untracked(self, error: ProviderError) -> Self::Verdict;
    fn undetermined(self, error: ProviderError) -> Self::Verdict;
    fn issued(self, identifier: AppliedIdentifier) -> Self::Resumed;
    fn pending(
        self,
        token: CcCreateToken,
        poll_after_secs: u32,
    ) -> Result<Self::Step, Self::Verdict>;
    fn resend(self) -> Self::Resent;
}

#[doc(hidden)]
pub enum CcCreateResume<U, I> {
    Unissued(U),
    Issued(I),
}

#[derive(Debug)]
#[doc(hidden)]
pub enum CoreCreateStep {
    Done(CreateOutcome),
    Pending(PendingCreate),
}

#[derive(Debug)]
#[doc(hidden)]
pub enum CoreResumeStep {
    Done(ResumeOutcome),
    Pending(Box<Resume>),
    ResentPending(ResentPendingCreate),
}

fn encode_create_token(token: &CcCreateToken) -> Result<PendingToken, String> {
    serde_json::to_string(token)
        .map(PendingToken::new)
        .map_err(|error| format!("failed to encode Cloud Control create continuation: {error}"))
}

fn decode_create_token(token: &PendingToken) -> ProviderResult<CcCreateToken> {
    serde_json::from_str(token.expose_for_provider_call()).map_err(|error| {
        ProviderError::internal_caused_by(
            "failed to decode Cloud Control create continuation",
            error,
        )
    })
}

fn diagnostic(reason: String, missing_attributes: Vec<String>) -> Option<PartialReadDiagnostic> {
    PartialReadDiagnostic::new(reason, missing_attributes)
}

impl CcCreateAttempt for CreateAttempt {
    type Step = CoreCreateStep;
    type Verdict = CreateFailure;
    type Issued = IssuedCreate;

    fn untracked(self, error: ProviderError) -> Self::Verdict {
        self.untracked(error)
    }

    fn issued(self, identifier: AppliedIdentifier, sent: Sent) -> Self::Issued {
        self.issued(identifier, sent)
    }

    fn pending(
        self,
        token: CcCreateToken,
        sent: Sent,
        poll_after_secs: u32,
    ) -> Result<Self::Step, Self::Verdict> {
        match encode_create_token(&token) {
            Ok(token) => Ok(CoreCreateStep::Pending(self.pending(
                token,
                sent,
                poll_after_secs,
            ))),
            Err(message) => Err(self.untracked(ProviderError::internal(message))),
        }
    }
}

impl CcPollAttempt for PollAttempt {
    type Step = CoreCreateStep;
    type Verdict = CreateFailure;
    type Issued = IssuedCreate;

    fn untracked(self, error: ProviderError) -> Self::Verdict {
        self.untracked(error)
    }

    fn issued(self, identifier: AppliedIdentifier) -> Self::Issued {
        self.issued(identifier)
    }

    fn pending(
        self,
        token: CcCreateToken,
        poll_after_secs: u32,
    ) -> Result<Self::Step, Self::Verdict> {
        match encode_create_token(&token) {
            Ok(token) => Ok(CoreCreateStep::Pending(
                self.pending(token, poll_after_secs),
            )),
            Err(message) => Err(self.untracked(ProviderError::internal(message))),
        }
    }
}

impl CcIssuedCreate for IssuedCreate {
    type Step = CoreCreateStep;
    type Verdict = CreateFailure;

    fn identifier(&self) -> &AppliedIdentifier {
        self.identifier()
    }

    fn succeeded(
        self,
        identifier: AppliedIdentifier,
        attributes: HashMap<String, Value>,
    ) -> Self::Step {
        CoreCreateStep::Done(self.canonicalize(identifier).succeeded(attributes))
    }

    fn partial(
        self,
        identifier: AppliedIdentifier,
        attributes: HashMap<String, Value>,
        reason: String,
        missing_attributes: Vec<String>,
    ) -> Self::Step {
        let finishing = self.canonicalize(identifier);
        match diagnostic(reason, missing_attributes) {
            Some(diagnostic) => CoreCreateStep::Done(finishing.partial(attributes, diagnostic)),
            None => CoreCreateStep::Done(finishing.succeeded(attributes)),
        }
    }

    fn failed(
        self,
        identifier: AppliedIdentifier,
        observation: Observation,
        error: ProviderError,
    ) -> Self::Verdict {
        self.canonicalize(identifier).failed(observation, error)
    }

    fn discarded(self, error: ProviderError) -> Self::Verdict {
        self.discarded(error)
    }

    fn pending(
        self,
        token: CcCreateToken,
        poll_after_secs: u32,
    ) -> Result<Self::Step, Self::Verdict> {
        match encode_create_token(&token) {
            Ok(token) => Ok(CoreCreateStep::Pending(
                self.pending(token, poll_after_secs),
            )),
            Err(message) => {
                let observation_error = ProviderError::internal(message.clone());
                Err(self.failed(
                    Observation::ReadFailed(observation_error),
                    ProviderError::internal(message),
                ))
            }
        }
    }
}

impl CcResumeAttempt for ResumeAttempt {
    type Step = CoreResumeStep;
    type Verdict = ResumeFailure;
    type Resumed = ResumedCreate;
    type Resent = ResentCreateAttempt;

    fn untracked(self, error: ProviderError) -> Self::Verdict {
        self.untracked(error)
    }

    fn undetermined(self, error: ProviderError) -> Self::Verdict {
        self.preserve(error)
    }

    fn issued(self, identifier: AppliedIdentifier) -> Self::Resumed {
        self.issued(identifier)
    }

    fn pending(
        self,
        token: CcCreateToken,
        _poll_after_secs: u32,
    ) -> Result<Self::Step, Self::Verdict> {
        match encode_create_token(&token) {
            Ok(token) => Ok(CoreResumeStep::Pending(Box::new(self.repend(token)))),
            Err(message) => Err(self.preserve(ProviderError::internal(message))),
        }
    }

    fn resend(self) -> Self::Resent {
        self.resend()
    }
}

impl CcResumedCreate for ResumedCreate {
    type Step = CoreResumeStep;
    type Verdict = ResumeFailure;

    fn identifier(&self) -> &AppliedIdentifier {
        self.identifier()
    }

    fn completed(self, identifier: AppliedIdentifier, state: AppliedState) -> Self::Step {
        CoreResumeStep::Done(self.canonicalize(identifier).completed(state))
    }

    fn failed(
        self,
        identifier: AppliedIdentifier,
        observation: Observation,
        error: ProviderError,
    ) -> Self::Verdict {
        self.canonicalize(identifier).failed(observation, error)
    }

    fn discarded(self, error: ProviderError) -> Self::Verdict {
        self.discarded(error)
    }

    fn pending(
        self,
        token: CcCreateToken,
        _poll_after_secs: u32,
    ) -> Result<Self::Step, Self::Verdict> {
        match encode_create_token(&token) {
            Ok(token) => Ok(CoreResumeStep::Pending(Box::new(self.repend(token)))),
            Err(message) => Err(self.preserve(ProviderError::internal(message))),
        }
    }
}

impl CcCreateAttempt for ResentCreateAttempt {
    type Step = CoreResumeStep;
    type Verdict = ResumeFailure;
    type Issued = ResentIssuedCreate;

    fn untracked(self, error: ProviderError) -> Self::Verdict {
        self.untracked(error)
    }

    fn issued(self, identifier: AppliedIdentifier, sent: Sent) -> Self::Issued {
        self.issued(identifier, sent)
    }

    fn pending(
        self,
        token: CcCreateToken,
        sent: Sent,
        poll_after_secs: u32,
    ) -> Result<Self::Step, Self::Verdict> {
        match encode_create_token(&token) {
            Ok(token) => Ok(CoreResumeStep::ResentPending(self.pending(
                token,
                sent,
                poll_after_secs,
            ))),
            Err(message) => Err(self.untracked(ProviderError::internal(message))),
        }
    }
}

impl CcPollAttempt for ResentPollAttempt {
    type Step = CoreResumeStep;
    type Verdict = ResumeFailure;
    type Issued = ResentIssuedCreate;

    fn untracked(self, error: ProviderError) -> Self::Verdict {
        self.untracked(error)
    }

    fn issued(self, identifier: AppliedIdentifier) -> Self::Issued {
        self.issued(identifier)
    }

    fn pending(
        self,
        token: CcCreateToken,
        poll_after_secs: u32,
    ) -> Result<Self::Step, Self::Verdict> {
        match encode_create_token(&token) {
            Ok(token) => Ok(CoreResumeStep::ResentPending(
                self.pending(token, poll_after_secs),
            )),
            Err(message) => Err(self.untracked(ProviderError::internal(message))),
        }
    }
}

impl CcIssuedCreate for ResentIssuedCreate {
    type Step = CoreResumeStep;
    type Verdict = ResumeFailure;

    fn identifier(&self) -> &AppliedIdentifier {
        self.identifier()
    }

    fn succeeded(
        self,
        identifier: AppliedIdentifier,
        attributes: HashMap<String, Value>,
    ) -> Self::Step {
        CoreResumeStep::Done(self.canonicalize(identifier).succeeded(attributes))
    }

    fn partial(
        self,
        identifier: AppliedIdentifier,
        attributes: HashMap<String, Value>,
        reason: String,
        missing_attributes: Vec<String>,
    ) -> Self::Step {
        let finishing = self.canonicalize(identifier);
        match diagnostic(reason, missing_attributes) {
            Some(diagnostic) => CoreResumeStep::Done(finishing.partial(attributes, diagnostic)),
            None => CoreResumeStep::Done(finishing.succeeded(attributes)),
        }
    }

    fn failed(
        self,
        identifier: AppliedIdentifier,
        observation: Observation,
        error: ProviderError,
    ) -> Self::Verdict {
        self.canonicalize(identifier).failed(observation, error)
    }

    fn discarded(self, error: ProviderError) -> Self::Verdict {
        self.discarded(error)
    }

    fn pending(
        self,
        token: CcCreateToken,
        poll_after_secs: u32,
    ) -> Result<Self::Step, Self::Verdict> {
        match encode_create_token(&token) {
            Ok(token) => Ok(CoreResumeStep::ResentPending(
                self.pending(token, poll_after_secs),
            )),
            Err(message) => {
                let observation_error = ProviderError::internal(message.clone());
                Err(self.failed(
                    Observation::ReadFailed(observation_error),
                    ProviderError::internal(message),
                ))
            }
        }
    }
}

#[derive(Debug)]
enum CreateCompletion {
    Success {
        identifier: AppliedIdentifier,
        attributes: HashMap<String, Value>,
    },
    Partial {
        identifier: AppliedIdentifier,
        attributes: HashMap<String, Value>,
        reason: String,
        missing_attributes: Vec<String>,
    },
}

impl AwsccProvider {
    fn ensure_initialized(&self, id: &ResourceId) -> ProviderResult<()> {
        if let Some(err) = self.init_error() {
            return Err(ProviderError::invalid_input(err.to_string())
                .for_provider("awscc")
                .for_resource(id.clone()));
        }
        Ok(())
    }

    /// Read a resource using its configuration
    pub async fn read_resource(
        &self,
        id: &ResourceId,
        identifier: &AppliedIdentifier,
    ) -> ProviderResult<ReadOutcome> {
        self.ensure_initialized(id)?;

        let config = get_schema_config(&id.resource_type).ok_or_else(|| {
            ProviderError::internal(format!("Unknown resource type: {}", id.resource_type))
                .for_resource(id.clone())
        })?;

        let props = match self
            .cc_get_resource(config.aws_type_name, identifier.as_str())
            .await?
        {
            Some(props) => props,
            None => return Ok(ReadOutcome::not_found(id.clone())),
        };

        let mut attributes = map_aws_props_to_attributes(
            &props,
            &config.schema.attributes,
            &id.resource_type,
            &config.schema.defs,
        );

        // Match the #2544 optional-collection canonicalization contract: absent or
        // empty tags read as `tags = {}` so state never oscillates between absent
        // and an explicitly empty map. The schema declaration is authoritative here:
        // Tag metadata carries both the provider property name and its wire shape.
        if let Some(tags) = config.tags.as_ref() {
            let tags_map = tags.decode_wire_tags(props.get(tags.provider_name));
            attributes.insert(
                "tags".to_string(),
                Value::Concrete(ConcreteValue::Map(tags_map)),
            );
        }

        // Handle special cases
        self.read_special_attributes(&id.resource_type, &props, &mut attributes);

        // Synthesize attributes the Cloud Control read does not return
        // (e.g. cloudfront.Distribution.arn). Reads STS once, then caches.
        let _ = self
            .synthesize_read_attributes(&id.resource_type, &mut attributes)
            .await?;

        Ok(ReadOutcome::found(AppliedState::from_identifier(
            id.clone(),
            identifier.clone(),
            attributes,
        )))
    }

    pub async fn read_data_source_resource(&self, id: &ResourceId) -> ProviderResult<State> {
        Err(ProviderError::invalid_input(format!(
            "awscc does not provide data sources; requested type {}",
            id.resource_type
        ))
        .for_provider("awscc")
        .for_resource(id.clone()))
    }

    fn operation_budget(&self, resource_type: &str, kind: OperationKind) -> Duration {
        get_schema_config(resource_type)
            .and_then(|config| config.schema.operation_config.as_ref())
            .map_or(DEFAULT_OPERATION_BUDGET, |config| config.budget(kind))
    }

    fn create_desired_state(
        &self,
        resource: &Resource,
        config: &AwsccSchemaConfig,
    ) -> ProviderResult<serde_json::Value> {
        let mut desired_state = serde_json::Map::new();
        for (dsl_name, attr_schema) in &config.schema.attributes {
            if dsl_name == "tags" {
                continue;
            }
            if let Some(aws_name) = &attr_schema.provider_name
                && let Some(value) = resource.get_attr(dsl_name.as_str())
            {
                let aws_value = dsl_value_to_aws_with_defs(
                    value,
                    &attr_schema.attr_type,
                    &resource.id.resource_type,
                    dsl_name,
                    &config.schema.defs,
                );
                if let Some(v) = aws_value {
                    desired_state.insert(aws_name.to_string(), v);
                }
            }
        }
        self.create_special_attributes(resource, &mut desired_state);
        if let Some(tags_property) = config.tags.as_ref()
            && let Some(value) = resource.get_attr("tags")
        {
            let tags = tags_property.encode_dsl_tags(value).ok_or_else(|| {
                ProviderError::invalid_input(format!(
                    "failed to encode `tags` attribute for resource {}: expected a map with string values",
                    resource.id.resource_type
                ))
                .for_provider("awscc")
                .for_resource(resource.id.clone())
            })?;
            if !wire_tags_are_empty(&tags) {
                desired_state.insert(tags_property.provider_name.to_string(), tags);
            }
        }
        self.set_default_values(&resource.id.resource_type, &mut desired_state);
        Ok(serde_json::Value::Object(desired_state))
    }

    /// Perform the initial, non-blocking CloudControl create step.
    pub async fn create_step<A: CcCreateAttempt>(
        &self,
        resource: &Resource,
        attempt: A,
    ) -> Result<A::Step, A::Verdict> {
        if let Err(error) = self.ensure_initialized(&resource.id) {
            return Err(attempt.untracked(error));
        }
        let Some(config) = get_schema_config(&resource.id.resource_type) else {
            return Err(attempt.untracked(
                ProviderError::internal(format!(
                    "Unknown resource type: {}",
                    resource.id.resource_type
                ))
                .for_resource(resource.id.clone()),
            ));
        };
        let desired_state = match self.create_desired_state(resource, config) {
            Ok(desired_state) => desired_state,
            Err(error) => return Err(attempt.untracked(error)),
        };
        let pending = match self
            .cc_create_step(
                config.aws_type_name,
                desired_state,
                config.schema.operation_config.as_ref(),
            )
            .await
        {
            Ok(pending) => pending,
            Err(error) => return Err(attempt.untracked(error.for_resource(resource.id.clone()))),
        };
        self.finish_initial_pending(&resource.id, attempt, pending)
    }

    fn finish_initial_pending<A: CcCreateAttempt>(
        &self,
        id: &ResourceId,
        attempt: A,
        pending: CcCreatePending,
    ) -> Result<A::Step, A::Verdict> {
        let identifier = pending
            .token
            .identifier()
            .map(ToString::to_string)
            .and_then(|identifier| applied_identifier(id, "create", identifier).ok());
        match identifier {
            Some(identifier) => attempt
                .issued(identifier, Sent::AllRequestAttributes)
                .pending(pending.token, pending.poll_after_secs),
            None => attempt.pending(
                pending.token,
                Sent::AllRequestAttributes,
                pending.poll_after_secs,
            ),
        }
    }

    async fn resend_create_step<A: CcCreateAttempt>(
        &self,
        resource: &Resource,
        token: CcCreateToken,
        attempt: A,
    ) -> Result<A::Step, A::Verdict> {
        if let Err(error) = self.ensure_initialized(&resource.id) {
            return Err(attempt.untracked(error));
        }
        let Some(config) = get_schema_config(&resource.id.resource_type) else {
            return Err(attempt.untracked(
                ProviderError::internal(format!(
                    "Unknown resource type: {}",
                    resource.id.resource_type
                ))
                .for_resource(resource.id.clone()),
            ));
        };
        let desired_state = match self.create_desired_state(resource, config) {
            Ok(desired_state) => desired_state,
            Err(error) => return Err(attempt.untracked(error)),
        };
        let pending = match self
            .cc_reissue_create_step(
                config.aws_type_name,
                desired_state,
                config.schema.operation_config.as_ref(),
                token,
            )
            .await
        {
            Ok(pending) => pending,
            Err(error) => return Err(attempt.untracked(error.for_resource(resource.id.clone()))),
        };
        self.finish_initial_pending(&resource.id, attempt, pending)
    }

    /// Perform one CloudControl status or re-issue step for create.
    pub async fn poll_create_step<U: CcPollAttempt>(
        &self,
        resource: &Resource,
        token: CcCreateToken,
        poll: CcCreatePoll<U, U::Issued>,
    ) -> Result<U::Step, U::Verdict> {
        if self.ensure_initialized(&resource.id).is_err() {
            return self.repend_polled(&resource.id, poll, token, POLL_AFTER_SECS);
        }
        let Some(config) = get_schema_config(&resource.id.resource_type) else {
            return self.repend_polled(&resource.id, poll, token, POLL_AFTER_SECS);
        };

        match token {
            token @ CcCreateToken::NotIssued { .. } => {
                let pending = match self.create_desired_state(resource, config) {
                    Ok(desired_state) => {
                        self.cc_reissue_create_step(
                            config.aws_type_name,
                            desired_state,
                            config.schema.operation_config.as_ref(),
                            token,
                        )
                        .await
                    }
                    Err(error) => Err(error),
                };
                let pending = match pending {
                    Ok(pending) => pending,
                    Err(error) => {
                        let error = error.for_resource(resource.id.clone());
                        return match poll {
                            CcCreatePoll::Unissued(attempt) => Err(attempt.untracked(error)),
                            CcCreatePoll::Issued(issued) => {
                                Err(self.failed_issued(&resource.id, issued, error).await)
                            }
                        };
                    }
                };
                self.continue_polled_pending(&resource.id, poll, pending)
            }
            token @ CcCreateToken::InFlight { .. } => {
                let status = self.cc_create_request_status(token).await;
                self.finish_polled_status(resource, config, poll, status)
                    .await
            }
        }
    }

    fn repend_polled<U: CcPollAttempt>(
        &self,
        id: &ResourceId,
        poll: CcCreatePoll<U, U::Issued>,
        token: CcCreateToken,
        poll_after_secs: u32,
    ) -> Result<U::Step, U::Verdict> {
        self.continue_polled_pending(
            id,
            poll,
            CcCreatePending {
                token,
                poll_after_secs,
            },
        )
    }

    fn continue_polled_pending<U: CcPollAttempt>(
        &self,
        id: &ResourceId,
        poll: CcCreatePoll<U, U::Issued>,
        pending: CcCreatePending,
    ) -> Result<U::Step, U::Verdict> {
        match poll {
            CcCreatePoll::Unissued(attempt) => {
                let identifier = pending
                    .token
                    .identifier()
                    .map(ToString::to_string)
                    .and_then(|identifier| applied_identifier(id, "create", identifier).ok());
                match identifier {
                    Some(identifier) => attempt
                        .issued(identifier)
                        .pending(pending.token, pending.poll_after_secs),
                    None => attempt.pending(pending.token, pending.poll_after_secs),
                }
            }
            CcCreatePoll::Issued(issued) => {
                let identifier = issued.identifier().as_str().to_string();
                issued.pending(
                    pending.token.with_identifier(Some(identifier)),
                    pending.poll_after_secs,
                )
            }
        }
    }

    async fn finish_polled_status<U: CcPollAttempt>(
        &self,
        resource: &Resource,
        config: &AwsccSchemaConfig,
        poll: CcCreatePoll<U, U::Issued>,
        status: CcCreateStatus,
    ) -> Result<U::Step, U::Verdict> {
        match status {
            CcCreateStatus::Pending(pending) => {
                self.continue_polled_pending(&resource.id, poll, pending)
            }
            CcCreateStatus::StatusError {
                token,
                error: _error,
                poll_after_secs,
            } => self.repend_polled(&resource.id, poll, token, poll_after_secs),
            CcCreateStatus::RequestTokenNotFound { token, error } => {
                self.recover_expired_poll(resource, config, poll, token, error)
                    .await
            }
            CcCreateStatus::Success { token, identifier } => {
                self.complete_polled_success(resource, config, poll, token, identifier)
                    .await
            }
            CcCreateStatus::Failed {
                token,
                status,
                identifier,
                error_code,
                status_message,
            } => {
                self.finish_polled_failure(
                    resource,
                    config,
                    poll,
                    CreateTerminalFailure {
                        token,
                        status,
                        identifier,
                        error_code,
                        status_message,
                    },
                )
                .await
            }
        }
    }

    async fn recover_expired_poll<U: CcPollAttempt>(
        &self,
        resource: &Resource,
        config: &AwsccSchemaConfig,
        poll: CcCreatePoll<U, U::Issued>,
        token: CcCreateToken,
        token_error: ProviderError,
    ) -> Result<U::Step, U::Verdict> {
        let issued = match poll {
            CcCreatePoll::Issued(issued) => issued,
            CcCreatePoll::Unissued(attempt) => {
                let Some(identifier) = token_identifier(&resource.id, &token) else {
                    return attempt.pending(token, POLL_AFTER_SECS);
                };
                attempt.issued(identifier)
            }
        };
        let identifier = issued.identifier().clone();
        match self.read_resource(&resource.id, &identifier).await {
            Ok(found @ ReadOutcome::Found { .. }) => {
                self.finish_create_success(resource, config, issued, identifier, Ok(found))
                    .await
            }
            // Cloud Control only forgets request tokens after the operation has
            // finished. A successful absence read is therefore terminal; the
            // forgotten token can never make progress if it is re-pended.
            Ok(ReadOutcome::NotFound { .. }) => Err(issued.discarded(token_error)),
            // A read failure does not prove absence, so keep the accepted
            // create in flight and let a later observation resolve it.
            Err(_) => issued.pending(token, POLL_AFTER_SECS),
        }
    }

    async fn complete_polled_success<U: CcPollAttempt>(
        &self,
        resource: &Resource,
        config: &AwsccSchemaConfig,
        poll: CcCreatePoll<U, U::Issued>,
        token: CcCreateToken,
        status_identifier: Option<String>,
    ) -> Result<U::Step, U::Verdict> {
        let status_identifier = status_identifier
            .and_then(|identifier| applied_identifier(&resource.id, "create", identifier).ok());
        let (issued, read_identifier) = match poll {
            CcCreatePoll::Unissued(attempt) => {
                let Some(identifier) =
                    status_identifier.or_else(|| token_identifier(&resource.id, &token))
                else {
                    return Err(attempt.untracked(
                        ProviderError::api_error(
                            "Cloud Control reported create success without an identifier",
                        )
                        .for_resource(resource.id.clone()),
                    ));
                };
                (attempt.issued(identifier.clone()), identifier)
            }
            CcCreatePoll::Issued(issued) => {
                let identifier = status_identifier.unwrap_or_else(|| issued.identifier().clone());
                (issued, identifier)
            }
        };
        let read_back = self.read_resource(&resource.id, &read_identifier).await;
        self.finish_create_success(resource, config, issued, read_identifier, read_back)
            .await
    }

    async fn finish_polled_failure<U: CcPollAttempt>(
        &self,
        resource: &Resource,
        config: &AwsccSchemaConfig,
        poll: CcCreatePoll<U, U::Issued>,
        failure: CreateTerminalFailure,
    ) -> Result<U::Step, U::Verdict> {
        let CreateTerminalFailure {
            token,
            status,
            identifier: status_identifier,
            error_code,
            status_message,
        } = failure;
        let error = cloudcontrol_create_failure(
            &resource.id,
            &status,
            error_code.as_ref(),
            &status_message,
        );
        let status_identifier = status_identifier
            .and_then(|identifier| applied_identifier(&resource.id, "create", identifier).ok());
        match poll {
            CcCreatePoll::Issued(issued) => {
                let identifier = status_identifier.unwrap_or_else(|| issued.identifier().clone());
                Err(self
                    .failed_issued_as(&resource.id, identifier, issued, error)
                    .await)
            }
            CcCreatePoll::Unissued(attempt) => {
                if let Some(identifier) = status_identifier {
                    let issued = attempt.issued(identifier.clone());
                    Err(self
                        .failed_issued_as(&resource.id, identifier, issued, error)
                        .await)
                } else if matches!(status, OperationStatus::Failed)
                    && handler_failure_retryable(&status_message)
                    && token.attempt() < create_max_attempts(config)
                {
                    let attempt_number = token.attempt();
                    attempt.pending(
                        token.retry_not_issued(attempt_number + 1),
                        create_retry_delay(attempt_number),
                    )
                } else {
                    Err(attempt.untracked(error))
                }
            }
        }
    }

    async fn finish_create_success<I: CcIssuedCreate>(
        &self,
        resource: &Resource,
        config: &AwsccSchemaConfig,
        issued: I,
        wait_identifier: AppliedIdentifier,
        read_back: ProviderResult<ReadOutcome>,
    ) -> Result<I::Step, I::Verdict> {
        let completion = self
            .create_completion_from_read(resource, config, wait_identifier, read_back)
            .await;
        Ok(match completion {
            CreateCompletion::Success {
                identifier,
                attributes,
            } => issued.succeeded(identifier, attributes),
            CreateCompletion::Partial {
                identifier,
                attributes,
                reason,
                missing_attributes,
            } => issued.partial(identifier, attributes, reason, missing_attributes),
        })
    }

    async fn failed_issued<I: CcIssuedCreate>(
        &self,
        id: &ResourceId,
        issued: I,
        error: ProviderError,
    ) -> I::Verdict {
        let identifier = issued.identifier().clone();
        self.failed_issued_as(id, identifier, issued, error).await
    }

    async fn failed_issued_as<I: CcIssuedCreate>(
        &self,
        id: &ResourceId,
        identifier: AppliedIdentifier,
        issued: I,
        error: ProviderError,
    ) -> I::Verdict {
        let observation = match self.read_resource(id, &identifier).await {
            Ok(outcome) => Observation::Read(outcome),
            Err(read_error) => Observation::ReadFailed(read_error),
        };
        issued.failed(identifier, observation, error)
    }

    async fn create_completion_from_read(
        &self,
        resource: &Resource,
        config: &AwsccSchemaConfig,
        wait_identifier: AppliedIdentifier,
        read_back: ProviderResult<ReadOutcome>,
    ) -> CreateCompletion {
        let read_state = match read_back {
            Ok(ReadOutcome::Found { state }) => state,
            Ok(ReadOutcome::NotFound { .. }) => {
                return CreateCompletion::Partial {
                    identifier: wait_identifier,
                    attributes: resource.resolved_attributes(),
                    reason: "read-back returned not_found".to_string(),
                    missing_attributes: vec!["state".to_string()],
                };
            }
            Err(read_error) => {
                let missing_attributes = nonempty_missing_attributes(
                    config
                        .schema
                        .attributes
                        .keys()
                        .filter(|name| resource.get_attr(name.as_str()).is_some())
                        .cloned()
                        .collect(),
                );
                return CreateCompletion::Partial {
                    identifier: wait_identifier,
                    attributes: HashMap::new(),
                    reason: format!("read error: {read_error}"),
                    missing_attributes,
                };
            }
        };

        let mut attributes = merge_desired_attributes(read_state.attributes(), resource, config);
        let canonical_identifier =
            match canonicalize_identifier_from_read(config, &resource.id, &attributes) {
                Ok(Some(identifier)) => identifier,
                Ok(None) => {
                    let missing_attributes =
                        missing_primary_identifier_attributes(config, &attributes);
                    return CreateCompletion::Partial {
                        identifier: wait_identifier,
                        attributes,
                        reason: format!(
                            "read-back missing primaryIdentifier attributes: {}",
                            missing_attributes.join(", ")
                        ),
                        missing_attributes: nonempty_missing_attributes(missing_attributes),
                    };
                }
                Err(error) => {
                    return CreateCompletion::Partial {
                        identifier: wait_identifier,
                        attributes,
                        reason: format!("failed to canonicalize read-back identifier: {error}"),
                        missing_attributes: vec!["state".to_string()],
                    };
                }
            };

        let synthesis = self
            .synthesize_read_attributes(&resource.id.resource_type, &mut attributes)
            .await;
        let synthesis = match synthesis {
            Ok(synthesis) => synthesis,
            Err(error) => {
                return CreateCompletion::Partial {
                    identifier: wait_identifier,
                    attributes,
                    reason: format!("failed to synthesize read-back attributes: {error}"),
                    missing_attributes: vec!["state".to_string()],
                };
            }
        };

        match synthesis {
            SynthesisStatus::Complete => CreateCompletion::Success {
                identifier: canonical_identifier,
                attributes,
            },
            SynthesisStatus::Missing {
                attributes: missing_attributes,
            } => CreateCompletion::Partial {
                identifier: canonical_identifier,
                attributes,
                reason: format!(
                    "read-back missing synthesized attributes: {}",
                    missing_attributes.join(", ")
                ),
                missing_attributes: nonempty_missing_attributes(missing_attributes),
            },
        }
    }

    pub(crate) async fn drive_native_create(
        &self,
        resource: &Resource,
        attempt: CreateAttempt,
    ) -> Result<CreateOutcome, CreateFailure> {
        let budget =
            self.operation_budget(resource.id.resource_type.as_str(), OperationKind::Create);
        let started = tokio::time::Instant::now();
        let mut step = self.create_step(resource, attempt).await?;
        loop {
            let pending = match step {
                CoreCreateStep::Done(outcome) => return Ok(outcome),
                CoreCreateStep::Pending(pending) => pending,
            };
            if started.elapsed() >= budget {
                return Err(pending.preserve(
                    ProviderError::timeout(format!(
                        "create of {} did not finish within {budget:?}",
                        resource.id.resource_type
                    ))
                    .for_resource(resource.id.clone()),
                ));
            }
            let remaining = budget.saturating_sub(started.elapsed());
            let delay = Duration::from_secs(u64::from(pending.poll_after_secs().clamp(1, 300)))
                .min(remaining);
            tokio::time::sleep(delay).await;
            let token = match decode_create_token(pending.token()) {
                Ok(token) => token,
                Err(error) => return Err(pending.preserve(error)),
            };
            let poll = match pending.into_poll() {
                Poll::Unissued(attempt) => CcCreatePoll::Unissued(attempt),
                Poll::Issued(issued) => CcCreatePoll::Issued(issued),
            };
            step = self.poll_create_step(resource, token, poll).await?;
        }
    }

    /// Continue one create step loaded from persisted state.
    pub async fn resume_create_step<U: CcResumeAttempt>(
        &self,
        resource: &Resource,
        token: CcCreateToken,
        resume: CcCreateResume<U, U::Resumed>,
    ) -> Result<U::Step, U::Verdict> {
        if self.ensure_initialized(&resource.id).is_err() {
            return self.repend_or_preserve_resume(resume, token);
        }
        let Some(config) = get_schema_config(&resource.id.resource_type) else {
            return self.repend_or_preserve_resume(resume, token);
        };

        match token {
            token @ CcCreateToken::NotIssued { .. } => match resume {
                CcCreateResume::Unissued(attempt) => {
                    self.resend_create_step(resource, token, attempt.resend())
                        .await
                }
                CcCreateResume::Issued(resumed) => resumed.pending(token, POLL_AFTER_SECS),
            },
            token @ CcCreateToken::InFlight { .. } => {
                // The persisted identifier does not prove completion. Always
                // ask CloudControl about its accepted request before any read.
                let status = self.cc_create_request_status(token).await;
                self.finish_resumed_status(resource, config, resume, status)
                    .await
            }
        }
    }

    fn repend_or_preserve_resume<U: CcResumeAttempt>(
        &self,
        resume: CcCreateResume<U, U::Resumed>,
        token: CcCreateToken,
    ) -> Result<U::Step, U::Verdict> {
        match resume {
            CcCreateResume::Unissued(attempt) => Err(attempt.undetermined(
                ProviderError::internal("cannot prepare the recorded create for resume"),
            )),
            CcCreateResume::Issued(resumed) => resumed.pending(token, POLL_AFTER_SECS),
        }
    }

    fn continue_resumed_pending<U: CcResumeAttempt>(
        &self,
        id: &ResourceId,
        resume: CcCreateResume<U, U::Resumed>,
        pending: CcCreatePending,
    ) -> Result<U::Step, U::Verdict> {
        match resume {
            CcCreateResume::Unissued(attempt) => {
                let identifier = token_identifier(id, &pending.token);
                match identifier {
                    Some(identifier) => attempt
                        .issued(identifier)
                        .pending(pending.token, pending.poll_after_secs),
                    None => attempt.pending(pending.token, pending.poll_after_secs),
                }
            }
            CcCreateResume::Issued(resumed) => {
                let identifier = resumed.identifier().as_str().to_string();
                resumed.pending(
                    pending.token.with_identifier(Some(identifier)),
                    pending.poll_after_secs,
                )
            }
        }
    }

    async fn finish_resumed_status<U: CcResumeAttempt>(
        &self,
        resource: &Resource,
        config: &AwsccSchemaConfig,
        resume: CcCreateResume<U, U::Resumed>,
        status: CcCreateStatus,
    ) -> Result<U::Step, U::Verdict> {
        match status {
            CcCreateStatus::Pending(pending) => {
                self.continue_resumed_pending(&resource.id, resume, pending)
            }
            CcCreateStatus::StatusError {
                token,
                error: _error,
                poll_after_secs,
            } => match resume {
                CcCreateResume::Unissued(attempt) => attempt.pending(token, poll_after_secs),
                CcCreateResume::Issued(resumed) => resumed.pending(token, poll_after_secs),
            },
            CcCreateStatus::RequestTokenNotFound { token, error } => {
                self.recover_expired_resume(resource, config, resume, token, error)
                    .await
            }
            CcCreateStatus::Success { token, identifier } => {
                self.complete_resumed_success(resource, config, resume, token, identifier)
                    .await
            }
            CcCreateStatus::Failed {
                token,
                status,
                identifier,
                error_code,
                status_message,
            } => {
                self.finish_resumed_failure(
                    resource,
                    config,
                    resume,
                    CreateTerminalFailure {
                        token,
                        status,
                        identifier,
                        error_code,
                        status_message,
                    },
                )
                .await
            }
        }
    }

    async fn recover_expired_resume<U: CcResumeAttempt>(
        &self,
        resource: &Resource,
        config: &AwsccSchemaConfig,
        resume: CcCreateResume<U, U::Resumed>,
        token: CcCreateToken,
        token_error: ProviderError,
    ) -> Result<U::Step, U::Verdict> {
        let resumed = match resume {
            CcCreateResume::Issued(resumed) => resumed,
            CcCreateResume::Unissued(attempt) => {
                let Some(identifier) = token_identifier(&resource.id, &token) else {
                    return Err(attempt.undetermined(token_error));
                };
                attempt.issued(identifier)
            }
        };
        let identifier = resumed.identifier().clone();
        match self.read_resource(&resource.id, &identifier).await {
            Ok(found @ ReadOutcome::Found { .. }) => {
                self.finish_resumed_read(resource, config, resumed, identifier, Ok(found))
                    .await
            }
            // Cloud Control only forgets request tokens after the operation has
            // finished. A successful absence read is therefore terminal; the
            // forgotten token can never make progress if it is re-pended.
            Ok(ReadOutcome::NotFound { .. }) => Err(resumed.discarded(token_error)),
            // A read failure does not prove absence, so preserve the accepted
            // create for a later observation.
            Err(_) => resumed.pending(token, POLL_AFTER_SECS),
        }
    }

    async fn complete_resumed_success<U: CcResumeAttempt>(
        &self,
        resource: &Resource,
        config: &AwsccSchemaConfig,
        resume: CcCreateResume<U, U::Resumed>,
        token: CcCreateToken,
        status_identifier: Option<String>,
    ) -> Result<U::Step, U::Verdict> {
        let status_identifier = status_identifier
            .and_then(|identifier| applied_identifier(&resource.id, "create", identifier).ok());
        let (resumed, read_identifier) = match resume {
            CcCreateResume::Unissued(attempt) => {
                let Some(identifier) =
                    status_identifier.or_else(|| token_identifier(&resource.id, &token))
                else {
                    return Err(attempt.untracked(
                        ProviderError::api_error(
                            "Cloud Control reported create success without an identifier",
                        )
                        .for_resource(resource.id.clone()),
                    ));
                };
                (attempt.issued(identifier.clone()), identifier)
            }
            CcCreateResume::Issued(resumed) => {
                let identifier = status_identifier.unwrap_or_else(|| resumed.identifier().clone());
                (resumed, identifier)
            }
        };
        let read_back = self.read_resource(&resource.id, &read_identifier).await;
        self.finish_resumed_read(resource, config, resumed, read_identifier, read_back)
            .await
    }

    async fn finish_resumed_failure<U: CcResumeAttempt>(
        &self,
        resource: &Resource,
        config: &AwsccSchemaConfig,
        resume: CcCreateResume<U, U::Resumed>,
        failure: CreateTerminalFailure,
    ) -> Result<U::Step, U::Verdict> {
        let CreateTerminalFailure {
            token,
            status,
            identifier: status_identifier,
            error_code,
            status_message,
        } = failure;
        let error = cloudcontrol_create_failure(
            &resource.id,
            &status,
            error_code.as_ref(),
            &status_message,
        );
        let status_identifier = status_identifier
            .and_then(|identifier| applied_identifier(&resource.id, "create", identifier).ok());
        match resume {
            CcCreateResume::Issued(resumed) => {
                let identifier = status_identifier.unwrap_or_else(|| resumed.identifier().clone());
                Err(self
                    .failed_resumed_as(&resource.id, identifier, resumed, error)
                    .await)
            }
            CcCreateResume::Unissued(attempt) => {
                if let Some(identifier) = status_identifier {
                    let resumed = attempt.issued(identifier.clone());
                    Err(self
                        .failed_resumed_as(&resource.id, identifier, resumed, error)
                        .await)
                } else if matches!(status, OperationStatus::Failed)
                    && handler_failure_retryable(&status_message)
                    && token.attempt() < create_max_attempts(config)
                {
                    let attempt_number = token.attempt();
                    attempt.resend().pending(
                        token.retry_not_issued(attempt_number + 1),
                        Sent::AllRequestAttributes,
                        create_retry_delay(attempt_number),
                    )
                } else {
                    Err(attempt.untracked(error))
                }
            }
        }
    }

    async fn finish_resumed_read<R: CcResumedCreate>(
        &self,
        resource: &Resource,
        config: &AwsccSchemaConfig,
        resumed: R,
        wait_identifier: AppliedIdentifier,
        read_back: ProviderResult<ReadOutcome>,
    ) -> Result<R::Step, R::Verdict> {
        let read_state = match read_back {
            Ok(ReadOutcome::Found { state }) => state,
            Ok(not_found @ ReadOutcome::NotFound { .. }) => {
                return Err(resumed.failed(
                    wait_identifier,
                    Observation::Read(not_found),
                    ProviderError::api_error(
                        "Cloud Control reported create success, but read-back returned not_found",
                    )
                    .for_resource(resource.id.clone()),
                ));
            }
            Err(read_error) => {
                let message = format!(
                    "Cloud Control reported create success, but read-back failed: {read_error}"
                );
                return Err(resumed.failed(
                    wait_identifier,
                    Observation::ReadFailed(read_error),
                    ProviderError::api_error(message).for_resource(resource.id.clone()),
                ));
            }
        };
        let observed = AppliedState::from_identifier(
            resource.id.clone(),
            read_state.identifier().clone(),
            read_state.attributes().clone(),
        );
        // A resumed completion is an observation. The host combines it with
        // the accepted request's persisted `sent` values when writing state.
        let mut attributes = read_state.attributes().clone();
        let canonical_identifier =
            match canonicalize_identifier_from_read(config, &resource.id, &attributes) {
                Ok(Some(identifier)) => identifier,
                Ok(None) => {
                    let missing = missing_primary_identifier_attributes(config, &attributes);
                    return Err(resumed.failed(
                        wait_identifier,
                        Observation::Read(ReadOutcome::found(observed)),
                        ProviderError::api_error(format!(
                            "post-create read-back is missing primaryIdentifier attributes: {}",
                            missing.join(", ")
                        ))
                        .for_resource(resource.id.clone()),
                    ));
                }
                Err(error) => {
                    return Err(resumed.failed(
                        wait_identifier,
                        Observation::Read(ReadOutcome::found(observed)),
                        error,
                    ));
                }
            };
        match self
            .synthesize_read_attributes(&resource.id.resource_type, &mut attributes)
            .await
        {
            Ok(SynthesisStatus::Complete) => {
                let state = AppliedState::from_identifier(
                    resource.id.clone(),
                    canonical_identifier.clone(),
                    attributes,
                );
                Ok(resumed.completed(canonical_identifier, state))
            }
            Ok(SynthesisStatus::Missing {
                attributes: missing,
            }) => Err(resumed.failed(
                wait_identifier,
                Observation::Read(ReadOutcome::found(observed)),
                ProviderError::api_error(format!(
                    "post-create read-back is missing synthesized attributes: {}",
                    missing.join(", ")
                ))
                .for_resource(resource.id.clone()),
            )),
            Err(error) => Err(resumed.failed(
                wait_identifier,
                Observation::Read(ReadOutcome::found(observed)),
                error,
            )),
        }
    }

    async fn failed_resumed_as<R: CcResumedCreate>(
        &self,
        id: &ResourceId,
        identifier: AppliedIdentifier,
        resumed: R,
        error: ProviderError,
    ) -> R::Verdict {
        let observation = match self.read_resource(id, &identifier).await {
            Ok(outcome) => Observation::Read(outcome),
            Err(read_error) => Observation::ReadFailed(read_error),
        };
        resumed.failed(identifier, observation, error)
    }

    pub(crate) async fn drive_native_resume(
        &self,
        request: ResumeCreateRequest,
    ) -> Result<ResumeOutcome, ResumeFailure> {
        let budget =
            self.operation_budget(request.id().resource_type.as_str(), OperationKind::Create);
        let started = tokio::time::Instant::now();
        let (resume, create_request) = request.into_parts();
        let resource = create_request.resource().as_resource().clone();
        let token = match decode_create_token(resume.token()) {
            Ok(token) => token,
            Err(error) => return Err(resume.preserve(error)),
        };
        let resume = match resume {
            Resume::Unissued(attempt) => CcCreateResume::Unissued(attempt),
            Resume::Issued(resumed) => CcCreateResume::Issued(resumed),
        };
        let mut step = self.resume_create_step(&resource, token, resume).await?;
        loop {
            enum Next {
                Resume(Resume),
                Resent(ResentPendingCreate),
            }
            let next = match step {
                CoreResumeStep::Done(outcome) => return Ok(outcome),
                CoreResumeStep::Pending(resume) => Next::Resume(*resume),
                CoreResumeStep::ResentPending(pending) => Next::Resent(pending),
            };
            if started.elapsed() >= budget {
                let error = ProviderError::timeout(format!(
                    "create of {} did not finish within {budget:?}",
                    resource.id.resource_type
                ))
                .for_resource(resource.id.clone());
                return Err(match next {
                    Next::Resume(resume) => resume.preserve(error),
                    Next::Resent(pending) => pending.preserve(error),
                });
            }
            let remaining = budget.saturating_sub(started.elapsed());
            let poll_after_secs = match &next {
                Next::Resume(_resume) => POLL_AFTER_SECS,
                Next::Resent(pending) => pending.poll_after_secs(),
            };
            let delay =
                Duration::from_secs(u64::from(poll_after_secs.clamp(1, 300))).min(remaining);
            tokio::time::sleep(delay).await;
            step = match next {
                Next::Resume(resume) => {
                    let token = match decode_create_token(resume.token()) {
                        Ok(token) => token,
                        Err(error) => return Err(resume.preserve(error)),
                    };
                    let resume = match resume {
                        Resume::Unissued(attempt) => CcCreateResume::Unissued(attempt),
                        Resume::Issued(resumed) => CcCreateResume::Issued(resumed),
                    };
                    self.resume_create_step(&resource, token, resume).await?
                }
                Next::Resent(pending) => {
                    let token = match decode_create_token(pending.token()) {
                        Ok(token) => token,
                        Err(error) => return Err(pending.preserve(error)),
                    };
                    let poll = match pending.into_poll() {
                        ResentPoll::Unissued(attempt) => CcCreatePoll::Unissued(attempt),
                        ResentPoll::Issued(issued) => CcCreatePoll::Issued(issued),
                    };
                    self.poll_create_step(&resource, token, poll).await?
                }
            };
        }
    }

    /// Perform the initial, non-blocking CloudControl update step.
    pub async fn update_step(
        &self,
        id: &ResourceId,
        identifier: &AppliedIdentifier,
        from: &State,
        patch: &UpdatePatch,
    ) -> ProviderResult<CcUpdateStep> {
        self.ensure_initialized(id)?;
        let config = get_schema_config(&id.resource_type).ok_or_else(|| {
            ProviderError::internal(format!("Unknown resource type: {}", id.resource_type))
                .for_resource(id.clone())
        })?;
        let patch_plan = build_update_patches(config, &id.resource_type, patch);
        let step = self
            .cc_update_step(
                config.aws_type_name,
                identifier.as_str(),
                patch_plan.ops.clone(),
                config.schema.operation_config.as_ref(),
            )
            .await
            .map_err(|error| error.for_resource(id.clone()))?;
        let desired = post_update_attributes(from, patch);
        self.finish_update_step(id, identifier, desired, config, patch_plan, step)
            .await
    }

    /// Perform one CloudControl status or re-issue step for update.
    pub async fn poll_update_step(
        &self,
        id: &ResourceId,
        identifier: &AppliedIdentifier,
        from: &State,
        patch: &UpdatePatch,
        token: CcUpdateToken,
    ) -> ProviderResult<CcUpdateStep> {
        self.ensure_initialized(id)?;
        let config = get_schema_config(&id.resource_type).ok_or_else(|| {
            ProviderError::internal(format!("Unknown resource type: {}", id.resource_type))
                .for_resource(id.clone())
        })?;
        let patch_plan = build_update_patches(config, &id.resource_type, patch);
        let step = self
            .cc_poll_update_step(
                config.aws_type_name,
                identifier.as_str(),
                patch_plan.ops.clone(),
                config.schema.operation_config.as_ref(),
                token,
            )
            .await
            .map_err(|error| error.for_resource(id.clone()))?;
        let desired = post_update_attributes(from, patch);
        self.finish_update_step(id, identifier, desired, config, patch_plan, step)
            .await
    }

    async fn finish_update_step(
        &self,
        id: &ResourceId,
        identifier: &AppliedIdentifier,
        desired: HashMap<String, Value>,
        config: &AwsccSchemaConfig,
        patch_plan: UpdatePatchPlan,
        step: OperationStep<WaitOutcome, ProviderError, CcUpdateToken>,
    ) -> ProviderResult<CcUpdateStep> {
        let outcome = match step {
            OperationStep::Pending {
                pending,
                poll_after_secs,
            } => {
                return Ok(OperationStep::Pending {
                    pending,
                    poll_after_secs,
                });
            }
            OperationStep::Verdict(error) => return Ok(OperationStep::Verdict(error)),
            OperationStep::Done(outcome) => outcome,
        };
        let handler_failure = match outcome {
            WaitOutcome::Success { .. } => None,
            WaitOutcome::PartialOrFailed { status_message, .. } => Some(status_message),
        };
        let read_back = self.read_resource(id, identifier).await;
        let outcome = update_outcome_from_read_back(
            id,
            identifier,
            read_back,
            desired,
            &patch_plan,
            config,
            handler_failure.as_deref(),
        );
        Ok(OperationStep::Done(outcome))
    }

    /// Drive update steps to a settled result for native callers.
    pub async fn update_resource(
        &self,
        id: ResourceId,
        identifier: &AppliedIdentifier,
        from: &State,
        patch: &UpdatePatch,
    ) -> ProviderResult<UpdateOutcome> {
        let budget = self.operation_budget(id.resource_type.as_str(), OperationKind::Update);
        drive_operation(
            &id,
            OperationKind::Update,
            budget,
            self.update_step(&id, identifier, from, patch),
            |token| self.poll_update_step(&id, identifier, from, patch, token),
            tokio::time::sleep,
        )
        .await
    }

    /// Perform the initial, non-blocking CloudControl delete step.
    pub async fn delete_step(
        &self,
        id: &ResourceId,
        identifier: &AppliedIdentifier,
        request: &DeleteRequest,
    ) -> ProviderResult<CcDeleteStep> {
        self.ensure_initialized(id)?;
        let config = get_schema_config(&id.resource_type).ok_or_else(|| {
            ProviderError::internal(format!("Unknown resource type: {}", id.resource_type))
                .for_resource(id.clone())
        })?;

        if let Some(patch_ops) = self
            .internet_gateway_detach_patch_if_attached(id, config, identifier.as_str())
            .await?
        {
            let detach = self
                .cc_update_step(
                    config.aws_type_name,
                    identifier.as_str(),
                    patch_ops,
                    config.schema.operation_config.as_ref(),
                )
                .await
                .map_err(|error| detach_igw_error(id, error))?;
            return self
                .continue_after_igw_detach(id, config, identifier, detach)
                .await;
        }

        if request.directives.force_delete && id.resource_type == "s3.Bucket" {
            self.empty_s3_bucket(identifier.as_str())
                .await
                .map_err(|error| {
                    ProviderError::api_error_caused_by(
                        "Failed to empty S3 bucket before deletion",
                        error,
                    )
                    .for_resource(id.clone())
                })?;
        }

        self.cc_delete_step(
            config.aws_type_name,
            identifier.as_str(),
            config.schema.operation_config.as_ref(),
        )
        .await
        .map(wrap_delete_phase)
        .map_err(|error| error.for_resource(id.clone()))
    }

    /// Perform one CloudControl status or re-issue step for delete.
    pub async fn poll_delete_step(
        &self,
        id: &ResourceId,
        identifier: &AppliedIdentifier,
        _request: &DeleteRequest,
        token: CcDeleteToken,
    ) -> ProviderResult<CcDeleteStep> {
        self.ensure_initialized(id)?;
        let config = get_schema_config(&id.resource_type).ok_or_else(|| {
            ProviderError::internal(format!("Unknown resource type: {}", id.resource_type))
                .for_resource(id.clone())
        })?;

        match token {
            CcDeleteToken::DetachIgw(phase) => {
                let patch_ops = if matches!(&phase, CcRequestPhase::NotIssued { .. }) {
                    let Some(patch_ops) = self
                        .internet_gateway_detach_patch_if_attached(id, config, identifier.as_str())
                        .await?
                    else {
                        return self
                            .cc_delete_step(
                                config.aws_type_name,
                                identifier.as_str(),
                                config.schema.operation_config.as_ref(),
                            )
                            .await
                            .map(wrap_delete_phase)
                            .map_err(|error| error.for_resource(id.clone()));
                    };
                    patch_ops
                } else {
                    internet_gateway_detach_patch()
                };
                let detach = self
                    .cc_poll_update_step(
                        config.aws_type_name,
                        identifier.as_str(),
                        patch_ops,
                        config.schema.operation_config.as_ref(),
                        phase,
                    )
                    .await
                    .map_err(|error| detach_igw_error(id, error))?;
                self.continue_after_igw_detach(id, config, identifier, detach)
                    .await
            }
            CcDeleteToken::Delete(phase) => self
                .cc_poll_delete_step(
                    config.aws_type_name,
                    identifier.as_str(),
                    config.schema.operation_config.as_ref(),
                    phase,
                )
                .await
                .map(wrap_delete_phase)
                .map_err(|error| error.for_resource(id.clone())),
        }
    }

    async fn continue_after_igw_detach(
        &self,
        id: &ResourceId,
        config: &AwsccSchemaConfig,
        identifier: &AppliedIdentifier,
        step: OperationStep<WaitOutcome, ProviderError, CcUpdateToken>,
    ) -> ProviderResult<CcDeleteStep> {
        match step {
            OperationStep::Pending {
                pending,
                poll_after_secs,
            } => Ok(OperationStep::Pending {
                pending: CcDeleteToken::DetachIgw(pending),
                poll_after_secs,
            }),
            OperationStep::Verdict(error) => Ok(OperationStep::Verdict(error)),
            OperationStep::Done(WaitOutcome::Success { .. }) => self
                .cc_delete_step(
                    config.aws_type_name,
                    identifier.as_str(),
                    config.schema.operation_config.as_ref(),
                )
                .await
                .map(wrap_delete_phase)
                .map_err(|error| error.for_resource(id.clone())),
            OperationStep::Done(WaitOutcome::PartialOrFailed { status_message, .. }) => {
                Err(ProviderError::api_error_caused_by(
                    "Failed to detach Internet Gateway from VPC before deletion",
                    crate::provider::error_helpers::CloudControlOperationFailure::new(
                        status_message,
                    ),
                )
                .for_resource(id.clone()))
            }
        }
    }

    /// Drive delete steps to a settled result for native callers.
    pub async fn delete_resource(
        &self,
        id: &ResourceId,
        identifier: &AppliedIdentifier,
        directives: &Directives,
    ) -> ProviderResult<()> {
        let budget = self.operation_budget(id.resource_type.as_str(), OperationKind::Delete);
        let request = DeleteRequest {
            directives: directives.clone(),
        };
        drive_operation(
            id,
            OperationKind::Delete,
            budget,
            self.delete_step(id, identifier, &request),
            |token| self.poll_delete_step(id, identifier, &request, token),
            tokio::time::sleep,
        )
        .await
    }
}

fn wrap_delete_phase(step: OperationStep<(), ProviderError, CcRequestPhase>) -> CcDeleteStep {
    match step {
        OperationStep::Done(()) => OperationStep::Done(()),
        OperationStep::Verdict(error) => OperationStep::Verdict(error),
        OperationStep::Pending {
            pending,
            poll_after_secs,
        } => OperationStep::Pending {
            pending: CcDeleteToken::Delete(pending),
            poll_after_secs,
        },
    }
}

fn detach_igw_error(id: &ResourceId, error: ProviderError) -> ProviderError {
    ProviderError::api_error_caused_by(
        "Failed to detach Internet Gateway from VPC before deletion",
        error,
    )
    .for_resource(id.clone())
}

fn applied_identifier(
    id: &ResourceId,
    operation: &'static str,
    identifier: String,
) -> ProviderResult<AppliedIdentifier> {
    AppliedIdentifier::new(identifier).map_err(|_| {
        ProviderError::api_error(format!(
            "Cloud Control {operation} returned an empty provider identifier"
        ))
        .for_resource(id.clone())
        .with_operation(operation)
    })
}

fn token_identifier(id: &ResourceId, token: &CcCreateToken) -> Option<AppliedIdentifier> {
    token
        .identifier()
        .and_then(|identifier| applied_identifier(id, "create", identifier.to_string()).ok())
}

fn create_max_attempts(config: &AwsccSchemaConfig) -> u32 {
    config
        .schema
        .operation_config
        .as_ref()
        .and_then(|operation| operation.create_max_retries)
        .unwrap_or(super::CREATE_RETRY_MAX_ATTEMPTS)
}

fn create_retry_delay(attempt: u32) -> u32 {
    let delay = super::CREATE_RETRY_INITIAL_DELAY_SECS
        .saturating_mul(2_u64.saturating_pow(attempt))
        .min(super::CREATE_RETRY_MAX_DELAY_SECS);
    u32::try_from(delay).unwrap_or(u32::MAX)
}

fn handler_failure_retryable(status_message: &str) -> bool {
    AwsccProvider::is_retryable_status_message(status_message)
}

fn cloudcontrol_create_failure(
    id: &ResourceId,
    status: &OperationStatus,
    error_code: Option<&HandlerErrorCode>,
    status_message: &str,
) -> ProviderError {
    let mut error = ProviderError::api_error(format!(
        "Cloud Control create ended with {}: {status_message}",
        status.as_str()
    ))
    .for_resource(id.clone())
    .with_operation("cloudcontrol.CreateResource");
    if let Some(error_code) = error_code {
        error = error.with_code(error_code.as_str());
    }
    error
}

fn read_back_reason(handler_failure: Option<&str>, detail: impl Into<String>) -> String {
    let detail = detail.into();
    match handler_failure {
        Some(status_message) => format!("handler failed: {status_message}; {detail}"),
        None => detail,
    }
}

fn nonempty_missing_attributes(mut attributes: Vec<String>) -> Vec<String> {
    if attributes.is_empty() {
        attributes.push("state".to_string());
    }
    attributes
}

fn canonicalize_identifier_from_read(
    config: &AwsccSchemaConfig,
    id: &ResourceId,
    attributes: &HashMap<String, Value>,
) -> ProviderResult<Option<AppliedIdentifier>> {
    let mut segments = Vec::with_capacity(config.primary_identifier.len());
    for attr in config.primary_identifier {
        let Some(value) = attributes.get(attr.dsl_name) else {
            return Ok(None);
        };
        let Some(segment) = primary_identifier_segment(value) else {
            return Ok(None);
        };
        segments.push(segment);
    }
    AppliedIdentifier::new(segments.join("|"))
        .map(Some)
        .map_err(|error| {
            ProviderError::api_error_caused_by(
                "Cloud Control read-back produced an empty canonical identifier",
                error,
            )
            .for_resource(id.clone())
            .with_operation("cloudcontrol.GetResource")
        })
}

fn missing_primary_identifier_attributes(
    config: &AwsccSchemaConfig,
    attributes: &HashMap<String, Value>,
) -> Vec<String> {
    config
        .primary_identifier
        .iter()
        .filter(|attr| {
            attributes
                .get(attr.dsl_name)
                .and_then(primary_identifier_segment)
                .is_none()
        })
        .map(|attr| attr.provider_name.to_string())
        .collect()
}

fn primary_identifier_segment(value: &Value) -> Option<String> {
    match value {
        Value::Concrete(ConcreteValue::String(value)) if !value.expose_plaintext().is_empty() => {
            Some(value.expose_plaintext().to_string())
        }
        Value::Concrete(ConcreteValue::CanonicalEnum(value)) => Some(value.api_value().to_string()),
        Value::Concrete(ConcreteValue::Int(value)) => Some(value.to_string()),
        Value::Concrete(ConcreteValue::Float(value)) => Some(value.to_string()),
        Value::Concrete(ConcreteValue::Bool(value)) => Some(value.to_string()),
        _ => None,
    }
}

fn merge_desired_attributes(
    read_attributes: &HashMap<String, Value>,
    resource: &Resource,
    config: &crate::schemas::config::AwsccSchemaConfig,
) -> HashMap<String, Value> {
    let mut attributes = read_attributes.clone();
    // Preserve desired attributes not returned by CloudControl API.
    // CloudControl doesn't always return all properties in GetResource responses
    // (create-only properties, and some normal properties like `description`).
    // Carry them forward from the desired state.
    for dsl_name in config.schema.attributes.keys() {
        if !attributes.contains_key(dsl_name)
            && let Some(value) = resource.get_attr(dsl_name.as_str())
        {
            attributes.insert(dsl_name.to_string(), value.clone());
        }
    }
    attributes
}

/// Merge the post-update desired attributes into a read-back and classify the outcome.
///
/// When the handler reported failure, no operation was dropped before sending, and every
/// confirmation-required sent attribute is present in the read-back, this deliberately
/// returns success and drops the handler status to preserve pre-existing behavior. Value
/// divergence, including a read-back of the old value, is the differ's responsibility
/// because confirmation here is presence-only.
fn update_outcome_from_read_back(
    id: &ResourceId,
    identifier: &AppliedIdentifier,
    read_back: ProviderResult<ReadOutcome>,
    desired: HashMap<String, Value>,
    patch_plan: &UpdatePatchPlan,
    config: &AwsccSchemaConfig,
    handler_failure: Option<&str>,
) -> UpdateOutcome {
    let mut attributes = match read_back {
        Ok(ReadOutcome::Found { state }) => state.attributes().clone(),
        Ok(ReadOutcome::NotFound { .. }) => {
            return update_partial_without_read_back(
                id,
                identifier,
                desired,
                patch_plan,
                read_back_reason(
                    handler_failure,
                    "post-update read-back did not find the updated resource",
                ),
            );
        }
        Err(read_error) => {
            return update_partial_without_read_back(
                id,
                identifier,
                desired,
                patch_plan,
                read_back_reason(handler_failure, format!("read error: {read_error}")),
            );
        }
    };

    // Cloud Control can report UPDATE SUCCESS while silently discarding a property:
    // AWS::S3::Bucket ObjectLockEnabled add-on-update returned OperationStatus SUCCESS
    // with no error while GetResource afterwards showed the property absent. Measured
    // ap-northeast-1, 2026-08-14 (issue #419). Attributes actually sent that the
    // read-back does not confirm, and requested attributes the provider could not send,
    // are therefore reported as partial success and never fabricated into state.
    let missing_sent_attributes = patch_plan
        .requires_confirmation
        .iter()
        .filter(|key| !patch_plan.unsent.contains(key) && !attributes.contains_key(key.as_str()))
        .cloned()
        .collect::<Vec<_>>();
    let unconfirmed_attributes = patch_plan
        .requires_confirmation
        .iter()
        .filter(|key| patch_plan.unsent.contains(key) || !attributes.contains_key(key.as_str()))
        .cloned()
        .collect::<Vec<_>>();

    for dsl_name in config.schema.attributes.keys() {
        if !attributes.contains_key(dsl_name)
            && !unconfirmed_attributes.contains(dsl_name)
            && let Some(value) = desired.get(dsl_name)
        {
            attributes.insert(dsl_name.clone(), value.clone());
        }
    }

    let state = AppliedState::from_identifier(id.clone(), identifier.clone(), attributes);

    if unconfirmed_attributes.is_empty() {
        return UpdateOutcome::Success { state };
    }

    let reason = update_confirmation_failure_reason(
        &missing_sent_attributes,
        &patch_plan.unsent,
        handler_failure,
    );
    UpdateOutcome::partial_success(state, reason, unconfirmed_attributes)
}

fn update_partial_without_read_back(
    id: &ResourceId,
    identifier: &AppliedIdentifier,
    mut desired: HashMap<String, Value>,
    patch_plan: &UpdatePatchPlan,
    mut reason: String,
) -> UpdateOutcome {
    let missing_attributes = nonempty_missing_attributes(patch_plan.touched.clone());
    for attr in &missing_attributes {
        desired.remove(attr);
    }
    if !patch_plan.unsent.is_empty() {
        reason.push_str("; ");
        reason.push_str(&unsent_attributes_reason(&patch_plan.unsent));
    }
    UpdateOutcome::partial_success(
        AppliedState::from_identifier(id.clone(), identifier.clone(), desired),
        reason,
        missing_attributes,
    )
}

fn update_confirmation_failure_reason(
    missing_sent_attributes: &[String],
    unsent_attributes: &[String],
    handler_failure: Option<&str>,
) -> String {
    let mut reasons = Vec::new();
    if let Some(status_message) = handler_failure {
        reasons.push(format!("handler failed: {status_message}"));
    }
    if !missing_sent_attributes.is_empty() {
        let absence_reason = format!(
            "the post-update read-back does not contain patched attributes: {}",
            missing_sent_attributes.join(", ")
        );
        reasons.push(if handler_failure.is_some() {
            absence_reason
        } else {
            format!("Cloud Control reported success, but {absence_reason}")
        });
    }
    if !unsent_attributes.is_empty() {
        reasons.push(unsent_attributes_reason(unsent_attributes));
    }
    reasons.join("; ")
}

fn unsent_attributes_reason(attributes: &[String]) -> String {
    format!(
        "update attributes were not sent to Cloud Control (schema marks the attribute read-only/create-only or the value is not convertible): {}",
        attributes.join(", ")
    )
}

/// Map a CloudControl `GetResource` properties payload onto the DSL
/// attribute map declared by `schema_attributes`.
///
/// CloudControl omits empty optional list/map fields from its read
/// response. If we treated "absent" as "untracked", the differ would
/// see `(none) → []` against a user that explicitly wrote `= []`,
/// producing a persistent plan diff (carina-rs/carina#2544).
///
/// This helper canonicalizes the read shape at the provider boundary:
/// when an optional list- or map-typed attribute is absent from the
/// AWS response, an empty `Value::List` / `Value::Map` is inserted in
/// its place. Scalars and structs are not synthesized — for them
/// "absent" really means "untracked", and downstream carry-forward
/// reuses the saved/desired value.
///
/// Tags are skipped here because they go through a dedicated parsing
/// path in [`AwsccProvider::read_resource`].
fn map_aws_props_to_attributes(
    props: &serde_json::Value,
    schema_attributes: &HashMap<String, AttributeSchema>,
    resource_type: &str,
    defs: &std::collections::BTreeMap<String, AttributeType>,
) -> HashMap<String, Value> {
    let mut attributes = HashMap::new();
    let schema_view = Schema::with_defs(defs.clone());

    for (dsl_name, attr_schema) in schema_attributes {
        if dsl_name == "tags" {
            continue;
        }
        let Some(aws_name) = &attr_schema.provider_name else {
            continue;
        };
        match props.get(aws_name.as_str()) {
            Some(value) => {
                if let Some(v) = aws_value_to_dsl_with_defs(
                    dsl_name,
                    value,
                    &attr_schema.attr_type,
                    resource_type,
                    defs,
                ) {
                    attributes.insert(dsl_name.clone(), v);
                }
            }
            None if !attr_schema.is_required() && !attr_schema.write_only => {
                match schema_view.shape_of(&attr_schema.attr_type) {
                    Shape::List { .. } => {
                        attributes.insert(
                            dsl_name.clone(),
                            Value::Concrete(ConcreteValue::List(Vec::new())),
                        );
                    }
                    Shape::Map { .. } => {
                        attributes.insert(
                            dsl_name.clone(),
                            Value::Concrete(ConcreteValue::Map(IndexMap::new())),
                        );
                    }
                    _ => {}
                }
            }
            None => {}
        }
    }

    attributes
}

/// Reconstruct the post-update desired-state attribute map: take the
/// current provider-side `from` state and apply each patch op on top.
///
/// Used by `update_resource` to know which attributes to carry forward
/// when CloudControl's read response omits them. The map is the same
/// logical shape as the old `to: Resource.attributes`, but built
/// without exposing a full `Resource` to the update path.
fn post_update_attributes(
    from: &State,
    patch: &UpdatePatch,
) -> std::collections::HashMap<String, Value> {
    use carina_core::provider::PatchOpKind;

    let mut attributes = from.attributes.clone();
    for op in &patch.ops {
        match op.kind {
            PatchOpKind::Add | PatchOpKind::Replace => {
                if let Some(value) = &op.value {
                    attributes.insert(op.key.clone(), value.clone());
                } else {
                    attributes.remove(&op.key);
                }
            }
            PatchOpKind::Remove => {
                attributes.remove(&op.key);
            }
        }
    }
    attributes
}

#[cfg(test)]
mod tests {
    use super::*;
    use carina_core::provider::{
        AppliedIdentifier, CreateFailureKind, CreateOutcome, CreateOutcomeKind, Observation,
        PatchOp, PatchOpKind, ReadOutcome, UpdateOutcome,
    };
    use carina_core::resource::{DeferredValue, UnknownReason};
    use carina_core::schema::AttributeType;
    use serde_json::json;
    use std::future::Future;
    use std::pin::Pin;
    use std::sync::Arc;
    use std::sync::Mutex;
    use std::sync::atomic::{AtomicBool, Ordering};
    use winterbaume_core::{MockAws, MockRequest, MockResponse, MockService};

    const WEB_ACL_ASSOCIATION_RESOURCE_ARN: &str = "arn:aws:elasticloadbalancing:ap-northeast-1:123456789012:loadbalancer/app/publish-api/abc123";
    const WEB_ACL_ASSOCIATION_WEB_ACL_ARN: &str =
        "arn:aws:wafv2:ap-northeast-1:123456789012:regional/webacl/publish-api/def456";

    fn test_string(value: &str) -> Value {
        Value::Concrete(ConcreteValue::String(value.to_string().into()))
    }

    fn test_identifier(value: &str) -> AppliedIdentifier {
        AppliedIdentifier::new(value).expect("test provider identifier must be non-empty")
    }

    async fn dispatch_test_create(
        provider: &AwsccProvider,
        resource: Resource,
    ) -> Result<CreateOutcome, CreateFailure> {
        let id = resource.id.clone();
        let bindings = carina_core::binding_index::ResolvedBindings::default();
        let module_gate = carina_core::executor::ModuleConstraintGate::new(&[]);
        let mut schemas = carina_core::schema::SchemaRegistry::new();
        for schema in crate::schemas::all_schemas() {
            schemas.insert("awscc", schema);
        }
        let preparation = carina_core::executor::ProviderPreparationContext::new(
            &bindings,
            &module_gate,
            &[],
            &crate::AwsccNormalizer,
            &[],
            &schemas,
        );
        let request = carina_core::executor::prepare_create_request(resource, &preparation)
            .await
            .expect("test resource should pass checked create preparation");
        carina_core::executor::dispatch_create(provider, &id, request).await
    }

    #[tokio::test]
    async fn native_driver_maps_over_budget_to_provider_timeout() {
        let id = ResourceId::with_provider_identity("awscc", "ec2.Vpc", "budget", None);
        let error = drive_operation(
            &id,
            OperationKind::Create,
            Duration::ZERO,
            std::future::ready(Ok::<_, ProviderError>(OperationStep::Pending {
                pending: (),
                poll_after_secs: 5,
            })),
            |_| std::future::ready(Ok(OperationStep::Done(()))),
            |_| std::future::ready(()),
        )
        .await
        .expect_err("pending operation with zero budget must time out");

        assert_eq!(error.variant_name(), "timeout");
        assert!(
            error
                .to_string()
                .contains("create of ec2.Vpc did not finish within 0ns")
        );
    }

    fn web_acl_association_state(
        entries: impl IntoIterator<Item = (&'static str, Value)>,
    ) -> State {
        let id = ResourceId::with_provider_identity(
            "awscc",
            "wafv2.WebAclAssociation",
            "publish_api_waf",
            None,
        );
        State::existing(
            id,
            entries
                .into_iter()
                .map(|(key, value)| (key.to_string(), value))
                .collect(),
        )
    }

    fn web_acl_association_config() -> AwsccSchemaConfig {
        crate::schemas::generated::wafv2::web_acl_association::wafv2_web_acl_association_config()
    }

    #[test]
    fn canonicalize_identifier_from_read_joins_composite_segments() {
        let config = web_acl_association_config();
        let state = web_acl_association_state([
            (
                "resource_arn",
                test_string(WEB_ACL_ASSOCIATION_RESOURCE_ARN),
            ),
            ("web_acl_arn", test_string(WEB_ACL_ASSOCIATION_WEB_ACL_ARN)),
        ]);

        let identifier = canonicalize_identifier_from_read(&config, &state.id, &state.attributes)
            .expect("canonicalization should succeed")
            .expect("both primary identifier attributes are present");
        assert_eq!(
            identifier.as_str(),
            format!(
                "{}|{}",
                WEB_ACL_ASSOCIATION_RESOURCE_ARN, WEB_ACL_ASSOCIATION_WEB_ACL_ARN
            )
        );
    }

    #[test]
    fn canonicalize_identifier_from_read_returns_none_for_incomplete_composite_identifier() {
        let config = web_acl_association_config();
        let missing_segment = web_acl_association_state([(
            "resource_arn",
            test_string(WEB_ACL_ASSOCIATION_RESOURCE_ARN),
        )]);
        let unknown_segment = web_acl_association_state([
            (
                "resource_arn",
                test_string(WEB_ACL_ASSOCIATION_RESOURCE_ARN),
            ),
            (
                "web_acl_arn",
                Value::Deferred(DeferredValue::Unknown(UnknownReason::ForValue)),
            ),
        ]);

        assert!(
            canonicalize_identifier_from_read(
                &config,
                &missing_segment.id,
                &missing_segment.attributes,
            )
            .expect("missing primary identifier attributes are not provider errors")
            .is_none()
        );
        assert!(
            canonicalize_identifier_from_read(
                &config,
                &unknown_segment.id,
                &unknown_segment.attributes,
            )
            .expect("unknown primary identifier attributes are not provider errors")
            .is_none()
        );
    }

    #[test]
    fn canonicalize_identifier_from_read_rejects_empty_canonical_identifier() {
        let mut config = web_acl_association_config();
        config.primary_identifier = &[];
        let state = web_acl_association_state([]);

        let error = canonicalize_identifier_from_read(&config, &state.id, &state.attributes)
            .expect_err("an empty canonical identifier must be a provider error");

        assert!(error.to_string().contains("empty canonical identifier"));
    }

    #[derive(Debug)]
    struct PartialCreateCloudControlService {
        wait_succeeds: bool,
        read_succeeds: bool,
        read_not_found: bool,
        saw_create_resource: AtomicBool,
        saw_get_resource: AtomicBool,
    }

    impl PartialCreateCloudControlService {
        fn new(read_succeeds: bool) -> Arc<Self> {
            Self::with_wait_result(false, read_succeeds)
        }

        fn with_wait_result(wait_succeeds: bool, read_succeeds: bool) -> Arc<Self> {
            Arc::new(Self {
                wait_succeeds,
                read_succeeds,
                read_not_found: false,
                saw_create_resource: AtomicBool::new(false),
                saw_get_resource: AtomicBool::new(false),
            })
        }

        fn with_not_found(wait_succeeds: bool) -> Arc<Self> {
            Arc::new(Self {
                wait_succeeds,
                read_succeeds: false,
                read_not_found: true,
                saw_create_resource: AtomicBool::new(false),
                saw_get_resource: AtomicBool::new(false),
            })
        }
    }

    impl MockService for PartialCreateCloudControlService {
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
                    .and_then(|v| v.to_str().ok())
                    .and_then(|v| v.split('.').next_back())
                    .unwrap_or_default();
                match action {
                    "CreateResource" => {
                        self.saw_create_resource.store(true, Ordering::SeqCst);
                        MockResponse::json(
                            200,
                            r#"{"ProgressEvent":{"RequestToken":"request-token-1"}}"#,
                        )
                    }
                    "GetResourceRequestStatus" if self.wait_succeeds => MockResponse::json(
                        200,
                        r#"{"ProgressEvent":{"RequestToken":"request-token-1","OperationStatus":"SUCCESS","Identifier":"partial-bucket"}}"#,
                    ),
                    "GetResourceRequestStatus" => MockResponse::json(
                        200,
                        r#"{"ProgressEvent":{"RequestToken":"request-token-1","OperationStatus":"FAILED","Identifier":"partial-bucket","StatusMessage":"post-create read denied"}}"#,
                    ),
                    "GetResource" if self.read_not_found => {
                        self.saw_get_resource.store(true, Ordering::SeqCst);
                        MockResponse::json(
                            404,
                            r#"{"__type":"ResourceNotFoundException","message":"bucket not found"}"#,
                        )
                    }
                    "GetResource" if self.read_succeeds => {
                        self.saw_get_resource.store(true, Ordering::SeqCst);
                        MockResponse::json(
                            200,
                            r#"{"TypeName":"AWS::S3::Bucket","ResourceDescription":{"Identifier":"partial-bucket","Properties":"{}"}}"#,
                        )
                    }
                    "GetResource" => {
                        self.saw_get_resource.store(true, Ordering::SeqCst);
                        MockResponse::json(
                            403,
                            r#"{"__type":"AccessDeniedException","message":"read denied"}"#,
                        )
                    }
                    _ => MockResponse::json(
                        400,
                        format!(
                            r#"{{"__type":"InvalidAction","message":"unexpected action {action}"}}"#
                        ),
                    ),
                }
            })
        }
    }

    async fn provider_with_partial_create_service(
        service: Arc<PartialCreateCloudControlService>,
    ) -> AwsccProvider {
        use aws_config::{BehaviorVersion, Region};

        let mock = MockAws::builder().with_service(service).build();
        let config = aws_config::defaults(BehaviorVersion::latest())
            .http_client(mock.http_client())
            .credentials_provider(mock.credentials_provider())
            .region(Region::new("us-east-1"))
            .load()
            .await;

        AwsccProvider::from_sdk_config(config, &super::super::AwsccProviderConfig::default()).await
    }

    struct ErrorTestCreateAttempt;

    struct ErrorTestIssuedCreate {
        identifier: AppliedIdentifier,
    }

    impl CcCreateAttempt for ErrorTestCreateAttempt {
        type Step = ();
        type Verdict = ProviderError;
        type Issued = ErrorTestIssuedCreate;

        fn untracked(self, error: ProviderError) -> Self::Verdict {
            error
        }

        fn issued(self, identifier: AppliedIdentifier, _sent: Sent) -> Self::Issued {
            ErrorTestIssuedCreate { identifier }
        }

        fn pending(
            self,
            _token: CcCreateToken,
            _sent: Sent,
            _poll_after_secs: u32,
        ) -> Result<Self::Step, Self::Verdict> {
            Ok(())
        }
    }

    impl CcIssuedCreate for ErrorTestIssuedCreate {
        type Step = ();
        type Verdict = ProviderError;

        fn identifier(&self) -> &AppliedIdentifier {
            &self.identifier
        }

        fn succeeded(
            self,
            _identifier: AppliedIdentifier,
            _attributes: HashMap<String, Value>,
        ) -> Self::Step {
        }

        fn partial(
            self,
            _identifier: AppliedIdentifier,
            _attributes: HashMap<String, Value>,
            _reason: String,
            _missing_attributes: Vec<String>,
        ) -> Self::Step {
        }

        fn failed(
            self,
            _identifier: AppliedIdentifier,
            _observation: Observation,
            error: ProviderError,
        ) -> Self::Verdict {
            error
        }

        fn discarded(self, error: ProviderError) -> Self::Verdict {
            error
        }

        fn pending(
            self,
            _token: CcCreateToken,
            _poll_after_secs: u32,
        ) -> Result<Self::Step, Self::Verdict> {
            Ok(())
        }
    }

    fn s3_bucket_resource(tags: Option<Value>) -> Resource {
        let resource = Resource::with_provider("awscc", "s3.Bucket", "invalid_tags", None)
            .with_attribute("bucket_name", test_string("carina-test-invalid-tags"));
        match tags {
            Some(tags) => resource.with_attribute("tags", tags),
            None => resource,
        }
    }

    #[tokio::test]
    async fn create_rejects_non_string_tag_value() {
        let service = PartialCreateCloudControlService::new(true);
        let provider = provider_with_partial_create_service(service.clone()).await;
        let resource =
            s3_bucket_resource(Some(Value::Concrete(ConcreteValue::Map(IndexMap::from([
                (
                    "env".to_string(),
                    Value::Concrete(ConcreteValue::Bool(true)),
                ),
            ])))));

        let error = provider
            .create_step(&resource, ErrorTestCreateAttempt)
            .await
            .expect_err("a non-string tag value must fail before create is sent");

        assert_eq!(error.variant_name(), "invalid_input");
        assert!(error.to_string().contains("`tags`"));
        assert!(error.to_string().contains("s3.Bucket"));
        assert!(!service.saw_create_resource.load(Ordering::SeqCst));
    }

    #[tokio::test]
    async fn create_desired_state_omits_absent_or_empty_key_value_list_tags() {
        let provider =
            provider_with_partial_create_service(PartialCreateCloudControlService::new(true)).await;
        let config =
            get_schema_config("s3.Bucket").expect("s3.Bucket schema configuration should exist");
        let resources = [
            ("absent", s3_bucket_resource(None)),
            (
                "empty",
                s3_bucket_resource(Some(Value::Concrete(ConcreteValue::Map(IndexMap::new())))),
            ),
        ];

        for (label, resource) in resources {
            let desired_state = provider
                .create_desired_state(&resource, config)
                .expect("absent or empty tags should be encodable");
            assert!(
                desired_state.get("Tags").is_none(),
                "{label} tags must not emit a Tags property"
            );
        }
    }

    fn ssm_parameter_resource(tags: Option<Value>) -> Resource {
        let resource = Resource::with_provider("awscc", "ssm.Parameter", "invalid_tags", None)
            .with_attribute("name", test_string("/carina/test/invalid-tags"))
            .with_attribute("type", test_string("String"))
            .with_attribute("value", test_string("test"));
        match tags {
            Some(tags) => resource.with_attribute("tags", tags),
            None => resource,
        }
    }

    #[tokio::test]
    async fn create_desired_state_omits_absent_or_empty_map_tags() {
        let provider =
            provider_with_partial_create_service(PartialCreateCloudControlService::new(true)).await;
        let config = get_schema_config("ssm.Parameter")
            .expect("ssm.Parameter schema configuration should exist");
        let resources = [
            ("absent", ssm_parameter_resource(None)),
            (
                "empty",
                ssm_parameter_resource(Some(Value::Concrete(ConcreteValue::Map(IndexMap::new())))),
            ),
        ];

        for (label, resource) in resources {
            let desired_state = provider
                .create_desired_state(&resource, config)
                .expect("absent or empty tags should be encodable");
            assert!(
                desired_state.get("Tags").is_none(),
                "{label} tags must not emit a Tags property"
            );
        }
    }

    #[tokio::test]
    async fn read_resource_accepts_pending_identity_with_applied_identifier() {
        let service = PartialCreateCloudControlService::new(true);
        let provider = provider_with_partial_create_service(service.clone()).await;
        let id = ResourceId::pending_with_provider("awscc", "s3.Bucket", None);
        let identifier = test_identifier("partial-bucket");

        let outcome = provider
            .read_resource(&id, &identifier)
            .await
            .expect("pending identity must not prevent a provider read");

        let ReadOutcome::Found { state } = outcome else {
            panic!("expected the real identifier to be read")
        };
        assert_eq!(state.id(), &id);
        assert!(state.id().identity().is_none());
        assert_eq!(state.identifier(), &identifier);
        assert!(service.saw_get_resource.load(Ordering::SeqCst));
    }

    #[tokio::test]
    async fn read_resource_still_enforces_initialization_error_for_pending_identity() {
        let service = PartialCreateCloudControlService::new(true);
        let mut provider = provider_with_partial_create_service(service.clone()).await;
        provider.init_error = Some("provider initialization failed".to_string());
        let id = ResourceId::pending_with_provider("awscc", "s3.Bucket", None);
        let identifier = test_identifier("partial-bucket");

        let error = provider
            .read_resource(&id, &identifier)
            .await
            .expect_err("initialization error must still short-circuit provider reads");

        assert_eq!(error.detail().context(), "provider initialization failed");
        assert!(!service.saw_get_resource.load(Ordering::SeqCst));
    }

    #[tokio::test]
    async fn read_data_source_reports_unsupported_type_without_managed_read() {
        let service = PartialCreateCloudControlService::new(true);
        let mut provider = provider_with_partial_create_service(service.clone()).await;
        provider.init_error = Some("provider initialization failed".to_string());
        let id = ResourceId::pending_with_provider("awscc", "s3.Bucket", None);

        let error = provider
            .read_data_source_resource(&id)
            .await
            .expect_err("awscc must reject data-source reads");

        assert!(
            error
                .to_string()
                .contains("awscc does not provide data sources; requested type s3.Bucket")
        );
        assert!(!service.saw_get_resource.load(Ordering::SeqCst));
    }

    #[tokio::test]
    async fn create_outcome_preserves_wait_identifier_when_canonicalization_fails() {
        let service = PartialCreateCloudControlService::new(true);
        let provider = provider_with_partial_create_service(service).await;
        let resource = partial_create_bucket_resource();
        let wait_identifier = test_identifier("partial-bucket");
        let mut config = crate::schemas::generated::s3::bucket::s3_bucket_config();
        config.primary_identifier = &[];
        let read_back = ReadOutcome::found(AppliedState::from_identifier(
            resource.id.clone(),
            wait_identifier.clone(),
            HashMap::from([("bucket_name".to_string(), test_string("partial-bucket"))]),
        ));

        let outcome = provider
            .create_completion_from_read(&resource, &config, wait_identifier.clone(), Ok(read_back))
            .await;

        let CreateCompletion::Partial {
            identifier,
            attributes,
            reason,
            missing_attributes: _,
        } = outcome
        else {
            panic!("canonicalization failure must preserve the created resource");
        };
        assert_eq!(identifier, wait_identifier);
        assert_eq!(
            attributes.get("bucket_name"),
            Some(&test_string("partial-bucket"))
        );
        assert!(reason.contains("empty canonical identifier"));
    }

    #[derive(Clone, Copy, Debug)]
    enum IamRoleReadMode {
        WithRoleName,
        WithoutRoleName,
        NotFound,
    }

    #[derive(Debug)]
    struct IamRoleCreateCloudControlService {
        read_mode: IamRoleReadMode,
        wait_succeeds: bool,
        saw_create_resource: AtomicBool,
        saw_get_resource: AtomicBool,
    }

    impl IamRoleCreateCloudControlService {
        fn new(include_role_name_in_read: bool) -> Arc<Self> {
            let read_mode = if include_role_name_in_read {
                IamRoleReadMode::WithRoleName
            } else {
                IamRoleReadMode::WithoutRoleName
            };
            Self::with_read_mode(read_mode, true)
        }

        fn with_read_mode(read_mode: IamRoleReadMode, wait_succeeds: bool) -> Arc<Self> {
            Arc::new(Self {
                read_mode,
                wait_succeeds,
                saw_create_resource: AtomicBool::new(false),
                saw_get_resource: AtomicBool::new(false),
            })
        }
    }

    impl MockService for IamRoleCreateCloudControlService {
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
                    .and_then(|v| v.to_str().ok())
                    .and_then(|v| v.split('.').next_back())
                    .unwrap_or_default();
                match action {
                    "CreateResource" => {
                        self.saw_create_resource.store(true, Ordering::SeqCst);
                        MockResponse::json(
                            200,
                            r#"{"ProgressEvent":{"RequestToken":"iam-role-token"}}"#,
                        )
                    }
                    "GetResourceRequestStatus" if self.wait_succeeds => MockResponse::json(
                        200,
                        r#"{"ProgressEvent":{"RequestToken":"iam-role-token","OperationStatus":"SUCCESS","Identifier":"flow-log-role"}}"#,
                    ),
                    "GetResourceRequestStatus" => MockResponse::json(
                        200,
                        r#"{"ProgressEvent":{"RequestToken":"iam-role-token","OperationStatus":"FAILED","Identifier":"flow-log-role","StatusMessage":"post-create validation failed"}}"#,
                    ),
                    "GetResource" => {
                        self.saw_get_resource.store(true, Ordering::SeqCst);
                        let properties = match self.read_mode {
                            IamRoleReadMode::WithRoleName => {
                                r#"{"Path":"/service-role/","RoleName":"flow-log-role","RoleId":"AROATESTROLEID"}"#
                            }
                            IamRoleReadMode::WithoutRoleName => {
                                r#"{"Path":"/service-role/","RoleId":"AROATESTROLEID"}"#
                            }
                            IamRoleReadMode::NotFound => {
                                return MockResponse::json(
                                    404,
                                    r#"{"__type":"ResourceNotFoundException","message":"role not found"}"#,
                                );
                            }
                        };
                        MockResponse::json(
                            200,
                            format!(
                                r#"{{"TypeName":"AWS::IAM::Role","ResourceDescription":{{"Identifier":"flow-log-role","Properties":{properties:?}}}}}"#
                            ),
                        )
                    }
                    _ => MockResponse::json(
                        400,
                        format!(
                            r#"{{"__type":"InvalidAction","message":"unexpected action {action}"}}"#
                        ),
                    ),
                }
            })
        }
    }

    #[derive(Debug)]
    struct CallerIdentityService {
        succeeds: bool,
        saw_get_caller_identity: AtomicBool,
    }

    impl CallerIdentityService {
        fn new() -> Arc<Self> {
            Self::with_result(true)
        }

        fn failing() -> Arc<Self> {
            Self::with_result(false)
        }

        fn with_result(succeeds: bool) -> Arc<Self> {
            Arc::new(Self {
                succeeds,
                saw_get_caller_identity: AtomicBool::new(false),
            })
        }
    }

    impl MockService for CallerIdentityService {
        fn service_name(&self) -> &str {
            "sts"
        }

        fn url_patterns(&self) -> Vec<&str> {
            vec![
                r"https?://sts\..*\.amazonaws\.com",
                r"https?://sts\.amazonaws\.com",
            ]
        }

        fn handle(
            &self,
            _request: MockRequest,
        ) -> Pin<Box<dyn Future<Output = MockResponse> + Send + '_>> {
            Box::pin(async move {
                self.saw_get_caller_identity.store(true, Ordering::SeqCst);
                if !self.succeeds {
                    return MockResponse::xml(
                        403,
                        r#"<ErrorResponse xmlns="https://sts.amazonaws.com/doc/2011-06-15/">
  <Error>
    <Type>Sender</Type>
    <Code>AccessDenied</Code>
    <Message>synthesis denied</Message>
  </Error>
  <RequestId>00000000-0000-0000-0000-000000000000</RequestId>
</ErrorResponse>"#,
                    );
                }
                MockResponse::xml(
                    200,
                    r#"<GetCallerIdentityResponse xmlns="https://sts.amazonaws.com/doc/2011-06-15/">
  <GetCallerIdentityResult>
    <Arn>arn:aws-us-gov:iam::123456789012:user/test</Arn>
    <UserId>AIDATESTUSER</UserId>
    <Account>123456789012</Account>
  </GetCallerIdentityResult>
  <ResponseMetadata>
    <RequestId>00000000-0000-0000-0000-000000000000</RequestId>
  </ResponseMetadata>
</GetCallerIdentityResponse>"#,
                )
            })
        }
    }

    async fn provider_with_iam_role_create_service(
        cloudcontrol: Arc<IamRoleCreateCloudControlService>,
        sts: Arc<CallerIdentityService>,
    ) -> AwsccProvider {
        use aws_config::{BehaviorVersion, Region};

        let mock = MockAws::builder()
            .with_service(cloudcontrol)
            .with_service(sts)
            .build();
        let config = aws_config::defaults(BehaviorVersion::latest())
            .http_client(mock.http_client())
            .credentials_provider(mock.credentials_provider())
            .region(Region::new("us-gov-west-1"))
            .load()
            .await;

        AwsccProvider::from_sdk_config(config, &super::super::AwsccProviderConfig::default()).await
    }

    #[derive(Debug)]
    struct PartialUpdateCloudControlService {
        wait_succeeds: bool,
        read_succeeds: bool,
        read_not_found: bool,
        read_properties: &'static str,
        saw_update_resource: AtomicBool,
        saw_get_resource: AtomicBool,
        update_request_body: Mutex<Option<String>>,
    }

    impl PartialUpdateCloudControlService {
        fn new(wait_succeeds: bool, read_succeeds: bool) -> Arc<Self> {
            Arc::new(Self {
                wait_succeeds,
                read_succeeds,
                read_not_found: false,
                read_properties: r#"{"Tags":[{"Key":"env","Value":"prod"}]}"#,
                saw_update_resource: AtomicBool::new(false),
                saw_get_resource: AtomicBool::new(false),
                update_request_body: Mutex::new(None),
            })
        }

        fn with_read_properties(wait_succeeds: bool, read_properties: &'static str) -> Arc<Self> {
            Arc::new(Self {
                wait_succeeds,
                read_succeeds: true,
                read_not_found: false,
                read_properties,
                saw_update_resource: AtomicBool::new(false),
                saw_get_resource: AtomicBool::new(false),
                update_request_body: Mutex::new(None),
            })
        }

        fn with_not_found(wait_succeeds: bool) -> Arc<Self> {
            Arc::new(Self {
                wait_succeeds,
                read_succeeds: false,
                read_not_found: true,
                read_properties: "{}",
                saw_update_resource: AtomicBool::new(false),
                saw_get_resource: AtomicBool::new(false),
                update_request_body: Mutex::new(None),
            })
        }
    }

    impl MockService for PartialUpdateCloudControlService {
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
                    .and_then(|v| v.to_str().ok())
                    .and_then(|v| v.split('.').next_back())
                    .unwrap_or_default();
                match action {
                    "UpdateResource" => {
                        self.saw_update_resource.store(true, Ordering::SeqCst);
                        *self
                            .update_request_body
                            .lock()
                            .expect("update request body lock poisoned") =
                            Some(String::from_utf8_lossy(&request.body).into_owned());
                        MockResponse::json(
                            200,
                            r#"{"ProgressEvent":{"RequestToken":"update-token-1"}}"#,
                        )
                    }
                    "GetResourceRequestStatus" if self.wait_succeeds => MockResponse::json(
                        200,
                        r#"{"ProgressEvent":{"RequestToken":"update-token-1","OperationStatus":"SUCCESS","Identifier":"partial-bucket"}}"#,
                    ),
                    "GetResourceRequestStatus" => MockResponse::json(
                        200,
                        r#"{"ProgressEvent":{"RequestToken":"update-token-1","OperationStatus":"FAILED","Identifier":"partial-bucket","StatusMessage":"post-update read denied"}}"#,
                    ),
                    "GetResource" if self.read_not_found => {
                        self.saw_get_resource.store(true, Ordering::SeqCst);
                        MockResponse::json(
                            404,
                            r#"{"__type":"ResourceNotFoundException","message":"bucket not found"}"#,
                        )
                    }
                    "GetResource" if self.read_succeeds => {
                        self.saw_get_resource.store(true, Ordering::SeqCst);
                        MockResponse::json(
                            200,
                            format!(
                                r#"{{"TypeName":"AWS::S3::Bucket","ResourceDescription":{{"Identifier":"partial-bucket","Properties":{:?}}}}}"#,
                                self.read_properties
                            ),
                        )
                    }
                    "GetResource" => {
                        self.saw_get_resource.store(true, Ordering::SeqCst);
                        MockResponse::json(
                            403,
                            r#"{"__type":"AccessDeniedException","message":"read denied"}"#,
                        )
                    }
                    _ => MockResponse::json(
                        400,
                        format!(
                            r#"{{"__type":"InvalidAction","message":"unexpected action {action}"}}"#
                        ),
                    ),
                }
            })
        }
    }

    async fn provider_with_partial_update_service(
        service: Arc<PartialUpdateCloudControlService>,
    ) -> AwsccProvider {
        use aws_config::{BehaviorVersion, Region};

        let mock = MockAws::builder().with_service(service).build();
        let config = aws_config::defaults(BehaviorVersion::latest())
            .http_client(mock.http_client())
            .credentials_provider(mock.credentials_provider())
            .region(Region::new("us-east-1"))
            .load()
            .await;

        AwsccProvider::from_sdk_config(config, &super::super::AwsccProviderConfig::default()).await
    }

    fn partial_update_bucket_state() -> State {
        State::existing(
            ResourceId::with_provider_identity("awscc", "s3.Bucket", "partial_bucket", None),
            HashMap::from([(
                "bucket_name".to_string(),
                Value::Concrete(ConcreteValue::String("partial-bucket".to_string().into())),
            )]),
        )
        .with_identifier("partial-bucket")
    }

    fn partial_update_bucket_patch() -> UpdatePatch {
        let mut tags = indexmap::IndexMap::new();
        tags.insert(
            "env".to_string(),
            Value::Concrete(ConcreteValue::String("prod".to_string().into())),
        );
        UpdatePatch {
            ops: vec![PatchOp {
                kind: PatchOpKind::Replace,
                key: "tags".to_string(),
                value: Some(Value::Concrete(ConcreteValue::Map(tags))),
            }],
        }
    }

    fn create_only_bucket_name_patch() -> UpdatePatch {
        let mut patch = partial_update_bucket_patch();
        patch.ops.insert(
            0,
            PatchOp {
                kind: PatchOpKind::Replace,
                key: "bucket_name".to_string(),
                value: Some(Value::Concrete(ConcreteValue::String(
                    "renamed-bucket".to_string().into(),
                ))),
            },
        );
        patch
    }

    fn empty_tags_patch() -> UpdatePatch {
        UpdatePatch {
            ops: vec![PatchOp {
                kind: PatchOpKind::Replace,
                key: "tags".to_string(),
                value: Some(Value::Concrete(ConcreteValue::Map(IndexMap::new()))),
            }],
        }
    }

    fn object_lock_enabled_patch() -> UpdatePatch {
        UpdatePatch {
            ops: vec![PatchOp {
                kind: PatchOpKind::Add,
                key: "object_lock_enabled".to_string(),
                value: Some(Value::Concrete(ConcreteValue::Bool(true))),
            }],
        }
    }

    fn replace_object_lock_enabled_patch() -> UpdatePatch {
        UpdatePatch {
            ops: vec![PatchOp {
                kind: PatchOpKind::Replace,
                key: "object_lock_enabled".to_string(),
                value: Some(Value::Concrete(ConcreteValue::Bool(true))),
            }],
        }
    }

    fn access_control_patch() -> UpdatePatch {
        UpdatePatch {
            ops: vec![PatchOp {
                kind: PatchOpKind::Add,
                key: "access_control".to_string(),
                value: Some(Value::Concrete(ConcreteValue::String(
                    "private".to_string().into(),
                ))),
            }],
        }
    }

    #[tokio::test]
    async fn update_resource_returns_partial_success_when_successful_update_drops_patched_attribute()
     {
        let service = PartialUpdateCloudControlService::new(true, true);
        let provider = provider_with_partial_update_service(service.clone()).await;
        let from = partial_update_bucket_state();
        let patch = object_lock_enabled_patch();

        let outcome = provider
            .update_resource(
                from.id.clone(),
                &test_identifier("partial-bucket"),
                &from,
                &patch,
            )
            .await
            .expect("update should return a partial outcome");

        let UpdateOutcome::PartialSuccess { state, diagnostic } = outcome else {
            panic!("expected partial success when read-back omits a patched attribute");
        };
        assert!(service.saw_update_resource.load(Ordering::SeqCst));
        assert!(service.saw_get_resource.load(Ordering::SeqCst));
        assert!(
            !state.attributes().contains_key("object_lock_enabled"),
            "an unconfirmed patched value must not be fabricated into state"
        );
        assert_eq!(
            diagnostic.missing_attributes(),
            &["object_lock_enabled".to_string()]
        );
        assert_eq!(
            diagnostic.reason(),
            "Cloud Control reported success, but the post-update read-back does not contain patched attributes: object_lock_enabled"
        );
    }

    #[tokio::test]
    async fn update_resource_reports_unsent_create_only_attribute_as_partial_success() {
        for (read_properties, expected_bucket_name, label) in [
            (r#"{"Tags":[{"Key":"env","Value":"prod"}]}"#, None, "absent"),
            (
                r#"{"BucketName":"partial-bucket","Tags":[{"Key":"env","Value":"prod"}]}"#,
                Some("partial-bucket"),
                "present with its old value",
            ),
        ] {
            let service =
                PartialUpdateCloudControlService::with_read_properties(true, read_properties);
            let provider = provider_with_partial_update_service(service.clone()).await;
            let from = partial_update_bucket_state();
            let patch = create_only_bucket_name_patch();

            let outcome = provider
                .update_resource(
                    from.id.clone(),
                    &test_identifier("partial-bucket"),
                    &from,
                    &patch,
                )
                .await
                .expect("an unsent requested update should return a partial outcome");

            let UpdateOutcome::PartialSuccess { state, diagnostic } = outcome else {
                panic!("expected partial success when the create-only value is {label}");
            };
            assert!(service.saw_update_resource.load(Ordering::SeqCst));
            assert_eq!(
                diagnostic.missing_attributes(),
                &["bucket_name".to_string()]
            );
            assert_eq!(
                diagnostic.reason(),
                "update attributes were not sent to Cloud Control (schema marks the attribute read-only/create-only or the value is not convertible): bucket_name"
            );
            let actual_bucket_name = state.attributes().get("bucket_name").and_then(|value| {
                let Value::Concrete(ConcreteValue::String(name)) = value else {
                    return None;
                };
                Some(name.expose_plaintext())
            });
            assert_eq!(
                actual_bucket_name, expected_bucket_name,
                "read-back must remain honest when the create-only value is {label}"
            );
            assert_ne!(
                state.attributes().get("bucket_name"),
                Some(&Value::Concrete(ConcreteValue::String(
                    "renamed-bucket".to_string().into()
                )))
            );
        }
    }

    #[tokio::test]
    async fn update_resource_returns_success_when_read_back_confirms_patched_attribute() {
        let service = PartialUpdateCloudControlService::with_read_properties(
            true,
            r#"{"ObjectLockEnabled":true}"#,
        );
        let provider = provider_with_partial_update_service(service).await;
        let from = partial_update_bucket_state();
        let patch = object_lock_enabled_patch();

        let outcome = provider
            .update_resource(
                from.id.clone(),
                &test_identifier("partial-bucket"),
                &from,
                &patch,
            )
            .await
            .expect("update should succeed");

        let UpdateOutcome::Success { state } = outcome else {
            panic!("expected success when read-back confirms the patched attribute");
        };
        assert_eq!(
            state.attributes().get("object_lock_enabled"),
            Some(&Value::Concrete(ConcreteValue::Bool(true)))
        );
    }

    #[tokio::test]
    async fn update_resource_returns_success_when_failed_handler_read_back_has_old_patched_value() {
        let service = PartialUpdateCloudControlService::with_read_properties(
            false,
            r#"{"ObjectLockEnabled":false}"#,
        );
        let provider = provider_with_partial_update_service(service).await;
        let mut from = partial_update_bucket_state();
        from.attributes.insert(
            "object_lock_enabled".to_string(),
            Value::Concrete(ConcreteValue::Bool(false)),
        );
        let patch = replace_object_lock_enabled_patch();

        let outcome = provider
            .update_resource(
                from.id.clone(),
                &test_identifier("partial-bucket"),
                &from,
                &patch,
            )
            .await
            .expect("confirmed read-back should preserve pre-existing success behavior");

        let UpdateOutcome::Success { state } = outcome else {
            panic!("expected success when the patched attribute is present in read-back");
        };
        assert_eq!(
            state.attributes().get("object_lock_enabled"),
            Some(&Value::Concrete(ConcreteValue::Bool(false))),
            "the read-back value must win even when it differs from the patch"
        );
    }

    #[tokio::test]
    async fn update_resource_carries_forward_patched_write_only_attribute() {
        let service = PartialUpdateCloudControlService::new(true, true);
        let provider = provider_with_partial_update_service(service).await;
        let from = partial_update_bucket_state();
        let patch = access_control_patch();

        let outcome = provider
            .update_resource(
                from.id.clone(),
                &test_identifier("partial-bucket"),
                &from,
                &patch,
            )
            .await
            .expect("update should succeed");

        let UpdateOutcome::Success { state } = outcome else {
            panic!("expected success for an absent patched write-only attribute");
        };
        assert_eq!(
            state.attributes().get("access_control"),
            Some(&Value::Concrete(ConcreteValue::String(
                "private".to_string().into()
            )))
        );
    }

    #[tokio::test]
    async fn update_resource_carries_forward_unpatched_attribute_absent_from_read_back() {
        let service = PartialUpdateCloudControlService::new(true, true);
        let provider = provider_with_partial_update_service(service).await;
        let mut from = partial_update_bucket_state();
        from.attributes.insert(
            "object_lock_enabled".to_string(),
            Value::Concrete(ConcreteValue::Bool(false)),
        );
        let patch = partial_update_bucket_patch();

        let outcome = provider
            .update_resource(
                from.id.clone(),
                &test_identifier("partial-bucket"),
                &from,
                &patch,
            )
            .await
            .expect("update should succeed");

        let UpdateOutcome::Success { state } = outcome else {
            panic!("expected success when only an unpatched attribute is absent");
        };
        assert_eq!(
            state.attributes().get("object_lock_enabled"),
            Some(&Value::Concrete(ConcreteValue::Bool(false)))
        );
    }

    #[tokio::test]
    async fn update_resource_carries_unpatched_redacted_secret_without_sending_digest() {
        let service = PartialUpdateCloudControlService::new(true, true);
        let provider = provider_with_partial_update_service(service.clone()).await;
        let mut from = partial_update_bucket_state();
        let secret = Value::Deferred(DeferredValue::Secret(Box::new(Value::Concrete(
            ConcreteValue::String("stored-write-only-secret".into()),
        ))));
        let redacted = carina_core::value::redact_secrets_in_value(&secret)
            .expect("redact saved-state placeholder");
        let Value::Deferred(DeferredValue::RedactedSecret { hash }) = &redacted else {
            panic!("expected a redacted secret")
        };
        let digest = hash.encoded_digest();
        from.attributes
            .insert("access_control".to_string(), redacted.clone());

        let outcome = provider
            .update_resource(
                from.id.clone(),
                &test_identifier("partial-bucket"),
                &from,
                &partial_update_bucket_patch(),
            )
            .await
            .expect("update should succeed");

        let UpdateOutcome::Success { state } = outcome else {
            panic!("expected success when an unpatched secret is absent from read-back");
        };
        assert_eq!(state.attributes().get("access_control"), Some(&redacted));

        let request_body = service
            .update_request_body
            .lock()
            .expect("update request body lock poisoned")
            .clone()
            .expect("CloudControl update request should be captured");
        assert!(
            !request_body.contains(&digest),
            "persisted redacted-secret digests must never be sent to CloudControl"
        );
        assert!(
            !request_body.contains("AccessControl"),
            "an unpatched persisted attribute must not enter the patch document"
        );
    }

    #[tokio::test]
    async fn update_resource_returns_success_when_unset_attribute_is_absent_from_read_back() {
        for (kind, label) in [
            (PatchOpKind::Remove, "remove"),
            (PatchOpKind::Add, "add-none"),
            (PatchOpKind::Replace, "replace-none"),
        ] {
            let service = PartialUpdateCloudControlService::new(true, true);
            let provider = provider_with_partial_update_service(service).await;
            let mut from = partial_update_bucket_state();
            from.attributes.insert(
                "object_lock_enabled".to_string(),
                Value::Concrete(ConcreteValue::Bool(true)),
            );
            let patch = UpdatePatch {
                ops: vec![PatchOp {
                    kind,
                    key: "object_lock_enabled".to_string(),
                    value: None,
                }],
            };

            let outcome = provider
                .update_resource(
                    from.id.clone(),
                    &test_identifier("partial-bucket"),
                    &from,
                    &patch,
                )
                .await
                .expect("unset update should succeed");

            let UpdateOutcome::Success { state } = outcome else {
                panic!("expected success for {label} when the attribute is absent");
            };
            assert!(
                !state.attributes().contains_key("object_lock_enabled"),
                "unset attribute must remain absent for {label}"
            );
        }
    }

    #[tokio::test]
    async fn update_resource_returns_partial_success_when_failed_update_read_back_omits_patched_attribute()
     {
        let service = PartialUpdateCloudControlService::new(false, true);
        let provider = provider_with_partial_update_service(service).await;
        let from = partial_update_bucket_state();
        let patch = object_lock_enabled_patch();

        let outcome = provider
            .update_resource(
                from.id.clone(),
                &test_identifier("partial-bucket"),
                &from,
                &patch,
            )
            .await
            .expect("update should return a partial outcome");

        let UpdateOutcome::PartialSuccess { state, diagnostic } = outcome else {
            panic!("expected partial success when read-back omits a patched attribute");
        };
        assert!(!state.attributes().contains_key("object_lock_enabled"));
        assert_eq!(
            diagnostic.missing_attributes(),
            &["object_lock_enabled".to_string()]
        );
        assert_eq!(
            diagnostic.reason(),
            "handler failed: post-update read denied; the post-update read-back does not contain patched attributes: object_lock_enabled"
        );
    }

    #[tokio::test]
    async fn update_resource_returns_success_when_wait_success() {
        let service = PartialUpdateCloudControlService::new(true, true);
        let provider = provider_with_partial_update_service(service.clone()).await;
        let from = partial_update_bucket_state();
        let patch = partial_update_bucket_patch();

        let outcome = provider
            .update_resource(
                from.id.clone(),
                &test_identifier("partial-bucket"),
                &from,
                &patch,
            )
            .await
            .expect("update should succeed");

        let UpdateOutcome::Success { state } = outcome else {
            panic!("expected success when wait succeeds");
        };
        assert!(service.saw_update_resource.load(Ordering::SeqCst));
        assert!(service.saw_get_resource.load(Ordering::SeqCst));
        assert_eq!(state.identifier().as_str(), "partial-bucket");
        assert!(state.attributes().contains_key("tags"));
    }

    #[tokio::test]
    async fn update_resource_returns_success_when_clearing_all_tags() {
        let service = PartialUpdateCloudControlService::with_read_properties(true, r#"{}"#);
        let provider = provider_with_partial_update_service(service).await;
        let from = partial_update_bucket_state();
        let patch = empty_tags_patch();

        let outcome = provider
            .update_resource(
                from.id.clone(),
                &test_identifier("partial-bucket"),
                &from,
                &patch,
            )
            .await
            .expect("clearing all tags should succeed");

        let UpdateOutcome::Success { state } = outcome else {
            panic!("expected success when read-back omits the cleared tags property");
        };
        assert_eq!(
            state.attributes().get("tags"),
            Some(&Value::Concrete(ConcreteValue::Map(IndexMap::new())))
        );
    }

    #[tokio::test]
    async fn read_resource_canonicalizes_missing_or_empty_tags_to_empty_map() {
        for (read_properties, label) in [(r#"{}"#, "missing"), (r#"{"Tags":[]}"#, "empty")] {
            let service =
                PartialUpdateCloudControlService::with_read_properties(true, read_properties);
            let provider = provider_with_partial_update_service(service).await;
            let identifier = test_identifier("partial-bucket");

            let outcome = provider
                .read_resource(
                    &ResourceId::with_provider_identity(
                        "awscc",
                        "s3.Bucket",
                        "partial_bucket",
                        None,
                    ),
                    &identifier,
                )
                .await
                .expect("read should succeed");
            let ReadOutcome::Found { state } = outcome else {
                panic!("expected the bucket to be found")
            };

            assert_eq!(
                state.attributes().get("tags"),
                Some(&Value::Concrete(ConcreteValue::Map(IndexMap::new()))),
                "{label} Tags must use the canonical empty-map shape"
            );
        }
    }

    #[tokio::test]
    async fn read_resource_does_not_fabricate_tags_for_schema_without_tags_attribute() {
        let config = get_schema_config("route53.HostedZone")
            .expect("route53.HostedZone schema should exist");
        assert!(config.tags.is_none());
        assert!(!config.schema.attributes.contains_key("tags"));

        for (read_properties, label) in [
            (r#"{}"#, "missing"),
            (r#"{"Tags":[{"Key":"env","Value":"prod"}]}"#, "present"),
        ] {
            let service =
                PartialUpdateCloudControlService::with_read_properties(true, read_properties);
            let provider = provider_with_partial_update_service(service).await;
            let identifier = test_identifier("Z123456789");

            let outcome = provider
                .read_resource(
                    &ResourceId::with_provider_identity(
                        "awscc",
                        "route53.HostedZone",
                        "hosted_zone",
                        None,
                    ),
                    &identifier,
                )
                .await
                .expect("read should succeed");
            let ReadOutcome::Found { state } = outcome else {
                panic!("expected the hosted zone to be found")
            };

            assert!(
                !state.attributes().contains_key("tags"),
                "{label} Cloud Control Tags must not fabricate a schema-unknown attribute"
            );
        }
    }

    fn assert_structured_partial_read_error(reason: &str, status_message: &str) {
        assert!(
            reason.starts_with(&format!(
                "handler failed: {status_message}; read error: Failed to get resource"
            )),
            "unexpected partial-success reason: {reason}"
        );
        for expected in [
            "operation: cloudcontrol.GetResource",
            "status: 403 AccessDeniedException",
            "resource type AWS::S3::Bucket, identifier partial-bucket",
            "read denied",
        ] {
            assert!(
                reason.contains(expected),
                "partial-success reason lost `{expected}`: {reason}"
            );
        }
    }

    #[tokio::test]
    async fn update_resource_returns_partial_success_when_failed_identifier_read_fails() {
        let service = PartialUpdateCloudControlService::new(false, false);
        let provider = provider_with_partial_update_service(service.clone()).await;
        let from = partial_update_bucket_state();
        let patch = partial_update_bucket_patch();

        let outcome = provider
            .update_resource(
                from.id.clone(),
                &test_identifier("partial-bucket"),
                &from,
                &patch,
            )
            .await
            .expect("update should return a partial outcome instead of error");

        let UpdateOutcome::PartialSuccess { state, diagnostic } = outcome else {
            panic!("expected partial success when post-update read fails");
        };
        assert!(service.saw_update_resource.load(Ordering::SeqCst));
        assert!(
            service.saw_get_resource.load(Ordering::SeqCst),
            "provider must retry read with the identifier from failed progress"
        );
        assert_eq!(state.identifier().as_str(), "partial-bucket");
        assert_eq!(
            state.attributes().get("bucket_name"),
            Some(&Value::Concrete(ConcreteValue::String(
                "partial-bucket".to_string().into()
            )))
        );
        assert!(
            !state.attributes().contains_key("tags"),
            "missing authored attributes must be absent so the partial-read marker can materialize them as Unknown"
        );
        assert_structured_partial_read_error(diagnostic.reason(), "post-update read denied");
        assert_eq!(diagnostic.missing_attributes(), &["tags".to_string()]);
    }

    #[tokio::test]
    async fn update_resource_preserves_identifier_when_read_back_is_not_found() {
        let service = PartialUpdateCloudControlService::with_not_found(true);
        let provider = provider_with_partial_update_service(service.clone()).await;
        let from = partial_update_bucket_state();
        let identifier = test_identifier("partial-bucket");
        let patch = partial_update_bucket_patch();

        let outcome = provider
            .update_resource(from.id.clone(), &identifier, &from, &patch)
            .await
            .expect("update should return a partial outcome instead of losing its identifier");

        let UpdateOutcome::PartialSuccess { state, diagnostic } = outcome else {
            panic!("expected partial success when post-update read returns not_found");
        };
        assert!(service.saw_update_resource.load(Ordering::SeqCst));
        assert!(service.saw_get_resource.load(Ordering::SeqCst));
        assert_eq!(state.identifier(), &identifier);
        assert_eq!(
            state.attributes().get("bucket_name"),
            Some(&Value::Concrete(ConcreteValue::String(
                "partial-bucket".to_string().into()
            )))
        );
        assert!(!state.attributes().contains_key("tags"));
        assert_eq!(diagnostic.missing_attributes(), &["tags".to_string()]);
        assert_eq!(
            diagnostic.reason(),
            "post-update read-back did not find the updated resource"
        );
    }

    #[tokio::test]
    async fn update_resource_preserves_identifier_when_failed_wait_read_back_is_not_found() {
        let service = PartialUpdateCloudControlService::with_not_found(false);
        let provider = provider_with_partial_update_service(service.clone()).await;
        let from = partial_update_bucket_state();
        let identifier = test_identifier("partial-bucket");
        let patch = partial_update_bucket_patch();

        let outcome = provider
            .update_resource(from.id.clone(), &identifier, &from, &patch)
            .await
            .expect("update should return a partial outcome instead of losing its identifier");

        let UpdateOutcome::PartialSuccess { state, diagnostic } = outcome else {
            panic!("expected partial success when failed-update read-back returns not_found");
        };
        assert!(service.saw_update_resource.load(Ordering::SeqCst));
        assert!(service.saw_get_resource.load(Ordering::SeqCst));
        assert_eq!(state.identifier(), &identifier);
        assert_eq!(
            state.attributes().get("bucket_name"),
            Some(&test_string("partial-bucket"))
        );
        assert!(!state.attributes().contains_key("tags"));
        assert_eq!(diagnostic.missing_attributes(), &["tags".to_string()]);
        assert_eq!(
            diagnostic.reason(),
            "handler failed: post-update read denied; post-update read-back did not find the updated resource"
        );
    }

    #[tokio::test]
    async fn update_resource_preserves_identifier_when_successful_wait_read_back_fails() {
        let service = PartialUpdateCloudControlService::new(true, false);
        let provider = provider_with_partial_update_service(service.clone()).await;
        let from = partial_update_bucket_state();
        let identifier = test_identifier("partial-bucket");
        let patch = partial_update_bucket_patch();

        let outcome = provider
            .update_resource(from.id.clone(), &identifier, &from, &patch)
            .await
            .expect("update should return a partial outcome instead of losing its identifier");

        let UpdateOutcome::PartialSuccess { state, diagnostic } = outcome else {
            panic!("expected partial success when post-update read fails");
        };
        assert!(service.saw_update_resource.load(Ordering::SeqCst));
        assert!(service.saw_get_resource.load(Ordering::SeqCst));
        assert_eq!(state.identifier(), &identifier);
        assert_eq!(
            state.attributes().get("bucket_name"),
            Some(&test_string("partial-bucket"))
        );
        assert!(!state.attributes().contains_key("tags"));
        assert_eq!(diagnostic.missing_attributes(), &["tags".to_string()]);
        assert!(
            diagnostic
                .reason()
                .starts_with("read error: Failed to get resource")
        );
    }

    #[tokio::test]
    async fn update_resource_read_error_strips_write_only_and_reports_all_touched_attributes() {
        let service = PartialUpdateCloudControlService::new(false, false);
        let provider = provider_with_partial_update_service(service).await;
        let from = partial_update_bucket_state();
        let mut patch = access_control_patch();
        patch.ops.push(PatchOp {
            kind: PatchOpKind::Remove,
            key: "website_configuration".to_string(),
            value: None,
        });

        let outcome = provider
            .update_resource(
                from.id.clone(),
                &test_identifier("partial-bucket"),
                &from,
                &patch,
            )
            .await
            .expect("failed read-back should return a partial outcome");

        let UpdateOutcome::PartialSuccess { state, diagnostic } = outcome else {
            panic!("expected partial success when the post-update read fails");
        };
        assert!(
            !state.attributes().contains_key("access_control"),
            "write-only desired values must not be fabricated after a failed read-back"
        );
        assert_eq!(
            diagnostic.missing_attributes(),
            &[
                "access_control".to_string(),
                "website_configuration".to_string(),
            ]
        );
    }

    #[tokio::test]
    async fn update_resource_read_error_with_only_write_only_add_stays_partial() {
        let service = PartialUpdateCloudControlService::new(false, false);
        let provider = provider_with_partial_update_service(service).await;
        let from = partial_update_bucket_state();
        let patch = access_control_patch();

        let outcome = provider
            .update_resource(
                from.id.clone(),
                &test_identifier("partial-bucket"),
                &from,
                &patch,
            )
            .await
            .expect("failed read-back should return a partial outcome");

        let UpdateOutcome::PartialSuccess { state, diagnostic } = outcome else {
            panic!("a failed handler and failed read must not degrade to success");
        };
        assert!(!state.attributes().contains_key("access_control"));
        assert_eq!(
            diagnostic.missing_attributes(),
            &["access_control".to_string()]
        );
        assert_structured_partial_read_error(diagnostic.reason(), "post-update read denied");
    }

    #[tokio::test]
    async fn update_resource_read_error_reports_unsent_create_only_attribute() {
        let service = PartialUpdateCloudControlService::new(false, false);
        let provider = provider_with_partial_update_service(service).await;
        let from = partial_update_bucket_state();
        let patch = create_only_bucket_name_patch();

        let outcome = provider
            .update_resource(
                from.id.clone(),
                &test_identifier("partial-bucket"),
                &from,
                &patch,
            )
            .await
            .expect("failed read-back should return a partial outcome");

        let UpdateOutcome::PartialSuccess { state, diagnostic } = outcome else {
            panic!("expected partial success when the post-update read fails");
        };
        assert!(
            !state.attributes().contains_key("bucket_name"),
            "an unsent create-only desired value must not be fabricated into state"
        );
        assert_eq!(
            diagnostic.missing_attributes(),
            &["bucket_name".to_string(), "tags".to_string()]
        );
        assert!(
            diagnostic
                .reason()
                .contains("handler failed: post-update read denied")
        );
        assert_structured_partial_read_error(diagnostic.reason(), "post-update read denied");
        assert!(
            diagnostic.reason().contains("not sent to Cloud Control"),
            "the diagnostic must distinguish provider-dropped updates from read-back omissions"
        );
    }

    #[tokio::test]
    async fn update_resource_returns_success_when_failed_identifier_read_succeeds() {
        let service = PartialUpdateCloudControlService::new(false, true);
        let provider = provider_with_partial_update_service(service.clone()).await;
        let from = partial_update_bucket_state();
        let patch = partial_update_bucket_patch();

        let outcome = provider
            .update_resource(
                from.id.clone(),
                &test_identifier("partial-bucket"),
                &from,
                &patch,
            )
            .await
            .expect("update should recover when post-update read succeeds");

        let UpdateOutcome::Success { state } = outcome else {
            panic!("expected full success when post-update read succeeds");
        };
        assert!(service.saw_update_resource.load(Ordering::SeqCst));
        assert!(
            service.saw_get_resource.load(Ordering::SeqCst),
            "provider must retry read with the identifier from failed progress"
        );
        assert_eq!(state.identifier().as_str(), "partial-bucket");
        assert!(state.attributes().contains_key("tags"));
    }

    fn partial_create_bucket_resource() -> Resource {
        Resource::with_provider("awscc", "s3.Bucket", "partial_bucket", None).with_attribute(
            "bucket_name",
            Value::Concrete(ConcreteValue::String("partial-bucket".to_string().into())),
        )
    }

    fn assume_role_policy_document() -> Value {
        let mut statement = indexmap::IndexMap::new();
        statement.insert(
            "effect".to_string(),
            Value::Concrete(ConcreteValue::enum_identifier("allow")),
        );
        statement.insert(
            "action".to_string(),
            Value::Concrete(ConcreteValue::String("sts:AssumeRole".to_string().into())),
        );
        statement.insert(
            "principal".to_string(),
            Value::Concrete(ConcreteValue::Map(indexmap::IndexMap::from([(
                "service".to_string(),
                Value::Concrete(ConcreteValue::String(
                    "vpc-flow-logs.amazonaws.com".to_string().into(),
                )),
            )]))),
        );

        Value::Concrete(ConcreteValue::Map(indexmap::IndexMap::from([
            (
                "version".to_string(),
                Value::Concrete(ConcreteValue::enum_identifier("2012_10_17")),
            ),
            (
                "statement".to_string(),
                Value::Concrete(ConcreteValue::List(vec![Value::Concrete(
                    ConcreteValue::Map(statement),
                )])),
            ),
        ])))
    }

    fn iam_role_resource(include_role_name: bool) -> Resource {
        let resource = Resource::with_provider("awscc", "iam.Role", "flow_log_role", None)
            .with_attribute("assume_role_policy_document", assume_role_policy_document());
        if include_role_name {
            resource.with_attribute(
                "role_name",
                Value::Concrete(ConcreteValue::String("flow-log-role".to_string().into())),
            )
        } else {
            resource
        }
    }

    #[tokio::test(start_paused = true)]
    async fn create_resource_preserves_identifier_when_successful_wait_read_back_is_not_found() {
        let service = PartialCreateCloudControlService::with_not_found(true);
        let provider = provider_with_partial_create_service(service.clone()).await;

        let outcome = dispatch_test_create(&provider, partial_create_bucket_resource())
            .await
            .expect("create should return a partial outcome instead of losing its identifier");

        let CreateOutcomeKind::PartialSuccess { state, diagnostic } = outcome.into_kind() else {
            panic!("expected partial success when post-create read returns not_found");
        };
        assert!(service.saw_get_resource.load(Ordering::SeqCst));
        assert_eq!(state.identifier().as_str(), "partial-bucket");
        assert_eq!(
            state.attributes().get("bucket_name"),
            Some(&test_string("partial-bucket"))
        );
        assert_eq!(diagnostic.reason(), "read-back returned not_found");
        assert_eq!(diagnostic.missing_attributes(), &["state".to_string()]);
    }

    #[tokio::test(start_paused = true)]
    async fn create_resource_reports_created_when_failed_identifier_read_fails() {
        let service = PartialCreateCloudControlService::new(false);
        let provider = provider_with_partial_create_service(service.clone()).await;

        let failure = dispatch_test_create(&provider, partial_create_bucket_resource())
            .await
            .expect_err("a terminal CloudControl failure must remain a failure");

        let CreateFailureKind::Created { created, error } = failure.into_kind() else {
            panic!("an identifier from a failed create must be classified as created");
        };
        assert!(
            service.saw_get_resource.load(Ordering::SeqCst),
            "provider must retry read with the identifier from failed progress"
        );
        assert_eq!(created.identifier().as_str(), "partial-bucket");
        assert!(matches!(created.observation(), Observation::ReadFailed(_)));
        assert!(error.to_string().contains("post-create read denied"));
    }

    #[tokio::test(start_paused = true)]
    async fn create_resource_preserves_identifier_when_successful_wait_read_back_fails() {
        let service = PartialCreateCloudControlService::with_wait_result(true, false);
        let provider = provider_with_partial_create_service(service.clone()).await;

        let outcome = dispatch_test_create(&provider, partial_create_bucket_resource())
            .await
            .expect("create should return a partial outcome instead of losing its identifier");

        let CreateOutcomeKind::PartialSuccess { state, diagnostic } = outcome.into_kind() else {
            panic!("expected partial success when post-create read fails");
        };
        assert!(service.saw_get_resource.load(Ordering::SeqCst));
        assert_eq!(state.identifier().as_str(), "partial-bucket");
        assert!(
            diagnostic
                .reason()
                .starts_with("read error: Failed to get resource")
        );
    }

    #[tokio::test(start_paused = true)]
    async fn create_resource_reports_created_when_failed_identifier_read_succeeds() {
        let service = PartialCreateCloudControlService::new(true);
        let provider = provider_with_partial_create_service(service.clone()).await;

        let failure = dispatch_test_create(&provider, partial_create_bucket_resource())
            .await
            .expect_err("read-back must not turn a terminal CloudControl failure into success");

        let CreateFailureKind::Created { created, .. } = failure.into_kind() else {
            panic!("an identifier from a failed create must be classified as created");
        };
        assert!(
            service.saw_get_resource.load(Ordering::SeqCst),
            "provider must retry read with the identifier from failed progress"
        );
        assert_eq!(created.identifier().as_str(), "partial-bucket");
        let Observation::Read(ReadOutcome::Found { state }) = created.observation() else {
            panic!("successful read-back must be retained as the failure observation");
        };
        assert_eq!(state.identifier().as_str(), "partial-bucket");
        assert!(
            state.attributes().get("bucket_name").is_none(),
            "the observation must preserve the read result instead of merging sent values"
        );
    }

    #[tokio::test(start_paused = true)]
    async fn create_iam_role_synthesizes_arn_when_cloudcontrol_omits_arn() {
        let cloudcontrol = IamRoleCreateCloudControlService::new(false);
        let sts = CallerIdentityService::new();
        let provider =
            provider_with_iam_role_create_service(cloudcontrol.clone(), sts.clone()).await;

        let outcome = dispatch_test_create(&provider, iam_role_resource(true))
            .await
            .expect("IAM role create should succeed");

        let CreateOutcomeKind::Success { state } = outcome.into_kind() else {
            panic!("expected full success when role_name can be carried forward");
        };
        assert!(cloudcontrol.saw_create_resource.load(Ordering::SeqCst));
        assert!(cloudcontrol.saw_get_resource.load(Ordering::SeqCst));
        assert!(
            sts.saw_get_caller_identity.load(Ordering::SeqCst),
            "ARN synthesis must use the existing account lookup"
        );
        assert_eq!(
            state.attributes().get("arn"),
            Some(&Value::Concrete(ConcreteValue::String(
                "arn:aws-us-gov:iam::123456789012:role/service-role/flow-log-role"
                    .to_string()
                    .into()
            ))),
            "synthesis must derive the partition from the provider region instead of hardcoding aws"
        );
    }

    #[tokio::test(start_paused = true)]
    async fn create_iam_role_preserves_wait_identifier_when_synthesis_fails() {
        let cloudcontrol = IamRoleCreateCloudControlService::new(false);
        let sts = CallerIdentityService::failing();
        let provider =
            provider_with_iam_role_create_service(cloudcontrol.clone(), sts.clone()).await;

        let outcome = dispatch_test_create(&provider, iam_role_resource(true))
            .await
            .expect("post-create synthesis failure must become partial success");

        let CreateOutcomeKind::PartialSuccess { state, diagnostic } = outcome.into_kind() else {
            panic!("synthesis failure must preserve the created resource");
        };
        assert!(cloudcontrol.saw_create_resource.load(Ordering::SeqCst));
        assert!(cloudcontrol.saw_get_resource.load(Ordering::SeqCst));
        assert!(sts.saw_get_caller_identity.load(Ordering::SeqCst));
        assert_eq!(state.identifier().as_str(), "flow-log-role");
        assert_eq!(
            state.attributes().get("role_name"),
            Some(&test_string("flow-log-role"))
        );
        assert!(
            diagnostic
                .reason()
                .contains("Failed to call sts:GetCallerIdentity")
        );
        assert!(diagnostic.reason().contains("synthesis denied"));
    }

    #[tokio::test(start_paused = true)]
    async fn create_iam_role_synthesizes_arn_from_cloudcontrol_role_name() {
        let cloudcontrol = IamRoleCreateCloudControlService::new(true);
        let sts = CallerIdentityService::new();
        let provider = provider_with_iam_role_create_service(cloudcontrol, sts.clone()).await;

        let outcome = dispatch_test_create(&provider, iam_role_resource(false))
            .await
            .expect("IAM role create should succeed when read-back includes RoleName");

        let CreateOutcomeKind::Success { state } = outcome.into_kind() else {
            panic!("expected full success when CloudControl returns RoleName");
        };
        assert!(
            sts.saw_get_caller_identity.load(Ordering::SeqCst),
            "ARN synthesis must use the existing account lookup"
        );
        assert_eq!(
            state.attributes().get("arn"),
            Some(&Value::Concrete(ConcreteValue::String(
                "arn:aws-us-gov:iam::123456789012:role/service-role/flow-log-role"
                    .to_string()
                    .into()
            )))
        );
    }

    #[tokio::test(start_paused = true)]
    async fn create_iam_role_returns_partial_success_when_post_create_read_is_not_found() {
        let cloudcontrol =
            IamRoleCreateCloudControlService::with_read_mode(IamRoleReadMode::NotFound, true);
        let sts = CallerIdentityService::new();
        let provider =
            provider_with_iam_role_create_service(cloudcontrol.clone(), sts.clone()).await;

        let outcome = dispatch_test_create(&provider, iam_role_resource(true))
            .await
            .expect("IAM role create should report partial success");

        let CreateOutcomeKind::PartialSuccess { state, diagnostic } = outcome.into_kind() else {
            panic!("expected partial success when post-create read returns not_found");
        };
        assert!(cloudcontrol.saw_create_resource.load(Ordering::SeqCst));
        assert!(cloudcontrol.saw_get_resource.load(Ordering::SeqCst));
        assert_eq!(state.identifier().as_str(), "flow-log-role");
        assert_eq!(
            state.attributes().get("role_name"),
            Some(&test_string("flow-log-role"))
        );
        assert_eq!(diagnostic.reason(), "read-back returned not_found");
        assert_eq!(diagnostic.missing_attributes(), &["state".to_string()]);
        assert!(
            !sts.saw_get_caller_identity.load(Ordering::SeqCst),
            "not_found read-back must stop before ARN synthesis"
        );
    }

    #[tokio::test(start_paused = true)]
    async fn create_iam_role_reports_created_when_failed_create_read_back_is_not_found() {
        let cloudcontrol =
            IamRoleCreateCloudControlService::with_read_mode(IamRoleReadMode::NotFound, false);
        let sts = CallerIdentityService::new();
        let provider =
            provider_with_iam_role_create_service(cloudcontrol.clone(), sts.clone()).await;

        let failure = dispatch_test_create(&provider, iam_role_resource(true))
            .await
            .expect_err("a terminal CloudControl failure must remain a failure");

        let CreateFailureKind::Created { created, .. } = failure.into_kind() else {
            panic!("an identifier from a failed create must be classified as created");
        };
        assert!(cloudcontrol.saw_create_resource.load(Ordering::SeqCst));
        assert!(cloudcontrol.saw_get_resource.load(Ordering::SeqCst));
        assert_eq!(created.identifier().as_str(), "flow-log-role");
        assert!(matches!(
            created.observation(),
            Observation::Read(ReadOutcome::NotFound { .. })
        ));
        assert!(
            !sts.saw_get_caller_identity.load(Ordering::SeqCst),
            "not_found read-back must stop before ARN synthesis"
        );
    }

    #[tokio::test(start_paused = true)]
    async fn create_iam_role_returns_partial_success_when_role_name_is_unknown() {
        let cloudcontrol = IamRoleCreateCloudControlService::new(false);
        let sts = CallerIdentityService::new();
        let provider = provider_with_iam_role_create_service(cloudcontrol, sts.clone()).await;

        let outcome = dispatch_test_create(&provider, iam_role_resource(false))
            .await
            .expect("IAM role create should report partial success");

        let CreateOutcomeKind::PartialSuccess { state, diagnostic } = outcome.into_kind() else {
            panic!("expected partial success when primaryIdentifier cannot be canonicalized");
        };
        assert_eq!(state.identifier().as_str(), "flow-log-role");
        assert!(
            !state.attributes().contains_key("arn"),
            "unknown ARN must not be published as a successful attribute"
        );
        assert_eq!(diagnostic.missing_attributes(), &["RoleName".to_string()]);
        assert_eq!(
            diagnostic.reason(),
            "read-back missing primaryIdentifier attributes: RoleName"
        );
        assert!(
            !sts.saw_get_caller_identity.load(Ordering::SeqCst),
            "missing role_name should not call STS because the ARN is not synthesizable"
        );
    }

    fn make_schema_attrs(
        entries: Vec<(&str, &str, AttributeType, bool)>,
    ) -> HashMap<String, AttributeSchema> {
        let mut map = HashMap::new();
        for (dsl_name, provider_name, attr_type, required) in entries {
            let mut s = AttributeSchema::new(dsl_name, attr_type);
            s.provider_name = Some(provider_name.to_string());
            if required {
                s = s.required();
            }
            map.insert(dsl_name.to_string(), s);
        }
        map
    }

    /// carina-rs/carina#2544: CloudControl omits empty optional list
    /// fields from `GetResource`. The provider read path must canonicalize
    /// these absent-but-empty fields to `Value::Concrete(ConcreteValue::List(vec![]))` so the
    /// differ does not see `(none) → []` against a user-specified `= []`.
    #[test]
    fn absent_optional_list_becomes_empty_list() {
        let attrs = make_schema_attrs(vec![(
            "managed_policy_arns",
            "ManagedPolicyArns",
            AttributeType::list(AttributeType::string()),
            false,
        )]);
        let props = json!({});

        let result = map_aws_props_to_attributes(
            &props,
            &attrs,
            "iam.Role",
            &std::collections::BTreeMap::new(),
        );

        assert_eq!(
            result.get("managed_policy_arns"),
            Some(&Value::Concrete(ConcreteValue::List(Vec::new()))),
            "absent optional list-typed attribute must canonicalize to empty list, not be dropped"
        );
    }

    /// Same shape but for an optional map-typed attribute.
    #[test]
    fn absent_optional_map_becomes_empty_map() {
        let attrs = make_schema_attrs(vec![(
            "metadata",
            "Metadata",
            AttributeType::map(AttributeType::string()),
            false,
        )]);
        let props = json!({});

        let result = map_aws_props_to_attributes(
            &props,
            &attrs,
            "some.Resource",
            &std::collections::BTreeMap::new(),
        );

        assert_eq!(
            result.get("metadata"),
            Some(&Value::Concrete(ConcreteValue::Map(
                indexmap::IndexMap::new()
            ))),
            "absent optional map-typed attribute must canonicalize to empty map"
        );
    }

    /// Required attributes that are unexpectedly absent must NOT be
    /// fabricated — that would mask a real provider-side bug.
    #[test]
    fn absent_required_list_is_not_fabricated() {
        let attrs = make_schema_attrs(vec![(
            "required_list",
            "RequiredList",
            AttributeType::list(AttributeType::string()),
            true,
        )]);
        let props = json!({});

        let result = map_aws_props_to_attributes(
            &props,
            &attrs,
            "some.Resource",
            &std::collections::BTreeMap::new(),
        );

        assert!(
            !result.contains_key("required_list"),
            "required attributes must not be synthesized when AWS omits them"
        );
    }

    /// Scalar absence still means "untracked" — the carry-forward path
    /// in `read_resource` populates these from saved state. Synthesizing
    /// a default here would clobber that.
    #[test]
    fn absent_optional_scalar_is_not_fabricated() {
        let attrs = make_schema_attrs(vec![(
            "description",
            "Description",
            AttributeType::string(),
            false,
        )]);
        let props = json!({});

        let result = map_aws_props_to_attributes(
            &props,
            &attrs,
            "some.Resource",
            &std::collections::BTreeMap::new(),
        );

        assert!(
            !result.contains_key("description"),
            "absent scalar attributes must not be synthesized; carry-forward owns that case"
        );
    }

    /// Present list values flow through `aws_value_to_dsl` unchanged.
    #[test]
    fn present_list_passes_through() {
        let attrs = make_schema_attrs(vec![(
            "managed_policy_arns",
            "ManagedPolicyArns",
            AttributeType::list(AttributeType::string()),
            false,
        )]);
        let props = json!({
            "ManagedPolicyArns": ["arn:aws:iam::aws:policy/ReadOnlyAccess"]
        });

        let result = map_aws_props_to_attributes(
            &props,
            &attrs,
            "iam.Role",
            &std::collections::BTreeMap::new(),
        );

        assert_eq!(
            result.get("managed_policy_arns"),
            Some(&Value::Concrete(ConcreteValue::List(vec![
                Value::Concrete(ConcreteValue::String(
                    "arn:aws:iam::aws:policy/ReadOnlyAccess".to_string().into()
                ))
            ]))),
        );
    }

    /// Tags are always skipped at this layer.
    #[test]
    fn tags_attribute_is_skipped() {
        let attrs = make_schema_attrs(vec![(
            "tags",
            "Tags",
            AttributeType::map(AttributeType::string()),
            false,
        )]);
        let props = json!({});

        let result = map_aws_props_to_attributes(
            &props,
            &attrs,
            "some.Resource",
            &std::collections::BTreeMap::new(),
        );

        assert!(
            !result.contains_key("tags"),
            "tags must be skipped here; AwsccProvider::read_resource owns that path"
        );
    }
}
