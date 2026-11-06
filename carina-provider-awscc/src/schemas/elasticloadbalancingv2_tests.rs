use std::path::Path;

use carina_core::executor::prepare_provider_ready_resource;
use carina_core::resource::{ConcreteValue, Resource, Value};
use carina_core::schema::{CompiledAttributeType, RawShape, ValidatedResourceSchema};
use serde_json::json;

use crate::provider::conversion::{aws_value_to_dsl_with_defs, dsl_value_to_aws_with_defs};
use crate::test_support::{
    AssignmentContext, enum_identifier, field, float, int, list, string, validate_fixtures,
    with_preparation,
};

fn listener_schema() -> ValidatedResourceSchema {
    super::generated::elasticloadbalancingv2::listener::elasticloadbalancingv2_listener_config()
        .schema
}

fn load_balancer_schema() -> ValidatedResourceSchema {
    super::generated::elasticloadbalancingv2::load_balancer::elasticloadbalancingv2_load_balancer_config()
        .schema
}

fn target_group_schema() -> ValidatedResourceSchema {
    super::generated::elasticloadbalancingv2::target_group::elasticloadbalancingv2_target_group_config()
        .schema
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
fn integer_bounds_accept_endpoints_and_reject_outside_values_at_every_inline_path() {
    let listener = listener_schema();
    let target_group = target_group_schema();
    for (schema, path, min, max) in [
        (&listener, "port", 1, 65535),
        (&listener, "default_actions.order", 1, 50000),
        (
            &listener,
            "default_actions.forward_config.target_group_stickiness_config.duration_seconds",
            1,
            604800,
        ),
        (
            &listener,
            "default_actions.forward_config.target_groups.weight",
            0,
            999,
        ),
        (&target_group, "health_check_interval_seconds", 5, 300),
        (&target_group, "health_check_timeout_seconds", 2, 120),
        (&target_group, "healthy_threshold_count", 2, 10),
        (&target_group, "unhealthy_threshold_count", 2, 10),
        (&target_group, "port", 1, 65535),
        (&target_group, "target_control_port", 1, 65535),
        (&target_group, "targets.port", 1, 65535),
    ] {
        let attr_type = field(schema, path);
        for value in [min, max] {
            assert!(attr_type.validate(&int(value)).is_ok(), "{path}: {value}");
        }
        for value in [min - 1, max + 1] {
            assert!(attr_type.validate(&int(value)).is_err(), "{path}: {value}");
        }
        assert!(attr_type.validate(&float(min as f64)).is_err(), "{path}");
        assert!(
            attr_type.validate(&string(&min.to_string())).is_err(),
            "{path}"
        );
    }
}

#[test]
fn target_group_health_check_port_is_anonymous_and_requires_canonical_decimal_strings() {
    let schema = target_group_schema();
    let port = field(&schema, "health_check_port");
    assert!(matches!(
        port.as_type().raw_shape(),
        RawShape::String {
            identity: None,
            pattern: Some(_),
            length: None,
            validate: None,
            ..
        }
    ));
    for value in ["traffic-port", "1", "80", "8080", "65535"] {
        port.validate(&string(value)).unwrap();
    }
    // Live probe: carina-test-001, 2026-11-04. AWS accepts 08080 but reads
    // back 8080, so require canonical input to avoid a perpetual diff.
    for value in [
        "",
        "0",
        "65536",
        "01",
        "08080",
        "065535",
        "Traffic-port",
        "traffic-port\n",
        "traffic-portx",
        "+80",
        "-1",
        "1.0",
        "1e2",
        " 80",
        "80 ",
        "80\n",
        "80,81",
        "80-81",
        "８０",
    ] {
        assert!(port.validate(&string(value)).is_err(), "{value:?}");
    }
    assert!(port.validate(&int(8080)).is_err(), "CFN type stays String");
}

#[test]
fn target_group_matcher_codes_validate_every_inline_and_registered_definition() {
    let schema = target_group_schema();
    let RawShape::Struct { fields, .. } = schema.as_schema().defs()["Matcher"].raw_shape() else {
        panic!("Matcher definition must remain a struct");
    };
    // Matcher API domains are protocol-independent here; narrower protocol
    // restrictions remain cross-field. Live probe: carina-test-001, 2026-11-04
    // rejected HTTP 0200 / 200-0299 and gRPC 0,05 / 00-05.
    for (name, identity, accepted, rejected) in [
        (
            "http_code",
            "aws.elasticloadbalancingv2.TargetGroup.HttpCode",
            &[
                "200",
                "499",
                "500",
                "599",
                "200,202,599",
                "200-599",
                "200-299,302",
                "200-200",
            ][..],
            &[
                "",
                "199",
                "600",
                "0200",
                "200-0299",
                "200,0302",
                "199-200",
                "200-600",
                "599-200",
                ",200",
                "200,",
                "200,,201",
                "200, 201",
                "200-",
                "200--201",
                "200-201-202",
                "+200",
                "200.0",
                "200\n",
                "２００",
                "999999999999999999999999",
            ][..],
        ),
        (
            "grpc_code",
            "aws.elasticloadbalancingv2.TargetGroup.GrpcCode",
            &[
                "0", "99", "12", "0,1,99", "0-99", "0-5", "0-5,12", "0-0", "99-99",
            ][..],
            &[
                "",
                "-1",
                "100",
                "00",
                "0,05",
                "00-05",
                "0-100",
                "99-0",
                ",0",
                "0,",
                "0,,1",
                "0, 1",
                "0-",
                "0--1",
                "0-1-2",
                "+0",
                "0.0",
                "0\n",
                "０",
                "999999999999999999999999",
            ][..],
        ),
    ] {
        let inline = field(&schema, &format!("matcher.{name}"));
        let definition = fields
            .iter()
            .find(|field| field.name() == name)
            .unwrap()
            .field_type();
        for (site, attr_type) in [
            ("inline Matcher", inline.as_type()),
            ("definitions.Matcher", definition),
        ] {
            let RawShape::String {
                identity: Some(actual),
                pattern: None,
                length: None,
                validate: Some(validate),
                ..
            } = attr_type.raw_shape()
            else {
                panic!("{site}.{name} must use the shared named String validator");
            };
            assert_eq!(actual.to_string(), identity, "{site}.{name}");
            for value in accepted {
                assert!(validate(&string(value)).is_ok(), "{site}.{name}: {value:?}");
                inline.validate(&string(value)).unwrap();
            }
            for value in rejected {
                assert!(
                    validate(&string(value)).is_err(),
                    "{site}.{name}: {value:?}"
                );
                assert!(
                    inline.validate(&string(value)).is_err(),
                    "{name}: {value:?}"
                );
            }
            assert!(validate(&int(200)).is_err(), "{site}.{name} stays String");
        }
        assert!(inline.validate(&int(200)).is_err(), "{name} stays String");
    }
}

#[tokio::test]
async fn target_group_health_check_strings_and_attribute_values_round_trip_unchanged() {
    // These spellings round-tripped in carina-test-001 on 2026-11-04;
    // traffic-port and HTTP 200 also pin the existing Cloud Control fixture.
    for (port, code_name, wire_name, code) in [
        ("traffic-port", "http_code", "HttpCode", "200"),
        ("8080", "http_code", "HttpCode", "200-299,302"),
        ("traffic-port", "grpc_code", "GrpcCode", "0-5"),
        ("traffic-port", "grpc_code", "GrpcCode", "12"),
    ] {
        let resource = Resource::with_provider(
            "awscc",
            "elasticloadbalancingv2.TargetGroup",
            "health_check_wire",
            None,
        )
        .with_attribute("name", string("health-check-targets"))
        .with_attribute("protocol", enum_identifier("http"))
        .with_attribute(
            "protocol_version",
            enum_identifier(if code_name == "grpc_code" {
                "grpc"
            } else {
                "http1"
            }),
        )
        .with_attribute("port", int(8080))
        .with_attribute("target_type", enum_identifier("ip"))
        .with_attribute("vpc_id", string("vpc-dddd4444"))
        .with_attribute("health_check_port", string(port))
        .with_attribute("matcher", map([(code_name, string(code))]))
        .with_attribute(
            "target_group_attributes",
            list(vec![
                map([
                    ("key", string("deregistration_delay.timeout_seconds")),
                    ("value", string("300")),
                ]),
                map([
                    ("key", string("slow_start.duration_seconds")),
                    ("value", string("0")),
                ]),
                map([
                    ("key", string("stickiness.enabled")),
                    ("value", string("false")),
                ]),
            ]),
        );
        with_preparation("elasticloadbalancingv2.TargetGroup", async |preparation| {
            let ready = prepare_provider_ready_resource(resource, preparation)
                .await
                .unwrap();
            let schema = target_group_schema();
            for (attribute, expected) in [
                ("health_check_port", json!(port)),
                ("matcher", json!({ (wire_name): code })),
                (
                    "target_group_attributes",
                    json!([
                        {"Key": "deregistration_delay.timeout_seconds", "Value": "300"},
                        {"Key": "slow_start.duration_seconds", "Value": "0"},
                        {"Key": "stickiness.enabled", "Value": "false"},
                    ]),
                ),
            ] {
                let attr_type = field(&schema, attribute);
                assert_eq!(
                    dsl_value_to_aws_with_defs(
                        &ready.as_resource().attributes[attribute],
                        attr_type.as_type(),
                        "elasticloadbalancingv2.TargetGroup",
                        attribute,
                        schema.as_schema().defs(),
                    ),
                    Some(expected.clone()),
                    "authored {attribute}"
                );
                let read = aws_value_to_dsl_with_defs(
                    attribute,
                    &expected,
                    attr_type.as_type(),
                    "elasticloadbalancingv2.TargetGroup",
                    schema.as_schema().defs(),
                )
                .unwrap();
                let canonical = attr_type.canonicalize_state(read).into_present().unwrap();
                attr_type.validate(&canonical).unwrap();
                assert_eq!(
                    dsl_value_to_aws_with_defs(
                        &canonical,
                        attr_type.as_type(),
                        "elasticloadbalancingv2.TargetGroup",
                        attribute,
                        schema.as_schema().defs(),
                    ),
                    Some(expected),
                    "read-back {attribute}"
                );
            }
        })
        .await;
    }
}

#[test]
fn documented_enums_validate_and_round_trip_exact_aws_wire_strings() {
    let listener = listener_schema();
    let target_group = target_group_schema();
    for (schema, path, lookup, expected) in [
        (
            &listener,
            "alpn_policy",
            "alpn_policy",
            &[
                ("HTTP1Only", "http1_only"),
                ("HTTP2Only", "http2_only"),
                ("HTTP2Optional", "http2_optional"),
                ("HTTP2Preferred", "http2_preferred"),
                ("None", "none"),
            ][..],
        ),
        (
            &listener,
            "default_actions.authenticate_cognito_config.on_unauthenticated_request",
            "on_unauthenticated_request",
            &[
                ("deny", "deny"),
                ("allow", "allow"),
                ("authenticate", "authenticate"),
            ][..],
        ),
        (
            &listener,
            "default_actions.authenticate_oidc_config.on_unauthenticated_request",
            "on_unauthenticated_request",
            &[
                ("deny", "deny"),
                ("allow", "allow"),
                ("authenticate", "authenticate"),
            ][..],
        ),
        (
            &listener,
            "default_actions.jwt_validation_config.additional_claims.format",
            "format",
            &[
                ("single-string", "single_string"),
                ("string-array", "string_array"),
                ("space-separated-values", "space_separated_values"),
            ][..],
        ),
        (
            &listener,
            "mutual_authentication.advertise_trust_store_ca_names",
            "advertise_trust_store_ca_names",
            &[("on", "on"), ("off", "off")][..],
        ),
        (
            &target_group,
            "health_check_protocol",
            "health_check_protocol",
            &[("HTTP", "http"), ("HTTPS", "https"), ("TCP", "tcp")][..],
        ),
        (
            &target_group,
            "protocol",
            "protocol",
            &[
                ("HTTP", "http"),
                ("HTTPS", "https"),
                ("TCP", "tcp"),
                ("TLS", "tls"),
                ("UDP", "udp"),
                ("TCP_UDP", "tcp_udp"),
                ("GENEVE", "geneve"),
                ("QUIC", "quic"),
                ("TCP_QUIC", "tcp_quic"),
            ][..],
        ),
    ] {
        let attr_type = field(schema, path);
        let attr_type = attr_type.list_element().unwrap_or(attr_type);
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
            super::generated::get_enum_valid_values(schema.as_schema().resource_type(), lookup)
                .unwrap(),
            wire_values,
            "{path}"
        );
        for (wire, dsl) in expected {
            assert_eq!(spellings.dsl_for(wire), *dsl, "{path}");
            assert_eq!(spellings.api_for(dsl), *wire, "{path}");
            attr_type.validate(&enum_identifier(dsl)).unwrap();
            if wire != dsl {
                assert!(
                    attr_type.validate(&enum_identifier(wire)).is_err(),
                    "{path}"
                );
            }
            for quoted in [wire, dsl] {
                assert!(
                    attr_type.validate(&string(quoted)).is_err(),
                    "{path}: quoted {quoted}"
                );
            }
            let read = aws_value_to_dsl_with_defs(
                path,
                &json!(wire),
                attr_type.as_type(),
                schema.as_schema().resource_type(),
                schema.as_schema().defs(),
            )
            .unwrap();
            let canonical = attr_type.canonicalize_state(read).into_present().unwrap();
            assert_eq!(
                dsl_value_to_aws_with_defs(
                    &canonical,
                    attr_type.as_type(),
                    schema.as_schema().resource_type(),
                    path,
                    schema.as_schema().defs(),
                ),
                Some(json!(wire)),
                "{path}: AWS read-back must retain exact wire spelling"
            );
        }
        assert!(
            attr_type
                .validate(&enum_identifier("invalid_elbv2_enum"))
                .is_err(),
            "{path}"
        );
    }
    for unsupported in ["tls", "udp", "tcp_udp", "geneve", "quic", "tcp_quic"] {
        assert!(
            field(&target_group, "health_check_protocol")
                .validate(&enum_identifier(unsupported))
                .is_err()
        );
    }
}

