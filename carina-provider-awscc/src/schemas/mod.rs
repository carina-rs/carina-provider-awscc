//! AWS Cloud Control resource schema definitions

pub mod awscc_types;
pub mod generated;
mod operation_configs;

use carina_core::schema::ResourceSchema;

/// Returns all AWS Cloud Control schemas with operational configuration applied.
///
/// Generated schemas are enriched with per-resource operational config
/// (timeouts, retries) from `operation_configs` module.
pub fn all_schemas() -> Vec<ResourceSchema> {
    generated::configs()
        .iter()
        .map(|c| {
            let mut schema = c.schema.clone();
            if let Some(config) = operation_configs::get(c.resource_type_name) {
                schema.operation_config = Some(config);
            }
            schema
        })
        .collect()
}
