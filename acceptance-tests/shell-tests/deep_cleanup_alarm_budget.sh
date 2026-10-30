#!/bin/bash
# Test CloudWatch Alarm and Budgets deep-cleanup scoping, endpoint selection,
# and failure handling without making AWS calls.

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
fail() {
    echo "FAIL: $*" >&2
    FAILURES=$((FAILURES + 1))
}

if ! command -v jq >/dev/null 2>&1 || ! jq --version >/dev/null 2>&1; then
    echo "FAIL: jq is required because Alarm and Budget cleanup parses AWS JSON" >&2
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

for fn in \
    deep_cleanup_enumerate \
    _deep_cleanup_delete \
    deep_cleanup_delete_resource \
    deep_cleanup_cloudwatch_alarms \
    deep_cleanup_budgets
do
    if ! extract_function "$fn" >> "$FUNCTIONS_FILE"; then
        echo "FAIL: failed to extract $fn from run-tests.sh" >&2
        exit 1
    fi
done

SCENARIO="success"
AWS_ACCOUNT_ID="123456789012"
ALARM_ONE="carina-acc-test-cloudwatch-alarm"
ALARM_TWO="carina-acc-test cloudwatch alarm stale"
ALARM_OUTSIDE_PREFIX="old alarm"
BUDGET_ONE="carina-acc-test-monthly-cost"
BUDGET_TWO="carina-acc-test-monthly-cost-stale"
BUDGET_OUTSIDE_PREFIX="old budget"

with_account_creds() {
    shift
    local command="$*"
    echo "$command" >> "$COMMAND_LOG"

    case "$command" in
        "aws cloudwatch describe-alarms "*)
            if [ "$SCENARIO" = "alarm-list-failure" ]; then
                echo "An error occurred (AccessDenied) when calling DescribeAlarms" >&2
                return 254
            fi
            if [ "$SCENARIO" = "alarm-list-null" ]; then
                # Model an omitted MetricAlarms response member. The real AWS CLI
                # evaluates the query before printing its output.
                if [[ "$command" == *"--query (MetricAlarms || \`[]\`)[].AlarmName"* ]]; then
                    printf '[]\n'
                else
                    printf 'null\n'
                fi
                return 0
            fi
            printf '["%s","%s","%s"]\n' "$ALARM_ONE" "$ALARM_TWO" "$ALARM_OUTSIDE_PREFIX"
            ;;
        "aws cloudwatch delete-alarms "*)
            if [ "$SCENARIO" = "alarm-delete-failure" ] && \
                [[ "$command" == *"--alarm-names $ALARM_ONE"* ]]; then
                echo "An error occurred (AccessDenied) when calling DeleteAlarms" >&2
                return 254
            fi
            ;;
        "aws sts get-caller-identity --query Account --output text")
            if [ "$SCENARIO" = "identity-failure" ]; then
                echo "An error occurred (ExpiredToken) when calling GetCallerIdentity" >&2
                return 254
            fi
            printf '%s\n' "$AWS_ACCOUNT_ID"
            ;;
        "aws budgets describe-budgets "*)
            if [ "$SCENARIO" = "budget-list-failure" ]; then
                echo "An error occurred (AccessDeniedException) when calling DescribeBudgets" >&2
                return 254
            fi
            if [ "$SCENARIO" = "budget-list-null" ]; then
                # Model an omitted Budgets response member. Without the fallback,
                # the query result printed by the AWS CLI is null.
                if [[ "$command" == *"--query (Budgets || \`[]\`)[?starts_with(BudgetName, \`carina-acc-test\`)].BudgetName"* ]]; then
                    printf '[]\n'
                else
                    printf 'null\n'
                fi
                return 0
            fi
            if [ "$SCENARIO" = "budget-malformed-output" ]; then
                printf '{"unexpected":true}\n'
                return 0
            fi
            printf '["%s","%s","%s"]\n' "$BUDGET_ONE" "$BUDGET_TWO" "$BUDGET_OUTSIDE_PREFIX"
            ;;
        "aws budgets delete-budget "*)
            if [ "$SCENARIO" = "budget-delete-failure" ] && \
                [[ "$command" == *"--budget-name $BUDGET_ONE --region"* ]]; then
                echo "An error occurred (AccessDeniedException) when calling DeleteBudget" >&2
                return 254
            fi
            ;;
        *)
            echo "unexpected command: $command" >&2
            return 2
            ;;
    esac
}

# shellcheck source=/dev/null
source "$FUNCTIONS_FILE"

reset_run() {
    : > "$COMMAND_LOG"
    : > "$STDOUT_LOG"
    : > "$STDERR_LOG"
    DEEP_CLEANUP_DELETED_COUNT=0
    DEEP_CLEANUP_FAILED_COUNT=0
    DEEP_CLEANUP_ENUMERATION_FAILURE_COUNT=0
}

