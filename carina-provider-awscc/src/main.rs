#[cfg(any(target_arch = "wasm32", test))]
use std::collections::{BTreeMap, HashMap};

#[cfg(any(target_arch = "wasm32", test))]
use carina_plugin_sdk::types as proto;
#[cfg(any(target_arch = "wasm32", test))]
use carina_plugin_sdk::{
    CarinaProvider, CreateAttempt as SdkCreateAttempt, CreateStep as SdkCreateStep,
    CreateVerdict as SdkCreateVerdict, DeleteOutcome as SdkDeleteOutcome, FoundRead,
    IssuedCreate as SdkIssuedCreate, Pending, PendingDelete, Poll as SdkPoll,
    PollAttempt as SdkPollAttempt, PollRequest, PollVerdict as SdkPollVerdict, Resume as SdkResume,
    ResumeAttempt as SdkResumeAttempt, ResumeStep as SdkResumeStep,
    ResumeVerdict as SdkResumeVerdict, ResumedCreate as SdkResumedCreate,
    UpdateOutcome as SdkUpdateOutcome,
};
#[cfg(any(target_arch = "wasm32", test))]
use carina_provider_protocol::{OperationStep, PendingPhase};

#[cfg(any(target_arch = "wasm32", test))]
use carina_core::executor::normalized::NormalizedResource as CoreNormalizedResource;
#[cfg(any(target_arch = "wasm32", test))]
use carina_core::protocol_convert;
#[cfg(any(target_arch = "wasm32", test))]
use carina_core::provider::{
    AppliedIdentifier as CoreAppliedIdentifier, AppliedState as CoreAppliedState,
    DeleteRequest as CoreDeleteRequest, LiftedSavedAttrs, Observation as CoreObservation,
    PersistedAttributes, Provider, ProviderDirectiveSchema as CoreProviderDirectiveSchema,
    ProviderError as CoreProviderError, ProviderFactory, RawSavedAttrs,
    ReadOutcome as CoreReadOutcome, ReadRequest as CoreReadRequest, Sent as CoreSent,
};
#[cfg(any(target_arch = "wasm32", test))]
use carina_core::resource::{
    ConcreteValue, DataSource as CoreDataSource, Directives as CoreDirectives,
    Resource as CoreResource, ResourceId as CoreResourceId, Value as CoreValue,
};
#[cfg(any(target_arch = "wasm32", test))]
use carina_core::schema::{
    AssignabilityEdge, ReferenceForms, SchemaKind, SchemaRegistry, ValidatedConfigSchema,
    ValidatedResourceSchema,
};

#[cfg(any(target_arch = "wasm32", test))]
use carina_provider_awscc::provider::{
    AwsccProvider, AwsccProviderConfig, CcCreateAttempt, CcCreateOperation, CcCreatePoll,
    CcCreateResume, CcCreateToken, CcDeleteToken, CcIssuedCreate, CcPollAttempt, CcResumeAttempt,
    CcResumedCreate, CcUpdateToken, InFlightCreate, NotAppliedCreate,
};
#[cfg(any(target_arch = "wasm32", test))]
use carina_provider_awscc::schemas;
#[cfg(any(target_arch = "wasm32", test))]
use carina_provider_awscc::{AwsccNormalizer, AwsccProviderFactory};

#[cfg(any(target_arch = "wasm32", test))]
fn protocol_input_error(
    context: &'static str,
    error: protocol_convert::ProtocolConvertError,
) -> CoreProviderError {
    CoreProviderError::invalid_input_caused_by(context, error)
}

#[cfg(any(target_arch = "wasm32", test))]
fn protocol_output_error(
    context: &'static str,
    error: protocol_convert::ProtocolConvertError,
) -> CoreProviderError {
    CoreProviderError::internal_caused_by(context, error)
}

#[cfg(any(target_arch = "wasm32", test))]
fn wire_directives_to_core(directives: &proto::Directives) -> CoreDirectives {
    CoreDirectives::default()
        .with_force_delete(directives.force_delete)
        .with_create_before_destroy(directives.create_before_destroy)
        .with_prevent_destroy(directives.prevent_destroy)
}

#[cfg(any(target_arch = "wasm32", test))]
fn protocol_resource_to_core(
    resource: &proto::Resource,
) -> Result<CoreResource, protocol_convert::ProtocolConvertError> {
    let mut core = CoreResource::from_id(protocol_convert::protocol_to_resource_id(&resource.id));
    core.attributes = protocol_convert::protocol_to_core_value_map(&resource.attributes)?
        .into_iter()
        .collect();
    core.directives = wire_directives_to_core(&resource.directives);
    Ok(core)
}

#[cfg(any(target_arch = "wasm32", test))]
fn protocol_data_source_to_core(
    resource: &proto::Resource,
) -> Result<CoreDataSource, protocol_convert::ProtocolConvertError> {
    let id = protocol_convert::protocol_to_resource_id(&resource.id);
    let mut core = CoreDataSource::pending_with_provider(
        id.provider.clone(),
        id.resource_type.clone(),
        id.provider_instance.clone(),
    );
    core.id = id;
    core.attributes = protocol_convert::protocol_to_core_value_map(&resource.attributes)?
        .into_iter()
        .collect();
    core.directives = wire_directives_to_core(&resource.directives);
    Ok(core)
}

#[cfg(any(target_arch = "wasm32", test))]
fn provider_config_decode_error(
    error: protocol_convert::ProtocolConvertError,
) -> CoreProviderError {
    protocol_input_error("failed to decode provider configuration", error)
}

#[cfg(any(target_arch = "wasm32", test))]
fn read_outcome_to_proto(
    outcome: &CoreReadOutcome,
) -> Result<proto::ReadOutcome, CoreProviderError> {
    protocol_convert::try_read_outcome_to_protocol(outcome).map_err(|error| {
        let id = match outcome {
            CoreReadOutcome::NotFound { id } => id,
            CoreReadOutcome::Found { state } => state.id(),
        };
        protocol_output_error("failed to encode read result", error).for_resource(id.clone())
    })
}

#[cfg(any(target_arch = "wasm32", test))]
fn issued_create_attributes_to_proto(
    attributes: &HashMap<String, CoreValue>,
) -> Result<BTreeMap<String, proto::Value>, CoreProviderError> {
    let attributes = attributes
        .iter()
        .map(|(name, value)| {
            let known = value
                .knowledge(&carina_core::resource::knowledge::KnowledgeContext::validation())?
                .into_classification()
                .into_known()?;
            Ok((name.clone(), known))
        })
        .collect::<Result<HashMap<_, _>, carina_core::resource::knowledge::KnowledgeError>>()
        .map_err(|error| {
            CoreProviderError::internal_caused_by("failed to encode create result", error)
        })?;
    protocol_convert::try_core_to_protocol_value_map(&attributes)
        .map_err(|error| protocol_output_error("failed to encode create result", error))
}

#[cfg(any(target_arch = "wasm32", test))]
fn partial_create_attributes_to_proto(
    attributes: &HashMap<String, CoreValue>,
) -> Result<BTreeMap<String, proto::Value>, CoreProviderError> {
    let attributes = attributes
        .iter()
        .map(|(name, value)| {
            let known = value
                .knowledge(&carina_core::resource::knowledge::KnowledgeContext::validation())?
                .into_classification()
                .into_known()?;
            Ok((name.clone(), known))
        })
        .collect::<Result<HashMap<_, _>, carina_core::resource::knowledge::KnowledgeError>>()
        .map_err(|error| {
            CoreProviderError::internal_caused_by("failed to encode partial create result", error)
        })?;
    protocol_convert::try_core_to_protocol_value_map(&attributes)
        .map_err(|error| protocol_output_error("failed to encode partial create result", error))
}

#[cfg(any(target_arch = "wasm32", test))]
fn resumed_state_to_proto(
    state: &CoreAppliedState,
) -> Result<proto::AppliedState, CoreProviderError> {
    protocol_convert::try_applied_state_to_protocol(state)
        .map_err(|error| protocol_output_error("failed to encode resumed create result", error))
}

#[cfg(any(target_arch = "wasm32", test))]
fn update_outcome_to_sdk(
    outcome: &carina_core::provider::UpdateOutcome,
    state_after_patch: proto::PersistedState,
    id: &carina_core::resource::ResourceId,
) -> Result<proto::UpdateOutcome, CoreProviderError> {
    use carina_core::provider::UpdateOutcomeKind;

    let (state, diagnostic) = match outcome.kind() {
        UpdateOutcomeKind::Success { state } => (state, None),
        UpdateOutcomeKind::PartialSuccess { state, diagnostic } => (state, Some(diagnostic)),
    };
    let mut attributes = protocol_convert::try_core_to_protocol_persisted_value_map(
        state.attributes(),
    )
    .map_err(|error| {
        protocol_output_error("failed to encode update result", error).for_resource(id.clone())
    })?;
    for (key, value) in state_after_patch.attributes {
        attributes.entry(key).or_insert(value);
    }
    let state = proto::AppliedPersistedState {
        id: protocol_convert::resource_id_to_protocol(state.id()),
        identifier: state.identifier().clone(),
        attributes,
    };
    Ok(match diagnostic {
        Some(diagnostic) => proto::UpdateOutcome::PartialSuccess {
            state,
            diagnostic: diagnostic.clone(),
        },
        None => proto::UpdateOutcome::Success { state },
    })
}

#[cfg(any(target_arch = "wasm32", test))]
struct GuestCreateAttempt(SdkCreateAttempt);
#[cfg(any(target_arch = "wasm32", test))]
struct GuestPollAttempt(SdkPollAttempt);
#[cfg(any(target_arch = "wasm32", test))]
struct GuestIssuedCreate(SdkIssuedCreate);
#[cfg(any(target_arch = "wasm32", test))]
struct GuestPolledIssuedCreate(SdkIssuedCreate);
#[cfg(any(target_arch = "wasm32", test))]
struct GuestResumeAttempt(SdkResumeAttempt);
#[cfg(any(target_arch = "wasm32", test))]
struct GuestResumedCreate(SdkResumedCreate);
#[cfg(any(target_arch = "wasm32", test))]
struct GuestResentCreateAttempt(SdkCreateAttempt);
#[cfg(any(target_arch = "wasm32", test))]
struct GuestResentIssuedCreate(SdkIssuedCreate);

