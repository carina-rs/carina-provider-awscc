use super::schema_layer::{JsonType, Layer, SchemaError};
use super::schema_projection::{AlternativeKind, ProjectedType, Projection, project};
use serde_json::{Value, json};
use std::sync::Arc;

fn probe(property: Value, definitions: Value) -> Result<Projection, SchemaError> {
    let layer = Arc::new(Layer::from_json(&json!({
        "typeName":"AWS::Test::Projection", "properties":{"Probe":property},
        "definitions":definitions
    }))?);
    let node = layer.sources().node_at("/properties/Probe").unwrap();
    project(&layer, node)
}

#[test]
fn later_array_arm_constraints_are_not_discarded() {
    for keyword in ["oneOf", "anyOf"] {
        for later in [
            json!({"type":"array"}),
            json!({"type":"array","items":{"type":"string"},"minItems":2}),
            json!({"type":"array","items":{"type":"integer"}}),
            json!({"type":"array","items":{"type":"string"},"uniqueItems":true}),
            json!({"type":"array","items":{"type":"string"},"insertionOrder":false}),
            json!({"type":"object","minProperties":1}),
        ] {
            let property = json!({keyword:[{"type":"array","items":{"type":"string"}},later]});
            assert!(
                matches!(probe(property, json!({})), Err(SchemaError::Projection { pointer, reason: _ }) if pointer == "/properties/Probe")
            );
        }
    }
}

#[test]
fn intermediate_aliases_and_all_of_intersect_constraints() {
    let p = probe(json!({"$ref":"#/definitions/Alias", "maxItems":8}), json!({
        "Alias":{"$ref":"#/definitions/List","minItems":2,"uniqueItems":true},
        "List":{"allOf":[{"type":"array","items":{"type":"string","enum":["A","B"]},"maxItems":10,"insertionOrder":false},{"maxItems":6}]}
    })).unwrap();
    assert_eq!(
        (p.min_items, p.max_items, p.unique_items, p.insertion_order),
        (Some(2), Some(6), Some(true), Some(false))
    );
    assert_eq!(p.reference.as_ref().unwrap().name, "List");
    assert_eq!(
        p.items.as_ref().unwrap().query().enum_values,
        Some(vec![json!("A"), json!("B")])
    );
}

#[test]
fn scalar_constraints_survive_alias_siblings_and_all_of() {
    let p = probe(
        json!({"$ref":"#/definitions/Alias","maxLength":9}),
        json!({
            "Alias":{"$ref":"#/definitions/Value","minLength":3},
            "Value":{"allOf":[{"type":"string","enum":["A","ABCDE"]},{"maxLength":6}]}
        }),
    )
    .unwrap();
    assert_eq!((p.min_length, p.max_length), (Some(3), Some(6)));
    assert_eq!(p.reference.as_ref().unwrap().name, "Value");
    assert_eq!(p.enum_values, Some(vec![json!("ABCDE")]));
    assert!(matches!(
        probe(
            json!({"allOf":[{"minLength":9},{"maxLength":2}]}),
            json!({})
        ),
        Err(SchemaError::Projection {
            pointer: _,
            reason: _
        })
    ));
}

#[test]
fn dependencies_keep_their_conditional_scope_and_are_visited() {
    let p = probe(json!({"type":"object","dependencies":{
        "Names":["Other"],
        "Enabled":{"properties":{"Values":{"allOf":[{"type":"array","items":{"type":"string"}},{"minItems":2}]}}}
    }}),json!({})).unwrap();
    assert!(p.min_items.is_none());
    assert_eq!(p.dependencies.len(), 2);
    assert!(p.has_constrained_array());
}

#[test]
fn type_projection_is_validated_and_nullable_types_are_order_independent() {
    for types in [json!(["null", "string"]), json!(["string", "null"])] {
        assert_eq!(
            probe(json!({"type":types}), json!({}))
                .unwrap()
                .effective_type(),
            Some("string")
        );
    }
    for types in [json!(["integer", "string"]), json!(["string", "integer"])] {
        assert!(matches!(
            probe(json!({"type":types}), json!({})),
            Err(SchemaError::Projection {
                pointer: _,
                reason: _
            })
        ));
    }
}

