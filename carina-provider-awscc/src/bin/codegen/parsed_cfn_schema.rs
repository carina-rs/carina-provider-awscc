//! Keep the legacy model and lossless Layer inseparable during migration.

use super::CfnSchema;
use super::schema_layer::{Layer, SchemaError};
use anyhow::{Context, Result};
use serde_json::Value;

#[derive(Debug)]
pub(super) struct ParsedCfnSchema {
    schema: CfnSchema,
    source: Value,
    layer: std::result::Result<Layer, SchemaError>,
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
        let schema = serde_json::from_value(source.clone())
            .context("Failed to parse CloudFormation schema")?;
        let layer = Layer::for_codegen(&source);
        Ok(Self {
            schema,
            source,
            layer,
        })
    }

    pub(super) fn source(&self) -> &Value {
        &self.source
    }

    pub(super) fn layer(&self) -> &std::result::Result<Layer, SchemaError> {
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
