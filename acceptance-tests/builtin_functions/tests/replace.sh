#!/bin/bash
# Test: replace() function
source "$(dirname "$0")/../../shared/_helpers.sh"

echo "Test: replace() function"
echo ""

register_acceptance_work_dir "builtin_functions/replace.crn" "$SCRIPT_DIR/replace.crn" WORK_DIR
ACTIVE_WORK_DIR="$WORK_DIR"
cd "$WORK_DIR"

run_step "step1: apply" "$CARINA_BIN" apply --auto-approve .
run_step "step2: plan-verify" "$CARINA_BIN" plan .
assert_state_value \
    "assert: tag Name = 'web_test_vpc'" \
    '.resources[0].current.attributes.tags.Name' \
    'web_test_vpc' \
    "$WORK_DIR"

echo "  Cleanup: destroying resources..."
"$CARINA_BIN" destroy --auto-approve . > /dev/null 2>&1 || true
release_acceptance_work_dir "$WORK_DIR" || TEST_FAILED=$((TEST_FAILED + 1))
ACTIVE_WORK_DIR=""

finish_test
