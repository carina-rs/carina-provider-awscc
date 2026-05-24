//! Conversions between carina-core types and carina-provider-protocol types.
//!
//! This is a local copy of the convert module from carina-plugin-host,
//! needed because carina-plugin-host depends on wasmtime which cannot
//! compile to wasm32-wasip2.

use std::collections::HashMap;

use carina_core::provider::{
    PatchOp as CorePatchOp, PatchOpKind as CorePatchOpKind, ProviderError as CoreProviderError,
    UpdatePatch as CoreUpdatePatch,
};
use carina_core::resource::{
    ConcreteValue, DataSource as CoreDataSource, DeferredValue, Directives as CoreDirectives,
    ManagedResource as CoreResource, ResourceId as CoreResourceId, State as CoreState,
    Value as CoreValue,
};
use carina_core::schema::{
    AttributeSchema as CoreAttributeSchema, AttributeType as CoreAttributeType,
    ResourceSchema as CoreResourceSchema, StructField as CoreStructField, noop_validator,
};
use carina_provider_protocol::types::{
    AttributeSchema as ProtoAttributeSchema, AttributeType as ProtoAttributeType,
    Directives as ProtoDirectives, PatchOp as ProtoPatchOp, PatchOpKind as ProtoPatchOpKind,
    ProviderError as ProtoProviderError, ProviderErrorKind as ProtoProviderErrorKind,
    Resource as ProtoResource, ResourceId as ProtoResourceId,
    ResourceSchema as ProtoResourceSchema, State as ProtoState, StructField as ProtoStructField,
    UpdatePatch as ProtoUpdatePatch, Value as ProtoValue,
};

// -- ResourceId --

pub fn core_to_proto_resource_id(id: &CoreResourceId) -> ProtoResourceId {
    ProtoResourceId {
        provider: id.provider.clone(),
        resource_type: id.resource_type.clone(),
        name: id.name.to_string(),
    }
}

pub fn proto_to_core_resource_id(id: &ProtoResourceId) -> CoreResourceId {
    CoreResourceId::with_provider(&id.provider, &id.resource_type, &id.name, None)
}

// -- Value --

pub fn core_to_proto_value(v: &CoreValue) -> ProtoValue {
    match v {
        // `EnumIdentifier` carries identifier-shape text (parser-level
        // distinction from quoted-string literals, carina#2986). The
        // provider wire protocol has no native identifier variant, so we
        // emit it as `ProtoValue::String` — identical to the `String`
        // arm. The shape distinction is consumed at the validator entry
        // before reaching this conversion.
        CoreValue::Concrete(ConcreteValue::String(s))
        | CoreValue::Concrete(ConcreteValue::EnumIdentifier(s)) => ProtoValue::String(s.clone()),
        CoreValue::Concrete(ConcreteValue::Int(i)) => ProtoValue::Int(*i),
        CoreValue::Concrete(ConcreteValue::Float(f)) => ProtoValue::Float(*f),
        CoreValue::Concrete(ConcreteValue::Bool(b)) => ProtoValue::Bool(*b),
        // Duration: emit integer seconds; the WIT contract has no Duration variant today.
        CoreValue::Concrete(ConcreteValue::Duration(d)) => ProtoValue::Int(d.as_secs() as i64),
        CoreValue::Concrete(ConcreteValue::List(l)) => {
            ProtoValue::List(l.iter().map(core_to_proto_value).collect())
        }
        CoreValue::Concrete(ConcreteValue::StringList(items)) => ProtoValue::List(
            items
                .iter()
                .map(|s| ProtoValue::String(s.clone()))
                .collect(),
        ),
        CoreValue::Concrete(ConcreteValue::Map(m)) => ProtoValue::Map(
            m.iter()
                .map(|(k, v)| (k.clone(), core_to_proto_value(v)))
                .collect(),
        ),
        // Deferred-axis values must be resolved before reaching the provider.
        // Phase 5a of RFC #2972 makes the axis explicit; we emit redacted
        // sentinels instead of `format!("{v:?}")` so Secret plaintext never
        // leaks into a ProtoValue::String. New deferred variants break
        // compilation rather than silently leaking via Debug.
        CoreValue::Deferred(DeferredValue::Secret(_)) => {
            ProtoValue::String("<redacted-secret>".to_string())
        }
        CoreValue::Deferred(DeferredValue::ResourceRef { path }) => {
            ProtoValue::String(format!("<unresolved-ref:{}>", path.to_dot_string()))
        }
        CoreValue::Deferred(DeferredValue::BindingRef { binding }) => {
            ProtoValue::String(format!("<unresolved-binding:{binding}>"))
        }
        CoreValue::Deferred(DeferredValue::Interpolation(_)) => {
            ProtoValue::String("<unresolved-interpolation>".to_string())
        }
        CoreValue::Deferred(DeferredValue::FunctionCall { name, .. }) => {
            ProtoValue::String(format!("<unresolved-fn:{name}>"))
        }
        CoreValue::Deferred(DeferredValue::Unknown(_)) => {
            ProtoValue::String("<unknown>".to_string())
        }
    }
}

