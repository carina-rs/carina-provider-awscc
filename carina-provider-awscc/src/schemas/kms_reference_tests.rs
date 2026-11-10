//! Coverage of the final registry, including generated defs and list element structs.

#[path = "../bin/codegen/kms_references.rs"]
#[allow(dead_code)]
mod classification;
use carina_core::schema::{
    CompiledAttributeType, CompiledResourceSchema, Shape, ShapeWalkBudget, StringIdentityParts,
};
use classification::{
    CLASSIFICATIONS, Classification, NotReference, PathSegment, classify_string, property_path,
};
use std::collections::BTreeSet;

#[derive(Default)]
struct Coverage {
    references: BTreeSet<(String, String)>,
    ancestors: BTreeSet<usize>,
}

impl Coverage {
    #[allow(clippy::too_many_arguments)]
    fn walk(
        &mut self,
        schema: &CompiledResourceSchema,
        ty: &CompiledAttributeType,
        path: &str,
        name: &str,
        description: Option<&str>,
        read_only: bool,
    ) {
        let key = (schema.resource_type().to_string(), path.to_string());
        let pointer = ty as *const CompiledAttributeType as usize;
        if !self.ancestors.insert(pointer) {
            return;
        }
        let mut check = |pattern: Option<&carina_core::schema::PatternSpec>,
                         identity: Option<&carina_core::schema::TypeIdentity>,
                         length| {
            if read_only {
                return;
            }
            match classify_string(
                schema.resource_type(),
                path,
                name,
                pattern.map(carina_core::schema::PatternSpec::as_str),
                description,
                [],
            )
            .unwrap()
            {
                Some(Classification::Reference(kind)) => {
                    assert_eq!(
                        identity,
                        Some(&carina_aws_types::AwsType::KmsKeyReference.identity()),
                        "{}.{path} must be a reference leaf",
                        schema.resource_type()
                    );
                    let entry = classification::entry(schema.resource_type(), path).unwrap();
                    let expected_pattern = entry.cfn_pattern.map_or_else(
                        || kind.default_pattern().clone(),
                        |pattern| {
                            carina_core::schema::PatternSpec::new(
                                pattern.replace(r"\Z", r"\z"),
                                carina_core::schema::PatternMatch::Full,
                            )
                        },
                    );
                    assert_eq!(
                        pattern,
                        Some(&expected_pattern),
                        "{}.{path}: wrong KMS reference pattern",
                        schema.resource_type()
                    );
                    assert_eq!(
                        length,
                        entry.cfn_length.or_else(|| kind.default_length()),
                        "{}.{path}: wrong KMS reference length",
                        schema.resource_type()
                    );
                    self.references.insert(key.clone());
                }
                Some(Classification::NotReference(NotReference::Other(reason))) => {
                    assert!(!reason.is_empty());
                    assert!(identity.is_none());
                }
                None => assert_ne!(
                    identity,
                    Some(&carina_aws_types::AwsType::KmsKeyReference.identity()),
                    "unclassified KMS reference: {}.{path}",
                    schema.resource_type()
                ),
            }
        };
        match schema.shape_of(ty) {
            Shape::Reference {
                identity,
                pattern,
                length,
                ..
            } => check(pattern, Some(identity), length),
            Shape::String {
                identity_layer:
                    StringIdentityParts {
                        pattern, length, ..
                    },
                ..
            } => check(pattern, None, length),
            Shape::Enum { base, .. } => self.walk(schema, base, path, name, description, read_only),
            Shape::List { element_type, .. } => self.walk(
                schema,
                element_type,
                &property_path(path, PathSegment::ListElement),
                name,
                description,
                read_only,
            ),
            Shape::Map { value, .. } => self.walk(
                schema,
                value,
                &property_path(path, PathSegment::MapValue),
                name,
                description,
                read_only,
            ),
            Shape::Struct { .. } => {
                for field in schema
                    .struct_fields_with_budget(ty, &mut ShapeWalkBudget::new(256))
                    .unwrap()
                    .iter()
                {
                    self.walk(
                        schema,
                        field.field_type(),
                        &property_path(path, PathSegment::Field(field.name())),
                        field.name(),
                        field.description(),
                        read_only || field.parts().declared_input_mode.is_provider_populated(),
                    );
                }
            }
            Shape::Union => {
                for member in schema
                    .union_members_with_budget(ty, &mut ShapeWalkBudget::new(256))
                    .unwrap()
                    .iter()
                    .flatten()
                {
                    self.walk(schema, member.as_attr(), path, name, description, read_only);
                }
            }
            Shape::UnloadedSchemaType(unloaded) => {
                panic!("KMS coverage requires loaded type {}", unloaded.identity())
            }
            Shape::Bool | Shape::Int { .. } | Shape::Float { .. } | Shape::Duration => {}
        }
        self.ancestors.remove(&pointer);
    }
}

#[test]
fn final_registry_classifies_every_kms_input_and_preserves_reference_patterns() {
    let registry = super::schema_registry();
    let mut coverage = Coverage::default();
    for (_, _, _, schema) in registry.iter() {
        let schema = schema.as_schema();
        for (name, attribute) in schema.attributes() {
            coverage.walk(
                schema,
                attribute.attr_type(),
                name,
                name,
                attribute.description(),
                attribute.is_read_only(),
            );
        }
    }
    let mut unique = BTreeSet::new();
    for entry in CLASSIFICATIONS {
        let key = (entry.resource.to_string(), entry.path.to_string());
        assert!(
            unique.insert(key.clone()),
            "duplicate KMS classification: {key:?}"
        );
    }
    let expected = CLASSIFICATIONS
        .iter()
        .filter(|entry| matches!(entry.classification, Classification::Reference(_)))
        .map(|entry| (entry.resource.to_string(), entry.path.to_string()))
        .collect();
    assert_eq!(
        coverage.references, expected,
        "every classified reference must be present in the final registry"
    );
}

#[test]
fn kms_key_outputs_keep_their_output_types() {
    let config = super::generated::kms::key::kms_key_config();
    let schema = config.schema.as_schema();
    let arn = schema.attributes().get("arn").unwrap();
    let key_id = schema.attributes().get("key_id").unwrap();
    assert!(arn.is_read_only() && key_id.is_read_only());
    assert!(
        matches!(schema.shape_of(arn.attr_type()), Shape::String { identity_layer: StringIdentityParts { identity: Some(identity), .. }, .. } if identity == &carina_aws_types::AwsType::KmsKeyArn.identity())
    );
    assert!(matches!(
        schema.shape_of(key_id.attr_type()),
        Shape::String {
            identity_layer: StringIdentityParts { identity: None, .. },
            ..
        }
    ));
}
