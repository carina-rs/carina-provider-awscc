use std::collections::HashSet;
use std::sync::Arc;

use carina_core::binding_index::BindingIndex;
use carina_core::parser::{ParsedFile, ProviderContext};
use carina_core::provider::{
    ProviderFactory, collect_schemas, provider_custom_types_from_factories,
};
use carina_core::resource::{
    ConcreteValue, ModuleSource, ParserPlaceholder, Resource as GenericResource,
    Value as GenericValue,
};
use carina_core::schema::{
    ReferenceForms, SchemaRegistry, ValidatedConfigSchema, ValidatedProviderSchemaWitness,
};
use carina_core::validation::{validate_resource_ref_types, validate_resources};
use carina_provider_awscc::schemas::generated::scheduler::schedule::scheduler_schedule_config;
use carina_provider_awscc::schemas::generated::scheduler::schedule_group::scheduler_schedule_group_config;
use indexmap::IndexMap;

type Resource = GenericResource<ModuleSource, ParserPlaceholder>;
type Value = GenericValue<ParserPlaceholder>;

struct SchedulerSchemaFactory;

impl ProviderFactory for SchedulerSchemaFactory {
    fn name(&self) -> &str {
        "awscc"
    }

    fn display_name(&self) -> &str {
        "AWS Cloud Control Scheduler schema test factory"
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
        _attributes: &IndexMap<String, carina_core::resource::Value>,
    ) -> String {
        "ap-northeast-1".to_string()
    }

    fn create_provider(
        &self,
        _binding: Option<&str>,
        _config: &carina_core::provider::ProviderReadyConfig,
        _initialize_outcome: carina_core::provider::InitializeOutcomeMint,
    ) -> carina_core::provider::BoxFuture<
        '_,
        carina_core::provider::ProviderResult<carina_core::provider::ProviderInitialization>,
    > {
        Box::pin(async {
            Err(carina_core::provider::ProviderError::internal(
                "schema-only test factory cannot create providers",
            ))
        })
    }

    fn schemas(&self) -> Vec<carina_core::schema::ValidatedResourceSchema> {
        vec![
            scheduler_schedule_config()
                .schema
                .validate()
                .expect("Schedule schema must be valid"),
            scheduler_schedule_group_config()
                .schema
                .validate()
                .expect("ScheduleGroup schema must be valid"),
        ]
    }

    fn schema_witness(&self) -> ValidatedProviderSchemaWitness {
        ValidatedProviderSchemaWitness::new(
            self.schemas(),
            ValidatedConfigSchema::default(),
            ReferenceForms::new(),
        )
        .expect("Scheduler schema-only witness must be valid")
    }
}

fn string(value: &str) -> Value {
    Value::Concrete(ConcreteValue::String(value.to_string().into()))
}

fn enum_identifier(value: &str) -> Value {
    Value::Concrete(ConcreteValue::enum_identifier(value))
}

fn map(entries: impl IntoIterator<Item = (&'static str, Value)>) -> Value {
    Value::Concrete(ConcreteValue::Map(
        entries
            .into_iter()
            .map(|(key, value)| (key.to_string(), value))
            .collect::<IndexMap<_, _>>(),
    ))
}

fn scheduler_validation_context() -> (SchemaRegistry, ProviderContext) {
    let factories: Arc<Vec<Box<dyn ProviderFactory>>> =
        Arc::new(vec![Box::new(SchedulerSchemaFactory)]);
    let schemas = collect_schemas(&factories);
    let context = ProviderContext {
        custom_types: provider_custom_types_from_factories(factories),
        resource_types: ProviderContext::resource_types_from_schema_registry(&schemas),
        ..ProviderContext::default()
    };
    (schemas, context)
}

fn schedule(group_name: Option<Value>) -> Resource {
    let schedule =
        Resource::with_provider("awscc", "scheduler.Schedule", "validation_schedule", None)
            .with_attribute("name", string("carina-test-schedule"))
            .with_attribute(
                "flexible_time_window",
                map([("mode", enum_identifier("off"))]),
            )
            .with_attribute("schedule_expression", string("rate(6 hours)"))
            .with_attribute(
                "target",
                map([
                    (
                        "arn",
                        string("arn:aws:lambda:ap-northeast-1:123456789012:function:example:live"),
                    ),
                    (
                        "role_arn",
                        string("arn:aws:iam::123456789012:role/example-scheduler"),
                    ),
                ]),
            );

    match group_name {
        Some(group_name) => schedule.with_attribute("group_name", group_name),
        None => schedule,
    }
}

fn validate_scheduler(resources: Vec<Resource>) -> Result<(), String> {
    let mut parsed = ParsedFile::default();
    parsed.resources = resources;
    let (schemas, context) = scheduler_validation_context();
    let known_providers = HashSet::from(["awscc".to_string()]);

    validate_resources(&parsed, &schemas, &known_providers, &context)?;

    let bindings = BindingIndex::from_parsed(&parsed, &schemas);
    validate_resource_ref_types(&parsed, &schemas, &HashSet::new(), &bindings)
}

#[test]
fn scheduler_schedule_omitted_group_name_is_valid() {
    validate_scheduler(vec![schedule(None)])
        .expect("an omitted group_name must use the trackable default group");
}

#[test]
fn scheduler_schedule_default_group_name_literal_is_valid() {
    validate_scheduler(vec![schedule(Some(string("default")))])
        .expect("the literal default group must be trackable");
}

#[test]
fn scheduler_schedule_non_default_group_name_literal_is_rejected() {
    let error = validate_scheduler(vec![schedule(Some(string("carina-registry-publish-api")))])
        .expect_err("a non-default schedule group must be rejected before create");
    assert!(
        error.contains("Cloud Control identifies schedules by Name only")
            && error.contains("created but never read or tracked")
            && error.contains("group_name")
            && error.contains("'default'"),
        "expected the Cloud Control tracking limitation, got: {error}"
    );
}

#[test]
fn scheduler_schedule_group_name_reference_is_rejected() {
    let group =
        Resource::with_provider("awscc", "scheduler.ScheduleGroup", "validation_group", None)
            .with_binding("validation_group")
            .with_attribute("name", string("carina-registry-publish-api"));
    let group_name =
        Value::resource_ref("validation_group".to_string(), "name".to_string(), vec![]);

    let error = validate_scheduler(vec![group, schedule(Some(group_name))])
        .expect_err("an unresolved group_name cannot prove that the group is default");
    assert!(
        error.contains("group_name") && error.contains("aws.scheduler.Schedule.GroupName"),
        "expected an identified default-group type mismatch, got: {error}"
    );
}
