use super::*;
use carina_core::schema::{FieldGroup, FieldGroupKind};
use serde_json::json;

fn expected_group(kind: FieldGroupKind, fields: &[&str]) -> String {
    let expected = FieldGroup::new(kind, fields.iter().copied()).unwrap();
    let mut page: PageBuilder<()> = PageBuilder::new();
    carina_docgen::render_field_groups(&mut page, &[expected]);
    page.finish()
        .unwrap()
        .lines()
        .find(|line| line.starts_with("- "))
        .unwrap()
        .to_string()
}

#[test]
fn round2_equivalent_groups_merge_declared_description_membership() {
    let schema = ParsedCfnSchema::from_json(json!({
        "typeName":"AWS::Test::Groups","properties":{
            "A":{"type":"boolean"},"B":{"type":"boolean"},
            "C":{"type":"boolean","description":"Specify either C or D."},"D":{"type":"boolean"}
        },"anyOf":[
            {"anyOf":[{"required":["A"]},{"required":["B"]}]},
            {"anyOf":[{"required":["A"]},{"required":["B"]},{"required":["A","C"]}]}
        ]
    }))
    .unwrap();
    let rust = generate_synthetic_schema_code(&schema).unwrap();
    assert_eq!(
        rust.matches("FieldGroup::at_least_one").count(),
        1,
        "{rust}"
    );
    assert!(!rust.contains("FieldGroup::exactly_one"), "{rust}");
    let markdown = generate_test_markdown(&schema, "AWS::EC2::Route").unwrap();
    assert!(
        markdown.contains(&expected_group(FieldGroupKind::AtLeastOne, &["a", "b"])),
        "{markdown}"
    );
    assert!(!markdown.contains("Exactly one of"), "{markdown}");
}

#[test]
fn round2_simplified_declarations_suppress_overlapping_description_groups() {
    for (arms, description) in [
        (
            json!([{"required":["A"]},{"required":[]}]),
            "Specify either A or B.",
        ),
        (
            json!([{"required":["A"]},{"required":["A","C"]}]),
            "Specify either A or B.",
        ),
        (
            json!([{"required":["A"]},{"required":["A","C"]}]),
            "Specify either C or B.",
        ),
    ] {
        let schema = ParsedCfnSchema::from_json(json!({
            "typeName":"AWS::Test::Groups","properties":{
                "A":{"type":"boolean","description":description},
                "B":{"type":"boolean"},"C":{"type":"boolean"}
            },"allOf":[{"anyOf":arms}]
        }))
        .unwrap();
        let rust = generate_synthetic_schema_code(&schema).unwrap();
        assert!(!rust.contains("FieldGroup::exactly_one"), "{rust}");
        let markdown = generate_test_markdown(&schema, "AWS::EC2::Route").unwrap();
        assert!(!markdown.contains("Exactly one of"), "{markdown}");
    }
}

#[test]
fn round1_definition_identity_includes_descendant_groups_in_both_orders() {
    for local_first in [false, true] {
        let local = if local_first { "A" } else { "Z" };
        let shared = if local_first { "Z" } else { "A" };
        let schema = ParsedCfnSchema::from_json(json!({
            "typeName":"AWS::Test::Groups", "properties":{
                local:{"$ref":"#/definitions/Container","properties":{"Child":{
                    "anyOf":[{"required":["A"]},{"required":["B"]}]
                }}},
                shared:{"$ref":"#/definitions/Container"}
            }, "definitions":{"Container":{"type":"object","properties":{
                "Child":{"type":"object","properties":{"A":{"type":"boolean"},"B":{"type":"boolean"}}}
            }}}
        })).unwrap();
        let local_ref = schema.properties[local].query().reference.as_ref().unwrap();
        let shared_ref = schema.properties[shared]
            .query()
            .reference
            .as_ref()
            .unwrap();
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
        assert_ne!(key(local_ref), key(shared_ref));
        let rust = generate_synthetic_schema_code(&schema).unwrap();
        assert_eq!(
            rust.matches("FieldGroup::at_least_one([\"a\", \"b\"])")
                .count(),
            2,
            "{rust}"
        );
        let markdown = generate_test_markdown(&schema, "AWS::EC2::Route").unwrap();
        assert_eq!(
            markdown
                .matches(&expected_group(FieldGroupKind::AtLeastOne, &["a", "b"]))
                .count(),
            1,
            "{markdown}"
        );
    }
}

