//! Names and API constraints are independent. No entry accepts an AttributeType
//! or arbitrary Rust type expression. Every refinement is checked against CFN.
use super::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct TypeOverride {
    pub(super) identity: Option<StringIdentity>,
    pub(super) narrowing: Option<Narrowing>,
    to_dsl: Option<type_emission::DslTransformSpec>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct Narrowing {
    pub(super) kind: NarrowingKind,
    pub(super) evidence: AwsEvidence,
}

/// A reviewed source always includes a URL and a verbatim quotation.
/// Private fields and the checked constructor prevent empty evidence.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub(super) struct DocumentationQuote {
    url: &'static str,
    quote: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub(super) struct Documentation {
    source: DocumentationQuote,
    additional_sources: Vec<DocumentationQuote>,
    checked_on: CheckedOn,
}

/// An observation is a source in its own right, never an annotation that can
/// make a documentation quote claim something the documentation does not say.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub(super) enum AwsEvidence {
    Documentation(Documentation),
    Observation(Observation),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
pub(super) struct CheckedOn {
    pub(super) year: u16,
    pub(super) month: u8,
    pub(super) day: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
pub(super) enum VerifiedDirection {
    Accepted,
    Rejected,
    ReadBackForm,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub(super) struct ObservedInput {
    pub(super) exact_input: &'static serde_json::Value,
    pub(super) direction: VerifiedDirection,
    pub(super) result: &'static serde_json::Value,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub(super) struct Observation {
    cases: Vec<ObservedInput>,
    region: &'static str,
    observed_on: CheckedOn,
    source_record: &'static str,
    method: &'static str,
}

impl AwsEvidence {
    pub(super) fn new(
        url: &'static str,
        quote: &'static str,
        checked_on: CheckedOn,
    ) -> Result<Self> {
        let source = DocumentationQuote::new(url, quote)?;
        checked_on.validate()?;
        Ok(Self::Documentation(Documentation {
            source,
            additional_sources: Vec::new(),
            checked_on,
        }))
    }

    pub(super) fn observed(
        cases: Vec<ObservedInput>,
        region: &'static str,
        observed_on: CheckedOn,
        source_record: &'static str,
        method: &'static str,
    ) -> Result<Self> {
        observed_on.validate()?;
        let recorded_source = (source_record.len() == 40
            && source_record.bytes().all(|b| b.is_ascii_hexdigit()))
            || Regex::new(r"^https://github\.com/[^/\s]+/[^/\s]+/(?:(?:pull|issues)/[1-9][0-9]*(?:#issuecomment-[0-9]+)?|commit/[0-9a-fA-F]{40})$")?
                .is_match(source_record);
        anyhow::ensure!(
            !cases.is_empty()
                && cases
                    .iter()
                    .all(|case| !case.exact_input.is_null() && !case.result.is_null())
                && !region.trim().is_empty()
                && !method.trim().is_empty()
                && recorded_source,
            "observation requires exact input, region, date, verified direction, result and source record"
        );
        Ok(Self::Observation(Observation {
            cases,
            region,
            observed_on,
            source_record,
            method,
        }))
    }

    pub(super) fn checked_on(&self) -> CheckedOn {
        match self {
            Self::Documentation(doc) => doc.checked_on,
            Self::Observation(observation) => observation.observed_on,
        }
    }

    pub(super) fn add_source(&mut self, url: &'static str, quote: &'static str) -> Result<()> {
        let Self::Documentation(doc) = self else {
            anyhow::bail!("documentation sources belong to Documentation evidence")
        };
        doc.additional_sources
            .push(DocumentationQuote::new(url, quote)?);
        Ok(())
    }

    pub(super) fn sources(&self) -> impl Iterator<Item = (&'static str, &'static str)> + '_ {
        let sources = match self {
            Self::Documentation(doc) => Some(doc),
            Self::Observation(_) => None,
        };
        sources
            .into_iter()
            .flat_map(|doc| std::iter::once(&doc.source).chain(&doc.additional_sources))
            .map(|s| (s.url, s.quote))
    }

    fn display_records(&self) -> Vec<(&'static str, String)> {
        match self {
            Self::Documentation(_) => self
                .sources()
                .map(|(url, quote)| (url, quote.to_string()))
                .collect(),
            Self::Observation(observation) => observation
                .cases
                .iter()
                .map(|case| {
                    (
                        observation.source_record,
                        format!(
                            "{}; {}; {:?}; exact input: {}; result: {}",
                            observation.region,
                            observation.method,
                            case.direction,
                            case.exact_input,
                            case.result
                        ),
                    )
                })
                .collect(),
        }
    }

    /// Synthetic containment fixtures do not make a claim about real resources.
    #[cfg(test)]
    pub(super) fn fixture() -> Self {
        Self::new(
            "https://docs.aws.amazon.com/IAM/latest/APIReference/API_CreateRole.html#API_CreateRole_RequestParameters",
            "Length Constraints: Minimum length of 1. Maximum length of 64.",
            CheckedOn { year: 2026, month: 11, day: 7 },
        ).unwrap()
    }
}

impl CheckedOn {
    fn validate(self) -> Result<()> {
        let leap = self.year.is_multiple_of(4)
            && (!self.year.is_multiple_of(100) || self.year.is_multiple_of(400));
        let days = match self.month {
            2 if leap => 29,
            2 => 28,
            4 | 6 | 9 | 11 => 30,
            1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
            _ => 0,
        };
        anyhow::ensure!(
            self.year >= 2000 && self.day > 0 && self.day <= days,
            "invalid evidence checked date"
        );
        Ok(())
    }
}

impl DocumentationQuote {
    fn new(url: &'static str, quote: &'static str) -> Result<Self> {
        anyhow::ensure!(
            url.starts_with("https://docs.aws.amazon.com/")
                && url.len() > "https://docs.aws.amazon.com/".len()
                && !url.chars().any(char::is_whitespace),
            "evidence requires an AWS documentation URL"
        );
        anyhow::ensure!(
            !quote.trim().is_empty(),
            "evidence requires a verbatim quote"
        );
        Ok(Self { url, quote })
    }
}

/// A registration candidate is not a Narrowing. Only a reviewed source can
/// turn this into the evidence-bearing type consumed by the emitters.
pub(super) struct NarrowingSpec {
    pub(super) identity: Option<StringIdentity>,
    pub(super) kind: NarrowingKind,
}

impl NarrowingSpec {
    pub(super) fn named(identity: Option<StringIdentity>, kind: NarrowingKind) -> Self {
        Self { identity, kind }
    }
}

impl From<NarrowingKind> for NarrowingSpec {
    fn from(kind: NarrowingKind) -> Self {
        Self::named(None, kind)
    }
}

/// A closed vocabulary of same-kind refinements. Primitive expressions can
/// describe only typed scalar constraints or a reviewed object grammar.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum NarrowingKind {
    Primitive {
        expression: ConstraintSpec,
        block_name: Option<&'static str>,
        default: Option<AttributeTypeOverrideDefault>,
    },
    NamedType {
        name: &'static str,
        expression: ConstraintSpec,
    },
    ArrayItemType {
        expression: ConstraintSpec,
        block_name: Option<&'static str>,
        default: Option<AttributeTypeOverrideDefault>,
        min_items: Option<i64>,
        max_items: Option<i64>,
    },
    Array {
        items: Option<Box<NarrowingKind>>,
        min_items: Option<i64>,
        max_items: Option<i64>,
    },
    MapKeyLength {
        min_length: Option<u64>,
        max_length: Option<u64>,
    },
    Enum(Vec<&'static str>),
    EnumUnordered(Vec<&'static str>),
    IntRange(i64, i64),
    IntEnum(Vec<i64>),
}

#[cfg(test)]
impl From<NarrowingKind> for TypeOverride {
    fn from(kind: NarrowingKind) -> Self {
        Self::identity(None).with_narrowing(kind, AwsEvidence::fixture())
    }
}

impl NarrowingKind {
    pub(super) fn with_array_bounds(self, min_items: Option<i64>, max_items: Option<i64>) -> Self {
        Self::Array {
            items: Some(Box::new(self)),
            min_items,
            max_items,
        }
    }

    pub(super) fn item_override(&self) -> &Self {
        match self {
            Self::Array {
                items: Some(items), ..
            } => items.item_override(),
            _ => self,
        }
    }
}

impl TypeOverride {
    pub(super) fn identity(identity: Option<StringIdentity>) -> Self {
        Self {
            identity,
            narrowing: None,
            to_dsl: None,
        }
    }

    pub(super) fn with_narrowing(mut self, kind: NarrowingKind, evidence: AwsEvidence) -> Self {
        self.narrowing = Some(Narrowing { kind, evidence });
        self
    }

    pub(super) fn strip_suffix(suffix: &'static str) -> Self {
        assert_eq!(
            suffix, ".",
            "only the reviewed Route53 transform is registered"
        );
        Self {
            identity: None,
            narrowing: None,
            to_dsl: Some(type_emission::DslTransformSpec::StripSuffix(suffix)),
        }
    }

    pub(super) fn to_dsl(&self) -> Option<type_emission::DslTransformSpec> {
        self.to_dsl
    }

    pub(super) fn kind(&self) -> Option<&NarrowingKind> {
        self.narrowing.as_ref().map(|n| &n.kind)
    }

    pub(super) fn item_override(&self) -> Option<&NarrowingKind> {
        self.kind().map(NarrowingKind::item_override)
    }

    pub(super) fn string_binding(&self) -> Option<Option<StringIdentity>> {
        if self.identity.is_some() {
            return Some(self.identity);
        }
        match self.item_override() {
            None if self.to_dsl.is_none() => Some(self.identity),
            Some(
                NarrowingKind::Primitive { expression, .. }
                | NarrowingKind::ArrayItemType { expression, .. },
            ) if expression.is_string() => Some(self.identity),
            _ => None,
        }
    }

    pub(super) fn enum_values(&self) -> Option<&[&'static str]> {
        if self.identity.is_some() {
            None
        } else {
            self.raw_enum_values()
        }
    }

    /// String refinements and collection bounds retain the source leaf type.
    /// Array wrappers are still consumed separately by the collection writer.
    pub(super) fn replaces_leaf_type(&self) -> bool {
        match self.item_override() {
            None => false,
            Some(NarrowingKind::Primitive { .. } | NarrowingKind::ArrayItemType { .. }) => {
                self.string_binding().is_none()
            }
            Some(NarrowingKind::NamedType { .. }) => true,
            Some(NarrowingKind::Enum(_) | NarrowingKind::EnumUnordered(_)) => {
                self.identity.is_none()
            }
            Some(
                NarrowingKind::Array { .. }
                | NarrowingKind::MapKeyLength { .. }
                | NarrowingKind::IntRange(_, _)
                | NarrowingKind::IntEnum(_),
            ) => false,
        }
    }

    pub(super) fn requires_array_source(&self) -> bool {
        matches!(
            self.kind(),
            Some(NarrowingKind::Array { .. } | NarrowingKind::ArrayItemType { .. })
        )
    }

    pub(super) fn attribute_constraint(&self) -> Option<&ConstraintSpec> {
        match self.kind()? {
            NarrowingKind::Primitive { expression, .. }
            | NarrowingKind::NamedType { expression, .. } => Some(expression),
            _ => None,
        }
    }

    pub(super) fn array_item_constraint(&self) -> Option<&ConstraintSpec> {
        match self.item_override()? {
            NarrowingKind::ArrayItemType { expression, .. } => Some(expression),
            _ => None,
        }
    }

    pub(super) fn expression(&self) -> Option<&ConstraintSpec> {
        match self.item_override()? {
            NarrowingKind::Primitive { expression, .. }
            | NarrowingKind::NamedType { expression, .. }
            | NarrowingKind::ArrayItemType { expression, .. } => Some(expression),
            _ => None,
        }
    }

    pub(super) fn default(&self) -> Option<AttributeTypeOverrideDefault> {
        match self.item_override()? {
            NarrowingKind::Primitive { default, .. }
            | NarrowingKind::ArrayItemType { default, .. } => *default,
            _ => None,
        }
    }

    pub(super) fn block_name(&self) -> Option<&'static str> {
        match self.item_override()? {
            NarrowingKind::Primitive { block_name, .. }
            | NarrowingKind::ArrayItemType { block_name, .. } => *block_name,
            _ => None,
        }
    }

    pub(super) fn owner_name(&self) -> Option<&'static str> {
        Some(match self.kind()? {
            NarrowingKind::Primitive { .. } => "resource_type_overrides Primitive narrowing",
            NarrowingKind::NamedType { .. } => "resource_type_overrides NamedType narrowing",
            NarrowingKind::ArrayItemType { .. } => {
                "resource_type_overrides ArrayItemType narrowing"
            }
            NarrowingKind::Array { .. } => "resource_type_overrides Array narrowing",
            NarrowingKind::MapKeyLength { .. } => "resource_type_overrides MapKeyLength narrowing",
            _ => "resource_type_overrides narrowing",
        })
    }

    pub(super) fn validate_patterns(
        &self,
        schema: &CfnSchema,
        path: CfnPropertyPath<'_>,
    ) -> Result<()> {
        if let Some(expression) = self.expression() {
            for pattern in expression.patterns() {
                carina_core::schema::CompiledPattern::try_new(pattern.as_spec()).with_context(
                    || {
                        format!(
                            "Invalid narrowing string pattern for {}#{}: {}",
                            schema.type_name,
                            path.source_pointer(),
                            pattern.as_str()
                        )
                    },
                )?;
                anyhow::ensure!(
                    matches!(pattern.as_spec().mode(), PatternMatch::Full),
                    "Unapproved narrowing string pattern for {}#{}: {}",
                    schema.type_name,
                    path.source_pointer(),
                    pattern.as_str()
                );
            }
        }
        Ok(())
    }

    pub(super) fn validate_source_shape(
        &self,
        schema: &CfnSchema,
        path: CfnPropertyPath<'_>,
        property: &SchemaProperty,
    ) -> Result<()> {
        let projection = property.try_query()?;
        self.validate_source_shape_inner(schema, path, property)
            .map_err(|error| {
                projection
                    .emission_error(format!("{}.{path}: {error:#}", schema.type_name))
                    .into()
            })
    }

    fn validate_source_shape_inner(
        &self,
        schema: &CfnSchema,
        path: CfnPropertyPath<'_>,
        property: &SchemaProperty,
    ) -> Result<()> {
        self.validate_patterns(schema, path)?;
        let plan = emission_plan::lower(property.try_query()?)?;
        if self.identity.is_some() {
            let mut leaf = property;
            let mut seen = BTreeSet::new();
            while let Some(items) = leaf.try_query()?.items.as_deref() {
                if let Some(origin) = &items.origin
                    && !seen.insert(origin.node)
                {
                    return Err(leaf
                        .try_query()?
                        .emission_error("recursive items projection has no string identity leaf")
                        .into());
                }
                leaf = items;
            }
            let leaf_plan = emission_plan::lower(leaf.try_query()?)?;
            if leaf_plan.value_type()? != EmissionType::String {
                return Err(leaf
                    .try_query()?
                    .emission_error("string identity requires the CFN string kind")
                    .into());
            }
            if self
                .identity
                .is_some_and(|identity| identity.is_core_builtin())
                && (self.narrowing.is_some() || leaf_plan.literal.is_some())
            {
                return Err(leaf.try_query()?.emission_error("core built-in identity cannot carry an API or literal narrowing (carina#4294)").into());
            }
        }
        match self.kind() {
            None => {}
            Some(
                NarrowingKind::Primitive { expression, .. }
                | NarrowingKind::NamedType { expression, .. },
            ) => {
                expression.emit(property, schema, path, self.identity)?;
            }
            Some(NarrowingKind::ArrayItemType { .. } | NarrowingKind::Array { .. }) => {
                let array = self.array_source(schema, path, property)?;
                let array_plan = emission_plan::lower(array.try_query()?)?;
                let items = array_plan.array()?.items;
                if let Some(expression) = self.expression() {
                    expression.emit(items, schema, path, self.identity)?;
                }
                if let Some(values) = self.raw_enum_values() {
                    emission_plan::lower(items.try_query()?)?.validate_enum_candidates(values)?;
                }
            }
            Some(NarrowingKind::Enum(values) | NarrowingKind::EnumUnordered(values)) => {
                plan.validate_enum_candidates(values)?
            }
            Some(NarrowingKind::IntEnum(values)) => {
                if property.try_query()?.effective_type() != Some("integer") {
                    return Err(property
                        .try_query()?
                        .emission_error("integer enum narrowing must retain the CFN integer kind")
                        .into());
                }
                plan.validate_integer_enum(values)?;
            }
            Some(NarrowingKind::IntRange(min, max)) => {
                plan.numeric_override_range(*min, *max)?;
            }
            Some(NarrowingKind::MapKeyLength {
                min_length,
                max_length,
            }) => {
                let (pattern, _) = property_pattern_map_entry(property, schema)?.ok_or_else(|| property.try_query().unwrap().emission_error("MapKeyLength requires one patternProperties entry and additionalProperties:false on the same map"))?;
                if pattern_map_key_values(pattern).is_some()
                    || min_length
                        .zip(*max_length)
                        .is_some_and(|(min, max)| min > max)
                {
                    return Err(property
                        .try_query()?
                        .emission_error(
                            "MapKeyLength requires a derived string key and nonempty bounds",
                        )
                        .into());
                }
            }
        }
        Ok(())
    }

    pub(super) fn emit_expression(
        &self,
        property: &SchemaProperty,
        schema: &CfnSchema,
        path: CfnPropertyPath<'_>,
    ) -> Result<EmittedType> {
        self.expression()
            .context("missing typed narrowing")?
            .emit_at(
                property,
                schema,
                path,
                self.identity,
                matches!(self.kind(), Some(NarrowingKind::Primitive { .. })),
            )
    }

    // A containing property owns source facts; raw delegation must not attach
    // those facts before the property has selected its final type.
    pub(super) fn emit_expression_unnoted(
        &self,
        property: &SchemaProperty,
        schema: &CfnSchema,
        path: CfnPropertyPath<'_>,
    ) -> Result<EmittedType> {
        self.expression()
            .context("missing typed narrowing")?
            .emit_inner(
                property,
                schema,
                path,
                self.identity,
                matches!(self.kind(), Some(NarrowingKind::Primitive { .. })),
            )
    }

    pub(super) fn string_constraints(
        &self,
        mut constraints: StringConstraints,
    ) -> Result<StringConstraints> {
        if let Some(expression) = self.expression() {
            constraints = expression.intersect_string(constraints)?;
        }
        Ok(constraints)
    }

    pub(super) fn array_source<'a>(
        &self,
        schema: &'a CfnSchema,
        path: CfnPropertyPath<'_>,
        property: &'a SchemaProperty,
    ) -> Result<Cow<'a, SchemaProperty>> {
        let bounds_only = matches!(self.kind(), Some(NarrowingKind::Array { items: None, .. }));
        let source = if bounds_only {
            resolved_array_property(property, schema, &mut HashSet::new())?
        } else {
            array_item_override_source(property, schema)?
        };
        let mut array = source.with_context(|| format!("{}.{}: ArrayItemType/Array requires a single CFN array at this exact path (directly or through $ref), not a non-array or union",schema.type_name,path))?;
        let (min_items, max_items) = match self.kind() {
            Some(NarrowingKind::Array {
                items,
                min_items,
                max_items,
            }) => {
                anyhow::ensure!(
                    matches!(
                        items.as_deref(),
                        None | Some(
                            NarrowingKind::ArrayItemType { .. }
                                | NarrowingKind::Enum(_)
                                | NarrowingKind::EnumUnordered(_)
                        )
                    ),
                    "{}.{}: Array requires no item override, or an ArrayItemType, Enum, or EnumUnordered item narrowing",
                    schema.type_name,
                    path
                );
                (*min_items, *max_items)
            }
            Some(NarrowingKind::ArrayItemType {
                min_items,
                max_items,
                ..
            }) => (*min_items, *max_items),
            _ => (None, None),
        };
        for (name, bound, cfn_bound, lower) in [
            ("minItems", min_items, array.try_query()?.min_items, true),
            ("maxItems", max_items, array.try_query()?.max_items, false),
        ] {
            if let Some(bound) = bound {
                anyhow::ensure!(
                    bound >= 0,
                    "{}.{}: Array {name} must be nonnegative",
                    schema.type_name,
                    path
                );
                anyhow::ensure!(
                    cfn_bound.is_none_or(|cfn| if lower { bound >= cfn } else { bound <= cfn }),
                    "{}.{}: Array {name} override {bound} would loosen CFN bound {cfn_bound:?}",
                    schema.type_name,
                    path
                );
            }
        }
        let min = min_items.or(array.try_query()?.min_items);
        let max = max_items.or(array.try_query()?.max_items);
        anyhow::ensure!(
            min.zip(max).is_none_or(|(min, max)| min <= max),
            "{}.{}: Array merged minItems {min:?} exceeds maxItems {max:?}",
            schema.type_name,
            path
        );
        if min != array.try_query()?.min_items || max != array.try_query()?.max_items {
            let target = array.to_mut();
            let mut p = target.try_query()?.clone();
            p.min_items = min;
            p.max_items = max;
            *target = schema_projection::materialize(p);
        }
        Ok(array)
    }
}

