//! Plan-time normalization of enum identifiers and state hydration.
//!
//! This module contains standalone functions used by `ProviderNormalizer` to resolve
//! enum identifiers in resources and restore unreturned attributes from saved state.

use indexmap::IndexMap;
use std::collections::HashMap;

use carina_core::resource::{ConcreteValue, Resource, ResourceId, State, Value};
use carina_core::schema::{
    AttributeType, RawShape, ResourceSchema, Shape, ShapeWalkBudget, StructField,
};

/// Resolve enum identifiers in resources to their fully-qualified DSL format.
///
/// For each awscc resource, looks up the schema and resolves bare identifiers
/// (e.g., `advanced`) or TypeName.value identifiers (e.g., `Tier.advanced`)
/// into fully-qualified namespaced strings (e.g., `awscc.ec2.Ipam.Tier.advanced`).
pub fn resolve_enum_identifiers_impl(resources: &mut [Resource]) {
    for resource in resources.iter_mut() {
        // Only handle awscc resources
        if resource.id.provider != "awscc" {
            continue;
        }

        // Find the matching schema config via cached O(1) lookup
        let config = match crate::schemas::generated::get_config_by_type(&resource.id.resource_type)
        {
            Some(c) => c,
            None => continue,
        };

        // Resolve enum attributes
        let mut resolved_attrs = HashMap::new();
        for (key, value) in &resource.attributes {
            if let Some(attr_schema) = config.schema.attributes.get(key.as_str())
                && let Some(parts) = attr_schema.attr_type.enum_parts()
            {
                if let Some(resolved) = carina_core::utils::resolve_enum_value(value, &parts) {
                    resolved_attrs.insert(key.clone(), resolved);
                } else {
                    resolved_attrs.insert(key.clone(), value.clone());
                }
                continue;
            }

            // Handle struct fields containing schema-level string enums.
            if let Some(attr_schema) = config.schema.attributes.get(key.as_str()) {
                let struct_fields = struct_fields_for(&attr_schema.attr_type, &config.schema);

                if let Some(fields) = struct_fields {
                    let resolved = resolve_struct_enum_values(value, fields, &config.schema);
                    resolved_attrs.insert(key.clone(), resolved);
                    continue;
                }
            }

            resolved_attrs.insert(key.clone(), value.clone());
        }
        for (key, value) in resolved_attrs {
            resource.set_attr(key, value);
        }
    }
}

/// Extract the struct-field slice an attribute type advertises, looking
/// through a single `List` wrapper. Returns `None` for any other shape.
/// `defs` is used to peel `Ref` chains so cyclic-schema resources resolve
/// correctly (carina#3340).
fn struct_fields_for<'a>(
    attr_type: &'a AttributeType,
    schema: &'a ResourceSchema,
) -> Option<&'a [StructField]> {
    match schema.shape_of(attr_type) {
        Shape::List { inner, .. } => {
            schema.struct_fields_with_budget(inner, &mut ShapeWalkBudget::new(256))
        }
        Shape::Struct { .. } => {
            schema.struct_fields_with_budget(attr_type, &mut ShapeWalkBudget::new(256))
        }
        _ => None,
    }
}

/// Resolve enum identifiers within struct field values.
/// Recurses into List and Map values, resolving bare/shorthand enum values
/// for struct fields that have StringEnum type with namespace.
///
/// `defs` is the owning resource's `ResourceSchema::defs` map and must be
/// threaded through every recursion. The cyclic-schema Ref codegen
/// (awscc#281, carina#3340) emits `AttributeType::Ref(name)` inside struct
/// fields (e.g. WebACL's recursive `Statement` graph, CloudFront's
/// `forwarded_values`); peeling those `Ref`s requires the same `defs` map
/// the top-level attribute carries. Passing an empty definition map here panics in
/// `AttributeType::shape` on the first such field (awscc#286 / awscc#288).
fn resolve_struct_enum_values(
    value: &Value,
    fields: &[StructField],
    schema: &ResourceSchema,
) -> Value {
    match value {
        Value::Concrete(ConcreteValue::List(items)) => {
            let resolved_items: Vec<Value> = items
                .iter()
                .map(|item| resolve_struct_enum_values(item, fields, schema))
                .collect();
            Value::Concrete(ConcreteValue::List(resolved_items))
        }
        Value::Concrete(ConcreteValue::Map(map)) => {
            let mut resolved_map = IndexMap::new();
            for (field_key, field_value) in map {
                if let Some(field) = fields.iter().find(|f| f.name == *field_key) {
                    // Direct enum field (String value)
                    if let Some(parts) = field.field_type.enum_parts()
                        && let Some(resolved) =
                            carina_core::utils::resolve_enum_value(field_value, &parts)
                    {
                        resolved_map.insert(field_key.clone(), resolved);
                        continue;
                    }
                    // List(StringEnum): resolve each element.
                    if let Shape::List { inner, .. } = schema.shape_of(&field.field_type)
                        && let Some(parts) = inner.enum_parts()
                        && let Value::Concrete(ConcreteValue::List(items)) = field_value
                    {
                        let resolved_items: Vec<Value> = items
                            .iter()
                            .map(|item| {
                                carina_core::utils::resolve_enum_value(item, &parts)
                                    .unwrap_or_else(|| item.clone())
                            })
                            .collect();
                        resolved_map.insert(
                            field_key.clone(),
                            Value::Concrete(ConcreteValue::List(resolved_items)),
                        );
                        continue;
                    }
                    // Recurse into nested Struct or List(Struct) fields
                    let nested_fields = struct_fields_for(&field.field_type, schema);
                    if let Some(nested) = nested_fields {
                        resolved_map.insert(
                            field_key.clone(),
                            resolve_struct_enum_values(field_value, nested, schema),
                        );
                        continue;
                    }
                }
                resolved_map.insert(field_key.clone(), field_value.clone());
            }
            Value::Concrete(ConcreteValue::Map(resolved_map))
        }
        _ => value.clone(),
    }
}