#[cfg(any(target_arch = "wasm32", test))]
fn observation_to_sdk(
    observation: CoreObservation,
) -> proto::Observation<proto::ReadOutcome, proto::ProviderError> {
    match observation {
        CoreObservation::Read(outcome) => match read_outcome_to_proto(&outcome) {
            Ok(outcome) => proto::Observation::Read(outcome),
            Err(error) => proto::Observation::ReadFailed(error),
        },
        CoreObservation::ReadFailed(error) => proto::Observation::ReadFailed(error),
    }
}

#[cfg(any(target_arch = "wasm32", test))]
fn finish_encoded<Finishing, Encoded, Step, Verdict>(
    finishing: Finishing,
    encoded: Result<Encoded, CoreProviderError>,
    succeeded: impl FnOnce(Finishing, Encoded) -> Step,
    failed: impl FnOnce(
        Finishing,
        proto::Observation<proto::ReadOutcome, proto::ProviderError>,
        proto::ProviderError,
    ) -> Verdict,
) -> Result<Step, Verdict> {
    match encoded {
        Ok(encoded) => Ok(succeeded(finishing, encoded)),
        Err(error) => Err(failed(
            finishing,
            proto::Observation::ReadFailed(error.clone()),
            error,
        )),
    }
}

#[cfg(any(target_arch = "wasm32", test))]
impl CcCreateAttempt for GuestCreateAttempt {
    type Step = SdkCreateStep<CcCreateToken>;
    type Verdict = SdkCreateVerdict<CcCreateToken>;
    type Issued = GuestIssuedCreate;

    fn not_created(self, failure: NotAppliedCreate) -> Self::Verdict {
        self.0.not_created(failure.into_error())
    }

    fn preserve(self, token: CcCreateOperation, error: CoreProviderError) -> Self::Verdict {
        let poll_after_secs = token.poll_after_secs();
        self.0.preserve(
            token,
            CoreSent::AllRequestAttributes,
            poll_after_secs,
            error,
        )
    }

    fn issued(self, identifier: CoreAppliedIdentifier, sent: CoreSent) -> Self::Issued {
        GuestIssuedCreate(self.0.issued(identifier, sent))
    }

    fn pending(
        self,
        token: CcCreateOperation,
        sent: CoreSent,
        poll_after_secs: u32,
    ) -> Result<Self::Step, Self::Verdict> {
        let token = token.with_poll_after_secs(poll_after_secs);
        Ok(self.0.pending(token, sent, poll_after_secs))
    }
}

#[cfg(any(target_arch = "wasm32", test))]
impl CcPollAttempt for GuestPollAttempt {
    type Step = SdkCreateStep<CcCreateToken>;
    type Verdict = SdkPollVerdict<CcCreateToken>;
    type Issued = GuestPolledIssuedCreate;

    fn not_created(self, failure: NotAppliedCreate) -> Self::Verdict {
        self.0.not_created(failure.into_error())
    }

    fn preserve(self, token: CcCreateOperation, failure: InFlightCreate) -> Self::Verdict {
        let error = failure.into_error();
        let poll_after_secs = token.poll_after_secs();
        self.0.preserve(token, poll_after_secs, error)
    }

    fn issued(self, identifier: CoreAppliedIdentifier) -> Self::Issued {
        GuestPolledIssuedCreate(self.0.issued(identifier))
    }

    fn pending(
        self,
        token: CcCreateOperation,
        poll_after_secs: u32,
    ) -> Result<Self::Step, Self::Verdict> {
        let token = token.with_poll_after_secs(poll_after_secs);
        Ok(self.0.pending(token, poll_after_secs))
    }
}

#[cfg(any(target_arch = "wasm32", test))]
impl CcIssuedCreate for GuestIssuedCreate {
    type Step = SdkCreateStep<CcCreateToken>;
    type Verdict = SdkCreateVerdict<CcCreateToken>;

    fn identifier(&self) -> &CoreAppliedIdentifier {
        self.0.identifier()
    }

    fn succeeded(
        self,
        identifier: CoreAppliedIdentifier,
        attributes: HashMap<String, CoreValue>,
    ) -> Result<Self::Step, Self::Verdict> {
        let finishing = self.0.canonicalize(identifier);
        finish_encoded(
            finishing,
            issued_create_attributes_to_proto(&attributes),
            |finishing, attributes| finishing.succeeded(attributes),
            |finishing, observation, error| finishing.identifier_reported(observation, error),
        )
    }

    fn partial(
        self,
        identifier: CoreAppliedIdentifier,
        attributes: HashMap<String, CoreValue>,
        reason: CoreProviderError,
        missing_attributes: carina_provider_awscc::provider::read_back::NonEmptyAttributeNames,
    ) -> Result<Self::Step, Self::Verdict> {
        let finishing = self.0.canonicalize(identifier);
        finish_encoded(
            finishing,
            partial_create_attributes_to_proto(&attributes),
            |finishing, attributes| {
                finishing.partial(
                    attributes,
                    carina_core::provider::PartialReadDiagnostic::new(
                        carina_core::provider_text::ProviderDiagnostic::new(reason),
                        missing_attributes.into(),
                    ),
                )
            },
            |finishing, observation, error| finishing.identifier_reported(observation, error),
        )
    }

    fn identifier_reported(
        self,
        identifier: CoreAppliedIdentifier,
        observation: CoreObservation,
        error: CoreProviderError,
    ) -> Self::Verdict {
        self.0
            .canonicalize(identifier)
            .identifier_reported(observation_to_sdk(observation), error)
    }

    fn no_resource(self, error: CoreProviderError) -> Self::Verdict {
        self.0.no_resource(error)
    }

    fn preserve(self, token: CcCreateToken, error: CoreProviderError) -> Self::Verdict {
        let poll_after_secs = token.poll_after_secs();
        self.0.preserve(token, poll_after_secs, error)
    }

    fn pending(
        self,
        token: CcCreateToken,
        poll_after_secs: u32,
    ) -> Result<Self::Step, Self::Verdict> {
        let token = token.with_poll_after_secs(poll_after_secs);
        Ok(self.0.pending(token, poll_after_secs))
    }
}

#[cfg(any(target_arch = "wasm32", test))]
impl CcIssuedCreate for GuestPolledIssuedCreate {
    type Step = SdkCreateStep<CcCreateToken>;
    type Verdict = SdkPollVerdict<CcCreateToken>;

    fn identifier(&self) -> &CoreAppliedIdentifier {
        self.0.identifier()
    }

    fn succeeded(
        self,
        identifier: CoreAppliedIdentifier,
        attributes: HashMap<String, CoreValue>,
    ) -> Result<Self::Step, Self::Verdict> {
        let finishing = self.0.canonicalize(identifier);
        finish_encoded(
            finishing,
            issued_create_attributes_to_proto(&attributes),
            |finishing, attributes| finishing.succeeded(attributes),
            |finishing, observation, error| {
                finishing.identifier_reported(observation, error).into()
            },
        )
    }

    fn partial(
        self,
        identifier: CoreAppliedIdentifier,
        attributes: HashMap<String, CoreValue>,
        reason: CoreProviderError,
        missing_attributes: carina_provider_awscc::provider::read_back::NonEmptyAttributeNames,
    ) -> Result<Self::Step, Self::Verdict> {
        let finishing = self.0.canonicalize(identifier);
        finish_encoded(
            finishing,
            partial_create_attributes_to_proto(&attributes),
            |finishing, attributes| {
                finishing.partial(
                    attributes,
                    carina_core::provider::PartialReadDiagnostic::new(
                        carina_core::provider_text::ProviderDiagnostic::new(reason),
                        missing_attributes.into(),
                    ),
                )
            },
            |finishing, observation, error| {
                finishing.identifier_reported(observation, error).into()
            },
        )
    }

    fn identifier_reported(
        self,
        identifier: CoreAppliedIdentifier,
        observation: CoreObservation,
        error: CoreProviderError,
    ) -> Self::Verdict {
        self.0
            .canonicalize(identifier)
            .identifier_reported(observation_to_sdk(observation), error)
            .into()
    }

    fn no_resource(self, error: CoreProviderError) -> Self::Verdict {
        self.0.no_resource(error).into()
    }

    fn preserve(self, token: CcCreateToken, error: CoreProviderError) -> Self::Verdict {
        let poll_after_secs = token.poll_after_secs();
        self.0.preserve(token, poll_after_secs, error).into()
    }

    fn pending(
        self,
        token: CcCreateToken,
        poll_after_secs: u32,
    ) -> Result<Self::Step, Self::Verdict> {
        let token = token.with_poll_after_secs(poll_after_secs);
        Ok(self.0.pending(token, poll_after_secs))
    }
}

#[cfg(any(target_arch = "wasm32", test))]
impl CcResumeAttempt for GuestResumeAttempt {
    type Step = SdkResumeStep<CcCreateToken>;
    type Verdict = SdkResumeVerdict<CcCreateToken>;
    type Resumed = GuestResumedCreate;
    type Resent = GuestResentCreateAttempt;

    fn not_created(self, failure: NotAppliedCreate) -> Self::Verdict {
        self.0.not_created(failure.into_error())
    }

    fn preserve(self, token: CcCreateOperation, error: CoreProviderError) -> Self::Verdict {
        let poll_after_secs = token.poll_after_secs();
        self.0.preserve(token, poll_after_secs, error)
    }

    fn issued(self, identifier: CoreAppliedIdentifier) -> Self::Resumed {
        GuestResumedCreate(self.0.issued(identifier))
    }

    fn pending(
        self,
        token: CcCreateOperation,
        poll_after_secs: u32,
    ) -> Result<Self::Step, Self::Verdict> {
        let token = token.with_poll_after_secs(poll_after_secs);
        Ok(self.0.pending(token, poll_after_secs))
    }

    fn resend(self) -> Self::Resent {
        GuestResentCreateAttempt(self.0.resend())
    }
}

#[cfg(any(target_arch = "wasm32", test))]
impl CcResumedCreate for GuestResumedCreate {
    type Step = SdkResumeStep<CcCreateToken>;
    type Verdict = SdkResumeVerdict<CcCreateToken>;

    fn identifier(&self) -> &CoreAppliedIdentifier {
        self.0.identifier()
    }

