//! AWS Cloud Control resource schema definitions

pub mod awscc_types;
pub mod catalog;
pub mod config;
pub mod generated;

use carina_core::schema::ValidatedResourceSchema;

/// Returns all AWS Cloud Control schemas.
/// Auto-generated from CloudFormation schemas with per-resource operational config.
pub fn all_schemas() -> Vec<ValidatedResourceSchema> {
    generated::configs()
        .iter()
        .map(|config| {
            config
                .schema
                .clone()
                .validate()
                .unwrap_or_else(|error| panic!("{error}"))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use std::collections::{BTreeMap, BTreeSet, HashMap};

    use carina_core::differ::{Orphans, PlanPriorStates, create_plan};
    use carina_core::effect::Effect;
    use carina_core::provider::RawSavedAttrs;
    use carina_core::resource::{
        ConcreteValue, Current, ResolvedResource, Resource, ResourceId, State, Value,
    };
    use carina_core::schema::{AttributeType, RawShape, ResourceSchema, Shape, ShapeWalkBudget};
    use carina_core::schema::{SchemaKind, SchemaRegistry};
    use indexmap::IndexMap;

    fn nested_field_type<'a>(
        schema: &'a ResourceSchema,
        parent: &'a AttributeType,
        dsl_name: &str,
        provider_name: &str,
        required: bool,
    ) -> &'a AttributeType {
        let fields = schema
            .struct_fields_with_budget(parent, &mut ShapeWalkBudget::new(256))
            .unwrap_or_else(|| panic!("expected struct containing {dsl_name}"));
        let field = fields
            .iter()
            .find(|field| field.name() == dsl_name)
            .unwrap_or_else(|| panic!("missing struct field {dsl_name}"));
        assert_eq!(field.provider_name(), Some(provider_name));
        assert_eq!(field.parts().declared_input_mode.is_required(), required);
        assert!(
            field
                .description()
                .is_some_and(|description| !description.is_empty()),
            "{dsl_name} must have a description"
        );
        field.field_type()
    }

    fn list_element<'a>(
        schema: &'a ResourceSchema,
        attr_type: &'a AttributeType,
    ) -> &'a AttributeType {
        let Shape::List { element_type, .. } = schema.shape_of(attr_type) else {
            panic!("expected list type");
        };
        element_type
    }

    fn assert_all_nested_fields_are_described(schema: &ResourceSchema, attr_type: &AttributeType) {
        match schema.shape_of(attr_type) {
            Shape::Struct { .. } => {
                for field in schema
                    .struct_fields_with_budget(attr_type, &mut ShapeWalkBudget::new(256))
                    .expect("struct must expose fields")
                {
                    assert!(
                        field
                            .description()
                            .is_some_and(|description| !description.is_empty()),
                        "{} must have a description",
                        field.name()
                    );
                    assert_all_nested_fields_are_described(schema, field.field_type());
                }
            }
            Shape::List { element_type, .. } => {
                assert_all_nested_fields_are_described(schema, element_type);
            }
            _ => {}
        }
    }

    #[test]
    fn generated_policy_document_shapes_preserve_aws_keys_and_constraints() {
        let subscription = super::generated::sns::subscription::sns_subscription_config();
        let subscription_schema = &subscription.schema;
        assert!(
            !subscription_schema
                .attributes()
                .contains_key("filter_policy")
        );

        let redrive = subscription_schema
            .attributes()
            .get("redrive_policy")
            .expect("redrive_policy must be generated")
            .attr_type();
        let dead_letter_arn = nested_field_type(
            subscription_schema,
            redrive,
            "dead_letter_target_arn",
            "deadLetterTargetArn",
            true,
        );
        assert!(matches!(
            subscription_schema.shape_of(dead_letter_arn),
            Shape::String { .. }
        ));

        let delivery = subscription_schema
            .attributes()
            .get("delivery_policy")
            .expect("delivery_policy must be generated")
            .attr_type();
        let healthy = nested_field_type(
            subscription_schema,
            delivery,
            "healthy_retry_policy",
            "healthyRetryPolicy",
            false,
        );
        let expected_retry_fields = [
            ("min_delay_target", "minDelayTarget"),
            ("max_delay_target", "maxDelayTarget"),
            ("num_retries", "numRetries"),
            ("num_no_delay_retries", "numNoDelayRetries"),
            ("num_min_delay_retries", "numMinDelayRetries"),
            ("num_max_delay_retries", "numMaxDelayRetries"),
            ("backoff_function", "backoffFunction"),
        ];
        for (dsl_name, provider_name) in expected_retry_fields {
            nested_field_type(subscription_schema, healthy, dsl_name, provider_name, false);
        }
        let min_delay = nested_field_type(
            subscription_schema,
            healthy,
            "min_delay_target",
            "minDelayTarget",
            false,
        );
        assert!(matches!(
            subscription_schema.shape_of(min_delay),
            Shape::Int {
                range: Some((Some(1), Some(3600))),
                ..
            }
        ));
        let num_retries = nested_field_type(
            subscription_schema,
            healthy,
            "num_retries",
            "numRetries",
            false,
        );
        assert!(matches!(
            subscription_schema.shape_of(num_retries),
            Shape::Int {
                range: Some((Some(0), Some(100))),
                ..
            }
        ));
        let throttle = nested_field_type(
            subscription_schema,
            delivery,
            "throttle_policy",
            "throttlePolicy",
            false,
        );
        nested_field_type(
            subscription_schema,
            throttle,
            "max_receives_per_second",
            "maxReceivesPerSecond",
            false,
        );
        let request = nested_field_type(
            subscription_schema,
            delivery,
            "request_policy",
            "requestPolicy",
            false,
        );
        let header_content_type = nested_field_type(
            subscription_schema,
            request,
            "header_content_type",
            "headerContentType",
            false,
        );
        let Shape::Enum { values, .. } = subscription_schema.shape_of(header_content_type) else {
            panic!("header_content_type must be an enum");
        };
        assert_eq!(values.expect("closed content-type enum").len(), 12);
        nested_field_type(
            subscription_schema,
            delivery,
            "guaranteed",
            "guaranteed",
            false,
        );

        let replay = subscription_schema
            .attributes()
            .get("replay_policy")
            .expect("replay_policy must be generated")
            .attr_type();
        nested_field_type(subscription_schema, replay, "point_type", "PointType", true);
        nested_field_type(
            subscription_schema,
            replay,
            "starting_point",
            "StartingPoint",
            true,
        );
        nested_field_type(
            subscription_schema,
            replay,
            "ending_point",
            "EndingPoint",
            false,
        );

        let topic = super::generated::sns::topic::sns_topic_config();
        assert!(
            !topic
                .schema
                .attributes()
                .contains_key("data_protection_policy")
        );
        let archive = topic
            .schema
            .attributes()
            .get("archive_policy")
            .expect("archive_policy must be generated")
            .attr_type();
        let retention = nested_field_type(
            &topic.schema,
            archive,
            "message_retention_period",
            "MessageRetentionPeriod",
            true,
        );
        assert!(matches!(
            topic.schema.shape_of(retention),
            Shape::Int {
                range: Some((Some(1), Some(365))),
                ..
            }
        ));

        let logs = super::generated::logs::log_group::logs_log_group_config();
        let field_policies_attribute = logs
            .schema
            .attributes()
            .get("field_index_policies")
            .expect("field_index_policies must be generated");
        assert!(matches!(
            field_policies_attribute.default_value(),
            Some(Value::Concrete(ConcreteValue::List(items))) if items.is_empty()
        ));
        let field_policies = field_policies_attribute.attr_type();
        let Shape::List { ordered, .. } = logs.schema.shape_of(field_policies) else {
            panic!("field_index_policies must be a list");
        };
        assert!(!ordered);
        nested_field_type(
            &logs.schema,
            list_element(&logs.schema, field_policies),
            "fields",
            "Fields",
            true,
        );

        let data_policy = logs
            .schema
            .attributes()
            .get("data_protection_policy")
            .expect("data_protection_policy must be generated")
            .attr_type();
        for (dsl_name, provider_name, required) in [
            ("name", "Name", true),
            ("description", "Description", false),
            ("version", "Version", true),
            ("configuration", "Configuration", false),
            ("statement", "Statement", true),
        ] {
            nested_field_type(&logs.schema, data_policy, dsl_name, provider_name, required);
        }
        let statements =
            nested_field_type(&logs.schema, data_policy, "statement", "Statement", true);
        let statement = list_element(&logs.schema, statements);
        let operation = nested_field_type(&logs.schema, statement, "operation", "Operation", true);
        let audit = nested_field_type(&logs.schema, operation, "audit", "Audit", false);
        nested_field_type(
            &logs.schema,
            audit,
            "findings_destination",
            "FindingsDestination",
            true,
        );
        let deidentify =
            nested_field_type(&logs.schema, operation, "deidentify", "Deidentify", false);
        let mask = nested_field_type(&logs.schema, deidentify, "mask_config", "MaskConfig", true);
        assert!(
            logs.schema
                .struct_fields_with_budget(mask, &mut ShapeWalkBudget::new(256))
                .expect("mask_config must be an empty struct")
                .is_empty()
        );

        for (schema, attr_type) in [
            (subscription_schema, redrive),
            (subscription_schema, delivery),
            (subscription_schema, replay),
            (&topic.schema, archive),
            (&logs.schema, field_policies),
            (&logs.schema, data_policy),
        ] {
            assert_all_nested_fields_are_described(schema, attr_type);
        }
    }

    #[test]
    fn internet_gateway_tag_change_plans_update() {
        assert_tag_change_plans_update(
            super::generated::ec2::internet_gateway::ec2_internet_gateway_config().schema,
        );
    }

    #[test]
    fn ipam_tag_change_plans_update() {
        assert_tag_change_plans_update(super::generated::ec2::ipam::ec2_ipam_config().schema);
    }

    fn assert_tag_change_plans_update(schema: ResourceSchema) {
        let resource_type = schema.resource_type().to_string();
        let resource_id =
            ResourceId::with_provider_identity("awscc", resource_type.clone(), "test", None);
        let resource = Resource::with_provider("awscc", resource_type, "test", None)
            .with_attribute("tags", tags_value("new-name"));
        let resources = vec![resource.clone()];
        let managed = vec![ResolvedResource::new(resource)];

        let mut schemas = SchemaRegistry::new();
        schemas.insert(
            "awscc",
            schema
                .validate()
                .expect("tag-change fixture schema must be valid"),
        );
        assert!(
            schemas
                .get(
                    &resource_id.provider,
                    &resource_id.resource_type,
                    SchemaKind::Resource,
                )
                .is_some(),
            "schema must be resolvable under the key the differ uses"
        );

        let raw_states: HashMap<ResourceId, State> = HashMap::from([(
            resource_id.clone(),
            State::existing(
                resource_id.clone(),
                HashMap::from([("tags".to_string(), tags_value("old-name"))]),
            )
            .with_identifier("test-resource-id"),
        )]);
        let current_states = carina_core::resource::into_plan_input_current_map(
            raw_states
                .into_iter()
                .map(|(id, state)| (id, Current::from_state(state)))
                .collect(),
            &schemas,
            &resources,
        );
        let orphans = Orphans::new();
        let saved_attrs = RawSavedAttrs::default().lift(&schemas);

        let plan = create_plan(
            &managed,
            &[],
            &carina_core::provider::ProviderRouter::new(),
            PlanPriorStates::new(&current_states, &orphans),
            &HashMap::new(),
            &schemas,
            &saved_attrs,
            &HashMap::new(),
            &[],
        )
        .expect("tag-change planning should succeed");

        assert_eq!(plan.effects().len(), 1);
        assert!(
            matches!(plan.effects()[0], Effect::Update { .. }),
            "expected tag-only change to plan Update, got {:?}",
            plan.effects()[0]
        );
    }

    fn tags_value(name: &str) -> Value {
        Value::Concrete(ConcreteValue::Map(IndexMap::from([(
            "Name".to_string(),
            Value::Concrete(ConcreteValue::String(name.to_string().into())),
        )])))
    }

    #[test]
    fn generated_s3_lifecycle_rule_omits_deprecated_transition_fields() {
        let config = super::generated::s3::bucket::s3_bucket_config();
        let lifecycle = config
            .schema
            .attributes()
            .get("lifecycle_configuration")
            .expect("s3 bucket should expose lifecycle_configuration");
        let Shape::Struct { .. } = config.schema.shape_of(lifecycle.attr_type()) else {
            panic!("lifecycle_configuration should be a struct");
        };
        let fields = config
            .schema
            .struct_fields_with_budget(lifecycle.attr_type(), &mut ShapeWalkBudget::new(256))
            .expect("lifecycle_configuration should expose fields");
        let rules = fields
            .iter()
            .find(|field| field.name() == "rules")
            .expect("lifecycle_configuration should expose rules");
        let Shape::List {
            element_type: inner,
            ..
        } = config.schema.shape_of(rules.field_type())
        else {
            panic!("rules should be a list");
        };
        let Shape::Struct { .. } = config.schema.shape_of(inner) else {
            panic!("rules should contain lifecycle rule structs");
        };
        let fields = config
            .schema
            .struct_fields_with_budget(inner, &mut ShapeWalkBudget::new(256))
            .expect("rules should expose fields");

        assert!(fields.iter().any(|field| field.name() == "transitions"));
        assert!(
            fields
                .iter()
                .any(|field| field.name() == "noncurrent_version_transitions")
        );
        assert!(!fields.iter().any(|field| field.name() == "transition"));
        assert!(
            !fields
                .iter()
                .any(|field| field.name() == "noncurrent_version_transition")
        );
        assert!(
            !fields
                .iter()
                .any(|field| field.name() == "noncurrent_version_expiration_in_days")
        );
    }

    // CreateDistribution returns DomainName synchronously; see carina-rs/carina#3940.
    #[test]
    fn generated_cloudfront_distribution_domain_name_is_not_deferred_populate() {
        let config = super::generated::cloudfront::distribution::cloudfront_distribution_config();
        let domain_name = config
            .schema
            .attributes()
            .get("domain_name")
            .expect("CloudFront distribution should expose domain_name");

        assert!(!domain_name.is_deferred_populate());
    }

    #[test]
    fn generated_integer_enum_validators_preserve_diagnostics() {
        let cases = [
            (
                super::generated::ec2::flow_log::ec2_flow_log_config().schema,
                "max_aggregation_interval",
                61,
                "aws.ec2.FlowLog.MaxAggregationInterval",
                "Validation failed: Value 61 is not a valid value",
            ),
            (
                super::generated::logs::log_group::logs_log_group_config().schema,
                "retention_in_days",
                2,
                "aws.logs.LogGroup.RetentionInDays",
                "Validation failed: Value 2 is not a valid value",
            ),
        ];

        for (schema, attribute_name, invalid, expected_identity, expected_error) in cases {
            let schema = schema
                .validate()
                .expect("generated integer-enum schema must pass admission");
            let attribute = schema
                .effective_attribute(attribute_name)
                .unwrap_or_else(|| panic!("missing generated attribute {attribute_name}"));
            match attribute.field_type().raw_shape() {
                RawShape::Int {
                    identity: Some(identity),
                    ..
                } => assert_eq!(identity.to_string(), expected_identity),
                other => panic!("{attribute_name} must be an identity-bound Int, was {other:?}"),
            }

            let error = attribute
                .validate(&Value::Concrete(ConcreteValue::Int(invalid)))
                .expect_err("an undeclared integer-enum value must fail validation")
                .to_string();

            assert_eq!(error, expected_error);

            println!("{attribute_name}: {error}");
        }
    }

    #[test]
    fn generated_string_enum_identities_are_unique_within_each_resource() {
        let mut failures = Vec::new();

        for validated_schema in super::all_schemas() {
            let schema = validated_schema.as_schema();
            let mut collector = StringEnumIdentityCollector::new();

            for (def_name, def) in schema.defs() {
                if collector.duplicates.len() >= 20 {
                    break;
                }
                collect_string_enum_identities(
                    schema,
                    def,
                    &format!("def:{def_name}"),
                    Some(def_name),
                    &mut collector,
                    0,
                );
            }

            for (attr_name, attr) in schema.attributes() {
                if collector.duplicates.len() >= 20 {
                    break;
                }
                collect_string_enum_identities(
                    schema,
                    attr.attr_type(),
                    &format!("inline:{attr_name}"),
                    None,
                    &mut collector,
                    0,
                );
            }

            if !collector.duplicates.is_empty() {
                if collector.duplicates.len() == 20 {
                    collector
                        .duplicates
                        .push("... more duplicates omitted".to_string());
                }
                failures.push(format!(
                    "{}:\n  {}",
                    schema.resource_type(),
                    collector.duplicates.join("\n  ")
                ));
            }
        }

        assert!(
            failures.is_empty(),
            "duplicate generated StringEnum identities:\n{}",
            failures.join("\n")
        );
    }

    struct StringEnumIdentityCollector {
        first_occurrence_by_identity: BTreeMap<String, EnumIdentityOccurrence>,
        duplicates: Vec<String>,
        stack: BTreeSet<usize>,
    }

    impl StringEnumIdentityCollector {
        fn new() -> Self {
            Self {
                first_occurrence_by_identity: BTreeMap::new(),
                duplicates: Vec::new(),
                stack: BTreeSet::new(),
            }
        }
    }

    #[derive(Clone)]
    struct EnumIdentityOccurrence {
        defining_site: String,
    }

    fn collect_string_enum_identities(
        schema: &ResourceSchema,
        ty: &AttributeType,
        defining_site: &str,
        current_named_def: Option<&str>,
        collector: &mut StringEnumIdentityCollector,
        depth: usize,
    ) {
        if collector.duplicates.len() >= 20 {
            return;
        }
        if depth > 80 {
            return;
        }

        let ptr = ty as *const AttributeType as usize;
        if !collector.stack.insert(ptr) {
            return;
        }

        match ty.raw_shape() {
            RawShape::Enum {
                identity,
                values: Some(_),
                ..
            } => {
                let identity = identity.to_string();
                if !identity.starts_with("aws.") {
                    collector.stack.remove(&ptr);
                    return;
                }
                // This invariant catches distinct generated enum fields that collapse
                // to one identity. These IAM identities come from the single
                // hand-written iam_policy_document() helper and are intentionally
                // reused across all policy-document fields.
                if identity == "aws.iam.PolicyDocument.Version"
                    || identity == "aws.iam.PolicyDocument.Statement.Effect"
                {
                    collector.stack.remove(&ptr);
                    return;
                }
                let occurrence = EnumIdentityOccurrence {
                    defining_site: defining_site.to_string(),
                };
                if let Some(first) = collector
                    .first_occurrence_by_identity
                    .get(identity.as_str())
                {
                    if first.defining_site != occurrence.defining_site {
                        collector.duplicates.push(format!(
                            "{identity}: {} and {}",
                            first.defining_site, occurrence.defining_site
                        ));
                    }
                } else {
                    collector
                        .first_occurrence_by_identity
                        .insert(identity, occurrence);
                }
            }
            RawShape::Enum {
                values: None, base, ..
            } => {
                collect_string_enum_identities(
                    schema,
                    base,
                    defining_site,
                    current_named_def,
                    collector,
                    depth + 1,
                );
            }
            RawShape::List {
                element_type: inner,
                ..
            } => {
                let inner_defining_site = format!("{defining_site}[]");
                collect_string_enum_identities(
                    schema,
                    inner,
                    &inner_defining_site,
                    current_named_def,
                    collector,
                    depth + 1,
                );
            }
            RawShape::Map { value, .. } => {
                let value_defining_site = format!("{defining_site}.*");
                collect_string_enum_identities(
                    schema,
                    value,
                    &value_defining_site,
                    current_named_def,
                    collector,
                    depth + 1,
                );
            }
            RawShape::Struct { name, fields } => {
                if schema.defs().contains_key(name) && current_named_def != Some(name) {
                    collector.stack.remove(&ptr);
                    return;
                }
                for field in fields {
                    if collector.duplicates.len() >= 20 {
                        break;
                    }
                    let field_defining_site = format!("{defining_site}.{}", field.name());
                    collect_string_enum_identities(
                        schema,
                        field.field_type(),
                        &field_defining_site,
                        current_named_def,
                        collector,
                        depth + 1,
                    );
                }
            }
            RawShape::Union(members) => {
                for (index, member) in members.iter().enumerate() {
                    if collector.duplicates.len() >= 20 {
                        break;
                    }
                    let member_defining_site = format!("{defining_site}|{index}");
                    collect_string_enum_identities(
                        schema,
                        member,
                        &member_defining_site,
                        current_named_def,
                        collector,
                        depth + 1,
                    );
                }
            }
            RawShape::Ref(_) => {}
            RawShape::String { .. }
            | RawShape::Int { .. }
            | RawShape::Float { .. }
            | RawShape::Bool
            | RawShape::Duration => {}
        }

        collector.stack.remove(&ptr);
    }
}
