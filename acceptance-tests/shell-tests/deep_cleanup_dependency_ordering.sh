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
IPAM_STUB_FAILURE_LOG="$WORK_DIR/ipam-stub-failures.log"
IPAM_DELETE_LOG="$WORK_DIR/ipam-deletes.log"
KMS_STUB_FAILURE_LOG="$WORK_DIR/kms-stub-failures.log"
KMS_SCHEDULE_LOG="$WORK_DIR/kms-schedules.log"
CLOUDFRONT_STUB_FAILURE_LOG="$WORK_DIR/cloudfront-stub-failures.log"
CLOUDFRONT_DISTRIBUTION_DELETE_LOG="$WORK_DIR/cloudfront-distribution-deletes.log"
CLOUDFRONT_OAC_DELETE_LOG="$WORK_DIR/cloudfront-oac-deletes.log"

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
IPAM_ONE_ID="ipam-ordering-one"
IPAM_TWO_ID="ipam-ordering-two"
TRANSITIONING_IPAM_ID="ipam-ordering-transitioning"
MODIFYING_IPAM_ID="ipam-ordering-modifying"
DELETING_IPAM_ID="ipam-ordering-deleting"
DELETE_COMPLETE_IPAM_ID="ipam-ordering-delete-complete"
ISOLATING_IPAM_ID="ipam-ordering-isolating"
RESTORING_IPAM_ID="ipam-ordering-restoring"
MATCHING_EOIGW_ID="eigw-ordering"
UNRELATED_EOIGW_ID="eigw-other-vpc"
OTHER_VPC_ID="vpc-other"
KMS_AWS_MANAGED_KEY_ID="kms-aws-managed-tagged"
KMS_PENDING_DELETION_KEY_ID="kms-pending-deletion-tagged"
KMS_FOREIGN_TAG_KEY_ID="kms-foreign-tagged"
KMS_ACCESS_DENIED_KEY_ID="kms-access-denied"
KMS_ENABLED_KEY_ID="kms-enabled-tagged"
KMS_DISABLED_KEY_ID="kms-disabled-tagged"
KMS_PENDING_REPLICA_DELETION_KEY_ID="kms-pending-replica-deletion-tagged"
DISABLED_DEPLOYED_DISTRIBUTION_ID="E-DISABLED-DEPLOYED"
DISABLED_DEPLOYED_DISTRIBUTION_ETAG="ETAG-DISABLED-DEPLOYED"
DISABLED_IN_PROGRESS_DISTRIBUTION_ID="E-DISABLED-IN-PROGRESS"
DISABLED_IN_PROGRESS_DISTRIBUTION_ETAG="ETAG-DISABLED-IN-PROGRESS"
ENABLED_DEPLOYED_DISTRIBUTION_ID="E-ENABLED-DEPLOYED"
ENABLED_DEPLOYED_DISTRIBUTION_ETAG="ETAG-ENABLED-DEPLOYED"
ENABLED_IN_PROGRESS_DISTRIBUTION_ID="E-ENABLED-IN-PROGRESS"
ENABLED_IN_PROGRESS_DISTRIBUTION_ETAG="ETAG-ENABLED-IN-PROGRESS"
FIRST_OAC_ID="OAC-ACCEPTANCE-TEST"
FIRST_OAC_NAME="carina-acc-test-oac"
FIRST_OAC_ETAG="ETAG-OAC-ACCEPTANCE-TEST"
ENABLED_DISTRIBUTION_OAC_ID="OAC-ENABLED-DISTRIBUTION"
ENABLED_DISTRIBUTION_OAC_NAME="enabled-distribution-origin-control"
ENABLED_DISTRIBUTION_OAC_ETAG="ETAG-OAC-ENABLED-DISTRIBUTION"
DELETED_DISTRIBUTION_OAC_ID="OAC-DELETED-DISTRIBUTION"
DELETED_DISTRIBUTION_OAC_NAME="deleted-distribution-origin-control"
DELETED_DISTRIBUTION_OAC_ETAG="ETAG-OAC-DELETED-DISTRIBUTION"
GET_DENIED_OAC_ID="OAC-GET-DENIED"
GET_DENIED_OAC_NAME="carina-acc-denied-oac"
GET_DENIED_OAC_ETAG="ETAG-OAC-GET-DENIED"
ODDLY_NAMED_OAC_ID="OAC-ODDLY-NAMED"
ODDLY_NAMED_OAC_NAME="shared-assets-origin-control"
ODDLY_NAMED_OAC_ETAG="ETAG-OAC-ODDLY-NAMED"
KMS_FIXTURE=$(jq -cn \
    --arg aws_managed_key_id "$KMS_AWS_MANAGED_KEY_ID" \
    --arg pending_deletion_key_id "$KMS_PENDING_DELETION_KEY_ID" \
    --arg foreign_tag_key_id "$KMS_FOREIGN_TAG_KEY_ID" \
    --arg access_denied_key_id "$KMS_ACCESS_DENIED_KEY_ID" \
    --arg enabled_key_id "$KMS_ENABLED_KEY_ID" \
    --arg disabled_key_id "$KMS_DISABLED_KEY_ID" \
    --arg pending_replica_deletion_key_id "$KMS_PENDING_REPLICA_DELETION_KEY_ID" \
    '{
        Keys: [
            {
                KeyMetadata: {
                    KeyId: $aws_managed_key_id,
                    KeyArn: ("arn:aws:kms:us-east-1:123456789012:key/" + $aws_managed_key_id),
                    KeyManager: "AWS",
                    KeyState: "Enabled"
                },
                Tags: [{TagKey: "Environment", TagValue: "acceptance-test"}],
                TagReadDenied: false,
                ScheduleDenied: false
            },
            {
                KeyMetadata: {
                    KeyId: $pending_deletion_key_id,
                    KeyArn: ("arn:aws:kms:us-east-1:123456789012:key/" + $pending_deletion_key_id),
                    KeyManager: "CUSTOMER",
                    KeyState: "PendingDeletion"
                },
                Tags: [{TagKey: "Environment", TagValue: "acceptance-test"}],
                TagReadDenied: false,
                ScheduleDenied: false
            },
            {
                KeyMetadata: {
                    KeyId: $foreign_tag_key_id,
                    KeyArn: ("arn:aws:kms:us-east-1:123456789012:key/" + $foreign_tag_key_id),
                    KeyManager: "CUSTOMER",
                    KeyState: "Enabled"
                },
                Tags: [{TagKey: "Environment", TagValue: "foreign"}],
                TagReadDenied: false,
                ScheduleDenied: false
            },
            {
                KeyMetadata: {
                    KeyId: $access_denied_key_id,
                    KeyArn: ("arn:aws:kms:us-east-1:123456789012:key/" + $access_denied_key_id),
                    KeyManager: "CUSTOMER",
                    KeyState: "Enabled"
                },
                Tags: [{TagKey: "Environment", TagValue: "acceptance-test"}],
                TagReadDenied: true,
                ScheduleDenied: false
            },
            {
                KeyMetadata: {
                    KeyId: $enabled_key_id,
                    KeyArn: ("arn:aws:kms:us-east-1:123456789012:key/" + $enabled_key_id),
                    KeyManager: "CUSTOMER",
                    KeyState: "Enabled"
                },
                Tags: [{TagKey: "Environment", TagValue: "acceptance-test"}],
                TagReadDenied: false,
                ScheduleDenied: false
            },
            {
                KeyMetadata: {
                    KeyId: $disabled_key_id,
                    KeyArn: ("arn:aws:kms:us-east-1:123456789012:key/" + $disabled_key_id),
                    KeyManager: "CUSTOMER",
                    KeyState: "Disabled"
                },
                Tags: [{TagKey: "Environment", TagValue: "acceptance-test"}],
                TagReadDenied: false,
                ScheduleDenied: false
            },
            {
                KeyMetadata: {
                    KeyId: $pending_replica_deletion_key_id,
                    KeyArn: ("arn:aws:kms:us-east-1:123456789012:key/" + $pending_replica_deletion_key_id),
                    KeyManager: "CUSTOMER",
                    KeyState: "PendingReplicaDeletion"
                },
                Tags: [{TagKey: "Environment", TagValue: "acceptance-test"}],
                TagReadDenied: false,
                ScheduleDenied: false
            }
        ]
    }')
