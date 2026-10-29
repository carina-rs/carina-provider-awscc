#!/bin/bash
# Shared local-provider setup for the acceptance harness.

# Resolve the named provider-path variables to absolute paths, verify that each
# configured WASM exists, and grant every path AWS trust to child processes.
# Empty variables are ignored so callers can include optional providers.
# Args: provider path variable names
configure_trusted_local_provider_wasms() {
    local provider_var
    local provider_path
    local provider_dir
    local trust_grants=""

    for provider_var in "$@"; do
        provider_path="${!provider_var:-}"
        if [ -z "$provider_path" ]; then
            continue
        fi

        if [[ "$provider_path" != *.wasm ]]; then
            echo "ERROR: $provider_var points to a non-WASM provider binary: $provider_path" >&2
            echo "       Provider binaries must be .wasm components." >&2
            return 1
        fi
        if [ ! -f "$provider_path" ]; then
            echo "ERROR: configured provider WASM does not exist: $provider_var=$provider_path" >&2
            return 1
        fi

        provider_dir="$(cd "$(dirname "$provider_path")" && pwd -P)"
        provider_path="$provider_dir/${provider_path##*/}"
        if [[ "$provider_path" == *:* ]]; then
            echo "ERROR: provider WASM path in $provider_var contains ':', which separates CARINA_TRUST_LOCAL_PROVIDER entries: $provider_path" >&2
            return 1
        fi
        printf -v "$provider_var" '%s' "$provider_path"
        export "$provider_var"

        trust_grants="${trust_grants:+$trust_grants:}${provider_path}=aws"
    done

    if [ -n "$trust_grants" ]; then
        CARINA_TRUST_LOCAL_PROVIDER="${CARINA_TRUST_LOCAL_PROVIDER:+$CARINA_TRUST_LOCAL_PROVIDER:}$trust_grants"
        export CARINA_TRUST_LOCAL_PROVIDER
    fi
}

configure_awscc_provider_environment() {
    if [ -z "${PROJECT_ROOT:-}" ]; then
        echo "ERROR: PROJECT_ROOT must be set before configuring the awscc provider" >&2
        return 1
    fi

    AWSCC_PROVIDER_BIN="${AWSCC_PROVIDER_BIN:-$PROJECT_ROOT/target/wasm32-wasip2/release/carina-provider-awscc.wasm}"
    configure_trusted_local_provider_wasms AWSCC_PROVIDER_BIN
}
