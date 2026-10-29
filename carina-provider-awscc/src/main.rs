use std::collections::{BTreeMap, HashMap};
use std::sync::LazyLock;

use carina_plugin_sdk::{
    CarinaProvider, CreateAttempt as SdkCreateAttempt, CreateStep as SdkCreateStep,
    CreateVerdict as SdkCreateVerdict, DeleteOutcome as SdkDeleteOutcome, FoundRead,
    IssuedCreate as SdkIssuedCreate, Pending, Poll as SdkPoll, PollAttempt as SdkPollAttempt,
    PollVerdict as SdkPollVerdict, Resume as SdkResume, ResumeAttempt as SdkResumeAttempt,
    ResumeStep as SdkResumeStep, ResumeVerdict as SdkResumeVerdict,
    ResumedCreate as SdkResumedCreate, UpdateOutcome as SdkUpdateOutcome,
};
use carina_provider_protocol::OperationStep;
use carina_provider_protocol::types as proto;

use carina_core::executor::normalized::NormalizedResource;
use carina_core::protocol_convert;
use carina_core::provider::{
    AppliedIdentifier as CoreAppliedIdentifier, AppliedState as CoreAppliedState,
    DeleteRequest as CoreDeleteRequest, LiftedSavedAttrs, Observation as CoreObservation,
    PersistedAttributes, Provider, ProviderError as CoreProviderError, ProviderFactory,
    ProviderNormalizer, RawSavedAttrs, ReadOutcome as CoreReadOutcome,
    ReadRequest as CoreReadRequest, Sent as CoreSent,
};
use carina_core::resource::{ConcreteValue, ResourceId as CoreResourceId, Value as CoreValue};
use carina_core::schema::{SchemaRegistry, ValidatedConfigSchema, ValidatedResourceSchema};

use carina_provider_awscc::provider::{
    AwsccProvider, AwsccProviderConfig, CcCreateAttempt, CcCreatePoll, CcCreateResume,
    CcCreateToken, CcDeleteToken, CcIssuedCreate, CcPollAttempt, CcResumeAttempt, CcResumedCreate,
    CcUpdateToken, NotAppliedCreate, UndeterminedCreate,
};
use carina_provider_awscc::schemas;
use carina_provider_awscc::{AwsccNormalizer, AwsccProviderFactory};

const CREATE_POLL_AFTER_SECS: u32 = 5;

fn protocol_input_error(
    context: &'static str,
    error: protocol_convert::ProtocolConvertError,
) -> CoreProviderError {
    CoreProviderError::invalid_input_caused_by(context, error)
}

fn protocol_output_error(
    context: &'static str,
    error: protocol_convert::ProtocolConvertError,
) -> CoreProviderError {
    CoreProviderError::internal_caused_by(context, error)
}

fn provider_config_decode_error(error: protocol_convert::ProtocolConvertError) -> String {
    // The v12 `CarinaProvider` config methods return `Result<(), String>`, so
    // their ABI cannot retain a typed source. `CoreProviderError`'s Display
    // includes the conversion cause, preserving the complete chain in text.
    protocol_input_error("failed to decode provider configuration", error).to_string()
}

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

fn issued_create_attributes_to_proto(
    attributes: &HashMap<String, CoreValue>,
) -> Result<BTreeMap<String, proto::Value>, CoreProviderError> {
    protocol_convert::try_core_to_protocol_value_map(attributes)
        .map_err(|error| protocol_output_error("failed to encode create result", error))
}

fn partial_create_attributes_to_proto(
    attributes: &HashMap<String, CoreValue>,
) -> Result<BTreeMap<String, proto::Value>, CoreProviderError> {
    protocol_convert::try_core_to_protocol_value_map(attributes)
        .map_err(|error| protocol_output_error("failed to encode partial create result", error))
}

fn resumed_state_to_proto(
    state: &CoreAppliedState,
) -> Result<proto::AppliedState, CoreProviderError> {
    protocol_convert::try_applied_state_to_protocol(state)
        .map_err(|error| protocol_output_error("failed to encode resumed create result", error))
}

fn update_outcome_to_proto(
    outcome: &carina_core::provider::UpdateOutcome,
    state_after_patch: proto::PersistedState,
    id: &CoreResourceId,
) -> Result<proto::UpdateOutcome, CoreProviderError> {
    protocol_convert::try_update_outcome_to_protocol(outcome, state_after_patch).map_err(|error| {
        protocol_output_error("failed to encode update result", error).for_resource(id.clone())
    })
}