/// Normalize enum values in current states to their fully-qualified DSL format.
///
/// State files store raw AWS values (e.g., `"ap-northeast-1a"`, `"default"`).
/// After `normalize_desired()` converts desired values to DSL enum format
/// (e.g., `"awscc.ec2.Subnet.AvailabilityZone.ap_northeast_1a"`), the differ
/// would see a false diff. This function normalizes state values the same way
/// so that both sides use the same representation.
pub fn normalize_state_enums_impl(current_states: &mut HashMap<ResourceId, State>) {
    for (resource_id, state) in current_states.iter_mut() {
        if !state.exists || resource_id.provider != "awscc" {
            continue;
        }

        let config = match crate::schemas::generated::get_config_by_type(&resource_id.resource_type)
        {
            Some(c) => c,
            None => continue,
        };

        let mut resolved_attrs = HashMap::new();
        for (key, value) in &state.attributes {
            if let Some(attr_schema) = config.schema.attributes.get(key.as_str())
                && let Some(transformed) = apply_enum_dsl_transform(value, &attr_schema.attr_type)
            {
                resolved_attrs.insert(key.clone(), transformed);
                continue;
            }

            if let Some(attr_schema) = config.schema.attributes.get(key.as_str())
                && let Some(parts) = attr_schema.attr_type.enum_parts()
            {
                // AWSCC state normalization: only resolve bare values (no dots)
                if let Some(resolved) = carina_core::utils::resolve_enum_value(value, &parts) {
                    resolved_attrs.insert(key.clone(), resolved);
                } else {
                    resolved_attrs.insert(key.clone(), value.clone());
                }
                continue;
            }

            // Handle struct fields containing schema-level string enums.
            if let Some(attr_schema) = config.schema.attributes.get(key.as_str()) {
                let struct_fields = struct_fields_for(&attr_schema.attr_type, &config.schema);

                if let Some(fields) = struct_fields {
                    let resolved = resolve_struct_enum_values(value, fields, &config.schema);
                    resolved_attrs.insert(key.clone(), resolved);
                    continue;
                }
            }

            resolved_attrs.insert(key.clone(), value.clone());
        }
        state.attributes = resolved_attrs;
    }
}

fn apply_enum_dsl_transform(value: &Value, attr_type: &AttributeType) -> Option<Value> {
    let RawShape::Enum {
        to_dsl: Some(transform),
        ..
    } = attr_type.raw_shape()
    else {
        return None;
    };
    let Value::Concrete(ConcreteValue::String(s)) = value else {
        return None;
    };
    let transformed = transform.apply(s);
    (transformed != *s).then(|| Value::Concrete(ConcreteValue::String(transformed)))
}

/// Canonicalize attributes of every awscc state whose declared schema
/// type is `Union[String, list(String)]` (the IAM-style
/// `string_or_list_of_strings` shape) into `Value::StringList`.
///
/// AWS server-side normalizes single-element list condition values back
/// to scalars on read (e.g. `["x"]` is stored and returned as `"x"`).
/// Pre-canonicalizing the actual-side at the provider boundary lets
/// every downstream consumer (state writeback, plan display, the
/// differ) see the same shape as the canonicalized desired side.
/// See carina-rs/carina#2481, sub-issue 5.
pub fn canonicalize_string_or_list_states_impl(current_states: &mut HashMap<ResourceId, State>) {
    for (resource_id, state) in current_states.iter_mut() {
        if !state.exists || resource_id.provider != "awscc" {
            continue;
        }
        let config = match crate::schemas::generated::get_config_by_type(&resource_id.resource_type)
        {
            Some(c) => c,
            None => continue,
        };
        // Route canonicalization through `Schema::canonicalize_attr`
        // so cyclic CFN attributes (`AttributeType::Ref`) resolve
        // against this resource's def map. The defs-less
        // `carina_core::value::canonicalize_with_type` wrapper this
        // replaced silently used an empty definition map and panicked on every
        // Ref-containing attribute (carina#3345 Symptom B —
        // `ec2_vpc_endpoint/{gateway,interface}` plan-verify).
        let schema_view = carina_core::schema::Schema::with_defs(config.schema.defs.clone());
        let mut new_attrs = HashMap::with_capacity(state.attributes.len());
        for (key, value) in std::mem::take(&mut state.attributes) {
            let canon = match config.schema.attributes.get(key.as_str()) {
                Some(attr_schema) => schema_view.canonicalize_attr(&attr_schema.attr_type, value),
                None => value,
            };
            new_attrs.insert(key, canon);
        }
        state.attributes = new_attrs;
    }
}

