use carina_core::executor::prepare_provider_ready_resource;
use carina_core::resource::Resource;
use carina_core::schema::RawShape;
use serde_json::json;

use crate::provider::conversion::{aws_value_to_dsl_with_defs, dsl_value_to_aws_with_defs};
use crate::test_support::{
    enum_identifier, field, int, string, validate_fixtures, with_preparation,
};

#[test]
fn documented_enums_validate_and_round_trip_exact_aws_wire_strings() {
    let topic = super::generated::sns::topic::sns_topic_config().schema;
    let subscription = super::generated::sns::subscription::sns_subscription_config().schema;
    for (resource_type, schema, path, expected) in [
        (
            "sns.Subscription",
            &subscription,
            "protocol",
            &[
                ("http", "http"),
                ("https", "https"),
                ("email", "email"),
                ("email-json", "email_json"),
                ("sms", "sms"),
                ("sqs", "sqs"),
                ("application", "application"),
                ("lambda", "lambda"),
                ("firehose", "firehose"),
            ][..],
        ),
        (
            "sns.Subscription",
            &subscription,
            "filter_policy_scope",
            &[
                ("MessageAttributes", "message_attributes"),
                ("MessageBody", "message_body"),
            ][..],
        ),
        (
            "sns.Topic",
            &topic,
            "fifo_throughput_scope",
            &[("Topic", "topic"), ("MessageGroup", "message_group")][..],
        ),
        (
            "sns.Topic",
            &topic,
            "tracing_config",
            &[("PassThrough", "pass_through"), ("Active", "active")][..],
        ),
        (
            "sns.Topic",
            &topic,
            "delivery_status_logging.protocol",
            &[
                ("http/s", "http_s"),
                ("sqs", "sqs"),
                ("lambda", "lambda"),
                ("firehose", "firehose"),
                ("application", "application"),
            ][..],
        ),
    ] {
        let attr_type = field(schema, path);
        let (_, values, _, _, spellings) = attr_type.as_type().enum_parts().unwrap();
        assert_eq!(
            values
                .unwrap()
                .iter()
                .map(String::as_str)
                .collect::<Vec<_>>(),
            expected.iter().map(|(wire, _)| *wire).collect::<Vec<_>>(),
            "{resource_type}.{path}"
        );
        if path == "delivery_status_logging.protocol" {
            assert_eq!(
                super::generated::get_enum_valid_values(resource_type, "protocol").unwrap(),
                expected.iter().map(|(wire, _)| *wire).collect::<Vec<_>>(),
                "the omitted Subscription definition must not replace the logging lookup"
            );
        }
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
            assert!(
                attr_type.validate(&string(wire)).is_err(),
                "{path}: quoted enum"
            );

            let read = aws_value_to_dsl_with_defs(
                path,
                &json!(wire),
                attr_type.as_type(),
                resource_type,
                schema.as_schema().defs(),
            )
            .unwrap();
            let canonical = attr_type.canonicalize_state(read).into_present().unwrap();
            assert_eq!(
                dsl_value_to_aws_with_defs(
                    &canonical,
                    attr_type.as_type(),
                    resource_type,
                    path,
                    schema.as_schema().defs(),
                ),
                Some(json!(wire)),
                "{path}: read-back must preserve the AWS wire string"
            );
        }
        assert!(
            attr_type
                .validate(&enum_identifier("invalid_sns_enum"))
                .is_err()
        );
    }
}

#[tokio::test]
async fn authored_email_json_and_string_signature_version_serialize_as_wire_strings() {
    for (resource_type, path, value, wire) in [
        (
            "sns.Subscription",
            "protocol",
            enum_identifier("email_json"),
            "email-json",
        ),
        ("sns.Topic", "signature_version", string("2"), "2"),
    ] {
        let mut resource = Resource::with_provider("awscc", resource_type, "wire_test", None)
            .with_attribute(path, value);
        if resource_type == "sns.Subscription" {
            resource = resource
                .with_attribute(
                    "topic_arn",
                    string("arn:aws:sns:ap-northeast-1:123456789012:alerts"),
                )
                .with_attribute("endpoint", string("alerts@example.com"));
        }
        with_preparation(resource_type, async |preparation| {
            let ready = prepare_provider_ready_resource(resource, preparation)
                .await
                .unwrap();
            let schema = &super::generated::get_config_by_type(resource_type)
                .unwrap()
                .schema;
            assert_eq!(
                dsl_value_to_aws_with_defs(
                    &ready.as_resource().attributes[path],
                    field(schema, path).as_type(),
                    resource_type,
                    path,
                    schema.as_schema().defs(),
                ),
                Some(json!(wire)),
                "{path}: authored {wire} must serialize as a JSON string"
            );
        })
        .await;
    }
}

