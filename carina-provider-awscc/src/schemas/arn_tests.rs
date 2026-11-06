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
    (functions: [$($module:ident::$function:ident),* $(,)?], rows: [$($row_module:ident::AwsType::$row:ident),* $(,)?] $(,)?) => {
        &[
            $(SharedType {
                name: stringify!($function),
                constructor: $module::$function,
            },)*
            $(SharedType {
                name: concat!("AwsType::", stringify!($row)),
                constructor: || $row_module::AwsType::$row.attribute_type(),
            },)*
        ]
    };
}

const SHARED_TYPES: &[SharedType] = shared_type_allowlist! {
    functions: [
        carina_aws_types::allocation_id,
        carina_aws_types::availability_zone,
        carina_aws_types::availability_zone_id,
        carina_aws_types::aws_account_id,
        carina_aws_types::aws_region,
        carina_aws_types::aws_resource_id,
        carina_aws_types::carrier_gateway_id,
        carina_aws_types::egress_only_internet_gateway_id,
        carina_aws_types::elbv2_health_check_port,
        carina_aws_types::elbv2_target_group_grpc_code,
        carina_aws_types::elbv2_target_group_http_code,
        carina_aws_types::flow_log_id,
        carina_aws_types::gateway_id,
        carina_aws_types::iam_path,
        carina_aws_types::iam_policy_document,
        carina_aws_types::iam_policy_name,
        carina_aws_types::iam_role_id,
        carina_aws_types::iam_role_name,
        carina_aws_types::identity_store_id,
        carina_aws_types::instance_id,
        carina_aws_types::internet_gateway_id,
        carina_aws_types::ipam_id,
        carina_aws_types::ipam_pool_id,
        // The sole partition-bearing exception is the cross-provider KMS
        // comparison reference. It remains shared under #541.
        carina_aws_types::kms_key_reference,
        carina_aws_types::local_gateway_id,
        carina_aws_types::logs_log_group_retention_in_days,
        carina_aws_types::nat_gateway_id,
        carina_aws_types::network_acl_id,
        carina_aws_types::network_interface_id,
        carina_aws_types::prefix_list_id,
        carina_aws_types::route53_hosted_zone_id,
        carina_aws_types::route_table_id,
        carina_aws_types::security_group_id,
        carina_aws_types::sso_principal_id,
        carina_aws_types::subnet_id,
        carina_aws_types::subnet_route_table_association_id,
        carina_aws_types::tgw_route_table_id,
        carina_aws_types::transit_gateway_attachment_id,
        carina_aws_types::transit_gateway_id,
        carina_aws_types::vpc_cidr_block_association_id,
        carina_aws_types::vpc_endpoint_id,
        carina_aws_types::vpc_id,
        carina_aws_types::vpc_peering_connection_id,
        carina_aws_types::vpn_gateway_id,
    ],
    rows: [],
};

// These items supply metadata, SDK support or non-type utilities. They are
// exact paths, not prefixes: new items also require explicit review.
const SHARED_UTILITIES: &[&str] = &[
    "AssumeRoleFailure",
    "AwsType",
    "AwsType::ALL",
    "AwsType::Ec2FlowLogMaxAggregationInterval",
    "AwsType::IamOidcProviderArn",
    "AwsType::IamPolicyName",
    "AwsType::IamRoleName",
    "AwsType::KmsKeyArn",
    "AwsType::KmsKeyReference",
    "AwsType::LogsLogGroupRetentionInDays",
    "AwsType::OrganizationsAccountArn",
    "AwsType::OrganizationsAccountEmail",
    "AwsType::OrganizationsOrganizationArn",
    "AwsType::Region",
    "AwsType::S3BucketArn",
    "AwsType::identity",
    "FinalAttempt",
    "GuardedAssumeRoleProvider::new",
    "Idempotency",
    // Part of the existing #541 KMS reference exception, not another type.
    "KMS_KEY_REFERENCE_PATTERN",
    "ResendGuard",
    "SendClassification",
    "SendClassification::MaybeSent",
    "Transience",
    "canonicalize_enum_value",
    "condition_operator_to_aws",
    "condition_operator_to_snake",
    "define_aws_type_catalog",
    "dsl_enum_value",
    "find_matching_enum_value",
    "kms_reference::DescribeKeyClient",
    "kms_reference::DescribeKeyFailure",
    "kms_reference::describe_key_failure",
    "kms_reference::resolve_core_reference_batch",
    "kms_reference::resolve_reference_batch",
    "load_sdk_config",
    "region_completions",
    "validate_iam_policy_document",
];