/// Restore unreturned attributes from saved state into current read states.
///
/// CloudControl API doesn't always return all properties in GetResource responses
/// (create-only properties, and some normal properties like `description`).
/// We carry them forward from the previously saved attribute values.
pub fn restore_unreturned_attrs_impl(
    current_states: &mut HashMap<ResourceId, State>,
    saved_attrs: &HashMap<ResourceId, HashMap<String, Value>>,
) {
    for (resource_id, state) in current_states.iter_mut() {
        if !state.exists || resource_id.provider != "awscc" {
            continue;
        }
        let config = match crate::schemas::generated::get_config_by_type(&resource_id.resource_type)
        {
            Some(c) => c,
            None => continue,
        };
        let saved = match saved_attrs.get(resource_id) {
            Some(attrs) => attrs,
            None => continue,
        };
        for dsl_name in config.schema.attributes.keys() {
            if !state.attributes.contains_key(dsl_name)
                && let Some(value) = saved.get(dsl_name)
            {
                state.attributes.insert(dsl_name.clone(), value.clone());
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use carina_core::schema::legacy_validator;

    fn test_resource_schema() -> ResourceSchema {
        ResourceSchema::new("test.Resource")
    }

    #[test]
    fn test_resolve_enum_identifiers_bare_ident() {
        let mut resource = Resource::with_provider("awscc", "ec2.Vpc", "test", None);
        resource.set_attr(
            "instance_tenancy".to_string(),
            Value::Concrete(ConcreteValue::String("dedicated".to_string())),
        );

        let mut resources = vec![resource];
        resolve_enum_identifiers_impl(&mut resources);
        match resources[0].get_attr("instance_tenancy").unwrap() {
            Value::Concrete(ConcreteValue::String(s)) => assert!(
                s.contains("InstanceTenancy") && s.contains("dedicated"),
                "Expected namespaced enum, got: {}",
                s
            ),
            other => panic!("Expected String, got: {:?}", other),
        }
    }

    #[test]
    fn test_resolve_enum_identifiers_typename_value() {
        let mut resource = Resource::with_provider("awscc", "ec2.Vpc", "test", None);
        resource.set_attr(
            "instance_tenancy".to_string(),
            Value::Concrete(ConcreteValue::String(
                "InstanceTenancy.dedicated".to_string(),
            )),
        );

        let mut resources = vec![resource];
        resolve_enum_identifiers_impl(&mut resources);
        match resources[0].get_attr("instance_tenancy").unwrap() {
            Value::Concrete(ConcreteValue::String(s)) => assert!(
                s.contains("InstanceTenancy") && s.contains("dedicated"),
                "Expected namespaced enum, got: {}",
                s
            ),
            other => panic!("Expected String, got: {:?}", other),
        }
    }

    #[test]
    fn test_resolve_enum_identifiers_skips_non_awscc() {
        let mut resource = Resource::with_provider("aws", "s3.Bucket", "test", None);
        resource.set_attr(
            "instance_tenancy".to_string(),
            Value::Concrete(ConcreteValue::String("dedicated".to_string())),
        );

        let mut resources = vec![resource];
        resolve_enum_identifiers_impl(&mut resources);
        assert!(matches!(
            resources[0].get_attr("instance_tenancy").unwrap(),
            Value::Concrete(ConcreteValue::String(_))
        ));
    }

    /// aws#313 bare-`effect` path: the issue notes `effect` is
    /// typically written bare (`effect = allow`) — the parser emits
    /// `ConcreteValue::EnumIdentifier("allow")`, which exercises
    /// `resolve_enum_value` Case 1 (no dots), a different branch than
    /// the already-fully-qualified `version` path. The desired side
    /// must resolve to the same fully-qualified DSL form that the
    /// AWS-read side produces from raw `"Allow"`
    /// (`awscc.iam.PolicyDocument.Statement.Effect.allow`), or the differ diverges.
    #[test]
    fn test_aws313_bare_effect_desired_resolves_to_namespaced() {
        use indexmap::IndexMap;

        let mut stmt = IndexMap::new();
        stmt.insert(
            "effect".to_string(),
            Value::Concrete(ConcreteValue::EnumIdentifier("allow".to_string())),
        );
        stmt.insert(
            "action".to_string(),
            Value::Concrete(ConcreteValue::String("sts:AssumeRole".to_string())),
        );
        let mut policy = IndexMap::new();
        policy.insert(
            "version".to_string(),
            Value::Concrete(ConcreteValue::String(
                "awscc.iam.PolicyDocument.Version.2012_10_17".to_string(),
            )),
        );
        policy.insert(
            "statement".to_string(),
            Value::Concrete(ConcreteValue::List(vec![Value::Concrete(
                ConcreteValue::Map(stmt),
            )])),
        );
        let mut resource = Resource::with_provider("awscc", "iam.Role", "rd-role", None);
        resource.set_attr(
            "assume_role_policy_document".to_string(),
            Value::Concrete(ConcreteValue::Map(policy)),
        );

        let mut resources = vec![resource];
        resolve_enum_identifiers_impl(&mut resources);

        let Some(Value::Concrete(ConcreteValue::Map(doc))) =
            resources[0].get_attr("assume_role_policy_document")
        else {
            panic!("expected assume_role_policy_document Map");
        };
        let Some(Value::Concrete(ConcreteValue::List(stmts))) = doc.get("statement") else {
            panic!("expected statement List");
        };
        let Some(Value::Concrete(ConcreteValue::Map(s0))) = stmts.first() else {
            panic!("expected statement[0] Map");
        };
        assert_eq!(
            s0.get("effect"),
            Some(&Value::Concrete(ConcreteValue::String(
                "awscc.iam.PolicyDocument.Statement.Effect.allow".to_string()
            ))),
            "bare `effect = allow` desired must resolve to the same \
             fully-qualified form the read side produces from \"Allow\""
        );
    }

    #[test]
    fn test_resolve_enum_identifiers_hyphen_to_underscore() {
        let mut resource = Resource::with_provider("awscc", "ec2.FlowLog", "test", None);
        resource.set_attr(
            "log_destination_type".to_string(),
            Value::Concrete(ConcreteValue::String("cloud_watch_logs".to_string())),
        );

        let mut resources = vec![resource];
        resolve_enum_identifiers_impl(&mut resources);
        match resources[0].get_attr("log_destination_type").unwrap() {
            Value::Concrete(ConcreteValue::String(s)) => {
                assert_eq!(
                    s, "awscc.ec2.FlowLog.LogDestinationType.cloud_watch_logs",
                    "Expected underscored namespaced enum, got: {}",
                    s
                );
            }
            other => panic!("Expected String, got: {:?}", other),
        }
    }

    #[test]
    fn test_resolve_enum_identifiers_hyphen_string_to_underscore() {
        let mut resource = Resource::with_provider("awscc", "ec2.FlowLog", "test", None);
        resource.set_attr(
            "log_destination_type".to_string(),
            Value::Concrete(ConcreteValue::String("cloud-watch-logs".to_string())),
        );

        let mut resources = vec![resource];
        resolve_enum_identifiers_impl(&mut resources);
        match resources[0].get_attr("log_destination_type").unwrap() {
            Value::Concrete(ConcreteValue::String(s)) => {
                assert_eq!(
                    s, "awscc.ec2.FlowLog.LogDestinationType.cloud_watch_logs",
                    "Hyphenated string should be converted to underscore form, got: {}",
                    s
                );
            }
            other => panic!("Expected String, got: {:?}", other),
        }
    }

    #[test]
    fn test_restore_unreturned_attrs_impl_create_only() {
        let id = ResourceId::with_provider("awscc", "ec2.NatGateway", "test", None);
        let mut state = State::existing(id.clone(), HashMap::new());
        state.attributes.insert(
            "nat_gateway_id".to_string(),
            Value::Concrete(ConcreteValue::String("nat-123".to_string())),
        );

        let mut current_states = HashMap::new();
        current_states.insert(id.clone(), state);

        let mut saved = HashMap::new();
        saved.insert(
            "subnet_id".to_string(),
            Value::Concrete(ConcreteValue::String("subnet-abc".to_string())),
        );
        let mut saved_attrs = HashMap::new();
        saved_attrs.insert(id.clone(), saved);

        restore_unreturned_attrs_impl(&mut current_states, &saved_attrs);

        assert_eq!(
            current_states[&id].attributes.get("subnet_id"),
            Some(&Value::Concrete(ConcreteValue::String(
                "subnet-abc".to_string()
            )))
        );
    }

    #[test]
    fn test_restore_unreturned_attrs_skips_non_awscc() {
        let id = ResourceId::with_provider("aws", "s3.Bucket", "test", None);
        let state = State::existing(id.clone(), HashMap::new());

        let mut current_states = HashMap::new();
        current_states.insert(id.clone(), state);

        let mut saved = HashMap::new();
        saved.insert(
            "some_attr".to_string(),
            Value::Concrete(ConcreteValue::String("value".to_string())),
        );
        let mut saved_attrs = HashMap::new();
        saved_attrs.insert(id.clone(), saved);

        restore_unreturned_attrs_impl(&mut current_states, &saved_attrs);

        assert!(!current_states[&id].attributes.contains_key("some_attr"));
    }

    #[test]
    fn test_restore_unreturned_attrs_skips_already_present() {
        let id = ResourceId::with_provider("awscc", "ec2.NatGateway", "test", None);
        let mut attrs = HashMap::new();
        attrs.insert(
            "subnet_id".to_string(),
            Value::Concrete(ConcreteValue::String("subnet-current".to_string())),
        );
        let state = State::existing(id.clone(), attrs);

        let mut current_states = HashMap::new();
        current_states.insert(id.clone(), state);

        let mut saved = HashMap::new();
        saved.insert(
            "subnet_id".to_string(),
            Value::Concrete(ConcreteValue::String("subnet-saved".to_string())),
        );
        let mut saved_attrs = HashMap::new();
        saved_attrs.insert(id.clone(), saved);

        restore_unreturned_attrs_impl(&mut current_states, &saved_attrs);

        assert_eq!(
            current_states[&id].attributes.get("subnet_id"),
            Some(&Value::Concrete(ConcreteValue::String(
                "subnet-current".to_string()
            )))
        );
    }

    #[test]
    fn test_restore_unreturned_attrs_impl_non_create_only() {
        let id = ResourceId::with_provider("awscc", "ec2.SecurityGroupEgress", "test", None);
        let mut state = State::existing(id.clone(), HashMap::new());
        state.attributes.insert(
            "ip_protocol".to_string(),
            Value::Concrete(ConcreteValue::String(
                "awscc.ec2.SecurityGroupEgress.IpProtocol.all".to_string(),
            )),
        );

        let mut current_states = HashMap::new();
        current_states.insert(id.clone(), state);

        let mut saved = HashMap::new();
        saved.insert(
            "description".to_string(),
            Value::Concrete(ConcreteValue::String("Allow all outbound".to_string())),
        );
        let mut saved_attrs = HashMap::new();
        saved_attrs.insert(id.clone(), saved);

        restore_unreturned_attrs_impl(&mut current_states, &saved_attrs);

        assert_eq!(
            current_states[&id].attributes.get("description"),
            Some(&Value::Concrete(ConcreteValue::String(
                "Allow all outbound".to_string()
            )))
        );
    }

    #[test]
    fn route53_hosted_zone_name_trailing_dot_has_no_diff_without_registration() {
        let config = crate::schemas::generated::get_config_by_type("route53.HostedZone")
            .expect("route53.HostedZone schema should exist");
        let desired = Resource::with_provider("awscc", "route53.HostedZone", "example", None)
            .with_attribute(
                "name",
                Value::Concrete(ConcreteValue::String("example.com".to_string())),
            );
        let mut current_states = HashMap::from([(
            desired.id.clone(),
            State::existing(
                desired.id.clone(),
                HashMap::from([(
                    "name".to_string(),
                    Value::Concrete(ConcreteValue::String("example.com.".to_string())),
                )]),
            ),
        )]);

        normalize_state_enums_impl(&mut current_states);

        let current = current_states
            .get(&desired.id)
            .expect("normalized state should still exist");
        assert_eq!(
            current.attributes.get("name"),
            Some(&Value::Concrete(ConcreteValue::String(
                "example.com".to_string()
            )))
        );
        let diff = carina_core::differ::diff(&desired, current, None, None, Some(&config.schema));
        assert!(
            matches!(diff, carina_core::differ::Diff::NoChange(_)),
            "expected no diff for Route53 HostedZone name with trailing dot, got {diff:?}"
        );
    }

    #[test]
    fn test_resolve_enum_identifiers_ip_protocol_all_alias() {
        let mut resource =
            Resource::with_provider("awscc", "ec2.SecurityGroupEgress", "test", None);
        resource.set_attr(
            "ip_protocol".to_string(),
            Value::Concrete(ConcreteValue::String("all".to_string())),
        );

        let mut resources = vec![resource];
        resolve_enum_identifiers_impl(&mut resources);
        match resources[0].get_attr("ip_protocol").unwrap() {
            Value::Concrete(ConcreteValue::String(s)) => {
                assert_eq!(
                    s, "awscc.ec2.SecurityGroupEgress.IpProtocol.all",
                    "Expected namespaced IpProtocol.all, got: {}",
                    s
                );
            }
            other => panic!("Expected String, got: {:?}", other),
        }
    }

    #[test]
    fn test_resolve_enum_identifiers_ip_protocol_tcp() {
        let mut resource =
            Resource::with_provider("awscc", "ec2.SecurityGroupEgress", "test", None);
        resource.set_attr(
            "ip_protocol".to_string(),
            Value::Concrete(ConcreteValue::String("tcp".to_string())),
        );

        let mut resources = vec![resource];
        resolve_enum_identifiers_impl(&mut resources);
        match resources[0].get_attr("ip_protocol").unwrap() {
            Value::Concrete(ConcreteValue::String(s)) => {
                assert_eq!(
                    s, "awscc.ec2.SecurityGroupEgress.IpProtocol.tcp",
                    "Expected namespaced IpProtocol.tcp, got: {}",
                    s
                );
            }
            other => panic!("Expected String, got: {:?}", other),
        }
    }

    /// Helper to create struct fields with an enum type for testing
    fn test_ip_protocol_fields() -> Vec<StructField> {
        vec![
            StructField::new(
                "ip_protocol",
                AttributeType::enum_(
                    carina_core::schema::TypeIdentity::new(
                        Some("awscc"),
                        ["ec2", "SecurityGroup"],
                        "IpProtocol",
                    ),
                    None,
                    vec![],
                    Some(legacy_validator(|_| Ok(()))),
                    Some(carina_core::schema::DslTransform::ReplaceTable(vec![(
                        "-1".to_string(),
                        "all".to_string(),
                    )])),
                ),
            )
            .with_provider_name("IpProtocol"),
            StructField::new("from_port", AttributeType::int()).with_provider_name("FromPort"),
            StructField::new("cidr_ip", AttributeType::string()).with_provider_name("CidrIp"),
        ]
    }

    #[test]
    fn test_resolve_struct_enum_values_bare_ident() {
        let fields = test_ip_protocol_fields();
        let mut map = IndexMap::new();
        map.insert(
            "ip_protocol".to_string(),
            Value::Concrete(ConcreteValue::String("all".to_string())),
        );
        map.insert(
            "from_port".to_string(),
            Value::Concrete(ConcreteValue::Int(443)),
        );
        let value = Value::Concrete(ConcreteValue::List(vec![Value::Concrete(
            ConcreteValue::Map(map),
        )]));

        let schema = test_resource_schema();
        let resolved = resolve_struct_enum_values(&value, &fields, &schema);
        if let Value::Concrete(ConcreteValue::List(items)) = resolved {
            if let Value::Concrete(ConcreteValue::Map(m)) = &items[0] {
                match &m["ip_protocol"] {
                    Value::Concrete(ConcreteValue::String(s)) => {
                        assert_eq!(s, "awscc.ec2.SecurityGroup.IpProtocol.all");
                    }
                    other => panic!("Expected String, got: {:?}", other),
                }
                assert_eq!(m["from_port"], Value::Concrete(ConcreteValue::Int(443)));
            } else {
                panic!("Expected Map");
            }
        } else {
            panic!("Expected List");
        }
    }

    #[test]
    fn test_resolve_struct_enum_values_typename_dot_value() {
        let fields = test_ip_protocol_fields();
        let mut map = IndexMap::new();
        map.insert(
            "ip_protocol".to_string(),
            Value::Concrete(ConcreteValue::String("IpProtocol.tcp".to_string())),
        );
        let value = Value::Concrete(ConcreteValue::List(vec![Value::Concrete(
            ConcreteValue::Map(map),
        )]));

        let schema = test_resource_schema();
        let resolved = resolve_struct_enum_values(&value, &fields, &schema);
        if let Value::Concrete(ConcreteValue::List(items)) = resolved {
            if let Value::Concrete(ConcreteValue::Map(m)) = &items[0] {
                match &m["ip_protocol"] {
                    Value::Concrete(ConcreteValue::String(s)) => {
                        assert_eq!(s, "awscc.ec2.SecurityGroup.IpProtocol.tcp");
                    }
                    other => panic!("Expected String, got: {:?}", other),
                }
            } else {
                panic!("Expected Map");
            }
        } else {
            panic!("Expected List");
        }
    }

    #[test]
    fn test_resolve_struct_enum_values_string_passthrough() {
        let fields = test_ip_protocol_fields();
        let mut map = IndexMap::new();
        map.insert(
            "ip_protocol".to_string(),
            Value::Concrete(ConcreteValue::String(
                "awscc.ec2.SecurityGroup.IpProtocol.tcp".to_string(),
            )),
        );
        let value = Value::Concrete(ConcreteValue::List(vec![Value::Concrete(
            ConcreteValue::Map(map),
        )]));

        let schema = test_resource_schema();
        let resolved = resolve_struct_enum_values(&value, &fields, &schema);
        if let Value::Concrete(ConcreteValue::List(items)) = resolved {
            if let Value::Concrete(ConcreteValue::Map(m)) = &items[0] {
                match &m["ip_protocol"] {
                    Value::Concrete(ConcreteValue::String(s)) => {
                        assert_eq!(s, "awscc.ec2.SecurityGroup.IpProtocol.tcp");
                    }
                    other => panic!("Expected String, got: {:?}", other),
                }
            } else {
                panic!("Expected Map");
            }
        } else {
            panic!("Expected List");
        }
    }

    #[test]
    fn test_resolve_enum_identifiers_impl_struct_field() {
        let mut resource = Resource::with_provider("awscc", "ec2.SecurityGroup", "test-sg", None);
        resource.set_attr(
            "group_description".to_string(),
            Value::Concrete(ConcreteValue::String("test".to_string())),
        );
        let mut egress_map = IndexMap::new();
        egress_map.insert(
            "ip_protocol".to_string(),
            Value::Concrete(ConcreteValue::String("all".to_string())),
        );
        egress_map.insert(
            "cidr_ip".to_string(),
            Value::Concrete(ConcreteValue::String("0.0.0.0/0".to_string())),
        );
        resource.set_attr(
            "security_group_egress".to_string(),
            Value::Concrete(ConcreteValue::List(vec![Value::Concrete(
                ConcreteValue::Map(egress_map),
            )])),
        );

        let mut resources = vec![resource];
        resolve_enum_identifiers_impl(&mut resources);

        if let Value::Concrete(ConcreteValue::List(items)) =
            resources[0].get_attr("security_group_egress").unwrap()
        {
            if let Value::Concrete(ConcreteValue::Map(m)) = &items[0] {
                match &m["ip_protocol"] {
                    Value::Concrete(ConcreteValue::String(s)) => {
                        assert_eq!(
                            s, "awscc.ec2.SecurityGroup.Egress.IpProtocol.all",
                            "Expected namespaced Egress.IpProtocol.all in struct field, got: {}",
                            s
                        );
                    }
                    other => panic!("Expected String for ip_protocol, got: {:?}", other),
                }
                match &m["cidr_ip"] {
                    Value::Concrete(ConcreteValue::String(s)) => assert_eq!(s, "0.0.0.0/0"),
                    other => panic!("Expected String for cidr_ip, got: {:?}", other),
                }
            } else {
                panic!("Expected Map in egress list");
            }
        } else {
            panic!("Expected List for security_group_egress");
        }
    }

    // --- ip_protocol enum "all" variant tests (issue #1428) ---

    #[test]
    fn test_security_group_egress_schema_includes_all_variant() {
        let config =
            crate::schemas::generated::ec2::security_group_egress::ec2_security_group_egress_config(
            );
        let ip_protocol = config
            .schema
            .attributes
            .get("ip_protocol")
            .expect("ip_protocol attribute not found");
        if let carina_core::schema::Shape::Enum {
            values: Some(values),
            ..
        } = config.schema.shape_of(&ip_protocol.attr_type)
        {
            assert!(
                values.contains(&"all".to_string()),
                "Enum values must include 'all': {:?}",
                values
            );
        } else {
            panic!("ip_protocol should be Enum");
        }
    }

    #[test]
    fn test_security_group_ingress_schema_includes_all_variant() {
        let config = crate::schemas::generated::ec2::security_group_ingress::ec2_security_group_ingress_config();
        let ip_protocol = config
            .schema
            .attributes
            .get("ip_protocol")
            .expect("ip_protocol attribute not found");
        if let carina_core::schema::Shape::Enum {
            values: Some(values),
            ..
        } = config.schema.shape_of(&ip_protocol.attr_type)
        {
            assert!(
                values.contains(&"all".to_string()),
                "Enum values must include 'all': {:?}",
                values
            );
        } else {
            panic!("ip_protocol should be Enum");
        }
    }

    #[test]
    fn test_security_group_egress_struct_schema_includes_all_variant() {
        let config = crate::schemas::generated::ec2::security_group::ec2_security_group_config();
        let egress = config
            .schema
            .attributes
            .get("security_group_egress")
            .expect("security_group_egress attribute not found");
        // Drill into List -> Struct -> ip_protocol field
        if let carina_core::schema::Shape::List { inner, .. } =
            config.schema.shape_of(&egress.attr_type)
        {
            if let carina_core::schema::Shape::Struct { .. } = config.schema.shape_of(inner) {
                let fields = config
                    .schema
                    .struct_fields_with_budget(inner, &mut ShapeWalkBudget::new(256))
                    .expect("egress struct should expose fields");
                let ip_field = fields
                    .iter()
                    .find(|f| f.name == "ip_protocol")
                    .expect("ip_protocol field not found in egress struct");
                if let carina_core::schema::Shape::Enum {
                    values: Some(values),
                    ..
                } = config.schema.shape_of(&ip_field.field_type)
                {
                    assert!(
                        values.contains(&"all".to_string()),
                        "Enum values must include 'all': {:?}",
                        values
                    );
                } else {
                    panic!("ip_protocol should be Enum");
                }
            } else {
                panic!("Expected Struct inside List");
            }
        } else {
            panic!("Expected List for security_group_egress");
        }
    }

    #[test]
    fn test_security_group_ingress_struct_schema_includes_all_variant() {
        let config = crate::schemas::generated::ec2::security_group::ec2_security_group_config();
        let ingress = config
            .schema
            .attributes
            .get("security_group_ingress")
            .expect("security_group_ingress attribute not found");
        if let carina_core::schema::Shape::List { inner, .. } =
            config.schema.shape_of(&ingress.attr_type)
        {
            if let carina_core::schema::Shape::Struct { .. } = config.schema.shape_of(inner) {
                let fields = config
                    .schema
                    .struct_fields_with_budget(inner, &mut ShapeWalkBudget::new(256))
                    .expect("ingress struct should expose fields");
                let ip_field = fields
                    .iter()
                    .find(|f| f.name == "ip_protocol")
                    .expect("ip_protocol field not found in ingress struct");
                if let carina_core::schema::Shape::Enum {
                    values: Some(values),
                    ..
                } = config.schema.shape_of(&ip_field.field_type)
                {
                    assert!(
                        values.contains(&"all".to_string()),
                        "Enum values must include 'all': {:?}",
                        values
                    );
                } else {
                    panic!("ip_protocol should be Enum");
                }
            } else {
                panic!("Expected Struct inside List");
            }
        } else {
            panic!("Expected List for security_group_ingress");
        }
    }

    /// Nested struct: a Struct field containing another Struct with an enum.
    /// Reproduces the S3 bucket_encryption issue where
    /// blocked_encryption_types.encryption_type is a List(Enum) inside a nested Struct.
    #[test]
    fn test_resolve_struct_enum_values_nested_struct() {
        let inner_fields = vec![StructField::new(
            "encryption_type",
            AttributeType::list(AttributeType::enum_(
                carina_core::schema::enum_identity("EncryptionType", Some("awscc.s3.Bucket")),
                Some(vec!["NONE".to_string(), "SSE-C".to_string()]),
                vec![
                    ("NONE".to_string(), "none".to_string()),
                    ("SSE-C".to_string(), "sse_c".to_string()),
                ],
                None,
                None,
            )),
        )];

        let fields = vec![
            StructField::new(
                "blocked_encryption_types",
                AttributeType::struct_("BlockedEncryptionTypes".to_string(), inner_fields),
            ),
            StructField::new("bucket_key_enabled", AttributeType::bool()),
            StructField::new(
                "server_side_encryption_by_default",
                AttributeType::struct_(
                    "ServerSideEncryptionByDefault".to_string(),
                    vec![StructField::new(
                        "sse_algorithm",
                        AttributeType::enum_(
                            carina_core::schema::enum_identity(
                                "SseAlgorithm",
                                Some("awscc.s3.Bucket"),
                            ),
                            Some(vec!["AES256".to_string()]),
                            vec![("AES256".to_string(), "aes256".to_string())],
                            None,
                            None,
                        ),
                    )],
                ),
            ),
        ];

        let mut inner_map = IndexMap::new();
        inner_map.insert(
            "encryption_type".to_string(),
            Value::Concrete(ConcreteValue::List(vec![Value::Concrete(
                ConcreteValue::String("SSE-C".to_string()),
            )])),
        );
        let mut map = IndexMap::new();
        map.insert(
            "blocked_encryption_types".to_string(),
            Value::Concrete(ConcreteValue::Map(inner_map)),
        );
        map.insert(
            "bucket_key_enabled".to_string(),
            Value::Concrete(ConcreteValue::Bool(false)),
        );
        let mut sse_map = IndexMap::new();
        sse_map.insert(
            "sse_algorithm".to_string(),
            Value::Concrete(ConcreteValue::String("AES256".to_string())),
        );
        map.insert(
            "server_side_encryption_by_default".to_string(),
            Value::Concrete(ConcreteValue::Map(sse_map)),
        );

        let value = Value::Concrete(ConcreteValue::List(vec![Value::Concrete(
            ConcreteValue::Map(map),
        )]));
        let schema = test_resource_schema();
        let resolved = resolve_struct_enum_values(&value, &fields, &schema);

        // Verify the nested enum was resolved
        if let Value::Concrete(ConcreteValue::List(items)) = &resolved {
            if let Value::Concrete(ConcreteValue::Map(m)) = &items[0] {
                if let Value::Concrete(ConcreteValue::Map(blocked)) = &m["blocked_encryption_types"]
                {
                    if let Value::Concrete(ConcreteValue::List(types)) = &blocked["encryption_type"]
                    {
                        assert_eq!(
                            types[0],
                            Value::Concrete(ConcreteValue::String(
                                "awscc.s3.Bucket.EncryptionType.sse_c".to_string()
                            )),
                            "Nested struct enum should be resolved to its snake_case DSL form"
                        );
                    } else {
                        panic!("Expected List for encryption_type");
                    }
                } else {
                    panic!("Expected Map for blocked_encryption_types");
                }
                // Also verify sse_algorithm in sibling struct.
                // SHOUTY_SNAKE values follow the same D7 transform: API
                // `AES256` -> DSL `aes256`.
                if let Value::Concrete(ConcreteValue::Map(sse)) =
                    &m["server_side_encryption_by_default"]
                {
                    assert_eq!(
                        sse["sse_algorithm"],
                        Value::Concrete(ConcreteValue::String(
                            "awscc.s3.Bucket.SseAlgorithm.aes256".to_string()
                        )),
                        "Sibling struct enum should also be resolved to its snake_case DSL form"
                    );
                } else {
                    panic!("Expected Map for server_side_encryption_by_default");
                }
            } else {
                panic!("Expected Map");
            }
        } else {
            panic!("Expected List");
        }
    }

    // ---- canonicalize_string_or_list_states_impl tests ----
    // (carina-rs/carina#2481, sub-issue 5)

    fn make_state(
        provider: &str,
        resource_type: &str,
        name: &str,
        attrs: Vec<(&str, Value)>,
    ) -> (ResourceId, State) {
        use carina_core::resource::ResourceName;
        use std::collections::BTreeSet;
        let id = ResourceId {
            provider: provider.to_string(),
            resource_type: resource_type.to_string(),
            name: ResourceName::Bound(name.to_string()),
            provider_instance: None,
        };
        let mut attributes = HashMap::new();
        for (k, v) in attrs {
            attributes.insert(k.to_string(), v);
        }
        let state = State {
            id: id.clone(),
            identifier: Some(name.to_string()),
            attributes,
            exists: true,
            dependency_bindings: BTreeSet::new(),
        };
        (id, state)
    }

    #[test]
    fn canonicalize_string_or_list_recurses_into_iam_policy_document() {
        // AWS::IAM::RolePolicy has `policy_document` typed as
        // `iam_policy_document()`, a Struct that nests
        // Union[String, list(String)] fields like Statement[].Action.
        // The provider canonicalize pass walks via the Struct schema
        // and folds nested scalars to StringList.
        let mut policy_map = IndexMap::new();
        policy_map.insert(
            "Statement".to_string(),
            Value::Concrete(ConcreteValue::List(vec![Value::Concrete(
                ConcreteValue::Map({
                    let mut stmt = IndexMap::new();
                    stmt.insert(
                        "Effect".to_string(),
                        Value::Concrete(ConcreteValue::String("Allow".to_string())),
                    );
                    stmt.insert(
                        "Action".to_string(),
                        Value::Concrete(ConcreteValue::String("s3:GetObject".to_string())),
                    );
                    stmt.insert(
                        "Resource".to_string(),
                        Value::Concrete(ConcreteValue::String(
                            "arn:aws:s3:::my-bucket/*".to_string(),
                        )),
                    );
                    stmt
                }),
            )])),
        );

        let (id, state) = make_state(
            "awscc",
            "iam.RolePolicy",
            "test-policy",
            vec![
                (
                    "policy_name",
                    Value::Concrete(ConcreteValue::String("test-policy".to_string())),
                ),
                (
                    "policy_document",
                    Value::Concrete(ConcreteValue::Map(policy_map)),
                ),
            ],
        );
        let mut current_states: HashMap<ResourceId, State> = HashMap::new();
        current_states.insert(id.clone(), state);

        canonicalize_string_or_list_states_impl(&mut current_states);

        // Statement[0].Action: scalar → StringList(["s3:GetObject"])
        let state = &current_states[&id];
        let policy_document = state
            .attributes
            .get("policy_document")
            .expect("policy_document present");
        let Value::Concrete(ConcreteValue::Map(pd)) = policy_document else {
            panic!("expected Map for policy_document");
        };
        let Value::Concrete(ConcreteValue::List(stmts)) =
            pd.get("Statement").expect("Statement present")
        else {
            panic!("expected List for Statement");
        };
        let Value::Concrete(ConcreteValue::Map(stmt)) = &stmts[0] else {
            panic!("expected Map for Statement[0]");
        };
        assert_eq!(
            stmt.get("Action"),
            Some(&Value::Concrete(ConcreteValue::StringList(vec![
                "s3:GetObject".to_string()
            ]))),
            "Action should be canonicalized to StringList"
        );
        assert_eq!(
            stmt.get("Resource"),
            Some(&Value::Concrete(ConcreteValue::StringList(vec![
                "arn:aws:s3:::my-bucket/*".to_string()
            ]))),
            "Resource should be canonicalized to StringList"
        );
    }

    #[test]
    fn canonicalize_string_or_list_skips_non_awscc() {
        let (id, state) = make_state(
            "aws",
            "iam.RolePolicy",
            "test",
            vec![(
                "policy_name",
                Value::Concrete(ConcreteValue::String("test".to_string())),
            )],
        );
        let mut current_states: HashMap<ResourceId, State> = HashMap::new();
        current_states.insert(id.clone(), state);

        canonicalize_string_or_list_states_impl(&mut current_states);

        let state = &current_states[&id];
        assert_eq!(
            state.attributes.get("policy_name"),
            Some(&Value::Concrete(ConcreteValue::String("test".to_string()))),
            "non-awscc state untouched"
        );
    }

    #[test]
    fn canonicalize_string_or_list_skips_unknown_resource_type() {
        let (id, state) = make_state(
            "awscc",
            "unknown.UnknownType",
            "test",
            vec![(
                "attr",
                Value::Concrete(ConcreteValue::String("x".to_string())),
            )],
        );
        let mut current_states: HashMap<ResourceId, State> = HashMap::new();
        current_states.insert(id.clone(), state);

        canonicalize_string_or_list_states_impl(&mut current_states);

        let state = &current_states[&id];
        assert_eq!(
            state.attributes.get("attr"),
            Some(&Value::Concrete(ConcreteValue::String("x".to_string()))),
            "unknown resource types pass through unchanged"
        );
    }

    // --- cyclic-schema Ref normalization (awscc#286 / awscc#288) ---
    //
    // `resolve_struct_enum_values` recurses through nested struct fields
    // to resolve schema-level string enums. After the cyclic-schema Ref
    // codegen (awscc#281, carina#3340) the generated schemas emit
    // `AttributeType::Ref(name)` *inside* struct fields (e.g. WebACL's
    // recursive `Statement` graph, CloudFront's `forwarded_values`).
    // When the recursion reaches such a field it must peel the `Ref`
    // against this resource's `defs` map; peeling against an empty map
    // panics with "Ref(...) not found in schema defs", which poisons the
    // WASM instance and silently strips `default_tags`.

    #[test]
    fn resolve_struct_enum_values_threads_defs_for_webacl_ref() {
        // A WebACL rule carrying `captcha_config`, a `Ref("CaptchaConfig")`
        // field of the generated `Rule` struct (awscc#286). The recursion
        // must peel this `Ref` against the WebACL's `defs` map.
        let mut resource = Resource::with_provider("awscc", "wafv2.WebAcl", "test-acl", None);

        let mut immunity = IndexMap::new();
        immunity.insert(
            "immunity_time".to_string(),
            Value::Concrete(ConcreteValue::Int(60)),
        );
        let mut captcha_config = IndexMap::new();
        captcha_config.insert(
            "immunity_time_property".to_string(),
            Value::Concrete(ConcreteValue::Map(immunity)),
        );

        let mut rule = IndexMap::new();
        rule.insert(
            "name".to_string(),
            Value::Concrete(ConcreteValue::String("common-rule-set".to_string())),
        );
        rule.insert(
            "captcha_config".to_string(),
            Value::Concrete(ConcreteValue::Map(captcha_config)),
        );

        resource.set_attr(
            "rules".to_string(),
            Value::Concrete(ConcreteValue::List(vec![Value::Concrete(
                ConcreteValue::Map(rule),
            )])),
        );

        let mut resources = vec![resource];
        // Before the fix this panics: the recursion peels the
        // `Ref("CaptchaConfig")` field against an empty definition map.
        resolve_enum_identifiers_impl(&mut resources);

        let Value::Concrete(ConcreteValue::List(rules)) = resources[0].get_attr("rules").unwrap()
        else {
            panic!("expected rules list");
        };
        let Value::Concrete(ConcreteValue::Map(rule)) = &rules[0] else {
            panic!("expected rule map");
        };
        // The walk descends through the `Ref("CaptchaConfig")` field and
        // its nested `Ref("ImmunityTimeProperty")`, leaving the leaf value
        // intact — proving the recursion produced correct output, not just
        // that it did not panic.
        let Value::Concrete(ConcreteValue::Map(captcha_config)) = &rule["captcha_config"] else {
            panic!("expected captcha_config map");
        };
        let Value::Concrete(ConcreteValue::Map(immunity)) =
            &captcha_config["immunity_time_property"]
        else {
            panic!("expected immunity_time_property map");
        };
        assert_eq!(
            immunity["immunity_time"],
            Value::Concrete(ConcreteValue::Int(60)),
            "leaf value preserved through the Ref-peeled captcha_config"
        );
    }

    #[test]
    fn resolve_struct_enum_values_threads_defs_for_distribution_forwarded_values() {
        // A CloudFront Distribution whose
        // `distribution_config.default_cache_behavior.forwarded_values`
        // is a `Ref("ForwardedValues")` field (awscc#288). The state
        // path runs the same `resolve_struct_enum_values` recursion.
        let mut cookies = IndexMap::new();
        cookies.insert(
            "forward".to_string(),
            Value::Concrete(ConcreteValue::String("none".to_string())),
        );

        let mut forwarded_values = IndexMap::new();
        forwarded_values.insert(
            "query_string".to_string(),
            Value::Concrete(ConcreteValue::Bool(false)),
        );
        forwarded_values.insert(
            "cookies".to_string(),
            Value::Concrete(ConcreteValue::Map(cookies)),
        );

        let mut default_cache_behavior = IndexMap::new();
        default_cache_behavior.insert(
            "target_origin_id".to_string(),
            Value::Concrete(ConcreteValue::String("origin".to_string())),
        );
        default_cache_behavior.insert(
            "forwarded_values".to_string(),
            Value::Concrete(ConcreteValue::Map(forwarded_values)),
        );

        let mut distribution_config = IndexMap::new();
        distribution_config.insert(
            "enabled".to_string(),
            Value::Concrete(ConcreteValue::Bool(true)),
        );
        distribution_config.insert(
            "default_cache_behavior".to_string(),
            Value::Concrete(ConcreteValue::Map(default_cache_behavior)),
        );

        let (id, mut state) = make_state("awscc", "cloudfront.Distribution", "test-dist", vec![]);
        state.attributes.insert(
            "distribution_config".to_string(),
            Value::Concrete(ConcreteValue::Map(distribution_config)),
        );
        let mut current_states: HashMap<ResourceId, State> = HashMap::new();
        current_states.insert(id.clone(), state);

        // Before the fix this panics on `Ref("ForwardedValues")`.
        normalize_state_enums_impl(&mut current_states);

        let Value::Concrete(ConcreteValue::Map(cfg)) = current_states[&id]
            .attributes
            .get("distribution_config")
            .unwrap()
        else {
            panic!("expected distribution_config map");
        };
        // The walk descends through the `Ref("ForwardedValues")` field and
        // resolves the nested `cookies.forward` StringEnum, proving the
        // recursion produced correct output (not merely "did not panic").
        let Value::Concrete(ConcreteValue::Map(dcb)) = &cfg["default_cache_behavior"] else {
            panic!("expected default_cache_behavior map");
        };
        let Value::Concrete(ConcreteValue::Map(fv)) = &dcb["forwarded_values"] else {
            panic!("expected forwarded_values map");
        };
        let Value::Concrete(ConcreteValue::Map(cookies)) = &fv["cookies"] else {
            panic!("expected cookies map");
        };
        assert_eq!(
            cookies["forward"],
            Value::Concrete(ConcreteValue::String(
                "awscc.cloudfront.Distribution.DistributionConfig.CacheBehavior.ForwardedValues.Cookies.Forward.none".to_string()
            )),
            "nested CookiesForward.none resolved through the Ref-peeled forwarded_values"
        );
    }
}
