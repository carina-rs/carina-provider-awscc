#!/bin/bash
# Test deep-cleanup failure reporting and continuation without making AWS calls.
#
# Usage:
#   bash acceptance-tests/shell-tests/deep_cleanup_failure_visibility.sh
#
# This deliberately does not source acceptance-tests/shared/_helpers.sh: that
# helper resolves a carina binary and WASM provider artifacts at source time.
# This test requires jq, but must stay runnable in bare CI with no Rust
# toolchain, no carina binary, and no AWS credentials.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
RUN_TESTS_SH="$SCRIPT_DIR/../run-tests.sh"

WORK_DIR="$(mktemp -d)"
FUNCTIONS_FILE="$WORK_DIR/functions.sh"
COMMAND_LOG="$WORK_DIR/aws-calls.log"
STDOUT_LOG="$WORK_DIR/stdout.log"
STDERR_LOG="$WORK_DIR/stderr.log"

cleanup() {
    rm -rf "$WORK_DIR"
}
trap cleanup EXIT

FAILURES=0
# Unlike deep_cleanup_wafv2.sh, accumulate assertion failures and report them together at the end.
fail() {
    echo "FAIL: $*" >&2
    FAILURES=$((FAILURES + 1))
}

if ! command -v jq >/dev/null 2>&1 || ! jq --version >/dev/null 2>&1; then
    echo "FAIL: jq is required because the extracted cleanup helpers parse AWS JSON with jq" >&2
    exit 1
fi

extract_function() {
    local name="$1"
    awk -v name="$name" '
        $0 ~ "^" name "\\(\\) \\{" {
            in_fn = 1
        }
        in_fn {
            print
        }
        in_fn && /^\}/ {
            found = 1
            exit
        }
        END {
            exit found ? 0 : 1
        }
    ' "$RUN_TESTS_SH"
}

for guarded_fn in \
    deep_cleanup_account \
    deep_cleanup_list_wafv2_web_acls \
    deep_cleanup_disassociate_wafv2_web_acls \
    deep_cleanup_delete_wafv2_web_acls
do
    direct_calls_log="$WORK_DIR/direct-${guarded_fn}-aws-calls.log"
    if extract_function "$guarded_fn" | grep -nF "with_account_creds" > "$direct_calls_log"; then
        echo "FAIL: $guarded_fn must route AWS calls through deep-cleanup helpers" >&2
        sed 's/^/  /' "$direct_calls_log" >&2
        exit 1
    fi
done

for fn in \
    deep_cleanup_enumerate \
    _deep_cleanup_delete \
    deep_cleanup_delete_resource \
    deep_cleanup_delete_supporting \
    deep_cleanup_list_wafv2_web_acls \
    deep_cleanup_disassociate_wafv2_web_acls \
    deep_cleanup_delete_wafv2_web_acls \
    deep_cleanup_account
do
    if ! extract_function "$fn" >> "$FUNCTIONS_FILE"; then
        echo "FAIL: failed to extract $fn from run-tests.sh" >&2
        exit 1
    fi
done

LOAD_BALANCER_ARN="arn:aws:elasticloadbalancing:us-east-1:123456789012:loadbalancer/app/acceptance-test-stubborn/111"
DDB_TABLE="acceptance-test-later-table"
SUCCESS_STDERR_WARNING="urllib3 v2 only supports OpenSSL"
SUCCESS_STDERR_MODE="none"
CLEANUP_SCENARIO="main"