#[test]
fn json_document_shape_is_typed_for_unlisted_resources_and_paths() {
    for resource in ["AWS::Test::Projection", "AWS::Other::Document"] {
        for spelling in [
            json!(["object", "string"]),
            json!(["string", "object"]),
            json!(["null", "string", "object"]),
            json!(["string", "object", "null"]),
        ] {
            let layer = Arc::new(
                Layer::from_json(&json!({
                    "typeName":resource,
                    "properties":{"Unlisted":{"$ref":"#/definitions/UnlistedDocument"}},
                    "definitions":{"UnlistedDocument":{"type":spelling}}
                }))
                .unwrap(),
            );
            for pointer in ["/properties/Unlisted", "/definitions/UnlistedDocument"] {
                let p = project(&layer, layer.sources().node_at(pointer).unwrap()).unwrap();
                assert_eq!(p.projected_type, ProjectedType::JsonDocument);
                assert_eq!(p.effective_type(), Some("object"));
            }
        }
    }
    let narrowed = probe(
        json!({"type":["object","string"],"allOf":[{"type":"object"}]}),
        json!({}),
    )
    .unwrap();
    assert_eq!(
        narrowed.projected_type,
        ProjectedType::Value(JsonType::Object)
    );
}

#[test]
fn every_other_multiple_nonnull_type_shape_is_rejected() {
    let types = ["boolean", "object", "array", "number", "integer", "string"];
    for first in types {
        for second in types {
            if first == second
                || [first, second] == ["object", "string"]
                || [first, second] == ["string", "object"]
            {
                continue;
            }
            for spelling in [json!([first, second]), json!(["null", first, second])] {
                assert!(matches!(
                    probe(json!({"type":spelling}), json!({})),
                    Err(SchemaError::Projection { pointer, reason: _ })
                        if pointer == "/properties/Probe"
                ));
            }
        }
    }
    assert!(matches!(
        probe(json!({"type":["object","string","array"]}), json!({})),
        Err(SchemaError::Projection {
            pointer: _,
            reason: _
        })
    ));
}

#[test]
fn mixed_types_select_one_constrained_arm_and_record_every_bare_arm() {
    for resource in ["AWS::Test::Projection", "AWS::Other::List"] {
        for (keyword, kind) in [
            ("oneOf", AlternativeKind::OneOf),
            ("anyOf", AlternativeKind::AnyOf),
        ] {
            for selected in 0..3 {
                let mut arms = vec![json!({"type":"object"}), json!({"type":"boolean"})];
                arms.insert(selected, json!({"$ref":"#/definitions/Values"}));
                let layer = Arc::new(
                    Layer::from_json(&json!({
                        "typeName":resource,
                        "properties":{"Unlisted":{"$ref":"#/definitions/Alias"}},
                        "definitions":{
                            "Alias":{"allOf":[{"$ref":"#/definitions/Choice"},{"maxItems":9}]},
                            "Choice":{keyword:arms},
                            "Values":{"type":"array","items":{"type":"string"},"minItems":2}
                        }
                    }))
                    .unwrap(),
                );
                for pointer in [
                    "/properties/Unlisted",
                    "/definitions/Alias",
                    "/definitions/Choice",
                ] {
                    let p = project(&layer, layer.sources().node_at(pointer).unwrap()).unwrap();
                    assert_eq!(p.projected_type, ProjectedType::Value(JsonType::Array));
                    assert_eq!(p.min_items, Some(2));
                    assert_eq!(
                        p.items.as_ref().unwrap().query().effective_type(),
                        Some("string")
                    );
                    assert_eq!(p.dropped_bare_alternatives.len(), 2);
                    let bare_arms = (0..3)
                        .filter(|index| *index != selected)
                        .zip([JsonType::Object, JsonType::Boolean]);
                    for (dropped, (index, expected_type)) in
                        p.dropped_bare_alternatives.iter().zip(bare_arms)
                    {
                        assert_eq!(dropped.kind, kind);
                        assert_eq!(dropped.json_type, expected_type);
                        assert_eq!(
                            layer.sources().pointer(dropped.arm),
                            format!("/definitions/Choice/{keyword}/{index}")
                        );
                    }
                }
            }
        }
    }
}

