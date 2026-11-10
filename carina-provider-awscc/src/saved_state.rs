//! Saved-state reconstruction has a different contract from input validation.
//!
//! Only enum identities and container shapes are relevant here. In particular,
//! selecting a union arm must not enforce today's required fields, field groups,
//! scalar validators or collection bounds on an older saved value.

use std::collections::BTreeMap;

use carina_core::provider::{LiftedSavedAttrs, RawSavedAttrs};
use carina_core::schema::{
    AttributeSchema, AttributeType, Compiled, ListElements, PatternAttributeType, RawShape,
    ResourceSchema, SchemaKind, SchemaRegistry, StructField, ValidatedConfigSchema,
    ValidatedProviderSchemaWitness,
};

/// The private registry can only be produced by erasing input constraints.
/// Keep this type private to decoding; it must never be used to check new input.
pub(crate) struct SavedStateSchemas(SchemaRegistry);

impl SavedStateSchemas {
    pub(crate) fn from_input_schemas(input: &SchemaRegistry) -> Self {
        let mut resources = BTreeMap::<_, Vec<_>>::new();
        for (provider, name, kind, schema) in input.iter() {
            let parts = schema.as_schema().parts();
            let mut shape = ResourceSchema::new(name);
            if kind == SchemaKind::DataSource {
                shape = shape.as_data_source();
            }
            for (name, definition) in parts.defs {
                shape = shape.with_def(name, saved_shape(definition));
            }
            for (name, attribute) in parts.attributes {
                shape = shape.attribute(
                    AttributeSchema::optional(name, saved_shape(attribute.attr_type())).build(),
                );
            }
            resources.entry(provider).or_default().push(
                shape
                    .validate()
                    .expect("saved-state structural schema is valid"),
            );
        }
        let mut registry = SchemaRegistry::new();
        for (provider, resources) in resources {
            let witness = ValidatedProviderSchemaWitness::new(
                resources,
                ValidatedConfigSchema::default(),
                Default::default(),
                Vec::new(),
            )
            .expect("saved-state shapes retain coherent enum identities");
            registry
                .insert_provider_witness(provider, &witness)
                .expect("one structural witness per provider");
        }
        Self(registry)
    }

    pub(crate) fn lift(&self, saved: RawSavedAttrs) -> LiftedSavedAttrs {
        saved.lift(&self.0)
    }
}

fn saved_shape(input: &PatternAttributeType<Compiled>) -> AttributeType {
    match input.raw_shape() {
        // Names and validators on scalar inputs are irrelevant to reconstruction.
        RawShape::UnloadedSchemaType(_) | RawShape::Reference { .. } | RawShape::String { .. } => {
            AttributeType::string()
        }
        RawShape::Int { .. } => AttributeType::int(),
        RawShape::Float { .. } => AttributeType::float(),
        RawShape::Bool => AttributeType::bool(),
        RawShape::Duration => AttributeType::duration(),
        RawShape::Enum { identity, base, .. } => {
            // Membership is open at this boundary. A future wire spelling
            // must not prevent reconstruction of the surrounding container.
            // Persisted text is already provider spelling; DSL aliases do not
            // participate in decoding it. Keep the identity for enum lifting.
            AttributeType::enum_with_base(
                identity.clone(),
                saved_shape(base),
                None,
                Vec::new(),
                Some(std::sync::Arc::new(|_| Ok(()))),
                None,
            )
        }
        RawShape::List {
            element_type,
            ordered,
            ..
        } => AttributeType::refined_list(
            saved_shape(element_type),
            ordered,
            None,
            ListElements::AllowDuplicates,
        ),
        RawShape::Map { key, value, .. } => {
            AttributeType::map_with_key(saved_shape(key), saved_shape(value))
        }
        RawShape::Struct { name, fields, .. } => AttributeType::struct_(
            name,
            fields
                .iter()
                .map(|field| {
                    StructField::optional(field.name(), saved_shape(field.field_type())).build()
                })
                .collect(),
        ),
        RawShape::Union(members) => AttributeType::union(members.iter().map(saved_shape).collect()),
        RawShape::Ref(name) => AttributeType::ref_(name),
    }
}

#[cfg(test)]
mod tests {
    use super::super::{decode_persisted_saved_attrs, proto};
    use super::*;
    use carina_core::resource::{ConcreteValue, ResourceId, Value};
    use std::collections::HashMap;

    #[test]
    fn input_narrowings_cannot_gate_saved_union_reconstruction() {
        let strings = AttributeType::union(vec![
            AttributeType::string(),
            AttributeType::refined_list(
                AttributeType::string(),
                false,
                Some((Some(2), Some(2))),
                ListElements::Unique,
            ),
        ]);
        let statement = AttributeType::struct_with_field_groups(
            "Statement",
            vec![
                StructField::required(
                    "effect",
                    AttributeType::enum_(
                        carina_core::schema::enum_identity("Effect", Some("aws.test.Saved")),
                        Some(vec!["Allow".into()]),
                        Vec::new(),
                        None,
                        None,
                    ),
                )
                .build(),
                StructField::optional("action", strings).build(),
                StructField::optional("not_action", AttributeType::string()).build(),
            ],
            vec![carina_core::schema::FieldGroup::exactly_one(["action", "not_action"]).unwrap()],
        );
        let input = ResourceSchema::new("test.Saved")
            .attribute(
                AttributeSchema::required(
                    "policy",
                    AttributeType::union(vec![
                        AttributeType::refined_list(
                            statement.clone(),
                            true,
                            Some((Some(2), Some(3))),
                            ListElements::Unique,
                        ),
                        statement,
                    ]),
                )
                .build(),
            )
            .validate()
            .unwrap();
        let mut registry = SchemaRegistry::new();
        registry.insert("awscc", input);
        let id = ResourceId::with_provider_identity("awscc", "test.Saved", "old", None);
        let key = id.to_string();
        for effect in [None, Some("FutureEffect"), Some("Allow")] {
            let mut statement = BTreeMap::from([(
                "action".into(),
                proto::PersistedValue::StringList(vec!["old:Action".into()]),
            )]);
            if let Some(effect) = effect {
                statement.insert(
                    "effect".into(),
                    proto::PersistedValue::String(effect.into()),
                );
            }
            let wire = BTreeMap::from([(
                "policy".into(),
                proto::PersistedValue::List(vec![proto::PersistedValue::Map(statement)]),
            )]);
            let saved = HashMap::from([(key.clone(), wire)]);
            let decoded = decode_persisted_saved_attrs(
                &saved,
                &HashMap::from([(key.as_str(), id.clone())]),
                &registry,
            )
            .unwrap();
            let attrs = decoded.as_provider_saved_attrs().get(&id).unwrap();
            let Value::Concrete(ConcreteValue::List(statements)) = &attrs["policy"] else {
                panic!("saved statement list")
            };
            let Value::Concrete(ConcreteValue::Map(statement)) = &statements[0] else {
                panic!("saved statement object")
            };
            assert!(
                matches!(&statement["action"], Value::Concrete(ConcreteValue::StringList(values)) if values == &["old:Action"])
            );
            if effect == Some("Allow") {
                assert!(matches!(
                    &statement["effect"],
                    Value::Concrete(ConcreteValue::CanonicalEnum(_))
                ));
            }
        }
    }
}
