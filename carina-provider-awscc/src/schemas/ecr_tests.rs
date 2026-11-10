use carina_core::schema::{RawShape, StringIdentityParts, ValidatedResourceSchema};

use crate::test_support::{field, string};

const FILTER_VALUE_PATH: &str =
    "image_tag_mutability_exclusion_filters.image_tag_mutability_exclusion_filter_value";

fn schema() -> ValidatedResourceSchema {
    super::generated::get_config_by_type("ecr.Repository")
        .unwrap()
        .schema
        .clone()
}

#[test]
fn repository_exclusion_filter_values_accept_wildcards_and_the_maximum_length() {
    let schema = schema();
    let filter = field(&schema, FILTER_VALUE_PATH);
    let maximum = "a".repeat(128);
    for value in ["release-*", "v1.*_x", "*", &maximum] {
        assert!(filter.validate(&string(value)).is_ok(), "{value:?}");
    }
}

#[test]
fn repository_exclusion_filter_values_reject_invalid_characters_and_lengths() {
    let schema = schema();
    let filter = field(&schema, FILTER_VALUE_PATH);
    let too_long = "a".repeat(129);
    for value in [
        "release/*",
        "a!",
        "!release-*",
        " release-*",
        "release-*\n",
        "",
        &too_long,
    ] {
        assert!(filter.validate(&string(value)).is_err(), "{value:?}");
    }
}

#[test]
fn repository_exclusion_filter_values_remain_anonymous_refined_strings() {
    let schema = schema();
    let filter = field(&schema, FILTER_VALUE_PATH);
    assert!(matches!(
        filter.as_type().raw_shape(),
        RawShape::String { identity_layer: StringIdentityParts { identity: None,
            pattern: Some(pattern),
            length: Some((Some(1), Some(128))),
            validate: None,
            .. }, .. } if pattern == &carina_core::schema::PatternSpec::new(r"^[0-9a-zA-Z._*-]{1,128}\z", carina_core::schema::PatternMatch::Full)
    ));
}
