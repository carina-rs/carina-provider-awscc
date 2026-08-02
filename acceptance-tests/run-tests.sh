#!/bin/bash
# Run acceptance tests for AWSCC provider
#
# Usage (from project root):
#   ./carina-provider-awscc/acceptance-tests/run-tests.sh [options] [command] [filter...]
#
# Options:
#   --accounts START-END  Use only accounts in range (e.g., 0-3, 4-6, 7-9)
#                         Enables concurrent runs with non-overlapping account pools
#   --include-slow        Include tests marked as slow (skipped by default)
#                         Tests are marked slow by placing a .slow file next to the .crn file
#
# Commands:
#   validate   - Validate .crn files (default, no AWS credentials needed)
#   plan       - Run plan on .crn files (single account, needs AWS credentials)
#   apply      - Apply .crn files (single account, needs AWS credentials)
#   destroy    - Destroy resources created by .crn files (single account)
#   full       - Run apply+plan-verify+destroy per test, accounts in parallel
#   cleanup    - Run destroy across accounts in parallel (recover from stuck state)
#   deep-cleanup - Scan and remove orphaned AWS resources across accounts
#
# AWS authentication:
#   The runner selects credentials via AWS_PROFILE rather than wrapping in
#   aws-vault. Each carina-test-00X profile shares a single sso-session
#   (`carina`), so the AWS SDK can refresh the SSO access token on its own
#   during long-running operations (see carina-rs/carina-provider-awscc#157).
#
#   Before the first run in a session, log in once:
#     aws sso login --sso-session carina
#
# For validate, no AWS credentials are needed.
# For plan/apply/destroy, wrap with: with_account_creds "<profile>" ./run-tests.sh ...
# For full, AWS_PROFILE is set internally per account (carina-test-000..009).
#
# Filter (optional):
#   One or more substrings to match against test paths.
#   A test is included if it matches ANY of the provided filters (OR logic).
#   When no filter is provided, all tests are included.
#
# Examples:
#   ./run-tests.sh validate                          # validate all
#   ./run-tests.sh validate ec2_vpc/basic            # validate specific test
#   ./run-tests.sh full                              # apply+plan-verify+destroy, 10 parallel accounts
#   ./run-tests.sh full ec2_vpc                      # apply+plan-verify+destroy VPC tests only
#   ./run-tests.sh full ec2_ipam ec2_vpc/with_ipam   # multiple filters in single invocation
#   ./run-tests.sh full --accounts 0-3 iam_role      # use only accounts 000-003
#   ./run-tests.sh full --accounts 4-6 ec2_vpc       # concurrent run with accounts 004-006
#   ./run-tests.sh full --include-slow               # include slow tests in the run
#   ./run-tests.sh full ec2_subnet/with_ipam          # slow tests run when explicitly filtered
#   ./run-tests.sh cleanup                           # destroy all matching tests across 10 accounts
#   ./run-tests.sh cleanup ec2_vpc                   # destroy VPC tests only across 10 accounts
#
# Performance:
#   WASM provider loading with debug builds is slow (JIT compilation ~4s per invocation).
#   Use release builds for significantly faster test runs:
#     cd ../carina && cargo build --release
#     export CARINA_BIN="$PWD/../carina/target/release/carina"

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

# ── Parse options ─────────────────────────────────────────────────────
ACCOUNT_START=""
ACCOUNT_END=""
INCLUDE_SLOW=0

# Parse options before command
ARGS=()
while [ $# -gt 0 ]; do
    case "$1" in
        --accounts)
            if [ -z "${2:-}" ]; then
                echo "ERROR: --accounts requires a range argument (e.g., 0-3)"
                exit 1
            fi
            RANGE="$2"
            ACCOUNT_START="${RANGE%-*}"
            ACCOUNT_END="${RANGE#*-}"
            if ! echo "$ACCOUNT_START" | grep -q '^[0-9]\{1,\}$' || ! echo "$ACCOUNT_END" | grep -q '^[0-9]\{1,\}$'; then
                echo "ERROR: --accounts range must be START-END (e.g., 0-3)"
                exit 1
            fi
            if [ "$ACCOUNT_START" -gt "$ACCOUNT_END" ]; then
                echo "ERROR: --accounts START must be <= END (got $ACCOUNT_START-$ACCOUNT_END)"
                exit 1
            fi
            if [ "$ACCOUNT_START" -gt 9 ] || [ "$ACCOUNT_END" -gt 9 ]; then
                echo "ERROR: --accounts range must be within 0-9"
                exit 1
            fi
            shift 2
            ;;
        --include-slow)
            INCLUDE_SLOW=1
            shift
            ;;
        *)
            ARGS+=("$1")
            shift
            ;;
    esac
done

set -- ${ARGS[@]+"${ARGS[@]}"}

COMMAND="${1:-validate}"
shift || true
FILTERS=("$@")

# ── Build account list ────────────────────────────────────────────────
ALL_ACCOUNTS=(
    carina-test-000
    carina-test-001
    carina-test-002
    carina-test-003
    carina-test-004
    carina-test-005
    carina-test-006
    carina-test-007
    carina-test-008
    carina-test-009
)

if [ -n "$ACCOUNT_START" ]; then
    ACCOUNTS=()
    ACCOUNT_INDICES=()
    for i in $(seq "$ACCOUNT_START" "$ACCOUNT_END"); do
        ACCOUNTS+=("${ALL_ACCOUNTS[$i]}")
        ACCOUNT_INDICES+=("$i")
    done
else
    ACCOUNTS=("${ALL_ACCOUNTS[@]}")
    ACCOUNT_INDICES=()
    for i in $(seq 0 9); do
        ACCOUNT_INDICES+=("$i")
    done
fi
NUM_ACCOUNTS=${#ACCOUNTS[@]}

# ── Per-account lock files ────────────────────────────────────────────
# Each account gets its own lock to allow concurrent runs with
# non-overlapping account pools.
LOCK_BASE="/tmp/carina-acceptance-tests"
LOCKED_ACCOUNTS=()

acquire_account_lock() {
    local account_index="$1"
    local lock_dir="${LOCK_BASE}-${account_index}.lock"

    if mkdir "$lock_dir" 2>/dev/null; then
        echo $$ > "$lock_dir/pid"
        LOCKED_ACCOUNTS+=("$account_index")
        return 0
    fi

    # Lock exists - check if the holding process is still alive
    if [ -f "$lock_dir/pid" ]; then
        OLD_PID=$(cat "$lock_dir/pid" 2>/dev/null || echo "")
        if [ -n "$OLD_PID" ] && kill -0 "$OLD_PID" 2>/dev/null; then
            echo "ERROR: Account ${account_index} is already locked by another run-tests.sh instance (PID $OLD_PID)"
            return 1
        fi
    fi

    # Stale lock - reclaim
    echo "Removing stale lock for account ${account_index} (previous process no longer running)"
    rm -rf "$lock_dir"
    mkdir "$lock_dir" 2>/dev/null || {
        echo "ERROR: Failed to acquire lock for account ${account_index} (race condition). Retry."
        return 1
    }
    echo $$ > "$lock_dir/pid"
    LOCKED_ACCOUNTS+=("$account_index")
    return 0
}

release_account_locks() {
    for account_index in ${LOCKED_ACCOUNTS[@]+"${LOCKED_ACCOUNTS[@]}"}; do
        rm -rf "${LOCK_BASE}-${account_index}.lock"
    done
    LOCKED_ACCOUNTS=()
}

acquire_all_account_locks() {
    for account_index in "${ACCOUNT_INDICES[@]}"; do
        if ! acquire_account_lock "$account_index"; then
            # Release any locks we already acquired
            release_account_locks
            return 1
        fi
    done
    return 0
}

PIDS=()

if [ "$COMMAND" != "validate" ]; then
    if ! acquire_all_account_locks; then
        exit 1
    fi
    trap 'for p in ${PIDS[@]+"${PIDS[@]}"}; do kill "$p" 2>/dev/null || true; done; release_account_locks' EXIT
fi

# Validate command
case "$COMMAND" in
    validate|plan|apply|destroy|full|cleanup|deep-cleanup)
        ;;
    *)
        echo "ERROR: Unknown command '$COMMAND'"
        echo "Usage: $0 [--accounts START-END] [validate|plan|apply|destroy|full|cleanup|deep-cleanup] [filter...]"
        exit 1
        ;;
esac

