//! Confirms RedirectConfig fields on the ELBv2 Listener schema use the
//! placeholder-aware redirect_* sibling string types from carina-core. Asserts
//! both the inline Action.RedirectConfig site and the def-map RedirectConfig
//! site, so a future codegen regression that reverts either one is caught
//! locally.

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

fn assert_redirect_type(field: &StructField, expected_identity: &str, where_: &str) {
    let RawShape::String { identity, .. } = field.field_type.raw_shape() else {
        panic!(
            "{where_}: redirect field should be a refined string, got {:?}",
            field.field_type.raw_shape()
        );
    };
    assert_eq!(
        identity.map(|id| id.to_string()).as_deref(),
        Some(expected_identity),
        "{where_}: identity should be the bare carina-core {expected_identity} built-in"
    );
}

fn assert_redirect_fields(t: &AttributeType, where_: &str) {
    for (field_name, expected_identity) in [
        ("protocol", "RedirectProtocol"),
        ("host", "RedirectHost"),
        ("port", "RedirectPort"),
        ("path", "RedirectPath"),
        ("query", "RedirectQuery"),
    ] {
        let field = find_field(t, field_name)
            .unwrap_or_else(|| panic!("{where_} has a {field_name} field"));
        assert_redirect_type(field, expected_identity, where_);
    }
}

#[test]
fn redirect_config_uses_redirect_types_in_def_map() {
    let config = elasticloadbalancingv2_listener_config();
    let redirect_config = config
        .schema
        .defs
        .get("RedirectConfig")
        .expect("RedirectConfig def is registered");
    assert_redirect_fields(redirect_config, "defs[RedirectConfig]");
}

#[test]
fn redirect_config_uses_redirect_types_inline() {
    let config = elasticloadbalancingv2_listener_config();
    let default_actions = config
        .schema
        .attributes
        .get("default_actions")
        .expect("Listener has a default_actions attribute");
    let action = peel_list_element(&default_actions.attr_type);
    let redirect_config_field =
        find_field(action, "redirect_config").expect("Action has a redirect_config field");
    assert_redirect_fields(
        &redirect_config_field.field_type,
        "inline Action.RedirectConfig",
    );
}