with_account_creds() {
    shift
    local command="$*"
    echo "$command" >> "$COMMAND_LOG"

    case "$command" in
        "aws wafv2 list-web-acls"*)
            if [ "$SUCCESS_STDERR_MODE" = "wafv2-json" ]; then
                echo "$SUCCESS_STDERR_WARNING" >&2
            fi
            if [ "$SUCCESS_STDERR_MODE" = "wafv2-malformed" ]; then
                echo "not-json"
            else
                printf '{"WebACLs":[]}\n'
            fi
            ;;
        "aws elbv2 describe-load-balancers"*)
            if [ "$SUCCESS_STDERR_MODE" = "slow-enumeration" ]; then
                : > "$WORK_DIR/slow-enumeration.started"
                while [ ! -f "$WORK_DIR/slow-enumeration.stop" ]; do
                    command sleep 0.01
                done
                : > "$WORK_DIR/slow-enumeration.finished"
                echo "$LOAD_BALANCER_ARN"
                return 0
            fi
            if [ "$SUCCESS_STDERR_MODE" = "resource-list" ]; then
                echo "$SUCCESS_STDERR_WARNING" >&2
            fi
            if [ "$SUCCESS_STDERR_MODE" = "resource-list" ] || [ "$CLEANUP_SCENARIO" = "main" ]; then
                echo "$LOAD_BALANCER_ARN"
            fi
            ;;
        "aws elbv2 delete-load-balancer"*)
            echo "An error occurred (ResourceInUseException) when calling the DeleteLoadBalancer operation: $LOAD_BALANCER_ARN is still in use" >&2
            return 254
            ;;
        "aws elbv2 describe-target-groups"*)
            if [ "$CLEANUP_SCENARIO" = "main" ]; then
                echo "An error occurred (AccessDeniedException) when calling the DescribeTargetGroups operation: failure-account is not authorized" >&2
                return 254
            fi
            ;;
        "aws ec2 describe-vpcs"*)
            if [ "$CLEANUP_SCENARIO" = "supporting" ]; then
                echo "vpc-supporting"
            fi
            ;;
        "aws ec2 describe-subnets"*)
            if [ "$CLEANUP_SCENARIO" = "supporting" ]; then
                echo "subnet-supporting"
            fi
            ;;
        "aws ec2 delete-subnet"*)
            if [ "$CLEANUP_SCENARIO" = "supporting" ]; then
                echo "An error occurred (DependencyViolation) when calling the DeleteSubnet operation: subnet-supporting is in use" >&2
                return 254
            fi
            ;;
        "aws iam list-roles"*)
            if [ "$CLEANUP_SCENARIO" = "enumeration-only" ]; then
                echo "An error occurred (Throttling) when calling the ListRoles operation: Rate exceeded" >&2
                return 254
            fi
            ;;
        "aws dynamodb list-tables"*)
            if [ "$CLEANUP_SCENARIO" = "main" ]; then
                echo "$DDB_TABLE"
            fi
            ;;
        "aws dynamodb delete-table"*)
            :
            ;;
        *)
            :
            ;;
    esac
}

sleep() {
    :
}

# shellcheck source=/dev/null
source "$FUNCTIONS_FILE"

for fn in \
    deep_cleanup_enumerate \
    _deep_cleanup_delete \
    deep_cleanup_delete_resource \
    deep_cleanup_delete_supporting \
    deep_cleanup_list_wafv2_web_acls \
    deep_cleanup_disassociate_wafv2_web_acls \
    deep_cleanup_delete_wafv2_web_acls \
    deep_cleanup_account
do
    if ! declare -F "$fn" >/dev/null; then
        fail "$fn is not defined"
    fi
done
if [ "$FAILURES" -ne 0 ]; then
    exit 1
fi

SUCCESS_STDERR_MODE="resource-list"
warning_load_balancers=""
deep_cleanup_enumerate "warning-account" "ELBv2 load balancers" warning_load_balancers \
    aws elbv2 describe-load-balancers --output text
if [ "$warning_load_balancers" != "$LOAD_BALANCER_ARN" ]; then
    fail "successful enumeration stderr contaminated the parsed resource list"
fi

SLOW_ENUMERATION_TMP_DIR="$WORK_DIR/slow-enumeration-tmp"
mkdir -p "$SLOW_ENUMERATION_TMP_DIR"
SUCCESS_STDERR_MODE="slow-enumeration"
(
    TMPDIR="$SLOW_ENUMERATION_TMP_DIR"
    slow_load_balancers=""
    deep_cleanup_enumerate "slow-account" "ELBv2 load balancers" slow_load_balancers \
        aws elbv2 describe-load-balancers --output text
    : "$slow_load_balancers"
) >"$WORK_DIR/slow-enumeration.out" 2>"$WORK_DIR/slow-enumeration.err" &
slow_enumeration_pid=$!

