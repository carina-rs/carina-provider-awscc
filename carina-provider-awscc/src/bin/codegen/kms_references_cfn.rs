//! Follow every input use before omissions or type overrides can hide candidates.

use super::kms_references::{self, Classification, PathSegment, ReferenceKind, property_path};
use super::{
    CfnProperty, CfnPropertyPath, CfnSchema, cfn_property_path, dsl_resource_name_from_type,
    extend_property_path, ref_def_name,
};
use anyhow::{Result, bail};
use heck::ToSnakeCase;
use std::cell::RefCell;
use std::collections::{BTreeMap, HashSet};

type ReferenceSites = BTreeMap<String, ReferenceKind>;
type ActiveSites = Option<(String, ReferenceSites)>;

thread_local! {
    static ACTIVE_SITES: RefCell<ActiveSites> = const { RefCell::new(None) };
}

/// Validate once before either renderer, then restore the previous generation's
/// context on every return path. Like codegen's other recursion state, this is
/// thread-local so concurrent generation tests cannot share decisions.
pub(super) struct GenerationGuard {
    previous: ActiveSites,
}

impl GenerationGuard {
    pub(super) fn new(schema: &CfnSchema) -> Result<Self> {
        let sites = reference_sites(schema)?;
        let previous =
            ACTIVE_SITES.with(|active| active.replace(Some((schema.type_name.clone(), sites))));
        Ok(Self { previous })
    }
}

impl Drop for GenerationGuard {
    fn drop(&mut self) {
        ACTIVE_SITES.with(|active| active.replace(self.previous.take()));
    }
}

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

/// Derive definition-local emission sites from the full-path classification table.
/// Shared structs must agree at every use, because codegen reuses their definitions.
fn reference_sites(schema: &CfnSchema) -> Result<ReferenceSites> {
    struct Walker<'a> {
        schema: &'a CfnSchema,
        resource: String,
        ancestors: HashSet<&'a str>,
        sites: BTreeMap<String, Option<ReferenceKind>>,
    }

    impl<'a> Walker<'a> {
        fn walk(
            &mut self,
            prop: &'a CfnProperty,
            name: &str,
            path: &str,
            provider_path: &[String],
            parent: Option<&str>,
            evidence: Evidence<'a>,
        ) -> Result<()> {
            if self
                .schema
                .read_only_properties
                .iter()
                .filter_map(|p| cfn_property_path(p))
                .any(|read_only| provider_path.starts_with(&read_only))
            {
                return Ok(());
            }
            let evidence = evidence.with(prop);
            if let Some(reference) = prop.ref_path.as_deref() {
                let definition = ref_def_name(reference)
                    .and_then(|name| self.schema.definitions.as_ref()?.get_key_value(name));
                let Some((definition_name, target)) = definition else {
                    if evidence.is_candidate(name) {
                        bail!(
                            "{}.{path}: cannot classify KMS input with unresolved $ref {reference}",
                            self.resource
                        );
                    }
                    return Ok(());
                };
                if self.ancestors.insert(definition_name) {
                    // Keep the original property/parent for scalar aliases; object
                    // children below acquire the referenced definition's own name.
                    self.walk_resolved(
                        target,
                        name,
                        path,
                        provider_path,
                        parent,
                        evidence,
                        Some(definition_name),
                    )?;
                    self.ancestors.remove(definition_name.as_str());
                }
                return Ok(());
            }
            self.walk_resolved(prop, name, path, provider_path, parent, evidence, None)
        }

        #[allow(clippy::too_many_arguments)]
        fn walk_resolved(
            &mut self,
            prop: &'a CfnProperty,
            name: &str,
            path: &str,
            provider_path: &[String],
            parent: Option<&str>,
            evidence: Evidence<'a>,
            definition_name: Option<&str>,
        ) -> Result<()> {
            let evidence = evidence.with(prop);
            if prop.ref_path.is_some() {
                return self.walk(prop, name, path, provider_path, parent, evidence);
            }
            if prop
                .prop_type
                .as_ref()
                .is_some_and(|ty| ty.contains("string"))
            {
                let classification = kms_references::classify_string(
                    &self.resource,
                    path,
                    name,
                    Some(&evidence.patterns.join(" ")),
                    Some(&evidence.descriptions.join(" ")),
                )?;
                if let Some(classification) = classification {
                    let kind = match classification {
                        Classification::Reference(kind) => {
                            let entry = kms_references::entry(&self.resource, path).unwrap();
                            if evidence.patterns
                                != entry.cfn_pattern.into_iter().collect::<Vec<_>>()
                            {
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
                    let site = CfnPropertyPath::new(parent, name).to_string();
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
                        Some(definition_name.unwrap_or(name)),
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
                    parent,
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
                        parent,
                        evidence.clone(),
                    )?;
                }
            }
            for variant in prop.one_of.iter().chain(&prop.any_of) {
                self.walk(variant, name, path, provider_path, parent, evidence.clone())?;
            }
            Ok(())
        }
    }

    let mut walker = Walker {
        schema,
        resource: dsl_resource_name_from_type(&schema.type_name)?,
        ancestors: HashSet::new(),
        sites: BTreeMap::new(),
    };
    for (name, prop) in &schema.properties {
        walker.walk(
            prop,
            name,
            &name.to_snake_case(),
            std::slice::from_ref(name),
            None,
            Evidence::default(),
        )?;
    }
    Ok(walker
        .sites
        .into_iter()
        .filter_map(|(site, kind)| kind.map(|kind| (site, kind)))
        .collect())
}

pub(super) fn reference_at_property(
    schema: &CfnSchema,
    site: CfnPropertyPath<'_>,
) -> Option<ReferenceKind> {
    let active = ACTIVE_SITES.with(|active| {
        active
            .borrow()
            .as_ref()
            .filter(|(resource, _)| resource == &schema.type_name)
            .map(|(_, sites)| sites.get(&site.to_string()).copied())
    });
    active.unwrap_or_else(|| {
        // Direct leaf-generation unit tests do not install a renderer context.
        reference_sites(schema)
            .unwrap_or_else(|error| panic!("{error}"))
            .get(&site.to_string())
            .copied()
    })
}
