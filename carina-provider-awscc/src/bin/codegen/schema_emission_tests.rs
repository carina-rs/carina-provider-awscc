use super::*;
use serde_json::json;

fn emission_fixture(property: serde_json::Value) -> ParsedCfnSchema {
    ParsedCfnSchema::from_json(json!({
        "typeName":"AWS::Test::Emission", "properties":{"Probe":property}
    }))
    .unwrap()
}

#[test]
fn step4_unsupported_emission_returns_typed_errors() {
    for property in [
        json!(false),
        json!(true),
        json!({}),
        json!({"type":"null"}),
        json!({"maxLength":5}),
        json!({"type":"array"}),
        json!({"type":"array","items":{}}),
        json!({"oneOf":[{"type":"boolean"}]}),
        json!({"type":"object"}),
        json!({"type":"object","patternProperties":{".*":{"type":"integer"}}}),
        json!({"type":"object","additionalProperties":{"type":"integer"}}),
        json!({"type":["object","string"]}),
        json!({"type":"boolean","const":"true"}),
        json!({"type":"integer","enum":["one","two"]}),
        json!({"type":"string","const":1}),
        json!({"type":"array","items":{"type":"string"},"enum":["one","two"]}),
    ] {
        let schema = emission_fixture(property.clone());
        for (renderer, result) in [
            ("Rust", generate_synthetic_schema_code(&schema)),
            (
                "Markdown",
                generate_test_markdown(&schema, "AWS::EC2::Route"),
            ),
        ] {
            let error = result.expect_err(&format!("{renderer} accepted {property}"));
            assert!(
                matches!(error.downcast_ref::<schema_layer::SchemaError>(),
                    Some(schema_layer::SchemaError::Projection { pointer, reason: _ })
                    if pointer.starts_with("/properties/Probe")),
                "{renderer}: {property}: {error:#}"
            );
        }
    }
}

#[test]
fn step4_map_key_errors_are_typed_in_both_renderers() {
    let schema = emission_fixture(json!({
        "type":"object", "patternProperties":{"(?=x)x":{"type":"string"}},
        "additionalProperties":false
    }));
    for result in [
        generate_synthetic_schema_code(&schema),
        generate_test_markdown(&schema, "AWS::EC2::Route"),
    ] {
        let error = result.unwrap_err();
        assert!(
            matches!(error.downcast_ref::<schema_layer::SchemaError>(),
            Some(schema_layer::SchemaError::Projection { pointer, reason })
            if pointer == "/properties/Probe/patternProperties" && reason.contains("Invalid string pattern")),
            "{error:#}"
        );
    }
}

#[test]
fn step4_integer_override_errors_are_typed_in_both_renderers() {
    let mut schema = cfn_schema_for_codegen_tests("AWS::Scheduler::Schedule");
    schema.edit_json(|source| {
        source["definitions"]["FlexibleTimeWindow"]["properties"]["MaximumWindowInMinutes"] =
            json!({"type":"number"});
    });
    for result in [
        generate_schema_code(&schema, &schema.type_name),
        generate_test_markdown(&schema, &schema.type_name),
    ] {
        let error = result.unwrap_err();
        assert!(
            matches!(error.downcast_ref::<schema_layer::SchemaError>(),
            Some(schema_layer::SchemaError::Projection { pointer, reason })
            if pointer == "/definitions/FlexibleTimeWindow/properties/MaximumWindowInMinutes"
                && reason.contains("requires a CFN minimum or maximum")),
            "{error:#}"
        );
    }
}

#[test]
fn step4_renderers_share_alias_and_allof_projection() {
    let schema = ParsedCfnSchema::from_json(json!({
        "typeName":"AWS::Test::Emission", "properties":{
            "Probe":{"$ref":"#/definitions/Alias","maxItems":6}
        },
        "definitions":{
            "Alias":{"$ref":"#/definitions/Values","minItems":2},
            "Values":{"allOf":[
                {"type":"array","items":{"$ref":"#/definitions/Item"},"maxItems":8},
                {"insertionOrder":false,"uniqueItems":true}
            ]},
            "Item":{"allOf":[
                {"type":"object","properties":{
                    "Count":{"allOf":[{"type":"integer","minimum":1},{"maximum":9}]},
                    "Label":{"type":"string","description":"Declared label.","maxLength":7}
                }},
                {"required":["Count"]}
            ]}
        }
    }))
    .unwrap();
    let rust = generate_synthetic_schema_code(&schema).unwrap();
    let docs = generate_test_markdown(&schema, "AWS::EC2::Route").unwrap();
    assert!(
        rust.contains("false, Some((Some(2), Some(6))), carina_core::schema::ListElements::Unique"),
        "{rust}"
    );
    assert!(rust.contains("StructField::required(\"count\", AttributeType::refined_int(None, Some((Some(1), Some(9))))"), "{rust}");
    assert!(
        rust.contains(".with_description(\"Declared label.\")"),
        "{rust}"
    );
    assert!(docs.contains(" (items: 2..=6)"), "{docs}");
    assert!(docs.contains("| `count` | Int(1..=9) | Yes |"), "{docs}");
    assert!(
        docs.contains("| `label` | String(len: ..=7) | No | Declared label. |"),
        "{docs}"
    );
}

