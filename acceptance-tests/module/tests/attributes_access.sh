#!/bin/bash
# Test: module attributes access (net.vpc_id used to create dependent route table)
source "$(dirname "$0")/../../shared/_helpers.sh"

echo "Test: module attributes access"
echo ""

register_acceptance_work_dir "module/attributes_access.crn" "$SCRIPT_DIR/attributes_access.crn" WORK_DIR
ACTIVE_WORK_DIR="$WORK_DIR"
cp -r "$SCRIPT_DIR/modules" "$WORK_DIR/modules"
cd "$WORK_DIR"

run_step "step1: apply" "$CARINA_BIN" apply --auto-approve .
run_step "step2: plan-verify" "$CARINA_BIN" plan .

# Module produces vpc + subnet + virtual resource, plus route_table = 4 resources
# (virtual resources are not in state)
assert_state_resource_count "assert: 3 resources created" "3" "$WORK_DIR"

# Verify route table exists with correct tag
assert_state_value "assert: route_table Name tag" \
    '[.resources[] | select(.resource_type == "ec2.RouteTable")] | .[0].current.attributes.tags.Name' \
    'module-attr-test' "$WORK_DIR"

# Verify route table's vpc_id matches the module's vpc
assert_state_queries_equal "assert: route_table vpc_id matches module vpc" \
    '.resources[] | select(.resource_type == "ec2.RouteTable") | .current.attributes.vpc_id' \
    'route_table' \
    '.resources[] | select(.resource_type == "ec2.Vpc") | .current.attributes.vpc_id' \
    'vpc' "$WORK_DIR"

echo "  Cleanup: destroying resources..."
"$CARINA_BIN" destroy --auto-approve . > /dev/null 2>&1 || true
release_acceptance_work_dir "$WORK_DIR" || TEST_FAILED=$((TEST_FAILED + 1))
ACTIVE_WORK_DIR=""

finish_test
