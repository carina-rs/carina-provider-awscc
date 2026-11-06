//! Round-four regressions: classification must observe ordinary CFN shapes.
#![cfg(not(target_arch = "wasm32"))]

use carina_core::schema::Shape;
use serde_json::{Value, json};
use std::process::{Command, Output};

fn generate(schema: &Value, format: &str) -> Output {
    let file = tempfile::NamedTempFile::new().unwrap();
    std::fs::write(file.path(), schema.to_string()).unwrap();
    Command::new(env!("CARGO_BIN_EXE_codegen"))
        .args([
            "--type-name",
            schema["typeName"].as_str().unwrap(),
            "--file",
        ])
        .arg(file.path())
        .args(["--format", format, "--examples-dir"])
        .arg(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../examples"))
        .output()
        .unwrap()
}

fn rendered(schema: &Value, format: &str) -> String {
    let output = generate(schema, format);
    assert!(
        output.status.success(),
        "{format}: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).unwrap()
}

fn vpc() -> Value {
    serde_json::from_str(include_str!("../../cfn-schema-cache/AWS__EC2__VPC.json")).unwrap()
}

#[test]
fn skipped_maps_of_string_arrays_keep_mains_projection() {
    let source = json!({"type": "object", "patternProperties": {
        ".*": {"type": "array", "items": {"type": "string"}}
    }});
    let mut subscription: Value = serde_json::from_str(include_str!(
        "../../cfn-schema-cache/AWS__SNS__Subscription.json"
    ))
    .unwrap();
    subscription["properties"]["FilterPolicy"] = source.clone();
    let mut logs: Value = serde_json::from_str(include_str!(
        "../../cfn-schema-cache/AWS__Logs__LogGroup.json"
    ))
    .unwrap();
    logs["properties"]["FieldIndexPolicies"]["items"] = source;
    for schema in [subscription, logs] {
        for format in ["rust", "markdown"] {
            if schema["typeName"] == "AWS::Logs::LogGroup" && format == "markdown" {
                let output = generate(&schema, format);
                assert!(!output.status.success());
                assert!(String::from_utf8_lossy(&output.stderr).contains("unsupported object map"));
            } else {
                let output = rendered(&schema, format);
                assert!(!output.contains("kms_key_reference()"));
            }
        }
    }
}

#[test]
fn constrained_anyof_object_arm_exposes_its_kms_fields_to_classification() {
    let mut schema = vpc();
    schema["properties"]["Settings"] = json!({"anyOf": [
        {"type": "string"},
        {"type": "object", "properties": {"KmsKeyId": {"type": "string"}}}
    ]});
    for format in ["rust", "markdown"] {
        let output = generate(&schema, format);
        assert!(!output.status.success(), "hidden KmsKeyId passed {format}");
        let error = String::from_utf8_lossy(&output.stderr);
        assert!(
            error.contains("unclassified KMS string ec2.Vpc.settings.kms_key_id"),
            "{format}: {error}"
        );
    }
}

#[test]
fn discarded_source_shapes_fail_raw_accounting() {
    for source in [
        json!({"$ref": "#/definitions/Ordinary", "properties": {
            "KmsKeyId": {"type": "string"}
        }}),
        json!({"oneOf": [
            {"type": "array", "items": {"type": "string"}},
            {"type": "object", "properties": {"KmsKeyId": {"type": "string"}}}
        ]}),
        json!({"type": "object", "properties": {"Label": {"type": "string"}},
            "anyOf": [{"properties": {"KmsKeyId": {"type": "string"}}}]
        }),
    ] {
        let mut schema = vpc();
        schema["definitions"]["Ordinary"] = json!({"type": "object", "properties": {
            "Label": {"type": "string"}
        }});
        schema["properties"]["Settings"] = source.clone();
        for format in ["rust", "markdown"] {
            let output = generate(&schema, format);
            assert!(!output.status.success(), "{source} passed {format}");
            let error = String::from_utf8_lossy(&output.stderr);
            if source.get("oneOf").is_some() {
                assert!(
                    error.contains("unsupported schema projection at #/properties/Settings:")
                        && error.contains(
                            "different JSON types require exactly one constrained alternative arm"
                        ),
                    "{format}: {error}"
                );
            } else {
                assert!(
                    error.contains("unclassified KMS string ec2.Vpc.settings.kms_key_id"),
                    "{format}: {error}"
                );
            }
        }
    }
}

#[test]
fn skipped_unknown_keywords_are_scanned_without_type_inference() {
    let mut schema: Value = serde_json::from_str(include_str!(
        "../../cfn-schema-cache/AWS__SNS__Subscription.json"
    ))
    .unwrap();
    schema["properties"]["FilterPolicy"] = json!({"type": "object", "x-future-shape": {
        "arbitrary": [false, 7, {"description": "The customer master key identifier"}]
    }});
    for format in ["rust", "markdown"] {
        let output = generate(&schema, format);
        assert!(
            !output.status.success(),
            "unknown KMS evidence passed {format}"
        );
        assert!(
            String::from_utf8_lossy(&output.stderr)
                .contains("unaccounted KMS node sns.Subscription#/properties/FilterPolicy/x-future-shape/arbitrary/2")
        );
    }
}

#[test]
fn unvisited_table_entries_fail_after_generation() {
    let mut schema: Value =
        serde_json::from_str(include_str!("../../cfn-schema-cache/AWS__SNS__Topic.json")).unwrap();
    schema["properties"]
        .as_object_mut()
        .unwrap()
        .remove("KmsMasterKeyId");
    for format in ["rust", "markdown"] {
        let output = generate(&schema, format);
        assert!(!output.status.success(), "stale entry passed {format}");
        assert!(
            String::from_utf8_lossy(&output.stderr)
                .contains("unvisited KMS classification sns.Topic.kms_master_key_id")
        );
    }
}

#[test]
fn shared_kms_definition_requires_a_decision_at_each_use_site() {
    let mut schema: Value =
        serde_json::from_str(include_str!("../../cfn-schema-cache/AWS__SNS__Topic.json")).unwrap();
    schema["definitions"]["KmsKeyString"] = json!({"type": "string"});
    for name in ["KmsMasterKeyId", "EncryptionKey"] {
        schema["properties"][name] = json!({"$ref": "#/definitions/KmsKeyString"});
    }
    for read_only in [false, true] {
        if read_only {
            schema["readOnlyProperties"]
                .as_array_mut()
                .unwrap()
                .push(json!("/properties/EncryptionKey"));
        }
        for format in ["rust", "markdown"] {
            let output = generate(&schema, format);
            let error = String::from_utf8_lossy(&output.stderr);
            assert!(
                !output.status.success(),
                "shared KMS definition passed {format} (read_only={read_only})"
            );
            let expected = if read_only {
                "unaccounted KMS node sns.Topic#/definitions/KmsKeyString"
            } else {
                "unclassified KMS string sns.Topic.encryption_key"
            };
            assert!(
                error.contains(expected) && error.contains("sns.Topic.encryption_key"),
                "{format}: {error}"
            );
        }
    }
}

#[test]
fn non_reference_string_maps_refine_keys_and_keep_existing_value_behavior() {
    for value in [
        json!({"type": "string", "pattern": "^(?!aws:).*$", "minLength": 1, "maxLength": 256}),
        json!({"type": "string", "enum": ["enabled", "disabled"]}),
        json!({"type": "string", "description": "Allowed values: enabled | disabled"}),
    ] {
        let mut schema = vpc();
        let map = json!({"type": "object", "patternProperties": {".*": value}, "additionalProperties": false});
        schema["properties"]["Modes"] = map.clone();
        schema["properties"]["Settings"] = json!({"type": "object", "properties": {"Modes": map}});
        let rust = rendered(&schema, "rust");
        assert_eq!(
            rust.matches(r#"AttributeType::map_with_key(AttributeType::refined_string(None, Some("^(?:.*)\\z".to_string()), None, None), AttributeType::string())"#)
                .count(),
            2,
            "{rust}"
        );
        let markdown = rendered(&schema, "markdown");
        assert_eq!(
            markdown.matches("Map<String(pattern), String>").count(),
            2,
            "{markdown}"
        );
        assert_eq!(
            markdown
                .matches("Map<String(len: ..=128), String(len: ..=256)>")
                .count(),
            1,
            "Tags retain their EC2 API length bounds: {markdown}"
        );
    }
}

#[test]
fn inline_tag_pairs_render_in_rust_and_markdown() {
    let mut schema = vpc();
    schema["properties"]["Tags"]["items"] = json!({"type": "object", "properties": {
        "Key": {"type": "string"}, "Value": {"type": "string"}
    }, "required": ["Key", "Value"]});
    assert!(
        rendered(&schema, "rust").contains(
            "AttributeType::map_with_key(AttributeType::string(), AttributeType::string())"
        )
    );
    let markdown = rendered(&schema, "markdown");
    assert!(markdown.contains("`Map<String, String>`"), "{markdown}");
    assert!(!markdown.contains("Struct(Tags)"), "{markdown}");
}

#[test]
fn untyped_non_reference_properties_return_projection_errors() {
    for property in [
        json!({"pattern": "^(?!aws:).*$", "description": "Allowed values: enabled | disabled"}),
        json!({"$ref": "#/definitions/Missing", "pattern": "^(?!aws:).*$"}),
        json!({"type": "string", "$ref": "#/definitions/Plain", "const": "enabled"}),
    ] {
        let mut schema = vpc();
        schema["properties"]["Ordinary"] = property.clone();
        schema["definitions"]["Plain"] = json!({"type": "string"});
        if property["$ref"] == "#/definitions/Missing" {
            for format in ["rust", "markdown"] {
                let output = generate(&schema, format);
                let error = String::from_utf8_lossy(&output.stderr);
                assert!(
                    !output.status.success()
                        && error.contains("reference")
                        && error.contains("/properties/Ordinary/$ref")
                        && error.contains("no target"),
                    "{format}: {error}"
                );
                assert!(output.stdout.is_empty());
            }
            continue;
        }
        if property.get("const").is_some() {
            let rust = rendered(&schema, "rust");
            assert!(
                rust.contains("enum_identity(\"Plain\"")
                    && rust.contains("\"enabled\".to_string()"),
                "{rust}"
            );
            let markdown = rendered(&schema, "markdown");
            assert!(markdown.contains("aws.ec2.Vpc.Plain.enabled"), "{markdown}");
            continue;
        }
        for format in ["rust", "markdown"] {
            let output = generate(&schema, format);
            let error = String::from_utf8_lossy(&output.stderr);
            assert_eq!(output.status.code(), Some(1), "{format}: {error}");
            assert!(
                error.contains("unsupported schema projection at #/properties/Ordinary:")
                    && error.contains("no supported type or literal projection"),
                "{format}: {error}"
            );
            assert!(output.stdout.is_empty());
            assert!(!error.contains("panicked"), "{error}");
        }
    }
}

#[test]
fn implicit_array_items_return_projection_errors() {
    let mut schema = vpc();
    schema["properties"]["SubnetId"] = json!({"type": "array"});
    for format in ["rust", "markdown"] {
        let output = generate(&schema, format);
        let error = String::from_utf8_lossy(&output.stderr);
        assert_eq!(output.status.code(), Some(1), "{format}: {error}");
        assert!(
            error.contains("unsupported schema projection at #/properties/SubnetId:")
                && error.contains("array has no items projection"),
            "{format}: {error}"
        );
        assert!(output.stdout.is_empty());
        assert!(!error.contains("panicked"), "{error}");
    }
}

#[test]
fn read_only_kms_candidates_also_require_explicit_accounting() {
    for name in ["KmsKeyId", "KmsKeyArn", "KMSMasterKeyID", "ReplicaKmsKeyID"] {
        let mut schema = vpc();
        schema["properties"][name] = json!({"type": "string"});
        schema["readOnlyProperties"]
            .as_array_mut()
            .unwrap()
            .push(json!(format!("/properties/{name}")));
        for format in ["rust", "markdown"] {
            let output = generate(&schema, format);
            let error = String::from_utf8_lossy(&output.stderr);
            assert!(!output.status.success(), "{name} passed {format}");
            assert!(
                error.contains(&format!("unaccounted KMS node ec2.Vpc#/properties/{name}")),
                "{error}"
            );
        }
    }
}

#[test]
fn policy_document_projections_require_raw_source_accounting() {
    for (cache, name, path) in [
        (
            include_str!("../../cfn-schema-cache/AWS__Logs__LogGroup.json"),
            "ResourcePolicyDocument",
            "logs.LogGroup.resource_policy_document",
        ),
        (
            include_str!("../../cfn-schema-cache/AWS__S3__BucketPolicy.json"),
            "PolicyDocument",
            "s3.BucketPolicy.policy_document",
        ),
    ] {
        for source in [
            json!({"type": "object", "description": "The AWS KMS key"}),
            json!({"type": "object", "allOf": [{"properties": {
                "Value": {"type": "string", "description": "The AWS KMS key"}
            }}]}),
        ] {
            let mut schema: Value = serde_json::from_str(cache).unwrap();
            schema["properties"][name] = source.clone();
            for format in ["rust", "markdown"] {
                let output = generate(&schema, format);
                assert!(!output.status.success(), "{path} {source} passed {format}");
                let error = String::from_utf8_lossy(&output.stderr);
                assert!(
                    (error.contains("unaccounted KMS node")
                        && error.contains(&format!("/properties/{name}")))
                        || (source.get("allOf").is_some()
                            && error.contains(&format!("unclassified KMS string {path}.value"))),
                    "{error}"
                );
            }
        }
    }
}

#[test]
fn logs_reference_keeps_cfn_maximum_and_no_synthetic_minimum() {
    let config =
        carina_provider_awscc::schemas::generated::logs::log_group::logs_log_group_config();
    let schema = config.schema.as_schema();
    let ty = schema.attributes()["kms_key_id"].attr_type();
    let Shape::Reference {
        pattern, length, ..
    } = schema.shape_of(ty)
    else {
        panic!("logs kms_key_id must be a reference");
    };
    assert_eq!(
        pattern,
        Some(r"^arn:[a-z0-9-]+:kms:[a-z0-9-]+:\d{12}:(key|alias)/.+\z")
    );
    assert_eq!(length, Some((None, Some(256))));
    let pattern = carina_core::schema::CompiledPattern::try_new(pattern.unwrap()).unwrap();
    assert!(pattern.is_match("arn:aws:kms:us-east-1:123456789012:alias/team/scheduler"));
    assert!(!pattern.is_match("arn:aws:kms:us-east-1:123456789012:alias/customer\n"));
}

#[test]
fn changed_cfn_reference_length_requires_reclassification() {
    for constraint in [
        json!({"minLength": 0}),
        json!({"maxLength": 2048}),
        json!({}),
    ] {
        let mut schema: Value = serde_json::from_str(include_str!(
            "../../cfn-schema-cache/AWS__Logs__LogGroup.json"
        ))
        .unwrap();
        let property = schema["properties"]["KmsKeyId"].as_object_mut().unwrap();
        property.remove("maxLength");
        property.extend(constraint.as_object().unwrap().clone());
        for format in ["rust", "markdown"] {
            let output = generate(&schema, format);
            assert!(!output.status.success(), "{constraint} passed {format}");
            let error = String::from_utf8_lossy(&output.stderr);
            assert!(error.contains("KMS length changed"), "{error}");
        }
    }
}

#[test]
fn reference_length_evidence_in_aliases_and_branches_is_pinned() {
    for location in ["direct", "$ref", "oneOf", "anyOf", "allOf", "root"] {
        for bound in [json!({"minLength": 0}), json!({"maxLength": 1024})] {
            let mut schema: Value =
                serde_json::from_str(include_str!("../../cfn-schema-cache/AWS__SNS__Topic.json"))
                    .unwrap();
            match location {
                "direct" => schema["properties"]["KmsMasterKeyId"]
                    .as_object_mut()
                    .unwrap()
                    .extend(bound.as_object().unwrap().clone()),
                "$ref" => {
                    schema["properties"]["KmsMasterKeyId"] = json!({"$ref": "#/definitions/Key"});
                    schema["definitions"]["Key"] = json!({"type": "string", "allOf": [bound]});
                }
                "root" => schema["allOf"] = json!([{"properties": {"KmsMasterKeyId": bound}}]),
                keyword => schema["properties"]["KmsMasterKeyId"][keyword] = json!([bound]),
            }
            for format in ["rust", "markdown"] {
                let output = generate(&schema, format);
                assert!(!output.status.success(), "{location} passed {format}");
                let error = String::from_utf8_lossy(&output.stderr);
                assert!(error.contains("KMS length changed"), "{error}");
            }
        }
    }
}
