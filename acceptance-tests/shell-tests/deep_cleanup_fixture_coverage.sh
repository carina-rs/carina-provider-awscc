#!/bin/bash
# Test that every AWSCC resource type in an acceptance fixture is covered by
# deep_cleanup_account, without making AWS calls.
#
# Usage:
#   bash acceptance-tests/shell-tests/deep_cleanup_fixture_coverage.sh
#
# Coverage is anchored to the cleanup implementation itself. Each mapping row
# names the AWS CLI mutation command that a resource type's sweep must issue,
# and the test greps the extracted deep_cleanup_account body and its reachable
# helpers for that command. It deliberately does not store a per-type "covered"
# flag: such a flag would stay true after someone deleted the sweep pass, while
# anchoring coverage to the sweep text makes that deletion fail this test.
#
# The allowlist is only for types AWS removes as a side effect of deleting their
# parent, so no sweep pass is expected. Each reason is validated data rather
# than a comment; an entry with no reason fails. Known gaps are types with
# fixtures but no current sweep pass. They are reported without failing, but
# the list is self-retiring: once a listed type gains its sweep command, the
# test fails until the entry is removed. Without that check, known gaps would
# silently become a permanent excuse list—the exact rot this test prevents.
#
# Adding a fixture for a new resource type also requires a mapping row. The type
# must then be swept, allowlisted, or recorded as a known gap; otherwise this
# test reports it as unmapped.
#
# This deliberately does not source acceptance-tests/shared/_helpers.sh: that
# helper resolves a carina binary and WASM provider artifacts at source time.
# This test must stay runnable in bare CI with no Rust toolchain, carina binary,
# AWS credentials, or AWS calls.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
RUN_TESTS_SH="$SCRIPT_DIR/../run-tests.sh"
ACCEPTANCE_TESTS_DIR="$SCRIPT_DIR/.."

WORK_DIR="$(mktemp -d)"
FIXTURE_FILES_FILE="$WORK_DIR/fixture-files"
FIXTURE_TYPES_FILE="$WORK_DIR/fixture-types"
DEFINED_FUNCTIONS_FILE="$WORK_DIR/defined-functions"
REQUESTED_FUNCTIONS_FILE="$WORK_DIR/requested-functions"
NEXT_FUNCTIONS_FILE="$WORK_DIR/next-functions"
EXTRACTED_FUNCTIONS_FILE="$WORK_DIR/extracted-functions"
FUNCTION_BODY_FILE="$WORK_DIR/function-body.sh"
SWEEP_TEXT_FILE="$WORK_DIR/sweep-text.sh"
SWEEP_COMMANDS_FILE="$WORK_DIR/sweep-commands"
SWEEP_MAPPINGS_FILE="$WORK_DIR/sweep-mappings"
ALLOWLIST_FILE="$WORK_DIR/allowlist"
ALLOWLIST_TYPES_FILE="$WORK_DIR/allowlist-types"
KNOWN_GAPS_FILE="$WORK_DIR/known-gaps"
KNOWN_GAP_TYPES_FILE="$WORK_DIR/known-gap-types"

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

# Scan every fixture recursively, but never treat this test directory as
# fixture input if a .crn file is added here later.
if ! find "$ACCEPTANCE_TESTS_DIR" \
    -path "$ACCEPTANCE_TESTS_DIR/shell-tests" -prune -o \
    -type f -name '*.crn' -print0 > "$FIXTURE_FILES_FILE"; then
    fail "failed to enumerate acceptance fixture files"
fi

while IFS= read -r -d '' fixture_file; do
    grep -Eho 'awscc\.[[:alnum:]_]+\.[[:alnum:]_]+' "$fixture_file" || true
done < "$FIXTURE_FILES_FILE" | LC_ALL=C sort -u > "$FIXTURE_TYPES_FILE"

fixture_type_count=$(wc -l < "$FIXTURE_TYPES_FILE" | tr -d '[:space:]')
if [ "$fixture_type_count" -lt 10 ]; then
    fail "fixture scan found only $fixture_type_count resource types; expected at least 10"
fi

