use super::*;
use serde_json::{Value, json};

fn fixture(property: Value) -> ParsedCfnSchema {
    ParsedCfnSchema::from_json(json!({
        "typeName":"AWS::Test::Emission", "properties":{"Probe":property}
    }))
    .unwrap()
}

fn rejected(schema: &ParsedCfnSchema, fact: &str, kind: &str, pointer: &str) {
    for (renderer, result) in [
        ("Rust", generate_synthetic_schema_code(schema)),
        ("Markdown", generate_test_markdown(schema, "AWS::EC2::Route")),
    ] {
        let error = result.expect_err("an unpreserved fact must fail before emission");
        let projection = error.chain().find_map(|source| source.downcast_ref::<schema_layer::SchemaError>());
        assert!(matches!(projection,
            Some(schema_layer::SchemaError::Projection { pointer: actual, reason })
                if actual == pointer && reason.contains(fact) && reason.contains(kind)),
            "{renderer}: {error:#}");
    }
}

#[test]
fn exhaustive_one_of_number_integer_overlap_is_rejected() {
    for arms in [
        json!([{"type":"number"},{"type":"integer","minimum":1}]),
        json!([{"type":"integer","minimum":1},{"type":"number"}]),
    ] {
        rejected(&fixture(json!({"oneOf":arms})), "oneOf", "overlap", "/properties/Probe");
    }
}

#[test]
fn exhaustive_nonliteral_multiple_of_is_documented_without_rust_validation() {
    for kind in ["integer", "number"] {
        for divisor in [json!(2), json!(0.0001)] {
            let baseline = fixture(json!({"type":kind,"minimum":0,"maximum":100}));
            let baseline_rust = generate_synthetic_schema_code(&baseline).unwrap();
            for property in [
                json!({"type":kind,"minimum":0,"maximum":100,"multipleOf":divisor}),
                json!({"allOf":[{"type":kind,"minimum":0,"maximum":100},{"multipleOf":divisor}]}),
            ] {
                let schema = fixture(property);
                let plan = emission_plan::lower(schema.properties["Probe"].try_query().unwrap()).unwrap();
                assert_eq!(
                    plan.numeric.unwrap().multiple_of.unwrap().divisor,
                    divisor.as_number().unwrap()
                );
                let rust = generate_synthetic_schema_code(&schema).unwrap();
                assert_eq!(rust, baseline_rust, "multipleOf has no core validator: carina#4229");
                let markdown = generate_test_markdown(&schema, "AWS::EC2::Route").unwrap();
                assert!(markdown.contains(&format!("multiple of {divisor}")), "{markdown}");
            }
        }
    }
}

#[test]
fn exhaustive_closed_empty_objects_are_structs_in_both_renderers() {
    for closed in [
        json!({"type":"object","additionalProperties":false}),
        json!({"type":"object","properties":{},"additionalProperties":false}),
        json!({"type":"object","patternProperties":{},"additionalProperties":false}),
    ] {
        let schema = fixture(closed.clone());
        let plan = emission_plan::lower(schema.properties["Probe"].try_query().unwrap()).unwrap();
        assert_eq!(plan.kind, emission_plan::EmittedKind::Struct);
        for schema in [
            schema,
            ParsedCfnSchema::from_json(json!({
                "typeName":"AWS::Test::Emission",
                "properties":{"Probe":{"$ref":"#/definitions/Marker"}},
                "definitions":{"Marker":closed}
            })).unwrap(),
        ] {
            let rust = generate_synthetic_schema_code(&schema).unwrap();
            assert!(rust.contains("AttributeType::struct_(\"Probe\".to_string(), vec![])"), "{rust}");
            let markdown = generate_test_markdown(&schema, "AWS::EC2::Route").unwrap();
            assert!(markdown.contains("Empty struct (`{}`)"), "{markdown}");
        }
        for (property, display) in [
            (json!({"type":"object","properties":{"Marker":closed}}), "Empty struct (`{}`)"),
            (json!({"type":"array","items":closed}), "List<Empty struct ({})>"),
        ] {
            let schema = fixture(property);
            let rust = generate_synthetic_schema_code(&schema).unwrap();
            assert!(rust.contains("vec![]"), "{rust}");
            let markdown = generate_test_markdown(&schema, "AWS::EC2::Route").unwrap();
            assert!(markdown.contains(display), "{markdown}");
        }
    }
}