#[test]
fn mixed_type_policy_applies_to_constrained_objects_and_scalars_too() {
    for constrained in [
        json!({"type":"object","properties":{"Value":{"type":"string"}},"required":["Value"]}),
        json!({"type":"string","maxLength":7}),
        json!({"type":"integer","minimum":2}),
    ] {
        for keyword in ["oneOf", "anyOf"] {
            for arms in [
                json!([constrained.clone(), {"type":"array"}]),
                json!([{"type":"array"}, constrained.clone()]),
            ] {
                let p = probe(json!({keyword:arms}), json!({})).unwrap();
                let expected = probe(constrained.clone(), json!({})).unwrap();
                assert_eq!(p.projected_type, expected.projected_type);
                assert_eq!(p.required, expected.required);
                assert_eq!(p.max_length, expected.max_length);
                assert_eq!(p.minimum, expected.minimum);
                assert_eq!(p.dropped_bare_alternatives.len(), 1);
                assert_eq!(p.dropped_bare_alternatives[0].json_type, JsonType::Array);
            }
        }
    }
}

#[test]
fn mixed_type_policy_preserves_the_selected_reference_identity() {
    for constrained in [
        json!({"type":"object","properties":{"Value":{"type":"string"}}}),
        json!({"type":"array","items":{"type":"string"}}),
        json!({"type":"string","enum":["A","B"]}),
    ] {
        for keyword in ["oneOf", "anyOf"] {
            for arms in [
                json!([{"$ref":"#/definitions/Selected"},{"type":"boolean"}]),
                json!([{"type":"boolean"},{"$ref":"#/definitions/Selected"}]),
            ] {
                let p = probe(json!({keyword:arms}), json!({"Selected":constrained})).unwrap();
                let reference = p.reference.as_ref().unwrap();
                assert_eq!(reference.name, "Selected");
                assert_eq!(reference.property.query().projected_type, p.projected_type);
            }
        }
    }
}

#[test]
fn mixed_types_reject_zero_or_multiple_constrained_arms_in_either_order() {
    for keyword in ["oneOf", "anyOf"] {
        for arms in [
            json!([{"type":"array"},{"type":"object"}]),
            json!([{"type":"array","items":{"type":"string"}},{"type":"object","minProperties":1}]),
            json!([{"type":"array","items":{"type":"string"}},{"type":"array","items":{"type":"string"}},{"type":"object"}]),
        ] {
            let mut arms = arms.as_array().unwrap().clone();
            for _order in 0..2 {
                assert!(matches!(
                    probe(json!({keyword:arms}), json!({})),
                    Err(SchemaError::Projection { pointer, reason: _ })
                        if pointer == "/properties/Probe"
                ));
                arms.reverse();
            }
        }
    }
}

#[test]
fn mixed_types_never_drop_reference_or_composition_arms_as_bare() {
    for extra in [
        json!({"$ref":"#/definitions/Empty"}),
        json!({"allOf":[{"type":"object"}]}),
        json!({"properties":{}}),
        json!({"items":{}}),
        json!({"required":[]}),
        json!({"additionalProperties":true}),
        json!({"description":"An annotated object"}),
    ] {
        let mut object = json!({"type":"object"});
        object
            .as_object_mut()
            .unwrap()
            .extend(extra.as_object().unwrap().clone());
        for keyword in ["oneOf", "anyOf"] {
            let mut arms = vec![
                json!({"type":"array","items":{"type":"string"}}),
                object.clone(),
            ];
            for _order in 0..2 {
                assert!(
                    matches!(
                        probe(json!({keyword:arms}), json!({"Empty":{"type":"object"}})),
                        Err(SchemaError::Projection {
                            pointer: _,
                            reason: _
                        })
                    ),
                    "discarded {extra}"
                );
                arms.reverse();
            }
        }
    }
}

#[test]
fn object_field_alternatives_reject_branch_only_domains_with_bare_arms() {
    for keyword in ["oneOf", "anyOf"] {
        let mut arms = vec![
            json!({"type":"object","properties":{"Selected":{"type":"string"}},"required":["Selected"]}),
            json!({"type":"object"}),
        ];
        for _order in [0, 1] {
            let result = probe(json!({keyword:arms}), json!({}));
            if keyword == "oneOf" {
                assert!(
                    matches!(result, Err(SchemaError::Projection { pointer: _pointer, reason }) if reason.contains("unsupported required alternatives"))
                );
            } else {
                assert!(
                    matches!(result, Err(SchemaError::Projection { pointer: _pointer, reason }) if reason.contains("field domain"))
                );
            }
            arms.reverse();
        }
    }
}

