//! State normalization and state hydration.
//!
//! This module contains standalone functions used by `ProviderNormalizer` to normalize
//! read state and restore unreturned attributes from saved state.

use std::collections::HashMap;

use carina_core::resource::{ConcreteValue, ResourceId, State, Value};
use carina_core::schema::RawShape;

/// Apply provider-local string state transforms.
///
/// The host owns state enum canonicalization. This pass intentionally keeps
/// enum values untouched and only applies non-enum string transforms such as
/// Route53 hosted-zone trailing-dot stripping.
pub fn normalize_state_string_dsl_transforms_impl(current_states: &mut HashMap<ResourceId, State>) {
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
                && let RawShape::String {
                    to_dsl: Some(transform),
                    ..
                } = attr_schema.attr_type.raw_shape()
                && let Value::Concrete(ConcreteValue::String(s)) = value
            {
                let transformed = transform.apply(s);
                if transformed != s.as_str() {
                    resolved_attrs.insert(
                        key.clone(),
                        Value::Concrete(ConcreteValue::String(transformed.into_owned())),
                    );
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
    use carina_core::provider::ProviderNormalizer;
    use carina_core::resource::Resource;
    use carina_core::schema::ShapeWalkBudget;
    use indexmap::IndexMap;

    #[tokio::test]
    async fn normalize_desired_preserves_host_canonical_enum_strings() {
        let mut resource = Resource::with_provider("awscc", "ec2.Subnet", "test", None);
        resource.set_attr(
            "availability_zone".to_string(),
            Value::Concrete(ConcreteValue::String("ap-northeast-1a".to_string())),
        );
        let mut resources = vec![resource];

        crate::AwsccNormalizer
            .normalize_desired(&mut resources)
            .await;

        assert_eq!(
            resources[0].get_attr("availability_zone"),
            Some(&Value::Concrete(ConcreteValue::String(
                "ap-northeast-1a".to_string()
            ))),
            "host-canonical open-enum API values must not be re-namespaced"
        );
    }

    #[tokio::test]
    async fn normalize_state_keeps_api_enum_spellings_out_of_dsl_string_form() {
        let id = ResourceId::with_provider("awscc", "ec2.SecurityGroupEgress", "test", None);
        let attrs = HashMap::from([(
            "ip_protocol".to_string(),
            Value::Concrete(ConcreteValue::String("-1".to_string())),
        )]);
        let mut current_states = HashMap::from([(id.clone(), State::existing(id.clone(), attrs))]);

        crate::AwsccNormalizer
            .normalize_state(&mut current_states)
            .await;

        // The string-transform pass leaves the raw API string alone; the
        // following `canonicalize_string_or_list_states_impl` pass calls
        // `Schema::canonicalize_attr`, which host-lifts schema-known enums
        // to CanonicalEnum. The bug guard is that this must not become a
        // DSL-namespaced String.
        match current_states[&id].attributes.get("ip_protocol") {
            Some(Value::Concrete(ConcreteValue::CanonicalEnum(c))) => {
                assert_eq!(
                    c.identity().to_string(),
                    "aws.ec2.SecurityGroupEgress.IpProtocol"
                );
                assert_eq!(c.api_value(), "-1");
            }
            Some(Value::Concrete(ConcreteValue::String(s))) => {
                assert_ne!(s, "aws.ec2.SecurityGroupEgress.IpProtocol.all");
                assert_eq!(s, "-1");
            }
            other => panic!("expected CanonicalEnum(-1) or raw String(-1), got {other:?}"),
        }
    }

    #[tokio::test]
    async fn normalize_state_does_not_downgrade_canonical_enum_values() {
        let config = crate::schemas::generated::get_config_by_type("ec2.SecurityGroupEgress")
            .expect("ec2.SecurityGroupEgress schema should exist");
        let ip_protocol_attr = config
            .schema
            .attributes
            .get("ip_protocol")
            .expect("ec2.SecurityGroupEgress.ip_protocol should exist");
        let canonical = carina_core::utils::lift_enum_leaves(
            &Value::Concrete(ConcreteValue::String("-1".to_string())),
            &ip_protocol_attr.attr_type,
        )
        .expect("host should lift ip_protocol to CanonicalEnum");

        let id = ResourceId::with_provider("awscc", "ec2.SecurityGroupEgress", "test", None);
        let attrs = HashMap::from([("ip_protocol".to_string(), canonical.clone())]);
        let mut current_states = HashMap::from([(id.clone(), State::existing(id.clone(), attrs))]);

        crate::AwsccNormalizer
            .normalize_state(&mut current_states)
            .await;

        assert_eq!(
            current_states[&id].attributes.get("ip_protocol"),
            Some(&canonical),
            "provider state normalization must not downgrade CanonicalEnum to String"
        );
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
                "aws.ec2.SecurityGroupEgress.IpProtocol.all".to_string(),
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

        normalize_state_string_dsl_transforms_impl(&mut current_states);

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

    // --- ip_protocol enum canonical values and aliases ---

    #[test]
    fn test_security_group_egress_schema_keeps_all_as_alias_only() {
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
                values.contains(&"-1".to_string()) && !values.contains(&"all".to_string()),
                "enum values must include '-1' and exclude alias 'all': {:?}",
                values
            );
        } else {
            panic!("ip_protocol should be Enum");
        }
    }

    #[test]
    fn test_security_group_ingress_schema_keeps_all_as_alias_only() {
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
                values.contains(&"-1".to_string()) && !values.contains(&"all".to_string()),
                "enum values must include '-1' and exclude alias 'all': {:?}",
                values
            );
        } else {
            panic!("ip_protocol should be Enum");
        }
    }

    #[test]
    fn test_security_group_egress_struct_schema_keeps_all_as_alias_only() {
        let config = crate::schemas::generated::ec2::security_group::ec2_security_group_config();
        let egress = config
            .schema
            .attributes
            .get("security_group_egress")
            .expect("security_group_egress attribute not found");
        // Drill into List -> Struct -> ip_protocol field
        if let carina_core::schema::Shape::List {
            element_type: inner,
            ..
        } = config.schema.shape_of(&egress.attr_type)
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
                        values.contains(&"-1".to_string()) && !values.contains(&"all".to_string()),
                        "enum values must include '-1' and exclude alias 'all': {:?}",
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
    fn test_security_group_ingress_struct_schema_keeps_all_as_alias_only() {
        let config = crate::schemas::generated::ec2::security_group::ec2_security_group_config();
        let ingress = config
            .schema
            .attributes
            .get("security_group_ingress")
            .expect("security_group_ingress attribute not found");
        if let carina_core::schema::Shape::List {
            element_type: inner,
            ..
        } = config.schema.shape_of(&ingress.attr_type)
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
                        values.contains(&"-1".to_string()) && !values.contains(&"all".to_string()),
                        "enum values must include '-1' and exclude alias 'all': {:?}",
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
            partial_read: None,
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
}
