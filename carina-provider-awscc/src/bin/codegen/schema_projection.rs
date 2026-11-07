//! Shared, validated queries over the lossless Layer.
//!
//! SchemaProperty holds typed query handles. Its query origin always points
//! into the Layer; queries never reconstruct facts from fixture fields.
//! References and allOf intersect constraints. Same-type array anyOf arms must
//! be equivalent, including children, bounds, ordering, and uniqueness;
//! overlapping equivalent oneOf arms are rejected. Object
//! alternatives project portable sibling requirements for both renderers.
//! Mixed-type alternatives select one constrained arm and record bare arms.
//! The object|string type set projects to a JSON document, regardless of its site.
#[cfg(test)]
use super::EnumValue;
use super::SchemaProperty;
#[cfg(test)]
use super::TypeValue;
use super::field_groups::FieldGroups;
use super::schema_layer::{
    ArrayType, Dependency, JsonType, Layer, NodeId, PropertyKeywords, RefTarget, SchemaError,
    SchemaNode, TypeSet,
};
use serde_json::{Number, Value};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::{Arc, OnceLock};

#[derive(Debug, Clone)]
pub(super) struct Origin {
    pub(super) layer: Arc<Layer>,
    pub(super) node: NodeId,
    annotations: AnnotationProjection,
}

/// Site annotations do not inherit through references. Keep their provenance
/// separate from the effective constraints shared by both renderers.
#[derive(Debug, Clone, Default)]
struct AnnotationProjection {
    description: Option<String>,
    default: Option<Value>,
}

impl Origin {
    pub(super) fn new(layer: &Arc<Layer>, node: NodeId) -> Self {
        let annotations = match layer.node(node) {
            SchemaNode::Boolean(_value) => AnnotationProjection::default(),
            SchemaNode::Keywords(keywords) => AnnotationProjection {
                description: keywords.description.clone(),
                default: keywords.default.clone(),
            },
        };
        Self {
            layer: Arc::clone(layer),
            node,
            annotations,
        }
    }
}

#[derive(Debug, Clone)]
pub(super) struct Reference {
    pub(super) path: String,
    pub(super) name: String,
    /// Query handle on the same Layer, before reference-site requirements.
    pub(super) canonical: Box<SchemaProperty>,
    pub(super) property: Box<SchemaProperty>,
}

#[derive(Debug, Clone)]
pub(super) enum ConditionalDependency {
    Schema(Box<SchemaProperty>),
    PropertyList(Vec<String>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct ConditionalRequirement {
    pub(super) if_present: String,
    pub(super) then_present: String,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(super) enum ProjectedType {
    #[default]
    Unspecified,
    Value(JsonType),
    JsonDocument,
}

/// Closed value grammar used by Rust and Markdown emission. An absent type
/// requires literal evidence; neither renderer may infer a String for it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum EmissionType {
    String,
    Boolean,
    Integer,
    Number,
    Array,
    Object,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum AlternativeKind {
    OneOf,
    AnyOf,
}

/// The Layer keeps the complete arm; projection records every bare spelling
/// discarded by the mixed-type policy, including its composition and type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct DroppedBareAlternative {
    pub(super) kind: AlternativeKind,
    pub(super) arm: NodeId,
    pub(super) json_type: JsonType,
}

#[derive(Debug, Clone, Default)]
pub(super) struct Projection {
    pub(super) source_pointer: String,
    pub(super) source_name: String,
    pub(super) conjunction: bool,
    pub(super) types: Vec<JsonType>,
    pub(super) projected_type: ProjectedType,
    pub(super) description: Option<String>,
    pub(super) enum_values: Option<Vec<Value>>,
    pub(super) const_value: Option<Value>,
    pub(super) default_value: Option<Value>,
    pub(super) format: Option<String>,
    pub(super) pattern: Option<String>,
    pub(super) minimum: Option<Number>,
    pub(super) maximum: Option<Number>,
    pub(super) exclusive_minimum: Option<Number>,
    pub(super) exclusive_maximum: Option<Number>,
    pub(super) multiple_of: Option<Number>,
    pub(super) min_properties: Option<Number>,
    pub(super) max_properties: Option<Number>,
    pub(super) array_type: Option<ArrayType>,
    pub(super) additional_schema: Option<Box<SchemaProperty>>,
    pub(super) min_length: Option<u64>,
    pub(super) max_length: Option<u64>,
    pub(super) min_items: Option<i64>,
    pub(super) max_items: Option<i64>,
    pub(super) unique_items: Option<bool>,
    pub(super) insertion_order: Option<bool>,
    pub(super) items: Option<Box<SchemaProperty>>,
    pub(super) properties: Option<BTreeMap<String, SchemaProperty>>,
    pub(super) pattern_properties: Option<BTreeMap<String, SchemaProperty>>,
    pub(super) additional_properties: Option<bool>,
    pub(super) required: Vec<String>,
    pub(super) reference: Option<Reference>,
    pub(super) concrete: Option<Box<SchemaProperty>>,
    pub(super) field_groups: FieldGroups,
    pub(super) dropped_bare_alternatives: Vec<DroppedBareAlternative>,
    pub(super) dependencies: BTreeMap<String, ConditionalDependency>,
    pub(super) contains: Option<Box<SchemaProperty>>,
    pub(super) is_union: bool,
    pub(super) constrained_array_evidence: bool,
}

impl Projection {
    pub(super) fn map_bounds(&self) -> Result<(Option<i64>, Option<i64>), SchemaError> {
        let bound = |number: &Option<Number>, keyword: &str| {
            number
                .as_ref()
                .map(|number| {
                    number.as_i64().ok_or_else(|| {
                        self.emission_error(format!(
                            "{keyword} cannot fit the map projection exactly: {number}"
                        ))
                    })
                })
                .transpose()
        };
        Ok((
            bound(&self.min_properties, "minProperties")?,
            bound(&self.max_properties, "maxProperties")?,
        ))
    }

    fn with_required_groups(mut self, required: Vec<String>, field_groups: FieldGroups) -> Self {
        self.required = required;
        self.field_groups = field_groups;
        self
    }

    /// Lists expand their item grammar inline. A cycle consisting only of
    /// lists has no emitted named struct at which either renderer can stop.
    /// Struct recursion remains supported through named definitions.
    fn validate_array_emission(&self) -> Result<(), SchemaError> {
        if self.effective_type() == Some("array") && self.contains.is_some() {
            return Err(
                self.emission_error("contains requires existential array constraint emission")
            );
        }
        let mut projection = self;
        let mut seen = BTreeSet::new();
        while projection.effective_type() == Some("array") {
            let Some(items) = projection.items.as_deref() else {
                break;
            };
            if let Some(origin) = &items.origin
                && !seen.insert(origin.node)
            {
                return Err(projection.emission_error(
                    "recursive array projection requires a named collection definition",
                ));
            }
            projection = items.try_query()?;
        }
        Ok(())
    }

    fn validate_schema_dependencies(&self) -> Result<(), SchemaError> {
        if self.effective_type() != Some("object") {
            return Ok(());
        }
        for dependency in self.dependencies.values() {
            match dependency {
                ConditionalDependency::Schema(child) => {
                    return Err(child.try_query()?.emission_error(
                        "schema dependency requires conditional constraint emission",
                    ));
                }
                ConditionalDependency::PropertyList(_names) => {}
            }
        }
        Ok(())
    }

    pub(super) fn emission_error(&self, reason: impl Into<String>) -> SchemaError {
        SchemaError::Projection {
            pointer: self.source_pointer.clone(),
            reason: reason.into(),
        }
    }

    pub(super) fn value_type(&self) -> Result<EmissionType, SchemaError> {
        for literal in self
            .const_value
            .iter()
            .chain(self.enum_values.iter().flatten())
        {
            let agrees = match self.effective_json_type() {
                Some(JsonType::String) => literal.is_string(),
                Some(JsonType::Integer) | Some(JsonType::Number) => literal.as_i64().is_some(),
                Some(JsonType::Null)
                | Some(JsonType::Boolean)
                | Some(JsonType::Array)
                | Some(JsonType::Object) => false,
                None => true,
            };
            if !agrees {
                return Err(self.emission_error("literal conflicts with the projected value type"));
            }
        }
        match self.effective_json_type() {
            Some(JsonType::String) => Ok(EmissionType::String),
            Some(JsonType::Boolean) => Ok(EmissionType::Boolean),
            Some(JsonType::Integer) => Ok(EmissionType::Integer),
            Some(JsonType::Number) => Ok(EmissionType::Number),
            Some(JsonType::Array) => {
                if self.items.is_none() {
                    return Err(self.emission_error("array has no items projection"));
                }
                Ok(EmissionType::Array)
            }
            Some(JsonType::Object) => Ok(EmissionType::Object),
            Some(JsonType::Null) | None => {
                let literal = self
                    .const_value
                    .as_ref()
                    .or_else(|| self.enum_values.as_ref()?.first());
                match literal {
                    Some(Value::String(_value)) => Ok(EmissionType::String),
                    Some(Value::Number(value)) if value.as_i64().is_some() => {
                        Ok(EmissionType::Integer)
                    }
                    Some(Value::Number(_))
                    | Some(Value::Null)
                    | Some(Value::Bool(_))
                    | Some(Value::Array(_))
                    | Some(Value::Object(_))
                    | None => {
                        Err(self
                            .emission_error("value has no supported type or literal projection"))
                    }
                }
            }
        }
    }

    fn effective_json_type(&self) -> Option<JsonType> {
        match self.projected_type {
            ProjectedType::Unspecified => None,
            ProjectedType::Value(kind) => Some(kind),
            ProjectedType::JsonDocument => Some(JsonType::Object),
        }
    }

    pub(super) fn effective_type(&self) -> Option<&'static str> {
        self.effective_json_type().and_then(|kind| match kind {
            JsonType::Null => None,
            JsonType::Boolean => Some("boolean"),
            JsonType::Object => Some("object"),
            JsonType::Array => Some("array"),
            JsonType::Number => Some("number"),
            JsonType::Integer => Some("integer"),
            JsonType::String => Some("string"),
        })
    }

    #[cfg(test)]
    pub(super) fn constrained_array(&self) -> Result<bool, SchemaError> {
        fn property(
            property: &SchemaProperty,
            inherited_array: bool,
            visited: &mut BTreeSet<(NodeId, bool)>,
        ) -> Result<bool, SchemaError> {
            if let Some(origin) = &property.origin
                && !visited.insert((origin.node, inherited_array))
            {
                return Ok(false);
            }
            check(property.try_query()?, inherited_array, visited)
        }
        fn check(
            p: &Projection,
            inherited_array: bool,
            visited: &mut BTreeSet<(NodeId, bool)>,
        ) -> Result<bool, SchemaError> {
            let array =
                p.types.contains(&JsonType::Array) || (p.types.is_empty() && inherited_array);
            if p.constrained_array_evidence
                || (array
                    && (p.min_items.is_some()
                        || p.max_items.is_some()
                        || p.unique_items.is_some()
                        || p.contains.is_some()))
            {
                return Ok(true);
            }
            for dependency in p.dependencies.values() {
                match dependency {
                    ConditionalDependency::Schema(schema) => {
                        if property(schema, array, visited)? {
                            return Ok(true);
                        }
                        for child in schema
                            .try_query()?
                            .properties
                            .iter()
                            .flat_map(|p| p.values())
                        {
                            if property(child, false, visited)? {
                                return Ok(true);
                            }
                        }
                    }
                    ConditionalDependency::PropertyList(_names) => {}
                }
            }
            Ok(false)
        }
        check(self, false, &mut BTreeSet::new())
    }

    #[cfg(test)]
    pub(super) fn has_constrained_array(&self) -> bool {
        self.constrained_array()
            .expect("valid dependency projections")
    }

    pub(super) fn integer_bounds(&self) -> Result<(Option<i64>, Option<i64>), SchemaError> {
        let convert = |bound: &Option<Number>| {
            bound
                .as_ref()
                .map(|n| {
                    n.as_i64().ok_or_else(|| SchemaError::Projection {
                        pointer: self.source_pointer.clone(),
                        reason: format!("non-integral or overflowing integer bound {n}"),
                    })
                })
                .transpose()
        };
        let mut min = convert(&self.minimum)?;
        let mut max = convert(&self.maximum)?;
        for (number, lower) in [
            (&self.exclusive_minimum, true),
            (&self.exclusive_maximum, false),
        ] {
            if let Some(number) = number {
                let keyword = if lower {
                    "exclusiveMinimum"
                } else {
                    "exclusiveMaximum"
                };
                if self.effective_json_type() != Some(JsonType::Integer) {
                    return Err(self.emission_error(format!(
                        "{keyword} requires a strict non-integer bound API unavailable in pinned carina-core"
                    )));
                }
                let value = strict_integer_bound(number, lower).ok_or_else(|| {
                    self.emission_error(format!(
                        "{keyword} leaves no representable integer values: {number}"
                    ))
                })?;
                if lower {
                    min = Some(min.map_or(value, |old| old.max(value)));
                } else {
                    max = Some(max.map_or(value, |old| old.min(value)));
                }
            }
        }
        if min.zip(max).is_some_and(|(min, max)| min > max) {
            return Err(
                self.emission_error("empty integer interval after exclusive bound translation")
            );
        }
        Ok((min, max))
    }
}

thread_local! {
    static PROJECTING: std::cell::RefCell<BTreeSet<(usize,NodeId)>> = const {std::cell::RefCell::new(BTreeSet::new())};
}
struct QueryGuard((usize, NodeId));
impl Drop for QueryGuard {
    fn drop(&mut self) {
        PROJECTING.with(|active| {
            active.borrow_mut().remove(&self.0);
        });
    }
}

impl SchemaProperty {
    #[cfg(test)]
    pub(super) fn bind_fixture(&self, schema: &super::CfnSchema) {
        if self.origin.is_none() && self.projection.get().is_none() {
            self.projection.get_or_init(|| {
                let mut source=serde_json::json!({
                    "typeName":schema.type_name,
                    "properties":schema.properties,
                    "definitions":schema.definitions.as_ref().cloned().unwrap_or_default(),
                    "readOnlyProperties":schema.metadata().read_only_properties.clone().unwrap_or_default(),
                    "writeOnlyProperties":schema.metadata().write_only_properties.clone().unwrap_or_default(),
                    "createOnlyProperties":schema.metadata().create_only_properties.clone().unwrap_or_default()
                });
                source["properties"]["__ProjectionFixture"]=serde_json::to_value(self).expect("fixture property serializes");
                let layer=Arc::new(Layer::for_codegen(&source)?);
                let node=layer.sources().node_at("/properties/__ProjectionFixture").expect("fixture node");
                project(&layer,node)
            });
        }
    }

