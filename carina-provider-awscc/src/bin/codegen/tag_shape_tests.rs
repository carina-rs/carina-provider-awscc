use super::*;
use serde_json::{Value, json};

#[test]
fn round3_key_value_item_cardinality_is_rejected_before_caching() {
    for constraint in [json!({"maxProperties":1}), json!({"minProperties":3})] {
        for aliased in [false, true] {
            let item = json!({"allOf":[{"type":"object","properties":{
                "Key":{"type":"string"},"Value":{"type":"string"}
            },"required":["Key","Value"]},constraint]});
            let schema = ParsedCfnSchema::from_json(json!({
                "typeName":"AWS::Test::Tags","properties":{"Tags":{"type":"array","items":
                    if aliased { json!({"$ref":"#/definitions/Pair"}) } else { item.clone() }
                }},"definitions":{"Pair":item}
            })).unwrap();
            let shape = schema.tag_shape(&schema.properties["Tags"]).unwrap();
            assert!(matches!(shape, TagShape::Unreadable { source: schema_layer::SchemaError::Projection { pointer: _, reason }, candidate: _, pointer: _ }
                if reason.contains("property-count")), "{shape:?}");
            assert!(std::ptr::eq(shape, schema.tag_shape(&schema.properties["Tags"]).unwrap()));
            for result in [generate_synthetic_schema_code(&schema), generate_test_markdown(&schema, "AWS::EC2::Route")] {
                let error = result.expect_err("item property-count constraints cannot become tag entries");
                assert!(matches!(error.downcast_ref::<TagShapeError>(), Some(TagShapeError::Unreadable { resource: _, pointer: _, source })
                    if matches!(source.as_ref(), schema_layer::SchemaError::Projection { pointer: _, reason } if reason.contains("property-count"))), "{error:#}");
            }
        }
    }
}

#[test]
fn round2_exclusive_tag_alternatives_are_typed_errors() {
    for tags in [
        json!({"type":"array","items":{"type":"object","properties":{
            "Key":{"type":"string"},"Value":{"type":"string"}
        }}}),
        json!({"type":"object","patternProperties":{".*":{"type":"string"}},"additionalProperties":false}),
    ] {
        let schema = ParsedCfnSchema::from_json(json!({
            "typeName":"AWS::Test::Tags","properties":{"Tags":{"oneOf":[tags.clone(),tags]}}
        })).unwrap();
        for result in [generate_synthetic_schema_code(&schema), generate_test_markdown(&schema, "AWS::EC2::Route")] {
            let error = result.expect_err("tag classification must also reject overlapping exclusive arms");
            assert!(matches!(error.downcast_ref::<TagShapeError>(),
                Some(TagShapeError::Unreadable { resource: _, pointer: _, source })
                if matches!(source.as_ref(), schema_layer::SchemaError::Projection { pointer, reason }
                    if pointer == "/properties/Tags" && reason.contains("oneOf") && reason.contains("overlap"))), "{error:#}");
        }
    }
}

#[test]
fn round2_tag_map_property_bounds_reach_cache_rust_and_markdown() {
    for tags in [
        json!({"type":"object","patternProperties":{".*":{"type":"string"}},"additionalProperties":false,"minProperties":2,"maxProperties":5}),
        json!({"$ref":"#/definitions/Bounded"}),
        json!({"allOf":[{"$ref":"#/definitions/Map"},{"minProperties":2},{"maxProperties":5}]}),
    ] {
        let schema = ParsedCfnSchema::from_json(json!({
            "typeName":"AWS::Test::Tags","properties":{"Tags":tags},"definitions":{
                "Map":{"type":"object","patternProperties":{".*":{"type":"string"}},"additionalProperties":false},
                "Bounded":{"$ref":"#/definitions/Map","minProperties":2,"maxProperties":5}
            }
        })).unwrap();
        let map = schema.tag_shape(&schema.properties["Tags"]).unwrap().map("AWS::Test::Tags").unwrap();
        assert_eq!((map.min_entries, map.max_entries), (Some(2), Some(5)));
        let rust = generate_synthetic_schema_code(&schema).unwrap();
        assert!(rust.contains("Some((Some(2), Some(5)))"), "{rust}");
        let _markdown = generate_test_markdown(&schema, "AWS::EC2::Route").unwrap();

    }
}

