//! Confirms that `FixedResponseConfig.StatusCode` on the ELBv2 Listener schema
//! is the pattern-validated `http_response_status_code()` scalar from
//! `carina-aws-types`, not a plain `string()`. The plain-string shape (the
//! CFN schema's default) lets `carina validate` accept arbitrary values like
//! `"nonsense"` that are only rejected at apply-time by CloudControl
//! `ValidationException`.
//!
//! `FixedResponseConfig` appears in TWO places in the generated schema:
//! inline under `attributes[default_actions][].fixed_response_config` and as
//! the def-map entry `schema.defs["FixedResponseConfig"]`. Both must carry the
//! override; we assert each independently.

use carina_core::schema::{AttributeType, RawShape, StructField};
use carina_provider_awscc::schemas::generated::elasticloadbalancingv2::listener::elasticloadbalancingv2_listener_config;

/// Walks a struct definition for a field by snake-case name.
fn find_field<'a>(t: &'a AttributeType, field: &str) -> Option<&'a StructField> {
    match t.raw_shape() {
        RawShape::Struct { fields, .. } => fields.iter().find(|f| f.name == field),
        _ => None,
    }
}

/// Peels a `List` shape to its element type.
fn peel_list_element(t: &AttributeType) -> Option<&AttributeType> {
    match t.raw_shape() {
        RawShape::List { element_type, .. } => Some(element_type),
        _ => None,
    }
}

/// Asserts the given `status_code` field is the shared
/// `aws.HttpResponseStatusCode` scalar, naming `where_` in the panic message.
fn assert_http_response_status_code(status_code: &StructField, where_: &str) {
    let RawShape::String {
        identity, pattern, ..
    } = status_code.field_type.raw_shape()
    else {
        panic!(
            "{where_}: status_code should be a refined string, got {:?}",
            status_code.field_type.raw_shape()
        );
    };
    assert_eq!(
        identity.map(|id| id.to_string()).as_deref(),
        Some("aws.HttpResponseStatusCode"),
        "{where_}: status_code identity should be the shared HttpResponseStatusCode scalar"
    );
    assert_eq!(
        pattern,
        Some(r"^(2|4|5)\d{2}$"),
        "{where_}: status_code should carry the 2XX/4XX/5XX pattern"
    );
}

#[test]
fn fixed_response_status_code_def_uses_http_response_status_code_scalar() {
    let config = elasticloadbalancingv2_listener_config();

    // `FixedResponseConfig` is registered as a definition on the resource
    // schema; resolve it via the schema's def map.
    let fixed_response = config
        .schema
        .defs
        .get("FixedResponseConfig")
        .expect("FixedResponseConfig def is registered");
    let status_code = find_field(fixed_response, "status_code")
        .expect("schema.defs[FixedResponseConfig] has a status_code field");

    assert_http_response_status_code(status_code, "schema.defs[FixedResponseConfig]");
}

#[test]
fn fixed_response_status_code_inline_uses_http_response_status_code_scalar() {
    let config = elasticloadbalancingv2_listener_config();

    // Walk attributes[default_actions] (list of Action) → element struct →
    // fixed_response_config → status_code, asserting the inline Action element
    // carries the override (independent from the schema.defs entry).
    let default_actions = config
        .schema
        .attributes
        .get("default_actions")
        .expect("schema has a default_actions top-level attribute");
    let action_elem = peel_list_element(&default_actions.attr_type)
        .expect("default_actions is a list of Action struct elements");
    let fixed_response_config = find_field(action_elem, "fixed_response_config")
        .expect("Action has a fixed_response_config field");
    let status_code = find_field(&fixed_response_config.field_type, "status_code")
        .expect("inline default_actions[].fixed_response_config has a status_code field");

    assert_http_response_status_code(
        status_code,
        "attributes[default_actions][].fixed_response_config",
    );
}