#[test]
fn exhaustive_struct_closedness_preserves_cached_absence_and_rejects_open_domains() {
    let fields = json!({"type":"object","properties":{"A":{"type":"string"}}});
    // Cached WAFv2 WebACL Regex omits the flag. Keep HEAD's closed struct
    // emission for this shape without calling the absent flag enforced.
    same_rendering(fields.clone(), "additionalProperties", json!(false));
    for (additional, fact) in [
        (json!(true), "additionalProperties:true"),
        (json!({"type":"string"}), "additionalProperties schema"),
    ] {
        let mut property = fields.clone();
        property["additionalProperties"] = additional;
        rejected(&fixture(property), fact, "struct", "/properties/Probe");
    }
}

#[test]
fn exhaustive_empty_struct_closedness_rejects_unknown_fields_at_validate() {
    use carina_core::resource::{ConcreteValue, Value};
    use carina_core::schema::{AttributeSchema, AttributeType, ResourceSchema, TypeError};

    let schema = ResourceSchema::new("test.Emission")
        .attribute(AttributeSchema::optional("probe", AttributeType::struct_("Probe", vec![])).build())
        .validate().unwrap();
    let attribute = schema.effective_attribute("probe").unwrap();
    let empty: Value = Value::Concrete(ConcreteValue::Map(indexmap::IndexMap::new()));
    attribute.validate(&empty).expect("an empty object satisfies the closed struct");
    let unknown: Value = Value::Concrete(ConcreteValue::Map(indexmap::IndexMap::from([
        ("unknown".into(), Value::Concrete(ConcreteValue::Bool(true)))
    ])));
    assert!(matches!(attribute.validate(&unknown),
        Err(TypeError::UnknownStructField { field, .. }) if field == "unknown"));
}

fn same_rendering(baseline: Value, keyword: &str, value: Value) {
    let mut annotated = baseline.clone();
    annotated[keyword] = value;
    let baseline = fixture(baseline);
    let annotated = fixture(annotated);
    for (renderer, before, after) in [
        ("Rust", generate_synthetic_schema_code(&baseline), generate_synthetic_schema_code(&annotated)),
        ("Markdown", generate_test_markdown(&baseline, "AWS::EC2::Route"), generate_test_markdown(&annotated, "AWS::EC2::Route")),
    ] {
        assert_eq!(before.unwrap(), after.unwrap(), "{renderer}: inapplicable {keyword}");
    }
}

#[test]
fn exhaustive_array_keywords_do_not_apply_to_nonarrays() {
    for property in [
        json!({"type":"string"}), json!({"type":"integer"}),
        json!({"type":"number"}), json!({"type":"boolean"}),
        json!({"type":"string","enum":["x"]}),
        json!({"type":"object","properties":{"A":{"type":"string"}}}),
        json!({"type":"object","patternProperties":{".*":{"type":"string"}},"additionalProperties":false}),
    ] {
        for (keyword, value) in [
            ("minItems", json!(1)), ("maxItems", json!(2)),
            ("uniqueItems", json!(true)), ("insertionOrder", json!(false)),
            ("arrayType", json!("AttributeList")),
            ("items", json!(false)), ("contains", json!(false)),
        ] {
            same_rendering(property.clone(), keyword, value);
        }
    }
}

