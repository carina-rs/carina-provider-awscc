#!/bin/bash
# Resolve the local awscc provider WASM used by the acceptance harness and
# grant that exact file AWS-family trust to every carina subprocess.
#
# Callers must set PROJECT_ROOT before sourcing this file. The top-level runner
# sources it after building the provider; custom suites source it through
# _helpers.sh after the provider build exists.

_TEST_AWSCC_WASM="${_TEST_AWSCC_WASM:-$PROJECT_ROOT/target/wasm32-wasip2/release/carina-provider-awscc.wasm}"
_TEST_OPERATOR_TRUST_LOCAL_PROVIDER="${CARINA_TRUST_LOCAL_PROVIDER:-}"

PROVIDER_VERSION=$(grep '^version = ' "$PROJECT_ROOT/Cargo.toml" | head -1 | sed 's/version = "\(.*\)"/\1/')

resolve_provider_wasm_path() {
    local variable_name="$1"
    local configured_path="${!variable_name:-}"
    local provider_dir

    if [ -z "$configured_path" ]; then
        return 0
    fi

    if [[ "$configured_path" != /* ]]; then
        configured_path="$(pwd -P)/$configured_path"
    fi

    if [ ! -e "$configured_path" ]; then
        echo "ERROR: configured provider WASM does not exist ($variable_name): $configured_path" >&2
        return 1
    fi
    if [ ! -f "$configured_path" ]; then
        echo "ERROR: configured provider WASM is not a file ($variable_name): $configured_path" >&2
        return 1
    fi
    if [[ "$configured_path" == *:* ]]; then
        echo "ERROR: $variable_name provider WASM path contains ':' and cannot be used in CARINA_TRUST_LOCAL_PROVIDER: $configured_path" >&2
        return 1
    fi
    if [[ "$configured_path" != *.wasm ]]; then
        echo "ERROR: $variable_name points to a non-WASM binary: $configured_path" >&2
        echo "       Provider binaries must be .wasm components." >&2
        return 1
    fi

    provider_dir="$(cd "$(dirname "$configured_path")" && pwd -P)"
    configured_path="$provider_dir/$(basename "$configured_path")"
    printf -v "$variable_name" '%s' "$configured_path"
}

export_local_provider_trust() {
    local grants=""

    resolve_provider_wasm_path _TEST_AWSCC_WASM || return 1

    if [ -n "${_TEST_AWSCC_WASM:-}" ]; then
        grants="$_TEST_AWSCC_WASM=aws"
    fi

    CARINA_TRUST_LOCAL_PROVIDER="${_TEST_OPERATOR_TRUST_LOCAL_PROVIDER}${_TEST_OPERATOR_TRUST_LOCAL_PROVIDER:+:}$grants"
    export CARINA_TRUST_LOCAL_PROVIDER
}

_inject_provider_source_filter() {
    sed \
        -e '/^provider awscc {/a\
  source = "file://'"$_TEST_AWSCC_WASM"'"\
  version = "'"$PROVIDER_VERSION"'"'
}

# inject_provider_source: Create a temporary directory containing a .crn file
# with the local awscc provider source/version injected and assign it to a
# caller variable.
# Args: original_crn_file output_variable
inject_provider_source() {
    local original="$1"
    local output_variable="$2"
    local tmp_dir

    tmp_dir=$(mktemp -d) || return 1

    if ! _inject_provider_source_filter < "$original" > "$tmp_dir/main.crn"; then
        rm -rf "$tmp_dir"
        return 1
    fi

    printf -v "$output_variable" '%s' "$tmp_dir"
}

export_local_provider_trust || {
    return 1 2>/dev/null || exit 1
}