    pub(super) fn annotation_default(&self) -> Option<&Value> {
        if let Some(origin) = &self.origin {
            return origin.annotations.default.as_ref();
        }
        #[cfg(test)]
        if self.projection.get().is_none() {
            self.query();
        }
        self.projection
            .get()
            .and_then(|p| p.as_ref().ok())
            .and_then(|p| p.default_value.as_ref())
    }

    pub(super) fn annotation_description(&self) -> Option<&str> {
        if let Some(origin) = &self.origin {
            return origin.annotations.description.as_deref();
        }
        #[cfg(test)]
        if self.projection.get().is_none() {
            self.query();
        }
        self.projection
            .get()
            .and_then(|p| p.as_ref().ok())
            .and_then(|p| p.description.as_deref())
    }

    pub(super) fn try_query(&self) -> Result<&Projection, SchemaError> {
        if self.projection.get().is_none()
            && let Some(origin) = &self.origin
        {
            let key = (Arc::as_ptr(&origin.layer) as usize, origin.node);
            if PROJECTING.with(|active| active.borrow().contains(&key)) {
                return Err(error(
                    &origin.layer,
                    origin.node,
                    "recursive child conjunction requires a named intersection projection",
                ));
            }
        }
        self.projection.get_or_init(|| {
            if let Some(origin) = &self.origin {
                let key=(Arc::as_ptr(&origin.layer) as usize,origin.node);
                PROJECTING.with(|active| {active.borrow_mut().insert(key);});
                let _guard=QueryGuard(key);
                return project(&origin.layer, origin.node);
            }
            #[cfg(test)]
            {
                // Standalone legacy literals are test fixture input, not a production reader.
                let source = serde_json::json!({"typeName":"AWS::Test::Standalone", "properties":{"Value":self}});
                let layer = Arc::new(Layer::for_codegen(&source)?);
                let node = layer.sources().node_at("/properties/Value").expect("fixture node");
                direct(&layer, node)
            }
            #[cfg(not(test))]
            Err(SchemaError::Projection { pointer: String::new(), reason: "property has no Layer origin".into() })
        }).as_ref().map_err(Clone::clone)
    }

