#!/bin/bash
# Test: local let bindings inside resource blocks
source "$(dirname "$0")/../../shared/_helpers.sh"

echo "Test: local let bindings inside resource blocks"
echo ""

register_acceptance_work_dir "builtin_functions/local_let.crn" "$SCRIPT_DIR/local_let.crn" WORK_DIR
ACTIVE_WORK_DIR="$WORK_DIR"
cd "$WORK_DIR"

run_step "step1: apply" "$CARINA_BIN" apply --auto-approve .
run_step "step2: plan-verify" "$CARINA_BIN" plan .
assert_state_value "assert: tag Name = 'local-let-production'" '.resources[0].current.attributes.tags.Name' 'local-let-production' "$WORK_DIR"
assert_state_value "assert: tag Env = 'PRODUCTION'" '.resources[0].current.attributes.tags.Env' 'PRODUCTION' "$WORK_DIR"

echo "  Cleanup: destroying resources..."
"$CARINA_BIN" destroy --auto-approve . > /dev/null 2>&1 || true
release_acceptance_work_dir "$WORK_DIR" || TEST_FAILED=$((TEST_FAILED + 1))
ACTIVE_WORK_DIR=""

finish_test
