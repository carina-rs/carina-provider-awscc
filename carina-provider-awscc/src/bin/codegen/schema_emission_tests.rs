use super::*;
use serde_json::json;

fn emission_fixture(property: serde_json::Value) -> ParsedCfnSchema {
    ParsedCfnSchema::from_json(json!({
        "typeName":"AWS::Test::Emission", "properties":{"Probe":property}
    }))
    .unwrap()
}

fn round3_projection_error(schema: &ParsedCfnSchema, construct: &str) {
    for result in [generate_synthetic_schema_code(schema), generate_test_markdown(schema, "AWS::EC2::Route")] {
        let error = result.expect_err("an unpreserved constraint must be a typed error");
        assert!(matches!(error.downcast_ref::<schema_layer::SchemaError>(),
            Some(schema_layer::SchemaError::Projection { pointer, reason })
            if pointer.starts_with("/properties/Probe") && reason.contains(construct)), "{error:#}");
    }
}

#[test]
fn round3_allof_additional_property_domains_are_typed_errors() {
    let domain = json!({"type":"object","additionalProperties":{"type":"integer"}});
    let fields = json!({"properties":{"A":{"type":"boolean"}}});
    for arms in [json!([domain, fields]), json!([fields, domain])] {
        round3_projection_error(&emission_fixture(json!({"allOf":arms})), "additionalProperties");
    }
    let schema = ParsedCfnSchema::from_json(json!({
        "typeName":"AWS::Test::Emission", "properties":{"Probe":{
            "$ref":"#/definitions/Map", "properties":{"A":{"type":"boolean"}}
        }}, "definitions":{"Map":domain}
    })).unwrap();
    round3_projection_error(&schema, "additionalProperties");
}

#[test]
fn round3_explicit_nonobject_types_with_properties_are_typed_errors() {
    for kind in ["string", "integer", "boolean", "array"] {
        let mut property = json!({"type":kind,"properties":{"A":{"type":"boolean"}}});
        if kind == "array" { property["items"] = json!({"type":"string"}); }
        round3_projection_error(&emission_fixture(property), "properties");
    }
}

#[test]
fn round3_anyof_branch_field_domains_are_typed_errors() {
    let a = json!({"type":"object","properties":{"A":{"type":"integer"}},"required":["A"]});
    let b = json!({"type":"object","properties":{"B":{"type":"boolean"}},"required":["B"]});
    for keyword in ["anyOf", "oneOf"] {
        for arms in [json!([a,b]), json!([b,a])] {
            round3_projection_error(&emission_fixture(json!({keyword:arms})), "field domain");
        }
    }
}

#[test]
fn round3_literal_intersections_filter_every_effective_constraint() {
    for (literal, constraint, expected) in [
        (json!({"type":"integer","enum":[1,2]}), json!({"minimum":2}), json!([2])),
        (json!({"type":"integer","enum":[1,2,3]}), json!({"exclusiveMinimum":1,"exclusiveMaximum":3}), json!([2])),
        (json!({"type":"integer","enum":[1,2,3,4]}), json!({"multipleOf":2}), json!([2,4])),
        (json!({"type":"integer","enum":[1,2,3]}), json!({"multipleOf":0.3}), json!([3])),
        (json!({"type":"string","enum":["a","bb","ccc"]}), json!({"minLength":2,"maxLength":2}), json!(["bb"])),
        (json!({"type":"string","enum":["ab","bc"]}), json!({"pattern":"^a"}), json!(["ab"])),
    ] {
        for arms in [json!([literal,constraint]), json!([constraint,literal])] {
            let schema = emission_fixture(json!({"allOf":arms}));
            let rust = generate_synthetic_schema_code(&schema).unwrap();
            assert_eq!(serde_json::to_value(&schema.properties["Probe"].query().enum_values).unwrap(), expected);
            let markdown = generate_test_markdown(&schema, "AWS::EC2::Route").unwrap();
            if expected == json!([2]) {
                assert!(rust.contains("Some((Some(2), Some(2)))"), "{rust}");
                assert!(!rust.contains("Some((Some(1), Some(1)))"), "{rust}");
                assert!(markdown.contains("IntEnum([2])"), "{markdown}");
            }
        }
    }
}

