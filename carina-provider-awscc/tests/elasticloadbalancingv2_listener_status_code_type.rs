//! Confirms FixedResponseConfig.StatusCode on the ELBv2 Listener schema is
//! the pattern-validated http_response_status_code built-in from carina-core,
//! not a plain string. Asserts both the inline Action.FixedResponseConfig
//! site and the def-map FixedResponseConfig site, so a future codegen
//! regression that reverts either one is caught locally.

use carina_core::schema::{AttributeType, RawShape, StructField};
use carina_provider_awscc::schemas::generated::elasticloadbalancingv2::listener::elasticloadbalancingv2_listener_config;

fn find_field<'a>(t: &'a AttributeType, field: &str) -> Option<&'a StructField> {
    match t.raw_shape() {
        RawShape::Struct { fields, .. } => fields.iter().find(|f| f.name == field),
        _ => None,
    }
}

fn peel_list_element(t: &AttributeType) -> &AttributeType {
    match t.raw_shape() {
        RawShape::List { element_type, .. } => element_type,
        other => panic!("expected List shape, got {:?}", other),
    }
}

fn assert_http_response_status_code(field: &StructField, where_: &str) {
    let RawShape::String { identity, .. } = field.field_type.raw_shape() else {
        panic!(
            "{where_}: status_code should be a refined string, got {:?}",
            field.field_type.raw_shape()
        );
    };
    assert_eq!(
        identity.map(|id| id.to_string()).as_deref(),
        Some("HttpResponseStatusCode"),
        "{where_}: identity should be the bare carina-core HttpResponseStatusCode built-in"
    );
}

#[test]
fn fixed_response_status_code_uses_http_response_status_code_in_def_map() {
    let config = elasticloadbalancingv2_listener_config();
    let fixed_response = config
        .schema
        .defs
        .get("FixedResponseConfig")
        .expect("FixedResponseConfig def is registered");
    let status_code = find_field(fixed_response, "status_code")
        .expect("FixedResponseConfig has a status_code field");
    assert_http_response_status_code(status_code, "defs[FixedResponseConfig]");
}

#[test]
fn fixed_response_status_code_uses_http_response_status_code_inline() {
    let config = elasticloadbalancingv2_listener_config();
    let default_actions = config
        .schema
        .attributes
        .get("default_actions")
        .expect("Listener has a default_actions attribute");
    let action = peel_list_element(&default_actions.attr_type);
    let fixed_response_field = find_field(action, "fixed_response_config")
        .expect("Action has a fixed_response_config field");
    let status_code = find_field(&fixed_response_field.field_type, "status_code")
        .expect("Action.FixedResponseConfig has a status_code field");
    assert_http_response_status_code(status_code, "inline Action.FixedResponseConfig");
}
