//! Provider naming, declaration identity, and supplemental facts. Docgen owns type prose.
use super::*;
use carina_core::schema::{
    CompiledAttributeType, Draft, FieldGroup, PatternAttributeType, PatternForm, RawShape,
    StructField, TypeIdentity,
};
use carina_docgen::{
    Declaration, DeclarationKind, DocLink, Resolved, TypeDocHooks, TypeEdge, TypeRegistry,
    ValidationNote, render_attribute_type, render_field_groups,
};
use std::sync::Arc;

type DeclarationLinks<'a, R, H> = Vec<(
    <H as TypeDocHooks<'a, R>>::NodeId,
    &'a PatternAttributeType<R>,
    <H as TypeDocHooks<'a, R>>::Scope,
)>;

struct Discovery<'h, 'a, R: PatternForm, H: TypeDocHooks<'a, R>> {
    hooks: &'h H,
    declarations: HashMap<H::NodeId, Declaration<'a, R, H::NodeId>>,
}

impl<'a, R: PatternForm, H: TypeDocHooks<'a, R>> TypeDocHooks<'a, R> for Discovery<'_, 'a, R, H> {
    type NodeId = H::NodeId;
    type Target = ();
    type Scope = H::Scope;

    fn resolve(&self, ty: &'a PatternAttributeType<R>) -> Result<Resolved<'a, R, Self::NodeId>> {
        self.hooks.resolve(ty)
    }
    fn scalar_name(&self, identity: &TypeIdentity) -> String {
        self.hooks.scalar_name(identity)
    }
    fn child_scope(&self, parent: &Self::Scope, edge: TypeEdge<'_>) -> Self::Scope {
        self.hooks.child_scope(parent, edge)
    }
    fn register(
        &mut self,
        _: &Self::Scope,
        declaration: Declaration<'a, R, Self::NodeId>,
    ) -> Result<DocLink<()>> {
        self.declarations
            .insert(declaration.id.clone(), declaration);
        Ok(DocLink {
            target: (),
            label: String::new(),
        })
    }
}

// Discover links by asking docgen, never by classifying schema shapes locally.
// A registry can memoize repeated references in one cell. Walk collection edges
// afterwards to retain every occurrence's scope, stopping only at registered IDs.
fn discover<'h, 'a, R: PatternForm, H: TypeDocHooks<'a, R>>(
    ty: &'a PatternAttributeType<R>,
    scope: &H::Scope,
    hooks: &'h H,
) -> Result<Discovery<'h, 'a, R, H>> {
    let mut discovery = Discovery {
        hooks,
        declarations: HashMap::new(),
    };
    render_attribute_type(
        ty,
        scope,
        &mut discovery,
        &mut TypeRegistry::default(),
        None,
    )?;
    Ok(discovery)
}

impl<'a, R: PatternForm, H: TypeDocHooks<'a, R>> Discovery<'_, 'a, R, H> {
    fn links(
        &self,
        ty: &'a PatternAttributeType<R>,
        scope: &H::Scope,
    ) -> Result<DeclarationLinks<'a, R, H>> {
        fn collect<'a, R: PatternForm, H: TypeDocHooks<'a, R>>(
            ty: &'a PatternAttributeType<R>,
            scope: H::Scope,
            discovery: &Discovery<'_, 'a, R, H>,
            links: &mut DeclarationLinks<'a, R, H>,
        ) -> Result<()> {
            let resolved = discovery.resolve(ty)?;
            if discovery.declarations.contains_key(&resolved.id) {
                links.push((resolved.id, resolved.ty, scope));
                return Ok(());
            }
            match resolved.ty.raw_shape() {
                RawShape::List { element_type, .. } => collect(
                    element_type,
                    discovery.child_scope(&scope, TypeEdge::Item),
                    discovery,
                    links,
                )?,
                RawShape::Map { key, value, .. } => {
                    collect(
                        key,
                        discovery.child_scope(&scope, TypeEdge::Key),
                        discovery,
                        links,
                    )?;
                    collect(
                        value,
                        discovery.child_scope(&scope, TypeEdge::Value),
                        discovery,
                        links,
                    )?;
                }
                _ => {}
            }
            Ok(())
        }
        let mut links = Vec::new();
        collect(ty, scope.clone(), self, &mut links)?;
        Ok(links)
    }
}

#[cfg(test)]
pub(super) fn declaration_links<'a, R: PatternForm, H: TypeDocHooks<'a, R>>(
    ty: &'a PatternAttributeType<R>,
    scope: &H::Scope,
    hooks: &H,
) -> Result<DeclarationLinks<'a, R, H>> {
    discover(ty, scope, hooks)?.links(ty, scope)
}

