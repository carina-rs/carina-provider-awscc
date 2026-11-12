use super::doc_test_support::{NAMED_OVERRIDES, named_schema};
use super::*;
use carina_core::schema::{AttributeSchema, ResourceSchema};

fn json_differences(
    left: &serde_json::Value,
    right: &serde_json::Value,
    path: &str,
    differences: &mut Vec<String>,
) {
    match (left, right) {
        (serde_json::Value::Object(left), serde_json::Value::Object(right)) => {
            for key in left.keys().chain(right.keys()).collect::<BTreeSet<_>>() {
                let path = format!("{path}/{key}");
                match (left.get(key), right.get(key)) {
                    (Some(left), Some(right)) => json_differences(left, right, &path, differences),
                    _ => differences.push(path),
                }
            }
        }
        (serde_json::Value::Array(left), serde_json::Value::Array(right))
            if left.len() == right.len() =>
        {
            for (index, (left, right)) in left.iter().zip(right).enumerate() {
                json_differences(left, right, &format!("{path}/{index}"), differences);
            }
        }
        _ if left != right => differences.push(path.into()),
        _ => {}
    }
}

#[test]
fn cached_witnesses_equal_full_runtime_schemas() {
    let mut differences = Vec::new();
    for config in carina_provider_awscc::schemas::generated::configs() {
        let parsed = cfn_schema_for_codegen_tests(config.aws_type_name);
        let context = EmissionContext::from_source(&parsed).unwrap();
        let emission = emit_resource(&context, config.aws_type_name, false).unwrap();
        let actual = serde_json::to_value(
            carina_core::protocol_convert::resource_schema_to_protocol(&emission.witness).unwrap(),
        )
        .unwrap();
        let expected = serde_json::to_value(
            carina_core::protocol_convert::resource_schema_to_protocol(&config.schema).unwrap(),
        )
        .unwrap();
        json_differences(&actual, &expected, config.aws_type_name, &mut differences);
        assert_eq!(
            emission.witness.parts().default_wait_timeout,
            config.schema.parts().default_wait_timeout
        );
        assert_eq!(
            emission.witness.parts().default_wait_interval,
            config.schema.parts().default_wait_interval
        );
    }
    assert!(
        differences.is_empty(),
        "full witness drift at {} fields:\n{}",
        differences.len(),
        differences
            .iter()
            .take(100)
            .cloned()
            .collect::<Vec<_>>()
            .join("\n")
    );
}

#[test]
fn docs_emission_retains_resource_builder_facts() {
    let parsed = cfn_schema_for_codegen_tests("AWS::EC2::VPC");
    let rust = emit_resource(
        &EmissionContext::from_source(&parsed).unwrap(),
        &parsed.type_name,
        true,
    )
    .unwrap();
    let docs = emit_resource(
        &EmissionContext::from_source(&parsed).unwrap(),
        &parsed.type_name,
        false,
    )
    .unwrap();
    assert!(
        serde_json::to_value(
            carina_core::protocol_convert::resource_schema_to_protocol(&rust.witness).unwrap()
        )
        .unwrap()
            == serde_json::to_value(
                carina_core::protocol_convert::resource_schema_to_protocol(&docs.witness).unwrap()
            )
            .unwrap(),
        "the docs witness dropped a resource builder fact"
    );
}

#[test]
fn conflicting_definition_annotations_are_errors() {
    with_resource_type_overrides(&NAMED_OVERRIDES, || {
        let parsed = named_schema();
        let context = EmissionContext::from_source(&parsed).unwrap();
        reset_recursion_guard();
        let emit = |name: &str| {
            super::super::generate_attribute_type(
                context.property_at(&[name.to_string()], &context.properties[name]),
                "aws.test.NamedNotes",
                &BTreeMap::new(),
                CfnTypeGenerationPaths {
                    structural: &[],
                    provider: &[name.to_string()],
                    cfn_parent: None,
                },
            )
        };
        emit("Left").unwrap();
        EMITTED_TYPES.with(|defs| {
            defs.borrow_mut()
                .get_mut("SharedText")
                .unwrap()
                .annotations
                .descriptions
                .insert("conflict".into(), "different definition metadata".into());
        });
        let error =
            emit("Right").expect_err("conflicting definition annotations must not be first-wins");
        assert!(
            error
                .to_string()
                .contains("conflicting local type definition SharedText"),
            "{error:#}"
        );
    });
}

