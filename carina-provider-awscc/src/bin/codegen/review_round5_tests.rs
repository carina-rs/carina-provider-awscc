use super::*;
use schema_layer::{JsonType, SchemaError};
use schema_projection::ProjectedType;
use serde_json::json;

#[test]
fn review_r11_properties_only_shape_does_not_assert_an_object_type() {
    let fields = json!({"properties":{"Text":{"type":"string"}}});
    for property in [
        fields.clone(),
        json!({"allOf":[fields.clone()]}),
        json!({"$ref":"#/definitions/Fields"}),
    ] {
        let schema = ParsedCfnSchema::from_json(json!({
            "typeName":"AWS::Test::Projection",
            "properties":{"Probe":property},
            "definitions":{"Fields":fields}
        }))
        .unwrap();
        let projection = schema.properties["Probe"].try_query().unwrap();
        assert!(
            projection.types.is_empty(),
            "{property}: {:?}",
            projection.types
        );
        assert_eq!(
            projection.projected_type,
            ProjectedType::Value(JsonType::Object)
        );
        assert!(projection.properties.as_ref().unwrap().contains_key("Text"));
    }
}

#[test]
fn review_r11_properties_only_conjuncts_preserve_the_string_domain() {
    let mut failures = Vec::new();
    let fields = json!({"properties":{"Text":{"type":"string"}}});
    for types in [
        json!(["object", "string"]),
        json!(["string", "object"]),
        json!(["null", "string", "object"]),
    ] {
        let document = json!({"type":types});
        for property in [
            json!({"type":types,"allOf":[fields.clone()]}),
            json!({"allOf":[document.clone(),fields.clone()]}),
            json!({"allOf":[fields.clone(),document.clone()]}),
            json!({"$ref":"#/definitions/Document","allOf":[fields.clone()]}),
            json!({"type":types,"allOf":[{"$ref":"#/definitions/Fields"}]}),
            json!({"$ref":"#/definitions/Fields","type":types}),
            json!({"$ref":"#/definitions/Alias","allOf":[document.clone()]}),
            json!({"properties":fields["properties"],"allOf":[document.clone()]}),
        ] {
            let mut source = cfn_schema_for_codegen_tests("AWS::EC2::VPC").source;
            source["properties"]["Probe"] = property.clone();
            source["definitions"]["Document"] = document.clone();
            source["definitions"]["Fields"] = fields.clone();
            source["definitions"]["Alias"] = json!({"$ref":"#/definitions/Fields"});
            let schema = ParsedCfnSchema::from_json(source).unwrap();
            let projection = schema.properties["Probe"].try_query().unwrap();
            if projection.projected_type != ProjectedType::JsonDocument
                || !projection.types.contains(&JsonType::String)
                || !projection.types.contains(&JsonType::Object)
                || projection.types.contains(&JsonType::Null)
                    != types.as_array().unwrap().contains(&json!("null"))
            {
                failures.push(format!(
                    "{property} narrowed {types} to {:?}",
                    projection.types
                ));
            }
            for (renderer, output) in [
                ("Rust", generate_schema_code(&schema, &schema.type_name)),
                (
                    "Markdown",
                    generate_test_markdown(&schema, &schema.type_name),
                ),
            ] {
                if !output.as_ref().is_err_and(|error| {
                    matches!(error.downcast_ref::<SchemaError>(),
                        Some(SchemaError::Projection { pointer, reason })
                            if pointer == "/properties/Probe"
                                && reason.contains("JSON document")
                                && reason.contains("properties"))
                }) {
                    failures.push(format!(
                        "{renderer} must reject {property} with a typed JSON document properties error: {}",
                        output.err().map_or_else(|| "successful emission".into(), |error| format!("{error:#}"))
                    ));
                }
            }
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn review_r11_object_alternatives_keep_inference_out_of_conjunctive_types() {
    let fields = json!({"A":{"type":"string"},"B":{"type":"string"}});
    let typed = json!({"type":"object","properties":fields,"required":["A"]});
    let inferred = json!({"properties":fields,"required":["B"]});
    for keyword in ["oneOf", "anyOf"] {
        for arms in [
            [typed.clone(), inferred.clone()],
            [inferred.clone(), typed.clone()],
        ] {
            for conjunctive in [false, true] {
                let alternative = json!({keyword:arms});
                let property = if conjunctive {
                    json!({"type":["object","string"],"allOf":[alternative]})
                } else {
                    alternative
                };
                let schema = ParsedCfnSchema::from_json(json!({
                    "typeName":"AWS::Test::Projection", "properties":{"Probe":property}
                }))
                .unwrap();
                let projection = schema.properties["Probe"].try_query().unwrap();
                if conjunctive {
                    assert_eq!(projection.types, [JsonType::Object, JsonType::String]);
                    assert_eq!(projection.projected_type, ProjectedType::JsonDocument);
                    assert!(matches!(emission_plan::lower(projection),
                        Err(SchemaError::Projection { pointer, reason })
                            if pointer == "/properties/Probe"
                                && reason.contains("JSON document")
                                && reason.contains("properties")));
                } else {
                    assert!(projection.types.is_empty());
                    assert_eq!(
                        projection.projected_type,
                        ProjectedType::Value(JsonType::Object)
                    );
                    emission_plan::lower(projection).unwrap();
                }
            }
        }
    }
}

fn assert_identity_item_cycles_rejected(render: fn(&ParsedCfnSchema, &str) -> Result<String>) {
    for definitions in [
        json!({"Loop":{"type":"array","items":{"$ref":"#/definitions/Loop"}}}),
        json!({
            "Loop":{"type":"array","items":{"$ref":"#/definitions/Other"}},
            "Other":{"type":"array","items":{"$ref":"#/definitions/Loop"}}
        }),
        json!({
            "Loop":{"type":"array","items":{"$ref":"#/definitions/Alias"}},
            "Alias":{"allOf":[{"$ref":"#/definitions/Loop"}]}
        }),
    ] {
        let mut source = cfn_schema_for_codegen_tests("AWS::CloudWatch::Alarm").source;
        source["properties"]["AlarmActions"]["items"] = json!({"$ref":"#/definitions/Loop"});
        for (name, definition) in definitions.as_object().unwrap() {
            source["definitions"][name] = definition.clone();
        }
        let schema = ParsedCfnSchema::from_json(source).unwrap();
        let error = render(&schema, &schema.type_name).unwrap_err();
        assert!(
            matches!(error.downcast_ref::<SchemaError>(),
                Some(SchemaError::Projection { pointer, reason })
                    if pointer == "/properties/AlarmActions" && reason.contains("recursive")),
            "{error:#}"
        );
    }
}

#[test]
fn review_r11_identity_item_cycles_return_typed_errors_in_rust() {
    assert_identity_item_cycles_rejected(generate_schema_code);
}

#[test]
fn review_r11_identity_item_cycles_return_typed_errors_in_markdown() {
    assert_identity_item_cycles_rejected(generate_test_markdown);
}
