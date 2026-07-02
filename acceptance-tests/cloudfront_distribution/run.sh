#!/bin/bash
# Standalone acceptance test for CloudFront distribution.
#
# Usage:
#   env AWS_PROFILE="<profile>" ./run.sh
#
# CloudFront distribution create/delete can take 5-15 minutes, so this suite is
# intentionally skipped by acceptance-tests/run-tests.sh and must be run alone.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/../shared/_helpers.sh"

TOTAL_PASSED=0
TOTAL_FAILED=0
ACTIVE_WORK_DIR=""

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

run_with_timeout() {
    local seconds="$1"
    shift

    if command -v timeout >/dev/null 2>&1; then
        timeout "$seconds" "$@"
    elif command -v gtimeout >/dev/null 2>&1; then
        gtimeout "$seconds" "$@"
    else
        "$@"
    fi
}

record_step() {
    local description="$1"
    shift

    printf "  %-55s " "$description"
    local output
    if output=$("$@" 2>&1); then
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

plan_verify() {
    local work_dir="$1"

    printf "  %-55s " "plan-verify"
    local output
    local rc=0
    output=$(cd "$work_dir" && with_account_creds run_with_timeout 3600 "$CARINA_BIN" plan --detailed-exitcode . 2>&1) || rc=$?

    if [ $rc -eq 0 ]; then
        echo "OK"
        TOTAL_PASSED=$((TOTAL_PASSED + 1))
        return 0
    fi

    echo "FAIL"
    if [ $rc -eq 2 ]; then
        echo "  ERROR: Post-apply plan detected changes:"
    fi
    echo "  $output"
    TOTAL_FAILED=$((TOTAL_FAILED + 1))
    return 1
}

apply_distribution() {
    cd "$ACTIVE_WORK_DIR"
    with_account_creds run_with_timeout 3600 "$CARINA_BIN" apply --auto-approve .
}

destroy_distribution() {
    cd "$ACTIVE_WORK_DIR"
    with_account_creds run_with_timeout 3600 "$CARINA_BIN" destroy --auto-approve .
}

cleanup() {
    if [ -n "$ACTIVE_WORK_DIR" ] && [ -d "$ACTIVE_WORK_DIR" ]; then
        set +e
        echo ""
        echo "  Cleanup: destroying CloudFront distribution resources..."
        (cd "$ACTIVE_WORK_DIR" && with_account_creds run_with_timeout 3600 "$CARINA_BIN" destroy --auto-approve . 2>&1)
        (cd "$ACTIVE_WORK_DIR" && with_account_creds run_with_timeout 3600 "$CARINA_BIN" destroy --auto-approve . 2>&1)
        rm -rf "$ACTIVE_WORK_DIR"
        ACTIVE_WORK_DIR=""
    fi
}
trap cleanup EXIT

WORK_DIR="$(mktemp -d)"
ACTIVE_WORK_DIR="$WORK_DIR"

INJECTED_DIR="$(inject_provider_source "$SCRIPT_DIR/basic.crn")"
cp "$INJECTED_DIR/main.crn" "$WORK_DIR/main.crn"
rm -rf "$INJECTED_DIR"

echo "Running CloudFront distribution acceptance test with AWS_PROFILE=${AWS_PROFILE:-}"
echo ""

record_step "init" "$CARINA_BIN" init "$WORK_DIR"
record_step "apply" apply_distribution
plan_verify "$WORK_DIR"
record_step "destroy" destroy_distribution

rm -rf "$WORK_DIR"
ACTIVE_WORK_DIR=""

echo ""
echo "Results: $TOTAL_PASSED passed, $TOTAL_FAILED failed"
if [ "$TOTAL_FAILED" -gt 0 ]; then
    exit 1
fi
