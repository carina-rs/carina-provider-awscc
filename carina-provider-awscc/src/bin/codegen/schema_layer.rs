//! Lossless, source-indexed CFN syntax graph for shared projections.
//!
//! Reference resolution and conversion preserve every branch and constraint;
//! projection consumers decide how supported shapes are emitted.
#![allow(dead_code)] // Staged alongside the existing parser until consumers migrate.

use serde_json::{Map, Number, Value};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct NodeId(pub(super) usize);

#[derive(Debug, Default)]
pub(super) struct SourceIndex {
    by_pointer: BTreeMap<String, NodeId>,
    pointers: Vec<String>,
    // Accounting includes metadata and opaque JSON sites, not just schemas.
    // In particular propertyTransform keys and unknown-keyword descendants
    // must not disappear when a consumer projects only part of the schema.
    evidence: BTreeMap<String, SourceEvidence>,
}

impl SourceIndex {
    pub(super) fn node_at(&self, pointer: &str) -> Option<NodeId> {
        self.by_pointer.get(pointer).copied()
    }

    pub(super) fn pointer(&self, node: NodeId) -> &str {
        &self.pointers[node.0]
    }

    pub(super) fn evidence(&self) -> &BTreeMap<String, SourceEvidence> {
        &self.evidence
    }
}

/// Facts extracted once at conversion, including non-schema source sites.
/// No JSON payload or inferred schema shape is retained here. Defaults and
/// examples remain literals; their provenance can still require accounting.
#[derive(Debug, Default)]
pub(super) struct SourceEvidence {
    pub(super) name: String,
    pub(super) pattern: Option<String>,
    pub(super) description: Option<String>,
    pub(super) literals: Vec<String>,
    pub(super) has_enum_or_const: bool,
    pub(super) length: (Option<u64>, Option<u64>),
    pub(super) reference: Option<SourceReference>,
}

#[derive(Debug)]
pub(super) enum SourceReference {
    Local { target: String, spelling: String },
    Unresolved(SchemaError),
}

#[derive(Debug)]
pub(super) struct Layer {
    resource: Resource,
    nodes: Vec<SchemaNode>,
    sources: SourceIndex,
    diagnostics: Vec<SchemaError>,
    tag_shapes: Vec<std::sync::OnceLock<super::tag_shape::TagShape>>,
}

impl Layer {
    pub(super) fn from_json(source: &Value) -> Result<Self, SchemaError> {
        let layer = Self::for_codegen(source)?;
        if let Some(error) = layer.diagnostics.first() {
            return Err(error.clone());
        }
        Ok(layer)
    }

    /// Retain known sites and conversion diagnostics. Accounting runs even
    /// for rejected keywords; finish() must
    /// report remaining diagnostics before any generated output is accepted.
    pub(super) fn for_codegen(source: &Value) -> Result<Self, SchemaError> {
        let mut builder = Builder::default();
        index_source_evidence(source, "", "", &mut builder.sources.evidence);
        resolve_source_references(&mut builder.sources.evidence);
        let resource = builder.resource(source, "")?;
        // Pass one indexes ALL syntactic schema sites, including forward targets.
        let Builder {
            mut nodes,
            sources,
            diagnostics,
        } = builder;
        // Pass two connects edges without expanding targets or following cycles.
        for (index, node) in nodes.iter_mut().enumerate() {
            match node {
                SchemaNode::Boolean(_boolean) => {}
                SchemaNode::Keywords(keywords) => {
                    if let Some(reference) = &mut keywords.reference {
                        let pointer = child(sources.pointer(NodeId(index)), "$ref");
                        reference.target =
                            match resolve_reference(&sources, &pointer, &reference.spelling) {
                                Ok(target) => RefTarget::Node(target),
                                Err(reason) => RefTarget::Unresolved(reason),
                            };
                    }
                }
            }
        }
        let tag_shapes = (0..nodes.len())
            .map(|_index| std::sync::OnceLock::new())
            .collect();
        Ok(Self {
            tag_shapes,
            resource,
            nodes,
            sources,
            diagnostics,
        })
    }

    pub(super) fn tag_shape_cache(
        &self,
        node: NodeId,
    ) -> &std::sync::OnceLock<super::tag_shape::TagShape> {
        &self.tag_shapes[node.0]
    }

    pub(super) fn resource(&self) -> &Resource {
        &self.resource
    }
    pub(super) fn node(&self, id: NodeId) -> &SchemaNode {
        &self.nodes[id.0]
    }
    pub(super) fn nodes(&self) -> &[SchemaNode] {
        &self.nodes
    }
    pub(super) fn sources(&self) -> &SourceIndex {
        &self.sources
    }
    pub(super) fn diagnostics(&self) -> &[SchemaError] {
        &self.diagnostics
    }
}

#[derive(Debug, PartialEq)]
pub(super) enum SchemaNode {
    Boolean(bool),
    Keywords(Box<PropertyKeywords>),
}