#[test]
fn exhaustive_scalar_keywords_do_not_apply_to_other_kinds() {
    for property in [
        json!({"type":"string"}), json!({"type":"boolean"}),
        json!({"type":"object","properties":{"A":{"type":"string"}}}),
        json!({"type":"array","items":{"type":"string"}}),
        json!({"type":"array","uniqueItems":true,"items":{"type":"string"}}),
    ] {
        for (keyword, value) in [
            ("minimum", json!(1)), ("maximum", json!(2)),
            ("exclusiveMinimum", json!(1)), ("exclusiveMaximum", json!(2)),
            ("multipleOf", json!(2)),
        ] {
            same_rendering(property.clone(), keyword, value);
        }
    }
    for property in [
        json!({"type":"integer"}), json!({"type":"number"}),
        json!({"type":"boolean"}),
        json!({"type":"object","properties":{"A":{"type":"string"}}}),
        json!({"type":"array","items":{"type":"string"}}),
    ] {
        for (keyword, value) in [
            ("pattern", json!("^x$")), ("minLength", json!(1)), ("maxLength", json!(2)),
        ] {
            same_rendering(property.clone(), keyword, value);
        }
    }
}

#[test]
fn exhaustive_array_keywords_are_inapplicable_in_every_nonarray_arm() {
    use emission_plan::EmittedKind;
    for (property, kinds) in [
        (json!({"type":"string"}), vec![EmittedKind::String, EmittedKind::TagString]),
        (json!({"type":"integer"}), vec![EmittedKind::Integer]),
        (json!({"type":"number"}), vec![EmittedKind::Number]),
        (json!({"type":"boolean"}), vec![EmittedKind::Boolean]),
        (json!({"type":"string","enum":["x"]}), vec![EmittedKind::Literal]),
        (json!({"type":"object","properties":{"A":{"type":"string"}}}), vec![EmittedKind::Struct, EmittedKind::Root, EmittedKind::TagObject]),
        (json!({"type":"object","patternProperties":{".*":{"type":"string"}},"additionalProperties":false}), vec![EmittedKind::Map, EmittedKind::TagMap]),
        (json!({"type":"object"}), vec![EmittedKind::BareObject]),
        (json!({"type":["object","string"]}), vec![EmittedKind::JsonDocument]),
    ] {
        for (keyword, value) in [
            ("minItems", json!(1)), ("maxItems", json!(2)),
            ("uniqueItems", json!(true)), ("insertionOrder", json!(false)),
            ("arrayType", json!("AttributeList")),
            ("items", json!(false)), ("contains", json!(false)),
        ] {
            let mut annotated = property.clone();
            annotated[keyword] = value;
            let schema = fixture(annotated);
            let projection = schema.properties["Probe"].try_query().unwrap();
            for kind in &kinds {
                emission_plan::lower_as(projection, *kind).unwrap_or_else(|error| {
                    panic!("{kind:?}: inapplicable {keyword}: {error}")
                });
            }
        }
    }
}

#[test]
fn exhaustive_format_annotations_do_not_restrict_other_kinds() {
    for property in [
        json!({"type":"boolean"}),
        json!({"type":"object","properties":{"A":{"type":"string"}}}),
        json!({"type":"object","patternProperties":{".*":{"type":"string"}},"additionalProperties":false}),
        json!({"type":"array","items":{"type":"string"}}),
    ] {
        same_rendering(property, "format", json!("date-time"));
    }
}

#[test]
fn exhaustive_object_keywords_do_not_apply_to_nonobjects() {
    for property in [
        json!({"type":"string"}), json!({"type":"integer"}),
        json!({"type":"number"}), json!({"type":"boolean"}),
        json!({"type":"string","enum":["x"]}),
        json!({"type":"array","items":{"type":"string"}}),
    ] {
        for (keyword, value) in [
            ("minProperties", json!(1)), ("maxProperties", json!(2)),
            ("properties", json!({})),
            ("patternProperties", json!({".*":false})),
            ("additionalProperties", json!(false)),
            ("additionalProperties", json!({"type":"integer","multipleOf":2})),
            ("required", json!(["Ignored"])),
            ("dependencies", json!({"Ignored":["Other"]})),
            ("dependencies", json!({"Ignored":false})),
        ] {
            same_rendering(property.clone(), keyword, value);
        }
    }
}

#[test]
fn exhaustive_struct_property_count_is_rejected() {
    for bound in [json!({"minProperties":1}), json!({"maxProperties":1}), json!({"minProperties":1,"maxProperties":1})] {
        let mut property = json!({"type":"object","properties":{
            "A":{"type":"string"},"B":{"type":"string"}
        }});
        property.as_object_mut().unwrap().extend(bound.as_object().unwrap().clone());
        rejected(&fixture(property), "Properties", "struct", "/properties/Probe");
    }
}

