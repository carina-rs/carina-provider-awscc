use super::*;
use carina_core::schema::{AttributeSchema, ResourceSchema};
use comrak::{Arena, Options, nodes::NodeValue, parse_document};
use serde_json::json;

pub(super) fn text<'a>(node: &'a comrak::nodes::AstNode<'a>) -> String {
    match &node.data.borrow().value {
        NodeValue::Text(value) => value.to_string(),
        NodeValue::Code(value) => value.literal.clone(),
        _ => node.children().map(text).collect(),
    }
}

pub(super) fn emission(
    attributes: BTreeMap<String, EmittedType>,
    defs: BTreeMap<String, EmittedType>,
) -> ResourceEmission {
    let mut schema = ResourceSchema::new("test.Round2");
    for (name, value) in &attributes {
        schema = schema.attribute(AttributeSchema::optional(name, *value.ty.clone()).build());
    }
    ResourceEmission {
        rust: String::new(),
        witness: type_emission::witness(schema, &defs).unwrap(),
        attributes,
        defs,
    }
}

pub(super) fn page(emission: &ResourceEmission) -> Result<String> {
    let mut docs = type_docs::TypeDocs::new(
        emission,
        emission
            .attributes
            .iter()
            .map(|(name, value)| (value.ty.as_ref(), type_docs::field_scope(name))),
    )?;
    let mut page = PageBuilder::new();
    page.heading(Heading::plain(2, "Argument Reference"))?;
    for (name, value) in &emission.attributes {
        page.heading(Heading::code(3, name))?;
        page.push_str("- **Type:** ");
        docs.display(&value.ty, type_docs::field_scope(name))?
            .write_to(&mut page);
        page.push_str("\n- **Required:** No\n\n");
    }
    docs.write_declarations(&mut page)?;
    Ok(page.finish()?)
}

pub(super) fn declaration_page(resource: &ResourceEmission) -> Result<String> {
    let mut types = type_docs::TypeDocs::new(
        resource,
        resource
            .attributes
            .iter()
            .map(|(name, value)| (value.ty.as_ref(), type_docs::field_scope(name))),
    )?;
    let mut page = PageBuilder::new();
    for (name, emitted) in &resource.attributes {
        types
            .display(&emitted.ty, type_docs::field_scope(name))?
            .write_to(&mut page);
        page.push_str("\n\n");
    }
    types.write_declarations(&mut page)?;
    Ok(page.finish()?)
}

pub(super) fn markdown(source: serde_json::Value) -> String {
    let parsed = ParsedCfnSchema::from_json(source).unwrap();
    generate_test_markdown(&parsed, &parsed.type_name).unwrap()
}

pub(super) fn root_link(markdown: &str, name: &str) -> (String, String, String) {
    let arena = Arena::new();
    let root = parse_document(&arena, markdown, &Options::default());
    let mut field = String::new();
    let mut found = None;
    let mut headings = BTreeMap::new();
    let mut ids = carina_docgen::heading_ids(markdown).into_iter();
    for node in root.children() {
        if let NodeValue::Heading(_) = node.data.borrow().value {
            field = text(node);
            headings.insert(format!("#{}", ids.next().unwrap()), field.clone());
        } else if field == name && found.is_none() {
            for node in node.descendants() {
                if let NodeValue::Link(link) = &node.data.borrow().value {
                    found = Some((text(node), link.url.clone()));
                    break;
                }
            }
        }
    }
    let (label, target) = found.expect("root type must contain a link");
    (label, target.clone(), headings[&target].clone())
}

pub(super) fn targets(markdown: &str) -> BTreeMap<String, String> {
    let arena = Arena::new();
    let root = parse_document(&arena, markdown, &Options::default());
    let mut ids = carina_docgen::heading_ids(markdown).into_iter();
    root.children()
        .filter_map(|node| match node.data.borrow().value {
            NodeValue::Heading(_) => Some((format!("#{}", ids.next().unwrap()), text(node))),
            _ => None,
        })
        .collect()
}

pub(super) fn link_target(markdown: &str, label: &str) -> String {
    let arena = Arena::new();
    let mut options = Options::default();
    options.extension.table = true;
    let root = parse_document(&arena, markdown, &options);
    root.descendants()
        .find_map(|node| match &node.data.borrow().value {
            NodeValue::Link(link) if text(node) == label => Some(link.url.clone()),
            _ => None,
        })
        .unwrap_or_else(|| panic!("missing {label} in {markdown}"))
}

pub(super) static NAMED_OVERRIDES: LazyLock<ResourceTypeOverrides> = LazyLock::new(|| {
    let mut overrides = ResourceTypeOverrides::default();
    for name in ["Left", "Right"] {
        overrides.insert(
            ("AWS::Test::NamedNotes", name),
            NarrowingKind::NamedType {
                name: "SharedText",
                expression: ConstraintSpec::string(None, None),
            },
        );
    }
    overrides
});

pub(super) fn named_schema() -> ParsedCfnSchema {
    ParsedCfnSchema::from_json(json!({
        "typeName":"AWS::Test::NamedNotes",
        "properties":{"Left":{"type":"string","format":"email"}, "Right":{"type":"string","format":"hostname"}}
    })).unwrap()
}
