use carina_core::schema::{RawShape, ValidatedResourceSchema};

use crate::test_support::{field, int, string};

fn schema() -> ValidatedResourceSchema {
    super::generated::get_config_by_type("identitystore.Group")
        .unwrap()
        .schema
        .clone()
}

#[test]
fn group_display_name_rejects_only_the_exact_reserved_names() {
    let schema = schema();
    let name = field(&schema, "display_name");
    for value in [
        "admins", // Live Cloud Control read, ap-northeast-1, 2026-11-05.
        "administrator",
        "Administrators",
        "AWSAdministrator",
        "ADMINISTRATOR",
        "awsadministrators",
        "AWSADMINISTRATORS",
        "Administrator group",
        " Administrator",
        "AWSAdministrators ",
    ] {
        assert!(name.validate(&string(value)).is_ok(), "{value:?}");
    }
    for value in ["Administrator", "AWSAdministrators"] {
        assert!(name.validate(&string(value)).is_err(), "{value:?}");
    }
}

#[test]
fn group_display_name_preserves_pattern_and_bounds_in_direct_callback_dispatch() {
    let schema = schema();
    let name = field(&schema, "display_name");
    let attr = schema.effective_attribute("display_name").unwrap();
    assert!(attr.is_required());
    assert!(!attr.is_create_only());
    let RawShape::String {
        identity: Some(identity),
        pattern,
        length,
        validate: Some(callback),
        ..
    } = name.as_type().raw_shape()
    else {
        panic!("Group.DisplayName must retain its identity and validator callback");
    };
    assert_eq!(identity.to_string(), "aws.identitystore.Group.DisplayName");
    assert_eq!(
        pattern,
        Some(&carina_core::schema::PatternSpec::new(
            r"^[\p{L}\p{M}\p{S}\p{N}\p{P}\t\n\r  ]+$",
            carina_core::schema::PatternMatch::Full
        ))
    );
    assert_eq!(length, Some((Some(1), Some(1024))));
    for (value, valid) in [
        ("admins".to_string(), true),
        ("administrator".to_string(), true),
        ("Administrators".to_string(), true),
        ("AWSAdministrator".to_string(), true),
        ("ADMINISTRATOR".to_string(), true),
        ("awsadministrators".to_string(), true),
        ("AWSADMINISTRATORS".to_string(), true),
        ("Administrator group".to_string(), true),
        (" Administrator".to_string(), true),
        ("AWSAdministrators ".to_string(), true),
        ("a".to_string(), true),
        ("a".repeat(1024), true),
        ("é".repeat(1024), true), // Bounds count characters, not UTF-8 bytes.
        ("部署 e\u{301} +★ １２３.,\t\n\r".to_string(), true),
        (" \t\n\r".to_string(), true),
        ("Administrator".to_string(), false),
        ("AWSAdministrators".to_string(), false),
        ("".to_string(), false),
        ("a".repeat(1025), false),
        ("é".repeat(1025), false),
        ("admins\0".to_string(), false),
        ("\0admins".to_string(), false),
        ("admins\u{b}".to_string(), false),
        ("admins\u{a0}".to_string(), false),
        ("admins\u{3000}".to_string(), false),
        ("admins\u{200b}".to_string(), false),
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
