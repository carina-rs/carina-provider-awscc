#!/bin/bash
# Test deep-cleanup dependency ordering without making AWS calls.
#
# Usage:
#   bash acceptance-tests/shell-tests/deep_cleanup_dependency_ordering.sh
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

LB_ONE_ARN="arn:aws:elasticloadbalancing:us-east-1:123456789012:loadbalancer/app/acceptance-test-one/111"
LB_TWO_ARN="arn:aws:elasticloadbalancing:us-east-1:123456789012:loadbalancer/app/carina-acc-two/222"
TARGET_GROUP_ARN="arn:aws:elasticloadbalancing:us-east-1:123456789012:targetgroup/acceptance-test/333"
VPC_ID="vpc-ordering"
SUBNET_ID="subnet-ordering"
SOURCE_SG_ID="sg-source"
DEST_SG_ID="sg-destination"
SOURCE_INGRESS_RULE_ID="sgr-source-ingress"
SOURCE_EGRESS_RULE_ID="sgr-source-egress"
DEST_INGRESS_RULE_ID="sgr-destination-ingress"
DEST_EGRESS_RULE_ID="sgr-destination-egress"
UNEXPECTED_CASE_RULE_ID="sgr-unexpected-case"
IN_USE_ENI_ID="eni-in-use"
AVAILABLE_ENI_ID="eni-available"

with_account_creds() {
    shift
    local command="$*"
    echo "$command" >> "$COMMAND_LOG"

    # Match production's inner credential subshell and its fd inheritance.
    (
        case "$command" in
            "aws wafv2 list-web-acls"*)
                printf '{"WebACLs":[]}\n'
                ;;
            "aws elbv2 describe-load-balancers"*)
                printf '%s\t%s\n' "$LB_ONE_ARN" "$LB_TWO_ARN"
                ;;
            "aws elbv2 describe-target-groups"*)
                echo "$TARGET_GROUP_ARN"
                ;;
            "aws ec2 describe-vpcs"*)
                echo "$VPC_ID"
                ;;
            "aws ec2 describe-network-interfaces"*)
                if [[ "$command" == *"Name=vpc-id,Values=$VPC_ID"* ]] &&
                    [[ "$command" == *"Name=status,Values=in-use"* ]]; then
                    echo "$IN_USE_ENI_ID"
                elif [[ "$command" == *"Name=vpc-id,Values=$VPC_ID"* ]] &&
                    [[ "$command" == *"Name=status,Values=available"* ]]; then
                    echo "$AVAILABLE_ENI_ID"
                fi
                ;;
            "aws ec2 describe-subnets"*)
                echo "$SUBNET_ID"
                ;;
            "aws ec2 describe-security-groups"*)
                printf '%s\t%s\n' "$SOURCE_SG_ID" "$DEST_SG_ID"
                ;;
            "aws ec2 describe-security-group-rules"*)
                if [[ "$command" == *"Name=group-id,Values=$SOURCE_SG_ID"* ]]; then
                    printf '%s\tFalse\n%s\tTrue\n%s\tfalse\n' \
                        "$SOURCE_INGRESS_RULE_ID" "$SOURCE_EGRESS_RULE_ID" "$UNEXPECTED_CASE_RULE_ID"
                elif [[ "$command" == *"Name=group-id,Values=$DEST_SG_ID"* ]]; then
                    printf '%s\tFalse\n%s\tTrue\n' "$DEST_INGRESS_RULE_ID" "$DEST_EGRESS_RULE_ID"
                fi
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

first_line_containing() {
    local text="$1"
    grep -nF "$text" "$COMMAND_LOG" | head -1 | cut -d: -f1 || true
}

last_line_containing() {
    local text="$1"
    grep -nF "$text" "$COMMAND_LOG" | tail -1 | cut -d: -f1 || true
}

# shellcheck source=/dev/null
source "$FUNCTIONS_FILE"

: > "$COMMAND_LOG"
if ! deep_cleanup_account "ordering-account" >"$WORK_DIR/stdout.log" 2>"$WORK_DIR/stderr.log"; then
    fail "deep_cleanup_account returned non-zero for the successful ordering scenario"
fi