// SDK access is deliberately shared. No other module gets an implicit exemption
// for its descendants (in particular, not AwsType or kms_reference).
const SHARED_MODULE_PREFIXES: &[&str] = &["sdk"];

const SHARED_CRATE_PREFIX: &str = "carina_aws_types::";

fn shared_crate_identifiers() -> regex::Regex {
    regex::Regex::new(&format!(
        r"\b{}\b",
        SHARED_CRATE_PREFIX.trim_end_matches("::")
    ))
    .unwrap()
}

struct SharedPathParser<'a> {
    source: &'a str,
    cursor: usize,
    renames: Vec<usize>,
}

impl SharedPathParser<'_> {
    fn skip_trivia(&mut self) {
        loop {
            let rest = &self.source[self.cursor..];
            self.cursor += rest.len() - rest.trim_start().len();
            let rest = &self.source[self.cursor..];
            if rest.starts_with("//") {
                self.cursor += rest.find('\n').unwrap_or(rest.len());
            } else if rest.starts_with("/*") {
                self.cursor += 2;
                let mut depth = 1;
                while depth > 0 && self.cursor < self.source.len() {
                    let rest = &self.source[self.cursor..];
                    if rest.starts_with("/*") {
                        depth += 1;
                        self.cursor += 2;
                    } else if rest.starts_with("*/") {
                        depth -= 1;
                        self.cursor += 2;
                    } else {
                        self.cursor += rest.chars().next().unwrap().len_utf8();
                    }
                }
            } else if ["\\n", "\\r", "\\t"]
                .iter()
                .any(|escape| rest.starts_with(escape))
            {
                // Codegen also emits use trees inside escaped string literals.
                self.cursor += 2;
            } else {
                break;
            }
        }
    }

    fn take(&mut self, token: &str) -> bool {
        self.skip_trivia();
        if self.source[self.cursor..].starts_with(token) {
            self.cursor += token.len();
            true
        } else {
            false
        }
    }

    fn identifier(&mut self) -> Option<&str> {
        self.skip_trivia();
        self.take("r#");
        let start = self.cursor;
        for character in self.source[start..].chars() {
            if character == '_' || character.is_alphanumeric() {
                self.cursor += character.len_utf8();
            } else {
                break;
            }
        }
        (self.cursor > start).then_some(&self.source[start..self.cursor])
    }

    // Flatten paths for the allowlist. Record rename syntax so the source rules
    // can reject it; never resolve or follow an alias.
    fn tree(&mut self, prefix: &str, references: &mut Vec<(usize, String)>) -> Result<(), ()> {
        let prefix = if self.take("::") { "" } else { prefix };
        if self.take("{") {
            while !self.take("}") {
                self.tree(prefix, references)?;
                if !self.take(",") {
                    return self.take("}").then_some(()).ok_or(());
                }
            }
            return Ok(());
        }
        self.skip_trivia();
        let offset = self.cursor;
        let item = if self.take("*") {
            "*"
        } else {
            self.identifier().ok_or(())?
        };
        let path = match (prefix, item) {
            ("", "self") => "<crate>".to_string(),
            (_, "self") => prefix.to_string(),
            ("", _) => item.to_string(),
            _ => format!("{prefix}::{item}"),
        };
        if self.take("::") {
            self.tree(&path, references)
        } else {
            if self.take("as") {
                self.renames.push(self.cursor - "as".len());
                self.identifier().ok_or(())?;
            }
            references.push((offset, path));
            Ok(())
        }
    }
}

fn shared_item_references(source: &str) -> Vec<(usize, String)> {
    // Scan the raw source, including codegen strings, rather than only parsed
    // Rust expressions. The parser expands grouped paths after each crate root.
    let roots = shared_crate_identifiers();
    let mut references = Vec::new();
    for root in roots.find_iter(source) {
        let mut parser = SharedPathParser {
            source,
            cursor: root.end(),
            renames: Vec::new(),
        };
        if parser.take("::") {
            parser.skip_trivia();
            // A namespace-only string used by a negative assertion names no
            // item. Do not mistake it for an incomplete use tree.
            if source[parser.cursor..].starts_with('"') {
                continue;
            }
            let start = references.len();
            if parser.tree("", &mut references).is_err() {
                references.truncate(start);
                references.push((root.start(), "<unparsed shared path>".to_string()));
            }
        }
    }

    // Preserve the stricter constructor check for unqualified enum rows, too.
    // Allowing a row's identity as metadata must not allow its attribute_type.
    let constructors =
        regex::Regex::new(r"\bAwsType::([A-Za-z_][A-Za-z0-9_]*|\{[^}\n]+\})\.attribute_type\s*\(")
            .unwrap();
    for capture in constructors.captures_iter(source) {
        references.push((
            capture.get(0).unwrap().start(),
            format!("AwsType::{}.attribute_type", &capture[1]),
        ));
    }
    references
}