struct GuestCreateAttempt(SdkCreateAttempt);
struct GuestPollAttempt(SdkPollAttempt);
struct GuestIssuedCreate(SdkIssuedCreate);
struct GuestPolledIssuedCreate(SdkIssuedCreate);
struct GuestResumeAttempt(SdkResumeAttempt);
struct GuestResumedCreate(SdkResumedCreate);
struct GuestResentCreateAttempt(SdkCreateAttempt);
struct GuestResentIssuedCreate(SdkIssuedCreate);

fn observation_to_proto(
    observation: CoreObservation,
) -> proto::Observation<proto::ReadOutcome, proto::ProviderError> {
    match observation {
        CoreObservation::Read(outcome) => match read_outcome_to_proto(&outcome) {
            Ok(outcome) => proto::Observation::Read(outcome),
            Err(error) => proto::Observation::ReadFailed(error.to_wire()),
        },
        CoreObservation::ReadFailed(error) => proto::Observation::ReadFailed(error.to_wire()),
    }
}

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
        Err(error) => {
            let error = error.to_wire();
            Err(failed(
                finishing,
                proto::Observation::ReadFailed(error.clone()),
                error,
            ))
        }
    }
}

impl CcCreateAttempt for GuestCreateAttempt {
    type Step = SdkCreateStep<CcCreateToken>;
    type Verdict = SdkCreateVerdict;
    type Issued = GuestIssuedCreate;

    fn untracked(self, failure: NotAppliedCreate) -> Self::Verdict {
        self.0.untracked(failure.into_error().to_wire())
    }

    fn issued(self, identifier: CoreAppliedIdentifier, sent: CoreSent) -> Self::Issued {
        GuestIssuedCreate(self.0.issued(identifier, sent))
    }

    fn pending(
        self,
        token: CcCreateToken,
        sent: CoreSent,
        poll_after_secs: u32,
    ) -> Result<Self::Step, Self::Verdict> {
        Ok(self.0.pending(token, sent, poll_after_secs))
    }
}

impl CcPollAttempt for GuestPollAttempt {
    type Step = SdkCreateStep<CcCreateToken>;
    type Verdict = SdkPollVerdict;
    type Issued = GuestPolledIssuedCreate;

    fn untracked(self, failure: NotAppliedCreate) -> Self::Verdict {
        self.0.untracked(failure.into_error().to_wire())
    }

    fn undetermined(self, _token: CcCreateToken, failure: UndeterminedCreate) -> Self::Verdict {
        let error = failure.into_error();
        self.0.undetermined(error.to_wire())
    }

    fn issued(self, identifier: CoreAppliedIdentifier) -> Self::Issued {
        GuestPolledIssuedCreate(self.0.issued(identifier))
    }

    fn pending(
        self,
        token: CcCreateToken,
        poll_after_secs: u32,
    ) -> Result<Self::Step, Self::Verdict> {
        Ok(self.0.pending(token, poll_after_secs))
    }
}

impl CcIssuedCreate for GuestIssuedCreate {
    type Step = SdkCreateStep<CcCreateToken>;
    type Verdict = SdkCreateVerdict;

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
            |finishing, observation, error| finishing.failed(observation, error),
        )
    }

    fn partial(
        self,
        identifier: CoreAppliedIdentifier,
        attributes: HashMap<String, CoreValue>,
        reason: String,
        missing_attributes: Vec<String>,
    ) -> Result<Self::Step, Self::Verdict> {
        let finishing = self.0.canonicalize(identifier);
        finish_encoded(
            finishing,
            partial_create_attributes_to_proto(&attributes),
            |finishing, attributes| {
                finishing.partial(
                    attributes,
                    proto::PartialReadDiagnostic {
                        reason,
                        missing_attributes,
                    },
                )
            },
            |finishing, observation, error| finishing.failed(observation, error),
        )
    }

    fn failed(
        self,
        identifier: CoreAppliedIdentifier,
        observation: CoreObservation,
        error: CoreProviderError,
    ) -> Self::Verdict {
        self.0
            .canonicalize(identifier)
            .failed(observation_to_proto(observation), error.to_wire())
    }

    fn discarded(self, error: CoreProviderError) -> Self::Verdict {
        self.0.discarded(error.to_wire())
    }

    fn preserve(self, _token: CcCreateToken, error: CoreProviderError) -> Self::Verdict {
        self.0.failed(
            proto::Observation::ReadFailed(error.to_wire()),
            error.to_wire(),
        )
    }

    fn pending(
        self,
        token: CcCreateToken,
        poll_after_secs: u32,
    ) -> Result<Self::Step, Self::Verdict> {
        Ok(self.0.pending(token, poll_after_secs))
    }
}

