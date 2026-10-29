use std::collections::{BTreeMap, HashMap};
use std::sync::LazyLock;

mod convert;
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
use carina_core::provider::{
    AppliedIdentifier as CoreAppliedIdentifier, AppliedState as CoreAppliedState,
    DeleteRequest as CoreDeleteRequest, LiftedSavedAttrs, Observation as CoreObservation,
    PersistedAttributes, Provider, ProviderError as CoreProviderError, ProviderFactory,
    ProviderNormalizer, RawSavedAttrs, ReadRequest as CoreReadRequest, Sent as CoreSent,
};
use carina_core::resource::{
    ConcreteValue, ResourceId as CoreResourceId, State as CoreState, Value as CoreValue,
};
use carina_core::schema::{SchemaRegistry, ValidatedConfigSchema, ValidatedResourceSchema};

use carina_provider_awscc::provider::{
    AwsccProvider, AwsccProviderConfig, CcCreateAttempt, CcCreatePoll, CcCreateResume,
    CcCreateToken, CcDeleteToken, CcIssuedCreate, CcPollAttempt, CcResumeAttempt, CcResumedCreate,
    CcUpdateToken, NotAppliedCreate, UndeterminedCreate,
};
use carina_provider_awscc::schemas;
use carina_provider_awscc::{AwsccNormalizer, AwsccProviderFactory};

