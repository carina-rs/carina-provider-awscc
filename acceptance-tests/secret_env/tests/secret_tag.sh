#!/bin/bash
# Test: secret(env()) marks value as secret and redacts public state output
source "$(dirname "$0")/../../shared/_helpers.sh"

echo "Test: secret(env()) in tag value"
echo ""

WORK_DIR=$(mktemp -d)
ACTIVE_WORK_DIR="$WORK_DIR"
cp "$SCRIPT_DIR/secret_tag.crn" "$WORK_DIR/main.crn"
cd "$WORK_DIR"

# Set the secret value via environment variable
export CARINA_TEST_SECRET_VALUE="super-secret-value"

run_step "step1: apply" "$CARINA_BIN" apply --auto-approve .
run_step "step2: plan-verify" "$CARINA_BIN" plan .

assert_state_resource_count "assert: 1 resource created" "1" "$WORK_DIR"

# Read raw state on purpose: this checks bytes at rest; state show re-serializes.
printf "  %-50s" "assert: state does not contain plaintext"
set +e
LC_ALL=C grep -Fq -- "super-secret-value" "$WORK_DIR/carina.state.json"
case $? in
    0)
        echo "FAIL (plaintext found in state!)"
        TEST_FAILED=$((TEST_FAILED + 1))
        ;;
    1)
        echo "OK"
        TEST_PASSED=$((TEST_PASSED + 1))
        ;;
    *)
        echo "FAIL (could not inspect raw state)"
        TEST_FAILED=$((TEST_FAILED + 1))
        ;;
esac
set -e

# Public state must expose only the schema-v1 redaction marker.
assert_state_value "assert: secret tag is redacted" \
    '.resources[0].current.attributes.tags.SecretTag == {"redacted": true}' \
    'true' "$WORK_DIR"

# Verify the actual AWS resource has the real value (not the hash)
# We check by verifying plan is idempotent (no changes) — if the value
# were wrong, plan would detect a diff
run_step "step3: plan still idempotent" "$CARINA_BIN" plan .

echo "  Cleanup: destroying resources..."
"$CARINA_BIN" destroy --auto-approve . > /dev/null 2>&1 || true
rm -rf "$WORK_DIR"
ACTIVE_WORK_DIR=""

finish_test