impl CcIssuedCreate for GuestPolledIssuedCreate {
    type Step = SdkCreateStep<CcCreateToken>;
    type Verdict = SdkPollVerdict;

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
            |finishing, observation, error| finishing.failed(observation, error).into(),
        )
    }

    fn partial(
        self,
        identifier: CoreAppliedIdentifier,
        attributes: HashMap<String, CoreValue>,
        reason: String,
        missing_attributes: Vec<String>,
    ) -> Result<Self::Step, Self::Verdict> {
        let finishing = self.0.canonicalize(identifier);
        finish_encoded(
            finishing,
            partial_create_attributes_to_proto(&attributes),
            |finishing, attributes| {
                finishing.partial(
                    attributes,
                    proto::PartialReadDiagnostic {
                        reason,
                        missing_attributes,
                    },
                )
            },
            |finishing, observation, error| finishing.failed(observation, error).into(),
        )
    }

    fn failed(
        self,
        identifier: CoreAppliedIdentifier,
        observation: CoreObservation,
        error: CoreProviderError,
    ) -> Self::Verdict {
        self.0
            .canonicalize(identifier)
            .failed(observation_to_proto(observation), error.to_wire())
            .into()
    }

    fn discarded(self, error: CoreProviderError) -> Self::Verdict {
        self.0.discarded(error.to_wire()).into()
    }

    fn preserve(self, _token: CcCreateToken, error: CoreProviderError) -> Self::Verdict {
        self.0
            .failed(
                proto::Observation::ReadFailed(error.to_wire()),
                error.to_wire(),
            )
            .into()
    }

    fn pending(
        self,
        token: CcCreateToken,
        poll_after_secs: u32,
    ) -> Result<Self::Step, Self::Verdict> {
        Ok(self.0.pending(token, poll_after_secs))
    }
}

impl CcResumeAttempt for GuestResumeAttempt {
    type Step = SdkResumeStep<CcCreateToken>;
    type Verdict = SdkResumeVerdict;
    type Resumed = GuestResumedCreate;
    type Resent = GuestResentCreateAttempt;

    fn untracked(self, failure: NotAppliedCreate) -> Self::Verdict {
        self.0.untracked(failure.into_error().to_wire())
    }

    fn preserve(self, error: CoreProviderError) -> Self::Verdict {
        self.0.undetermined(error.to_wire())
    }

    fn issued(self, identifier: CoreAppliedIdentifier) -> Self::Resumed {
        GuestResumedCreate(self.0.issued(identifier))
    }

    fn pending(
        self,
        token: CcCreateToken,
        poll_after_secs: u32,
    ) -> Result<Self::Step, Self::Verdict> {
        Ok(self.0.pending(token, poll_after_secs))
    }

    fn resend(self) -> Self::Resent {
        GuestResentCreateAttempt(self.0.resend())
    }
}

impl CcResumedCreate for GuestResumedCreate {
    type Step = SdkResumeStep<CcCreateToken>;
    type Verdict = SdkResumeVerdict;

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
            |finishing, observation, error| finishing.failed(observation, error),
        )
    }

    fn failed(
        self,
        identifier: CoreAppliedIdentifier,
        observation: CoreObservation,
        error: CoreProviderError,
    ) -> Self::Verdict {
        self.0
            .canonicalize(identifier)
            .failed(observation_to_proto(observation), error.to_wire())
    }

    fn discarded(self, error: CoreProviderError) -> Self::Verdict {
        self.0.discarded(error.to_wire())
    }

    fn preserve(self, error: CoreProviderError) -> Self::Verdict {
        self.0.failed(
            proto::Observation::ReadFailed(error.to_wire()),
            error.to_wire(),
        )
    }

    fn pending(
        self,
        token: CcCreateToken,
        poll_after_secs: u32,
    ) -> Result<Self::Step, Self::Verdict> {
        Ok(self.0.pending(token, poll_after_secs))
    }
}

impl CcCreateAttempt for GuestResentCreateAttempt {
    type Step = SdkResumeStep<CcCreateToken>;
    type Verdict = SdkResumeVerdict;
    type Issued = GuestResentIssuedCreate;

    fn untracked(self, failure: NotAppliedCreate) -> Self::Verdict {
        self.0.untracked(failure.into_error().to_wire()).into()
    }

    fn issued(self, identifier: CoreAppliedIdentifier, sent: CoreSent) -> Self::Issued {
        GuestResentIssuedCreate(self.0.issued(identifier, sent))
    }

    fn pending(
        self,
        token: CcCreateToken,
        sent: CoreSent,
        poll_after_secs: u32,
    ) -> Result<Self::Step, Self::Verdict> {
        Ok(self.0.pending(token, sent, poll_after_secs).into())
    }
}