CLOUDFRONT_FIXTURE=$(jq -cn \
    --arg disabled_deployed_id "$DISABLED_DEPLOYED_DISTRIBUTION_ID" \
    --arg disabled_deployed_etag "$DISABLED_DEPLOYED_DISTRIBUTION_ETAG" \
    --arg disabled_in_progress_id "$DISABLED_IN_PROGRESS_DISTRIBUTION_ID" \
    --arg disabled_in_progress_etag "$DISABLED_IN_PROGRESS_DISTRIBUTION_ETAG" \
    --arg enabled_deployed_id "$ENABLED_DEPLOYED_DISTRIBUTION_ID" \
    --arg enabled_deployed_etag "$ENABLED_DEPLOYED_DISTRIBUTION_ETAG" \
    --arg enabled_in_progress_id "$ENABLED_IN_PROGRESS_DISTRIBUTION_ID" \
    --arg enabled_in_progress_etag "$ENABLED_IN_PROGRESS_DISTRIBUTION_ETAG" \
    --arg first_oac_id "$FIRST_OAC_ID" \
    --arg first_oac_name "$FIRST_OAC_NAME" \
    --arg first_oac_etag "$FIRST_OAC_ETAG" \
    --arg enabled_distribution_oac_id "$ENABLED_DISTRIBUTION_OAC_ID" \
    --arg enabled_distribution_oac_name "$ENABLED_DISTRIBUTION_OAC_NAME" \
    --arg enabled_distribution_oac_etag "$ENABLED_DISTRIBUTION_OAC_ETAG" \
    --arg deleted_distribution_oac_id "$DELETED_DISTRIBUTION_OAC_ID" \
    --arg deleted_distribution_oac_name "$DELETED_DISTRIBUTION_OAC_NAME" \
    --arg deleted_distribution_oac_etag "$DELETED_DISTRIBUTION_OAC_ETAG" \
    --arg get_denied_oac_id "$GET_DENIED_OAC_ID" \
    --arg get_denied_oac_name "$GET_DENIED_OAC_NAME" \
    --arg get_denied_oac_etag "$GET_DENIED_OAC_ETAG" \
    --arg oddly_named_oac_id "$ODDLY_NAMED_OAC_ID" \
    --arg oddly_named_oac_name "$ODDLY_NAMED_OAC_NAME" \
    --arg oddly_named_oac_etag "$ODDLY_NAMED_OAC_ETAG" \
    '{
        Distributions: [
            {
                Id: $disabled_deployed_id,
                Status: "Deployed",
                Enabled: false,
                ETag: $disabled_deployed_etag,
                Origins: {Items: [
                    {OriginAccessControlId: ""},
                    {OriginAccessControlId: $deleted_distribution_oac_id}
                ]}
            },
            {
                Id: $disabled_in_progress_id,
                Status: "InProgress",
                Enabled: false,
                ETag: $disabled_in_progress_etag,
                Origins: {Items: [{OriginAccessControlId: ""}]}
            },
            {
                Id: $enabled_deployed_id,
                Status: "Deployed",
                Enabled: true,
                ETag: $enabled_deployed_etag,
                Origins: {Items: [{OriginAccessControlId: $enabled_distribution_oac_id}]}
            },
            {
                Id: $enabled_in_progress_id,
                Status: "InProgress",
                Enabled: true,
                ETag: $enabled_in_progress_etag,
                Origins: {Items: [{OriginAccessControlId: ""}]}
            }
        ],
        OriginAccessControls: [
            {Id: $first_oac_id, Name: $first_oac_name, ETag: $first_oac_etag, GetDenied: false},
            {
                Id: $enabled_distribution_oac_id,
                Name: $enabled_distribution_oac_name,
                ETag: $enabled_distribution_oac_etag,
                GetDenied: false
            },
            {
                Id: $deleted_distribution_oac_id,
                Name: $deleted_distribution_oac_name,
                ETag: $deleted_distribution_oac_etag,
                GetDenied: false
            },
            {Id: $get_denied_oac_id, Name: $get_denied_oac_name, ETag: $get_denied_oac_etag, GetDenied: true},
            {Id: $oddly_named_oac_id, Name: $oddly_named_oac_name, ETag: $oddly_named_oac_etag, GetDenied: false}
        ]
    }')
CLOUDFRONT_IN_USE_ENUMERATION_DENIED=0
KMS_LIST_KEYS_RESPONSE=$(jq -c \
    '{Keys: [.Keys[].KeyMetadata | {KeyId: .KeyId, KeyArn: .KeyArn}]}' \
    <<< "$KMS_FIXTURE")
