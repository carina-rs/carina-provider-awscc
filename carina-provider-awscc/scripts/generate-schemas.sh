#!/bin/bash
# Generate awscc provider schemas from CloudFormation
#
# Usage (from project root):
#   aws-vault exec <profile> -- ./carina-provider-awscc/scripts/generate-schemas.sh
#   aws-vault exec <profile> -- ./carina-provider-awscc/scripts/generate-schemas.sh --refresh-cache
#   ./carina-provider-awscc/scripts/generate-schemas.sh --from-cache-only
#
# Options:
#   --refresh-cache    Force re-download of all CloudFormation schemas
#                      (requires AWS credentials via aws-vault).
#   --from-cache-only  Never call AWS; require every schema to be present
#                      in the committed cache. Fails with a clear error
#                      if any cache file is missing. Used by CI to detect
#                      codegen drift without needing AWS credentials.
#
# Downloaded schemas are cached in cfn-schema-cache/ at the repo root.
# Subsequent runs use cached schemas unless --refresh-cache is specified.
#
# This script generates Rust schema code from CloudFormation resource type schemas.

set -e

# Parse flags
REFRESH_CACHE=false
FROM_CACHE_ONLY=false
for arg in "$@"; do
    case "$arg" in
        --refresh-cache) REFRESH_CACHE=true ;;
        --from-cache-only) FROM_CACHE_ONLY=true ;;
    esac
done

# Cache lives at the repo root (where the user runs `cd <repo> && ...`).
# Keeping it at the repo root rather than nested under the crate dir lets
# CI access it without juggling per-crate working directories.
CACHE_DIR="cfn-schema-cache"
OUTPUT_DIR="carina-provider-awscc/src/schemas/generated"
mkdir -p "$CACHE_DIR"

# List of resource types to generate
RESOURCE_TYPES=(
    "AWS::EC2::VPC"
    "AWS::EC2::Subnet"
    "AWS::EC2::InternetGateway"
    "AWS::EC2::RouteTable"
    "AWS::EC2::Route"
    "AWS::EC2::SubnetRouteTableAssociation"
    "AWS::EC2::EIP"
    "AWS::EC2::NatGateway"
    "AWS::EC2::SecurityGroup"
    "AWS::EC2::SecurityGroupIngress"
    "AWS::EC2::SecurityGroupEgress"
    "AWS::EC2::VPCEndpoint"
    "AWS::EC2::VPCGatewayAttachment"
    "AWS::EC2::FlowLog"
    "AWS::EC2::IPAM"
    "AWS::EC2::IPAMPool"
    "AWS::EC2::VPNGateway"
    "AWS::EC2::TransitGateway"
    "AWS::EC2::VPCPeeringConnection"
    "AWS::EC2::EgressOnlyInternetGateway"
    "AWS::EC2::TransitGatewayAttachment"
    "AWS::S3::Bucket"
    "AWS::S3::BucketPolicy"
    "AWS::IAM::Role"
    "AWS::IAM::RolePolicy"
    "AWS::IAM::OIDCProvider"
    "AWS::Logs::LogGroup"
    "AWS::Organizations::Organization"
    "AWS::Organizations::Account"
    "AWS::SSO::Instance"
    "AWS::SSO::PermissionSet"
    "AWS::SSO::Assignment"
    "AWS::IdentityStore::Group"
    "AWS::IdentityStore::GroupMembership"
    "AWS::Route53::HostedZone"
    "AWS::CloudFront::Distribution"
    "AWS::CloudFront::OriginAccessControl"
    "AWS::WAFv2::WebACL"
    "AWS::KMS::Key"
    "AWS::DynamoDB::Table"
    "AWS::ECS::Cluster"
    "AWS::ElasticLoadBalancingV2::LoadBalancer"
    "AWS::ElasticLoadBalancingV2::Listener"
    "AWS::ElasticLoadBalancingV2::TargetGroup"
)

echo "Generating awscc provider schemas..."
echo "Output directory: $OUTPUT_DIR"
echo ""

# Build codegen tool first
# Use --quiet to suppress cargo output; build only the binary (not the lib)
cargo build -p carina-provider-awscc --bin codegen --quiet 2>/dev/null || true

# Find the built binary
CODEGEN_BIN="target/debug/codegen"
if [ ! -f "$CODEGEN_BIN" ]; then
    echo "ERROR: codegen binary not found at $CODEGEN_BIN"
    echo "Trying to build with cargo..."
    cargo build -p carina-provider-awscc --bin codegen
    if [ ! -f "$CODEGEN_BIN" ]; then
        echo "ERROR: Could not build codegen binary"
        exit 1
    fi
fi

# Helper: extract service name from CloudFormation type (e.g., AWS::EC2::VPC -> ec2)
service_name() {
    echo "$1" | awk -F'::' '{print tolower($2)}'
}

