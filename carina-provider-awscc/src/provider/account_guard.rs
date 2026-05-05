//! Account ID guard.
//!
//! Mirrors the Terraform AWS provider's `allowed_account_ids` /
//! `forbidden_account_ids` shape: lets a `.crn` pin which AWS account a
//! provider block is allowed (or forbidden) to operate against. The
//! caller's account is read once via `sts:GetCallerIdentity` and cached
//! on the provider instance; the check runs eagerly so a wrong-account
//! `apply` aborts before any CloudControl read or mutation.

/// Validate the caller's AWS account ID against the provider's
/// `allowed_account_ids` / `forbidden_account_ids` lists.
///
/// Both lists are independent and may be empty. Semantics:
///
/// - Both empty/unset → `Ok(())` (no check, current behavior).
/// - `allowed` non-empty and `account_id` not in it → `Err`.
/// - `forbidden` non-empty and `account_id` in it → `Err`.
///
/// Error messages always name BOTH the offending list and the caller's
/// actual account ID so an operator can tell at a glance which rule
/// fired and which credentials are loaded.
pub fn validate_account_against_lists(
    account_id: &str,
    allowed: &[String],
    forbidden: &[String],
) -> Result<(), String> {
    if !allowed.is_empty() && !allowed.iter().any(|a| a == account_id) {
        return Err(format!(
            "AWS account ID '{}' is not in the provider's allowed_account_ids {:?}. \
             Refusing to operate against this account. \
             Check the AWS credentials in your environment.",
            account_id, allowed
        ));
    }

    if !forbidden.is_empty() && forbidden.iter().any(|a| a == account_id) {
        return Err(format!(
            "AWS account ID '{}' is listed in the provider's forbidden_account_ids {:?}. \
             Refusing to operate against this account. \
             Check the AWS credentials in your environment.",
            account_id, forbidden
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn both_empty_is_ok() {
        assert!(validate_account_against_lists("123456789012", &[], &[]).is_ok());
    }

    #[test]
    fn allowed_match_is_ok() {
        let allowed = vec!["123456789012".to_string()];
        assert!(validate_account_against_lists("123456789012", &allowed, &[]).is_ok());
    }

    #[test]
    fn allowed_mismatch_is_err_naming_both() {
        let allowed = vec!["111111111111".to_string()];
        let err = validate_account_against_lists("222222222222", &allowed, &[]).unwrap_err();
        // Error must name both expected list and actual account.
        assert!(
            err.contains("222222222222"),
            "actual account missing: {err}"
        );
        assert!(err.contains("111111111111"), "allowed list missing: {err}");
        assert!(
            err.contains("allowed_account_ids"),
            "kind label missing: {err}"
        );
    }

    #[test]
    fn forbidden_match_is_err_naming_both() {
        let forbidden = vec!["999999999999".to_string()];
        let err = validate_account_against_lists("999999999999", &[], &forbidden).unwrap_err();
        assert!(
            err.contains("999999999999"),
            "actual account missing: {err}"
        );
        assert!(
            err.contains("forbidden_account_ids"),
            "kind label missing: {err}"
        );
    }

    #[test]
    fn forbidden_no_match_is_ok() {
        let forbidden = vec!["999999999999".to_string()];
        assert!(validate_account_against_lists("123456789012", &[], &forbidden).is_ok());
    }

    #[test]
    fn allowed_takes_precedence_over_forbidden_for_clear_signal() {
        // When the caller is in allowed but also in forbidden, allowed
        // mismatch fires first only if the caller is NOT in allowed.
        // Here the caller IS in allowed, so allowed passes — but is
        // also in forbidden, so forbidden fires.
        let allowed = vec!["123456789012".to_string()];
        let forbidden = vec!["123456789012".to_string()];
        let err = validate_account_against_lists("123456789012", &allowed, &forbidden).unwrap_err();
        assert!(err.contains("forbidden_account_ids"), "got: {err}");
    }

    #[test]
    fn allowed_with_multiple_accounts_one_match_is_ok() {
        let allowed = vec![
            "111111111111".to_string(),
            "222222222222".to_string(),
            "333333333333".to_string(),
        ];
        assert!(validate_account_against_lists("222222222222", &allowed, &[]).is_ok());
    }

    #[test]
    fn allowed_zero_account_does_not_accidentally_match() {
        // Acceptance scenario from the issue: allowed_account_ids =
        // ['000000000000'] must reject any real-credential account.
        let allowed = vec!["000000000000".to_string()];
        assert!(validate_account_against_lists("123456789012", &allowed, &[]).is_err());
    }
}
