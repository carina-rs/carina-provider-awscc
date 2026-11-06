//! Raw-source accounting must catch schemas discarded by every emitter path.
#![cfg(not(target_arch = "wasm32"))]

use serde_json::{Value, json};
use std::process::Command;

fn generate(schema: &Value, format: &str) -> std::process::Output {
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

#[test]
fn unaccounted_raw_kms_nodes_fail_at_their_json_pointers() {
    for (keyword, source, pointer) in [
        (
            "allOf",
            json!([{"properties": {"KmsHidden": {"type": "string"}}}]),
            "/allOf/0/properties/KmsHidden",
        ),
        (
            "properties",
            json!({"Settings": {"type": ["array", "object"], "items": {"type": "string"}, "patternProperties": {".*": {"type": "string", "description": "The AWS KMS key"}}}}),
            "/properties/Settings/patternProperties/.*",
        ),
        (
            "properties",
            json!({"Settings": {"type": "object", "oneOf": [{"properties": {"Label": {"type": "string"}}}, {"properties": {"Other": {"type": "string"}}, "allOf": [{"properties": {"KmsHidden": {"type": "string"}}}]}]}}),
            "/properties/Settings/oneOf/1/allOf/0/properties/KmsHidden",
        ),
        (
            "definitions",
            json!({"Unused": {"type": "string", "description": "The AWS KMS key"}}),
            "/definitions/Unused",
        ),
        (
            "allOf",
            json!([{"additionalProperties": {"description": "The AWS KMS key"}}]),
            "/allOf/0/additionalProperties",
        ),
        (
            "allOf",
            json!([{"x-future": [false, 17, {"description": "The AWS KMS key"}]}]),
            "/allOf/0/x-future/2",
        ),
    ] {
        let mut schema: Value =
            serde_json::from_str(include_str!("../../cfn-schema-cache/AWS__EC2__VPC.json"))
                .unwrap();
        if let Some(fields) = source.as_object() {
            schema[keyword]
                .as_object_mut()
                .unwrap()
                .extend(fields.clone());
        } else {
            schema[keyword] = source;
        }
        for format in ["rust", "markdown"] {
            let output = generate(&schema, format);
            let error = String::from_utf8_lossy(&output.stderr);
            assert!(!output.status.success(), "{pointer} passed {format}");
            if pointer == "/allOf/0/properties/KmsHidden" {
                // The closed root forbids the conjunct's added field, before
                // emitted-use KMS accounting or root contribution admission.
                assert!(
                    error.contains("unsupported schema projection at #:")
                        && error.contains("closed additionalProperties domain"),
                    "{format}: {error}"
                );
            } else if pointer == "/properties/Settings/patternProperties/.*" {
                assert!(
                    error.contains("unsupported schema projection at #/properties/Settings:")
                        && error.contains("multiple non-null types"),
                    "{format}: {error}"
                );
            } else if pointer == "/properties/Settings/oneOf/1/allOf/0/properties/KmsHidden" {
                // Both optional arms admit the empty object. Reject this
                // overlapping exclusive shape before emitted-use accounting.
                assert!(
                    error.contains("unsupported schema projection at #/properties/Settings:")
                        && error.contains("overlapping oneOf alternatives"),
                    "{error}"
                );
            } else {
                assert!(
                    error.contains("KMS") && error.contains(pointer),
                    "{pointer} {format}: {error}"
                );
            }
            assert!(
                output.stdout.is_empty(),
                "{pointer}/{format} published partial output"
            );
        }
    }
}

#[test]
fn service_name_and_literal_evidence_require_raw_node_accounting() {
    for evidence in [
        json!({"description": "The AWS Key Management Service key identifier"}),
        json!({"pattern": "kEy MaNaGeMeNt SeRvIcE"}),
        json!({"enum": ["AES256", "KMS"]}),
        json!({"enum": ["AES256", "aws:kms"]}),
        json!({"enum": ["AES256", "aws:kms:dsse"]}),
        json!({"const": "aWs:kMs"}),
    ] {
        let mut schema: Value =
            serde_json::from_str(include_str!("../../cfn-schema-cache/AWS__EC2__VPC.json"))
                .unwrap();
        schema["x-future"] = json!({"Unused": evidence});
        for format in ["rust", "markdown"] {
            let output = generate(&schema, format);
            let error = String::from_utf8_lossy(&output.stderr);
            assert!(
                !output.status.success(),
                "unaccounted {evidence} passed {format}"
            );
            assert!(
                error.contains("unaccounted KMS node ec2.Vpc#/x-future/Unused;"),
                "{format}: {error}"
            );
        }
    }
}

#[test]
fn whitespace_separated_kms_phrases_require_raw_node_accounting() {
    for phrase in [
        "kEy\t MaNaGeMeNt\nSeRvIcE",
        "Customer\r\nMaster\tKey",
        "Key\u{a0}Management\u{2003}Service",
    ] {
        for keyword in ["description", "pattern"] {
            let mut schema: Value =
                serde_json::from_str(include_str!("../../cfn-schema-cache/AWS__EC2__VPC.json"))
                    .unwrap();
            schema["x-future"] = json!({"Unused": {keyword: phrase}});
            for format in ["rust", "markdown"] {
                let output = generate(&schema, format);
                let error = String::from_utf8_lossy(&output.stderr);
                assert!(
                    !output.status.success(),
                    "unaccounted {keyword}: {phrase:?} passed {format}"
                );
                assert!(
                    error.contains("unaccounted KMS node ec2.Vpc#/x-future/Unused;"),
                    "{format}: {error}"
                );
            }
        }
    }
}

#[test]
fn reference_projection_cannot_hide_other_anyof_branches() {
    let mut schema: Value =
        serde_json::from_str(include_str!("../../cfn-schema-cache/AWS__SNS__Topic.json")).unwrap();
    schema["properties"]["KmsMasterKeyId"]["anyOf"] =
        json!([{ "type": "object", "properties": {"KmsHidden": {"type": "string"}}}]);
    for format in ["rust", "markdown"] {
        let output = generate(&schema, format);
        let error = String::from_utf8_lossy(&output.stderr);
        assert!(!output.status.success(), "discarded branch passed {format}");
        assert!(
            error.contains("unsupported schema projection at #/properties/KmsMasterKeyId:")
                && error.contains("incompatible conjunctive types"),
            "{error}"
        );
    }
}

#[test]
fn selected_anyof_ref_cannot_borrow_another_use_sites_kms_decision() {
    let mut schema: Value =
        serde_json::from_str(include_str!("../../cfn-schema-cache/AWS__SNS__Topic.json")).unwrap();
    schema["definitions"]["KmsKeyString"] = json!({"type": "string"});
    schema["properties"]["KmsMasterKeyId"] = json!({"$ref": "#/definitions/KmsKeyString"});
    schema["properties"]["Settings"] = json!({"anyOf": [
        {"type": "string"},
        {"type": "object", "properties": {
            "EncryptionKey": {"$ref": "#/definitions/KmsKeyString"}
        }}
    ]});
    for format in ["rust", "markdown"] {
        let output = generate(&schema, format);
        let error = String::from_utf8_lossy(&output.stderr);
        assert!(
            !output.status.success(),
            "unclassified $ref passed {format}"
        );
        assert!(
            error.contains("unclassified KMS string sns.Topic.settings.encryption_key"),
            "{format}: {error}"
        );
        assert!(output.stdout.is_empty());
    }
}

#[test]
fn selected_anyof_ref_requires_kms_classification_in_target_subtrees() {
    let baseline: Value =
        serde_json::from_str(include_str!("../../cfn-schema-cache/AWS__S3__Bucket.json")).unwrap();
    for target in [
        "MetadataTableEncryptionConfiguration",
        "InventoryTableConfiguration",
        "MetadataConfiguration",
    ] {
        let mut schema = baseline.clone();
        schema["properties"]["Settings"] = json!({"anyOf": [
            {"type": "string"},
            {"$ref": format!("#/definitions/{target}")}
        ]});
        for format in ["rust", "markdown"] {
            let output = generate(&schema, format);
            let error = String::from_utf8_lossy(&output.stderr);
            assert!(
                !output.status.success(),
                "unclassified {target} passed {format}"
            );
            assert!(
                error.contains("unclassified KMS string s3.Bucket.settings.")
                    && error.contains("kms_key_arn"),
                "{target}/{format}: {error}"
            );
            assert!(output.stdout.is_empty());
        }
    }
}

#[test]
fn reference_constraints_on_a_ref_use_cannot_bypass_classification() {
    for constraint in [
        json!({"const": "alias/only"}),
        json!({"const": null}),
        json!({"enum": ["alias/only"]}),
        json!({"enum": []}),
    ] {
        let mut schema: Value =
            serde_json::from_str(include_str!("../../cfn-schema-cache/AWS__SNS__Topic.json"))
                .unwrap();
        schema["definitions"]["Plain"] = json!({"type": "string"});
        schema["properties"]["KmsMasterKeyId"] = json!({"$ref": "#/definitions/Plain"});
        schema["properties"]["KmsMasterKeyId"]
            .as_object_mut()
            .unwrap()
            .extend(constraint.as_object().unwrap().clone());
        for format in ["rust", "markdown"] {
            let output = generate(&schema, format);
            let error = String::from_utf8_lossy(&output.stderr);
            assert!(!output.status.success(), "{constraint} passed {format}");
            if constraint.get("const") == Some(&Value::Null) {
                assert!(
                    error.contains("unsupported schema projection at #/properties/KmsMasterKeyId:")
                        && error.contains("null const"),
                    "{error}"
                );
            } else {
                assert!(error.contains("KMS enum/const constraint"), "{error}");
            }
            assert!(
                output.stdout.is_empty(),
                "{constraint}/{format} published partial output"
            );
        }
    }
}

#[test]
fn unused_unemitted_entries_are_stale_and_do_not_exempt_descendants() {
    let baseline: Value =
        serde_json::from_str(include_str!("../../cfn-schema-cache/AWS__KMS__Key.json")).unwrap();
    for hidden_child in [false, true] {
        let mut schema = baseline.clone();
        let expected = if hidden_child {
            schema["properties"]["Tags"]["allOf"] =
                json!([{"additionalProperties": {"description": "The AWS KMS key"}}]);
            "/properties/Tags/allOf/0/additionalProperties"
        } else {
            schema["properties"]["Tags"]
                .as_object_mut()
                .unwrap()
                .remove("description");
            "unused UNEMITTED_KMS_NODES entry kms.Key#/properties/Tags"
        };
        for format in ["rust", "markdown"] {
            let output = generate(&schema, format);
            let error = String::from_utf8_lossy(&output.stderr);
            assert!(
                !output.status.success(),
                "stale/hidden node passed {format}"
            );
            assert!(error.contains(expected), "{error}");
        }
    }
}
