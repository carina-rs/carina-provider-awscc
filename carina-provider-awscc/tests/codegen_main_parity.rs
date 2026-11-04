//! Expected snippets originated from origin/main (d02e242) codegen.
//! The Logs fixture includes the documented single-policy limit from #525.
#![cfg(not(target_arch = "wasm32"))]

use serde_json::{Value, json};
use std::process::Command;

fn cases() -> Vec<(&'static str, Value, &'static [&'static str])> {
    let mut vpc: Value =
        serde_json::from_str(include_str!("../../cfn-schema-cache/AWS__EC2__VPC.json")).unwrap();
    vpc["properties"]["Ratio"] = json!({"type": "number", "allOf": [{"minimum": 0.1}]});
    vpc["properties"]["CidrBlockAssociations"]["items"] =
        json!({"$ref": "#/definitions/Association"});
    vpc["definitions"]["Association"] =
        json!({"type": "object", "properties": {"AssociationId": {"type": "string"}}});
    vpc["properties"]["Modes"] = json!({"type": "object", "patternProperties": {".*": {"type": "string", "pattern": "^(?!aws:).*$", "minLength": 1, "maxLength": 256}}, "additionalProperties": false});
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
    logs["properties"]["FieldIndexPolicies"] = json!({"type": "array", "minItems": 1, "maxItems": 3, "uniqueItems": true, "insertionOrder": false, "items": {"type": "object", "properties": {"Fields": {"type": "array", "items": {"type": "string"}}}}});
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
fn non_reference_rust_and_markdown_match_main() {
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
fn string_override_keeps_the_linked_source_struct_heading() {
    let (_, mut schema, _) = cases().remove(0);
    schema["properties"]
        .as_object_mut()
        .unwrap()
        .remove("Ratio");
    let file = tempfile::NamedTempFile::new().unwrap();
    std::fs::write(file.path(), schema.to_string()).unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_codegen"))
        .args(["--type-name", "AWS::EC2::VPC", "--file"])
        .arg(file.path())
        .args(["--format", "markdown", "--examples-dir"])
        .arg(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../examples"))
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let markdown = String::from_utf8(output.stdout).unwrap();
    assert!(markdown.contains("### cidr_block_associations\n"));
    assert!(markdown.contains("| `association_id` |"));
}
