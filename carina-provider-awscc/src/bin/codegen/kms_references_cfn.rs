//! KMS decisions recorded by the emitters; no schema-site enumeration.

use super::kms_references::{self, Classification, PathSegment, ReferenceKind, property_path};
use super::{
    CfnProperty, CfnSchema, dsl_resource_name_from_type, is_property_read_only,
    is_read_only_property_path, ref_def_name,
};
use anyhow::{Result, bail};
use heck::ToSnakeCase;
use serde_json::Value;
use std::cell::RefCell;
use std::collections::{BTreeMap, HashSet};

pub(super) struct EmissionContext<'a> {
    schema: &'a CfnSchema,
    resource: String,
    visited: RefCell<HashSet<String>>,
    /// Collection suffixes come from actual emitter decisions, never shape lookahead.
    paths: RefCell<BTreeMap<Vec<String>, String>>,
    sites: RefCell<BTreeMap<Vec<String>, EmittedSite>>,
    enum_discovery: bool,
}

/// A use-site cursor, constructed as the emitter visits a property. It carries
/// raw CFN evidence, not a precomputed classification or an admission token.
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
        if self.schema.enum_discovery || self.provider_populated() {
            return Ok(None);
        }
        let site = self.schema.sites.borrow()[self.path].clone();
        self.schema.classify(
            &self.schema.dsl_path(self.path),
            self.name(),
            &site.evidence,
        )
    }
    fn provider_populated(&self) -> bool {
        self.schema.sites.borrow()[self.path].provider_populated
    }
    pub(super) fn audit_skipped_subtree(&self) -> Result<()> {
        if self.schema.enum_discovery || self.provider_populated() {
            return Ok(());
        }
        let site = self.schema.sites.borrow()[self.path].clone();
        for source in site.sources {
            self.schema.audit_json(
                &self.schema.dsl_path(self.path),
                self.path,
                self.name(),
                &serde_json::to_value(source)?,
                &mut HashSet::new(),
            )?;
        }
        Ok(())
    }

    /// The emitter supplies the keywords discarded by its actual decision.
    pub(super) fn audit_skipped_keywords(
        &self,
        keywords: &[&str],
        selected_variant: Option<&CfnProperty>,
    ) -> Result<()> {
        if self.schema.enum_discovery || self.provider_populated() {
            return Ok(());
        }
        let source = serde_json::to_value(self.property)?;
        let mut skipped = serde_json::Map::from_iter(self.property.extra.clone());
        for &keyword in keywords {
            if let Some(value) = source.get(keyword).filter(|value| !value.is_null()) {
                if keyword == "oneOf" {
                    let branches: Vec<_> = self
                        .property
                        .one_of
                        .iter()
                        .filter(|branch| {
                            selected_variant.is_none_or(|selected| !std::ptr::eq(*branch, selected))
                        })
                        .map(serde_json::to_value)
                        .collect::<Result<_, _>>()?;
                    if !branches.is_empty() {
                        skipped.insert(keyword.into(), Value::Array(branches));
                    }
                } else if !value.as_array().is_some_and(Vec::is_empty) {
                    skipped.insert(keyword.into(), value.clone());
                }
            }
        }
        if !skipped.is_empty() {
            self.schema.audit_json(
                &self.schema.dsl_path(self.path),
                self.path,
                self.name(),
                &Value::Object(skipped),
                &mut HashSet::new(),
            )?;
        }
        Ok(())
    }
}

