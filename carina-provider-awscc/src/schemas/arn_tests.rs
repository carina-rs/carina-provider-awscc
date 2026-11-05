use std::path::{Path, PathBuf};

use carina_core::resource::{ConcreteValue, Value};
use carina_core::schema::{CompiledPattern, RawShape, TypeIdentity};

use super::catalog::arn;
use crate::partition::Partition;

#[test]
fn generic_arn_accepts_every_awscc_partition() {
    let attribute_type = arn();
    let RawShape::String {
        pattern: Some(pattern),
        validate: Some(validate),
        ..
    } = attribute_type.raw_shape()
    else {
        panic!("Generic ARN must carry a string pattern and validator");
    };
    let pattern = CompiledPattern::try_new(pattern).unwrap();
    let mut failures = Vec::new();
    for partition in Partition::ALL {
        let sample = format!("arn:{}:s3:::example-bucket", partition.id());
        if !pattern.is_match(&sample) {
            failures.push(format!("Pattern rejected {sample}"));
        }
        if let Err(error) = validate(&Value::Concrete(ConcreteValue::String(sample.into()))) {
            failures.push(error.to_string());
        }
    }
    let unknown = "arn:aws-xx:s3:::example-bucket";
    assert!(!pattern.is_match(unknown));
    assert!(validate(&Value::Concrete(ConcreteValue::String(unknown.into()))).is_err());
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn generic_arn_preserves_identity_and_validation_semantics() {
    let attribute_type = arn();
    let RawShape::String {
        identity,
        pattern: Some(pattern),
        length,
        validate: Some(validate),
        ..
    } = attribute_type.raw_shape()
    else {
        panic!("Generic ARN must carry a string pattern and validator");
    };
    assert_eq!(
        identity,
        Some(&TypeIdentity {
            provider: Some("aws".to_string()),
            segments: Vec::new(),
            kind: "Arn".to_string(),
        })
    );
    assert_eq!(length, None);
    let pattern = CompiledPattern::try_new(pattern).unwrap();
    for sample in [
        "arn:aws:s3:::example-bucket",
        "arn:aws:custom:::resource:with:colons",
        "arn:aws:custom:::日本語",
        "arn:aws:custom:::",
    ] {
        assert!(pattern.is_match(sample), "{sample}");
        validate(&Value::Concrete(ConcreteValue::String(sample.into()))).unwrap();
    }
    for (sample, reason) in [
        ("not-an-arn", "must start with 'arn:'"),
        (
            "arn:aws:s3:short",
            "must have at least 6 colon-separated parts (arn:partition:service:region:account:resource)",
        ),
        (
            "arn:aws::::resource",
            "service must not be empty (e.g., 's3', 'iam', 'ec2')",
        ),
    ] {
        let error = validate(&Value::Concrete(ConcreteValue::String(sample.into()))).unwrap_err();
        assert_eq!(error.to_string(), format!("Validation failed: {reason}"));
    }
    let error = validate(&Value::Concrete(ConcreteValue::Int(1))).unwrap_err();
    assert_eq!(error.to_string(), "Validation failed: Expected string");
    assert!(!pattern.is_match("prefix-arn:aws:s3:::example-bucket"));
    assert!(!pattern.is_match("arn:aws:s3:::example-bucket\n"));
    assert!(!pattern.is_match("arn:aws::::resource"));
}

fn collect_rust_sources(directory: &Path, sources: &mut Vec<PathBuf>) {
    for entry in std::fs::read_dir(directory).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            collect_rust_sources(&path, sources);
        } else if path.extension().is_some_and(|extension| extension == "rs") {
            sources.push(path);
        }
    }
}

#[test]
fn schema_sources_use_awscc_generic_arn_constructor() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut sources = Vec::new();
    collect_rust_sources(&root, &mut sources);
    sources.sort();
    let legacy_constructor = concat!("carina_aws_types::", "arn()");
    let mut failures = Vec::new();
    for path in sources {
        let source = std::fs::read_to_string(&path).unwrap();
        for (line, text) in source.lines().enumerate() {
            if text.contains(legacy_constructor) {
                failures.push(format!(
                    "{}:{}",
                    path.strip_prefix(&root).unwrap().display(),
                    line + 1
                ));
            }
        }
    }
    assert!(
        failures.is_empty(),
        "Schema/codegen sources must use the AWSCC generic ARN constructor:\n{}",
        failures.join("\n")
    );
}