/// Each raw CFN field appears with the evidence carried by its narrowing.
pub(super) fn write_narrowing_notes(
    page: &mut PageBuilder<MarkdownTarget>,
    schema: &CfnSchema,
) -> Result<()> {
    let mut entries = resource_type_overrides()
        .for_resource(&schema.type_name)
        .filter_map(|(path, entry)| {
            entry
                .narrowing
                .as_ref()
                .map(|n| (path.source_pointer(), n.evidence.clone()))
        })
        .collect::<BTreeMap<_, _>>();
    entries.extend(global_narrowings());
    if entries.is_empty() {
        return Ok(());
    }
    page.heading(Heading::plain(2, "AWS API narrowings"))?;
    page.push_str("These fields are narrower than or equal to CFN. Input rules combine documented AWS domains, recorded live observations, and Carina canonical spellings for stable state comparison. A canonical DSL spelling does not claim AWS rejects other spellings. Attaching a type name adds no structural constraints. Field names below use CFN spelling.\n\n");
    page.push_str(
        "| CFN field | Relation to CFN | Evidence URL | Verbatim quote / exact observation | Checked / observed |\n| --- | --- | --- | --- | --- |\n",
    );
    for (path, evidence) in entries {
        let date = evidence.checked_on();
        for (url, quote) in evidence.display_records() {
            let quote = quote
                .lines()
                .map(|paragraph| {
                    // Code spans preserve literal regex backslashes, brackets,
                    // asterisks and HTML. A longer delimiter also preserves any
                    // literal backticks in an AWS quotation. GFM tables require
                    // escaping pipes even inside their code spans.
                    let longest = paragraph
                        .split(|c| c != '`')
                        .map(str::len)
                        .max()
                        .unwrap_or(0);
                    let delimiter = "`".repeat(longest + 1);
                    format!("{delimiter} {} {delimiter}", paragraph.replace('|', "\\|"))
                })
                .collect::<Vec<_>>()
                .join("<br>");
            page.push_str(&format!(
                "| `{path}` | Subset: CFN ∩ reviewed input domain | [Source record]({url}) | {quote} | {:04}-{:02}-{:02} |\n",
                date.year, date.month, date.day,
            ));
        }
    }
    page.push_str("\n");
    Ok(())
}

