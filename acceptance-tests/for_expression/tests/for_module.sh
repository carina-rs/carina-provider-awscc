#!/bin/bash
# Test: for expression with module calls (for + module iteration over map)
source "$(dirname "$0")/../../shared/_helpers.sh"

echo "Test: for expression with module calls"
echo ""

register_acceptance_work_dir "for_expression/for_module.crn" "$SCRIPT_DIR/for_module.crn" WORK_DIR
ACTIVE_WORK_DIR="$WORK_DIR"
cp -r "$SCRIPT_DIR/modules" "$WORK_DIR/modules"
cd "$WORK_DIR"

run_step "step1: apply" "$CARINA_BIN" apply --auto-approve .
run_step "step2: plan-verify" "$CARINA_BIN" plan .

# 2 iterations x 1 resource per module (vpc) = 2 resources
assert_state_resource_count "assert: 2 resources created" "2" "$WORK_DIR"

# vpc_only's VPC is anonymous, so it has no binding. Its identity is the
# canonical map-instance prefix followed by the schema-derived identity.
DEV_VPC='[.resources[] | select(.resource_type == "ec2.Vpc" and (.identity | startswith("networks.dev.")))]'
STG_VPC='[.resources[] | select(.resource_type == "ec2.Vpc" and (.identity | startswith("networks.stg.")))]'

assert_state_value "assert: networks.dev anonymous VPC exists" \
    "$DEV_VPC | length" \
    '1' "$WORK_DIR"

assert_state_value "assert: networks.stg anonymous VPC exists" \
    "$STG_VPC | length" \
    '1' "$WORK_DIR"

# Verify CIDRs are correct
assert_state_value "assert: dev vpc cidr_block = '10.0.0.0/16'" \
    "$DEV_VPC | .[0].current.attributes.cidr_block" \
    '10.0.0.0/16' "$WORK_DIR"

assert_state_value "assert: stg vpc cidr_block = '10.1.0.0/16'" \
    "$STG_VPC | .[0].current.attributes.cidr_block" \
    '10.1.0.0/16' "$WORK_DIR"

# Verify tags contain environment name
assert_state_value "assert: dev vpc Name tag" \
    "$DEV_VPC | .[0].current.attributes.tags.Name" \
    'for-module-test-dev' "$WORK_DIR"

assert_state_value "assert: stg vpc Name tag" \
    "$STG_VPC | .[0].current.attributes.tags.Name" \
    'for-module-test-stg' "$WORK_DIR"

echo "  Cleanup: destroying resources..."
"$CARINA_BIN" destroy --auto-approve . > /dev/null 2>&1 || true
release_acceptance_work_dir "$WORK_DIR" || TEST_FAILED=$((TEST_FAILED + 1))
ACTIVE_WORK_DIR=""

finish_test
