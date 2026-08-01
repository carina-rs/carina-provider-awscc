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
    deep_cleanup_with_timeout \
    deep_cleanup_wait_for_network_interface \
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
    deep_cleanup_with_timeout \
    deep_cleanup_wait_for_network_interface \
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
VANISHED_ENI_ID="eni-vanished"
REMAINING_ENI_ID="eni-remaining"
TIMEOUT_ENI_ID="eni-still-attached"
SUCCESS_STDERR_WARNING="urllib3 v2 only supports OpenSSL"
SUCCESS_STDERR_MODE="none"
CLEANUP_SCENARIO="main"

aws() {
    local command="$*"
    case "$CLEANUP_SCENARIO" in
        eni-not-found)
            if [[ "$command" == *"$VANISHED_ENI_ID"* ]]; then
                echo "Waiter NetworkInterfaceAvailable failed: An error occurred (InvalidNetworkInterfaceID.NotFound): The networkInterface ID '$VANISHED_ENI_ID' does not exist" >&2
                return 255
            fi
            ;;
        eni-timeout)
            echo "command timed out after 300 seconds" >&2
            return 137
            ;;
    esac

    return 0
}

with_account_creds() {
    shift
    local command="$*"
    echo "$command" >> "$COMMAND_LOG"

    # Match production's inner credential subshell and its fd inheritance.
    (
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
                if [ "$SUCCESS_STDERR_MODE" = "resource-list" ] || \
                    [ "$CLEANUP_SCENARIO" = "main" ] || \
                    [ "$CLEANUP_SCENARIO" = "waiter-failure" ]; then
                    echo "$LOAD_BALANCER_ARN"
                fi
                ;;
            "aws elbv2 delete-load-balancer"*)
                if [ "$CLEANUP_SCENARIO" = "main" ]; then
                    echo "An error occurred (ResourceInUseException) when calling the DeleteLoadBalancer operation: $LOAD_BALANCER_ARN is still in use" >&2
                    return 254
                fi
                ;;
            *"aws elbv2 wait load-balancers-deleted"*)
                if [ "$CLEANUP_SCENARIO" = "waiter-failure" ]; then
                    echo "Waiter LoadBalancersDeleted failed: Max attempts exceeded" >&2
                    return 255
                fi
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
                elif [ "$CLEANUP_SCENARIO" = "eni-not-found" ] || \
                    [ "$CLEANUP_SCENARIO" = "eni-timeout" ]; then
                    echo "vpc-eni-waiter"
                fi
                ;;
            "aws ec2 describe-network-interfaces"*)
                if [[ "$command" == *"Name=status,Values=in-use"* ]]; then
                    if [ "$CLEANUP_SCENARIO" = "eni-not-found" ]; then
                        printf '%s\t%s\n' "$VANISHED_ENI_ID" "$REMAINING_ENI_ID"
                    elif [ "$CLEANUP_SCENARIO" = "eni-timeout" ]; then
                        echo "$TIMEOUT_ENI_ID"
                    fi
                fi
                ;;
            *"aws ec2 wait network-interface-available"*)
                if [ "${1:-}" = "deep_cleanup_wait_for_network_interface" ]; then
                    "$@"
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
    )
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
    deep_cleanup_with_timeout \
    deep_cleanup_wait_for_network_interface \
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

FAST_SUCCESS_TIMEOUT_SECONDS=23
FAST_SUCCESS_MAX_ELAPSED_SECONDS=5
FAST_SUCCESS_ENI_ID="eni-fast-success"
FAST_SUCCESS_BIN_DIR="$WORK_DIR/fast-success-bin"
FAST_SUCCESS_TIMER_PID_FILE="$WORK_DIR/fast-success-timer.pid"
REAL_SLEEP_COMMAND="$(type -P sleep)"
mkdir -p "$FAST_SUCCESS_BIN_DIR"
# The generated shim expands these variables when the timer process starts.
# shellcheck disable=SC2016
printf '%s\n' \
    '#!/bin/sh' \
    'if [ "${1:-}" = "$FAST_SUCCESS_TIMEOUT_SECONDS" ]; then' \
    '    printf "%s\n" "$$" > "$FAST_SUCCESS_TIMER_PID_FILE"' \
    'fi' \
    'exec "$REAL_SLEEP_COMMAND" "$@"' \
    > "$FAST_SUCCESS_BIN_DIR/sleep"
chmod +x "$FAST_SUCCESS_BIN_DIR/sleep"
export FAST_SUCCESS_TIMEOUT_SECONDS FAST_SUCCESS_TIMER_PID_FILE REAL_SLEEP_COMMAND

# Exercise the production call shape: _deep_cleanup_delete captures a
# with_account_creds subshell that invokes a waiter returning immediately.
DEEP_CLEANUP_SUPPORTING_FAILURE_COUNT=0
fast_success_started=$SECONDS
PATH="$FAST_SUCCESS_BIN_DIR:$PATH" \
    deep_cleanup_delete_supporting "fast-success-account" "wait for fast-success network interface" \
    deep_cleanup_wait_for_network_interface "$FAST_SUCCESS_TIMEOUT_SECONDS" \
    aws ec2 wait network-interface-available \
    --network-interface-ids "$FAST_SUCCESS_ENI_ID"
fast_success_elapsed=$((SECONDS - fast_success_started))
if [ "$fast_success_elapsed" -gt "$FAST_SUCCESS_MAX_ELAPSED_SECONDS" ]; then
    fail "fast-success waiter took ${fast_success_elapsed}s with a ${FAST_SUCCESS_TIMEOUT_SECONDS}s timeout"
fi
if [ "$DEEP_CLEANUP_SUPPORTING_FAILURE_COUNT" -ne 0 ]; then
    fail "fast-success waiter was counted as a supporting cleanup failure"
