//! Record source pointers at emitter decisions, then account for raw KMS nodes once.

use super::kms_references::{self, Classification, PathSegment, ReferenceKind, property_path};
use super::{CfnProperty, CfnSchema, dsl_resource_name_from_type, is_property_read_only};
use anyhow::{Result, bail};
use heck::ToSnakeCase;
use serde_json::Value;
use std::borrow::Cow;
use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet};

struct UnemittedKmsNode {
    resource: &'static str,
    pointer: &'static str,
    reason: &'static str,
}

/// Exact source nodes that do not become independently classified leaves.
/// Descendants are never exempted; each candidate needs its own accounting.
const UNEMITTED_KMS_NODES: &[UnemittedKmsNode] = &[
    UnemittedKmsNode {
        resource: "dynamodb.Table",
        pointer: "/propertyTransform/~1properties~1SSESpecification~1KMSMasterKeyId",
        reason: "Cloud Control read transform metadata, not an emitted attribute.",
    },
    UnemittedKmsNode {
        resource: "ecs.Cluster",
        pointer: "/propertyTransform/~1properties~1Configuration~1ManagedStorageConfiguration~1FargateEphemeralStorageKmsKeyId",
        reason: "Cloud Control read transform metadata, not an emitted attribute.",
    },
    UnemittedKmsNode {
        resource: "kms.Key",
        pointer: "",
        reason: "Resource description, not an attribute.",
    },
    UnemittedKmsNode {
        resource: "kms.Key",
        pointer: "/properties/Tags",
        reason: "Tag-pair collection projected to a string map; the map value is classified separately.",
    },
    UnemittedKmsNode {
        resource: "ecr.Repository",
        pointer: "/definitions/EncryptionConfiguration",
        reason: "Struct container describing encryption, not a key identifier leaf.",
    },
    UnemittedKmsNode {
        resource: "s3.Bucket",
        pointer: "/properties/BucketEncryption",
        reason: "Use of the bucket-encryption struct, not a key identifier leaf.",
    },
    UnemittedKmsNode {
        resource: "s3.Bucket",
        pointer: "/definitions/BucketEncryption",
        reason: "Bucket-encryption struct container.",
    },
    UnemittedKmsNode {
        resource: "s3.Bucket",
        pointer: "/definitions/ServerSideEncryptionRule",
        reason: "Encryption-rule struct container.",
    },
    UnemittedKmsNode {
        resource: "s3.Bucket",
        pointer: "/definitions/ServerSideEncryptionByDefault",
        reason: "Default-encryption struct container.",
    },
    UnemittedKmsNode {
        resource: "s3.Bucket",
        pointer: "/definitions/EncryptionConfiguration",
        reason: "Replication-encryption struct container.",
    },
    UnemittedKmsNode {
        resource: "s3.Bucket",
        pointer: "/definitions/SourceSelectionCriteria/properties/SseKmsEncryptedObjects",
        reason: "Use of the replication-selection struct, not a key identifier leaf.",
    },
    UnemittedKmsNode {
        resource: "s3.Bucket",
        pointer: "/definitions/SseKmsEncryptedObjects",
        reason: "Replication-selection struct container.",
    },
];

pub(super) struct EmissionContext<'a> {
    schema: &'a CfnSchema,
    source: Cow<'a, Value>,
    resource: String,
    visited: RefCell<BTreeSet<String>>,
    classified_sources: RefCell<BTreeSet<String>>,
    /// Collection suffixes come from actual emitter decisions, never lookahead.
    paths: RefCell<BTreeMap<Vec<String>, String>>,
    sites: RefCell<BTreeMap<Vec<String>, EmittedSite>>,
    enum_discovery: bool,
}

pub(super) struct EmissionProperty<'a> {
    schema: &'a EmissionContext<'a>,
    property: &'a CfnProperty,
    path: &'a [String],
}

