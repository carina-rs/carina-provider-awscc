//! High-level resource operations (read, create, update, delete).
//!
//! This module implements the main resource lifecycle operations that bridge
//! between DSL resources and the Cloud Control API. It handles attribute mapping,
//! tags, special cases, and default values.

use std::collections::HashMap;

use carina_core::provider::{
    CreateOutcome, ProviderError, ProviderResult, UpdateOutcome, UpdatePatch,
};
use carina_core::resource::{ConcreteValue, Directives, Resource, ResourceId, State, Value};
use carina_core::schema::{AttributeSchema, AttributeType, Schema, Shape};
use indexmap::IndexMap;
use serde_json::json;

use super::conversion::{aws_value_to_dsl_with_defs, dsl_value_to_aws_with_defs};
use super::update::build_update_patches;
use super::{AwsccProvider, get_schema_config};
use crate::provider::arn_synthesis::SynthesisStatus;
use crate::provider::cloudcontrol::WaitOutcome;
use crate::schemas::config::AwsccSchemaConfig;

impl AwsccProvider {
    /// Read a resource using its configuration
    pub async fn read_resource(
        &self,
        resource_type: &str,
        name: &str,
        identifier: Option<&str>,
    ) -> ProviderResult<State> {
        let id = ResourceId::with_provider("awscc", resource_type, name, None);

        let config = get_schema_config(resource_type).ok_or_else(|| {
            ProviderError::internal(format!("Unknown resource type: {}", resource_type))
                .for_resource(id.clone())
        })?;

        let identifier = match identifier {
            Some(id) => id,
            None => return Ok(State::not_found(id)),
        };

        let props = match self
            .cc_get_resource(config.aws_type_name, identifier)
            .await?
        {
            Some(props) => props,
            None => return Ok(State::not_found(id)),
        };

        let mut attributes = map_aws_props_to_attributes(
            &props,
            &config.schema.attributes,
            resource_type,
            &config.schema.defs,
        );

        // Handle tags
        if config.has_tags
            && let Some(tags_array) = props.get("Tags").and_then(|v| v.as_array())
        {
            let tags_map = self.parse_tags(tags_array);
            if !tags_map.is_empty() {
                attributes.insert(
                    "tags".to_string(),
                    Value::Concrete(ConcreteValue::Map(tags_map)),
                );
            }
        }

        // Handle special cases
        self.read_special_attributes(resource_type, &props, &mut attributes);

        // Synthesize attributes the Cloud Control read does not return
        // (e.g. cloudfront.Distribution.arn). Reads STS once, then caches.
        let _ = self
            .synthesize_read_attributes(resource_type, &mut attributes)
            .await?;

        Ok(State::existing(id, attributes).with_identifier(identifier))
    }