#[test]
fn conflicting_definition_bodies_are_errors() {
    static OVERRIDES: LazyLock<ResourceTypeOverrides> = LazyLock::new(|| {
        let mut overrides = ResourceTypeOverrides::default();
        for (name, max) in [("Left", 3), ("Right", 9)] {
            overrides.insert(
                ("AWS::Test::NamedNotes", name),
                NarrowingKind::NamedType {
                    name: "SharedText",
                    expression: ConstraintSpec::string(None, Some((None, Some(max)))),
                },
            );
        }
        overrides
    });
    with_resource_type_overrides(&OVERRIDES, || {
        let parsed = named_schema();
        let error = generate_test_markdown(&parsed, &parsed.type_name)
            .expect_err("different final constraints cannot share one definition key");
        assert!(
            error
                .to_string()
                .contains("conflicting local type definition SharedText"),
            "{error:#}"
        );
    });
}

#[test]
fn runtime_create_only_parent_is_documented() {
    let parsed = cfn_schema_for_codegen_tests("AWS::EC2::VPCEndpoint");
    let md = generate_test_markdown(&parsed, &parsed.type_name).unwrap();
    let section = md
        .split("### `dns_options`\n")
        .nth(1)
        .unwrap()
        .split("\n### ")
        .next()
        .unwrap();
    assert!(section.contains("- **Create-only:** Yes"), "{section}");
}

#[test]
fn output_builders_preserve_declared_block_names() {
    let facts = type_emission::FieldFacts::default();
    let mode = type_emission::FieldMode::Output {
        block_name: Some("entry".into()),
    };
    let field = type_emission::field(
        "entries",
        AttributeType::list(AttributeType::bool()),
        mode,
        &facts,
    );
    assert_eq!(field.parts().block_name, Some("entry"));
    // Output attributes cannot declare blocks in the pinned core API. The
    // attribute mode has no slot for that unsupported fact.
    let attribute = type_emission::attribute(
        "entries",
        AttributeType::list(AttributeType::bool()),
        type_emission::AttributeMode::Output,
        &facts,
    );
    assert_eq!(attribute.parts().block_name, None);
}

#[test]
fn witness_preserves_the_typed_validation_error_chain() {
    let error = type_emission::witness(
        ResourceSchema::new("test.Round3")
            .attribute(AttributeSchema::optional("probe", AttributeType::ref_("Missing")).build()),
        &BTreeMap::new(),
    )
    .unwrap_err();
    assert!(
        error
            .downcast_ref::<carina_core::schema::ResourceSchemaValidationError>()
            .is_some(),
        "{error:#}"
    );
}

#[test]
fn prescan_reset_clears_repeated_named_enum_bodies() {
    reset_recursion_guard();
    let parsed = cfn_schema_for_codegen_tests("AWS::S3::Bucket");
    let context = EmissionContext::from_source(&parsed).unwrap();
    let definitions = schema_projection::definition_enum_fields(
        &context,
        &BTreeSet::from(["LambdaConfiguration".into()]),
    )
    .unwrap();
    let definition = definitions
        .iter()
        .find(|definition| definition.name == "LambdaConfiguration")
        .unwrap();
    let path = vec!["LambdaConfiguration".into(), "Event".into()];
    for _ in 0..2 {
        super::super::generate_attribute_type(
            context.property_at(&path, &definition.properties["Event"]),
            "aws.s3.Bucket",
            &BTreeMap::new(),
            CfnTypeGenerationPaths {
                structural: &path[..1],
                provider: &path,
                cfn_parent: Some("LambdaConfiguration"),
            },
        )
        .unwrap();
    }
    assert!(EMITTED_TYPES.with(|types| types.borrow().has_duplicates()));
    reset_emission_pass();
    assert!(
        !EMITTED_TYPES.with(|types| types.borrow().has_duplicates()),
        "pre-scan duplicate bodies survived their definitions"
    );
}