IPAM_RESPONSE=$(jq -cn \
    --arg ipam_one_id "$IPAM_ONE_ID" \
    --arg ipam_two_id "$IPAM_TWO_ID" \
    --arg transitioning_ipam_id "$TRANSITIONING_IPAM_ID" \
    --arg modifying_ipam_id "$MODIFYING_IPAM_ID" \
    --arg deleting_ipam_id "$DELETING_IPAM_ID" \
    --arg delete_complete_ipam_id "$DELETE_COMPLETE_IPAM_ID" \
    --arg isolating_ipam_id "$ISOLATING_IPAM_ID" \
    --arg restoring_ipam_id "$RESTORING_IPAM_ID" \
    '{
        Ipams: [
            {IpamId: $ipam_one_id, State: "create-complete"},
            {IpamId: $ipam_two_id, State: "delete-failed"},
            {IpamId: $transitioning_ipam_id, State: "create-in-progress"},
            {IpamId: $modifying_ipam_id, State: "modify-in-progress"},
            {IpamId: $deleting_ipam_id, State: "delete-in-progress"},
            {IpamId: $delete_complete_ipam_id, State: "delete-complete"},
            {IpamId: $isolating_ipam_id, State: "isolate-in-progress"},
            {IpamId: $restoring_ipam_id, State: "restore-in-progress"}
        ]
    }')
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
            "aws cloudfront list-distributions"*)
                local distribution_arguments=("$@")
                local distribution_query=""
                local distribution_output=""
                local argument_index=3
                while [ "$argument_index" -lt "${#distribution_arguments[@]}" ]; do
                    case "${distribution_arguments[$argument_index]}" in
                        --query)
                            argument_index=$((argument_index + 1))
                            if [ "$argument_index" -ge "${#distribution_arguments[@]}" ] || [ -n "$distribution_query" ]; then
                                record_stub_failure "$CLOUDFRONT_STUB_FAILURE_LOG" \
                                    "CloudFront list-distributions stub found a missing or duplicate --query in command: $command"
                                exit 1
                            fi
                            distribution_query="${distribution_arguments[$argument_index]}"
                            ;;
                        --output)
                            argument_index=$((argument_index + 1))
                            if [ "$argument_index" -ge "${#distribution_arguments[@]}" ] || [ -n "$distribution_output" ]; then
                                record_stub_failure "$CLOUDFRONT_STUB_FAILURE_LOG" \
                                    "CloudFront list-distributions stub found a missing or duplicate --output in command: $command"
                                exit 1
                            fi
                            distribution_output="${distribution_arguments[$argument_index]}"
                            ;;
                        *)
                            record_stub_failure "$CLOUDFRONT_STUB_FAILURE_LOG" \
                                "CloudFront list-distributions stub did not recognize argument '${distribution_arguments[$argument_index]}' in command: $command"
                            exit 1
                            ;;
                    esac
                    argument_index=$((argument_index + 1))
                done
                if [ -z "$distribution_query" ] || [ "$distribution_output" != "text" ]; then
                    record_stub_failure "$CLOUDFRONT_STUB_FAILURE_LOG" \
                        "CloudFront list-distributions stub requires --query and --output text in command: $command"
                    exit 1
                fi

                local distribution_state_query_pattern='^DistributionList\.Items\[\?(.+)\]\.\[Id,[[:space:]]*ETag\]$'
                local referenced_oac_query_pattern='^DistributionList\.Items\[\]\.Origins\.Items\[\?(.+)\]\.OriginAccessControlId$'
                if [[ "$distribution_query" =~ $referenced_oac_query_pattern ]]; then
                    local referenced_oac_filter="${BASH_REMATCH[1]}"
                    local non_empty_oac_condition_pattern="^OriginAccessControlId[[:space:]]*!=[[:space:]]*''$"
                    if [[ ! "$referenced_oac_filter" =~ $non_empty_oac_condition_pattern ]]; then
                        record_stub_failure "$CLOUDFRONT_STUB_FAILURE_LOG" \
                            "CloudFront list-distributions stub could not interpret reference condition '$referenced_oac_filter' in query '$distribution_query' from command: $command"
                        exit 1
                    fi
                    if [ "$CLOUDFRONT_IN_USE_ENUMERATION_DENIED" -eq 1 ]; then
                        echo "An error occurred (AccessDenied) when calling the ListDistributions operation for the OAC reference snapshot" >&2
                        exit 254
                    fi
                    jq -r \
                        '[
                            .Distributions[].Origins.Items[]
                            | select(.OriginAccessControlId != "")
                            | .OriginAccessControlId
                        ]
                        | @tsv' <<< "$CLOUDFRONT_FIXTURE"
                elif [[ "$distribution_query" =~ $distribution_state_query_pattern ]]; then
                    local distribution_filter="${BASH_REMATCH[1]}"
                    local enabled_condition_pattern="^Enabled==\`(true|false)\`$"
                    local status_condition_pattern="^Status==\`([^\`]*)\`$"
                    local distribution_conditions="${distribution_filter// && /$'\n'}"
                    local filtered_enabled=""
                    local filtered_status=""
                    local enabled_condition_seen=0
                    local status_condition_seen=0
                    local distribution_condition
                    while IFS= read -r distribution_condition; do
                        if [[ "$distribution_condition" =~ $enabled_condition_pattern ]]; then
                            if [ "$enabled_condition_seen" -eq 1 ]; then
                                record_stub_failure "$CLOUDFRONT_STUB_FAILURE_LOG" \
                                    "CloudFront list-distributions stub found duplicate Enabled conditions in query '$distribution_query' from command: $command"
                                exit 1
                            fi
                            enabled_condition_seen=1
                            filtered_enabled="${BASH_REMATCH[1]}"
                        elif [[ "$distribution_condition" =~ $status_condition_pattern ]]; then
                            if [ "$status_condition_seen" -eq 1 ]; then
                                record_stub_failure "$CLOUDFRONT_STUB_FAILURE_LOG" \
                                    "CloudFront list-distributions stub found duplicate Status conditions in query '$distribution_query' from command: $command"
                                exit 1
                            fi
                            status_condition_seen=1
                            filtered_status="${BASH_REMATCH[1]}"
                        else
                            record_stub_failure "$CLOUDFRONT_STUB_FAILURE_LOG" \
                                "CloudFront list-distributions stub could not interpret condition '$distribution_condition' in query '$distribution_query' from command: $command"
                            exit 1
                        fi
                    done <<< "$distribution_conditions"

                    jq -r \
                        --arg enabled "$filtered_enabled" \
                        --arg status "$filtered_status" \
                        '[
                            .Distributions[]
                            | select(
                                ($enabled == "" or (.Enabled | tostring) == $enabled)
                                and ($status == "" or .Status == $status)
                            )
                            | [.Id, .ETag]
                            | @tsv
                        ]
                        | .[]' <<< "$CLOUDFRONT_FIXTURE"
                else
                    record_stub_failure "$CLOUDFRONT_STUB_FAILURE_LOG" \
                        "CloudFront list-distributions stub did not recognize query '$distribution_query' in command: $command"
                    exit 1
                fi
                ;;
            "aws cloudfront delete-distribution"*)
                local distribution_delete_arguments=("$@")
                local deleted_distribution_id=""
                local deleted_distribution_etag=""
                local argument_index=3
                while [ "$argument_index" -lt "${#distribution_delete_arguments[@]}" ]; do
                    case "${distribution_delete_arguments[$argument_index]}" in
                        --id)
                            argument_index=$((argument_index + 1))
                            if [ "$argument_index" -ge "${#distribution_delete_arguments[@]}" ] || [ -n "$deleted_distribution_id" ]; then
                                record_stub_failure "$CLOUDFRONT_STUB_FAILURE_LOG" \
                                    "CloudFront delete-distribution stub found a missing or duplicate --id in command: $command"
                                exit 1
                            fi
                            deleted_distribution_id="${distribution_delete_arguments[$argument_index]}"
                            ;;
                        --if-match)
                            argument_index=$((argument_index + 1))
                            if [ "$argument_index" -ge "${#distribution_delete_arguments[@]}" ] || [ -n "$deleted_distribution_etag" ]; then
                                record_stub_failure "$CLOUDFRONT_STUB_FAILURE_LOG" \
                                    "CloudFront delete-distribution stub found a missing or duplicate --if-match in command: $command"
                                exit 1
                            fi
                            deleted_distribution_etag="${distribution_delete_arguments[$argument_index]}"
                            ;;
                        *)
                            record_stub_failure "$CLOUDFRONT_STUB_FAILURE_LOG" \
                                "CloudFront delete-distribution stub did not recognize argument '${distribution_delete_arguments[$argument_index]}' in command: $command"
                            exit 1
                            ;;
                    esac
                    argument_index=$((argument_index + 1))
                done
                if [ -z "$deleted_distribution_id" ] || [ -z "$deleted_distribution_etag" ]; then
                    record_stub_failure "$CLOUDFRONT_STUB_FAILURE_LOG" \
                        "CloudFront delete-distribution stub requires --id and --if-match in command: $command"
                    exit 1
                fi
                local distribution_fixture_count
                distribution_fixture_count=$(jq -r --arg id "$deleted_distribution_id" \
                    '[.Distributions[] | select(.Id == $id)] | length' <<< "$CLOUDFRONT_FIXTURE")
                if [ "$distribution_fixture_count" -ne 1 ]; then
                    record_stub_failure "$CLOUDFRONT_STUB_FAILURE_LOG" \
                        "CloudFront delete-distribution stub saw unknown or duplicate distribution ID '$deleted_distribution_id' in command: $command"
                    exit 1
                fi
                printf '%s\t%s\n' "$deleted_distribution_id" "$deleted_distribution_etag" >> "$CLOUDFRONT_DISTRIBUTION_DELETE_LOG"
                local expected_distribution_etag
                expected_distribution_etag=$(jq -r --arg id "$deleted_distribution_id" \
                    '.Distributions[] | select(.Id == $id) | .ETag' <<< "$CLOUDFRONT_FIXTURE")
                if [ "$deleted_distribution_etag" != "$expected_distribution_etag" ]; then
                    echo "An error occurred (PreconditionFailed) when calling the DeleteDistribution operation for distribution $deleted_distribution_id: stale ETag" >&2
                    exit 254
                fi
                ;;
            "aws cloudfront list-origin-access-controls"*)
                local oac_list_arguments=("$@")
                local oac_list_query=""
                local oac_list_output=""
                local argument_index=3
                while [ "$argument_index" -lt "${#oac_list_arguments[@]}" ]; do
                    case "${oac_list_arguments[$argument_index]}" in
                        --query)
                            argument_index=$((argument_index + 1))
                            if [ "$argument_index" -ge "${#oac_list_arguments[@]}" ] || [ -n "$oac_list_query" ]; then
                                record_stub_failure "$CLOUDFRONT_STUB_FAILURE_LOG" \
                                    "CloudFront list-origin-access-controls stub found a missing or duplicate --query in command: $command"
                                exit 1
                            fi
                            oac_list_query="${oac_list_arguments[$argument_index]}"
                            ;;
                        --output)
                            argument_index=$((argument_index + 1))
                            if [ "$argument_index" -ge "${#oac_list_arguments[@]}" ] || [ -n "$oac_list_output" ]; then
                                record_stub_failure "$CLOUDFRONT_STUB_FAILURE_LOG" \
                                    "CloudFront list-origin-access-controls stub found a missing or duplicate --output in command: $command"
                                exit 1
                            fi
                            oac_list_output="${oac_list_arguments[$argument_index]}"
                            ;;
                        *)
                            record_stub_failure "$CLOUDFRONT_STUB_FAILURE_LOG" \
                                "CloudFront list-origin-access-controls stub did not recognize argument '${oac_list_arguments[$argument_index]}' in command: $command"
                            exit 1
                            ;;
                    esac
                    argument_index=$((argument_index + 1))
                done
                if [ -z "$oac_list_query" ] || [ "$oac_list_output" != "text" ]; then
                    record_stub_failure "$CLOUDFRONT_STUB_FAILURE_LOG" \
                        "CloudFront list-origin-access-controls stub requires --query and --output text in command: $command"
                    exit 1
                fi

                local all_oac_ids_query_pattern='^OriginAccessControlList\.Items\[\*\]\.Id$'
                if [[ "$oac_list_query" =~ $all_oac_ids_query_pattern ]]; then
                    jq -r '[.OriginAccessControls[].Id] | @tsv' <<< "$CLOUDFRONT_FIXTURE"
                else
                    record_stub_failure "$CLOUDFRONT_STUB_FAILURE_LOG" \
                        "CloudFront list-origin-access-controls stub did not recognize query '$oac_list_query' in command: $command"
                    exit 1
                fi
                ;;
            "aws cloudfront get-origin-access-control"*)
                local oac_get_arguments=("$@")
                local fetched_oac_id=""
                local oac_get_query=""
                local oac_get_output=""
                local argument_index=3
                while [ "$argument_index" -lt "${#oac_get_arguments[@]}" ]; do
                    case "${oac_get_arguments[$argument_index]}" in
                        --id)
                            argument_index=$((argument_index + 1))
                            if [ "$argument_index" -ge "${#oac_get_arguments[@]}" ] || [ -n "$fetched_oac_id" ]; then
                                record_stub_failure "$CLOUDFRONT_STUB_FAILURE_LOG" \
                                    "CloudFront get-origin-access-control stub found a missing or duplicate --id in command: $command"
                                exit 1
                            fi
                            fetched_oac_id="${oac_get_arguments[$argument_index]}"
                            ;;
                        --query)
                            argument_index=$((argument_index + 1))
                            if [ "$argument_index" -ge "${#oac_get_arguments[@]}" ] || [ -n "$oac_get_query" ]; then
                                record_stub_failure "$CLOUDFRONT_STUB_FAILURE_LOG" \
                                    "CloudFront get-origin-access-control stub found a missing or duplicate --query in command: $command"
                                exit 1
                            fi
                            oac_get_query="${oac_get_arguments[$argument_index]}"
                            ;;
                        --output)
                            argument_index=$((argument_index + 1))
                            if [ "$argument_index" -ge "${#oac_get_arguments[@]}" ] || [ -n "$oac_get_output" ]; then
                                record_stub_failure "$CLOUDFRONT_STUB_FAILURE_LOG" \
                                    "CloudFront get-origin-access-control stub found a missing or duplicate --output in command: $command"
                                exit 1
                            fi
                            oac_get_output="${oac_get_arguments[$argument_index]}"
                            ;;
                        *)
                            record_stub_failure "$CLOUDFRONT_STUB_FAILURE_LOG" \
                                "CloudFront get-origin-access-control stub did not recognize argument '${oac_get_arguments[$argument_index]}' in command: $command"
                            exit 1
                            ;;
                    esac
                    argument_index=$((argument_index + 1))
                done
                if [ -z "$fetched_oac_id" ] || [ "$oac_get_query" != "ETag" ] || [ "$oac_get_output" != "text" ]; then
                    record_stub_failure "$CLOUDFRONT_STUB_FAILURE_LOG" \
                        "CloudFront get-origin-access-control stub requires --id, --query ETag, and --output text in command: $command"
                    exit 1
                fi
                local oac_fixture_count
                oac_fixture_count=$(jq -r --arg id "$fetched_oac_id" \
                    '[.OriginAccessControls[] | select(.Id == $id)] | length' <<< "$CLOUDFRONT_FIXTURE")
                if [ "$oac_fixture_count" -ne 1 ]; then
                    record_stub_failure "$CLOUDFRONT_STUB_FAILURE_LOG" \
                        "CloudFront get-origin-access-control stub saw unknown or duplicate OAC ID '$fetched_oac_id' in command: $command"
                    exit 1
                fi
                if [ "$(jq -r --arg id "$fetched_oac_id" \
                    '.OriginAccessControls[] | select(.Id == $id) | .GetDenied' <<< "$CLOUDFRONT_FIXTURE")" = "true" ]; then
                    echo "An error occurred (AccessDenied) when calling the GetOriginAccessControl operation for OAC $fetched_oac_id" >&2
                    exit 254
                fi
                jq -r --arg id "$fetched_oac_id" \
                    '.OriginAccessControls[] | select(.Id == $id) | .ETag' <<< "$CLOUDFRONT_FIXTURE"
                ;;
            "aws cloudfront delete-origin-access-control"*)
                local oac_delete_arguments=("$@")
                local deleted_oac_id=""
                local deleted_oac_etag=""
                local argument_index=3
                while [ "$argument_index" -lt "${#oac_delete_arguments[@]}" ]; do
                    case "${oac_delete_arguments[$argument_index]}" in
                        --id)
                            argument_index=$((argument_index + 1))
                            if [ "$argument_index" -ge "${#oac_delete_arguments[@]}" ] || [ -n "$deleted_oac_id" ]; then
                                record_stub_failure "$CLOUDFRONT_STUB_FAILURE_LOG" \
                                    "CloudFront delete-origin-access-control stub found a missing or duplicate --id in command: $command"
                                exit 1
                            fi
                            deleted_oac_id="${oac_delete_arguments[$argument_index]}"
                            ;;
                        --if-match)
                            argument_index=$((argument_index + 1))
                            if [ "$argument_index" -ge "${#oac_delete_arguments[@]}" ] || [ -n "$deleted_oac_etag" ]; then
                                record_stub_failure "$CLOUDFRONT_STUB_FAILURE_LOG" \
                                    "CloudFront delete-origin-access-control stub found a missing or duplicate --if-match in command: $command"
                                exit 1
                            fi
                            deleted_oac_etag="${oac_delete_arguments[$argument_index]}"
                            ;;
                        *)
                            record_stub_failure "$CLOUDFRONT_STUB_FAILURE_LOG" \
                                "CloudFront delete-origin-access-control stub did not recognize argument '${oac_delete_arguments[$argument_index]}' in command: $command"
                            exit 1
                            ;;
                    esac
                    argument_index=$((argument_index + 1))
                done
                if [ -z "$deleted_oac_id" ] || [ -z "$deleted_oac_etag" ]; then
                    record_stub_failure "$CLOUDFRONT_STUB_FAILURE_LOG" \
                        "CloudFront delete-origin-access-control stub requires --id and --if-match in command: $command"
                    exit 1
                fi
                local oac_fixture_count
                oac_fixture_count=$(jq -r --arg id "$deleted_oac_id" \
                    '[.OriginAccessControls[] | select(.Id == $id)] | length' <<< "$CLOUDFRONT_FIXTURE")
                if [ "$oac_fixture_count" -ne 1 ]; then
                    record_stub_failure "$CLOUDFRONT_STUB_FAILURE_LOG" \
                        "CloudFront delete-origin-access-control stub saw unknown or duplicate OAC ID '$deleted_oac_id' in command: $command"
                    exit 1
                fi
                printf '%s\t%s\n' "$deleted_oac_id" "$deleted_oac_etag" >> "$CLOUDFRONT_OAC_DELETE_LOG"
                local expected_oac_etag
                expected_oac_etag=$(jq -r --arg id "$deleted_oac_id" \
                    '.OriginAccessControls[] | select(.Id == $id) | .ETag' <<< "$CLOUDFRONT_FIXTURE")
                if [ "$deleted_oac_etag" != "$expected_oac_etag" ]; then
                    echo "An error occurred (PreconditionFailed) when calling the DeleteOriginAccessControl operation for OAC $deleted_oac_id: stale ETag" >&2
                    exit 254
                fi
                ;;
            "aws cloudfront "*)
                record_stub_failure "$CLOUDFRONT_STUB_FAILURE_LOG" \
                    "CloudFront stub did not recognize command: $command"
                exit 1
                ;;
            "aws kms list-keys"*)
                local query_marker=' --query '
                local output_marker=' --output '
                local kms_list_query=""
                local kms_list_output=""
                if [[ "$command" == *"$query_marker"*"$output_marker"* ]]; then
                    kms_list_query="${command#*"$query_marker"}"
                    kms_list_query="${kms_list_query%%"$output_marker"*}"
                    kms_list_output="${command##*"$output_marker"}"
                else
                    record_stub_failure "$KMS_STUB_FAILURE_LOG" \
                        "KMS list-keys stub did not find --query and --output in command: $command"
                    exit 1
                fi

                local key_id_projection_pattern='^Keys\[(\*)?\]\.KeyId$'
                if [ "$kms_list_output" = "text" ] && [[ "$kms_list_query" =~ $key_id_projection_pattern ]]; then
                    jq -r '[.Keys[] | .KeyId] | @tsv' <<< "$KMS_LIST_KEYS_RESPONSE"
                else
                    record_stub_failure "$KMS_STUB_FAILURE_LOG" \
                        "KMS list-keys stub did not recognize query '$kms_list_query' or output '$kms_list_output' in command: $command"
                    exit 1
                fi
                ;;
            "aws kms describe-key"*)
                local query_marker=' --query '
                local output_marker=' --output '
                local describe_prefix='aws kms describe-key --key-id '
                local describe_before_query="${command%%"$query_marker"*}"
                local kms_describe_query=""
                local kms_describe_output=""
                local described_key_id=""
                if [[ "$command" == *"$query_marker"*"$output_marker"* ]] && \
                    [[ "$describe_before_query" == "$describe_prefix"* ]]; then
                    described_key_id="${describe_before_query#"$describe_prefix"}"
                    kms_describe_query="${command#*"$query_marker"}"
                    kms_describe_query="${kms_describe_query%%"$output_marker"*}"
                    kms_describe_output="${command##*"$output_marker"}"
                else
                    record_stub_failure "$KMS_STUB_FAILURE_LOG" \
                        "KMS describe-key stub did not recognize command options: $command"
                    exit 1
                fi
                if [ -z "$described_key_id" ] || [[ "$described_key_id" == *[[:space:]]* ]] || \
                    [ "$kms_describe_output" != "text" ]; then
                    record_stub_failure "$KMS_STUB_FAILURE_LOG" \
                        "KMS describe-key stub found an invalid key ID or output in command: $command"
                    exit 1
                fi

                local metadata_query_pattern='^\[KeyMetadata\][[:space:]]*\|[[:space:]]*\[\?(.+)\]\.KeyId$'
                local manager_and_state_pattern="^KeyManager==\`([[:upper:]]+)\`[[:space:]]*&&[[:space:]]*contains\\(\\[(.*)\\],[[:space:]]*KeyState\\)$"
                local state_only_pattern='^contains\(\[(.*)\],[[:space:]]*KeyState\)$'
                local metadata_filter=""
                local filtered_key_manager=""
                local state_literals=""
                if [[ "$kms_describe_query" =~ $metadata_query_pattern ]]; then
                    metadata_filter="${BASH_REMATCH[1]}"
                else
                    record_stub_failure "$KMS_STUB_FAILURE_LOG" \
                        "KMS describe-key stub did not recognize query '$kms_describe_query' in command: $command"
                    exit 1
                fi
                if [[ "$metadata_filter" =~ $manager_and_state_pattern ]]; then
                    filtered_key_manager="${BASH_REMATCH[1]}"
                    state_literals="${BASH_REMATCH[2]}"
                elif [[ "$metadata_filter" =~ $state_only_pattern ]]; then
                    state_literals="${BASH_REMATCH[1]}"
                else
                    record_stub_failure "$KMS_STUB_FAILURE_LOG" \
                        "KMS describe-key stub could not interpret filter '$metadata_filter' in command: $command"
                    exit 1
                fi

                local state_literals_pattern="^\`[[:alpha:]]+\`([[:space:]]*,[[:space:]]*\`[[:alpha:]]+\`)*$"
                if ! [[ "$state_literals" =~ $state_literals_pattern ]]; then
                    record_stub_failure "$KMS_STUB_FAILURE_LOG" \
                        "KMS describe-key stub could not parse state literals in query '$kms_describe_query' from command: $command"
                    exit 1
                fi
                local state_csv="${state_literals//\`/}"
                state_csv="${state_csv// /}"
                local filtered_key_states=()
                IFS=',' read -r -a filtered_key_states <<< "$state_csv"
                local filtered_key_state
                for filtered_key_state in "${filtered_key_states[@]}"; do
                    case "$filtered_key_state" in
                        Enabled | Disabled | PendingDeletion | PendingReplicaDeletion | \
                            PendingImport | Unavailable | Creating | Updating)
                            ;;
                        *)
                            record_stub_failure "$KMS_STUB_FAILURE_LOG" \
                                "KMS describe-key stub did not recognize state '$filtered_key_state' in query '$kms_describe_query' from command: $command"
                            exit 1
                            ;;
                    esac
                done

                local kms_metadata
                kms_metadata=$(jq -c --arg key_id "$described_key_id" '
                    [.Keys[] | select(.KeyMetadata.KeyId == $key_id)]
                    | if length == 1 then .[0].KeyMetadata else empty end
                ' <<< "$KMS_FIXTURE")
                if [ -z "$kms_metadata" ]; then
                    record_stub_failure "$KMS_STUB_FAILURE_LOG" \
                        "KMS describe-key stub saw unknown or duplicate key ID '$described_key_id' in command: $command"
                    exit 1
                fi

                local fixture_key_manager
                local fixture_key_state
                fixture_key_manager=$(jq -r '.KeyManager' <<< "$kms_metadata")
                fixture_key_state=$(jq -r '.KeyState' <<< "$kms_metadata")
                local state_matched=0
                for filtered_key_state in "${filtered_key_states[@]}"; do
                    if [ "$fixture_key_state" = "$filtered_key_state" ]; then
                        state_matched=1
                        break
                    fi
                done
                if { [ -z "$filtered_key_manager" ] || [ "$fixture_key_manager" = "$filtered_key_manager" ]; } && \
                    [ "$state_matched" -eq 1 ]; then
                    echo "$described_key_id"
                fi
                ;;
            "aws kms list-resource-tags"*)
                local query_marker=' --query '
                local output_marker=' --output '
                local tags_prefix='aws kms list-resource-tags --key-id '
                local tags_before_query="${command%%"$query_marker"*}"
                local kms_tags_query=""
                local kms_tags_output=""
                local tagged_key_id=""
                if [[ "$command" == *"$query_marker"*"$output_marker"* ]] && \
                    [[ "$tags_before_query" == "$tags_prefix"* ]]; then
                    tagged_key_id="${tags_before_query#"$tags_prefix"}"
                    kms_tags_query="${command#*"$query_marker"}"
                    kms_tags_query="${kms_tags_query%%"$output_marker"*}"
                    kms_tags_output="${command##*"$output_marker"}"
                else
                    record_stub_failure "$KMS_STUB_FAILURE_LOG" \
                        "KMS list-resource-tags stub did not recognize command options: $command"
                    exit 1
                fi
                if [ -z "$tagged_key_id" ] || [[ "$tagged_key_id" == *[[:space:]]* ]] || \
                    [ "$kms_tags_output" != "text" ]; then
                    record_stub_failure "$KMS_STUB_FAILURE_LOG" \
                        "KMS list-resource-tags stub found an invalid key ID or output in command: $command"
                    exit 1
                fi

                local full_tag_query_pattern="^Tags\\[\\?TagKey==\`([^\`]*)\`[[:space:]]*&&[[:space:]]*TagValue==\`([^\`]*)\`\\]\\.TagKey$"
                local key_only_tag_query_pattern="^Tags\\[\\?TagKey==\`([^\`]*)\`\\]\\.TagKey$"
                local all_tags_query_pattern='^Tags\[(\*)?\]\.TagKey$'
                local tag_query_mode=""
                local filtered_tag_key=""
                local filtered_tag_value=""
                if [[ "$kms_tags_query" =~ $full_tag_query_pattern ]]; then
                    tag_query_mode="key-and-value"
                    filtered_tag_key="${BASH_REMATCH[1]}"
                    filtered_tag_value="${BASH_REMATCH[2]}"
                elif [[ "$kms_tags_query" =~ $key_only_tag_query_pattern ]]; then
                    tag_query_mode="key-only"
                    filtered_tag_key="${BASH_REMATCH[1]}"
                elif [[ "$kms_tags_query" =~ $all_tags_query_pattern ]]; then
                    tag_query_mode="all"
                else
                    record_stub_failure "$KMS_STUB_FAILURE_LOG" \
                        "KMS list-resource-tags stub did not recognize query '$kms_tags_query' in command: $command"
                    exit 1
                fi

                local kms_tag_fixture
                kms_tag_fixture=$(jq -c --arg key_id "$tagged_key_id" '
                    [.Keys[] | select(.KeyMetadata.KeyId == $key_id)]
                    | if length == 1 then .[0] else empty end
                ' <<< "$KMS_FIXTURE")
                if [ -z "$kms_tag_fixture" ]; then
                    record_stub_failure "$KMS_STUB_FAILURE_LOG" \
                        "KMS list-resource-tags stub saw unknown or duplicate key ID '$tagged_key_id' in command: $command"
                    exit 1
                fi
                if [ "$(jq -r '.TagReadDenied' <<< "$kms_tag_fixture")" = "true" ]; then
                    echo "An error occurred (AccessDeniedException) when calling the ListResourceTags operation for key $tagged_key_id" >&2
                    exit 254
                fi

                case "$tag_query_mode" in
                    key-and-value)
                        jq -r --arg tag_key "$filtered_tag_key" --arg tag_value "$filtered_tag_value" '
                            [.Tags[] | select(.TagKey == $tag_key and .TagValue == $tag_value) | .TagKey]
                            | @tsv
                        ' <<< "$kms_tag_fixture"
                        ;;
                    key-only)
                        jq -r --arg tag_key "$filtered_tag_key" '
                            [.Tags[] | select(.TagKey == $tag_key) | .TagKey]
                            | @tsv
                        ' <<< "$kms_tag_fixture"
                        ;;
                    all)
                        jq -r '[.Tags[].TagKey] | @tsv' <<< "$kms_tag_fixture"
                        ;;
                esac
                ;;
            "aws kms schedule-key-deletion"*)
                local kms_schedule_arguments_text="${command#aws kms schedule-key-deletion}"
                local kms_schedule_arguments=()
                read -r -a kms_schedule_arguments <<< "$kms_schedule_arguments_text"
                local scheduled_key_id=""
                local pending_window_in_days=""
                local argument_index=0
                while [ "$argument_index" -lt "${#kms_schedule_arguments[@]}" ]; do
                    case "${kms_schedule_arguments[$argument_index]}" in
                        --key-id)
                            argument_index=$((argument_index + 1))
                            if [ "$argument_index" -ge "${#kms_schedule_arguments[@]}" ] || [ -n "$scheduled_key_id" ]; then
                                record_stub_failure "$KMS_STUB_FAILURE_LOG" \
                                    "KMS schedule-key-deletion stub found a missing or duplicate --key-id in command: $command"
                                exit 1
                            fi
                            scheduled_key_id="${kms_schedule_arguments[$argument_index]}"
                            ;;
                        --pending-window-in-days)
                            argument_index=$((argument_index + 1))
                            if [ "$argument_index" -ge "${#kms_schedule_arguments[@]}" ] || [ -n "$pending_window_in_days" ]; then
                                record_stub_failure "$KMS_STUB_FAILURE_LOG" \
                                    "KMS schedule-key-deletion stub found a missing or duplicate --pending-window-in-days in command: $command"
                                exit 1
                            fi
                            pending_window_in_days="${kms_schedule_arguments[$argument_index]}"
                            ;;
                        *)
                            record_stub_failure "$KMS_STUB_FAILURE_LOG" \
                                "KMS schedule-key-deletion stub did not recognize argument '${kms_schedule_arguments[$argument_index]}' in command: $command"
                            exit 1
                            ;;
                    esac
                    argument_index=$((argument_index + 1))
                done
                if [ -z "$scheduled_key_id" ] || ! [[ "$pending_window_in_days" =~ ^[[:digit:]]+$ ]]; then
                    record_stub_failure "$KMS_STUB_FAILURE_LOG" \
                        "KMS schedule-key-deletion stub requires --key-id and an integer --pending-window-in-days in command: $command"
                    exit 1
                fi
                local kms_schedule_fixture
                kms_schedule_fixture=$(jq -c --arg key_id "$scheduled_key_id" '
                    [.Keys[] | select(.KeyMetadata.KeyId == $key_id)]
                    | if length == 1 then .[0] else empty end
                ' <<< "$KMS_FIXTURE")
                if [ -z "$kms_schedule_fixture" ]; then
                    record_stub_failure "$KMS_STUB_FAILURE_LOG" \
                        "KMS schedule-key-deletion stub saw an unknown or duplicate key ID '$scheduled_key_id' in command: $command"
                    exit 1
                fi
                printf '%s\t%s\n' "$scheduled_key_id" "$pending_window_in_days" >> "$KMS_SCHEDULE_LOG"
                if [ "$(jq -r '.ScheduleDenied' <<< "$kms_schedule_fixture")" = "true" ]; then
                    echo "An error occurred (AccessDeniedException) when calling the ScheduleKeyDeletion operation for key $scheduled_key_id" >&2
                    exit 254
                fi
                ;;
            "aws kms "*)
                record_stub_failure "$KMS_STUB_FAILURE_LOG" \
                    "KMS stub did not recognize command: $command"
                exit 1
                ;;
            "aws ec2 describe-ipams"*)
                local query_marker=' --query '
                local output_marker=' --output '
                local ipam_query=""
                if [[ "$command" == *"$query_marker"*"$output_marker"* ]]; then
                    ipam_query="${command#*"$query_marker"}"
                    ipam_query="${ipam_query%%"$output_marker"*}"
                else
                    record_stub_failure "$IPAM_STUB_FAILURE_LOG" \
                        "IPAM stub did not find --query in command: $command"
                    exit 1
                fi

                local state_filtered_query_pattern='^Ipams\[\?contains\(\[(.*)\],[[:space:]]*State\)\]\.IpamId$'
                if [[ "$ipam_query" =~ $state_filtered_query_pattern ]]; then
                    local state_literals="${BASH_REMATCH[1]}"
                    local state_literals_pattern="^\`[[:lower:]-]+\`([[:space:]]*,[[:space:]]*\`[[:lower:]-]+\`)*$"
                    if ! [[ "$state_literals" =~ $state_literals_pattern ]]; then
                        record_stub_failure "$IPAM_STUB_FAILURE_LOG" \
                            "IPAM stub could not parse state literals in query '$ipam_query' from command: $command"
                        exit 1
                    fi

                    local state_csv="${state_literals//\`/}"
                    state_csv="${state_csv// /}"
                    local filtered_ipam_states=()
                    IFS=',' read -r -a filtered_ipam_states <<< "$state_csv"
                    local filtered_ipam_state
                    for filtered_ipam_state in "${filtered_ipam_states[@]}"; do
                        case "$filtered_ipam_state" in
                            create-in-progress | create-complete | create-failed | \
                                modify-in-progress | modify-complete | modify-failed | \
                                delete-in-progress | delete-complete | delete-failed | \
                                isolate-in-progress | isolate-complete | restore-in-progress)
                                ;;
                            *)
                                record_stub_failure "$IPAM_STUB_FAILURE_LOG" \
                                    "IPAM stub did not recognize state '$filtered_ipam_state' in query '$ipam_query' from command: $command"
                                exit 1
                                ;;
                        esac
                    done
                    jq -r --arg state_csv "$state_csv" '
                        ($state_csv | split(",")) as $states
                        | [
                            .Ipams[]
                            | select(.State as $state | $states | index($state))
                            | .IpamId
                        ]
                        | @tsv
                    ' <<< "$IPAM_RESPONSE"
                else
                    record_stub_failure "$IPAM_STUB_FAILURE_LOG" \
                        "IPAM stub did not recognize query '$ipam_query' in command: $command"
                    exit 1
                fi
                ;;
            "aws ec2 delete-ipam-pool"*)
                record_stub_failure "$IPAM_STUB_FAILURE_LOG" \
                    "IPAM stub saw an unexpected standalone pool delete: $command"
                exit 1
                ;;
            "aws ec2 delete-ipam"*)
                local ipam_delete_arguments_text="${command#aws ec2 delete-ipam}"
                local ipam_delete_arguments=()
                read -r -a ipam_delete_arguments <<< "$ipam_delete_arguments_text"
                local parsed_ipam_id=""
                local parsed_cascade=0
                local argument_index=0
                while [ "$argument_index" -lt "${#ipam_delete_arguments[@]}" ]; do
                    case "${ipam_delete_arguments[$argument_index]}" in
                        --ipam-id)
                            argument_index=$((argument_index + 1))
                            if [ "$argument_index" -ge "${#ipam_delete_arguments[@]}" ] || [ -n "$parsed_ipam_id" ]; then
                                record_stub_failure "$IPAM_STUB_FAILURE_LOG" \
                                    "IPAM stub found a missing or duplicate --ipam-id in command: $command"
                                exit 1
                            fi
                            parsed_ipam_id="${ipam_delete_arguments[$argument_index]}"
                            ;;
                        --cascade)
                            if [ "$parsed_cascade" -eq 1 ]; then
                                record_stub_failure "$IPAM_STUB_FAILURE_LOG" \
                                    "IPAM stub found duplicate --cascade in command: $command"
                                exit 1
                            fi
                            parsed_cascade=1
                            ;;
                        *)
                            record_stub_failure "$IPAM_STUB_FAILURE_LOG" \
                                "IPAM stub did not recognize argument '${ipam_delete_arguments[$argument_index]}' in command: $command"
                            exit 1
                            ;;
                    esac
                    argument_index=$((argument_index + 1))
                done

                if [ -z "$parsed_ipam_id" ] || [ "$parsed_cascade" -ne 1 ]; then
                    record_stub_failure "$IPAM_STUB_FAILURE_LOG" \
                        "IPAM stub requires --ipam-id and --cascade in command: $command"
                    exit 1
                fi
                if [ "$parsed_ipam_id" != "$IPAM_ONE_ID" ] && [ "$parsed_ipam_id" != "$IPAM_TWO_ID" ]; then
                    record_stub_failure "$IPAM_STUB_FAILURE_LOG" \
                        "IPAM stub saw delete for non-enumerated IPAM $parsed_ipam_id in command: $command"
                    exit 1
                fi
                printf '%s\n' "$parsed_ipam_id" >> "$IPAM_DELETE_LOG"
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
: > "$IPAM_STUB_FAILURE_LOG"
: > "$IPAM_DELETE_LOG"
: > "$KMS_STUB_FAILURE_LOG"
: > "$KMS_SCHEDULE_LOG"
: > "$CLOUDFRONT_STUB_FAILURE_LOG"
: > "$CLOUDFRONT_DISTRIBUTION_DELETE_LOG"
: > "$CLOUDFRONT_OAC_DELETE_LOG"
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

