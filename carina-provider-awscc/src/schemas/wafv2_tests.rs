//! #525 WAFv2 API constraints, with boundary cases from live probes.
//! direct CreateWebACL, carina-test-008, ap-northeast-1, 2026-11-06:
//! accepted a 128-character WebACL name; rejected 129 characters and a.b.
//! Cloud Control, carina-test-008, ap-northeast-1, 2026-11-06:
//! accepted a 128-character MetricName; rejected 129 at CFN model validation.
//! Semantic limits encountered by the probes are deliberately not type bounds.

use std::sync::LazyLock;

use carina_core::resource::{ConcreteValue, Value};
use carina_core::schema::{EffectiveType, RawShape, Shape, ValidatedResourceSchema};

use crate::test_support::{enum_identifier, field, float, int, list, string};

static WEB_ACL: LazyLock<ValidatedResourceSchema> =
    LazyLock::new(|| super::generated::wafv2::web_acl::wafv2_web_acl_config().schema);

fn attribute(path: &str) -> EffectiveType<'static> {
    field(&WEB_ACL, path)
}

fn statement(path: &str) -> EffectiveType<'static> {
    attribute(&format!("rules.statement.{path}"))
}

fn map(entries: Vec<(&str, Value)>) -> Value {
    Value::Concrete(ConcreteValue::Map(
        entries
            .into_iter()
            .map(|(k, v)| (k.to_string(), v))
            .collect(),
    ))
}

fn strings(attr: EffectiveType<'_>, cases: &[(&str, bool)]) {
    for &(value, accepted) in cases {
        let result = attr.validate(&string(value));
        assert_eq!(result.is_ok(), accepted, "{value:?}: {result:?}");
    }
}

fn counts(attr: EffectiveType<'_>, item: Value, cases: &[(usize, bool)]) {
    let item = attr
        .list_element()
        .unwrap()
        .canonicalize_state(item)
        .into_present()
        .unwrap();
    for &(count, accepted) in cases {
        let result = attr.validate(&list(vec![item.clone(); count]));
        assert_eq!(result.is_ok(), accepted, "{count} items: {result:?}");
    }
}

#[test]
fn web_acl_name_matches_live_boundaries_and_ascii_grammar() {
    strings(
        attribute("name"),
        &[
            ("acl_1-a", true),
            (&"a".repeat(128), true),
            (&"a".repeat(129), false),
            ("a.b", false),
            ("日本語", false),
            ("", false),
        ],
    );
}

#[test]
fn description_preserves_live_inner_newline_and_rejects_unicode_word_characters() {
    strings(
        attribute("description"),
        &[
            ("ddd", true),
            (&"d".repeat(256), true),
            (&"d".repeat(257), false),
            ("d", false),
            ("dd", false),
            (" ddd", false),
            ("ddd ", false),
            ("d d", true),
            ("d\nd", true),
            ("d!d", false),
            ("日本語", false),
            ("d\u{a0}d", false),
        ],
    );
}

#[test]
fn custom_response_body_map_keys_match_live_vectors() {
    let body = attribute("custom_response_bodies")
        .map_value()
        .unwrap()
        .canonicalize_state(map(vec![
            ("content", string("body")),
            ("content_type", string("TEXT_PLAIN")),
        ]))
        .into_present()
        .unwrap();
    for (key, accepted) in [
        ("k_1-a".to_string(), true),
        ("k.1".to_string(), false),
        ("k".repeat(128), true),
        ("k".repeat(129), false),
        ("".to_string(), false),
    ] {
        let result = attribute("custom_response_bodies").validate(&map(vec![(&key, body.clone())]));
        assert_eq!(result.is_ok(), accepted, "{key:?}: {result:?}");
    }
}

#[test]
fn tag_key_and_value_patterns_preserve_unicode_and_empty_values() {
    for (key, value, accepted) in [
        ("a b", "v", true),
        ("k", "日本語", true),
        ("k", "", true),
        ("!", "v", false),
        ("k", "!", false),
    ] {
        let result = attribute("tags").validate(&map(vec![(key, string(value))]));
        assert_eq!(result.is_ok(), accepted, "{key:?}={value:?}: {result:?}");
    }
}