    #[cfg(test)]
    pub(super) fn query(&self) -> &Projection {
        self.try_query().unwrap_or_else(|error| panic!("{error}"))
    }
}

fn error(layer: &Layer, node: NodeId, reason: impl Into<String>) -> SchemaError {
    SchemaError::Projection {
        pointer: layer.sources().pointer(node).into(),
        reason: reason.into(),
    }
}

fn types(value: &Option<TypeSet>) -> Vec<JsonType> {
    let mut values = match value {
        Some(TypeSet::Single(kind)) => vec![*kind],
        Some(TypeSet::Multiple(kinds)) => kinds.clone(),
        None => Vec::new(),
    };
    values.sort();
    values.dedup();
    values
}

fn project_type(
    layer: &Layer,
    node: NodeId,
    types: &[JsonType],
) -> Result<ProjectedType, SchemaError> {
    let nonnull = types
        .iter()
        .copied()
        .filter(|kind| *kind != JsonType::Null)
        .collect::<Vec<_>>();
    match nonnull.as_slice() {
        [] if !types.is_empty() => Ok(ProjectedType::Value(JsonType::Null)),
        [] => Ok(ProjectedType::Unspecified),
        [kind] => Ok(ProjectedType::Value(*kind)),
        [JsonType::Object, JsonType::String] => Ok(ProjectedType::JsonDocument),
        unsupported => Err(error(
            layer,
            node,
            format!("unsupported multiple non-null types: {unsupported:?}"),
        )),
    }
}

pub(super) fn direct(layer: &Arc<Layer>, node: NodeId) -> Result<Projection, SchemaError> {
    match layer.node(node) {
        SchemaNode::Boolean(value) => Err(error(
            layer,
            node,
            format!("boolean schema {value} has no representable value projection"),
        )),
        SchemaNode::Keywords(k) => {
            let children = |nodes: &Option<BTreeMap<String, NodeId>>| {
                nodes.as_ref().map(|nodes| {
                    nodes
                        .iter()
                        .map(|(name, id)| (name.clone(), bridge(layer, *id)))
                        .collect()
                })
            };
            let item_bound =
                |bound: &Option<Number>, keyword: &str| -> Result<Option<i64>, SchemaError> {
                    match bound {
                        None => Ok(None),
                        Some(number) => number.as_i64().map(Some).ok_or_else(|| {
                            error(
                                layer,
                                node,
                                format!("{keyword} cannot fit the collection projection: {number}"),
                            )
                        }),
                    }
                };
            let length_bound = |bound: &Option<Number>,
                                keyword: &str|
             -> Result<Option<u64>, SchemaError> {
                match bound {
                    None => Ok(None),
                    Some(number) => number.as_u64().map(Some).ok_or_else(|| {
                        error(
                            layer,
                            node,
                            format!("{keyword} cannot fit the string projection exactly: {number}"),
                        )
                    }),
                }
            };
            let mut dependencies = BTreeMap::new();
            if let Some(entries) = &k.dependencies {
                for (name, dependency) in entries {
                    dependencies.insert(
                        name.clone(),
                        match dependency {
                            Dependency::Schema(id) => {
                                ConditionalDependency::Schema(Box::new(bridge(layer, *id)))
                            }
                            Dependency::PropertyList(names) => {
                                ConditionalDependency::PropertyList(names.clone())
                            }
                        },
                    );
                }
            }
            let types = if k.types.is_none() && k.properties.as_ref().is_some_and(|p| !p.is_empty())
            {
                vec![JsonType::Object]
            } else {
                types(&k.types)
            };
            Ok(Projection {
                source_pointer: layer.sources().pointer(node).to_string(),
                source_name: super::decode_json_pointer_segment(
                    layer
                        .sources()
                        .pointer(node)
                        .rsplit('/')
                        .next()
                        .unwrap_or(""),
                ),
                conjunction: false,
                projected_type: project_type(layer, node, &types)?,
                types,
                description: k.description.clone(),
                enum_values: k.enum_values.clone(),
                const_value: k.const_value.clone(),
                default_value: k.default.clone(),
                format: k.format.clone(),
                pattern: k.pattern.clone(),
                minimum: k.minimum.clone(),
                maximum: k.maximum.clone(),
                exclusive_minimum: k.exclusive_minimum.clone(),
                exclusive_maximum: k.exclusive_maximum.clone(),
                multiple_of: k.multiple_of.clone(),
                min_properties: k.min_properties.clone(),
                max_properties: k.max_properties.clone(),
                array_type: k.array_type,
                additional_schema: k.additional_properties.and_then(|id| match layer.node(id) {
                    SchemaNode::Boolean(_value) => None,
                    SchemaNode::Keywords(_keywords) => Some(Box::new(bridge(layer, id))),
                }),
                min_length: length_bound(&k.min_length, "minLength")?,
                max_length: length_bound(&k.max_length, "maxLength")?,
                min_items: item_bound(&k.min_items, "minItems")?,
                max_items: item_bound(&k.max_items, "maxItems")?,
                unique_items: k.unique_items,
                insertion_order: k.insertion_order,
                items: k.items.map(|id| Box::new(bridge(layer, id))),
                properties: children(&k.properties),
                pattern_properties: children(&k.pattern_properties),
                additional_properties: k.additional_properties.and_then(|id| {
                    match layer.node(id) {
                        SchemaNode::Boolean(value) => Some(*value),
                        SchemaNode::Keywords(_keywords) => None,
                    }
                }),
                required: k.required.clone().unwrap_or_default(),
                reference: None,
                concrete: None,
                field_groups: FieldGroups::default(),
                dropped_bare_alternatives: Vec::new(),
                dependencies,
                contains: k.contains.map(|id| Box::new(bridge(layer, id))),
                is_union: false,
                constrained_array_evidence: false,
            })
        }
    }
}

/// Query handles only carry their immutable Layer origin in production.
pub(super) fn bridge(layer: &Arc<Layer>, node: NodeId) -> SchemaProperty {
    #[cfg(test)]
    let mut property = {
        let mut property = materialize(direct(layer, node).unwrap_or_default());
        match layer.node(node) {
            SchemaNode::Boolean(_value) => {}
            SchemaNode::Keywords(k) => {
                property.ref_path = k.reference.as_ref().map(|r| r.spelling.clone());
                property.one_of = k
                    .compositions
                    .one_of
                    .iter()
                    .flatten()
                    .map(|id| bridge(layer, *id))
                    .collect();
                property.any_of = k
                    .compositions
                    .any_of
                    .iter()
                    .flatten()
                    .map(|id| bridge(layer, *id))
                    .collect();
            }
        }
        property
    };
    #[cfg(test)]
    {
        property.origin = Some(Origin::new(layer, node));
        property.projection = OnceLock::new();
        property
    }
    #[cfg(not(test))]
    SchemaProperty {
        origin: Some(Origin::new(layer, node)),
        projection: OnceLock::new(),
        materialized_tag_shape: OnceLock::new(),
    }
}

pub(super) fn materialize(p: Projection) -> SchemaProperty {
    #[cfg(test)]
    let type_names = p
        .types
        .iter()
        .map(|t| {
            match t {
                JsonType::Null => "null",
                JsonType::Boolean => "boolean",
                JsonType::Object => "object",
                JsonType::Array => "array",
                JsonType::Number => "number",
                JsonType::Integer => "integer",
                JsonType::String => "string",
            }
            .to_string()
        })
        .collect::<Vec<_>>();
    #[cfg(test)]
    let prop_type = if type_names.is_empty() {
        None
    } else if type_names.len() == 1 {
        Some(TypeValue::Single(type_names[0].clone()))
    } else {
        Some(TypeValue::Multiple(type_names))
    };
    #[cfg(test)]
    let enum_values = p.enum_values.as_ref().and_then(|values| {
        values
            .iter()
            .map(|v| match v {
                Value::String(s) => Some(EnumValue::Str(s.clone())),
                Value::Number(n) => n.as_i64().map(EnumValue::Int),
                Value::Null => None,
                Value::Bool(_boolean) => None,
                Value::Array(_array) => None,
                Value::Object(_object) => None,
            })
            .collect()
    });
    SchemaProperty {
        #[cfg(test)]
        prop_type,
        #[cfg(test)]
        description: p.description.clone(),
        #[cfg(test)]
        enum_values,
        #[cfg(test)]
        items: p.items.clone(),
        #[cfg(test)]
        ref_path: None,
        #[cfg(test)]
        insertion_order: p.insertion_order,
        #[cfg(test)]
        unique_items: p.unique_items,
        #[cfg(test)]
        properties: p.properties.clone(),
        #[cfg(test)]
        pattern_properties: p.pattern_properties.clone(),
        #[cfg(test)]
        required: p.required.clone(),
        #[cfg(test)]
        minimum: p.minimum.as_ref().and_then(Number::as_i64),
        #[cfg(test)]
        maximum: p.maximum.as_ref().and_then(Number::as_i64),
        #[cfg(test)]
        additional_properties: p.additional_properties,
        #[cfg(test)]
        const_value: p.const_value.clone(),
        #[cfg(test)]
        default_value: p.default_value.clone(),
        #[cfg(test)]
        pattern: p.pattern.clone(),
        #[cfg(test)]
        min_items: p.min_items,
        #[cfg(test)]
        max_items: p.max_items,
        #[cfg(test)]
        min_length: p.min_length,
        #[cfg(test)]
        max_length: p.max_length,
        #[cfg(test)]
        format: p.format.clone(),
        #[cfg(test)]
        one_of: Vec::new(),
        #[cfg(test)]
        any_of: Vec::new(),
        origin: None,
        projection: OnceLock::from(Ok(p)),
        materialized_tag_shape: OnceLock::new(),
    }
}

pub(super) fn project(layer: &Arc<Layer>, node: NodeId) -> Result<Projection, SchemaError> {
    let p = project_inner(layer, node, &mut Vec::new())?;
    Ok(complete_projection(p))
}

fn complete_projection(mut p: Projection) -> Projection {
    // A reference keeps its final definition identity, with ALL alias/site constraints.
    if let Some(reference) = &p.reference {
        let mut target = p.clone();
        target.reference = None;
        if target.effective_type() == Some("array") {
            let mut array = target.clone();
            array.is_union = false;
            target.concrete = Some(Box::new(materialize(array)));
        }
        let name = reference.name.clone();
        let path = reference.path.clone();
        let canonical = reference.canonical.clone();
        p.reference = Some(Reference {
            path,
            name,
            canonical,
            property: Box::new(materialize(target)),
        });
    }
    if p.effective_type() == Some("array") {
        let mut concrete = p.clone();
        concrete.reference = None;
        concrete.is_union = false;
        p.concrete = Some(Box::new(materialize(concrete)));
    }
    p
}

/// A productive reference/allOf SCC contributes each conjunct once. Returning
/// the neutral projection at a back edge is the fixed point of these idempotent
/// intersections; siblings are still applied on unwind. Pure alias cycles have
/// no value shape and fail. Identity is the lexicographically smallest named
/// definition in the SCC, independent of the entry alias or arm order.
fn cycle_projection(
    layer: &Arc<Layer>,
    node: NodeId,
    cycle: &[NodeId],
) -> Result<Projection, SchemaError> {
    let mut pending = vec![node];
    let mut seen = BTreeSet::new();
    let mut productive = false;
    while let Some(id) = pending.pop() {
        if !seen.insert(id) {
            continue;
        }
        match layer.node(id) {
            SchemaNode::Boolean(_value) => {}
            SchemaNode::Keywords(k) => {
                productive |= types(&k.types).iter().any(|kind| *kind != JsonType::Null)
                    || k.properties.as_ref().is_some_and(|p| !p.is_empty())
                    || k.enum_values.is_some()
                    || k.const_value.is_some();
                if let Some(reference) = &k.reference {
                    match &reference.target {
                        RefTarget::Node(target) => pending.push(*target),
                        RefTarget::Unresolved(_reason) => {}
                    }
                }
                pending.extend(k.compositions.all_of.iter().flatten().copied());
            }
        }
    }
    if !productive {
        return Err(error(
            layer,
            node,
            "unproductive reference/composition cycle",
        ));
    }
    let name = layer
        .resource()
        .definitions
        .iter()
        .flat_map(|defs| defs.iter())
        .filter(|(_name, id)| cycle.contains(id))
        .map(|(name, _id)| name.clone())
        .min()
        .unwrap_or_else(|| layer.sources().pointer(node).to_string());
    Ok(Projection {
        reference: Some(Reference {
            path: format!("#{}", layer.sources().pointer(node)),
            name,
            canonical: Box::new(bridge(layer, node)),
            property: Box::new(SchemaProperty::default()),
        }),
        ..Projection::default()
    })
}

fn project_inner(
    layer: &Arc<Layer>,
    node: NodeId,
    visiting: &mut Vec<NodeId>,
) -> Result<Projection, SchemaError> {
    if let Some(start) = visiting.iter().position(|id| *id == node) {
        return cycle_projection(layer, node, &visiting[start..]);
    }
    visiting.push(node);
    let mut p = direct(layer, node)?;
    if let SchemaNode::Keywords(k) = layer.node(node) {
        if let Some(reference) = &k.reference {
            let target = match &reference.target {
                RefTarget::Node(id) => *id,
                RefTarget::Unresolved(reason) => return Err(reason.clone()),
            };
            let q = project_inner(layer, target, visiting)?;
            let canonical = q.reference.as_ref().map_or_else(
                || Box::new(bridge(layer, target)),
                |reference| reference.canonical.clone(),
            );
            let name = q
                .reference
                .as_ref()
                .map_or_else(|| q.source_name.clone(), |r| r.name.clone());
            merge(layer, node, &mut p, q)?;
            // Filled by project() after every sibling has been applied.
            p.reference = Some(Reference {
                path: reference.spelling.clone(),
                name,
                canonical,
                property: Box::new(SchemaProperty::default()),
            });
        }
        p.conjunction |= k
            .compositions
            .all_of
            .as_ref()
            .is_some_and(|arms| !arms.is_empty());
        for id in k.compositions.all_of.iter().flatten() {
            let q = project_inner(layer, *id, visiting)?;
            merge(layer, node, &mut p, q)?;
        }
        for (kind, composition) in [
            (AlternativeKind::OneOf, &k.compositions.one_of),
            (AlternativeKind::AnyOf, &k.compositions.any_of),
        ] {
            if let Some(arms) = composition
                && !arms.is_empty()
            {
                let mut projected = Vec::new();
                for id in arms {
                    projected.push((*id, project_inner(layer, *id, visiting)?));
                }
                let (q, retained_reference) = match alternatives(layer, node, kind, projected)? {
                    AlternativeProjection::ConstrainedArm(q) => {
                        let reference = q.reference.clone();
                        (q, reference)
                    }
                    AlternativeProjection::EquivalentArms(q) => {
                        let reference = if q.effective_type() == Some("array") {
                            q.reference.clone()
                        } else {
                            None
                        };
                        (q, reference)
                    }
                    AlternativeProjection::FieldGroup(q) => (q, None),
                };
                let shape_union = !group_only(&q);
                merge(layer, node, &mut p, q)?;
                if p.reference.is_none() {
                    p.reference = retained_reference;
                }
                p.is_union |= shape_union;
            }
        }
    }
    visiting.pop();
    intersect_literals(&mut p)?;
    validate(layer, node, &p)?;
    Ok(p)
}

pub(super) fn merge(
    layer: &Arc<Layer>,
    node: NodeId,
    p: &mut Projection,
    q: Projection,
) -> Result<(), SchemaError> {
    if widens_closed_domain(p, &q) || widens_closed_domain(&q, p) {
        return Err(error(
            layer,
            node,
            "closed additionalProperties domain in allOf/$ref cannot admit fields from another conjunct",
        ));
    }
    if crosses_additional_schema_domain(p, &q) || crosses_additional_schema_domain(&q, p) {
        return Err(error(
            layer,
            node,
            "additionalProperties schema domain in allOf/$ref requires conjunct-local property emission",
        ));
    }
    p.concrete = None;
    if p.types.is_empty() {
        p.types = q.types.clone();
    } else if !q.types.is_empty() {
        let mut intersection = BTreeSet::new();
        for a in &p.types {
            for b in &q.types {
                if a == b {
                    intersection.insert(*a);
                } else if (*a == JsonType::Integer && *b == JsonType::Number)
                    || (*a == JsonType::Number && *b == JsonType::Integer)
                {
                    intersection.insert(JsonType::Integer);
                }
            }
        }
        p.types = intersection.into_iter().collect();
        if p.types.is_empty() {
            return Err(error(layer, node, "incompatible conjunctive types"));
        }
    }
    p.projected_type = project_type(layer, node, &p.types)?;
    macro_rules! same { ($($field:ident),+) => {$({
        if p.$field.is_some() && q.$field.is_some() && p.$field!=q.$field {return Err(error(layer,node,concat!("incompatible ",stringify!($field))));}
        if p.$field.is_none() {p.$field=q.$field.clone();}
    })+}; }
    same!(
        pattern,
        format,
        const_value,
        multiple_of,
        additional_properties,
        array_type
    );
    // A default annotates this site; it neither restricts accepted values nor
    // inherits from a reference target or another conjunction arm.
    for (target, source, lower) in [
        (&mut p.min_properties, q.min_properties, true),
        (&mut p.max_properties, q.max_properties, false),
    ] {
        if let Some(value) = source
            && target.as_ref().is_none_or(|old| {
                if lower {
                    number_cmp(&value, old).is_gt()
                } else {
                    number_cmp(&value, old).is_lt()
                }
            })
        {
            *target = Some(value);
        }
    }
    if let Some(child) = q.additional_schema {
        p.additional_schema = Some(Box::new(conjoin_child(
            layer,
            node,
            p.additional_schema.take().map(|p| *p),
            *child,
        )?));
    }
    p.description = p.description.clone().or(q.description.clone());
    if let Some(values) = q.enum_values {
        p.enum_values = Some(match p.enum_values.take() {
            Some(existing) => existing
                .into_iter()
                .filter(|v| values.contains(v))
                .collect(),
            None => values,
        });
    }
    macro_rules! bound {
        ($field:ident,$method:ident) => {
            p.$field = match (p.$field, q.$field) {
                (Some(a), Some(b)) => Some(a.$method(b)),
                (Some(a), None) => Some(a),
                (None, Some(b)) => Some(b),
                (None, None) => None,
            };
        };
    }
    bound!(min_length, max);
    bound!(max_length, min);
    bound!(min_items, max);
    bound!(max_items, min);
    for (target, source, lower) in [
        (&mut p.minimum, q.minimum, true),
        (&mut p.maximum, q.maximum, false),
        (&mut p.exclusive_minimum, q.exclusive_minimum, true),
        (&mut p.exclusive_maximum, q.exclusive_maximum, false),
    ] {
        if let Some(value) = source
            && target.as_ref().is_none_or(|old| {
                if lower {
                    number_cmp(&value, old).is_gt()
                } else {
                    number_cmp(&value, old).is_lt()
                }
            })
        {
            *target = Some(value);
        }
    }
    p.unique_items = match (p.unique_items, q.unique_items) {
        (Some(a), Some(b)) => Some(a || b),
        (Some(a), None) => Some(a),
        (None, Some(b)) => Some(b),
        (None, None) => None,
    };
    // Explicit ordering is a wire contract, not a validation bound. Conflicting
    // annotations cannot be silently replaced by target or sibling precedence.
    if p.insertion_order.is_some()
        && q.insertion_order.is_some()
        && p.insertion_order != q.insertion_order
    {
        return Err(error(layer, node, "incompatible insertionOrder"));
    }
    p.insertion_order = p.insertion_order.or(q.insertion_order);
    if let Some(items) = q.items {
        p.items = Some(Box::new(conjoin_child(
            layer,
            node,
            p.items.take().map(|p| *p),
            *items,
        )?));
    }
    for (target, source) in [
        (&mut p.properties, q.properties),
        (&mut p.pattern_properties, q.pattern_properties),
    ] {
        if let Some(properties) = source {
            let target = target.get_or_insert_with(BTreeMap::new);
            for (name, property) in properties {
                let combined = conjoin_child(layer, node, target.remove(&name), property)?;
                target.insert(name, combined);
            }
        }
    }
    for name in q.required {
        if !p.required.contains(&name) {
            p.required.push(name);
        }
    }
    p.field_groups.extend(q.field_groups);
    p.dropped_bare_alternatives
        .extend(q.dropped_bare_alternatives);
    for (name, dependency) in q.dependencies {
        if p.dependencies.insert(name, dependency).is_some() {
            return Err(error(
                layer,
                node,
                "overlapping dependency scopes require a conjunctive dependency projection",
            ));
        }
    }
    if p.contains.is_some() && q.contains.is_some() {
        return Err(error(
            layer,
            node,
            "multiple contains schemas require a conjunctive contains projection",
        ));
    }
    p.contains = p.contains.take().or(q.contains);
    p.conjunction |= q.conjunction;
    p.constrained_array_evidence |= q.constrained_array_evidence;
    p.is_union |= q.is_union;
    Ok(())
}

/// Closure belongs to the declaring schema, rather than the eventual union
/// of properties. Reject domain expansion before merging erases that scope.
fn widens_closed_domain(closed: &Projection, other: &Projection) -> bool {
    closed.additional_properties == Some(false)
        && [
            (&closed.properties, &other.properties),
            (&closed.pattern_properties, &other.pattern_properties),
        ]
        .into_iter()
        .any(|(domain, fields)| {
            fields.iter().flat_map(|fields| fields.keys()).any(|name| {
                domain
                    .as_ref()
                    .is_none_or(|domain| !domain.contains_key(name))
            })
        })
}

fn crosses_additional_schema_domain(domain: &Projection, other: &Projection) -> bool {
    domain.additional_schema.is_some()
        && [
            (&domain.properties, &other.properties),
            (&domain.pattern_properties, &other.pattern_properties),
        ]
        .into_iter()
        .any(|(declared, added)| {
            added.iter().flat_map(|fields| fields.keys()).any(|name| {
                declared
                    .as_ref()
                    .is_none_or(|fields| !fields.contains_key(name))
            })
        })
}

fn conjoin_child(
    layer: &Arc<Layer>,
    node: NodeId,
    existing: Option<SchemaProperty>,
    child: SchemaProperty,
) -> Result<SchemaProperty, SchemaError> {
    match existing {
        None => Ok(child),
        Some(existing) => {
            if let (Some(a), Some(b)) = (&existing.origin, &child.origin)
                && Arc::ptr_eq(&a.layer, &b.layer)
                && (a.node == b.node || a.layer.node(a.node) == b.layer.node(b.node))
            {
                return Ok(existing);
            }
            let mut p = existing.try_query()?.clone();
            let q = child.try_query()?.clone();
            if let (Some(a), Some(b)) = (&p.reference, &q.reference)
                && a.name != b.name
            {
                return Err(error(
                    layer,
                    node,
                    "conjunctive fields have incompatible definition identities",
                ));
            }
            let reference = p.reference.clone().or(q.reference.clone());
            merge(layer, node, &mut p, q)?;
            p.reference = reference;
            intersect_literals(&mut p)?;
            validate(layer, node, &p)?;
            Ok(materialize(complete_projection(p)))
        }
    }
}

enum AlternativeProjection {
    ConstrainedArm(Projection),
    EquivalentArms(Projection),
    FieldGroup(Projection),
}

fn alternatives(
    layer: &Arc<Layer>,
    node: NodeId,
    kind: AlternativeKind,
    arms: Vec<(NodeId, Projection)>,
) -> Result<AlternativeProjection, SchemaError> {
    if kind == AlternativeKind::OneOf {
        for (index, (_id, arm)) in arms.iter().enumerate() {
            for (_id, other) in arms.iter().skip(index + 1) {
                if matches!(
                    (arm.effective_json_type(), other.effective_json_type()),
                    (Some(JsonType::Number), Some(JsonType::Integer))
                        | (Some(JsonType::Integer), Some(JsonType::Number))
                ) {
                    return Err(error(
                        layer,
                        node,
                        "overlapping integer/number oneOf alternatives cannot be emitted",
                    ));
                }
            }
        }
    }
    // Object arms are field-group alternatives, even when one is bare. Only
    // different projected JSON types use the constrained-arm policy.
    let first_type = arms[0].1.effective_json_type();
    if arms
        .iter()
        .any(|(_id, p)| p.effective_json_type() != first_type)
    {
        let mut constrained = None;
        let mut dropped = Vec::new();
        for (arm, projection) in arms {
            if projection.effective_json_type().is_none() {
                return Err(error(
                    layer,
                    node,
                    "alternative arm has no projected JSON type",
                ));
            }
            if let Some(json_type) = bare_type(layer, arm) {
                dropped.push(DroppedBareAlternative {
                    kind,
                    arm,
                    json_type,
                });
            } else if constrained.replace(projection).is_some() {
                return Err(error(
                    layer,
                    node,
                    "different JSON types require exactly one constrained alternative arm",
                ));
            }
        }
        let mut result = constrained.ok_or_else(|| {
            error(
                layer,
                node,
                "different JSON types have no constrained alternative arm",
            )
        })?;
        result.dropped_bare_alternatives.extend(dropped);
        return Ok(AlternativeProjection::ConstrainedArm(result));
    }
    if kind == AlternativeKind::OneOf {
        for (index, (_id, arm)) in arms.iter().enumerate() {
            for (_id, other) in arms.iter().skip(index + 1) {
                if equivalent(arm, other, &mut BTreeSet::new())? {
                    return Err(error(
                        layer,
                        node,
                        "overlapping equivalent oneOf alternatives require exclusive constraint emission",
                    ));
                }
            }
        }
    }
    let mut arms = arms.into_iter().map(|(_id, p)| p).collect::<Vec<_>>();
    let mut first = arms.remove(0);
    if group_only(&first) && arms.iter().all(group_only) {
        if !first.field_groups.groups.is_empty()
            || arms.iter().any(|p| !p.field_groups.groups.is_empty())
        {
            for other in arms {
                if !equivalent(&first, &other, &mut BTreeSet::new())? {
                    return Err(error(
                        layer,
                        node,
                        "conflicting nested required alternatives",
                    ));
                }
                first.field_groups.extend(other.field_groups);
                first
                    .dropped_bare_alternatives
                    .extend(other.dropped_bare_alternatives);
            }
            return Ok(AlternativeProjection::FieldGroup(first));
        }
        let mut declarations = first.field_groups;
        let mut requirements = vec![first.required];
        for arm in arms {
            declarations.extend(arm.field_groups);
            requirements.push(arm.required);
        }
        let (required, mut field_groups) =
            FieldGroups::project(kind, requirements, layer.sources().pointer(node))?;
        field_groups.extend(declarations);
        let result = Projection::default().with_required_groups(required, field_groups);
        return Ok(AlternativeProjection::FieldGroup(result));
    }
    let any_array = first.effective_type() == Some("array")
        || arms.iter().any(|p| p.effective_type() == Some("array"));
    if any_array {
        for other in arms {
            if !equivalent(&first, &other, &mut BTreeSet::new())? {
                return Err(error(
                    layer,
                    node,
                    "conflicting alternative array projections (shape, children, bounds, order, or uniqueness)",
                ));
            }
            first.field_groups.extend(other.field_groups);
            first
                .dropped_bare_alternatives
                .extend(other.dropped_bare_alternatives);
        }
        return Ok(AlternativeProjection::EquivalentArms(first));
    }
    if first.effective_type() == Some("object")
        && arms.iter().all(|p| p.effective_type() == Some("object"))
    {
        // A closed singleton arm's domain is enforced by exactly-one itself
        // (for example, two alternative object fields). Optional branch-local
        // fields and inclusive alternatives need constraints we cannot emit.
        let exclusive_single_fields = kind == AlternativeKind::OneOf
            && std::iter::once(&first).chain(&arms).all(|arm| {
                arm.additional_properties == Some(false)
                    && arm.required.len() == 1
                    && arm.properties.as_ref().is_some_and(|fields| {
                        fields.len() == 1 && fields.contains_key(&arm.required[0])
                    })
                    && arm
                        .pattern_properties
                        .as_ref()
                        .is_none_or(BTreeMap::is_empty)
            });
        for arm in std::iter::once(&first).chain(&arms) {
            if !exclusive_single_fields
                && std::iter::once(&first)
                    .chain(&arms)
                    .any(|other| widens_closed_domain(arm, other))
            {
                let keyword = match kind {
                    AlternativeKind::OneOf => "oneOf",
                    AlternativeKind::AnyOf => "anyOf",
                };
                return Err(error(
                    layer,
                    node,
                    format!(
                        "{keyword} closed additionalProperties branch domains require branch-local object emission"
                    ),
                ));
            }
        }
        let requirements = std::iter::once(&first)
            .chain(&arms)
            .map(|arm| arm.required.clone())
            .collect();
        let (required, field_groups) =
            FieldGroups::project(kind, requirements, layer.sources().pointer(node))?;
        // Only equivalent field domains can be hoisted out of open branches.
        // The containing struct owns the resulting presence group.
        let common_facts = |p: &Projection| {
            let mut common = p.clone();
            common.properties = None;
            common.required.clear();
            common.reference = None;
            common
        };
        let common = common_facts(&first);
        if !exclusive_single_fields {
            for other in &arms {
                let mut first_domain = first.clone();
                let mut other_domain = other.clone();
                first_domain.required.clear();
                other_domain.required.clear();
                first_domain.field_groups = FieldGroups::default();
                other_domain.field_groups = FieldGroups::default();
                first_domain.reference = None;
                other_domain.reference = None;
                if !equivalent(&first_domain, &other_domain, &mut BTreeSet::new())? {
                    let keyword = match kind {
                        AlternativeKind::OneOf => "oneOf",
                        AlternativeKind::AnyOf => "anyOf",
                    };
                    return Err(error(
                        layer,
                        node,
                        format!(
                            "{keyword} object field domains require equivalent branch-local constraints"
                        ),
                    ));
                }
            }
        }
        let mut result = first;
        result.required.clear();
        for other in arms {
            let mut other = other;
            if !equivalent(&common, &common_facts(&other), &mut BTreeSet::new())? {
                return Err(error(
                    layer,
                    node,
                    "conflicting alternative object constraints",
                ));
            }
            other.required.clear();
            result.field_groups.extend(other.field_groups);
            result
                .dropped_bare_alternatives
                .extend(other.dropped_bare_alternatives);
            // Fields are optional at the containing alternative scope. Duplicate
            // fields must agree; intersection would strengthen an alternative.
            if let Some(properties) = other.properties.take() {
                let target = result.properties.get_or_insert_with(BTreeMap::new);
                for (name, property) in properties {
                    if let Some(existing) = target.get(&name) {
                        if !equivalent(
                            existing.try_query()?,
                            property.try_query()?,
                            &mut BTreeSet::new(),
                        )? {
                            return Err(error(
                                layer,
                                node,
                                format!("conflicting alternative field {name}"),
                            ));
                        }
                    } else {
                        target.insert(name, property);
                    }
                }
            }
        }
        result.required = required;
        result.field_groups.extend(field_groups);
        return Ok(AlternativeProjection::FieldGroup(result));
    }
    for other in arms {
        if !equivalent(&first, &other, &mut BTreeSet::new())? {
            return Err(error(
                layer,
                node,
                "conflicting alternative scalar projections",
            ));
        }
        first.field_groups.extend(other.field_groups);
        first
            .dropped_bare_alternatives
            .extend(other.dropped_bare_alternatives);
    }
    Ok(AlternativeProjection::EquivalentArms(first))
}

fn group_only(p: &Projection) -> bool {
    p.types.is_empty()
        && p.properties.is_none()
        && p.pattern_properties.is_none()
        && p.items.is_none()
        && p.minimum.is_none()
        && p.maximum.is_none()
        && p.exclusive_minimum.is_none()
        && p.exclusive_maximum.is_none()
        && p.multiple_of.is_none()
        && p.min_length.is_none()
        && p.max_length.is_none()
        && p.pattern.is_none()
        && p.format.is_none()
        && p.enum_values.is_none()
        && p.const_value.is_none()
        && p.default_value.is_none()
        && p.array_type.is_none()
        && p.min_items.is_none()
        && p.max_items.is_none()
        && p.unique_items.is_none()
        && p.insertion_order.is_none()
        && p.min_properties.is_none()
        && p.max_properties.is_none()
        && p.dependencies.is_empty()
        && p.contains.is_none()
        && p.additional_properties.is_none()
        && p.additional_schema.is_none()
}

fn bare_type(layer: &Layer, node: NodeId) -> Option<JsonType> {
    match layer.node(node) {
        SchemaNode::Boolean(_value) => None,
        SchemaNode::Keywords(k) => {
            let spelling = match &k.types {
                Some(TypeSet::Single(kind)) => TypeSet::Single(*kind),
                Some(TypeSet::Multiple(kinds)) => TypeSet::Multiple(kinds.clone()),
                None => return None,
            };
            let nonnull = types(&k.types)
                .into_iter()
                .filter(|kind| *kind != JsonType::Null)
                .collect::<Vec<_>>();
            let [kind] = nonnull.as_slice() else {
                return None;
            };
            // Compare the typed source, not the flattened projection: even an
            // empty properties/items schema, a reference, or an annotation
            // makes this more than a bare type spelling.
            let type_only = PropertyKeywords {
                types: Some(spelling),
                ..PropertyKeywords::default()
            };
            (k.as_ref() == &type_only).then_some(*kind)
        }
    }
}

fn equivalent_scalar(p: &Projection, q: &Projection) -> bool {
    p.types == q.types
        && p.projected_type == q.projected_type
        && p.enum_values == q.enum_values
        && p.const_value == q.const_value
        && p.minimum == q.minimum
        && p.maximum == q.maximum
        && p.exclusive_minimum == q.exclusive_minimum
        && p.exclusive_maximum == q.exclusive_maximum
        && p.multiple_of == q.multiple_of
        && p.format == q.format
        && p.pattern == q.pattern
        && p.min_length == q.min_length
        && p.max_length == q.max_length
        && p.min_items == q.min_items
        && p.max_items == q.max_items
        && p.min_properties == q.min_properties
        && p.max_properties == q.max_properties
        && p.array_type == q.array_type
        && p.unique_items == q.unique_items
        && p.insertion_order == q.insertion_order
        && p.additional_properties == q.additional_properties
}
fn equivalent(
    p: &Projection,
    q: &Projection,
    seen: &mut BTreeSet<(NodeId, NodeId)>,
) -> Result<bool, SchemaError> {
    equivalent_with_identity(p, q, seen, true)
}

pub(super) fn equivalent_definition_projection(
    p: &Projection,
    q: &Projection,
) -> Result<bool, SchemaError> {
    // A reference site's default annotates the containing field, not its type.
    // Compare the definition body without that outer annotation; defaults on
    // descendants still belong to their fields inside the body.
    equivalent_body(p, q, &mut BTreeSet::new(), true)
}

pub(super) fn equivalent_tag_projection(
    p: &Projection,
    q: &Projection,
) -> Result<bool, SchemaError> {
    equivalent_with_identity(p, q, &mut BTreeSet::new(), false)
}

fn equivalent_with_identity(
    p: &Projection,
    q: &Projection,
    seen: &mut BTreeSet<(NodeId, NodeId)>,
    identity: bool,
) -> Result<bool, SchemaError> {
    if p.default_value != q.default_value {
        return Ok(false);
    }
    equivalent_body(p, q, seen, identity)
}

fn equivalent_body(
    p: &Projection,
    q: &Projection,
    seen: &mut BTreeSet<(NodeId, NodeId)>,
    identity: bool,
) -> Result<bool, SchemaError> {
    if identity && p.reference.as_ref().map(|r| &r.name) != q.reference.as_ref().map(|r| &r.name) {
        return Ok(false);
    }
    if !equivalent_scalar(p, q)
        || p.required != q.required
        // Declaration membership is precedence evidence, not a value
        // constraint. Equivalent alternatives merge that evidence separately.
        || p.field_groups.groups != q.field_groups.groups
        || p.dependencies.len() != q.dependencies.len()
    {
        return Ok(false);
    }
    let mut child_equal = |a: &SchemaProperty, b: &SchemaProperty| -> Result<bool, SchemaError> {
        if let (Some(a), Some(b)) = (&a.origin, &b.origin)
            && !seen.insert((a.node, b.node))
        {
            return Ok(true);
        }
        equivalent_with_identity(a.try_query()?, b.try_query()?, seen, identity)
    };
    for (a, b) in [
        (&p.items, &q.items),
        (&p.contains, &q.contains),
        (&p.additional_schema, &q.additional_schema),
    ] {
        match (a, b) {
            (Some(a), Some(b)) => {
                if !child_equal(a, b)? {
                    return Ok(false);
                }
            }
            (None, None) => {}
            (Some(_a), None) => return Ok(false),
            (None, Some(_b)) => return Ok(false),
        }
    }
    for (name, a) in &p.dependencies {
        let Some(b) = q.dependencies.get(name) else {
            return Ok(false);
        };
        match (a, b) {
            (ConditionalDependency::PropertyList(a), ConditionalDependency::PropertyList(b)) => {
                if a != b {
                    return Ok(false);
                }
            }
            (ConditionalDependency::Schema(a), ConditionalDependency::Schema(b)) => {
                if !child_equal(a, b)? {
                    return Ok(false);
                }
            }
            (ConditionalDependency::Schema(_a), ConditionalDependency::PropertyList(_b)) => {
                return Ok(false);
            }
            (ConditionalDependency::PropertyList(_a), ConditionalDependency::Schema(_b)) => {
                return Ok(false);
            }
        }
    }
    for (a, b) in [
        (&p.properties, &q.properties),
        (&p.pattern_properties, &q.pattern_properties),
    ] {
        match (a, b) {
            (Some(a), Some(b)) => {
                if a.len() != b.len() {
                    return Ok(false);
                }
                for (name, child) in a {
                    let Some(other) = b.get(name) else {
                        return Ok(false);
                    };
                    if !child_equal(child, other)? {
                        return Ok(false);
                    }
                }
            }
            (None, None) => {}
            (Some(_a), None) => return Ok(false),
            (None, Some(_b)) => return Ok(false),
        }
    }
    Ok(true)
}

// Compare canonical JSON decimals directly. Converting an integer endpoint to
// f64 would make 9007199254740993 equal to 9007199254740992.0 and make an
// allOf intersection depend on arm order. No endpoint is rounded or truncated.
fn decimal_number(number: &Number) -> (bool, String, i32) {
    let spelling = number.to_string();
    let negative = spelling.starts_with('-');
    let magnitude = spelling.strip_prefix('-').unwrap_or(&spelling);
    let (mantissa, exponent) =
        magnitude
            .split_once(['e', 'E'])
            .map_or((magnitude, 0), |(digits, exponent)| {
                (
                    digits,
                    exponent
                        .parse::<i32>()
                        .expect("finite JSON number exponent"),
                )
            });
    let fraction = mantissa
        .split_once('.')
        .map_or(0, |(_whole, fraction)| fraction.len() as i32);
    let digits = mantissa
        .replace('.', "")
        .trim_start_matches('0')
        .to_string();
    if digits.is_empty() {
        (false, "0".into(), 0)
    } else {
        (negative, digits, exponent - fraction)
    }
}

fn number_cmp(a: &Number, b: &Number) -> std::cmp::Ordering {
    let (negative_a, digits_a, power_a) = decimal_number(a);
    let (negative_b, digits_b, power_b) = decimal_number(b);
    if negative_a != negative_b {
        return if negative_a {
            std::cmp::Ordering::Less
        } else {
            std::cmp::Ordering::Greater
        };
    }
    if digits_a == "0" && digits_b == "0" {
        return std::cmp::Ordering::Equal;
    }
    if digits_a == "0" {
        return std::cmp::Ordering::Less;
    }
    if digits_b == "0" {
        return std::cmp::Ordering::Greater;
    }
    let magnitude_a = digits_a.len() as i32 + power_a;
    let magnitude_b = digits_b.len() as i32 + power_b;
    let length = digits_a.len().max(digits_b.len());
    let ordering = magnitude_a.cmp(&magnitude_b).then_with(|| {
        digits_a
            .chars()
            .chain(std::iter::repeat('0'))
            .take(length)
            .cmp(digits_b.chars().chain(std::iter::repeat('0')).take(length))
    });
    if negative_a {
        ordering.reverse()
    } else {
        ordering
    }
}

/// Search the integer lattice with decimal comparisons, including fractional
/// and exponent endpoints. Never round an i64 endpoint through f64.
fn strict_integer_bound(number: &Number, lower: bool) -> Option<i64> {
    if (lower && !number_cmp(number, &Number::from(i64::MAX)).is_lt())
        || (!lower && !number_cmp(number, &Number::from(i64::MIN)).is_gt())
    {
        return None;
    }
    let mut lo = i128::from(i64::MIN);
    let mut hi = i128::from(i64::MAX);
    while lo < hi {
        let mid = lo + (hi - lo + i128::from(!lower)) / 2;
        let candidate = i64::try_from(mid).expect("search stays inside the i64 domain");
        let order = number_cmp(&Number::from(candidate), number);
        if lower {
            if order.is_gt() {
                hi = mid;
            } else {
                lo = mid + 1;
            }
        } else if order.is_lt() {
            lo = mid;
        } else {
            hi = mid - 1;
        }
    }
    i64::try_from(lo).ok()
}

/// Finite literal domains can absorb scalar constraints exactly before enum
/// emission. Evaluate the source regex with JSON Schema's substring semantics;
/// a finite enum no longer needs a separate pattern/range validator.
/// Literal kind compatibility is checked by `Projection::value_type` at the
/// emission boundary, preserving unsupported-literal diagnostics.
pub(super) fn intersect_literals(p: &mut Projection) -> Result<(), SchemaError> {
    if p.const_value.is_none() && p.enum_values.is_none() {
        return Ok(());
    }
    let pattern = p
        .pattern
        .as_ref()
        .filter(|_pattern| {
            p.const_value.as_ref().is_some_and(Value::is_string)
                || p.enum_values
                    .as_ref()
                    .is_some_and(|values| values.iter().any(Value::is_string))
        })
        .map(|pattern| {
            carina_core::schema::CompiledPattern::try_new(pattern).map_err(|error| {
                p.emission_error(format!(
                    "literal pattern intersection is unsupported: {error}"
                ))
            })
        })
        .transpose()?;
    let accepts = |value: &Value| -> Result<bool, SchemaError> {
        match value {
            Value::String(value) => {
                if p.format.is_some() {
                    return Err(p.emission_error(
                        "literal format intersection requires a format validator",
                    ));
                }
                let length = value.chars().count() as u64;
                Ok(p.min_length.is_none_or(|min| length >= min)
                    && p.max_length.is_none_or(|max| length <= max)
                    && pattern
                        .as_ref()
                        .is_none_or(|pattern| pattern.is_match(value)))
            }
            Value::Number(value) => {
                for (bound, lower, strict) in [
                    (&p.minimum, true, false),
                    (&p.maximum, false, false),
                    (&p.exclusive_minimum, true, true),
                    (&p.exclusive_maximum, false, true),
                ] {
                    if let Some(bound) = bound {
                        let order = number_cmp(value, bound);
                        if (lower && order.is_lt())
                            || (!lower && order.is_gt())
                            || (strict && order.is_eq())
                        {
                            return Ok(false);
                        }
                    }
                }
                if let Some(multiple) = &p.multiple_of {
                    let integer = value.as_i64().ok_or_else(|| {
                        p.emission_error("non-integral literal multipleOf intersection")
                    })?;
                    let (_negative, digits, power) = decimal_number(multiple);
                    let divisor = digits.parse::<u128>().map_err(|error| {
                        p.emission_error(format!(
                            "literal multipleOf intersection is unsupported: {error}"
                        ))
                    })?;
                    if divisor == 0 {
                        return Err(p.emission_error("literal multipleOf must be positive"));
                    }
                    let mut magnitude = u128::from(integer.unsigned_abs());
                    if power >= 0 {
                        for _power in 0..power {
                            if !magnitude.is_multiple_of(10) {
                                return Ok(false);
                            }
                            magnitude /= 10;
                        }
                        Ok(magnitude.is_multiple_of(divisor))
                    } else {
                        let mut remainder = magnitude % divisor;
                        for _power in power..0 {
                            remainder = (remainder * 10) % divisor;
                        }
                        Ok(remainder == 0)
                    }
                } else {
                    Ok(true)
                }
            }
            Value::Null => Ok(true),
            Value::Bool(_value) => Ok(true),
            Value::Array(_value) => Ok(true),
            Value::Object(_value) => Ok(true),
        }
    };
    if let Some(value) = &p.const_value
        && !accepts(value)?
    {
        return Err(
            p.emission_error("empty literal intersection for const and effective constraints")
        );
    }
    if let Some(values) = &p.enum_values {
        let mut retained = Vec::new();
        for value in values {
            if accepts(value)? {
                retained.push(value.clone());
            }
        }
        if retained.is_empty() {
            return Err(
                p.emission_error("empty literal intersection for enum and effective constraints")
            );
        }
        p.enum_values = Some(retained);
    }
    Ok(())
}

pub(super) fn validate(layer: &Layer, node: NodeId, p: &Projection) -> Result<(), SchemaError> {
    for dropped in &p.dropped_bare_alternatives {
        if bare_type(layer, dropped.arm) != Some(dropped.json_type) {
            return Err(error(
                layer,
                node,
                format!("invalid {:?} bare alternative provenance", dropped.kind),
            ));
        }
    }
    for (min, max, name) in [
        (p.min_length, p.max_length, "length"),
        (
            p.min_items.and_then(|n| u64::try_from(n).ok()),
            p.max_items.and_then(|n| u64::try_from(n).ok()),
            "items",
        ),
    ] {
        if min.zip(max).is_some_and(|(min, max)| min > max) {
            return Err(error(layer, node, format!("incompatible {name} bounds")));
        }
    }
    for (lower, lower_strict) in p
        .minimum
        .as_ref()
        .map(|n| (n, false))
        .into_iter()
        .chain(p.exclusive_minimum.as_ref().map(|n| (n, true)))
    {
        for (upper, upper_strict) in p
            .maximum
            .as_ref()
            .map(|n| (n, false))
            .into_iter()
            .chain(p.exclusive_maximum.as_ref().map(|n| (n, true)))
        {
            let order = number_cmp(lower, upper);
            if order.is_gt() || (order.is_eq() && (lower_strict || upper_strict)) {
                return Err(error(layer, node, "incompatible numeric bounds"));
            }
        }
    }
    if p.min_properties
        .as_ref()
        .zip(p.max_properties.as_ref())
        .is_some_and(|(a, b)| number_cmp(a, b).is_gt())
    {
        return Err(error(layer, node, "incompatible property-count bounds"));
    }
    if let Some(value) = &p.const_value {
        if p.enum_values
            .as_ref()
            .is_some_and(|values| !values.contains(value))
        {
            return Err(error(
                layer,
                node,
                "incompatible const and enum constraints",
            ));
        }
        if p.effective_type() == Some("array") || p.effective_type() == Some("object") {
            return Err(error(
                layer,
                node,
                "scalar constant conflicts with collection shape",
            ));
        }
        match value {
            Value::String(_value) => {}
            Value::Number(value) if value.as_i64().is_some() => {}
            Value::Number(_value) => {
                return Err(error(
                    layer,
                    node,
                    "non-integral const requires a typed literal projection",
                ));
            }
            Value::Bool(_value) => {
                return Err(error(
                    layer,
                    node,
                    "boolean const requires a typed literal projection",
                ));
            }
            Value::Null => {
                return Err(error(
                    layer,
                    node,
                    "null const requires a typed literal projection",
                ));
            }
            Value::Array(_value) => {
                return Err(error(
                    layer,
                    node,
                    "array const requires a typed literal projection",
                ));
            }
            Value::Object(_value) => {
                return Err(error(
                    layer,
                    node,
                    "object const requires a typed literal projection",
                ));
            }
        }
    }
    if let Some(values) = &p.enum_values
        && !(values.iter().all(Value::is_string) || values.iter().all(|v| v.as_i64().is_some()))
    {
        return Err(error(
            layer,
            node,
            "mixed or non-integral enum requires a typed literal projection",
        ));
    }
    if p.enum_values.as_ref().is_some_and(Vec::is_empty) {
        return Err(error(layer, node, "empty enum intersection"));
    }
    Ok(())
}

/// Match core admission against the containing scope's emitted siblings,
/// including filters, per-use provider modes and local/override defaults.
pub(super) fn validate_emitted_field_groups(
    schema: &super::CfnSchema,
    projection: &Projection,
    groups: &FieldGroups,
    parent: Option<&str>,
    provider_path: &[String],
    provider_populated: bool,
) -> Result<(), SchemaError> {
    groups.validate_members(|member| {
        let reject = |reason: &str| {
            Err(projection.emission_error(format!(
                "unsupported required alternatives: member {member} {reason}"
            )))
        };
        let Some(property) = projection
            .properties
            .as_ref()
            .and_then(|fields| fields.get(member))
        else {
            return reject("must be an emitted optional sibling field");
        };
        property.try_query()?;
        if super::is_deprecated_property(property) {
            return reject("is deprecated and excluded from emitted fields");
        }
        let cfn_path = super::CfnPropertyPath::new(parent, member);
        if super::should_omit_property(&schema.type_name, cfn_path) {
            return reject("is omitted from emitted fields");
        }
        let path = super::extend_property_path(provider_path, member);
        if provider_populated || super::is_property_read_only(schema, property, &path)? {
            return reject("is provider_populated; an optional input sibling is required");
        }
        if projection
            .required
            .iter()
            .any(|required| required == member)
        {
            return reject("is required; an optional sibling field is required");
        }
        if property.annotation_default().is_some()
            || super::resource_type_override(&schema.type_name, cfn_path)
                .and_then(super::TypeOverride::default)
                .is_some()
        {
            return reject("is defaulted; a group member must have no default");
        }
        Ok(())
    })
}

/// Query validation precedes either renderer. A renderer cannot turn a failed
/// query into a fallback String, an inferred type, or a panic at the CLI boundary.
/// Tag properties use the separately cached TagShape admission.
pub(super) fn validate_shared_queries(schema: &super::CfnSchema) -> anyhow::Result<()> {
    #[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
    enum ValidationSource {
        Node(NodeId),
        Materialized(usize),
    }
    type ValidationStates = BTreeSet<(ValidationSource, String, bool, Vec<Vec<String>>)>;

    fn visit(
        schema: &super::CfnSchema,
        property: &SchemaProperty,
        name: &str,
        provider_path: &[String],
        provider_populated: bool,
        active: &mut ValidationStates,
        canonical: &mut BTreeSet<(String, String)>,
    ) -> Result<(), SchemaError> {
        let p = property.try_query()?;
        let emitted = super::emission_plan::lower(p)?;
        if let Some(reference) = &p.reference {
            emitted.validate_canonical_domains(
                &super::emission_plan::lower(reference.canonical.try_query()?)?,
                canonical,
            )?;
        }
        p.validate_array_emission()?;
        p.validate_schema_dependencies()?;
        let scope_name = p
            .reference
            .as_ref()
            .map_or(name, |reference| reference.name.as_str());
        emitted.validate_required_fields(&p.source_pointer, &schema.type_name, Some(scope_name))?;
        let provider_populated =
            provider_populated || super::is_property_read_only(schema, property, provider_path)?;
        validate_emitted_field_groups(
            schema,
            p,
            &emitted.scope.field_groups,
            Some(scope_name),
            provider_path,
            provider_populated,
        )?;
        // Path-dependent admission is determined by the inherited mode and
        // the read-only paths relative to this use. Memoizing only by NodeId
        // would miss invalid input groups at another use. Keeping these facts
        // in the state also terminates recursive shared definitions without
        // enumerating exponentially many equivalent provider paths.
        let source = property.origin.as_ref().map_or_else(
            || ValidationSource::Materialized(property as *const SchemaProperty as usize),
            |origin| ValidationSource::Node(origin.node),
        );
        let mut read_only = schema
            .metadata()
            .read_only_properties
            .iter()
            .flatten()
            .filter_map(|path| super::cfn_property_path(path))
            .filter_map(|path| path.strip_prefix(provider_path).map(<[String]>::to_vec))
            .collect::<Vec<_>>();
        read_only.sort();
        if !active.insert((
            source,
            scope_name.to_string(),
            provider_populated,
            read_only,
        )) {
            return Ok(());
        }
        if matches!(p.effective_type(), Some("integer" | "number"))
            && p.enum_values.is_none()
            && p.const_value.is_none()
        {
            p.integer_bounds()?;
        }
        for child in emitted
            .array
            .iter()
            .map(|array| array.items)
            .chain(emitted.map.iter().map(|map| map.value))
        {
            let path = super::extend_property_path(provider_path, "*");
            visit(
                schema,
                child,
                name,
                &path,
                provider_populated,
                active,
                canonical,
            )?;
        }
        if let Some(properties) = emitted.properties {
            for (name, child) in properties {
                if super::is_deprecated_property(child) {
                    continue;
                }
                // Whole-attribute and item replacements discard child grammar,
                // but their own source shape and collection constraints are
                // validated separately by the override registry.
                let path = super::CfnPropertyPath::new(Some(scope_name), name);
                if super::should_omit_property(&schema.type_name, path) {
                    continue;
                }
                let override_type = super::resource_type_override(&schema.type_name, path);
                let child_projection = child.try_query()?;
                if override_type
                    .is_some_and(|o| o.string_binding().is_some() || o.enum_values().is_some())
                {
                    child_projection.validate_array_emission()?;
                }
                if override_type.is_some_and(|o| {
                    o.attribute_type_expression().is_some()
                        || o.array_item_type_expression().is_some()
                        || o.enum_values().is_some()
                }) {
                    continue;
                }
                let path = super::extend_property_path(provider_path, name);
                visit(
                    schema,
                    child,
                    name,
                    &path,
                    provider_populated,
                    active,
                    canonical,
                )?;
            }
        }
        Ok(())
    }
    schema.validate_root_emission()?;
    let root = schema.root_projection()?;
    let emitted = super::emission_plan::lower_as(root, super::emission_plan::EmittedKind::Root)?;
    emitted.validate_required_fields(&root.source_pointer, &schema.type_name, None)?;
    root.validate_schema_dependencies()?;
    validate_emitted_field_groups(schema, root, &root.field_groups, None, &[], false)?;
    let mut active = BTreeSet::new();
    let mut canonical = BTreeSet::new();
    for (name, property) in super::documented_top_level_properties(schema) {
        if super::is_dsl_tags_property(schema, std::slice::from_ref(name))? {
            continue;
        }
        let path = super::CfnPropertyPath::new(None, name);
        let override_type = super::resource_type_override(&schema.type_name, path);
        let projection = property.try_query()?;
        if override_type.is_some_and(|o| o.string_binding().is_some() || o.enum_values().is_some())
        {
            projection.validate_array_emission()?;
        }
        if override_type.is_some_and(|o| {
            o.attribute_type_expression().is_some()
                || o.array_item_type_expression().is_some()
                || o.enum_values().is_some()
        }) {
            continue;
        }
        visit(
            schema,
            property,
            name,
            std::slice::from_ref(name),
            false,
            &mut active,
            &mut canonical,
        )?;
    }
    Ok(())
}

pub(super) struct DefinitionEnumFields {
    pub(super) source_name: String,
    pub(super) name: String,
    pub(super) properties: BTreeMap<String, SchemaProperty>,
}

/// Discover declared definition fields from the Layer, including unconditional
/// allOf fields. Pure aliases reuse their target's identity instead of creating
/// a second set of enum identities. Both renderers share these resolved fields.
pub(super) fn definition_enum_fields(
    schema: &super::CfnSchema,
    emitted: &BTreeSet<String>,
) -> anyhow::Result<Vec<DefinitionEnumFields>> {
    let layer = schema.layer.as_ref().expect("parsed schema has a Layer");
    let mut fields = Vec::new();
    let mut seen = BTreeSet::new();
    for (name, id) in layer.resource().definitions.iter().flatten() {
        if !emitted.contains(name) {
            continue;
        }
        let declared = match layer.node(*id) {
            SchemaNode::Boolean(_value) => false,
            SchemaNode::Keywords(k) => k.properties.is_some() || k.compositions.all_of.is_some(),
        };
        if !declared {
            continue;
        }
        let p = schema.query_node(*id).try_query()?.clone();
        let identity = p
            .reference
            .as_ref()
            .map_or_else(|| name.clone(), |r| r.name.clone());
        if !seen.insert(identity.clone()) {
            continue;
        }
        if let Some(properties) = p.properties {
            fields.push(DefinitionEnumFields {
                source_name: name.clone(),
                name: identity,
                properties,
            });
        }
    }
    Ok(fields)
}
