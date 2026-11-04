use carina_core::schema::{ListElements, RawShape, ValidatedResourceSchema};
use serde_json::json;

use crate::provider::conversion::aws_value_to_dsl_with_defs;
use crate::test_support::{field, int, list, string};

fn schema() -> ValidatedResourceSchema {
    super::generated::get_config_by_type("organizations.Account")
        .unwrap()
        .schema
        .clone()
}

#[test]
fn account_email_keeps_core_validation_and_adds_documented_length_bounds() {
    let schema = schema();
    let email = field(&schema, "email");
    let RawShape::String {
        validate: Some(callback),
        ..
    } = email.as_type().raw_shape()
    else {
        panic!("Account.Email must retain email validation in direct callback dispatch");
    };
    for (value, valid) in [
        ("a@b.c".to_string(), false), // 5 characters: valid core Email, too short here.
        ("a@b.co".to_string(), true), // 6 characters.
        (format!("{}@b.com", "a".repeat(58)), true), // 64 characters.
        (format!("{}@b.com", "a".repeat(59)), false), // 65 characters.
        ("not-an-email".to_string(), false),
        ("user@example..com".to_string(), false),
        ("user@host".to_string(), false),
        ("us er@example.com".to_string(), false),
        // Historical reads retain core's grammar, without CreateAccount-only rules.
        (".historical@example.com".to_string(), true),
    ] {
        assert_eq!(email.validate(&string(&value)).is_ok(), valid, "{value:?}");
        assert_eq!(
            callback(&string(&value)).is_ok(),
            valid,
            "callback: {value:?}"
        );
    }
    assert!(email.validate(&int(1)).is_err());
    assert!(matches!(
        email.as_type().raw_shape(),
        RawShape::String {
            identity: Some(identity),
            length: Some((Some(6), Some(64))),
            validate: Some(_),
            ..
        } if identity.to_string() == "aws.organizations.Account.Email"
    ));
    assert!(matches!(
        carina_core::schema::types::email().raw_shape(),
        RawShape::String { length: None, .. }
    ));
}

#[test]
fn account_name_requires_printable_ascii_for_the_entire_value() {
    let schema = schema();
    let name = field(&schema, "account_name");
    for value in ["a".to_string(), " <registry> ~".to_string(), "a".repeat(50)] {
        assert!(name.validate(&string(&value)).is_ok(), "{value:?}");
    }
    for value in [
        "".to_string(),
        "a".repeat(51),
        "é".to_string(),
        "registry-é".to_string(),
        "é-registry".to_string(),
        "registry\n".to_string(),
        "registry\u{7f}".to_string(),
    ] {
        assert!(name.validate(&string(&value)).is_err(), "{value:?}");
    }
    assert!(name.validate(&int(1)).is_err());
}

#[test]
fn role_name_checks_the_whole_value_length_and_reserved_prefix() {
    let schema = schema();
    let role = field(&schema, "role_name");
    let attr = schema.effective_attribute("role_name").unwrap();
    assert!(attr.is_write_only());
    assert!(!attr.is_required());
    let default = schema
        .as_schema()
        .default_value_attributes()
        .into_iter()
        .find(|(name, _)| *name == "role_name")
        .map(|(_, value)| value.clone());
    assert_eq!(default, Some(string("OrganizationAccountAccessRole")));
    let RawShape::String {
        validate: Some(callback),
        ..
    } = role.as_type().raw_shape()
    else {
        panic!("Account.RoleName must reject the reserved prefix in direct callback dispatch");
    };
    for value in [
        "a".to_string(),
        "OrganizationAccountAccessRole".to_string(),
        "AZaz09_+=,.@-".to_string(),
        "a".repeat(64),
        "MyAWSServiceRoleForX".to_string(),
    ] {
        assert!(role.validate(&string(&value)).is_ok(), "{value:?}");
        assert!(callback(&string(&value)).is_ok(), "callback: {value:?}");
    }
    for value in [
        "".to_string(),
        "a".repeat(65),
        "AWSServiceRoleFor".to_string(),
        "AWSServiceRoleForX".to_string(),
        "bad role".to_string(),
        "role/".to_string(),
        "rôle".to_string(),
        "role\n".to_string(),
    ] {
        assert!(role.validate(&string(&value)).is_err(), "{value:?}");
        assert!(callback(&string(&value)).is_err(), "callback: {value:?}");
    }
    assert!(role.validate(&int(1)).is_err());
    assert!(matches!(
        role.as_type().raw_shape(),
        RawShape::String { identity: Some(identity), validate: Some(_), .. }
            if identity.to_string() == "aws.organizations.Account.RoleName"
    ));
}