#[test]
fn round3_literal_patterns_use_the_schema_dialect() {
    let schema = emission_fixture(json!({"type":"string","enum":["1","１"],"pattern":r"^\d+$"}));
    generate_synthetic_schema_code(&schema).unwrap();
    assert_eq!(schema.properties["Probe"].query().enum_values, Some(vec![json!("1")]));
    let markdown = generate_test_markdown(&schema, "AWS::EC2::Route").unwrap();
    assert!(!markdown.contains('１'), "{markdown}");
}

#[test]
fn round3_map_scalar_constraints_and_bounds_survive_nested_wrappers() {
    let map = json!({"type":"object","patternProperties":{"^[a-z]+$":{"type":"string","enum":["alpha","beta"]}},
        "additionalProperties":false,"minProperties":2,"maxProperties":5});
    for property in [map.clone(), json!({"type":"array","items":map}), json!({"type":"object","properties":{"Entries":map}})] {
        let schema = emission_fixture(property);
        let rust = generate_synthetic_schema_code(&schema).unwrap();
        assert!(rust.contains("AttributeType::refined_map(") && rust.contains("Some((Some(2), Some(5)))"), "{rust}");
        assert!(rust.contains("\"alpha\".to_string(), \"beta\".to_string()"), "{rust}");
        let markdown = generate_test_markdown(&schema, "AWS::EC2::Route").unwrap();
        assert!(markdown.contains("(entries: 2..=5)"), "{markdown}");
    }
}

#[test]
fn round3_empty_literal_intersections_are_typed_errors() {
    for property in [
        json!({"allOf":[{"type":"integer","const":1},{"minimum":2}]}),
        json!({"type":"integer","enum":[1,2],"minimum":3}),
        json!({"type":"string","const":"a","minLength":2}),
        json!({"type":"string","enum":["a","b"],"pattern":"^c$"}),
    ] {
        round3_projection_error(&emission_fixture(property), "literal intersection");
    }
}

#[test]
fn round3_property_dependencies_reach_markdown_without_field_groups() {
    for root in [false, true] {
        let object = json!({"type":"object","properties":{
            "Trigger":{"type":"integer"},"Needed":{"type":"boolean"}
        },"dependencies":{"Trigger":["Needed"]}});
        let source = if root {
            json!({"typeName":"AWS::Test::Emission","properties":object["properties"],"dependencies":object["dependencies"]})
        } else {
            json!({"typeName":"AWS::Test::Emission","properties":{"Probe":object}})
        };
        let schema = ParsedCfnSchema::from_json(source).unwrap();
        let rust = generate_synthetic_schema_code(&schema).unwrap();
        assert!(!rust.contains("FieldGroup::"), "{rust}");
        let markdown = generate_test_markdown(&schema, "AWS::EC2::Route").unwrap();
        assert!(markdown.contains("`trigger` requires `needed`"), "{markdown}");
    }
}

#[test]
fn round3_contains_is_a_typed_error_in_both_renderers() {
    for contains in [json!({"type":"integer","minimum":1}), json!({"type":"integer","maximum":0})] {
        round3_projection_error(&emission_fixture(json!({
            "type":"array","items":{"type":"integer"},"contains":contains
        })), "contains");
    }
}

#[test]
fn round3_ordinary_map_bounds_reach_rust_and_markdown() {
    for value in [json!({"type":"string"}), json!({"type":"object","properties":{"Text":{"type":"string"}}})] {
        let schema = emission_fixture(json!({"type":"object","patternProperties":{"^[a-z]+$":value},
            "additionalProperties":false,"minProperties":2,"maxProperties":5}));
        let rust = generate_synthetic_schema_code(&schema).unwrap();
        assert!(rust.contains("AttributeType::refined_map(") && rust.contains("Some((Some(2), Some(5)))"), "{rust}");
        let markdown = generate_test_markdown(&schema, "AWS::EC2::Route").unwrap();
        assert!(markdown.contains("(entries: 2..=5)"), "{markdown}");
    }
}

#[test]
fn round3_direct_string_map_values_use_shared_scalar_emission() {
    for (value, rust_fragment, markdown_fragment) in [
        (json!({"type":"string","minLength":2,"maxLength":5,"pattern":"^[a-z]+$"}), "AttributeType::refined_string(", "String(pattern, len: 2..=5)"),
        (json!({"type":"string","enum":["alpha","beta"]}), "\"alpha\".to_string(), \"beta\".to_string()", "Enum (Probe)"),
        (json!({"type":"string","const":"fixed"}), "\"fixed\".to_string()", "Enum (Probe)"),
    ] {
        let schema = emission_fixture(json!({"type":"object","patternProperties":{".*":value},"additionalProperties":false}));
        let rust = generate_synthetic_schema_code(&schema).unwrap();
        assert!(rust.contains(rust_fragment), "{rust}");
        let markdown = generate_test_markdown(&schema, "AWS::EC2::Route").unwrap();
        assert!(markdown.contains(markdown_fragment), "{markdown}");
    }
}