pub fn proto_to_core_value(v: &ProtoValue) -> CoreValue {
    match v {
        ProtoValue::String(s) => CoreValue::Concrete(ConcreteValue::String(s.clone())),
        ProtoValue::Int(i) => CoreValue::Concrete(ConcreteValue::Int(*i)),
        ProtoValue::Float(f) => CoreValue::Concrete(ConcreteValue::Float(*f)),
        ProtoValue::Bool(b) => CoreValue::Concrete(ConcreteValue::Bool(*b)),
        ProtoValue::List(l) => CoreValue::Concrete(ConcreteValue::List(
            l.iter().map(proto_to_core_value).collect(),
        )),
        ProtoValue::Map(m) => CoreValue::Concrete(ConcreteValue::Map(
            m.iter()
                .map(|(k, v)| (k.clone(), proto_to_core_value(v)))
                .collect(),
        )),
    }
}

pub fn core_to_proto_value_map(m: &HashMap<String, CoreValue>) -> HashMap<String, ProtoValue> {
    m.iter()
        .map(|(k, v)| (k.clone(), core_to_proto_value(v)))
        .collect()
}

pub fn proto_to_core_value_map(m: &HashMap<String, ProtoValue>) -> HashMap<String, CoreValue> {
    m.iter()
        .map(|(k, v)| (k.clone(), proto_to_core_value(v)))
        .collect()
}

// -- State --

pub fn core_to_proto_state(s: &CoreState) -> ProtoState {
    ProtoState {
        id: core_to_proto_resource_id(&s.id),
        identifier: s.identifier.clone(),
        attributes: core_to_proto_value_map(&s.attributes),
        exists: s.exists,
    }
}

pub fn proto_to_core_state(s: &ProtoState) -> CoreState {
    let id = proto_to_core_resource_id(&s.id);
    if s.exists {
        let mut state = CoreState::existing(id, proto_to_core_value_map(&s.attributes));
        if let Some(ref ident) = s.identifier {
            state = state.with_identifier(ident);
        }
        state
    } else {
        CoreState::not_found(id)
    }
}

// -- Resource --

pub fn core_to_proto_resource(r: &CoreResource) -> ProtoResource {
    ProtoResource {
        id: core_to_proto_resource_id(&r.id),
        attributes: core_to_proto_value_map(&r.resolved_attributes()),
        directives: core_to_proto_directives(&r.directives),
    }
}

// -- Directives --

pub fn core_to_proto_directives(l: &CoreDirectives) -> ProtoDirectives {
    ProtoDirectives {
        force_delete: l.force_delete,
        create_before_destroy: l.create_before_destroy,
        prevent_destroy: l.prevent_destroy,
    }
}

// -- proto_to_core_resource (reverse of core_to_proto_resource) --

pub fn proto_to_core_resource(r: &ProtoResource) -> CoreResource {
    let mut resource =
        CoreResource::with_provider(&r.id.provider, &r.id.resource_type, &r.id.name, None);
    resource.attributes = r
        .attributes
        .iter()
        .map(|(k, v)| (k.clone(), proto_to_core_value(v)))
        .collect();
    resource.directives = CoreDirectives {
        force_delete: r.directives.force_delete,
        create_before_destroy: r.directives.create_before_destroy,
        prevent_destroy: r.directives.prevent_destroy,
        depends_on: Vec::new(),
        provider_instance: None,
    };
    resource
}

