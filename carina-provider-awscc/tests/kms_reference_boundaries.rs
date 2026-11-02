//! Compile adversarial entry points inside the crate's own visibility boundary.
#![cfg(not(target_arch = "wasm32"))]

use std::path::{Path, PathBuf};
use std::process::Command;

fn check_probe(target: &str, probe: &str) -> std::process::Output {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
    let fixture = tempfile::Builder::new()
        .prefix("awscc541-boundary-")
        .tempdir()
        .unwrap();
    let package = fixture.path().join("fixture");
    std::fs::create_dir_all(package.join("src/bin")).unwrap();
    let workspace = std::fs::read_to_string(root.join("Cargo.toml"))
        .unwrap()
        .replace("\"carina-provider-awscc\",", "\"fixture\",");
    std::fs::write(fixture.path().join("Cargo.toml"), workspace).unwrap();
    std::fs::copy(root.join("Cargo.lock"), fixture.path().join("Cargo.lock")).unwrap();
    std::fs::copy(
        root.join("carina-provider-awscc/Cargo.toml"),
        package.join("Cargo.toml"),
    )
    .unwrap();
    let source = root.join("carina-provider-awscc/src");
    for entry in std::fs::read_dir(&source).unwrap() {
        let entry = entry.unwrap();
        if entry.file_name() != "lib.rs" && entry.file_name() != "bin" {
            std::os::unix::fs::symlink(entry.path(), package.join("src").join(entry.file_name()))
                .unwrap();
        }
    }
    std::os::unix::fs::symlink(source.join("bin/codegen"), package.join("src/bin/codegen"))
        .unwrap();
    for file in ["lib.rs", "bin/codegen.rs"] {
        let mut code = std::fs::read_to_string(source.join(file)).unwrap();
        if (target == "--lib") == (file == "lib.rs") {
            code.push_str(probe);
        }
        std::fs::write(package.join("src").join(file), code).unwrap();
    }

    // Reuse compiler artifacts in place; never copy target/. The caller can
    // put a resource-gated cargo wrapper on PATH (including for nested checks).
    let executable = std::env::current_exe().unwrap();
    let target_dir: PathBuf = executable
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .into();
    let mut command = Command::new("cargo");
    command
        .current_dir(fixture.path())
        .env("RUSTC_WRAPPER", "")
        .env("CARGO_BUILD_RUSTC_WRAPPER", "")
        .env("CARGO_BUILD_JOBS", "8")
        .args([
            "check",
            "--offline",
            "-p",
            "carina-provider-awscc",
            "-j",
            "8",
            "--target-dir",
        ])
        .arg(target_dir);
    if target == "--lib" {
        command.arg(target);
    } else {
        command.args(["--bin", target]);
    }
    command.output().unwrap()
}

#[test]
fn unchecked_config_cannot_construct_a_reference_client_inside_the_crate() {
    let output = check_probe(
        "--lib",
        r#"
#[allow(dead_code)]
fn unchecked_reference_entry(config: &BoundedSdkConfig) {
    let _ = references::KmsClient::from_bounded_config(config);
    let _ = config.clients().kms;
    let _ = references::KmsClient::new(config);
}
"#,
    );
    assert!(
        !output.status.success(),
        "unchecked config still constructs a usable KMS client"
    );
    let error = String::from_utf8_lossy(&output.stderr);
    assert!(
        error.contains("from_bounded_config") && error.contains("no field `kms`"),
        "{error}"
    );
    assert!(
        error.contains("private associated function") && error.contains("InitializedState"),
        "{error}"
    );
    let output = check_probe(
        "--lib",
        r#"
#[allow(dead_code)]
fn forge_initialized_state(config: &BoundedSdkConfig) {
    let _ = references::InitializedState {
        config: config.clone(),
        kms_client: std::sync::OnceLock::new(),
    };
}
"#,
    );
    assert!(
        !output.status.success(),
        "initialized state can still be forged"
    );
    let error = String::from_utf8_lossy(&output.stderr);
    assert!(
        error.contains("private") && error.contains("InitializedState"),
        "{error}"
    );
}

#[test]
fn raw_string_emission_cannot_be_paired_with_an_unrelated_classified_schema() {
    let output = check_probe(
        "codegen",
        r#"
#[allow(dead_code)]
fn unchecked_string_entry(schema: &ClassifiedSchema<'_>, foreign: &CfnProperty) {
    let _ = string_leaf(foreign, "KmsFoo", schema, &["Unknown".into()], CfnPropertyPath::new(None, "KmsFoo"), None);
}
"#,
    );
    assert!(
        !output.status.success(),
        "raw string emission still accepts a foreign property and unchecked path"
    );
    let error = String::from_utf8_lossy(&output.stderr);
    assert!(
        error.contains("string_leaf") && error.contains("Classified"),
        "{error}"
    );
    let output = check_probe(
        "codegen",
        r#"
#[allow(dead_code)]
fn unchecked_override_entry(schema: &ClassifiedSchema<'_>, foreign: &CfnProperty) {
    let _ = generate_top_level_attribute_type("GroupName", foreign, schema, "aws.scheduler.Schedule", &std::collections::BTreeMap::new(), false);
}
"#,
    );
    assert!(
        !output.status.success(),
        "type overrides accept a foreign property"
    );
    let error = String::from_utf8_lossy(&output.stderr);
    assert!(
        error.contains("generate_top_level_attribute_type") && error.contains("Classified"),
        "{error}"
    );
}
