//! Baselines recorded with the raw KMS walker at 80d58ad, before Layer migration.
//! Keep source and evidence order as well as decisions: membership alone misses
//! alias siblings, collection inheritance and shared-definition use-site bugs.

use super::*;
use serde_json::{Value, json};
use std::path::Path;

fn observation(context: &EmissionContext<'_>) -> Value {
    let candidates = kms_nodes(context.layer);
    let sites: BTreeMap<_, _> = context
        .sites
        .borrow()
        .iter()
        .filter(|(path, site)| {
            site.sources
                .iter()
                .any(|source| candidates.contains(source))
                || context.visited.borrow().contains(&context.dsl_path(path))
        })
        .map(|(path, site)| (context.dsl_path(path), serde_json::to_value(site).unwrap()))
        .collect();
    let decisions: BTreeMap<_, _> = context
        .decisions
        .borrow()
        .iter()
        .map(|(path, decision)| (path.clone(), format!("{decision:?}")))
        .collect();
    json!({
        "sites": sites,
        "decisions": decisions,
        "classified_sources": *context.classified_sources.borrow(),
        "visited": *context.visited.borrow(),
    })
}

fn candidate_evidence(layer: &Layer) -> Value {
    let candidates: BTreeMap<_, _> = kms_nodes(layer)
        .into_iter()
        .map(|pointer| {
            let mut evidence = Evidence::default();
            evidence.add_source(&pointer, &layer.sources().evidence()[&pointer]);
            (pointer, evidence)
        })
        .collect();
    serde_json::to_value(candidates).unwrap()
}

fn markdown_example(type_name: &str) -> carina_docgen::example::DocExample {
    let resource = dsl_resource_name_from_type(type_name).unwrap();
    carina_docgen::example::check(
        &format!("awscc.{resource} {{}}\n"),
        Path::new("kms-snapshot.crn"),
        "awscc",
        &resource,
        carina_docgen::example::ExampleKind::Resource,
    )
    .unwrap()
}

fn assert_snapshot(name: &str, actual: Value) {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name);
    if std::env::var_os("UPDATE_KMS_LAYER_SNAPSHOTS").is_some() {
        std::fs::write(
            &path,
            format!("{}\n", serde_json::to_string_pretty(&actual).unwrap()),
        )
        .unwrap();
    }
    let expected: Value = serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
    let actual = actual.as_object().unwrap();
    let expected = expected.as_object().unwrap();
    assert_eq!(
        actual.keys().collect::<Vec<_>>(),
        expected.keys().collect::<Vec<_>>()
    );
    for (resource, actual) in actual {
        assert_eq!(actual, &expected[resource], "KMS baseline: {resource}");
    }
}

#[test]
fn supported_cache_kms_decisions_match_pre_layer_golden() {
    let cache = Path::new(env!("CARGO_MANIFEST_DIR")).join("../cfn-schema-cache");
    let mut paths: Vec<_> = std::fs::read_dir(cache)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| {
            path.file_name()
                .unwrap()
                .to_str()
                .is_some_and(|name| name.starts_with("AWS__") && name.ends_with(".json"))
        })
        .collect();
    paths.sort();
    assert_eq!(paths.len(), 55);
    let mut snapshot = BTreeMap::new();
    for path in paths {
        let source: Value = serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
        let schema = ParsedCfnSchema::from_json(source.clone()).unwrap();
        let rust = EmissionContext::from_source(&schema).unwrap();
        super::super::render_schema_code(&rust, &schema.type_name)
            .unwrap_or_else(|error| panic!("{} Rust: {error}", schema.type_name));
        let markdown = EmissionContext::from_source(&schema).unwrap();
        super::super::render_markdown(
            &markdown,
            &schema.type_name,
            &markdown_example(&schema.type_name),
        )
        .unwrap_or_else(|error| panic!("{} Markdown: {error}", schema.type_name));
        snapshot.insert(
            schema.type_name.clone(),
            json!({
                "candidates": candidate_evidence(rust.layer),
                "rust": observation(&rust),
                "markdown": observation(&markdown),
            }),
        );
    }
    assert_snapshot("kms_supported_cache_pre_layer.json", json!(snapshot));
}

