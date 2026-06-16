//! AWS Cloud Control resource schema definitions

pub mod config;
pub mod generated;

use carina_core::schema::ResourceSchema;

/// Returns all AWS Cloud Control schemas.
/// Auto-generated from CloudFormation schemas with per-resource operational config.
pub fn all_schemas() -> Vec<ResourceSchema> {
    generated::configs()
        .iter()
        .map(|c| c.schema.clone())
        .collect()
}

#[cfg(test)]
mod tests {
    use std::collections::{BTreeMap, BTreeSet, HashMap};

    use carina_core::differ::create_plan;
    use carina_core::effect::Effect;
    use carina_core::resource::{
        ConcreteValue, PlanInputState, Resource, ResourceId, State, Value,
    };
    use carina_core::schema::{AttributeType, RawShape, ResourceSchema, Shape, ShapeWalkBudget};
    use carina_core::schema::{SchemaKind, SchemaRegistry};
    use indexmap::IndexMap;

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
        let resource_type = schema.resource_type.clone();
        let resource_id = ResourceId::with_provider("awscc", resource_type.clone(), "test", None);
        let resources = vec![
            Resource::with_provider("awscc", resource_type, "test", None)
                .with_attribute("tags", tags_value("new-name")),
        ];

        let mut current_states: HashMap<ResourceId, PlanInputState> = HashMap::new();
        current_states.insert(
            resource_id.clone(),
            State::existing(
                resource_id.clone(),
                HashMap::from([("tags".to_string(), tags_value("old-name"))]),
            )
            .into_plan_input(),
        );

        let mut schemas = SchemaRegistry::new();
        schemas.insert("awscc", schema);
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

        let plan = create_plan(
            &resources,
            &[],
            &carina_core::provider::ProviderRouter::new(),
            &current_states,
            &HashMap::new(),
            &schemas,
            &HashMap::new(),
            &HashMap::new(),
            &HashMap::new(),
            &[],
        );

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
            Value::Concrete(ConcreteValue::String(name.to_string())),
        )])))
    }

    #[test]
    fn generated_s3_lifecycle_rule_omits_deprecated_transition_fields() {
        let config = super::generated::s3::bucket::s3_bucket_config();
        let lifecycle = config
            .schema
            .attributes
            .get("lifecycle_configuration")
            .expect("s3 bucket should expose lifecycle_configuration");
        let Shape::Struct { .. } = config.schema.shape_of(&lifecycle.attr_type) else {
            panic!("lifecycle_configuration should be a struct");
        };
        let fields = config
            .schema
            .struct_fields_with_budget(&lifecycle.attr_type, &mut ShapeWalkBudget::new(256))
            .expect("lifecycle_configuration should expose fields");
        let rules = fields
            .iter()
            .find(|field| field.name == "rules")
            .expect("lifecycle_configuration should expose rules");
        let Shape::List {
            element_type: inner,
            ..
        } = config.schema.shape_of(&rules.field_type)
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

        assert!(fields.iter().any(|field| field.name == "transitions"));
        assert!(
            fields
                .iter()
                .any(|field| field.name == "noncurrent_version_transitions")
        );
        assert!(!fields.iter().any(|field| field.name == "transition"));
        assert!(
            !fields
                .iter()
                .any(|field| field.name == "noncurrent_version_transition")
        );
        assert!(
            !fields
                .iter()
                .any(|field| field.name == "noncurrent_version_expiration_in_days")
        );
    }

    #[test]
    fn generated_string_enum_identities_are_unique_within_each_resource() {
        let mut failures = Vec::new();

        for schema in super::all_schemas() {
            let mut collector = StringEnumIdentityCollector::new();

            for (def_name, def) in &schema.defs {
                if collector.duplicates.len() >= 20 {
                    break;
                }
                collect_string_enum_identities(
                    &schema,
                    def,
                    &format!("def:{def_name}"),
                    Some(def_name),
                    &mut collector,
                    0,
                );
            }

            for (attr_name, attr) in &schema.attributes {
                if collector.duplicates.len() >= 20 {
                    break;
                }
                collect_string_enum_identities(
                    &schema,
                    &attr.attr_type,
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
                    schema.resource_type,
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
                if schema.defs.contains_key(name) && current_named_def != Some(name) {
                    collector.stack.remove(&ptr);
                    return;
                }
                for field in fields {
                    if collector.duplicates.len() >= 20 {
                        break;
                    }
                    let field_defining_site = format!("{defining_site}.{}", field.name);
                    collect_string_enum_identities(
                        schema,
                        &field.field_type,
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