while IFS= read -r stub_failure; do
    [ -z "$stub_failure" ] && continue
    fail "$stub_failure"
done < "$IPAM_STUB_FAILURE_LOG"

while IFS= read -r stub_failure; do
    [ -z "$stub_failure" ] && continue
    fail "$stub_failure"
done < "$KMS_STUB_FAILURE_LOG"

while IFS= read -r stub_failure; do
    [ -z "$stub_failure" ] && continue
    fail "$stub_failure"
done < "$CLOUDFRONT_STUB_FAILURE_LOG"

# Successful top-level deletes are 2 load balancers + 1 target group + 1 VPC +
# 2 IPAMs + 3 KMS keys + 1 CloudFront distribution + 2 unreferenced CloudFront
# OACs = 12. The two referenced OACs are deferred and therefore not counted.
# The unexpected security-group IsEgress value, denied KMS tag read, and denied
# OAC ETag read are enumeration warnings.
expected_ordering_summary="  WARNING: ordering-account: 12 found, 12 deleted, 0 failed; 3 enumeration errors"
ordering_summary=$(grep -F "ordering-account:" "$WORK_DIR/stderr.log" || true)
if [ "$ordering_summary" != "$expected_ordering_summary" ]; then
    fail "unexpected successful ordering summary: expected '$expected_ordering_summary', got '${ordering_summary:-<missing>}'"
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