slow_enumeration_started=0
for ((attempt = 0; attempt < 500; attempt++)); do
    if [ -f "$WORK_DIR/slow-enumeration.started" ]; then
        slow_enumeration_started=1
        break
    fi
    command sleep 0.01
done
if [ "$slow_enumeration_started" -eq 0 ]; then
    fail "slow enumeration did not reach the stubbed AWS call"
fi
if find "$SLOW_ENUMERATION_TMP_DIR" -type f -print -quit | grep -q .; then
    fail "enumeration stderr temporary file remained linked while the AWS call was running"
fi

kill -KILL "$slow_enumeration_pid" 2>/dev/null || true
: > "$WORK_DIR/slow-enumeration.stop"
{ wait "$slow_enumeration_pid" || true; } 2>/dev/null
for ((attempt = 0; attempt < 500; attempt++)); do
    [ -f "$WORK_DIR/slow-enumeration.finished" ] && break
    command sleep 0.01
done
if [ ! -f "$WORK_DIR/slow-enumeration.finished" ]; then
    fail "stubbed AWS call did not exit after the slow enumeration was killed"
fi
if find "$SLOW_ENUMERATION_TMP_DIR" -type f -print -quit | grep -q .; then
    fail "enumeration stderr temporary file survived a kill during the AWS call"
fi
SUCCESS_STDERR_MODE="none"

SUCCESS_STDERR_MODE="wafv2-json"
(
    set -euo pipefail
    DEEP_CLEANUP_ENUMERATION_FAILURE_COUNT=0
    warning_web_acls=""
    deep_cleanup_list_wafv2_web_acls "warning-account" warning_web_acls
    [ -z "$warning_web_acls" ]
    echo "enumeration_failures=$DEEP_CLEANUP_ENUMERATION_FAILURE_COUNT"
) >"$WORK_DIR/wafv2-success-warning.out" 2>"$WORK_DIR/wafv2-success-warning.err" &
wafv2_warning_pid=$!
if wait "$wafv2_warning_pid"; then
    wafv2_warning_status=0
else
    wafv2_warning_status=$?
fi
if [ "$wafv2_warning_status" -ne 0 ]; then
    fail "WAFv2 JSON enumeration did not survive stderr from a successful AWS call"
fi
if grep -F "Failed to parse WAFv2 web ACL page" "$WORK_DIR/wafv2-success-warning.err" >/dev/null ||
    ! grep -F "enumeration_failures=0" "$WORK_DIR/wafv2-success-warning.out" >/dev/null; then
    fail "successful enumeration stderr contaminated the WAFv2 JSON page"
fi

SUCCESS_STDERR_MODE="wafv2-malformed"
(
    set -euo pipefail
    DEEP_CLEANUP_ENUMERATION_FAILURE_COUNT=0
    malformed_web_acls=""
    deep_cleanup_list_wafv2_web_acls "malformed-account" malformed_web_acls
    [ -z "$malformed_web_acls" ]
    echo "enumeration_failures=$DEEP_CLEANUP_ENUMERATION_FAILURE_COUNT"
) >"$WORK_DIR/wafv2-malformed.out" 2>"$WORK_DIR/wafv2-malformed.err" &
wafv2_malformed_pid=$!
if wait "$wafv2_malformed_pid"; then
    wafv2_malformed_status=0
else
    wafv2_malformed_status=$?
fi
if [ "$wafv2_malformed_status" -ne 0 ]; then
    fail "malformed WAFv2 JSON aborted the cleanup step"
fi
if ! grep -F "Failed to parse WAFv2 web ACL page for account malformed-account" "$WORK_DIR/wafv2-malformed.err" >/dev/null; then
    fail "malformed WAFv2 JSON was not reported with the account name"
