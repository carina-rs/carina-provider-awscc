#!/bin/bash
# Multi-step acceptance tests for AWSCC tag deletion via CloudControl "remove" patch
#
# Usage:
#   env AWS_PROFILE="<profile>" ./run.sh [filter]
#
# AWS authentication:
#   Set AWS_PROFILE to one of the carina-test-00X profiles. The script
#   resolves fresh STS credentials from that profile before each carina
#   invocation (via `aws configure export-credentials`), so long-running
#   apply/destroy cycles are not bound to a single SSO access-token
#   lifetime. The first run in a session needs `aws sso login --sso-session carina`.
#
# Tests:
#   ec2_vpc              - Remove a tag from VPC
#   s3_bucket            - Remove a tag from S3 bucket
#   iam_role             - Remove a tag from IAM role
#   ec2_nat_gateway      - Remove a tag from NAT Gateway
#   ec2_vpc_all_tags     - Remove entire tags block from VPC
#   s3_bucket_all_tags   - Remove entire tags block from S3 bucket
#   iam_role_all_tags    - Remove entire tags block from IAM role
#   ec2_nat_gateway_all_tags - Remove entire tags block from NAT Gateway
#
# Filter (optional): substring to match test names (e.g. "ec2_vpc", "s3_bucket")

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../../.." && pwd)"
FILTER="${1:-}"

source "$SCRIPT_DIR/../shared/_helpers.sh"

TOTAL_PASSED=0
TOTAL_FAILED=0

# ── with_account_creds: resolve credentials for $AWS_PROFILE into env vars ──
# and exec the given command. See run-tests.sh for rationale.
with_account_creds() {
    local account="${AWS_PROFILE:-}"
    if [ -z "$account" ]; then
        echo "ERROR: AWS_PROFILE is not set; cannot resolve credentials" >&2
        return 1
    fi
    local creds
    if ! creds=$(aws configure export-credentials --profile "$account" --format env-no-export 2>&1); then
        echo "ERROR: aws configure export-credentials failed for $account: $creds" >&2
        return 1
    fi
    (
        set -a
        eval "$creds"
        set +a
        unset AWS_PROFILE
        "$@"
    )
}

# Track active work dir for signal cleanup
ACTIVE_WORK_DIR=""

signal_cleanup() {
    if [ -n "$ACTIVE_WORK_DIR" ] && [ -d "$ACTIVE_WORK_DIR" ]; then
        set +e
        echo ""
        echo "Interrupted. Cleaning up resources..."
        (cd "$ACTIVE_WORK_DIR" && with_account_creds "$CARINA_BIN" destroy --auto-approve . 2>&1)
        (cd "$ACTIVE_WORK_DIR" && with_account_creds "$CARINA_BIN" destroy --auto-approve . 2>&1)
        rm -rf "$ACTIVE_WORK_DIR"
        ACTIVE_WORK_DIR=""
    fi
    exit 1
}

trap signal_cleanup INT TERM

run_step() {
    local work_dir="$1"
    local description="$2"
    local command="$3"
    local extra_args="${4:-}"

    printf "  %-55s " "$description"

    local output
    if output=$(cd "$work_dir" && with_account_creds "$CARINA_BIN" $command $extra_args . 2>&1); then
        echo "OK"
        TOTAL_PASSED=$((TOTAL_PASSED + 1))
        return 0
    else
        echo "FAIL"
        echo "  ERROR: $output"
        TOTAL_FAILED=$((TOTAL_FAILED + 1))
        return 1
    fi
}

run_plan_verify() {
    local work_dir="$1"
    local description="$2"

    printf "  %-55s " "$description"

    local output
    local rc
    output=$(cd "$work_dir" && with_account_creds "$CARINA_BIN" plan --detailed-exitcode . 2>&1) || rc=$?
    rc=${rc:-0}

    if [ $rc -eq 2 ]; then
        echo "FAIL"
        echo "  ERROR: Post-apply plan detected changes (not idempotent):"
        echo "  $output"
        TOTAL_FAILED=$((TOTAL_FAILED + 1))
        return 1
    elif [ $rc -ne 0 ]; then
        echo "FAIL"
        echo "  ERROR: $output"
        TOTAL_FAILED=$((TOTAL_FAILED + 1))
        return 1
    fi

    echo "OK"
    TOTAL_PASSED=$((TOTAL_PASSED + 1))
    return 0
}

# destroy_work_dir: destroy resources in work_dir, retrying to handle dependencies
# Returns 0 if at least one destroy succeeded, 1 if ALL failed
# (Named distinctly from `cleanup` so the EXIT trap in _helpers.sh keeps using
# its own cleanup() — overriding it would call this with no args at exit and
# emit "No .crn files found in .".)
destroy_work_dir() {
    local work_dir="$1"
    local any_success=false

    # Disable set -e to ensure all destroy attempts run
    set +e
    echo "  Cleanup: destroying resources..."
    if (cd "$work_dir" && with_account_creds "$CARINA_BIN" destroy --auto-approve . 2>&1); then
        any_success=true
    fi
    # Retry: resources may have dependencies that prevent deletion on first pass
    if (cd "$work_dir" && with_account_creds "$CARINA_BIN" destroy --auto-approve . 2>&1); then
        any_success=true
    fi
    set -e

    if [ "$any_success" = false ]; then
        return 1
    fi
    return 0
}

# Swap the active .crn into work_dir/main.crn (with provider source injected).
# Args: source_crn target_work_dir
swap_crn() {
    local src="$1"
    local target="$2"
    local injected_dir
    injected_dir=$(inject_provider_source "$src")
    cp "$injected_dir/main.crn" "$target/main.crn"
    rm -rf "$injected_dir"
}