fn shared_item_is_allowlisted(name: &str) -> bool {
    if let Some(row) = name.strip_suffix(".attribute_type") {
        return SHARED_TYPES.iter().any(|entry| entry.name == row);
    }
    SHARED_TYPES.iter().any(|entry| entry.name == name)
        || SHARED_UTILITIES.contains(&name)
        || SHARED_MODULE_PREFIXES
            .iter()
            .any(|prefix| name == *prefix || name.starts_with(&format!("{prefix}::")))
}

fn shared_source_failures(source: &str) -> Vec<(usize, String)> {
    let mut failures = Vec::new();

    // Rule 1: every occurrence, including emitted code, must spell a crate path.
    for identifier in shared_crate_identifiers().find_iter(source) {
        if !source[identifier.end()..].starts_with("::") {
            failures.push((
                identifier.start(),
                "shared crate identifier must be immediately followed by '::'".to_string(),
            ));
        }
    }

    // Rules 2 and 3 apply to complete use items, including nested groups. Check
    // syntax before the allowlist so even an otherwise allowed item cannot be
    // renamed, and the shared enum cannot hide its row behind an import.
    for keyword in regex::Regex::new(r"\buse\b").unwrap().find_iter(source) {
        let mut parser = SharedPathParser {
            source,
            cursor: keyword.end(),
            renames: Vec::new(),
        };
        parser.take("::");
        let mut imports = Vec::new();
        if parser.tree("", &mut imports).is_err() || !parser.take(";") {
            continue;
        }
        if !imports.iter().any(|(_, path)| {
            path == SHARED_CRATE_PREFIX.trim_end_matches("::")
                || path.starts_with(SHARED_CRATE_PREFIX)
        }) {
            continue;
        }
        for offset in parser.renames {
            failures.push((offset, "shared use items must not contain 'as'".to_string()));
        }
        for (offset, path) in imports {
            let Some(item) = path.strip_prefix(SHARED_CRATE_PREFIX) else {
                continue;
            };
            if item == "AwsType" || item.starts_with("AwsType::") {
                failures.push((
                    offset,
                    "AwsType imports are forbidden; use the fully qualified row path".to_string(),
                ));
            }
            if item == "*" {
                failures.push((
                    offset,
                    "shared crate glob imports are forbidden".to_string(),
                ));
            }
        }
    }

    failures.extend(
        shared_item_references(source)
            .into_iter()
            .filter(|(_, name)| !shared_item_is_allowlisted(name)),
    );
    failures
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
    let prefix = SHARED_CRATE_PREFIX;
    let enum_prefix = concat!("AwsType", "::");
    for name in ["arn", "iam_role_arn", "kms_key_id", "future_identifier"] {
        let calls = shared_item_references(&format!("{prefix}{name}()"));
        assert_eq!(calls.len(), 1);
        assert!(!shared_item_is_allowlisted(&calls[0].1), "{name}");
    }
    for source in [
        format!("{prefix}{enum_prefix}KmsKeyId.attribute_type()"),
        format!("{enum_prefix}KmsKeyId.attribute_type()"),
        // An allowlisted identity path does not permit its shared constructor.
        format!("{prefix}{enum_prefix}KmsKeyReference.attribute_type()"),
        format!("{prefix}{enum_prefix}{{aws_type:?}}.attribute_type()"),
    ] {
        let references = shared_item_references(&source);
        assert!(
            references
                .iter()
                .any(|(_, item)| !shared_item_is_allowlisted(item)),
            "{source}"
        );
    }
    for entry in SHARED_TYPES {
        let source = if entry.name.starts_with(enum_prefix) {
            format!("{prefix}{}.attribute_type()", entry.name)
        } else {
            format!("{prefix}{}()", entry.name)
        };
        let references = shared_item_references(&source);
        assert!(!references.is_empty(), "{source}");
        assert!(
            references
                .iter()
                .all(|(_, item)| shared_item_is_allowlisted(item)),
            "{source}"
        );
    }
    // #593: Metadata paths are references even without a constructor call.
    assert_eq!(
        shared_item_references(&format!("{prefix}AwsType::IamRoleArn.identity()"))[0].1,
        "AwsType::IamRoleArn"
    );
    assert!(shared_item_references("crate::schemas::catalog::kms_key_id()").is_empty());
}