/// Rebuild a [`CoreDataSource`] from the WIT `Resource` record carried
/// over the plugin boundary. `Provider::read_data_source` consumes a
/// `DataSource`, so a data-source read request maps to this typed
/// projection (carina#3181).
pub fn proto_to_core_data_source(r: &ProtoResource) -> CoreDataSource {
    let mut data_source =
        CoreDataSource::with_provider(&r.id.provider, &r.id.resource_type, &r.id.name, None);
    data_source.attributes = r
        .attributes
        .iter()
        .map(|(k, v)| (k.clone(), proto_to_core_value(v)))
        .collect();
    data_source.directives = CoreDirectives {
        force_delete: r.directives.force_delete,
        create_before_destroy: r.directives.create_before_destroy,
        prevent_destroy: r.directives.prevent_destroy,
        depends_on: Vec::new(),
        provider_instance: None,
    };
    data_source
}

// -- AttributeType --

fn proto_to_core_attribute_type(t: &ProtoAttributeType) -> CoreAttributeType {
    match t {
        ProtoAttributeType::String => CoreAttributeType::String,
        ProtoAttributeType::Int => CoreAttributeType::Int,
        ProtoAttributeType::Float => CoreAttributeType::Float,
        ProtoAttributeType::Bool => CoreAttributeType::Bool,
        ProtoAttributeType::Duration => CoreAttributeType::Duration,
        ProtoAttributeType::StringEnum {
            name,
            values,
            namespace,
            dsl_aliases,
        } => CoreAttributeType::StringEnum {
            name: name.clone(),
            values: values.clone(),
            // Lift the wire-form flat dotted prefix into the
            // structured `TypeIdentity` the core schema carries
            // post-#3222.
            identity: namespace
                .as_deref()
                .filter(|s| !s.is_empty())
                .map(|ns| carina_core::schema::string_enum_identity(name, Some(ns))),
            dsl_aliases: dsl_aliases.clone(),
        },
        ProtoAttributeType::List { inner, ordered } => CoreAttributeType::List {
            inner: Box::new(proto_to_core_attribute_type(inner)),
            ordered: *ordered,
        },
        ProtoAttributeType::Map { inner, key } => CoreAttributeType::Map {
            key: Box::new(proto_to_core_attribute_type(key)),
            value: Box::new(proto_to_core_attribute_type(inner)),
        },
        ProtoAttributeType::Struct { name, fields } => CoreAttributeType::Struct {
            name: name.clone(),
            fields: fields.iter().map(proto_to_core_struct_field).collect(),
        },
        ProtoAttributeType::Union { members } => {
            CoreAttributeType::Union(members.iter().map(proto_to_core_attribute_type).collect())
        }
        ProtoAttributeType::Custom { name, base } => CoreAttributeType::Custom {
            identity: if name.is_empty() {
                None
            } else {
                Some(carina_core::schema::TypeIdentity::from_dotted(name))
            },
            pattern: None,
            length: None,
            base: Box::new(proto_to_core_attribute_type(base)),
            validate: noop_validator(),
            // The wire form drops `to_dsl` because `fn` pointers
            // cannot cross the WASM boundary; structural state
            // normalization for plugin-provided types is registered
            // separately on the host.
            to_dsl: None,
        },
        ProtoAttributeType::CustomEnum {
            name,
            base,
            namespace,
        } => CoreAttributeType::CustomEnum {
            identity: carina_core::schema::string_enum_identity(name, Some(namespace.as_str())),
            base: Box::new(proto_to_core_attribute_type(base)),
            validate: noop_validator(),
            to_dsl: None,
        },
    }
}

fn proto_to_core_struct_field(f: &ProtoStructField) -> CoreStructField {
    CoreStructField {
        name: f.name.clone(),
        field_type: proto_to_core_attribute_type(&f.field_type),
        required: f.required,
        description: f.description.clone(),
        provider_name: f.provider_name.clone(),
        block_name: f.block_name.clone(),
        // The WIT contract does not transmit `deferred_populate`
        // (carina#3034). The annotation lives entirely in the host-
        // side schema (set by codegen output in
        // `carina-provider-awscc/src/schemas/generated/`), which is
        // loaded directly via `SchemaRegistry` rather than crossing
        // the WASM boundary.
        deferred_populate: false,
    }
}