cloudfront_distribution_enumeration_count=$(grep -cF "aws cloudfront list-distributions --query " "$COMMAND_LOG" || true)
if [ "$cloudfront_distribution_enumeration_count" -ne 2 ]; then
    fail "expected one CloudFront reference snapshot and one state-aware distribution enumeration, got $cloudfront_distribution_enumeration_count list-distributions calls"
fi
cloudfront_in_use_oac_enumeration=$(first_line_containing "aws cloudfront list-distributions --query DistributionList.Items[].Origins.Items")
if [ -z "$cloudfront_in_use_oac_enumeration" ]; then
    fail "missing CloudFront in-use OAC snapshot"
fi
cloudfront_distribution_enumeration=$(first_line_containing "aws cloudfront list-distributions --query DistributionList.Items[?Enabled")
if [ -z "$cloudfront_distribution_enumeration" ]; then
    fail "missing state-aware CloudFront distribution enumeration"
fi
first_cloudfront_distribution_delete=$(first_line_containing "aws cloudfront delete-distribution --id")
if [ -z "$first_cloudfront_distribution_delete" ]; then
    fail "missing CloudFront distribution delete after the in-use OAC snapshot"
elif [ -n "$cloudfront_in_use_oac_enumeration" ] && \
    [ "$cloudfront_in_use_oac_enumeration" -ge "$first_cloudfront_distribution_delete" ]; then
    fail "CloudFront in-use OAC snapshot did not precede distribution deletion"
