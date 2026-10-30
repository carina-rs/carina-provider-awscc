#!/bin/bash
# Test state helpers against schema-v1 JSON emitted by `carina state show --json`.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
HELPERS_SH="$SCRIPT_DIR/../shared/_helpers.sh"
TEST_DIR="$(mktemp -d)"
TEST_DIR="$(cd "$TEST_DIR" && pwd -P)"

cleanup_test_dir() {
    rm -rf "$TEST_DIR"
}
trap cleanup_test_dir EXIT

STATE_DIR="$TEST_DIR/state"
NO_STATE_DIR="$TEST_DIR/no-state"
EMPTY_STATE_DIR="$TEST_DIR/empty-state-dir"
OTHER_ERROR_DIR="$TEST_DIR/other-error"
STATE_JSON="$TEST_DIR/state.json"
ABSENT_STATE_JSON="$TEST_DIR/absent-state.json"
SCHEMA_V2_JSON="$TEST_DIR/schema-v2.json"
STUB_BIN="$TEST_DIR/carina"
CALL_LOG="$TEST_DIR/carina-calls"
AWSCC_WASM="$TEST_DIR/carina-provider-awscc.wasm"
STDOUT_FILE="$TEST_DIR/helper.stdout"
STDERR_FILE="$TEST_DIR/helper.stderr"
mkdir -p "$STATE_DIR" "$NO_STATE_DIR" "$EMPTY_STATE_DIR" "$OTHER_ERROR_DIR"
touch "$CALL_LOG" "$AWSCC_WASM"

cat > "$STATE_JSON" <<'JSON'
{
  "schema_version": 1,
  "state": "present",
  "resources": [
    {
      "binding": "network.vpc",
      "identity": "main-vpc",
      "provider": "awscc",
      "resource_type": "ec2.Vpc",
      "protected": false,
      "current": {
        "kind": "applied",
        "identifier": "vpc-0123456789abcdef0",
        "attributes": {
          "cidr_block": "10.0.0.0/16",
          "enable_dns_hostnames": false
        }
      },
      "deposed": []
    },
    {
      "binding": "network.subnet",
      "identity": "main-subnet",
      "provider": "awscc",
      "resource_type": "ec2.Subnet",
      "protected": false,
      "current": {
        "kind": "applied",
        "identifier": "subnet-0123456789abcdef0",
        "attributes": {
          "cidr_block": "10.0.1.0/24",
          "vpc_id": "vpc-0123456789abcdef0"
        }
      },
      "deposed": []
    }
  ]
}
JSON

jq '.state = "absent" | .resources = []' "$STATE_JSON" > "$ABSENT_STATE_JSON"
jq '.schema_version = 2' "$STATE_JSON" > "$SCHEMA_V2_JSON"

cat > "$STUB_BIN" <<'STUB'
#!/bin/bash
set -euo pipefail

printf 'call\n' >> "$CALL_LOG"
if [ "${CARINA_TRUST_LOCAL_PROVIDER:-}" != "$EXPECTED_TRUST_GRANT" ]; then
    echo "missing trust grant: ${CARINA_TRUST_LOCAL_PROVIDER:-<unset>}" >&2
    exit 1
fi
if [ "$#" -ne 4 ] || [ "$1" != "state" ] || [ "$2" != "show" ] || \
   [ "$3" != "--json" ] || [ "$4" != "$EXPECTED_STATE_DIR" ]; then
    echo "unexpected arguments: $*" >&2
    exit 1
fi

echo "stub warning on stderr" >&2
if [ "${STUB_FAIL:-0}" = "1" ]; then
    echo "stub carina state failure" >&2
    exit 23
fi

cat "$EXPECTED_STATE_JSON"
STUB
chmod +x "$STUB_BIN"

fail() {
    echo "FAIL: $*" >&2
    exit 1
}

assert_equal() {
    local description="$1"
    local expected="$2"
    local actual="$3"

    if [ "$actual" != "$expected" ]; then
        fail "$description: expected '$expected', got '$actual'"
    fi
}

assert_contains() {
    local description="$1"
    local actual="$2"
    local expected_fragment="$3"

    if [[ "$actual" != *"$expected_fragment"* ]]; then
        fail "$description: expected fragment '$expected_fragment', got '$actual'"
    fi
}

