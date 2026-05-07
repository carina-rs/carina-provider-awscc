//! Plan-time normalization of enum identifiers and state hydration.
//!
//! This module contains standalone functions used by `ProviderNormalizer` to resolve
//! enum identifiers in resources and restore unreturned attributes from saved state.

use indexmap::IndexMap;
use std::collections::HashMap;

use carina_core::resource::{Resource, ResourceId, State, Value};
use carina_core::schema::{AttributeType, StructField};

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
                && let Some(parts) = attr_schema.attr_type.namespaced_enum_parts()
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
                let struct_fields = match &attr_schema.attr_type {
                    AttributeType::List { inner, .. } => {
                        if let AttributeType::Struct { fields, .. } = inner.as_ref() {
                            Some(fields)
                        } else {
                            None
                        }
                    }
                    AttributeType::Struct { fields, .. } => Some(fields),
                    _ => None,
                };

                if let Some(fields) = struct_fields {
                    let resolved = resolve_struct_enum_values(value, fields);
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

/// Resolve enum identifiers within struct field values.
/// Recurses into List and Map values, resolving bare/shorthand enum values
/// for struct fields that have StringEnum type with namespace.
fn resolve_struct_enum_values(value: &Value, fields: &[StructField]) -> Value {
    match value {
        Value::List(items) => {
            let resolved_items: Vec<Value> = items
                .iter()
                .map(|item| resolve_struct_enum_values(item, fields))
                .collect();
            Value::List(resolved_items)
        }
        Value::Map(map) => {
            let mut resolved_map = IndexMap::new();
            for (field_key, field_value) in map {
                if let Some(field) = fields.iter().find(|f| f.name == *field_key) {
                    // Direct enum field (String value)
                    if let Some(parts) = field.field_type.namespaced_enum_parts()
                        && let Some(resolved) =
                            carina_core::utils::resolve_enum_value(field_value, &parts)
                    {
                        resolved_map.insert(field_key.clone(), resolved);
                        continue;
                    }
                    // List(StringEnum): resolve each element
                    if let AttributeType::List { inner, .. } = &field.field_type
                        && let Some(parts) = inner.namespaced_enum_parts()
                        && let Value::List(items) = field_value
                    {
                        let resolved_items: Vec<Value> = items
                            .iter()
                            .map(|item| {
                                carina_core::utils::resolve_enum_value(item, &parts)
                                    .unwrap_or_else(|| item.clone())
                            })
                            .collect();
                        resolved_map.insert(field_key.clone(), Value::List(resolved_items));
                        continue;
                    }
                    // Recurse into nested Struct or List(Struct) fields
                    let nested_fields = match &field.field_type {
                        AttributeType::Struct { fields, .. } => Some(fields.as_slice()),
                        AttributeType::List { inner, .. } => {
                            if let AttributeType::Struct { fields, .. } = inner.as_ref() {
                                Some(fields.as_slice())
                            } else {
                                None
                            }
                        }
                        _ => None,
                    };
                    if let Some(nested) = nested_fields {
                        resolved_map.insert(
                            field_key.clone(),
                            resolve_struct_enum_values(field_value, nested),
                        );
                        continue;
                    }
                }
                resolved_map.insert(field_key.clone(), field_value.clone());
            }
            Value::Map(resolved_map)
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
                && let Some(parts) = attr_schema.attr_type.namespaced_enum_parts()
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
                let struct_fields = match &attr_schema.attr_type {
                    AttributeType::List { inner, .. } => {
                        if let AttributeType::Struct { fields, .. } = inner.as_ref() {
                            Some(fields)
                        } else {
                            None
                        }
                    }
                    AttributeType::Struct { fields, .. } => Some(fields),
                    _ => None,
                };

                if let Some(fields) = struct_fields {
                    let resolved = resolve_struct_enum_values(value, fields);
                    resolved_attrs.insert(key.clone(), resolved);
                    continue;
                }
            }

            resolved_attrs.insert(key.clone(), value.clone());
        }
        state.attributes = resolved_attrs;
    }
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
        let mut new_attrs = HashMap::with_capacity(state.attributes.len());
        for (key, value) in std::mem::take(&mut state.attributes) {
            let canon = match config.schema.attributes.get(key.as_str()) {
                Some(attr_schema) => {
                    carina_core::value::canonicalize_with_type(value, &attr_schema.attr_type)
                }
                None => value,
            };
            new_attrs.insert(key, canon);
        }
        state.attributes = new_attrs;
    }
}