impl CcIssuedCreate for GuestResentIssuedCreate {
    type Step = SdkResumeStep<CcCreateToken>;
    type Verdict = SdkResumeVerdict;

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
            |finishing, observation, error| finishing.failed(observation, error).into(),
        )
    }

    fn partial(
        self,
        identifier: CoreAppliedIdentifier,
        attributes: HashMap<String, CoreValue>,
        reason: String,
        missing_attributes: Vec<String>,
    ) -> Result<Self::Step, Self::Verdict> {
        let finishing = self.0.canonicalize(identifier);
        finish_encoded(
            finishing,
            partial_create_attributes_to_proto(&attributes),
            |finishing, attributes| {
                finishing
                    .partial(
                        attributes,
                        proto::PartialReadDiagnostic {
                            reason,
                            missing_attributes,
                        },
                    )
                    .into()
            },
            |finishing, observation, error| finishing.failed(observation, error).into(),
        )
    }

    fn failed(
        self,
        identifier: CoreAppliedIdentifier,
        observation: CoreObservation,
        error: CoreProviderError,
    ) -> Self::Verdict {
        self.0
            .canonicalize(identifier)
            .failed(observation_to_proto(observation), error.to_wire())
            .into()
    }

    fn discarded(self, error: CoreProviderError) -> Self::Verdict {
        self.0.discarded(error.to_wire()).into()
    }

    fn preserve(self, _token: CcCreateToken, error: CoreProviderError) -> Self::Verdict {
        self.0
            .failed(
                proto::Observation::ReadFailed(error.to_wire()),
                error.to_wire(),
            )
            .into()
    }

    fn pending(
        self,
        token: CcCreateToken,
        poll_after_secs: u32,
    ) -> Result<Self::Step, Self::Verdict> {
        Ok(self.0.pending(token, poll_after_secs).into())
    }
}

struct AwsccProcessProvider {
    runtime: tokio::runtime::Runtime,
    provider: Option<AwsccProvider>,
    normalizer: AwsccNormalizer,
}

impl Default for AwsccProcessProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl AwsccProcessProvider {
    fn new() -> Self {
        #[cfg(not(target_arch = "wasm32"))]
        let runtime = tokio::runtime::Runtime::new().expect("Failed to create tokio runtime");
        #[cfg(target_arch = "wasm32")]
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_time()
            .build()
            .expect("Failed to create tokio runtime");
        Self {
            runtime,
            provider: None,
            normalizer: AwsccNormalizer,
        }
    }

    fn convert_error(e: CoreProviderError) -> proto::ProviderError {
        e.to_wire()
    }

    fn provider(&self) -> &AwsccProvider {
        self.provider
            .as_ref()
            .expect("Provider not initialized; call initialize() first")
    }