assert_not_contains() {
    local description="$1"
    local actual="$2"
    local unexpected_fragment="$3"

    if [[ "$actual" == *"$unexpected_fragment"* ]]; then
        fail "$description: unexpected fragment '$unexpected_fragment' in '$actual'"
    fi
}

assert_helper_failure() {
    local description="$1"
    local expected_fragment="$2"
    shift 2

    : > "$STDOUT_FILE"
    : > "$STDERR_FILE"
    if "$@" > "$STDOUT_FILE" 2> "$STDERR_FILE"; then
        fail "$description unexpectedly succeeded"
    fi

    assert_equal "$description keeps stdout empty" "" "$(<"$STDOUT_FILE")"
    assert_contains "$description reports the expected error" \
        "$(<"$STDERR_FILE")" "$expected_fragment"
}

(
    export CARINA_BIN="$STUB_BIN"
    export _TEST_AWSCC_WASM="$AWSCC_WASM"
    export CALL_LOG
    unset CARINA_TRUST_LOCAL_PROVIDER
    export EXPECTED_STATE_DIR="$STATE_DIR"
    export EXPECTED_STATE_JSON="$STATE_JSON"
    export EXPECTED_TRUST_GRANT="$AWSCC_WASM=aws"

    source "$HELPERS_SH"

    state_json="$(carina_state_json "$STATE_DIR" 2> "$STDERR_FILE")"
    assert_equal "state JSON schema version" "1" "$(jq -r '.schema_version' <<< "$state_json")"
    assert_not_contains "state JSON stdout stays clean" "$state_json" "stub warning"
    assert_contains "state JSON preserves warning on stderr" \
        "$(<"$STDERR_FILE")" "stub warning on stderr"

    identifiers="$(carina_state_identifiers "$STATE_DIR" 2> "$STDERR_FILE")"
    assert_equal "sorted schema-v1 identifiers" \
        $'subnet-0123456789abcdef0\nvpc-0123456789abcdef0' "$identifiers"
    assert_not_contains "identifier stdout stays clean" "$identifiers" "stub warning"
    assert_contains "identifier read preserves warning on stderr" \
        "$(<"$STDERR_FILE")" "stub warning on stderr"

    cidr_block="$(carina_state_query "$STATE_DIR" \
        '.resources[] | select(.binding == "network.vpc") | .current.attributes.cidr_block' \
        2> "$STDERR_FILE")"
    assert_equal "schema-v1 current attribute" "10.0.0.0/16" "$cidr_block"
    assert_not_contains "query stdout stays clean" "$cidr_block" "stub warning"
    assert_contains "query preserves warning on stderr" \
        "$(<"$STDERR_FILE")" "stub warning on stderr"

    dns_hostnames="$(carina_state_query "$STATE_DIR" \
        '.resources[] | select(.binding == "network.vpc") | .current.attributes.enable_dns_hostnames' \
        2> "$STDERR_FILE")"
    assert_equal "false remains a valid state value" "false" "$dns_hostnames"

    export EXPECTED_STATE_JSON="$SCHEMA_V2_JSON"
    assert_helper_failure "unsupported state schema" "schema_version must be 1" \
        carina_state_json "$STATE_DIR"
    export EXPECTED_STATE_JSON="$STATE_JSON"

    TEST_PASSED=0
    TEST_FAILED=0
    assert_state_queries_equal "assert: distinct identifiers differ" \
        '.resources[] | select(.binding == "network.vpc") | .current.identifier' \
        'vpc' \
        '.resources[] | select(.binding == "network.subnet") | .current.identifier' \
        'subnet' "$STATE_DIR" > "$STDOUT_FILE" 2> "$STDERR_FILE"
    assert_equal "unequal state values record a failure" "1" "$TEST_FAILED"
    assert_contains "mismatch reports both values" "$(<"$STDOUT_FILE")" \
        'vpc=vpc-0123456789abcdef0, subnet=subnet-0123456789abcdef0'

    TEST_PASSED=0
    TEST_FAILED=0
    assert_state_queries_equal "assert: missing identifiers differ" \
        '.resources[] | select(.binding == "missing-left") | .current.identifier' \
        'left' \
        '.resources[] | select(.binding == "missing-right") | .current.identifier' \
        'right' "$STATE_DIR" > "$STDOUT_FILE" 2> "$STDERR_FILE"
    assert_equal "two missing values cannot compare equal" "1" "$TEST_FAILED"
    assert_contains "missing comparison reports query failure" "$(<"$STDOUT_FILE")" \
        'FAIL (state query failed)'

    assert_helper_failure "multiple state values" \
        "returned 2 values (expected exactly one)" \
        carina_state_query "$STATE_DIR" '.resources[] | .current.kind'

    assert_helper_failure "missing state value" "state query returned no value" \
        carina_state_query "$STATE_DIR" \
        '.resources[] | select(.binding == "network.vpc") | .current.attributes.missing'

    EMPTY_ATTRIBUTE_JSON="$TEST_DIR/empty-attribute.json"
    jq '.resources[0].current.attributes.cidr_block = ""' \
        "$STATE_JSON" > "$EMPTY_ATTRIBUTE_JSON"
    export EXPECTED_STATE_JSON="$EMPTY_ATTRIBUTE_JSON"

    assert_helper_failure "empty state attribute" "state query returned no value" \
        carina_state_query "$STATE_DIR" \
        '.resources[] | select(.binding == "network.vpc") | .current.attributes.cidr_block'

    MISSING_IDENTIFIER_JSON="$TEST_DIR/missing-identifier.json"
    jq '.resources[0].current.identifier = null' \
        "$STATE_JSON" > "$MISSING_IDENTIFIER_JSON"
    export EXPECTED_STATE_JSON="$MISSING_IDENTIFIER_JSON"

    assert_helper_failure "applied row missing identifier" "missing current.identifier" \
        carina_state_identifiers "$STATE_DIR"
    assert_contains "missing identifier names its binding" \
        "$(<"$STDERR_FILE")" 'binding "network.vpc"'

    NON_APPLIED_JSON="$TEST_DIR/non-applied.json"
    jq '
        .resources[0].current = {"kind": "none", "identifier": null, "attributes": {}}
        | .resources[1].current = {
            "kind": "create_in_flight",
            "identifier": "subnet-0123456789abcdef0",
            "attributes": {}
          }
    ' "$STATE_JSON" > "$NON_APPLIED_JSON"
    export EXPECTED_STATE_JSON="$NON_APPLIED_JSON"

    non_applied_json="$(carina_state_json "$STATE_DIR" 2> "$STDERR_FILE")"
    assert_equal "raw state read permits non-applied rows" "none" \
        "$(jq -r '.resources[0].current.kind' <<< "$non_applied_json")"
    assert_contains "raw non-applied read preserves warning on stderr" \
        "$(<"$STDERR_FILE")" "stub warning on stderr"

    non_applied_count="$(carina_state_query "$STATE_DIR" '.resources | length' \
        2> "$STDERR_FILE")"
    assert_equal "plain query permits non-applied rows" "2" "$non_applied_count"
    assert_contains "non-applied query preserves warning on stderr" \
        "$(<"$STDERR_FILE")" "stub warning on stderr"

    assert_helper_failure "non-applied resource rows" \
        "state contains non-applied resource rows" \
        carina_state_identifiers "$STATE_DIR"
    non_applied_error="$(<"$STDERR_FILE")"
    assert_contains "none row names binding, identity, and kind" "$non_applied_error" \
        'binding "network.vpc" identity "main-vpc" has current.kind "none"'
    assert_contains "create-in-flight row names binding, identity, and kind" "$non_applied_error" \
        'binding "network.subnet" identity "main-subnet" has current.kind "create_in_flight"'
    assert_not_contains "non-applied rows are not reported as missing identifiers" \
        "$non_applied_error" "missing current.identifier"

    EMPTY_STATE_JSON="$TEST_DIR/empty-state.json"
    jq '.resources = []' "$STATE_JSON" > "$EMPTY_STATE_JSON"
    export EXPECTED_STATE_JSON="$EMPTY_STATE_JSON"
    assert_helper_failure "empty state" "state contains no resources" \
        carina_state_identifiers "$STATE_DIR"

    export EXPECTED_STATE_JSON="$STATE_JSON"
    export STUB_FAIL=1
    assert_helper_failure "state JSON failure" "stub carina state failure" \
        carina_state_json "$STATE_DIR"
    assert_helper_failure "state query failure" "stub carina state failure" \
        carina_state_query "$STATE_DIR" '.schema_version'
    assert_helper_failure "identifier failure" "stub carina state failure" \
        carina_state_identifiers "$STATE_DIR"

    # A false condition creates an absent state. That is distinct from a
    # present state containing zero resources and from every read error.
    unset STUB_FAIL
    export EXPECTED_STATE_DIR="$NO_STATE_DIR"
    export EXPECTED_STATE_JSON="$ABSENT_STATE_JSON"
    calls_before="$(wc -l < "$CALL_LOG" | tr -d ' ')"
    TEST_PASSED=0
    TEST_FAILED=0
    assert_no_state "assert: no state for false condition" "$NO_STATE_DIR" \
        > "$STDOUT_FILE" 2> "$STDERR_FILE"
    assert_equal "no-state assertion passes" "1" "$TEST_PASSED"
    assert_equal "no-state assertion records no failure" "0" "$TEST_FAILED"
    assert_contains "no-state assertion reports OK" "$(<"$STDOUT_FILE")" "OK"
    assert_contains "absent-state read preserves warning on stderr" \
        "$(<"$STDERR_FILE")" "stub warning on stderr"
    calls_after="$(wc -l < "$CALL_LOG" | tr -d ' ')"
    assert_equal "no-state assertion invokes carina" "$((calls_before + 1))" "$calls_after"

    TEST_PASSED=0
    TEST_FAILED=0
    assert_state_resource_count "assert: strict zero-resource read" "0" "$NO_STATE_DIR" \
        > "$STDOUT_FILE" 2> "$STDERR_FILE"
    assert_equal "absent-state resource count cannot pass" "0" "$TEST_PASSED"
    assert_equal "absent-state resource count records failure" "1" "$TEST_FAILED"
    assert_contains "absent-state resource count reports query failure" \
        "$(<"$STDOUT_FILE")" "FAIL (state query failed)"
    assert_contains "absent-state resource count explains rejection" \
        "$(<"$STDERR_FILE")" "state is absent"

    export EXPECTED_STATE_DIR="$EMPTY_STATE_DIR"
    export EXPECTED_STATE_JSON="$EMPTY_STATE_JSON"
    TEST_PASSED=0
    TEST_FAILED=0
    assert_no_state "assert: empty state is still state" "$EMPTY_STATE_DIR" \
        > "$STDOUT_FILE" 2> "$STDERR_FILE"
    assert_equal "empty-state no-state assertion cannot pass" "0" "$TEST_PASSED"
    assert_equal "empty-state no-state assertion records failure" "1" "$TEST_FAILED"
    assert_contains "empty-state no-state assertion reports existing state" \
        "$(<"$STDOUT_FILE")" "FAIL (state is 'present')"

    TEST_PASSED=0
    TEST_FAILED=0
    assert_state_resource_count "assert: zero resources in real state" "0" "$EMPTY_STATE_DIR" \
        > "$STDOUT_FILE" 2> "$STDERR_FILE"
    assert_equal "real empty-state resource count passes" "1" "$TEST_PASSED"
    assert_equal "real empty-state resource count records no failure" "0" "$TEST_FAILED"

    export EXPECTED_STATE_DIR="$OTHER_ERROR_DIR"
    export EXPECTED_STATE_JSON="$STATE_JSON"
    export STUB_FAIL=1
    TEST_PASSED=0
    TEST_FAILED=0
    assert_no_state "assert: other state error is not no-state" "$OTHER_ERROR_DIR" \
        > "$STDOUT_FILE" 2> "$STDERR_FILE"
    assert_equal "other-error no-state assertion cannot pass" "0" "$TEST_PASSED"
    assert_equal "other-error no-state assertion records failure" "1" "$TEST_FAILED"
    assert_contains "other-error no-state assertion reports unexpected error" \
        "$(<"$STDOUT_FILE")" "FAIL (state read failed)"
    assert_contains "other-error no-state assertion preserves carina error" \
        "$(<"$STDERR_FILE")" "stub carina state failure"
    unset STUB_FAIL

)

echo "state_helpers: OK"