#[test]
fn round1_tag_literal_constraints_are_typed_errors() {
    for literal in [json!({"enum":["fixed"]}), json!({"const":"fixed"})] {
        for constrained in ["Key", "Value", "MapValue"] {
            for aliased in [false, true] {
                let mut leaf = literal.clone();
                leaf["type"] = json!("string");
                let site = if aliased { json!({"$ref":"#/definitions/Leaf"}) } else { leaf.clone() };
                let tags = if constrained == "MapValue" {
                    json!({"type":"object","patternProperties":{".*":site},"additionalProperties":false})
                } else {
                    let mut properties = json!({"Key":{"type":"string"},"Value":{"type":"string"}});
                    properties[constrained] = site;
                    json!({"type":"array","items":{"type":"object","properties":properties}})
                };
                let schema = ParsedCfnSchema::from_json(json!({
                    "typeName":"AWS::Test::Tags", "properties":{"Tags":tags},
                    "definitions":{"Leaf":leaf}
                })).unwrap();
                for result in [generate_synthetic_schema_code(&schema), generate_test_markdown(&schema, "AWS::EC2::Route")] {
                    let error = result.expect_err("tag string literals cannot become unrestricted leaves");
                    assert!(matches!(error.downcast_ref::<TagShapeError>(),
                        Some(TagShapeError::Unreadable { resource: _, pointer: _, source })
                        if matches!(source.as_ref(), schema_layer::SchemaError::Projection { pointer: _, reason }
                            if reason.contains("enum") || reason.contains("const"))), "{error:#}");
                }
            }
        }
    }
}

#[test]
fn round1_tag_minimum_entries_reach_rust_and_markdown() {
    static OVERRIDES: LazyLock<ResourceTypeOverrides> = LazyLock::new(|| {
        let mut overrides = ResourceTypeOverrides::default();
        overrides.insert(("AWS::Test::Tags", "Tags"), NarrowingKind::Array {
            items: None, min_items: Some(3), max_items: None,
        });
        overrides
    });
    for max in [None, Some(5)] {
        let mut tags = json!({"type":"array","minItems":2,"items":{
            "type":"object","properties":{"Key":{"type":"string"},"Value":{"type":"string"}}
        }});
        if let Some(max) = max { tags["maxItems"] = json!(max); }
        let schema = ParsedCfnSchema::from_json(json!({
            "typeName":"AWS::Test::Tags", "properties":{"Tags":tags}
        })).unwrap();
        let check = |min| {
            let rust = generate_synthetic_schema_code(&schema).unwrap();
            let upper = max.map_or_else(|| "None".to_string(), |max| format!("Some({max})"));
            assert!(rust.contains(&format!("Some((Some({min}), {upper}))")), "{rust}");
            let markdown = generate_test_markdown(&schema, "AWS::EC2::Route").unwrap();
            let range = carina_core::schema::format_refinement_bounds((Some(min), max));
            assert!(markdown.contains(&format!("(entries: {range})")), "{markdown}");
        };
        check(2);
        with_resource_type_overrides(&OVERRIDES, || check(3));
    }
}

#[test]
fn round1_tag_zero_minimum_uses_the_shared_length_normal_form() {
    for max in [None, Some(5)] {
        let mut source = json!({"typeName":"AWS::Test::Tags","properties":{"Tags":{
            "type":"array","items":{"type":"object","properties":{
                "Key":{"type":"string"},"Value":{"type":"string"}
            }}
        }}});
        if let Some(max) = max { source["properties"]["Tags"]["maxItems"] = json!(max); }
        let unbounded = ParsedCfnSchema::from_json(source.clone()).unwrap();
        source["properties"]["Tags"]["minItems"] = json!(0);
        let zero = ParsedCfnSchema::from_json(source).unwrap();
        assert_eq!(generate_synthetic_schema_code(&zero).unwrap(), generate_synthetic_schema_code(&unbounded).unwrap());
        assert_eq!(generate_test_markdown(&zero, "AWS::EC2::Route").unwrap(), generate_test_markdown(&unbounded, "AWS::EC2::Route").unwrap());
    }
}