/// Historical name-based candidates are not evidence. Emission requires a
/// separate field-specific documentation review before consuming a value.
pub(super) struct GlobalOverrideCandidates<T> {
    entries: HashMap<&'static str, T>,
}
impl<T> GlobalOverrideCandidates<T> {
    pub(super) fn new() -> Self {
        Self {
            entries: HashMap::new(),
        }
    }
    pub(super) fn insert(&mut self, key: &'static str, value: T) {
        self.entries.insert(key, value);
    }
    pub(super) fn get(&self, key: &str) -> Option<&T> {
        self.entries.get(key)
    }
    pub(super) fn contains_key(&self, key: &str) -> bool {
        self.entries.contains_key(key)
    }
    #[cfg(test)]
    pub(super) fn keys(&self) -> impl Iterator<Item = &&'static str> {
        self.entries.keys()
    }
}

thread_local! {
    static GLOBAL_NARROWINGS: RefCell<BTreeMap<String, AwsEvidence>> = const { RefCell::new(BTreeMap::new()) };
}
pub(super) fn record_global_narrowing(pointer: &str, evidence: &AwsEvidence) {
    GLOBAL_NARROWINGS.with(|rows| {
        rows.borrow_mut()
            .insert(pointer.to_string(), evidence.clone());
    });
}
pub(super) fn reset_global_narrowings() {
    GLOBAL_NARROWINGS.with(|rows| rows.borrow_mut().clear());
}
pub(super) fn global_narrowings() -> BTreeMap<String, AwsEvidence> {
    GLOBAL_NARROWINGS.with(|rows| rows.borrow().clone())
}