const CREATE_POLL_AFTER_SECS: u32 = 5;

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
        CoreObservation::Read(outcome) => {
            proto::Observation::Read(convert::core_to_proto_read_outcome(outcome))
        }
        CoreObservation::ReadFailed(error) => proto::Observation::ReadFailed(error.to_wire()),
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
    ) -> Self::Step {
        self.0
            .canonicalize(identifier)
            .succeeded(convert::core_to_proto_value_map(&attributes))
    }

    fn partial(
        self,
        identifier: CoreAppliedIdentifier,
        attributes: HashMap<String, CoreValue>,
        reason: String,
        missing_attributes: Vec<String>,
    ) -> Self::Step {
        self.0.canonicalize(identifier).partial(
            convert::core_to_proto_value_map(&attributes),
            proto::PartialReadDiagnostic {
                reason,
                missing_attributes,
            },
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
    ) -> Self::Step {
        self.0
            .canonicalize(identifier)
            .succeeded(convert::core_to_proto_value_map(&attributes))
    }

    fn partial(
        self,
        identifier: CoreAppliedIdentifier,
        attributes: HashMap<String, CoreValue>,
        reason: String,
        missing_attributes: Vec<String>,
    ) -> Self::Step {
        self.0.canonicalize(identifier).partial(
            convert::core_to_proto_value_map(&attributes),
            proto::PartialReadDiagnostic {
                reason,
                missing_attributes,
            },
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

    fn completed(self, identifier: CoreAppliedIdentifier, state: CoreAppliedState) -> Self::Step {
        self.0
            .canonicalize(identifier)
            .completed(FoundRead::from_applied_state(
                convert::core_to_proto_applied_state(&state),
            ))
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
    ) -> Self::Step {
        self.0
            .canonicalize(identifier)
            .succeeded(convert::core_to_proto_value_map(&attributes))
            .into()
    }

    fn partial(
        self,
        identifier: CoreAppliedIdentifier,
        attributes: HashMap<String, CoreValue>,
        reason: String,
        missing_attributes: Vec<String>,
    ) -> Self::Step {
        self.0
            .canonicalize(identifier)
            .partial(
                convert::core_to_proto_value_map(&attributes),
                proto::PartialReadDiagnostic {
                    reason,
                    missing_attributes,
                },
            )
            .into()
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
        let core_resource = convert::proto_to_core_resource(resource);
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
            directives: carina_core::resource::Directives {
                force_delete: request.directives.force_delete,
                create_before_destroy: request.directives.create_before_destroy,
                prevent_destroy: request.directives.prevent_destroy,
                depends_on: Vec::new(),
                provider_instance: None,
            },
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
        let core_attrs = convert::proto_to_core_value_map(attrs);
        let assume_role = extract_assume_role(core_attrs.get("assume_role"))?;
        if let Some(ar) = &assume_role {
            let allowed = extract_string_list(&core_attrs, "allowed_account_ids");
            check_cross_account(&ar.role_arn, &allowed)?;
        }
        Ok(())
    }

    fn initialize(&mut self, attrs: &BTreeMap<String, proto::Value>) -> Result<(), String> {
        let core_attrs = convert::proto_to_core_value_map(attrs);
        let region = if let Some(CoreValue::Concrete(ConcreteValue::String(region))) =
            core_attrs.get("region")
        {
            region.expose_plaintext().to_string()
        } else {
            "ap-northeast-1".to_string()
        };
        let cfg = extract_account_guard_config(&core_attrs)?;
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
        let core_id = convert::proto_to_core_resource_id(id);
        let result =
            self.runtime
                .block_on(self.provider().read(&core_id, identifier, CoreReadRequest));
        match result {
            Ok(outcome) => Ok(convert::core_to_proto_read_outcome(outcome)),
            Err(e) => Err(Self::convert_error(e)),
        }
    }

    fn read_data_source(
        &self,
        resource: &proto::Resource,
    ) -> Result<proto::State, proto::ProviderError> {
        let core_data_source = convert::proto_to_core_data_source(resource);
        let result = self.runtime.block_on(
            self.provider()
                .read_data_source_resource(&core_data_source.id),
        );
        match result {
            Ok(state) => Ok(convert::core_to_proto_state(&state)),
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
        let core_id = convert::proto_to_core_resource_id(id);
        let state_after_patch = request.state_after_patch();
        let core_from = convert::proto_to_core_persisted_state(&request.from).map_err(|error| {
            Self::convert_error(
                CoreProviderError::internal_caused_by(
                    "failed to decode persisted update state",
                    error,
                )
                .for_resource(core_id.clone()),
            )
        })?;
        let core_patch = convert::proto_to_core_update_patch(&request.patch);
        let result = self.runtime.block_on(self.provider().update_step(
            &core_id,
            identifier,
            &core_from,
            &core_patch,
        ));
        Self::map_step(
            result,
            |outcome| {
                SdkUpdateOutcome::Done(convert::core_to_proto_update_outcome(
                    outcome,
                    state_after_patch,
                ))
            },
            SdkUpdateOutcome::Pending,
        )
        .map_err(Self::convert_error)
    }

    fn poll_update(
        &self,
        id: &proto::ResourceId,
        identifier: &proto::AppliedIdentifier,
        request: proto::UpdateRequest,
        pending: Pending<Self::UpdateToken>,
    ) -> Result<SdkUpdateOutcome<Self::UpdateToken>, proto::ProviderError> {
        let core_id = convert::proto_to_core_resource_id(id);
        let state_after_patch = request.state_after_patch();
        let core_from = convert::proto_to_core_persisted_state(&request.from).map_err(|error| {
            Self::convert_error(
                CoreProviderError::internal_caused_by(
                    "failed to decode persisted update state",
                    error,
                )
                .for_resource(core_id.clone()),
            )
        })?;
        let core_patch = convert::proto_to_core_update_patch(&request.patch);
        let result = self.runtime.block_on(self.provider().poll_update_step(
            &core_id,
            identifier,
            &core_from,
            &core_patch,
            pending.token,
        ));
        Self::map_step(
            result,
            |outcome| {
                SdkUpdateOutcome::Done(convert::core_to_proto_update_outcome(
                    outcome,
                    state_after_patch,
                ))
            },
            SdkUpdateOutcome::Pending,
        )
        .map_err(Self::convert_error)
    }

    fn delete(
        &self,
        id: &proto::ResourceId,
        identifier: &proto::AppliedIdentifier,
        request: proto::DeleteRequest,
    ) -> Result<SdkDeleteOutcome<Self::DeleteToken>, proto::ProviderError> {
        let core_id = convert::proto_to_core_resource_id(id);
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
        let core_id = convert::proto_to_core_resource_id(id);
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
        let mut core_resources: Vec<_> = resources
            .iter()
            .map(|resource| convert::proto_to_core_resource(resource).into())
            .collect();
        // Guest-side: drive the now-async normalizer on the guest's own
        // outermost runtime — same pattern as the guest's `Provider`
        // CRUD methods. Not a nested runtime: the host drives the WASM
        // call, the guest drives its internal async (carina#3112).
        self.runtime
            .block_on(self.normalizer.normalize_desired(&mut core_resources))
            .map_err(Self::convert_error)?;
        Ok(core_resources
            .iter()
            .map(convert::core_to_proto_resource)
            .collect())
    }

    fn normalize_state(
        &self,
        states: HashMap<String, proto::PersistedState>,
    ) -> Result<HashMap<String, proto::PersistedState>, proto::ProviderError> {
        let mut core_states: HashMap<CoreResourceId, CoreState> = states
            .iter()
            .map(|(key, state)| {
                convert::proto_to_core_persisted_state(state)
                    .map(|core_state| (core_state.id.clone(), core_state))
                    .map_err(|error| {
                        CoreProviderError::internal(format!(
                            "normalize_state could not decode persisted state `{key}`: {error}"
                        ))
                    })
            })
            .collect::<Result<_, _>>()
            .map_err(Self::convert_error)?;
        self.runtime
            .block_on(carina_core::provider::normalize_provider_states(
                &self.normalizer,
                &mut core_states,
            ))
            .map_err(Self::convert_error)?;
        Ok(core_states
            .iter()
            .map(|(id, state)| {
                (
                    id.to_string(),
                    convert::core_to_proto_persisted_state(state),
                )
            })
            .collect())
    }

    fn hydrate_read_state(
        &self,
        states: &mut HashMap<String, proto::PersistedState>,
        saved_attrs: &HashMap<String, BTreeMap<String, proto::PersistedValue>>,
    ) -> Result<(), proto::ProviderError> {
        // Build a key-to-CoreResourceId lookup from states (which contain structured IDs)
        let key_to_id: HashMap<&str, CoreResourceId> = states
            .iter()
            .map(|(k, s)| (k.as_str(), convert::proto_to_core_resource_id(&s.id)))
            .collect();

        let mut core_states: HashMap<CoreResourceId, CoreState> = states
            .iter()
            .map(|(key, state)| {
                convert::proto_to_core_persisted_state(state)
                    .map(|core_state| (core_state.id.clone(), core_state))
                    .map_err(|error| {
                        CoreProviderError::internal(format!(
                            "hydrate_read_state could not decode persisted state `{key}`: {error}"
                        ))
                    })
            })
            .collect::<Result<_, _>>()
            .map_err(Self::convert_error)?;
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
        *states = core_states
            .iter()
            .map(|(id, state)| {
                (
                    id.to_string(),
                    convert::core_to_proto_persisted_state(state),
                )
            })
            .collect();
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
            let typed_values =
                convert::proto_to_core_persisted_value_map(values).map_err(|error| {
                    CoreProviderError::internal(format!(
                        "hydrate_read_state could not decode persisted attributes for `{key}`: \
                         {error}"
                    ))
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
    use carina_core::schema::{RawShape, ResourceValidation};
    use carina_plugin_sdk::types::{TypeIdentity, ValidatorType};
    use carina_plugin_sdk::{ProviderSchemaCache, dispatch_custom_type_validation};

    fn ident(segments: &[&str], kind: &str) -> TypeIdentity {
        TypeIdentity {
            provider: "aws".to_string(),
            segments: segments
                .iter()
                .map(|segment| (*segment).to_string())
                .collect(),
            kind: kind.to_string(),
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
    fn provider_schema_cache_admits_all_schemas_and_dispatches_region() {
        let provider = AwsccProcessProvider::new();
        let expected_schema_count = schemas::generated::configs().len();
        let cache = ProviderSchemaCache::from_provider(&provider)
            .expect("every AWSCC resource and config schema must pass v12 admission");

        assert_eq!(cache.schemas().len(), expected_schema_count);
        assert!(cache.config().parts().contains_key("region"));

        let region = ident(&[], "Region");
        dispatch_custom_type_validation(&cache, &region, "ap-northeast-1")
            .expect("a valid AWS Region must pass every declared callback");
        assert!(
            dispatch_custom_type_validation(&cache, &region, "not-a-region").is_err(),
            "an invalid AWS Region must fail declared callback dispatch"
        );

        for (identity, valid, invalid) in [
            (
                ident(&["ec2", "FlowLog"], "MaxAggregationInterval"),
                "60",
                "61",
            ),
            (ident(&["logs", "LogGroup"], "RetentionInDays"), "30", "2"),
        ] {
            dispatch_custom_type_validation(&cache, &identity, valid)
                .unwrap_or_else(|error| panic!("valid {identity:?} value failed: {error}"));
            assert!(
                dispatch_custom_type_validation(&cache, &identity, invalid).is_err(),
                "invalid {identity:?} value must fail its Int callback"
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
            convert::proto_to_core_persisted_value_map(&distribution_wire_attrs)
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
        let expected_policy = convert::proto_to_core_persisted_value_map(&policy_wire_attrs)
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