#[test]
fn round1_group_members_must_be_emitted_optional_and_without_defaults() {
    for scope in ["root", "inline", "reference", "list", "map"] {
        for invalid in ["deprecated", "defaulted", "provider_populated"] {
            let mut object = json!({"type":"object","properties":{
                "A":{"type":"boolean"},"B":{"type":"boolean"}
            },"anyOf":[{"required":["A"]},{"required":["B"]}]});
            if invalid == "deprecated" {
                object["properties"]["A"]["description"] = json!("(Deprecated.) Removed.");
            }
            if invalid == "defaulted" {
                object["properties"]["A"]["default"] = json!(false);
            }
            let mut source = json!({"typeName":"AWS::Test::Groups"});
            let member_path = match scope {
                "root" => {
                    source["properties"] = object["properties"].clone();
                    source["anyOf"] = object["anyOf"].clone();
                    "/properties/A"
                }
                "inline" => {
                    source["properties"] = json!({"Parent":object});
                    "/properties/Parent/A"
                }
                "reference" => {
                    source["properties"] = json!({"Parent":{"$ref":"#/definitions/Choice"}});
                    source["definitions"] = json!({"Choice":object});
                    "/properties/Parent/A"
                }
                "list" => {
                    source["properties"] = json!({"Parent":{"type":"array","items":object}});
                    "/properties/Parent/*/A"
                }
                "map" => {
                    source["properties"] =
                        json!({"Parent":{"type":"object","patternProperties":{".*":object}}});
                    "/properties/Parent/*/A"
                }
                unsupported => panic!("unexpected scope {unsupported}"),
            };
            if invalid == "provider_populated" {
                source["readOnlyProperties"] = json!([member_path]);
            }
            let schema = ParsedCfnSchema::from_json(source).unwrap();
            for result in [
                generate_synthetic_schema_code(&schema),
                generate_test_markdown(&schema, "AWS::EC2::Route"),
            ] {
                let error = result.expect_err(&format!("{scope}: {invalid}"));
                assert!(
                    matches!(error.downcast_ref::<schema_layer::SchemaError>(),
                    Some(schema_layer::SchemaError::Projection { pointer: _, reason })
                    if reason.contains("member A") && reason.contains(invalid)),
                    "{scope}: {invalid}: {error:#}"
                );
            }
        }
    }
}

#[test]
fn round1_group_admission_checks_each_reference_use() {
    let schema = ParsedCfnSchema::from_json(json!({
        "typeName":"AWS::Test::Groups", "properties":{
            "First":{"$ref":"#/definitions/Choice"},
            "Second":{"$ref":"#/definitions/Choice"}
        }, "definitions":{"Choice":{"type":"object","properties":{
            "A":{"type":"boolean"},"B":{"type":"boolean"}
        },"anyOf":[{"required":["A"]},{"required":["B"]}]}},
        "readOnlyProperties":["/properties/Second/A"]
    }))
    .unwrap();
    for result in [
        generate_synthetic_schema_code(&schema),
        generate_test_markdown(&schema, "AWS::EC2::Route"),
    ] {
        let error = result.expect_err("the second use has a provider-populated group member");
        assert!(
            matches!(error.downcast_ref::<schema_layer::SchemaError>(),
            Some(schema_layer::SchemaError::Projection { pointer: _, reason })
            if reason.contains("member A") && reason.contains("provider_populated")),
            "{error:#}"
        );
    }
}

