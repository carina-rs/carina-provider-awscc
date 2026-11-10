use super::*;
use serde_json::{Value, json};

fn outputs(schema: &ParsedCfnSchema) -> [(&'static str, Result<String>); 2] {
    [
        ("Rust", generate_synthetic_schema_code(schema)),
        (
            "Markdown",
            generate_test_markdown(schema, "AWS::EC2::Route"),
        ),
    ]
}

fn refined_choices(exclusive_first: bool) -> ParsedCfnSchema {
    let exclusive = if exclusive_first { "First" } else { "Last" };
    let inclusive = if exclusive_first { "Last" } else { "First" };
    ParsedCfnSchema::from_json(json!({
        "typeName":"AWS::Test::Emission", "properties":{
            exclusive:{"$ref":"#/definitions/Container", "allOf":[{"properties":{
                "Child":{"oneOf":[{"required":["A"]},{"required":["B"]}]}
            }}]},
            inclusive:{"$ref":"#/definitions/Container", "allOf":[{"properties":{
                "Child":{"anyOf":[{"required":["A"]},{"required":["B"]}]}
            }}]}
        }, "definitions":{
            "Container":{"type":"object","properties":{"Child":{"$ref":"#/definitions/Choice"}}},
            "Choice":{"type":"object","properties":{"A":{"type":"boolean"},"B":{"type":"boolean"}}}
        }
    }))
    .unwrap()
}

#[test]
fn review_r8_definition_keys_distinguish_refinements_with_the_same_source_in_both_orders() {
    for exclusive_first in [false, true] {
        let schema = refined_choices(exclusive_first);
        let reference = |name: &str| {
            schema.properties[name].query().properties.as_ref().unwrap()["Child"]
                .query()
                .reference
                .as_ref()
                .unwrap()
        };
        let first = reference("First");
        let last = reference("Last");
        assert_eq!(
            first.property.query().source_pointer,
            last.property.query().source_pointer
        );
        assert!(
            !schema_projection::equivalent_definition_projection(
                first.property.query(),
                last.property.query()
            )
            .unwrap()
        );
        let key = |reference: &schema_projection::Reference| {
            ResolvedRef {
                name: &reference.name,
                property: &reference.property,
                canonical: &reference.canonical,
            }
            .emission_key(&schema)
            .unwrap()
            .into_owned()
        };
        assert_ne!(key(first), key(last), "exclusive_first={exclusive_first}");
    }
}

#[test]
fn review_r8_definition_outputs_keep_each_refinements_own_group_in_both_orders() {
    let mut failures = Vec::new();
    for exclusive_first in [false, true] {
        let schema = refined_choices(exclusive_first);
        let rust = generate_synthetic_schema_code(&schema).unwrap();
        // Each Choice appears in the root attribute, its registered Container,
        // and its own registered definition.
        for group in ["exactly_one", "at_least_one"] {
            let actual = rust
                .matches(&format!("FieldGroup::{group}([\"a\", \"b\"])"))
                .count();
            if actual != 3 {
                failures.push(format!(
                    "Rust: exclusive_first={exclusive_first}, {group}: {actual} copies"
                ));
            }
        }
        for (name, exclusive) in [("first", exclusive_first), ("last", !exclusive_first)] {
            let attribute = rust
                .split_once(&format!("fn build_{name}_attribute()"))
                .unwrap()
                .1
                .split_once("\n    schema =")
                .unwrap()
                .0;
            let expected = if exclusive {
                "exactly_one"
            } else {
                "at_least_one"
            };
            let forbidden = if exclusive {
                "at_least_one"
            } else {
                "exactly_one"
            };
            if !attribute.contains(&format!("FieldGroup::{expected}"))
                || attribute.contains(&format!("FieldGroup::{forbidden}"))
            {
                failures.push(format!("Rust: exclusive_first={exclusive_first}, {name} inherited the wrong group: {attribute}"));
            }
        }
        let markdown = generate_test_markdown(&schema, "AWS::EC2::Route").unwrap();
        for group in ["Exactly one", "At least one"] {
            let actual = markdown
                .matches(&format!("{group} of `a`, `b` must be specified."))
                .count();
            if actual != 1 {
                failures.push(format!(
                    "Markdown: exclusive_first={exclusive_first}, {group}: {actual} copies"
                ));
            }
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn review_r8_equivalent_refinements_reuse_a_key_across_distinct_sources() {
    let use_site =
        json!({"$ref":"#/definitions/Choice", "anyOf":[{"required":["A"]},{"required":["B"]}]});
    let schema = ParsedCfnSchema::from_json(json!({
        "typeName":"AWS::Test::Emission", "properties":{"First":use_site,"Last":use_site},
        "definitions":{"Choice":{"type":"object","properties":{"A":{"type":"boolean"},"B":{"type":"boolean"}}}}
    })).unwrap();
    let key = |name: &str| {
        let reference = schema.properties[name].query().reference.as_ref().unwrap();
        ResolvedRef {
            name: &reference.name,
            property: &reference.property,
            canonical: &reference.canonical,
        }
        .emission_key(&schema)
        .unwrap()
        .into_owned()
    };
    assert_eq!(key("First"), key("Last"));
    let markdown = generate_test_markdown(&schema, "AWS::EC2::Route").unwrap();
    assert_eq!(
        markdown
            .matches("At least one of `a`, `b` must be specified.")
            .count(),
        1
    );
}

#[test]
fn review_r8_materialized_tag_references_keep_merged_value_length_in_both_outputs() {
    let schema = ParsedCfnSchema::from_json(json!({
        "typeName":"AWS::Test::Emission", "properties":{
            "Settings":{"$ref":"#/definitions/Container","allOf":[{"properties":{
                "Labels":{"properties":{"Value":{"minLength":7}}}
            }}]}
        }, "definitions":{
            "Container":{"type":"object","properties":{"Labels":{"$ref":"#/definitions/Tag"}}},
            "Tag":{"type":"object","properties":{"Key":{"type":"string"},"Value":{"type":"string"}}}
        }
    }))
    .unwrap();
    let mut failures = Vec::new();
    for (renderer, output) in outputs(&schema) {
        let output = output.unwrap();
        let expected = if renderer == "Rust" {
            "Some((Some(7), None))"
        } else {
            "String(len: 7..)"
        };
        if !output.contains(expected) {
            failures.push(format!(
                "{renderer} discarded the effective Value.minLength:7"
            ));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

fn literal_schema(literal: Value, site: &str) -> ParsedCfnSchema {
    let property = match site {
        "direct" => literal.clone(),
        "reference" => json!({"$ref":"#/definitions/Literal"}),
        "list" => json!({"type":"array","items":literal}),
        "referenced-list" => json!({"type":"array","items":{"$ref":"#/definitions/Literal"}}),
        "map" => {
            json!({"type":"object","patternProperties":{".*":literal},"additionalProperties":false})
        }
        "referenced-map" => {
            json!({"type":"object","patternProperties":{".*":{"$ref":"#/definitions/Literal"}},"additionalProperties":false})
        }
        _ => panic!("unknown site {site}"),
    };
    ParsedCfnSchema::from_json(json!({
        "typeName":"AWS::Test::Emission","properties":{"Probe":property},
        "definitions":{"Literal":literal}
    }))
    .unwrap()
}

#[test]
fn review_r8_number_literals_keep_float_types_direct_referenced_and_in_collections() {
    let mut failures = Vec::new();
    for literal in [
        json!({"type":"number","enum":[1,2]}),
        json!({"type":"number","const":1}),
    ] {
        for site in ["direct", "reference", "list", "referenced-list"] {
            for (renderer, output) in outputs(&literal_schema(literal.clone(), site)) {
                let output = output.unwrap();
                let expected = if renderer == "Rust" {
                    "AttributeType::refined_float(None,"
                } else {
                    "FloatEnum([1"
                };
                let forbidden = if renderer == "Rust" {
                    "AttributeType::refined_int("
                } else {
                    "IntEnum("
                };
                if !output.contains(expected) || output.contains(forbidden) {
                    failures.push(format!("{renderer}: {site}: {literal} lost Number"));
                }
                if renderer == "Rust" && !output.contains("FiniteF64::new(1.0)") {
                    failures.push(format!("{renderer}: {site}: missing singleton Float range"));
                }
            }
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn review_r8_number_literals_reject_inexact_float_endpoints() {
    let mut failures = Vec::new();
    for value in [9007199254740993_i64, -9007199254740993, i64::MAX] {
        for literal in [
            json!({"type":"number","enum":[value]}),
            json!({"type":"number","const":value}),
        ] {
            for (renderer, output) in outputs(&literal_schema(literal.clone(), "reference")) {
                let rejected = output.err().is_some_and(|error| error.chain().any(|source| {
                    matches!(source.downcast_ref::<schema_layer::SchemaError>(),
                        Some(schema_layer::SchemaError::Projection { reason, .. }) if reason.contains("exactly representable"))
                }));
                if !rejected {
                    failures.push(format!("{renderer}: {literal} must reject Float rounding"));
                }
            }
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn review_r8_tag_field_defaults_are_typed_errors_in_both_outputs() {
    let mut failures = Vec::new();
    for field in ["Key", "Value"] {
        for site in [
            "tag-object",
            "tag-list",
            "referenced-tag-list",
            "nested-reference",
        ] {
            let mut tag = json!({"type":"object","properties":{"Key":{"type":"string"},"Value":{"type":"string"}}});
            tag["properties"][field]["default"] = json!("annotation");
            let property = match site {
                "tag-object" => json!({"$ref":"#/definitions/Tag"}),
                "tag-list" => json!({"type":"array","items":tag}),
                "referenced-tag-list" => {
                    json!({"type":"array","items":{"$ref":"#/definitions/Tag"}})
                }
                "nested-reference" => {
                    json!({"type":"object","properties":{"Labels":{"$ref":"#/definitions/Tag"}}})
                }
                _ => unreachable!(),
            };
            let name = if site.ends_with("tag-list") {
                "Tags"
            } else {
                "Probe"
            };
            let schema = ParsedCfnSchema::from_json(json!({
                "typeName":"AWS::Test::Emission","properties":{name:property},"definitions":{"Tag":tag}
            })).unwrap();
            for (renderer, output) in outputs(&schema) {
                let rejected = output.as_ref().is_err_and(|error| {
                    let projection = error.chain().find_map(|source| source.downcast_ref::<schema_layer::SchemaError>())
                        .or_else(|| match error.downcast_ref::<TagShapeError>() {
                            Some(TagShapeError::Unreadable { resource: _resource, pointer: _pointer, source }) => Some(source.as_ref()),
                            Some(TagShapeError::NotTag { resource: _resource, pointer: _pointer, reason: _reason }) => None,
                            None => None,
                        });
                    matches!(projection, Some(schema_layer::SchemaError::Projection { pointer, reason })
                        if pointer.ends_with(&format!("/properties/{field}")) && reason.contains("default"))
                });
                if !rejected {
                    failures.push(format!(
                        "{renderer}: {site}: {field}.default was silently discarded: {}",
                        output.err().map_or_else(
                            || "successful emission".into(),
                            |error| format!("{error:#}")
                        )
                    ));
                }
            }
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn review_r8_tag_shape_consumers_name_every_field() {
    // Rest patterns would silently accept future TagShape facts. Guard the
    // production consumers, including branches rarely reached by fixtures.
    let source = include_str!("../codegen.rs")
        .split("\nmod tests {")
        .next()
        .unwrap();
    let patterns = Regex::new(r"TagShape::(?:NotTag|Unreadable)\s*\{([^}]+)\}").unwrap();
    let mut checked = 0;
    for captures in patterns.captures_iter(source) {
        checked += 1;
        assert!(
            !captures[1].contains(".."),
            "non-exhaustive consumer: {}",
            &captures[0]
        );
    }
    assert!(
        checked >= 6,
        "production TagShape consumers must be covered"
    );
}