#[tokio::test]
async fn authored_listener_identifiers_serialize_to_exact_wire_strings() {
    // Exercise desired-value preparation too: reading/canonicalizing an AWS
    // string alone would not catch a DSL alias leaking onto the create wire.
    async fn check(attribute: &str, value: Value, expected: serde_json::Value) {
        let resource = Resource::with_provider(
            "awscc", "elasticloadbalancingv2.Listener", "enum_wire", None,
        )
        .with_attribute("load_balancer_arn", string(
            "arn:aws:elasticloadbalancing:us-east-1:123456789012:loadbalancer/app/alb/abc123",
        ))
        .with_attribute("port", int(443))
        .with_attribute("protocol", enum_identifier("https"))
        .with_attribute("default_actions", list(vec![map([
            ("type", enum_identifier("forward")),
            ("target_group_arn", string(
                "arn:aws:elasticloadbalancing:us-east-1:123456789012:targetgroup/group/abc123",
            )),
        ])]))
        .with_attribute(attribute, value);
        with_preparation("elasticloadbalancingv2.Listener", async |preparation| {
            let ready = prepare_provider_ready_resource(resource, preparation)
                .await
                .unwrap();
            let schema = listener_schema();
            assert_eq!(
                dsl_value_to_aws_with_defs(
                    &ready.as_resource().attributes[attribute],
                    field(&schema, attribute).as_type(),
                    "elasticloadbalancingv2.Listener",
                    attribute,
                    schema.as_schema().defs(),
                ),
                Some(expected)
            );
        })
        .await;
    }
    for (wire, dsl) in [
        ("HTTP1Only", "http1_only"),
        ("HTTP2Only", "http2_only"),
        ("HTTP2Optional", "http2_optional"),
        ("HTTP2Preferred", "http2_preferred"),
        ("None", "none"),
    ] {
        check(
            "alpn_policy",
            list(vec![enum_identifier(dsl)]),
            json!([wire]),
        )
        .await;
    }
    for wire in ["on", "off"] {
        check(
            "mutual_authentication",
            map([("advertise_trust_store_ca_names", enum_identifier(wire))]),
            json!({"AdvertiseTrustStoreCaNames": wire}),
        )
        .await;
    }
    for (field_name, provider_name, wire_action, dsl_action, required, expected_required) in [
        (
            "authenticate_cognito_config",
            "AuthenticateCognitoConfig",
            "authenticate-cognito",
            "authenticate_cognito",
            vec![
                (
                    "user_pool_arn",
                    string("arn:aws:cognito-idp:us-east-1:123456789012:userpool/us-east-1_example"),
                ),
                ("user_pool_client_id", string("client")),
                ("user_pool_domain", string("domain-prefix")),
            ],
            json!({"UserPoolArn": "arn:aws:cognito-idp:us-east-1:123456789012:userpool/us-east-1_example", "UserPoolClientId": "client", "UserPoolDomain": "domain-prefix"}),
        ),
        (
            "authenticate_oidc_config",
            "AuthenticateOidcConfig",
            "authenticate-oidc",
            "authenticate_oidc",
            vec![
                (
                    "authorization_endpoint",
                    string("https://idp.example.com/authorize"),
                ),
                ("issuer", string("https://idp.example.com/issuer")),
                ("token_endpoint", string("https://idp.example.com/token")),
                (
                    "user_info_endpoint",
                    string("https://idp.example.com/userinfo"),
                ),
                ("client_id", string("client")),
                (
                    "use_existing_client_secret",
                    Value::Concrete(ConcreteValue::Bool(false)),
                ),
            ],
            json!({"AuthorizationEndpoint": "https://idp.example.com/authorize", "Issuer": "https://idp.example.com/issuer", "TokenEndpoint": "https://idp.example.com/token", "UserInfoEndpoint": "https://idp.example.com/userinfo", "ClientId": "client", "UseExistingClientSecret": false}),
        ),
    ] {
        for wire in ["deny", "allow", "authenticate"] {
            let mut config = required.clone();
            config.extend([
                ("on_unauthenticated_request", enum_identifier(wire)),
                ("session_timeout", string("604801")),
            ]);
            let mut expected_config = expected_required.clone();
            expected_config["OnUnauthenticatedRequest"] = json!(wire);
            expected_config["SessionTimeout"] = json!("604801");
            let mut expected = json!({"Type": wire_action});
            expected[provider_name] = expected_config;
            check(
                "default_actions",
                list(vec![map([
                    ("type", enum_identifier(dsl_action)),
                    (field_name, map(config)),
                ])]),
                json!([expected]),
            )
            .await;
        }
    }
    for (wire, dsl) in [
        ("single-string", "single_string"),
        ("string-array", "string_array"),
        ("space-separated-values", "space_separated_values"),
    ] {
        check(
            "default_actions",
            list(vec![map([
                ("type", enum_identifier("jwt_validation")),
                (
                    "jwt_validation_config",
                    map([
                        ("issuer", string("issuer-without-https")),
                        ("jwks_endpoint", string("https://idp.example.com/jwks")),
                        (
                            "additional_claims",
                            list(vec![map([
                                ("format", enum_identifier(dsl)),
                                ("name", string("group")),
                                ("values", list(vec![string("engineering")])),
                            ])]),
                        ),
                    ]),
                ),
            ])]),
            json!([{"Type": "jwt-validation", "JwtValidationConfig": {
                "Issuer": "issuer-without-https", "JwksEndpoint": "https://idp.example.com/jwks",
                "AdditionalClaims": [{"Format": wire, "Name": "group", "Values": ["engineering"]}],
            }}]),
        )
        .await;
    }
}