fi

fast_success_timer_pid=""
if ! IFS= read -r fast_success_timer_pid < "$FAST_SUCCESS_TIMER_PID_FILE" || [ -z "$fast_success_timer_pid" ]; then
    fail "fast-success waiter did not expose its timer process for the orphan check"
elif kill -0 "$fast_success_timer_pid" 2>/dev/null; then
    fail "fast-success waiter left timer process $fast_success_timer_pid running"
    kill "$fast_success_timer_pid" 2>/dev/null || true
fi

if deep_cleanup_with_timeout 1 command sleep 30 >"$WORK_DIR/timeout.out" 2>"$WORK_DIR/timeout.err"; then
    fail "deep_cleanup_with_timeout returned zero after its wall-clock limit"
fi
if ! grep -F "command timed out after 1 seconds" "$WORK_DIR/timeout.err" >/dev/null; then
    fail "deep_cleanup_with_timeout did not report its wall-clock limit"
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
if grep -F "aws elbv2 wait load-balancers-deleted" "$COMMAND_LOG" >/dev/null; then
    fail "load-balancer waiter ran even though the load-balancer delete failed"
fi
if ! grep -F "failure-account: 2 found, 1 deleted, 1 failed; 1 enumeration error" "$STDERR_LOG" >/dev/null; then
    fail "summary did not split found, deleted, failed, and enumeration-error counts"
fi
if [ "$cleanup_status" -eq 0 ]; then
    fail "deep_cleanup_account returned zero despite cleanup failures"
fi

CLEANUP_SCENARIO="waiter-failure"
: > "$COMMAND_LOG"
if deep_cleanup_account "waiter-failure-account" >"$WORK_DIR/waiter-failure.out" 2>"$WORK_DIR/waiter-failure.err"; then
    waiter_failure_status=0
else
    waiter_failure_status=$?
fi
if [ "$waiter_failure_status" -ne 0 ]; then
    fail "deep_cleanup_account returned non-zero for a load-balancer waiter failure"
fi
if ! grep -F "Failed to wait for deleted ELBv2 load balancers" "$WORK_DIR/waiter-failure.err" >/dev/null; then
    fail "load-balancer waiter failure did not name the failed wait operation"
fi
if ! grep -F "Max attempts exceeded" "$WORK_DIR/waiter-failure.err" >/dev/null; then
    fail "load-balancer waiter failure did not include the AWS error"
fi
if ! grep -F "aws ec2 describe-addresses" "$COMMAND_LOG" >/dev/null; then
    fail "sweep did not continue after the load-balancer waiter failure"
fi
if ! grep -F "WARNING: waiter-failure-account: 1 found, 1 deleted, 0 failed; 1 supporting cleanup error" "$WORK_DIR/waiter-failure.err" >/dev/null; then
    fail "summary did not report the load-balancer waiter failure as a supporting cleanup error"
fi

CLEANUP_SCENARIO="eni-not-found"
: > "$COMMAND_LOG"
if deep_cleanup_account "eni-not-found-account" >"$WORK_DIR/eni-not-found.out" 2>"$WORK_DIR/eni-not-found.err"; then
    eni_not_found_status=0
else
    eni_not_found_status=$?
fi
if [ "$eni_not_found_status" -ne 0 ]; then
    fail "deep_cleanup_account returned non-zero when an AWS-released ENI vanished"
fi
if grep -F "supporting cleanup error" "$WORK_DIR/eni-not-found.err" >/dev/null; then
    fail "a vanished ENI was counted as a supporting cleanup failure"
fi
if grep -F "InvalidNetworkInterfaceID.NotFound" "$WORK_DIR/eni-not-found.err" >/dev/null; then
    fail "a vanished ENI waiter error escaped the not-found success handling"
fi
vanished_eni_wait_count=$(grep -cF "aws ec2 wait network-interface-available --network-interface-ids $VANISHED_ENI_ID" "$COMMAND_LOG" || true)
if [ "$vanished_eni_wait_count" -ne 1 ]; then
    fail "expected one isolated wait for vanished ENI $VANISHED_ENI_ID, got $vanished_eni_wait_count"
fi
remaining_eni_wait_count=$(grep -cF "aws ec2 wait network-interface-available --network-interface-ids $REMAINING_ENI_ID" "$COMMAND_LOG" || true)
if [ "$remaining_eni_wait_count" -ne 1 ]; then
    fail "vanished ENI prevented the isolated wait for remaining ENI $REMAINING_ENI_ID"
fi

CLEANUP_SCENARIO="eni-timeout"
: > "$COMMAND_LOG"
if deep_cleanup_account "eni-timeout-account" >"$WORK_DIR/eni-timeout.out" 2>"$WORK_DIR/eni-timeout.err"; then
    eni_timeout_status=0
else
    eni_timeout_status=$?
fi
if [ "$eni_timeout_status" -ne 0 ]; then
    fail "deep_cleanup_account returned non-zero for an ENI waiter timeout"
fi
if ! grep -F "Failed to wait for network interface $TIMEOUT_ENI_ID to become available" "$WORK_DIR/eni-timeout.err" >/dev/null; then
    fail "ENI waiter timeout did not name the still-attached interface"
fi
if ! grep -F "command timed out after 300 seconds" "$WORK_DIR/eni-timeout.err" >/dev/null; then
    fail "ENI waiter timeout did not retain the wall-clock timeout diagnostic"
fi
if ! grep -F "WARNING: eni-timeout-account: 1 found, 1 deleted, 0 failed; 1 supporting cleanup error" "$WORK_DIR/eni-timeout.err" >/dev/null; then
    fail "ENI waiter timeout was not counted as a supporting cleanup error"
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
