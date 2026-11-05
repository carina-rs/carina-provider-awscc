use carina_core::schema::{CompiledAttributeType, CompiledPattern, RawShape};

use super::pattern::{ApprovedPattern, SUBSTRING_PATTERN_EXCEPTIONS, is_whole_anchored};
use crate::test_support::{field, string};

#[test]
fn whole_pattern_preserves_only_whole_anchored_sources_and_is_idempotent() {
    for source in [
        r"^A$",
        r"^(?:A|B)\z",
        r"^A$|^B$",
        r"^A\z|^B$",
        r"^[|()\[\]$]+$",
        r"^A\|B$",
        r"^$",
        r"^[a&&b]$", // Keep anchored syntax even when its language is empty.
        r"^(?m:A)$", // Multiline mode inside a group does not affect the outer anchors.
        r"^A(?m)B(?-m)$",
    ] {
        assert!(is_whole_anchored(source), "{source}");
        assert_eq!(ApprovedPattern::whole(source).as_str(), source);
    }
    for source in [
        "",
        "A",
        r"^A",
        r"A$",
        r"^A|B$",
        r"^A$|B$",
        r"^A$|^B",
        r"^A\$",
        r"^A\\z",
        r"^A$|",
        r"^A(?m)$",
        r"(?m)^A$",
        r"^A(?m)$|^B$",
    ] {
        assert!(!is_whole_anchored(source), "{source}");
        let approved = ApprovedPattern::whole(source);
        assert_eq!(approved.as_str(), format!(r"^(?:{source})\z"));
        assert!(is_whole_anchored(approved.as_str()), "{source}");
        assert_eq!(ApprovedPattern::whole(approved.as_str()), approved);
    }
    // Unsupported syntax must survive normalization for the compilation gate
    // to report it; normalization must never drop a constraint.
    assert_eq!(ApprovedPattern::whole("(?=x)x").as_str(), r"^(?:(?=x)x)\z");
}

#[test]
fn substring_exception_lookup_requires_the_exact_resource_path_and_source() {
    assert_eq!(SUBSTRING_PATTERN_EXCEPTIONS.len(), 1);
    let exception = &SUBSTRING_PATTERN_EXCEPTIONS[0];
    assert_eq!(exception.resource_type, "AWS::Scheduler::Schedule");
    assert_eq!(
        exception.source_path,
        "/definitions/EventBridgeParameters/properties/Source"
    );
    assert!(!exception.reason.trim().is_empty());
    let lookup =
        |resource, path, source| ApprovedPattern::substring_exception(resource, path, source);
    assert_eq!(
        lookup(
            exception.resource_type,
            exception.source_path,
            exception.source
        )
        .unwrap()
        .as_str(),
        exception.emitted,
    );
    assert!(
        lookup(
            "AWS::Test::Resource",
            exception.source_path,
            exception.source
        )
        .is_none()
    );
    assert!(
        lookup(
            exception.resource_type,
            "/properties/Source",
            exception.source
        )
        .is_none()
    );
    assert!(lookup(exception.resource_type, exception.source_path, "changed").is_none());
}

fn audit_patterns(
    resource: &str,
    path: &str,
    attr: &CompiledAttributeType,
    failures: &mut Vec<String>,
) -> usize {
    let mut count = 0;
    match attr.raw_shape() {
        RawShape::String { pattern, .. } | RawShape::Reference { pattern, .. } => {
            if let Some(pattern) = pattern {
                count += 1;
                if !is_whole_anchored(pattern)
                    && !SUBSTRING_PATTERN_EXCEPTIONS.iter().any(|exception| {
                        exception.resource_type == resource
                            && exception.schema_paths.contains(&path)
                            && exception.emitted == pattern
                    })
                {
                    failures.push(format!("{resource} {path}: {pattern:?}"));
                }
            }
        }
        RawShape::Struct { fields, .. } => {
            for field in fields {
                count += audit_patterns(
                    resource,
                    &format!("{path}.{}", field.name()),
                    field.field_type(),
                    failures,
                );
            }
        }
        RawShape::List { element_type, .. } => {
            count += audit_patterns(resource, &format!("{path}[]"), element_type, failures);
        }
        RawShape::Map { key, value } => {
            count += audit_patterns(resource, &format!("{path}.<key>"), key, failures);
            count += audit_patterns(resource, &format!("{path}.*"), value, failures);
        }
        RawShape::Union(members) => {
            for (index, member) in members.iter().enumerate() {
                count += audit_patterns(resource, &format!("{path}|{index}"), member, failures);
            }
        }
        RawShape::Enum { base, .. } => {
            count += audit_patterns(resource, &format!("{path}.<base>"), base, failures);
        }
        // Every definition is visited independently below, including unused
        // definitions. Following Ref here would loop on recursive WAF shapes.
        RawShape::Ref(_) => {}
        RawShape::UnloadedSchemaType(unloaded) => {
            panic!(
                "pattern audit needs a loaded type at {resource} {path}: {}",
                unloaded.identity()
            );
        }
        RawShape::Int { .. } | RawShape::Float { .. } | RawShape::Bool | RawShape::Duration => {}
    }
    count
}

#[test]
fn pattern_audit_requires_the_exception_schema_path() {
    let exception = &SUBSTRING_PATTERN_EXCEPTIONS[0];
    let config = super::generated::get_config_by_type("scheduler.Schedule").unwrap();
    let source = field(&config.schema, "target.event_bridge_parameters.source");
    let mut failures = Vec::new();
    for path in exception.schema_paths {
        assert_eq!(
            audit_patterns(
                exception.resource_type,
                path,
                source.as_type(),
                &mut failures
            ),
            1
        );
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));

    let unrelated_path = "target.input";
    assert_eq!(
        audit_patterns(
            exception.resource_type,
            unrelated_path,
            source.as_type(),
            &mut failures,
        ),
        1
    );
    assert_eq!(
        failures,
        vec![format!(
            "{} {unrelated_path}: {:?}",
            exception.resource_type, exception.emitted
        )]
    );
}

