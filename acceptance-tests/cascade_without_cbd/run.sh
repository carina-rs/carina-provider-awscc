#!/usr/bin/env bash
# Cascading update without create_before_destroy test
#
# Tests that when a VPC is replaced (without CBD),
# dependent resources (subnet) appear as cascading updates in the plan.
#
# Usage:
#   aws-vault exec carina-test-000 -- ./carina-provider-awscc/acceptance-tests/cascade_without_cbd/run.sh

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"

source "$SCRIPT_DIR/../shared/_helpers.sh"

STEP1="$SCRIPT_DIR/step1.crn"
STEP2="$SCRIPT_DIR/step2.crn"
WORK_DIR=$(mktemp -d)
trap 'rm -rf "$WORK_DIR"' EXIT

PASS=0
FAIL=0

swap_crn() {
    local source_crn="$1"
    local work_dir="$2"

    cp "$source_crn" "$work_dir/main.crn"
    prepare_work_dir "$work_dir"
}

run_step() {
    local description="$1"
    local source_crn="$2"
    shift 2

    echo "── $description ──"
    swap_crn "$source_crn" "$WORK_DIR"
    if (cd "$WORK_DIR" && "$CARINA_BIN" "$@" .); then
        echo "  ✓ $description"
        PASS=$((PASS + 1))
    else
        echo "  ✗ $description"
        FAIL=$((FAIL + 1))
    fi
}

destroy_work_dir() {
    local any_success=false

    set +e
    echo "── destroy (cleanup) ──"
    swap_crn "$STEP2" "$WORK_DIR"
    if (cd "$WORK_DIR" && "$CARINA_BIN" destroy --auto-approve . 2>&1); then
        any_success=true
    fi
    swap_crn "$STEP1" "$WORK_DIR"
    if (cd "$WORK_DIR" && "$CARINA_BIN" destroy --auto-approve . 2>&1); then
        any_success=true
    fi
    swap_crn "$STEP2" "$WORK_DIR"
    if (cd "$WORK_DIR" && "$CARINA_BIN" destroy --auto-approve . 2>&1); then
        any_success=true
    fi
    swap_crn "$STEP1" "$WORK_DIR"
    if (cd "$WORK_DIR" && "$CARINA_BIN" destroy --auto-approve . 2>&1); then
        any_success=true
    fi
    set -e

    if [ "$any_success" = true ]; then
        echo "  ✓ destroy (cleanup)"
        PASS=$((PASS + 1))
    else
        echo "  ✗ destroy (cleanup)"
        FAIL=$((FAIL + 1))
    fi
}

echo ""
echo "════════════════════════════════════════"
echo " Cascade without create_before_destroy"
echo "════════════════════════════════════════"
echo ""

# Step 1: Apply initial state (VPC + SG + Ingress)
run_step "apply step1 (create VPC + subnet)" "$STEP1" apply --auto-approve

# Step 2: Plan with changed group_description
# The plan should show:
#   -/+ SG (replace, forces replacement)
#   ~ ingress rule (cascading update)
echo ""
echo "── plan step2 (expect cascade) ──"
swap_crn "$STEP2" "$WORK_DIR"
PLAN_OUTPUT=$(cd "$WORK_DIR" && "$CARINA_BIN" plan . 2>&1) || true
echo "$PLAN_OUTPUT"

if echo "$PLAN_OUTPUT" | grep -q "create before destroy"; then
    echo "  ✓ create_before_destroy auto-detected in plan"
    PASS=$((PASS + 1))
else
    echo "  ✗ create_before_destroy NOT auto-detected in plan"
    FAIL=$((FAIL + 1))
fi

# Step 2: Apply (VPC CBD replace + subnet replace)
run_step "apply step2 (replace VPC + subnet)" "$STEP2" apply --auto-approve

# Step 2: Plan verify (should show no changes after apply)
echo ""
echo "── plan-verify step2 ──"
swap_crn "$STEP2" "$WORK_DIR"
VERIFY_OUTPUT=$(cd "$WORK_DIR" && "$CARINA_BIN" plan . 2>&1) || true
echo "$VERIFY_OUTPUT"

if echo "$VERIFY_OUTPUT" | grep -q "No changes"; then
    echo "  ✓ plan-verify: no changes (idempotent)"
    PASS=$((PASS + 1))
else
    echo "  ✗ plan-verify: unexpected changes detected"
    FAIL=$((FAIL + 1))
fi

# Cleanup: destroy
destroy_work_dir

echo ""
echo "════════════════════════════════════════"
echo "Total: $PASS passed, $FAIL failed"
echo "════════════════════════════════════════"

[ "$FAIL" -eq 0 ]