#[test]
fn equivalent_alternatives_retain_all_nested_dropped_arm_provenance() {
    for constrained in [
        json!({"type":"array","items":{"type":"string"}}),
        json!({"type":"object","properties":{"Value":{"type":"string"}}}),
        json!({"type":"string","minLength":2}),
    ] {
        let p = probe(
            json!({"anyOf":[
                {"anyOf":[constrained.clone(),{"type":"boolean"}]},
                {"anyOf":[{"type":"boolean"},constrained]}
            ]}),
            json!({}),
        )
        .unwrap();
        assert_eq!(p.dropped_bare_alternatives.len(), 2);
        assert_ne!(
            p.dropped_bare_alternatives[0].arm,
            p.dropped_bare_alternatives[1].arm
        );
        for dropped in p.dropped_bare_alternatives {
            assert_eq!(dropped.kind, AlternativeKind::AnyOf);
            assert_eq!(dropped.json_type, JsonType::Boolean);
        }
    }
}

#[test]
fn cached_object_field_alternatives_preserve_both_required_scopes() {
    for (source, definition, expected) in [
        (
            include_str!("../../../../cfn-schema-cache/AWS__S3__Bucket.json"),
            "TargetObjectKeyFormat",
            vec![vec!["SimplePrefix"], vec!["PartitionedPrefix"]],
        ),
        (
            include_str!("../../../../cfn-schema-cache/AWS__CloudWatch__Alarm.json"),
            "EvaluationWindow",
            vec![vec!["WallClockWindow"], vec!["SlidingWindow"]],
        ),
    ] {
        let source: Value = serde_json::from_str(source).unwrap();
        let layer = Arc::new(Layer::from_json(&source).unwrap());
        let node = layer
            .sources()
            .node_at(&format!("/definitions/{definition}"))
            .unwrap();
        let p = project(&layer, node).unwrap();
        assert_eq!(p.projected_type, ProjectedType::Value(JsonType::Object));
        assert!(p.required.is_empty());
        assert_eq!(p.field_groups.groups.len(), 1);
        assert_eq!(p.field_groups.groups[0].kind, AlternativeKind::OneOf);
        let mut members = expected.iter().flatten().copied().collect::<Vec<_>>();
        members.sort();
        assert_eq!(p.field_groups.groups[0].members, members);
        assert!(p.dropped_bare_alternatives.is_empty());
        for name in expected.into_iter().flatten() {
            assert!(p.properties.as_ref().unwrap().contains_key(name));
        }
    }
}

#[test]
fn equivalent_array_alternatives_preserve_every_collection_constraint() {
    let array = json!({"type":"array","items":{"type":"string","enum":["A","B"]},"minItems":2,"maxItems":5,"uniqueItems":true,"insertionOrder":false});
    let p = probe(json!({"anyOf":[array.clone(),array]}), json!({})).unwrap();
    assert_eq!(
        (p.min_items, p.max_items, p.unique_items, p.insertion_order),
        (Some(2), Some(5), Some(true), Some(false))
    );
}

#[test]
fn numeric_bounds_remain_json_numbers() {
    let p = probe(
        json!({"type":"number","minimum":1.25,"maximum":9.75}),
        json!({}),
    )
    .unwrap();
    assert_eq!(p.minimum, Some(serde_json::Number::from_f64(1.25).unwrap()));
    assert_eq!(p.maximum, Some(serde_json::Number::from_f64(9.75).unwrap()));
}

#[test]
fn scalar_alternatives_and_aliases_cannot_hide_later_constraints() {
    for keyword in ["oneOf", "anyOf"] {
        for later in [
            json!({"type":"string","maxLength":4}),
            json!({"type":"string","enum":["B"]}),
            json!({"type":"string","pattern":"^B$"}),
        ] {
            let p = json!({keyword:[{"$ref":"#/definitions/Alias"},later]});
            assert!(matches!(
                probe(
                    p,
                    json!({"Alias":{"$ref":"#/definitions/Value"},"Value":{"type":"string"}})
                ),
                Err(SchemaError::Projection {
                    pointer: _,
                    reason: _
                })
            ));
        }
    }
}

