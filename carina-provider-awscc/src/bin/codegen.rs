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
use carina_aws_types::dsl_enum_value;
use carina_docgen::{Heading, PageBuilder, TableCell};
use carina_provider_awscc::schemas::catalog::AwsccType;
use clap::Parser;
use heck::{ToPascalCase, ToSnakeCase};
use regex::Regex;
use serde::Deserialize;
use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::io::{self, Read};
use std::path::{Path, PathBuf};
use std::sync::LazyLock;

// carina#3340: in-progress recursion guard for cyclic CFN $ref
// definitions (WAFv2 `WebACL.Statement` → `AndStatement` → `List<Statement>`).
// When `generate_struct_type` is asked to expand a definition whose name
// is already on the stack, it emits `AttributeType::Ref(name)` and
// records the def's body into [`STRUCT_DEFS_OUT`] for later emission
// into `ResourceSchema.defs`. The set is per-resource — cleared at the
// top of each resource generation pass.
#[derive(Debug, Clone)]
struct StructReadOnlyUse {
    property_path: Vec<String>,
    signature: BTreeSet<Vec<String>>,
}

#[derive(Debug, Clone)]
struct StructWriteOnlyUse {
    property_path: Vec<String>,
    signature: BTreeSet<Vec<String>>,
}

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

fn emit_primary_identifier(
    primary_identifier: Option<&[String]>,
    properties: &BTreeMap<String, CfnProperty>,
) -> Result<String> {
    let Some(primary_identifier) = primary_identifier else {
        return Ok("&[]".to_string());
    };
    let props: Vec<String> = primary_identifier
        .iter()
        .map(|path| {
            let prop = path.strip_prefix("/properties/").ok_or_else(|| {
                anyhow::anyhow!("primaryIdentifier path {path:?} is not a property path")
            })?;
            if !properties.contains_key(prop) {
                anyhow::bail!("primaryIdentifier property {prop:?} is not declared in properties");
            }
            Ok(format!(
                "crate::schemas::config::PrimaryIdentifierAttribute {{ provider_name: {}, dsl_name: {} }}",
                rust_lit(prop),
                rust_lit(&prop.to_snake_case())
            ))
        })
        .collect::<Result<_>>()?;
    if props.is_empty() {
        Ok("&[]".to_string())
    } else {
        Ok(format!("&[{}]", props.join(", ")))
    }
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
    /// Definition names for which generation emitted an `AttributeType::Ref`
    /// because the definition was already in progress. List-item structs use
    /// this to publish a schema def only when their inline body is recursive.
    #[allow(clippy::missing_const_for_thread_local)]
    static BACK_REFERENCED_DEFS: RefCell<HashSet<String>> = RefCell::new(HashSet::new());
    /// Collected `(def_name, AttributeType-rendered Rust code)` for the
    /// resource currently being generated. Populated by the recursion
    /// guard in `cfn_type_to_carina_type_with_enum`; consumed by the
    /// resource emitter to populate `ResourceSchema.defs`.
    #[allow(clippy::missing_const_for_thread_local)]
    static EMITTED_DEFS: RefCell<BTreeMap<String, String>> = RefCell::new(BTreeMap::new());
    /// Provider paths whose generated `AttributeSchema` or `StructField`
    /// actually uses the `provider_populated` constructor in the current
    /// emission pass.
    #[allow(clippy::missing_const_for_thread_local)]
    static EMITTED_READ_ONLY_PATHS: RefCell<Vec<Vec<String>>> = RefCell::new(Vec::new());
    /// Read-only signatures produced by each emitted struct name. Reusing a
    /// name with a different signature is unsafe for both Rust definition
    /// caching and Markdown struct collection.
    #[allow(clippy::missing_const_for_thread_local)]
    static EMITTED_STRUCT_READ_ONLY_USES: RefCell<BTreeMap<String, Vec<StructReadOnlyUse>>> =
        RefCell::new(BTreeMap::new());
    /// Provider paths whose generated `AttributeSchema` or `StructField`
    /// actually carries `.write_only()` in the current emission pass.
    #[allow(clippy::missing_const_for_thread_local)]
    static EMITTED_WRITE_ONLY_PATHS: RefCell<Vec<Vec<String>>> = RefCell::new(Vec::new());
    /// Nested write-only paths deliberately omitted because the target
    /// property is deprecated. These paths satisfy fail-closed validation
    /// without restoring deprecated fields to the generated DSL surface.
    #[allow(clippy::missing_const_for_thread_local)]
    static SKIPPED_DEPRECATED_WRITE_ONLY_PATHS: RefCell<BTreeSet<Vec<String>>> =
        RefCell::new(BTreeSet::new());
    /// Write-only signatures produced by each emitted struct name. Reusing a
    /// name with a different signature is unsafe for both Rust definition
    /// caching and Markdown struct collection.
    #[allow(clippy::missing_const_for_thread_local)]
    static EMITTED_STRUCT_WRITE_ONLY_USES: RefCell<BTreeMap<String, Vec<StructWriteOnlyUse>>> =
        RefCell::new(BTreeMap::new());
}

/// Reset the per-resource recursion guard state. Called at the start
/// of every resource generation so a cycle detected for resource A
/// does not leak into resource B.
fn reset_recursion_guard() {
    IN_PROGRESS_DEFS.with(|s| s.borrow_mut().clear());
    BACK_REFERENCED_DEFS.with(|s| s.borrow_mut().clear());
    EMITTED_DEFS.with(|s| s.borrow_mut().clear());
    reset_property_mode_emission_state();
}

fn mark_back_reference(def_name: &str) {
    BACK_REFERENCED_DEFS.with(|defs| {
        defs.borrow_mut().insert(def_name.to_string());
    });
}

fn was_back_referenced(def_name: &str) -> bool {
    BACK_REFERENCED_DEFS.with(|defs| defs.borrow().contains(def_name))
}

fn reset_property_mode_emission_state() {
    EMITTED_READ_ONLY_PATHS.with(|s| s.borrow_mut().clear());
    EMITTED_STRUCT_READ_ONLY_USES.with(|s| s.borrow_mut().clear());
    EMITTED_WRITE_ONLY_PATHS.with(|s| s.borrow_mut().clear());
    SKIPPED_DEPRECATED_WRITE_ONLY_PATHS.with(|s| s.borrow_mut().clear());
    EMITTED_STRUCT_WRITE_ONLY_USES.with(|s| s.borrow_mut().clear());
}

/// Take the defs collected during the current resource pass and
/// return them as a sorted `BTreeMap`. Empty for non-cyclic resources.
fn take_emitted_defs() -> BTreeMap<String, String> {
    EMITTED_DEFS.with(|s| std::mem::take(&mut *s.borrow_mut()))
}

/// Exit status used when a schema is deliberately skipped (NON_PROVISIONABLE).
/// The generate-schemas.sh / generate-docs.sh wrappers branch on this to
/// distinguish intentional skips from real errors.
const EXIT_SKIPPED: i32 = 2;

/// Unified type override for resource-scoped property overrides.
/// Allows overriding string type, enum values, numeric shape, integer range,
/// or integer enum for a specific (resource_type, property_name) pair.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct AttributeTypeOverrideDefault {
    value_expression: &'static str,
    markdown: &'static str,
}

#[derive(Debug, Clone, PartialEq)]
enum TypeOverride {
    /// Override to a specific string type (e.g., "super::super::iam::role::arn()")
    StringType(&'static str),
    /// Replace the whole generated attribute type and its Markdown display.
    /// Used when the CFN schema omits enough shape information that normal
    /// inference cannot express the provider API type.
    AttributeType {
        expression: &'static str,
        display: &'static str,
        block_name: Option<&'static str>,
        default: Option<AttributeTypeOverrideDefault>,
    },
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
    /// Treat a CFN `number` property as an integer while retaining its CFN bounds.
    IntegerFromNumber,
    /// Override to an integer enum with specific allowed values
    IntEnum(Vec<i64>),
    /// Override to_dsl on a Custom type (Rust code for the closure)
    ToDsl(&'static str),
}

impl TypeOverride {
    fn attribute_type_expression(&self) -> Option<&'static str> {
        match self {
            Self::StringType(expression) | Self::AttributeType { expression, .. } => {
                Some(expression)
            }
            _ => None,
        }
    }
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
enum ReplacementClass {
    Attribute(&'static str),
    Coexisting,
    Conflicting,
}

const REPLACEMENT_CLASSES: &[(&str, ReplacementClass)] = &[
    // Alarm names are account/region-unique when set. CloudFormation generates
    // one when omitted, so named replacements can use a temporary name.
    (
        "cloudwatch.Alarm",
        ReplacementClass::Attribute("alarm_name"),
    ),
    // Distribution aliases are globally unique and cannot coexist.
    ("cloudfront.Distribution", ReplacementClass::Conflicting),
    // The top-level create-only name is account-unique, so replacement must
    // temporarily rename it.
    ("cloudfront.Function", ReplacementClass::Attribute("name")),
    // The account-unique name is nested, so it cannot be temporarily renamed.
    (
        "cloudfront.OriginAccessControl",
        ReplacementClass::Conflicting,
    ),
    // The account-unique name is nested in the config, so it cannot be temporarily renamed.
    (
        "cloudfront.ResponseHeadersPolicy",
        ReplacementClass::Conflicting,
    ),
    // Table names are user-chosen and unique within an account and region.
    ("dynamodb.Table", ReplacementClass::Attribute("table_name")),
    // Address and transfer_address pin a specific IP that cannot coexist.
    ("ec2.Eip", ReplacementClass::Conflicting),
    // Replacement changes vpc_id, so it targets a different VPC.
    (
        "ec2.EgressOnlyInternetGateway",
        ReplacementClass::Coexisting,
    ),
    // AWS rejects duplicate flow logs for the same source, traffic, and destination.
    ("ec2.FlowLog", ReplacementClass::Conflicting),
    // The default regional quota permits only one IPAM.
    ("ec2.Ipam", ReplacementClass::Conflicting),
    // Provisioned CIDRs must not overlap within an IPAM scope.
    ("ec2.IpamPool", ReplacementClass::Conflicting),
    // Internet gateways use server-generated identities without a unique name.
    ("ec2.InternetGateway", ReplacementClass::Coexisting),
    // An EIP can be associated with only one NAT gateway during replacement.
    ("ec2.NatGateway", ReplacementClass::Conflicting),
    // Every create-only field participates in a route's uniqueness key.
    ("ec2.Route", ReplacementClass::Coexisting),
    // Replacement changes vpc_id, so it targets a different VPC.
    ("ec2.RouteTable", ReplacementClass::Coexisting),
    // Group names are unique within a VPC and AWS generates one when omitted.
    (
        "ec2.SecurityGroup",
        ReplacementClass::Attribute("group_name"),
    ),
    // Some create-only changes respell the same rule (group_name vs group_id,
    // ports ignored for protocol -1), which AWS rejects as a duplicate.
    ("ec2.SecurityGroupEgress", ReplacementClass::Conflicting),
    // Some create-only changes respell the same rule (group_name vs group_id,
    // ports ignored for protocol -1), which AWS rejects as a duplicate.
    ("ec2.SecurityGroupIngress", ReplacementClass::Conflicting),
    // CIDRs cannot overlap when an AZ-driven replacement keeps the same VPC and CIDR.
    ("ec2.Subnet", ReplacementClass::Conflicting),
    // A subnet can have only one route table association.
    (
        "ec2.SubnetRouteTableAssociation",
        ReplacementClass::Conflicting,
    ),
    // Transit gateways use server-generated identities without a unique name.
    ("ec2.TransitGateway", ReplacementClass::Coexisting),
    // Replacement changes the transit-gateway/VPC pair, which can coexist.
    ("ec2.TransitGatewayAttachment", ReplacementClass::Coexisting),
    // VPCs use server-generated identities without a unique name.
    ("ec2.Vpc", ReplacementClass::Coexisting),
    // A VPC or route table cannot have a duplicate endpoint for the same service.
    ("ec2.VpcEndpoint", ReplacementClass::Conflicting),
    // A VPC can have only one attached internet gateway.
    ("ec2.VpcGatewayAttachment", ReplacementClass::Conflicting),
    // Only one active peering connection may exist for a VPC pair.
    ("ec2.VpcPeeringConnection", ReplacementClass::Conflicting),
    // VPN gateways use server-generated identities without a unique name.
    ("ec2.VpnGateway", ReplacementClass::Coexisting),
    // Repository names are user-chosen and unique within a registry.
    (
        "ecr.Repository",
        ReplacementClass::Attribute("repository_name"),
    ),
    // Cluster names are user-chosen and unique within an account and region.
    ("ecs.Cluster", ReplacementClass::Attribute("cluster_name")),
    // A target group associates with one load balancer, so the new listener's forward
    // action conflicts with the old one.
    (
        "elasticloadbalancingv2.Listener",
        ReplacementClass::Conflicting,
    ),
    // subnet_mappings can pin an EIP allocation or private IPv4 address that the
    // replacement cannot share with the original.
    (
        "elasticloadbalancingv2.LoadBalancer",
        ReplacementClass::Conflicting,
    ),
    // Target group names are unique per account and region.
    (
        "elasticloadbalancingv2.TargetGroup",
        ReplacementClass::Attribute("name"),
    ),
    // The URL is both the uniqueness key and the only create-only field.
    ("iam.OidcProvider", ReplacementClass::Coexisting),
    // Role names are user-chosen and unique within an account.
    ("iam.Role", ReplacementClass::Attribute("role_name")),
    // Policy names are unique within their containing role.
    ("iam.RolePolicy", ReplacementClass::Attribute("policy_name")),
    // Display names are unique within an identity store.
    (
        "identitystore.Group",
        ReplacementClass::Attribute("display_name"),
    ),
    // Every create-only field participates in the membership's uniqueness key.
    (
        "identitystore.GroupMembership",
        ReplacementClass::Coexisting,
    ),
    // KMS keys use server-generated identities without a unique name.
    ("kms.Key", ReplacementClass::Coexisting),
    // Log group names are user-chosen and unique within an account and region.
    (
        "logs.LogGroup",
        ReplacementClass::Attribute("log_group_name"),
    ),
    // Account email addresses are globally unique.
    ("organizations.Account", ReplacementClass::Conflicting),
    // An account can belong to only one organization.
    ("organizations.Organization", ReplacementClass::Conflicting),
    // Hosted zone names are not unique, and replacement occurs only on name change.
    ("route53.HostedZone", ReplacementClass::Coexisting),
    // Bucket names are user-chosen and globally unique.
    ("s3.Bucket", ReplacementClass::Attribute("bucket_name")),
    // One policy per bucket; `bucket` is the parent, not this resource's own name.
    ("s3.BucketPolicy", ReplacementClass::Conflicting),
    // A temporary-name suffix would make a FIFO topic name stop ending in
    // `.fifo`, so SNS topics cannot use create-before-destroy replacement.
    ("sns.Topic", ReplacementClass::Conflicting),
    // Subscribe is idempotent for a topic/protocol/endpoint tuple and returns
    // the same ARN; create-before-destroy would then delete that subscription.
    ("sns.Subscription", ReplacementClass::Conflicting),
    // BudgetName is nested in Budget and account-unique, so replacement cannot
    // temporarily rename an explicitly named budget.
    ("budgets.Budget", ReplacementClass::Conflicting),
    // Schedule names are user-chosen and replacement can use a temporary name
    // within the same schedule group.
    ("scheduler.Schedule", ReplacementClass::Attribute("name")),
    // Schedule group names are user-chosen and unique within an account and region.
    (
        "scheduler.ScheduleGroup",
        ReplacementClass::Attribute("name"),
    ),
    // Parameter names are user-chosen and unique within an account and region.
    ("ssm.Parameter", ReplacementClass::Attribute("name")),
    // Every create-only field participates in the assignment's uniqueness key.
    ("sso.Assignment", ReplacementClass::Coexisting),
    // An account can have only one IAM Identity Center instance.
    ("sso.Instance", ReplacementClass::Conflicting),
    // Permission set names are unique within an IAM Identity Center instance.
    ("sso.PermissionSet", ReplacementClass::Attribute("name")),
    // Web ACL names are unique within a scope and region.
    ("wafv2.WebAcl", ReplacementClass::Attribute("name")),
    // A resource holds one web ACL; a web_acl_arn change keeps resource_arn.
    ("wafv2.WebAclAssociation", ReplacementClass::Conflicting),
];

fn resource_identity_parts(name: &str) -> Option<(&str, &str)> {
    name.split_once('.')
}

fn is_resource_identity(
    identity: &carina_core::schema::TypeIdentity,
    service: &str,
    resource: &str,
    kind: &str,
) -> bool {
    identity.provider.as_deref() == Some("aws")
        && identity.kind == kind
        && matches!(
            identity.segments.as_slice(),
            [identity_service, identity_resource]
                if identity_service == service && identity_resource == resource
        )
}

fn catalog_knows_arn_service(service: &str) -> bool {
    AwsccType::ALL
        .iter()
        .copied()
        .map(AwsccType::identity)
        .chain(
            carina_aws_types::AwsType::ALL
                .iter()
                .copied()
                .map(carina_aws_types::AwsType::identity),
        )
        .any(|identity| {
            identity.kind == "Arn"
                && identity
                    .segments
                    .first()
                    .is_some_and(|known| known == service)
        })
}

fn arn_constructor(service: &str, resource: &str) -> Result<String> {
    if let Some(awscc_type) = AwsccType::ALL
        .iter()
        .copied()
        .find(|awscc_type| is_resource_identity(&awscc_type.identity(), service, resource, "Arn"))
    {
        return Ok(awscc_type.constructor_path().to_string());
    }

    if let Some(aws_type) = carina_aws_types::AwsType::ALL
        .iter()
        .copied()
        .find(|aws_type| is_resource_identity(&aws_type.identity(), service, resource, "Arn"))
    {
        return Ok(format!(
            "carina_aws_types::AwsType::{aws_type:?}.attribute_type()"
        ));
    }

    if catalog_knows_arn_service(service) {
        anyhow::bail!(
            "AWS service {service:?} is cataloged, but resource {resource:?} has no Arn row; add it to AwsType or AwsccType"
        );
    }

    Ok("carina_aws_types::arn()".to_string())
}

fn emit_arn_helper(service: &str, resource: &str) -> Result<String> {
    let constructor = arn_constructor(service, resource)?;
    Ok(format!(
        r#"pub fn arn() -> AttributeType {{
    {constructor}
}}

"#
    ))
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

    /// Print a helper-only module containing the catalog-derived ARN constructor
    #[arg(long)]
    print_arn_helper_module: bool,

    /// Output format: rust (default) or markdown (for documentation)
    #[arg(long, default_value = "rust")]
    format: String,

    /// Directory containing generated-doc examples (`<service>_<resource>/main.crn`).
    #[arg(long)]
    examples_dir: Option<PathBuf>,
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
    /// Cloud Control handlers (create/read/update/delete/list). Most
    /// NON_PROVISIONABLE types omit this map, but the registry has known
    /// provisionable exceptions; see [`HANDLERLESS_CLOUD_CONTROL_TYPES`].
    #[serde(default)]
    handlers: BTreeMap<String, serde_json::Value>,
}

#[derive(Debug)]
struct HandlerlessPermissions {
    create: &'static [&'static str],
    read: &'static [&'static str],
    update: &'static [&'static str],
    delete: &'static [&'static str],
    list: &'static [&'static str],
}

impl HandlerlessPermissions {
    const fn complete(
        create: &'static [&'static str],
        read: &'static [&'static str],
        update: &'static [&'static str],
        delete: &'static [&'static str],
        list: &'static [&'static str],
    ) -> Self {
        assert!(
            !create.is_empty(),
            "handlerless create permissions are empty"
        );
        assert!(!read.is_empty(), "handlerless read permissions are empty");
        assert!(
            !update.is_empty(),
            "handlerless update permissions are empty"
        );
        assert!(
            !delete.is_empty(),
            "handlerless delete permissions are empty"
        );
        assert!(!list.is_empty(), "handlerless list permissions are empty");
        Self {
            create,
            read,
            update,
            delete,
            list,
        }
    }

    fn for_operation(&self, operation: &str) -> &'static [&'static str] {
        match operation {
            "create" => self.create,
            "read" => self.read,
            "update" => self.update,
            "delete" => self.delete,
            "list" => self.list,
            _ => panic!("unknown Cloud Control handler operation: {operation}"),
        }
    }
}

#[derive(Debug)]
struct HandlerlessCloudControlType {
    type_name: &'static str,
    permissions: HandlerlessPermissions,
}

impl HandlerlessCloudControlType {
    const fn new(type_name: &'static str, permissions: HandlerlessPermissions) -> Self {
        assert!(
            !type_name.is_empty(),
            "handlerless Cloud Control type is empty"
        );
        Self {
            type_name,
            permissions,
        }
    }
}

// Measured on 2026-10-30: CloudFormation DescribeType reported
// AWS::Budgets::Budget as FULLY_MUTABLE, and Cloud Control ListResources
// succeeded in us-east-1, even though the live registry schema has no handlers block.
// Keep this fail-closed: every further handlerless exception needs its own
// measured ProvisioningType and Cloud Control operation evidence.
//
// IAM operation mappings and their aws-portal dependencies:
// https://docs.aws.amazon.com/service-authorization/latest/reference/list_budgets.html
// ResourceTags is supported on create and update, requiring the documented
// list/tag/untag operations in the corresponding lifecycle handlers:
// https://docs.aws.amazon.com/AWSCloudFormation/latest/TemplateReference/aws-resource-budgets-budget.html
const HANDLERLESS_CLOUD_CONTROL_TYPES: &[HandlerlessCloudControlType] =
    &[HandlerlessCloudControlType::new(
        "AWS::Budgets::Budget",
        HandlerlessPermissions::complete(
            &[
                "aws-portal:ModifyBilling",
                "budgets:ModifyBudget",
                "budgets:TagResource",
            ],
            &[
                "aws-portal:ViewBilling",
                "budgets:ListTagsForResource",
                "budgets:ViewBudget",
            ],
            &[
                "aws-portal:ModifyBilling",
                "aws-portal:ViewBilling",
                "budgets:ListTagsForResource",
                "budgets:ModifyBudget",
                "budgets:TagResource",
                "budgets:UntagResource",
            ],
            &["aws-portal:ModifyBilling", "budgets:ModifyBudget"],
            &["aws-portal:ViewBilling", "budgets:ViewBudget"],
        ),
    )];

enum CloudControlPermissionSource<'a> {
    SchemaHandlers(&'a BTreeMap<String, serde_json::Value>),
    Handlerless(&'static HandlerlessCloudControlType),
}

impl CfnSchema {
    /// Returns true if the schema declares any Cloud Control handlers.
    fn has_cloud_control_handlers(&self) -> bool {
        !self.handlers.is_empty()
    }

    fn cloud_control_permission_source(&self) -> Option<CloudControlPermissionSource<'_>> {
        if let Some(entry) = HANDLERLESS_CLOUD_CONTROL_TYPES
            .iter()
            .find(|entry| entry.type_name == self.type_name)
        {
            assert!(
                self.handlers.is_empty(),
                "{} now declares Cloud Control handlers; remove its handlerless allowlist entry",
                self.type_name
            );
            Some(CloudControlPermissionSource::Handlerless(entry))
        } else if self.has_cloud_control_handlers() {
            Some(CloudControlPermissionSource::SchemaHandlers(&self.handlers))
        } else {
            None
        }
    }

    /// Returns true when the registry evidence says Cloud Control can operate
    /// this resource, including measured exceptions whose schemas omit handlers.
    fn is_cloud_control_provisionable(&self) -> bool {
        self.cloud_control_permission_source().is_some()
    }
}

fn handler_permissions(schema: &CfnSchema, operation: &str) -> Vec<String> {
    match schema.cloud_control_permission_source() {
        Some(CloudControlPermissionSource::Handlerless(entry)) => entry
            .permissions
            .for_operation(operation)
            .iter()
            .map(|permission| (*permission).to_string())
            .collect(),
        Some(CloudControlPermissionSource::SchemaHandlers(handlers)) => handlers
            .get(operation)
            .and_then(|handler| handler.get("permissions"))
            .and_then(serde_json::Value::as_array)
            .map(|permissions| {
                permissions
                    .iter()
                    .filter_map(serde_json::Value::as_str)
                    .map(str::to_string)
                    .collect()
            })
            .unwrap_or_default(),
        None => Vec::new(),
    }
}

fn emit_permissions_slice(permissions: &[String]) -> String {
    if permissions.is_empty() {
        "&[]".to_string()
    } else {
        let entries = permissions
            .iter()
            .map(|permission| format!("        {},", rust_lit(permission)))
            .collect::<Vec<_>>()
            .join("\n");
        format!("&[\n{entries}\n    ]")
    }
}

/// CloudFormation Tagging metadata
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
struct CfnTagging {
    #[serde(default)]
    taggable: bool,
    tag_property: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum GeneratedTagsWire {
    KeyValueList,
    Map,
}

impl GeneratedTagsWire {
    fn rust_variant(self) -> &'static str {
        match self {
            Self::KeyValueList => "TagsWire::KeyValueList",
            Self::Map => "TagsWire::Map",
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
struct GeneratedTagsProperty {
    provider_name: String,
    wire: GeneratedTagsWire,
}

/// Whether this CloudFormation property is generated as the canonical DSL
/// `tags: Map<String, String>` attribute.
fn is_dsl_tags_property(schema: &CfnSchema, provider_name: &str) -> bool {
    let Some(tagging) = schema.tagging.as_ref().filter(|tagging| tagging.taggable) else {
        return false;
    };
    let tag_property = tagging
        .tag_property
        .as_deref()
        .unwrap_or("/properties/Tags");

    tag_property.strip_prefix("/properties/") == Some(provider_name)
        && provider_name.to_snake_case() == "tags"
}

/// Whether a definition uses one of Carina's canonical tag-map shapes.
///
/// The `Tag` prefix remains a namespace guard because cached schemas also use
/// the exact same Key/Value string-object shape for non-tag ELB attributes.
/// The shape check prevents unrelated `Tag*` objects from silently losing
/// fields when converted to Carina's map representation. CloudFormation uses
/// either a Key/Value object or a single-pattern string map for these refs.
fn is_tag_map_ref(schema: &CfnSchema, ref_path: &str) -> bool {
    let Some(def_name) = ref_def_name(ref_path) else {
        return false;
    };
    let Some(definition) = schema
        .definitions
        .as_ref()
        .and_then(|definitions| definitions.get(def_name))
    else {
        return false;
    };
    let is_direct_string = |property: &CfnProperty| {
        matches!(
            property.prop_type.as_ref(),
            Some(TypeValue::Single(property_type)) if property_type == "string"
        )
    };
    let is_key_value_object = definition.properties.as_ref().is_some_and(|properties| {
        properties.len() == 2
            && properties.get("Key").is_some_and(is_direct_string)
            && properties.get("Value").is_some_and(is_direct_string)
    });
    let is_single_pattern_string_map = definition.properties.is_none()
        && definition
            .pattern_properties
            .as_ref()
            .is_some_and(|patterns| patterns.len() == 1 && patterns.values().all(is_direct_string));

    def_name.starts_with("Tag")
        && definition.def_type.as_deref() == Some("object")
        && (is_key_value_object || is_single_pattern_string_map)
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct CfnPropertyPath<'a> {
    parent: Option<&'a str>,
    property: &'a str,
}

impl<'a> CfnPropertyPath<'a> {
    /// Build the one canonical classification/override key for a CFN
    /// property. `parent` is the raw definition name (or raw inline-property
    /// name), never the PascalCased Rust struct namespace.
    const fn new(parent: Option<&'a str>, property: &'a str) -> Self {
        Self { parent, property }
    }
}

impl std::fmt::Display for CfnPropertyPath<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(parent) = self.parent {
            write!(f, "{parent}.{}", self.property)
        } else {
            f.write_str(self.property)
        }
    }
}

type IamPolicyDocumentProperty = (&'static str, CfnPropertyPath<'static>);

/// IAM policy documents whose names previously matched the implicit
/// `*PolicyDocument` suffix rule. The cache contains only these seven paths,
/// so they are explicit and walker-checked like every other classification.
const IAM_POLICY_DOCUMENT_PROPERTIES: &[IamPolicyDocumentProperty] = &[
    (
        "AWS::S3::BucketPolicy",
        CfnPropertyPath::new(None, "PolicyDocument"),
    ),
    (
        "AWS::Logs::LogGroup",
        CfnPropertyPath::new(None, "ResourcePolicyDocument"),
    ),
    (
        "AWS::EC2::VPCEndpoint",
        CfnPropertyPath::new(None, "PolicyDocument"),
    ),
    (
        "AWS::IAM::RolePolicy",
        CfnPropertyPath::new(None, "PolicyDocument"),
    ),
    (
        "AWS::IAM::Role",
        CfnPropertyPath::new(Some("Policy"), "PolicyDocument"),
    ),
    (
        "AWS::IAM::Role",
        CfnPropertyPath::new(None, "AssumeRolePolicyDocument"),
    ),
    (
        "AWS::DynamoDB::Table",
        CfnPropertyPath::new(Some("ResourcePolicy"), "PolicyDocument"),
    ),
];

/// IAM policy document properties whose CloudFormation names do not use the
/// conventional `PolicyDocument` suffix.
const NONSTANDARD_IAM_POLICY_DOCUMENT_PROPERTIES: &[IamPolicyDocumentProperty] = &[
    ("AWS::KMS::Key", CfnPropertyPath::new(None, "KeyPolicy")),
    (
        "AWS::ECR::Repository",
        CfnPropertyPath::new(None, "RepositoryPolicyText"),
    ),
    (
        "AWS::SSO::PermissionSet",
        CfnPropertyPath::new(None, "InlinePolicy"),
    ),
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ClassifiedProperty {
    cfn_type: &'static str,
    property_path: CfnPropertyPath<'static>,
    reason: &'static str,
}

/// Free-form JSON properties omitted until Carina has an `AttributeType` for
/// recursive, heterogeneous JSON documents. Neither String nor
/// Map<String, String> preserves Cloud Control's canonical object read-back.
///
/// These properties come back together when the JSON-document type from
/// [carina-rs/carina#4144](https://github.com/carina-rs/carina/issues/4144)
/// lands.
const FREE_FORM_JSON_PROPERTIES: &[ClassifiedProperty] = &[ClassifiedProperty {
    cfn_type: "AWS::SNS::Subscription",
    property_path: CfnPropertyPath::new(None, "FilterPolicy"),
    reason: "Filter keys are user-defined message attribute or body-property names, and the operator grammar nests recursively; this remains blocked on carina-rs/carina#4144.",
}];

#[cfg(test)]
const TEST_FREE_FORM_JSON_PROPERTIES: &[ClassifiedProperty] = &[ClassifiedProperty {
    cfn_type: "AWS::Test::NestedOmission",
    property_path: CfnPropertyPath::new(Some("ParentConfig"), "ChildPolicy"),
    reason: "Synthetic nested omission used to exercise the canonical raw-CFN path.",
}];

#[cfg(not(test))]
const TEST_FREE_FORM_JSON_PROPERTIES: &[ClassifiedProperty] = &[];

/// Bare objects whose only meaningful provider value is `{}`. These must be
/// represented as empty structs so the DSL accepts `field = {}` while still
/// rejecting arbitrary keys.
const EMPTY_OBJECT_PROPERTIES: &[ClassifiedProperty] = &[
    ClassifiedProperty {
        cfn_type: "AWS::CloudWatch::Alarm",
        property_path: CfnPropertyPath::new(Some("EvaluationWindow"), "SlidingWindow"),
        reason: "SlidingWindow is a marker selecting the rolling evaluation-window mode.",
    },
    ClassifiedProperty {
        cfn_type: "AWS::S3::Bucket",
        property_path: CfnPropertyPath::new(Some("TargetObjectKeyFormat"), "SimplePrefix"),
        reason: "SimplePrefix is a marker selecting the non-partitioned log-object key format.",
    },
    ClassifiedProperty {
        cfn_type: "AWS::WAFv2::WebACL",
        property_path: CfnPropertyPath::new(Some("FieldToMatch"), "AllQueryArguments"),
        reason: "AllQueryArguments is an empty marker selecting every query argument.",
    },
    ClassifiedProperty {
        cfn_type: "AWS::WAFv2::WebACL",
        property_path: CfnPropertyPath::new(Some("FieldToMatch"), "UriPath"),
        reason: "UriPath is an empty marker selecting the request URI path.",
    },
    ClassifiedProperty {
        cfn_type: "AWS::WAFv2::WebACL",
        property_path: CfnPropertyPath::new(Some("FieldToMatch"), "QueryString"),
        reason: "QueryString is an empty marker selecting the request query string.",
    },
    ClassifiedProperty {
        cfn_type: "AWS::WAFv2::WebACL",
        property_path: CfnPropertyPath::new(Some("FieldToMatch"), "Method"),
        reason: "Method is an empty marker selecting the request HTTP method.",
    },
    ClassifiedProperty {
        cfn_type: "AWS::WAFv2::WebACL",
        property_path: CfnPropertyPath::new(Some("OverrideAction"), "Count"),
        reason: "Count is an empty marker selecting the count override action.",
    },
    ClassifiedProperty {
        cfn_type: "AWS::WAFv2::WebACL",
        property_path: CfnPropertyPath::new(Some("OverrideAction"), "None"),
        reason: "None is an empty marker preserving the referenced rule action.",
    },
    ClassifiedProperty {
        cfn_type: "AWS::WAFv2::WebACL",
        property_path: CfnPropertyPath::new(Some("JsonMatchPattern"), "All"),
        reason: "All is an empty marker selecting every JSON-body element.",
    },
    ClassifiedProperty {
        cfn_type: "AWS::WAFv2::WebACL",
        property_path: CfnPropertyPath::new(Some("HeaderMatchPattern"), "All"),
        reason: "All is an empty marker selecting every request header.",
    },
    ClassifiedProperty {
        cfn_type: "AWS::WAFv2::WebACL",
        property_path: CfnPropertyPath::new(Some("CookieMatchPattern"), "All"),
        reason: "All is an empty marker selecting every request cookie.",
    },
    ClassifiedProperty {
        cfn_type: "AWS::WAFv2::WebACL",
        property_path: CfnPropertyPath::new(Some("RateBasedStatementCustomKey"), "ForwardedIP"),
        reason: "ForwardedIP is an empty marker selecting the forwarded IP custom key.",
    },
    ClassifiedProperty {
        cfn_type: "AWS::WAFv2::WebACL",
        property_path: CfnPropertyPath::new(Some("RateBasedStatementCustomKey"), "HTTPMethod"),
        reason: "HTTPMethod is an empty marker selecting the HTTP method custom key.",
    },
    ClassifiedProperty {
        cfn_type: "AWS::WAFv2::WebACL",
        property_path: CfnPropertyPath::new(Some("RateBasedStatementCustomKey"), "IP"),
        reason: "IP is an empty marker selecting the source IP custom key.",
    },
    ClassifiedProperty {
        cfn_type: "AWS::WAFv2::WebACL",
        property_path: CfnPropertyPath::new(Some("RateBasedStatementCustomKey"), "ASN"),
        reason: "ASN is an empty marker selecting the autonomous-system-number custom key.",
    },
];

#[cfg(test)]
const TEST_EMPTY_OBJECT_PROPERTIES: &[ClassifiedProperty] = &[
    ClassifiedProperty {
        cfn_type: "AWS::Test::AcronymPath",
        property_path: CfnPropertyPath::new(Some("SSESpecification"), "Marker"),
        reason: "Synthetic acronym-parent marker used to verify raw-CFN path spelling.",
    },
    ClassifiedProperty {
        cfn_type: "AWS::Test::ArrayReference",
        property_path: CfnPropertyPath::new(None, "Markers"),
        reason: "Synthetic array item used to verify classification at the referencing path.",
    },
];

#[cfg(not(test))]
const TEST_EMPTY_OBJECT_PROPERTIES: &[ClassifiedProperty] = &[];

/// Bare objects that are genuinely homogeneous string-to-string maps.
///
/// The current cache has no such *bare* object: known string maps publish
/// `patternProperties` and are handled by their existing map/tag paths. This
/// explicit table is the only place a future bare string map may opt in.
const STRING_MAP_PROPERTIES: &[ClassifiedProperty] = &[];

/// Otherwise representable properties deliberately excluded for a documented
/// provider limitation, recorded separately for each row.
const OMITTED_RESOURCE_PROPERTIES: &[ClassifiedProperty] = &[
    ClassifiedProperty {
        cfn_type: "AWS::SNS::Topic",
        property_path: CfnPropertyPath::new(None, "Subscription"),
        reason: "Cloud Control aggregates separately managed topic subscriptions; use awscc.sns.Subscription instead.",
    },
    ClassifiedProperty {
        cfn_type: "AWS::SNS::Topic",
        property_path: CfnPropertyPath::new(None, "DataProtectionPolicy"),
        // Cloud Control create/get was tested in both directions with
        // carina-test-008 on 2026-10-31. AWS rejected this property in
        // ap-northeast-1 and us-east-1 because SNS message data protection is
        // no longer available to new customers, so no real read shape can be
        // verified.
        reason: "AWS SNS message data protection is no longer available to new customers; Cloud Control rejected it in ap-northeast-1 and us-east-1 on 2026-10-31.",
    },
];

fn find_omitted_property<'a>(
    table: &'a [ClassifiedProperty],
    cfn_type: &str,
    property_path: CfnPropertyPath<'_>,
) -> Option<&'a ClassifiedProperty> {
    table
        .iter()
        .find(|entry| entry.cfn_type == cfn_type && entry.property_path == property_path)
}

fn free_form_json_property(
    cfn_type: &str,
    property_path: CfnPropertyPath<'_>,
) -> Option<&'static ClassifiedProperty> {
    find_omitted_property(FREE_FORM_JSON_PROPERTIES, cfn_type, property_path)
        .or_else(|| find_omitted_property(TEST_FREE_FORM_JSON_PROPERTIES, cfn_type, property_path))
}

fn should_omit_property(cfn_type: &str, property_path: CfnPropertyPath<'_>) -> bool {
    free_form_json_property(cfn_type, property_path).is_some()
        || find_omitted_property(OMITTED_RESOURCE_PROPERTIES, cfn_type, property_path).is_some()
}

fn should_omit_top_level_property(cfn_type: &str, property_name: &str) -> bool {
    should_omit_property(cfn_type, CfnPropertyPath::new(None, property_name))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BareObjectClassification {
    IamPolicyDocument(&'static IamPolicyDocumentProperty),
    FreeFormJson(&'static ClassifiedProperty),
    EmptyObject(&'static ClassifiedProperty),
    StringMap(&'static ClassifiedProperty),
    OmittedResource(&'static ClassifiedProperty),
    TypedOverride {
        expression: &'static str,
        display: &'static str,
    },
}

impl BareObjectClassification {
    fn owner_name(self) -> &'static str {
        match self {
            Self::IamPolicyDocument(entry)
                if IAM_POLICY_DOCUMENT_PROPERTIES
                    .iter()
                    .any(|candidate| std::ptr::eq(candidate, entry)) =>
            {
                "IAM_POLICY_DOCUMENT_PROPERTIES"
            }
            Self::IamPolicyDocument(_) => "NONSTANDARD_IAM_POLICY_DOCUMENT_PROPERTIES",
            Self::FreeFormJson(entry)
                if FREE_FORM_JSON_PROPERTIES
                    .iter()
                    .any(|candidate| std::ptr::eq(candidate, entry)) =>
            {
                "FREE_FORM_JSON_PROPERTIES"
            }
            Self::FreeFormJson(_) => "TEST_FREE_FORM_JSON_PROPERTIES",
            Self::EmptyObject(entry)
                if EMPTY_OBJECT_PROPERTIES
                    .iter()
                    .any(|candidate| std::ptr::eq(candidate, entry)) =>
            {
                "EMPTY_OBJECT_PROPERTIES"
            }
            Self::EmptyObject(_) => "TEST_EMPTY_OBJECT_PROPERTIES",
            Self::StringMap(_) => "STRING_MAP_PROPERTIES",
            Self::OmittedResource(_) => "OMITTED_RESOURCE_PROPERTIES",
            Self::TypedOverride { .. } => "resource_type_overrides AttributeType",
        }
    }
}

fn unique_bare_object_classification(
    cfn_type: &str,
    property_path: CfnPropertyPath<'_>,
    mut matches: Vec<BareObjectClassification>,
) -> Option<BareObjectClassification> {
    if matches.len() > 1 {
        let owners = matches
            .iter()
            .map(|classification| classification.owner_name())
            .collect::<Vec<_>>()
            .join(", ");
        panic!(
            "multiply classified bare object {cfn_type}.{property_path}: {owners}; each shape must be classified in exactly one place; see CODEGEN.md#classifying-bare-objects"
        );
    }
    matches.pop()
}

fn property_declares_object(prop: &CfnProperty) -> bool {
    match &prop.prop_type {
        Some(TypeValue::Single(value)) => value == "object",
        Some(TypeValue::Multiple(values)) => values.iter().any(|value| value == "object"),
        None => false,
    }
}

fn is_direct_bare_object(prop: &CfnProperty) -> bool {
    property_declares_object(prop)
        && prop.ref_path.is_none()
        && prop.one_of.is_empty()
        && prop.properties.as_ref().is_none_or(BTreeMap::is_empty)
        && prop
            .pattern_properties
            .as_ref()
            .is_none_or(BTreeMap::is_empty)
}

fn is_bare_object_definition(definition: &CfnDefinition) -> bool {
    definition.def_type.as_deref() == Some("object")
        && definition.one_of.is_empty()
        && definition.items.is_none()
        && definition.enum_values.is_none()
        && definition
            .properties
            .as_ref()
            .is_none_or(BTreeMap::is_empty)
        && definition
            .pattern_properties
            .as_ref()
            .is_none_or(BTreeMap::is_empty)
}

fn property_is_bare_object(prop: &CfnProperty, schema: &CfnSchema) -> bool {
    if let Some(ref_path) = &prop.ref_path {
        return resolve_ref(schema, ref_path).is_some_and(is_bare_object_definition);
    }
    is_direct_bare_object(prop)
}

fn resource_type_override(
    cfn_type: &str,
    property_path: CfnPropertyPath<'_>,
) -> Option<&'static TypeOverride> {
    resource_type_overrides().get_path(cfn_type, property_path)
}

fn classify_bare_object(
    cfn_type: &str,
    property_path: CfnPropertyPath<'_>,
) -> Option<BareObjectClassification> {
    let mut matches = Vec::new();
    for table in [
        IAM_POLICY_DOCUMENT_PROPERTIES,
        NONSTANDARD_IAM_POLICY_DOCUMENT_PROPERTIES,
    ] {
        if let Some(entry) = table.iter().find(|(entry_type, entry_path)| {
            *entry_type == cfn_type && *entry_path == property_path
        }) {
            matches.push(BareObjectClassification::IamPolicyDocument(entry));
        }
    }
    for table in [FREE_FORM_JSON_PROPERTIES, TEST_FREE_FORM_JSON_PROPERTIES] {
        if let Some(entry) = find_omitted_property(table, cfn_type, property_path) {
            matches.push(BareObjectClassification::FreeFormJson(entry));
        }
    }
    for table in [EMPTY_OBJECT_PROPERTIES, TEST_EMPTY_OBJECT_PROPERTIES] {
        if let Some(entry) = find_omitted_property(table, cfn_type, property_path) {
            matches.push(BareObjectClassification::EmptyObject(entry));
        }
    }
    if let Some(entry) = find_omitted_property(STRING_MAP_PROPERTIES, cfn_type, property_path) {
        matches.push(BareObjectClassification::StringMap(entry));
    }
    if let Some(entry) = find_omitted_property(OMITTED_RESOURCE_PROPERTIES, cfn_type, property_path)
    {
        matches.push(BareObjectClassification::OmittedResource(entry));
    }
    if let Some(TypeOverride::AttributeType {
        expression,
        display,
        ..
    }) = resource_type_override(cfn_type, property_path)
    {
        matches.push(BareObjectClassification::TypedOverride {
            expression,
            display,
        });
    }

    unique_bare_object_classification(cfn_type, property_path, matches)
}

fn classify_bare_object_or_panic(
    cfn_type: &str,
    property_path: CfnPropertyPath<'_>,
) -> BareObjectClassification {
    classify_bare_object(cfn_type, property_path).unwrap_or_else(|| {
        panic!(
            "unclassified bare object {cfn_type}.{property_path}; classify it in FREE_FORM_JSON_PROPERTIES, EMPTY_OBJECT_PROPERTIES, or STRING_MAP_PROPERTIES, or add a dedicated typed override; see CODEGEN.md#classifying-bare-objects"
        )
    })
}

fn bare_object_attribute_type(
    cfn_type: &str,
    property_path: CfnPropertyPath<'_>,
    prop_name: &str,
) -> String {
    match classify_bare_object_or_panic(cfn_type, property_path) {
        BareObjectClassification::IamPolicyDocument(_) => {
            "carina_aws_types::iam_policy_document()".to_string()
        }
        BareObjectClassification::EmptyObject(_) => format!(
            r#"AttributeType::struct_({}.to_string(), vec![])"#,
            rust_lit(&prop_name.to_pascal_case())
        ),
        BareObjectClassification::StringMap(_) => {
            "AttributeType::map(AttributeType::string())".to_string()
        }
        BareObjectClassification::TypedOverride { expression, .. } => expression.to_string(),
        BareObjectClassification::FreeFormJson(entry)
        | BareObjectClassification::OmittedResource(entry) => panic!(
            "omitted property {}.{} reached AttributeType generation: {}",
            entry.cfn_type, entry.property_path, entry.reason
        ),
    }
}

fn is_object_string_union(prop: &CfnProperty) -> bool {
    let Some(TypeValue::Multiple(types)) = &prop.prop_type else {
        return false;
    };

    types.iter().any(|value| value == "object") && types.iter().any(|value| value == "string")
}

fn iam_policy_document_property(
    cfn_type: &str,
    property_path: CfnPropertyPath<'_>,
) -> Option<&'static IamPolicyDocumentProperty> {
    IAM_POLICY_DOCUMENT_PROPERTIES
        .iter()
        .chain(NONSTANDARD_IAM_POLICY_DOCUMENT_PROPERTIES)
        .find(|(entry_type, entry_path)| *entry_type == cfn_type && *entry_path == property_path)
}

fn is_iam_policy_document_property(cfn_type: &str, property_path: CfnPropertyPath<'_>) -> bool {
    iam_policy_document_property(cfn_type, property_path).is_some()
}

/// Classify CloudFormation's free-form JSON union before `TypeValue::as_str()`
/// reduces it to its first member. Unknown unions must fail closed so a newly
/// cached resource cannot silently become `Map<String, String>`.
fn object_string_union_is_iam_policy_document(
    prop: &CfnProperty,
    cfn_type: &str,
    property_path: CfnPropertyPath<'_>,
) -> bool {
    if !is_object_string_union(prop) {
        return false;
    }

    match classify_bare_object(cfn_type, property_path) {
        Some(BareObjectClassification::IamPolicyDocument(_)) => return true,
        Some(
            BareObjectClassification::FreeFormJson(_)
            | BareObjectClassification::TypedOverride { .. },
        ) => return false,
        Some(classification) => panic!(
            "object|string property {cfn_type}.{property_path} has incompatible classification in {}; classify it as an IAM policy document or an omitted free-form JSON document; see CODEGEN.md#classifying-bare-objects",
            classification.owner_name()
        ),
        None => {}
    }

    panic!(
        "unclassified object|string property {cfn_type}.{property_path}; classify it as an IAM policy document in IAM_POLICY_DOCUMENT_PROPERTIES or NONSTANDARD_IAM_POLICY_DOCUMENT_PROPERTIES, or as an omitted document in FREE_FORM_JSON_PROPERTIES; see CODEGEN.md#classifying-bare-objects"
    );
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
    /// JSON object-map value schemas keyed by key-pattern.
    #[serde(default)]
    pattern_properties: Option<BTreeMap<String, CfnProperty>>,
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

fn derive_tags_property(schema: &CfnSchema) -> Result<Option<GeneratedTagsProperty>> {
    let Some(tagging) = schema.tagging.as_ref().filter(|tagging| tagging.taggable) else {
        return Ok(None);
    };
    let tag_property = tagging
        .tag_property
        .as_deref()
        .unwrap_or("/properties/Tags");
    let Some(provider_name) = tag_property.strip_prefix("/properties/") else {
        anyhow::bail!(
            "{} has invalid tagging.tagProperty {tag_property:?}; expected /properties/<name>",
            schema.type_name
        );
    };
    if provider_name.is_empty() || provider_name.contains('/') {
        anyhow::bail!(
            "{} has invalid tagging.tagProperty {tag_property:?}; expected one top-level property",
            schema.type_name
        );
    }
    let property = schema.properties.get(provider_name).ok_or_else(|| {
        anyhow::anyhow!(
            "{} is taggable but tag property {provider_name:?} does not exist",
            schema.type_name
        )
    })?;

    if !is_dsl_tags_property(schema, provider_name) {
        return Ok(None);
    }

    let wire = if is_key_value_tag_array(property, schema) {
        GeneratedTagsWire::KeyValueList
    } else if is_string_tag_map(property, schema) {
        GeneratedTagsWire::Map
    } else {
        anyhow::bail!(
            "{} tag property {provider_name:?} must be an array of Key/Value strings or an object map with string values",
            schema.type_name
        );
    };

    Ok(Some(GeneratedTagsProperty {
        provider_name: provider_name.to_string(),
        wire,
    }))
}

fn validate_mutually_exclusive_property_groups(
    schema: &CfnSchema,
    groups: &[&[&str]],
) -> Result<Vec<Vec<String>>> {
    let mut dsl_groups = Vec::with_capacity(groups.len());

    for (group_index, group) in groups.iter().enumerate() {
        if group.len() < 2 {
            anyhow::bail!(
                "{} mutually exclusive property group {} has {} member(s); expected at least 2",
                schema.type_name,
                group_index + 1,
                group.len()
            );
        }

        let mut dsl_group = Vec::with_capacity(group.len());
        for &property_name in *group {
            let property = schema.properties.get(property_name).ok_or_else(|| {
                anyhow::anyhow!(
                    "{} mutually exclusive property group {} member {property_name:?} is not a top-level schema property",
                    schema.type_name,
                    group_index + 1
                )
            })?;
            let property_path = [property_name.to_string()];
            if is_property_read_only(schema, property, &property_path) {
                anyhow::bail!(
                    "{} mutually exclusive property group {} member {property_name:?} is read-only and not updatable",
                    schema.type_name,
                    group_index + 1
                );
            }
            let is_create_only = schema
                .create_only_properties
                .iter()
                .filter_map(|pointer| cfn_property_path(pointer))
                .any(|path| path.starts_with(&property_path));
            if is_create_only {
                anyhow::bail!(
                    "{} mutually exclusive property group {} member {property_name:?} is create-only and not updatable",
                    schema.type_name,
                    group_index + 1
                );
            }
            dsl_group.push(property_name.to_snake_case());
        }
        dsl_groups.push(dsl_group);
    }

    Ok(dsl_groups)
}

fn derive_mutually_exclusive_property_groups(schema: &CfnSchema) -> Result<Vec<Vec<String>>> {
    let groups = mutually_exclusive_property_groups()
        .get(schema.type_name.as_str())
        .copied()
        .unwrap_or(&[]);
    validate_mutually_exclusive_property_groups(schema, groups)
}

fn emit_exclusive_groups_slice(groups: &[Vec<String>]) -> String {
    if groups.is_empty() {
        return "&[]".to_string();
    }

    let entries = groups
        .iter()
        .map(|group| {
            let attributes = group
                .iter()
                .map(|attribute| rust_lit(attribute).to_string())
                .collect::<Vec<_>>()
                .join(", ");
            format!("ExclusiveGroup {{ attributes: &[{attributes}] }}")
        })
        .collect::<Vec<_>>()
        .join(", ");
    format!("&[{entries}]")
}

fn is_key_value_tag_array(property: &CfnProperty, schema: &CfnSchema) -> bool {
    if property.prop_type.as_ref().and_then(TypeValue::as_str) != Some("array") {
        return false;
    }
    let Some(items) = property.items.as_deref() else {
        return false;
    };
    let properties = items.properties.as_ref().or_else(|| {
        items
            .ref_path
            .as_deref()
            .and_then(|path| path.strip_prefix("#/definitions/"))
            .and_then(|name| schema.definitions.as_ref()?.get(name))
            .and_then(|definition| definition.properties.as_ref())
    });
    let Some(properties) = properties else {
        return false;
    };
    ["Key", "Value"].into_iter().all(|name| {
        properties
            .get(name)
            .is_some_and(|property| is_string_property(schema, property))
    })
}

fn is_string_tag_map(property: &CfnProperty, schema: &CfnSchema) -> bool {
    if property.prop_type.as_ref().and_then(TypeValue::as_str) != Some("object")
        || property.additional_properties != Some(false)
    {
        return false;
    }
    property
        .pattern_properties
        .as_ref()
        .is_some_and(|patterns| {
            !patterns.is_empty()
                && patterns
                    .values()
                    .all(|value| is_string_property(schema, value))
        })
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
struct CfnDefinition {
    #[serde(rename = "type")]
    def_type: Option<String>,
    properties: Option<BTreeMap<String, CfnProperty>>,
    /// JSON object-map value schemas keyed by key-pattern.
    #[serde(default)]
    pattern_properties: Option<BTreeMap<String, CfnProperty>>,
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

fn load_markdown_example(examples_dir: Option<&Path>, type_name: &str) -> Result<Option<String>> {
    let Some(examples_dir) = examples_dir else {
        return Ok(None);
    };
    if !examples_dir.is_dir() {
        anyhow::bail!(
            "--examples-dir must be an existing directory: {}",
            examples_dir.display()
        );
    }
    let path = examples_dir
        .join(full_resource_name_from_type(type_name)?)
        .join("main.crn");
    match std::fs::read_to_string(&path) {
        Ok(source) => Ok(Some(preprocess_markdown_example(&source))),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error).with_context(|| format!("Failed to read {}", path.display())),
    }
}

fn preprocess_markdown_example(source: &str) -> String {
    let mut filtered = Vec::new();
    let mut in_provider_block = false;
    for line in source.split_inclusive('\n') {
        if !in_provider_block && line.starts_with("provider ") {
            in_provider_block = true;
            continue;
        }
        if in_provider_block {
            if line.starts_with('}') {
                in_provider_block = false;
            }
            continue;
        }
        if line.starts_with('#') {
            continue;
        }
        filtered.push(line);
    }

    let first_content = filtered
        .iter()
        .position(|line| !line.strip_suffix('\n').unwrap_or(line).is_empty())
        .unwrap_or(filtered.len());
    filtered[first_content..].concat()
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

fn generate_arn_helper_module(type_name: &str) -> Result<String> {
    let dsl_resource = dsl_resource_name_from_type(type_name)?;
    let (service, resource) = resource_identity_parts(&dsl_resource)
        .context("DSL resource name must contain a service and resource")?;
    let display_name = type_name
        .strip_prefix("AWS::")
        .unwrap_or(type_name)
        .replace("::", " ");
    let helper = emit_arn_helper(service, resource)?;
    Ok(format!(
        "//! Auto-generated helper schema module for AWSCC {display_name} identifiers\n\
         //!\n\
         //! DO NOT EDIT MANUALLY - regenerate with carina-codegen\n\n\
         use carina_core::schema::AttributeType;\n\n\
         {helper}"
    ))
}

fn is_string_property(schema: &CfnSchema, property: &CfnProperty) -> bool {
    if matches!(
        property.prop_type.as_ref().and_then(TypeValue::as_str),
        Some("string")
    ) {
        return true;
    }

    property
        .ref_path
        .as_deref()
        .and_then(|ref_path| resolve_ref(schema, ref_path))
        .is_some_and(|definition| definition.def_type.as_deref() == Some("string"))
}

fn declared_replacement_class(
    resource_type_name: &str,
    schema: &CfnSchema,
) -> Result<ReplacementClass> {
    let class = REPLACEMENT_CLASSES
        .iter()
        .find_map(|(name, class)| (*name == resource_type_name).then_some(*class))
        .ok_or_else(|| {
            anyhow::anyhow!(
                "replacement class is not declared for {resource_type_name}; add it to REPLACEMENT_CLASSES"
            )
        })?;

    match class {
        ReplacementClass::Attribute(attribute) => {
            let (provider_name, property) = schema
                .properties
                .iter()
                .find(|(provider_name, _)| provider_name.to_snake_case() == attribute)
                .ok_or_else(|| {
                    anyhow::anyhow!(
                        "replacement class for {resource_type_name} declares attribute {attribute:?}, but it does not match a top-level property"
                    )
                })?;
            let property_path = format!("/properties/{provider_name}");
            if schema
                .read_only_properties
                .iter()
                .any(|read_only| read_only == &property_path)
            {
                anyhow::bail!(
                    "replacement class for {resource_type_name} declares attribute {attribute:?}, but {property_path} is read-only"
                );
            }
            if !is_string_property(schema, property) {
                anyhow::bail!(
                    "replacement class for {resource_type_name} declares attribute {attribute:?}, but {property_path} must be a string"
                );
            }
        }
        ReplacementClass::Coexisting | ReplacementClass::Conflicting => {}
    }

    Ok(class)
}

fn emit_replacement_class_builder(code: &mut String, class: ReplacementClass) {
    match class {
        ReplacementClass::Attribute(attribute) => {
            code.push_str(&format!(
                "        .with_unique_name_attribute({})\n",
                rust_lit(attribute)
            ));
        }
        ReplacementClass::Coexisting => {
            code.push_str("        .with_coexisting_replacement()\n");
        }
        ReplacementClass::Conflicting => {
            code.push_str("        .with_conflicting_replacement()\n");
        }
    }
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

    if args.print_arn_helper_module {
        print!("{}", generate_arn_helper_module(&args.type_name)?);
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

    if !schema.is_cloud_control_provisionable() {
        eprintln!(
            "Skipping {}: the schema has no Cloud Control handlers and no \
             HANDLERLESS_CLOUD_CONTROL_TYPES allowlist entry.\n\
             If DescribeType reports a provisionable type and a Cloud Control operation succeeds, \
             follow CODEGEN.md to add a sourced handlerless capability record.\n\
             Use carina-provider-aws if you need to manage this resource via the AWS SDK directly.",
            args.type_name
        );
        std::process::exit(EXIT_SKIPPED);
    }

    // Generate output based on format
    let output = match args.format.as_str() {
        "markdown" | "md" => {
            let example = load_markdown_example(args.examples_dir.as_deref(), &args.type_name)?;
            generate_markdown_with_example(&schema, &args.type_name, example.as_deref())?
        }
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
    /// First CloudFormation property path through which this definition was reached.
    first_property_path: Vec<String>,
    /// Every distinct property path through which this definition is reached.
    property_paths: BTreeSet<Vec<String>>,
    /// Properties of the definition
    properties: BTreeMap<String, CfnProperty>,
    /// Required fields
    required: Vec<String>,
}

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
struct MarkdownEnumKey {
    namespace: String,
    type_name: String,
}

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
enum MarkdownStructKey {
    /// A named `$ref` definition. Reaching the same definition through
    /// multiple property paths must still produce exactly one target.
    Definition(String),
    /// An inline object. Its writable DSL field path is its identity because
    /// the same property name can describe unrelated objects under different
    /// parents.
    Inline(Vec<String>),
}

impl MarkdownStructKey {
    fn stable_identity(&self) -> String {
        match self {
            Self::Definition(name) => format!("definition {name}"),
            Self::Inline(property_path) => {
                format!("inline {}", markdown_heading_path(property_path).join("."))
            }
        }
    }
}

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
struct MarkdownStructFieldKey {
    struct_key: MarkdownStructKey,
    field_name: String,
}

#[derive(Default)]
struct MarkdownEnumTargets {
    properties: BTreeMap<String, MarkdownEnumKey>,
    struct_fields: BTreeMap<MarkdownStructFieldKey, MarkdownEnumKey>,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
enum MarkdownTarget {
    Enum(MarkdownEnumKey),
    Struct(MarkdownStructKey),
}

#[derive(Debug)]
enum MarkdownType {
    Plain(String),
    Linked(MarkdownLinkedType),
}

#[derive(Debug)]
enum MarkdownLinkedType {
    Enum(MarkdownEnumKey),
    Struct {
        key: MarkdownStructKey,
        name: String,
    },
    List {
        item: Box<MarkdownLinkedType>,
        suffix: Option<String>,
    },
    Map {
        value: Box<MarkdownLinkedType>,
    },
}

impl MarkdownType {
    fn write_to(&self, page: &mut PageBuilder<MarkdownTarget>) {
        match self {
            Self::Plain(value) => page.push_str(value),
            Self::Linked(value) => value.write_to(page),
        }
    }

    fn table_cell(&self) -> TableCell<MarkdownTarget> {
        let mut cell = TableCell::new();
        self.write_to_table_cell(&mut cell);
        cell
    }

    fn write_to_table_cell(&self, cell: &mut TableCell<MarkdownTarget>) {
        match self {
            Self::Plain(value) => cell.push_text(value),
            Self::Linked(value) => value.write_to_table_cell(cell),
        }
    }
}

impl MarkdownLinkedType {
    fn write_to(&self, page: &mut PageBuilder<MarkdownTarget>) {
        match self {
            Self::Enum(key) => page.link(
                MarkdownTarget::Enum(key.clone()),
                format!("Enum ({})", key.type_name),
            ),
            Self::Struct { key, name } => page.link(
                MarkdownTarget::Struct(key.clone()),
                format!("Struct({name})"),
            ),
            Self::List { item, suffix } => {
                page.push_str("List\\<");
                item.write_to(page);
                page.push_str("\\>");
                if let Some(suffix) = suffix {
                    page.push_str(suffix);
                }
            }
            Self::Map { value } => {
                page.push_str("Map\\<String, ");
                value.write_to(page);
                page.push_str("\\>");
            }
        }
    }

    fn write_to_table_cell(&self, cell: &mut TableCell<MarkdownTarget>) {
        match self {
            Self::Enum(key) => cell.push_link(
                MarkdownTarget::Enum(key.clone()),
                format!("Enum ({})", key.type_name),
            ),
            Self::Struct { key, name } => cell.push_link(
                MarkdownTarget::Struct(key.clone()),
                format!("Struct({name})"),
            ),
            Self::List { item, suffix } => {
                cell.push_text("List\\<");
                item.write_to_table_cell(cell);
                cell.push_text("\\>");
                if let Some(suffix) = suffix {
                    cell.push_text(suffix);
                }
            }
            Self::Map { value } => {
                cell.push_text("Map\\<String, ");
                value.write_to_table_cell(cell);
                cell.push_text("\\>");
            }
        }
    }
}

#[derive(Debug)]
struct MarkdownEnum {
    prop_name: String,
    heading_path: Vec<String>,
    info: EnumInfo,
}

fn decode_json_pointer_segment(segment: &str) -> String {
    segment.replace("~1", "/").replace("~0", "~")
}

fn cfn_property_path(pointer: &str) -> Option<Vec<String>> {
    let path = pointer.strip_prefix("/properties/")?;
    if path.is_empty() {
        return None;
    }
    Some(path.split('/').map(decode_json_pointer_segment).collect())
}

fn extend_property_path(property_path: &[String], segment: &str) -> Vec<String> {
    let mut extended = property_path.to_vec();
    extended.push(segment.to_string());
    extended
}

fn is_read_only_property_path(schema: &CfnSchema, property_path: &[String]) -> bool {
    schema
        .read_only_properties
        .iter()
        .filter_map(|pointer| cfn_property_path(pointer))
        .any(|path| path == property_path)
}

fn is_write_only_property_path(schema: &CfnSchema, property_path: &[String]) -> bool {
    schema
        .write_only_properties
        .iter()
        .filter_map(|pointer| cfn_property_path(pointer))
        .any(|path| path == property_path)
}

fn has_nested_read_only_property_path(schema: &CfnSchema, property_path: &[String]) -> bool {
    schema
        .read_only_properties
        .iter()
        .filter_map(|pointer| cfn_property_path(pointer))
        .any(|path| path.len() > property_path.len() && path.starts_with(property_path))
}

fn struct_fields_are_fully_read_only(
    schema: &CfnSchema,
    properties: &BTreeMap<String, CfnProperty>,
    property_path: &[String],
    visiting_defs: &mut HashSet<String>,
) -> bool {
    let mut fields = properties
        .iter()
        .filter(|(_, property)| !is_deprecated_property(property))
        .peekable();
    fields.peek().is_some()
        && fields.all(|(field_name, field)| {
            let field_path = extend_property_path(property_path, field_name);
            property_is_fully_read_only(schema, field, &field_path, visiting_defs)
        })
}

fn property_is_fully_read_only(
    schema: &CfnSchema,
    property: &CfnProperty,
    property_path: &[String],
    visiting_defs: &mut HashSet<String>,
) -> bool {
    if is_read_only_property_path(schema, property_path) {
        return true;
    }
    if !has_nested_read_only_property_path(schema, property_path) {
        return false;
    }

    if let Some(ref_path) = &property.ref_path {
        if !visiting_defs.insert(ref_path.clone()) {
            return false;
        }
        let fully_read_only = match resolve_ref_classified(schema, ref_path) {
            Some(ResolvedDef::Struct { properties, .. }) => {
                struct_fields_are_fully_read_only(schema, properties, property_path, visiting_defs)
            }
            Some(ResolvedDef::OneOf { variants }) => {
                let merged = variants
                    .iter()
                    .filter_map(|variant| variant.properties.as_ref())
                    .flat_map(|properties| properties.iter())
                    .map(|(name, property)| (name.clone(), property.clone()))
                    .collect::<BTreeMap<_, _>>();
                struct_fields_are_fully_read_only(schema, &merged, property_path, visiting_defs)
            }
            Some(ResolvedDef::Array { items }) => {
                let item_path = extend_property_path(property_path, "*");
                property_is_fully_read_only(schema, items, &item_path, visiting_defs)
            }
            _ => false,
        };
        visiting_defs.remove(ref_path);
        return fully_read_only;
    }

    match property.prop_type.as_ref().and_then(TypeValue::as_str) {
        Some("object") => property.properties.as_ref().is_some_and(|properties| {
            struct_fields_are_fully_read_only(schema, properties, property_path, visiting_defs)
        }),
        Some("array") => property.items.as_deref().is_some_and(|items| {
            let item_path = extend_property_path(property_path, "*");
            property_is_fully_read_only(schema, items, &item_path, visiting_defs)
        }),
        _ => false,
    }
}

fn is_property_read_only(
    schema: &CfnSchema,
    property: &CfnProperty,
    property_path: &[String],
) -> bool {
    property_is_fully_read_only(schema, property, property_path, &mut HashSet::new())
}

fn record_emitted_read_only_path(property_path: &[String]) {
    EMITTED_READ_ONLY_PATHS.with(|paths| paths.borrow_mut().push(property_path.to_vec()));
}

fn read_only_emission_mark() -> usize {
    EMITTED_READ_ONLY_PATHS.with(|paths| paths.borrow().len())
}

fn record_struct_read_only_use(struct_name: &str, property_path: &[String], emission_mark: usize) {
    let signature = EMITTED_READ_ONLY_PATHS.with(|paths| {
        paths.borrow()[emission_mark..]
            .iter()
            .filter_map(|path| {
                path.strip_prefix(property_path)
                    .filter(|relative| !relative.is_empty())
                    .map(<[String]>::to_vec)
            })
            .collect::<BTreeSet<_>>()
    });
    EMITTED_STRUCT_READ_ONLY_USES.with(|uses| {
        uses.borrow_mut()
            .entry(struct_name.to_string())
            .or_default()
            .push(StructReadOnlyUse {
                property_path: property_path.to_vec(),
                signature,
            });
    });
}

fn record_emitted_write_only_path(property_path: &[String]) {
    EMITTED_WRITE_ONLY_PATHS.with(|paths| paths.borrow_mut().push(property_path.to_vec()));
}

fn record_skipped_deprecated_write_only_path(schema: &CfnSchema, property_path: &[String]) {
    if is_write_only_property_path(schema, property_path) {
        SKIPPED_DEPRECATED_WRITE_ONLY_PATHS.with(|paths| {
            paths.borrow_mut().insert(property_path.to_vec());
        });
    }
}

fn write_only_emission_mark() -> usize {
    EMITTED_WRITE_ONLY_PATHS.with(|paths| paths.borrow().len())
}

fn record_struct_write_only_use(struct_name: &str, property_path: &[String], emission_mark: usize) {
    let signature = EMITTED_WRITE_ONLY_PATHS.with(|paths| {
        paths.borrow()[emission_mark..]
            .iter()
            .filter_map(|path| {
                path.strip_prefix(property_path)
                    .filter(|relative| !relative.is_empty())
                    .map(<[String]>::to_vec)
            })
            .collect::<BTreeSet<_>>()
    });
    EMITTED_STRUCT_WRITE_ONLY_USES.with(|uses| {
        uses.borrow_mut()
            .entry(struct_name.to_string())
            .or_default()
            .push(StructWriteOnlyUse {
                property_path: property_path.to_vec(),
                signature,
            });
    });
}

fn format_cfn_property_path(path: &[String]) -> String {
    let encoded = path
        .iter()
        .map(|segment| segment.replace('~', "~0").replace('/', "~1"))
        .collect::<Vec<_>>()
        .join("/");
    format!("/properties/{encoded}")
}

fn validate_emitted_nested_read_only_properties(schema: &CfnSchema) -> Result<()> {
    let emitted = EMITTED_READ_ONLY_PATHS.with(|paths| {
        paths
            .borrow()
            .iter()
            .cloned()
            .collect::<BTreeSet<Vec<String>>>()
    });
    for pointer in &schema.read_only_properties {
        let Some(path) = cfn_property_path(pointer) else {
            continue;
        };
        if path.len() > 1 && !emitted.contains(&path) {
            anyhow::bail!(
                "{} nested readOnlyProperties pointer {pointer:?} did not resolve to an emitted StructField",
                schema.type_name
            );
        }
    }

    EMITTED_STRUCT_READ_ONLY_USES.with(|all_uses| -> Result<()> {
        for (struct_name, uses) in all_uses.borrow().iter() {
            let Some(first) = uses.first() else {
                continue;
            };
            if let Some(differing) = uses
                .iter()
                .skip(1)
                .find(|use_| use_.signature != first.signature)
            {
                anyhow::bail!(
                    "{} struct {struct_name:?} is emitted from {} and {} with different nested readOnlyProperties sets; name-based caching cannot represent path-dependent provider-populated fields",
                    schema.type_name,
                    format_cfn_property_path(&first.property_path),
                    format_cfn_property_path(&differing.property_path),
                );
            }
        }
        Ok(())
    })
}

fn validate_emitted_nested_write_only_properties(schema: &CfnSchema) -> Result<()> {
    let read_only = schema
        .read_only_properties
        .iter()
        .filter_map(|pointer| cfn_property_path(pointer).map(|path| (path, pointer)))
        .collect::<BTreeMap<_, _>>();
    for pointer in &schema.write_only_properties {
        let Some(path) = cfn_property_path(pointer) else {
            continue;
        };
        if let Some(read_only_pointer) = read_only.get(&path) {
            anyhow::bail!(
                "{} property pointer {pointer:?} is both readOnlyProperties and writeOnlyProperties (read-only pointer {read_only_pointer:?})",
                schema.type_name
            );
        }
    }

    let emitted = EMITTED_WRITE_ONLY_PATHS.with(|paths| {
        paths
            .borrow()
            .iter()
            .cloned()
            .collect::<BTreeSet<Vec<String>>>()
    });
    let skipped_deprecated =
        SKIPPED_DEPRECATED_WRITE_ONLY_PATHS.with(|paths| paths.borrow().clone());
    for pointer in &schema.write_only_properties {
        let Some(path) = cfn_property_path(pointer) else {
            continue;
        };
        if path.len() > 1 && !emitted.contains(&path) && !skipped_deprecated.contains(&path) {
            anyhow::bail!(
                "{} nested writeOnlyProperties pointer {pointer:?} did not resolve to an emitted StructField",
                schema.type_name
            );
        }
    }

    EMITTED_STRUCT_WRITE_ONLY_USES.with(|all_uses| -> Result<()> {
        for (struct_name, uses) in all_uses.borrow().iter() {
            let Some(first) = uses.first() else {
                continue;
            };
            if let Some(differing) = uses
                .iter()
                .skip(1)
                .find(|use_| use_.signature != first.signature)
            {
                anyhow::bail!(
                    "{} struct {struct_name:?} is emitted from {} and {} with different nested writeOnlyProperties sets; name-based caching cannot represent path-dependent provider-omitted fields",
                    schema.type_name,
                    format_cfn_property_path(&first.property_path),
                    format_cfn_property_path(&differing.property_path),
                );
            }
        }
        Ok(())
    })
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

const SCHEDULER_GROUP_NAME_LIMITATION: &str = "Carina requires group_name to be omitted or set to \
    the literal 'default'. Cloud Control identifies schedules by Name only and reads them in the \
    default group, so a schedule in another group can be created but never read or tracked.";

fn generated_property_description(
    schema: &CfnSchema,
    prop_name: &str,
    prop: &CfnProperty,
) -> Option<String> {
    let mut description = prop.description.clone();
    if schema.type_name == "AWS::Scheduler::Schedule" && prop_name == "GroupName" {
        let description = description.get_or_insert_with(String::new);
        if !description.is_empty() {
            description.push(' ');
        }
        description.push_str(SCHEDULER_GROUP_NAME_LIMITATION);
    }

    description
}

fn documented_top_level_properties(
    schema: &CfnSchema,
) -> impl Iterator<Item = (&String, &CfnProperty)> {
    schema.properties.iter().filter(|(name, property)| {
        !is_deprecated_property(property)
            && !should_omit_top_level_property(&schema.type_name, name)
    })
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
        resource_type_override(resource_type, CfnPropertyPath::new(None, prop_name))
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
        "carina_aws_types::route53_hosted_zone_id()" => "Route53HostedZoneId",
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
        "crate::schemas::catalog::scheduler_schedule_group_name()" => "SchedulerScheduleGroupName",
        "AttributeType::string()" => "String",
        _ => "String",
    }
}

/// Determine the display string for a property's type in markdown docs
#[cfg(test)]
fn type_display_string(prop_name: &str, prop: &CfnProperty, schema: &CfnSchema) -> String {
    type_display_string_at(
        prop_name,
        prop,
        schema,
        CfnPropertyPath::new(None, prop_name),
    )
}

fn object_map_type_display(
    prop_name: &str,
    schema: &CfnSchema,
    property_path: CfnPropertyPath<'_>,
    value: &CfnProperty,
) -> String {
    match object_map_value_or_panic(schema, property_path, value) {
        ObjectMapValueKind::DirectString => "`Map<String, String>`".to_string(),
        ObjectMapValueKind::Struct => {
            let struct_name = value
                .ref_path
                .as_deref()
                .and_then(ref_def_name)
                .unwrap_or(prop_name);
            format!("`Map<String, Struct({struct_name})>`")
        }
    }
}

fn type_display_string_at(
    prop_name: &str,
    prop: &CfnProperty,
    schema: &CfnSchema,
    cfn_property_path: CfnPropertyPath<'_>,
) -> String {
    if let Some(TypeOverride::AttributeType { display, .. }) =
        resource_type_override(&schema.type_name, cfn_property_path)
    {
        (*display).to_string()
    } else if object_string_union_is_iam_policy_document(prop, &schema.type_name, cfn_property_path)
    {
        "PolicyDocument".to_string()
    } else if prop_name == "Tags" {
        "`Map<String, String>`".to_string()
    } else if property_is_bare_object(prop, schema) {
        match classify_bare_object_or_panic(&schema.type_name, cfn_property_path) {
            BareObjectClassification::IamPolicyDocument(_) => "PolicyDocument".to_string(),
            BareObjectClassification::EmptyObject(_) => "Empty struct (`{}`)".to_string(),
            BareObjectClassification::StringMap(_) => "`Map<String, String>`".to_string(),
            BareObjectClassification::TypedOverride { display, .. } => display.to_string(),
            BareObjectClassification::FreeFormJson(entry)
            | BareObjectClassification::OmittedResource(entry) => panic!(
                "omitted property {}.{} reached Markdown type generation: {}",
                entry.cfn_type, entry.property_path, entry.reason
            ),
        }
    } else if !prop.one_of.is_empty() {
        if let Some(array_variant) = prop
            .one_of
            .iter()
            .find(|variant| property_resolves_to_array(variant, schema))
        {
            return type_display_string_at(prop_name, array_variant, schema, cfn_property_path);
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
        if is_tag_map_ref(schema, ref_path) {
            "`Map<String, String>`".to_string()
        } else {
            // Scalar refs (awscc#291: WAFv2 `Priority`/`Limit`) recurse for
            // their Int/Float/Bool display with the definition's range. A
            // struct-shaped ref gets a plain label here; the typed Markdown
            // path is solely responsible for turning it into a link.
            match resolve_ref_classified(schema, ref_path) {
                Some(ResolvedDef::Struct { .. } | ResolvedDef::OneOf { .. }) => {
                    let def_name = ref_def_name(ref_path).unwrap_or(prop_name);
                    format!("Struct({def_name})")
                }
                Some(ResolvedDef::ObjectMap { value }) => {
                    object_map_type_display(prop_name, schema, cfn_property_path, value)
                }
                Some(scalar @ ResolvedDef::Scalar { .. }) => {
                    let scalar_prop = scalar
                        .scalar_as_property()
                        .expect("Scalar variant always rebuilds a scalar property");
                    type_display_string_at(prop_name, &scalar_prop, schema, cfn_property_path)
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
                if is_iam_policy_document_property(&schema.type_name, cfn_property_path) {
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
                let res_override = resource_type_override(&schema.type_name, cfn_property_path);
                if let Some(TypeOverride::IntEnum(values)) = res_override {
                    let observed_values = prop
                        .enum_values
                        .as_ref()
                        .and_then(|enum_values| {
                            enum_values
                                .iter()
                                .map(|value| match value {
                                    EnumValue::Int(value) => Some(*value),
                                    _ => None,
                                })
                                .collect::<Option<Vec<_>>>()
                        })
                        .filter(|values| !values.is_empty());
                    let _ = int_enum_type(
                        &schema.type_name,
                        prop_name,
                        observed_values.as_deref().unwrap_or(values),
                    );
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
                let res_override = resource_type_override(&schema.type_name, cfn_property_path);
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
                        if !is_tag_map_ref(schema, ref_path) {
                            // Keep this renderer fragment-free. Typed enum and
                            // struct links are emitted only by MarkdownType;
                            // scalar items (awscc#291) still mirror the
                            // schema-code array branch here.
                            let item_def = resolve_ref_classified(schema, ref_path);
                            if let Some(ResolvedDef::Struct { .. } | ResolvedDef::OneOf { .. }) =
                                item_def
                                && let Some(def_name) = ref_def_name(ref_path)
                            {
                                format!("`List<{def_name}>`")
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
                    format!("Struct({prop_name})")
                } else if let Some(value) = direct_object_map_value(prop) {
                    object_map_type_display(prop_name, schema, cfn_property_path, value)
                } else {
                    panic!(
                        "unsupported object property {}.{}; expected a struct, direct-string map, tag map, typed map of struct, or classified bare object; see CODEGEN.md#classifying-bare-objects",
                        schema.type_name, cfn_property_path
                    );
                }
            }
            _ => "String".to_string(),
        }
    }
}

fn markdown_property_is_array(prop: &CfnProperty, schema: &CfnSchema) -> bool {
    property_resolves_to_array(prop, schema)
        || prop
            .one_of
            .iter()
            .any(|variant| markdown_property_is_array(variant, schema))
}

struct MarkdownStructDef {
    key: MarkdownStructKey,
    name: String,
    properties: BTreeMap<String, CfnProperty>,
    required: Vec<String>,
}

/// Classify exactly the property shapes that own a Markdown struct target.
/// Both target collection and type rendering use this function, so rendering
/// cannot create a struct link unless the same property shape is collectable.
fn markdown_struct_def(
    prop_name: &str,
    prop: &CfnProperty,
    schema: &CfnSchema,
    property_path: &[String],
) -> Option<MarkdownStructDef> {
    // Preserve the pre-PageBuilder precedence: a direct non-array oneOf is a
    // String display, not a struct target. Array variants are selected by the
    // callers before classification.
    if !prop.one_of.is_empty() {
        return None;
    }

    if let Some(ref_path) = &prop.ref_path
        && !is_tag_map_ref(schema, ref_path)
        && let Some(def_name) = ref_def_name(ref_path)
    {
        match resolve_ref_classified(schema, ref_path) {
            Some(ResolvedDef::Struct {
                properties,
                required,
            }) => {
                return Some(MarkdownStructDef {
                    key: MarkdownStructKey::Definition(def_name.to_string()),
                    name: def_name.to_string(),
                    properties: properties.clone(),
                    required: required.to_vec(),
                });
            }
            Some(ResolvedDef::OneOf { variants }) => {
                let properties = variants
                    .iter()
                    .filter_map(|variant| variant.properties.as_ref())
                    .flat_map(|properties| properties.iter())
                    .map(|(name, property)| (name.clone(), property.clone()))
                    .collect::<BTreeMap<_, _>>();
                if !properties.is_empty() {
                    return Some(MarkdownStructDef {
                        key: MarkdownStructKey::Definition(def_name.to_string()),
                        name: def_name.to_string(),
                        properties,
                        required: Vec::new(),
                    });
                }
            }
            Some(
                ResolvedDef::ObjectMap { .. }
                | ResolvedDef::Enum { .. }
                | ResolvedDef::Array { .. }
                | ResolvedDef::StringPattern { .. }
                | ResolvedDef::Scalar { .. }
                | ResolvedDef::Opaque,
            )
            | None => {}
        }
    }

    if prop.prop_type.as_ref().and_then(TypeValue::as_str) == Some("object")
        && let Some(properties) = &prop.properties
        && !properties.is_empty()
    {
        return Some(MarkdownStructDef {
            key: MarkdownStructKey::Inline(property_path.to_vec()),
            name: prop_name.to_string(),
            properties: properties.clone(),
            required: prop.required.clone(),
        });
    }

    None
}

fn markdown_array_items<'a>(
    prop: &'a CfnProperty,
    schema: &'a CfnSchema,
) -> Option<&'a CfnProperty> {
    if prop.prop_type.as_ref().and_then(TypeValue::as_str) == Some("array") {
        return prop.items.as_deref();
    }
    prop.ref_path
        .as_deref()
        .and_then(|ref_path| resolve_ref_classified(schema, ref_path))
        .and_then(|resolved| match resolved {
            ResolvedDef::Array { items } => Some(items),
            ResolvedDef::Struct { .. }
            | ResolvedDef::ObjectMap { .. }
            | ResolvedDef::OneOf { .. }
            | ResolvedDef::Enum { .. }
            | ResolvedDef::StringPattern { .. }
            | ResolvedDef::Scalar { .. }
            | ResolvedDef::Opaque => None,
        })
}

fn markdown_list_type(item: MarkdownLinkedType, prop: &CfnProperty) -> MarkdownLinkedType {
    let suffix = if prop.min_items.is_some() || prop.max_items.is_some() {
        Some(format!(
            " (items: {})",
            range_display_string(prop.min_items, prop.max_items)
        ))
    } else {
        None
    };
    MarkdownLinkedType::List {
        item: Box::new(item),
        suffix,
    }
}

fn markdown_link_type(
    prop_name: &str,
    prop: &CfnProperty,
    schema: &CfnSchema,
    enum_target: Option<&MarkdownEnumKey>,
    property_path: &[String],
    cfn_parent: Option<&str>,
) -> Option<MarkdownLinkedType> {
    if let Some(key) = enum_target {
        let enum_type = MarkdownLinkedType::Enum(key.clone());
        return Some(if markdown_property_is_array(prop, schema) {
            markdown_list_type(enum_type, prop)
        } else {
            enum_type
        });
    }

    if !prop.one_of.is_empty() {
        return prop
            .one_of
            .iter()
            .find(|variant| markdown_property_is_array(variant, schema))
            .and_then(|variant| {
                markdown_link_type(prop_name, variant, schema, None, property_path, cfn_parent)
            });
    }

    if let Some(value) = property_object_map_value(prop, schema) {
        object_map_value_or_panic(schema, CfnPropertyPath::new(cfn_parent, prop_name), value);
        let value_path = extend_property_path(property_path, "*");
        if let Some(value_type) =
            markdown_link_type(prop_name, value, schema, None, &value_path, cfn_parent)
        {
            return Some(MarkdownLinkedType::Map {
                value: Box::new(value_type),
            });
        }
    }

    if let Some(struct_def) = markdown_struct_def(prop_name, prop, schema, property_path) {
        return Some(MarkdownLinkedType::Struct {
            key: struct_def.key,
            name: struct_def.name,
        });
    }

    if let Some(items) = markdown_array_items(prop, schema) {
        let item_path = extend_property_path(property_path, "*");
        let item_type = markdown_link_type(prop_name, items, schema, None, &item_path, cfn_parent)?;
        return Some(markdown_list_type(item_type, prop));
    }

    None
}

fn markdown_type(
    prop_name: &str,
    prop: &CfnProperty,
    schema: &CfnSchema,
    enum_target: Option<&MarkdownEnumKey>,
    property_path: &[String],
    cfn_parent: Option<&str>,
) -> MarkdownType {
    markdown_link_type(
        prop_name,
        prop,
        schema,
        enum_target,
        property_path,
        cfn_parent,
    )
    .map(MarkdownType::Linked)
    .unwrap_or_else(|| {
        MarkdownType::Plain(type_display_string_at(
            prop_name,
            prop,
            schema,
            CfnPropertyPath::new(cfn_parent, prop_name),
        ))
    })
}

fn markdown_heading_path(property_path: &[String]) -> Vec<String> {
    property_path
        .iter()
        .filter(|segment| segment.as_str() != "*")
        .map(|segment| segment.to_snake_case())
        .collect()
}

fn classified_property_doc_name(
    entry: &ClassifiedProperty,
    struct_defs: &BTreeMap<MarkdownStructKey, StructDefInfo>,
) -> (String, bool) {
    let Some(parent) = entry.property_path.parent else {
        return (entry.property_path.property.to_snake_case(), false);
    };
    if let Some(def_info) = struct_defs.values().find(|def_info| {
        def_info.def_name == parent
            && def_info
                .properties
                .contains_key(entry.property_path.property)
    }) {
        let field_path =
            extend_property_path(&def_info.first_property_path, entry.property_path.property);
        return (markdown_heading_path(&field_path).join("."), false);
    }

    // A classification can mention a registry definition that is not
    // reachable from this resource's emitted property graph. Do not present
    // that raw definition name as though it were a valid DSL path.
    (entry.property_path.to_string(), true)
}

fn markdown_section_heading_path(
    property_path: &[String],
    struct_defs: &BTreeMap<MarkdownStructKey, StructDefInfo>,
    strict_ancestor_of: Option<&StructDefInfo>,
) -> Vec<String> {
    let heading_path_depth = |path: &[String]| {
        path.iter()
            .filter(|segment| segment.as_str() != "*")
            .count()
    };

    // Inline struct identity is its full property path, so only a named
    // definition can have more than one distinct path.
    let Some((shared_ancestor, ancestor_path)) = struct_defs
        .values()
        .filter(|def_info| {
            def_info.property_paths.len() > 1
                && strict_ancestor_of.is_none_or(|descendant| {
                    heading_path_depth(&def_info.first_property_path)
                        < heading_path_depth(&descendant.first_property_path)
                })
        })
        .flat_map(|def_info| {
            def_info
                .property_paths
                .iter()
                .filter(|ancestor_path| property_path.starts_with(ancestor_path))
                .map(move |ancestor_path| (def_info, ancestor_path))
        })
        .max_by_key(|(_, ancestor_path)| heading_path_depth(ancestor_path))
    else {
        return markdown_heading_path(property_path);
    };

    let mut heading_path = vec![shared_ancestor.def_name.to_snake_case()];
    heading_path.extend(markdown_heading_path(&property_path[ancestor_path.len()..]));
    heading_path
}

fn register_markdown_enum(
    markdown_enums: &mut BTreeMap<MarkdownEnumKey, MarkdownEnum>,
    enum_namespace: String,
    heading_path: Vec<String>,
    prop_name: String,
    info: &EnumInfo,
) -> Result<MarkdownEnumKey> {
    let key = MarkdownEnumKey {
        namespace: enum_namespace,
        type_name: info.type_name.clone(),
    };
    if let Some(existing) = markdown_enums.get_mut(&key) {
        if existing.info.values != info.values {
            anyhow::bail!(
                "Markdown enum identity {}.{} has conflicting values {:?} and {:?}",
                key.namespace,
                key.type_name,
                existing.info.values,
                info.values
            );
        }
        if heading_path < existing.heading_path {
            existing.heading_path = heading_path;
            existing.prop_name = prop_name;
        }
    } else {
        markdown_enums.insert(
            key.clone(),
            MarkdownEnum {
                prop_name,
                heading_path,
                info: info.clone(),
            },
        );
    }

    Ok(key)
}

fn collect_markdown_enums(
    namespace: &str,
    property_enums: &BTreeMap<String, EnumInfo>,
    struct_field_enums: &BTreeMap<MarkdownStructFieldKey, EnumInfo>,
    struct_defs: &BTreeMap<MarkdownStructKey, StructDefInfo>,
) -> Result<(BTreeMap<MarkdownEnumKey, MarkdownEnum>, MarkdownEnumTargets)> {
    let mut markdown_enums: BTreeMap<MarkdownEnumKey, MarkdownEnum> = BTreeMap::new();
    let mut targets = MarkdownEnumTargets::default();

    for (prop_name, info) in property_enums {
        let key = register_markdown_enum(
            &mut markdown_enums,
            namespace.to_string(),
            markdown_section_heading_path(std::slice::from_ref(prop_name), struct_defs, None),
            prop_name.clone(),
            info,
        )?;
        targets.properties.insert(prop_name.clone(), key);
    }

    for (source_key, info) in struct_field_enums {
        let def_info = struct_defs.get(&source_key.struct_key).with_context(|| {
            format!(
                "enum {}.{} has no enclosing Markdown struct definition",
                source_key.struct_key.stable_identity(),
                source_key.field_name
            )
        })?;
        let field_path =
            extend_property_path(&def_info.first_property_path, &source_key.field_name);
        let heading_path = markdown_section_heading_path(&field_path, struct_defs, None);
        let key = register_markdown_enum(
            &mut markdown_enums,
            namespace_with_struct_path(namespace, &def_info.struct_path),
            heading_path,
            source_key.field_name.clone(),
            info,
        )?;
        targets.struct_fields.insert(source_key.clone(), key);
    }

    Ok((markdown_enums, targets))
}

fn markdown_enum_heading_texts(
    enums: &BTreeMap<MarkdownEnumKey, MarkdownEnum>,
) -> Vec<(MarkdownEnumKey, String)> {
    let candidates: Vec<_> = enums
        .iter()
        .map(|(key, enum_doc)| {
            let base = format!(
                "{} ({})",
                enum_doc.heading_path.join("."),
                enum_doc.info.type_name
            );
            let identity = format!("{} {}", key.namespace, key.type_name);
            (key.clone(), base, identity)
        })
        .collect();
    let mut base_counts = HashMap::new();
    for (_, heading, _) in &candidates {
        *base_counts
            .entry(carina_docgen::heading_slug(heading))
            .or_insert(0_usize) += 1;
    }
    let mut used_slugs = HashSet::new();
    let mut headings: Vec<_> = candidates
        .into_iter()
        .map(|(key, base, identity)| {
            let primary = if base_counts[&carina_docgen::heading_slug(&base)] > 1 {
                format!("{base} — {identity}")
            } else {
                base
            };
            let heading = claim_unique_markdown_heading(primary, &identity, &mut used_slugs);
            (key, heading)
        })
        .collect();
    sort_markdown_headings(&mut headings);
    headings
}

fn markdown_struct_heading_texts(
    struct_defs: &BTreeMap<MarkdownStructKey, StructDefInfo>,
) -> Vec<(MarkdownStructKey, String)> {
    let mut used_slugs = HashSet::new();
    let mut headings: Vec<_> = struct_defs
        .iter()
        .map(|(key, def_info)| {
            let base =
                markdown_section_heading_path(&def_info.first_property_path, struct_defs, None)
                    .join(".");
            let heading =
                claim_unique_markdown_heading(base, &key.stable_identity(), &mut used_slugs);
            (key.clone(), heading)
        })
        .collect();
    sort_markdown_headings(&mut headings);
    headings
}

fn sort_markdown_headings<K: Ord>(headings: &mut [(K, String)]) {
    headings.sort_by(|(left_key, left_heading), (right_key, right_heading)| {
        left_heading
            .cmp(right_heading)
            .then_with(|| left_key.cmp(right_key))
    });
}

fn claim_unique_markdown_heading(
    primary: String,
    stable_identity: &str,
    used_slugs: &mut HashSet<String>,
) -> String {
    let candidates = [primary.clone(), format!("{primary} — {stable_identity}")];
    for candidate in candidates {
        if used_slugs.insert(carina_docgen::heading_slug(&candidate)) {
            return candidate;
        }
    }
    for suffix in 2.. {
        let candidate = format!("{primary} {suffix}");
        if used_slugs.insert(carina_docgen::heading_slug(&candidate)) {
            return candidate;
        }
    }
    unreachable!("an integer suffix always yields a fresh heading")
}

#[cfg(test)]
fn generate_markdown(schema: &CfnSchema, type_name: &str) -> Result<String> {
    generate_markdown_with_example(schema, type_name, None)
}

fn generate_markdown_with_example(
    schema: &CfnSchema,
    type_name: &str,
    example: Option<&str>,
) -> Result<String> {
    reset_recursion_guard();
    derive_mutually_exclusive_property_groups(schema)?;

    let dsl_resource = dsl_resource_name_from_type(type_name)?;
    let namespace = format!("aws.{}", dsl_resource);

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
    let mut property_enums: BTreeMap<String, EnumInfo> = BTreeMap::new();
    let mut struct_field_enums: BTreeMap<MarkdownStructFieldKey, EnumInfo> = BTreeMap::new();
    let mut struct_defs: BTreeMap<MarkdownStructKey, StructDefInfo> = BTreeMap::new();

    for (prop_name, prop) in documented_top_level_properties(schema) {
        // Apply resource-scoped enum overlay (awscc#357) for top-level
        // properties. The nested-struct loop below already does the same
        // thing for `Struct.Field` keys (awscc#246); without this branch a
        // top-level Enum override (e.g. `TargetGroup.Protocol`) was
        // recorded in the schema (`generate_schema_code`) but never
        // mirrored into the markdown — `cfn_type_to_carina_type_with_enum`
        // does not consult `resource_type_overrides` itself.
        if let Some(TypeOverride::Enum(values) | TypeOverride::EnumUnordered(values)) =
            resource_type_override(type_name, CfnPropertyPath::new(None, prop_name))
        {
            property_enums.insert(prop_name.clone(), enum_info_for_override(prop_name, values));
            collect_struct_defs(
                prop,
                prop_name,
                schema,
                &[],
                std::slice::from_ref(prop_name),
                None,
                &mut struct_defs,
            );
            continue;
        }
        let (_, enum_info) =
            cfn_type_to_carina_type_with_enum(prop, prop_name, schema, &namespace, &property_enums);
        if let Some(info) = enum_info {
            property_enums.insert(prop_name.clone(), info);
        }
        // Collect struct definitions from $ref
        collect_struct_defs(
            prop,
            prop_name,
            schema,
            &[],
            std::slice::from_ref(prop_name),
            None,
            &mut struct_defs,
        );
    }

    // Scan struct definition fields for enum info (const values, $ref to enum-only definitions)
    for (struct_key, def_info) in &struct_defs {
        for (field_name, field_prop) in &def_info.properties {
            if is_deprecated_property(field_prop) {
                continue;
            }
            let cfn_property_path = CfnPropertyPath::new(Some(&def_info.def_name), field_name);
            if should_omit_property(type_name, cfn_property_path) {
                continue;
            }
            let source_key = MarkdownStructFieldKey {
                struct_key: struct_key.clone(),
                field_name: field_name.clone(),
            };
            // Apply nested-field enum overlay (awscc#246) — same lookup as
            // `generate_schema_code`, so the markdown's enum tables stay in
            // sync with the generated Rust schema.
            if let Some(nested_override) = resource_type_override(type_name, cfn_property_path) {
                match nested_override {
                    TypeOverride::Enum(values) | TypeOverride::EnumUnordered(values) => {
                        struct_field_enums
                            .insert(source_key, enum_info_for_override(field_name, values));
                        continue;
                    }
                    TypeOverride::StringType(_) | TypeOverride::AttributeType { .. } => continue,
                    _ => {}
                }
            }
            let field_path = extend_property_path(&def_info.first_property_path, field_name);
            let (_, enum_info) = cfn_type_to_carina_type_with_enum_with_paths(
                field_prop,
                field_name,
                schema,
                &namespace,
                &property_enums,
                CfnTypeGenerationPaths {
                    structural: &def_info.struct_path,
                    provider: &field_path,
                    cfn_parent: Some(&def_info.def_name),
                },
            );
            if let Some(info) = enum_info {
                struct_field_enums.insert(source_key, info);
            }
        }
    }

    // Validate nested read-only pointers against the same AttributeType
    // emitter used for generated Rust. Markdown has its own presentation
    // traversal, but it must not document a shape that the schema artifact
    // cannot represent safely.
    reset_recursion_guard();
    let has_arn_helper = resource_identity_parts(&dsl_resource).is_some()
        && (schema.properties.contains_key("Arn")
            || synthetic_attributes()
                .iter()
                .any(|synth| synth.cfn_type == type_name && synth.attr_name == "arn"));
    for (prop_name, prop) in documented_top_level_properties(schema) {
        let _ = generate_top_level_attribute_type(
            prop_name,
            prop,
            schema,
            &namespace,
            &property_enums,
            has_arn_helper,
        );
        let property_path = [prop_name.clone()];
        if is_property_read_only(schema, prop, &property_path) {
            record_emitted_read_only_path(&property_path);
        }
        if is_write_only_property_path(schema, &property_path) {
            record_emitted_write_only_path(&property_path);
        }
    }
    validate_emitted_nested_read_only_properties(schema)?;
    validate_emitted_nested_write_only_properties(schema)?;
    reset_recursion_guard();

    let (markdown_enums, enum_targets) = collect_markdown_enums(
        &namespace,
        &property_enums,
        &struct_field_enums,
        &struct_defs,
    )?;
    let enum_heading_texts = markdown_enum_heading_texts(&markdown_enums);
    let struct_heading_texts = markdown_struct_heading_texts(&struct_defs);
    let mut page = PageBuilder::new();

    // Title
    page.heading(Heading::plain(1, format!("awscc.{dsl_resource}")))?;
    page.push_str(&format!("CloudFormation Type: `{type_name}`\n\n"));

    // Description
    if let Some(desc) = &schema.description {
        page.push_str(&format!("{desc}\n\n"));
    }

    write_markdown_example(&mut page, example)?;

    // Argument Reference (writable attributes)
    page.heading(Heading::plain(2, "Argument Reference"))?;

    for (prop_name, prop) in documented_top_level_properties(schema) {
        let property_path = [prop_name.clone()];
        if is_property_read_only(schema, prop, &property_path) {
            continue;
        }

        let attr_name = prop_name.to_snake_case();
        let is_required = required.contains(prop_name);
        let type_display = markdown_type(
            prop_name,
            prop,
            schema,
            enum_targets.properties.get(prop_name),
            &property_path,
            None,
        );

        page.heading(Heading::code(3, &attr_name))?;
        page.push_str("- **Type:** ");
        type_display.write_to(&mut page);
        page.push_str("\n");
        if is_required {
            page.push_str("- **Required:** Yes\n");
        } else {
            page.push_str("- **Required:** No\n");
        }
        if create_only.contains(prop_name) {
            page.push_str("- **Create-only:** Yes\n");
        }
        if write_only.contains(prop_name) {
            page.push_str("- **Write-only:** Yes\n");
        }
        let override_default =
            match resource_type_override(type_name, CfnPropertyPath::new(None, prop_name)) {
                Some(TypeOverride::AttributeType { default, .. }) => *default,
                _ => None,
            };
        if let Some(default) = override_default {
            page.push_str(&format!("- **Default:** `{}`\n", default.markdown));
        } else if let Some(default_val) = &prop.default_value
            && let Some(display) = json_default_to_markdown(default_val)
        {
            page.push_str(&format!("- **Default:** `{display}`\n"));
        }
        page.push_str("\n");

        if let Some(d) = generated_property_description(schema, prop_name, prop) {
            let desc = collapse_whitespace(&d.replace('\n', " "));
            page.push_str(&format!("{desc}\n\n"));
        }
    }

    let unsupported_json_properties = FREE_FORM_JSON_PROPERTIES
        .iter()
        .chain(TEST_FREE_FORM_JSON_PROPERTIES)
        .filter(|entry| entry.cfn_type == type_name)
        .collect::<Vec<_>>();
    let omitted_resource_properties = OMITTED_RESOURCE_PROPERTIES
        .iter()
        .filter(|entry| entry.cfn_type == type_name)
        .collect::<Vec<_>>();
    if !unsupported_json_properties.is_empty() || !omitted_resource_properties.is_empty() {
        page.heading(Heading::plain(2, "Not yet supported"))?;
        if !unsupported_json_properties.is_empty() {
            page.push_str(
                "These CloudFormation properties require the free-form JSON document type tracked by [carina-rs/carina#4144](https://github.com/carina-rs/carina/issues/4144):\n\n",
            );
            for entry in unsupported_json_properties {
                let (name, definition_qualified) =
                    classified_property_doc_name(entry, &struct_defs);
                if definition_qualified {
                    page.push_str(&format!(
                        "- Definition-qualified name `{name}` — {}\n",
                        entry.reason
                    ));
                } else {
                    page.push_str(&format!("- `{name}` — {}\n", entry.reason));
                }
            }
            page.push_str("\n");
        }
        if !omitted_resource_properties.is_empty() {
            page.push_str(
                "These CloudFormation properties are deliberately omitted for the provider-specific reasons below:\n\n",
            );
            for entry in omitted_resource_properties {
                let (name, definition_qualified) =
                    classified_property_doc_name(entry, &struct_defs);
                if definition_qualified {
                    page.push_str(&format!(
                        "- Definition-qualified name `{name}` — {}\n",
                        entry.reason
                    ));
                } else {
                    page.push_str(&format!("- `{name}` — {}\n", entry.reason));
                }
            }
            page.push_str("\n");
        }
    }

    // Enum values section
    if !markdown_enums.is_empty() {
        let aliases = known_enum_aliases();
        page.heading(Heading::plain(2, "Enum Values"))?;
        for (key, heading_text) in &enum_heading_texts {
            let enum_doc = markdown_enums
                .get(key)
                .expect("every enum heading must refer to a discovered enum");
            let enum_info = &enum_doc.info;
            let prop_aliases = aliases.get(enum_doc.prop_name.as_str());
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
            page.target_heading(
                MarkdownTarget::Enum(key.clone()),
                Heading::plain(3, heading_text),
            )?;
            let rows = enum_info.values.iter().map(|value| {
                let dsl_value = dsl_for(value);
                let dsl_id = format!("{}.{}.{}", key.namespace, enum_info.type_name, dsl_value);
                [TableCell::code(value), TableCell::code(dsl_id)]
            });
            page.table(["Value", "DSL Identifier"], rows)?;
            page.push_str("\n");
            let empty = String::new();
            let first_value = enum_info.values.first().unwrap_or(&empty);
            let first_dsl = dsl_for(first_value);
            page.push_str(&format!(
                "Shorthand formats: `{}` or `{}.{}`\n\n",
                first_dsl, enum_info.type_name, first_dsl,
            ));
        }
    }

    // Struct Definitions section
    if !struct_defs.is_empty() {
        page.heading(Heading::plain(2, "Struct Definitions"))?;
        for (key, heading_text) in &struct_heading_texts {
            let def_info = struct_defs
                .get(key)
                .expect("every struct heading must refer to a discovered struct");
            page.target_heading(
                MarkdownTarget::Struct(key.clone()),
                Heading::plain(3, heading_text),
            )?;
            let used_by = def_info
                .property_paths
                .iter()
                .map(|path| {
                    markdown_section_heading_path(path, &struct_defs, Some(def_info)).join(".")
                })
                .collect::<BTreeSet<_>>();
            if used_by.len() > 1 {
                let used_by = used_by
                    .into_iter()
                    .map(|path| format!("`{path}`"))
                    .collect::<Vec<_>>()
                    .join(", ");
                page.push_str(&format!("Used by: {used_by}\n\n"));
            }
            let required_set: HashSet<&str> =
                def_info.required.iter().map(|s| s.as_str()).collect();
            let overrides = known_enum_overrides();
            let mut rows = Vec::new();
            for (field_name, field_prop) in &def_info.properties {
                if is_deprecated_property(field_prop) {
                    continue;
                }
                let cfn_property_path = CfnPropertyPath::new(Some(&def_info.def_name), field_name);
                if should_omit_property(type_name, cfn_property_path) {
                    continue;
                }
                let field_path = extend_property_path(&def_info.first_property_path, field_name);
                let snake_name = field_name.to_snake_case();
                let is_read_only = is_property_read_only(schema, field_prop, &field_path);
                let is_req = required_set.contains(field_name.as_str()) && !is_read_only;
                let source_key = MarkdownStructFieldKey {
                    struct_key: key.clone(),
                    field_name: field_name.clone(),
                };
                let nested_key = cfn_property_path.to_string();
                let nested_override = resource_type_override(type_name, cfn_property_path);
                let field_type_display =
                    if let Some(TypeOverride::AttributeType { display, .. }) = nested_override {
                        MarkdownType::Plain((*display).to_string())
                    } else if matches!(nested_override, Some(TypeOverride::IntegerFromNumber)) {
                        MarkdownType::Plain(integer_from_number_display(
                            field_prop,
                            type_name,
                            &nested_key,
                        ))
                    } else if let Some(enum_target) = enum_targets.struct_fields.get(&source_key) {
                        markdown_type(
                            field_name,
                            field_prop,
                            schema,
                            Some(enum_target),
                            &field_path,
                            Some(&def_info.def_name),
                        )
                    } else if overrides.contains_key(field_name.as_str()) {
                        MarkdownType::Plain("Enum".to_string())
                    } else {
                        markdown_type(
                            field_name,
                            field_prop,
                            schema,
                            None,
                            &field_path,
                            Some(&def_info.def_name),
                        )
                    };
                let mut desc = collapse_whitespace(
                    &field_prop
                        .description
                        .as_deref()
                        .unwrap_or("")
                        .replace('\n', " "),
                );
                if is_read_only {
                    if !desc.is_empty() {
                        desc.push(' ');
                    }
                    desc.push_str("Provider-populated; cannot be set.");
                }
                rows.push([
                    TableCell::code(snake_name),
                    field_type_display.table_cell(),
                    TableCell::text(if is_req { "Yes" } else { "No" }),
                    TableCell::text(desc),
                ]);
            }
            page.table(["Field", "Type", "Required", "Description"], rows)?;
            page.push_str("\n");
        }
    }

    // Attribute Reference (read-only attributes)
    let synth_for_type: Vec<&SyntheticAttribute> = synthetic_attributes()
        .iter()
        .filter(|s| s.cfn_type == type_name)
        .collect();
    let has_read_only = documented_top_level_properties(schema).any(|(name, property)| {
        let property_path = [name.clone()];
        is_property_read_only(schema, property, &property_path)
    }) || !synth_for_type.is_empty();
    if has_read_only {
        page.heading(Heading::plain(2, "Attribute Reference"))?;

        for (prop_name, prop) in documented_top_level_properties(schema) {
            let property_path = [prop_name.clone()];
            if !is_property_read_only(schema, prop, &property_path) {
                continue;
            }

            let attr_name = prop_name.to_snake_case();
            let type_display = markdown_type(
                prop_name,
                prop,
                schema,
                enum_targets.properties.get(prop_name),
                &property_path,
                None,
            );

            page.heading(Heading::code(3, &attr_name))?;
            page.push_str("- **Type:** ");
            type_display.write_to(&mut page);
            page.push_str("\n");

            if let Some(d) = &prop.description {
                let desc = collapse_whitespace(&d.replace('\n', " "));
                page.push_str(&format!("\n{desc}\n\n"));
            } else {
                page.push_str("\n");
            }
        }

        for synth in synth_for_type {
            page.heading(Heading::code(3, synth.attr_name))?;
            page.push_str("- **Type:** String\n\n");
            page.push_str(&format!(
                "{}\n\n",
                collapse_whitespace(&synth.description.replace('\n', " "))
            ));
        }
    }

    page.finish()
        .with_context(|| format!("generated invalid Markdown for awscc.{dsl_resource}"))
}

fn write_markdown_example(
    page: &mut PageBuilder<MarkdownTarget>,
    example: Option<&str>,
) -> Result<()> {
    let Some(example) = example else {
        return Ok(());
    };

    page.heading(Heading::plain(2, "Example"))?;
    page.push_str("```crn\n");
    page.push_str(example);
    if !example.ends_with('\n') {
        page.push_str("\n");
    }
    page.push_str("```\n\n");
    Ok(())
}

/// Collect struct definitions from properties for markdown documentation
fn collect_struct_defs(
    prop: &CfnProperty,
    prop_name: &str,
    schema: &CfnSchema,
    struct_path: &[String],
    property_path: &[String],
    cfn_parent: Option<&str>,
    struct_defs: &mut BTreeMap<MarkdownStructKey, StructDefInfo>,
) {
    if !prop.one_of.is_empty() {
        if let Some(array_variant) = prop
            .one_of
            .iter()
            .find(|variant| markdown_property_is_array(variant, schema))
        {
            collect_struct_defs(
                array_variant,
                prop_name,
                schema,
                struct_path,
                property_path,
                cfn_parent,
                struct_defs,
            );
        }
        return;
    }

    if let Some(struct_def) = markdown_struct_def(prop_name, prop, schema, property_path) {
        let MarkdownStructDef {
            key,
            name,
            properties,
            required,
        } = struct_def;
        let struct_cfn_parent = name.clone();
        let def_path = extend_struct_path(struct_path, &name);
        let should_walk_children = match struct_defs.entry(key) {
            std::collections::btree_map::Entry::Vacant(entry) => {
                entry.insert(StructDefInfo {
                    def_name: name,
                    struct_path: def_path.clone(),
                    first_property_path: property_path.to_vec(),
                    property_paths: BTreeSet::from([property_path.to_vec()]),
                    properties: properties.clone(),
                    required,
                });
                true
            }
            std::collections::btree_map::Entry::Occupied(mut entry) => {
                entry
                    .get_mut()
                    .property_paths
                    .insert(property_path.to_vec());
                false
            }
        };
        if should_walk_children {
            for (field_name, field_prop) in &properties {
                if is_deprecated_property(field_prop) {
                    continue;
                }
                if should_omit_property(
                    &schema.type_name,
                    CfnPropertyPath::new(Some(&struct_cfn_parent), field_name),
                ) {
                    continue;
                }
                let field_path = extend_property_path(property_path, field_name);
                collect_struct_defs(
                    field_prop,
                    field_name,
                    schema,
                    &def_path,
                    &field_path,
                    Some(&struct_cfn_parent),
                    struct_defs,
                );
            }
        }
        return;
    }

    if let Some(value) = property_object_map_value(prop, schema) {
        object_map_value_or_panic(schema, CfnPropertyPath::new(cfn_parent, prop_name), value);
        let value_path = extend_property_path(property_path, "*");
        collect_struct_defs(
            value,
            prop_name,
            schema,
            struct_path,
            &value_path,
            cfn_parent,
            struct_defs,
        );
        return;
    }

    let Some(items) = markdown_array_items(prop, schema) else {
        return;
    };
    let array_def_name = prop.ref_path.as_deref().and_then(|ref_path| {
        matches!(
            resolve_ref_classified(schema, ref_path),
            Some(ResolvedDef::Array { .. })
        )
        .then(|| ref_def_name(ref_path))
        .flatten()
    });
    let should_visit = array_def_name.is_none_or(|name| {
        IN_PROGRESS_DEFS.with(|defs| defs.borrow_mut().insert(name.to_string()))
    });
    if should_visit {
        let item_path = extend_property_path(property_path, "*");
        collect_struct_defs(
            items,
            prop_name,
            schema,
            struct_path,
            &item_path,
            cfn_parent,
            struct_defs,
        );
        if let Some(name) = array_def_name {
            IN_PROGRESS_DEFS.with(|defs| {
                defs.borrow_mut().remove(name);
            });
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

/// Regex for detecting unqualified "specify either X or Y" sentences in
/// property descriptions. X and Y can be PascalCase names or
/// ``backtick-quoted`` names.
static EXCLUSIVE_DESC_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?im)(?:^|[.!?]\s+|\n\s*)(?:you\s+must\s+)?specify\s+either\s*`*([A-Z][A-Za-z0-9]*)`*\s+or\s+`*([A-Z][A-Za-z0-9]*)`*",
    )
    .unwrap()
});

/// Detect mutually exclusive field groups from property description text.
///
/// Looks for patterns like:
/// - "You must specify either InternetGatewayId or VpnGatewayId, but not both"
/// - "You must specify either``CidrBlock`` or ``Ipv4IpamPoolId``"
/// - "Specify either X or Y"
///
/// A condition-qualified sentence such as "For an alarm based on a metric,
/// you must specify either X or Y" is deliberately ignored. Carina's portable
/// `exclusive_required` constraint means exactly one globally; treating a
/// conditional clause as that unconditional rule rejects the other valid
/// resource modes.
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

/// Produce the exact AttributeType expression used by a top-level generated
/// attribute. Besides returning Rust source, this walks every nested struct
/// that survives overrides and records its emitted read-only fields.
fn generate_top_level_attribute_type(
    prop_name: &str,
    prop: &CfnProperty,
    schema: &CfnSchema,
    namespace: &str,
    enums: &BTreeMap<String, EnumInfo>,
    has_arn_helper: bool,
) -> String {
    if has_arn_helper && prop_name == "Arn" {
        return "self::arn()".to_string();
    }

    if let Some(TypeOverride::AttributeType { expression, .. }) =
        resource_type_override(&schema.type_name, CfnPropertyPath::new(None, prop_name))
    {
        return (*expression).to_string();
    }

    if let Some(enum_info) = enums.get(prop_name) {
        let aliases = known_enum_aliases();
        let prop_aliases = aliases.get(prop_name);
        let dsl_aliases_code = dsl_aliases_code(&enum_info.values, prop_aliases);
        let values_str = enum_info
            .values
            .iter()
            .map(|value| format!("{}.to_string()", rust_lit(value)))
            .collect::<Vec<_>>()
            .join(", ");
        let enum_type = format!(
            r#"AttributeType::enum_(carina_core::schema::enum_identity({}, Some({})), Some(vec![{}]), {}, None, None)"#,
            rust_lit(&enum_info.type_name),
            rust_lit(namespace),
            values_str,
            dsl_aliases_code
        );
        let is_array = prop
            .prop_type
            .as_ref()
            .and_then(TypeValue::as_str)
            .is_some_and(|prop_type| prop_type == "array");
        let is_ref_array = prop
            .ref_path
            .as_ref()
            .and_then(|ref_path| resolve_ref_classified(schema, ref_path))
            .is_some_and(|definition| matches!(definition, ResolvedDef::Array { .. }));
        if is_array || is_ref_array {
            let list_ctor = list_constructor(
                prop.insertion_order,
                override_forces_unordered(&schema.type_name, CfnPropertyPath::new(None, prop_name)),
            );
            return format!("{}({})", list_ctor, enum_type);
        }
        return enum_type;
    }

    cfn_type_to_carina_type_with_enum(prop, prop_name, schema, namespace, enums).0
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
    let arn_helper = match resource_identity_parts(&dsl_resource) {
        Some((service, resource))
            if schema.properties.contains_key("Arn")
                || synthetic_attributes()
                    .iter()
                    .any(|synth| synth.cfn_type == type_name && synth.attr_name == "arn") =>
        {
            Some(emit_arn_helper(service, resource)?)
        }
        _ => None,
    };
    let has_arn_helper = arn_helper.is_some();

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

    // Pre-scan properties to determine which imports are needed and collect enum info.
    // Numeric, length, and pattern constraints are represented declaratively in
    // `AttributeType`; they do not need generated callback functions.
    let mut needs_types = false;
    let mut needs_attribute_type = false;
    let mut needs_struct_field = false;
    let mut enums: BTreeMap<String, EnumInfo> = BTreeMap::new();

    for (prop_name, prop) in &schema.properties {
        if is_deprecated_property(prop)
            || should_omit_top_level_property(&schema.type_name, prop_name)
        {
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
        // Check resource-scoped enum overrides.
        let resource_override =
            resource_type_override(type_name, CfnPropertyPath::new(None, prop_name));
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
                    let cfn_property_path = CfnPropertyPath::new(Some(def_name), field_name);
                    if should_omit_property(&schema.type_name, cfn_property_path) {
                        continue;
                    }
                    let composite_key = cfn_property_path.to_string();
                    // Curated nested-field enum overlay (awscc#246). Wins over
                    // the natural cfn_type inference so that fields the CFN
                    // schema leaves as plain strings still register as enums
                    // in the `enums` map (and therefore get a VALID_* constant
                    // and an enum_valid_values() entry alongside StringEnum
                    // emission in `generate_struct_type`).
                    if let Some(nested_override) =
                        resource_type_override(&schema.type_name, cfn_property_path)
                    {
                        match nested_override {
                            TypeOverride::Enum(values) | TypeOverride::EnumUnordered(values) => {
                                enums.insert(
                                    composite_key.clone(),
                                    enum_info_for_override(field_name, values),
                                );
                                continue;
                            }
                            TypeOverride::StringType(_) | TypeOverride::AttributeType { .. } => {
                                continue;
                            }
                            _ => {}
                        }
                    }
                    let structural_path = [def_name.clone()];
                    let provider_path = [field_name.clone()];
                    let (_, field_enum_info) = cfn_type_to_carina_type_with_enum_with_paths(
                        field_prop,
                        field_name,
                        schema,
                        &namespace,
                        &enums,
                        CfnTypeGenerationPaths {
                            structural: &structural_path,
                            provider: &provider_path,
                            cfn_parent: Some(def_name),
                        },
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
    // definitions to populate `enums`. That same recursion also populated
    // `EMITTED_DEFS` with
    // def bodies that hard-coded the *unqualified* enum `type_name`s.
    // Now that qualification has rewritten the enums map, drop those
    // stale bodies so the upcoming attribute-emission phase re-walks
    // every def with the qualified enums in scope. (carina#3350.)
    //
    EMITTED_DEFS.with(|s| s.borrow_mut().clear());
    BACK_REFERENCED_DEFS.with(|s| s.borrow_mut().clear());
    reset_property_mode_emission_state();

    let has_enums = !enums.is_empty();
    // Enums use AttributeType::Custom with AttributeType::String base
    if has_enums {
        needs_attribute_type = true;
    }

    // Derive both the tag property name and CloudControl wire representation
    // from the schema. Unsupported shapes are codegen errors rather than an
    // implicit Key/Value-list default.
    let tags = derive_tags_property(schema)?;
    let tags_expr = tags.as_ref().map_or_else(
        || "None".to_string(),
        |tags| {
            format!(
                "Some(TagsProperty {{ provider_name: {}, wire: {} }})",
                rust_lit(&tags.provider_name),
                tags.wire.rust_variant()
            )
        },
    );

    // Detect mutually exclusive required fields for validation, and derive the
    // separately curated updatable groups used to build CloudControl patches.
    let exclusive_required_groups = detect_exclusive_fields(schema, type_name);
    let exclusive_groups = derive_mutually_exclusive_property_groups(schema)?;
    let exclusive_groups_expr = emit_exclusive_groups_slice(&exclusive_groups);

    // Build the body into a separate buffer so import selection can scan it
    // for `legacy_validator(` mentions instead of
    // approximating from `has_*` flags (which over-included for files whose
    // enums lower to `types::*` rather than to `Custom`). The header — including
    // all `use` lines — is constructed at the end.
    let mut body = String::new();
    if let Some(helper) = &arn_helper {
        body.push_str(helper);
    }

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

    // Generate config function
    let config_fn_name = format!("{}_config", full_resource);
    // Schema `resource_type` is the un-prefixed DSL form (e.g. `ec2.Vpc`).
    // The provider name (`awscc`) is supplied separately by
    // `SchemaRegistry::insert("awscc", schema)` on the consumer side.
    let schema_name = dsl_resource.clone();
    let primary_identifier =
        emit_primary_identifier(schema.primary_identifier.as_deref(), &schema.properties)?;

    // Build every schema incrementally through non-inlined top-level helpers.
    // This keeps generated builder frame sizes bounded independently of how
    // large a resource schema grows.
    body.push_str(&format!(
        r#"/// Returns the schema config for {} ({})
#[rustfmt::skip]
pub fn {}() -> AwsccSchemaConfig {{
    let mut schema = ResourceSchema::new("{}")
"#,
        full_resource, type_name, config_fn_name, schema_name
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
    body.pop();
    body.push_str(";\n");

    // Generate attributes for each property
    for (prop_name, prop) in &schema.properties {
        if should_omit_top_level_property(&schema.type_name, prop_name) {
            continue;
        }
        let attr_name = prop_name.to_snake_case();
        let property_path = [prop_name.clone()];
        let is_read_only = is_property_read_only(schema, prop, &property_path);
        let is_required = required.contains(prop_name) && !is_read_only;
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

        let attr_type = generate_top_level_attribute_type(
            prop_name,
            prop,
            schema,
            &namespace,
            &enums,
            has_arn_helper,
        );

        let constructor = if is_read_only {
            "provider_populated"
        } else if is_required {
            "required"
        } else {
            "optional"
        };
        let attr_builder_fn = format!("build_{}_attribute", attr_name);
        let mut attr_code = format!(
            "    #[inline(never)]\n    fn {}() -> AttributeSchema {{\n        AttributeSchema::{}(\"{}\", {})", // rust-lit-guard: allow (snake_case output cannot contain " or \\)
            attr_builder_fn, constructor, attr_name, attr_type
        );

        if is_create_only {
            attr_code.push_str("\n                .create_only()");
        }

        if is_read_only {
            record_emitted_read_only_path(&property_path);
        }

        if is_write_only {
            attr_code.push_str("\n                .write_only()");
            record_emitted_write_only_path(&property_path);
        }

        if is_identity {
            attr_code.push_str("\n                .identity()");
        }

        if let Some(desc) = generated_property_description(schema, prop_name, prop) {
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
        let override_default = match resource_type_override(
            &schema.type_name,
            CfnPropertyPath::new(None, prop_name),
        ) {
            Some(TypeOverride::AttributeType { default, .. }) => *default,
            _ => None,
        };
        if let Some(default) = override_default {
            attr_code.push_str(&format!(
                "\n                .with_default({})",
                default.value_expression
            ));
        } else if let Some(default_val) = &prop.default_value
            && let Some(default_code) = json_default_to_value_code(default_val)
            && attr_type_accepts_scalar_default(&attr_type)
        {
            attr_code.push_str(&format!("\n                .with_default({default_code})"));
        }

        // Add block_name for List(Struct) attributes with a natural singular form.
        // Even if the singular form conflicts with an existing field name,
        // resolve_block_names distinguishes block syntax (Value::List) from
        // attribute assignment (Value::Map) so the block_name is safe to add.
        let override_block_name = match resource_type_override(
            &schema.type_name,
            CfnPropertyPath::new(None, prop_name),
        ) {
            Some(TypeOverride::AttributeType { block_name, .. }) => *block_name,
            _ => None,
        };
        let block_name = override_block_name.map(str::to_owned).or_else(|| {
            if attr_type.contains("list(AttributeType::struct_")
                || is_list_of_struct_property(prop, schema)
            {
                compute_block_name(&attr_name)
            } else {
                None
            }
        });
        if !is_read_only && let Some(singular) = block_name {
            attr_code.push_str(&format!(
                "\n                .with_block_name({})",
                rust_lit(&singular)
            ));
        }

        attr_code.push_str(&format!(
            "\n            .build()\n    }}\n    schema = schema.attribute({}());\n",
            attr_builder_fn
        ));
        body.push_str(&attr_code);
    }

    validate_emitted_nested_read_only_properties(schema)?;
    validate_emitted_nested_write_only_properties(schema)?;

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
        let attr_type = if has_arn_helper && synth.attr_name == "arn" {
            "self::arn()"
        } else {
            synth.attr_type
        };
        body.push_str(&format!(
            concat!(
                "    #[inline(never)]\n",
                "    fn build_{}_attribute() -> AttributeSchema {{\n",
                "        AttributeSchema::provider_populated({}, {})\n",
                "            .with_description(\"{} (read-only)\")\n", // rust-lit-guard: allow (already escaped and newline-collapsed)
                "            .build()\n",
                "    }}\n",
                "    schema = schema.attribute(build_{}_attribute());\n", // rust-lit-guard: allow (snake_case identifier)
            ),
            synth.attr_name,
            rust_lit(synth.attr_name),
            attr_type,
            escaped,
            synth.attr_name,
        ));
    }

    let replacement_class = declared_replacement_class(&schema_name, schema)?;
    body.push_str("    schema = schema\n");
    emit_replacement_class_builder(&mut body, replacement_class);
    body.pop();
    body.push_str(";\n");

    // Per-resource operational config for resources with slow CloudControl operations.
    let op_config = operation_config_for_type(type_name);
    if let Some(cfg) = op_config {
        body.push_str("    schema = schema.with_operation_config(OperationConfig {\n");
        emit_option_field(&mut body, "delete_timeout_secs", cfg.delete_timeout_secs);
        emit_option_field(&mut body, "delete_max_retries", cfg.delete_max_retries);
        emit_option_field(&mut body, "create_timeout_secs", cfg.create_timeout_secs);
        emit_option_field(&mut body, "create_max_retries", cfg.create_max_retries);
        emit_option_field(&mut body, "update_timeout_secs", cfg.update_timeout_secs);
        body.push_str("    });\n");
    }

    // Mutually exclusive required groups are emitted as declarative data so
    // the constraint survives the WASM plugin boundary (function-pointer
    // validators are lost when schemas cross into the host).
    for group in &exclusive_required_groups {
        let fields_str = group
            .iter()
            .map(|f| {
                let snake = f.to_snake_case();
                rust_lit(&snake).to_string()
            })
            .collect::<Vec<_>>()
            .join(", ");
        body.push_str(&format!(
            "    schema = schema.exclusive_required(&[{}]);\n",
            fields_str
        ));
    }

    // Tags map-shape validation is portable schema data so native and WASM
    // consumers admit the same schema and enforce the same rule.
    if tags.is_some() {
        body.push_str("    schema = schema.with_declared_validators(vec![carina_provider_protocol::types::ValidatorType::TagsKeyValueCheck]);\n");
    }

    // carina#3340: drain any cyclic-struct definitions that the
    // `$ref` recursion guard accumulated during attribute emission
    // and attach them to the schema via `.with_def(...)`. Non-cyclic
    // resources contribute nothing here; the map is empty and no
    // calls are emitted. The Rust string in `def_body` is already a
    // valid `AttributeType` expression (the same string the inline
    // attribute emitter would have produced).
    let defs = take_emitted_defs();
    for (def_index, (def_name, def_body)) in defs.iter().enumerate() {
        body.push_str(&format!(
            concat!(
                "    #[inline(never)]\n",
                "    fn build_schema_def_{}() -> AttributeType {{ {} }}\n",
                "    schema = schema.with_def({}, build_schema_def_{}());\n",
            ),
            def_index,
            def_body,
            rust_lit(def_name),
            def_index,
        ));
    }

    // Close the schema (ResourceSchema) and the AwsccSchemaConfig struct.
    body.push_str(&format!(
        r#"    AwsccSchemaConfig {{
        aws_type_name: "{}",
        resource_type_name: "{}",
        primary_identifier: {},
        exclusive_groups: {},
        tags: {},
        schema,
    }}
}}
"#,
        type_name, dsl_resource, primary_identifier, exclusive_groups_expr, tags_expr
    ));

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

    let create_permissions = handler_permissions(schema, "create");
    let read_permissions = handler_permissions(schema, "read");
    let update_permissions = handler_permissions(schema, "update");
    let delete_permissions = handler_permissions(schema, "delete");
    body.push_str(&format!(
        "\n/// Returns the IAM permissions declared by the CloudFormation handler for this operation.\n\
         pub fn required_permissions(op: carina_core::effect::PlanOp) -> &'static [&'static str] {{\n\
         \x20   match op {{\n\
         \x20       carina_core::effect::PlanOp::Create => {},\n\
         \x20       carina_core::effect::PlanOp::Read => {},\n\
         \x20       carina_core::effect::PlanOp::Update => {},\n\
         \x20       carina_core::effect::PlanOp::Delete => {},\n\
         \x20   }}\n\
         }}\n",
        emit_permissions_slice(&create_permissions),
        emit_permissions_slice(&read_permissions),
        emit_permissions_slice(&update_permissions),
        emit_permissions_slice(&delete_permissions)
    ));

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
    let config_import = match (
        body.contains("ExclusiveGroup"),
        body.contains("TagsProperty"),
    ) {
        (true, true) => {
            "use crate::schemas::config::{AwsccSchemaConfig, ExclusiveGroup, TagsProperty, TagsWire};"
        }
        (true, false) => "use crate::schemas::config::{AwsccSchemaConfig, ExclusiveGroup};",
        (false, true) => "use crate::schemas::config::{AwsccSchemaConfig, TagsProperty, TagsWire};",
        (false, false) => "use crate::schemas::config::AwsccSchemaConfig;",
    };
    code.push_str(&format!(
        r#"//! {} schema definition for AWS Cloud Control
//!
//! Auto-generated from CloudFormation schema: {}
//!
//! DO NOT EDIT MANUALLY - regenerate with carina-codegen

use carina_core::schema::{{{}}};
{}
"#,
        resource, type_name, schema_imports_str, config_import
    ));
    let has_defaults = body.contains(".with_default(");
    if has_defaults || body.contains("Value::") {
        code.push_str("use carina_core::resource::{ConcreteValue, Value};\n");
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
    update_timeout_secs: Option<u64>,
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
            delete_timeout_secs: Some(3600),
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

fn strip_wrapping_double_backticks(value: &str) -> &str {
    value
        .strip_prefix("``")
        .and_then(|value| value.strip_suffix("``"))
        .unwrap_or(value)
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

    // Prefer list members introduced as a leading ``value`` entry under a
    // value-list sentence. Lexical anchors recognize documented intent words
    // ("valid values", "supported values", "following", "available",
    // "specify which"); the structural anchor recognizes CFN's free-form
    // colon-terminated lead-in immediately followed by backticked bullets.
    // Neither shape covers the full cache alone, and both keep extraction
    // scoped to value-list bullets instead of all description backticks.
    let lower_description = description.to_lowercase();
    let has_bullet_value_list_anchor = lower_description.contains("following")
        || lower_description.contains("available")
        || lower_description.contains("valid values")
        || lower_description.contains("supported values")
        || lower_description.contains("specify which")
        || Regex::new(r"(?m)(?:^|\n)\s*[^`\n]{1,240}:\s*\n\s*(?:[+*]|\d+[.)])?\s*``")
            .ok()
            .is_some_and(|re| re.is_match(description));
    if has_bullet_value_list_anchor
        && let Ok(bullet_leader_re) =
            Regex::new(r"(?m)(?:^|\n)\s*(?:[+*]|\d+[.)])?\s*``([^`]+)``\s*(?::|-|–|—)")
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
        r"(?is)(?:supported values?\s+(?:are|is)|valid values?(?:\s+for\s+``[^`]+``)?):\s*(.+?)(?:\.|\n|$)|(?:supported values?|valid values?(?:\s+for\s+``[^`]+``)?)\s+(?:are|is)\s+(.+?)(?:\.|\n|$)",
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

        let values: Vec<String> = candidates
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

        let values: Vec<String> = candidates
            .into_iter()
            .map(|v| strip_wrapping_double_backticks(v.trim()).to_string())
            .filter(|v| {
                !v.is_empty()
                    && !looks_like_property_name(v)
                    && !v.contains(' ')
                    && looks_like_enum_value(v)
            })
            .collect();

        if values.len() >= 2 {
            return deduplicate_enum_values(values);
        }
    }

    // Strategy 4: Look for prose lists like
    // "The possible values are A, B, and C" without backticks or a colon.
    //
    // The head noun is gated to a small allowlist
    // (values / protocols / modes / policies). Plain `supported X are ...`
    // without one of those nouns is too generic and false-positives on
    // sentences like "supported features are described in the docs".
    //
    // Multiple matching sentences are *unioned* — `Listener.Protocol`
    // carries two "supported protocols are ..." clauses (ALB vs NLB), and
    // the legal enum value set is the union over load-balancer types.
    if let (Ok(prose_values_re), Ok(connector_re)) = (
        Regex::new(
            r"(?i)(?:possible|valid|supported)\s+(?:values?|protocols?|modes?|policies)\s+(?:are|is)\s+([^\n.]+)",
        ),
        Regex::new(r"(?i)\s+(?:and|or)\s+"),
    ) {
        let mut candidates: Vec<String> = vec![];
        for cap in prose_values_re.captures_iter(description) {
            let clause = cap[1].trim();
            for comma_part in clause.split(',') {
                for part in connector_re.split(comma_part) {
                    let value = part
                        .split_once(" (")
                        .map(|(value, _)| value)
                        .unwrap_or(part)
                        .trim();
                    candidates.push(strip_wrapping_double_backticks(value).to_string());
                }
            }
        }

        let values: Vec<String> = candidates
            .into_iter()
            .filter(|v| {
                !v.is_empty()
                    && !v.contains(' ')
                    && looks_like_enum_value(v)
                    && !looks_like_property_name(v)
            })
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
    /// Object map whose `patternProperties` declares one value schema.
    ObjectMap { value: &'a CfnProperty },
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
/// has always used, extended with the object-map shape: properties → object
/// map → oneOf → enum → array → string-pattern → scalar → opaque. It is the
/// one place that order lives; every `ResolvedDef`
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
    if def.def_type.as_deref() == Some("object")
        && let Some(patterns) = &def.pattern_properties
        && patterns.len() == 1
    {
        return ResolvedDef::ObjectMap {
            value: patterns
                .values()
                .next()
                .expect("one patternProperties entry"),
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ObjectMapValueKind {
    DirectString,
    Struct,
}

fn is_direct_string_property(property: &CfnProperty) -> bool {
    matches!(
        property.prop_type.as_ref(),
        Some(TypeValue::Single(property_type)) if property_type == "string"
    ) && property.ref_path.is_none()
        && property.one_of.is_empty()
        && property.any_of.is_empty()
}

fn classify_object_map_value(
    schema: &CfnSchema,
    value: &CfnProperty,
) -> Option<ObjectMapValueKind> {
    if is_direct_string_property(value) {
        return Some(ObjectMapValueKind::DirectString);
    }
    if value
        .properties
        .as_ref()
        .is_some_and(|properties| !properties.is_empty())
        || value.ref_path.as_deref().is_some_and(|ref_path| {
            matches!(
                resolve_ref_classified(schema, ref_path),
                Some(ResolvedDef::Struct { .. })
            )
        })
    {
        return Some(ObjectMapValueKind::Struct);
    }
    None
}

fn object_map_value_or_panic(
    schema: &CfnSchema,
    property_path: CfnPropertyPath<'_>,
    value: &CfnProperty,
) -> ObjectMapValueKind {
    classify_object_map_value(schema, value).unwrap_or_else(|| {
        panic!(
            "unsupported object map {}.{}; patternProperties values must be direct strings or structs; see CODEGEN.md#classifying-bare-objects",
            schema.type_name, property_path
        )
    })
}

fn direct_object_map_value(prop: &CfnProperty) -> Option<&CfnProperty> {
    let patterns = prop.pattern_properties.as_ref()?;
    (patterns.len() == 1).then(|| {
        patterns
            .values()
            .next()
            .expect("one patternProperties entry")
    })
}

fn property_object_map_value<'a>(
    prop: &'a CfnProperty,
    schema: &'a CfnSchema,
) -> Option<&'a CfnProperty> {
    if let Some(value) = direct_object_map_value(prop) {
        return Some(value);
    }
    prop.ref_path
        .as_deref()
        .and_then(|ref_path| resolve_ref_classified(schema, ref_path))
        .and_then(|resolved| match resolved {
            ResolvedDef::ObjectMap { value } => Some(value),
            _ => None,
        })
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

struct StructGenerationPaths<'a> {
    structural: &'a [String],
    provider: &'a [String],
}

/// Generate Rust code for an AttributeType::Struct from a set of properties
fn generate_struct_type(
    def_name: &str,
    properties: &BTreeMap<String, CfnProperty>,
    required: &[String],
    schema: &CfnSchema,
    namespace: &str,
    enums: &BTreeMap<String, EnumInfo>,
    paths: StructGenerationPaths<'_>,
) -> String {
    let emission_mark = read_only_emission_mark();
    let write_only_emission_mark = write_only_emission_mark();
    let current_struct_path = extend_struct_path(paths.structural, def_name);
    let enum_namespace = namespace_with_struct_path(namespace, &current_struct_path);
    let required_set: HashSet<&str> = required.iter().map(|s| s.as_str()).collect();
    let aliases = known_enum_aliases();

    let fields: Vec<String> = properties
        .iter()
        .filter(|(field_name, field_prop)| {
            if is_deprecated_property(field_prop) {
                let field_path = extend_property_path(paths.provider, field_name);
                record_skipped_deprecated_write_only_path(schema, &field_path);
                return false;
            }
            if should_omit_property(
                &schema.type_name,
                CfnPropertyPath::new(Some(def_name), field_name),
            ) {
                return false;
            }
            true
        })
        .map(|(field_name, field_prop)| {
            let snake_name = field_name.to_snake_case();
            let field_path = extend_property_path(paths.provider, field_name);
            // Apply nested-field enum overlay (awscc#246). When the resource
            // schema does not annotate a struct field as an enum but we have
            // a curated value list in `resource_type_overrides()` keyed by
            // ("AWS::Service::Type", "DefName.FieldName"), promote it to a
            // synthetic StringEnum here. List fields (`array<string>`) keep
            // the list wrapping below; only the item type changes.
            let cfn_property_path = CfnPropertyPath::new(Some(def_name), field_name);
            let nested_key = cfn_property_path.to_string();
            let nested_override = resource_type_override(&schema.type_name, cfn_property_path);
            // A composite StringType override replaces the entire source
            // property, including any object children. Do not walk those
            // discarded children: read-only bookkeeping must describe the
            // AttributeType that is actually emitted.
            let (field_type, mut enum_info) = if let Some(override_type) =
                nested_override.and_then(TypeOverride::attribute_type_expression)
            {
                (override_type.to_string(), None)
            } else {
                    cfn_type_to_carina_type_with_enum_with_paths(
                        field_prop,
                        field_name,
                        schema,
                        namespace,
                        enums,
                        CfnTypeGenerationPaths {
                            structural: &current_struct_path,
                            provider: &field_path,
                            cfn_parent: Some(def_name),
                        },
                    )
            };
            // `EnumUnordered` is element-handled identically to `Enum`;
            // it differs only in forcing `unordered_list` below
            // (carina#3093).
            if let Some(TypeOverride::Enum(values) | TypeOverride::EnumUnordered(values)) =
                nested_override
            {
                enum_info = Some(enum_info_for_override(field_name, values));
            } else if nested_override
                .and_then(TypeOverride::attribute_type_expression)
                .is_some()
            {
                enum_info = None;
            }
            let field_force_unordered =
                matches!(nested_override, Some(TypeOverride::EnumUnordered(_)));
            // If enum detected in struct field and it's in the enums map, use shared string enum type.
            // Try composite key "DefName.FieldName" first (for definition-scoped enums),
            // then fall back to plain "FieldName" (for top-level property enums).
            // Check if the original field type was a List (array)
            let is_list_field =
                field_type.contains("::list(") || field_type.contains("::unordered_list(");
            let field_type = if let Some(expression) =
                nested_override.and_then(TypeOverride::attribute_type_expression)
            {
                expression.to_string()
            } else if matches!(nested_override, Some(TypeOverride::IntegerFromNumber)) {
                integer_from_number_type(field_prop, &schema.type_name, &nested_key)
            } else if let Some(local_enum_info) = enum_info {
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
            // Other nested override variants are caught by the trailing guard
            // so new variants cannot silently do nothing for struct fields.
            } else if let Some(other) = nested_override {
                panic!(
                    "nested override {:?} at {} uses TypeOverride variant {:?} that is not wired through generate_struct_type — add a branch here or move the entry to a top-level key",
                    nested_key,
                    schema.type_name,
                    other,
                );
            } else {
                field_type
            };
            let is_read_only = is_property_read_only(schema, field_prop, &field_path);
            let is_write_only = is_write_only_property_path(schema, &field_path);
            let is_required = required_set.contains(field_name.as_str()) && !is_read_only;

            let constructor = if is_read_only {
                "provider_populated"
            } else if is_required {
                "required"
            } else {
                "optional"
            };
            let mut field_code = format!(
                "StructField::{}({}, {})",
                constructor,
                rust_lit(&snake_name),
                field_type
            );
            if is_read_only {
                record_emitted_read_only_path(&field_path);
            }
            if is_write_only {
                field_code.push_str(".write_only()");
                record_emitted_write_only_path(&field_path);
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
                && !is_read_only
                && let Some(singular) = compute_block_name(&snake_name)
            {
                field_code.push_str(&format!(".with_block_name({})", rust_lit(&singular)));
            }

            field_code.push_str(".build()");

            field_code
        })
        .collect();

    let fields_str = fields.join(",\n                    ");
    let struct_type = format!(
        "AttributeType::struct_({}.to_string(), vec![{}])",
        rust_lit(def_name),
        fields_str
    );
    record_struct_read_only_use(def_name, paths.provider, emission_mark);
    record_struct_write_only_use(def_name, paths.provider, write_only_emission_mark);
    struct_type
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

/// Format a range display string for string length type names.
fn string_length_display(min: Option<u64>, max: Option<u64>) -> String {
    match (min, max) {
        (Some(min), Some(max)) => format!("{}..={}", min, max),
        (Some(min), None) => format!("{}..", min),
        (None, Some(max)) => format!("..={}", max),
        (None, None) => unreachable!("at least one bound must be present"),
    }
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

fn catalog_type_for_property(resource_type: &str, prop_name: &str) -> Option<AwsccType> {
    let dsl_resource = dsl_resource_name_from_type(resource_type).ok()?;
    let (service, resource) = resource_identity_parts(&dsl_resource)?;
    AwsccType::ALL.iter().copied().find(|awscc_type| {
        is_resource_identity(&awscc_type.identity(), service, resource, prop_name)
    })
}

fn int_enum_type(resource_type: &str, prop_name: &str, values: &[i64]) -> String {
    if let Some(awscc_type) = catalog_type_for_property(resource_type, prop_name)
        && let Some(expected) = awscc_type.allowed_int_values()
    {
        assert_eq!(
            values, expected,
            "CloudFormation integer-enum values for {resource_type}.{prop_name} drifted from the AwsccType catalog"
        );
        return awscc_type.constructor_path().to_string();
    }

    let members = values
        .iter()
        .map(|value| {
            format!("AttributeType::refined_int(None, Some((Some({value}), Some({value}))))")
        })
        .collect::<Vec<_>>()
        .join(", ");
    format!("AttributeType::union(vec![{members}])")
}

fn refined_float_type(identity: &str, range: &str) -> String {
    format!("AttributeType::refined_float({identity}, {range})")
}

fn refined_list_type(element_type: &str, ordered: bool, length: &str) -> String {
    format!("AttributeType::refined_list({element_type}, {ordered}, {length})")
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

fn integer_from_number_bounds(
    prop: &CfnProperty,
    type_name: &str,
    property_name: &str,
) -> (Option<i64>, Option<i64>) {
    assert_eq!(
        prop.prop_type.as_ref().and_then(TypeValue::as_str),
        Some("number"),
        "IntegerFromNumber override for {type_name}.{property_name} requires a CFN number property"
    );
    let bounds = (prop.minimum, prop.maximum);
    assert!(
        bounds.0.is_some() || bounds.1.is_some(),
        "IntegerFromNumber override for {type_name}.{property_name} requires a CFN minimum or maximum"
    );
    bounds
}

fn integer_from_number_type(prop: &CfnProperty, type_name: &str, property_name: &str) -> String {
    let (min, max) = integer_from_number_bounds(prop, type_name, property_name);
    refined_int_type("None", &emit_int_range_option(min, max))
}

fn integer_from_number_display(prop: &CfnProperty, type_name: &str, property_name: &str) -> String {
    let (min, max) = integer_from_number_bounds(prop, type_name, property_name);
    format!("Int({})", range_display_string(min, max))
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

/// Resource-keyed groups of updatable CloudFormation properties where exactly
/// one member may be present in CloudControl's update model.
fn mutually_exclusive_property_groups()
-> &'static HashMap<&'static str, &'static [&'static [&'static str]]> {
    const EC2_ROUTE_TARGETS: &[&str] = &[
        "CarrierGatewayId",
        "CoreNetworkArn",
        "EgressOnlyInternetGatewayId",
        "GatewayId",
        "InstanceId",
        "LocalGatewayId",
        "NatGatewayId",
        "NetworkInterfaceId",
        "TransitGatewayId",
        "VpcEndpointId",
        "VpcPeeringConnectionId",
    ];
    const EC2_ROUTE_GROUPS: &[&[&str]] = &[EC2_ROUTE_TARGETS];

    static GROUPS: LazyLock<HashMap<&'static str, &'static [&'static [&'static str]]>> =
        LazyLock::new(|| {
            // Measured with carina-test-009 in ap-northeast-1 on 2026-10-29:
            // CloudControl's Route read mirrors GatewayId into VpcEndpointId, and
            // an update may remove an absent JSON Patch path successfully.
            HashMap::from([("AWS::EC2::Route", EC2_ROUTE_GROUPS)])
        });
    &GROUPS
}

/// Unified resource-specific property type overrides.
/// Maps (CloudFormation type name, property name) to a TypeOverride.
/// Use this when a property needs resource-specific type treatment that differs
/// from global overrides or pattern-based inference.
///
/// Two-layer system: global overrides (known_*_overrides) apply by property name,
/// resource-specific overrides here take precedence.
#[derive(Default)]
struct ResourceOverridesForType {
    top_level: HashMap<&'static str, TypeOverride>,
    nested: HashMap<&'static str, HashMap<&'static str, TypeOverride>>,
}

#[derive(Default)]
struct ResourceTypeOverrides {
    #[cfg(test)]
    flat: HashMap<(&'static str, &'static str), TypeOverride>,
    by_type: HashMap<&'static str, ResourceOverridesForType>,
}

impl ResourceTypeOverrides {
    fn insert(
        &mut self,
        (cfn_type, property_path): (&'static str, &'static str),
        override_type: TypeOverride,
    ) {
        #[cfg(test)]
        let flat_override = override_type.clone();
        let resource = self.by_type.entry(cfn_type).or_default();
        let previous = if let Some((parent, property)) = property_path.split_once('.') {
            resource
                .nested
                .entry(parent)
                .or_default()
                .insert(property, override_type)
        } else {
            resource.top_level.insert(property_path, override_type)
        };
        assert!(
            previous.is_none(),
            "duplicate resource type override {cfn_type}.{property_path}"
        );
        #[cfg(test)]
        assert!(
            self.flat
                .insert((cfn_type, property_path), flat_override)
                .is_none(),
            "duplicate flat resource type override {cfn_type}.{property_path}"
        );
    }

    fn get_path(
        &self,
        cfn_type: &str,
        property_path: CfnPropertyPath<'_>,
    ) -> Option<&TypeOverride> {
        let resource = self.by_type.get(cfn_type)?;
        match property_path.parent {
            Some(parent) => resource.nested.get(parent)?.get(property_path.property),
            None => resource.top_level.get(property_path.property),
        }
    }

    #[cfg(test)]
    fn get(&self, (cfn_type, property_path): &(&str, &str)) -> Option<&TypeOverride> {
        let property_path = property_path.split_once('.').map_or_else(
            || CfnPropertyPath::new(None, property_path),
            |(parent, property)| CfnPropertyPath::new(Some(parent), property),
        );
        self.get_path(cfn_type, property_path)
    }

    #[cfg(test)]
    fn iter(&self) -> impl Iterator<Item = (&(&'static str, &'static str), &TypeOverride)> {
        self.flat.iter()
    }

    #[cfg(test)]
    fn keys(&self) -> impl Iterator<Item = &(&'static str, &'static str)> {
        self.flat.keys()
    }
}

fn resource_type_overrides() -> &'static ResourceTypeOverrides {
    static OVERRIDES: LazyLock<ResourceTypeOverrides> = LazyLock::new(|| {
        let mut m = ResourceTypeOverrides::default();

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
        // Hosted-zone ID producers must carry the shared Route 53 identity so
        // they can flow into Route53 alias-target sinks. The cached CFN schemas
        // expose both read-only attributes as unrefined strings. CloudControl
        // HostedZone create/read in us-east-1 on 2026-10-04 returned the same
        // unprefixed Z... ID from the create response and read-back state.
        m.insert(
            (
                "AWS::ElasticLoadBalancingV2::LoadBalancer",
                "CanonicalHostedZoneID",
            ),
            TypeOverride::StringType("carina_aws_types::route53_hosted_zone_id()"),
        );
        m.insert(
            ("AWS::Route53::HostedZone", "Id"),
            TypeOverride::StringType("carina_aws_types::route53_hosted_zone_id()"),
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

        // The CloudFormation schema publishes these two BudgetData fields
        // only as bare objects. The Budgets API defines their map values:
        // https://docs.aws.amazon.com/aws-cost-management/latest/APIReference/API_budgets_Budget.html
        m.insert(
            ("AWS::Budgets::Budget", "BudgetData.CostFilters"),
            TypeOverride::AttributeType {
                expression: "AttributeType::map(AttributeType::list(AttributeType::string()))",
                display: "`Map<String, List<String>>`",
                block_name: None,
                default: None,
            },
        );
        m.insert(
            ("AWS::Budgets::Budget", "BudgetData.PlannedBudgetLimits"),
            TypeOverride::AttributeType {
                expression: "AttributeType::map(AttributeType::ref_(\"Spend\".to_string()))",
                display: "`Map<String, Spend>`",
                block_name: None,
                default: None,
            },
        );

        // Cloud Control create/get in both directions with carina-test-008 in
        // ap-northeast-1 on 2026-10-31 returned these policy values as native
        // JSON objects/arrays, never JSON strings. Keep each override at its
        // full CFN property path so unrelated name suffixes cannot match it.
        m.insert(
            ("AWS::SNS::Subscription", "RedrivePolicy"),
            TypeOverride::AttributeType {
                expression: "crate::schemas::awscc_types::sns_subscription_redrive_policy()",
                display: "`Struct(SubscriptionRedrivePolicy)`",
                block_name: None,
                default: None,
            },
        );
        m.insert(
            ("AWS::SNS::Subscription", "DeliveryPolicy"),
            TypeOverride::AttributeType {
                expression: "crate::schemas::awscc_types::sns_subscription_delivery_policy()",
                display: "`Struct(SubscriptionDeliveryPolicy)`",
                block_name: None,
                default: None,
            },
        );
        m.insert(
            ("AWS::SNS::Subscription", "ReplayPolicy"),
            TypeOverride::AttributeType {
                expression: "crate::schemas::awscc_types::sns_subscription_replay_policy()",
                display: "`Struct(SubscriptionReplayPolicy)`",
                block_name: None,
                default: None,
            },
        );
        m.insert(
            ("AWS::SNS::Topic", "ArchivePolicy"),
            TypeOverride::AttributeType {
                expression: "crate::schemas::awscc_types::sns_topic_archive_policy()",
                display: "`Struct(TopicArchivePolicy)`",
                block_name: None,
                default: None,
            },
        );
        m.insert(
            ("AWS::Logs::LogGroup", "FieldIndexPolicies"),
            TypeOverride::AttributeType {
                expression: "crate::schemas::awscc_types::logs_field_index_policies()",
                display: "`List<Struct(FieldIndexPolicy)>`",
                block_name: Some("field_index_policy"),
                default: Some(AttributeTypeOverrideDefault {
                    value_expression: "Value::Concrete(ConcreteValue::List(vec![]))",
                    markdown: "[]",
                }),
            },
        );
        m.insert(
            ("AWS::Logs::LogGroup", "DataProtectionPolicy"),
            TypeOverride::AttributeType {
                expression: "crate::schemas::awscc_types::logs_data_protection_policy()",
                display: "`Struct(LogGroupDataProtectionPolicy)`",
                block_name: None,
                default: None,
            },
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

        // ELBv2 fixed-response status codes are constrained to ^(2|4|5)\d{2}$.
        // The CFN schema leaves them as plain strings; CloudControl only catches
        // bad values at apply-time. http_response_status_code is a carina-core
        // built-in (TypeIdentity::bare("HttpResponseStatusCode")).
        m.insert(
            (
                "AWS::ElasticLoadBalancingV2::Listener",
                "FixedResponseConfig.StatusCode",
            ),
            TypeOverride::StringType("carina_core::schema::types::http_response_status_code()"),
        );

        // === Enum overrides ===

        // VPN Gateway Type only accepts "ipsec.1"
        m.insert(
            ("AWS::EC2::VPNGateway", "Type"),
            TypeOverride::Enum(vec!["ipsec.1"]),
        );
        // EC2 VPC InstanceTenancy is a pre-existing generated enum. The
        // cached description lists `default` and `dedicated` as bullet
        // leaders, but only mentions `host` inside a bullet body. Keep the
        // heuristic scoped to leading value-list bullets and pin the full
        // provider value set here.
        m.insert(
            ("AWS::EC2::VPC", "InstanceTenancy"),
            TypeOverride::Enum(vec!["default", "dedicated", "host"]),
        );

        // ELBv2 TargetGroup TargetType / Protocol (awscc#357). The CFN
        // descriptions carry no value list, so the description-scraping
        // heuristic in `extract_enum_from_description` returns nothing
        // and the type would otherwise stay a free-form string.
        //
        // Scope these by resource type — `Protocol` is a field name
        // shared with `Listener.Protocol` and the listener uses a
        // different value union (it adds `QUIC`/`TCP_QUIC`), so we must
        // NOT let a field-name-only override (`known_enum_overrides`)
        // leak this list across resources.
        m.insert(
            ("AWS::ElasticLoadBalancingV2::TargetGroup", "TargetType"),
            TypeOverride::Enum(vec!["instance", "ip", "lambda", "alb"]),
        );
        m.insert(
            ("AWS::ElasticLoadBalancingV2::TargetGroup", "Protocol"),
            TypeOverride::Enum(vec![
                "HTTP", "HTTPS", "TCP", "TLS", "UDP", "TCP_UDP", "GENEVE",
            ]),
        );

        // ELBv2 LoadBalancer closed string values (awscc#365). The
        // CFN descriptions either have no value-list sentence (`Type`,
        // `Scheme`) or contain unrelated backticked cross-references
        // (`EnablePrefixForIpv6SourceNat`). Keep these resource-scoped:
        // `Type` and `Scheme` are generic field names and must not leak
        // into siblings like VPNGateway.Type.
        m.insert(
            ("AWS::ElasticLoadBalancingV2::LoadBalancer", "Type"),
            TypeOverride::Enum(vec!["application", "network", "gateway"]),
        );
        m.insert(
            ("AWS::ElasticLoadBalancingV2::LoadBalancer", "Scheme"),
            TypeOverride::Enum(vec!["internet-facing", "internal"]),
        );
        m.insert(
            (
                "AWS::ElasticLoadBalancingV2::LoadBalancer",
                "EnablePrefixForIpv6SourceNat",
            ),
            TypeOverride::Enum(vec!["on", "off"]),
        );
        m.insert(
            (
                "AWS::ElasticLoadBalancingV2::LoadBalancer",
                "EnforceSecurityGroupInboundRulesOnPrivateLinkTraffic",
            ),
            TypeOverride::Enum(vec!["on", "off"]),
        );
        // LoadBalancerAttribute.Key intentionally has no enum overlay.
        // Pre-365 Strategy 1 extracted attribute VALUES (`true`, `false`,
        // modes, etc.) as the Key enum because the real attribute keys
        // are bullet leaders containing dots and were filtered out by
        // `looks_like_enum_value`. Later value-list sentences in the
        // bullet bodies describe per-key values, not legal Key names, so
        // force this field back to plain String.
        m.insert(
            (
                "AWS::ElasticLoadBalancingV2::LoadBalancer",
                "LoadBalancerAttribute.Key",
            ),
            TypeOverride::StringType("AttributeType::string()"),
        );

        // ELBv2 Listener Action.Type (awscc#359). The CFN description
        // is literally "The type of action." with no value list, but
        // the sibling fields' descriptions enumerate the closed set
        // ("Specify only when ``Type`` is ``forward``" etc.). Real AWS
        // values: forward, authenticate-oidc, authenticate-cognito,
        // redirect, fixed-response, jwt-validation.
        //
        // Resource-scoped for the same reason as TargetGroup.Protocol:
        // `Type` is a generic field name (it also shows up on
        // VPNGateway with a different value set), so a field-name-only
        // entry would silently clobber sibling resources.
        m.insert(
            ("AWS::ElasticLoadBalancingV2::Listener", "Action.Type"),
            TypeOverride::Enum(vec![
                "forward",
                "authenticate-oidc",
                "authenticate-cognito",
                "redirect",
                "fixed-response",
                "jwt-validation",
            ]),
        );

        // ELBv2 Listener RedirectConfig placeholders (awscc#363,
        // carina#3494). Protocol, Host, Port, Path, and Query are
        // sibling string types in carina-core so each field accepts
        // its documented `#{...}` placeholder without widening the
        // others. StatusCode remains a closed enum below because it
        // has no placeholder form.
        m.insert(
            (
                "AWS::ElasticLoadBalancingV2::Listener",
                "RedirectConfig.Protocol",
            ),
            TypeOverride::StringType("carina_core::schema::types::redirect_protocol()"),
        );
        m.insert(
            (
                "AWS::ElasticLoadBalancingV2::Listener",
                "RedirectConfig.Host",
            ),
            TypeOverride::StringType("carina_core::schema::types::redirect_host()"),
        );
        m.insert(
            (
                "AWS::ElasticLoadBalancingV2::Listener",
                "RedirectConfig.Port",
            ),
            TypeOverride::StringType("carina_core::schema::types::redirect_port()"),
        );
        m.insert(
            (
                "AWS::ElasticLoadBalancingV2::Listener",
                "RedirectConfig.Path",
            ),
            TypeOverride::StringType("carina_core::schema::types::redirect_path()"),
        );
        m.insert(
            (
                "AWS::ElasticLoadBalancingV2::Listener",
                "RedirectConfig.Query",
            ),
            TypeOverride::StringType("carina_core::schema::types::redirect_query()"),
        );
        // ELBv2 Listener RedirectConfig.StatusCode (awscc#360). The
        // description ("either permanent (HTTP 301) or temporary
        // (HTTP 302)") names the values in prose but the CFN string
        // form uses the underscored variants `HTTP_301` / `HTTP_302`.
        // No existing matcher recovers this mapping; pin the two
        // canonical values.
        m.insert(
            (
                "AWS::ElasticLoadBalancingV2::Listener",
                "RedirectConfig.StatusCode",
            ),
            TypeOverride::Enum(vec!["HTTP_301", "HTTP_302"]),
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
        // ResponseHeadersPolicy CORS methods are also an order-insensitive
        // set whose valid values appear only in the CFN description.
        m.insert(
            (
                "AWS::CloudFront::ResponseHeadersPolicy",
                "AccessControlAllowMethods.Items",
            ),
            TypeOverride::EnumUnordered(vec![
                "GET", "DELETE", "HEAD", "OPTIONS", "PATCH", "POST", "PUT", "ALL",
            ]),
        );
        // The CFN schema describes Runtime as a plain string, while the
        // CloudFront Functions API exposes this closed runtime set.
        m.insert(
            ("AWS::CloudFront::Function", "FunctionConfig.Runtime"),
            TypeOverride::Enum(vec!["cloudfront-js-1.0", "cloudfront-js-2.0"]),
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

        // Cloud Control's Schedule identifier omits GroupName and reads
        // Name in the default group. The catalog validator records the
        // measured behavior and rejects non-default or unproven groups.
        m.insert(
            ("AWS::Scheduler::Schedule", "GroupName"),
            TypeOverride::StringType("crate::schemas::catalog::scheduler_schedule_group_name()"),
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
        // CloudFormation publishes these Scheduler API integers as `number`.
        // Their ranges come from each property's CFN minimum/maximum.
        for path in [
            "FlexibleTimeWindow.MaximumWindowInMinutes",
            "RetryPolicy.MaximumEventAgeInSeconds",
            "RetryPolicy.MaximumRetryAttempts",
            "EcsParameters.TaskCount",
            "CapacityProviderStrategyItem.Base",
            "CapacityProviderStrategyItem.Weight",
        ] {
            m.insert(
                ("AWS::Scheduler::Schedule", path),
                TypeOverride::IntegerFromNumber,
            );
        }

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
                1, 3, 5, 7, 14, 30, 60, 90, 120, 150, 180, 365, 400, 545, 731, 1096, 1827, 2192,
                2557, 2922, 3288, 3653,
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
        resource_type_override(resource_type, CfnPropertyPath::new(None, prop_name))
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

/// Attributes that do **not** exist in the CloudFormation schema but that
/// the provider synthesizes on the read path so downstream `.crn`
/// references can resolve. Each entry expands into an extra
/// `AttributeSchema::provider_populated(...)` block in the generated schema
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
        resource_type_override(resource_type, CfnPropertyPath::new(None, prop_name))
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
fn override_forces_unordered(type_name: &str, property_path: CfnPropertyPath<'_>) -> bool {
    matches!(
        resource_type_override(type_name, property_path),
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
    let property_path = [prop_name.to_string()];
    cfn_type_to_carina_type_with_enum_with_paths(
        prop,
        prop_name,
        schema,
        namespace,
        enums,
        CfnTypeGenerationPaths {
            structural: &[],
            provider: &property_path,
            cfn_parent: None,
        },
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

#[derive(Clone, Copy)]
struct CfnTypeGenerationPaths<'a> {
    structural: &'a [String],
    provider: &'a [String],
    cfn_parent: Option<&'a str>,
}

fn generate_object_map_type(
    value: &CfnProperty,
    prop_name: &str,
    schema: &CfnSchema,
    namespace: &str,
    enums: &BTreeMap<String, EnumInfo>,
    paths: CfnTypeGenerationPaths<'_>,
) -> (String, Option<EnumInfo>) {
    let property_path = CfnPropertyPath::new(paths.cfn_parent, prop_name);
    match object_map_value_or_panic(schema, property_path, value) {
        ObjectMapValueKind::DirectString => (
            "AttributeType::map(AttributeType::string())".to_string(),
            None,
        ),
        ObjectMapValueKind::Struct => {
            let value_provider_path = extend_property_path(paths.provider, "*");
            let (value_type, enum_info) = cfn_type_to_carina_type_with_enum_with_paths(
                value,
                prop_name,
                schema,
                namespace,
                enums,
                CfnTypeGenerationPaths {
                    provider: &value_provider_path,
                    ..paths
                },
            );
            assert!(
                enum_info.is_none(),
                "typed object-map values must resolve to structs"
            );
            (format!("AttributeType::map({value_type})"), None)
        }
    }
}

fn cfn_type_to_carina_type_with_enum_with_paths(
    prop: &CfnProperty,
    prop_name: &str,
    schema: &CfnSchema,
    namespace: &str,
    enums: &BTreeMap<String, EnumInfo>,
    paths: CfnTypeGenerationPaths<'_>,
) -> (String, Option<EnumInfo>) {
    let struct_path = paths.structural;
    let property_path = paths.provider;
    let cfn_parent = paths.cfn_parent;
    let cfn_property_path = CfnPropertyPath::new(cfn_parent, prop_name);
    if object_string_union_is_iam_policy_document(prop, &schema.type_name, cfn_property_path) {
        return ("carina_aws_types::iam_policy_document()".to_string(), None);
    }

    if property_is_bare_object(prop, schema) {
        return (
            bare_object_attribute_type(&schema.type_name, cfn_property_path, prop_name),
            None,
        );
    }

    // The declared tag property is special when its generated DSL name is
    // `tags`: it is a Map in Carina (Terraform-style).
    if is_dsl_tags_property(schema, prop_name) {
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
            return cfn_type_to_carina_type_with_enum_with_paths(
                array_variant,
                prop_name,
                schema,
                namespace,
                enums,
                paths,
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
        if is_tag_map_ref(schema, ref_path) {
            return ("carina_aws_types::tags_type()".to_string(), None);
        }

        if let Some(def_name) = ref_def_name(ref_path)
            && IN_PROGRESS_DEFS.with(|s| s.borrow().contains(def_name))
        {
            mark_back_reference(def_name);
            return (
                format!("AttributeType::ref_({}.to_string())", rust_lit(def_name)),
                None,
            );
        }

        let definition_already_emitted = ref_def_name(ref_path)
            .is_some_and(|def_name| EMITTED_DEFS.with(|s| s.borrow().contains_key(def_name)));

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
                        StructGenerationPaths {
                            structural: struct_path,
                            provider: property_path,
                        },
                    );
                    IN_PROGRESS_DEFS.with(|s| {
                        s.borrow_mut().remove(def_name);
                    });
                    if definition_already_emitted {
                        return (
                            format!("AttributeType::ref_({}.to_string())", rust_lit(def_name)),
                            None,
                        );
                    }
                    EMITTED_DEFS.with(|s| {
                        s.borrow_mut().insert(def_name.to_string(), body.clone());
                    });
                    return (body, None);
                }
                ResolvedDef::ObjectMap { value } => {
                    return generate_object_map_type(
                        value, prop_name, schema, namespace, enums, paths,
                    );
                }
                ResolvedDef::OneOf { variants } => {
                    let mut merged_props = BTreeMap::new();
                    for variant in variants {
                        if let Some(props) = &variant.properties {
                            for (k, v) in props {
                                if is_deprecated_property(v) {
                                    let field_path = extend_property_path(property_path, k);
                                    record_skipped_deprecated_write_only_path(schema, &field_path);
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
                            StructGenerationPaths {
                                structural: struct_path,
                                provider: property_path,
                            },
                        );
                        IN_PROGRESS_DEFS.with(|s| {
                            s.borrow_mut().remove(def_name);
                        });
                        if definition_already_emitted {
                            return (
                                format!("AttributeType::ref_({}.to_string())", rust_lit(def_name)),
                                None,
                            );
                        }
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
                    let item_path = extend_property_path(property_path, "*");
                    let (item_type, item_enum) = cfn_type_to_carina_type_with_enum_with_paths(
                        items,
                        prop_name,
                        schema,
                        namespace,
                        enums,
                        CfnTypeGenerationPaths {
                            provider: &item_path,
                            ..paths
                        },
                    );
                    let effective_item_type = if item_enum.is_some() {
                        "AttributeType::string()".to_string()
                    } else {
                        item_type
                    };
                    let list_ctor = list_constructor(
                        prop.insertion_order,
                        override_forces_unordered(&schema.type_name, cfn_property_path),
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
                        return cfn_type_to_carina_type_with_enum_with_paths(
                            &scalar_prop,
                            prop_name,
                            schema,
                            namespace,
                            enums,
                            paths,
                        );
                    }
                }
                ResolvedDef::Opaque => {
                    if resolve_ref(schema, ref_path)
                        .is_some_and(|definition| definition.def_type.as_deref() == Some("object"))
                    {
                        panic!(
                            "unsupported object definition {}.{} referenced by {ref_path}; expected a struct, direct-string map, tag map, typed map of struct, or classified bare object; see CODEGEN.md#classifying-bare-objects",
                            schema.type_name, cfn_property_path
                        );
                    }
                }
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
            return (int_enum_type(&schema.type_name, prop_name, &values), None);
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
                resource_type_override(&schema.type_name, cfn_property_path)
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
            if is_iam_policy_document_property(&schema.type_name, cfn_property_path) {
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
            let res_override = resource_type_override(&schema.type_name, cfn_property_path);
            if let Some(TypeOverride::IntRange(min, max)) = res_override {
                return (
                    refined_int_type("None", &emit_int_range_option(Some(*min), Some(*max))),
                    None,
                );
            }
            if let Some(TypeOverride::IntEnum(values)) = res_override {
                return (int_enum_type(&schema.type_name, prop_name, values), None);
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
            let res_override = resource_type_override(&schema.type_name, cfn_property_path);
            if let Some(TypeOverride::IntRange(min, max)) = res_override {
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
                override_forces_unordered(&schema.type_name, cfn_property_path),
            );
            let (list_type, item_enum) = if let Some(items) = &prop.items {
                let item_path = extend_property_path(property_path, "*");
                // The list-of-struct inlining path bypasses the normal `$ref`
                // branch above. Apply the same in-progress guard here so both
                // direct self-references and multi-definition list cycles
                // terminate with a schema Ref.
                if let Some(ref_path) = &items.ref_path
                    && let Some(def_name) = ref_def_name(ref_path)
                    && IN_PROGRESS_DEFS.with(|defs| defs.borrow().contains(def_name))
                {
                    mark_back_reference(def_name);
                    (
                        format!(
                            "{}(AttributeType::ref_({}.to_string()))",
                            list_ctor,
                            rust_lit(def_name)
                        ),
                        None,
                    )
                // Struct-shaped $ref items are inlined as a list-of-struct;
                // every other item shape (scalar/enum/array/string/opaque)
                // falls through to the normal recursion below.
                } else if let Some(ref_path) = &items.ref_path
                    && !is_tag_map_ref(schema, ref_path)
                    && let Some(ResolvedDef::Struct {
                        properties,
                        required,
                    }) = resolve_ref_classified(schema, ref_path)
                {
                    let def_name = ref_def_name(ref_path).unwrap_or(prop_name);
                    IN_PROGRESS_DEFS.with(|defs| {
                        defs.borrow_mut().insert(def_name.to_string());
                    });
                    let struct_type = generate_struct_type(
                        def_name,
                        properties,
                        required,
                        schema,
                        namespace,
                        enums,
                        StructGenerationPaths {
                            structural: struct_path,
                            provider: &item_path,
                        },
                    );
                    IN_PROGRESS_DEFS.with(|defs| {
                        defs.borrow_mut().remove(def_name);
                    });
                    if was_back_referenced(def_name) {
                        EMITTED_DEFS.with(|defs| {
                            defs.borrow_mut()
                                .insert(def_name.to_string(), struct_type.clone());
                        });
                    }
                    (format!("{}({})", list_ctor, struct_type), None)
                } else {
                    let (item_type, item_enum) = cfn_type_to_carina_type_with_enum_with_paths(
                        items,
                        prop_name,
                        schema,
                        namespace,
                        enums,
                        CfnTypeGenerationPaths {
                            provider: &item_path,
                            ..paths
                        },
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
                let ordered = list_ctor == "AttributeType::list";
                let inner = list_type
                    .strip_prefix(&format!("{list_ctor}("))
                    .and_then(|s| s.strip_suffix(')'))
                    .unwrap_or("AttributeType::string()");
                let length = emit_length_option(
                    prop.min_items.and_then(|v| u64::try_from(v).ok()),
                    prop.max_items.and_then(|v| u64::try_from(v).ok()),
                );
                (refined_list_type(inner, ordered, &length), item_enum)
            } else {
                (list_type, item_enum)
            }
        }
        Some("object") => {
            // Check known string type overrides first, mirroring the string branch
            // for CFN union types whose first type is object.
            let res_override = resource_type_override(&schema.type_name, cfn_property_path);
            if let Some(override_type) =
                res_override.and_then(TypeOverride::attribute_type_expression)
            {
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
                        StructGenerationPaths {
                            structural: struct_path,
                            provider: property_path,
                        },
                    ),
                    None,
                );
            }
            if let Some(value) = direct_object_map_value(prop) {
                return generate_object_map_type(value, prop_name, schema, namespace, enums, paths);
            }
            panic!(
                "unsupported object property {}.{}; expected a struct, direct-string map, tag map, typed map of struct, or classified bare object; see CODEGEN.md#classifying-bare-objects",
                schema.type_name, cfn_property_path
            );
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
                "Value::Concrete(ConcreteValue::String(\"{}\".to_string().into()))", // rust-lit-guard: allow (already escaped JSON default)
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

    const EXTRACT_ENUM_SNAPSHOT_PRE_365: &str = "tests/fixtures/extract_enum_snapshot_pre_365.json";

    fn cfn_schema_for_codegen_tests(type_name: &str) -> CfnSchema {
        let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../cfn-schema-cache")
            .join(format!("{}.json", type_name.replace("::", "__")));
        let raw = std::fs::read_to_string(path).expect("cached CFN schema");
        serde_json::from_str(&raw).expect("cached CFN schema parses")
    }

    fn cached_schema_paths() -> Vec<std::path::PathBuf> {
        let cache_dir =
            std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../cfn-schema-cache");
        let mut paths: Vec<std::path::PathBuf> = std::fs::read_dir(cache_dir)
            .expect("read cfn-schema-cache")
            .map(|entry| entry.expect("schema cache entry").path())
            .filter(|path| {
                path.file_name()
                    .and_then(|name| name.to_str())
                    .is_some_and(|name| name.starts_with("AWS__") && name.ends_with(".json"))
            })
            .collect();
        paths.sort();
        paths
    }

    /// Generate an intentionally partial schema fixture under a declared
    /// resource identity whose replacement class requires no name property.
    /// Use only when the assertion is independent of the emitted identity.
    fn generate_synthetic_schema_code(schema: &CfnSchema) -> Result<String> {
        generate_schema_code(schema, "AWS::EC2::Route")
    }

    #[test]
    fn test_generate_schema_code_uses_helper_shape_for_small_schema() {
        let mut properties = BTreeMap::new();
        properties.insert(
            "Name".to_string(),
            CfnProperty {
                prop_type: Some(TypeValue::Single("string".to_string())),
                ..Default::default()
            },
        );
        let schema = CfnSchema {
            type_name: "AWS::Test::Small".to_string(),
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

        let generated = generate_synthetic_schema_code(&schema).unwrap();

        assert!(generated.contains("#[rustfmt::skip]\npub fn ec2_route_config()"));
        assert!(
            generated
                .contains("#[inline(never)]\n    fn build_name_attribute() -> AttributeSchema")
        );
        assert!(generated.contains("schema = schema.attribute(build_name_attribute());"));
        assert!(!generated.contains("schema: ResourceSchema::new("));
    }

    fn has_resource_enum_override(type_name: &str, property_path: &str) -> bool {
        resource_type_overrides()
            .iter()
            .any(|(&(resource, property), override_)| {
                resource == type_name
                    && property == property_path
                    && matches!(
                        override_,
                        &TypeOverride::Enum(_) | &TypeOverride::EnumUnordered(_)
                    )
            })
    }

    fn has_known_enum_override(property_path: &str) -> bool {
        let field_name = property_path.rsplit('.').next().unwrap_or(property_path);
        known_enum_overrides().contains_key(field_name)
    }

    fn has_curated_enum_override(type_name: &str, property_path: &str) -> bool {
        has_resource_enum_override(type_name, property_path)
            || has_known_enum_override(property_path)
    }

    fn snapshot_key_has_curated_enum_override(key: &str) -> bool {
        let Some((type_name, property_path)) = key.rsplit_once("::") else {
            return false;
        };
        has_curated_enum_override(type_name, property_path)
    }

    fn enum_extraction_snapshot() -> BTreeMap<String, Vec<String>> {
        let mut snapshot = BTreeMap::new();

        for path in cached_schema_paths() {
            let raw = std::fs::read_to_string(&path).expect("cached CFN schema");
            let schema: CfnSchema = serde_json::from_str(&raw).expect("cached CFN schema parses");

            for (property_name, property) in &schema.properties {
                if property.enum_values.is_none()
                    && is_string_property(&schema, property)
                    && !has_curated_enum_override(&schema.type_name, property_name)
                    && let Some(description) = &property.description
                    && let Some(values) = extract_enum_from_description(description)
                {
                    snapshot.insert(format!("{}::{}", schema.type_name, property_name), values);
                }
            }

            if let Some(definitions) = &schema.definitions {
                for (definition_name, definition) in definitions {
                    let Some(properties) = &definition.properties else {
                        continue;
                    };
                    for (property_name, property) in properties {
                        let property_path = format!("{definition_name}.{property_name}");
                        if property.enum_values.is_none()
                            && is_string_property(&schema, property)
                            && !has_curated_enum_override(&schema.type_name, &property_path)
                            && let Some(description) = &property.description
                            && let Some(values) = extract_enum_from_description(description)
                        {
                            snapshot
                                .insert(format!("{}::{}", schema.type_name, property_path), values);
                        }
                    }
                }
            }
        }

        snapshot
    }

    #[test]
    fn test_extract_enum_description_cache_snapshot_has_no_unintended_retractions() {
        let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join(EXTRACT_ENUM_SNAPSHOT_PRE_365);
        let raw = std::fs::read_to_string(path).expect("pre-365 enum extraction snapshot fixture");
        let old_snapshot: BTreeMap<String, Vec<String>> =
            serde_json::from_str(&raw).expect("pre-365 enum extraction snapshot parses");
        let new_snapshot = enum_extraction_snapshot();
        let intentional_value_changes: BTreeMap<&str, &[&str]> = [
            (
                // Old extraction picked up `origin-request`/`origin-response`
                // from the "You cannot use origin-facing event types ..."
                // negation clause — those are not legal values for this field.
                // Strategy 1 deletion fixes the false positive.
                "AWS::CloudFront::Distribution::FunctionAssociation.EventType",
                &["origin-request", "origin-response"][..],
            ),
            (
                // Pre-365 Strategy 1 extracted `dualstack` from the
                // cross-reference sentence "The IP address type must be
                // ``dualstack``."; the real canonical values are `on`/`off`,
                // supplied by the curated overlay. Both `dualstack`
                // (cross-reference false positive) and `off` (default-value
                // sentence, not a value list) were emitted as the field's enum
                // pre-PR. Both lost.
                "AWS::ElasticLoadBalancingV2::LoadBalancer::EnablePrefixForIpv6SourceNat",
                &["dualstack", "off"][..],
            ),
            (
                // Pre-365 Strategy 1 extracted attribute VALUES
                // (true/false/monitor/defensive/append/...) as the Key enum
                // because real Keys all contain `.` and were filtered out by
                // looks_like_enum_value. The field's closed set is undecidable
                // from the description (key namespace is open-ended), so we
                // revert to plain String. See PR body.
                "AWS::ElasticLoadBalancingV2::LoadBalancer::LoadBalancerAttribute.Key",
                &[
                    "true",
                    "false",
                    "monitor",
                    "defensive",
                    "strictest",
                    "x-amzn-tls-version",
                    "x-amzn-tls-cipher-suite",
                    "append",
                    "preserve",
                    "remove",
                    "availability_zone_affinity",
                    "partial_availability_zone_affinity",
                    "any_availability_zone",
                ][..],
            ),
        ]
        .into_iter()
        .collect();

        let mut retractions = Vec::new();
        for (key, old_values) in &old_snapshot {
            let allow_missing = intentional_value_changes
                .get(key.as_str())
                .copied()
                .unwrap_or(&[]);
            if allow_missing.is_empty() && snapshot_key_has_curated_enum_override(key) {
                continue;
            }
            match new_snapshot.get(key) {
                None => {
                    let unexplained: Vec<&String> = old_values
                        .iter()
                        .filter(|v| !allow_missing.contains(&v.as_str()))
                        .collect();
                    if !unexplained.is_empty() {
                        retractions.push(format!(
                            "{}: extraction lost entirely; unexplained missing values {:?}",
                            key, unexplained
                        ));
                    }
                }
                Some(new_values) => {
                    let missing: Vec<&String> = old_values
                        .iter()
                        .filter(|v| !new_values.contains(v) && !allow_missing.contains(&v.as_str()))
                        .collect();
                    if !missing.is_empty() {
                        retractions.push(format!(
                            "{}: lost values {:?} (had {:?}, now {:?})",
                            key, missing, old_values, new_values
                        ));
                    }
                }
            }
        }

        assert!(
            retractions.is_empty(),
            "enum heuristic must not retract previously extracted fields without an explicit allowlist: {retractions:?}"
        );
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
            r#""{}".to_string()"#,                    // rust-lit-guard: allow
            r#"string_enum("{}"#,                     // rust-lit-guard: allow
            r#"struct_("{}"#,                         // rust-lit-guard: allow
            r#"ref_("{}"#,                            // rust-lit-guard: allow
            r#"StructField::required("{}"#,           // rust-lit-guard: allow
            r#"StructField::optional("{}"#,           // rust-lit-guard: allow
            r#"StructField::provider_populated("{}"#, // rust-lit-guard: allow
            r#"with_provider_name("{}"#,              // rust-lit-guard: allow
            r#"with_block_name("{}"#,                 // rust-lit-guard: allow
            r#"with_description("{}"#,                // rust-lit-guard: allow
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

    fn empty_schema(type_name: &str) -> CfnSchema {
        CfnSchema {
            type_name: type_name.to_string(),
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
        }
    }

    #[test]
    fn test_is_tag_map_ref_requires_canonical_tag_name_and_shape() {
        let string_property = || CfnProperty {
            prop_type: Some(TypeValue::Single("string".to_string())),
            ..Default::default()
        };
        let key_value_properties = || {
            BTreeMap::from([
                ("Key".to_string(), string_property()),
                ("Value".to_string(), string_property()),
            ])
        };
        let object_definition = |properties| CfnDefinition {
            def_type: Some("object".to_string()),
            properties: Some(properties),
            ..Default::default()
        };

        let mut tag_with_extra_properties = key_value_properties();
        tag_with_extra_properties.insert("Values".to_string(), string_property());

        let mut schema = empty_schema("AWS::Test::Resource");
        schema.definitions = Some(BTreeMap::from([
            ("Tag".to_string(), object_definition(key_value_properties())),
            (
                "TagWithExtraField".to_string(),
                object_definition(tag_with_extra_properties),
            ),
            (
                "AttributePair".to_string(),
                object_definition(key_value_properties()),
            ),
        ]));

        assert!(is_tag_map_ref(&schema, "#/definitions/Tag"));
        assert!(
            !is_tag_map_ref(&schema, "#/definitions/TagWithExtraField"),
            "a Tag-prefixed definition with additional fields is not a canonical tag map"
        );
        assert!(
            !is_tag_map_ref(&schema, "#/definitions/AttributePair"),
            "non-Tag Key/Value definitions include ELB attributes and must retain their struct shape"
        );
    }

    #[test]
    fn test_is_tag_map_ref_accepts_only_direct_string_pattern_map() {
        let schema: CfnSchema = serde_json::from_value(serde_json::json!({
            "typeName": "AWS::Test::Resource",
            "properties": {},
            "definitions": {
                "TagMap": {
                    "type": "object",
                    "patternProperties": {
                        ".+": {
                            "type": "string",
                            "minLength": 1,
                            "maxLength": 256
                        }
                    },
                    "additionalProperties": false
                },
                "TagNumberMap": {
                    "type": "object",
                    "patternProperties": {
                        ".+": {
                            "type": "number"
                        }
                    },
                    "additionalProperties": false
                }
            }
        }))
        .expect("synthetic tag-map schema should deserialize");

        assert!(
            is_tag_map_ref(&schema, "#/definitions/TagMap"),
            "a single direct-string patternProperties entry is a canonical tag map"
        );
        assert!(
            !is_tag_map_ref(&schema, "#/definitions/TagNumberMap"),
            "patternProperties with non-string values is not a canonical tag map"
        );
    }

    fn emitted_replacement_class(class: ReplacementClass) -> String {
        let mut code = String::new();
        emit_replacement_class_builder(&mut code, class);
        code
    }

    #[test]
    fn test_replacement_class_rejects_unclassified_resource() {
        let error = declared_replacement_class(
            "example.Unclassified",
            &empty_schema("AWS::Example::Unclassified"),
        )
        .expect_err("an unclassified resource must fail codegen");

        let message = error.to_string();
        assert!(message.contains("example.Unclassified"));
        assert!(message.contains("add it to REPLACEMENT_CLASSES"));
    }

    #[test]
    fn test_replacement_class_rejects_invalid_attribute_declarations() {
        let missing_error = declared_replacement_class("iam.Role", &empty_schema("AWS::IAM::Role"))
            .expect_err("a nonexistent unique-name attribute must fail codegen");
        assert!(missing_error.to_string().contains("role_name"));
        assert!(missing_error.to_string().contains("top-level property"));

        let mut read_only_schema = empty_schema("AWS::IAM::Role");
        read_only_schema.properties.insert(
            "RoleName".to_string(),
            CfnProperty {
                prop_type: Some(TypeValue::Single("string".to_string())),
                ..Default::default()
            },
        );
        read_only_schema.read_only_properties = vec!["/properties/RoleName".to_string()];
        let read_only_error = declared_replacement_class("iam.Role", &read_only_schema)
            .expect_err("a read-only unique-name attribute must fail codegen");
        assert!(read_only_error.to_string().contains("role_name"));
        assert!(read_only_error.to_string().contains("read-only"));

        let mut non_string_schema = empty_schema("AWS::IAM::Role");
        non_string_schema.properties.insert(
            "RoleName".to_string(),
            CfnProperty {
                prop_type: Some(TypeValue::Single("integer".to_string())),
                ..Default::default()
            },
        );
        let non_string_error = declared_replacement_class("iam.Role", &non_string_schema)
            .expect_err("a non-string unique-name attribute must fail codegen");
        assert!(non_string_error.to_string().contains("role_name"));
        assert!(non_string_error.to_string().contains("must be a string"));
    }

    #[test]
    fn test_declared_replacement_class_covers_all_classes() {
        let mut attr_schema = empty_schema("AWS::IAM::Role");
        attr_schema.properties.insert(
            "RoleName".to_string(),
            CfnProperty {
                prop_type: Some(TypeValue::Single("string".to_string())),
                ..Default::default()
            },
        );
        let attr_class = declared_replacement_class("iam.Role", &attr_schema).unwrap();
        assert_eq!(attr_class, ReplacementClass::Attribute("role_name"));
        assert_eq!(
            emitted_replacement_class(attr_class),
            "        .with_unique_name_attribute(\"role_name\")\n"
        );

        let coexisting_class =
            declared_replacement_class("ec2.Vpc", &empty_schema("AWS::EC2::VPC")).unwrap();
        assert_eq!(coexisting_class, ReplacementClass::Coexisting);
        assert_eq!(
            emitted_replacement_class(coexisting_class),
            "        .with_coexisting_replacement()\n"
        );

        let conflicting_class =
            declared_replacement_class("ec2.Subnet", &empty_schema("AWS::EC2::Subnet")).unwrap();
        assert_eq!(conflicting_class, ReplacementClass::Conflicting);
        assert_eq!(
            emitted_replacement_class(conflicting_class),
            "        .with_conflicting_replacement()\n"
        );
    }

    #[test]
    fn test_declared_replacement_class_issue_441_named_scenarios() {
        let security_group = cfn_schema_for_codegen_tests("AWS::EC2::SecurityGroup");
        assert_eq!(
            declared_replacement_class("ec2.SecurityGroup", &security_group).unwrap(),
            ReplacementClass::Attribute("group_name")
        );

        let web_acl = cfn_schema_for_codegen_tests("AWS::WAFv2::WebACL");
        assert_eq!(
            declared_replacement_class("wafv2.WebAcl", &web_acl).unwrap(),
            ReplacementClass::Attribute("name")
        );

        let bucket_policy = cfn_schema_for_codegen_tests("AWS::S3::BucketPolicy");
        assert_eq!(
            declared_replacement_class("s3.BucketPolicy", &bucket_policy).unwrap(),
            ReplacementClass::Conflicting
        );
    }

    #[test]
    fn test_cloudfront_function_replacement_class_uses_top_level_name() {
        let function = cfn_schema_for_codegen_tests("AWS::CloudFront::Function");

        assert_eq!(
            declared_replacement_class("cloudfront.Function", &function).unwrap(),
            ReplacementClass::Attribute("name")
        );
    }

    #[test]
    fn test_scheduler_resources_are_registered() {
        for resource in ["scheduler.Schedule", "scheduler.ScheduleGroup"] {
            assert_eq!(
                REPLACEMENT_CLASSES
                    .iter()
                    .find(|(registered, _)| *registered == resource)
                    .map(|(_, class)| *class),
                Some(ReplacementClass::Attribute("name")),
                "{resource} must be registered with name-based replacement"
            );
        }
    }

    #[test]
    fn test_cloudwatch_alarm_and_budgets_budget_are_registered() {
        let alarm = cfn_schema_for_codegen_tests("AWS::CloudWatch::Alarm");
        assert_eq!(
            declared_replacement_class("cloudwatch.Alarm", &alarm).unwrap(),
            ReplacementClass::Attribute("alarm_name")
        );

        let budget = cfn_schema_for_codegen_tests("AWS::Budgets::Budget");
        assert_eq!(
            declared_replacement_class("budgets.Budget", &budget).unwrap(),
            ReplacementClass::Conflicting
        );
    }

    #[test]
    fn test_sns_resources_are_registered() {
        let topic = cfn_schema_for_codegen_tests("AWS::SNS::Topic");
        assert_eq!(
            declared_replacement_class("sns.Topic", &topic).unwrap(),
            ReplacementClass::Conflicting
        );

        let subscription = cfn_schema_for_codegen_tests("AWS::SNS::Subscription");
        assert_eq!(
            declared_replacement_class("sns.Subscription", &subscription).unwrap(),
            ReplacementClass::Conflicting
        );
    }

    #[test]
    fn test_typed_json_policy_classifications_use_exact_full_paths() {
        let expected_typed = [
            ("AWS::SNS::Subscription", "DeliveryPolicy"),
            ("AWS::SNS::Subscription", "RedrivePolicy"),
            ("AWS::SNS::Subscription", "ReplayPolicy"),
            ("AWS::SNS::Topic", "ArchivePolicy"),
            ("AWS::Logs::LogGroup", "DataProtectionPolicy"),
            ("AWS::Logs::LogGroup", "FieldIndexPolicies"),
        ];
        for (cfn_type, property) in expected_typed {
            assert!(
                matches!(
                    resource_type_override(cfn_type, CfnPropertyPath::new(None, property)),
                    Some(TypeOverride::AttributeType { .. })
                ),
                "{cfn_type}.{property} must have a full-path AttributeType override"
            );
            assert!(
                matches!(
                    classify_bare_object(cfn_type, CfnPropertyPath::new(None, property)),
                    Some(BareObjectClassification::TypedOverride { .. })
                ),
                "{cfn_type}.{property} must be classified exactly once as a typed override"
            );
        }

        let topic_data_protection = find_omitted_property(
            OMITTED_RESOURCE_PROPERTIES,
            "AWS::SNS::Topic",
            CfnPropertyPath::new(None, "DataProtectionPolicy"),
        )
        .expect("SNS Topic DataProtectionPolicy must have a measured omission reason");
        assert!(
            topic_data_protection
                .reason
                .contains("no longer available to new customers")
        );
        assert!(matches!(
            classify_bare_object(
                "AWS::SNS::Topic",
                CfnPropertyPath::new(None, "DataProtectionPolicy")
            ),
            Some(BareObjectClassification::OmittedResource(_))
        ));

        let expected_entries = BTreeSet::from([(
            "AWS::SNS::Subscription",
            CfnPropertyPath::new(None, "FilterPolicy"),
        )]);
        let actual_entries = FREE_FORM_JSON_PROPERTIES
            .iter()
            .map(|entry| (entry.cfn_type, entry.property_path))
            .collect::<BTreeSet<_>>();
        assert_eq!(actual_entries, expected_entries);
        assert!(matches!(
            classify_bare_object(
                "AWS::SNS::Subscription",
                CfnPropertyPath::new(None, "FilterPolicy")
            ),
            Some(BareObjectClassification::FreeFormJson(_))
        ));
        assert!(
            FREE_FORM_JSON_PROPERTIES
                .iter()
                .all(|entry| !entry.reason.is_empty()),
            "every omitted free-form JSON property must document why it is unsupported"
        );

        let subscription = cfn_schema_for_codegen_tests("AWS::SNS::Subscription");
        let subscription_generated =
            generate_schema_code(&subscription, "AWS::SNS::Subscription").unwrap();
        for property in ["DeliveryPolicy", "RedrivePolicy", "ReplayPolicy"] {
            assert!(
                subscription_generated
                    .contains(&format!("build_{}_attribute", property.to_snake_case())),
                "AWS::SNS::Subscription.{property} must be emitted"
            );
        }
        assert!(!subscription_generated.contains("build_filter_policy_attribute"));

        let subscription_markdown =
            generate_markdown(&subscription, "AWS::SNS::Subscription").unwrap();
        assert!(subscription_markdown.contains("`filter_policy`"));

        let topic = cfn_schema_for_codegen_tests("AWS::SNS::Topic");
        let topic_generated = generate_schema_code(&topic, "AWS::SNS::Topic").unwrap();
        assert!(topic_generated.contains("build_archive_policy_attribute"));
        assert!(!topic_generated.contains("build_data_protection_policy_attribute"));
        let topic_markdown = generate_markdown(&topic, "AWS::SNS::Topic").unwrap();
        assert!(topic_markdown.contains("`data_protection_policy`"));

        let log_group = cfn_schema_for_codegen_tests("AWS::Logs::LogGroup");
        let log_group_generated = generate_schema_code(&log_group, "AWS::Logs::LogGroup").unwrap();
        assert!(log_group_generated.contains("build_data_protection_policy_attribute"));
        assert!(log_group_generated.contains("build_field_index_policies_attribute"));
        assert!(
            log_group_generated
                .contains(".with_default(Value::Concrete(ConcreteValue::List(vec![])))")
        );
        let log_group_markdown = generate_markdown(&log_group, "AWS::Logs::LogGroup").unwrap();
        assert!(log_group_markdown.contains("- **Default:** `[]`"));
    }

    fn synthetic_struct_schema(
        type_name: &str,
        top_level_name: &str,
        definition_name: &str,
        fields: BTreeMap<String, CfnProperty>,
    ) -> CfnSchema {
        let mut schema = empty_schema(type_name);
        schema.properties.insert(
            top_level_name.to_string(),
            CfnProperty {
                ref_path: Some(format!("#/definitions/{definition_name}")),
                ..Default::default()
            },
        );
        schema.definitions = Some(BTreeMap::from([(
            definition_name.to_string(),
            CfnDefinition {
                def_type: Some("object".to_string()),
                properties: Some(fields),
                ..Default::default()
            },
        )]));
        schema
    }

    #[test]
    fn test_raw_acronym_parent_path_agrees_in_prescan_generation_walker_and_docs() {
        let marker = CfnProperty {
            prop_type: Some(TypeValue::Single("object".to_string())),
            properties: Some(BTreeMap::new()),
            ..Default::default()
        };
        let schema = synthetic_struct_schema(
            "AWS::Test::AcronymPath",
            "Config",
            "SSESpecification",
            BTreeMap::from([("Marker".to_string(), marker.clone())]),
        );
        let raw_path = CfnPropertyPath::new(Some("SSESpecification"), "Marker");

        let (prescanned_type, _) = cfn_type_to_carina_type_with_enum_with_paths(
            &marker,
            "Marker",
            &schema,
            "aws.test.AcronymPath",
            &BTreeMap::new(),
            CfnTypeGenerationPaths {
                structural: &["SseSpecification".to_string()],
                provider: &["Config".to_string(), "Marker".to_string()],
                cfn_parent: Some("SSESpecification"),
            },
        );
        assert!(prescanned_type.contains("vec![]"), "{prescanned_type}");

        let generated = generate_synthetic_schema_code(&schema)
            .expect("raw acronym parent must classify during generation");
        assert!(
            generated.contains(
                "StructField::optional(\"marker\", AttributeType::struct_(\"Marker\".to_string(), vec![]))"
            ),
            "{generated}"
        );

        let used_rows = assert_schema_bare_objects_classified(&schema);
        assert!(used_rows.contains(&(
            TEST_EMPTY_OBJECT_PROPERTIES[0].cfn_type,
            TEST_EMPTY_OBJECT_PROPERTIES[0].property_path,
        )));

        let markdown = generate_markdown(&schema, "AWS::Test::AcronymPath")
            .expect("raw acronym parent must classify during Markdown rendering");
        assert!(
            markdown.contains("| `marker` | Empty struct (`{}`) |"),
            "{markdown}"
        );
        assert!(
            classify_bare_object(
                &schema.type_name,
                CfnPropertyPath::new(Some("OtherSpecification"), "Marker"),
            )
            .is_none(),
            "classification must not fall back to a matching leaf name"
        );
        assert_eq!(raw_path.to_string(), "SSESpecification.Marker");
    }

    #[test]
    fn test_nested_free_form_json_is_omitted_documented_and_rejected_by_schema_shape() {
        let schema = synthetic_struct_schema(
            "AWS::Test::NestedOmission",
            "Config",
            "ParentConfig",
            BTreeMap::from([
                (
                    "Enabled".to_string(),
                    CfnProperty {
                        prop_type: Some(TypeValue::Single("boolean".to_string())),
                        ..Default::default()
                    },
                ),
                (
                    "ChildPolicy".to_string(),
                    CfnProperty {
                        prop_type: Some(TypeValue::Single("object".to_string())),
                        properties: Some(BTreeMap::new()),
                        ..Default::default()
                    },
                ),
            ]),
        );

        let generated = generate_synthetic_schema_code(&schema)
            .expect("nested free-form JSON must be skipped before type generation");
        assert!(generated.contains("StructField::optional(\"enabled\""));
        assert!(!generated.contains("child_policy"), "{generated}");

        let markdown = generate_markdown(&schema, "AWS::Test::NestedOmission")
            .expect("nested omission must be documentable");
        assert!(!markdown.contains("| `child_policy` |"), "{markdown}");
        assert!(markdown.contains("`config.child_policy`"), "{markdown}");

        // This is the exact parent Struct shape emitted above after omission.
        // Carina's schema validator must reject the removed field instead of
        // silently discarding it.
        let parent_schema = carina_core::schema::ResourceSchema::new("test.NestedOmission")
            .attribute(
                carina_core::schema::AttributeSchema::optional(
                    "config",
                    carina_core::schema::AttributeType::struct_(
                        "ParentConfig".to_string(),
                        vec![
                            carina_core::schema::StructField::optional(
                                "enabled",
                                carina_core::schema::AttributeType::bool(),
                            )
                            .build(),
                        ],
                    ),
                )
                .build(),
            )
            .validate()
            .expect("synthetic nested-omission schema must validate");
        let unknown_value = carina_core::resource::Value::Concrete(
            carina_core::resource::ConcreteValue::Map(indexmap::IndexMap::from([
                (
                    "enabled".to_string(),
                    carina_core::resource::Value::Concrete(
                        carina_core::resource::ConcreteValue::Bool(true),
                    ),
                ),
                (
                    "child_policy".to_string(),
                    carina_core::resource::Value::Concrete(
                        carina_core::resource::ConcreteValue::Map(indexmap::IndexMap::new()),
                    ),
                ),
            ])),
        );
        let error = parent_schema
            .effective_attribute("config")
            .expect("synthetic config attribute")
            .validate(&unknown_value)
            .expect_err("an omitted nested field must be unknown")
            .to_string();
        assert!(error.contains("child_policy"), "{error}");
        assert!(error.to_lowercase().contains("unknown"), "{error}");
    }

    type UsedClassificationRows = BTreeSet<(&'static str, CfnPropertyPath<'static>)>;

    #[test]
    fn test_iam_policy_classification_does_not_match_an_unlisted_suffix_leaf() {
        assert!(!is_iam_policy_document_property(
            "AWS::Test::PolicyCollision",
            CfnPropertyPath::new(None, "PolicyDocument"),
        ));
    }

    #[test]
    fn test_unreferenced_definitions_are_not_classification_inputs() {
        let mut schema = empty_schema("AWS::Test::UnreferencedDefinitions");
        schema.definitions = Some(BTreeMap::from([
            (
                "UnusedMarker".to_string(),
                CfnDefinition {
                    def_type: Some("object".to_string()),
                    properties: Some(BTreeMap::new()),
                    ..Default::default()
                },
            ),
            (
                "UnusedMarkers".to_string(),
                CfnDefinition {
                    def_type: Some("array".to_string()),
                    items: Some(Box::new(CfnProperty {
                        prop_type: Some(TypeValue::Single("object".to_string())),
                        properties: Some(BTreeMap::new()),
                        ..Default::default()
                    })),
                    ..Default::default()
                },
            ),
        ]));

        assert!(assert_schema_bare_objects_classified(&schema).is_empty());
    }

    #[test]
    fn test_array_definition_items_are_classified_at_the_referencing_property_path() {
        let mut schema = empty_schema("AWS::Test::ArrayReference");
        schema.properties.insert(
            "Markers".to_string(),
            CfnProperty {
                ref_path: Some("#/definitions/MarkerList".to_string()),
                ..Default::default()
            },
        );
        schema.definitions = Some(BTreeMap::from([(
            "MarkerList".to_string(),
            CfnDefinition {
                def_type: Some("array".to_string()),
                items: Some(Box::new(CfnProperty {
                    prop_type: Some(TypeValue::Single("object".to_string())),
                    properties: Some(BTreeMap::new()),
                    ..Default::default()
                })),
                ..Default::default()
            },
        )]));

        let used_rows = assert_schema_bare_objects_classified(&schema);
        assert!(used_rows.contains(&(
            TEST_EMPTY_OBJECT_PROPERTIES[1].cfn_type,
            TEST_EMPTY_OBJECT_PROPERTIES[1].property_path,
        )));
    }

    fn collect_property_refs(prop: &CfnProperty, refs: &mut BTreeSet<String>) {
        if let Some(ref_path) = &prop.ref_path
            && let Some(def_name) = ref_def_name(ref_path)
        {
            refs.insert(def_name.to_string());
        }
        if let Some(items) = &prop.items {
            collect_property_refs(items, refs);
        }
        for child in prop
            .properties
            .iter()
            .flat_map(|properties| properties.values())
            .chain(
                prop.pattern_properties
                    .iter()
                    .flat_map(|properties| properties.values()),
            )
            .chain(prop.one_of.iter())
            .chain(prop.any_of.iter())
        {
            collect_property_refs(child, refs);
        }
    }

    fn record_classified_row(
        classification: BareObjectClassification,
        used_rows: &mut UsedClassificationRows,
    ) {
        let entry = match classification {
            BareObjectClassification::FreeFormJson(entry)
            | BareObjectClassification::EmptyObject(entry)
            | BareObjectClassification::StringMap(entry)
            | BareObjectClassification::OmittedResource(entry) => {
                Some((entry.cfn_type, entry.property_path))
            }
            BareObjectClassification::IamPolicyDocument(entry) => Some(*entry),
            BareObjectClassification::TypedOverride { .. } => None,
        };
        if let Some(entry) = entry {
            used_rows.insert(entry);
        }
    }

    fn assert_property_classified(
        schema: &CfnSchema,
        prop_name: &str,
        prop: &CfnProperty,
        parent_struct: Option<&str>,
        used_rows: &mut UsedClassificationRows,
        active_array_definitions: &mut BTreeSet<String>,
    ) {
        let property_path = CfnPropertyPath::new(parent_struct, prop_name);
        let mut omitted = false;
        for table in [
            FREE_FORM_JSON_PROPERTIES,
            TEST_FREE_FORM_JSON_PROPERTIES,
            OMITTED_RESOURCE_PROPERTIES,
        ] {
            if let Some(entry) = find_omitted_property(table, &schema.type_name, property_path) {
                used_rows.insert((entry.cfn_type, entry.property_path));
                omitted = true;
            }
        }
        if omitted {
            return;
        }
        if let Some(entry) = iam_policy_document_property(&schema.type_name, property_path) {
            used_rows.insert(*entry);
        }

        if property_is_bare_object(prop, schema) {
            let classification = classify_bare_object(&schema.type_name, property_path)
                .unwrap_or_else(|| {
                    panic!(
                        "unclassified cached bare object {}.{}; use FREE_FORM_JSON_PROPERTIES, EMPTY_OBJECT_PROPERTIES, STRING_MAP_PROPERTIES, or a dedicated typed override",
                        schema.type_name, property_path
                    )
                });
            record_classified_row(classification, used_rows);
            return;
        }

        if is_dsl_tags_property(schema, prop_name)
            || prop
                .ref_path
                .as_deref()
                .is_some_and(|ref_path| is_tag_map_ref(schema, ref_path))
        {
            return;
        }

        if let Some(ref_path) = &prop.ref_path
            && let Some(resolved) = resolve_ref_classified(schema, ref_path)
        {
            match resolved {
                ResolvedDef::ObjectMap { value } => {
                    object_map_value_or_panic(schema, property_path, value);
                    return;
                }
                ResolvedDef::Opaque
                    if resolve_ref(schema, ref_path).is_some_and(|definition| {
                        definition.def_type.as_deref() == Some("object")
                    }) =>
                {
                    panic!(
                        "unsupported cached object definition {}.{} referenced by {ref_path}; expected a struct, direct-string map, tag map, typed map of struct, or classified bare object",
                        schema.type_name, property_path
                    );
                }
                // Array-definition items must keep the referencing property
                // path, so they are handled by the dedicated branch below.
                ResolvedDef::Array { .. } => {}
                // Every other resolved ref owns its shape; reachable struct
                // and oneOf children are walked from the definitions pass.
                _ => return,
            }
        }

        if property_declares_object(prop) {
            if let Some(properties) = &prop.properties
                && !properties.is_empty()
            {
                // Walked below.
            } else if let Some(value) = direct_object_map_value(prop) {
                object_map_value_or_panic(schema, property_path, value);
                if value
                    .properties
                    .as_ref()
                    .is_some_and(|properties| !properties.is_empty())
                {
                    assert_property_classified(
                        schema,
                        prop_name,
                        value,
                        parent_struct,
                        used_rows,
                        active_array_definitions,
                    );
                }
                return;
            } else {
                panic!(
                    "unsupported cached object property {}.{}; expected a struct, direct-string map, tag map, typed map of struct, or classified bare object",
                    schema.type_name, property_path
                );
            }
        }

        // Array definitions are emitted through the property that references
        // them, so their item shape uses that property's canonical path.
        if let Some(ref_path) = &prop.ref_path
            && let Some(definition_name) = ref_def_name(ref_path)
            && let Some(definition) = schema
                .definitions
                .as_ref()
                .and_then(|definitions| definitions.get(definition_name))
            && definition.def_type.as_deref() == Some("array")
            && let Some(items) = &definition.items
        {
            if active_array_definitions.insert(definition_name.to_string()) {
                assert_property_classified(
                    schema,
                    prop_name,
                    items,
                    parent_struct,
                    used_rows,
                    active_array_definitions,
                );
                active_array_definitions.remove(definition_name);
            }
            return;
        }

        if !prop.one_of.is_empty() {
            // Match codegen's compatibility-union behavior: when one variant
            // is an array, only that representable source shape is walked.
            if let Some(array_variant) = prop
                .one_of
                .iter()
                .find(|variant| property_resolves_to_array(variant, schema))
            {
                assert_property_classified(
                    schema,
                    prop_name,
                    array_variant,
                    parent_struct,
                    used_rows,
                    active_array_definitions,
                );
                return;
            }
            for variant in &prop.one_of {
                if let Some(properties) = &variant.properties {
                    for (field_name, field_prop) in properties {
                        assert_property_classified(
                            schema,
                            field_name,
                            field_prop,
                            Some(prop_name),
                            used_rows,
                            active_array_definitions,
                        );
                    }
                }
            }
        }

        if let Some(properties) = &prop.properties {
            for (field_name, field_prop) in properties {
                assert_property_classified(
                    schema,
                    field_name,
                    field_prop,
                    Some(prop_name),
                    used_rows,
                    active_array_definitions,
                );
            }
        }
        if let Some(items) = &prop.items {
            assert_property_classified(
                schema,
                prop_name,
                items,
                parent_struct,
                used_rows,
                active_array_definitions,
            );
        }
    }

    fn reachable_definition_names(schema: &CfnSchema) -> BTreeSet<String> {
        let mut reachable = BTreeSet::new();
        for property in schema.properties.values() {
            collect_property_refs(property, &mut reachable);
        }

        let mut expanded = BTreeSet::new();
        while let Some(definition_name) = reachable
            .iter()
            .find(|definition_name| !expanded.contains(*definition_name))
            .cloned()
        {
            expanded.insert(definition_name.clone());
            let Some(definition) = schema
                .definitions
                .as_ref()
                .and_then(|definitions| definitions.get(&definition_name))
            else {
                continue;
            };
            for property in definition
                .properties
                .iter()
                .flat_map(|properties| properties.values())
                .chain(
                    definition
                        .pattern_properties
                        .iter()
                        .flat_map(|properties| properties.values()),
                )
                .chain(definition.items.iter().map(Box::as_ref))
                .chain(definition.one_of.iter())
            {
                collect_property_refs(property, &mut reachable);
            }
        }
        reachable
    }

    fn assert_schema_bare_objects_classified(schema: &CfnSchema) -> UsedClassificationRows {
        let reachable_definitions = reachable_definition_names(schema);

        let mut used_rows = BTreeSet::new();
        let mut active_array_definitions = BTreeSet::new();
        for (property_name, property) in &schema.properties {
            assert_property_classified(
                schema,
                property_name,
                property,
                None,
                &mut used_rows,
                &mut active_array_definitions,
            );
        }
        if let Some(definitions) = &schema.definitions {
            for (definition_name, definition) in definitions {
                if !reachable_definitions.contains(definition_name) {
                    continue;
                }
                if is_bare_object_definition(definition) {
                    // The referencing property already classified this shape
                    // using the path codegen uses to emit it.
                    continue;
                }
                if let Some(properties) = &definition.properties {
                    for (field_name, field_prop) in properties {
                        assert_property_classified(
                            schema,
                            field_name,
                            field_prop,
                            Some(definition_name),
                            &mut used_rows,
                            &mut active_array_definitions,
                        );
                    }
                }
                for variant in &definition.one_of {
                    if let Some(properties) = &variant.properties {
                        for (field_name, field_prop) in properties {
                            assert_property_classified(
                                schema,
                                field_name,
                                field_prop,
                                Some(definition_name),
                                &mut used_rows,
                                &mut active_array_definitions,
                            );
                        }
                    }
                }
                // Array-definition items are walked from each referencing
                // property above so their classification path matches codegen.
            }
        }
        used_rows
    }

    #[test]
    fn test_every_cached_bare_object_is_explicitly_classified_without_stale_rows() {
        let mut used_rows = BTreeSet::new();
        for path in cached_schema_paths() {
            let raw = std::fs::read_to_string(&path).expect("cached CFN schema");
            let schema: CfnSchema = serde_json::from_str(&raw).expect("cached CFN schema parses");
            used_rows.extend(assert_schema_bare_objects_classified(&schema));
        }

        for table in [
            FREE_FORM_JSON_PROPERTIES,
            EMPTY_OBJECT_PROPERTIES,
            OMITTED_RESOURCE_PROPERTIES,
            STRING_MAP_PROPERTIES,
        ] {
            assert!(
                table.iter().all(|entry| !entry.reason.is_empty()),
                "every classification row needs a reason"
            );
            for entry in table {
                assert!(
                    used_rows.contains(&(entry.cfn_type, entry.property_path)),
                    "stale classification row {}.{} was not used by any cached schema",
                    entry.cfn_type,
                    entry.property_path
                );
            }
        }
        for table in [
            IAM_POLICY_DOCUMENT_PROPERTIES,
            NONSTANDARD_IAM_POLICY_DOCUMENT_PROPERTIES,
        ] {
            for entry in table {
                assert!(
                    used_rows.contains(entry),
                    "stale IAM policy-document row {}.{} was not used by any cached schema",
                    entry.0,
                    entry.1
                );
            }
        }
    }

    #[test]
    fn test_empty_marker_objects_generate_empty_structs() {
        let assert_empty_field = |generated: &str, field: &str| {
            let needle = format!("StructField::optional(\"{field}\", AttributeType::struct_(");
            let emitted_empty_field = generated.match_indices(&needle).find_map(|(start, _)| {
                let end = generated[start..]
                    .find(".build()")
                    .map_or(generated.len(), |end| start + end);
                let emitted_field = &generated[start..end];
                emitted_field.contains("vec![]").then_some(emitted_field)
            });
            let emitted_empty_field = emitted_empty_field
                .unwrap_or_else(|| panic!("{field} must generate as an empty struct: {generated}"));
            assert!(
                !emitted_empty_field.contains("AttributeType::map("),
                "{field} must not remain a string map: {emitted_empty_field}"
            );
        };

        let waf_schema = cfn_schema_for_codegen_tests("AWS::WAFv2::WebACL");
        let waf_generated = generate_schema_code(&waf_schema, "AWS::WAFv2::WebACL")
            .expect("WAFv2 WebACL schema generation should succeed");
        for field in [
            "all_query_arguments",
            "uri_path",
            "query_string",
            "method",
            "count",
            "none",
            "all",
            "forwarded_ip",
            "http_method",
            "ip",
            "asn",
        ] {
            assert_empty_field(&waf_generated, field);
        }
        let waf_markdown = generate_markdown(&waf_schema, "AWS::WAFv2::WebACL")
            .expect("WAFv2 WebACL markdown generation should succeed");
        assert!(
            waf_markdown.contains("| `all_query_arguments` | Empty struct (`{}`) |"),
            "WAF marker docs must expose an empty struct, not a map: {waf_markdown}"
        );

        let s3_schema = cfn_schema_for_codegen_tests("AWS::S3::Bucket");
        let s3_generated = generate_schema_code(&s3_schema, "AWS::S3::Bucket")
            .expect("S3 Bucket schema generation should succeed");
        assert_empty_field(&s3_generated, "simple_prefix");

        let alarm_schema = cfn_schema_for_codegen_tests("AWS::CloudWatch::Alarm");
        let alarm_generated = generate_schema_code(&alarm_schema, "AWS::CloudWatch::Alarm")
            .expect("CloudWatch Alarm schema generation should succeed");
        assert_empty_field(&alarm_generated, "sliding_window");
    }

    #[test]
    fn test_sns_topic_inline_subscription_is_omitted() {
        let schema = cfn_schema_for_codegen_tests("AWS::SNS::Topic");
        let generated = generate_schema_code(&schema, "AWS::SNS::Topic")
            .expect("SNS Topic schema generation should succeed");
        let markdown = generate_markdown(&schema, "AWS::SNS::Topic")
            .expect("SNS Topic markdown generation should succeed");

        assert!(!generated.contains("build_subscription_attribute"));
        assert!(!markdown.contains("### `subscription`"));
        assert!(markdown.contains("## Not yet supported"));
        assert!(markdown.contains("`subscription`"));
        assert!(markdown.contains("use awscc.sns.Subscription instead"));
    }

    #[test]
    fn test_cloudwatch_alarm_numeric_shapes_and_evaluation_window() {
        let schema = cfn_schema_for_codegen_tests("AWS::CloudWatch::Alarm");
        assert_eq!(schema.properties.len(), 28);

        let generated = generate_schema_code(&schema, "AWS::CloudWatch::Alarm")
            .expect("CloudWatch Alarm schema generation should succeed");
        let markdown = generate_markdown(&schema, "AWS::CloudWatch::Alarm")
            .expect("CloudWatch Alarm markdown generation should succeed");

        for property_name in [
            "EvaluationPeriods",
            "EvaluationInterval",
            "DatapointsToAlarm",
            "Period",
        ] {
            let property = schema
                .properties
                .get(property_name)
                .expect("Alarm integer property");
            assert_eq!(
                property.prop_type.as_ref().and_then(TypeValue::as_str),
                Some("integer"),
                "{property_name} must use the CloudWatch API's integer shape"
            );
            let dsl_name = property_name.to_snake_case();
            assert!(
                generated.contains(&format!(
                    "AttributeSchema::optional({}, AttributeType::int())",
                    rust_lit(&dsl_name)
                )),
                "{property_name} must generate an Int attribute: {generated}"
            );
        }

        let number_properties: Vec<&str> = schema
            .properties
            .iter()
            .filter_map(|(name, property)| {
                (property.prop_type.as_ref().and_then(TypeValue::as_str) == Some("number"))
                    .then_some(name.as_str())
            })
            .collect();
        assert_eq!(number_properties, vec!["Threshold"]);
        assert!(
            generated.contains("AttributeSchema::optional(\"threshold\", AttributeType::float())"),
            "Threshold must remain Float: {generated}"
        );
        assert!(
            markdown.contains("### `threshold`") && markdown.contains("- **Type:** Float"),
            "Threshold docs must remain Float: {markdown}"
        );

        assert!(
            generated.contains("AttributeType::struct_(\"EvaluationWindow\".to_string()"),
            "EvaluationWindow oneOf must generate a struct: {generated}"
        );
        assert!(
            generated.contains("StructField::optional(\"sliding_window\"")
                && generated.contains("StructField::optional(\"wall_clock_window\""),
            "EvaluationWindow must expose both union members: {generated}"
        );
    }

    #[test]
    fn test_cloudwatch_alarm_conditional_statistic_choice_is_not_globally_required() {
        let schema = cfn_schema_for_codegen_tests("AWS::CloudWatch::Alarm");
        let groups = detect_exclusive_from_descriptions(&schema);

        assert!(
            !groups.iter().any(|group| {
                group == &["ExtendedStatistic".to_string(), "Statistic".to_string()]
            }),
            "the metric-only Statistic choice must not reject metric-math or PromQL alarms: {groups:?}"
        );

        let generated = generate_schema_code(&schema, "AWS::CloudWatch::Alarm")
            .expect("CloudWatch Alarm schema generation should succeed");
        assert!(
            !generated.contains(
                "schema = schema.exclusive_required(&[\"extended_statistic\", \"statistic\"]);"
            ),
            "a condition-qualified description must not become a global exactly-one constraint: {generated}"
        );
    }

    #[test]
    fn test_budgets_budget_recursive_expression_emits_schema_ref() {
        let schema = cfn_schema_for_codegen_tests("AWS::Budgets::Budget");
        assert_eq!(schema.properties.len(), 4);

        let generated = generate_schema_code(&schema, "AWS::Budgets::Budget")
            .expect("Budgets Budget schema generation should succeed");
        let markdown = generate_markdown(&schema, "AWS::Budgets::Budget")
            .expect("Budgets Budget markdown generation should succeed");

        assert!(
            generated.contains("schema = schema.with_def(\"Expression\""),
            "the recursive Expression definition must be emitted once: {generated}"
        );
        assert!(
            generated.contains("AttributeType::ref_(\"Expression\".to_string())"),
            "recursive Expression edges must use AttributeType::Ref: {generated}"
        );
        for field in ["and", "or"] {
            assert!(
                generated.contains(&format!(
                    "StructField::optional(\"{field}\", AttributeType::list(AttributeType::ref_(\"Expression\".to_string())))"
                )),
                "Expression.{field} must be List<Expression>: {generated}"
            );
        }
        assert!(
            generated.contains(
                "StructField::optional(\"not\", AttributeType::ref_(\"Expression\".to_string()))"
            ),
            "Expression.Not must refer to Expression: {generated}"
        );
        assert!(
            generated.contains(
                "StructField::optional(\"tags\", AttributeType::struct_(\"TagValues\".to_string()"
            ) && !generated
                .contains("StructField::optional(\"tags\", carina_aws_types::tags_type())"),
            "Expression.Tags is a Budgets filter expression, not a Key/Value tag map: {generated}"
        );
        assert!(
            generated.contains("AttributeSchema::optional(\"notifications_with_subscribers\"")
                && generated.contains(".create_only()"),
            "NotificationsWithSubscribers must remain create-only: {generated}"
        );
        assert!(
            markdown.contains("Expression") && markdown.contains("filter_expression"),
            "Budget docs must include the recursive expression surface: {markdown}"
        );
    }

    #[test]
    fn test_budgets_budget_bare_object_fields_use_api_map_shapes() {
        let schema = cfn_schema_for_codegen_tests("AWS::Budgets::Budget");
        let generated = generate_schema_code(&schema, "AWS::Budgets::Budget")
            .expect("Budgets Budget schema generation should succeed");
        let markdown = generate_markdown(&schema, "AWS::Budgets::Budget")
            .expect("Budgets Budget markdown generation should succeed");

        assert!(
            generated.contains(
                "StructField::optional(\"cost_filters\", AttributeType::map(AttributeType::list(AttributeType::string())))"
            ),
            "CostFilters must be Map<String, List<String>>: {generated}"
        );
        assert!(
            generated.contains(
                "StructField::optional(\"planned_budget_limits\", AttributeType::map(AttributeType::ref_(\"Spend\".to_string())))"
            ),
            "PlannedBudgetLimits must be Map<String, Spend>: {generated}"
        );
        assert!(
            markdown.contains("`Map<String, List<String>>`")
                && markdown.contains("`Map<String, Spend>`"),
            "Budget docs must expose both API map shapes: {markdown}"
        );
    }

    #[test]
    fn test_list_struct_mutual_cycle_emits_only_back_referenced_definition() {
        let list_of = |definition: &str| CfnProperty {
            prop_type: Some(TypeValue::Single("array".to_string())),
            items: Some(Box::new(CfnProperty {
                ref_path: Some(format!("#/definitions/{definition}")),
                ..Default::default()
            })),
            ..Default::default()
        };
        let mut schema = empty_schema("AWS::Test::ListCycle");
        schema
            .properties
            .insert("Roots".to_string(), list_of("CycleA"));
        schema.definitions = Some(BTreeMap::from([
            (
                "CycleA".to_string(),
                CfnDefinition {
                    def_type: Some("object".to_string()),
                    properties: Some(BTreeMap::from([(
                        "Children".to_string(),
                        list_of("CycleB"),
                    )])),
                    ..Default::default()
                },
            ),
            (
                "CycleB".to_string(),
                CfnDefinition {
                    def_type: Some("object".to_string()),
                    properties: Some(BTreeMap::from([("Parents".to_string(), list_of("CycleA"))])),
                    ..Default::default()
                },
            ),
        ]));

        let generated = generate_synthetic_schema_code(&schema)
            .expect("a two-definition list-only cycle must terminate");
        assert!(
            generated.contains(
                "StructField::optional(\"parents\", AttributeType::list(AttributeType::ref_(\"CycleA\".to_string())))"
            ),
            "the CycleB -> CycleA back-edge must use a schema ref: {generated}"
        );
        assert_eq!(
            generated
                .matches("schema = schema.with_def(\"CycleA\"")
                .count(),
            1,
            "the referenced definition must be emitted exactly once: {generated}"
        );
        assert!(
            !generated.contains("schema = schema.with_def(\"CycleB\""),
            "an inline definition with no back-reference to itself must stay inline: {generated}"
        );
    }

    #[test]
    fn test_scheduler_api_integer_fields_are_integer_shaped() {
        let schema = cfn_schema_for_codegen_tests("AWS::Scheduler::Schedule");
        let generated = generate_schema_code(&schema, "AWS::Scheduler::Schedule")
            .expect("Scheduler Schedule schema generation should succeed");
        let markdown = generate_markdown(&schema, "AWS::Scheduler::Schedule")
            .expect("Scheduler Schedule markdown generation should succeed");
        let definitions = schema.definitions.as_ref().expect("Schedule definitions");

        for provider_path in [
            "FlexibleTimeWindow.MaximumWindowInMinutes",
            "RetryPolicy.MaximumEventAgeInSeconds",
            "RetryPolicy.MaximumRetryAttempts",
            "EcsParameters.TaskCount",
            "CapacityProviderStrategyItem.Base",
            "CapacityProviderStrategyItem.Weight",
        ] {
            let (definition_name, provider_name) = provider_path
                .split_once('.')
                .expect("nested Scheduler override path");
            assert_eq!(
                resource_type_override(
                    &schema.type_name,
                    CfnPropertyPath::new(Some(definition_name), provider_name),
                ),
                Some(&TypeOverride::IntegerFromNumber),
                "{provider_path} must use the bounds-preserving integer marker"
            );
            let field_prop = definitions
                .get(definition_name)
                .and_then(|definition| definition.properties.as_ref())
                .and_then(|properties| properties.get(provider_name))
                .expect("overridden field in cached Scheduler schema");
            assert_eq!(
                field_prop.prop_type.as_ref().and_then(TypeValue::as_str),
                Some("number"),
                "{provider_path} must remain a CFN number while this override exists"
            );
            assert!(
                field_prop.minimum.is_some() || field_prop.maximum.is_some(),
                "{provider_path} must declare a CFN range"
            );
            let field_name = provider_name.to_snake_case();
            let range = emit_int_range_option(field_prop.minimum, field_prop.maximum);
            let int_fragment = format!(
                "StructField::optional(\"{field_name}\", \
                 AttributeType::refined_int(None, {range}))"
            );
            assert!(
                generated.contains(&int_fragment),
                "{provider_path} must preserve its range and use the integer DSL shape: \
                 {generated}"
            );
            assert!(
                !generated.contains(&format!(
                    "StructField::optional(\"{field_name}\", AttributeType::refined_float"
                )),
                "{provider_path} must not use the float DSL shape: {generated}"
            );
            assert!(
                markdown.contains(&format!(
                    "| `{field_name}` | Int({}) |",
                    range_display_string(field_prop.minimum, field_prop.maximum)
                )),
                "{provider_path} docs must match the integer DSL shape: {markdown}"
            );
        }
    }

    #[test]
    #[should_panic(
        expected = "IntegerFromNumber override for AWS::Test::IntegerSource.Nested.Count requires a CFN minimum or maximum"
    )]
    fn test_integer_from_number_requires_cfn_bounds() {
        let prop = CfnProperty {
            prop_type: Some(TypeValue::Single("number".to_string())),
            ..Default::default()
        };

        integer_from_number_bounds(&prop, "AWS::Test::IntegerSource", "Nested.Count");
    }

    #[test]
    fn test_scheduler_group_name_uses_default_group_validator_and_documents_limitation() {
        let schema = cfn_schema_for_codegen_tests("AWS::Scheduler::Schedule");
        let generated = generate_schema_code(&schema, "AWS::Scheduler::Schedule")
            .expect("Scheduler Schedule schema generation should succeed");
        assert!(
            generated.contains(
                "AttributeSchema::optional(\"group_name\", \
                 crate::schemas::catalog::scheduler_schedule_group_name())"
            ),
            "Schedule.GroupName must use the Cloud Control default-group validator: {generated}"
        );
        assert!(
            generated.contains(
                "Cloud Control identifies schedules by Name only and reads them in the default \
                 group, so a schedule in another group can be created but never read or tracked."
            ),
            "generated schema must explain the measured tracking limitation: {generated}"
        );

        let markdown = generate_markdown(&schema, "AWS::Scheduler::Schedule")
            .expect("Scheduler Schedule markdown generation should succeed");
        assert!(
            markdown.contains("- **Type:** SchedulerScheduleGroupName"),
            "Schedule.GroupName docs must expose the restricted type: {markdown}"
        );
        assert!(
            markdown.contains(
                "Cloud Control identifies schedules by Name only and reads them in the default \
                 group, so a schedule in another group can be created but never read or tracked."
            ),
            "Schedule.GroupName docs must explain the measured tracking limitation: {markdown}"
        );
    }

    #[test]
    fn test_replacement_class_table_exactly_matches_schema_cache() {
        let mut cached_resources = BTreeSet::new();

        for path in cached_schema_paths() {
            let raw = std::fs::read_to_string(&path).expect("cached CFN schema");
            let schema: CfnSchema = serde_json::from_str(&raw).expect("cached CFN schema parses");
            let dsl_name = dsl_resource_name_from_type(&schema.type_name)
                .expect("cached CFN type name is valid");

            assert!(
                cached_resources.insert(dsl_name.clone()),
                "duplicate cached DSL resource name: {dsl_name}"
            );
            declared_replacement_class(&dsl_name, &schema).unwrap_or_else(|error| {
                panic!("replacement class for {dsl_name} must validate: {error}")
            });
        }

        let declared_resources: BTreeSet<&str> = REPLACEMENT_CLASSES
            .iter()
            .map(|(resource, _)| *resource)
            .collect();
        assert_eq!(
            declared_resources.len(),
            REPLACEMENT_CLASSES.len(),
            "replacement class table must not contain duplicate resource names"
        );
        assert_eq!(
            declared_resources,
            cached_resources.iter().map(String::as_str).collect(),
            "replacement class table and schema cache must contain the same resource types"
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

    #[test]
    fn test_pattern_properties_object_ref_generates_typed_struct_map_and_docs() {
        let mut schema = empty_schema("AWS::WAFv2::WebACL");
        schema.properties.insert(
            "CustomResponseBodies".to_string(),
            CfnProperty {
                ref_path: Some("#/definitions/CustomResponseBodies".to_string()),
                ..Default::default()
            },
        );
        schema.definitions = Some(BTreeMap::from([
            (
                "CustomResponseBodies".to_string(),
                CfnDefinition {
                    def_type: Some("object".to_string()),
                    pattern_properties: Some(BTreeMap::from([(
                        "^[a-z]+$".to_string(),
                        CfnProperty {
                            ref_path: Some("#/definitions/CustomResponseBody".to_string()),
                            ..Default::default()
                        },
                    )])),
                    ..Default::default()
                },
            ),
            (
                "CustomResponseBody".to_string(),
                CfnDefinition {
                    def_type: Some("object".to_string()),
                    properties: Some(BTreeMap::from([
                        (
                            "Content".to_string(),
                            CfnProperty {
                                prop_type: Some(TypeValue::Single("string".to_string())),
                                ..Default::default()
                            },
                        ),
                        (
                            "ContentType".to_string(),
                            CfnProperty {
                                prop_type: Some(TypeValue::Single("string".to_string())),
                                ..Default::default()
                            },
                        ),
                    ])),
                    required: vec!["Content".to_string(), "ContentType".to_string()],
                    ..Default::default()
                },
            ),
        ]));

        reset_recursion_guard();
        let property = schema
            .properties
            .get("CustomResponseBodies")
            .expect("synthetic map property");
        let (generated, enum_info) = cfn_type_to_carina_type_with_enum(
            property,
            "CustomResponseBodies",
            &schema,
            "aws.wafv2.WebAcl",
            &BTreeMap::new(),
        );
        assert!(enum_info.is_none());
        assert!(
            generated.contains(
                "AttributeType::map(AttributeType::struct_(\"CustomResponseBody\".to_string()"
            ),
            "patternProperties $ref values must generate a typed struct map: {generated}"
        );
        assert!(
            generated.contains("StructField::required(\"content\", AttributeType::string())"),
            "map value struct fields must be generated: {generated}"
        );

        let markdown = generate_markdown(&schema, "AWS::WAFv2::WebACL")
            .expect("typed object map markdown generation should succeed");
        assert!(
            markdown.contains("Map\\<String, [Struct(CustomResponseBody)]"),
            "typed object-map docs must link the map value struct: {markdown}"
        );
    }

    #[test]
    #[should_panic(expected = "unsupported object map")]
    fn test_pattern_properties_rejects_non_string_non_struct_values() {
        let mut schema = empty_schema("AWS::Test::UnsupportedObjectMap");
        schema.definitions = Some(BTreeMap::from([(
            "UnsupportedMap".to_string(),
            CfnDefinition {
                def_type: Some("object".to_string()),
                pattern_properties: Some(BTreeMap::from([(
                    ".*".to_string(),
                    CfnProperty {
                        prop_type: Some(TypeValue::Single("integer".to_string())),
                        ..Default::default()
                    },
                )])),
                ..Default::default()
            },
        )]));
        let property = CfnProperty {
            ref_path: Some("#/definitions/UnsupportedMap".to_string()),
            ..Default::default()
        };

        reset_recursion_guard();
        let _ = cfn_type_to_carina_type_with_enum(
            &property,
            "UnsupportedMap",
            &schema,
            "aws.test.UnsupportedObjectMap",
            &BTreeMap::new(),
        );
    }

    #[test]
    fn test_nested_object_map_diagnostics_use_canonical_cfn_path() {
        let mut schema = empty_schema("AWS::Test::UnsupportedNestedObjectMap");
        schema.properties.insert(
            "Config".to_string(),
            CfnProperty {
                ref_path: Some("#/definitions/Container".to_string()),
                ..Default::default()
            },
        );
        schema.definitions = Some(BTreeMap::from([
            (
                "Container".to_string(),
                CfnDefinition {
                    def_type: Some("object".to_string()),
                    properties: Some(BTreeMap::from([(
                        "NestedMap".to_string(),
                        CfnProperty {
                            ref_path: Some("#/definitions/UnsupportedMap".to_string()),
                            ..Default::default()
                        },
                    )])),
                    ..Default::default()
                },
            ),
            (
                "UnsupportedMap".to_string(),
                CfnDefinition {
                    def_type: Some("object".to_string()),
                    pattern_properties: Some(BTreeMap::from([(
                        ".*".to_string(),
                        CfnProperty {
                            prop_type: Some(TypeValue::Single("integer".to_string())),
                            ..Default::default()
                        },
                    )])),
                    ..Default::default()
                },
            ),
        ]));
        let expected_path = "AWS::Test::UnsupportedNestedObjectMap.Container.NestedMap";
        let nested_map = schema
            .definitions
            .as_ref()
            .and_then(|definitions| definitions.get("Container"))
            .and_then(|definition| definition.properties.as_ref())
            .and_then(|properties| properties.get("NestedMap"))
            .expect("synthetic nested object map");

        let markdown_type_panic = std::panic::catch_unwind(|| {
            markdown_type(
                "NestedMap",
                nested_map,
                &schema,
                None,
                &["Config".to_string(), "NestedMap".to_string()],
                Some("Container"),
            )
        })
        .expect_err("unsupported nested object map must fail Markdown type generation");
        let markdown_type_message = markdown_type_panic
            .downcast_ref::<String>()
            .map(String::as_str)
            .or_else(|| markdown_type_panic.downcast_ref::<&str>().copied())
            .expect("Markdown type panic should contain a string message");
        assert!(
            markdown_type_message.contains(expected_path),
            "Markdown type diagnostic must use the canonical CFN path: {markdown_type_message}"
        );

        reset_recursion_guard();
        let mut struct_defs = BTreeMap::new();
        let config = schema
            .properties
            .get("Config")
            .expect("synthetic top-level config property");
        let collect_panic = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            collect_struct_defs(
                config,
                "Config",
                &schema,
                &[],
                &["Config".to_string()],
                None,
                &mut struct_defs,
            );
        }))
        .expect_err("unsupported nested object map must fail struct collection");
        let collect_message = collect_panic
            .downcast_ref::<String>()
            .map(String::as_str)
            .or_else(|| collect_panic.downcast_ref::<&str>().copied())
            .expect("struct collection panic should contain a string message");
        assert!(
            collect_message.contains(expected_path),
            "struct collection diagnostic must use the canonical CFN path: {collect_message}"
        );
    }

    #[test]
    fn test_classification_tables_are_disjoint() {
        let mut owners: BTreeMap<(&str, CfnPropertyPath<'_>), Vec<&str>> = BTreeMap::new();
        let mut record = |cfn_type, property_path, table| {
            owners
                .entry((cfn_type, property_path))
                .or_default()
                .push(table);
        };

        for (table_name, table) in [
            ("FREE_FORM_JSON_PROPERTIES", FREE_FORM_JSON_PROPERTIES),
            (
                "TEST_FREE_FORM_JSON_PROPERTIES",
                TEST_FREE_FORM_JSON_PROPERTIES,
            ),
            ("EMPTY_OBJECT_PROPERTIES", EMPTY_OBJECT_PROPERTIES),
            ("TEST_EMPTY_OBJECT_PROPERTIES", TEST_EMPTY_OBJECT_PROPERTIES),
            ("STRING_MAP_PROPERTIES", STRING_MAP_PROPERTIES),
            ("OMITTED_RESOURCE_PROPERTIES", OMITTED_RESOURCE_PROPERTIES),
        ] {
            for entry in table {
                record(entry.cfn_type, entry.property_path, table_name);
            }
        }
        for (table_name, table) in [
            (
                "IAM_POLICY_DOCUMENT_PROPERTIES",
                IAM_POLICY_DOCUMENT_PROPERTIES,
            ),
            (
                "NONSTANDARD_IAM_POLICY_DOCUMENT_PROPERTIES",
                NONSTANDARD_IAM_POLICY_DOCUMENT_PROPERTIES,
            ),
        ] {
            for (cfn_type, property_path) in table {
                record(cfn_type, *property_path, table_name);
            }
        }
        for (&(cfn_type, property_path), override_type) in resource_type_overrides().iter() {
            if matches!(override_type, TypeOverride::AttributeType { .. }) {
                let property_path = property_path.split_once('.').map_or_else(
                    || CfnPropertyPath::new(None, property_path),
                    |(parent, property)| CfnPropertyPath::new(Some(parent), property),
                );
                record(
                    cfn_type,
                    property_path,
                    "resource_type_overrides AttributeType",
                );
            }
        }

        let duplicates = owners
            .into_iter()
            .filter(|(_, owners)| owners.len() > 1)
            .collect::<Vec<_>>();
        assert!(
            duplicates.is_empty(),
            "each classified shape must have exactly one owner: {duplicates:?}"
        );
    }

    #[test]
    #[should_panic(
        expected = "each shape must be classified in exactly one place; see CODEGEN.md#classifying-bare-objects"
    )]
    fn test_multiple_bare_object_classifications_panic() {
        let _ = unique_bare_object_classification(
            "AWS::Test::Duplicate",
            CfnPropertyPath::new(None, "Document"),
            vec![
                BareObjectClassification::FreeFormJson(&FREE_FORM_JSON_PROPERTIES[0]),
                BareObjectClassification::EmptyObject(&EMPTY_OBJECT_PROPERTIES[0]),
            ],
        );
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

    /// CloudFront method-list overrides must be `EnumUnordered` so the
    /// generated schema uses `unordered_list` (carina#3093, awscc#469).
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
            (
                "AWS::CloudFront::ResponseHeadersPolicy",
                "AccessControlAllowMethods.Items",
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
    fn test_cloudfront_function_runtime_override_and_dsl_aliases() {
        let schema = cfn_schema_for_codegen_tests("AWS::CloudFront::Function");
        let generated = generate_schema_code(&schema, "AWS::CloudFront::Function")
            .expect("CloudFront Function schema generation should succeed");
        assert!(
            generated
                .contains(r#"("cloudfront-js-2.0".to_string(), "cloudfront_js_2_0".to_string())"#),
            "cloudfront-js-2.0 must expose the DSL alias cloudfront_js_2_0: {generated}"
        );
    }

    #[test]
    fn test_cloudfront_function_nested_read_only_metadata_is_provider_populated() {
        let schema = cfn_schema_for_codegen_tests("AWS::CloudFront::Function");
        let generated = generate_schema_code(&schema, "AWS::CloudFront::Function")
            .expect("CloudFront Function schema generation should succeed");

        assert!(
            generated.contains(
                r#"StructField::provider_populated("function_arn", carina_aws_types::arn())"#
            ),
            "FunctionMetadata.FunctionARN must be provider-populated: {generated}"
        );

        let function_metadata = generated
            .split("AttributeSchema::provider_populated(\"function_metadata\"")
            .nth(1)
            .and_then(|tail| tail.split("AttributeSchema::required(\"name\"").next())
            .expect("generated Function schema should contain function_metadata before name");
        let function_metadata_attribute_builders = function_metadata
            .split(r#".with_provider_name("FunctionARN").build()]))"#)
            .nth(1)
            .expect("function_metadata should contain its nested FunctionARN field");
        assert!(
            function_metadata_attribute_builders.contains(
                r#".with_provider_name("FunctionMetadata")
            .build()"#
            ),
            "FunctionMetadata contains only provider-populated fields, so the top-level attribute must be read-only: {function_metadata_attribute_builders}"
        );
    }

    #[test]
    fn test_s3_bucket_nested_read_only_metadata_fields_are_provider_populated() {
        let schema = cfn_schema_for_codegen_tests("AWS::S3::Bucket");
        let generated = generate_schema_code(&schema, "AWS::S3::Bucket")
            .expect("S3 Bucket schema generation should succeed");

        let s3_tables_destination = generated
            .split(r#"AttributeType::struct_("S3TablesDestination"#)
            .nth(1)
            .and_then(|tail| {
                tail.split(r#".with_provider_name("S3TablesDestination")"#)
                    .next()
            })
            .expect("generated S3 schema should contain S3TablesDestination");
        for field in ["table_namespace", "table_arn"] {
            assert!(
                s3_tables_destination.contains(&format!(
                    r#"StructField::provider_populated("{field}", carina_aws_types::arn())"#
                )) || s3_tables_destination.contains(&format!(
                    r#"StructField::provider_populated("{field}", AttributeType::string())"#
                )),
                "S3TablesDestination.{field} must be provider-populated: {s3_tables_destination}"
            );
        }
        let table_name = s3_tables_destination
            .split(r#"StructField::required("table_name"#)
            .nth(1)
            .and_then(|tail| {
                tail.split(r#"StructField::provider_populated("table_namespace"#)
                    .next()
            })
            .expect("S3TablesDestination should contain table_name before table_namespace");
        assert!(
            !table_name.contains("provider_populated"),
            "S3TablesDestination.TableName must remain writable: {table_name}"
        );

        let metadata_configuration = generated
            .split(r#"AttributeType::struct_("MetadataConfiguration"#)
            .nth(1)
            .expect("generated S3 schema should contain MetadataConfiguration");
        let destination = metadata_configuration
            .split(r#"StructField::provider_populated("destination"#)
            .nth(1)
            .and_then(|tail| {
                tail.split(r#"StructField::optional("inventory_table_configuration"#)
                    .next()
            })
            .expect("MetadataConfiguration should contain destination");
        assert!(
            destination.contains(r#".with_provider_name("Destination").build()"#),
            "MetadataConfiguration.Destination must be provider-populated: {destination}"
        );

        let inventory_table_configuration = metadata_configuration
            .split(r#"StructField::optional("inventory_table_configuration""#)
            .nth(1)
            .and_then(|tail| {
                tail.split(r#"StructField::required("journal_table_configuration""#)
                    .next()
            })
            .expect("MetadataConfiguration should contain InventoryTableConfiguration");
        assert!(
            inventory_table_configuration.contains(
                r#"StructField::provider_populated("table_arn", carina_aws_types::arn())"#
            ),
            "InventoryTableConfiguration.TableArn must be provider-populated: {inventory_table_configuration}"
        );
        assert!(
            inventory_table_configuration.contains(
                r#"StructField::provider_populated("table_name", AttributeType::string())"#
            ),
            "InventoryTableConfiguration.TableName must be provider-populated: {inventory_table_configuration}"
        );

        let journal_table_configuration = metadata_configuration
            .split(r#"StructField::required("journal_table_configuration""#)
            .nth(1)
            .and_then(|tail| {
                tail.split(r#".with_provider_name("JournalTableConfiguration")"#)
                    .next()
            })
            .expect("MetadataConfiguration should contain JournalTableConfiguration");
        assert!(
            journal_table_configuration.contains(
                r#"StructField::provider_populated("table_arn", carina_aws_types::arn())"#
            ),
            "JournalTableConfiguration.TableArn must be provider-populated: {journal_table_configuration}"
        );
        assert!(
            journal_table_configuration.contains(
                r#"StructField::provider_populated("table_name", AttributeType::string())"#
            ),
            "JournalTableConfiguration.TableName must be provider-populated: {journal_table_configuration}"
        );
    }

    #[test]
    fn test_nested_read_only_path_through_array_wildcard_is_provider_populated() {
        let mut schema = empty_schema("AWS::Test::ArrayReadOnly");
        schema.properties.insert(
            "Items".to_string(),
            CfnProperty {
                prop_type: Some(TypeValue::Single("array".to_string())),
                items: Some(Box::new(CfnProperty {
                    prop_type: Some(TypeValue::Single("object".to_string())),
                    properties: Some(BTreeMap::from([
                        (
                            "Owned".to_string(),
                            CfnProperty {
                                prop_type: Some(TypeValue::Single("string".to_string())),
                                ..Default::default()
                            },
                        ),
                        (
                            "Writable".to_string(),
                            CfnProperty {
                                prop_type: Some(TypeValue::Single("string".to_string())),
                                ..Default::default()
                            },
                        ),
                    ])),
                    ..Default::default()
                })),
                ..Default::default()
            },
        );
        schema.read_only_properties = vec!["/properties/Items/*/Owned".to_string()];

        let items = schema
            .properties
            .get("Items")
            .expect("array read-only test schema should contain Items");
        let (generated, _) = cfn_type_to_carina_type_with_enum(
            items,
            "Items",
            &schema,
            "aws.test.ArrayReadOnly",
            &BTreeMap::new(),
        );
        assert!(
            generated
                .contains(r#"StructField::provider_populated("owned", AttributeType::string())"#),
            "Items[*].Owned must be provider-populated: {generated}"
        );
        assert!(
            !generated.contains(
                r#"StructField::provider_populated("writable", AttributeType::string())"#
            ),
            "Items[*].Writable must remain writable: {generated}"
        );
    }

    #[test]
    fn test_nested_read_only_path_through_ref_array_items_is_provider_populated() {
        let item_definition = CfnDefinition {
            def_type: Some("object".to_string()),
            properties: Some(BTreeMap::from([
                (
                    "Owned".to_string(),
                    CfnProperty {
                        prop_type: Some(TypeValue::Single("string".to_string())),
                        ..Default::default()
                    },
                ),
                (
                    "Writable".to_string(),
                    CfnProperty {
                        prop_type: Some(TypeValue::Single("string".to_string())),
                        ..Default::default()
                    },
                ),
            ])),
            ..Default::default()
        };
        let item_ref = CfnProperty {
            ref_path: Some("#/definitions/Item".to_string()),
            ..Default::default()
        };
        let mut schema = empty_schema("AWS::Test::NestedReadOnlyRefArray");
        schema.properties = BTreeMap::from([
            (
                "DirectItems".to_string(),
                CfnProperty {
                    prop_type: Some(TypeValue::Single("array".to_string())),
                    items: Some(Box::new(item_ref.clone())),
                    ..Default::default()
                },
            ),
            (
                "ReferencedItems".to_string(),
                CfnProperty {
                    ref_path: Some("#/definitions/ItemList".to_string()),
                    ..Default::default()
                },
            ),
        ]);
        schema.definitions = Some(BTreeMap::from([
            ("Item".to_string(), item_definition),
            (
                "ItemList".to_string(),
                CfnDefinition {
                    def_type: Some("array".to_string()),
                    items: Some(Box::new(item_ref)),
                    ..Default::default()
                },
            ),
        ]));
        schema.read_only_properties = vec![
            "/properties/DirectItems/*/Owned".to_string(),
            "/properties/ReferencedItems/*/Owned".to_string(),
        ];

        let generated = generate_synthetic_schema_code(&schema)
            .expect("both $ref item array forms should generate");
        for attribute in ["direct_items", "referenced_items"] {
            let attribute_body = generated
                .split(&format!(r#"AttributeSchema::optional("{attribute}""#))
                .nth(1)
                .and_then(|tail| tail.split(".with_provider_name(").next())
                .unwrap_or_else(|| panic!("generated schema should contain {attribute}"));
            assert!(
                attribute_body.contains(
                    r#"StructField::provider_populated("owned", AttributeType::string())"#
                ),
                "{attribute}[*].Owned must be provider-populated: {attribute_body}"
            );
            assert!(
                !attribute_body.contains(
                    r#"StructField::provider_populated("writable", AttributeType::string())"#
                ),
                "{attribute}[*].Writable must remain writable: {attribute_body}"
            );
        }
    }

    #[test]
    fn test_shared_definition_with_path_dependent_read_only_fields_is_rejected() {
        let mut schema = empty_schema("AWS::Test::PathDependentReadOnly");
        let shared_ref = CfnProperty {
            ref_path: Some("#/definitions/Shared".to_string()),
            ..Default::default()
        };
        schema.properties = BTreeMap::from([
            ("Left".to_string(), shared_ref.clone()),
            ("Right".to_string(), shared_ref),
        ]);
        schema.definitions = Some(BTreeMap::from([(
            "Shared".to_string(),
            CfnDefinition {
                def_type: Some("object".to_string()),
                properties: Some(BTreeMap::from([
                    (
                        "Owned".to_string(),
                        CfnProperty {
                            prop_type: Some(TypeValue::Single("string".to_string())),
                            ..Default::default()
                        },
                    ),
                    (
                        "Writable".to_string(),
                        CfnProperty {
                            prop_type: Some(TypeValue::Single("string".to_string())),
                            ..Default::default()
                        },
                    ),
                ])),
                ..Default::default()
            },
        )]));
        schema.read_only_properties = vec!["/properties/Left/Owned".to_string()];

        for (generator, result) in [
            ("Rust", generate_synthetic_schema_code(&schema)),
            (
                "Markdown",
                generate_markdown(&schema, "AWS::Test::PathDependentReadOnly"),
            ),
        ] {
            let error = result.expect_err(
                "a shared definition with path-dependent read-only fields must fail closed",
            );
            let message = error.to_string();
            assert!(message.contains("Shared"), "{generator}: {message}");
            assert!(
                message.contains("/properties/Left")
                    && message.contains("/properties/Right")
                    && message.contains("different nested readOnlyProperties"),
                "{generator}: {message}"
            );
        }
    }

    #[test]
    fn test_unmatched_nested_read_only_pointer_is_rejected() {
        let mut schema = empty_schema("AWS::Test::UnmatchedReadOnly");
        schema.properties.insert(
            "Config".to_string(),
            CfnProperty {
                prop_type: Some(TypeValue::Single("object".to_string())),
                properties: Some(BTreeMap::from([(
                    "Present".to_string(),
                    CfnProperty {
                        prop_type: Some(TypeValue::Single("string".to_string())),
                        ..Default::default()
                    },
                )])),
                ..Default::default()
            },
        );
        let pointer = "/properties/Config/Missing";
        schema.read_only_properties = vec![pointer.to_string()];

        let error = generate_synthetic_schema_code(&schema)
            .expect_err("an unresolved nested readOnlyProperties pointer must fail closed");
        let message = error.to_string();
        assert!(message.contains(pointer), "{message}");
        assert!(message.contains("did not resolve"), "{message}");
    }

    #[test]
    fn test_nested_read_only_pointer_discarded_by_composite_string_override_is_rejected() {
        let mut schema = empty_schema("AWS::ElasticLoadBalancingV2::Listener");
        schema.properties.insert(
            "Config".to_string(),
            CfnProperty {
                ref_path: Some("#/definitions/RedirectConfig".to_string()),
                ..Default::default()
            },
        );
        schema.definitions = Some(BTreeMap::from([(
            "RedirectConfig".to_string(),
            CfnDefinition {
                def_type: Some("object".to_string()),
                properties: Some(BTreeMap::from([(
                    "Protocol".to_string(),
                    CfnProperty {
                        prop_type: Some(TypeValue::Single("object".to_string())),
                        properties: Some(BTreeMap::from([(
                            "Owned".to_string(),
                            CfnProperty {
                                prop_type: Some(TypeValue::Single("string".to_string())),
                                ..Default::default()
                            },
                        )])),
                        ..Default::default()
                    },
                )])),
                ..Default::default()
            },
        )]));
        let pointer = "/properties/Config/Protocol/Owned";
        schema.read_only_properties = vec![pointer.to_string()];

        for (generator, result) in [
            ("Rust", generate_synthetic_schema_code(&schema)),
            (
                "Markdown",
                generate_markdown(&schema, "AWS::ElasticLoadBalancingV2::Listener"),
            ),
        ] {
            let error = result.expect_err(
                "a composite StringType override must not claim its discarded child was emitted",
            );
            let message = error.to_string();
            assert!(message.contains(pointer), "{generator}: {message}");
            assert!(
                message.contains("did not resolve"),
                "{generator}: {message}"
            );
        }
    }

    #[test]
    fn test_required_nested_struct_with_only_read_only_fields_is_read_only_not_required() {
        let mut schema = empty_schema("AWS::Test::NestedReadOnly");
        schema.properties.insert(
            "Config".to_string(),
            CfnProperty {
                prop_type: Some(TypeValue::Single("object".to_string())),
                properties: Some(BTreeMap::from([
                    (
                        "Meta".to_string(),
                        CfnProperty {
                            prop_type: Some(TypeValue::Single("object".to_string())),
                            properties: Some(BTreeMap::from([(
                                "Arn".to_string(),
                                CfnProperty {
                                    prop_type: Some(TypeValue::Single("string".to_string())),
                                    ..Default::default()
                                },
                            )])),
                            required: vec!["Arn".to_string()],
                            ..Default::default()
                        },
                    ),
                    (
                        "Writable".to_string(),
                        CfnProperty {
                            prop_type: Some(TypeValue::Single("string".to_string())),
                            ..Default::default()
                        },
                    ),
                ])),
                required: vec!["Meta".to_string()],
                ..Default::default()
            },
        );
        schema.read_only_properties = vec!["/properties/Config/Meta/Arn".to_string()];

        let generated = generate_synthetic_schema_code(&schema)
            .expect("nested fully read-only structs should generate");
        assert!(
            generated.contains(r#"StructField::provider_populated("meta", AttributeType::struct_("Meta".to_string(), vec![StructField::provider_populated("arn", "#),
            "Config.Meta must be provider-populated when every emitted child is read-only: {generated}"
        );
        assert!(
            !generated.contains(r#"StructField::required("meta", "#),
            "Config.Meta must not remain required when it is provider-populated: {generated}"
        );
    }

    #[test]
    fn test_markdown_rejects_same_inline_struct_name_with_different_read_only_fields() {
        fn parent() -> CfnProperty {
            CfnProperty {
                prop_type: Some(TypeValue::Single("object".to_string())),
                properties: Some(BTreeMap::from([(
                    "Config".to_string(),
                    CfnProperty {
                        prop_type: Some(TypeValue::Single("object".to_string())),
                        properties: Some(BTreeMap::from([
                            (
                                "Owned".to_string(),
                                CfnProperty {
                                    prop_type: Some(TypeValue::Single("string".to_string())),
                                    ..Default::default()
                                },
                            ),
                            (
                                "Writable".to_string(),
                                CfnProperty {
                                    prop_type: Some(TypeValue::Single("string".to_string())),
                                    ..Default::default()
                                },
                            ),
                        ])),
                        ..Default::default()
                    },
                )])),
                ..Default::default()
            }
        }

        let mut schema = empty_schema("AWS::Test::InlineCollision");
        schema.properties = BTreeMap::from([
            ("Left".to_string(), parent()),
            ("Right".to_string(), parent()),
        ]);
        schema.read_only_properties = vec!["/properties/Left/Config/Owned".to_string()];

        let error = generate_markdown(&schema, "AWS::Test::InlineCollision")
            .expect_err("Markdown must not reuse the first path for ambiguous inline structs");
        let message = error.to_string();
        assert!(message.contains("Config"), "{message}");
        assert!(
            message.contains("/properties/Left/Config")
                && message.contains("/properties/Right/Config")
                && message.contains("different nested readOnlyProperties"),
            "{message}"
        );
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
        // This period-plus-bullet shape is not a CFN value-list anchor. The
        // resource-scoped override supplies the canonical InstanceTenancy
        // values instead of widening the heuristic to bullet-body examples.
        let description = r#"The allowed tenancy of instances launched into the VPC.
  +  ``default``: An instance launched into the VPC runs on shared hardware by default.
  +  ``dedicated``: An instance launched into the VPC runs on dedicated hardware by default.
  +  ``host``: Some description.
 Updating ``InstanceTenancy`` requires no replacement."#;

        let result = extract_enum_from_description(description);
        assert_eq!(result, None);
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
    fn test_extract_enum_rejects_cross_reference_backticks_without_value_list_sentence() {
        // awscc#365: LoadBalancer.EnablePrefixForIpv6SourceNat description
        // mentions IpAddressType's ``dualstack`` and the default ``off``,
        // but neither sentence is a value-list sentence for this field.
        let schema = cfn_schema_for_codegen_tests("AWS::ElasticLoadBalancingV2::LoadBalancer");
        let description = schema
            .properties
            .get("EnablePrefixForIpv6SourceNat")
            .expect("EnablePrefixForIpv6SourceNat property exists")
            .description
            .as_deref()
            .expect("EnablePrefixForIpv6SourceNat has description");
        assert_eq!(
            extract_enum_from_description(description),
            None,
            "no value-list sentence anchors here; the matcher must not scoop random backticks"
        );
    }

    #[test]
    fn test_extract_enum_from_description_loadbalancer_ip_address_type_backtick_oxford() {
        // awscc#365: LoadBalancer.IpAddressType uses backtick-wrapped
        // Oxford prose in the cached CFN description.
        // Strategy 4 must strip wrapping `` from each value.
        let schema = cfn_schema_for_codegen_tests("AWS::ElasticLoadBalancingV2::LoadBalancer");
        let description = schema
            .properties
            .get("IpAddressType")
            .expect("IpAddressType property exists")
            .description
            .as_deref()
            .expect("IpAddressType has description");
        assert_eq!(
            extract_enum_from_description(description),
            Some(vec![
                "ipv4".to_string(),
                "dualstack".to_string(),
                "dualstack-without-public-ipv4".to_string()
            ]),
        );
    }

    #[test]
    fn test_extract_enum_from_description_dynamodb_billing_mode_via_bullet_leader() {
        // awscc#365: "Valid values include:" bullet lists must be handled
        // by the bullet-leader path after the broad backtick fallback is removed.
        let schema = cfn_schema_for_codegen_tests("AWS::DynamoDB::Table");
        let description = schema
            .properties
            .get("BillingMode")
            .expect("BillingMode property exists")
            .description
            .as_deref()
            .expect("BillingMode has description");
        assert_eq!(
            extract_enum_from_description(description),
            Some(vec![
                "PAY_PER_REQUEST".to_string(),
                "PROVISIONED".to_string()
            ]),
        );
    }

    #[test]
    fn test_extract_enum_from_description_structural_anchor_colon_bullet() {
        // awscc#365: descriptions like CloudFront
        // GeoRestriction.RestrictionType use a free-form colon-terminated
        // lead-in followed by a backtick bullet list. No lexical value-list
        // keyword appears. The structural anchor captures this shape.
        let description = "The method that you want to use to restrict distribution of your content by country:\n  +  ``none``: No geo restriction is enabled.\n  +  ``blacklist``: The Location elements specify the countries in which you don't want CloudFront to distribute your content.\n  +  ``whitelist``: The Location elements specify the countries in which you want CloudFront to distribute your content.";
        assert_eq!(
            extract_enum_from_description(description),
            Some(vec![
                "none".to_string(),
                "blacklist".to_string(),
                "whitelist".to_string()
            ]),
        );
    }

    #[test]
    fn test_extract_enum_from_description_structural_anchor_rejects_non_value_list() {
        // The structural anchor must not fire on colon-terminated paragraphs
        // that are not value-list lead-ins. For example, a "Note:" section
        // followed by code-style examples in backticks.
        let description = "Some prose. See also examples below:\n  + ``example1`` - a code-style identifier\n  + ``example2`` - another code-style identifier.\n Both are illustrations, not legal values.";
        // The structural anchor IS expected to fire here too (it's
        // structural, not semantic). The test documents that this case
        // currently gets extracted — it's a known limitation. Use this test
        // to lock the current behavior so a future narrowing of the
        // structural anchor can be detected.
        let result = extract_enum_from_description(description);
        // Pin current behavior — Strategy 0 with dash terminator captures
        // example1/example2. This is acceptable because the current cache has
        // no such field; if it ever does, the field can go into
        // resource_type_overrides as a plain string. Update this assertion if
        // the heuristic is later narrowed.
        assert_eq!(
            result,
            Some(vec!["example1".to_string(), "example2".to_string()]),
            "structural anchor currently fires on any colon+bullet shape; if narrowed, update this assertion"
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
            None,
            "awscc#365 removed unanchored backtick/list scraping; IpProtocol's canonical values come from known_enum_overrides()"
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
        // An unanchored "Use X or Y" sentence is not a value-list anchor after
        // awscc#365 deleted the broad backtick scoop.
        let description =
            r#"Use ``enabled`` or ``disabled``. When ``enabled`` is set, the feature activates."#;
        let result = extract_enum_from_description(description);
        assert_eq!(result, None);
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
    fn test_extract_enum_from_description_possible_values_are_two_values() {
        // awscc#357 motivating case: ELBv2 TargetGroup.IpAddressType
        // description (verbatim from the CFN schema).
        let description = "The type of IP address used for this target group. The possible values are ipv4 and ipv6.";
        assert_eq!(
            extract_enum_from_description(description),
            Some(vec!["ipv4".to_string(), "ipv6".to_string()])
        );
    }

    #[test]
    fn test_extract_enum_from_description_possible_values_are_oxford_comma() {
        // awscc#357 motivating case: ELBv2 TargetGroup.ProtocolVersion
        // description (verbatim from the CFN schema).
        let description = "[HTTP/HTTPS protocol] The protocol version. The possible values are GRPC, HTTP1, and HTTP2.";
        assert_eq!(
            extract_enum_from_description(description),
            Some(vec![
                "GRPC".to_string(),
                "HTTP1".to_string(),
                "HTTP2".to_string()
            ])
        );
    }

    #[test]
    fn test_extract_enum_from_description_possible_values_are_no_oxford() {
        // Same prose shape without the Oxford comma — common variant.
        let description = "Some flag. The possible values are foo, bar and baz.";
        assert_eq!(
            extract_enum_from_description(description),
            Some(vec![
                "foo".to_string(),
                "bar".to_string(),
                "baz".to_string()
            ])
        );
    }

    #[test]
    fn test_extract_enum_from_description_possible_values_are_rejects_prose() {
        // "The possible values are ..." followed by a sentence-shaped clause
        // (multi-word tokens, no enum-shaped identifiers) must NOT yield an enum.
        let description = "The mode. The possible values are described in the AWS documentation.";
        assert_eq!(extract_enum_from_description(description), None);
    }

    #[test]
    fn test_resource_type_overrides_target_group_target_type() {
        // awscc#357: TargetGroup.TargetType has no value list in its CFN
        // description, so the legal values are kept in the resource-scoped
        // curated overlay (NOT the field-name-only `known_enum_overrides`
        // table — `TargetType` collides nowhere today but would the moment
        // any other resource grows a field of the same name).
        let overrides = resource_type_overrides();
        assert_eq!(
            overrides.get(&("AWS::ElasticLoadBalancingV2::TargetGroup", "TargetType")),
            Some(&TypeOverride::Enum(vec!["instance", "ip", "lambda", "alb"]))
        );
    }

    #[test]
    fn test_resource_type_overrides_target_group_protocol() {
        // awscc#357: TargetGroup.Protocol has no value list in its CFN
        // description. Use the union of all TargetType-dependent values.
        //
        // This MUST be resource-scoped, not field-name-scoped:
        // `Listener.Protocol` shares the name but allows a different value
        // union (adds `QUIC`/`TCP_QUIC`), so a field-name-only override
        // would silently reject those at validate.
        let overrides = resource_type_overrides();
        assert_eq!(
            overrides.get(&("AWS::ElasticLoadBalancingV2::TargetGroup", "Protocol")),
            Some(&TypeOverride::Enum(vec![
                "HTTP", "HTTPS", "TCP", "TLS", "UDP", "TCP_UDP", "GENEVE"
            ]))
        );
    }

    #[test]
    fn test_resource_type_overrides_listener_top_level_protocol_listed_via_prose() {
        // awscc#360: Listener.Protocol is now enum-derived via the
        // description-scraping path ("supported protocols are HTTP and
        // HTTPS. ... supported protocols are TCP, TLS, UDP, TCP_UDP, QUIC,
        // and TCP_QUIC."). The static list is the UNION across all
        // load-balancer types — apply-time CloudControl enforces the
        // contextual subset. The resource_type_overrides() table itself
        // intentionally still does NOT have a top-level Listener.Protocol
        // entry: the prose matcher carries this case, leaving the overlay
        // table free to express the negative contract "this is not a
        // field-name leak from TargetGroup.Protocol".
        let overrides = resource_type_overrides();
        assert!(
            overrides
                .get(&("AWS::ElasticLoadBalancingV2::Listener", "Protocol"))
                .is_none(),
            "Listener.Protocol must not be in resource_type_overrides — it is derived from description prose, not the overlay"
        );
    }

    #[test]
    fn test_resource_type_overrides_listener_redirect_config_sibling_string_types() {
        // awscc#363 / carina#3494: RedirectConfig placeholder fields use
        // sibling string types so each documented placeholder is scoped to
        // the field that accepts it.
        let overrides = resource_type_overrides();
        for (field, type_constructor) in [
            (
                "RedirectConfig.Protocol",
                "carina_core::schema::types::redirect_protocol()",
            ),
            (
                "RedirectConfig.Host",
                "carina_core::schema::types::redirect_host()",
            ),
            (
                "RedirectConfig.Port",
                "carina_core::schema::types::redirect_port()",
            ),
            (
                "RedirectConfig.Path",
                "carina_core::schema::types::redirect_path()",
            ),
            (
                "RedirectConfig.Query",
                "carina_core::schema::types::redirect_query()",
            ),
        ] {
            assert_eq!(
                overrides.get(&("AWS::ElasticLoadBalancingV2::Listener", field)),
                Some(&TypeOverride::StringType(type_constructor))
            );
        }
    }

    #[test]
    fn test_resource_type_overrides_listener_redirect_config_status_code() {
        // awscc#360: RedirectConfig.StatusCode description is
        // "either permanent (HTTP 301) or temporary (HTTP 302)". The CFN
        // string form uses underscores (`HTTP_301`, `HTTP_302`); no
        // existing matcher recovers this without a curated mapping.
        let overrides = resource_type_overrides();
        assert_eq!(
            overrides.get(&(
                "AWS::ElasticLoadBalancingV2::Listener",
                "RedirectConfig.StatusCode",
            )),
            Some(&TypeOverride::Enum(vec!["HTTP_301", "HTTP_302"]))
        );
    }

    #[test]
    fn test_extract_enum_from_description_supported_protocols_are_union() {
        // awscc#360: Listener.Protocol CFN description verbatim. Two
        // "supported protocols are" sentences — the matcher must union
        // both, not return the first only.
        let description = "The protocol for connections from clients to the load balancer. \
            For Application Load Balancers, the supported protocols are HTTP and HTTPS. \
            For Network Load Balancers, the supported protocols are TCP, TLS, UDP, TCP_UDP, QUIC, and TCP_QUIC. \
            You can't specify the UDP, TCP_UDP, QUIC, or TCP_QUIC protocol if dual-stack mode is enabled. \
            You can't specify a protocol for a Gateway Load Balancer.";
        let result = extract_enum_from_description(description);
        assert!(
            result.is_some(),
            "Listener.Protocol prose should be matched"
        );
        let values = result.unwrap();
        // Order: ALB sentence first (HTTP, HTTPS), then NLB sentence
        // (TCP, TLS, UDP, TCP_UDP, QUIC, TCP_QUIC). Deduplication preserves
        // first occurrence order.
        assert_eq!(
            values,
            vec![
                "HTTP", "HTTPS", "TCP", "TLS", "UDP", "TCP_UDP", "QUIC", "TCP_QUIC"
            ]
        );
    }

    #[test]
    fn test_extract_enum_from_description_supported_values_oxford_comma() {
        // Same Strategy-5 path, common sibling phrasing.
        let description =
            "The block mode. The supported values are off, block-bidirectional, and block-ingress.";
        assert_eq!(
            extract_enum_from_description(description),
            Some(vec![
                "off".to_string(),
                "block-bidirectional".to_string(),
                "block-ingress".to_string()
            ])
        );
    }

    #[test]
    fn test_resource_type_overrides_listener_action_type() {
        // awscc#359: Listener Action.Type carries no value list in its CFN
        // description ("The type of action."), but the legal set is closed
        // and small. Keep the canonical values resource-scoped — `Type` is
        // a field name shared with `VPNGateway.Type` (= `vec!["ipsec.1"]`),
        // so a field-name-only override would clobber it.
        let overrides = resource_type_overrides();
        assert_eq!(
            overrides.get(&("AWS::ElasticLoadBalancingV2::Listener", "Action.Type")),
            Some(&TypeOverride::Enum(vec![
                "forward",
                "authenticate-oidc",
                "authenticate-cognito",
                "redirect",
                "fixed-response",
                "jwt-validation",
            ]))
        );
    }

    #[test]
    fn test_resource_type_overrides_listener_top_level_type_not_scoped() {
        // awscc#359: ensure the Action.Type override does NOT leak into a
        // hypothetical top-level `Type` lookup on the listener. The schema
        // has no top-level `Type` on Listener today, but a field-name-only
        // override would collide with VPNGateway.Type if one ever appeared.
        let overrides = resource_type_overrides();
        assert!(
            overrides
                .get(&("AWS::ElasticLoadBalancingV2::Listener", "Type"))
                .is_none(),
            "Listener.Type (top-level) must not inherit Action.Type's value list"
        );
    }

    #[test]
    fn test_resource_type_overrides_load_balancer_enums() {
        // awscc#365: LoadBalancer fields whose CFN descriptions do not
        // provide reliable value-list anchors are pinned in the resource-
        // scoped overlay. `Type` and `Scheme` are too generic for
        // known_enum_overrides.
        let overrides = resource_type_overrides();
        assert_eq!(
            overrides.get(&("AWS::ElasticLoadBalancingV2::LoadBalancer", "Type")),
            Some(&TypeOverride::Enum(vec![
                "application",
                "network",
                "gateway"
            ]))
        );
        assert_eq!(
            overrides.get(&("AWS::ElasticLoadBalancingV2::LoadBalancer", "Scheme")),
            Some(&TypeOverride::Enum(vec!["internet-facing", "internal"]))
        );
        assert_eq!(
            overrides.get(&(
                "AWS::ElasticLoadBalancingV2::LoadBalancer",
                "EnablePrefixForIpv6SourceNat",
            )),
            Some(&TypeOverride::Enum(vec!["on", "off"]))
        );
        assert_eq!(
            overrides.get(&(
                "AWS::ElasticLoadBalancingV2::LoadBalancer",
                "EnforceSecurityGroupInboundRulesOnPrivateLinkTraffic",
            )),
            Some(&TypeOverride::Enum(vec!["on", "off"]))
        );
    }

    #[test]
    fn test_resource_type_overrides_load_balancer_enums_do_not_collide_with_siblings() {
        let overrides = resource_type_overrides();

        // VPNGateway.Type has its own override (`ipsec.1`) that must not be
        // clobbered by LoadBalancer.Type (`application`/`network`/`gateway`).
        assert_eq!(
            overrides.get(&("AWS::EC2::VPNGateway", "Type")),
            Some(&TypeOverride::Enum(vec!["ipsec.1"])),
            "VPNGateway.Type must keep its own value list"
        );

        // Listener.Action.Type has its own nested override that must not be
        // clobbered by LoadBalancer.Type.
        assert_eq!(
            overrides.get(&("AWS::ElasticLoadBalancingV2::Listener", "Action.Type")),
            Some(&TypeOverride::Enum(vec![
                "forward",
                "authenticate-oidc",
                "authenticate-cognito",
                "redirect",
                "fixed-response",
                "jwt-validation",
            ])),
            "Listener.Action.Type must keep its own value list"
        );

        // None of the LB-specific keys (Scheme/EnablePrefixForIpv6SourceNat/
        // EnforceSecurityGroupInboundRulesOnPrivateLinkTraffic) appear on any
        // sibling resource. Spot-check that the overrides map has exactly one
        // entry per (resource, property), and that the LB entries we added are
        // present at LB only.
        let lb_entries: Vec<&(&str, &str)> = overrides
            .keys()
            .filter(|(_, prop)| {
                matches!(
                    *prop,
                    "Type"
                        | "Scheme"
                        | "EnablePrefixForIpv6SourceNat"
                        | "EnforceSecurityGroupInboundRulesOnPrivateLinkTraffic"
                )
            })
            .collect();
        let lb_type_count = lb_entries
            .iter()
            .filter(|(type_name, _)| *type_name == "AWS::ElasticLoadBalancingV2::LoadBalancer")
            .count();
        assert_eq!(
            lb_type_count, 4,
            "LoadBalancer must own exactly 4 of these property names"
        );
    }

    #[test]
    fn test_resource_type_overrides_vpc_instance_tenancy() {
        // awscc#365: pre-PR the description-scraping heuristic picked up
        // `default`/`dedicated`/`host` via the broad backtick scoop. After
        // the scoop is deleted, the description's period+bullet shape no
        // longer anchors as a value list. The resource overlay supplies the
        // canonical value set.
        let overrides = resource_type_overrides();
        assert_eq!(
            overrides.get(&("AWS::EC2::VPC", "InstanceTenancy")),
            Some(&TypeOverride::Enum(vec!["default", "dedicated", "host"])),
        );
    }

    #[test]
    fn test_resource_type_overrides_load_balancer_attribute_key_is_plain_string() {
        // awscc#365: LoadBalancerAttribute.Key's description contains legal
        // keys as dotted bullet leaders, plus many backticked value examples in
        // bullet bodies. The key namespace must remain a plain String rather
        // than an enum of those example values.
        let overrides = resource_type_overrides();
        assert_eq!(
            overrides.get(&(
                "AWS::ElasticLoadBalancingV2::LoadBalancer",
                "LoadBalancerAttribute.Key",
            )),
            Some(&TypeOverride::StringType("AttributeType::string()"))
        );
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
        // in backticks. The enum extractor must not infer a value list from those
        // cross-references, and the curated override must still provide the real
        // canonical values. After awscc#365 deleted the unscoped backtick scoop,
        // the heuristic correctly returns None here; the override remains the
        // layer that supplies the canonical TLS version list.
        let description = "If the distribution uses ``Aliases`` (alternate domain \
            names or CNAMEs), specify the security policy that you want CloudFront \
            to use for HTTPS connections with viewers. When you're using SNI only \
            (you set ``SSLSupportMethod`` to ``sni-only``), you must specify \
            ``TLSv1`` or higher. If the distribution uses the CloudFront domain \
            name such as ``d111111abcdef8.cloudfront.net`` (you set \
            ``CloudFrontDefaultCertificate`` to ``true``), CloudFront automatically \
            sets the security policy to ``TLSv1``.";

        let scraped = extract_enum_from_description(description);
        assert_eq!(
            scraped, None,
            "heuristic must not infer sibling values from non-value-list sentences"
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
            definitions: Some(BTreeMap::from([(
                "Tag".to_string(),
                CfnDefinition {
                    def_type: Some("object".to_string()),
                    properties: Some(BTreeMap::from([
                        (
                            "Key".to_string(),
                            CfnProperty {
                                prop_type: Some(TypeValue::Single("string".to_string())),
                                ..Default::default()
                            },
                        ),
                        (
                            "Value".to_string(),
                            CfnProperty {
                                prop_type: Some(TypeValue::Single("string".to_string())),
                                ..Default::default()
                            },
                        ),
                    ])),
                    ..Default::default()
                },
            )])),
            tagging: None,
            one_of: vec![],
            any_of: vec![],
            handlers: BTreeMap::new(),
        };
        assert_eq!(
            type_display_string("ResourceTags", &prop, &schema),
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
        assert_eq!(
            type_display_string("Items", &prop, &schema),
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
        let display = type_display_string("Limit", &prop, &schema);
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
        let display = type_display_string("Weights", &prop, &schema);
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
        assert_eq!(
            type_display_string("SomeList", &prop, &schema),
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
        let result = type_display_string("Prop1", &prop, &schema);
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
        let result = type_display_string("Prop2", &prop, &schema);
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
        let result = type_display_string("Prop3", &prop, &schema);
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
        let result = type_display_string("Prop4", &prop, &schema);
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
    fn test_selected_integer_enum_uses_identified_validator() {
        let retention_values = AwsccType::LogsLogGroupRetentionInDays
            .allowed_int_values()
            .unwrap();
        let prop = CfnProperty {
            prop_type: Some(TypeValue::Single("integer".to_string())),
            description: Some("Retention in days.".to_string()),
            enum_values: Some(
                retention_values
                    .iter()
                    .copied()
                    .map(EnumValue::Int)
                    .collect(),
            ),
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
        assert_eq!(
            type_str,
            "crate::schemas::catalog::logs_log_group_retention_in_days()"
        );

        assert_eq!(
            int_enum_type(
                "AWS::EC2::FlowLog",
                "MaxAggregationInterval",
                AwsccType::Ec2FlowLogMaxAggregationInterval
                    .allowed_int_values()
                    .unwrap()
            ),
            "crate::schemas::catalog::ec2_flow_log_max_aggregation_interval()"
        );

        assert!(
            int_enum_type("AWS::Other::Widget", "Priority", &[1, 3])
                .starts_with("AttributeType::union(vec!["),
            "unselected integer enums must remain anonymous exact-range unions"
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
        let result = type_display_string("SecondaryPrivateIpAddressCount", &prop, &schema);
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
        let result = type_display_string("Priority", &prop, &schema);
        assert_eq!(result, "IntEnum([1, 3, 5])");
    }

    #[test]
    fn test_type_display_string_int_enum_resource_override() {
        // RetentionInDays on AWS::Logs::LogGroup has a resource-scoped IntEnum override
        let retention_values = AwsccType::LogsLogGroupRetentionInDays
            .allowed_int_values()
            .unwrap();
        let prop = CfnProperty {
            prop_type: Some(TypeValue::Single("integer".to_string())),
            description: Some("Retention in days.".to_string()),
            enum_values: Some(
                retention_values
                    .iter()
                    .copied()
                    .map(EnumValue::Int)
                    .collect(),
            ),
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
        // Resource-scoped override takes precedence over schema enum values
        let result = type_display_string("RetentionInDays", &prop, &schema);
        let expected = retention_values
            .iter()
            .map(i64::to_string)
            .collect::<Vec<_>>()
            .join(", ");
        assert_eq!(result, format!("IntEnum([{expected}])"));
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
        let result = type_display_string("Ipv4NetmaskLength", &prop, &schema);
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
    fn arn_constructor_routes_shared_overlay_and_generic_paths() {
        assert_eq!(
            arn_constructor("iam", "OidcProvider").unwrap(),
            "crate::schemas::catalog::iam_oidc_provider_arn()"
        );
        assert_eq!(
            arn_constructor("organizations", "Account").unwrap(),
            "crate::schemas::catalog::organizations_account_arn()"
        );
        assert_eq!(
            arn_constructor("cloudfront", "Distribution").unwrap(),
            "crate::schemas::catalog::cloudfront_distribution_arn()"
        );
        assert_eq!(
            arn_constructor("iam", "Role").unwrap(),
            "carina_aws_types::AwsType::IamRoleArn.attribute_type()"
        );
        assert_eq!(
            arn_constructor("unknownservice", "Thing").unwrap(),
            "carina_aws_types::arn()"
        );
        assert_eq!(
            arn_constructor("scheduler", "Schedule").unwrap(),
            "carina_aws_types::arn()"
        );
    }

    #[test]
    fn review_regression_known_service_arn_requires_a_catalog_row() {
        let type_name = "AWS::EC2::NewCataloglessResource";
        let schema = CfnSchema {
            type_name: type_name.to_string(),
            description: None,
            properties: BTreeMap::from([(
                "Arn".to_string(),
                CfnProperty {
                    prop_type: Some(TypeValue::Single("string".to_string())),
                    ..Default::default()
                },
            )]),
            required: vec![],
            read_only_properties: vec!["/properties/Arn".to_string()],
            create_only_properties: vec![],
            write_only_properties: vec![],
            primary_identifier: None,
            definitions: None,
            tagging: None,
            one_of: vec![],
            any_of: vec![],
            handlers: BTreeMap::new(),
        };
        let error = generate_schema_code(&schema, type_name)
            .expect_err("a cataloged service must require an exact ARN row");
        assert!(
            error.to_string().contains("has no Arn row"),
            "unexpected codegen error: {error:#}"
        );
    }

    #[test]
    fn virtual_arn_module_uses_the_same_catalog_path() {
        let module = generate_arn_helper_module("AWS::IAM::Policy").unwrap();
        assert!(module.contains("carina_aws_types::AwsType::IamPolicyArn.attribute_type()"));
        assert!(!module.contains("carina_aws_types::iam_policy_arn()"));
    }

    #[test]
    fn review_regression_catalog_int_enum_rejects_mismatched_values() {
        let mismatch = std::panic::catch_unwind(|| {
            int_enum_type("AWS::EC2::FlowLog", "MaxAggregationInterval", &[60, 61])
        });
        assert!(
            mismatch.is_err(),
            "catalog-selected integer enums must fail codegen when CFN values drift"
        );
    }

    #[test]
    fn review_regression_declarative_constraints_emit_no_dead_helpers() {
        let schema = CfnSchema {
            type_name: "AWS::Test::Resource".to_string(),
            description: None,
            properties: BTreeMap::from([
                (
                    "Count".to_string(),
                    CfnProperty {
                        prop_type: Some(TypeValue::Single("integer".to_string())),
                        minimum: Some(1),
                        maximum: Some(10),
                        ..Default::default()
                    },
                ),
                (
                    "Label".to_string(),
                    CfnProperty {
                        prop_type: Some(TypeValue::Single("string".to_string())),
                        pattern: Some("^[a-z]+$".to_string()),
                        min_length: Some(1),
                        max_length: Some(10),
                        ..Default::default()
                    },
                ),
            ]),
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

        let generated = generate_synthetic_schema_code(&schema).expect("generate fixture");
        assert!(
            !generated.contains("fn validate_count_range")
                && !generated.contains("fn validate_string_length_")
                && !generated.contains("fn validate_string_pattern_"),
            "declarative range, length, and pattern metadata must not emit dead callbacks: {generated}"
        );
    }

    #[test]
    fn generated_s3_bucket_owns_arn_helper() {
        let schema = cfn_schema_for_codegen_tests("AWS::S3::Bucket");
        let generated =
            generate_schema_code(&schema, "AWS::S3::Bucket").expect("generate s3 bucket");
        assert!(generated.contains("pub fn arn() -> AttributeType"));
        assert!(generated.contains("crate::schemas::catalog::s3_bucket_arn()"));
        assert!(generated.contains("AttributeSchema::provider_populated(\"arn\", self::arn())"));
    }

    #[test]
    fn generated_organizations_account_gets_service_prefix_arn_helper() {
        let schema = cfn_schema_for_codegen_tests("AWS::Organizations::Account");
        let generated = generate_schema_code(&schema, "AWS::Organizations::Account")
            .expect("generate organizations account");
        assert!(generated.contains("pub fn arn() -> AttributeType"));
        assert!(generated.contains("crate::schemas::catalog::organizations_account_arn()"));
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
    fn test_kms_key_policy_object_string_maps_to_iam_policy_document() {
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

        let (carina_type, enum_info) =
            cfn_type_to_carina_type_with_enum(&prop, "KeyPolicy", &schema, "", &BTreeMap::new());

        assert_eq!(carina_type, "carina_aws_types::iam_policy_document()");
        assert!(enum_info.is_none());
    }

    #[test]
    fn test_ecr_repository_policy_text_object_string_maps_to_iam_policy_document() {
        let schema = cfn_schema_for_codegen_tests("AWS::ECR::Repository");
        let prop = schema
            .properties
            .get("RepositoryPolicyText")
            .expect("cached ECR Repository schema should declare RepositoryPolicyText");

        let (carina_type, enum_info) = cfn_type_to_carina_type_with_enum(
            prop,
            "RepositoryPolicyText",
            &schema,
            "",
            &BTreeMap::new(),
        );

        assert_eq!(carina_type, "carina_aws_types::iam_policy_document()");
        assert!(enum_info.is_none());
        assert_eq!(
            type_display_string("RepositoryPolicyText", prop, &schema),
            "PolicyDocument"
        );
    }

    #[test]
    fn test_sso_inline_policy_object_string_maps_to_iam_policy_document() {
        let schema = cfn_schema_for_codegen_tests("AWS::SSO::PermissionSet");
        let prop = schema
            .properties
            .get("InlinePolicy")
            .expect("cached SSO PermissionSet schema should declare InlinePolicy");

        let (carina_type, enum_info) =
            cfn_type_to_carina_type_with_enum(prop, "InlinePolicy", &schema, "", &BTreeMap::new());

        assert_eq!(carina_type, "carina_aws_types::iam_policy_document()");
        assert!(enum_info.is_none());
        assert_eq!(
            type_display_string("InlinePolicy", prop, &schema),
            "PolicyDocument"
        );
    }

    #[test]
    fn test_unlisted_object_string_property_requires_explicit_classification() {
        let prop = CfnProperty {
            prop_type: Some(TypeValue::Multiple(vec![
                "object".to_string(),
                "string".to_string(),
            ])),
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

        let panic = std::panic::catch_unwind(|| {
            cfn_type_to_carina_type_with_enum(&prop, "FreeFormJson", &schema, "", &BTreeMap::new())
        })
        .expect_err("an unlisted object|string property must fail classification");
        let message = panic
            .downcast_ref::<String>()
            .map(String::as_str)
            .or_else(|| panic.downcast_ref::<&str>().copied())
            .expect("classification panic should contain a string message");

        assert!(message.contains("AWS::Test::Resource.FreeFormJson"));
        assert!(message.contains("classify"));
    }

    #[test]
    fn test_unlisted_bare_object_property_requires_explicit_classification() {
        let prop = CfnProperty {
            prop_type: Some(TypeValue::Single("object".to_string())),
            properties: Some(BTreeMap::new()),
            // This exact shape previously reached the bare-object fallback and
            // silently became Map<String, String>.
            additional_properties: None,
            ..Default::default()
        };
        let schema = empty_schema("AWS::Test::Resource");

        let panic = std::panic::catch_unwind(|| {
            cfn_type_to_carina_type_with_enum(&prop, "BareObject", &schema, "", &BTreeMap::new())
        })
        .expect_err("an unlisted bare object property must fail classification");
        let message = panic
            .downcast_ref::<String>()
            .map(String::as_str)
            .or_else(|| panic.downcast_ref::<&str>().copied())
            .expect("classification panic should contain a string message");

        assert!(message.contains("AWS::Test::Resource.BareObject"));
        assert!(message.contains("FREE_FORM_JSON_PROPERTIES"));
        assert!(message.contains("EMPTY_OBJECT_PROPERTIES"));
        assert!(message.contains("STRING_MAP_PROPERTIES"));
        assert!(message.contains("CODEGEN.md#classifying-bare-objects"));
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

        let (carina_type, enum_info) =
            cfn_type_to_carina_type_with_enum(&prop, "KeySchema", &schema, "", &BTreeMap::new());

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

        let _ = cfn_type_to_carina_type_with_enum(
            &prop,
            "UnionProperty",
            &schema,
            "",
            &BTreeMap::new(),
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
        assert_eq!(
            infer_string_type_display(
                "CanonicalHostedZoneID",
                "AWS::ElasticLoadBalancingV2::LoadBalancer"
            ),
            "Route53HostedZoneId"
        );
        assert_eq!(
            infer_string_type_display("Id", "AWS::Route53::HostedZone"),
            "Route53HostedZoneId"
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
            Some(&TypeOverride::IntEnum(
                AwsccType::Ec2FlowLogMaxAggregationInterval
                    .allowed_int_values()
                    .unwrap()
                    .to_vec()
            ))
        );
        assert_eq!(
            overrides.get(&("AWS::Logs::LogGroup", "RetentionInDays")),
            Some(&TypeOverride::IntEnum(
                AwsccType::LogsLogGroupRetentionInDays
                    .allowed_int_values()
                    .unwrap()
                    .to_vec()
            ))
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
                pattern_properties: None,
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

        let generated = generate_synthetic_schema_code(&schema).unwrap();

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
                pattern_properties: None,
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
                pattern_properties: None,
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
        let generated = generate_synthetic_schema_code(&schema).unwrap();
        assert!(
            generated.contains("StructField::required(\"priority\", AttributeType::refined_int("),
            "nested struct field via scalar integer $ref must generate as int: {generated}"
        );
        assert!(
            !generated.contains("StructField::required(\"priority\", AttributeType::string()"),
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
        let generated = generate_synthetic_schema_code(&schema).unwrap();
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
        let generated = generate_synthetic_schema_code(&schema).unwrap();
        assert!(
            generated.contains("AttributeType::list(AttributeType::enum_("),
            "array with enum items should generate list(Enum): {generated}"
        );
    }

    #[test]
    fn test_markdown_struct_description_escapes_table_delimiter() {
        let description = "Indicates whether to enable Ipv6 Support for Vpc Attachment. Valid Values: enable | disable";
        let mut schema = empty_schema("AWS::Test::Resource");
        schema.properties.insert(
            "Options".to_string(),
            CfnProperty {
                ref_path: Some("#/definitions/Options".to_string()),
                ..Default::default()
            },
        );
        schema.definitions = Some(BTreeMap::from([(
            "Options".to_string(),
            def(
                Some("object"),
                Some(BTreeMap::from([(
                    "Ipv6Support".to_string(),
                    CfnProperty {
                        prop_type: Some(TypeValue::Single("string".to_string())),
                        description: Some(description.to_string()),
                        ..Default::default()
                    },
                )])),
                vec![],
                None,
                None,
                None,
            ),
        )]));

        let md = generate_markdown(&schema, &schema.type_name).unwrap();
        let expected = format!(
            "| `ipv6_support` | [Enum (Ipv6Support)](#optionsipv6_support-ipv6support) | No | {} |",
            description.replace('|', "\\|")
        );

        assert!(
            md.lines().any(|line| line == expected),
            "struct description must remain one four-cell table row:\n{md}"
        );
        carina_docgen::check_page(&md).expect("generated Markdown should pass shared checks");
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
                pattern_properties: None,
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
                pattern_properties: None,
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
                pattern_properties: None,
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
                pattern_properties: None,
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
                pattern_properties: None,
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
                pattern_properties: None,
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
        assert!(
            md.contains("### security_group_egress.ip_protocol (IpProtocol)"),
            "Egress enum heading should use the writable DSL field path:\n{md}"
        );
        assert!(
            md.contains("### security_group_ingress.ip_protocol (IpProtocol)"),
            "Ingress enum heading should use the writable DSL field path:\n{md}"
        );
        assert_eq!(
            md.matches("### ip_protocol (IpProtocol)").count(),
            0,
            "Structurally distinct enum identities must not share an ambiguous heading.",
        );
        carina_docgen::check_page(&md).expect("generated Markdown should pass shared checks");
    }

    #[test]
    fn test_markdown_struct_link_uses_allocated_slug_after_attribute_collision() {
        for (type_name, expected) in [
            (
                "AWS::EC2::TransitGatewayAttachment",
                "- **Type:** [Struct(Options)](#options-1)",
            ),
            (
                "AWS::ElasticLoadBalancingV2::TargetGroup",
                "- **Type:** [Struct(Matcher)](#matcher-1)",
            ),
        ] {
            let schema = cfn_schema_for_codegen_tests(type_name);
            let md = generate_markdown(&schema, type_name).unwrap();

            assert!(md.contains(expected), "missing {expected}:\n{md}");
            carina_docgen::check_page(&md).expect("generated Markdown should pass shared checks");
        }
    }

    #[test]
    fn test_markdown_definition_sections_are_sorted_by_visible_heading() {
        let schema = cfn_schema_for_codegen_tests("AWS::CloudFront::Distribution");
        let md = generate_markdown(&schema, "AWS::CloudFront::Distribution").unwrap();

        let enum_section = md
            .split_once("## Enum Values\n\n")
            .expect("enum values section")
            .1
            .split_once("\n## Struct Definitions")
            .expect("struct definitions section")
            .0;
        let struct_rest = md
            .split_once("## Struct Definitions\n\n")
            .expect("struct definitions section")
            .1;
        let struct_section = struct_rest
            .split_once("\n## Attribute Reference")
            .map_or(struct_rest, |(section, _)| section);

        for (name, section) in [("enum", enum_section), ("struct", struct_section)] {
            let headings: Vec<&str> = section
                .lines()
                .filter(|line| line.starts_with("### "))
                .collect();
            let mut sorted = headings.clone();
            sorted.sort_unstable();
            assert_eq!(headings, sorted, "{name} headings were not sorted:\n{md}");
        }
    }

    #[test]
    fn test_markdown_definition_heading_uses_name_when_reached_by_multiple_paths() {
        let struct_definition = |field_name: &str| {
            def(
                Some("object"),
                Some(BTreeMap::from([(
                    field_name.to_string(),
                    CfnProperty {
                        prop_type: Some(TypeValue::Single("string".to_string())),
                        ..Default::default()
                    },
                )])),
                vec![],
                None,
                None,
                None,
            )
        };
        let struct_ref = |definition: &str| CfnProperty {
            ref_path: Some(format!("#/definitions/{definition}")),
            ..Default::default()
        };

        let mut schema = empty_schema("AWS::Test::SharedDefinition");
        schema.properties = BTreeMap::from([
            ("Left".to_string(), struct_ref("SharedOptions")),
            ("Only".to_string(), struct_ref("UniqueOptions")),
            ("Right".to_string(), struct_ref("SharedOptions")),
        ]);
        schema.definitions = Some(BTreeMap::from([
            (
                "SharedOptions".to_string(),
                struct_definition("SharedField"),
            ),
            (
                "UniqueOptions".to_string(),
                struct_definition("UniqueField"),
            ),
        ]));

        let md = generate_markdown(&schema, "AWS::Test::SharedDefinition").unwrap();

        assert_eq!(md.matches("### shared_options\n\n").count(), 1, "{md}");
        assert_eq!(
            md.matches("[Struct(SharedOptions)](#shared_options)")
                .count(),
            2,
            "{md}"
        );
        assert_eq!(md.matches("### only\n\n").count(), 1, "{md}");
        assert_eq!(
            md.matches("[Struct(UniqueOptions)](#only-1)").count(),
            1,
            "{md}"
        );
        assert!(!md.contains("### unique_options\n\n"), "{md}");
        carina_docgen::check_page(&md).expect("generated Markdown should pass shared checks");
    }

    #[test]
    fn test_markdown_shared_definition_anchors_nested_sections_and_lists_uses() {
        let struct_ref = |definition: &str| CfnProperty {
            ref_path: Some(format!("#/definitions/{definition}")),
            ..Default::default()
        };
        let mut schema = empty_schema("AWS::Test::SharedNestedDefinition");
        schema.properties = BTreeMap::from([
            ("Left".to_string(), struct_ref("SharedOptions")),
            ("Right".to_string(), struct_ref("SharedOptions")),
        ]);
        schema.definitions = Some(BTreeMap::from([(
            "SharedOptions".to_string(),
            def(
                Some("object"),
                Some(BTreeMap::from([
                    (
                        "Details".to_string(),
                        CfnProperty {
                            prop_type: Some(TypeValue::Single("object".to_string())),
                            properties: Some(BTreeMap::from([(
                                "Label".to_string(),
                                CfnProperty {
                                    prop_type: Some(TypeValue::Single("string".to_string())),
                                    ..Default::default()
                                },
                            )])),
                            ..Default::default()
                        },
                    ),
                    (
                        "Mode".to_string(),
                        CfnProperty {
                            enum_values: Some(vec![
                                EnumValue::Str("fast".to_string()),
                                EnumValue::Str("slow".to_string()),
                            ]),
                            ..Default::default()
                        },
                    ),
                ])),
                vec![],
                None,
                None,
                None,
            ),
        )]));

        let md = generate_markdown(&schema, "AWS::Test::SharedNestedDefinition").unwrap();

        assert!(
            md.contains("### shared_options\n\nUsed by: `left`, `right`\n\n"),
            "{md}"
        );
        assert!(
            md.contains("| `details` | [Struct(Details)](#shared_optionsdetails) |"),
            "{md}"
        );
        assert!(
            md.contains("| `mode` | [Enum (Mode)](#shared_optionsmode-mode) |"),
            "{md}"
        );
        assert_eq!(
            md.matches("### shared_options.details\n\n").count(),
            1,
            "{md}"
        );
        assert_eq!(
            md.matches("### shared_options.mode (Mode)\n\n").count(),
            1,
            "{md}"
        );
        assert!(!md.contains("### left.details\n\n"), "{md}");
        assert!(!md.contains("### left.mode (Mode)\n\n"), "{md}");
        carina_docgen::check_page(&md).expect("generated Markdown should pass shared checks");
    }

    #[test]
    fn test_markdown_nested_shared_definition_omits_single_resolved_use() {
        let struct_ref = |definition: &str| CfnProperty {
            ref_path: Some(format!("#/definitions/{definition}")),
            ..Default::default()
        };
        let mut schema = empty_schema("AWS::Test::NestedSharedDefinition");
        schema.properties = BTreeMap::from([
            ("Left".to_string(), struct_ref("Outer")),
            ("Right".to_string(), struct_ref("Outer")),
        ]);
        schema.definitions = Some(BTreeMap::from([
            (
                "Inner".to_string(),
                def(
                    Some("object"),
                    Some(BTreeMap::from([(
                        "Value".to_string(),
                        CfnProperty {
                            prop_type: Some(TypeValue::Single("string".to_string())),
                            ..Default::default()
                        },
                    )])),
                    vec![],
                    None,
                    None,
                    None,
                ),
            ),
            (
                "Outer".to_string(),
                def(
                    Some("object"),
                    Some(BTreeMap::from([(
                        "InnerField".to_string(),
                        struct_ref("Inner"),
                    )])),
                    vec![],
                    None,
                    None,
                    None,
                ),
            ),
        ]));

        let md = generate_markdown(&schema, "AWS::Test::NestedSharedDefinition").unwrap();

        assert!(md.contains("### outer.inner_field\n\n| Field |"), "{md}");
        assert!(!md.contains("Used by: `outer.inner_field`"), "{md}");
        assert!(!md.contains("Used by: `left.inner_field`"), "{md}");
        assert!(!md.contains("Used by: `right.inner_field`"), "{md}");
        carina_docgen::check_page(&md).expect("generated Markdown should pass shared checks");
    }

    #[test]
    fn test_markdown_used_by_paths_follow_all_shared_ancestor_paths() {
        let schema = cfn_schema_for_codegen_tests("AWS::WAFv2::WebACL");
        let md = generate_markdown(&schema, "AWS::WAFv2::WebACL").unwrap();

        assert!(
            md.contains(concat!(
                "Used by: `allow_action.custom_request_handling`, ",
                "`rule_action.captcha.custom_request_handling`, ",
                "`rule_action.challenge.custom_request_handling`, ",
                "`rule_action.count.custom_request_handling`"
            )),
            "{md}"
        );
        assert!(
            md.contains(concat!(
                "Used by: `block_action.custom_response.response_headers`, ",
                "`custom_request_handling.insert_headers`"
            )),
            "{md}"
        );
        assert!(
            md.contains("Used by: `statement.byte_match_statement.text_transformations`"),
            "{md}"
        );
        carina_docgen::check_page(&md).expect("generated Markdown should pass shared checks");
    }

    #[test]
    fn test_markdown_example_is_preprocessed_and_validated_inside_codegen() {
        let source = concat!(
            "\n",
            "# leading comment\n",
            "provider awscc {\n",
            "  region = \"us-east-1\"\n",
            "}\n",
            "\n",
            "resource transit_gateway_attachment \"fixture\" {}\n",
            "# trailing comment\n",
        );
        let example = preprocess_markdown_example(source);
        assert_eq!(
            example,
            "resource transit_gateway_attachment \"fixture\" {}\n"
        );

        let schema = cfn_schema_for_codegen_tests("AWS::EC2::TransitGatewayAttachment");
        let md = generate_markdown_with_example(
            &schema,
            "AWS::EC2::TransitGatewayAttachment",
            Some(&example),
        )
        .unwrap();

        let example_position = md.find("## Example\n\n```crn\n").expect("example heading");
        let arguments_position = md.find("## Argument Reference").expect("argument heading");
        assert!(example_position < arguments_position, "{md}");
        assert!(md.contains(&format!("```crn\n{example}```\n\n")), "{md}");
        carina_docgen::check_page(&md).expect("the page checked by codegen should be valid");
    }

    #[test]
    fn test_load_markdown_example_rejects_non_directory() {
        let not_directory = tempfile::NamedTempFile::new().unwrap();

        let error = load_markdown_example(
            Some(not_directory.path()),
            "AWS::EC2::TransitGatewayAttachment",
        )
        .unwrap_err();

        assert_eq!(
            error.to_string(),
            format!(
                "--examples-dir must be an existing directory: {}",
                not_directory.path().display()
            )
        );
    }

    #[test]
    fn test_markdown_collects_struct_below_ref_to_array_definition() {
        let schema = cfn_schema_for_codegen_tests("AWS::WAFv2::WebACL");

        let md = generate_markdown(&schema, "AWS::WAFv2::WebACL")
            .expect("array-definition item structs should have target headings");

        assert!(
            md.contains("### application_config.attributes\n\n| Field |"),
            "{md}"
        );
        carina_docgen::check_page(&md).expect("generated Markdown should pass shared checks");
    }

    #[test]
    fn test_markdown_distinguishes_same_named_inline_structs_by_field_path() {
        fn parent(option_field: &str, option_type: &str, mode_values: &[&str]) -> CfnProperty {
            CfnProperty {
                prop_type: Some(TypeValue::Single("object".to_string())),
                properties: Some(BTreeMap::from([(
                    "Options".to_string(),
                    CfnProperty {
                        prop_type: Some(TypeValue::Single("object".to_string())),
                        properties: Some(BTreeMap::from([
                            (
                                "Mode".to_string(),
                                CfnProperty {
                                    enum_values: Some(
                                        mode_values
                                            .iter()
                                            .map(|value| EnumValue::Str((*value).to_string()))
                                            .collect(),
                                    ),
                                    ..Default::default()
                                },
                            ),
                            (
                                option_field.to_string(),
                                CfnProperty {
                                    prop_type: Some(TypeValue::Single(option_type.to_string())),
                                    ..Default::default()
                                },
                            ),
                        ])),
                        ..Default::default()
                    },
                )])),
                ..Default::default()
            }
        }

        let mut schema = empty_schema("AWS::Test::InlineOptions");
        schema.properties = BTreeMap::from([
            ("Left".to_string(), parent("Alpha", "string", &["a", "b"])),
            (
                "Right".to_string(),
                parent("Retries", "integer", &["x", "y"]),
            ),
        ]);

        let md = generate_markdown(&schema, "AWS::Test::InlineOptions").unwrap();

        assert!(md.contains("[Struct(Options)](#leftoptions)"), "{md}");
        assert!(md.contains("[Struct(Options)](#rightoptions)"), "{md}");
        assert_eq!(md.matches("### left.options\n\n").count(), 1, "{md}");
        assert_eq!(md.matches("### right.options\n\n").count(), 1, "{md}");
        assert!(md.contains("| `alpha` | String |"), "{md}");
        assert!(md.contains("| `retries` | Int |"), "{md}");
        assert_eq!(
            md.matches("### left.options.mode (Mode)\n\n").count(),
            1,
            "{md}"
        );
        assert_eq!(
            md.matches("### right.options.mode (Mode)\n\n").count(),
            1,
            "{md}"
        );
        assert!(
            md.contains("| `mode` | [Enum (Mode)](#leftoptionsmode-mode) |"),
            "{md}"
        );
        assert!(
            md.contains("| `mode` | [Enum (Mode)](#rightoptionsmode-mode) |"),
            "{md}"
        );
        assert!(
            md.contains("aws.test.InlineOptions.Left.Options.Mode.a"),
            "{md}"
        );
        assert!(
            md.contains("aws.test.InlineOptions.Right.Options.Mode.x"),
            "{md}"
        );
        carina_docgen::check_page(&md).expect("generated Markdown should pass shared checks");
    }

    #[test]
    fn test_markdown_collects_one_of_struct_referenced_by_array_item() {
        let mut schema = empty_schema("AWS::Test::OneOfArray");
        schema.properties.insert(
            "Entries".to_string(),
            CfnProperty {
                prop_type: Some(TypeValue::Single("array".to_string())),
                items: Some(Box::new(CfnProperty {
                    ref_path: Some("#/definitions/Choice".to_string()),
                    ..Default::default()
                })),
                ..Default::default()
            },
        );
        schema.definitions = Some(BTreeMap::from([(
            "Choice".to_string(),
            CfnDefinition {
                one_of: vec![
                    CfnProperty {
                        properties: Some(BTreeMap::from([(
                            "Alpha".to_string(),
                            CfnProperty {
                                prop_type: Some(TypeValue::Single("string".to_string())),
                                ..Default::default()
                            },
                        )])),
                        ..Default::default()
                    },
                    CfnProperty {
                        properties: Some(BTreeMap::from([(
                            "Retries".to_string(),
                            CfnProperty {
                                prop_type: Some(TypeValue::Single("integer".to_string())),
                                ..Default::default()
                            },
                        )])),
                        ..Default::default()
                    },
                ],
                ..Default::default()
            },
        )]));

        let md = generate_markdown(&schema, "AWS::Test::OneOfArray")
            .expect("a linked OneOf array item must have a registered target heading");

        assert!(
            md.contains("- **Type:** List\\<[Struct(Choice)](#entries-1)\\>"),
            "{md}"
        );
        assert!(md.contains("### entries\n\n| Field |"), "{md}");
        assert!(md.contains("| `alpha` | String |"), "{md}");
        assert!(md.contains("| `retries` | Int |"), "{md}");
        carina_docgen::check_page(&md).expect("generated Markdown should pass shared checks");
    }

    #[test]
    fn test_markdown_direct_non_array_one_of_keeps_string_precedence() {
        let mut schema = empty_schema("AWS::Test::DirectOneOf");
        schema.properties.insert(
            "Choice".to_string(),
            CfnProperty {
                one_of: vec![CfnProperty {
                    properties: Some(BTreeMap::from([(
                        "Alpha".to_string(),
                        CfnProperty {
                            prop_type: Some(TypeValue::Single("string".to_string())),
                            ..Default::default()
                        },
                    )])),
                    ..Default::default()
                }],
                ..Default::default()
            },
        );

        let md = generate_markdown(&schema, "AWS::Test::DirectOneOf").unwrap();

        assert!(md.contains("### `choice`\n\n- **Type:** String\n"), "{md}");
        assert!(!md.contains("## Struct Definitions"), "{md}");
    }

    #[test]
    fn test_markdown_plain_values_never_contain_fragment_links() {
        let struct_definition = CfnDefinition {
            def_type: Some("object".to_string()),
            properties: Some(BTreeMap::from([(
                "Value".to_string(),
                CfnProperty {
                    prop_type: Some(TypeValue::Single("string".to_string())),
                    ..Default::default()
                },
            )])),
            ..Default::default()
        };
        let mut schema = empty_schema("AWS::Test::PlainTypes");
        schema.definitions = Some(BTreeMap::from([("Options".to_string(), struct_definition)]));
        let enum_prop = CfnProperty {
            prop_type: Some(TypeValue::Single("string".to_string())),
            ..Default::default()
        };
        let struct_ref = CfnProperty {
            ref_path: Some("#/definitions/Options".to_string()),
            ..Default::default()
        };
        let struct_list = CfnProperty {
            prop_type: Some(TypeValue::Single("array".to_string())),
            items: Some(Box::new(struct_ref.clone())),
            ..Default::default()
        };
        let inline_struct = CfnProperty {
            prop_type: Some(TypeValue::Single("object".to_string())),
            properties: Some(BTreeMap::from([(
                "Value".to_string(),
                CfnProperty {
                    prop_type: Some(TypeValue::Single("string".to_string())),
                    ..Default::default()
                },
            )])),
            ..Default::default()
        };
        let plain_values = [
            MarkdownType::Plain(type_display_string("Mode", &enum_prop, &schema)),
            MarkdownType::Plain(type_display_string("Options", &struct_ref, &schema)),
            MarkdownType::Plain(type_display_string("OptionsList", &struct_list, &schema)),
            MarkdownType::Plain(type_display_string(
                "InlineOptions",
                &inline_struct,
                &schema,
            )),
        ];

        let offenders: Vec<&str> = plain_values
            .iter()
            .filter_map(|value| match value {
                MarkdownType::Plain(value) if value.contains("](#") => Some(value.as_str()),
                _ => None,
            })
            .collect();
        assert!(
            offenders.is_empty(),
            "MarkdownType::Plain contained fragment links: {offenders:?}"
        );
    }

    #[test]
    fn test_truly_identical_nested_enum_identity_is_emitted_once() {
        let protocol = CfnDefinition {
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
            pattern_properties: None,
            format: None,
        };
        let shared_protocol = || CfnProperty {
            ref_path: Some("#/definitions/Protocol".to_string()),
            ..Default::default()
        };
        let rule = CfnDefinition {
            def_type: Some("object".to_string()),
            properties: Some(BTreeMap::from([
                ("DestinationProtocol".to_string(), shared_protocol()),
                ("SourceProtocol".to_string(), shared_protocol()),
            ])),
            required: vec![],
            enum_values: None,
            items: None,
            one_of: vec![],
            pattern: None,
            min_length: None,
            max_length: None,
            minimum: None,
            maximum: None,
            pattern_properties: None,
            format: None,
        };
        let schema = CfnSchema {
            type_name: "AWS::Test::Resource".to_string(),
            description: None,
            properties: BTreeMap::from([(
                "Rules".to_string(),
                CfnProperty {
                    ref_path: Some("#/definitions/Rule".to_string()),
                    ..Default::default()
                },
            )]),
            required: vec![],
            read_only_properties: vec![],
            create_only_properties: vec![],
            write_only_properties: vec![],
            primary_identifier: None,
            definitions: Some(BTreeMap::from([
                ("Protocol".to_string(), protocol),
                ("Rule".to_string(), rule),
            ])),
            tagging: None,
            one_of: vec![],
            any_of: vec![],
            handlers: BTreeMap::new(),
        };

        let md = generate_markdown(&schema, "AWS::Test::Resource").unwrap();

        assert_eq!(
            md.matches("### rules.destination_protocol (Protocol)")
                .count(),
            1,
            "one runtime enum identity should have one section:\n{md}"
        );
        assert_eq!(
            md.matches("(#rulesdestination_protocol-protocol)").count(),
            2,
            "both struct fields should link to that one section:\n{md}"
        );
        assert_eq!(
            md.matches("aws.test.Resource.Rule.Protocol.tcp").count(),
            1,
            "the shared enum table should be emitted once:\n{md}"
        );
        carina_docgen::check_page(&md).expect("generated Markdown should pass shared checks");
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
                pattern_properties: None,
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
                pattern_properties: None,
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
                pattern_properties: None,
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

        let generated = generate_synthetic_schema_code(&schema).unwrap();

        assert!(
            generated.contains(
                r#".with_default(Value::Concrete(ConcreteValue::String("/".to_string().into())))"#
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

        let generated = generate_synthetic_schema_code(&schema).unwrap();

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

        let generated = generate_synthetic_schema_code(&schema).unwrap();

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
            "AttributeType::refined_int_with_validator(identity, None, legacy_validator(f))"
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
                r#"AttributeSchema::optional("key_policy", carina_aws_types::iam_policy_document())"#
            ),
            "Should resolve KeyPolicy to iam_policy_document: {generated}"
        );
        assert!(
            !generated.contains(".with_default("),
            "Should not emit scalar JSON defaults for structured policy documents: {generated}"
        );
    }

    #[test]
    fn test_generate_markdown_top_level_enum_override_renders_table() {
        // awscc#357: top-level `Enum` entries in `resource_type_overrides`
        // must show up in the markdown's `## Enum Values` section. Before
        // this fix the nested-struct loop consulted `resource_type_overrides`
        // but the top-level attribute loop did not, so `VPNGateway.Type`
        // (and the new `TargetGroup.{Protocol,TargetType}`) were silently
        // rendered as plain `String` even though the generated schema
        // correctly typed them as enums.
        let mut properties = BTreeMap::new();
        properties.insert(
            "Type".to_string(),
            CfnProperty {
                prop_type: Some(TypeValue::Single("string".to_string())),
                description: Some(
                    "The type of VPN connection the virtual private gateway supports.".to_string(),
                ),
                ..Default::default()
            },
        );

        let schema = CfnSchema {
            type_name: "AWS::EC2::VPNGateway".to_string(),
            description: None,
            properties,
            required: vec!["Type".to_string()],
            read_only_properties: vec![],
            create_only_properties: vec!["/properties/Type".to_string()],
            write_only_properties: vec![],
            primary_identifier: None,
            definitions: None,
            tagging: None,
            one_of: vec![],
            any_of: vec![],
            handlers: BTreeMap::new(),
        };

        let md = generate_markdown(&schema, "AWS::EC2::VPNGateway").unwrap();

        assert!(
            md.contains("[Enum (Type)](#type-type)"),
            "top-level enum override should render as an Enum link, not String: {md}"
        );
        assert!(
            md.contains("## Enum Values"),
            "top-level enum override should add an Enum Values section: {md}"
        );
        assert!(
            md.contains("`ipsec.1`"),
            "the enum value list should be rendered: {md}"
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
        // Tags property should display Map<String, String>
        assert_eq!(
            type_display_string("Tags", &prop, &schema),
            "`Map<String, String>`"
        );
    }

    #[test]
    fn test_type_display_string_pattern_map_shows_map_string() {
        let prop = CfnProperty {
            prop_type: Some(TypeValue::Single("object".to_string())),
            pattern_properties: Some(BTreeMap::from([(
                "[a-zA-Z0-9]+".to_string(),
                CfnProperty {
                    prop_type: Some(TypeValue::Single("string".to_string())),
                    ..Default::default()
                },
            )])),
            ..Default::default()
        };
        let schema = CfnSchema {
            type_name: "AWS::ElasticLoadBalancingV2::Listener".to_string(),
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
        // A schema-declared homogeneous map remains Map<String, String>.
        assert_eq!(
            type_display_string("AuthenticationRequestExtraParams", &prop, &schema),
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
    fn test_markdown_skips_deprecated_read_only_top_level_object() {
        let mut schema = empty_schema("AWS::Test::DeprecatedReadOnly");
        schema.properties.insert(
            "Info".to_string(),
            CfnProperty {
                prop_type: Some(TypeValue::Single("object".to_string())),
                description: Some("(Deprecated.) Legacy provider output.".to_string()),
                properties: Some(BTreeMap::from([(
                    "Value".to_string(),
                    CfnProperty {
                        prop_type: Some(TypeValue::Single("string".to_string())),
                        ..Default::default()
                    },
                )])),
                ..Default::default()
            },
        );
        schema.read_only_properties = vec!["/properties/Info".to_string()];

        let md = generate_markdown(&schema, "AWS::Test::DeprecatedReadOnly")
            .expect("deprecated read-only properties must not create unresolved links");

        assert!(!md.contains("### `info`"), "{md}");
        assert!(!md.contains("Struct(Info)"), "{md}");
        assert!(!md.contains("## Attribute Reference"), "{md}");
        carina_docgen::check_page(&md).expect("generated Markdown should pass shared checks");
    }

    #[test]
    fn test_json_default_to_value_code() {
        assert_eq!(
            json_default_to_value_code(&serde_json::Value::String("hello".to_string())),
            Some(
                "Value::Concrete(ConcreteValue::String(\"hello\".to_string().into()))".to_string()
            )
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
    fn test_pattern_constraint_is_declarative_in_schema_code() {
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
            code.contains("AttributeType::refined_string(None, Some(")
                && code.contains(r"{1,512}\\Z"),
            "Generated code should retain the pattern as type metadata:\n{}",
            code
        );
        assert!(
            !code.contains("fn validate_string_pattern_") && !code.contains("Regex::new"),
            "Pattern metadata should not emit an unreferenced callback:\n{}",
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
            !type_str.contains("validate_list_items_1_10"),
            "refined-list bounds are validated by the schema type itself: {type_str}"
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
            type_str.contains("AttributeType::string(), true, Some((Some(1), None))"),
            "refined list should carry its minimum length directly: {type_str}"
        );
        assert!(
            !type_str.contains("validate_list_items_min_1"),
            "refined-list bounds are validated by the schema type itself: {type_str}"
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
    fn test_generate_schema_code_emits_list_length_without_callback() {
        // Test that generate_schema_code emits declarative length bounds for array
        // properties with minItems/maxItems constraints, without a callback.
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
                pattern_properties: None,
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
        assert!(generated.contains("Some((Some(1), Some(10)))"));
        assert!(!generated.contains("validate_list_items_"));
        assert!(!generated.contains("ConcreteValue::List"));
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

        assert!(
            !generated.contains("validate_string_length_"),
            "Should not generate a dead length validation function: {generated}"
        );

        // Should use AttributeType::Custom wrapping String
        assert!(
            generated.contains(
                "AttributeType::refined_string(None, None, Some((Some(1), Some(512))), None)"
            ),
            "Should wrap String in Custom type with length constraint: {generated}"
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

        let generated = generate_synthetic_schema_code(&schema).unwrap();

        assert!(
            generated
                .contains("AttributeType::refined_string(None, None, Some((Some(1), None)), None)")
                && !generated.contains("validate_string_length_"),
            "Should retain a declarative minimum without a helper: {generated}"
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

        let generated = generate_synthetic_schema_code(&schema).unwrap();

        assert!(
            generated.contains(
                "AttributeType::refined_string(None, None, Some((None, Some(256))), None)"
            ) && !generated.contains("validate_string_length_"),
            "Should retain a declarative maximum without a helper: {generated}"
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
        // A struct field using $ref retains the definition's pattern metadata.
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

        let generated = generate_synthetic_schema_code(&schema).unwrap();
        assert!(
            generated.contains("AttributeType::refined_string(None, Some(")
                && generated.contains(r#"^(\\d{4})"#),
            "Should retain $ref pattern metadata: {generated}"
        );
        assert!(
            !generated.contains("fn validate_string_pattern_"),
            "Should not emit a dead $ref pattern callback: {generated}"
        );
    }

    #[test]
    fn test_top_level_ref_pattern_collected_in_generate_schema_code() {
        // A top-level property using $ref retains the definition's pattern.
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

        let generated = generate_synthetic_schema_code(&schema).unwrap();
        assert!(
            generated.contains("AttributeType::refined_string(None, Some(")
                && generated.contains(r"(\\d{4})"),
            "Should retain top-level $ref pattern metadata: {generated}"
        );
        assert!(
            !generated.contains("fn validate_string_pattern_"),
            "Should not emit a dead top-level $ref callback: {generated}"
        );
    }

    #[test]
    fn test_repeated_string_lengths_stay_declarative() {
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

        let generated = generate_synthetic_schema_code(&schema).unwrap();

        let constraint =
            "AttributeType::refined_string(None, None, Some((None, Some(1024))), None)";
        assert_eq!(
            generated.matches(constraint).count(),
            2,
            "Both properties should carry the declarative bound: {generated}"
        );
        assert!(
            !generated.contains("fn validate_string_length_"),
            "Repeated bounds should not emit a helper: {generated}"
        );
    }

    #[test]
    fn test_non_standard_tag_definition_keeps_declarative_lengths() {
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

        assert!(
            generated.contains(
                "AttributeType::refined_string(None, None, Some((None, Some(128))), None)"
            ),
            "HostedZoneTag.Key should retain maxLength 128: {generated}"
        );
        assert!(
            generated.contains(
                "AttributeType::refined_string(None, None, Some((None, Some(256))), None)"
            ),
            "HostedZoneTag.Value should retain maxLength 256: {generated}"
        );
        assert!(
            !generated.contains("fn validate_string_length_"),
            "Non-standard tags should not emit dead helpers: {generated}"
        );
    }

    #[test]
    fn test_route53_hosted_zone_name_emits_refined_string_strip_suffix() {
        let schema = cfn_schema_for_codegen_tests("AWS::Route53::HostedZone");
        let generated = generate_schema_code(&schema, "AWS::Route53::HostedZone").unwrap();

        assert!(
            generated.contains(
                r#"AttributeSchema::optional("name", AttributeType::refined_string(None, None, Some((None, Some(1024))), Some(carina_core::schema::DslTransform::StripSuffix(".".to_string()))))"#
            ),
            "Route53 HostedZone.Name should be refined String with max length and StripSuffix transform: {generated}"
        );
        assert!(
            !generated.contains(r#"AttributeSchema::optional("name", AttributeType::enum_"#),
            "Route53 HostedZone.Name must not be emitted as a pseudo enum: {generated}"
        );
    }

    #[test]
    fn test_string_length_validation_is_declarative() {
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

        let generated = generate_synthetic_schema_code(&schema).unwrap();

        assert!(
            generated.contains(
                "AttributeType::refined_string(None, None, Some((None, Some(10))), None)"
            ),
            "String length should remain declarative: {generated}"
        );
        assert!(
            !generated.contains("fn validate_string_length_"),
            "String length should not emit a dead helper: {generated}"
        );
    }

    #[test]
    fn test_repeated_patterns_stay_declarative() {
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

        let generated = generate_synthetic_schema_code(&schema).unwrap();

        assert_eq!(
            generated.matches(r#"Some("[0-9]+".to_string())"#).count(),
            2,
            "Both properties should retain their pattern metadata: {generated}"
        );
        assert!(
            !generated.contains("fn validate_string_pattern_"),
            "Repeated patterns should not emit a helper: {generated}"
        );
    }

    #[test]
    fn test_does_not_emit_list_validation_functions() {
        // Multiple arrays retain their declarative minItems/maxItems constraints without
        // generating either per-property or shared callback functions.
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
                pattern_properties: None,
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

        let generated = generate_synthetic_schema_code(&schema).unwrap();

        // Should NOT have per-property items validation functions
        assert!(
            !generated.contains("fn validate_allowed_origins_items"),
            "Should not generate per-property items function: {generated}"
        );
        assert!(
            !generated.contains("fn validate_allowed_headers_items"),
            "Should not generate per-property items function: {generated}"
        );

        // Length constraints are declarative; no shared callback is emitted.
        let fn_count = generated.matches("fn validate_list_items_").count();
        assert_eq!(
            fn_count, 0,
            "Should not generate list validation functions, got {fn_count}: {generated}"
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
        let display = type_display_string("SomeValue", &prop, &schema);
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
        let display = type_display_string("Url", &prop, &schema);
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
        let display = type_display_string("ObjectSizeGreaterThan", &prop, &schema);
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
        let display = type_display_string("Value", &prop, &schema);
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
    fn test_array_items_pattern_with_length_stays_declarative() {
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

        let code = generate_synthetic_schema_code(&schema).expect("codegen should succeed");

        assert!(
            code.contains(
                r#"AttributeType::refined_string(None, Some("[0-9A-Fa-f]{40}".to_string()), Some((Some(40), Some(40))), None)"#
            ),
            "Array items should retain both constraints, got:\n{code}"
        );
        assert!(
            !code.contains("fn validate_string_pattern_")
                && !code.contains("fn validate_string_length_"),
            "Array item constraints should not emit dead helpers"
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

        let generated = generate_synthetic_schema_code(&schema).unwrap();

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

    fn tag_schema(tags_property: &str) -> CfnSchema {
        serde_json::from_str(&format!(
            r#"{{
                "typeName": "AWS::Test::Resource",
                "properties": {{"Tags": {tags_property}}},
                "tagging": {{
                    "taggable": true,
                    "tagProperty": "/properties/Tags"
                }}
            }}"#
        ))
        .unwrap()
    }

    #[test]
    fn taggable_object_map_uses_map_wire_shape() {
        let schema = tag_schema(
            r#"{
                "type": "object",
                "patternProperties": {".*": {"type": "string"}},
                "additionalProperties": false
            }"#,
        );

        let tags = derive_tags_property(&schema).unwrap().unwrap();

        assert_eq!(tags.provider_name, "Tags");
        assert_eq!(tags.wire, GeneratedTagsWire::Map);
        let (attribute_type, enum_info) = cfn_type_to_carina_type_with_enum(
            schema.properties.get("Tags").unwrap(),
            "Tags",
            &schema,
            "aws.test.Resource",
            &BTreeMap::new(),
        );
        assert_eq!(attribute_type, "carina_aws_types::tags_type()");
        assert!(enum_info.is_none());
    }

    #[test]
    fn taggable_key_value_array_uses_key_value_list_wire_shape() {
        let schema = tag_schema(
            r#"{
                "type": "array",
                "items": {
                    "type": "object",
                    "properties": {
                        "Key": {"type": "string"},
                        "Value": {"type": "string"}
                    },
                    "required": ["Key", "Value"]
                }
            }"#,
        );

        let tags = derive_tags_property(&schema).unwrap().unwrap();

        assert_eq!(tags.provider_name, "Tags");
        assert_eq!(tags.wire, GeneratedTagsWire::KeyValueList);
    }

    #[test]
    fn taggable_unknown_tag_property_shape_is_an_error() {
        let schema = tag_schema(r#"{"type": "string"}"#);

        let error = derive_tags_property(&schema)
            .expect_err("unsupported tag property shape must fail codegen");

        assert!(error.to_string().contains("AWS::Test::Resource"));
        assert!(error.to_string().contains("Tags"));
    }

    #[test]
    fn taggable_non_dsl_tag_property_has_no_tags_metadata() {
        let schema: CfnSchema = serde_json::from_str(
            r#"{
                "typeName": "AWS::Route53::HostedZone",
                "properties": {
                    "HostedZoneTags": {
                        "type": "array",
                        "items": {
                            "type": "object",
                            "properties": {
                                "Key": {"type": "string"},
                                "Value": {"type": "string"}
                            }
                        }
                    }
                },
                "tagging": {
                    "taggable": true,
                    "tagProperty": "/properties/HostedZoneTags"
                }
            }"#,
        )
        .unwrap();

        assert_eq!(derive_tags_property(&schema).unwrap(), None);
    }

    fn schema_with_updatable_properties(names: &[&str]) -> CfnSchema {
        let mut schema = empty_schema("AWS::Test::Resource");
        schema.properties = names
            .iter()
            .map(|name| {
                (
                    (*name).to_string(),
                    CfnProperty {
                        prop_type: Some(TypeValue::Single("string".to_string())),
                        ..Default::default()
                    },
                )
            })
            .collect();
        schema
    }

    #[test]
    fn mutually_exclusive_group_with_fewer_than_two_members_is_rejected() {
        let schema = schema_with_updatable_properties(&["Only"]);

        let error = validate_mutually_exclusive_property_groups(&schema, &[&["Only"]])
            .expect_err("a one-member group must fail codegen");

        assert!(error.to_string().contains("expected at least 2"));
    }

    #[test]
    fn mutually_exclusive_group_member_must_be_a_top_level_property() {
        let schema = schema_with_updatable_properties(&["Present"]);

        let error =
            validate_mutually_exclusive_property_groups(&schema, &[&["Present", "Missing"]])
                .expect_err("a missing group member must fail codegen");

        assert!(error.to_string().contains("Missing"));
        assert!(
            error
                .to_string()
                .contains("not a top-level schema property")
        );
    }

    #[test]
    fn mutually_exclusive_group_member_must_not_be_read_only() {
        let mut schema = schema_with_updatable_properties(&["First", "Second"]);
        schema.read_only_properties = vec!["/properties/First".to_string()];

        let error = validate_mutually_exclusive_property_groups(&schema, &[&["First", "Second"]])
            .expect_err("a read-only group member must fail codegen");

        assert!(error.to_string().contains("First"));
        assert!(error.to_string().contains("read-only and not updatable"));
    }

    #[test]
    fn mutually_exclusive_group_member_must_not_be_create_only() {
        let mut schema = schema_with_updatable_properties(&["First", "Second"]);
        schema.create_only_properties = vec!["/properties/Second".to_string()];

        let error = validate_mutually_exclusive_property_groups(&schema, &[&["First", "Second"]])
            .expect_err("a create-only group member must fail codegen");

        assert!(error.to_string().contains("Second"));
        assert!(error.to_string().contains("create-only and not updatable"));
    }

    #[test]
    fn ec2_route_mutually_exclusive_group_is_valid_and_uses_dsl_names() {
        let schema: CfnSchema = serde_json::from_str(include_str!(
            "../../../cfn-schema-cache/AWS__EC2__Route.json"
        ))
        .expect("cached Route schema should deserialize");

        let groups = derive_mutually_exclusive_property_groups(&schema)
            .expect("the curated Route target group should be valid");

        assert_eq!(
            groups,
            [vec![
                "carrier_gateway_id",
                "core_network_arn",
                "egress_only_internet_gateway_id",
                "gateway_id",
                "instance_id",
                "local_gateway_id",
                "nat_gateway_id",
                "network_interface_id",
                "transit_gateway_id",
                "vpc_endpoint_id",
                "vpc_peering_connection_id",
            ]]
        );
        let generated = generate_schema_code(&schema, "AWS::EC2::Route")
            .expect("Route schema generation should succeed");
        assert!(
            generated.contains("use crate::schemas::config::{AwsccSchemaConfig, ExclusiveGroup};")
        );
        assert!(generated.contains(
            "ExclusiveGroup { attributes: &[\"carrier_gateway_id\", \"core_network_arn\""
        ));
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

        let code = generate_synthetic_schema_code(&schema).unwrap();

        // Password should have .write_only()
        assert!(
            code.contains(".write_only()"),
            "Should emit .write_only() for Password: {code}"
        );
        // Name should NOT have .write_only()
        let name_section = code
            .split("AttributeSchema::optional(\"name\"")
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

        let code = generate_synthetic_schema_code(&schema).unwrap();

        assert!(
            code.contains(
                r#"StructField::optional("secret_field", AttributeType::string()).write_only()"#
            ),
            "the matched nested field must be write-only: {code}"
        );
        // Config should NOT have .write_only() because only its child is write-only.
        assert!(
            !code.contains("\n                .write_only()"),
            "Config should not be write-only when only its child is write-only: {code}"
        );
        assert!(
            code.matches(".write_only()").count() == 1,
            "Parent attribute should NOT be marked write-only when only nested \
             sub-properties are write-only. Generated code:\n{code}"
        );
    }

    #[test]
    fn test_deprecated_nested_write_only_property_is_skipped_without_error() {
        let mut schema = empty_schema("AWS::Test::DeprecatedWriteOnly");
        schema.properties.insert(
            "Config".to_string(),
            CfnProperty {
                prop_type: Some(TypeValue::Single("object".to_string())),
                properties: Some(BTreeMap::from([(
                    "LegacySecret".to_string(),
                    CfnProperty {
                        prop_type: Some(TypeValue::Single("string".to_string())),
                        description: Some("(Deprecated.) Legacy write input.".to_string()),
                        ..Default::default()
                    },
                )])),
                ..Default::default()
            },
        );
        schema.write_only_properties = vec!["/properties/Config/LegacySecret".to_string()];

        for (generator, result) in [
            ("Rust", generate_synthetic_schema_code(&schema)),
            (
                "Markdown",
                generate_markdown(&schema, "AWS::Test::DeprecatedWriteOnly"),
            ),
        ] {
            let generated =
                result.expect("a deprecated write-only target is intentionally skipped");
            assert!(
                !generated.contains("legacy_secret"),
                "{generator} must not expose the deprecated write alias: {generated}"
            );
        }
    }

    #[test]
    fn test_nested_write_only_path_through_array_wildcard_is_provider_omitted() {
        let mut schema = empty_schema("AWS::Test::NestedWriteOnlyArray");
        schema.properties.insert(
            "Items".to_string(),
            CfnProperty {
                prop_type: Some(TypeValue::Single("array".to_string())),
                items: Some(Box::new(CfnProperty {
                    prop_type: Some(TypeValue::Single("object".to_string())),
                    properties: Some(BTreeMap::from([
                        (
                            "Secret".to_string(),
                            CfnProperty {
                                prop_type: Some(TypeValue::Single("string".to_string())),
                                ..Default::default()
                            },
                        ),
                        (
                            "Visible".to_string(),
                            CfnProperty {
                                prop_type: Some(TypeValue::Single("string".to_string())),
                                ..Default::default()
                            },
                        ),
                    ])),
                    ..Default::default()
                })),
                ..Default::default()
            },
        );
        schema.write_only_properties = vec!["/properties/Items/*/Secret".to_string()];

        let generated = generate_synthetic_schema_code(&schema)
            .expect("a matched array wildcard write-only pointer should generate");
        assert!(
            generated.contains(
                r#"StructField::optional("secret", AttributeType::string()).write_only()"#
            ),
            "Items[*].Secret must be provider-omitted: {generated}"
        );
        assert!(
            !generated.contains(
                r#"StructField::optional("visible", AttributeType::string()).write_only()"#
            ),
            "Items[*].Visible must remain observable: {generated}"
        );
    }

    #[test]
    fn test_nested_write_only_path_through_ref_array_items_is_provider_omitted() {
        let mut schema = empty_schema("AWS::Test::NestedWriteOnlyRefArray");
        schema.properties.insert(
            "Items".to_string(),
            CfnProperty {
                prop_type: Some(TypeValue::Single("array".to_string())),
                items: Some(Box::new(CfnProperty {
                    ref_path: Some("#/definitions/Item".to_string()),
                    ..Default::default()
                })),
                ..Default::default()
            },
        );
        schema.definitions = Some(BTreeMap::from([(
            "Item".to_string(),
            CfnDefinition {
                def_type: Some("object".to_string()),
                properties: Some(BTreeMap::from([
                    (
                        "Secret".to_string(),
                        CfnProperty {
                            prop_type: Some(TypeValue::Single("string".to_string())),
                            ..Default::default()
                        },
                    ),
                    (
                        "Visible".to_string(),
                        CfnProperty {
                            prop_type: Some(TypeValue::Single("string".to_string())),
                            ..Default::default()
                        },
                    ),
                ])),
                ..Default::default()
            },
        )]));
        schema.write_only_properties = vec!["/properties/Items/*/Secret".to_string()];

        let generated = generate_synthetic_schema_code(&schema)
            .expect("a matched write-only pointer through $ref array items should generate");
        assert!(
            generated.contains(
                r#"StructField::optional("secret", AttributeType::string()).write_only()"#
            ),
            "Items[*].Secret must be provider-omitted: {generated}"
        );
        assert!(
            !generated.contains(
                r#"StructField::optional("visible", AttributeType::string()).write_only()"#
            ),
            "Items[*].Visible must remain observable: {generated}"
        );
    }

    #[test]
    fn test_non_deprecated_unmatched_nested_write_only_pointer_is_rejected() {
        let mut schema = empty_schema("AWS::Test::NestedWriteOnlyMissing");
        schema.properties.insert(
            "Config".to_string(),
            CfnProperty {
                prop_type: Some(TypeValue::Single("object".to_string())),
                properties: Some(BTreeMap::from([(
                    "Present".to_string(),
                    CfnProperty {
                        prop_type: Some(TypeValue::Single("string".to_string())),
                        ..Default::default()
                    },
                )])),
                ..Default::default()
            },
        );
        let pointer = "/properties/Config/Missing";
        schema.write_only_properties = vec![pointer.to_string()];

        for (generator, result) in [
            ("Rust", generate_synthetic_schema_code(&schema)),
            (
                "Markdown",
                generate_markdown(&schema, "AWS::Test::NestedWriteOnlyMissing"),
            ),
        ] {
            let error = result
                .expect_err("an unresolved nested writeOnlyProperties pointer must fail closed");
            let message = error.to_string();
            assert!(message.contains(pointer), "{generator}: {message}");
            assert!(
                message.contains("did not resolve"),
                "{generator}: {message}"
            );
        }
    }

    #[test]
    fn test_shared_definition_with_path_dependent_write_only_fields_is_rejected() {
        let mut schema = empty_schema("AWS::Test::NestedWriteOnlySharedRef");
        let shared_ref = CfnProperty {
            ref_path: Some("#/definitions/Shared".to_string()),
            ..Default::default()
        };
        schema.properties = BTreeMap::from([
            ("Left".to_string(), shared_ref.clone()),
            ("Right".to_string(), shared_ref),
        ]);
        schema.definitions = Some(BTreeMap::from([(
            "Shared".to_string(),
            CfnDefinition {
                def_type: Some("object".to_string()),
                properties: Some(BTreeMap::from([
                    (
                        "Secret".to_string(),
                        CfnProperty {
                            prop_type: Some(TypeValue::Single("string".to_string())),
                            ..Default::default()
                        },
                    ),
                    (
                        "Visible".to_string(),
                        CfnProperty {
                            prop_type: Some(TypeValue::Single("string".to_string())),
                            ..Default::default()
                        },
                    ),
                ])),
                ..Default::default()
            },
        )]));
        schema.write_only_properties = vec!["/properties/Left/Secret".to_string()];

        for (generator, result) in [
            ("Rust", generate_synthetic_schema_code(&schema)),
            (
                "Markdown",
                generate_markdown(&schema, "AWS::Test::NestedWriteOnlySharedRef"),
            ),
        ] {
            let error = result.expect_err(
                "a shared definition with path-dependent write-only fields must fail closed",
            );
            let message = error.to_string();
            assert!(message.contains("Shared"), "{generator}: {message}");
            assert!(
                message.contains("/properties/Left")
                    && message.contains("/properties/Right")
                    && message.contains("different nested writeOnlyProperties"),
                "{generator}: {message}"
            );
        }
    }

    #[test]
    fn test_same_named_inline_struct_with_path_dependent_write_only_fields_is_rejected() {
        fn parent() -> CfnProperty {
            CfnProperty {
                prop_type: Some(TypeValue::Single("object".to_string())),
                properties: Some(BTreeMap::from([(
                    "Config".to_string(),
                    CfnProperty {
                        prop_type: Some(TypeValue::Single("object".to_string())),
                        properties: Some(BTreeMap::from([
                            (
                                "Secret".to_string(),
                                CfnProperty {
                                    prop_type: Some(TypeValue::Single("string".to_string())),
                                    ..Default::default()
                                },
                            ),
                            (
                                "Visible".to_string(),
                                CfnProperty {
                                    prop_type: Some(TypeValue::Single("string".to_string())),
                                    ..Default::default()
                                },
                            ),
                        ])),
                        ..Default::default()
                    },
                )])),
                ..Default::default()
            }
        }

        let mut schema = empty_schema("AWS::Test::NestedWriteOnlyInlineCollision");
        schema.properties = BTreeMap::from([
            ("Left".to_string(), parent()),
            ("Right".to_string(), parent()),
        ]);
        schema.write_only_properties = vec!["/properties/Left/Config/Secret".to_string()];

        for (generator, result) in [
            ("Rust", generate_synthetic_schema_code(&schema)),
            (
                "Markdown",
                generate_markdown(&schema, "AWS::Test::NestedWriteOnlyInlineCollision"),
            ),
        ] {
            let error = result.expect_err(
                "same-named inline structs with path-dependent write-only fields must fail closed",
            );
            let message = error.to_string();
            assert!(message.contains("Config"), "{generator}: {message}");
            assert!(
                message.contains("/properties/Left/Config")
                    && message.contains("/properties/Right/Config")
                    && message.contains("different nested writeOnlyProperties"),
                "{generator}: {message}"
            );
        }
    }

    #[test]
    fn test_nested_read_only_write_only_pointer_is_rejected() {
        let mut schema = empty_schema("AWS::Test::NestedReadWriteConflict");
        schema.properties.insert(
            "Config".to_string(),
            CfnProperty {
                prop_type: Some(TypeValue::Single("object".to_string())),
                properties: Some(BTreeMap::from([(
                    "Value".to_string(),
                    CfnProperty {
                        prop_type: Some(TypeValue::Single("string".to_string())),
                        ..Default::default()
                    },
                )])),
                ..Default::default()
            },
        );
        let pointer = "/properties/Config/Value";
        schema.read_only_properties = vec![pointer.to_string()];
        schema.write_only_properties = vec![pointer.to_string()];

        for (generator, result) in [
            ("Rust", generate_synthetic_schema_code(&schema)),
            (
                "Markdown",
                generate_markdown(&schema, "AWS::Test::NestedReadWriteConflict"),
            ),
        ] {
            let error = result
                .expect_err("a pointer cannot be both read-only and write-only in generated code");
            let message = error.to_string();
            assert!(message.contains(pointer), "{generator}: {message}");
            assert!(
                message.contains("both readOnlyProperties and writeOnlyProperties"),
                "{generator}: {message}"
            );
        }
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

    #[test]
    fn test_budgets_budget_is_provisionable_without_schema_handlers() {
        let schema = cfn_schema_for_codegen_tests("AWS::Budgets::Budget");

        assert!(schema.handlers.is_empty());
        assert!(schema.is_cloud_control_provisionable());
    }

    #[test]
    fn test_budgets_budget_handlerless_permissions_are_complete() {
        let schema = cfn_schema_for_codegen_tests("AWS::Budgets::Budget");

        assert_eq!(
            handler_permissions(&schema, "create"),
            [
                "aws-portal:ModifyBilling",
                "budgets:ModifyBudget",
                "budgets:TagResource",
            ]
        );
        assert_eq!(
            handler_permissions(&schema, "read"),
            [
                "aws-portal:ViewBilling",
                "budgets:ListTagsForResource",
                "budgets:ViewBudget",
            ]
        );
        assert_eq!(
            handler_permissions(&schema, "update"),
            [
                "aws-portal:ModifyBilling",
                "aws-portal:ViewBilling",
                "budgets:ListTagsForResource",
                "budgets:ModifyBudget",
                "budgets:TagResource",
                "budgets:UntagResource",
            ]
        );
        assert_eq!(
            handler_permissions(&schema, "delete"),
            ["aws-portal:ModifyBilling", "budgets:ModifyBudget"]
        );
        assert_eq!(
            handler_permissions(&schema, "list"),
            ["aws-portal:ViewBilling", "budgets:ViewBudget"]
        );
    }

    // === Issue #199: snake_case DSL spelling for every StringEnum value ===

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
        let generated = generate_synthetic_schema_code(&schema).unwrap();

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
        let generated = generate_synthetic_schema_code(&schema).unwrap();
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
            generated.contains("StructField::optional(\"price_class\", AttributeType::enum_("),
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