// Admission supplies the body used for interning, while docgen supplies the set
// of declarations. Pointer identity here is local to this discovery pass only.
struct BodyHooks<'a>(&'a ResourceEmission);
impl<'a> TypeDocHooks<'a, Draft> for BodyHooks<'a> {
    type NodeId = usize;
    type Target = ();
    type Scope = ();
    fn resolve(&self, mut ty: &'a AttributeType) -> Result<Resolved<'a, Draft, usize>> {
        let mut seen = BTreeSet::new();
        while let RawShape::Ref(name) = ty.raw_shape() {
            anyhow::ensure!(seen.insert(name), "documentation alias cycle: {name}");
            ty = &self
                .0
                .defs
                .get(name)
                .with_context(|| format!("missing emitted definition {name}"))?
                .ty;
        }
        Ok(Resolved {
            id: std::ptr::from_ref(ty) as usize,
            ty,
        })
    }
    fn scalar_name(&self, identity: &TypeIdentity) -> String {
        identity.to_string()
    }
    fn child_scope(&self, _: &(), _: TypeEdge<'_>) {}
    fn register(&mut self, _: &(), _: Declaration<'a, Draft, usize>) -> Result<DocLink<()>> {
        Ok(DocLink {
            target: (),
            label: String::new(),
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(super) enum ScopeSegment {
    Field(String),
    Item,
    Key,
    Value,
    Member(usize),
}

type Scope = Vec<ScopeSegment>;
type LinkedType<'a> = (NodeId, &'a AttributeType, Scope, DeclarationBody<'a>);
type Usages = BTreeMap<NodeId, BTreeSet<Scope>>;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(super) enum NodeId {
    Enum(TypeIdentity),
    Body(Arc<str>),
    Attribute(String),
    Definition(String),
    Child(Box<NodeId>, ScopeSegment),
}

impl NodeId {
    fn child(&self, edge: ScopeSegment) -> Self {
        Self::Child(Box::new(self.clone()), edge)
    }
    fn description(&self) -> String {
        match self {
            Self::Body(_) => "interned compound declaration".into(),
            Self::Enum(identity) => format!("enum {identity}"),
            Self::Attribute(name) => format!("attribute {name}"),
            Self::Definition(name) => format!("definition {name}"),
            Self::Child(parent, edge) => format!("{} {edge:?}", parent.description()),
        }
    }
}

#[derive(Debug)]
pub(super) enum TypeDocError {
    RecursiveCollection { node: NodeId, scope: String },
    MissingHeading { node: NodeId },
}

impl std::fmt::Display for TypeDocError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::RecursiveCollection { node, scope } => write!(
                f,
                "unsupported recursive collection at {scope} (node {})",
                node.description()
            ),
            Self::MissingHeading { node } => write!(
                f,
                "missing prepared declaration heading for node {}",
                node.description()
            ),
        }
    }
}
impl std::error::Error for TypeDocError {}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum DeclarationSection {
    Enum,
    Struct,
    Union,
}
impl DeclarationSection {
    fn heading(self) -> &'static str {
        match self {
            Self::Enum => "Enum Values",
            Self::Struct => "Struct Definitions",
            Self::Union => "Union Definitions",
        }
    }
}

// Recursive registrations are rejected at the boundary. Everything retained
// for heading preparation and table rendering has one of these three bodies.
#[derive(Clone, Copy)]
enum DeclarationBody<'a> {
    Enum {
        identity: &'a TypeIdentity,
        values: &'a [String],
        dsl_aliases: &'a [(String, String)],
    },
    Struct {
        name: &'a str,
        fields: &'a [StructField],
        field_groups: &'a [FieldGroup],
    },
    Union {
        members: &'a [AttributeType],
    },
}

impl<'a> DeclarationBody<'a> {
    fn from_docgen(kind: &DeclarationKind<'a, Draft>, id: &NodeId, scope: &Scope) -> Result<Self> {
        Ok(match kind {
            DeclarationKind::Enum {
                identity,
                values,
                dsl_aliases,
                ..
            } => Self::Enum {
                identity,
                values,
                dsl_aliases,
            },
            DeclarationKind::Struct {
                name,
                fields,
                field_groups,
            } => Self::Struct {
                name,
                fields,
                field_groups,
            },
            DeclarationKind::Union { members } => Self::Union { members },
            DeclarationKind::Recursive => {
                return Err(TypeDocError::RecursiveCollection {
                    node: id.clone(),
                    scope: visible_scope(scope),
                }
                .into());
            }
        })
    }

    fn section(self) -> DeclarationSection {
        match self {
            Self::Enum { .. } => DeclarationSection::Enum,
            Self::Struct { .. } => DeclarationSection::Struct,
            Self::Union { .. } => DeclarationSection::Union,
        }
    }

    fn heading(self, scope: &Scope) -> String {
        match self {
            Self::Enum { identity, .. } => format!("{} ({})", visible_scope(scope), identity.kind),
            Self::Struct { .. } | Self::Union { .. } => visible_scope(scope),
        }
    }

    fn label(self, scope: &Scope) -> String {
        match self {
            Self::Enum { identity, .. } => identity.kind.clone(),
            Self::Struct { name, .. } => name.into(),
            Self::Union { .. } => visible_scope(scope),
        }
    }

    fn base(self) -> String {
        match self {
            Self::Enum { identity, .. } => identity.kind.to_snake_case(),
            Self::Struct { name, .. } => name.to_snake_case(),
            Self::Union { members } => format!(
                "union_of_{}",
                members
                    .iter()
                    .map(heading_part)
                    .collect::<Vec<_>>()
                    .join("_or_")
            ),
        }
    }

