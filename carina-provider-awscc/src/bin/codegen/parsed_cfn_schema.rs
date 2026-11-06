//! Keep the legacy model and lossless Layer inseparable during migration.

use super::CfnSchema;
use super::schema_layer::{Layer, SchemaError};
use anyhow::{Context, Result};
use serde_json::Value;

#[derive(Debug)]
pub(super) struct ParsedCfnSchema {
    schema: CfnSchema,
    source: Value,
    layer: std::result::Result<std::sync::Arc<Layer>, SchemaError>,
}

impl ParsedCfnSchema {
    pub(super) fn from_json(source: Value) -> Result<Self> {
        let parsed = Self::parse(source)?;
        #[cfg(test)]
        if let Err(error) = parsed.checked_layer() {
            panic!("lossless CFN layer failed: {error}");
        }
        Ok(parsed)
    }

    fn parse(source: Value) -> Result<Self> {
        let layer = Layer::for_codegen(&source).map(std::sync::Arc::new);
        if let Err(error) = &layer {
            return Err(error.clone().into());
        }
        let mut schema: CfnSchema = serde_json::from_value(source.clone())
            .context("Failed to parse CloudFormation schema")?;
        if let Ok(layer) = &layer {
            schema.layer = Some(std::sync::Arc::clone(layer));
            schema.query_nodes = (0..layer.nodes().len())
                .map(|index| {
                    let id = super::schema_layer::NodeId(index);
                    (id, super::schema_projection::bridge(layer, id))
                })
                .collect();
            for (name, property) in &mut schema.properties {
                attach(
                    property,
                    layer,
                    &format!("/properties/{}", name.replace('~', "~0").replace('/', "~1")),
                );
            }
            if let Some(definitions) = &mut schema.definitions {
                for (name, property) in definitions {
                    attach(
                        property,
                        layer,
                        &format!(
                            "/definitions/{}",
                            name.replace('~', "~0").replace('/', "~1")
                        ),
                    );
                }
            }
        }
        Ok(Self {
            schema,
            source,
            layer,
        })
    }

    pub(super) fn source(&self) -> &Value {
        &self.source
    }

    pub(super) fn layer(&self) -> &std::result::Result<std::sync::Arc<Layer>, SchemaError> {
        &self.layer
    }

    #[cfg(test)]
    fn checked_layer(&self) -> std::result::Result<&Layer, &SchemaError> {
        let layer = self.layer.as_ref()?;
        if let Some(error) = layer.diagnostics().first() {
            return Err(error);
        }
        Ok(layer)
    }

    /// Reparse edits atomically so neither an error nor a panic leaves a mismatched pair.
    #[cfg(test)]
    pub(super) fn edit_json(&mut self, edit: impl FnOnce(&mut Value)) {
        let mut source = self.source.clone();
        edit(&mut source);
        *self = Self::from_json(source).expect("edited CFN fixture parses");
    }

    /// Existing consumer regressions may deliberately exercise invalid constraints.
    /// Require their exact layer diagnostic instead of silently bypassing the check.
    #[cfg(test)]
    pub(super) fn edit_json_with_layer_error(
        &mut self,
        edit: impl FnOnce(&mut Value),
        expected: SchemaError,
    ) {
        let mut source = self.source.clone();
        edit(&mut source);
        let parsed = Self::parse(source).expect("edited legacy CFN fixture parses");
        assert_eq!(parsed.checked_layer().unwrap_err(), &expected);
        *self = parsed;
    }
}

impl std::ops::Deref for ParsedCfnSchema {
    type Target = CfnSchema;

    fn deref(&self) -> &Self::Target {
        &self.schema
    }
}

// Compatibility conversion only: keep legacy tag/emission fields intact while
// connecting every existing carrier to its immutable, lossless schema site.
fn attach(property: &mut super::CfnProperty, layer: &std::sync::Arc<Layer>, pointer: &str) {
    if let Some(node) = layer.sources().node_at(pointer) {
        property.origin = Some(super::schema_projection::Origin {
            layer: std::sync::Arc::clone(layer),
            node,
        });
    }
    if let Some(items) = &mut property.items {
        attach(items, layer, &format!("{pointer}/items"));
    }
    for (keyword, properties) in [
        ("properties", &mut property.properties),
        ("patternProperties", &mut property.pattern_properties),
    ] {
        if let Some(properties) = properties {
            for (name, child) in properties {
                attach(
                    child,
                    layer,
                    &format!(
                        "{pointer}/{keyword}/{}",
                        name.replace('~', "~0").replace('/', "~1")
                    ),
                );
            }
        }
    }
    for (keyword, arms) in [
        ("oneOf", &mut property.one_of),
        ("anyOf", &mut property.any_of),
    ] {
        for (index, child) in arms.iter_mut().enumerate() {
            attach(child, layer, &format!("{pointer}/{keyword}/{index}"));
        }
    }
}
