#!/bin/bash
# Real-AWS acceptance test for typed SNS Subscription policies.
#
# Usage:
#   env AWS_PROFILE="carina-test-008" ./run.sh
#
# AWSCC does not expose AWS::SQS::Queue, so this runner creates four queues
# with AWS CLI, renders their ARNs into temporary Carina fixtures, and removes
# every queue during cleanup. This directory has a custom runner and is
# intentionally skipped by acceptance-tests/run-tests.sh.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck disable=SC1091
source "$SCRIPT_DIR/../shared/_helpers.sh"

REGION="ap-northeast-1"
STANDARD_TOPIC_NAME="carina-acc-test-sns-policies-standard"
FIFO_TOPIC_NAME="carina-acc-test-sns-policies-fifo.fifo"
STANDARD_QUEUE_NAME="carina-acc-test-sns-policies-queue"
STANDARD_DLQ_NAME="carina-acc-test-sns-policies-dlq"
FIFO_QUEUE_NAME="carina-acc-test-sns-policies-queue.fifo"
FIFO_DLQ_NAME="carina-acc-test-sns-policies-dlq.fifo"

TOTAL_PASSED=0
TOTAL_FAILED=0
RUN_TEMP_DIR=""
ACTIVE_WORK_DIR=""
APPLIED_STEP=0
QUEUES_CLEANED=0
ACCOUNT_ID=""
STANDARD_TOPIC_ARN=""
FIFO_TOPIC_ARN=""
STANDARD_QUEUE_URL=""
STANDARD_QUEUE_ARN=""
STANDARD_DLQ_URL=""
STANDARD_DLQ_ARN=""
FIFO_QUEUE_URL=""
FIFO_QUEUE_ARN=""
FIFO_DLQ_URL=""
FIFO_DLQ_ARN=""