#[test]
fn exhaustive_struct_pattern_property_overlap_is_rejected() {
    let fields = json!({"type":"object","properties":{"Apple":{"type":"string"}}});
    let patterns = json!({"patternProperties":{"^A":{"type":"string","minLength":5}}});
    let mut direct = fields.clone();
    direct.as_object_mut().unwrap().extend(patterns.as_object().unwrap().clone());
    for property in [direct, json!({"allOf":[fields,patterns]}), json!({"allOf":[patterns,fields]})] {
        rejected(&fixture(property), "patternProperties", "struct", "/properties/Probe");
    }
}

#[test]
fn exhaustive_pattern_map_additional_schema_is_rejected() {
    rejected(&fixture(json!({"type":"object","patternProperties":{"^A":{"type":"string"}},
        "additionalProperties":{"type":"string","minLength":5}})),
        "additionalProperties", "map", "/properties/Probe");
}

#[test]
fn exhaustive_map_required_keys_are_rejected() {
    rejected(&fixture(json!({"type":"object","patternProperties":{".*":{"type":"string"}},
        "additionalProperties":false,"required":["RequiredKey"]})),
        "required", "map", "/properties/Probe");
}

#[test]
fn exhaustive_tag_string_format_is_preserved_or_rejected() {
    for patterned in [false, true] {
        let mut value = json!({"type":"string","format":"date-time"});
        if patterned { value["pattern"] = json!("^x$"); }
        let schema = ParsedCfnSchema::from_json(json!({
            "typeName":"AWS::Test::Tags", "tagging":{"taggable":true,"tagProperty":"/properties/Tags"},
            "properties":{"Tags":{"type":"array","items":{"type":"object","properties":{
                "Key":{"type":"string"},"Value":value
            }}}}
        })).unwrap();
        if patterned {
            rejected(&schema, "date-time", "pattern", "/properties/Tags/items/properties/Value");
        } else {
            let rust = generate_synthetic_schema_code(&schema).unwrap();
            assert!(rust.contains("[Tt]") && rust.contains("AttributeType::refined_string("), "{rust}");
            let markdown = generate_test_markdown(&schema, "AWS::EC2::Route").unwrap();
            assert!(markdown.contains("String(date-time)"), "{markdown}");
        }
    }
}

#[test]
fn exhaustive_tag_candidate_includes_required_and_dependency_names() {
    for item in [
        json!({"type":"object","properties":{"Key":{"type":"string"}},"required":["Value"]}),
        json!({"type":"object","properties":{"Key":{"type":"string"}},"dependencies":{"Key":["Value"]}}),
        json!({"type":"object","properties":{"Value":{"type":"string"}},"dependencies":{"Key":["Other"]}}),
    ] {
        let schema = ParsedCfnSchema::from_json(json!({
            "typeName":"AWS::Test::Emission","properties":{"Probe":{"type":"array","items":item}}
        })).unwrap();
        assert!(schema.tag_shape(&schema.properties["Probe"]).unwrap().candidate());
        for result in [generate_synthetic_schema_code(&schema), generate_test_markdown(&schema, "AWS::EC2::Route")] {
            let error = result.expect_err("tag candidates require classification without tagging metadata");
            assert!(error.to_string().contains("unclassified tag-shaped"), "{error:#}");
        }
    }
}

#[test]
fn exhaustive_map_dependencies_reach_markdown() {
    let schema = fixture(json!({"type":"object","patternProperties":{".*":{"type":"string"}},
        "additionalProperties":false,"dependencies":{"A":["B"]}}));
    let rust = generate_synthetic_schema_code(&schema).unwrap();
    assert!(!rust.contains("FieldGroup::"), "{rust}");
    let markdown = generate_test_markdown(&schema, "AWS::EC2::Route").unwrap();
    assert!(markdown.contains("`A` requires `B`."), "{markdown}");
}
