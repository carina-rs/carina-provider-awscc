//! One tag judgement over the Layer's typed projections. Candidate evidence and
//! admission are folded together; callers never inspect schema grammar again.

use super::schema_layer::{
    Dependency, JsonType, Layer, NodeId, RefTarget, SchemaError, SchemaNode,
};
use super::schema_projection::{self, Projection};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

#[derive(Debug, Clone, Copy, Default)]
pub(super) struct TagCandidate {
    key: bool,
    value: bool,
}

impl TagCandidate {
    fn complete(self) -> bool {
        self.key && self.value
    }
    fn union(self, other: Self) -> Self {
        Self {
            key: self.key || other.key,
            value: self.value || other.value,
        }
    }
}

#[derive(Debug, Clone)]
pub(super) struct TagString {
    pub(super) source: String,
    pub(super) pattern: Option<String>,
    pub(super) min_length: Option<u64>,
    pub(super) max_length: Option<u64>,
}

#[derive(Debug, Clone)]
pub(super) enum TagKey {
    Property(TagString),
    Pattern { pattern: String, source: String },
}

#[derive(Debug, Clone)]
pub(super) struct TagMap {
    pub(super) key: TagKey,
    pub(super) value: TagString,
    pub(super) min_entries: Option<i64>,
    pub(super) max_entries: Option<i64>,
    pub(super) definition_name: Option<String>,
    candidate: TagCandidate,
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub(super) enum NotTagReason {
    #[error(
        "expected an array of objects with exactly the case-sensitive Key and Value string properties (no extra fields), or an object map with string values"
    )]
    Shape,
    #[error(
        "item properties and required entries must name only the case-sensitive Key and Value string properties (no extra fields); extra fields: {0}"
    )]
    ExtraFields(String),
    #[error("every oneOf/anyOf arm must agree on the tag shape and constraints")]
    Alternatives,
    #[error("conditional dependencies cannot be projected as unconditional tag constraints")]
    Dependency,
    #[error("contains cannot be projected as unconditional tag constraints")]
    Contains,
}

#[derive(Debug, Clone, thiserror::Error)]
pub(super) enum TagShapeError {
    #[error(
        "{resource}#{pointer}: tag property cannot be projected to a string-to-string map; {reason}"
    )]
    NotTag {
        resource: String,
        pointer: String,
        reason: NotTagReason,
    },
    #[error("{resource}#{pointer}: unreadable tag shape: {source}")]
    Unreadable {
        resource: String,
        pointer: String,
        #[source]
        source: Box<SchemaError>,
    },
}

#[derive(Debug, Clone)]
pub(super) enum TagShape {
    KeyValueList(TagMap),
    /// Individual Tag references in ordinary lists retain their existing map
    /// representation without becoming a resource's default_tags attribute.
    KeyValueObject(TagMap),
    StringMap(TagMap),
    NotTag {
        reason: NotTagReason,
        candidate: TagCandidate,
        pointer: String,
    },
    Unreadable {
        source: SchemaError,
        candidate: TagCandidate,
        pointer: String,
    },
}

impl TagShape {
    fn evidence(&self) -> TagCandidate {
        match self {
            Self::KeyValueList(_map) => TagCandidate {
                key: true,
                value: true,
            },
            Self::KeyValueObject(map) | Self::StringMap(map) => map.candidate,
            Self::NotTag {
                reason: _reason,
                candidate,
                pointer: _pointer,
            } => *candidate,
            Self::Unreadable {
                source: _source,
                candidate,
                pointer: _pointer,
            } => *candidate,
        }
    }

    #[cfg(test)]
    pub(super) fn candidate(&self) -> bool {
        self.evidence().complete()
    }

    /// Resource compositions may contribute separate source sites for one
    /// top-level property. Combine their already-classified evidence only.
    pub(super) fn candidates<'a>(shapes: impl IntoIterator<Item = &'a Self>) -> bool {
        shapes
            .into_iter()
            .fold(TagCandidate::default(), |evidence, shape| {
                evidence.union(shape.evidence())
            })
            .complete()
    }

    pub(super) fn map(&self, resource: &str) -> Result<&TagMap, TagShapeError> {
        match self {
            Self::KeyValueList(map) | Self::KeyValueObject(map) | Self::StringMap(map) => Ok(map),
            Self::NotTag {
                reason,
                candidate: _candidate,
                pointer,
            } => Err(TagShapeError::NotTag {
                resource: resource.into(),
                pointer: pointer.clone(),
                reason: reason.clone(),
            }),
            Self::Unreadable {
                source,
                candidate: _candidate,
                pointer,
            } => Err(TagShapeError::Unreadable {
                resource: resource.into(),
                pointer: pointer.clone(),
                source: Box::new(source.clone()),
            }),
        }
    }
}