last_lb_delete=$(last_line_containing "aws elbv2 delete-load-balancer --load-balancer-arn")
lb_wait=$(first_line_containing "deep_cleanup_with_timeout 720 aws elbv2 wait load-balancers-deleted --load-balancer-arns $LB_ONE_ARN $LB_TWO_ARN")
first_target_group_delete=$(first_line_containing "aws elbv2 delete-target-group --target-group-arn")
if [ -z "$lb_wait" ]; then
    fail "missing load-balancers-deleted waiter for both deleted load balancers"
elif [ -z "$last_lb_delete" ] || [ "$lb_wait" -le "$last_lb_delete" ]; then
    fail "load-balancers-deleted waiter did not run after all load-balancer deletes"
elif [ -z "$first_target_group_delete" ] || [ "$lb_wait" -ge "$first_target_group_delete" ]; then
    fail "load-balancers-deleted waiter did not run before the first target-group delete"
fi
if grep -F "aws elbv2 delete-listener" "$COMMAND_LOG" >/dev/null; then
    fail "deep cleanup must rely on load-balancer deletion to remove listeners"
fi

first_sg_delete=$(first_line_containing "aws ec2 delete-security-group --group-id")
last_sg_revoke=""
for expected_revoke in \
    "aws ec2 revoke-security-group-ingress --group-id $SOURCE_SG_ID --security-group-rule-ids $SOURCE_INGRESS_RULE_ID" \
    "aws ec2 revoke-security-group-egress --group-id $SOURCE_SG_ID --security-group-rule-ids $SOURCE_EGRESS_RULE_ID" \
    "aws ec2 revoke-security-group-ingress --group-id $DEST_SG_ID --security-group-rule-ids $DEST_INGRESS_RULE_ID" \
    "aws ec2 revoke-security-group-egress --group-id $DEST_SG_ID --security-group-rule-ids $DEST_EGRESS_RULE_ID"
do
    revoke_line=$(first_line_containing "$expected_revoke")
    if [ -z "$revoke_line" ]; then
        fail "missing security-group rule revoke: $expected_revoke"
        continue
    fi
    if [ -z "$last_sg_revoke" ] || [ "$revoke_line" -gt "$last_sg_revoke" ]; then
        last_sg_revoke="$revoke_line"
    fi
done
if [ -n "$last_sg_revoke" ] && { [ -z "$first_sg_delete" ] || [ "$last_sg_revoke" -ge "$first_sg_delete" ]; }; then
    fail "not every security-group rule revoke ran before the first security-group delete"
fi
if grep -F "sg-default" "$COMMAND_LOG" >/dev/null; then
    fail "default security-group rules must not be enumerated, revoked, or deleted"
fi
if ! grep -F "ERROR: Failed to classify security-group rule $UNEXPECTED_CASE_RULE_ID for $SOURCE_SG_ID: unexpected IsEgress value 'false'" "$WORK_DIR/stderr.log" >/dev/null; then
    fail "unexpected IsEgress value did not reach the classification guard"
fi
if grep -F -- "--security-group-rule-ids $UNEXPECTED_CASE_RULE_ID" "$COMMAND_LOG" >/dev/null; then
    fail "unexpected IsEgress value was incorrectly revoked as ingress or egress"
fi

eni_wait=$(first_line_containing "deep_cleanup_wait_for_network_interface 300 aws ec2 wait network-interface-available --network-interface-ids $IN_USE_ENI_ID")
available_eni_delete=$(first_line_containing "aws ec2 delete-network-interface --network-interface-id $AVAILABLE_ENI_ID")
first_subnet_delete=$(first_line_containing "aws ec2 delete-subnet --subnet-id")
if [ -z "$eni_wait" ]; then
    fail "missing bounded availability wait for the VPC's in-use network interface"
elif [ -z "$first_subnet_delete" ] || [ "$eni_wait" -ge "$first_subnet_delete" ]; then
    fail "network-interface availability wait did not run before the first subnet delete"
fi
if [ -z "$available_eni_delete" ]; then
    fail "missing delete for the VPC's available network interface"
elif [ -z "$first_subnet_delete" ] || [ "$available_eni_delete" -ge "$first_subnet_delete" ]; then
    fail "available network interface was not deleted before the first subnet delete"
fi
if grep -F "aws ec2 delete-network-interface --network-interface-id $IN_USE_ENI_ID" "$COMMAND_LOG" >/dev/null; then
    fail "in-use network interface must not be force-deleted"
fi

if [ "$FAILURES" -ne 0 ]; then
    exit 1
fi

echo "deep_cleanup_dependency_ordering: OK"