# ── with_account_creds: resolve the profile's credentials into env vars ──
# and exec the given command. Uses `aws configure export-credentials` so the
# resulting AWS_ACCESS_KEY_ID / AWS_SESSION_TOKEN are fresh STS creds derived
# from the sso-session; the SSO access token in ~/.aws/sso/cache is refreshed
# automatically when it is close to expiring. Resolving creds before each
# carina invocation means every test starts with a new STS session, so a
# single long-running apply/destroy is not limited by the SSO access-token
# lifetime (see carina-rs/carina-provider-awscc#157).
#
# Usage: with_account_creds <account> <cmd> [args...]
with_account_creds() {
    local account="$1"
    shift
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

# Writes successful stdout to the named caller variable. Keeping the helper out
# of a command substitution lets its failure counter remain in this shell.
deep_cleanup_enumerate() {
    local account="$1"
    local resource_type="$2"
    local output_variable="$3"
    shift 3

    local enumeration_output=""
    local enumeration_stderr=""
    local enumeration_stderr_file
    local enumeration_status=0
    local enumeration_succeeded=0
    if ! enumeration_stderr_file=$(mktemp); then
        DEEP_CLEANUP_LAST_ENUMERATION_SUCCEEDED=0
        DEEP_CLEANUP_ENUMERATION_FAILURE_COUNT=$((${DEEP_CLEANUP_ENUMERATION_FAILURE_COUNT:-0} + 1))
        printf -v "$output_variable" '%s' ""
        echo "  ERROR: Failed to enumerate $resource_type for account $account: unable to create a temporary stderr file" >&2
        return 0
    fi

    # Keep separate read/write descriptions, then unlink the path before the AWS
    # call. The open file survives long enough to capture stderr, while a killed
    # cleanup process cannot leave a named temporary file behind.
    if ! exec 8<"$enumeration_stderr_file"; then
        rm -f "$enumeration_stderr_file" 2>/dev/null || true
        DEEP_CLEANUP_LAST_ENUMERATION_SUCCEEDED=0
        DEEP_CLEANUP_ENUMERATION_FAILURE_COUNT=$((${DEEP_CLEANUP_ENUMERATION_FAILURE_COUNT:-0} + 1))
        printf -v "$output_variable" '%s' ""
        echo "  ERROR: Failed to enumerate $resource_type for account $account: unable to open the temporary stderr file for reading" >&2
        return 0
    fi
    if ! exec 9>"$enumeration_stderr_file"; then
        exec 8<&-
        rm -f "$enumeration_stderr_file" 2>/dev/null || true
        DEEP_CLEANUP_LAST_ENUMERATION_SUCCEEDED=0
        DEEP_CLEANUP_ENUMERATION_FAILURE_COUNT=$((${DEEP_CLEANUP_ENUMERATION_FAILURE_COUNT:-0} + 1))
        printf -v "$output_variable" '%s' ""
        echo "  ERROR: Failed to enumerate $resource_type for account $account: unable to open the temporary stderr file for writing" >&2
        return 0
    fi
    if ! rm -f "$enumeration_stderr_file"; then
        exec 9>&-
        exec 8<&-
        DEEP_CLEANUP_LAST_ENUMERATION_SUCCEEDED=0
        DEEP_CLEANUP_ENUMERATION_FAILURE_COUNT=$((${DEEP_CLEANUP_ENUMERATION_FAILURE_COUNT:-0} + 1))
        printf -v "$output_variable" '%s' ""
        echo "  ERROR: Failed to enumerate $resource_type for account $account: unable to unlink the temporary stderr file" >&2
        return 0
    fi

    if enumeration_output=$(with_account_creds "$account" "$@" 2>&9); then
        enumeration_succeeded=1
    else
        enumeration_status=$?
    fi
    exec 9>&-
    enumeration_stderr=$(cat <&8) || enumeration_stderr="unable to read captured stderr"
    exec 8<&-

    if [ "$enumeration_succeeded" -eq 1 ]; then
        DEEP_CLEANUP_LAST_ENUMERATION_SUCCEEDED=1
        printf -v "$output_variable" '%s' "$enumeration_output"
    else
        DEEP_CLEANUP_LAST_ENUMERATION_SUCCEEDED=0
        DEEP_CLEANUP_ENUMERATION_FAILURE_COUNT=$((${DEEP_CLEANUP_ENUMERATION_FAILURE_COUNT:-0} + 1))
        printf -v "$output_variable" '%s' ""
        local enumeration_diagnostic="$enumeration_stderr"
        if [ -n "$enumeration_output" ]; then
            enumeration_diagnostic="${enumeration_diagnostic}${enumeration_diagnostic:+$'\n'}${enumeration_output}"
        fi
        if [ -z "$enumeration_diagnostic" ]; then
            enumeration_diagnostic="no error output"
        fi
        echo "  ERROR: Failed to enumerate $resource_type for account $account (exit $enumeration_status): $enumeration_diagnostic" >&2
    fi

    # Enumeration failure means an empty result for this step, never an aborted sweep.
    return 0
}

# Runs one cleanup mutation, reports any failure, and writes 1 or 0 to the
# named caller variable without propagating the command's non-zero status.
_deep_cleanup_delete() {
    local account="$1"
    local operation="$2"
    local result_variable="$3"
    shift 3

    local delete_output
    local delete_status
    if delete_output=$(with_account_creds "$account" "$@" 2>&1); then
        printf -v "$result_variable" '%s' "1"
    else
        delete_status=$?
        printf -v "$result_variable" '%s' "0"
        echo "  ERROR: Failed to $operation (exit $delete_status): $delete_output" >&2
    fi

    # A stubborn resource is recorded above but must not prevent later cleanup steps.
    return 0
}

deep_cleanup_delete_resource() {
    local account="$1"
    local operation="$2"
    shift 2

    local delete_succeeded=0
    _deep_cleanup_delete "$account" "$operation" delete_succeeded "$@"
    DEEP_CLEANUP_LAST_DELETE_SUCCEEDED="$delete_succeeded"
    if [ "$delete_succeeded" -eq 1 ]; then
        DEEP_CLEANUP_DELETED_COUNT=$((${DEEP_CLEANUP_DELETED_COUNT:-0} + 1))
    else
        DEEP_CLEANUP_FAILED_COUNT=$((${DEEP_CLEANUP_FAILED_COUNT:-0} + 1))
    fi

    return 0
}

deep_cleanup_delete_supporting() {
    local account="$1"
    local operation="$2"
    shift 2

    local delete_succeeded=0
    _deep_cleanup_delete "$account" "$operation" delete_succeeded "$@"
    if [ "$delete_succeeded" -eq 0 ]; then
        DEEP_CLEANUP_SUPPORTING_FAILURE_COUNT=$((${DEEP_CLEANUP_SUPPORTING_FAILURE_COUNT:-0} + 1))
    fi

    return 0
}

# Runs a command with a hard wall-clock limit. This is used around AWS waiters,
# whose generated polling limits can otherwise hold up an entire account sweep.
deep_cleanup_with_timeout() {
    local timeout_seconds="$1"
    shift

    local timer_pid_file
    if ! timer_pid_file=$(mktemp); then
        echo "unable to create timeout state file" >&2
        return 1
    fi

    local command_pid
    local timeout_pid
    local command_status=0

    "$@" &
    command_pid=$!
    (
        local timer_sleep_pid=""
        # Invoked by the TERM trap while this subshell still owns the timer.
        # shellcheck disable=SC2329
        _deep_cleanup_stop_timer() {
            local active_timer_pid=""
            active_timer_pid=$(jobs -pr || true)
            if [ -n "$timer_sleep_pid" ] && [ "$active_timer_pid" = "$timer_sleep_pid" ]; then
                kill "$timer_sleep_pid" 2>/dev/null || true
            fi
            if [ -n "$timer_sleep_pid" ]; then
                wait "$timer_sleep_pid" 2>/dev/null || true
            fi
            exit 0
        }
        trap _deep_cleanup_stop_timer TERM

        command sleep "$timeout_seconds" 3>&- >/dev/null 2>&1 &
        timer_sleep_pid=$!
        printf '%s\n' "$timer_sleep_pid" > "$timer_pid_file"
        wait "$timer_sleep_pid" || exit 0
        timer_sleep_pid=""
        if kill -0 "$command_pid" 2>/dev/null; then
            echo "command timed out after $timeout_seconds seconds" >&3
            kill -KILL "$command_pid" 2>/dev/null || true
        fi
    ) 3>&2 </dev/null >/dev/null 2>/dev/null &
    timeout_pid=$!

    local timer_sleep_pid=""
    local timer_setup_attempt=0
    while [ ! -s "$timer_pid_file" ] && [ "$timer_setup_attempt" -lt 100 ]; do
        if ! kill -0 "$timeout_pid" 2>/dev/null; then
            break
        fi
        command sleep 0.01
        timer_setup_attempt=$((timer_setup_attempt + 1))
    done
    if ! IFS= read -r timer_sleep_pid < "$timer_pid_file" || [ -z "$timer_sleep_pid" ]; then
        rm -f "$timer_pid_file" 2>/dev/null || true
        kill "$timeout_pid" 2>/dev/null || true
        wait "$timeout_pid" 2>/dev/null || true
        kill -KILL "$command_pid" 2>/dev/null || true
        wait "$command_pid" 2>/dev/null || true
        echo "unable to start timeout timer" >&2
        return 1
    fi
    rm -f "$timer_pid_file" 2>/dev/null || true

    if wait "$command_pid"; then
        command_status=0
    else
        command_status=$?
    fi

    kill "$timeout_pid" 2>/dev/null || true
    wait "$timeout_pid" 2>/dev/null || true
    return "$command_status"
}

# EC2 treats an ENI disappearing during this waiter as failure, although that
# means AWS completed the release. Suppress only that error; propagate timeouts
# and every other waiter failure to the supporting-cleanup reporter.
deep_cleanup_wait_for_network_interface() {
    local timeout_seconds="$1"
    shift

    local wait_output=""
    local wait_status=0
    if wait_output=$(deep_cleanup_with_timeout "$timeout_seconds" "$@" 2>&1); then
        return 0
    else
        wait_status=$?
    fi

    if [[ "$wait_output" == *"InvalidNetworkInterfaceID.NotFound"* ]]; then
        return 0
    fi
    if [ -n "$wait_output" ]; then
        printf '%s\n' "$wait_output" >&2
    fi
    return "$wait_status"
}

deep_cleanup_list_wafv2_web_acls() {
    local account="$1"
    local output_variable="${2:-}"
    local max_pages=100
    local page_count=0
    local next_marker=""
    local listed_web_acls=""
    local pagination_finished=0

    while [ "$page_count" -lt "$max_pages" ]; do
        local list_args=(aws wafv2 list-web-acls --scope REGIONAL --limit 100 --output json)
        if [ -n "$next_marker" ] && [ "$next_marker" != "None" ]; then
            list_args+=(--next-marker "$next_marker")
        fi

        local page
        deep_cleanup_enumerate "$account" "WAFv2 web ACLs" page "${list_args[@]}"
        if [ "$DEEP_CLEANUP_LAST_ENUMERATION_SUCCEEDED" -eq 0 ]; then
            pagination_finished=1
            break
        fi
        page_count=$((page_count + 1))

        # ASCII unit separator cannot occur in these AWS fields and, unlike a
        # tab, preserves an empty LockToken when Bash reads the record.
        local page_web_acls=""
        local jq_status
        if page_web_acls=$(printf '%s\n' "$page" | jq -r '
                .WebACLs[]?
                | select((.Name | contains("acceptance-test")) or (.Name | contains("carina-acc")))
                | [(.Name // ""), (.Id // ""), (.LockToken // ""), (.ARN // "")]
                | join("\u001f")
            ' 2>&1); then
            :
        else
            jq_status=$?
            DEEP_CLEANUP_ENUMERATION_FAILURE_COUNT=$((${DEEP_CLEANUP_ENUMERATION_FAILURE_COUNT:-0} + 1))
            echo "  ERROR: Failed to parse WAFv2 web ACL page for account $account (exit $jq_status): $page_web_acls" >&2
            pagination_finished=1
            break
        fi
        if [ -n "$page_web_acls" ]; then
            listed_web_acls="${listed_web_acls}${listed_web_acls:+$'\n'}${page_web_acls}"
        fi

        local returned_marker=""
        if returned_marker=$(printf '%s\n' "$page" | jq -r '.NextMarker // empty' 2>&1); then
            :
        else
            jq_status=$?
            DEEP_CLEANUP_ENUMERATION_FAILURE_COUNT=$((${DEEP_CLEANUP_ENUMERATION_FAILURE_COUNT:-0} + 1))
            echo "  ERROR: Failed to parse WAFv2 NextMarker for account $account (exit $jq_status): $returned_marker" >&2
            pagination_finished=1
            break
        fi
        if [ -z "$returned_marker" ] || [ "$returned_marker" = "None" ]; then
            pagination_finished=1
            break
        fi

        if [ "$returned_marker" = "$next_marker" ]; then
            DEEP_CLEANUP_ENUMERATION_FAILURE_COUNT=$((${DEEP_CLEANUP_ENUMERATION_FAILURE_COUNT:-0} + 1))
            echo "  ERROR: WAFv2 WebACL pagination truncated for account $account: repeated NextMarker after $page_count pages" >&2
            pagination_finished=1
            break
        fi
        next_marker="$returned_marker"
    done

    if [ "$pagination_finished" -eq 0 ]; then
        DEEP_CLEANUP_ENUMERATION_FAILURE_COUNT=$((${DEEP_CLEANUP_ENUMERATION_FAILURE_COUNT:-0} + 1))
        echo "  ERROR: WAFv2 WebACL pagination truncated for account $account after $max_pages pages" >&2
    fi

    if [ -n "$output_variable" ]; then
        printf -v "$output_variable" '%s' "$listed_web_acls"
    elif [ -n "$listed_web_acls" ]; then
        printf '%s\n' "$listed_web_acls"
    fi

    return 0
}

deep_cleanup_disassociate_wafv2_web_acls() {
    local account="$1"
    local web_acls=""
    deep_cleanup_list_wafv2_web_acls "$account" web_acls

    while IFS=$'\x1f' read -r acl_name _ _ acl_arn; do
        [ -z "$acl_name" ] && continue
        [ -z "$acl_arn" ] && continue

        # list-resources-for-web-acl silently defaults to APPLICATION_LOAD_BALANCER,
        # so enumerate all supported REGIONAL association resource types.
        local resource_type
        for resource_type in \
            APPLICATION_LOAD_BALANCER \
            API_GATEWAY \
            APPSYNC \
            COGNITO_USER_POOL \
            APP_RUNNER_SERVICE \
            VERIFIED_ACCESS_INSTANCE \
            AMPLIFY
        do
            local resource_arns
            deep_cleanup_enumerate "$account" "resources associated with WAFv2 web ACL $acl_name ($resource_type)" resource_arns \
                aws wafv2 list-resources-for-web-acl \
                --web-acl-arn "$acl_arn" --resource-type "$resource_type" \
                --query 'ResourceArns' --output text
            if [ "$DEEP_CLEANUP_LAST_ENUMERATION_SUCCEEDED" -eq 0 ]; then
                continue
            fi

            local resource_arn
            for resource_arn in $resource_arns; do
                [ -z "$resource_arn" ] && continue
                [ "$resource_arn" = "None" ] && continue
                echo "  Disassociating WAFv2 web ACL $acl_name from $resource_arn..."
                deep_cleanup_delete_supporting "$account" "disassociate WAFv2 web ACL $acl_name from $resource_arn" \
                    aws wafv2 disassociate-web-acl --resource-arn "$resource_arn"
            done
        done
    done <<< "$web_acls"

    return 0
}

DEEP_CLEANUP_WAFV2_WEB_ACL_DELETE_COUNT=0
deep_cleanup_delete_wafv2_web_acls() {
    local account="$1"
    DEEP_CLEANUP_WAFV2_WEB_ACL_DELETE_COUNT=0

    local web_acls=""
    deep_cleanup_list_wafv2_web_acls "$account" web_acls

    while IFS=$'\x1f' read -r acl_name acl_id lock_token _; do
        [ -z "$acl_name" ] && continue
        DEEP_CLEANUP_WAFV2_WEB_ACL_DELETE_COUNT=$((DEEP_CLEANUP_WAFV2_WEB_ACL_DELETE_COUNT + 1))
        echo "  Cleaning WAFv2 web ACL $acl_name..."

        if [ -z "$lock_token" ]; then
            DEEP_CLEANUP_FAILED_COUNT=$((${DEEP_CLEANUP_FAILED_COUNT:-0} + 1))
            echo "  ERROR: Cannot delete WAFv2 web ACL $acl_name: list-web-acls returned no LockToken; the ACL may be updating" >&2
            continue
        fi

        deep_cleanup_delete_resource "$account" "delete WAFv2 web ACL $acl_name" \
            aws wafv2 delete-web-acl \
            --name "$acl_name" --scope REGIONAL --id "$acl_id" --lock-token "$lock_token"
    done <<< "$web_acls"

    return 0
}

# ── deep_cleanup_account: scan AWS for orphaned resources ────────────
# Args: account_profile (e.g., "carina-test-000")
deep_cleanup_account() {
    local account="$1"
    DEEP_CLEANUP_ENUMERATION_FAILURE_COUNT=0
    DEEP_CLEANUP_LAST_ENUMERATION_SUCCEEDED=1
    DEEP_CLEANUP_DELETED_COUNT=0
    DEEP_CLEANUP_LAST_DELETE_SUCCEEDED=0
    DEEP_CLEANUP_FAILED_COUNT=0
    DEEP_CLEANUP_SUPPORTING_FAILURE_COUNT=0
    # Native waiter budgets: 15s * 40 = 600s (ELBv2), 20s * 10 = 200s (ENI).
    # Wall caps of 720s and 300s leave 120s/100s of API-call margin as hang backstops.
    local elbv2_wait_timeout_seconds=720
    local eni_wait_timeout_seconds=300

    # Pre-cleanup: disassociate WAFv2 web ACLs before deleting associated resources.
    deep_cleanup_disassociate_wafv2_web_acls "$account"

    # 0. Delete orphaned ELBv2 load balancers and target groups before VPC cleanup
    # Deleting a load balancer also deletes its listeners, so no listener pass is needed.
    # Wait for that deletion because target groups remain undeletable while listeners reference them.
    local load_balancers
    deep_cleanup_enumerate "$account" "ELBv2 load balancers" load_balancers \
        aws elbv2 describe-load-balancers \
        --query 'LoadBalancers[?contains(LoadBalancerName, `acceptance-test`) || contains(LoadBalancerName, `carina-acc`)].LoadBalancerArn' --output text
    local deleted_load_balancers=()
    local lb_arn
    for lb_arn in $load_balancers; do
        [ -z "$lb_arn" ] && continue
        echo "  Cleaning ELBv2 load balancer $lb_arn..."
        deep_cleanup_delete_resource "$account" "delete ELBv2 load balancer $lb_arn" \
            aws elbv2 delete-load-balancer --load-balancer-arn "$lb_arn"
        if [ "$DEEP_CLEANUP_LAST_DELETE_SUCCEEDED" -eq 1 ]; then
            deleted_load_balancers+=("$lb_arn")
        fi
    done
    if [ "${#deleted_load_balancers[@]}" -gt 0 ]; then
        echo "  Waiting for deleted ELBv2 load balancers..."
        local lb_wait_offset=0
        while [ "$lb_wait_offset" -lt "${#deleted_load_balancers[@]}" ]; do
            # describe-load-balancers accepts at most 20 ARNs per waiter poll.
            local lb_wait_batch=("${deleted_load_balancers[@]:lb_wait_offset:20}")
            deep_cleanup_delete_supporting "$account" "wait for deleted ELBv2 load balancers" \
                deep_cleanup_with_timeout "$elbv2_wait_timeout_seconds" \
                aws elbv2 wait load-balancers-deleted \
                --load-balancer-arns "${lb_wait_batch[@]}"
            lb_wait_offset=$((lb_wait_offset + 20))
        done
    fi

    local target_groups
    deep_cleanup_enumerate "$account" "ELBv2 target groups" target_groups \
        aws elbv2 describe-target-groups \
        --query 'TargetGroups[?contains(TargetGroupName, `acceptance-test`) || contains(TargetGroupName, `carina-acc`)].TargetGroupArn' --output text
    for tg_arn in $target_groups; do
        [ -z "$tg_arn" ] && continue
        echo "  Cleaning ELBv2 target group $tg_arn..."
        deep_cleanup_delete_resource "$account" "delete ELBv2 target group $tg_arn" \
            aws elbv2 delete-target-group --target-group-arn "$tg_arn"
    done

    # 1. Delete non-default VPCs and all dependencies
    local vpcs
    deep_cleanup_enumerate "$account" "non-default VPCs" vpcs \
        aws ec2 describe-vpcs \
        --filters "Name=isDefault,Values=false" \
        --query 'Vpcs[*].VpcId' --output text

    for vpc_id in $vpcs; do
        [ -z "$vpc_id" ] && continue
        echo "  Cleaning VPC $vpc_id..."

        # Delete NAT Gateways (must be first, takes time)
        local nat_gws
        deep_cleanup_enumerate "$account" "NAT gateways for VPC $vpc_id" nat_gws \
            aws ec2 describe-nat-gateways \
            --filter "Name=vpc-id,Values=$vpc_id" "Name=state,Values=available,pending" \
            --query 'NatGateways[*].NatGatewayId' --output text
        for nat_id in $nat_gws; do
            [ -z "$nat_id" ] && continue
            deep_cleanup_delete_supporting "$account" "delete NAT gateway $nat_id" \
                aws ec2 delete-nat-gateway --nat-gateway-id "$nat_id"
        done
        # Wait for NAT GWs to delete if any were found
        if [ -n "$nat_gws" ] && [ "$nat_gws" != "None" ]; then
            sleep 30
        fi

        # Delete flow logs before their IAM delivery roles are swept later
        local flow_logs
        deep_cleanup_enumerate "$account" "flow logs for VPC $vpc_id" flow_logs \
            aws ec2 describe-flow-logs \
            --filter "Name=resource-id,Values=$vpc_id" \
            --query 'FlowLogs[*].FlowLogId' --output text
        local flow_log_id
        for flow_log_id in $flow_logs; do
            [ -z "$flow_log_id" ] && continue
            [ "$flow_log_id" = "None" ] && continue
            deep_cleanup_delete_supporting "$account" "delete flow log $flow_log_id" \
                aws ec2 delete-flow-logs --flow-log-ids "$flow_log_id"
        done

        # Delete VPC endpoints
        local endpoints
        deep_cleanup_enumerate "$account" "VPC endpoints for VPC $vpc_id" endpoints \
            aws ec2 describe-vpc-endpoints \
            --filters "Name=vpc-id,Values=$vpc_id" \
            --query 'VpcEndpoints[*].VpcEndpointId' --output text
        for ep_id in $endpoints; do
            [ -z "$ep_id" ] && continue
            deep_cleanup_delete_supporting "$account" "delete VPC endpoint $ep_id" \
                aws ec2 delete-vpc-endpoints --vpc-endpoint-ids "$ep_id"
        done

        # Detach and delete internet gateways
        local igws
        deep_cleanup_enumerate "$account" "internet gateways for VPC $vpc_id" igws \
            aws ec2 describe-internet-gateways \
            --filters "Name=attachment.vpc-id,Values=$vpc_id" \
            --query 'InternetGateways[*].InternetGatewayId' --output text
        for igw_id in $igws; do
            [ -z "$igw_id" ] && continue
            deep_cleanup_delete_supporting "$account" "detach internet gateway $igw_id from VPC $vpc_id" \
                aws ec2 detach-internet-gateway --internet-gateway-id "$igw_id" --vpc-id "$vpc_id"
            deep_cleanup_delete_supporting "$account" "delete internet gateway $igw_id" \
                aws ec2 delete-internet-gateway --internet-gateway-id "$igw_id"
        done

        # Delete egress-only internet gateways attached to this VPC
        local egress_only_internet_gateways
        deep_cleanup_enumerate "$account" "egress-only internet gateways for VPC $vpc_id" egress_only_internet_gateways \
            aws ec2 describe-egress-only-internet-gateways \
            --query "EgressOnlyInternetGateways[?Attachments[?VpcId=='$vpc_id']].EgressOnlyInternetGatewayId" \
            --output text
        local egress_only_internet_gateway_id
        for egress_only_internet_gateway_id in $egress_only_internet_gateways; do
            [ -z "$egress_only_internet_gateway_id" ] && continue
            [ "$egress_only_internet_gateway_id" = "None" ] && continue
            deep_cleanup_delete_supporting "$account" "delete egress-only internet gateway $egress_only_internet_gateway_id" \
                aws ec2 delete-egress-only-internet-gateway \
                --egress-only-internet-gateway-id "$egress_only_internet_gateway_id"
        done

        # Detach and delete VPN gateways
        local vpn_gws
        deep_cleanup_enumerate "$account" "VPN gateways for VPC $vpc_id" vpn_gws \
            aws ec2 describe-vpn-gateways \
            --filters "Name=attachment.vpc-id,Values=$vpc_id" \
            --query 'VpnGateways[*].VpnGatewayId' --output text
        for vpn_id in $vpn_gws; do
            [ -z "$vpn_id" ] && continue
            deep_cleanup_delete_supporting "$account" "detach VPN gateway $vpn_id from VPC $vpc_id" \
                aws ec2 detach-vpn-gateway --vpn-gateway-id "$vpn_id" --vpc-id "$vpc_id"
            deep_cleanup_delete_supporting "$account" "delete VPN gateway $vpn_id" \
                aws ec2 delete-vpn-gateway --vpn-gateway-id "$vpn_id"
        done

        # Wait a bounded period for AWS-side releases, then delete only detached ENIs.
        # A still in-use ENI may belong to a live resource and is never forced off.
        local in_use_network_interfaces
        deep_cleanup_enumerate "$account" "in-use network interfaces for VPC $vpc_id" in_use_network_interfaces \
            aws ec2 describe-network-interfaces \
            --filters "Name=vpc-id,Values=$vpc_id" "Name=status,Values=in-use" \
            --query 'NetworkInterfaces[*].NetworkInterfaceId' --output text
        local in_use_network_interface_ids=()
        local eni_id
        for eni_id in $in_use_network_interfaces; do
            [ -z "$eni_id" ] && continue
            [ "$eni_id" = "None" ] && continue
            in_use_network_interface_ids+=("$eni_id")
        done
        if [ "${#in_use_network_interface_ids[@]}" -gt 0 ]; then
            echo "  Waiting for network interfaces in VPC $vpc_id to become available..."
            # Unlike ELBv2, wait per interface so one vanished ENI cannot poison a batch;
            # each remaining ENI must still reach its own availability check.
            for eni_id in "${in_use_network_interface_ids[@]}"; do
                deep_cleanup_delete_supporting "$account" "wait for network interface $eni_id to become available" \
                    deep_cleanup_wait_for_network_interface "$eni_wait_timeout_seconds" \
                    aws ec2 wait network-interface-available \
                    --network-interface-ids "$eni_id"
            done
        fi

        local available_network_interfaces
        deep_cleanup_enumerate "$account" "available network interfaces for VPC $vpc_id" available_network_interfaces \
            aws ec2 describe-network-interfaces \
            --filters "Name=vpc-id,Values=$vpc_id" "Name=status,Values=available" \
            --query 'NetworkInterfaces[*].NetworkInterfaceId' --output text
        for eni_id in $available_network_interfaces; do
            [ -z "$eni_id" ] && continue
            [ "$eni_id" = "None" ] && continue
            echo "  Cleaning network interface $eni_id..."
            deep_cleanup_delete_supporting "$account" "delete network interface $eni_id" \
                aws ec2 delete-network-interface --network-interface-id "$eni_id"
        done

        # Delete subnets
        local subnets
        deep_cleanup_enumerate "$account" "subnets for VPC $vpc_id" subnets \
            aws ec2 describe-subnets \
            --filters "Name=vpc-id,Values=$vpc_id" \
            --query 'Subnets[*].SubnetId' --output text
        for subnet_id in $subnets; do
            [ -z "$subnet_id" ] && continue
            deep_cleanup_delete_supporting "$account" "delete subnet $subnet_id" \
                aws ec2 delete-subnet --subnet-id "$subnet_id"
        done

        # Delete non-main route tables
        local rts
        # Associations are unordered; inspect every slot so #402 cannot select the main table.
        # Do not use [?!Associations[?Main==`true`]]: truthiness collapses that projection
        # so it matches nothing. Keep not_null(Associations, `[]`), since without it
        # length() type-errors on a missing Associations key and fails the enumeration.
        deep_cleanup_enumerate "$account" "non-main route tables for VPC $vpc_id" rts \
            aws ec2 describe-route-tables \
            --filters "Name=vpc-id,Values=$vpc_id" \
            --query 'RouteTables[?length(not_null(Associations, `[]`)[?Main==`true`])==`0`].RouteTableId' --output text
        local rt_id
        # Disassociate every non-main association before deleting any route table.
        for rt_id in $rts; do
            [ -z "$rt_id" ] && continue
            [ "$rt_id" = "None" ] && continue

            local rt_associations
            deep_cleanup_enumerate "$account" "non-main associations for route table $rt_id" rt_associations \
                aws ec2 describe-route-tables --route-table-ids "$rt_id" \
                --query 'RouteTables[*].Associations[?Main!=`true`].RouteTableAssociationId' --output text
            local rt_association_id
            for rt_association_id in $rt_associations; do
                [ -z "$rt_association_id" ] && continue
                [ "$rt_association_id" = "None" ] && continue
                echo "  Cleaning route table association $rt_association_id from route table $rt_id..."
                deep_cleanup_delete_supporting "$account" "disassociate route table association $rt_association_id from route table $rt_id" \
                    aws ec2 disassociate-route-table --association-id "$rt_association_id"
            done
        done

        # Delete candidate route tables only after the complete disassociation pass.
        for rt_id in $rts; do
            [ -z "$rt_id" ] && continue
            [ "$rt_id" = "None" ] && continue
            deep_cleanup_delete_supporting "$account" "delete route table $rt_id" \
                aws ec2 delete-route-table --route-table-id "$rt_id"
        done

        # Delete non-default security groups
        local sgs
        deep_cleanup_enumerate "$account" "non-default security groups for VPC $vpc_id" sgs \
            aws ec2 describe-security-groups \
            --filters "Name=vpc-id,Values=$vpc_id" \
            --query 'SecurityGroups[?GroupName!=`default`].GroupId' --output text
        local sg_id
        # Revoke every candidate group's rules before deleting any group. This
        # breaks references in either direction between candidate groups.
        for sg_id in $sgs; do
            [ -z "$sg_id" ] && continue
            [ "$sg_id" = "None" ] && continue

            local sg_rules
            deep_cleanup_enumerate "$account" "rules for security group $sg_id" sg_rules \
                aws ec2 describe-security-group-rules \
                --filters "Name=group-id,Values=$sg_id" \
                --query 'SecurityGroupRules[].[SecurityGroupRuleId,IsEgress]' --output text
            local sg_rule_id
            local is_egress
            while IFS=$'\t' read -r sg_rule_id is_egress; do
                [ -z "$sg_rule_id" ] && continue
                [ "$sg_rule_id" = "None" ] && continue

                case "$is_egress" in
                    False)
                        echo "  Cleaning ingress rule $sg_rule_id from security group $sg_id..."
                        deep_cleanup_delete_supporting "$account" "revoke ingress rule $sg_rule_id from security group $sg_id" \
                            aws ec2 revoke-security-group-ingress \
                            --group-id "$sg_id" --security-group-rule-ids "$sg_rule_id"
                        ;;
                    True)
                        echo "  Cleaning egress rule $sg_rule_id from security group $sg_id..."
                        deep_cleanup_delete_supporting "$account" "revoke egress rule $sg_rule_id from security group $sg_id" \
                            aws ec2 revoke-security-group-egress \
                            --group-id "$sg_id" --security-group-rule-ids "$sg_rule_id"
                        ;;
                    *)
                        DEEP_CLEANUP_ENUMERATION_FAILURE_COUNT=$((DEEP_CLEANUP_ENUMERATION_FAILURE_COUNT + 1))
                        echo "  ERROR: Failed to classify security-group rule $sg_rule_id for $sg_id: unexpected IsEgress value '$is_egress'" >&2
                        ;;
                esac
            done <<< "$sg_rules"
        done

        # Delete candidate groups only after the complete revoke pass.
        for sg_id in $sgs; do
            [ -z "$sg_id" ] && continue
            [ "$sg_id" = "None" ] && continue
            deep_cleanup_delete_supporting "$account" "delete security group $sg_id" \
                aws ec2 delete-security-group --group-id "$sg_id"
        done

        # Delete VPC peering connections
        local peerings
        deep_cleanup_enumerate "$account" "peering connections for VPC $vpc_id" peerings \
            aws ec2 describe-vpc-peering-connections \
            --filters "Name=requester-vpc-info.vpc-id,Values=$vpc_id" \
            --query 'VpcPeeringConnections[*].VpcPeeringConnectionId' --output text
        for peer_id in $peerings; do
            [ -z "$peer_id" ] && continue
            deep_cleanup_delete_supporting "$account" "delete VPC peering connection $peer_id" \
                aws ec2 delete-vpc-peering-connection --vpc-peering-connection-id "$peer_id"
        done

        # Finally delete the VPC
        deep_cleanup_delete_resource "$account" "delete VPC $vpc_id" \
            aws ec2 delete-vpc --vpc-id "$vpc_id"
    done

    # 2. Delete orphaned S3 buckets matching carina-acc-test*
    local buckets
    deep_cleanup_enumerate "$account" "S3 buckets" buckets \
        aws s3api list-buckets \
        --query 'Buckets[?starts_with(Name, `carina-acc-test`)].Name' --output text
    for bucket in $buckets; do
        [ -z "$bucket" ] && continue
        echo "  Cleaning S3 bucket $bucket..."
        deep_cleanup_delete_resource "$account" "delete S3 bucket $bucket" \
            aws s3 rb "s3://$bucket" --force
    done

    # 3. Delete orphaned DynamoDB tables
    local ddb_tables
    deep_cleanup_enumerate "$account" "DynamoDB tables" ddb_tables \
        aws dynamodb list-tables \
        --query 'TableNames[?starts_with(@, `carina-acc-test`) || starts_with(@, `acceptance-test`)]' --output text
    for table in $ddb_tables; do
        [ -z "$table" ] && continue
        echo "  Cleaning DynamoDB table $table..."
        deep_cleanup_delete_resource "$account" "delete DynamoDB table $table" \
            aws dynamodb delete-table --table-name "$table"
    done

    # 4. Delete orphaned ECS clusters
    local ecs_clusters
    deep_cleanup_enumerate "$account" "ECS clusters" ecs_clusters \
        aws ecs list-clusters \
        --query 'clusterArns[?contains(@, `acceptance-test`) || contains(@, `carina-acc`)]' --output text
    for cluster_arn in $ecs_clusters; do
        [ -z "$cluster_arn" ] && continue
        echo "  Cleaning ECS cluster $cluster_arn..."
        deep_cleanup_delete_resource "$account" "delete ECS cluster $cluster_arn" \
            aws ecs delete-cluster --cluster "$cluster_arn"
    done

    # 5. Delete orphaned Route 53 hosted zones
    local hosted_zones
    deep_cleanup_enumerate "$account" "Route 53 hosted zones" hosted_zones \
        aws route53 list-hosted-zones \
        --query 'HostedZones[?contains(Name, `acceptance-test`) || contains(Name, `carina-acc`)].Id' --output text
    for zone_id in $hosted_zones; do
        [ -z "$zone_id" ] && continue
        echo "  Cleaning Route 53 hosted zone $zone_id..."
        deep_cleanup_delete_resource "$account" "delete Route 53 hosted zone $zone_id" \
            aws route53 delete-hosted-zone --id "$zone_id"
    done

    # 6. Delete orphaned regional WAFv2 web ACLs
    deep_cleanup_delete_wafv2_web_acls "$account"

    # 7. Delete orphaned IAM OIDC providers tagged by acceptance tests
    local oidc_providers
    deep_cleanup_enumerate "$account" "IAM OIDC providers" oidc_providers \
        aws iam list-open-id-connect-providers \
        --query 'OpenIDConnectProviderList[*].Arn' --output text
    for provider_arn in $oidc_providers; do
        [ -z "$provider_arn" ] && continue
        local provider_tags
        deep_cleanup_enumerate "$account" "acceptance-test tags for IAM OIDC provider $provider_arn" provider_tags \
            aws iam list-open-id-connect-provider-tags \
            --open-id-connect-provider-arn "$provider_arn" \
            --query 'Tags[?Key==`Environment` && Value==`acceptance-test`].Key' --output text
        if [ -z "$provider_tags" ] || [ "$provider_tags" = "None" ]; then
            continue
        fi
        echo "  Cleaning IAM OIDC provider $provider_arn..."
        deep_cleanup_delete_resource "$account" "delete IAM OIDC provider $provider_arn" \
            aws iam delete-open-id-connect-provider --open-id-connect-provider-arn "$provider_arn"
    done

    # 8. Delete orphaned IAM roles
    local roles
    deep_cleanup_enumerate "$account" "IAM roles" roles \
        aws iam list-roles \
        --query 'Roles[?contains(RoleName, `acceptance-test`) || contains(RoleName, `carina-acc`)].RoleName' --output text
    for role in $roles; do
        [ -z "$role" ] && continue
        echo "  Cleaning IAM role $role..."
        # Detach all policies first
        local policies
        deep_cleanup_enumerate "$account" "attached policies for IAM role $role" policies \
            aws iam list-attached-role-policies --role-name "$role" \
            --query 'AttachedPolicies[*].PolicyArn' --output text
        for policy_arn in $policies; do
            [ -z "$policy_arn" ] && continue
            deep_cleanup_delete_supporting "$account" "detach policy $policy_arn from IAM role $role" \
                aws iam detach-role-policy --role-name "$role" --policy-arn "$policy_arn"
        done
        local inline_policies
        deep_cleanup_enumerate "$account" "inline policies for IAM role $role" inline_policies \
            aws iam list-role-policies --role-name "$role" \
            --query 'PolicyNames[*]' --output text
        for policy_name in $inline_policies; do
            [ -z "$policy_name" ] && continue
            deep_cleanup_delete_supporting "$account" "delete inline policy $policy_name from IAM role $role" \
                aws iam delete-role-policy --role-name "$role" --policy-name "$policy_name"
        done
        deep_cleanup_delete_resource "$account" "delete IAM role $role" \
            aws iam delete-role --role-name "$role"
    done

    # 9. Delete orphaned IAM customer managed policies
    local iam_policies
    deep_cleanup_enumerate "$account" "IAM customer managed policies" iam_policies \
        aws iam list-policies --scope Local \
        --query 'Policies[?contains(PolicyName, `acceptance-test`) || contains(PolicyName, `carina-acc`)].Arn' --output text
    for policy_arn in $iam_policies; do
        [ -z "$policy_arn" ] && continue
        echo "  Cleaning IAM policy $policy_arn..."

        local attached_roles
        deep_cleanup_enumerate "$account" "roles attached to IAM policy $policy_arn" attached_roles \
            aws iam list-entities-for-policy --policy-arn "$policy_arn" \
            --query 'PolicyRoles[*].RoleName' --output text
        for role_name in $attached_roles; do
            [ -z "$role_name" ] && continue
            deep_cleanup_delete_supporting "$account" "detach IAM policy $policy_arn from role $role_name" \
                aws iam detach-role-policy --role-name "$role_name" --policy-arn "$policy_arn"
        done

        local attached_users
        deep_cleanup_enumerate "$account" "users attached to IAM policy $policy_arn" attached_users \
            aws iam list-entities-for-policy --policy-arn "$policy_arn" \
            --query 'PolicyUsers[*].UserName' --output text
        for user_name in $attached_users; do
            [ -z "$user_name" ] && continue
            deep_cleanup_delete_supporting "$account" "detach IAM policy $policy_arn from user $user_name" \
                aws iam detach-user-policy --user-name "$user_name" --policy-arn "$policy_arn"
        done

        local attached_groups
        deep_cleanup_enumerate "$account" "groups attached to IAM policy $policy_arn" attached_groups \
            aws iam list-entities-for-policy --policy-arn "$policy_arn" \
            --query 'PolicyGroups[*].GroupName' --output text
        for group_name in $attached_groups; do
            [ -z "$group_name" ] && continue
            deep_cleanup_delete_supporting "$account" "detach IAM policy $policy_arn from group $group_name" \
                aws iam detach-group-policy --group-name "$group_name" --policy-arn "$policy_arn"
        done

        local policy_versions
        deep_cleanup_enumerate "$account" "non-default versions of IAM policy $policy_arn" policy_versions \
            aws iam list-policy-versions --policy-arn "$policy_arn" \
            --query 'Versions[?IsDefaultVersion==`false`].VersionId' --output text
        for version_id in $policy_versions; do
            [ -z "$version_id" ] && continue
            deep_cleanup_delete_supporting "$account" "delete version $version_id of IAM policy $policy_arn" \
                aws iam delete-policy-version --policy-arn "$policy_arn" --version-id "$version_id"
        done

        deep_cleanup_delete_resource "$account" "delete IAM policy $policy_arn" \
            aws iam delete-policy --policy-arn "$policy_arn"
    done

    # 10. Delete orphaned log groups
    local log_groups
    deep_cleanup_enumerate "$account" "CloudWatch log groups" log_groups \
        aws logs describe-log-groups \
        --log-group-name-prefix "/acceptance-test/" \
        --query 'logGroups[*].logGroupName' --output text
    for lg in $log_groups; do
        [ -z "$lg" ] && continue
        echo "  Cleaning log group $lg..."
        deep_cleanup_delete_resource "$account" "delete log group $lg" \
            aws logs delete-log-group --log-group-name "$lg"
    done

    # 11. Delete transit gateways
    local tgws
    deep_cleanup_enumerate "$account" "transit gateways" tgws \
        aws ec2 describe-transit-gateways \
        --filters "Name=state,Values=available,pending" \
        --query 'TransitGateways[*].TransitGatewayId' --output text
    for tgw_id in $tgws; do
        [ -z "$tgw_id" ] && continue
        echo "  Cleaning transit gateway $tgw_id..."
        # Delete attachments first
        local attachments
        deep_cleanup_enumerate "$account" "attachments for transit gateway $tgw_id" attachments \
            aws ec2 describe-transit-gateway-attachments \
            --filters "Name=transit-gateway-id,Values=$tgw_id" "Name=state,Values=available" \
            --query 'TransitGatewayAttachments[*].TransitGatewayAttachmentId' --output text
        for att_id in $attachments; do
            [ -z "$att_id" ] && continue
            deep_cleanup_delete_supporting "$account" "delete transit gateway attachment $att_id" \
                aws ec2 delete-transit-gateway-vpc-attachment --transit-gateway-attachment-id "$att_id"
        done
        deep_cleanup_delete_resource "$account" "delete transit gateway $tgw_id" \
            aws ec2 delete-transit-gateway --transit-gateway-id "$tgw_id"
    done

    # 12. Delete IPAMs and their private-scope pools
    local ipams
    deep_cleanup_enumerate "$account" "IPAM cleanup candidates" ipams \
        aws ec2 describe-ipams \
        --query "Ipams[?contains([\`create-complete\`, \`create-failed\`, \`modify-complete\`, \`modify-failed\`, \`delete-failed\`, \`isolate-complete\`], State)].IpamId" --output text
    local ipam_id
    for ipam_id in $ipams; do
        [ -z "$ipam_id" ] && continue
        [ "$ipam_id" = "None" ] && continue
        echo "  Cleaning IPAM $ipam_id..."
        # Cascade deletes private-scope pools, their provisioned CIDRs, and allocations.
        # It intentionally fails visibly if the IPAM has a public-scope pool.
        deep_cleanup_delete_resource "$account" "delete IPAM $ipam_id" \
            aws ec2 delete-ipam --ipam-id "$ipam_id" --cascade
    done

    # 13. Release Elastic IPs not associated with anything
    local eips
    deep_cleanup_enumerate "$account" "unassociated Elastic IPs" eips \
        aws ec2 describe-addresses \
        --query 'Addresses[?AssociationId==null].AllocationId' --output text
    for eip_id in $eips; do
        [ -z "$eip_id" ] && continue
        echo "  Releasing Elastic IP $eip_id..."
        deep_cleanup_delete_resource "$account" "release Elastic IP $eip_id" \
            aws ec2 release-address --allocation-id "$eip_id"
    done

    local found=$((DEEP_CLEANUP_DELETED_COUNT + DEEP_CLEANUP_FAILED_COUNT))
    local summary="$account: $found found, $DEEP_CLEANUP_DELETED_COUNT deleted, $DEEP_CLEANUP_FAILED_COUNT failed"
    if [ "$DEEP_CLEANUP_ENUMERATION_FAILURE_COUNT" -gt 0 ]; then
        local enumeration_error_label="errors"
        if [ "$DEEP_CLEANUP_ENUMERATION_FAILURE_COUNT" -eq 1 ]; then
            enumeration_error_label="error"
        fi
        summary="$summary; $DEEP_CLEANUP_ENUMERATION_FAILURE_COUNT enumeration $enumeration_error_label"
    fi
    if [ "$DEEP_CLEANUP_SUPPORTING_FAILURE_COUNT" -gt 0 ]; then
        local supporting_error_label="errors"
        if [ "$DEEP_CLEANUP_SUPPORTING_FAILURE_COUNT" -eq 1 ]; then
            supporting_error_label="error"
        fi
        summary="$summary; $DEEP_CLEANUP_SUPPORTING_FAILURE_COUNT supporting cleanup $supporting_error_label"
    fi

    if [ "$DEEP_CLEANUP_FAILED_COUNT" -gt 0 ]; then
        echo "  ERROR: $summary" >&2
        return 1
    fi
    if [ "$DEEP_CLEANUP_ENUMERATION_FAILURE_COUNT" -gt 0 ] || \
        [ "$DEEP_CLEANUP_SUPPORTING_FAILURE_COUNT" -gt 0 ]; then
        echo "  WARNING: $summary" >&2
        return 0
    fi

    echo "  $summary"
    return 0
}

# Build provider binary
echo "Building provider binary..."
cargo build -p carina-provider-awscc --target wasm32-wasip2 --release --quiet 2>/dev/null || cargo build -p carina-provider-awscc --target wasm32-wasip2 --release

# Optimize WASM with vela (reduces binary size and load time)
if command -v vela &>/dev/null; then
    echo "Optimizing WASM with vela..."
    vela optimize "$PROJECT_ROOT/target/wasm32-wasip2/release/carina-provider-awscc.wasm" -o "$PROJECT_ROOT/target/wasm32-wasip2/release/carina-provider-awscc.wasm"
else
    echo "Note: vela not found, skipping WASM optimization. Install: cargo install --git https://github.com/mizzy/vela vela-cli"
fi
echo ""

# CARINA_BIN can be set externally (e.g., when running from the monorepo).
# If not set, look for it in common locations.
if [ -z "${CARINA_BIN:-}" ]; then
    # Prefer release build (WASM JIT is much faster with release)
    if [ -f "$PROJECT_ROOT/../carina/target/release/carina" ]; then
        CARINA_BIN="$PROJECT_ROOT/../carina/target/release/carina"
    elif [ -f "$PROJECT_ROOT/../carina/target/debug/carina" ]; then
        CARINA_BIN="$PROJECT_ROOT/../carina/target/debug/carina"
    elif command -v carina &>/dev/null; then
        CARINA_BIN="$(command -v carina)"
    else
        echo "ERROR: carina binary not found. Set CARINA_BIN or install carina."
        exit 1
    fi
fi

if [ ! -f "$CARINA_BIN" ]; then
    echo "ERROR: carina binary not found at $CARINA_BIN"
    exit 1
fi

# ── Provider source injection ────────────────────────────────────────
# Provider blocks require source and version attributes.
# Inject them dynamically so .crn files don't need to hard-code binary paths.
AWSCC_PROVIDER_BIN="${AWSCC_PROVIDER_BIN:-$PROJECT_ROOT/target/wasm32-wasip2/release/carina-provider-awscc.wasm}"
AWS_PROVIDER_BIN="${AWS_PROVIDER_BIN:-$PROJECT_ROOT/target/wasm32-wasip2/release/carina-provider-aws.wasm}"

# Read provider version from workspace Cargo.toml to avoid hardcoding
PROVIDER_VERSION=$(grep '^version = ' "$PROJECT_ROOT/Cargo.toml" | head -1 | sed 's/version = "\(.*\)"/\1/')

# Validate that provider binaries are WASM components, not native binaries.
# Native binaries are no longer supported and will cause cryptic linker errors.
for bin_var in AWSCC_PROVIDER_BIN AWS_PROVIDER_BIN; do
    bin_path="${!bin_var}"
    if [ -n "$bin_path" ] && [ -f "$bin_path" ] && [[ "$bin_path" != *.wasm ]]; then
        echo "ERROR: $bin_var points to a non-WASM binary: $bin_path"
        echo "       Provider binaries must be .wasm components."
        echo "       Unset $bin_var to use the default WASM build, or point it to a .wasm file."
        exit 1
    fi
done

# inject_provider_source: Create a temp copy of a .crn file with source/version
# injected into provider blocks. Prints the temp file path.
# Args: original_crn_file
inject_provider_source() {
    local original="$1"
    local tmp_file
    tmp_file=$(mktemp).crn

    sed \
        -e '/^provider awscc {/a\
  source = "file://'"$AWSCC_PROVIDER_BIN"'"\
  version = "'"$PROVIDER_VERSION"'"' \
        -e '/^provider aws {/a\
  source = "file://'"$AWS_PROVIDER_BIN"'"\
  version = "'"$PROVIDER_VERSION"'"' \
        "$original" > "$tmp_file"

    echo "$tmp_file"
}

# Find test files
# matches_any_filter: returns 0 if rel_path matches any filter, or if no filters given
matches_any_filter() {
    local rel_path="$1"
    if [ ${#FILTERS[@]} -eq 0 ]; then
        return 0
    fi
    for f in "${FILTERS[@]}"; do
        if [[ "$rel_path" == *"$f"* ]]; then
            return 0
        fi
    done
    return 1
}

# is_slow_test: returns 0 if the test has a .slow marker file
is_slow_test() {
    local crn_file="$1"
    local slow_file="${crn_file%.crn}.slow"
    [ -f "$slow_file" ]
}

TESTS=()
SKIPPED_SLOW=()
while IFS= read -r -d '' file; do
    REL_PATH="${file#$SCRIPT_DIR/}"
    if ! matches_any_filter "$REL_PATH"; then
        continue
    fi
    # Skip .crn files in directories with a custom run.sh (multi-step tests)
    DIR_OF_FILE="$(dirname "$file")"
    if [ -f "$DIR_OF_FILE/run.sh" ]; then
        continue
    fi
    # Skip .crn files inside modules/ directories (they are imported by other tests, not standalone)
    if echo "$REL_PATH" | grep -q '/modules/'; then
        continue
    fi
    # Skip slow tests unless --include-slow is set or a filter is provided
    if is_slow_test "$file" && [ $INCLUDE_SLOW -eq 0 ] && [ ${#FILTERS[@]} -eq 0 ]; then
        SKIPPED_SLOW+=("$REL_PATH")
        continue
    fi
    TESTS+=("$file")
done < <(find "$SCRIPT_DIR" -name "*.crn" -print0 | sort -z)

if [ ${#SKIPPED_SLOW[@]} -gt 0 ]; then
    echo "Skipping ${#SKIPPED_SLOW[@]} slow test(s) (use --include-slow to include):"
    for SLOW_TEST in "${SKIPPED_SLOW[@]}"; do
        echo "  $SLOW_TEST"
    done
    echo ""
fi

if [ ${#TESTS[@]} -eq 0 ]; then
    if [ ${#FILTERS[@]} -gt 0 ]; then
        echo "No test files found matching: ${FILTERS[*]}"
    else
        echo "No test files found"
    fi
    exit 0
fi

# ── deep-cleanup: scan and remove orphaned AWS resources ─────────────
if [ "$COMMAND" = "deep-cleanup" ]; then
    echo "Running deep cleanup to remove orphaned AWS resources across $NUM_ACCOUNTS accounts"
    echo ""

    # Pre-authenticate accounts
    echo "Pre-authenticating AWS accounts..."
    for SLOT in $(seq 0 $((NUM_ACCOUNTS - 1))); do
        ACCOUNT="${ACCOUNTS[$SLOT]}"
        echo "  Authenticating $ACCOUNT..."
        if ! with_account_creds "$ACCOUNT" aws sts get-caller-identity >/dev/null 2>&1; then
            echo "  WARNING: Failed to pre-authenticate $ACCOUNT"
        fi
    done
    echo "Pre-authentication complete."
    echo ""

    PIDS=()
    for SLOT in $(seq 0 $((NUM_ACCOUNTS - 1))); do
        ACCOUNT="${ACCOUNTS[$SLOT]}"
        (
            echo "── $ACCOUNT ──"
            deep_cleanup_account "$ACCOUNT"
        ) &
        PIDS+=($!)
        echo "  [$ACCOUNT] started (PID $!)"
    done

    echo ""
    echo "Waiting for all accounts to finish deep cleanup..."
    echo ""

    OVERALL_EXIT=0
    for PID in ${PIDS[@]+"${PIDS[@]}"}; do
        wait "$PID" || OVERALL_EXIT=1
    done

    echo "════════════════════════════════════════"
    echo "Deep cleanup complete."
    echo "════════════════════════════════════════"

    exit $OVERALL_EXIT
fi

# ── cleanup: destroy across accounts in parallel ─────────────────────
if [ "$COMMAND" = "cleanup" ]; then
    TOTAL=${#TESTS[@]}
    echo "Running cleanup (destroy) on $TOTAL test(s) across $NUM_ACCOUNTS accounts"
    echo ""

    WORK_DIR=$(mktemp -d)
    trap 'for p in ${PIDS[@]+"${PIDS[@]}"}; do kill "$p" 2>/dev/null || true; done; rm -rf '"$WORK_DIR"'; release_account_locks' EXIT

    # Pre-authenticate accounts
    echo "Pre-authenticating AWS accounts..."
    for SLOT in $(seq 0 $((NUM_ACCOUNTS - 1))); do
        ACCOUNT="${ACCOUNTS[$SLOT]}"
        echo "  Authenticating $ACCOUNT..."
        if ! with_account_creds "$ACCOUNT" aws sts get-caller-identity >/dev/null 2>&1; then
            echo "  WARNING: Failed to pre-authenticate $ACCOUNT"
        fi
    done
    echo "Pre-authentication complete."
    echo ""

    # Each account tries to destroy all tests (we don't know which account
    # created which resources, so every account attempts every test)
    PIDS=()
    for SLOT in $(seq 0 $((NUM_ACCOUNTS - 1))); do
        ACCOUNT="${ACCOUNTS[$SLOT]}"
        LOG_FILE="$WORK_DIR/slot_${SLOT}.log"
        STATE_DIR="$WORK_DIR/state_${SLOT}"
        mkdir -p "$STATE_DIR"

        (
            set +e
            DESTROYED=0
            SKIPPED=0

            TEST_INDEX=0
            for TEST_FILE in "${TESTS[@]}"; do
                REL_PATH="${TEST_FILE#$SCRIPT_DIR/}"
                INJECTED_FILE=$(inject_provider_source "$TEST_FILE")
                TEST_STATE_DIR="$STATE_DIR/test_${TEST_INDEX}"
                mkdir -p "$TEST_STATE_DIR"
                cp "$INJECTED_FILE" "$TEST_STATE_DIR/main.crn"
                rm -f "$INJECTED_FILE"
                TEST_INDEX=$((TEST_INDEX + 1))
                # Skip if no state file exists — nothing to destroy
                if [ ! -f "$TEST_STATE_DIR/carina.state.json" ]; then
                    SKIPPED=$((SKIPPED + 1))
                    continue
                fi
                echo "RUNNING destroy $REL_PATH"
                DESTROY_OUTPUT=$(cd "$TEST_STATE_DIR" && with_account_creds "$ACCOUNT" "$CARINA_BIN" destroy --auto-approve "$TEST_STATE_DIR" 2>&1)
                DESTROY_RC=$?
                if [ $DESTROY_RC -eq 0 ]; then
                    if echo "$DESTROY_OUTPUT" | grep -q "No resources to destroy"; then
                        SKIPPED=$((SKIPPED + 1))
                    else
                        echo "DESTROYED $REL_PATH"
                        DESTROYED=$((DESTROYED + 1))
                    fi
                else
                    echo "FAIL (destroy) $REL_PATH"
                    echo "  ERROR: $DESTROY_OUTPUT"
                fi
            done

            echo "---"
            echo "SUMMARY $ACCOUNT: $DESTROYED destroyed, $SKIPPED already clean"
        ) > "$LOG_FILE" 2>&1 &

        PIDS+=($!)
        echo "  [$ACCOUNT] started (PID $!)"
    done

    echo ""
    echo "Logs: $WORK_DIR/slot_*.log"
    echo "Waiting for all accounts to finish cleanup..."
    echo ""

    # Wait for all workers
    OVERALL_EXIT=0
    for PID in ${PIDS[@]+"${PIDS[@]}"}; do
        wait "$PID" || OVERALL_EXIT=1
    done

    # Print results per account
    for SLOT in $(seq 0 $((NUM_ACCOUNTS - 1))); do
        LOG_FILE="$WORK_DIR/slot_${SLOT}.log"
        if [ ! -f "$LOG_FILE" ]; then
            continue
        fi
        ACCOUNT="${ACCOUNTS[$SLOT]}"
        echo "── $ACCOUNT ──"
        cat "$LOG_FILE"
        echo ""
    done

    echo "════════════════════════════════════════"
    echo "Cleanup complete."
    echo "════════════════════════════════════════"

    exit $OVERALL_EXIT
fi

# ── full: parallel execution across selected accounts ─────────────────
if [ "$COMMAND" = "full" ]; then
    TOTAL=${#TESTS[@]}
    echo "Running full cycle (apply -> plan-verify -> destroy) on $TOTAL test(s) across $NUM_ACCOUNTS accounts"
    if [ -n "$ACCOUNT_START" ]; then
        echo "  Using accounts: ${ACCOUNTS[*]}"
    fi
    echo ""

    WORK_DIR=$(mktemp -d)

    # Signal handling for the main process: forward signals to workers
    PIDS=()
    cleanup_main() {
        echo ""
        echo "Interrupted! Signaling workers to clean up..."
        for PID in ${PIDS[@]+"${PIDS[@]}"}; do
            kill -TERM "$PID" 2>/dev/null || true
        done
        echo "Waiting for workers to finish cleanup..."
        for PID in ${PIDS[@]+"${PIDS[@]}"}; do
            wait "$PID" 2>/dev/null || true
        done
        echo "All workers finished."
        rm -rf "$WORK_DIR"
        release_account_locks
        exit 1
    }
    trap cleanup_main INT TERM

    # Clean up temp dir and release locks on normal exit
    trap 'for p in ${PIDS[@]+"${PIDS[@]}"}; do kill "$p" 2>/dev/null || true; done; rm -rf '"$WORK_DIR"'; release_account_locks' EXIT

    # Pre-authenticate accounts sequentially to avoid opening
    # multiple SSO browser tabs simultaneously
    echo "Pre-authenticating AWS accounts..."
    for SLOT in $(seq 0 $((NUM_ACCOUNTS - 1))); do
        ACCOUNT="${ACCOUNTS[$SLOT]}"
        echo "  Authenticating $ACCOUNT..."
        if ! with_account_creds "$ACCOUNT" aws sts get-caller-identity >/dev/null 2>&1; then
            echo "  WARNING: Failed to pre-authenticate $ACCOUNT"
        fi
    done
    echo "Pre-authentication complete."
    echo ""

    # Deep cleanup: remove orphaned resources before running tests
    echo "Running deep cleanup to remove orphaned resources..."
    DEEP_CLEANUP_ACCOUNTS=()
    for SLOT in $(seq 0 $((NUM_ACCOUNTS - 1))); do
        ACCOUNT="${ACCOUNTS[$SLOT]}"
        deep_cleanup_account "$ACCOUNT" &
        PIDS+=($!)
        DEEP_CLEANUP_ACCOUNTS+=("$ACCOUNT")
    done

    DEEP_CLEANUP_EXIT=0
    for PID_IDX in "${!PIDS[@]}"; do
        if wait "${PIDS[$PID_IDX]}"; then
            :
        else
            DEEP_CLEANUP_EXIT=1
            echo "  ERROR: Deep cleanup failed for ${DEEP_CLEANUP_ACCOUNTS[$PID_IDX]}" >&2
        fi
    done
    PIDS=()
    echo ""

    if [ "$DEEP_CLEANUP_EXIT" -ne 0 ]; then
        echo "ERROR: Pre-test deep cleanup failed; aborting the full run before acceptance tests." >&2
        exit 1
    fi

    # Distribute tests round-robin across accounts
    for i in "${!TESTS[@]}"; do
        SLOT=$((i % NUM_ACCOUNTS))
        echo "${TESTS[$i]}" >> "$WORK_DIR/slot_${SLOT}.list"
    done

    # Launch one worker per account
    for SLOT in $(seq 0 $((NUM_ACCOUNTS - 1))); do
        LIST_FILE="$WORK_DIR/slot_${SLOT}.list"
        if [ ! -f "$LIST_FILE" ]; then
            continue
        fi

        ACCOUNT="${ACCOUNTS[$SLOT]}"
        LOG_FILE="$WORK_DIR/slot_${SLOT}.log"
        SLOT_DIR="$WORK_DIR/state_${SLOT}"
        mkdir -p "$SLOT_DIR"

        (
            set +e
            PASSED=0
            FAILED=0
            CURRENT_STATE_DIR=""
            INTERRUPTED=0

            # Worker trap: attempt destroy for the current test on interruption
            worker_cleanup() {
                INTERRUPTED=1
                if [ -n "$CURRENT_STATE_DIR" ]; then
                    echo "INTERRUPTED - destroying resources for current test"
                    cd "$CURRENT_STATE_DIR" && with_account_creds "$ACCOUNT" "$CARINA_BIN" destroy --auto-approve "$CURRENT_STATE_DIR" 2>&1 || true
                fi
            }
            trap worker_cleanup INT TERM

            TEST_INDEX=0
            while IFS= read -r TEST_FILE; do
                if [ $INTERRUPTED -eq 1 ]; then
                    break
                fi

                REL_PATH="${TEST_FILE#$SCRIPT_DIR/}"
                INJECTED_FILE=$(inject_provider_source "$TEST_FILE")

                # Use a separate state directory per test to prevent
                # state file cross-contamination between tests (issue #537)
                CURRENT_STATE_DIR="$SLOT_DIR/test_${TEST_INDEX}"
                mkdir -p "$CURRENT_STATE_DIR"
                cp "$INJECTED_FILE" "$CURRENT_STATE_DIR/main.crn"
                rm -f "$INJECTED_FILE"
                TEST_INDEX=$((TEST_INDEX + 1))

                # Initialize backend lock and install providers before apply
                if ! INIT_OUTPUT=$("$CARINA_BIN" init "$CURRENT_STATE_DIR" 2>&1); then
                    echo "FAIL (init) $REL_PATH"
                    echo "  ERROR: $INIT_OUTPUT"
                    FAILED=$((FAILED + 1))
                    CURRENT_STATE_DIR=""
                    continue
                fi

                # Apply (run from state dir so each test has its own state file)
                echo "RUNNING apply $REL_PATH"
                APPLY_OUTPUT=$(cd "$CURRENT_STATE_DIR" && with_account_creds "$ACCOUNT" "$CARINA_BIN" apply --auto-approve "$CURRENT_STATE_DIR" 2>&1)
                APPLY_RC=$?
                if [ $INTERRUPTED -eq 1 ]; then
                    break
                fi
                if [ $APPLY_RC -ne 0 ] || echo "$APPLY_OUTPUT" | grep -q "failed"; then
                    echo "FAIL (apply) $REL_PATH"
                    echo "  ERROR: $APPLY_OUTPUT"
                    FAILED=$((FAILED + 1))
                    # Try to destroy whatever was partially created
                    cd "$CURRENT_STATE_DIR" && with_account_creds "$ACCOUNT" "$CARINA_BIN" destroy --auto-approve "$CURRENT_STATE_DIR" 2>&1 || true
                    CURRENT_STATE_DIR=""
                    continue
                fi

                # Post-apply plan verification (idempotency check)
                echo "RUNNING plan-verify $REL_PATH"
                PLAN_OUTPUT=$(cd "$CURRENT_STATE_DIR" && with_account_creds "$ACCOUNT" "$CARINA_BIN" plan --detailed-exitcode "$CURRENT_STATE_DIR" 2>&1)
                PLAN_RC=$?
                if [ $INTERRUPTED -eq 1 ]; then
                    break
                fi
                if [ $PLAN_RC -eq 2 ]; then
                    echo "FAIL (plan-verify) $REL_PATH"
                    echo "  ERROR: Post-apply plan detected changes (not idempotent):"
                    echo "  $PLAN_OUTPUT"
                    FAILED=$((FAILED + 1))
                    # Still destroy to clean up
                    cd "$CURRENT_STATE_DIR" && with_account_creds "$ACCOUNT" "$CARINA_BIN" destroy --auto-approve "$CURRENT_STATE_DIR" 2>&1 || true
                    CURRENT_STATE_DIR=""
                    continue
                elif [ $PLAN_RC -ne 0 ]; then
                    echo "FAIL (plan-verify) $REL_PATH"
                    echo "  ERROR: $PLAN_OUTPUT"
                    FAILED=$((FAILED + 1))
                    cd "$CURRENT_STATE_DIR" && with_account_creds "$ACCOUNT" "$CARINA_BIN" destroy --auto-approve "$CURRENT_STATE_DIR" 2>&1 || true
                    CURRENT_STATE_DIR=""
                    continue
                fi

                # Destroy
                echo "RUNNING destroy $REL_PATH"
                DESTROY_OUTPUT=$(cd "$CURRENT_STATE_DIR" && with_account_creds "$ACCOUNT" "$CARINA_BIN" destroy --auto-approve "$CURRENT_STATE_DIR" 2>&1)
                DESTROY_RC=$?
                if [ $DESTROY_RC -ne 0 ] || echo "$DESTROY_OUTPUT" | grep -q "failed"; then
                    echo "FAIL (destroy) $REL_PATH"
                    echo "  ERROR: $DESTROY_OUTPUT"
                    FAILED=$((FAILED + 1))
                    CURRENT_STATE_DIR=""
                    continue
                fi

                echo "OK   $REL_PATH"
                PASSED=$((PASSED + 1))
                CURRENT_STATE_DIR=""
            done < "$LIST_FILE"

            echo "---"
            echo "SUMMARY $ACCOUNT: $PASSED passed, $FAILED failed"
        ) > "$LOG_FILE" 2>&1 &

        PIDS+=($!)
        echo "  [$ACCOUNT] started (PID $!) - $(wc -l < "$LIST_FILE" | tr -d ' ') test(s)"
    done

    echo ""
    echo "Logs: $WORK_DIR/slot_*.log"
    echo "Waiting for all accounts to finish..."
    echo ""

    # Wait and collect results
    OVERALL_EXIT=0
    for SLOT in $(seq 0 $((NUM_ACCOUNTS - 1))); do
        LOG_FILE="$WORK_DIR/slot_${SLOT}.log"
        if [ ! -f "$LOG_FILE" ] && [ $SLOT -ge ${#PIDS[@]} ]; then
            continue
        fi
        PID_IDX=$SLOT
        if [ $PID_IDX -lt ${#PIDS[@]} ]; then
            wait "${PIDS[$PID_IDX]}" || OVERALL_EXIT=1
        fi
    done

    # Print results per account
    for SLOT in $(seq 0 $((NUM_ACCOUNTS - 1))); do
        LOG_FILE="$WORK_DIR/slot_${SLOT}.log"
        if [ ! -f "$LOG_FILE" ]; then
            continue
        fi
        ACCOUNT="${ACCOUNTS[$SLOT]}"
        echo "── $ACCOUNT ──"
        cat "$LOG_FILE"
        echo ""
    done

    # Aggregate
    TOTAL_PASSED=0
    TOTAL_FAILED=0
    for SLOT in $(seq 0 $((NUM_ACCOUNTS - 1))); do
        LOG_FILE="$WORK_DIR/slot_${SLOT}.log"
        if [ ! -f "$LOG_FILE" ]; then
            continue
        fi
        P=$(grep "^SUMMARY" "$LOG_FILE" | sed 's/.*: \([0-9]*\) passed.*/\1/' || echo 0)
        F=$(grep "^SUMMARY" "$LOG_FILE" | sed 's/.*, \([0-9]*\) failed/\1/' || echo 0)
        TOTAL_PASSED=$((TOTAL_PASSED + P))
        TOTAL_FAILED=$((TOTAL_FAILED + F))
    done

    echo "════════════════════════════════════════"
    echo "Total: $TOTAL_PASSED passed, $TOTAL_FAILED failed (of $TOTAL)"
    echo "════════════════════════════════════════"

    exit $OVERALL_EXIT
fi

# ── Single-command mode (validate/plan/apply/destroy) ────────────────
echo "Running '$COMMAND' on ${#TESTS[@]} test file(s):"
echo ""

# Use per-test isolated state directories for commands that touch state
# (plan/apply/destroy) to prevent cross-contamination between tests (issue #839)
if [ "$COMMAND" != "validate" ]; then
    WORK_DIR=$(mktemp -d)
    trap 'for p in ${PIDS[@]+"${PIDS[@]}"}; do kill "$p" 2>/dev/null || true; done; rm -rf '"$WORK_DIR"'; release_account_locks' EXIT
fi

PASSED=0
FAILED=0
ERRORS=()
TEST_INDEX=0

for TEST_FILE in "${TESTS[@]}"; do
    REL_PATH="${TEST_FILE#$SCRIPT_DIR/}"
    printf "  %-55s " "$REL_PATH"

    INJECTED_FILE=$(inject_provider_source "$TEST_FILE")

    AUTO_APPROVE=""
    if [ "$COMMAND" = "apply" ] || [ "$COMMAND" = "destroy" ]; then
        AUTO_APPROVE="--auto-approve"
    fi

    # Run from an isolated state directory for stateful commands
    if [ "$COMMAND" != "validate" ]; then
        STATE_DIR="$WORK_DIR/test_${TEST_INDEX}"
        mkdir -p "$STATE_DIR"
        cp "$INJECTED_FILE" "$STATE_DIR/main.crn"
        rm -f "$INJECTED_FILE"
        TEST_INDEX=$((TEST_INDEX + 1))
        TARGET_PATH="$STATE_DIR"

        # Initialize backend lock and install providers before the stateful command
        if ! INIT_OUTPUT=$("$CARINA_BIN" init "$STATE_DIR" 2>&1); then
            echo "FAIL (init)"
            ERRORS+=("$REL_PATH: $INIT_OUTPUT")
            FAILED=$((FAILED + 1))
            continue
        fi
    else
        # For validate, create a temp directory with the injected file
        VALIDATE_DIR=$(mktemp -d)
        cp "$INJECTED_FILE" "$VALIDATE_DIR/main.crn"
        rm -f "$INJECTED_FILE"
        TARGET_PATH="$VALIDATE_DIR"

        # `carina validate` loads the provider WASM to do schema-driven
        # validation, so the provider must be installed first. `carina init`
        # on a file:// source does not require AWS credentials.
        if ! INIT_OUTPUT=$("$CARINA_BIN" init "$VALIDATE_DIR" 2>&1); then
            echo "FAIL (init)"
            ERRORS+=("$REL_PATH: $INIT_OUTPUT")
            FAILED=$((FAILED + 1))
            rm -rf "$VALIDATE_DIR"
            continue
        fi
    fi

    RUN_PREFIX=""
    if [ "$COMMAND" != "validate" ]; then
        RUN_PREFIX="cd $STATE_DIR &&"
    fi

    if OUTPUT=$(eval "$RUN_PREFIX \"$CARINA_BIN\" \"$COMMAND\" $AUTO_APPROVE \"$TARGET_PATH\"" 2>&1); then
        if [ "$COMMAND" = "apply" ]; then
            # Post-apply plan verification (idempotency check)
            # Use || to capture non-zero exit codes without triggering set -e
            PLAN_RC=0
            PLAN_OUTPUT=$(eval "$RUN_PREFIX \"$CARINA_BIN\" plan --detailed-exitcode \"$TARGET_PATH\"" 2>&1) || PLAN_RC=$?
            if [ $PLAN_RC -eq 2 ]; then
                echo "FAIL (plan-verify)"
                ERRORS+=("$REL_PATH: Post-apply plan detected changes (not idempotent): $PLAN_OUTPUT")
                FAILED=$((FAILED + 1))
                continue
            elif [ $PLAN_RC -ne 0 ]; then
                echo "FAIL (plan-verify)"
                ERRORS+=("$REL_PATH: $PLAN_OUTPUT")
                FAILED=$((FAILED + 1))
                continue
            fi
        fi
        echo "OK"
        PASSED=$((PASSED + 1))
    else
        echo "FAIL"
        ERRORS+=("$REL_PATH: $OUTPUT")
        FAILED=$((FAILED + 1))
    fi
    if [ "$COMMAND" = "validate" ]; then
        rm -rf "$VALIDATE_DIR"
    fi
done

echo ""
echo "Results: $PASSED passed, $FAILED failed (total: $((PASSED + FAILED)))"

if [ ${#ERRORS[@]} -gt 0 ]; then
    echo ""
    echo "Failures:"
    for ERR in "${ERRORS[@]}"; do
        echo "  $ERR"
    done
    exit 1
fi
