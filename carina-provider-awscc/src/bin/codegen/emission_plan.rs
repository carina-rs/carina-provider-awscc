//! The only Projection-to-emission lowering boundary, shared by Rust and Markdown.
//!
//! Each emitted kind destructures the entire Projection without a rest pattern.
//! Adding a Projection field must fail compilation in EVERY kind until its fact
//! is carried to an emitter, proved irrelevant/default, or rejected here.
//! Source names, annotations, references, union selections and conjunction
//! evidence serve projection validation and the outer provenance/naming traversal;
//! they impose no independent value constraint at this boundary.
//! Constructors, inferred literal domains and collection displays receive these
//! per-kind plans. A replacement must prove it implies every decided constraint;
//! unknown implications fail with SchemaError rather than dropping a refinement.
//! Type-specific keywords do not apply outside their JSON instance types
//! (JSON Schema draft 7, sections 3.2.1 and 6.2 through 6.5). Each arm names
//! those facts explicitly. JsonDocument includes strings and objects, so both
//! families still apply there. CFN arrayType is a presentation annotation;
//! insertionOrder only describes arrays. Non-string/non-numeric format is
//! annotation-only; numeric formats remain informational in NumericPlan.
//! Explicit non-object schemas declaring properties are rejected as mixed shapes.
//! Keyword applicability is checked before discarding a fact: oneOf over
//! vacuously true object requirements is unsatisfiable on non-object values.

use super::SchemaProperty;
use super::field_groups::{FieldGroups, RequiredGroup};
use super::schema_layer::{ArrayType, SchemaError};
use super::schema_projection::{
    AlternativeKind, ConditionalDependency, ConditionalRequirement, EmissionType, ProjectedType,
    Projection,
};
use serde_json::{Number, Value};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum EmittedKind {
    String,
    Integer,
    Number,
    Boolean,
    Literal,
    Struct,
    Map,
    List,
    Set,
    JsonDocument,
    BareObject,
    Root,
    TagString,
    TagObject,
    TagList,
    TagMap,
}

impl EmittedKind {
    fn name(self) -> &'static str {
        match self {
            Self::String => "string",
            Self::Integer => "integer",
            Self::Number => "number",
            Self::Boolean => "boolean",
            Self::Literal => "enum/literal",
            Self::Struct => "struct",
            Self::Map => "map",
            Self::List => "list",
            Self::Set => "set",
            Self::JsonDocument => "JSON document",
            Self::BareObject => "bare object",
            Self::Root => "root",
            Self::TagString => "tag string",
            Self::TagObject => "tag object",
            Self::TagList => "tag list",
            Self::TagMap => "tag map",
        }
    }
}

#[derive(Debug, Clone)]
pub(super) struct StringPlan<'a> {
    pub(super) source_pointer: &'a str,
    pub(super) description: Option<&'a str>,
    pub(super) format: Option<&'a str>,
    pub(super) pattern: Option<&'a str>,
    pub(super) min_length: Option<u64>,
    pub(super) max_length: Option<u64>,
}

impl StringPlan<'_> {
    pub(super) fn emission_error(&self, reason: impl Into<String>) -> SchemaError {
        SchemaError::Projection {
            pointer: self.source_pointer.into(),
            reason: reason.into(),
        }
    }

    /// Inferred domains are additional literal evidence, subject to exactly the
    /// same intersection as a schema enum. No inference caller receives a Projection.
    pub(super) fn literals(&self, values: Vec<String>) -> Result<LiteralPlan, SchemaError> {
        let Self {
            source_pointer,
            description: _description,
            format,
            pattern,
            min_length,
            max_length,
        } = self;
        let projection = Projection {
            source_pointer: (*source_pointer).into(),
            types: vec![super::schema_layer::JsonType::String],
            enum_values: Some(values.into_iter().map(Value::String).collect()),
            format: format.map(str::to_string),
            pattern: pattern.map(str::to_string),
            min_length: *min_length,
            max_length: *max_length,
            ..Projection::default()
        };
        lower_as(&projection, EmittedKind::Literal)?
            .literal
            .ok_or_else(|| self.emission_error("missing inferred literal plan"))
    }
}

pub(super) struct NumericPlan<'a> {
    pub(super) range: Option<(Option<i64>, Option<i64>)>,
    pub(super) format: Option<&'a str>,
    pub(super) multiple_of: Option<MultipleOf<'a>>,
}

/// Documented obligation, not enforced: carina#4229 tracks the missing core
/// multiple-of constraint. Keep the JSON decimal intact for Markdown.
pub(super) struct MultipleOf<'a> {
    pub(super) divisor: &'a Number,
}

#[derive(Clone)]
pub(super) struct LiteralPlan {
    pub(super) source_pointer: String,
    pub(super) value_type: EmissionType,
    pub(super) values: Vec<Value>,
}

pub(super) struct ArrayPlan<'a> {
    pub(super) items: &'a SchemaProperty,
    pub(super) min_items: Option<i64>,
    pub(super) max_items: Option<i64>,
    pub(super) unique_items: bool,
    pub(super) insertion_order: bool,
}

pub(super) struct MapPlan<'a> {
    pub(super) pattern: &'a str,
    pub(super) value: &'a SchemaProperty,
    pub(super) restrict_keys: bool,
    pub(super) min_entries: Option<i64>,
    pub(super) max_entries: Option<i64>,
}

#[derive(Default)]
pub(super) struct ScopePlan {
    pub(super) required: Vec<String>,
    pub(super) field_groups: FieldGroups,
    pub(super) conditional_requirements: Vec<ConditionalRequirement>,
}

pub(super) struct EmissionPlan<'a> {
    pub(super) source_pointer: &'a str,
    pub(super) kind: EmittedKind,
    pub(super) string: Option<StringPlan<'a>>,
    pub(super) numeric: Option<NumericPlan<'a>>,
    pub(super) literal: Option<LiteralPlan>,
    pub(super) array: Option<ArrayPlan<'a>>,
    pub(super) map: Option<MapPlan<'a>>,
    pub(super) properties: Option<&'a BTreeMap<String, SchemaProperty>>,
    pub(super) scope: ScopePlan,
}

impl<'a> EmissionPlan<'a> {
    pub(super) fn is_empty_struct(&self) -> bool {
        self.kind == EmittedKind::Struct && self.properties.is_none_or(BTreeMap::is_empty)
    }

    /// Complete required-member admission with the resource/scope filters used
    /// by both renderers. Provider-populated fields are still emitted fields.
    pub(super) fn validate_required_fields(
        &self,
        pointer: &str,
        cfn_type: &str,
        parent: Option<&str>,
    ) -> Result<(), SchemaError> {
        for member in &self.scope.required {
            let property = self
                .properties
                .and_then(|fields| fields.get(member))
                .ok_or_else(|| {
                    unsupported(
                        pointer,
                        self.kind,
                        &format!("required member {member} missing from emitted fields"),
                    )
                })?;
            let reason = if super::is_deprecated_property(property) {
                "deprecated"
            } else if super::should_omit_property(
                cfn_type,
                super::CfnPropertyPath::new(parent, member),
            ) {
                "omitted"
            } else {
                continue;
            };
            return Err(unsupported(
                pointer,
                self.kind,
                &format!("required member {member} {reason} from emitted fields"),
            ));
        }
        Ok(())
    }

