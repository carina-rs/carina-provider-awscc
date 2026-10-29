#!/bin/bash
# Generate awscc provider documentation from CloudFormation schemas
#
# Usage (from project root):
#   aws-vault exec <profile> -- ./scripts/generate-docs.sh
#   aws-vault exec <profile> -- ./scripts/generate-docs.sh --refresh-cache
#   ./scripts/generate-docs.sh --from-cache-only
#
# Options:
#   --refresh-cache    Force re-download of all CloudFormation schemas
#                      (requires AWS credentials via aws-vault).
#   --from-cache-only  Never call AWS; require every schema to be present
#                      in the committed cache. Fails with a clear error
#                      if any cache file is missing. Used by CI to detect
#                      docs drift without needing AWS credentials.
#
# Downloaded schemas are cached in cfn-schema-cache/.
# Subsequent runs use cached schemas unless --refresh-cache is specified.
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

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

CACHE_DIR="$PROJECT_ROOT/cfn-schema-cache"
DOCS_DIR="$PROJECT_ROOT/generated-docs/awscc"
EXAMPLES_DIR="$PROJECT_ROOT/examples"
SCHEMAS_DIR="$PROJECT_ROOT/carina-provider-awscc/src/schemas/generated"

cd "$PROJECT_ROOT"

# Build codegen successfully before deleting the existing generated docs. This
# prevents a stale binary's clap exit code from being mistaken for the
# deliberate NON_PROVISIONABLE skip below.
cargo build -p carina-provider-awscc --bin codegen --quiet

CODEGEN_BIN="target/debug/codegen"
if [ ! -x "$CODEGEN_BIN" ]; then
    echo "ERROR: codegen binary not found at $CODEGEN_BIN" >&2
    exit 1
fi

mkdir -p "$CACHE_DIR"
rm -rf "$DOCS_DIR"
mkdir -p "$DOCS_DIR"

# Derive resource types from generated schema files (single source of truth).
# Each .rs file (excluding mod.rs) has a header comment:
#   Auto-generated from CloudFormation schema: AWS::EC2::VPC
RESOURCE_TYPES=()
for rs_file in "$SCHEMAS_DIR"/*/*.rs; do
    [ "$(basename "$rs_file")" = "mod.rs" ] && continue
    cfn_type=$(grep -m1 'Auto-generated from CloudFormation schema:' "$rs_file" | sed 's/.*schema: //')
    if [ -n "$cfn_type" ]; then
        RESOURCE_TYPES+=("$cfn_type")
    fi
done

echo "Generating awscc provider documentation..."
echo "Output directory: $DOCS_DIR"
echo ""

for TYPE_NAME in "${RESOURCE_TYPES[@]}"; do
    DSL_NAME=$("$CODEGEN_BIN" --type-name "$TYPE_NAME" --print-dsl-resource-name)
    SERVICE=$(echo "$DSL_NAME" | cut -d'.' -f1)
    RESOURCE=$(echo "$DSL_NAME" | cut -d'.' -f2-)
    # On-disk filenames are snake_case to match the schema files
    # (carina-provider-awscc/src/schemas/generated/<svc>/<resource>.rs) and the
    # 1:1 basename invariant enforced by scripts/check-docs-drift.sh.
    # Strategy Y casing makes the DSL surface PascalCase ("VpcEndpoint"),
    # so we convert here only for the filename. The title and the
    # markdown body keep the codegen's PascalCase DSL form.
    RESOURCE_FILE=$(echo "$RESOURCE" | sed -E 's/([a-z0-9])([A-Z])/\1_\2/g' | tr '[:upper:]' '[:lower:]')
    mkdir -p "$DOCS_DIR/$SERVICE"
    OUTPUT_FILE="$DOCS_DIR/$SERVICE/$RESOURCE_FILE.md"

    echo "Generating: $TYPE_NAME → $OUTPUT_FILE"

    # Cache CloudFormation schema to avoid redundant API calls
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
            --output text > "$CACHE_FILE"
    else
        echo "  Using cached schema"
    fi

    # Generate documentation. Exit code 2 signals a deliberate skip
    # (NON_PROVISIONABLE type); anything else non-zero is a real error.
    set +e
    "$CODEGEN_BIN" \
        --file "$CACHE_FILE" \
        --type-name "$TYPE_NAME" \
        --format markdown \
        --examples-dir "$EXAMPLES_DIR" \
        --output "$OUTPUT_FILE"
    EXIT_CODE=$?
    set -e

    case "$EXIT_CODE" in
        0) ;;
        2)
            rm -f "$OUTPUT_FILE"
            continue
            ;;
        *)
            echo "ERROR: failed to generate docs for $TYPE_NAME" >&2
            rm -f "$OUTPUT_FILE"
            exit "$EXIT_CODE"
            ;;
    esac

    # Add Starlight frontmatter
    if [ -f "$OUTPUT_FILE" ]; then
        DSL_NAME_FULL="awscc.$DSL_NAME"
        FRONTMATTER_TMPFILE=$(mktemp)
        {
            echo "---"
            echo "title: \"$DSL_NAME_FULL\""
            echo "---"
            echo ""
            sed '1{/^# /d;}' "$OUTPUT_FILE"
        } > "$FRONTMATTER_TMPFILE"
        mv "$FRONTMATTER_TMPFILE" "$OUTPUT_FILE"
    fi

done

echo ""
echo "Done! Generated documentation in $DOCS_DIR"
