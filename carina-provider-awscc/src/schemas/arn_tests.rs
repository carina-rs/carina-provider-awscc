use std::path::{Path, PathBuf};

use carina_core::resource::{ConcreteValue, Value};
use carina_core::schema::{AttributeType, CompiledPattern, RawShape, TypeIdentity};

use super::catalog::{self, arn};
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

fn shared_arn_calls(source: &str) -> Vec<(usize, &str)> {
    let calls = regex::Regex::new(
        r"carina_aws_types::(?:([A-Za-z_][A-Za-z0-9_]*)\s*\(|AwsType::([A-Za-z_][A-Za-z0-9_]*|\{[^}\n]+\})\.attribute_type\s*\()",
    ).unwrap();
    calls
        .captures_iter(source)
        .filter_map(|capture| {
            let name = capture.get(1).or_else(|| capture.get(2)).unwrap().as_str();
            // The cross-provider comparison reference is intentionally shared (#541).
            if name == "kms_key_reference" {
                return None;
            }
            (name.ends_with("Arn")
                || name.starts_with('{')
                || name.split('_').any(|part| part == "arn"))
            .then(|| (capture.get(0).unwrap().start(), name))
        })
        .collect()
}

#[test]
fn source_scan_rejects_shared_arn_constructors_and_enum_bypasses() {
    let prefix = concat!("carina_aws_types", "::");
    for name in [
        "arn",
        "iam_role_arn",
        "iam_policy_arn",
        "iam_oidc_provider_arn",
        "kms_key_arn",
        "sso_instance_arn",
        "sso_permission_set_arn",
        "future_service_arn",
        "validate_arn",
        "kms_key_arn_reference",
    ] {
        assert_eq!(
            shared_arn_calls(&format!("{prefix}{name}()")).len(),
            1,
            "{name}"
        );
    }
    assert_eq!(
        shared_arn_calls(&format!("{prefix}AwsType::IamRoleArn.attribute_type()")).len(),
        1
    );
    assert_eq!(
        shared_arn_calls(&format!("{prefix}AwsType::{{aws_type:?}}.attribute_type()")).len(),
        1
    );
    for source in [
        format!("{prefix}kms_key_reference()"),
        format!("{prefix}AwsType::IamRoleArn.identity()"),
        "crate::schemas::catalog::iam_role_arn()".to_string(),
    ] {
        assert!(shared_arn_calls(&source).is_empty(), "{source}");
    }
}

#[test]
fn schema_sources_use_awscc_generic_arn_constructor() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut sources = Vec::new();
    collect_rust_sources(&root, &mut sources);
    sources.sort();
    let mut failures = Vec::new();
    for path in sources {
        let source = std::fs::read_to_string(&path).unwrap();
        for (offset, name) in shared_arn_calls(&source) {
            failures.push(format!(
                "{}:{}: {name}",
                path.strip_prefix(&root).unwrap().display(),
                source[..offset]
                    .bytes()
                    .filter(|byte| *byte == b'\n')
                    .count()
                    + 1
            ));
        }
    }
    assert!(
        failures.is_empty(),
        "Schema/codegen sources must use AWSCC ARN constructors (only kms_key_reference is shared, #541):\n{}",
        failures.join("\n")
    );
}

