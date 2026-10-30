#!/bin/bash
# Standalone acceptance test for an HTTPS listener update that must retain a
# nested write-only authenticate-oidc client secret (carina#3984).
#
# Usage:
#   env AWS_PROFILE="<profile>" ./run.sh
#
# This suite is intentionally skipped by acceptance-tests/run-tests.sh because
# its directory contains a custom run.sh.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# The helper path is resolved dynamically from this script's directory.
# shellcheck disable=SC1091
source "$SCRIPT_DIR/../shared/_helpers.sh"

REGION="ap-northeast-1"
LOAD_BALANCER_NAME="carina-acc-test-oidc-alb"
TARGET_GROUP_NAME="carina-acc-test-oidc-tg"
OIDC_CLIENT_ID="carina-acc-oidc"
STEP2_SSL_POLICY="ELBSecurityPolicy-TLS13-1-2-Res-2021-06"
CERTIFICATE_PLACEHOLDER="__ACM_CERTIFICATE_ARN__"
TOTAL_PASSED=0
TOTAL_FAILED=0
LAST_STEP_SUCCEEDED=0
ACTIVE_WORK_DIR=""
CERT_TEMP_DIR=""
CERTIFICATE_ARN=""
LISTENER_ARN=""

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
        LAST_STEP_SUCCEEDED=1
        return 0
    else
        echo "FAIL"
        echo "  ERROR: $output"
        TOTAL_FAILED=$((TOTAL_FAILED + 1))
        LAST_STEP_SUCCEEDED=0
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

plan_ssl_policy_update() {
    local output
    local rc=0
    output=$(cd "$ACTIVE_WORK_DIR" && with_account_creds run_with_timeout 600 "$CARINA_BIN" plan --detailed-exitcode . 2>&1) || rc=$?

    if [ "$rc" -ne 2 ]; then
        echo "Expected a change plan with exit code 2, got $rc:" >&2
        echo "$output" >&2
        return 1
    fi

    if grep -Eq '^[[:space:]]*-/\+[[:space:]]+awscc\.elasticloadbalancingv2\.Listener([[:space:]]|$)|must be replaced' <<< "$output"; then
        echo "Listener was planned for replacement, not an in-place update:" >&2
        echo "$output" >&2
        return 1
    fi

    if ! grep -Eq '^[[:space:]]*~[[:space:]]+awscc\.elasticloadbalancingv2\.Listener([[:space:]]|$)' <<< "$output"; then
        echo "Listener plan did not contain the in-place update marker:" >&2
        echo "$output" >&2
        return 1
    fi

    return 0
}

install_config() {
    local source_crn="$1"
    local injected_dir
    inject_provider_source "$source_crn" injected_dir || return 1
    if ! sed "s|$CERTIFICATE_PLACEHOLDER|$CERTIFICATE_ARN|g" \
        "$injected_dir/main.crn" > "$ACTIVE_WORK_DIR/main.crn"; then
        rm -rf "$injected_dir"
        return 1
    fi
    rm -rf "$injected_dir"
}

generate_certificate() {
    local private_key="$CERT_TEMP_DIR/private-key.pem"
    local certificate="$CERT_TEMP_DIR/certificate.pem"

    openssl req -x509 -newkey rsa:2048 -sha256 -days 1 -nodes \
        -subj "/CN=carina-acc-test-oidc.invalid" \
        -keyout "$private_key" -out "$certificate" >/dev/null 2>&1
    chmod 600 "$private_key"
}

