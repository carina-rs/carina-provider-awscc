use super::*;
use serde_json::json;
use std::collections::BTreeSet;

fn keywords<'a>(layer: &'a Layer, pointer: &str) -> &'a PropertyKeywords {
    let id = layer.sources().node_at(pointer).expect(pointer);
    match layer.node(id) {
        SchemaNode::Keywords(keywords) => keywords,
        SchemaNode::Boolean(value) => panic!("expected keywords at {pointer}, got {value}"),
    }
}

fn reference<'a>(layer: &'a Layer, pointer: &str) -> &'a ResolvedRef {
    keywords(layer, pointer).reference.as_ref().unwrap()
}

#[test]
fn supported_keywords_equal_pinned_cli_list() {
    // definitions/properties/allOf/1/properties, fetched 2026-11-06 from
    // https://raw.githubusercontent.com/aws-cloudformation/cloudformation-cli/master/src/rpdk/core/data/schema/base.definition.schema.v1.json
    // Independent of the enum and its spelling conversion; includes CLI-only relationshipRef.
    let pinned = [
        "insertionOrder",
        "arrayType",
        "relationshipRef",
        "$ref",
        "$comment",
        "title",
        "description",
        "examples",
        "default",
        "multipleOf",
        "maximum",
        "exclusiveMaximum",
        "minimum",
        "exclusiveMinimum",
        "maxLength",
        "minLength",
        "pattern",
        "items",
        "maxItems",
        "minItems",
        "uniqueItems",
        "contains",
        "maxProperties",
        "minProperties",
        "required",
        "properties",
        "additionalProperties",
        "patternProperties",
        "dependencies",
        "const",
        "enum",
        "type",
        "format",
        "allOf",
        "anyOf",
        "oneOf",
    ];
    let supported: BTreeSet<_> = PropertyKeyword::ALL
        .iter()
        .map(|key| key.as_str())
        .collect();
    assert_eq!(supported.len(), PropertyKeyword::ALL.len());
    assert_eq!(supported, BTreeSet::from(pinned));
    for keyword in PropertyKeyword::ALL {
        assert_eq!(
            PropertyKeyword::from_name(keyword.as_str(), "").unwrap(),
            keyword
        );
    }
}