#[derive(Clone, Copy, Debug)]
enum Expected {
    List,
    Map,
    Rejected,
    Unreadable,
}

#[test]
fn layer_tag_shape_matrix_checks_both_emitters() {
    let pair = json!({"type":"object","properties":{
        "Key":{"type":"string","minLength":2,"maxLength":17},
        "Value":{"type":"string","maxLength":23}
    },"required":["Key","Value"]});
    let list = |item: Value| json!({"type":"array","maxItems":7,"items":item});
    let array = list(json!({"$ref":"#/definitions/PairAlias"}));
    let map = json!({"type":"object","properties":{},"patternProperties":{
        ".*":{"type":"string","maxLength":23}
    },"additionalProperties":false});
    let mut property_dependency = pair.clone();
    property_dependency["dependencies"] = json!({"Key":["Value"]});
    let mut schema_dependency = pair.clone();
    schema_dependency["dependencies"] = json!({"Key":{"properties":{"Value":{"maxLength":9}}}});
    let mut conditional_required = pair.clone();
    conditional_required["required"] = json!(["Key"]);
    conditional_required["dependencies"] = json!({"Key":["Value"]});
    let mut integer = pair.clone();
    integer["properties"]["Value"] = json!({"type":"integer"});
    let mut extra_required = pair.clone();
    extra_required["required"] = json!(["Key", "Value", "Extra"]);
    let mut extra_field = pair.clone();
    extra_field["properties"]["Extra"] = json!({"type":"string"});
    let mut scalar_all_of = pair.clone();
    scalar_all_of["properties"]["Value"] = json!({"allOf":[
        {"$ref":"#/definitions/ValueAlias"},{"maxLength":23}
    ]});
    let mut rows = vec![
        ("direct", list(pair.clone()), Expected::List, true),
        (
            "item ref and intermediate alias",
            array.clone(),
            Expected::List,
            true,
        ),
        (
            "property ref",
            json!({"$ref":"#/definitions/TagList"}),
            Expected::List,
            true,
        ),
        (
            "array alias siblings",
            json!({"$ref":"#/definitions/ListAlias","maxItems":7}),
            Expected::List,
            true,
        ),
        (
            "allOf array",
            json!({"allOf":[{"type":"array"},{"items":pair,"maxItems":7}]}),
            Expected::List,
            true,
        ),
        (
            "allOf item fields",
            list(json!({"allOf":[
                {"properties":{"Key":pair["properties"]["Key"]},"required":["Key"]},
                {"properties":{"Value":pair["properties"]["Value"]},"required":["Value"]}
            ]})),
            Expected::List,
            true,
        ),
        (
            "allOf scalar and intermediate aliases",
            list(scalar_all_of),
            Expected::List,
            true,
        ),
        (
            "redundant property dependency",
            list(property_dependency),
            Expected::List,
            true,
        ),
        (
            "conditional property dependency",
            list(conditional_required),
            Expected::Rejected,
            true,
        ),
        (
            "schema dependency",
            list(schema_dependency),
            Expected::Rejected,
            true,
        ),
        (
            "schema dependency supplies candidate",
            json!({"type":"array","items":{
                "properties":{"Key":{"type":"string"}},
                "dependencies":{"Key":{"properties":{"Value":{"type":"string"}}}}
            }}),
            Expected::Rejected,
            true,
        ),
        (
            "both dependency forms",
            json!({"type":"array","items":{
                "properties":{"Key":{"type":"string"}},
                "dependencies":{"Key":{"properties":{"Value":{"type":"string"}}},"Value":["Key"]}
            }}),
            Expected::Rejected,
            true,
        ),
        (
            "contains",
            json!({"type":"array","items":{"type":"string"},"contains":pair}),
            Expected::Rejected,
            true,
        ),
        (
            "contains alongside items",
            json!({"type":"array","items":pair,"contains":pair}),
            Expected::Rejected,
            true,
        ),
        (
            "integer Value",
            list(integer.clone()),
            Expected::Rejected,
            true,
        ),
        (
            "incompatible scalar allOf",
            list(json!({"type":"object","properties":{
                "Key":{"allOf":[{"type":"string","maxLength":17},{"minLength":999}]},
                "Value":{"type":"string"}
            }})),
            Expected::Unreadable,
            true,
        ),
        (
            "extra required name",
            list(extra_required),
            Expected::Rejected,
            true,
        ),
        (
            "extra property",
            list(extra_field.clone()),
            Expected::Rejected,
            true,
        ),
        (
            "lowercase",
            list(json!({"type":"object","properties":{
                "key":{"type":"string"},"value":{"type":"string"}
            }})),
            Expected::Rejected,
            true,
        ),
        (
            "mixed case",
            list(json!({"type":"object","properties":{
                "kEy":{"type":"string"},"VaLuE":{"type":"string"}
            }})),
            Expected::Rejected,
            true,
        ),
        (
            "alias cycle",
            list(json!({"$ref":"#/definitions/CycleA"})),
            Expected::Unreadable,
            false,
        ),
        (
            "cycle preserves sibling evidence",
            list(json!({"$ref":"#/definitions/CycleA","allOf":[pair]})),
            Expected::List,
            true,
        ),
        (
            "structural cycle",
            list(json!({"type":"object","properties":{
                "Key":{"type":"string"},"Value":{"$ref":"#/definitions/Recursive"}
            }})),
            Expected::Rejected,
            true,
        ),
        (
            "missing ref",
            list(json!({"$ref":"#/definitions/Missing"})),
            Expected::Unreadable,
            false,
        ),
        (
            "external ref",
            list(json!({"$ref":"https://example.com/schema"})),
            Expected::Unreadable,
            false,
        ),
        (
            "metadata points at scalar",
            json!({"type":"string"}),
            Expected::Rejected,
            false,
        ),
        (
            "metadata points at nested tag list",
            json!({"type":"object","properties":{"Nested":array}}),
            Expected::Rejected,
            false,
        ),
        (
            "nested tag list in items",
            list(json!({"type":"object","properties":{"Nested":array}})),
            Expected::Rejected,
            false,
        ),
        ("string map", map.clone(), Expected::Map, false),
        (
            "string map alias",
            json!({"$ref":"#/definitions/MapAlias"}),
            Expected::Map,
            false,
        ),
    ];
    for keyword in ["oneOf", "anyOf"] {
        rows.push((
            "agreeing list arms",
            json!({keyword:[array,array,array]}),
            if keyword == "oneOf" { Expected::Unreadable } else { Expected::List },
            true,
        ));
        rows.push((
            "agreeing item arms",
            list(json!({keyword:[pair,pair,pair]})),
            if keyword == "oneOf" { Expected::Unreadable } else { Expected::List },
            true,
        ));
        rows.push((
            "agreeing map arms",
            json!({keyword:[map,map,map]}),
            if keyword == "oneOf" { Expected::Unreadable } else { Expected::Map },
            false,
        ));
        rows.push((
            "nullable list",
            json!({keyword:[{"type":"null"},array]}),
            Expected::List,
            true,
        ));
        for position in 0..3 {
            let mut arms = vec![pair.clone(); 3];
            arms[position] = integer.clone();
            rows.push((
                "integer in each item arm",
                list(json!({keyword:arms})),
                Expected::Rejected,
                true,
            ));
            let mut arms = vec![array.clone(); 3];
            arms[position] = list(extra_field.clone());
            rows.push((
                "extra field in each array arm",
                json!({keyword:arms}),
                Expected::Rejected,
                true,
            ));
            let mut arms = vec![json!({"type":"array","items":{"type":"string"}}); 3];
            arms[position] = array.clone();
            rows.push((
                "candidate in each array arm",
                json!({keyword:arms}),
                Expected::Rejected,
                true,
            ));
        }
        rows.push((
            "split Key/Value arms",
            list(json!({keyword:[
                {"properties":{"Key":{"type":"string"}}},
                {"properties":{"Value":{"type":"string"}}}
            ]})),
            Expected::Rejected,
            true,
        ));
    }
    assert_eq!(rows.len(), 58);
    for (name, property, expected, candidate) in rows {
        let source = json!({"typeName":"AWS::Test::Resource","properties":{"Labels":property},
            "tagging":{"taggable":true,"tagProperty":"/properties/Labels"},
            "definitions":{
                "Pair":pair,"PairAlias":{"$ref":"#/definitions/Pair"},
                "TagList":array,"ListAlias":{"$ref":"#/definitions/TagList","maxItems":11},
                "MapAlias":{"$ref":"#/definitions/TagMap"},"TagMap":map,
                "ValueAlias":{"$ref":"#/definitions/StringValue","maxLength":29},
                "StringValue":{"type":"string","maxLength":31},
                "CycleA":{"$ref":"#/definitions/CycleB"},"CycleB":{"$ref":"#/definitions/CycleA"},
                "Recursive":{"type":"object","properties":{"Child":{"$ref":"#/definitions/Recursive"}}}
            }
        });
        let schema = ParsedCfnSchema::from_json(source.clone()).unwrap();
        let shape = schema.tag_shape(&schema.properties["Labels"]).unwrap();
        assert!(
            std::ptr::eq(
                shape,
                schema.tag_shape(&schema.properties["Labels"]).unwrap()
            ),
            "{name}: cached judgement"
        );
        assert_eq!(shape.candidate(), candidate, "{name}: {shape:?}");
        let actual = match shape {
            TagShape::KeyValueList(_map) => Expected::List,
            TagShape::StringMap(_map) => Expected::Map,
            TagShape::KeyValueObject(_map) => Expected::Rejected,
            TagShape::NotTag {
                reason: _reason,
                candidate: _candidate,
                pointer: _pointer,
            } => Expected::Rejected,
            TagShape::Unreadable {
                source: _source,
                candidate: _candidate,
                pointer: _pointer,
            } => Expected::Unreadable,
        };
        assert_eq!(
            std::mem::discriminant(&actual),
            std::mem::discriminant(&expected),
            "{name}: {shape:?}"
        );
        for (mode, result) in [
            ("Rust", generate_synthetic_schema_code(&schema)),
            (
                "Markdown",
                generate_test_markdown(&schema, &schema.type_name),
            ),
        ] {
            match expected {
                Expected::List | Expected::Map => {
                    let output = result.unwrap_or_else(|error| panic!("{name} {mode}: {error:#}"));
                    if mode == "Rust" {
                        assert!(
                            output.contains("AttributeSchema::optional(\"tags\","),
                            "{name}: {output}"
                        );
                        assert!(
                            output.contains(".with_provider_name(\"Labels\")"),
                            "{name}: {output}"
                        );
                        assert!(output.contains("TagsKeyValueCheck"), "{name}: {output}");
                        let wire = match expected {
                            Expected::List => "TagsWire::KeyValueList",
                            Expected::Map => "TagsWire::Map",
                            Expected::Rejected | Expected::Unreadable => {
                                unreachable!("admitted rows")
                            }
                        };
                        assert!(
                            output.contains(&format!("provider_name: \"Labels\", wire: {wire}")),
                            "{name}: {output}"
                        );
                        assert!(
                            output.contains("Some((None, Some(23)))"),
                            "{name}: {output}"
                        );
                    } else {
                        assert!(output.contains("### `tags`\n"), "{name}: {output}");
                        assert!(output.contains(&shared_type(AttributeType::refined_string(None,None,Some((None,Some(23))),None))), "{name}: {output}");
                        carina_docgen::check_page(&output).unwrap();
                    }
                }
                Expected::Rejected | Expected::Unreadable => {
                    let error = result.expect_err(name);
                    assert!(
                        error.downcast_ref::<TagShapeError>().is_some(),
                        "{name} {mode}: {error:#}"
                    );
                    assert!(
                        error
                            .to_string()
                            .contains("AWS::Test::Resource#/properties/Labels"),
                        "{name}: {error:#}"
                    );
                }
            }
        }
        if candidate {
            let mut source = source;
            source.as_object_mut().unwrap().remove("tagging");
            let unclassified = ParsedCfnSchema::from_json(source).unwrap();
            for result in [
                generate_synthetic_schema_code(&unclassified),
                generate_test_markdown(&unclassified, &unclassified.type_name),
            ] {
                let error = result.expect_err(name);
                assert!(
                    error.to_string().contains("unclassified tag-shaped"),
                    "{name}: {error:#}"
                );
            }
        }
    }
}

