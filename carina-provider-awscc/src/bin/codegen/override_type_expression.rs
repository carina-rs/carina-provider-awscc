//! Typed API refinements. There is deliberately no raw-expression constructor.
use super::*;
use carina_core::schema::{AttributeType, IntIdentityParts, RawShape, StringIdentityParts};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum PatternLiteral {
    Quoted,
    Raw,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum DocumentKind {
    IamPolicyDocument,
    S3AnalyticsDestination,
    SnsSubscriptionRedrivePolicy,
    SnsSubscriptionDeliveryPolicy,
    SnsSubscriptionReplayPolicy,
    SnsTopicArchivePolicy,
    LogsFieldIndexPolicy,
    LogsDataProtectionPolicy,
}

impl DocumentKind {
    pub(super) fn constructor(self) -> &'static str {
        match self {
            Self::IamPolicyDocument => "carina_aws_types::iam_policy_document()",
            Self::S3AnalyticsDestination => {
                "crate::schemas::awscc_types::s3_analytics_destination()"
            }
            Self::SnsSubscriptionRedrivePolicy => {
                "crate::schemas::awscc_types::sns_subscription_redrive_policy()"
            }
            Self::SnsSubscriptionDeliveryPolicy => {
                "crate::schemas::awscc_types::sns_subscription_delivery_policy()"
            }
            Self::SnsSubscriptionReplayPolicy => {
                "crate::schemas::awscc_types::sns_subscription_replay_policy()"
            }
            Self::SnsTopicArchivePolicy => {
                "crate::schemas::awscc_types::sns_topic_archive_policy()"
            }
            Self::LogsFieldIndexPolicy => "crate::schemas::awscc_types::logs_field_index_policy()",
            Self::LogsDataProtectionPolicy => {
                "crate::schemas::awscc_types::logs_data_protection_policy()"
            }
        }
    }
    fn declaration(self) -> AttributeType {
        use carina_provider_awscc::schemas::awscc_types::*;
        match self {
            Self::IamPolicyDocument => iam_policy_document(),
            Self::S3AnalyticsDestination => s3_analytics_destination(),
            Self::SnsSubscriptionRedrivePolicy => sns_subscription_redrive_policy(),
            Self::SnsSubscriptionDeliveryPolicy => sns_subscription_delivery_policy(),
            Self::SnsSubscriptionReplayPolicy => sns_subscription_replay_policy(),
            Self::SnsTopicArchivePolicy => sns_topic_archive_policy(),
            Self::LogsFieldIndexPolicy => logs_field_index_policy(),
            Self::LogsDataProtectionPolicy => logs_data_protection_policy(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum MapKind {
    BudgetCostFilters,
    BudgetPlannedLimits,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum EnumDomain {
    S3NotificationEvent,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Constraint {
    String {
        pattern: Option<ApprovedPattern>,
        length: Option<(Option<u64>, Option<u64>)>,
    },
    #[cfg(test)]
    Boolean,
    IntegerRange(Option<i64>, Option<i64>),
    NumberRange(Option<i64>, Option<i64>),
    Union(Vec<ConstraintSpec>),
    AvailabilityZoneOrAll,
    Document(DocumentKind),
    Map(MapKind),
    EmptyObject,
    StringMap,
    Enum(EnumDomain),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct ConstraintSpec {
    code: String,
    patterns: Vec<ApprovedPattern>,
    constraint: Constraint,
}

impl ConstraintSpec {
    pub(super) fn availability_zone_or_all() -> Self {
        let pattern = ApprovedPattern::whole(r"^all\z");
        let zone = string_identity::emit(
            Some(StringIdentity::AvailabilityZoneZoneName),
            &StringConstraints {
                pattern: None,
                min_length: None,
                max_length: None,
            },
        );
        let all = string_identity::emit(
            None,
            &StringConstraints {
                pattern: Some(pattern.clone()),
                min_length: None,
                max_length: None,
            },
        );
        Self {
            code: format!("AttributeType::union(vec![{zone}, {all}])"),
            patterns: vec![pattern],
            constraint: Constraint::AvailabilityZoneOrAll,
        }
    }
    pub(super) fn string_length(&self) -> Option<(Option<u64>, Option<u64>)> {
        let Constraint::String { length, .. } = self.constraint else {
            panic!("reviewed string amendment requires a string constraint")
        };
        length
    }
    pub(super) fn without_string_pattern(&self) -> Self {
        let Constraint::String { length, .. } = self.constraint else {
            panic!("reviewed string amendment requires a string constraint")
        };
        Self::string(None, length, PatternLiteral::Quoted)
    }

    pub(super) fn with_minimum(&self, minimum: i64) -> Self {
        match self.constraint {
            Constraint::IntegerRange(_, max) => Self::int_range(Some(minimum), max),
            Constraint::NumberRange(_, max) => Self::number_range(Some(minimum), max),
            _ => panic!("reviewed numeric amendment requires a numeric constraint"),
        }
    }

    pub(super) fn string(
        pattern: Option<ApprovedPattern>,
        length: Option<(Option<u64>, Option<u64>)>,
        literal: PatternLiteral,
    ) -> Self {
        let emitted = pattern.as_ref().map_or_else(
            || "None".to_string(),
            |pattern| format!("Some({})", emit_pattern_spec(pattern, literal)),
        );
        let (min, max) = length.unwrap_or((None, None));
        let code_length = emit_length_option(min, max);
        Self {
            code: format!("AttributeType::refined_string(None, {emitted}, {code_length}, None)"),
            patterns: pattern.clone().into_iter().collect(),
            constraint: Constraint::String { pattern, length },
        }
    }
    #[cfg(test)]
    pub(super) fn boolean() -> Self {
        Self {
            code: "AttributeType::bool()".into(),
            patterns: vec![],
            constraint: Constraint::Boolean,
        }
    }
    pub(super) fn int_range(min: Option<i64>, max: Option<i64>) -> Self {
        Self {
            code: refined_int_type("None", &emit_int_range_option(min, max)),
            patterns: vec![],
            constraint: Constraint::IntegerRange(min, max),
        }
    }
    pub(super) fn number_range(min: Option<i64>, max: Option<i64>) -> Self {
        Self {
            code: refined_float_type("None", Some((min, max))),
            patterns: vec![],
            constraint: Constraint::NumberRange(min, max),
        }
    }
    pub(super) fn union(members: Vec<Self>) -> Self {
        Self {
            code: format!(
                "AttributeType::union(vec![{}])",
                members
                    .iter()
                    .map(Self::as_str)
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            patterns: members.iter().flat_map(|m| m.patterns.clone()).collect(),
            constraint: Constraint::Union(members),
        }
    }
    pub(super) fn document(kind: DocumentKind) -> Self {
        Self {
            code: kind.constructor().into(),
            patterns: vec![],
            constraint: Constraint::Document(kind),
        }
    }
    pub(super) fn map(kind: MapKind) -> Self {
        let code = match kind {
            MapKind::BudgetCostFilters => {
                "AttributeType::map(AttributeType::list(AttributeType::string()))"
            }
            MapKind::BudgetPlannedLimits => {
                "AttributeType::map(AttributeType::ref_(\"Spend\".to_string()))"
            }
        };
        Self {
            code: code.into(),
            patterns: vec![],
            constraint: Constraint::Map(kind),
        }
    }
    pub(super) fn empty_object(name: &str) -> Self {
        Self {
            code: format!(
                "AttributeType::struct_({}.to_string(), vec![])",
                rust_lit(&name.to_pascal_case())
            ),
            patterns: vec![],
            constraint: Constraint::EmptyObject,
        }
    }
    pub(super) fn string_map() -> Self {
        Self {
            code: "AttributeType::map(AttributeType::string())".into(),
            patterns: vec![],
            constraint: Constraint::StringMap,
        }
    }
    pub(super) fn enum_domain(domain: EnumDomain) -> Self {
        Self {
            code: "crate::schemas::awscc_types::s3_notification_event()".into(),
            patterns: vec![],
            constraint: Constraint::Enum(domain),
        }
    }
    pub(super) fn is_string(&self) -> bool {
        matches!(self.constraint, Constraint::String { .. })
    }
    pub(super) fn as_str(&self) -> &str {
        &self.code
    }
    pub(super) fn patterns(&self) -> &[ApprovedPattern] {
        &self.patterns
    }
    pub(super) fn normalized_display(&self, display: &'static str) -> Cow<'static, str> {
        if let (Some(start), [pattern]) = (display.find("pattern: "), self.patterns.as_slice()) {
            let start = start + "pattern: ".len();
            let end = display.rfind(')').expect("string refinement display");
            return Cow::Owned(format!(
                "{}{}{}",
                &display[..start],
                pattern.as_str(),
                &display[end..]
            ));
        }
        Cow::Borrowed(display)
    }

    pub(super) fn filter_string_values(&self, values: Vec<String>) -> Result<Vec<String>> {
        let Constraint::String { pattern, length } = &self.constraint else {
            anyhow::bail!("string enum refinement requires string constraints");
        };
        let pattern = pattern
            .as_ref()
            .map(|pattern| carina_core::schema::CompiledPattern::try_new(pattern.as_spec()))
            .transpose()?;
        let (min, max) = length.unwrap_or((None, None));
        Ok(values
            .into_iter()
            .filter(|value| {
                let size = value.chars().count() as u64;
                min.is_none_or(|min| size >= min)
                    && max.is_none_or(|max| size <= max)
                    && pattern
                        .as_ref()
                        .is_none_or(|pattern| pattern.is_match(value))
            })
            .collect())
    }

    pub(super) fn intersect_string(&self, mut cfn: StringConstraints) -> Result<StringConstraints> {
        let Constraint::String { pattern, length } = &self.constraint else {
            anyhow::bail!("string narrowing requires string constraints");
        };
        if let Some((min, max)) = length {
            cfn.min_length = match (cfn.min_length, *min) {
                (Some(a), Some(b)) => Some(a.max(b)),
                (a, b) => a.or(b),
            };
            cfn.max_length = match (cfn.max_length, *max) {
                (Some(a), Some(b)) => Some(a.min(b)),
                (a, b) => a.or(b),
            };
        }
        anyhow::ensure!(
            cfn.min_length
                .zip(cfn.max_length)
                .is_none_or(|(min, max)| min <= max),
            "empty string length narrowing intersection"
        );
        if let Some(pattern) = pattern {
            if let Some(source) = &cfn.pattern {
                if bounded_pattern_subset(
                    pattern.as_str(),
                    source.as_str(),
                    cfn.min_length,
                    cfn.max_length,
                )? {
                    cfn.pattern = Some(pattern.clone());
                } else if !bounded_pattern_subset(
                    source.as_str(),
                    pattern.as_str(),
                    cfn.min_length,
                    cfn.max_length,
                )? {
                    if let Some(intersection) =
                        reviewed_pattern_intersection(source.as_str(), pattern.as_str())
                    {
                        anyhow::ensure!(
                            pattern_subset(intersection, source.as_str())?
                                && pattern_subset(intersection, pattern.as_str())?,
                            "reviewed intersection is not contained in both patterns"
                        );
                        cfn.pattern = Some(ApprovedPattern::whole(intersection));
                    } else {
                        anyhow::bail!(
                            "cannot prove string pattern narrowing is contained in CFN; a reviewed pattern intersection is required: {:?} AND {:?}",
                            source.as_str(),
                            pattern.as_str()
                        );
                    }
                }
            } else {
                cfn.pattern = Some(pattern.clone());
            }
        }
        Ok(cfn)
    }

    pub(super) fn emit(
        &self,
        property: &SchemaProperty,
        schema: &CfnSchema,
        path: CfnPropertyPath<'_>,
        identity: Option<StringIdentity>,
    ) -> Result<String> {
        let p = property.try_query()?;
        let plan = emission_plan::lower(p)?;
        let fail = |reason: &str| p.emission_error(reason);
        match &self.constraint {
            Constraint::String { .. } => {
                let string = plan.string()?;
                let cfn = project_string_constraints(
                    string,
                    &schema.type_name,
                    path,
                    string.source_pointer,
                )?;
                let constraints = self
                    .intersect_string(cfn)
                    .map_err(|e| fail(&e.to_string()))?;
                Ok(string_identity::emit(identity, &constraints))
            }
            #[cfg(test)]
            Constraint::Boolean => {
                if plan.value_type()? != EmissionType::Boolean {
                    return Err(fail("boolean narrowing must have the same kind as CFN").into());
                }
                Ok(self.code.clone())
            }
            Constraint::IntegerRange(min, max) | Constraint::NumberRange(min, max) => {
                let expected = plan.value_type()?;
                if !matches!(expected, EmissionType::Integer | EmissionType::Number) {
                    return Err(fail("numeric narrowing requires the CFN numeric kind").into());
                }
                let (cmin, cmax) = plan.numeric()?.range.unwrap_or((None, None));
                let min = match (cmin, *min) {
                    (Some(a), Some(b)) => Some(a.max(b)),
                    (a, b) => a.or(b),
                };
                let max = match (cmax, *max) {
                    (Some(a), Some(b)) => Some(a.min(b)),
                    (a, b) => a.or(b),
                };
                if min.zip(max).is_some_and(|(a, b)| a > b) {
                    return Err(fail("empty numeric narrowing intersection").into());
                }
                plan.validate_numeric_range(min, max)?;
                Ok(
                    if min.is_none() && max.is_none() && plan.numeric()?.format.is_none() {
                        if expected == EmissionType::Integer {
                            "AttributeType::int()".into()
                        } else {
                            "AttributeType::float()".into()
                        }
                    } else if expected == EmissionType::Integer {
                        refined_int_type("None", &emit_int_range_option(min, max))
                    } else {
                        refined_float_type("None", Some((min, max)))
                    },
                )
            }
            Constraint::AvailabilityZoneOrAll => {
                let zone = Self::string(None, None, PatternLiteral::Raw).emit(
                    property,
                    schema,
                    path,
                    Some(StringIdentity::AvailabilityZoneZoneName),
                )?;
                let all = Self::string(
                    Some(ApprovedPattern::whole(r"^all\z")),
                    None,
                    PatternLiteral::Raw,
                )
                .emit(property, schema, path, None)?;
                Ok(format!("AttributeType::union(vec![{zone}, {all}])"))
            }
            Constraint::Union(members) => {
                if members.is_empty() {
                    return Err(fail("empty narrowing union").into());
                }
                let expressions = members
                    .iter()
                    .map(|m| m.emit(property, schema, path, identity))
                    .collect::<Result<Vec<_>>>()?;
                Ok(format!(
                    "AttributeType::union(vec![{}])",
                    expressions.join(", ")
                ))
            }
            Constraint::Document(kind) => {
                validate_document(p, &kind.declaration(), schema)?;
                Ok(self.code.clone())
            }
            Constraint::Map(_) | Constraint::EmptyObject | Constraint::StringMap => {
                require_unconstrained_object_form(p)?;
                Ok(self.code.clone())
            }
            Constraint::Enum(EnumDomain::S3NotificationEvent) => {
                let declaration =
                    carina_provider_awscc::schemas::awscc_types::s3_notification_event();
                let RawShape::Enum {
                    values: Some(values),
                    ..
                } = declaration.raw_shape()
                else {
                    unreachable!("closed string enum")
                };
                plan.validate_scalar_enum_candidates(
                    &values.iter().map(String::as_str).collect::<Vec<_>>(),
                )?;
                Ok(self.code.clone())
            }
        }
    }
}

impl std::fmt::Display for ConstraintSpec {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

fn object_form(p: &Projection) -> Result<()> {
    if !p.types.contains(&schema_layer::JsonType::Object) {
        return Err(p
            .emission_error(
                "document narrowing requires an object form allowed by the CFN document shape",
            )
            .into());
    }
    Ok(())
}

fn require_unconstrained_object_form(p: &Projection) -> Result<()> {
    object_form(p)?;
    if p.properties.as_ref().is_some_and(|p| !p.is_empty())
        || p.pattern_properties.is_some()
        || p.additional_schema.is_some()
        || !p.required.is_empty()
        || p.min_properties.is_some()
        || p.max_properties.is_some()
        || p.additional_properties == Some(false)
        || p.enum_values.is_some()
        || p.const_value.is_some()
        || !p.field_groups.groups.is_empty()
        || !p.dependencies.is_empty()
    {
        return Err(p
            .emission_error("object map narrowing cannot discard CFN object constraints")
            .into());
    }
    Ok(())
}

/// This is specific to reviewed JSON object grammars, never an arbitrary
/// constructor implication check. Scalar fields are checked recursively.
fn validate_document(
    p: &Projection,
    declaration: &AttributeType,
    schema: &CfnSchema,
) -> Result<()> {
    object_form(p)?;
    let RawShape::Struct { fields, .. } = declaration.raw_shape() else {
        return Err(p.emission_error("document grammar must be a struct").into());
    };
    let fields = fields.iter().map(|field| field.parts()).collect::<Vec<_>>();
    for required in &p.required {
        if !fields.iter().any(|f| {
            f.provider_name.unwrap_or(f.name) == required
                && f.declared_input_mode == carina_core::schema::InputMode::Required
        }) {
            return Err(p
                .emission_error(format!(
                    "document narrowing drops required field {required}"
                ))
                .into());
        }
    }
    if p.pattern_properties.is_some()
        || p.additional_schema.is_some()
        || p.enum_values.is_some()
        || p.const_value.is_some()
        || !p.field_groups.groups.is_empty()
        || !p.dependencies.is_empty()
    {
        return Err(p
            .emission_error("document narrowing cannot discard CFN object constraints")
            .into());
    }
    let required = fields
        .iter()
        .filter(|f| f.declared_input_mode == carina_core::schema::InputMode::Required)
        .count() as u64;
    if p.min_properties
        .as_ref()
        .and_then(serde_json::Number::as_u64)
        .is_some_and(|n| required < n)
        || p.max_properties
            .as_ref()
            .and_then(serde_json::Number::as_u64)
            .is_some_and(|n| fields.len() as u64 > n)
    {
        return Err(p
            .emission_error("document narrowing does not retain CFN property count bounds")
            .into());
    }
    for field in fields {
        let name = field.provider_name.unwrap_or(field.name);
        if let Some(source) = p.properties.as_ref().and_then(|ps| ps.get(name)) {
            validate_document_field(
                source.try_query()?,
                field.field_type,
                schema,
                CfnPropertyPath::new(Some(&p.source_name), name),
            )?;
        } else if p.additional_properties == Some(false) {
            return Err(p
                .emission_error(format!(
                    "document narrowing adds forbidden CFN field {name}"
                ))
                .into());
        }
    }
    Ok(())
}

fn validate_document_field(
    p: &Projection,
    declaration: &AttributeType,
    schema: &CfnSchema,
    path: CfnPropertyPath<'_>,
) -> Result<()> {
    let fail = |why: &str| p.emission_error(format!("document field narrowing: {why}"));
    match declaration.raw_shape() {
        RawShape::Struct { .. } => validate_document(p, declaration, schema),
        RawShape::String {
            identity_layer: StringIdentityParts {
                pattern, length, ..
            },
            ..
        } => {
            let plan = emission_plan::lower(p)?;
            let string = plan.string()?;
            if string.format.is_some_and(|format| format != "date-time") {
                return Err(fail("CFN format has no normalized containment proof").into());
            }
            // Use the same normalized constraints as ordinary string emission,
            // including format-derived patterns. Raw CFN pattern alone loses
            // date-time and any future format with a normalized constraint.
            let constraints =
                project_string_constraints(string, &schema.type_name, path, &p.source_pointer)?;
            let (min, max) = length.unwrap_or((None, None));
            if constraints
                .min_length
                .is_some_and(|n| min.is_none_or(|m| m < n))
                || constraints
                    .max_length
                    .is_some_and(|n| max.is_none_or(|m| m > n))
            {
                return Err(fail("CFN length is not retained").into());
            }
            if let Some(source) = &constraints.pattern
                && !pattern.is_some_and(|pattern| {
                    pattern_subset(pattern.as_str(), source.as_str()).unwrap_or(false)
                })
            {
                return Err(fail("CFN pattern is not retained").into());
            }
            Ok(())
        }
        RawShape::Enum {
            values: Some(values),
            ..
        } => {
            emission_plan::lower(p)?.validate_scalar_enum_candidates(
                &values.iter().map(String::as_str).collect::<Vec<_>>(),
            )?;
            Ok(())
        }
        RawShape::Int {
            identity_layer: IntIdentityParts { range, .. },
            ..
        } => {
            let plan = emission_plan::lower(p)?;
            if plan.value_type()? != EmissionType::Integer {
                return Err(fail("integer kind differs from CFN").into());
            }
            let (min, max) = range.unwrap_or((None, None));
            let (cmin, cmax) = plan.numeric()?.range.unwrap_or((None, None));
            if cmin.is_some_and(|n| min.is_none_or(|m| m < n))
                || cmax.is_some_and(|n| max.is_none_or(|m| m > n))
                || plan.numeric()?.multiple_of.is_some()
            {
                return Err(fail("CFN numeric constraints are not retained").into());
            }
            Ok(())
        }
        _ => Err(fail("unsupported constrained CFN field in document grammar").into()),
    }
}

/// Language containment, over all byte strings, for the same regex dialect as
/// CompiledPattern. Bounds are intersected separately. Unknown proofs fail closed.
fn pattern_subset(narrow: &str, wide: &str) -> Result<bool> {
    if narrow == wide {
        return Ok(true);
    }
    thread_local! {static CACHE:RefCell<HashMap<(String,String),bool>>=RefCell::new(HashMap::new());}
    let key = (narrow.into(), wide.into());
    if let Some(value) = CACHE.with(|c| c.borrow().get(&key).copied()) {
        return Ok(value);
    }
    use regex_automata::{Anchored, Input, dfa::Automaton};
    let build = proof_dfa;
    let narrow = build(narrow)?;
    let wide = build(wide)?;
    let input = Input::new("").anchored(Anchored::Yes);
    let start = (
        narrow.start_state_forward(&input)?,
        wide.start_state_forward(&input)?,
    );
    let mut seen = HashSet::from([start]);
    let mut queue = vec![start];
    while let Some((n, w)) = queue.pop() {
        if narrow.is_match_state(narrow.next_eoi_state(n))
            && !wide.is_match_state(wide.next_eoi_state(w))
        {
            CACHE.with(|c| c.borrow_mut().insert(key, false));
            return Ok(false);
        }
        for byte in 0..=255 {
            let next = (narrow.next_state(n, byte), wide.next_state(w, byte));
            if !narrow.is_dead_state(next.0) && seen.insert(next) {
                anyhow::ensure!(
                    seen.len() <= 100_000,
                    "pattern containment exceeded the proof state limit"
                );
                queue.push(next);
            }
        }
    }
    CACHE.with(|c| c.borrow_mut().insert(key, true));
    Ok(true)
}

// The API and CFN alphabets overlap but neither implies the other. Their
// conjunction excludes '_' permits ASCII whitespace only internally,
// and retains CFN's three-character minimum. Registered by pattern text.
pub(super) const WAF_DESCRIPTION_CFN: &str =
    r"^[a-zA-Z0-9=:#@/\-,.][a-zA-Z0-9+=:#@/\-,.\s]+[a-zA-Z0-9+=:#@/\-,.]{1,256}$";
pub(super) const WAF_DESCRIPTION_API: &str =
    r"^[A-Za-z0-9_+=:#@/,.-][A-Za-z0-9_+=:#@/,. \t\n\r\x0B\x0C-]+[A-Za-z0-9_+=:#@/,.-]$";
pub(super) const WAF_DESCRIPTION_INTERSECTION: &str =
    r"^[A-Za-z0-9=:#@/,.-][A-Za-z0-9+=:#@/,. \t\n\r\x0B\x0C-]+[A-Za-z0-9+=:#@/,.-]$";

fn reviewed_pattern_intersection(cfn: &str, api: &str) -> Option<&'static str> {
    if cfn == WAF_DESCRIPTION_CFN && api == WAF_DESCRIPTION_API {
        Some(WAF_DESCRIPTION_INTERSECTION)
    } else {
        None
    }
}

fn proof_dfa(source: &str) -> Result<regex_automata::dfa::dense::DFA<Vec<u32>>> {
    use regex_automata::{
        MatchKind,
        dfa::{StartKind, dense},
    };
    let translate = |source: &str| -> Result<String> {
        carina_core::schema::CompiledPattern::try_new(&carina_core::schema::PatternSpec::new(
            source,
            PatternMatch::Full,
        ))?;
        let mut result = String::new();
        let mut chars = source.chars();
        while let Some(ch) = chars.next() {
            if ch != '\\' {
                result.push(ch);
                continue;
            }
            let Some(next) = chars.next() else {
                anyhow::bail!("incomplete pattern escape")
            };
            result.push_str(match next {
                'd' => "[0-9]",
                'D' => "[^0-9]",
                'w' => "[A-Za-z0-9_]",
                'W' => "[^A-Za-z0-9_]",
                'b' | 'B' => {
                    anyhow::bail!("word boundary containment needs a reviewed intersection")
                }
                _ => {
                    result.push('\\');
                    result.push(next);
                    continue;
                }
            });
        }
        Ok(result)
    };
    Ok(dense::Builder::new()
        .configure(
            dense::Config::new()
                .start_kind(StartKind::Anchored)
                .match_kind(MatchKind::All)
                .dfa_size_limit(Some(32 * 1024 * 1024)),
        )
        .build(&translate(source)?)?)
}

fn prove_bounded_pattern_subset(
    narrow: &str,
    wide: &str,
    min: Option<u64>,
    max: Option<u64>,
) -> Result<bool> {
    if pattern_subset(narrow, wide)? {
        return Ok(true);
    }
    if min.is_none() && max.is_none() {
        return Ok(false);
    }
    use regex_automata::{Anchored, Input, dfa::Automaton};
    let length = length_pattern(min, max);
    let narrow = proof_dfa(narrow)?;
    let wide = proof_dfa(wide)?;
    let length = proof_dfa(&length)?;
    let input = Input::new("").anchored(Anchored::Yes);
    let start = (
        narrow.start_state_forward(&input)?,
        wide.start_state_forward(&input)?,
        length.start_state_forward(&input)?,
    );
    let mut seen = HashSet::from([start]);
    let mut queue = vec![start];
    while let Some((n, w, l)) = queue.pop() {
        if narrow.is_match_state(narrow.next_eoi_state(n))
            && length.is_match_state(length.next_eoi_state(l))
            && !wide.is_match_state(wide.next_eoi_state(w))
        {
            return Ok(false);
        }
        for byte in 0..=255 {
            let next = (
                narrow.next_state(n, byte),
                wide.next_state(w, byte),
                length.next_state(l, byte),
            );
            if !narrow.is_dead_state(next.0) && !length.is_dead_state(next.2) && seen.insert(next) {
                anyhow::ensure!(
                    seen.len() <= 200_000,
                    "bounded pattern containment exceeded the proof state limit"
                );
                queue.push(next);
            }
        }
    }
    Ok(true)
}

fn length_pattern(min: Option<u64>, max: Option<u64>) -> String {
    match (min, max) {
        (min, Some(max)) => format!(r"^[\s\S]{{{},{max}}}\z", min.unwrap_or(0)),
        (Some(min), None) => format!(r"^[\s\S]{{{min},}}\z"),
        (None, None) => r"^[\s\S]*\z".into(),
    }
}

fn bounded_pattern_subset(
    narrow: &str,
    wide: &str,
    min: Option<u64>,
    max: Option<u64>,
) -> Result<bool> {
    type Key = (String, String, Option<u64>, Option<u64>);
    thread_local! { static CACHE: RefCell<HashMap<Key, bool>> = RefCell::new(HashMap::new()); }
    let key = (narrow.to_string(), wide.to_string(), min, max);
    if let Some(answer) = CACHE.with(|cache| cache.borrow().get(&key).copied()) {
        return Ok(answer);
    }
    let answer = prove_bounded_pattern_subset(narrow, wide, min, max)?;
    CACHE.with(|cache| cache.borrow_mut().insert(key, answer));
    Ok(answer)
}

/// Find an actual value in the new structural domain outside the historical
/// domain. The product includes both regexes and both character-count bounds.
#[cfg(test)]
pub(super) fn loosening_example(
    new: &StringConstraints,
    old: &StringConstraints,
) -> Result<Option<String>> {
    use regex_automata::{Anchored, Input, dfa::Automaton};
    use std::collections::VecDeque;
    let any = r"^[\s\S]*\z";
    let np = new.pattern.as_ref().map_or(any, ApprovedPattern::as_str);
    let op = old.pattern.as_ref().map_or(any, ApprovedPattern::as_str);
    let contained_bounds = new.min_length.unwrap_or(0) >= old.min_length.unwrap_or(0)
        && old
            .max_length
            .is_none_or(|max| new.max_length.is_some_and(|n| n <= max));
    if contained_bounds && bounded_pattern_subset(np, op, new.min_length, new.max_length)? {
        return Ok(None);
    }
    let dfas = [
        proof_dfa(np)?,
        proof_dfa(&length_pattern(new.min_length, new.max_length))?,
        proof_dfa(op)?,
        proof_dfa(&length_pattern(old.min_length, old.max_length))?,
    ];
    let input = Input::new("").anchored(Anchored::Yes);
    let start = [
        dfas[0].start_state_forward(&input)?,
        dfas[1].start_state_forward(&input)?,
        dfas[2].start_state_forward(&input)?,
        dfas[3].start_state_forward(&input)?,
    ];
    let mut nodes = vec![(start, None::<(usize, u8)>)];
    let mut seen = HashMap::from([(start, 0)]);
    let mut queue = VecDeque::from([0]);
    while let Some(index) = queue.pop_front() {
        let state = nodes[index].0;
        let matches = std::array::from_fn::<_, 4, _>(|i| {
            dfas[i].is_match_state(dfas[i].next_eoi_state(state[i]))
        });
        if matches[0] && matches[1] && !(matches[2] && matches[3]) {
            let mut bytes = Vec::new();
            let mut current = index;
            while let Some((parent, byte)) = nodes[current].1 {
                bytes.push(byte);
                current = parent;
            }
            bytes.reverse();
            return Ok(Some(
                String::from_utf8(bytes).expect("the admitted regex domain is UTF-8"),
            ));
        }
        // Printable bytes first produce readable, deterministic counterexamples.
        for byte in (b' '..=b'~').chain(0..b' ').chain(127..=255) {
            let next = std::array::from_fn(|i| dfas[i].next_state(state[i], byte));
            if dfas[0].is_dead_state(next[0])
                || dfas[1].is_dead_state(next[1])
                || seen.contains_key(&next)
            {
                continue;
            }
            anyhow::ensure!(
                nodes.len() < 200_000,
                "structural difference proof exceeded its state limit"
            );
            let next_index = nodes.len();
            seen.insert(next, next_index);
            nodes.push((next, Some((index, byte))));
            queue.push_back(next_index);
        }
    }
    Ok(None)
}

impl ConstraintSpec {
    pub(super) fn numeric_display(
        &self,
        plan: &emission_plan::EmissionPlan<'_>,
    ) -> Result<Option<String>> {
        match &self.constraint {
            Constraint::IntegerRange(min, max) | Constraint::NumberRange(min, max) => {
                let (cmin, cmax) = plan.numeric()?.range.unwrap_or((None, None));
                let min = match (cmin, *min) {
                    (Some(a), Some(b)) => Some(a.max(b)),
                    (a, b) => a.or(b),
                };
                let max = match (cmax, *max) {
                    (Some(a), Some(b)) => Some(a.min(b)),
                    (a, b) => a.or(b),
                };
                let kind = if plan.value_type()? == EmissionType::Integer {
                    "Int"
                } else {
                    "Float"
                };
                let display = if min.is_none() && max.is_none() {
                    kind.to_string()
                } else {
                    format!("{kind}({})", range_display_string(min, max))
                };
                Ok(Some(numeric_type_display(display, plan).type_text()))
            }
            Constraint::Union(members) => {
                let displays = members
                    .iter()
                    .map(|member| member.numeric_display(plan))
                    .collect::<Result<Option<Vec<_>>>>()?;
                Ok(displays.map(|displays| displays.join(" | ")))
            }
            _ => Ok(None),
        }
    }
}

#[cfg(test)]
mod pattern_mode_tests {
    #[test]
    fn containment_proofs_use_full_value_languages() {
        for (narrow, wide, contained) in [
            ("a|b", "^a$|^b$", true),
            ("ab", "a", false),
            ("a", "ab", false),
            ("(?m)^a$", r"^a\z", true),
        ] {
            assert_eq!(
                super::pattern_subset(narrow, wide).unwrap(),
                contained,
                "{narrow:?} within {wide:?}"
            );
        }
    }
}
