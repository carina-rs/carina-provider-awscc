#!/bin/bash
# Test SNS deep-cleanup scoping and failure handling without making AWS calls.

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
    echo "FAIL: jq is required because SNS cleanup parses AWS JSON" >&2
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
    deep_cleanup_sns
do
    if ! extract_function "$fn" >> "$FUNCTIONS_FILE"; then
        echo "FAIL: failed to extract $fn from run-tests.sh" >&2
        exit 1
    fi
done

TOPIC_ONE="arn:aws:sns:ap-northeast-1:123456789012:carina-acc-test-sns-topic"
TOPIC_OUTSIDE_PREFIX="arn:aws:sns:ap-northeast-1:123456789012:shared-alerts"
SCENARIO="success"

with_account_creds() {
    shift
    local command="$*"
    echo "$command" >> "$COMMAND_LOG"

    case "$command" in
        "aws sns list-topics "*)
            case "$SCENARIO" in
                topic-list-failure)
                    echo "An error occurred (AccessDenied) when calling ListTopics" >&2
                    return 254
                    ;;
                topic-list-empty)
                    printf '[]\n'
                    ;;
                topic-malformed-output)
                    printf '{"unexpected":true}\n'
                    ;;
                *)
                    printf '["%s","%s"]\n' "$TOPIC_ONE" "$TOPIC_OUTSIDE_PREFIX"
                    ;;
            esac
            ;;
        "aws sns delete-topic "*)
            if [ "$SCENARIO" = "topic-delete-failure" ]; then
                echo "An error occurred (AccessDenied) when calling DeleteTopic" >&2
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

if grep -E 'list-subscriptions-by-topic|unsubscribe' "$FUNCTIONS_FILE" >/dev/null; then
    fail "SNS cleanup must not enumerate or unsubscribe individual subscriptions"
fi

reset_run() {
    : > "$COMMAND_LOG"
    : > "$STDOUT_LOG"
    : > "$STDERR_LOG"
    DEEP_CLEANUP_DELETED_COUNT=0
    DEEP_CLEANUP_FAILED_COUNT=0
    DEEP_CLEANUP_ENUMERATION_FAILURE_COUNT=0
}

reset_run
deep_cleanup_sns "cleanup-account" >> "$STDOUT_LOG" 2>> "$STDERR_LOG"

if [ "$(grep -cxF "aws sns delete-topic --topic-arn $TOPIC_ONE" "$COMMAND_LOG" || true)" -ne 1 ]; then
    fail "expected exactly one scoped SNS topic deletion"
fi
topic_list_command=$(grep -F "aws sns list-topics " "$COMMAND_LOG" || true)
if [[ "$topic_list_command" != *"--query (Topics || \`[]\`)[?contains(TopicArn, \`:carina-acc-test\`)].TopicArn"* ]] || \
    [[ "$topic_list_command" != *"--output json"* ]]; then
    fail "SNS topic enumeration must default an omitted list and narrow by acceptance prefix"
fi
if grep -F "aws sns delete-topic --topic-arn $TOPIC_OUTSIDE_PREFIX" "$COMMAND_LOG" >/dev/null; then
    fail "SNS cleanup must not delete topics outside the acceptance prefix"
fi
if [ "$DEEP_CLEANUP_DELETED_COUNT" -ne 1 ] || [ "$DEEP_CLEANUP_FAILED_COUNT" -ne 0 ] || \
    [ "$DEEP_CLEANUP_ENUMERATION_FAILURE_COUNT" -ne 0 ]; then
    fail "successful SNS cleanup must record one topic deletion"
fi

SCENARIO="topic-list-empty"
reset_run
deep_cleanup_sns "cleanup-account" >> "$STDOUT_LOG" 2>> "$STDERR_LOG"
if grep -F "aws sns delete-topic " "$COMMAND_LOG" >/dev/null; then
    fail "an empty Topics list must not produce SNS deletions"
fi
if [ "$DEEP_CLEANUP_LAST_ENUMERATION_SUCCEEDED" -ne 1 ] || \
    [ "$DEEP_CLEANUP_ENUMERATION_FAILURE_COUNT" -ne 0 ]; then
    fail "an empty Topics list must be a successful empty enumeration"
fi

SCENARIO="topic-malformed-output"
reset_run
deep_cleanup_sns "cleanup-account" >> "$STDOUT_LOG" 2>> "$STDERR_LOG"
if grep -F "aws sns delete-topic " "$COMMAND_LOG" >/dev/null; then
    fail "malformed topic output must fail closed without SNS deletions"
fi
if [ "$DEEP_CLEANUP_LAST_ENUMERATION_SUCCEEDED" -ne 0 ] || \
    [ "$DEEP_CLEANUP_ENUMERATION_FAILURE_COUNT" -ne 1 ]; then
    fail "malformed topic output must record one enumeration failure"
fi

SCENARIO="topic-list-failure"
reset_run
deep_cleanup_sns "cleanup-account" >> "$STDOUT_LOG" 2>> "$STDERR_LOG"
if grep -F "aws sns delete-topic " "$COMMAND_LOG" >/dev/null; then
    fail "SNS cleanup must fail closed after topic enumeration fails"
fi
if ! grep -F "Skipping SNS cleanup because topics could not be enumerated" "$STDOUT_LOG" >/dev/null; then
    fail "missing fail-closed diagnostic for SNS topic enumeration"
fi
if ! grep -F "Failed to enumerate acceptance-test SNS topics" "$STDERR_LOG" >/dev/null; then
    fail "missing SNS topic enumeration failure diagnostic"
fi

SCENARIO="topic-delete-failure"
reset_run
deep_cleanup_sns "cleanup-account" >> "$STDOUT_LOG" 2>> "$STDERR_LOG"
if [ "$DEEP_CLEANUP_FAILED_COUNT" -ne 1 ] || [ "$DEEP_CLEANUP_DELETED_COUNT" -ne 0 ]; then
    fail "topic delete failure must be recorded"
fi
if ! grep -F "Failed to delete SNS topic $TOPIC_ONE" "$STDERR_LOG" >/dev/null; then
    fail "missing SNS topic delete failure diagnostic"
fi

if [ "$FAILURES" -ne 0 ]; then
    exit 1
fi

echo "PASS: SNS deep cleanup is topic-scoped and fail-closed"