#[test]
fn source_scan_detects_calls_and_function_references() {
    let prefix = SHARED_CRATE_PREFIX;
    for name in ["arn", "iam_role_arn"] {
        for source in [
            format!("{prefix}{name}()"),
            format!("LazyLock::new({prefix}{name})"),
            format!("let constructor = {prefix}{name};"),
            format!("let emitted = r#\"LazyLock::new({prefix}{name})\"#;"),
        ] {
            let references = shared_item_references(&source);
            assert_eq!(references.len(), 1, "{source}");
            assert_eq!(references[0].1, name, "{source}");
            assert!(!shared_item_is_allowlisted(&references[0].1), "{source}");
            assert!(!shared_source_failures(&source).is_empty(), "{source}");
        }
    }
}

#[test]
fn source_scan_detects_imported_renamed_and_grouped_items() {
    let prefix = SHARED_CRATE_PREFIX;
    for name in ["arn", "iam_role_arn"] {
        for source in [
            format!("use {prefix}{name}; {name}();"),
            format!("use {prefix}{{{name}, vpc_id}}; {name}();"),
            format!("use {prefix}{name} as imported; imported();"),
            format!("use {prefix}{{{name} as imported, vpc_id as safe}}; imported();"),
            format!(
                "use {prefix}{{sdk::{{self as clients, s3::S3Client}}, {{{name} as imported, vpc_id}}}}; imported();"
            ),
            format!("let emitted = \"use {prefix}{{{name} as imported, vpc_id}}; imported();\";"),
        ] {
            let references = shared_item_references(&source);
            assert!(
                references
                    .iter()
                    .any(|(_, item)| item == name && !shared_item_is_allowlisted(item)),
                "missed {name}: {source}; found {references:?}"
            );
            assert!(!shared_source_failures(&source).is_empty(), "{source}");
        }
    }
}

#[test]
fn source_scan_preserves_nested_paths_and_restricts_module_prefixes() {
    let prefix = SHARED_CRATE_PREFIX;
    let source = format!(
        "use ::{prefix}{{sdk::{{self as clients, s3::{{S3Client as Client, types::*}}}}, vpc_id as safe, {{arn as forbidden, iam_role_arn}}}};"
    );
    let references = shared_item_references(&source);
    assert_eq!(
        references
            .iter()
            .map(|(_, item)| item.as_str())
            .collect::<Vec<_>>(),
        [
            "sdk",
            "sdk::s3::S3Client",
            "sdk::s3::types::*",
            "vpc_id",
            "arn",
            "iam_role_arn"
        ]
    );
    assert!(shared_item_is_allowlisted("sdk::s3::S3Client"));
    for name in ["sdk_lookalike::arn", "vpc_id::arn", "AwsType::Unknown", "*"] {
        assert!(!shared_item_is_allowlisted(name), "{name}");
    }
}