fi
if ! grep -F "enumeration_failures=1" "$WORK_DIR/wafv2-malformed.out" >/dev/null; then
    fail "malformed WAFv2 JSON was not counted as an enumeration failure"
fi
SUCCESS_STDERR_MODE="none"

: > "$COMMAND_LOG"
(
    set -euo pipefail
    deep_cleanup_account "failure-account"
) >"$STDOUT_LOG" 2>"$STDERR_LOG" &
cleanup_pid=$!

if wait "$cleanup_pid"; then
    cleanup_status=0
else
    cleanup_status=$?
fi

if ! grep -F "Failed to delete ELBv2 load balancer $LOAD_BALANCER_ARN" "$STDERR_LOG" >/dev/null; then
    fail "delete failure did not name the stubborn load balancer"
fi
if ! grep -F "ResourceInUseException" "$STDERR_LOG" >/dev/null; then
    fail "delete failure did not include the AWS error"
fi
if ! grep -F "Failed to enumerate ELBv2 target groups for account failure-account" "$STDERR_LOG" >/dev/null; then
    fail "enumeration failure did not name the skipped step and account"
fi
if ! grep -F "AccessDeniedException" "$STDERR_LOG" >/dev/null; then
    fail "enumeration failure did not include the AWS error"
fi
if ! grep -F "aws dynamodb delete-table --table-name $DDB_TABLE" "$COMMAND_LOG" >/dev/null; then
    fail "sweep did not continue to the later DynamoDB deletion"
fi
if ! grep -F "aws ec2 describe-addresses" "$COMMAND_LOG" >/dev/null; then
    fail "sweep did not continue through the final enumeration step"
fi
if ! grep -F "failure-account: 2 found, 1 deleted, 1 failed; 1 enumeration error" "$STDERR_LOG" >/dev/null; then
    fail "summary did not split found, deleted, failed, and enumeration-error counts"
fi
if [ "$cleanup_status" -eq 0 ]; then
    fail "deep_cleanup_account returned zero despite cleanup failures"
fi

CLEANUP_SCENARIO="supporting"
if deep_cleanup_account "supporting-account" >"$WORK_DIR/supporting.out" 2>"$WORK_DIR/supporting.err"; then
    supporting_status=0
else
    supporting_status=$?
fi
if [ "$supporting_status" -ne 0 ]; then
    fail "deep_cleanup_account returned non-zero for a supporting-only cleanup failure"
fi
if ! grep -F "Failed to delete subnet subnet-supporting" "$WORK_DIR/supporting.err" >/dev/null; then
    fail "supporting cleanup failure did not name the stubborn subnet"
fi
if ! grep -F "WARNING: supporting-account: 1 found, 1 deleted, 0 failed; 1 supporting cleanup error" "$WORK_DIR/supporting.err" >/dev/null; then
    fail "summary did not report the supporting cleanup failure as a non-fatal warning"
fi

CLEANUP_SCENARIO="enumeration-only"
if deep_cleanup_account "enumeration-only-account" >"$WORK_DIR/enumeration-only.out" 2>"$WORK_DIR/enumeration-only.err"; then
    enumeration_only_status=0
else
    enumeration_only_status=$?
fi
if [ "$enumeration_only_status" -ne 0 ]; then
    fail "deep_cleanup_account returned non-zero for an enumeration-only failure"
fi
if ! grep -F "Throttling" "$WORK_DIR/enumeration-only.err" >/dev/null; then
    fail "enumeration-only failure did not include the AWS error"
fi
if ! grep -F "WARNING: enumeration-only-account: 0 found, 0 deleted, 0 failed; 1 enumeration error" "$WORK_DIR/enumeration-only.err" >/dev/null; then
    fail "summary did not report the enumeration failure as a non-fatal warning"
fi
CLEANUP_SCENARIO="main"

if [ "$FAILURES" -ne 0 ]; then
    exit 1
fi

echo "deep_cleanup_failure_visibility: OK"
