#!/bin/bash
# Test trusted-local-provider setup without invoking carina or AWS.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
HELPERS_FILE="$SCRIPT_DIR/../shared/_helpers.sh"
PROVIDER_ENV_FILE="$SCRIPT_DIR/../shared/_provider_env.sh"
EXPECTED_PROVIDER_VERSION=$(grep '^version = ' "$PROJECT_ROOT/Cargo.toml" | head -1 | sed 's/version = "\(.*\)"/\1/')

WORK_DIR="$(mktemp -d)"
cleanup() {
    rm -rf "$WORK_DIR"
}
trap cleanup EXIT

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

absolute_file_path() {
    local path="$1"
    local directory
    directory="$(cd "$(dirname "$path")" && pwd -P)"
    printf '%s/%s\n' "$directory" "${path##*/}"
}

file_mode() {
    local path="$1"
    local mode

    if mode=$(stat -f '%Lp' "$path" 2>/dev/null) && [[ "$mode" =~ ^[0-7]+$ ]]; then
        printf '%s\n' "$mode"
    else
        stat -c '%a' "$path"
    fi
}

run_helpers_case() {
    local working_dir="$1"
    local awscc_wasm="$2"
    local preset_value="$3"

    (
        cd "$working_dir"
        env \
            CARINA_BIN="$CARINA_STUB" \
            _TEST_AWSCC_WASM="$awscc_wasm" \
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
OPERATOR_WASM="$WORK_DIR/providers/operator.wasm"
RELATIVE_WASM="$WORK_DIR/relative/awscc.wasm"
COLON_WASM="$WORK_DIR/providers/colon:awscc.wasm"
INJECTION_SOURCE="$WORK_DIR/injection.crn"
touch "$CARINA_STUB" "$AWSCC_WASM" "$OPERATOR_WASM" "$RELATIVE_WASM" "$COLON_WASM"

cat > "$INJECTION_SOURCE" <<'CRN'
provider awscc {
  region = aws.Region.ap_northeast_1
}

provider aws {
  region = aws.Region.ap_northeast_1
}
CRN

AWSCC_WASM_ABS="$(absolute_file_path "$AWSCC_WASM")"
OPERATOR_WASM_ABS="$(absolute_file_path "$OPERATOR_WASM")"
RELATIVE_WASM_ABS="$(absolute_file_path "$RELATIVE_WASM")"

actual="$(run_helpers_case "$WORK_DIR" "$AWSCC_WASM" "")"
assert_equal "single provider grant" "$AWSCC_WASM_ABS=aws" "$actual"

actual="$(run_helpers_case "$WORK_DIR" "relative/awscc.wasm" "")"
assert_equal "relative provider path" "$RELATIVE_WASM_ABS=aws" "$actual"

preset="$OPERATOR_WASM_ABS=github"
actual="$(run_helpers_case "$WORK_DIR" "$AWSCC_WASM" "$preset")"
assert_equal "operator preset value" "$preset:$AWSCC_WASM_ABS=aws" "$actual"

COLON_OUTPUT="$WORK_DIR/colon-output"
if run_helpers_case "$WORK_DIR" "$COLON_WASM" "" >"$COLON_OUTPUT" 2>&1; then
    fail "provider WASM path containing ':' unexpectedly succeeded"
fi
if ! grep -F "ERROR: _TEST_AWSCC_WASM provider WASM path contains ':' and cannot be used in CARINA_TRUST_LOCAL_PROVIDER:" "$COLON_OUTPUT" >/dev/null; then
    fail "provider WASM path containing ':' did not produce the expected error"
fi

MISSING_OUTPUT="$WORK_DIR/missing-output"
if run_helpers_case "$WORK_DIR" "missing/awscc.wasm" "" >"$MISSING_OUTPUT" 2>&1; then
    fail "missing provider WASM unexpectedly succeeded"
fi
if ! grep -F "ERROR: configured provider WASM does not exist (_TEST_AWSCC_WASM):" "$MISSING_OUTPUT" >/dev/null; then
    fail "missing provider WASM did not produce the expected error"
fi

INJECTED_DIR="$(env \
    PROJECT_ROOT="$PROJECT_ROOT" \
    _TEST_AWSCC_WASM="$AWSCC_WASM" \
    PROVIDER_ENV_FILE="$PROVIDER_ENV_FILE" \
    INJECTION_SOURCE="$INJECTION_SOURCE" \
    /bin/bash -c '
        source "$PROVIDER_ENV_FILE"
        injected_dir=""
        inject_provider_source "$INJECTION_SOURCE" injected_dir
        printf "%s\n" "$injected_dir"
    ')"

AWSCC_BLOCK="$(sed -n '/^provider awscc {/,/^}/p' "$INJECTED_DIR/main.crn")"
AWS_BLOCK="$(sed -n '/^provider aws {/,/^}/p' "$INJECTED_DIR/main.crn")"
grep -F "source = \"file://$AWSCC_WASM_ABS\"" <<< "$AWSCC_BLOCK" >/dev/null || \
    fail "awscc provider source was not injected"
grep -F "version = \"$EXPECTED_PROVIDER_VERSION\"" <<< "$AWSCC_BLOCK" >/dev/null || \
    fail "awscc provider version was not injected"
if grep -q '^  source = ' <<< "$AWS_BLOCK" || grep -q '^  version = ' <<< "$AWS_BLOCK"; then
    fail "aws provider block received metadata for the awscc provider"
fi
rm -rf "$INJECTED_DIR"

IN_PLACE_DIR="$WORK_DIR/in-place"
mkdir -p "$IN_PLACE_DIR"
cp "$INJECTION_SOURCE" "$IN_PLACE_DIR/main.crn"
chmod 640 "$IN_PLACE_DIR/main.crn"
MODE_BEFORE="$(file_mode "$IN_PLACE_DIR/main.crn")"
env \
    CARINA_BIN="$CARINA_STUB" \
    _TEST_AWSCC_WASM="$AWSCC_WASM" \
    HELPERS_FILE="$HELPERS_FILE" \
    TEST_SCRIPT_DIR="$SCRIPT_DIR" \
    IN_PLACE_DIR="$IN_PLACE_DIR" \
    /bin/bash -c '
        SCRIPT_DIR="$TEST_SCRIPT_DIR"
        source "$HELPERS_FILE"
        inject_provider_source_dir "$IN_PLACE_DIR"
    '
MODE_AFTER="$(file_mode "$IN_PLACE_DIR/main.crn")"
assert_equal "in-place injection preserves mode" "$MODE_BEFORE" "$MODE_AFTER"
grep -F "source = \"file://$AWSCC_WASM_ABS\"" "$IN_PLACE_DIR/main.crn" >/dev/null || \
    fail "in-place injector did not use the shared provider filter"

if env \
    CARINA_BIN="$CARINA_STUB" \
    _TEST_AWSCC_WASM="$AWSCC_WASM" \
    HELPERS_FILE="$HELPERS_FILE" \
    TEST_SCRIPT_DIR="$SCRIPT_DIR" \
    IN_PLACE_DIR="$IN_PLACE_DIR" \
    /bin/bash -c '
        SCRIPT_DIR="$TEST_SCRIPT_DIR"
        source "$HELPERS_FILE"
        inject_provider_source_dir() { return 23; }
        carina_work_dir_initialized() { return 0; }
        prepare_work_dir "$IN_PLACE_DIR"
    '; then
    fail "prepare_work_dir did not propagate provider-injection failure"
fi

echo "trusted_local_provider: OK"
