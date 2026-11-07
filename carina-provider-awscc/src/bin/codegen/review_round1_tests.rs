use super::*;
use serde_json::json;

fn fixture(property: serde_json::Value) -> ParsedCfnSchema {
    ParsedCfnSchema::from_json(json!({
        "typeName":"AWS::Test::Emission", "properties":{"Probe":property}
    }))
    .unwrap()
}

fn assert_projection_error(schema: &ParsedCfnSchema) {
    for result in [
        generate_synthetic_schema_code(schema),
        generate_test_markdown(schema, "AWS::EC2::Route"),
    ] {
        let error = result.expect_err("unrepresentable facts must produce a typed error");
        assert!(
            matches!(
                error.downcast_ref::<schema_layer::SchemaError>(),
                Some(schema_layer::SchemaError::Projection { .. })
            ),
            "{error:#}"
        );
    }
}

#[test]
fn review_r7_document_containment_retains_string_formats() {
    let mut overrides = ResourceTypeOverrides::default();
    overrides.insert(
        ("AWS::Test::Emission", "Probe"),
        TypeOverride::identity(None).with_narrowing(
            NarrowingKind::Primitive {
                expression: ConstraintSpec::document(DocumentKind::SnsSubscriptionRedrivePolicy),
                display: OverrideTypeDisplay::new("RedrivePolicy"),
                block_name: None,
                default: None,
            },
            AwsEvidence::fixture(),
        ),
    );
    with_resource_type_overrides(Box::leak(Box::new(overrides)), || {
        let property = json!({"type":"object", "properties":{
            "deadLetterTargetArn":{"type":"string", "format":"date-time"}
        }});
        assert_projection_error(&fixture(property));
        let supported = fixture(json!({"type":"object", "properties":{
            "deadLetterTargetArn":{"type":"string"}
        }}));
        generate_synthetic_schema_code(&supported).unwrap();
        generate_test_markdown(&supported, "AWS::EC2::Route").unwrap();
    });
}

#[test]
fn review_r7_bare_object_narrowings_have_evidence_in_both_outputs() {
    let schema = cfn_schema_for_codegen_tests("AWS::WAFv2::WebACL");
    generate_schema_code(&schema, &schema.type_name).unwrap();
    let evidence = overrides::global_narrowings();
    for entry in EMPTY_OBJECT_PROPERTIES {
        assert!(
            evidence.contains_key(&entry.property_path.source_pointer()),
            "missing narrowing evidence for {}",
            entry.property_path
        );
    }
    let markdown = generate_test_markdown(&schema, &schema.type_name).unwrap();
    for entry in EMPTY_OBJECT_PROPERTIES {
        assert!(
            markdown.contains(&format!(
                "| `{}` | Subset:",
                entry.property_path.source_pointer()
            )),
            "missing documented narrowing for {}",
            entry.property_path
        );
    }
}

#[test]
fn review_r7_null_only_types_reject_conflicting_literals_and_intersections() {
    for literal in [json!({"const":"x"}), json!({"enum":["x",1]})] {
        let mut direct = literal.clone();
        direct["type"] = json!("null");
        for property in [
            direct,
            json!({"allOf":[{"type":"null"},literal]}),
            json!({"allOf":[literal,{"type":["null"]}]}),
        ] {
            assert_projection_error(&fixture(property));
        }
    }
    assert_projection_error(&fixture(json!({"type":"null"})));
    let nullable = fixture(json!({"type":["null","string"],"const":"x"}));
    generate_synthetic_schema_code(&nullable).unwrap();
    generate_test_markdown(&nullable, "AWS::EC2::Route").unwrap();
}

#[test]
fn review_r7_float_endpoints_must_be_exactly_representable() {
    for keyword in ["minimum", "maximum"] {
        for endpoint in [9007199254740993_i64, -9007199254740993, i64::MAX] {
            assert_projection_error(&fixture(json!({"type":"number",keyword:endpoint})));
        }
        for endpoint in [9007199254740992_i64, 9007199254740994, i64::MIN] {
            let schema = fixture(json!({"type":"number",keyword:endpoint}));
            generate_synthetic_schema_code(&schema).unwrap();
            generate_test_markdown(&schema, "AWS::EC2::Route").unwrap();
        }
    }
}

#[test]
fn review_r7_named_string_displays_include_effective_local_constraints() {
    for schema in [
        ParsedCfnSchema::from_json(json!({"typeName":"AWS::Test::Emission", "properties":{
            "VpcId":{"type":"string","minLength":37,"maxLength":49}
        }}))
        .unwrap(),
        ParsedCfnSchema::from_json(json!({"typeName":"AWS::Test::Emission", "properties":{
            "Settings":{"type":"object","properties":{
                "VpcId":{"type":"string","minLength":37,"maxLength":49}
            }}
        }}))
        .unwrap(),
    ] {
        let rust = generate_synthetic_schema_code(&schema).unwrap();
        assert!(rust.contains("Some((Some(37), Some(49)))"));
        let markdown = generate_test_markdown(&schema, "AWS::EC2::Route").unwrap();
        assert!(
            markdown
                .lines()
                .any(|line| line.contains("aws.ec2.Vpc.Id") && line.contains("37..=49")),
            "named string lost its local length constraints"
        );
    }
}

