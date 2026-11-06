use std::path::{Path, PathBuf};

use carina_core::resource::{ConcreteValue, Value};
use carina_core::schema::{
    AttributeType, CompiledPattern, CustomValidator, RawShape, TypeIdentity, legacy_validator,
};

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

struct SharedType {
    name: &'static str,
    constructor: fn() -> AttributeType,
}

// The identifiers are explicit: adding a shared type requires reviewing its
// constructor, not inferring safety from its name or its current source uses.
macro_rules! shared_type_allowlist {
    (functions: [$($function:ident),* $(,)?], rows: [$($row:ident),* $(,)?] $(,)?) => {
        &[
            $(SharedType {
                name: stringify!($function),
                constructor: carina_aws_types::$function,
            },)*
            $(SharedType {
                name: concat!("AwsType::", stringify!($row)),
                constructor: || carina_aws_types::AwsType::$row.attribute_type(),
            },)*
        ]
    };
}

const SHARED_TYPES: &[SharedType] = shared_type_allowlist! {
    functions: [
        allocation_id,
        availability_zone,
        availability_zone_id,
        aws_account_id,
        aws_region,
        aws_resource_id,
        carrier_gateway_id,
        egress_only_internet_gateway_id,
        elbv2_health_check_port,
        elbv2_target_group_grpc_code,
        elbv2_target_group_http_code,
        flow_log_id,
        gateway_id,
        iam_path,
        iam_policy_document,
        iam_policy_name,
        iam_role_id,
        iam_role_name,
        identity_store_id,
        instance_id,
        internet_gateway_id,
        ipam_id,
        ipam_pool_id,
        // The sole partition-bearing exception is the cross-provider KMS
        // comparison reference. It remains shared under #541.
        kms_key_reference,
        local_gateway_id,
        logs_log_group_retention_in_days,
        nat_gateway_id,
        network_acl_id,
        network_interface_id,
        prefix_list_id,
        route53_hosted_zone_id,
        route_table_id,
        security_group_id,
        sso_principal_id,
        subnet_id,
        subnet_route_table_association_id,
        tgw_route_table_id,
        transit_gateway_attachment_id,
        transit_gateway_id,
        vpc_cidr_block_association_id,
        vpc_endpoint_id,
        vpc_id,
        vpc_peering_connection_id,
        vpn_gateway_id,
    ],
    rows: [],
};

// These APIs produce strings, SDK configuration or completion data, rather
// than AttributeType values. New utility calls also require explicit review.
const SHARED_UTILITIES: &[&str] = &[
    "condition_operator_to_aws",
    "condition_operator_to_snake",
    "load_sdk_config",
    "region_completions",
];

fn shared_type_calls(source: &str) -> Vec<(usize, String)> {
    let calls = regex::Regex::new(
        r"(?:(?:carina_aws_types::)?AwsType::([A-Za-z_][A-Za-z0-9_]*|\{[^}\n]+\})\.attribute_type\s*\(|carina_aws_types::([A-Za-z_][A-Za-z0-9_]*)\s*\()",
    ).unwrap();
    calls
        .captures_iter(source)
        .map(|capture| {
            let name = capture.get(1).map_or_else(
                || capture.get(2).unwrap().as_str().to_string(),
                |row| format!("AwsType::{}", row.as_str()),
            );
            (capture.get(0).unwrap().start(), name)
        })
        .collect()
}

fn shared_call_is_allowlisted(name: &str) -> bool {
    SHARED_TYPES.iter().any(|entry| entry.name == name) || SHARED_UTILITIES.contains(&name)
}

fn assert_callback_has_no_partition_decision(name: &str, validate: &CustomValidator) {
    let sample = "arn:aws:kms:us-east-1:123456789012:key/12345678-1234-1234-1234-123456789abc";
    let baseline = validate(&Value::Concrete(ConcreteValue::String(sample.into()))).is_ok();
    for partition in Partition::ALL
        .map(Partition::id)
        .into_iter()
        .chain(["aws-xx"])
    {
        let value = sample.replacen("arn:aws:", &format!("arn:{partition}:"), 1);
        let result = validate(&Value::Concrete(ConcreteValue::String(value.into())));
        assert_eq!(
            result.is_ok(),
            baseline,
            "{name} validator makes a partition decision for {partition}"
        );
        if let Err(error) = result {
            assert!(
                !error.to_string().to_ascii_lowercase().contains("partition"),
                "{name}: {error}"
            );
        }
    }
}