# Helper: extract resource module name (e.g., AWS::EC2::VPC -> vpc)
resource_module_name() {
    "$CODEGEN_BIN" --type-name "$1" --print-module-name
}

# Remove old flat-structure files (migration from flat to service/resource layout)
for TYPE_NAME in "${RESOURCE_TYPES[@]}"; do
    FLAT_NAME=$("$CODEGEN_BIN" --type-name "$TYPE_NAME" --print-full-resource-name)
    OLD_FILE="$OUTPUT_DIR/${FLAT_NAME}.rs"
    if [ -f "$OLD_FILE" ]; then
        rm -f "$OLD_FILE"
    fi
done

# Types that were successfully generated (filters out NON_PROVISIONABLE skips).
# All mod.rs generation below references this array, not RESOURCE_TYPES.
GENERATED_TYPES=()

for TYPE_NAME in "${RESOURCE_TYPES[@]}"; do
    SVC=$(service_name "$TYPE_NAME")
    RESOURCE=$(resource_module_name "$TYPE_NAME")
    OUTPUT_FILE="$OUTPUT_DIR/$SVC/${RESOURCE}.rs"

    echo "Generating $TYPE_NAME -> $OUTPUT_FILE"

    mkdir -p "$OUTPUT_DIR/$SVC"

    # Cache CloudFormation schema to avoid redundant API calls.
    CACHE_FILE="$CACHE_DIR/${TYPE_NAME//::/__}.json"
    if [ "$FROM_CACHE_ONLY" = true ]; then
        # CI / offline mode: cache must already exist. Surface a clear
        # error rather than silently calling AWS (which would fail with
        # a credential error and obscure the real cause).
        if [ ! -f "$CACHE_FILE" ]; then
            echo "ERROR: --from-cache-only set but cache file not found:" >&2
            echo "  $CACHE_FILE" >&2
            echo "Run 'aws-vault exec <profile> -- $0 --refresh-cache'" >&2
            echo "locally and commit the updated cfn-schema-cache/." >&2
            exit 1
        fi
        echo "  Using cached schema (--from-cache-only)"
    elif [ "$REFRESH_CACHE" = true ] || [ ! -f "$CACHE_FILE" ]; then
        aws cloudformation describe-type \
            --type RESOURCE \
            --type-name "$TYPE_NAME" \
            --query 'Schema' \
            --output text 2>/dev/null > "$CACHE_FILE"
    else
        echo "  Using cached schema"
    fi

    set +e
    "$CODEGEN_BIN" --type-name "$TYPE_NAME" < "$CACHE_FILE" > "$OUTPUT_FILE"
    EXIT_CODE=$?
    set -e

    case "$EXIT_CODE" in
        0)
            GENERATED_TYPES+=("$TYPE_NAME")
            ;;
        2)
            # NON_PROVISIONABLE: codegen logged the reason to stderr.
            rm -f "$OUTPUT_FILE"
            ;;
        *)
            echo "  ERROR: Failed to generate $TYPE_NAME"
            rm -f "$OUTPUT_FILE"
            ;;
    esac
done

# Services that have at least one generated resource. Drives mod.rs emission
# below; services whose every resource was skipped are omitted entirely.
SERVICES=""
for TYPE_NAME in "${GENERATED_TYPES[@]}"; do
    SVC=$(service_name "$TYPE_NAME")
    case " $SERVICES " in
        *" $SVC "*) ;;
        *) SERVICES="$SERVICES $SVC" ;;
    esac
done
SERVICES=$(echo "$SERVICES" | tr ' ' '\n' | sort | tr '\n' ' ')