#[test]
fn review_r7_nested_and_collection_defaults_reach_both_outputs() {
    for property in [
        json!({"type":"object","properties":{"Enabled":{"type":"boolean","default":false}}}),
        json!({"type":"array","items":{"type":"string"},"default":["alpha","beta"]}),
        json!({"type":"object","properties":{"Enabled":{"type":"boolean"}},"default":{"Enabled":false}}),
        json!({"type":"object","patternProperties":{".*":{"type":"string"}},
            "additionalProperties":false,"default":{"Key":"value"}}),
    ] {
        let schema = fixture(property);
        let rust = generate_synthetic_schema_code(&schema).unwrap();
        assert!(
            rust.contains(".with_default(Value::Concrete(") || rust.contains("CFN default: false"),
            "default disappeared from Rust, including unsupported nested-default metadata"
        );
        let markdown = generate_test_markdown(&schema, "AWS::EC2::Route").unwrap();
        assert!(
            markdown.contains("Default:"),
            "default disappeared from Markdown"
        );
    }
}

#[test]
fn review_r7_unsupported_defaults_are_explicitly_diagnosed() {
    for property in [
        json!({"type":"string","default":null}),
        json!({"type":"array","items":{"type":"string"},"default":[null]}),
    ] {
        assert_projection_error(&fixture(property));
    }
    // A CFN tag list becomes a DSL map, so installing its raw default
    // would have the wrong value shape even though ordinary lists are supported.
    let tag = json!({"type":"object","properties":{
        "Key":{"type":"string"},"Value":{"type":"string"}
    }});
    let schema = ParsedCfnSchema::from_json(json!({
        "typeName":"AWS::Test::Emission","properties":{"Tags":{
            "type":"array","items":tag,"default":[{"Key":"env","Value":"prod"}]
        }}
    }))
    .unwrap();
    assert_projection_error(&schema);
    // CFN's cached policy default is a JSON string on a structured DSL field.
    // Preserve the annotation and explain why it remains a server-side default.
    let schema = cfn_schema_for_codegen_tests("AWS::KMS::Key");
    let rust = generate_schema_code(&schema, &schema.type_name).unwrap();
    let markdown = generate_test_markdown(&schema, &schema.type_name).unwrap();
    assert!(rust.contains("CFN default") && rust.contains("server"));
    assert!(markdown.contains("server") && markdown.contains("Enable IAM User Permissions"));
}

#[test]
fn review_r7_defaults_without_field_sites_are_typed_errors() {
    for property in [
        json!({"type":"array","items":{"type":"string","default":"x"}}),
        json!({"type":"object","patternProperties":{".*":{"type":"string","default":"x"}},"additionalProperties":false}),
    ] {
        assert_projection_error(&fixture(property));
    }
}

#[test]
fn review_r7_named_string_displays_include_pattern_only_constraints() {
    let schema =
        ParsedCfnSchema::from_json(json!({"typeName":"AWS::Test::Emission", "properties":{
            "VpcId":{"type":"string","pattern":"^vpc-[a-z]+$"}
        }}))
        .unwrap();
    let markdown = generate_test_markdown(&schema, "AWS::EC2::Route").unwrap();
    assert!(
        markdown
            .lines()
            .any(|line| line.contains("aws.ec2.Vpc.Id") && line.contains("pattern"))
    );
}

#[test]
fn review_r7_codegen_guide_uses_evidenced_same_kind_overrides() {
    let guide = include_str!("../../../../CODEGEN.md");
    assert!(!guide.contains("TypeOverride::Array"));
    assert!(!guide.contains("IntegerFromNumber"));
    for example in [
        "TypeOverride::identity",
        "NarrowingKind::Array",
        "AwsEvidence::new",
        "with_narrowing",
    ] {
        assert!(
            guide.contains(example),
            "missing current API example: {example}"
        );
    }
}

#[test]
fn review_r7_top_level_arn_displays_include_local_constraints() {
    let mut schema = cfn_schema_for_codegen_tests("AWS::IAM::Role");
    schema.edit_json(|source| {
        source["properties"]["Arn"]["minLength"] = json!(37);
        source["properties"]["Arn"]["maxLength"] = json!(49);
    });
    let rust = generate_schema_code(&schema, &schema.type_name).unwrap();
    assert!(rust.contains("Some((Some(37), Some(49)))"));
    let markdown = generate_test_markdown(&schema, &schema.type_name).unwrap();
    assert!(
        markdown
            .lines()
            .any(|line| line.contains("aws.") && line.contains("37..=49"))
    );
}

#[test]
fn review_r7_required_and_output_defaults_are_preserved_as_annotations() {
    for mode in ["required", "readOnlyProperties"] {
        let schema = ParsedCfnSchema::from_json(json!({
            "typeName":"AWS::Test::Emission",
            "properties":{"Probe":{"type":"integer","default":3}},
            mode:if mode == "required" { json!(["Probe"]) } else { json!(["/properties/Probe"]) }
        }))
        .unwrap();
        let rust = generate_synthetic_schema_code(&schema).unwrap();
        assert!(rust.contains("CFN default: 3"));
        assert!(!rust.contains(".with_default("));
        let markdown = generate_test_markdown(&schema, "AWS::EC2::Route").unwrap();
        assert!(markdown.contains("**Default:** `3`") && markdown.contains("server default"));
    }
}
