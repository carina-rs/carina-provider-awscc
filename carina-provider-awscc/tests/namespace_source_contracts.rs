//! Source-level backstops for emitter assertions and the upstream-test manifest.
#![cfg(not(target_arch = "wasm32"))]

use std::collections::BTreeSet;
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
    tests: BTreeSet<String>,
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
            self.tests.insert(self.function.clone());
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
    static OLD_NAMESPACE: std::sync::LazyLock<regex::Regex> = std::sync::LazyLock::new(|| {
        regex::Regex::new(r"\baws\.(?:[a-z][A-Za-z0-9_]*\.)?[A-Z][A-Za-z0-9_]*").unwrap()
    });
    SPELLINGS.iter().any(|spelling| value.contains(spelling))
        || OLD_NAMESPACE.is_match(value)
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

#[test]
fn emitter_assertions_use_current_spellings() {
    let sources = sources();
    if let Some(path) = std::env::var_os("AWSCC_EMITTER_AUDIT_OUTPUT") {
        let mut audit = "source\tfunction\tline\tretired\tliteral_json\n".to_string();
        for literal in &sources.literals {
            audit.push_str(&format!(
                "{}\t{}\t{}\t{}\t{}\n",
                literal.path,
                literal.function,
                literal.line,
                retired(&literal.value),
                serde_json::to_string(&literal.value).unwrap()
            ));
        }
        std::fs::write(path, audit).unwrap();
    }
    let mut exemptions = BTreeSet::new();
    for row in include_str!("fixtures/emitter_spelling_exceptions_572.tsv")
        .lines()
        .skip(1)
    {
        let columns = row.splitn(4, '\t').collect::<Vec<_>>();
        assert_eq!(columns.len(), 4, "{row}");
        assert!(!columns[3].is_empty(), "exemptions need a reason: {row}");
        let value: String = serde_json::from_str(columns[2]).unwrap();
        exemptions.insert((columns[0].to_string(), columns[1].to_string(), value));
    }
    let mut failures = Vec::new();
    let mut used = BTreeSet::new();
    for literal in sources
        .literals
        .iter()
        .filter(|literal| retired(&literal.value))
    {
        let key = (
            literal.path.clone(),
            literal.function.clone(),
            literal.value.clone(),
        );
        if exemptions.contains(&key) {
            used.insert(key);
        } else {
            failures.push(format!(
                "{}\t{}\t{}\t{}",
                literal.path,
                literal.function,
                literal.line,
                serde_json::to_string(&literal.value).unwrap()
            ));
        }
    }
    assert!(
        failures.is_empty(),
        "assert on the current emitter spelling (#572):\n{}",
        failures.join("\n")
    );
    assert_eq!(exemptions, used, "unused emitter spelling exemptions");
}

#[test]
fn ported_test_manifest_names_existing_tests() {
    let tests = sources().tests;
    let mut checked = 0;
    for row in include_str!("fixtures/ported_schema_tests_5ad417e.tsv")
        .lines()
        .skip(1)
    {
        let columns = row.splitn(4, '\t').collect::<Vec<_>>();
        assert_eq!(columns.len(), 4, "{row}");
        if matches!(columns[2], "ported" | "already ported") {
            checked += 1;
            assert!(
                tests.contains(columns[1]),
                "manifest test does not exist: {row}"
            );
        }
    }
    assert!(checked > 0);
}
