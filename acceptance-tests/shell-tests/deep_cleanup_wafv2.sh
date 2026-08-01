#!/bin/bash
# Test deep-cleanup WAFv2 WebACL disassociation without making AWS calls.
#
# Usage:
#   bash acceptance-tests/shell-tests/deep_cleanup_wafv2.sh
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
STDERR_LOG="$WORK_DIR/stderr.log"

cleanup() {
    rm -rf "$WORK_DIR"
}
trap cleanup EXIT

fail() {
    echo "FAIL: $*" >&2
    exit 1
}

if ! command -v jq >/dev/null 2>&1 || ! jq --version >/dev/null 2>&1; then
    fail "jq is required because the extracted cleanup helpers parse AWS JSON with jq"
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

for fn in \
    deep_cleanup_list_wafv2_web_acls \
    deep_cleanup_disassociate_wafv2_web_acls \
    deep_cleanup_delete_wafv2_web_acls \
    deep_cleanup_account
do
    if ! extract_function "$fn" >> "$FUNCTIONS_FILE"; then
        fail "failed to extract $fn from run-tests.sh"
    fi
done

WEB_ACL_ONE_NAME="acceptance-test-web-acl"
WEB_ACL_ONE_ID="web-acl-id"
WEB_ACL_ONE_LOCK_TOKEN="lock-token"
WEB_ACL_ONE_ARN="arn:aws:wafv2:us-east-1:123456789012:regional/webacl/acceptance-test-web-acl/web-acl-id"
WEB_ACL_TWO_NAME="carina-acc-page-two-web-acl"
WEB_ACL_TWO_ID="web-acl-id-page-two"
WEB_ACL_TWO_LOCK_TOKEN="lock-token-page-two"
WEB_ACL_TWO_ARN="arn:aws:wafv2:us-east-1:123456789012:regional/webacl/carina-acc-page-two-web-acl/web-acl-id-page-two"
WEB_ACL_THREE_NAME="acceptance-test-empty-lock-web-acl"
WEB_ACL_THREE_ID="web-acl-id-empty-lock"
WEB_ACL_THREE_ARN="arn:aws:wafv2:us-east-1:123456789012:regional/webacl/acceptance-test-empty-lock-web-acl/web-acl-id-empty-lock"
PAGE_TWO_MARKER="page-two-marker"
STUCK_PAGE_MARKER="stuck-page-marker"
WAFV2_LIST_MODE="normal"

web_acl_name_for_arn() {
    local web_acl_arn="$1"
    case "$web_acl_arn" in
        "$WEB_ACL_ONE_ARN")
            echo "$WEB_ACL_ONE_NAME"
            ;;
        "$WEB_ACL_TWO_ARN")
            echo "$WEB_ACL_TWO_NAME"
            ;;
        "$WEB_ACL_THREE_ARN")
            echo "$WEB_ACL_THREE_NAME"
            ;;
        *)
            return 1
            ;;
    esac
}

web_acl_page_json() {
    local next_marker="$1"
    if [ "$WAFV2_LIST_MODE" = "constant-marker" ]; then
        printf '{"NextMarker":"%s","WebACLs":[]}\n' "$STUCK_PAGE_MARKER"
        return
    fi

    if [ "$next_marker" = "$PAGE_TWO_MARKER" ]; then
        printf '{"WebACLs":[{"Name":"%s","Id":"%s","LockToken":"%s","ARN":"%s"},{"Name":"%s","Id":"%s","LockToken":null,"ARN":"%s"}]}\n' \
            "$WEB_ACL_TWO_NAME" \
            "$WEB_ACL_TWO_ID" \
            "$WEB_ACL_TWO_LOCK_TOKEN" \
            "$WEB_ACL_TWO_ARN" \
            "$WEB_ACL_THREE_NAME" \
            "$WEB_ACL_THREE_ID" \
            "$WEB_ACL_THREE_ARN"
    else
        printf '{"NextMarker":"%s","WebACLs":[{"Name":"%s","Id":"%s","LockToken":"%s","ARN":"%s"}]}\n' \
            "$PAGE_TWO_MARKER" \
            "$WEB_ACL_ONE_NAME" \
            "$WEB_ACL_ONE_ID" \
            "$WEB_ACL_ONE_LOCK_TOKEN" \
            "$WEB_ACL_ONE_ARN"
    fi
}