    fn completed(
        self,
        identifier: CoreAppliedIdentifier,
        state: CoreAppliedState,
    ) -> Result<Self::Step, Self::Verdict> {
        let finishing = self.0.canonicalize(identifier);
        finish_encoded(
            finishing,
            resumed_state_to_proto(&state),
            |finishing, state| finishing.completed(FoundRead::from_applied_state(state)),
            |finishing, observation, error| finishing.identifier_reported(observation, error),
        )
    }

    fn partial(
        self,
        identifier: CoreAppliedIdentifier,
        attributes: HashMap<String, CoreValue>,
        reason: CoreProviderError,
        missing_attributes: carina_provider_awscc::provider::read_back::NonEmptyAttributeNames,
    ) -> Result<Self::Step, Self::Verdict> {
        let finishing = self.0.canonicalize(identifier);
        finish_encoded(
            finishing,
            partial_create_attributes_to_proto(&attributes),
            |finishing, attributes| {
                finishing.partial(
                    attributes,
                    carina_core::provider::PartialReadDiagnostic::new(
                        carina_core::provider_text::ProviderDiagnostic::new(reason),
                        missing_attributes.into(),
                    ),
                )
            },
            |finishing, observation, error| finishing.identifier_reported(observation, error),
        )
    }

    fn identifier_reported(
        self,
        identifier: CoreAppliedIdentifier,
        observation: CoreObservation,
        error: CoreProviderError,
    ) -> Self::Verdict {
        self.0
            .canonicalize(identifier)
            .identifier_reported(observation_to_sdk(observation), error)
    }

    fn no_resource(self, error: CoreProviderError) -> Self::Verdict {
        self.0.no_resource(error)
    }

    fn preserve(self, token: CcCreateToken, error: CoreProviderError) -> Self::Verdict {
        let poll_after_secs = token.poll_after_secs();
        self.0.preserve(token, poll_after_secs, error)
    }

    fn pending(
        self,
        token: CcCreateToken,
        poll_after_secs: u32,
    ) -> Result<Self::Step, Self::Verdict> {
        let token = token.with_poll_after_secs(poll_after_secs);
        Ok(self.0.pending(token, poll_after_secs))
    }
}

#[cfg(any(target_arch = "wasm32", test))]
impl CcCreateAttempt for GuestResentCreateAttempt {
    type Step = SdkResumeStep<CcCreateToken>;
    type Verdict = SdkResumeVerdict<CcCreateToken>;
    type Issued = GuestResentIssuedCreate;

    fn not_created(self, failure: NotAppliedCreate) -> Self::Verdict {
        self.0.not_created(failure.into_error()).into()
    }

    fn preserve(self, token: CcCreateOperation, error: CoreProviderError) -> Self::Verdict {
        let poll_after_secs = token.poll_after_secs();
        self.0
            .preserve(
                token,
                CoreSent::AllRequestAttributes,
                poll_after_secs,
                error,
            )
            .into()
    }

    fn issued(self, identifier: CoreAppliedIdentifier, sent: CoreSent) -> Self::Issued {
        GuestResentIssuedCreate(self.0.issued(identifier, sent))
    }

    fn pending(
        self,
        token: CcCreateOperation,
        sent: CoreSent,
        poll_after_secs: u32,
    ) -> Result<Self::Step, Self::Verdict> {
        let token = token.with_poll_after_secs(poll_after_secs);
        Ok(self.0.pending(token, sent, poll_after_secs).into())
    }
}

#[cfg(any(target_arch = "wasm32", test))]
impl CcIssuedCreate for GuestResentIssuedCreate {
    type Step = SdkResumeStep<CcCreateToken>;
    type Verdict = SdkResumeVerdict<CcCreateToken>;

    fn identifier(&self) -> &CoreAppliedIdentifier {
        self.0.identifier()
    }

    fn succeeded(
        self,
        identifier: CoreAppliedIdentifier,
        attributes: HashMap<String, CoreValue>,
    ) -> Result<Self::Step, Self::Verdict> {
        let finishing = self.0.canonicalize(identifier);
        finish_encoded(
            finishing,
            issued_create_attributes_to_proto(&attributes),
            |finishing, attributes| finishing.succeeded(attributes).into(),
            |finishing, observation, error| {
                finishing.identifier_reported(observation, error).into()
            },
        )
    }

    fn partial(
        self,
        identifier: CoreAppliedIdentifier,
        attributes: HashMap<String, CoreValue>,
        reason: CoreProviderError,
        missing_attributes: carina_provider_awscc::provider::read_back::NonEmptyAttributeNames,
    ) -> Result<Self::Step, Self::Verdict> {
        let finishing = self.0.canonicalize(identifier);
        finish_encoded(
            finishing,
            partial_create_attributes_to_proto(&attributes),
            |finishing, attributes| {
                finishing
                    .partial(
                        attributes,
                        carina_core::provider::PartialReadDiagnostic::new(
                            carina_core::provider_text::ProviderDiagnostic::new(reason),
                            missing_attributes.into(),
                        ),
                    )
                    .into()
            },
            |finishing, observation, error| {
                finishing.identifier_reported(observation, error).into()
            },
        )
    }

    fn identifier_reported(
        self,
        identifier: CoreAppliedIdentifier,
        observation: CoreObservation,
        error: CoreProviderError,
    ) -> Self::Verdict {
        self.0
            .canonicalize(identifier)
            .identifier_reported(observation_to_sdk(observation), error)
            .into()
    }

    fn no_resource(self, error: CoreProviderError) -> Self::Verdict {
        self.0.no_resource(error).into()
    }

    fn preserve(self, token: CcCreateToken, error: CoreProviderError) -> Self::Verdict {
        let poll_after_secs = token.poll_after_secs();
        self.0.preserve(token, poll_after_secs, error).into()
    }

    fn pending(
        self,
        token: CcCreateToken,
        poll_after_secs: u32,
    ) -> Result<Self::Step, Self::Verdict> {
        let token = token.with_poll_after_secs(poll_after_secs);
        Ok(self.0.pending(token, poll_after_secs).into())
    }
}

#[cfg(any(target_arch = "wasm32", test))]
struct AwsccProcessProvider {
    runtime: tokio::runtime::Runtime,
    provider: Option<AwsccProvider>,
    normalizer: AwsccNormalizer,
    #[cfg(not(target_arch = "wasm32"))]
    diagnostic_session: carina_core::provider::ProviderSession,
}

#[cfg(any(target_arch = "wasm32", test))]
impl Default for AwsccProcessProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(any(target_arch = "wasm32", test))]
impl AwsccProcessProvider {
    fn new() -> Self {
        #[cfg(not(target_arch = "wasm32"))]
        let runtime = carina_plugin_sdk::expect_diagnostic(
            tokio::runtime::Runtime::new()
                .map_err(carina_core::provider_text::ProviderDiagnostic::new),
        );
        #[cfg(target_arch = "wasm32")]
        let runtime = carina_plugin_sdk::expect_diagnostic(
            tokio::runtime::Builder::new_current_thread()
                .enable_time()
                .build()
                .map_err(carina_core::provider_text::ProviderDiagnostic::new),
        );
        Self {
            runtime,
            provider: None,
            normalizer: AwsccNormalizer,
            #[cfg(not(target_arch = "wasm32"))]
            diagnostic_session: carina_core::provider::ProviderPlan::state_only()
                .session("awscc", None, None),
        }
    }

    fn provider(&self) -> &AwsccProvider {
        self.provider
            .as_ref()
            .expect("Provider not initialized; call initialize() first")
    }

    fn schema_registry() -> SchemaRegistry {
        schemas::schema_registry()
    }

    fn normalize_create_resource(
        &self,
        resource: &proto::Resource,
    ) -> Result<CoreNormalizedResource, CoreProviderError> {
        let core_resource = protocol_resource_to_core(resource).map_err(|error| {
            protocol_input_error("failed to decode create request", error)
                .for_resource(protocol_convert::protocol_to_resource_id(&resource.id))
        })?;
        let schemas = Self::schema_registry();
        self.runtime.block_on(
            carina_core::executor::normalized::apply_desired_normalization(
                core_resource,
                &[],
                &self.normalizer,
                &schemas,
            ),
        )
    }

    fn map_step<T, Token, Outcome>(
        result: Result<OperationStep<T, CoreProviderError, Token>, CoreProviderError>,
        done: impl FnOnce(T) -> Outcome,
        pending: impl FnOnce(PendingDelete<Token>) -> Outcome,
    ) -> Result<Outcome, CoreProviderError> {
        match result {
            Ok(OperationStep::Done(value)) => Ok(done(value)),
            Ok(OperationStep::Verdict(error)) => Err(error),
            Ok(OperationStep::Pending {
                pending: token,
                poll_after_secs,
                phase: PendingPhase::Operation,
            }) => Ok(pending(PendingDelete {
                token,
                poll_after_secs,
            })),
            Ok(OperationStep::Pending { phase, .. }) => Err(CoreProviderError::internal(format!(
                "unexpected pending phase for delete: {phase:?}"
            ))),
            Err(error) => Err(error),
        }
    }

    fn core_delete_request(
        id: &CoreResourceId,
        request: proto::DeleteRequest,
    ) -> Result<CoreDeleteRequest, CoreProviderError> {
        let schemas = Self::schema_registry();
        let Some(schema) = schemas.get("awscc", &id.resource_type, SchemaKind::Resource) else {
            return Err(CoreProviderError::internal(
                "missing schema while resolving delete directives",
            )
            .for_resource(id.clone()));
        };
        let directives = wire_directives_to_core(&request.directives);
        Ok(CoreDeleteRequest::resolve(
            &directives,
            CoreProviderDirectiveSchema::Available(schema.schema().as_schema()),
        ))
    }
}

#[cfg(any(target_arch = "wasm32", test))]
impl CarinaProvider for AwsccProcessProvider {
    type CreateToken = CcCreateToken;
    type UpdateToken = CcUpdateToken;
    type DeleteToken = CcDeleteToken;

    fn info(&self) -> proto::ProviderInfo {
        proto::ProviderInfo {
            name: "awscc".into(),
            display_name: "AWS Cloud Control provider".into(),
            capabilities: vec![],
            version: env!("CARGO_PKG_VERSION").into(),
        }
    }

    fn schemas(&self) -> Vec<ValidatedResourceSchema> {
        schemas::all_schemas()
    }