#[test]
fn every_keyword_is_stored_without_flattening() {
    let source = json!({
        "$ref": "#/properties/child", "$comment": "comment", "title": "title",
        "description": "description", "examples": [null, false, 2, "x", [], {"unknown": 1}],
        "default": null, "const": {"arbitrary": [1, false]},
        "enum": [null, false, 2, "x", [], {}], "type": ["null", "string", "object"],
        "format": "custom-format", "pattern": "^x$", "minLength": 1, "maxLength": 9,
        "multipleOf": 0.25, "minimum": -1.5, "maximum": 9.75,
        "exclusiveMinimum": -2.25, "exclusiveMaximum": 10.5,
        "minItems": 0, "maxItems": 8, "uniqueItems": false, "insertionOrder": true,
        "arrayType": "AttributeList", "items": false, "contains": {"const": 7},
        "minProperties": 0, "maxProperties": 5, "required": ["child", "other"],
        "properties": {"child": {"type": "string"}}, "additionalProperties": true,
        "patternProperties": {"^key$": {"type": "integer"}},
        "dependencies": {"child": ["other"], "other": {"required": ["child"]}},
        "allOf": [{"minimum": 1}, {"maximum": 9}],
        "oneOf": [{"required": ["a"]}, {"required": ["b"]}],
        "anyOf": [{"title": "first"}, {"title": "last"}],
        "relationshipRef": {"typeName": "AWS::KMS::Key", "propertyPath": "/properties/Arn",
            "publisherId": "publisher", "majorVersion": 2}
    });
    assert_eq!(
        source.as_object().unwrap().len(),
        PropertyKeyword::ALL.len()
    );
    let layer = Layer::from_json(&source).unwrap();
    let node = keywords(&layer, "");
    assert_eq!(node.comment.as_deref(), Some("comment"));
    assert_eq!(node.title.as_deref(), Some("title"));
    assert_eq!(node.description.as_deref(), Some("description"));
    assert_eq!(
        node.examples.as_ref().unwrap(),
        source["examples"].as_array().unwrap()
    );
    assert_eq!(node.default, Some(Value::Null));
    assert_eq!(node.const_value, Some(source["const"].clone()));
    assert_eq!(
        node.enum_values.as_ref().unwrap(),
        source["enum"].as_array().unwrap()
    );
    assert_eq!(
        node.types,
        Some(TypeSet::Multiple(vec![
            JsonType::Null,
            JsonType::String,
            JsonType::Object
        ]))
    );
    assert_eq!(node.format.as_deref(), Some("custom-format"));
    assert_eq!(node.pattern.as_deref(), Some("^x$"));
    for (actual, name) in [
        (&node.multiple_of, "multipleOf"),
        (&node.minimum, "minimum"),
        (&node.maximum, "maximum"),
        (&node.exclusive_minimum, "exclusiveMinimum"),
        (&node.exclusive_maximum, "exclusiveMaximum"),
        (&node.min_length, "minLength"),
        (&node.max_length, "maxLength"),
        (&node.min_items, "minItems"),
        (&node.max_items, "maxItems"),
        (&node.min_properties, "minProperties"),
        (&node.max_properties, "maxProperties"),
    ] {
        assert_eq!(actual.as_ref(), source[name].as_number(), "{name}");
    }
    assert_eq!(node.unique_items, Some(false));
    assert_eq!(node.insertion_order, Some(true));
    assert_eq!(node.array_type, Some(ArrayType::AttributeList));
    assert_eq!(node.required.as_ref().unwrap(), &["child", "other"]);
    assert_eq!(layer.node(node.items.unwrap()), &SchemaNode::Boolean(false));
    assert_eq!(
        layer.node(node.additional_properties.unwrap()),
        &SchemaNode::Boolean(true)
    );
    assert_eq!(node.contains, layer.sources().node_at("/contains"));
    assert_eq!(
        node.properties.as_ref().unwrap()["child"],
        layer.sources().node_at("/properties/child").unwrap()
    );
    assert_eq!(
        node.pattern_properties.as_ref().unwrap()["^key$"],
        layer.sources().node_at("/patternProperties/^key$").unwrap()
    );
    let relationship = node.relationship_ref.as_ref().unwrap();
    assert_eq!(relationship.type_name, "AWS::KMS::Key");
    assert_eq!(relationship.property_path, "/properties/Arn");
    assert_eq!(relationship.publisher_id.as_deref(), Some("publisher"));
    assert_eq!(
        relationship.major_version.as_ref(),
        source["relationshipRef"]["majorVersion"].as_number()
    );
    assert_eq!(reference(&layer, "").spelling, "#/properties/child");
    assert_eq!(
        reference(&layer, "").target,
        RefTarget::Node(node.properties.as_ref().unwrap()["child"])
    );
    assert_eq!(
        keywords(&layer, "/allOf/0").minimum.as_ref(),
        json!(1).as_number()
    );
    assert_eq!(
        keywords(&layer, "/allOf/1").maximum.as_ref(),
        json!(9).as_number()
    );
    assert_eq!(
        keywords(&layer, "/oneOf/0").required.as_ref().unwrap(),
        &["a"]
    );
    assert_eq!(
        keywords(&layer, "/oneOf/1").required.as_ref().unwrap(),
        &["b"]
    );
    assert_eq!(keywords(&layer, "/anyOf/0").title.as_deref(), Some("first"));
    assert_eq!(keywords(&layer, "/anyOf/1").title.as_deref(), Some("last"));
    for (ids, prefix) in [
        (&node.compositions.all_of, "/allOf"),
        (&node.compositions.one_of, "/oneOf"),
        (&node.compositions.any_of, "/anyOf"),
    ] {
        let ids = ids.as_ref().unwrap();
        assert_eq!(ids.len(), 2);
        for (index, id) in ids.iter().enumerate() {
            assert_eq!(layer.sources().pointer(*id), format!("{prefix}/{index}"));
        }
    }
}

