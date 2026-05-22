#!/bin/bash
# Negative acceptance test for the allowed_account_ids guard.
#
# Usage:
#   env AWS_PROFILE="<profile>" ./run.sh
#
# basic.crn pins `allowed_account_ids` to the all-zeros account, which no
# real AWS credentials can match. `carina apply` MUST fail fast with an
# "AWS account mismatch" error before any CloudControl call.
#
# This is a NEGATIVE test: a non-zero exit is the success condition, so it
# cannot run inside run-tests.sh (which treats any non-zero exit as FAIL).
# The presence of this run.sh makes run-tests.sh skip the directory.
#
# AWS authentication:
#   Set AWS_PROFILE to one of the carina-test-00X profiles. The first run
#   in a session needs `aws sso login --sso-session carina`.

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../../.." && pwd)"

source "$SCRIPT_DIR/../shared/_helpers.sh"

# The EXIT trap in _helpers.sh runs cleanup() against ACTIVE_WORK_DIR; we
# manage our own temp dir, so override the trap with our own cleanup.
WORK_DIR=""
negative_cleanup() {
    if [ -n "$WORK_DIR" ] && [ -d "$WORK_DIR" ]; then
        rm -rf "$WORK_DIR"
    fi
}
trap negative_cleanup EXIT INT TERM

# ── with_account_creds: resolve credentials for $AWS_PROFILE into env vars ──
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

echo "allowed_account_ids guard negative acceptance test"
echo "════════════════════════════════════════"
echo ""

PASSED=0
FAILED=0

WORK_DIR=$(mktemp -d)
INJECTED_DIR=$(inject_provider_source "$SCRIPT_DIR/basic.crn")
cp "$INJECTED_DIR/main.crn" "$WORK_DIR/main.crn"
rm -rf "$INJECTED_DIR"

# carina init must succeed — the guard fires during apply, not init.
printf "  %-55s " "init"
if (cd "$WORK_DIR" && with_account_creds "$CARINA_BIN" init . >/dev/null 2>&1); then
    echo "OK"
    PASSED=$((PASSED + 1))
else
    echo "FAIL"
    echo "  ERROR: carina init failed; cannot run the guard test"
    FAILED=$((FAILED + 1))
fi

# apply MUST fail fast. Capture exit code and output.
printf "  %-55s " "apply rejected (non-zero exit)"
set +e
APPLY_OUTPUT=$(cd "$WORK_DIR" && with_account_creds "$CARINA_BIN" apply --auto-approve . 2>&1)
APPLY_RC=$?
set -e

if [ "$APPLY_RC" -ne 0 ]; then
    echo "OK"
    PASSED=$((PASSED + 1))
else
    echo "FAIL"
    echo "  ERROR: apply succeeded but should have been rejected by the guard"
    FAILED=$((FAILED + 1))
fi

# The error must name the guard and the expected all-zeros account.
printf "  %-55s " "error names 'AWS account mismatch'"
if echo "$APPLY_OUTPUT" | grep -q "AWS account mismatch"; then
    echo "OK"
    PASSED=$((PASSED + 1))
else
    echo "FAIL"
    echo "  ERROR: output did not contain 'AWS account mismatch':"
    echo "$APPLY_OUTPUT" | sed 's/^/    /'
    FAILED=$((FAILED + 1))
fi

printf "  %-55s " "error names expected account 000000000000"
if echo "$APPLY_OUTPUT" | grep -q "000000000000"; then
    echo "OK"
    PASSED=$((PASSED + 1))
else
    echo "FAIL"
    echo "  ERROR: output did not name the expected account 000000000000:"
    echo "$APPLY_OUTPUT" | sed 's/^/    /'
    FAILED=$((FAILED + 1))
fi

echo ""
echo "════════════════════════════════════════"
echo "Total: $PASSED passed, $FAILED failed"
echo "════════════════════════════════════════"

if [ "$FAILED" -gt 0 ]; then
    exit 1
fi