reset_run
deep_cleanup_cloudwatch_alarms "cleanup-account" >> "$STDOUT_LOG" 2>> "$STDERR_LOG"
deep_cleanup_budgets "cleanup-account" >> "$STDOUT_LOG" 2>> "$STDERR_LOG"

for expected_command in \
    "aws cloudwatch delete-alarms --alarm-names $ALARM_ONE" \
    "aws cloudwatch delete-alarms --alarm-names $ALARM_TWO" \
    "aws budgets delete-budget --account-id $AWS_ACCOUNT_ID --budget-name $BUDGET_ONE --region us-east-1" \
    "aws budgets delete-budget --account-id $AWS_ACCOUNT_ID --budget-name $BUDGET_TWO --region us-east-1"
do
    if [ "$(grep -cxF "$expected_command" "$COMMAND_LOG" || true)" -ne 1 ]; then
        fail "expected exactly one cleanup command: $expected_command"
    fi
done

alarm_list_command=$(grep -F "aws cloudwatch describe-alarms " "$COMMAND_LOG" || true)
if [[ "$alarm_list_command" != *"--alarm-name-prefix carina-acc-test"* ]]; then
    fail "CloudWatch enumeration must use the service-side carina-acc-test prefix"
fi
if [[ "$alarm_list_command" != *"--query (MetricAlarms || \`[]\`)[].AlarmName"* ]]; then
    fail "CloudWatch enumeration must default an omitted MetricAlarms list before selecting names"
fi
if [[ "$alarm_list_command" != *"--output json"* ]]; then
    fail "CloudWatch enumeration must preserve alarm names as JSON"
fi

budget_list_command=$(grep -F "aws budgets describe-budgets " "$COMMAND_LOG" || true)
if [[ "$budget_list_command" != *"--account-id $AWS_ACCOUNT_ID"* ]] || \
    [[ "$budget_list_command" != *"--region us-east-1"* ]] || \
    [[ "$budget_list_command" != *"--query (Budgets || \`[]\`)[?starts_with(BudgetName, \`carina-acc-test\`)].BudgetName"* ]] || \
    [[ "$budget_list_command" != *"--output json"* ]]; then
    fail "Budgets enumeration must default an omitted list and be scoped by account, prefix, and us-east-1"
fi

if grep -F "aws cloudwatch delete-alarms --alarm-names old" "$COMMAND_LOG" >/dev/null; then
    fail "a space-containing alarm name must not word-split into an out-of-prefix delete"
fi
if grep -F "aws cloudwatch delete-alarms --alarm-names $ALARM_OUTSIDE_PREFIX" "$COMMAND_LOG" >/dev/null; then
    fail "CloudWatch cleanup must re-check the prefix on every returned name"
fi
if grep -F "aws budgets delete-budget --account-id $AWS_ACCOUNT_ID --budget-name $BUDGET_OUTSIDE_PREFIX" "$COMMAND_LOG" >/dev/null; then
    fail "Budgets cleanup must re-check the prefix on every returned name"
fi

if [ "$DEEP_CLEANUP_DELETED_COUNT" -ne 4 ]; then
    fail "expected two alarms and two budgets deleted, got $DEEP_CLEANUP_DELETED_COUNT"
fi
if [ "$DEEP_CLEANUP_FAILED_COUNT" -ne 0 ] || \
    [ "$DEEP_CLEANUP_ENUMERATION_FAILURE_COUNT" -ne 0 ]; then
    fail "successful cleanup recorded failures"
fi

SCENARIO="alarm-list-null"
reset_run
deep_cleanup_cloudwatch_alarms "cleanup-account" >> "$STDOUT_LOG" 2>> "$STDERR_LOG"

if grep -F "aws cloudwatch delete-alarms " "$COMMAND_LOG" >/dev/null; then
    fail "an omitted MetricAlarms list must not produce alarm deletions"
fi
if [ "$DEEP_CLEANUP_LAST_ENUMERATION_SUCCEEDED" -ne 1 ] || \
    [ "$DEEP_CLEANUP_ENUMERATION_FAILURE_COUNT" -ne 0 ]; then
    fail "an omitted MetricAlarms list must be a successful empty enumeration"
fi

SCENARIO="budget-list-null"
reset_run
deep_cleanup_budgets "cleanup-account" >> "$STDOUT_LOG" 2>> "$STDERR_LOG"

if grep -F "aws budgets delete-budget " "$COMMAND_LOG" >/dev/null; then
    fail "an omitted Budgets list must not produce budget deletions"
fi
if [ "$DEEP_CLEANUP_LAST_ENUMERATION_SUCCEEDED" -ne 1 ] || \
    [ "$DEEP_CLEANUP_ENUMERATION_FAILURE_COUNT" -ne 0 ]; then
    fail "an omitted Budgets list must be a successful empty enumeration"
