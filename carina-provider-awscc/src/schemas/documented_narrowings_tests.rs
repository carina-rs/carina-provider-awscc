use crate::provider::conversion::{aws_value_to_dsl_with_defs, dsl_value_to_aws_with_defs};
use crate::test_support::{enum_identifier, field, int, string, validate_fixtures};
use carina_core::schema::ValidatedResourceSchema;
use serde_json::json;

fn schema(name: &str) -> ValidatedResourceSchema {
    super::generated::get_config_by_type(name)
        .unwrap()
        .schema
        .clone()
}

#[test]
fn iam_single_and_list_statements_keep_typed_conditions_and_wire_names() {
    let schema = schema("iam.RolePolicy");
    let policy = field(&schema, "policy_document");
    for statement in [
        json!({"Effect":"Allow", "Action":["s3:GetObject"], "Resource":["*"],
            "Condition":{"Bool":{"aws:SecureTransport":true}, "NumericLessThan":{"s3:max-keys":10}}}),
        json!([{"Effect":"Deny", "NotAction":["s3:GetObject"], "NotResource":["*"],
            "Condition":{"Bool":{"aws:SecureTransport":[false,true]}}}]),
    ] {
        let wire = json!({"Version":"2012-10-17", "Statement":statement});
        let value = aws_value_to_dsl_with_defs(
            "policy_document",
            &wire,
            policy.as_type(),
            "iam.RolePolicy",
            schema.as_schema().defs(),
        )
        .unwrap();
        let value = policy.canonicalize_state(value).into_present().unwrap();
        policy.validate(&value).unwrap();
        assert_eq!(
            dsl_value_to_aws_with_defs(
                &value,
                policy.as_type(),
                "iam.RolePolicy",
                "policy_document",
                schema.as_schema().defs()
            ),
            Some(wire)
        );
    }
}

#[test]
fn iam_existing_version_enum_and_statement_block_syntax_validate() {
    validate_fixtures(&[(
        "documented_iam.crn",
        r#"
provider awscc { region = awscc.Region.ap_northeast_1 }
awscc.iam.Role {
  role_name = 'documented-policy'
  assume_role_policy_document = {
    version = aws.iam.PolicyDocument.Version.2012_10_17
    statement {
      effect = allow
      principal = { service = 'ec2.amazonaws.com' }
      action = 'sts:AssumeRole'
    }
  }
}
"#,
    )]);
}

#[test]
fn security_group_protocols_include_named_aliases_and_every_protocol_number() {
    for (resource, path) in [
        ("ec2.SecurityGroupIngress", "ip_protocol"),
        ("ec2.SecurityGroupEgress", "ip_protocol"),
        ("ec2.SecurityGroup", "security_group_ingress.ip_protocol"),
        ("ec2.SecurityGroup", "security_group_egress.ip_protocol"),
    ] {
        let schema = schema(resource);
        let protocol = field(&schema, path);
        for value in ["tcp", "udp", "icmp", "icmpv6", "all"] {
            protocol.validate(&enum_identifier(value)).unwrap();
        }
        for number in 0..=255 {
            protocol
                .validate(&enum_identifier(&number.to_string()))
                .unwrap();
            let read = protocol
                .canonicalize_state(string(&number.to_string()))
                .into_present()
                .unwrap();
            protocol.validate(&read).unwrap();
        }
        for value in ["-2", "256", "garbage"] {
            assert!(
                protocol.validate(&string(value)).is_err(),
                "{resource}.{path}: {value}"
            );
        }
        assert!(protocol.validate(&int(6)).is_err());
    }
    validate_fixtures(&[(
        "documented_protocol_number.crn",
        r#"
provider awscc { region = awscc.Region.ap_northeast_1 }
awscc.ec2.SecurityGroupIngress {
  group_id = 'sg-0123456789abcdef0'
  ip_protocol = aws.ec2.SecurityGroupIngress.IpProtocol.6
  from_port = 80
  to_port = 80
  cidr_ip = '10.0.0.0/8'
}
"#,
    )]);
}

#[test]
fn cloudfront_custom_error_codes_follow_the_documented_supported_set() {
    let schema = schema("cloudfront.Distribution");
    let error_code = field(
        &schema,
        "distribution_config.custom_error_responses.error_code",
    );
    let supported = [400, 403, 404, 405, 414, 416, 500, 501, 502, 503, 504];
    for code in 399..=600 {
        assert_eq!(
            error_code.validate(&int(code)).is_ok(),
            supported.contains(&code),
            "{code}"
        );
    }
}