    fn reference_forms(&self) -> ReferenceForms {
        ReferenceForms::new()
    }

    fn assignability_edges(&self) -> Vec<AssignabilityEdge> {
        schemas::assignability_edges()
    }

    fn resolve_references(
        &self,
        requests: Vec<proto::ReferenceRequest>,
    ) -> Result<Vec<Result<proto::ReferenceOutcome, proto::ReferenceError>>, proto::ProviderError>
    {
        Ok(self
            .runtime
            .block_on(self.provider().resolve_references(requests)))
    }

    fn provider_config_attribute_types(&self) -> ValidatedConfigSchema {
        AwsccProviderFactory.provider_config_attribute_types()
    }

    fn validate_config(
        &self,
        attrs: &BTreeMap<String, proto::Value>,
    ) -> Result<(), proto::ProviderError> {
        // Cross-account guardrail: when `assume_role.role_arn` parses to
        // an account id that is not in `allowed_account_ids` (when that
        // list is configured), refuse the configuration. Avoids the
        // foot-gun of an assume-role silently landing in the wrong AWS
        // account (awscc#260).
        use carina_provider_awscc::provider::assume_role::{
            check_cross_account, extract_assume_role,
        };
        let core_attrs = protocol_convert::protocol_to_core_value_map(attrs)
            .map_err(provider_config_decode_error)?;
        let assume_role = extract_assume_role(core_attrs.get("assume_role"))
            .map_err(CoreProviderError::invalid_input)?;
        if let Some(ar) = &assume_role {
            let allowed = extract_string_list(&core_attrs, "allowed_account_ids");
            check_cross_account(&ar.role_arn, &allowed)
                .map_err(CoreProviderError::invalid_input)?;
        }
        Ok(())
    }

    fn initialize(
        &mut self,
        attrs: &BTreeMap<String, proto::Value>,
    ) -> Result<proto::ReferenceScope, proto::ProviderError> {
        let core_attrs = protocol_convert::protocol_to_core_value_map(attrs)
            .map_err(provider_config_decode_error)?;
        let cfg =
            extract_account_guard_config(&core_attrs).map_err(CoreProviderError::invalid_input)?;
        let ordered_attrs: indexmap::IndexMap<_, _> = core_attrs.into_iter().collect();
        let region =
            carina_core::utils::extract_region_from_attrs(&ordered_attrs, "ap-northeast-1");
        #[cfg(target_arch = "wasm32")]
        let diagnostics = carina_plugin_sdk::ProviderDiagnostics::guest();
        #[cfg(not(target_arch = "wasm32"))]
        let diagnostics =
            carina_plugin_sdk::ProviderDiagnostics::from_session(&self.diagnostic_session);
        let provider =
            self.runtime
                .block_on(AwsccProvider::new_with_config(&region, &cfg, diagnostics));
        // Surface the account-guard rejection eagerly: if the caller's
        // AWS account violates allowed_account_ids / forbidden_account_ids,
        // initialize() must fail so the host aborts before any plan,
        // refresh, or apply step.
        if let Some(err) = provider.init_error() {
            return Err(err.clone());
        }
        self.provider = Some(provider);
        // This change declares no ReferenceForm table. Returning an available
        // scope would therefore add no capability; in particular, do not add
        // a new STS call solely to discover an account for an empty forms set.
        Ok(proto::ReferenceScope::Unavailable)
    }

    fn config_completions(&self) -> HashMap<String, Vec<proto::CompletionValue>> {
        HashMap::from([(
            "region".to_string(),
            carina_aws_types::region_completions("awscc")
                .into_iter()
                .map(|c| proto::CompletionValue {
                    value: c.value,
                    description: c.description.to_string(),
                })
                .collect(),
        )])
    }

    fn identity_attributes(&self) -> Vec<String> {
        vec!["region".to_string()]
    }

    // Protocol v6 resolves enum aliases in the host. The guest receives API
    // spellings as wire strings, so it does not expose an enum-alias callback.

    fn read(
        &self,
        id: &proto::ResourceId,
        identifier: &proto::AppliedIdentifier,
        _request: proto::ReadRequest,
    ) -> Result<proto::ReadOutcome, proto::ProviderError> {
        let core_id = protocol_convert::protocol_to_resource_id(id);
        let result =
            self.runtime
                .block_on(self.provider().read(&core_id, identifier, CoreReadRequest));
        match result {
            Ok(outcome) => read_outcome_to_proto(&outcome),
            Err(e) => Err(e),
        }
    }

    fn read_data_source(
        &self,
        resource: &proto::Resource,
    ) -> Result<proto::State, proto::ProviderError> {
        let core_data_source = protocol_data_source_to_core(resource).map_err(|error| {
            protocol_input_error("failed to decode data source", error)
                .for_resource(protocol_convert::protocol_to_resource_id(&resource.id))
        })?;
        let result = self.runtime.block_on(
            self.provider()
                .read_data_source_resource(&core_data_source.id),
        );
        match result {
            Ok(state) => protocol_convert::try_state_to_protocol(&state).map_err(|error| {
                protocol_output_error("failed to encode data-source read result", error)
                    .for_resource(core_data_source.id.clone())
            }),
            Err(e) => Err(e),
        }
    }

    fn create(
        &self,
        _id: &proto::ResourceId,
        request: proto::CreateRequest,
        attempt: SdkCreateAttempt,
    ) -> Result<SdkCreateStep<Self::CreateToken>, SdkCreateVerdict<Self::CreateToken>> {
        let normalized_resource = match self.normalize_create_resource(&request.resource) {
            Ok(resource) => resource,
            Err(error) => return Err(attempt.not_created(error)),
        };
        self.runtime.block_on(self.provider().create_step(
            normalized_resource.as_resource(),
            GuestCreateAttempt(attempt),
        ))
    }

    fn poll_create(
        &self,
        _id: &proto::ResourceId,
        request: proto::CreateRequest,
        poll: SdkPoll<Self::CreateToken>,
        poll_request: PollRequest,
    ) -> Result<SdkCreateStep<Self::CreateToken>, SdkPollVerdict<Self::CreateToken>> {
        let normalized_resource = match self.normalize_create_resource(&request.resource) {
            Ok(resource) => resource,
            Err(error) => {
                return match poll {
                    SdkPoll::Unissued(attempt, token) => {
                        let poll_after_secs = token.poll_after_secs();
                        Err(attempt.preserve(token, poll_after_secs, error))
                    }
                    SdkPoll::Issued(issued, token) => {
                        let poll_after_secs = token.poll_after_secs();
                        Err(issued.preserve(token, poll_after_secs, error).into())
                    }
                };
            }
        };
        let poll = match poll {
            SdkPoll::Unissued(attempt, token) => {
                CcCreatePoll::Unissued(GuestPollAttempt(attempt), token)
            }
            SdkPoll::Issued(issued, token) => {
                CcCreatePoll::Issued(GuestPolledIssuedCreate(issued), token)
            }
        };
        self.runtime.block_on(self.provider().poll_create_step(
            normalized_resource.as_resource(),
            poll,
            poll_request,
        ))
    }

    fn resume_create(
        &self,
        _id: &proto::ResourceId,
        request: proto::CreateRequest,
        resume: SdkResume<Self::CreateToken>,
        poll_request: PollRequest,
    ) -> Result<SdkResumeStep<Self::CreateToken>, SdkResumeVerdict<Self::CreateToken>> {
        let normalized_resource = match self.normalize_create_resource(&request.resource) {
            Ok(resource) => resource,
            Err(error) => {
                return match resume {
                    SdkResume::Unissued(attempt, token) => {
                        let poll_after_secs = token.poll_after_secs();
                        Err(attempt.preserve(token, poll_after_secs, error))
                    }
                    SdkResume::Issued(resumed, token) => {
                        let poll_after_secs = token.poll_after_secs();
                        Err(resumed.preserve(token, poll_after_secs, error))
                    }
                };
            }
        };
        let resume = match resume {
            SdkResume::Unissued(attempt, token) => {
                CcCreateResume::Unissued(GuestResumeAttempt(attempt), token)
            }
            SdkResume::Issued(resumed, token) => {
                CcCreateResume::Issued(GuestResumedCreate(resumed), token)
            }
        };
        self.runtime.block_on(self.provider().resume_create_step(
            normalized_resource.as_resource(),
            resume,
            poll_request,
        ))
    }

    fn update(
        &self,
        id: &proto::ResourceId,
        identifier: &proto::AppliedIdentifier,
        request: proto::UpdateRequest,
    ) -> Result<SdkUpdateOutcome<Self::UpdateToken>, proto::ProviderError> {
        let core_id = protocol_convert::protocol_to_resource_id(id);
        let state_after_patch = request.state_after_patch();
        let core_from =
            protocol_convert::protocol_persisted_state_to_core(&request.from).map_err(|error| {
                protocol_input_error("failed to decode persisted update state", error)
                    .for_resource(core_id.clone())
            })?;
        let core_patch =
            protocol_convert::protocol_update_patch_to_core(&request.patch).map_err(|error| {
                protocol_input_error("failed to decode update patch", error)
                    .for_resource(core_id.clone())
            })?;
        let result = self.runtime.block_on(self.provider().update_step(
            &core_id,
            identifier,
            &core_from,
            &core_patch,
        ));
        match result {
            Ok(OperationStep::Done(outcome)) => {
                update_outcome_to_sdk(&outcome, state_after_patch, &core_id)
                    .map(SdkUpdateOutcome::Done)
            }
            Ok(OperationStep::Verdict(error)) | Err(error) => Err(error),
            Ok(OperationStep::Pending {
                pending: token,
                poll_after_secs,
                ..
            }) => Ok(SdkUpdateOutcome::Pending(Pending::new(
                token,
                poll_after_secs,
            ))),
        }
    }