#[test]
fn literals_are_never_traversed_as_schemas() {
    for literal in [
        json!(null),
        json!(false),
        json!(-1.25),
        json!("text"),
        json!([1, false]),
        json!({"unknown": true, "$ref": "https://external", "properties": {"x": 42}}),
    ] {
        let source =
            json!({"default": literal, "const": literal, "enum": [literal], "examples": [literal]});
        let layer = Layer::from_json(&source).unwrap();
        let node = keywords(&layer, "");
        assert_eq!(node.default.as_ref(), Some(&literal));
        assert_eq!(node.const_value.as_ref(), Some(&literal));
        assert_eq!(
            node.enum_values.as_ref().unwrap(),
            std::slice::from_ref(&literal)
        );
        assert_eq!(node.examples.as_ref().unwrap(), &[literal]);
        assert_eq!(layer.nodes().len(), 1);
    }
}

#[test]
fn legal_scalar_and_collection_shapes_preserve_presence_and_numbers() {
    for name in [
        "null", "boolean", "object", "array", "number", "integer", "string",
    ] {
        for value in [json!(name), json!([name])] {
            assert!(Layer::from_json(&json!({"type": value})).is_ok(), "{value}");
        }
    }
    for name in [
        "minLength",
        "maxLength",
        "minItems",
        "maxItems",
        "minProperties",
        "maxProperties",
    ] {
        for value in [json!(0), json!(3), json!(3.0), json!(u64::MAX)] {
            let layer = Layer::from_json(&json!({name: value})).unwrap();
            assert_eq!(layer.nodes().len(), 1);
        }
    }
    for name in [
        "minimum",
        "maximum",
        "exclusiveMinimum",
        "exclusiveMaximum",
        "multipleOf",
    ] {
        for value in [json!(0.125), json!(i64::MAX), json!(u64::MAX)] {
            Layer::from_json(&json!({name: value})).unwrap();
        }
    }
    for name in ["uniqueItems", "insertionOrder"] {
        for value in [false, true] {
            Layer::from_json(&json!({name: value})).unwrap();
        }
    }
    for value in ["Standard", "AttributeList"] {
        Layer::from_json(&json!({"arrayType": value})).unwrap();
    }
    for name in [
        "$ref",
        "$comment",
        "title",
        "description",
        "pattern",
        "format",
    ] {
        Layer::from_json(&json!({name: ""})).unwrap();
    }
    let layer = Layer::from_json(&json!({"properties": {}, "patternProperties": {},
        "dependencies": {}, "required": [], "examples": [], "default": null}))
    .unwrap();
    let node = keywords(&layer, "");
    assert!(node.properties.as_ref().unwrap().is_empty());
    assert!(node.pattern_properties.as_ref().unwrap().is_empty());
    assert!(node.dependencies.as_ref().unwrap().is_empty());
    assert_eq!(node.required, Some(vec![]));
    assert_eq!(node.examples, Some(vec![]));
    assert_eq!(node.default, Some(Value::Null));
    assert_eq!(node.enum_values, None);
}

#[test]
fn boolean_schemas_work_at_every_schema_site() {
    for boolean in [false, true] {
        let root = Layer::from_json(&json!(boolean)).unwrap();
        assert_eq!(
            root.node(root.resource().root),
            &SchemaNode::Boolean(boolean)
        );
        let layer = Layer::from_json(&json!({
            "properties": {"x": boolean}, "definitions": {"x": boolean},
            "patternProperties": {"x": boolean}, "items": boolean, "contains": boolean,
            "additionalProperties": boolean, "dependencies": {"x": boolean},
            "allOf": [boolean], "oneOf": [boolean], "anyOf": [boolean],
            "handlers": {"list": {"handlerSchema": boolean}}
        }))
        .unwrap();
        for pointer in [
            "/properties/x",
            "/definitions/x",
            "/patternProperties/x",
            "/items",
            "/contains",
            "/additionalProperties",
            "/dependencies/x",
            "/allOf/0",
            "/oneOf/0",
            "/anyOf/0",
            "/handlers/list/handlerSchema",
        ] {
            assert_eq!(
                layer.node(layer.sources().node_at(pointer).unwrap()),
                &SchemaNode::Boolean(boolean)
            );
        }
    }
}

