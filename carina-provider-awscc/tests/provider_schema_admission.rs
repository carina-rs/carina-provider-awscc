use carina_core::provider::ProviderFactory;
use carina_core::schema::ValidatedProviderSchemaWitness;
use carina_provider_awscc::{AwsccProviderFactory, schemas};

#[test]
fn every_reference_identity_has_one_definition_across_resources_and_config() {
    use carina_core::schema::{CompiledAttributeType, PatternSpec, RawShape, TypeIdentity};
    use std::collections::BTreeMap;

    type Definition = (Option<PatternSpec>, Option<(Option<u64>, Option<u64>)>);

    fn walk(
        ty: &CompiledAttributeType,
        path: &str,
        definitions: &mut BTreeMap<TypeIdentity, (Definition, String)>,
        conflicts: &mut Vec<String>,
        references: &mut usize,
    ) {
        let declaration = match ty.raw_shape() {
            RawShape::Reference {
                identity,
                pattern,
                length,
            } => {
                *references += 1;
                Some((identity, pattern, length))
            }
            RawShape::List { element_type, .. } => {
                walk(
                    element_type,
                    &format!("{path}[]"),
                    definitions,
                    conflicts,
                    references,
                );
                None
            }
            RawShape::Map { key, value, .. } => {
                walk(
                    key,
                    &format!("{path}.key"),
                    definitions,
                    conflicts,
                    references,
                );
                walk(
                    value,
                    &format!("{path}.value"),
                    definitions,
                    conflicts,
                    references,
                );
                None
            }
            RawShape::Struct { fields, .. } => {
                for field in fields.iter() {
                    walk(
                        field.field_type(),
                        &format!("{path}.{}", field.name()),
                        definitions,
                        conflicts,
                        references,
                    );
                }
                None
            }
            RawShape::Union(members) => {
                for member in members {
                    walk(member, path, definitions, conflicts, references);
                }
                None
            }
            // Every definition is visited independently below, including unused
            // definitions, so named refs need no expansion or cycle guard here.
            RawShape::Ref(_)
            | RawShape::String { .. }
            | RawShape::Enum { .. }
            | RawShape::Bool
            | RawShape::Int { .. }
            | RawShape::Float { .. }
            | RawShape::Duration => None,
            RawShape::UnloadedSchemaType(_) => panic!("unloaded schema type at {path}"),
        };
        if let Some((identity, pattern, length)) = declaration {
            let definition = (pattern.cloned(), length);
            if let Some((previous, first_path)) = definitions.get(identity) {
                if previous != &definition {
                    conflicts.push(format!(
                        "{identity}: {first_path} {previous:?} != {path} {definition:?}"
                    ));
                }
            } else {
                definitions.insert(identity.clone(), (definition, path.to_string()));
            }
        }
    }

    let mut definitions = BTreeMap::new();
    let mut conflicts = Vec::new();
    let mut references = 0;
    for resource in schemas::all_schemas() {
        let schema = resource.as_schema();
        for (name, attribute) in schema.attributes() {
            walk(
                attribute.attr_type(),
                &format!("{}.{name}", schema.resource_type()),
                &mut definitions,
                &mut conflicts,
                &mut references,
            );
        }
        for (name, ty) in schema.defs() {
            walk(
                ty,
                &format!("{}.defs.{name}", schema.resource_type()),
                &mut definitions,
                &mut conflicts,
                &mut references,
            );
        }
    }
    for (name, ty) in AwsccProviderFactory
        .provider_config_attribute_types()
        .parts()
    {
        walk(
            ty,
            &format!("config.{name}"),
            &mut definitions,
            &mut conflicts,
            &mut references,
        );
    }
    assert!(references > 0, "must inspect Reference declarations");
    assert!(
        conflicts.is_empty(),
        "conflicting identity definitions:\n{}",
        conflicts.join("\n")
    );
}

#[test]
fn every_generated_schema_has_consistent_identity_declarations() {
    let resources = schemas::all_schemas();
    assert!(!resources.is_empty(), "must admit the generated catalog");
    let count = resources.len();
    let admitted = ValidatedProviderSchemaWitness::new(
        resources,
        AwsccProviderFactory.provider_config_attribute_types(),
        Default::default(),
        schemas::assignability_edges(),
    );
    let witness = admitted.unwrap_or_else(|error| {
        panic!("all {count} generated schemas must share consistent identity layers: {error:?}")
    });
    assert_eq!(witness.resources().len(), count);
}

#[test]
fn every_generated_schema_preserves_identity_and_member_layers_over_protocol() {
    use carina_core::protocol_convert::{protocol_to_resource_schema, resource_schema_to_protocol};

    let resources = schemas::all_schemas()
        .iter()
        .map(|schema| {
            let wire = resource_schema_to_protocol(schema).unwrap();
            let json = serde_json::to_string(&wire).unwrap();
            protocol_to_resource_schema(&serde_json::from_str(&json).unwrap()).unwrap()
        })
        .collect();
    ValidatedProviderSchemaWitness::new(
        resources,
        AwsccProviderFactory.provider_config_attribute_types(),
        Default::default(),
        schemas::assignability_edges(),
    )
    .expect("transported schemas must retain one definition per identity");
}