import_certificate() {
    local arn
    with_account_creds aws acm import-certificate \
        --region "$REGION" \
        --certificate "fileb://$CERT_TEMP_DIR/certificate.pem" \
        --private-key "fileb://$CERT_TEMP_DIR/private-key.pem" \
        --query CertificateArn --output text > "$CERT_TEMP_DIR/certificate-arn"
    arn="$(<"$CERT_TEMP_DIR/certificate-arn")"
    case "$arn" in
        arn:*:acm:*:certificate/*) ;;
        *)
            echo "ACM import-certificate returned no certificate ARN" >&2
            return 1
            ;;
    esac
}

load_certificate_arn() {
    local arn
    [ -s "$CERT_TEMP_DIR/certificate-arn" ] || return 1
    arn="$(<"$CERT_TEMP_DIR/certificate-arn")"
    case "$arn" in
        arn:*:acm:*:certificate/*)
            CERTIFICATE_ARN="$arn"
            ;;
        *) return 1 ;;
    esac
}

apply_listener() {
    cd "$ACTIVE_WORK_DIR"
    with_account_creds run_with_timeout 900 "$CARINA_BIN" apply --auto-approve .
}

destroy_listener() {
    cd "$ACTIVE_WORK_DIR"
    with_account_creds run_with_timeout 900 "$CARINA_BIN" destroy --auto-approve .
}

assert_updated_listener() {
    local load_balancer_arn
    local listeners_json
    local listener_json

    load_balancer_arn=$(with_account_creds aws elbv2 describe-load-balancers \
        --region "$REGION" --names "$LOAD_BALANCER_NAME" \
        --query 'LoadBalancers[0].LoadBalancerArn' --output text) || return 1
    if [ -z "$load_balancer_arn" ] || [ "$load_balancer_arn" = "None" ]; then
        echo "Could not resolve load balancer $LOAD_BALANCER_NAME" >&2
        return 1
    fi

    listeners_json=$(with_account_creds aws elbv2 describe-listeners \
        --region "$REGION" --load-balancer-arn "$load_balancer_arn" \
        --output json) || return 1
    listener_json=$(jq -ce '[.Listeners[]? | select(.Port == 443)][0]' \
        <<< "$listeners_json") || return 1

    if ! jq -e --arg policy "$STEP2_SSL_POLICY" --arg client_id "$OIDC_CLIENT_ID" '
        .SslPolicy == $policy
        and any(.DefaultActions[]?;
            .Type == "authenticate-oidc"
            and .AuthenticateOidcConfig.ClientId == $client_id)
    ' <<< "$listener_json" >/dev/null; then
        echo "Listener did not retain the expected SSL policy and authenticate-oidc action:" >&2
        echo "$listener_json" >&2
        return 1
    fi

    jq -er '.ListenerArn' <<< "$listener_json" > "$CERT_TEMP_DIR/listener-arn"
}

assert_listener_gone() {
    local output
    [ -n "$LISTENER_ARN" ] || return 1
    if output=$(with_account_creds aws elbv2 describe-listeners \
        --region "$REGION" --listener-arns "$LISTENER_ARN" 2>&1); then
        return 1
    fi
    if grep -q 'ListenerNotFound' <<< "$output"; then
        return 0
    fi
    echo "$output" >&2
    return 1
}

assert_load_balancer_gone() {
    local resources
    resources=$(with_account_creds aws elbv2 describe-load-balancers \
        --region "$REGION" --output json) || return 1
    ! jq -e --arg name "$LOAD_BALANCER_NAME" '
        .LoadBalancers[]? | select(.LoadBalancerName == $name)
    ' <<< "$resources" >/dev/null
}

assert_target_group_gone() {
    local resources
    resources=$(with_account_creds aws elbv2 describe-target-groups \
        --region "$REGION" --output json) || return 1
    ! jq -e --arg name "$TARGET_GROUP_NAME" '
        .TargetGroups[]? | select(.TargetGroupName == $name)
    ' <<< "$resources" >/dev/null
}

assert_certificate_gone() {
    local resources
    [ -n "$CERTIFICATE_ARN" ] || return 0
    resources=$(with_account_creds aws acm list-certificates \
        --region "$REGION" --output json) || return 1
    ! jq -e --arg arn "$CERTIFICATE_ARN" '
        .CertificateSummaryList[]? | select(.CertificateArn == $arn)
    ' <<< "$resources" >/dev/null
}

delete_certificate_with_retries() {
    local attempt
    local output=""

    [ -n "$CERTIFICATE_ARN" ] || return 0
    for attempt in $(seq 1 12); do
        if output=$(with_account_creds aws acm delete-certificate \
            --region "$REGION" --certificate-arn "$CERTIFICATE_ARN" 2>&1); then
            return 0
        fi
        if assert_certificate_gone; then
            return 0
        fi
        if [ "$attempt" -lt 12 ]; then
            sleep 5
        fi
    done

    echo "Unable to delete ACM certificate after 12 attempts: $output" >&2
    return 1
}

cleanup() {
    local exit_status=$?
    trap - EXIT
    set +e

    if [ -z "$CERTIFICATE_ARN" ] && [ -n "$CERT_TEMP_DIR" ]; then
        load_certificate_arn || true
    fi

    if [ -n "$ACTIVE_WORK_DIR" ] && [ -d "$ACTIVE_WORK_DIR" ]; then
        echo ""
        echo "  Cleanup: destroying OIDC listener resources..."
        (cd "$ACTIVE_WORK_DIR" && with_account_creds run_with_timeout 900 "$CARINA_BIN" destroy --auto-approve . 2>&1)
        (cd "$ACTIVE_WORK_DIR" && with_account_creds run_with_timeout 900 "$CARINA_BIN" destroy --auto-approve . 2>&1)
    fi

    if [ -n "$CERTIFICATE_ARN" ]; then
        echo "  Cleanup: deleting imported ACM certificate..."
        if ! delete_certificate_with_retries; then
            exit_status=1
        fi
        if ! assert_certificate_gone; then
            echo "  ERROR: imported ACM certificate still exists or could not be verified" >&2
            exit_status=1
        fi
    fi

    if [ -n "$ACTIVE_WORK_DIR" ] && [ -d "$ACTIVE_WORK_DIR" ]; then
        rm -rf "$ACTIVE_WORK_DIR"
    fi
    if [ -n "$CERT_TEMP_DIR" ] && [ -d "$CERT_TEMP_DIR" ]; then
        rm -rf "$CERT_TEMP_DIR"
    fi

    exit "$exit_status"
}
trap cleanup EXIT

WORK_DIR="$(mktemp -d)"
ACTIVE_WORK_DIR="$WORK_DIR"
CERT_TEMP_DIR="$(mktemp -d)"

echo "Running ELBv2 Listener OIDC update acceptance test with AWS_PROFILE=${AWS_PROFILE:-}"
echo ""

record_step "generate temporary self-signed certificate" generate_certificate
if [ "$LAST_STEP_SUCCEEDED" -ne 1 ]; then
    exit 1
fi

record_step "import temporary certificate into ACM" import_certificate
if [ "$LAST_STEP_SUCCEEDED" -ne 1 ]; then
    exit 1
fi
if ! load_certificate_arn; then
    echo "ERROR: imported certificate ARN was not recorded" >&2
    exit 1
fi

install_config "$SCRIPT_DIR/step1.crn"
record_step "init" "$CARINA_BIN" init "$WORK_DIR"
record_step "step1: apply HTTPS OIDC listener" apply_listener
plan_verify "$WORK_DIR"

install_config "$SCRIPT_DIR/step2.crn"
record_step "step2: plan in-place ssl_policy update" plan_ssl_policy_update
record_step "step2: apply ssl_policy update" apply_listener
plan_verify "$WORK_DIR"
record_step "assert: policy and OIDC action survived update" assert_updated_listener
if [ -s "$CERT_TEMP_DIR/listener-arn" ]; then
    LISTENER_ARN="$(<"$CERT_TEMP_DIR/listener-arn")"
fi

record_step "destroy" destroy_listener
destroy_succeeded="$LAST_STEP_SUCCEEDED"
record_step "assert: listener is gone" assert_listener_gone
listener_gone_succeeded="$LAST_STEP_SUCCEEDED"
record_step "assert: load balancer is gone" assert_load_balancer_gone
load_balancer_gone_succeeded="$LAST_STEP_SUCCEEDED"
record_step "assert: target group is gone" assert_target_group_gone
target_group_gone_succeeded="$LAST_STEP_SUCCEEDED"
record_step "delete imported ACM certificate" delete_certificate_with_retries
certificate_delete_succeeded="$LAST_STEP_SUCCEEDED"
record_step "assert: certificate is gone" assert_certificate_gone
certificate_gone_succeeded="$LAST_STEP_SUCCEEDED"

if [ "$certificate_gone_succeeded" -eq 1 ]; then
    CERTIFICATE_ARN=""
fi

if [ "$destroy_succeeded" -eq 1 ] \
    && [ "$listener_gone_succeeded" -eq 1 ] \
    && [ "$load_balancer_gone_succeeded" -eq 1 ] \
    && [ "$target_group_gone_succeeded" -eq 1 ] \
    && [ "$certificate_delete_succeeded" -eq 1 ] \
    && [ "$certificate_gone_succeeded" -eq 1 ]; then
    rm -rf "$WORK_DIR" "$CERT_TEMP_DIR"
    ACTIVE_WORK_DIR=""
    CERT_TEMP_DIR=""
fi

echo ""
echo "Results: $TOTAL_PASSED passed, $TOTAL_FAILED failed"
if [ "$TOTAL_FAILED" -gt 0 ]; then
    exit 1
fi