#[test]
fn both_dependency_forms_keep_all_members() {
    let layer = Layer::from_json(&json!({"dependencies": {
        "a": ["z", "b"], "b": [], "c": {"required": ["a"], "properties": {"d": false}}
    }}))
    .unwrap();
    let dependencies = keywords(&layer, "").dependencies.as_ref().unwrap();
    assert_eq!(
        dependencies["a"],
        Dependency::PropertyList(vec!["z".into(), "b".into()])
    );
    assert_eq!(dependencies["b"], Dependency::PropertyList(vec![]));
    assert_eq!(
        dependencies["c"],
        Dependency::Schema(layer.sources().node_at("/dependencies/c").unwrap())
    );
    assert_eq!(
        keywords(&layer, "/dependencies/c")
            .required
            .as_ref()
            .unwrap(),
        &["a"]
    );
    assert_eq!(layer.sources().node_at("/dependencies/a"), None);
}

#[test]
fn unknown_keywords_are_typed_at_the_exact_site() {
    for (source, pointer, keyword) in [
        (json!({"unknown": 1}), "/unknown", "unknown"),
        (
            json!({"properties": {"a/b~": {"not": {}}}}),
            "/properties/a~1b~0/not",
            "not",
        ),
        (
            json!({"items": {"definitions": {}}}),
            "/items/definitions",
            "definitions",
        ),
        (
            json!({"relationshipRef": {"typeName": "X", "propertyPath": "/x", "extra": 1}}),
            "/relationshipRef/extra",
            "extra",
        ),
        (
            json!({"handlers": {"list": {"future": []}}}),
            "/handlers/list/future",
            "future",
        ),
    ] {
        assert_eq!(
            Layer::from_json(&source).unwrap_err(),
            SchemaError::UnknownKeyword {
                pointer: pointer.into(),
                keyword: keyword.into()
            }
        );
    }
}

#[test]
fn malformed_value_shapes_are_rejected_for_every_keyword() {
    // No bare Value is silently ignored, including null.
    for source in [json!(null), json!(7), json!("string"), json!([])] {
        assert!(
            matches!(Layer::from_json(&source), Err(SchemaError::InvalidShape { pointer, expected, actual })
            if pointer.is_empty() && expected == "schema object or boolean" && actual == ValueKind::of(&source))
        );
    }
    for name in [
        "$ref",
        "$comment",
        "title",
        "description",
        "format",
        "pattern",
        "arrayType",
    ] {
        for bad in [json!(null), json!(false), json!(1), json!([]), json!({})] {
            assert!(
                Layer::from_json(&json!({name: bad})).is_err(),
                "{name}: {bad}"
            );
        }
    }
    for name in [
        "minimum",
        "maximum",
        "exclusiveMinimum",
        "exclusiveMaximum",
        "multipleOf",
        "minLength",
        "maxLength",
        "minItems",
        "maxItems",
        "minProperties",
        "maxProperties",
    ] {
        for bad in [json!(null), json!(false), json!("1"), json!([]), json!({})] {
            assert!(
                Layer::from_json(&json!({name: bad})).is_err(),
                "{name}: {bad}"
            );
        }
    }
    for name in ["insertionOrder", "uniqueItems"] {
        for bad in [json!(null), json!(1), json!("true"), json!([]), json!({})] {
            assert!(
                Layer::from_json(&json!({name: bad})).is_err(),
                "{name}: {bad}"
            );
        }
    }
    for name in ["examples", "enum", "required", "allOf", "oneOf", "anyOf"] {
        for bad in [json!(null), json!(false), json!(1), json!("x"), json!({})] {
            assert!(
                Layer::from_json(&json!({name: bad})).is_err(),
                "{name}: {bad}"
            );
        }
    }
    for name in ["items", "contains", "additionalProperties"] {
        for bad in [json!(null), json!(1), json!("x"), json!([])] {
            assert!(
                Layer::from_json(&json!({name: bad})).is_err(),
                "{name}: {bad}"
            );
        }
    }
    for name in [
        "properties",
        "patternProperties",
        "dependencies",
        "relationshipRef",
    ] {
        for bad in [json!(null), json!(false), json!(1), json!("x"), json!([])] {
            assert!(
                Layer::from_json(&json!({name: bad})).is_err(),
                "{name}: {bad}"
            );
        }
    }
    for bad in [json!(null), json!(3), json!("x")] {
        assert!(Layer::from_json(&json!({"dependencies": {"a": bad}})).is_err());
    }
}