#[test]
fn association_request_body_keys_follow_cached_pattern_properties() {
    let attr = attribute("association_config.request_body");
    let Shape::Map { key, .. } = attr.shape() else {
        panic!("expected a map")
    };
    let RawShape::Enum {
        values: Some(values),
        ..
    } = key.raw_shape()
    else {
        panic!("request body keys must be an enum")
    };
    let cfn: serde_json::Value = serde_json::from_str(include_str!(
        "../../../cfn-schema-cache/AWS__WAFv2__WebACL.json"
    ))
    .unwrap();
    let patterns = cfn["definitions"]["RequestBody"]["patternProperties"]
        .as_object()
        .unwrap();
    assert_eq!(patterns.len(), 1);
    let cached_keys = patterns
        .keys()
        .next()
        .unwrap()
        .strip_prefix("^(")
        .unwrap()
        .strip_suffix(")$")
        .unwrap()
        .split('|')
        .collect::<Vec<_>>();
    assert_eq!(
        values.iter().map(String::as_str).collect::<Vec<_>>(),
        cached_keys
    );
    let config = attr
        .map_value()
        .unwrap()
        .canonicalize_state(map(vec![(
            "default_size_inspection_limit",
            string("KB_16"),
        )]))
        .into_present()
        .unwrap();
    for (key, accepted) in [
        ("VERIFIED_ACCESS_INSTANCE", true),
        ("FOO", false),
        // Cloud Control, carina-test-008, ap-northeast-1, 2026-11-06:
        // accepted VERIFIED_ACCESS_INSTANCE with get-resource readback;
        // rejected AGENTCORE_GATEWAY as an extraneous key at CFN model validation.
        // direct CreateWebACL, carina-test-008, ap-northeast-1, 2026-11-06:
        // accepted AGENTCORE_GATEWAY; rejected FOO.
        // Let a cache refresh update the key enum and this expectation together.
        (
            "AGENTCORE_GATEWAY",
            cached_keys.contains(&"AGENTCORE_GATEWAY"),
        ),
    ] {
        let result = attr.validate(&map(vec![(key, config.clone())]));
        assert_eq!(result.is_ok(), accepted, "{key}: {result:?}");
    }
}

#[test]
fn custom_response_body_reference_matches_live_key_grammar() {
    strings(
        attribute("rules.action.block.custom_response.custom_response_body_key"),
        &[
            ("k_1", true),
            ("k.1", false),
            (&"k".repeat(128), true),
            (&"k".repeat(129), false),
        ],
    );
}

#[test]
fn custom_http_header_name_preserves_dollar_and_live_length_boundaries() {
    strings(
        attribute("rules.action.block.custom_response.response_headers.name"),
        &[
            ("x-a.b_c$", true),
            ("a b", false),
            ("a:b", false),
            (&"a".repeat(64), true),
            (&"a".repeat(65), false),
        ],
    );
}

#[test]
fn data_protections_have_the_live_maximum_of_26() {
    let item = map(vec![
        ("action", string("HASH")),
        ("field", map(vec![("field_type", string("BODY"))])),
    ]);
    counts(
        attribute("data_protection_config.data_protections"),
        item,
        &[(1, true), (26, true), (27, false), (0, false)],
    );
}

#[test]
fn protected_field_keys_keep_the_documented_100_instead_of_the_semantic_30_limit() {
    let attr = attribute("data_protection_config.data_protections.field.field_keys");
    strings(
        attr.list_element().unwrap(),
        &[
            ("a", true),
            (" ", false),
            (&"a".repeat(64), true),
            (&"a".repeat(65), false),
        ],
    );
    counts(attr, string("a"), &[(100, true), (101, false)]);
}

#[test]
fn rule_names_match_live_grammar_and_documented_lengths() {
    for path in [
        "rules.name",
        "rules.statement.managed_rule_group_statement.excluded_rules.name",
        "rules.statement.managed_rule_group_statement.rule_action_overrides.name",
    ] {
        strings(
            attribute(path),
            &[
                ("NoUserAgent_HEADER", true),
                (&"r".repeat(128), true),
                (&"r".repeat(129), false),
                ("a.b", false),
                ("a b", false),
            ],
        );
    }
}