resource_arns_for_type() {
    local web_acl_arn="$1"
    local resource_type="$2"
    local acl_name
    acl_name="$(web_acl_name_for_arn "$web_acl_arn")"
    printf 'arn:aws:wafv2-test:us-east-1:123456789012:resource/%s/%s/one\tarn:aws:wafv2-test:us-east-1:123456789012:resource/%s/%s/two\n' \
        "$acl_name" \
        "$resource_type" \
        "$acl_name" \
        "$resource_type"
}

arg_after() {
    local wanted="$1"
    shift
    while [ "$#" -gt 0 ]; do
        if [ "$1" = "$wanted" ]; then
            echo "$2"
            return 0
        fi
        shift
    done
    return 1
}

with_account_creds() {
    shift
    local command="$*"
    echo "$command" >> "$COMMAND_LOG"

    case "$command" in
        "aws wafv2 list-web-acls"*)
            local next_marker
            next_marker="$(arg_after "--next-marker" "$@" || true)"
            web_acl_page_json "$next_marker"
            ;;
        "aws wafv2 list-resources-for-web-acl"*)
            local resource_type
            local web_acl_arn
            web_acl_arn="$(arg_after "--web-acl-arn" "$@")"
            resource_type="$(arg_after "--resource-type" "$@")"
            resource_arns_for_type "$web_acl_arn" "$resource_type"
            ;;
        "aws wafv2 delete-web-acl"*)
            local acl_name
            acl_name="$(arg_after "--name" "$@")"
            echo "An error occurred (WAFAssociatedItemException) when calling the DeleteWebACL operation: $acl_name is still associated" >&2
            return 254
            ;;
        "aws elbv2 describe-load-balancers"*)
            echo "arn:aws:elasticloadbalancing:us-east-1:123456789012:loadbalancer/app/acceptance-test-alb/111"
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

declare -F deep_cleanup_list_wafv2_web_acls >/dev/null ||
    fail "deep_cleanup_list_wafv2_web_acls is not defined"
declare -F deep_cleanup_disassociate_wafv2_web_acls >/dev/null ||
    fail "deep_cleanup_disassociate_wafv2_web_acls is not defined"
declare -F deep_cleanup_delete_wafv2_web_acls >/dev/null ||
    fail "deep_cleanup_delete_wafv2_web_acls is not defined"
declare -F deep_cleanup_account >/dev/null ||
    fail "deep_cleanup_account is not defined"

WAFV2_RESOURCE_TYPES="
APPLICATION_LOAD_BALANCER
API_GATEWAY
APPSYNC
COGNITO_USER_POOL
APP_RUNNER_SERVICE
VERIFIED_ACCESS_INSTANCE
AMPLIFY
"

WEB_ACL_ARNS="
$WEB_ACL_ONE_ARN
$WEB_ACL_TWO_ARN
$WEB_ACL_THREE_ARN
"

: > "$COMMAND_LOG"
deep_cleanup_disassociate_wafv2_web_acls "test-account" >/dev/null

deep_cleanup_delete_wafv2_web_acls "test-account" >/dev/null 2>"$STDERR_LOG"

empty_lock_failures=""
empty_lock_list_count=$(grep -cF "aws wafv2 list-resources-for-web-acl --web-acl-arn $WEB_ACL_THREE_ARN" "$COMMAND_LOG" || true)
if [ "$empty_lock_list_count" -ne 7 ]; then
    empty_lock_failures="expected 7 association resource types for $WEB_ACL_THREE_NAME, got $empty_lock_list_count"
fi
empty_lock_delete=$(grep -F "aws wafv2 delete-web-acl --name $WEB_ACL_THREE_NAME" "$COMMAND_LOG" || true)
if [ -n "$empty_lock_delete" ]; then
    empty_lock_failures="${empty_lock_failures:+$empty_lock_failures; }delete-web-acl used the ARN as --lock-token for $WEB_ACL_THREE_NAME"
fi
if ! grep -F "$WEB_ACL_THREE_NAME" "$STDERR_LOG" >/dev/null ||
    ! grep -F "LockToken" "$STDERR_LOG" >/dev/null; then
    empty_lock_failures="${empty_lock_failures:+$empty_lock_failures; }missing LockToken was not reported for $WEB_ACL_THREE_NAME"
fi
[ -z "$empty_lock_failures" ] || fail "$empty_lock_failures"