#[test]
fn invalid_keyword_values_are_typed() {
    for source in [
        json!({"arrayType": "unknown"}),
        json!({"type": "unknown"}),
        json!({"type": []}),
        json!({"type": ["string", 1]}),
        json!({"type": ["string", "string"]}),
        json!({"enum": []}),
        json!({"multipleOf": 0}),
        json!({"multipleOf": -0.5}),
        json!({"minItems": -1}),
        json!({"maxLength": 1.5}),
        json!({"required": [false]}),
        json!({"allOf": []}),
        json!({"oneOf": [null]}),
        json!({"anyOf": [42]}),
        json!({"dependencies": {"a": [1]}}),
        json!({"relationshipRef": {}}),
    ] {
        assert!(Layer::from_json(&source).is_err(), "{source}");
    }
}

#[test]
fn reference_siblings_and_each_branch_stay_separate() {
    let layer = Layer::from_json(&json!({
        "properties": {"x": {"$ref": "#/definitions/A", "minLength": 3,
            "allOf": [{"pattern": "a"}, {"pattern": "b"}],
            "anyOf": [{"type": "string"}, {"type": "number"}],
            "oneOf": [{"required": ["a"]}, {"required": ["b", "c"]}]}},
        "definitions": {"A": {"$ref": "#/definitions/B", "maxLength": 12},
            "B": {"description": "target", "minLength": 1}}
    }))
    .unwrap();
    assert_eq!(
        keywords(&layer, "/properties/x").min_length.as_ref(),
        json!(3).as_number()
    );
    assert_eq!(
        keywords(&layer, "/definitions/A").max_length.as_ref(),
        json!(12).as_number()
    );
    assert_eq!(
        keywords(&layer, "/definitions/B").min_length.as_ref(),
        json!(1).as_number()
    );
    assert_eq!(
        reference(&layer, "/properties/x").target,
        RefTarget::Node(layer.sources().node_at("/definitions/A").unwrap())
    );
    assert_eq!(
        reference(&layer, "/definitions/A").target,
        RefTarget::Node(layer.sources().node_at("/definitions/B").unwrap())
    );
    assert_eq!(
        keywords(&layer, "/properties/x/oneOf/1")
            .required
            .as_ref()
            .unwrap(),
        &["b", "c"]
    );
    assert_eq!(
        keywords(&layer, "/properties/x/allOf/1").pattern.as_deref(),
        Some("b")
    );
    assert_eq!(
        keywords(&layer, "/properties/x/anyOf/1").types,
        Some(TypeSet::Single(JsonType::Number))
    );
}

#[test]
fn pointers_support_escaping_percent_encoding_and_non_definition_targets() {
    let mut properties = serde_json::Map::new();
    let targets = [
        ("a/b~c", "#/properties/a~1b~0c"),
        ("a/b~c", "#/%70roperties/a%7E1b%7E0c"),
        ("space here", "#/properties/space%20here"),
        ("雪", "#/properties/%E9%9B%AA"),
        ("+", "#/properties/+"),
        ("%", "#/properties/%25"),
        ("~1", "#/properties/~01"),
        ("", "#/properties/"),
    ];
    for (index, (name, spelling)) in targets.iter().enumerate() {
        properties.insert(name.to_string(), json!(false));
        properties.insert(format!("ref{index}"), json!({"$ref": spelling}));
    }
    let layer = Layer::from_json(&json!({"properties": properties,
        "allOf": [{"const": 1}, {"$ref": "#/allOf/0"}],
        "handlers": {"list": {"handlerSchema": {"$ref": "#/properties/a~1b~0c"}}},
        "oneOf": [{"$ref": "#/handlers/list/handlerSchema"}, {"$ref": "#"}]
    }))
    .unwrap();
    for (index, (name, spelling)) in targets.iter().enumerate() {
        let pointer = format!("/properties/{}", name.replace('~', "~0").replace('/', "~1"));
        let site = reference(&layer, &format!("/properties/ref{index}"));
        assert_eq!(site.spelling, *spelling);
        assert_eq!(
            site.target,
            RefTarget::Node(layer.sources().node_at(&pointer).unwrap())
        );
    }
    for (site, target) in [
        ("/allOf/1", "/allOf/0"),
        ("/oneOf/0", "/handlers/list/handlerSchema"),
        ("/oneOf/1", ""),
    ] {
        assert_eq!(
            reference(&layer, site).target,
            RefTarget::Node(layer.sources().node_at(target).unwrap())
        );
    }
}

