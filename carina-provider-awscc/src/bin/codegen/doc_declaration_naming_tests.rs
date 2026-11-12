use super::doc_test_support::{
    declaration_page, emission, link_target, markdown, page, root_link, targets,
};
use super::*;
use carina_core::schema::StructField;
use serde_json::json;

#[test]
fn hidden_attributes_do_not_make_definitions_shared() {
    let md = markdown(json!({
        "typeName":"AWS::Test::HiddenUsage",
        "properties":{
            "VisibleSettings":{"$ref":"#/definitions/SharedOptions"},
            "OldSettings":{"$ref":"#/definitions/SharedOptions","description":"(Deprecated.) Use VisibleSettings."}
        },
        "definitions":{"SharedOptions":{"type":"object","properties":{"Enabled":{"type":"boolean"}}}}
    }));
    assert!(md.contains("### visible_settings\n\n| Field |"), "{md}");
    assert!(!md.contains("### shared_options\n"), "{md}");
    assert!(!md.contains("Used by:"), "{md}");
}

#[test]
fn literal_field_names_are_not_collection_edges() {
    let md = markdown(json!({
        "typeName":"AWS::Test::LiteralFields",
        "properties":{"Container":{"type":"object","properties":{
            "Value":{"type":"object","properties":{"Enabled":{"type":"boolean"}}},
            "Item":{"type":"object","properties":{"Enabled":{"type":"boolean"}}},
            "Key":{"type":"object","properties":{"Enabled":{"type":"boolean"}}}
        }}}
    }));
    for name in ["value", "item", "key"] {
        assert!(md.contains(&format!("### container.{name}\n")), "{md}");
    }
    assert!(!md.contains("container.<key>"), "{md}");
}

#[test]
fn shared_headings_follow_emitted_definition_keys() {
    let definition = |limit| {
        EmittedType::new(
            "fixture",
            AttributeType::struct_(
                "SameName",
                vec![
                    StructField::optional(
                        "count",
                        AttributeType::refined_int(None, Some((None, Some(limit)))),
                    )
                    .build(),
                ],
            ),
        )
    };
    let resource = emission(
        BTreeMap::from([
            ("left_a".into(), EmittedType::reference("LeftDefinition")),
            ("left_b".into(), EmittedType::reference("LeftDefinition")),
            ("right_a".into(), EmittedType::reference("RightDefinition")),
            ("right_b".into(), EmittedType::reference("RightDefinition")),
        ]),
        BTreeMap::from([
            ("LeftDefinition".into(), definition(3)),
            ("RightDefinition".into(), definition(9)),
        ]),
    );
    let first = declaration_page(&resource).unwrap();
    assert!(first.contains("### left_definition\n"), "{first}");
    assert!(first.contains("### right_definition\n"), "{first}");
    assert_eq!(first, declaration_page(&resource).unwrap());
}

#[test]
fn heading_and_used_by_share_visible_usages() {
    let mut outer = EmittedType::new(
        "fixture",
        AttributeType::struct_(
            "Outer",
            vec![StructField::optional("inner_field", AttributeType::ref_("Inner")).build()],
        ),
    );
    outer.annotations.definition = Some("Outer".into());
    let resource = emission(
        BTreeMap::from([("left".into(), outer.clone()), ("right".into(), outer)]),
        BTreeMap::from([(
            "Inner".into(),
            EmittedType::new(
                "fixture",
                AttributeType::struct_(
                    "Inner",
                    vec![StructField::optional("enabled", AttributeType::bool()).build()],
                ),
            ),
        )]),
    );
    let md = declaration_page(&resource).unwrap();
    assert!(md.contains("### outer\n\nUsed by: `left`, `right`"), "{md}");
    assert!(md.contains("### outer.inner_field\n\n| Field |"), "{md}");
    assert!(!md.contains("### inner\n"), "{md}");
    assert!(!md.contains("Used by: `outer.inner_field`"), "{md}");
}

#[test]
fn identical_struct_and_union_bodies_share_one_table() {
    for (ty, header) in [
        (
            AttributeType::struct_(
                "Settings",
                vec![StructField::optional("enabled", AttributeType::bool()).build()],
            ),
            "| Field |",
        ),
        (
            AttributeType::union(vec![AttributeType::bool(), AttributeType::string()]),
            "| Member |",
        ),
    ] {
        let emitted = EmittedType::new("fixture", ty);
        let md = page(&emission(
            BTreeMap::from([("left".into(), emitted.clone()), ("right".into(), emitted)]),
            BTreeMap::new(),
        ))
        .unwrap();
        assert_eq!(md.matches(header).count(), 1, "{md}");
        assert!(md.contains("Used by: `left`, `right`"), "{md}");
    }
}