#[test]
fn included_json_paths_match_live_pointer_and_list_boundaries() {
    let attr =
        statement("byte_match_statement.field_to_match.json_body.match_pattern.included_paths");
    let longest = format!("/{}", "a".repeat(511));
    let too_long = format!("/{}", "a".repeat(512));
    strings(
        attr.list_element().unwrap(),
        &[
            ("/", true),
            ("/a/b", true),
            ("/a~0", true),
            (&longest, true),
            ("abc", false),
            ("/a~2", false),
            (&too_long, false),
        ],
    );
    counts(attr, string("/a"), &[(1, true), (0, false)]);
}

#[test]
fn single_header_and_query_argument_names_keep_inner_spaces() {
    for name in ["single_header", "single_query_argument"] {
        strings(
            statement(&format!("byte_match_statement.field_to_match.{name}.name")),
            &[
                ("a b", true),
                (" ", false),
                (&"a".repeat(64), true),
                (&"a".repeat(65), false),
            ],
        );
    }
}

#[test]
fn forwarded_ip_header_names_use_ascii_and_the_live_255_boundary() {
    // IPSet variant: docs only, not probed; same documented API shape.
    for path in [
        "geo_match_statement.forwarded_ip_config.header_name",
        "ip_set_reference_statement.ip_set_forwarded_ip_config.header_name",
    ] {
        strings(
            statement(path),
            &[
                ("X-Forwarded-For", true),
                ("X_F", false),
                (&"a".repeat(255), true),
                (&"a".repeat(256), false),
            ],
        );
    }
}

fn acfp(path: &str) -> EffectiveType<'static> {
    statement(&format!(
        "managed_rule_group_statement.managed_rule_group_configs.aws_managed_rules_acfp_rule_set.{path}"
    ))
}

#[test]
fn response_status_codes_keep_live_0_and_999_and_existing_list_bounds() {
    for name in ["failure_codes", "success_codes"] {
        let attr = acfp(&format!("response_inspection.status_code.{name}"));
        let item = attr.list_element().unwrap();
        for (value, accepted) in [(0, true), (999, true), (1000, false), (-1, false)] {
            assert_eq!(
                item.validate(&int(value)).is_ok(),
                accepted,
                "{name}={value}"
            );
        }
        counts(
            attr,
            int(0),
            &[(1, true), (10, true), (0, false), (11, false)],
        );
    }
}

#[test]
fn label_names_keep_documented_lengths_instead_of_semantic_limits() {
    for path in [
        "rules.rule_labels.name",
        "rules.statement.label_match_statement.key",
    ] {
        strings(
            attribute(path),
            &[
                ("ns:label_1", true),
                (&"a".repeat(1024), true),
                (&"a".repeat(1025), false),
                ("a.b", false),
                ("a b", false),
            ],
        );
    }
}

#[test]
fn and_and_or_require_two_statements_as_in_the_live_probe() {
    for name in ["and_statement", "or_statement"] {
        counts(
            statement(&format!("{name}.statements")),
            map(vec![(
                "asn_match_statement",
                map(vec![("asn_list", list(vec![int(1)]))]),
            )]),
            &[(2, true), (1, false)],
        );
    }
}

#[test]
fn asn_list_has_live_1_to_100_items_and_keeps_its_integer_range() {
    let attr = statement("asn_match_statement.asn_list");
    counts(
        attr.clone(),
        int(1),
        &[(1, true), (100, true), (0, false), (101, false)],
    );
    let item = attr.list_element().unwrap();
    assert!(item.validate(&int(0)).is_ok());
    assert!(item.validate(&int(4294967295)).is_ok());
    assert!(item.validate(&int(4294967296)).is_err());
}

