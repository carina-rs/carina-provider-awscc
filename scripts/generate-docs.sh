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
SERVICE_TITLES_FILE="$PROJECT_ROOT/docs/registry/service-titles.json"
SCHEMAS_DIR="$PROJECT_ROOT/carina-provider-awscc/src/schemas/generated"
mkdir -p "$CACHE_DIR"
rm -rf "$DOCS_DIR"
mkdir -p "$DOCS_DIR"

cd "$PROJECT_ROOT"

# Look up a lowercase service code's canonical AWS display name.
service_display_name() {
    python3 - "$SERVICE_TITLES_FILE" "$1" <<'PY'
import json
import sys


class DuplicateJSONKeyError(ValueError):
    def __init__(self, key):
        super().__init__(key)
        self.key = key


def reject_duplicate_json_keys(pairs):
    result = {}
    for key, value in pairs:
        if key in result:
            raise DuplicateJSONKeyError(key)
        result[key] = value
    return result


path, service_id = sys.argv[1:]
try:
    with open(path, encoding="utf-8") as file:
        titles = json.load(file, object_pairs_hook=reject_duplicate_json_keys)
except DuplicateJSONKeyError as error:
    print(
        f"error: {path}: duplicate AWS service title key {error.key!r}",
        file=sys.stderr,
    )
    raise SystemExit(1)
except (OSError, UnicodeError, json.JSONDecodeError) as error:
    print(f"error: cannot read AWS service titles from {path}: {error}", file=sys.stderr)
    raise SystemExit(1)

if not isinstance(titles, dict):
    print(f"error: AWS service titles file is not a JSON object: {path}", file=sys.stderr)
    raise SystemExit(1)
if service_id not in titles:
    print(
        f"error: AWS service {service_id!r} has no display title in {path}",
        file=sys.stderr,
    )
    raise SystemExit(1)

title = titles[service_id]
if not isinstance(title, str):
    print(
        f"error: display title for AWS service {service_id!r} is not a string in {path}",
        file=sys.stderr,
    )
    raise SystemExit(1)
print(title)
PY
}

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
    # On-disk filenames are snake_case to match the schema files
    # (carina-provider-awscc/src/schemas/generated/<svc>/<resource>.rs) and the
    # 1:1 basename invariant enforced by scripts/check-docs-drift.sh.
    # Strategy Y casing makes the DSL surface PascalCase ("VpcEndpoint"),
    # so we convert here only for the filename. Title, description, and the
    # markdown body keep the codegen's PascalCase DSL form.
    RESOURCE_FILE=$(echo "$RESOURCE" | sed -E 's/([a-z0-9])([A-Z])/\1_\2/g' | tr '[:upper:]' '[:lower:]')
    FULL_RESOURCE=$("$CODEGEN_BIN" --type-name "$TYPE_NAME" --print-full-resource-name)
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
            echo "  Warning: failed to generate docs for $TYPE_NAME, skipping"
            rm -f "$OUTPUT_FILE"
            continue
            ;;
    esac

    # Add Starlight frontmatter
    if [ -f "$OUTPUT_FILE" ]; then
        DSL_NAME_FULL="awscc.$DSL_NAME"
        SERVICE_DISPLAY=$(service_display_name "$SERVICE")
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
