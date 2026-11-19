use carina_core::schema::AttributeType;
use carina_core::schema::{PatternMatch, PatternSpec};

/// Established whole-value IAM path check, including the root path '/'.
/// Smithy's pathType has a looser alternation (`^(/)|(/.../)$`); member
/// derivation preserves that search and applies this check as an input rule.
pub const IAM_PATH_PATTERN: &str = r"^(?:/|/[\x21-\x7E]+/)\z";

/// Shared standalone anonymous IAM path, with length 1..512.
pub fn iam_path() -> AttributeType {
    AttributeType::string_member(
        carina_core::schema::StringIdentityLayer::new(None, None, None, None),
        carina_core::schema::StringMemberConstraints::new(
            Some(PatternSpec::new(IAM_PATH_PATTERN, PatternMatch::Full)),
            Some((Some(1), Some(512))),
        ),
    )
}
