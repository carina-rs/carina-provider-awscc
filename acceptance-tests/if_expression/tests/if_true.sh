#!/bin/bash
# Test: if expression with true condition creates resource
source "$(dirname "$0")/../../shared/_helpers.sh"

echo "Test: if expression (true condition)"
echo ""

register_acceptance_work_dir "if_expression/if_true.crn" "$SCRIPT_DIR/if_true.crn" WORK_DIR
ACTIVE_WORK_DIR="$WORK_DIR"
cd "$WORK_DIR"

run_step "step1: apply" "$CARINA_BIN" apply --auto-approve .
run_step "step2: plan-verify" "$CARINA_BIN" plan .

assert_state_resource_count "assert: 1 resource created" "1" "$WORK_DIR"

assert_state_value "assert: vpc Name = 'if-true-test'" \
    '.resources[0].current.attributes.tags.Name' \
    'if-true-test' "$WORK_DIR"

echo "  Cleanup: destroying resources..."
"$CARINA_BIN" destroy --auto-approve . > /dev/null 2>&1 || true
release_acceptance_work_dir "$WORK_DIR" || TEST_FAILED=$((TEST_FAILED + 1))
ACTIVE_WORK_DIR=""

finish_test