#[test]
fn shared_enums_and_unions_show_used_by() {
    for ty in [
        AttributeType::enum_(
            carina_core::schema::enum_identity("Mode", Some("aws.test.Round2")),
            Some(vec!["a".into()]),
            vec![],
            None,
            None,
        ),
        AttributeType::union(vec![AttributeType::bool(), AttributeType::string()]),
    ] {
        let emitted = EmittedType::new("fixture", ty);
        let md = page(&emission(
            BTreeMap::from([("left".into(), emitted.clone()), ("right".into(), emitted)]),
            BTreeMap::new(),
        ))
        .unwrap();
        assert!(md.contains("Used by: `left`, `right`"), "{md}");
    }
}

#[test]
fn union_collection_labels_follow_visible_headings() {
    let union = || AttributeType::union(vec![AttributeType::bool(), AttributeType::string()]);
    for (ty, scope) in [
        (AttributeType::list(union()), "probe"),
        (
            AttributeType::map(AttributeType::map(union())),
            "probe.<value>.<value>",
        ),
    ] {
        let md = page(&emission(
            BTreeMap::from([("probe".into(), EmittedType::new("fixture", ty))]),
            BTreeMap::new(),
        ))
        .unwrap();
        let md = md.replace("\\<", "<").replace("\\>", ">");
        assert!(md.contains(&format!("[Union({scope})]")), "{md}");
        assert!(md.contains(&format!("### {scope}\n")), "{md}");
    }
}

#[test]
fn shared_headings_and_anchors_do_not_depend_on_sibling_uses() {
    for ty in [
        AttributeType::union(vec![AttributeType::bool(), AttributeType::string()]),
        AttributeType::struct_(
            "Settings",
            vec![StructField::optional("enabled", AttributeType::bool()).build()],
        ),
        AttributeType::enum_(
            carina_core::schema::enum_identity("Mode", Some("aws.test.Round3")),
            Some(vec!["a".into()]),
            vec![],
            None,
            None,
        ),
    ] {
        let value = EmittedType::new("fixture", ty);
        let mut attributes =
            BTreeMap::from([("b".into(), value.clone()), ("c".into(), value.clone())]);
        let before = page(&emission(attributes.clone(), BTreeMap::new())).unwrap();
        attributes.insert("a".into(), value);
        attributes.insert(
            "unrelated".into(),
            EmittedType::new(
                "fixture",
                AttributeType::union(vec![AttributeType::float(), AttributeType::bool()]),
            ),
        );
        let after = page(&emission(attributes, BTreeMap::new())).unwrap();
        assert_eq!(
            root_link(&before, "b"),
            root_link(&after, "b"),
            "adding a sibling use moved a shared declaration"
        );
    }
}

#[test]
fn top_level_and_nested_union_labels_use_the_final_heading() {
    let union = AttributeType::union(vec![AttributeType::bool(), AttributeType::string()]);
    let resource = emission(
        BTreeMap::from([
            (
                "a".into(),
                EmittedType::new(
                    "fixture",
                    AttributeType::struct_(
                        "Outer",
                        vec![StructField::optional("x", union.clone()).build()],
                    ),
                ),
            ),
            ("b".into(), EmittedType::new("fixture", union)),
        ]),
        BTreeMap::new(),
    );
    let md = page(&resource).unwrap();
    let (label, _, heading) = root_link(&md, "b");
    assert_eq!(label, format!("Union({heading})"), "{md}");
    assert!(md.contains("Used by: `a.x`, `b`"), "{md}");
}

#[test]
fn member_declarations_use_their_names_in_headings_and_links() {
    let principal = AttributeType::struct_(
        "Principal",
        vec![StructField::optional("aws", AttributeType::string()).build()],
    );
    for ty in [principal.clone(), AttributeType::list(principal)] {
        let resource = emission(
            BTreeMap::from([(
                "probe".into(),
                EmittedType::new(
                    "fixture",
                    AttributeType::union(vec![ty, AttributeType::string()]),
                ),
            )]),
            BTreeMap::new(),
        );
        let md = page(&resource).unwrap();
        let target = link_target(&md, "Struct(Principal)");
        assert_eq!(targets(&md)[&target], "principal", "{md}");
        let parsed = ParsedCfnSchema::from_json(
            serde_json::json!({"typeName":"AWS::Test::Round4", "properties":{}}),
        )
        .unwrap();
        doc_type_parity_tests::check_parity(resource.witness.as_schema(), &parsed, &md).unwrap();
    }
    let inner = AttributeType::union(vec![AttributeType::bool(), AttributeType::string()]);
    let resource = emission(
        BTreeMap::from([(
            "probe".into(),
            EmittedType::new(
                "fixture",
                AttributeType::union(vec![inner, AttributeType::int()]),
            ),
        )]),
        BTreeMap::new(),
    );
    let md = page(&resource).unwrap();
    let name = "union_of_bool_or_string";
    let target = link_target(&md, &format!("Union({name})"));
    assert_eq!(targets(&md)[&target], name, "{md}");
}

