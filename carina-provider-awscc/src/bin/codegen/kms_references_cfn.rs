//! Classify the same inferred leaves that the renderers consume.

use super::cfn::{
    CfnProperty, CfnSchema, PropertyShape, dsl_resource_name_from_type, emits_string_leaf,
    extend_property_path, inline_property_type, is_dsl_tags_property, is_property_read_only,
    is_tag_map_ref, property_shape, ref_def_name,
};
use super::kms_references::{self, Classification, PathSegment, ReferenceKind, property_path};
use anyhow::{Result, bail};
use heck::ToSnakeCase;
use std::collections::{BTreeMap, HashSet};

/// Only this module can construct a string emission input. The property and
/// decision belong to the same classified resource and complete use-site path.
pub(super) struct ClassifiedStringLeaf<'a> {
    property: &'a CfnProperty,
    reference: Option<ReferenceKind>,
    cfn_type: &'a str,
    name: &'a str,
    path: &'a [String],
}

impl<'a> ClassifiedStringLeaf<'a> {
    pub(super) fn property(&self) -> &'a CfnProperty {
        self.property
    }

    pub(super) fn reference(&self) -> Option<ReferenceKind> {
        self.reference
    }

    pub(super) fn cfn_type(&self) -> &'a str {
        self.cfn_type
    }

    pub(super) fn name(&self) -> &'a str {
        self.name
    }

    pub(super) fn path(&self) -> &'a [String] {
        self.path
    }
}

/// An emission entry point cannot combine a raw property with another
/// schema: this input carries the canonical walked property, path and owner.
pub(super) struct ClassifiedProperty<'a> {
    schema: &'a ClassifiedSchema<'a>,
    property: &'a CfnProperty,
    path: &'a [String],
    name: &'a str,
}

impl<'a> ClassifiedProperty<'a> {
    pub(super) fn schema(&self) -> &'a ClassifiedSchema<'a> {
        self.schema
    }
    pub(super) fn property(&self) -> &'a CfnProperty {
        self.property
    }
    pub(super) fn path(&self) -> &'a [String] {
        self.path
    }
    pub(super) fn name(&self) -> &'a str {
        self.name
    }
}

struct PropertySite {
    name: String,
    sources: Vec<CfnProperty>,
}

struct StringSite {
    name: String,
    property: CfnProperty,
    reference: Option<ReferenceKind>,
}

/// Renderers can obtain a leaf only at a path this schema actually walked.
/// Recording ordinary and provider-populated strings distinguishes them from
/// unknown paths; a missing reference decision is never used as a fallback.
pub(super) struct ClassifiedSchema<'a> {
    schema: &'a CfnSchema,
    strings: BTreeMap<Vec<String>, StringSite>,
    properties: BTreeMap<Vec<String>, PropertySite>,
}

impl<'a> ClassifiedSchema<'a> {
    pub(super) fn new(
        schema: &'a CfnSchema,
        overridden_properties: impl IntoIterator<Item = &'a CfnProperty>,
    ) -> Result<Self> {
        let mut walker = Walker {
            schema,
            overridden_properties: overridden_properties.into_iter().collect(),
            resource: dsl_resource_name_from_type(&schema.type_name)?,
            ancestors: HashSet::new(),
            strings: BTreeMap::new(),
            properties: BTreeMap::new(),
        };
        for (name, property) in &schema.properties {
            let path = std::slice::from_ref(name);
            walker.walk(
                property,
                name,
                &name.to_snake_case(),
                path,
                is_property_read_only(schema, property, path),
            )?;
        }
        Ok(Self {
            schema,
            strings: walker.strings,
            properties: walker.properties,
        })
    }

