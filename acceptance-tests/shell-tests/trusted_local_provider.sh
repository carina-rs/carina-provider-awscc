#!/bin/bash
# Test trusted-local-provider setup without invoking carina or AWS.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
HELPERS_FILE="$SCRIPT_DIR/../shared/_helpers.sh"
PROVIDER_ENV_FILE="$SCRIPT_DIR/../shared/_provider_env.sh"

WORK_DIR="$(mktemp -d)"
cleanup() {
    rm -rf "$WORK_DIR"
}
trap cleanup EXIT

fail() {
    echo "FAIL: $*" >&2
    exit 1
}

assert_equals() {
    local description="$1"
    local expected="$2"
    local actual="$3"

    if [ "$actual" != "$expected" ]; then
        fail "$description: expected '$expected', got '$actual'"
    fi
}

absolute_file_path() {
    local path="$1"
    local directory
    directory="$(cd "$(dirname "$path")" && pwd -P)"
    printf '%s/%s\n' "$directory" "${path##*/}"
}

run_helpers_case() {
    local working_dir="$1"
    local awscc_wasm="$2"
    local preset_value="$3"

    (
        cd "$working_dir"
        env \
            CARINA_BIN="$CARINA_STUB" \
            AWSCC_PROVIDER_BIN="$awscc_wasm" \
            CARINA_TRUST_LOCAL_PROVIDER="$preset_value" \
            HELPERS_FILE="$HELPERS_FILE" \
            TEST_SCRIPT_DIR="$SCRIPT_DIR" \
            /bin/bash -c '
                SCRIPT_DIR="$TEST_SCRIPT_DIR"
                source "$HELPERS_FILE"
                printenv CARINA_TRUST_LOCAL_PROVIDER
            '
    )
}

mkdir -p "$WORK_DIR/bin" "$WORK_DIR/providers" "$WORK_DIR/relative"
CARINA_STUB="$WORK_DIR/bin/carina"
AWSCC_WASM="$WORK_DIR/providers/awscc.wasm"
AWS_WASM="$WORK_DIR/providers/aws.wasm"
OPERATOR_WASM="$WORK_DIR/providers/operator.wasm"
RELATIVE_WASM="$WORK_DIR/relative/awscc.wasm"
COLON_WASM="$WORK_DIR/providers/colon:awscc.wasm"
touch "$CARINA_STUB" "$AWSCC_WASM" "$AWS_WASM" "$OPERATOR_WASM" "$RELATIVE_WASM" "$COLON_WASM"

AWSCC_WASM_ABS="$(absolute_file_path "$AWSCC_WASM")"
AWS_WASM_ABS="$(absolute_file_path "$AWS_WASM")"
OPERATOR_WASM_ABS="$(absolute_file_path "$OPERATOR_WASM")"
RELATIVE_WASM_ABS="$(absolute_file_path "$RELATIVE_WASM")"

actual="$(run_helpers_case "$WORK_DIR" "$AWSCC_WASM" "")"
assert_equals "single provider grant" "$AWSCC_WASM_ABS=aws" "$actual"

actual="$(env \
    CARINA_BIN="$CARINA_STUB" \
    CARINA_TRUST_LOCAL_PROVIDER="" \
    _TEST_AWS_WASM="$AWS_WASM" \
    _TEST_AWSCC_WASM="$AWSCC_WASM" \
    PROVIDER_ENV_FILE="$PROVIDER_ENV_FILE" \
    /bin/bash -c '
        source "$PROVIDER_ENV_FILE"
        configure_trusted_local_provider_wasms _TEST_AWS_WASM _TEST_AWSCC_WASM
        printenv CARINA_TRUST_LOCAL_PROVIDER
    ')"
assert_equals "two provider grants" "$AWS_WASM_ABS=aws:$AWSCC_WASM_ABS=aws" "$actual"

actual="$(run_helpers_case "$WORK_DIR" "relative/awscc.wasm" "")"
assert_equals "relative provider path" "$RELATIVE_WASM_ABS=aws" "$actual"

preset="$OPERATOR_WASM_ABS=github"
actual="$(run_helpers_case "$WORK_DIR" "$AWSCC_WASM" "$preset")"
assert_equals "operator preset value" "$preset:$AWSCC_WASM_ABS=aws" "$actual"

COLON_OUTPUT="$WORK_DIR/colon-output"
if run_helpers_case "$WORK_DIR" "$COLON_WASM" "" >"$COLON_OUTPUT" 2>&1; then
    fail "provider WASM path containing ':' unexpectedly succeeded"
fi
if ! grep -F "ERROR: provider WASM path in AWSCC_PROVIDER_BIN contains ':', which separates CARINA_TRUST_LOCAL_PROVIDER entries:" "$COLON_OUTPUT" >/dev/null; then
    fail "provider WASM path containing ':' did not produce the expected error"
fi

MISSING_OUTPUT="$WORK_DIR/missing-output"
if run_helpers_case "$WORK_DIR" "missing/awscc.wasm" "" >"$MISSING_OUTPUT" 2>&1; then
    fail "missing provider WASM unexpectedly succeeded"
fi
if ! grep -F "ERROR: configured provider WASM does not exist: AWSCC_PROVIDER_BIN=missing/awscc.wasm" "$MISSING_OUTPUT" >/dev/null; then
    fail "missing provider WASM did not produce the expected error"
fi

echo "trusted_local_provider: OK"
