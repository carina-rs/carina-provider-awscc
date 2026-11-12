// The oracle is the compiled runtime schema, independently of CFN type inference.
use super::doc_test_support::{emission, page};
use super::*;
use carina_core::schema::{AttributeSchema, ResourceSchema, StructField};
use carina_core::schema::{
    Compiled, CompiledAttributeType, CompiledResourceSchema, RawShape, TypeIdentity,
};
use carina_docgen::{
    Declaration, DocLink, Resolved, TypeDocHooks, TypeEdge, TypeRegistry, render_attribute_type,
};
use comrak::{
    Arena, Options,
    nodes::{AstNode, NodeValue},
    parse_document,
};
use serde_json::json;
use std::collections::VecDeque;

#[derive(Clone, Debug, PartialEq, Eq)]
enum Inline {
    Text(String),
    Code(String),
    Link { label: String, url: String },
}

fn plain<'a>(node: &'a AstNode<'a>) -> String {
    match &node.data.borrow().value {
        NodeValue::Text(text) => text.to_string(),
        NodeValue::Code(code) => code.literal.clone(),
        NodeValue::SoftBreak | NodeValue::LineBreak => " ".into(),
        _ => node.children().map(plain).collect(),
    }
}

fn inlines<'a>(node: &'a AstNode<'a>) -> Vec<Inline> {
    fn visit<'a>(node: &'a AstNode<'a>, output: &mut Vec<Inline>) {
        match &node.data.borrow().value {
            NodeValue::Text(text) => output.push(Inline::Text(text.to_string())),
            NodeValue::Code(code) => output.push(Inline::Code(code.literal.clone())),
            NodeValue::Link(link) => output.push(Inline::Link {
                label: plain(node),
                url: link.url.clone(),
            }),
            NodeValue::SoftBreak | NodeValue::LineBreak => output.push(Inline::Text(" ".into())),
            _ => {
                for child in node.children() {
                    visit(child, output);
                }
            }
        }
    }
    let mut raw = Vec::new();
    visit(node, &mut raw);
    let mut output = Vec::new();
    for atom in raw {
        if let Inline::Text(text) = &atom
            && let Some(Inline::Text(previous)) = output.last_mut()
        {
            previous.push_str(text);
            continue;
        }
        output.push(atom);
    }
    output
}

fn options() -> Options<'static> {
    let mut options = Options::default();
    options.extension.table = true;
    options.extension.strikethrough = true;
    options.extension.autolink = true;
    options
}

#[derive(Default, Debug)]
struct MarkdownTypes {
    attributes: BTreeMap<String, Vec<Inline>>,
    declarations: BTreeMap<String, BTreeMap<String, Vec<Inline>>>,
    enums: BTreeMap<String, Vec<(String, String)>>,
    metadata: BTreeMap<String, BTreeMap<String, Vec<Inline>>>,
    headings: BTreeMap<String, (String, String)>,
    uses: BTreeMap<String, BTreeSet<String>>,
}

pub(super) fn assert_inline_cells(markdown: &str, name: &str, expected: &str) -> Result<()> {
    let docs = parse_types(markdown)?;
    let cells = docs
        .attributes
        .get(name)
        .into_iter()
        .chain(
            docs.declarations
                .values()
                .filter_map(|fields| fields.get(name)),
        )
        .collect::<Vec<_>>();
    anyhow::ensure!(!cells.is_empty(), "missing Type cell {name}");
    for cell in cells {
        anyhow::ensure!(
            *cell == [Inline::Code(expected.into())],
            "{name}: expected inline {expected:?}, got {cell:?}"
        );
    }
    anyhow::ensure!(
        !docs
            .headings
            .values()
            .any(|(section, heading)| section == "Union Definitions"
                && heading.split('.').next_back() == Some(name)),
        "unused union table for {name}"
    );
    Ok(())
}