fn assert_shared_type_has_no_partition_decision(name: &str, ty: &AttributeType) {
    match ty.raw_shape() {
        RawShape::String {
            validate: Some(validate),
            ..
        }
        | RawShape::Int {
            validate: Some(validate),
            ..
        }
        | RawShape::Float {
            validate: Some(validate),
            ..
        }
        | RawShape::Enum {
            validate: Some(validate),
            ..
        } => {
            assert_callback_has_no_partition_decision(name, validate);
        }
        _ => {}
    }
    match ty.raw_shape() {
        RawShape::String { pattern, .. } | RawShape::Reference { pattern, .. } => {
            if let Some(pattern) = pattern {
                assert!(
                    !pattern.contains("arn:"),
                    "{name} carries an ARN pattern: {pattern}"
                );
            }
        }
        RawShape::Enum { base, .. } => assert_shared_type_has_no_partition_decision(name, base),
        RawShape::List { element_type, .. } => {
            assert_shared_type_has_no_partition_decision(name, element_type)
        }
        RawShape::Map { key, value } => {
            assert_shared_type_has_no_partition_decision(name, key);
            assert_shared_type_has_no_partition_decision(name, value);
        }
        RawShape::Struct { fields, .. } => {
            for field in fields {
                assert_shared_type_has_no_partition_decision(name, field.field_type());
            }
        }
        RawShape::Union(members) => {
            for member in members {
                assert_shared_type_has_no_partition_decision(name, member);
            }
        }
        RawShape::Ref(_) | RawShape::UnloadedSchemaType(_) => {
            panic!("{name} needs resolved definitions before it can be allowlisted");
        }
        RawShape::Int { .. } | RawShape::Float { .. } | RawShape::Bool | RawShape::Duration => {}
    }
}

#[test]
fn shared_type_allowlist_has_no_partition_decisions() {
    let mut names = std::collections::BTreeSet::new();
    for entry in SHARED_TYPES {
        assert!(
            names.insert(entry.name),
            "duplicate allowlist entry {}",
            entry.name
        );
        let ty = (entry.constructor)();
        if entry.name == "kms_key_reference" {
            // #541: the one deliberately shared partition-bearing type.
            assert!(
                matches!(ty.raw_shape(), RawShape::Reference { pattern: Some(pattern), .. } if pattern.contains("arn:"))
            );
        } else {
            assert_shared_type_has_no_partition_decision(entry.name, &ty);
        }
    }
    assert!(names.contains("kms_key_reference"));
}

#[test]
fn source_scan_requires_explicit_shared_type_allowlist() {
    let prefix = concat!("carina_aws_types", "::");
    let enum_prefix = concat!("AwsType", "::");
    for name in ["arn", "iam_role_arn", "kms_key_id", "future_identifier"] {
        let calls = shared_type_calls(&format!("{prefix}{name}()"));
        assert_eq!(calls.len(), 1);
        assert!(!shared_call_is_allowlisted(&calls[0].1), "{name}");
    }
    for source in [
        format!("{prefix}{enum_prefix}KmsKeyId.attribute_type()"),
        format!("{enum_prefix}KmsKeyId.attribute_type()"),
        format!("{prefix}{enum_prefix}{{aws_type:?}}.attribute_type()"),
    ] {
        let calls = shared_type_calls(&source);
        assert_eq!(calls.len(), 1);
        assert!(!shared_call_is_allowlisted(&calls[0].1), "{source}");
    }
    for entry in SHARED_TYPES {
        let source = if entry.name.starts_with(enum_prefix) {
            format!("{prefix}{}.attribute_type()", entry.name)
        } else {
            format!("{prefix}{}()", entry.name)
        };
        let calls = shared_type_calls(&source);
        assert_eq!(calls.len(), 1);
        assert!(shared_call_is_allowlisted(&calls[0].1));
    }
    assert!(shared_type_calls(&format!("{prefix}AwsType::IamRoleArn.identity()")).is_empty());
    assert!(shared_type_calls("crate::schemas::catalog::kms_key_id()").is_empty());
}

#[test]
fn shared_type_audit_rejects_arn_patterns_and_hidden_partition_validators() {
    assert!(
        std::panic::catch_unwind(|| {
            let ty = AttributeType::list(AttributeType::refined_string(
                None,
                Some("^arn:.*$".into()),
                None,
                None,
            ));
            assert_shared_type_has_no_partition_decision("innocent_name", &ty);
        })
        .is_err()
    );
    assert!(
        std::panic::catch_unwind(|| {
            let ty = AttributeType::refined_string_with_validator(
                TypeIdentity::bare("Hidden"),
                None,
                None,
                legacy_validator(|value| {
                    if let Value::Concrete(ConcreteValue::String(value)) = value
                        && value.expose_plaintext().starts_with("arn:aws:")
                    {
                        Ok(())
                    } else {
                        Err("unsupported value".to_string())
                    }
                }),
                None,
            );
            assert_shared_type_has_no_partition_decision("innocent_name", &ty);
        })
        .is_err()
    );
}