#[test]
fn conditional_dependencies_do_not_become_unconditional_bounds_or_requirements() {
    let p = probe(
        json!({"type":"array","items":{"type":"string"},"dependencies":{
            "Enabled":{"minItems":3,"uniqueItems":true},"Names":["Other"]
        }}),
        json!({}),
    )
    .unwrap();
    assert_eq!(p.min_items, None);
    assert_eq!(p.unique_items, None);
    assert!(p.required.is_empty());
    assert_eq!(p.dependencies.len(), 2);
    assert!(p.has_constrained_array());
}

#[test]
fn object_alternatives_preserve_required_scopes_and_reject_conflicting_members() {
    let p = probe(
        json!({"type":"object","required":["Always"],"oneOf":[
            {"properties":{"A":{"type":"integer"},"B":{"type":"string"}},"required":["A"]},
            {"properties":{"A":{"type":"integer"},"B":{"type":"string"}},"required":["B"]}
        ]}),
        json!({}),
    )
    .unwrap();
    assert_eq!(p.required, vec!["Always"]);
    assert_eq!(p.field_groups.groups.len(), 1);
    assert_eq!(p.field_groups.groups[0].kind, AlternativeKind::OneOf);
    assert_eq!(p.field_groups.groups[0].members, ["A", "B"]);
    assert!(matches!(
        probe(
            json!({"oneOf":[
                {"properties":{"A":{"type":"string"}}},
                {"properties":{"A":{"type":"string","maxLength":3}}}
            ]}),
            json!({})
        ),
        Err(SchemaError::Projection {
            pointer: _,
            reason: _
        })
    ));
}

#[test]
fn later_object_alternative_map_and_dependency_constraints_are_not_discarded() {
    for extra in [
        json!({"patternProperties":{"^x$":{"type":"integer"}}}),
        json!({"additionalProperties":{"type":"integer"}}),
        json!({"dependencies":{"A":["B"]}}),
        json!({"dependencies":{"A":{"properties":{"B":{"type":"integer"}}}}}),
        json!({"contains":{"type":"string"}}),
    ] {
        let mut later = json!({"type":"object"});
        later
            .as_object_mut()
            .unwrap()
            .extend(extra.as_object().unwrap().clone());
        assert!(
            matches!(
                probe(json!({"oneOf":[{"type":"object"},later]}), json!({})),
                Err(SchemaError::Projection {
                    pointer: _,
                    reason: _
                })
            ),
            "later object facts were discarded: {extra}"
        );
        let p = probe(json!({"anyOf":[later.clone(),later]}), json!({})).unwrap();
        assert_eq!(p.effective_type(), Some("object"));
    }
}

#[test]
fn alternative_array_items_must_preserve_nested_required_alternatives() {
    let mut first = json!({"type":"array","items":{"type":"object","properties":{"A":{"type":"string"},"B":{"type":"string"}},"oneOf":[{"required":["A"]},{"required":["B"]}]}});
    let mut second = first.clone();
    second["items"]["oneOf"][1]["required"] = json!(["A", "B"]);
    assert!(matches!(
        probe(json!({"oneOf":[first.clone(),second]}), json!({})),
        Err(SchemaError::Projection {
            pointer: _,
            reason: _
        })
    ));
    first["items"]["oneOf"][1]["required"] = json!(["B"]);
    assert!(probe(json!({"anyOf":[first.clone(),first]}), json!({})).is_ok());
}

#[test]
fn numeric_constants_are_typed_and_unsupported_literals_are_errors() {
    let p = probe(json!({"type":"integer","const":7}), json!({})).unwrap();
    assert_eq!(p.const_value, Some(json!(7)));
    for value in [
        json!(true),
        Value::Null,
        json!([1]),
        json!({"A":1}),
        json!(1.25),
    ] {
        assert!(matches!(
            probe(json!({"const":value}), json!({})),
            Err(SchemaError::Projection {
                pointer: _,
                reason: _
            })
        ));
    }
    assert!(matches!(
        probe(json!({"enum":["1",1]}), json!({})),
        Err(SchemaError::Projection {
            pointer: _,
            reason: _
        })
    ));
}