#[test]
fn source_scan_handles_use_tree_trivia_and_rejects_incomplete_paths() {
    let prefix = SHARED_CRATE_PREFIX;
    for source in [
        format!(
            "use {prefix}{{\n r#arn /* outer /* nested */ comment */ as forbidden, // another\n vpc_id, }};"
        ),
        format!(r#"let emitted = "use {prefix}{{\n r#arn as forbidden,\n vpc_id,\n }};";"#),
    ] {
        let references = shared_item_references(&source);
        assert_eq!(
            references
                .iter()
                .map(|(_, item)| item.as_str())
                .collect::<Vec<_>>(),
            ["arn", "vpc_id"],
            "{source}"
        );
        assert!(!shared_item_is_allowlisted(&references[0].1));
    }
    assert!(
        shared_item_references(&format!("assert!(!module.contains(\"{prefix}\"));")).is_empty()
    );
    for tree in [
        "{sdk::{S3Client}, arn",
        "{vpc_id, unknown::{",
        "$constructor",
        "self as shared",
        "*",
    ] {
        let source = format!("use {prefix}{tree};");
        assert!(
            shared_item_references(&source)
                .iter()
                .any(|(_, item)| !shared_item_is_allowlisted(item)),
            "{source}"
        );
    }
}

#[test]
fn source_scan_requires_the_shared_crate_identifier_to_be_qualified() {
    let root = "carina_aws_types::".trim_end_matches("::");
    for source in [
        format!("use {root} as shared; shared::arn()"),
        format!("use {root};"),
        format!("pub use {root} as shared;"),
        format!("extern crate {root} as shared;"),
        format!("extern crate {root};"),
        format!("let shared = {root};"),
        format!("use {root} ::vpc_id;"),
    ] {
        for source in [source.clone(), format!("let emitted = r#\"{source}\"#;")] {
            assert!(
                shared_source_failures(&source)
                    .iter()
                    .any(|(_, error)| error.contains("immediately followed by '::'")),
                "{source}"
            );
        }
    }
}

#[test]
fn source_scan_forbids_renamed_shared_imports_at_every_depth() {
    let prefix = "carina_aws_types::";
    for source in [
        format!("use {prefix}AwsType as Shared; Shared::Arn.attribute_type()"),
        format!("use {prefix}vpc_id as safe; safe()"),
        format!("use {prefix}{{vpc_id as safe, subnet_id}};"),
        format!("use {prefix}sdk::{{s3::{{S3Client as Client}}, sts::StsClient}};"),
        format!("use {{{prefix}sdk::{{s3::S3Client as Client}}}};"),
        format!("use {{::{prefix}sdk::s3::S3Client as Client}};"),
        format!("use {prefix}{{self as shared, vpc_id}}; shared::arn()"),
    ] {
        for source in [source.clone(), format!("let emitted = r#\"{source}\"#;")] {
            assert!(
                shared_source_failures(&source)
                    .iter()
                    .any(|(_, error)| error.contains("must not contain 'as'")),
                "{source}"
            );
        }
    }
}

#[test]
fn source_scan_forbids_aws_type_and_shared_crate_glob_imports() {
    let prefix = "carina_aws_types::";
    for (import, reason) in [
        (format!("{prefix}AwsType"), "AwsType imports are forbidden"),
        (
            format!("{prefix}{{AwsType, vpc_id}}"),
            "AwsType imports are forbidden",
        ),
        (
            format!("{prefix}AwsType::{{Arn}}"),
            "AwsType imports are forbidden",
        ),
        (
            format!("{prefix}AwsType::KmsKeyReference"),
            "AwsType imports are forbidden",
        ),
        (
            format!("{{::{prefix}AwsType}}"),
            "AwsType imports are forbidden",
        ),
        (
            format!("{prefix}*"),
            "shared crate glob imports are forbidden",
        ),
        (
            format!("{prefix}{{*}}"),
            "shared crate glob imports are forbidden",
        ),
    ] {
        let source = format!("use {import};");
        for source in [source.clone(), format!("let emitted = r#\"{source}\"#;")] {
            assert!(
                shared_source_failures(&source)
                    .iter()
                    .any(|(_, error)| error.contains(reason)),
                "{source}"
            );
        }
    }
}

#[test]
fn source_scan_accepts_qualified_paths_and_unrenamed_imports() {
    let prefix = SHARED_CRATE_PREFIX;
    for source in [
        format!("{prefix}vpc_id()"),
        format!("LazyLock::new({prefix}vpc_id)"),
        format!("let constructor = {prefix}vpc_id as fn() -> AttributeType;"),
        format!("use {prefix}vpc_id; vpc_id()"),
        format!("use {prefix}{{vpc_id, subnet_id}};"),
        format!("use ::{prefix}sdk::{{s3::{{S3Client, types::*}}, sts::StsClient}};"),
        format!("pub use {{{prefix}vpc_id, std::vec::Vec}};"),
        format!("{prefix}AwsType::KmsKeyReference.identity()"),
        format!("let rows = {prefix}AwsType::ALL;"),
        format!("use std::vec::Vec as Values; {prefix}vpc_id()"),
    ] {
        for source in [source.clone(), format!("let emitted = r#\"{source}\"#;")] {
            let failures = shared_source_failures(&source);
            assert!(failures.is_empty(), "{source}: {failures:?}");
        }
    }
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
        for (offset, reason) in shared_source_failures(&source) {
            failures.push(format!(
                "{}:{}: {reason}",
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
        "Shared items must obey the source rules and explicit allowlist:\n{}",
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