with_account_creds() {
    local profile="${AWS_PROFILE:-}"
    if [ -z "$profile" ]; then
        echo "ERROR: AWS_PROFILE is not set; cannot resolve credentials" >&2
        return 1
    fi

    local creds
    if ! creds=$(aws configure export-credentials \
        --profile "$profile" --format env-no-export 2>&1); then
        echo "ERROR: aws configure export-credentials failed for $profile: $creds" >&2
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

run_step() {
    local description="$1"
    shift

    printf "  %-62s " "$description"
    local output
    if output=$("$@" 2>&1); then
        echo "OK"
        TOTAL_PASSED=$((TOTAL_PASSED + 1))
        return 0
    fi

    echo "FAIL"
    echo "  ERROR: $output"
    TOTAL_FAILED=$((TOTAL_FAILED + 1))
    return 1
}

resolve_account_id() {
    with_account_creds aws sts get-caller-identity \
        --region "$REGION" --query Account --output text > "$RUN_TEMP_DIR/account-id"
}

create_queue() {
    local queue_name="$1"
    local kind="$2"
    local output_prefix="$3"
    local queue_url
    local queue_arn

    if [ "$kind" = "fifo" ]; then
        queue_url=$(with_account_creds aws sqs create-queue \
            --region "$REGION" \
            --queue-name "$queue_name" \
            --attributes '{"FifoQueue":"true","ContentBasedDeduplication":"true"}' \
            --query QueueUrl --output text) || return 1
    else
        queue_url=$(with_account_creds aws sqs create-queue \
            --region "$REGION" \
            --queue-name "$queue_name" \
            --query QueueUrl --output text) || return 1
    fi
    printf '%s\n' "$queue_url" > "$RUN_TEMP_DIR/$output_prefix.url"

    queue_arn=$(with_account_creds aws sqs get-queue-attributes \
        --region "$REGION" \
        --queue-url "$queue_url" \
        --attribute-names QueueArn \
        --query 'Attributes.QueueArn' --output text) || return 1
    printf '%s\n' "$queue_arn" > "$RUN_TEMP_DIR/$output_prefix.arn"
}

load_queue_values() {
    STANDARD_QUEUE_URL="$(<"$RUN_TEMP_DIR/standard-queue.url")"
    STANDARD_QUEUE_ARN="$(<"$RUN_TEMP_DIR/standard-queue.arn")"
    STANDARD_DLQ_URL="$(<"$RUN_TEMP_DIR/standard-dlq.url")"
    STANDARD_DLQ_ARN="$(<"$RUN_TEMP_DIR/standard-dlq.arn")"
    FIFO_QUEUE_URL="$(<"$RUN_TEMP_DIR/fifo-queue.url")"
    FIFO_QUEUE_ARN="$(<"$RUN_TEMP_DIR/fifo-queue.arn")"
    FIFO_DLQ_URL="$(<"$RUN_TEMP_DIR/fifo-dlq.url")"
    FIFO_DLQ_ARN="$(<"$RUN_TEMP_DIR/fifo-dlq.arn")"
}

load_queue_values_if_present() {
    [ -s "$RUN_TEMP_DIR/standard-queue.url" ] && \
        STANDARD_QUEUE_URL="$(<"$RUN_TEMP_DIR/standard-queue.url")"
    [ -s "$RUN_TEMP_DIR/standard-queue.arn" ] && \
        STANDARD_QUEUE_ARN="$(<"$RUN_TEMP_DIR/standard-queue.arn")"
    [ -s "$RUN_TEMP_DIR/standard-dlq.url" ] && \
        STANDARD_DLQ_URL="$(<"$RUN_TEMP_DIR/standard-dlq.url")"
    [ -s "$RUN_TEMP_DIR/standard-dlq.arn" ] && \
        STANDARD_DLQ_ARN="$(<"$RUN_TEMP_DIR/standard-dlq.arn")"
    [ -s "$RUN_TEMP_DIR/fifo-queue.url" ] && \
        FIFO_QUEUE_URL="$(<"$RUN_TEMP_DIR/fifo-queue.url")"
    [ -s "$RUN_TEMP_DIR/fifo-queue.arn" ] && \
        FIFO_QUEUE_ARN="$(<"$RUN_TEMP_DIR/fifo-queue.arn")"
    [ -s "$RUN_TEMP_DIR/fifo-dlq.url" ] && \
        FIFO_DLQ_URL="$(<"$RUN_TEMP_DIR/fifo-dlq.url")"
    [ -s "$RUN_TEMP_DIR/fifo-dlq.arn" ] && \
        FIFO_DLQ_ARN="$(<"$RUN_TEMP_DIR/fifo-dlq.arn")"
    return 0
}

set_sns_queue_policy() {
    local queue_url="$1"
    local queue_arn="$2"
    local topic_arn="$3"
    local policy
    local attributes

    policy=$(jq -nc \
        --arg queue_arn "$queue_arn" \
        --arg topic_arn "$topic_arn" \
        '{
            Version: "2012-10-17",
            Statement: [{
                Sid: "AllowSnsSendMessage",
                Effect: "Allow",
                Principal: {Service: "sns.amazonaws.com"},
                Action: "sqs:SendMessage",
                Resource: $queue_arn,
                Condition: {ArnEquals: {"aws:SourceArn": $topic_arn}}
            }]
        }') || return 1
    attributes=$(jq -nc --arg policy "$policy" '{Policy: $policy}') || return 1

    with_account_creds aws sqs set-queue-attributes \
        --region "$REGION" --queue-url "$queue_url" --attributes "$attributes"
}

configure_queue_policies() {
    set_sns_queue_policy "$STANDARD_QUEUE_URL" "$STANDARD_QUEUE_ARN" "$STANDARD_TOPIC_ARN" || return 1
    set_sns_queue_policy "$STANDARD_DLQ_URL" "$STANDARD_DLQ_ARN" "$STANDARD_TOPIC_ARN" || return 1
    set_sns_queue_policy "$FIFO_QUEUE_URL" "$FIFO_QUEUE_ARN" "$FIFO_TOPIC_ARN" || return 1
    set_sns_queue_policy "$FIFO_DLQ_URL" "$FIFO_DLQ_ARN" "$FIFO_TOPIC_ARN"
}

render_fixture() {
    local source_file="$1"
    local target_file="$2"
    local replay_starting_point="$3"

    sed \
        -e "s|__STANDARD_QUEUE_ARN__|$STANDARD_QUEUE_ARN|g" \
        -e "s|__STANDARD_DLQ_ARN__|$STANDARD_DLQ_ARN|g" \
        -e "s|__FIFO_QUEUE_ARN__|$FIFO_QUEUE_ARN|g" \
        -e "s|__FIFO_DLQ_ARN__|$FIFO_DLQ_ARN|g" \
        -e "s|__REPLAY_STARTING_POINT__|$replay_starting_point|g" \
        "$source_file" > "$target_file"
}

render_step1_fixture() {
    render_fixture "$SCRIPT_DIR/step1.crn" "$RUN_TEMP_DIR/step1.crn" ""
}

render_replay_fixtures() {
    local replay_starting_point
    replay_starting_point=$(date -u '+%Y-%m-%dT%H:%M:%S.000Z') || return 1
    printf '%s\n' "$replay_starting_point" > "$RUN_TEMP_DIR/replay-starting-point"

    render_fixture "$SCRIPT_DIR/step2.crn" "$RUN_TEMP_DIR/step2.crn" \
        "$replay_starting_point" || return 1
    render_fixture "$SCRIPT_DIR/step3.crn" "$RUN_TEMP_DIR/step3.crn" \
        "$replay_starting_point"
}

install_config() {
    local rendered_crn="$1"
    local injected_dir

    inject_provider_source "$rendered_crn" injected_dir || return 1
    if ! cp "$injected_dir/main.crn" "$ACTIVE_WORK_DIR/main.crn"; then
        rm -rf "$injected_dir"
        return 1
    fi
    rm -rf "$injected_dir"
}

carina_init() {
    cd "$ACTIVE_WORK_DIR"
    with_account_creds run_with_timeout 600 "$CARINA_BIN" init .
}

carina_apply() {
    cd "$ACTIVE_WORK_DIR"
    with_account_creds run_with_timeout 900 "$CARINA_BIN" apply --auto-approve .
}

carina_destroy() {
    cd "$ACTIVE_WORK_DIR"
    with_account_creds run_with_timeout 900 "$CARINA_BIN" destroy --auto-approve .
}

plan_verify() {
    local description="$1"
    printf "  %-62s " "$description"

    local output
    local rc=0
    output=$(cd "$ACTIVE_WORK_DIR" && \
        with_account_creds run_with_timeout 600 \
            "$CARINA_BIN" plan --detailed-exitcode . 2>&1) || rc=$?

    if [ "$rc" -eq 0 ]; then
        echo "OK"
        TOTAL_PASSED=$((TOTAL_PASSED + 1))
        return 0
    fi

    echo "FAIL"
    if [ "$rc" -eq 2 ]; then
        echo "  ERROR: Post-apply plan detected changes (not idempotent):"
    else
        echo "  ERROR: plan failed with exit code $rc:"
    fi
    echo "$output"
    TOTAL_FAILED=$((TOTAL_FAILED + 1))
    return 1
}

delete_queue_by_name() {
    local queue_name="$1"
    local output
    local queue_url

    if output=$(with_account_creds aws sqs get-queue-url \
        --region "$REGION" --queue-name "$queue_name" \
        --query QueueUrl --output text 2>&1); then
        queue_url="$output"
    elif grep -Eq 'AWS.SimpleQueueService.NonExistentQueue|QueueDoesNotExist|does not exist' \
        <<< "$output"; then
        return 0
    else
        echo "$output" >&2
        return 1
    fi

    with_account_creds aws sqs delete-queue \
        --region "$REGION" --queue-url "$queue_url"
}

delete_all_queues() {
    local failed=0
    delete_queue_by_name "$STANDARD_QUEUE_NAME" || failed=1
    delete_queue_by_name "$STANDARD_DLQ_NAME" || failed=1
    delete_queue_by_name "$FIFO_QUEUE_NAME" || failed=1
    delete_queue_by_name "$FIFO_DLQ_NAME" || failed=1
    return "$failed"
}

queue_exists() {
    local queue_name="$1"
    local output

    if output=$(with_account_creds aws sqs get-queue-url \
        --region "$REGION" --queue-name "$queue_name" \
        --query QueueUrl --output text 2>&1); then
        return 0
    fi
    if grep -Eq 'AWS.SimpleQueueService.NonExistentQueue|QueueDoesNotExist|does not exist' \
        <<< "$output"; then
        return 1
    fi
    echo "$output" >&2
    return 2
}

assert_queues_gone() {
    local attempt
    local queue_name
    local status
    local any_exists

    for attempt in $(seq 1 12); do
        any_exists=0
        for queue_name in \
            "$STANDARD_QUEUE_NAME" "$STANDARD_DLQ_NAME" \
            "$FIFO_QUEUE_NAME" "$FIFO_DLQ_NAME"; do
            if queue_exists "$queue_name"; then
                any_exists=1
            else
                status=$?
                if [ "$status" -ne 1 ]; then
                    return 1
                fi
            fi
        done
        if [ "$any_exists" -eq 0 ]; then
            return 0
        fi
        [ "$attempt" -eq 12 ] || sleep 5
    done

    echo "One or more acceptance-test queues still exist after deletion" >&2
    return 1
}

assert_topics_gone() {
    local attempt
    local topics

    for attempt in $(seq 1 12); do
        topics=$(with_account_creds aws sns list-topics \
            --region "$REGION" --output json) || return 1
        if ! jq -e \
            --arg standard "$STANDARD_TOPIC_ARN" \
            --arg fifo "$FIFO_TOPIC_ARN" \
            '.Topics[]?.TopicArn | select(. == $standard or . == $fifo)' \
            <<< "$topics" >/dev/null; then
            return 0
        fi
        [ "$attempt" -eq 12 ] || sleep 5
    done

    echo "One or more acceptance-test topics still exist after deletion" >&2
    return 1
}

unsubscribe_topic() {
    local topic_arn="$1"
    local subscriptions
    local subscription_arn

    subscriptions=$(with_account_creds aws sns list-subscriptions-by-topic \
        --region "$REGION" --topic-arn "$topic_arn" --output json 2>/dev/null) || return 0
    while IFS= read -r subscription_arn; do
        [ -n "$subscription_arn" ] || continue
        [ "$subscription_arn" = "PendingConfirmation" ] && continue
        with_account_creds aws sns unsubscribe \
            --region "$REGION" --subscription-arn "$subscription_arn" || return 1
    done < <(jq -r '.Subscriptions[]?.SubscriptionArn' <<< "$subscriptions")
}

force_cleanup_topics() {
    local failed=0

    unsubscribe_topic "$STANDARD_TOPIC_ARN" || failed=1
    unsubscribe_topic "$FIFO_TOPIC_ARN" || failed=1

    # ArchivePolicy was measured to block topic deletion. This is only a
    # last-resort cleanup for these dedicated acceptance resources; the test's
    # normal path removes it through step3 and verifies the resulting plan.
    with_account_creds aws sns set-topic-attributes \
        --region "$REGION" \
        --topic-arn "$FIFO_TOPIC_ARN" \
        --attribute-name ArchivePolicy \
        --attribute-value '{}' >/dev/null 2>&1 || true

    with_account_creds aws sns delete-topic \
        --region "$REGION" --topic-arn "$STANDARD_TOPIC_ARN" >/dev/null 2>&1 || true
    with_account_creds aws sns delete-topic \
        --region "$REGION" --topic-arn "$FIFO_TOPIC_ARN" >/dev/null 2>&1 || true

    assert_topics_gone || failed=1
    return "$failed"
}

cleanup() {
    local original_status=$?
    local cleanup_failed=0
    trap - EXIT INT TERM
    set +e

    if [ -n "$RUN_TEMP_DIR" ] && [ -d "$RUN_TEMP_DIR" ]; then
        load_queue_values_if_present
    fi

    if [ -n "$ACTIVE_WORK_DIR" ] && [ -d "$ACTIVE_WORK_DIR" ]; then
        echo ""
        echo "  Cleanup: removing SNS resources..."

        if [ "$APPLIED_STEP" -ge 2 ] && [ -s "$RUN_TEMP_DIR/step3.crn" ]; then
            if install_config "$RUN_TEMP_DIR/step3.crn"; then
                carina_apply || true
            fi
        fi

        carina_destroy || true
        carina_destroy || true

        if [ -n "$ACCOUNT_ID" ]; then
            force_cleanup_topics || cleanup_failed=1
        fi

        # Reconcile state after an out-of-band fallback removed the resources.
        carina_destroy || true
        if ! release_acceptance_work_dir "$ACTIVE_WORK_DIR"; then
            cleanup_failed=1
        fi
        ACTIVE_WORK_DIR=""
    fi

    if [ "$QUEUES_CLEANED" -ne 1 ] && [ -n "${AWS_PROFILE:-}" ]; then
        echo "  Cleanup: deleting SQS queues..."
        delete_all_queues || cleanup_failed=1
        assert_queues_gone || cleanup_failed=1
    fi

    if [ -n "$RUN_TEMP_DIR" ] && [ -d "$RUN_TEMP_DIR" ]; then
        rm -rf "$RUN_TEMP_DIR"
    fi

    if [ "$cleanup_failed" -ne 0 ]; then
        echo "  ERROR: cleanup could not verify that every acceptance resource was removed" >&2
        exit 1
    fi
    exit "$original_status"
}

trap cleanup EXIT
trap 'exit 130' INT
trap 'exit 143' TERM

for command_name in aws jq sed date; do
    if ! command -v "$command_name" >/dev/null 2>&1; then
        echo "ERROR: required command is unavailable: $command_name" >&2
        exit 1
    fi
done
if [ -z "${AWS_PROFILE:-}" ]; then
    echo "ERROR: AWS_PROFILE must name the carina test account profile" >&2
    exit 1
fi

RUN_TEMP_DIR="$(mktemp -d)"

echo "Running SNS Subscription typed-policy acceptance test with AWS_PROFILE=$AWS_PROFILE"
echo ""

run_step "resolve AWS account id" resolve_account_id || exit 1
ACCOUNT_ID="$(<"$RUN_TEMP_DIR/account-id")"
case "$ACCOUNT_ID" in
    [0-9][0-9][0-9][0-9][0-9][0-9][0-9][0-9][0-9][0-9][0-9][0-9]) ;;
    *)
        echo "ERROR: invalid AWS account id: $ACCOUNT_ID" >&2
        exit 1
        ;;