fn _proto_to_core_attribute_schema(a: &ProtoAttributeSchema) -> CoreAttributeSchema {
    CoreAttributeSchema {
        name: a.name.clone(),
        attr_type: proto_to_core_attribute_type(&a.attr_type),
        required: a.required,
        default: a.default.as_ref().map(proto_to_core_value),
        description: a.description.clone(),
        completions: None,
        provider_name: a.provider_name.clone(),
        create_only: a.create_only,
        read_only: a.read_only,
        removable: a.removable,
        block_name: a.block_name.clone(),
        write_only: a.write_only,
        identity: a.identity,
        // See `proto_to_core_struct_field` for the rationale.
        deferred_populate: false,
    }
}

pub fn proto_to_core_schema(s: &ProtoResourceSchema) -> CoreResourceSchema {
    use carina_core::schema::SchemaKind as CoreSchemaKind;
    use carina_provider_protocol::types::SchemaKind as ProtoSchemaKind;
    let kind = match s.kind {
        ProtoSchemaKind::Managed => CoreSchemaKind::Managed,
        ProtoSchemaKind::DataSource => CoreSchemaKind::DataSource,
    };
    CoreResourceSchema {
        resource_type: s.resource_type.clone(),
        attributes: s
            .attributes
            .iter()
            .map(|(k, v)| (k.clone(), _proto_to_core_attribute_schema(v)))
            .collect(),
        description: s.description.clone(),
        validator: None,
        kind,
        name_attribute: s.name_attribute.clone(),
        force_replace: s.force_replace,
        operation_config: s.operation_config.as_ref().map(|c| {
            carina_core::schema::OperationConfig {
                delete_timeout_secs: c.delete_timeout_secs,
                delete_max_retries: c.delete_max_retries,
                create_timeout_secs: c.create_timeout_secs,
                create_max_retries: c.create_max_retries,
            }
        }),
        exclusive_required: s.exclusive_required.clone(),
        default_wait_timeout: None,
        default_wait_interval: None,
    }
}

fn core_to_proto_attribute_type(t: &CoreAttributeType) -> ProtoAttributeType {
    match t {
        CoreAttributeType::String => ProtoAttributeType::String,
        CoreAttributeType::Int => ProtoAttributeType::Int,
        CoreAttributeType::Float => ProtoAttributeType::Float,
        CoreAttributeType::Bool => ProtoAttributeType::Bool,
        // `Duration` is now a first-class proto variant (carina#3166) so
        // providers can declare Duration-typed schema attributes and the
        // host's type checker accepts DSL literals like `30min` / `1h` /
        // `15s` against them. The WIT *value* boundary is still
        // integer-seconds (see carina-plugin-host wasm_convert.rs:60-76),
        // but the *type* boundary now round-trips faithfully.
        CoreAttributeType::Duration => ProtoAttributeType::Duration,
        CoreAttributeType::StringEnum {
            name,
            values,
            identity,
            dsl_aliases,
        } => ProtoAttributeType::StringEnum {
            name: name.clone(),
            values: values.clone(),
            // Wire form still carries the flat dotted prefix; the
            // structured identity is the inverse of
            // `string_enum_identity` (post-#3222).
            namespace: identity.as_ref().and_then(|id| id.dotted_prefix()),
            dsl_aliases: dsl_aliases.clone(),
        },
        CoreAttributeType::List { inner, ordered } => ProtoAttributeType::List {
            inner: Box::new(core_to_proto_attribute_type(inner)),
            ordered: *ordered,
        },
        CoreAttributeType::Map { key, value: inner } => ProtoAttributeType::Map {
            inner: Box::new(core_to_proto_attribute_type(inner)),
            key: Box::new(core_to_proto_attribute_type(key)),
        },
        CoreAttributeType::Struct { name, fields } => ProtoAttributeType::Struct {
            name: name.clone(),
            fields: fields.iter().map(core_to_proto_struct_field).collect(),
        },
        CoreAttributeType::Custom { identity, base, .. } => ProtoAttributeType::Custom {
            // Serialize the structured identity to its dotted display
            // form for the JSON wire; the host parses it back via
            // `TypeIdentity::from_dotted`.
            name: identity
                .as_ref()
                .map(|id| id.to_string())
                .unwrap_or_default(),
            base: Box::new(core_to_proto_attribute_type(base)),
        },
        // CustomEnum carries the enum-shorthand marker as a type-level
        // fact (carina#3222); the wire form is a separate
        // `CustomEnum` variant with the dotted prefix as a flat
        // string.
        CoreAttributeType::CustomEnum { identity, base, .. } => ProtoAttributeType::CustomEnum {
            name: identity.to_string(),
            base: Box::new(core_to_proto_attribute_type(base)),
            namespace: identity.dotted_prefix().unwrap_or_default(),
        },
        CoreAttributeType::Union(members) => ProtoAttributeType::Union {
            members: members.iter().map(core_to_proto_attribute_type).collect(),
        },
    }
}