#[test]
fn byte_match_search_string_counts_utf8_bytes_as_in_cloud_control() {
    // direct CreateWebACL, carina-test-008, ap-northeast-1, 2026-11-06:
    // accepted 200 ASCII bytes and 66×'あ' (198 bytes);
    // rejected 201 ASCII bytes, 67×'あ' (201 bytes) and empty string.
    // Cloud Control, carina-test-008, ap-northeast-1, 2026-11-06:
    // accepted 66×'あ' and read back all 66 characters through get-resource.
    strings(
        statement("byte_match_statement.search_string"),
        &[
            ("a", true),
            (&"a".repeat(200), true),
            (&"a".repeat(201), false),
            (&"あ".repeat(66), true),
            (&"あ".repeat(67), false),
            ("", false),
        ],
    );
}

#[test]
fn byte_match_base64_uses_decoded_bytes_and_accepts_both_padding_forms() {
    // docs only, not probed separately. Standard base64, padding unspecified.
    let attr = statement("byte_match_statement.search_string_base64");
    let encoded_200 = format!("{}YWE=", "YWFh".repeat(66));
    let encoded_201 = "YWFh".repeat(67);
    strings(
        attr,
        &[
            ("YQ==", true),
            ("YQ", true),
            ("YWJj", true),
            ("/w==", true),
            (&encoded_200, true),
            (encoded_200.trim_end_matches('='), true),
            (&encoded_201, false),
            ("", false),
            ("!", false),
            ("Y", false),
            ("YQ=", false),
            ("_w==", false),
        ],
    );
}

#[test]
fn byte_match_catalog_types_have_the_exact_local_identities_and_callbacks() {
    for (field_name, identity, accepted, rejected) in [
        (
            "search_string",
            "aws.wafv2.WebACL.ByteMatchSearchString",
            "あ".repeat(66),
            "あ".repeat(67),
        ),
        (
            "search_string_base64",
            "aws.wafv2.WebACL.ByteMatchSearchStringBase64",
            "YQ==".into(),
            "".into(),
        ),
    ] {
        let attr = statement(&format!("byte_match_statement.{field_name}"));
        let RawShape::String {
            identity: Some(actual),
            length: None,
            validate: Some(callback),
            ..
        } = attr.as_type().raw_shape()
        else {
            panic!("byte bounds must use a named callback, not character bounds")
        };
        assert_eq!(actual.to_string(), identity);
        assert!(callback(&string(&accepted)).is_ok());
        assert!(callback(&string(&rejected)).is_err());
        assert!(callback(&int(1)).is_err());
    }
}

#[test]
fn country_codes_have_all_250_values_in_docs_order_and_live_list_bounds() {
    // https://docs.aws.amazon.com/waf/latest/APIReference/API_GeoMatchStatement.html#waf-Type-GeoMatchStatement-CountryCodes
    // The maximum of 50 CountryCodes is NOT documented on the API reference page.
    // direct CreateWebACL, carina-test-008, ap-northeast-1, 2026-11-06:
    // accepted all 250 documented codes in five batches of 50; rejected
    // XX/EU/UK/AA/jp and 51 entries ("Member must have length less than or equal to 50").
    // Cloud Control, carina-test-008, ap-northeast-1, 2026-11-06:
    // rejected CountryCodes [] server-side (minimum length 1).
    let attr = statement("geo_match_statement.country_codes");
    let item = attr.list_element().unwrap();
    let RawShape::Enum {
        values: Some(values),
        ..
    } = item.as_type().raw_shape()
    else {
        panic!("country codes must be an enum")
    };
    let expected = "AF AX AL DZ AS AD AO AI AQ AG AR AM AW AU AT AZ BS BH BD BB BY BE BZ BJ BM BT BO BQ BA BW BV BR IO BN BG BF BI KH CM CA CV KY CF TD CL CN CX CC CO KM CG CD CK CR CI HR CU CW CY CZ DK DJ DM DO EC EG SV GQ ER EE ET FK FO FJ FI FR GF PF TF GA GM GE DE GH GI GR GL GD GP GU GT GG GN GW GY HT HM VA HN HK HU IS IN ID IR IQ IE IM IL IT JM JP JE JO KZ KE KI KP KR KW KG LA LV LB LS LR LY LI LT LU MO MK MG MW MY MV ML MT MH MQ MR MU YT MX FM MD MC MN ME MS MA MZ MM NA NR NP NL NC NZ NI NE NG NU NF MP NO OM PK PW PS PA PG PY PE PH PN PL PT PR QA RE RO RU RW BL SH KN LC MF PM VC WS SM ST SA SN RS SC SL SG SX SK SI SB SO ZA GS SS ES LK SD SR SJ SZ SE CH SY TW TJ TZ TH TL TG TK TO TT TN TR TM TC TV UG UA AE GB US UM UY UZ VU VE VN VG VI WF EH YE ZM ZW XK";
    assert_eq!(values.len(), 250);
    assert_eq!(
        values.iter().map(String::as_str).collect::<Vec<_>>(),
        expected.split_whitespace().collect::<Vec<_>>()
    );
    for batch in values.chunks(50) {
        let value = attr
            .canonicalize_state(list(batch.iter().map(|v| string(v)).collect()))
            .into_present()
            .unwrap();
        assert!(attr.validate(&value).is_ok());
    }
    for value in ["XX", "EU", "UK", "AA"] {
        assert!(item.validate(&enum_identifier(value)).is_err(), "{value}");
    }
    // A quoted lowercase API value must not be mistaken for the DSL alias jp.
    assert!(item.validate(&string("jp")).is_err());
    counts(
        attr.clone(),
        string("JP"),
        &[(1, true), (50, true), (0, false), (51, false)],
    );
}