#[test]
fn plain_strings_accept_arn_and_identity_typed_references() {
    let topic = super::generated::sns::topic::sns_topic_config().schema;
    let subscription = super::generated::sns::subscription::sns_subscription_config().schema;
    let vpc = super::generated::ec2::vpc::ec2_vpc_config().schema;
    for (schema, path) in [
        (&topic, "topic_name"),
        (&topic, "signature_version"),
        (
            &topic,
            "delivery_status_logging.success_feedback_sample_rate",
        ),
        (&topic, "display_name"),
        (&subscription, "endpoint"),
    ] {
        let sink = field(schema, path);
        assert!(
            matches!(
                sink.as_type().raw_shape(),
                RawShape::String {
                    identity: None,
                    pattern: None,
                    length: None,
                    validate: None,
                    to_dsl: None,
                }
            ),
            "{path} must remain plain String"
        );
        // Anonymous pattern/length refinements reject identity-typed resource
        // outputs until carina-rs/carina#4230; check assignability across schemas.
        for (source_schema, source_path) in [(&topic, "topic_arn"), (&vpc, "vpc_id")] {
            let source = field(source_schema, source_path);
            assert!(matches!(
                source.as_type().raw_shape(),
                RawShape::String {
                    identity: Some(_),
                    ..
                }
            ));
            assert!(
                source_schema
                    .as_schema()
                    .type_in_schema(source.as_type())
                    .is_assignable_to(schema.as_schema().type_in_schema(sink.as_type())),
                "{source_path} must be assignable to {path}"
            );
        }
    }
    let sample_rate = field(
        &topic,
        "delivery_status_logging.success_feedback_sample_rate",
    );
    for value in ["0", "100"] {
        sample_rate.validate(&string(value)).unwrap();
    }
    assert!(sample_rate.validate(&int(100)).is_err());
}

#[test]
fn existing_maximum_message_size_bounds_remain_enforced() {
    let topic = super::generated::sns::topic::sns_topic_config().schema;
    let size = field(&topic, "maximum_message_size");
    for value in [1024, 1048576] {
        size.validate(&int(value)).unwrap();
    }
    for value in [1023, 1048577] {
        assert!(size.validate(&int(value)).is_err());
    }
}

#[test]
fn sns_examples_acceptance_fixtures_and_enum_dsl_forms_validate() {
    let cases = [
        (
            "examples/sns_topic/main.crn",
            include_str!("../../../examples/sns_topic/main.crn"),
        ),
        (
            "examples/sns_subscription/main.crn",
            include_str!("../../../examples/sns_subscription/main.crn"),
        ),
        (
            "acceptance-tests/sns_subscription_policies/step1.crn",
            include_str!("../../../acceptance-tests/sns_subscription_policies/step1.crn"),
        ),
        (
            "acceptance-tests/sns_subscription_policies/step2.crn",
            include_str!("../../../acceptance-tests/sns_subscription_policies/step2.crn"),
        ),
        (
            "acceptance-tests/sns_subscription_policies/step3.crn",
            include_str!("../../../acceptance-tests/sns_subscription_policies/step3.crn"),
        ),
        (
            "enum_forms.crn",
            r#"
provider awscc {
  region = awscc.Region.ap_northeast_1
}
let standard = awscc.sns.Topic {
  topic_name = 'signature-one'
  signature_version = '1'
  tracing_config = pass_through
}
let fifo = awscc.sns.Topic {
  topic_name = 'signature-two.fifo'
  fifo_topic = true
  fifo_throughput_scope = message_group
  signature_version = '2'
}
let first = awscc.sns.Subscription {
  topic_arn = standard.topic_arn
  protocol = email_json
  filter_policy_scope = message_body
  endpoint = 'alerts@example.com'
}
let second = awscc.sns.Subscription {
  topic_arn = fifo.topic_arn
  protocol = aws.sns.Subscription.Protocol.sqs
  filter_policy_scope = aws.sns.Subscription.FilterPolicyScope.message_attributes
  endpoint = 'arn:aws:sqs:ap-northeast-1:123456789012:alerts.fifo'
}
"#,
        ),
    ];
    // The acceptance runner supplies these values from real queues and time.
    let rendered: Vec<_> = cases
        .iter()
        .map(|(path, source)| {
            let mut source = source.to_string();
            for placeholder in [
                "__STANDARD_QUEUE_ARN__",
                "__STANDARD_DLQ_ARN__",
                "__FIFO_QUEUE_ARN__",
                "__FIFO_DLQ_ARN__",
            ] {
                source = source.replace(
                    placeholder,
                    "arn:aws:sqs:ap-northeast-1:123456789012:fixture",
                );
            }
            (
                *path,
                source.replace("__REPLAY_STARTING_POINT__", "2026-10-31T00:00:00Z"),
            )
        })
        .collect();
    validate_fixtures(
        &rendered
            .iter()
            .map(|(path, source)| (*path, source.as_str()))
            .collect::<Vec<_>>(),
    );
}
