use carina_core::schema::RawShape;

use crate::test_support::{field, try_validate_fixtures, validate_fixtures};

fn fixture(attributes: &str) -> String {
    format!(
        r#"awscc.ssm.Parameter {{
  name = '/carina/ssm-length-validation'
  type = string
  value = 'example'
  {attributes}
}}"#
    )
}

#[test]
fn ssm_parameter_length_bounds_are_anonymous_strings() {
    let schema = super::generated::ssm::parameter::ssm_parameter_config().schema;
    for (path, expected_length) in [
        ("allowed_pattern", (Some(0), Some(1024))),
        ("policies", (Some(1), Some(4096))),
    ] {
        assert!(
            matches!(
                field(&schema, path).as_type().raw_shape(),
                RawShape::String {
                    identity: None,
                    pattern: None,
                    length: Some(length),
                    validate: None,
                    to_dsl: None,
                } if length == expected_length
            ),
            "{path} must be an anonymous string with only length bounds"
        );
    }
}

fn assert_length_fixtures(attribute: &str, cases: &[(usize, bool)]) {
    let mut failures = Vec::new();
    for &(length, accepted) in cases {
        // Policies JSON structure is outside these length checks (#524).
        let source = fixture(&format!("{attribute} = '{}'", "x".repeat(length)));
        let result = try_validate_fixtures(&[("ssm_parameter_length.crn", &source)]);
        if result.is_ok() != accepted {
            failures.push(format!(
                "{attribute} length {length}: expected accepted={accepted}, got {result:?}"
            ));
        } else if let Err(error) = result {
            assert!(error.contains(attribute), "{error}");
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn ssm_parameter_allowed_pattern_length_fixtures() {
    assert_length_fixtures("allowed_pattern", &[(0, true), (1024, true), (1025, false)]);
}

#[test]
fn ssm_parameter_policies_length_fixtures() {
    assert_length_fixtures(
        "policies",
        &[(1, true), (4096, true), (4097, false), (0, false)],
    );
}

#[test]
fn ssm_parameter_length_bounded_fields_remain_optional() {
    validate_fixtures(&[("ssm_parameter_optional.crn", &fixture(""))]);
}
