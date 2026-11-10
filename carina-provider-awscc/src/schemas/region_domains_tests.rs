use carina_core::provider::ProviderFactory;
use carina_core::schema::RawShape;
use carina_core::schema::StringIdentityParts;
use serde_json::json;

use crate::provider::conversion::{aws_value_to_dsl_with_defs, dsl_value_to_aws_with_defs};
use crate::test_support::{
    enum_identifier, field, string, try_validate_fixtures, validate_fixtures,
};

// In the order published by both sources:
// https://docs.aws.amazon.com/Route53/latest/APIReference/API_VPC.html
// https://docs.aws.amazon.com/AWSCloudFormation/latest/TemplateReference/aws-properties-route53-hostedzone-vpc.html
const VPC_REGIONS: &str = "us-east-1 | us-east-2 | us-west-1 | us-west-2 | eu-west-1 | eu-west-2 | eu-west-3 | eu-central-1 | eu-central-2 | ap-east-1 | me-south-1 | us-gov-west-1 | us-gov-east-1 | us-iso-east-1 | us-iso-west-1 | us-isob-east-1 | me-central-1 | ap-southeast-1 | ap-southeast-2 | ap-southeast-3 | ap-south-1 | ap-south-2 | ap-northeast-1 | ap-northeast-2 | ap-northeast-3 | eu-north-1 | sa-east-1 | ca-central-1 | cn-north-1 | cn-northwest-1 | af-south-1 | eu-south-1 | eu-south-2 | ap-southeast-4 | il-central-1 | ca-west-1 | ap-southeast-5 | mx-central-1 | us-isof-south-1 | us-isof-east-1 | ap-southeast-7 | ap-east-2 | eu-isoe-west-1 | ap-southeast-6 | us-isob-west-1";

fn hosted_zone_source(region: &str) -> String {
    format!(
        r#"
provider awscc {{ region = awscc.Region.ap_northeast_1 }}
awscc.route53.HostedZone {{
  name = 'example.com.'
  vp_c {{
    vpc_id = 'vpc-12345678'
    vpc_region = {region}
  }}
}}
"#
    )
}

#[test]
fn vpc_region_has_exact_documented_enum_and_round_trips_all_45_values() {
    let schema = super::generated::route53::hosted_zone::route53_hosted_zone_config().schema;
    let region = field(&schema, "vp_cs.vpc_region");
    let (identity, values, _, _, spellings) = region.as_type().enum_parts().unwrap();
    assert_eq!(identity.to_string(), "aws.route53.HostedZone.VPCRegion");
    let expected = VPC_REGIONS.split(" | ").collect::<Vec<_>>();
    assert_eq!(expected.len(), 45);
    assert_eq!(
        values
            .unwrap()
            .iter()
            .map(String::as_str)
            .collect::<Vec<_>>(),
        expected
    );
    assert_eq!(
        super::generated::get_enum_valid_values("route53.HostedZone", "vpc_region").unwrap(),
        expected
    );
    for wire in expected {
        let dsl = wire.replace('-', "_");
        assert_eq!(spellings.dsl_for(wire), dsl);
        assert_eq!(spellings.api_for(&dsl), wire);
        region.validate(&enum_identifier(&dsl)).unwrap();
        let read = aws_value_to_dsl_with_defs(
            &crate::test_support::diagnostics(),
            "vpc_region",
            &json!(wire),
            region.as_type(),
            "route53.HostedZone",
            schema.as_schema().defs(),
        )
        .unwrap();
        let canonical = region.canonicalize_state(read).into_present().unwrap();
        assert_eq!(
            dsl_value_to_aws_with_defs(
                &crate::test_support::diagnostics(),
                &canonical,
                region.as_type(),
                "route53.HostedZone",
                "vpc_region",
                schema.as_schema().defs(),
            ),
            Some(json!(wire))
        );
    }
    assert!(
        region
            .validate(&enum_identifier("us_not_a_region_1"))
            .is_err()
    );
}

