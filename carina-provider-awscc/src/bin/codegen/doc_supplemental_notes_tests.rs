use super::doc_test_support::{NAMED_OVERRIDES, declaration_page, emission, named_schema, text};
use super::*;
use comrak::{Arena, Options, parse_document};
use serde_json::json;

#[test]
fn source_facts_are_emitted_once_through_property_wrappers() {
    let parsed = ParsedCfnSchema::from_json(json!({
        "typeName":"AWS::Test::Notes",
        "properties":{
            "Direct":{"type":"integer","format":"int64","multipleOf":5},
            "ViaAlias":{"$ref":"#/definitions/Alias"},
            "List":{"type":"array","items":{"$ref":"#/definitions/Alias"}},
            "Map":{"type":"object","patternProperties":{".*":{"type":"string","format":"email"}}},
            "Nested":{"type":"object","properties":{"Value":{"$ref":"#/definitions/Alias"}}}
        },
        "definitions":{
            "Number":{"type":"integer","format":"int64","multipleOf":5},
            "Alias":{"$ref":"#/definitions/Number"}
        }
    }))
    .unwrap();
    let context = EmissionContext::from_source(&parsed).unwrap();
    let emitted = emit_resource(&context, &parsed.type_name, false).unwrap();
    for info in [
        emitted.attributes["direct"].annotations.as_ref(),
        emitted.attributes["via_alias"].annotations.as_ref(),
        &emitted.attributes["list"].annotations.children["item"],
        &emitted.attributes["nested"].annotations.children["value"],
    ] {
        assert_eq!(
            info.notes,
            ["CFN format: int64", "multiple of 5 (not enforced)"]
        );
    }
    assert_eq!(
        emitted.attributes["map"].annotations.children["value"].notes,
        ["CFN format: email"]
    );
}

#[test]
fn tag_keys_have_one_numeric_string_annotation() {
    let parsed = ParsedCfnSchema::from_json(json!({
        "typeName":"AWS::Test::Notes",
        "properties":{"Tags":{"type":"array","items":{"type":"object","properties":{
            "Key":{"type":"string","pattern":"^[0-9]+$"},"Value":{"type":"string"}
        },"required":["Key","Value"]}}},
        "tagging":{"taggable":true,"tagProperty":"/properties/Tags"}
    }))
    .unwrap();
    let context = EmissionContext::from_source(&parsed).unwrap();
    let emitted = emit_resource(&context, &parsed.type_name, false).unwrap();
    assert_eq!(
        emitted.attributes["tags"].annotations.children["key"].notes,
        ["NumericString"]
    );
}

#[test]
fn named_override_notes_stay_at_each_use_site() {
    with_resource_type_overrides(&NAMED_OVERRIDES, || {
        let parsed = named_schema();
        let md = generate_test_markdown(&parsed, &parsed.type_name).unwrap();
        for (name, expected, other) in [
            ("left", "email", "hostname"),
            ("right", "hostname", "email"),
        ] {
            let section = md
                .split(&format!("### `{name}`\n"))
                .nth(1)
                .unwrap()
                .split("\n### ")
                .next()
                .unwrap();
            assert!(
                section.contains(&format!("CFN format: {expected}")),
                "{name}: {section}"
            );
            assert!(
                !section.contains(&format!("CFN format: {other}")),
                "{name}: {section}"
            );
        }
    });
}

#[test]
fn reference_notes_are_not_lost_during_resolution() {
    let resource = emission(
        BTreeMap::from([(
            "probe".into(),
            EmittedType::reference("Text").note("only this use"),
        )]),
        BTreeMap::from([(
            "Text".into(),
            EmittedType::new("fixture", AttributeType::string()),
        )]),
    );
    let md = declaration_page(&resource).unwrap();
    assert!(md.contains("; only this use"), "{md}");
}

#[test]
fn union_source_notes_belong_only_to_the_root() {
    let parsed = ParsedCfnSchema::from_json(json!({
        "typeName":"AWS::Test::UnionNotes",
        "properties":{"Probe":{"type":"integer","format":"int64","multipleOf":5}}
    }))
    .unwrap();
    let context = EmissionContext::from_source(&parsed).unwrap();
    let expression = ConstraintSpec::union(vec![
        ConstraintSpec::int_range(Some(1), Some(10)),
        ConstraintSpec::int_range(Some(20), Some(30)),
    ]);
    let emitted = expression
        .emit(
            &context.properties["Probe"],
            &context,
            CfnPropertyPath::new(None, "Probe"),
            None,
        )
        .unwrap();
    assert_eq!(
        emitted.annotations.notes,
        vec!["CFN format: int64", "multiple of 5 (not enforced)"]
    );
    for child in emitted.annotations.children.values() {
        assert!(
            child.notes.is_empty(),
            "union member repeated root facts: {:?}",
            child.notes
        );
    }
}

#[test]
fn map_key_keeps_numeric_string_note_from_shared_producer() {
    let parsed = ParsedCfnSchema::from_json(json!({
        "typeName":"AWS::Test::Round1", "properties":{"Probe":{
            "type":"object", "patternProperties":{"[0-9]+":{"type":"string"}},
            "additionalProperties":false
        }}
    }))
    .unwrap();
    let markdown = generate_test_markdown(&parsed, &parsed.type_name).unwrap();
    assert_eq!(markdown.matches("NumericString").count(), 1, "{markdown}");
}

#[test]
fn opaque_retention_catalog_type_keeps_cfn_allowed_values_note() {
    let parsed = cfn_schema_for_codegen_tests("AWS::Logs::LogGroup");
    let context = EmissionContext::from_source(&parsed).unwrap();
    let values = context.properties["RetentionInDays"]
        .try_query()
        .unwrap()
        .enum_values
        .as_ref()
        .unwrap();
    let expected = format!(
        "CFN allowed values: {}",
        serde_json::to_string(values).unwrap()
    );
    let emitted = emit_resource(&context, &parsed.type_name, true).unwrap();
    let retention = &emitted.attributes["retention_in_days"];
    assert_eq!(
        retention
            .annotations
            .notes
            .iter()
            .filter(|note| **note == expected)
            .count(),
        1
    );
    let md = generate_test_markdown(&parsed, &parsed.type_name).unwrap();
    let arena = Arena::new();
    let root = parse_document(&arena, &md, &Options::default());
    assert!(text(root).contains(&expected), "{md}");
}