impl<'a> EmissionContext<'a> {
    pub(super) fn new(schema: &'a CfnSchema) -> Result<Self> {
        Ok(Self {
            schema,
            resource: dsl_resource_name_from_type(&schema.type_name)?,
            visited: RefCell::new(HashSet::new()),
            paths: RefCell::new(BTreeMap::new()),
            sites: RefCell::new(BTreeMap::new()),
            enum_discovery: false,
        })
    }
    /// Enum metadata is discovered in definition-local namespaces on main.
    /// Those are not resource emission sites and cannot visit the KMS table.
    pub(super) fn for_enum_discovery(schema: &'a CfnSchema) -> Result<Self> {
        Ok(Self {
            enum_discovery: true,
            ..Self::new(schema)?
        })
    }
    pub(super) fn property_at<'b>(
        &'b self,
        path: &'b [String],
        property: &'b CfnProperty,
    ) -> EmissionProperty<'b> {
        if self.enum_discovery {
            return EmissionProperty {
                schema: self,
                property,
                path,
            };
        }
        let mut site = self.sites.borrow().get(path).cloned().unwrap_or_else(|| {
            let mut site = EmittedSite::default();
            if let Some((name, parent_path)) = path.split_last() {
                if let Some(parent) = self.sites.borrow().get(parent_path) {
                    if name == "*" {
                        site.evidence = parent.evidence.clone();
                        site.provider_populated = parent.provider_populated;
                        for source in &parent.sources {
                            if self.dsl_path(path).ends_with("{}") {
                                for value in source
                                    .pattern_properties
                                    .iter()
                                    .flat_map(|values| values.values())
                                {
                                    site.add_source(self, value);
                                }
                            } else if let Some(items) = source.items.as_deref() {
                                site.add_source(self, items);
                            }
                        }
                    } else {
                        for source in &parent.sources {
                            if let Some(field) = source
                                .properties
                                .as_ref()
                                .and_then(|fields| fields.get(name))
                            {
                                site.add_source(self, field);
                            }
                        }
                    }
                } else if parent_path.is_empty() {
                    // The emitter is visiting this top-level field. Select
                    // only that field's constraints from resource branches.
                    let mut root = EmittedSite::default();
                    for branch in self.one_of.iter().chain(&self.any_of).chain(&self.all_of) {
                        root.add_source(self, branch);
                    }
                    for source in &root.sources {
                        if let Some(field) = source
                            .properties
                            .as_ref()
                            .and_then(|fields| fields.get(name))
                        {
                            site.add_source(self, field);
                        }
                    }
                }
                if name != "*" {
                    site.provider_populated = is_property_read_only(self, property, path);
                }
            }
            site
        });
        site.add_source(self, property);
        self.sites.borrow_mut().insert(path.to_vec(), site);
        EmissionProperty {
            schema: self,
            property,
            path,
        }
    }
    pub(super) fn collection_path(
        &self,
        parent: &[String],
        segment: PathSegment<'_>,
    ) -> Vec<String> {
        let path = super::extend_property_path(parent, "*");
        if self.enum_discovery {
            return path;
        }
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
                // Markdown may document inline tag pairs hidden by Rust's map projection.
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
        evidence: &Evidence,
    ) -> Result<Option<ReferenceKind>> {
        let decision = kms_references::classify_string(
            &self.resource,
            path,
            name,
            Some(&evidence.patterns.join(" ")),
            Some(&evidence.descriptions.join(" ")),
        )?;
        if decision.is_some() {
            self.visited.borrow_mut().insert(path.to_string());
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
    /// The tag projection emits a map value even when CFN uses an array of pairs.
    pub(super) fn audit_tag_value(&self, path: &[String]) -> Result<()> {
        if self.enum_discovery {
            return Ok(());
        }
        let site = self.sites.borrow()[path].clone();
        if !site.provider_populated {
            self.classify(
                &property_path(&self.dsl_path(path), PathSegment::MapValue),
                "",
                &site.evidence,
            )?;
        }
        Ok(())
    }
    /// Scan raw JSON only. Keywords establish paths, not types; unknown JSON
    /// is recursively scanned at the current site, without shape validation.
    fn audit_json(
        &self,
        path: &str,
        provider_path: &[String],
        name: &str,
        value: &Value,
        visiting: &mut HashSet<String>,
    ) -> Result<()> {
        if is_read_only_property_path(self, provider_path) {
            return Ok(());
        }
        if kms_references::entry(&self.resource, path).is_some() {
            self.visited.borrow_mut().insert(path.to_string());
        }
        let require = |evidence: &str| -> Result<()> {
            kms_references::classify_string(&self.resource, path, name, None, Some(evidence))?;
            Ok(())
        };
        require("")?;
        match value {
            Value::String(text) => require(text)?,
            Value::Array(values) => {
                for value in values {
                    self.audit_json(path, provider_path, name, value, visiting)?;
                }
            }
            Value::Object(object) => {
                for (key, value) in object {
                    match key.as_str() {
                        "$ref" => {
                            if let Some(definition) = value.as_str().and_then(ref_def_name)
                                && let Some(target) = self
                                    .definitions
                                    .as_ref()
                                    .and_then(|defs| defs.get(definition))
                                && visiting.insert(definition.to_string())
                            {
                                self.audit_json(
                                    path,
                                    provider_path,
                                    name,
                                    &serde_json::to_value(target)?,
                                    visiting,
                                )?;
                                visiting.remove(definition);
                            }
                        }
                        "properties" => {
                            if let Some(fields) = value.as_object() {
                                for (field, value) in fields {
                                    self.audit_json(
                                        &property_path(
                                            path,
                                            PathSegment::Field(&field.to_snake_case()),
                                        ),
                                        &super::extend_property_path(provider_path, field),
                                        field,
                                        value,
                                        visiting,
                                    )?;
                                }
                            } else {
                                self.audit_json(path, provider_path, name, value, visiting)?;
                            }
                        }
                        "items" => self.audit_json(
                            &property_path(path, PathSegment::ListElement),
                            &super::extend_property_path(provider_path, "*"),
                            name,
                            value,
                            visiting,
                        )?,
                        "patternProperties" => {
                            if let Some(patterns) = value.as_object() {
                                for value in patterns.values() {
                                    self.audit_json(
                                        &property_path(path, PathSegment::MapValue),
                                        &super::extend_property_path(provider_path, "*"),
                                        name,
                                        value,
                                        visiting,
                                    )?;
                                }
                            } else {
                                self.audit_json(path, provider_path, name, value, visiting)?;
                            }
                        }
                        _ => self.audit_json(path, provider_path, name, value, visiting)?,
                    }
                }
            }
            _ => {}
        }
        Ok(())
    }
    pub(super) fn finish(&self) -> Result<()> {
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

#[derive(Clone, Default)]
struct EmittedSite {
    sources: Vec<CfnProperty>,
    evidence: Evidence,
    provider_populated: bool,
}

impl EmittedSite {
    /// Same-site evidence only: never enumerate fields or collection values.
    /// Child sources are selected above when the emitter actually visits one.
    fn add_source(&mut self, schema: &CfnSchema, property: &CfnProperty) {
        if self.sources.contains(property) {
            return;
        }
        self.sources.push(property.clone());
        let evidence = &mut self.evidence;
        evidence.has_enum_or_const |=
            property.enum_values.is_some() || property.const_value.is_some();
        if let Some(pattern) = &property.pattern
            && !evidence.patterns.contains(pattern)
        {
            evidence.patterns.push(pattern.clone());
        }
        let length = (property.min_length, property.max_length);
        if length != (None, None) && !evidence.lengths.contains(&length) {
            evidence.lengths.push(length);
        }
        if let Some(description) = &property.description
            && !evidence.descriptions.contains(description)
        {
            evidence.descriptions.push(description.clone());
        }
        if let Some(name) = property.ref_path.as_deref().and_then(ref_def_name)
            && let Some(target) = schema.definitions.as_ref().and_then(|defs| defs.get(name))
        {
            self.add_source(schema, target);
        }
        for branch in property
            .one_of
            .iter()
            .chain(&property.any_of)
            .chain(&property.all_of)
        {
            self.add_source(schema, branch);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn setup_does_not_walk_or_classify() {
        let schema: CfnSchema = serde_json::from_value(json!({
            "typeName": "AWS::EC2::VPC", "properties": {"KmsFoo": {"enum": ["value"]}}
        }))
        .unwrap();
        let context = EmissionContext::new(&schema).unwrap();
        assert!(context.sites.borrow().is_empty());
        let path = ["KmsFoo".into()];
        let property = context.property_at(&path, &schema.properties["KmsFoo"]);
        assert!(context.visited.borrow().is_empty());
        let error = property.classify().unwrap_err();
        assert!(
            error
                .to_string()
                .contains("unclassified KMS string ec2.Vpc.kms_foo")
        );
    }

    #[test]
    fn completeness_requires_actual_emission_or_audit() {
        let schema: CfnSchema = serde_json::from_value(json!({
            "typeName": "AWS::SNS::Topic", "properties": {"KmsMasterKeyId": {"type": "string"}}
        }))
        .unwrap();
        let context = EmissionContext::new(&schema).unwrap();
        let path = ["KmsMasterKeyId".into()];
        let property = context.property_at(&path, &schema.properties["KmsMasterKeyId"]);
        assert!(context.finish().is_err());
        assert!(property.classify().unwrap().is_some());
        context.finish().unwrap();
    }

    #[test]
    fn audit_accepts_an_explicit_non_reference_for_unknown_json() {
        let schema: CfnSchema = serde_json::from_value(json!({
            "typeName": "AWS::KMS::Key", "properties": {"KeyPolicy": {
                "type": "object", "x-future": [17, {"text": "The AWS KMS key"}]
            }}
        }))
        .unwrap();
        let context = EmissionContext::new(&schema).unwrap();
        let path = ["KeyPolicy".into()];
        context
            .property_at(&path, &schema.properties["KeyPolicy"])
            .audit_skipped_subtree()
            .unwrap();
        assert!(context.visited.borrow().contains("key_policy"));
    }

    #[test]
    fn audit_follows_cycles_without_inventing_emission_shapes() {
        let schema: CfnSchema = serde_json::from_value(json!({
            "typeName": "AWS::EC2::VPC", "properties": {"Settings": {"$ref": "#/definitions/Cycle"}},
            "definitions": {"Cycle": {"$ref": "#/definitions/Cycle", "patternProperties": {
                ".*": {"type": "array", "items": {"type": "string"}}
            }}}
        })).unwrap();
        let context = EmissionContext::new(&schema).unwrap();
        let path = ["Settings".into()];
        context
            .property_at(&path, &schema.properties["Settings"])
            .audit_skipped_subtree()
            .unwrap();
    }
}