#[tokio::test]
async fn authored_quic_protocols_and_all_availability_zone_serialize_unchanged() {
    for (wire, dsl) in [("QUIC", "quic"), ("TCP_QUIC", "tcp_quic")] {
        let resource = Resource::with_provider(
            "awscc",
            "elasticloadbalancingv2.TargetGroup",
            "quic_wire",
            None,
        )
        .with_attribute("name", string("quic-targets"))
        .with_attribute("protocol", enum_identifier(dsl))
        .with_attribute("port", int(443))
        .with_attribute("target_type", enum_identifier("ip"))
        .with_attribute("vpc_id", string("vpc-dddd4444"))
        .with_attribute(
            "targets",
            list(vec![map([
                ("id", string("10.0.1.10")),
                ("availability_zone", string("all")),
                ("port", int(443)),
                ("quic_server_id", string("0x0123456789abcdef")),
            ])]),
        );
        with_preparation("elasticloadbalancingv2.TargetGroup", async |preparation| {
            let ready = prepare_provider_ready_resource(resource, preparation).await.unwrap();
            let schema = target_group_schema();
            for (attribute, expected) in [
                ("protocol", json!(wire)),
                ("targets", json!([{"Id": "10.0.1.10", "AvailabilityZone": "all", "Port": 443, "QuicServerId": "0x0123456789abcdef"}])),
            ] {
                assert_eq!(dsl_value_to_aws_with_defs(
                    &ready.as_resource().attributes[attribute], field(&schema, attribute).as_type(),
                    "elasticloadbalancingv2.TargetGroup", attribute, schema.as_schema().defs(),
                ), Some(expected));
            }
        }).await;
    }
}