for web_acl_arn in $WEB_ACL_ARNS; do
    acl_name="$(web_acl_name_for_arn "$web_acl_arn")"
    for resource_type in $WAFV2_RESOURCE_TYPES; do
        count=$(grep -cF "aws wafv2 list-resources-for-web-acl --web-acl-arn $web_acl_arn --resource-type $resource_type" "$COMMAND_LOG" || true)
        [ "$count" -eq 1 ] ||
            fail "expected one list-resources-for-web-acl call for $acl_name $resource_type, got $count"

        for resource_arn in $(resource_arns_for_type "$web_acl_arn" "$resource_type"); do
            grep -F "aws wafv2 disassociate-web-acl --resource-arn $resource_arn" "$COMMAND_LOG" >/dev/null ||
                fail "missing disassociate-web-acl call for $acl_name $resource_arn"
        done
    done
done

: > "$COMMAND_LOG"
if ! deep_cleanup_account "test-account" >/dev/null 2>"$STDERR_LOG"; then
    fail "deep_cleanup_account returned non-zero"
fi

first_lb_delete=$(grep -nF "aws elbv2 delete-load-balancer --load-balancer-arn" "$COMMAND_LOG" | head -1 | cut -d: -f1 || true)
[ -n "$first_lb_delete" ] || fail "missing elbv2 delete-load-balancer call"

for web_acl_arn in $WEB_ACL_ARNS; do
    acl_name="$(web_acl_name_for_arn "$web_acl_arn")"
    for resource_type in $WAFV2_RESOURCE_TYPES; do
        for resource_arn in $(resource_arns_for_type "$web_acl_arn" "$resource_type"); do
            disassociate_line=$(grep -nF "aws wafv2 disassociate-web-acl --resource-arn $resource_arn" "$COMMAND_LOG" | head -1 | cut -d: -f1 || true)
            [ -n "$disassociate_line" ] ||
                fail "deep_cleanup_account did not disassociate $acl_name $resource_arn"
            [ "$disassociate_line" -lt "$first_lb_delete" ] ||
                fail "disassociate for $acl_name $resource_arn happened after elbv2 delete-load-balancer"
        done
    done
done

grep -F "aws wafv2 delete-web-acl --name $WEB_ACL_ONE_NAME --scope REGIONAL --id $WEB_ACL_ONE_ID --lock-token $WEB_ACL_ONE_LOCK_TOKEN" "$COMMAND_LOG" >/dev/null ||
    fail "missing delete-web-acl attempt for $WEB_ACL_ONE_NAME"
grep -F "aws wafv2 delete-web-acl --name $WEB_ACL_TWO_NAME --scope REGIONAL --id $WEB_ACL_TWO_ID --lock-token $WEB_ACL_TWO_LOCK_TOKEN" "$COMMAND_LOG" >/dev/null ||
    fail "missing delete-web-acl attempt for $WEB_ACL_TWO_NAME"

grep -F "$WEB_ACL_ONE_NAME" "$STDERR_LOG" >/dev/null ||
    fail "delete-web-acl failure did not name $WEB_ACL_ONE_NAME"
grep -F "$WEB_ACL_TWO_NAME" "$STDERR_LOG" >/dev/null ||
    fail "delete-web-acl failure did not name $WEB_ACL_TWO_NAME"
grep -F "WAFAssociatedItemException" "$STDERR_LOG" >/dev/null ||
    fail "delete-web-acl failure did not include the AWS error"

: > "$COMMAND_LOG"
WAFV2_LIST_MODE="constant-marker"
deep_cleanup_list_wafv2_web_acls "stuck-marker-account" >"$WORK_DIR/stuck-pagination.out" 2>"$WORK_DIR/stuck-pagination.err" &
pagination_pid=$!
pagination_checks=0
while kill -0 "$pagination_pid" 2>/dev/null && [ "$pagination_checks" -lt 40 ]; do
    pagination_checks=$((pagination_checks + 1))
    command sleep 0.05
done
if kill -0 "$pagination_pid" 2>/dev/null; then
    kill "$pagination_pid" 2>/dev/null || true
    wait "$pagination_pid" 2>/dev/null || true
    WAFV2_LIST_MODE="normal"
    fail "WAFv2 list-web-acls pagination did not terminate for repeated NextMarker"
fi
if ! wait "$pagination_pid"; then
    WAFV2_LIST_MODE="normal"
    fail "WAFv2 list-web-acls pagination returned non-zero for repeated NextMarker"
fi
WAFV2_LIST_MODE="normal"
grep -F "stuck-marker-account" "$WORK_DIR/stuck-pagination.err" >/dev/null ||
    fail "repeated NextMarker diagnostic did not name the account"
grep -F "pagination truncated" "$WORK_DIR/stuck-pagination.err" >/dev/null ||
    fail "repeated NextMarker did not report truncated pagination"

echo "deep_cleanup_wafv2: OK"