#[test]
fn parent_ids_allow_at_most_one_parent_without_inventing_a_minimum() {
    let schema = schema();
    let parents = field(&schema, "parent_ids");
    assert!(
        !schema
            .effective_attribute("parent_ids")
            .unwrap()
            .is_required()
    );
    assert!(parents.validate(&list(vec![])).is_ok());
    for value in ["r-e4oi", "ou-abcd-12345678"] {
        assert!(parents.validate(&list(vec![string(value)])).is_ok());
    }
    assert!(
        parents
            .validate(&list(vec![string("r-e4oi"), string("ou-abcd-12345678")]))
            .is_err()
    );
    assert!(parents.validate(&string("r-e4oi")).is_err());
    assert!(matches!(
        parents.as_type().raw_shape(),
        RawShape::List { ordered: false, constraints, .. }
            if constraints.length() == Some((None, Some(1)))
                && constraints.elements() == ListElements::Unique
    ));
}

#[test]
fn parent_items_match_the_entire_root_or_ou_id_and_keep_the_documented_cap() {
    let schema = schema();
    let parents = field(&schema, "parent_ids");
    let item = parents.list_element().unwrap();
    assert!(matches!(
        item.as_type().raw_shape(),
        RawShape::String {
            identity: None,
            length: Some((None, Some(100))),
            ..
        }
    ));
    for value in [
        "r-e4oi".to_string(),
        format!("r-{}", "a".repeat(32)),
        "ou-abcd-12345678".to_string(),
        format!("ou-{}-{}", "a".repeat(32), "b".repeat(32)),
    ] {
        assert!(
            parents.validate(&list(vec![string(&value)])).is_ok(),
            "{value:?}"
        );
    }
    for value in [
        "".to_string(),
        "r-abc".to_string(),
        format!("r-{}", "a".repeat(33)),
        "ou-abc-12345678".to_string(),
        "ou-abcd-1234567".to_string(),
        format!("ou-{}-12345678", "a".repeat(33)),
        format!("ou-abcd-{}", "a".repeat(33)),
        "x-r-e4oi".to_string(),
        "r-e4oi/x".to_string(),
        "x-ou-abcd-12345678".to_string(),
        "ou-abcd-12345678/x".to_string(),
        "r-e4oi\n".to_string(),
        "ou-abcd-12345678\n".to_string(),
        "r-E4OI".to_string(),
        "a".repeat(101),
    ] {
        assert!(
            parents.validate(&list(vec![string(&value)])).is_err(),
            "{value:?}"
        );
    }
    assert!(parents.validate(&list(vec![int(1)])).is_err());
}

#[test]
fn live_account_reads_from_the_approved_plan_still_validate() {
    let schema = schema();
    // Read-only Cloud Control GetResource, management account, 2026-11-04.
    for (path, value) in [
        ("account_name", json!("registry-dev")),
        ("account_name", json!("registry-prod")),
        ("email", json!("admin+registry-dev@example.com")),
        ("email", json!("admin+registry-prod@example.com")),
        ("parent_ids", json!(["r-e4oi"])),
    ] {
        let attr = field(&schema, path);
        let read = aws_value_to_dsl_with_defs(
            path,
            &value,
            attr.as_type(),
            "organizations.Account",
            schema.as_schema().defs(),
        )
        .unwrap_or_else(|| panic!("{path}: could not convert {value}"));
        attr.validate(&read)
            .unwrap_or_else(|error| panic!("{path}: {value}: {error}"));
    }
}
