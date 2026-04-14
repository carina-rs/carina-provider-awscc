#!/bin/bash
# Check that generated docs cover every generated schema.
#
# Fails if any schema resource is missing a corresponding docs file,
# or if docs exist for a resource that has no schema.
set -e

SCHEMAS_DIR="carina-provider-awscc/src/schemas/generated"
DOCS_DIR="generated-docs/awscc"

EXIT_CODE=0

# Collect schema resources: service/resource from .rs files (excluding mod.rs)
schema_resources=()
for rs_file in "$SCHEMAS_DIR"/*/*.rs; do
    [ "$(basename "$rs_file")" = "mod.rs" ] && continue
    svc=$(basename "$(dirname "$rs_file")")
    resource=$(basename "$rs_file" .rs)
    schema_resources+=("$svc/$resource")
done

# Collect doc resources: service/resource from .md files
doc_resources=()
for md_file in "$DOCS_DIR"/*/*.md; do
    [ -f "$md_file" ] || continue
    svc=$(basename "$(dirname "$md_file")")
    resource=$(basename "$md_file" .md)
    doc_resources+=("$svc/$resource")
done

# Check for schemas without docs
for sr in "${schema_resources[@]}"; do
    found=false
    for dr in "${doc_resources[@]}"; do
        if [ "$sr" = "$dr" ]; then
            found=true
            break
        fi
    done
    if [ "$found" = false ]; then
        echo "MISSING DOCS: $sr (schema exists but docs missing)"
        EXIT_CODE=1
    fi
done

# Check for docs without schemas
for dr in "${doc_resources[@]}"; do
    found=false
    for sr in "${schema_resources[@]}"; do
        if [ "$dr" = "$sr" ]; then
            found=true
            break
        fi
    done
    if [ "$found" = false ]; then
        echo "ORPHAN DOCS: $dr (docs exist but schema missing)"
        EXIT_CODE=1
    fi
done

if [ "$EXIT_CODE" = 0 ]; then
    echo "OK: schemas and docs are in sync (${#schema_resources[@]} resources)"
fi

exit $EXIT_CODE
