#!/bin/bash
# Test: split() function
source "$(dirname "$0")/../../shared/_helpers.sh"

echo "Test: split() function"
echo ""

register_acceptance_work_dir "builtin_functions/split.crn" "$SCRIPT_DIR/split.crn" WORK_DIR
ACTIVE_WORK_DIR="$WORK_DIR"
cd "$WORK_DIR"

run_step "step1: apply" "$CARINA_BIN" apply --auto-approve .
run_step "step2: plan-verify" "$CARINA_BIN" plan .
assert_state_value \
    "assert: tag Name = 'web-test-vpc'" \
    '.resources[0].current.attributes.tags.Name' \
    'web-test-vpc' \
    "$WORK_DIR"

echo "  Cleanup: destroying resources..."
"$CARINA_BIN" destroy --auto-approve . > /dev/null 2>&1 || true
release_acceptance_work_dir "$WORK_DIR" || TEST_FAILED=$((TEST_FAILED + 1))
ACTIVE_WORK_DIR=""

finish_test
