//! record_set schema definition for AWS Cloud Control
//!
//! Auto-generated from CloudFormation schema: AWS::Route53::RecordSet
//!
//! DO NOT EDIT MANUALLY - regenerate with carina-codegen

use super::AwsccSchemaConfig;
use carina_core::schema::{AttributeSchema, AttributeType, ResourceSchema, StructField};

/// Returns the schema config for route53_record_set (AWS::Route53::RecordSet)
pub fn route53_record_set_config() -> AwsccSchemaConfig {
    AwsccSchemaConfig {
        aws_type_name: "AWS::Route53::RecordSet",
        resource_type_name: "route53.record_set",
        has_tags: false,
        schema: ResourceSchema::new("awscc.route53.record_set")
            .with_description("Resource Type definition for AWS::Route53::RecordSet")
            .attribute(
                AttributeSchema::new(
                    "alias_target",
                    AttributeType::Struct {
                        name: "AliasTarget".to_string(),
                        fields: vec![
                            StructField::new("dns_name", AttributeType::String)
                                .required()
                                .with_provider_name("DNSName"),
                            StructField::new("evaluate_target_health", AttributeType::Bool)
                                .with_provider_name("EvaluateTargetHealth"),
                            StructField::new("hosted_zone_id", AttributeType::String)
                                .required()
                                .with_provider_name("HostedZoneId"),
                        ],
                    },
                )
                .with_provider_name("AliasTarget"),
            )
            .attribute(
                AttributeSchema::new(
                    "cidr_routing_config",
                    AttributeType::Struct {
                        name: "CidrRoutingConfig".to_string(),
                        fields: vec![
                            StructField::new("collection_id", AttributeType::String)
                                .required()
                                .with_provider_name("CollectionId"),
                            StructField::new("location_name", AttributeType::String)
                                .required()
                                .with_provider_name("LocationName"),
                        ],
                    },
                )
                .with_provider_name("CidrRoutingConfig"),
            )
            .attribute(
                AttributeSchema::new("comment", AttributeType::String)
                    .with_provider_name("Comment"),
            )
            .attribute(
                AttributeSchema::new("failover", AttributeType::String)
                    .with_provider_name("Failover"),
            )
            .attribute(
                AttributeSchema::new(
                    "geo_location",
                    AttributeType::Struct {
                        name: "GeoLocation".to_string(),
                        fields: vec![
                            StructField::new("continent_code", AttributeType::String)
                                .with_provider_name("ContinentCode"),
                            StructField::new("country_code", AttributeType::String)
                                .with_provider_name("CountryCode"),
                            StructField::new("subdivision_code", AttributeType::String)
                                .with_provider_name("SubdivisionCode"),
                        ],
                    },
                )
                .with_provider_name("GeoLocation"),
            )
            .attribute(
                AttributeSchema::new(
                    "geo_proximity_location",
                    AttributeType::Struct {
                        name: "GeoProximityLocation".to_string(),
                        fields: vec![
                            StructField::new("aws_region", super::awscc_region())
                                .with_provider_name("AWSRegion"),
                            StructField::new("bias", AttributeType::Int).with_provider_name("Bias"),
                            StructField::new(
                                "coordinates",
                                AttributeType::Struct {
                                    name: "Coordinates".to_string(),
                                    fields: vec![
                                        StructField::new("latitude", AttributeType::String)
                                            .required()
                                            .with_provider_name("Latitude"),
                                        StructField::new("longitude", AttributeType::String)
                                            .required()
                                            .with_provider_name("Longitude"),
                                    ],
                                },
                            )
                            .with_provider_name("Coordinates"),
                            StructField::new("local_zone_group", AttributeType::String)
                                .with_provider_name("LocalZoneGroup"),
                        ],
                    },
                )
                .with_provider_name("GeoProximityLocation"),
            )
            .attribute(
                AttributeSchema::new("health_check_id", AttributeType::String)
                    .with_provider_name("HealthCheckId"),
            )
            .attribute(
                AttributeSchema::new("hosted_zone_id", AttributeType::String)
                    .create_only()
                    .with_provider_name("HostedZoneId"),
            )
            .attribute(
                AttributeSchema::new("hosted_zone_name", AttributeType::String)
                    .create_only()
                    .with_provider_name("HostedZoneName"),
            )
            .attribute(
                AttributeSchema::new("id", AttributeType::String)
                    .read_only()
                    .with_provider_name("Id"),
            )
            .attribute(
                AttributeSchema::new("multi_value_answer", AttributeType::Bool)
                    .with_provider_name("MultiValueAnswer"),
            )
            .attribute(
                AttributeSchema::new("name", AttributeType::String)
                    .required()
                    .create_only()
                    .with_provider_name("Name"),
            )
            .attribute(
                AttributeSchema::new("region", super::awscc_region()).with_provider_name("Region"),
            )
            .attribute(
                AttributeSchema::new(
                    "resource_records",
                    AttributeType::list(AttributeType::String),
                )
                .with_provider_name("ResourceRecords"),
            )
            .attribute(
                AttributeSchema::new("set_identifier", AttributeType::String)
                    .with_provider_name("SetIdentifier"),
            )
            .attribute(AttributeSchema::new("ttl", AttributeType::String).with_provider_name("TTL"))
            .attribute(
                AttributeSchema::new("type", AttributeType::String)
                    .required()
                    .identity()
                    .with_provider_name("Type"),
            )
            .attribute(
                AttributeSchema::new("weight", AttributeType::Int).with_provider_name("Weight"),
            ),
    }
}

/// Returns the resource type name and all enum valid values for this module
pub fn enum_valid_values() -> (
    &'static str,
    &'static [(&'static str, &'static [&'static str])],
) {
    ("route53.record_set", &[])
}

/// Maps DSL alias values back to canonical AWS values for this module.
/// e.g., ("ip_protocol", "all") -> Some("-1")
pub fn enum_alias_reverse(attr_name: &str, value: &str) -> Option<&'static str> {
    let _ = (attr_name, value);
    None
}

/// Returns all enum alias entries as (attr_name, alias, canonical) tuples.
pub fn enum_alias_entries() -> &'static [(&'static str, &'static str, &'static str)] {
    &[]
}