impl<'a> EmissionProperty<'a> {
    pub(super) fn schema(&self) -> &'a EmissionContext<'a> {
        self.schema
    }
    pub(super) fn property(&self) -> &'a CfnProperty {
        self.property
    }
    pub(super) fn path(&self) -> &'a [String] {
        self.path
    }
    pub(super) fn name(&self) -> &'a str {
        self.path
            .iter()
            .rev()
            .find(|segment| segment.as_str() != "*")
            .expect("property name")
    }
    pub(super) fn classify(&self) -> Result<Option<ReferenceKind>> {
        if self.schema.enum_discovery
            || is_property_read_only(self.schema, self.property, self.path)
        {
            return Ok(None);
        }
        let site = self.schema.sites.borrow()[self.path].clone();
        self.schema
            .classify(&self.schema.dsl_path(self.path), self.name(), &site, true)
    }
}

impl<'a> EmissionContext<'a> {
    pub(super) fn from_source(schema: &'a CfnSchema, source: &'a Value) -> Result<Self> {
        Ok(Self {
            schema,
            source: Cow::Borrowed(source),
            resource: dsl_resource_name_from_type(&schema.type_name)?,
            visited: RefCell::new(BTreeSet::new()),
            classified_sources: RefCell::new(BTreeSet::new()),
            paths: RefCell::new(BTreeMap::new()),
            sites: RefCell::new(BTreeMap::new()),
            enum_discovery: false,
        })
    }

    #[cfg(test)]
    pub(super) fn new(schema: &'a CfnSchema) -> Result<Self> {
        // Unit tests mutate parsed fixtures. Production always uses the original JSON.
        let mut source = serde_json::to_value(schema)?;
        fn omit_absent_fields(value: &mut Value) {
            match value {
                Value::Object(fields) => {
                    fields.retain(|_, value| !value.is_null());
                    fields.values_mut().for_each(omit_absent_fields);
                }
                Value::Array(values) => values.iter_mut().for_each(omit_absent_fields),
                _ => {}
            }
        }
        omit_absent_fields(&mut source);
        Ok(Self {
            source: Cow::Owned(source),
            ..Self::from_source(schema, &Value::Null)?
        })
    }

    pub(super) fn for_enum_discovery(schema: &'a CfnSchema) -> Result<Self> {
        Ok(Self {
            enum_discovery: true,
            ..Self::from_source(schema, &Value::Null)?
        })
    }