#[test]
fn round1_group_admission_checks_override_defaults_and_provider_ancestors() {
    static OVERRIDES: LazyLock<ResourceTypeOverrides> = LazyLock::new(|| {
        let mut overrides = ResourceTypeOverrides::default();
        overrides.insert(
            ("AWS::Test::Groups", "A"),
            NarrowingKind::ArrayItemType {
                expression: ConstraintSpec::boolean(),
                block_name: None,
                default: Some(AttributeTypeOverrideDefault {
                    value: OverrideDefaultValue::EmptyList,
                    markdown: "[]",
                }),
                min_items: None,
                max_items: None,
            },
        );
        overrides
    });
    let object = json!({"type":"object","properties":{
        "A":{"type":"array","items":{"type":"boolean"}},"B":{"type":"boolean"}
    },"anyOf":[{"required":["A"]},{"required":["B"]}]});
    let root = ParsedCfnSchema::from_json(json!({
        "typeName":"AWS::Test::Groups","properties":object["properties"],"anyOf":object["anyOf"]
    }))
    .unwrap();
    with_resource_type_overrides(&OVERRIDES, || {
        for result in [
            generate_synthetic_schema_code(&root),
            generate_test_markdown(&root, "AWS::EC2::Route"),
        ] {
            let error = result.expect_err("override defaults participate in group admission");
            assert!(
                matches!(error.downcast_ref::<schema_layer::SchemaError>(),
                Some(schema_layer::SchemaError::Projection { pointer: _, reason })
                if reason.contains("member A") && reason.contains("defaulted")),
                "{error:#}"
            );
        }
    });
    let nested = ParsedCfnSchema::from_json(json!({
        "typeName":"AWS::Test::Groups","properties":{"Parent":object},
        "readOnlyProperties":["/properties/Parent"]
    }))
    .unwrap();
    for result in [
        generate_synthetic_schema_code(&nested),
        generate_test_markdown(&nested, "AWS::EC2::Route"),
    ] {
        let error =
            result.expect_err("a provider-populated ancestor changes effective member modes");
        assert!(
            matches!(error.downcast_ref::<schema_layer::SchemaError>(),
            Some(schema_layer::SchemaError::Projection { pointer: _, reason })
            if reason.contains("member A") && reason.contains("provider_populated")),
            "{error:#}"
        );
    }
}

#[test]
fn round1_schema_groups_take_precedence_over_overlapping_descriptions() {
    for description in ["Specify either A or B.", "Specify either B or D."] {
        let schema = ParsedCfnSchema::from_json(json!({
            "typeName":"AWS::Test::Groups", "properties":{
                "A":{"type":"boolean","description":description},"B":{"type":"boolean"},
                "C":{"type":"boolean"},"D":{"type":"boolean"}
            },"anyOf":[{"required":["A"]},{"required":["B"]},{"required":["C"]}]
        }))
        .unwrap();
        let rust = generate_synthetic_schema_code(&schema).unwrap();
        assert_eq!(rust.matches("FieldGroup::at_least_one").count(), 1);
        assert!(!rust.contains("FieldGroup::exactly_one"), "{rust}");
        let markdown = generate_test_markdown(&schema, "AWS::EC2::Route").unwrap();
        assert!(markdown.contains(&expected_group(
            FieldGroupKind::AtLeastOne,
            &["a", "b", "c"]
        )));
        assert!(!markdown.contains("Exactly one of"), "{markdown}");
    }
}