fi
distribution_delete_count=$(awk 'END { print NR }' "$CLOUDFRONT_DISTRIBUTION_DELETE_LOG")
if [ "$distribution_delete_count" -ne 1 ]; then
    fail "expected exactly one CloudFront distribution delete, got $distribution_delete_count"
fi
expected_distribution_delete="$DISABLED_DEPLOYED_DISTRIBUTION_ID"$'\t'"$DISABLED_DEPLOYED_DISTRIBUTION_ETAG"
expected_distribution_delete_count=$(grep -cFx \
    "$expected_distribution_delete" "$CLOUDFRONT_DISTRIBUTION_DELETE_LOG" || true)
if [ "$expected_distribution_delete_count" -ne 1 ]; then
    fail "disabled Deployed distribution was not deleted exactly once with its enumerated ETag"
fi
for excluded_distribution_id in \
    "$DISABLED_IN_PROGRESS_DISTRIBUTION_ID" \
    "$ENABLED_DEPLOYED_DISTRIBUTION_ID" \
    "$ENABLED_IN_PROGRESS_DISTRIBUTION_ID"
do
    if grep -F "aws cloudfront delete-distribution --id $excluded_distribution_id " "$COMMAND_LOG" >/dev/null; then
        fail "non-sweepable CloudFront distribution $excluded_distribution_id was deleted"
    fi