    /// Create a resource using its configuration
    pub async fn create_resource(&self, resource: &Resource) -> ProviderResult<CreateOutcome> {
        let config = get_schema_config(&resource.id.resource_type).ok_or_else(|| {
            ProviderError::internal(format!(
                "Unknown resource type: {}",
                resource.id.resource_type
            ))
            .for_resource(resource.id.clone())
        })?;

        let mut desired_state = serde_json::Map::new();

        // Map DSL attributes to AWS attributes using provider_name
        for (dsl_name, attr_schema) in &config.schema.attributes {
            // Skip tags - handled separately below
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

        // Handle special cases for create
        self.create_special_attributes(resource, &mut desired_state);

        // Handle tags
        if config.has_tags {
            let tags = self.build_tags(resource.get_attr("tags"));
            if !tags.is_empty() {
                desired_state.insert("Tags".to_string(), json!(tags));
            }
        }

        // Set default values
        self.set_default_values(&resource.id.resource_type, &mut desired_state);

        let outcome = self
            .cc_create_resource(
                config.aws_type_name,
                serde_json::Value::Object(desired_state),
                config.schema.operation_config.as_ref(),
            )
            .await
            .map_err(|e| e.for_resource(resource.id.clone()))?;

        let identifier = match outcome {
            WaitOutcome::Success { identifier } => identifier,
            WaitOutcome::PartialOrFailed {
                identifier,
                status_message,
            } => {
                match self
                    .read_resource(
                        &resource.id.resource_type,
                        resource.id.name_str(),
                        Some(&identifier),
                    )
                    .await
                {
                    Ok(state) => {
                        if !state.exists {
                            return Ok(create_read_not_found_outcome(
                                state,
                                format!(
                                    "handler failed: {}; read-back returned not_found",
                                    status_message
                                ),
                            ));
                        }
                        let mut state = merge_desired_attributes(state, resource, config);
                        let Some(canonical_identifier) =
                            canonicalize_identifier_from_read(config, &state)
                        else {
                            let missing_attributes =
                                missing_primary_identifier_attributes(config, &state);
                            let reason = format!(
                                "handler failed: {}; read-back missing primaryIdentifier attributes: {}",
                                status_message,
                                missing_attributes.join(", ")
                            );
                            return Ok(CreateOutcome::partial_success(
                                state,
                                reason,
                                missing_attributes,
                            ));
                        };
                        state.identifier = Some(canonical_identifier);
                        if let Some(missing_attributes) = self
                            .synthesize_read_attributes(
                                &resource.id.resource_type,
                                &mut state.attributes,
                            )
                            .await?
                            .missing_attributes()
                        {
                            let reason = format!(
                                "handler failed: {}; read-back missing synthesized attributes: {}",
                                status_message,
                                missing_attributes.join(", ")
                            );
                            return Ok(CreateOutcome::partial_success(
                                state,
                                reason,
                                missing_attributes,
                            ));
                        }
                        return Ok(CreateOutcome::Success { state });
                    }
                    Err(read_err) => {
                        let state = State::existing(resource.id.clone(), HashMap::new())
                            .with_identifier(identifier);
                        let missing_attributes = config
                            .schema
                            .attributes
                            .keys()
                            .filter(|name| resource.get_attr(name.as_str()).is_some())
                            .cloned()
                            .collect();
                        let reason = format!(
                            "handler failed: {}; read error: {}",
                            status_message,
                            read_err.message(),
                        );
                        return Ok(CreateOutcome::partial_success(
                            state,
                            reason,
                            missing_attributes,
                        ));
                    }
                }
            }
        };

        let state = self
            .read_resource(
                &resource.id.resource_type,
                resource.id.name_str(),
                Some(&identifier),
            )
            .await?;

        if !state.exists {
            return Ok(create_read_not_found_outcome(
                state,
                "read-back returned not_found",
            ));
        }
        let mut state = merge_desired_attributes(state, resource, config);
        let Some(canonical_identifier) = canonicalize_identifier_from_read(config, &state) else {
            let missing_attributes = missing_primary_identifier_attributes(config, &state);
            return Ok(CreateOutcome::partial_success(
                state,
                format!(
                    "read-back missing primaryIdentifier attributes: {}",
                    missing_attributes.join(", ")
                ),
                missing_attributes,
            ));
        };
        state.identifier = Some(canonical_identifier);

        match self
            .synthesize_read_attributes(&resource.id.resource_type, &mut state.attributes)
            .await?
        {
            SynthesisStatus::Complete => Ok(CreateOutcome::Success { state }),
            SynthesisStatus::Missing { attributes } => Ok(CreateOutcome::partial_success(
                state,
                format!(
                    "read-back missing synthesized attributes: {}",
                    attributes.join(", ")
                ),
                attributes,
            )),
        }
    }

    /// Update a resource by applying an [`UpdatePatch`] to its
    /// CloudControl-side state.
    ///
    /// The patch is the sole source of truth for the update payload —
    /// fields the user has never specified are not in `patch.ops` and
    /// therefore generate no JSON Patch op, leaving CloudControl's
    /// other state alone (this is the actual fix for
    /// `carina-rs/carina#2559`).
    ///
    /// `from` is the current provider-side state; it is used only to
    /// reconstruct the post-update desired-state view that's carried
    /// forward into the returned `State` for attributes the API does
    /// not return in its read response. It MUST NOT be used to derive
    /// additional fields to write back.
    pub async fn update_resource(
        &self,
        id: ResourceId,
        identifier: &str,
        from: &State,
        patch: &UpdatePatch,
    ) -> ProviderResult<UpdateOutcome> {
        let config = get_schema_config(&id.resource_type).ok_or_else(|| {
            ProviderError::internal(format!("Unknown resource type: {}", id.resource_type))
                .for_resource(id.clone())
        })?;

        let patch_ops = build_update_patches(config, &id.resource_type, patch);

        let outcome = self
            .cc_update_resource(config.aws_type_name, identifier, patch_ops)
            .await
            .map_err(|e| e.for_resource(id.clone()))?;

        // Reconstruct the post-update desired view (current state + the
        // patch we just applied). This is the source of values to carry
        // forward for attributes CloudControl's read does not return —
        // same logic as `create_resource` but built without a `to:
        // Resource` (which Level 3 deliberately does not pass through).
        let desired = post_update_attributes(from, patch);

        match outcome {
            WaitOutcome::Success { identifier } => {
                let state = self
                    .read_resource(&id.resource_type, id.name_str(), Some(&identifier))
                    .await?;
                let state = merge_update_desired_attributes(state, &desired, config);
                Ok(UpdateOutcome::Success { state })
            }
            WaitOutcome::PartialOrFailed {
                identifier,
                status_message,
            } => {
                match self
                    .read_resource(&id.resource_type, id.name_str(), Some(&identifier))
                    .await
                {
                    Ok(state) => {
                        let state = merge_update_desired_attributes(state, &desired, config);
                        Ok(UpdateOutcome::Success { state })
                    }
                    Err(read_err) => {
                        let missing_attributes = patch
                            .ops
                            .iter()
                            .filter(|op| config.schema.attributes.contains_key(&op.key))
                            .map(|op| op.key.clone())
                            .collect::<Vec<_>>();
                        let mut state_attributes = desired;
                        for attr in &missing_attributes {
                            state_attributes.remove(attr);
                        }
                        let state = State::existing(id.clone(), state_attributes)
                            .with_identifier(identifier);
                        let reason = format!(
                            "handler failed: {}; read error: {}",
                            status_message,
                            read_err.message(),
                        );
                        Ok(UpdateOutcome::partial_success(
                            state,
                            reason,
                            missing_attributes,
                        ))
                    }
                }
            }
        }
    }

    /// Delete a resource
    pub async fn delete_resource(
        &self,
        id: &ResourceId,
        identifier: &str,
        directives: &Directives,
    ) -> ProviderResult<()> {
        let config = get_schema_config(&id.resource_type).ok_or_else(|| {
            ProviderError::internal(format!("Unknown resource type: {}", id.resource_type))
                .for_resource(id.clone())
        })?;

        // Handle special pre-delete operations
        self.pre_delete_operations(id, config, identifier).await?;

        // Handle force_delete for S3 buckets: empty the bucket before deletion
        if directives.force_delete && id.resource_type == "s3.Bucket" {
            self.empty_s3_bucket(identifier).await.map_err(|e| {
                ProviderError::api_error("Failed to empty S3 bucket before deletion")
                    .with_cause(e)
                    .for_resource(id.clone())
            })?;
        }

        self.cc_delete_resource(
            config.aws_type_name,
            identifier,
            config.schema.operation_config.as_ref(),
        )
        .await
        .map_err(|e| e.for_resource(id.clone()))
    }
}

fn canonicalize_identifier_from_read(
    config: &AwsccSchemaConfig,
    read_state: &State,
) -> Option<String> {
    let mut segments = Vec::with_capacity(config.primary_identifier.len());
    for attr in config.primary_identifier {
        segments.push(primary_identifier_segment(
            read_state.attributes.get(attr.dsl_name)?,
        )?);
    }
    if segments.is_empty() {
        None
    } else {
        Some(segments.join("|"))
    }
}

fn missing_primary_identifier_attributes(
    config: &AwsccSchemaConfig,
    read_state: &State,
) -> Vec<String> {
    config
        .primary_identifier
        .iter()
        .filter(|attr| {
            read_state
                .attributes
                .get(attr.dsl_name)
                .and_then(primary_identifier_segment)
                .is_none()
        })
        .map(|attr| attr.provider_name.to_string())
        .collect()
}

fn primary_identifier_segment(value: &Value) -> Option<String> {
    match value {
        Value::Concrete(ConcreteValue::String(value)) if !value.is_empty() => Some(value.clone()),
        Value::Concrete(ConcreteValue::EnumIdentifier(value)) => Some(value.to_string()),
        Value::Concrete(ConcreteValue::CanonicalEnum(value)) => Some(value.to_string()),
        Value::Concrete(ConcreteValue::Int(value)) => Some(value.to_string()),
        Value::Concrete(ConcreteValue::Float(value)) => Some(value.to_string()),
        Value::Concrete(ConcreteValue::Bool(value)) => Some(value.to_string()),
        _ => None,
    }
}

fn merge_desired_attributes(
    mut state: State,
    resource: &Resource,
    config: &crate::schemas::config::AwsccSchemaConfig,
) -> State {
    // Preserve desired attributes not returned by CloudControl API.
    // CloudControl doesn't always return all properties in GetResource responses
    // (create-only properties, and some normal properties like `description`).
    // Carry them forward from the desired state.
    for dsl_name in config.schema.attributes.keys() {
        if !state.attributes.contains_key(dsl_name)
            && let Some(value) = resource.get_attr(dsl_name.as_str())
        {
            state.attributes.insert(dsl_name.to_string(), value.clone());
        }
    }
    state
}

fn merge_update_desired_attributes(
    mut state: State,
    desired: &HashMap<String, Value>,
    config: &crate::schemas::config::AwsccSchemaConfig,
) -> State {
    for dsl_name in config.schema.attributes.keys() {
        if !state.attributes.contains_key(dsl_name)
            && let Some(value) = desired.get(dsl_name)
        {
            state.attributes.insert(dsl_name.clone(), value.clone());
        }
    }
    state
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
            None if !attr_schema.required && !attr_schema.write_only => {
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

fn create_read_not_found_outcome(state: State, diagnostic: impl Into<String>) -> CreateOutcome {
    CreateOutcome::partial_success(state, diagnostic.into(), vec!["state".to_string()])
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
    use carina_core::provider::{CreateOutcome, PatchOp, PatchOpKind, UpdateOutcome};
    use carina_core::schema::AttributeType;
    use serde_json::json;
    use std::future::Future;
    use std::pin::Pin;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicBool, Ordering};
    use winterbaume_core::{MockAws, MockRequest, MockResponse, MockService};

    #[derive(Debug)]
    struct PartialCreateCloudControlService {
        read_succeeds: bool,
        saw_get_resource: AtomicBool,
    }

    impl PartialCreateCloudControlService {
        fn new(read_succeeds: bool) -> Arc<Self> {
            Arc::new(Self {
                read_succeeds,
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
                    "GetResourceRequestStatus" => MockResponse::json(
                        200,
                        r#"{"ProgressEvent":{"RequestToken":"request-token-1","OperationStatus":"FAILED","Identifier":"partial-bucket","StatusMessage":"post-create read denied"}}"#,
                    ),
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
        saw_get_caller_identity: AtomicBool,
    }

    impl CallerIdentityService {
        fn new() -> Arc<Self> {
            Arc::new(Self {
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
        saw_update_resource: AtomicBool,
        saw_get_resource: AtomicBool,
    }

    impl PartialUpdateCloudControlService {
        fn new(wait_succeeds: bool, read_succeeds: bool) -> Arc<Self> {
            Arc::new(Self {
                wait_succeeds,
                read_succeeds,
                saw_update_resource: AtomicBool::new(false),
                saw_get_resource: AtomicBool::new(false),
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
            ResourceId::with_provider("awscc", "s3.Bucket", "partial_bucket", None),
            HashMap::from([(
                "bucket_name".to_string(),
                Value::Concrete(ConcreteValue::String("partial-bucket".to_string())),
            )]),
        )
        .with_identifier("partial-bucket")
    }

    fn partial_update_bucket_patch() -> UpdatePatch {
        let mut tags = indexmap::IndexMap::new();
        tags.insert(
            "env".to_string(),
            Value::Concrete(ConcreteValue::String("prod".to_string())),
        );
        UpdatePatch {
            ops: vec![PatchOp {
                kind: PatchOpKind::Replace,
                key: "tags".to_string(),
                value: Some(Value::Concrete(ConcreteValue::Map(tags))),
            }],
        }
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
                from.identifier.as_deref().expect("identifier"),
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
        assert!(state.exists);
        assert_eq!(state.identifier.as_deref(), Some("partial-bucket"));
        assert!(state.attributes.contains_key("tags"));
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
                from.identifier.as_deref().expect("identifier"),
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
        assert!(state.exists);
        assert_eq!(state.identifier.as_deref(), Some("partial-bucket"));
        assert_eq!(
            state.attributes.get("bucket_name"),
            Some(&Value::Concrete(ConcreteValue::String(
                "partial-bucket".to_string()
            )))
        );
        assert!(
            !state.attributes.contains_key("tags"),
            "missing authored attributes must be absent so the partial-read marker can materialize them as Unknown"
        );
        assert_eq!(
            diagnostic.reason(),
            "handler failed: post-update read denied; read error: Failed to get resource: AccessDeniedException: read denied"
        );
        assert_eq!(diagnostic.missing_attributes(), &["tags".to_string()]);
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
                from.identifier.as_deref().expect("identifier"),
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
        assert!(state.exists);
        assert_eq!(state.identifier.as_deref(), Some("partial-bucket"));
        assert!(state.attributes.contains_key("tags"));
    }

    fn partial_create_bucket_resource() -> Resource {
        Resource::with_provider("awscc", "s3.Bucket", "partial_bucket", None).with_attribute(
            "bucket_name",
            Value::Concrete(ConcreteValue::String("partial-bucket".to_string())),
        )
    }

    fn assume_role_policy_document() -> Value {
        let mut statement = indexmap::IndexMap::new();
        statement.insert(
            "Effect".to_string(),
            Value::Concrete(ConcreteValue::String("Allow".to_string())),
        );
        statement.insert(
            "Action".to_string(),
            Value::Concrete(ConcreteValue::String("sts:AssumeRole".to_string())),
        );
        statement.insert(
            "Principal".to_string(),
            Value::Concrete(ConcreteValue::Map(indexmap::IndexMap::from([(
                "Service".to_string(),
                Value::Concrete(ConcreteValue::String(
                    "vpc-flow-logs.amazonaws.com".to_string(),
                )),
            )]))),
        );

        Value::Concrete(ConcreteValue::Map(indexmap::IndexMap::from([
            (
                "Version".to_string(),
                Value::Concrete(ConcreteValue::String("2012-10-17".to_string())),
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
                Value::Concrete(ConcreteValue::String("flow-log-role".to_string())),
            )
        } else {
            resource
        }
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
        assert!(state.exists);
        assert_eq!(state.identifier.as_deref(), Some("partial-bucket"));
        assert!(state.attributes.is_empty());
        assert_eq!(
            diagnostic.reason(),
            "handler failed: post-create read denied; read error: Failed to get resource: AccessDeniedException: read denied"
        );
        assert_eq!(
            diagnostic.missing_attributes(),
            &["bucket_name".to_string()]
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
        assert!(state.exists);
        assert_eq!(state.identifier.as_deref(), Some("partial-bucket"));
        assert_eq!(
            state.attributes.get("bucket_name"),
            Some(&Value::Concrete(ConcreteValue::String(
                "partial-bucket".to_string()
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
            state.attributes.get("arn"),
            Some(&Value::Concrete(ConcreteValue::String(
                "arn:aws-us-gov:iam::123456789012:role/service-role/flow-log-role".to_string()
            ))),
            "synthesis must derive the partition from the provider region instead of hardcoding aws"
        );
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
            state.attributes.get("arn"),
            Some(&Value::Concrete(ConcreteValue::String(
                "arn:aws-us-gov:iam::123456789012:role/service-role/flow-log-role".to_string()
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
        assert!(!state.exists);
        assert_eq!(state.identifier.as_deref(), None);
        assert!(state.attributes.is_empty());
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
        assert!(!state.exists);
        assert_eq!(state.identifier.as_deref(), None);
        assert!(state.attributes.is_empty());
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
        assert!(state.exists);
        assert_eq!(state.identifier.as_deref(), Some("flow-log-role"));
        assert!(
            !state.attributes.contains_key("arn"),
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
            s.required = required;
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
                    "arn:aws:iam::aws:policy/ReadOnlyAccess".to_string()
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
