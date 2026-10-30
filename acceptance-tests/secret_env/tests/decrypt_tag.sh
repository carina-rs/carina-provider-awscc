#!/bin/bash
# Test: secret(decrypt()) — decrypted value is redacted in public state output
# Requires carina-test-000 account with alias/carina-test KMS key
source "$(dirname "$0")/../../shared/_helpers.sh"

echo "Test: secret(decrypt()) with KMS"
echo ""

register_acceptance_work_dir "secret_env/decrypt_tag.crn" "$SCRIPT_DIR/decrypt_tag.crn" WORK_DIR
ACTIVE_WORK_DIR="$WORK_DIR"
cd "$WORK_DIR"

run_step "step1: apply" "$CARINA_BIN" apply --auto-approve .
run_step "step2: plan-verify" "$CARINA_BIN" plan .

assert_state_resource_count "assert: 1 resource created" "1" "$WORK_DIR"

# Read raw state on purpose: this checks bytes at rest; state show re-serializes.
printf "  %-50s" "assert: state does not contain plaintext"
set +e
LC_ALL=C grep -Fq -- "decrypt-test-value" "$WORK_DIR/carina.state.json"
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
assert_state_value "assert: decrypted tag is redacted" \
    '.resources[0].current.attributes.tags.DecryptedTag == {"redacted": true}' \
    'true' "$WORK_DIR"

# Verify the actual AWS resource has the decrypted value
# (plan idempotency confirms the value was sent correctly)
run_step "step3: plan still idempotent" "$CARINA_BIN" plan .

echo "  Cleanup: destroying resources..."
"$CARINA_BIN" destroy --auto-approve . > /dev/null 2>&1 || true
release_acceptance_work_dir "$WORK_DIR" || TEST_FAILED=$((TEST_FAILED + 1))
ACTIVE_WORK_DIR=""

finish_test