#[test]
fn single_use_same_named_struct_does_not_renumber_shared_heading() {
    let value = EmittedType::new(
        "fixture",
        AttributeType::struct_(
            "Settings",
            vec![StructField::optional("enabled", AttributeType::bool()).build()],
        ),
    );
    let mut attributes = BTreeMap::from([("left".into(), value.clone()), ("right".into(), value)]);
    let before = page(&emission(attributes.clone(), BTreeMap::new())).unwrap();
    attributes.insert(
        "unrelated".into(),
        EmittedType::new(
            "fixture",
            AttributeType::struct_(
                "Settings",
                vec![StructField::optional("count", AttributeType::int()).build()],
            ),
        ),
    );
    let after = page(&emission(attributes, BTreeMap::new())).unwrap();
    let before_target = link_target(&before, "Struct(Settings)");
    let after_target = link_target(&after, "Struct(Settings)");
    assert_eq!(
        (&before_target, &targets(&before)[&before_target]),
        (&after_target, &targets(&after)[&after_target]),
        "{before}\n{after}"
    );
}

#[test]
fn literal_member_field_is_a_dsl_path() {
    let resource = emission(
        BTreeMap::from([(
            "probe".into(),
            EmittedType::new(
                "fixture",
                AttributeType::struct_(
                    "Outer",
                    vec![
                        StructField::optional(
                            "member_0",
                            AttributeType::struct_("Settings", vec![]),
                        )
                        .build(),
                    ],
                ),
            ),
        )]),
        BTreeMap::new(),
    );
    let md = page(&resource).unwrap();
    let target = link_target(&md, "Struct(Settings)");
    assert_eq!(targets(&md)[&target], "probe.member_0", "{md}");
}

#[test]
fn shared_headings_are_readable_without_hash_suffixes() {
    let parsed = cfn_schema_for_codegen_tests("AWS::IAM::Role");
    let md = generate_test_markdown(&parsed, &parsed.type_name).unwrap();
    let hash = regex::Regex::new(r"_[0-9a-f]{16}(?:\b|_)").unwrap();
    let opaque = md
        .lines()
        .filter(|line| line.starts_with('#') && hash.is_match(line))
        .collect::<Vec<_>>();
    assert!(opaque.is_empty(), "hash-bearing headings: {opaque:?}");
    assert!(md.contains("### statement\n"), "{md}");
    assert!(
        md.contains("### union_of_string_or_list_of_string\n"),
        "{md}"
    );
}

#[test]
fn readable_heading_ordinals_are_local_and_stably_ordered() {
    let mut attributes = BTreeMap::new();
    for (field, limit) in [("left", 3), ("right", 9)] {
        let value = EmittedType::new(
            "fixture",
            AttributeType::struct_(
                "Settings",
                vec![
                    StructField::optional(
                        "count",
                        AttributeType::refined_int(None, Some((Some(1), Some(limit)))),
                    )
                    .build(),
                ],
            ),
        );
        attributes.insert(format!("{field}_a"), value.clone());
        attributes.insert(format!("{field}_b"), value);
    }
    let before = page(&emission(attributes.clone(), BTreeMap::new())).unwrap();
    assert!(before.contains("### settings_1\n"), "{before}");
    assert!(before.contains("### settings_2\n"), "{before}");
    for field in ["unrelated_a", "unrelated_b"] {
        attributes.insert(
            field.into(),
            EmittedType::new("fixture", AttributeType::struct_("Other", vec![])),
        );
    }
    let after = page(&emission(attributes, BTreeMap::new())).unwrap();
    let declaration = |md: &str, heading: &str| {
        md.split(&format!("### {heading}\n"))
            .nth(1)
            .unwrap()
            .split("\n### ")
            .next()
            .unwrap()
            .trim()
            .to_owned()
    };
    for heading in ["settings_1", "settings_2"] {
        assert_eq!(declaration(&before, heading), declaration(&after, heading));
    }
}