fn parse_types(markdown: &str) -> Result<MarkdownTypes> {
    let arena = Arena::new();
    let root = parse_document(&arena, markdown, &options());
    let mut heading_ids = carina_docgen::heading_ids(markdown).into_iter();
    let mut section = String::new();
    let mut heading = String::new();
    let mut target = String::new();
    let mut output = MarkdownTypes::default();
    for node in root.children() {
        match &node.data.borrow().value {
            NodeValue::Heading(value) => {
                let id = heading_ids.next().context("missing heading ID")?;
                if value.level == 2 {
                    section = plain(node);
                }
                heading = plain(node);
                target = format!("#{id}");
                if value.level == 3
                    && matches!(
                        section.as_str(),
                        "Struct Definitions" | "Union Definitions" | "Enum Values"
                    )
                {
                    output
                        .headings
                        .insert(target.clone(), (section.clone(), heading.clone()));
                }
            }
            NodeValue::List(_)
                if matches!(
                    section.as_str(),
                    "Argument Reference" | "Attribute Reference"
                ) =>
            {
                for item in node.children() {
                    let Some(paragraph) = item.first_child() else {
                        continue;
                    };
                    let mut atoms = inlines(paragraph);
                    if let Some(Inline::Text(first)) = atoms.first_mut()
                        && let Some((label, rest)) = first.split_once(": ")
                    {
                        let label = label.to_owned();
                        *first = rest.to_owned();
                        if first.is_empty() {
                            atoms.remove(0);
                        }
                        if label == "Type" {
                            anyhow::ensure!(
                                output.attributes.insert(heading.clone(), atoms).is_none(),
                                "duplicate Type entry for {heading}"
                            );
                        } else if matches!(
                            label.as_str(),
                            "Required" | "Create-only" | "Write-only" | "Identity" | "Default"
                        ) {
                            anyhow::ensure!(
                                output
                                    .metadata
                                    .entry(heading.clone())
                                    .or_default()
                                    .insert(label, atoms)
                                    .is_none(),
                                "duplicate attribute fact for {heading}"
                            );
                        }
                    }
                }
            }
            NodeValue::Paragraph if plain(node).starts_with("Used by: ") => {
                let paths = inlines(node)
                    .into_iter()
                    .filter_map(|atom| {
                        if let Inline::Code(path) = atom {
                            Some(path)
                        } else {
                            None
                        }
                    })
                    .collect();
                anyhow::ensure!(
                    output.uses.insert(target.clone(), paths).is_none(),
                    "duplicate Used by paragraph under {heading}"
                );
            }
            NodeValue::Table(_) => {
                let mut rows = node.children();
                let header = rows.next().context("table has no header")?;
                let headers = header.children().map(plain).collect::<Vec<_>>();
                if section == "Enum Values" && headers == ["Value", "DSL Identifier"] {
                    let mut values = Vec::new();
                    for row in rows {
                        let cells = row.children().collect::<Vec<_>>();
                        anyhow::ensure!(cells.len() == 2, "malformed enum table under {heading}");
                        values.push((plain(cells[0]), plain(cells[1])));
                    }
                    anyhow::ensure!(
                        output.enums.insert(target.clone(), values).is_none(),
                        "duplicate enum table under {heading}"
                    );
                    continue;
                }
                let Some(column) = headers.iter().position(|cell| cell == "Type") else {
                    continue;
                };
                let fields = output.declarations.entry(target.clone()).or_default();
                for row in rows {
                    let cells = row.children().collect::<Vec<_>>();
                    anyhow::ensure!(
                        cells.len() == headers.len(),
                        "malformed table under {heading}"
                    );
                    let field = plain(cells[0]);
                    anyhow::ensure!(
                        fields
                            .insert(field.clone(), inlines(cells[column]))
                            .is_none(),
                        "duplicate Type cell {heading}.{field}"
                    );
                }
            }
            _ => {}
        }
    }
    Ok(output)
}

struct Oracle<'a> {
    schema: &'a CompiledResourceSchema,
    declarations: BTreeMap<usize, (&'a CompiledAttributeType, Vec<String>)>,
    canonical_scopes: HashMap<usize, String>,
}

impl<'a> TypeDocHooks<'a, Compiled> for Oracle<'a> {
    type NodeId = usize;
    type Target = usize;
    type Scope = Vec<String>;

    fn resolve(&self, mut ty: &'a CompiledAttributeType) -> Result<Resolved<'a, Compiled, usize>> {
        let mut seen = BTreeSet::new();
        while let RawShape::Ref(name) = ty.raw_shape() {
            anyhow::ensure!(seen.insert(name), "alias cycle: {name}");
            ty = self
                .schema
                .defs()
                .get(name)
                .with_context(|| format!("missing runtime def {name}"))?;
        }
        Ok(Resolved {
            id: std::ptr::from_ref(ty) as usize,
            ty,
        })
    }

    fn scalar_name(&self, identity: &TypeIdentity) -> String {
        identity.to_string()
    }

    fn child_scope(&self, parent: &Vec<String>, edge: TypeEdge<'_>) -> Vec<String> {
        let mut child = parent.clone();
        child.push(match edge {
            TypeEdge::Field(name) => name.into(),
            TypeEdge::Item => return child,
            TypeEdge::Key => "<key>".into(),
            TypeEdge::Value => "<value>".into(),
            TypeEdge::Member(index) => format!("member_{index}"),
        });
        child
    }

    fn register(
        &mut self,
        scope: &Vec<String>,
        declaration: Declaration<'a, Compiled, usize>,
    ) -> Result<DocLink<usize>> {
        let label = match declaration.ty.raw_shape() {
            RawShape::Enum { identity, .. } => identity.kind.clone(),
            RawShape::Struct { name, .. } => name.to_owned(),
            RawShape::Union(_) => self
                .canonical_scopes
                .get(&declaration.id)
                .cloned()
                .unwrap_or_else(|| scope.join(".")),
            shape => anyhow::bail!("unexpected declaration: {shape:?}"),
        };
        self.declarations
            .insert(declaration.id, (declaration.ty, scope.clone()));
        Ok(DocLink {
            target: declaration.id,
            label,
        })
    }
}

fn expected_inline<'a>(
    ty: &'a CompiledAttributeType,
    scope: &[String],
    oracle: &mut Oracle<'a>,
    registry: &mut TypeRegistry<usize, usize>,
    note: Option<&str>,
) -> Result<Vec<Inline>> {
    let display = render_attribute_type(
        ty,
        &scope.to_vec(),
        oracle,
        registry,
        note.map(carina_docgen::ValidationNote::new),
    )?;
    let mut page = PageBuilder::new();
    display.write_to(&mut page);
    page.push_str("\n\n");
    for id in oracle.declarations.keys() {
        page.target_heading(*id, Heading::plain(2, format!("type{id}")))?;
    }
    let markdown = page.finish()?;
    let arena = Arena::new();
    let root = parse_document(&arena, &markdown, &options());
    Ok(inlines(root.first_child().context("empty expected type")?))
}