#[test]
fn round3_exclusive_integer_bounds_are_exact() {
    for (property, range, display) in [
        (json!({"type":"integer","exclusiveMinimum":0}), "Some((Some(1), None))", "Int(1..)"),
        (json!({"type":"integer","exclusiveMaximum":0}), "Some((None, Some(-1)))", "Int(..=-1)"),
        (json!({"type":"integer","minimum":3,"exclusiveMinimum":0,"maximum":9,"exclusiveMaximum":8}), "Some((Some(3), Some(7)))", "Int(3..=7)"),
        (json!({"type":"integer","exclusiveMinimum":9007199254740992_i64}), "Some((Some(9007199254740993), None))", "Int(9007199254740993..)"),
    ] {
        let schema = emission_fixture(property);
        let rust = generate_synthetic_schema_code(&schema).unwrap();
        assert!(rust.contains(range), "{rust}");
        let markdown = generate_test_markdown(&schema, "AWS::EC2::Route").unwrap();
        assert!(markdown.contains(display), "{markdown}");
    }
}

#[test]
fn round3_unrepresentable_exclusive_bounds_are_typed_errors() {
    for property in [
        json!({"type":"number","exclusiveMinimum":0}),
        json!({"type":"number","exclusiveMaximum":1}),
        json!({"type":"integer","exclusiveMinimum":i64::MAX}),
        json!({"type":"integer","exclusiveMaximum":i64::MIN}),
    ] {
        round3_projection_error(&emission_fixture(property), "exclusive");
    }
}

#[test]
fn round3_fractional_exclusive_integer_bounds_are_exact() {
    for (lower, upper, range) in [
        (json!(0.5), json!(3.5), "Some((Some(1), Some(3)))"),
        (json!(-3.5), json!(-0.5), "Some((Some(-3), Some(-1)))"),
        (json!(0.0), json!(3.0), "Some((Some(1), Some(2)))"),
        (json!(1e-200), json!(3), "Some((Some(1), Some(2)))"),
        (json!(-1e-200), json!(3), "Some((Some(0), Some(2)))"),
    ] {
        let schema = emission_fixture(json!({"type":"integer","exclusiveMinimum":lower,"exclusiveMaximum":upper}));
        let rust = generate_synthetic_schema_code(&schema).unwrap();
        assert!(rust.contains(range), "{rust}");
        generate_test_markdown(&schema, "AWS::EC2::Route").unwrap();
    }
}

#[test]
fn round2_closed_allof_domains_are_typed_errors() {
    let closed = json!({"type":"object","properties":{"A":{"type":"boolean"}},"additionalProperties":false});
    let added = json!({"properties":{"B":{"type":"boolean"}}});
    for arms in [json!([closed, added]), json!([added, closed])] {
        let schema = emission_fixture(json!({"allOf":arms}));
        for result in [generate_synthetic_schema_code(&schema), generate_test_markdown(&schema, "AWS::EC2::Route")] {
            let error = result.expect_err("a conjunct's closed property domain cannot be widened");
            assert!(matches!(error.downcast_ref::<schema_layer::SchemaError>(),
                Some(schema_layer::SchemaError::Projection { pointer, reason })
                if pointer == "/properties/Probe" && reason.contains("closed") && reason.contains("additionalProperties")), "{error:#}");
        }
    }
}

#[test]
fn round2_equivalent_exclusive_alternatives_are_typed_errors() {
    for arm in [
        json!({"type":"array","items":{"type":"string"}}),
        json!({"anyOf":[{"required":["A"]},{"required":["B"]}]}),
        json!({"anyOf":[{"required":["A"]},{"required":[]}]}),
        json!({"type":"string","maxLength":7}),
    ] {
        let mut property = json!({"oneOf":[arm.clone(),arm]});
        if arm.get("anyOf").is_some() {
            property["properties"] = json!({"A":{"type":"boolean"},"B":{"type":"boolean"}});
        }
        let schema = emission_fixture(property);
        for result in [generate_synthetic_schema_code(&schema), generate_test_markdown(&schema, "AWS::EC2::Route")] {
            let error = result.expect_err("equivalent exclusive arms overlap");
            assert!(matches!(error.downcast_ref::<schema_layer::SchemaError>(),
                Some(schema_layer::SchemaError::Projection { pointer, reason })
                if pointer == "/properties/Probe" && reason.contains("oneOf") && reason.contains("overlap")), "{error:#}");
        }
    }
}

