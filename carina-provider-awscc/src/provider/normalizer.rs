//! State normalization.
//!
//! This module contains standalone functions used by `ProviderNormalizer` to normalize
//! read state.

use std::collections::HashMap;

use carina_core::resource::{
    CanonicalComparisonAttributes, ConcreteValue, ResourceId, SnapshotState, Value,
};
use carina_core::schema::RawShape;

/// Apply provider-local string state transforms.
///
/// The host owns state enum canonicalization. This pass intentionally keeps
/// enum values untouched and only applies non-enum string transforms such as
/// Route53 hosted-zone trailing-dot stripping.
pub fn normalize_state_string_dsl_transforms_impl<'a, I>(current_states: I)
where
    I: IntoIterator<Item = (&'a ResourceId, &'a mut SnapshotState)>,
{
    for (resource_id, state) in current_states {
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
            if let Some(attr_schema) = config.schema.as_schema().attributes().get(key.as_str())
                && let RawShape::String {
                    to_dsl: Some(transform),
                    ..
                } = attr_schema.attr_type().raw_shape()
                && let Value::Concrete(ConcreteValue::String(s)) = value
            {
                let s = s.expose_plaintext();
                let transformed = transform.apply(s);
                if transformed != s {
                    resolved_attrs.insert(
                        key.clone(),
                        Value::Concrete(ConcreteValue::String(transformed.into_owned().into())),
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
///
/// State canonicalization also removes empty structs with declared fields,
/// including nested fields, while preserving zero-field marker structs.
pub fn canonicalize_string_or_list_states_impl<'a, I>(current_states: I)
where
    I: IntoIterator<Item = (&'a ResourceId, &'a mut SnapshotState)>,
{
    for (resource_id, state) in current_states {
        if !state.exists || resource_id.provider != "awscc" {
            continue;
        }
        let config = match crate::schemas::generated::get_config_by_type(&resource_id.resource_type)
        {
            Some(c) => c,
            None => continue,
        };
        // Canonicalize snapshot structure directly, preserving opaque secret hashes
        // and resolving cyclic CFN types against this resource's definition map.
        state.attributes = CanonicalComparisonAttributes::canonicalize(
            std::mem::take(&mut state.attributes),
            config.schema.as_schema(),
        )
        .into_iter()
        .collect();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use carina_core::provider::{
        LiftedSavedAttrs, PersistedAttributes, ProviderNormalizer, RawSavedAttrs,
    };
    use carina_core::resource::{Resource, SnapshotState as State};
    use carina_core::schema::{SchemaRegistry, ShapeWalkBudget};
    use indexmap::IndexMap;

    fn saved_attrs_from_decoded(
        saved_attrs: HashMap<ResourceId, HashMap<String, Value>>,
    ) -> LiftedSavedAttrs {
        let persisted = saved_attrs
            .into_iter()
            .map(|(id, attributes)| {
                let attributes = PersistedAttributes::from_typed_persisted_wire_values(
                    attributes
                        .into_iter()
                        .map(|(key, value)| (key, value.to_snapshot()))
                        .collect(),
                )
                .expect("test saved attributes should encode as persisted values");
                (id, attributes)
            })
            .collect();
        RawSavedAttrs::from_persisted(persisted, HashMap::new()).lift(&SchemaRegistry::new())
    }

    #[tokio::test]
    async fn normalize_desired_preserves_host_canonical_enum_strings() {
        let mut resource = Resource::with_provider("awscc", "ec2.Subnet", "test", None);
        resource.set_attr(
            "availability_zone".to_string(),
            Value::Concrete(ConcreteValue::String("ap-northeast-1a".to_string().into())),
        );
        let mut resources = vec![resource.into()];

        crate::AwsccNormalizer
            .normalize_desired(&mut resources)
            .await
            .expect("desired-state normalization should succeed");

        assert_eq!(
            resources[0].get_attr("availability_zone"),
            Some(&Value::Concrete(ConcreteValue::String(
                "ap-northeast-1a".to_string().into()
            ))),
            "host-canonical open-enum API values must not be re-namespaced"
        );
    }

    #[tokio::test]
    async fn normalize_state_keeps_api_enum_spellings_out_of_dsl_string_form() {
        let id =
            ResourceId::with_provider_identity("awscc", "ec2.SecurityGroupEgress", "test", None);
        let attrs = HashMap::from([(
            "ip_protocol".to_string(),
            Value::Concrete(ConcreteValue::String("-1".to_string().into())),
        )]);
        let mut current_states = HashMap::from([(id.clone(), State::existing(id.clone(), attrs))]);

        carina_core::provider::normalize_provider_states(
            &crate::AwsccNormalizer,
            &mut current_states,
        )
        .await
        .expect("state normalization should succeed");

        // The string-transform pass leaves the raw API string alone; the
        // following `canonicalize_string_or_list_states_impl` pass calls
        // `EffectiveAttribute::canonicalize_state`, which lifts schema-known state
        // enums to CanonicalEnum. The bug guard is that this must not become
        // a DSL-namespaced String.
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
            .as_schema()
            .attributes()
            .get("ip_protocol")
            .expect("ec2.SecurityGroupEgress.ip_protocol should exist");
        let canonical = carina_core::utils::lift_enum_leaves(
            &Value::Concrete(ConcreteValue::String("-1".to_string().into())),
            ip_protocol_attr.attr_type(),
        )
        .expect("host should lift ip_protocol to CanonicalEnum");

        let id =
            ResourceId::with_provider_identity("awscc", "ec2.SecurityGroupEgress", "test", None);
        let attrs = HashMap::from([("ip_protocol".to_string(), canonical.clone())]);
        let mut current_states = HashMap::from([(id.clone(), State::existing(id.clone(), attrs))]);

        carina_core::provider::normalize_provider_states(
            &crate::AwsccNormalizer,
            &mut current_states,
        )
        .await
        .expect("state normalization should succeed");

        assert_eq!(
            current_states[&id].attributes.get("ip_protocol"),
            Some(&canonical),
            "provider state normalization must not downgrade CanonicalEnum to String"
        );
    }

    #[tokio::test]
    async fn hydrate_read_state_leaves_unreturned_create_only_attribute_absent() {
        let id = ResourceId::with_provider_identity("awscc", "ec2.NatGateway", "test", None);
        let mut state = State::existing(id.clone(), HashMap::new());
        state.attributes.insert(
            "nat_gateway_id".to_string(),
            Value::Concrete(ConcreteValue::String("nat-123".to_string().into())),
        );

        let mut current_states = HashMap::new();
        current_states.insert(id.clone(), state);

        let mut saved = HashMap::new();
        saved.insert(
            "subnet_id".to_string(),
            Value::Concrete(ConcreteValue::String("subnet-abc".to_string().into())),
        );
        let mut saved_attrs = HashMap::new();
        saved_attrs.insert(id.clone(), saved);
        let saved_attrs = saved_attrs_from_decoded(saved_attrs);

        carina_core::provider::hydrate_provider_states(
            &crate::AwsccNormalizer,
            &mut current_states,
            saved_attrs.as_provider_saved_attrs(),
        )
        .await
        .expect("read-state hydration should succeed");

        assert!(
            !current_states[&id].attributes.contains_key("subnet_id"),
            "an omitted create-only attribute must remain absent"
        );
    }

    #[tokio::test]
    async fn hydrate_read_state_is_a_noop_for_non_awscc_state() {
        let id = ResourceId::with_provider_identity("aws", "s3.Bucket", "test", None);
        let state = State::existing(id.clone(), HashMap::new());

        let mut current_states = HashMap::new();
        current_states.insert(id.clone(), state);

        let mut saved = HashMap::new();
        saved.insert(
            "some_attr".to_string(),
            Value::Concrete(ConcreteValue::String("value".to_string().into())),
        );
        let mut saved_attrs = HashMap::new();
        saved_attrs.insert(id.clone(), saved);
        let saved_attrs = saved_attrs_from_decoded(saved_attrs);

        carina_core::provider::hydrate_provider_states(
            &crate::AwsccNormalizer,
            &mut current_states,
            saved_attrs.as_provider_saved_attrs(),
        )
        .await
        .expect("read-state hydration should succeed");

        assert!(!current_states[&id].attributes.contains_key("some_attr"));
    }

    #[tokio::test]
    async fn hydrate_read_state_keeps_returned_attribute_unchanged() {
        let id = ResourceId::with_provider_identity("awscc", "ec2.NatGateway", "test", None);
        let mut attrs = HashMap::new();
        attrs.insert(
            "subnet_id".to_string(),
            Value::Concrete(ConcreteValue::String("subnet-current".to_string().into())),
        );
        let state = State::existing(id.clone(), attrs);

        let mut current_states = HashMap::new();
        current_states.insert(id.clone(), state);

        let mut saved = HashMap::new();
        saved.insert(
            "subnet_id".to_string(),
            Value::Concrete(ConcreteValue::String("subnet-saved".to_string().into())),
        );
        let mut saved_attrs = HashMap::new();
        saved_attrs.insert(id.clone(), saved);
        let saved_attrs = saved_attrs_from_decoded(saved_attrs);

        carina_core::provider::hydrate_provider_states(
            &crate::AwsccNormalizer,
            &mut current_states,
            saved_attrs.as_provider_saved_attrs(),
        )
        .await
        .expect("read-state hydration should succeed");

        assert_eq!(
            current_states[&id].attributes.get("subnet_id"),
            Some(&Value::Concrete(ConcreteValue::String(
                "subnet-current".to_string().into()
            )))
        );
    }

    #[tokio::test]
    async fn hydrate_read_state_leaves_unreturned_non_write_only_attribute_absent() {
        let id =
            ResourceId::with_provider_identity("awscc", "ec2.SecurityGroupEgress", "test", None);
        let mut state = State::existing(id.clone(), HashMap::new());
        state.attributes.insert(
            "ip_protocol".to_string(),
            Value::Concrete(ConcreteValue::String(
                "aws.ec2.SecurityGroupEgress.IpProtocol.all"
                    .to_string()
                    .into(),
            )),
        );

        let mut current_states = HashMap::new();
        current_states.insert(id.clone(), state);

        let mut saved = HashMap::new();
        saved.insert(
            "description".to_string(),
            Value::Concrete(ConcreteValue::String(
                "Allow all outbound".to_string().into(),
            )),
        );
        let mut saved_attrs = HashMap::new();
        saved_attrs.insert(id.clone(), saved);
        let saved_attrs = saved_attrs_from_decoded(saved_attrs);

        carina_core::provider::hydrate_provider_states(
            &crate::AwsccNormalizer,
            &mut current_states,
            saved_attrs.as_provider_saved_attrs(),
        )
        .await
        .expect("read-state hydration should succeed");

        assert!(
            !current_states[&id].attributes.contains_key("description"),
            "an optional non-write-only attribute omitted by the read must remain absent"
        );
    }

    #[test]
    fn route53_hosted_zone_name_trailing_dot_has_no_diff_without_registration() {
        let desired = carina_core::resource::ResolvedResource::new(
            Resource::with_provider("awscc", "route53.HostedZone", "example", None)
                .with_attribute(
                    "name",
                    Value::Concrete(ConcreteValue::String("example.com".to_string().into())),
                )
                .into_resolved_dependency_phase_for_test(),
        );
        let mut current_states = HashMap::from([(
            desired.id.clone(),
            State::existing(
                desired.id.clone(),
                HashMap::from([(
                    "name".to_string(),
                    Value::Concrete(ConcreteValue::String("example.com.".to_string().into())),
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
                "example.com".to_string().into()
            )))
        );
        let registry = crate::schemas::schema_registry();
        let saved_attrs = RawSavedAttrs::default().lift(&registry);
        let router = crate::test_comparison_router();
        let comparison = router
            .comparison_for(&registry, &desired.id)
            .expect("HostedZone comparison context");
        let diff = comparison
            .context()
            .with_references(
                &carina_core::references::prepare_empty_for_tests(),
                &desired.id,
                |context| {
                    carina_core::differ::diff(
                        &desired,
                        current,
                        &saved_attrs,
                        None,
                        comparison,
                        context,
                    )
                },
            )
            .expect("AWSCC fixtures must not require reference resolution")
            .expect("schema-bound diff should succeed");
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
            .as_schema()
            .attributes()
            .get("ip_protocol")
            .expect("ip_protocol attribute not found");
        if let carina_core::schema::Shape::Enum {
            values: Some(values),
            ..
        } = config.schema.as_schema().shape_of(ip_protocol.attr_type())
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
            .as_schema()
            .attributes()
            .get("ip_protocol")
            .expect("ip_protocol attribute not found");
        if let carina_core::schema::Shape::Enum {
            values: Some(values),
            ..
        } = config.schema.as_schema().shape_of(ip_protocol.attr_type())
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
            .as_schema()
            .attributes()
            .get("security_group_egress")
            .expect("security_group_egress attribute not found");
        // Drill into List -> Struct -> ip_protocol field
        if let carina_core::schema::Shape::List {
            element_type: inner,
            ..
        } = config.schema.as_schema().shape_of(egress.attr_type())
        {
            if let carina_core::schema::Shape::Struct { .. } =
                config.schema.as_schema().shape_of(inner)
            {
                let fields = config
                    .schema
                    .as_schema()
                    .struct_fields_with_budget(inner, &mut ShapeWalkBudget::new(256))
                    .expect("egress struct should expose fields");
                let ip_field = fields
                    .iter()
                    .find(|f| f.name() == "ip_protocol")
                    .expect("ip_protocol field not found in egress struct");
                if let carina_core::schema::Shape::Enum {
                    values: Some(values),
                    ..
                } = config.schema.as_schema().shape_of(ip_field.field_type())
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
            .as_schema()
            .attributes()
            .get("security_group_ingress")
            .expect("security_group_ingress attribute not found");
        if let carina_core::schema::Shape::List {
            element_type: inner,
            ..
        } = config.schema.as_schema().shape_of(ingress.attr_type())
        {
            if let carina_core::schema::Shape::Struct { .. } =
                config.schema.as_schema().shape_of(inner)
            {
                let fields = config
                    .schema
                    .as_schema()
                    .struct_fields_with_budget(inner, &mut ShapeWalkBudget::new(256))
                    .expect("ingress struct should expose fields");
                let ip_field = fields
                    .iter()
                    .find(|f| f.name() == "ip_protocol")
                    .expect("ip_protocol field not found in ingress struct");
                if let carina_core::schema::Shape::Enum {
                    values: Some(values),
                    ..
                } = config.schema.as_schema().shape_of(ip_field.field_type())
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
        let identity = match carina_core::resource::ResourceIdentity::try_from(name.to_string()) {
            Ok(identity) => identity,
            Err(carina_core::resource::ResourceIdentityError::Empty) => {
                panic!("test resource identity must not be empty")
            }
        };
        let id = ResourceId::with_provider_identity(
            provider.to_string(),
            resource_type.to_string(),
            identity,
            None,
        );
        let mut attributes = HashMap::new();
        for (k, v) in attrs {
            attributes.insert(k.to_string(), v.to_snapshot());
        }
        let state = State {
            id: id.clone(),
            identifier: Some(name.to_string()),
            attributes,
            exists: true,
            dependency_bindings: Default::default(),
            partial_read: None,
        };
        (id, state)
    }

    #[tokio::test]
    async fn normalize_state_drops_empty_structs_and_preserves_empty_markers() {
        let empty = Value::Concrete(ConcreteValue::Map(IndexMap::new()));
        let (id, state) = make_state(
            "awscc",
            "s3.Bucket",
            "empty-structs",
            vec![
                ("versioning_configuration", empty.clone()),
                (
                    "logging_configuration",
                    Value::Concrete(ConcreteValue::Map(IndexMap::from([(
                        "target_object_key_format".to_string(),
                        Value::Concrete(ConcreteValue::Map(IndexMap::from([
                            ("partitioned_prefix".to_string(), empty.clone()),
                            ("simple_prefix".to_string(), empty.clone()),
                        ]))),
                    )]))),
                ),
            ],
        );
        let mut states = HashMap::from([(id.clone(), state)]);

        carina_core::provider::normalize_provider_states(&crate::AwsccNormalizer, &mut states)
            .await
            .expect("read-state normalization should succeed");

        let attributes = &states[&id].attributes;
        assert!(!attributes.contains_key("versioning_configuration"));
        let Some(Value::Concrete(ConcreteValue::Map(logging))) =
            attributes.get("logging_configuration")
        else {
            panic!("logging configuration must retain its marker field");
        };
        let Some(Value::Concrete(ConcreteValue::Map(format))) =
            logging.get("target_object_key_format")
        else {
            panic!("target object key format must retain its marker field");
        };
        assert_eq!(
            format,
            &IndexMap::from([("simple_prefix".to_string(), empty.to_snapshot())])
        );
    }

    #[test]
    fn canonicalize_string_or_list_recurses_into_iam_policy_document() {
        // AWS::IAM::RolePolicy has `policy_document` typed as
        // `iam_policy_document()`, a Struct that nests
        // Union[String, list(String)] fields like Statement[].Action.
        // Read conversion has already translated CloudFormation property
        // names to DSL keys. The provider state pass walks that host-facing
        // shape via the Struct schema and folds nested scalars to StringList.
        let mut policy_map = IndexMap::new();
        policy_map.insert(
            "statement".to_string(),
            Value::Concrete(ConcreteValue::List(vec![Value::Concrete(
                ConcreteValue::Map({
                    let mut stmt = IndexMap::new();
                    stmt.insert(
                        "effect".to_string(),
                        Value::Concrete(ConcreteValue::String("Allow".to_string().into())),
                    );
                    stmt.insert(
                        "action".to_string(),
                        Value::Concrete(ConcreteValue::String("s3:GetObject".to_string().into())),
                    );
                    stmt.insert(
                        "resource".to_string(),
                        Value::Concrete(ConcreteValue::String(
                            "arn:aws:s3:::my-bucket/*".to_string().into(),
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
                    Value::Concrete(ConcreteValue::String("test-policy".to_string().into())),
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

        // statement[0].action: scalar → StringList(["s3:GetObject"])
        let state = &current_states[&id];
        let policy_document = state
            .attributes
            .get("policy_document")
            .expect("policy_document present");
        let Value::Concrete(ConcreteValue::Map(pd)) = policy_document else {
            panic!("expected Map for policy_document");
        };
        let Value::Concrete(ConcreteValue::List(stmts)) =
            pd.get("statement").expect("statement present")
        else {
            panic!("expected List for statement");
        };
        let Value::Concrete(ConcreteValue::Map(stmt)) = &stmts[0] else {
            panic!("expected Map for statement[0]");
        };
        assert_eq!(
            stmt.get("action"),
            Some(&Value::Concrete(ConcreteValue::StringList(vec![
                "s3:GetObject".to_string().into()
            ]))),
            "action should be canonicalized to StringList"
        );
        assert_eq!(
            stmt.get("resource"),
            Some(&Value::Concrete(ConcreteValue::StringList(vec![
                "arn:aws:s3:::my-bucket/*".to_string().into()
            ]))),
            "resource should be canonicalized to StringList"
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
                Value::Concrete(ConcreteValue::String("test".to_string().into())),
            )],
        );
        let mut current_states: HashMap<ResourceId, State> = HashMap::new();
        current_states.insert(id.clone(), state);

        canonicalize_string_or_list_states_impl(&mut current_states);

        let state = &current_states[&id];
        assert_eq!(
            state.attributes.get("policy_name"),
            Some(&Value::Concrete(ConcreteValue::String(
                "test".to_string().into()
            ))),
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
                Value::Concrete(ConcreteValue::String("x".to_string().into())),
            )],
        );
        let mut current_states: HashMap<ResourceId, State> = HashMap::new();
        current_states.insert(id.clone(), state);

        canonicalize_string_or_list_states_impl(&mut current_states);

        let state = &current_states[&id];
        assert_eq!(
            state.attributes.get("attr"),
            Some(&Value::Concrete(ConcreteValue::String(
                "x".to_string().into()
            ))),
            "unknown resource types pass through unchanged"
        );
    }
}
