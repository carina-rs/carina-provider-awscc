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
            let _markdown = generate_test_markdown(&schema, "AWS::EC2::Route").unwrap();

        }
        for (property, display) in [
            (json!({"type":"object","properties":{"Marker":closed}}), "Struct(Marker)"),
            (json!({"type":"array","items":closed}), "Struct(Probe)"),
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
            assert!(markdown.contains("CFN format: date-time"), "{markdown}");
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

fn review602_render(schema: &ParsedCfnSchema, markdown: bool) -> Result<String> {
    if markdown {
        generate_test_markdown(schema, "AWS::EC2::Route")
    } else {
        generate_synthetic_schema_code(schema)
    }
}

fn review602_rejected(schema: &ParsedCfnSchema, markdown: bool, facts: &[&str]) {
    let error = review602_render(schema, markdown).expect_err("unpreserved constraint was admitted");
    let projection = error
        .chain()
        .find_map(|source| {
            source.downcast_ref::<schema_layer::SchemaError>().or_else(|| {
                source.downcast_ref::<Box<schema_layer::SchemaError>>().map(Box::as_ref)
            })
        });
    assert!(
        matches!(projection, Some(schema_layer::SchemaError::Projection { pointer, reason })
            if !pointer.is_empty() && facts.iter().all(|fact| reason.contains(fact))),
        "markdown={markdown}: {error:#}"
    );
}

macro_rules! boundary_renderers {
    ($rust:ident, $markdown:ident, $case:ident) => {
        #[test]
        fn $rust() { $case(false); }
        #[test]
        fn $markdown() { $case(true); }
    };
}

fn boundary_canonical_enum(markdown: bool) {
    let schema = ParsedCfnSchema::from_json(json!({
        "typeName":"AWS::Test::Emission",
        "definitions":{"Choice":{"type":"object","properties":{
            "Mode":{"type":"string","enum":["red","blue"]}
        }}},
        "properties":{"Probe":{"$ref":"#/definitions/Choice",
            "properties":{"Mode":{"enum":["red"]}}}}
    })).unwrap();
    // Rejecting a conflicting canonical identity is permitted; widening it is not.
    review602_rejected(&schema, markdown, &["enum", "domain"]);
}
boundary_renderers!(boundary_canonical_enum_rust, boundary_canonical_enum_markdown, boundary_canonical_enum);

fn boundary_inferred_string(markdown: bool) {
    let schema = ParsedCfnSchema::from_json(json!({
        "typeName":"AWS::Test::Emission",
        "properties":{"SubnetId":{"allOf":[{"type":"string"},{"maxLength":16}]}}
    })).unwrap();
    let output = review602_render(&schema, markdown).unwrap();
    if markdown { assert!(output.contains("aws.ec2.Subnet.Id")); }
    else { assert!(output.contains("Some((None, Some(16)))"), "{output}"); }
}
boundary_renderers!(boundary_inferred_string_rust, boundary_inferred_string_markdown, boundary_inferred_string);

fn boundary_inferred_enum(markdown: bool) {
    for (name, property, removed, retained) in [
        ("Probe", json!({"type":"string","description":"Allowed values: ab | cd", "pattern":"^ab$"}), "cd", "ab"),
        ("ConnectivityType", json!({"type":"string","pattern":"^public$"}), "private", "public"),
    ] {
        let schema = ParsedCfnSchema::from_json(json!({
            "typeName":"AWS::Test::Emission","properties":{name:property}
        })).unwrap();
        let output = review602_render(&schema, markdown).unwrap();
        let literal = if markdown { format!("`{removed}`") } else { format!("\"{removed}\".to_string()") };
        assert!(!output.contains(&literal), "filtered literal survived: {output}");
        assert!(output.contains(retained), "{output}");
    }
}
boundary_renderers!(boundary_inferred_enum_rust, boundary_inferred_enum_markdown, boundary_inferred_enum);

fn boundary_open_map(markdown: bool) {
    let property = json!({"type":"object","patternProperties":{"^Known[0-9]+$":{"type":"string","minLength":2}}});
    for restrict_keys in [false, true] {
        let mut property = property.clone();
        if restrict_keys {
            property["additionalProperties"] = json!(false);
        }
        let schema = fixture(property);
        let plan = emission_plan::lower(schema.properties["Probe"].try_query().unwrap()).unwrap();
        assert_eq!(plan.map().unwrap().restrict_keys, restrict_keys);
        let output = review602_render(&schema, markdown).unwrap();
        let expected = match (markdown, restrict_keys) {
            (true, false) => shared_type(AttributeType::map(AttributeType::refined_string(None,None,Some((Some(2),None)),None))),
            (true, true) => shared_type(AttributeType::map_with_key(AttributeType::refined_string(None, Some(ApprovedPattern::whole("^Known[0-9]+$").as_spec().clone()),None,None), AttributeType::refined_string(None,None,Some((Some(2),None)),None))),
            (false, false) => "AttributeType::map(AttributeType::refined_string(None, None, Some((Some(2), None)), None))".to_string(),
            (false, true) => "AttributeType::map_with_key(".to_string(),
        };
        assert!(output.contains(&expected), "{output}");
        if !markdown {
            assert_eq!(output.contains("^Known[0-9]+$"), restrict_keys, "{output}");
            assert!(output.contains("Some((Some(2), None))"), "{output}");
        }
    }
    for additional in [json!(true), json!({"type":"string"})] {
        let mut property = property.clone();
        property["additionalProperties"] = additional;
        review602_rejected(&fixture(property), markdown, &["additionalProperties", "map"]);
    }
}
boundary_renderers!(boundary_open_map_rust, boundary_open_map_markdown, boundary_open_map);

fn boundary_constructor_definition(markdown: bool) {
    let mut overrides = ResourceTypeOverrides::default();
    overrides.insert(("AWS::Test::Emission", "Probe"), TypeOverride::identity(Some(StringIdentity::SsoInstanceArn)));
    with_resource_type_overrides(Box::leak(Box::new(overrides)), || {
        for max in [19, 20] {
            let output = review602_render(&fixture(json!({"type":"string","maxLength":max})), markdown).unwrap();
            assert!(!output.is_empty());
            if !markdown { assert!(output.contains(&format!("Some({max})")), "{output}"); }
        }
    });
}
boundary_renderers!(boundary_constructor_definition_rust, boundary_constructor_definition_markdown, boundary_constructor_definition);

fn boundary_numeric_override(markdown: bool) {
    static OVERRIDES: LazyLock<ResourceTypeOverrides> = LazyLock::new(|| {
        let mut overrides = ResourceTypeOverrides::default();
        overrides.insert(("AWS::ElasticLoadBalancingV2::Listener", "Port"), NarrowingKind::IntRange(1, 65535));
        overrides
    });
    with_resource_type_overrides(&OVERRIDES, || {
    for kind in ["integer", "number"] {
        let schema = ParsedCfnSchema::from_json(json!({
            "typeName":"AWS::ElasticLoadBalancingV2::Listener",
            "properties":{"Port":{"type":kind,"multipleOf":2}}
        })).unwrap();
        let output = review602_render(&schema, markdown).unwrap();
        if markdown { assert!(output.contains("multiple of 2"), "{output}"); }
        else { assert!(output.contains("65535"), "{output}"); }
    }
    });
}
boundary_renderers!(boundary_numeric_override_rust, boundary_numeric_override_markdown, boundary_numeric_override);

fn boundary_unique_collection(markdown: bool) {
    for items in [json!({"type":"string"}), json!({"type":"string","enum":["a","b"]}),
        json!({"type":"object","properties":{"Value":{"type":"string"}}})] {
        let schema = fixture(json!({"type":"array","items":items,"uniqueItems":true}));
        let output = review602_render(&schema, markdown).unwrap();
        assert!(output.contains(if markdown { "unique items" } else { "ListElements::Unique" }), "{output}");
    }
}
boundary_renderers!(boundary_unique_collection_rust, boundary_unique_collection_markdown, boundary_unique_collection);

fn boundary_read_only_query(markdown: bool) {
    let schema = ParsedCfnSchema::from_json(json!({
        "typeName":"AWS::Test::Emission",
        "properties":{"Probe":{"type":"object","properties":{"Nested":{
            "anyOf":[{"type":"string"},{"type":"integer"}]}}}},
        "readOnlyProperties":["/properties/Probe/Nested/Output"]
    })).unwrap();
    let mode = std::panic::catch_unwind(|| super::super::is_property_read_only(&schema, &schema.properties["Probe"], &["Probe".to_string()]));
    assert!(mode.expect("mode inference must propagate the descendant query error").is_err());
    review602_rejected(&schema, markdown, &[]);
}
boundary_renderers!(boundary_read_only_query_rust, boundary_read_only_query_markdown, boundary_read_only_query);

fn boundary_replacement_refinements(markdown: bool) {
    for (override_type, property) in [
        (NarrowingKind::Primitive { expression: ConstraintSpec::int_range(Some(1), Some(5)),  block_name: None, default: None }, json!({"type":"string"})),
        (NarrowingKind::Primitive { expression: ConstraintSpec::document(DocumentKind::SnsSubscriptionRedrivePolicy),  block_name: None, default: None }, json!({"type":"array","items":{"type":"string"},"uniqueItems":true})),
        (NarrowingKind::Enum(vec!["ab","cd"]), json!({"type":"string","pattern":"^ab$"})),
        (NarrowingKind::IntEnum(vec![2,3]), json!({"type":"integer","multipleOf":2})),
    ] {
        let mut overrides = ResourceTypeOverrides::default();
        overrides.insert(("AWS::Test::Emission","Probe"), override_type);
        with_resource_type_overrides(Box::leak(Box::new(overrides)), || {
            review602_rejected(&fixture(property), markdown, &[]);
        });
    }
}
boundary_renderers!(boundary_replacement_refinements_rust, boundary_replacement_refinements_markdown, boundary_replacement_refinements);

fn boundary_override_collections_and_ranges(markdown: bool) {
    let mut overrides = ResourceTypeOverrides::default();
    overrides.insert(("AWS::Test::Emission","Probe"), NarrowingKind::ArrayItemType {
        expression: ConstraintSpec::string(None, None),  block_name: None, default: None, min_items: None, max_items: None,
    }.with_array_bounds(Some(1), None));
    with_resource_type_overrides(Box::leak(Box::new(overrides)), || {
        let output = review602_render(&fixture(json!({"type":"array","items":{"type":"string"},"uniqueItems":true})), markdown).unwrap();
        assert!(output.contains(if markdown {"unique items"} else {"ListElements::Unique"}), "{output}");
    });
    for kind in ["integer", "number"] {
        let mut overrides = ResourceTypeOverrides::default();
        overrides.insert(("AWS::Test::Emission","Probe"), NarrowingKind::IntRange(1,10));
        with_resource_type_overrides(Box::leak(Box::new(overrides)), || {
            let output = review602_render(&fixture(json!({"type":kind,"minimum":8,"maximum":12})), markdown).unwrap();
            assert!(output.contains(if markdown {"8..=10"} else if kind == "integer" {"Some(8), Some(10)"} else {"FiniteF64::new(8.0)"}), "{output}");
        });
    }
}
boundary_renderers!(boundary_override_collections_rust, boundary_override_collections_markdown, boundary_override_collections_and_ranges);

fn review602_nonobject_one_of(markdown: bool) {
    for (property, kind) in [
        (json!({"type":"string"}), "string"),
        (json!({"type":"integer"}), "integer"),
        (json!({"type":"number"}), "number"),
        (json!({"type":"boolean"}), "boolean"),
        (json!({"type":"string","enum":["x"]}), "enum/literal"),
        (json!({"type":"array","items":{"type":"string"}}), "list"),
        (json!({"type":"array","uniqueItems":true,"items":{"type":"string"}}), "set"),
    ] {
        let choice = json!({"oneOf":[{"required":["A"]},{"required":["B"]}]});
        let mut direct = property.clone();
        direct["oneOf"] = choice["oneOf"].clone();
        for property in [direct, json!({"allOf":[property,choice]})] {
            review602_rejected(&fixture(property), markdown, &["oneOf", kind]);
        }
    }
}

#[test]
fn review602_nonobject_one_of_rust() {
    review602_nonobject_one_of(false);
}

#[test]
fn review602_nonobject_one_of_markdown() {
    review602_nonobject_one_of(true);
}

#[test]
fn review602_vacuous_inclusive_and_single_required_arms_are_inapplicable() {
    for property in [
        json!({"type":"string"}), json!({"type":"integer"}),
        json!({"type":"number"}), json!({"type":"boolean"}),
        json!({"type":"string","enum":["x"]}),
        json!({"type":"array","items":{"type":"string"}}),
        json!({"type":"array","uniqueItems":true,"items":{"type":"string"}}),
    ] {
        same_rendering(property.clone(), "anyOf", json!([
            {"required":["Always","A"]},{"required":["Always","B"]}
        ]));
        same_rendering(property, "oneOf", json!([{"required":["A"]}]));
    }
}

fn review602_materialized_tag_alternatives(markdown: bool) {
    for keyword in ["anyOf", "oneOf"] {
        for leaf in ["Key", "Value", "MapValue"] {
            for reversed in [false, true] {
                let alternative = json!({keyword:[
                    {"type":"string","minLength":1},{"type":"integer"}
                ]});
                let mut properties = json!({"Key":{"type":"string"},"Value":{"type":"string"}});
                properties[leaf] = alternative.clone();
                let (object, constraint) = if leaf == "MapValue" {
                    (
                        json!({"type":"object","patternProperties":{".*":alternative},"additionalProperties":false}),
                        json!({"patternProperties":{".*":{"maxLength":10}}}),
                    )
                } else {
                    (
                        json!({"type":"object","properties":properties}),
                        json!({"properties":{leaf:{"maxLength":10}}}),
                    )
                };
                let item = if reversed {
                    json!({"allOf":[constraint,object]})
                } else {
                    json!({"allOf":[object,constraint]})
                };
                let schema = ParsedCfnSchema::from_json(json!({
                    "typeName":"AWS::Test::Tags",
                    "tagging":{"taggable":true,"tagProperty":"/properties/Tags"},
                    "properties":{"Tags":if leaf == "MapValue" { item } else { json!({"type":"array","items":item}) }}
                })).unwrap();
                review602_rejected(&schema, markdown, &["alternative", "tag string"]);
            }
        }
    }
}

#[test]
fn review602_materialized_tag_alternatives_rust() {
    review602_materialized_tag_alternatives(false);
}

#[test]
fn review602_materialized_tag_alternatives_markdown() {
    review602_materialized_tag_alternatives(true);
}

fn review602_missing_required_members(markdown: bool) {
    for requirement in [
        json!({"oneOf":[{"required":["Always","A"]},{"required":["Always","B"]}]}),
        json!({"anyOf":[{"required":["Always","A"]},{"required":["Always","B"]}]}),
        json!({"required":["Always"]}),
    ] {
        let mut object = json!({"type":"object","properties":{
            "A":{"type":"boolean"},"B":{"type":"boolean"}
        }});
        object.as_object_mut().unwrap().extend(requirement.as_object().unwrap().clone());
        for scope in ["inline", "root", "reference", "list", "map"] {
            let schema = match scope {
                "root" => {
                    let mut source = object.clone();
                    source["typeName"] = json!("AWS::Test::Emission");
                    ParsedCfnSchema::from_json(source).unwrap()
                }
                "reference" => ParsedCfnSchema::from_json(json!({
                    "typeName":"AWS::Test::Emission",
                    "properties":{"Probe":{"$ref":"#/definitions/Choice"}},
                    "definitions":{"Choice":object}
                })).unwrap(),
                "list" => fixture(json!({"type":"array","items":object})),
                "map" => fixture(json!({"type":"object","patternProperties":{".*":object},"additionalProperties":false})),
                "inline" => fixture(object.clone()),
                _ => unreachable!(),
            };
            // Root's JSON pointer is the empty string, which is valid.
            let error = review602_render(&schema, markdown).expect_err("missing required member was dropped");
            assert!(matches!(error.downcast_ref::<schema_layer::SchemaError>(),
                Some(schema_layer::SchemaError::Projection { pointer: _, reason })
                    if reason.contains("required") && reason.contains("Always")),
                "{scope}, markdown={markdown}: {error:#}");
        }
    }
}

#[test]
fn review602_missing_required_members_rust() {
    review602_missing_required_members(false);
}

#[test]
fn review602_missing_required_members_markdown() {
    review602_missing_required_members(true);
}

fn review602_filtered_required_members(markdown: bool) {
    for root in [false, true] {
        let object = json!({"type":"object","properties":{
            "Always":{"type":"string","description":"(Deprecated.) Removed."},
            "A":{"type":"boolean"},"B":{"type":"boolean"}
        },"oneOf":[{"required":["Always","A"]},{"required":["Always","B"]}]});
        let schema = if root {
            let mut source = object;
            source["typeName"] = json!("AWS::Test::Emission");
            ParsedCfnSchema::from_json(source).unwrap()
        } else {
            fixture(object)
        };
        let error = review602_render(&schema, markdown).expect_err("deprecated required member was dropped");
        assert!(matches!(error.downcast_ref::<schema_layer::SchemaError>(),
            Some(schema_layer::SchemaError::Projection { pointer: _, reason })
                if reason.contains("required") && reason.contains("Always") && reason.contains("deprecated")),
            "{error:#}");
    }
    let schema = ParsedCfnSchema::from_json(json!({
        "typeName":"AWS::Test::NestedOmission",
        "properties":{"Config":{"$ref":"#/definitions/ParentConfig"}},
        "definitions":{"ParentConfig":{"type":"object","properties":{
            "ChildPolicy":{"type":"object"},"Enabled":{"type":"boolean"}
        },"required":["ChildPolicy"]}}
    })).unwrap();
    review602_rejected(&schema, markdown, &["required", "ChildPolicy", "omitted"]);
}

#[test]
fn review602_filtered_required_members_rust() {
    review602_filtered_required_members(false);
}

#[test]
fn review602_filtered_required_members_markdown() {
    review602_filtered_required_members(true);
}

#[test]
fn review602_vacuous_keyword_one_of_and_not_are_never_discarded() {
    for property in [
        json!({"type":"string","oneOf":[{"minItems":1},{"minItems":2}]}),
        json!({"type":"string","oneOf":[{"minimum":1},{"minimum":2}]}),
        json!({"type":"boolean","oneOf":[{"minLength":1},{"minLength":2}]}),
        json!({"type":"string","oneOf":[{"minProperties":1},{"minProperties":2}]}),
        json!({"type":"string","oneOf":[{"items":false},{"contains":false}]}),
    ] {
        for markdown in [false, true] {
            review602_rejected(&fixture(property.clone()), markdown, &["alternative"]);
        }
    }
    for property in [
        json!({"type":"string","not":{"required":["A"]}}),
        json!({"type":"string","not":{"minItems":1}}),
        json!({"type":"array","contains":{"not":{"required":["A"]}}}),
    ] {
        let error = schema_layer::Layer::from_json(&json!({
            "typeName":"AWS::Test::Emission","properties":{"Probe":property}
        })).expect_err("not must retain a typed diagnostic");
        assert!(error.to_string().contains("not"), "{error}");
    }
}

#[test]
fn review602_nonobject_scope_checks_cover_tag_lowering() {
    use emission_plan::EmittedKind;
    for (property, kinds) in [
        (json!({"type":"string"}), vec![EmittedKind::String, EmittedKind::TagString]),
        (json!({"type":"array","items":{"type":"string"}}), vec![EmittedKind::List, EmittedKind::TagList]),
        (json!({"type":"array","uniqueItems":true,"items":{"type":"string"}}), vec![EmittedKind::Set]),
    ] {
        for keyword in ["oneOf", "anyOf"] {
            let mut property = property.clone();
            property[keyword] = json!([{"required":["A"]},{"required":["B"]}]);
            let schema = fixture(property);
            let projection = schema.properties["Probe"].try_query().unwrap();
            for kind in &kinds {
                let result = emission_plan::lower_as(projection, *kind);
                if keyword == "oneOf" {
                    assert!(matches!(result, Err(schema_layer::SchemaError::Projection { pointer: _, reason })
                        if reason.contains("oneOf") && reason.contains("unsatisfiable")));
                } else {
                    result.unwrap_or_else(|error| panic!("{kind:?}: {error}"));
                }
            }
        }
    }
}

fn review602_pattern_enum_intersection(markdown: bool) {
    let identity = Regex::new(r"ProbeRefined[0-9a-f]{16}").unwrap();
    let anchor = Regex::new(r"proberefined[0-9a-f]{16}").unwrap();
    for (bounds, retained) in [
        (json!({"minLength":2}), vec!["bbb"]),
        (json!({"maxLength":2}), vec!["a"]),
        (json!({"minLength":1,"maxLength":3}), vec!["a", "bbb"]),
    ] {
        let pattern = json!({"type":"string","pattern":"^(a|bbb)$"});
        let mut direct = pattern.clone();
        direct.as_object_mut().unwrap().extend(bounds.as_object().unwrap().clone());
        for property in [
            direct,
            json!({"allOf":[pattern,bounds]}),
            json!({"allOf":[bounds,pattern]}),
            json!({"$ref":"#/definitions/Mode","allOf":[bounds]}),
        ] {
            let schema = ParsedCfnSchema::from_json(json!({
                "typeName":"AWS::Test::Emission", "properties":{"Probe":property},
                "definitions":{"Mode":pattern}
            })).unwrap();
            let narrowed = retained.len() < 2;
            let expected = review602_render(&fixture(json!({"type":"string","enum":retained})), markdown).unwrap();
            let actual = review602_render(&schema, markdown).unwrap();
            assert_eq!(identity.is_match(&actual), narrowed, "refined enum identity: {actual}");
            let normalized = identity.replace_all(&actual, "Probe");
            assert_eq!(anchor.replace_all(&normalized, "probe"), expected, "markdown={markdown}: inferred values must satisfy effective bounds");
        }
    }
    review602_rejected(&fixture(json!({
        "type":"string","pattern":"^(a|bbb)$","minLength":2,"maxLength":2
    })), markdown, &["empty literal intersection"]);
}

#[test]
fn review602_pattern_enum_intersection_rust() {
    review602_pattern_enum_intersection(false);
}

#[test]
fn review602_pattern_enum_intersection_markdown() {
    review602_pattern_enum_intersection(true);
}