done
if grep -F "aws cloudfront get-distribution " "$COMMAND_LOG" >/dev/null; then
    fail "CloudFront distribution ETags must come from list-distributions, not get-distribution"
fi
if grep -F "aws cloudfront update-distribution" "$COMMAND_LOG" >/dev/null; then
    fail "deep cleanup must not auto-disable CloudFront distributions"
fi

oac_list_enumeration=$(first_line_containing "aws cloudfront list-origin-access-controls")
if [ -z "$oac_list_enumeration" ]; then
    fail "missing CloudFront origin access control enumeration"
fi
for oac_id in "$FIRST_OAC_ID" "$GET_DENIED_OAC_ID" "$ODDLY_NAMED_OAC_ID"; do
    oac_get_count=$(grep -cF "aws cloudfront get-origin-access-control --id $oac_id " "$COMMAND_LOG" || true)
    if [ "$oac_get_count" -ne 1 ]; then
        fail "expected one ETag enumeration for CloudFront OAC $oac_id, got $oac_get_count"
    fi
done
# Both OACs referenced in the pre-delete snapshot are deferred before their ETag
# round trip. This includes the OAC on the distribution deleted above: retrying it
# on the next sweep avoids depending on immediate CloudFront consistency and on
# the distribution delete having succeeded.
for deferred_oac_id in "$ENABLED_DISTRIBUTION_OAC_ID" "$DELETED_DISTRIBUTION_OAC_ID"; do
    if grep -F "aws cloudfront get-origin-access-control --id $deferred_oac_id " "$COMMAND_LOG" >/dev/null; then
        fail "referenced CloudFront OAC $deferred_oac_id reached its ETag enumeration"
    fi
    if grep -F "aws cloudfront delete-origin-access-control --id $deferred_oac_id " "$COMMAND_LOG" >/dev/null; then
        fail "referenced CloudFront OAC $deferred_oac_id reached its delete attempt"
    fi
    if ! grep -Fx "  Skipping CloudFront origin access control $deferred_oac_id because a distribution references it." \
        "$WORK_DIR/stdout.log" >/dev/null; then
        fail "referenced CloudFront OAC $deferred_oac_id was not reported as deferred"
    fi
done
oac_delete_count=$(awk 'END { print NR }' "$CLOUDFRONT_OAC_DELETE_LOG")
if [ "$oac_delete_count" -ne 2 ]; then
    fail "expected exactly two CloudFront OAC deletes, got $oac_delete_count"
fi
for expected_oac_delete in \
    "$FIRST_OAC_ID"$'\t'"$FIRST_OAC_ETAG" \
    "$ODDLY_NAMED_OAC_ID"$'\t'"$ODDLY_NAMED_OAC_ETAG"
do
    expected_oac_delete_count=$(grep -cFx "$expected_oac_delete" "$CLOUDFRONT_OAC_DELETE_LOG" || true)
    if [ "$expected_oac_delete_count" -ne 1 ]; then
        fail "CloudFront OAC delete did not use the exact enumerated ID and ETag: $expected_oac_delete"
    fi
done
if grep -F "aws cloudfront delete-origin-access-control --id $GET_DENIED_OAC_ID " "$COMMAND_LOG" >/dev/null; then
    fail "CloudFront OAC $GET_DENIED_OAC_ID was deleted despite its failed ETag enumeration"
fi

last_distribution_delete=$(last_line_containing "aws cloudfront delete-distribution --id")
first_oac_delete=$(first_line_containing "aws cloudfront delete-origin-access-control --id")
if [ -z "$last_distribution_delete" ] || [ -z "$first_oac_delete" ]; then
    fail "missing CloudFront deletes needed to verify distribution-before-OAC ordering"
elif [ "$last_distribution_delete" -ge "$first_oac_delete" ]; then
    fail "not every CloudFront distribution delete ran before the first OAC delete"
fi

denied_oac_get=$(first_line_containing "aws cloudfront get-origin-access-control --id $GET_DENIED_OAC_ID")
later_oac_delete=$(first_line_containing "aws cloudfront delete-origin-access-control --id $ODDLY_NAMED_OAC_ID")
if [ -z "$denied_oac_get" ]; then
    fail "missing AccessDenied CloudFront OAC ETag-enumeration fixture call"
elif [ -z "$later_oac_delete" ] || [ "$later_oac_delete" -le "$denied_oac_get" ]; then
    fail "failed CloudFront OAC ETag enumeration stopped a later OAC from being deleted"
fi
if ! grep -F "$GET_DENIED_OAC_ID" "$WORK_DIR/stderr.log" | grep -F "AccessDenied" >/dev/null; then
    fail "CloudFront OAC ETag-enumeration failure was not reported"
fi

ipam_enumeration=$(first_line_containing "aws ec2 describe-ipams")
if [ -z "$ipam_enumeration" ]; then
    fail "missing state-aware IPAM enumeration"
fi
for ipam_id in "$IPAM_ONE_ID" "$IPAM_TWO_ID"; do
    ipam_delete_count=$(grep -cFx "$ipam_id" "$IPAM_DELETE_LOG" || true)
    ipam_delete=$(first_line_containing "aws ec2 delete-ipam --ipam-id $ipam_id")
    if [ "$ipam_delete_count" -ne 1 ]; then
        fail "expected one cascade delete for enumerated IPAM $ipam_id, got $ipam_delete_count"
    elif [ -z "$ipam_delete" ]; then
        fail "missing delete command-log entry for enumerated IPAM $ipam_id"
    elif [ -z "$ipam_enumeration" ] || [ "$ipam_delete" -le "$ipam_enumeration" ]; then
        fail "IPAM $ipam_id was not deleted after IPAM enumeration"
    fi
done
for excluded_ipam_id in \
    "$TRANSITIONING_IPAM_ID" \
    "$MODIFYING_IPAM_ID" \
    "$DELETING_IPAM_ID" \
    "$DELETE_COMPLETE_IPAM_ID" \
    "$ISOLATING_IPAM_ID" \
    "$RESTORING_IPAM_ID"
do
    if grep -F "aws ec2 delete-ipam --ipam-id $excluded_ipam_id" "$COMMAND_LOG" >/dev/null; then
        fail "IPAM $excluded_ipam_id is in a non-swept state but was deleted"
    fi
done
if grep -F "aws ec2 delete-ipam-pool" "$COMMAND_LOG" >/dev/null; then
    fail "deep cleanup must rely on cascade IPAM deletion to remove private-scope pools"
fi

kms_list_enumeration=$(first_line_containing "aws kms list-keys")
if [ -z "$kms_list_enumeration" ]; then
    fail "missing KMS key enumeration"
fi
for kms_key_id in \
    "$KMS_AWS_MANAGED_KEY_ID" \
    "$KMS_PENDING_DELETION_KEY_ID" \
    "$KMS_FOREIGN_TAG_KEY_ID" \
    "$KMS_ACCESS_DENIED_KEY_ID" \
    "$KMS_ENABLED_KEY_ID" \
    "$KMS_DISABLED_KEY_ID" \
    "$KMS_PENDING_REPLICA_DELETION_KEY_ID"
do
    kms_describe_count=$(grep -cF "aws kms describe-key --key-id $kms_key_id" "$COMMAND_LOG" || true)
    if [ "$kms_describe_count" -ne 1 ]; then
        fail "expected one metadata enumeration for KMS key $kms_key_id, got $kms_describe_count"
    fi
done
for prefiltered_kms_key_id in "$KMS_AWS_MANAGED_KEY_ID" "$KMS_PENDING_DELETION_KEY_ID"; do
    if grep -F "aws kms list-resource-tags --key-id $prefiltered_kms_key_id" "$COMMAND_LOG" >/dev/null; then
        fail "KMS key $prefiltered_kms_key_id must be rejected by metadata before tag enumeration"
    fi
done

kms_schedule_count=$(awk 'END { print NR }' "$KMS_SCHEDULE_LOG")
if [ "$kms_schedule_count" -ne 3 ]; then
    fail "expected exactly three KMS schedule-key-deletion calls, got $kms_schedule_count"
fi
for expected_kms_key_id in \
    "$KMS_ENABLED_KEY_ID" \
    "$KMS_DISABLED_KEY_ID" \
    "$KMS_PENDING_REPLICA_DELETION_KEY_ID"