#[test]
fn all_eleven_text_transformation_lists_require_an_item() {
    let item = map(vec![("priority", int(0)), ("type", string("NONE"))]);
    for name in [
        "byte_match_statement",
        "regex_match_statement",
        "regex_pattern_set_reference_statement",
        "size_constraint_statement",
        "sqli_match_statement",
        "xss_match_statement",
    ] {
        counts(
            statement(&format!("{name}.text_transformations")),
            item.clone(),
            &[(1, true), (0, false)],
        );
    }
    // Other variants are documented with the same minimum; [] was probed on ByteMatch.
    for name in [
        "cookie",
        "header",
        "query_argument",
        "query_string",
        "uri_path",
    ] {
        counts(
            statement(&format!(
                "rate_based_statement.custom_keys.{name}.text_transformations"
            )),
            item.clone(),
            &[(1, true), (0, false)],
        );
    }
}

#[test]
fn excluded_rule_lists_keep_the_live_maximum_of_100() {
    // RuleGroupReference variant: docs only, not probed.
    for name in [
        "managed_rule_group_statement",
        "rule_group_reference_statement",
    ] {
        counts(
            statement(&format!("{name}.excluded_rules")),
            map(vec![("name", string("NoUserAgent_HEADER"))]),
            &[(100, true), (101, false)],
        );
    }
}

#[test]
fn rate_limit_label_namespace_requires_the_live_trailing_colon() {
    strings(
        statement("rate_based_statement.custom_keys.label_namespace.namespace"),
        &[
            ("ns:", true),
            ("ns", false),
            ("a.b:", false),
            (&format!("{}:", "a".repeat(1023)), true),
            (&format!("{}:", "a".repeat(1024)), false),
        ],
    );
}

#[test]
fn size_constraint_retains_the_cfn_number_kind_and_range() {
    let attr = statement("size_constraint_statement.size");
    for (value, accepted) in [
        (0, true),
        (21474836480, true),
        (-1, false),
        (21474836481, false),
    ] {
        assert_eq!(attr.validate(&int(value)).is_ok(), accepted, "{value}");
    }
    // The observed API coercion cannot justify replacing the CFN number kind.
    assert!(attr.validate(&float(1.5)).is_ok());
}

#[test]
fn acfp_creation_and_registration_paths_reject_live_blank_and_257_char_values() {
    for name in ["creation_path", "registration_page_path"] {
        strings(
            acfp(name),
            &[
                ("/register", true),
                (&"a".repeat(256), true),
                (" ", false),
                (&"a".repeat(257), false),
            ],
        );
    }
}

#[test]
fn metric_name_keeps_live_punctuation_and_all_and_cloud_control_maximum() {
    strings(
        attribute("visibility_config.metric_name"),
        &[
            ("a#b", true),
            ("a:b.c/d", true),
            ("All", true),
            ("a b", false),
            (&"a".repeat(128), true),
            (&"a".repeat(129), false),
        ],
    );
}