#[test]
fn step6_cached_field_groups_reach_every_rust_copy_and_markdown() {
    for (resource, constructor, members, copies, note) in [
        (
            "AWS::CloudWatch::Alarm",
            "exactly_one",
            "\"sliding_window\", \"wall_clock_window\"",
            2,
            &expected_group(
                FieldGroupKind::ExactlyOne,
                &["sliding_window", "wall_clock_window"],
            ),
        ),
        (
            "AWS::DynamoDB::Table",
            "at_least_one",
            "\"read_units_per_second\", \"write_units_per_second\"",
            2,
            &expected_group(
                FieldGroupKind::AtLeastOne,
                &["read_units_per_second", "write_units_per_second"],
            ),
        ),
        (
            "AWS::ElasticLoadBalancingV2::Listener",
            "at_least_one",
            "\"client_secret\", \"use_existing_client_secret\"",
            2,
            &expected_group(
                FieldGroupKind::AtLeastOne,
                &["client_secret", "use_existing_client_secret"],
            ),
        ),
        (
            "AWS::S3::Bucket",
            "exactly_one",
            "\"partitioned_prefix\", \"simple_prefix\"",
            3,
            &expected_group(
                FieldGroupKind::ExactlyOne,
                &["partitioned_prefix", "simple_prefix"],
            ),
        ),
    ] {
        let schema = cfn_schema_for_codegen_tests(resource);
        let rust = generate_schema_code(&schema, resource).unwrap();
        let group = format!("FieldGroup::{constructor}([{members}])");
        assert_eq!(rust.matches(&group).count(), copies, "{resource}");
        assert_eq!(
            rust.matches("struct_with_field_groups").count(),
            copies,
            "{resource}"
        );
        let markdown = generate_test_markdown(&schema, resource).unwrap();
        assert_eq!(markdown.matches(note).count(), 1, "{resource}");
    }
}

fn group_fixture(keyword: &str) -> ParsedCfnSchema {
    let grouped = json!({"type":"object", "properties":{
        "A":{"type":"boolean"}, "B":{"type":"boolean"}, "Always":{"type":"string"}
    }, "required":["Always"], keyword:[{"required":["A"]},{"required":["B"]}]});
    ParsedCfnSchema::from_json(json!({
        "typeName":"AWS::Test::Groups", "properties":{
            "Inline":grouped,
            "Named":{"$ref":"#/definitions/Choice"},
            "List":{"type":"array","items":{"$ref":"#/definitions/Choice"}},
            "Map":{"type":"object","patternProperties":{".*":{"$ref":"#/definitions/Choice"}}}
        }, "definitions":{"Choice":grouped}
    }))
    .unwrap()
}

#[test]
fn step6_field_groups_follow_inline_named_lists_maps_and_refs() {
    for (keyword, constructor, note) in [
        (
            "oneOf",
            "exactly_one",
            &expected_group(FieldGroupKind::ExactlyOne, &["a", "b"]),
        ),
        (
            "anyOf",
            "at_least_one",
            &expected_group(FieldGroupKind::AtLeastOne, &["a", "b"]),
        ),
    ] {
        let schema = group_fixture(keyword);
        let rust = generate_synthetic_schema_code(&schema).unwrap();
        let copies = rust.matches("struct_with_field_groups").count();
        assert!(
            copies >= 4,
            "inline, named, list, map and registered copies: {rust}"
        );
        assert_eq!(
            rust.matches(&format!("FieldGroup::{constructor}([\"a\", \"b\"])"))
                .count(),
            copies
        );
        assert_eq!(
            rust.matches("StructField::required(\"always\"").count(),
            copies
        );
        assert_eq!(rust.matches("StructField::optional(\"a\"").count(), copies);
        assert_eq!(rust.matches("StructField::optional(\"b\"").count(), copies);
        let markdown = generate_test_markdown(&schema, "AWS::EC2::Route").unwrap();
        assert_eq!(
            markdown.matches(note).count(),
            2,
            "inline and shared definition: {markdown}"
        );
    }
}

#[test]
fn step6_root_anyof_and_oneof_have_distinct_rules() {
    for (keyword, constructor, note) in [
        ("oneOf", "exactly_one", "Exactly one"),
        ("anyOf", "at_least_one", "At least one"),
    ] {
        let schema = ParsedCfnSchema::from_json(json!({
            "typeName":"AWS::Test::Groups", "properties":{
                "A":{"type":"boolean"}, "B":{"type":"boolean"}, "Always":{"type":"string"}
            }, "required":["Always"], keyword:[{"required":["A"]},{"required":["B"]}]
        }))
        .unwrap();
        let rust = generate_synthetic_schema_code(&schema).unwrap();
        assert!(
            rust.contains(&format!(
                "schema.field_group(FieldGroup::{constructor}([\"a\", \"b\"])"
            )),
            "{rust}"
        );
        assert!(rust.contains("AttributeSchema::required(\"always\""));
        let markdown = generate_test_markdown(&schema, "AWS::EC2::Route").unwrap();
        assert!(
            markdown.contains(&expected_group(
                if note == "Exactly one" {
                    FieldGroupKind::ExactlyOne
                } else {
                    FieldGroupKind::AtLeastOne
                },
                &["a", "b"]
            )),
            "{markdown}"
        );
    }
}