# Run a single multi-step tag deletion test
# Args: test_name step1_crn step2_crn description
run_test() {
    local test_name="$1"
    local step1="$2"
    local step2="$3"
    local desc="$4"

    # Apply filter
    if [ -n "$FILTER" ] && [[ "$test_name" != *"$FILTER"* ]]; then
        return 0
    fi

    local work_dir
    work_dir=$(mktemp -d)

    # Register for signal cleanup
    ACTIVE_WORK_DIR="$work_dir"

    echo "$desc"
    echo ""

    # Load step1 and initialize (creates backend lock + installs providers)
    swap_crn "$step1" "$work_dir"
    if ! (cd "$work_dir" && with_account_creds "$CARINA_BIN" init . >/dev/null 2>&1); then
        echo "  step1: init                                             FAIL"
        TOTAL_FAILED=$((TOTAL_FAILED + 1))
        rm -rf "$work_dir"
        ACTIVE_WORK_DIR=""
        return 1
    fi

    # Step 1: Apply initial config (resource with two tags)
    if ! run_step "$work_dir" "step1: apply initial (two tags)" "apply" "--auto-approve"; then
        destroy_work_dir "$work_dir"
        rm -rf "$work_dir"
        ACTIVE_WORK_DIR=""
        return 1
    fi

    # Step 1b: Plan-verify initial state
    if ! run_plan_verify "$work_dir" "step1: plan-verify initial"; then
        destroy_work_dir "$work_dir"
        rm -rf "$work_dir"
        ACTIVE_WORK_DIR=""
        return 1
    fi

    # Swap in step2 config (providers are the same, lock already present)
    swap_crn "$step2" "$work_dir"

    # Step 2: Apply modified config (tag(s) removed)
    if ! run_step "$work_dir" "step2: apply tag removal" "apply" "--auto-approve"; then
        destroy_work_dir "$work_dir"
        rm -rf "$work_dir"
        ACTIVE_WORK_DIR=""
        return 1
    fi

    # Step 3: Plan-verify after tag removal (must be idempotent)
    if ! run_plan_verify "$work_dir" "step3: plan-verify after tag removal"; then
        destroy_work_dir "$work_dir"
        rm -rf "$work_dir"
        ACTIVE_WORK_DIR=""
        return 1
    fi

    # Step 4: Destroy
    if ! destroy_work_dir "$work_dir"; then
        echo "  WARNING: All destroy attempts failed. Preserving work dir for debugging:"
        echo "    $work_dir"
        TOTAL_FAILED=$((TOTAL_FAILED + 1))
        ACTIVE_WORK_DIR=""
        echo ""
        return 1
    fi

    rm -rf "$work_dir"
    ACTIVE_WORK_DIR=""
    echo ""
}

echo "tag_deletion multi-step acceptance tests (AWSCC)"
echo "════════════════════════════════════════"
echo ""

# Test 1: EC2 VPC - tag removal via CloudControl "remove" patch
run_test "ec2_vpc" \
    "$SCRIPT_DIR/ec2_vpc_step1.crn" \
    "$SCRIPT_DIR/ec2_vpc_step2.crn" \
    "Test 1: EC2 VPC (remove one tag key)"

# Test 2: S3 Bucket - tag removal via CloudControl "remove" patch
run_test "s3_bucket" \
    "$SCRIPT_DIR/s3_bucket_step1.crn" \
    "$SCRIPT_DIR/s3_bucket_step2.crn" \
    "Test 2: S3 Bucket (remove one tag key)"

# Test 3: IAM Role - tag removal via CloudControl "remove" patch
run_test "iam_role" \
    "$SCRIPT_DIR/iam_role_step1.crn" \
    "$SCRIPT_DIR/iam_role_step2.crn" \
    "Test 3: IAM Role (remove one tag key)"

# Test 4: EC2 NAT Gateway - tag removal via CloudControl "remove" patch
run_test "ec2_nat_gateway" \
    "$SCRIPT_DIR/ec2_nat_gateway_step1.crn" \
    "$SCRIPT_DIR/ec2_nat_gateway_step2.crn" \
    "Test 4: EC2 NAT Gateway (remove one tag key)"

# Test 5: EC2 VPC - entire tags block removal
run_test "ec2_vpc_all_tags" \
    "$SCRIPT_DIR/ec2_vpc_step1.crn" \
    "$SCRIPT_DIR/ec2_vpc_step3.crn" \
    "Test 5: EC2 VPC (remove entire tags block)"

# Test 6: S3 Bucket - entire tags block removal
run_test "s3_bucket_all_tags" \
    "$SCRIPT_DIR/s3_bucket_step1.crn" \
    "$SCRIPT_DIR/s3_bucket_step3.crn" \
    "Test 6: S3 Bucket (remove entire tags block)"

# Test 7: IAM Role - entire tags block removal
run_test "iam_role_all_tags" \
    "$SCRIPT_DIR/iam_role_step1.crn" \
    "$SCRIPT_DIR/iam_role_step3.crn" \
    "Test 7: IAM Role (remove entire tags block)"

# Test 8: EC2 NAT Gateway - entire tags block removal
run_test "ec2_nat_gateway_all_tags" \
    "$SCRIPT_DIR/ec2_nat_gateway_step1.crn" \
    "$SCRIPT_DIR/ec2_nat_gateway_step3.crn" \
    "Test 8: EC2 NAT Gateway (remove entire tags block)"

echo "════════════════════════════════════════"
echo "Total: $TOTAL_PASSED passed, $TOTAL_FAILED failed"
echo "════════════════════════════════════════"

if [ $TOTAL_FAILED -gt 0 ]; then
    exit 1
fi