#[test]
fn parsed_shared_queries_keep_fractional_endpoints_and_literal_defaults() {
    let schema =
        super::ParsedCfnSchema::from_json(json!({"typeName":"AWS::Test::Projection","properties":{
            "Rate":{"type":"number","minimum":1.25,"maximum":9.75,"default":3.125},
            "Payload":{"type":"object","default":{"type":"literal","values":[1,true,null]}}
        }}))
        .unwrap();
    let p = schema.properties["Rate"].query();
    assert_eq!(p.minimum.as_ref().unwrap().to_string(), "1.25");
    assert_eq!(p.maximum.as_ref().unwrap().to_string(), "9.75");
    let default = p.default_value.as_ref().unwrap();
    assert!(
        super::json_default_to_value_code(default)
            .unwrap()
            .contains("new(3.125)")
    );
    assert_eq!(
        super::json_default_to_markdown(default).as_deref(),
        Some("3.125")
    );
    assert_eq!(
        schema.properties["Payload"].query().default_value,
        Some(json!({"type":"literal","values":[1,true,null]}))
    );
}

#[test]
fn references_to_escaped_definitions_and_non_definition_targets_keep_identity_and_constraints() {
    let schema =
        super::ParsedCfnSchema::from_json(json!({"typeName":"AWS::Test::Projection","properties":{
        "Original":{"type":"string","maxLength":9},
        "Original/Value~Key":{"type":"string"},
        "ViaEscapedProperty":{"$ref":"#/properties/Original~1Value~0Key"},
        "ViaProperty":{"$ref":"#/properties/Original","minLength":2},
        "ViaDefinition":{"$ref":"#/definitions/A~1B~0C","maxLength":6}
    },"definitions":{"A/B~C":{"type":"string","enum":["AAA","BBB"],"minLength":3}}}))
        .unwrap();
    let p = schema.properties["ViaProperty"].query();
    assert_eq!((p.min_length, p.max_length), (Some(2), Some(9)));
    assert_eq!(p.reference.as_ref().unwrap().name, "Original");
    let p = schema.properties["ViaEscapedProperty"].query();
    assert_eq!(p.reference.as_ref().unwrap().name, "Original/Value~Key");
    let p = schema.properties["ViaDefinition"].query();
    assert_eq!((p.min_length, p.max_length), (Some(3), Some(6)));
    assert_eq!(p.reference.as_ref().unwrap().name, "A/B~C");
}

#[test]
fn projection_failures_reach_both_renderer_boundaries_as_schema_errors() {
    let schema=super::ParsedCfnSchema::from_json(json!({"typeName":"AWS::Test::Projection","properties":{
        "Choice":{"oneOf":[{"type":"array","items":{"type":"string"}},{"type":"array","items":{"type":"string"},"uniqueItems":true}]}
    }})).unwrap();
    let resource = super::dsl_resource_name_from_type(&schema.type_name).unwrap();
    let example = carina_docgen::example::check(
        &format!("awscc.{resource} {{}}\n"),
        std::path::Path::new("projection.crn"),
        "awscc",
        &resource,
        carina_docgen::example::ExampleKind::Resource,
    )
    .unwrap();
    let context = super::EmissionContext::from_source(&schema).unwrap();
    for error in [
        super::render_schema_code(&context, &schema.type_name).unwrap_err(),
        super::render_markdown(&context, &schema.type_name, &example).unwrap_err(),
    ] {
        assert!(
            matches!(error.downcast_ref::<SchemaError>(),Some(SchemaError::Projection {pointer,reason: _}) if pointer=="/properties/Choice")
        );
    }
}

#[test]
fn cached_key_schema_shape_is_order_independent_and_validates_the_object_alternative() {
    let source: Value = serde_json::from_str(include_str!(
        "../../../../cfn-schema-cache/AWS__DynamoDB__Table.json"
    ))
    .unwrap();
    let outputs = |source: Value| {
        let schema = super::ParsedCfnSchema::from_json(source).unwrap();
        let context = super::EmissionContext::from_source(&schema).unwrap();
        let resource = super::dsl_resource_name_from_type(&schema.type_name).unwrap();
        let example = carina_docgen::example::check(
            &format!("awscc.{resource} {{}}\n"),
            std::path::Path::new("projection.crn"),
            "awscc",
            &resource,
            carina_docgen::example::ExampleKind::Resource,
        )
        .unwrap();
        (
            super::render_schema_code(&context, &schema.type_name).unwrap(),
            super::render_markdown(
                &super::EmissionContext::from_source(&schema).unwrap(),
                &schema.type_name,
                &example,
            )
            .unwrap(),
        )
    };
    let baseline = outputs(source.clone());
    let mut reversed = source.clone();
    reversed["properties"]["KeySchema"]["oneOf"]
        .as_array_mut()
        .unwrap()
        .reverse();
    assert_eq!(outputs(reversed), baseline);
    for extra in [
        json!({"minProperties":1}),
        json!({"default":{}}),
        json!({"oneOf":[{"required":["A"]},{"required":["B"]}]}),
    ] {
        let mut changed = source.clone();
        changed["properties"]["KeySchema"]["oneOf"][1]
            .as_object_mut()
            .unwrap()
            .extend(extra.as_object().unwrap().clone());
        let schema = super::ParsedCfnSchema::from_json(changed).unwrap();
        assert!(
            matches!(
                schema.properties["KeySchema"].try_query(),
                Err(SchemaError::Projection {
                    pointer: _,
                    reason: _
                })
            ),
            "object alternative lost {extra}"
        );
    }
}