    fn children(self) -> Vec<(TypeEdge<'a>, &'a AttributeType)> {
        match self {
            Self::Enum { .. } => Vec::new(),
            Self::Struct { fields, .. } => fields
                .iter()
                .map(|field| (TypeEdge::Field(field.name()), field.field_type()))
                .collect(),
            Self::Union { members } => members
                .iter()
                .enumerate()
                .map(|(index, member)| (TypeEdge::Member(index), member))
                .collect(),
        }
    }
}

struct PreparedDeclaration<'a> {
    scope: Scope,
    id: NodeId,
    ty: &'a AttributeType,
    body: DeclarationBody<'a>,
}

impl PreparedDeclaration<'_> {
    fn heading(&self) -> String {
        self.body.heading(&self.scope)
    }
}

pub(super) fn field_scope(name: impl Into<String>) -> Scope {
    vec![ScopeSegment::Field(name.into())]
}

struct Node<'a> {
    id: NodeId,
    annotations: Option<&'a TypeAnnotations>,
}

// Intern only compounds registered by docgen, by admitted body and annotations.
fn body_ids(emission: &ResourceEmission) -> Result<HashMap<usize, NodeId>> {
    fn collect<'a>(
        hooks: &BodyHooks<'a>,
        ty: &'a AttributeType,
        info: Option<&TypeAnnotations>,
        compiled: &CompiledAttributeType,
        owners: &mut Vec<(usize, String)>,
        variants: &mut BTreeSet<String>,
    ) -> Result<()> {
        let discovery = discover(ty, &(), hooks)?;
        if discovery
            .declarations
            .get(&(std::ptr::from_ref(ty) as usize))
            .is_some_and(|declaration| {
                matches!(
                    declaration.kind,
                    DeclarationKind::Struct { .. } | DeclarationKind::Union { .. }
                )
            })
        {
            let body = carina_core::protocol_convert::attribute_type_to_protocol(compiled)?;
            let annotations = info.cloned().unwrap_or_default();
            let signature = format!(
                "{}; annotations: {annotations:?}",
                serde_json::to_string(&body)?
            );
            variants.insert(signature.clone());
            owners.push((std::ptr::from_ref(ty) as usize, signature));
        }
        let child = |name: &str| info.and_then(|info| info.children.get(name));
        match (ty.raw_shape(), compiled.raw_shape()) {
            (
                RawShape::Struct { fields, .. },
                RawShape::Struct {
                    fields: admitted, ..
                },
            ) => {
                anyhow::ensure!(
                    fields.len() == admitted.len(),
                    "witness struct shape differs"
                );
                for (field, admitted) in fields.iter().zip(admitted) {
                    anyhow::ensure!(
                        field.name() == admitted.name(),
                        "witness field order differs"
                    );
                    collect(
                        hooks,
                        field.field_type(),
                        child(field.name()),
                        admitted.field_type(),
                        owners,
                        variants,
                    )?;
                }
            }
            (
                RawShape::List { element_type, .. },
                RawShape::List {
                    element_type: admitted,
                    ..
                },
            ) => collect(
                hooks,
                element_type,
                child("item"),
                admitted,
                owners,
                variants,
            )?,
            (
                RawShape::Map { key, value, .. },
                RawShape::Map {
                    key: admitted_key,
                    value: admitted_value,
                    ..
                },
            ) => {
                collect(hooks, key, child("key"), admitted_key, owners, variants)?;
                collect(
                    hooks,
                    value,
                    child("value"),
                    admitted_value,
                    owners,
                    variants,
                )?;
            }
            (RawShape::Union(members), RawShape::Union(admitted)) => {
                anyhow::ensure!(
                    members.len() == admitted.len(),
                    "witness union shape differs"
                );
                for (index, (member, admitted)) in members.iter().zip(admitted).enumerate() {
                    collect(
                        hooks,
                        member,
                        child(&format!("member_{index}")),
                        admitted,
                        owners,
                        variants,
                    )?;
                }
            }
            _ => {}
        }
        Ok(())
    }
    let mut owners = Vec::new();
    let mut variants = BTreeSet::new();
    let witness = emission.witness.parts();
    let hooks = BodyHooks(emission);
    for (name, emitted) in &emission.attributes {
        collect(
            &hooks,
            &emitted.ty,
            Some(&emitted.annotations),
            witness.attributes[name].attr_type(),
            &mut owners,
            &mut variants,
        )?;
    }
    for (name, emitted) in &emission.defs {
        collect(
            &hooks,
            &emitted.ty,
            Some(&emitted.annotations),
            &witness.defs[name],
            &mut owners,
            &mut variants,
        )?;
    }
    let mut ids = HashMap::new();
    let interned = variants
        .into_iter()
        .map(|signature| {
            let id = NodeId::Body(Arc::from(signature.as_str()));
            (signature, id)
        })
        .collect::<BTreeMap<_, _>>();
    for (pointer, signature) in owners {
        ids.insert(pointer, interned[&signature].clone());
    }
    Ok(ids)
}