# Remove directories for services whose resources were all skipped this run.
for DIR in "$OUTPUT_DIR"/*/; do
    [ -d "$DIR" ] || continue
    SVC=$(basename "$DIR")
    case " $SERVICES " in
        *" $SVC "*) ;;
        *) rm -rf "$DIR" ;;
    esac
done

# Generate per-service mod.rs files
for SVC in $SERVICES; do
    SVC_MOD="$OUTPUT_DIR/$SVC/mod.rs"
    echo "Generating $SVC_MOD"

    cat > "$SVC_MOD" << 'EOF'
//! Auto-generated — DO NOT EDIT MANUALLY
//!
//! Regenerate with:
//!   aws-vault exec <profile> -- ./carina-provider-awscc/scripts/generate-schemas.sh

// Re-export parent types so resource modules can use `super::` to access them.
pub use super::*;

EOF

    # Add module declarations for resources in this service
    for TYPE_NAME in "${GENERATED_TYPES[@]}"; do
        TYPE_SVC=$(service_name "$TYPE_NAME")
        if [ "$TYPE_SVC" = "$SVC" ]; then
            RESOURCE=$(resource_module_name "$TYPE_NAME")
            echo "pub mod ${RESOURCE};" >> "$SVC_MOD"
        fi
    done
done

# Generate top-level mod.rs
echo ""
echo "Generating $OUTPUT_DIR/mod.rs"

cat > "$OUTPUT_DIR/mod.rs" << 'EOF'
//! Auto-generated AWS Cloud Control resource schemas
//!
//! DO NOT EDIT MANUALLY - regenerate with:
//!   aws-vault exec <profile> -- ./carina-provider-awscc/scripts/generate-schemas.sh

use std::collections::HashMap;
use std::sync::LazyLock;

// Re-export all types and validators from awscc_types so that
// generated schema files can use `super::` to access them.
pub use super::awscc_types::*;

EOF

# Add service module declarations
for SVC in $SERVICES; do
    echo "pub mod ${SVC};" >> "$OUTPUT_DIR/mod.rs"
done

# --- SCHEMA_CONFIGS LazyLock ---
cat >> "$OUTPUT_DIR/mod.rs" << 'EOF'

/// Cached schema configs, initialized once on first access.
static SCHEMA_CONFIGS: LazyLock<Vec<AwsccSchemaConfig>> = LazyLock::new(build_configs);

/// Index from resource_type_name (e.g., "ec2.vpc") to position in SCHEMA_CONFIGS.
static SCHEMA_CONFIG_INDEX: LazyLock<HashMap<&'static str, usize>> = LazyLock::new(|| {
    SCHEMA_CONFIGS
        .iter()
        .enumerate()
        .map(|(i, c)| (c.resource_type_name, i))
        .collect()
});

/// Cached enum valid values: resource_type -> (attr_name -> valid values slice).
static ENUM_VALID_VALUES: LazyLock<
    HashMap<&'static str, HashMap<&'static str, &'static [&'static str]>>,
> = LazyLock::new(|| {
    #[allow(clippy::type_complexity)]
    let modules: &[(&str, &[(&str, &[&str])])] = &[
EOF

# Add enum_valid_values() calls for ENUM_VALID_VALUES
for TYPE_NAME in "${GENERATED_TYPES[@]}"; do
    SVC=$(service_name "$TYPE_NAME")
    RESOURCE=$(resource_module_name "$TYPE_NAME")
    echo "        ${SVC}::${RESOURCE}::enum_valid_values()," >> "$OUTPUT_DIR/mod.rs"
done

cat >> "$OUTPUT_DIR/mod.rs" << 'EOF'
    ];
    let mut map: HashMap<&str, HashMap<&str, &[&str]>> = HashMap::new();
    for (rt, attrs) in modules {
        let attr_map = map.entry(rt).or_default();
        for (attr, values) in *attrs {
            attr_map.insert(attr, values);
        }
    }
    map
});

/// Build all schema configs (called once by LazyLock).
fn build_configs() -> Vec<AwsccSchemaConfig> {
    vec![
EOF

# Add config function calls dynamically
for TYPE_NAME in "${GENERATED_TYPES[@]}"; do
    SVC=$(service_name "$TYPE_NAME")
    RESOURCE=$(resource_module_name "$TYPE_NAME")
    FULL_NAME=$("$CODEGEN_BIN" --type-name "$TYPE_NAME" --print-full-resource-name)
    FUNC_NAME="${FULL_NAME}_config"

    echo "        ${SVC}::${RESOURCE}::${FUNC_NAME}()," >> "$OUTPUT_DIR/mod.rs"
done

cat >> "$OUTPUT_DIR/mod.rs" << 'EOF'
    ]
}

/// Returns a reference to the cached schema configs slice.
pub fn configs() -> &'static [AwsccSchemaConfig] {
    &SCHEMA_CONFIGS
}

/// Look up a schema config by resource_type_name (e.g., "ec2.vpc"). O(1).
pub fn get_config_by_type(resource_type: &str) -> Option<&'static AwsccSchemaConfig> {
    SCHEMA_CONFIG_INDEX
        .get(resource_type)
        .map(|&i| &SCHEMA_CONFIGS[i])
}

/// Get valid enum values for a given resource type and attribute name. O(1).
/// Used during read-back to normalize AWS-returned values to canonical DSL form.
///
/// Auto-generated from schema enum constants.
pub fn get_enum_valid_values(resource_type: &str, attr_name: &str) -> Option<&'static [&'static str]> {
    ENUM_VALID_VALUES
        .get(resource_type)
        .and_then(|attrs| attrs.get(attr_name))
        .copied()
}

// `get_enum_alias_reverse` and `build_enum_aliases_map` are no
// longer emitted — DSL → API canonical conversion now goes through
// `DslMap::api_for` against the exhaustive `dsl_aliases` table on
// each `StringEnum` (awscc#220). The single source of truth lives
// inline in `schemas/generated/<service>/<resource>.rs`.
EOF

echo ""
echo "Running cargo fmt..."
cargo fmt -p carina-provider-awscc

echo ""
echo "Done! Generated schemas in $OUTPUT_DIR"