#[test]
fn dangling_external_and_non_schema_targets_retain_typed_reasons() {
    for (spelling, reason) in [
        (
            "#/definitions/Missing",
            SchemaError::MissingReferenceTarget {
                pointer: "/$ref".into(),
                reference: "#/definitions/Missing".into(),
            },
        ),
        (
            "https://example.com/schema#/x",
            SchemaError::UnsupportedReference {
                pointer: "/$ref".into(),
                reference: "https://example.com/schema#/x".into(),
            },
        ),
        (
            "#/default",
            SchemaError::NonSchemaReferenceTarget {
                pointer: "/$ref".into(),
                reference: "#/default".into(),
                target: "/default".into(),
            },
        ),
        (
            "#/properties",
            SchemaError::NonSchemaReferenceTarget {
                pointer: "/$ref".into(),
                reference: "#/properties".into(),
                target: "/properties".into(),
            },
        ),
        (
            "#/examples/0",
            SchemaError::NonSchemaReferenceTarget {
                pointer: "/$ref".into(),
                reference: "#/examples/0".into(),
                target: "/examples/0".into(),
            },
        ),
    ] {
        let layer = Layer::from_json(&json!({"$ref": spelling, "default": {"type": "string"},
            "properties": {}, "examples": [{"type": "string"}]}))
        .unwrap();
        assert_eq!(reference(&layer, "").target, RefTarget::Unresolved(reason));
    }
}

#[test]
fn malformed_pointers_have_typed_unresolved_states() {
    for spelling in [
        "#not-a-pointer",
        "#/%",
        "#/%0",
        "#/%GG",
        "#/%FF",
        "#/a~2",
        "#/a~",
    ] {
        let layer = Layer::from_json(&json!({"$ref": spelling})).unwrap();
        assert!(matches!(&reference(&layer, "").target,
            RefTarget::Unresolved(SchemaError::InvalidPointer {pointer, reference, reason})
            if pointer == "/$ref" && reference == spelling && !reason.is_empty()));
    }
    let layer = Layer::from_json(&json!({"allOf": [{"$ref": "#/allOf/00"}]})).unwrap();
    assert_eq!(
        reference(&layer, "/allOf/0").target,
        RefTarget::Unresolved(SchemaError::MissingReferenceTarget {
            pointer: "/allOf/0/$ref".into(),
            reference: "#/allOf/00".into()
        })
    );
}

#[test]
fn alias_and_structural_cycles_are_finite_node_edges() {
    let layer = Layer::from_json(&json!({"definitions": {
        "A": {"$ref": "#/definitions/B"}, "B": {"$ref": "#/definitions/A"},
        "Self": {"$ref": "#/definitions/Self"},
        "Tree": {"type": "object", "properties": {"children": {"type": "array", "items": {"$ref": "#/definitions/Tree"}}}}
    }})).unwrap();
    assert_eq!(layer.nodes().len(), 7);
    for (site, target) in [
        ("/definitions/A", "/definitions/B"),
        ("/definitions/B", "/definitions/A"),
        ("/definitions/Self", "/definitions/Self"),
        (
            "/definitions/Tree/properties/children/items",
            "/definitions/Tree",
        ),
    ] {
        assert_eq!(
            reference(&layer, site).target,
            RefTarget::Node(layer.sources().node_at(target).unwrap())
        );
    }
}