/// Classification tables supply candidates, just like resource overrides.
/// Every emitted restriction has reviewed evidence and crosses the same typed
/// source-shape/containment boundary before either renderer can consume it.
pub(super) fn bare_object_narrowing(
    schema: &CfnSchema,
    path: CfnPropertyPath<'_>,
    property: &SchemaProperty,
) -> Result<EmittedType> {
    static ENTRIES: LazyLock<HashMap<(&'static str, CfnPropertyPath<'static>), TypeOverride>> =
        LazyLock::new(|| {
            let empty = EMPTY_OBJECT_PROPERTIES
                .iter()
                .chain(TEST_EMPTY_OBJECT_PROPERTIES)
                .map(|entry| {
                    (
                        entry,
                        ConstraintSpec::empty_object(entry.property_path.property),
                    )
                });
            let maps = STRING_MAP_PROPERTIES
                .iter()
                .map(|entry| (entry, ConstraintSpec::string_map()));
            empty
                .chain(maps)
                .map(|(entry, expression)| {
                    let kind = NarrowingKind::Primitive {
                        expression,
                        block_name: None,
                        default: None,
                    };
                    let narrowing = narrowing_evidence::reviewed(
                        entry.cfn_type,
                        &entry.property_path.to_string(),
                        kind,
                    )
                    .expect("bare object restrictions require reviewed AWS evidence");
                    (
                        (entry.cfn_type, entry.property_path),
                        TypeOverride::identity(None)
                            .with_narrowing(narrowing.kind, narrowing.evidence),
                    )
                })
                .collect()
        });
    let projection = property.try_query()?;
    let entry = ENTRIES
        .get(&(schema.type_name.as_str(), path))
        .ok_or_else(|| {
            projection.emission_error("bare object restriction has no evidenced Narrowing")
        })?;
    entry.validate_source_shape(schema, path, property)?;
    record_global_narrowing(
        &path.source_pointer(),
        &entry
            .narrowing
            .as_ref()
            .expect("evidenced restriction")
            .evidence,
    );
    entry.emit_expression_unnoted(property, schema, path)
}

