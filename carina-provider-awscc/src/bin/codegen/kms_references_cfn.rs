//! Record source pointers at emitter decisions, then account for raw KMS nodes once.

use super::kms_references::{self, Classification, PathSegment, ReferenceKind, property_path};
use super::{
    CfnProperty, CfnPropertyPath, CfnSchema, ParsedCfnSchema, dsl_resource_name_from_type,
    emit_pattern_option, is_property_read_only, string_constraints,
};
use anyhow::{Result, bail};
use heck::ToSnakeCase;
use serde_json::Value;
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
        resource: "dynamodb.Table",
        pointer: "/properties/SSESpecification",
        reason: "Use of the server-side-encryption struct; its leaves are classified separately.",
    },
    UnemittedKmsNode {
        resource: "ecs.Cluster",
        pointer: "/propertyTransform/~1properties~1Configuration~1ManagedStorageConfiguration~1FargateEphemeralStorageKmsKeyId",
        reason: "Cloud Control read transform metadata, not an emitted attribute.",
    },
    UnemittedKmsNode {
        resource: "ecs.Cluster",
        pointer: "/properties/Configuration",
        reason: "Use of the cluster-configuration struct; its leaves are classified separately.",
    },
    UnemittedKmsNode {
        resource: "ecs.Cluster",
        pointer: "/definitions/ClusterConfiguration/properties/ExecuteCommandConfiguration",
        reason: "Use of the execute-command configuration struct; its leaves are classified separately.",
    },
    UnemittedKmsNode {
        resource: "ecs.Cluster",
        pointer: "/definitions/ClusterConfiguration/properties/ManagedStorageConfiguration",
        reason: "Use of the managed-storage configuration struct; its leaves are classified separately.",
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
        resource: "ecr.Repository",
        pointer: "/properties/EncryptionConfiguration",
        reason: "Use of the encryption-configuration struct, not a key identifier leaf.",
    },
    UnemittedKmsNode {
        resource: "s3.Bucket",
        pointer: "/properties/MetadataConfiguration",
        reason: "Use of the metadata-configuration struct; its leaves are classified separately.",
    },
    UnemittedKmsNode {
        resource: "s3.Bucket",
        pointer: "/definitions/MetadataConfiguration/properties/InventoryTableConfiguration",
        reason: "Use of the inventory-table configuration struct; its leaves are classified separately.",
    },
    UnemittedKmsNode {
        resource: "s3.Bucket",
        pointer: "/definitions/MetadataConfiguration/properties/JournalTableConfiguration",
        reason: "Use of the journal-table configuration struct; its leaves are classified separately.",
    },
    UnemittedKmsNode {
        resource: "s3.Bucket",
        pointer: "/definitions/InventoryTableConfiguration/properties/EncryptionConfiguration",
        reason: "Use of the metadata-table encryption struct; its leaves are classified separately.",
    },
    UnemittedKmsNode {
        resource: "s3.Bucket",
        pointer: "/definitions/JournalTableConfiguration/properties/EncryptionConfiguration",
        reason: "Use of the metadata-table encryption struct; its leaves are classified separately.",
    },
    UnemittedKmsNode {
        resource: "s3.Bucket",
        pointer: "/properties/ReplicationConfiguration",
        reason: "Use of the replication-configuration struct; its leaves are classified separately.",
    },
    UnemittedKmsNode {
        resource: "s3.Bucket",
        pointer: "/definitions/ReplicationConfiguration/properties/Rules/items",
        reason: "List element using the replication-rule struct; its leaves are classified separately.",
    },
    UnemittedKmsNode {
        resource: "s3.Bucket",
        pointer: "/definitions/ReplicationRule/properties/Destination",
        reason: "Use of the replication-destination struct; its leaves are classified separately.",
    },
    UnemittedKmsNode {
        resource: "s3.Bucket",
        pointer: "/definitions/ReplicationRule/properties/SourceSelectionCriteria",
        reason: "Use of the replication-selection struct; its leaves are classified separately.",
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
        pointer: "/definitions/BucketEncryption/properties/ServerSideEncryptionConfiguration/items",
        reason: "List element using the encryption-rule struct, not a key identifier leaf.",
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
        pointer: "/definitions/ServerSideEncryptionRule/properties/ServerSideEncryptionByDefault",
        reason: "Use of the default-encryption struct, not a key identifier leaf.",
    },
    UnemittedKmsNode {
        resource: "s3.Bucket",
        pointer: "/definitions/EncryptionConfiguration",
        reason: "Replication-encryption struct container.",
    },
    UnemittedKmsNode {
        resource: "s3.Bucket",
        pointer: "/definitions/ReplicationDestination/properties/EncryptionConfiguration",
        reason: "Use of the replication-encryption struct, not a key identifier leaf.",
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
    schema: &'a ParsedCfnSchema,
    source: &'a Value,
    resource: String,
    visited: RefCell<BTreeSet<String>>,
    /// Each source node is covered separately at every emitted DSL path.
    classified_sources: RefCell<BTreeMap<String, BTreeSet<String>>>,
    /// Collection suffixes come from actual emitter decisions, never lookahead.
    paths: RefCell<BTreeMap<Vec<String>, String>>,
    sites: RefCell<BTreeMap<Vec<String>, EmittedSite>>,
    /// Cached struct identities follow the CFN tree before type overrides.
    definition_identity_paths: BTreeMap<String, Vec<String>>,
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
    pub(super) fn from_source(schema: &'a ParsedCfnSchema) -> Result<Self> {
        Ok(Self {
            schema,
            source: schema.source(),
            resource: dsl_resource_name_from_type(&schema.type_name)?,
            visited: RefCell::new(BTreeSet::new()),
            classified_sources: RefCell::new(BTreeMap::new()),
            paths: RefCell::new(BTreeMap::new()),
            sites: RefCell::new(BTreeMap::new()),
            definition_identity_paths: super::collect_definition_identity_paths(schema),
            enum_discovery: false,
        })
    }

    /// Keep pattern diagnostics and registry lookups tied to the exact JSON
    /// leaf, through $refs, arrays, maps and merged/inline object traversal.
    pub(super) fn pattern_source_path(
        &self,
        path: &[String],
        source: Option<&str>,
        cfn_path: CfnPropertyPath<'_>,
    ) -> String {
        if let Some(site) = self.sites.borrow().get(path)
            && let Some(pointer) = site.sources.iter().find(|pointer| {
                self.source
                    .pointer(pointer)
                    .and_then(|node| node.get("pattern"))
                    .and_then(Value::as_str)
                    .is_some_and(|pattern| Some(pattern) == source)
            })
        {
            return pointer.clone();
        }
        // Isolated leaf tests and synthetic overlays may have no source site.
        // The canonical CFN key still identifies the leaf in that case.
        let mut pointer = cfn_path.source_pointer();
        for _ in path
            .iter()
            .rev()
            .take_while(|segment| segment.as_str() == "*")
        {
            pointer.push_str("/items");
        }
        pointer
    }

    pub(super) fn definition_identity_path(&self, name: &str) -> Option<&[String]> {
        self.definition_identity_paths.get(name).map(Vec::as_slice)
    }

    pub(super) fn for_enum_discovery(&self) -> Result<Self> {
        Ok(Self {
            enum_discovery: true,
            ..Self::from_source(self.schema)?
        })
    }

    pub(super) fn begin_definition_enum_discovery(&self, definition: &str) {
        assert!(self.enum_discovery);
        // Definition pre-scans reuse paths such as [Name] for different
        // definitions. Seed each scan at its actual source, not at resource
        // properties with the same names, and discard the previous paths.
        self.paths.borrow_mut().clear();
        let sources = self.same_site_sources([pointer_child("/definitions", definition)]);
        let mut sites = self.sites.borrow_mut();
        sites.clear();
        sites.insert(
            Vec::new(),
            EmittedSite {
                sources,
                evidence: Evidence::default(),
            },
        );
    }

    pub(super) fn property_at<'b>(
        &'b self,
        path: &'b [String],
        property: &'b CfnProperty,
    ) -> EmissionProperty<'b> {
        if !self.sites.borrow().contains_key(path) {
            let (name, parent_path) = path.split_last().expect("property path");
            let parents = self
                .sites
                .borrow()
                .get(parent_path)
                .map(|site| site.sources.clone())
                .unwrap_or_else(|| {
                    if parent_path.is_empty() {
                        self.same_site_sources([String::new()])
                    } else {
                        Vec::new()
                    }
                });
            let mut sources = Vec::new();
            let mut inherited = Evidence::default();
            for parent in parents {
                let value = self
                    .source
                    .pointer(&parent)
                    .expect("recorded source pointer");
                if name == "*" {
                    // Collections retain same-site scalar constraints on their value.
                    inherited.add_source(&parent, value);
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
                    .add_source(pointer, self.source.pointer(pointer).expect("source node"));
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
        let dsl_path = property_path(&self.dsl_path(parent), segment);
        self.paths.borrow_mut().insert(path.clone(), dsl_path);
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
        let names = format!("{name} {}", evidence.definition_names.join(" "));
        let decision = kms_references::classify_string(
            &self.resource,
            path,
            &names,
            Some(&evidence.patterns.join(" ")),
            Some(&evidence.descriptions.join(" ")),
            evidence.literals.iter().map(String::as_str),
        )?;
        if decision.is_some() {
            self.visited.borrow_mut().insert(path.to_string());
            if record_sources {
                let mut classified = self.classified_sources.borrow_mut();
                for pointer in &site.sources {
                    classified
                        .entry(pointer.clone())
                        .or_default()
                        .insert(path.to_string());
                }
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

    pub(super) fn reference_constructor(
        &self,
        path: &[String],
        kind: ReferenceKind,
        cfn_property_path: CfnPropertyPath<'_>,
    ) -> String {
        let entry = kms_references::entry(&self.resource, &self.dsl_path(path))
            .expect("classified reference");
        let Some(pattern) = entry.cfn_pattern else {
            let pattern =
                super::ApprovedPattern::whole(carina_aws_types::KMS_KEY_REFERENCE_PATTERN);
            return kind.constructor(entry.cfn_length, &pattern);
        };
        let property = CfnProperty {
            pattern: Some(pattern.to_string()),
            ..Default::default()
        };
        let source_path = self.pattern_source_path(path, Some(pattern), cfn_property_path);
        let constraints = string_constraints(
            &property,
            &self.schema.type_name,
            cfn_property_path,
            &source_path,
        );
        let pattern = emit_pattern_option(constraints.pattern.as_ref());
        format!(
            "carina_core::schema::AttributeType::reference(carina_aws_types::AwsType::KmsKeyReference.identity(), {pattern}, {:?})",
            entry.cfn_length
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
        let candidates = kms_nodes(self.source);
        let classified = self.classified_sources.borrow();
        let sites = self.sites.borrow();
        let exceptions: Vec<_> = UNEMITTED_KMS_NODES
            .iter()
            .filter(|entry| entry.resource == self.resource)
            .collect();
        for pointer in &candidates {
            if exceptions.iter().any(|entry| entry.pointer == pointer) {
                continue;
            }
            let Some(classified_paths) = classified.get(pointer) else {
                bail!(
                    "unaccounted KMS node {}#{pointer}; classify its emitted source or add an explicit UNEMITTED_KMS_NODES entry with a reason",
                    self.resource
                );
            };
            for (path, site) in sites.iter() {
                if site.sources.contains(pointer) {
                    let path = self.dsl_path(path);
                    if !classified_paths.contains(&path) {
                        bail!(
                            "unaccounted KMS node {}#{pointer} at emitted use site {}.{path}; classify this use site separately",
                            self.resource,
                            self.resource
                        );
                    }
                }
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
            if !candidates.contains(entry.pointer) || classified.contains_key(entry.pointer) {
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
    definition_names: Vec<String>,
    patterns: Vec<String>,
    lengths: Vec<(Option<u64>, Option<u64>)>,
    descriptions: Vec<String>,
    literals: Vec<String>,
    has_enum_or_const: bool,
}

impl Evidence {
    fn add_source(&mut self, pointer: &str, value: &Value) {
        if let Some(name) = pointer.strip_prefix("/definitions/")
            && !name.contains('/')
        {
            let name = name.replace("~1", "/").replace("~0", "~");
            if !self.definition_names.contains(&name) {
                self.definition_names.push(name);
            }
        }
        self.add(value);
    }

    fn add(&mut self, value: &Value) {
        self.has_enum_or_const |= value.get("enum").is_some() || value.get("const").is_some();
        self.literals.extend(
            value
                .get("enum")
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
                .chain(value.get("const"))
                .filter_map(Value::as_str)
                .map(str::to_string),
        );
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

/// Pure JSON accounting: track local $ref edges without expanding schema shapes.
fn kms_nodes(source: &Value) -> BTreeSet<String> {
    fn scan(
        value: &Value,
        pointer: &str,
        name: &str,
        found: &mut BTreeSet<String>,
        references: &mut BTreeMap<String, Vec<String>>,
    ) {
        if kms_references::is_candidate(
            name,
            value.get("pattern").and_then(Value::as_str),
            value.get("description").and_then(Value::as_str),
            value
                .get("enum")
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
                .chain(value.get("const"))
                .filter_map(Value::as_str),
        ) {
            found.insert(pointer.to_string());
        }
        if let Some(target) = value
            .get("$ref")
            .and_then(Value::as_str)
            .and_then(|reference| reference.strip_prefix('#'))
        {
            references
                .entry(target.to_string())
                .or_default()
                .push(pointer.to_string());
        }
        match value {
            Value::Object(fields) => {
                for (key, value) in fields {
                    scan(value, &pointer_child(pointer, key), key, found, references);
                }
            }
            Value::Array(values) => {
                for (index, value) in values.iter().enumerate() {
                    scan(value, &format!("{pointer}/{index}"), "", found, references);
                }
            }
            _ => {}
        }
    }
    let mut found = BTreeSet::new();
    let mut references = BTreeMap::new();
    scan(source, "", "", &mut found, &mut references);
    // Memoize subtree evidence at every source pointer, including definitions.
    // Propagation across containment and $ref edges visits each subtree once,
    // so shared definitions and cycles never require recursive rescanning.
    let mut subtrees_with_evidence = found.clone();
    let mut pending: Vec<_> = subtrees_with_evidence.iter().cloned().collect();
    while let Some(target) = pending.pop() {
        if let Some((parent, _)) = target.rsplit_once('/')
            && subtrees_with_evidence.insert(parent.to_string())
        {
            pending.push(parent.to_string());
        }
        if let Some(sites) = references.get(&target) {
            for pointer in sites {
                found.insert(pointer.clone());
                if subtrees_with_evidence.insert(pointer.clone()) {
                    pending.push(pointer.clone());
                }
            }
        }
    }
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
    fn scan_accounts_for_each_ref_by_target_evidence_through_aliases_and_cycles() {
        let source = json!({
            "definitions": {
                "Kms/a~b": {"$ref": "#/definitions/Alias"},
                "Alias": {"$ref": "#/definitions/Kms~1a~0b"},
                "Described": {"description": "A customer master key"},
                "Patterned": {"pattern": "^arn:aws:kms:"},
                "Plain": {"$ref": "#/definitions/Plain"}
            },
            "properties": {
                "ByName": {"$ref": "#/definitions/Alias"},
                "ByDescription": {"$ref": "#/definitions/Described"},
                "ByPattern": {"$ref": "#/definitions/Patterned"},
                "ByPlain": {"$ref": "#/definitions/Plain"},
                "Missing": {"$ref": "#/definitions/Missing"},
                "External": {"$ref": "external.json#/definitions/Described"}
            }
        });
        assert_eq!(
            kms_nodes(&source),
            BTreeSet::from([
                "/definitions/Kms~1a~0b".into(),
                "/definitions/Alias".into(),
                "/definitions/Described".into(),
                "/definitions/Patterned".into(),
                "/properties/ByName".into(),
                "/properties/ByDescription".into(),
                "/properties/ByPattern".into(),
            ])
        );
    }

    #[test]
    fn scan_follows_subtree_evidence_through_ref_cycles_and_shared_uses() {
        let source = json!({
            "definitions": {
                "First": {"type": "object", "properties": {
                    "Nested": {"$ref": "#/definitions/Second"}
                }},
                "Second": {"anyOf": [
                    {"$ref": "#/definitions/First"},
                    {"properties": {"KmsKeyId": {"type": "string"}}}
                ]},
                "PlainFirst": {"properties": {
                    "Nested": {"$ref": "#/definitions/PlainSecond"}
                }},
                "PlainSecond": {"$ref": "#/definitions/PlainFirst"}
            },
            "properties": {
                "FirstUse": {"$ref": "#/definitions/First"},
                "SecondUse": {"$ref": "#/definitions/Second"},
                "SharedUse": {"$ref": "#/definitions/First"},
                "PlainUse": {"$ref": "#/definitions/PlainFirst"},
                "Missing": {"$ref": "#/definitions/Missing"},
                "External": {"$ref": "external.json#/definitions/First"}
            }
        });
        assert_eq!(
            kms_nodes(&source),
            BTreeSet::from([
                "/definitions/First/properties/Nested".into(),
                "/definitions/Second/anyOf/0".into(),
                "/definitions/Second/anyOf/1/properties/KmsKeyId".into(),
                "/properties/FirstUse".into(),
                "/properties/SecondUse".into(),
                "/properties/SharedUse".into(),
            ])
        );
    }

    #[test]
    fn service_name_and_literal_evidence_belong_to_nodes_and_follow_refs() {
        for evidence in [
            json!({"description": "The AWS Key Management Service key identifier"}),
            json!({"pattern": "kEy MaNaGeMeNt SeRvIcE"}),
            json!({"enum": ["AES256", "KMS"]}),
            json!({"enum": ["AES256", "aws:kms:dsse"]}),
            json!({"const": "aWs:kMs"}),
        ] {
            let source = json!({
                "definitions": {
                    "SharedString": evidence,
                    "Alias": {"$ref": "#/definitions/SharedString"}
                },
                "properties": {"EncryptionMode": {"$ref": "#/definitions/Alias"}}
            });
            assert_eq!(
                kms_nodes(&source),
                BTreeSet::from([
                    "/definitions/SharedString".into(),
                    "/definitions/Alias".into(),
                    "/properties/EncryptionMode".into(),
                ]),
                "{evidence}"
            );
        }
        assert!(
            kms_nodes(&json!({
                "enum": ["AES256", 17, null, false],
                "const": 17,
                "default": "KMS",
                "examples": ["KMS"],
                "arbitrary": "KMS"
            }))
            .is_empty()
        );
    }

    #[test]
    fn setup_and_source_selection_do_not_classify_or_cover_nodes() {
        let source = json!({"typeName": "AWS::SNS::Topic", "properties": {
            "KmsMasterKeyId": {"type": "string"}
        }});
        let schema = ParsedCfnSchema::from_json(source).unwrap();
        let context = EmissionContext::from_source(&schema).unwrap();
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
    fn enum_discovery_pattern_provenance_tracks_definitions_maps_and_list_items() {
        let mapping = json!({"type": "object", "patternProperties": {"x/y": {
            "type": "object", "properties": {"Values": {
                "type": "array", "items": {"type": "string", "pattern": "[a-z]+"}
            }}
        }}});
        let schema = ParsedCfnSchema::from_json(json!({
            "typeName": "AWS::Test::Resource",
            "properties": {"Mapping": mapping},
            "definitions": {
                "First": {"type": "object", "properties": {"Mapping": mapping}},
                "Second": {"type": "object", "properties": {"Mapping": mapping}}
            }
        }))
        .unwrap();
        let context = EmissionContext::from_source(&schema)
            .unwrap()
            .for_enum_discovery()
            .unwrap();
        for definition in ["First", "Second"] {
            context.begin_definition_enum_discovery(definition);
            let mapping = &schema.definitions.as_ref().unwrap()[definition]
                .properties
                .as_ref()
                .unwrap()["Mapping"];
            let root = ["Mapping".to_string()];
            context.property_at(&root, mapping);
            let value_path = context.collection_path(&root, PathSegment::MapValue);
            let value = &mapping.pattern_properties.as_ref().unwrap()["x/y"];
            context.property_at(&value_path, value);
            let values_path = super::super::extend_property_path(&value_path, "Values");
            let values = &value.properties.as_ref().unwrap()["Values"];
            context.property_at(&values_path, values);
            let item_path = context.collection_path(&values_path, PathSegment::ListElement);
            let item = values.items.as_ref().unwrap();
            context.property_at(&item_path, item);
            assert_eq!(
                context.pattern_source_path(
                    &item_path,
                    item.pattern.as_deref(),
                    CfnPropertyPath::new(Some("Mapping"), "Values"),
                ),
                format!(
                    "/definitions/{definition}/properties/Mapping/patternProperties/x~1y/properties/Values/items"
                ),
            );
        }
    }

    #[test]
    fn mutated_json_fixtures_preserve_property_transform_kms_nodes() {
        let mut schema = ParsedCfnSchema::from_json(json!({
            "typeName": "AWS::SNS::Topic",
            "properties": {"KmsMasterKeyId": {"type": "string"}},
            "propertyTransform": {"/properties/KmsMasterKeyId": "original transform"}
        }))
        .unwrap();
        schema.edit_json(|source| {
            source["propertyTransform"]["/properties/KmsMasterKeyId"] =
                json!("mutated KMS transform");
            source["properties"]["DisplayName"] = json!({"type": "string"});
        });
        assert!(schema.properties.contains_key("DisplayName"));
        assert_eq!(
            schema.source()["propertyTransform"]["/properties/KmsMasterKeyId"],
            "mutated KMS transform"
        );

        let context = EmissionContext::from_source(&schema).unwrap();
        let path = ["KmsMasterKeyId".into()];
        context
            .property_at(&path, &schema.properties["KmsMasterKeyId"])
            .classify()
            .unwrap();
        let error = context.finish().unwrap_err().to_string();
        assert!(
            error.contains(
                "unaccounted KMS node sns.Topic#/propertyTransform/~1properties~1KmsMasterKeyId"
            ),
            "{error}"
        );
    }

    #[test]
    fn a_leaf_decision_does_not_cover_unemitted_descendants() {
        let source = json!({"typeName": "AWS::SNS::Topic", "properties": {
            "KmsMasterKeyId": {"type": "string", "anyOf": [{"properties": {
                "KmsHidden": {"type": "string"}
            }}]}
        }});
        let schema = ParsedCfnSchema::from_json(source).unwrap();
        let context = EmissionContext::from_source(&schema).unwrap();
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
    fn a_shared_definition_requires_every_emitted_use_site_to_be_classified() {
        let source = json!({"typeName": "AWS::SNS::Topic", "properties": {
            "KmsMasterKeyId": {"$ref": "#/definitions/KmsKeyString"},
            "EncryptionKey": {"$ref": "#/definitions/KmsKeyString"}
        }, "definitions": {"KmsKeyString": {"type": "string"}}});
        let schema = ParsedCfnSchema::from_json(source).unwrap();
        let context = EmissionContext::from_source(&schema).unwrap();
        let path = ["KmsMasterKeyId".into()];
        context
            .property_at(&path, &schema.properties["KmsMasterKeyId"])
            .classify()
            .unwrap();
        let error = context.finish().unwrap_err().to_string();
        assert!(error.contains("/properties/EncryptionKey"), "{error}");

        // Source selection alone cannot borrow another use site's decision.
        let path = ["EncryptionKey".into()];
        context.property_at(&path, &schema.properties["EncryptionKey"]);
        let error = context.finish().unwrap_err().to_string();
        assert!(
            error.contains("unaccounted KMS node sns.Topic#/definitions/KmsKeyString")
                && error.contains("sns.Topic.encryption_key"),
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
        let schema = ParsedCfnSchema::from_json(source).unwrap();
        let context = EmissionContext::from_source(&schema).unwrap();
        let path = ["Settings".into()];
        context.property_at(&path, &schema.properties["Settings"]);
        assert_eq!(context.sites.borrow()[&path[..]].sources.len(), 2);
        let error = context.finish().unwrap_err().to_string();
        assert!(
            error.contains("unaccounted KMS node ec2.Vpc#/definitions/Cycle;"),
            "{error}"
        );
    }
}
