//! Field-site defaults shared by Rust and Markdown. Reference-target annotations
//! never become use-site defaults. Unsupported runtime application retains the
//! exact CFN annotation and an explicit diagnostic in both artifacts.
use super::*;
use carina_core::resource::{ConcreteValue, Value as CoreValue};
use serde_json::Value;

#[derive(Clone, Copy)]
pub(super) enum Site {
    OptionalAttribute,
    RequiredOrOutput,
    StructField,
}

pub(super) struct DefaultPlan {
    pub(super) rust: Option<String>,
    pub(super) value: Option<carina_core::resource::Value>,
    pub(super) display: String,
    pub(super) diagnostic: Option<&'static str>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct DefaultAnnotation {
    pub(super) display: String,
    pub(super) diagnostic: &'static str,
}

impl DefaultAnnotation {
    pub(super) fn from_plan(plan: &DefaultPlan) -> Option<Self> {
        Some(Self {
            display: plan.display.clone(),
            diagnostic: plan.diagnostic?,
        })
    }

    pub(super) fn write_markdown(&self, page: &mut PageBuilder<MarkdownTarget>) {
        page.push_str("- **CFN default:** ");
        page.inline(TableCell::code(&self.display));
        page.push_str("\n");
        page.push_str(&format!("- **Default application:** {}\n", self.diagnostic));
    }
}

impl DefaultPlan {
    pub(super) fn annotation(&self) -> String {
        format!(
            "CFN default: {}. {}",
            self.display,
            self.diagnostic.unwrap_or("")
        )
    }
}

pub(super) fn display(value: &CoreValue) -> Result<String> {
    let known = value
        .knowledge(&carina_core::resource::knowledge::KnowledgeContext::validation())?
        .into_classification()
        .into_known()?;
    Ok(serde_json::to_string(
        &carina_core::protocol_convert::try_core_to_protocol_value(&known)?,
    )?)
}

pub(super) fn lower(
    property: &SchemaProperty,
    override_type: Option<&TypeOverride>,
    site: Site,
) -> Result<Option<DefaultPlan>> {
    let (value, display) = if let Some(default) = override_type.and_then(TypeOverride::default) {
        (Some(default.value.core()), default.markdown.to_string())
    } else if let Some(value) = property.annotation_default() {
        // Even when core cannot apply a default at this site, unsupported
        // JSON values must be diagnosed before a renderer discards them.
        let projection = property.try_query()?;
        let core = from_json(value).ok_or_else(|| {
            projection.emission_error("default contains null or an unrepresentable numeric value")
        })?;
        (
            normalize(projection, core)?,
            json_default_to_markdown(value).expect("supported JSON default"),
        )
    } else {
        return Ok(None);
    };
    let diagnostic = match site {
        Site::StructField => Some(
            "AWS applies the server default; pinned carina-core cannot apply nested field defaults.",
        ),
        Site::RequiredOrOutput => Some(
            "AWS applies the server default; defaults cannot be attached to required or provider-populated fields.",
        ),
        Site::OptionalAttribute if value.is_none() => Some(
            "AWS applies the server default; its JSON shape differs from the emitted DSL type.",
        ),
        Site::OptionalAttribute => None,
    };
    let value = value.filter(|_| diagnostic.is_none());
    Ok(Some(DefaultPlan {
        rust: value.as_ref().and_then(rust),
        value,
        display,
        diagnostic,
    }))
}

pub(super) fn from_json(value: &Value) -> Option<CoreValue> {
    let concrete = match value {
        Value::String(value) => ConcreteValue::String(value.clone().into()),
        Value::Bool(value) => ConcreteValue::Bool(*value),
        Value::Number(value) => {
            if let Some(integer) = value.as_i64() {
                ConcreteValue::Int(integer)
            } else if value.is_f64() {
                ConcreteValue::Float(carina_core::FiniteF64::new(value.as_f64()?).ok()?)
            } else {
                return None;
            }
        }
        Value::Array(values) => {
            ConcreteValue::List(values.iter().map(from_json).collect::<Option<_>>()?)
        }
        Value::Object(values) => ConcreteValue::Map(
            values
                .iter()
                .map(|(key, value)| Some((key.clone(), from_json(value)?)))
                .collect::<Option<_>>()?,
        ),
        Value::Null => return None,
    };
    Some(CoreValue::Concrete(concrete))
}