esac
STANDARD_TOPIC_ARN="arn:aws:sns:$REGION:$ACCOUNT_ID:$STANDARD_TOPIC_NAME"
FIFO_TOPIC_ARN="arn:aws:sns:$REGION:$ACCOUNT_ID:$FIFO_TOPIC_NAME"

run_step "create standard SQS queue" \
    create_queue "$STANDARD_QUEUE_NAME" standard standard-queue || exit 1
run_step "create standard SQS dead-letter queue" \
    create_queue "$STANDARD_DLQ_NAME" standard standard-dlq || exit 1
run_step "create FIFO SQS queue" \
    create_queue "$FIFO_QUEUE_NAME" fifo fifo-queue || exit 1
run_step "create FIFO SQS dead-letter queue" \
    create_queue "$FIFO_DLQ_NAME" fifo fifo-dlq || exit 1
load_queue_values

run_step "configure SQS policies for SNS delivery" configure_queue_policies || exit 1
run_step "render queue ARNs into step1 fixture" render_step1_fixture || exit 1

inject_provider_source "$RUN_TEMP_DIR/step1.crn" INJECTED_DIR || exit 1
register_acceptance_work_dir \
    "sns_subscription_policies" "$INJECTED_DIR/main.crn" WORK_DIR
rm -rf "$INJECTED_DIR"
ACTIVE_WORK_DIR="$WORK_DIR"

