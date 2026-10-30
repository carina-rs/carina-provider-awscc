#!/bin/bash
# Test: module resources use dot-path addressing in state
source "$(dirname "$0")/../../shared/_helpers.sh"

echo "Test: module dot-path addressing"
echo ""

WORK_DIR=$(mktemp -d)
ACTIVE_WORK_DIR="$WORK_DIR"
cp "$SCRIPT_DIR/basic.crn" "$WORK_DIR/main.crn"
cp -r "$SCRIPT_DIR/modules" "$WORK_DIR/modules"
cd "$WORK_DIR"

run_step "step1: apply" "$CARINA_BIN" apply --auto-approve .
run_step "step2: plan-verify" "$CARINA_BIN" plan .

assert_state_resource_count "assert: 2 resources" "2" "$WORK_DIR"

# An anonymous module call gets a hash-derived network_<hex> prefix. Its
# resources must still use dot-path suffixes, not flattened underscores.
assert_state_value "assert: hashed binding uses dot path (.vpc)" \
    '[.resources[] | select((.binding // "") | test("^network_[0-9a-f]+\\.vpc$"))] | length' \
    '1' "$WORK_DIR"

assert_state_value "assert: hashed binding uses dot path (.subnet)" \
    '[.resources[] | select((.binding // "") | test("^network_[0-9a-f]+\\.subnet$"))] | length' \
    '1' "$WORK_DIR"

# Verify neither stable nor hash-derived prefixes use flattened underscores.
assert_state_value "assert: no underscore-flattened binding" \
    '[.resources[] | select((.binding // "") | test("^network(_[0-9a-f]+)?_(vpc|subnet)$"))] | length' \
    '0' "$WORK_DIR"

# Verify the supported public state output uses schema version 1.
assert_state_value "assert: state schema version = 1" \
    '.schema_version' '1' "$WORK_DIR"

echo "  Cleanup: destroying resources..."
"$CARINA_BIN" destroy --auto-approve . > /dev/null 2>&1 || true
rm -rf "$WORK_DIR"
ACTIVE_WORK_DIR=""

finish_test
