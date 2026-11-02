//! Shared checked-preparation fixture for library tests.

use carina_core::binding_index::ResolvedBindings;
use carina_core::executor::{ModuleConstraintGate, ProviderPreparationContext};
use carina_core::provider::ProviderFactory;
use carina_core::schema::SchemaRegistry;

pub(crate) async fn with_preparation<T>(
    resource_type: &str,
    run: impl for<'a> AsyncFnOnce(&'a ProviderPreparationContext<'a>) -> T,
) -> T {
    carina_core::validation::deferred_types::with_session_for_test_async(
        &carina_core::parser::LoadedProviderContext::empty_for_test(),
        async |type_checks| {
            let bindings = ResolvedBindings::default();
            let gate = ModuleConstraintGate::from_live_compositions(&[]);
            let mut schemas = SchemaRegistry::new();
            let config = crate::schemas::generated::get_config_by_type(resource_type)
                .unwrap_or_else(|| panic!("missing test schema for {resource_type}"));
            schemas.insert("awscc", config.schema.clone());
            let factories: Vec<Box<dyn ProviderFactory>> =
                vec![Box::new(crate::OwnerOnlyAwsccFactory)];
            let preparation = ProviderPreparationContext::new(
                &bindings,
                &gate,
                &[],
                &crate::AwsccNormalizer,
                &factories,
                &schemas,
                type_checks,
            );
            run(&preparation).await
        },
    )
    .await
}