run_step "init registered Carina work directory" carina_init || exit 1

run_step "step1: apply standard policies and FIFO archive" carina_apply || exit 1
APPLIED_STEP=1
plan_verify "step1: plan-verify standard policies and FIFO archive" || exit 1

run_step "capture post-archive timestamp and render step2/step3" \
    render_replay_fixtures || exit 1
run_step "step2: install FIFO replay fixture" \
    install_config "$RUN_TEMP_DIR/step2.crn" || exit 1
run_step "step2: apply FIFO replay policy" carina_apply || exit 1
APPLIED_STEP=2
plan_verify "step2: plan-verify archive and replay read-back" || exit 1

run_step "step3: install archive/replay removal fixture" \
    install_config "$RUN_TEMP_DIR/step3.crn" || exit 1
if ! run_step "step3: apply archive and replay removal" carina_apply; then
    echo ""
    echo "MEASUREMENT: step3 failed while a replay subscription existed."
    echo "The AWS/Carina error above is intentionally fatal and is not hidden by cleanup."
    exit 1
fi
APPLIED_STEP=3
plan_verify "step3: plan-verify archive and replay removal" || exit 1

run_step "destroy SNS subscriptions and topics through Carina" carina_destroy || exit 1
run_step "assert SNS topics are gone" assert_topics_gone || exit 1
run_step "delete out-of-band SQS queues" delete_all_queues || exit 1
run_step "assert SQS queues are gone" assert_queues_gone || exit 1
QUEUES_CLEANED=1

if ! release_acceptance_work_dir "$ACTIVE_WORK_DIR"; then
    TOTAL_FAILED=$((TOTAL_FAILED + 1))
    exit 1
fi
TOTAL_PASSED=$((TOTAL_PASSED + 1))
ACTIVE_WORK_DIR=""

rm -rf "$RUN_TEMP_DIR"
RUN_TEMP_DIR=""

echo ""
echo "Results: $TOTAL_PASSED passed, $TOTAL_FAILED failed"
if [ "$TOTAL_FAILED" -gt 0 ]; then
    exit 1
fi
