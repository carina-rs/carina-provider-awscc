#!/bin/bash
# Test: if expression with false condition creates no resource
source "$(dirname "$0")/../../shared/_helpers.sh"

echo "Test: if expression (false condition)"
echo ""

register_acceptance_work_dir "if_expression/if_false.crn" "$SCRIPT_DIR/if_false.crn" WORK_DIR
ACTIVE_WORK_DIR="$WORK_DIR"
cd "$WORK_DIR"

run_step "step1: apply" "$CARINA_BIN" apply --auto-approve .

assert_no_state "assert: no state created" "$WORK_DIR"

release_acceptance_work_dir "$WORK_DIR" || TEST_FAILED=$((TEST_FAILED + 1))
ACTIVE_WORK_DIR=""

finish_test
