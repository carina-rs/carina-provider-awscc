//! Source-level backstops for emitter assertions and the upstream-test manifest.
#![cfg(not(target_arch = "wasm32"))]

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use syn::visit::Visit;

#[derive(Debug)]
struct Literal {
    path: String,
    function: String,
    line: usize,
    value: String,
}

#[derive(Default)]
struct Sources {
    path: String,
    function: String,
    literals: Vec<Literal>,
    tests: BTreeSet<(String, String)>,
}

impl Sources {
    fn tokens(&mut self, tokens: proc_macro2::TokenStream) {
        let mut attribute = false;
        for token in tokens {
            if let proc_macro2::TokenTree::Punct(punctuation) = &token {
                if punctuation.as_char() == '#' {
                    attribute = true;
                    continue;
                }
                if attribute && punctuation.as_char() == '!' {
                    continue;
                }
            }
            if attribute {
                attribute = false;
                if let proc_macro2::TokenTree::Group(group) = &token
                    && group.delimiter() == proc_macro2::Delimiter::Bracket
                {
                    continue;
                }
            }
            match token {
                proc_macro2::TokenTree::Group(group) => self.tokens(group.stream()),
                proc_macro2::TokenTree::Literal(literal) => {
                    if let Ok(value) =
                        syn::parse2::<syn::LitStr>(proc_macro2::TokenTree::Literal(literal).into())
                    {
                        self.visit_lit_str(&value);
                    }
                }
                _ => {}
            }
        }
    }
}

impl<'ast> Visit<'ast> for Sources {
    fn visit_attribute(&mut self, _: &'ast syn::Attribute) {
        // Doc comments are attributes, not asserted string literals.
    }

    fn visit_item_fn(&mut self, item: &'ast syn::ItemFn) {
        let previous = std::mem::replace(&mut self.function, item.sig.ident.to_string());
        if item.attrs.iter().any(|attr| {
            attr.path()
                .segments
                .last()
                .is_some_and(|segment| segment.ident == "test")
        }) {
            self.tests
                .insert((self.path.clone(), self.function.clone()));
        }
        syn::visit::visit_item_fn(self, item);
        self.function = previous;
    }

    fn visit_macro(&mut self, node: &'ast syn::Macro) {
        // Assertion/format macro bodies are token streams rather than AST expressions.
        self.tokens(node.tokens.clone());
    }

    fn visit_lit_str(&mut self, literal: &'ast syn::LitStr) {
        self.literals.push(Literal {
            path: self.path.clone(),
            function: self.function.clone(),
            line: literal.span().start().line,
            value: literal.value(),
        });
    }
}

fn rust_files(directory: &Path, files: &mut Vec<PathBuf>) {
    for entry in std::fs::read_dir(directory).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            rust_files(&path, files);
        } else if path.extension().is_some_and(|extension| extension == "rs") {
            files.push(path);
        }
    }
}

fn sources() -> Sources {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut files = Vec::new();
    rust_files(&root.join("src"), &mut files);
    rust_files(&root.join("tests"), &mut files);
    files.sort();
    let mut sources = Sources::default();
    for file in files {
        sources.path = file
            .strip_prefix(root)
            .unwrap()
            .to_string_lossy()
            .into_owned();
        sources.visit_file(&syn::parse_file(&std::fs::read_to_string(file).unwrap()).unwrap());
    }
    sources
}

fn retired(value: &str) -> bool {
    static SPELLINGS: std::sync::LazyLock<[String; 6]> = std::sync::LazyLock::new(|| {
        [
            ["enum_", "identity("].concat(),
            ["carina_core::schema::", "enum_identity"].concat(),
            ["carina_aws_", "types::"].concat(),
            ["Some(\"", "aws."].concat(),
            ["TypeIdentity::", "new"].concat(),
            ["awscc_enum_", "identity"].concat(),
        ]
    });
    SPELLINGS.iter().any(|spelling| value.contains(spelling))
        || old_namespace_spelling(value)
        || (value.contains("awscc_identity")
            && [
                ".expect(\"valid AWSCC identity\")",
                r#"\.expect\("valid AWSCC identity"\)"#,
            ]
            .iter()
            .any(|ending| {
                value.split(ending).skip(1).any(|tail| {
                    let tail = tail.trim_start();
                    tail.is_empty() || tail.starts_with([',', ')']) || tail.starts_with(r"\)")
                })
            }))
}

fn old_namespace_spelling(value: &str) -> bool {
    static OLD_NAMESPACE: std::sync::LazyLock<regex::Regex> = std::sync::LazyLock::new(|| {
        regex::Regex::new(
            r"\baws\.(?:[A-Z][A-Za-z0-9_]*|[a-z][A-Za-z0-9_]*\.[A-Za-z_][A-Za-z0-9_]*)(?:\.[A-Za-z_][A-Za-z0-9_]*)*",
        )
        .unwrap()
    });
    OLD_NAMESPACE.find_iter(value).any(|candidate| {
        // AWS documentation hostnames and URL text are not type spellings.
        if matches!(candidate.as_str(), "aws.amazon.com" | "aws.amazon.com.cn") {
            return false;
        }
        let token_start = value[..candidate.start()]
            .char_indices()
            .rev()
            .find(|(_, c)| c.is_whitespace() || ['"', '\'', '`', '<', '>'].contains(c))
            .map_or(0, |(index, c)| index + c.len_utf8());
        !value[token_start..candidate.start()].contains("://")
    })
}

type SpellingKey = (String, String, String);