#[test]
fn session_timeouts_require_canonical_decimal_strings_with_minimum_one_and_no_maximum() {
    let schema = listener_schema();
    for path in [
        "default_actions.authenticate_cognito_config.session_timeout",
        "default_actions.authenticate_oidc_config.session_timeout",
    ] {
        let timeout = field(&schema, path);
        for value in [
            "1",
            "604800",
            "604801",
            "999999999999999999999999999999999999",
        ] {
            timeout.validate(&string(value)).unwrap();
        }
        for value in [
            "", "0", "01", "0604800", "000", "0001", "-1", "+1", "1.0", "1e3", " 1", "1 ", "1\n",
            "١",
        ] {
            assert!(
                timeout.validate(&string(value)).is_err(),
                "{path}: {value:?}"
            );
        }
        assert!(
            timeout.validate(&int(1)).is_err(),
            "{path} must remain CFN String"
        );
    }
}

#[test]
fn names_enforce_documented_length_and_characters_for_input_and_read_back() {
    let load_balancer = load_balancer_schema();
    let target_group = target_group_schema();
    for (schema, path) in [
        (&load_balancer, "name"),
        (&load_balancer, "load_balancer_name"),
        (&target_group, "name"),
        (&target_group, "target_group_name"),
    ] {
        let name = field(schema, path);
        for value in [
            "a".to_string(),
            "AZaz09-a".to_string(),
            "a--b".to_string(),
            "a".repeat(32),
        ] {
            name.validate(&string(&value)).unwrap();
        }
        for value in [
            "".to_string(),
            "a".repeat(33),
            "-a".to_string(),
            "a-".to_string(),
            "a_b".to_string(),
            "a b".to_string(),
            "a/b".to_string(),
            "a\n".to_string(),
            "\na".to_string(),
            "é".to_string(),
        ] {
            assert!(name.validate(&string(&value)).is_err(), "{path}: {value:?}");
        }
    }
    // The reserved LB prefix stays AWS-side without a readable lookaround-free
    // expression. Target groups have no corresponding restriction.
    for schema in [&load_balancer, &target_group] {
        field(schema, "name")
            .validate(&string("internal-app"))
            .unwrap();
    }
}