/// Translate provider field names in struct literals, preserve map keys, and
/// retain the emitted numeric kind. A shape mismatch remains a documented
/// server-side annotation instead of installing an invalid Carina default.
fn normalize(p: &Projection, value: CoreValue) -> Result<Option<CoreValue>> {
    let plan = emission_plan::lower(p)?;
    if plan.kind == emission_plan::EmittedKind::JsonDocument {
        return Ok(None);
    }
    let CoreValue::Concrete(value) = value else {
        return Ok(None);
    };
    let normalized = match (plan.value_type()?, value) {
        (EmissionType::String, value @ ConcreteValue::String(_))
        | (EmissionType::Boolean, value @ ConcreteValue::Bool(_))
        | (EmissionType::Integer, value @ ConcreteValue::Int(_)) => Some(value),
        (EmissionType::Number, ConcreteValue::Int(integer)) => {
            plan.validate_numeric_range(Some(integer), None)?;
            Some(ConcreteValue::Float(
                carina_core::FiniteF64::new(integer as f64).expect("finite integer default"),
            ))
        }
        (EmissionType::Number, value @ ConcreteValue::Float(_)) => Some(value),
        (EmissionType::Array, ConcreteValue::List(values)) => {
            let items = plan.array()?.items.try_query()?;
            let mut normalized = Vec::new();
            for value in values {
                let Some(value) = normalize(items, value)? else {
                    return Ok(None);
                };
                normalized.push(value);
            }
            Some(ConcreteValue::List(normalized))
        }
        (EmissionType::Object, ConcreteValue::Map(values)) => {
            let mut normalized = BTreeMap::new();
            for (name, value) in values {
                let (key, child) = if let Some(fields) = plan.properties {
                    let Some(child) = fields.get(&name) else {
                        return Ok(None);
                    };
                    (name.to_snake_case(), child)
                } else if let Some(map) = &plan.map {
                    (name, map.value)
                } else {
                    return Ok(None);
                };
                let Some(value) = normalize(child.try_query()?, value)? else {
                    return Ok(None);
                };
                normalized.insert(key, value);
            }
            Some(ConcreteValue::Map(normalized.into_iter().collect()))
        }
        _ => None,
    };
    Ok(normalized.map(CoreValue::Concrete))
}

pub(super) fn rust(value: &CoreValue) -> Option<String> {
    let CoreValue::Concrete(value) = value else {
        return None;
    };
    Some(match value {
        ConcreteValue::String(value) => format!(
            "Value::Concrete(ConcreteValue::String({}.to_string().into()))",
            rust_lit(value.diagnostic_literal()?)
        ),
        ConcreteValue::Bool(value) => format!("Value::Concrete(ConcreteValue::Bool({value}))"),
        ConcreteValue::Int(value) => format!("Value::Concrete(ConcreteValue::Int({value}))"),
        ConcreteValue::Float(value) => {
            let literal = serde_json::to_string(value).expect("finite JSON default");
            format!(
                "Value::Concrete(ConcreteValue::Float(carina_core::FiniteF64::new({literal}).expect(\"finite JSON default\")))"
            )
        }
        ConcreteValue::List(values) => format!(
            "Value::Concrete(ConcreteValue::List(vec![{}]))",
            values
                .iter()
                .map(rust)
                .collect::<Option<Vec<_>>>()?
                .join(", ")
        ),
        ConcreteValue::Map(values) => format!(
            "Value::Concrete(ConcreteValue::Map(vec![{}].into_iter().collect()))",
            values
                .iter()
                .map(|(key, value)| Some(format!(
                    "({}.to_string(), {})",
                    rust_lit(key),
                    rust(value)?
                )))
                .collect::<Option<Vec<_>>>()?
                .join(", ")
        ),
        _ => return None,
    })
}
