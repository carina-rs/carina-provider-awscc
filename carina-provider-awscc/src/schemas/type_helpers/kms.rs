use std::sync::LazyLock;

use crate::partition::{arn_regex, regex_fragment as arn_partition_pattern};

/// Canonical KMS key ARN grammar over all supported partitions.
pub fn kms_key_arn_pattern() -> &'static str {
    static PATTERN: LazyLock<String> = LazyLock::new(|| {
        arn_regex(
            r"kms:[a-z]{2,4}(-[a-z0-9]+)+-[0-9]+:[0-9]{12}:key/([0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12}|mrk-[0-9a-fA-F]{32})$",
        )
    });
    &PATTERN
}

/// KMS reference grammar, retaining aliases and bare key IDs alongside ARNs.
pub fn kms_key_reference_pattern() -> &'static str {
    static PATTERN: LazyLock<String> = LazyLock::new(|| {
        [
            "^((arn:",
            &arn_partition_pattern(),
            r":kms:[a-z]{2,4}(-[a-z0-9]+)+-[0-9]+:[0-9]{12}:(key/([0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12}|mrk-[0-9a-fA-F]{32})|alias/[a-zA-Z0-9:/_-]+))|(alias/[a-zA-Z0-9:/_-]+)|([0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12})|(mrk-[0-9a-fA-F]{32}))$",
        ].concat()
    });
    &PATTERN
}

/// Validate the canonical, key-only output type used by both guests.
pub fn validate_kms_key_arn(value: &str) -> Result<(), String> {
    static PATTERN: std::sync::OnceLock<regex::Regex> = std::sync::OnceLock::new();
    if PATTERN
        .get_or_init(|| regex::Regex::new(kms_key_arn_pattern()).unwrap())
        .is_match(value)
    {
        Ok(())
    } else {
        Err("expected a KMS key ARN with a UUID or multi-region key ID".into())
    }
}

/// Offline syntax validation, using the reference leaf's portable constraints.
pub fn validate_kms_key_reference(value: &str) -> Result<(), String> {
    static PATTERN: std::sync::OnceLock<regex::Regex> = std::sync::OnceLock::new();
    if value.len() <= 2048
        && PATTERN
            .get_or_init(|| regex::Regex::new(kms_key_reference_pattern()).unwrap())
            .is_match(value)
    {
        Ok(())
    } else {
        Err("expected a KMS key ARN, alias ARN, alias name, UUID or multi-region key ID".into())
    }
}
