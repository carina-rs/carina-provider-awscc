#!/bin/bash
# Generate awscc provider documentation from CloudFormation schemas
#
# Usage (from project root):
#   aws-vault exec <profile> -- ./scripts/generate-docs.sh
#   aws-vault exec <profile> -- ./scripts/generate-docs.sh --refresh-cache
#
# Options:
#   --refresh-cache  Force re-download of all CloudFormation schemas
#
# Downloaded schemas are cached in cfn-schema-cache/.
# Subsequent runs use cached schemas unless --refresh-cache is specified.
set -e

# Parse flags
REFRESH_CACHE=false
for arg in "$@"; do
    case "$arg" in
        --refresh-cache) REFRESH_CACHE=true ;;
    esac
done

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

CACHE_DIR="$PROJECT_ROOT/cfn-schema-cache"
DOCS_DIR="$PROJECT_ROOT/generated-docs/awscc"
EXAMPLES_DIR="$PROJECT_ROOT/examples"
mkdir -p "$CACHE_DIR"
rm -rf "$DOCS_DIR"
mkdir -p "$DOCS_DIR"

cd "$PROJECT_ROOT"

# Same resource types as generate-schemas.sh
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
    "AWS::IAM::Role"
    "AWS::Logs::LogGroup"
    "AWS::SSO::Instance"
    "AWS::SSO::PermissionSet"
    "AWS::SSO::Assignment"
    "AWS::IdentityStore::Group"
    "AWS::IdentityStore::GroupMembership"
)

echo "Generating awscc provider documentation..."
echo "Output directory: $DOCS_DIR"
echo ""

# Build codegen tool first
cargo build --bin codegen --quiet 2>/dev/null || true

CODEGEN_BIN="target/debug/codegen"
if [ ! -f "$CODEGEN_BIN" ]; then
    echo "Trying to build with cargo..."
    cargo build --bin codegen
    if [ ! -f "$CODEGEN_BIN" ]; then
        echo "ERROR: Could not build codegen binary"
        exit 1
    fi
fi

for TYPE_NAME in "${RESOURCE_TYPES[@]}"; do
    DSL_NAME=$("$CODEGEN_BIN" --type-name "$TYPE_NAME" --print-dsl-resource-name)
    SERVICE=$(echo "$DSL_NAME" | cut -d'.' -f1)
    RESOURCE=$(echo "$DSL_NAME" | cut -d'.' -f2-)
    FULL_RESOURCE=$("$CODEGEN_BIN" --type-name "$TYPE_NAME" --print-full-resource-name)
    mkdir -p "$DOCS_DIR/$SERVICE"
    OUTPUT_FILE="$DOCS_DIR/$SERVICE/$RESOURCE.md"

    echo "Generating: $TYPE_NAME → $OUTPUT_FILE"

    # Cache CloudFormation schema to avoid redundant API calls
    CACHE_FILE="$CACHE_DIR/${TYPE_NAME//::/__}.json"
    if [ "$REFRESH_CACHE" = true ] || [ ! -f "$CACHE_FILE" ]; then
        aws cloudformation describe-type \
            --type RESOURCE \
            --type-name "$TYPE_NAME" \
            --query 'Schema' \
            --output text > "$CACHE_FILE"
    else
        echo "  Using cached schema"
    fi

    # Generate documentation
    "$CODEGEN_BIN" \
        --file "$CACHE_FILE" \
        --type-name "$TYPE_NAME" \
        --format markdown \
        --output "$OUTPUT_FILE" 2>/dev/null || {
        echo "  Warning: failed to generate docs for $TYPE_NAME, skipping"
        continue
    }

    # Add Starlight frontmatter
    if [ -f "$OUTPUT_FILE" ]; then
        DSL_NAME_FULL="awscc.$DSL_NAME"
        SERVICE_DISPLAY=$(echo "$SERVICE" | tr '[:lower:]' '[:upper:]')
        FRONTMATTER_TMPFILE=$(mktemp)
        {
            echo "---"
            echo "title: \"$DSL_NAME_FULL\""
            echo "description: \"AWSCC $SERVICE_DISPLAY $RESOURCE resource reference\""
            echo "---"
            echo ""
            sed '1{/^# /d;}' "$OUTPUT_FILE"
        } > "$FRONTMATTER_TMPFILE"
        mv "$FRONTMATTER_TMPFILE" "$OUTPUT_FILE"
    fi

    # Insert example from hand-written example file (after description, before Argument Reference)
    EXAMPLE_FILE="$EXAMPLES_DIR/${FULL_RESOURCE}/main.crn"
    if [ -f "$EXAMPLE_FILE" ]; then
        EXAMPLE_TMPFILE=$(mktemp)
        {
            echo "## Example"
            echo ""
            echo '```crn'
            # Strip provider block, leading comments, and leading blank lines
            sed -n '/^provider /,/^}/!p' "$EXAMPLE_FILE" | \
                sed '/^#/d' | \
                sed '/./,$!d'
            echo '```'
            echo ""
        } > "$EXAMPLE_TMPFILE"
        # Insert the example block before "## Argument Reference"
        MERGED_TMPFILE=$(mktemp)
        while IFS= read -r line || [ -n "$line" ]; do
            if [ "$line" = "## Argument Reference" ]; then
                cat "$EXAMPLE_TMPFILE"
            fi
            printf '%s\n' "$line"
        done < "$OUTPUT_FILE" > "$MERGED_TMPFILE"
        mv "$MERGED_TMPFILE" "$OUTPUT_FILE"
        rm -f "$EXAMPLE_TMPFILE"
    fi
done

echo ""
echo "Done! Generated documentation in $DOCS_DIR"
