#!/bin/bash
# Test: if expression with false condition creates no resource
source "$(dirname "$0")/../../shared/_helpers.sh"

echo "Test: if expression (false condition)"
echo ""

WORK_DIR=$(mktemp -d)
ACTIVE_WORK_DIR="$WORK_DIR"
cp "$SCRIPT_DIR/if_false.crn" "$WORK_DIR/main.crn"
cd "$WORK_DIR"

run_step "step1: apply" "$CARINA_BIN" apply --auto-approve .

assert_no_state "assert: no state created" "$WORK_DIR"

rm -rf "$WORK_DIR"
ACTIVE_WORK_DIR=""

finish_test