#[test]
fn nested_tag_shapes_are_not_claimed_by_resource_tagging() {
    for nested in [
        json!({"type":"object","properties":{"Nested":{"type":"array","items":{"$ref":"#/definitions/Tag"}}}}),
        json!({"type":"array","items":{"type":"object","properties":{"Nested":{"type":"array","items":{"$ref":"#/definitions/Tag"}}}}}),
    ] {
        let schema = ParsedCfnSchema::from_json(json!({"typeName":"AWS::Test::Resource",
            "properties":{"Settings":nested},"definitions":{"Tag":{"type":"object","properties":{
                "Key":{"type":"string"},"Value":{"type":"string"}
            }}}
        }))
        .unwrap();
        assert!(
            !schema
                .tag_shape(&schema.properties["Settings"])
                .unwrap()
                .candidate()
        );
        let rust = generate_synthetic_schema_code(&schema).unwrap();
        assert!(!rust.contains("TagsKeyValueCheck"), "{rust}");
        let docs = generate_test_markdown(&schema, &schema.type_name).unwrap();
        assert!(!docs.contains("### `tags`\n"), "{docs}");
        carina_docgen::check_page(&docs).unwrap();
    }
}

#[test]
fn productive_tag_scalar_cycle_preserves_constraints_from_both_entries() {
    let schema = ParsedCfnSchema::from_json(json!({
        "typeName":"AWS::Test::Resource",
        "properties":{"Tags":{"type":"array","items":{"type":"object","properties":{
            "Key":{"$ref":"#/definitions/A","maxLength":17},
            "Value":{"$ref":"#/definitions/B"}
        }}}},
        "definitions":{
            "A":{"$ref":"#/definitions/B","minLength":2},
            "B":{"$ref":"#/definitions/A","type":"string","maxLength":23}
        }
    }))
    .unwrap();
    let rust = generate_synthetic_schema_code(&schema).unwrap();
    for bounds in ["Some((Some(2), Some(17)))", "Some((Some(2), Some(23)))"] {
        assert!(rust.contains(bounds), "{rust}");
    }
    let docs = generate_test_markdown(&schema, &schema.type_name).unwrap();

    carina_docgen::check_page(&docs).unwrap();
}

#[test]
fn resource_compositions_combine_partial_candidate_evidence() {
    for keyword in ["allOf", "oneOf", "anyOf"] {
        let schema = ParsedCfnSchema::from_json(json!({
            "typeName":"AWS::Test::Resource", "properties":{}, keyword:[
                {"properties":{"Labels":{"type":"array","items":{"properties":{"Key":{"type":"string"}}}}}},
                {"properties":{"Labels":{"type":"array","items":{"properties":{"Value":{"type":"string"}}}}}}
            ]
        })).unwrap();
        let context = EmissionContext::from_source(&schema).unwrap();
        let error = validate_tagging_classifications(&context).unwrap_err();
        assert!(error.to_string().contains(
            "unclassified tag-shaped top-level arrays: /properties/Labels"
        ), "{keyword}: {error:#}");
        for result in [
            generate_synthetic_schema_code(&schema),
            generate_test_markdown(&schema, &schema.type_name),
        ] {
            let error = result.expect_err("root property contributions require merged provenance");
            assert!(
                matches!(error.downcast_ref::<schema_layer::SchemaError>(),
                    Some(schema_layer::SchemaError::Projection { pointer, reason: _ })
                    if pointer.is_empty()),
                "{keyword}: {error:#}"
            );
        }
    }
}
