#!/bin/bash
# Generate awscc provider documentation from CloudFormation schemas
set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

DOCS_DIR="$PROJECT_ROOT/generated-docs/awscc"
SCHEMA_DIR="$PROJECT_ROOT/cfn-schema-cache"
rm -rf "$DOCS_DIR"
mkdir -p "$DOCS_DIR"

cd "$PROJECT_ROOT"

# Generate docs for each schema file
for SCHEMA_FILE in "$SCHEMA_DIR"/*.json; do
    [ -f "$SCHEMA_FILE" ] || continue
    FILENAME=$(basename "$SCHEMA_FILE" .json)

    # Convert filename to CloudFormation type name
    # AWS__EC2__VPC.json → AWS::EC2::VPC
    TYPE_NAME=$(echo "$FILENAME" | sed 's/__/::/g')

    # Derive output path using codegen's print-dsl-resource-name
    # AWS::EC2::VPC → ec2.vpc → ec2/vpc
    DSL_NAME=$(cargo run --bin codegen -- --type-name "$TYPE_NAME" --print-dsl-resource-name 2>/dev/null)
    SERVICE=$(echo "$DSL_NAME" | cut -d'.' -f1)
    RESOURCE=$(echo "$DSL_NAME" | cut -d'.' -f2-)
    mkdir -p "$DOCS_DIR/$SERVICE"
    OUTPUT_FILE="$DOCS_DIR/$SERVICE/$RESOURCE.md"

    echo "Generating: $TYPE_NAME → $OUTPUT_FILE"
    cargo run --bin codegen -- \
        --file "$SCHEMA_FILE" \
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
done

echo ""
echo "Done! Generated documentation in $DOCS_DIR"
