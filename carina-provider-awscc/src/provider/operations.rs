//! High-level resource operations (read, create, update, delete).
//!
//! This module implements the main resource lifecycle operations that bridge
//! between DSL resources and the Cloud Control API. It handles attribute mapping,
//! tags, special cases, and default values.

use std::collections::HashMap;
use std::future::Future;
use std::time::Duration;

use carina_core::provider::{
    AppliedIdentifier, AppliedState, CreateOutcome, DeleteRequest, ProviderError, ProviderResult,
    ReadOutcome, UpdateOutcome, UpdatePatch,
};
use carina_core::resource::{ConcreteValue, Directives, Resource, ResourceId, State, Value};
use carina_core::schema::{AttributeSchema, AttributeType, Schema, Shape};
use carina_provider_protocol::{
    DEFAULT_OPERATION_BUDGET, DriveOutcome, OperationKind, OperationStep, drive_pending,
};
use indexmap::IndexMap;
use serde_json::json;

use super::conversion::{aws_value_to_dsl_with_defs, dsl_value_to_aws_with_defs};
use super::tags::tags_provider_name;
use super::update::{UpdatePatchPlan, build_update_patches};
use super::{
    AwsccProvider, CcCreateToken, CcDeleteToken, CcRequestPhase, CcUpdateToken, get_schema_config,
};
use crate::provider::arn_synthesis::SynthesisStatus;
use crate::provider::cloudcontrol::WaitOutcome;
use crate::provider::special_cases::internet_gateway_detach_patch;
use crate::schemas::config::AwsccSchemaConfig;

pub type CcCreateStep = OperationStep<CreateOutcome, CcCreateToken>;
pub type CcUpdateStep = OperationStep<UpdateOutcome, CcUpdateToken>;
pub type CcDeleteStep = OperationStep<(), CcDeleteToken>;