#[test]
fn productive_reference_cycles_reach_a_stable_shape_and_definition_identity() {
    let definitions = json!({
        "A":{"$ref":"#/definitions/B","minItems":2},
        "B":{"$ref":"#/definitions/A","type":"array","items":{"type":"string"},"maxItems":5,"uniqueItems":true}
    });
    for name in ["A", "B"] {
        let p = probe(
            json!({"$ref":format!("#/definitions/{name}")}),
            definitions.clone(),
        )
        .unwrap();
        assert_eq!(p.effective_type(), Some("array"));
        assert_eq!(
            (p.min_items, p.max_items, p.unique_items),
            (Some(2), Some(5), Some(true))
        );
        assert_eq!(p.reference.as_ref().unwrap().name, "A");
    }
    let p = probe(
        json!({"$ref":"#/definitions/A"}),
        json!({"A":{"allOf":[{"$ref":"#/definitions/A"},{"type":"string","minLength":2}]}}),
    )
    .unwrap();
    assert_eq!(p.min_length, Some(2));
    assert_eq!(p.effective_type(), Some("string"));
}

#[test]
fn integer_number_intersection_is_independent_of_all_of_order() {
    for arms in [
        json!([{"type":"number","minimum":1},{"type":"integer","maximum":9}]),
        json!([{"type":"integer","maximum":9},{"type":"number","minimum":1}]),
    ] {
        let p = probe(json!({"allOf":arms}), json!({})).unwrap();
        assert_eq!(p.effective_type(), Some("integer"));
        assert_eq!(p.integer_bounds().unwrap(), (Some(1), Some(9)));
    }
}

#[test]
fn collection_bounds_that_cannot_fit_the_compatibility_projection_are_errors() {
    assert!(matches!(
        probe(
            json!({"type":"array","minItems":u64::MAX,"items":{"type":"string"}}),
            json!({})
        ),
        Err(SchemaError::Projection {
            pointer: _,
            reason: _
        })
    ));
}

#[test]
fn exact_definition_override_queries_retain_typed_projection_errors() {
    let schema = super::ParsedCfnSchema::from_json(
        json!({"typeName":"AWS::Test::Projection","properties":{},"definitions":{
            "Settings":{"oneOf":[{"type":"array","items":{"type":"string"}},{"type":"object","minProperties":1}]}
        }}),
    )
    .unwrap();
    assert!(matches!(
        super::cfn_override_property(
            &schema,
            super::CfnPropertyPath::new(Some("Settings"), "Value")
        ),
        Err(SchemaError::Projection {
            pointer: _,
            reason: _
        })
    ));
}

#[test]
fn incompatible_literal_and_strict_numeric_constraints_in_later_all_of_arms_are_errors() {
    for arms in [
        json!([{"type":"string","enum":["A"]},{"const":"B"}]),
        json!([{"type":"number","minimum":2.5},{"exclusiveMaximum":2.5}]),
        json!([{"type":"object","minProperties":3},{"maxProperties":2}]),
    ] {
        assert!(matches!(
            probe(json!({"allOf":arms}), json!({})),
            Err(SchemaError::Projection {
                pointer: _,
                reason: _
            })
        ));
    }
}

#[test]
fn numeric_intersection_does_not_round_integer_endpoints_through_f64() {
    for arms in [
        json!([{"type":"number","minimum":9007199254740993_u64},{"minimum":9007199254740992.0}]),
        json!([{"minimum":9007199254740992.0},{"type":"number","minimum":9007199254740993_u64}]),
    ] {
        let p = probe(json!({"allOf":arms}), json!({})).unwrap();
        assert_eq!(p.minimum.unwrap().to_string(), "9007199254740993");
    }
    assert!(matches!(
        probe(
            json!({"type":"number","minimum":9007199254740993_u64,"maximum":9007199254740992.0}),
            json!({})
        ),
        Err(SchemaError::Projection {
            pointer: _,
            reason: _
        })
    ));
}