#[derive(Debug, Default, PartialEq)]
pub(super) struct PropertyKeywords {
    pub(super) reference: Option<ResolvedRef>,
    pub(super) comment: Option<String>,
    pub(super) title: Option<String>,
    pub(super) description: Option<String>,
    // Values here are JSON literals, never schema sites. Some(Null) != None.
    pub(super) default: Option<Value>,
    pub(super) examples: Option<Vec<Value>>,
    pub(super) const_value: Option<Value>,
    pub(super) enum_values: Option<Vec<Value>>,
    pub(super) types: Option<TypeSet>,
    pub(super) format: Option<String>,
    pub(super) multiple_of: Option<Number>,
    pub(super) minimum: Option<Number>,
    pub(super) maximum: Option<Number>,
    pub(super) exclusive_minimum: Option<Number>,
    pub(super) exclusive_maximum: Option<Number>,
    pub(super) min_length: Option<Number>,
    pub(super) max_length: Option<Number>,
    pub(super) pattern: Option<String>,
    pub(super) items: Option<NodeId>,
    pub(super) contains: Option<NodeId>,
    pub(super) min_items: Option<Number>,
    pub(super) max_items: Option<Number>,
    pub(super) unique_items: Option<bool>,
    pub(super) insertion_order: Option<bool>,
    pub(super) array_type: Option<ArrayType>,
    pub(super) min_properties: Option<Number>,
    pub(super) max_properties: Option<Number>,
    pub(super) required: Option<Vec<String>>,
    pub(super) properties: Option<BTreeMap<String, NodeId>>,
    pub(super) pattern_properties: Option<BTreeMap<String, NodeId>>,
    pub(super) additional_properties: Option<NodeId>,
    pub(super) dependencies: Option<BTreeMap<String, Dependency>>,
    pub(super) compositions: Compositions,
    pub(super) relationship_ref: Option<RelationshipRef>,
}

#[derive(Debug, Default, PartialEq)]
pub(super) struct Compositions {
    pub(super) all_of: Option<Vec<NodeId>>,
    pub(super) one_of: Option<Vec<NodeId>>,
    pub(super) any_of: Option<Vec<NodeId>>,
}

#[derive(Debug, PartialEq)]
pub(super) enum Dependency {
    Schema(NodeId),
    PropertyList(Vec<String>),
}

#[derive(Debug, PartialEq)]
pub(super) struct ResolvedRef {
    pub(super) spelling: String,
    pub(super) target: RefTarget,
}

#[derive(Debug, PartialEq)]
pub(super) enum RefTarget {
    Node(NodeId),
    // Keep failures as data, including dangling refs used by the legacy fallback.
    // Consumers, when migrated, decide whether a particular use requires resolution.
    Unresolved(SchemaError),
}