#[test]
fn round2_closed_anyof_domains_are_typed_errors() {
    let schema = emission_fixture(json!({"anyOf":[
        {"type":"object","properties":{"A":{"type":"boolean"}},"required":["A"],"additionalProperties":false},
        {"type":"object","properties":{"B":{"type":"boolean"}},"required":["B"],"additionalProperties":false}
    ]}));
    for result in [generate_synthetic_schema_code(&schema), generate_test_markdown(&schema, "AWS::EC2::Route")] {
        let error = result.expect_err("closed anyOf arms cannot admit both fields together");
        assert!(matches!(error.downcast_ref::<schema_layer::SchemaError>(),
            Some(schema_layer::SchemaError::Projection { pointer, reason })
            if pointer == "/properties/Probe" && reason.contains("anyOf") && reason.contains("closed")), "{error:#}");
    }
}

#[test]
fn round2_overlapping_optional_oneof_arms_are_typed_errors() {
    for arms in [
        json!([
            {"properties":{"A":{"type":"boolean"}}},
            {"properties":{"B":{"type":"boolean"}}}
        ]),
        json!([
            {"anyOf":[{"required":["A"]},{"required":[]}]},
            {"anyOf":[{"required":["B"]},{"required":[]}]}
        ]),
    ] {
        let schema = emission_fixture(json!({"properties":{
            "A":{"type":"boolean"},"B":{"type":"boolean"}
        },"oneOf":arms}));
        for result in [generate_synthetic_schema_code(&schema), generate_test_markdown(&schema, "AWS::EC2::Route")] {
            let error = result.expect_err("the empty object matches both optional arms");
            assert!(matches!(error.downcast_ref::<schema_layer::SchemaError>(),
                Some(schema_layer::SchemaError::Projection { pointer, reason })
                if pointer == "/properties/Probe" && reason.contains("oneOf") && reason.contains("overlap")), "{error:#}");
        }
    }
}

#[test]
fn round2_schema_dependencies_are_typed_errors_in_both_renderers() {
    let dependency = json!({"type":"object","properties":{
        "A":{"type":"boolean"},"B":{"type":"boolean"}
    },"oneOf":[{"required":["A"]},{"required":["B"]}]});
    for root in [false, true] {
        let object = json!({"type":"object","properties":{"Enabled":{"type":"boolean"}},
            "dependencies":{"Enabled":dependency}});
        let source = if root {
            json!({"typeName":"AWS::Test::Emission","properties":object["properties"],"dependencies":object["dependencies"]})
        } else {
            json!({"typeName":"AWS::Test::Emission","properties":{"Probe":object}})
        };
        let schema = ParsedCfnSchema::from_json(source).unwrap();
        for result in [generate_synthetic_schema_code(&schema), generate_test_markdown(&schema, "AWS::EC2::Route")] {
            let error = result.expect_err("conditional schema constraints need an emission feature");
            assert!(matches!(error.downcast_ref::<schema_layer::SchemaError>(),
                Some(schema_layer::SchemaError::Projection { pointer, reason })
                if pointer == if root { "/dependencies/Enabled" } else { "/properties/Probe/dependencies/Enabled" }
                    && reason == &format!("schema dependencies on {} cannot be emitted", if root { "root" } else { "struct" })), "{error:#}");
        }
    }
}