#[test]
fn step6_unsupported_multifield_arms_are_typed_errors_in_both_renderers() {
    for keyword in ["oneOf", "anyOf"] {
        for nested in [false, true] {
            let object = json!({"type":"object", "properties":{
                "A":{"type":"boolean"}, "B":{"type":"boolean"}, "C":{"type":"boolean"}
            }, keyword:[{"required":["A","B"]},{"required":["C"]}]});
            let mut source = json!({"typeName":"AWS::Test::Groups"});
            if nested {
                source["properties"] = json!({"Parent":object});
            } else {
                source["properties"] = object["properties"].clone();
                source[keyword] = object[keyword].clone();
            }
            let schema = ParsedCfnSchema::from_json(source).unwrap();
            for result in [
                generate_synthetic_schema_code(&schema),
                generate_test_markdown(&schema, "AWS::EC2::Route"),
            ] {
                let error = result.expect_err("multi-field arms cannot be flattened");
                assert!(
                    matches!(error.downcast_ref::<schema_layer::SchemaError>(),
                    Some(schema_layer::SchemaError::Projection { pointer, reason })
                    if pointer == if nested { "/properties/Parent" } else { "" }
                        && reason.contains("unsupported required alternatives")),
                    "{error:#}"
                );
            }
        }
    }
}

#[test]
fn step6_schema_declared_anyof_wins_over_description_exactly_one() {
    let schema = ParsedCfnSchema::from_json(json!({
        "typeName":"AWS::Test::Groups", "properties":{
            "A":{"type":"boolean", "description":"Specify either A or B."}, "B":{"type":"boolean"}
        }, "anyOf":[{"required":["A"]},{"required":["B"]}]
    }))
    .unwrap();
    let rust = generate_synthetic_schema_code(&schema).unwrap();
    assert_eq!(rust.matches("FieldGroup::at_least_one").count(), 1);
    assert!(!rust.contains("FieldGroup::exactly_one"));
}

#[test]
fn step6_handler_and_relationship_alternatives_are_not_field_groups() {
    let schema = cfn_schema_for_codegen_tests("AWS::ElasticLoadBalancingV2::Listener");
    let rust = generate_schema_code(&schema, &schema.type_name).unwrap();
    assert!(!rust.contains("schema.field_group("));
    assert!(!rust.contains("FieldGroup::exactly_one("));
    assert_eq!(rust.matches("FieldGroup::at_least_one(").count(), 2);
}

#[test]
fn step6_field_groups_preserve_alias_siblings_allof_and_recursive_defs() {
    let schema = ParsedCfnSchema::from_json(json!({
        "typeName":"AWS::Test::Groups", "properties":{
            "Parent":{"$ref":"#/definitions/Alias"}
        }, "definitions":{
            "Alias":{"$ref":"#/definitions/Choice", "allOf":[{
                "anyOf":[{"required":["C"]},{"required":["D"]}]
            }]},
            "Choice":{"type":"object", "properties":{
                "A":{"type":"boolean"}, "B":{"type":"boolean"},
                "C":{"type":"boolean"}, "D":{"type":"boolean"},
                "Child":{"$ref":"#/definitions/Choice"}
            }, "oneOf":[{"required":["A"]},{"required":["B"]}]}
        }
    }))
    .unwrap();
    let rust = generate_synthetic_schema_code(&schema).unwrap();
    assert!(rust.contains("AttributeType::ref_(\"Choice\""), "{rust}");
    assert!(rust.contains("FieldGroup::exactly_one([\"a\", \"b\"])"));
    assert!(rust.contains("FieldGroup::at_least_one([\"c\", \"d\"])"));
    let markdown = generate_test_markdown(&schema, "AWS::EC2::Route").unwrap();
    assert!(markdown.contains(&expected_group(FieldGroupKind::ExactlyOne, &["a", "b"])));
    assert!(markdown.contains(&expected_group(FieldGroupKind::AtLeastOne, &["c", "d"])));
}

