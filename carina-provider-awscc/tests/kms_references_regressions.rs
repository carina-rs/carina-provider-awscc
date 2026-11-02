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
fn non_reference_string_maps_ignore_scalar_constraints_like_main() {
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
            rust.matches("AttributeType::map(AttributeType::string())")
                .count(),
            2,
            "{rust}"
        );
        let markdown = rendered(&schema, "markdown");
        assert_eq!(
            markdown.matches("Map<String, String>").count(),
            3,
            "{markdown}"
        );
    }
}

#[test]
fn inline_tag_pairs_render_in_rust_and_markdown() {
    let mut schema = vpc();
    schema["properties"]["Tags"]["items"] = json!({"type": "object", "properties": {
        "Key": {"type": "string"}, "Value": {"type": "string"}
    }, "required": ["Key", "Value"]});
    assert!(rendered(&schema, "rust").contains("carina_aws_types::tags_type()"));
    let markdown = rendered(&schema, "markdown");
    assert!(markdown.contains("List\\<[Struct(Tags)]"), "{markdown}");
    // Main also documents the source inline struct; keep those classified paths.
    assert!(markdown.contains("| `key` |"), "{markdown}");
    assert!(markdown.contains("| `value` |"), "{markdown}");
}

#[test]
fn non_reference_fallbacks_do_not_gain_scalar_inference() {
    for property in [
        json!({"pattern": "^(?!aws:).*$", "description": "Allowed values: enabled | disabled"}),
        json!({"$ref": "#/definitions/Missing", "pattern": "^(?!aws:).*$"}),
        json!({"type": "string", "$ref": "#/definitions/Plain", "const": "enabled"}),
    ] {
        let mut schema = vpc();
        schema["properties"]["Ordinary"] = property;
        schema["definitions"]["Plain"] = json!({"type": "string"});
        let rust = rendered(&schema, "rust");
        assert!(
            rust.contains("AttributeSchema::optional(\"ordinary\", AttributeType::string())"),
            "{rust}"
        );
        let markdown = rendered(&schema, "markdown");
        assert!(
            markdown.contains("### `ordinary`\n\n- **Type:** String\n"),
            "{markdown}"
        );
    }
}

#[test]
fn implicit_array_items_keep_mains_plain_string_type() {
    let mut schema = vpc();
    schema["properties"]["SubnetId"] = json!({"type": "array"});
    let rust = rendered(&schema, "rust");
    assert!(
        rust.contains(
            "AttributeSchema::optional(\"subnet_id\", AttributeType::list(AttributeType::string())"
        ),
        "{rust}"
    );
    let markdown = rendered(&schema, "markdown");
    assert!(
        markdown.contains("### `subnet_id`\n\n- **Type:** `List<String>`\n"),
        "{markdown}"
    );
}

#[test]
fn read_only_kms_strings_keep_mains_inferred_output_types() {
    let mut schema = vpc();
    for name in ["KmsKeyId", "KmsKeyArn", "KMSMasterKeyID", "ReplicaKmsKeyID"] {
        schema["properties"][name] = json!({"type": "string"});
        schema["readOnlyProperties"]
            .as_array_mut()
            .unwrap()
            .push(json!(format!("/properties/{name}")));
    }
    let rust = rendered(&schema, "rust");
    assert!(
        rust.contains(
            "AttributeSchema::provider_populated(\"kms_key_id\", super::super::kms::key::arn())"
        ),
        "{rust}"
    );
    assert!(rust.contains("AttributeSchema::provider_populated(\"kms_master_key_id\", carina_aws_types::kms_key_id())"), "{rust}");
    let markdown = rendered(&schema, "markdown");
    assert!(
        markdown.contains("### `kms_key_arn`\n\n- **Type:** KmsKeyArn\n"),
        "{markdown}"
    );
}

#[test]
fn policy_document_projections_audit_the_source_and_hidden_branches() {
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
                    error.contains(&format!("unclassified KMS string {path}")),
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
        Some(carina_aws_types::KMS_KEY_OR_ALIAS_ARN_PATTERN)
    );
    assert_eq!(length, Some((None, Some(256))));
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