#[test]
fn vpc_region_accepts_iso_and_commercial_regions_in_dsl() {
    for region in [
        "us_isob_east_1",
        "aws.route53.HostedZone.VPCRegion.us_isob_east_1",
        "ap_northeast_1",
        "VPCRegion.ap_northeast_1",
        "aws.route53.HostedZone.VPCRegion.ap_northeast_1",
    ] {
        validate_fixtures(&[("vpc_region.crn", &hosted_zone_source(region))]);
    }
    assert!(
        try_validate_fixtures(&[(
            "invalid_vpc_region.crn",
            &hosted_zone_source("us_not_a_region_1"),
        )])
        .is_err()
    );
}

#[test]
fn vpc_region_closed_enum_reports_the_previous_shared_types_dsl_migration() {
    // Before #588, the open aws.Region accepted bare ap_northeast_1,
    // Region.ap_northeast_1, aws.Region.ap_northeast_1, and quoted API strings.
    // A normal field-owned closed enum preserves the bare spelling, but uses
    // VPCRegion qualifiers and requires enum identifiers instead of strings.
    // awscc.Region.ap_northeast_1 was already rejected for this resource field.
    for region in [
        "Region.ap_northeast_1",
        "aws.Region.ap_northeast_1",
        "awscc.Region.ap_northeast_1",
        "'ap-northeast-1'",
        "\"ap-northeast-1\"",
    ] {
        let error = try_validate_fixtures(&[("region_spellings.crn", &hosted_zone_source(region))])
            .expect_err("old qualifiers and quoted API strings require migration");
        assert!(error.contains("vpc_region"), "{region}: {error}");
    }
}

#[test]
fn other_region_fields_and_provider_configuration_keep_the_shared_region_domain() {
    for (resource_type, path) in [
        ("ec2.Ipam", "operating_regions.region_name"),
        ("ec2.IpamPool", "source_resource.resource_region"),
        ("ec2.VpcEndpoint", "service_region"),
        ("ec2.VpcPeeringConnection", "peer_region"),
        ("ec2.VpcPeeringConnection", "assume_role_region"),
        ("sns.Subscription", "region"),
        (
            "cloudfront.Distribution",
            "distribution_config.origins.origin_shield.origin_shield_region",
        ),
    ] {
        let schema = &super::generated::get_config_by_type(resource_type)
            .unwrap()
            .schema;
        let region = field(schema, path);
        assert_eq!(
            region.as_type().enum_parts().unwrap().0.to_string(),
            "aws.Region"
        );
        region.validate(&enum_identifier("ap_northeast_1")).unwrap();
        assert!(
            region.validate(&enum_identifier("us_isob_east_1")).is_err(),
            "{path}"
        );
    }
    let config = crate::AwsccProviderFactory.provider_config_attribute_types();
    let region = config.rootless_type("region").unwrap();
    assert_eq!(
        config
            .parts()
            .get("region")
            .unwrap()
            .enum_parts()
            .unwrap()
            .0
            .to_string(),
        "aws.Region"
    );
    region.validate(&enum_identifier("ap_northeast_1")).unwrap();
    assert!(region.validate(&enum_identifier("us_isob_east_1")).is_err());
}

#[test]
fn ipam_pool_locale_is_plain_string_accepting_regions_and_local_zone_border_groups() {
    let schema = super::generated::ec2::ipam_pool::ec2_ipam_pool_config().schema;
    let locale = field(&schema, "locale");
    assert!(matches!(
        locale.as_type().raw_shape(),
        RawShape::String {
            identity_layer: StringIdentityParts {
                identity: None,
                pattern: None,
                length: None,
                validate: None,
                to_dsl: None
            },
            ..
        }
    ));
    for value in ["us-west-2-lax-1", "ap-northeast-1"] {
        locale.validate(&string(value)).unwrap();
        let source = format!(
            r#"
provider awscc {{ region = awscc.Region.ap_northeast_1 }}
awscc.ec2.IpamPool {{
  address_family = ipv4
  ipam_scope_id = 'ipam-scope-12345678'
  locale = '{value}'
}}
"#
        );
        validate_fixtures(&[("ipam_locale.crn", &source)]);
    }
}