fn core_to_proto_struct_field(f: &CoreStructField) -> ProtoStructField {
    ProtoStructField {
        name: f.name.clone(),
        field_type: core_to_proto_attribute_type(&f.field_type),
        required: f.required,
        description: f.description.clone(),
        block_name: f.block_name.clone(),
        provider_name: f.provider_name.clone(),
    }
}

fn core_to_proto_attribute_schema(a: &CoreAttributeSchema) -> ProtoAttributeSchema {
    ProtoAttributeSchema {
        name: a.name.clone(),
        attr_type: core_to_proto_attribute_type(&a.attr_type),
        required: a.required,
        default: a.default.as_ref().map(core_to_proto_value),
        description: a.description.clone(),
        create_only: a.create_only,
        read_only: a.read_only,
        write_only: a.write_only,
        block_name: a.block_name.clone(),
        provider_name: a.provider_name.clone(),
        removable: a.removable,
        identity: a.identity,
    }
}

pub fn core_to_proto_schema(s: &CoreResourceSchema) -> ProtoResourceSchema {
    use carina_core::schema::SchemaKind as CoreSchemaKind;
    use carina_provider_protocol::types::SchemaKind as ProtoSchemaKind;
    let kind = match s.kind {
        CoreSchemaKind::Managed => ProtoSchemaKind::Managed,
        CoreSchemaKind::DataSource => ProtoSchemaKind::DataSource,
    };
    ProtoResourceSchema {
        resource_type: s.resource_type.clone(),
        attributes: s
            .attributes
            .iter()
            .map(|(k, v)| (k.clone(), core_to_proto_attribute_schema(v)))
            .collect(),
        description: s.description.clone(),
        kind,
        name_attribute: s.name_attribute.clone(),
        force_replace: s.force_replace,
        operation_config: s.operation_config.as_ref().map(|c| {
            carina_provider_protocol::OperationConfig {
                delete_timeout_secs: c.delete_timeout_secs,
                delete_max_retries: c.delete_max_retries,
                create_timeout_secs: c.create_timeout_secs,
                create_max_retries: c.create_max_retries,
            }
        }),
        validators: vec![],
        exclusive_required: s.exclusive_required.clone(),
    }
}

// -- UpdatePatch --

fn proto_to_core_patch_op_kind(k: ProtoPatchOpKind) -> CorePatchOpKind {
    match k {
        ProtoPatchOpKind::Add => CorePatchOpKind::Add,
        ProtoPatchOpKind::Replace => CorePatchOpKind::Replace,
        ProtoPatchOpKind::Remove => CorePatchOpKind::Remove,
    }
}

fn proto_to_core_patch_op(op: &ProtoPatchOp) -> CorePatchOp {
    CorePatchOp {
        kind: proto_to_core_patch_op_kind(op.kind),
        key: op.key.clone(),
        value: op.value.as_ref().map(proto_to_core_value),
    }
}

pub fn proto_to_core_update_patch(p: &ProtoUpdatePatch) -> CoreUpdatePatch {
    CoreUpdatePatch {
        ops: p.ops.iter().map(proto_to_core_patch_op).collect(),
    }
}

// -- ProviderError --

fn core_to_proto_provider_error_kind(e: &CoreProviderError) -> ProtoProviderErrorKind {
    match e {
        CoreProviderError::InvalidInput(_) => ProtoProviderErrorKind::InvalidInput,
        CoreProviderError::ApiError(_) => ProtoProviderErrorKind::ApiError,
        CoreProviderError::NotFound(_) => ProtoProviderErrorKind::NotFound,
        CoreProviderError::Timeout(_) => ProtoProviderErrorKind::Timeout,
        CoreProviderError::Internal(_) => ProtoProviderErrorKind::Internal,
    }
}