fn compare_inline(expected: &[Inline], actual: &[Inline]) -> Result<Vec<(usize, String)>> {
    let links = expected
        .iter()
        .filter_map(|atom| match atom {
            Inline::Link { url, .. } => Some(url.trim_start_matches("#type").parse::<usize>()),
            _ => None,
        })
        .collect::<std::result::Result<Vec<_>, _>>()?;
    let actual_links = actual
        .iter()
        .filter_map(|atom| match atom {
            Inline::Link { url, .. } => Some(url.clone()),
            _ => None,
        })
        .collect::<Vec<_>>();
    anyhow::ensure!(
        expected.len() == actual.len(),
        "expected {expected:?}, got {actual:?}"
    );
    for (expected, actual) in expected.iter().zip(actual) {
        match (expected, actual) {
            (
                Inline::Link {
                    label: expected, ..
                },
                Inline::Link { label: actual, url },
            ) => {
                anyhow::ensure!(
                    expected == actual && url.starts_with('#'),
                    "expected link {expected:?}, got {actual:?} ({url})"
                );
            }
            _ => anyhow::ensure!(expected == actual, "expected {expected:?}, got {actual:?}"),
        }
    }
    Ok(links.into_iter().zip(actual_links).collect())
}

fn verify_attribute_metadata(
    attribute: &carina_core::schema::CompiledAttributeSchema,
    actual: Option<&BTreeMap<String, Vec<Inline>>>,
) -> Result<()> {
    let parts = attribute.parts();
    let mut expected = BTreeMap::new();
    let mut text = |name: &str, value: &str| {
        expected.insert(name.to_owned(), vec![Inline::Text(value.into())]);
    };
    if !parts.declared_input_mode.is_provider_populated() {
        text(
            "Required",
            if parts.declared_input_mode.is_required() {
                "Yes"
            } else {
                "No"
            },
        );
    }
    if parts.create_only {
        text("Create-only", "Yes");
    }
    if parts.write_only {
        text("Write-only", "Yes");
    }
    if parts.identity_mode.is_identity() {
        text(
            "Identity",
            "Yes (anonymous identifiers hash only declared identity values)",
        );
    }
    if let Some(default) =
        carina_core::protocol_convert::attribute_schema_to_protocol(attribute)?.default
    {
        expected.insert(
            "Default".into(),
            vec![Inline::Code(serde_json::to_string(&default)?)],
        );
    }
    anyhow::ensure!(
        actual.cloned().unwrap_or_default() == expected,
        "runtime attribute facts differ for {}: expected {expected:?}, got {actual:?}",
        parts.name
    );
    Ok(())
}

// Constraints come from the runtime schema. Supplemental facts are retained
// emission annotations, independently collected here and rendered by docgen in
// the expected cell; no arbitrary trailing text is discarded.
struct NoteIndex<'a> {
    schema: &'a CompiledResourceSchema,
    emitted: &'a ResourceEmission,
    notes: HashMap<usize, String>,
    visited: BTreeSet<(usize, usize)>,
}

impl<'a> NoteIndex<'a> {
    fn build(
        schema: &'a CompiledResourceSchema,
        emitted: &'a ResourceEmission,
    ) -> Result<HashMap<usize, String>> {
        let mut index = Self {
            schema,
            emitted,
            notes: HashMap::new(),
            visited: BTreeSet::new(),
        };
        for (name, attr) in schema.attributes() {
            if let Some(value) = emitted.attributes.get(name) {
                index.visit(attr.attr_type(), &value.ty, Some(&value.annotations))?;
            }
        }
        Ok(index.notes)
    }

    fn visit(
        &mut self,
        runtime: &'a CompiledAttributeType,
        ty: &'a AttributeType,
        info: Option<&'a TypeAnnotations>,
    ) -> Result<()> {
        let pointer = std::ptr::from_ref(runtime) as usize;
        if !self
            .visited
            .insert((pointer, std::ptr::from_ref(ty) as usize))
        {
            return Ok(());
        }
        let note = self.collect(ty, info, "", &mut BTreeSet::new())?.join("; ");
        if let Some(previous) = self.notes.insert(pointer, note.clone()) {
            anyhow::ensure!(
                previous == note,
                "conflicting supplemental facts for one runtime node"
            );
        }
        let child = |name: &str| info.and_then(|info| info.children.get(name));
        match (runtime.raw_shape(), ty.raw_shape()) {
            (RawShape::Ref(r), RawShape::Ref(e)) => {
                let value = &self.emitted.defs[e];
                self.visit(&self.schema.defs()[r], &value.ty, Some(&value.annotations))?;
            }
            (RawShape::Ref(r), _) => self.visit(&self.schema.defs()[r], ty, info)?,
            (_, RawShape::Ref(e)) => {
                let value = &self.emitted.defs[e];
                self.visit(runtime, &value.ty, Some(&value.annotations))?;
            }
            (RawShape::Struct { fields: r, .. }, RawShape::Struct { fields: e, .. }) => {
                for r in r {
                    if let Some(e) = e.iter().find(|e| e.name() == r.name()) {
                        self.visit(r.field_type(), e.field_type(), child(e.name()))?;
                    }
                }
            }
            (RawShape::Union(r), RawShape::Union(e)) => {
                for (i, (r, e)) in r.iter().zip(e).enumerate() {
                    self.visit(r, e, child(&format!("member_{i}")))?;
                }
            }
            (
                RawShape::List {
                    element_type: r, ..
                },
                RawShape::List {
                    element_type: e, ..
                },
            ) => self.visit(r, e, child("item"))?,
            (
                RawShape::Map {
                    key: rk, value: rv, ..
                },
                RawShape::Map {
                    key: ek, value: ev, ..
                },
            ) => {
                self.visit(rk, ek, child("key"))?;
                self.visit(rv, ev, child("value"))?;
            }
            _ => {}
        }
        Ok(())
    }