/// Names are evidence, never admission. In particular, fields split between
/// mutually exclusive arms still require an explicit NonTagList declaration.
#[derive(Debug, Clone, Default)]
struct Evidence {
    fields: BTreeSet<String>,
    required: BTreeSet<String>,
    item_fields: BTreeSet<String>,
    item_required: BTreeSet<String>,
    rejection: Option<NotTagReason>,
    cycle: bool,
}

impl Evidence {
    fn extend(&mut self, other: &Self) {
        self.cycle |= other.cycle;
        self.fields.extend(other.fields.iter().cloned());
        self.required.extend(other.required.iter().cloned());
        self.item_fields.extend(other.item_fields.iter().cloned());
        self.item_required
            .extend(other.item_required.iter().cloned());
        self.rejection = self.rejection.take().or_else(|| other.rejection.clone());
    }

    fn candidate(&self) -> TagCandidate {
        TagCandidate {
            key: self
                .item_fields
                .iter()
                .any(|name| name.eq_ignore_ascii_case("key")),
            value: self
                .item_fields
                .iter()
                .any(|name| name.eq_ignore_ascii_case("value")),
        }
    }
}

#[derive(Debug, Clone)]
struct Analysis {
    projection: Result<Projection, SchemaError>,
    evidence: Evidence,
}

/// This is the tag projection policy: references and allOf intersect using the
/// shared projection algebra, unions require agreement, and conditional facts
/// remain conditional. It memoizes acyclic nodes, including failed
/// arms, and excludes child object scopes from resource-tag evidence.
struct Classifier<'a> {
    layer: &'a Arc<Layer>,
    active: BTreeSet<(NodeId, bool)>,
    memo: BTreeMap<(NodeId, bool), Analysis>,
}