    fn empty(kind: EmittedKind, source_pointer: &'a str) -> Self {
        Self {
            source_pointer,
            kind,
            string: None,
            numeric: None,
            literal: None,
            array: None,
            map: None,
            properties: None,
            scope: ScopePlan::default(),
        }
    }

    pub(super) fn emission_error(&self, reason: impl Into<String>) -> SchemaError {
        SchemaError::Projection {
            pointer: self.source_pointer.into(),
            reason: reason.into(),
        }
    }

    pub(super) fn string(&self) -> Result<&StringPlan<'a>, SchemaError> {
        self.string
            .as_ref()
            .ok_or_else(|| self.emission_error("missing string plan"))
    }

    pub(super) fn numeric(&self) -> Result<&NumericPlan<'a>, SchemaError> {
        self.numeric
            .as_ref()
            .ok_or_else(|| self.emission_error("missing numeric plan"))
    }

    pub(super) fn numeric_override_range(
        &self,
        min: i64,
        max: i64,
    ) -> Result<(Option<i64>, Option<i64>), SchemaError> {
        let (effective_min, effective_max) = self.numeric()?.range.unwrap_or((None, None));
        let min = effective_min.map_or(min, |n| n.max(min));
        let max = effective_max.map_or(max, |n| n.min(max));
        if min > max {
            return Err(self.emission_error("empty numeric override range intersection"));
        }
        Ok((Some(min), Some(max)))
    }

    pub(super) fn validate_integer_enum(&self, values: &[i64]) -> Result<(), SchemaError> {
        if let Some(literal) = &self.literal {
            if literal.value_type == EmissionType::Integer
                && !values.is_empty()
                && values
                    .iter()
                    .all(|v| literal.values.contains(&Value::from(*v)))
            {
                return Ok(());
            }
            return Err(self.emission_error(
                "effective integer enum domain differs from constructor enum domain",
            ));
        }
        let numeric = self.numeric()?;
        let (min, max) = numeric.range.unwrap_or((None, None));
        let projection = Projection {
            source_pointer: self.source_pointer.into(),
            types: vec![super::schema_layer::JsonType::Integer],
            enum_values: Some(values.iter().map(|&v| Value::from(v)).collect()),
            minimum: min.map(Number::from),
            maximum: max.map(Number::from),
            multiple_of: numeric.multiple_of.as_ref().map(|m| m.divisor.clone()),
            ..Projection::default()
        };
        let filtered = lower_as(&projection, EmittedKind::Literal)?;
        if filtered
            .literal
            .as_ref()
            .is_some_and(|p| p.values.len() == values.len())
        {
            return Ok(());
        }
        Err(self
            .emission_error("effective integer enum domain differs from constructor enum domain"))
    }

    pub(super) fn array(&self) -> Result<&ArrayPlan<'a>, SchemaError> {
        self.array
            .as_ref()
            .ok_or_else(|| self.emission_error("missing array plan"))
    }

    pub(super) fn map(&self) -> Result<&MapPlan<'a>, SchemaError> {
        self.map
            .as_ref()
            .ok_or_else(|| self.emission_error("missing map plan"))
    }

    pub(super) fn validate_enum_candidates(&self, values: &[&str]) -> Result<(), SchemaError> {
        if let Some(array) = &self.array {
            return lower(array.items.try_query()?)?.validate_enum_candidates(values);
        }
        if let Some(literal) = &self.literal {
            if literal.value_type == EmissionType::String
                && !values.is_empty()
                && values
                    .iter()
                    .all(|v| literal.values.contains(&Value::String((*v).to_string())))
            {
                return Ok(());
            }
        } else if let Some(string) = &self.string {
            let filtered = string.literals(values.iter().map(|s| s.to_string()).collect())?;
            if filtered.values.len() == values.len() {
                return Ok(());
            }
        }
        Err(self.emission_error(
            "effective enum domain differs from replacement constructor enum domain",
        ))
    }

    pub(super) fn enum_literals(&self, values: &[String]) -> Result<LiteralPlan, SchemaError> {
        if let Some(array) = &self.array {
            return lower(array.items.try_query()?)?.enum_literals(values);
        }
        self.validate_enum_candidates(&values.iter().map(String::as_str).collect::<Vec<_>>())?;
        if let Some(string) = &self.string {
            return string.literals(values.to_vec());
        }
        Ok(LiteralPlan {
            source_pointer: self.source_pointer.into(),
            value_type: EmissionType::String,
            values: values.iter().cloned().map(Value::String).collect(),
        })
    }

    /// Canonical identities are reusable only for an equal effective domain.
    /// Descendant use-site intersections must not inherit wider enum metadata.
    pub(super) fn validate_canonical_domains(
        &self,
        canonical: &Self,
        active: &mut std::collections::BTreeSet<(String, String)>,
    ) -> Result<(), SchemaError> {
        let key = (
            self.source_pointer.to_string(),
            canonical.source_pointer.to_string(),
        );
        if !active.insert(key.clone()) {
            return Ok(());
        }
        if let (Some(effective), Some(original)) = (&self.literal, &canonical.literal)
            && (effective.values.len() != original.values.len()
                || effective
                    .values
                    .iter()
                    .any(|value| !original.values.contains(value)))
        {
            return Err(
                self.emission_error("effective enum domain differs from canonical enum identity")
            );
        }
        if let (Some(effective), Some(original)) = (&self.array, &canonical.array) {
            lower(effective.items.try_query()?)?
                .validate_canonical_domains(&lower(original.items.try_query()?)?, active)?;
        }
        if let (Some(effective), Some(original)) = (&self.map, &canonical.map) {
            lower(effective.value.try_query()?)?
                .validate_canonical_domains(&lower(original.value.try_query()?)?, active)?;
        }
        if let (Some(effective), Some(original)) = (self.properties, canonical.properties) {
            for (name, child) in effective {
                if let Some(original) = original.get(name) {
                    lower(child.try_query()?)?
                        .validate_canonical_domains(&lower(original.try_query()?)?, active)?;
                }
            }
        }
        Ok(())
    }

    pub(super) fn value_type(&self) -> Result<EmissionType, SchemaError> {
        Ok(match self.kind {
            EmittedKind::String | EmittedKind::TagString => EmissionType::String,
            EmittedKind::Integer => EmissionType::Integer,
            EmittedKind::Number => EmissionType::Number,
            EmittedKind::Boolean => EmissionType::Boolean,
            EmittedKind::Literal => {
                self.literal
                    .as_ref()
                    .ok_or_else(|| self.emission_error("missing literal plan"))?
                    .value_type
            }
            EmittedKind::List | EmittedKind::Set | EmittedKind::TagList => EmissionType::Array,
            EmittedKind::Struct
            | EmittedKind::Map
            | EmittedKind::JsonDocument
            | EmittedKind::BareObject
            | EmittedKind::Root
            | EmittedKind::TagObject
            | EmittedKind::TagMap => EmissionType::Object,
        })
    }
}

fn unsupported(pointer: &str, kind: EmittedKind, fact: &str) -> SchemaError {
    let detail = if matches!(fact, "minProperties" | "maxProperties") {
        " (property-count constraint)"
    } else {
        ""
    };
    SchemaError::Projection {
        pointer: pointer.into(),
        reason: format!("{fact} on {} cannot be emitted{detail}", kind.name()),
    }
}

