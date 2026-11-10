use carina_core::provider::ProviderFactory;
use carina_core::schema::ValidatedProviderSchemaWitness;
use carina_provider_awscc::{AwsccProviderFactory, schemas};

#[test]
fn every_generated_schema_has_consistent_identity_declarations() {
    let resources = schemas::all_schemas();
    assert!(!resources.is_empty(), "must admit the generated catalog");
    let count = resources.len();
    let admitted = ValidatedProviderSchemaWitness::new(
        resources,
        AwsccProviderFactory.provider_config_attribute_types(),
        Default::default(),
        schemas::assignability_edges(),
    );
    let witness = admitted.unwrap_or_else(|error| {
        panic!("all {count} generated schemas must share consistent identity layers: {error:?}")
    });
    assert_eq!(witness.resources().len(), count);
}

#[test]
fn every_generated_schema_preserves_identity_and_member_layers_over_protocol() {
    use carina_core::protocol_convert::{protocol_to_resource_schema, resource_schema_to_protocol};

    let resources = schemas::all_schemas()
        .iter()
        .map(|schema| {
            let wire = resource_schema_to_protocol(schema).unwrap();
            let json = serde_json::to_string(&wire).unwrap();
            protocol_to_resource_schema(&serde_json::from_str(&json).unwrap()).unwrap()
        })
        .collect();
    ValidatedProviderSchemaWitness::new(
        resources,
        AwsccProviderFactory.provider_config_attribute_types(),
        Default::default(),
        schemas::assignability_edges(),
    )
    .expect("transported schemas must retain one definition per identity");
}