#[derive(Debug, PartialEq)]
pub(super) enum TypeSet {
    Single(JsonType),
    Multiple(Vec<JsonType>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(super) enum JsonType {
    Null,
    Boolean,
    Object,
    Array,
    Number,
    Integer,
    String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ArrayType {
    Standard,
    AttributeList,
}

#[derive(Debug, Default, PartialEq)]
pub(super) struct RelationshipRef {
    pub(super) type_name: String,
    pub(super) property_path: String,
    pub(super) publisher_id: Option<String>,
    pub(super) major_version: Option<Number>,
}

#[derive(Debug, Default)]
pub(super) struct Resource {
    pub(super) root: NodeId,
    pub(super) schema_uri: Option<String>,
    pub(super) id: Option<String>,
    pub(super) resource_type: Option<String>,
    pub(super) type_name: Option<String>,
    pub(super) source_url: Option<String>,
    pub(super) documentation_url: Option<String>,
    pub(super) definitions: Option<BTreeMap<String, NodeId>>,
    pub(super) read_only_properties: Option<Vec<String>>,
    pub(super) write_only_properties: Option<Vec<String>>,
    pub(super) create_only_properties: Option<Vec<String>>,
    pub(super) conditional_create_only_properties: Option<Vec<String>>,
    pub(super) deprecated_properties: Option<Vec<String>>,
    pub(super) non_public_properties: Option<Vec<String>>,
    pub(super) non_public_definitions: Option<Vec<String>>,
    pub(super) primary_identifier: Option<Vec<String>>,
    pub(super) additional_identifiers: Option<Vec<Vec<String>>>,
    pub(super) tagging: Option<Tagging>,
    pub(super) taggable: Option<bool>,
    pub(super) handlers: Option<BTreeMap<String, Handler>>,
    // Paths and expressions are metadata, retained for later KMS accounting.
    pub(super) property_transform: Option<BTreeMap<String, String>>,
    pub(super) replacement_strategy: Option<ReplacementStrategy>,
    pub(super) resource_link: Option<ResourceLink>,
    pub(super) remote: Option<BTreeMap<String, RemoteSchema>>,
    pub(super) type_configuration: Option<Box<Resource>>,
}

#[derive(Debug, Default)]
pub(super) struct Tagging {
    pub(super) taggable: Option<bool>,
    pub(super) tag_on_create: Option<bool>,
    pub(super) tag_updatable: Option<bool>,
    pub(super) cloud_formation_system_tags: Option<bool>,
    pub(super) tag_property: Option<String>,
    pub(super) permissions: Option<Vec<String>>,
}

#[derive(Debug, Default)]
pub(super) struct Handler {
    pub(super) permissions: Option<Vec<String>>,
    pub(super) timeout_in_minutes: Option<Number>,
    pub(super) schema: Option<NodeId>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ReplacementStrategy {
    CreateThenDelete,
    DeleteThenCreate,
}

#[derive(Debug, Default)]
pub(super) struct ResourceLink {
    pub(super) comment: Option<String>,
    pub(super) template_uri: Option<String>,
    pub(super) mappings: Option<BTreeMap<String, String>>,
}

#[derive(Debug, Default)]
pub(super) struct RemoteSchema {
    pub(super) comment: Option<String>,
    pub(super) properties: Option<BTreeMap<String, NodeId>>,
    pub(super) definitions: Option<BTreeMap<String, NodeId>>,
    // The CLI explicitly allows additional metadata on remote namespaces.
    pub(super) annotations: BTreeMap<String, Value>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ValueKind {
    Null,
    Boolean,
    Number,
    String,
    Array,
    Object,
}

impl ValueKind {
    pub(super) fn of(value: &Value) -> Self {
        match value {
            Value::Null => Self::Null,
            Value::Bool(_) => Self::Boolean,
            Value::Number(_) => Self::Number,
            Value::String(_) => Self::String,
            Value::Array(_) => Self::Array,
            Value::Object(_) => Self::Object,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub(super) enum SchemaError {
    #[error("unsupported schema projection at #{pointer}: {reason}")]
    Projection { pointer: String, reason: String },
    #[error("unknown schema keyword {keyword:?} at #{pointer}")]
    UnknownKeyword { pointer: String, keyword: String },
    #[error("invalid shape at #{pointer}: expected {expected}, got {actual:?}")]
    InvalidShape {
        pointer: String,
        expected: &'static str,
        actual: ValueKind,
    },
    #[error("invalid value at #{pointer}: {reason}")]
    InvalidValue { pointer: String, reason: String },
    #[error("invalid reference {reference:?} at #{pointer}: {reason}")]
    InvalidPointer {
        pointer: String,
        reference: String,
        reason: String,
    },
    #[error("external reference {reference:?} at #{pointer} is unsupported")]
    UnsupportedReference { pointer: String, reference: String },
    #[error("reference {reference:?} at #{pointer} has no target")]
    MissingReferenceTarget { pointer: String, reference: String },
    #[error("reference {reference:?} at #{pointer} targets non-schema JSON at #{target}")]
    NonSchemaReferenceTarget {
        pointer: String,
        reference: String,
        target: String,
    },
}

// Closed keyword set pinned from definitions/properties/allOf/1/properties in the
// CLI-distributed base.definition.schema.v1.json, fetched 2026-11-06:
// https://raw.githubusercontent.com/aws-cloudformation/cloudformation-cli/master/src/rpdk/core/data/schema/base.definition.schema.v1.json
// The resource-schema repository copy differs (it omits relationshipRef).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum PropertyKeyword {
    InsertionOrder,
    ArrayType,
    RelationshipRef,
    Ref,
    Comment,
    Title,
    Description,
    Examples,
    Default,
    MultipleOf,
    Maximum,
    ExclusiveMaximum,
    Minimum,
    ExclusiveMinimum,
    MaxLength,
    MinLength,
    Pattern,
    Items,
    MaxItems,
    MinItems,
    UniqueItems,
    Contains,
    MaxProperties,
    MinProperties,
    Required,
    Properties,
    AdditionalProperties,
    PatternProperties,
    Dependencies,
    Const,
    Enum,
    Type,
    Format,
    AllOf,
    AnyOf,
    OneOf,
}

impl PropertyKeyword {
    pub(super) const ALL: [Self; 36] = [
        Self::InsertionOrder,
        Self::ArrayType,
        Self::RelationshipRef,
        Self::Ref,
        Self::Comment,
        Self::Title,
        Self::Description,
        Self::Examples,
        Self::Default,
        Self::MultipleOf,
        Self::Maximum,
        Self::ExclusiveMaximum,
        Self::Minimum,
        Self::ExclusiveMinimum,
        Self::MaxLength,
        Self::MinLength,
        Self::Pattern,
        Self::Items,
        Self::MaxItems,
        Self::MinItems,
        Self::UniqueItems,
        Self::Contains,
        Self::MaxProperties,
        Self::MinProperties,
        Self::Required,
        Self::Properties,
        Self::AdditionalProperties,
        Self::PatternProperties,
        Self::Dependencies,
        Self::Const,
        Self::Enum,
        Self::Type,
        Self::Format,
        Self::AllOf,
        Self::AnyOf,
        Self::OneOf,
    ];

    pub(super) fn as_str(self) -> &'static str {
        match self {
            Self::InsertionOrder => "insertionOrder",
            Self::ArrayType => "arrayType",
            Self::RelationshipRef => "relationshipRef",
            Self::Ref => "$ref",
            Self::Comment => "$comment",
            Self::Title => "title",
            Self::Description => "description",
            Self::Examples => "examples",
            Self::Default => "default",
            Self::MultipleOf => "multipleOf",
            Self::Maximum => "maximum",
            Self::ExclusiveMaximum => "exclusiveMaximum",
            Self::Minimum => "minimum",
            Self::ExclusiveMinimum => "exclusiveMinimum",
            Self::MaxLength => "maxLength",
            Self::MinLength => "minLength",
            Self::Pattern => "pattern",
            Self::Items => "items",
            Self::MaxItems => "maxItems",
            Self::MinItems => "minItems",
            Self::UniqueItems => "uniqueItems",
            Self::Contains => "contains",
            Self::MaxProperties => "maxProperties",
            Self::MinProperties => "minProperties",
            Self::Required => "required",
            Self::Properties => "properties",
            Self::AdditionalProperties => "additionalProperties",
            Self::PatternProperties => "patternProperties",
            Self::Dependencies => "dependencies",
            Self::Const => "const",
            Self::Enum => "enum",
            Self::Type => "type",
            Self::Format => "format",
            Self::AllOf => "allOf",
            Self::AnyOf => "anyOf",
            Self::OneOf => "oneOf",
        }
    }

    pub(super) fn from_name(name: &str, pointer: &str) -> Result<Self, SchemaError> {
        match name {
            "insertionOrder" => Ok(Self::InsertionOrder),
            "arrayType" => Ok(Self::ArrayType),
            "relationshipRef" => Ok(Self::RelationshipRef),
            "$ref" => Ok(Self::Ref),
            "$comment" => Ok(Self::Comment),
            "title" => Ok(Self::Title),
            "description" => Ok(Self::Description),
            "examples" => Ok(Self::Examples),
            "default" => Ok(Self::Default),
            "multipleOf" => Ok(Self::MultipleOf),
            "maximum" => Ok(Self::Maximum),
            "exclusiveMaximum" => Ok(Self::ExclusiveMaximum),
            "minimum" => Ok(Self::Minimum),
            "exclusiveMinimum" => Ok(Self::ExclusiveMinimum),
            "maxLength" => Ok(Self::MaxLength),
            "minLength" => Ok(Self::MinLength),
            "pattern" => Ok(Self::Pattern),
            "items" => Ok(Self::Items),
            "maxItems" => Ok(Self::MaxItems),
            "minItems" => Ok(Self::MinItems),
            "uniqueItems" => Ok(Self::UniqueItems),
            "contains" => Ok(Self::Contains),
            "maxProperties" => Ok(Self::MaxProperties),
            "minProperties" => Ok(Self::MinProperties),
            "required" => Ok(Self::Required),
            "properties" => Ok(Self::Properties),
            "additionalProperties" => Ok(Self::AdditionalProperties),
            "patternProperties" => Ok(Self::PatternProperties),
            "dependencies" => Ok(Self::Dependencies),
            "const" => Ok(Self::Const),
            "enum" => Ok(Self::Enum),
            "type" => Ok(Self::Type),
            "format" => Ok(Self::Format),
            "allOf" => Ok(Self::AllOf),
            "anyOf" => Ok(Self::AnyOf),
            "oneOf" => Ok(Self::OneOf),
            unknown => Err(SchemaError::UnknownKeyword {
                pointer: pointer.into(),
                keyword: unknown.into(),
            }),
        }
    }
}

#[derive(Default)]
struct Builder {
    nodes: Vec<SchemaNode>,
    sources: SourceIndex,
    diagnostics: Vec<SchemaError>,
}

impl Builder {
    fn allocate(&mut self, pointer: &str, node: SchemaNode) -> NodeId {
        let id = NodeId(self.nodes.len());
        self.nodes.push(node);
        self.sources.pointers.push(pointer.into());
        assert!(
            self.sources.by_pointer.insert(pointer.into(), id).is_none(),
            "duplicate schema site #{pointer}"
        );
        id
    }

    fn schema(&mut self, value: &Value, pointer: &str) -> Result<NodeId, SchemaError> {
        let node = match value {
            Value::Bool(boolean) => SchemaNode::Boolean(*boolean),
            Value::Object(object) => {
                let mut keywords = PropertyKeywords::default();
                for (name, value) in object {
                    let at = child(pointer, name);
                    self.property_keyword(&mut keywords, name, value, &at);
                }
                SchemaNode::Keywords(Box::new(keywords))
            }
            Value::Null | Value::Number(_) | Value::String(_) | Value::Array(_) => {
                return Err(invalid_shape(value, pointer, "schema object or boolean"));
            }
        };
        Ok(self.allocate(pointer, node))
    }

    fn resource(&mut self, value: &Value, pointer: &str) -> Result<Resource, SchemaError> {
        let mut resource = Resource::default();
        match value {
            Value::Bool(boolean) => {
                resource.root = self.allocate(pointer, SchemaNode::Boolean(*boolean))
            }
            Value::Object(object) => {
                let mut keywords = PropertyKeywords::default();
                for (name, value) in object {
                    let at = child(pointer, name);
                    match name.as_str() {
                        "$schema" => resource.schema_uri = Some(string(value, &at)?),
                        "$id" => resource.id = Some(string(value, &at)?),
                        "typeName" => resource.type_name = Some(string(value, &at)?),
                        "type" if value == "RESOURCE" => {
                            resource.resource_type = Some(string(value, &at)?)
                        }
                        "sourceUrl" => resource.source_url = Some(string(value, &at)?),
                        "documentationUrl" => {
                            resource.documentation_url = Some(string(value, &at)?)
                        }
                        "definitions" => resource.definitions = Some(self.schema_map(value, &at)?),
                        "readOnlyProperties" => {
                            resource.read_only_properties = Some(strings(value, &at)?)
                        }
                        "writeOnlyProperties" => {
                            resource.write_only_properties = Some(strings(value, &at)?)
                        }
                        "createOnlyProperties" => {
                            resource.create_only_properties = Some(strings(value, &at)?)
                        }
                        "conditionalCreateOnlyProperties" => {
                            resource.conditional_create_only_properties = Some(strings(value, &at)?)
                        }
                        "deprecatedProperties" => {
                            resource.deprecated_properties = Some(strings(value, &at)?)
                        }
                        "nonPublicProperties" => {
                            resource.non_public_properties = Some(strings(value, &at)?)
                        }
                        "nonPublicDefinitions" => {
                            resource.non_public_definitions = Some(strings(value, &at)?)
                        }
                        "primaryIdentifier" => {
                            resource.primary_identifier = Some(strings(value, &at)?)
                        }
                        "additionalIdentifiers" => {
                            resource.additional_identifiers = Some(
                                array(value, &at)?
                                    .iter()
                                    .enumerate()
                                    .map(|(index, value)| {
                                        strings(value, &child(&at, &index.to_string()))
                                    })
                                    .collect::<Result<_, _>>()?,
                            )
                        }
                        "tagging" => resource.tagging = Some(tagging(value, &at)?),
                        "taggable" => resource.taggable = Some(boolean(value, &at)?),
                        "handlers" => resource.handlers = Some(self.handlers(value, &at)?),
                        "propertyTransform" => {
                            resource.property_transform = Some(string_map(value, &at)?)
                        }
                        "replacementStrategy" => {
                            resource.replacement_strategy =
                                Some(match string(value, &at)?.as_str() {
                                    "create_then_delete" => ReplacementStrategy::CreateThenDelete,
                                    "delete_then_create" => ReplacementStrategy::DeleteThenCreate,
                                    unknown => {
                                        return Err(invalid_value(
                                            &at,
                                            format!("unknown replacement strategy {unknown:?}"),
                                        ));
                                    }
                                })
                        }
                        "resourceLink" => resource.resource_link = Some(resource_link(value, &at)?),
                        "remote" => resource.remote = Some(self.remote(value, &at)?),
                        "typeConfiguration" => {
                            resource.type_configuration = Some(Box::new(self.resource(value, &at)?))
                        }
                        property => self.property_keyword(&mut keywords, property, value, &at),
                    }
                }
                resource.root = self.allocate(pointer, SchemaNode::Keywords(Box::new(keywords)));
            }
            Value::Null | Value::Number(_) | Value::String(_) | Value::Array(_) => {
                return Err(invalid_shape(value, pointer, "schema object or boolean"));
            }
        }
        Ok(resource)
    }

    fn property_keyword(
        &mut self,
        node: &mut PropertyKeywords,
        name: &str,
        value: &Value,
        at: &str,
    ) {
        let result = PropertyKeyword::from_name(name, at)
            .and_then(|keyword| self.keyword(node, keyword, value, at));
        if let Err(error) = result {
            self.diagnostics.push(error);
        }
    }

    fn keyword(
        &mut self,
        node: &mut PropertyKeywords,
        keyword: PropertyKeyword,
        value: &Value,
        at: &str,
    ) -> Result<(), SchemaError> {
        match keyword {
            PropertyKeyword::Ref => {
                let spelling = string(value, at)?;
                node.reference = Some(ResolvedRef {
                    target: RefTarget::Unresolved(SchemaError::MissingReferenceTarget {
                        pointer: at.into(),
                        reference: spelling.clone(),
                    }),
                    spelling,
                });
            }
            PropertyKeyword::Comment => node.comment = Some(string(value, at)?),
            PropertyKeyword::Title => node.title = Some(string(value, at)?),
            PropertyKeyword::Description => node.description = Some(string(value, at)?),
            PropertyKeyword::Examples => node.examples = Some(array(value, at)?.clone()),
            PropertyKeyword::Default => node.default = Some(value.clone()),
            PropertyKeyword::Const => node.const_value = Some(value.clone()),
            PropertyKeyword::Enum => {
                let values = array(value, at)?;
                if values.is_empty() {
                    return Err(invalid_value(at, "enum must not be empty"));
                }
                node.enum_values = Some(values.clone());
            }
            PropertyKeyword::Type => node.types = Some(type_set(value, at)?),
            PropertyKeyword::Format => node.format = Some(string(value, at)?),
            PropertyKeyword::MultipleOf => {
                let number = number(value, at)?;
                if number.as_f64().is_some_and(|number| number <= 0.0) {
                    return Err(invalid_value(at, "multipleOf must be positive"));
                }
                node.multiple_of = Some(number);
            }
            PropertyKeyword::Minimum => node.minimum = Some(number(value, at)?),
            PropertyKeyword::Maximum => node.maximum = Some(number(value, at)?),
            PropertyKeyword::ExclusiveMinimum => node.exclusive_minimum = Some(number(value, at)?),
            PropertyKeyword::ExclusiveMaximum => node.exclusive_maximum = Some(number(value, at)?),
            PropertyKeyword::MinLength => node.min_length = Some(nonnegative_integer(value, at)?),
            PropertyKeyword::MaxLength => node.max_length = Some(nonnegative_integer(value, at)?),
            PropertyKeyword::Pattern => node.pattern = Some(string(value, at)?),
            PropertyKeyword::Items => node.items = Some(self.schema(value, at)?),
            PropertyKeyword::Contains => node.contains = Some(self.schema(value, at)?),
            PropertyKeyword::MinItems => node.min_items = Some(nonnegative_integer(value, at)?),
            PropertyKeyword::MaxItems => node.max_items = Some(nonnegative_integer(value, at)?),
            PropertyKeyword::UniqueItems => node.unique_items = Some(boolean(value, at)?),
            PropertyKeyword::InsertionOrder => node.insertion_order = Some(boolean(value, at)?),
            PropertyKeyword::ArrayType => {
                node.array_type = Some(match string(value, at)?.as_str() {
                    "Standard" => ArrayType::Standard,
                    "AttributeList" => ArrayType::AttributeList,
                    unknown => {
                        return Err(invalid_value(at, format!("unknown arrayType {unknown:?}")));
                    }
                })
            }
            PropertyKeyword::MinProperties => {
                node.min_properties = Some(nonnegative_integer(value, at)?)
            }
            PropertyKeyword::MaxProperties => {
                node.max_properties = Some(nonnegative_integer(value, at)?)
            }
            PropertyKeyword::Required => node.required = Some(strings(value, at)?),
            PropertyKeyword::Properties => node.properties = Some(self.schema_map(value, at)?),
            PropertyKeyword::PatternProperties => {
                node.pattern_properties = Some(self.schema_map(value, at)?)
            }
            PropertyKeyword::AdditionalProperties => {
                node.additional_properties = Some(self.schema(value, at)?)
            }
            PropertyKeyword::Dependencies => {
                node.dependencies = Some(self.dependencies(value, at)?)
            }
            PropertyKeyword::AllOf => {
                node.compositions.all_of = Some(self.schema_array(value, at)?)
            }
            PropertyKeyword::AnyOf => {
                node.compositions.any_of = Some(self.schema_array(value, at)?)
            }
            PropertyKeyword::OneOf => {
                node.compositions.one_of = Some(self.schema_array(value, at)?)
            }
            PropertyKeyword::RelationshipRef => {
                node.relationship_ref = Some(relationship_ref(value, at)?)
            }
        }
        Ok(())
    }

    fn schema_map(
        &mut self,
        value: &Value,
        pointer: &str,
    ) -> Result<BTreeMap<String, NodeId>, SchemaError> {
        object(value, pointer)?
            .iter()
            .map(|(name, value)| Ok((name.clone(), self.schema(value, &child(pointer, name))?)))
            .collect()
    }

    fn schema_array(&mut self, value: &Value, pointer: &str) -> Result<Vec<NodeId>, SchemaError> {
        let values = array(value, pointer)?;
        if values.is_empty() {
            return Err(invalid_value(pointer, "schema array must not be empty"));
        }
        values
            .iter()
            .enumerate()
            .map(|(index, value)| self.schema(value, &child(pointer, &index.to_string())))
            .collect()
    }

    fn dependencies(
        &mut self,
        value: &Value,
        pointer: &str,
    ) -> Result<BTreeMap<String, Dependency>, SchemaError> {
        object(value, pointer)?
            .iter()
            .map(|(name, value)| {
                let at = child(pointer, name);
                let dependency = match value {
                    Value::Array(_) => Dependency::PropertyList(strings(value, &at)?),
                    Value::Bool(_) | Value::Object(_) => {
                        Dependency::Schema(self.schema(value, &at)?)
                    }
                    Value::Null | Value::Number(_) | Value::String(_) => {
                        return Err(invalid_shape(value, &at, "schema or property-name array"));
                    }
                };
                Ok((name.clone(), dependency))
            })
            .collect()
    }

    fn handlers(
        &mut self,
        value: &Value,
        pointer: &str,
    ) -> Result<BTreeMap<String, Handler>, SchemaError> {
        object(value, pointer)?
            .iter()
            .map(|(operation, value)| {
                let at = child(pointer, operation);
                match operation.as_str() {
                    "create" | "read" | "update" | "delete" | "list" => {}
                    unknown => {
                        return Err(SchemaError::UnknownKeyword {
                            pointer: at,
                            keyword: unknown.into(),
                        });
                    }
                }
                let mut handler = Handler::default();
                for (name, value) in object(value, &at)? {
                    let at = child(&at, name);
                    match name.as_str() {
                        "permissions" => handler.permissions = Some(strings(value, &at)?),
                        "timeoutInMinutes" => {
                            handler.timeout_in_minutes = Some(nonnegative_integer(value, &at)?)
                        }
                        "handlerSchema" => handler.schema = Some(self.schema(value, &at)?),
                        unknown => {
                            return Err(SchemaError::UnknownKeyword {
                                pointer: at,
                                keyword: unknown.into(),
                            });
                        }
                    }
                }
                Ok((operation.clone(), handler))
            })
            .collect()
    }

    fn remote(
        &mut self,
        value: &Value,
        pointer: &str,
    ) -> Result<BTreeMap<String, RemoteSchema>, SchemaError> {
        object(value, pointer)?
            .iter()
            .map(|(name, value)| {
                let at = child(pointer, name);
                let mut remote = RemoteSchema::default();
                for (name, value) in object(value, &at)? {
                    let at = child(&at, name);
                    match name.as_str() {
                        "$comment" => remote.comment = Some(string(value, &at)?),
                        "properties" => remote.properties = Some(self.schema_map(value, &at)?),
                        "definitions" => remote.definitions = Some(self.schema_map(value, &at)?),
                        annotation => {
                            remote.annotations.insert(annotation.into(), value.clone());
                        }
                    }
                }
                Ok((name.clone(), remote))
            })
            .collect()
    }
}

fn child(pointer: &str, name: &str) -> String {
    format!("{pointer}/{}", name.replace('~', "~0").replace('/', "~1"))
}

fn invalid_shape(value: &Value, pointer: &str, expected: &'static str) -> SchemaError {
    SchemaError::InvalidShape {
        pointer: pointer.into(),
        expected,
        actual: ValueKind::of(value),
    }
}

fn invalid_value(pointer: &str, reason: impl Into<String>) -> SchemaError {
    SchemaError::InvalidValue {
        pointer: pointer.into(),
        reason: reason.into(),
    }
}

fn object<'a>(value: &'a Value, at: &str) -> Result<&'a Map<String, Value>, SchemaError> {
    match value {
        Value::Object(object) => Ok(object),
        Value::Null | Value::Bool(_) | Value::Number(_) | Value::String(_) | Value::Array(_) => {
            Err(invalid_shape(value, at, "object"))
        }
    }
}

fn array<'a>(value: &'a Value, at: &str) -> Result<&'a Vec<Value>, SchemaError> {
    match value {
        Value::Array(array) => Ok(array),
        Value::Null | Value::Bool(_) | Value::Number(_) | Value::String(_) | Value::Object(_) => {
            Err(invalid_shape(value, at, "array"))
        }
    }
}

fn string(value: &Value, at: &str) -> Result<String, SchemaError> {
    match value {
        Value::String(string) => Ok(string.clone()),
        Value::Null | Value::Bool(_) | Value::Number(_) | Value::Array(_) | Value::Object(_) => {
            Err(invalid_shape(value, at, "string"))
        }
    }
}

fn boolean(value: &Value, at: &str) -> Result<bool, SchemaError> {
    match value {
        Value::Bool(boolean) => Ok(*boolean),
        Value::Null | Value::Number(_) | Value::String(_) | Value::Array(_) | Value::Object(_) => {
            Err(invalid_shape(value, at, "boolean"))
        }
    }
}

fn number(value: &Value, at: &str) -> Result<Number, SchemaError> {
    match value {
        Value::Number(number) => Ok(number.clone()),
        Value::Null | Value::Bool(_) | Value::String(_) | Value::Array(_) | Value::Object(_) => {
            Err(invalid_shape(value, at, "number"))
        }
    }
}

fn nonnegative_integer(value: &Value, at: &str) -> Result<Number, SchemaError> {
    let number = number(value, at)?;
    // Keep Number, including integer-valued floats and u64s beyond i64::MAX.
    if number.as_u64().is_some()
        || number
            .as_f64()
            .is_some_and(|number| number >= 0.0 && number.fract() == 0.0)
    {
        Ok(number)
    } else {
        Err(invalid_value(at, "expected a non-negative integer"))
    }
}

fn strings(value: &Value, at: &str) -> Result<Vec<String>, SchemaError> {
    array(value, at)?
        .iter()
        .enumerate()
        .map(|(index, value)| string(value, &child(at, &index.to_string())))
        .collect()
}

fn string_map(value: &Value, at: &str) -> Result<BTreeMap<String, String>, SchemaError> {
    object(value, at)?
        .iter()
        .map(|(name, value)| Ok((name.clone(), string(value, &child(at, name))?)))
        .collect()
}

fn json_type(value: &Value, at: &str) -> Result<JsonType, SchemaError> {
    match string(value, at)?.as_str() {
        "null" => Ok(JsonType::Null),
        "boolean" => Ok(JsonType::Boolean),
        "object" => Ok(JsonType::Object),
        "array" => Ok(JsonType::Array),
        "number" => Ok(JsonType::Number),
        "integer" => Ok(JsonType::Integer),
        "string" => Ok(JsonType::String),
        unknown => Err(invalid_value(at, format!("unknown JSON type {unknown:?}"))),
    }
}

fn type_set(value: &Value, at: &str) -> Result<TypeSet, SchemaError> {
    match value {
        Value::String(_) => Ok(TypeSet::Single(json_type(value, at)?)),
        Value::Array(values) => {
            if values.is_empty() {
                return Err(invalid_value(at, "type array must not be empty"));
            }
            let types = values
                .iter()
                .enumerate()
                .map(|(index, value)| json_type(value, &child(at, &index.to_string())))
                .collect::<Result<Vec<_>, _>>()?;
            if types.iter().collect::<BTreeSet<_>>().len() != types.len() {
                return Err(invalid_value(
                    at,
                    "type array must contain distinct members",
                ));
            }
            Ok(TypeSet::Multiple(types))
        }
        Value::Null | Value::Bool(_) | Value::Number(_) | Value::Object(_) => {
            Err(invalid_shape(value, at, "type name or type-name array"))
        }
    }
}

fn relationship_ref(value: &Value, at: &str) -> Result<RelationshipRef, SchemaError> {
    let object = object(value, at)?;
    let mut relationship = RelationshipRef::default();
    for (name, value) in object {
        let at = child(at, name);
        match name.as_str() {
            "typeName" => relationship.type_name = string(value, &at)?,
            "propertyPath" => relationship.property_path = string(value, &at)?,
            "publisherId" => relationship.publisher_id = Some(string(value, &at)?),
            "majorVersion" => relationship.major_version = Some(nonnegative_integer(value, &at)?),
            unknown => {
                return Err(SchemaError::UnknownKeyword {
                    pointer: at,
                    keyword: unknown.into(),
                });
            }
        }
    }
    for name in ["typeName", "propertyPath"] {
        if !object.contains_key(name) {
            return Err(invalid_value(
                at,
                format!("relationshipRef requires {name}"),
            ));
        }
    }
    Ok(relationship)
}

fn tagging(value: &Value, at: &str) -> Result<Tagging, SchemaError> {
    let mut tagging = Tagging::default();
    for (name, value) in object(value, at)? {
        let at = child(at, name);
        match name.as_str() {
            "taggable" => tagging.taggable = Some(boolean(value, &at)?),
            "tagOnCreate" => tagging.tag_on_create = Some(boolean(value, &at)?),
            "tagUpdatable" => tagging.tag_updatable = Some(boolean(value, &at)?),
            "cloudFormationSystemTags" => {
                tagging.cloud_formation_system_tags = Some(boolean(value, &at)?)
            }
            "tagProperty" => tagging.tag_property = Some(string(value, &at)?),
            "permissions" => tagging.permissions = Some(strings(value, &at)?),
            unknown => {
                return Err(SchemaError::UnknownKeyword {
                    pointer: at,
                    keyword: unknown.into(),
                });
            }
        }
    }
    Ok(tagging)
}

fn resource_link(value: &Value, at: &str) -> Result<ResourceLink, SchemaError> {
    let mut link = ResourceLink::default();
    for (name, value) in object(value, at)? {
        let at = child(at, name);
        match name.as_str() {
            "$comment" => link.comment = Some(string(value, &at)?),
            "templateUri" => link.template_uri = Some(string(value, &at)?),
            "mappings" => link.mappings = Some(string_map(value, &at)?),
            unknown => {
                return Err(SchemaError::UnknownKeyword {
                    pointer: at,
                    keyword: unknown.into(),
                });
            }
        }
    }
    Ok(link)
}

fn index_source_evidence(
    value: &Value,
    pointer: &str,
    name: &str,
    sites: &mut BTreeMap<String, SourceEvidence>,
) {
    let mut evidence = SourceEvidence {
        name: name.into(),
        ..SourceEvidence::default()
    };
    match value {
        Value::Object(fields) => {
            evidence.pattern = fields
                .get("pattern")
                .and_then(Value::as_str)
                .map(str::to_string);
            evidence.description = fields
                .get("description")
                .and_then(Value::as_str)
                .map(str::to_string);
            evidence.has_enum_or_const =
                fields.contains_key("enum") || fields.contains_key("const");
            evidence.literals = fields
                .get("enum")
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
                .chain(fields.get("const"))
                .filter_map(Value::as_str)
                .map(str::to_string)
                .collect();
            evidence.length = (
                fields.get("minLength").and_then(Value::as_u64),
                fields.get("maxLength").and_then(Value::as_u64),
            );
            if let Some(spelling) = fields.get("$ref").and_then(Value::as_str) {
                evidence.reference = Some(
                    match canonical_reference(&child(pointer, "$ref"), spelling) {
                        Ok(target) => SourceReference::Local {
                            target,
                            spelling: spelling.into(),
                        },
                        Err(reason) => SourceReference::Unresolved(reason),
                    },
                );
            }
            for (key, value) in fields {
                index_source_evidence(value, &child(pointer, key), key, sites);
            }
        }
        Value::Array(values) => {
            for (index, value) in values.iter().enumerate() {
                index_source_evidence(value, &child(pointer, &index.to_string()), "", sites);
            }
        }
        Value::Null => {}
        Value::Bool(_boolean) => {}
        Value::Number(_number) => {}
        Value::String(_string) => {}
    }
    sites.insert(pointer.into(), evidence);
}

fn resolve_source_references(sites: &mut BTreeMap<String, SourceEvidence>) {
    let pointers: BTreeSet<_> = sites.keys().cloned().collect();
    for (pointer, evidence) in sites {
        if let Some(SourceReference::Local { target, spelling }) = &evidence.reference
            && !pointers.contains(target)
        {
            evidence.reference = Some(SourceReference::Unresolved(
                SchemaError::MissingReferenceTarget {
                    pointer: child(pointer, "$ref"),
                    reference: spelling.clone(),
                },
            ));
        }
    }
}

fn resolve_reference(
    sources: &SourceIndex,
    at: &str,
    spelling: &str,
) -> Result<NodeId, SchemaError> {
    let canonical = canonical_reference(at, spelling)?;
    if let Some(target) = sources.node_at(&canonical) {
        return Ok(target);
    }
    if sources.evidence.contains_key(&canonical) {
        Err(SchemaError::NonSchemaReferenceTarget {
            pointer: at.into(),
            reference: spelling.into(),
            target: canonical,
        })
    } else {
        Err(SchemaError::MissingReferenceTarget {
            pointer: at.into(),
            reference: spelling.into(),
        })
    }
}

pub(super) fn canonical_reference(at: &str, spelling: &str) -> Result<String, SchemaError> {
    let invalid = |reason: &str| SchemaError::InvalidPointer {
        pointer: at.into(),
        reference: spelling.into(),
        reason: reason.into(),
    };
    let fragment = spelling
        .strip_prefix('#')
        .ok_or_else(|| SchemaError::UnsupportedReference {
            pointer: at.into(),
            reference: spelling.into(),
        })?;
    // URI fragments are percent-decoded BEFORE JSON Pointer token unescaping.
    // '+' is a literal plus, not form encoding. Invalid UTF-8 is an error.
    let mut decoded = Vec::new();
    let mut bytes = fragment.bytes();
    while let Some(byte) = bytes.next() {
        match byte {
            b'%' => {
                let high = bytes
                    .next()
                    .and_then(hex_digit)
                    .ok_or_else(|| invalid("invalid percent escape"))?;
                let low = bytes
                    .next()
                    .and_then(hex_digit)
                    .ok_or_else(|| invalid("invalid percent escape"))?;
                decoded.push(high * 16 + low);
            }
            literal => decoded.push(literal),
        }
    }
    let decoded = String::from_utf8(decoded).map_err(|_error| invalid("fragment is not UTF-8"))?;
    let mut canonical = String::new();
    if !decoded.is_empty() {
        let tokens = decoded
            .strip_prefix('/')
            .ok_or_else(|| invalid("fragment must be empty or start with '/'"))?;
        for token in tokens.split('/') {
            let mut unescaped = String::new();
            let mut characters = token.chars();
            while let Some(character) = characters.next() {
                match character {
                    '~' => match characters.next() {
                        Some('0') => unescaped.push('~'),
                        Some('1') => unescaped.push('/'),
                        Some(_) | None => return Err(invalid("invalid JSON Pointer '~' escape")),
                    },
                    literal => unescaped.push(literal),
                }
            }
            canonical = child(&canonical, &unescaped);
        }
    }
    Ok(canonical)
}

fn hex_digit(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _invalid_hex => None,
    }
}

#[cfg(test)]
#[path = "schema_layer_tests.rs"]
mod tests;