#[test]
fn round2_resolved_scalar_alternatives_reach_both_renderers() {
    for keyword in ["oneOf", "anyOf"] {
        for (arm, rust_type, display) in [
            (json!({"type":"string","minLength":2,"maxLength":7}), "AttributeType::refined_string(", "String(len: 2..=7)"),
            (json!({"type":"integer","minimum":2,"maximum":7}), "AttributeType::refined_int(", "Int(2..=7)"),
            (json!({"type":"number","minimum":2,"maximum":7}), "AttributeType::refined_float(", "Float(2..=7)"),
            (json!({"type":"boolean","description":"Selected scalar."}), "AttributeType::bool()", "Bool"),
        ] {
            let bare = if arm["type"] == "boolean" { "string" } else { "boolean" };
            for arms in [json!([arm,{"type":bare}]), json!([{"type":bare},arm])] {
                let schema = emission_fixture(json!({keyword:arms}));
                let rust = generate_synthetic_schema_code(&schema).unwrap();
                assert!(rust.contains(rust_type), "{rust}");
                if arm["type"] == "number" {
                    assert!(rust.contains("FiniteF64::new(2.0)") && rust.contains("FiniteF64::new(7.0)"), "{rust}");
                } else if bare == "boolean" {
                    assert!(rust.contains("Some((Some(2), Some(7)))"), "{rust}");
                }
                let markdown = generate_test_markdown(&schema, "AWS::EC2::Route").unwrap();
                assert!(markdown.contains(display), "{markdown}");
            }
        }
    }
}

#[test]
fn round1_root_composition_contributions_are_typed_errors() {
    for contribution in [
        json!({"properties":{"Tags":{"maxItems":3}}}),
        json!({"properties":{"Added":{"type":"string"}}}),
    ] {
        let schema = ParsedCfnSchema::from_json(json!({
            "typeName":"AWS::Test::Emission", "properties":{
                "Tags":{"type":"array","items":{"type":"object","properties":{
                    "Key":{"type":"string"},"Value":{"type":"string"}
                }}}
            }, "allOf":[contribution]
        })).unwrap();
        for result in [generate_synthetic_schema_code(&schema), generate_test_markdown(&schema, "AWS::EC2::Route")] {
            let error = result.expect_err("resource composition must survive or fail explicitly");
            assert!(matches!(error.downcast_ref::<schema_layer::SchemaError>(),
                Some(schema_layer::SchemaError::Projection { pointer, reason })
                if pointer.is_empty() && reason.contains("resource composition")), "{error:#}");
        }
    }
}

#[test]
fn round1_float_string_bounds_are_typed_errors() {
    for keyword in ["minLength", "maxLength"] {
        for bound in [json!(2.0), json!(9007199254740992.0)] {
            for property in [
                json!({"type":"string", keyword:bound}),
                json!({"$ref":"#/definitions/Text", keyword:bound}),
                json!({"allOf":[{"type":"string"},{keyword:bound}]}),
            ] {
                let schema = ParsedCfnSchema::from_json(json!({
                    "typeName":"AWS::Test::Emission", "properties":{"Probe":property},
                    "definitions":{"Text":{"type":"string"}}
                })).unwrap();
                for result in [generate_synthetic_schema_code(&schema), generate_test_markdown(&schema, "AWS::EC2::Route")] {
                    let error = result.expect_err("floating point bounds cannot disappear");
                    assert!(matches!(error.downcast_ref::<schema_layer::SchemaError>(),
                        Some(schema_layer::SchemaError::Projection { pointer, reason })
                        if pointer.starts_with("/properties/Probe") && reason.contains(keyword)), "{error:#}");
                }
            }
        }
    }
}

#[test]
fn round1_reference_default_is_a_local_annotation() {
    let schema = ParsedCfnSchema::from_json(json!({
        "typeName":"AWS::Test::Emission", "properties":{
            "Local":{"$ref":"#/definitions/Count","default":2},
            "Unannotated":{"$ref":"#/definitions/Count"}
        }, "definitions":{"Count":{"type":"integer","default":1,"minimum":0,"maximum":9}}
    })).unwrap();
    let rust = generate_synthetic_schema_code(&schema).unwrap();
    assert_eq!(rust.matches(".with_default(Value::Concrete(ConcreteValue::Int(2)))").count(), 1, "{rust}");
    assert!(!rust.contains(".with_default(Value::Concrete(ConcreteValue::Int(1)))"), "{rust}");
    let markdown = generate_test_markdown(&schema, "AWS::EC2::Route").unwrap();
    assert!(markdown.contains("**Default:** `2`"), "{markdown}");
    assert!(!markdown.contains("**Default:** `1`"), "{markdown}");
}