    pub(super) fn property_at(
        &self,
        path: &[String],
        property: &CfnProperty,
    ) -> Result<ClassifiedProperty<'_>> {
        let (path, site) = self.properties.get_key_value(path).ok_or_else(|| {
            anyhow::anyhow!(
                "{}: unclassified property path {}",
                self.schema.type_name,
                path.join("/")
            )
        })?;
        let property = site
            .sources
            .iter()
            .find(|source| *source == property)
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "{}: foreign property at classified path {}",
                    self.schema.type_name,
                    path.join("/")
                )
            })?;
        Ok(ClassifiedProperty {
            schema: self,
            property,
            path,
            name: &site.name,
        })
    }

    pub(super) fn string_leaf_at(&self, path: &[String]) -> Result<ClassifiedStringLeaf<'_>> {
        let (path, site) = self.strings.get_key_value(path).ok_or_else(|| {
            anyhow::anyhow!(
                "{}: unclassified string leaf path {}",
                self.schema.type_name,
                path.join("/")
            )
        })?;
        Ok(ClassifiedStringLeaf {
            property: &site.property,
            reference: site.reference,
            cfn_type: &self.schema.type_name,
            name: &site.name,
            path,
        })
    }
}

impl std::ops::Deref for ClassifiedSchema<'_> {
    type Target = CfnSchema;

    fn deref(&self) -> &Self::Target {
        self.schema
    }
}

#[derive(Default)]
struct Evidence<'a> {
    patterns: Vec<&'a str>,
    descriptions: Vec<&'a str>,
    has_enum_or_const: bool,
    /// All schemas at this exact path, for auditing replacement subtrees too.
    sources: Vec<&'a CfnProperty>,
}

impl<'a> Evidence<'a> {
    /// Follow all same-site schemas, independently of shape precedence. A
    /// constraint-only branch has evidence even though it owns no Rust type.
    /// Collection evidence follows its scalar values; struct fields start
    /// their own evidence, just as they start their own input-mode decision.
    fn collect(
        &mut self,
        property: &'a CfnProperty,
        path: &[String],
        schema: &'a CfnSchema,
        visiting: &mut HashSet<(&'a str, usize)>,
    ) {
        if path.is_empty() && !self.sources.contains(&property) {
            self.sources.push(property);
        }
        if path.iter().all(|segment| segment == "*") {
            self.has_enum_or_const |=
                property.enum_values.is_some() || property.const_value.is_some();
            if let Some(pattern) = property.pattern.as_deref()
                && !self.patterns.contains(&pattern)
            {
                self.patterns.push(pattern);
            }
            if let Some(description) = property.description.as_deref()
                && !self.descriptions.contains(&description)
            {
                self.descriptions.push(description);
            }
        }
        if let Some(name) = property.ref_path.as_deref().and_then(ref_def_name)
            && let Some((name, target)) = schema
                .definitions
                .as_ref()
                .and_then(|defs| defs.get_key_value(name))
            && visiting.insert((name, path.len()))
        {
            self.collect(target, path, schema, visiting);
            visiting.remove(&(name.as_str(), path.len()));
        }
        for variant in property
            .one_of
            .iter()
            .chain(&property.any_of)
            .chain(&property.all_of)
        {
            self.collect(variant, path, schema, visiting);
        }
        if let Some((segment, remaining)) = path.split_first() {
            if segment == "*" {
                if let Some(items) = property.items.as_deref() {
                    self.collect(items, remaining, schema, visiting);
                }
                for value in property
                    .pattern_properties
                    .iter()
                    .flat_map(|patterns| patterns.values())
                {
                    self.collect(value, remaining, schema, visiting);
                }
            } else if let Some(child) = property
                .properties
                .as_ref()
                .and_then(|fields| fields.get(segment))
            {
                self.collect(child, remaining, schema, visiting);
            }
        }
    }

    fn at(schema: &'a CfnSchema, path: &[String]) -> Self {
        let mut evidence = Self::default();
        if let Some((root, remaining)) = path.split_first()
            && let Some(property) = schema.properties.get(root)
        {
            evidence.collect(property, remaining, schema, &mut HashSet::new());
        }
        for branch in schema
            .one_of
            .iter()
            .chain(&schema.any_of)
            .chain(&schema.all_of)
        {
            evidence.collect(branch, path, schema, &mut HashSet::new());
        }
        evidence
    }
}

struct Walker<'a> {
    schema: &'a CfnSchema,
    overridden_properties: Vec<&'a CfnProperty>,
    resource: String,
    ancestors: HashSet<String>,
    strings: BTreeMap<Vec<String>, StringSite>,
    properties: BTreeMap<Vec<String>, PropertySite>,
}