#[test]
fn step4_errors_keep_nested_reference_provenance() {
    let schema = ParsedCfnSchema::from_json(json!({
        "typeName":"AWS::Test::Emission",
        "properties":{"Probe":{"$ref":"#/definitions/Container"}},
        "definitions":{"Container":{"type":"object","properties":{"Unknown":{}}}}
    }))
    .unwrap();
    for result in [
        generate_synthetic_schema_code(&schema),
        generate_test_markdown(&schema, "AWS::EC2::Route"),
    ] {
        let error = result.unwrap_err();
        assert!(
            matches!(error.downcast_ref::<schema_layer::SchemaError>(),
            Some(schema_layer::SchemaError::Projection { pointer, reason: _ })
            if pointer == "/definitions/Container/properties/Unknown"),
            "{error:#}"
        );
    }
}

#[test]
fn step4_emission_ignores_legacy_tag_carrier_fields() {
    let schema = emission_fixture(json!({
        "type":"array", "minItems":2, "maxItems":4, "insertionOrder":false,
        "uniqueItems":true, "items":{"type":"integer","minimum":3,"maximum":9}
    }));
    let context = EmissionContext::from_source(&schema).unwrap();
    let mut property = schema.properties["Probe"].clone();
    property.prop_type = Some(TypeValue::Single("string".to_string()));
    property.ref_path = Some("#/definitions/DoesNotExist".to_string());
    property.items = None;
    property.min_items = Some(100);
    property.max_items = Some(200);
    property.insertion_order = Some(true);
    property.unique_items = Some(false);
    let (rust, _enum_info) = super::super::cfn_type_to_carina_type_with_enum(
        &property,
        "Probe",
        &context,
        "aws.test.Emission",
        &BTreeMap::new(),
    )
    .unwrap();
    let docs = super::super::type_display_string_at(
        &property,
        &context,
        &["Probe".into()],
        CfnPropertyPath::new(None, "Probe"),
    )
    .unwrap();
    assert!(rust.contains("Some((Some(3), Some(9)))"), "{rust}");
    assert!(
        rust.contains("false, Some((Some(2), Some(4))), carina_core::schema::ListElements::Unique"),
        "{rust}"
    );
    assert!(docs.contains("`List<Int(3..=9)>` (items: 2..=4)"), "{docs}");
}

#[test]
fn step4_description_and_default_use_site_annotation_projection() {
    let schema = ParsedCfnSchema::from_json(json!({
        "typeName":"AWS::Test::Emission", "properties":{
            "Local":{"$ref":"#/definitions/Value","description":"Local annotation.","default":7},
            "Inherited":{"$ref":"#/definitions/Value"}
        },
        "definitions":{"Value":{"type":"integer","description":"Target annotation.","default":7}}
    }))
    .unwrap();
    let rust = generate_synthetic_schema_code(&schema).unwrap();
    let docs = generate_test_markdown(&schema, "AWS::EC2::Route").unwrap();
    assert_eq!(rust.matches("Local annotation.").count(), 1, "{rust}");
    assert_eq!(docs.matches("Local annotation.").count(), 1, "{docs}");
    assert!(!rust.contains("Target annotation."), "{rust}");
    assert!(!docs.contains("Target annotation."), "{docs}");
    assert_eq!(
        rust.matches(".with_default(Value::Concrete(ConcreteValue::Int(7)))")
            .count(),
        1,
        "{rust}"
    );
    assert_eq!(docs.matches("**Default:** `7`").count(), 1, "{docs}");
}

#[test]
fn step4_recursive_array_emission_returns_typed_error() {
    for definitions in [
        json!({"Loop":{"type":"array","items":{"$ref":"#/definitions/Loop"}}}),
        json!({
            "Loop":{"type":"array","items":{"$ref":"#/definitions/Other"}},
            "Other":{"type":"array","items":{"$ref":"#/definitions/Loop"}}
        }),
    ] {
        let schema = ParsedCfnSchema::from_json(json!({
            "typeName":"AWS::Test::Emission",
            "properties":{"Probe":{"$ref":"#/definitions/Loop"}},
            "definitions":definitions
        }))
        .unwrap();
        // Check preflight first so a regression fails without starting an
        // unbounded renderer recursion (which can abort the test process).
        let error = schema_projection::validate_shared_queries(&schema).unwrap_err();
        assert!(
            matches!(error.downcast_ref::<schema_layer::SchemaError>(),
            Some(schema_layer::SchemaError::Projection { pointer: _, reason })
            if reason.contains("recursive array projection")),
            "{error:#}"
        );
        for result in [
            generate_synthetic_schema_code(&schema),
            generate_test_markdown(&schema, "AWS::EC2::Route"),
        ] {
            let error = result.unwrap_err();
            assert!(
                error.downcast_ref::<schema_layer::SchemaError>().is_some(),
                "{error:#}"
            );
        }
    }
}

#[test]
fn step4_named_struct_recursion_remains_supported() {
    let schema = ParsedCfnSchema::from_json(json!({
        "typeName":"AWS::Test::Emission",
        "properties":{"Probe":{"$ref":"#/definitions/Node"}},
        "definitions":{"Node":{"type":"object","properties":{
            "Children":{"type":"array","items":{"$ref":"#/definitions/Node"}}
        }}}
    }))
    .unwrap();
    let rust = generate_synthetic_schema_code(&schema).unwrap();
    let docs = generate_test_markdown(&schema, "AWS::EC2::Route").unwrap();
    assert!(
        rust.contains("AttributeType::ref_(\"Node\".to_string())"),
        "{rust}"
    );
    assert!(docs.contains("[Struct(Node)]"), "{docs}");
}