#[test]
fn attribute_keys_enforce_shape_and_load_balancer_values_allow_empty_strings() {
    let listener = listener_schema();
    let load_balancer = load_balancer_schema();
    let target_group = target_group_schema();
    for (schema, path) in [
        (&listener, "listener_attributes.key"),
        (&load_balancer, "load_balancer_attributes.key"),
        (&target_group, "target_group_attributes.key"),
    ] {
        let key = field(schema, path);
        for value in ["a".to_string(), "AZaz09._".to_string(), "a".repeat(256)] {
            key.validate(&string(&value)).unwrap();
        }
        for value in [
            "".to_string(),
            "a".repeat(257),
            "a-b".to_string(),
            "a b".to_string(),
            "a/b".to_string(),
            "a\n".to_string(),
            "é".to_string(),
        ] {
            assert!(key.validate(&string(&value)).is_err(), "{path}: {value:?}");
        }
    }
    let value = field(&load_balancer, "load_balancer_attributes.value");
    for text in ["".to_string(), "a".repeat(1024), "é".repeat(1024)] {
        value.validate(&string(&text)).unwrap();
    }
    assert!(value.validate(&string(&"a".repeat(1025))).is_err());
}

#[test]
fn health_check_paths_jwt_issuer_and_claim_values_enforce_only_documented_lengths() {
    let listener = listener_schema();
    let target_group = target_group_schema();
    for (schema, path, min, max) in [
        (&target_group, "health_check_path", 1, 1024),
        (
            &listener,
            "default_actions.jwt_validation_config.issuer",
            0,
            256,
        ),
        (
            &listener,
            "default_actions.jwt_validation_config.additional_claims.values",
            0,
            256,
        ),
    ] {
        let attr_type = field(schema, path);
        let attr_type = attr_type.list_element().unwrap_or(attr_type);
        for value in ["a".repeat(min), "a".repeat(max), "é".repeat(max)] {
            attr_type.validate(&string(&value)).unwrap();
        }
        assert!(
            attr_type.validate(&string(&"a".repeat(max + 1))).is_err(),
            "{path}"
        );
        if min > 0 {
            assert!(attr_type.validate(&string("")).is_err(), "{path}");
        }
    }
    field(&listener, "default_actions.jwt_validation_config.issuer")
        .validate(&string("issuer-without-https"))
        .unwrap();
}