do
    expected_kms_schedule_count=$(awk -F '\t' -v key_id="$expected_kms_key_id" '
        $1 == key_id && $2 == "7" {
            count++
        }
        END {
            print count + 0
        }
    ' "$KMS_SCHEDULE_LOG")
    all_kms_schedule_count=$(awk -F '\t' -v key_id="$expected_kms_key_id" '
        $1 == key_id {
            count++
        }
        END {
            print count + 0
        }
    ' "$KMS_SCHEDULE_LOG")
    if [ "$all_kms_schedule_count" -ne 1 ]; then
        fail "expected exactly one schedule-key-deletion call for KMS key $expected_kms_key_id, got $all_kms_schedule_count"
    elif [ "$expected_kms_schedule_count" -ne 1 ]; then
        fail "KMS key $expected_kms_key_id was not scheduled with --pending-window-in-days 7"
    fi
done
for excluded_kms_key_id in \
    "$KMS_AWS_MANAGED_KEY_ID" \
    "$KMS_PENDING_DELETION_KEY_ID" \
    "$KMS_FOREIGN_TAG_KEY_ID" \
    "$KMS_ACCESS_DENIED_KEY_ID"
do
    excluded_kms_schedule_count=$(awk -F '\t' -v key_id="$excluded_kms_key_id" '
        $1 == key_id {
            count++
        }
        END {
            print count + 0
        }
    ' "$KMS_SCHEDULE_LOG")
    if [ "$excluded_kms_schedule_count" -ne 0 ]; then
        fail "excluded KMS key $excluded_kms_key_id appeared in a schedule-key-deletion call"
    fi
done

access_denied_tag_enumeration=$(first_line_containing "aws kms list-resource-tags --key-id $KMS_ACCESS_DENIED_KEY_ID")
later_kms_schedule=$(first_line_containing "aws kms schedule-key-deletion --key-id $KMS_ENABLED_KEY_ID")
if [ -z "$access_denied_tag_enumeration" ]; then
    fail "missing AccessDenied KMS tag-enumeration fixture call"
elif [ -z "$later_kms_schedule" ] || [ "$later_kms_schedule" -le "$access_denied_tag_enumeration" ]; then
    fail "AccessDenied while reading KMS tags stopped a later sweepable key from being scheduled"
fi
if ! grep -F "$KMS_ACCESS_DENIED_KEY_ID" "$WORK_DIR/stderr.log" | \
    grep -F "AccessDeniedException" >/dev/null; then
    fail "AccessDenied KMS tag-enumeration failure was not reported"
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

# If the account-wide OAC reference snapshot is unreadable, skip the entire OAC
# pass. Treating the blank enumeration output as an empty in-use set would make
# every OAC look deletable and recreate the permanent enabled-distribution wedge.
CLOUDFRONT_IN_USE_ENUMERATION_DENIED=1
COMMAND_LOG="$WORK_DIR/cloudfront-reference-failure-aws-calls.log"
CLOUDFRONT_STUB_FAILURE_LOG="$WORK_DIR/cloudfront-reference-failure-stub-failures.log"
CLOUDFRONT_REFERENCE_FAILURE_STDOUT_LOG="$WORK_DIR/cloudfront-reference-failure-stdout.log"
CLOUDFRONT_REFERENCE_FAILURE_STDERR_LOG="$WORK_DIR/cloudfront-reference-failure-stderr.log"
: > "$COMMAND_LOG"
: > "$CLOUDFRONT_STUB_FAILURE_LOG"
if ! deep_cleanup_account "cloudfront-reference-failure-account" \
    >"$CLOUDFRONT_REFERENCE_FAILURE_STDOUT_LOG" \
    2>"$CLOUDFRONT_REFERENCE_FAILURE_STDERR_LOG"; then
    fail "deep_cleanup_account returned non-zero after the OAC reference enumeration failed"
fi
CLOUDFRONT_IN_USE_ENUMERATION_DENIED=0

while IFS= read -r stub_failure; do
    [ -z "$stub_failure" ] && continue
    fail "$stub_failure"
done < "$CLOUDFRONT_STUB_FAILURE_LOG"

failed_reference_snapshot_count=$(grep -cF \
    "aws cloudfront list-distributions --query DistributionList.Items[].Origins.Items" \
    "$COMMAND_LOG" || true)
if [ "$failed_reference_snapshot_count" -ne 1 ]; then
    fail "expected exactly one failed CloudFront OAC reference snapshot, got $failed_reference_snapshot_count"
fi
if ! grep -F "CloudFront origin access controls referenced by distributions" \
    "$CLOUDFRONT_REFERENCE_FAILURE_STDERR_LOG" | grep -F "AccessDenied" >/dev/null; then
    fail "failed CloudFront OAC reference snapshot was not reported with its AccessDenied diagnostic"
fi
if ! grep -Fx "  Skipping CloudFront origin access control cleanup because distribution references could not be enumerated." \
    "$CLOUDFRONT_REFERENCE_FAILURE_STDOUT_LOG" >/dev/null; then
    fail "failed CloudFront OAC reference snapshot did not report the fail-closed pass skip"
fi
if ! grep -F "aws cloudfront delete-distribution --id $DISABLED_DEPLOYED_DISTRIBUTION_ID " \
    "$COMMAND_LOG" >/dev/null; then
    fail "failed CloudFront OAC reference snapshot incorrectly stopped distribution cleanup"
fi
for forbidden_oac_command in \
    "aws cloudfront list-origin-access-controls" \
    "aws cloudfront get-origin-access-control" \
    "aws cloudfront delete-origin-access-control"
do
    if grep -F "$forbidden_oac_command" "$COMMAND_LOG" >/dev/null; then
        fail "failed CloudFront OAC reference snapshot fell through to: $forbidden_oac_command"
    fi
done

# A key policy may permit tag reads but deny ScheduleKeyDeletion. Exercise that
# fatal top-level resource failure separately so the successful ordering scenario
# above keeps its exact schedule assertions.
KMS_FIXTURE=$(jq -c --arg key_id "$KMS_ENABLED_KEY_ID" '
    .Keys |= map(
        if .KeyMetadata.KeyId == $key_id then
            .ScheduleDenied = true
        else
            .
        end
    )
' <<< "$KMS_FIXTURE")
schedule_denied_fixture_count=$(jq -r '[.Keys[] | select(.ScheduleDenied == true)] | length' <<< "$KMS_FIXTURE")
if [ "$schedule_denied_fixture_count" -ne 1 ]; then
    fail "expected exactly one ScheduleDenied KMS fixture, got $schedule_denied_fixture_count"
fi

COMMAND_LOG="$WORK_DIR/kms-schedule-failure-aws-calls.log"
KMS_SCHEDULE_LOG="$WORK_DIR/kms-schedule-failure-schedules.log"
KMS_STUB_FAILURE_LOG="$WORK_DIR/kms-schedule-failure-stub-failures.log"
KMS_SCHEDULE_FAILURE_STDERR_LOG="$WORK_DIR/kms-schedule-failure-stderr.log"
: > "$COMMAND_LOG"
: > "$KMS_SCHEDULE_LOG"
: > "$KMS_STUB_FAILURE_LOG"
if deep_cleanup_account "kms-schedule-failure-account" \
    >"$WORK_DIR/kms-schedule-failure-stdout.log" \
    2>"$KMS_SCHEDULE_FAILURE_STDERR_LOG"; then
    kms_schedule_failure_status=0
else
    kms_schedule_failure_status=$?
fi

while IFS= read -r stub_failure; do
    [ -z "$stub_failure" ] && continue
    fail "$stub_failure"
done < "$KMS_STUB_FAILURE_LOG"

if [ "$kms_schedule_failure_status" -eq 0 ]; then
    fail "deep_cleanup_account returned zero after ScheduleKeyDeletion failed"
fi
if [ "$DEEP_CLEANUP_FAILED_COUNT" -ne 1 ]; then
    fail "ScheduleKeyDeletion failure was not counted as exactly one failed resource"
fi
if [ "$DEEP_CLEANUP_SUPPORTING_FAILURE_COUNT" -ne 0 ]; then
    fail "ScheduleKeyDeletion failure was incorrectly counted as a supporting failure"
fi
if ! grep -F "Failed to schedule KMS key $KMS_ENABLED_KEY_ID for deletion" \
    "$KMS_SCHEDULE_FAILURE_STDERR_LOG" >/dev/null; then
    fail "ScheduleKeyDeletion failure did not name the KMS key and operation"
fi
if ! grep -F "ScheduleKeyDeletion operation for key $KMS_ENABLED_KEY_ID" \
    "$KMS_SCHEDULE_FAILURE_STDERR_LOG" | grep -F "AccessDeniedException" >/dev/null; then
    fail "ScheduleKeyDeletion failure did not retain the AWS AccessDeniedException"
fi

denied_kms_schedule=$(first_line_containing "aws kms schedule-key-deletion --key-id $KMS_ENABLED_KEY_ID")
if [ -z "$denied_kms_schedule" ]; then
    fail "missing attempted schedule for the ScheduleDenied KMS key"
fi
for later_kms_key_id in "$KMS_DISABLED_KEY_ID" "$KMS_PENDING_REPLICA_DELETION_KEY_ID"; do
    later_kms_schedule=$(first_line_containing "aws kms schedule-key-deletion --key-id $later_kms_key_id")
    if [ -z "$later_kms_schedule" ]; then
        fail "ScheduleKeyDeletion failure stopped the later attempt for KMS key $later_kms_key_id"
    elif [ -n "$denied_kms_schedule" ] && [ "$later_kms_schedule" -le "$denied_kms_schedule" ]; then
        fail "later KMS key $later_kms_key_id was not attempted after the ScheduleKeyDeletion failure"
    fi
done

if [ "$FAILURES" -ne 0 ]; then
    exit 1
fi

echo "deep_cleanup_dependency_ordering: OK"