#[derive(PartialEq)]
struct EnumBody<'a> {
    values: &'a [String],
    aliases: &'a [(String, String)],
    to_dsl: Option<&'a carina_core::schema::DslTransform>,
}

struct Hooks<'a> {
    emission: &'a ResourceEmission,
    nodes: HashMap<usize, Node<'a>>,
    declarations: Vec<PreparedDeclaration<'a>>,
    uses: Usages,
    definition_names: BTreeMap<NodeId, String>,
    headings: BTreeMap<NodeId, Scope>,
    body_ids: HashMap<usize, NodeId>,
    enum_bodies: RefCell<BTreeMap<NodeId, EnumBody<'a>>>,
}

impl<'a> Hooks<'a> {
    fn index(
        &mut self,
        ty: &'a AttributeType,
        annotations: Option<&'a TypeAnnotations>,
        path: NodeId,
    ) {
        let id = if let RawShape::Enum {
            identity,
            values: Some(_),
            ..
        } = ty.raw_shape()
        {
            NodeId::Enum(identity.clone())
        } else {
            self.body_ids
                .get(&(std::ptr::from_ref(ty) as usize))
                .cloned()
                .unwrap_or(path)
        };
        if let Some(name) = annotations.and_then(|info| info.definition.as_ref()) {
            self.definition_names
                .insert(id.clone(), name.to_snake_case());
        }
        self.nodes.insert(
            std::ptr::from_ref(ty) as usize,
            Node {
                id: id.clone(),
                annotations,
            },
        );
        let child = |name: &str| annotations.and_then(|annotations| annotations.children.get(name));
        match ty.raw_shape() {
            RawShape::Struct { fields, .. } => {
                for field in fields {
                    self.index(
                        field.field_type(),
                        child(field.name()),
                        id.child(ScopeSegment::Field(field.name().into())),
                    );
                }
            }
            RawShape::List { element_type, .. } => {
                self.index(element_type, child("item"), id.child(ScopeSegment::Item))
            }
            RawShape::Map { key, value, .. } => {
                self.index(key, child("key"), id.child(ScopeSegment::Key));
                self.index(value, child("value"), id.child(ScopeSegment::Value));
            }
            RawShape::Union(members) => {
                for (index, member) in members.iter().enumerate() {
                    let name = format!("member_{index}");
                    self.index(member, child(&name), id.child(ScopeSegment::Member(index)));
                }
            }
            _ => {}
        }
    }

    fn heading(&self, id: &NodeId) -> Result<&Scope> {
        self.headings
            .get(id)
            .ok_or_else(|| TypeDocError::MissingHeading { node: id.clone() }.into())
    }

    fn shared_base(&self, id: &NodeId, body: DeclarationBody<'a>) -> String {
        self.definition_names
            .get(id)
            .cloned()
            .unwrap_or_else(|| body.base())
    }

    fn node(&self, ty: &AttributeType) -> Result<&Node<'a>> {
        self.nodes
            .get(&(std::ptr::from_ref(ty) as usize))
            .context("unindexed emitted type")
    }

    fn links(
        &self,
        ty: &'a AttributeType,
        scope: Scope,
        links: &mut Vec<LinkedType<'a>>,
    ) -> Result<()> {
        let discovery = discover(ty, &scope, self)?;
        for (id, ty, scope) in discovery.links(ty, &scope)? {
            let body =
                DeclarationBody::from_docgen(&discovery.declarations[&id].kind, &id, &scope)?;
            links.push((id, ty, scope, body));
        }
        Ok(())
    }

    fn notes(&self, ty: &'a AttributeType) -> Result<String> {
        fn collect<'a>(
            hooks: &Hooks<'a>,
            ty: &'a AttributeType,
            path: &str,
            notes: &mut Vec<String>,
            active: &mut BTreeSet<usize>,
        ) -> Result<()> {
            let mut ty = ty;
            let mut seen = BTreeSet::new();
            loop {
                let info = hooks.node(ty)?;
                for note in info.annotations.into_iter().flat_map(|info| &info.notes) {
                    let note = if path.is_empty() {
                        note.clone()
                    } else {
                        format!("{path}: {note}")
                    };
                    notes.push(note);
                }
                let RawShape::Ref(name) = ty.raw_shape() else {
                    break;
                };
                anyhow::ensure!(seen.insert(name), "documentation alias cycle: {name}");
                ty = &hooks
                    .emission
                    .defs
                    .get(name)
                    .with_context(|| format!("missing emitted definition {name}"))?
                    .ty;
            }
            let pointer = std::ptr::from_ref(ty) as usize;
            if !active.insert(pointer) {
                return Ok(());
            }
            let child = |name: &str| {
                if path.is_empty() {
                    name.into()
                } else {
                    format!("{path}.{name}")
                }
            };
            match ty.raw_shape() {
                RawShape::List { element_type, .. } => {
                    collect(hooks, element_type, &child("item"), notes, active)?
                }
                RawShape::Map { key, value, .. } => {
                    collect(hooks, key, &child("key"), notes, active)?;
                    collect(hooks, value, &child("value"), notes, active)?;
                }
                _ => {}
            }
            active.remove(&pointer);
            Ok(())
        }
        let mut notes = Vec::new();
        collect(self, ty, "", &mut notes, &mut BTreeSet::new())?;
        Ok(notes.join("; "))
    }
}