    fn collect(
        &self,
        ty: &'a AttributeType,
        info: Option<&'a TypeAnnotations>,
        path: &str,
        active: &mut BTreeSet<usize>,
    ) -> Result<Vec<String>> {
        let pointer = std::ptr::from_ref(ty) as usize;
        if !active.insert(pointer) {
            return Ok(Vec::new());
        }
        let mut notes = info
            .into_iter()
            .flat_map(|info| &info.notes)
            .map(|note| {
                if path.is_empty() {
                    note.clone()
                } else {
                    format!("{path}: {note}")
                }
            })
            .collect::<Vec<_>>();
        let child = |name: &str| info.and_then(|info| info.children.get(name));
        let path_for = |name: &str| {
            if path.is_empty() {
                name.into()
            } else {
                format!("{path}.{name}")
            }
        };
        match ty.raw_shape() {
            RawShape::Ref(name) => {
                let value = &self.emitted.defs[name];
                notes.extend(self.collect(&value.ty, Some(&value.annotations), path, active)?);
            }
            RawShape::List { element_type, .. } => notes.extend(self.collect(
                element_type,
                child("item"),
                &path_for("item"),
                active,
            )?),
            RawShape::Map { key, value, .. } => {
                notes.extend(self.collect(key, child("key"), &path_for("key"), active)?);
                notes.extend(self.collect(value, child("value"), &path_for("value"), active)?);
            }
            _ => {}
        }
        active.remove(&pointer);
        Ok(notes)
    }
}

fn link_contexts<'a>(
    ty: &'a CompiledAttributeType,
    scope: &Vec<String>,
    oracle: &Oracle<'a>,
) -> Result<Vec<(usize, Vec<String>)>> {
    Ok(type_docs::declaration_links(ty, scope, oracle)?
        .into_iter()
        .map(|(id, _, scope)| (id, scope))
        .collect())
}

fn verify_target(
    docs: &MarkdownTypes,
    target: &str,
    ty: &CompiledAttributeType,
    scope: &[String],
) -> Result<Vec<String>> {
    let (section, heading) = docs
        .headings
        .get(target)
        .with_context(|| format!("missing declaration heading {target}"))?;
    let (expected_section, path) = match ty.raw_shape() {
        RawShape::Struct { .. } => ("Struct Definitions", heading.as_str()),
        RawShape::Union(_) => ("Union Definitions", heading.as_str()),
        RawShape::Enum { identity, .. } => (
            "Enum Values",
            heading
                .strip_suffix(&format!(" ({})", identity.kind))
                .context("wrong enum heading identity")?,
        ),
        shape => anyhow::bail!("unexpected declaration target shape {shape:?}"),
    };
    anyhow::ensure!(
        section == expected_section,
        "wrong declaration kind at {target}"
    );
    anyhow::ensure!(
        docs.declarations.contains_key(target) || docs.enums.contains_key(target),
        "declaration has no table: {target}"
    );
    let usage = scope.join(".");
    anyhow::ensure!(
        path == usage
            || docs
                .uses
                .get(target)
                .is_some_and(|uses| uses.contains(&usage)),
        "{usage} links to unrelated declaration {heading} ({target})"
    );
    Ok(vec![path.to_owned()])
}

pub(super) fn check_parity(
    schema: &CompiledResourceSchema,
    parsed: &ParsedCfnSchema,
    markdown: &str,
) -> Result<(usize, usize, usize, usize)> {
    let emitted = if parsed.properties.is_empty() {
        None
    } else {
        Some(emit_resource(
            &EmissionContext::from_source(parsed)?,
            &parsed.type_name,
            false,
        )?)
    };
    check_parity_with_emission(schema, parsed, markdown, emitted.as_ref())
}