#[test]
fn oidc_urls_and_jwks_endpoint_reuse_https_url_without_redefining_the_builtin() {
    let schema = listener_schema();
    for path in [
        "default_actions.authenticate_oidc_config.authorization_endpoint",
        "default_actions.authenticate_oidc_config.issuer",
        "default_actions.authenticate_oidc_config.token_endpoint",
        "default_actions.authenticate_oidc_config.user_info_endpoint",
        "default_actions.jwt_validation_config.jwks_endpoint",
    ] {
        let endpoint = field(&schema, path);
        assert!(
            matches!(endpoint.as_type().raw_shape(), RawShape::String {
            identity: Some(identity), pattern: None, length: None, ..
        } if identity.to_string() == "HttpsUrl"),
            "{path}"
        );
        for value in [
            "https://idp.example.com/path".to_string(),
            format!("https://idp.example.com/{}", "a".repeat(300)),
        ] {
            endpoint.validate(&string(&value)).unwrap();
        }
        for value in [
            "",
            "http://idp.example.com/path",
            "idp.example.com/path",
            "https://",
        ] {
            assert!(
                endpoint.validate(&string(value)).is_err(),
                "{path}: {value:?}"
            );
        }
    }
}

#[test]
fn load_balancer_dns_and_subnet_addresses_reuse_core_builtins_unchanged() {
    let schema = load_balancer_schema();
    for (path, identity, accepted, rejected) in [
        (
            "dns_name",
            "DomainName",
            &[
                "alb.example.com",
                "registry-alb-123.ap-northeast-1.elb.amazonaws.com",
            ][..],
            &["https://alb.example.com", "a b.example.com", ""][..],
        ),
        (
            "subnet_mappings.private_i_pv4_address",
            "Ipv4Address",
            &["10.0.1.1", "8.8.8.8", "0.0.0.0", "255.255.255.255"][..],
            &["256.1.1.1", "10.0.1.1/24", "::1", ""][..],
        ),
        (
            "subnet_mappings.i_pv6_address",
            "Ipv6Address",
            &[
                "::",
                "::1",
                "2001:db8::1",
                "ffff:ffff:ffff:ffff:ffff:ffff:ffff:ffff",
            ][..],
            &["2001:db8::/80", "2001:gggg::1", "10.0.0.1", ""][..],
        ),
    ] {
        let attr_type = field(&schema, path);
        assert!(
            matches!(attr_type.as_type().raw_shape(), RawShape::String {
            identity: Some(actual), pattern: None, length: None, ..
        } if actual.to_string() == identity),
            "{path}"
        );
        for value in accepted {
            attr_type.validate(&string(value)).unwrap();
        }
        for value in rejected {
            assert!(
                attr_type.validate(&string(value)).is_err(),
                "{path}: {value:?}"
            );
        }
    }
}

#[test]
fn list_bounds_preserve_struct_items_and_cfn_uniqueness() {
    let schema = listener_schema();
    for (path, maximum, make_item) in [
        (
            "certificates",
            1,
            (|i| {
                map([(
                    "certificate_arn",
                    string(&format!(
                        "arn:aws:acm:us-east-1:123456789012:certificate/cert-{i}"
                    )),
                )])
            }) as fn(usize) -> Value,
        ),
        (
            "default_actions.forward_config.target_groups",
            5,
            (|i| {
                map([
                    (
                        "target_group_arn",
                        string(&format!(
                            "arn:aws:elasticloadbalancing:us-east-1:123456789012:targetgroup/group-{i}/abc123"
                        )),
                    ),
                    ("weight", int(i as i64)),
                ])
            }) as fn(usize) -> Value,
        ),
        (
            "default_actions.jwt_validation_config.additional_claims",
            10,
            (|i| {
                map([
                    ("format", enum_identifier("single_string")),
                    ("name", string(&format!("claim_{i}"))),
                    ("values", list(vec![string("value")])),
                ])
            }) as fn(usize) -> Value,
        ),
    ] {
        let attr_type = field(&schema, path);
        for count in [0, 1, maximum] {
            attr_type
                .validate(&list((0..count).map(make_item).collect()))
                .unwrap();
        }
        assert!(
            attr_type
                .validate(&list((0..=maximum).map(make_item).collect()))
                .is_err(),
            "{path}"
        );
        assert!(
            attr_type
                .validate(&list(vec![make_item(0), make_item(0)]))
                .is_err(),
            "{path}: CFN uniqueness"
        );
        assert!(
            attr_type
                .validate(&list(vec![string("flattened")]))
                .is_err(),
            "{path}: struct items"
        );
    }
    let alpn = field(&schema, "alpn_policy");
    alpn.validate(&list(vec![])).unwrap();
    alpn.validate(&list(vec![enum_identifier("http1_only")]))
        .unwrap();
    assert!(
        alpn.validate(&list(vec![
            enum_identifier("http1_only"),
            enum_identifier("http2_only")
        ]))
        .is_err()
    );
    assert!(
        alpn.validate(&list(vec![enum_identifier("not_an_alpn_policy")]))
            .is_err()
    );

    let values = field(
        &schema,
        "default_actions.jwt_validation_config.additional_claims.values",
    );
    for count in [0, 1, 10] {
        values
            .validate(&list(
                (0..count).map(|i| string(&format!("value_{i}"))).collect(),
            ))
            .unwrap();
    }
    assert!(
        values
            .validate(&list(
                (0..11).map(|i| string(&format!("value_{i}"))).collect()
            ))
            .is_err()
    );
    assert!(
        values
            .validate(&list(vec![string("valid"), string(&"a".repeat(257))]))
            .is_err()
    );
    assert!(
        values
            .validate(&list(vec![string("same"), string("same")]))
            .is_err()
    );
}