impl<'a> TypeDocHooks<'a, Draft> for Hooks<'a> {
    type NodeId = NodeId;
    type Target = MarkdownTarget;
    type Scope = Scope;

    fn resolve(&self, mut ty: &'a AttributeType) -> Result<Resolved<'a, Draft, NodeId>> {
        let mut seen = BTreeSet::new();
        while let RawShape::Ref(name) = ty.raw_shape() {
            anyhow::ensure!(seen.insert(name), "documentation alias cycle: {name}");
            ty = &self
                .emission
                .defs
                .get(name)
                .with_context(|| format!("missing emitted definition {name}"))?
                .ty;
        }
        let node = self.node(ty)?;
        if let RawShape::Enum {
            values: Some(values),
            dsl_aliases,
            to_dsl,
            ..
        } = ty.raw_shape()
        {
            let body = EnumBody {
                values,
                aliases: dsl_aliases,
                to_dsl,
            };
            let mut bodies = self.enum_bodies.borrow_mut();
            if let Some(previous) = bodies.get(&node.id) {
                anyhow::ensure!(
                    previous == &body,
                    "conflicting enum declaration {:?}",
                    node.id
                );
            } else {
                bodies.insert(node.id.clone(), body);
            }
        }
        Ok(Resolved {
            id: node.id.clone(),
            ty,
        })
    }

    fn scalar_name(&self, identity: &TypeIdentity) -> String {
        identity.to_string()
    }

    fn child_scope(&self, parent: &Scope, edge: TypeEdge<'_>) -> Scope {
        let mut child = parent.clone();
        child.push(match edge {
            TypeEdge::Field(name) => ScopeSegment::Field(name.into()),
            TypeEdge::Item => ScopeSegment::Item,
            TypeEdge::Key => ScopeSegment::Key,
            TypeEdge::Value => ScopeSegment::Value,
            TypeEdge::Member(index) => ScopeSegment::Member(index),
        });
        child
    }

    fn register(
        &mut self,
        scope: &Scope,
        declaration: Declaration<'a, Draft, NodeId>,
    ) -> Result<DocLink<MarkdownTarget>> {
        let body = DeclarationBody::from_docgen(&declaration.kind, &declaration.id, scope)?;
        let target = MarkdownTarget::Typed(declaration.id.clone());
        let scope = self.heading(&declaration.id)?.clone();
        let label = body.label(&scope);
        self.declarations.push(PreparedDeclaration {
            scope,
            id: declaration.id,
            ty: declaration.ty,
            body,
        });
        Ok(DocLink { target, label })
    }
}

pub(super) struct TypeDocs<'a> {
    hooks: Hooks<'a>,
    registry: TypeRegistry<NodeId, MarkdownTarget>,
    roots: Vec<(&'a AttributeType, Scope)>,
}