#[test]
fn specialized_arn_rows_accept_every_awscc_partition() {
    struct ArnRow {
        constructor: fn() -> AttributeType,
        segments: &'static [&'static str],
        rest: &'static str,
        sample: &'static str,
        has_callback: bool,
    }

    let rows = [
        ArnRow {
            constructor: catalog::iam_role_arn,
            segments: &["iam", "Role"],
            rest: "iam::[^:]*:role/.+$",
            sample: "arn:aws:iam::123456789012:role/service-role/example",
            has_callback: true,
        },
        ArnRow {
            constructor: catalog::iam_policy_arn,
            segments: &["iam", "Policy"],
            rest: "iam::[^:]*:policy/.+$",
            sample: "arn:aws:iam::aws:policy/service-role/example",
            has_callback: true,
        },
        ArnRow {
            constructor: catalog::iam_oidc_provider_arn,
            segments: &["iam", "OidcProvider"],
            rest: "iam::[^:]*:oidc-provider/.+$",
            sample: "arn:aws:iam::123456789012:oidc-provider/example.com",
            has_callback: true,
        },
        ArnRow {
            constructor: catalog::kms_key_arn,
            segments: &["kms", "Key"],
            rest: "kms:[a-z]{2,4}(-[a-z0-9]+)+-[0-9]+:[0-9]{12}:key/([0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12}|mrk-[0-9a-fA-F]{32})$",
            sample: "arn:aws:kms:us-east-1:123456789012:key/12345678-1234-1234-1234-123456789abc",
            has_callback: true,
        },
        ArnRow {
            constructor: catalog::sso_instance_arn,
            segments: &["sso", "Instance"],
            rest: "sso:::instance/.+$",
            sample: "arn:aws:sso:::instance/ssoins-1234567890abcdef",
            has_callback: false,
        },
        ArnRow {
            constructor: catalog::sso_permission_set_arn,
            segments: &["sso", "PermissionSet"],
            rest: "sso:::permissionSet/.+$",
            sample: "arn:aws:sso:::permissionSet/ssoins-1234567890abcdef/ps-1234567890abcdef",
            has_callback: false,
        },
    ];
    let mut failures = Vec::new();
    for ArnRow {
        constructor,
        segments,
        rest,
        sample,
        has_callback,
    } in rows
    {
        let ty = constructor();
        let RawShape::String {
            identity,
            pattern: Some(pattern),
            length,
            validate,
            ..
        } = ty.raw_shape()
        else {
            panic!("{segments:?} must be a refined string");
        };
        assert_eq!(
            identity,
            Some(&TypeIdentity {
                provider: Some("aws".into()),
                segments: segments
                    .iter()
                    .map(|segment| (*segment).to_string())
                    .collect(),
                kind: "Arn".into()
            })
        );
        assert_eq!(length, None);
        assert_eq!(
            validate.is_some(),
            has_callback,
            "{segments:?} callback metadata"
        );
        assert_eq!(
            pattern,
            crate::partition::arn_regex(rest),
            "{segments:?}: partition builder, resource grammar and anchors"
        );
        let pattern = CompiledPattern::try_new(pattern).unwrap();
        for partition in Partition::ALL {
            let value = sample.replacen("arn:aws:", &format!("arn:{}:", partition.id()), 1);
            if !pattern.is_match(&value) {
                failures.push(format!("{segments:?}: pattern rejected {}", partition.id()));
            }
            if let Some(validate) = validate
                && let Err(error) = validate(&Value::Concrete(ConcreteValue::String(value.into())))
            {
                failures.push(format!("{segments:?}: {error}"));
            }
        }
        let invalid = sample.replacen("arn:aws:", "arn:aws-xx:", 1);
        assert!(!pattern.is_match(&invalid), "{segments:?} accepted aws-xx");
        if let Some(validate) = validate {
            assert!(validate(&Value::Concrete(ConcreteValue::String(invalid.into()))).is_err());
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn specialized_iam_and_kms_arns_keep_validation_diagnostics() {
    for (constructor, name, prefix) in [
        (
            catalog::iam_role_arn as fn() -> AttributeType,
            "Role",
            "role/",
        ),
        (
            catalog::iam_policy_arn as fn() -> AttributeType,
            "Policy",
            "policy/",
        ),
    ] {
        let ty = constructor();
        let RawShape::String {
            validate: Some(validate),
            ..
        } = ty.raw_shape()
        else {
            panic!("IAM ARN must have a callback");
        };
        for (value, reason) in [
            (
                "not-an-arn".to_string(),
                "must start with 'arn:'".to_string(),
            ),
            (
                format!("arn:aws:s3::123456789012:{prefix}example"),
                format!("expected IAM {name} ARN but service is 's3'"),
            ),
            (
                format!("arn:aws:iam:us-east-1:123456789012:{prefix}example"),
                format!("IAM {name} ARN region must be empty"),
            ),
            (
                format!("arn:aws:iam::123:{prefix}example"),
                format!("IAM {name} ARN account must be 'aws' or a 12-digit ID"),
            ),
            (
                format!("arn:aws:iam::123456789012:{prefix}"),
                format!("IAM {name} ARN name after '{prefix}' must not be empty"),
            ),
            (
                format!("arn:aws:iam::123456789012:{prefix}has space"),
                format!("IAM {name} ARN name contains invalid characters"),
            ),
            (
                "arn:aws:iam::123456789012:user/example".to_string(),
                format!("IAM {name} ARN resource must begin with '{prefix}'"),
            ),
        ] {
            let error = validate(&Value::Concrete(ConcreteValue::String(value.into())))
                .unwrap_err()
                .to_string();
            assert!(
                error.starts_with(&format!("Validation failed: {reason}")),
                "{error}"
            );
        }
        assert_eq!(
            validate(&Value::Concrete(ConcreteValue::Int(1)))
                .unwrap_err()
                .to_string(),
            "Validation failed: Expected string"
        );
    }
    let ty = catalog::kms_key_arn();
    let RawShape::String {
        pattern: Some(pattern),
        validate: Some(validate),
        ..
    } = ty.raw_shape()
    else {
        panic!("KMS ARN must have a pattern and callback");
    };
    let pattern = CompiledPattern::try_new(pattern).unwrap();
    for key in [
        "1234ABCD-12AB-34CD-56EF-1234567890AB",
        "mrk-0123456789abcdef0123456789abcdef",
    ] {
        let value = format!("arn:aws-eusc:kms:eusc-de-east-1:123456789012:key/{key}");
        assert!(pattern.is_match(&value));
        validate(&Value::Concrete(ConcreteValue::String(value.into()))).unwrap();
    }
    for value in [
        "arn:aws:kms:us-east-1:123456789012:alias/example",
        "arn:aws:kms:us-east-1:123456789012:key/not-a-uuid",
        "arn:aws:kms:us-east-1:123456789012:key/mrk-1234",
        "arn:aws:kms::123456789012:key/12345678-1234-1234-1234-123456789abc",
        "arn:aws:kms:us-east-1:123:key/12345678-1234-1234-1234-123456789abc",
        "arn:aws-xx:kms:us-east-1:123456789012:key/12345678-1234-1234-1234-123456789abc",
    ] {
        assert!(!pattern.is_match(value), "{value}");
        assert_eq!(
            validate(&Value::Concrete(ConcreteValue::String(value.into())))
                .unwrap_err()
                .to_string(),
            "Validation failed: expected a KMS key ARN with a UUID or multi-region key ID"
        );
    }
    assert_eq!(
        validate(&Value::Concrete(ConcreteValue::Int(1)))
            .unwrap_err()
            .to_string(),
        "Validation failed: Expected string"
    );
}

#[test]
fn sso_arns_keep_pattern_only_metadata_and_resource_language() {
    for (constructor, resource) in [
        (
            catalog::sso_instance_arn as fn() -> AttributeType,
            "instance",
        ),
        (
            catalog::sso_permission_set_arn as fn() -> AttributeType,
            "permissionSet",
        ),
    ] {
        let ty = constructor();
        let RawShape::String {
            pattern: Some(pattern),
            validate,
            ..
        } = ty.raw_shape()
        else {
            panic!("SSO ARN must have a pattern");
        };
        assert!(validate.is_none());
        let pattern = CompiledPattern::try_new(pattern).unwrap();
        assert!(pattern.is_match(&format!("arn:aws:sso:::{resource}/arbitrary/non-hex-id")));
        for invalid in [
            format!("arn:aws:sso:::{resource}/"),
            format!("arn:aws:sso:us-east-1:123456789012:{resource}/example"),
            format!("arn:aws:sso:::{resource}/example\n"),
        ] {
            assert!(!pattern.is_match(&invalid), "{invalid}");
        }
    }
}