#[test]
fn step6_multifield_arms_factor_only_equivalent_common_requirements() {
    for (keyword, constructor) in [("oneOf", "exactly_one"), ("anyOf", "at_least_one")] {
        for object in [
            json!({"type":"object", "properties":{
                "A":{"type":"boolean"}, "B":{"type":"boolean"}, "Always":{"type":"string"}
            }, keyword:[{"required":["Always","A"]},{"required":["Always","B"]}]}),
            json!({"type":"object", keyword:[
                {"properties":{"A":{"type":"boolean"},"B":{"type":"boolean"},"Always":{"type":"string"}},"required":["Always","A"]},
                {"properties":{"A":{"type":"boolean"},"B":{"type":"boolean"},"Always":{"type":"string"}},"required":["Always","B"]}
            ]}),
        ] {
            let schema = ParsedCfnSchema::from_json(json!({
                "typeName":"AWS::Test::Groups", "properties":{"Parent":object}
            }))
            .unwrap();
            let rust = generate_synthetic_schema_code(&schema).unwrap();
            assert!(rust.contains(&format!("FieldGroup::{constructor}([\"a\", \"b\"])")));
            assert!(rust.contains("StructField::required(\"always\""));
            let markdown = generate_test_markdown(&schema, "AWS::EC2::Route").unwrap();
            assert!(
                markdown.contains("| `always` | `String` | Yes |"),
                "{markdown}"
            );
        }
    }
}

#[test]
fn step6_reference_site_groups_do_not_change_shared_or_recursive_definitions() {
    for local_first in [false, true] {
        let local = if local_first { "A" } else { "Z" };
        let shared = if local_first { "Z" } else { "A" };
        let schema = ParsedCfnSchema::from_json(json!({
            "typeName":"AWS::Test::Groups", "properties":{
                local:{"$ref":"#/definitions/Choice", "anyOf":[{"required":["C"]},{"required":["D"]}]},
                shared:{"$ref":"#/definitions/Choice"}
            }, "definitions":{"Choice":{
                "type":"object", "properties":{
                    "A":{"type":"boolean"}, "B":{"type":"boolean"},
                    "C":{"type":"boolean"}, "D":{"type":"boolean"},
                    "Child":{"$ref":"#/definitions/Choice"}
                }, "oneOf":[{"required":["A"]},{"required":["B"]}]
            }}
        })).unwrap();
        let rust = generate_synthetic_schema_code(&schema).unwrap();
        assert_eq!(
            rust.matches("FieldGroup::at_least_one([\"c\", \"d\"])")
                .count(),
            2,
            "{rust}"
        );
        assert!(rust.contains("schema.with_def(\"Choice\""));
        let canonical = rust
            .split_once("fn build_schema_def_0()")
            .unwrap()
            .1
            .split_once("schema = schema.with_def")
            .unwrap()
            .0;
        assert!(
            !canonical.contains("FieldGroup::at_least_one"),
            "{canonical}"
        );
        let markdown = generate_test_markdown(&schema, "AWS::EC2::Route").unwrap();
        assert_eq!(
            markdown
                .matches(&expected_group(FieldGroupKind::AtLeastOne, &["c", "d"]))
                .count(),
            1,
            "{markdown}"
        );
        assert_eq!(
            markdown
                .matches(&expected_group(FieldGroupKind::ExactlyOne, &["a", "b"]))
                .count(),
            2,
            "{markdown}"
        );
    }
}
