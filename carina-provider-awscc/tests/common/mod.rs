use carina_core::binding_index::ResolvedBindings;
use carina_core::executor::{
    ModuleConstraintGate, ProviderPreparationContext, prepare_provider_ready_resource,
};
use carina_core::provider::ProviderReadyResource;
use carina_core::resource::Resource;
use carina_core::schema::AttributeType;
use carina_core::schema::SchemaRegistry;
use carina_provider_awscc::AwsccNormalizer;

#[allow(dead_code)]
pub fn assert_arn_identity(t: AttributeType, expected: &str) {
    let carina_core::schema::RawShape::String { identity, .. } = t.raw_shape() else {
        panic!("arn() should be a refined string");
    };
    assert_eq!(identity.map(|id| id.to_string()).as_deref(), Some(expected));
}

#[allow(dead_code)]
pub async fn normalize_resource(resource: Resource) -> ProviderReadyResource {
    let bindings = ResolvedBindings::default();
    let module_gate = ModuleConstraintGate::new(&[]);
    let mut schemas = SchemaRegistry::new();
    for schema in carina_provider_awscc::schemas::all_schemas() {
        schemas.insert("awscc", schema);
    }
    let preparation = ProviderPreparationContext::new(
        &bindings,
        &module_gate,
        &[],
        &AwsccNormalizer,
        &[],
        &schemas,
    );
    prepare_provider_ready_resource(resource, &preparation)
        .await
        .expect("test resource should pass checked provider preparation")
}

#[allow(dead_code)]
pub fn is_uuidish(value: &str) -> bool {
    value.len() == 36
        && value.bytes().enumerate().all(|(index, byte)| match index {
            8 | 13 | 18 | 23 => byte == b'-',
            _ => byte.is_ascii_hexdigit(),
        })
}