    pub(super) fn property_at<'b>(
        &'b self,
        path: &'b [String],
        property: &'b CfnProperty,
    ) -> EmissionProperty<'b> {
        if !self.enum_discovery && !self.sites.borrow().contains_key(path) {
            let (name, parent_path) = path.split_last().expect("property path");
            let parents = if parent_path.is_empty() {
                self.same_site_sources([String::new()])
            } else {
                self.sites
                    .borrow()
                    .get(parent_path)
                    .map(|site| site.sources.clone())
                    .unwrap_or_default()
            };
            let mut sources = Vec::new();
            let mut inherited = Evidence::default();
            for parent in parents {
                let value = self
                    .source
                    .pointer(&parent)
                    .expect("recorded source pointer");
                if name == "*" {
                    // Collections retain same-site scalar constraints on their value.
                    inherited.add(value);
                    if self.dsl_path(path).ends_with("{}") {
                        if let Some(patterns) =
                            value.get("patternProperties").and_then(Value::as_object)
                        {
                            sources.extend(patterns.keys().map(|key| {
                                pointer_child(&pointer_child(&parent, "patternProperties"), key)
                            }));
                        }
                    } else if value.get("items").is_some() {
                        sources.push(pointer_child(&parent, "items"));
                    }
                } else if value
                    .get("properties")
                    .and_then(|fields| fields.get(name))
                    .is_some()
                {
                    sources.push(pointer_child(&pointer_child(&parent, "properties"), name));
                }
            }
            let sources = self.same_site_sources(sources);
            let mut site = EmittedSite {
                sources,
                evidence: inherited,
            };
            for pointer in &site.sources {
                site.evidence
                    .add(self.source.pointer(pointer).expect("source node"));
            }
            #[cfg(test)]
            if site.sources.is_empty() {
                let mut value = serde_json::to_value(property).expect("unit-test property");
                value
                    .as_object_mut()
                    .unwrap()
                    .retain(|_, value| !value.is_null());
                site.evidence.add(&value);
            }
            self.sites.borrow_mut().insert(path.to_vec(), site);
        }
        EmissionProperty {
            schema: self,
            property,
            path,
        }
    }

    /// Follow only same-site evidence. Children are selected when the emitter visits them.
    fn same_site_sources(&self, roots: impl IntoIterator<Item = String>) -> Vec<String> {
        let mut sources = Vec::new();
        let mut pending: Vec<_> = roots.into_iter().collect();
        let mut seen = BTreeSet::new();
        while let Some(pointer) = pending.pop() {
            if !seen.insert(pointer.clone()) {
                continue;
            }
            let Some(value) = self.source.pointer(&pointer) else {
                continue;
            };
            sources.push(pointer.clone());
            if let Some(reference) = value
                .get("$ref")
                .and_then(Value::as_str)
                .and_then(|reference| reference.strip_prefix('#'))
            {
                pending.push(reference.to_string());
            }
            for keyword in ["oneOf", "anyOf", "allOf"] {
                if let Some(branches) = value.get(keyword).and_then(Value::as_array) {
                    pending.extend(
                        (0..branches.len())
                            .rev()
                            .map(|index| format!("{pointer}/{keyword}/{index}")),
                    );
                }
            }
        }
        sources
    }

    pub(super) fn collection_path(
        &self,
        parent: &[String],
        segment: PathSegment<'_>,
    ) -> Vec<String> {
        let path = super::extend_property_path(parent, "*");
        if !self.enum_discovery {
            let dsl_path = property_path(&self.dsl_path(parent), segment);
            self.paths.borrow_mut().insert(path.clone(), dsl_path);
        }
        path
    }

    fn dsl_path(&self, path: &[String]) -> String {
        let paths = self.paths.borrow();
        let mut result = String::new();
        for (index, segment) in path.iter().enumerate() {
            result = if let Some(recorded) = paths.get(&path[..=index]) {
                recorded.clone()
            } else if segment == "*" {
                property_path(&result, PathSegment::ListElement)
            } else {
                property_path(&result, PathSegment::Field(&segment.to_snake_case()))
            };
        }
        result
    }

    fn classify(
        &self,
        path: &str,
        name: &str,
        site: &EmittedSite,
        record_sources: bool,
    ) -> Result<Option<ReferenceKind>> {
        let evidence = &site.evidence;
        let decision = kms_references::classify_string(
            &self.resource,
            path,
            name,
            Some(&evidence.patterns.join(" ")),
            Some(&evidence.descriptions.join(" ")),
        )?;
        if decision.is_some() {
            self.visited.borrow_mut().insert(path.to_string());
            if record_sources {
                self.classified_sources
                    .borrow_mut()
                    .extend(site.sources.iter().cloned());
            }
        }
        let Some(Classification::Reference(kind)) = decision else {
            return Ok(None);
        };
        let entry = kms_references::entry(&self.resource, path).expect("classified reference");
        if evidence.has_enum_or_const {
            bail!(
                "{}.{path}: KMS enum/const constraint is incompatible with the reference constructor; revisit the classification table entry",
                self.resource
            );
        }
        if evidence
            .patterns
            .iter()
            .map(String::as_str)
            .ne(entry.cfn_pattern)
        {
            bail!(
                "{}.{path}: KMS pattern changed from {:?} to {:?}; classify its constraint before generation",
                self.resource,
                entry.cfn_pattern,
                evidence.patterns
            );
        }
        if evidence.lengths.as_slice() != entry.cfn_length.as_slice() {
            bail!(
                "{}.{path}: KMS length changed from {:?} to {:?}; classify its constraint before generation",
                self.resource,
                entry.cfn_length,
                evidence.lengths
            );
        }
        Ok(Some(kind))
    }

    pub(super) fn reference_constructor(&self, path: &[String], kind: ReferenceKind) -> String {
        kind.constructor(
            kms_references::entry(&self.resource, &self.dsl_path(path))
                .and_then(|entry| entry.cfn_length),
        )
    }

    /// The projected map value has no independent CFN node to record.
    pub(super) fn classify_tag_value(&self, path: &[String]) -> Result<()> {
        if !self.enum_discovery {
            let site = self.sites.borrow()[path].clone();
            self.classify(
                &property_path(&self.dsl_path(path), PathSegment::MapValue),
                "",
                &site,
                false,
            )?;
        }
        Ok(())
    }

    pub(super) fn finish(&self) -> Result<()> {
        let candidates = kms_nodes(&self.source);
        let classified = self.classified_sources.borrow();
        let exceptions: Vec<_> = UNEMITTED_KMS_NODES
            .iter()
            .filter(|entry| entry.resource == self.resource)
            .collect();
        for pointer in &candidates {
            if !classified.contains(pointer)
                && !exceptions.iter().any(|entry| entry.pointer == pointer)
            {
                bail!(
                    "unaccounted KMS node {}#{pointer}; classify its emitted source or add an explicit UNEMITTED_KMS_NODES entry with a reason",
                    self.resource
                );
            }
        }
        let mut unique = BTreeSet::new();
        for entry in exceptions {
            if entry.reason.trim().is_empty() || !unique.insert(entry.pointer) {
                bail!(
                    "invalid UNEMITTED_KMS_NODES entry {}#{}",
                    self.resource,
                    entry.pointer
                );
            }
            if !candidates.contains(entry.pointer) || classified.contains(entry.pointer) {
                bail!(
                    "unused UNEMITTED_KMS_NODES entry {}#{}; remove or revisit the stale entry",
                    self.resource,
                    entry.pointer
                );
            }
        }
        for entry in kms_references::CLASSIFICATIONS
            .iter()
            .filter(|entry| entry.resource == self.resource)
        {
            if !self.visited.borrow().contains(entry.path) {
                bail!(
                    "unvisited KMS classification {}.{}; remove or revisit the stale table entry",
                    self.resource,
                    entry.path
                );
            }
        }
        Ok(())
    }
}