fi

SCENARIO="budget-malformed-output"
reset_run
deep_cleanup_budgets "cleanup-account" >> "$STDOUT_LOG" 2>> "$STDERR_LOG"

if grep -F "aws budgets delete-budget " "$COMMAND_LOG" >/dev/null; then
    fail "malformed Budget output must fail closed without deletions"
fi
if [ "$DEEP_CLEANUP_LAST_ENUMERATION_SUCCEEDED" -ne 0 ] || \
    [ "$DEEP_CLEANUP_ENUMERATION_FAILURE_COUNT" -ne 1 ]; then
    fail "malformed Budget output must record one enumeration failure"
fi

SCENARIO="alarm-list-failure"
reset_run
deep_cleanup_cloudwatch_alarms "cleanup-account" >> "$STDOUT_LOG" 2>> "$STDERR_LOG"

if grep -F "aws cloudwatch delete-alarms " "$COMMAND_LOG" >/dev/null; then
    fail "CloudWatch cleanup must fail closed after alarm enumeration fails"
fi
if ! grep -F "Skipping CloudWatch alarm cleanup because alarms could not be enumerated" "$STDOUT_LOG" >/dev/null; then
    fail "missing fail-closed diagnostic for Alarm enumeration"
fi
if ! grep -F "Failed to enumerate acceptance-test CloudWatch alarms" "$STDERR_LOG" >/dev/null; then
    fail "missing Alarm enumeration failure diagnostic"
fi
if [ "$DEEP_CLEANUP_ENUMERATION_FAILURE_COUNT" -ne 1 ]; then
    fail "expected one Alarm enumeration failure"
fi

SCENARIO="budget-list-failure"
reset_run
deep_cleanup_budgets "cleanup-account" >> "$STDOUT_LOG" 2>> "$STDERR_LOG"

if grep -F "aws budgets delete-budget " "$COMMAND_LOG" >/dev/null; then
    fail "Budgets cleanup must fail closed after budget enumeration fails"
fi
if ! grep -F "Skipping Budgets cleanup because budgets could not be enumerated" "$STDOUT_LOG" >/dev/null; then
    fail "missing fail-closed diagnostic for Budget enumeration"
fi
if ! grep -F "Failed to enumerate acceptance-test Budgets" "$STDERR_LOG" >/dev/null; then
    fail "missing Budget enumeration failure diagnostic"
fi
if [ "$DEEP_CLEANUP_ENUMERATION_FAILURE_COUNT" -ne 1 ]; then
    fail "expected one Budget enumeration failure"
fi

SCENARIO="alarm-delete-failure"
reset_run
deep_cleanup_cloudwatch_alarms "cleanup-account" >> "$STDOUT_LOG" 2>> "$STDERR_LOG"

if [ "$DEEP_CLEANUP_FAILED_COUNT" -ne 1 ] || [ "$DEEP_CLEANUP_DELETED_COUNT" -ne 1 ]; then
    fail "Alarm delete failure must increment failed count and continue cleanup"
fi
if ! grep -F "Failed to delete CloudWatch alarm $ALARM_ONE" "$STDERR_LOG" >/dev/null; then
    fail "missing Alarm delete failure diagnostic"
fi

SCENARIO="budget-delete-failure"
reset_run
deep_cleanup_budgets "cleanup-account" >> "$STDOUT_LOG" 2>> "$STDERR_LOG"

if [ "$DEEP_CLEANUP_FAILED_COUNT" -ne 1 ] || [ "$DEEP_CLEANUP_DELETED_COUNT" -ne 1 ]; then
    fail "Budget delete failure must increment failed count and continue cleanup"
fi
if ! grep -F "Failed to delete Budget $BUDGET_ONE" "$STDERR_LOG" >/dev/null; then
    fail "missing Budget delete failure diagnostic"
fi

SCENARIO="identity-failure"
reset_run
deep_cleanup_budgets "cleanup-account" >> "$STDOUT_LOG" 2>> "$STDERR_LOG"

if grep -F "aws budgets describe-budgets " "$COMMAND_LOG" >/dev/null || \
    grep -F "aws budgets delete-budget " "$COMMAND_LOG" >/dev/null; then
    fail "Budgets cleanup must stop when account identity cannot be resolved"
fi
if ! grep -F "Skipping Budgets cleanup because the AWS account ID could not be resolved" "$STDOUT_LOG" >/dev/null; then
    fail "missing account-identity fail-closed diagnostic"
fi

if [ "$FAILURES" -ne 0 ]; then
    exit 1
fi

echo "PASS: Alarm and Budget deep cleanup is prefix-scoped, region-correct, and fail-closed"