#[test]
fn all_registered_provider_schema_patterns_are_whole_anchored_or_registered_exceptions() {
    let mut failures = Vec::new();
    let mut count = 0;
    let mut tag_patterns = 0;
    let configs = super::generated::configs();
    assert!(!configs.is_empty());
    for config in configs {
        let schema = config.schema.as_schema();
        for (name, attr) in schema.attributes() {
            let inspected =
                audit_patterns(config.aws_type_name, name, attr.attr_type(), &mut failures);
            count += inspected;
            if name == "tags" {
                tag_patterns += inspected;
            }
        }
        for (name, attr) in schema.defs() {
            count += audit_patterns(
                config.aws_type_name,
                &format!("def:{name}"),
                attr,
                &mut failures,
            );
        }
    }
    assert!(count > 0, "audit must inspect emitted patterns");
    assert!(
        tag_patterns >= 9,
        "audit must inspect Tag key/value patterns, including SSM map keys"
    );
    assert!(
        failures.is_empty(),
        "unapproved provider patterns:\n{}",
        failures.join("\n")
    );
}

#[test]
fn oidc_thumbprint_items_match_exactly_forty_hex_characters() {
    let config = super::generated::get_config_by_type("iam.OidcProvider").unwrap();
    let thumbprints = field(&config.schema, "thumbprint_list");
    let item = thumbprints.list_element().unwrap();
    let RawShape::String {
        pattern: Some(pattern),
        ..
    } = item.as_type().raw_shape()
    else {
        panic!("thumbprint items must retain their pattern");
    };
    assert_eq!(pattern, r"^(?:[0-9A-Fa-f]{40})\z");
    let forty_hex = "aB01".repeat(10);
    assert!(item.validate(&string(&forty_hex)).is_ok());
    let embedded = format!("x{forty_hex}");
    // Length already rejects this; also exercise the pattern alone so the
    // regression cannot be hidden by the existing exact-length constraint.
    assert!(
        !CompiledPattern::try_new(pattern)
            .unwrap()
            .is_match(&embedded)
    );
    assert!(item.validate(&string(&embedded)).is_err());
}

#[test]
fn organization_management_account_email_rejects_embedded_email_values() {
    let config = super::generated::get_config_by_type("organizations.Organization").unwrap();
    let email = field(&config.schema, "management_account_email");
    assert!(matches!(
        email.as_type().raw_shape(),
        RawShape::String {
            length: Some((Some(6), Some(64))),
            ..
        }
    ));
    assert!(email.validate(&string("ab@x.y")).is_ok());
    for value in ["a b@x.y", "ab@x.y trailing", "\nab@x.y"] {
        assert!(email.validate(&string(value)).is_err(), "{value:?}");
    }
}

#[test]
fn organizations_account_path_items_reject_trailing_junk() {
    let config = super::generated::get_config_by_type("organizations.Account").unwrap();
    let paths = field(&config.schema, "paths");
    let item = paths.list_element().unwrap();
    for path in [
        "o-1234567890/r-abcd/",
        "o-1234567890/r-abcd/ou-abcd-12345678/123456789012/",
        // Preserve the published grammar, including repeated account IDs.
        "o-1234567890/r-abcd/123456789012/234567890123/",
    ] {
        assert!(item.validate(&string(path)).is_ok(), "{path}");
        for suffix in ["junk", "\n"] {
            assert!(
                item.validate(&string(&format!("{path}{suffix}"))).is_err(),
                "{path}{suffix:?}"
            );
        }
    }
}

#[test]
fn wafv2_non_whitespace_name_pattern_preserves_multiline_search_language() {
    let config = super::generated::get_config_by_type("wafv2.WebAcl").unwrap();
    let name = field(
        &config.schema,
        "rules.statement.rate_based_statement.custom_keys.cookie.name",
    );
    let RawShape::String {
        pattern: Some(pattern),
        ..
    } = name.as_type().raw_shape()
    else {
        panic!("WAF cookie name must retain its pattern");
    };
    assert_eq!(pattern, r"^[\s\S]*\S[\s\S]*\z");
    for (value, valid) in [
        ("\nx\n", true),
        ("\r\nx\r\n", true),
        ("", false),
        ("  \n", false),
    ] {
        assert_eq!(name.validate(&string(value)).is_ok(), valid, "{value:?}");
    }
}

#[test]
fn scheduler_source_preserves_the_registered_substring_language() {
    let config = super::generated::get_config_by_type("scheduler.Schedule").unwrap();
    let source = field(&config.schema, "target.event_bridge_parameters.source");
    let RawShape::String {
        pattern: Some(pattern),
        ..
    } = source.as_type().raw_shape()
    else {
        panic!("Scheduler Source must retain its reviewed pattern");
    };
    assert_eq!(pattern, SUBSTRING_PATTERN_EXCEPTIONS[0].emitted);
    for (value, valid) in [
        ("my source!?", true),
        ("a\n", true),
        ("!prefix$", true),
        ("\n$.detail", true),
        ("aws.$.detail", true),
        ("$", true),
        ("", false),
        ("aws.events", false),
        ("!source", false),
        ("$.detail trailing", false),
        ("$.detail\n", false),
    ] {
        assert_eq!(source.validate(&string(value)).is_ok(), valid, "{value:?}");
    }
}