#[test]
fn schema_sources_use_only_allowlisted_shared_types() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut sources = Vec::new();
    collect_rust_sources(&root, &mut sources);
    sources.sort();
    let mut failures = Vec::new();
    for path in sources {
        let source = std::fs::read_to_string(&path).unwrap();
        for (offset, name) in shared_type_calls(&source) {
            if !shared_call_is_allowlisted(&name) {
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
    }
    assert!(
        failures.is_empty(),
        "Shared type uses need an explicit partition-free allowlist entry:\n{}",
        failures.join("\n")
    );
}

#[test]
fn kms_key_id_accepts_every_awscc_partition() {
    let ty = catalog::kms_key_id();
    let RawShape::String {
        identity,
        pattern: Some(pattern),
        length,
        validate: Some(validate),
        ..
    } = ty.raw_shape()
    else {
        panic!("KMS key ID must have a pattern and callback");
    };
    assert_eq!(
        identity,
        Some(&TypeIdentity {
            provider: Some("aws".into()),
            segments: vec!["kms".into(), "Key".into()],
            kind: "Id".into()
        })
    );
    assert_eq!(length, None);
    let pattern = CompiledPattern::try_new(pattern).unwrap();
    let mut failures = Vec::new();
    for partition in Partition::ALL {
        for resource in ["key/12345678-1234-1234-1234-123456789abc", "alias/example"] {
            let value = format!(
                "arn:{}:kms:us-east-1:123456789012:{resource}",
                partition.id()
            );
            if !pattern.is_match(&value) {
                failures.push(format!("pattern rejected {value}"));
            }
            if let Err(error) = validate(&Value::Concrete(ConcreteValue::String(value.into()))) {
                failures.push(error.to_string());
            }
        }
    }
    let invalid = "arn:aws-xx:kms:us-east-1:123456789012:key/example";
    assert!(!pattern.is_match(invalid));
    assert!(validate(&Value::Concrete(ConcreteValue::String(invalid.into()))).is_err());
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn kms_key_id_keeps_alias_uuid_grammar_and_validation_diagnostics() {
    let ty = catalog::kms_key_id();
    let RawShape::String {
        pattern: Some(pattern),
        validate: Some(validate),
        ..
    } = ty.raw_shape()
    else {
        panic!("KMS key ID must have a pattern and callback");
    };
    let pattern = CompiledPattern::try_new(pattern).unwrap();
    for value in [
        "alias/example",
        "alias/arbitrary name/with spaces",
        "12345678-1234-1234-1234-123456789abc",
        "12345678-1234-ABCD-abcd-123456789ABC",
        "arn:aws:kms:::key/arbitrary",
        "arn:aws:kms:::alias/arbitrary",
    ] {
        assert!(pattern.is_match(value), "{value}");
        validate(&Value::Concrete(ConcreteValue::String(value.into()))).unwrap();
    }
    for (value, reason) in [
        ("alias/", "missing alias name after 'alias/'"),
        (
            "arn:aws:kms",
            "must have at least 6 colon-separated parts (arn:partition:service:region:account:resource)",
        ),
        ("arn:aws:s3:::key/example", "expected kms service, got 's3'"),
        (
            "arn:aws:kms:::other/example",
            "KMS ARN resource 'other/example' must start with 'key/' or 'alias/'",
        ),
        (
            "arn:aws::::key/example",
            "service must not be empty (e.g., 's3', 'iam', 'ec2')",
        ),
        (
            "mrk-0123456789abcdef0123456789abcdef",
            "expected a key ARN, alias ARN, alias name (alias/...), or key ID (UUID format)",
        ),
        (
            "invalid",
            "expected a key ARN, alias ARN, alias name (alias/...), or key ID (UUID format)",
        ),
        (
            "1234567g-1234-1234-1234-123456789abc",
            "expected a key ARN, alias ARN, alias name (alias/...), or key ID (UUID format)",
        ),
    ] {
        assert!(!pattern.is_match(value), "{value}");
        assert_eq!(
            validate(&Value::Concrete(ConcreteValue::String(value.into())))
                .unwrap_err()
                .to_string(),
            format!("Validation failed: {reason}")
        );
    }
    // The shared callback checks only the ARN resource prefix, while the
    // pattern separately requires a non-empty name. Preserve that distinction.
    for value in ["arn:aws:kms:::key/", "arn:aws:kms:::alias/"] {
        assert!(!pattern.is_match(value));
        validate(&Value::Concrete(ConcreteValue::String(value.into()))).unwrap();
    }
    assert_eq!(
        validate(&Value::Concrete(ConcreteValue::Int(1)))
            .unwrap_err()
            .to_string(),
        "Validation failed: Expected string"
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
