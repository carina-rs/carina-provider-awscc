#!/bin/bash
# Test: value-returning user-defined function
source "$(dirname "$0")/../../shared/_helpers.sh"

echo "Test: value-returning user function"
echo ""

register_acceptance_work_dir "user_functions/value_fn.crn" "$SCRIPT_DIR/value_fn.crn" WORK_DIR
ACTIVE_WORK_DIR="$WORK_DIR"
cd "$WORK_DIR"

run_step "step1: apply" "$CARINA_BIN" apply --auto-approve .
run_step "step2: plan-verify" "$CARINA_BIN" plan .

assert_state_resource_count "assert: 1 resource created" "1" "$WORK_DIR"

assert_state_value "assert: Name = 'test-web-vpc'" \
    '.resources[0].current.attributes.tags.Name' \
    'test-web-vpc' "$WORK_DIR"

echo "  Cleanup: destroying resources..."
"$CARINA_BIN" destroy --auto-approve . > /dev/null 2>&1 || true
release_acceptance_work_dir "$WORK_DIR" || TEST_FAILED=$((TEST_FAILED + 1))
ACTIVE_WORK_DIR=""

finish_test
