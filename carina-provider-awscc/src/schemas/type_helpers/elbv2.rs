use carina_core::resource::{ConcreteValue, Value};
use carina_core::schema::{TypeIdentity, legacy_validator};

// Real-AWS probe by the awscc agent in carina-test-001, 2026-11-04:
// HTTP 0200, 200-0299, 200,0302 and gRPC 00-05, 0,05 were rejected.
// HealthCheckPort 08080 was accepted but read back as 8080; reject leading
// zeros in all three types to avoid phantom diffs. Equal range endpoints are accepted.

pub(crate) fn elbv2_matcher_codes(
    identity: TypeIdentity,
    min: u16,
    max: u16,
) -> crate::schemas::catalog_definition::IdentityDefinition {
    crate::schemas::catalog_definition::IdentityDefinition::string(
        identity,
        None,
        None,
        Some(legacy_validator(move |value| {
            if let Value::Concrete(ConcreteValue::String(value)) = value {
                validate_matcher_codes(value.expose_plaintext(), min, max).map_err(Into::into)
            } else {
                Err("Expected string".into())
            }
        })),
        None,
    )
}

fn validate_matcher_codes(codes: &str, min: u16, max: u16) -> Result<(), String> {
    let parse_code = |code: &str| {
        if code.is_empty()
            || !code.bytes().all(|digit| digit.is_ascii_digit())
            || (code.len() > 1 && code.starts_with('0'))
        {
            return None;
        }
        code.parse::<u16>()
            .ok()
            .filter(|code| (min..=max).contains(code))
    };

    let valid = codes.split(',').all(|item| {
        if let Some((start, end)) = item.split_once('-') {
            match (parse_code(start), parse_code(end)) {
                (Some(start), Some(end)) => start <= end,
                _ => false,
            }
        } else {
            parse_code(item).is_some()
        }
    });
    if valid {
        Ok(())
    } else {
        Err(format!(
            "ELBv2 matcher codes must be comma-separated items N or N-N (canonical decimal digits only, no leading zeros), with every N in {min}..={max} and range start <= end; no spaces or empty items"
        ))
    }
}
