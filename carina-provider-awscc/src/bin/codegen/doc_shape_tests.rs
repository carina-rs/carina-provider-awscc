// Synthetic shapes use independently constructed compiled schemas as the oracle.
use super::*;
use carina_core::schema::{AttributeSchema, ResourceSchema, StructField};
use serde_json::json;

fn check(property: serde_json::Value, expected: AttributeType) -> String {
    let parsed = ParsedCfnSchema::from_json(
        json!({"typeName":"AWS::Test::DocParity", "properties":{"Probe":property}}),
    )
    .unwrap();
    let runtime = ResourceSchema::new("test.DocParity")
        .attribute(AttributeSchema::optional("probe", expected).build())
        .validate()
        .unwrap();
    let markdown = generate_test_markdown(&parsed, &parsed.type_name).unwrap();
    doc_type_parity_tests::check_parity(runtime.as_schema(), &parsed, &markdown).unwrap();
    markdown
}

#[test]
fn scalar_and_collection_shape_matrix_matches_compiled_nodes() {
    let pattern = ApprovedPattern::whole("^[a-z|`]+$").as_spec().clone();
    let scalar = AttributeType::refined_string(None, Some(pattern), Some((Some(2), Some(8))), None);
    for (source, expected) in [
        (json!({"type":"boolean"}), AttributeType::bool()),
        (
            json!({"type":"integer","minimum":3,"maximum":9}),
            AttributeType::refined_int(None, Some((Some(3), Some(9)))),
        ),
        (
            json!({"type":"string","pattern":"^[a-z|`]+$","minLength":2,"maxLength":8}),
            scalar.clone(),
        ),
        (
            json!({"type":"array","insertionOrder":false,"minItems":1,"maxItems":3,"uniqueItems":true,"items":{"type":"string","pattern":"^[a-z|`]+$","minLength":2,"maxLength":8}}),
            AttributeType::refined_list(
                scalar,
                false,
                Some((Some(1), Some(3))),
                ListElements::Unique,
            ),
        ),
        (
            json!({"type":"object","patternProperties":{"^[a-z]+$":{"type":"string","maxLength":12}},"additionalProperties":false,"minProperties":1,"maxProperties":4}),
            AttributeType::refined_map(
                AttributeType::refined_string(
                    None,
                    Some(ApprovedPattern::whole("^[a-z]+$").as_spec().clone()),
                    None,
                    None,
                ),
                AttributeType::refined_string(None, None, Some((None, Some(12))), None),
                Some((Some(1), Some(4))),
            ),
        ),
        (
            json!({"type":"integer","enum":[1,2]}),
            AttributeType::union(vec![
                AttributeType::refined_int(None, Some((Some(1), Some(1)))),
                AttributeType::refined_int(None, Some((Some(2), Some(2)))),
            ]),
        ),
        (
            json!({"type":"object","properties":{},"additionalProperties":false}),
            AttributeType::struct_("Probe", vec![]),
        ),
    ] {
        check(source, expected);
    }
}

#[test]
fn nested_lists_keep_both_levels_and_constraints() {
    let enum_type = AttributeType::enum_(
        carina_core::schema::enum_identity("Probe", Some("aws.test.DocParity")),
        Some(vec!["alpha".into(), "beta".into()]),
        vec![
            ("alpha".into(), "alpha".into()),
            ("beta".into(), "beta".into()),
        ],
        None,
        None,
    );
    check(
        json!({"type":"array","minItems":2,"maxItems":5,"items":{"type":"array","minItems":1,"maxItems":3,"insertionOrder":false,
        "items":{"type":"string","enum":["alpha","beta"]}}}),
        AttributeType::refined_list(
            AttributeType::refined_list(
                enum_type,
                false,
                Some((Some(1), Some(3))),
                ListElements::AllowDuplicates,
            ),
            true,
            Some((Some(2), Some(5))),
            ListElements::AllowDuplicates,
        ),
    );
}

#[test]
fn provider_facts_use_shared_supplemental_placement_in_collections() {
    use carina_docgen::{ScalarDoc, ValidationNote, list_display, scalar_display};
    let markdown = check(
        json!({"type":"array","items":{"type":"integer","format":"int64","multipleOf":10}}),
        AttributeType::list(AttributeType::int()),
    );
    let expected = list_display(
        scalar_display(
            ScalarDoc::Int {
                range: None,
                validate: false,
            },
            None,
        ),
        true,
        carina_docgen::ListConstraints {
            length: None,
            elements: ListElements::AllowDuplicates,
        },
        Some(ValidationNote::new(
            "item: CFN format: int64; item: multiple of 10 (not enforced)",
        )),
    );
    let mut page: PageBuilder<()> = PageBuilder::new();
    expected.write_to(&mut page);
    assert!(markdown.contains(page.finish().unwrap().trim()));

    let pattern = ApprovedPattern::whole("[0-9]+").as_spec().clone();
    let markdown = check(
        json!({"type":"string","pattern":"[0-9]+","maxLength":20}),
        AttributeType::refined_string(None, Some(pattern.clone()), Some((None, Some(20))), None),
    );
    let expected = scalar_display(
        ScalarDoc::String {
            pattern: Some(&pattern),
            length: Some((None, Some(20))),
            validate: false,
            to_dsl: None,
        },
        Some(ValidationNote::new("NumericString")),
    );
    let mut page: PageBuilder<()> = PageBuilder::new();
    expected.write_to(&mut page);
    assert!(markdown.contains(page.finish().unwrap().trim()));
}

#[test]
fn definition_names_with_path_separators_do_not_merge_with_inline_children() {
    let parsed = ParsedCfnSchema::from_json(json!({
        "typeName":"AWS::Test::DocParity",
        "properties":{"Container":{"$ref":"#/definitions/Container"}, "Other":{"$ref":"#/definitions/Container~1child"}},
        "definitions":{
            "Container":{"type":"object","properties":{"Child":{"type":"object","properties":{"Enabled":{"type":"boolean"}}}}},
            "Container/child":{"type":"object","properties":{"Name":{"type":"string"}}}
        }
    })).unwrap();
    let runtime = ResourceSchema::new("test.DocParity")
        .attribute(
            AttributeSchema::optional(
                "container",
                AttributeType::struct_(
                    "Container",
                    vec![
                        StructField::optional(
                            "child",
                            AttributeType::struct_(
                                "Child",
                                vec![
                                    StructField::optional("enabled", AttributeType::bool()).build(),
                                ],
                            ),
                        )
                        .build(),
                    ],
                ),
            )
            .build(),
        )
        .attribute(
            AttributeSchema::optional(
                "other",
                AttributeType::struct_(
                    "Container/child",
                    vec![StructField::optional("name", AttributeType::string()).build()],
                ),
            )
            .build(),
        )
        .validate()
        .unwrap();
    let markdown = generate_test_markdown(&parsed, &parsed.type_name).unwrap();
    doc_type_parity_tests::check_parity(runtime.as_schema(), &parsed, &markdown).unwrap();
}
