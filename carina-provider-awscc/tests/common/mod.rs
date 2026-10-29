use carina_core::binding_index::ResolvedBindings;
use carina_core::executor::{
    ModuleConstraintGate, ProviderPreparationContext, prepare_provider_ready_resource,
};
use carina_core::provider::{
    AppliedIdentifier, AppliedState, CreateRequest, ProviderFactory, ProviderReadyResource,
    ReadOutcome,
};
use carina_core::resource::Resource;
use carina_core::schema::AttributeType;
use carina_core::schema::SchemaRegistry;
use carina_provider_awscc::AwsccNormalizer;

/// The schema under test comes from `SchemaRegistry`; this fixture only
/// supplies the owning factory required for validator dispatch.
struct OwnerOnlyAwsccFactory;

impl ProviderFactory for OwnerOnlyAwsccFactory {
    fn name(&self) -> &str {
        "awscc"
    }

    fn display_name(&self) -> &str {
        "AWS Cloud Control schema-only test factory"
    }

    fn provider_config_attribute_types(&self) -> carina_core::schema::ValidatedConfigSchema {
        carina_core::schema::ValidatedConfigSchema::default()
    }

    fn validate_config(
        &self,
        _attributes: &carina_core::provider::CanonicalProviderConfigAttributes,
    ) -> carina_core::provider::ProviderResult<()> {
        Ok(())
    }

    fn extract_region(
        &self,
        _attributes: &indexmap::IndexMap<String, carina_core::resource::Value>,
    ) -> String {
        "ap-northeast-1".to_string()
    }

    fn create_provider(
        &self,
        _binding: Option<&str>,
        _config: &carina_core::provider::ProviderReadyConfig,
    ) -> carina_core::provider::BoxFuture<
        '_,
        carina_core::provider::ProviderResult<Box<dyn carina_core::provider::Provider>>,
    > {
        Box::pin(async {
            Err::<Box<dyn carina_core::provider::Provider>, _>(
                carina_core::provider::ProviderError::internal(
                    "schema-only test factory cannot create providers",
                ),
            )
        })
    }

    fn schemas(&self) -> Vec<carina_core::schema::ValidatedResourceSchema> {
        Vec::new()
    }
}

#[allow(dead_code)]
pub fn assert_arn_identity(t: AttributeType, expected: &str) {
    let carina_core::schema::RawShape::String { identity, .. } = t.raw_shape() else {
        panic!("arn() should be a refined string");
    };
    assert_eq!(identity.map(|id| id.to_string()).as_deref(), Some(expected));
}

#[allow(dead_code)]
pub async fn normalize_resource(resource: Resource) -> ProviderReadyResource {
    let resource_type = resource.id.resource_type.clone();
    let bindings = ResolvedBindings::default();
    let module_gate = ModuleConstraintGate::from_live_compositions(&[]);
    let mut schemas = SchemaRegistry::new();
    schemas.insert("awscc", schema_for_test(&resource_type));
    let factories: Vec<Box<dyn ProviderFactory>> = vec![Box::new(OwnerOnlyAwsccFactory)];
    let preparation = ProviderPreparationContext::new(
        &bindings,
        &module_gate,
        &[],
        &AwsccNormalizer,
        &factories,
        &schemas,
    );
    prepare_provider_ready_resource(resource, &preparation)
        .await
        .expect("test resource should pass checked provider preparation")
}

#[allow(dead_code)]
pub async fn create_request(resource: Resource) -> CreateRequest {
    let resource_type = resource.id.resource_type.clone();
    let bindings = ResolvedBindings::default();
    let module_gate = ModuleConstraintGate::from_live_compositions(&[]);
    let mut schemas = SchemaRegistry::new();
    schemas.insert("awscc", schema_for_test(&resource_type));
    let factories: Vec<Box<dyn ProviderFactory>> = vec![Box::new(OwnerOnlyAwsccFactory)];
    let preparation = ProviderPreparationContext::new(
        &bindings,
        &module_gate,
        &[],
        &AwsccNormalizer,
        &factories,
        &schemas,
    );
    carina_core::executor::prepare_create_request(resource, &preparation)
        .await
        .expect("test resource should pass checked create preparation")
}

fn schema_for_test(resource_type: &str) -> carina_core::schema::ValidatedResourceSchema {
    let resource_type = resource_type.to_string();
    std::thread::Builder::new()
        .name("awscc-integration-schema".to_string())
        .stack_size(8 * 1024 * 1024)
        .spawn(move || {
            carina_provider_awscc::schemas::generated::get_config_by_type(&resource_type)
                .unwrap_or_else(|| panic!("missing test schema for {resource_type}"))
                .schema
                .clone()
                .validate()
                .expect("test schema must pass admission")
        })
        .expect("test schema thread should spawn")
        .join()
        .expect("test schema thread should not panic")
}

#[allow(dead_code)]
pub fn is_uuidish(value: &str) -> bool {
    value.len() == 36
        && value.bytes().enumerate().all(|(index, byte)| match index {
            8 | 13 | 18 | 23 => byte == b'-',
            _ => byte.is_ascii_hexdigit(),
        })
}

#[allow(dead_code)]
pub fn applied_identifier(value: impl Into<String>) -> AppliedIdentifier {
    AppliedIdentifier::new(value).expect("test provider identifier must be non-empty")
}

#[allow(dead_code)]
pub fn found_state(outcome: ReadOutcome) -> AppliedState {
    match outcome {
        ReadOutcome::Found { state } => state,
        ReadOutcome::NotFound { id } => panic!("expected {id} to be found"),
    }
}