#[test]
fn root_metadata_and_handler_schemas_are_typed_and_source_located() {
    let source = json!({"$schema": "schema", "$id": "id", "type": "RESOURCE", "typeName": "AWS::Test::Resource",
        "sourceUrl": "source", "documentationUrl": "docs", "description": "resource",
        "definitions": {"D": true}, "properties": {}, "additionalProperties": false,
        "readOnlyProperties": ["/properties/a"], "writeOnlyProperties": ["/properties/b"],
        "createOnlyProperties": ["/properties/c"], "conditionalCreateOnlyProperties": ["/properties/d"],
        "deprecatedProperties": ["/properties/e"], "nonPublicProperties": ["/properties/f"],
        "nonPublicDefinitions": ["/definitions/D"], "primaryIdentifier": ["/properties/a"],
        "additionalIdentifiers": [["/properties/b", "/properties/c"]], "taggable": true,
        "replacementStrategy": "delete_then_create", "propertyTransform": {"/properties/a": "$lowercase(a)"},
        "tagging": {"taggable": true, "tagOnCreate": false, "tagUpdatable": true,
            "cloudFormationSystemTags": false, "tagProperty": "/properties/Tags", "permissions": ["svc:Tag"]},
        "handlers": {"list": {"permissions": ["svc:List"], "timeoutInMinutes": 120,
            "handlerSchema": {"oneOf": [{"required": ["a"]}, {"required": ["b"]}]}}},
        "resourceLink": {"$comment": "link", "templateUri": "/console", "mappings": {"id": "/properties/a"}},
        "remote": {"schema0": {"$comment": "remote", "properties": {"a": true}, "definitions": {"D": false}, "vendor": {"unknown": 1}}},
        "typeConfiguration": {"properties": {"token": {"type": "string"}}, "required": ["token"]}
    });
    let layer = Layer::from_json(&source).unwrap();
    let resource = layer.resource();
    assert_eq!(resource.schema_uri.as_deref(), Some("schema"));
    assert_eq!(resource.id.as_deref(), Some("id"));
    assert_eq!(resource.resource_type.as_deref(), Some("RESOURCE"));
    assert_eq!(resource.type_name.as_deref(), Some("AWS::Test::Resource"));
    assert_eq!(resource.source_url.as_deref(), Some("source"));
    assert_eq!(resource.documentation_url.as_deref(), Some("docs"));
    assert_eq!(
        resource.definitions.as_ref().unwrap()["D"],
        layer.sources().node_at("/definitions/D").unwrap()
    );
    for (actual, expected) in [
        (&resource.read_only_properties, "readOnlyProperties"),
        (&resource.write_only_properties, "writeOnlyProperties"),
        (&resource.create_only_properties, "createOnlyProperties"),
        (
            &resource.conditional_create_only_properties,
            "conditionalCreateOnlyProperties",
        ),
        (&resource.deprecated_properties, "deprecatedProperties"),
        (&resource.non_public_properties, "nonPublicProperties"),
        (&resource.non_public_definitions, "nonPublicDefinitions"),
        (&resource.primary_identifier, "primaryIdentifier"),
    ] {
        assert_eq!(serde_json::to_value(actual).unwrap(), source[expected]);
    }
    assert_eq!(
        resource.additional_identifiers,
        Some(vec![vec!["/properties/b".into(), "/properties/c".into()]])
    );
    assert_eq!(resource.taggable, Some(true));
    assert_eq!(
        resource.replacement_strategy,
        Some(ReplacementStrategy::DeleteThenCreate)
    );
    assert_eq!(
        resource.property_transform.as_ref().unwrap()["/properties/a"],
        "$lowercase(a)"
    );
    let tagging = resource.tagging.as_ref().unwrap();
    assert_eq!(tagging.taggable, Some(true));
    assert_eq!(tagging.tag_on_create, Some(false));
    assert_eq!(tagging.tag_updatable, Some(true));
    assert_eq!(tagging.cloud_formation_system_tags, Some(false));
    assert_eq!(tagging.tag_property.as_deref(), Some("/properties/Tags"));
    assert_eq!(tagging.permissions.as_ref().unwrap(), &["svc:Tag"]);
    let handler = &resource.handlers.as_ref().unwrap()["list"];
    assert_eq!(handler.permissions.as_ref().unwrap(), &["svc:List"]);
    assert_eq!(handler.timeout_in_minutes.as_ref(), json!(120).as_number());
    assert_eq!(
        handler.schema,
        layer.sources().node_at("/handlers/list/handlerSchema")
    );
    assert_eq!(
        keywords(&layer, "/handlers/list/handlerSchema/oneOf/1")
            .required
            .as_ref()
            .unwrap(),
        &["b"]
    );
    let link = resource.resource_link.as_ref().unwrap();
    assert_eq!(link.comment.as_deref(), Some("link"));
    assert_eq!(link.template_uri.as_deref(), Some("/console"));
    assert_eq!(link.mappings.as_ref().unwrap()["id"], "/properties/a");
    let remote = &resource.remote.as_ref().unwrap()["schema0"];
    assert_eq!(remote.comment.as_deref(), Some("remote"));
    assert_eq!(
        remote.annotations["vendor"],
        source["remote"]["schema0"]["vendor"]
    );
    assert_eq!(
        remote.properties.as_ref().unwrap()["a"],
        layer
            .sources()
            .node_at("/remote/schema0/properties/a")
            .unwrap()
    );
    assert_eq!(
        remote.definitions.as_ref().unwrap()["D"],
        layer
            .sources()
            .node_at("/remote/schema0/definitions/D")
            .unwrap()
    );
    assert_eq!(
        resource.type_configuration.as_ref().unwrap().root,
        layer.sources().node_at("/typeConfiguration").unwrap()
    );
}

