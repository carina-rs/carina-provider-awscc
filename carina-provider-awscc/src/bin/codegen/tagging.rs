//! Explicit classification of top-level tag-shaped properties.
//! Only top-level properties can be a resource's tag attribute: carina-core merges
//! default_tags and runs TagsKeyValueCheck on the top-level `tags` only.
//! Nested Key/Value arrays such as S3 TagFilters are outside this guard.

use anyhow::{Result, ensure};
use heck::ToSnakeCase;
use std::collections::BTreeSet;

use super::{
    CfnSchema, EmissionContext, TagShape, derive_tags_property, resource_type_overrides,
    should_omit_top_level_property, synthetic_attributes, tag_shape,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum TaggingOverride {
    TagProperty,
    NonTagList { reason: &'static str },
}

/// CFN metadata, conventional Tags and declarations identify one tag property.
pub(super) fn tag_property_path(schema: &CfnSchema) -> Result<Option<&str>> {
    let mut paths = BTreeSet::new();
    if let Some(resource) = resource_type_overrides()
        .by_type
        .get(schema.type_name.as_str())
    {
        paths.extend(
            resource
                .tagging
                .iter()
                .filter_map(|(path, classification)| {
                    matches!(classification, TaggingOverride::TagProperty).then_some(*path)
                }),
        );
    }
    if let Some(tagging) = schema
        .metadata()
        .tagging
        .as_ref()
        .filter(|tagging| tagging.taggable == Some(true))
    {
        paths.insert(
            tagging
                .tag_property
                .as_deref()
                .unwrap_or("/properties/Tags"),
        );
    }
    if schema.properties.contains_key("Tags") {
        paths.insert("/properties/Tags");
    }
    ensure!(
        paths.len() <= 1,
        "{} has multiple tag properties: {}. A resource must have exactly one DSL `tags` attribute",
        schema.type_name,
        paths.iter().copied().collect::<Vec<_>>().join(", ")
    );
    Ok(paths.into_iter().next())
}

/// Core merges default_tags and validates tag pairs through this exact DSL name.
pub(super) fn dsl_attribute_name(schema: &CfnSchema, provider_name: &str) -> Result<String> {
    Ok(
        if super::is_dsl_tags_property(schema, &[provider_name.to_string()])? {
            "tags".to_string()
        } else {
            provider_name.to_snake_case()
        },
    )
}

/// A Key/Value list must never silently escape resource-tag classification.
/// Names and definition prefixes cannot distinguish tags from ELB attributes.
pub(super) fn validate_tagging_classifications(schema: &EmissionContext<'_>) -> Result<()> {
    let tags = derive_tags_property(schema)?;
    if let Some(tags) = &tags {
        for name in schema
            .properties
            .keys()
            .filter(|name| !should_omit_top_level_property(&schema.type_name, name))
        {
            ensure!(
                *name == tags.provider_name || name.to_snake_case() != "tags",
                "{}#/properties/{}: DSL `tags` name collision with /properties/{name}",
                schema.type_name,
                tags.provider_name
            );
        }
        ensure!(
            !synthetic_attributes().iter().any(|attribute| {
                attribute.cfn_type == schema.type_name && attribute.attr_name == "tags"
            }),
            "{}#/properties/{}: DSL `tags` name collision with a synthetic attribute",
            schema.type_name,
            tags.provider_name
        );
    }
    let overrides = resource_type_overrides()
        .by_type
        .get(schema.type_name.as_str());
    if let Some(resource) = overrides {
        for (path, classification) in &resource.tagging {
            let name = path
                .strip_prefix("/properties/")
                .filter(|name| !name.is_empty() && !name.contains('/'));
            ensure!(
                name.is_some(),
                "{}#{path}: tagging override must name one top-level CFN property",
                schema.type_name
            );
            match classification {
                // tag_property_path already rejects conflicting declarations.
                TaggingOverride::TagProperty => {}
                TaggingOverride::NonTagList { reason } => {
                    ensure!(
                        !reason.trim().is_empty(),
                        "{}#{path}: NonTagList classification requires a reason",
                        schema.type_name
                    );
                    ensure!(
                        tags.as_ref()
                            .is_none_or(|tags| Some(tags.provider_name.as_str()) != name),
                        "{}#{path}: NonTagList classification conflicts with the resource tag property",
                        schema.type_name
                    );
                }
            }
        }
    }
    let mut unclassified = Vec::new();
    for name in schema.top_level_property_names() {
        let sources = schema.top_level_property_sources(name);
        let layer = schema.layer.as_ref().expect("parsed schema has a Layer");
        if !TagShape::candidates(sources.iter().map(|pointer| {
            let node = layer
                .sources()
                .node_at(pointer)
                .expect("top-level property source");
            tag_shape::classify_tag_shape(layer, node)
        })) || tags.as_ref().is_some_and(|tags| tags.provider_name == name)
        {
            continue;
        }
        let path = format!("/properties/{name}");
        if overrides.is_some_and(|resource| {
            matches!(
                resource.tagging.get(path.as_str()),
                Some(TaggingOverride::NonTagList { reason: _reason })
            )
        }) {
            continue;
        }
        unclassified.push(path);
    }
    ensure!(
        unclassified.is_empty(),
        "{} has unclassified tag-shaped top-level arrays: {}. Declare each raw CFN path in resource_type_overrides() with TagProperty or NonTagList {{ reason: ... }}; default_tags must not be silently dropped",
        schema.type_name,
        unclassified.join(", ")
    );
    Ok(())
}