impl Classifier<'_> {
    fn analyze(&mut self, node: NodeId, crossed_items: bool) -> Analysis {
        let site = (node, crossed_items);
        if let Some(analysis) = self.memo.get(&site) {
            return analysis.clone();
        }
        if !self.active.insert(site) {
            // Reference/allOf back edges are neutral in an idempotent
            // intersection. Siblings supply the productive fixed point.
            return Analysis {
                projection: Ok(Projection::default()),
                evidence: Evidence {
                    cycle: true,
                    ..Evidence::default()
                },
            };
        }
        let mut result = Analysis {
            projection: schema_projection::direct(self.layer, node),
            evidence: Evidence::default(),
        };
        match self.layer.node(node) {
            SchemaNode::Boolean(_value) => {}
            SchemaNode::Keywords(k) => {
                result
                    .evidence
                    .fields
                    .extend(k.properties.iter().flat_map(|p| p.keys()).cloned());
                result
                    .evidence
                    .required
                    .extend(k.required.iter().flatten().cloned());
                for (child, contains) in [(k.items, false), (k.contains, true)] {
                    if let Some(child) = child {
                        let item = self.analyze(child, true);
                        result.evidence.cycle |= item.evidence.cycle;
                        result.evidence.item_fields.extend(
                            item.evidence
                                .fields
                                .iter()
                                .chain(&item.evidence.item_fields)
                                .cloned(),
                        );
                        result.evidence.item_required.extend(
                            item.evidence
                                .required
                                .iter()
                                .chain(&item.evidence.item_required)
                                .cloned(),
                        );
                        result.evidence.rejection =
                            result.evidence.rejection.take().or(item.evidence.rejection);
                        if contains {
                            result.evidence.rejection = Some(NotTagReason::Contains);
                        } else {
                            result.projection = result.projection.and_then(|mut p| {
                                let item = item.projection?;
                                p.items = Some(Box::new(schema_projection::materialize(item)));
                                Ok(p)
                            });
                        }
                    }
                }
                if let Some(reference) = &k.reference {
                    match &reference.target {
                        RefTarget::Node(target) => {
                            let other = self.analyze(*target, crossed_items);
                            result.evidence.extend(&other.evidence);
                            let source = other
                                .projection
                                .as_ref()
                                .ok()
                                .map(|p| (p.source_pointer.clone(), p.source_name.clone()));
                            result.projection =
                                self.intersect(node, result.projection, other.projection);
                            if k.pattern.is_none()
                                && let (Ok(p), Some((pointer, name))) =
                                    (&mut result.projection, source)
                            {
                                p.source_pointer = pointer;
                                p.source_name = name;
                            }
                        }
                        RefTarget::Unresolved(error) => result.projection = Err(error.clone()),
                    }
                }
                for arm in k.compositions.all_of.iter().flatten() {
                    let other = self.analyze(*arm, crossed_items);
                    result.evidence.extend(&other.evidence);
                    result.projection = self.intersect(node, result.projection, other.projection);
                }
                for (exclusive, arms) in [
                    (true, &k.compositions.one_of),
                    (false, &k.compositions.any_of),
                ] {
                    let Some(arms) = arms else {
                        continue;
                    };
                    let mut projected = Vec::new();
                    for arm in arms {
                        let other = self.analyze(*arm, crossed_items);
                        result.evidence.extend(&other.evidence);
                        // Nullability does not add another wire shape.
                        if other
                            .projection
                            .as_ref()
                            .is_ok_and(|p| p.types == [JsonType::Null])
                        {
                            continue;
                        }
                        projected.push(self.intersect(
                            node,
                            result.projection.clone(),
                            other.projection,
                        ));
                    }
                    if let Some(first) = projected.first() {
                        for other in projected.iter().skip(1) {
                            match (first, other) {
                                (Ok(first), Ok(other)) => {
                                    match schema_projection::equivalent_tag_projection(first, other)
                                    {
                                        Ok(true) => {}
                                        Ok(false) => {
                                            result.evidence.rejection =
                                                Some(NotTagReason::Alternatives)
                                        }
                                        Err(error) => result.projection = Err(error),
                                    }
                                }
                                (Err(error), Ok(_other)) | (Ok(_other), Err(error)) => {
                                    result.projection = Err(error.clone())
                                }
                                (Err(error), Err(_other_error)) => {
                                    result.projection = Err(error.clone())
                                }
                            }
                        }
                        if exclusive
                            && projected.len() > 1
                            && result.evidence.rejection.is_none()
                            && result.projection.is_ok()
                        {
                            result.projection = Err(SchemaError::Projection {
                                pointer: self.layer.sources().pointer(node).into(),
                                reason: "overlapping equivalent oneOf tag alternatives require exclusive constraint emission".into(),
                            });
                        }
                        result.projection = match &result.projection {
                            Ok(_projection) => first.clone(),
                            Err(error) => Err(error.clone()),
                        };
                    }
                }
                for dependency in k.dependencies.iter().flat_map(|d| d.values()) {
                    match dependency {
                        Dependency::Schema(child) => {
                            let conditional = self.analyze(*child, crossed_items);
                            result.evidence.extend(&conditional.evidence);
                            result.evidence.rejection = Some(NotTagReason::Dependency);
                        }
                        Dependency::PropertyList(names) => {
                            result.evidence.required.extend(names.iter().cloned());
                            if !names.iter().all(|name| {
                                result
                                    .projection
                                    .as_ref()
                                    .is_ok_and(|p| p.required.contains(name))
                            }) {
                                result.evidence.rejection = Some(NotTagReason::Dependency);
                            }
                        }
                    }
                }
            }
        }
        result.projection = result.projection.and_then(|p| {
            schema_projection::validate(self.layer, node, &p)?;
            Ok(p)
        });
        self.active.remove(&site);
        // A partial SCC member may not yet include its caller's siblings.
        // Do not reuse that partial result from another entry into the cycle.
        if !result.evidence.cycle {
            self.memo.insert(site, result.clone());
        }
        result
    }

    fn intersect(
        &self,
        node: NodeId,
        first: Result<Projection, SchemaError>,
        second: Result<Projection, SchemaError>,
    ) -> Result<Projection, SchemaError> {
        let mut first = first?;
        schema_projection::merge(self.layer, node, &mut first, second?)?;
        Ok(first)
    }

    fn string(
        &mut self,
        property: &super::SchemaProperty,
    ) -> Result<Option<TagString>, SchemaError> {
        let projection = match &property.origin {
            Some(origin) => {
                let analysis = self.analyze(origin.node, false);
                if analysis.evidence.rejection.is_some() {
                    return Ok(None);
                }
                let projection = analysis.projection?;
                if analysis.evidence.cycle && projection.effective_type().is_none() {
                    return Err(
                        projection.emission_error("unproductive tag reference/composition cycle")
                    );
                }
                projection
            }
            None => property.try_query()?.clone(),
        };
        if projection.effective_type() == Some("string") {
            if projection.enum_values.is_some() || projection.const_value.is_some() {
                return Err(projection.emission_error(
                    "tag string enum/const constraints require a literal-preserving map projection",
                ));
            }
            Ok(Some(TagString {
                source: projection.source_pointer,
                pattern: projection.pattern,
                min_length: projection.min_length,
                max_length: projection.max_length,
            }))
        } else {
            Ok(None)
        }
    }

    fn object(
        &mut self,
        p: &Projection,
        max_entries: Option<i64>,
    ) -> Result<Option<TagMap>, SchemaError> {
        if p.effective_type() != Some("object") {
            return Ok(None);
        }
        let definition_name = p
            .source_pointer
            .strip_prefix("/definitions/")
            .filter(|name| !name.contains('/'))
            .map(super::decode_json_pointer_segment);
        if let Some(properties) = &p.properties
            && properties.len() == 2
            && p.pattern_properties.as_ref().is_none_or(BTreeMap::is_empty)
            && p.additional_schema.is_none()
            && p.required
                .iter()
                .all(|name| matches!(name.as_str(), "Key" | "Value"))
            && let (Some(key), Some(value)) = (properties.get("Key"), properties.get("Value"))
            && let (Some(key), Some(value)) = (self.string(key)?, self.string(value)?)
        {
            return Ok(Some(TagMap {
                key: TagKey::Property(key),
                value,
                min_entries: None,
                max_entries,
                definition_name,
                candidate: TagCandidate::default(),
            }));
        }
        if p.properties.as_ref().is_none_or(BTreeMap::is_empty)
            && p.required.is_empty()
            && p.additional_properties == Some(false)
            && let Some(patterns) = &p.pattern_properties
            && patterns.len() == 1
            && let Some((pattern, value)) = patterns.iter().next()
            && let Some(value) = self.string(value)?
        {
            let bound = |number: &Option<serde_json::Number>, keyword| {
                number
                    .as_ref()
                    .map(|number| {
                        number.as_i64().ok_or_else(|| {
                            p.emission_error(format!(
                                "Tags {keyword} cannot fit the map projection exactly: {number}"
                            ))
                        })
                    })
                    .transpose()
            };
            return Ok(Some(TagMap {
                key: TagKey::Pattern {
                    pattern: pattern.clone(),
                    source: format!("{}/patternProperties", p.source_pointer),
                },
                value,
                min_entries: bound(&p.min_properties, "minProperties")?,
                max_entries: bound(&p.max_properties, "maxProperties")?,
                definition_name,
                candidate: TagCandidate::default(),
            }));
        }
        Ok(None)
    }
}

