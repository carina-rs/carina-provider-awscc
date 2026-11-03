use std::collections::HashSet;
use std::path::Path;
use std::sync::Arc;

use carina_core::binding_index::BindingIndex;
use carina_core::parser::{ProviderContext, parse_file};
use carina_core::provider::{
    ProviderFactory, collect_schemas, provider_custom_types_from_factories,
};
use carina_core::schema::{
    ReferenceForms, SchemaRegistry, ValidatedConfigSchema, ValidatedProviderSchemaWitness,
};
use carina_core::validation::{validate_resource_ref_types, validate_resources};
use carina_provider_awscc::schemas::generated::scheduler::schedule::scheduler_schedule_config;
use carina_provider_awscc::schemas::generated::scheduler::schedule_group::scheduler_schedule_group_config;

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
            scheduler_schedule_config().schema,
            scheduler_schedule_group_config().schema,
        ]
    }

    fn schema_witness(&self) -> ValidatedProviderSchemaWitness {
        ValidatedProviderSchemaWitness::new(
            self.schemas(),
            ValidatedConfigSchema::default(),
            ReferenceForms::new(),
            carina_provider_awscc::schemas::assignability_edges(),
        )
        .expect("Scheduler schema-only witness must be valid")
    }
}

fn scheduler_validation_context() -> (SchemaRegistry, ProviderContext) {
    let factories: Arc<Vec<Box<dyn ProviderFactory>>> =
        Arc::new(vec![Box::new(SchedulerSchemaFactory)]);
    let schemas = collect_schemas(&factories);
    let context = ProviderContext {
        custom_types: provider_custom_types_from_factories(factories),
        resource_types: ProviderContext::resource_types_from_schema_registry(&schemas),
        ..ProviderContext::new(carina_core::EnvironmentSnapshot::empty())
    };
    (schemas, context)
}

fn schedule_fixture(group_name: Option<&str>) -> String {
    let group_name = group_name
        .map(|value| format!("group_name = {value}"))
        .unwrap_or_default();
    format!(
        r#"
awscc.scheduler.Schedule {{
  name = 'carina-test-schedule'
  {group_name}
  schedule_expression = 'rate(6 hours)'
  flexible_time_window = {{
    mode = aws.scheduler.Schedule.FlexibleTimeWindow.FlexibleTimeWindowMode.off
  }}
  target = {{
    arn = 'arn:aws:lambda:ap-northeast-1:123456789012:function:example:live'
    role_arn = 'arn:aws:iam::123456789012:role/example-scheduler'
  }}
}}
"#
    )
}

fn validate_scheduler(source: &str) -> Result<(), String> {
    let (schemas, context) = scheduler_validation_context();
    let parsed = parse_file(source, Path::new("scheduler_validation.crn"), &context)
        .map_err(|error| error.to_string())?;
    let known_providers = HashSet::from(["awscc".to_string()]);

    validate_resources(&parsed, &schemas, &known_providers, &context)?;

    let bindings = BindingIndex::from_parsed(&parsed, &schemas, &context);
    validate_resource_ref_types(&parsed, &schemas, &HashSet::new(), &bindings)
        .map_err(|error| error.to_string())
}

#[test]
fn scheduler_schedule_omitted_group_name_is_valid() {
    validate_scheduler(&schedule_fixture(None))
        .expect("an omitted group_name must use the trackable default group");
}

#[test]
fn scheduler_schedule_default_group_name_literal_is_valid() {
    validate_scheduler(&schedule_fixture(Some("'default'")))
        .expect("the literal default group must be trackable");
}

#[test]
fn scheduler_schedule_non_default_group_name_literal_is_rejected() {
    let error = validate_scheduler(&schedule_fixture(Some("'carina-registry-publish-api'")))
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
    let source = format!(
        r#"
let validation_group = awscc.scheduler.ScheduleGroup {{
  name = 'carina-registry-publish-api'
}}
{}
"#,
        schedule_fixture(Some("validation_group.name"))
    );
    let error = validate_scheduler(&source)
        .expect_err("an unresolved group_name cannot prove that the group is default");
    assert!(
        error.contains("group_name") && error.contains("aws.scheduler.Schedule.GroupName"),
        "expected an identified default-group type mismatch, got: {error}"
    );
}
