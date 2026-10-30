#!/bin/bash
# Test Scheduler deep-cleanup scoping, dependency order, and failure handling
# without making AWS calls.

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

if extract_function deep_cleanup_scheduler | grep -F "with_account_creds" >/dev/null; then
    echo "FAIL: deep_cleanup_scheduler must route AWS calls through deep-cleanup helpers" >&2
    exit 1
fi

for fn in \
    deep_cleanup_enumerate \
    _deep_cleanup_delete \
    deep_cleanup_delete_resource \
    deep_cleanup_scheduler
do
    if ! extract_function "$fn" >> "$FUNCTIONS_FILE"; then
        echo "FAIL: failed to extract $fn from run-tests.sh" >&2
        exit 1
    fi
done

SCENARIO="partial"
GOOD_GROUP="carina-acc-test-scheduler"
BROKEN_GROUP="carina-acc-test-scheduler-broken"
DEFAULT_SCHEDULE="carina-acc-test-scheduler-default"
REFRESH_SCHEDULE="carina-acc-test-refresh"
RERENDER_SCHEDULE="carina-acc-test-rerender"

with_account_creds() {
    shift
    local command="$*"
    echo "$command" >> "$COMMAND_LOG"

    case "$command" in
        *"aws scheduler list-schedules --group-name default"*)
            printf '%s\n' "$DEFAULT_SCHEDULE"
            ;;
        "aws scheduler list-schedule-groups"*)
            if [ "$SCENARIO" = "group-list-failure" ]; then
                echo "An error occurred (AccessDeniedException) when calling ListScheduleGroups" >&2
                return 254
            fi
            printf '%s\n%s\n' "$GOOD_GROUP" "$BROKEN_GROUP"
            ;;
        *"aws scheduler list-schedules --group-name $BROKEN_GROUP"*)
            echo "An error occurred (ThrottlingException) when calling ListSchedules" >&2
            return 254
            ;;
        *"aws scheduler list-schedules --group-name $GOOD_GROUP"*)
            printf '%s\n%s\n' "$REFRESH_SCHEDULE" "$RERENDER_SCHEDULE"
            ;;
        "aws scheduler delete-schedule "*|"aws scheduler delete-schedule-group "*)
            ;;
        *)
            echo "unexpected command: $command" >&2
            return 2
            ;;
    esac
}

# shellcheck source=/dev/null
source "$FUNCTIONS_FILE"

DEEP_CLEANUP_DELETED_COUNT=0
DEEP_CLEANUP_FAILED_COUNT=0
DEEP_CLEANUP_ENUMERATION_FAILURE_COUNT=0
deep_cleanup_scheduler "scheduler-test-account" > "$STDOUT_LOG" 2> "$STDERR_LOG"

refresh_delete="aws scheduler delete-schedule --group-name $GOOD_GROUP --name $REFRESH_SCHEDULE"
rerender_delete="aws scheduler delete-schedule --group-name $GOOD_GROUP --name $RERENDER_SCHEDULE"
default_delete="aws scheduler delete-schedule --group-name default --name $DEFAULT_SCHEDULE"
group_delete="aws scheduler delete-schedule-group --name $GOOD_GROUP"

for expected_command in "$default_delete" "$refresh_delete" "$rerender_delete" "$group_delete"; do
    if [ "$(grep -cxF "$expected_command" "$COMMAND_LOG" || true)" -ne 1 ]; then
        fail "expected exactly one cleanup command: $expected_command"
    fi
done

refresh_line=$(grep -nxF "$refresh_delete" "$COMMAND_LOG" | cut -d: -f1 || true)
rerender_line=$(grep -nxF "$rerender_delete" "$COMMAND_LOG" | cut -d: -f1 || true)
group_line=$(grep -nxF "$group_delete" "$COMMAND_LOG" | cut -d: -f1 || true)
if [ -z "$refresh_line" ] || [ -z "$rerender_line" ] || [ -z "$group_line" ] || \
    [ "$refresh_line" -ge "$group_line" ] || [ "$rerender_line" -ge "$group_line" ]; then
    fail "Scheduler schedules must be deleted before their group"
fi

if grep -Fx "aws scheduler delete-schedule-group --name $BROKEN_GROUP" "$COMMAND_LOG" >/dev/null; then
    fail "a group must not be deleted after its schedule enumeration fails"
fi
if ! grep -F "Skipping Scheduler group $BROKEN_GROUP" "$STDOUT_LOG" >/dev/null; then
    fail "missing fail-closed skip diagnostic for the broken group"
fi
if ! grep -F "Failed to enumerate schedules in Scheduler group $BROKEN_GROUP" "$STDERR_LOG" >/dev/null; then
    fail "missing schedule-enumeration failure diagnostic"
fi
if [ "$DEEP_CLEANUP_DELETED_COUNT" -ne 4 ]; then
    fail "expected three schedules and one group deleted, got $DEEP_CLEANUP_DELETED_COUNT"
fi
if [ "$DEEP_CLEANUP_FAILED_COUNT" -ne 0 ]; then
    fail "successful Scheduler deletions were counted as failures"
fi
if [ "$DEEP_CLEANUP_ENUMERATION_FAILURE_COUNT" -ne 1 ]; then
    fail "expected one schedule-enumeration failure, got $DEEP_CLEANUP_ENUMERATION_FAILURE_COUNT"
fi

SCENARIO="group-list-failure"
: > "$COMMAND_LOG"
: > "$STDOUT_LOG"
: > "$STDERR_LOG"
DEEP_CLEANUP_DELETED_COUNT=0
DEEP_CLEANUP_FAILED_COUNT=0
DEEP_CLEANUP_ENUMERATION_FAILURE_COUNT=0
deep_cleanup_scheduler "scheduler-test-account" > "$STDOUT_LOG" 2> "$STDERR_LOG"

if [ "$(grep -cxF "$default_delete" "$COMMAND_LOG" || true)" -ne 1 ]; then
    fail "default-group cleanup must remain independent of schedule-group enumeration"
fi
if grep -F "aws scheduler delete-schedule-group" "$COMMAND_LOG" >/dev/null; then
    fail "Scheduler cleanup must not delete a group after group enumeration fails"
fi
if ! grep -F "Skipping Scheduler cleanup because schedule groups could not be enumerated" "$STDOUT_LOG" >/dev/null; then
    fail "missing fail-closed skip diagnostic for group enumeration"
fi
if ! grep -F "Failed to enumerate acceptance-test Scheduler schedule groups" "$STDERR_LOG" >/dev/null; then
    fail "missing group-enumeration failure diagnostic"
fi
if [ "$DEEP_CLEANUP_ENUMERATION_FAILURE_COUNT" -ne 1 ]; then
    fail "expected one group-enumeration failure, got $DEEP_CLEANUP_ENUMERATION_FAILURE_COUNT"
fi
if [ "$DEEP_CLEANUP_DELETED_COUNT" -ne 1 ]; then
    fail "expected only the default-group schedule deleted, got $DEEP_CLEANUP_DELETED_COUNT"
fi

if [ "$FAILURES" -ne 0 ]; then
    exit 1
fi

echo "PASS: Scheduler deep cleanup covers default schedules and preserves group identity, dependency order, and failure visibility"
