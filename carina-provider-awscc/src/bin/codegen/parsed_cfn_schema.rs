//! Keep the parsed schema inseparable from the JSON used by the classifier.

use super::CfnSchema;
use anyhow::{Context, Result};
use serde_json::Value;

#[derive(Debug)]
pub(super) struct ParsedCfnSchema {
    schema: CfnSchema,
    source: Value,
}

impl ParsedCfnSchema {
    pub(super) fn from_json(source: Value) -> Result<Self> {
        let schema = serde_json::from_value(source.clone())
            .context("Failed to parse CloudFormation schema")?;
        Ok(Self { schema, source })
    }

    pub(super) fn source(&self) -> &Value {
        &self.source
    }

    /// Reparse edits atomically so neither an error nor a panic leaves a mismatched pair.
    #[cfg(test)]
    pub(super) fn edit_json(&mut self, edit: impl FnOnce(&mut Value)) {
        let mut source = self.source.clone();
        edit(&mut source);
        *self = Self::from_json(source).expect("edited CFN fixture parses");
    }
}

impl std::ops::Deref for ParsedCfnSchema {
    type Target = CfnSchema;

    fn deref(&self) -> &Self::Target {
        &self.schema
    }
}
