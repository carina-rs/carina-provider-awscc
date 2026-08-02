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
FLOW_LOG_STUB_FAILURE_LOG="$WORK_DIR/flow-log-stub-failures.log"
EOIGW_STUB_FAILURE_LOG="$WORK_DIR/eoigw-stub-failures.log"
ROUTE_TABLE_STUB_FAILURE_LOG="$WORK_DIR/route-table-stub-failures.log"

cleanup() {
    rm -rf "$WORK_DIR"
}
trap cleanup EXIT

FAILURES=0
fail() {
    echo "FAIL: $*" >&2
    FAILURES=$((FAILURES + 1))
}

record_stub_failure() {
    local failure_file="$1"
    shift
    local message="$*"
    printf '%s\n' "$message" >> "$failure_file"
    printf 'STUB FAILURE: %s\n' "$message" >&2
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
MAIN_ROUTE_TABLE_ID="rtb-ordering-main"
ROUTE_TABLE_ONE_ID="rtb-ordering-one"
ROUTE_TABLE_TWO_ID="rtb-ordering-two"
MAIN_ROUTE_TABLE_SUBNET_ASSOCIATION_ID="rtbassoc-main-subnet"
SUBNET_ROUTE_TABLE_ASSOCIATION_ID="rtbassoc-subnet"
GATEWAY_ROUTE_TABLE_ASSOCIATION_ID="rtbassoc-gateway"
SECOND_ROUTE_TABLE_ASSOCIATION_ID="rtbassoc-second"
MAIN_ROUTE_TABLE_ASSOCIATION_ID="rtbassoc-main"
SOURCE_SG_ID="sg-source"
DEST_SG_ID="sg-destination"
SOURCE_INGRESS_RULE_ID="sgr-source-ingress"
SOURCE_EGRESS_RULE_ID="sgr-source-egress"
DEST_INGRESS_RULE_ID="sgr-destination-ingress"
DEST_EGRESS_RULE_ID="sgr-destination-egress"
UNEXPECTED_CASE_RULE_ID="sgr-unexpected-case"
IN_USE_ENI_ID="eni-in-use"
AVAILABLE_ENI_ID="eni-available"
FLOW_LOG_ID="fl-ordering"
MATCHING_EOIGW_ID="eigw-ordering"
UNRELATED_EOIGW_ID="eigw-other-vpc"
OTHER_VPC_ID="vpc-other"
EOIGW_RESPONSE=$(jq -cn \
    --arg matching_id "$MATCHING_EOIGW_ID" \
    --arg unrelated_id "$UNRELATED_EOIGW_ID" \
    --arg vpc_id "$VPC_ID" \
    --arg other_vpc_id "$OTHER_VPC_ID" \
    '{
        EgressOnlyInternetGateways: [
            {
                EgressOnlyInternetGatewayId: $matching_id,
                Attachments: [
                    {VpcId: $other_vpc_id},
                    {VpcId: $vpc_id}
                ]
            },
            {
                EgressOnlyInternetGatewayId: $unrelated_id,
                Attachments: [{VpcId: $other_vpc_id}]
            }
        ]
    }')
ROUTE_TABLE_RESPONSE=$(jq -cn \
    --arg main_route_table_id "$MAIN_ROUTE_TABLE_ID" \
    --arg route_table_one_id "$ROUTE_TABLE_ONE_ID" \
    --arg route_table_two_id "$ROUTE_TABLE_TWO_ID" \
    --arg main_subnet_association_id "$MAIN_ROUTE_TABLE_SUBNET_ASSOCIATION_ID" \
    --arg main_association_id "$MAIN_ROUTE_TABLE_ASSOCIATION_ID" \
    --arg subnet_association_id "$SUBNET_ROUTE_TABLE_ASSOCIATION_ID" \
    --arg gateway_association_id "$GATEWAY_ROUTE_TABLE_ASSOCIATION_ID" \
    --arg second_association_id "$SECOND_ROUTE_TABLE_ASSOCIATION_ID" \
    '{
        RouteTables: [
            {
                RouteTableId: $main_route_table_id,
                Associations: [
                    {RouteTableAssociationId: $main_subnet_association_id, Main: false},
                    {RouteTableAssociationId: $main_association_id, Main: true}
                ]
            },
            {
                RouteTableId: $route_table_one_id,
                Associations: [
                    {RouteTableAssociationId: $subnet_association_id, Main: false},
                    {RouteTableAssociationId: $gateway_association_id, Main: false}
                ]
            },
            {
                RouteTableId: $route_table_two_id,
                Associations: [
                    {RouteTableAssociationId: $second_association_id, Main: false}
                ]
            }
        ]
    }')

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
            "aws ec2 describe-flow-logs"*)
                if [[ "$command" == *"Name=resource-id,Values=$VPC_ID"* ]]; then
                    echo "$FLOW_LOG_ID"
                else
                    record_stub_failure "$FLOW_LOG_STUB_FAILURE_LOG" \
                        "FlowLog stub did not recognize the VPC-scoped filter in command: $command"
                    exit 1
                fi
                ;;
            "aws ec2 describe-egress-only-internet-gateways"*)
                local query_argument_pattern='(^|[[:space:]])--query[[:space:]]+([^[:space:]]+)'
                local eoigw_query=""
                if [[ "$command" =~ $query_argument_pattern ]]; then
                    eoigw_query="${BASH_REMATCH[2]}"
                else
                    record_stub_failure "$EOIGW_STUB_FAILURE_LOG" \
                        "EOIGW stub did not find --query in command: $command"
                    exit 1
                fi

                local attachment_aware_query_pattern="^EgressOnlyInternetGateways\\[\\?Attachments\\[\\?VpcId=='([^']+)'\\]\\]\\.EgressOnlyInternetGatewayId$"
                local slot_zero_query_pattern="^EgressOnlyInternetGateways\\[\\?Attachments\\[0\\]\\.VpcId=='([^']+)'\\]\\.EgressOnlyInternetGatewayId$"
                local query_vpc_id
                if [[ "$eoigw_query" =~ $attachment_aware_query_pattern ]]; then
                    query_vpc_id="${BASH_REMATCH[1]}"
                    jq -r --arg vpc_id "$query_vpc_id" '
                        [
                            .EgressOnlyInternetGateways[]
                            | select(.Attachments | any(.VpcId == $vpc_id))
                            | .EgressOnlyInternetGatewayId
                        ]
                        | @tsv
                    ' <<< "$EOIGW_RESPONSE"
                elif [[ "$eoigw_query" =~ $slot_zero_query_pattern ]]; then
                    query_vpc_id="${BASH_REMATCH[1]}"
                    jq -r --arg vpc_id "$query_vpc_id" '
                        [
                            .EgressOnlyInternetGateways[]
                            | select(.Attachments[0].VpcId == $vpc_id)
                            | .EgressOnlyInternetGatewayId
                        ]
                        | @tsv
                    ' <<< "$EOIGW_RESPONSE"
                else
                    record_stub_failure "$EOIGW_STUB_FAILURE_LOG" \
                        "EOIGW stub did not recognize query '$eoigw_query' in command: $command"
                    exit 1
                fi
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
            "aws ec2 describe-route-tables --route-table-ids $MAIN_ROUTE_TABLE_ID"*)
                if [[ "$command" == *"Associations[?Main!=\`true\`].RouteTableAssociationId"* ]]; then
                    echo "$MAIN_ROUTE_TABLE_SUBNET_ASSOCIATION_ID"
                else
                    printf '%s\t%s\n' \
                        "$MAIN_ROUTE_TABLE_SUBNET_ASSOCIATION_ID" \
                        "$MAIN_ROUTE_TABLE_ASSOCIATION_ID"
                fi
                ;;
            "aws ec2 describe-route-tables --route-table-ids $ROUTE_TABLE_ONE_ID"*)
                printf '%s\t%s\n' \
                    "$SUBNET_ROUTE_TABLE_ASSOCIATION_ID" \
                    "$GATEWAY_ROUTE_TABLE_ASSOCIATION_ID"
                ;;
            "aws ec2 describe-route-tables --route-table-ids $ROUTE_TABLE_TWO_ID"*)
                printf '%s\n' "$SECOND_ROUTE_TABLE_ASSOCIATION_ID"
                ;;
            "aws ec2 describe-route-tables"*)
                local query_marker=' --query '
                local output_marker=' --output '
                local route_table_query=""
                if [[ "$command" == *"$query_marker"*"$output_marker"* ]]; then
                    route_table_query="${command#*"$query_marker"}"
                    route_table_query="${route_table_query%%"$output_marker"*}"
                else
                    record_stub_failure "$ROUTE_TABLE_STUB_FAILURE_LOG" \
                        "RouteTable stub did not find --query in command: $command"
                    exit 1
                fi

                local slot_zero_query="RouteTables[?Associations[0].Main!=\`true\`].RouteTableId"
                local any_slot_query="RouteTables[?length(not_null(Associations, \`[]\`)[?Main==\`true\`])==\`0\`].RouteTableId"
                local truthiness_collapsing_query="RouteTables[?!Associations[?Main==\`true\`]].RouteTableId"
                if [ "$route_table_query" = "$slot_zero_query" ]; then
                    jq -r '
                        [
                            .RouteTables[]
                            | select(.Associations[0].Main != true)
                            | .RouteTableId
                        ]
                        | @tsv
                    ' <<< "$ROUTE_TABLE_RESPONSE"
                elif [ "$route_table_query" = "$any_slot_query" ]; then
                    jq -r '
                        [
                            .RouteTables[]
                            | select(
                                ((.Associations // []) | map(select(.Main == true)) | length) == 0
                            )
                            | .RouteTableId
                        ]
                        | @tsv
                    ' <<< "$ROUTE_TABLE_RESPONSE"
                elif [ "$route_table_query" = "$truthiness_collapsing_query" ]; then
                    # JMESPath truthiness collapses this nested projection so it matches nothing.
                    jq -r '[] | @tsv' <<< "$ROUTE_TABLE_RESPONSE"
                else
                    record_stub_failure "$ROUTE_TABLE_STUB_FAILURE_LOG" \
                        "RouteTable stub did not recognize query '$route_table_query' in command: $command"
                    exit 1
                fi
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
: > "$FLOW_LOG_STUB_FAILURE_LOG"
: > "$EOIGW_STUB_FAILURE_LOG"
: > "$ROUTE_TABLE_STUB_FAILURE_LOG"
if ! deep_cleanup_account "ordering-account" >"$WORK_DIR/stdout.log" 2>"$WORK_DIR/stderr.log"; then
    fail "deep_cleanup_account returned non-zero for the successful ordering scenario"
fi

flow_log_stub_failed=0
while IFS= read -r stub_failure; do
    [ -z "$stub_failure" ] && continue
    flow_log_stub_failed=1
    fail "$stub_failure"
done < "$FLOW_LOG_STUB_FAILURE_LOG"

eoigw_stub_failed=0
while IFS= read -r stub_failure; do
    [ -z "$stub_failure" ] && continue
    eoigw_stub_failed=1
    fail "$stub_failure"
done < "$EOIGW_STUB_FAILURE_LOG"

while IFS= read -r stub_failure; do
    [ -z "$stub_failure" ] && continue
    fail "$stub_failure"
done < "$ROUTE_TABLE_STUB_FAILURE_LOG"

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

flow_log_enumeration=$(first_line_containing "aws ec2 describe-flow-logs")
flow_log_delete=$(first_line_containing "aws ec2 delete-flow-logs --flow-log-ids $FLOW_LOG_ID")
eoigw_enumeration=$(first_line_containing "aws ec2 describe-egress-only-internet-gateways")
eoigw_delete=$(first_line_containing "aws ec2 delete-egress-only-internet-gateway --egress-only-internet-gateway-id $MATCHING_EOIGW_ID")
vpc_delete=$(first_line_containing "aws ec2 delete-vpc --vpc-id $VPC_ID")
first_iam_role_call=$(first_line_containing "aws iam list-roles")
if [ "$flow_log_stub_failed" -eq 0 ]; then
    if [ -z "$flow_log_enumeration" ]; then
        fail "missing VPC-scoped flow-log enumeration"
    fi
    if [ -z "$flow_log_delete" ]; then
        fail "missing delete for the VPC's flow log"
    elif [ -z "$vpc_delete" ] || [ "$flow_log_delete" -ge "$vpc_delete" ]; then
        fail "flow log was not deleted before its VPC"
    fi
fi
if [ -z "$first_iam_role_call" ]; then
    fail "missing IAM role enumeration used as the flow-log dependency boundary"
elif [ -n "$flow_log_delete" ] && [ "$flow_log_delete" -ge "$first_iam_role_call" ]; then
    fail "flow log was not deleted before IAM role enumeration"
fi
if [ "$eoigw_stub_failed" -eq 0 ]; then
    if [ -z "$eoigw_enumeration" ]; then
        fail "missing egress-only internet gateway enumeration"
    fi
    if [ -z "$eoigw_delete" ]; then
        fail "missing delete for the VPC's egress-only internet gateway"
    elif [ -z "$vpc_delete" ] || [ "$eoigw_delete" -ge "$vpc_delete" ]; then
        fail "egress-only internet gateway was not deleted before its VPC"
    fi
fi
if grep -F "aws ec2 delete-egress-only-internet-gateway --egress-only-internet-gateway-id $UNRELATED_EOIGW_ID" "$COMMAND_LOG" >/dev/null; then
    fail "egress-only internet gateway attached only to another VPC must not be deleted"
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
first_route_table_teardown=$(first_line_containing "aws ec2 describe-route-tables --filters Name=vpc-id,Values=$VPC_ID")
if [ -n "$flow_log_delete" ]; then
    if [ -z "$first_subnet_delete" ] || [ "$flow_log_delete" -ge "$first_subnet_delete" ]; then
        fail "flow log was not deleted before subnet teardown"
    fi
    if [ -z "$first_route_table_teardown" ] || [ "$flow_log_delete" -ge "$first_route_table_teardown" ]; then
        fail "flow log was not deleted before route-table teardown"
    fi
fi
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

for route_table_id in "$ROUTE_TABLE_ONE_ID" "$ROUTE_TABLE_TWO_ID"; do
    association_enumeration_count=$(grep -cF "aws ec2 describe-route-tables --route-table-ids $route_table_id" "$COMMAND_LOG" || true)
    if [ "$association_enumeration_count" -ne 1 ]; then
        fail "expected one association enumeration for route table $route_table_id, got $association_enumeration_count"
    fi
done
main_route_table_association_enumeration_count=$(grep -cF "aws ec2 describe-route-tables --route-table-ids $MAIN_ROUTE_TABLE_ID" "$COMMAND_LOG" || true)
if [ "$main_route_table_association_enumeration_count" -ne 0 ]; then
    fail "main route table was selected for association enumeration"
fi

first_route_table_delete=$(first_line_containing "aws ec2 delete-route-table --route-table-id")
last_route_table_disassociation=""
for expected_disassociation in \
    "aws ec2 disassociate-route-table --association-id $SUBNET_ROUTE_TABLE_ASSOCIATION_ID" \
    "aws ec2 disassociate-route-table --association-id $GATEWAY_ROUTE_TABLE_ASSOCIATION_ID" \
    "aws ec2 disassociate-route-table --association-id $SECOND_ROUTE_TABLE_ASSOCIATION_ID"
do
    disassociation_line=$(first_line_containing "$expected_disassociation")
    if [ -z "$disassociation_line" ]; then
        fail "missing route-table disassociation: $expected_disassociation"
        continue
    fi
    if [ -z "$last_route_table_disassociation" ] || [ "$disassociation_line" -gt "$last_route_table_disassociation" ]; then
        last_route_table_disassociation="$disassociation_line"
    fi
done
for protected_association_id in \
    "$MAIN_ROUTE_TABLE_SUBNET_ASSOCIATION_ID" \
    "$MAIN_ROUTE_TABLE_ASSOCIATION_ID"
do
    if grep -Fx "aws ec2 disassociate-route-table --association-id $protected_association_id" "$COMMAND_LOG" >/dev/null; then
        fail "association on the main route table must not be disassociated: $protected_association_id"
    fi
done
if [ -n "$last_route_table_disassociation" ] && \
    { [ -z "$first_route_table_delete" ] || [ "$last_route_table_disassociation" -ge "$first_route_table_delete" ]; }; then
    fail "not every route-table disassociation ran before the first route-table delete"
fi
for route_table_id in "$ROUTE_TABLE_ONE_ID" "$ROUTE_TABLE_TWO_ID"; do
    route_table_delete=$(first_line_containing "aws ec2 delete-route-table --route-table-id $route_table_id")
    if [ -z "$route_table_delete" ]; then
        fail "missing route-table delete after disassociations: $route_table_id"
    fi
done
if grep -Fx "aws ec2 delete-route-table --route-table-id $MAIN_ROUTE_TABLE_ID" "$COMMAND_LOG" >/dev/null; then
    fail "main route table must not be deleted"
fi

if [ "$FAILURES" -ne 0 ]; then
    exit 1
fi

echo "deep_cleanup_dependency_ordering: OK"