pub(super) fn check_parity_with_emission(
    schema: &CompiledResourceSchema,
    parsed: &ParsedCfnSchema,
    markdown: &str,
    emitted: Option<&ResourceEmission>,
) -> Result<(usize, usize, usize, usize)> {
    let docs = parse_types(markdown)?;
    let mut oracle = Oracle {
        schema,
        declarations: BTreeMap::new(),
        canonical_scopes: HashMap::new(),
    };
    let mut registry = TypeRegistry::default();
    let notes = emitted
        .map(|emitted| NoteIndex::build(schema, emitted))
        .transpose()?
        .unwrap_or_default();
    let mut queue = VecDeque::new();
    let mut expected_attributes = BTreeSet::new();
    let mut errors = Vec::new();
    for (name, attr) in schema.attributes() {
        // Top-level deprecated properties are deliberately hidden in docs but
        // retained at runtime. Nested deprecated and omitted properties are
        // absent from the compiled runtime graph itself.
        if attr
            .provider_name()
            .and_then(|name| parsed.properties.get(name))
            .is_some_and(is_deprecated_property)
        {
            anyhow::ensure!(
                !docs.attributes.contains_key(name),
                "deprecated attribute {name} is documented"
            );
            continue;
        }
        verify_attribute_metadata(attr, docs.metadata.get(name))?;
        expected_attributes.insert(name.clone());
        queue.push_back((
            name.clone(),
            attr.attr_type(),
            vec![name.clone()],
            None::<String>,
        ));
    }
    anyhow::ensure!(
        expected_attributes == docs.attributes.keys().cloned().collect(),
        "attribute coverage differs: runtime {expected_attributes:?}, docs {:?}",
        docs.attributes.keys()
    );
    let mut visited = BTreeSet::new();
    let mut visited_cells = BTreeSet::new();
    let mut visited_enums = BTreeSet::new();
    let mut visited_targets = BTreeSet::new();
    let mut targets = BTreeMap::new();
    let mut bodies = BTreeMap::new();
    while let Some((name, ty, scope, declaration_target)) = queue.pop_front() {
        let actual = if let Some(target) = &declaration_target {
            visited_cells.insert((target.clone(), name.clone()));
            docs.declarations
                .get(target)
                .and_then(|fields| fields.get(&name))
        } else {
            docs.attributes.get(&name)
        };
        let note = notes
            .get(&(std::ptr::from_ref(ty) as usize))
            .map(String::as_str);
        let mut contexts = VecDeque::from(link_contexts(ty, &scope, &oracle)?);
        // Heading naming is provider policy. Feed that name through docgen's
        // typed hook; the independent runtime path/body checks below establish
        // that this really is the declaration for this use.
        if let Some(actual) = actual {
            let links = actual.iter().filter_map(|atom| {
                if let Inline::Link { url, .. } = atom {
                    Some(url)
                } else {
                    None
                }
            });
            for ((id, _), target) in contexts.iter().zip(links) {
                if let Some((_, heading)) = docs.headings.get(target) {
                    oracle.canonical_scopes.insert(*id, heading.clone());
                }
            }
        }
        let expected = expected_inline(ty, &scope, &mut oracle, &mut registry, note)?;
        let result = actual
            .context("missing Type cell")
            .and_then(|actual| compare_inline(&expected, actual));
        match result {
            Err(error) => errors.push(format!("{}: {error:#}", scope.join("."))),
            Ok(links) => {
                for (id, target) in links {
                    let (context_id, usage) = contexts
                        .pop_front()
                        .context("missing runtime link context")?;
                    anyhow::ensure!(context_id == id, "runtime link order differs");
                    let (declaration, _) = oracle
                        .declarations
                        .get(&id)
                        .context("unregistered expected link")?;
                    let scope = verify_target(&docs, &target, declaration, &usage)?;
                    visited_targets.insert(target.clone());
                    if let Some(previous) = targets.insert(id, target.clone()) {
                        anyhow::ensure!(
                            previous == target,
                            "runtime declaration maps to different headings"
                        );
                    }
                    let body = serde_json::to_string(
                        &carina_core::protocol_convert::attribute_type_to_protocol(declaration)?,
                    )?;
                    if let Some(previous) = bodies.insert(target.clone(), body.clone()) {
                        anyhow::ensure!(
                            previous == body,
                            "different runtime declaration bodies map to {target}"
                        );
                    }
                    if !visited.insert((id, target.clone())) {
                        continue;
                    }
                    match declaration.raw_shape() {
                        RawShape::Struct { fields, .. } => {
                            for field in fields {
                                let child =
                                    oracle.child_scope(&scope, TypeEdge::Field(field.name()));
                                queue.push_back((
                                    field.name().into(),
                                    field.field_type(),
                                    child,
                                    Some(target.clone()),
                                ));
                            }
                        }
                        RawShape::Union(members) => {
                            for (index, member) in members.iter().enumerate() {
                                let child = oracle.child_scope(&scope, TypeEdge::Member(index));
                                queue.push_back((
                                    format!("member_{index}"),
                                    member,
                                    child,
                                    Some(target.clone()),
                                ));
                            }
                        }
                        RawShape::Enum {
                            identity,
                            values: Some(values),
                            dsl_aliases,
                            ..
                        } => {
                            visited_enums.insert(target.clone());
                            let expected = values
                                .iter()
                                .map(|value| {
                                    let alias = dsl_aliases
                                        .iter()
                                        .find(|(wire, _)| wire == value)
                                        .map_or(value.as_str(), |(_, alias)| alias.as_str());
                                    (value.clone(), format!("{identity}.{alias}"))
                                })
                                .collect::<Vec<_>>();
                            if docs.enums.get(&target) != Some(&expected) {
                                errors.push(format!(
                                    "enum table {target}: expected {expected:?}, got {:?}",
                                    docs.enums.get(&target)
                                ));
                            }
                        }
                        shape => anyhow::bail!("unexpected runtime declaration {shape:?}"),
                    }
                }
            }
        }
    }
    let documented_cells = docs
        .declarations
        .iter()
        .flat_map(|(target, fields)| {
            fields
                .keys()
                .map(move |name| (target.clone(), name.clone()))
        })
        .collect::<BTreeSet<_>>();
    if visited_cells != documented_cells {
        errors.push(format!(
            "nested Type coverage differs: missing {:?}, extra {:?}",
            visited_cells
                .difference(&documented_cells)
                .collect::<Vec<_>>(),
            documented_cells
                .difference(&visited_cells)
                .collect::<Vec<_>>()
        ));
    }
    let declared_targets = docs.headings.keys().cloned().collect::<BTreeSet<_>>();
    if visited_targets != declared_targets {
        errors.push(format!("declaration target coverage differs: expected {visited_targets:?}, documented {declared_targets:?}"));
    }
    let documented_enums = docs.enums.keys().cloned().collect::<BTreeSet<_>>();
    if visited_enums != documented_enums {
        errors.push(format!("enum table coverage differs: expected {visited_enums:?}, documented {documented_enums:?}"));
    }
    anyhow::ensure!(errors.is_empty(), "{}", errors.join("\n"));
    Ok((
        expected_attributes.len(),
        visited_cells.len(),
        visited_enums.len(),
        docs.enums.values().map(Vec::len).sum(),
    ))
}

