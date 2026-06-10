//! CloudFormation Schema to Carina Schema Code Generator
//!
//! This tool generates Rust schema code for carina-provider-awscc
//! from AWS CloudFormation resource type schemas.
//!
//! Usage:
//!   # Generate from stdin (pipe from aws cli)
//!   aws-vault exec <profile> -- aws cloudformation describe-type \
//!     --type RESOURCE --type-name AWS::EC2::VPC --query 'Schema' --output text | \
//!     carina-codegen --type-name AWS::EC2::VPC
//!
//!   # Generate from file
//!   carina-codegen --file schema.json --type-name AWS::EC2::VPC

use anyhow::{Context, Result};
use clap::Parser;
use heck::{ToPascalCase, ToSnakeCase};
use regex::Regex;
use serde::Deserialize;
use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::io::{self, Read};
use std::sync::LazyLock;

// carina#3340: in-progress recursion guard for cyclic CFN $ref
// definitions (WAFv2 `WebACL.Statement` → `AndStatement` → `List<Statement>`).
// When `generate_struct_type` is asked to expand a definition whose name
// is already on the stack, it emits `AttributeType::Ref(name)` and
// records the def's body into [`STRUCT_DEFS_OUT`] for later emission
// into `ResourceSchema.defs`. The set is per-resource — cleared at the
// top of each resource generation pass.
/// `(pattern, min, max)` triples accumulated by the recursion guard.
type PatternWithLength = (String, Option<u64>, Option<u64>);
/// `(min, max, is_float)` tuple keyed by prop name for int / float
/// range validators accumulated by the recursion guard.
type RangeValidator = (Option<i64>, Option<i64>, bool);

struct RustStrLit<'a>(&'a str);

impl std::fmt::Display for RustStrLit<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("\"")?;
        for ch in self.0.escape_default() {
            write!(f, "{ch}")?;
        }
        f.write_str("\"")
    }
}

fn rust_lit(s: &str) -> RustStrLit<'_> {
    RustStrLit(s)
}

// `HashSet::new()` is not `const fn`, so clippy's
// `missing_const_for_thread_local` suggestion is rejected for the
// HashSet-typed cells by rustc. The BTree-typed cells can use `const`
// init, but mixing styles inside one `thread_local!` block makes the
// allow attribute awkward; suppressing the lint for the whole block
// keeps the source consistent. (carina#3340.)
thread_local! {
    #[allow(clippy::missing_const_for_thread_local)]
    static IN_PROGRESS_DEFS: RefCell<HashSet<String>> = RefCell::new(HashSet::new());
    /// Collected `(def_name, AttributeType-rendered Rust code)` for the
    /// resource currently being generated. Populated by the recursion
    /// guard in `cfn_type_to_carina_type_with_enum`; consumed by the
    /// resource emitter to populate `ResourceSchema.defs`.
    #[allow(clippy::missing_const_for_thread_local)]
    static EMITTED_DEFS: RefCell<BTreeMap<String, String>> = RefCell::new(BTreeMap::new());
    /// Patterns discovered during recursive expansion of CFN `$ref`
    /// struct definitions. The original pre-scan walks only the
    /// top-level properties + definitions one hop deep; once the
    /// recursion guard lets `generate_struct_type` walk N levels deep
    /// into a cyclic struct, the patterns inside those nested levels
    /// must still be emitted as `fn` definitions or the generated
    /// file will reference undefined symbols. Populated by
    /// `cfn_type_to_carina_type_with_enum` at every pattern emission
    /// site; the resource emitter drains this into the same
    /// `pattern_with_lengths` set used by the existing pre-scan
    /// emitter (carina#3340).
    #[allow(clippy::missing_const_for_thread_local)]
    static EMITTED_PATTERNS_WITH_LENGTHS: RefCell<BTreeSet<PatternWithLength>> =
        RefCell::new(BTreeSet::new());
    #[allow(clippy::missing_const_for_thread_local)]
    static EMITTED_STANDALONE_PATTERNS: RefCell<BTreeSet<String>> =
        RefCell::new(BTreeSet::new());
    /// `(prop_name -> (min, max, is_float))` for integer/number range
    /// validators referenced by the recursive cycle expansion. Same
    /// rationale as `EMITTED_PATTERNS_WITH_LENGTHS`: the existing
    /// `ranged_ints` / `ranged_floats` pre-scan walks only one hop
    /// into definitions; nested cyclic fields can reference
    /// `validate_<snake>_range` fns the pre-scan never collected.
    #[allow(clippy::missing_const_for_thread_local)]
    static EMITTED_RANGE_VALIDATORS: RefCell<BTreeMap<String, RangeValidator>> =
        RefCell::new(BTreeMap::new());
    /// `prop_name -> integer enum values`, same rationale as
    /// `EMITTED_RANGE_VALIDATORS` (carina#3340).
    #[allow(clippy::missing_const_for_thread_local)]
    static EMITTED_INT_ENUMS: RefCell<BTreeMap<String, Vec<i64>>> =
        RefCell::new(BTreeMap::new());
}

/// Reset the per-resource recursion guard state. Called at the start
/// of every resource generation so a cycle detected for resource A
/// does not leak into resource B.
fn reset_recursion_guard() {
    IN_PROGRESS_DEFS.with(|s| s.borrow_mut().clear());
    EMITTED_DEFS.with(|s| s.borrow_mut().clear());
    EMITTED_PATTERNS_WITH_LENGTHS.with(|s| s.borrow_mut().clear());
    EMITTED_STANDALONE_PATTERNS.with(|s| s.borrow_mut().clear());
    EMITTED_RANGE_VALIDATORS.with(|s| s.borrow_mut().clear());
    EMITTED_INT_ENUMS.with(|s| s.borrow_mut().clear());
}

/// Take the defs collected during the current resource pass and
/// return them as a sorted `BTreeMap`. Empty for non-cyclic resources.
fn take_emitted_defs() -> BTreeMap<String, String> {
    EMITTED_DEFS.with(|s| std::mem::take(&mut *s.borrow_mut()))
}

/// Record a pattern+length validator emission so the function-emitter
/// can produce its matching `fn` definition. Called from every
/// emission site in `cfn_type_to_carina_type_with_enum` that
/// references a `validate_string_pattern_<hash>_len_<min>_<max>`
/// symbol. The pre-scan over `definitions` already collects most
/// patterns; this set is the cycle-recursion safety net that catches
/// patterns nested deeper than the pre-scan walks (carina#3340).
fn record_pattern_with_length(pattern: &str, min: Option<u64>, max: Option<u64>) {
    EMITTED_PATTERNS_WITH_LENGTHS.with(|s| {
        s.borrow_mut().insert((pattern.to_string(), min, max));
    });
}

fn record_standalone_pattern(pattern: &str) {
    EMITTED_STANDALONE_PATTERNS.with(|s| {
        s.borrow_mut().insert(pattern.to_string());
    });
}

fn take_emitted_patterns_with_lengths() -> BTreeSet<(String, Option<u64>, Option<u64>)> {
    EMITTED_PATTERNS_WITH_LENGTHS.with(|s| std::mem::take(&mut *s.borrow_mut()))
}

fn take_emitted_standalone_patterns() -> BTreeSet<String> {
    EMITTED_STANDALONE_PATTERNS.with(|s| std::mem::take(&mut *s.borrow_mut()))
}

fn record_range_validator(prop_name: &str, min: Option<i64>, max: Option<i64>, is_float: bool) {
    EMITTED_RANGE_VALIDATORS.with(|s| {
        s.borrow_mut()
            .insert(prop_name.to_string(), (min, max, is_float));
    });
}

fn take_emitted_range_validators() -> BTreeMap<String, (Option<i64>, Option<i64>, bool)> {
    EMITTED_RANGE_VALIDATORS.with(|s| std::mem::take(&mut *s.borrow_mut()))
}

fn record_int_enum(prop_name: &str, values: &[i64]) {
    EMITTED_INT_ENUMS.with(|s| {
        s.borrow_mut()
            .insert(prop_name.to_string(), values.to_vec());
    });
}

fn take_emitted_int_enums() -> BTreeMap<String, Vec<i64>> {
    EMITTED_INT_ENUMS.with(|s| std::mem::take(&mut *s.borrow_mut()))
}

/// Exit status used when a schema is deliberately skipped (NON_PROVISIONABLE).
/// The generate-schemas.sh / generate-docs.sh wrappers branch on this to
/// distinguish intentional skips from real errors.
const EXIT_SKIPPED: i32 = 2;

/// Unified type override for resource-scoped property overrides.
/// Allows overriding string type, enum values, integer range, or integer enum
/// for a specific (resource_type, property_name) pair.
#[derive(Debug, Clone, PartialEq)]
enum TypeOverride {
    /// Override to a specific string type (e.g., "super::super::iam::role::arn()")
    StringType(&'static str),
    /// Override to an enum with specific values
    Enum(Vec<&'static str>),
    /// Like [`TypeOverride::Enum`], but when the property is an array
    /// the generated list is `unordered_list` regardless of the CFN
    /// `insertionOrder` (which defaults to ordered when unspecified).
    /// For order-insensitive set-valued fields whose CloudFormation
    /// schema omits `insertionOrder: false` — e.g. CloudFront
    /// `AllowedMethods`/`CachedMethods` (carina#3093). Element-type
    /// handling is identical to `Enum`; only the list ordering differs.
    EnumUnordered(Vec<&'static str>),
    /// Override to an integer range (min, max)
    IntRange(i64, i64),
    /// Override to an integer enum with specific allowed values
    IntEnum(Vec<i64>),
    /// Override to_dsl on a Custom type (Rust code for the closure)
    ToDsl(&'static str),
}

/// Information about a detected enum type
#[derive(Debug, Clone)]
struct EnumInfo {
    /// Property name in PascalCase (e.g., "InstanceTenancy")
    type_name: String,
    /// Valid enum values (e.g., ["default", "dedicated", "host"])
    values: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ArnEmitChoice {
    PerKind(&'static ArnValidation),
    ServicePrefix(&'static str),
    Generic,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ArnValidation {
    service: &'static str,
    resource: &'static str,
    regex: &'static str,
    validator: ArnValidator,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ArnValidator {
    Iam {
        prefix: &'static str,
        label: &'static str,
    },
    Service {
        service: &'static str,
        prefix: Option<&'static str>,
        label: &'static str,
    },
}

static ARN_VALIDATIONS: &[ArnValidation] = &[
    ArnValidation {
        service: "iam",
        resource: "Role",
        regex: "^arn:(aws|aws-cn|aws-us-gov):iam::[^:]*:role/.+$",
        validator: ArnValidator::Iam {
            prefix: "role/",
            label: "IAM Role",
        },
    },
    ArnValidation {
        service: "iam",
        resource: "Policy",
        regex: "^arn:(aws|aws-cn|aws-us-gov):iam::[^:]*:policy/.+$",
        validator: ArnValidator::Iam {
            prefix: "policy/",
            label: "IAM Policy",
        },
    },
    ArnValidation {
        service: "iam",
        resource: "OidcProvider",
        regex: "^arn:(aws|aws-cn|aws-us-gov):iam::[^:]*:oidc-provider/.+$",
        validator: ArnValidator::Iam {
            prefix: "oidc-provider/",
            label: "IAM OIDC Provider",
        },
    },
    ArnValidation {
        service: "kms",
        resource: "Key",
        regex: "^arn:(aws|aws-cn|aws-us-gov):kms:[^:]*:[^:]*:key/.+$",
        validator: ArnValidator::Service {
            service: "kms",
            prefix: Some("key/"),
            label: "KMS Key",
        },
    },
    ArnValidation {
        service: "s3",
        resource: "Bucket",
        regex: "^arn:(aws|aws-cn|aws-us-gov):s3:::.+$",
        validator: ArnValidator::Service {
            service: "s3",
            prefix: None,
            label: "s3",
        },
    },
    ArnValidation {
        service: "cloudfront",
        resource: "Distribution",
        regex: "^arn:(aws|aws-cn|aws-us-gov):cloudfront::[^:]*:distribution/.+$",
        validator: ArnValidator::Service {
            service: "cloudfront",
            prefix: Some("distribution/"),
            label: "cloudfront",
        },
    },
    ArnValidation {
        service: "ecs",
        resource: "Cluster",
        regex: "^arn:(aws|aws-cn|aws-us-gov):ecs:[^:]*:[^:]*:cluster/.+$",
        validator: ArnValidator::Service {
            service: "ecs",
            prefix: Some("cluster/"),
            label: "ecs",
        },
    },
    ArnValidation {
        service: "dynamodb",
        resource: "Table",
        regex: "^arn:(aws|aws-cn|aws-us-gov):dynamodb:[^:]*:[^:]*:table/.+$",
        validator: ArnValidator::Service {
            service: "dynamodb",
            prefix: Some("table/"),
            label: "dynamodb",
        },
    },
    ArnValidation {
        service: "logs",
        resource: "LogGroup",
        regex: "^arn:(aws|aws-cn|aws-us-gov):logs:[^:]*:[^:]*:log-group:.+$",
        validator: ArnValidator::Service {
            service: "logs",
            prefix: Some("log-group:"),
            label: "logs",
        },
    },
];

static KNOWN_SERVICES: &[&str] = &[
    "cloudfront",
    "dynamodb",
    "ec2",
    "ecs",
    "iam",
    "kms",
    "logs",
    "organizations",
    "s3",
    "wafv2",
];

fn arn_emit_choice(service: &str, resource: &str) -> ArnEmitChoice {
    if let Some(entry) = ARN_VALIDATIONS
        .iter()
        .find(|v| v.service == service && v.resource == resource)
    {
        ArnEmitChoice::PerKind(entry)
    } else if let Some(&known) = KNOWN_SERVICES.iter().find(|&&known| known == service) {
        ArnEmitChoice::ServicePrefix(known)
    } else {
        ArnEmitChoice::Generic
    }
}

fn resource_identity_parts(name: &str) -> Option<(&str, &str)> {
    name.split_once('.')
}

fn arn_validator_for(validator: ArnValidator) -> String {
    match validator {
        ArnValidator::Iam { prefix, label } => format!(
            r#"|value| {{
            if let Value::Concrete(ConcreteValue::String(s)) = value {{
                carina_aws_types::validate_iam_arn(s, {prefix:?})
                    .map_err(|reason| format!("Invalid {label} ARN '{{}}': {{}}", s, reason))
            }} else {{
                Err("Expected string".to_string())
            }}
        }}"#
        ),
        ArnValidator::Service {
            service,
            prefix,
            label,
        } => {
            let prefix_expr = match prefix {
                Some(prefix) => format!("Some({prefix:?})"),
                None => "None".to_string(),
            };
            format!(
                r#"|value| {{
            if let Value::Concrete(ConcreteValue::String(s)) = value {{
                carina_aws_types::validate_service_arn(s, {service:?}, {prefix_expr})
                    .map_err(|reason| format!("Invalid {label} ARN '{{}}': {{}}", s, reason))
            }} else {{
                Err("Expected string".to_string())
            }}
        }}"#
            )
        }
    }
}

fn arn_validator_expression(choice: ArnEmitChoice) -> String {
    match choice {
        ArnEmitChoice::PerKind(entry) => arn_validator_for(entry.validator),
        ArnEmitChoice::ServicePrefix(service) => format!(
            r#"|value| {{
            if let Value::Concrete(ConcreteValue::String(s)) = value {{
                carina_aws_types::validate_service_arn(s, {service:?}, None)
                    .map_err(|reason| format!("Invalid {service} ARN '{{}}': {{}}", s, reason))
            }} else {{
                Err("Expected string".to_string())
            }}
        }}"#
        ),
        ArnEmitChoice::Generic => unreachable!("generic ARN helper has no validator expression"),
    }
}

fn emit_arn_helper(service: &str, resource: &str, choice: ArnEmitChoice) -> String {
    if matches!(choice, ArnEmitChoice::Generic) {
        return "pub fn arn() -> AttributeType {\n    carina_aws_types::arn()\n}\n\n".to_string();
    }

    let regex_expr = match choice {
        ArnEmitChoice::PerKind(entry) => format!("Some({:?}.to_string())", entry.regex),
        ArnEmitChoice::ServicePrefix(service) => {
            format!(
                "Some(\"^arn:(aws|aws-cn|aws-us-gov):{}:.*$\".to_string())",
                service
            )
        }
        ArnEmitChoice::Generic => unreachable!("handled above"),
    };
    let validator_expr = arn_validator_expression(choice);
    format!(
        r#"pub fn arn() -> AttributeType {{
    AttributeType::refined_string_with_validator(
        Some(carina_aws_types::provider_type("{service}", "{resource}", "Arn")),
        {regex_expr},
        None,
        legacy_validator({validator_expr}),
        None,
    )
}}

"#
    )
}

#[derive(Parser, Debug)]
#[command(name = "carina-codegen")]
#[command(about = "Generate Carina schema code from CloudFormation schemas")]
struct Args {
    /// CloudFormation type name (e.g., AWS::EC2::VPC)
    #[arg(long)]
    type_name: String,

    /// Input file (reads from stdin if not specified)
    #[arg(long)]
    file: Option<String>,

    /// Output file (writes to stdout if not specified)
    #[arg(long, short)]
    output: Option<String>,

    /// Print module name for the given type and exit
    /// e.g., AWS::EC2::SecurityGroupEgress -> security_group_egress
    #[arg(long)]
    print_module_name: bool,

    /// Print full resource name (service_resource) for the given type and exit
    /// e.g., AWS::EC2::SecurityGroupEgress -> ec2_security_group_egress
    #[arg(long)]
    print_full_resource_name: bool,

    /// Print DSL resource name (service.resource) for the given type and exit
    /// e.g., AWS::EC2::SecurityGroupEgress -> ec2.security_group_egress
    #[arg(long)]
    print_dsl_resource_name: bool,

    /// Output format: rust (default) or markdown (for documentation)
    #[arg(long, default_value = "rust")]
    format: String,
}

/// CloudFormation Resource Schema
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
struct CfnSchema {
    type_name: String,
    description: Option<String>,
    properties: BTreeMap<String, CfnProperty>,
    #[serde(default)]
    required: Vec<String>,
    #[serde(default)]
    read_only_properties: Vec<String>,
    #[serde(default)]
    create_only_properties: Vec<String>,
    #[serde(default)]
    write_only_properties: Vec<String>,
    primary_identifier: Option<Vec<String>>,
    definitions: Option<BTreeMap<String, CfnDefinition>>,
    tagging: Option<CfnTagging>,
    /// Top-level oneOf variants (mutually exclusive required field groups)
    #[serde(default, rename = "oneOf")]
    one_of: Vec<CfnProperty>,
    /// Top-level anyOf variants
    #[serde(default, rename = "anyOf")]
    any_of: Vec<CfnProperty>,
    /// Cloud Control handlers (create/read/update/delete/list).
    /// Absent or empty means the type is NON_PROVISIONABLE and cannot be
    /// managed via Cloud Control API.
    #[serde(default)]
    handlers: BTreeMap<String, serde_json::Value>,
}

impl CfnSchema {
    /// Returns true if the schema declares any Cloud Control handlers.
    /// NON_PROVISIONABLE resource types have no handlers and cannot be
    /// operated via Cloud Control API, so the codegen skips them.
    fn has_cloud_control_handlers(&self) -> bool {
        !self.handlers.is_empty()
    }
}

/// CloudFormation Tagging metadata
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
struct CfnTagging {
    #[serde(default)]
    taggable: bool,
}

/// Type can be a string or an array of strings in JSON Schema
#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
enum TypeValue {
    Single(String),
    Multiple(Vec<String>),
}

impl TypeValue {
    fn as_str(&self) -> Option<&str> {
        match self {
            TypeValue::Single(s) => Some(s),
            TypeValue::Multiple(v) => v.first().map(|s| s.as_str()),
        }
    }
}

/// Enum value can be a string or an integer in JSON Schema
#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
enum EnumValue {
    Str(String),
    Int(i64),
}

impl EnumValue {
    fn to_string_value(&self) -> String {
        match self {
            EnumValue::Str(s) => s.clone(),
            EnumValue::Int(i) => i.to_string(),
        }
    }
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
struct CfnProperty {
    #[serde(rename = "type")]
    prop_type: Option<TypeValue>,
    description: Option<String>,
    #[serde(rename = "enum")]
    enum_values: Option<Vec<EnumValue>>,
    items: Option<Box<CfnProperty>>,
    #[serde(rename = "$ref")]
    ref_path: Option<String>,
    #[serde(default)]
    insertion_order: Option<bool>,
    /// Inline object properties (for nested objects)
    properties: Option<BTreeMap<String, CfnProperty>>,
    /// Required fields for inline objects
    #[serde(default)]
    required: Vec<String>,
    /// Minimum value constraint (for integer/number types)
    #[serde(default)]
    minimum: Option<i64>,
    /// Maximum value constraint (for integer/number types)
    #[serde(default)]
    maximum: Option<i64>,
    /// Whether additional properties are allowed (for object types)
    #[serde(default)]
    additional_properties: Option<bool>,
    /// Const value (single fixed value for this property)
    #[serde(rename = "const")]
    const_value: Option<serde_json::Value>,
    /// Default value for this property
    #[serde(rename = "default")]
    default_value: Option<serde_json::Value>,
    /// Regex pattern constraint (for string types)
    #[serde(default)]
    pattern: Option<String>,
    /// Minimum number of items (for array types)
    #[serde(default)]
    min_items: Option<i64>,
    /// Maximum number of items (for array types)
    #[serde(default)]
    max_items: Option<i64>,
    /// Minimum length constraint (for string types)
    #[serde(default, rename = "minLength")]
    min_length: Option<u64>,
    /// Maximum length constraint (for string types)
    #[serde(default, rename = "maxLength")]
    max_length: Option<u64>,
    /// Format constraint (e.g., "int64", "uri", "double")
    #[serde(default)]
    format: Option<String>,
    /// Property-level oneOf variants (union types)
    #[serde(default, rename = "oneOf")]
    one_of: Vec<CfnProperty>,
    #[serde(default)]
    #[serde(rename = "anyOf")]
    any_of: Vec<CfnProperty>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
struct CfnDefinition {
    #[serde(rename = "type")]
    def_type: Option<String>,
    properties: Option<BTreeMap<String, CfnProperty>>,
    #[serde(default)]
    required: Vec<String>,
    /// oneOf variants (union types)
    #[serde(default, rename = "oneOf")]
    one_of: Vec<CfnProperty>,
    /// Array item schema (for array-typed definitions)
    items: Option<Box<CfnProperty>>,
    /// Enum values (for enum-only definitions)
    #[serde(rename = "enum")]
    enum_values: Option<Vec<EnumValue>>,
    /// Regex pattern constraint (for string-typed definitions)
    #[serde(default)]
    pattern: Option<String>,
    /// Minimum string length constraint (for string-typed definitions)
    #[serde(default, rename = "minLength")]
    min_length: Option<u64>,
    /// Maximum string length constraint (for string-typed definitions)
    #[serde(default, rename = "maxLength")]
    max_length: Option<u64>,
    /// Minimum value constraint (for integer/number-typed definitions).
    /// CloudFormation factors scalar fields like `RulePriority`/`RateLimit`
    /// into shared `$ref` definitions that carry their own range; this lets
    /// a scalar `$ref` reproduce the same ranged int/float a direct property
    /// would (awscc#291).
    #[serde(default)]
    minimum: Option<i64>,
    /// Maximum value constraint (for integer/number-typed definitions).
    #[serde(default)]
    maximum: Option<i64>,
    /// Format constraint (e.g., "int64", "double") for scalar definitions.
    #[serde(default)]
    format: Option<String>,
}

/// Compute module name from CloudFormation type name
/// e.g., "AWS::EC2::SecurityGroupEgress" -> "security_group_egress"
fn module_name_from_type(type_name: &str) -> Result<String> {
    let parts: Vec<&str> = type_name.split("::").collect();
    if parts.len() != 3 {
        anyhow::bail!("Invalid type name format: {}", type_name);
    }
    Ok(parts[2].to_snake_case())
}

/// Compute full resource name (service_resource) from CloudFormation type name
/// e.g., "AWS::EC2::SecurityGroupEgress" -> "ec2_security_group_egress"
/// Used for Rust module names, file names, and function names.
fn full_resource_name_from_type(type_name: &str) -> Result<String> {
    let parts: Vec<&str> = type_name.split("::").collect();
    if parts.len() != 3 {
        anyhow::bail!("Invalid type name format: {}", type_name);
    }
    let service = parts[1].to_lowercase();
    let resource = parts[2].to_snake_case();
    Ok(format!("{}_{}", service, resource))
}

/// Compute DSL resource name (service.resource) from CloudFormation type name
/// e.g., "AWS::EC2::SecurityGroupEgress" -> "ec2.SecurityGroupEgress"
/// and "AWS::EC2::VPC" -> "ec2.Vpc" (acronyms normalised via snake -> pascal).
/// The service segment is lowercase (a namespace); the resource segment is
/// PascalCase with acronyms treated as single words, matching carina-core's
/// `snake_to_pascal` round-trip (design D2).
fn dsl_resource_name_from_type(type_name: &str) -> Result<String> {
    let parts: Vec<&str> = type_name.split("::").collect();
    if parts.len() != 3 {
        anyhow::bail!("Invalid type name format: {}", type_name);
    }
    let service = parts[1].to_lowercase();
    // Route through snake_case first so CloudFormation-style acronyms
    // ("VPC", "IPAM", "EIP") become single Pascal tokens ("Vpc", "Ipam",
    // "Eip") rather than shouty residue.
    let resource = parts[2].to_snake_case().to_pascal_case();
    Ok(format!("{}.{}", service, resource))
}

/// Convert a CloudFormation-side enum value to its DSL (snake_case) form.
///
/// Per naming-conventions design D7:
/// - SHOUTY_SNAKE (`GROUP`, `AWS_ACCOUNT`) → lowercase (`group`, `aws_account`)
/// - PascalCase (`Enabled`, `VersioningStatus`) → snake_case (`enabled`,
///   `versioning_status`)
/// - Already kebab/snake (`ap-northeast-1`, `ipsec.1`) → `-` to `_` if present
///   (`ap_northeast_1`), leave `.`-containing values verbatim so they round-trip
/// - Numeric values pass through unchanged.
fn dsl_enum_value(value: &str) -> String {
    if value.is_empty() {
        return String::new();
    }
    // Passthrough for already-numeric or dotted values (e.g. "ipsec.1").
    if value.chars().all(|c| c.is_ascii_digit() || c == '.') {
        return value.to_string();
    }
    // SHOUTY_SNAKE: uppercase ASCII letters + underscores + optional digits.
    if value
        .chars()
        .all(|c| c.is_ascii_uppercase() || c == '_' || c.is_ascii_digit())
        && value.chars().any(|c| c.is_ascii_uppercase())
    {
        return value.to_ascii_lowercase();
    }
    // Already snake / kebab (no uppercase): normalize hyphens, colons,
    // and dotted-numeric splits (e.g. `cloud-watch-logs` →
    // `cloud_watch_logs`, `ipsec.1` → `ipsec_1`, `aws:kms` → `aws_kms`).
    // The strict-DSL validator (carina-rs/carina#2980) gates acceptance
    // on the DSL spelling, so any non-identifier separator must collapse
    // to `_` to keep the value reachable as a bare DSL identifier.
    if !value.chars().any(|c| c.is_ascii_uppercase()) {
        return value.replace(['-', '.', ':'], "_");
    }
    // Special case: acronym + lowercase + digits (e.g. "IPv4", "IPv6").
    // Heck's snake_case splits these as "i_pv4" which loses the acronym
    // structure. Treat them as a single all-lowercase word so the DSL
    // spelling matches the conventional reading "ipv4".
    if let Some(idx) = value.chars().position(|c| c.is_ascii_lowercase())
        && idx >= 1
        && value[..idx].chars().all(|c| c.is_ascii_uppercase())
        && value[idx..]
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit())
    {
        return value.to_ascii_lowercase();
    }
    // PascalCase (or anything else mixed): route through heck's ToSnakeCase.
    value.to_snake_case()
}

/// Build the `dsl_aliases: ...` Rust source code for a `StringEnum`'s
/// alias table, following naming-conventions design D7.
///
/// Per awscc#199 / carina#2831 / D3 / D7, every enum value gets a
/// snake_case DSL spelling and the validator must accept both the
/// canonical (API) spelling and the DSL one. The alias table is data
/// — `Vec<(api, dsl)>` — so it can serialize through the WASM
/// boundary; the earlier `Option<fn(&str) -> String>` closure could
/// not, which is why provider plugins silently lost their alias map
/// and validation rejected legitimate snake_case spellings end-to-end.
///
/// `prop_aliases` carries hand-curated overrides from
/// [`known_enum_aliases`] (e.g. `IpProtocol`'s `"-1"` ↔ `"all"`); those
/// win over the mechanical D7 transform.
///
/// Returns `"vec![]"` when every value's DSL spelling already equals
/// the canonical spelling (i.e. all values are already snake_case and
/// there are no manual aliases) — the empty vector signals "no
/// rewrites" without dragging unused identifier text into the
/// generated schema.
fn dsl_aliases_code(
    values: &[String],
    prop_aliases: Option<&Vec<(&'static str, &'static str)>>,
) -> String {
    // Build (canonical, dsl_form) pairs for **every** value, including
    // identity rows where the DSL spelling equals the canonical. The
    // exhaustive table is what makes the carina-core strict-DSL
    // validator (see carina-rs/carina#2980) treat the whole enum
    // uniformly — once any pair in `dsl_aliases` actually rewrites,
    // strict mode kicks in for every variant, and identity rows are
    // needed so the legitimate same-spelling values still validate.
    //
    // Manual aliases from known_enum_aliases win over the D7
    // mechanical transform.
    let mut entries: Vec<(String, String)> = Vec::new();
    for value in values {
        let dsl_form = if let Some(aliases) = prop_aliases {
            if let Some((_, alias)) = aliases.iter().find(|(c, _)| c == value) {
                alias.to_string()
            } else {
                dsl_enum_value(value)
            }
        } else {
            dsl_enum_value(value)
        };
        entries.push((value.clone(), dsl_form));
    }

    // Also include alias entries whose canonical does not appear in `values`
    // (defensive — known_enum_aliases is small and historically all entries
    // do appear in `values`, but emit them anyway so the table stays
    // exhaustive against the hand-curated table).
    if let Some(aliases) = prop_aliases {
        for (canonical, alias) in aliases {
            let already = entries.iter().any(|(c, _)| c == canonical);
            if !already && canonical != alias {
                entries.push(((*canonical).to_string(), (*alias).to_string()));
            }
        }
    }

    if entries.is_empty() {
        return "vec![]".to_string();
    }

    let pairs: Vec<String> = entries
        .iter()
        .map(|(canonical, alias)| {
            format!(
                "({}.to_string(), {}.to_string())",
                rust_lit(canonical),
                rust_lit(alias)
            )
        })
        .collect();
    format!("vec![{}]", pairs.join(", "))
}

fn main() -> Result<()> {
    let args = Args::parse();

    if args.print_module_name {
        println!("{}", module_name_from_type(&args.type_name)?);
        return Ok(());
    }

    if args.print_full_resource_name {
        println!("{}", full_resource_name_from_type(&args.type_name)?);
        return Ok(());
    }

    if args.print_dsl_resource_name {
        println!("{}", dsl_resource_name_from_type(&args.type_name)?);
        return Ok(());
    }

    // Read schema JSON
    let schema_json = if let Some(file_path) = &args.file {
        std::fs::read_to_string(file_path)
            .with_context(|| format!("Failed to read file: {}", file_path))?
    } else {
        let mut buffer = String::new();
        io::stdin()
            .read_to_string(&mut buffer)
            .context("Failed to read from stdin")?;
        buffer
    };

    // Parse schema
    let schema: CfnSchema =
        serde_json::from_str(&schema_json).context("Failed to parse CloudFormation schema")?;

    if !schema.has_cloud_control_handlers() {
        eprintln!(
            "Skipping {}: NON_PROVISIONABLE (no Cloud Control handlers).\n\
             Use carina-provider-aws if you need to manage this resource via the AWS SDK directly.",
            args.type_name
        );
        std::process::exit(EXIT_SKIPPED);
    }

    // Generate output based on format
    let output = match args.format.as_str() {
        "markdown" | "md" => generate_markdown(&schema, &args.type_name)?,
        "rust" => generate_schema_code(&schema, &args.type_name)?,
        other => anyhow::bail!("Unknown format: {}. Use 'rust' or 'markdown'.", other),
    };

    // Output
    if let Some(output_path) = &args.output {
        std::fs::write(output_path, &output)
            .with_context(|| format!("Failed to write to: {}", output_path))?;
        eprintln!("Generated: {}", output_path);
    } else {
        println!("{}", output);
    }

    Ok(())
}

/// Information about a resolved struct definition for markdown docs
struct StructDefInfo {
    /// Definition name (e.g., "Ingress")
    def_name: String,
    /// Structural path from the resource root to this definition.
    struct_path: Vec<String>,
    /// Properties of the definition
    properties: BTreeMap<String, CfnProperty>,
    /// Required fields
    required: Vec<String>,
}

fn extend_struct_path(struct_path: &[String], struct_name: &str) -> Vec<String> {
    let segment = struct_name.to_pascal_case();
    let existing: Vec<usize> = struct_path
        .iter()
        .enumerate()
        .filter_map(|(index, existing)| (existing == &segment).then_some(index))
        .collect();
    if existing.len() >= 2 {
        let index = *existing.last().expect("existing is non-empty");
        return struct_path[..=index].to_vec();
    }

    let mut extended = struct_path.to_vec();
    extended.push(segment);
    extended
}

fn namespace_with_struct_path(namespace: &str, struct_path: &[String]) -> String {
    if struct_path.is_empty() {
        namespace.to_string()
    } else {
        format!("{}.{}", namespace, struct_path.join("."))
    }
}

fn is_deprecated_property(prop: &CfnProperty) -> bool {
    prop.description
        .as_deref()
        .is_some_and(|description| description.starts_with("(Deprecated.)"))
}

fn is_struct_property(prop: &CfnProperty, schema: &CfnSchema) -> bool {
    if prop
        .properties
        .as_ref()
        .is_some_and(|properties| !properties.is_empty())
    {
        return true;
    }
    prop.ref_path.as_ref().is_some_and(|ref_path| {
        matches!(
            resolve_ref_classified(schema, ref_path),
            Some(ResolvedDef::Struct { .. } | ResolvedDef::OneOf { .. })
        )
    })
}

fn is_list_of_struct_property(prop: &CfnProperty, schema: &CfnSchema) -> bool {
    if prop.prop_type.as_ref().and_then(|t| t.as_str()) == Some("array") {
        return prop
            .items
            .as_ref()
            .is_some_and(|items| is_struct_property(items, schema));
    }
    prop.ref_path.as_ref().is_some_and(|ref_path| {
        matches!(
            resolve_ref_classified(schema, ref_path),
            Some(ResolvedDef::Array { items }) if is_struct_property(items, schema)
        )
    })
}

fn uses_core_schema_types_path(s: &str) -> bool {
    s.contains("types::ipv4_")
        || s.contains("types::ipv6_")
        || s.contains("types::cidr()")
        || s.contains("types::email()")
}

/// Display string for List element types based on items property type and property name.
/// The `prop_name` is used for name-based type inference (e.g., SubnetIds -> `List<SubnetId>`).
fn list_element_type_display(items: &CfnProperty, prop_name: &str, resource_type: &str) -> String {
    match items.prop_type.as_ref().and_then(|t| t.as_str()) {
        Some("string") => {
            let element_type = infer_string_type_display(prop_name, resource_type);
            format!("`List<{}>`", element_type)
        }
        Some("integer") => "`List<Int>`".to_string(),
        Some("number") => "`List<Float>`".to_string(),
        Some("boolean") => "`List<Bool>`".to_string(),
        _ => "`List<String>`".to_string(),
    }
}

/// Infer a display type name for a string property based on its name.
/// Used by both scalar `type_display_string()` and `list_element_type_display()`.
/// Handles both singular (e.g., "SubnetId") and plural (e.g., "SubnetIds") property names.
fn infer_string_type_display(prop_name: &str, resource_type: &str) -> String {
    // Check resource-specific overrides first (StringType only for display)
    if let Some(TypeOverride::StringType(override_type)) =
        resource_type_overrides().get(&(resource_type, prop_name))
    {
        return override_type_to_display_name(override_type).to_string();
    }
    // Check known string type overrides
    let string_overrides = known_string_type_overrides();
    if let Some(&override_type) = string_overrides.get(prop_name) {
        // Extract display name from override type string
        // e.g., "carina_aws_types::security_group_id()" -> "SecurityGroupId"
        //       "carina_aws_types::iam_role_arn()" -> "IamRoleArn"
        return override_type_to_display_name(override_type).to_string();
    }

    // Normalize plural forms to singular for type inference
    // e.g., "SubnetIds" -> "SubnetId", "CidrBlocks" -> "CidrBlock"
    let singular_name = if prop_name.ends_with("Ids")
        || prop_name.ends_with("ids")
        || prop_name.ends_with("Arns")
        || prop_name.ends_with("arns")
    {
        &prop_name[..prop_name.len() - 1]
    } else {
        prop_name
    };

    // Check overrides for singular form too (e.g., list items)
    if let Some(&override_type) = string_overrides.get(singular_name) {
        return override_type_to_display_name(override_type).to_string();
    }

    let prop_lower = singular_name.to_lowercase();
    if prop_lower.contains("cidr") {
        if prop_lower.contains("ipv6") {
            "Ipv6Cidr".to_string()
        } else {
            "Ipv4Cidr".to_string()
        }
    } else if (prop_lower.contains("ipaddress")
        || prop_lower.ends_with("ip")
        || prop_lower.contains("ipaddresses"))
        && !prop_lower.contains("cidr")
        && !prop_lower.contains("count")
        && !prop_lower.contains("type")
    {
        if prop_lower.contains("ipv6") {
            "Ipv6Address".to_string()
        } else {
            "Ipv4Address".to_string()
        }
    } else if prop_lower == "availabilityzone" || prop_lower == "availabilityzones" {
        "AvailabilityZone".to_string()
    } else if prop_lower == "availabilityzoneid" || prop_lower == "availabilityzoneids" {
        "AvailabilityZoneId".to_string()
    } else if prop_lower.ends_with("region") || prop_lower == "regionname" {
        "Region".to_string()
    } else if prop_lower.ends_with("arn")
        || prop_lower.ends_with("arns")
        || prop_lower.contains("_arn")
    {
        "Arn".to_string()
    } else if is_ipam_pool_id_property(singular_name) {
        "IpamPoolId".to_string()
    } else if is_aws_resource_id_property(singular_name, Some(resource_type)) {
        get_resource_id_display_name(singular_name, Some(resource_type)).to_string()
    } else if prop_lower.ends_with("ownerid") || prop_lower.ends_with("accountid") {
        "AwsAccountId".to_string()
    } else {
        "String".to_string()
    }
}

/// Convert an override type string to a display name
/// e.g., "carina_aws_types::security_group_id()" -> "SecurityGroupId"
fn override_type_to_display_name(override_type: &str) -> &str {
    match override_type {
        "carina_aws_types::security_group_id()" => "SecurityGroupId",
        "carina_aws_types::aws_resource_id()" => "AwsResourceId",
        "carina_aws_types::iam_role_arn()" | "super::super::iam::role::arn()" => "IamRoleArn",
        "carina_aws_types::iam_policy_arn()" | "super::super::iam::policy::arn()" => "IamPolicyArn",
        "carina_aws_types::iam_oidc_provider_arn()" | "super::super::iam::oidc_provider::arn()" => {
            "IamOidcProviderArn"
        }
        "carina_aws_types::kms_key_arn()" | "super::super::kms::key::arn()" => "KmsKeyArn",
        "carina_aws_types::kms_key_id()" => "KmsKeyId",
        "carina_aws_types::gateway_id()" => "GatewayId",
        "carina_aws_types::network_acl_id()" => "NetworkAclId",
        "carina_aws_types::aws_account_id()" => "AwsAccountId",
        "carina_aws_types::instance_id()" => "InstanceId",
        "carina_aws_types::network_interface_id()" => "NetworkInterfaceId",
        "carina_aws_types::allocation_id()" => "AllocationId",
        "carina_aws_types::prefix_list_id()" => "PrefixListId",
        "carina_aws_types::carrier_gateway_id()" => "CarrierGatewayId",
        "carina_aws_types::local_gateway_id()" => "LocalGatewayId",
        "carina_aws_types::egress_only_internet_gateway_id()" => "EgressOnlyInternetGatewayId",
        "carina_aws_types::transit_gateway_id()" => "TransitGatewayId",
        "carina_aws_types::vpc_peering_connection_id()" => "VpcPeeringConnectionId",
        "carina_aws_types::vpc_endpoint_id()" => "VpcEndpointId",
        "carina_aws_types::transit_gateway_attachment_id()" => "TransitGatewayAttachmentId",
        "carina_aws_types::flow_log_id()" => "FlowLogId",
        "carina_aws_types::subnet_route_table_association_id()" => "SubnetRouteTableAssociationId",
        "carina_aws_types::ipam_id()" => "IpamId",
        "carina_aws_types::iam_role_id()" => "IamRoleId",
        "carina_aws_types::aws_region()" => "Region",
        "types::ipv4_address()" => "Ipv4Address",
        "carina_aws_types::arn()" => "Arn",
        "carina_aws_types::ipam_pool_id()" => "IpamPoolId",
        "carina_aws_types::vpc_cidr_block_association_id()" => "VpcCidrBlockAssociationId",
        "carina_aws_types::tgw_route_table_id()" => "TgwRouteTableId",
        "types::cidr()" => "Cidr",
        "types::email()" => "Email",
        "AttributeType::string()" => "String",
        _ => "String",
    }
}

/// Determine the display string for a property's type in markdown docs
fn type_display_string(
    prop_name: &str,
    prop: &CfnProperty,
    schema: &CfnSchema,
    enums: &BTreeMap<String, EnumInfo>,
) -> String {
    if enums.contains_key(prop_name) {
        let enum_link = format!(
            "[Enum ({})](#{}-{})",
            enums[prop_name].type_name,
            prop_name.to_snake_case(),
            enums[prop_name].type_name.to_lowercase()
        );
        // Check if property is an array type — if so, wrap as List<Enum>
        let is_array = prop
            .prop_type
            .as_ref()
            .and_then(|t| t.as_str())
            .map(|t| t == "array")
            .unwrap_or(false);
        let is_ref_array = prop
            .ref_path
            .as_ref()
            .and_then(|ref_path| resolve_ref_classified(schema, ref_path))
            .map(|def| matches!(def, ResolvedDef::Array { .. }))
            .unwrap_or(false);
        if is_array || is_ref_array {
            format!("List\\<{}\\>", enum_link)
        } else {
            enum_link
        }
    } else if prop_name == "Tags" {
        "`Map<String, String>`".to_string()
    } else if !prop.one_of.is_empty() {
        if let Some(array_variant) = prop
            .one_of
            .iter()
            .find(|variant| property_resolves_to_array(variant, schema))
        {
            return type_display_string(prop_name, array_variant, schema, enums);
        }

        let has_mergeable_object_variant = prop.one_of.iter().any(|variant| {
            variant
                .properties
                .as_ref()
                .is_some_and(|props| !props.is_empty())
        });
        if !has_mergeable_object_variant {
            let variant_types: Vec<Vec<String>> =
                prop.one_of.iter().map(property_type_values).collect();
            panic!(
                "unresolved oneOf for {}.{}: variants {:?}",
                schema.type_name, prop_name, variant_types
            );
        }

        "String".to_string()
    } else if let Some(ref_path) = &prop.ref_path {
        if ref_path.contains("/Tag") {
            "`Map<String, String>`".to_string()
        } else {
            // Classify the resolved def once. Only Struct/OneOf (rendered as a
            // struct link) and Scalar (awscc#291: WAFv2 `Priority`/`Limit` are
            // $refs to scalar integer defs — recurse for the Int/Float/Bool
            // display with the def's range) get a non-string display; every
            // other shape falls back to the name-based heuristic, as before.
            match resolve_ref_classified(schema, ref_path) {
                Some(ResolvedDef::Struct { .. } | ResolvedDef::OneOf { .. }) => {
                    let def_name = ref_def_name(ref_path).unwrap_or(prop_name);
                    format!("[Struct({})](#{})", def_name, def_name.to_lowercase())
                }
                Some(scalar @ ResolvedDef::Scalar { .. }) => {
                    let scalar_prop = scalar
                        .scalar_as_property()
                        .expect("Scalar variant always rebuilds a scalar property");
                    type_display_string(prop_name, &scalar_prop, schema, enums)
                }
                Some(
                    ResolvedDef::Enum { .. }
                    | ResolvedDef::Array { .. }
                    | ResolvedDef::StringPattern { .. }
                    | ResolvedDef::Opaque,
                )
                | None => {
                    // These shapes have no dedicated non-list scalar display
                    // here; apply the name-based heuristic. Listed explicitly
                    // (no `_`) so a new shape forces a decision at this site.
                    infer_string_type_display(prop_name, &schema.type_name)
                }
            }
        }
    } else {
        match prop.prop_type.as_ref().and_then(|t| t.as_str()) {
            Some("string") => {
                if prop_name.ends_with("PolicyDocument") {
                    "PolicyDocument".to_string()
                } else {
                    let base = infer_string_type_display(prop_name, &schema.type_name);
                    if base != "String" {
                        return base;
                    }
                    let effective_min = prop.min_length.filter(|&m| m > 0);
                    let has_length = effective_min.is_some() || prop.max_length.is_some();
                    // Check for numeric string pattern
                    if let Some(ref pattern) = prop.pattern
                        && is_numeric_string_pattern(pattern)
                    {
                        return if has_length {
                            let range = string_length_display(effective_min, prop.max_length);
                            format!("NumericString(len: {})", range)
                        } else {
                            "NumericString".to_string()
                        };
                    }
                    // Check for pattern + length combination
                    if prop.pattern.is_some() && has_length {
                        let range = string_length_display(effective_min, prop.max_length);
                        return format!("String(pattern, len: {})", range);
                    }
                    // Check for string format constraint
                    if let Some(ref fmt) = prop.format {
                        return format!("String({})", fmt);
                    }
                    // Append length constraint if present and type is plain String
                    if has_length && prop.enum_values.is_none() {
                        let range = string_length_display(effective_min, prop.max_length);
                        format!("String(len: {})", range)
                    } else {
                        base
                    }
                }
            }
            Some("boolean") => "Bool".to_string(),
            Some("integer") => {
                // Check resource-scoped overrides first
                let res_override =
                    resource_type_overrides().get(&(schema.type_name.as_str(), prop_name));
                if let Some(TypeOverride::IntEnum(values)) = res_override {
                    let values_str = values
                        .iter()
                        .map(|v| v.to_string())
                        .collect::<Vec<_>>()
                        .join(", ");
                    return format!("IntEnum([{}])", values_str);
                }
                if let Some(TypeOverride::IntRange(min, max)) = res_override {
                    return format!("Int({})", range_display_string(Some(*min), Some(*max)));
                }
                // Check for integer enum values from schema
                if let Some(enum_values) = &prop.enum_values {
                    let all_ints = enum_values.iter().all(|v| matches!(v, EnumValue::Int(_)));
                    if all_ints && !enum_values.is_empty() {
                        let values_str = enum_values
                            .iter()
                            .map(|v| v.to_string_value())
                            .collect::<Vec<_>>()
                            .join(", ");
                        return format!("IntEnum([{}])", values_str);
                    }
                }
                let range: Option<(Option<i64>, Option<i64>)> =
                    if prop.minimum.is_some() || prop.maximum.is_some() {
                        Some((prop.minimum, prop.maximum))
                    } else {
                        known_int_range_overrides()
                            .get(prop_name)
                            .map(|&(min, max)| (Some(min), Some(max)))
                    };
                if let Some((min, max)) = range {
                    let range_str = range_display_string(min, max);
                    if let Some(ref fmt) = prop.format {
                        format!("Int({}, {})", range_str, fmt)
                    } else {
                        format!("Int({})", range_str)
                    }
                } else if let Some(ref fmt) = prop.format {
                    format!("Int({})", fmt)
                } else {
                    "Int".to_string()
                }
            }
            Some("number") => {
                // Check resource-scoped overrides first
                let res_override =
                    resource_type_overrides().get(&(schema.type_name.as_str(), prop_name));
                if let Some(TypeOverride::IntRange(min, max)) = res_override {
                    return format!("Float({})", range_display_string(Some(*min), Some(*max)));
                }
                let range: Option<(Option<i64>, Option<i64>)> =
                    if prop.minimum.is_some() || prop.maximum.is_some() {
                        Some((prop.minimum, prop.maximum))
                    } else {
                        known_int_range_overrides()
                            .get(prop_name)
                            .map(|&(min, max)| (Some(min), Some(max)))
                    };
                if let Some((min, max)) = range {
                    let range_str = range_display_string(min, max);
                    if let Some(ref fmt) = prop.format {
                        format!("Float({}, {})", range_str, fmt)
                    } else {
                        format!("Float({})", range_str)
                    }
                } else if let Some(ref fmt) = prop.format {
                    format!("Float({})", fmt)
                } else {
                    "Float".to_string()
                }
            }
            Some("array") => {
                let base_display = if let Some(items) = &prop.items {
                    if let Some(ref_path) = &items.ref_path {
                        if !ref_path.contains("/Tag") {
                            // Classify the item def once. Struct items link to
                            // the struct; a registered enum (keyed on the
                            // property name) renders List<Enum>; scalar items
                            // (awscc#291) render List<Int>/Float/Bool to mirror
                            // the schema-code array branch; anything else falls
                            // back to List<String>.
                            let item_def = resolve_ref_classified(schema, ref_path);
                            if let Some(ResolvedDef::Struct { .. }) = item_def
                                && let Some(def_name) = ref_def_name(ref_path)
                            {
                                format!("[List\\<{}\\>](#{})", def_name, def_name.to_lowercase())
                            } else if enums.contains_key(prop_name) {
                                format!(
                                    "List\\<[Enum ({})](#{}-{})\\>",
                                    enums[prop_name].type_name,
                                    prop_name.to_snake_case(),
                                    enums[prop_name].type_name.to_lowercase()
                                )
                            } else if let Some(scalar_prop) =
                                item_def.as_ref().and_then(ResolvedDef::scalar_as_property)
                            {
                                // List items that $ref a scalar def (awscc#291):
                                // render List<Int>/Float/Bool, mirroring the
                                // schema-code array branch.
                                list_element_type_display(
                                    &scalar_prop,
                                    prop_name,
                                    &schema.type_name,
                                )
                            } else {
                                "`List<String>`".to_string()
                            }
                        } else {
                            "`List<Map<String, String>>`".to_string()
                        }
                    } else {
                        list_element_type_display(items, prop_name, &schema.type_name)
                    }
                } else {
                    "`List<String>`".to_string()
                };
                // Append item count constraints if present
                if prop.min_items.is_some() || prop.max_items.is_some() {
                    let constraint = range_display_string(prop.min_items, prop.max_items);
                    format!("{} (items: {})", base_display, constraint)
                } else {
                    base_display
                }
            }
            Some("object") => {
                if let Some(props) = &prop.properties
                    && !props.is_empty()
                {
                    format!("[Struct({})](#{})", prop_name, prop_name.to_lowercase())
                } else if prop_name.ends_with("PolicyDocument") {
                    "PolicyDocument".to_string()
                } else {
                    "`Map<String, String>`".to_string()
                }
            }
            _ => "String".to_string(),
        }
    }
}

fn generate_markdown(schema: &CfnSchema, type_name: &str) -> Result<String> {
    let mut md = String::new();

    let dsl_resource = dsl_resource_name_from_type(type_name)?;
    let namespace = format!("aws.{}", dsl_resource);

    // Build read-only properties set
    let read_only: HashSet<String> = schema
        .read_only_properties
        .iter()
        .map(|p| p.trim_start_matches("/properties/").to_string())
        .collect();

    // Build create-only properties set
    let create_only: HashSet<String> = schema
        .create_only_properties
        .iter()
        .map(|p| p.trim_start_matches("/properties/").to_string())
        .collect();

    // Build write-only properties set
    let write_only: HashSet<String> = schema
        .write_only_properties
        .iter()
        .map(|p| p.trim_start_matches("/properties/").to_string())
        .collect();

    let required: HashSet<String> = schema.required.iter().cloned().collect();

    // Collect enum info and struct definitions
    let mut enums: BTreeMap<String, EnumInfo> = BTreeMap::new();
    let mut struct_defs: BTreeMap<String, StructDefInfo> = BTreeMap::new();

    for (prop_name, prop) in &schema.properties {
        if is_deprecated_property(prop) {
            continue;
        }
        let (_, enum_info) =
            cfn_type_to_carina_type_with_enum(prop, prop_name, schema, &namespace, &enums);
        if let Some(info) = enum_info {
            enums.insert(prop_name.clone(), info);
        }
        // Collect struct definitions from $ref
        collect_struct_defs(prop, prop_name, schema, &[], &mut struct_defs);
    }

    // Scan struct definition fields for enum info (const values, $ref to enum-only definitions)
    for (def_name, def_info) in &struct_defs {
        for (field_name, field_prop) in &def_info.properties {
            if is_deprecated_property(field_prop) {
                continue;
            }
            let composite_key = format!("{}.{}", def_name, field_name);
            // Apply nested-field enum overlay (awscc#246) — same lookup as
            // `generate_schema_code`, so the markdown's enum tables stay in
            // sync with the generated Rust schema.
            if let Some(TypeOverride::Enum(values) | TypeOverride::EnumUnordered(values)) =
                resource_type_overrides().get(&(type_name, composite_key.as_str()))
            {
                enums.insert(
                    composite_key.clone(),
                    enum_info_for_override(field_name, values),
                );
                continue;
            }
            let (_, enum_info) = cfn_type_to_carina_type_with_enum_with_struct_path(
                field_prop,
                field_name,
                schema,
                &namespace,
                &enums,
                &def_info.struct_path,
            );
            if let Some(info) = enum_info {
                enums.insert(composite_key, info);
            }
        }
    }

    qualify_nested_enum_type_names(&mut enums);

    // Title
    md.push_str(&format!("# awscc.{}\n\n", dsl_resource));
    md.push_str(&format!("CloudFormation Type: `{}`\n\n", type_name));

    // Description
    if let Some(desc) = &schema.description {
        md.push_str(&format!("{}\n\n", desc));
    }

    // Argument Reference (writable attributes)
    md.push_str("## Argument Reference\n\n");

    for (prop_name, prop) in &schema.properties {
        if is_deprecated_property(prop) {
            continue;
        }
        if read_only.contains(prop_name) {
            continue;
        }

        let attr_name = prop_name.to_snake_case();
        let is_required = required.contains(prop_name);
        let type_display = type_display_string(prop_name, prop, schema, &enums);

        md.push_str(&format!("### `{}`\n\n", attr_name));
        md.push_str(&format!("- **Type:** {}\n", type_display));
        if is_required {
            md.push_str("- **Required:** Yes\n");
        } else {
            md.push_str("- **Required:** No\n");
        }
        if create_only.contains(prop_name) {
            md.push_str("- **Create-only:** Yes\n");
        }
        if write_only.contains(prop_name) {
            md.push_str("- **Write-only:** Yes\n");
        }
        if let Some(default_val) = &prop.default_value
            && let Some(display) = json_default_to_markdown(default_val)
        {
            md.push_str(&format!("- **Default:** `{}`\n", display));
        }
        md.push('\n');

        if let Some(d) = &prop.description {
            let desc = collapse_whitespace(&d.replace('\n', " "));
            md.push_str(&format!("{}\n\n", desc));
        }
    }

    // Enum values section
    if !enums.is_empty() {
        let aliases = known_enum_aliases();
        md.push_str("## Enum Values\n\n");
        let mut rendered_enums: HashSet<(String, String, Vec<String>)> = HashSet::new();
        for (prop_name, enum_info) in &enums {
            // Skip duplicate enum types with identical values
            // (e.g., Ingress.IpProtocol and Egress.IpProtocol share the same type and values)
            let enum_namespace = prop_name
                .split_once('.')
                .and_then(|(def_name, _)| struct_defs.get(def_name))
                .map(|def_info| namespace_with_struct_path(&namespace, &def_info.struct_path))
                .unwrap_or_else(|| namespace.clone());
            let key = (
                enum_namespace.clone(),
                enum_info.type_name.clone(),
                enum_info.values.clone(),
            );
            if !rendered_enums.insert(key) {
                continue;
            }
            // For composite keys "DefName.FieldName", use just "FieldName" for display
            let field_name = prop_name
                .split('.')
                .next_back()
                .unwrap_or(prop_name.as_str());
            let attr_name = field_name.to_snake_case();
            let prop_aliases = aliases.get(field_name);
            // Per #199 / D7: render the snake_case DSL spelling in the
            // "DSL Identifier" column and the "Shorthand formats" line.
            // The hand-curated `known_enum_aliases` table wins over the
            // mechanical D7 transform when both apply (e.g. "-1" -> "all").
            let dsl_for = |value: &str| -> String {
                if let Some(alias_list) = prop_aliases
                    && let Some((_, alias)) = alias_list.iter().find(|(c, _)| *c == value)
                {
                    return alias.to_string();
                }
                dsl_enum_value(value)
            };
            md.push_str(&format!("### {} ({})\n\n", attr_name, enum_info.type_name));
            md.push_str("| Value | DSL Identifier |\n");
            md.push_str("|-------|----------------|\n");
            for value in &enum_info.values {
                let dsl_value = dsl_for(value);
                let dsl_id = format!("{}.{}.{}", enum_namespace, enum_info.type_name, dsl_value);
                md.push_str(&format!("| `{}` | `{}` |\n", value, dsl_id));
            }
            md.push('\n');
            let empty = String::new();
            let first_value = enum_info.values.first().unwrap_or(&empty);
            let first_dsl = dsl_for(first_value);
            md.push_str(&format!(
                "Shorthand formats: `{}` or `{}.{}`\n\n",
                first_dsl, enum_info.type_name, first_dsl,
            ));
        }
    }

    // Struct Definitions section
    if !struct_defs.is_empty() {
        md.push_str("## Struct Definitions\n\n");
        for def_info in struct_defs.values() {
            md.push_str(&format!("### {}\n\n", def_info.def_name));
            md.push_str("| Field | Type | Required | Description |\n");
            md.push_str("|-------|------|----------|-------------|\n");
            let required_set: HashSet<&str> =
                def_info.required.iter().map(|s| s.as_str()).collect();
            let overrides = known_enum_overrides();
            for (field_name, field_prop) in &def_info.properties {
                if is_deprecated_property(field_prop) {
                    continue;
                }
                let snake_name = field_name.to_snake_case();
                let is_req = required_set.contains(field_name.as_str());
                let composite_key = format!("{}.{}", def_info.def_name, field_name);
                let field_type_display: String = if enums.contains_key(&composite_key) {
                    let enum_info = &enums[&composite_key];
                    let enum_link = format!(
                        "[Enum ({})](#{}-{})",
                        enum_info.type_name,
                        snake_name,
                        enum_info.type_name.to_lowercase()
                    );
                    let is_array = field_prop
                        .prop_type
                        .as_ref()
                        .and_then(|t| t.as_str())
                        .map(|t| t == "array")
                        .unwrap_or(false);
                    if is_array {
                        format!("List\\<{}\\>", enum_link)
                    } else {
                        enum_link
                    }
                } else if overrides.contains_key(field_name.as_str()) {
                    "Enum".to_string()
                } else {
                    type_display_string(field_name, field_prop, schema, &enums)
                };
                let desc = collapse_whitespace(
                    &field_prop
                        .description
                        .as_deref()
                        .unwrap_or("")
                        .replace('\n', " "),
                );
                md.push_str(&format!(
                    "| `{}` | {} | {} | {} |\n",
                    snake_name,
                    field_type_display,
                    if is_req { "Yes" } else { "No" },
                    desc
                ));
            }
            md.push('\n');
        }
    }

    // Attribute Reference (read-only attributes)
    let synth_for_type: Vec<&SyntheticAttribute> = synthetic_attributes()
        .iter()
        .filter(|s| s.cfn_type == type_name)
        .collect();
    let has_read_only = schema
        .properties
        .keys()
        .any(|name| read_only.contains(name))
        || !synth_for_type.is_empty();
    if has_read_only {
        md.push_str("## Attribute Reference\n\n");

        for (prop_name, prop) in &schema.properties {
            if !read_only.contains(prop_name) {
                continue;
            }

            let attr_name = prop_name.to_snake_case();
            let type_display = type_display_string(prop_name, prop, schema, &enums);

            md.push_str(&format!("### `{}`\n\n", attr_name));
            md.push_str(&format!("- **Type:** {}\n", type_display));

            if let Some(d) = &prop.description {
                let desc = collapse_whitespace(&d.replace('\n', " "));
                md.push_str(&format!("\n{}\n\n", desc));
            } else {
                md.push('\n');
            }
        }

        for synth in synth_for_type {
            md.push_str(&format!("### `{}`\n\n", synth.attr_name));
            md.push_str("- **Type:** String\n\n");
            md.push_str(&format!(
                "{}\n\n",
                collapse_whitespace(&synth.description.replace('\n', " "))
            ));
        }
    }

    Ok(md)
}

/// Collect struct definitions from properties for markdown documentation
fn collect_struct_defs(
    prop: &CfnProperty,
    prop_name: &str,
    schema: &CfnSchema,
    struct_path: &[String],
    struct_defs: &mut BTreeMap<String, StructDefInfo>,
) {
    // Handle $ref. Only struct-shaped defs (inline properties, or a oneOf
    // merged into one struct) contribute a struct def; the other shapes carry
    // no nested structs to collect.
    if let Some(ref_path) = &prop.ref_path
        && !ref_path.contains("/Tag")
        && let Some(def_name) = ref_def_name(ref_path)
        && let Some(resolved) = resolve_ref_classified(schema, ref_path)
    {
        match resolved {
            ResolvedDef::Struct {
                properties,
                required,
            } => {
                let def_path = extend_struct_path(struct_path, def_name);
                if !struct_defs.contains_key(def_name) {
                    struct_defs.insert(
                        def_name.to_string(),
                        StructDefInfo {
                            def_name: def_name.to_string(),
                            struct_path: def_path.clone(),
                            properties: properties.clone(),
                            required: required.to_vec(),
                        },
                    );
                    // Recursively collect nested struct defs
                    for (field_name, field_prop) in properties {
                        if is_deprecated_property(field_prop) {
                            continue;
                        }
                        collect_struct_defs(field_prop, field_name, schema, &def_path, struct_defs);
                    }
                }
            }
            ResolvedDef::OneOf { variants } => {
                // Merge oneOf variant properties into a single struct
                let mut merged_props = BTreeMap::new();
                for variant in variants {
                    if let Some(props) = &variant.properties {
                        for (k, v) in props {
                            merged_props.insert(k.clone(), v.clone());
                        }
                    }
                }
                let def_path = extend_struct_path(struct_path, def_name);
                if !merged_props.is_empty() && !struct_defs.contains_key(def_name) {
                    struct_defs.insert(
                        def_name.to_string(),
                        StructDefInfo {
                            def_name: def_name.to_string(),
                            struct_path: def_path.clone(),
                            properties: merged_props.clone(),
                            required: vec![], // oneOf variants are mutually exclusive
                        },
                    );
                    // Recursively collect struct defs from merged properties
                    for (field_name, field_prop) in &merged_props {
                        if is_deprecated_property(field_prop) {
                            continue;
                        }
                        collect_struct_defs(field_prop, field_name, schema, &def_path, struct_defs);
                    }
                }
            }
            ResolvedDef::Enum { .. }
            | ResolvedDef::Array { .. }
            | ResolvedDef::StringPattern { .. }
            | ResolvedDef::Scalar { .. }
            | ResolvedDef::Opaque => {
                // No struct body to collect for these shapes.
            }
        }
    }
    // Handle array items with $ref. Only struct-shaped items contribute.
    if let Some(items) = &prop.items
        && let Some(ref_path) = &items.ref_path
        && !ref_path.contains("/Tag")
        && let Some(def_name) = ref_def_name(ref_path)
        && let Some(ResolvedDef::Struct {
            properties,
            required,
        }) = resolve_ref_classified(schema, ref_path)
        && !struct_defs.contains_key(def_name)
    {
        let def_path = extend_struct_path(struct_path, def_name);
        struct_defs.insert(
            def_name.to_string(),
            StructDefInfo {
                def_name: def_name.to_string(),
                struct_path: def_path.clone(),
                properties: properties.clone(),
                required: required.to_vec(),
            },
        );
        // Recursively collect nested struct defs
        for (field_name, field_prop) in properties {
            if is_deprecated_property(field_prop) {
                continue;
            }
            collect_struct_defs(field_prop, field_name, schema, &def_path, struct_defs);
        }
    }
    // Handle inline object with properties
    if let Some(type_val) = &prop.prop_type
        && type_val.as_str() == Some("object")
        && let Some(props) = &prop.properties
        && !props.is_empty()
        && !struct_defs.contains_key(prop_name)
    {
        let def_path = extend_struct_path(struct_path, prop_name);
        struct_defs.insert(
            prop_name.to_string(),
            StructDefInfo {
                def_name: prop_name.to_string(),
                struct_path: def_path.clone(),
                properties: props.clone(),
                required: prop.required.clone(),
            },
        );
        // Recursively collect nested struct defs
        for (field_name, field_prop) in props {
            if is_deprecated_property(field_prop) {
                continue;
            }
            collect_struct_defs(field_prop, field_name, schema, &def_path, struct_defs);
        }
    }
}

/// Detect mutually exclusive field groups from top-level `oneOf` / `anyOf` patterns.
///
/// CloudFormation schemas may have top-level `oneOf` or `anyOf` arrays where each variant
/// has a different `required` field list. When each variant requires exactly one field and
/// the fields differ across variants, they form a mutually exclusive group.
fn detect_exclusive_from_oneof(schema: &CfnSchema) -> Vec<Vec<String>> {
    let mut groups = Vec::new();

    for variants in [&schema.one_of, &schema.any_of] {
        if variants.len() < 2 {
            continue;
        }
        // Collect the required fields from each variant
        let required_sets: Vec<&Vec<String>> = variants.iter().map(|v| &v.required).collect();

        // Check that every variant has exactly one required field
        if required_sets.iter().all(|r| r.len() == 1) {
            let fields: Vec<String> = required_sets.iter().map(|r| r[0].clone()).collect();
            // Only create a group if all fields are distinct property names
            let unique: HashSet<&String> = fields.iter().collect();
            if unique.len() == fields.len() && fields.len() >= 2 {
                groups.push(fields);
            }
        }
    }

    groups
}

/// Regex for detecting "specify either X or Y" patterns in property descriptions.
/// X and Y can be PascalCase names or ``backtick-quoted`` names.
static EXCLUSIVE_DESC_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)specify either\s*`*([A-Z][A-Za-z0-9]*)`*\s+or\s+`*([A-Z][A-Za-z0-9]*)`*")
        .unwrap()
});

/// Detect mutually exclusive field groups from property description text.
///
/// Looks for patterns like:
/// - "You must specify either InternetGatewayId or VpnGatewayId, but not both"
/// - "You must specify either``CidrBlock`` or ``Ipv4IpamPoolId``"
/// - "Specify either X or Y"
fn detect_exclusive_from_descriptions(schema: &CfnSchema) -> Vec<Vec<String>> {
    let property_names: HashSet<&String> = schema.properties.keys().collect();
    let mut seen_groups: HashSet<Vec<String>> = HashSet::new();

    for prop in schema.properties.values() {
        let desc = match &prop.description {
            Some(d) => d,
            None => continue,
        };

        for cap in EXCLUSIVE_DESC_RE.captures_iter(desc) {
            let field_a = cap[1].to_string();
            let field_b = cap[2].to_string();

            // Only include groups where both fields are actual properties of this resource
            if property_names.contains(&field_a) && property_names.contains(&field_b) {
                let mut group = vec![field_a, field_b];
                group.sort();
                if seen_groups.insert(group.clone()) {
                    eprintln!("  [exclusive:description] detected: [{}]", group.join(", "));
                }
            }
        }
    }

    seen_groups.into_iter().collect()
}

/// Detect all mutually exclusive field groups for a schema, combining both detection methods.
/// Returns a deduplicated list of groups (each group is a sorted Vec of PascalCase field names).
fn detect_exclusive_fields(schema: &CfnSchema, type_name: &str) -> Vec<Vec<String>> {
    let mut all_groups: HashSet<Vec<String>> = HashSet::new();

    // Approach 1: oneOf / anyOf patterns
    let oneof_groups = detect_exclusive_from_oneof(schema);
    for mut group in oneof_groups {
        group.sort();
        eprintln!(
            "  [exclusive:oneOf] {}: detected: [{}]",
            type_name,
            group.join(", ")
        );
        all_groups.insert(group);
    }

    // Approach 2: description text parsing
    let desc_groups = detect_exclusive_from_descriptions(schema);
    for group in desc_groups {
        // group is already sorted
        if all_groups.contains(&group) {
            eprintln!(
                "  [exclusive:both] {}: detected by both methods: [{}]",
                type_name,
                group.join(", ")
            );
        } else {
            all_groups.insert(group);
        }
    }

    all_groups.into_iter().collect()
}

fn generate_schema_code(schema: &CfnSchema, type_name: &str) -> Result<String> {
    // carina#3340: clear per-resource codegen state at function entry
    // (cyclic-$ref guard + emitted-pattern accumulators) so prior
    // resources don't leak entries into this resource's pre-scan or
    // post-emit validator-fn generation.
    reset_recursion_guard();

    let mut code = String::new();

    // Parse type name: AWS::EC2::VPC -> (ec2, vpc)
    let parts: Vec<&str> = type_name.split("::").collect();
    if parts.len() != 3 {
        anyhow::bail!("Invalid type name format: {}", type_name);
    }
    let resource = parts[2].to_snake_case();
    let full_resource = full_resource_name_from_type(type_name)?;
    let dsl_resource = dsl_resource_name_from_type(type_name)?;
    // Namespace for AWS type identities: aws.ec2.Vpc
    let namespace = format!("aws.{}", dsl_resource);
    let arn_helper = resource_identity_parts(&dsl_resource).and_then(|(service, resource)| {
        let has_schema_arn = schema.properties.contains_key("Arn");
        let has_synthetic_arn = synthetic_attributes()
            .iter()
            .any(|synth| synth.cfn_type == type_name && synth.attr_name == "arn");
        (has_schema_arn || has_synthetic_arn)
            .then(|| emit_arn_helper(service, resource, arn_emit_choice(service, resource)))
    });
    let has_arn_helper = arn_helper.is_some();

    // Build read-only properties set
    let read_only: HashSet<String> = schema
        .read_only_properties
        .iter()
        .map(|p| p.trim_start_matches("/properties/").to_string())
        .collect();

    // Build create-only properties set
    let create_only: HashSet<String> = schema
        .create_only_properties
        .iter()
        .map(|p| p.trim_start_matches("/properties/").to_string())
        .collect();

    // Build write-only properties set
    let write_only: HashSet<String> = schema
        .write_only_properties
        .iter()
        .map(|p| p.trim_start_matches("/properties/").to_string())
        .collect();

    let required: HashSet<String> = schema.required.iter().cloned().collect();

    // Pre-scan properties to determine which imports are needed and collect enum info
    let mut needs_types = false;
    let mut needs_attribute_type = false;
    let mut needs_struct_field = false;
    let mut enums: BTreeMap<String, EnumInfo> = BTreeMap::new();
    let mut ranged_ints: BTreeMap<String, (Option<i64>, Option<i64>)> = BTreeMap::new();
    let mut ranged_floats: BTreeMap<String, (Option<i64>, Option<i64>)> = BTreeMap::new();
    let mut int_enums: BTreeMap<String, Vec<i64>> = BTreeMap::new();
    let mut ranged_lists: BTreeMap<String, (Option<i64>, Option<i64>)> = BTreeMap::new();
    let mut patterns: BTreeMap<String, String> = BTreeMap::new();
    let mut ranged_strings: BTreeMap<String, (Option<u64>, Option<u64>)> = BTreeMap::new();
    // Combined pattern + length constraints: (pattern, min_length, max_length) set
    let mut pattern_with_lengths: BTreeSet<(String, Option<u64>, Option<u64>)> = BTreeSet::new();
    // Patterns that are used standalone (without length constraints)
    let mut patterns_used_standalone: BTreeSet<String> = BTreeSet::new();

    for (prop_name, prop) in &schema.properties {
        if is_deprecated_property(prop) {
            continue;
        }
        let (attr_type, enum_info) =
            cfn_type_to_carina_type_with_enum(prop, prop_name, schema, &namespace, &enums);
        if uses_core_schema_types_path(&attr_type) {
            needs_types = true;
        }
        if attr_type.contains("AttributeType::") {
            needs_attribute_type = true;
        }
        if attr_type.contains("StructField::") {
            needs_struct_field = true;
        }
        if let Some(info) = enum_info {
            enums.insert(prop_name.clone(), info);
        }
        // Check resource-scoped overrides for enum, int range, and int enum
        let resource_override = resource_type_overrides().get(&(type_name, prop_name.as_str()));
        if let Some(TypeOverride::Enum(values) | TypeOverride::EnumUnordered(values)) =
            resource_override
        {
            enums.insert(
                prop_name.clone(),
                EnumInfo {
                    type_name: prop_name.clone(),
                    values: values.iter().map(|v| v.to_string()).collect(),
                },
            );
        }
        // Collect ranged integer/number properties (including one-sided ranges)
        match prop.prop_type.as_ref().and_then(|t| t.as_str()) {
            Some("integer") => {
                if let Some(TypeOverride::IntRange(min, max)) = resource_override {
                    ranged_ints.insert(prop_name.clone(), (Some(*min), Some(*max)));
                } else if let Some(TypeOverride::IntEnum(values)) = resource_override {
                    int_enums.insert(prop_name.clone(), values.clone());
                } else if prop.minimum.is_some() || prop.maximum.is_some() {
                    ranged_ints.insert(prop_name.clone(), (prop.minimum, prop.maximum));
                } else if let Some(&(min, max)) =
                    known_int_range_overrides().get(prop_name.as_str())
                {
                    ranged_ints.insert(prop_name.clone(), (Some(min), Some(max)));
                }
                // Collect integer enum values (only if no resource-scoped override)
                if resource_override.is_none()
                    && let Some(enum_values) = &prop.enum_values
                {
                    let values: Vec<i64> = enum_values
                        .iter()
                        .filter_map(|v| match v {
                            EnumValue::Int(i) => Some(*i),
                            _ => None,
                        })
                        .collect();
                    if !values.is_empty()
                        && enum_values.iter().all(|v| matches!(v, EnumValue::Int(_)))
                    {
                        int_enums.insert(prop_name.clone(), values);
                    }
                }
            }
            Some("number") => {
                if let Some(TypeOverride::IntRange(min, max)) = resource_override {
                    ranged_floats.insert(prop_name.clone(), (Some(*min), Some(*max)));
                } else if prop.minimum.is_some() || prop.maximum.is_some() {
                    ranged_floats.insert(prop_name.clone(), (prop.minimum, prop.maximum));
                } else if let Some(&(min, max)) =
                    known_int_range_overrides().get(prop_name.as_str())
                {
                    ranged_floats.insert(prop_name.clone(), (Some(min), Some(max)));
                }
            }
            Some("array") if prop.min_items.is_some() || prop.max_items.is_some() => {
                ranged_lists.insert(prop_name.clone(), (prop.min_items, prop.max_items));
            }
            Some("string") => {
                // Collect string length constraints (only for plain strings without
                // enum values or type overrides that would take precedence)
                // Treat minLength=0 as no constraint since usize is always >= 0
                let effective_min = prop.min_length.filter(|&m| m > 0);
                if effective_min.is_some() || prop.max_length.is_some() {
                    let has_desc_enum = prop
                        .description
                        .as_ref()
                        .and_then(|d| extract_enum_from_description(d))
                        .is_some();
                    if prop.enum_values.is_none()
                        && !enums.contains_key(prop_name)
                        && !has_desc_enum
                        && infer_string_type(prop_name, &schema.type_name).is_none()
                        && !prop_name.ends_with("PolicyDocument")
                        && prop_name != "Tags"
                        && prop.pattern.is_none()
                        && !matches!(
                            resource_type_overrides().get(&(schema.type_name.as_str(), prop_name)),
                            Some(TypeOverride::ToDsl(_))
                        )
                    {
                        ranged_strings.insert(prop_name.clone(), (effective_min, prop.max_length));
                    }
                }
            }
            _ => {}
        }
        // Collect pattern constraints for string properties (or array items with patterns)
        if attr_type.contains("validate_") && attr_type.contains("_pattern") {
            // Pattern can come from the property directly, from a $ref definition,
            // or from array items (e.g., type: array, items: { pattern: ... })
            // Track which source provided the pattern so we use the correct length constraints.
            let pattern_source: Option<(String, Option<u64>, Option<u64>)> = prop
                .pattern
                .as_ref()
                .map(|p| (p.clone(), prop.min_length, prop.max_length))
                .or_else(|| {
                    prop.ref_path
                        .as_ref()
                        .and_then(|ref_path| resolve_ref_classified(schema, ref_path))
                        .and_then(|def| match def {
                            ResolvedDef::StringPattern {
                                pattern,
                                min_length,
                                max_length,
                            } => Some((pattern.to_string(), min_length, max_length)),
                            _ => None,
                        })
                })
                .or_else(|| {
                    prop.items.as_ref().and_then(|items| {
                        items
                            .pattern
                            .as_ref()
                            .map(|p| (p.clone(), items.min_length, items.max_length))
                    })
                });
            if let Some((pattern, min_length, max_length)) = pattern_source {
                // Check if this pattern also has length constraints
                let effective_min = min_length.filter(|&m| m > 0);
                let has_length = effective_min.is_some() || max_length.is_some();
                if has_length {
                    pattern_with_lengths.insert((pattern.clone(), effective_min, max_length));
                } else {
                    patterns_used_standalone.insert(pattern.clone());
                }
                patterns.insert(prop_name.clone(), pattern);
            }
        }
    }

    // Also scan definitions for struct field integer/number properties
    let int_overrides = known_int_range_overrides();
    if let Some(definitions) = &schema.definitions {
        for def in definitions.values() {
            if let Some(props) = &def.properties {
                for (field_name, field_prop) in props {
                    if is_deprecated_property(field_prop) {
                        continue;
                    }
                    let prop_type = field_prop.prop_type.as_ref().and_then(|t| t.as_str());
                    if matches!(prop_type, Some("integer") | Some("number")) {
                        // Collect ranges from definitions (including one-sided ranges)
                        if field_prop.minimum.is_some() || field_prop.maximum.is_some() {
                            if prop_type == Some("number") {
                                if !ranged_floats.contains_key(field_name) {
                                    ranged_floats.insert(
                                        field_name.clone(),
                                        (field_prop.minimum, field_prop.maximum),
                                    );
                                }
                            } else if !ranged_ints.contains_key(field_name) {
                                ranged_ints.insert(
                                    field_name.clone(),
                                    (field_prop.minimum, field_prop.maximum),
                                );
                            }
                        } else if int_overrides.contains_key(field_name.as_str()) {
                            // Fall back to known overrides
                            let (min, max) = int_overrides[field_name.as_str()];
                            if prop_type == Some("number") {
                                if !ranged_floats.contains_key(field_name) {
                                    ranged_floats
                                        .insert(field_name.clone(), (Some(min), Some(max)));
                                }
                            } else if !ranged_ints.contains_key(field_name) {
                                ranged_ints.insert(field_name.clone(), (Some(min), Some(max)));
                            }
                        }
                    }
                    // Collect array item count constraints from definitions
                    if prop_type == Some("array")
                        && (field_prop.min_items.is_some() || field_prop.max_items.is_some())
                        && !ranged_lists.contains_key(field_name)
                    {
                        ranged_lists.insert(
                            field_name.clone(),
                            (field_prop.min_items, field_prop.max_items),
                        );
                    }
                    // Also collect integer enums from definitions
                    if prop_type == Some("integer")
                        && let Some(enum_values) = &field_prop.enum_values
                    {
                        let values: Vec<i64> = enum_values
                            .iter()
                            .filter_map(|v| match v {
                                EnumValue::Int(i) => Some(*i),
                                _ => None,
                            })
                            .collect();
                        if !values.is_empty()
                            && enum_values.iter().all(|v| matches!(v, EnumValue::Int(_)))
                            && !int_enums.contains_key(field_name)
                        {
                            int_enums.insert(field_name.clone(), values);
                        }
                    }
                    // Also collect pattern constraints from definitions
                    if prop_type == Some("string")
                        && field_prop.pattern.is_some()
                        && !patterns.contains_key(field_name)
                    {
                        // Check if this field would actually use a pattern type
                        // (skip if it has a known type override)
                        let (field_type, _) = cfn_type_to_carina_type_with_enum(
                            field_prop, field_name, schema, &namespace, &enums,
                        );
                        if field_type.contains("validate_") && field_type.contains("_pattern") {
                            let pattern = field_prop.pattern.clone().unwrap();
                            // Check if this pattern also has length constraints
                            let effective_min = field_prop.min_length.filter(|&m| m > 0);
                            let has_length =
                                effective_min.is_some() || field_prop.max_length.is_some();
                            if has_length {
                                pattern_with_lengths.insert((
                                    pattern.clone(),
                                    effective_min,
                                    field_prop.max_length,
                                ));
                            } else {
                                patterns_used_standalone.insert(pattern.clone());
                            }
                            patterns.insert(field_name.clone(), pattern);
                        }
                    }
                    // Collect pattern constraints from $ref to string definitions with patterns
                    if let Some(ref_path) = &field_prop.ref_path
                        && let Some(ResolvedDef::StringPattern {
                            pattern,
                            min_length,
                            max_length,
                        }) = resolve_ref_classified(schema, ref_path)
                        && !patterns.contains_key(field_name)
                    {
                        let (field_type, _) = cfn_type_to_carina_type_with_enum(
                            field_prop, field_name, schema, &namespace, &enums,
                        );
                        if field_type.contains("validate_") && field_type.contains("_pattern") {
                            // Check length constraints from the $ref definition
                            let effective_min = min_length.filter(|&m| m > 0);
                            let has_length = effective_min.is_some() || max_length.is_some();
                            if has_length {
                                pattern_with_lengths.insert((
                                    pattern.to_string(),
                                    effective_min,
                                    max_length,
                                ));
                            } else {
                                patterns_used_standalone.insert(pattern.to_string());
                            }
                            patterns.insert(field_name.clone(), pattern.to_string());
                        }
                    }
                }
            }
        }
    }

    // Also scan definitions for struct field string length constraints
    // (including array items with string length constraints)
    // Skip the standard "Tag" definition since it's handled by tags_type().
    // Non-standard tag definitions like "HostedZoneTag" are rendered as structs
    // and need their validators collected.
    if let Some(definitions) = &schema.definitions {
        for (def_name, def) in definitions {
            if def_name == "Tag" {
                continue;
            }
            if let Some(props) = &def.properties {
                for (field_name, field_prop) in props {
                    if is_deprecated_property(field_prop) {
                        continue;
                    }
                    collect_string_length_constraints(field_name, field_prop, &mut ranged_strings);
                }
            }
        }
    }
    // Also scan top-level array items for string length constraints
    for (prop_name, prop) in &schema.properties {
        if is_deprecated_property(prop) {
            continue;
        }
        if let Some(items) = &prop.items {
            collect_string_length_constraints(prop_name, items, &mut ranged_strings);
        }
    }

    // Also scan definitions for struct field enum properties
    // Use composite key "DefName.FieldName" to avoid conflicts when different
    // definitions have fields with the same name but different enum values
    // (e.g., IntelligentTieringConfiguration.Status vs VersioningConfiguration.Status)
    if let Some(definitions) = &schema.definitions {
        for (def_name, def) in definitions {
            if let Some(props) = &def.properties {
                for (field_name, field_prop) in props {
                    if is_deprecated_property(field_prop) {
                        continue;
                    }
                    let composite_key = format!("{}.{}", def_name, field_name);
                    // Curated nested-field enum overlay (awscc#246). Wins over
                    // the natural cfn_type inference so that fields the CFN
                    // schema leaves as plain strings still register as enums
                    // in the `enums` map (and therefore get a VALID_* constant
                    // and an enum_valid_values() entry alongside StringEnum
                    // emission in `generate_struct_type`).
                    if let Some(TypeOverride::Enum(values) | TypeOverride::EnumUnordered(values)) =
                        resource_type_overrides().get(&(type_name, composite_key.as_str()))
                    {
                        enums.insert(
                            composite_key.clone(),
                            enum_info_for_override(field_name, values),
                        );
                        continue;
                    }
                    let (_, field_enum_info) = cfn_type_to_carina_type_with_enum(
                        field_prop, field_name, schema, &namespace, &enums,
                    );
                    if let Some(info) = field_enum_info {
                        enums.insert(composite_key, info);
                    }
                }
            }
        }
    }

    qualify_nested_enum_type_names(&mut enums);

    // The top-level prop pre-scan above recursed into `$ref` struct
    // definitions to populate `enums` and the various pattern/range
    // caches. That same recursion also populated `EMITTED_DEFS` with
    // def bodies that hard-coded the *unqualified* enum `type_name`s.
    // Now that qualification has rewritten the enums map, drop those
    // stale bodies so the upcoming attribute-emission phase re-walks
    // every def with the qualified enums in scope. (carina#3350.)
    //
    // Note: pattern / range / int-enum caches do not embed enum
    // `type_name`s — they key on pattern strings, numeric bounds, and
    // integer values — so they survive qualification unchanged.
    EMITTED_DEFS.with(|s| s.borrow_mut().clear());

    let has_enums = !enums.is_empty();
    let has_ranged_ints = !ranged_ints.is_empty();
    let has_ranged_floats = !ranged_floats.is_empty();
    let has_int_enums = !int_enums.is_empty();
    let has_ranged_lists = !ranged_lists.is_empty();
    let has_patterns = !patterns.is_empty();
    let has_ranged_strings = !ranged_strings.is_empty();
    // Enums use AttributeType::Custom with AttributeType::String base
    if has_enums {
        needs_attribute_type = true;
    }

    // Ranged ints/floats/lists/strings use AttributeType::Custom with AttributeType::Int/Float/List/String base
    if has_ranged_ints || has_ranged_floats || has_ranged_lists || has_ranged_strings {
        needs_attribute_type = true;
    }

    // Int enums use AttributeType::Custom with AttributeType::Int base
    if has_int_enums {
        needs_attribute_type = true;
    }

    // Patterns use AttributeType::Custom with AttributeType::String base
    if has_patterns {
        needs_attribute_type = true;
    }

    // Determine has_tags from tagging metadata
    let has_tags = schema.tagging.as_ref().map(|t| t.taggable).unwrap_or(false);

    // Detect mutually exclusive field groups from schema structure and descriptions
    let exclusive_groups = detect_exclusive_fields(schema, type_name);

    // Build the body into a separate buffer so import selection can scan it
    // for `legacy_validator(` mentions instead of
    // approximating from `has_*` flags (which over-included for files whose
    // enums lower to `types::*` rather than to `Custom`). The header — including
    // all `use` lines — is constructed at the end.
    let mut body = String::new();
    if let Some(helper) = &arn_helper {
        body.push_str(helper);
    }

    // Aliases feed the `dsl_aliases` table on each `StringEnum` below.
    let aliases = known_enum_aliases();

    // Generate enum constants.
    // Constants are always emitted (referenced by enum_valid_values()).
    for (prop_name, enum_info) in &enums {
        let const_name = format!("VALID_{}", prop_name.to_snake_case().to_uppercase());

        // Generate constants from canonical AWS wire values only. DSL spellings
        // live in `dsl_aliases`; mixing them into `VALID_*` makes the host treat
        // aliases as API-canonical.
        let values_str = enum_info
            .values
            .iter()
            .map(|v| rust_lit(v).to_string())
            .collect::<Vec<_>>()
            .join(", ");
        body.push_str(&format!(
            "const {}: &[&str] = &[{}];\n\n",
            const_name, values_str
        ));
    }

    // Generate range validation functions for integer properties
    for (prop_name, (min, max)) in &ranged_ints {
        let fn_name = format!("validate_{}_range", prop_name.to_snake_case());
        let (condition, range_display) = int_range_condition_and_display(*min, *max);
        body.push_str(&format!(
            r#"#[allow(dead_code)]
fn {}(value: &Value) -> Result<(), String> {{
    if let Value::Concrete(ConcreteValue::Int(n)) = value {{
        if {} {{
            Err(format!("Value {{}} is out of range {}", n))
        }} else {{
            Ok(())
        }}
    }} else {{
        Err("Expected integer".to_string())
    }}
}}

"#,
            fn_name, condition, range_display
        ));
    }

    // Generate range validation functions for float (number) properties
    for (prop_name, (min, max)) in &ranged_floats {
        let fn_name = format!("validate_{}_range", prop_name.to_snake_case());
        let (condition, range_display) = float_range_condition_and_display(*min, *max);
        body.push_str(&format!(
            r#"#[allow(dead_code)]
fn {}(value: &Value) -> Result<(), String> {{
    let n = match value {{
        Value::Concrete(ConcreteValue::Int(i)) => *i as f64,
        Value::Concrete(ConcreteValue::Float(f)) => *f,
        _ => return Err("Expected number".to_string()),
    }};
    if {} {{
        Err(format!("Value {{}} is out of range {}", n))
    }} else {{
        Ok(())
    }}
}}

"#,
            fn_name, condition, range_display
        ));
    }

    // Generate validation functions for integer enum properties
    for (prop_name, values) in &int_enums {
        let fn_name = format!("validate_{}_int_enum", prop_name.to_snake_case());
        let const_name = format!("VALID_{}_VALUES", prop_name.to_snake_case().to_uppercase());
        let values_str = values
            .iter()
            .map(|v| v.to_string())
            .collect::<Vec<_>>()
            .join(", ");
        body.push_str(&format!(
            r#"#[allow(dead_code)]
const {}: &[i64] = &[{}];

#[allow(dead_code)]
fn {}(value: &Value) -> Result<(), String> {{
    if let Value::Concrete(ConcreteValue::Int(n)) = value {{
        if {}.contains(n) {{
            Ok(())
        }} else {{
            Err(format!("Value {{}} is not a valid value", n))
        }}
    }} else {{
        Err("Expected integer".to_string())
    }}
}}

"#,
            const_name, values_str, fn_name, const_name
        ));
    }

    // Generate pattern validation functions for string properties (deduplicated)
    // Multiple properties with the same pattern share one validation function.
    // Skip patterns that are only used with combined length constraints
    // (those get combined validation functions generated separately).
    {
        let mut generated_patterns: HashSet<String> = HashSet::new();
        for pattern in patterns.values() {
            if generated_patterns.contains(pattern) {
                continue;
            }
            // Skip if this pattern is only used with combined length constraints
            if pattern_with_lengths.iter().any(|(p, _, _)| p == pattern)
                && !patterns_used_standalone.contains(pattern)
            {
                generated_patterns.insert(pattern.clone());
                continue;
            }
            generated_patterns.insert(pattern.clone());
            let fn_name = pattern_fn_name(pattern);
            // Convert PCRE-specific constructs to Rust regex equivalents
            // (falling back to `.*` if the pattern uses unsupported features).
            let rust_pattern = rust_compatible_pattern(pattern);
            // Escape for Rust string literal (double the backslashes, escape quotes)
            let escaped_for_rust = rust_pattern.replace('\\', "\\\\").replace('"', "\\\"");
            // For the error message, also escape { and } for the inner format!() macro
            let escaped_for_msg = escaped_for_rust.replace('{', "{{").replace('}', "}}");
            body.push_str("#[allow(dead_code)]\n");
            body.push_str("fn ");
            body.push_str(&fn_name);
            body.push_str("(value: &Value) -> Result<(), String> {\n");
            body.push_str("    if let Value::Concrete(ConcreteValue::String(s)) = value {\n");
            body.push_str(
                "        static RE: std::sync::LazyLock<Regex> = std::sync::LazyLock::new(|| {\n",
            );
            body.push_str(&format!(
                "            Regex::new(\"{}\").expect(\"invalid pattern regex\")\n", // rust-lit-guard: allow (already escaped; preserves regex syntax)
                escaped_for_rust
            ));
            body.push_str("        });\n");
            body.push_str("        if RE.is_match(s) {\n");
            body.push_str("            Ok(())\n");
            body.push_str("        } else {\n");
            body.push_str(&format!(
                "            Err(format!(\"Value '{{}}' does not match pattern {}\", s))\n",
                escaped_for_msg
            ));
            body.push_str("        }\n");
            body.push_str("    } else {\n");
            body.push_str("        Err(\"Expected string\".to_string())\n");
            body.push_str("    }\n");
            body.push_str("}\n\n");
        }
    }

    // Generate validation functions for array item count constraints (deduplicated)
    // Multiple properties with the same min/max items share one validation function.
    {
        let mut generated_items: HashSet<(Option<i64>, Option<i64>)> = HashSet::new();
        for (min, max) in ranged_lists.values() {
            let key = (*min, *max);
            if generated_items.contains(&key) {
                continue;
            }
            generated_items.insert(key);
            let fn_name = list_items_fn_name(*min, *max);
            let (condition, range_display) = list_items_condition_and_display(*min, *max);
            body.push_str(&format!(
                r#"#[allow(dead_code)]
fn {}(value: &Value) -> Result<(), String> {{
    if let Value::Concrete(ConcreteValue::List(items)) = value {{
        let len = items.len();
        if {} {{
            Err(format!("List has {{}} items, expected {}", len))
        }} else {{
            Ok(())
        }}
    }} else {{
        Err("Expected list".to_string())
    }}
}}

"#,
                fn_name, condition, range_display
            ));
        }
    }

    // Generate length validation functions for string properties (deduplicated)
    // Multiple properties with the same min/max length share one validation function.
    {
        let mut generated_lengths: HashSet<(Option<u64>, Option<u64>)> = HashSet::new();
        for (min, max) in ranged_strings.values() {
            let key = (*min, *max);
            if generated_lengths.contains(&key) {
                continue;
            }
            generated_lengths.insert(key);
            let fn_name = string_length_fn_name(*min, *max);
            let (condition, range_display) = string_length_condition_and_display(*min, *max);
            body.push_str(&format!(
                r#"#[allow(dead_code)]
fn {}(value: &Value) -> Result<(), String> {{
    if let Value::Concrete(ConcreteValue::String(s)) = value {{
        let len = s.chars().count();
        if {} {{
            Err(format!("String length {{}} is out of range {}", len))
        }} else {{
            Ok(())
        }}
    }} else {{
        Ok(())
    }}
}}

"#,
                fn_name, condition, range_display
            ));
        }
    }

    // Generate combined pattern + length validation functions (deduplicated)
    {
        let mut generated_combined: HashSet<String> = HashSet::new();
        for (pattern, min, max) in &pattern_with_lengths {
            let fn_name = pattern_and_length_fn_name(pattern, *min, *max);
            if generated_combined.contains(&fn_name) {
                continue;
            }
            generated_combined.insert(fn_name.clone());
            let rust_pattern = rust_compatible_pattern(pattern);
            let escaped_for_rust = rust_pattern.replace('\\', "\\\\").replace('"', "\\\"");
            let escaped_for_msg = escaped_for_rust.replace('{', "{{").replace('}', "}}");
            let (len_condition, range_display) = string_length_condition_and_display(*min, *max);
            body.push_str(&format!(
                r#"#[allow(dead_code)]
fn {fn_name}(value: &Value) -> Result<(), String> {{
    if let Value::Concrete(ConcreteValue::String(s)) = value {{
        static RE: std::sync::LazyLock<Regex> = std::sync::LazyLock::new(|| {{
            Regex::new("{escaped_for_rust}").expect("invalid pattern regex")
        }});
        if !RE.is_match(s) {{
            return Err(format!("Value '{{}}' does not match pattern {escaped_for_msg}", s));
        }}
        let len = s.chars().count();
        if {len_condition} {{
            return Err(format!("String length {{}} is out of range {range_display}", len));
        }}
        Ok(())
    }} else {{
        Err("Expected string".to_string())
    }}
}}

"#
            ));
        }
    }

    // Generate config function
    let config_fn_name = format!("{}_config", full_resource);
    // Schema `resource_type` is the un-prefixed DSL form (e.g. `ec2.Vpc`).
    // The provider name (`awscc`) is supplied separately by
    // `SchemaRegistry::insert("awscc", schema)` on the consumer side.
    let schema_name = dsl_resource.clone();

    body.push_str(&format!(
        r#"/// Returns the schema config for {} ({})
pub fn {}() -> AwsccSchemaConfig {{
    AwsccSchemaConfig {{
        aws_type_name: "{}",
        resource_type_name: "{}",
        has_tags: {},
        schema: ResourceSchema::new("{}")
"#,
        full_resource, type_name, config_fn_name, type_name, dsl_resource, has_tags, schema_name
    ));

    // Add description
    if let Some(desc) = &schema.description {
        let escaped_desc = desc
            .replace('\\', "\\\\")
            .replace('"', "\\\"")
            .replace('\n', " ");
        body.push_str(&format!(
            "        .with_description(\"{}\")\n", // rust-lit-guard: allow (already escaped and newline-collapsed)
            escaped_desc
        ));
    }

    // Generate attributes for each property
    for (prop_name, prop) in &schema.properties {
        let attr_name = prop_name.to_snake_case();
        let is_required = required.contains(prop_name) && !read_only.contains(prop_name);
        let is_read_only = read_only.contains(prop_name);
        let is_create_only = create_only.contains(prop_name)
            || create_only
                .iter()
                .any(|p| p.starts_with(&format!("{}/", prop_name)));
        // Write-only: only mark the attribute itself as write-only, NOT when
        // only nested sub-properties are write-only. Unlike create-only (where
        // a nested create-only property forces re-creation of the parent), a
        // nested write-only property doesn't make the entire parent write-only.
        // The parent is still returned by the CloudControl API read; only the
        // specific nested sub-properties are omitted.
        let is_write_only = write_only.contains(prop_name);
        let is_identity = identity_properties().contains(&(type_name, prop_name));
        let is_deferred_populate = deferred_populate_properties().contains(&(type_name, prop_name));

        let attr_type = if has_arn_helper && prop_name == "Arn" {
            "self::arn()".to_string()
        } else if let Some(enum_info) = enums.get(prop_name) {
            // Use shared schema enum type for constrained strings.
            // Emit a `dsl_aliases` data table mapping every canonical value
            // to its snake_case DSL spelling per naming-conventions design
            // D7. Data form (rather than a `fn` closure) is required so the
            // alias map survives the WASM-component boundary — see
            // awscc#199 / carina#2831.
            let prop_aliases = aliases.get(prop_name.as_str());
            let dsl_aliases_code = dsl_aliases_code(&enum_info.values, prop_aliases);
            let values_str = enum_info
                .values
                .iter()
                .map(|v| format!("{}.to_string()", rust_lit(v)))
                .collect::<Vec<_>>()
                .join(", ");
            let enum_type = format!(
                r#"AttributeType::enum_(carina_core::schema::enum_identity({}, Some({})), Some(vec![{}]), {}, None, None)"#,
                rust_lit(&enum_info.type_name),
                rust_lit(&namespace),
                values_str,
                dsl_aliases_code
            );
            // Wrap in List if the property is an array type
            let is_array = prop
                .prop_type
                .as_ref()
                .and_then(|t| t.as_str())
                .map(|t| t == "array")
                .unwrap_or(false);
            // Also check if the property is a $ref to an array-typed definition
            let is_ref_array = prop
                .ref_path
                .as_ref()
                .and_then(|ref_path| resolve_ref_classified(schema, ref_path))
                .map(|def| matches!(def, ResolvedDef::Array { .. }))
                .unwrap_or(false);
            if is_array || is_ref_array {
                let list_ctor = list_constructor(
                    prop.insertion_order,
                    override_forces_unordered(schema.type_name.as_str(), prop_name),
                );
                format!("{}({})", list_ctor, enum_type)
            } else {
                enum_type
            }
        } else {
            let (attr_type, _) =
                cfn_type_to_carina_type_with_enum(prop, prop_name, schema, &namespace, &enums);
            attr_type
        };

        let mut attr_code = format!(
            "        .attribute(\n            AttributeSchema::new(\"{}\", {})", // rust-lit-guard: allow (snake_case output cannot contain " or \\)
            attr_name, attr_type
        );

        if is_required {
            attr_code.push_str("\n                .required()");
        }

        if is_create_only {
            attr_code.push_str("\n                .create_only()");
        }

        if is_read_only {
            attr_code.push_str("\n                .read_only()");
        }

        if is_write_only {
            attr_code.push_str("\n                .write_only()");
        }

        if is_identity {
            attr_code.push_str("\n                .identity()");
        }

        if is_deferred_populate {
            attr_code.push_str("\n                .deferred_populate()");
        }

        if let Some(desc) = &prop.description {
            let escaped = collapse_whitespace(
                &desc
                    .replace('\\', "\\\\")
                    .replace('"', "\\\"")
                    .replace('\n', " "),
            );
            let suffix = if is_read_only { " (read-only)" } else { "" };
            attr_code.push_str(&format!(
                "\n                .with_description(\"{}{}\")", // rust-lit-guard: allow (already escaped and newline-collapsed)
                escaped, suffix
            ));
        }

        // Add provider_name mapping (AWS property name)
        attr_code.push_str(&format!(
            "\n                .with_provider_name({})",
            rust_lit(prop_name)
        ));

        // Add default value if defined in CloudFormation schema. A CFN JSON scalar
        // default whose shape does not match the resolved Carina type (for example
        // a JSON-string default on a structured policy document) would cause a
        // perpetual phantom diff. Policy document fields follow the IAM Role
        // convention: omit the Carina default and let AWS apply its server default.
        if let Some(default_val) = &prop.default_value
            && let Some(default_code) = json_default_to_value_code(default_val)
            && attr_type_accepts_scalar_default(&attr_type)
        {
            attr_code.push_str(&format!(
                "\n                .with_default({})",
                default_code
            ));
        }

        // Add block_name for List(Struct) attributes with a natural singular form.
        // Even if the singular form conflicts with an existing field name,
        // resolve_block_names distinguishes block syntax (Value::List) from
        // attribute assignment (Value::Map) so the block_name is safe to add.
        if (attr_type.contains("list(AttributeType::struct_")
            || is_list_of_struct_property(prop, schema))
            && let Some(singular) = compute_block_name(&attr_name)
        {
            attr_code.push_str(&format!(
                "\n                .with_block_name({})",
                rust_lit(&singular)
            ));
        }

        attr_code.push_str(",\n        )\n");
        body.push_str(&attr_code);
    }

    // Emit synthetic attributes — values the Cloud Control read path does
    // not return because the CFN schema omits them. The provider fills
    // these in on the read path; the schema entry exists so DSL references
    // (e.g. `distribution.arn`) resolve at validate time.
    for synth in synthetic_attributes() {
        if synth.cfn_type != type_name {
            continue;
        }
        let escaped = collapse_whitespace(
            &synth
                .description
                .replace('\\', "\\\\")
                .replace('"', "\\\"")
                .replace('\n', " "),
        );
        body.push_str(&format!(
            concat!(
                "        .attribute(\n",
                "            AttributeSchema::new({}, {})\n",
                "                .read_only()\n",
                "                .with_description(\"{} (read-only)\"),\n        )\n", // rust-lit-guard: allow (already escaped and newline-collapsed)
            ),
            rust_lit(synth.attr_name),
            if has_arn_helper && synth.attr_name == "arn" {
                "self::arn()"
            } else {
                synth.attr_type
            },
            escaped,
        ));
    }

    // Determine name_attribute from primaryIdentifier:
    // If the primary identifier points to a single user-settable (non-read-only) string property,
    // it's the name attribute used for unique name generation during create-before-destroy.
    if let Some(primary_ids) = &schema.primary_identifier
        && primary_ids.len() == 1
    {
        let prop_path = primary_ids[0].trim_start_matches("/properties/");
        if !read_only.contains(prop_path)
            && let Some(prop) = schema.properties.get(prop_path)
        {
            let is_string = prop
                .prop_type
                .as_ref()
                .and_then(|t| t.as_str())
                .map(|t| t == "string")
                .unwrap_or(false);
            if is_string {
                let attr_name = prop_path.to_snake_case();
                body.push_str(&format!(
                    "        .with_name_attribute(\"{}\")\n", // rust-lit-guard: allow (snake_case output cannot contain " or \\)
                    attr_name
                ));
            }
        }
    }

    // Resource types where CloudControl API rejects updates despite the schema
    // having an update handler. These must be replaced instead of updated.
    const FORCE_REPLACE_TYPES: &[&str] = &["AWS::EC2::InternetGateway", "AWS::EC2::IPAM"];
    if FORCE_REPLACE_TYPES.contains(&type_name) {
        body.push_str("        .force_replace()\n");
    }

    // Per-resource operational config for resources with slow CloudControl operations.
    let op_config = operation_config_for_type(type_name);
    if let Some(cfg) = op_config {
        body.push_str("        .with_operation_config(OperationConfig {\n");
        emit_option_field(&mut body, "delete_timeout_secs", cfg.delete_timeout_secs);
        emit_option_field(&mut body, "delete_max_retries", cfg.delete_max_retries);
        emit_option_field(&mut body, "create_timeout_secs", cfg.create_timeout_secs);
        emit_option_field(&mut body, "create_max_retries", cfg.create_max_retries);
        body.push_str("        })\n");
    }

    // Mutually exclusive required groups are emitted as declarative data so
    // the constraint survives the WASM plugin boundary (function-pointer
    // validators are lost when schemas cross into the host).
    for group in &exclusive_groups {
        let fields_str = group
            .iter()
            .map(|f| {
                let snake = f.to_snake_case();
                rust_lit(&snake).to_string()
            })
            .collect::<Vec<_>>()
            .join(", ");
        body.push_str(&format!("        .exclusive_required(&[{}])\n", fields_str));
    }

    // Tags map-shape check is still emitted as a closure. It's a local
    // safety net; the host side re-attaches the equivalent check via
    // `ValidatorType::TagsKeyValueCheck` (see `main.rs::schemas`).
    if has_tags {
        body.push_str("        .with_validator(|attrs| {\n");
        body.push_str("            let mut errors = Vec::new();\n");
        body.push_str("            if let Err(mut e) = carina_aws_types::validate_tags_map(attrs) {\n                errors.append(&mut e);\n            }\n");
        body.push_str(
            "            if errors.is_empty() { Ok(()) } else { Err(errors) }\n        })\n",
        );
    }

    // carina#3340: drain any cyclic-struct definitions that the
    // `$ref` recursion guard accumulated during attribute emission
    // and attach them to the schema via `.with_def(...)`. Non-cyclic
    // resources contribute nothing here; the map is empty and no
    // calls are emitted. The Rust string in `def_body` is already a
    // valid `AttributeType` expression (the same string the inline
    // attribute emitter would have produced).
    let defs = take_emitted_defs();
    for (def_name, def_body) in &defs {
        body.push_str(&format!(
            "        .with_def({}, {})\n",
            rust_lit(def_name),
            def_body
        ));
    }

    // Close the schema (ResourceSchema) and the AwsccSchemaConfig struct
    body.push_str("    }\n}\n");

    // carina#3340: emit validator fns for patterns that the cycle
    // expansion in `cfn_type_to_carina_type_with_enum` referenced
    // but the original `pattern_with_lengths` pre-scan didn't see.
    // The pre-scan walks top-level properties + each definition one
    // hop deep; with the recursion guard, `generate_struct_type` can
    // recurse arbitrarily many hops into cyclic CFN structures
    // (WAFv2 `WebACL.Statement` → `AndStatement` → ...) and emit
    // pattern validators that the pre-scan would never have visited.
    // Each emission site records its (pattern, min, max) into
    // `EMITTED_PATTERNS_WITH_LENGTHS`; here we drain that set,
    // subtract anything the pre-scan already emitted, and append the
    // remaining `fn` definitions. Rust allows fn definitions in any
    // order, so appending after the schema body works.
    let extra_pwl = take_emitted_patterns_with_lengths();
    let extra_standalone = take_emitted_standalone_patterns();
    let already_pwl: HashSet<(String, Option<u64>, Option<u64>)> =
        pattern_with_lengths.iter().cloned().collect();
    for (pattern, min, max) in &extra_pwl {
        if already_pwl.contains(&(pattern.clone(), *min, *max)) {
            continue;
        }
        let fn_name = pattern_and_length_fn_name(pattern, *min, *max);
        let rust_pattern = rust_compatible_pattern(pattern);
        let escaped_for_rust = rust_pattern.replace('\\', "\\\\").replace('"', "\\\"");
        let escaped_for_msg = escaped_for_rust.replace('{', "{{").replace('}', "}}");
        let (len_condition, range_display) = string_length_condition_and_display(*min, *max);
        body.push_str(&format!(
            r#"
#[allow(dead_code)]
fn {fn_name}(value: &Value) -> Result<(), String> {{
    if let Value::Concrete(ConcreteValue::String(s)) = value {{
        static RE: std::sync::LazyLock<Regex> = std::sync::LazyLock::new(|| {{
            Regex::new("{escaped_for_rust}").expect("invalid pattern regex")
        }});
        if !RE.is_match(s) {{
            return Err(format!("Value '{{}}' does not match pattern {escaped_for_msg}", s));
        }}
        let len = s.chars().count();
        if {len_condition} {{
            return Err(format!("String length {{}} is out of range {range_display}", len));
        }}
        Ok(())
    }} else {{
        Err("Expected string".to_string())
    }}
}}
"#
        ));
    }
    // Same shape for int / float range validators referenced by
    // cyclic nested struct fields (carina#3340).
    let extra_ranges = take_emitted_range_validators();
    for (prop_name, (min, max, is_float)) in &extra_ranges {
        // Skip if pre-scan already emitted via ranged_ints / ranged_floats.
        if ranged_ints.contains_key(prop_name) || ranged_floats.contains_key(prop_name) {
            continue;
        }
        let fn_name = format!("validate_{}_range", prop_name.to_snake_case());
        if *is_float {
            let (condition, range_display) = float_range_condition_and_display(*min, *max);
            body.push_str(&format!(
                r#"
#[allow(dead_code)]
fn {}(value: &Value) -> Result<(), String> {{
    let n = match value {{
        Value::Concrete(ConcreteValue::Int(i)) => *i as f64,
        Value::Concrete(ConcreteValue::Float(f)) => *f,
        _ => return Err("Expected number".to_string()),
    }};
    if {} {{
        Err(format!("Value {{}} is out of range {}", n))
    }} else {{
        Ok(())
    }}
}}
"#,
                fn_name, condition, range_display
            ));
        } else {
            let (condition, range_display) = int_range_condition_and_display(*min, *max);
            body.push_str(&format!(
                r#"
#[allow(dead_code)]
fn {}(value: &Value) -> Result<(), String> {{
    if let Value::Concrete(ConcreteValue::Int(n)) = value {{
        if {} {{
            Err(format!("Value {{}} is out of range {}", n))
        }} else {{
            Ok(())
        }}
    }} else {{
        Err("Expected integer".to_string())
    }}
}}
"#,
                fn_name, condition, range_display
            ));
        }
    }

    // Same shape for integer enum validators referenced from cyclic
    // nested fields (carina#3340).
    let extra_int_enums = take_emitted_int_enums();
    for (prop_name, values) in &extra_int_enums {
        if int_enums.contains_key(prop_name) {
            continue;
        }
        let fn_name = format!("validate_{}_int_enum", prop_name.to_snake_case());
        let const_name = format!("VALID_{}_VALUES", prop_name.to_snake_case().to_uppercase());
        let values_str = values
            .iter()
            .map(|v| v.to_string())
            .collect::<Vec<_>>()
            .join(", ");
        body.push_str(&format!(
            r#"
#[allow(dead_code)]
const {}: &[i64] = &[{}];

#[allow(dead_code)]
fn {}(value: &Value) -> Result<(), String> {{
    if let Value::Concrete(ConcreteValue::Int(n)) = value {{
        if {}.contains(n) {{
            Ok(())
        }} else {{
            Err(format!("Value {{}} is not a valid value", n))
        }}
    }} else {{
        Err("Expected integer".to_string())
    }}
}}
"#,
            const_name, values_str, fn_name, const_name
        ));
    }

    // And the standalone (no-length) patterns recorded by the
    // recursion guard. Skip any the pre-scan already emitted.
    for pattern in &extra_standalone {
        if patterns_used_standalone.contains(pattern) || patterns.values().any(|p| p == pattern) {
            continue;
        }
        let fn_name = pattern_fn_name(pattern);
        let rust_pattern = rust_compatible_pattern(pattern);
        let escaped_for_rust = rust_pattern.replace('\\', "\\\\").replace('"', "\\\"");
        let escaped_for_msg = escaped_for_rust.replace('{', "{{").replace('}', "}}");
        body.push_str(&format!(
            r#"
#[allow(dead_code)]
fn {fn_name}(value: &Value) -> Result<(), String> {{
    if let Value::Concrete(ConcreteValue::String(s)) = value {{
        static RE: std::sync::LazyLock<Regex> = std::sync::LazyLock::new(|| {{
            Regex::new("{escaped_for_rust}").expect("invalid pattern regex")
        }});
        if RE.is_match(s) {{
            Ok(())
        }} else {{
            Err(format!("Value '{{}}' does not match pattern {escaped_for_msg}", s))
        }}
    }} else {{
        Err("Expected string".to_string())
    }}
}}
"#
        ));
    }

    // Generate enum_valid_values() function that exposes VALID_* constants
    body.push_str(&format!(
        "\n/// Returns the resource type name and all enum valid values for this module\n\
         pub fn enum_valid_values() -> (&'static str, &'static [(&'static str, &'static [&'static str])]) {{\n\
         {}\
        }}\n",
        if enums.is_empty() {
            format!("    ({}, &[])\n", rust_lit(&dsl_resource))
        } else {
            let entries: Vec<String> = enums
                .keys()
                .map(|prop_name| {
                    // For composite keys "DefName.FieldName", use just "FieldName" for attribute name
                    let field_name = prop_name
                        .split('.')
                        .next_back()
                        .unwrap_or(prop_name.as_str());
                    let attr_name = field_name.to_snake_case();
                    // Constant name uses the full composite key for uniqueness
                    let const_name =
                        format!("VALID_{}", prop_name.to_snake_case().to_uppercase());
                    format!("        (\"{}\", {}),", attr_name, const_name) // rust-lit-guard: allow (snake_case output cannot contain " or \\)
                })
                .collect();
            format!(
                "    ({}, &[\n{}\n    ])\n",
                rust_lit(&dsl_resource),
                entries.join("\n")
            )
        }
    ));

    // `enum_alias_reverse()` / `enum_alias_entries()` are no longer
    // emitted. DSL → API canonical conversion is now done through
    // `DslMap::api_for` against the exhaustive `dsl_aliases` table on
    // each enum, sourced from a single place (see awscc#220).

    // Header is built last so import selection can scan the body for
    // `legacy_validator(` actually-emitted mentions.
    if body.contains("AttributeType::") {
        needs_attribute_type = true;
    }
    if body.contains("StructField::") {
        needs_struct_field = true;
    }
    if uses_core_schema_types_path(&body) {
        needs_types = true;
    }
    let mut schema_imports = vec!["AttributeSchema", "ResourceSchema"];
    if needs_attribute_type {
        schema_imports.insert(1, "AttributeType");
    }
    if needs_struct_field {
        schema_imports.push("StructField");
    }
    if needs_types {
        schema_imports.push("types");
    }
    if body.contains("legacy_validator(") {
        schema_imports.push("legacy_validator");
    }
    const OPERATION_CONFIG_TYPES: &[&str] = &[
        "AWS::EC2::TransitGateway",
        "AWS::EC2::TransitGatewayAttachment",
        "AWS::EC2::IPAM",
        "AWS::EC2::IPAMPool",
        "AWS::EC2::NatGateway",
        "AWS::EC2::VPCGatewayAttachment",
    ];
    if OPERATION_CONFIG_TYPES.contains(&type_name) {
        schema_imports.push("OperationConfig");
    }
    let schema_imports_str = schema_imports.join(", ");
    code.push_str(&format!(
        r#"//! {} schema definition for AWS Cloud Control
//!
//! Auto-generated from CloudFormation schema: {}
//!
//! DO NOT EDIT MANUALLY - regenerate with carina-codegen

use carina_core::schema::{{{}}};
use crate::schemas::config::AwsccSchemaConfig;
"#,
        resource, type_name, schema_imports_str
    ));
    let has_defaults = body.contains(".with_default(");
    if has_ranged_ints
        || has_ranged_floats
        || has_int_enums
        || has_ranged_lists
        || has_ranged_strings
        || has_defaults
        || has_patterns
        || body.contains("Value::")
    {
        code.push_str("use carina_core::resource::{ConcreteValue, Value};\n");
    }
    if body.contains("Regex::new(") {
        code.push_str("use regex::Regex;\n");
    }
    code.push('\n');
    code.push_str(&body);

    Ok(code)
}

/// Codegen-time representation of per-resource operational config.
#[derive(Default)]
struct CodegenOperationConfig {
    delete_timeout_secs: Option<u64>,
    delete_max_retries: Option<u32>,
    create_timeout_secs: Option<u64>,
    create_max_retries: Option<u32>,
}

/// Returns operational config overrides for resource types with slow CloudControl operations.
fn operation_config_for_type(type_name: &str) -> Option<CodegenOperationConfig> {
    match type_name {
        "AWS::EC2::TransitGateway" | "AWS::EC2::TransitGatewayAttachment" => {
            Some(CodegenOperationConfig {
                delete_timeout_secs: Some(1800),
                delete_max_retries: Some(24),
                ..Default::default()
            })
        }
        "AWS::EC2::IPAM" | "AWS::EC2::IPAMPool" => Some(CodegenOperationConfig {
            delete_timeout_secs: Some(1800),
            ..Default::default()
        }),
        "AWS::EC2::NatGateway" => Some(CodegenOperationConfig {
            delete_timeout_secs: Some(1200),
            ..Default::default()
        }),
        "AWS::EC2::VPCGatewayAttachment" => Some(CodegenOperationConfig {
            delete_timeout_secs: Some(1800),
            ..Default::default()
        }),
        _ => None,
    }
}

/// Emit a single `Option<T>` field for generated OperationConfig code.
fn emit_option_field<T: std::fmt::Display>(code: &mut String, name: &str, value: Option<T>) {
    match value {
        Some(v) => code.push_str(&format!("            {name}: Some({v}),\n")),
        None => code.push_str(&format!("            {name}: None,\n")),
    }
}

/// Check if a string looks like a property name (CamelCase or PascalCase)
/// rather than an enum value (lowercase, kebab-case, or UPPER_CASE)
fn looks_like_property_name(s: &str) -> bool {
    if s.is_empty() {
        return false;
    }
    // Property names typically start with uppercase and contain mixed case
    // e.g., "InstanceTenancy", "VpcId"
    let first_char = s.chars().next().unwrap();
    if first_char.is_uppercase() {
        // Check if it has lowercase letters too (CamelCase)
        let has_lowercase = s.chars().any(|c| c.is_lowercase());
        return has_lowercase;
    }
    false
}

/// Check if a string looks like a valid enum value (not a code example, unicode escape, etc.)
fn looks_like_enum_value(s: &str) -> bool {
    if s.is_empty() {
        return false;
    }
    if s.contains('{') || s.contains('}') {
        return false;
    }
    if s.contains("\\u") {
        return false;
    }
    if s.len() > 50 {
        return false;
    }
    if s.contains(' ') {
        return false;
    }
    // Real enum identifiers always include at least one ASCII alphanumeric
    // character. Backticked punctuation in CFN descriptions is usually an
    // allowed-character list, not an enum member.
    if !s.chars().any(|c| c.is_ascii_alphanumeric()) {
        return false;
    }
    // CFN descriptions use double-backticks for both enum members and code-style
    // examples (paths, URLs, file extensions, regex character ranges). They cannot
    // be told apart syntactically, so we exclude shapes that real enum identifiers
    // never take. Escape hatch for genuine outliers: known_enum_overrides.
    if s.contains('/') || s.contains('*') || s.contains(':') || s.contains('.') {
        return false;
    }
    // Regex character ranges in CFN docs (``A-Z``, ``a-z``, ``0-9``).
    if s.len() == 3 && s.contains('-') {
        return false;
    }
    true
}

fn description_accepts_custom_values(description: &str) -> bool {
    let normalized = description.to_lowercase();
    ["the name of a custom", "name of a custom"]
        .iter()
        .any(|marker| normalized.contains(marker))
}

fn filtered_backtick_values(text: &str, backtick_re: &Regex) -> Vec<String> {
    backtick_re
        .captures_iter(text)
        .map(|cap| cap[1].to_string())
        .filter(|v| !looks_like_property_name(v) && looks_like_enum_value(v))
        .collect()
}

/// Return true when all enum-looking backticked values are governed by a
/// negated "specify" clause. For example, the ELBv2 listener JWT claim name
/// description says you can't specify ``exp``, ``iss``, ``nbf``, or ``iat``;
/// those are forbidden values, not an allow-list enum. The negated clause is
/// bounded at semicolon, period, or newline boundaries so a following
/// allow-list clause is not swallowed.
fn description_negates_all_backtick_values(description: &str, backtick_re: &Regex) -> bool {
    let candidate_spans: Vec<(usize, usize)> = backtick_re
        .captures_iter(description)
        .filter_map(|cap| {
            let value = cap.get(1)?.as_str();
            if looks_like_property_name(value) || !looks_like_enum_value(value) {
                return None;
            }
            cap.get(0).map(|m| (m.start(), m.end()))
        })
        .collect();

    if candidate_spans.is_empty() {
        return false;
    }

    let Ok(negated_specify_re) = Regex::new(r"(?is)(?:can't|cannot|can not)\s+specify\b[^.;\n]*")
    else {
        return false;
    };
    let negated_spans: Vec<(usize, usize)> = negated_specify_re
        .find_iter(description)
        .map(|m| (m.start(), m.end()))
        .collect();

    !negated_spans.is_empty()
        && candidate_spans
            .iter()
            .all(|(candidate_start, candidate_end)| {
                negated_spans.iter().any(|(negated_start, negated_end)| {
                    negated_start <= candidate_start && candidate_end <= negated_end
                })
            })
}

/// Extract enum values from description text.
/// Looks for patterns like ``value`` (double backticks) which CloudFormation uses
/// to indicate allowed values in descriptions.
fn extract_enum_from_description(description: &str) -> Option<Vec<String>> {
    if description_accepts_custom_values(description) {
        return None;
    }

    let backtick_re = Regex::new(r"``([^`]+)``").ok()?;
    if description_negates_all_backtick_values(description, &backtick_re) {
        return None;
    }

    // Prefer list members introduced as a leading ``value``: entry. Other
    // backticked identifiers in the list body are examples or property names.
    let lower_description = description.to_lowercase();
    if (lower_description.contains("following") || lower_description.contains("available"))
        && let Ok(bullet_leader_re) =
            Regex::new(r"(?m)(?:^|\n)\s*(?:[+*]|\d+[.)])?\s*``([^`]+)``\s*:")
    {
        let values: Vec<String> = bullet_leader_re
            .captures_iter(description)
            .map(|cap| cap[1].to_string())
            .filter(|v| !looks_like_property_name(v) && looks_like_enum_value(v))
            .collect();

        if values.len() >= 2 {
            return deduplicate_enum_values(values);
        }
    }

    // Scope "supported values are ..." / "valid values are ..." /
    // "valid values: ..." backtick lists to the clause sentence so later
    // code-style mentions are not treated as values.
    if let Ok(scoped_values_re) = Regex::new(
        r"(?is)(?:supported values?\s+(?:are|is)|valid values?):\s*(.+?)(?:\.|\n|$)|(?:supported values?|valid values?)\s+(?:are|is)\s+(.+?)(?:\.|\n|$)",
    ) {
        for cap in scoped_values_re.captures_iter(description) {
            let clause = cap
                .get(1)
                .or_else(|| cap.get(2))
                .map(|m| m.as_str())
                .unwrap_or("");
            let values = filtered_backtick_values(clause, &backtick_re);
            if values.len() >= 2 {
                return deduplicate_enum_values(values);
            }
        }
    }

    // Strategy 1: Look for double-backtick values (existing fallback behavior)
    let mut values: Vec<String> = filtered_backtick_values(description, &backtick_re);

    if values.len() >= 2 {
        return deduplicate_enum_values(values);
    }

    // Strategy 2: Look for "Valid values: X | Y | Z" or "Options: X | Y" patterns
    if let Ok(pipe_re) = Regex::new(r"(?i)(?:valid values?|options?):\s*([^\n.]+)")
        && let Some(cap) = pipe_re.captures(description)
    {
        let list = cap[1].trim();
        // Split by pipe or comma
        let candidates: Vec<String> = if list.contains('|') {
            list.split('|').map(|s| s.trim().to_string()).collect()
        } else if list.contains(',') {
            list.split(',').map(|s| s.trim().to_string()).collect()
        } else {
            vec![]
        };

        values = candidates
            .into_iter()
            .filter(|v| !v.is_empty() && !looks_like_property_name(v))
            .collect();

        if values.len() >= 2 {
            return deduplicate_enum_values(values);
        }
    }

    // Strategy 3: Look for "Options are X, Y, Z" or "Can be X, Y, or Z" patterns
    if let Ok(list_re) =
        Regex::new(r"(?i)(?:options (?:here )?are|can be|either)\s+(.+?)(?:\.|\n|$)")
        && let Some(cap) = list_re.captures(description)
    {
        let list = cap[1].trim();

        // Extract the enum list part before any trailing explanatory text
        // e.g., "default, dedicated, or host for instances" -> "default, dedicated, or host"
        let enum_list = if let Some(idx) = list
            .find(" for ")
            .or_else(|| list.find(" when "))
            .or_else(|| list.find(" where "))
            .or_else(|| list.find(" by "))
            .or_else(|| list.find(" with "))
            .or_else(|| list.find(" that "))
        {
            &list[..idx]
        } else {
            list
        };

        // Split by comma and "or"
        let mut candidates: Vec<String> = vec![];
        for part in enum_list.split(',') {
            let part = part.trim();
            // Handle "or X" pattern
            if let Some(stripped) = part.strip_prefix("or ") {
                candidates.push(stripped.trim().to_string());
            } else if part.contains(" or ") {
                // Split on " or " within a part
                for subpart in part.split(" or ") {
                    candidates.push(subpart.trim().to_string());
                }
            } else {
                candidates.push(part.to_string());
            }
        }

        values = candidates
            .into_iter()
            .filter(|v| !v.is_empty() && !looks_like_property_name(v) && !v.contains(' '))
            .collect();

        if values.len() >= 2 {
            return deduplicate_enum_values(values);
        }
    }

    None
}

/// Preserve enum kind names exactly as detected.
///
/// Nested enum identity is no longer represented by inflating
/// `EnumInfo::type_name`. The generated `TypeIdentity` now uses the enum's
/// own plain kind plus a structural namespace assembled from the enclosing
/// struct type path.
fn qualify_nested_enum_type_names(enums: &mut BTreeMap<String, EnumInfo>) {
    let _ = enums;
}

/// Deduplicate enum values while preserving order.
/// Performs case-insensitive deduplication, preferring uppercase variants
/// (e.g., "GLACIER" over "Glacier") to normalize inconsistent CloudFormation schemas.
fn deduplicate_enum_values(values: Vec<String>) -> Option<Vec<String>> {
    let mut seen_lower: HashMap<String, usize> = HashMap::new();
    let mut unique: Vec<String> = Vec::new();
    for v in values {
        let lower = v.to_lowercase();
        if let Some(&idx) = seen_lower.get(&lower) {
            // Prefer the uppercase variant (e.g., "GLACIER" over "Glacier")
            if v.chars().all(|c| !c.is_alphabetic() || c.is_uppercase()) {
                unique[idx] = v;
            }
        } else {
            seen_lower.insert(lower, unique.len());
            unique.push(v);
        }
    }
    if unique.len() >= 2 {
        Some(unique)
    } else {
        None
    }
}

/// Resolve a `$ref` path to the raw `CfnDefinition`.
/// e.g., "#/definitions/Ingress" -> Some(&CfnDefinition)
///
/// Prefer [`resolve_ref_classified`] in consumers: the raw `CfnDefinition`
/// exposes the open `def_type: Option<String>` discriminant, and
/// hand-dispatching on it is exactly what let a scalar `$ref` be silently
/// typed as `String` (awscc#291). This raw resolver exists only as the
/// building block `resolve_ref_classified` is built on; new code that needs
/// to act on a `$ref`'s shape should go through the classifier so the
/// compiler forces every shape (including `Scalar`) to be handled.
fn resolve_ref<'a>(schema: &'a CfnSchema, ref_path: &str) -> Option<&'a CfnDefinition> {
    let def_name = ref_path.strip_prefix("#/definitions/")?;
    schema.definitions.as_ref()?.get(def_name)
}

/// Extract the definition name from a $ref path
/// e.g., "#/definitions/Ingress" -> Some("Ingress")
fn ref_def_name(ref_path: &str) -> Option<&str> {
    ref_path.strip_prefix("#/definitions/")
}

/// The scalar kinds a `$ref` definition can carry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ScalarKind {
    Integer,
    Number,
    Boolean,
}

impl ScalarKind {
    /// The CloudFormation `"type"` string this kind came from. Used to
    /// reconstruct an inline `CfnProperty` whose typing goes through the same
    /// path a direct property would.
    fn cfn_type(self) -> &'static str {
        match self {
            ScalarKind::Integer => "integer",
            ScalarKind::Number => "number",
            ScalarKind::Boolean => "boolean",
        }
    }
}

/// A `$ref` definition (`CfnDefinition`) classified into the one shape it
/// actually represents.
///
/// `CfnDefinition` is a flat serde DTO whose `def_type` is an open
/// `Option<String>` — nothing in the type system forces a consumer that
/// dispatches on it to handle every shape, so a missed arm silently falls
/// back to `String` (this is exactly how awscc#291 happened: scalar `$ref`s
/// were never matched and became `String`). `classify_def` is the **single
/// source of truth** for that dispatch: it runs the canonical precedence
/// once, and every consumer `match`es the result exhaustively, so the
/// compiler refuses to let a new consumer (or a new shape) drop a case.
///
/// Borrows from the `CfnDefinition` so no cloning happens at classification
/// time; callers clone only the slices they keep.
enum ResolvedDef<'a> {
    /// Object with inline properties.
    Struct {
        properties: &'a BTreeMap<String, CfnProperty>,
        required: &'a [String],
    },
    /// `oneOf` union of variants.
    OneOf { variants: &'a [CfnProperty] },
    /// Enum-only def (a fixed list of values, no properties). Takes
    /// precedence over `Scalar` so an integer-typed enum (e.g. WAFv2
    /// `EvaluationWindowSec`) is treated as an enum, matching the canonical
    /// codegen precedence.
    Enum { values: &'a [EnumValue] },
    /// Array def carrying an item schema.
    Array { items: &'a CfnProperty },
    /// String def constrained by a regex pattern (with optional length).
    StringPattern {
        pattern: &'a str,
        min_length: Option<u64>,
        max_length: Option<u64>,
    },
    /// Scalar (`integer` / `number` / `boolean`) def, carrying its own range
    /// and format so a `$ref` to it reproduces the ranged int/float/bool a
    /// direct property would (awscc#291).
    Scalar {
        kind: ScalarKind,
        minimum: Option<i64>,
        maximum: Option<i64>,
        format: Option<&'a str>,
    },
    /// None of the known shapes — the def has no inline properties, oneOf,
    /// enum, array items, string pattern, or scalar type. Consumers fall back
    /// to name-based string heuristics for these (e.g. an opaque object whose
    /// fields are not modelled). Made an explicit variant — rather than a
    /// catch-all `_` — so adding a new shape forces every consumer to decide
    /// where it belongs instead of silently landing here.
    Opaque,
}

/// Classify a resolved `$ref` definition into the single shape it represents.
///
/// The precedence below is the canonical order the schema-code `$ref` branch
/// has always used: properties → oneOf → enum → array → string-pattern →
/// scalar → opaque. It is the one place that order lives; every `ResolvedDef`
/// consumer inherits it by matching the result. Preserving this order keeps
/// generated output byte-identical (e.g. an integer-typed enum classifies as
/// `Enum`, not `Scalar`, because the enum arm comes first).
fn classify_def(def: &CfnDefinition) -> ResolvedDef<'_> {
    if let Some(props) = &def.properties
        && !props.is_empty()
    {
        return ResolvedDef::Struct {
            properties: props,
            required: &def.required,
        };
    }
    if !def.one_of.is_empty() {
        return ResolvedDef::OneOf {
            variants: &def.one_of,
        };
    }
    if let Some(values) = &def.enum_values
        && !values.is_empty()
    {
        return ResolvedDef::Enum { values };
    }
    if def.def_type.as_deref() == Some("array")
        && let Some(items) = &def.items
    {
        return ResolvedDef::Array { items };
    }
    if def.def_type.as_deref() == Some("string")
        && let Some(pattern) = &def.pattern
    {
        return ResolvedDef::StringPattern {
            pattern,
            min_length: def.min_length,
            max_length: def.max_length,
        };
    }
    let scalar_kind = match def.def_type.as_deref() {
        Some("integer") => Some(ScalarKind::Integer),
        Some("number") => Some(ScalarKind::Number),
        Some("boolean") => Some(ScalarKind::Boolean),
        _ => None,
    };
    if let Some(kind) = scalar_kind {
        return ResolvedDef::Scalar {
            kind,
            minimum: def.minimum,
            maximum: def.maximum,
            format: def.format.as_deref(),
        };
    }
    ResolvedDef::Opaque
}

/// Resolve a `$ref` path and classify the target in one step.
///
/// The primary entry point for `$ref` consumers: returns `None` only when the
/// path does not resolve to a known definition. A resolved-but-unrecognized
/// def yields `ResolvedDef::Opaque`, never a silent `None`.
fn resolve_ref_classified<'a>(schema: &'a CfnSchema, ref_path: &str) -> Option<ResolvedDef<'a>> {
    resolve_ref(schema, ref_path).map(classify_def)
}

impl ResolvedDef<'_> {
    /// If this is a `Scalar`, reconstruct an equivalent inline `CfnProperty`
    /// so callers can recurse through the normal property-typing logic instead
    /// of treating the def as a struct.
    ///
    /// CloudFormation factors recurring scalar fields into shared definitions
    /// — e.g. WAFv2 WebACL's `Rule.Priority` and `RateBasedStatement.Limit`
    /// are `$ref`s to the `RulePriority` / `RateLimit` integer defs. Without
    /// this, the `$ref` branches fall through to the String fallback and
    /// mistype the field, so the resource is rejected at apply (awscc#291).
    /// Returns `None` for every non-scalar shape.
    fn scalar_as_property(&self) -> Option<CfnProperty> {
        match self {
            ResolvedDef::Scalar {
                kind,
                minimum,
                maximum,
                format,
            } => Some(CfnProperty {
                prop_type: Some(TypeValue::Single(kind.cfn_type().to_string())),
                minimum: *minimum,
                maximum: *maximum,
                format: format.map(str::to_string),
                // A scalar def with enum values classifies as `Enum`, not
                // `Scalar`, so the reconstructed property never needs to carry
                // enum values — they are handled by the enum branch upstream.
                ..Default::default()
            }),
            _ => None,
        }
    }
}

/// Generate Rust code for an AttributeType::Struct from a set of properties
fn generate_struct_type(
    def_name: &str,
    properties: &BTreeMap<String, CfnProperty>,
    required: &[String],
    schema: &CfnSchema,
    namespace: &str,
    enums: &BTreeMap<String, EnumInfo>,
    struct_path: &[String],
) -> String {
    let current_struct_path = extend_struct_path(struct_path, def_name);
    let enum_namespace = namespace_with_struct_path(namespace, &current_struct_path);
    let required_set: HashSet<&str> = required.iter().map(|s| s.as_str()).collect();
    let aliases = known_enum_aliases();

    let fields: Vec<String> = properties
        .iter()
        .filter(|(_, field_prop)| !is_deprecated_property(field_prop))
        .map(|(field_name, field_prop)| {
            let snake_name = field_name.to_snake_case();
            let (field_type, mut enum_info) =
                cfn_type_to_carina_type_with_enum_with_struct_path(
                    field_prop,
                    field_name,
                    schema,
                    namespace,
                    enums,
                    &current_struct_path,
                );
            // Apply nested-field enum overlay (awscc#246). When the resource
            // schema does not annotate a struct field as an enum but we have
            // a curated value list in `resource_type_overrides()` keyed by
            // ("AWS::Service::Type", "DefName.FieldName"), promote it to a
            // synthetic StringEnum here. List fields (`array<string>`) keep
            // the list wrapping below; only the item type changes.
            let nested_key = format!("{}.{}", def_name, field_name);
            let nested_override =
                resource_type_overrides().get(&(schema.type_name.as_str(), nested_key.as_str()));
            // `EnumUnordered` is element-handled identically to `Enum`;
            // it differs only in forcing `unordered_list` below
            // (carina#3093).
            if let Some(TypeOverride::Enum(values) | TypeOverride::EnumUnordered(values)) =
                nested_override
            {
                enum_info = Some(enum_info_for_override(field_name, values));
            }
            let field_force_unordered =
                matches!(nested_override, Some(TypeOverride::EnumUnordered(_)));
            // If enum detected in struct field and it's in the enums map, use shared string enum type.
            // Try composite key "DefName.FieldName" first (for definition-scoped enums),
            // then fall back to plain "FieldName" (for top-level property enums).
            // Check if the original field type was a List (array)
            let is_list_field =
                field_type.contains("::list(") || field_type.contains("::unordered_list(");
            let field_type = if let Some(local_enum_info) = enum_info {
                let composite_key = format!("{}.{}", def_name, field_name);
                let enum_type = if let Some(enum_info) =
                    enums.get(&composite_key).or_else(|| enums.get(field_name))
                {
                    let prop_aliases = aliases.get(field_name.as_str());
                    let dsl_aliases_code = dsl_aliases_code(&enum_info.values, prop_aliases);
                    let values_str = enum_info
                        .values
                        .iter()
                        .map(|v| format!("{}.to_string()", rust_lit(v)))
                        .collect::<Vec<_>>()
                        .join(", ");
                    format!(
                        r#"AttributeType::enum_(carina_core::schema::enum_identity({}, Some({})), Some(vec![{}]), {}, None, None)"#,
                        rust_lit(&enum_info.type_name),
                        rust_lit(&enum_namespace),
                        values_str,
                        dsl_aliases_code
                    )
                } else {
                    // Fallback: emit StringEnum with identity even when the
                    // pre-scanned enums map missed this field. The pre-scan
                    // no longer skips struct fields whose name snake-cases
                    // to a top-level prop, so reaching this branch is rare
                    // — but we still apply the same parent-qualification
                    // emitted `TypeIdentity` is still scoped to its
                    // enclosing struct path.
                    let prop_aliases = aliases.get(field_name.as_str());
                    let dsl_aliases_code = dsl_aliases_code(&local_enum_info.values, prop_aliases);
                    let values_str = local_enum_info
                        .values
                        .iter()
                        .map(|v| format!("{}.to_string()", rust_lit(v)))
                        .collect::<Vec<_>>()
                        .join(", ");
                    format!(
                        r#"AttributeType::enum_(carina_core::schema::enum_identity({}, Some({})), Some(vec![{}]), {}, None, None)"#,
                        rust_lit(&local_enum_info.type_name),
                        rust_lit(&enum_namespace),
                        values_str,
                        dsl_aliases_code
                    )
                };
                // Wrap in List if the original field type was a List
                if is_list_field {
                    let list_ctor =
                        list_constructor(field_prop.insertion_order, field_force_unordered);
                    format!("{}({})", list_ctor, enum_type)
                } else {
                    enum_type
                }
            } else {
                field_type
            };
            let is_required = required_set.contains(field_name.as_str());

            let mut field_code =
                format!("StructField::new({}, {})", rust_lit(&snake_name), field_type);
            if is_required {
                field_code.push_str(".required()");
            }
            if let Some(desc) = &field_prop.description {
                let escaped = collapse_whitespace(
                    &desc
                        .replace('\\', "\\\\")
                        .replace('"', "\\\"")
                        .replace('\n', " "),
                );
                field_code.push_str(&format!(".with_description(\"{}\")", escaped)); // rust-lit-guard: allow (already escaped and newline-collapsed)
            }
            field_code.push_str(&format!(".with_provider_name({})", rust_lit(field_name)));

            // Add block_name for List(Struct) fields with a natural singular form.
            // Even if the singular form conflicts with an existing field name,
            // resolve_block_names distinguishes block syntax (Value::List) from
            // attribute assignment (Value::Map) so the block_name is safe to add.
            if (field_type.contains("list(AttributeType::struct_")
                || is_list_of_struct_property(field_prop, schema))
                && let Some(singular) = compute_block_name(&snake_name)
            {
                field_code.push_str(&format!(".with_block_name({})", rust_lit(&singular)));
            }

            field_code
        })
        .collect();

    let fields_str = fields.join(",\n                    ");
    format!(
        "AttributeType::struct_({}.to_string(), vec![{}])",
        rust_lit(def_name),
        fields_str
    )
}

/// Known enum overrides for properties where `extract_enum_from_description()` fails
/// due to inconsistent description formatting in CloudFormation schemas.
fn known_enum_overrides() -> &'static HashMap<&'static str, Vec<&'static str>> {
    static OVERRIDES: LazyLock<HashMap<&'static str, Vec<&'static str>>> = LazyLock::new(|| {
        let mut m = HashMap::new();
        m.insert("IpProtocol", vec!["tcp", "udp", "icmp", "icmpv6", "-1"]);
        m.insert("ConnectivityType", vec!["public", "private"]);
        m.insert("AvailabilityMode", vec!["zonal", "regional"]);
        m.insert("AddressFamily", vec!["IPv4", "IPv6"]);
        m.insert("Domain", vec!["vpc", "standard"]);
        // HostnameType enum values are in parent struct description, not field description
        m.insert("HostnameType", vec!["ip-name", "resource-name"]);
        // InternetGatewayBlockMode removed - now auto-detected via "Options here are" pattern
        // Transit gateway enable/disable properties
        m.insert("AutoAcceptSharedAttachments", vec!["enable", "disable"]);
        m.insert("DefaultRouteTableAssociation", vec!["enable", "disable"]);
        m.insert("DefaultRouteTablePropagation", vec!["enable", "disable"]);
        m.insert("DnsSupport", vec!["enable", "disable"]);
        m.insert("MulticastSupport", vec!["enable", "disable"]);
        m.insert("SecurityGroupReferencingSupport", vec!["enable", "disable"]);
        m.insert("VpnEcmpSupport", vec!["enable", "disable"]);
        m.insert("EncryptionSupportState", vec!["disable", "enable"]);
        // CloudFront ViewerCertificate.MinimumProtocolVersion: CFN schema declares
        // it as a free-form string, so the description-scraping heuristic in
        // extract_enum_from_description() picked up unrelated backticked tokens
        // from sibling fields (`sni-only` from SslSupportMethod, `true` from
        // CloudFrontDefaultCertificate) while filtering out the real value
        // `TLSv1` as a "property name". Pin the canonical security-policy values
        // here so the heuristic is short-circuited (overrides are consulted
        // before description extraction). awscc#242.
        m.insert(
            "MinimumProtocolVersion",
            vec![
                "SSLv3",
                "TLSv1",
                "TLSv1_2016",
                "TLSv1.1_2016",
                "TLSv1.2_2018",
                "TLSv1.2_2019",
                "TLSv1.2_2021",
            ],
        );
        m
    });
    &OVERRIDES
}

/// Known enum aliases for properties where an AWS canonical value should have
/// a human-readable alias in DSL. Maps PropertyName -> [(canonical_aws_value, dsl_alias)].
/// e.g., IpProtocol: "-1" -> "all"
fn known_enum_aliases() -> &'static HashMap<&'static str, Vec<(&'static str, &'static str)>> {
    static ALIASES: LazyLock<HashMap<&'static str, Vec<(&'static str, &'static str)>>> =
        LazyLock::new(|| {
            let mut m = HashMap::new();
            m.insert("IpProtocol", vec![("-1", "all")]);
            // CloudFront security-policy values mix uppercase acronyms (`TLS`),
            // version digits, and dotted minor versions. The mechanical D7
            // snake_case transform splits `TLSv1_2016` as `tl_sv1_2016`, which
            // is unreadable. Pin the human-friendly DSL spellings here so
            // users can write `tlsv1_2_2021` etc. instead.
            m.insert(
                "MinimumProtocolVersion",
                vec![
                    ("SSLv3", "sslv3"),
                    ("TLSv1", "tlsv1"),
                    ("TLSv1_2016", "tlsv1_2016"),
                    ("TLSv1.1_2016", "tlsv1_1_2016"),
                    ("TLSv1.2_2018", "tlsv1_2_2018"),
                    ("TLSv1.2_2019", "tlsv1_2_2019"),
                    ("TLSv1.2_2021", "tlsv1_2_2021"),
                ],
            );
            // CloudFront CustomOriginConfig.OriginSSLProtocols /
            // LegacyCustomOrigin.OriginSSLProtocols (awscc#246):
            // heck snake-cases `TLSv1.1` to `tl_sv1_1` because it splits at
            // every upper→lower boundary. Pin the natural DSL spelling so
            // users can write `tlsv1_1` etc.
            m.insert(
                "OriginSSLProtocols",
                vec![
                    ("SSLv3", "sslv3"),
                    ("TLSv1", "tlsv1"),
                    ("TLSv1.1", "tlsv1_1"),
                    ("TLSv1.2", "tlsv1_2"),
                ],
            );
            m
        });
    &ALIASES
}

/// Build an `EnumInfo` for a curated nested-field override
/// (`resource_type_overrides()` Enum entries; awscc#246).
///
/// Mirrors the logic at the `known_enum_overrides` call site: keep the
/// canonical AWS values in `EnumInfo::values`. DSL spellings are emitted
/// separately through `dsl_aliases_code`.
fn enum_info_for_override(field_name: &str, values: &[&'static str]) -> EnumInfo {
    EnumInfo {
        type_name: field_name.to_pascal_case(),
        values: values.iter().map(|s| s.to_string()).collect(),
    }
}

/// Generate condition string and display string for integer range validation.
fn int_range_condition_and_display(min: Option<i64>, max: Option<i64>) -> (String, String) {
    match (min, max) {
        (Some(min), Some(max)) => (
            format!("*n < {} || *n > {}", min, max),
            format!("{}..={}", min, max),
        ),
        (Some(min), None) => (format!("*n < {}", min), format!("{}..", min)),
        (None, Some(max)) => (format!("*n > {}", max), format!("..={}", max)),
        (None, None) => unreachable!("at least one bound must be present"),
    }
}

/// Generate condition string and display string for float range validation.
/// Generate condition string and display string for list item count validation.
fn list_items_condition_and_display(min: Option<i64>, max: Option<i64>) -> (String, String) {
    match (min, max) {
        (Some(min), Some(max)) => (
            format!("!({}..={}).contains(&len)", min, max),
            format!("{}..={}", min, max),
        ),
        (Some(min), None) => (format!("len < {}", min), format!("{}..", min)),
        (None, Some(max)) => (format!("len > {}", max), format!("..={}", max)),
        (None, None) => unreachable!("at least one bound must be present"),
    }
}

fn float_range_condition_and_display(min: Option<i64>, max: Option<i64>) -> (String, String) {
    match (min, max) {
        // Clippy 1.95 flags `n < x || n > y` as `manual_range_contains`;
        // emit the `!RangeInclusive::contains` form directly so generated
        // files stay `-D warnings` clean (carina#3340 post-emit drain).
        (Some(min), Some(max)) => (
            format!("!({}.0..={}.0).contains(&n)", min, max),
            format!("{}..={}", min, max),
        ),
        (Some(min), None) => (format!("n < {}.0", min), format!("{}..", min)),
        (None, Some(max)) => (format!("n > {}.0", max), format!("..={}", max)),
        (None, None) => unreachable!("at least one bound must be present"),
    }
}

/// Format a range display string for type names.
fn range_display_string(min: Option<i64>, max: Option<i64>) -> String {
    match (min, max) {
        (Some(min), Some(max)) => format!("{}..={}", min, max),
        (Some(min), None) => format!("{}..", min),
        (None, Some(max)) => format!("..={}", max),
        (None, None) => unreachable!("at least one bound must be present"),
    }
}

/// Check if a regex pattern represents a numeric string (digits only).
/// Matches patterns like "[0-9]+", "^[0-9]+$", "^\d+$", etc.
fn is_numeric_string_pattern(pattern: &str) -> bool {
    matches!(
        pattern,
        "[0-9]+" | "^[0-9]+$" | "\\d+" | "^\\d+$" | "[0-9]*" | "^[0-9]*$"
    )
}

/// Detect a regex pattern that is a pure alternation of literal values
/// and return those literals in source order.
///
/// Recognized shapes (anchors required):
///   - `^(a|b|c)$`  -> Some(vec!["a", "b", "c"])
///   - `^(only)$`   -> Some(vec!["only"])
///   - `^only$`     -> Some(vec!["only"])
///
/// An "alternative" must be a literal: ASCII letters, digits, or any of
/// the few separator characters that real CFN enum-as-pattern values use
/// (`-`, `_`, `:`, `.`, `/`). Any regex metacharacter (`[`, `]`, `(`,
/// `)`, `*`, `+`, `?`, `\`, `{`, `}`, `^`, `$`, `|` inside the alt) or
/// an empty alternative disqualifies the pattern; it then flows through
/// the existing `Custom { pattern }` path. See awscc#245.
fn extract_enum_from_alternation_pattern(pattern: &str) -> Option<Vec<String>> {
    let inner = pattern
        .strip_prefix('^')
        .and_then(|s| s.strip_suffix('$'))?;
    // Two recognized shapes:
    //   1. `^(alt1|alt2|...)$` — one outer paren group, `|` only inside.
    //   2. `^literal$`         — no parens, no `|`.
    // Anything else (nested groups, top-level alternation without parens
    // like `^a|b$` which regex-parses as `(^a)|(b$)`, regex meta inside
    // alts, etc.) flows through the existing `Custom { pattern }` path.
    let alternatives: Vec<&str> = if let Some(stripped) =
        inner.strip_prefix('(').and_then(|s| s.strip_suffix(')'))
    {
        if stripped.is_empty() || stripped.contains('(') || stripped.contains(')') {
            return None;
        }
        stripped.split('|').collect()
    } else {
        if inner.is_empty() || inner.contains('(') || inner.contains(')') || inner.contains('|') {
            return None;
        }
        vec![inner]
    };
    if alternatives.iter().any(|alt| !is_literal_enum_value(alt)) {
        return None;
    }
    Some(alternatives.into_iter().map(|s| s.to_string()).collect())
}

/// True when `s` is a non-empty literal value safe to treat as an enum
/// member: ASCII letters, digits, or one of `-`, `_`, `:`, `.`, `/`.
fn is_literal_enum_value(s: &str) -> bool {
    !s.is_empty()
        && s.chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | ':' | '.' | '/'))
}

/// Recursively collect string length constraints from a property and its array items.
/// Treats minLength=0 as no constraint since usize is always >= 0.
fn collect_string_length_constraints(
    prop_name: &str,
    prop: &CfnProperty,
    ranged_strings: &mut BTreeMap<String, (Option<u64>, Option<u64>)>,
) {
    let prop_type = prop.prop_type.as_ref().and_then(|t| t.as_str());
    let effective_min = prop.min_length.filter(|&m| m > 0);
    if prop_type == Some("string")
        && (effective_min.is_some() || prop.max_length.is_some())
        && prop.enum_values.is_none()
        && prop.pattern.is_none()
        && !ranged_strings.contains_key(prop_name)
    {
        ranged_strings.insert(prop_name.to_string(), (effective_min, prop.max_length));
    }
    // Recurse into array items
    if prop_type == Some("array")
        && let Some(items) = &prop.items
    {
        collect_string_length_constraints(prop_name, items, ranged_strings);
    }
}

/// Format a range display string for string length type names.
fn string_length_display(min: Option<u64>, max: Option<u64>) -> String {
    match (min, max) {
        (Some(min), Some(max)) => format!("{}..={}", min, max),
        (Some(min), None) => format!("{}..", min),
        (None, Some(max)) => format!("..={}", max),
        (None, None) => unreachable!("at least one bound must be present"),
    }
}

/// Generate a constraint-based function name for string length validation.
/// E.g., `validate_string_length_max_1024` or `validate_string_length_1_512`.
fn string_length_fn_name(min: Option<u64>, max: Option<u64>) -> String {
    let effective_min = min.filter(|&m| m > 0);
    match (effective_min, max) {
        (Some(min), Some(max)) => format!("validate_string_length_{min}_{max}"),
        (Some(min), None) => format!("validate_string_length_min_{min}"),
        (None, Some(max)) => format!("validate_string_length_max_{max}"),
        (None, None) => unreachable!("at least one bound must be present"),
    }
}

/// Generate a constraint-based function name for list items validation.
/// E.g., `validate_list_items_1_100` or `validate_list_items_max_10`.
fn list_items_fn_name(min: Option<i64>, max: Option<i64>) -> String {
    match (min, max) {
        (Some(min), Some(max)) => format!("validate_list_items_{min}_{max}"),
        (Some(min), None) => format!("validate_list_items_min_{min}"),
        (None, Some(max)) => format!("validate_list_items_max_{max}"),
        (None, None) => unreachable!("at least one bound must be present"),
    }
}

/// Generate a constraint-based function name for pattern validation.
/// Uses a hash of the pattern to create a unique but deterministic name.
fn pattern_fn_name(pattern: &str) -> String {
    use std::hash::{DefaultHasher, Hash, Hasher};
    let mut hasher = DefaultHasher::new();
    pattern.hash(&mut hasher);
    let hash = hasher.finish();
    format!("validate_string_pattern_{:016x}", hash)
}

/// Emit a Rust expression for `Option<String>` pattern field. Escapes `"` and `\`
/// so the generated source compiles; matching/validation logic itself stays in
/// the `validate` closure.
fn emit_pattern_option(pattern: Option<&str>) -> String {
    match pattern {
        Some(p) => {
            let escaped = p.replace('\\', "\\\\").replace('"', "\\\"");
            format!("Some(\"{}\".to_string())", escaped) // rust-lit-guard: allow (already escaped; pattern text is not escape_default-normalized)
        }
        None => "None".to_string(),
    }
}

/// Emit a Rust expression for the `length` field on `AttributeType::Custom`
/// (i.e. `Option<(Option<u64>, Option<u64>)>`).
fn emit_length_option(min: Option<u64>, max: Option<u64>) -> String {
    let effective_min = min.filter(|&m| m > 0);
    if effective_min.is_none() && max.is_none() {
        return "None".to_string();
    }
    let min_str = match effective_min {
        Some(m) => format!("Some({})", m),
        None => "None".to_string(),
    };
    let max_str = match max {
        Some(m) => format!("Some({})", m),
        None => "None".to_string(),
    };
    format!("Some(({}, {}))", min_str, max_str)
}

fn refined_string_type(identity: &str, pattern: &str, length: &str, to_dsl: &str) -> String {
    format!("AttributeType::refined_string({identity}, {pattern}, {length}, {to_dsl})")
}

fn refined_int_type(identity: &str, range: &str) -> String {
    format!("AttributeType::refined_int({identity}, {range})")
}

fn refined_int_type_with_validator(identity: &str, range: &str, validator: &str) -> String {
    format!(
        "AttributeType::refined_int_with_validator({identity}, {range}, legacy_validator({validator}))"
    )
}

fn refined_float_type(identity: &str, range: &str) -> String {
    format!("AttributeType::refined_float({identity}, {range})")
}

fn refined_list_type(element_type: &str, ordered: bool, length: &str, validator: &str) -> String {
    format!(
        "AttributeType::refined_list({element_type}, {ordered}, {length}, legacy_validator({validator}))"
    )
}

fn emit_int_range_option(min: Option<i64>, max: Option<i64>) -> String {
    let min_str = match min {
        Some(m) => format!("Some({m})"),
        None => "None".to_string(),
    };
    let max_str = match max {
        Some(m) => format!("Some({m})"),
        None => "None".to_string(),
    };
    format!("Some(({min_str}, {max_str}))")
}

fn emit_float_range_option(min: Option<i64>, max: Option<i64>) -> String {
    let min_str = match min {
        Some(m) => format!("Some({m}.0)"),
        None => "None".to_string(),
    };
    let max_str = match max {
        Some(m) => format!("Some({m}.0)"),
        None => "None".to_string(),
    };
    format!("Some(({min_str}, {max_str}))")
}

/// Convert a CloudFormation regex pattern into one that Rust's `regex` crate can
/// compile. Rust's regex does not support PCRE lookaround (`(?=`, `(?!`, `(?<=`,
/// `(?<!`) or the `\Z` end-of-string anchor. Patterns that cannot be converted
/// fall back to the permissive `.*` so the generated validator still compiles
/// and the length constraint (if any) still runs.
fn rust_compatible_pattern(pattern: &str) -> String {
    // Cheap fixups first: convert PCRE-only anchors that have Rust equivalents.
    let candidate = pattern.replace("\\Z", "\\z");
    if regex::Regex::new(&candidate).is_ok() {
        return candidate;
    }
    // Pattern uses a feature Rust's regex crate doesn't support (typically
    // lookaround). Fall back to match-anything so the generated code compiles.
    ".*".to_string()
}

/// Generate a constraint-based function name for combined pattern + length validation.
/// Uses a hash of the pattern combined with length bounds.
fn pattern_and_length_fn_name(pattern: &str, min: Option<u64>, max: Option<u64>) -> String {
    use std::hash::{DefaultHasher, Hash, Hasher};
    let mut hasher = DefaultHasher::new();
    pattern.hash(&mut hasher);
    let hash = hasher.finish();
    let effective_min = min.filter(|&m| m > 0);
    match (effective_min, max) {
        (Some(min_val), Some(max_val)) => {
            format!(
                "validate_string_pattern_{:016x}_len_{min_val}_{max_val}",
                hash
            )
        }
        (Some(min_val), None) => {
            format!("validate_string_pattern_{:016x}_len_min_{min_val}", hash)
        }
        (None, Some(max_val)) => {
            format!("validate_string_pattern_{:016x}_len_max_{max_val}", hash)
        }
        (None, None) => pattern_fn_name(pattern),
    }
}

/// Generate condition string and display string for string length validation.
/// Treats min=0 as no minimum since `usize` is always >= 0.
fn string_length_condition_and_display(min: Option<u64>, max: Option<u64>) -> (String, String) {
    let effective_min = min.filter(|&m| m > 0);
    match (effective_min, max) {
        (Some(min), Some(max)) => (
            format!("!({}..={}).contains(&len)", min, max),
            format!("{}..={}", min, max),
        ),
        (Some(min), None) => (format!("len < {}", min), format!("{}..", min)),
        (None, Some(max)) => (format!("len > {}", max), format!("..={}", max)),
        (None, None) => unreachable!("at least one bound must be present"),
    }
}

/// Known integer range overrides for properties where CloudFormation schemas
/// don't include min/max constraints but the ranges are well-known.
fn known_int_range_overrides() -> &'static HashMap<&'static str, (i64, i64)> {
    static OVERRIDES: LazyLock<HashMap<&'static str, (i64, i64)>> = LazyLock::new(|| {
        let mut m = HashMap::new();
        m.insert("Ipv4NetmaskLength", (0, 32));
        m.insert("Ipv6NetmaskLength", (0, 128));
        m.insert("FromPort", (-1, 65535));
        m.insert("ToPort", (-1, 65535));
        m.insert("MaxSessionDuration", (3600, 43200));
        m
    });
    &OVERRIDES
}

/// Known string type overrides for properties where the CloudFormation type is
/// plain "string" but should use a more specific type.
fn known_string_type_overrides() -> &'static HashMap<&'static str, &'static str> {
    static OVERRIDES: LazyLock<HashMap<&'static str, &'static str>> = LazyLock::new(|| {
        let mut m = HashMap::new();
        m.insert(
            "DefaultSecurityGroup",
            "carina_aws_types::security_group_id()",
        );
        m.insert("DefaultNetworkAcl", "carina_aws_types::network_acl_id()");
        m.insert("DeliverCrossAccountRole", "super::super::iam::role::arn()");
        m.insert("DeliverLogsPermissionArn", "super::super::iam::role::arn()");
        m.insert("PeerRoleArn", "super::super::iam::role::arn()");
        m.insert("PermissionsBoundary", "super::super::iam::policy::arn()");
        m.insert("ManagedPolicyArns", "super::super::iam::policy::arn()");
        m.insert("KmsKeyId", "super::super::kms::key::arn()");
        m.insert("KMSMasterKeyID", "carina_aws_types::kms_key_id()");
        m.insert("ReplicaKmsKeyID", "carina_aws_types::kms_key_id()");
        m.insert("KmsKeyArn", "super::super::kms::key::arn()");
        m.insert("IpamId", "carina_aws_types::ipam_id()");
        m.insert("Locale", "carina_aws_types::aws_region()");
        m.insert("BucketAccountId", "carina_aws_types::aws_account_id()");
        m
    });
    &OVERRIDES
}

/// Unified resource-specific property type overrides.
/// Maps (CloudFormation type name, property name) to a TypeOverride.
/// Use this when a property needs resource-specific type treatment that differs
/// from global overrides or pattern-based inference.
///
/// Two-layer system: global overrides (known_*_overrides) apply by property name,
/// resource-specific overrides here take precedence.
fn resource_type_overrides() -> &'static HashMap<(&'static str, &'static str), TypeOverride> {
    static OVERRIDES: LazyLock<HashMap<(&'static str, &'static str), TypeOverride>> =
        LazyLock::new(|| {
            let mut m = HashMap::new();

            // === StringType overrides ===

            // IAM Role's Arn is always an IAM Role ARN
            m.insert(
                ("AWS::IAM::Role", "Arn"),
                TypeOverride::StringType("super::super::iam::role::arn()"),
            );
            // IAM OIDCProvider's Arn is always an IAM OIDC Provider ARN
            m.insert(
                ("AWS::IAM::OIDCProvider", "Arn"),
                TypeOverride::StringType("super::super::iam::oidc_provider::arn()"),
            );
            // IAM Role's RoleId uses AROA prefix pattern
            m.insert(
                ("AWS::IAM::Role", "RoleId"),
                TypeOverride::StringType("carina_aws_types::iam_role_id()"),
            );
            // KMS KeyPolicy is an IAM policy document, though CFN types it object|string.
            m.insert(
                ("AWS::KMS::Key", "KeyPolicy"),
                TypeOverride::StringType("carina_aws_types::iam_policy_document()"),
            );
            // EC2 Route's GatewayId accepts both igw-* and vgw-*
            m.insert(
                ("AWS::EC2::Route", "GatewayId"),
                TypeOverride::StringType("carina_aws_types::gateway_id()"),
            );
            // Generic "Id" attributes on resources where the specific ID type is known
            m.insert(
                ("AWS::EC2::EgressOnlyInternetGateway", "Id"),
                TypeOverride::StringType("carina_aws_types::egress_only_internet_gateway_id()"),
            );
            m.insert(
                ("AWS::EC2::TransitGateway", "Id"),
                TypeOverride::StringType("carina_aws_types::transit_gateway_id()"),
            );
            m.insert(
                ("AWS::EC2::VPCPeeringConnection", "Id"),
                TypeOverride::StringType("carina_aws_types::vpc_peering_connection_id()"),
            );
            m.insert(
                ("AWS::EC2::VPCEndpoint", "Id"),
                TypeOverride::StringType("carina_aws_types::vpc_endpoint_id()"),
            );
            m.insert(
                ("AWS::EC2::SecurityGroup", "Id"),
                TypeOverride::StringType("carina_aws_types::security_group_id()"),
            );
            m.insert(
                ("AWS::EC2::TransitGatewayAttachment", "Id"),
                TypeOverride::StringType("carina_aws_types::transit_gateway_attachment_id()"),
            );
            m.insert(
                ("AWS::EC2::FlowLog", "Id"),
                TypeOverride::StringType("carina_aws_types::flow_log_id()"),
            );
            m.insert(
                ("AWS::EC2::SubnetRouteTableAssociation", "Id"),
                TypeOverride::StringType("carina_aws_types::subnet_route_table_association_id()"),
            );
            // EIP Address and TransferAddress are IPv4 addresses
            m.insert(
                ("AWS::EC2::EIP", "Address"),
                TypeOverride::StringType("types::ipv4_address()"),
            );
            m.insert(
                ("AWS::EC2::EIP", "TransferAddress"),
                TypeOverride::StringType("types::ipv4_address()"),
            );
            // FlowLog LogDestination is an ARN (S3 bucket, CloudWatch log group, or Firehose)
            m.insert(
                ("AWS::EC2::FlowLog", "LogDestination"),
                TypeOverride::StringType("carina_aws_types::arn()"),
            );
            // S3 Bucket notification ARNs
            m.insert(
                ("AWS::S3::Bucket", "Function"),
                TypeOverride::StringType("carina_aws_types::arn()"),
            );
            m.insert(
                ("AWS::S3::Bucket", "Queue"),
                TypeOverride::StringType("carina_aws_types::arn()"),
            );
            m.insert(
                ("AWS::S3::Bucket", "Topic"),
                TypeOverride::StringType("carina_aws_types::arn()"),
            );
            // S3 Bucket replication role is an IAM Role ARN
            m.insert(
                ("AWS::S3::Bucket", "Role"),
                TypeOverride::StringType("super::super::iam::role::arn()"),
            );
            // S3 Bucket replication destination account
            m.insert(
                ("AWS::S3::Bucket", "Account"),
                TypeOverride::StringType("carina_aws_types::aws_account_id()"),
            );
            // VPC CidrBlockAssociations are association IDs (vpc-cidr-assoc-xxx), not CIDRs
            m.insert(
                ("AWS::EC2::VPC", "CidrBlockAssociations"),
                TypeOverride::StringType("carina_aws_types::vpc_cidr_block_association_id()"),
            );
            // Transit Gateway route table IDs use tgw-rtb- prefix, not rtb-
            m.insert(
                ("AWS::EC2::TransitGateway", "AssociationDefaultRouteTableId"),
                TypeOverride::StringType("carina_aws_types::tgw_route_table_id()"),
            );
            m.insert(
                ("AWS::EC2::TransitGateway", "PropagationDefaultRouteTableId"),
                TypeOverride::StringType("carina_aws_types::tgw_route_table_id()"),
            );
            // Transit Gateway CIDR blocks support both IPv4 and IPv6
            m.insert(
                ("AWS::EC2::TransitGateway", "TransitGatewayCidrBlocks"),
                TypeOverride::StringType("types::cidr()"),
            );
            // IPAM Pool provisioned CIDRs support both IPv4 and IPv6
            m.insert(
                ("AWS::EC2::IPAMPool", "Cidr"),
                TypeOverride::StringType("types::cidr()"),
            );
            // S3 ReplaceKeyPrefixWith is a free-form string, not an enum
            m.insert(
                ("AWS::S3::Bucket", "ReplaceKeyPrefixWith"),
                TypeOverride::StringType("AttributeType::string()"),
            );

            // SSO / IdentityStore identity semantics
            //
            // Sinks (consumers in assignments, permission sets):
            m.insert(
                ("AWS::SSO::Assignment", "TargetId"),
                TypeOverride::StringType("carina_aws_types::aws_account_id()"),
            );
            m.insert(
                ("AWS::SSO::Assignment", "PrincipalId"),
                TypeOverride::StringType("carina_aws_types::sso_principal_id()"),
            );
            m.insert(
                ("AWS::SSO::Assignment", "InstanceArn"),
                TypeOverride::StringType("carina_aws_types::sso_instance_arn()"),
            );
            m.insert(
                ("AWS::SSO::PermissionSet", "InstanceArn"),
                TypeOverride::StringType("carina_aws_types::sso_instance_arn()"),
            );
            //
            // Sources (produced by SSO/IdentityStore resources themselves).
            // Without these, the sinks above can only accept values from
            // hand-typed literals — references to the canonical producers
            // fail with "expected SsoInstanceArn, got Arn".
            m.insert(
                ("AWS::SSO::Instance", "InstanceArn"),
                TypeOverride::StringType("carina_aws_types::sso_instance_arn()"),
            );
            m.insert(
                ("AWS::SSO::Instance", "IdentityStoreId"),
                TypeOverride::StringType("carina_aws_types::identity_store_id()"),
            );
            m.insert(
                ("AWS::SSO::PermissionSet", "PermissionSetArn"),
                TypeOverride::StringType("carina_aws_types::sso_permission_set_arn()"),
            );
            m.insert(
                ("AWS::SSO::Assignment", "PermissionSetArn"),
                TypeOverride::StringType("carina_aws_types::sso_permission_set_arn()"),
            );
            m.insert(
                ("AWS::IdentityStore::Group", "GroupId"),
                TypeOverride::StringType("carina_aws_types::sso_principal_id()"),
            );
            m.insert(
                ("AWS::IdentityStore::Group", "IdentityStoreId"),
                TypeOverride::StringType("carina_aws_types::identity_store_id()"),
            );
            m.insert(
                ("AWS::IdentityStore::GroupMembership", "IdentityStoreId"),
                TypeOverride::StringType("carina_aws_types::identity_store_id()"),
            );
            m.insert(
                ("AWS::IdentityStore::GroupMembership", "GroupId"),
                TypeOverride::StringType("carina_aws_types::sso_principal_id()"),
            );

            // === Enum overrides ===

            // VPN Gateway Type only accepts "ipsec.1"
            m.insert(
                ("AWS::EC2::VPNGateway", "Type"),
                TypeOverride::Enum(vec!["ipsec.1"]),
            );

            // CloudFront Distribution: fields the CFN schema leaves as plain
            // strings but are documented as a closed value set (awscc#246).
            // Keys use "DefName.FieldName" composite form so the lookup hits
            // each nested struct field at codegen time. Source for the value
            // lists: AWS docs + Terraform AWS provider validators.
            m.insert(
                (
                    "AWS::CloudFront::Distribution",
                    "DistributionConfig.PriceClass",
                ),
                TypeOverride::Enum(vec!["PriceClass_100", "PriceClass_200", "PriceClass_All"]),
            );
            m.insert(
                (
                    "AWS::CloudFront::Distribution",
                    "DistributionConfig.HttpVersion",
                ),
                TypeOverride::Enum(vec!["http1.1", "http2", "http2and3", "http3"]),
            );
            m.insert(
                ("AWS::CloudFront::Distribution", "Cookies.Forward"),
                TypeOverride::Enum(vec!["all", "none", "whitelist"]),
            );
            // AllowedMethods / CachedMethods appear in both DefaultCacheBehavior
            // and CacheBehavior with identical semantics; register both.
            let allowed_methods = vec!["GET", "HEAD", "OPTIONS", "PUT", "PATCH", "POST", "DELETE"];
            let cached_methods = vec!["GET", "HEAD", "OPTIONS"];
            m.insert(
                (
                    "AWS::CloudFront::Distribution",
                    "DefaultCacheBehavior.AllowedMethods",
                ),
                TypeOverride::EnumUnordered(allowed_methods.clone()),
            );
            m.insert(
                (
                    "AWS::CloudFront::Distribution",
                    "CacheBehavior.AllowedMethods",
                ),
                TypeOverride::EnumUnordered(allowed_methods),
            );
            m.insert(
                (
                    "AWS::CloudFront::Distribution",
                    "DefaultCacheBehavior.CachedMethods",
                ),
                TypeOverride::EnumUnordered(cached_methods.clone()),
            );
            m.insert(
                (
                    "AWS::CloudFront::Distribution",
                    "CacheBehavior.CachedMethods",
                ),
                TypeOverride::EnumUnordered(cached_methods),
            );
            // OriginProtocolPolicy: the modern CustomOriginConfig path is
            // already CFN-enum, but the LegacyCustomOrigin definition is
            // plain string. Override there.
            m.insert(
                (
                    "AWS::CloudFront::Distribution",
                    "LegacyCustomOrigin.OriginProtocolPolicy",
                ),
                TypeOverride::Enum(vec!["http-only", "https-only", "match-viewer"]),
            );
            // OriginSSLProtocols: list-of-string in both LegacyCustomOrigin
            // and CustomOriginConfig; AWS-documented closed set.
            let ssl_protocols = vec!["SSLv3", "TLSv1", "TLSv1.1", "TLSv1.2"];
            m.insert(
                (
                    "AWS::CloudFront::Distribution",
                    "LegacyCustomOrigin.OriginSSLProtocols",
                ),
                TypeOverride::Enum(ssl_protocols.clone()),
            );
            m.insert(
                (
                    "AWS::CloudFront::Distribution",
                    "CustomOriginConfig.OriginSSLProtocols",
                ),
                TypeOverride::Enum(ssl_protocols),
            );

            // === IntRange overrides ===

            // VPN Gateway and Transit Gateway ASN range
            m.insert(
                ("AWS::EC2::VPNGateway", "AmazonSideAsn"),
                TypeOverride::IntRange(1, 4294967294),
            );
            m.insert(
                ("AWS::EC2::TransitGateway", "AmazonSideAsn"),
                TypeOverride::IntRange(1, 4294967294),
            );

            // === IntEnum overrides ===

            // FlowLog max aggregation interval: 60 or 600 seconds
            m.insert(
                ("AWS::EC2::FlowLog", "MaxAggregationInterval"),
                TypeOverride::IntEnum(vec![60, 600]),
            );
            // CloudWatch Logs retention: specific allowed day counts
            m.insert(
                ("AWS::Logs::LogGroup", "RetentionInDays"),
                TypeOverride::IntEnum(vec![
                    1, 3, 5, 7, 14, 30, 60, 90, 120, 150, 180, 365, 400, 545, 731, 1096, 1827,
                    2192, 2557, 2922, 3288, 3653,
                ]),
            );

            // === ToDsl overrides ===

            // Route 53 HostedZone Name: AWS returns FQDN with trailing dot
            m.insert(
                ("AWS::Route53::HostedZone", "Name"),
                TypeOverride::ToDsl(
                    r#"Some(carina_core::schema::DslTransform::StripSuffix(".".to_string()))"#,
                ),
            );

            m
        });
    &OVERRIDES
}

/// Return the `to_dsl` code for a Custom type, checking for ToDsl overrides.
fn to_dsl_code_for(resource_type: &str, prop_name: &str) -> &'static str {
    if let Some(TypeOverride::ToDsl(code)) =
        resource_type_overrides().get(&(resource_type, prop_name))
    {
        code
    } else {
        "None"
    }
}

/// Properties that should be marked as `identity` in the schema.
/// Identity attributes are included in the anonymous resource identifier hash
/// alongside create-only attributes.
fn identity_properties() -> &'static HashSet<(&'static str, &'static str)> {
    static IDENTITY: LazyLock<HashSet<(&'static str, &'static str)>> = LazyLock::new(|| {
        let mut s = HashSet::new();
        // Route 53 RecordSet: (HostedZoneId, Name, Type) uniquely identifies a record,
        // but Type is not create-only in CloudFormation. Without identity, two records
        // with the same name/zone but different types (A vs AAAA) collide.
        s.insert(("AWS::Route53::RecordSet", "Type"));
        s
    });
    &IDENTITY
}

/// Properties that should be marked as `deferred_populate` in the schema
/// (carina#3034). The AWS API populates these *asynchronously after Create
/// returns* — chained references to them from another resource will be
/// rejected at validate time unless the user has declared a `wait` block
/// on the binding.
///
/// CloudFormation `readOnlyProperties` is a coarser bucket: it includes
/// both attributes that are echoed back synchronously (e.g. ARNs) and
/// attributes that genuinely transition state asynchronously (e.g. ACM
/// `Status`, CloudFront `DomainName`). This list narrows the second class
/// so the validate-time rule does not over-flag.
fn deferred_populate_properties() -> &'static HashSet<(&'static str, &'static str)> {
    static DEFERRED: LazyLock<HashSet<(&'static str, &'static str)>> = LazyLock::new(|| {
        let mut s = HashSet::new();
        // CloudFront Distribution: `DomainName` (the
        // `<random>.cloudfront.net` host) is computed by the service
        // after CreateDistribution returns. Downstream consumers
        // that wire this into, e.g., a route53 alias record need a
        // `wait` synchronization. (`Status` transitions InProgress →
        // Deployed asynchronously too, but CFN does not expose it
        // as a top-level property — wait predicates that need it
        // hit the Cloud Control read path directly.)
        s.insert(("AWS::CloudFront::Distribution", "DomainName"));
        s
    });
    &DEFERRED
}

/// Attributes that do **not** exist in the CloudFormation schema but that
/// the provider synthesizes on the read path so downstream `.crn`
/// references can resolve. Each entry expands into an extra
/// `AttributeSchema::new(...).read_only()` block in the generated schema
/// with no `provider_name` (the missing `provider_name` is what tells the
/// read-path mapper to skip CFN lookup and rely on synthesis instead).
///
/// Closes carina-rs/carina-provider-awscc#240 for cloudfront.Distribution.
///
/// If AWS later updates a CFN type to surface one of these attributes
/// natively, drop the entry from this list so the regular CFN-driven
/// emission takes over — otherwise the synthesis path becomes dead code
/// silently (the value AWS would return and the one we'd synthesize are
/// identical, so a regression here would not be observable).
fn synthetic_attributes() -> &'static [SyntheticAttribute] {
    static ATTRS: LazyLock<Vec<SyntheticAttribute>> = LazyLock::new(|| {
        vec![SyntheticAttribute {
            cfn_type: "AWS::CloudFront::Distribution",
            attr_name: "arn",
            attr_type: "carina_aws_types::arn()",
            description: "The ARN of the CloudFront distribution. Synthesized by the provider \
                 from the distribution id; CloudFront's CloudFormation type does not \
                 expose ARN through the Cloud Control API.",
        }]
    });
    &ATTRS
}

struct SyntheticAttribute {
    cfn_type: &'static str,
    attr_name: &'static str,
    attr_type: &'static str,
    description: &'static str,
}

/// Infer the Carina type string for a property based on its name.
/// Checks resource-specific overrides, known string type overrides, ARN patterns,
/// and resource ID patterns.
/// Returns None if no heuristic matches (caller should default to String).
fn infer_string_type(prop_name: &str, resource_type: &str) -> Option<String> {
    // Check resource-specific overrides first (StringType only for string inference)
    if let Some(TypeOverride::StringType(override_type)) =
        resource_type_overrides().get(&(resource_type, prop_name))
    {
        return Some(override_type.to_string());
    }
    // Check known string type overrides
    if let Some(&override_type) = known_string_type_overrides().get(prop_name) {
        return Some(override_type.to_string());
    }

    // Normalize plural forms for type inference
    let singular_name = if prop_name.ends_with("Ids")
        || prop_name.ends_with("ids")
        || prop_name.ends_with("Arns")
        || prop_name.ends_with("arns")
    {
        &prop_name[..prop_name.len() - 1]
    } else {
        prop_name
    };

    // Check overrides for singular form too (e.g., list items)
    if let Some(&override_type) = known_string_type_overrides().get(singular_name) {
        return Some(override_type.to_string());
    }

    let prop_lower = singular_name.to_lowercase();

    // CIDR types - differentiate IPv4 vs IPv6 based on property name
    if prop_lower.contains("cidr") {
        if prop_lower.contains("ipv6") {
            return Some("types::ipv6_cidr()".to_string());
        }
        return Some("types::ipv4_cidr()".to_string());
    }

    // IP address types (not CIDR) - e.g., PrivateIpAddress, PublicIp
    if (prop_lower.contains("ipaddress")
        || prop_lower.ends_with("ip")
        || prop_lower.contains("ipaddresses"))
        && !prop_lower.contains("count")
        && !prop_lower.contains("type")
    {
        if prop_lower.contains("ipv6") {
            return Some("types::ipv6_address()".to_string());
        }
        return Some("types::ipv4_address()".to_string());
    }

    // Availability zone (but not AvailabilityZoneId which uses AZ ID format like "use1-az1")
    if prop_lower == "availabilityzone" || prop_lower == "availabilityzones" {
        return Some("carina_aws_types::availability_zone()".to_string());
    }

    // Availability zone ID (e.g., "use1-az1", "usw2-az2")
    if prop_lower == "availabilityzoneid" || prop_lower == "availabilityzoneids" {
        return Some("carina_aws_types::availability_zone_id()".to_string());
    }

    // Region types (e.g., PeerRegion, ServiceRegion, RegionName, ResourceRegion)
    if prop_lower.ends_with("region") || prop_lower == "regionname" {
        return Some("carina_aws_types::aws_region()".to_string());
    }

    // Check ARN pattern
    if prop_lower.ends_with("arn") || prop_lower.ends_with("arns") || prop_lower.contains("_arn") {
        return Some("carina_aws_types::arn()".to_string());
    }

    // IPAM Pool IDs
    if is_ipam_pool_id_property(singular_name) {
        return Some("carina_aws_types::ipam_pool_id()".to_string());
    }

    // Check resource ID pattern
    if is_aws_resource_id_property(singular_name, Some(resource_type)) {
        return Some(get_resource_id_type(singular_name, Some(resource_type)).to_string());
    }

    // AWS Account ID (owner IDs and account IDs are 12-digit account IDs)
    if prop_lower.ends_with("ownerid") || prop_lower.ends_with("accountid") {
        return Some("carina_aws_types::aws_account_id()".to_string());
    }

    // Email addresses. Conservative match: property name is exactly "Email"
    // or ends with "EmailAddress". Avoids false positives on names like
    // "EmailEnabled" or "PrimaryEmailType" that are flags/categories rather
    // than email addresses.
    if is_email_property(prop_name) {
        return Some("types::email()".to_string());
    }

    None
}

/// Check if a property name represents an email address.
/// Conservative: matches `Email` exactly or any name ending in `EmailAddress`
/// (case-insensitive). Used to map CFN string properties to `types::email()`.
fn is_email_property(prop_name: &str) -> bool {
    let lower = prop_name.to_lowercase();
    lower == "email" || lower.ends_with("emailaddress")
}

/// Check if a property name represents an AWS resource ID with the standard
/// prefix-hex format (e.g., vpc-1a2b3c4d, subnet-0123456789abcdef0).
///
/// `resource_type` narrows the check for property names that are ambiguous
/// without context. Notably, bare `GroupId` only counts as an AWS resource
/// ID on EC2 security-group resources; on unrelated resources (e.g.,
/// `AWS::IdentityStore::Group`) the same name refers to a different ID
/// format and must stay generic.
fn is_aws_resource_id_property(prop_name: &str, resource_type: Option<&str>) -> bool {
    let lower = prop_name.to_lowercase();
    // Known resource ID suffixes that use prefix-hex format
    let resource_id_suffixes = [
        "vpcid",
        "subnetid",
        "securitygroupid",
        "gatewayid",
        "routetableid",
        "allocationid",
        "networkinterfaceid",
        "instanceid",
        "endpointid",
        "connectionid",
        "prefixlistid",
        "eniid",
    ];
    // Exclude properties that don't follow prefix-hex format
    if lower.contains("owner") || lower.contains("availabilityzone") || lower == "resourceid" {
        return false;
    }
    // Strip trailing "s" for plural forms (e.g., "RouteTableIds" -> "routetableid")
    let singular = if lower.ends_with("ids") {
        &lower[..lower.len() - 1]
    } else {
        &lower
    };
    if resource_id_suffixes
        .iter()
        .any(|suffix| lower.ends_with(suffix) || singular.ends_with(suffix))
    {
        return true;
    }
    // Bare `GroupId`/`groupId` on EC2 security-group resources. `classify_resource_id`
    // narrows this further; here we just admit the property into the ID pipeline.
    if (lower == "groupid")
        && matches!(
            resource_type,
            Some("AWS::EC2::SecurityGroup")
                | Some("AWS::EC2::SecurityGroupIngress")
                | Some("AWS::EC2::SecurityGroupEgress")
        )
    {
        return true;
    }
    false
}

/// Classification of AWS resource ID types.
/// Used to derive both the type function name and display name from a single
/// matching logic, avoiding duplication (see #243).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ResourceIdKind {
    VpcId,
    SubnetId,
    SecurityGroupId,
    EgressOnlyInternetGatewayId,
    InternetGatewayId,
    RouteTableId,
    NatGatewayId,
    VpcPeeringConnectionId,
    TransitGatewayId,
    VpnGatewayId,
    VpcEndpointId,
    InstanceId,
    NetworkInterfaceId,
    AllocationId,
    PrefixListId,
    CarrierGatewayId,
    LocalGatewayId,
    NetworkAclId,
    Generic,
}

/// Classify a property name into a specific resource ID kind.
/// The matching order matters: more specific patterns (e.g., EgressOnlyInternetGateway)
/// must be checked before more general ones (e.g., InternetGateway).
///
/// `resource_type` is the CloudFormation type name (e.g., `AWS::EC2::SecurityGroup`) of the
/// resource the property belongs to. It's used for a narrow allowlist that lets bare
/// `GroupId` on EC2 security-group resources classify as `SecurityGroupId`, without
/// affecting `GroupId` on other resources (e.g., `AWS::IdentityStore::Group`).
fn classify_resource_id(prop_name: &str, resource_type: Option<&str>) -> ResourceIdKind {
    let lower = prop_name.to_lowercase();

    // VPC IDs
    if lower.ends_with("vpcid") || lower == "vpcid" {
        return ResourceIdKind::VpcId;
    }
    // Subnet IDs
    if lower.ends_with("subnetid") || lower == "subnetid" {
        return ResourceIdKind::SubnetId;
    }
    // Security Group IDs.
    //
    // Matches either:
    // - any property name containing "securitygroup" and ending with "id"
    //   (covers DestinationSecurityGroupId, SourceSecurityGroupId, etc.),
    // - or bare `GroupId`/`groupId` on EC2 security-group resources
    //   (AWS::EC2::SecurityGroup{,Ingress,Egress}). The resource-type
    //   allowlist keeps `GroupId` on other resources — e.g., AWS::IdentityStore::Group —
    //   classified as Generic.
    if lower.contains("securitygroup") && lower.ends_with("id") {
        return ResourceIdKind::SecurityGroupId;
    }
    if (lower == "groupid")
        && matches!(
            resource_type,
            Some("AWS::EC2::SecurityGroup")
                | Some("AWS::EC2::SecurityGroupIngress")
                | Some("AWS::EC2::SecurityGroupEgress")
        )
    {
        return ResourceIdKind::SecurityGroupId;
    }
    // Egress Only Internet Gateway IDs (must be checked before Internet Gateway IDs)
    if lower.contains("egressonlyinternetgateway") && lower.ends_with("id") {
        return ResourceIdKind::EgressOnlyInternetGatewayId;
    }
    // Internet Gateway IDs
    if lower.contains("internetgateway") && lower.ends_with("id") {
        return ResourceIdKind::InternetGatewayId;
    }
    // Route Table IDs
    if lower.contains("routetable") && lower.ends_with("id") {
        return ResourceIdKind::RouteTableId;
    }
    // NAT Gateway IDs
    if lower.contains("natgateway") && lower.ends_with("id") {
        return ResourceIdKind::NatGatewayId;
    }
    // VPC Peering Connection IDs
    if lower.contains("peeringconnection") && lower.ends_with("id") {
        return ResourceIdKind::VpcPeeringConnectionId;
    }
    // Transit Gateway IDs
    if lower.contains("transitgateway") && lower.ends_with("id") {
        return ResourceIdKind::TransitGatewayId;
    }
    // VPN Gateway IDs
    if lower.contains("vpngateway") && lower.ends_with("id") {
        return ResourceIdKind::VpnGatewayId;
    }
    // VPC Endpoint IDs
    if lower.contains("vpcendpoint") && lower.ends_with("id") {
        return ResourceIdKind::VpcEndpointId;
    }
    // Instance IDs (e.g., InstanceId)
    if lower.ends_with("instanceid") {
        return ResourceIdKind::InstanceId;
    }
    // Network Interface IDs (e.g., NetworkInterfaceId, EniId)
    if lower.ends_with("networkinterfaceid") || lower.ends_with("eniid") {
        return ResourceIdKind::NetworkInterfaceId;
    }
    // Allocation IDs (e.g., AllocationId)
    if lower.ends_with("allocationid") {
        return ResourceIdKind::AllocationId;
    }
    // Prefix List IDs (e.g., PrefixListId, DestinationPrefixListId)
    if lower.ends_with("prefixlistid") {
        return ResourceIdKind::PrefixListId;
    }
    // Carrier Gateway IDs (e.g., CarrierGatewayId)
    if lower.contains("carriergateway") && lower.ends_with("id") {
        return ResourceIdKind::CarrierGatewayId;
    }
    // Local Gateway IDs (e.g., LocalGatewayId)
    if lower.contains("localgateway") && lower.ends_with("id") {
        return ResourceIdKind::LocalGatewayId;
    }
    // Network ACL IDs (e.g., NetworkAclId)
    if lower.contains("networkacl") && lower.ends_with("id") {
        return ResourceIdKind::NetworkAclId;
    }

    ResourceIdKind::Generic
}

/// Get the specific resource ID type function for a property name.
/// Returns the function name (e.g., "carina_aws_types::vpc_id()") or generic aws_resource_id.
///
/// `resource_type` is the CloudFormation type name of the enclosing resource; see
/// `classify_resource_id`.
fn get_resource_id_type(prop_name: &str, resource_type: Option<&str>) -> &'static str {
    match classify_resource_id(prop_name, resource_type) {
        ResourceIdKind::VpcId => "carina_aws_types::vpc_id()",
        ResourceIdKind::SubnetId => "carina_aws_types::subnet_id()",
        ResourceIdKind::SecurityGroupId => "carina_aws_types::security_group_id()",
        ResourceIdKind::EgressOnlyInternetGatewayId => {
            "carina_aws_types::egress_only_internet_gateway_id()"
        }
        ResourceIdKind::InternetGatewayId => "carina_aws_types::internet_gateway_id()",
        ResourceIdKind::RouteTableId => "carina_aws_types::route_table_id()",
        ResourceIdKind::NatGatewayId => "carina_aws_types::nat_gateway_id()",
        ResourceIdKind::VpcPeeringConnectionId => "carina_aws_types::vpc_peering_connection_id()",
        ResourceIdKind::TransitGatewayId => "carina_aws_types::transit_gateway_id()",
        ResourceIdKind::VpnGatewayId => "carina_aws_types::vpn_gateway_id()",
        ResourceIdKind::VpcEndpointId => "carina_aws_types::vpc_endpoint_id()",
        ResourceIdKind::InstanceId => "carina_aws_types::instance_id()",
        ResourceIdKind::NetworkInterfaceId => "carina_aws_types::network_interface_id()",
        ResourceIdKind::AllocationId => "carina_aws_types::allocation_id()",
        ResourceIdKind::PrefixListId => "carina_aws_types::prefix_list_id()",
        ResourceIdKind::CarrierGatewayId => "carina_aws_types::carrier_gateway_id()",
        ResourceIdKind::LocalGatewayId => "carina_aws_types::local_gateway_id()",
        ResourceIdKind::NetworkAclId => "carina_aws_types::network_acl_id()",
        ResourceIdKind::Generic => "carina_aws_types::aws_resource_id()",
    }
}

/// Get the display name for a resource ID type (for markdown documentation).
fn get_resource_id_display_name(prop_name: &str, resource_type: Option<&str>) -> &'static str {
    match classify_resource_id(prop_name, resource_type) {
        ResourceIdKind::VpcId => "VpcId",
        ResourceIdKind::SubnetId => "SubnetId",
        ResourceIdKind::SecurityGroupId => "SecurityGroupId",
        ResourceIdKind::EgressOnlyInternetGatewayId => "EgressOnlyInternetGatewayId",
        ResourceIdKind::InternetGatewayId => "InternetGatewayId",
        ResourceIdKind::RouteTableId => "RouteTableId",
        ResourceIdKind::NatGatewayId => "NatGatewayId",
        ResourceIdKind::VpcPeeringConnectionId => "VpcPeeringConnectionId",
        ResourceIdKind::TransitGatewayId => "TransitGatewayId",
        ResourceIdKind::VpnGatewayId => "VpnGatewayId",
        ResourceIdKind::VpcEndpointId => "VpcEndpointId",
        ResourceIdKind::InstanceId => "InstanceId",
        ResourceIdKind::NetworkInterfaceId => "NetworkInterfaceId",
        ResourceIdKind::AllocationId => "AllocationId",
        ResourceIdKind::PrefixListId => "PrefixListId",
        ResourceIdKind::CarrierGatewayId => "CarrierGatewayId",
        ResourceIdKind::LocalGatewayId => "LocalGatewayId",
        ResourceIdKind::NetworkAclId => "NetworkAclId",
        ResourceIdKind::Generic => "AwsResourceId",
    }
}

/// Check if a property name represents an IPAM Pool ID
/// (e.g., IpamPoolId, Ipv4IpamPoolId, Ipv6IpamPoolId, SourceIpamPoolId)
fn is_ipam_pool_id_property(prop_name: &str) -> bool {
    let lower = prop_name.to_lowercase();
    // Exclude properties that don't follow prefix-hex format
    if lower.contains("owner") || lower.contains("availabilityzone") || lower == "resourceid" {
        return false;
    }
    lower.ends_with("poolid")
}

/// `true` when the resource-type override for `(type_name, key)` is
/// [`TypeOverride::EnumUnordered`], i.e. an array field that must be
/// `unordered_list` regardless of the CFN `insertionOrder`
/// (carina#3093). `key` is the property name for top-level props or
/// `"DefName.FieldName"` for nested struct fields — whichever the
/// caller used to look the override up.
fn override_forces_unordered(type_name: &str, key: &str) -> bool {
    matches!(
        resource_type_overrides().get(&(type_name, key)),
        Some(TypeOverride::EnumUnordered(_))
    )
}

/// Return the correct list constructor based on insertionOrder.
/// CloudFormation defaults insertionOrder to true when not specified.
///
/// `force_unordered` lets a `TypeOverride::EnumUnordered` declare a
/// field order-insensitive even when the CFN schema omits
/// `insertionOrder: false` (carina#3093). It only ever forces
/// *unordered*: a CFN-declared `insertionOrder: false` already yields
/// unordered, and forcing has no inverse (we never coerce an
/// order-sensitive list to ordered against the schema).
fn list_constructor(insertion_order: Option<bool>, force_unordered: bool) -> &'static str {
    if force_unordered || insertion_order == Some(false) {
        "AttributeType::unordered_list"
    } else {
        "AttributeType::list"
    }
}

/// Returns (type_string, Option<EnumInfo>)
/// EnumInfo is Some if this property is an enum that should use AttributeType::Custom
fn cfn_type_to_carina_type_with_enum(
    prop: &CfnProperty,
    prop_name: &str,
    schema: &CfnSchema,
    namespace: &str,
    enums: &BTreeMap<String, EnumInfo>,
) -> (String, Option<EnumInfo>) {
    cfn_type_to_carina_type_with_enum_with_struct_path(
        prop,
        prop_name,
        schema,
        namespace,
        enums,
        &[],
    )
}

fn property_type_values(prop: &CfnProperty) -> Vec<String> {
    let mut values = match &prop.prop_type {
        Some(TypeValue::Single(value)) => vec![value.clone()],
        Some(TypeValue::Multiple(values)) => values.clone(),
        None => Vec::new(),
    };

    if let Some(ref_path) = &prop.ref_path {
        values.push(format!("$ref:{ref_path}"));
    }

    values
}

fn property_resolves_to_array(prop: &CfnProperty, schema: &CfnSchema) -> bool {
    if property_type_values(prop).iter().any(|ty| ty == "array") {
        return true;
    }

    prop.ref_path
        .as_deref()
        .and_then(|ref_path| resolve_ref_classified(schema, ref_path))
        .is_some_and(|resolved| matches!(resolved, ResolvedDef::Array { .. }))
}

fn cfn_type_to_carina_type_with_enum_with_struct_path(
    prop: &CfnProperty,
    prop_name: &str,
    schema: &CfnSchema,
    namespace: &str,
    enums: &BTreeMap<String, EnumInfo>,
    struct_path: &[String],
) -> (String, Option<EnumInfo>) {
    // Tags property is special - it's a Map in Carina (Terraform-style)
    if prop_name == "Tags" {
        return ("carina_aws_types::tags_type()".to_string(), None);
    }

    // Handle const values - generate single-value enum
    if let Some(const_val) = &prop.const_value {
        let value_str = match const_val {
            serde_json::Value::String(s) => s.clone(),
            other => other.to_string(),
        };
        let type_name = prop_name.to_pascal_case();
        let enum_info = EnumInfo {
            type_name,
            values: vec![value_str],
        };
        return ("/* enum */".to_string(), Some(enum_info));
    }

    if !prop.one_of.is_empty() {
        if let Some(array_variant) = prop
            .one_of
            .iter()
            .find(|variant| property_resolves_to_array(variant, schema))
        {
            return cfn_type_to_carina_type_with_enum_with_struct_path(
                array_variant,
                prop_name,
                schema,
                namespace,
                enums,
                struct_path,
            );
        }

        let has_mergeable_object_variant = prop.one_of.iter().any(|variant| {
            variant
                .properties
                .as_ref()
                .is_some_and(|props| !props.is_empty())
        });
        if !has_mergeable_object_variant {
            let variant_types: Vec<Vec<String>> =
                prop.one_of.iter().map(property_type_values).collect();
            panic!(
                "unresolved oneOf for {}.{}: variants {:?}",
                schema.type_name, prop_name, variant_types
            );
        }
    }

    // Handle $ref
    if let Some(ref_path) = &prop.ref_path {
        if ref_path.contains("/Tag") {
            return ("carina_aws_types::tags_type()".to_string(), None);
        }

        if let Some(def_name) = ref_def_name(ref_path) {
            let already_known = IN_PROGRESS_DEFS.with(|s| s.borrow().contains(def_name))
                || EMITTED_DEFS.with(|s| s.borrow().contains_key(def_name));
            if already_known {
                return (
                    format!("AttributeType::ref_({}.to_string())", rust_lit(def_name)),
                    None,
                );
            }
        }

        if let Some(resolved) = resolve_ref_classified(schema, ref_path) {
            match resolved {
                ResolvedDef::Struct {
                    properties,
                    required,
                } => {
                    let def_name = ref_def_name(ref_path).unwrap_or(prop_name);
                    IN_PROGRESS_DEFS.with(|s| {
                        s.borrow_mut().insert(def_name.to_string());
                    });
                    let body = generate_struct_type(
                        def_name,
                        properties,
                        required,
                        schema,
                        namespace,
                        enums,
                        struct_path,
                    );
                    IN_PROGRESS_DEFS.with(|s| {
                        s.borrow_mut().remove(def_name);
                    });
                    EMITTED_DEFS.with(|s| {
                        s.borrow_mut().insert(def_name.to_string(), body.clone());
                    });
                    return (body, None);
                }
                ResolvedDef::OneOf { variants } => {
                    let mut merged_props = BTreeMap::new();
                    for variant in variants {
                        if let Some(props) = &variant.properties {
                            for (k, v) in props {
                                if is_deprecated_property(v) {
                                    continue;
                                }
                                merged_props.insert(k.clone(), v.clone());
                            }
                        }
                    }
                    if !merged_props.is_empty() {
                        let def_name = ref_def_name(ref_path).unwrap_or(prop_name);
                        IN_PROGRESS_DEFS.with(|s| {
                            s.borrow_mut().insert(def_name.to_string());
                        });
                        let body = generate_struct_type(
                            def_name,
                            &merged_props,
                            &[],
                            schema,
                            namespace,
                            enums,
                            struct_path,
                        );
                        IN_PROGRESS_DEFS.with(|s| {
                            s.borrow_mut().remove(def_name);
                        });
                        EMITTED_DEFS.with(|s| {
                            s.borrow_mut().insert(def_name.to_string(), body.clone());
                        });
                        return (body, None);
                    }
                }
                ResolvedDef::Enum { values } => {
                    let def_name = ref_def_name(ref_path).unwrap_or(prop_name);
                    let type_name = def_name.to_pascal_case();
                    let string_values: Vec<String> =
                        values.iter().map(|v| v.to_string_value()).collect();
                    let deduped =
                        deduplicate_enum_values(string_values.clone()).unwrap_or(string_values);
                    let enum_info = EnumInfo {
                        type_name,
                        values: deduped,
                    };
                    return ("/* enum */".to_string(), Some(enum_info));
                }
                ResolvedDef::Array { items } => {
                    let (item_type, item_enum) = cfn_type_to_carina_type_with_enum_with_struct_path(
                        items,
                        prop_name,
                        schema,
                        namespace,
                        enums,
                        struct_path,
                    );
                    let effective_item_type = if item_enum.is_some() {
                        "AttributeType::string()".to_string()
                    } else {
                        item_type
                    };
                    let list_ctor = list_constructor(
                        prop.insertion_order,
                        override_forces_unordered(schema.type_name.as_str(), prop_name),
                    );
                    return (format!("{}({})", list_ctor, effective_item_type), item_enum);
                }
                ResolvedDef::StringPattern {
                    pattern,
                    min_length,
                    max_length,
                } => {
                    if infer_string_type(prop_name, &schema.type_name).is_none() {
                        if let Some(values) = extract_enum_from_alternation_pattern(pattern) {
                            let type_name = prop_name.to_pascal_case();
                            let enum_info = EnumInfo { type_name, values };
                            return ("/* enum */".to_string(), Some(enum_info));
                        }
                        let effective_min = min_length.filter(|&m| m > 0);
                        let has_length = effective_min.is_some() || max_length.is_some();
                        if has_length {
                            record_pattern_with_length(pattern, effective_min, max_length);
                        } else {
                            record_standalone_pattern(pattern);
                        };
                        let pattern_expr = emit_pattern_option(Some(pattern));
                        let length_expr = emit_length_option(effective_min, max_length);
                        return (
                            refined_string_type("None", &pattern_expr, &length_expr, "None"),
                            None,
                        );
                    }
                }
                scalar @ ResolvedDef::Scalar { .. } => {
                    if let Some(scalar_prop) = scalar.scalar_as_property() {
                        return cfn_type_to_carina_type_with_enum_with_struct_path(
                            &scalar_prop,
                            prop_name,
                            schema,
                            namespace,
                            enums,
                            struct_path,
                        );
                    }
                }
                ResolvedDef::Opaque => {}
            }
        }
        if let Some(inferred) = infer_string_type(prop_name, &schema.type_name) {
            return (inferred, None);
        }
        return ("AttributeType::string()".to_string(), None);
    }

    // Handle explicit enum
    if let Some(enum_values) = &prop.enum_values {
        // If all enum values are integers, generate IntEnum Custom type
        let all_ints = enum_values.iter().all(|v| matches!(v, EnumValue::Int(_)));
        if all_ints && !enum_values.is_empty() {
            let values: Vec<i64> = enum_values
                .iter()
                .filter_map(|v| match v {
                    EnumValue::Int(i) => Some(*i),
                    _ => None,
                })
                .collect();
            record_int_enum(prop_name, &values);
            let _ = values;
            let validate_fn = format!("validate_{}_int_enum", prop_name.to_snake_case());
            return (
                refined_int_type_with_validator("None", "None", &validate_fn),
                None,
            );
        }

        let type_name = prop_name.to_pascal_case();
        let string_values: Vec<String> = enum_values.iter().map(|v| v.to_string_value()).collect();
        // Deduplicate case-insensitive duplicates (e.g., "GLACIER" and "Glacier")
        let deduped = deduplicate_enum_values(string_values.clone()).unwrap_or(string_values);
        let enum_info = EnumInfo {
            type_name,
            values: deduped,
        };
        // Return placeholder - actual type will be generated using enum_info
        return ("/* enum */".to_string(), Some(enum_info));
    }

    // Check known enum overrides (for properties with inconsistent description formatting)
    let overrides = known_enum_overrides();
    if let Some(values) = overrides.get(prop_name) {
        let type_name = prop_name.to_pascal_case();
        let enum_info = EnumInfo {
            type_name,
            values: values.iter().map(|s| s.to_string()).collect(),
        };
        return ("/* enum */".to_string(), Some(enum_info));
    }

    // Handle type
    match prop.prop_type.as_ref().and_then(|t| t.as_str()) {
        Some("string") => {
            if let Some(TypeOverride::ToDsl(to_dsl)) =
                resource_type_overrides().get(&(schema.type_name.as_str(), prop_name))
            {
                let length_expr =
                    emit_length_option(prop.min_length.filter(|&m| m > 0), prop.max_length);
                return (
                    refined_string_type("None", "None", &length_expr, to_dsl),
                    None,
                );
            }

            // Check known string type overrides first (includes CIDR, IP, AZ,
            // ARN, resource IDs, IPAM Pool IDs, and owner IDs)
            if let Some(inferred) = infer_string_type(prop_name, &schema.type_name) {
                return (inferred, None);
            }

            // Check if this is a policy document field (CFN sometimes types these as "string")
            if prop_name.ends_with("PolicyDocument") {
                return ("carina_aws_types::iam_policy_document()".to_string(), None);
            }

            // Try to extract enum values from description
            if let Some(desc) = &prop.description
                && let Some(enum_values) = extract_enum_from_description(desc)
            {
                let type_name = prop_name.to_pascal_case();
                let enum_info = EnumInfo {
                    type_name,
                    values: enum_values,
                };
                // Return placeholder - actual type will be generated using enum_info
                return ("/* enum */".to_string(), Some(enum_info));
            }

            // Check for string length constraints
            let effective_min = prop.min_length.filter(|&m| m > 0);
            let has_length = effective_min.is_some() || prop.max_length.is_some();

            // Check for numeric string pattern (e.g., "[0-9]+")
            if let Some(ref pattern) = prop.pattern
                && is_numeric_string_pattern(pattern)
            {
                if has_length {
                    record_pattern_with_length(pattern, effective_min, prop.max_length);
                } else {
                    record_standalone_pattern(pattern);
                };
                let pattern_expr = emit_pattern_option(Some(pattern));
                let length_expr = emit_length_option(effective_min, prop.max_length);
                return (
                    refined_string_type("None", &pattern_expr, &length_expr, "None"),
                    None,
                );
            }

            // Check for alternation-only pattern (e.g., "^(a|b|c)$") and
            // promote it to a real StringEnum so users get bare-identifier
            // syntax, did-you-mean diagnostics, and LSP completion. See
            // awscc#245.
            if let Some(ref pattern) = prop.pattern
                && let Some(values) = extract_enum_from_alternation_pattern(pattern)
            {
                let type_name = prop_name.to_pascal_case();
                let enum_info = EnumInfo { type_name, values };
                return ("/* enum */".to_string(), Some(enum_info));
            }

            // Check for regex pattern constraint
            if let Some(pattern) = &prop.pattern {
                if has_length {
                    record_pattern_with_length(pattern, effective_min, prop.max_length);
                } else {
                    record_standalone_pattern(pattern);
                };
                let pattern_expr = emit_pattern_option(Some(pattern));
                let length_expr = emit_length_option(effective_min, prop.max_length);
                return (
                    refined_string_type("None", &pattern_expr, &length_expr, "None"),
                    None,
                );
            }

            // Check for string format constraint (e.g., "uri", "date-time")
            if prop.format.is_some() {
                return (refined_string_type("None", "None", "None", "None"), None);
            }

            // Check for string length constraints (minLength/maxLength)
            if has_length {
                let length_expr = emit_length_option(effective_min, prop.max_length);
                let to_dsl = to_dsl_code_for(&schema.type_name, prop_name);
                return (
                    refined_string_type("None", "None", &length_expr, to_dsl),
                    None,
                );
            }

            ("AttributeType::string()".to_string(), None)
        }
        Some("boolean") => ("AttributeType::bool()".to_string(), None),
        Some("integer") => {
            // Check resource-scoped overrides first
            let res_override =
                resource_type_overrides().get(&(schema.type_name.as_str(), prop_name));
            if let Some(TypeOverride::IntRange(min, max)) = res_override {
                record_range_validator(prop_name, Some(*min), Some(*max), false);
                return (
                    refined_int_type("None", &emit_int_range_option(Some(*min), Some(*max))),
                    None,
                );
            }
            if let Some(TypeOverride::IntEnum(values)) = res_override {
                record_int_enum(prop_name, values);
                let validate_fn = format!("validate_{}_int_enum", prop_name.to_snake_case());
                return (
                    refined_int_type_with_validator("None", "None", &validate_fn),
                    None,
                );
            }
            // Use CF min/max if available (including one-sided), otherwise check known overrides
            let range: Option<(Option<i64>, Option<i64>)> =
                if prop.minimum.is_some() || prop.maximum.is_some() {
                    Some((prop.minimum, prop.maximum))
                } else {
                    known_int_range_overrides()
                        .get(prop_name)
                        .map(|&(min, max)| (Some(min), Some(max)))
                };
            if let Some((min, max)) = range {
                // Generate a ranged int type with validation
                record_range_validator(prop_name, min, max, false);
                (
                    refined_int_type("None", &emit_int_range_option(min, max)),
                    None,
                )
            } else if prop.format.is_some() {
                // Format-only integer (e.g., int64) - informational, no range validation
                (refined_int_type("None", "None"), None)
            } else {
                ("AttributeType::int()".to_string(), None)
            }
        }
        Some("number") => {
            // Check resource-scoped overrides first
            let res_override =
                resource_type_overrides().get(&(schema.type_name.as_str(), prop_name));
            if let Some(TypeOverride::IntRange(min, max)) = res_override {
                record_range_validator(prop_name, Some(*min), Some(*max), true);
                return (
                    refined_float_type("None", &emit_float_range_option(Some(*min), Some(*max))),
                    None,
                );
            }
            // Use CF min/max if available (including one-sided), otherwise check known overrides
            let range: Option<(Option<i64>, Option<i64>)> =
                if prop.minimum.is_some() || prop.maximum.is_some() {
                    Some((prop.minimum, prop.maximum))
                } else {
                    known_int_range_overrides()
                        .get(prop_name)
                        .map(|&(min, max)| (Some(min), Some(max)))
                };
            if let Some((min, max)) = range {
                // Generate a ranged float type with validation
                record_range_validator(prop_name, min, max, true);
                (
                    refined_float_type("None", &emit_float_range_option(min, max)),
                    None,
                )
            } else if prop.format.is_some() {
                // Format-only float (e.g., double) - informational, no range validation
                (refined_float_type("None", "None"), None)
            } else {
                ("AttributeType::float()".to_string(), None)
            }
        }
        Some("array") => {
            let list_ctor = list_constructor(
                prop.insertion_order,
                override_forces_unordered(schema.type_name.as_str(), prop_name),
            );
            let (list_type, item_enum) = if let Some(items) = &prop.items {
                // Struct-shaped $ref items are inlined as a list-of-struct;
                // every other item shape (scalar/enum/array/string/opaque)
                // falls through to the normal recursion below.
                if let Some(ref_path) = &items.ref_path
                    && !ref_path.contains("/Tag")
                    && let Some(ResolvedDef::Struct {
                        properties,
                        required,
                    }) = resolve_ref_classified(schema, ref_path)
                {
                    let def_name = ref_def_name(ref_path).unwrap_or(prop_name);
                    let struct_type = generate_struct_type(
                        def_name,
                        properties,
                        required,
                        schema,
                        namespace,
                        enums,
                        struct_path,
                    );
                    (format!("{}({})", list_ctor, struct_type), None)
                } else {
                    let (item_type, item_enum) = cfn_type_to_carina_type_with_enum_with_struct_path(
                        items,
                        prop_name,
                        schema,
                        namespace,
                        enums,
                        struct_path,
                    );
                    // If array items have enum values, propagate the enum info so the
                    // caller can register it. The item type uses String as a placeholder;
                    // the actual enum type will be substituted when the enum is registered.
                    let effective_item_type = if item_enum.is_some() {
                        "AttributeType::string()".to_string()
                    } else {
                        item_type
                    };
                    (format!("{}({})", list_ctor, effective_item_type), item_enum)
                }
            } else {
                (format!("{}(AttributeType::string())", list_ctor), None)
            };
            // Wrap in Custom type if minItems/maxItems constraints exist
            if prop.min_items.is_some() || prop.max_items.is_some() {
                let validate_fn = list_items_fn_name(prop.min_items, prop.max_items);
                let ordered = list_ctor == "AttributeType::list";
                let inner = list_type
                    .strip_prefix(&format!("{list_ctor}("))
                    .and_then(|s| s.strip_suffix(')'))
                    .unwrap_or("AttributeType::string()");
                let length = emit_length_option(
                    prop.min_items.and_then(|v| u64::try_from(v).ok()),
                    prop.max_items.and_then(|v| u64::try_from(v).ok()),
                );
                (
                    refined_list_type(inner, ordered, &length, &validate_fn),
                    item_enum,
                )
            } else {
                (list_type, item_enum)
            }
        }
        Some("object") => {
            // Check known string type overrides first, mirroring the string branch
            // for CFN union types whose first type is object.
            let res_override =
                resource_type_overrides().get(&(schema.type_name.as_str(), prop_name));
            if let Some(TypeOverride::StringType(override_type)) = res_override {
                return (override_type.to_string(), None);
            }

            // Check if this object has inline properties -> Struct
            if let Some(props) = &prop.properties
                && !props.is_empty()
            {
                return (
                    generate_struct_type(
                        prop_name,
                        props,
                        &prop.required,
                        schema,
                        namespace,
                        enums,
                        struct_path,
                    ),
                    None,
                );
            }
            // Check if this is an IAM policy document
            if prop_name.ends_with("PolicyDocument") {
                return ("carina_aws_types::iam_policy_document()".to_string(), None);
            }
            // Empty object with no or empty properties and additionalProperties: false
            // -> empty Struct (e.g., SimplePrefix)
            if prop.additional_properties == Some(false) {
                let struct_name = prop_name.to_pascal_case();
                return (
                    format!(
                        r#"AttributeType::struct_({}.to_string(), vec![])"#,
                        rust_lit(&struct_name)
                    ),
                    None,
                );
            }
            (
                "AttributeType::map(AttributeType::string())".to_string(),
                None,
            )
        }
        _ => {
            // Fallback: apply name-based heuristics for properties with no explicit type
            if let Some(inferred) = infer_string_type(prop_name, &schema.type_name) {
                (inferred, None)
            } else {
                ("AttributeType::string()".to_string(), None)
            }
        }
    }
}

/// Convert a JSON default value to a Rust `Value::...` expression string.
/// Returns `None` for unsupported types (arrays, objects, null).
fn json_default_to_value_code(val: &serde_json::Value) -> Option<String> {
    match val {
        serde_json::Value::String(s) => {
            let escaped = s.replace('\\', "\\\\").replace('"', "\\\"");
            Some(format!(
                "Value::Concrete(ConcreteValue::String(\"{}\".to_string()))", // rust-lit-guard: allow (already escaped JSON default)
                escaped
            ))
        }
        serde_json::Value::Bool(b) => Some(format!("Value::Concrete(ConcreteValue::Bool({}))", b)),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Some(format!("Value::Concrete(ConcreteValue::Int({}))", i))
            } else {
                n.as_f64()
                    .map(|f| format!("Value::Concrete(ConcreteValue::Float({:.1}))", f))
            }
        }
        _ => None,
    }
}

fn attr_type_accepts_scalar_default(attr_type: &str) -> bool {
    ![
        "iam_policy_document",
        "struct_",
        "::map(",
        "::map (",
        "list(",
        "ref_(",
    ]
    .iter()
    .any(|non_scalar| attr_type.contains(non_scalar))
}

/// Convert a JSON default value to a display string for markdown documentation.
/// Returns `None` for unsupported types (arrays, objects, null).
fn json_default_to_markdown(val: &serde_json::Value) -> Option<String> {
    match val {
        serde_json::Value::String(s) => Some(format!("\"{}\"", s)), // rust-lit-guard: allow (markdown display, not Rust emission)
        serde_json::Value::Bool(b) => Some(format!("{}", b)),
        serde_json::Value::Number(n) => Some(format!("{}", n)),
        _ => None,
    }
}

/// Compute block name from a snake_case attribute name.
///
/// For plural names, returns the singular form (e.g., "tags" -> "tag").
/// For already-singular names (e.g., "statement", "security_group_egress"),
/// returns the name itself so that `List<Struct>` attributes always have a
/// `block_name` for the formatter to use.
fn compute_block_name(name: &str) -> Option<String> {
    let singular = if let Some(stem) = name.strip_suffix("ies") {
        // policies -> policy, entries -> entry
        format!("{}y", stem)
    } else if let Some(stem) = name.strip_suffix("sses") {
        // accesses -> access
        format!("{}ss", stem)
    } else if let Some(stem) = name.strip_suffix("xes") {
        // boxes -> box
        format!("{}x", stem)
    } else if let Some(stem) = name.strip_suffix("ses") {
        // buses -> bus
        format!("{}s", stem)
    } else if let Some(stem) = name.strip_suffix('s') {
        if name.ends_with("ss") || name.ends_with("us") {
            // "access" ends in "ss", "status" ends in "us" -> already singular
            return Some(name.to_string());
        }
        // regions -> region, tags -> tag, sizes -> size
        stem.to_string()
    } else {
        // Already singular (e.g., "statement", "server_side_encryption_configuration")
        return Some(name.to_string());
    };

    if singular.is_empty() {
        None
    } else {
        Some(singular)
    }
}

/// Tags type helper (to be included in generated module)
#[allow(dead_code)]
fn tags_type_helper() -> &'static str {
    r#"
/// Tags type for AWS resources
pub fn tags_type() -> AttributeType {
    AttributeType::map(AttributeType::string())
}
"#
}

fn collapse_whitespace(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut prev_space = false;
    for c in s.chars() {
        if c == ' ' {
            if !prev_space {
                result.push(' ');
            }
            prev_space = true;
        } else {
            result.push(c);
            prev_space = false;
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cfn_schema_for_codegen_tests(type_name: &str) -> CfnSchema {
        let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../cfn-schema-cache")
            .join(format!("{}.json", type_name.replace("::", "__")));
        let raw = std::fs::read_to_string(path).expect("cached CFN schema");
        serde_json::from_str(&raw).expect("cached CFN schema parses")
    }

    /// Build a `CfnDefinition` for classification tests, overriding only the
    /// fields a case cares about.
    fn def(
        def_type: Option<&str>,
        properties: Option<BTreeMap<String, CfnProperty>>,
        one_of: Vec<CfnProperty>,
        enum_values: Option<Vec<EnumValue>>,
        items: Option<CfnProperty>,
        pattern: Option<&str>,
    ) -> CfnDefinition {
        CfnDefinition {
            def_type: def_type.map(str::to_string),
            properties,
            one_of,
            enum_values,
            items: items.map(Box::new),
            pattern: pattern.map(str::to_string),
            ..Default::default()
        }
    }

    fn one_prop() -> BTreeMap<String, CfnProperty> {
        let mut m = BTreeMap::new();
        m.insert("Field".to_string(), CfnProperty::default());
        m
    }

    #[test]
    fn test_rust_lit_escapes_rust_string_literals() {
        assert_eq!(rust_lit("plain").to_string(), "\"plain\"");
        assert_eq!(rust_lit("a\"b").to_string(), "\"a\\\"b\"");
        assert_eq!(rust_lit(r"a\b").to_string(), "\"a\\\\b\"");
        assert_eq!(rust_lit("a\nb").to_string(), "\"a\\nb\"");
        assert_eq!(rust_lit("a\tb").to_string(), "\"a\\tb\"");
    }

    #[test]
    fn test_codegen_raw_string_literal_emissions_are_guarded() {
        let source = include_str!("codegen.rs");
        let escaped_literal_shape = "\\\"{}\\\""; // rust-lit-guard: allow
        let raw_literal_shapes = [
            r#""{}".to_string()"#,       // rust-lit-guard: allow
            r#"string_enum("{}"#,        // rust-lit-guard: allow
            r#"struct_("{}"#,            // rust-lit-guard: allow
            r#"ref_("{}"#,               // rust-lit-guard: allow
            r#"StructField::new("{}"#,   // rust-lit-guard: allow
            r#"with_provider_name("{}"#, // rust-lit-guard: allow
            r#"with_block_name("{}"#,    // rust-lit-guard: allow
            r#"with_description("{}"#,   // rust-lit-guard: allow
        ];

        let offenders: Vec<String> = source
            .lines()
            .enumerate()
            .filter_map(|(idx, line)| {
                if line.contains("rust-lit-guard: allow") {
                    return None;
                }
                let has_escaped_literal_shape = line.contains(escaped_literal_shape);
                let has_raw_literal_shape = raw_literal_shapes.iter().any(|shape| {
                    line.contains(shape) || line.contains(&shape.replace('"', "\\\""))
                });
                if has_escaped_literal_shape || has_raw_literal_shape {
                    Some(format!("{}: {}", idx + 1, line.trim()))
                } else {
                    None
                }
            })
            .collect();

        assert!(
            offenders.is_empty(),
            "raw Rust string literal emission site(s) must use rust_lit or be explicitly allowed:\n{}",
            offenders.join("\n")
        );
    }

    #[test]
    fn test_classify_def_precedence_matches_canonical_order() {
        // classify_def is the single source of truth for $ref shape dispatch
        // (awscc#293). These cases pin the canonical precedence
        // properties > oneOf > enum > array > string-pattern > scalar > opaque
        // so generated output stays byte-identical and a future reorder is
        // caught.

        // Struct wins even if other shape-fields are also present.
        assert!(matches!(
            classify_def(&def(
                Some("object"),
                Some(one_prop()),
                vec![CfnProperty::default()],
                Some(vec![EnumValue::Str("x".into())]),
                None,
                None,
            )),
            ResolvedDef::Struct { .. }
        ));

        // oneOf wins over enum/array/etc when there are no inline properties.
        assert!(matches!(
            classify_def(&def(
                Some("object"),
                None,
                vec![CfnProperty::default()],
                Some(vec![EnumValue::Str("x".into())]),
                None,
                None,
            )),
            ResolvedDef::OneOf { .. }
        ));

        // An integer-typed enum classifies as Enum, NOT Scalar — this is the
        // precedence that keeps WAFv2 `EvaluationWindowSec` an enum.
        assert!(matches!(
            classify_def(&def(
                Some("integer"),
                None,
                vec![],
                Some(vec![EnumValue::Int(60), EnumValue::Int(120)]),
                None,
                None,
            )),
            ResolvedDef::Enum { .. }
        ));

        // Array (type=array + items) beats a stray string-pattern/scalar.
        assert!(matches!(
            classify_def(&def(
                Some("array"),
                None,
                vec![],
                None,
                Some(CfnProperty::default()),
                None,
            )),
            ResolvedDef::Array { .. }
        ));

        // String + pattern -> StringPattern.
        assert!(matches!(
            classify_def(&def(Some("string"), None, vec![], None, None, Some("^a$"))),
            ResolvedDef::StringPattern { .. }
        ));

        // Bare scalar (no enum) -> Scalar with the right kind.
        assert!(matches!(
            classify_def(&def(Some("integer"), None, vec![], None, None, None)),
            ResolvedDef::Scalar {
                kind: ScalarKind::Integer,
                ..
            }
        ));
        assert!(matches!(
            classify_def(&def(Some("number"), None, vec![], None, None, None)),
            ResolvedDef::Scalar {
                kind: ScalarKind::Number,
                ..
            }
        ));
        assert!(matches!(
            classify_def(&def(Some("boolean"), None, vec![], None, None, None)),
            ResolvedDef::Scalar {
                kind: ScalarKind::Boolean,
                ..
            }
        ));

        // Empty properties is NOT a struct — falls through to Opaque.
        assert!(matches!(
            classify_def(&def(
                Some("object"),
                Some(BTreeMap::new()),
                vec![],
                None,
                None,
                None,
            )),
            ResolvedDef::Opaque
        ));

        // A typeless def with none of the shape markers -> Opaque (the
        // explicit fallback that replaced the silent String drop).
        assert!(matches!(
            classify_def(&def(None, None, vec![], None, None, None)),
            ResolvedDef::Opaque
        ));

        // String type with no pattern is not StringPattern -> Opaque.
        assert!(matches!(
            classify_def(&def(Some("string"), None, vec![], None, None, None)),
            ResolvedDef::Opaque
        ));
    }

    #[test]
    fn test_classify_def_scalar_carries_range_and_format() {
        let mut d = def(Some("integer"), None, vec![], None, None, None);
        d.minimum = Some(10);
        d.maximum = Some(2_000_000_000);
        d.format = Some("int64".to_string());
        match classify_def(&d) {
            ResolvedDef::Scalar {
                kind,
                minimum,
                maximum,
                format,
            } => {
                assert_eq!(kind, ScalarKind::Integer);
                assert_eq!(minimum, Some(10));
                assert_eq!(maximum, Some(2_000_000_000));
                assert_eq!(format, Some("int64"));
            }
            _ => panic!("expected Scalar"),
        }
    }

    /// carina#3093: `list_constructor` must let an override force
    /// `unordered_list` even when CFN `insertionOrder` is unspecified
    /// (defaults to ordered). CloudFront `AllowedMethods`/`CachedMethods`
    /// are order-insensitive HTTP-method sets but the CFN schema does
    /// not set `insertionOrder: false`.
    #[test]
    fn test_list_constructor_force_unordered_overrides_default_ordered() {
        // CFN default (insertion_order unspecified) is ordered.
        assert_eq!(
            list_constructor(None, false),
            "AttributeType::list",
            "unspecified insertionOrder defaults to ordered"
        );
        // An override forcing unordered wins over the ordered default.
        assert_eq!(
            list_constructor(None, true),
            "AttributeType::unordered_list",
            "force_unordered must override the ordered CFN default"
        );
        // CFN-declared insertionOrder:false still yields unordered.
        assert_eq!(
            list_constructor(Some(false), false),
            "AttributeType::unordered_list"
        );
        // Explicit CFN ordered + no force = ordered (unchanged behavior).
        assert_eq!(list_constructor(Some(true), false), "AttributeType::list");
    }

    /// The four CloudFront methods overrides must be `EnumUnordered`
    /// so the generated schema uses `unordered_list` (carina#3093).
    #[test]
    fn test_cloudfront_methods_overrides_are_enum_unordered() {
        let ov = resource_type_overrides();
        for key in [
            (
                "AWS::CloudFront::Distribution",
                "DefaultCacheBehavior.AllowedMethods",
            ),
            (
                "AWS::CloudFront::Distribution",
                "CacheBehavior.AllowedMethods",
            ),
            (
                "AWS::CloudFront::Distribution",
                "DefaultCacheBehavior.CachedMethods",
            ),
            (
                "AWS::CloudFront::Distribution",
                "CacheBehavior.CachedMethods",
            ),
        ] {
            match ov.get(&key) {
                Some(TypeOverride::EnumUnordered(_)) => {}
                other => panic!(
                    "{key:?} must be TypeOverride::EnumUnordered (carina#3093), got {other:?}"
                ),
            }
        }
    }

    #[test]
    fn test_collapse_whitespace() {
        assert_eq!(collapse_whitespace("no  extra"), "no extra");
        assert_eq!(collapse_whitespace("many      spaces"), "many spaces");
        assert_eq!(collapse_whitespace("a  b  c"), "a b c");
        assert_eq!(collapse_whitespace("already fine"), "already fine");
        assert_eq!(collapse_whitespace("            12 spaces"), " 12 spaces");
        assert_eq!(collapse_whitespace(""), "");
    }

    #[test]
    fn test_looks_like_property_name() {
        // CamelCase property names should be detected
        assert!(looks_like_property_name("InstanceTenancy"));
        assert!(looks_like_property_name("VpcId"));
        assert!(looks_like_property_name("CidrBlock"));

        // Enum values should not be detected as property names
        assert!(!looks_like_property_name("default"));
        assert!(!looks_like_property_name("dedicated"));
        assert!(!looks_like_property_name("host"));

        // Edge cases
        assert!(!looks_like_property_name(""));
        assert!(!looks_like_property_name("UPPERCASE")); // All uppercase, no lowercase
    }

    #[test]
    fn test_looks_like_enum_value_rejects_lone_punctuation() {
        for value in ["_", "=", "+", "@", "-", "\""] {
            assert!(
                !looks_like_enum_value(value),
                "lone punctuation token {value:?} must not be treated as an enum value"
            );
        }

        assert!(looks_like_enum_value("ENCRYPT_DECRYPT"));
        assert!(looks_like_enum_value("SIGN_VERIFY"));
    }

    #[test]
    fn test_extract_enum_from_description_instance_tenancy() {
        let description = r#"The allowed tenancy of instances launched into the VPC.
  +  ``default``: An instance launched into the VPC runs on shared hardware by default.
  +  ``dedicated``: An instance launched into the VPC runs on dedicated hardware by default.
  +  ``host``: Some description.
 Updating ``InstanceTenancy`` requires no replacement."#;

        let result = extract_enum_from_description(description);
        assert!(result.is_some());
        let values = result.unwrap();
        assert_eq!(values, vec!["default", "dedicated", "host"]);
    }

    #[test]
    fn test_extract_enum_from_description_ecs_cluster_settings_value_scopes_supported_values() {
        let description = r#"The value to set for the cluster setting. The supported values are ``enhanced``, ``enabled``, and ``disabled``.
 To use Container Insights with enhanced observability, set the ``containerInsights`` account setting to ``enhanced``.
 To use Container Insights, set the ``containerInsights`` account setting to ``enabled``.
 If a cluster value is specified, it will override the ``containerInsights`` value set with [PutAccountSetting](https://docs.aws.amazon.com/AmazonECS/latest/APIReference/API_PutAccountSetting.html) or [PutAccountSettingDefault](https://docs.aws.amazon.com/AmazonECS/latest/APIReference/API_PutAccountSettingDefault.html)."#;

        assert_eq!(
            extract_enum_from_description(description),
            Some(vec![
                "enhanced".to_string(),
                "enabled".to_string(),
                "disabled".to_string()
            ])
        );
    }

    #[test]
    fn test_extract_enum_from_description_ecs_execute_command_logging_uses_bullet_leaders() {
        let description = r#"The log setting to use for redirecting logs for your execute command results. The following log settings are available.
  +  ``NONE``: The execute command session is not logged.
  +  ``DEFAULT``: The ``awslogs`` configuration in the task definition is used. If no logging parameter is specified, it defaults to this value. If no ``awslogs`` log driver is configured in the task definition, the output won't be logged.
  +  ``OVERRIDE``: Specify the logging details as a part of ``logConfiguration``. If the ``OVERRIDE`` logging option is specified, the ``logConfiguration`` is required."#;

        assert_eq!(
            extract_enum_from_description(description),
            Some(vec![
                "NONE".to_string(),
                "DEFAULT".to_string(),
                "OVERRIDE".to_string()
            ])
        );
    }

    #[test]
    fn test_extract_enum_from_description_ecs_capacity_provider_allows_custom_names() {
        let description = r#"The short name of the capacity provider. This can be either an AWS managed capacity provider (``FARGATE`` or ``FARGATE_SPOT``) or the name of a custom capacity provider that you created."#;

        assert_eq!(extract_enum_from_description(description), None);
    }

    #[test]
    fn test_extract_enum_from_description_jwt_claim_name_negation_is_not_enum() {
        // CFN says these backticked values are FORBIDDEN ("you can't specify ..."),
        // so they must NOT be scraped as allowed enum members.
        let description = r#"The name of the claim. You can't specify ``exp``, ``iss``, ``nbf``, or ``iat`` because we validate them by default."#;
        assert_eq!(extract_enum_from_description(description), None);
    }

    #[test]
    fn test_extract_enum_negation_followed_by_allowlist_same_sentence_still_extracts() {
        // A "can't specify" clause must NOT swallow a following allow-list that
        // shares the line via a semicolon boundary. The allow-list is a real enum.
        let description = r#"You can't specify ``internal`` for this; valid values are ``ipv4`` or ``dualstack``."#;
        assert_eq!(
            extract_enum_from_description(description),
            Some(vec!["ipv4".to_string(), "dualstack".to_string()])
        );
    }

    #[test]
    fn test_extract_enum_from_description_regression_dynamodb_billing_mode() {
        let description = r#"Specify how you are charged for read and write throughput and how you manage capacity.
 Valid values include:
  +  ``PAY_PER_REQUEST`` - We recommend using ``PAY_PER_REQUEST`` for most DynamoDB workloads. ``PAY_PER_REQUEST`` sets the billing mode to [On-demand capacity mode](https://docs.aws.amazon.com/amazondynamodb/latest/developerguide/on-demand-capacity-mode.html).
  +  ``PROVISIONED`` - We recommend using ``PROVISIONED`` for steady workloads with predictable growth where capacity requirements can be reliably forecasted. ``PROVISIONED`` sets the billing mode to [Provisioned capacity mode](https://docs.aws.amazon.com/amazondynamodb/latest/developerguide/provisioned-capacity-mode.html).

 If not specified, the default is ``PROVISIONED``."#;

        assert_eq!(
            extract_enum_from_description(description),
            Some(vec![
                "PAY_PER_REQUEST".to_string(),
                "PROVISIONED".to_string()
            ])
        );
    }

    #[test]
    fn test_extract_enum_from_description_regression_cloudfront_viewer_protocol_policy() {
        let description = r#"The protocol that viewers can use to access the files in the origin specified by ``TargetOriginId`` when a request matches the path pattern in ``PathPattern``. You can specify the following options:
  +  ``allow-all``: Viewers can use HTTP or HTTPS.
  +  ``redirect-to-https``: If a viewer submits an HTTP request, CloudFront returns an HTTP status code of 301 (Moved Permanently) to the viewer along with the HTTPS URL. The viewer then resubmits the request using the new URL.
  +  ``https-only``: If a viewer sends an HTTP request, CloudFront returns an HTTP status code of 403 (Forbidden)."#;

        assert_eq!(
            extract_enum_from_description(description),
            Some(vec![
                "allow-all".to_string(),
                "redirect-to-https".to_string(),
                "https-only".to_string()
            ])
        );
    }

    #[test]
    fn test_extract_enum_from_description_regression_ec2_ip_protocol() {
        let description = r#"The IP protocol name (``tcp``, ``udp``, ``icmp``, ``icmpv6``) or number (see [Protocol Numbers](https://docs.aws.amazon.com/http://www.iana.org/assignments/protocol-numbers/protocol-numbers.xhtml)).
 Use ``-1`` to specify all protocols. When authorizing security group rules, specifying ``-1`` or a protocol number other than ``tcp``, ``udp``, ``icmp``, or ``icmpv6`` allows traffic on all ports, regardless of any port range you specify. For ``tcp``, ``udp``, and ``icmp``, you must specify a port range. For ``icmpv6``, the port range is optional; if you omit the port range, traffic for all types and codes is allowed."#;

        assert_eq!(
            extract_enum_from_description(description),
            Some(vec![
                "tcp".to_string(),
                "udp".to_string(),
                "icmp".to_string(),
                "icmpv6".to_string(),
                "-1".to_string()
            ])
        );
    }

    #[test]
    fn test_extract_enum_from_description_rejects_kms_tag_allowed_punctuation() {
        let description = r#"The key of the tag. The allowed characters are Unicode letters,
            digits, whitespace, ``_``, ``.``, ``:``, ``/``, ``=``, ``+``, ``@``, ``-``,
            and ``"``."#;
        let result = extract_enum_from_description(description);
        if let Some(values) = &result {
            assert!(
                !values
                    .iter()
                    .any(|v| matches!(v.as_str(), "\"" | "_" | "=")),
                "KMS tag punctuation must not be extracted as enum values (got {values:?})"
            );
        }
        assert!(
            result.is_none() || result.as_ref().is_some_and(|values| values.is_empty()),
            "KMS tag allowed-character punctuation must not form an enum (got {result:?})"
        );
    }

    #[test]
    fn test_extract_enum_from_description_single_value() {
        // Only one value should not be treated as enum
        let description = "Set to ``true`` to enable.";
        let result = extract_enum_from_description(description);
        assert!(result.is_none());
    }

    #[test]
    fn test_extract_enum_from_description_no_backticks() {
        let description = "A regular description without any special formatting.";
        let result = extract_enum_from_description(description);
        assert!(result.is_none());
    }

    #[test]
    fn test_extract_enum_from_description_deduplication() {
        // Same value mentioned multiple times should be deduplicated
        let description =
            r#"Use ``enabled`` or ``disabled``. When ``enabled`` is set, the feature activates."#;
        let result = extract_enum_from_description(description);
        assert!(result.is_some());
        let values = result.unwrap();
        assert_eq!(values, vec!["enabled", "disabled"]);
    }

    #[test]
    fn test_extract_enum_from_description_valid_values_pipe() {
        // "Valid values: X | Y | Z" pattern
        let description = "The connectivity type. Valid values: public | private";
        let result = extract_enum_from_description(description);
        assert!(result.is_some());
        let values = result.unwrap();
        assert_eq!(values, vec!["public", "private"]);
    }

    #[test]
    fn test_extract_enum_from_description_options_colon() {
        // "Options: X, Y, Z" pattern
        let description =
            "Block mode for internet gateway. Options: off, block-bidirectional, block-ingress";
        let result = extract_enum_from_description(description);
        assert!(result.is_some());
        let values = result.unwrap();
        assert_eq!(values, vec!["off", "block-bidirectional", "block-ingress"]);
    }

    #[test]
    fn test_extract_enum_from_description_options_here_are() {
        // "Options here are X, Y, Z" pattern (real CloudFormation format)
        let description =
            "The mode of VPC BPA. Options here are off, block-bidirectional, block-ingress";
        let result = extract_enum_from_description(description);
        assert!(result.is_some());
        let values = result.unwrap();
        assert_eq!(values, vec!["off", "block-bidirectional", "block-ingress"]);
    }

    #[test]
    fn test_extract_enum_from_description_options_are() {
        // "Options are X, Y, or Z" pattern
        let description =
            "The tenancy options. Options are default, dedicated, or host for instances.";
        let result = extract_enum_from_description(description);
        assert!(result.is_some());
        let values = result.unwrap();
        assert_eq!(values, vec!["default", "dedicated", "host"]);
    }

    #[test]
    fn test_extract_enum_from_description_can_be() {
        // "Can be X or Y" pattern
        let description = "The allocation strategy can be zonal or regional";
        let result = extract_enum_from_description(description);
        assert!(result.is_some());
        let values = result.unwrap();
        assert_eq!(values, vec!["zonal", "regional"]);
    }

    #[test]
    fn test_extract_enum_from_description_either() {
        // "Either X or Y" pattern
        let description = "Set the mode to either enabled or disabled";
        let result = extract_enum_from_description(description);
        assert!(result.is_some());
        let values = result.unwrap();
        assert_eq!(values, vec!["enabled", "disabled"]);
    }

    #[test]
    fn test_extract_enum_from_description_rejects_path_examples() {
        let description = "An optional string that you want CloudFront to prefix to the access \
            log ``filenames`` for this distribution, for example, ``myprefix/``. If you want \
            to enable logging, but you don't want to specify a prefix, you still must include \
            an empty ``Prefix`` element in the ``Logging`` element.";
        let result = extract_enum_from_description(description);
        assert!(
            result.is_none(),
            "Path-like example values must not be extracted as enums (got {:?})",
            result
        );
    }

    #[test]
    fn test_extract_enum_from_description_rejects_glob_path_examples() {
        let description = "The pattern (for example, ``images/*.jpg``) that specifies which \
            requests to apply the behavior to. You can also specify a single character such \
            as ``/`` or ``*`` or a path with wildcards like ``/images/*.jpg``.";
        let result = extract_enum_from_description(description);
        assert!(
            result.is_none(),
            "Glob/path examples must not be extracted as enums (got {:?})",
            result
        );
    }

    #[test]
    fn test_extract_enum_from_description_rejects_regex_class_examples() {
        let description = "The key of the tag. Valid characters are ``A-Z``, ``a-z``, \
            ``0-9``, space, and the special characters _ . / = + - @";
        let result = extract_enum_from_description(description);
        assert!(
            result.is_none(),
            "Regex character classes must not be extracted as enums (got {:?})",
            result
        );
    }

    #[test]
    fn test_extract_enum_from_description_rejects_html_path_examples() {
        let description = "The object that you want CloudFront to request from your origin \
            (for example, ``index.html``) when a viewer requests the root URL for your \
            distribution (``https://www.example.com``) instead of an object in your \
            distribution (``https://www.example.com/product-description.html``). You can \
            also specify a folder name (such as ``exampleFolderName/index.html``) or a \
            forward slash (``/``).";
        let result = extract_enum_from_description(description);
        assert!(
            result.is_none(),
            "URL/HTML-path examples must not be extracted as enums (got {:?})",
            result
        );
    }

    #[test]
    fn test_known_enum_overrides() {
        let overrides = known_enum_overrides();

        // IpProtocol should be overridden (canonical AWS values only; alias
        // "all" for "-1" is injected separately via known_enum_aliases)
        let ip_protocol = overrides.get("IpProtocol");
        assert!(ip_protocol.is_some(), "IpProtocol should be in overrides");
        assert_eq!(
            ip_protocol.unwrap(),
            &vec!["tcp", "udp", "icmp", "icmpv6", "-1"]
        );

        // ConnectivityType should be overridden
        let connectivity = overrides.get("ConnectivityType");
        assert!(
            connectivity.is_some(),
            "ConnectivityType should be in overrides"
        );
        assert_eq!(connectivity.unwrap(), &vec!["public", "private"]);
    }

    #[test]
    fn test_known_enum_override_used_in_codegen() {
        // IpProtocol with plain description (no double backticks) should still
        // produce an EnumInfo via known_enum_overrides
        let prop = CfnProperty {
            prop_type: Some(TypeValue::Single("string".to_string())),
            description: Some(
                "The IP protocol name (tcp, udp, icmp, icmpv6) or number.".to_string(),
            ),
            enum_values: None,
            items: None,
            ref_path: None,
            insertion_order: None,
            properties: None,
            required: vec![],
            minimum: None,
            maximum: None,
            additional_properties: None,
            const_value: None,
            default_value: None,
            pattern: None,
            min_items: None,
            max_items: None,
            min_length: None,
            max_length: None,
            format: None,
            ..Default::default()
        };
        let schema = CfnSchema {
            type_name: "AWS::EC2::SecurityGroupIngress".to_string(),
            description: None,
            properties: BTreeMap::new(),
            required: vec![],
            read_only_properties: vec![],
            create_only_properties: vec![],
            write_only_properties: vec![],
            primary_identifier: None,
            definitions: None,
            tagging: None,
            one_of: vec![],
            any_of: vec![],
            handlers: BTreeMap::new(),
        };
        let (_, enum_info) =
            cfn_type_to_carina_type_with_enum(&prop, "IpProtocol", &schema, "", &BTreeMap::new());
        assert!(
            enum_info.is_some(),
            "IpProtocol should produce EnumInfo via overrides"
        );
        let info = enum_info.unwrap();
        assert_eq!(info.type_name, "IpProtocol");
        assert_eq!(info.values, vec!["tcp", "udp", "icmp", "icmpv6", "-1"]);
    }

    #[test]
    fn test_minimum_protocol_version_override_short_circuits_sibling_bleed() {
        // Regression test for awscc#242. The CFN description for
        // ViewerCertificate.MinimumProtocolVersion mentions sibling-field values
        // (`sni-only` from SslSupportMethod, `true` from CloudFrontDefaultCertificate)
        // in backticks, while filtering out the real value `TLSv1` as a "property
        // name". Without the override, extract_enum_from_description picks up the
        // sibling values and ships them as the enum membership; the override must
        // short-circuit that path.
        let description = "If the distribution uses ``Aliases`` (alternate domain \
            names or CNAMEs), specify the security policy that you want CloudFront \
            to use for HTTPS connections with viewers. When you're using SNI only \
            (you set ``SSLSupportMethod`` to ``sni-only``), you must specify \
            ``TLSv1`` or higher. If the distribution uses the CloudFront domain \
            name such as ``d111111abcdef8.cloudfront.net`` (you set \
            ``CloudFrontDefaultCertificate`` to ``true``), CloudFront automatically \
            sets the security policy to ``TLSv1``.";

        // Demonstrate the underlying heuristic still misfires on this description
        // — overrides are the correct fix layer, not a heuristic tweak.
        let scraped = extract_enum_from_description(description);
        assert_eq!(
            scraped,
            Some(vec!["sni-only".to_string(), "true".to_string()]),
            "heuristic still produces the bogus pair; override must short-circuit it"
        );

        let prop = CfnProperty {
            prop_type: Some(TypeValue::Single("string".to_string())),
            description: Some(description.to_string()),
            enum_values: None,
            items: None,
            ref_path: None,
            insertion_order: None,
            properties: None,
            required: vec![],
            minimum: None,
            maximum: None,
            additional_properties: None,
            const_value: None,
            default_value: None,
            pattern: None,
            min_items: None,
            max_items: None,
            min_length: None,
            max_length: None,
            format: None,
            ..Default::default()
        };
        let schema = CfnSchema {
            type_name: "AWS::CloudFront::Distribution".to_string(),
            description: None,
            properties: BTreeMap::new(),
            required: vec![],
            read_only_properties: vec![],
            create_only_properties: vec![],
            write_only_properties: vec![],
            primary_identifier: None,
            definitions: None,
            tagging: None,
            one_of: vec![],
            any_of: vec![],
            handlers: BTreeMap::new(),
        };
        let (_, enum_info) = cfn_type_to_carina_type_with_enum(
            &prop,
            "MinimumProtocolVersion",
            &schema,
            "",
            &BTreeMap::new(),
        );
        let info = enum_info.expect("MinimumProtocolVersion should produce EnumInfo via overrides");
        assert_eq!(info.type_name, "MinimumProtocolVersion");
        // The canonical AWS values must be present (the headline fix for awscc#242).
        for canonical in [
            "SSLv3",
            "TLSv1",
            "TLSv1_2016",
            "TLSv1.1_2016",
            "TLSv1.2_2018",
            "TLSv1.2_2019",
            "TLSv1.2_2021",
        ] {
            assert!(
                info.values.iter().any(|v| v == canonical),
                "MinimumProtocolVersion must carry canonical value {canonical}; got {:?}",
                info.values
            );
        }
        // The sibling-bleed values must NOT appear.
        for bogus in ["sni-only", "true"] {
            assert!(
                !info.values.iter().any(|v| v == bogus),
                "MinimumProtocolVersion must not carry sibling-bleed value {bogus}; got {:?}",
                info.values
            );
        }
        // The DSL aliases registered in known_enum_aliases for this property
        // stay out of `values`; they are accepted through dsl_aliases.
        for alias in [
            "tlsv1_2_2021",
            "tlsv1_2_2019",
            "tlsv1_2_2018",
            "tlsv1_1_2016",
            "tlsv1_2016",
            "tlsv1",
            "sslv3",
        ] {
            assert!(
                !info.values.iter().any(|v| v == alias),
                "MinimumProtocolVersion alias {alias} must stay out of values; got {:?}",
                info.values
            );
        }
    }

    #[test]
    fn test_to_dsl_generated_for_hyphenated_enum_values() {
        // EnumInfo with hyphenated values should trigger to_dsl generation
        let with_hyphens = EnumInfo {
            type_name: "LogDestinationType".to_string(),
            values: vec![
                "cloud-watch-logs".to_string(),
                "s3".to_string(),
                "kinesis-data-firehose".to_string(),
            ],
        };
        assert!(
            with_hyphens.values.iter().any(|v| v.contains('-')),
            "Enum with hyphenated values should be detected"
        );

        let without_hyphens = EnumInfo {
            type_name: "InstanceTenancy".to_string(),
            values: vec![
                "default".to_string(),
                "dedicated".to_string(),
                "host".to_string(),
            ],
        };
        assert!(
            !without_hyphens.values.iter().any(|v| v.contains('-')),
            "Enum without hyphenated values should not be detected"
        );
    }

    #[test]
    fn test_cidr_ip_detected_as_ipv4_cidr() {
        // CidrIp (PascalCase from CloudFormation) should be detected as IPv4 CIDR
        let prop = CfnProperty {
            prop_type: Some(TypeValue::Single("string".to_string())),
            description: Some("The IPv4 address range, in CIDR format.".to_string()),
            enum_values: None,
            items: None,
            ref_path: None,
            insertion_order: None,
            properties: None,
            required: vec![],
            minimum: None,
            maximum: None,
            additional_properties: None,
            const_value: None,
            default_value: None,
            pattern: None,
            min_items: None,
            max_items: None,
            min_length: None,
            max_length: None,
            format: None,
            ..Default::default()
        };
        let schema = CfnSchema {
            type_name: "AWS::EC2::SecurityGroupIngress".to_string(),
            description: None,
            properties: BTreeMap::new(),
            required: vec![],
            read_only_properties: vec![],
            create_only_properties: vec![],
            write_only_properties: vec![],
            primary_identifier: None,
            definitions: None,
            tagging: None,
            one_of: vec![],
            any_of: vec![],
            handlers: BTreeMap::new(),
        };
        let (type_str, _) =
            cfn_type_to_carina_type_with_enum(&prop, "CidrIp", &schema, "", &BTreeMap::new());
        assert_eq!(
            type_str, "types::ipv4_cidr()",
            "CidrIp should produce types::ipv4_cidr()"
        );
    }

    #[test]
    fn test_cidr_ipv6_detected_as_ipv6_cidr() {
        // CidrIpv6 (PascalCase from CloudFormation) should be detected as IPv6 CIDR
        let prop = CfnProperty {
            prop_type: Some(TypeValue::Single("string".to_string())),
            description: Some("The IPv6 address range, in CIDR format.".to_string()),
            enum_values: None,
            items: None,
            ref_path: None,
            insertion_order: None,
            properties: None,
            required: vec![],
            minimum: None,
            maximum: None,
            additional_properties: None,
            const_value: None,
            default_value: None,
            pattern: None,
            min_items: None,
            max_items: None,
            min_length: None,
            max_length: None,
            format: None,
            ..Default::default()
        };
        let schema = CfnSchema {
            type_name: "AWS::EC2::SecurityGroupIngress".to_string(),
            description: None,
            properties: BTreeMap::new(),
            required: vec![],
            read_only_properties: vec![],
            create_only_properties: vec![],
            write_only_properties: vec![],
            primary_identifier: None,
            definitions: None,
            tagging: None,
            one_of: vec![],
            any_of: vec![],
            handlers: BTreeMap::new(),
        };
        let (type_str, _) =
            cfn_type_to_carina_type_with_enum(&prop, "CidrIpv6", &schema, "", &BTreeMap::new());
        assert_eq!(
            type_str, "types::ipv6_cidr()",
            "CidrIpv6 should produce types::ipv6_cidr()"
        );
    }

    #[test]
    fn test_ip_address_detected_as_ipv4_address() {
        // PrivateIpAddress should be detected as IPv4 address
        let prop = CfnProperty {
            prop_type: Some(TypeValue::Single("string".to_string())),
            description: Some("The private IPv4 address.".to_string()),
            enum_values: None,
            items: None,
            ref_path: None,
            insertion_order: None,
            properties: None,
            required: vec![],
            minimum: None,
            maximum: None,
            additional_properties: None,
            const_value: None,
            default_value: None,
            pattern: None,
            min_items: None,
            max_items: None,
            min_length: None,
            max_length: None,
            format: None,
            ..Default::default()
        };
        let schema = CfnSchema {
            type_name: "AWS::EC2::NatGateway".to_string(),
            description: None,
            properties: BTreeMap::new(),
            required: vec![],
            read_only_properties: vec![],
            create_only_properties: vec![],
            write_only_properties: vec![],
            primary_identifier: None,
            definitions: None,
            tagging: None,
            one_of: vec![],
            any_of: vec![],
            handlers: BTreeMap::new(),
        };
        let (type_str, _) = cfn_type_to_carina_type_with_enum(
            &prop,
            "PrivateIpAddress",
            &schema,
            "",
            &BTreeMap::new(),
        );
        assert_eq!(
            type_str, "types::ipv4_address()",
            "PrivateIpAddress should produce types::ipv4_address()"
        );
    }

    #[test]
    fn test_public_ip_detected_as_ipv4_address() {
        // PublicIp should be detected as IPv4 address
        let prop = CfnProperty {
            prop_type: Some(TypeValue::Single("string".to_string())),
            description: Some("The public IP address.".to_string()),
            enum_values: None,
            items: None,
            ref_path: None,
            insertion_order: None,
            properties: None,
            required: vec![],
            minimum: None,
            maximum: None,
            additional_properties: None,
            const_value: None,
            default_value: None,
            pattern: None,
            min_items: None,
            max_items: None,
            min_length: None,
            max_length: None,
            format: None,
            ..Default::default()
        };
        let schema = CfnSchema {
            type_name: "AWS::EC2::EIP".to_string(),
            description: None,
            properties: BTreeMap::new(),
            required: vec![],
            read_only_properties: vec![],
            create_only_properties: vec![],
            write_only_properties: vec![],
            primary_identifier: None,
            definitions: None,
            tagging: None,
            one_of: vec![],
            any_of: vec![],
            handlers: BTreeMap::new(),
        };
        let (type_str, _) =
            cfn_type_to_carina_type_with_enum(&prop, "PublicIp", &schema, "", &BTreeMap::new());
        assert_eq!(
            type_str, "types::ipv4_address()",
            "PublicIp should produce types::ipv4_address()"
        );
    }

    #[test]
    fn test_ip_address_count_stays_int() {
        // SecondaryPrivateIpAddressCount should stay Int, not become IP address
        let prop = CfnProperty {
            prop_type: Some(TypeValue::Single("integer".to_string())),
            description: Some("The number of secondary private IPv4 addresses.".to_string()),
            enum_values: None,
            items: None,
            ref_path: None,
            insertion_order: None,
            properties: None,
            required: vec![],
            minimum: None,
            maximum: None,
            additional_properties: None,
            const_value: None,
            default_value: None,
            pattern: None,
            min_items: None,
            max_items: None,
            min_length: None,
            max_length: None,
            format: None,
            ..Default::default()
        };
        let schema = CfnSchema {
            type_name: "AWS::EC2::NatGateway".to_string(),
            description: None,
            properties: BTreeMap::new(),
            required: vec![],
            read_only_properties: vec![],
            create_only_properties: vec![],
            write_only_properties: vec![],
            primary_identifier: None,
            definitions: None,
            tagging: None,
            one_of: vec![],
            any_of: vec![],
            handlers: BTreeMap::new(),
        };
        let (type_str, _) = cfn_type_to_carina_type_with_enum(
            &prop,
            "SecondaryPrivateIpAddressCount",
            &schema,
            "",
            &BTreeMap::new(),
        );
        assert_eq!(
            type_str, "AttributeType::int()",
            "SecondaryPrivateIpAddressCount should stay Int"
        );
    }

    #[test]
    fn test_availability_zone_detected() {
        let prop = CfnProperty {
            prop_type: Some(TypeValue::Single("string".to_string())),
            description: Some("The Availability Zone.".to_string()),
            enum_values: None,
            items: None,
            ref_path: None,
            insertion_order: None,
            properties: None,
            required: vec![],
            minimum: None,
            maximum: None,
            additional_properties: None,
            const_value: None,
            default_value: None,
            pattern: None,
            min_items: None,
            max_items: None,
            min_length: None,
            max_length: None,
            format: None,
            ..Default::default()
        };
        let schema = CfnSchema {
            type_name: "AWS::EC2::Subnet".to_string(),
            description: None,
            properties: BTreeMap::new(),
            required: vec![],
            read_only_properties: vec![],
            create_only_properties: vec![],
            write_only_properties: vec![],
            primary_identifier: None,
            definitions: None,
            tagging: None,
            one_of: vec![],
            any_of: vec![],
            handlers: BTreeMap::new(),
        };

        // AvailabilityZone should use carina_aws_types::availability_zone()
        let (type_str, _) = cfn_type_to_carina_type_with_enum(
            &prop,
            "AvailabilityZone",
            &schema,
            "",
            &BTreeMap::new(),
        );
        assert_eq!(type_str, "carina_aws_types::availability_zone()");

        // AvailabilityZoneId should use carina_aws_types::availability_zone_id()
        let (type_str, _) = cfn_type_to_carina_type_with_enum(
            &prop,
            "AvailabilityZoneId",
            &schema,
            "",
            &BTreeMap::new(),
        );
        assert_eq!(type_str, "carina_aws_types::availability_zone_id()");
    }

    #[test]
    fn test_is_aws_resource_id_property() {
        // Known resource ID properties
        assert!(is_aws_resource_id_property("VpcId", None));
        assert!(is_aws_resource_id_property("SubnetId", None));
        assert!(is_aws_resource_id_property("SecurityGroupId", None));
        assert!(is_aws_resource_id_property("RouteTableId", None));
        assert!(is_aws_resource_id_property("InternetGatewayId", None));
        assert!(is_aws_resource_id_property("AllocationId", None));
        assert!(is_aws_resource_id_property("NetworkInterfaceId", None));
        assert!(is_aws_resource_id_property("InstanceId", None));
        assert!(is_aws_resource_id_property(
            "DestinationSecurityGroupId",
            None
        ));
        assert!(is_aws_resource_id_property("DestinationPrefixListId", None));
        assert!(is_aws_resource_id_property("VpcEndpointId", None));

        // Non-resource ID properties (should stay String)
        assert!(!is_aws_resource_id_property("AvailabilityZoneId", None));
        assert!(!is_aws_resource_id_property(
            "SourceSecurityGroupOwnerId",
            None
        ));
        assert!(!is_aws_resource_id_property("ResourceId", None));

        // IPAM Pool ID properties should NOT match AwsResourceId
        assert!(!is_aws_resource_id_property("IpamPoolId", None));
        assert!(!is_aws_resource_id_property("Ipv4IpamPoolId", None));
        assert!(!is_aws_resource_id_property("Ipv6IpamPoolId", None));
        assert!(!is_aws_resource_id_property("SourceIpamPoolId", None));
    }

    #[test]
    fn test_is_ipam_pool_id_property() {
        // Known IPAM Pool ID properties
        assert!(is_ipam_pool_id_property("IpamPoolId"));
        assert!(is_ipam_pool_id_property("Ipv4IpamPoolId"));
        assert!(is_ipam_pool_id_property("Ipv6IpamPoolId"));
        assert!(is_ipam_pool_id_property("SourceIpamPoolId"));

        // Non-IPAM Pool ID properties
        assert!(!is_ipam_pool_id_property("VpcId"));
        assert!(!is_ipam_pool_id_property("SubnetId"));
        assert!(!is_ipam_pool_id_property("AllocationId"));
    }

    #[test]
    fn test_list_element_type_display() {
        // String items
        let prop = CfnProperty {
            prop_type: Some(TypeValue::Single("string".to_string())),
            description: None,
            enum_values: None,
            items: None,
            ref_path: None,
            insertion_order: None,
            properties: None,
            required: vec![],
            minimum: None,
            maximum: None,
            additional_properties: None,
            const_value: None,
            default_value: None,
            pattern: None,
            min_items: None,
            max_items: None,
            min_length: None,
            max_length: None,
            format: None,
            ..Default::default()
        };
        assert_eq!(
            list_element_type_display(&prop, "GenericProp", ""),
            "`List<String>`"
        );

        // Integer items
        let prop = CfnProperty {
            prop_type: Some(TypeValue::Single("integer".to_string())),
            description: None,
            enum_values: None,
            items: None,
            ref_path: None,
            insertion_order: None,
            properties: None,
            required: vec![],
            minimum: None,
            maximum: None,
            additional_properties: None,
            const_value: None,
            default_value: None,
            pattern: None,
            min_items: None,
            max_items: None,
            min_length: None,
            max_length: None,
            format: None,
            ..Default::default()
        };
        assert_eq!(
            list_element_type_display(&prop, "GenericProp", ""),
            "`List<Int>`"
        );

        // Boolean items
        let prop = CfnProperty {
            prop_type: Some(TypeValue::Single("boolean".to_string())),
            description: None,
            enum_values: None,
            items: None,
            ref_path: None,
            insertion_order: None,
            properties: None,
            required: vec![],
            minimum: None,
            maximum: None,
            additional_properties: None,
            const_value: None,
            default_value: None,
            pattern: None,
            min_items: None,
            max_items: None,
            min_length: None,
            max_length: None,
            format: None,
            ..Default::default()
        };
        assert_eq!(
            list_element_type_display(&prop, "GenericProp", ""),
            "`List<Bool>`"
        );

        // No type (fallback)
        let prop = CfnProperty {
            prop_type: None,
            description: None,
            enum_values: None,
            items: None,
            ref_path: None,
            insertion_order: None,
            properties: None,
            required: vec![],
            minimum: None,
            maximum: None,
            additional_properties: None,
            const_value: None,
            default_value: None,
            pattern: None,
            min_items: None,
            max_items: None,
            min_length: None,
            max_length: None,
            format: None,
            ..Default::default()
        };
        assert_eq!(
            list_element_type_display(&prop, "GenericProp", ""),
            "`List<String>`"
        );
    }

    #[test]
    fn test_list_element_type_display_with_name_inference() {
        let prop = CfnProperty {
            prop_type: Some(TypeValue::Single("string".to_string())),
            description: None,
            enum_values: None,
            items: None,
            ref_path: None,
            insertion_order: None,
            properties: None,
            required: vec![],
            minimum: None,
            maximum: None,
            additional_properties: None,
            const_value: None,
            default_value: None,
            pattern: None,
            min_items: None,
            max_items: None,
            min_length: None,
            max_length: None,
            format: None,
            ..Default::default()
        };
        assert_eq!(
            list_element_type_display(&prop, "SubnetIds", ""),
            "`List<SubnetId>`"
        );
        assert_eq!(
            list_element_type_display(&prop, "SecurityGroupIds", ""),
            "`List<SecurityGroupId>`"
        );
        assert_eq!(
            list_element_type_display(&prop, "RouteTableIds", ""),
            "`List<RouteTableId>`"
        );
        assert_eq!(
            list_element_type_display(&prop, "NetworkInterfaceIds", ""),
            "`List<NetworkInterfaceId>`"
        );
        assert_eq!(
            list_element_type_display(&prop, "VpcEndpointIds", ""),
            "`List<VpcEndpointId>`"
        );
        assert_eq!(
            list_element_type_display(&prop, "RoleArns", ""),
            "`List<Arn>`"
        );
        assert_eq!(
            list_element_type_display(&prop, "CidrBlocks", ""),
            "`List<Ipv4Cidr>`"
        );
        assert_eq!(
            list_element_type_display(&prop, "Ipv6CidrBlocks", ""),
            "`List<Ipv6Cidr>`"
        );
        assert_eq!(
            list_element_type_display(&prop, "Names", ""),
            "`List<String>`"
        );
        assert_eq!(
            list_element_type_display(&prop, "SubnetId", ""),
            "`List<SubnetId>`"
        );
    }

    #[test]
    fn test_type_display_string_array_tag_ref() {
        // Array with Tag $ref should display List<Map(String)>
        let prop = CfnProperty {
            prop_type: Some(TypeValue::Single("array".to_string())),
            description: None,
            enum_values: None,
            items: Some(Box::new(CfnProperty {
                prop_type: None,
                description: None,
                enum_values: None,
                items: None,
                ref_path: Some("#/definitions/Tag".to_string()),
                insertion_order: None,
                properties: None,
                required: vec![],
                minimum: None,
                maximum: None,
                additional_properties: None,
                const_value: None,
                default_value: None,
                pattern: None,
                min_items: None,
                max_items: None,
                min_length: None,
                max_length: None,
                format: None,
                ..Default::default()
            })),
            ref_path: None,
            insertion_order: None,
            properties: None,
            required: vec![],
            minimum: None,
            maximum: None,
            additional_properties: None,
            const_value: None,
            default_value: None,
            pattern: None,
            min_items: None,
            max_items: None,
            min_length: None,
            max_length: None,
            format: None,
            ..Default::default()
        };
        let schema = CfnSchema {
            type_name: "AWS::EC2::VPC".to_string(),
            description: None,
            properties: BTreeMap::new(),
            required: vec![],
            read_only_properties: vec![],
            create_only_properties: vec![],
            write_only_properties: vec![],
            primary_identifier: None,
            definitions: None,
            tagging: None,
            one_of: vec![],
            any_of: vec![],
            handlers: BTreeMap::new(),
        };
        let enums = BTreeMap::new();
        assert_eq!(
            type_display_string("ResourceTags", &prop, &schema, &enums),
            "`List<Map<String, String>>`"
        );
    }

    #[test]
    fn test_type_display_string_array_unresolvable_ref() {
        // Array with $ref that cannot be resolved should display List<String>
        let prop = CfnProperty {
            prop_type: Some(TypeValue::Single("array".to_string())),
            description: None,
            enum_values: None,
            items: Some(Box::new(CfnProperty {
                prop_type: None,
                description: None,
                enum_values: None,
                items: None,
                ref_path: Some("#/definitions/NonExistent".to_string()),
                insertion_order: None,
                properties: None,
                required: vec![],
                minimum: None,
                maximum: None,
                additional_properties: None,
                const_value: None,
                default_value: None,
                pattern: None,
                min_items: None,
                max_items: None,
                min_length: None,
                max_length: None,
                format: None,
                ..Default::default()
            })),
            ref_path: None,
            insertion_order: None,
            properties: None,
            required: vec![],
            minimum: None,
            maximum: None,
            additional_properties: None,
            const_value: None,
            default_value: None,
            pattern: None,
            min_items: None,
            max_items: None,
            min_length: None,
            max_length: None,
            format: None,
            ..Default::default()
        };
        let schema = CfnSchema {
            type_name: "AWS::EC2::VPC".to_string(),
            description: None,
            properties: BTreeMap::new(),
            required: vec![],
            read_only_properties: vec![],
            create_only_properties: vec![],
            write_only_properties: vec![],
            primary_identifier: None,
            definitions: None,
            tagging: None,
            one_of: vec![],
            any_of: vec![],
            handlers: BTreeMap::new(),
        };
        let enums = BTreeMap::new();
        assert_eq!(
            type_display_string("Items", &prop, &schema, &enums),
            "`List<String>`"
        );
    }

    #[test]
    fn test_type_display_scalar_integer_ref_shows_int_not_string() {
        // awscc#291 markdown sibling: a $ref to a scalar integer def
        // (WAFv2 `RateLimit`) must display as an Int range, not String.
        let mut definitions = BTreeMap::new();
        definitions.insert(
            "RateLimit".to_string(),
            CfnDefinition {
                def_type: Some("integer".to_string()),
                minimum: Some(10),
                maximum: Some(2_000_000_000),
                ..Default::default()
            },
        );
        let prop = CfnProperty {
            ref_path: Some("#/definitions/RateLimit".to_string()),
            ..Default::default()
        };
        let schema = CfnSchema {
            type_name: "AWS::WAFv2::WebACL".to_string(),
            description: None,
            properties: BTreeMap::new(),
            required: vec![],
            read_only_properties: vec![],
            create_only_properties: vec![],
            write_only_properties: vec![],
            primary_identifier: None,
            definitions: Some(definitions),
            tagging: None,
            one_of: vec![],
            any_of: vec![],
            handlers: BTreeMap::new(),
        };
        let enums = BTreeMap::new();
        let display = type_display_string("Limit", &prop, &schema, &enums);
        assert!(
            display.starts_with("Int"),
            "scalar integer $ref must display as Int, got: {display}"
        );
        assert_ne!(display, "String");
    }

    #[test]
    fn test_type_display_list_of_scalar_integer_ref_shows_list_int() {
        // awscc#291 markdown list sibling: an array whose items $ref a scalar
        // integer def must display as `List<Int>`, not `List<String>`. The
        // schema-code path already produces list(int) via its recursing array
        // branch; the markdown array branch inlined its own ref handling and
        // would otherwise diverge.
        let mut definitions = BTreeMap::new();
        definitions.insert(
            "Weight".to_string(),
            CfnDefinition {
                def_type: Some("integer".to_string()),
                ..Default::default()
            },
        );
        let prop = CfnProperty {
            prop_type: Some(TypeValue::Single("array".to_string())),
            items: Some(Box::new(CfnProperty {
                ref_path: Some("#/definitions/Weight".to_string()),
                ..Default::default()
            })),
            ..Default::default()
        };
        let schema = CfnSchema {
            type_name: "AWS::Example::Thing".to_string(),
            description: None,
            properties: BTreeMap::new(),
            required: vec![],
            read_only_properties: vec![],
            create_only_properties: vec![],
            write_only_properties: vec![],
            primary_identifier: None,
            definitions: Some(definitions),
            tagging: None,
            one_of: vec![],
            any_of: vec![],
            handlers: BTreeMap::new(),
        };
        let enums = BTreeMap::new();
        let display = type_display_string("Weights", &prop, &schema, &enums);
        assert_eq!(display, "`List<Int>`", "got: {display}");
    }

    #[test]
    fn test_type_display_string_array_no_items() {
        // Array with no items should display List<String>
        let prop = CfnProperty {
            prop_type: Some(TypeValue::Single("array".to_string())),
            description: None,
            enum_values: None,
            items: None,
            ref_path: None,
            insertion_order: None,
            properties: None,
            required: vec![],
            minimum: None,
            maximum: None,
            additional_properties: None,
            const_value: None,
            default_value: None,
            pattern: None,
            min_items: None,
            max_items: None,
            min_length: None,
            max_length: None,
            format: None,
            ..Default::default()
        };
        let schema = CfnSchema {
            type_name: "AWS::EC2::VPC".to_string(),
            description: None,
            properties: BTreeMap::new(),
            required: vec![],
            read_only_properties: vec![],
            create_only_properties: vec![],
            write_only_properties: vec![],
            primary_identifier: None,
            definitions: None,
            tagging: None,
            one_of: vec![],
            any_of: vec![],
            handlers: BTreeMap::new(),
        };
        let enums = BTreeMap::new();
        assert_eq!(
            type_display_string("SomeList", &prop, &schema, &enums),
            "`List<String>`"
        );
    }

    #[test]
    fn test_list_element_type_display_always_includes_element_type() {
        // Regression guard: list_element_type_display must always return "List<...>"
        // with element type info, never bare "List".
        let test_cases: Vec<Option<TypeValue>> = vec![
            Some(TypeValue::Single("string".to_string())),
            Some(TypeValue::Single("integer".to_string())),
            Some(TypeValue::Single("number".to_string())),
            Some(TypeValue::Single("boolean".to_string())),
            Some(TypeValue::Single("object".to_string())),
            Some(TypeValue::Single("unknown".to_string())),
            None,
        ];
        for type_val in test_cases {
            let prop = CfnProperty {
                prop_type: type_val.clone(),
                description: None,
                enum_values: None,
                items: None,
                ref_path: None,
                insertion_order: None,
                properties: None,
                required: vec![],
                minimum: None,
                maximum: None,
                additional_properties: None,
                const_value: None,
                default_value: None,
                pattern: None,
                min_items: None,
                max_items: None,
                min_length: None,
                max_length: None,
                format: None,
                ..Default::default()
            };
            let result = list_element_type_display(&prop, "GenericProp", "");
            assert!(
                result.contains('<') && result.contains('>'),
                "list_element_type_display should include element type for {:?}, got: {}",
                type_val,
                result
            );
        }
    }

    #[test]
    fn test_type_display_string_array_never_bare_list() {
        // Regression guard: type_display_string for array types must never return
        // bare "List" without element type information.
        let schema = CfnSchema {
            type_name: "AWS::EC2::VPC".to_string(),
            description: None,
            properties: BTreeMap::new(),
            required: vec![],
            read_only_properties: vec![],
            create_only_properties: vec![],
            write_only_properties: vec![],
            primary_identifier: None,
            definitions: None,
            tagging: None,
            one_of: vec![],
            any_of: vec![],
            handlers: BTreeMap::new(),
        };
        let enums = BTreeMap::new();

        // Case 1: array with no items
        let prop = CfnProperty {
            prop_type: Some(TypeValue::Single("array".to_string())),
            description: None,
            enum_values: None,
            items: None,
            ref_path: None,
            insertion_order: None,
            properties: None,
            required: vec![],
            minimum: None,
            maximum: None,
            additional_properties: None,
            const_value: None,
            default_value: None,
            pattern: None,
            min_items: None,
            max_items: None,
            min_length: None,
            max_length: None,
            format: None,
            ..Default::default()
        };
        let result = type_display_string("Prop1", &prop, &schema, &enums);
        assert_ne!(
            result, "List",
            "array with no items should not return bare 'List'"
        );

        // Case 2: array with Tag ref items
        let prop = CfnProperty {
            prop_type: Some(TypeValue::Single("array".to_string())),
            description: None,
            enum_values: None,
            items: Some(Box::new(CfnProperty {
                prop_type: None,
                description: None,
                enum_values: None,
                items: None,
                ref_path: Some("#/definitions/Tag".to_string()),
                insertion_order: None,
                properties: None,
                required: vec![],
                minimum: None,
                maximum: None,
                additional_properties: None,
                const_value: None,
                default_value: None,
                pattern: None,
                min_items: None,
                max_items: None,
                min_length: None,
                max_length: None,
                format: None,
                ..Default::default()
            })),
            ref_path: None,
            insertion_order: None,
            properties: None,
            required: vec![],
            minimum: None,
            maximum: None,
            additional_properties: None,
            const_value: None,
            default_value: None,
            pattern: None,
            min_items: None,
            max_items: None,
            min_length: None,
            max_length: None,
            format: None,
            ..Default::default()
        };
        let result = type_display_string("Prop2", &prop, &schema, &enums);
        assert_ne!(
            result, "List",
            "array with Tag ref should not return bare 'List'"
        );

        // Case 3: array with unresolvable ref items
        let prop = CfnProperty {
            prop_type: Some(TypeValue::Single("array".to_string())),
            description: None,
            enum_values: None,
            items: Some(Box::new(CfnProperty {
                prop_type: None,
                description: None,
                enum_values: None,
                items: None,
                ref_path: Some("#/definitions/Missing".to_string()),
                insertion_order: None,
                properties: None,
                required: vec![],
                minimum: None,
                maximum: None,
                additional_properties: None,
                const_value: None,
                default_value: None,
                pattern: None,
                min_items: None,
                max_items: None,
                min_length: None,
                max_length: None,
                format: None,
                ..Default::default()
            })),
            ref_path: None,
            insertion_order: None,
            properties: None,
            required: vec![],
            minimum: None,
            maximum: None,
            additional_properties: None,
            const_value: None,
            default_value: None,
            pattern: None,
            min_items: None,
            max_items: None,
            min_length: None,
            max_length: None,
            format: None,
            ..Default::default()
        };
        let result = type_display_string("Prop3", &prop, &schema, &enums);
        assert_ne!(
            result, "List",
            "array with unresolvable ref should not return bare 'List'"
        );

        // Case 4: array with items that have no type
        let prop = CfnProperty {
            prop_type: Some(TypeValue::Single("array".to_string())),
            description: None,
            enum_values: None,
            items: Some(Box::new(CfnProperty {
                prop_type: None,
                description: None,
                enum_values: None,
                items: None,
                ref_path: None,
                insertion_order: None,
                properties: None,
                required: vec![],
                minimum: None,
                maximum: None,
                additional_properties: None,
                const_value: None,
                default_value: None,
                pattern: None,
                min_items: None,
                max_items: None,
                min_length: None,
                max_length: None,
                format: None,
                ..Default::default()
            })),
            ref_path: None,
            insertion_order: None,
            properties: None,
            required: vec![],
            minimum: None,
            maximum: None,
            additional_properties: None,
            const_value: None,
            default_value: None,
            pattern: None,
            min_items: None,
            max_items: None,
            min_length: None,
            max_length: None,
            format: None,
            ..Default::default()
        };
        let result = type_display_string("Prop4", &prop, &schema, &enums);
        assert_ne!(
            result, "List",
            "array with typeless items should not return bare 'List'"
        );
    }

    #[test]
    fn test_integer_with_range_produces_custom_type() {
        let prop = CfnProperty {
            prop_type: Some(TypeValue::Single("integer".to_string())),
            description: Some("The netmask length.".to_string()),
            enum_values: None,
            items: None,
            ref_path: None,
            insertion_order: None,
            properties: None,
            required: vec![],
            minimum: Some(0),
            maximum: Some(32),
            additional_properties: None,
            const_value: None,
            default_value: None,
            pattern: None,
            min_items: None,
            max_items: None,
            min_length: None,
            max_length: None,
            format: None,
            ..Default::default()
        };
        let schema = CfnSchema {
            type_name: "AWS::EC2::VPC".to_string(),
            description: None,
            properties: BTreeMap::new(),
            required: vec![],
            read_only_properties: vec![],
            create_only_properties: vec![],
            write_only_properties: vec![],
            primary_identifier: None,
            definitions: None,
            tagging: None,
            one_of: vec![],
            any_of: vec![],
            handlers: BTreeMap::new(),
        };
        let (type_str, _) = cfn_type_to_carina_type_with_enum(
            &prop,
            "Ipv4NetmaskLength",
            &schema,
            "",
            &BTreeMap::new(),
        );
        assert!(
            type_str.contains("AttributeType::refined_int("),
            "Integer with min/max should produce refined Int type, got: {}",
            type_str
        );
        assert!(
            type_str.contains("Some((Some(0), Some(32)))"),
            "refined Int should carry 0..=32 range, got: {}",
            type_str
        );
        assert!(
            type_str.contains("Some((Some(0), Some(32)))"),
            "Should carry range metadata, got: {}",
            type_str
        );
    }

    #[test]
    fn test_integer_without_range_produces_plain_int() {
        let prop = CfnProperty {
            prop_type: Some(TypeValue::Single("integer".to_string())),
            description: Some("Some integer value.".to_string()),
            enum_values: None,
            items: None,
            ref_path: None,
            insertion_order: None,
            properties: None,
            required: vec![],
            minimum: None,
            maximum: None,
            additional_properties: None,
            const_value: None,
            default_value: None,
            pattern: None,
            min_items: None,
            max_items: None,
            min_length: None,
            max_length: None,
            format: None,
            ..Default::default()
        };
        let schema = CfnSchema {
            type_name: "AWS::EC2::VPC".to_string(),
            description: None,
            properties: BTreeMap::new(),
            required: vec![],
            read_only_properties: vec![],
            create_only_properties: vec![],
            write_only_properties: vec![],
            primary_identifier: None,
            definitions: None,
            tagging: None,
            one_of: vec![],
            any_of: vec![],
            handlers: BTreeMap::new(),
        };
        let (type_str, _) =
            cfn_type_to_carina_type_with_enum(&prop, "SomeCount", &schema, "", &BTreeMap::new());
        assert_eq!(type_str, "AttributeType::int()");
    }

    #[test]
    fn test_integer_with_only_minimum_produces_custom_type() {
        // Only minimum set - should produce Custom type with one-sided range
        let prop = CfnProperty {
            prop_type: Some(TypeValue::Single("integer".to_string())),
            description: Some("Some integer value.".to_string()),
            enum_values: None,
            items: None,
            ref_path: None,
            insertion_order: None,
            properties: None,
            required: vec![],
            minimum: Some(0),
            maximum: None,
            additional_properties: None,
            const_value: None,
            default_value: None,
            pattern: None,
            min_items: None,
            max_items: None,
            min_length: None,
            max_length: None,
            format: None,
            ..Default::default()
        };
        let schema = CfnSchema {
            type_name: "AWS::EC2::VPC".to_string(),
            description: None,
            properties: BTreeMap::new(),
            required: vec![],
            read_only_properties: vec![],
            create_only_properties: vec![],
            write_only_properties: vec![],
            primary_identifier: None,
            definitions: None,
            tagging: None,
            one_of: vec![],
            any_of: vec![],
            handlers: BTreeMap::new(),
        };
        let (type_str, _) =
            cfn_type_to_carina_type_with_enum(&prop, "SomeCount", &schema, "", &BTreeMap::new());
        assert!(
            type_str.contains("AttributeType::refined_int("),
            "Integer with only minimum should produce refined Int type, got: {}",
            type_str
        );
        assert!(
            type_str.contains("Some((Some(0), None))"),
            "refined Int should carry minimum range, got: {}",
            type_str
        );
    }

    #[test]
    fn test_integer_with_only_maximum_produces_custom_type() {
        // Only maximum set - should produce Custom type with one-sided range
        let prop = CfnProperty {
            prop_type: Some(TypeValue::Single("integer".to_string())),
            description: Some("Some integer value.".to_string()),
            enum_values: None,
            items: None,
            ref_path: None,
            insertion_order: None,
            properties: None,
            required: vec![],
            minimum: None,
            maximum: Some(100),
            additional_properties: None,
            const_value: None,
            default_value: None,
            pattern: None,
            min_items: None,
            max_items: None,
            min_length: None,
            max_length: None,
            format: None,
            ..Default::default()
        };
        let schema = CfnSchema {
            type_name: "AWS::EC2::VPC".to_string(),
            description: None,
            properties: BTreeMap::new(),
            required: vec![],
            read_only_properties: vec![],
            create_only_properties: vec![],
            write_only_properties: vec![],
            primary_identifier: None,
            definitions: None,
            tagging: None,
            one_of: vec![],
            any_of: vec![],
            handlers: BTreeMap::new(),
        };
        let (type_str, _) =
            cfn_type_to_carina_type_with_enum(&prop, "SomeCount", &schema, "", &BTreeMap::new());
        assert!(
            type_str.contains("AttributeType::refined_int("),
            "Integer with only maximum should produce refined Int type, got: {}",
            type_str
        );
        assert!(
            type_str.contains("Some((None, Some(100)))"),
            "refined Int should carry maximum range, got: {}",
            type_str
        );
    }

    #[test]
    fn test_integer_enum_produces_custom_type() {
        // Integer enum values should produce Custom IntEnum type
        let prop = CfnProperty {
            prop_type: Some(TypeValue::Single("integer".to_string())),
            description: Some("Retention in days.".to_string()),
            enum_values: Some(vec![
                EnumValue::Int(1),
                EnumValue::Int(3),
                EnumValue::Int(5),
                EnumValue::Int(7),
                EnumValue::Int(14),
            ]),
            items: None,
            ref_path: None,
            insertion_order: None,
            properties: None,
            required: vec![],
            minimum: None,
            maximum: None,
            additional_properties: None,
            const_value: None,
            default_value: None,
            pattern: None,
            min_items: None,
            max_items: None,
            min_length: None,
            max_length: None,
            format: None,
            ..Default::default()
        };
        let schema = CfnSchema {
            type_name: "AWS::Logs::LogGroup".to_string(),
            description: None,
            properties: BTreeMap::new(),
            required: vec![],
            read_only_properties: vec![],
            create_only_properties: vec![],
            write_only_properties: vec![],
            primary_identifier: None,
            definitions: None,
            tagging: None,
            one_of: vec![],
            any_of: vec![],
            handlers: BTreeMap::new(),
        };
        let (type_str, _) = cfn_type_to_carina_type_with_enum(
            &prop,
            "RetentionInDays",
            &schema,
            "",
            &BTreeMap::new(),
        );
        assert!(
            type_str.contains("AttributeType::refined_int_with_validator("),
            "Integer enum should produce refined int type, got: {}",
            type_str
        );
        assert!(
            type_str.contains("legacy_validator("),
            "Refined int should carry a validator, got: {}",
            type_str
        );
        assert!(
            type_str.contains("validate_retention_in_days_int_enum"),
            "Should reference int enum validation function, got: {}",
            type_str
        );
    }

    #[test]
    fn test_type_display_string_one_sided_range() {
        let prop = CfnProperty {
            prop_type: Some(TypeValue::Single("integer".to_string())),
            description: Some("Count.".to_string()),
            enum_values: None,
            items: None,
            ref_path: None,
            insertion_order: None,
            properties: None,
            required: vec![],
            minimum: Some(1),
            maximum: None,
            additional_properties: None,
            const_value: None,
            default_value: None,
            pattern: None,
            min_items: None,
            max_items: None,
            min_length: None,
            max_length: None,
            format: None,
            ..Default::default()
        };
        let schema = CfnSchema {
            type_name: "AWS::EC2::NatGateway".to_string(),
            description: None,
            properties: BTreeMap::new(),
            required: vec![],
            read_only_properties: vec![],
            create_only_properties: vec![],
            write_only_properties: vec![],
            primary_identifier: None,
            definitions: None,
            tagging: None,
            one_of: vec![],
            any_of: vec![],
            handlers: BTreeMap::new(),
        };
        let enums = BTreeMap::new();
        let result = type_display_string("SecondaryPrivateIpAddressCount", &prop, &schema, &enums);
        assert_eq!(result, "Int(1..)");
    }

    #[test]
    fn test_type_display_string_int_enum() {
        // Use a property/resource pair that has no resource-scoped override
        let prop = CfnProperty {
            prop_type: Some(TypeValue::Single("integer".to_string())),
            description: Some("Priority values.".to_string()),
            enum_values: Some(vec![
                EnumValue::Int(1),
                EnumValue::Int(3),
                EnumValue::Int(5),
            ]),
            items: None,
            ref_path: None,
            insertion_order: None,
            properties: None,
            required: vec![],
            minimum: None,
            maximum: None,
            additional_properties: None,
            const_value: None,
            default_value: None,
            pattern: None,
            min_items: None,
            max_items: None,
            min_length: None,
            max_length: None,
            format: None,
            ..Default::default()
        };
        let schema = CfnSchema {
            type_name: "AWS::SomeService::Resource".to_string(),
            description: None,
            properties: BTreeMap::new(),
            required: vec![],
            read_only_properties: vec![],
            create_only_properties: vec![],
            write_only_properties: vec![],
            primary_identifier: None,
            definitions: None,
            tagging: None,
            one_of: vec![],
            any_of: vec![],
            handlers: BTreeMap::new(),
        };
        let enums = BTreeMap::new();
        let result = type_display_string("Priority", &prop, &schema, &enums);
        assert_eq!(result, "IntEnum([1, 3, 5])");
    }

    #[test]
    fn test_type_display_string_int_enum_resource_override() {
        // RetentionInDays on AWS::Logs::LogGroup has a resource-scoped IntEnum override
        let prop = CfnProperty {
            prop_type: Some(TypeValue::Single("integer".to_string())),
            description: Some("Retention in days.".to_string()),
            enum_values: Some(vec![
                EnumValue::Int(1),
                EnumValue::Int(3),
                EnumValue::Int(5),
            ]),
            items: None,
            ref_path: None,
            insertion_order: None,
            properties: None,
            required: vec![],
            minimum: None,
            maximum: None,
            additional_properties: None,
            const_value: None,
            default_value: None,
            pattern: None,
            min_items: None,
            max_items: None,
            min_length: None,
            max_length: None,
            format: None,
            ..Default::default()
        };
        let schema = CfnSchema {
            type_name: "AWS::Logs::LogGroup".to_string(),
            description: None,
            properties: BTreeMap::new(),
            required: vec![],
            read_only_properties: vec![],
            create_only_properties: vec![],
            write_only_properties: vec![],
            primary_identifier: None,
            definitions: None,
            tagging: None,
            one_of: vec![],
            any_of: vec![],
            handlers: BTreeMap::new(),
        };
        let enums = BTreeMap::new();
        // Resource-scoped override takes precedence over schema enum values
        let result = type_display_string("RetentionInDays", &prop, &schema, &enums);
        assert_eq!(
            result,
            "IntEnum([1, 3, 5, 7, 14, 30, 60, 90, 120, 150, 180, 365, 400, 545, 731, 1096, 1827, 2192, 2557, 2922, 3288, 3653])"
        );
    }

    #[test]
    fn test_type_display_string_ranged_int_markdown() {
        let prop = CfnProperty {
            prop_type: Some(TypeValue::Single("integer".to_string())),
            description: Some("The netmask length.".to_string()),
            enum_values: None,
            items: None,
            ref_path: None,
            insertion_order: None,
            properties: None,
            required: vec![],
            minimum: Some(0),
            maximum: Some(32),
            additional_properties: None,
            const_value: None,
            default_value: None,
            pattern: None,
            min_items: None,
            max_items: None,
            min_length: None,
            max_length: None,
            format: None,
            ..Default::default()
        };
        let schema = CfnSchema {
            type_name: "AWS::EC2::VPC".to_string(),
            description: None,
            properties: BTreeMap::new(),
            required: vec![],
            read_only_properties: vec![],
            create_only_properties: vec![],
            write_only_properties: vec![],
            primary_identifier: None,
            definitions: None,
            tagging: None,
            one_of: vec![],
            any_of: vec![],
            handlers: BTreeMap::new(),
        };
        let enums = BTreeMap::new();
        let result = type_display_string("Ipv4NetmaskLength", &prop, &schema, &enums);
        assert_eq!(result, "Int(0..=32)");
    }

    #[test]
    fn test_known_int_range_overrides() {
        let overrides = known_int_range_overrides();
        assert_eq!(overrides.get("Ipv4NetmaskLength"), Some(&(0, 32)));
        assert_eq!(overrides.get("Ipv6NetmaskLength"), Some(&(0, 128)));
        assert_eq!(overrides.get("FromPort"), Some(&(-1, 65535)));
        assert_eq!(overrides.get("ToPort"), Some(&(-1, 65535)));
        assert_eq!(overrides.get("MaxSessionDuration"), Some(&(3600, 43200)));
    }

    #[test]
    fn test_known_string_type_overrides() {
        let overrides = known_string_type_overrides();
        assert_eq!(
            overrides.get("DefaultSecurityGroup"),
            Some(&"carina_aws_types::security_group_id()")
        );
        assert_eq!(
            overrides.get("DeliverLogsPermissionArn"),
            Some(&"super::super::iam::role::arn()")
        );
        assert_eq!(
            overrides.get("KmsKeyId"),
            Some(&"super::super::kms::key::arn()")
        );
        assert_eq!(
            overrides.get("KmsKeyArn"),
            Some(&"super::super::kms::key::arn()")
        );
        assert_eq!(
            overrides.get("KMSMasterKeyID"),
            Some(&"carina_aws_types::kms_key_id()")
        );
        assert_eq!(
            overrides.get("ReplicaKmsKeyID"),
            Some(&"carina_aws_types::kms_key_id()")
        );
        assert_eq!(
            overrides.get("PermissionsBoundary"),
            Some(&"super::super::iam::policy::arn()")
        );
    }

    #[test]
    fn arn_overrides_point_to_resource_modules() {
        let overrides = known_string_type_overrides();
        assert_eq!(
            overrides.get("DeliverLogsPermissionArn"),
            Some(&"super::super::iam::role::arn()")
        );
        assert_eq!(
            overrides.get("KmsKeyArn"),
            Some(&"super::super::kms::key::arn()")
        );
        assert_eq!(
            override_type_to_display_name("super::super::iam::oidc_provider::arn()"),
            "IamOidcProviderArn"
        );
    }

    #[test]
    fn arn_emit_choice_has_table_service_and_generic_paths() {
        assert!(matches!(
            arn_emit_choice("iam", "OidcProvider"),
            ArnEmitChoice::PerKind(entry) if entry.validator
                == ArnValidator::Iam {
                    prefix: "oidc-provider/",
                    label: "IAM OIDC Provider",
                }
        ));
        assert!(matches!(
            arn_emit_choice("organizations", "Account"),
            ArnEmitChoice::ServicePrefix("organizations")
        ));
        assert!(matches!(
            arn_emit_choice("unknownservice", "Thing"),
            ArnEmitChoice::Generic
        ));
    }

    #[test]
    fn generated_s3_bucket_owns_arn_helper() {
        let schema = cfn_schema_for_codegen_tests("AWS::S3::Bucket");
        let generated =
            generate_schema_code(&schema, "AWS::S3::Bucket").expect("generate s3 bucket");
        assert!(generated.contains("pub fn arn() -> AttributeType"));
        assert!(
            generated
                .contains("Some(carina_aws_types::provider_type(\"s3\", \"Bucket\", \"Arn\"))")
        );
        assert!(generated.contains("AttributeSchema::new(\"arn\", self::arn())"));
    }

    #[test]
    fn generated_organizations_account_gets_service_prefix_arn_helper() {
        let schema = cfn_schema_for_codegen_tests("AWS::Organizations::Account");
        let generated = generate_schema_code(&schema, "AWS::Organizations::Account")
            .expect("generate organizations account");
        assert!(generated.contains("pub fn arn() -> AttributeType"));
        assert!(generated.contains("validate_service_arn(s, \"organizations\", None)"));
    }

    #[test]
    fn test_string_type_override_applied() {
        // DefaultSecurityGroup should use security_group_id() via override
        let prop = CfnProperty {
            prop_type: Some(TypeValue::Single("string".to_string())),
            description: Some("The ID of the default security group.".to_string()),
            enum_values: None,
            items: None,
            ref_path: None,
            insertion_order: None,
            properties: None,
            required: vec![],
            minimum: None,
            maximum: None,
            additional_properties: None,
            const_value: None,
            default_value: None,
            pattern: None,
            min_items: None,
            max_items: None,
            min_length: None,
            max_length: None,
            format: None,
            ..Default::default()
        };
        let schema = CfnSchema {
            type_name: "AWS::EC2::VPC".to_string(),
            description: None,
            properties: BTreeMap::new(),
            required: vec![],
            read_only_properties: vec![],
            create_only_properties: vec![],
            write_only_properties: vec![],
            primary_identifier: None,
            definitions: None,
            tagging: None,
            one_of: vec![],
            any_of: vec![],
            handlers: BTreeMap::new(),
        };
        let (type_str, _) = cfn_type_to_carina_type_with_enum(
            &prop,
            "DefaultSecurityGroup",
            &schema,
            "",
            &BTreeMap::new(),
        );
        assert_eq!(type_str, "carina_aws_types::security_group_id()");
    }

    #[test]
    fn test_int_range_override_applied() {
        // FromPort without CF min/max should use override (-1..=65535)
        let prop = CfnProperty {
            prop_type: Some(TypeValue::Single("integer".to_string())),
            description: Some("The start of port range.".to_string()),
            enum_values: None,
            items: None,
            ref_path: None,
            insertion_order: None,
            properties: None,
            required: vec![],
            minimum: None,
            maximum: None,
            additional_properties: None,
            const_value: None,
            default_value: None,
            pattern: None,
            min_items: None,
            max_items: None,
            min_length: None,
            max_length: None,
            format: None,
            ..Default::default()
        };
        let schema = CfnSchema {
            type_name: "AWS::EC2::SecurityGroupIngress".to_string(),
            description: None,
            properties: BTreeMap::new(),
            required: vec![],
            read_only_properties: vec![],
            create_only_properties: vec![],
            write_only_properties: vec![],
            primary_identifier: None,
            definitions: None,
            tagging: None,
            one_of: vec![],
            any_of: vec![],
            handlers: BTreeMap::new(),
        };
        let (type_str, _) =
            cfn_type_to_carina_type_with_enum(&prop, "FromPort", &schema, "", &BTreeMap::new());
        assert!(
            type_str.contains("Some((Some(-1), Some(65535)))"),
            "FromPort should carry override range, got: {}",
            type_str
        );
        assert!(
            type_str.contains("AttributeType::refined_int("),
            "FromPort override should use refined Int, got: {}",
            type_str
        );
    }

    #[test]
    fn test_ref_fallback_arn_heuristic() {
        // A $ref property named "Arn" with no resolvable definition should use arn()
        let prop = CfnProperty {
            prop_type: None,
            description: None,
            enum_values: None,
            items: None,
            ref_path: Some("#/definitions/Arn".to_string()),
            insertion_order: None,
            properties: None,
            required: vec![],
            minimum: None,
            maximum: None,
            additional_properties: None,
            const_value: None,
            default_value: None,
            pattern: None,
            min_items: None,
            max_items: None,
            min_length: None,
            max_length: None,
            format: None,
            ..Default::default()
        };
        let schema = CfnSchema {
            type_name: "AWS::S3::Bucket".to_string(),
            description: None,
            properties: BTreeMap::new(),
            required: vec![],
            read_only_properties: vec![],
            create_only_properties: vec![],
            write_only_properties: vec![],
            primary_identifier: None,
            definitions: None,
            tagging: None,
            one_of: vec![],
            any_of: vec![],
            handlers: BTreeMap::new(),
        };
        let (type_str, _) =
            cfn_type_to_carina_type_with_enum(&prop, "Arn", &schema, "", &BTreeMap::new());
        assert_eq!(type_str, "carina_aws_types::arn()");
    }

    #[test]
    fn test_transit_gateway_enum_overrides() {
        let overrides = known_enum_overrides();
        assert_eq!(
            overrides.get("AutoAcceptSharedAttachments"),
            Some(&vec!["enable", "disable"])
        );
        assert_eq!(
            overrides.get("DnsSupport"),
            Some(&vec!["enable", "disable"])
        );
        assert_eq!(
            overrides.get("VpnEcmpSupport"),
            Some(&vec!["enable", "disable"])
        );
    }

    #[test]
    fn test_number_with_range_produces_custom_float_type() {
        // "number" type should produce Float with range constraints
        let prop = CfnProperty {
            prop_type: Some(TypeValue::Single("number".to_string())),
            description: Some("Port number.".to_string()),
            enum_values: None,
            items: None,
            ref_path: None,
            insertion_order: None,
            properties: None,
            required: vec![],
            minimum: Some(0),
            maximum: Some(65535),
            additional_properties: None,
            const_value: None,
            default_value: None,
            pattern: None,
            min_items: None,
            max_items: None,
            min_length: None,
            max_length: None,
            format: None,
            ..Default::default()
        };
        let schema = CfnSchema {
            type_name: "AWS::EC2::SecurityGroupIngress".to_string(),
            description: None,
            properties: BTreeMap::new(),
            required: vec![],
            read_only_properties: vec![],
            create_only_properties: vec![],
            write_only_properties: vec![],
            primary_identifier: None,
            definitions: None,
            tagging: None,
            one_of: vec![],
            any_of: vec![],
            handlers: BTreeMap::new(),
        };
        let (type_str, _) =
            cfn_type_to_carina_type_with_enum(&prop, "FromPort", &schema, "", &BTreeMap::new());
        assert!(
            type_str.contains("Some((Some(0.0), Some(65535.0)))"),
            "Number with range should carry refined range, got: {}",
            type_str
        );
        assert!(
            type_str.contains("AttributeType::refined_float("),
            "Number type should use refined Float, got: {}",
            type_str
        );
    }

    #[test]
    fn test_get_resource_id_type_vpc_id() {
        assert_eq!(
            get_resource_id_type("VpcId", None),
            "carina_aws_types::vpc_id()"
        );
    }

    #[test]
    fn test_get_resource_id_type_subnet_id() {
        assert_eq!(
            get_resource_id_type("SubnetId", None),
            "carina_aws_types::subnet_id()"
        );
    }

    #[test]
    fn test_get_resource_id_type_security_group_id() {
        assert_eq!(
            get_resource_id_type("SecurityGroupId", None),
            "carina_aws_types::security_group_id()"
        );
        assert_eq!(
            get_resource_id_type("DestinationSecurityGroupId", None),
            "carina_aws_types::security_group_id()"
        );
        assert_eq!(
            get_resource_id_type("SourceSecurityGroupId", None),
            "carina_aws_types::security_group_id()"
        );
        // Bare "GroupId" should NOT match SecurityGroupId without a resource-type hint
        // — it's too broad and catches non-EC2 identifiers.
        assert_eq!(
            get_resource_id_type("GroupId", None),
            "carina_aws_types::aws_resource_id()"
        );
    }

    #[test]
    fn test_get_resource_id_type_group_id_on_ec2_security_group_resources() {
        // `GroupId` on the EC2 security-group resources should classify as
        // SecurityGroupId so cross-resource reference typechecking works.
        for resource_type in [
            "AWS::EC2::SecurityGroup",
            "AWS::EC2::SecurityGroupIngress",
            "AWS::EC2::SecurityGroupEgress",
        ] {
            assert_eq!(
                get_resource_id_type("GroupId", Some(resource_type)),
                "carina_aws_types::security_group_id()",
                "GroupId on {resource_type} should be SecurityGroupId",
            );
        }

        // On unrelated resources, bare `GroupId` stays Generic — this is the
        // existing behavior the allowlist is intentionally narrow to preserve
        // (e.g., identitystore false-positive from issue #128).
        assert_eq!(
            get_resource_id_type("GroupId", Some("AWS::IdentityStore::Group")),
            "carina_aws_types::aws_resource_id()",
        );
    }

    #[test]
    fn test_get_resource_id_type_egress_only_internet_gateway_id() {
        assert_eq!(
            get_resource_id_type("EgressOnlyInternetGatewayId", None),
            "carina_aws_types::egress_only_internet_gateway_id()"
        );
    }

    #[test]
    fn test_get_resource_id_type_internet_gateway_id() {
        assert_eq!(
            get_resource_id_type("InternetGatewayId", None),
            "carina_aws_types::internet_gateway_id()"
        );
    }

    #[test]
    fn test_get_resource_id_type_route_table_id() {
        assert_eq!(
            get_resource_id_type("RouteTableId", None),
            "carina_aws_types::route_table_id()"
        );
    }

    #[test]
    fn test_get_resource_id_type_nat_gateway_id() {
        assert_eq!(
            get_resource_id_type("NatGatewayId", None),
            "carina_aws_types::nat_gateway_id()"
        );
    }

    #[test]
    fn test_get_resource_id_type_vpc_peering_connection_id() {
        assert_eq!(
            get_resource_id_type("VpcPeeringConnectionId", None),
            "carina_aws_types::vpc_peering_connection_id()"
        );
    }

    #[test]
    fn test_get_resource_id_type_transit_gateway_id() {
        assert_eq!(
            get_resource_id_type("TransitGatewayId", None),
            "carina_aws_types::transit_gateway_id()"
        );
    }

    #[test]
    fn test_get_resource_id_type_vpn_gateway_id() {
        assert_eq!(
            get_resource_id_type("VpnGatewayId", None),
            "carina_aws_types::vpn_gateway_id()"
        );
    }

    #[test]
    fn test_get_resource_id_type_vpc_endpoint_id() {
        assert_eq!(
            get_resource_id_type("VpcEndpointId", None),
            "carina_aws_types::vpc_endpoint_id()"
        );
    }

    #[test]
    fn test_get_resource_id_type_non_vpc_endpoint_id() {
        // Regression test for #244: ServiceEndpointId should NOT match VPC Endpoint ID
        // Previously, due to operator precedence, anything ending with "endpointid" matched
        assert_eq!(
            get_resource_id_type("ServiceEndpointId", None),
            "carina_aws_types::aws_resource_id()"
        );
    }

    #[test]
    fn test_get_resource_id_type_fallback() {
        assert_eq!(
            get_resource_id_type("SomeUnknownId", None),
            "carina_aws_types::aws_resource_id()"
        );
    }

    #[test]
    fn test_get_resource_id_display_name_vpc_id() {
        assert_eq!(get_resource_id_display_name("VpcId", None), "VpcId");
    }

    #[test]
    fn test_get_resource_id_display_name_subnet_id() {
        assert_eq!(get_resource_id_display_name("SubnetId", None), "SubnetId");
    }

    #[test]
    fn test_get_resource_id_display_name_security_group_id() {
        assert_eq!(
            get_resource_id_display_name("SecurityGroupId", None),
            "SecurityGroupId"
        );
        assert_eq!(
            get_resource_id_display_name("DestinationSecurityGroupId", None),
            "SecurityGroupId"
        );
        // Bare "GroupId" should NOT map to SecurityGroupId
        assert_eq!(
            get_resource_id_display_name("GroupId", None),
            "AwsResourceId"
        );
    }

    #[test]
    fn test_get_resource_id_display_name_egress_only_internet_gateway_id() {
        assert_eq!(
            get_resource_id_display_name("EgressOnlyInternetGatewayId", None),
            "EgressOnlyInternetGatewayId"
        );
    }

    #[test]
    fn test_get_resource_id_display_name_internet_gateway_id() {
        assert_eq!(
            get_resource_id_display_name("InternetGatewayId", None),
            "InternetGatewayId"
        );
    }

    #[test]
    fn test_get_resource_id_display_name_route_table_id() {
        assert_eq!(
            get_resource_id_display_name("RouteTableId", None),
            "RouteTableId"
        );
    }

    #[test]
    fn test_get_resource_id_display_name_nat_gateway_id() {
        assert_eq!(
            get_resource_id_display_name("NatGatewayId", None),
            "NatGatewayId"
        );
    }

    #[test]
    fn test_get_resource_id_display_name_vpc_peering_connection_id() {
        assert_eq!(
            get_resource_id_display_name("VpcPeeringConnectionId", None),
            "VpcPeeringConnectionId"
        );
    }

    #[test]
    fn test_get_resource_id_display_name_transit_gateway_id() {
        assert_eq!(
            get_resource_id_display_name("TransitGatewayId", None),
            "TransitGatewayId"
        );
    }

    #[test]
    fn test_get_resource_id_display_name_vpn_gateway_id() {
        assert_eq!(
            get_resource_id_display_name("VpnGatewayId", None),
            "VpnGatewayId"
        );
    }

    #[test]
    fn test_get_resource_id_display_name_vpc_endpoint_id() {
        assert_eq!(
            get_resource_id_display_name("VpcEndpointId", None),
            "VpcEndpointId"
        );
    }

    #[test]
    fn test_get_resource_id_display_name_non_vpc_endpoint_id() {
        // Regression test for #244: ServiceEndpointId should NOT match VPC Endpoint ID
        assert_eq!(
            get_resource_id_display_name("ServiceEndpointId", None),
            "AwsResourceId"
        );
    }

    #[test]
    fn test_get_resource_id_display_name_fallback() {
        assert_eq!(
            get_resource_id_display_name("SomeUnknownId", None),
            "AwsResourceId"
        );
    }

    #[test]
    fn test_classify_resource_id() {
        assert_eq!(classify_resource_id("VpcId", None), ResourceIdKind::VpcId);
        assert_eq!(
            classify_resource_id("SubnetId", None),
            ResourceIdKind::SubnetId
        );
        assert_eq!(
            classify_resource_id("SecurityGroupId", None),
            ResourceIdKind::SecurityGroupId
        );
        // Bare "GroupId" should be Generic, not SecurityGroupId
        assert_eq!(
            classify_resource_id("GroupId", None),
            ResourceIdKind::Generic
        );
        assert_eq!(
            classify_resource_id("EgressOnlyInternetGatewayId", None),
            ResourceIdKind::EgressOnlyInternetGatewayId
        );
        assert_eq!(
            classify_resource_id("InternetGatewayId", None),
            ResourceIdKind::InternetGatewayId
        );
        assert_eq!(
            classify_resource_id("RouteTableId", None),
            ResourceIdKind::RouteTableId
        );
        assert_eq!(
            classify_resource_id("NatGatewayId", None),
            ResourceIdKind::NatGatewayId
        );
        assert_eq!(
            classify_resource_id("VpcPeeringConnectionId", None),
            ResourceIdKind::VpcPeeringConnectionId
        );
        assert_eq!(
            classify_resource_id("TransitGatewayId", None),
            ResourceIdKind::TransitGatewayId
        );
        assert_eq!(
            classify_resource_id("VpnGatewayId", None),
            ResourceIdKind::VpnGatewayId
        );
        assert_eq!(
            classify_resource_id("VpcEndpointId", None),
            ResourceIdKind::VpcEndpointId
        );
        assert_eq!(
            classify_resource_id("SomeUnknownId", None),
            ResourceIdKind::Generic
        );
        // Regression: ServiceEndpointId should NOT match VpcEndpointId
        assert_eq!(
            classify_resource_id("ServiceEndpointId", None),
            ResourceIdKind::Generic
        );
        // New resource ID types
        assert_eq!(
            classify_resource_id("InstanceId", None),
            ResourceIdKind::InstanceId
        );
        assert_eq!(
            classify_resource_id("NetworkInterfaceId", None),
            ResourceIdKind::NetworkInterfaceId
        );
        assert_eq!(
            classify_resource_id("EniId", None),
            ResourceIdKind::NetworkInterfaceId
        );
        assert_eq!(
            classify_resource_id("AllocationId", None),
            ResourceIdKind::AllocationId
        );
        assert_eq!(
            classify_resource_id("PrefixListId", None),
            ResourceIdKind::PrefixListId
        );
        assert_eq!(
            classify_resource_id("DestinationPrefixListId", None),
            ResourceIdKind::PrefixListId
        );
        assert_eq!(
            classify_resource_id("CarrierGatewayId", None),
            ResourceIdKind::CarrierGatewayId
        );
        assert_eq!(
            classify_resource_id("LocalGatewayId", None),
            ResourceIdKind::LocalGatewayId
        );
        assert_eq!(
            classify_resource_id("NetworkAclId", None),
            ResourceIdKind::NetworkAclId
        );
    }

    #[test]
    fn test_classify_resource_id_type_and_display_name_consistency() {
        // Verify that get_resource_id_type and get_resource_id_display_name
        // agree on classification for all test inputs
        let test_inputs = [
            "VpcId",
            "SubnetId",
            "SecurityGroupId",
            "DestinationSecurityGroupId",
            "GroupId",
            "EgressOnlyInternetGatewayId",
            "InternetGatewayId",
            "RouteTableId",
            "NatGatewayId",
            "VpcPeeringConnectionId",
            "TransitGatewayId",
            "VpnGatewayId",
            "VpcEndpointId",
            "InstanceId",
            "NetworkInterfaceId",
            "EniId",
            "AllocationId",
            "PrefixListId",
            "CarrierGatewayId",
            "LocalGatewayId",
            "NetworkAclId",
            "ServiceEndpointId",
            "SomeUnknownId",
        ];

        for input in &test_inputs {
            let kind = classify_resource_id(input, None);
            let is_generic = kind == ResourceIdKind::Generic;
            let type_is_generic =
                get_resource_id_type(input, None) == "carina_aws_types::aws_resource_id()";
            let display_is_generic = get_resource_id_display_name(input, None) == "AwsResourceId";
            assert_eq!(
                is_generic, type_is_generic,
                "Mismatch for {input}: classify says generic={is_generic}, type says generic={type_is_generic}"
            );
            assert_eq!(
                is_generic, display_is_generic,
                "Mismatch for {input}: classify says generic={is_generic}, display says generic={display_is_generic}"
            );
        }
    }

    #[test]
    fn test_resource_type_overrides_string_type() {
        // IAM Role's Arn should use the schema-owned Role ARN helper, not generic arn
        assert_eq!(
            infer_string_type("Arn", "AWS::IAM::Role"),
            Some("super::super::iam::role::arn()".to_string())
        );
        // Other resources' Arn should use generic arn
        assert_eq!(
            infer_string_type("Arn", "AWS::S3::Bucket"),
            Some("carina_aws_types::arn()".to_string())
        );
        // Non-overridden properties are unaffected
        assert_eq!(
            infer_string_type("VpcId", "AWS::IAM::Role"),
            Some("carina_aws_types::vpc_id()".to_string())
        );
        // EC2 Route's GatewayId should use gateway_id (union), not generic aws_resource_id
        assert_eq!(
            infer_string_type("GatewayId", "AWS::EC2::Route"),
            Some("carina_aws_types::gateway_id()".to_string())
        );
        // Other resources' GatewayId should use generic resource ID type
        assert_eq!(
            infer_string_type("GatewayId", "AWS::EC2::VPNGatewayRoutePropagation"),
            Some("carina_aws_types::aws_resource_id()".to_string())
        );
    }

    #[test]
    fn test_kms_key_policy_object_string_uses_iam_policy_document_override() {
        let prop = CfnProperty {
            prop_type: Some(TypeValue::Multiple(vec![
                "object".to_string(),
                "string".to_string(),
            ])),
            ..Default::default()
        };
        let schema = CfnSchema {
            type_name: "AWS::KMS::Key".to_string(),
            description: None,
            properties: BTreeMap::new(),
            required: vec![],
            read_only_properties: vec![],
            create_only_properties: vec![],
            write_only_properties: vec![],
            primary_identifier: None,
            definitions: None,
            tagging: None,
            one_of: vec![],
            any_of: vec![],
            handlers: BTreeMap::new(),
        };

        let (carina_type, enum_info) = cfn_type_to_carina_type_with_enum_with_struct_path(
            &prop,
            "KeyPolicy",
            &schema,
            "",
            &BTreeMap::new(),
            &[],
        );

        assert_eq!(carina_type, "carina_aws_types::iam_policy_document()");
        assert!(enum_info.is_none());
    }

    #[test]
    fn test_property_oneof_array_ref_maps_to_list_of_struct() {
        let mut key_schema_props = BTreeMap::new();
        key_schema_props.insert(
            "AttributeName".to_string(),
            CfnProperty {
                prop_type: Some(TypeValue::Single("string".to_string())),
                ..Default::default()
            },
        );
        key_schema_props.insert(
            "KeyType".to_string(),
            CfnProperty {
                prop_type: Some(TypeValue::Single("string".to_string())),
                enum_values: Some(vec![
                    EnumValue::Str("HASH".to_string()),
                    EnumValue::Str("RANGE".to_string()),
                ]),
                ..Default::default()
            },
        );

        let mut definitions = BTreeMap::new();
        definitions.insert(
            "KeySchema".to_string(),
            CfnDefinition {
                def_type: Some("object".to_string()),
                properties: Some(key_schema_props),
                required: vec!["AttributeName".to_string(), "KeyType".to_string()],
                ..Default::default()
            },
        );

        let prop = CfnProperty {
            one_of: vec![
                CfnProperty {
                    prop_type: Some(TypeValue::Single("array".to_string())),
                    items: Some(Box::new(CfnProperty {
                        ref_path: Some("#/definitions/KeySchema".to_string()),
                        ..Default::default()
                    })),
                    ..Default::default()
                },
                CfnProperty {
                    prop_type: Some(TypeValue::Single("object".to_string())),
                    ..Default::default()
                },
            ],
            ..Default::default()
        };
        let schema = CfnSchema {
            type_name: "AWS::DynamoDB::Table".to_string(),
            description: None,
            properties: BTreeMap::new(),
            required: vec![],
            read_only_properties: vec![],
            create_only_properties: vec![],
            write_only_properties: vec![],
            primary_identifier: None,
            definitions: Some(definitions),
            tagging: None,
            one_of: vec![],
            any_of: vec![],
            handlers: BTreeMap::new(),
        };

        let (carina_type, enum_info) = cfn_type_to_carina_type_with_enum_with_struct_path(
            &prop,
            "KeySchema",
            &schema,
            "",
            &BTreeMap::new(),
            &[],
        );

        assert!(enum_info.is_none());
        assert!(carina_type.contains("AttributeType::list("));
        assert!(carina_type.contains(r#"AttributeType::struct_("KeySchema""#));
        assert!(carina_type.contains("attribute_name"));
        assert!(carina_type.contains("key_type"));
    }

    #[test]
    #[should_panic(expected = "unresolved oneOf")]
    fn test_property_oneof_unhandled_scalar_union_panics() {
        let prop = CfnProperty {
            one_of: vec![
                CfnProperty {
                    prop_type: Some(TypeValue::Single("string".to_string())),
                    ..Default::default()
                },
                CfnProperty {
                    prop_type: Some(TypeValue::Single("integer".to_string())),
                    ..Default::default()
                },
            ],
            ..Default::default()
        };
        let schema = CfnSchema {
            type_name: "AWS::Test::Resource".to_string(),
            description: None,
            properties: BTreeMap::new(),
            required: vec![],
            read_only_properties: vec![],
            create_only_properties: vec![],
            write_only_properties: vec![],
            primary_identifier: None,
            definitions: None,
            tagging: None,
            one_of: vec![],
            any_of: vec![],
            handlers: BTreeMap::new(),
        };

        let _ = cfn_type_to_carina_type_with_enum_with_struct_path(
            &prop,
            "UnionProperty",
            &schema,
            "",
            &BTreeMap::new(),
            &[],
        );
    }

    #[test]
    fn test_iam_oidc_provider_arn_override() {
        // IAM OIDCProvider's Arn should use the schema-owned OIDC Provider ARN helper.
        assert_eq!(
            infer_string_type("Arn", "AWS::IAM::OIDCProvider"),
            Some("super::super::iam::oidc_provider::arn()".to_string())
        );
        assert_eq!(
            infer_string_type_display("Arn", "AWS::IAM::OIDCProvider"),
            "IamOidcProviderArn"
        );
    }

    #[test]
    fn test_resource_type_overrides_string_type_display() {
        // IAM Role's Arn should display as IamRoleArn
        assert_eq!(
            infer_string_type_display("Arn", "AWS::IAM::Role"),
            "IamRoleArn"
        );
        // Other resources' Arn should display as generic Arn
        assert_eq!(infer_string_type_display("Arn", "AWS::S3::Bucket"), "Arn");
        // EC2 Route's GatewayId should display as GatewayId
        assert_eq!(
            infer_string_type_display("GatewayId", "AWS::EC2::Route"),
            "GatewayId"
        );
        // Other resources' GatewayId should use generic type
        assert_eq!(
            infer_string_type_display("GatewayId", "AWS::EC2::VPNGatewayRoutePropagation"),
            "AwsResourceId"
        );
    }

    #[test]
    fn test_resource_type_overrides_enum() {
        let overrides = resource_type_overrides();
        assert_eq!(
            overrides.get(&("AWS::EC2::VPNGateway", "Type")),
            Some(&TypeOverride::Enum(vec!["ipsec.1"]))
        );
    }

    #[test]
    fn test_resource_type_overrides_int_range() {
        let overrides = resource_type_overrides();
        assert_eq!(
            overrides.get(&("AWS::EC2::VPNGateway", "AmazonSideAsn")),
            Some(&TypeOverride::IntRange(1, 4294967294))
        );
        assert_eq!(
            overrides.get(&("AWS::EC2::TransitGateway", "AmazonSideAsn")),
            Some(&TypeOverride::IntRange(1, 4294967294))
        );
    }

    #[test]
    fn test_resource_type_overrides_int_enum() {
        let overrides = resource_type_overrides();
        assert_eq!(
            overrides.get(&("AWS::EC2::FlowLog", "MaxAggregationInterval")),
            Some(&TypeOverride::IntEnum(vec![60, 600]))
        );
        assert_eq!(
            overrides.get(&("AWS::Logs::LogGroup", "RetentionInDays")),
            Some(&TypeOverride::IntEnum(vec![
                1, 3, 5, 7, 14, 30, 60, 90, 120, 150, 180, 365, 400, 545, 731, 1096, 1827, 2192,
                2557, 2922, 3288, 3653
            ]))
        );
    }

    #[test]
    fn test_infer_string_type_cidr() {
        assert_eq!(
            infer_string_type("CidrBlock", ""),
            Some("types::ipv4_cidr()".to_string())
        );
        assert_eq!(
            infer_string_type("CidrIp", ""),
            Some("types::ipv4_cidr()".to_string())
        );
        assert_eq!(
            infer_string_type("Ipv6CidrBlock", ""),
            Some("types::ipv6_cidr()".to_string())
        );
        assert_eq!(
            infer_string_type("DestinationCidrBlock", ""),
            Some("types::ipv4_cidr()".to_string())
        );
    }

    #[test]
    fn test_infer_string_type_ip_address() {
        assert_eq!(
            infer_string_type("PrivateIpAddress", ""),
            Some("types::ipv4_address()".to_string())
        );
        assert_eq!(
            infer_string_type("PublicIp", ""),
            Some("types::ipv4_address()".to_string())
        );
        assert_eq!(
            infer_string_type("Ipv6IpAddress", ""),
            Some("types::ipv6_address()".to_string())
        );
    }

    #[test]
    fn test_infer_string_type_availability_zone() {
        assert_eq!(
            infer_string_type("AvailabilityZone", ""),
            Some("carina_aws_types::availability_zone()".to_string())
        );
        // AvailabilityZoneId should use availability_zone_id()
        assert_eq!(
            infer_string_type("AvailabilityZoneId", ""),
            Some("carina_aws_types::availability_zone_id()".to_string())
        );
    }

    #[test]
    fn test_infer_string_type_owner_id() {
        assert_eq!(
            infer_string_type("SourceSecurityGroupOwnerId", ""),
            Some("carina_aws_types::aws_account_id()".to_string())
        );
        assert_eq!(
            infer_string_type("PeerOwnerId", ""),
            Some("carina_aws_types::aws_account_id()".to_string())
        );
    }

    #[test]
    fn test_infer_string_type_new_resource_ids() {
        assert_eq!(
            infer_string_type("InstanceId", ""),
            Some("carina_aws_types::instance_id()".to_string())
        );
        assert_eq!(
            infer_string_type("NetworkInterfaceId", ""),
            Some("carina_aws_types::network_interface_id()".to_string())
        );
        assert_eq!(
            infer_string_type("EniId", ""),
            Some("carina_aws_types::network_interface_id()".to_string())
        );
        assert_eq!(
            infer_string_type("AllocationId", ""),
            Some("carina_aws_types::allocation_id()".to_string())
        );
        assert_eq!(
            infer_string_type("PrefixListId", ""),
            Some("carina_aws_types::prefix_list_id()".to_string())
        );
        assert_eq!(
            infer_string_type("DestinationPrefixListId", ""),
            Some("carina_aws_types::prefix_list_id()".to_string())
        );
        assert_eq!(
            infer_string_type("CarrierGatewayId", ""),
            Some("carina_aws_types::carrier_gateway_id()".to_string())
        );
        assert_eq!(
            infer_string_type("LocalGatewayId", ""),
            Some("carina_aws_types::local_gateway_id()".to_string())
        );
    }

    #[test]
    fn test_infer_string_type_ipam_pool_id() {
        assert_eq!(
            infer_string_type("IpamPoolId", ""),
            Some("carina_aws_types::ipam_pool_id()".to_string())
        );
        assert_eq!(
            infer_string_type("Ipv4IpamPoolId", ""),
            Some("carina_aws_types::ipam_pool_id()".to_string())
        );
    }

    #[test]
    fn test_infer_string_type_default_network_acl() {
        assert_eq!(
            infer_string_type("DefaultNetworkAcl", ""),
            Some("carina_aws_types::network_acl_id()".to_string())
        );
    }

    #[test]
    fn test_known_enum_aliases() {
        let aliases = known_enum_aliases();

        // IpProtocol should have "-1" -> "all" alias
        let ip_protocol = aliases.get("IpProtocol");
        assert!(ip_protocol.is_some(), "IpProtocol should have aliases");
        assert_eq!(ip_protocol.unwrap(), &vec![("-1", "all")]);
    }

    #[test]
    fn test_known_enum_aliases_kept_out_of_valid_values() {
        // VALID_* content stays API-canonical; aliases are emitted through dsl_aliases.
        let overrides = known_enum_overrides();
        let aliases = known_enum_aliases();

        let ip_protocol_values = overrides.get("IpProtocol").unwrap();
        assert!(ip_protocol_values.contains(&"-1"), "Should have -1");
        assert!(
            !ip_protocol_values.contains(&"all"),
            "'all' must stay alias-only"
        );

        let ip_protocol_aliases = aliases.get("IpProtocol").unwrap();
        let alias_values: Vec<&str> = ip_protocol_aliases.iter().map(|(_, a)| *a).collect();
        assert!(alias_values.contains(&"all"), "Should have 'all' alias");
    }

    #[test]
    fn test_compute_block_name_simple_plural() {
        assert_eq!(compute_block_name("regions"), Some("region".to_string()));
        assert_eq!(compute_block_name("tags"), Some("tag".to_string()));
        assert_eq!(
            compute_block_name("operating_regions"),
            Some("operating_region".to_string())
        );
    }

    #[test]
    fn test_compute_block_name_ies_suffix() {
        assert_eq!(compute_block_name("policies"), Some("policy".to_string()));
        assert_eq!(compute_block_name("entries"), Some("entry".to_string()));
    }

    #[test]
    fn test_compute_block_name_ses_xes_suffix() {
        assert_eq!(compute_block_name("buses"), Some("bus".to_string()));
        assert_eq!(compute_block_name("boxes"), Some("box".to_string()));
    }

    #[test]
    fn test_compute_block_name_sses_suffix() {
        assert_eq!(compute_block_name("accesses"), Some("access".to_string()));
        // "addresses" -> "address" (sses rule takes priority)
        assert_eq!(compute_block_name("addresses"), Some("address".to_string()));
    }

    #[test]
    fn test_compute_block_name_already_singular() {
        // Already singular names return the name itself
        assert_eq!(compute_block_name("name"), Some("name".to_string()));
        assert_eq!(
            compute_block_name("statement"),
            Some("statement".to_string())
        );
        assert_eq!(
            compute_block_name("server_side_encryption_configuration"),
            Some("server_side_encryption_configuration".to_string())
        );
        // "ss" and "us" endings are treated as singular
        assert_eq!(compute_block_name("status"), Some("status".to_string()));
        assert_eq!(compute_block_name("access"), Some("access".to_string()));
        assert_eq!(
            compute_block_name("security_group_egress"),
            Some("security_group_egress".to_string())
        );
        assert_eq!(
            compute_block_name("security_group_ingress"),
            Some("security_group_ingress".to_string())
        );
    }

    #[test]
    fn test_generate_schema_code_uses_string_enum_for_enum_like_strings() {
        let mut properties = BTreeMap::new();
        properties.insert(
            "AddressFamily".to_string(),
            CfnProperty {
                prop_type: Some(TypeValue::Single("string".to_string())),
                description: Some("The address family. Either IPv4 or IPv6.".to_string()),
                enum_values: Some(vec![
                    EnumValue::Str("IPv4".to_string()),
                    EnumValue::Str("IPv6".to_string()),
                ]),
                items: None,
                ref_path: None,
                insertion_order: None,
                properties: None,
                required: vec![],
                minimum: None,
                maximum: None,
                additional_properties: None,
                const_value: None,
                default_value: None,
                pattern: None,
                min_items: None,
                max_items: None,
                min_length: None,
                max_length: None,
                format: None,
                ..Default::default()
            },
        );

        let schema = CfnSchema {
            type_name: "AWS::EC2::IPAMPool".to_string(),
            description: None,
            properties,
            required: vec!["AddressFamily".to_string()],
            read_only_properties: vec![],
            create_only_properties: vec!["AddressFamily".to_string()],
            write_only_properties: vec![],
            primary_identifier: None,
            definitions: None,
            tagging: None,
            one_of: vec![],
            any_of: vec![],
            handlers: BTreeMap::new(),
        };

        let generated = generate_schema_code(&schema, "AWS::EC2::IPAMPool").unwrap();

        assert!(
            generated.contains("AttributeType::enum_("),
            "enum-like strings should be emitted as Enum: {generated}"
        );
        assert!(
            !generated.contains("validate_address_family"),
            "Enum generation should not fall back to per-attribute validators: {generated}"
        );
        assert!(
            !generated.contains(".with_completions("),
            "enum completions should come from schema type metadata: {generated}"
        );
    }

    #[test]
    fn test_struct_field_enum_emits_enum_not_string() {
        // Simulate a resource with a struct property whose field has an enum.
        // The struct comes from a $ref definition. If the definition's enum field
        // was not picked up during pre-scanning (e.g., due to snake_case conflict),
        // the fallback should still emit Enum, not String.
        let mut def_props = BTreeMap::new();
        def_props.insert(
            "Mode".to_string(),
            CfnProperty {
                prop_type: Some(TypeValue::Single("string".to_string())),
                description: Some("The mode setting".to_string()),
                enum_values: Some(vec![
                    EnumValue::Str("off".to_string()),
                    EnumValue::Str("block-bidirectional".to_string()),
                    EnumValue::Str("block-ingress".to_string()),
                ]),
                items: None,
                ref_path: None,
                insertion_order: None,
                properties: None,
                required: vec![],
                minimum: None,
                maximum: None,
                additional_properties: None,
                const_value: None,
                default_value: None,
                pattern: None,
                min_items: None,
                max_items: None,
                min_length: None,
                max_length: None,
                format: None,
                ..Default::default()
            },
        );

        let mut definitions = BTreeMap::new();
        definitions.insert(
            "BlockSettings".to_string(),
            CfnDefinition {
                def_type: None,
                properties: Some(def_props),
                required: vec![],
                one_of: vec![],
                pattern: None,
                items: None,
                enum_values: None,
                min_length: None,
                max_length: None,
                minimum: None,
                maximum: None,
                format: None,
            },
        );

        let mut properties = BTreeMap::new();
        properties.insert(
            "BlockSettings".to_string(),
            CfnProperty {
                prop_type: None,
                description: Some("Block settings".to_string()),
                enum_values: None,
                items: None,
                ref_path: Some("#/definitions/BlockSettings".to_string()),
                insertion_order: None,
                properties: None,
                required: vec![],
                minimum: None,
                maximum: None,
                additional_properties: None,
                const_value: None,
                default_value: None,
                pattern: None,
                min_items: None,
                max_items: None,
                min_length: None,
                max_length: None,
                format: None,
                ..Default::default()
            },
        );

        let schema = CfnSchema {
            type_name: "AWS::EC2::TestResource".to_string(),
            description: None,
            properties,
            required: vec![],
            read_only_properties: vec![],
            create_only_properties: vec![],
            write_only_properties: vec![],
            primary_identifier: None,
            definitions: Some(definitions),
            tagging: None,
            one_of: vec![],
            any_of: vec![],
            handlers: BTreeMap::new(),
        };

        let generated = generate_schema_code(&schema, "AWS::EC2::TestResource").unwrap();

        assert!(
            generated.contains("AttributeType::enum_("),
            "struct field enums should be emitted as Enum: {generated}"
        );
        // Should have a structured identity (post-#3222, `namespace`
        // was replaced by `identity: Some(enum_identity(...))`).
        assert!(
            generated.contains("carina_core::schema::enum_identity("),
            "Enum should include structured identity: {generated}"
        );
        // Should handle hyphens in values via dsl_aliases. Per #199 / D7,
        // the generated alias table maps each canonical value to its
        // snake_case DSL spelling as data, rather than via a `fn` pointer.
        assert!(
            generated.contains(
                "(\"block-bidirectional\".to_string(), \"block_bidirectional\".to_string())"
            ),
            "hyphenated enum values should generate per-value dsl_aliases entries: {generated}"
        );
    }

    #[test]
    fn test_const_value_generates_single_value_enum() {
        let prop = CfnProperty {
            const_value: Some(serde_json::Value::String("V_1".to_string())),
            ..Default::default()
        };
        let schema = CfnSchema {
            type_name: "AWS::S3::Bucket".to_string(),
            description: None,
            properties: BTreeMap::new(),
            required: vec![],
            read_only_properties: vec![],
            create_only_properties: vec![],
            write_only_properties: vec![],
            primary_identifier: None,
            definitions: None,
            tagging: None,
            one_of: vec![],
            any_of: vec![],
            handlers: BTreeMap::new(),
        };
        let enums = BTreeMap::new();
        let (_, enum_info) = cfn_type_to_carina_type_with_enum(
            &prop,
            "OutputSchemaVersion",
            &schema,
            "aws.s3.Bucket",
            &enums,
        );
        assert!(enum_info.is_some(), "const value should produce enum info");
        let info = enum_info.unwrap();
        assert_eq!(info.values, vec!["V_1"]);
        assert_eq!(info.type_name, "OutputSchemaVersion");
    }

    #[test]
    fn test_array_items_enum_propagated() {
        let prop = CfnProperty {
            prop_type: Some(TypeValue::Single("array".to_string())),
            items: Some(Box::new(CfnProperty {
                prop_type: Some(TypeValue::Single("string".to_string())),
                enum_values: Some(vec![
                    EnumValue::Str("GET".to_string()),
                    EnumValue::Str("PUT".to_string()),
                    EnumValue::Str("HEAD".to_string()),
                    EnumValue::Str("POST".to_string()),
                    EnumValue::Str("DELETE".to_string()),
                ]),
                ..Default::default()
            })),
            ..Default::default()
        };
        let schema = CfnSchema {
            type_name: "AWS::S3::Bucket".to_string(),
            description: None,
            properties: BTreeMap::new(),
            required: vec![],
            read_only_properties: vec![],
            create_only_properties: vec![],
            write_only_properties: vec![],
            primary_identifier: None,
            definitions: None,
            tagging: None,
            one_of: vec![],
            any_of: vec![],
            handlers: BTreeMap::new(),
        };
        let enums = BTreeMap::new();
        let (type_str, enum_info) = cfn_type_to_carina_type_with_enum(
            &prop,
            "AllowedMethods",
            &schema,
            "aws.s3.Bucket",
            &enums,
        );
        assert!(
            type_str.contains("::list(") || type_str.contains("::unordered_list("),
            "array type should produce list: {type_str}"
        );
        assert!(
            enum_info.is_some(),
            "array item enum should be propagated back"
        );
        let info = enum_info.unwrap();
        assert_eq!(info.values.len(), 5);
        assert!(info.values.contains(&"GET".to_string()));
    }

    #[test]
    fn test_ref_enum_only_definition_resolved() {
        let mut definitions = BTreeMap::new();
        definitions.insert(
            "HttpMethod".to_string(),
            CfnDefinition {
                def_type: Some("string".to_string()),
                properties: None,
                required: vec![],
                one_of: vec![],
                pattern: None,
                items: None,
                enum_values: Some(vec![
                    EnumValue::Str("GET".to_string()),
                    EnumValue::Str("POST".to_string()),
                ]),
                min_length: None,
                max_length: None,
                minimum: None,
                maximum: None,
                format: None,
            },
        );
        let prop = CfnProperty {
            ref_path: Some("#/definitions/HttpMethod".to_string()),
            ..Default::default()
        };
        let schema = CfnSchema {
            type_name: "AWS::S3::Bucket".to_string(),
            description: None,
            properties: BTreeMap::new(),
            required: vec![],
            read_only_properties: vec![],
            create_only_properties: vec![],
            write_only_properties: vec![],
            primary_identifier: None,
            definitions: Some(definitions),
            tagging: None,
            one_of: vec![],
            any_of: vec![],
            handlers: BTreeMap::new(),
        };
        let enums = BTreeMap::new();
        let (_, enum_info) = cfn_type_to_carina_type_with_enum(
            &prop,
            "HttpMethod",
            &schema,
            "aws.s3.Bucket",
            &enums,
        );
        assert!(
            enum_info.is_some(),
            "enum-only $ref definition should produce enum info"
        );
        let info = enum_info.unwrap();
        assert_eq!(info.values, vec!["GET", "POST"]);
        assert_eq!(info.type_name, "HttpMethod");
    }

    #[test]
    fn test_ref_array_with_enum_items_definition() {
        // Test a $ref to an array-typed definition where items have enum values
        // (e.g., BlockedEncryptionTypeList)
        let mut definitions = BTreeMap::new();
        definitions.insert(
            "BlockedEncryptionTypeList".to_string(),
            CfnDefinition {
                def_type: Some("array".to_string()),
                properties: None,
                required: vec![],
                one_of: vec![],
                pattern: None,
                items: Some(Box::new(CfnProperty {
                    prop_type: Some(TypeValue::Single("string".to_string())),
                    enum_values: Some(vec![
                        EnumValue::Str("NONE".to_string()),
                        EnumValue::Str("SSE-C".to_string()),
                    ]),
                    ..Default::default()
                })),
                enum_values: None,
                min_length: None,
                max_length: None,
                minimum: None,
                maximum: None,
                format: None,
            },
        );
        let prop = CfnProperty {
            ref_path: Some("#/definitions/BlockedEncryptionTypeList".to_string()),
            ..Default::default()
        };
        let schema = CfnSchema {
            type_name: "AWS::S3::Bucket".to_string(),
            description: None,
            properties: BTreeMap::new(),
            required: vec![],
            read_only_properties: vec![],
            create_only_properties: vec![],
            write_only_properties: vec![],
            primary_identifier: None,
            definitions: Some(definitions),
            tagging: None,
            one_of: vec![],
            any_of: vec![],
            handlers: BTreeMap::new(),
        };
        let enums = BTreeMap::new();
        let (type_str, enum_info) = cfn_type_to_carina_type_with_enum(
            &prop,
            "BlockedEncryptionTypes",
            &schema,
            "aws.s3.Bucket",
            &enums,
        );
        assert!(
            type_str.contains("::list(") || type_str.contains("::unordered_list("),
            "should produce list type: {type_str}"
        );
        assert!(
            enum_info.is_some(),
            "array item enum should be propagated from $ref definition"
        );
        let info = enum_info.unwrap();
        assert_eq!(info.values.len(), 2);
        assert!(info.values.contains(&"NONE".to_string()));
        assert!(info.values.contains(&"SSE-C".to_string()));
    }

    #[test]
    fn test_ref_to_scalar_integer_definition_yields_int_not_string() {
        // awscc#291: WAFv2 WebACL types `Priority`/`Limit` via $ref to
        // scalar integer definitions (`RulePriority`, `RateLimit`). The
        // $ref branch previously fell through to the String fallback for
        // scalar defs, so the field generated as `AttributeType::string()`
        // and the resource became un-appliable (AWS requires Integer).
        // A $ref to a scalar integer def must resolve to an int attribute,
        // honoring the def's own minimum/maximum range constraints.
        let mut definitions = BTreeMap::new();
        definitions.insert(
            "RateLimit".to_string(),
            CfnDefinition {
                def_type: Some("integer".to_string()),
                minimum: Some(10),
                maximum: Some(2_000_000_000),
                ..Default::default()
            },
        );
        let prop = CfnProperty {
            ref_path: Some("#/definitions/RateLimit".to_string()),
            ..Default::default()
        };
        let schema = CfnSchema {
            type_name: "AWS::WAFv2::WebACL".to_string(),
            description: None,
            properties: BTreeMap::new(),
            required: vec![],
            read_only_properties: vec![],
            create_only_properties: vec![],
            write_only_properties: vec![],
            primary_identifier: None,
            definitions: Some(definitions),
            tagging: None,
            one_of: vec![],
            any_of: vec![],
            handlers: BTreeMap::new(),
        };
        let enums = BTreeMap::new();
        let (type_str, _) =
            cfn_type_to_carina_type_with_enum(&prop, "Limit", &schema, "aws.wafv2.WebAcl", &enums);
        assert!(
            type_str.contains("AttributeType::refined_int(")
                || type_str.contains("AttributeType::int()"),
            "scalar integer $ref must yield an int attribute, got: {type_str}"
        );
        assert!(
            !type_str.contains("AttributeType::string()"),
            "scalar integer $ref must not fall back to string: {type_str}"
        );
    }

    #[test]
    fn test_ref_to_scalar_number_definition_yields_float_not_string() {
        // Sibling of the integer case: a $ref to a scalar `number` def must
        // resolve to a float attribute, not the String fallback.
        let mut definitions = BTreeMap::new();
        definitions.insert(
            "Weight".to_string(),
            CfnDefinition {
                def_type: Some("number".to_string()),
                ..Default::default()
            },
        );
        let prop = CfnProperty {
            ref_path: Some("#/definitions/Weight".to_string()),
            ..Default::default()
        };
        let schema = CfnSchema {
            type_name: "AWS::Example::Thing".to_string(),
            description: None,
            properties: BTreeMap::new(),
            required: vec![],
            read_only_properties: vec![],
            create_only_properties: vec![],
            write_only_properties: vec![],
            primary_identifier: None,
            definitions: Some(definitions),
            tagging: None,
            one_of: vec![],
            any_of: vec![],
            handlers: BTreeMap::new(),
        };
        let enums = BTreeMap::new();
        let (type_str, _) = cfn_type_to_carina_type_with_enum(
            &prop,
            "Weight",
            &schema,
            "aws.example.Thing",
            &enums,
        );
        assert!(
            type_str.contains("AttributeType::float()"),
            "scalar number $ref must yield a float attribute, got: {type_str}"
        );
    }

    #[test]
    fn test_ref_to_scalar_boolean_definition_yields_bool_not_string() {
        // Sibling of the integer case: a $ref to a scalar `boolean` def
        // must resolve to a bool attribute, not the String fallback.
        let mut definitions = BTreeMap::new();
        definitions.insert(
            "Toggle".to_string(),
            CfnDefinition {
                def_type: Some("boolean".to_string()),
                ..Default::default()
            },
        );
        let prop = CfnProperty {
            ref_path: Some("#/definitions/Toggle".to_string()),
            ..Default::default()
        };
        let schema = CfnSchema {
            type_name: "AWS::Example::Thing".to_string(),
            description: None,
            properties: BTreeMap::new(),
            required: vec![],
            read_only_properties: vec![],
            create_only_properties: vec![],
            write_only_properties: vec![],
            primary_identifier: None,
            definitions: Some(definitions),
            tagging: None,
            one_of: vec![],
            any_of: vec![],
            handlers: BTreeMap::new(),
        };
        let enums = BTreeMap::new();
        let (type_str, _) = cfn_type_to_carina_type_with_enum(
            &prop,
            "Toggle",
            &schema,
            "aws.example.Thing",
            &enums,
        );
        assert_eq!(type_str, "AttributeType::bool()");
    }

    #[test]
    fn test_struct_field_scalar_ref_integer_generates_int_in_schema_code() {
        // awscc#291 end-to-end: a struct field whose type is a $ref to a
        // scalar integer def (the real WAFv2 `Rule.Priority -> RulePriority`
        // shape) must generate as an int attribute in the final schema code,
        // not a string. This is the symptom site: the mistype lived on a
        // *nested struct field*, and apply rejected the resulting String.
        let mut def_props = BTreeMap::new();
        def_props.insert(
            "Priority".to_string(),
            CfnProperty {
                ref_path: Some("#/definitions/RulePriority".to_string()),
                ..Default::default()
            },
        );
        let mut definitions = BTreeMap::new();
        definitions.insert(
            "Rule".to_string(),
            CfnDefinition {
                def_type: Some("object".to_string()),
                properties: Some(def_props),
                required: vec!["Priority".to_string()],
                ..Default::default()
            },
        );
        definitions.insert(
            "RulePriority".to_string(),
            CfnDefinition {
                def_type: Some("integer".to_string()),
                minimum: Some(0),
                ..Default::default()
            },
        );
        let mut properties = BTreeMap::new();
        properties.insert(
            "Rule".to_string(),
            CfnProperty {
                ref_path: Some("#/definitions/Rule".to_string()),
                ..Default::default()
            },
        );
        let schema = CfnSchema {
            type_name: "AWS::WAFv2::WebACL".to_string(),
            description: None,
            properties,
            required: vec![],
            read_only_properties: vec![],
            create_only_properties: vec![],
            write_only_properties: vec![],
            primary_identifier: None,
            definitions: Some(definitions),
            tagging: None,
            one_of: vec![],
            any_of: vec![],
            handlers: BTreeMap::new(),
        };
        let generated = generate_schema_code(&schema, "AWS::WAFv2::WebACL").unwrap();
        assert!(
            generated.contains("StructField::new(\"priority\", AttributeType::refined_int("),
            "nested struct field via scalar integer $ref must generate as int: {generated}"
        );
        assert!(
            !generated.contains("StructField::new(\"priority\", AttributeType::string()"),
            "nested struct field via scalar integer $ref must not generate as string: {generated}"
        );
    }

    #[test]
    fn test_list_of_scalar_integer_ref_generates_list_of_int_in_schema_code() {
        // awscc#291 schema-code list sibling: a top-level array property whose
        // items $ref a scalar integer def must generate `list(int())`, not
        // `list(string())`. The schema-code array branch recurses through the
        // scalar-ref handling; this locks that path (the markdown list path is
        // covered by test_type_display_list_of_scalar_integer_ref_shows_list_int).
        let mut definitions = BTreeMap::new();
        definitions.insert(
            "Weight".to_string(),
            CfnDefinition {
                def_type: Some("integer".to_string()),
                ..Default::default()
            },
        );
        let mut properties = BTreeMap::new();
        properties.insert(
            "Weights".to_string(),
            CfnProperty {
                prop_type: Some(TypeValue::Single("array".to_string())),
                items: Some(Box::new(CfnProperty {
                    ref_path: Some("#/definitions/Weight".to_string()),
                    ..Default::default()
                })),
                ..Default::default()
            },
        );
        let schema = CfnSchema {
            type_name: "AWS::Example::Thing".to_string(),
            description: None,
            properties,
            required: vec![],
            read_only_properties: vec![],
            create_only_properties: vec![],
            write_only_properties: vec![],
            primary_identifier: None,
            definitions: Some(definitions),
            tagging: None,
            one_of: vec![],
            any_of: vec![],
            handlers: BTreeMap::new(),
        };
        let generated = generate_schema_code(&schema, "AWS::Example::Thing").unwrap();
        assert!(
            generated.contains("AttributeType::list(AttributeType::int())"),
            "list of scalar integer $ref must generate list(int()): {generated}"
        );
        assert!(
            !generated.contains("AttributeType::list(AttributeType::string())"),
            "list of scalar integer $ref must not generate list(string()): {generated}"
        );
    }

    #[test]
    fn test_list_enum_generates_list_of_string_enum_in_schema_code() {
        // Ensure that when an array property has enum items, the generated
        // schema code wraps the StringEnum in List
        let mut properties = BTreeMap::new();
        properties.insert(
            "AllowedMethods".to_string(),
            CfnProperty {
                prop_type: Some(TypeValue::Single("array".to_string())),
                items: Some(Box::new(CfnProperty {
                    prop_type: Some(TypeValue::Single("string".to_string())),
                    enum_values: Some(vec![
                        EnumValue::Str("GET".to_string()),
                        EnumValue::Str("PUT".to_string()),
                    ]),
                    ..Default::default()
                })),
                ..Default::default()
            },
        );
        let schema = CfnSchema {
            type_name: "AWS::S3::Bucket".to_string(),
            description: None,
            properties,
            required: vec![],
            read_only_properties: vec![],
            create_only_properties: vec![],
            write_only_properties: vec![],
            primary_identifier: None,
            definitions: None,
            tagging: None,
            one_of: vec![],
            any_of: vec![],
            handlers: BTreeMap::new(),
        };
        let generated = generate_schema_code(&schema, "AWS::S3::Bucket").unwrap();
        assert!(
            generated.contains("AttributeType::list(AttributeType::enum_("),
            "array with enum items should generate list(Enum): {generated}"
        );
    }

    #[test]
    fn test_struct_field_const_value_shows_enum_in_markdown() {
        let mut def_props = BTreeMap::new();
        def_props.insert(
            "ObjectLockEnabled".to_string(),
            CfnProperty {
                prop_type: Some(TypeValue::Single("string".to_string())),
                const_value: Some(serde_json::Value::String("Enabled".to_string())),
                description: Some("Object lock enabled".to_string()),
                ..Default::default()
            },
        );

        let mut definitions = BTreeMap::new();
        definitions.insert(
            "ObjectLockConfiguration".to_string(),
            CfnDefinition {
                def_type: Some("object".to_string()),
                properties: Some(def_props),
                required: vec![],
                enum_values: None,
                items: None,
                one_of: vec![],
                pattern: None,
                min_length: None,
                max_length: None,
                minimum: None,
                maximum: None,
                format: None,
            },
        );

        let mut properties = BTreeMap::new();
        properties.insert(
            "ObjectLockConfiguration".to_string(),
            CfnProperty {
                ref_path: Some("#/definitions/ObjectLockConfiguration".to_string()),
                ..Default::default()
            },
        );

        let schema = CfnSchema {
            type_name: "AWS::S3::Bucket".to_string(),
            description: None,
            properties,
            required: vec![],
            read_only_properties: vec![],
            create_only_properties: vec![],
            write_only_properties: vec![],
            primary_identifier: None,
            definitions: Some(definitions),
            tagging: None,
            one_of: vec![],
            any_of: vec![],
            handlers: BTreeMap::new(),
        };

        let md = generate_markdown(&schema, "AWS::S3::Bucket").unwrap();
        assert!(
            md.contains("Enum (ObjectLockEnabled)"),
            "Struct field with const_value should display as Enum in markdown, got:\n{}",
            md.lines()
                .filter(|l| l.contains("object_lock_enabled"))
                .collect::<Vec<_>>()
                .join("\n")
        );
    }

    #[test]
    fn test_struct_field_ref_enum_shows_enum_in_markdown() {
        // Struct field with $ref to an enum-only definition
        let mut def_props = BTreeMap::new();
        def_props.insert(
            "Mode".to_string(),
            CfnProperty {
                ref_path: Some("#/definitions/RetentionMode".to_string()),
                ..Default::default()
            },
        );

        let mut definitions = BTreeMap::new();
        definitions.insert(
            "DefaultRetention".to_string(),
            CfnDefinition {
                def_type: Some("object".to_string()),
                properties: Some(def_props),
                required: vec![],
                enum_values: None,
                items: None,
                one_of: vec![],
                pattern: None,
                min_length: None,
                max_length: None,
                minimum: None,
                maximum: None,
                format: None,
            },
        );
        definitions.insert(
            "RetentionMode".to_string(),
            CfnDefinition {
                def_type: Some("string".to_string()),
                properties: None,
                required: vec![],
                enum_values: Some(vec![
                    EnumValue::Str("COMPLIANCE".to_string()),
                    EnumValue::Str("GOVERNANCE".to_string()),
                ]),
                items: None,
                one_of: vec![],
                pattern: None,
                min_length: None,
                max_length: None,
                minimum: None,
                maximum: None,
                format: None,
            },
        );

        let mut properties = BTreeMap::new();
        properties.insert(
            "DefaultRetention".to_string(),
            CfnProperty {
                ref_path: Some("#/definitions/DefaultRetention".to_string()),
                ..Default::default()
            },
        );

        let schema = CfnSchema {
            type_name: "AWS::S3::Bucket".to_string(),
            description: None,
            properties,
            required: vec![],
            read_only_properties: vec![],
            create_only_properties: vec![],
            write_only_properties: vec![],
            primary_identifier: None,
            definitions: Some(definitions),
            tagging: None,
            one_of: vec![],
            any_of: vec![],
            handlers: BTreeMap::new(),
        };

        let md = generate_markdown(&schema, "AWS::S3::Bucket").unwrap();
        assert!(
            md.contains("Enum (RetentionMode)"),
            "Struct field with $ref to enum-only definition should display as Enum, got:\n{}",
            md.lines()
                .filter(|l| l.contains("mode"))
                .collect::<Vec<_>>()
                .join("\n")
        );
    }

    #[test]
    fn test_shared_enum_type_across_structs_is_qualified_in_markdown() {
        // Two structs (Ingress, Egress) sharing the same enum field name
        // (IpProtocol) get parent-qualified TypeIdentities — even when the
        // CFN schema factors the enum into a single `#/definitions/IpProtocol`
        // and points both fields at it. This is the carina#3350 rule: nested
        // enums are always identified by their enclosing struct so a future
        // diverging values list (Ingress-only / Egress-only) cannot be
        // silently routed to the wrong validator.
        let mut ingress_props = BTreeMap::new();
        ingress_props.insert(
            "IpProtocol".to_string(),
            CfnProperty {
                ref_path: Some("#/definitions/IpProtocol".to_string()),
                ..Default::default()
            },
        );

        let mut egress_props = BTreeMap::new();
        egress_props.insert(
            "IpProtocol".to_string(),
            CfnProperty {
                ref_path: Some("#/definitions/IpProtocol".to_string()),
                ..Default::default()
            },
        );

        let mut definitions = BTreeMap::new();
        definitions.insert(
            "Ingress".to_string(),
            CfnDefinition {
                def_type: Some("object".to_string()),
                properties: Some(ingress_props),
                required: vec![],
                enum_values: None,
                items: None,
                one_of: vec![],
                pattern: None,
                min_length: None,
                max_length: None,
                minimum: None,
                maximum: None,
                format: None,
            },
        );
        definitions.insert(
            "Egress".to_string(),
            CfnDefinition {
                def_type: Some("object".to_string()),
                properties: Some(egress_props),
                required: vec![],
                enum_values: None,
                items: None,
                one_of: vec![],
                pattern: None,
                min_length: None,
                max_length: None,
                minimum: None,
                maximum: None,
                format: None,
            },
        );
        definitions.insert(
            "IpProtocol".to_string(),
            CfnDefinition {
                def_type: Some("string".to_string()),
                properties: None,
                required: vec![],
                enum_values: Some(vec![
                    EnumValue::Str("tcp".to_string()),
                    EnumValue::Str("udp".to_string()),
                ]),
                items: None,
                one_of: vec![],
                pattern: None,
                min_length: None,
                max_length: None,
                minimum: None,
                maximum: None,
                format: None,
            },
        );

        let mut properties = BTreeMap::new();
        properties.insert(
            "SecurityGroupIngress".to_string(),
            CfnProperty {
                prop_type: Some(TypeValue::Single("array".to_string())),
                items: Some(Box::new(CfnProperty {
                    ref_path: Some("#/definitions/Ingress".to_string()),
                    ..Default::default()
                })),
                ..Default::default()
            },
        );
        properties.insert(
            "SecurityGroupEgress".to_string(),
            CfnProperty {
                prop_type: Some(TypeValue::Single("array".to_string())),
                items: Some(Box::new(CfnProperty {
                    ref_path: Some("#/definitions/Egress".to_string()),
                    ..Default::default()
                })),
                ..Default::default()
            },
        );

        let schema = CfnSchema {
            type_name: "AWS::EC2::SecurityGroup".to_string(),
            description: None,
            properties,
            required: vec![],
            read_only_properties: vec![],
            create_only_properties: vec![],
            write_only_properties: vec![],
            primary_identifier: None,
            definitions: Some(definitions),
            tagging: None,
            one_of: vec![],
            any_of: vec![],
            handlers: BTreeMap::new(),
        };

        let md = generate_markdown(&schema, "AWS::EC2::SecurityGroup").unwrap();
        let egress_count = md
            .matches("aws.ec2.SecurityGroup.Egress.IpProtocol.tcp")
            .count();
        let ingress_count = md
            .matches("aws.ec2.SecurityGroup.Ingress.IpProtocol.tcp")
            .count();
        assert_eq!(
            egress_count,
            1,
            "Expected one Egress structural enum identifier, got {}.\nEnum sections:\n{}",
            egress_count,
            md.lines()
                .filter(|l| l.contains("IpProtocol"))
                .collect::<Vec<_>>()
                .join("\n")
        );
        assert_eq!(
            ingress_count,
            1,
            "Expected one Ingress structural enum identifier, got {}.\nEnum sections:\n{}",
            ingress_count,
            md.lines()
                .filter(|l| l.contains("IpProtocol"))
                .collect::<Vec<_>>()
                .join("\n")
        );
        assert_eq!(
            md.matches("### ip_protocol (IpProtocol)").count(),
            2,
            "The enum kind remains plain while the namespace carries the struct path.",
        );
    }

    #[test]
    fn test_same_type_name_different_values_not_deduplicated() {
        // Two structs with a "Status" field but different enum values
        // should produce two separate enum sections
        let mut config_a_props = BTreeMap::new();
        config_a_props.insert(
            "Status".to_string(),
            CfnProperty {
                enum_values: Some(vec![
                    EnumValue::Str("Disabled".to_string()),
                    EnumValue::Str("Enabled".to_string()),
                ]),
                ..Default::default()
            },
        );

        let mut config_b_props = BTreeMap::new();
        config_b_props.insert(
            "Status".to_string(),
            CfnProperty {
                enum_values: Some(vec![
                    EnumValue::Str("Enabled".to_string()),
                    EnumValue::Str("Suspended".to_string()),
                ]),
                ..Default::default()
            },
        );

        let mut definitions = BTreeMap::new();
        definitions.insert(
            "ConfigA".to_string(),
            CfnDefinition {
                def_type: Some("object".to_string()),
                properties: Some(config_a_props),
                required: vec![],
                enum_values: None,
                items: None,
                one_of: vec![],
                pattern: None,
                min_length: None,
                max_length: None,
                minimum: None,
                maximum: None,
                format: None,
            },
        );
        definitions.insert(
            "ConfigB".to_string(),
            CfnDefinition {
                def_type: Some("object".to_string()),
                properties: Some(config_b_props),
                required: vec![],
                enum_values: None,
                items: None,
                one_of: vec![],
                pattern: None,
                min_length: None,
                max_length: None,
                minimum: None,
                maximum: None,
                format: None,
            },
        );

        let mut properties = BTreeMap::new();
        properties.insert(
            "ConfigA".to_string(),
            CfnProperty {
                ref_path: Some("#/definitions/ConfigA".to_string()),
                ..Default::default()
            },
        );
        properties.insert(
            "ConfigB".to_string(),
            CfnProperty {
                ref_path: Some("#/definitions/ConfigB".to_string()),
                ..Default::default()
            },
        );

        let schema = CfnSchema {
            type_name: "AWS::Test::Resource".to_string(),
            description: None,
            properties,
            required: vec![],
            read_only_properties: vec![],
            create_only_properties: vec![],
            write_only_properties: vec![],
            primary_identifier: None,
            definitions: Some(definitions),
            tagging: None,
            one_of: vec![],
            any_of: vec![],
            handlers: BTreeMap::new(),
        };

        let md = generate_markdown(&schema, "AWS::Test::Resource").unwrap();

        // With structural disambiguation, the two Status enums keep the same
        // plain kind and differ by namespace.
        assert!(
            md.contains("aws.test.Resource.ConfigA.Status.disabled"),
            "Expected ConfigA structural DSL identifier.\nEnum Values section:\n{}",
            md.lines()
                .filter(|l| l.contains("Status") || l.contains("status"))
                .collect::<Vec<_>>()
                .join("\n")
        );
        assert!(
            md.contains("aws.test.Resource.ConfigB.Status.suspended"),
            "Expected ConfigB structural DSL identifier.\nEnum Values section:\n{}",
            md.lines()
                .filter(|l| l.contains("Status") || l.contains("status"))
                .collect::<Vec<_>>()
                .join("\n")
        );

        assert!(
            !md.contains("aws.test.Resource.Status."),
            "Should not have ambiguous root-scoped Status identifiers"
        );
    }

    #[test]
    fn test_inline_enum_struct_fields_display_as_links_not_plain_enum() {
        // Struct fields with inline enum_values should display as linked
        // "[Enum (TypeName)](#...)" in the struct table, not plain "Enum".
        let mut config_props = BTreeMap::new();
        config_props.insert(
            "Status".to_string(),
            CfnProperty {
                enum_values: Some(vec![
                    EnumValue::Str("Enabled".to_string()),
                    EnumValue::Str("Disabled".to_string()),
                ]),
                description: Some("The status field".to_string()),
                ..Default::default()
            },
        );

        let mut definitions = BTreeMap::new();
        definitions.insert(
            "MyConfig".to_string(),
            CfnDefinition {
                def_type: Some("object".to_string()),
                properties: Some(config_props),
                required: vec![],
                enum_values: None,
                items: None,
                one_of: vec![],
                pattern: None,
                min_length: None,
                max_length: None,
                minimum: None,
                maximum: None,
                format: None,
            },
        );

        let mut properties = BTreeMap::new();
        properties.insert(
            "MyConfig".to_string(),
            CfnProperty {
                ref_path: Some("#/definitions/MyConfig".to_string()),
                ..Default::default()
            },
        );

        let schema = CfnSchema {
            type_name: "AWS::Test::Resource".to_string(),
            description: None,
            properties,
            required: vec![],
            read_only_properties: vec![],
            create_only_properties: vec![],
            write_only_properties: vec![],
            primary_identifier: None,
            definitions: Some(definitions),
            tagging: None,
            one_of: vec![],
            any_of: vec![],
            handlers: BTreeMap::new(),
        };

        let md = generate_markdown(&schema, "AWS::Test::Resource").unwrap();

        // The struct field table should contain a link, not plain "Enum".
        assert!(
            md.contains("[Enum (Status)]"),
            "Expected linked enum type in struct field table, got plain 'Enum'.\nStruct lines:\n{}",
            md.lines()
                .filter(|l| l.contains("status") || l.contains("Enum"))
                .collect::<Vec<_>>()
                .join("\n")
        );
        assert!(
            !md.contains("| Enum |"),
            "Should not have plain 'Enum' in struct field table.\nStruct lines:\n{}",
            md.lines()
                .filter(|l| l.contains("status") || l.contains("Enum"))
                .collect::<Vec<_>>()
                .join("\n")
        );
    }

    #[test]
    fn test_generate_schema_code_emits_default_string_value() {
        let mut properties = BTreeMap::new();
        properties.insert(
            "Path".to_string(),
            CfnProperty {
                prop_type: Some(TypeValue::Single("string".to_string())),
                description: Some("The path to the role.".to_string()),
                default_value: Some(serde_json::Value::String("/".to_string())),
                ..Default::default()
            },
        );

        let schema = CfnSchema {
            type_name: "AWS::IAM::Role".to_string(),
            description: None,
            properties,
            required: vec![],
            read_only_properties: vec![],
            create_only_properties: vec![],
            write_only_properties: vec![],
            primary_identifier: None,
            definitions: None,
            tagging: None,
            one_of: vec![],
            any_of: vec![],
            handlers: BTreeMap::new(),
        };

        let generated = generate_schema_code(&schema, "AWS::IAM::Role").unwrap();

        assert!(
            generated.contains(
                r#".with_default(Value::Concrete(ConcreteValue::String("/".to_string())))"#
            ),
            "Should emit .with_default() for string default value: {generated}"
        );
        assert!(
            generated.contains("use carina_core::resource::{ConcreteValue, Value};"),
            "Should import Value when defaults are present: {generated}"
        );
    }

    #[test]
    fn test_generate_schema_code_emits_default_bool_value() {
        let mut properties = BTreeMap::new();
        properties.insert(
            "DeletionProtectionEnabled".to_string(),
            CfnProperty {
                prop_type: Some(TypeValue::Single("boolean".to_string())),
                description: Some("Whether deletion protection is enabled.".to_string()),
                default_value: Some(serde_json::Value::Bool(false)),
                ..Default::default()
            },
        );

        let schema = CfnSchema {
            type_name: "AWS::Logs::LogGroup".to_string(),
            description: None,
            properties,
            required: vec![],
            read_only_properties: vec![],
            create_only_properties: vec![],
            write_only_properties: vec![],
            primary_identifier: None,
            definitions: None,
            tagging: None,
            one_of: vec![],
            any_of: vec![],
            handlers: BTreeMap::new(),
        };

        let generated = generate_schema_code(&schema, "AWS::Logs::LogGroup").unwrap();

        assert!(
            generated.contains(".with_default(Value::Concrete(ConcreteValue::Bool(false)))"),
            "Should emit .with_default() for boolean default value: {generated}"
        );
    }

    #[test]
    fn test_generate_schema_code_emits_default_int_value() {
        let mut properties = BTreeMap::new();
        properties.insert(
            "MaxRetries".to_string(),
            CfnProperty {
                prop_type: Some(TypeValue::Single("integer".to_string())),
                description: Some("Maximum retry count.".to_string()),
                default_value: Some(serde_json::json!(3)),
                ..Default::default()
            },
        );

        let schema = CfnSchema {
            type_name: "AWS::Test::Resource".to_string(),
            description: None,
            properties,
            required: vec![],
            read_only_properties: vec![],
            create_only_properties: vec![],
            write_only_properties: vec![],
            primary_identifier: None,
            definitions: None,
            tagging: None,
            one_of: vec![],
            any_of: vec![],
            handlers: BTreeMap::new(),
        };

        let generated = generate_schema_code(&schema, "AWS::Test::Resource").unwrap();

        assert!(
            generated.contains(".with_default(Value::Concrete(ConcreteValue::Int(3)))"),
            "Should emit .with_default() for integer default value: {generated}"
        );
    }

    #[test]
    fn test_attr_type_accepts_scalar_default() {
        assert!(attr_type_accepts_scalar_default("AttributeType::bool()"));
        assert!(attr_type_accepts_scalar_default("AttributeType::int()"));
        assert!(attr_type_accepts_scalar_default("AttributeType::float()"));
        assert!(attr_type_accepts_scalar_default(
            "AttributeType::enum_(VALID_KEY_USAGE)"
        ));
        assert!(
            attr_type_accepts_scalar_default("carina_aws_types::prefix_list_id()"),
            "scalar string types whose name merely contains 'list' must still accept defaults"
        );
        assert!(attr_type_accepts_scalar_default(
            "AttributeType::refined_int_with_validator(None, None, legacy_validator(f))"
        ));
        assert!(attr_type_accepts_scalar_default("carina_aws_types::arn()"));
        assert!(!attr_type_accepts_scalar_default(
            "carina_aws_types::iam_policy_document()"
        ));
        assert!(!attr_type_accepts_scalar_default(
            "AttributeType::struct_(\"Config\")"
        ));
        assert!(!attr_type_accepts_scalar_default(
            "AttributeType::unordered_list(AttributeType::string())"
        ));
        assert!(!attr_type_accepts_scalar_default(
            "AttributeType::map(AttributeType::string())"
        ));
        assert!(!attr_type_accepts_scalar_default(
            "AttributeType::list(AttributeType::string())"
        ));
        assert!(!attr_type_accepts_scalar_default(
            "AttributeType::ref_(\"Other\")"
        ));
    }

    #[test]
    fn test_generate_schema_code_suppresses_default_for_iam_policy_document() {
        let mut properties = BTreeMap::new();
        properties.insert(
            "KeyPolicy".to_string(),
            CfnProperty {
                prop_type: Some(TypeValue::Multiple(vec![
                    "object".to_string(),
                    "string".to_string(),
                ])),
                default_value: Some(serde_json::Value::String(
                    r#"{"Version":"2012-10-17","Statement":[]}"#.to_string(),
                )),
                ..Default::default()
            },
        );

        let schema = CfnSchema {
            type_name: "AWS::KMS::Key".to_string(),
            description: None,
            properties,
            required: vec![],
            read_only_properties: vec![],
            create_only_properties: vec![],
            write_only_properties: vec![],
            primary_identifier: None,
            definitions: None,
            tagging: None,
            one_of: vec![],
            any_of: vec![],
            handlers: BTreeMap::new(),
        };

        let generated = generate_schema_code(&schema, "AWS::KMS::Key").unwrap();

        assert!(
            generated.contains(
                r#"AttributeSchema::new("key_policy", carina_aws_types::iam_policy_document())"#
            ),
            "Should resolve KeyPolicy to iam_policy_document: {generated}"
        );
        assert!(
            !generated.contains(".with_default("),
            "Should not emit scalar JSON defaults for structured policy documents: {generated}"
        );
    }

    #[test]
    fn test_generate_markdown_shows_default_values() {
        let mut properties = BTreeMap::new();
        properties.insert(
            "Path".to_string(),
            CfnProperty {
                prop_type: Some(TypeValue::Single("string".to_string())),
                description: Some("The path.".to_string()),
                default_value: Some(serde_json::Value::String("/".to_string())),
                ..Default::default()
            },
        );
        properties.insert(
            "Enabled".to_string(),
            CfnProperty {
                prop_type: Some(TypeValue::Single("boolean".to_string())),
                description: Some("Whether enabled.".to_string()),
                default_value: Some(serde_json::Value::Bool(true)),
                ..Default::default()
            },
        );

        let schema = CfnSchema {
            type_name: "AWS::Test::Resource".to_string(),
            description: None,
            properties,
            required: vec![],
            read_only_properties: vec![],
            create_only_properties: vec![],
            write_only_properties: vec![],
            primary_identifier: None,
            definitions: None,
            tagging: None,
            one_of: vec![],
            any_of: vec![],
            handlers: BTreeMap::new(),
        };

        let md = generate_markdown(&schema, "AWS::Test::Resource").unwrap();

        assert!(
            md.contains("**Default:** `\"/\"`"),
            "Should show string default in markdown: {md}"
        );
        assert!(
            md.contains("**Default:** `true`"),
            "Should show boolean default in markdown: {md}"
        );
    }

    #[test]
    fn test_type_display_string_tags_shows_map_string() {
        let prop = CfnProperty {
            prop_type: None,
            ref_path: Some("#/definitions/Tag".to_string()),
            ..Default::default()
        };
        let schema = CfnSchema {
            type_name: "AWS::EC2::VPC".to_string(),
            description: None,
            properties: BTreeMap::new(),
            required: vec![],
            read_only_properties: vec![],
            create_only_properties: vec![],
            write_only_properties: vec![],
            primary_identifier: None,
            definitions: None,
            tagging: None,
            one_of: vec![],
            any_of: vec![],
            handlers: BTreeMap::new(),
        };
        let enums = BTreeMap::new();
        // Tags property should display Map<String, String>
        assert_eq!(
            type_display_string("Tags", &prop, &schema, &enums),
            "`Map<String, String>`"
        );
    }

    #[test]
    fn test_type_display_string_generic_object_shows_map_string() {
        let prop = CfnProperty {
            prop_type: Some(TypeValue::Single("object".to_string())),
            ..Default::default()
        };
        let schema = CfnSchema {
            type_name: "AWS::Logs::LogGroup".to_string(),
            description: None,
            properties: BTreeMap::new(),
            required: vec![],
            read_only_properties: vec![],
            create_only_properties: vec![],
            write_only_properties: vec![],
            primary_identifier: None,
            definitions: None,
            tagging: None,
            one_of: vec![],
            any_of: vec![],
            handlers: BTreeMap::new(),
        };
        let enums = BTreeMap::new();
        // Generic object should display Map<String, String>
        assert_eq!(
            type_display_string("DataProtectionPolicy", &prop, &schema, &enums),
            "`Map<String, String>`"
        );
    }

    #[test]
    fn test_generate_markdown_shows_create_only() {
        let mut properties = BTreeMap::new();
        properties.insert(
            "CidrBlock".to_string(),
            CfnProperty {
                prop_type: Some(TypeValue::Single("string".to_string())),
                description: Some("The CIDR block.".to_string()),
                ..Default::default()
            },
        );
        properties.insert(
            "Name".to_string(),
            CfnProperty {
                prop_type: Some(TypeValue::Single("string".to_string())),
                description: Some("The name.".to_string()),
                ..Default::default()
            },
        );

        let schema = CfnSchema {
            type_name: "AWS::Test::Resource".to_string(),
            description: None,
            properties,
            required: vec![],
            read_only_properties: vec![],
            create_only_properties: vec!["/properties/CidrBlock".to_string()],
            write_only_properties: vec![],
            primary_identifier: None,
            definitions: None,
            tagging: None,
            one_of: vec![],
            any_of: vec![],
            handlers: BTreeMap::new(),
        };

        let md = generate_markdown(&schema, "AWS::Test::Resource").unwrap();

        assert!(
            md.contains("**Create-only:** Yes"),
            "Should show create-only annotation for CidrBlock: {md}"
        );
        // Name should NOT have create-only
        let name_section = md.split("### `name`").nth(1).unwrap_or("");
        assert!(
            !name_section
                .split("###")
                .next()
                .unwrap_or("")
                .contains("Create-only"),
            "Name should not have create-only annotation: {md}"
        );
    }

    #[test]
    fn test_generate_markdown_read_only_description() {
        let mut properties = BTreeMap::new();
        properties.insert(
            "Id".to_string(),
            CfnProperty {
                prop_type: Some(TypeValue::Single("string".to_string())),
                description: Some("The resource ID.".to_string()),
                ..Default::default()
            },
        );

        let schema = CfnSchema {
            type_name: "AWS::Test::Resource".to_string(),
            description: None,
            properties,
            required: vec![],
            read_only_properties: vec!["/properties/Id".to_string()],
            create_only_properties: vec![],
            write_only_properties: vec![],
            primary_identifier: None,
            definitions: None,
            tagging: None,
            one_of: vec![],
            any_of: vec![],
            handlers: BTreeMap::new(),
        };

        let md = generate_markdown(&schema, "AWS::Test::Resource").unwrap();

        assert!(
            md.contains("## Attribute Reference"),
            "Should have Attribute Reference section: {md}"
        );
        assert!(
            md.contains("The resource ID."),
            "Should show description for read-only attribute: {md}"
        );
    }

    #[test]
    fn test_json_default_to_value_code() {
        assert_eq!(
            json_default_to_value_code(&serde_json::Value::String("hello".to_string())),
            Some("Value::Concrete(ConcreteValue::String(\"hello\".to_string()))".to_string())
        );
        assert_eq!(
            json_default_to_value_code(&serde_json::Value::Bool(true)),
            Some("Value::Concrete(ConcreteValue::Bool(true))".to_string())
        );
        assert_eq!(
            json_default_to_value_code(&serde_json::json!(42)),
            Some("Value::Concrete(ConcreteValue::Int(42))".to_string())
        );
        assert_eq!(json_default_to_value_code(&serde_json::Value::Null), None,);
        assert_eq!(json_default_to_value_code(&serde_json::json!([])), None,);
    }

    #[test]
    fn test_string_with_pattern_produces_custom_type() {
        let prop = CfnProperty {
            prop_type: Some(TypeValue::Single("string".to_string())),
            description: Some("The name of the log group.".to_string()),
            pattern: Some(r"^[.\-_/#A-Za-z0-9]{1,512}\Z".to_string()),
            ..Default::default()
        };
        let schema = CfnSchema {
            type_name: "AWS::Logs::LogGroup".to_string(),
            description: None,
            properties: BTreeMap::new(),
            required: vec![],
            read_only_properties: vec![],
            create_only_properties: vec![],
            write_only_properties: vec![],
            primary_identifier: None,
            definitions: None,
            tagging: None,
            one_of: vec![],
            any_of: vec![],
            handlers: BTreeMap::new(),
        };
        let (type_str, _) = cfn_type_to_carina_type_with_enum(
            &prop,
            "LogGroupName",
            &schema,
            "aws.logs.LogGroup",
            &BTreeMap::new(),
        );
        assert!(
            type_str.contains("AttributeType::refined_string("),
            "String with pattern should produce refined String type, got: {}",
            type_str
        );
        assert!(
            type_str.contains(r#"Some("^[.\\-_/#A-Za-z0-9]{1,512}\\Z".to_string())"#),
            "Should carry pattern metadata, got: {}",
            type_str
        );
    }

    #[test]
    fn test_string_with_pattern_but_known_type_skips_pattern() {
        // KmsKeyId has a pattern but is already recognized as a KMS ARN type
        let prop = CfnProperty {
            prop_type: Some(TypeValue::Single("string".to_string())),
            description: Some("The ARN of the KMS key.".to_string()),
            pattern: Some(r"^arn:[a-z0-9-]+:kms:[a-z0-9-]+:\d{12}:(key|alias)/.+\Z".to_string()),
            ..Default::default()
        };
        let schema = CfnSchema {
            type_name: "AWS::Logs::LogGroup".to_string(),
            description: None,
            properties: BTreeMap::new(),
            required: vec![],
            read_only_properties: vec![],
            create_only_properties: vec![],
            write_only_properties: vec![],
            primary_identifier: None,
            definitions: None,
            tagging: None,
            one_of: vec![],
            any_of: vec![],
            handlers: BTreeMap::new(),
        };
        let (type_str, _) = cfn_type_to_carina_type_with_enum(
            &prop,
            "KmsKeyId",
            &schema,
            "aws.logs.LogGroup",
            &BTreeMap::new(),
        );
        // KmsKeyId should be resolved to a specific ARN type, not pattern-validated
        assert!(
            !type_str.contains("validate_string_pattern_"),
            "Known type (KmsKeyId) should not get pattern validation, got: {}",
            type_str
        );
    }

    #[test]
    fn test_pattern_validation_generated_in_schema_code() {
        let mut properties = BTreeMap::new();
        properties.insert(
            "LogGroupName".to_string(),
            CfnProperty {
                prop_type: Some(TypeValue::Single("string".to_string())),
                description: Some("The name of the log group.".to_string()),
                pattern: Some(r"^[.\-_/#A-Za-z0-9]{1,512}\Z".to_string()),
                ..Default::default()
            },
        );

        let schema = CfnSchema {
            type_name: "AWS::Logs::LogGroup".to_string(),
            description: None,
            properties,
            required: vec![],
            read_only_properties: vec![],
            create_only_properties: vec![],
            write_only_properties: vec![],
            primary_identifier: None,
            definitions: None,
            tagging: None,
            one_of: vec![],
            any_of: vec![],
            handlers: BTreeMap::new(),
        };

        let code = generate_schema_code(&schema, "AWS::Logs::LogGroup").unwrap();
        assert!(
            code.contains("fn validate_string_pattern_"),
            "Generated code should contain pattern validation function:\n{}",
            code
        );
        assert!(
            code.contains("Regex::new"),
            "Pattern validation should use Regex:\n{}",
            code
        );
    }

    #[test]
    fn test_array_with_min_max_items_generates_custom_list_type() {
        // Array with minItems/maxItems should generate a Custom type wrapping List
        let prop = CfnProperty {
            prop_type: Some(TypeValue::Single("array".to_string())),
            items: Some(Box::new(CfnProperty {
                prop_type: Some(TypeValue::Single("string".to_string())),
                ..Default::default()
            })),
            min_items: Some(1),
            max_items: Some(10),
            ..Default::default()
        };
        let schema = CfnSchema {
            type_name: "AWS::EC2::VPCEndpoint".to_string(),
            description: None,
            properties: BTreeMap::new(),
            required: vec![],
            read_only_properties: vec![],
            create_only_properties: vec![],
            write_only_properties: vec![],
            primary_identifier: None,
            definitions: None,
            tagging: None,
            one_of: vec![],
            any_of: vec![],
            handlers: BTreeMap::new(),
        };
        let enums = BTreeMap::new();
        let (type_str, _) = cfn_type_to_carina_type_with_enum(
            &prop,
            "PrivateDnsSpecifiedDomains",
            &schema,
            "aws.ec2.VpcEndpoint",
            &enums,
        );
        assert!(
            type_str.contains("AttributeType::refined_list("),
            "array with minItems/maxItems should produce refined list type: {type_str}"
        );
        assert!(
            type_str.contains("AttributeType::string(), true, Some((Some(1), Some(10)))"),
            "refined list should carry element type, ordering, and length: {type_str}"
        );
        assert!(
            type_str.contains("validate_list_items_1_10"),
            "should reference items validation function: {type_str}"
        );
    }

    #[test]
    fn test_array_with_only_min_items_generates_custom_list_type() {
        let prop = CfnProperty {
            prop_type: Some(TypeValue::Single("array".to_string())),
            items: Some(Box::new(CfnProperty {
                prop_type: Some(TypeValue::Single("string".to_string())),
                ..Default::default()
            })),
            min_items: Some(1),
            max_items: None,
            ..Default::default()
        };
        let schema = CfnSchema {
            type_name: "AWS::EC2::VPCEndpoint".to_string(),
            description: None,
            properties: BTreeMap::new(),
            required: vec![],
            read_only_properties: vec![],
            create_only_properties: vec![],
            write_only_properties: vec![],
            primary_identifier: None,
            definitions: None,
            tagging: None,
            one_of: vec![],
            any_of: vec![],
            handlers: BTreeMap::new(),
        };
        let enums = BTreeMap::new();
        let (type_str, _) = cfn_type_to_carina_type_with_enum(
            &prop,
            "SomeArray",
            &schema,
            "aws.ec2.VpcEndpoint",
            &enums,
        );
        assert!(
            type_str.contains("AttributeType::refined_list("),
            "array with only minItems should produce refined list type: {type_str}"
        );
        assert!(
            type_str.contains("validate_list_items_min_1"),
            "should reference min-items validator: {type_str}"
        );
    }

    #[test]
    fn test_array_without_min_max_items_produces_plain_list() {
        let prop = CfnProperty {
            prop_type: Some(TypeValue::Single("array".to_string())),
            items: Some(Box::new(CfnProperty {
                prop_type: Some(TypeValue::Single("string".to_string())),
                ..Default::default()
            })),
            min_items: None,
            max_items: None,
            ..Default::default()
        };
        let schema = CfnSchema {
            type_name: "AWS::EC2::VPCEndpoint".to_string(),
            description: None,
            properties: BTreeMap::new(),
            required: vec![],
            read_only_properties: vec![],
            create_only_properties: vec![],
            write_only_properties: vec![],
            primary_identifier: None,
            definitions: None,
            tagging: None,
            one_of: vec![],
            any_of: vec![],
            handlers: BTreeMap::new(),
        };
        let enums = BTreeMap::new();
        let (type_str, _) = cfn_type_to_carina_type_with_enum(
            &prop,
            "SomeArray",
            &schema,
            "aws.ec2.VpcEndpoint",
            &enums,
        );
        assert!(
            !type_str.contains("custom("),
            "array without minItems/maxItems should not produce Custom type: {type_str}"
        );
        assert!(
            type_str.contains("::list(") || type_str.contains("::unordered_list("),
            "should produce plain list type: {type_str}"
        );
    }

    #[test]
    fn test_generate_schema_code_emits_list_items_validation() {
        // Test that generate_schema_code emits validation functions for array properties
        // with minItems/maxItems constraints (via struct field in a definition)
        let mut dns_props = BTreeMap::new();
        dns_props.insert(
            "PrivateDnsSpecifiedDomains".to_string(),
            CfnProperty {
                prop_type: Some(TypeValue::Single("array".to_string())),
                items: Some(Box::new(CfnProperty {
                    prop_type: Some(TypeValue::Single("string".to_string())),
                    ..Default::default()
                })),
                min_items: Some(1),
                max_items: Some(10),
                ..Default::default()
            },
        );

        let mut definitions = BTreeMap::new();
        definitions.insert(
            "DnsOptionsSpecification".to_string(),
            CfnDefinition {
                def_type: Some("object".to_string()),
                properties: Some(dns_props),
                required: vec![],
                one_of: vec![],
                pattern: None,
                items: None,
                enum_values: None,
                min_length: None,
                max_length: None,
                minimum: None,
                maximum: None,
                format: None,
            },
        );

        let mut properties = BTreeMap::new();
        properties.insert(
            "DnsOptions".to_string(),
            CfnProperty {
                ref_path: Some("#/definitions/DnsOptionsSpecification".to_string()),
                ..Default::default()
            },
        );

        let schema = CfnSchema {
            type_name: "AWS::EC2::VPCEndpoint".to_string(),
            description: Some("Test".to_string()),
            properties,
            required: vec![],
            read_only_properties: vec![],
            create_only_properties: vec![],
            write_only_properties: vec![],
            primary_identifier: None,
            definitions: Some(definitions),
            tagging: None,
            one_of: vec![],
            any_of: vec![],
            handlers: BTreeMap::new(),
        };

        let generated = generate_schema_code(&schema, "AWS::EC2::VPCEndpoint").unwrap();
        assert!(
            generated.contains("validate_list_items_1_10"),
            "should generate items validation function: {generated}"
        );
        assert!(
            generated.contains("Value::Concrete(ConcreteValue::List(items))"),
            "validation function should check Value::List: {generated}"
        );
        assert!(
            generated.contains("1..=10"),
            "should show range 1..=10 in validation: {generated}"
        );
    }

    #[test]
    fn test_generate_schema_code_string_min_max_length() {
        let mut properties = BTreeMap::new();
        properties.insert(
            "LogGroupName".to_string(),
            CfnProperty {
                prop_type: Some(TypeValue::Single("string".to_string())),
                description: Some("The name of the log group.".to_string()),
                min_length: Some(1),
                max_length: Some(512),
                ..Default::default()
            },
        );

        let schema = CfnSchema {
            type_name: "AWS::Logs::LogGroup".to_string(),
            description: None,
            properties,
            required: vec![],
            read_only_properties: vec![],
            create_only_properties: vec![],
            write_only_properties: vec![],
            primary_identifier: None,
            definitions: None,
            tagging: None,
            one_of: vec![],
            any_of: vec![],
            handlers: BTreeMap::new(),
        };

        let generated = generate_schema_code(&schema, "AWS::Logs::LogGroup").unwrap();

        // Should generate a length validation function
        assert!(
            generated.contains("validate_string_length_1_512"),
            "Should generate length validation function: {generated}"
        );

        // Should use AttributeType::Custom wrapping String
        assert!(
            generated.contains(
                "AttributeType::refined_string(None, None, Some((Some(1), Some(512))), None)"
            ),
            "Should wrap String in Custom type with length constraint: {generated}"
        );

        // Should display length range
        assert!(
            generated.contains("1..=512"),
            "Should display length range: {generated}"
        );
    }

    #[test]
    fn test_generate_schema_code_string_min_length_only() {
        let mut properties = BTreeMap::new();
        properties.insert(
            "ResourceName".to_string(),
            CfnProperty {
                prop_type: Some(TypeValue::Single("string".to_string())),
                description: Some("Name of the resource.".to_string()),
                min_length: Some(1),
                ..Default::default()
            },
        );

        let schema = CfnSchema {
            type_name: "AWS::Test::Resource".to_string(),
            description: None,
            properties,
            required: vec![],
            read_only_properties: vec![],
            create_only_properties: vec![],
            write_only_properties: vec![],
            primary_identifier: None,
            definitions: None,
            tagging: None,
            one_of: vec![],
            any_of: vec![],
            handlers: BTreeMap::new(),
        };

        let generated = generate_schema_code(&schema, "AWS::Test::Resource").unwrap();

        assert!(
            generated.contains("validate_string_length_min_1"),
            "Should generate length validation function for min-only: {generated}"
        );
        assert!(
            generated.contains("1.."),
            "Should display open-ended range for min-only: {generated}"
        );
    }

    #[test]
    fn test_generate_schema_code_string_max_length_only() {
        let mut properties = BTreeMap::new();
        properties.insert(
            "Description".to_string(),
            CfnProperty {
                prop_type: Some(TypeValue::Single("string".to_string())),
                description: Some("A description.".to_string()),
                max_length: Some(256),
                ..Default::default()
            },
        );

        let schema = CfnSchema {
            type_name: "AWS::Test::Resource".to_string(),
            description: None,
            properties,
            required: vec![],
            read_only_properties: vec![],
            create_only_properties: vec![],
            write_only_properties: vec![],
            primary_identifier: None,
            definitions: None,
            tagging: None,
            one_of: vec![],
            any_of: vec![],
            handlers: BTreeMap::new(),
        };

        let generated = generate_schema_code(&schema, "AWS::Test::Resource").unwrap();

        assert!(
            generated.contains("validate_string_length_max_256"),
            "Should generate length validation function for max-only: {generated}"
        );
        assert!(
            generated.contains("..=256"),
            "Should display open-ended range for max-only: {generated}"
        );
    }

    #[test]
    fn test_is_email_property_matches_expected_names() {
        // Exact match (case-insensitive)
        assert!(is_email_property("Email"));
        assert!(is_email_property("email"));
        // EmailAddress suffix
        assert!(is_email_property("EmailAddress"));
        assert!(is_email_property("PrimaryEmailAddress"));
        assert!(is_email_property("ContactEmailAddress"));
        // Should NOT match — these are flags / categories, not addresses
        assert!(!is_email_property("EmailEnabled"));
        assert!(!is_email_property("EmailType"));
        assert!(!is_email_property("EmailVerified"));
        assert!(!is_email_property("Name"));
    }

    #[test]
    fn test_infer_string_type_email() {
        assert_eq!(
            infer_string_type("Email", ""),
            Some("types::email()".to_string())
        );
        assert_eq!(
            infer_string_type("EmailAddress", ""),
            Some("types::email()".to_string())
        );
        assert_eq!(
            infer_string_type("ContactEmailAddress", ""),
            Some("types::email()".to_string())
        );
        // Negative cases
        assert_eq!(infer_string_type("EmailEnabled", ""), None);
    }

    #[test]
    fn test_generate_schema_code_email_uses_types_email() {
        // Mirrors the AWS::Organizations::Account "Email" property: a CFN
        // string with a pattern + length range. The generated code must use
        // `types::email()` instead of an ad-hoc Custom validator.
        let mut properties = BTreeMap::new();
        properties.insert(
            "Email".to_string(),
            CfnProperty {
                prop_type: Some(TypeValue::Single("string".to_string())),
                description: Some(
                    "The email address of the owner to assign to the new member account."
                        .to_string(),
                ),
                pattern: Some("[^\\s@]+@[^\\s@]+\\.[^\\s@]+".to_string()),
                min_length: Some(6),
                max_length: Some(64),
                ..Default::default()
            },
        );

        let schema = CfnSchema {
            type_name: "AWS::Organizations::Account".to_string(),
            description: None,
            properties,
            required: vec!["Email".to_string()],
            read_only_properties: vec![],
            create_only_properties: vec![],
            write_only_properties: vec![],
            primary_identifier: None,
            definitions: None,
            tagging: None,
            one_of: vec![],
            any_of: vec![],
            handlers: BTreeMap::new(),
        };

        let generated = generate_schema_code(&schema, "AWS::Organizations::Account").unwrap();

        assert!(
            generated.contains("types::email()"),
            "Expected types::email() in generated code: {generated}"
        );
        assert!(
            !generated.contains("validate_string_pattern_ec4d9bee0dcd262b_len_6_64"),
            "Should not emit ad-hoc pattern validator for email: {generated}"
        );
        assert!(
            !generated.contains("[^\\s@]+@[^\\s@]+"),
            "Should not embed the email regex pattern in generated code: {generated}"
        );
    }

    #[test]
    fn test_ref_to_definition_with_pattern_produces_custom_type() {
        // When a property uses $ref to a definition that is a simple string with a pattern,
        // the pattern should be propagated to the referencing property.
        let prop = CfnProperty {
            ref_path: Some("#/definitions/iso8601UTC".to_string()),
            description: Some("Indicates when objects are deleted.".to_string()),
            ..Default::default()
        };

        let mut definitions = BTreeMap::new();
        definitions.insert(
            "iso8601UTC".to_string(),
            CfnDefinition {
                def_type: Some("string".to_string()),
                pattern: Some(
                    r"^(\d{4})-(0[0-9]|1[0-2])-([0-2]\d|3[01])T([01]\d|2[0-4]):([0-5]\d):([0-6]\d)((\.\d{3})?)Z$".to_string(),
                ),
                ..Default::default()
            },
        );

        let schema = CfnSchema {
            type_name: "AWS::S3::Bucket".to_string(),
            description: None,
            properties: BTreeMap::new(),
            required: vec![],
            read_only_properties: vec![],
            create_only_properties: vec![],
            write_only_properties: vec![],
            primary_identifier: None,
            definitions: Some(definitions),
            tagging: None,
            one_of: vec![],
            any_of: vec![],
            handlers: BTreeMap::new(),
        };

        let (type_str, _) = cfn_type_to_carina_type_with_enum(
            &prop,
            "ExpirationDate",
            &schema,
            "aws.s3.Bucket",
            &BTreeMap::new(),
        );
        assert!(
            type_str.contains("AttributeType::refined_string("),
            "Ref to string definition with pattern should produce refined String type, got: {}",
            type_str
        );
        assert!(
            type_str.contains(r#"Some("^(\\d{4})"#),
            "Should carry pattern metadata, got: {}",
            type_str
        );
    }

    #[test]
    fn test_ref_pattern_collected_in_generate_schema_code() {
        // When a struct field uses $ref to a definition with a pattern,
        // the pattern validation function should be generated in the schema code.
        let mut rule_props = BTreeMap::new();
        rule_props.insert(
            "ExpirationDate".to_string(),
            CfnProperty {
                ref_path: Some("#/definitions/iso8601UTC".to_string()),
                description: Some("Indicates when objects are deleted.".to_string()),
                ..Default::default()
            },
        );
        rule_props.insert(
            "Status".to_string(),
            CfnProperty {
                prop_type: Some(TypeValue::Single("string".to_string())),
                enum_values: Some(vec![
                    EnumValue::Str("Enabled".to_string()),
                    EnumValue::Str("Disabled".to_string()),
                ]),
                ..Default::default()
            },
        );

        let mut definitions = BTreeMap::new();
        definitions.insert(
            "iso8601UTC".to_string(),
            CfnDefinition {
                def_type: Some("string".to_string()),
                pattern: Some(
                    r"^(\d{4})-(0[0-9]|1[0-2])-([0-2]\d|3[01])T([01]\d|2[0-4]):([0-5]\d):([0-6]\d)((\.\d{3})?)Z$".to_string(),
                ),
                ..Default::default()
            },
        );
        definitions.insert(
            "Rule".to_string(),
            CfnDefinition {
                def_type: Some("object".to_string()),
                properties: Some(rule_props),
                required: vec!["Status".to_string()],
                ..Default::default()
            },
        );

        let mut properties = BTreeMap::new();
        properties.insert(
            "LifecycleConfiguration".to_string(),
            CfnProperty {
                prop_type: Some(TypeValue::Single("object".to_string())),
                properties: Some({
                    let mut lc_props = BTreeMap::new();
                    lc_props.insert(
                        "Rules".to_string(),
                        CfnProperty {
                            prop_type: Some(TypeValue::Single("array".to_string())),
                            items: Some(Box::new(CfnProperty {
                                ref_path: Some("#/definitions/Rule".to_string()),
                                ..Default::default()
                            })),
                            ..Default::default()
                        },
                    );
                    lc_props
                }),
                ..Default::default()
            },
        );

        let schema = CfnSchema {
            type_name: "AWS::S3::Bucket".to_string(),
            description: None,
            properties,
            required: vec![],
            read_only_properties: vec![],
            create_only_properties: vec![],
            write_only_properties: vec![],
            primary_identifier: None,
            definitions: Some(definitions),
            tagging: None,
            one_of: vec![],
            any_of: vec![],
            handlers: BTreeMap::new(),
        };

        let generated = generate_schema_code(&schema, "AWS::S3::Bucket").unwrap();
        assert!(
            generated.contains("fn validate_string_pattern_"),
            "Should generate hash-based pattern validation function for $ref definition pattern: {generated}"
        );
    }

    #[test]
    fn test_top_level_ref_pattern_collected_in_generate_schema_code() {
        // When a top-level property uses $ref to a definition with a pattern,
        // the pattern validation function should be generated.
        let mut definitions = BTreeMap::new();
        definitions.insert(
            "iso8601UTC".to_string(),
            CfnDefinition {
                def_type: Some("string".to_string()),
                pattern: Some(r"^(\d{4})-(0[0-9]|1[0-2])$".to_string()),
                ..Default::default()
            },
        );

        let mut properties = BTreeMap::new();
        properties.insert(
            "CreatedDate".to_string(),
            CfnProperty {
                ref_path: Some("#/definitions/iso8601UTC".to_string()),
                description: Some("When the resource was created.".to_string()),
                ..Default::default()
            },
        );

        let schema = CfnSchema {
            type_name: "AWS::Test::Resource".to_string(),
            description: None,
            properties,
            required: vec![],
            read_only_properties: vec![],
            create_only_properties: vec![],
            write_only_properties: vec![],
            primary_identifier: None,
            definitions: Some(definitions),
            tagging: None,
            one_of: vec![],
            any_of: vec![],
            handlers: BTreeMap::new(),
        };

        let generated = generate_schema_code(&schema, "AWS::Test::Resource").unwrap();
        assert!(
            generated.contains("fn validate_string_pattern_"),
            "Should generate hash-based pattern validation function for top-level $ref pattern: {generated}"
        );
    }

    #[test]
    fn test_dedup_string_length_validation_functions() {
        // Two properties with identical maxLength constraints should share one validation function
        let mut properties = BTreeMap::new();
        properties.insert(
            "Name".to_string(),
            CfnProperty {
                prop_type: Some(TypeValue::Single("string".to_string())),
                description: Some("Name field".to_string()),
                max_length: Some(1024),
                ..Default::default()
            },
        );
        properties.insert(
            "Prefix".to_string(),
            CfnProperty {
                prop_type: Some(TypeValue::Single("string".to_string())),
                description: Some("Prefix field".to_string()),
                max_length: Some(1024),
                ..Default::default()
            },
        );

        let schema = CfnSchema {
            type_name: "AWS::Test::Resource".to_string(),
            description: None,
            properties,
            required: vec![],
            read_only_properties: vec![],
            create_only_properties: vec![],
            write_only_properties: vec![],
            primary_identifier: None,
            definitions: None,
            tagging: None,
            one_of: vec![],
            any_of: vec![],
            handlers: BTreeMap::new(),
        };

        let generated = generate_schema_code(&schema, "AWS::Test::Resource").unwrap();

        // Should NOT have per-property validation functions
        assert!(
            !generated.contains("fn validate_name_length"),
            "Should not generate per-property length function: {generated}"
        );
        assert!(
            !generated.contains("fn validate_prefix_length"),
            "Should not generate per-property length function: {generated}"
        );

        // Should have a single shared validation function named by constraints
        let fn_count = generated.matches("fn validate_string_length_").count();
        assert_eq!(
            fn_count, 1,
            "Should generate exactly one shared length validation function, got {fn_count}: {generated}"
        );

        // Both properties should reference the same validation function
        let ref_count = generated.matches("validate_string_length_max_1024").count();
        assert!(
            ref_count == 1,
            "Both properties should reference the shared function (1 def; refined String carries length directly), got {ref_count}: {generated}"
        );
    }

    #[test]
    fn test_non_standard_tag_definition_generates_validators() {
        // Non-standard tag definitions like "HostedZoneTag" should have their
        // string length validators generated. Only the standard "Tag" definition
        // is skipped (handled by tags_type()).
        let mut properties = BTreeMap::new();
        properties.insert(
            "HostedZoneTags".to_string(),
            CfnProperty {
                prop_type: Some(TypeValue::Single("array".to_string())),
                description: Some("Tags".to_string()),
                insertion_order: Some(false),
                items: Some(Box::new(CfnProperty {
                    ref_path: Some("#/definitions/HostedZoneTag".to_string()),
                    ..Default::default()
                })),
                ..Default::default()
            },
        );

        let mut definitions = BTreeMap::new();
        let mut tag_props = BTreeMap::new();
        tag_props.insert(
            "Key".to_string(),
            CfnProperty {
                prop_type: Some(TypeValue::Single("string".to_string())),
                max_length: Some(128),
                ..Default::default()
            },
        );
        tag_props.insert(
            "Value".to_string(),
            CfnProperty {
                prop_type: Some(TypeValue::Single("string".to_string())),
                max_length: Some(256),
                ..Default::default()
            },
        );
        definitions.insert(
            "HostedZoneTag".to_string(),
            CfnDefinition {
                properties: Some(tag_props),
                required: vec!["Key".to_string(), "Value".to_string()],
                ..Default::default()
            },
        );

        let schema = CfnSchema {
            type_name: "AWS::Route53::HostedZone".to_string(),
            description: None,
            properties,
            required: vec![],
            read_only_properties: vec![],
            create_only_properties: vec![],
            write_only_properties: vec![],
            primary_identifier: None,
            definitions: Some(definitions),
            tagging: None,
            one_of: vec![],
            any_of: vec![],
            handlers: BTreeMap::new(),
        };

        let generated = generate_schema_code(&schema, "AWS::Route53::HostedZone").unwrap();

        // The HostedZoneTag struct fields should have validators generated
        assert!(
            generated.contains("fn validate_string_length_max_128"),
            "Should generate validate_string_length_max_128 for HostedZoneTag.Key: {generated}"
        );
        assert!(
            generated.contains("fn validate_string_length_max_256"),
            "Should generate validate_string_length_max_256 for HostedZoneTag.Value: {generated}"
        );
    }

    #[test]
    fn test_route53_hosted_zone_name_emits_refined_string_strip_suffix() {
        let schema = cfn_schema_for_codegen_tests("AWS::Route53::HostedZone");
        let generated = generate_schema_code(&schema, "AWS::Route53::HostedZone").unwrap();

        assert!(
            generated.contains(
                r#"AttributeSchema::new("name", AttributeType::refined_string(None, None, Some((None, Some(1024))), Some(carina_core::schema::DslTransform::StripSuffix(".".to_string()))))"#
            ),
            "Route53 HostedZone.Name should be refined String with max length and StripSuffix transform: {generated}"
        );
        assert!(
            !generated.contains(r#"AttributeSchema::new("name", AttributeType::enum_"#),
            "Route53 HostedZone.Name must not be emitted as a pseudo enum: {generated}"
        );
    }

    #[test]
    fn test_string_length_validation_uses_char_count() {
        // String length validation should use s.chars().count() (character count)
        // instead of s.len() (byte count) to correctly handle multi-byte characters.
        // e.g., "テスト" is 3 characters but 9 bytes in UTF-8.
        let mut properties = BTreeMap::new();
        properties.insert(
            "DisplayName".to_string(),
            CfnProperty {
                prop_type: Some(TypeValue::Single("string".to_string())),
                description: Some("Display name.".to_string()),
                max_length: Some(10),
                ..Default::default()
            },
        );

        let schema = CfnSchema {
            type_name: "AWS::Test::Resource".to_string(),
            description: None,
            properties,
            required: vec![],
            read_only_properties: vec![],
            create_only_properties: vec![],
            write_only_properties: vec![],
            primary_identifier: None,
            definitions: None,
            tagging: None,
            one_of: vec![],
            any_of: vec![],
            handlers: BTreeMap::new(),
        };

        let generated = generate_schema_code(&schema, "AWS::Test::Resource").unwrap();

        // Should use chars().count() for character-based length, not byte-based len()
        assert!(
            generated.contains("s.chars().count()"),
            "String length validation should use s.chars().count() instead of s.len(): {generated}"
        );
        assert!(
            !generated.contains("let len = s.len()"),
            "String length validation should NOT use s.len() for string properties: {generated}"
        );
    }

    #[test]
    fn test_dedup_pattern_validation_functions() {
        // Two properties with identical pattern constraints should share one validation function
        let mut properties = BTreeMap::new();
        properties.insert(
            "ObjectSizeGreaterThan".to_string(),
            CfnProperty {
                prop_type: Some(TypeValue::Single("string".to_string())),
                pattern: Some("[0-9]+".to_string()),
                ..Default::default()
            },
        );
        properties.insert(
            "ObjectSizeLessThan".to_string(),
            CfnProperty {
                prop_type: Some(TypeValue::Single("string".to_string())),
                pattern: Some("[0-9]+".to_string()),
                ..Default::default()
            },
        );

        let schema = CfnSchema {
            type_name: "AWS::Test::Resource".to_string(),
            description: None,
            properties,
            required: vec![],
            read_only_properties: vec![],
            create_only_properties: vec![],
            write_only_properties: vec![],
            primary_identifier: None,
            definitions: None,
            tagging: None,
            one_of: vec![],
            any_of: vec![],
            handlers: BTreeMap::new(),
        };

        let generated = generate_schema_code(&schema, "AWS::Test::Resource").unwrap();

        // Should NOT have per-property pattern validation functions
        assert!(
            !generated.contains("fn validate_object_size_greater_than_pattern"),
            "Should not generate per-property pattern function: {generated}"
        );
        assert!(
            !generated.contains("fn validate_object_size_less_than_pattern"),
            "Should not generate per-property pattern function: {generated}"
        );

        // Should have exactly one shared pattern validation function
        let fn_count = generated.matches("fn validate_string_pattern_").count();
        assert_eq!(
            fn_count, 1,
            "Should generate exactly one shared pattern validation function, got {fn_count}: {generated}"
        );
    }

    #[test]
    fn test_dedup_list_items_validation_functions() {
        // Two array properties with identical minItems/maxItems should share one validation function
        let mut def_props = BTreeMap::new();
        def_props.insert(
            "AllowedOrigins".to_string(),
            CfnProperty {
                prop_type: Some(TypeValue::Single("array".to_string())),
                items: Some(Box::new(CfnProperty {
                    prop_type: Some(TypeValue::Single("string".to_string())),
                    ..Default::default()
                })),
                min_items: Some(1),
                max_items: Some(100),
                ..Default::default()
            },
        );
        def_props.insert(
            "AllowedHeaders".to_string(),
            CfnProperty {
                prop_type: Some(TypeValue::Single("array".to_string())),
                items: Some(Box::new(CfnProperty {
                    prop_type: Some(TypeValue::Single("string".to_string())),
                    ..Default::default()
                })),
                min_items: Some(1),
                max_items: Some(100),
                ..Default::default()
            },
        );

        let mut definitions = BTreeMap::new();
        definitions.insert(
            "CorsRule".to_string(),
            CfnDefinition {
                def_type: Some("object".to_string()),
                properties: Some(def_props),
                required: vec![],
                one_of: vec![],
                items: None,
                enum_values: None,
                pattern: None,
                min_length: None,
                max_length: None,
                minimum: None,
                maximum: None,
                format: None,
            },
        );

        let mut properties = BTreeMap::new();
        properties.insert(
            "CorsRules".to_string(),
            CfnProperty {
                ref_path: Some("#/definitions/CorsRule".to_string()),
                ..Default::default()
            },
        );

        let schema = CfnSchema {
            type_name: "AWS::Test::Resource".to_string(),
            description: None,
            properties,
            required: vec![],
            read_only_properties: vec![],
            create_only_properties: vec![],
            write_only_properties: vec![],
            primary_identifier: None,
            definitions: Some(definitions),
            tagging: None,
            one_of: vec![],
            any_of: vec![],
            handlers: BTreeMap::new(),
        };

        let generated = generate_schema_code(&schema, "AWS::Test::Resource").unwrap();

        // Should NOT have per-property items validation functions
        assert!(
            !generated.contains("fn validate_allowed_origins_items"),
            "Should not generate per-property items function: {generated}"
        );
        assert!(
            !generated.contains("fn validate_allowed_headers_items"),
            "Should not generate per-property items function: {generated}"
        );

        // Should have exactly one shared items validation function
        let fn_count = generated.matches("fn validate_list_items_").count();
        assert_eq!(
            fn_count, 1,
            "Should generate exactly one shared items validation function, got {fn_count}: {generated}"
        );
    }

    #[test]
    fn test_integer_format_int64_generates_int_with_format() {
        // "type": "integer", "format": "int64" should include int64 in type name
        let prop = CfnProperty {
            prop_type: Some(TypeValue::Single("integer".to_string())),
            description: Some("A 64-bit integer value".to_string()),
            format: Some("int64".to_string()),
            ..CfnProperty::default()
        };
        let schema = CfnSchema {
            type_name: "AWS::Test::Resource".to_string(),
            description: None,
            properties: BTreeMap::new(),
            required: vec![],
            read_only_properties: vec![],
            create_only_properties: vec![],
            write_only_properties: vec![],
            primary_identifier: None,
            definitions: None,
            tagging: None,
            one_of: vec![],
            any_of: vec![],
            handlers: BTreeMap::new(),
        };
        let (type_str, _) =
            cfn_type_to_carina_type_with_enum(&prop, "SomeValue", &schema, "", &BTreeMap::new());
        assert!(
            type_str.contains("AttributeType::refined_int("),
            "int64 format should produce refined Int type: {type_str}"
        );
        assert!(
            type_str.contains("AttributeType::refined_int("),
            "int64 format should use refined Int: {type_str}"
        );
    }

    #[test]
    fn test_integer_format_int64_display() {
        // Markdown display for int64 should show Int(int64)
        let prop = CfnProperty {
            prop_type: Some(TypeValue::Single("integer".to_string())),
            description: Some("A 64-bit integer value".to_string()),
            format: Some("int64".to_string()),
            ..CfnProperty::default()
        };
        let schema = CfnSchema {
            type_name: "AWS::Test::Resource".to_string(),
            description: None,
            properties: BTreeMap::new(),
            required: vec![],
            read_only_properties: vec![],
            create_only_properties: vec![],
            write_only_properties: vec![],
            primary_identifier: None,
            definitions: None,
            tagging: None,
            one_of: vec![],
            any_of: vec![],
            handlers: BTreeMap::new(),
        };
        let display = type_display_string("SomeValue", &prop, &schema, &BTreeMap::new());
        assert!(
            display.contains("int64"),
            "int64 format should appear in display: {display}"
        );
    }

    #[test]
    fn test_string_format_uri_type() {
        // "type": "string", "format": "uri" should produce String(uri) type
        let prop = CfnProperty {
            prop_type: Some(TypeValue::Single("string".to_string())),
            description: Some("A URL".to_string()),
            format: Some("uri".to_string()),
            ..CfnProperty::default()
        };
        let schema = CfnSchema {
            type_name: "AWS::Test::Resource".to_string(),
            description: None,
            properties: BTreeMap::new(),
            required: vec![],
            read_only_properties: vec![],
            create_only_properties: vec![],
            write_only_properties: vec![],
            primary_identifier: None,
            definitions: None,
            tagging: None,
            one_of: vec![],
            any_of: vec![],
            handlers: BTreeMap::new(),
        };
        let (type_str, _) =
            cfn_type_to_carina_type_with_enum(&prop, "Url", &schema, "", &BTreeMap::new());
        assert!(
            type_str.contains("AttributeType::refined_string("),
            "uri format should produce refined String type: {type_str}"
        );
        assert!(
            type_str.contains("AttributeType::refined_string("),
            "uri format should use refined String: {type_str}"
        );
    }

    #[test]
    fn test_string_format_uri_display() {
        // "type": "string", "format": "uri" should show String(uri) in display
        let prop = CfnProperty {
            prop_type: Some(TypeValue::Single("string".to_string())),
            description: Some("A URL".to_string()),
            format: Some("uri".to_string()),
            ..CfnProperty::default()
        };
        let schema = CfnSchema {
            type_name: "AWS::Test::Resource".to_string(),
            description: None,
            properties: BTreeMap::new(),
            required: vec![],
            read_only_properties: vec![],
            create_only_properties: vec![],
            write_only_properties: vec![],
            primary_identifier: None,
            definitions: None,
            tagging: None,
            one_of: vec![],
            any_of: vec![],
            handlers: BTreeMap::new(),
        };
        let display = type_display_string("Url", &prop, &schema, &BTreeMap::new());
        assert!(
            display.contains("uri"),
            "uri format should appear in display: {display}"
        );
    }

    #[test]
    fn test_numeric_string_pattern_generates_numeric_string_type() {
        // "type": "string", "pattern": "[0-9]+" should be treated as numeric string
        let prop = CfnProperty {
            prop_type: Some(TypeValue::Single("string".to_string())),
            description: Some("Object size in bytes".to_string()),
            pattern: Some("[0-9]+".to_string()),
            max_length: Some(20),
            ..CfnProperty::default()
        };
        let schema = CfnSchema {
            type_name: "AWS::S3::Bucket".to_string(),
            description: None,
            properties: BTreeMap::new(),
            required: vec![],
            read_only_properties: vec![],
            create_only_properties: vec![],
            write_only_properties: vec![],
            primary_identifier: None,
            definitions: None,
            tagging: None,
            one_of: vec![],
            any_of: vec![],
            handlers: BTreeMap::new(),
        };
        let (type_str, _) = cfn_type_to_carina_type_with_enum(
            &prop,
            "ObjectSizeGreaterThan",
            &schema,
            "",
            &BTreeMap::new(),
        );
        assert!(
            type_str.contains("Some(\"[0-9]+\".to_string())"),
            "Numeric string pattern should be captured in Custom.pattern: {type_str}"
        );
        assert!(
            type_str.contains("Some((None, Some(20)))"),
            "maxLength=20 should be captured in Custom.length: {type_str}"
        );
    }

    #[test]
    fn test_numeric_string_pattern_display() {
        // Markdown display for numeric string pattern
        let prop = CfnProperty {
            prop_type: Some(TypeValue::Single("string".to_string())),
            description: Some("Object size in bytes".to_string()),
            pattern: Some("[0-9]+".to_string()),
            max_length: Some(20),
            ..CfnProperty::default()
        };
        let schema = CfnSchema {
            type_name: "AWS::S3::Bucket".to_string(),
            description: None,
            properties: BTreeMap::new(),
            required: vec![],
            read_only_properties: vec![],
            create_only_properties: vec![],
            write_only_properties: vec![],
            primary_identifier: None,
            definitions: None,
            tagging: None,
            one_of: vec![],
            any_of: vec![],
            handlers: BTreeMap::new(),
        };
        let display =
            type_display_string("ObjectSizeGreaterThan", &prop, &schema, &BTreeMap::new());
        assert!(
            display.contains("NumericString"),
            "Numeric string should show NumericString in display: {display}"
        );
    }

    #[test]
    fn test_number_format_double_display() {
        // "type": "number", "format": "double" should show Float(double) in display
        let prop = CfnProperty {
            prop_type: Some(TypeValue::Single("number".to_string())),
            description: Some("A floating point value".to_string()),
            format: Some("double".to_string()),
            ..CfnProperty::default()
        };
        let schema = CfnSchema {
            type_name: "AWS::Test::Resource".to_string(),
            description: None,
            properties: BTreeMap::new(),
            required: vec![],
            read_only_properties: vec![],
            create_only_properties: vec![],
            write_only_properties: vec![],
            primary_identifier: None,
            definitions: None,
            tagging: None,
            one_of: vec![],
            any_of: vec![],
            handlers: BTreeMap::new(),
        };
        let display = type_display_string("Value", &prop, &schema, &BTreeMap::new());
        assert!(
            display.contains("double"),
            "double format should appear in display: {display}"
        );
    }

    #[test]
    fn test_integer_format_int64_with_range_combines() {
        // "type": "integer", "format": "int64", "minimum": 0, "maximum": 100
        // should show both the format and range
        let prop = CfnProperty {
            prop_type: Some(TypeValue::Single("integer".to_string())),
            description: Some("A bounded int64 value".to_string()),
            format: Some("int64".to_string()),
            minimum: Some(0),
            maximum: Some(100),
            ..CfnProperty::default()
        };
        let schema = CfnSchema {
            type_name: "AWS::Test::Resource".to_string(),
            description: None,
            properties: BTreeMap::new(),
            required: vec![],
            read_only_properties: vec![],
            create_only_properties: vec![],
            write_only_properties: vec![],
            primary_identifier: None,
            definitions: None,
            tagging: None,
            one_of: vec![],
            any_of: vec![],
            handlers: BTreeMap::new(),
        };
        let (type_str, _) =
            cfn_type_to_carina_type_with_enum(&prop, "BoundedValue", &schema, "", &BTreeMap::new());
        // Range validation is encoded in the generated validator fn name.
        assert!(
            type_str.contains("Some((Some(0), Some(100)))"),
            "Should carry range metadata: {type_str}"
        );
        assert!(
            type_str.contains("AttributeType::refined_int("),
            "Should use refined Int: {type_str}"
        );
    }

    #[test]
    fn test_numeric_string_pattern_with_max_length_combines_constraints() {
        // "type": "string", "pattern": "[0-9]+", "maxLength": 20
        // should produce a validate function that checks BOTH pattern and length
        let prop = CfnProperty {
            prop_type: Some(TypeValue::Single("string".to_string())),
            description: Some("Object size in bytes".to_string()),
            pattern: Some("[0-9]+".to_string()),
            max_length: Some(20),
            ..CfnProperty::default()
        };
        let schema = CfnSchema {
            type_name: "AWS::S3::Bucket".to_string(),
            description: None,
            properties: BTreeMap::new(),
            required: vec![],
            read_only_properties: vec![],
            create_only_properties: vec![],
            write_only_properties: vec![],
            primary_identifier: None,
            definitions: None,
            tagging: None,
            one_of: vec![],
            any_of: vec![],
            handlers: BTreeMap::new(),
        };
        let (type_str, _) = cfn_type_to_carina_type_with_enum(
            &prop,
            "ObjectSizeGreaterThan",
            &schema,
            "",
            &BTreeMap::new(),
        );
        // Should mention both constraints
        assert!(
            type_str.contains("Some(\"[0-9]+\".to_string())"),
            "Should capture pattern: {type_str}"
        );
        assert!(
            type_str.contains("Some((None, Some(20)))"),
            "Should capture max length 20: {type_str}"
        );
    }

    #[test]
    fn test_string_pattern_with_length_constraints_combines() {
        // "type": "string", "pattern": "^[a-z]+$", "minLength": 1, "maxLength": 64
        // should produce a validate function that checks BOTH pattern and length
        let prop = CfnProperty {
            prop_type: Some(TypeValue::Single("string".to_string())),
            description: Some("A constrained string".to_string()),
            pattern: Some("^[a-z]+$".to_string()),
            min_length: Some(1),
            max_length: Some(64),
            ..CfnProperty::default()
        };
        let schema = CfnSchema {
            type_name: "AWS::Test::Resource".to_string(),
            description: None,
            properties: BTreeMap::new(),
            required: vec![],
            read_only_properties: vec![],
            create_only_properties: vec![],
            write_only_properties: vec![],
            primary_identifier: None,
            definitions: None,
            tagging: None,
            one_of: vec![],
            any_of: vec![],
            handlers: BTreeMap::new(),
        };
        let (type_str, _) =
            cfn_type_to_carina_type_with_enum(&prop, "SomeName", &schema, "", &BTreeMap::new());
        // Should mention both constraints
        assert!(
            type_str.contains("Some(\"^[a-z]+$\".to_string())"),
            "Should capture pattern: {type_str}"
        );
        assert!(
            type_str.contains("Some((Some(1), Some(64)))"),
            "Should capture length bounds 1..=64: {type_str}"
        );
    }

    #[test]
    fn test_array_items_pattern_with_length_generates_combined_validator() {
        // When an array property has items with both pattern and length constraints,
        // the generated code must emit a combined pattern+length validator function.
        // Regression test: previously the codegen used the array container's (empty)
        // length constraints instead of the items' constraints.
        let schema = CfnSchema {
            type_name: "AWS::Test::ArrayPattern".to_string(),
            description: None,
            properties: {
                let mut props = BTreeMap::new();
                props.insert(
                    "ThumbprintList".to_string(),
                    CfnProperty {
                        prop_type: Some(TypeValue::Single("array".to_string())),
                        items: Some(Box::new(CfnProperty {
                            prop_type: Some(TypeValue::Single("string".to_string())),
                            pattern: Some("[0-9A-Fa-f]{40}".to_string()),
                            min_length: Some(40),
                            max_length: Some(40),
                            ..CfnProperty::default()
                        })),
                        max_items: Some(5),
                        ..CfnProperty::default()
                    },
                );
                props
            },
            required: vec![],
            read_only_properties: vec![],
            create_only_properties: vec![],
            write_only_properties: vec![],
            primary_identifier: None,
            definitions: None,
            tagging: None,
            one_of: vec![],
            any_of: vec![],
            handlers: BTreeMap::new(),
        };

        let code = generate_schema_code(&schema, "AWS::Test::ArrayPattern")
            .expect("codegen should succeed");

        // The combined validator function must be defined
        assert!(
            code.contains("fn validate_string_pattern_") && code.contains("_len_40_40"),
            "Should generate combined pattern+length validator function, got:\n{code}"
        );

        // The validator should check both pattern and length
        assert!(
            code.contains("[0-9A-Fa-f]{40}") && code.contains("40..=40"),
            "Validator should check both pattern and length range"
        );
    }

    #[test]
    fn test_qualify_prefixes_colliding_struct_field_enums() {
        // Structural identity keeps the enum kind plain. The enclosing
        // struct path now lives in the namespace emitted at each use site.
        let mut enums = BTreeMap::new();

        enums.insert(
            "VersioningConfiguration.Status".to_string(),
            EnumInfo {
                type_name: "Status".to_string(),
                values: vec!["Enabled".to_string(), "Suspended".to_string()],
            },
        );
        enums.insert(
            "IntelligentTieringConfiguration.Status".to_string(),
            EnumInfo {
                type_name: "Status".to_string(),
                values: vec!["Disabled".to_string(), "Enabled".to_string()],
            },
        );

        qualify_nested_enum_type_names(&mut enums);

        let versioning = enums.get("VersioningConfiguration.Status").unwrap();
        assert_eq!(
            versioning.type_name, "Status",
            "Nested struct field enum kind should stay plain"
        );
        let tiering = enums.get("IntelligentTieringConfiguration.Status").unwrap();
        assert_eq!(
            tiering.type_name, "Status",
            "Nested struct field enum kind should stay plain"
        );
    }

    #[test]
    fn test_qualify_prefixes_even_when_values_match() {
        // Long-term type-safety guard: nested struct field enums keep their
        // own kind; the parent struct path is emitted in the namespace.
        let mut enums = BTreeMap::new();
        enums.insert(
            "DeleteMarkerReplication.Status".to_string(),
            EnumInfo {
                type_name: "Status".to_string(),
                values: vec!["Disabled".to_string(), "Enabled".to_string()],
            },
        );
        enums.insert(
            "ReplicationTime.Status".to_string(),
            EnumInfo {
                type_name: "Status".to_string(),
                values: vec!["Disabled".to_string(), "Enabled".to_string()],
            },
        );

        qualify_nested_enum_type_names(&mut enums);

        assert_eq!(
            enums
                .get("DeleteMarkerReplication.Status")
                .unwrap()
                .type_name,
            "Status",
        );
        assert_eq!(
            enums.get("ReplicationTime.Status").unwrap().type_name,
            "Status",
        );
    }

    #[test]
    fn test_qualify_leaves_top_level_property_enums_untouched() {
        // Top-level property enums (no parent struct, key has no dot) keep
        // their original type_name — they are already disambiguated by
        // being at the resource root.
        let mut enums = BTreeMap::new();
        enums.insert(
            "AbacStatus".to_string(),
            EnumInfo {
                type_name: "AbacStatus".to_string(),
                values: vec!["Enabled".to_string(), "Disabled".to_string()],
            },
        );

        qualify_nested_enum_type_names(&mut enums);

        assert_eq!(enums.get("AbacStatus").unwrap().type_name, "AbacStatus");
    }

    #[test]
    fn test_qualify_keeps_plain_kind_for_value_set_collision() {
        // Different value sets may share a plain kind because their emitted
        // TypeIdentity namespace is structural at the use site.
        let mut enums = BTreeMap::new();
        enums.insert(
            "A.Same".to_string(),
            EnumInfo {
                type_name: "Same".to_string(),
                values: vec!["X".to_string()],
            },
        );
        enums.insert(
            "Other".to_string(),
            EnumInfo {
                type_name: "ASame".to_string(),
                values: vec!["Y".to_string()],
            },
        );

        qualify_nested_enum_type_names(&mut enums);

        assert_eq!(enums.get("A.Same").unwrap().type_name, "Same");
        assert_eq!(enums.get("Other").unwrap().type_name, "ASame");
    }

    #[test]
    fn test_read_only_attribute_preserves_original_description() {
        let mut properties = BTreeMap::new();
        properties.insert(
            "FlowLogId".to_string(),
            CfnProperty {
                prop_type: Some(TypeValue::Single("string".to_string())),
                description: Some("The Flow Log ID".to_string()),
                ..Default::default()
            },
        );
        properties.insert(
            "Name".to_string(),
            CfnProperty {
                prop_type: Some(TypeValue::Single("string".to_string())),
                description: Some("The name of the resource.".to_string()),
                ..Default::default()
            },
        );

        let schema = CfnSchema {
            type_name: "AWS::EC2::FlowLog".to_string(),
            description: None,
            properties,
            required: vec![],
            read_only_properties: vec!["/properties/FlowLogId".to_string()],
            create_only_properties: vec![],
            write_only_properties: vec![],
            primary_identifier: None,
            definitions: None,
            tagging: None,
            one_of: vec![],
            any_of: vec![],
            handlers: BTreeMap::new(),
        };

        let generated = generate_schema_code(&schema, "AWS::EC2::FlowLog").unwrap();

        // Read-only attribute WITH description should append " (read-only)" to the original
        assert!(
            generated.contains(r#".with_description("The Flow Log ID (read-only)")"#),
            "Read-only attribute with description should have ' (read-only)' appended: {generated}"
        );
    }

    #[test]
    fn test_read_only_attribute_without_description_has_no_read_only_description() {
        let mut properties = BTreeMap::new();
        properties.insert(
            "ResourceId".to_string(),
            CfnProperty {
                prop_type: Some(TypeValue::Single("string".to_string())),
                description: None,
                ..Default::default()
            },
        );

        let schema = CfnSchema {
            type_name: "AWS::EC2::TestResource".to_string(),
            description: None,
            properties,
            required: vec![],
            read_only_properties: vec!["/properties/ResourceId".to_string()],
            create_only_properties: vec![],
            write_only_properties: vec![],
            primary_identifier: None,
            definitions: None,
            tagging: None,
            one_of: vec![],
            any_of: vec![],
            handlers: BTreeMap::new(),
        };

        let generated = generate_schema_code(&schema, "AWS::EC2::TestResource").unwrap();

        // A read-only attribute with no description should not get a "(read-only)" description
        assert!(
            !generated.contains(r#".with_description("(read-only)")"#),
            "Read-only attribute without original description should not get a '(read-only)' description: {generated}"
        );
    }

    #[test]
    fn test_format_field_parsed_from_schema() {
        // Verify that the format field is correctly parsed from JSON
        let json = r#"{
            "type": "integer",
            "format": "int64",
            "description": "Test"
        }"#;
        let prop: CfnProperty = serde_json::from_str(json).unwrap();
        assert_eq!(prop.format, Some("int64".to_string()));
    }

    // --- Exclusive field detection tests ---

    fn make_schema_with_props(props: BTreeMap<String, CfnProperty>) -> CfnSchema {
        CfnSchema {
            type_name: "AWS::Test::Resource".to_string(),
            description: None,
            properties: props,
            required: vec![],
            read_only_properties: vec![],
            create_only_properties: vec![],
            write_only_properties: vec![],
            primary_identifier: None,
            definitions: None,
            tagging: None,
            one_of: vec![],
            any_of: vec![],
            handlers: BTreeMap::new(),
        }
    }

    #[test]
    fn test_detect_exclusive_from_oneof_two_variants() {
        let mut schema = make_schema_with_props(BTreeMap::new());
        schema.one_of = vec![
            CfnProperty {
                required: vec!["FieldA".to_string()],
                ..Default::default()
            },
            CfnProperty {
                required: vec!["FieldB".to_string()],
                ..Default::default()
            },
        ];
        let groups = detect_exclusive_from_oneof(&schema);
        assert_eq!(groups.len(), 1);
        let mut group = groups[0].clone();
        group.sort();
        assert_eq!(group, vec!["FieldA", "FieldB"]);
    }

    #[test]
    fn test_detect_exclusive_from_oneof_three_variants() {
        let mut schema = make_schema_with_props(BTreeMap::new());
        schema.one_of = vec![
            CfnProperty {
                required: vec!["A".to_string()],
                ..Default::default()
            },
            CfnProperty {
                required: vec!["B".to_string()],
                ..Default::default()
            },
            CfnProperty {
                required: vec!["C".to_string()],
                ..Default::default()
            },
        ];
        let groups = detect_exclusive_from_oneof(&schema);
        assert_eq!(groups.len(), 1);
        let mut group = groups[0].clone();
        group.sort();
        assert_eq!(group, vec!["A", "B", "C"]);
    }

    #[test]
    fn test_detect_exclusive_from_oneof_multi_required_ignored() {
        // Variants with multiple required fields are not simple exclusive groups
        let mut schema = make_schema_with_props(BTreeMap::new());
        schema.one_of = vec![
            CfnProperty {
                required: vec!["A".to_string(), "B".to_string()],
                ..Default::default()
            },
            CfnProperty {
                required: vec!["C".to_string()],
                ..Default::default()
            },
        ];
        let groups = detect_exclusive_from_oneof(&schema);
        assert!(groups.is_empty());
    }

    #[test]
    fn test_detect_exclusive_from_oneof_empty() {
        let schema = make_schema_with_props(BTreeMap::new());
        let groups = detect_exclusive_from_oneof(&schema);
        assert!(groups.is_empty());
    }

    #[test]
    fn test_detect_exclusive_from_anyof() {
        let mut schema = make_schema_with_props(BTreeMap::new());
        schema.any_of = vec![
            CfnProperty {
                required: vec!["X".to_string()],
                ..Default::default()
            },
            CfnProperty {
                required: vec!["Y".to_string()],
                ..Default::default()
            },
        ];
        let groups = detect_exclusive_from_oneof(&schema);
        assert_eq!(groups.len(), 1);
    }

    #[test]
    fn test_detect_exclusive_from_description_but_not_both() {
        let mut props = BTreeMap::new();
        props.insert(
            "InternetGatewayId".to_string(),
            CfnProperty {
                description: Some(
                    "The ID of the internet gateway. You must specify either InternetGatewayId or VpnGatewayId, but not both.".to_string(),
                ),
                ..CfnProperty::default()
            },
        );
        props.insert(
            "VpnGatewayId".to_string(),
            CfnProperty {
                description: Some(
                    "The ID of the virtual private gateway. You must specify either InternetGatewayId or VpnGatewayId, but not both.".to_string(),
                ),
                ..CfnProperty::default()
            },
        );
        let schema = make_schema_with_props(props);
        let groups = detect_exclusive_from_descriptions(&schema);
        assert_eq!(groups.len(), 1);
        let mut group = groups[0].clone();
        group.sort();
        assert_eq!(group, vec!["InternetGatewayId", "VpnGatewayId"]);
    }

    #[test]
    fn test_detect_exclusive_from_description_backtick_format() {
        let mut props = BTreeMap::new();
        props.insert(
            "CidrBlock".to_string(),
            CfnProperty {
                description: Some(
                    "You must specify either``CidrBlock`` or ``Ipv4IpamPoolId``.".to_string(),
                ),
                ..CfnProperty::default()
            },
        );
        props.insert(
            "Ipv4IpamPoolId".to_string(),
            CfnProperty {
                description: Some(
                    "You must specify either``CidrBlock`` or ``Ipv4IpamPoolId``.".to_string(),
                ),
                ..CfnProperty::default()
            },
        );
        let schema = make_schema_with_props(props);
        let groups = detect_exclusive_from_descriptions(&schema);
        assert_eq!(groups.len(), 1);
        let mut group = groups[0].clone();
        group.sort();
        assert_eq!(group, vec!["CidrBlock", "Ipv4IpamPoolId"]);
    }

    #[test]
    fn test_detect_exclusive_from_description_field_not_in_schema() {
        // If the referenced field name isn't a property, no group is detected
        let mut props = BTreeMap::new();
        props.insert(
            "FieldA".to_string(),
            CfnProperty {
                description: Some(
                    "You must specify either FieldA or NonExistentField, but not both.".to_string(),
                ),
                ..CfnProperty::default()
            },
        );
        let schema = make_schema_with_props(props);
        let groups = detect_exclusive_from_descriptions(&schema);
        assert!(groups.is_empty());
    }

    #[test]
    fn test_detect_exclusive_from_description_no_pattern() {
        let mut props = BTreeMap::new();
        props.insert(
            "SomeField".to_string(),
            CfnProperty {
                description: Some("Just a normal description.".to_string()),
                ..CfnProperty::default()
            },
        );
        let schema = make_schema_with_props(props);
        let groups = detect_exclusive_from_descriptions(&schema);
        assert!(groups.is_empty());
    }

    #[test]
    fn test_detect_exclusive_fields_combined() {
        // Test that detect_exclusive_fields combines both methods and deduplicates
        let mut props = BTreeMap::new();
        props.insert(
            "FieldA".to_string(),
            CfnProperty {
                description: Some(
                    "You must specify either FieldA or FieldB, but not both.".to_string(),
                ),
                ..CfnProperty::default()
            },
        );
        props.insert("FieldB".to_string(), CfnProperty::default());
        let mut schema = make_schema_with_props(props);
        // Also add oneOf that detects the same group
        schema.one_of = vec![
            CfnProperty {
                required: vec!["FieldA".to_string()],
                ..Default::default()
            },
            CfnProperty {
                required: vec!["FieldB".to_string()],
                ..Default::default()
            },
        ];
        let groups = detect_exclusive_fields(&schema, "AWS::Test::Resource");
        // Should be deduplicated to 1 group
        assert_eq!(groups.len(), 1);
        let mut group = groups[0].clone();
        group.sort();
        assert_eq!(group, vec!["FieldA", "FieldB"]);
    }

    #[test]
    fn test_vpc_gateway_attachment_detected_via_description() {
        // Verify VPCGatewayAttachment exclusive fields are detected from descriptions
        let mut props = BTreeMap::new();
        props.insert(
            "InternetGatewayId".to_string(),
            CfnProperty {
                description: Some(
                    "The ID of the internet gateway. You must specify either InternetGatewayId or VpnGatewayId, but not both.".to_string(),
                ),
                ..CfnProperty::default()
            },
        );
        props.insert(
            "VpnGatewayId".to_string(),
            CfnProperty {
                description: Some(
                    "The ID of the virtual private gateway. You must specify either InternetGatewayId or VpnGatewayId, but not both.".to_string(),
                ),
                ..CfnProperty::default()
            },
        );
        props.insert(
            "VpcId".to_string(),
            CfnProperty {
                description: Some("The ID of the VPC.".to_string()),
                ..CfnProperty::default()
            },
        );
        let schema = make_schema_with_props(props);
        let groups = detect_exclusive_fields(&schema, "AWS::EC2::VPCGatewayAttachment");
        assert_eq!(groups.len(), 1, "Should detect exactly one exclusive group");
        let mut group = groups[0].clone();
        group.sort();
        assert_eq!(group, vec!["InternetGatewayId", "VpnGatewayId"]);
    }

    #[test]
    fn test_generated_code_contains_validator_for_detected_exclusives() {
        // Full integration: generate_schema_code should produce a declarative
        // .exclusive_required(&[...]) for description-detected exclusive
        // fields — not a closure. Closures don't cross the WASM boundary.
        let mut props = BTreeMap::new();
        props.insert(
            "InternetGatewayId".to_string(),
            CfnProperty {
                description: Some(
                    "The ID of the internet gateway. You must specify either InternetGatewayId or VpnGatewayId, but not both.".to_string(),
                ),
                prop_type: Some(TypeValue::Single("string".to_string())),
                ..CfnProperty::default()
            },
        );
        props.insert(
            "VpnGatewayId".to_string(),
            CfnProperty {
                description: Some(
                    "The ID of the virtual private gateway. You must specify either InternetGatewayId or VpnGatewayId, but not both.".to_string(),
                ),
                prop_type: Some(TypeValue::Single("string".to_string())),
                ..CfnProperty::default()
            },
        );
        props.insert(
            "VpcId".to_string(),
            CfnProperty {
                description: Some("The ID of the VPC.".to_string()),
                prop_type: Some(TypeValue::Single("string".to_string())),
                ..CfnProperty::default()
            },
        );
        let schema = CfnSchema {
            type_name: "AWS::EC2::VPCGatewayAttachment".to_string(),
            description: Some("VPC Gateway Attachment".to_string()),
            properties: props,
            required: vec!["VpcId".to_string()],
            read_only_properties: vec![],
            create_only_properties: vec![],
            write_only_properties: vec![],
            primary_identifier: None,
            definitions: None,
            tagging: None,
            one_of: vec![],
            any_of: vec![],
            handlers: BTreeMap::new(),
        };
        let code = generate_schema_code(&schema, "AWS::EC2::VPCGatewayAttachment").unwrap();
        assert!(
            code.contains(".exclusive_required(&["),
            "Generated code should emit declarative .exclusive_required: {code}"
        );
        assert!(
            code.contains("\"internet_gateway_id\"") && code.contains("\"vpn_gateway_id\""),
            "Generated code should reference snake_case field names: {code}"
        );
        assert!(
            !code.contains("validators::validate_exclusive_required"),
            "Generated code should not emit the old closure form: {code}"
        );
    }

    #[test]
    fn test_generate_schema_code_emits_write_only() {
        let mut properties = BTreeMap::new();
        properties.insert(
            "Password".to_string(),
            CfnProperty {
                prop_type: Some(TypeValue::Single("string".to_string())),
                description: Some("The password.".to_string()),
                ..Default::default()
            },
        );
        properties.insert(
            "Name".to_string(),
            CfnProperty {
                prop_type: Some(TypeValue::Single("string".to_string())),
                description: Some("The name.".to_string()),
                ..Default::default()
            },
        );

        let schema = CfnSchema {
            type_name: "AWS::Test::Resource".to_string(),
            description: None,
            properties,
            required: vec![],
            read_only_properties: vec![],
            create_only_properties: vec![],
            write_only_properties: vec!["/properties/Password".to_string()],
            primary_identifier: None,
            definitions: None,
            tagging: None,
            one_of: vec![],
            any_of: vec![],
            handlers: BTreeMap::new(),
        };

        let code = generate_schema_code(&schema, "AWS::Test::Resource").unwrap();

        // Password should have .write_only()
        assert!(
            code.contains(".write_only()"),
            "Should emit .write_only() for Password: {code}"
        );
        // Name should NOT have .write_only()
        let name_section = code
            .split("AttributeSchema::new(\"name\"")
            .nth(1)
            .unwrap_or("");
        let name_attr_end = name_section
            .split(".attribute(")
            .next()
            .unwrap_or(name_section);
        assert!(
            !name_attr_end.contains(".write_only()"),
            "Name should not have .write_only(): {code}"
        );
    }

    #[test]
    fn test_generate_markdown_shows_write_only() {
        let mut properties = BTreeMap::new();
        properties.insert(
            "Password".to_string(),
            CfnProperty {
                prop_type: Some(TypeValue::Single("string".to_string())),
                description: Some("The password.".to_string()),
                ..Default::default()
            },
        );
        properties.insert(
            "Name".to_string(),
            CfnProperty {
                prop_type: Some(TypeValue::Single("string".to_string())),
                description: Some("The name.".to_string()),
                ..Default::default()
            },
        );

        let schema = CfnSchema {
            type_name: "AWS::Test::Resource".to_string(),
            description: None,
            properties,
            required: vec![],
            read_only_properties: vec![],
            create_only_properties: vec![],
            write_only_properties: vec!["/properties/Password".to_string()],
            primary_identifier: None,
            definitions: None,
            tagging: None,
            one_of: vec![],
            any_of: vec![],
            handlers: BTreeMap::new(),
        };

        let md = generate_markdown(&schema, "AWS::Test::Resource").unwrap();

        assert!(
            md.contains("**Write-only:** Yes"),
            "Should show write-only annotation for Password: {md}"
        );
        // Name should NOT have write-only
        let name_section = md.split("### `name`").nth(1).unwrap_or("");
        assert!(
            !name_section
                .split("###")
                .next()
                .unwrap_or("")
                .contains("Write-only"),
            "Name should not have write-only annotation: {md}"
        );
    }

    #[test]
    fn test_nested_write_only_does_not_propagate_to_parent() {
        // Regression test for #1346: when only nested sub-properties are write-only,
        // the parent attribute should NOT be marked write-only.
        let mut properties = BTreeMap::new();
        properties.insert(
            "Config".to_string(),
            CfnProperty {
                prop_type: Some(TypeValue::Single("object".to_string())),
                description: Some("A configuration object.".to_string()),
                properties: Some(BTreeMap::from([(
                    "SecretField".to_string(),
                    CfnProperty {
                        prop_type: Some(TypeValue::Single("string".to_string())),
                        description: Some("A secret nested field.".to_string()),
                        ..Default::default()
                    },
                )])),
                ..Default::default()
            },
        );

        let schema = CfnSchema {
            type_name: "AWS::Test::Nested".to_string(),
            description: None,
            properties,
            required: vec![],
            read_only_properties: vec![],
            create_only_properties: vec![],
            // Only the nested sub-property is write-only, NOT the parent
            write_only_properties: vec!["/properties/Config/SecretField".to_string()],
            primary_identifier: None,
            definitions: None,
            tagging: None,
            one_of: vec![],
            any_of: vec![],
            handlers: BTreeMap::new(),
        };

        let code = generate_schema_code(&schema, "AWS::Test::Nested").unwrap();

        // Config should NOT have .write_only() because only its child is write-only
        assert!(
            !code.contains(".write_only()"),
            "Parent attribute should NOT be marked write-only when only nested \
             sub-properties are write-only. Generated code:\n{code}"
        );
    }

    #[test]
    fn test_has_cloud_control_handlers_with_handlers() {
        let json = r#"{
            "typeName": "AWS::EC2::VPC",
            "properties": {},
            "handlers": {
                "create": {}, "read": {}, "update": {}, "delete": {}, "list": {}
            }
        }"#;
        let schema: CfnSchema = serde_json::from_str(json).unwrap();
        assert!(schema.has_cloud_control_handlers());
    }

    #[test]
    fn test_has_cloud_control_handlers_empty() {
        // NON_PROVISIONABLE types like AWS::Route53::RecordSet have handlers: {}
        let json = r#"{
            "typeName": "AWS::Route53::RecordSet",
            "properties": {},
            "handlers": {}
        }"#;
        let schema: CfnSchema = serde_json::from_str(json).unwrap();
        assert!(!schema.has_cloud_control_handlers());
    }

    #[test]
    fn test_has_cloud_control_handlers_missing() {
        // Schemas without a handlers key at all are also unsupported.
        let json = r#"{
            "typeName": "AWS::Test::NoHandlers",
            "properties": {}
        }"#;
        let schema: CfnSchema = serde_json::from_str(json).unwrap();
        assert!(!schema.has_cloud_control_handlers());
    }

    // === Issue #199: snake_case DSL spelling for every StringEnum value ===

    #[test]
    fn test_dsl_enum_value_ipv4_ipv6() {
        // IPv4/IPv6 are PascalCase-ish but heck's snake_case turns them into
        // "i_pv4"/"i_pv6" which is wrong. The DSL form should be the all-lowercase
        // "ipv4"/"ipv6".
        assert_eq!(dsl_enum_value("IPv4"), "ipv4");
        assert_eq!(dsl_enum_value("IPv6"), "ipv6");
    }

    #[test]
    fn test_dsl_enum_value_pascal_case() {
        // PascalCase -> snake_case
        assert_eq!(dsl_enum_value("Enabled"), "enabled");
        assert_eq!(
            dsl_enum_value("BucketOwnerEnforced"),
            "bucket_owner_enforced"
        );
        assert_eq!(dsl_enum_value("VersioningStatus"), "versioning_status");
        assert_eq!(dsl_enum_value("ObjectWriter"), "object_writer");
    }

    #[test]
    fn test_dsl_enum_value_shouty_snake() {
        // SHOUTY_SNAKE -> lowercase
        assert_eq!(dsl_enum_value("GROUP"), "group");
        assert_eq!(dsl_enum_value("AES256"), "aes256");
        assert_eq!(dsl_enum_value("DEEP_ARCHIVE"), "deep_archive");
        assert_eq!(dsl_enum_value("AWS_ACCOUNT"), "aws_account");
    }

    #[test]
    fn test_dsl_enum_value_kebab_to_snake() {
        // kebab -> snake (replace hyphens with underscores)
        assert_eq!(dsl_enum_value("ap-northeast-1"), "ap_northeast_1");
        assert_eq!(dsl_enum_value("cloud-watch-logs"), "cloud_watch_logs");
    }

    #[test]
    fn test_dsl_enum_value_already_snake_passthrough() {
        // Already snake_case stays unchanged
        assert_eq!(dsl_enum_value("default"), "default");
        assert_eq!(dsl_enum_value("dedicated"), "dedicated");
        assert_eq!(dsl_enum_value("ap_northeast_1"), "ap_northeast_1");
    }

    #[test]
    fn test_dsl_enum_value_colon_to_snake() {
        // AWS S3 SSE algorithm values carry a colon (`aws:kms`, `aws:kms:dsse`).
        // The colon makes the value unwritable as a bare DSL identifier, so
        // codegen must rewrite it to `_` to keep the value reachable under
        // the strict-DSL validator (carina-rs/carina#2986). See awscc#230.
        assert_eq!(dsl_enum_value("aws:kms"), "aws_kms");
        assert_eq!(dsl_enum_value("aws:kms:dsse"), "aws_kms_dsse");
    }

    #[test]
    fn test_dsl_enum_value_dotted_mixed_to_snake() {
        // Mixed dotted values (letters + `.`) like "ipsec.1" rewrite
        // to snake_case (`ipsec_1`) so the DSL spelling stays a bare
        // identifier — the strict-DSL validator
        // (carina-rs/carina#2980 / awscc#222) gates acceptance on it.
        assert_eq!(dsl_enum_value("ipsec.1"), "ipsec_1");
    }

    #[test]
    fn test_dsl_enum_value_numeric_passthrough() {
        // Pure numeric stays
        assert_eq!(dsl_enum_value("1"), "1");
        assert_eq!(dsl_enum_value("128"), "128");
    }

    #[test]
    fn test_string_enum_emits_snake_case_dsl_aliases_for_pascal_values() {
        // Regression for awscc#199 / carina#2831: PascalCase enum values
        // must produce `dsl_aliases` entries that pair the canonical (API)
        // spelling with a snake_case DSL spelling, so the validator
        // accepts both forms — and the data crosses the WASM boundary
        // intact (where a `fn` pointer would not).
        let mut properties = BTreeMap::new();
        properties.insert(
            "ObjectOwnership".to_string(),
            CfnProperty {
                prop_type: Some(TypeValue::Single("string".to_string())),
                description: Some("Object ownership".to_string()),
                enum_values: Some(vec![
                    EnumValue::Str("ObjectWriter".to_string()),
                    EnumValue::Str("BucketOwnerPreferred".to_string()),
                    EnumValue::Str("BucketOwnerEnforced".to_string()),
                ]),
                items: None,
                ref_path: None,
                insertion_order: None,
                properties: None,
                required: vec![],
                minimum: None,
                maximum: None,
                additional_properties: None,
                const_value: None,
                default_value: None,
                pattern: None,
                min_items: None,
                max_items: None,
                min_length: None,
                max_length: None,
                format: None,
                ..Default::default()
            },
        );
        let schema = CfnSchema {
            type_name: "AWS::S3::Bucket".to_string(),
            description: None,
            properties,
            required: vec![],
            read_only_properties: vec![],
            create_only_properties: vec![],
            write_only_properties: vec![],
            primary_identifier: None,
            definitions: None,
            tagging: None,
            one_of: vec![],
            any_of: vec![],
            handlers: BTreeMap::new(),
        };
        let generated = generate_schema_code(&schema, "AWS::S3::Bucket").unwrap();

        // The values list still carries the API spellings (canonical for round-trip
        // with the AWS API) but a populated dsl_aliases entry must be present.
        assert!(
            generated.contains("\"BucketOwnerEnforced\".to_string()"),
            "API spelling must remain in the values list: {generated}"
        );
        assert!(
            !generated.contains("dsl_aliases: vec![],"),
            "dsl_aliases must be non-empty for an enum with PascalCase values: {generated}"
        );
        // The generated dsl_aliases vec must contain a (PascalCase, snake_case) pair.
        assert!(
            generated.contains(
                "(\"BucketOwnerEnforced\".to_string(), \"bucket_owner_enforced\".to_string())"
            ),
            "dsl_aliases must map PascalCase to snake_case: {generated}"
        );
        // And the closure shape is gone — the alias map is data only.
        assert!(
            !generated.contains("|s: &str| match s"),
            "no `fn` closure should remain in StringEnum emission: {generated}"
        );
    }

    #[test]
    fn test_string_enum_emits_snake_case_dsl_aliases_for_shouty_snake_values() {
        // Regression for #199: SHOUTY_SNAKE values like AES256 must get a
        // snake_case alias entry in dsl_aliases.
        let mut properties = BTreeMap::new();
        properties.insert(
            "StorageClass".to_string(),
            CfnProperty {
                prop_type: Some(TypeValue::Single("string".to_string())),
                description: Some("Storage class".to_string()),
                enum_values: Some(vec![
                    EnumValue::Str("STANDARD".to_string()),
                    EnumValue::Str("DEEP_ARCHIVE".to_string()),
                    EnumValue::Str("GLACIER_IR".to_string()),
                ]),
                items: None,
                ref_path: None,
                insertion_order: None,
                properties: None,
                required: vec![],
                minimum: None,
                maximum: None,
                additional_properties: None,
                const_value: None,
                default_value: None,
                pattern: None,
                min_items: None,
                max_items: None,
                min_length: None,
                max_length: None,
                format: None,
                ..Default::default()
            },
        );
        let schema = CfnSchema {
            type_name: "AWS::S3::Bucket".to_string(),
            description: None,
            properties,
            required: vec![],
            read_only_properties: vec![],
            create_only_properties: vec![],
            write_only_properties: vec![],
            primary_identifier: None,
            definitions: None,
            tagging: None,
            one_of: vec![],
            any_of: vec![],
            handlers: BTreeMap::new(),
        };
        let generated = generate_schema_code(&schema, "AWS::S3::Bucket").unwrap();
        assert!(
            generated.contains("(\"DEEP_ARCHIVE\".to_string(), \"deep_archive\".to_string())"),
            "dsl_aliases must map SHOUTY_SNAKE to snake_case: {generated}"
        );
    }

    #[test]
    fn test_string_enum_dsl_aliases_empty_when_all_values_already_snake() {
        // When every enum value is already snake_case, no transformation is
        // required and dsl_aliases is the empty vec — no rewrites needed.
        let mut properties = BTreeMap::new();
        properties.insert(
            "InstanceTenancy".to_string(),
            CfnProperty {
                prop_type: Some(TypeValue::Single("string".to_string())),
                description: Some("Instance tenancy".to_string()),
                enum_values: Some(vec![
                    EnumValue::Str("default".to_string()),
                    EnumValue::Str("dedicated".to_string()),
                    EnumValue::Str("host".to_string()),
                ]),
                items: None,
                ref_path: None,
                insertion_order: None,
                properties: None,
                required: vec![],
                minimum: None,
                maximum: None,
                additional_properties: None,
                const_value: None,
                default_value: None,
                pattern: None,
                min_items: None,
                max_items: None,
                min_length: None,
                max_length: None,
                format: None,
                ..Default::default()
            },
        );
        let schema = CfnSchema {
            type_name: "AWS::EC2::VPC".to_string(),
            description: None,
            properties,
            required: vec![],
            read_only_properties: vec![],
            create_only_properties: vec![],
            write_only_properties: vec![],
            primary_identifier: None,
            definitions: None,
            tagging: None,
            one_of: vec![],
            any_of: vec![],
            handlers: BTreeMap::new(),
        };
        let generated = generate_schema_code(&schema, "AWS::EC2::VPC").unwrap();
        // After carina-rs/carina#2980 / awscc#222 the dsl_aliases table
        // is exhaustive — every value (including identity rows where
        // the DSL spelling equals the canonical) is emitted. The
        // strict-DSL validator gates on the presence of any rewrite
        // pair; identity rows are needed so legitimate same-spelling
        // values still validate.
        for v in ["default", "dedicated", "host"] {
            let identity_pair = format!("(\"{v}\".to_string(), \"{v}\".to_string())");
            assert!(
                generated.contains(&identity_pair),
                "dsl_aliases must include identity row for `{v}`: {generated}"
            );
        }
        assert!(
            !generated.contains("|s: &str| match s"),
            "no `fn` closure should be emitted when every value is already snake_case: {generated}"
        );
    }

    // (test_enum_alias_reverse_covers_pascal_values removed in
    // awscc#220 — the enum_alias_reverse() / enum_alias_entries() /
    // ENUM_ALIAS_DISPATCH path is gone. The (snake_case, API)
    // round-trip is now exercised by `DslMap::api_for` against the
    // exhaustive `dsl_aliases` table; see #223's tests.)

    #[test]
    fn test_generate_markdown_uses_snake_case_dsl_identifier() {
        // Regression for #199: the "DSL Identifier" column and "Shorthand
        // formats" line in the generated markdown must use the snake_case DSL
        // spelling, not the API spelling.
        let mut properties = BTreeMap::new();
        properties.insert(
            "ObjectOwnership".to_string(),
            CfnProperty {
                prop_type: Some(TypeValue::Single("string".to_string())),
                description: Some("Object ownership".to_string()),
                enum_values: Some(vec![
                    EnumValue::Str("ObjectWriter".to_string()),
                    EnumValue::Str("BucketOwnerPreferred".to_string()),
                    EnumValue::Str("BucketOwnerEnforced".to_string()),
                ]),
                items: None,
                ref_path: None,
                insertion_order: None,
                properties: None,
                required: vec![],
                minimum: None,
                maximum: None,
                additional_properties: None,
                const_value: None,
                default_value: None,
                pattern: None,
                min_items: None,
                max_items: None,
                min_length: None,
                max_length: None,
                format: None,
                ..Default::default()
            },
        );
        let schema = CfnSchema {
            type_name: "AWS::S3::Bucket".to_string(),
            description: None,
            properties,
            required: vec![],
            read_only_properties: vec![],
            create_only_properties: vec![],
            write_only_properties: vec![],
            primary_identifier: None,
            definitions: None,
            tagging: None,
            one_of: vec![],
            any_of: vec![],
            handlers: BTreeMap::new(),
        };
        let md = generate_markdown(&schema, "AWS::S3::Bucket").unwrap();
        // DSL Identifier column must use snake_case
        assert!(
            md.contains("`aws.s3.Bucket.ObjectOwnership.bucket_owner_enforced`"),
            "DSL Identifier column must use snake_case: {md}"
        );
        // The PascalCase form must NOT appear as the DSL Identifier
        assert!(
            !md.contains("`aws.s3.Bucket.ObjectOwnership.BucketOwnerEnforced`"),
            "DSL Identifier column must not use the API spelling: {md}"
        );
        // Shorthand formats line must use snake_case (first value: ObjectWriter -> object_writer).
        assert!(
            md.contains("Shorthand formats: `object_writer` or `ObjectOwnership.object_writer`"),
            "Shorthand formats must use snake_case: {md}"
        );
        // The "Value" column may keep the API spelling — verify it still does.
        assert!(
            md.contains("`BucketOwnerEnforced`"),
            "Value column should retain the API spelling for round-trip clarity: {md}"
        );
    }

    // === awscc#245: alternation-only patterns become real enums ===

    #[test]
    fn test_extract_enum_from_alternation_pattern_multi() {
        assert_eq!(
            extract_enum_from_alternation_pattern("^(s3|mediastore|lambda|mediapackagev2)$"),
            Some(vec![
                "s3".to_string(),
                "mediastore".to_string(),
                "lambda".to_string(),
                "mediapackagev2".to_string(),
            ])
        );
        assert_eq!(
            extract_enum_from_alternation_pattern("^(never|no-override|always)$"),
            Some(vec![
                "never".to_string(),
                "no-override".to_string(),
                "always".to_string(),
            ])
        );
    }

    #[test]
    fn test_extract_enum_from_alternation_pattern_single_value() {
        // Parenthesized single value.
        assert_eq!(
            extract_enum_from_alternation_pattern("^(sigv4)$"),
            Some(vec!["sigv4".to_string()])
        );
        // Bare single value, no parens.
        assert_eq!(
            extract_enum_from_alternation_pattern("^sigv4$"),
            Some(vec!["sigv4".to_string()])
        );
    }

    #[test]
    fn test_extract_enum_from_alternation_pattern_rejects_non_alternation() {
        // Missing anchors.
        assert_eq!(extract_enum_from_alternation_pattern("(a|b|c)"), None);
        assert_eq!(extract_enum_from_alternation_pattern("a|b"), None);
        // Character classes / quantifiers / escapes.
        assert_eq!(extract_enum_from_alternation_pattern("^[0-9]+$"), None);
        assert_eq!(extract_enum_from_alternation_pattern("^(a|b)+$"), None);
        assert_eq!(extract_enum_from_alternation_pattern("^(a|b\\d)$"), None);
        // Empty alternative (trailing `|`).
        assert_eq!(extract_enum_from_alternation_pattern("^(a|b|)$"), None);
        // Nested groups.
        assert_eq!(extract_enum_from_alternation_pattern("^((a)|b)$"), None);
        // Real-world non-alternation patterns from the CFN cache.
        assert_eq!(
            extract_enum_from_alternation_pattern(
                "^([0-9a-f]{10}-|)[A-Fa-f0-9]{8}-[A-Fa-f0-9]{4}$"
            ),
            None
        );
        // Top-level alternation without an outer group is NOT equivalent to
        // `^(a|b)$`: regex parses `^a|b$` as `(^a)|(b$)`. Reject it.
        assert_eq!(extract_enum_from_alternation_pattern("^a|b$"), None);
        assert_eq!(extract_enum_from_alternation_pattern("^a|b|c$"), None);
        // Empty parens.
        assert_eq!(extract_enum_from_alternation_pattern("^()$"), None);
        assert_eq!(extract_enum_from_alternation_pattern("^$"), None);
    }

    #[test]
    fn test_alternation_pattern_emits_string_enum_not_custom() {
        // Mirrors the awscc#245 reproducer: a CFN string property whose only
        // constraint is `pattern: "^(a|b|c)$"` must become a real
        // StringEnum, not Custom { pattern, base: String }.
        let mut properties = BTreeMap::new();
        properties.insert(
            "OriginAccessControlOriginType".to_string(),
            CfnProperty {
                prop_type: Some(TypeValue::Single("string".to_string())),
                description: None,
                enum_values: None,
                items: None,
                ref_path: None,
                insertion_order: None,
                properties: None,
                required: vec![],
                minimum: None,
                maximum: None,
                additional_properties: None,
                const_value: None,
                default_value: None,
                pattern: Some("^(s3|mediastore|lambda|mediapackagev2)$".to_string()),
                min_items: None,
                max_items: None,
                min_length: None,
                max_length: None,
                format: None,
                ..Default::default()
            },
        );
        let schema = CfnSchema {
            type_name: "AWS::CloudFront::OriginAccessControl".to_string(),
            description: None,
            properties,
            required: vec![],
            read_only_properties: vec![],
            create_only_properties: vec![],
            write_only_properties: vec![],
            primary_identifier: None,
            definitions: None,
            tagging: None,
            one_of: vec![],
            any_of: vec![],
            handlers: BTreeMap::new(),
        };
        let generated =
            generate_schema_code(&schema, "AWS::CloudFront::OriginAccessControl").unwrap();
        assert!(
            generated.contains("AttributeType::enum_("),
            "alternation pattern should produce Enum: {generated}"
        );
        // Must not fall back to a Custom { pattern, ... } emission.
        assert!(
            !generated.contains("Some(\"^(s3|mediastore|lambda|mediapackagev2)$\""),
            "alternation pattern should not flow through Custom: {generated}"
        );
        assert!(
            !generated.contains("validate_string_pattern_"),
            "alternation pattern should not emit a per-pattern validator: {generated}"
        );
        // All four literals must appear in the StringEnum values list.
        for v in ["s3", "mediastore", "lambda", "mediapackagev2"] {
            assert!(
                generated.contains(&format!("\"{}\".to_string()", v)), // rust-lit-guard: allow
                "missing enum value {v} in: {generated}"
            );
        }
    }

    // === awscc#246: nested-field enum overlay ===

    #[test]
    fn test_resource_type_overrides_has_nested_distribution_enum_keys() {
        // Spot-check the overlay registration. If any of these keys are
        // missing the codegen will silently fall back to plain String for
        // the affected field, which is the bug awscc#246 fixed.
        let table = resource_type_overrides();
        // Plain `Enum` overrides (order-sensitive scalars/lists).
        for key in [
            "DistributionConfig.PriceClass",
            "DistributionConfig.HttpVersion",
            "Cookies.Forward",
            "LegacyCustomOrigin.OriginProtocolPolicy",
            "LegacyCustomOrigin.OriginSSLProtocols",
            "CustomOriginConfig.OriginSSLProtocols",
        ] {
            assert!(
                matches!(
                    table.get(&("AWS::CloudFront::Distribution", key)),
                    Some(TypeOverride::Enum(_))
                ),
                "missing nested-field enum override: {key}"
            );
        }
        // AllowedMethods / CachedMethods are order-insensitive sets:
        // they must be `EnumUnordered` so the generated list is
        // `unordered_list` (carina#3093).
        for key in [
            "DefaultCacheBehavior.AllowedMethods",
            "CacheBehavior.AllowedMethods",
            "DefaultCacheBehavior.CachedMethods",
            "CacheBehavior.CachedMethods",
        ] {
            assert!(
                matches!(
                    table.get(&("AWS::CloudFront::Distribution", key)),
                    Some(TypeOverride::EnumUnordered(_))
                ),
                "carina#3093: {key} must be EnumUnordered"
            );
        }
    }

    #[test]
    fn test_nested_field_enum_overlay_promotes_string_to_string_enum() {
        // A struct field whose CFN type is plain `string` (no enum, no
        // pattern) but appears in `resource_type_overrides()` keyed by
        // "DefName.FieldName" must emit as StringEnum, not String.
        let mut config_props = BTreeMap::new();
        config_props.insert(
            "PriceClass".to_string(),
            CfnProperty {
                prop_type: Some(TypeValue::Single("string".to_string())),
                description: None,
                enum_values: None,
                items: None,
                ref_path: None,
                insertion_order: None,
                properties: None,
                required: vec![],
                minimum: None,
                maximum: None,
                additional_properties: None,
                const_value: None,
                default_value: None,
                pattern: None,
                min_items: None,
                max_items: None,
                min_length: None,
                max_length: None,
                format: None,
                ..Default::default()
            },
        );
        let mut definitions = BTreeMap::new();
        definitions.insert(
            "DistributionConfig".to_string(),
            CfnDefinition {
                def_type: Some("object".to_string()),
                properties: Some(config_props),
                ..Default::default()
            },
        );
        let mut top = BTreeMap::new();
        top.insert(
            "DistributionConfig".to_string(),
            CfnProperty {
                prop_type: None,
                description: None,
                enum_values: None,
                items: None,
                ref_path: Some("#/definitions/DistributionConfig".to_string()),
                insertion_order: None,
                properties: None,
                required: vec![],
                minimum: None,
                maximum: None,
                additional_properties: None,
                const_value: None,
                default_value: None,
                pattern: None,
                min_items: None,
                max_items: None,
                min_length: None,
                max_length: None,
                format: None,
                ..Default::default()
            },
        );
        let schema = CfnSchema {
            type_name: "AWS::CloudFront::Distribution".to_string(),
            description: None,
            properties: top,
            required: vec![],
            read_only_properties: vec![],
            create_only_properties: vec![],
            write_only_properties: vec![],
            primary_identifier: None,
            definitions: Some(definitions),
            tagging: None,
            one_of: vec![],
            any_of: vec![],
            handlers: BTreeMap::new(),
        };
        let generated = generate_schema_code(&schema, "AWS::CloudFront::Distribution").unwrap();
        // The struct field must emit as Enum, not String.
        assert!(
            generated.contains("StructField::new(\"price_class\", AttributeType::enum_("),
            "nested-field overlay should promote price_class to Enum: {generated}"
        );
        // All three documented values must be present.
        for v in ["PriceClass_100", "PriceClass_200", "PriceClass_All"] {
            assert!(
                generated.contains(&format!("\"{}\"", v)), // rust-lit-guard: allow
                "missing override value {v}: {generated}"
            );
        }
        // The override must register in enum_valid_values() so DSL value
        // lookup and did-you-mean diagnostics fire on it.
        assert!(
            generated.contains("VALID_DISTRIBUTION_CONFIG_PRICE_CLASS"),
            "VALID_* constant for the nested override should be emitted: {generated}"
        );
        assert!(
            generated.contains("(\"price_class\", VALID_DISTRIBUTION_CONFIG_PRICE_CLASS)"),
            "enum_valid_values() should include the nested override: {generated}"
        );
    }

    #[test]
    fn test_nested_field_enum_overlay_promotes_list_string_to_list_string_enum() {
        // List fields (CFN `array<string>`) overridden via the nested-field
        // overlay must produce `list(StringEnum)`, not `list(String)`.
        let mut behavior_props = BTreeMap::new();
        behavior_props.insert(
            "AllowedMethods".to_string(),
            CfnProperty {
                prop_type: Some(TypeValue::Single("array".to_string())),
                description: None,
                enum_values: None,
                items: Some(Box::new(CfnProperty {
                    prop_type: Some(TypeValue::Single("string".to_string())),
                    description: None,
                    enum_values: None,
                    items: None,
                    ref_path: None,
                    insertion_order: None,
                    properties: None,
                    required: vec![],
                    minimum: None,
                    maximum: None,
                    additional_properties: None,
                    const_value: None,
                    default_value: None,
                    pattern: None,
                    min_items: None,
                    max_items: None,
                    min_length: None,
                    max_length: None,
                    format: None,
                    ..Default::default()
                })),
                ref_path: None,
                insertion_order: None,
                properties: None,
                required: vec![],
                minimum: None,
                maximum: None,
                additional_properties: None,
                const_value: None,
                default_value: None,
                pattern: None,
                min_items: None,
                max_items: None,
                min_length: None,
                max_length: None,
                format: None,
                ..Default::default()
            },
        );
        let mut definitions = BTreeMap::new();
        definitions.insert(
            "DefaultCacheBehavior".to_string(),
            CfnDefinition {
                def_type: Some("object".to_string()),
                properties: Some(behavior_props),
                ..Default::default()
            },
        );
        let mut top = BTreeMap::new();
        top.insert(
            "DefaultCacheBehavior".to_string(),
            CfnProperty {
                prop_type: None,
                description: None,
                enum_values: None,
                items: None,
                ref_path: Some("#/definitions/DefaultCacheBehavior".to_string()),
                insertion_order: None,
                properties: None,
                required: vec![],
                minimum: None,
                maximum: None,
                additional_properties: None,
                const_value: None,
                default_value: None,
                pattern: None,
                min_items: None,
                max_items: None,
                min_length: None,
                max_length: None,
                format: None,
                ..Default::default()
            },
        );
        let schema = CfnSchema {
            type_name: "AWS::CloudFront::Distribution".to_string(),
            description: None,
            properties: top,
            required: vec![],
            read_only_properties: vec![],
            create_only_properties: vec![],
            write_only_properties: vec![],
            primary_identifier: None,
            definitions: Some(definitions),
            tagging: None,
            one_of: vec![],
            any_of: vec![],
            handlers: BTreeMap::new(),
        };
        let generated = generate_schema_code(&schema, "AWS::CloudFront::Distribution").unwrap();
        // The list wrapping is kept and the element promoted to
        // Enum. `AllowedMethods` is an order-insensitive set, so
        // post-carina#3093 the list is `unordered_list` (the overlay
        // still wraps a list — only the ordering flag changed).
        assert!(
            generated.contains("AttributeType::unordered_list(AttributeType::enum_("),
            "nested list-of-string overlay should produce unordered_list(Enum) \
             for AllowedMethods (carina#3093): {generated}"
        );
        // No plain `String`-element list for this field, ordered or not.
        assert!(
            !generated
                .contains("\"allowed_methods\", AttributeType::list(AttributeType::string())")
                && !generated.contains(
                    "\"allowed_methods\", AttributeType::unordered_list(AttributeType::string())"
                ),
            "list(String) must be replaced for overridden list field: {generated}"
        );
        for v in ["GET", "HEAD", "OPTIONS", "PUT", "PATCH", "POST", "DELETE"] {
            assert!(
                generated.contains(&format!("\"{}\".to_string()", v)), // rust-lit-guard: allow
                "missing override value {v}: {generated}"
            );
        }
    }
}
