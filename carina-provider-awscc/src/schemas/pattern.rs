//! Pattern approval shared by the generator and AWSCC runtime constructors.

use carina_core::schema::{AttributeType, CustomValidator, DslTransform, TypeIdentity};
use regex_syntax::ast::{AssertionKind, Ast, Flag};

/// An emitted whole-value pattern, or one exact reviewed substring exception.
/// The private field prevents producers from approving arbitrary regex text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApprovedPattern(String);

impl ApprovedPattern {
    /// Preserve sources satisfying [`is_whole_anchored`]; otherwise surround
    /// the complete source with `^(?:` and `)\z`. No source is discarded or
    /// repaired here, including invalid or unsupported syntax. Codegen must
    /// compile the result in Carina's schema dialect and report its provenance.
    pub fn whole(source: &str) -> Self {
        Self(if is_whole_anchored(source) {
            source.to_string()
        } else {
            format!(r"^(?:{source})\z")
        })
    }

    /// Approve only an exact resource, JSON source pointer and source regex
    /// registered below. A changed source cannot inherit an old exception.
    pub fn substring_exception(
        resource_type: &str,
        source_path: &str,
        source: &str,
    ) -> Option<Self> {
        SUBSTRING_PATTERN_EXCEPTIONS
            .iter()
            .find(|entry| {
                entry.resource_type == resource_type
                    && entry.source_path == source_path
                    && entry.source == source
            })
            .map(|entry| Self(entry.emitted.to_string()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Whether the source begins with `^`, ends with `$` or `\z`, and every
/// alternative requires both the beginning and end of the entire value.
/// Thus `^A$|^B$` qualifies and `^A|B$` does not. Escaped anchors, classes,
/// nested alternatives and inline multiline flags are interpreted by the
/// regex parser, rather than guessed from the first and last characters.
/// Rust's `$` is end of text without multiline mode. Line-only assertions
/// do not qualify. Unparseable sources return false and are still wrapped
/// verbatim by `whole`, so the schema-dialect compiler can diagnose them.
pub fn is_whole_anchored(source: &str) -> bool {
    if !source.starts_with('^') || !(source.ends_with('$') || source.ends_with(r"\z")) {
        return false;
    }
    // Inspect syntax before regex optimization: an empty character-class
    // intersection must not erase otherwise valid outer anchors. Track flags
    // in lexical order (also across alternatives), restoring them at groups.
    fn boundaries(ast: &Ast, multiline: &mut bool) -> (bool, bool) {
        match ast {
            Ast::Assertion(assertion) => match assertion.kind {
                AssertionKind::StartLine => (!*multiline, false),
                AssertionKind::EndLine => (false, !*multiline),
                AssertionKind::StartText => (true, false),
                AssertionKind::EndText => (false, true),
                _ => (false, false),
            },
            Ast::Concat(concat) => {
                let mut result = (false, false);
                for (index, part) in concat.asts.iter().enumerate() {
                    let (start, end) = boundaries(part, multiline);
                    if index == 0 {
                        result.0 = start;
                    }
                    result.1 = end;
                }
                result
            }
            Ast::Alternation(alternation) => {
                let mut result = (true, true);
                for branch in &alternation.asts {
                    let (start, end) = boundaries(branch, multiline);
                    result.0 &= start;
                    result.1 &= end;
                }
                result
            }
            Ast::Group(group) => {
                let mut scoped = group
                    .flags()
                    .and_then(|flags| flags.flag_state(Flag::MultiLine))
                    .unwrap_or(*multiline);
                boundaries(&group.ast, &mut scoped)
            }
            Ast::Flags(flags) => {
                if let Some(enabled) = flags.flags.flag_state(Flag::MultiLine) {
                    *multiline = enabled;
                }
                (false, false)
            }
            Ast::Repetition(repetition) => {
                boundaries(&repetition.ast, multiline);
                (false, false)
            }
            _ => (false, false),
        }
    }
    regex_syntax::ast::parse::Parser::new()
        .parse(source)
        .is_ok_and(|ast| boundaries(&ast, &mut false) == (true, true))
}

pub struct SubstringPatternException {
    pub resource_type: &'static str,
    pub source_path: &'static str,
    pub source: &'static str,
    pub emitted: &'static str,
    pub reason: &'static str,
}

pub const SUBSTRING_PATTERN_EXCEPTIONS: &[SubstringPatternException] = &[
    SubstringPatternException {
        resource_type: "AWS::Scheduler::Schedule",
        source_path: "/definitions/EventBridgeParameters/properties/Source",
        source: r"^(?=[/\.\-_A-Za-z0-9]+)((?!aws\.).*)|(\$(\.[\w_-]+(\[(\d+|\*)\])*)*)$",
        // Preserve ^A|B$ precedence and search semantics. A only requires an
        // allowed first character and excludes the fixed prefix `aws.`; its .*
        // may match zero characters. Enumerate the first mismatch against that
        // prefix, including a/aw/aws. B remains an unanchored JSON-path suffix,
        // including a bare `$`. Spell out ECMAScript's ASCII \w and \d.
        emitted: r"^(?:[/.\-_A-Zb-z0-9]|a(?:\z|[^w])|aw(?:\z|[^s])|aws(?:\z|[^.]))|(\$(\.[0-9A-Za-z_-]+(\[([0-9]+|\*)\])*)*)$",
        // CFN and the API publish differently anchored versions. This reason is
        // compatibility with the existing reviewed language, not an AWS probe.
        // https://docs.aws.amazon.com/AWSCloudFormation/latest/TemplateReference/aws-properties-scheduler-schedule-eventbridgeparameters.html
        // https://docs.aws.amazon.com/scheduler/latest/APIReference/API_EventBridgeParameters.html
        reason: "Preserve the existing reviewed CFN search language for Scheduler EventBridgeParameters.Source; CFN and API publish differently anchored versions.",
    },
];

/// AWSCC runtime producers accept approval evidence before calling core.
pub(super) fn refined_string(
    identity: Option<TypeIdentity>,
    pattern: Option<ApprovedPattern>,
    length: Option<(Option<u64>, Option<u64>)>,
    to_dsl: Option<DslTransform>,
) -> AttributeType {
    AttributeType::refined_string(identity, pattern.map(|pattern| pattern.0), length, to_dsl)
}

pub(super) fn refined_string_with_validator(
    identity: TypeIdentity,
    pattern: Option<ApprovedPattern>,
    length: Option<(Option<u64>, Option<u64>)>,
    validate: CustomValidator,
    to_dsl: Option<DslTransform>,
) -> AttributeType {
    AttributeType::refined_string_with_validator(
        identity,
        pattern.map(|pattern| pattern.0),
        length,
        validate,
        to_dsl,
    )
}
