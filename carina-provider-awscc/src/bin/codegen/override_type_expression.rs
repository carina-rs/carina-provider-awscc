//! Override expressions with typed pattern emission, including union members.

use super::{ApprovedPattern, catalog_type_display, emit_pattern_option};
use std::borrow::Cow;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum PatternLiteral {
    Quoted,
    Raw,
}

/// Raw expressions cannot contain core pattern constructors. String
/// refinements and unions are built here from approval evidence instead.
/// Preserve existing raw/quoted literal spelling to avoid unrelated output
/// changes when routing already-whole override patterns through approval.
#[derive(Debug, Clone, PartialEq)]
pub(super) struct OverrideTypeExpression {
    code: Cow<'static, str>,
    patterns: Vec<ApprovedPattern>,
}

impl OverrideTypeExpression {
    pub(super) fn raw(code: &'static str) -> Self {
        assert!(
            !code.contains("refined_string") && !code.contains("::reference("),
            "pattern-bearing overrides must use a typed string refinement: {code}",
        );
        Self {
            code: Cow::Borrowed(code),
            patterns: Vec::new(),
        }
    }

    pub(super) fn string(
        pattern: Option<ApprovedPattern>,
        length: Option<(Option<u64>, Option<u64>)>,
        literal: PatternLiteral,
    ) -> Self {
        let emitted = match (literal, pattern.as_ref()) {
            (PatternLiteral::Raw, Some(pattern)) => {
                let source = pattern.as_str();
                // Choose enough raw-string delimiters even if a future
                // approved source contains a quote or a quote/hash pair.
                let mut hashes = String::new();
                while source.contains(&format!("\"{hashes}")) {
                    hashes.push('#');
                }
                format!("Some(r{hashes}\"{source}\"{hashes}.to_string())")
            }
            _ => emit_pattern_option(pattern.as_ref()),
        };
        Self {
            code: Cow::Owned(format!(
                "AttributeType::refined_string(None, {emitted}, {length:?}, None)"
            )),
            patterns: pattern.into_iter().collect(),
        }
    }

    pub(super) fn union(members: Vec<Self>) -> Self {
        let code = format!(
            "AttributeType::union(vec![{}])",
            members
                .iter()
                .map(Self::as_str)
                .collect::<Vec<_>>()
                .join(", "),
        );
        Self {
            code: Cow::Owned(code),
            patterns: members
                .into_iter()
                .flat_map(|member| member.patterns)
                .collect(),
        }
    }

    pub(super) fn as_str(&self) -> &str {
        &self.code
    }

    pub(super) fn patterns(&self) -> &[ApprovedPattern] {
        &self.patterns
    }

    /// Explicit regex displays consume the same normalized pattern as Rust.
    /// Compact labels such as String(pattern, len: ...) contain no regex.
    pub(super) fn normalized_display(&self, display: &'static str) -> Cow<'static, str> {
        // A catalog constructor's identity wins over even a stale explicit
        // label, so every override variant follows the same metadata rule.
        if let Some(identity) = catalog_type_display(self.as_str()) {
            return Cow::Borrowed(identity);
        }
        // Other runtime constructors keep their existing displays. Their
        // patterns are approved inside awscc_types.rs instead.
        if self.patterns.is_empty() {
            return Cow::Borrowed(display);
        }
        let Some(start) = display.find("pattern: ") else {
            return Cow::Borrowed(display);
        };
        let [pattern] = self.patterns.as_slice() else {
            panic!(
                "an explicit pattern display must describe exactly one approved pattern: {display}"
            );
        };
        let start = start + "pattern: ".len();
        let end = display.rfind(')').expect("string refinement display");
        Cow::Owned(format!(
            "{}{}{}",
            &display[..start],
            pattern.as_str(),
            &display[end..]
        ))
    }
}

impl std::fmt::Display for OverrideTypeExpression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}