impl std::ops::Deref for EmissionContext<'_> {
    type Target = CfnSchema;
    fn deref(&self) -> &Self::Target {
        self.schema
    }
}

#[derive(Clone, Default)]
struct Evidence {
    patterns: Vec<String>,
    lengths: Vec<(Option<u64>, Option<u64>)>,
    descriptions: Vec<String>,
    has_enum_or_const: bool,
}

impl Evidence {
    fn add(&mut self, value: &Value) {
        self.has_enum_or_const |= value.get("enum").is_some() || value.get("const").is_some();
        if let Some(pattern) = value.get("pattern").and_then(Value::as_str)
            && !self.patterns.iter().any(|old| old == pattern)
        {
            self.patterns.push(pattern.to_string());
        }
        let length = (
            value.get("minLength").and_then(Value::as_u64),
            value.get("maxLength").and_then(Value::as_u64),
        );
        if length != (None, None) && !self.lengths.contains(&length) {
            self.lengths.push(length);
        }
        if let Some(description) = value.get("description").and_then(Value::as_str)
            && !self.descriptions.iter().any(|old| old == description)
        {
            self.descriptions.push(description.to_string());
        }
    }
}

#[derive(Clone, Default)]
struct EmittedSite {
    sources: Vec<String>,
    evidence: Evidence,
}

fn pointer_child(parent: &str, key: &str) -> String {
    format!("{parent}/{}", key.replace('~', "~0").replace('/', "~1"))
}