#[test]
fn target_group_load_balancer_arns_allow_zero_or_one() {
    let schema = target_group_schema();
    let arns = field(&schema, "load_balancer_arns");
    let arn = |i| {
        string(&format!(
            "arn:aws:elasticloadbalancing:us-east-1:123456789012:loadbalancer/app/alb-{i}/abc123"
        ))
    };
    arns.validate(&list(vec![])).unwrap();
    arns.validate(&list(vec![arn(0)])).unwrap();
    assert!(arns.validate(&list(vec![arn(0), arn(1)])).is_err());
}

#[test]
fn load_balancer_attributes_accept_the_22_entries_returned_by_cloud_control() {
    let schema = load_balancer_schema();
    let attributes = field(&schema, "load_balancer_attributes");
    attributes
        .validate(&list(
            (0..22)
                .map(|i| {
                    map([
                        ("key", string(&format!("attribute_{i}.enabled"))),
                        ("value", string("")),
                    ])
                })
                .collect(),
        ))
        .unwrap();
}

#[test]
fn quic_server_ids_require_lowercase_hex_and_exact_length() {
    let schema = target_group_schema();
    let server_id = field(&schema, "targets.quic_server_id");
    for value in [
        "0x0000000000000000",
        "0xffffffffffffffff",
        "0x0123456789abcdef",
    ] {
        server_id.validate(&string(value)).unwrap();
    }
    for value in [
        "",
        "0x0123456789abcde",
        "0x0123456789abcdef0",
        "0x0123456789abcdeF",
        "0X0123456789abcdef",
        "0x0123456789abcdeg",
        "0x0123456789abcdef\n",
    ] {
        assert!(server_id.validate(&string(value)).is_err(), "{value:?}");
    }
}

#[test]
fn target_availability_zone_accepts_all_and_preserves_the_shared_zone_type() {
    let schema = target_group_schema();
    let zone = field(&schema, "targets.availability_zone");
    let RawShape::Union(members) = zone.as_type().raw_shape() else {
        panic!("target AvailabilityZone must be a union");
    };
    assert_eq!(members.len(), 2);
    assert!(
        matches!(members[0].raw_shape(), RawShape::Enum { identity, values: None, .. }
        if identity.to_string() == "aws.AvailabilityZone.ZoneName")
    );
    zone.validate(&enum_identifier("ap_northeast_1a")).unwrap();
    zone.validate(&string("all")).unwrap();
    for value in ["", "ALL", "all\n", "all-zones", "not-a-zone"] {
        assert!(zone.validate(&string(value)).is_err(), "{value:?}");
    }
    let read = aws_value_to_dsl_with_defs(
        "availability_zone",
        &json!("all"),
        zone.as_type(),
        "elasticloadbalancingv2.TargetGroup",
        schema.as_schema().defs(),
    )
    .unwrap();
    let canonical = zone.canonicalize_state(read).into_present().unwrap();
    assert_eq!(
        dsl_value_to_aws_with_defs(
            &canonical,
            zone.as_type(),
            "elasticloadbalancingv2.TargetGroup",
            "availability_zone",
            schema.as_schema().defs(),
        ),
        Some(json!("all"))
    );
}