pub fn core_to_proto_provider_error(e: CoreProviderError) -> ProtoProviderError {
    let kind = core_to_proto_provider_error_kind(&e);
    let detail = e.detail();
    let resource_id = detail.resource_id.as_deref().map(core_to_proto_resource_id);
    let cause = detail.cause.as_ref().map(|c| c.to_string());
    let provider_name = detail.provider_name.clone();
    let message = detail.message.clone();
    let operation = detail.operation.clone();
    let status = detail.status;
    let code = detail.code.clone();
    let request_id = detail.request_id.clone();
    ProtoProviderError {
        kind,
        message,
        resource_id,
        cause,
        provider_name,
        operation,
        status,
        code,
        request_id,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn string_enum_name_preserved_through_core_to_proto_roundtrip() {
        let core_type = CoreAttributeType::StringEnum {
            name: "VersioningStatus".to_string(),
            values: vec!["Enabled".to_string(), "Suspended".to_string()],
            identity: Some(carina_core::schema::string_enum_identity(
                "VersioningStatus",
                Some("awscc.s3.Bucket"),
            )),
            dsl_aliases: vec![],
        };

        let proto_type = core_to_proto_attribute_type(&core_type);

        // Proto should preserve the name
        match &proto_type {
            ProtoAttributeType::StringEnum {
                name,
                values,
                namespace,
                dsl_aliases,
            } => {
                assert_eq!(name, "VersioningStatus");
                assert_eq!(values.len(), 2);
                assert_eq!(namespace.as_deref(), Some("awscc.s3.Bucket"));
                assert!(dsl_aliases.is_empty());
            }
            _ => panic!("Expected StringEnum"),
        }

        // Round-trip back to core should preserve the name
        let roundtripped = proto_to_core_attribute_type(&proto_type);
        match &roundtripped {
            CoreAttributeType::StringEnum { name, values, .. } => {
                assert_eq!(name, "VersioningStatus");
                assert_eq!(values.len(), 2);
            }
            _ => panic!("Expected StringEnum"),
        }
    }

    /// Regression for awscc#199: when codegen populates `dsl_aliases` on
    /// a `StringEnum`, those alias pairs MUST cross the WASM boundary
    /// intact (proto-side carries them as data, not a fn pointer) so the
    /// host-side validator accepts the snake_case DSL spelling.
    #[test]
    fn string_enum_dsl_aliases_round_trip_through_proto() {
        let aliases = vec![
            (
                "BucketOwnerEnforced".to_string(),
                "bucket_owner_enforced".to_string(),
            ),
            (
                "BucketOwnerPreferred".to_string(),
                "bucket_owner_preferred".to_string(),
            ),
            ("ObjectWriter".to_string(), "object_writer".to_string()),
        ];
        let core_type = CoreAttributeType::StringEnum {
            name: "ObjectOwnership".to_string(),
            values: vec![
                "ObjectWriter".to_string(),
                "BucketOwnerPreferred".to_string(),
                "BucketOwnerEnforced".to_string(),
            ],
            identity: Some(carina_core::schema::string_enum_identity(
                "ObjectOwnership",
                Some("awscc.s3.Bucket"),
            )),
            dsl_aliases: aliases.clone(),
        };

        let proto_type = core_to_proto_attribute_type(&core_type);
        match &proto_type {
            ProtoAttributeType::StringEnum { dsl_aliases: a, .. } => {
                assert_eq!(a, &aliases, "proto must carry the alias data verbatim");
            }
            _ => panic!("Expected StringEnum"),
        }

        let roundtripped = proto_to_core_attribute_type(&proto_type);
        match &roundtripped {
            CoreAttributeType::StringEnum { dsl_aliases: a, .. } => {
                assert_eq!(
                    a, &aliases,
                    "alias data must survive proto -> core round-trip"
                );
            }
            _ => panic!("Expected StringEnum"),
        }
    }

    #[test]
    fn core_to_proto_schema_initializes_empty_validators() {
        let schema = CoreResourceSchema::new("s3.Bucket");
        let proto = core_to_proto_schema(&schema);
        assert!(proto.validators.is_empty());
    }
}