/// Pure JSON accounting: no $ref expansion, shape inference, or schema validation.
fn kms_nodes(source: &Value) -> BTreeSet<String> {
    fn scan(value: &Value, pointer: &str, name: &str, found: &mut BTreeSet<String>) {
        if kms_references::is_candidate(
            name,
            value.get("pattern").and_then(Value::as_str),
            value.get("description").and_then(Value::as_str),
        ) {
            found.insert(pointer.to_string());
        }
        match value {
            Value::Object(fields) => {
                for (key, value) in fields {
                    scan(value, &pointer_child(pointer, key), key, found);
                }
            }
            Value::Array(values) => {
                for (index, value) in values.iter().enumerate() {
                    scan(value, &format!("{pointer}/{index}"), "", found);
                }
            }
            _ => {}
        }
    }
    let mut found = BTreeSet::new();
    scan(source, "", "", &mut found);
    found
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn scan_accounts_for_all_json_branches_and_escapes_pointer_segments() {
        let source = json!({
            "properties": {"Kms/a~b": false},
            "definitions": {"Ordinary": {"description": "A customer master key"}},
            "allOf": [{"items": {"pattern": "^arn:aws:kms:"}}],
            "anyOf": [{"patternProperties": {".*": {"description": "cMk"}}}],
            "oneOf": [{"additionalProperties": {"description": "AWS KMS"}}],
            "x-future": [null, true, 17, {"unknown": [{"description": "AWS KMS"}]}]
        });
        assert_eq!(
            kms_nodes(&source),
            BTreeSet::from([
                "/properties/Kms~1a~0b".into(),
                "/definitions/Ordinary".into(),
                "/allOf/0/items".into(),
                "/anyOf/0/patternProperties/.*".into(),
                "/oneOf/0/additionalProperties".into(),
                "/x-future/3/unknown/0".into(),
            ])
        );
        for unknown in [json!(null), json!(17), json!([false, {"items": true}])] {
            assert!(kms_nodes(&unknown).is_empty());
        }
    }

    #[test]
    fn setup_and_source_selection_do_not_classify_or_cover_nodes() {
        let source = json!({"typeName": "AWS::SNS::Topic", "properties": {
            "KmsMasterKeyId": {"type": "string"}
        }});
        let schema: CfnSchema = serde_json::from_value(source.clone()).unwrap();
        let context = EmissionContext::from_source(&schema, &source).unwrap();
        assert!(context.sites.borrow().is_empty());
        let path = ["KmsMasterKeyId".into()];
        let property = context.property_at(&path, &schema.properties["KmsMasterKeyId"]);
        assert!(context.visited.borrow().is_empty());
        assert!(context.classified_sources.borrow().is_empty());
        assert!(
            context
                .finish()
                .unwrap_err()
                .to_string()
                .contains("unaccounted KMS node")
        );
        assert!(property.classify().unwrap().is_some());
        context.finish().unwrap();
    }

    #[test]
    fn a_leaf_decision_does_not_cover_unemitted_descendants() {
        let source = json!({"typeName": "AWS::SNS::Topic", "properties": {
            "KmsMasterKeyId": {"type": "string", "anyOf": [{"properties": {
                "KmsHidden": {"type": "string"}
            }}]}
        }});
        let schema: CfnSchema = serde_json::from_value(source.clone()).unwrap();
        let context = EmissionContext::from_source(&schema, &source).unwrap();
        let path = ["KmsMasterKeyId".into()];
        context
            .property_at(&path, &schema.properties["KmsMasterKeyId"])
            .classify()
            .unwrap();
        let error = context.finish().unwrap_err().to_string();
        assert!(
            error.contains("/properties/KmsMasterKeyId/anyOf/0/properties/KmsHidden"),
            "{error}"
        );
    }

    #[test]
    fn source_discovery_follows_ref_cycles_without_walking_children() {
        let source = json!({"typeName": "AWS::EC2::VPC", "properties": {
            "Settings": {"$ref": "#/definitions/Cycle"}
        }, "definitions": {"Cycle": {"$ref": "#/definitions/Cycle", "properties": {
            "KmsHidden": {"type": "string"}
        }}}});
        let schema: CfnSchema = serde_json::from_value(source.clone()).unwrap();
        let context = EmissionContext::from_source(&schema, &source).unwrap();
        let path = ["Settings".into()];
        context.property_at(&path, &schema.properties["Settings"]);
        assert_eq!(context.sites.borrow()[&path[..]].sources.len(), 2);
        let error = context.finish().unwrap_err().to_string();
        assert!(
            error.contains("/definitions/Cycle/properties/KmsHidden"),
            "{error}"
        );
    }
}