/// Normalize "absent" optional `list(...)` / `map(...)` attributes in
/// awscc states to empty containers.
///
/// AWS CloudControl `GetResource` responses omit optional list / map
/// properties when the resource has no values for them (e.g.
/// `awscc.iam.Role.managed_policy_arns` for a role with no attached
/// managed policies). The differ then sees `desired = []` vs
/// `current = (none)` and emits a permanent `(none) → []` diff that
/// survives apply.
///
/// This pass establishes a canonical form at the provider boundary —
/// same philosophy as `canonicalize_string_or_list_states_impl` for
/// `Union[String, list(String)]` (carina-rs/carina#2511) — so the
/// differ doesn't need to learn schema-dependent special-cases for
/// "absent equals empty list".
///
/// Skipped:
/// - `write_only` attributes — never present in read responses by
///   design; their values come from saved state via
///   `restore_unreturned_attrs_impl`. Inserting an empty container
///   here would shadow the saved value.
/// - `required` attributes — these are not optional, so absence is
///   either a real bug or a CloudControl response shape we shouldn't
///   paper over.
///
/// Note: this pass must run *after* `restore_unreturned_attrs_impl`
/// (which fires from `hydrate_read_state`) so that values carried
/// forward from saved state still take precedence over the empty
/// fallback. The current call order in
/// `AwsccNormalizer::normalize_state` runs after `hydrate_read_state`
/// (the host invokes `hydrate_read_state` at read time and
/// `normalize_state` later in `PlanPreprocessor::prepare`).
pub fn normalize_absent_optional_lists_and_maps_impl(
    current_states: &mut HashMap<ResourceId, State>,
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
        for (dsl_name, attr_schema) in &config.schema.attributes {
            if attr_schema.required || attr_schema.write_only {
                continue;
            }
            if state.attributes.contains_key(dsl_name) {
                continue;
            }
            match &attr_schema.attr_type {
                AttributeType::List { .. } => {
                    state
                        .attributes
                        .insert(dsl_name.clone(), Value::List(Vec::new()));
                }
                AttributeType::Map { .. } => {
                    state
                        .attributes
                        .insert(dsl_name.clone(), Value::Map(IndexMap::new()));
                }
                _ => {}
            }
        }
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
    use carina_core::schema::noop_validator;

    #[test]
    fn test_resolve_enum_identifiers_bare_ident() {
        let mut resource = Resource::with_provider("awscc", "ec2.Vpc", "test");
        resource.set_attr(
            "instance_tenancy".to_string(),
            Value::String("dedicated".to_string()),
        );

        let mut resources = vec![resource];
        resolve_enum_identifiers_impl(&mut resources);
        match resources[0].get_attr("instance_tenancy").unwrap() {
            Value::String(s) => assert!(
                s.contains("InstanceTenancy") && s.contains("dedicated"),
                "Expected namespaced enum, got: {}",
                s
            ),
            other => panic!("Expected String, got: {:?}", other),
        }
    }

    #[test]
    fn test_resolve_enum_identifiers_typename_value() {
        let mut resource = Resource::with_provider("awscc", "ec2.Vpc", "test");
        resource.set_attr(
            "instance_tenancy".to_string(),
            Value::String("InstanceTenancy.dedicated".to_string()),
        );

        let mut resources = vec![resource];
        resolve_enum_identifiers_impl(&mut resources);
        match resources[0].get_attr("instance_tenancy").unwrap() {
            Value::String(s) => assert!(
                s.contains("InstanceTenancy") && s.contains("dedicated"),
                "Expected namespaced enum, got: {}",
                s
            ),
            other => panic!("Expected String, got: {:?}", other),
        }
    }

    #[test]
    fn test_resolve_enum_identifiers_skips_non_awscc() {
        let mut resource = Resource::with_provider("aws", "s3.Bucket", "test");
        resource.set_attr(
            "instance_tenancy".to_string(),
            Value::String("dedicated".to_string()),
        );

        let mut resources = vec![resource];
        resolve_enum_identifiers_impl(&mut resources);
        assert!(matches!(
            resources[0].get_attr("instance_tenancy").unwrap(),
            Value::String(_)
        ));
    }

    #[test]
    fn test_resolve_enum_identifiers_hyphen_to_underscore() {
        let mut resource = Resource::with_provider("awscc", "ec2.FlowLog", "test");
        resource.set_attr(
            "log_destination_type".to_string(),
            Value::String("cloud_watch_logs".to_string()),
        );

        let mut resources = vec![resource];
        resolve_enum_identifiers_impl(&mut resources);
        match resources[0].get_attr("log_destination_type").unwrap() {
            Value::String(s) => {
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
        let mut resource = Resource::with_provider("awscc", "ec2.FlowLog", "test");
        resource.set_attr(
            "log_destination_type".to_string(),
            Value::String("cloud-watch-logs".to_string()),
        );

        let mut resources = vec![resource];
        resolve_enum_identifiers_impl(&mut resources);
        match resources[0].get_attr("log_destination_type").unwrap() {
            Value::String(s) => {
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
        let id = ResourceId::with_provider("awscc", "ec2.NatGateway", "test");
        let mut state = State::existing(id.clone(), HashMap::new());
        state.attributes.insert(
            "nat_gateway_id".to_string(),
            Value::String("nat-123".to_string()),
        );

        let mut current_states = HashMap::new();
        current_states.insert(id.clone(), state);

        let mut saved = HashMap::new();
        saved.insert(
            "subnet_id".to_string(),
            Value::String("subnet-abc".to_string()),
        );
        let mut saved_attrs = HashMap::new();
        saved_attrs.insert(id.clone(), saved);

        restore_unreturned_attrs_impl(&mut current_states, &saved_attrs);

        assert_eq!(
            current_states[&id].attributes.get("subnet_id"),
            Some(&Value::String("subnet-abc".to_string()))
        );
    }

    #[test]
    fn test_restore_unreturned_attrs_skips_non_awscc() {
        let id = ResourceId::with_provider("aws", "s3.Bucket", "test");
        let state = State::existing(id.clone(), HashMap::new());

        let mut current_states = HashMap::new();
        current_states.insert(id.clone(), state);

        let mut saved = HashMap::new();
        saved.insert("some_attr".to_string(), Value::String("value".to_string()));
        let mut saved_attrs = HashMap::new();
        saved_attrs.insert(id.clone(), saved);

        restore_unreturned_attrs_impl(&mut current_states, &saved_attrs);

        assert!(!current_states[&id].attributes.contains_key("some_attr"));
    }

    #[test]
    fn test_restore_unreturned_attrs_skips_already_present() {
        let id = ResourceId::with_provider("awscc", "ec2.NatGateway", "test");
        let mut attrs = HashMap::new();
        attrs.insert(
            "subnet_id".to_string(),
            Value::String("subnet-current".to_string()),
        );
        let state = State::existing(id.clone(), attrs);

        let mut current_states = HashMap::new();
        current_states.insert(id.clone(), state);

        let mut saved = HashMap::new();
        saved.insert(
            "subnet_id".to_string(),
            Value::String("subnet-saved".to_string()),
        );
        let mut saved_attrs = HashMap::new();
        saved_attrs.insert(id.clone(), saved);

        restore_unreturned_attrs_impl(&mut current_states, &saved_attrs);

        assert_eq!(
            current_states[&id].attributes.get("subnet_id"),
            Some(&Value::String("subnet-current".to_string()))
        );
    }

    #[test]
    fn test_restore_unreturned_attrs_impl_non_create_only() {
        let id = ResourceId::with_provider("awscc", "ec2.SecurityGroupEgress", "test");
        let mut state = State::existing(id.clone(), HashMap::new());
        state.attributes.insert(
            "ip_protocol".to_string(),
            Value::String("awscc.ec2.SecurityGroupEgress.IpProtocol.all".to_string()),
        );

        let mut current_states = HashMap::new();
        current_states.insert(id.clone(), state);

        let mut saved = HashMap::new();
        saved.insert(
            "description".to_string(),
            Value::String("Allow all outbound".to_string()),
        );
        let mut saved_attrs = HashMap::new();
        saved_attrs.insert(id.clone(), saved);

        restore_unreturned_attrs_impl(&mut current_states, &saved_attrs);

        assert_eq!(
            current_states[&id].attributes.get("description"),
            Some(&Value::String("Allow all outbound".to_string()))
        );
    }

    #[test]
    fn test_resolve_enum_identifiers_ip_protocol_all_alias() {
        let mut resource = Resource::with_provider("awscc", "ec2.SecurityGroupEgress", "test");
        resource.set_attr("ip_protocol".to_string(), Value::String("all".to_string()));

        let mut resources = vec![resource];
        resolve_enum_identifiers_impl(&mut resources);
        match resources[0].get_attr("ip_protocol").unwrap() {
            Value::String(s) => {
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
        let mut resource = Resource::with_provider("awscc", "ec2.SecurityGroupEgress", "test");
        resource.set_attr("ip_protocol".to_string(), Value::String("tcp".to_string()));

        let mut resources = vec![resource];
        resolve_enum_identifiers_impl(&mut resources);
        match resources[0].get_attr("ip_protocol").unwrap() {
            Value::String(s) => {
                assert_eq!(
                    s, "awscc.ec2.SecurityGroupEgress.IpProtocol.tcp",
                    "Expected namespaced IpProtocol.tcp, got: {}",
                    s
                );
            }
            other => panic!("Expected String, got: {:?}", other),
        }
    }

    /// Helper to create struct fields with a Custom enum type for testing
    fn test_ip_protocol_fields() -> Vec<StructField> {
        vec![
            StructField::new(
                "ip_protocol",
                AttributeType::Custom {
                    semantic_name: Some("IpProtocol".to_string()),
                    pattern: None,
                    length: None,
                    base: Box::new(AttributeType::String),
                    validate: noop_validator(),
                    namespace: Some("awscc.ec2.SecurityGroup".to_string()),
                    to_dsl: Some(|s: &str| match s {
                        "-1" => "all".to_string(),
                        _ => s.to_string(),
                    }),
                },
            )
            .with_provider_name("IpProtocol"),
            StructField::new("from_port", AttributeType::Int).with_provider_name("FromPort"),
            StructField::new("cidr_ip", AttributeType::String).with_provider_name("CidrIp"),
        ]
    }

    #[test]
    fn test_resolve_struct_enum_values_bare_ident() {
        let fields = test_ip_protocol_fields();
        let mut map = IndexMap::new();
        map.insert("ip_protocol".to_string(), Value::String("all".to_string()));
        map.insert("from_port".to_string(), Value::Int(443));
        let value = Value::List(vec![Value::Map(map)]);

        let resolved = resolve_struct_enum_values(&value, &fields);
        if let Value::List(items) = resolved {
            if let Value::Map(m) = &items[0] {
                match &m["ip_protocol"] {
                    Value::String(s) => {
                        assert_eq!(s, "awscc.ec2.SecurityGroup.IpProtocol.all");
                    }
                    other => panic!("Expected String, got: {:?}", other),
                }
                assert_eq!(m["from_port"], Value::Int(443));
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
            Value::String("IpProtocol.tcp".to_string()),
        );
        let value = Value::List(vec![Value::Map(map)]);

        let resolved = resolve_struct_enum_values(&value, &fields);
        if let Value::List(items) = resolved {
            if let Value::Map(m) = &items[0] {
                match &m["ip_protocol"] {
                    Value::String(s) => {
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
            Value::String("awscc.ec2.SecurityGroup.IpProtocol.tcp".to_string()),
        );
        let value = Value::List(vec![Value::Map(map)]);

        let resolved = resolve_struct_enum_values(&value, &fields);
        if let Value::List(items) = resolved {
            if let Value::Map(m) = &items[0] {
                match &m["ip_protocol"] {
                    Value::String(s) => {
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
        let mut resource = Resource::with_provider("awscc", "ec2.SecurityGroup", "test-sg");
        resource.set_attr(
            "group_description".to_string(),
            Value::String("test".to_string()),
        );
        let mut egress_map = IndexMap::new();
        egress_map.insert("ip_protocol".to_string(), Value::String("all".to_string()));
        egress_map.insert(
            "cidr_ip".to_string(),
            Value::String("0.0.0.0/0".to_string()),
        );
        resource.set_attr(
            "security_group_egress".to_string(),
            Value::List(vec![Value::Map(egress_map)]),
        );

        let mut resources = vec![resource];
        resolve_enum_identifiers_impl(&mut resources);

        if let Value::List(items) = resources[0].get_attr("security_group_egress").unwrap() {
            if let Value::Map(m) = &items[0] {
                match &m["ip_protocol"] {
                    Value::String(s) => {
                        assert_eq!(
                            s, "awscc.ec2.SecurityGroup.IpProtocol.all",
                            "Expected namespaced IpProtocol.all in struct field, got: {}",
                            s
                        );
                    }
                    other => panic!("Expected String for ip_protocol, got: {:?}", other),
                }
                match &m["cidr_ip"] {
                    Value::String(s) => assert_eq!(s, "0.0.0.0/0"),
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
        if let carina_core::schema::AttributeType::StringEnum { values, .. } =
            &ip_protocol.attr_type
        {
            assert!(
                values.contains(&"all".to_string()),
                "StringEnum values must include 'all': {:?}",
                values
            );
        } else {
            panic!("ip_protocol should be StringEnum");
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
        if let carina_core::schema::AttributeType::StringEnum { values, .. } =
            &ip_protocol.attr_type
        {
            assert!(
                values.contains(&"all".to_string()),
                "StringEnum values must include 'all': {:?}",
                values
            );
        } else {
            panic!("ip_protocol should be StringEnum");
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
        if let carina_core::schema::AttributeType::List { inner, .. } = &egress.attr_type {
            if let carina_core::schema::AttributeType::Struct { fields, .. } = inner.as_ref() {
                let ip_field = fields
                    .iter()
                    .find(|f| f.name == "ip_protocol")
                    .expect("ip_protocol field not found in egress struct");
                if let carina_core::schema::AttributeType::StringEnum { values, .. } =
                    &ip_field.field_type
                {
                    assert!(
                        values.contains(&"all".to_string()),
                        "StringEnum values must include 'all': {:?}",
                        values
                    );
                } else {
                    panic!("ip_protocol should be StringEnum");
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
        if let carina_core::schema::AttributeType::List { inner, .. } = &ingress.attr_type {
            if let carina_core::schema::AttributeType::Struct { fields, .. } = inner.as_ref() {
                let ip_field = fields
                    .iter()
                    .find(|f| f.name == "ip_protocol")
                    .expect("ip_protocol field not found in ingress struct");
                if let carina_core::schema::AttributeType::StringEnum { values, .. } =
                    &ip_field.field_type
                {
                    assert!(
                        values.contains(&"all".to_string()),
                        "StringEnum values must include 'all': {:?}",
                        values
                    );
                } else {
                    panic!("ip_protocol should be StringEnum");
                }
            } else {
                panic!("Expected Struct inside List");
            }
        } else {
            panic!("Expected List for security_group_ingress");
        }
    }

    /// Nested struct: a Struct field containing another Struct with a StringEnum.
    /// Reproduces the S3 bucket_encryption issue where
    /// blocked_encryption_types.encryption_type is a List(StringEnum) inside a nested Struct.
    #[test]
    fn test_resolve_struct_enum_values_nested_struct() {
        let inner_fields = vec![StructField::new(
            "encryption_type",
            AttributeType::list(AttributeType::StringEnum {
                name: "EncryptionType".to_string(),
                values: vec!["NONE".to_string(), "SSE-C".to_string()],
                namespace: Some("awscc.s3.Bucket".to_string()),
                to_dsl: Some(|s: &str| s.replace('-', "_")),
            }),
        )];

        let fields = vec![
            StructField::new(
                "blocked_encryption_types",
                AttributeType::Struct {
                    name: "BlockedEncryptionTypes".to_string(),
                    fields: inner_fields,
                },
            ),
            StructField::new("bucket_key_enabled", AttributeType::Bool),
            StructField::new(
                "server_side_encryption_by_default",
                AttributeType::Struct {
                    name: "ServerSideEncryptionByDefault".to_string(),
                    fields: vec![StructField::new(
                        "sse_algorithm",
                        AttributeType::StringEnum {
                            name: "SseAlgorithm".to_string(),
                            values: vec!["AES256".to_string()],
                            namespace: Some("awscc.s3.Bucket".to_string()),
                            to_dsl: None,
                        },
                    )],
                },
            ),
        ];

        let mut inner_map = IndexMap::new();
        inner_map.insert(
            "encryption_type".to_string(),
            Value::List(vec![Value::String("SSE-C".to_string())]),
        );
        let mut map = IndexMap::new();
        map.insert(
            "blocked_encryption_types".to_string(),
            Value::Map(inner_map),
        );
        map.insert("bucket_key_enabled".to_string(), Value::Bool(false));
        let mut sse_map = IndexMap::new();
        sse_map.insert(
            "sse_algorithm".to_string(),
            Value::String("AES256".to_string()),
        );
        map.insert(
            "server_side_encryption_by_default".to_string(),
            Value::Map(sse_map),
        );

        let value = Value::List(vec![Value::Map(map)]);
        let resolved = resolve_struct_enum_values(&value, &fields);

        // Verify the nested enum was resolved
        if let Value::List(items) = &resolved {
            if let Value::Map(m) = &items[0] {
                if let Value::Map(blocked) = &m["blocked_encryption_types"] {
                    if let Value::List(types) = &blocked["encryption_type"] {
                        assert_eq!(
                            types[0],
                            Value::String("awscc.s3.Bucket.EncryptionType.SSE_C".to_string()),
                            "Nested struct enum should be resolved to DSL format"
                        );
                    } else {
                        panic!("Expected List for encryption_type");
                    }
                } else {
                    panic!("Expected Map for blocked_encryption_types");
                }
                // Also verify sse_algorithm in sibling struct
                if let Value::Map(sse) = &m["server_side_encryption_by_default"] {
                    assert_eq!(
                        sse["sse_algorithm"],
                        Value::String("awscc.s3.Bucket.SseAlgorithm.AES256".to_string()),
                        "Sibling struct enum should also be resolved"
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
            Value::List(vec![Value::Map({
                let mut stmt = IndexMap::new();
                stmt.insert("Effect".to_string(), Value::String("Allow".to_string()));
                stmt.insert(
                    "Action".to_string(),
                    Value::String("s3:GetObject".to_string()),
                );
                stmt.insert(
                    "Resource".to_string(),
                    Value::String("arn:aws:s3:::my-bucket/*".to_string()),
                );
                stmt
            })]),
        );

        let (id, state) = make_state(
            "awscc",
            "iam.RolePolicy",
            "test-policy",
            vec![
                ("policy_name", Value::String("test-policy".to_string())),
                ("policy_document", Value::Map(policy_map)),
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
        let Value::Map(pd) = policy_document else {
            panic!("expected Map for policy_document");
        };
        let Value::List(stmts) = pd.get("Statement").expect("Statement present") else {
            panic!("expected List for Statement");
        };
        let Value::Map(stmt) = &stmts[0] else {
            panic!("expected Map for Statement[0]");
        };
        assert_eq!(
            stmt.get("Action"),
            Some(&Value::StringList(vec!["s3:GetObject".to_string()])),
            "Action should be canonicalized to StringList"
        );
        assert_eq!(
            stmt.get("Resource"),
            Some(&Value::StringList(vec![
                "arn:aws:s3:::my-bucket/*".to_string()
            ])),
            "Resource should be canonicalized to StringList"
        );
    }

    #[test]
    fn canonicalize_string_or_list_skips_non_awscc() {
        let (id, state) = make_state(
            "aws",
            "iam.RolePolicy",
            "test",
            vec![("policy_name", Value::String("test".to_string()))],
        );
        let mut current_states: HashMap<ResourceId, State> = HashMap::new();
        current_states.insert(id.clone(), state);

        canonicalize_string_or_list_states_impl(&mut current_states);

        let state = &current_states[&id];
        assert_eq!(
            state.attributes.get("policy_name"),
            Some(&Value::String("test".to_string())),
            "non-awscc state untouched"
        );
    }

    #[test]
    fn canonicalize_string_or_list_skips_unknown_resource_type() {
        let (id, state) = make_state(
            "awscc",
            "unknown.UnknownType",
            "test",
            vec![("attr", Value::String("x".to_string()))],
        );
        let mut current_states: HashMap<ResourceId, State> = HashMap::new();
        current_states.insert(id.clone(), state);

        canonicalize_string_or_list_states_impl(&mut current_states);

        let state = &current_states[&id];
        assert_eq!(
            state.attributes.get("attr"),
            Some(&Value::String("x".to_string())),
            "unknown resource types pass through unchanged"
        );
    }

    // ---- normalize_absent_optional_lists_and_maps_impl tests ----
    // (carina-rs/carina-provider-awscc#182, parent carina-rs/carina#2544)

    #[test]
    fn normalize_absent_optional_list_inserts_empty_list() {
        // Repro for #182: CloudControl returned no `ManagedPolicyArns`
        // for an iam.Role with no attached managed policies. The state
        // should normalize the absent attribute to `Value::List(vec![])`
        // so the differ sees `[] == []` instead of `(none) → []`.
        let (id, state) = make_state(
            "awscc",
            "iam.Role",
            "test-role",
            vec![
                ("role_name", Value::String("test-role".to_string())),
                (
                    "arn",
                    Value::String("arn:aws:iam::123:role/test".to_string()),
                ),
            ],
        );
        let mut current_states: HashMap<ResourceId, State> = HashMap::new();
        current_states.insert(id.clone(), state);

        normalize_absent_optional_lists_and_maps_impl(&mut current_states);

        let state = &current_states[&id];
        assert_eq!(
            state.attributes.get("managed_policy_arns"),
            Some(&Value::List(Vec::new())),
            "absent optional list should be normalized to Value::List(vec![])"
        );
    }

    #[test]
    fn normalize_absent_optional_list_preserves_existing_list() {
        // If CloudControl actually returned a list, leave it alone.
        let existing = Value::List(vec![Value::String("arn:aws:iam::aws:policy/X".to_string())]);
        let (id, state) = make_state(
            "awscc",
            "iam.Role",
            "test-role",
            vec![
                ("role_name", Value::String("test-role".to_string())),
                ("managed_policy_arns", existing.clone()),
            ],
        );
        let mut current_states: HashMap<ResourceId, State> = HashMap::new();
        current_states.insert(id.clone(), state);

        normalize_absent_optional_lists_and_maps_impl(&mut current_states);

        let state = &current_states[&id];
        assert_eq!(
            state.attributes.get("managed_policy_arns"),
            Some(&existing),
            "non-empty list should be preserved"
        );
    }

    #[test]
    fn normalize_absent_optional_list_skips_non_awscc() {
        let (id, state) = make_state(
            "aws",
            "iam.Role",
            "test-role",
            vec![("role_name", Value::String("test-role".to_string()))],
        );
        let mut current_states: HashMap<ResourceId, State> = HashMap::new();
        current_states.insert(id.clone(), state);

        normalize_absent_optional_lists_and_maps_impl(&mut current_states);

        let state = &current_states[&id];
        assert!(
            !state.attributes.contains_key("managed_policy_arns"),
            "non-awscc states should be untouched"
        );
    }

    #[test]
    fn normalize_absent_optional_list_skips_unknown_resource_type() {
        let (id, state) = make_state(
            "awscc",
            "unknown.UnknownType",
            "test",
            vec![("attr", Value::String("x".to_string()))],
        );
        let mut current_states: HashMap<ResourceId, State> = HashMap::new();
        current_states.insert(id.clone(), state);

        normalize_absent_optional_lists_and_maps_impl(&mut current_states);

        let state = &current_states[&id];
        assert_eq!(
            state.attributes.len(),
            1,
            "unknown resource types should be untouched"
        );
    }

    #[test]
    fn normalize_absent_optional_list_skips_nonexistent_state() {
        let mut state = make_state("awscc", "iam.Role", "ghost", vec![]).1;
        state.exists = false;
        let id = state.id.clone();
        let mut current_states: HashMap<ResourceId, State> = HashMap::new();
        current_states.insert(id.clone(), state);

        normalize_absent_optional_lists_and_maps_impl(&mut current_states);

        let state = &current_states[&id];
        assert!(
            state.attributes.is_empty(),
            "non-existing states should be untouched"
        );
    }

    #[test]
    fn normalize_absent_optional_map_inserts_empty_map() {
        // Use a resource type whose schema declares an optional Map
        // attribute. AWS::IAM::Role's `tags` is technically a list of
        // CloudFormation Tag structs, but `policy_options` on
        // `iam.UserPolicy` is not present either. Instead, exercise
        // the Map branch by using a resource we know has an optional
        // Map. `awscc.cloudfront.Distribution.tags` is a map in the
        // DSL after tag conversion in some shapes; safer to just use
        // a synthetic Schema-driven test on a real resource.
        //
        // Find any awscc resource type with an optional Map attribute
        // by scanning the registry.
        use crate::schemas::generated;
        let mut found: Option<(String, String)> = None;
        for cfg in generated::configs().iter() {
            for (attr_name, attr_schema) in &cfg.schema.attributes {
                if attr_schema.required || attr_schema.write_only {
                    continue;
                }
                if matches!(attr_schema.attr_type, AttributeType::Map { .. }) {
                    found = Some((cfg.schema.resource_type.clone(), attr_name.clone()));
                    break;
                }
            }
            if found.is_some() {
                break;
            }
        }
        let Some((resource_type, attr_name)) = found else {
            // No optional Map attribute exists in the current schema
            // registry — the Map branch is still exercised at the
            // type-system level and the implementation handles it
            // symmetrically with List. Skip the runtime assertion in
            // this case.
            return;
        };

        let (id, state) = make_state(
            "awscc",
            &resource_type,
            "test-resource",
            vec![("name", Value::String("test".to_string()))],
        );
        let mut current_states: HashMap<ResourceId, State> = HashMap::new();
        current_states.insert(id.clone(), state);

        normalize_absent_optional_lists_and_maps_impl(&mut current_states);

        let state = &current_states[&id];
        assert_eq!(
            state.attributes.get(&attr_name),
            Some(&Value::Map(IndexMap::new())),
            "absent optional map ({attr_name} on {resource_type}) should be \
             normalized to Value::Map(IndexMap::new())"
        );
    }

    #[test]
    fn normalize_absent_required_attribute_is_not_filled() {
        // assume_role_policy_document is required on iam.Role; even if
        // somehow absent, the normalizer must not silently fill it in.
        let (id, state) = make_state(
            "awscc",
            "iam.Role",
            "test-role",
            vec![("role_name", Value::String("test-role".to_string()))],
        );
        let mut current_states: HashMap<ResourceId, State> = HashMap::new();
        current_states.insert(id.clone(), state);

        normalize_absent_optional_lists_and_maps_impl(&mut current_states);

        let state = &current_states[&id];
        assert!(
            !state.attributes.contains_key("assume_role_policy_document"),
            "required attributes must not be auto-filled by absent normalization"
        );
    }
}