impl<'a> TypeDocs<'a> {
    pub(super) fn new(
        emission: &'a ResourceEmission,
        roots: impl IntoIterator<Item = (&'a AttributeType, Scope)>,
    ) -> Result<Self> {
        debug_assert_eq!(
            emission.witness.as_schema().attributes().len(),
            emission.attributes.len()
        );
        let mut hooks = Hooks {
            emission,
            nodes: HashMap::new(),
            declarations: Vec::new(),
            uses: BTreeMap::new(),
            definition_names: BTreeMap::new(),
            headings: BTreeMap::new(),
            body_ids: body_ids(emission)?,
            enum_bodies: RefCell::new(BTreeMap::new()),
        };
        for (name, emitted) in &emission.attributes {
            hooks.index(
                &emitted.ty,
                Some(&emitted.annotations),
                NodeId::Attribute(name.clone()),
            );
        }
        for (name, emitted) in &emission.defs {
            hooks.index(
                &emitted.ty,
                Some(&emitted.annotations),
                NodeId::Definition(name.clone()),
            );
            hooks.definition_names.insert(
                hooks
                    .node(&emitted.ty)
                    .expect("indexed definition")
                    .id
                    .clone(),
                name.to_snake_case(),
            );
        }
        let mut docs = Self {
            hooks,
            registry: TypeRegistry::default(),
            roots: roots.into_iter().collect(),
        };
        docs.prepare_usages()?;
        Ok(docs)
    }

    pub(super) fn display(
        &mut self,
        ty: &'a AttributeType,
        scope: Scope,
    ) -> Result<carina_docgen::TypeDisplay<MarkdownTarget>> {
        let note = self.hooks.notes(ty)?;
        render_attribute_type(
            ty,
            &scope,
            &mut self.hooks,
            &mut self.registry,
            Some(ValidationNote::new(&note)),
        )
    }

    fn prepare_usages(&mut self) -> Result<()> {
        let mut roots = Vec::new();
        for (ty, scope) in &self.roots {
            self.hooks.links(ty, scope.clone(), &mut roots)?;
        }
        let mut pending = roots.clone();
        let mut nodes = BTreeMap::new();
        let mut edges = BTreeMap::new();
        while let Some((id, ty, _, body)) = pending.pop() {
            if nodes.insert(id.clone(), (ty, body)).is_some() {
                continue;
            }
            let mut children = Vec::new();
            for (edge, child) in body.children() {
                self.hooks.links(
                    child,
                    self.hooks.child_scope(&Vec::new(), edge),
                    &mut children,
                )?;
            }
            pending.extend(children.iter().cloned());
            edges.insert(id, children);
        }
        let mut root_uses = Usages::new();
        for (id, _, scope, _) in roots {
            root_uses.entry(id).or_default().insert(visible_path(scope));
        }
        let mut uses = root_uses.clone();
        // A shared parent's declaration replaces its multiple use paths with
        // one visible table path. Propagate that path before rendering cells.
        // The same final set controls both headings and the Used by line.
        for _ in 0..(nodes.len() * 4 + 4) {
            // Only declarations that actually use a base name participate in
            // its ordinal group. Recompute after visible parent paths settle.
            let mut groups: BTreeMap<String, Vec<NodeId>> = BTreeMap::new();
            for (id, (_, body)) in &nodes {
                if uses.get(id).is_some_and(needs_base_name) {
                    groups
                        .entry(self.hooks.shared_base(id, *body))
                        .or_default()
                        .push(id.clone());
                }
            }
            let mut shared_names = BTreeMap::new();
            for (base, ids) in &groups {
                for (index, id) in ids.iter().enumerate() {
                    let name = if ids.len() == 1 {
                        base.clone()
                    } else {
                        format!("{base}_{}", index + 1)
                    };
                    shared_names.insert(id.clone(), name);
                }
            }
            let mut headings = BTreeMap::new();
            let mut reserved = BTreeSet::new();
            for (id, (_, body)) in &nodes {
                let Some(paths) = uses.get(id) else { continue };
                let base = if needs_base_name(paths) {
                    shared_names[id].clone()
                } else {
                    visible_scope(paths.first().expect("nonempty usage set"))
                };
                let title = |name: &str| body.heading(&field_scope(name));
                let mut name = base.clone();
                let mut ordinal = 2;
                while !reserved.insert(title(&name)) {
                    name = format!("{base}_{ordinal}");
                    ordinal += 1;
                }
                headings.insert(id.clone(), field_scope(name));
            }
            let mut next = root_uses.clone();
            for (parent, children) in &edges {
                let Some(scope) = headings.get(parent) else {
                    continue;
                };
                for (child, _, relative, _) in children {
                    let mut path = scope.clone();
                    path.extend(relative.clone());
                    next.entry(child.clone())
                        .or_default()
                        .insert(visible_path(path));
                }
            }
            if next == uses {
                self.hooks.uses = uses;
                self.hooks.headings = headings;
                for (id, (ty, _)) in nodes {
                    let scope = self.hooks.heading(&id)?.clone();
                    // This registers declaration metadata only. Compound nodes
                    // do not render their fields or members during this pass.
                    render_attribute_type(ty, &scope, &mut self.hooks, &mut self.registry, None)?;
                }
                return Ok(());
            }
            uses = next;
        }
        anyhow::bail!("declaration usage paths did not converge")
    }

    pub(super) fn write_declarations(
        &mut self,
        page: &mut PageBuilder<MarkdownTarget>,
    ) -> Result<()> {
        let mut cells = Vec::new();
        for cursor in 0..self.hooks.declarations.len() {
            let item = &self.hooks.declarations[cursor];
            let scope = item.scope.clone();
            let children = item.body.children();
            let mut rendered = Vec::new();
            for (edge, ty) in children {
                rendered.push(self.display(ty, self.hooks.child_scope(&scope, edge))?);
            }
            cells.push(rendered);
        }
        let mut order = self
            .hooks
            .declarations
            .iter()
            .enumerate()
            .map(|(index, item)| (item.body.section(), item.heading(), item.id.clone(), index))
            .collect::<Vec<_>>();
        order.sort();
        let mut section = None;
        for (category, heading, id, cursor) in order {
            let item = &self.hooks.declarations[cursor];
            let ty = item.ty;
            if section != Some(category) {
                page.heading(Heading::plain(2, category.heading()))?;
                section = Some(category);
            }
            let uses = self
                .hooks
                .uses
                .get(&id)
                .filter(|uses| needs_base_name(uses))
                .cloned();
            let target = MarkdownTarget::Typed(id);
            page.target_heading(target, Heading::plain(3, heading))?;
            if let Some(uses) = uses {
                let uses = uses
                    .iter()
                    .map(|path| format!("`{}`", visible_scope(path)))
                    .collect::<Vec<_>>()
                    .join(", ");
                page.push_str(&format!("Used by: {uses}\n\n"));
            }
            match &item.body {
                DeclarationBody::Enum {
                    identity,
                    values,
                    dsl_aliases,
                    ..
                } => {
                    let dsl_alias = |value: &'a String| {
                        dsl_aliases
                            .iter()
                            .find(|(canonical, _)| canonical == value)
                            .map_or(value.as_str(), |(_, dsl)| dsl.as_str())
                    };
                    page.table(
                        ["Value", "DSL Identifier"],
                        values.iter().map(|value| {
                            let dsl = dsl_alias(value);
                            [
                                TableCell::code(value),
                                TableCell::code(format!("{identity}.{dsl}")),
                            ]
                        }),
                    )?;
                    if let Some(value) = values.first() {
                        let dsl = dsl_alias(value);
                        if !dsl.starts_with(|ch: char| ch.is_ascii_digit()) {
                            page.push_str(&format!(
                                "\nShorthand formats: `{dsl}` or `{}.{dsl}`\n\n",
                                identity.kind
                            ));
                        } else {
                            page.push_str("\nUse the fully qualified DSL identifiers above; bare numeric values parse as integers.\n\n");
                        }
                    }
                }
                DeclarationBody::Struct {
                    fields,
                    field_groups,
                    ..
                } => {
                    let mut rows = Vec::new();
                    for (index, field) in fields.iter().enumerate() {
                        let display = &cells[cursor][index];
                        let description = self
                            .hooks
                            .node(ty)?
                            .annotations
                            .and_then(|info| info.descriptions.get(field.name()))
                            .map_or_else(
                                || field.description().unwrap_or_default().to_string(),
                                Clone::clone,
                            );
                        rows.push([
                            TableCell::code(field.name()),
                            display.table_cell(),
                            TableCell::text(if field.parts().declared_input_mode.is_required() {
                                "Yes"
                            } else {
                                "No"
                            }),
                            TableCell::text(description),
                        ]);
                    }
                    page.table(["Field", "Type", "Required", "Description"], rows)?;
                    page.push_str("\n");
                    render_field_groups(page, field_groups);
                    if let Some(annotations) = self.hooks.node(ty)?.annotations {
                        write_conditional_requirement_notes(
                            page,
                            &annotations.conditional_requirements,
                            |member| Ok(member.to_snake_case()),
                        )?;
                        for groups in annotations.map_requirements.values() {
                            for requirements in groups {
                                write_conditional_requirement_notes(page, requirements, |key| {
                                    Ok(key.into())
                                })?;
                            }
                        }
                    }
                }
                DeclarationBody::Union { members } => {
                    let mut rows = Vec::new();
                    for (index, _member) in members.iter().enumerate() {
                        rows.push([
                            TableCell::code(format!("member_{index}")),
                            cells[cursor][index].table_cell(),
                        ]);
                    }
                    page.table(["Member", "Type"], rows)?;
                    page.push_str("\n");
                }
            }
        }
        Ok(())
    }
}

