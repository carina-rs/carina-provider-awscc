//! Verifies that AWSCC hosted-zone ID producers carry the shared Route 53
//! identity used by the AWS provider's Route53 alias-target sink.

use carina_core::resource::{ConcreteValue, Value};
use carina_core::schema::{AttributeType, Schema, TypeInSchema};
use carina_provider_awscc::schemas::generated::elasticloadbalancingv2::load_balancer::elasticloadbalancingv2_load_balancer_config;
use carina_provider_awscc::schemas::generated::route53::hosted_zone::route53_hosted_zone_config;

#[test]
fn hosted_zone_id_producers_are_assignable_to_alias_target_union_and_string() {
    let alias_target_hosted_zone_id = AttributeType::union(vec![
        carina_aws_types::cloudfront_hosted_zone_id(),
        carina_aws_types::route53_hosted_zone_id(),
    ]);
    let plain_string = AttributeType::string();

    for (label, config, attribute_name) in [
        (
            "LoadBalancer.canonical_hosted_zone_id",
            elasticloadbalancingv2_load_balancer_config(),
            "canonical_hosted_zone_id",
        ),
        ("HostedZone.id", route53_hosted_zone_config(), "id"),
    ] {
        let source = &config
            .schema
            .attributes
            .get(attribute_name)
            .unwrap_or_else(|| panic!("{label} must exist"))
            .attr_type;

        assert!(
            config
                .schema
                .type_in_schema(source)
                .is_assignable_to(TypeInSchema::schemaless(&alias_target_hosted_zone_id)),
            "{label} must be assignable to the AWS Route53 alias-target union"
        );
        assert!(
            config
                .schema
                .type_in_schema(source)
                .is_assignable_to(TypeInSchema::schemaless(&plain_string)),
            "{label} must remain assignable to plain String sinks"
        );
    }
}

#[test]
fn hosted_zone_id_producers_validate_route53_values() {
    for (label, config, attribute_name) in [
        (
            "LoadBalancer.canonical_hosted_zone_id",
            elasticloadbalancingv2_load_balancer_config(),
            "canonical_hosted_zone_id",
        ),
        ("HostedZone.id", route53_hosted_zone_config(), "id"),
    ] {
        let producer_type = config
            .schema
            .attributes
            .get(attribute_name)
            .unwrap_or_else(|| panic!("{label} must exist"))
            .attr_type
            .clone();
        let schema = Schema::flat(producer_type);

        for hosted_zone_id in [
            "Z35SXDOTRQ7X7K",
            "Z14GRHDCWA56QT",
            "Z2P70J7HTTTPLU",
            "Z05136711ZXUHBDOA8D5O",
        ] {
            let result = schema.validate(&Value::Concrete(ConcreteValue::String(
                hosted_zone_id.to_string(),
            )));
            assert!(
                result.is_ok(),
                "{label} must accept {hosted_zone_id}: {result:?}"
            );
        }

        for hosted_zone_id in ["", "/hostedzone/Z05136711ZXUHBDOA8D5O"] {
            assert!(
                schema
                    .validate(&Value::Concrete(ConcreteValue::String(
                        hosted_zone_id.to_string()
                    )))
                    .is_err(),
                "{label} must reject {hosted_zone_id:?}"
            );
        }
    }
}
