//! High-level resource operations (read, create, update, delete).
//!
//! This module implements the main resource lifecycle operations that bridge
//! between DSL resources and the Cloud Control API. It handles attribute mapping,
//! tags, special cases, and default values.

use std::collections::HashMap;

use carina_core::provider::{CreateOutcome, ProviderError, ProviderResult, UpdatePatch};
use carina_core::resource::{ConcreteValue, Directives, Resource, ResourceId, State, Value};
use carina_core::schema::{AttributeSchema, AttributeType, Schema, Shape};
use indexmap::IndexMap;
use serde_json::json;

use super::conversion::{aws_value_to_dsl_with_defs, dsl_value_to_aws_with_defs};
use super::update::build_update_patches;
use super::{AwsccProvider, get_schema_config};
use crate::provider::cloudcontrol::WaitOutcome;

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
        self.synthesize_read_attributes(resource_type, &mut attributes)
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
                    Ok(state) => return Ok(CreateOutcome::Success { state }),
                    Err(read_err) => {
                        let state = State::existing(resource.id.clone(), HashMap::new())
                            .with_identifier(identifier);
                        let missing_attributes = config.schema.attributes.keys().cloned().collect();
                        let reason = format!(
                            "post-create read failed after handler failure: {}; read error: {}",
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

        let mut state = self
            .read_resource(
                &resource.id.resource_type,
                resource.id.name_str(),
                Some(&identifier),
            )
            .await?;

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

        Ok(CreateOutcome::Success { state })
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
    ) -> ProviderResult<State> {
        let config = get_schema_config(&id.resource_type).ok_or_else(|| {
            ProviderError::internal(format!("Unknown resource type: {}", id.resource_type))
                .for_resource(id.clone())
        })?;

        let patch_ops = build_update_patches(config, &id.resource_type, patch);

        self.cc_update_resource(config.aws_type_name, identifier, patch_ops)
            .await
            .map_err(|e| e.for_resource(id.clone()))?;

        let mut state = self
            .read_resource(&id.resource_type, id.name_str(), Some(identifier))
            .await?;

        // Reconstruct the post-update desired view (current state + the
        // patch we just applied). This is the source of values to carry
        // forward for attributes CloudControl's read does not return —
        // same logic as `create_resource` but built without a `to:
        // Resource` (which Level 3 deliberately does not pass through).
        let desired = post_update_attributes(from, patch);
        for dsl_name in config.schema.attributes.keys() {
            if !state.attributes.contains_key(dsl_name)
                && let Some(value) = desired.get(dsl_name)
            {
                state.attributes.insert(dsl_name.clone(), value.clone());
            }
        }

        Ok(state)
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
    use carina_core::provider::CreateOutcome;
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
                            r#"{"TypeName":"AWS::S3::Bucket","ResourceDescription":{"Identifier":"partial-bucket","Properties":"{\"BucketName\":\"partial-bucket\"}"}}"#,
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

    fn partial_create_bucket_resource() -> Resource {
        Resource::with_provider("awscc", "s3.Bucket", "partial_bucket", None).with_attribute(
            "bucket_name",
            Value::Concrete(ConcreteValue::String("partial-bucket".to_string())),
        )
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
            "post-create read failed after handler failure: post-create read denied; read error: Failed to get resource: AccessDeniedException: read denied"
        );
        assert!(
            diagnostic
                .missing_attributes()
                .contains(&"bucket_name".to_string())
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