#[test]
fn aliases_collections_compositions_dependencies_and_transforms_match_pre_layer_golden() {
    let leaf = json!({"type": "string", "description": "AWS KMS leaf", "maxLength": 256});
    let mut fixtures = BTreeMap::new();
    fixtures.insert("direct", leaf.clone());
    fixtures.insert(
        "alias",
        json!({"$ref": "#/definitions/Alias", "description": "AWS KMS use"}),
    );
    for keyword in ["allOf", "oneOf", "anyOf"] {
        fixtures.insert(
            keyword,
            json!({keyword: [
                {"description": "first branch"},
                {"$ref": "#/definitions/Alias", "description": "AWS KMS later branch"}
            ]}),
        );
    }
    fixtures.insert(
        "dependencies",
        json!({"dependencies": {
            "A": {"$ref": "#/definitions/Alias", "description": "AWS KMS dependency A"},
            "B": ["KmsRequiredName"],
            "Z": {"description": "AWS KMS dependency Z", "pattern": "kms"}
        }}),
    );
    fixtures.insert("mixed_order", json!({
        "$ref": "#/definitions/Alias", "description": "AWS KMS use",
        "allOf": [{"description": "all 0"}, {"description": "all 1"}],
        "anyOf": [{"description": "any 0"}, {"description": "any 1"}],
        "oneOf": [{"description": "one 0"}, {"description": "one 1"}],
        "dependencies": {"A": {"description": "dependency A"}, "Z": {"description": "dependency Z"}}
    }));
    fixtures.insert(
        "list",
        json!({"type": "array", "description": "AWS KMS list", "items": {
            "$ref": "#/definitions/Alias", "description": "AWS KMS item"
        }}),
    );
    fixtures.insert(
        "map",
        json!({"type": "object", "description": "AWS KMS map", "patternProperties": {
            "a/b~c": {"$ref": "#/definitions/Alias", "description": "AWS KMS value"}
        }}),
    );
    fixtures.insert("dangling", json!({"$ref": "#/definitions/Missing"}));
    fixtures.insert("transform", json!({"type": "string"}));
    fixtures.insert("root_composition", json!({"type": "string"}));
    let mut snapshot = BTreeMap::new();
    for (name, property) in fixtures {
        let mut source = json!({
            "typeName": "AWS::SNS::Topic", "properties": {"KmsMasterKeyId": property},
            "definitions": {
                "Alias": {"$ref": "#/definitions/Key", "description": "AWS KMS alias"},
                "Key": leaf
            }
        });
        if name == "transform" {
            source["propertyTransform"] =
                json!({"/properties/KmsMasterKeyId": "opaque KMS expression"});
        }
        if name == "root_composition" {
            source["allOf"] =
                json!([{"properties": {"KmsMasterKeyId": {"$ref": "#/definitions/Alias"}}}]);
        }
        let schema = ParsedCfnSchema::from_json(source.clone()).unwrap();
        let context = EmissionContext::from_source(&schema).unwrap();
        let mut path = vec!["KmsMasterKeyId".to_string()];
        context.property_at(&path, &schema.properties["KmsMasterKeyId"]);
        if name == "list" || name == "map" {
            path = context.collection_path(
                &path,
                if name == "map" {
                    PathSegment::MapValue
                } else {
                    PathSegment::ListElement
                },
            );
        }
        // Source selection depends on emitter choices, not the legacy property
        // supplied here. In particular it must keep all alias/branch siblings.
        let decision = context
            .property_at(&path, &schema.properties["KmsMasterKeyId"])
            .classify()
            .map(|kind| format!("{kind:?}"))
            .unwrap_or_else(|error| error.to_string());
        snapshot.insert(
            name,
            json!({
                "candidates": candidate_evidence(context.layer),
                "emission": observation(&context),
                "decision": decision,
                "finish": context.finish().err().map(|error| error.to_string()),
            }),
        );
    }
    assert_snapshot("kms_adversarial_pre_layer.json", json!(snapshot));
}