    fn schema_registry() -> &'static SchemaRegistry {
        static REGISTRY: LazyLock<SchemaRegistry> = LazyLock::new(|| {
            let mut registry = SchemaRegistry::new();
            for config in schemas::generated::configs() {
                registry.insert(
                    "awscc",
                    config
                        .schema
                        .clone()
                        .validate()
                        .unwrap_or_else(|error| panic!("{error}")),
                );
            }
            registry
        });
        &REGISTRY
    }

    fn normalize_create_resource(
        &self,
        resource: &proto::Resource,
    ) -> Result<NormalizedResource, CoreProviderError> {
        let core_resource = protocol_convert::protocol_to_resource(resource).map_err(|error| {
            protocol_input_error("failed to decode create request", error)
                .for_resource(protocol_convert::protocol_to_resource_id(&resource.id))
        })?;
        self.runtime.block_on(
            carina_core::executor::normalized::apply_desired_normalization(
                core_resource,
                &[],
                &self.normalizer,
                Self::schema_registry(),
            ),
        )
    }

    fn map_step<T, Token, Outcome>(
        result: Result<OperationStep<T, CoreProviderError, Token>, CoreProviderError>,
        done: impl FnOnce(T) -> Outcome,
        pending: impl FnOnce(Pending<Token>) -> Outcome,
    ) -> Result<Outcome, CoreProviderError> {
        match result {
            Ok(OperationStep::Done(value)) => Ok(done(value)),
            Ok(OperationStep::Verdict(error)) => Err(error),
            Ok(OperationStep::Pending {
                pending: token,
                poll_after_secs,
            }) => Ok(pending(Pending {
                token,
                poll_after_secs,
            })),
            Err(error) => Err(error),
        }
    }

    fn core_delete_request(request: proto::DeleteRequest) -> CoreDeleteRequest {
        CoreDeleteRequest {
            directives: protocol_convert::protocol_to_directives(&request.directives),
        }
    }
}

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

    fn provider_config_attribute_types(&self) -> ValidatedConfigSchema {
        AwsccProviderFactory.provider_config_attribute_types()
    }

    fn validate_config(&self, attrs: &BTreeMap<String, proto::Value>) -> Result<(), String> {
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
        let assume_role = extract_assume_role(core_attrs.get("assume_role"))?;
        if let Some(ar) = &assume_role {
            let allowed = extract_string_list(&core_attrs, "allowed_account_ids");
            check_cross_account(&ar.role_arn, &allowed)?;
        }
        Ok(())
    }

    fn initialize(&mut self, attrs: &BTreeMap<String, proto::Value>) -> Result<(), String> {
        let core_attrs = protocol_convert::protocol_to_core_value_map(attrs)
            .map_err(provider_config_decode_error)?;
        let cfg = extract_account_guard_config(&core_attrs)?;
        let ordered_attrs: indexmap::IndexMap<_, _> = core_attrs.into_iter().collect();
        let region =
            carina_core::utils::extract_region_from_attrs(&ordered_attrs, "ap-northeast-1");
        let provider = self
            .runtime
            .block_on(AwsccProvider::new_with_config(&region, &cfg));
        // Surface the account-guard rejection eagerly: if the caller's
        // AWS account violates allowed_account_ids / forbidden_account_ids,
        // initialize() must fail so the host aborts before any plan,
        // refresh, or apply step.
        if let Some(err) = provider.init_error() {
            return Err(err.to_string());
        }
        self.provider = Some(provider);
        Ok(())
    }

    fn config_completions(&self) -> HashMap<String, Vec<proto::CompletionValue>> {
        HashMap::from([(
            "region".to_string(),
            carina_aws_types::region_completions("awscc")
                .into_iter()
                .map(|c| proto::CompletionValue {
                    value: c.value,
                    description: c.description,
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
            Ok(outcome) => read_outcome_to_proto(&outcome).map_err(Self::convert_error),
            Err(e) => Err(Self::convert_error(e)),
        }
    }

    fn read_data_source(
        &self,
        resource: &proto::Resource,
    ) -> Result<proto::State, proto::ProviderError> {
        let core_data_source =
            protocol_convert::protocol_to_data_source(resource).map_err(|error| {
                Self::convert_error(
                    protocol_input_error("failed to decode data source", error)
                        .for_resource(protocol_convert::protocol_to_resource_id(&resource.id)),
                )
            })?;
        let result = self.runtime.block_on(
            self.provider()
                .read_data_source_resource(&core_data_source.id),
        );
        match result {
            Ok(state) => protocol_convert::try_state_to_protocol(&state).map_err(|error| {
                Self::convert_error(
                    protocol_output_error("failed to encode data-source read result", error)
                        .for_resource(core_data_source.id.clone()),
                )
            }),
            Err(e) => Err(Self::convert_error(e)),
        }
    }

    fn create(
        &self,
        _id: &proto::ResourceId,
        request: proto::CreateRequest,
        attempt: SdkCreateAttempt,
    ) -> Result<SdkCreateStep<Self::CreateToken>, SdkCreateVerdict> {
        let normalized_resource = match self.normalize_create_resource(&request.resource) {
            Ok(resource) => resource,
            Err(error) => return Err(attempt.untracked(Self::convert_error(error))),
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
    ) -> Result<SdkCreateStep<Self::CreateToken>, SdkPollVerdict> {
        let normalized_resource = match self.normalize_create_resource(&request.resource) {
            Ok(resource) => resource,
            Err(error) => {
                let error = Self::convert_error(error);
                return match poll {
                    SdkPoll::Unissued(attempt, _token) => Err(attempt.undetermined(error)),
                    SdkPoll::Issued(issued, token) => {
                        Ok(issued.pending(token, CREATE_POLL_AFTER_SECS))
                    }
                };
            }
        };
        let (token, poll) = match poll {
            SdkPoll::Unissued(attempt, token) => {
                (token, CcCreatePoll::Unissued(GuestPollAttempt(attempt)))
            }
            SdkPoll::Issued(issued, token) => {
                (token, CcCreatePoll::Issued(GuestPolledIssuedCreate(issued)))
            }
        };
        self.runtime.block_on(self.provider().poll_create_step(
            normalized_resource.as_resource(),
            token,
            poll,
        ))
    }

    fn resume_create(
        &self,
        _id: &proto::ResourceId,
        request: proto::CreateRequest,
        resume: SdkResume<Self::CreateToken>,
    ) -> Result<SdkResumeStep<Self::CreateToken>, SdkResumeVerdict> {
        let normalized_resource = match self.normalize_create_resource(&request.resource) {
            Ok(resource) => resource,
            Err(error) => {
                let error = Self::convert_error(error);
                return match resume {
                    SdkResume::Unissued(attempt, _token) => Err(attempt.undetermined(error)),
                    SdkResume::Issued(resumed, token) => {
                        Ok(resumed.pending(token, CREATE_POLL_AFTER_SECS))
                    }
                };
            }
        };
        let (token, resume) = match resume {
            SdkResume::Unissued(attempt, token) => {
                (token, CcCreateResume::Unissued(GuestResumeAttempt(attempt)))
            }
            SdkResume::Issued(resumed, token) => {
                (token, CcCreateResume::Issued(GuestResumedCreate(resumed)))
            }
        };
        self.runtime.block_on(self.provider().resume_create_step(
            normalized_resource.as_resource(),
            token,
            resume,
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
                Self::convert_error(
                    protocol_input_error("failed to decode persisted update state", error)
                        .for_resource(core_id.clone()),
                )
            })?;
        let core_patch =
            protocol_convert::protocol_update_patch_to_core(&request.patch).map_err(|error| {
                Self::convert_error(
                    protocol_input_error("failed to decode update patch", error)
                        .for_resource(core_id.clone()),
                )
            })?;
        let result = self.runtime.block_on(self.provider().update_step(
            &core_id,
            identifier,
            &core_from,
            &core_patch,
        ));
        match result {
            Ok(OperationStep::Done(outcome)) => {
                update_outcome_to_proto(&outcome, state_after_patch, &core_id)
                    .map(SdkUpdateOutcome::Done)
                    .map_err(Self::convert_error)
            }
            Ok(OperationStep::Verdict(error)) | Err(error) => Err(Self::convert_error(error)),
            Ok(OperationStep::Pending {
                pending: token,
                poll_after_secs,
            }) => Ok(SdkUpdateOutcome::Pending(Pending {
                token,
                poll_after_secs,
            })),
        }
    }

    fn poll_update(
        &self,
        id: &proto::ResourceId,
        identifier: &proto::AppliedIdentifier,
        request: proto::UpdateRequest,
        pending: Pending<Self::UpdateToken>,
    ) -> Result<SdkUpdateOutcome<Self::UpdateToken>, proto::ProviderError> {
        let core_id = protocol_convert::protocol_to_resource_id(id);
        let state_after_patch = request.state_after_patch();
        let core_from =
            protocol_convert::protocol_persisted_state_to_core(&request.from).map_err(|error| {
                Self::convert_error(
                    protocol_input_error("failed to decode persisted update state", error)
                        .for_resource(core_id.clone()),
                )
            })?;
        let core_patch =
            protocol_convert::protocol_update_patch_to_core(&request.patch).map_err(|error| {
                Self::convert_error(
                    protocol_input_error("failed to decode update patch", error)
                        .for_resource(core_id.clone()),
                )
            })?;
        let result = self.runtime.block_on(self.provider().poll_update_step(
            &core_id,
            identifier,
            &core_from,
            &core_patch,
            pending.token,
        ));
        match result {
            Ok(OperationStep::Done(outcome)) => {
                update_outcome_to_proto(&outcome, state_after_patch, &core_id)
                    .map(SdkUpdateOutcome::Done)
                    .map_err(Self::convert_error)
            }
            Ok(OperationStep::Verdict(error)) | Err(error) => Err(Self::convert_error(error)),
            Ok(OperationStep::Pending {
                pending: token,
                poll_after_secs,
            }) => Ok(SdkUpdateOutcome::Pending(Pending {
                token,
                poll_after_secs,
            })),
        }
    }

    fn delete(
        &self,
        id: &proto::ResourceId,
        identifier: &proto::AppliedIdentifier,
        request: proto::DeleteRequest,
    ) -> Result<SdkDeleteOutcome<Self::DeleteToken>, proto::ProviderError> {
        let core_id = protocol_convert::protocol_to_resource_id(id);
        let core_request = Self::core_delete_request(request);
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
        .map_err(Self::convert_error)
    }

    fn poll_delete(
        &self,
        id: &proto::ResourceId,
        identifier: &proto::AppliedIdentifier,
        request: proto::DeleteRequest,
        pending: Pending<Self::DeleteToken>,
    ) -> Result<SdkDeleteOutcome<Self::DeleteToken>, proto::ProviderError> {
        let core_id = protocol_convert::protocol_to_resource_id(id);
        let core_request = Self::core_delete_request(request);
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
        .map_err(Self::convert_error)
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

    fn normalize_desired(
        &self,
        resources: Vec<proto::Resource>,
    ) -> Result<Vec<proto::Resource>, proto::ProviderError> {
        let mut core_resources = Vec::with_capacity(resources.len());
        for resource in &resources {
            let resource = protocol_convert::protocol_to_resource(resource).map_err(|error| {
                Self::convert_error(
                    protocol_input_error("failed to decode desired resource", error)
                        .for_resource(protocol_convert::protocol_to_resource_id(&resource.id)),
                )
            })?;
            core_resources.push(resource.into());
        }
        // Guest-side: drive the now-async normalizer on the guest's own
        // outermost runtime — same pattern as the guest's `Provider`
        // CRUD methods. Not a nested runtime: the host drives the WASM
        // call, the guest drives its internal async (carina#3112).
        self.runtime
            .block_on(self.normalizer.normalize_desired(&mut core_resources))
            .map_err(Self::convert_error)?;
        let mut normalized = Vec::with_capacity(core_resources.len());
        for resource in &core_resources {
            let resource =
                protocol_convert::try_resource_to_protocol(resource).map_err(|error| {
                    Self::convert_error(
                        protocol_output_error("failed to encode normalized resource", error)
                            .for_resource(resource.id.clone()),
                    )
                })?;
            normalized.push(resource);
        }
        Ok(normalized)
    }

    fn normalize_state(
        &self,
        states: HashMap<String, proto::PersistedState>,
    ) -> Result<HashMap<String, proto::PersistedState>, proto::ProviderError> {
        let mut core_states = HashMap::with_capacity(states.len());
        for state in states.values() {
            let core_state =
                protocol_convert::protocol_persisted_state_to_core(state).map_err(|error| {
                    Self::convert_error(
                        protocol_input_error(
                            "normalize_state could not decode persisted state",
                            error,
                        )
                        .for_resource(protocol_convert::protocol_to_resource_id(&state.id)),
                    )
                })?;
            core_states.insert(core_state.id.clone(), core_state);
        }
        self.runtime
            .block_on(carina_core::provider::normalize_provider_states(
                &self.normalizer,
                &mut core_states,
            ))
            .map_err(Self::convert_error)?;
        let mut normalized = HashMap::with_capacity(core_states.len());
        for (id, state) in &core_states {
            let state =
                protocol_convert::try_state_to_persisted_protocol(state).map_err(|error| {
                    Self::convert_error(
                        protocol_output_error("failed to encode normalized state", error)
                            .for_resource(id.clone()),
                    )
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
                    Self::convert_error(
                        protocol_input_error(
                            "hydrate_read_state could not decode persisted state",
                            error,
                        )
                        .for_resource(protocol_convert::protocol_to_resource_id(&state.id)),
                    )
                })?;
            core_states.insert(core_state.id.clone(), core_state);
        }
        let core_saved =
            decode_persisted_saved_attrs(saved_attrs, &key_to_id, Self::schema_registry())
                .map_err(Self::convert_error)?;
        self.runtime
            .block_on(carina_core::provider::hydrate_provider_states(
                &self.normalizer,
                &mut core_states,
                core_saved.as_provider_saved_attrs(),
            ))
            .map_err(Self::convert_error)?;
        let mut hydrated = HashMap::with_capacity(core_states.len());
        for (id, state) in &core_states {
            let state =
                protocol_convert::try_state_to_persisted_protocol(state).map_err(|error| {
                    Self::convert_error(
                        protocol_output_error("failed to encode hydrated state", error)
                            .for_resource(id.clone()),
                    )
                })?;
            hydrated.insert(id.to_string(), state);
        }
        *states = hydrated;
        Ok(())
    }
}

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
    fn assert_component_provider<P: CarinaProvider>() {}
    assert_component_provider::<AwsccProcessProvider>();
    eprintln!(
        "carina-provider-awscc no longer has a native provider transport; build the wasm32-wasip2 component"
    );
    std::process::exit(1);
}

#[cfg(target_arch = "wasm32")]
carina_plugin_sdk::export_provider!(AwsccProcessProvider, http);

#[cfg(target_arch = "wasm32")]
fn main() {}

#[cfg(test)]
mod tests {
    use super::*;
    use carina_core::schema::{
        AttributeType, CustomValidator, RawShape, ResourceValidation,
        TypeIdentity as CoreTypeIdentity,
    };
    use carina_plugin_sdk::types::{TypeIdentity as WireTypeIdentity, ValidatorType};
    use carina_plugin_sdk::{ProviderSchemaCache, dispatch_custom_type_validation};

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

    fn scalar_kind(attribute_type: &AttributeType) -> CallbackScalar {
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
        attribute_type: &AttributeType,
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
            RawShape::Map { key, value } => {
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
            RawShape::Bool | RawShape::Duration | RawShape::Ref(_) => {}
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
                text.parse().expect("float bridge probe"),
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
            "Email" => "user@example.com",
            "HttpResponseStatusCode" => "200",
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

    fn non_finite_attributes() -> HashMap<String, CoreValue> {
        HashMap::from([(
            "value".to_string(),
            CoreValue::Concrete(ConcreteValue::Float(f64::NAN)),
        )])
    }

    fn non_finite_state() -> CoreAppliedState {
        CoreAppliedState::new(
            CoreResourceId::pending_with_provider("awscc", "test.NonFiniteOutput", None),
            "provider-id",
            non_finite_attributes(),
        )
        .expect("fixture identifier is non-empty")
    }

    fn assert_typed_conversion_error(error: CoreProviderError, message: &str) {
        let error = error.to_wire();
        assert_eq!(error.kind, proto::ProviderErrorKind::Internal);
        assert_eq!(error.message, message);
        assert!(
            error
                .cause
                .as_deref()
                .is_some_and(|cause| cause.contains("non-finite float")),
            "typed protocol-conversion cause must be retained: {error:?}"
        );
    }

    #[test]
    fn non_finite_read_output_surfaces_as_a_provider_error() {
        let error = read_outcome_to_proto(&CoreReadOutcome::found(non_finite_state()))
            .expect_err("non-finite provider output must be rejected without panicking");
        assert_typed_conversion_error(error, "failed to encode read result");
    }

    #[test]
    fn non_finite_issued_create_output_surfaces_as_a_provider_error() {
        let error = issued_create_attributes_to_proto(&non_finite_attributes());
        assert_typed_conversion_error(
            error.expect_err("non-finite issued-create output must be rejected"),
            "failed to encode create result",
        );
    }

    #[test]
    fn non_finite_partial_create_output_surfaces_as_a_provider_error() {
        let error = partial_create_attributes_to_proto(&non_finite_attributes());
        assert_typed_conversion_error(
            error.expect_err("non-finite partial-create output must be rejected"),
            "failed to encode partial create result",
        );
    }

    #[test]
    fn non_finite_resumed_create_output_surfaces_as_a_provider_error() {
        let error = resumed_state_to_proto(&non_finite_state())
            .expect_err("non-finite resumed-create output must be rejected");
        assert_typed_conversion_error(error, "failed to encode resumed create result");
    }

    #[test]
    fn non_finite_update_output_surfaces_as_a_provider_error() {
        let state = non_finite_state();
        let id = state.id().clone();
        let outcome = carina_core::provider::UpdateOutcome::Success { state };
        let state_after_patch = proto::PersistedState {
            id: protocol_convert::resource_id_to_protocol(&id),
            identifier: None,
            attributes: BTreeMap::new(),
            exists: true,
        };
        let error = update_outcome_to_proto(&outcome, state_after_patch, &id)
            .expect_err("non-finite update output must be rejected");
        assert_typed_conversion_error(error, "failed to encode update result");
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
            RawShape::Struct { name, fields } => {
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
            .expect_err("cross-account role outside allowed_account_ids must fail");
        assert!(err.contains("412038850359"), "must name target: {err}");
        assert!(err.contains("111111111111"), "must name allow list: {err}");
    }

    #[test]
    fn provider_config_decode_error_keeps_the_conversion_cause_in_text() {
        let provider = AwsccProcessProvider::new();
        let attrs = BTreeMap::from([("region".to_string(), proto::Value::Float(f64::NAN))]);
        let error = provider
            .validate_config(&attrs)
            .expect_err("non-finite provider configuration must fail decoding");
        assert!(
            error.contains("failed to decode provider configuration")
                && error.contains("non-finite float"),
            "v12's string-only config error must retain the complete chain: {error}"
        );
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

        assert!(matches!(error.kind, proto::ProviderErrorKind::Internal));
        assert!(
            error.message.contains("unknown state key"),
            "unexpected error: {}",
            error.message
        );
    }

    #[test]
    fn hydrate_read_state_round_trips_saved_write_only_redacted_secret() {
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
            .expect("saved write-only redacted secrets must survive hydration");

        let hydrated = states
            .values()
            .next()
            .expect("hydrated state must remain present");
        assert_eq!(
            hydrated.attributes.get("address"),
            Some(&proto::PersistedValue::RedactedSecret(digest)),
            "state-side serialization must preserve the structural redacted secret"
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
                    persisted_map(&[("default_ttl", proto::PersistedValue::Float(60.5))]),
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
            AwsccProcessProvider::schema_registry(),
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
        let exclusive_required = vpc.parts().exclusive_required;
        assert!(
            exclusive_required
                .iter()
                .any(|g| g.contains(&"cidr_block".to_string())
                    && g.contains(&"ipv4_ipam_pool_id".to_string())),
            "vpc schema should expose cidr_block/ipv4_ipam_pool_id as a declarative \
             exclusive_required group, got: {:?}",
            exclusive_required
        );
    }
}
