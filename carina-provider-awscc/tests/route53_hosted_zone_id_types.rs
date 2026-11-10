//! Verifies that AWSCC hosted-zone ID producers carry the shared Route 53
//! identity layer. HostedZone.Id adds CFN's 32-character member maximum;
//! LoadBalancer.CanonicalHostedZoneID has no CFN member constraints.

use std::sync::Arc;

use carina_core::provider::provider_custom_types_from_factories;
use carina_core::resource::{ConcreteValue, Value};
use carina_core::schema::{
    AttributeSchema, AttributeType, RawShape, ResourceSchema, SchemaKind, SchemaRegistry,
    StringIdentityParts, StringMemberParts,
};
use carina_provider_awscc::AwsccProviderFactory;
use carina_provider_awscc::schemas::generated::elasticloadbalancingv2::load_balancer::elasticloadbalancingv2_load_balancer_config;
use carina_provider_awscc::schemas::generated::route53::hosted_zone::route53_hosted_zone_config;

#[test]
fn hosted_zone_id_producers_share_catalog_identity_and_keep_cfn_member_bounds() {
    let alias_target_hosted_zone_id = AttributeType::union(vec![
        carina_aws_types::cloudfront_hosted_zone_id(),
        carina_aws_types::route53_hosted_zone_id(),
    ]);
    let sinks = ResourceSchema::new("test.Route53Sinks")
        .attribute(AttributeSchema::optional("alias_target", alias_target_hosted_zone_id).build())
        .attribute(AttributeSchema::optional("plain_string", AttributeType::string()).build())
        .validate()
        .expect("test sinks must pass admission");
    let mut registry = SchemaRegistry::new();
    registry.insert("awscc", sinks);
    let owner = registry
        .get("awscc", "test.Route53Sinks", SchemaKind::Resource)
        .expect("test sink schema must be registered");
    let catalog =
        provider_custom_types_from_factories(Arc::new(vec![Box::new(AwsccProviderFactory)]));
    let alias_target_hosted_zone_id = owner.schema().parts().attributes["alias_target"].attr_type();
    let plain_string = owner.schema().parts().attributes["plain_string"].attr_type();

    for (label, config, attribute_name) in [
        (
            "LoadBalancer.canonical_hosted_zone_id",
            elasticloadbalancingv2_load_balancer_config(),
            "canonical_hosted_zone_id",
        ),
        ("HostedZone.id", route53_hosted_zone_config(), "id"),
    ] {
        let source = config
            .schema
            .as_schema()
            .attributes()
            .get(attribute_name)
            .unwrap_or_else(|| panic!("{label} must exist"))
            .attr_type();

        let RawShape::String {
            identity_layer:
                StringIdentityParts {
                    identity: Some(identity),
                    pattern: Some(pattern),
                    length: Some((None, Some(32))),
                    validate: None,
                    ..
                },
            member_layer:
                StringMemberParts {
                    pattern: None,
                    length,
                },
        } = source.raw_shape()
        else {
            panic!("{label} must preserve the catalog definition and CFN member layer");
        };
        assert_eq!(identity.to_string(), "aws.route53.HostedZone.Id");
        assert_eq!(pattern.as_str(), "^Z[A-Z0-9]+$");
        assert_eq!(length, (attribute_name == "id").then_some((None, Some(32))));
        assert!(
            config
                .schema
                .as_schema()
                .type_in_schema(source)
                .is_assignable_to(owner.assignment_sink(alias_target_hosted_zone_id, &catalog)),
            "{label}: the catalog definition must satisfy the shared hosted-zone ID sink"
        );
        assert!(
            config
                .schema
                .as_schema()
                .type_in_schema(source)
                .is_assignable_to(owner.assignment_sink(plain_string, &catalog)),
            "{label} must remain assignable to plain String sinks"
        );
    }
}

#[test]
fn hosted_zone_id_values_obey_catalog_and_member_bounds() {
    for (label, config, attribute_name) in [
        (
            "LoadBalancer.canonical_hosted_zone_id",
            elasticloadbalancingv2_load_balancer_config(),
            "canonical_hosted_zone_id",
        ),
        ("HostedZone.id", route53_hosted_zone_config(), "id"),
    ] {
        let schema = config.schema;
        let producer_type = schema
            .effective_attribute(attribute_name)
            .unwrap_or_else(|| panic!("{label} must exist"));

        for hosted_zone_id in [
            "Z35SXDOTRQ7X7K",
            "Z14GRHDCWA56QT",
            "Z2P70J7HTTTPLU",
            "Z05136711ZXUHBDOA8D5O",
        ] {
            let value: Value =
                Value::Concrete(ConcreteValue::String(hosted_zone_id.to_string().into()));
            let result = producer_type.validate(&value);
            assert!(
                result.is_ok(),
                "{label} must accept {hosted_zone_id}: {result:?}"
            );
        }

        let maximum = format!("Z{}", "1".repeat(31));
        let value: Value = Value::Concrete(ConcreteValue::String(maximum.clone().into()));
        assert!(producer_type.validate(&value).is_ok(), "{label}: {maximum}");
        for hosted_zone_id in [
            "".to_string(),
            "/hostedzone/Z1".to_string(),
            format!("{maximum}1"),
        ] {
            let value: Value =
                Value::Concrete(ConcreteValue::String(hosted_zone_id.to_string().into()));
            assert!(
                producer_type.validate(&value).is_err(),
                "{label}: the shared identity must reject {hosted_zone_id:?}"
            );
        }
    }
}