#[test]
fn overriding_an_opaque_object_still_accounts_for_each_source_candidate() {
    let mut source: Value = serde_json::from_str(include_str!(
        "../../../../cfn-schema-cache/AWS__SNS__Subscription.json"
    ))
    .unwrap();
    source["properties"]["FilterPolicy"] = json!({"type": "object", "allOf": [{
        "properties": {"KmsHidden": {"type": "string"}}
    }]});
    let schema = ParsedCfnSchema::from_json(source).unwrap();
    let rust = EmissionContext::from_source(&schema).unwrap();
    let markdown = EmissionContext::from_source(&schema).unwrap();
    for result in [
        super::super::render_schema_code(&rust, &schema.type_name),
        super::super::render_markdown(
            &markdown,
            &schema.type_name,
            &markdown_example(&schema.type_name),
        ),
    ] {
        let error = result.unwrap_err().to_string();
        assert!(error.contains("unaccounted KMS node sns.Subscription#/properties/FilterPolicy/allOf/0/properties/KmsHidden"), "{error}");
    }
}

#[test]
fn typed_unresolved_references_keep_use_site_evidence_and_opaque_fallback() {
    use super::super::schema_layer::SchemaError;

    for (reference, expected) in [
        (
            "#/definitions/Missing",
            SchemaError::MissingReferenceTarget {
                pointer: "/properties/KmsMasterKeyId/$ref".into(),
                reference: "#/definitions/Missing".into(),
            },
        ),
        (
            "other.json#/definitions/Key",
            SchemaError::UnsupportedReference {
                pointer: "/properties/KmsMasterKeyId/$ref".into(),
                reference: "other.json#/definitions/Key".into(),
            },
        ),
        (
            "#/properties/Plain/description",
            SchemaError::NonSchemaReferenceTarget {
                pointer: "/properties/KmsMasterKeyId/$ref".into(),
                reference: "#/properties/Plain/description".into(),
                target: "/properties/Plain/description".into(),
            },
        ),
    ] {
        let schema = ParsedCfnSchema::from_json(json!({
            "typeName": "AWS::SNS::Topic",
            "properties": {
                "KmsMasterKeyId": {"$ref": reference, "description": "AWS KMS use-site evidence"},
                "Plain": {"type": "string", "description": "ordinary value"}
            }
        }))
        .unwrap();
        let context = EmissionContext::from_source(&schema).unwrap();
        let keywords = context.keywords_at("/properties/KmsMasterKeyId").unwrap();
        assert_eq!(
            keywords.reference.as_ref().unwrap().target,
            RefTarget::Unresolved(expected)
        );
        let path = ["KmsMasterKeyId".into()];
        assert_eq!(
            context
                .property_at(&path, &schema.properties["KmsMasterKeyId"])
                .classify()
                .unwrap(),
            Some(ReferenceKind::Key),
        );
        let sites = context.sites.borrow();
        let site = &sites[&path[..]];
        assert_eq!(site.sources, ["/properties/KmsMasterKeyId"]);
        assert_eq!(site.evidence.descriptions, ["AWS KMS use-site evidence"]);
        context.finish().unwrap();
    }
}

#[test]
fn retained_conversion_diagnostics_cannot_be_accepted_after_accounting() {
    use super::super::schema_layer::SchemaError;

    let mut schema = ParsedCfnSchema::from_json(json!({
        "typeName": "AWS::Test::Resource", "properties": {"Settings": {"type": "object"}}
    }))
    .unwrap();
    let expected = SchemaError::UnknownKeyword {
        pointer: "/properties/Settings/x-future".into(),
        keyword: "x-future".into(),
    };
    schema.edit_json_with_layer_error(
        |source| {
            source["properties"]["Settings"]["x-future"] = json!({"unknown": [false, 7]});
        },
        expected.clone(),
    );
    let context = EmissionContext::from_source(&schema).unwrap();
    assert!(kms_nodes(context.layer).is_empty());
    let error = context.finish().unwrap_err();
    assert_eq!(error.downcast_ref::<SchemaError>(), Some(&expected));
}
