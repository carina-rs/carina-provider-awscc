#!/bin/bash
# Standalone acceptance test for publishing CloudFront Function updates.
#
# Usage:
#   env AWS_PROFILE="<profile>" ./run.sh
#
# This suite is intentionally skipped by acceptance-tests/run-tests.sh because
# its directory contains a custom run.sh.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/../shared/_helpers.sh"

FUNCTION_NAME="carina-acc-test-fn-publish"
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
        # Aggregate failures so set -e does not skip later assertions or cleanup.
        return 0
    fi
}

plan_verify() {
    local work_dir="$1"

    printf "  %-55s " "plan-verify"
    local output
    local rc=0
    output=$(cd "$work_dir" && with_account_creds run_with_timeout 600 "$CARINA_BIN" plan --detailed-exitcode . 2>&1) || rc=$?

    if [ "$rc" -eq 0 ]; then
        echo "OK"
        TOTAL_PASSED=$((TOTAL_PASSED + 1))
        return 0
    fi

    echo "FAIL"
    if [ "$rc" -eq 2 ]; then
        echo "  ERROR: Post-apply plan detected changes:"
    fi
    echo "  $output"
    TOTAL_FAILED=$((TOTAL_FAILED + 1))
    # Aggregate failures so set -e does not skip later assertions or cleanup.
    return 0
}

install_config() {
    local source_crn="$1"
    local injected_dir
    injected_dir=$(inject_provider_source "$source_crn")
    cp "$injected_dir/main.crn" "$ACTIVE_WORK_DIR/main.crn"
    rm -rf "$injected_dir"
}

apply_function() {
    cd "$ACTIVE_WORK_DIR"
    with_account_creds run_with_timeout 600 "$CARINA_BIN" apply --auto-approve .
}

destroy_function() {
    cd "$ACTIVE_WORK_DIR"
    with_account_creds run_with_timeout 600 "$CARINA_BIN" destroy --auto-approve .
}

extract_function_code() {
    local source_crn="$1"
    local output_file="$2"
    awk '
        in_code && $0 == "JS" { exit }
        in_code {
            if (wrote_line) {
                printf "\n"
            }
            printf "%s", $0
            wrote_line = 1
            next
        }
        /^[[:space:]]*function_code[[:space:]]*=/ { in_code = 1 }
    ' "$source_crn" > "$output_file"
}

assert_stage_code() {
    local stage="$1"
    local expected_file="$2"
    local actual_file="$ACTIVE_WORK_DIR/function-${stage}.js"

    with_account_creds aws cloudfront get-function \
        --name "$FUNCTION_NAME" --stage "$stage" "$actual_file" >/dev/null
    if cmp -s "$expected_file" "$actual_file"; then
        return 0
    fi

    echo "CloudFront $stage code differs from step2 function_code:" >&2
    diff -u "$expected_file" "$actual_file" >&2 || true
    return 1
}

assert_function_gone() {
    local found
    found=$(with_account_creds aws cloudfront list-functions \
        --query "FunctionList.Items[?Name==\`$FUNCTION_NAME\`].Name" --output text)
    [ -z "$found" ] || [ "$found" = "None" ]
}

cleanup() {
    if [ -n "$ACTIVE_WORK_DIR" ] && [ -d "$ACTIVE_WORK_DIR" ]; then
        set +e
        echo ""
        echo "  Cleanup: destroying CloudFront Function resources..."
        (cd "$ACTIVE_WORK_DIR" && with_account_creds run_with_timeout 600 "$CARINA_BIN" destroy --auto-approve . 2>&1)
        (cd "$ACTIVE_WORK_DIR" && with_account_creds run_with_timeout 600 "$CARINA_BIN" destroy --auto-approve . 2>&1)
        rm -rf "$ACTIVE_WORK_DIR"
        ACTIVE_WORK_DIR=""
    fi
}
trap cleanup EXIT

WORK_DIR="$(mktemp -d)"
ACTIVE_WORK_DIR="$WORK_DIR"
EXPECTED_CODE="$WORK_DIR/expected-step2.js"

install_config "$SCRIPT_DIR/step1.crn"
extract_function_code "$SCRIPT_DIR/step2.crn" "$EXPECTED_CODE"

echo "Running CloudFront Function publish acceptance test with AWS_PROFILE=${AWS_PROFILE:-}"
echo ""

record_step "init" "$CARINA_BIN" init "$WORK_DIR"
record_step "step1: apply initial function" apply_function
plan_verify "$WORK_DIR"

install_config "$SCRIPT_DIR/step2.crn"
record_step "step2: apply function_code update" apply_function
plan_verify "$WORK_DIR"
record_step "assert: LIVE serves step2 code" assert_stage_code LIVE "$EXPECTED_CODE"
record_step "assert: DEVELOPMENT serves step2 code" assert_stage_code DEVELOPMENT "$EXPECTED_CODE"
record_step "destroy" destroy_function
record_step "assert: function is gone" assert_function_gone

rm -rf "$WORK_DIR"
ACTIVE_WORK_DIR=""

echo ""
echo "Results: $TOTAL_PASSED passed, $TOTAL_FAILED failed"
if [ "$TOTAL_FAILED" -gt 0 ]; then
    exit 1
fi
