#!/bin/bash
# Test: cidr_subnet() function
source "$(dirname "$0")/../../shared/_helpers.sh"

echo "Test: cidr_subnet() function"
echo ""

register_acceptance_work_dir "builtin_functions/cidr_subnet.crn" "$SCRIPT_DIR/cidr_subnet.crn" WORK_DIR
ACTIVE_WORK_DIR="$WORK_DIR"
cd "$WORK_DIR"

run_step "step1: apply" "$CARINA_BIN" apply --auto-approve .
run_step "step2: plan-verify" "$CARINA_BIN" plan .
assert_state_value \
    "assert: subnet cidr_block = '10.0.1.0/24'" \
    '.resources[] | select(.resource_type == "ec2.Subnet") | .current.attributes.cidr_block' \
    '10.0.1.0/24' \
    "$WORK_DIR"

echo "  Cleanup: destroying resources..."
"$CARINA_BIN" destroy --auto-approve . > /dev/null 2>&1 || true
release_acceptance_work_dir "$WORK_DIR" || TEST_FAILED=$((TEST_FAILED + 1))
ACTIVE_WORK_DIR=""

finish_test
