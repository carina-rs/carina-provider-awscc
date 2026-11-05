//! Shared schema assertions and checked-preparation fixtures for library tests.

use std::collections::HashSet;
use std::path::Path;
use std::sync::Arc;

use carina_core::binding_index::{BindingIndex, ResolvedBindings};
use carina_core::config_loader::load_buffer_configuration;
use carina_core::executor::{ModuleConstraintGate, ProviderPreparationContext};
use carina_core::parser::{
    LoadedProviderContext, ProviderContext, ProviderCustomTypes, parse_file,
};
use carina_core::provider::{ProviderFactory, provider_custom_types_from_factories};
use carina_core::resource::{ConcreteValue, Value};
use carina_core::schema::{
    AssignmentSink, Compiled, CompiledAttributeType, EffectiveType, SchemaKind, SchemaRegistry,
    ShapeWalkBudget, ValidatedResourceSchema, resolve_file_block_names_with_schemas,
};
use carina_core::validation::{
    ValidationTypeContext, check_bare_binding_uses, validate_resource_ref_types, validate_resources,
};

pub(crate) struct AssignmentContext {
    schemas: SchemaRegistry,
    catalog: ProviderCustomTypes,
}

impl AssignmentContext {
    pub(crate) fn new() -> Self {
        let factories: Arc<Vec<Box<dyn ProviderFactory>>> =
            Arc::new(vec![Box::new(crate::AwsccProviderFactory)]);
        Self {
            schemas: crate::schemas::schema_registry(),
            catalog: provider_custom_types_from_factories(factories),
        }
    }

    pub(crate) fn sink<'a>(
        &'a self,
        schema: &ValidatedResourceSchema,
        attr: &'a CompiledAttributeType,
    ) -> AssignmentSink<'a, Compiled> {
        self.schemas
            .get("awscc", schema.parts().resource_type, SchemaKind::Resource)
            .expect("assignment sink schema must be registered")
            .assignment_sink(attr, &self.catalog)
    }
}

pub(crate) fn field<'a>(schema: &'a ValidatedResourceSchema, path: &str) -> EffectiveType<'a> {
    let mut segments = path.split('.');
    let mut attr_type = schema
        .effective_attribute(segments.next().unwrap())
        .unwrap()
        .value_type();
    for segment in segments {
        while let Some(element) = attr_type.list_element().or_else(|| attr_type.map_value()) {
            attr_type = element;
        }
        attr_type = attr_type
            .find_field_with_budget(segment, &mut ShapeWalkBudget::new(64))
            .unwrap_or_else(|| panic!("missing schema field {path}"))
            .value_type();
    }
    attr_type
}

pub(crate) fn string(value: &str) -> Value {
    Value::Concrete(ConcreteValue::String(value.into()))
}

pub(crate) fn enum_identifier(value: &str) -> Value {
    Value::Concrete(ConcreteValue::enum_identifier(value))
}

pub(crate) fn int(value: i64) -> Value {
    Value::Concrete(ConcreteValue::Int(value))
}

pub(crate) fn float(value: f64) -> Value {
    Value::Concrete(ConcreteValue::Float(value))
}

pub(crate) fn list(items: Vec<Value>) -> Value {
    Value::Concrete(ConcreteValue::List(items))
}

pub(crate) fn validate_fixtures(cases: &[(&str, &str)]) {
    try_validate_fixtures(cases).unwrap_or_else(|error| panic!("{error}"));
}

pub(crate) fn try_validate_fixtures(cases: &[(&str, &str)]) -> Result<(), String> {
    let factories: Arc<Vec<Box<dyn ProviderFactory>>> =
        Arc::new(vec![Box::new(crate::AwsccProviderFactory)]);
    let schemas = crate::schemas::schema_registry();
    let loaded_context = LoadedProviderContext::for_test(ProviderContext {
        custom_types: provider_custom_types_from_factories(factories),
        resource_types: ProviderContext::resource_types_from_schema_registry(&schemas),
        ..ProviderContext::new(carina_core::EnvironmentSnapshot::empty())
    });
    let context = loaded_context.as_provider_context();
    let signatures = Default::default();
    for (path, source) in cases {
        let mut parsed = parse_file(source, Path::new(path), context)
            .map_err(|error| format!("{path}: {error}"))?;
        let configuration = load_buffer_configuration(&parsed, &schemas, context);
        let binding_uses = check_bare_binding_uses(&ValidationTypeContext::new(
            configuration.judged(),
            &loaded_context,
            &signatures,
        ));
        if !binding_uses.is_ready() {
            return Err(format!("{path}: {binding_uses}"));
        }
        // Match CLI order: carina-cli/src/wiring/mod.rs:3172 and commands/mod.rs:805/881.
        resolve_file_block_names_with_schemas(&mut parsed, &schemas).map_err(|errors| {
            format!(
                "{path}: {}",
                errors
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
                    .join("\n")
            )
        })?;
        carina_core::identifier::resolve_attr_prefixes(&mut parsed.resources, &schemas)
            .map_err(|error| format!("{path}: {error}"))?;
        validate_resources(
            &parsed,
            &schemas,
            &HashSet::from(["awscc".to_string()]),
            context,
        )
        .map_err(|error| format!("{path}: {error}"))?;
        let bindings = BindingIndex::from_parsed(&parsed, &schemas, context);
        let configuration = load_buffer_configuration(&parsed, &schemas, context);
        validate_resource_ref_types(configuration.judged(), &HashSet::new(), &bindings)
            .map_err(|error| format!("{path}: {error}"))?;
    }
    Ok(())
}

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