fn visible_path(scope: Scope) -> Scope {
    scope
        .into_iter()
        .filter(|part| !matches!(part, ScopeSegment::Item))
        .collect()
}

fn needs_base_name(paths: &BTreeSet<Scope>) -> bool {
    paths.len() > 1
        || paths
            .first()
            .is_some_and(|path| matches!(path.last(), Some(ScopeSegment::Member(_))))
}

fn visible_scope(scope: &[ScopeSegment]) -> String {
    scope
        .iter()
        .filter_map(|part| match part {
            ScopeSegment::Field(name) => Some(name.clone()),
            ScopeSegment::Key => Some("<key>".into()),
            ScopeSegment::Member(index) => Some(format!("member_{index}")),
            ScopeSegment::Value => Some("<value>".into()),
            ScopeSegment::Item => None,
        })
        .collect::<Vec<_>>()
        .join(".")
}

// These are anchor names, not type cells. Type wording/classification remains
// entirely in docgen; member/field names make shared declarations recognizable.
fn heading_part(ty: &AttributeType) -> String {
    let named = |identity: Option<&TypeIdentity>, builtin: &str| {
        identity.map_or_else(|| builtin.into(), |identity| identity.kind.to_snake_case())
    };
    match ty.raw_shape() {
        RawShape::String { identity_layer, .. } => named(identity_layer.identity, "string"),
        RawShape::Int { identity_layer, .. } => named(identity_layer.identity, "int"),
        RawShape::Float { identity, .. } => named(identity, "float"),
        RawShape::Bool => "bool".into(),
        RawShape::Duration => "duration".into(),
        RawShape::Reference { identity, .. } | RawShape::Enum { identity, .. } => {
            identity.kind.to_snake_case()
        }
        RawShape::Struct { name, .. } | RawShape::Ref(name) => name.to_snake_case(),
        RawShape::List { element_type, .. } => format!("list_of_{}", heading_part(element_type)),
        RawShape::Map { key, value, .. } => {
            format!("map_of_{}_to_{}", heading_part(key), heading_part(value))
        }
        RawShape::Union(members) => format!(
            "union_of_{}",
            members
                .iter()
                .map(heading_part)
                .collect::<Vec<_>>()
                .join("_or_")
        ),
        RawShape::UnloadedSchemaType(_) => "unloaded".into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inline_union_has_no_declaration_headings_usages_or_interned_body() {
        let ty = AttributeType::union(vec![
            AttributeType::refined_int(None, Some((Some(1), Some(1)))),
            AttributeType::refined_int(None, Some((Some(3), Some(3)))),
        ]);
        let emission = ResourceEmission {
            rust: String::new(),
            attributes: BTreeMap::from([("probe".into(), EmittedType::new("fixture", ty.clone()))]),
            defs: BTreeMap::new(),
            witness: carina_core::schema::ResourceSchema::new("test.Inline")
                .attribute(carina_core::schema::AttributeSchema::optional("probe", ty).build())
                .validate()
                .unwrap(),
        };
        let docs = TypeDocs::new(
            &emission,
            [(
                emission.attributes["probe"].ty.as_ref(),
                field_scope("probe"),
            )],
        )
        .unwrap();
        assert!(docs.hooks.declarations.is_empty());
        assert!(docs.hooks.headings.is_empty());
        assert!(docs.hooks.uses.is_empty());
        assert!(docs.hooks.body_ids.is_empty());
    }

    #[test]
    fn missing_prepared_heading_returns_a_typed_error() {
        let ty = AttributeType::union(vec![AttributeType::bool(), AttributeType::string()]);
        let emission = ResourceEmission {
            rust: String::new(),
            attributes: BTreeMap::from([("probe".into(), EmittedType::new("fixture", ty.clone()))]),
            defs: BTreeMap::new(),
            witness: carina_core::schema::ResourceSchema::new("test.Prepared")
                .attribute(carina_core::schema::AttributeSchema::optional("probe", ty).build())
                .validate()
                .unwrap(),
        };
        let ty = emission.attributes["probe"].ty.as_ref();
        let mut docs = TypeDocs::new(&emission, [(ty, field_scope("probe"))]).unwrap();
        docs.hooks.headings.clear();
        docs.registry = TypeRegistry::default();
        let error = docs.display(ty, field_scope("probe")).unwrap_err();
        assert!(
            matches!(
                error.downcast_ref::<TypeDocError>(),
                Some(TypeDocError::MissingHeading { .. })
            ),
            "{error:#}"
        );
    }

    #[test]
    fn recursive_collection_is_an_error_without_a_panic() {
        let defs = BTreeMap::from([(
            "A".into(),
            EmittedType::new("fixture", AttributeType::list(AttributeType::ref_("A"))),
        )]);
        let emission = ResourceEmission {
            rust: String::new(),
            attributes: BTreeMap::new(),
            witness: type_emission::witness(
                carina_core::schema::ResourceSchema::new("test.Recursive"),
                &defs,
            )
            .unwrap(),
            defs,
        };
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| -> Result<()> {
            let mut docs = TypeDocs::new(&emission, [])?;
            // Isolate Recursive registration from supplemental-note traversal.
            render_attribute_type(
                &emission.defs["A"].ty,
                &field_scope("root"),
                &mut docs.hooks,
                &mut docs.registry,
                None,
            )?;
            docs.write_declarations(&mut PageBuilder::new())
        }));
        let error = result
            .expect("recursive collection documentation must not panic")
            .expect_err("recursive collections must have an explicit outcome");
        assert!(error.downcast_ref::<TypeDocError>().is_some(), "{error:#}");
        let mut docs = TypeDocs::new(&emission, []).unwrap();
        let error = docs
            .display(&emission.defs["A"].ty, field_scope("root"))
            .unwrap_err();
        assert!(error.downcast_ref::<TypeDocError>().is_some(), "{error:#}");
    }

    fn conflicting_enum(changed_values: bool) {
        let identity = carina_core::schema::enum_identity("Mode", Some("aws.test.Round1"));
        let first = AttributeType::enum_(
            identity.clone(),
            Some(vec!["a".into()]),
            vec![("a".into(), "alpha".into())],
            None,
            None,
        );
        let wire = if changed_values { "b" } else { "a" };
        let second = AttributeType::enum_(
            identity,
            Some(vec![wire.into()]),
            vec![(wire.into(), "beta".into())],
            None,
            None,
        );
        let emission = ResourceEmission {
            rust: String::new(),
            attributes: BTreeMap::new(),
            defs: BTreeMap::new(),
            witness: carina_core::schema::ResourceSchema::new("test.Registry")
                .validate()
                .unwrap(),
        };
        // Exercise the documentation registry independently of core admission.
        let mut types = TypeDocs::new(&emission, []).unwrap();
        types
            .hooks
            .index(&first, None, NodeId::Attribute("first".into()));
        types
            .hooks
            .index(&second, None, NodeId::Attribute("second".into()));
        types.roots = vec![
            (&first, field_scope("first")),
            (&second, field_scope("second")),
        ];
        let result = types.prepare_usages();
        let error = result.expect_err("same-identity conflicting enums must never collapse");
        assert!(
            error.to_string().contains("conflicting enum declaration"),
            "{error:#}"
        );
    }

    #[test]
    fn enum_values_conflict_is_rejected_before_registry_reuse() {
        conflicting_enum(true);
    }

    #[test]
    fn enum_alias_conflict_is_rejected_before_registry_reuse() {
        conflicting_enum(false);
    }
}
