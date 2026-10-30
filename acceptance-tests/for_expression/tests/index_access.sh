#!/bin/bash
# Test: index access on for expression results (vpcs[0].vpc_id used as subnet dependency)
source "$(dirname "$0")/../../shared/_helpers.sh"

echo "Test: index access on for expression results"
echo ""

register_acceptance_work_dir "for_expression/index_access.crn" "$SCRIPT_DIR/index_access.crn" WORK_DIR
ACTIVE_WORK_DIR="$WORK_DIR"
cd "$WORK_DIR"

run_step "step1: apply" "$CARINA_BIN" apply --auto-approve .
run_step "step2: plan-verify" "$CARINA_BIN" plan .

# 2 VPCs from for + 1 subnet = 3 resources
assert_state_resource_count "assert: 3 resources created" "3" "$WORK_DIR"

# Subnet should exist and reference vpcs[0]'s VPC ID
assert_state_value "assert: subnet Name tag" \
    '[.resources[] | select(.resource_type == "ec2.Subnet")] | .[0].current.attributes.tags.Name' \
    'index-access-test-subnet' "$WORK_DIR"

# Subnet's vpc_id should match vpcs[0]'s vpc_id
assert_state_queries_equal "assert: subnet vpc_id matches vpcs[0].vpc_id" \
    '.resources[] | select(.resource_type == "ec2.Subnet") | .current.attributes.vpc_id' \
    'subnet' \
    '.resources[] | select(.binding == "vpcs[0]") | .current.attributes.vpc_id' \
    'vpcs[0]' "$WORK_DIR"

echo "  Cleanup: destroying resources..."
"$CARINA_BIN" destroy --auto-approve . > /dev/null 2>&1 || true
release_acceptance_work_dir "$WORK_DIR" || TEST_FAILED=$((TEST_FAILED + 1))
ACTIVE_WORK_DIR=""

finish_test