async fn drive_operation<T, Token, Initial, Poll, PollFuture, Sleep, SleepFuture>(
    id: &ResourceId,
    kind: OperationKind,
    budget: Duration,
    initial: Initial,
    poll: Poll,
    sleep: Sleep,
) -> ProviderResult<T>
where
    Initial: Future<Output = ProviderResult<OperationStep<T, Token>>>,
    Poll: FnMut(Token) -> PollFuture,
    PollFuture: Future<Output = ProviderResult<OperationStep<T, Token>>>,
    Sleep: FnMut(Duration) -> SleepFuture,
    SleepFuture: Future<Output = ()>,
{
    match drive_pending(budget, initial, poll, sleep).await? {
        DriveOutcome::Done(value) => Ok(value),
        DriveOutcome::OverBudget => {
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
    }
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
        // `has_tags` can describe a differently named attribute such as HostedZoneTags.
        if let Some(tags_aws_name) = tags_provider_name(config) {
            let tags_map = props
                .get(tags_aws_name)
                .and_then(|value| value.as_array())
                .map(|tags| self.parse_tags(tags))
                .unwrap_or_default();
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
    ) -> serde_json::Value {
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
        if let Some(tags_aws_name) = tags_provider_name(config) {
            let tags = self.build_tags(resource.get_attr("tags"));
            if !tags.is_empty() {
                desired_state.insert(tags_aws_name.to_string(), json!(tags));
            }
        }
        self.set_default_values(&resource.id.resource_type, &mut desired_state);
        serde_json::Value::Object(desired_state)
    }

    /// Perform the initial, non-blocking CloudControl create step.
    pub async fn create_step(&self, resource: &Resource) -> ProviderResult<CcCreateStep> {
        self.ensure_initialized(&resource.id)?;
        let config = get_schema_config(&resource.id.resource_type).ok_or_else(|| {
            ProviderError::internal(format!(
                "Unknown resource type: {}",
                resource.id.resource_type
            ))
            .for_resource(resource.id.clone())
        })?;
        let step = self
            .cc_create_step(
                config.aws_type_name,
                self.create_desired_state(resource, config),
                config.schema.operation_config.as_ref(),
            )
            .await
            .map_err(|e| e.for_resource(resource.id.clone()))?;
        self.finish_create_step(resource, config, step).await
    }

    /// Perform one CloudControl status or re-issue step for create.
    pub async fn poll_create_step(
        &self,
        resource: &Resource,
        token: CcCreateToken,
    ) -> ProviderResult<CcCreateStep> {
        self.ensure_initialized(&resource.id)?;
        let config = get_schema_config(&resource.id.resource_type).ok_or_else(|| {
            ProviderError::internal(format!(
                "Unknown resource type: {}",
                resource.id.resource_type
            ))
            .for_resource(resource.id.clone())
        })?;
        let step = self
            .cc_poll_create_step(
                config.aws_type_name,
                self.create_desired_state(resource, config),
                config.schema.operation_config.as_ref(),
                token,
            )
            .await
            .map_err(|e| e.for_resource(resource.id.clone()))?;
        self.finish_create_step(resource, config, step).await
    }

    async fn finish_create_step(
        &self,
        resource: &Resource,
        config: &AwsccSchemaConfig,
        step: OperationStep<WaitOutcome, CcCreateToken>,
    ) -> ProviderResult<CcCreateStep> {
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
            OperationStep::Done(outcome) => outcome,
        };

        let (identifier, handler_failure) = match outcome {
            WaitOutcome::Success { identifier } => (identifier, None),
            WaitOutcome::PartialOrFailed {
                identifier,
                status_message,
            } => (identifier, Some(status_message)),
        };
        let identifier = applied_identifier(&resource.id, "create", identifier)?;
        let read_back = self.read_resource(&resource.id, &identifier).await;
        let outcome = self
            .create_outcome_from_read(
                resource,
                config,
                identifier,
                read_back,
                handler_failure.as_deref(),
            )
            .await;
        Ok(OperationStep::Done(outcome))
    }

    async fn create_outcome_from_read(
        &self,
        resource: &Resource,
        config: &AwsccSchemaConfig,
        wait_identifier: AppliedIdentifier,
        read_back: ProviderResult<ReadOutcome>,
        handler_failure: Option<&str>,
    ) -> CreateOutcome {
        let read_state = match read_back {
            Ok(ReadOutcome::Found { state }) => state,
            Ok(ReadOutcome::NotFound { .. }) => {
                let state = AppliedState::from_identifier(
                    resource.id.clone(),
                    wait_identifier,
                    resource.resolved_attributes(),
                );
                return CreateOutcome::partial_success(
                    state,
                    read_back_reason(handler_failure, "read-back returned not_found"),
                    vec!["state".to_string()],
                );
            }
            Err(read_error) => {
                let state = AppliedState::from_identifier(
                    resource.id.clone(),
                    wait_identifier,
                    HashMap::new(),
                );
                let missing_attributes = nonempty_missing_attributes(
                    config
                        .schema
                        .attributes
                        .keys()
                        .filter(|name| resource.get_attr(name.as_str()).is_some())
                        .cloned()
                        .collect(),
                );
                return CreateOutcome::partial_success(
                    state,
                    read_back_reason(handler_failure, format!("read error: {read_error}")),
                    missing_attributes,
                );
            }
        };

        let mut attributes = merge_desired_attributes(read_state.attributes(), resource, config);
        let canonical_identifier =
            match canonicalize_identifier_from_read(config, &resource.id, &attributes) {
                Ok(Some(identifier)) => identifier,
                Ok(None) => {
                    let missing_attributes =
                        missing_primary_identifier_attributes(config, &attributes);
                    let reason = read_back_reason(
                        handler_failure,
                        format!(
                            "read-back missing primaryIdentifier attributes: {}",
                            missing_attributes.join(", ")
                        ),
                    );
                    let state = AppliedState::from_identifier(
                        resource.id.clone(),
                        wait_identifier,
                        attributes,
                    );
                    return CreateOutcome::partial_success(
                        state,
                        reason,
                        nonempty_missing_attributes(missing_attributes),
                    );
                }
                Err(error) => {
                    let state = AppliedState::from_identifier(
                        resource.id.clone(),
                        wait_identifier,
                        attributes,
                    );
                    return CreateOutcome::partial_success(
                        state,
                        read_back_reason(
                            handler_failure,
                            format!("failed to canonicalize read-back identifier: {error}"),
                        ),
                        vec!["state".to_string()],
                    );
                }
            };

        let synthesis = self
            .synthesize_read_attributes(&resource.id.resource_type, &mut attributes)
            .await;
        let synthesis = match synthesis {
            Ok(synthesis) => synthesis,
            Err(error) => {
                let state =
                    AppliedState::from_identifier(resource.id.clone(), wait_identifier, attributes);
                return CreateOutcome::partial_success(
                    state,
                    read_back_reason(
                        handler_failure,
                        format!("failed to synthesize read-back attributes: {error}"),
                    ),
                    vec!["state".to_string()],
                );
            }
        };

        match synthesis {
            SynthesisStatus::Complete => CreateOutcome::Success {
                state: AppliedState::from_identifier(
                    resource.id.clone(),
                    canonical_identifier,
                    attributes,
                ),
            },
            SynthesisStatus::Missing {
                attributes: missing_attributes,
            } => {
                let reason = read_back_reason(
                    handler_failure,
                    format!(
                        "read-back missing synthesized attributes: {}",
                        missing_attributes.join(", ")
                    ),
                );
                let state = AppliedState::from_identifier(
                    resource.id.clone(),
                    canonical_identifier,
                    attributes,
                );
                CreateOutcome::partial_success(
                    state,
                    reason,
                    nonempty_missing_attributes(missing_attributes),
                )
            }
        }
    }

    /// Drive create steps to a settled result for native callers.
    pub async fn create_resource(&self, resource: &Resource) -> ProviderResult<CreateOutcome> {
        let budget =
            self.operation_budget(resource.id.resource_type.as_str(), OperationKind::Create);
        drive_operation(
            &resource.id,
            OperationKind::Create,
            budget,
            self.create_step(resource),
            |token| self.poll_create_step(resource, token),
            tokio::time::sleep,
        )
        .await
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
        step: OperationStep<WaitOutcome, CcUpdateToken>,
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
        step: OperationStep<WaitOutcome, CcUpdateToken>,
    ) -> ProviderResult<CcDeleteStep> {
        match step {
            OperationStep::Pending {
                pending,
                poll_after_secs,
            } => Ok(OperationStep::Pending {
                pending: CcDeleteToken::DetachIgw(pending),
                poll_after_secs,
            }),
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

fn wrap_delete_phase(step: OperationStep<(), CcRequestPhase>) -> CcDeleteStep {
    match step {
        OperationStep::Done(()) => OperationStep::Done(()),
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
        AppliedIdentifier, CreateOutcome, PatchOp, PatchOpKind, ReadOutcome, UpdateOutcome,
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
                saw_get_resource: AtomicBool::new(false),
            })
        }

        fn with_not_found(wait_succeeds: bool) -> Arc<Self> {
            Arc::new(Self {
                wait_succeeds,
                read_succeeds: false,
                read_not_found: true,
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
                    "CreateResource" => MockResponse::json(
                        200,
                        r#"{"ProgressEvent":{"RequestToken":"request-token-1"}}"#,
                    ),
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
            .create_outcome_from_read(
                &resource,
                &config,
                wait_identifier.clone(),
                Ok(read_back),
                None,
            )
            .await;

        let CreateOutcome::PartialSuccess { state, diagnostic } = outcome else {
            panic!("canonicalization failure must preserve the created resource");
        };
        assert_eq!(state.identifier(), &wait_identifier);
        assert_eq!(
            state.attributes().get("bucket_name"),
            Some(&test_string("partial-bucket"))
        );
        assert!(diagnostic.reason().contains("empty canonical identifier"));
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
        assert!(config.has_tags);
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
            "Effect".to_string(),
            Value::Concrete(ConcreteValue::String("Allow".to_string().into())),
        );
        statement.insert(
            "Action".to_string(),
            Value::Concrete(ConcreteValue::String("sts:AssumeRole".to_string().into())),
        );
        statement.insert(
            "Principal".to_string(),
            Value::Concrete(ConcreteValue::Map(indexmap::IndexMap::from([(
                "Service".to_string(),
                Value::Concrete(ConcreteValue::String(
                    "vpc-flow-logs.amazonaws.com".to_string().into(),
                )),
            )]))),
        );

        Value::Concrete(ConcreteValue::Map(indexmap::IndexMap::from([
            (
                "Version".to_string(),
                Value::Concrete(ConcreteValue::String("2012-10-17".to_string().into())),
            ),
            (
                "Statement".to_string(),
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

    #[tokio::test]
    async fn create_resource_preserves_identifier_when_successful_wait_read_back_is_not_found() {
        let service = PartialCreateCloudControlService::with_not_found(true);
        let provider = provider_with_partial_create_service(service.clone()).await;

        let outcome = provider
            .create_resource(&partial_create_bucket_resource())
            .await
            .expect("create should return a partial outcome instead of losing its identifier");

        let CreateOutcome::PartialSuccess { state, diagnostic } = outcome else {
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

    #[tokio::test]
    async fn create_resource_returns_partial_success_when_failed_identifier_read_fails() {
        let service = PartialCreateCloudControlService::new(false);
        let provider = provider_with_partial_create_service(service.clone()).await;

        let outcome = provider
            .create_resource(&partial_create_bucket_resource())
            .await
            .expect("create should return a partial outcome instead of error");

        let CreateOutcome::PartialSuccess { state, diagnostic } = outcome else {
            panic!("expected partial success when post-create read fails");
        };
        assert!(
            service.saw_get_resource.load(Ordering::SeqCst),
            "provider must retry read with the identifier from failed progress"
        );
        assert_eq!(state.identifier().as_str(), "partial-bucket");
        assert!(state.attributes().is_empty());
        assert_structured_partial_read_error(diagnostic.reason(), "post-create read denied");
        assert_eq!(
            diagnostic.missing_attributes(),
            &["bucket_name".to_string()]
        );
    }

    #[tokio::test]
    async fn create_resource_preserves_identifier_when_successful_wait_read_back_fails() {
        let service = PartialCreateCloudControlService::with_wait_result(true, false);
        let provider = provider_with_partial_create_service(service.clone()).await;

        let outcome = provider
            .create_resource(&partial_create_bucket_resource())
            .await
            .expect("create should return a partial outcome instead of losing its identifier");

        let CreateOutcome::PartialSuccess { state, diagnostic } = outcome else {
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

    #[tokio::test]
    async fn create_resource_returns_success_when_failed_identifier_read_succeeds() {
        let service = PartialCreateCloudControlService::new(true);
        let provider = provider_with_partial_create_service(service.clone()).await;

        let outcome = provider
            .create_resource(&partial_create_bucket_resource())
            .await
            .expect("create should succeed after post-create read recovers");

        let CreateOutcome::Success { state } = outcome else {
            panic!("expected full success when post-create read succeeds");
        };
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
    }

    #[tokio::test]
    async fn create_iam_role_synthesizes_arn_when_cloudcontrol_omits_arn() {
        let cloudcontrol = IamRoleCreateCloudControlService::new(false);
        let sts = CallerIdentityService::new();
        let provider =
            provider_with_iam_role_create_service(cloudcontrol.clone(), sts.clone()).await;

        let outcome = provider
            .create_resource(&iam_role_resource(true))
            .await
            .expect("IAM role create should succeed");

        let CreateOutcome::Success { state } = outcome else {
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

    #[tokio::test]
    async fn create_iam_role_preserves_wait_identifier_when_synthesis_fails() {
        let cloudcontrol = IamRoleCreateCloudControlService::new(false);
        let sts = CallerIdentityService::failing();
        let provider =
            provider_with_iam_role_create_service(cloudcontrol.clone(), sts.clone()).await;

        let outcome = provider
            .create_resource(&iam_role_resource(true))
            .await
            .expect("post-create synthesis failure must become partial success");

        let CreateOutcome::PartialSuccess { state, diagnostic } = outcome else {
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

    #[tokio::test]
    async fn create_iam_role_synthesizes_arn_from_cloudcontrol_role_name() {
        let cloudcontrol = IamRoleCreateCloudControlService::new(true);
        let sts = CallerIdentityService::new();
        let provider = provider_with_iam_role_create_service(cloudcontrol, sts.clone()).await;

        let outcome = provider
            .create_resource(&iam_role_resource(false))
            .await
            .expect("IAM role create should succeed when read-back includes RoleName");

        let CreateOutcome::Success { state } = outcome else {
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

    #[tokio::test]
    async fn create_iam_role_returns_partial_success_when_post_create_read_is_not_found() {
        let cloudcontrol =
            IamRoleCreateCloudControlService::with_read_mode(IamRoleReadMode::NotFound, true);
        let sts = CallerIdentityService::new();
        let provider =
            provider_with_iam_role_create_service(cloudcontrol.clone(), sts.clone()).await;

        let outcome = provider
            .create_resource(&iam_role_resource(true))
            .await
            .expect("IAM role create should report partial success");

        let CreateOutcome::PartialSuccess { state, diagnostic } = outcome else {
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

    #[tokio::test]
    async fn create_iam_role_returns_partial_success_when_failed_create_read_back_is_not_found() {
        let cloudcontrol =
            IamRoleCreateCloudControlService::with_read_mode(IamRoleReadMode::NotFound, false);
        let sts = CallerIdentityService::new();
        let provider =
            provider_with_iam_role_create_service(cloudcontrol.clone(), sts.clone()).await;

        let outcome = provider
            .create_resource(&iam_role_resource(true))
            .await
            .expect("IAM role create should report partial success");

        let CreateOutcome::PartialSuccess { state, diagnostic } = outcome else {
            panic!("expected partial success when failed-create read-back returns not_found");
        };
        assert!(cloudcontrol.saw_create_resource.load(Ordering::SeqCst));
        assert!(cloudcontrol.saw_get_resource.load(Ordering::SeqCst));
        assert_eq!(state.identifier().as_str(), "flow-log-role");
        assert_eq!(
            state.attributes().get("role_name"),
            Some(&test_string("flow-log-role"))
        );
        assert_eq!(
            diagnostic.reason(),
            "handler failed: post-create validation failed; read-back returned not_found"
        );
        assert_eq!(diagnostic.missing_attributes(), &["state".to_string()]);
        assert!(
            !sts.saw_get_caller_identity.load(Ordering::SeqCst),
            "not_found read-back must stop before ARN synthesis"
        );
    }

    #[tokio::test]
    async fn create_iam_role_returns_partial_success_when_role_name_is_unknown() {
        let cloudcontrol = IamRoleCreateCloudControlService::new(false);
        let sts = CallerIdentityService::new();
        let provider = provider_with_iam_role_create_service(cloudcontrol, sts.clone()).await;

        let outcome = provider
            .create_resource(&iam_role_resource(false))
            .await
            .expect("IAM role create should report partial success");

        let CreateOutcome::PartialSuccess { state, diagnostic } = outcome else {
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