fn check_spellings(
    literals: &[Literal],
    exemptions: &BTreeMap<SpellingKey, usize>,
) -> Result<(), String> {
    let mut observed = BTreeMap::<SpellingKey, Vec<usize>>::new();
    for literal in literals.iter().filter(|literal| retired(&literal.value)) {
        observed
            .entry((
                literal.path.clone(),
                literal.function.clone(),
                literal.value.clone(),
            ))
            .or_default()
            .push(literal.line);
    }
    let mut failures = Vec::new();
    for (key, lines) in &observed {
        let expected = exemptions.get(key).copied().unwrap_or(0);
        if lines.len() != expected {
            failures.push(format!(
                "{}\t{}\tlines {lines:?}\t{}: {} occurrences; expected {expected}",
                key.0,
                key.1,
                serde_json::to_string(&key.2).unwrap(),
                lines.len()
            ));
        }
    }
    for key in exemptions.keys() {
        if !observed.contains_key(key) {
            failures.push(format!("unused emitter spelling exemption: {key:?}"));
        }
    }
    if failures.is_empty() {
        Ok(())
    } else {
        Err(format!(
            "assert on the current emitter spelling (#572):\n{}",
            failures.join("\n")
        ))
    }
}

#[test]
fn emitter_assertions_use_current_spellings() {
    let mut exemptions = BTreeMap::new();
    for row in include_str!("fixtures/emitter_spelling_exceptions_572.tsv")
        .lines()
        .skip(1)
    {
        let columns = row.splitn(5, '\t').collect::<Vec<_>>();
        assert_eq!(columns.len(), 5, "{row}");
        assert!(!columns[4].is_empty(), "exemptions need a reason: {row}");
        let value: String = serde_json::from_str(columns[2]).unwrap();
        let occurrences: usize = columns[3].parse().unwrap();
        assert!(
            occurrences > 0,
            "exemptions need an exact positive count: {row}"
        );
        assert!(
            exemptions
                .insert(
                    (columns[0].to_string(), columns[1].to_string(), value),
                    occurrences,
                )
                .is_none(),
            "duplicate exemption: {row}"
        );
    }
    check_spellings(&sources().literals, &exemptions).unwrap();
}

#[test]
fn old_namespace_spelling_detects_nested_identities_without_matching_aws_text() {
    for tail in [
        "test.resource.Status.x",
        "test.resource.deeper.Status.x",
        "test.resource.status.x",
        "ec2.Vpc.Id",
        "Region.ap_northeast_1",
        "Arn",
    ] {
        let value = format!("{}.{}", "aws", tail);
        assert!(old_namespace_spelling(&value), "missed {value}");
    }
    for value in [
        "arn:aws:ec2:ap-northeast-1:123456789012:vpc/vpc-12345678",
        "aws:PrincipalArn",
        "amazonaws.com",
        "aws.amazon.com",
        "aws.amazon.com.cn",
        "https://docs.aws.amazon.com/AWSCloudFormation/latest/",
        "https://aws.example.com/path",
        "https://aws.test.resource.Status.x/path",
        "awscc.ec2.Vpc.Id",
    ] {
        assert!(!old_namespace_spelling(value), "false positive {value}");
    }
    for separator in [" ", "\u{a0}", "\u{2003}"] {
        let diagnostic = format!(
            "https://aws.amazon.com/{separator}{}.{}",
            "aws", "test.resource.Status.x"
        );
        assert!(old_namespace_spelling(&diagnostic));
    }
}

#[test]
fn emitter_spelling_exemptions_pin_each_occurrence_including_top_level() {
    for function in ["probe", ""] {
        let value = format!("{}.{}", "aws", "test.resource.Status.x");
        let literal = |line| Literal {
            path: "src/probe.rs".into(),
            function: function.into(),
            line,
            value: value.clone(),
        };
        let exemptions =
            BTreeMap::from([(("src/probe.rs".into(), function.into(), value.clone()), 1)]);
        check_spellings(&[literal(1)], &exemptions).unwrap();
        let failure = check_spellings(&[literal(1), literal(2)], &exemptions).unwrap_err();
        assert!(failure.contains("2 occurrences; expected 1"), "{failure}");
        assert!(failure.contains("lines [1, 2]"), "{failure}");
        assert!(
            check_spellings(&[], &exemptions)
                .unwrap_err()
                .contains("unused")
        );
    }
}

#[test]
fn ported_test_manifest_names_existing_tests() {
    let tests = sources().tests;
    let mut checked = 0;
    for row in include_str!("fixtures/ported_schema_tests_5ad417e.tsv")
        .lines()
        .skip(1)
    {
        let columns = row.splitn(5, '\t').collect::<Vec<_>>();
        assert_eq!(columns.len(), 5, "{row}");
        if matches!(columns[2], "ported" | "already ported") {
            checked += 1;
            assert!(
                tests.contains(&(columns[3].to_string(), columns[1].to_string())),
                "manifest test does not exist in its recorded local source file: {row}"
            );
        }
    }
    assert!(checked > 0);
}

#[test]
fn ported_test_manifest_distinguishes_same_name_in_different_files() {
    let mut sources = Sources {
        path: "src/actual.rs".into(),
        ..Sources::default()
    };
    sources.visit_file(&syn::parse_file("#[test] fn same_name() {}").unwrap());
    assert!(
        sources
            .tests
            .contains(&("src/actual.rs".into(), "same_name".into()))
    );
    assert!(
        !sources
            .tests
            .contains(&("src/other.rs".into(), "same_name".into()))
    );
}