fn absent<T>(
    pointer: &str,
    kind: EmittedKind,
    fact: &str,
    value: &Option<T>,
) -> Result<(), SchemaError> {
    if value.is_some() {
        return Err(unsupported(pointer, kind, fact));
    }
    Ok(())
}

fn empty_map<T>(
    pointer: &str,
    kind: EmittedKind,
    fact: &str,
    value: &Option<BTreeMap<String, T>>,
) -> Result<(), SchemaError> {
    if value.as_ref().is_some_and(|map| !map.is_empty()) {
        return Err(unsupported(pointer, kind, fact));
    }
    Ok(())
}

fn closed_or_legacy(
    pointer: &str,
    kind: EmittedKind,
    value: &Option<bool>,
) -> Result<(), SchemaError> {
    match value {
        // Core structs reject unknown fields, including zero-field structs.
        Some(false) => Ok(()),
        // Preserve HEAD's closed emission for an omitted flag. Cached WAFv2
        // WebACL Regex depends on this shape; absence itself is not enforced.
        None => Ok(()),
        Some(true) => Err(unsupported(pointer, kind, "additionalProperties:true")),
    }
}

/// arrayType changes CFN presentation, not the accepted JSON array values.
/// Block names are decided separately from the value grammar.
fn array_layout(value: &Option<ArrayType>) {
    match value {
        None | Some(ArrayType::Standard) | Some(ArrayType::AttributeList) => {}
    }
}

#[derive(Clone, Copy)]
enum ScopeKind {
    Scalar,
    Struct,
    Root,
    Map,
    Tag,
}

fn inapplicable_object_scope(
    pointer: &str,
    kind: EmittedKind,
    required: &[String],
    field_groups: &FieldGroups,
    dependencies: &BTreeMap<String, ConditionalDependency>,
) -> Result<(), SchemaError> {
    let FieldGroups {
        groups,
        declared_members,
    } = field_groups;
    // Every required-only arm is true for a non-object. An inclusive group
    // is therefore true, but multiple exclusive arms make the domain empty.
    for RequiredGroup {
        kind: alternative_kind,
        members,
    } in groups
    {
        if *alternative_kind == AlternativeKind::OneOf && members.len() > 1 {
            return Err(unsupported(
                pointer,
                kind,
                "unsatisfiable oneOf required alternatives",
            ));
        }
    }
    // Unconditional requirements and dependencies assert object presence only.
    let _inapplicable = (required, groups, declared_members, dependencies);
    Ok(())
}

fn required_fields(
    pointer: &str,
    kind: EmittedKind,
    required: &[String],
    properties: &Option<BTreeMap<String, SchemaProperty>>,
) -> Result<(), SchemaError> {
    for member in required {
        if properties
            .as_ref()
            .is_none_or(|fields| !fields.contains_key(member))
        {
            return Err(unsupported(
                pointer,
                kind,
                &format!("required member {member} missing from emitted fields"),
            ));
        }
    }
    Ok(())
}

fn scope(
    pointer: &str,
    kind: EmittedKind,
    scope_kind: ScopeKind,
    required: &[String],
    groups: &FieldGroups,
    dependencies: &BTreeMap<String, ConditionalDependency>,
) -> Result<ScopePlan, SchemaError> {
    // Each scope makes its own exhaustive decision. Declaration evidence
    // prevents prose from strengthening a simplified rule; keep it with rules.
    let field_groups = match scope_kind {
        ScopeKind::Struct => {
            let FieldGroups {
                groups,
                declared_members,
            } = groups;
            FieldGroups {
                groups: groups.clone(),
                declared_members: declared_members.clone(),
            }
        }
        ScopeKind::Root => {
            let FieldGroups {
                groups,
                declared_members,
            } = groups;
            FieldGroups {
                groups: groups.clone(),
                declared_members: declared_members.clone(),
            }
        }
        ScopeKind::Map => {
            let FieldGroups {
                groups,
                declared_members,
            } = groups;
            if !groups.is_empty() {
                return Err(unsupported(pointer, kind, "FieldGroups"));
            }
            FieldGroups {
                groups: groups.clone(),
                declared_members: declared_members.clone(),
            }
        }
        ScopeKind::Scalar => {
            let FieldGroups {
                groups,
                declared_members,
            } = groups;
            if !groups.is_empty() {
                return Err(unsupported(pointer, kind, "FieldGroups"));
            }
            FieldGroups {
                groups: groups.clone(),
                declared_members: declared_members.clone(),
            }
        }
        ScopeKind::Tag => {
            let FieldGroups {
                groups,
                declared_members,
            } = groups;
            if !groups.is_empty() {
                return Err(unsupported(pointer, kind, "FieldGroups"));
            }
            FieldGroups {
                groups: groups.clone(),
                declared_members: declared_members.clone(),
            }
        }
    };
    let rules = &field_groups.groups;
    for RequiredGroup {
        kind: _alternative_kind,
        members,
    } in rules
    {
        if members.is_empty() {
            return Err(unsupported(pointer, kind, "empty FieldGroups"));
        }
    }
    match scope_kind {
        ScopeKind::Scalar => {
            if !required.is_empty() {
                return Err(unsupported(pointer, kind, "required"));
            }
            if !rules.is_empty() {
                return Err(unsupported(pointer, kind, "FieldGroups"));
            }
            if !dependencies.is_empty() {
                return Err(unsupported(pointer, kind, "dependencies"));
            }
        }
        ScopeKind::Map => {
            if !required.is_empty() {
                return Err(unsupported(pointer, kind, "required keys"));
            }
            if !rules.is_empty() {
                return Err(unsupported(pointer, kind, "FieldGroups"));
            }
        }
        ScopeKind::Tag => {
            if !rules.is_empty() {
                return Err(unsupported(pointer, kind, "FieldGroups"));
            }
        }
        ScopeKind::Struct | ScopeKind::Root => {}
    }
    let mut conditional_requirements = Vec::new();
    for (trigger, dependency) in dependencies {
        match dependency {
            ConditionalDependency::PropertyList(names) => {
                for name in names {
                    if let Some(requirement) = conditional_requirement(
                        pointer,
                        kind,
                        scope_kind,
                        required,
                        ConditionalRequirement {
                            if_present: trigger.clone(),
                            then_present: name.clone(),
                        },
                    )? {
                        conditional_requirements.push(requirement);
                    }
                }
            }
            ConditionalDependency::Schema(child) => {
                return Err(unsupported(
                    &child.try_query()?.source_pointer,
                    kind,
                    "schema dependencies",
                ));
            }
        }
    }
    Ok(ScopePlan {
        required: required.to_vec(),
        field_groups,
        conditional_requirements,
    })
}

/// Pinned core has no presence-conditioned rule. Struct, root and map scopes
/// carry the complete implication as a Markdown obligation; tag implications
/// must already be true because their target is unconditionally required.
fn conditional_requirement(
    pointer: &str,
    kind: EmittedKind,
    scope_kind: ScopeKind,
    required: &[String],
    requirement: ConditionalRequirement,
) -> Result<Option<ConditionalRequirement>, SchemaError> {
    match scope_kind {
        ScopeKind::Struct => {
            let ConditionalRequirement {
                if_present,
                then_present,
            } = requirement;
            Ok(Some(ConditionalRequirement {
                if_present,
                then_present,
            }))
        }
        ScopeKind::Root => {
            let ConditionalRequirement {
                if_present,
                then_present,
            } = requirement;
            Ok(Some(ConditionalRequirement {
                if_present,
                then_present,
            }))
        }
        ScopeKind::Map => {
            let ConditionalRequirement {
                if_present,
                then_present,
            } = requirement;
            Ok(Some(ConditionalRequirement {
                if_present,
                then_present,
            }))
        }
        ScopeKind::Tag => {
            let ConditionalRequirement {
                if_present,
                then_present,
            } = requirement;
            if required.contains(&then_present) {
                Ok(None)
            } else {
                Err(unsupported(
                    pointer,
                    kind,
                    &format!("dependencies {if_present} requires {then_present}"),
                ))
            }
        }
        ScopeKind::Scalar => {
            let ConditionalRequirement {
                if_present,
                then_present,
            } = requirement;
            Err(unsupported(
                pointer,
                kind,
                &format!("dependencies {if_present} requires {then_present}"),
            ))
        }
    }
}

