//! Offline generation checks for schema-checked read-back declarations.
#![cfg(not(target_arch = "wasm32"))]

use std::process::Command;

#[test]
fn read_back_codegen_checks_declared_attribute_before_emitting_artifacts() {
    let file = tempfile::NamedTempFile::new().unwrap();
    let mut schema: serde_json::Value =
        serde_json::from_str(include_str!("../../cfn-schema-cache/AWS__IAM__Role.json")).unwrap();
    schema["properties"].as_object_mut().unwrap().remove("Arn");
    schema["readOnlyProperties"]
        .as_array_mut()
        .unwrap()
        .retain(|path| path != "/properties/Arn");
    std::fs::write(file.path(), schema.to_string()).unwrap();
    for format in ["rust", "markdown"] {
        let output = Command::new(env!("CARGO_BIN_EXE_codegen"))
            .args(["--type-name", "AWS::IAM::Role", "--file"])
            .arg(file.path())
            .args(["--format", format, "--examples-dir"])
            .arg(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../examples"))
            .output()
            .unwrap();
        assert!(
            !output.status.success(),
            "an unknown read-back attribute must fail {format} generation"
        );
        let error = String::from_utf8_lossy(&output.stderr);
        assert!(
            error.contains("declares unknown read-back attribute arn"),
            "{error}"
        );
    }
}

#[test]
fn read_back_codegen_emits_checked_declaration_and_shared_default_budget() {
    let cache = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../cfn-schema-cache/AWS__IAM__Role.json");
    let output = Command::new(env!("CARGO_BIN_EXE_codegen"))
        .args(["--type-name", "AWS::IAM::Role", "--file"])
        .arg(&cache)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let code = String::from_utf8(output.stdout).unwrap();
    assert!(
        code.contains("ReadBackAttribute::new(\"arn\")"),
        "generated declarations must carry the checked type"
    );
    assert!(
        !code.contains("read_back: Some("),
        "default budgets should remain None"
    );
    let output = Command::new(env!("CARGO_BIN_EXE_codegen"))
        .args(["--type-name", "AWS::IAM::Role", "--file"])
        .arg(cache)
        .args(["--format", "markdown", "--examples-dir"])
        .arg(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../examples"))
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(String::from_utf8(output.stdout).unwrap().contains(&format!(
        "The read-back budget is {} seconds.",
        carina_provider_protocol::DEFAULT_READ_BACK_BUDGET.as_secs()
    )));
}
