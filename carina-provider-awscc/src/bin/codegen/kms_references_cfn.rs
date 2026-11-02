//! Follow every input use before omissions or type overrides can hide candidates.

use super::kms_references::{self, Classification, PathSegment, ReferenceKind, property_path};
use super::{
    CfnProperty, CfnSchema, dsl_resource_name_from_type, extend_property_path,
    is_read_only_property_path, ref_def_name,
};
use anyhow::{Result, bail};
use heck::ToSnakeCase;
use std::collections::{BTreeMap, HashSet};

/// Renderers require a schema whose input strings have all been classified.
/// Raw CFN pointers map to the full DSL paths produced by `property_path`;
/// definition names and inline parent names never identify reference sites.
pub(super) struct ClassifiedSchema<'a> {
    schema: &'a CfnSchema,
    sites: BTreeMap<String, ReferenceKind>,
    property_paths: BTreeMap<Vec<String>, String>,
}

impl<'a> ClassifiedSchema<'a> {
    pub(super) fn new(schema: &'a CfnSchema) -> Result<Self> {
        let (sites, property_paths) = reference_sites(schema)?;
        Ok(Self {
            schema,
            sites,
            property_paths,
        })
    }

    pub(super) fn reference_at_property(&self, property_path: &[String]) -> Option<ReferenceKind> {
        self.property_paths
            .get(property_path)
            .and_then(|path| self.sites.get(path))
            .copied()
    }
}

impl std::ops::Deref for ClassifiedSchema<'_> {
    type Target = CfnSchema;

    fn deref(&self) -> &Self::Target {
        self.schema
    }
}

type ReferenceSites = (
    BTreeMap<String, ReferenceKind>,
    BTreeMap<Vec<String>, String>,
);

#[derive(Clone, Default)]
struct Evidence<'a> {
    patterns: Vec<&'a str>,
    descriptions: Vec<&'a str>,
}

impl<'a> Evidence<'a> {
    fn with(mut self, property: &'a CfnProperty) -> Self {
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
        self
    }

    fn is_candidate(&self, name: &str) -> bool {
        kms_references::is_candidate(
            name,
            Some(&self.patterns.join(" ")),
            Some(&self.descriptions.join(" ")),
        )
    }
}

/// Classify each use at its resource-relative path before either renderer runs.
fn reference_sites(schema: &CfnSchema) -> Result<ReferenceSites> {
    struct Walker<'a> {
        schema: &'a CfnSchema,
        resource: String,
        ancestors: HashSet<&'a str>,
        sites: BTreeMap<String, Option<ReferenceKind>>,
        property_paths: BTreeMap<Vec<String>, String>,
    }

    impl<'a> Walker<'a> {
        fn walk(
            &mut self,
            prop: &'a CfnProperty,
            name: &str,
            path: &str,
            provider_path: &[String],
            evidence: Evidence<'a>,
        ) -> Result<()> {
            let evidence = evidence.with(prop);
            if let Some(reference) = prop.ref_path.as_deref() {
                let definition = ref_def_name(reference)
                    .and_then(|name| self.schema.definitions.as_ref()?.get_key_value(name));
                let Some((definition_name, target)) = definition else {
                    if !is_read_only_property_path(self.schema, provider_path)
                        && evidence.is_candidate(name)
                    {
                        bail!(
                            "{}.{path}: cannot classify KMS input with unresolved $ref {reference}",
                            self.resource
                        );
                    }
                    return Ok(());
                };
                if self.ancestors.insert(definition_name) {
                    self.walk(target, name, path, provider_path, evidence)?;
                    self.ancestors.remove(definition_name.as_str());
                }
                return Ok(());
            }
            if prop
                .prop_type
                .as_ref()
                .is_some_and(|ty| ty.contains("string"))
            {
                let classification = if is_read_only_property_path(self.schema, provider_path) {
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
                if let Some(classification) = classification {
                    self.property_paths
                        .insert(provider_path.to_vec(), path.to_string());
                    let kind = match classification {
                        Classification::Reference(kind) => {
                            let entry =
                                kms_references::entry(&self.resource, path).ok_or_else(|| {
                                    anyhow::anyhow!(
                                        "missing KMS classification for {}.{path}",
                                        self.resource
                                    )
                                })?;
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
                        Classification::NotReference(_) => None,
                    };
                    let site = path.to_string();
                    if let Some(previous) = self.sites.insert(site.clone(), kind)
                        && previous != kind
                    {
                        bail!(
                            "{}.{path}: shared KMS site {site} has conflicting reference constructors",
                            self.resource
                        );
                    }
                }
            }
            if let Some(properties) = &prop.properties {
                for (child_name, child) in properties {
                    self.walk(
                        child,
                        child_name,
                        &property_path(path, PathSegment::Field(&child_name.to_snake_case())),
                        &extend_property_path(provider_path, child_name),
                        Evidence::default(),
                    )?;
                }
            }
            if let Some(items) = prop.items.as_deref() {
                self.walk(
                    items,
                    name,
                    &property_path(path, PathSegment::ListElement),
                    &extend_property_path(provider_path, "*"),
                    evidence.clone(),
                )?;
            }
            if let Some(patterns) = &prop.pattern_properties {
                for value in patterns.values() {
                    self.walk(
                        value,
                        name,
                        &property_path(path, PathSegment::MapValue),
                        &extend_property_path(provider_path, "*"),
                        evidence.clone(),
                    )?;
                }
            }
            for variant in prop.one_of.iter().chain(&prop.any_of) {
                self.walk(variant, name, path, provider_path, evidence.clone())?;
            }
            Ok(())
        }
    }

    let mut walker = Walker {
        schema,
        resource: dsl_resource_name_from_type(&schema.type_name)?,
        ancestors: HashSet::new(),
        sites: BTreeMap::new(),
        property_paths: BTreeMap::new(),
    };
    for (name, prop) in &schema.properties {
        walker.walk(
            prop,
            name,
            &name.to_snake_case(),
            std::slice::from_ref(name),
            Evidence::default(),
        )?;
    }
    Ok((
        walker
            .sites
            .into_iter()
            .filter_map(|(site, kind)| kind.map(|kind| (site, kind)))
            .collect(),
        walker.property_paths,
    ))
}