#[test]
fn round1_definition_reuse_ignores_reference_site_annotations() {
    for annotated_first in [false, true] {
        let annotated = if annotated_first { "A" } else { "Z" };
        let unannotated = if annotated_first { "Z" } else { "A" };
        let schema = ParsedCfnSchema::from_json(json!({
            "typeName":"AWS::Test::Emission", "properties":{
                annotated:{"$ref":"#/definitions/Container", "default":{"Mode":"alpha"}, "description":"Local annotation."},
                unannotated:{"$ref":"#/definitions/Container"},
                "Other":{"$ref":"#/definitions/Container", "default":{"Mode":"beta"}}
            }, "definitions":{"Container":{
                "type":"object", "default":{"Mode":"canonical"}, "description":"Target annotation.",
                "properties":{"Mode":{"type":"string", "enum":["alpha","beta","canonical"]}}
            }}
        })).unwrap();
        for name in [annotated, unannotated, "Other"] {
            let reference = schema.properties[name].query().reference.as_ref().unwrap();
            assert_eq!(ResolvedRef {
                name: &reference.name, property: &reference.property, canonical: &reference.canonical
            }.emission_key(&schema).unwrap(), "Container", "{name}");
        }
        assert_eq!(schema.properties[annotated].annotation_default(), Some(&json!({"Mode":"alpha"})));
        assert_eq!(schema.properties["Other"].annotation_default(), Some(&json!({"Mode":"beta"})));
        assert_eq!(schema.properties[unannotated].annotation_default(), None);
        let rust = generate_synthetic_schema_code(&schema).unwrap();
        assert_eq!(rust.matches("schema.with_def(\"Container\",").count(), 1, "{rust}");
        assert!(!rust.contains("Container@"), "{rust}");
        assert_eq!(rust.matches("Local annotation.").count(), 1, "{rust}");
        let markdown = generate_test_markdown(&schema, "AWS::EC2::Route").unwrap();
        assert_eq!(markdown.matches("Local annotation.").count(), 1, "{markdown}");
    }
}

#[test]
fn round1_definition_identity_retains_descendant_constraints_and_defaults() {
    for contribution in [
        json!({"properties":{"Text":{"minLength":3}}}),
        json!({"required":["Text"]}),
        json!({"properties":{"Text":{"default":"local"}}}),
    ] {
        let schema = ParsedCfnSchema::from_json(json!({
            "typeName":"AWS::Test::Emission", "properties":{
                "Local":{"$ref":"#/definitions/Container", "properties":{"Child":contribution}},
                "Shared":{"$ref":"#/definitions/Container"}
            }, "definitions":{"Container":{"type":"object", "properties":{
                "Child":{"type":"object", "properties":{"Text":{"type":"string", "default":"canonical"}}}
            }}}
        })).unwrap();
        let key = |name: &str| {
            let reference = schema.properties[name].query().reference.as_ref().unwrap();
            ResolvedRef {
                name: &reference.name, property: &reference.property, canonical: &reference.canonical
            }.emission_key(&schema).unwrap().into_owned()
        };
        assert_ne!(key("Local"), key("Shared"), "{contribution}");
        assert_eq!(key("Shared"), "Container");
    }
}

#[test]
fn round1_definition_reuse_preserves_cloudfront_enum_registration() {
    let schema = cfn_schema_for_codegen_tests("AWS::CloudFront::Distribution");
    let rust = generate_schema_code(&schema, &schema.type_name).unwrap();
    for name in ["Cookies", "Restrictions", "ViewerCertificate"] {
        assert_eq!(rust.matches(&format!("schema.with_def(\"{name}\",")).count(), 1, "{name}");
        assert!(!rust.contains(&format!("{name}@")), "{name}");
    }
    for (field, constant) in [
        ("forward", "VALID_COOKIES_FORWARD"),
        ("minimum_protocol_version", "VALID_VIEWER_CERTIFICATE_MINIMUM_PROTOCOL_VERSION"),
    ] {
        assert!(rust.contains(&format!("const {constant}: &[&str]")), "{constant}");
        assert!(rust.contains(&format!("(\"{field}\", {constant})")), "{field}");
    }
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
    schema.edit_json(|source| { source["definitions"]["FlexibleTimeWindow"]["properties"]["MaximumWindowInMinutes"] = json!({"type":"number"}); });
    let code = generate_schema_code(&schema,&schema.type_name).unwrap();
    assert!(code.contains("StructField::optional(\"maximum_window_in_minutes\", AttributeType::float())"));
    generate_test_markdown(&schema,&schema.type_name).unwrap();
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
