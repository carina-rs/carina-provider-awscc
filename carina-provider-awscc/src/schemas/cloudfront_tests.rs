use std::path::Path;

use carina_core::executor::prepare_provider_ready_resource;
use carina_core::resource::{ConcreteValue, Resource, Value};
use carina_core::schema::{
    IntIdentityParts, RawShape, Shape, StringIdentityParts, TypeIdentity, ValidatedResourceSchema,
};
use serde_json::json;

use crate::provider::conversion::{aws_value_to_dsl_with_defs, dsl_value_to_aws_with_defs};
use crate::test_support::{
    AssignmentContext, enum_identifier, field, float, int, list, string, validate_fixtures,
    with_preparation,
};

const BEHAVIORS: [&str; 2] = [
    "distribution_config.default_cache_behavior",
    "distribution_config.cache_behaviors",
];

fn schema(resource_type: &str) -> ValidatedResourceSchema {
    super::generated::get_config_by_type(resource_type)
        .unwrap()
        .schema
        .clone()
}

fn map(entries: impl IntoIterator<Item = (&'static str, Value)>) -> Value {
    Value::Concrete(ConcreteValue::Map(
        entries
            .into_iter()
            .map(|(key, value)| (key.to_string(), value))
            .collect(),
    ))
}

#[test]
fn documented_enums_validate_and_round_trip_exact_wire_strings() {
    for (resource_type, path, lookup, expected) in [
        (
            "cloudfront.Distribution",
            "distribution_config.price_class",
            "price_class",
            &[
                ("PriceClass_100", "price_class_100"),
                ("PriceClass_200", "price_class_200"),
                ("PriceClass_All", "price_class_all"),
                ("None", "none"),
            ][..],
        ),
        (
            "cloudfront.Distribution",
            "distribution_config.viewer_certificate.minimum_protocol_version",
            "minimum_protocol_version",
            &[
                ("SSLv3", "sslv3"),
                ("TLSv1", "tlsv1"),
                ("TLSv1_2016", "tlsv1_2016"),
                ("TLSv1.1_2016", "tlsv1_1_2016"),
                ("TLSv1.2_2018", "tlsv1_2_2018"),
                ("TLSv1.2_2019", "tlsv1_2_2019"),
                ("TLSv1.2_2021", "tlsv1_2_2021"),
                ("TLSv1.3_2025", "tlsv1_3_2025"),
                ("TLSv1.2_2025", "tlsv1_2_2025"),
            ][..],
        ),
        (
            "cloudfront.Function",
            "stage",
            "stage",
            &[("DEVELOPMENT", "development"), ("LIVE", "live")][..],
        ),
    ] {
        let schema = schema(resource_type);
        let attr_type = field(&schema, path);
        let (_, values, _, _, spellings) = attr_type.as_type().enum_parts().unwrap();
        let wire_values = expected.iter().map(|(wire, _)| *wire).collect::<Vec<_>>();
        assert_eq!(
            values
                .unwrap()
                .iter()
                .map(String::as_str)
                .collect::<Vec<_>>(),
            wire_values,
            "{path}"
        );
        assert_eq!(
            super::generated::get_enum_valid_values(resource_type, lookup).unwrap(),
            wire_values,
            "{path}"
        );
        for (wire, dsl) in expected {
            assert_eq!(spellings.dsl_for(wire), *dsl, "{path}");
            assert_eq!(spellings.api_for(dsl), *wire, "{path}");
            attr_type.validate(&enum_identifier(dsl)).unwrap();
            assert!(
                attr_type.validate(&enum_identifier(wire)).is_err(),
                "{path}"
            );
            for quoted in [wire, dsl] {
                assert!(attr_type.validate(&string(quoted)).is_err(), "{path}");
            }
            let read = aws_value_to_dsl_with_defs(
                &crate::test_support::diagnostics(),
                lookup,
                &json!(wire),
                attr_type.as_type(),
                resource_type,
                schema.as_schema().defs(),
            )
            .unwrap();
            let canonical = attr_type.canonicalize_state(read).into_present().unwrap();
            assert_eq!(
                dsl_value_to_aws_with_defs(
                    &crate::test_support::diagnostics(),
                    &canonical,
                    attr_type.as_type(),
                    resource_type,
                    lookup,
                    schema.as_schema().defs(),
                ),
                Some(json!(wire)),
                "{path}: exact AWS spelling must survive read-back"
            );
        }
        assert!(attr_type.validate(&enum_identifier("unknown")).is_err());
    }
    assert!(
        schema("cloudfront.Function")
            .effective_attribute("stage")
            .unwrap()
            .is_read_only()
    );
}

#[tokio::test]
async fn authored_none_price_class_and_2025_tls_policies_serialize_exact_wire_strings() {
    for (dsl, wire) in [
        ("tlsv1_3_2025", "TLSv1.3_2025"),
        ("tlsv1_2_2025", "TLSv1.2_2025"),
    ] {
        let resource =
            Resource::with_provider("awscc", "cloudfront.Distribution", "wire_test", None)
                .with_attribute(
                    "distribution_config",
                    map([
                        ("enabled", Value::Concrete(ConcreteValue::Bool(false))),
                        ("price_class", enum_identifier("none")),
                        (
                            "viewer_certificate",
                            map([("minimum_protocol_version", enum_identifier(dsl))]),
                        ),
                        (
                            "default_cache_behavior",
                            map([
                                ("target_origin_id", string("origin")),
                                ("viewer_protocol_policy", enum_identifier("allow_all")),
                                ("min_ttl", int(0)),
                                ("default_ttl", int(300)),
                                ("max_ttl", int(86400)),
                            ]),
                        ),
                        (
                            "origins",
                            list(vec![map([
                                ("id", string("origin")),
                                ("domain_name", string("example.com")),
                            ])]),
                        ),
                    ]),
                );
        with_preparation("cloudfront.Distribution", async |preparation| {
            let ready = prepare_provider_ready_resource(resource, preparation)
                .await
                .unwrap();
            let schema = schema("cloudfront.Distribution");
            let serialized = dsl_value_to_aws_with_defs(
                &crate::test_support::diagnostics(),
                &ready.as_resource().attributes["distribution_config"],
                field(&schema, "distribution_config").as_type(),
                "cloudfront.Distribution",
                "distribution_config",
                schema.as_schema().defs(),
            )
            .unwrap();
            assert_eq!(serialized["PriceClass"], "None");
            assert_eq!(
                serialized["ViewerCertificate"]["MinimumProtocolVersion"],
                wire
            );
            for (name, value) in [("MinTTL", 0), ("DefaultTTL", 300), ("MaxTTL", 86400)] {
                assert_eq!(
                    serialized["DefaultCacheBehavior"][name].as_i64(),
                    Some(value)
                );
            }
        })
        .await;
    }
}

#[test]
fn all_seven_ttls_retain_cfn_numbers_and_round_trip_fractions() {
    let schema = schema("cloudfront.Distribution");
    let mut paths = BEHAVIORS
        .iter()
        .flat_map(|behavior| {
            ["min_ttl", "default_ttl", "max_ttl"].map(|ttl| format!("{behavior}.{ttl}"))
        })
        .collect::<Vec<_>>();
    paths.push("distribution_config.custom_error_responses.error_caching_min_ttl".to_string());
    for path in paths {
        let ttl = field(&schema, &path);
        assert!(
            matches!(
                ttl.as_type().raw_shape(),
                RawShape::Float {
                    identity: None,
                    range: None,
                    ..
                }
            ),
            "{path}"
        );
        // CFN number stays Float; attaching metadata cannot replace its kind.
        for value in [-10000, -1, 0, 10, 300, 86400, 100000] {
            ttl.validate(&int(value)).unwrap();
            let read = aws_value_to_dsl_with_defs(
                &crate::test_support::diagnostics(),
                &path,
                &json!(value),
                ttl.as_type(),
                "cloudfront.Distribution",
                schema.as_schema().defs(),
            )
            .unwrap();
            assert_eq!(read, int(value), "{path}");
            let canonical = ttl.canonicalize_state(read).into_present().unwrap();
            let wire = dsl_value_to_aws_with_defs(
                &crate::test_support::diagnostics(),
                &canonical,
                ttl.as_type(),
                "cloudfront.Distribution",
                &path,
                schema.as_schema().defs(),
            )
            .unwrap();
            assert_eq!(wire.as_i64(), Some(value), "{path}");
        }
        for value in [0.0, 1.0, 1.5] {
            assert!(ttl.validate(&float(value)).is_ok(), "{path}: {value}");
            let read = aws_value_to_dsl_with_defs(
                &crate::test_support::diagnostics(),
                &path,
                &json!(value),
                ttl.as_type(),
                "cloudfront.Distribution",
                schema.as_schema().defs(),
            )
            .unwrap();
            assert_eq!(read, float(value));
            let canonical = ttl.canonicalize_state(read).into_present().unwrap();
            assert_eq!(
                dsl_value_to_aws_with_defs(
                    &crate::test_support::diagnostics(),
                    &canonical,
                    ttl.as_type(),
                    "cloudfront.Distribution",
                    &path,
                    schema.as_schema().defs()
                )
                .unwrap()
                .as_f64(),
                Some(value)
            );
        }
        assert!(ttl.validate(&string("300")).is_err(), "{path}");
    }
}

#[test]
fn method_lists_enforce_counts_at_both_behaviors_and_remain_unordered_enums() {
    let schema = schema("cloudfront.Distribution");
    for behavior in BEHAVIORS {
        for (name, maximum, methods) in [
            (
                "allowed_methods",
                7,
                &["get", "head", "options", "put", "patch", "post", "delete"][..],
            ),
            ("cached_methods", 3, &["get", "head", "options"][..]),
        ] {
            let path = format!("{behavior}.{name}");
            let attr_type = field(&schema, &path);
            assert!(
                matches!(attr_type.shape(), Shape::List { ordered: false, .. }),
                "{path}"
            );
            for count in 0..=maximum + 1 {
                let value = list(
                    (0..count)
                        .map(|i| enum_identifier(methods[i % methods.len()]))
                        .collect(),
                );
                assert_eq!(
                    attr_type.validate(&value).is_ok(),
                    (2..=maximum).contains(&count),
                    "{path}: {count}"
                );
            }
            let item = attr_type.list_element().unwrap();
            for method in methods {
                item.validate(&enum_identifier(method)).unwrap();
            }
            assert!(item.validate(&enum_identifier("trace")).is_err());
            if name == "cached_methods" {
                assert!(item.validate(&enum_identifier("post")).is_err());
            }
            // carina-test-001, 2026-11-04: Cloud Control returned these
            // three strings in an order that may differ from the authored list.
            let read = aws_value_to_dsl_with_defs(
                &crate::test_support::diagnostics(),
                name,
                &json!(["OPTIONS", "HEAD", "GET"]),
                attr_type.as_type(),
                "cloudfront.Distribution",
                schema.as_schema().defs(),
            )
            .unwrap();
            let canonical = attr_type.canonicalize_state(read).into_present().unwrap();
            attr_type.validate(&canonical).unwrap();
            let wire = dsl_value_to_aws_with_defs(
                &crate::test_support::diagnostics(),
                &canonical,
                attr_type.as_type(),
                "cloudfront.Distribution",
                name,
                schema.as_schema().defs(),
            )
            .unwrap();
            let mut methods = wire
                .as_array()
                .unwrap()
                .iter()
                .map(|value| value.as_str().unwrap())
                .collect::<Vec<_>>();
            methods.sort_unstable();
            assert_eq!(methods, ["GET", "HEAD", "OPTIONS"]);
        }
    }
}

#[test]
fn lambda_and_key_value_store_association_bounds_preserve_struct_items() {
    let distribution = schema("cloudfront.Distribution");
    for behavior in BEHAVIORS {
        let path = format!("{behavior}.lambda_function_associations");
        let associations = field(&distribution, &path);
        for count in 0..=5 {
            let value = list(
                (0..count)
                    .map(|i| {
                        map([
                            ("event_type", enum_identifier("viewer_request")),
                            (
                                "lambda_function_arn",
                                string(&format!(
                                    "arn:aws:lambda:us-east-1:123456789012:function:example{i}:1"
                                )),
                            ),
                        ])
                    })
                    .collect(),
            );
            assert_eq!(
                associations.validate(&value).is_ok(),
                count <= 4,
                "{path}: {count}"
            );
        }
        assert!(
            associations
                .validate(&list(vec![string("flattened")]))
                .is_err()
        );
    }
    let function = schema("cloudfront.Function");
    let associations = field(&function, "function_config.key_value_store_associations");
    let association = map([(
        "key_value_store_arn",
        string(
            "arn:aws:cloudfront::123456789012:key-value-store/12345678-1234-1234-1234-123456789012",
        ),
    )]);
    for count in 0..=2 {
        let value = list((0..count).map(|i| map([("key_value_store_arn", string(&format!("arn:aws:cloudfront::123456789012:key-value-store/{i}2345678-1234-1234-1234-123456789012")))])).collect());
        assert_eq!(
            associations.validate(&value).is_ok(),
            count <= 1,
            "KVS: {count}"
        );
    }
    assert!(
        associations
            .validate(&list(vec![association.clone(), association]))
            .is_err()
    );
    assert!(
        associations
            .validate(&list(vec![string("flattened")]))
            .is_err()
    );
}

#[test]
fn origins_group_members_and_failover_codes_enforce_only_documented_counts() {
    let schema = schema("cloudfront.Distribution");
    let origins = field(&schema, "distribution_config.origins");
    for count in [0, 1, 101] {
        let value = list(
            (0..count)
                .map(|i| {
                    map([
                        ("id", string(&format!("origin{i}"))),
                        ("domain_name", string("example.com")),
                    ])
                })
                .collect(),
        );
        assert_eq!(
            origins.validate(&value).is_ok(),
            count >= 1,
            "origins: {count}"
        );
    }
    let members = field(
        &schema,
        "distribution_config.origin_groups.items.members.items",
    );
    for count in 0..=3 {
        let value = list(
            (0..count)
                .map(|i| map([("origin_id", string(&format!("origin{i}")))]))
                .collect(),
        );
        assert_eq!(
            members.validate(&value).is_ok(),
            count == 2,
            "members: {count}"
        );
    }
    assert!(
        members
            .validate(&list(vec![string("one"), string("two")]))
            .is_err()
    );
    let codes = field(
        &schema,
        "distribution_config.origin_groups.items.failover_criteria.status_codes.items",
    );
    for count in [0, 1, 20] {
        // No element enum or maximum is documented by API_StatusCodes.
        assert_eq!(
            codes.validate(&list(vec![int(500); count])).is_ok(),
            count >= 1,
            "status codes: {count}"
        );
    }
    assert!(codes.validate(&list(vec![float(500.0)])).is_err());
    assert!(codes.validate(&list(vec![string("500")])).is_err());
}

#[test]
fn connection_attempts_timeouts_and_error_codes_enforce_integer_boundaries() {
    let schema = schema("cloudfront.Distribution");
    for (path, minimum, maximum) in [
        ("distribution_config.origins.connection_attempts", 1, 3),
        ("distribution_config.origins.connection_timeout", 1, 10),
    ] {
        let attr_type = field(&schema, path);
        for value in [minimum, maximum] {
            attr_type.validate(&int(value)).unwrap();
        }
        for value in [minimum - 1, maximum + 1] {
            assert!(attr_type.validate(&int(value)).is_err(), "{path}: {value}");
        }
        assert!(attr_type.validate(&float(minimum as f64)).is_err());
        assert!(attr_type.validate(&string(&minimum.to_string())).is_err());
    }
    // AWS documents this supported set, rather than every 4xx/5xx code.
    // https://docs.aws.amazon.com/cloudfront/latest/APIReference/API_CustomErrorResponse.html
    let error_code = field(
        &schema,
        "distribution_config.custom_error_responses.error_code",
    );
    let supported = [400, 403, 404, 405, 414, 416, 500, 501, 502, 503, 504];
    for value in 399..=600 {
        assert_eq!(
            error_code.validate(&int(value)).is_ok(),
            supported.contains(&value),
            "{value}"
        );
    }
    assert!(error_code.validate(&float(400.0)).is_err());
    assert!(error_code.validate(&string("400")).is_err());
    for path in [
        "distribution_config.origins.custom_origin_config.origin_read_timeout",
        "distribution_config.origins.custom_origin_config.origin_keepalive_timeout",
        "distribution_config.origins.s3_origin_config.origin_read_timeout",
        "distribution_config.origins.vpc_origin_config.origin_read_timeout",
        "distribution_config.origins.vpc_origin_config.origin_keepalive_timeout",
    ] {
        let timeout = field(&schema, path);
        assert!(
            matches!(
                timeout.as_type().raw_shape(),
                RawShape::Int {
                    identity_layer: IntIdentityParts {
                        identity: None,
                        range: Some((Some(1), None)),
                        ..
                    },
                    ..
                }
            ),
            "{path}"
        );
        for value in [1, 120, 301, i64::MAX] {
            timeout.validate(&int(value)).unwrap();
        }
        for value in [-1, 0] {
            assert!(timeout.validate(&int(value)).is_err(), "{path}: {value}");
        }
        assert!(timeout.validate(&float(1.0)).is_err());
    }
}

#[test]
fn default_root_object_and_modern_origin_access_identity_match_complete_grammars() {
    let schema = schema("cloudfront.Distribution");
    let root = field(&schema, "distribution_config.default_root_object");
    for value in [
        "",
        "index.html",
        "folder/index.html",
        ".",
        "\n/index.html",
        "index.html\n",
    ] {
        root.validate(&string(value)).unwrap();
    }
    for value in ["/", "/index.html", "/\nindex.html"] {
        assert!(root.validate(&string(value)).is_err(), "{value:?}");
    }
    let identity = field(
        &schema,
        "distribution_config.origins.s3_origin_config.origin_access_identity",
    );
    for value in [
        "",
        "origin-access-identity/cloudfront/E123",
        "origin-access-identity/cloudfront/any suffix/é",
        "origin-access-identity/cloudfront/\n",
    ] {
        identity.validate(&string(value)).unwrap();
    }
    for value in [
        "E123",
        "origin-access-identity/cloudfront/",
        "/origin-access-identity/cloudfront/E123",
        "origin-access-identity/CloudFront/E123",
    ] {
        assert!(identity.validate(&string(value)).is_err(), "{value:?}");
    }
}

#[test]
fn geo_restriction_locations_are_two_uppercase_ascii_letters() {
    let schema = schema("cloudfront.Distribution");
    let locations = field(
        &schema,
        "distribution_config.restrictions.geo_restriction.locations",
    );
    let country = locations.list_element().unwrap();
    for value in ["US", "JP", "ZZ"] {
        country.validate(&string(value)).unwrap();
    }
    for value in ["", "U", "USA", "us", "Us", "U1", "ＵＳ", "US\n", "\nUS"] {
        assert!(country.validate(&string(value)).is_err(), "{value:?}");
    }
    locations.validate(&list(vec![])).unwrap();
    locations
        .validate(&list(vec![string("US"), string("JP")]))
        .unwrap();
}

#[test]
fn function_and_parameter_names_enforce_documented_ascii_grammars_and_lengths() {
    let function = schema("cloudfront.Function");
    let distribution = schema("cloudfront.Distribution");
    for (schema, path, maximum) in [
        (&function, "name", 64),
        (
            &distribution,
            "distribution_config.tenant_config.parameter_definitions.name",
            128,
        ),
    ] {
        let name = field(schema, path);
        for value in ["a".to_string(), "AZaz09-_".to_string(), "a".repeat(maximum)] {
            name.validate(&string(&value)).unwrap();
        }
        for value in [
            "".to_string(),
            "a".repeat(maximum + 1),
            "a.b".to_string(),
            "a/b".to_string(),
            "a b".to_string(),
            "é".to_string(),
            "a\n".to_string(),
            "a:b".to_string(),
        ] {
            assert!(name.validate(&string(&value)).is_err(), "{path}: {value:?}");
        }
    }
}

#[test]
fn string_caps_apply_at_every_header_certificate_parameter_and_oac_site() {
    let distribution = schema("cloudfront.Distribution");
    let oac = schema("cloudfront.OriginAccessControl");
    let policy = schema("cloudfront.ResponseHeadersPolicy");
    for (schema, path, minimum, maximum) in [
        (
            &distribution,
            "distribution_config.viewer_certificate.iam_certificate_id",
            0,
            32,
        ),
        (
            &distribution,
            "distribution_config.connection_function_association.id",
            1,
            64,
        ),
        (
            &distribution,
            "distribution_config.tenant_config.parameter_definitions.definition.string_schema.default_value",
            1,
            256,
        ),
        (
            &distribution,
            "distribution_config.origins.origin_custom_headers.header_name",
            0,
            256,
        ),
        (
            &distribution,
            "distribution_config.origins.origin_custom_headers.header_value",
            0,
            1783,
        ),
        (
            &policy,
            "response_headers_policy_config.custom_headers_config.items.header",
            0,
            256,
        ),
        (
            &policy,
            "response_headers_policy_config.custom_headers_config.items.value",
            0,
            1783,
        ),
        (&oac, "origin_access_control_config.name", 0, 64),
    ] {
        let attr_type = field(schema, path);
        for value in [
            "a".repeat(minimum),
            "a".repeat(maximum),
            "é".repeat(maximum),
            " /:#\n".to_string(),
        ] {
            attr_type.validate(&string(&value)).unwrap();
        }
        assert!(
            attr_type
                .validate(&string(&"a".repeat(maximum + 1)))
                .is_err(),
            "{path}"
        );
        assert!(
            attr_type
                .validate(&string(&"é".repeat(maximum + 1)))
                .is_err(),
            "{path}"
        );
        if minimum == 1 {
            assert!(attr_type.validate(&string("")).is_err(), "{path}");
        }
        assert!(attr_type.validate(&int(1)).is_err(), "{path}");
    }
}

#[test]
fn distribution_domain_name_reuses_core_and_reference_sinks_remain_strings() {
    let assignment = AssignmentContext::new();
    // Aggregate admission catches accidental redefinitions of core built-ins.
    let registry = super::schema_registry();
    let distribution = registry
        .get(
            "awscc",
            "cloudfront.Distribution",
            carina_core::schema::SchemaKind::Resource,
        )
        .unwrap();
    let schema = distribution.schema();
    assert!(
        schema
            .effective_attribute("domain_name")
            .unwrap()
            .is_read_only()
    );
    let domain = field(schema, "domain_name");
    assert!(
        matches!(domain.as_type().raw_shape(), RawShape::String { identity_layer: StringIdentityParts { identity: Some(identity), .. }, .. } if identity == &TypeIdentity::bare("DomainName"))
    );
    for value in ["d111111abcdef8.cloudfront.net", "example.com."] {
        domain.validate(&string(value)).unwrap();
    }
    for value in [
        "",
        "https://example.com",
        "*.example.com",
        "bad_name.example.com",
    ] {
        assert!(domain.validate(&string(value)).is_err());
    }
    // carina-rs/carina-provider-awscc#569: these inputs still receive String
    // outputs/module arguments. Only the Distribution output is tightened.
    for path in [
        "distribution_config.origins.domain_name",
        "distribution_config.custom_origin.dns_name",
        "distribution_config.s3_origin.dns_name",
        "distribution_config.logging.bucket",
        "distribution_config.aliases",
        "distribution_config.cnam_es",
    ] {
        let sink = field(schema, path);
        let sink = sink.list_element().unwrap_or(sink);
        assert!(
            matches!(
                sink.as_type().raw_shape(),
                RawShape::String {
                    identity_layer: StringIdentityParts {
                        identity: None,
                        pattern: None,
                        length: None,
                        ..
                    },
                    ..
                }
            ),
            "{path}"
        );
        sink.validate(&string("unrefined String reference"))
            .unwrap();
        assert!(
            schema
                .as_schema()
                .type_in_schema(domain.as_type())
                .is_assignable_to(assignment.sink(schema, sink.as_type())),
            "{path}"
        );
    }
    // The legacy API does not document the modern OAI prefix grammar.
    field(
        schema,
        "distribution_config.s3_origin.origin_access_identity",
    )
    .validate(&string("legacy-id"))
    .unwrap();
}

#[test]
fn widened_enums_and_integer_ttls_validate_in_complete_dsl_resources() {
    for tls in ["tlsv1_3_2025", "tlsv1_2_2025"] {
        let fixture = format!(
            r#"
provider awscc {{ region = awscc.Region.ap_northeast_1 }}
awscc.cloudfront.Distribution {{
  distribution_config = {{
    enabled = false
    price_class = none
    default_root_object = ''
    default_cache_behavior = {{
      target_origin_id = 'origin'
      viewer_protocol_policy = allow_all
      allowed_methods = [get, head, options]
      cached_methods = [head, get, options]
      min_ttl = 0
      default_ttl = 300
      max_ttl = 86400
    }}
    cache_behavior {{
      path_pattern = 'other/*'
      target_origin_id = 'origin'
      viewer_protocol_policy = https_only
      allowed_methods = [get, head]
      cached_methods = [get, head]
      min_ttl = 0
      default_ttl = 300
      max_ttl = 86400
    }}
    origin {{
      id = 'origin'
      domain_name = 'example.com'
    }}
    custom_error_responses = [{{
      error_code = 500
      error_caching_min_ttl = 10
    }}]
    viewer_certificate = {{ minimum_protocol_version = {tls} }}
  }}
}}
"#
        );
        validate_fixtures(&[("cloudfront_refinements.crn", &fixture)]);
    }
}

#[test]
fn every_example_and_acceptance_fixture_declaring_cloudfront_resources_validates() {
    fn collect(directory: &Path, root: &Path, fixtures: &mut Vec<(String, String)>) {
        let mut entries = std::fs::read_dir(directory)
            .unwrap()
            .map(|entry| entry.unwrap().path())
            .collect::<Vec<_>>();
        entries.sort();
        for path in entries {
            if path.is_dir() {
                collect(&path, root, fixtures);
            } else if path.extension().is_some_and(|extension| extension == "crn") {
                let source = std::fs::read_to_string(&path).unwrap();
                if source.contains("awscc.cloudfront.") {
                    fixtures.push((
                        path.strip_prefix(root)
                            .unwrap()
                            .to_string_lossy()
                            .into_owned(),
                        source,
                    ));
                }
            }
        }
    }
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
    let mut fixtures = Vec::new();
    for directory in ["examples", "acceptance-tests"] {
        collect(&root.join(directory), root, &mut fixtures);
    }
    assert!(
        !fixtures.is_empty(),
        "the CloudFront fixture inventory must not be empty"
    );
    let cases = fixtures
        .iter()
        .map(|(path, source)| (path.as_str(), source.as_str()))
        .collect::<Vec<_>>();
    validate_fixtures(&cases);
}