pub(super) fn classify_tag_shape(layer: &Arc<Layer>, property: NodeId) -> &TagShape {
    layer
        .tag_shape_cache(property)
        .get_or_init(|| classify(layer, property))
}

fn classify(layer: &Arc<Layer>, property: NodeId) -> TagShape {
    let mut classifier = Classifier {
        layer,
        active: BTreeSet::new(),
        memo: BTreeMap::new(),
    };
    let analysis = classifier.analyze(property, false);
    let candidate = analysis.evidence.candidate();
    let pointer = layer.sources().pointer(property).to_string();
    let not_tag = |reason| TagShape::NotTag {
        reason,
        candidate,
        pointer: pointer.clone(),
    };
    let unreadable = |source| TagShape::Unreadable {
        source,
        candidate,
        pointer: pointer.clone(),
    };
    let p = match analysis.projection {
        Ok(p) => p,
        Err(error) => return unreadable(error),
    };
    let (fields, required) = if p.effective_type() == Some("array") || candidate.complete() {
        (
            &analysis.evidence.item_fields,
            &analysis.evidence.item_required,
        )
    } else {
        (&analysis.evidence.fields, &analysis.evidence.required)
    };
    let extra = fields
        .union(required)
        .filter(|name| !matches!(name.as_str(), "Key" | "Value"))
        .cloned()
        .collect::<Vec<_>>();
    if candidate.complete() && !extra.is_empty() {
        return not_tag(NotTagReason::ExtraFields(extra.join(", ")));
    }
    if let Some(reason) = analysis.evidence.rejection {
        return not_tag(reason);
    }
    if p.effective_type() == Some("array") {
        if p.max_items.is_some_and(|max| max < 0) {
            return unreadable(p.emission_error("Tags maxItems must be nonnegative"));
        }
        if let Some(item) = &p.items {
            let item = match item.try_query() {
                Ok(item) => item,
                Err(error) => return unreadable(error),
            };
            match classifier.object(item, p.max_items) {
                Ok(Some(mut map)) => {
                    map.min_entries = p.min_items;
                    match &map.key {
                        TagKey::Property(_key) => return TagShape::KeyValueList(map),
                        TagKey::Pattern {
                            pattern: _pattern,
                            source: _source,
                        } => {}
                    }
                }
                Ok(None) => {}
                Err(error) => return unreadable(error),
            }
        }
    } else {
        match classifier.object(&p, None) {
            Ok(Some(mut map)) => {
                map.candidate = candidate;
                return match &map.key {
                    TagKey::Property(_key) => TagShape::KeyValueObject(map),
                    TagKey::Pattern {
                        pattern: _pattern,
                        source: _source,
                    } => TagShape::StringMap(map),
                };
            }
            Ok(None) => {}
            Err(error) => return unreadable(error),
        }
    }
    if analysis.evidence.cycle {
        unreadable(p.emission_error("unproductive tag reference/composition cycle"))
    } else {
        not_tag(NotTagReason::Shape)
    }
}