# Extract deep_cleanup_account and only the top-level helper functions that are
# reachable from its text. In particular, an unused helper cannot keep a
# deleted sweep call looking covered.
awk '
    /^[[:alpha:]_][[:alnum:]_]*\(\) \{/ {
        name = $1
        sub(/\(\)$/, "", name)
        print name
    }
' "$RUN_TESTS_SH" | LC_ALL=C sort -u > "$DEFINED_FUNCTIONS_FILE"

printf '%s\n' deep_cleanup_account > "$REQUESTED_FUNCTIONS_FILE"
: > "$EXTRACTED_FUNCTIONS_FILE"
: > "$SWEEP_TEXT_FILE"

while :; do
    newly_extracted=0
    while IFS= read -r function_name; do
        [ -z "$function_name" ] && continue
        if grep -Fx "$function_name" "$EXTRACTED_FUNCTIONS_FILE" >/dev/null; then
            continue
        fi

        if extract_function "$function_name" > "$FUNCTION_BODY_FILE"; then
            printf '\n' >> "$SWEEP_TEXT_FILE"
            command cat "$FUNCTION_BODY_FILE" >> "$SWEEP_TEXT_FILE"
            newly_extracted=1
        else
            fail "failed to extract $function_name from run-tests.sh"
        fi
        printf '%s\n' "$function_name" >> "$EXTRACTED_FUNCTIONS_FILE"
    done < "$REQUESTED_FUNCTIONS_FILE"

    [ "$newly_extracted" -eq 0 ] && break

    awk '
        function shell_code(line, mask_quoted,    result, i, ch, previous, escaped) {
            result = ""
            escaped = 0
            for (i = 1; i <= length(line); i++) {
                ch = substr(line, i, 1)
                previous = i > 1 ? substr(line, i - 1, 1) : ""

                if (in_single_quote) {
                    if (ch == "\047") {
                        in_single_quote = 0
                    }
                    result = result (mask_quoted ? " " : ch)
                    continue
                }
                if (in_double_quote) {
                    if (escaped) {
                        escaped = 0
                    } else if (ch == "\\") {
                        escaped = 1
                    } else if (ch == "\"") {
                        in_double_quote = 0
                    }
                    result = result (mask_quoted ? " " : ch)
                    continue
                }
                if (escaped) {
                    escaped = 0
                    result = result ch
                    continue
                }
                if (ch == "\\") {
                    escaped = 1
                    result = result ch
                    continue
                }
                if (ch == "\047") {
                    in_single_quote = 1
                    result = result (mask_quoted ? " " : ch)
                    continue
                }
                if (ch == "\"") {
                    in_double_quote = 1
                    result = result (mask_quoted ? " " : ch)
                    continue
                }
                if (ch == "#" && (i == 1 || previous ~ /[[:space:]]/)) {
                    break
                }
                result = result ch
            }
            return result
        }
        {
            line = shell_code($0, 0)
            while (match(line, /[[:alpha:]_][[:alnum:]_]*/)) {
                print substr(line, RSTART, RLENGTH)
                line = substr(line, RSTART + RLENGTH)
            }
        }
    ' "$SWEEP_TEXT_FILE" | LC_ALL=C sort -u > "$NEXT_FUNCTIONS_FILE"

    : > "$REQUESTED_FUNCTIONS_FILE"
    while IFS= read -r function_name; do
        if grep -Fx "$function_name" "$DEFINED_FUNCTIONS_FILE" >/dev/null; then
            printf '%s\n' "$function_name" >> "$REQUESTED_FUNCTIONS_FILE"
        fi
    done < "$NEXT_FUNCTIONS_FILE"
done

sweep_function_count=$(wc -l < "$EXTRACTED_FUNCTIONS_FILE" | tr -d '[:space:]')
if [ "$sweep_function_count" -lt 5 ]; then
    fail "sweep extraction found only $sweep_function_count reachable functions; expected at least 5"
fi

# Normalize each literal AWS CLI command to its service and verb so checks use
# exact command tokens. Exact matching keeps delete-transit-gateway from being
# confused with delete-transit-gateway-vpc-attachment, for example. Comments
# are removed from each physical line before backslash continuations are folded
# into logical lines, so a backslash inside a comment cannot consume the next
# line.
awk '
    function shell_code(line, mask_quoted,    result, i, ch, previous, escaped) {
        result = ""
        escaped = 0
        for (i = 1; i <= length(line); i++) {
            ch = substr(line, i, 1)
            previous = i > 1 ? substr(line, i - 1, 1) : ""

            if (in_single_quote) {
                if (ch == "\047") {
                    in_single_quote = 0
                }
                result = result (mask_quoted ? " " : ch)
                continue
            }
            if (in_double_quote) {
                if (escaped) {
                    escaped = 0
                } else if (ch == "\\") {
                    escaped = 1
                } else if (ch == "\"") {
                    in_double_quote = 0
                }
                result = result (mask_quoted ? " " : ch)
                continue
            }
            if (escaped) {
                escaped = 0
                result = result ch
                continue
            }
            if (ch == "\\") {
                escaped = 1
                result = result ch
                continue
            }
            if (ch == "\047") {
                in_single_quote = 1
                result = result (mask_quoted ? " " : ch)
                continue
            }
            if (ch == "\"") {
                in_double_quote = 1
                result = result (mask_quoted ? " " : ch)
                continue
            }
            if (ch == "#" && (i == 1 || previous ~ /[[:space:]]/)) {
                break
            }
            result = result ch
        }
        return result
    }
    function has_line_continuation(line,    backslashes, i) {
        backslashes = 0
        for (i = length(line); i > 0 && substr(line, i, 1) == "\\"; i--) {
            backslashes++
        }
        return backslashes % 2 == 1
    }
    function emit_commands(line,    command) {
        # A standalone colon consumes its arguments without executing them.
        # It may follow a cleanup helper invocation after lines are folded.
        if (line ~ /(^|[[:space:]]):([[:space:]]|$)/) {
            return
        }
        while (match(line, /aws[[:space:]]+[[:alnum:]-]+[[:space:]]+[[:alnum:]-]+/)) {
            command = substr(line, RSTART, RLENGTH)
            gsub(/[[:space:]]+/, " ", command)
            print command
            line = substr(line, RSTART + RLENGTH)
        }
    }
    {
        physical_line = shell_code($0, 1)
        if (has_line_continuation(physical_line)) {
            logical_line = logical_line substr(physical_line, 1, length(physical_line) - 1) " "
            next
        }
        logical_line = logical_line physical_line
        emit_commands(logical_line)
        logical_line = ""
    }
    END {
        if (logical_line != "") {
            emit_commands(logical_line)
        }
    }
' "$SWEEP_TEXT_FILE" | LC_ALL=C sort -u > "$SWEEP_COMMANDS_FILE"

MUTATION_VERB_PATTERN='^(delete-|release-|revoke-|disassociate-|detach-|schedule-)|^rb$'
sweep_mutation_count=$(awk -v mutation_verb_pattern="$MUTATION_VERB_PATTERN" '
    $3 ~ mutation_verb_pattern {
        count++
    }
    END {
        print count + 0
    }
' "$SWEEP_COMMANDS_FILE")
if [ "$sweep_mutation_count" -lt 10 ]; then
    fail "sweep extraction found only $sweep_mutation_count mutation commands; expected at least 10"
fi

# Each mapping names the literal AWS CLI mutation command that must be present
# in deep_cleanup_account or one of its reachable helpers. Multiple commands
# are comma-separated only when the same DSL type has multiple fixture forms.
command cat > "$SWEEP_MAPPINGS_FILE" <<'EOF'
awscc.cloudfront.Distribution|aws cloudfront delete-distribution
awscc.cloudfront.OriginAccessControl|aws cloudfront delete-origin-access-control
awscc.dynamodb.Table|aws dynamodb delete-table
awscc.ec2.EgressOnlyInternetGateway|aws ec2 delete-egress-only-internet-gateway
awscc.ec2.Eip|aws ec2 release-address
awscc.ec2.FlowLog|aws ec2 delete-flow-logs
awscc.ec2.InternetGateway|aws ec2 delete-internet-gateway
awscc.ec2.Ipam|aws ec2 delete-ipam
awscc.ec2.IpamPool|aws ec2 delete-ipam-pool
awscc.ec2.NatGateway|aws ec2 delete-nat-gateway
awscc.ec2.Route|aws ec2 delete-route
awscc.ec2.RouteTable|aws ec2 delete-route-table
awscc.ec2.SecurityGroup|aws ec2 delete-security-group
awscc.ec2.SecurityGroupEgress|aws ec2 revoke-security-group-egress
awscc.ec2.SecurityGroupIngress|aws ec2 revoke-security-group-ingress
awscc.ec2.Subnet|aws ec2 delete-subnet
awscc.ec2.SubnetRouteTableAssociation|aws ec2 disassociate-route-table
awscc.ec2.TransitGateway|aws ec2 delete-transit-gateway
awscc.ec2.TransitGatewayAttachment|aws ec2 delete-transit-gateway-vpc-attachment
awscc.ec2.Vpc|aws ec2 delete-vpc
awscc.ec2.VpcEndpoint|aws ec2 delete-vpc-endpoints
awscc.ec2.VpcGatewayAttachment|aws ec2 detach-internet-gateway,aws ec2 detach-vpn-gateway
awscc.ec2.VpcPeeringConnection|aws ec2 delete-vpc-peering-connection
awscc.ec2.VpnGateway|aws ec2 delete-vpn-gateway
awscc.ecs.Cluster|aws ecs delete-cluster
awscc.elasticloadbalancingv2.Listener|aws elbv2 delete-listener
awscc.elasticloadbalancingv2.LoadBalancer|aws elbv2 delete-load-balancer
awscc.elasticloadbalancingv2.TargetGroup|aws elbv2 delete-target-group
awscc.iam.OidcProvider|aws iam delete-open-id-connect-provider
awscc.iam.Role|aws iam delete-role
awscc.iam.RolePolicy|aws iam delete-role-policy
awscc.kms.Key|aws kms schedule-key-deletion
awscc.logs.LogGroup|aws logs delete-log-group
awscc.route53.HostedZone|aws route53 delete-hosted-zone
awscc.s3.Bucket|aws s3 rb
awscc.s3.BucketPolicy|aws s3api delete-bucket-policy
awscc.wafv2.WebAcl|aws wafv2 delete-web-acl
EOF

command cat > "$ALLOWLIST_FILE" <<'EOF'
awscc.ec2.IpamPool|Deleted with its parent awscc.ec2.Ipam by delete-ipam --cascade.
awscc.ec2.Route|Deleted with its parent awscc.ec2.RouteTable.
awscc.elasticloadbalancingv2.Listener|Deleted with its parent awscc.elasticloadbalancingv2.LoadBalancer.
awscc.s3.BucketPolicy|Deleted with its parent awscc.s3.Bucket.
EOF

command cat > "$KNOWN_GAPS_FILE" <<'EOF'
awscc.cloudfront.Distribution|Tracked by #408.
awscc.cloudfront.OriginAccessControl|Tracked by #408.
EOF

duplicate_mapping_types=$(cut -d '|' -f 1 "$SWEEP_MAPPINGS_FILE" | LC_ALL=C sort | uniq -d)
while IFS= read -r resource_type; do
    [ -z "$resource_type" ] && continue
    fail "duplicate sweep verb mapping for $resource_type"
done <<< "$duplicate_mapping_types"

while IFS='|' read -r resource_type expected_verbs unexpected_fields; do
    if [ -z "$resource_type" ] || [ -z "$expected_verbs" ] || [ -n "$unexpected_fields" ]; then
        fail "malformed sweep verb mapping for ${resource_type:-<empty type>}"
        continue
    fi
    if ! grep -Fx "$resource_type" "$FIXTURE_TYPES_FILE" >/dev/null; then
        fail "sweep verb mapping for $resource_type has no acceptance fixture; remove the stale row"
    fi

    old_ifs=$IFS
    IFS=','
    for expected_verb in $expected_verbs; do
        mapping_verb=$(awk '{ print $3 }' <<< "$expected_verb")
        if ! [[ $mapping_verb =~ $MUTATION_VERB_PATTERN ]]; then
            fail "sweep verb mapping for $resource_type names non-mutation command '$expected_verb'"
        fi
    done
    IFS=$old_ifs
done < "$SWEEP_MAPPINGS_FILE"

validate_reasoned_entries() {
    local list_name="$1"
    local entries_file="$2"
    local tracking_required="$3"
    local duplicate_exception_types
    local resource_type
    local reason
    local unexpected_fields
    local reason_without_whitespace
    local tracking_pattern='#[[:digit:]]+'

    duplicate_exception_types=$(cut -d '|' -f 1 "$entries_file" | LC_ALL=C sort | uniq -d)
    while IFS= read -r resource_type; do
        [ -z "$resource_type" ] && continue
        fail "duplicate $list_name entry for $resource_type"
    done <<< "$duplicate_exception_types"

    while IFS='|' read -r resource_type reason unexpected_fields; do
        if [ -z "$resource_type" ] || [ -n "$unexpected_fields" ]; then
            fail "malformed $list_name entry for ${resource_type:-<empty type>}"
            continue
        fi

        reason_without_whitespace=${reason//[[:space:]]/}
        if [ -z "$reason_without_whitespace" ]; then
            fail "$list_name entry for $resource_type is missing a reason"
            continue
        fi
        if [ "$tracking_required" -eq 1 ] && ! [[ "$reason" =~ $tracking_pattern ]]; then
            fail "$list_name entry for $resource_type must reference #<number>"
        fi

        if ! grep -Fx "$resource_type" "$FIXTURE_TYPES_FILE" >/dev/null; then
            fail "$list_name entry for $resource_type has no acceptance fixture; remove the stale entry"
        fi
    done < "$entries_file"
}

validate_reasoned_entries "allowlist" "$ALLOWLIST_FILE" 0
validate_reasoned_entries "known-gap" "$KNOWN_GAPS_FILE" 1
cut -d '|' -f 1 "$ALLOWLIST_FILE" > "$ALLOWLIST_TYPES_FILE"
cut -d '|' -f 1 "$KNOWN_GAPS_FILE" > "$KNOWN_GAP_TYPES_FILE"

while IFS= read -r resource_type; do
    [ -z "$resource_type" ] && continue

    is_allowlisted=0
    is_known_gap=0
    if grep -Fx "$resource_type" "$ALLOWLIST_TYPES_FILE" >/dev/null; then
        is_allowlisted=1
    fi
    if grep -Fx "$resource_type" "$KNOWN_GAP_TYPES_FILE" >/dev/null; then
        is_known_gap=1
    fi
    if [ "$is_allowlisted" -eq 1 ] && [ "$is_known_gap" -eq 1 ]; then
        fail "$resource_type is both allowlisted and a known gap"
        continue
    fi

    mapping=$(awk -F '|' -v resource_type="$resource_type" '
        $1 == resource_type {
            print
            exit
        }
    ' "$SWEEP_MAPPINGS_FILE")
    if [ -z "$mapping" ]; then
        fail "unmapped fixture resource type: $resource_type; update deep_cleanup_fixture_coverage.sh by adding a mapping row and sweep pass, allowlisting it when its parent deletes it, or recording it as a known gap"
        continue
    fi

    expected_verbs=${mapping#*|}
    present_verbs=""
    missing_verbs=""
    old_ifs=$IFS
    IFS=','
    for expected_verb in $expected_verbs; do
        if grep -Fx "$expected_verb" "$SWEEP_COMMANDS_FILE" >/dev/null; then
            present_verbs="${present_verbs}${present_verbs:+, }'$expected_verb'"
        else
            missing_verbs="${missing_verbs}${missing_verbs:+, }'$expected_verb'"
        fi
    done
    IFS=$old_ifs

    if [ "$is_known_gap" -eq 1 ]; then
        if [ -n "$present_verbs" ]; then
            fail "$resource_type now has a sweep pass ($present_verbs); remove it from known-gaps"
        else
            known_gap_reason=$(awk -F '|' -v resource_type="$resource_type" '
                $1 == resource_type {
                    print $2
                    exit
                }
            ' "$KNOWN_GAPS_FILE")
            echo "KNOWN GAP: $resource_type has no sweep pass (expected $missing_verbs); $known_gap_reason"
        fi
        continue
    fi

    if [ "$is_allowlisted" -eq 1 ]; then
        if [ -n "$present_verbs" ]; then
            fail "$resource_type now has a sweep pass ($present_verbs); remove it from the allowlist"
        fi
        continue
    fi

    if [ -n "$missing_verbs" ]; then
        fail "$resource_type is missing its sweep pass: expected $missing_verbs in deep_cleanup_account() or a reachable helper"
    fi
done < "$FIXTURE_TYPES_FILE"

if [ "$FAILURES" -ne 0 ]; then
    exit 1
fi

echo "deep_cleanup_fixture_coverage: OK"