    fn poll_update(
        &self,
        id: &proto::ResourceId,
        identifier: &proto::AppliedIdentifier,
        request: proto::UpdateRequest,
        pending: Pending<Self::UpdateToken>,
        poll_request: PollRequest,
    ) -> Result<SdkUpdateOutcome<Self::UpdateToken>, proto::ProviderError> {
        let core_id = protocol_convert::protocol_to_resource_id(id);
        let state_after_patch = request.state_after_patch();
        let core_from =
            protocol_convert::protocol_persisted_state_to_core(&request.from).map_err(|error| {
                protocol_input_error("failed to decode persisted update state", error)
                    .for_resource(core_id.clone())
            })?;
        let core_patch =
            protocol_convert::protocol_update_patch_to_core(&request.patch).map_err(|error| {
                protocol_input_error("failed to decode update patch", error)
                    .for_resource(core_id.clone())
            })?;
        let result = self.runtime.block_on(self.provider().poll_update_step(
            &core_id,
            identifier,
            &core_from,
            &core_patch,
            pending.token,
            poll_request,
        ));
        match result {
            Ok(OperationStep::Done(outcome)) => {
                update_outcome_to_sdk(&outcome, state_after_patch, &core_id)
                    .map(SdkUpdateOutcome::Done)
            }
            Ok(OperationStep::Verdict(error)) | Err(error) => Err(error),
            Ok(OperationStep::Pending {
                pending: token,
                poll_after_secs,
                ..
            }) => Ok(SdkUpdateOutcome::Pending(Pending::new(
                token,
                poll_after_secs,
            ))),
        }
    }

    fn delete(
        &self,
        id: &proto::ResourceId,
        identifier: &proto::AppliedIdentifier,
        request: proto::DeleteRequest,
    ) -> Result<SdkDeleteOutcome<Self::DeleteToken>, proto::ProviderError> {
        let core_id = protocol_convert::protocol_to_resource_id(id);
        let core_request = Self::core_delete_request(&core_id, request)?;
        let result = self.runtime.block_on(self.provider().delete_step(
            &core_id,
            identifier,
            &core_request,
        ));
        Self::map_step(
            result,
            |()| SdkDeleteOutcome::Deleted,
            SdkDeleteOutcome::Pending,
        )
    }

    fn poll_delete(
        &self,
        id: &proto::ResourceId,
        identifier: &proto::AppliedIdentifier,
        request: proto::DeleteRequest,
        pending: PendingDelete<Self::DeleteToken>,
    ) -> Result<SdkDeleteOutcome<Self::DeleteToken>, proto::ProviderError> {
        let core_id = protocol_convert::protocol_to_resource_id(id);
        let core_request = Self::core_delete_request(&core_id, request)?;
        let result = self.runtime.block_on(self.provider().poll_delete_step(
            &core_id,
            identifier,
            &core_request,
            pending.token,
        ));
        Self::map_step(
            result,
            |()| SdkDeleteOutcome::Deleted,
            SdkDeleteOutcome::Pending,
        )
    }

    fn required_permissions(
        &self,
        id: &proto::ResourceId,
        op: carina_plugin_sdk::PlanOp,
    ) -> Result<Vec<String>, proto::ProviderError> {
        let op = match op {
            carina_plugin_sdk::PlanOp::Create => carina_core::effect::PlanOp::Create,
            carina_plugin_sdk::PlanOp::Read => carina_core::effect::PlanOp::Read,
            carina_plugin_sdk::PlanOp::Update => carina_core::effect::PlanOp::Update,
            carina_plugin_sdk::PlanOp::Delete => carina_core::effect::PlanOp::Delete,
        };
        Ok(
            schemas::generated::required_permissions(&id.resource_type, op)
                .iter()
                .map(|permission| (*permission).to_string())
                .collect(),
        )
    }

    fn normalize_state(
        &self,
        states: HashMap<String, proto::PersistedState>,
    ) -> Result<HashMap<String, proto::PersistedState>, proto::ProviderError> {
        let mut core_states = HashMap::with_capacity(states.len());
        for state in states.values() {
            let core_state =
                protocol_convert::protocol_persisted_state_to_core(state).map_err(|error| {
                    protocol_input_error("normalize_state could not decode persisted state", error)
                        .for_resource(protocol_convert::protocol_to_resource_id(&state.id))
                })?;
            core_states.insert(core_state.id.clone(), core_state);
        }
        self.runtime
            .block_on(carina_core::provider::normalize_provider_states(
                &self.normalizer,
                &mut core_states,
            ))?;
        let mut normalized = HashMap::with_capacity(core_states.len());
        for (id, state) in &core_states {
            let state =
                protocol_convert::try_state_to_persisted_protocol(state).map_err(|error| {
                    protocol_output_error("failed to encode normalized state", error)
                        .for_resource(id.clone())
                })?;
            normalized.insert(id.to_string(), state);
        }
        Ok(normalized)
    }

    fn hydrate_read_state(
        &self,
        states: &mut HashMap<String, proto::PersistedState>,
        saved_attrs: &HashMap<String, BTreeMap<String, proto::PersistedValue>>,
    ) -> Result<(), proto::ProviderError> {
        // Build a key-to-CoreResourceId lookup from states (which contain structured IDs)
        let key_to_id: HashMap<&str, CoreResourceId> = states
            .iter()
            .map(|(k, s)| (k.as_str(), protocol_convert::protocol_to_resource_id(&s.id)))
            .collect();

        let mut core_states = HashMap::with_capacity(states.len());
        for state in states.values() {
            let core_state =
                protocol_convert::protocol_persisted_state_to_core(state).map_err(|error| {
                    protocol_input_error(
                        "hydrate_read_state could not decode persisted state",
                        error,
                    )
                    .for_resource(protocol_convert::protocol_to_resource_id(&state.id))
                })?;
            core_states.insert(core_state.id.clone(), core_state);
        }
        let schemas = Self::schema_registry();
        let core_saved = decode_persisted_saved_attrs(saved_attrs, &key_to_id, &schemas)?;
        self.runtime
            .block_on(carina_core::provider::hydrate_provider_states(
                &self.normalizer,
                &mut core_states,
                core_saved.as_provider_saved_attrs(),
            ))?;
        let mut hydrated = HashMap::with_capacity(core_states.len());
        for (id, state) in &core_states {
            let state =
                protocol_convert::try_state_to_persisted_protocol(state).map_err(|error| {
                    protocol_output_error("failed to encode hydrated state", error)
                        .for_resource(id.clone())
                })?;
            hydrated.insert(id.to_string(), state);
        }
        *states = hydrated;
        Ok(())
    }
}

#[cfg(any(target_arch = "wasm32", test))]
fn decode_persisted_saved_attrs(
    saved_attrs: &HashMap<String, BTreeMap<String, proto::PersistedValue>>,
    key_to_id: &HashMap<&str, CoreResourceId>,
    registry: &SchemaRegistry,
) -> Result<LiftedSavedAttrs, CoreProviderError> {
    let persisted_saved: HashMap<CoreResourceId, PersistedAttributes> = saved_attrs
        .iter()
        .map(|(key, values)| {
            let id = key_to_id.get(key.as_str()).cloned().ok_or_else(|| {
                CoreProviderError::internal(format!(
                    "hydrate_read_state received saved attributes for unknown state key `{key}`"
                ))
            })?;
            let typed_values = protocol_convert::protocol_to_core_persisted_value_map(values)
                .map_err(|error| {
                    protocol_input_error(
                        "hydrate_read_state could not decode persisted attributes",
                        error,
                    )
                    .for_resource(id.clone())
                })?;
            let attrs = PersistedAttributes::from_typed_persisted_wire_values(typed_values)
                .map_err(|error| {
                    CoreProviderError::internal(format!(
                        "hydrate_read_state could not encode persisted attributes for `{key}`: \
                         {error}"
                    ))
                })?;
            Ok((id, attrs))
        })
        .collect::<Result<_, CoreProviderError>>()?;
    Ok(RawSavedAttrs::from_persisted(persisted_saved, HashMap::new()).lift(registry))
}

/// Pull `allowed_account_ids` / `forbidden_account_ids` / `assume_role`
/// out of the provider's configured attributes. Both lists unset/empty
/// means "no account check"; `assume_role` absent means "do not chain
/// sts:AssumeRole". Mirrors the in-process Provider wiring in
/// `lib.rs::extract_provider_config`.
#[cfg(any(target_arch = "wasm32", test))]
fn extract_account_guard_config(
    core_attrs: &HashMap<String, CoreValue>,
) -> Result<AwsccProviderConfig, String> {
    let assume_role = carina_provider_awscc::provider::assume_role::extract_assume_role(
        core_attrs.get("assume_role"),
    )?;
    Ok(AwsccProviderConfig {
        allowed_account_ids: extract_string_list(core_attrs, "allowed_account_ids"),
        forbidden_account_ids: extract_string_list(core_attrs, "forbidden_account_ids"),
        assume_role,
    })
}