#[test]
fn all_55_cached_resources_build_the_layer_alongside_legacy_parse() {
    let cache = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../cfn-schema-cache");
    let mut paths: Vec<_> = std::fs::read_dir(cache)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| {
            path.extension()
                .is_some_and(|extension| extension == "json")
        })
        .collect();
    paths.sort();
    assert_eq!(paths.len(), 55);
    let mut relationships = 0;
    let mut external_references = BTreeSet::new();
    for path in paths {
        let source: Value = serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
        let parsed =
            super::super::parsed_cfn_schema::ParsedCfnSchema::from_json(source.clone()).unwrap();
        let layer = parsed
            .layer()
            .as_ref()
            .unwrap_or_else(|error| panic!("{}: {error}", path.display()));
        assert_eq!(
            layer.resource().type_name.as_deref(),
            source["typeName"].as_str()
        );
        for (index, node) in layer.nodes().iter().enumerate() {
            let pointer = layer.sources().pointer(NodeId(index));
            assert_eq!(layer.sources().node_at(pointer), Some(NodeId(index)));
            assert!(
                source.pointer(pointer).is_some(),
                "{}#{pointer}",
                path.display()
            );
            match node {
                SchemaNode::Boolean(value) => {
                    assert_eq!(source.pointer(pointer), Some(&json!(value)))
                }
                SchemaNode::Keywords(keywords) => {
                    relationships += usize::from(keywords.relationship_ref.is_some());
                    if let Some(reference) = &keywords.reference {
                        match &reference.target {
                            RefTarget::Node(id) => assert!(id.0 < layer.nodes().len()),
                            RefTarget::Unresolved(reason) => {
                                assert_eq!(
                                    reason,
                                    &SchemaError::UnsupportedReference {
                                        pointer: child(pointer, "$ref"),
                                        reference: reference.spelling.clone()
                                    },
                                    "{}#{pointer}",
                                    path.display()
                                );
                                external_references.insert((
                                    path.file_name().unwrap().to_str().unwrap().to_string(),
                                    pointer.to_string(),
                                    reference.spelling.clone(),
                                ));
                            }
                        }
                    }
                }
            }
        }
    }
    assert_eq!(relationships, 13);
    assert_eq!(external_references.len(), 15);
    assert!(
        external_references
            .iter()
            .all(|(_resource, pointer, reference)| pointer
                .starts_with("/handlers/list/handlerSchema/")
                && reference.starts_with("resource-schema.json#/properties/"))
    );
}

#[test]
fn unexpected_layer_errors_in_test_fixtures_are_loud_and_edits_are_atomic() {
    let source =
        json!({"typeName": "AWS::Test::Resource", "properties": {"x": {"type": "string"}}});
    let mut parsed =
        super::super::parsed_cfn_schema::ParsedCfnSchema::from_json(source.clone()).unwrap();
    let panic = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        parsed.edit_json(|source| source["properties"]["x"]["unknown"] = json!(true));
    }))
    .expect_err("a layer error must not silently pass the unit tests");
    assert!(
        panic
            .downcast_ref::<String>()
            .unwrap()
            .contains("lossless CFN layer failed: unknown schema keyword")
    );
    assert_eq!(parsed.source(), &source);
    assert_eq!(
        keywords(parsed.layer().as_ref().unwrap(), "/properties/x").types,
        Some(TypeSet::Single(JsonType::String))
    );
    parsed.edit_json(|source| source["properties"]["x"]["type"] = json!("integer"));
    assert_eq!(
        keywords(parsed.layer().as_ref().unwrap(), "/properties/x").types,
        Some(TypeSet::Single(JsonType::Integer))
    );
}