impl TypeOverride {
    fn raw_enum_values(&self) -> Option<&[&'static str]> {
        match self.item_override()? {
            NarrowingKind::Enum(values) | NarrowingKind::EnumUnordered(values) => Some(values),
            _ => None,
        }
    }

    pub(super) fn named_literal_constraints(
        &self,
        property: &SchemaProperty,
        schema: &CfnSchema,
        path: CfnPropertyPath<'_>,
    ) -> Result<Option<StringConstraints>> {
        if self.identity.is_none() {
            return Ok(None);
        }
        let p = property.try_query()?;
        let plan = emission_plan::lower(p)?;
        let mut values = if let Some(values) = self.raw_enum_values() {
            plan.validate_enum_candidates(values)?;
            values
                .iter()
                .map(|value| (*value).to_string())
                .collect::<Vec<_>>()
        } else if let Some(literal) = &plan.literal {
            if literal.value_type != EmissionType::String {
                return Ok(None);
            }
            literal
                .values
                .iter()
                .map(|value| {
                    value
                        .as_str()
                        .expect("validated string literal")
                        .to_string()
                })
                .collect()
        } else {
            return Ok(None);
        };
        let string = emission_plan::StringPlan {
            source_pointer: &p.source_pointer,
            description: p.description.as_deref(),
            format: p.format.as_deref(),
            pattern: p.pattern.as_deref(),
            min_length: p.min_length,
            max_length: p.max_length,
        };
        let mut constraints =
            project_string_constraints(&string, &schema.type_name, path, &p.source_pointer)?;
        if let Some(expression) = self.expression() {
            values = expression.filter_string_values(values)?;
            // Every remaining enum member has already satisfied CFN's pattern.
            // Intersect bounds without needing a general regex conjunction.
            constraints.pattern = None;
            constraints = expression.intersect_string(constraints)?;
        }
        if values.is_empty() {
            return Err(p
                .emission_error("empty named string enum narrowing intersection")
                .into());
        }
        constraints.pattern = Some(ApprovedPattern::whole(&format!(
            r"^(?:{})\z",
            values
                .iter()
                .map(|value| regex::escape(value))
                .collect::<Vec<_>>()
                .join("|")
        )));
        Ok(Some(constraints))
    }
}