/// Shared `list(string)` extractor used by both `validate_config` and
/// `extract_account_guard_config`. Drops non-string elements silently —
/// the host enforces the declared `list(string)` shape before we see
/// the value.
#[cfg(any(target_arch = "wasm32", test))]
fn extract_string_list(attrs: &HashMap<String, CoreValue>, key: &str) -> Vec<String> {
    match attrs.get(key) {
        Some(CoreValue::Concrete(ConcreteValue::List(items))) => items
            .iter()
            .filter_map(|v| match v {
                CoreValue::Concrete(ConcreteValue::String(s)) => {
                    Some(s.expose_plaintext().to_string())
                }
                _ => None,
            })
            .collect(),
        _ => Vec::new(),
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn main() {
    #[cfg(test)]
    fn assert_component_provider<P: CarinaProvider>() {}
    #[cfg(test)]
    assert_component_provider::<AwsccProcessProvider>();
    eprintln!(
        "carina-provider-awscc no longer has a native provider transport; build the wasm32-wasip2 component"
    );
    std::process::exit(1);
}

#[cfg(target_arch = "wasm32")]
carina_plugin_sdk::export_provider!(
    AwsccProcessProvider,
    aws,
    egress: ["**.amazonaws.com", "**.amazonaws.com.cn"],
);

#[cfg(target_arch = "wasm32")]
fn main() {}

#[cfg(test)]
mod tests {
    use super::*;
    use carina_core::schema::{
        CompiledAttributeType, CustomValidator, RawShape, ResourceValidation,
        TypeIdentity as CoreTypeIdentity,
    };
    use carina_core::test_utils::provider_text::{BindProviderText as _, NativeWireFixture as _};
    use carina_plugin_sdk::types::{TypeIdentity as WireTypeIdentity, ValidatorType};
    use carina_plugin_sdk::{
        ProviderSchemaCache, dispatch_custom_type_validation, provider_schema_to_json,
    };

    #[test]
    fn delete_step_rejects_read_back_phase() {
        let error = AwsccProcessProvider::map_step::<(), _, ()>(
            Ok(OperationStep::Pending {
                pending: "delete-token",
                poll_after_secs: 5,
                phase: PendingPhase::ReadBack,
            }),
            |_| panic!("a pending delete cannot be done"),
            |_| panic!("a delete cannot enter read-back polling"),
        )
        .expect_err("read-back is an internal phase error for delete");

        assert_eq!(error.variant_name(), "internal");
        assert!(
            error
                .clone()
                .into_host_for_test()
                .to_string()
                .contains("ReadBack")
        );
    }

    #[test]
    fn desired_floats_reject_non_finite_input_at_admission() {
        use carina_core::{FiniteF64, NonFiniteFloatError};

        for (number, expected) in [
            (f64::NAN, NonFiniteFloatError::NaN),
            (f64::INFINITY, NonFiniteFloatError::PositiveInfinity),
            (f64::NEG_INFINITY, NonFiniteFloatError::NegativeInfinity),
        ] {
            let result: Result<CoreValue, _> = FiniteF64::new(number)
                .map(|number| CoreValue::Concrete(ConcreteValue::Float(number)));
            assert_eq!(result, Err(expected));
        }
    }

    #[test]
    fn desired_normalization_codec_rejects_malformed_containers_with_resource_context() {
        use carina_provider_protocol::normalization::{NormalizationErrorKind, NormalizationValue};
        let id = proto::ResourceId {
            provider: "awscc".into(),
            resource_type: "s3.Bucket".into(),
            identity: "invalid-desired".into(),
        };
        // SDK guest dispatch decodes container JSON through this codec before
        // calling CarinaProvider::normalize_desired's default implementation.
        for error in [
            NormalizationValue::from_json_list("[", &id, "values").unwrap_err(),
            NormalizationValue::from_json_map("{", &id, "values").unwrap_err(),
        ] {
            assert!(matches!(error.kind, NormalizationErrorKind::Conversion));
            assert_eq!(*error.resource_id, id);
            assert_eq!(error.attribute, "values");
            let error: carina_provider_protocol::types::ProviderError = error.into();
            assert_eq!(error.resource_id.as_ref(), Some(&id));
            assert_eq!(
                error.operation.as_ref().map(ToString::to_string).as_deref(),
                Some("normalize_desired")
            );
            assert!(error.cause.is_some());
        }
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    enum CallbackScalar {
        String,
        Int,
        Float,
    }

    struct CallbackBinding {
        scalar: CallbackScalar,
        callbacks: Vec<CustomValidator>,
    }

    fn wire_identity(identity: &CoreTypeIdentity) -> WireTypeIdentity {
        WireTypeIdentity {
            provider: identity.provider.clone().unwrap_or_default(),
            segments: identity.segments.clone(),
            kind: identity.kind.clone(),
        }
    }

    fn scalar_kind(attribute_type: &CompiledAttributeType) -> CallbackScalar {
        match attribute_type.raw_shape() {
            RawShape::String { .. } => CallbackScalar::String,
            RawShape::Int { .. } => CallbackScalar::Int,
            RawShape::Float { .. } => CallbackScalar::Float,
            RawShape::Enum { base, .. } => scalar_kind(base),
            shape => panic!("callback identity must refine a scalar, got {shape:?}"),
        }
    }

    fn record_callback(
        bindings: &mut BTreeMap<CoreTypeIdentity, CallbackBinding>,
        identity: &CoreTypeIdentity,
        scalar: CallbackScalar,
        callback: &CustomValidator,
    ) {
        let binding = bindings
            .entry(identity.clone())
            .or_insert_with(|| CallbackBinding {
                scalar,
                callbacks: Vec::new(),
            });
        assert_eq!(
            binding.scalar, scalar,
            "identity {identity} has conflicting scalar kinds in validated witnesses"
        );
        binding.callbacks.push(callback.clone());
    }

    fn collect_callbacks(
        attribute_type: &CompiledAttributeType,
        bindings: &mut BTreeMap<CoreTypeIdentity, CallbackBinding>,
    ) {
        match attribute_type.raw_shape() {
            RawShape::String {
                identity, validate, ..
            }
            | RawShape::Int {
                identity, validate, ..
            }
            | RawShape::Float {
                identity, validate, ..
            } => {
                if let Some(callback) = validate {
                    let identity =
                        identity.expect("validated callback-bearing scalar must carry an identity");
                    record_callback(bindings, identity, scalar_kind(attribute_type), callback);
                }
            }
            RawShape::Enum {
                identity, validate, ..
            } => {
                if let Some(callback) = validate {
                    record_callback(bindings, identity, scalar_kind(attribute_type), callback);
                }
            }
            RawShape::List { element_type, .. } => collect_callbacks(element_type, bindings),
            RawShape::Map {
                key,
                value,
                constraints: _,
            } => {
                collect_callbacks(key, bindings);
                collect_callbacks(value, bindings);
            }
            RawShape::Struct { fields, .. } => {
                for field in fields {
                    collect_callbacks(field.field_type(), bindings);
                }
            }
            RawShape::Union(members) => {
                for member in members {
                    collect_callbacks(member, bindings);
                }
            }
            RawShape::UnloadedSchemaType(unloaded) => panic!(
                "callback collection requires a loaded schema type for {}",
                unloaded.identity(),
            ),
            RawShape::Reference { .. } | RawShape::Bool | RawShape::Duration | RawShape::Ref(_) => {
            }
        }
    }

    fn bridge_probe(scalar: CallbackScalar, text: &str) -> CoreValue {
        match scalar {
            CallbackScalar::String => {
                CoreValue::Concrete(ConcreteValue::String(text.to_string().into()))
            }
            CallbackScalar::Int => CoreValue::Concrete(ConcreteValue::Int(
                text.parse().expect("integer bridge probe"),
            )),
            CallbackScalar::Float => CoreValue::Concrete(ConcreteValue::Float(
                carina_core::FiniteF64::new(text.parse().expect("float bridge probe"))
                    .expect("finite float bridge probe"),
            )),
        }
    }

    fn probe_texts(scalar: CallbackScalar) -> &'static [&'static str] {
        match scalar {
            CallbackScalar::String => &[
                "",
                "__carina_sdk_bridge_probe__",
                "not-an-arn",
                "0",
                "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            ],
            CallbackScalar::Int => &["0", "-1", "1", "9223372036854775807"],
            CallbackScalar::Float => &["0", "-1", "1", "1.5"],
        }
    }

    fn valid_bridge_sample(identity: &CoreTypeIdentity) -> &'static str {
        if let Some(awscc_type) = schemas::catalog::AwsccType::ALL
            .iter()
            .copied()
            .find(|awscc_type| awscc_type.identity() == *identity)
        {
            return awscc_type.valid_sample();
        }
        if let Some(aws_type) = carina_aws_types::AwsType::ALL
            .iter()
            .copied()
            .find(|aws_type| aws_type.identity() == *identity)
        {
            assert!(aws_type.has_validator());
            return aws_type.valid_sample();
        }

        assert!(
            identity.provider.is_none() && identity.segments.is_empty(),
            "validated identity {identity} is neither a catalog row nor a core built-in"
        );
        match identity.kind.as_str() {
            "DomainName" => "example.com",
            "Email" => "user@example.com",
            "HttpResponseStatusCode" => "200",
            "HttpsUrl" => "https://example.com/issuer",
            "Ipv4Address" => "192.0.2.1",
            "Ipv4Cidr" => "10.0.0.0/16",
            "Ipv6Address" => "2001:db8::1",
            "Ipv6Cidr" => "2001:db8::/32",
            "PositiveInt" => "1",
            "RedirectHost" => "example.com",
            "RedirectPath" => "/#{path}",
            "RedirectPort" => "443",
            "RedirectProtocol" => "HTTPS",
            "RedirectQuery" => "next=#{path}",
            kind => panic!("core built-in validator {kind} needs a bridge sample"),
        }
    }

    fn declared_validators(
        schema: &ValidatedResourceSchema,
    ) -> &[carina_provider_protocol::types::ValidatorType] {
        match schema.parts().validation {
            ResourceValidation::Declared(validators) => validators,
            ResourceValidation::None => &[],
        }
    }

    fn out_of_range_duration_attributes() -> HashMap<String, CoreValue> {
        HashMap::from([(
            "value".to_string(),
            CoreValue::Concrete(ConcreteValue::Duration(std::time::Duration::from_secs(
                u64::MAX,
            ))),
        )])
    }

    fn out_of_range_duration_state() -> CoreAppliedState {
        CoreAppliedState::new(
            CoreResourceId::pending_with_provider("awscc", "test.OutOfRangeDurationOutput", None),
            "provider-id",
            out_of_range_duration_attributes(),
        )
        .expect("fixture identifier is non-empty")
    }

    fn assert_typed_conversion_error(error: CoreProviderError, message: &str) {
        assert!(
            error
                .clone()
                .into_host_for_test()
                .detail()
                .cause()
                .is_some_and(|cause| cause.is::<protocol_convert::ProtocolConvertError>()),
            "typed protocol-conversion cause must be retained: {error:?}"
        );
        let error = error.to_wire();
        assert_eq!(
            error.kind,
            carina_provider_protocol::types::ProviderErrorKind::Internal
        );
        assert_eq!(error.message, message);
        assert!(
            error.cause.as_deref().is_some_and(|cause| cause
                .contains("duration in seconds does not fit the provider protocol integer domain")),
            "typed protocol-conversion cause must be retained: {error:?}"
        );
    }

    #[test]
    fn out_of_range_duration_read_output_surfaces_as_a_provider_error() {
        let error = read_outcome_to_proto(&CoreReadOutcome::found(out_of_range_duration_state()))
            .expect_err(
                "out-of-range duration in provider output must be rejected without panicking",
            );
        assert_typed_conversion_error(error, "failed to encode read result");
    }

    #[test]
    fn out_of_range_duration_issued_create_output_surfaces_as_a_provider_error() {
        let error = issued_create_attributes_to_proto(&out_of_range_duration_attributes());
        assert_typed_conversion_error(
            error.expect_err("out-of-range duration in issued-create output must be rejected"),
            "failed to encode create result",
        );
    }

    #[test]
    fn out_of_range_duration_partial_create_output_surfaces_as_a_provider_error() {
        let error = partial_create_attributes_to_proto(&out_of_range_duration_attributes());
        assert_typed_conversion_error(
            error.expect_err("out-of-range duration in partial-create output must be rejected"),
            "failed to encode partial create result",
        );
    }

    #[test]
    fn out_of_range_duration_resumed_create_output_surfaces_as_a_provider_error() {
        let error = resumed_state_to_proto(&out_of_range_duration_state())
            .expect_err("out-of-range duration in resumed-create output must be rejected");
        assert_typed_conversion_error(error, "failed to encode resumed create result");
    }

    #[test]
    fn out_of_range_duration_update_output_surfaces_as_a_provider_error() {
        let state = out_of_range_duration_state();
        let id = state.id().clone();
        let outcome = carina_core::provider::UpdateOutcome::success(state);
        let state_after_patch = proto::PersistedState {
            id: protocol_convert::resource_id_to_protocol(&id),
            identifier: None,
            attributes: BTreeMap::new(),
            exists: true,
        };
        let error = update_outcome_to_sdk(&outcome, state_after_patch, &id)
            .expect_err("out-of-range duration in update output must be rejected");
        assert_typed_conversion_error(error, "failed to encode update result");
    }

    #[test]
    fn update_encoding_matches_main_post_patch_merge_for_omitted_arn() {
        let id = CoreResourceId::with_provider_identity("awscc", "s3.Bucket", "example", None);
        let digest = concat!(
            "$argon2id$v=19$m=19456,t=2,p=1$MDEyMzQ1Njc4OWFiY2RlZg$",
            "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA"
        )
        .to_string();
        let secret = proto::PersistedValue::RedactedSecret(digest);
        let arn = persisted_str("arn:aws:s3:::example");
        let tags = persisted_map(&[("env", persisted_str("prod"))]);
        let request = proto::UpdateRequest {
            from: proto::PersistedState {
                id: protocol_convert::resource_id_to_protocol(&id),
                identifier: Some("example".into()),
                attributes: BTreeMap::from([
                    ("arn".into(), arn.clone()),
                    ("notification_configuration".into(), secret.clone()),
                    (
                        "tags".into(),
                        persisted_map(&[("env", persisted_str("dev"))]),
                    ),
                ]),
                exists: true,
            },
            patch: proto::UpdatePatch {
                ops: vec![proto::PatchOp::Replace {
                    key: "tags".into(),
                    value: proto_map(&[("env", proto_str("prod"))]),
                }],
            },
        };
        let state = CoreAppliedState::from_identifier(
            id.clone(),
            CoreAppliedIdentifier::new("example").unwrap(),
            protocol_convert::protocol_to_core_value_map(&BTreeMap::from([
                ("bucket_name".into(), proto_str("example")),
                ("tags".into(), proto_map(&[("env", proto_str("prod"))])),
            ]))
            .unwrap(),
        );
        let outcomes = [
            carina_core::provider::UpdateOutcome::success(state.clone()),
            carina_core::provider::UpdateOutcome::partial_success(
                state,
                carina_core::provider_text::ProviderDiagnostic::message("arn omitted".into()),
                vec!["arn".into()].try_into().unwrap(),
            ),
        ];
        for outcome in outcomes {
            let encoded =
                update_outcome_to_sdk(&outcome, request.state_after_patch(), &id).unwrap();
            let is_partial = matches!(
                outcome.kind(),
                carina_core::provider::UpdateOutcomeKind::PartialSuccess { .. }
            );
            assert_eq!(
                matches!(encoded, proto::UpdateOutcome::PartialSuccess { .. }),
                is_partial
            );
            let state = match encoded {
                proto::UpdateOutcome::Success { state }
                | proto::UpdateOutcome::PartialSuccess { state, .. } => state,
            };
            // af3d9ff fills absent roots from post-patch state for both outcomes.
            // The retained secret remains in the persisted value domain.
            assert_eq!(
                state.attributes,
                BTreeMap::from([
                    ("arn".into(), arn.clone()),
                    ("bucket_name".into(), persisted_str("example")),
                    ("notification_configuration".into(), secret.clone()),
                    ("tags".into(), tags.clone()),
                ])
            );
        }
    }

    #[test]
    fn schemas_include_tags_validator_for_tagged_resources() {
        let provider = AwsccProcessProvider::new();
        let schemas = provider.schemas();
        let bucket = schemas
            .iter()
            .find(|schema| schema.parts().resource_type == "s3.Bucket")
            .expect("s3.bucket schema should exist");
        assert!(
            declared_validators(bucket).contains(&ValidatorType::TagsKeyValueCheck),
            "s3.bucket should have TagsKeyValueCheck validator"
        );
    }

    #[test]
    fn schemas_exclude_tags_validator_for_untagged_resources() {
        let provider = AwsccProcessProvider::new();
        let schemas = provider.schemas();
        let configs = schemas::generated::configs();
        if let Some(untagged) = configs.iter().find(|c| c.tags.is_none()) {
            let schema = schemas
                .iter()
                .find(|schema| schema.parts().resource_type == untagged.resource_type_name)
                .expect("untagged schema should exist");
            assert!(
                !declared_validators(schema).contains(&ValidatorType::TagsKeyValueCheck),
                "untagged resource should not have TagsKeyValueCheck"
            );
        }
    }

    #[test]
    fn native_and_sdk_schema_witnesses_declare_no_assignability_edges() {
        let native = AwsccProviderFactory.schema_witness();
        let provider = AwsccProcessProvider::new();
        let cache = ProviderSchemaCache::from_provider(&provider)
            .expect("AWSCC schemas must pass aggregate admission");

        assert!(native.assignability_edges().is_empty());
        assert_eq!(
            native.assignability_edges(),
            cache.witness().assignability_edges()
        );
        for witness in [&native, cache.witness()] {
            let serialized = provider_schema_to_json(witness).expect("serialize AWSCC witness");
            assert!(serialized.contains("\"assignability_edges\":[]"));
        }
    }

    #[test]
    fn every_callback_bearing_witness_dispatches_through_the_sdk_bridge() {
        let provider = AwsccProcessProvider::new();
        let expected_schema_count = schemas::generated::configs().len();
        let cache = ProviderSchemaCache::from_provider(&provider)
            .expect("every AWSCC resource and config schema must pass v12 admission");

        assert_eq!(cache.schemas().len(), expected_schema_count);
        let mut bindings = BTreeMap::new();
        for schema in cache.schemas() {
            let parts = schema.parts();
            for attribute in parts.attributes.values() {
                collect_callbacks(attribute.attr_type(), &mut bindings);
            }
            for definition in parts.defs.values() {
                collect_callbacks(definition, &mut bindings);
            }
        }
        for attribute_type in cache.config().parts().values() {
            collect_callbacks(attribute_type, &mut bindings);
        }

        assert!(
            !bindings.is_empty(),
            "generated resource and config witnesses must expose callback-bearing identities"
        );
        for (identity, binding) in bindings {
            let sample = valid_bridge_sample(&identity);
            let sample_value = bridge_probe(binding.scalar, sample);
            assert!(
                binding
                    .callbacks
                    .iter()
                    .all(|callback| callback(&sample_value).is_ok()),
                "valid sample {sample:?} must pass every direct callback for {identity}"
            );
            dispatch_custom_type_validation(&cache, &wire_identity(&identity), sample)
                .unwrap_or_else(|error| {
                    panic!(
                        "valid sample {sample:?} must pass the SDK bridge for {identity}: {error}"
                    )
                });

            let mut rejected_by_both = false;
            for text in probe_texts(binding.scalar) {
                let value = bridge_probe(binding.scalar, text);
                let direct = binding
                    .callbacks
                    .iter()
                    .all(|callback| callback(&value).is_ok());
                let bridged =
                    dispatch_custom_type_validation(&cache, &wire_identity(&identity), text)
                        .is_ok();
                assert_eq!(
                    bridged, direct,
                    "SDK dispatch for {identity} diverged from its validated witness callbacks on {text:?}"
                );
                rejected_by_both |= !direct && !bridged;
            }
            assert!(
                rejected_by_both,
                "at least one invalid probe must be rejected directly and through the SDK bridge for {identity}"
            );
        }
    }

    #[test]
    fn s3_lifecycle_calendar_callback_dispatches_through_the_witness_and_sdk_bridge() {
        let cache = ProviderSchemaCache::from_provider(&AwsccProcessProvider::new()).unwrap();
        // The catalog supplies valid_bridge_sample automatically, while the
        // aggregate witness registers the callback at both lifecycle sinks.
        let identity = schemas::catalog::AwsccType::S3LifecycleDate.identity();
        assert_eq!(valid_bridge_sample(&identity), "2030-01-01T00:00:00Z");
        for value in [
            "2030-01-01T00:00:00Z",
            "2030-01-01T00:00:00.000Z",
            "2000-02-29T00:00:00Z",
        ] {
            dispatch_custom_type_validation(&cache, &wire_identity(&identity), value)
                .unwrap_or_else(|error| panic!("{value}: {error}"));
        }
        for value in [
            "1900-02-29T00:00:00Z",
            "2030-04-31T00:00:00Z",
            "2030-01-01T00:00:01Z",
            "2030-01-01T00:00:00.001Z",
            "2030-01-01T00:00:00Z\n",
        ] {
            assert!(
                dispatch_custom_type_validation(&cache, &wire_identity(&identity), value).is_err(),
                "{value:?}: SDK dispatch must retain the calendar/midnight callback"
            );
        }
    }

    #[test]
    fn provider_config_attribute_types_declares_assume_role_struct() {
        // Mirrors the in-process Provider's assume_role declaration over
        // the WASM/proto boundary. If the proto declaration goes missing
        // or its required field changes, the host stops surfacing
        // schema-level errors for malformed assume_role blocks
        // (awscc#260).
        let provider = AwsccProcessProvider::new();
        let types = provider.provider_config_attribute_types();
        let ty = types
            .parts()
            .get("assume_role")
            .expect("assume_role must be declared as a provider config attribute");
        match ty.raw_shape() {
            RawShape::Struct { name, fields, .. } => {
                assert_eq!(name, "AssumeRole");
                let role_arn = fields
                    .iter()
                    .find(|field| field.name() == "role_arn")
                    .expect("assume_role.role_arn must be declared");
                assert!(
                    role_arn.parts().declared_input_mode.is_required(),
                    "role_arn must be required"
                );
                for opt in ["session_name", "external_id"] {
                    let f = fields
                        .iter()
                        .find(|field| field.name() == opt)
                        .unwrap_or_else(|| panic!("assume_role.{opt} must be declared"));
                    assert!(
                        !f.parts().declared_input_mode.is_required(),
                        "assume_role.{opt} must be optional"
                    );
                    assert!(matches!(
                        f.field_type().raw_shape(),
                        RawShape::String { .. }
                    ));
                }
                let duration = fields
                    .iter()
                    .find(|field| field.name() == "duration")
                    .expect("assume_role.duration must be declared");
                assert!(
                    !duration.parts().declared_input_mode.is_required(),
                    "duration must be optional"
                );
                assert!(
                    matches!(duration.field_type().raw_shape(), RawShape::Duration),
                    "duration must be a Duration so DSL literals like `30min` are accepted; \
                     was {:?}",
                    duration.field_type()
                );
            }
            other => panic!("assume_role must be a Struct, was {other:?}"),
        }
    }

    fn proto_str(s: &str) -> proto::Value {
        proto::Value::String(s.to_string())
    }

    fn proto_list_str(items: &[&str]) -> proto::Value {
        proto::Value::List(items.iter().copied().map(proto_str).collect())
    }

    fn proto_map(items: &[(&str, proto::Value)]) -> proto::Value {
        let mut m = BTreeMap::new();
        for (k, v) in items {
            m.insert((*k).to_string(), v.clone());
        }
        proto::Value::Map(m)
    }

    fn persisted_str(s: &str) -> proto::PersistedValue {
        proto::PersistedValue::String(s.to_string())
    }

    fn persisted_map(items: &[(&str, proto::PersistedValue)]) -> proto::PersistedValue {
        let mut m = BTreeMap::new();
        for (k, v) in items {
            m.insert((*k).to_string(), v.clone());
        }
        proto::PersistedValue::Map(m)
    }

    #[test]
    fn validate_config_accepts_assume_role_in_allowed_account() {
        let provider = AwsccProcessProvider::new();
        let attrs = BTreeMap::from([
            (
                "allowed_account_ids".to_string(),
                proto_list_str(&["412038850359"]),
            ),
            (
                "assume_role".to_string(),
                proto_map(&[(
                    "role_arn",
                    proto_str("arn:aws:iam::412038850359:role/delegation"),
                )]),
            ),
        ]);
        provider
            .validate_config(&attrs)
            .expect("matching allowed_account_ids must validate");
    }

    #[test]
    fn validate_config_rejects_assume_role_outside_allowed_account() {
        let provider = AwsccProcessProvider::new();
        let attrs = BTreeMap::from([
            (
                "allowed_account_ids".to_string(),
                proto_list_str(&["111111111111"]),
            ),
            (
                "assume_role".to_string(),
                proto_map(&[(
                    "role_arn",
                    proto_str("arn:aws:iam::412038850359:role/delegation"),
                )]),
            ),
        ]);
        let err = provider
            .validate_config(&attrs)
            .expect_err("cross-account role outside allowed_account_ids must fail")
            .into_host_for_test()
            .to_string();
        assert!(err.contains("412038850359"), "must name target: {err}");
        assert!(err.contains("111111111111"), "must name allow list: {err}");
    }

    #[test]
    fn provider_config_floats_reject_non_finite_input_at_admission() {
        use carina_provider_protocol::{FiniteF64, NonFiniteFloatError};

        for (number, expected) in [
            (f64::NAN, NonFiniteFloatError::NaN),
            (f64::INFINITY, NonFiniteFloatError::PositiveInfinity),
            (f64::NEG_INFINITY, NonFiniteFloatError::NegativeInfinity),
        ] {
            assert_eq!(
                FiniteF64::new(number).map(proto::Value::Float),
                Err(expected)
            );
        }
    }

    #[test]
    fn hydrate_read_state_rejects_saved_attrs_without_matching_state() {
        let provider = AwsccProcessProvider::new();
        let mut states = HashMap::new();
        let saved_attrs = HashMap::from([(
            "awscc.ec2.Vpc.orphan".to_string(),
            BTreeMap::<String, proto::PersistedValue>::new(),
        )]);

        let error = provider
            .hydrate_read_state(&mut states, &saved_attrs)
            .expect_err("an unknown saved-attribute key must not be silently discarded");

        let error = error.to_wire();
        assert!(matches!(
            error.kind,
            carina_provider_protocol::types::ProviderErrorKind::Internal
        ));
        assert!(
            error.message.contains("unknown state key"),
            "unexpected error: {}",
            error.message
        );
    }

    #[test]
    fn hydrate_read_state_does_not_restore_saved_write_only_redacted_secret() {
        let provider = AwsccProcessProvider::new();
        let digest = concat!(
            "$argon2id$v=19$m=19456,t=2,p=1$MDEyMzQ1Njc4OWFiY2RlZg$",
            "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA"
        )
        .to_string();
        let state_key = "awscc.ec2.Eip.example".to_string();
        let mut states = HashMap::from([(
            state_key.clone(),
            proto::PersistedState {
                id: proto::ResourceId {
                    provider: "awscc".to_string(),
                    resource_type: "ec2.Eip".to_string(),
                    identity: "example".to_string(),
                },
                identifier: Some("203.0.113.10".to_string()),
                attributes: BTreeMap::new(),
                exists: true,
            },
        )]);
        let saved_attrs = HashMap::from([(
            state_key,
            BTreeMap::from([(
                "address".to_string(),
                proto::PersistedValue::RedactedSecret(digest.clone()),
            )]),
        )]);

        provider
            .hydrate_read_state(&mut states, &saved_attrs)
            .expect("no-op hydration should still accept persisted saved attributes");

        let hydrated = states
            .values()
            .next()
            .expect("hydrated state must remain present");
        assert!(
            !hydrated.attributes.contains_key("address"),
            "provider hydration must not fabricate an attribute omitted by the read"
        );
    }

    #[test]
    fn persisted_saved_attr_decode_preserves_non_secret_wire_values() {
        let distribution_id = CoreResourceId::with_provider_identity(
            "awscc",
            "cloudfront.Distribution",
            "example",
            None,
        );
        let distribution_key = distribution_id.to_string();
        let distribution_wire_attrs = BTreeMap::from([(
            "distribution_config".to_string(),
            persisted_map(&[
                (
                    "aliases",
                    proto::PersistedValue::List(vec![persisted_str("cdn.example.com")]),
                ),
                ("comment", persisted_str("shape parity")),
                ("enabled", proto::PersistedValue::Bool(true)),
                (
                    "default_cache_behavior",
                    persisted_map(&[(
                        "default_ttl",
                        proto::PersistedValue::Float(
                            carina_provider_protocol::FiniteF64::new(60.5)
                                .expect("finite persisted test value"),
                        ),
                    )]),
                ),
                (
                    "origins",
                    proto::PersistedValue::List(vec![persisted_map(&[
                        ("connection_attempts", proto::PersistedValue::Int(3)),
                        ("id", persisted_str("primary")),
                    ])]),
                ),
            ]),
        )]);
        let expected_distribution =
            protocol_convert::protocol_to_core_persisted_value_map(&distribution_wire_attrs)
                .expect("ordinary persisted values should decode");

        let policy_id =
            CoreResourceId::with_provider_identity("awscc", "iam.RolePolicy", "example", None);
        let policy_key = policy_id.to_string();
        let policy_wire_attrs = BTreeMap::from([(
            "policy_document".to_string(),
            persisted_map(&[(
                "statement",
                proto::PersistedValue::List(vec![persisted_map(&[(
                    "action",
                    proto::PersistedValue::StringList(vec!["s3:GetObject".to_string()]),
                )])]),
            )]),
        )]);
        let expected_policy =
            protocol_convert::protocol_to_core_persisted_value_map(&policy_wire_attrs)
                .expect("ordinary persisted values should decode");

        let saved_attrs = HashMap::from([
            (distribution_key.clone(), distribution_wire_attrs),
            (policy_key.clone(), policy_wire_attrs),
        ]);
        let key_to_id = HashMap::from([
            (distribution_key.as_str(), distribution_id.clone()),
            (policy_key.as_str(), policy_id.clone()),
        ]);

        let decoded = decode_persisted_saved_attrs(
            &saved_attrs,
            &key_to_id,
            &AwsccProcessProvider::schema_registry(),
        )
        .expect("ordinary persisted values should decode");

        assert_eq!(
            decoded.as_provider_saved_attrs().get(&distribution_id),
            Some(&expected_distribution),
            "typed persisted decoding plus schema lifting must preserve strings, ints, floats, \
             bools, lists, and real-schema nested structs"
        );
        assert_eq!(
            decoded.as_provider_saved_attrs().get(&policy_id),
            Some(&expected_policy),
            "schema lifting must reconstruct the StringList wire shape for a real \
             Union[String, list(String)] field"
        );
    }

    /// Regression for carina-rs/carina#2025: the VPC schema must carry the
    /// `cidr_block` / `ipv4_ipam_pool_id` oneOf constraint as serializable
    /// data, so it survives the WASM plugin boundary and validate/plan can
    /// reject `awscc.ec2.Vpc {}` before any provider call.
    #[test]
    fn vpc_schema_carries_cidr_block_exclusive_group_across_proto() {
        let provider = AwsccProcessProvider::new();
        let schemas = provider.schemas();
        let vpc = schemas
            .iter()
            .find(|schema| schema.parts().resource_type == "ec2.Vpc")
            .expect("ec2.vpc schema should exist");
        let field_groups = vpc.parts().field_groups;
        assert!(
            field_groups.iter().any(|group| group.kind()
                == carina_core::schema::FieldGroupKind::ExactlyOne
                && group.fields().iter().any(|field| field == "cidr_block")
                && group
                    .fields()
                    .iter()
                    .any(|field| field == "ipv4_ipam_pool_id")),
            "vpc schema should expose cidr_block/ipv4_ipam_pool_id as a declarative \
             exactly-one field group, got: {:?}",
            field_groups
        );
    }
}
