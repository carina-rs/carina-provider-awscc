//! SessionDuration's 1..=12 hour bound comes from the IAM Identity Center docs:
//! https://docs.aws.amazon.com/singlesignon/latest/userguide/howtosessionduration.html
//! It has NOT been confirmed live: no IAM Identity Center instance is available
//! in a carina-test-* account, CreatePermissionSet is AccessDenied there, and
//! the test organization forbids member account instances. These vectors keep
//! the existing CFN grammar and check duration length without canonicalizing.

use carina_core::schema::RawShape;
use carina_core::schema::StringIdentityParts;

use crate::test_support::{field, int, string, try_validate_fixtures};

const SESSION_DURATIONS: &[(&str, bool)] = &[
    ("PT1H", true),
    ("PT60M", true),
    ("PT3600S", true),
    ("PT3600.000S", true),
    ("PT1H30M", true),
    ("PT8H", true),
    ("PT12H", true),
    ("PT720M", true),
    ("PT43200S", true),
    ("P0DT8H", true),
    ("P0Y0M0DT2H", true),
    ("PT59M", false),
    ("PT3599S", false),
    ("PT3599.9999S", false),
    ("PT12H1M", false),
    ("PT12H0.001S", false),
    ("PT43200.000001S", false),
    ("PT13H", false),
    ("P1D", false),
    ("P1W", false),
    ("P1M", false),
    ("P1Y", false),
    ("PT0S", false),
    ("P0D", false),
    ("-PT8H", false),
    ("PT", false),
    ("P", false),
    ("", false),
    ("8h", false),
    ("pt8h", false),
    ("PT99999999999999999999999H", false),
    // Equivalent spellings, unnormalized components, and exact decimal bounds.
    ("P0Y0MT2H", true),
    ("P0WT8H", true),
    ("P000Y000M000WT0008H", true),
    ("PT0H60M", true),
    ("PT0H0M3600S", true),
    ("PT59M60S", true),
    ("PT11H60M", true),
    ("PT43199.999999999999999999S", true),
    ("PT43200.000000000000000000S", true),
    ("PT43200.000000000000000001S", false),
    ("PT3599.999999999999999999S", false),
    ("P1YT8H", false),
    ("P1MT8H", false),
    ("P1WT8H", false),
    ("P1DT8H", false),
    // Preserve CFN's ASCII digits, component order, and seconds-only fractions.
    ("PT0.5H", false),
    ("PT60.0M", false),
    ("PT1H.5S", false),
    ("PT1H0.S", false),
    ("PT1M2H", false),
    ("P0W0DT8H", false),
    ("PT8H8H", false),
    ("PT8HT", false),
    ("+PT8H", false),
    ("PT８H", false),
    ("PT٨H", false),
    ("PT1H0.١S", false),
    ("PT8H\n", false),
    (" PT8H", false),
];

#[test]
fn sso_permission_set_session_duration_length_vectors() {
    let schema = super::generated::sso::permission_set::sso_permission_set_config().schema;
    let duration = field(&schema, "session_duration");
    let failures = SESSION_DURATIONS
        .iter()
        .filter_map(|&(value, accepted)| {
            let result = duration.validate(&string(value));
            (result.is_ok() != accepted)
                .then(|| format!("{value:?}: expected accepted={accepted}, got {result:?}"))
        })
        .collect::<Vec<_>>();
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    assert!(duration.validate(&int(8)).is_err());
}

#[test]
fn sso_permission_set_session_duration_catalog_identity_bounds_and_callback() {
    let row = super::catalog::AwsccType::ALL
        .iter()
        .find(|row| row.identity().to_string() == "aws.sso.PermissionSet.SessionDuration")
        .expect("SessionDuration must be an awscc-local catalog row under aws");
    assert_eq!(row.refines(), None);
    assert_eq!(row.valid_sample(), "PT8H");
    let attribute_type = row.attribute_type();
    let RawShape::String {
        identity_layer:
            StringIdentityParts {
                identity: Some(identity),
                pattern: Some(_),
                length,
                validate: Some(callback),
                ..
            },
        ..
    } = attribute_type.raw_shape()
    else {
        panic!("SessionDuration must retain its string identity, grammar, and callback");
    };
    assert_eq!(identity.provider.as_deref(), Some("aws"));
    assert_eq!(identity.segments, ["sso", "PermissionSet"]);
    assert_eq!(identity.kind, "SessionDuration");
    assert_eq!(length, Some((Some(1), Some(100))));
    for &(value, accepted) in SESSION_DURATIONS {
        assert_eq!(
            callback(&string(value)).is_ok(),
            accepted,
            "callback: {value:?}"
        );
    }
    assert!(callback(&int(8)).is_err());
    let schema = super::generated::sso::permission_set::sso_permission_set_config().schema;
    assert!(matches!(
        field(&schema, "session_duration").as_type().raw_shape(),
        RawShape::String { identity_layer: StringIdentityParts { identity: Some(schema_identity),
            .. }, .. } if schema_identity == identity
    ));
}

#[test]
fn sso_permission_set_session_duration_large_components_and_cfn_length_bound() {
    let schema = super::generated::sso::permission_set::sso_permission_set_config().schema;
    let duration = field(&schema, "session_duration");
    for (value, accepted) in [
        (format!("PT{}8H", "0".repeat(96)), true),
        (format!("PT{}8H", "0".repeat(97)), false),
        (format!("P{}YT8H", "0".repeat(94)), true),
        (format!("PT3600.{}S", "0".repeat(92)), true),
        (format!("PT3600.{}S", "0".repeat(93)), false),
        (format!("PT43200.{}1S", "0".repeat(90)), false),
        (format!("PT{}H", "9".repeat(97)), false),
        (format!("PT{}M", "9".repeat(97)), false),
        (format!("PT{}S", "9".repeat(97)), false),
        (format!("PT{}H", u64::MAX), false),
        (format!("PT{}M", u64::MAX), false),
        (format!("PT{}S", u64::MAX), false),
    ] {
        assert_eq!(
            duration.validate(&string(&value)).is_ok(),
            accepted,
            "{value:?}"
        );
        let attribute_type = duration.as_type();
        let RawShape::String {
            identity_layer:
                StringIdentityParts {
                    validate: Some(callback),
                    ..
                },
            ..
        } = attribute_type.raw_shape()
        else {
            panic!("SessionDuration must retain its callback");
        };
        assert_eq!(callback(&string(&value)).is_ok(), accepted, "{value:?}");
    }
}

#[test]
fn sso_permission_set_session_duration_fixtures() {
    for (attribute, accepted) in [
        ("session_duration = 'P0DT8H'", true),
        ("session_duration = 'PT43200.000001S'", false),
        ("session_duration = 'PT3599.9999S'", false),
        ("", true),
    ] {
        let source = format!(
            r#"awscc.sso.PermissionSet {{
  instance_arn = 'arn:aws:sso:::instance/ssoins-1234567890abcdef'
  name = 'SessionDurationValidation'
  {attribute}
}}"#
        );
        let result = try_validate_fixtures(&[("sso_session_duration.crn", &source)]);
        assert_eq!(result.is_ok(), accepted, "{attribute}: {result:?}");
        if let Err(error) = result {
            assert!(error.contains("session_duration"), "{error}");
        }
    }
}