// Generated named definitions are emitted separately from their inline uses.
// Compare validation metadata recursively (enum identities differ by owning
// path) so the boundary assertions above also cover every registered copy.
fn assert_same_constraints(left: &CompiledAttributeType, right: &CompiledAttributeType) {
    match (left.raw_shape(), right.raw_shape()) {
        (
            RawShape::String {
                identity: li,
                pattern: lp,
                length: ll,
                ..
            },
            RawShape::String {
                identity: ri,
                pattern: rp,
                length: rl,
                ..
            },
        ) => {
            assert_eq!(li, ri);
            assert_eq!(lp, rp);
            assert_eq!(ll, rl);
        }
        (RawShape::Int { range: left, .. }, RawShape::Int { range: right, .. }) => {
            assert_eq!(left, right)
        }
        (
            RawShape::Enum {
                values: left,
                dsl_aliases: la,
                base: lb,
                ..
            },
            RawShape::Enum {
                values: right,
                dsl_aliases: ra,
                base: rb,
                ..
            },
        ) => {
            assert_eq!(left, right);
            assert_eq!(la, ra);
            assert_same_constraints(lb, rb);
        }
        (
            RawShape::List {
                element_type: left,
                constraints: lc,
                ordered: lo,
            },
            RawShape::List {
                element_type: right,
                constraints: rc,
                ordered: ro,
            },
        ) => {
            assert_eq!(lc, rc);
            assert_eq!(lo, ro);
            assert_same_constraints(left, right);
        }
        (
            RawShape::Map {
                key: lk,
                value: lv,
                constraints: lc,
            },
            RawShape::Map {
                key: rk,
                value: rv,
                constraints: rc,
            },
        ) => {
            assert_eq!(lc, rc);
            assert_same_constraints(lk, rk);
            assert_same_constraints(lv, rv);
        }
        (RawShape::Struct { fields: left, .. }, RawShape::Struct { fields: right, .. }) => {
            assert_eq!(left.len(), right.len());
            for (left, right) in left.iter().zip(right) {
                assert_eq!(left.name(), right.name());
                assert_same_constraints(left.field_type(), right.field_type());
            }
        }
        (RawShape::Union(left), RawShape::Union(right)) => {
            assert_eq!(left.len(), right.len());
            for (left, right) in left.iter().zip(right.iter()) {
                assert_same_constraints(left, right);
            }
        }
        (RawShape::Bool, RawShape::Bool) => {}
        (left, right) => panic!("different validation shapes: {left:?} / {right:?}"),
    }
}

#[test]
fn every_registered_definition_and_nested_copy_retains_the_inline_constraints() {
    fn visit(
        schema: &ValidatedResourceSchema,
        attr_type: &CompiledAttributeType,
        comparisons: &mut usize,
    ) {
        match attr_type.raw_shape() {
            RawShape::Struct { name, fields, .. } => {
                if let Some(definition) = schema.as_schema().defs().get(name) {
                    assert_same_constraints(attr_type, definition);
                    *comparisons += 1;
                }
                for field in fields {
                    visit(schema, field.field_type(), comparisons);
                }
            }
            RawShape::List { element_type, .. } => visit(schema, element_type, comparisons),
            RawShape::Map { value, .. } => visit(schema, value, comparisons),
            _ => {}
        }
    }
    for schema in [
        listener_schema(),
        load_balancer_schema(),
        target_group_schema(),
    ] {
        let mut comparisons = 0;
        for attribute in schema.effective_attributes() {
            visit(&schema, attribute.value_type().as_type(), &mut comparisons);
        }
        assert!(comparisons > 0, "registered definitions must be exercised");
        // Include nested occurrences inside registered definitions as well.
        for definition in schema.as_schema().defs().values() {
            visit(&schema, definition, &mut comparisons);
        }
    }
}

#[test]
fn target_group_arn_references_validate_in_forward_config_with_refinements() {
    let assignment = AssignmentContext::new();
    let target_group = target_group_schema();
    let listener = listener_schema();
    let source = field(&target_group, "target_group_arn");
    let sink = field(
        &listener,
        "default_actions.forward_config.target_groups.target_group_arn",
    );
    assert!(
        target_group
            .as_schema()
            .type_in_schema(source.as_type())
            .is_assignable_to(assignment.sink(&listener, sink.as_type()))
    );
    validate_fixtures(&[(
        "elbv2_references.crn",
        r#"
provider awscc {
  region = awscc.Region.ap_northeast_1
}
let target_group = awscc.elasticloadbalancingv2.TargetGroup {
  name = 'registry-tg'
  protocol = http
  port = 80
  health_check_protocol = http
  target_type = ip
  vpc_id = 'vpc-dddd4444'
  target {
    id = '10.0.1.10'
    availability_zone = 'all'
    port = 80
  }
}
awscc.elasticloadbalancingv2.Listener {
  load_balancer_arn = 'arn:aws:elasticloadbalancing:ap-northeast-1:123456789012:loadbalancer/app/registry/abc123'
  port = 80
  protocol = http
  default_action {
    type = forward
    order = 1
    forward_config = {
      target_groups = [{ target_group_arn = target_group.target_group_arn, weight = 999 }]
      target_group_stickiness_config = { enabled = true, duration_seconds = 604800 }
    }
  }
}
"#,
    )]);
}

#[test]
fn every_example_and_acceptance_fixture_declaring_elbv2_resources_validates() {
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
                if source.contains("awscc.elasticloadbalancingv2.") {
                    // The OIDC runner imports an ACM certificate and substitutes
                    // this marker before invoking Carina (run.sh). Supply a
                    // valid representative ARN for the same offline validation.
                    let source = source.replace(
                        "__ACM_CERTIFICATE_ARN__",
                        "arn:aws:acm:ap-northeast-1:123456789012:certificate/00000000-0000-0000-0000-000000000001",
                    );
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
        "the ELBv2 fixture inventory must not be empty"
    );
    let cases = fixtures
        .iter()
        .map(|(path, source)| (path.as_str(), source.as_str()))
        .collect::<Vec<_>>();
    validate_fixtures(&cases);
}
