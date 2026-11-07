//! Field-site defaults shared by Rust and Markdown. Reference-target annotations
//! never become use-site defaults. Unsupported runtime application retains the
//! exact CFN annotation and an explicit diagnostic in both artifacts.
use super::*;
use serde_json::Value;

#[derive(Clone, Copy)]
pub(super) enum Site {
    OptionalAttribute,
    RequiredOrOutput,
    StructField,
}

pub(super) struct DefaultPlan {
    pub(super) rust: Option<String>,
    pub(super) display: String,
    pub(super) diagnostic: Option<&'static str>,
}

impl DefaultPlan {
    pub(super) fn write_markdown(&self, page: &mut PageBuilder<MarkdownTarget>) {
        page.push_str("- **Default:** ");
        page.push_str(&TypeDisplay::Code(self.display.clone()).markdown());
        page.push_str("\n");
        if let Some(diagnostic) = self.diagnostic {
            page.push_str(&format!("- **Default application:** {diagnostic}\n"));
        }
    }

    pub(super) fn annotation(&self) -> String {
        format!(
            "CFN default: {}. {}",
            self.display,
            self.diagnostic.unwrap_or("")
        )
    }
}

pub(super) fn lower(
    property: &SchemaProperty,
    override_type: Option<&TypeOverride>,
    site: Site,
) -> Result<Option<DefaultPlan>> {
    let (rust, display) = if let Some(default) = override_type.and_then(TypeOverride::default) {
        (
            Some(default.value_expression.to_string()),
            default.markdown.to_string(),
        )
    } else if let Some(value) = property.annotation_default() {
        // Even when core cannot apply a default at this site, unsupported
        // JSON values must be diagnosed before a renderer discards them.
        if json_default_to_value_code(value).is_none() {
            return Err(property
                .try_query()?
                .emission_error("default contains null or an unrepresentable numeric value")
                .into());
        }
        let rust = normalize(property.try_query()?, value)?
            .and_then(|value| json_default_to_value_code(&value));
        (
            rust,
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
        Site::OptionalAttribute if rust.is_none() => Some(
            "AWS applies the server default; its JSON shape differs from the emitted DSL type.",
        ),
        Site::OptionalAttribute => None,
    };
    Ok(Some(DefaultPlan {
        rust: if diagnostic.is_none() { rust } else { None },
        display,
        diagnostic,
    }))
}

/// Translate provider field names in struct literals, preserve map keys, and
/// retain the emitted numeric kind. A shape mismatch remains a documented
/// server-side annotation instead of installing an invalid Carina default.
fn normalize(p: &Projection, value: &Value) -> Result<Option<Value>> {
    let plan = emission_plan::lower(p)?;
    if plan.kind == emission_plan::EmittedKind::JsonDocument {
        return Ok(None);
    }
    Ok(match (plan.value_type()?, value) {
        (EmissionType::String, Value::String(_)) | (EmissionType::Boolean, Value::Bool(_)) => {
            Some(value.clone())
        }
        (EmissionType::Integer, Value::Number(n)) if n.as_i64().is_some() => Some(value.clone()),
        (EmissionType::Number, Value::Number(n)) => {
            if let Some(integer) = n.as_i64() {
                plan.validate_numeric_range(Some(integer), None)?;
            }
            n.as_f64()
                .and_then(serde_json::Number::from_f64)
                .map(Value::Number)
        }
        (EmissionType::Array, Value::Array(values)) => {
            let items = plan.array()?.items.try_query()?;
            let mut normalized = Vec::new();
            for value in values {
                let Some(value) = normalize(items, value)? else {
                    return Ok(None);
                };
                normalized.push(value);
            }
            Some(Value::Array(normalized))
        }
        (EmissionType::Object, Value::Object(values)) => {
            let mut normalized = serde_json::Map::new();
            for (name, value) in values {
                let (key, child) = if let Some(fields) = plan.properties {
                    let Some(child) = fields.get(name) else {
                        return Ok(None);
                    };
                    (name.to_snake_case(), child)
                } else if let Some(map) = &plan.map {
                    (name.clone(), map.value)
                } else {
                    return Ok(None);
                };
                let Some(value) = normalize(child.try_query()?, value)? else {
                    return Ok(None);
                };
                normalized.insert(key, value);
            }
            Some(Value::Object(normalized))
        }
        _ => None,
    })
}