impl Walker<'_> {
    fn record(&mut self, property: &CfnProperty, name: &str, path: &[String]) {
        let site = self
            .properties
            .entry(path.to_vec())
            .or_insert_with(|| PropertySite {
                name: name.to_owned(),
                sources: Vec::new(),
            });
        if !site.sources.contains(property) {
            site.sources.push(property.clone());
        }
    }

    fn decision(
        &self,
        name: &str,
        path: &str,
        provider_path: &[String],
        provider_populated: bool,
    ) -> Result<Option<ReferenceKind>> {
        let evidence = Evidence::at(self.schema, provider_path);
        let classification = if provider_populated {
            None
        } else {
            kms_references::classify_string(
                &self.resource,
                path,
                name,
                Some(&evidence.patterns.join(" ")),
                Some(&evidence.descriptions.join(" ")),
            )?
        };
        Ok(match classification {
            Some(Classification::Reference(kind)) => {
                if evidence.has_enum_or_const {
                    bail!(
                        "{}.{path}: KMS enum/const constraint is incompatible with the reference constructor; revisit the classification table entry",
                        self.resource
                    );
                }
                let entry = kms_references::entry(&self.resource, path)
                    .expect("classified reference has a table entry");
                if evidence.patterns.as_slice() != entry.cfn_pattern.as_slice() {
                    bail!(
                        "{}.{path}: KMS pattern changed from {:?} to {:?}; classify its constraint before generation",
                        self.resource,
                        entry.cfn_pattern,
                        evidence.patterns
                    );
                }
                Some(kind)
            }
            Some(Classification::NotReference(_)) | None => None,
        })
    }

    fn string(
        &mut self,
        property: &CfnProperty,
        name: &str,
        path: &str,
        provider_path: &[String],
        provider_populated: bool,
    ) -> Result<()> {
        let reference = self.decision(name, path, provider_path, provider_populated)?;
        self.strings.insert(
            provider_path.to_vec(),
            StringSite {
                name: name.to_owned(),
                property: property.clone(),
                reference,
            },
        );
        Ok(())
    }

    /// An override or tag projection owns its emitted type, but not its CFN
    /// evidence. Audit every source branch, including opaque container nodes,
    /// without adding synthetic emission inputs or changing shape precedence.
    fn audit(
        &self,
        property: &CfnProperty,
        name: &str,
        path: &str,
        provider_path: &[String],
        provider_populated: bool,
        visiting: &mut HashSet<String>,
    ) -> Result<()> {
        if !matches!(
            inline_property_type(property),
            Some("boolean" | "integer" | "number")
        ) {
            self.decision(name, path, provider_path, provider_populated)?;
        }
        if let Some(definition) = property.ref_path.as_deref().and_then(ref_def_name)
            && let Some(target) = self
                .schema
                .definitions
                .as_ref()
                .and_then(|defs| defs.get(definition))
            && visiting.insert(definition.to_string())
        {
            self.audit(
                target,
                name,
                path,
                provider_path,
                provider_populated,
                visiting,
            )?;
            visiting.remove(definition);
        }
        for branch in property
            .one_of
            .iter()
            .chain(&property.any_of)
            .chain(&property.all_of)
        {
            self.audit(
                branch,
                name,
                path,
                provider_path,
                provider_populated,
                visiting,
            )?;
        }
        for (name, child) in property
            .properties
            .iter()
            .flat_map(|properties| properties.iter())
        {
            let child_path = extend_property_path(provider_path, name);
            self.audit(
                child,
                name,
                &property_path(path, PathSegment::Field(&name.to_snake_case())),
                &child_path,
                // These fields are hidden by the replacement type, so they
                // inherit its input mode rather than becoming emitted fields.
                provider_populated || is_property_read_only(self.schema, child, &child_path),
                visiting,
            )?;
        }
        if let Some(items) = property.items.as_deref() {
            self.audit(
                items,
                name,
                &property_path(path, PathSegment::ListElement),
                &extend_property_path(provider_path, "*"),
                provider_populated,
                visiting,
            )?;
        }
        for value in property
            .pattern_properties
            .iter()
            .flat_map(|patterns| patterns.values())
        {
            self.audit(
                value,
                name,
                &property_path(path, PathSegment::MapValue),
                &extend_property_path(provider_path, "*"),
                provider_populated,
                visiting,
            )?;
        }
        Ok(())
    }

    fn walk(
        &mut self,
        property: &CfnProperty,
        name: &str,
        path: &str,
        provider_path: &[String],
        provider_populated: bool,
    ) -> Result<()> {
        self.record(property, name, provider_path);
        let projected = is_dsl_tags_property(self.schema, name)
            || property
                .ref_path
                .as_deref()
                .is_some_and(|reference| is_tag_map_ref(self.schema, reference));
        if projected || self.overridden_properties.contains(&property) {
            for source in Evidence::at(self.schema, provider_path).sources {
                self.audit(
                    source,
                    name,
                    path,
                    provider_path,
                    provider_populated,
                    &mut HashSet::new(),
                )?;
            }
        }
        if projected {
            return self.string(
                &CfnProperty::default(),
                name,
                &property_path(path, PathSegment::MapValue),
                &extend_property_path(provider_path, "*"),
                provider_populated,
            );
        }
        // Const strings precede shape resolution in both renderers.
        if property.const_value.is_some() {
            return self.string(property, name, path, provider_path, provider_populated);
        }
        match property_shape(property, self.schema) {
            PropertyShape::Reference {
                path: reference,
                resolved,
            } => {
                if let Some(resolved) = resolved {
                    if self.ancestors.insert(resolved.name.to_string()) {
                        self.walk(
                            resolved.property,
                            name,
                            path,
                            provider_path,
                            provider_populated,
                        )?;
                        self.ancestors.remove(resolved.name);
                    }
                    return Ok(());
                }
                let evidence = Evidence::at(self.schema, provider_path);
                if !provider_populated
                    && kms_references::is_candidate(
                        name,
                        Some(&evidence.patterns.join(" ")),
                        Some(&evidence.descriptions.join(" ")),
                    )
                {
                    bail!(
                        "{}.{path}: cannot classify KMS input with unresolved $ref {reference}",
                        self.resource
                    );
                }
                return self.string(property, name, path, provider_path, provider_populated);
            }
            PropertyShape::Array { variant, .. } => {
                return self.walk(variant, name, path, provider_path, provider_populated);
            }
            PropertyShape::Direct(properties) => {
                return self.fields(properties, path, provider_path);
            }
            PropertyShape::Merged(properties) => {
                let mut inline = property.clone();
                inline.prop_type = Some(super::cfn::TypeValue::Single("object".into()));
                inline.properties = Some(properties.clone());
                inline.required.clear();
                inline.one_of.clear();
                self.record(&inline, name, provider_path);
                return self.fields(&properties, path, provider_path);
            }
            PropertyShape::Inline | PropertyShape::Unresolved => {}
        }
        if emits_string_leaf(property, name) {
            return self.string(property, name, path, provider_path, provider_populated);
        }
        if inline_property_type(property) == Some("array") {
            self.walk(
                property.items.as_deref().unwrap_or(&CfnProperty::default()),
                name,
                &property_path(path, PathSegment::ListElement),
                &extend_property_path(provider_path, "*"),
                provider_populated,
            )?;
        }
        if let Some(patterns) = &property.pattern_properties {
            for value in patterns.values() {
                self.walk(
                    value,
                    name,
                    &property_path(path, PathSegment::MapValue),
                    &extend_property_path(provider_path, "*"),
                    provider_populated,
                )?;
            }
        }
        Ok(())
    }

    fn fields(
        &mut self,
        properties: &BTreeMap<String, CfnProperty>,
        path: &str,
        provider_path: &[String],
    ) -> Result<()> {
        for (name, property) in properties {
            let field_path = extend_property_path(provider_path, name);
            self.walk(
                property,
                name,
                &property_path(path, PathSegment::Field(&name.to_snake_case())),
                &field_path,
                is_property_read_only(self.schema, property, &field_path),
            )?;
        }
        Ok(())
    }
}
