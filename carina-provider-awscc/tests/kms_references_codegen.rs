//! Offline checks for fail-closed KMS classification and generated reference leaves.
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

fn vpc_schema() -> Value {
    serde_json::from_str(include_str!("../../cfn-schema-cache/AWS__EC2__VPC.json")).unwrap()
}

#[test]
fn unclassified_kms_strings_fail_rust_and_markdown_generation() {
    for (name, property, path) in [
        ("KmsFoo", json!({"type": "string"}), "kms_foo"),
        ("CmkFoo", json!({"type": "string"}), "cmk_foo"),
        (
            "EncryptionKey",
            json!({"type": "string", "description": "The ARN of an AWS KMS key."}),
            "encryption_key",
        ),
        (
            "EncryptionKey",
            json!({"type": "string", "pattern": "^arn:aws:kms:.*$"}),
            "encryption_key",
        ),
    ] {
        let mut schema = vpc_schema();
        schema["properties"][name] = property;
        for format in ["rust", "markdown"] {
            let output = generate(&schema, format);
            assert!(
                !output.status.success(),
                "unclassified {name} passed {format}"
            );
            let error = String::from_utf8_lossy(&output.stderr);
            assert!(
                error.contains(&format!("unclassified KMS string ec2.Vpc.{path}")),
                "{error}"
            );
        }
    }
}

#[test]
fn classification_follows_shared_definitions_and_array_paths_before_overrides() {
    let mut schema: Value =
        serde_json::from_str(include_str!("../../cfn-schema-cache/AWS__S3__Bucket.json")).unwrap();
    schema["definitions"]["EncryptionConfiguration"]["properties"]["KmsFoo"] =
        json!({"$ref": "#/definitions/SharedKmsString"});
    schema["definitions"]["SharedKmsString"] = json!({"type": "string"});
    for format in ["rust", "markdown"] {
        let output = generate(&schema, format);
        assert!(
            !output.status.success(),
            "unclassified nested KmsFoo passed {format}"
        );
        let error = String::from_utf8_lossy(&output.stderr);
        assert!(error.contains("unclassified KMS string s3.Bucket.replication_configuration.rules[].destination.encryption_configuration.kms_foo"), "{error}");
    }
}

#[test]
fn scheduler_generates_key_or_alias_arn_reference() {
    let schema: Value = serde_json::from_str(include_str!(
        "../../cfn-schema-cache/AWS__Scheduler__Schedule.json"
    ))
    .unwrap();
    let output = generate(&schema, "rust");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8(output.stdout)
            .unwrap()
            .contains("carina_aws_types::kms_key_or_alias_arn_reference()")
    );
}

#[test]
fn final_scheduler_leaf_is_a_reference_accepting_alias_arns() {
    let config =
        carina_provider_awscc::schemas::generated::scheduler::schedule::scheduler_schedule_config();
    let schema = config.schema.as_schema();
    let attribute = schema.attributes().get("kms_key_arn").unwrap();
    let Shape::Reference {
        identity,
        pattern: Some(pattern),
        length,
        ..
    } = schema.shape_of(attribute.attr_type())
    else {
        panic!("scheduler kms_key_arn must be a reference leaf");
    };
    assert_eq!(identity.to_string(), "aws.kms.Key.Reference");
    assert_eq!(length, Some((Some(1), Some(2048))));
    let pattern = regex::Regex::new(pattern).unwrap();
    assert!(pattern.is_match("arn:aws:kms:us-east-1:123456789012:alias/customer"));
    assert!(
        pattern.is_match(
            "arn:aws:kms:us-east-1:123456789012:key/11111111-1111-1111-1111-111111111111"
        )
    );
    assert!(!pattern.is_match("alias/customer"));
}

#[test]
fn changed_kms_pattern_requires_reclassification() {
    let mut schema: Value = serde_json::from_str(include_str!(
        "../../cfn-schema-cache/AWS__Scheduler__Schedule.json"
    ))
    .unwrap();
    schema["properties"]["KmsKeyArn"]["pattern"] = json!("^arbitrary-new-constraint$");
    for format in ["rust", "markdown"] {
        let output = generate(&schema, format);
        assert!(!output.status.success());
        assert!(String::from_utf8_lossy(&output.stderr).contains("KMS pattern changed"));
    }
}

#[test]
fn reference_inside_list_struct_preserves_the_array_constraints() {
    let mut schema: Value =
        serde_json::from_str(include_str!("../../cfn-schema-cache/AWS__S3__Bucket.json")).unwrap();
    let array = &mut schema["definitions"]["BucketEncryption"]["properties"]["ServerSideEncryptionConfiguration"];
    assert_eq!(array["type"], "array");
    array["minItems"] = json!(1);
    array["maxItems"] = json!(3);
    array["uniqueItems"] = json!(true);
    array["insertionOrder"] = json!(false);
    let output = generate(&schema, "rust");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let code = String::from_utf8(output.stdout).unwrap();
    assert!(code.contains("carina_aws_types::kms_key_reference()"));
    assert!(
        code.contains(
            "false, Some((Some(1), Some(3))), carina_core::schema::ListElements::Unique)"
        )
    );
}

#[test]
fn shared_scalar_description_is_checked_at_the_input_path() {
    let mut schema = vpc_schema();
    schema["properties"]["EncryptionKey"] = json!({"$ref": "#/definitions/SharedString"});
    schema["definitions"]["SharedString"] =
        json!({"type": "string", "description": "The ARN of an AWS KMS key."});
    for format in ["rust", "markdown"] {
        let output = generate(&schema, format);
        assert!(!output.status.success());
        assert!(
            String::from_utf8_lossy(&output.stderr)
                .contains("unclassified KMS string ec2.Vpc.encryption_key")
        );
    }
}

#[test]
fn classification_checks_constraints_on_ref_uses() {
    let mut schema = vpc_schema();
    schema["properties"]["EncryptionKey"] = json!({
        "$ref": "#/definitions/SharedString", "pattern": "^arn:aws:kms:.*$",
    });
    schema["definitions"]["SharedString"] = json!({"type": "string"});
    for format in ["rust", "markdown"] {
        let output = generate(&schema, format);
        assert!(
            !output.status.success(),
            "a KMS pattern on a $ref use must be classified"
        );
        assert!(
            String::from_utf8_lossy(&output.stderr)
                .contains("unclassified KMS string ec2.Vpc.encryption_key")
        );
    }
}

#[test]
fn read_only_strings_and_non_string_kms_properties_keep_their_types() {
    let mut schema = vpc_schema();
    schema["properties"]["KmsOutput"] = json!({"type": "string"});
    schema["readOnlyProperties"]
        .as_array_mut()
        .unwrap()
        .push(json!("/properties/KmsOutput"));
    schema["properties"]["KmsEnabled"] = json!({"type": "boolean"});
    for format in ["rust", "markdown"] {
        let output = generate(&schema, format);
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
}
