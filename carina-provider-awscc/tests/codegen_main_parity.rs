//! Expected snippets originated from origin/main (d02e242) codegen.
//! The Logs fixture keeps the documented single-policy limit and the plain
//! typed policy item, including the documented FieldsV2 alternative.
//! The VPC map fixtures include CFN-derived patternProperties keys from #525.
//! Map value constraints and enums now follow the shared scalar projection.
#![cfg(not(target_arch = "wasm32"))]

use serde_json::{Value, json};
use std::process::Command;

#[test]
fn step4_unsupported_emission_reaches_cli_error_output() {
    for property in [
        json!({}),
        json!(false),
        json!({"type":"array"}),
        json!({"oneOf":[{"type":"boolean"},{"type":"boolean"}]}),
        json!({"type":"object","patternProperties":{".*":{"type":"integer"}}}),
    ] {
        let mut schema: Value =
            serde_json::from_str(include_str!("../../cfn-schema-cache/AWS__EC2__VPC.json"))
                .unwrap();
        schema["properties"]["UnsupportedProbe"] = property.clone();
        let file = tempfile::NamedTempFile::new().unwrap();
        std::fs::write(file.path(), schema.to_string()).unwrap();
        for format in ["rust", "markdown"] {
            let output = Command::new(env!("CARGO_BIN_EXE_codegen"))
                .args(["--type-name", "AWS::EC2::VPC", "--format", format, "--file"])
                .arg(file.path())
                .arg("--examples-dir")
                .arg(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../examples"))
                .output()
                .unwrap();
            let stderr = String::from_utf8(output.stderr).unwrap();
            assert_eq!(
                output.status.code(),
                Some(1),
                "{format}/{property}: {stderr}"
            );
            assert!(
                stderr.contains("unsupported schema projection at #/properties/UnsupportedProbe"),
                "{format}/{property}: {stderr}"
            );
            assert!(!stderr.contains("panicked"), "{stderr}");
            assert!(output.stdout.is_empty(), "{format}/{property}");
        }
    }
}

fn cases() -> Vec<(&'static str, Value, &'static [&'static str])> {
    let mut vpc: Value =
        serde_json::from_str(include_str!("../../cfn-schema-cache/AWS__EC2__VPC.json")).unwrap();
    // Fractional allOf endpoints are now retained and explicitly rejected by
    // the staged integer-endpoint emitter; keep this unrelated-output pin plain.
    vpc["properties"]["Ratio"] = json!({"type": "number"});
    vpc["properties"]["Modes"] = json!({"type": "object", "patternProperties": {".*": {"type": "string", "pattern": "^[a-z]+$", "minLength": 1, "maxLength": 256}}, "additionalProperties": false});
    vpc["properties"]["EnumModes"] = json!({"type": "object", "patternProperties": {".*": {"type": "string", "enum": ["enabled", "disabled"]}}, "additionalProperties": false});
    vpc["properties"]["Tags"]["items"] = json!({"type": "object", "properties": {"Key": {"type": "string"}, "Value": {"type": "string"}}, "required": ["Key", "Value"]});
    let mut subscription: Value = serde_json::from_str(include_str!(
        "../../cfn-schema-cache/AWS__SNS__Subscription.json"
    ))
    .unwrap();
    subscription["properties"]["FilterPolicy"] = json!({"type": "object", "patternProperties": {".*": {"type": "array", "items": {"type": "string"}}}});
    let mut logs: Value = serde_json::from_str(include_str!(
        "../../cfn-schema-cache/AWS__Logs__LogGroup.json"
    ))
    .unwrap();
    logs["properties"]["FieldIndexPolicies"] = json!({"type": "array", "minItems": 1, "maxItems": 3, "uniqueItems": true, "insertionOrder": false, "items": {"type": "object"}});
    vec![
        (
            "vpc",
            vpc,
            &[
                "ratio",
                "cidr_block_associations",
                "modes",
                "enum_modes",
                "tags",
                "Association",
                "Tags",
            ],
        ),
        ("subscription", subscription, &["filter_policy"]),
        ("logs", logs, &["field_index_policies"]),
    ]
}

fn snippet(output: &str, format: &str, names: &[&str]) -> String {
    let mut active = false;
    let mut lines = Vec::new();
    for line in output.lines() {
        if format == "rust" {
            if line.starts_with("    fn build_") {
                active = names
                    .iter()
                    .any(|name| line.contains(&format!("build_{name}_attribute()")));
            }
            if active {
                lines.push(line);
            }
            if line == "    }" {
                active = false;
            }
        } else {
            if line.starts_with('#') {
                active = names.iter().any(|name| {
                    line.contains(&format!("`{name}`")) || line == format!("### {name}")
                });
            }
            if active {
                lines.push(line);
            }
        }
    }
    format!("{}\n", lines.join("\n").trim_end())
}

#[test]
fn non_reference_rust_and_markdown_match_reviewed_fixtures() {
    for (case, schema, names) in cases() {
        let file = tempfile::NamedTempFile::new().unwrap();
        std::fs::write(file.path(), schema.to_string()).unwrap();
        for format in ["rust", "markdown"] {
            let output = Command::new(env!("CARGO_BIN_EXE_codegen"))
                .args([
                    "--type-name",
                    schema["typeName"].as_str().unwrap(),
                    "--file",
                ])
                .arg(file.path())
                .args(["--format", format, "--examples-dir"])
                .arg(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../examples"))
                .output()
                .unwrap();
            assert!(
                output.status.success(),
                "{case}/{format}: {}",
                String::from_utf8_lossy(&output.stderr)
            );
            let expected =
                std::fs::read_to_string(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(
                    format!("tests/fixtures/codegen_main_parity/{case}.{format}.txt"),
                ))
                .unwrap();
            assert_eq!(
                snippet(&String::from_utf8(output.stdout).unwrap(), format, names),
                expected,
                "{case}/{format}"
            );
        }
    }
}

#[test]
fn string_identity_cannot_replace_a_linked_source_struct() {
    let (_, mut schema, _) = cases().remove(0);
    schema["properties"]["CidrBlockAssociations"]["items"] =
        json!({"$ref":"#/definitions/Association"});
    schema["definitions"]["Association"] =
        json!({"type":"object","properties":{"AssociationId":{"type":"string"}}});
    let file = tempfile::NamedTempFile::new().unwrap();
    std::fs::write(file.path(), schema.to_string()).unwrap();
    for format in ["rust", "markdown"] {
        let output = Command::new(env!("CARGO_BIN_EXE_codegen"))
            .args(["--type-name", "AWS::EC2::VPC", "--file"])
            .arg(file.path())
            .args(["--format", format, "--examples-dir"])
            .arg(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../examples"))
            .output()
            .unwrap();
        let error = String::from_utf8_lossy(&output.stderr);
        assert!(
            !output.status.success()
                && error.contains("CidrBlockAssociations")
                && error.contains("string identity requires the CFN string kind"),
            "{format}: {error}"
        );
        assert!(output.stdout.is_empty());
    }
}

#[test]
fn fractional_all_of_bounds_fail_explicitly_instead_of_becoming_an_unbounded_float() {
    let (_, mut schema, _) = cases().remove(0);
    schema["properties"]["Ratio"] = json!({"type":"number","allOf":[{"minimum":0.1}]});
    let file = tempfile::NamedTempFile::new().unwrap();
    std::fs::write(file.path(), schema.to_string()).unwrap();
    for format in ["rust", "markdown"] {
        let output = Command::new(env!("CARGO_BIN_EXE_codegen"))
            .args(["--type-name", "AWS::EC2::VPC", "--file"])
            .arg(file.path())
            .args(["--format", format, "--examples-dir"])
            .arg(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../examples"))
            .output()
            .unwrap();
        let error = String::from_utf8_lossy(&output.stderr);
        assert!(
            !output.status.success()
                && error.contains("unsupported schema projection at #/properties/Ratio:")
                && error.contains("non-integral"),
            "{format}: {error}"
        );
        assert!(output.stdout.is_empty());
    }
}