#[test]
fn cached_docs_match_every_compiled_runtime_type() {
    let cache = Path::new(env!("CARGO_MANIFEST_DIR")).join("../cfn-schema-cache");
    let cached = std::fs::read_dir(cache)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "json"))
        .map(|path| {
            path.file_stem()
                .unwrap()
                .to_str()
                .unwrap()
                .replace("__", "::")
        })
        .collect::<BTreeSet<_>>();
    let configs = carina_provider_awscc::schemas::generated::configs();
    assert_eq!(
        cached,
        configs
            .iter()
            .map(|config| config.aws_type_name.to_string())
            .collect(),
        "cached resources and compiled configs must cover each other"
    );
    let mut failures = Vec::new();
    let mut counts = (0, 0, 0, 0);
    for config in configs {
        let parsed = cfn_schema_for_codegen_tests(config.aws_type_name);
        let result = generate_test_markdown(&parsed, config.aws_type_name)
            .and_then(|markdown| check_parity(config.schema.as_schema(), &parsed, &markdown));
        match result {
            Ok((attributes, fields, enums, enum_rows)) => {
                counts.0 += attributes;
                counts.1 += fields;
                counts.2 += enums;
                counts.3 += enum_rows;
            }
            Err(error) => failures.push(format!("{}: {error:#}", config.aws_type_name)),
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n\n"));
    eprintln!(
        "runtime Type parity: {} resources, {} attributes, {} nested cells, {} enum tables, {} enum rows",
        configs.len(),
        counts.0,
        counts.1,
        counts.2,
        counts.3
    );
}

#[test]
fn structural_parser_preserves_pipes_and_backticks_and_rejects_duplicates() {
    let markdown = "## Argument Reference\n\n### `probe`\n\n- **Type:** ``String (`a|b`)``\n\n## Struct Definitions\n\n### Object\n\n| Field | Type |\n|---|---|\n| `value` | ``String (`a\\|b`)`` |\n";
    let parsed = parse_types(markdown).unwrap();
    assert_eq!(
        parsed.attributes["probe"],
        vec![Inline::Code("String (`a|b`)".into())]
    );
    assert_eq!(
        parsed.declarations["#object"]["value"],
        parsed.attributes["probe"]
    );
    assert!(parse_types(&format!("{markdown}\n### `probe`\n\n| Field | Type |\n|---|---|\n| `value` | `String` |\n| `value` | `String` |\n")).is_err());
}

#[test]
fn oracle_rejects_changed_bounds_patterns_ordering_uniqueness_and_missing_cells() {
    use carina_core::schema::{AttributeSchema, AttributeType, ResourceSchema};
    let ty = AttributeType::list(AttributeType::refined_string(
        None,
        Some(ApprovedPattern::whole("^(a|b)`$").as_spec().clone()),
        Some((Some(1), Some(8))),
        None,
    ));
    let compiled = ResourceSchema::new("test.Parity")
        .attribute(AttributeSchema::optional("probe", ty).build())
        .validate()
        .unwrap();
    let schema = compiled.as_schema();
    let parsed = ParsedCfnSchema::from_json(
        serde_json::json!({"typeName":"AWS::Test::Parity", "properties":{}}),
    )
    .unwrap();
    let mut oracle = Oracle {
        schema,
        declarations: BTreeMap::new(),
        canonical_scopes: HashMap::new(),
    };
    let display = render_attribute_type(
        schema.attributes()["probe"].attr_type(),
        &vec!["probe".into()],
        &mut oracle,
        &mut TypeRegistry::default(),
        None,
    )
    .unwrap();
    let mut page: PageBuilder<usize> = PageBuilder::new();
    page.heading(Heading::plain(2, "Argument Reference"))
        .unwrap();
    page.heading(Heading::code(3, "probe")).unwrap();
    page.push_str("- **Type:** ");
    display.write_to(&mut page);
    page.push_str("\n- **Required:** No\n");
    let markdown = page.finish().unwrap();
    check_parity(schema, &parsed, &markdown).unwrap();
    for (before, after) in [
        ("1..=8", "1..=9"),
        ("(a|b)", "(a|c)"),
        ("ordered", "unordered"),
        ("ordered", "unique items, ordered"),
        ("**Type:**", "**Missing:**"),
    ] {
        let mutated = markdown.replace(before, after);
        assert_ne!(mutated, markdown, "mutation must apply: {before}");
        assert!(
            check_parity(schema, &parsed, &mutated).is_err(),
            "accepted mutation {before} -> {after}"
        );
    }
}

#[test]
fn enum_parity_rejects_changed_values_aliases_and_coverage() {
    let identity = carina_core::schema::enum_identity("Mode", Some("aws.test.Round1"));
    let runtime = ResourceSchema::new("test.Round1")
        .attribute(
            AttributeSchema::optional(
                "mode",
                AttributeType::enum_(
                    identity,
                    Some(vec!["a".into(), "b".into()]),
                    vec![("a".into(), "alpha".into()), ("b".into(), "beta".into())],
                    None,
                    None,
                ),
            )
            .build(),
        )
        .validate()
        .unwrap();
    let parsed = ParsedCfnSchema::from_json(json!({
        "typeName":"AWS::Test::Round1", "properties":{"Mode":{"type":"string","enum":["a","b"]}}
    }))
    .unwrap();
    let markdown = "## Argument Reference\n\n### `mode`\n\n- **Type:** [Mode](#mode-mode)\n- **Required:** No\n\n## Enum Values\n\n### mode (Mode)\n\n| Value | DSL Identifier |\n|---|---|\n| `a` | `aws.test.Round1.Mode.alpha` |\n| `b` | `aws.test.Round1.Mode.beta` |\n";
    check_parity(runtime.as_schema(), &parsed, markdown).unwrap();
    let mutations = [
        ("wire value", markdown.replace("| `a` |", "| `changed` |")),
        ("alias", markdown.replace("Mode.alpha", "Mode.changed")),
        (
            "missing row",
            markdown.replace("| `b` | `aws.test.Round1.Mode.beta` |\n", ""),
        ),
        (
            "extra row",
            format!("{markdown}| `extra` | `aws.test.Round1.Mode.extra` |\n"),
        ),
        (
            "orphan table",
            format!(
                "{markdown}\n### orphan\n\n| Value | DSL Identifier |\n|---|---|\n| `a` | `aws.test.Round1.Mode.alpha` |\n"
            ),
        ),
    ];
    let accepted = mutations
        .iter()
        .filter_map(|(name, mutated)| {
            check_parity(runtime.as_schema(), &parsed, mutated)
                .is_ok()
                .then_some(*name)
        })
        .collect::<Vec<_>>();
    assert!(
        accepted.is_empty(),
        "enum oracle accepted mutations: {accepted:?}"
    );
}

#[test]
fn oracle_rejects_dangling_and_wrong_empty_struct_targets() {
    let emitted = emission(
        BTreeMap::from([
            (
                "left".into(),
                EmittedType::new("fixture", AttributeType::struct_("Left", vec![])),
            ),
            (
                "right".into(),
                EmittedType::new("fixture", AttributeType::struct_("Right", vec![])),
            ),
        ]),
        BTreeMap::new(),
    );
    let parsed =
        ParsedCfnSchema::from_json(json!({"typeName":"AWS::Test::Round2", "properties":{}}))
            .unwrap();
    let md = page(&emitted).unwrap();
    check_parity(emitted.witness.as_schema(), &parsed, &md).unwrap();
    for (name, mutated) in [
        ("dangling", md.replace("](#left-1)", "](#missing)")),
        ("wrong declaration", md.replace("](#left-1)", "](#right-1)")),
    ] {
        assert_ne!(md, mutated);
        assert!(
            check_parity(emitted.witness.as_schema(), &parsed, &mutated).is_err(),
            "oracle accepted {name}: {mutated}"
        );
    }
}

#[test]
fn oracle_rejects_unexpected_root_notes() {
    let parsed = ParsedCfnSchema::from_json(
        json!({"typeName":"AWS::Test::Round2", "properties":{"Probe":{"type":"boolean"}}}),
    )
    .unwrap();
    let runtime = ResourceSchema::new("test.Round2")
        .attribute(AttributeSchema::optional("probe", AttributeType::bool()).build())
        .validate()
        .unwrap();
    let md = generate_test_markdown(&parsed, &parsed.type_name).unwrap();
    check_parity(runtime.as_schema(), &parsed, &md).unwrap();
    let mutated = md.replace("**Type:** `Bool`", "**Type:** `Bool`; invented note");
    assert_ne!(md, mutated);
    assert!(
        check_parity(runtime.as_schema(), &parsed, &mutated).is_err(),
        "oracle accepted an invented note"
    );
}

#[test]
fn oracle_rejects_wrong_runtime_mode_bullets() {
    let parsed = cfn_schema_for_codegen_tests("AWS::IAM::Role");
    let config = carina_provider_awscc::schemas::generated::configs()
        .iter()
        .find(|c| c.aws_type_name == parsed.type_name)
        .unwrap();
    let md = generate_test_markdown(&parsed, &parsed.type_name).unwrap();
    check_parity(config.schema.as_schema(), &parsed, &md).unwrap();
    for (before, after) in [
        ("**Required:** Yes", "**Required:** No"),
        ("**Create-only:** Yes", "**Create-only:** No"),
        ("**Default:**", "**Other default:**"),
    ] {
        let mutated = md.replacen(before, after, 1);
        assert_ne!(md, mutated, "missing fixture bullet {before}");
        assert!(
            check_parity(config.schema.as_schema(), &parsed, &mutated).is_err(),
            "oracle accepted changed {before}"
        );
    }
}

#[test]
fn cfn_json_recursive_union_renders_list_map_and_note() {
    let map_rust = r#"AttributeType::map(AttributeType::ref_("__CfnJsonValue".to_string()))"#;
    for object_or_string in [false, true] {
        reset_recursion_guard();
        let kind = if object_or_string {
            json!(["object", "string"])
        } else {
            json!("object")
        };
        let parsed = ParsedCfnSchema::from_json(json!({
            "typeName":"AWS::Test::CfnJson", "properties":{"Probe":{"type":kind}}
        }))
        .unwrap();
        let context = EmissionContext::from_source(&parsed).unwrap();
        let root = cfn_json_type(&context.properties["Probe"]).unwrap();
        let expected_rust = if object_or_string {
            format!("AttributeType::union(vec![{map_rust}, AttributeType::string()])")
        } else {
            map_rust.into()
        };
        assert_eq!(root.rust, expected_rust);
        let defs = take_emitted_defs();
        assert_eq!(
            defs["__CfnJsonValue"].rust,
            concat!(
                "AttributeType::union(vec![AttributeType::string(), AttributeType::float(), AttributeType::bool(), ",
                "AttributeType::list(AttributeType::ref_(\"__CfnJsonValue\".to_string())), ",
                "AttributeType::map(AttributeType::ref_(\"__CfnJsonValue\".to_string()))])"
            )
        );
        let emitted = emission(BTreeMap::from([("probe".into(), root)]), defs);
        let reference = AttributeType::ref_("__CfnJsonValue");
        let map = AttributeType::map(reference.clone());
        let root_type = if object_or_string {
            AttributeType::union(vec![map, AttributeType::string()])
        } else {
            map
        };
        let runtime = ResourceSchema::new("test.CfnJson")
            .with_def(
                "__CfnJsonValue",
                AttributeType::union(vec![
                    AttributeType::string(),
                    AttributeType::float(),
                    AttributeType::bool(),
                    AttributeType::list(reference.clone()),
                    AttributeType::map(reference),
                ]),
            )
            .attribute(AttributeSchema::optional("probe", root_type).build())
            .validate()
            .unwrap();
        let md = page(&emitted).unwrap();
        assert_eq!(md.matches("; CFN-JSON").count(), 1, "{md}");
        assert!(md.contains("List\\<[Union("), "{md}");
        assert!(md.contains("Map\\<`String`, [Union("), "{md}");
        carina_docgen::check_page(&md).unwrap();
        check_parity_with_emission(runtime.as_schema(), &parsed, &md, Some(&emitted)).unwrap();
        for mutated in [
            md.replace("; CFN-JSON", ""),
            md.replace("; CFN-JSON", "; wrong CFN note"),
        ] {
            assert!(
                check_parity_with_emission(runtime.as_schema(), &parsed, &mutated, Some(&emitted))
                    .is_err()
            );
        }
    }
}

#[test]
fn four_integer_enum_attributes_are_inline_without_union_tables() {
    let cases = [
        (
            "AWS::CloudWatch::Alarm",
            "evaluation_interval",
            [10, 20, 30]
                .into_iter()
                .chain((60..=3600).step_by(60))
                .collect::<Vec<_>>(),
        ),
        (
            "AWS::CloudFront::Distribution",
            "error_code",
            vec![400, 403, 404, 405, 414, 416, 500, 501, 502, 503, 504],
        ),
        (
            "AWS::WAFv2::WebACL",
            "evaluation_window_sec",
            vec![60, 120, 300, 600],
        ),
        (
            "AWS::EC2::Subnet",
            "ipv6_netmask_length",
            vec![44, 48, 52, 56, 60, 64],
        ),
    ];
    for (name, field, values) in cases {
        let parsed = cfn_schema_for_codegen_tests(name);
        let md = generate_test_markdown(&parsed, name).unwrap();
        let expected = format!(
            "Int (one of: {})",
            values
                .iter()
                .map(i64::to_string)
                .collect::<Vec<_>>()
                .join(", ")
        );
        assert_inline_cells(&md, field, &expected).unwrap();
    }
}

#[test]
fn inline_union_key_before_struct_link_keeps_runtime_context() {
    let key = AttributeType::union(vec![
        AttributeType::refined_int(None, Some((Some(3), Some(3)))),
        AttributeType::refined_int(None, Some((Some(1), Some(1)))),
        AttributeType::refined_int(None, Some((Some(3), Some(3)))),
    ]);
    let value = AttributeType::struct_(
        "Settings",
        vec![StructField::optional("enabled", AttributeType::bool()).build()],
    );
    let resource = emission(
        BTreeMap::from([(
            "probe".into(),
            EmittedType::new(
                "fixture",
                AttributeType::map_with_key(key.clone(), value.clone()),
            ),
        )]),
        BTreeMap::new(),
    );
    let runtime = ResourceSchema::new("test.Mixed")
        .attribute(
            AttributeSchema::optional("probe", AttributeType::map_with_key(key, value)).build(),
        )
        .validate()
        .unwrap();
    let parsed =
        ParsedCfnSchema::from_json(json!({"typeName":"AWS::Test::Mixed", "properties":{}}))
            .unwrap();
    let md = page(&resource).unwrap();
    assert!(md.contains("`Int (one of: 1, 3)`"), "{md}");
    check_parity(runtime.as_schema(), &parsed, &md).unwrap();
    assert!(!md.contains("## Union Definitions"), "{md}");
}

#[test]
fn repeated_reference_links_retain_each_collection_scope() {
    let definition = EmittedType::new("fixture", AttributeType::struct_("Settings", vec![]));
    let resource = emission(
        BTreeMap::from([(
            "probe".into(),
            EmittedType::new(
                "fixture",
                AttributeType::map_with_key(
                    AttributeType::ref_("Settings"),
                    AttributeType::ref_("Settings"),
                ),
            ),
        )]),
        BTreeMap::from([("Settings".into(), definition)]),
    );
    let parsed =
        ParsedCfnSchema::from_json(json!({"typeName":"AWS::Test::Mixed", "properties":{}}))
            .unwrap();
    let md = page(&resource).unwrap();
    assert!(
        md.contains("Used by: `probe.<key>`, `probe.<value>`"),
        "{md}"
    );
    check_parity(resource.witness.as_schema(), &parsed, &md).unwrap();
}

#[test]
fn nested_cell_coverage_rejects_missing_extra_and_changed_fields() {
    let property =
        json!({"type":"object","properties":{"Value":{"type":"integer","minimum":2,"maximum":8}}});
    let expected = AttributeType::struct_(
        "Probe",
        vec![
            StructField::optional(
                "value",
                AttributeType::refined_int(None, Some((Some(2), Some(8)))),
            )
            .build(),
        ],
    );
    let parsed = ParsedCfnSchema::from_json(
        json!({"typeName":"AWS::Test::DocParity","properties":{"Probe":property}}),
    )
    .unwrap();
    let runtime = ResourceSchema::new("test.DocParity")
        .attribute(AttributeSchema::optional("probe", expected).build())
        .validate()
        .unwrap();
    let markdown = generate_test_markdown(&parsed, &parsed.type_name).unwrap();
    check_parity(runtime.as_schema(), &parsed, &markdown).unwrap();
    let missing = markdown
        .lines()
        .filter(|line| !line.starts_with("| `value` |"))
        .collect::<Vec<_>>()
        .join("\n");
    for mutated in [
        missing,
        markdown.replace("`value`", "`extra`"),
        markdown.replace("2..=8", "3..=8"),
    ] {
        assert!(check_parity(runtime.as_schema(), &parsed, &mutated).is_err());
    }
}
