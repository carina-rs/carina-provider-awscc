//! Keep the parsed schema inseparable from the JSON used by the classifier.

use super::CfnSchema;
use super::schema_layer::{Layer, SchemaError};
use anyhow::{Context, Result};
use serde_json::Value;

#[derive(Debug)]
pub(super) struct ParsedCfnSchema {
    schema: CfnSchema,
    source: Value,
    // Preserve diagnostics without changing legacy CLI behavior during migration.
    #[allow(dead_code)]
    layer: std::result::Result<Layer, SchemaError>,
}

impl ParsedCfnSchema {
    pub(super) fn from_json(source: Value) -> Result<Self> {
        let parsed = Self::parse(source)?;
        #[cfg(test)]
        if let Err(error) = &parsed.layer {
            panic!("lossless CFN layer failed: {error}");
        }
        Ok(parsed)
    }

    fn parse(source: Value) -> Result<Self> {
        let schema = serde_json::from_value(source.clone())
            .context("Failed to parse CloudFormation schema")?;
        let layer = Layer::from_json(&source);
        Ok(Self {
            schema,
            source,
            layer,
        })
    }

    pub(super) fn source(&self) -> &Value {
        &self.source
    }

    #[cfg(test)]
    pub(super) fn layer(&self) -> &std::result::Result<Layer, SchemaError> {
        &self.layer
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
        assert_eq!(parsed.layer.as_ref().unwrap_err(), &expected);
        *self = parsed;
    }
}

impl std::ops::Deref for ParsedCfnSchema {
    type Target = CfnSchema;

    fn deref(&self) -> &Self::Target {
        &self.schema
    }
}
