//! Parse raw input at the Layer boundary, then expose typed projection handles.

use super::CfnSchema;
use super::schema_layer::{Layer, SchemaError};
use anyhow::Result;
use serde_json::Value;

#[derive(Debug)]
pub(super) struct ParsedCfnSchema {
    schema: CfnSchema,
    #[cfg(test)]
    pub(super) source: Value,
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
        let typed = layer.as_ref().map_err(Clone::clone)?;
        let root_projection = super::schema_projection::direct(typed, typed.resource().root)?;
        let properties = root_projection.properties.clone().unwrap_or_default();
        let definitions = typed.resource().definitions.as_ref().map(|definitions| {
            definitions
                .iter()
                .map(|(name, id)| (name.clone(), super::schema_projection::bridge(typed, *id)))
                .collect()
        });
        let schema = CfnSchema {
            layer: Some(std::sync::Arc::clone(typed)),
            query_nodes: (0..typed.nodes().len())
                .map(|index| {
                    let id = super::schema_layer::NodeId(index);
                    (id, super::schema_projection::bridge(typed, id))
                })
                .collect(),
            root_projection,
            type_name: typed.resource().type_name.clone().unwrap_or_default(),
            properties,
            definitions,
            #[cfg(test)]
            tagging: typed
                .resource()
                .tagging
                .as_ref()
                .map(|tagging| super::CfnTagging {
                    taggable: tagging.taggable.unwrap_or(false),
                    tag_property: tagging.tag_property.clone(),
                }),
        };
        Ok(Self {
            schema,
            #[cfg(test)]
            source,
            layer,
        })
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
