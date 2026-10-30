#!/bin/bash
# Test: trim() function
source "$(dirname "$0")/../../shared/_helpers.sh"

echo "Test: trim() function"
echo ""

register_acceptance_work_dir "builtin_functions/trim.crn" "$SCRIPT_DIR/trim.crn" WORK_DIR
ACTIVE_WORK_DIR="$WORK_DIR"
cd "$WORK_DIR"

run_step "step1: apply" "$CARINA_BIN" apply --auto-approve .
run_step "step2: plan-verify" "$CARINA_BIN" plan .
assert_state_value "assert: tag Name = 'trim-test-vpc'" '.resources[0].current.attributes.tags.Name' 'trim-test-vpc' "$WORK_DIR"

echo "  Cleanup: destroying resources..."
"$CARINA_BIN" destroy --auto-approve . > /dev/null 2>&1 || true
release_acceptance_work_dir "$WORK_DIR" || TEST_FAILED=$((TEST_FAILED + 1))
ACTIVE_WORK_DIR=""

finish_test