pub(super) fn lower(p: &Projection) -> Result<EmissionPlan<'_>, SchemaError> {
    let kind = if p.projected_type == ProjectedType::JsonDocument {
        EmittedKind::JsonDocument
    } else if p.const_value.is_some() || p.enum_values.is_some() {
        EmittedKind::Literal
    } else {
        match p.value_type()? {
            EmissionType::String => EmittedKind::String,
            EmissionType::Integer => EmittedKind::Integer,
            EmissionType::Number => EmittedKind::Number,
            EmissionType::Boolean => EmittedKind::Boolean,
            EmissionType::Array if p.unique_items == Some(true) => EmittedKind::Set,
            EmissionType::Array => EmittedKind::List,
            EmissionType::Object
                if p.properties
                    .as_ref()
                    .is_some_and(|fields| !fields.is_empty()) =>
            {
                EmittedKind::Struct
            }
            EmissionType::Object
                if p.pattern_properties
                    .as_ref()
                    .is_some_and(|fields| !fields.is_empty()) =>
            {
                EmittedKind::Map
            }
            EmissionType::Object if p.additional_properties == Some(false) => EmittedKind::Struct,
            EmissionType::Object => EmittedKind::BareObject,
        }
    };
    lower_as(p, kind)
}

pub(super) fn lower_as(p: &Projection, kind: EmittedKind) -> Result<EmissionPlan<'_>, SchemaError> {
    let mut plan = EmissionPlan::empty(kind, &p.source_pointer);
    match kind {
        EmittedKind::String => {
            let Projection {
                source_pointer,
                source_name: _source_name,
                conjunction: _conjunction,
                types: _types,
                projected_type: _projected_type,
                description,
                enum_values,
                const_value,
                default_value: _default_value,
                format,
                pattern,
                minimum,
                maximum,
                exclusive_minimum,
                exclusive_maximum,
                multiple_of,
                min_properties,
                max_properties,
                array_type,
                additional_schema,
                min_length,
                max_length,
                min_items,
                max_items,
                unique_items,
                insertion_order,
                items,
                properties,
                pattern_properties,
                additional_properties,
                required,
                reference: _reference,
                concrete: _concrete,
                field_groups,
                dropped_bare_alternatives: _dropped_bare_alternatives,
                dependencies,
                contains,
                is_union: _is_union,
                constrained_array_evidence: _constrained_array_evidence,
            } = p;
            let _inapplicable_array_keywords = (
                array_type,
                min_items,
                max_items,
                unique_items,
                insertion_order,
                items,
                contains,
            );
            let _inapplicable_numeric_keywords = (
                minimum,
                maximum,
                exclusive_minimum,
                exclusive_maximum,
                multiple_of,
            );
            empty_map(source_pointer, kind, "properties", properties)?;
            let _inapplicable_object_keywords = (
                min_properties,
                max_properties,
                pattern_properties,
                additional_schema,
                additional_properties,
            );
            absent(source_pointer, kind, "enum", enum_values)?;
            absent(source_pointer, kind, "const", const_value)?;
            inapplicable_object_scope(source_pointer, kind, required, field_groups, dependencies)?;
            plan.string = Some(StringPlan {
                source_pointer,
                description: description.as_deref(),
                format: format.as_deref(),
                pattern: pattern.as_deref(),
                min_length: *min_length,
                max_length: *max_length,
            });
        }
        EmittedKind::Integer => {
            let Projection {
                source_pointer,
                source_name: _source_name,
                conjunction: _conjunction,
                types: _types,
                projected_type: _projected_type,
                description: _description,
                enum_values,
                const_value,
                default_value: _default_value,
                format,
                pattern,
                minimum,
                maximum,
                exclusive_minimum,
                exclusive_maximum,
                multiple_of,
                min_properties,
                max_properties,
                array_type,
                additional_schema,
                min_length,
                max_length,
                min_items,
                max_items,
                unique_items,
                insertion_order,
                items,
                properties,
                pattern_properties,
                additional_properties,
                required,
                reference: _reference,
                concrete: _concrete,
                field_groups,
                dropped_bare_alternatives: _dropped_bare_alternatives,
                dependencies,
                contains,
                is_union: _is_union,
                constrained_array_evidence: _constrained_array_evidence,
            } = p;
            let _inapplicable_array_keywords = (
                array_type,
                min_items,
                max_items,
                unique_items,
                insertion_order,
                items,
                contains,
            );
            let _inapplicable_string_keywords = (pattern, min_length, max_length);
            empty_map(source_pointer, kind, "properties", properties)?;
            let _inapplicable_object_keywords = (
                min_properties,
                max_properties,
                pattern_properties,
                additional_schema,
                additional_properties,
            );
            absent(source_pointer, kind, "enum", enum_values)?;
            absent(source_pointer, kind, "const", const_value)?;
            inapplicable_object_scope(source_pointer, kind, required, field_groups, dependencies)?;
            let has_bounds = minimum.is_some()
                || maximum.is_some()
                || exclusive_minimum.is_some()
                || exclusive_maximum.is_some();
            plan.numeric = Some(NumericPlan {
                range: if has_bounds {
                    Some(p.integer_bounds()?)
                } else {
                    None
                },
                format: format.as_deref(),
                multiple_of: multiple_of.as_ref().map(|divisor| MultipleOf { divisor }),
            });
        }
        EmittedKind::Number => {
            let Projection {
                source_pointer,
                source_name: _source_name,
                conjunction: _conjunction,
                types: _types,
                projected_type: _projected_type,
                description: _description,
                enum_values,
                const_value,
                default_value: _default_value,
                format,
                pattern,
                minimum,
                maximum,
                exclusive_minimum,
                exclusive_maximum,
                multiple_of,
                min_properties,
                max_properties,
                array_type,
                additional_schema,
                min_length,
                max_length,
                min_items,
                max_items,
                unique_items,
                insertion_order,
                items,
                properties,
                pattern_properties,
                additional_properties,
                required,
                reference: _reference,
                concrete: _concrete,
                field_groups,
                dropped_bare_alternatives: _dropped_bare_alternatives,
                dependencies,
                contains,
                is_union: _is_union,
                constrained_array_evidence: _constrained_array_evidence,
            } = p;
            let _inapplicable_array_keywords = (
                array_type,
                min_items,
                max_items,
                unique_items,
                insertion_order,
                items,
                contains,
            );
            let _inapplicable_string_keywords = (pattern, min_length, max_length);
            empty_map(source_pointer, kind, "properties", properties)?;
            let _inapplicable_object_keywords = (
                min_properties,
                max_properties,
                pattern_properties,
                additional_schema,
                additional_properties,
            );
            absent(source_pointer, kind, "enum", enum_values)?;
            absent(source_pointer, kind, "const", const_value)?;
            inapplicable_object_scope(source_pointer, kind, required, field_groups, dependencies)?;
            let has_bounds = minimum.is_some()
                || maximum.is_some()
                || exclusive_minimum.is_some()
                || exclusive_maximum.is_some();
            plan.numeric = Some(NumericPlan {
                range: if has_bounds {
                    Some(p.integer_bounds()?)
                } else {
                    None
                },
                format: format.as_deref(),
                multiple_of: multiple_of.as_ref().map(|divisor| MultipleOf { divisor }),
            });
        }
        EmittedKind::Boolean => {
            let Projection {
                source_pointer,
                source_name: _source_name,
                conjunction: _conjunction,
                types: _types,
                projected_type: _projected_type,
                description: _description,
                enum_values,
                const_value,
                default_value: _default_value,
                format,
                pattern,
                minimum,
                maximum,
                exclusive_minimum,
                exclusive_maximum,
                multiple_of,
                min_properties,
                max_properties,
                array_type,
                additional_schema,
                min_length,
                max_length,
                min_items,
                max_items,
                unique_items,
                insertion_order,
                items,
                properties,
                pattern_properties,
                additional_properties,
                required,
                reference: _reference,
                concrete: _concrete,
                field_groups,
                dropped_bare_alternatives: _dropped_bare_alternatives,
                dependencies,
                contains,
                is_union: _is_union,
                constrained_array_evidence: _constrained_array_evidence,
            } = p;
            let _inapplicable_array_keywords = (
                array_type,
                min_items,
                max_items,
                unique_items,
                insertion_order,
                items,
                contains,
            );
            let _inapplicable_numeric_keywords = (
                minimum,
                maximum,
                exclusive_minimum,
                exclusive_maximum,
                multiple_of,
            );
            let _inapplicable_string_keywords = (pattern, min_length, max_length);
            empty_map(source_pointer, kind, "properties", properties)?;
            let _inapplicable_object_keywords = (
                min_properties,
                max_properties,
                pattern_properties,
                additional_schema,
                additional_properties,
            );
            let _format_annotation = format;
            absent(source_pointer, kind, "enum", enum_values)?;
            absent(source_pointer, kind, "const", const_value)?;
            inapplicable_object_scope(source_pointer, kind, required, field_groups, dependencies)?;
        }
        EmittedKind::Literal => {
            let Projection {
                source_pointer,
                source_name: _source_name,
                conjunction: _conjunction,
                types: _types,
                projected_type: _projected_type,
                description: _description,
                enum_values,
                const_value,
                default_value: _default_value,
                format,
                pattern,
                minimum,
                maximum,
                exclusive_minimum,
                exclusive_maximum,
                multiple_of,
                min_properties,
                max_properties,
                array_type,
                additional_schema,
                min_length,
                max_length,
                min_items,
                max_items,
                unique_items,
                insertion_order,
                items,
                properties,
                pattern_properties,
                additional_properties,
                required,
                reference: _reference,
                concrete: _concrete,
                field_groups,
                dropped_bare_alternatives: _dropped_bare_alternatives,
                dependencies,
                contains,
                is_union: _is_union,
                constrained_array_evidence: _constrained_array_evidence,
            } = p;
            let _inapplicable_array_keywords = (
                array_type,
                min_items,
                max_items,
                unique_items,
                insertion_order,
                items,
                contains,
            );
            empty_map(source_pointer, kind, "properties", properties)?;
            let _inapplicable_object_keywords = (
                min_properties,
                max_properties,
                pattern_properties,
                additional_schema,
                additional_properties,
            );
            inapplicable_object_scope(source_pointer, kind, required, field_groups, dependencies)?;
            // These constraints have filtered the finite literal domain. Recheck
            // here so materialized/cached projections have the same proof.
            let mut filtered = p.clone();
            super::schema_projection::intersect_literals(&mut filtered)?;
            let values = match (const_value, enum_values) {
                (Some(value), _enum_values) => vec![value.clone()],
                (None, Some(_values)) => filtered.enum_values.ok_or_else(|| {
                    unsupported(source_pointer, kind, "missing filtered literal domain")
                })?,
                (None, None) => {
                    return Err(unsupported(
                        source_pointer,
                        kind,
                        "missing literal evidence",
                    ));
                }
            };
            let _intersected_constraints = (
                format,
                pattern,
                min_length,
                max_length,
                minimum,
                maximum,
                exclusive_minimum,
                exclusive_maximum,
                multiple_of,
            );
            plan.literal = Some(LiteralPlan {
                source_pointer: source_pointer.clone(),
                value_type: p.value_type()?,
                values,
            });
        }
        EmittedKind::Struct => {
            let Projection {
                source_pointer,
                source_name: _source_name,
                conjunction: _conjunction,
                types: _types,
                projected_type: _projected_type,
                description: _description,
                enum_values,
                const_value,
                default_value: _default_value,
                format,
                pattern,
                minimum,
                maximum,
                exclusive_minimum,
                exclusive_maximum,
                multiple_of,
                min_properties,
                max_properties,
                array_type,
                additional_schema,
                min_length,
                max_length,
                min_items,
                max_items,
                unique_items,
                insertion_order,
                items,
                properties,
                pattern_properties,
                additional_properties,
                required,
                reference: _reference,
                concrete: _concrete,
                field_groups,
                dropped_bare_alternatives: _dropped_bare_alternatives,
                dependencies,
                contains,
                is_union: _is_union,
                constrained_array_evidence: _constrained_array_evidence,
            } = p;
            let _inapplicable_array_keywords = (
                array_type,
                min_items,
                max_items,
                unique_items,
                insertion_order,
                items,
                contains,
            );
            let _inapplicable_numeric_keywords = (
                minimum,
                maximum,
                exclusive_minimum,
                exclusive_maximum,
                multiple_of,
            );
            let _inapplicable_string_keywords = (pattern, min_length, max_length);
            let _format_annotation = format;
            absent(source_pointer, kind, "enum", enum_values)?;
            absent(source_pointer, kind, "const", const_value)?;
            absent(source_pointer, kind, "minProperties", min_properties)?;
            absent(source_pointer, kind, "maxProperties", max_properties)?;
            absent(
                source_pointer,
                kind,
                "additionalProperties schema",
                additional_schema,
            )?;
            empty_map(
                source_pointer,
                kind,
                "patternProperties",
                pattern_properties,
            )?;
            required_fields(source_pointer, kind, required, properties)?;
            plan.scope = scope(
                source_pointer,
                kind,
                ScopeKind::Struct,
                required,
                field_groups,
                dependencies,
            )?;
            closed_or_legacy(source_pointer, kind, additional_properties)?;
            plan.properties = properties.as_ref();
        }
        EmittedKind::Map => {
            let Projection {
                source_pointer,
                source_name: _source_name,
                conjunction: _conjunction,
                types: _types,
                projected_type: _projected_type,
                description: _description,
                enum_values,
                const_value,
                default_value: _default_value,
                format,
                pattern,
                minimum,
                maximum,
                exclusive_minimum,
                exclusive_maximum,
                multiple_of,
                min_properties,
                max_properties,
                array_type,
                additional_schema,
                min_length,
                max_length,
                min_items,
                max_items,
                unique_items,
                insertion_order,
                items,
                properties,
                pattern_properties,
                additional_properties,
                required,
                reference: _reference,
                concrete: _concrete,
                field_groups,
                dropped_bare_alternatives: _dropped_bare_alternatives,
                dependencies,
                contains,
                is_union: _is_union,
                constrained_array_evidence: _constrained_array_evidence,
            } = p;
            let _inapplicable_array_keywords = (
                array_type,
                min_items,
                max_items,
                unique_items,
                insertion_order,
                items,
                contains,
            );
            let _inapplicable_numeric_keywords = (
                minimum,
                maximum,
                exclusive_minimum,
                exclusive_maximum,
                multiple_of,
            );
            let _inapplicable_string_keywords = (pattern, min_length, max_length);
            let _format_annotation = format;
            absent(source_pointer, kind, "enum", enum_values)?;
            absent(source_pointer, kind, "const", const_value)?;
            absent(
                source_pointer,
                kind,
                "additionalProperties schema",
                additional_schema,
            )?;
            empty_map(source_pointer, kind, "properties", properties)?;
            plan.scope = scope(
                source_pointer,
                kind,
                ScopeKind::Map,
                required,
                field_groups,
                dependencies,
            )?;
            let patterns = pattern_properties
                .as_ref()
                .filter(|patterns| patterns.len() == 1)
                .ok_or_else(|| {
                    unsupported(
                        source_pointer,
                        kind,
                        "multiple patternProperties value domains",
                    )
                })?;
            if *additional_properties == Some(true) {
                return Err(unsupported(
                    source_pointer,
                    kind,
                    "additionalProperties open value domain",
                ));
            }
            // Single patternProperties entry + omitted additionalProperties =>
            // map(any key, value = that entry's value type). This is stricter
            // than Draft-07 only for values of non-matching keys; keys remain
            // unrestricted. Explicit false restricts keys to the pattern, while
            // explicit true/schema-valued open domains are typed errors above.
            let (pattern, value) = patterns.iter().next().ok_or_else(|| {
                unsupported(
                    source_pointer,
                    kind,
                    "missing patternProperties value domain",
                )
            })?;
            let (min_entries, max_entries) = p.map_bounds()?;
            let _count_bounds = (min_properties, max_properties);
            plan.map = Some(MapPlan {
                pattern,
                value,
                restrict_keys: *additional_properties == Some(false),
                min_entries,
                max_entries,
            });
        }
        EmittedKind::List => {
            let Projection {
                source_pointer,
                source_name: _source_name,
                conjunction: _conjunction,
                types: _types,
                projected_type: _projected_type,
                description: _description,
                enum_values,
                const_value,
                default_value: _default_value,
                format,
                pattern,
                minimum,
                maximum,
                exclusive_minimum,
                exclusive_maximum,
                multiple_of,
                min_properties,
                max_properties,
                array_type,
                additional_schema,
                min_length,
                max_length,
                min_items,
                max_items,
                unique_items,
                insertion_order,
                items,
                properties,
                pattern_properties,
                additional_properties,
                required,
                reference: _reference,
                concrete: _concrete,
                field_groups,
                dropped_bare_alternatives: _dropped_bare_alternatives,
                dependencies,
                contains,
                is_union: _is_union,
                constrained_array_evidence: _constrained_array_evidence,
            } = p;
            let _inapplicable_numeric_keywords = (
                minimum,
                maximum,
                exclusive_minimum,
                exclusive_maximum,
                multiple_of,
            );
            let _inapplicable_string_keywords = (pattern, min_length, max_length);
            empty_map(source_pointer, kind, "properties", properties)?;
            let _inapplicable_object_keywords = (
                min_properties,
                max_properties,
                pattern_properties,
                additional_schema,
                additional_properties,
            );
            let _format_annotation = format;
            absent(source_pointer, kind, "enum", enum_values)?;
            absent(source_pointer, kind, "const", const_value)?;
            absent(source_pointer, kind, "contains", contains)?;
            inapplicable_object_scope(source_pointer, kind, required, field_groups, dependencies)?;
            array_layout(array_type);
            let items = items
                .as_deref()
                .ok_or_else(|| unsupported(source_pointer, kind, "missing items"))?;
            for (fact, bound) in [("minItems", min_items), ("maxItems", max_items)] {
                if bound.is_some_and(|bound| bound < 0) {
                    return Err(unsupported(source_pointer, kind, fact));
                }
            }
            plan.array = Some(ArrayPlan {
                items,
                min_items: *min_items,
                max_items: *max_items,
                unique_items: *unique_items == Some(true),
                insertion_order: *insertion_order != Some(false),
            });
        }
        EmittedKind::Set => {
            let Projection {
                source_pointer,
                source_name: _source_name,
                conjunction: _conjunction,
                types: _types,
                projected_type: _projected_type,
                description: _description,
                enum_values,
                const_value,
                default_value: _default_value,
                format,
                pattern,
                minimum,
                maximum,
                exclusive_minimum,
                exclusive_maximum,
                multiple_of,
                min_properties,
                max_properties,
                array_type,
                additional_schema,
                min_length,
                max_length,
                min_items,
                max_items,
                unique_items,
                insertion_order,
                items,
                properties,
                pattern_properties,
                additional_properties,
                required,
                reference: _reference,
                concrete: _concrete,
                field_groups,
                dropped_bare_alternatives: _dropped_bare_alternatives,
                dependencies,
                contains,
                is_union: _is_union,
                constrained_array_evidence: _constrained_array_evidence,
            } = p;
            let _inapplicable_numeric_keywords = (
                minimum,
                maximum,
                exclusive_minimum,
                exclusive_maximum,
                multiple_of,
            );
            let _inapplicable_string_keywords = (pattern, min_length, max_length);
            empty_map(source_pointer, kind, "properties", properties)?;
            let _inapplicable_object_keywords = (
                min_properties,
                max_properties,
                pattern_properties,
                additional_schema,
                additional_properties,
            );
            let _format_annotation = format;
            absent(source_pointer, kind, "enum", enum_values)?;
            absent(source_pointer, kind, "const", const_value)?;
            absent(source_pointer, kind, "contains", contains)?;
            inapplicable_object_scope(source_pointer, kind, required, field_groups, dependencies)?;
            array_layout(array_type);
            let items = items
                .as_deref()
                .ok_or_else(|| unsupported(source_pointer, kind, "missing items"))?;
            for (fact, bound) in [("minItems", min_items), ("maxItems", max_items)] {
                if bound.is_some_and(|bound| bound < 0) {
                    return Err(unsupported(source_pointer, kind, fact));
                }
            }
            plan.array = Some(ArrayPlan {
                items,
                min_items: *min_items,
                max_items: *max_items,
                unique_items: *unique_items == Some(true),
                insertion_order: *insertion_order != Some(false),
            });
        }
        EmittedKind::JsonDocument => {
            let Projection {
                source_pointer,
                source_name: _source_name,
                conjunction: _conjunction,
                types: _types,
                projected_type: _projected_type,
                description: _description,
                enum_values,
                const_value,
                default_value: _default_value,
                format,
                pattern,
                minimum,
                maximum,
                exclusive_minimum,
                exclusive_maximum,
                multiple_of,
                min_properties,
                max_properties,
                array_type,
                additional_schema,
                min_length,
                max_length,
                min_items,
                max_items,
                unique_items,
                insertion_order,
                items,
                properties,
                pattern_properties,
                additional_properties,
                required,
                reference: _reference,
                concrete: _concrete,
                field_groups,
                dropped_bare_alternatives: _dropped_bare_alternatives,
                dependencies,
                contains,
                is_union: _is_union,
                constrained_array_evidence: _constrained_array_evidence,
            } = p;
            let _inapplicable_array_keywords = (
                array_type,
                min_items,
                max_items,
                unique_items,
                insertion_order,
                items,
                contains,
            );
            let _inapplicable_numeric_keywords = (
                minimum,
                maximum,
                exclusive_minimum,
                exclusive_maximum,
                multiple_of,
            );
            absent(source_pointer, kind, "enum", enum_values)?;
            absent(source_pointer, kind, "const", const_value)?;
            absent(source_pointer, kind, "format", format)?;
            absent(source_pointer, kind, "pattern", pattern)?;
            absent(source_pointer, kind, "minProperties", min_properties)?;
            absent(source_pointer, kind, "maxProperties", max_properties)?;
            absent(
                source_pointer,
                kind,
                "additionalProperties schema",
                additional_schema,
            )?;
            absent(source_pointer, kind, "minLength", min_length)?;
            absent(source_pointer, kind, "maxLength", max_length)?;
            empty_map(source_pointer, kind, "properties", properties)?;
            empty_map(
                source_pointer,
                kind,
                "patternProperties",
                pattern_properties,
            )?;
            absent(
                source_pointer,
                kind,
                "additionalProperties",
                additional_properties,
            )?;
            plan.scope = scope(
                source_pointer,
                kind,
                ScopeKind::Scalar,
                required,
                field_groups,
                dependencies,
            )?;
        }
        EmittedKind::BareObject => {
            let Projection {
                source_pointer,
                source_name: _source_name,
                conjunction: _conjunction,
                types: _types,
                projected_type: _projected_type,
                description: _description,
                enum_values,
                const_value,
                default_value: _default_value,
                format,
                pattern,
                minimum,
                maximum,
                exclusive_minimum,
                exclusive_maximum,
                multiple_of,
                min_properties,
                max_properties,
                array_type,
                additional_schema,
                min_length,
                max_length,
                min_items,
                max_items,
                unique_items,
                insertion_order,
                items,
                properties,
                pattern_properties,
                additional_properties,
                required,
                reference: _reference,
                concrete: _concrete,
                field_groups,
                dropped_bare_alternatives: _dropped_bare_alternatives,
                dependencies,
                contains,
                is_union: _is_union,
                constrained_array_evidence: _constrained_array_evidence,
            } = p;
            let _inapplicable_array_keywords = (
                array_type,
                min_items,
                max_items,
                unique_items,
                insertion_order,
                items,
                contains,
            );
            let _inapplicable_numeric_keywords = (
                minimum,
                maximum,
                exclusive_minimum,
                exclusive_maximum,
                multiple_of,
            );
            let _inapplicable_string_keywords = (pattern, min_length, max_length);
            let _format_annotation = format;
            absent(source_pointer, kind, "enum", enum_values)?;
            absent(source_pointer, kind, "const", const_value)?;
            absent(source_pointer, kind, "minProperties", min_properties)?;
            absent(source_pointer, kind, "maxProperties", max_properties)?;
            absent(
                source_pointer,
                kind,
                "additionalProperties schema",
                additional_schema,
            )?;
            empty_map(source_pointer, kind, "properties", properties)?;
            empty_map(
                source_pointer,
                kind,
                "patternProperties",
                pattern_properties,
            )?;
            absent(
                source_pointer,
                kind,
                "additionalProperties",
                additional_properties,
            )?;
            plan.scope = scope(
                source_pointer,
                kind,
                ScopeKind::Scalar,
                required,
                field_groups,
                dependencies,
            )?;
        }
        EmittedKind::Root => {
            let Projection {
                source_pointer,
                source_name: _source_name,
                conjunction: _conjunction,
                types: _types,
                projected_type: _projected_type,
                description: _description,
                enum_values,
                const_value,
                default_value: _default_value,
                format,
                pattern,
                minimum,
                maximum,
                exclusive_minimum,
                exclusive_maximum,
                multiple_of,
                min_properties,
                max_properties,
                array_type,
                additional_schema,
                min_length,
                max_length,
                min_items,
                max_items,
                unique_items,
                insertion_order,
                items,
                properties,
                pattern_properties,
                additional_properties,
                required,
                reference: _reference,
                concrete: _concrete,
                field_groups,
                dropped_bare_alternatives: _dropped_bare_alternatives,
                dependencies,
                contains,
                is_union: _is_union,
                constrained_array_evidence: _constrained_array_evidence,
            } = p;
            let _inapplicable_array_keywords = (
                array_type,
                min_items,
                max_items,
                unique_items,
                insertion_order,
                items,
                contains,
            );
            let _inapplicable_numeric_keywords = (
                minimum,
                maximum,
                exclusive_minimum,
                exclusive_maximum,
                multiple_of,
            );
            let _inapplicable_string_keywords = (pattern, min_length, max_length);
            let _format_annotation = format;
            absent(source_pointer, kind, "enum", enum_values)?;
            absent(source_pointer, kind, "const", const_value)?;
            absent(source_pointer, kind, "minProperties", min_properties)?;
            absent(source_pointer, kind, "maxProperties", max_properties)?;
            absent(
                source_pointer,
                kind,
                "additionalProperties schema",
                additional_schema,
            )?;
            empty_map(
                source_pointer,
                kind,
                "patternProperties",
                pattern_properties,
            )?;
            required_fields(source_pointer, kind, required, properties)?;
            plan.scope = scope(
                source_pointer,
                kind,
                ScopeKind::Root,
                required,
                field_groups,
                dependencies,
            )?;
            closed_or_legacy(source_pointer, kind, additional_properties)?;
            plan.properties = properties.as_ref();
        }
        EmittedKind::TagString => {
            let Projection {
                source_pointer,
                source_name: _source_name,
                conjunction: _conjunction,
                types: _types,
                projected_type: _projected_type,
                description: _description,
                enum_values,
                const_value,
                default_value: _default_value,
                format,
                pattern,
                minimum,
                maximum,
                exclusive_minimum,
                exclusive_maximum,
                multiple_of,
                min_properties,
                max_properties,
                array_type,
                additional_schema,
                min_length,
                max_length,
                min_items,
                max_items,
                unique_items,
                insertion_order,
                items,
                properties,
                pattern_properties,
                additional_properties,
                required,
                reference: _reference,
                concrete: _concrete,
                field_groups,
                dropped_bare_alternatives,
                dependencies,
                contains,
                is_union: _is_union,
                constrained_array_evidence: _constrained_array_evidence,
            } = p;
            let _inapplicable_array_keywords = (
                array_type,
                min_items,
                max_items,
                unique_items,
                insertion_order,
                items,
                contains,
            );
            let _inapplicable_numeric_keywords = (
                minimum,
                maximum,
                exclusive_minimum,
                exclusive_maximum,
                multiple_of,
            );
            empty_map(source_pointer, kind, "properties", properties)?;
            let _inapplicable_object_keywords = (
                min_properties,
                max_properties,
                pattern_properties,
                additional_schema,
                additional_properties,
            );
            if !dropped_bare_alternatives.is_empty() {
                // Materialized conjuncts have no Layer origin to revisit for
                // strict tag arm agreement. Discarded domains cannot qualify.
                return Err(unsupported(
                    source_pointer,
                    kind,
                    "dropped oneOf/anyOf alternatives",
                ));
            }
            absent(source_pointer, kind, "enum", enum_values)?;
            absent(source_pointer, kind, "const", const_value)?;
            inapplicable_object_scope(source_pointer, kind, required, field_groups, dependencies)?;
            plan.string = Some(StringPlan {
                source_pointer,
                description: None,
                format: format.as_deref(),
                pattern: pattern.as_deref(),
                min_length: *min_length,
                max_length: *max_length,
            });
        }
        EmittedKind::TagObject => {
            let Projection {
                source_pointer,
                source_name: _source_name,
                conjunction: _conjunction,
                types: _types,
                projected_type: _projected_type,
                description: _description,
                enum_values,
                const_value,
                default_value: _default_value,
                format,
                pattern,
                minimum,
                maximum,
                exclusive_minimum,
                exclusive_maximum,
                multiple_of,
                min_properties,
                max_properties,
                array_type,
                additional_schema,
                min_length,
                max_length,
                min_items,
                max_items,
                unique_items,
                insertion_order,
                items,
                properties,
                pattern_properties,
                additional_properties,
                required,
                reference: _reference,
                concrete: _concrete,
                field_groups,
                dropped_bare_alternatives: _dropped_bare_alternatives,
                dependencies,
                contains,
                is_union: _is_union,
                constrained_array_evidence: _constrained_array_evidence,
            } = p;
            let _inapplicable_array_keywords = (
                array_type,
                min_items,
                max_items,
                unique_items,
                insertion_order,
                items,
                contains,
            );
            let _inapplicable_numeric_keywords = (
                minimum,
                maximum,
                exclusive_minimum,
                exclusive_maximum,
                multiple_of,
            );
            let _inapplicable_string_keywords = (pattern, min_length, max_length);
            let _format_annotation = format;
            absent(source_pointer, kind, "enum", enum_values)?;
            absent(source_pointer, kind, "const", const_value)?;
            absent(source_pointer, kind, "minProperties", min_properties)?;
            absent(source_pointer, kind, "maxProperties", max_properties)?;
            absent(
                source_pointer,
                kind,
                "additionalProperties schema",
                additional_schema,
            )?;
            empty_map(
                source_pointer,
                kind,
                "patternProperties",
                pattern_properties,
            )?;
            plan.scope = scope(
                source_pointer,
                kind,
                ScopeKind::Tag,
                required,
                field_groups,
                dependencies,
            )?;
            closed_or_legacy(source_pointer, kind, additional_properties)?;
            plan.properties = properties.as_ref();
        }
        EmittedKind::TagList => {
            let Projection {
                source_pointer,
                source_name: _source_name,
                conjunction: _conjunction,
                types: _types,
                projected_type: _projected_type,
                description: _description,
                enum_values,
                const_value,
                default_value: _default_value,
                format,
                pattern,
                minimum,
                maximum,
                exclusive_minimum,
                exclusive_maximum,
                multiple_of,
                min_properties,
                max_properties,
                array_type,
                additional_schema,
                min_length,
                max_length,
                min_items,
                max_items,
                unique_items,
                insertion_order,
                items,
                properties,
                pattern_properties,
                additional_properties,
                required,
                reference: _reference,
                concrete: _concrete,
                field_groups,
                dropped_bare_alternatives: _dropped_bare_alternatives,
                dependencies,
                contains,
                is_union: _is_union,
                constrained_array_evidence: _constrained_array_evidence,
            } = p;
            let _inapplicable_numeric_keywords = (
                minimum,
                maximum,
                exclusive_minimum,
                exclusive_maximum,
                multiple_of,
            );
            let _inapplicable_string_keywords = (pattern, min_length, max_length);
            empty_map(source_pointer, kind, "properties", properties)?;
            let _inapplicable_object_keywords = (
                min_properties,
                max_properties,
                pattern_properties,
                additional_schema,
                additional_properties,
            );
            let _format_annotation = format;
            absent(source_pointer, kind, "enum", enum_values)?;
            absent(source_pointer, kind, "const", const_value)?;
            absent(source_pointer, kind, "contains", contains)?;
            inapplicable_object_scope(source_pointer, kind, required, field_groups, dependencies)?;
            array_layout(array_type);
            let items = items
                .as_deref()
                .ok_or_else(|| unsupported(source_pointer, kind, "missing items"))?;
            for (fact, bound) in [("minItems", min_items), ("maxItems", max_items)] {
                if bound.is_some_and(|bound| bound < 0) {
                    return Err(unsupported(source_pointer, kind, fact));
                }
            }
            plan.array = Some(ArrayPlan {
                items,
                min_items: *min_items,
                max_items: *max_items,
                unique_items: *unique_items == Some(true),
                insertion_order: *insertion_order != Some(false),
            });
        }
        EmittedKind::TagMap => {
            let Projection {
                source_pointer,
                source_name: _source_name,
                conjunction: _conjunction,
                types: _types,
                projected_type: _projected_type,
                description: _description,
                enum_values,
                const_value,
                default_value: _default_value,
                format,
                pattern,
                minimum,
                maximum,
                exclusive_minimum,
                exclusive_maximum,
                multiple_of,
                min_properties,
                max_properties,
                array_type,
                additional_schema,
                min_length,
                max_length,
                min_items,
                max_items,
                unique_items,
                insertion_order,
                items,
                properties,
                pattern_properties,
                additional_properties,
                required,
                reference: _reference,
                concrete: _concrete,
                field_groups,
                dropped_bare_alternatives: _dropped_bare_alternatives,
                dependencies,
                contains,
                is_union: _is_union,
                constrained_array_evidence: _constrained_array_evidence,
            } = p;
            let _inapplicable_array_keywords = (
                array_type,
                min_items,
                max_items,
                unique_items,
                insertion_order,
                items,
                contains,
            );
            let _inapplicable_numeric_keywords = (
                minimum,
                maximum,
                exclusive_minimum,
                exclusive_maximum,
                multiple_of,
            );
            let _inapplicable_string_keywords = (pattern, min_length, max_length);
            let _format_annotation = format;
            absent(source_pointer, kind, "enum", enum_values)?;
            absent(source_pointer, kind, "const", const_value)?;
            absent(
                source_pointer,
                kind,
                "additionalProperties schema",
                additional_schema,
            )?;
            empty_map(source_pointer, kind, "properties", properties)?;
            plan.scope = scope(
                source_pointer,
                kind,
                ScopeKind::Tag,
                required,
                field_groups,
                dependencies,
            )?;
            let patterns = pattern_properties
                .as_ref()
                .filter(|patterns| patterns.len() == 1)
                .ok_or_else(|| {
                    unsupported(
                        source_pointer,
                        kind,
                        "multiple patternProperties value domains",
                    )
                })?;
            let (pattern, value) = patterns
                .iter()
                .next()
                .ok_or_else(|| unsupported(source_pointer, kind, "missing map value domain"))?;
            let (min_entries, max_entries) = p.map_bounds()?;
            let _count_bounds = (min_properties, max_properties);
            plan.map = Some(MapPlan {
                pattern,
                value,
                restrict_keys: *additional_properties == Some(false),
                min_entries,
                max_entries,
            });
        }
    }
    Ok(plan)
}
