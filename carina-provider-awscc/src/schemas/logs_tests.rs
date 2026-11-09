use carina_core::resource::{ConcreteValue, Value};
use carina_core::schema::{ListElements, RawShape, TypeError, ValidatedResourceSchema};

use crate::test_support::{field, int, list, string};

fn schema() -> ValidatedResourceSchema {
    super::generated::get_config_by_type("logs.LogGroup")
        .unwrap()
        .schema
        .clone()
}

fn field_index_policy(name: &str) -> Value {
    Value::Concrete(ConcreteValue::Map(
        [("fields".to_string(), list(vec![string(name)]))]
            .into_iter()
            .collect(),
    ))
}

#[test]
fn log_group_name_rejects_the_reserved_prefix_and_accepts_infra_names() {
    let schema = schema();
    let name = field(&schema, "log_group_name");
    for value in [
        "/aws/lambda/example",
        "/aws/lambda/carina-registry-publish-api",
        "/aws/ecs/carina-registry-publish-api",
        "name#1",
        "/AWS/x",
        "my-aws/x",
    ] {
        assert!(name.validate(&string(value)).is_ok(), "{value:?}");
    }
    for value in ["aws/x", "aws/", "AWS/x", "Aws/x", "aWs/x"] {
        assert!(name.validate(&string(value)).is_err(), "{value:?}");
    }
}

#[test]
fn log_group_name_preserves_pattern_and_bounds_in_direct_callback_dispatch() {
    let schema = schema();
    let name = field(&schema, "log_group_name");
    let attr = schema.effective_attribute("log_group_name").unwrap();
    assert!(!attr.is_required());
    assert!(attr.is_create_only());
    let RawShape::String {
        identity: Some(identity),
        pattern,
        length,
        validate: Some(callback),
        ..
    } = name.as_type().raw_shape()
    else {
        panic!("LogGroup.LogGroupName must retain its identity and validator callback");
    };
    assert_eq!(identity.to_string(), "aws.logs.LogGroup.LogGroupName");
    assert_eq!(
        pattern,
        Some(&carina_core::schema::PatternSpec::new(
            r"^[.\-_/#A-Za-z0-9]{1,512}\z",
            carina_core::schema::PatternMatch::Full
        ))
    );
    assert_eq!(length, Some((Some(1), Some(512))));
    for (value, valid) in [
        ("a".to_string(), true),
        ("a".repeat(512), true),
        ("a".repeat(513), false),
        ("".to_string(), false),
        ("AZaz09.-_/#".to_string(), true),
        ("/aws/lambda/example".to_string(), true),
        ("/AWS/x".to_string(), true),
        ("name#1".to_string(), true),
        ("aws/x".to_string(), false),
        ("aws/".to_string(), false),
        ("AWS/x".to_string(), false),
        ("Aws/x".to_string(), false),
        ("aWs/x".to_string(), false),
        ("bad name".to_string(), false),
        ("name!".to_string(), false),
        ("name\n".to_string(), false),
        ("namé".to_string(), false),
    ] {
        assert_eq!(name.validate(&string(&value)).is_ok(), valid, "{value:?}");
        assert_eq!(
            callback(&string(&value)).is_ok(),
            valid,
            "callback: {value:?}"
        );
    }
    assert!(name.validate(&int(1)).is_err());
    assert!(callback(&int(1)).is_err());
}

#[test]
fn field_index_policies_allow_zero_or_one_policy_and_keep_list_semantics() {
    let schema = schema();
    let policies = field(&schema, "field_index_policies");
    assert!(
        !schema
            .effective_attribute("field_index_policies")
            .unwrap()
            .is_required()
    );
    let default = schema
        .as_schema()
        .default_value_attributes()
        .into_iter()
        .find(|(name, _)| *name == "field_index_policies")
        .map(|(_, value)| value.clone());
    assert_eq!(default, Some(list(vec![])));
    assert!(policies.validate(&list(vec![])).is_ok());
    assert!(
        policies
            .validate(&list(vec![field_index_policy("requestId")]))
            .is_ok()
    );
    assert!(matches!(
        policies.validate(&list(vec![
            field_index_policy("requestId"),
            field_index_policy("accountId"),
        ])),
        Err(TypeError::LengthOutOfRange {
            length: 2,
            min: None,
            max: Some(1),
            ..
        })
    ));
    assert!(matches!(
        policies.as_type().raw_shape(),
        RawShape::List { ordered: false, constraints, .. }
            if constraints.length() == Some((None, Some(1)))
                && constraints.elements() == ListElements::Unique
    ));
}
