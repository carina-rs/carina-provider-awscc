//! Rust spelling and semantic type are produced together at each emission decision.
use super::*;
use carina_core::schema::{
    AttributeSchema, FieldGroup, ResourceSchema, StructField, ValidatedResourceSchema,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum DslTransformSpec {
    StripSuffix(&'static str),
}

impl DslTransformSpec {
    pub(super) fn rust(self) -> String {
        match self {
            Self::StripSuffix(suffix) => format!(
                "Some(carina_core::schema::DslTransform::StripSuffix({}.to_string()))",
                rust_lit(suffix)
            ),
        }
    }
    pub(super) fn value(self) -> carina_core::schema::DslTransform {
        match self {
            Self::StripSuffix(suffix) => {
                carina_core::schema::DslTransform::StripSuffix(suffix.into())
            }
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(super) struct TypeAnnotations {
    pub(super) definition: Option<String>,
    pub(super) inline_struct: bool,
    pub(super) notes: Vec<String>,
    pub(super) children: BTreeMap<String, TypeAnnotations>,
    pub(super) conditional_requirements: Vec<schema_projection::ConditionalRequirement>,
    pub(super) map_requirements:
        BTreeMap<String, Vec<Vec<schema_projection::ConditionalRequirement>>>,
    pub(super) document_kind: Option<DocumentKind>,
    pub(super) default_annotation: Option<default_plan::DefaultAnnotation>,
    pub(super) descriptions: BTreeMap<String, String>,
}

#[derive(Clone, Debug)]
pub(super) struct EmittedType {
    pub(super) rust: String,
    pub(super) ty: Box<AttributeType>,
    pub(super) annotations: Box<TypeAnnotations>,
}

impl EmittedType {
    pub(super) fn builder_call(
        &self,
        family: &str,
        constructor: &str,
        name: &str,
        separator: &str,
    ) -> String {
        let mut rust = format!("{family}::{constructor}({name}, {})", self.rust);
        if self.annotations.document_kind == Some(DocumentKind::IamPolicyDocument) {
            rust.push_str(separator);
            rust.push_str(".with_documented_iam_grammar()");
        }
        rust
    }

    pub(super) fn new(rust: impl Into<String>, ty: AttributeType) -> Self {
        Self {
            rust: rust.into(),
            ty: Box::new(ty),
            annotations: Box::default(),
        }
    }

    pub(super) fn inline_struct(rust: impl Into<String>, ty: AttributeType) -> Self {
        let mut emitted = Self::new(rust, ty);
        emitted.annotations.inline_struct = true;
        emitted
    }

    // Preserve the established block-name policy from emission decisions,
    // including struct lists nested inside an inline aggregate. Opaque named
    // constructors and references do not expose their implementation here.
    pub(super) fn contains_inline_struct_list(&self) -> bool {
        fn visit(ty: &AttributeType, info: &TypeAnnotations) -> bool {
            use carina_core::schema::RawShape;
            let child = |key: &str, ty: &AttributeType| {
                info.children.get(key).is_some_and(|child| visit(ty, child))
            };
            match ty.raw_shape() {
                RawShape::List { element_type, .. } => {
                    info.children
                        .get("item")
                        .is_some_and(|item| item.inline_struct)
                        || child("item", element_type)
                }
                RawShape::Map { key, value, .. } => child("key", key) || child("value", value),
                RawShape::Union(members) => members
                    .iter()
                    .enumerate()
                    .any(|(index, member)| child(&format!("member_{index}"), member)),
                RawShape::Struct { fields, .. } if info.inline_struct => fields
                    .iter()
                    .any(|field| child(field.name(), field.field_type())),
                _ => false,
            }
        }
        visit(&self.ty, &self.annotations)
    }

    pub(super) fn reference(name: &str) -> Self {
        Self::new(
            format!("AttributeType::ref_({}.to_string())", rust_lit(name)),
            AttributeType::ref_(name),
        )
    }

    pub(super) fn int(range: Option<(Option<i64>, Option<i64>)>) -> Self {
        let rust = range.map_or_else(
            || "AttributeType::int()".into(),
            |(min, max)| refined_int_type(&emit_int_range_option(min, max)),
        );
        Self::new(rust, AttributeType::refined_int(None, range))
    }

    pub(super) fn float(range: Option<(Option<i64>, Option<i64>)>) -> Self {
        let rust = range.map_or_else(
            || "AttributeType::float()".into(),
            |range| refined_float_type("None", Some(range)),
        );
        let range = range.map(|(min, max)| {
            let finite = |value: i64| {
                carina_core::FiniteF64::new(value as f64).expect("finite float range endpoint")
            };
            (min.map(finite), max.map(finite))
        });
        Self::new(rust, AttributeType::refined_float(None, range))
    }

    pub(super) fn string() -> Self {
        Self::new("AttributeType::string()", AttributeType::string())
    }

    pub(super) fn boolean() -> Self {
        Self::new("AttributeType::bool()", AttributeType::bool())
    }

    pub(super) fn list_of(item: &Self) -> Self {
        Self::list(
            format!("AttributeType::list({})", item.rust),
            item,
            true,
            None,
            ListElements::AllowDuplicates,
        )
    }

    pub(super) fn map_of(value: &Self) -> Self {
        Self::map(
            format!("AttributeType::map({})", value.rust),
            &Self::string(),
            value,
            None,
        )
    }

    pub(super) fn union(members: Vec<Self>) -> Self {
        let rust = format!(
            "AttributeType::union(vec![{}])",
            members
                .iter()
                .map(|member| member.rust.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        );
        let mut annotations = TypeAnnotations::default();
        for (index, member) in members.iter().enumerate() {
            annotations.children.insert(
                format!("member_{index}"),
                member.annotations.as_ref().clone(),
            );
        }
        Self {
            rust,
            ty: Box::new(AttributeType::union(
                members.into_iter().map(|member| *member.ty).collect(),
            )),
            annotations: Box::new(annotations),
        }
    }

    pub(super) fn list(
        rust: String,
        item: &Self,
        ordered: bool,
        length: Option<(Option<u64>, Option<u64>)>,
        elements: ListElements,
    ) -> Self {
        let mut emitted = Self::new(
            rust,
            AttributeType::refined_list(item.ty.as_ref().clone(), ordered, length, elements),
        );
        emitted
            .annotations
            .children
            .insert("item".into(), item.annotations.as_ref().clone());
        emitted
    }

    pub(super) fn map(
        rust: String,
        key: &Self,
        value: &Self,
        length: Option<(Option<u64>, Option<u64>)>,
    ) -> Self {
        let mut emitted = Self::new(
            rust,
            AttributeType::refined_map(key.ty.as_ref().clone(), value.ty.as_ref().clone(), length),
        );
        emitted
            .annotations
            .children
            .insert("key".into(), key.annotations.as_ref().clone());
        emitted
            .annotations
            .children
            .insert("value".into(), value.annotations.as_ref().clone());
        emitted
    }

    pub(super) fn note(mut self, note: impl Into<String>) -> Self {
        self.annotations.notes.push(note.into());
        self
    }
}

pub(super) fn length(min: Option<u64>, max: Option<u64>) -> Option<(Option<u64>, Option<u64>)> {
    let min = min.filter(|min| *min > 0);
    (min.is_some() || max.is_some()).then_some((min, max))
}

pub(super) struct SourceNotes<'a> {
    format: Option<&'a str>,
    multiple_of: Option<&'a serde_json::Number>,
    enum_values: Option<&'a [serde_json::Value]>,
}

impl<'a> SourceNotes<'a> {
    pub(super) fn string(format: Option<&'a str>) -> Self {
        Self {
            format,
            multiple_of: None,
            enum_values: None,
        }
    }
}

impl<'a> From<&'a Projection> for SourceNotes<'a> {
    fn from(projection: &'a Projection) -> Self {
        let scalar = matches!(
            projection.effective_type(),
            Some("string" | "integer" | "number")
        );
        let numeric = matches!(projection.effective_type(), Some("integer" | "number"));
        Self {
            format: scalar.then_some(projection.format.as_deref()).flatten(),
            multiple_of: numeric.then_some(projection.multiple_of.as_ref()).flatten(),
            enum_values: projection.enum_values.as_deref(),
        }
    }
}

pub(super) fn source_notes<'a>(value: &mut EmittedType, source: impl Into<SourceNotes<'a>>) {
    let source = source.into();
    // A named override's source annotations belong to its reference use node,
    // while its final scalar constraints live in the registered definition.
    fn flags(
        ty: &AttributeType,
        defs: &DefinitionRegistry,
        seen: &mut BTreeSet<String>,
    ) -> (bool, bool) {
        use carina_core::schema::RawShape;
        match ty.raw_shape() {
            RawShape::String {
                identity_layer,
                member_layer,
            } => (
                identity_layer.identity.is_some(),
                [identity_layer.pattern, member_layer.pattern]
                    .into_iter()
                    .flatten()
                    .any(|pattern| is_numeric_string_pattern(pattern.as_str())),
            ),
            RawShape::Int { identity_layer, .. } => (identity_layer.identity.is_some(), false),
            RawShape::Float { identity, .. } => (identity.is_some(), false),
            RawShape::Reference { .. } | RawShape::Enum { values: None, .. } => (true, false),
            RawShape::Ref(name) => {
                if seen.insert(name.into()) {
                    defs.definitions
                        .get(name)
                        .map_or((false, false), |definition| {
                            flags(&definition.ty, defs, seen)
                        })
                } else {
                    (false, false)
                }
            }
            RawShape::UnloadedSchemaType(_)
            | RawShape::Bool
            | RawShape::Duration
            | RawShape::Enum {
                values: Some(_), ..
            }
            | RawShape::List { .. }
            | RawShape::Map { .. }
            | RawShape::Struct { .. }
            | RawShape::Union(_) => (false, false),
        }
    }
    let (opaque, numeric_string) =
        EMITTED_TYPES.with(|defs| flags(&value.ty, &defs.borrow(), &mut BTreeSet::new()));
    if opaque && let Some(values) = source.enum_values {
        value.annotations.notes.push(format!(
            "CFN allowed values: {}",
            serde_json::to_string(values).expect("CFN JSON values are serializable")
        ));
    }
    if let Some(format) = source.format {
        value
            .annotations
            .notes
            .push(format!("CFN format: {format}"));
    }
    if let Some(divisor) = source.multiple_of {
        value
            .annotations
            .notes
            .push(format!("multiple of {divisor} (not enforced)"));
    }
    if numeric_string {
        value.annotations.notes.push("NumericString".into());
    }
}

pub(super) fn map_requirements(
    property: &SchemaProperty,
) -> Result<Vec<Vec<schema_projection::ConditionalRequirement>>> {
    let plan = emission_plan::lower(property.try_query()?)?;
    let mut notes = Vec::new();
    if let Some(map) = &plan.map {
        notes.push(plan.scope.conditional_requirements.clone());
        notes.extend(map_requirements(map.value)?);
    }
    if let Some(array) = &plan.array {
        notes.extend(map_requirements(array.items)?);
    }
    Ok(notes)
}

pub(super) fn field_group(
    group: &RequiredGroup,
    name: impl Fn(&str) -> Result<String>,
) -> Result<FieldGroup> {
    let members = group
        .members
        .iter()
        .map(|member| name(member))
        .collect::<Result<Vec<_>>>()?;
    Ok(match group.kind {
        AlternativeKind::OneOf => FieldGroup::exactly_one(members)?,
        AlternativeKind::AnyOf => FieldGroup::at_least_one(members)?,
    })
}

#[derive(Clone, Copy, Default)]
pub(super) enum Mutability {
    #[default]
    Mutable,
    CreateOnly,
    Identity,
}

#[derive(Default)]
pub(super) struct InputFacts {
    pub(super) mutability: Mutability,
    pub(super) write_only: bool,
    pub(super) block_name: Option<String>,
}

pub(super) enum AttributeMode {
    Output,
    Required(InputFacts),
    Optional {
        input: InputFacts,
        default: Option<carina_core::resource::Value>,
    },
}

pub(super) enum FieldMode {
    Output { block_name: Option<String> },
    Required(InputFacts),
    Optional(InputFacts),
}

#[derive(Clone, Default)]
pub(super) struct FieldFacts {
    pub(super) provider_name: Option<String>,
    pub(super) description: Option<String>,
}

macro_rules! described {
    ($builder:expr, $facts:expr) => {{
        let mut builder = $builder;
        if let Some(name) = &$facts.provider_name {
            builder = builder.with_provider_name(name);
        }
        if let Some(description) = &$facts.description {
            builder = builder.with_description(description);
        }
        builder
    }};
}

macro_rules! input_builder {
    ($builder:expr, $input:expr, $facts:expr) => {{
        let InputFacts {
            mutability,
            write_only,
            block_name,
        } = $input;
        let mut builder = described!($builder, $facts);
        if write_only {
            builder = builder.write_only();
        }
        if let Some(block) = block_name {
            builder = builder.with_block_name(block);
        }
        match mutability {
            Mutability::Mutable => builder.build(),
            Mutability::CreateOnly => builder.create_only().build(),
            Mutability::Identity => builder.create_only().identity().build(),
        }
    }};
}

pub(super) fn field(
    name: &str,
    ty: AttributeType,
    mode: FieldMode,
    facts: &FieldFacts,
) -> StructField {
    match mode {
        FieldMode::Output { block_name } => {
            let mut builder = described!(StructField::provider_populated(name, ty), facts);
            if let Some(block) = block_name {
                builder = builder.with_block_name(block);
            }
            builder.build()
        }
        FieldMode::Required(input) => input_builder!(StructField::required(name, ty), input, facts),
        FieldMode::Optional(input) => input_builder!(StructField::optional(name, ty), input, facts),
    }
}

pub(super) fn attribute(
    name: &str,
    ty: AttributeType,
    mode: AttributeMode,
    facts: &FieldFacts,
) -> AttributeSchema {
    match mode {
        AttributeMode::Output => {
            described!(AttributeSchema::provider_populated(name, ty), facts).build()
        }
        AttributeMode::Required(input) => {
            input_builder!(AttributeSchema::required(name, ty), input, facts)
        }
        AttributeMode::Optional { input, default } => {
            let mut builder = AttributeSchema::optional(name, ty);
            if let Some(default) = default {
                builder = builder.with_default(default);
            }
            input_builder!(builder, input, facts)
        }
    }
}

pub(super) struct ResourceEmission {
    pub(super) rust: String,
    pub(super) attributes: BTreeMap<String, EmittedType>,
    pub(super) defs: BTreeMap<String, EmittedType>,
    pub(super) witness: ValidatedResourceSchema,
}

type DefinitionBodies = BTreeMap<String, EmittedType>;
type DuplicateBodies = Vec<(String, Box<AttributeType>)>;

#[derive(Default)]
pub(super) struct DefinitionRegistry {
    definitions: DefinitionBodies,
    duplicates: DuplicateBodies,
}

impl DefinitionRegistry {
    pub(super) fn clear(&mut self) {
        *self = Self::default();
    }
    #[cfg(test)]
    pub(super) fn get_mut(&mut self, name: &str) -> Option<&mut EmittedType> {
        self.definitions.get_mut(name)
    }
    pub(super) fn contains_key(&self, name: &str) -> bool {
        self.definitions.contains_key(name)
    }
    pub(super) fn insert(&mut self, name: String, body: EmittedType) -> Option<EmittedType> {
        self.definitions.insert(name, body)
    }
    pub(super) fn register_named(&mut self, name: &str, body: EmittedType) -> Result<()> {
        if let Some(previous) = self.definitions.get(name) {
            anyhow::ensure!(
                previous.annotations == body.annotations,
                "conflicting local type definition {name}"
            );
            self.duplicates.push((name.into(), body.ty));
        } else {
            self.definitions.insert(name.into(), body);
        }
        Ok(())
    }
    pub(super) fn into_parts(self) -> (DefinitionBodies, DuplicateBodies) {
        (self.definitions, self.duplicates)
    }
    #[cfg(test)]
    pub(super) fn has_duplicates(&self) -> bool {
        !self.duplicates.is_empty()
    }
}

pub(super) fn witness(
    mut schema: ResourceSchema,
    defs: &BTreeMap<String, EmittedType>,
) -> Result<ValidatedResourceSchema> {
    for (name, body) in defs {
        schema = schema.with_def(name, body.ty.as_ref().clone());
    }
    schema.validate().context("emitted resource witness")
}

// Compare admitted typed bodies after all forward definitions are available.
// Core owns both pattern compilation and the complete structural wire view.
pub(super) fn validate_duplicate_definitions(
    mut schema: ResourceSchema,
    defs: &BTreeMap<String, EmittedType>,
    canonical: &ValidatedResourceSchema,
    duplicates: Vec<(String, Box<AttributeType>)>,
) -> Result<()> {
    if duplicates.is_empty() {
        return Ok(());
    }
    let names = duplicates
        .iter()
        .map(|(name, _)| name.as_str())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>()
        .join(", ");
    let mut comparisons = Vec::new();
    for (index, (name, body)) in duplicates.into_iter().enumerate() {
        let mut key = format!("__CodegenDuplicate{index}");
        while defs.contains_key(&key) {
            key.push('_');
        }
        schema = schema.with_def(&key, *body);
        comparisons.push((name, key));
    }
    let compared = witness(schema, defs)
        .with_context(|| format!("conflicting local type definition {names}"))?;
    for (name, key) in comparisons {
        let original = carina_core::protocol_convert::attribute_type_to_protocol(
            &canonical.parts().defs[&name],
        )?;
        let candidate = carina_core::protocol_convert::attribute_type_to_protocol(
            &compared.parts().defs[&key],
        )?;
        anyhow::ensure!(
            serde_json::to_value(original)? == serde_json::to_value(candidate)?,
            "conflicting local type definition {name}"
        );
    }
    Ok(())
}