#[test]
fn numeric_intersections_compare_zero_and_small_exponents_exactly() {
    for (min, max) in [(0.0, 1e-200), (-1e-200, 0.0), (-1e100, -1e-200), (0.0, 0.0)] {
        probe(
            json!({"type":"number","minimum":min,"maximum":max}),
            json!({}),
        )
        .unwrap();
    }
    assert!(matches!(
        probe(
            json!({"type":"number","minimum":1e-200,"maximum":0}),
            json!({})
        ),
        Err(SchemaError::Projection {
            pointer: _,
            reason: _
        })
    ));
}

#[test]
fn all_of_compatibility_views_preserve_emitted_bounds_and_unconditional_requirements() {
    let schema=super::ParsedCfnSchema::from_json(json!({"typeName":"AWS::EC2::RouteTable","properties":{
        "Choices":{"allOf":[{"type":"array","items":{"$ref":"#/definitions/Mode"},"insertionOrder":false},{"minItems":2,"maxItems":5,"uniqueItems":true}]},
        "Count":{"allOf":[{"type":"integer","minimum":1},{"maximum":9}]},
        "Settings":{"allOf":[{"properties":{"A":{"type":"string"}},"required":["A"]},{"properties":{"B":{"type":"integer"}}}]}
    },"definitions":{"Mode":{"type":"string","enum":["A","B"]}}})).unwrap();
    let resource = super::dsl_resource_name_from_type(&schema.type_name).unwrap();
    let example = carina_docgen::example::check(
        &format!("awscc.{resource} {{}}\n"),
        std::path::Path::new("projection.crn"),
        "awscc",
        &resource,
        carina_docgen::example::ExampleKind::Resource,
    )
    .unwrap();
    let context = super::EmissionContext::from_source(&schema).unwrap();
    let code = super::render_schema_code(&context, &schema.type_name).unwrap();
    assert!(
        code.contains("false, Some((Some(2), Some(5))), carina_core::schema::ListElements::Unique"),
        "{code}"
    );
    assert!(code.contains("enum_identity(\"Mode\""), "{code}");
    assert!(code.contains("Some((Some(1), Some(9)))"), "{code}");
    assert!(
        code.contains("StructField::required(\"a\", AttributeType::string())"),
        "{code}"
    );
    let context = super::EmissionContext::from_source(&schema).unwrap();
    let _docs = super::render_markdown(&context, &schema.type_name, &example).unwrap();
}

#[test]
fn invalid_handler_and_resource_metadata_return_typed_errors_before_consumers_run() {
    for metadata in [
        json!({"typeConfiguration":17}),
        json!({"handlers":{"create":{"permissions":[17]}}}),
    ] {
        let mut source = json!({"typeName":"AWS::Test::Projection","properties":{}});
        source
            .as_object_mut()
            .unwrap()
            .extend(metadata.as_object().unwrap().clone());
        let error = super::ParsedCfnSchema::from_json(source).unwrap_err();
        assert!(
            matches!(
                error.downcast_ref::<SchemaError>(),
                Some(SchemaError::InvalidShape {
                    pointer: _,
                    expected: _,
                    actual: _
                })
            ),
            "{error:#}"
        );
    }
}

#[test]
fn budget_tag_scalar_all_of_is_validated_by_both_emitters() {
    let mut source: Value = serde_json::from_str(include_str!(
        "../../../../cfn-schema-cache/AWS__Budgets__Budget.json"
    ))
    .unwrap();
    source["definitions"]["ResourceTag"]["properties"]["Key"]["allOf"] = json!([{"minLength":999}]);
    let schema = super::ParsedCfnSchema::from_json(source).unwrap();
    let context = super::EmissionContext::from_source(&schema).unwrap();
    let example = super::tests::markdown_example(&schema.type_name);
    let rust = super::render_schema_code(&context, &schema.type_name).unwrap();
    assert!(
        rust.contains("Some((Some(999), None))"),
        "allOf tag length must reach Rust"
    );
    let _docs = super::render_markdown(&context, &schema.type_name, &example).unwrap();
}
