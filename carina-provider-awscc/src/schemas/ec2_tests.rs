//! #525 EC2 scalar constraints from live API and Cloud Control probes.
//! carina-test-008, ap-northeast-1, 2026-11-06. Subnet IPAM netmask
//! bounds are docs only, not probed. Contextual constraints remain deferred.

use std::collections::BTreeSet;

use carina_core::resource::{ConcreteValue, Value};
use carina_core::schema::{EffectiveType, ValidatedResourceSchema};

use crate::test_support::{enum_identifier, field, float, int, string, validate_fixtures};

fn ec2(resource: &str) -> ValidatedResourceSchema {
    super::all_schemas()
        .into_iter()
        .find(|schema| schema.as_schema().resource_type() == format!("ec2.{resource}"))
        .unwrap_or_else(|| panic!("missing EC2 resource {resource}"))
}

fn strings(attribute: EffectiveType<'_>, cases: &[(&str, bool)]) {
    for &(value, accepted) in cases {
        let result = attribute.validate(&string(value));
        assert_eq!(result.is_ok(), accepted, "{value:?}: {result:?}");
    }
}

fn integers(attribute: EffectiveType<'_>, cases: &[(i64, bool)]) {
    for &(value, accepted) in cases {
        let result = attribute.validate(&int(value));
        assert_eq!(result.is_ok(), accepted, "{value}: {result:?}");
    }
    assert!(attribute.validate(&float(1.5)).is_err());
    assert!(attribute.validate(&string("1")).is_err());
}

#[test]
fn issue_525_ec2_tags_match_cache_and_live_character_boundaries() {
    let resources = [
        ("EIP", "Eip"),
        ("EgressOnlyInternetGateway", "EgressOnlyInternetGateway"),
        ("FlowLog", "FlowLog"),
        ("InternetGateway", "InternetGateway"),
        ("IPAM", "Ipam"),
        ("IPAMPool", "IpamPool"),
        ("NatGateway", "NatGateway"),
        ("RouteTable", "RouteTable"),
        ("SecurityGroup", "SecurityGroup"),
        ("Subnet", "Subnet"),
        ("TransitGateway", "TransitGateway"),
        ("TransitGatewayAttachment", "TransitGatewayAttachment"),
        ("VPC", "Vpc"),
        ("VPCEndpoint", "VpcEndpoint"),
        ("VPCPeeringConnection", "VpcPeeringConnection"),
        ("VPNGateway", "VpnGateway"),
    ];
    let cache = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../cfn-schema-cache");
    let cached = std::fs::read_dir(&cache)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|file| {
            file.file_name()
                .unwrap()
                .to_str()
                .unwrap()
                .starts_with("AWS__EC2__")
        })
        .map(|file| {
            serde_json::from_str::<serde_json::Value>(&std::fs::read_to_string(file).unwrap())
                .unwrap()
        })
        .filter(|schema| schema["properties"]["Tags"]["items"]["$ref"] == "#/definitions/Tag")
        .map(|schema| schema["typeName"].as_str().unwrap().to_string())
        .collect::<BTreeSet<_>>();
    assert_eq!(
        cached,
        resources.map(|(raw, _)| format!("AWS::EC2::{raw}")).into()
    );

    for (raw, resource) in resources {
        let cached: serde_json::Value = serde_json::from_str(
            &std::fs::read_to_string(cache.join(format!("AWS__EC2__{raw}.json"))).unwrap(),
        )
        .unwrap();
        let schema = ec2(resource);
        let tags = field(&schema, "tags");
        let value = tags.map_value().unwrap();
        let minimum = cached["definitions"]["Tag"]["properties"]["Key"]["minLength"].as_u64();
        let keys: &[(&str, bool)] = &[
            (&"k".repeat(127), true),
            (&"k".repeat(128), true),
            (&"k".repeat(129), false),
            (&"日".repeat(128), true),
            ("", minimum.unwrap_or(0) == 0),
            // Reserved prefixes belong to the service-wide tagging work.
            ("aws:probe", true),
            ("AWS:probe", true),
        ];
        for &(key, accepted) in keys {
            let entry = Value::Concrete(ConcreteValue::Map(
                [(key.to_string(), string("v"))].into_iter().collect(),
            ));
            let result = tags.validate(&entry);
            assert_eq!(result.is_ok(), accepted, "{raw} key {key:?}: {result:?}");
        }
        strings(
            value,
            &[
                ("", true),
                (&"v".repeat(256), true),
                (&"v".repeat(257), false),
                (&"日".repeat(256), true),
            ],
        );
    }
}

#[test]
fn issue_525_ec2_security_group_descriptions_match_live_vectors() {
    for (resource, path) in [
        ("SecurityGroup", "group_description"),
        ("SecurityGroup", "security_group_ingress.description"),
        ("SecurityGroup", "security_group_egress.description"),
        ("SecurityGroupIngress", "description"),
        ("SecurityGroupEgress", "description"),
    ] {
        let schema = ec2(resource);
        strings(
            field(&schema, path),
            &[
                ("", true),
                (&"d".repeat(255), true),
                (&"d".repeat(256), false),
                ("a-z A-Z 0-9 . _:/()#,@[]+=&;{}!$*-", true),
                ("&", true),
                ("'", false),
                ("%", false),
                ("a\nb", false),
                ("日本語", false),
                ("é", false),
                ("\\", false),
            ],
        );
    }
}

#[test]
fn issue_525_ec2_subnet_documented_ipam_masks() {
    let schema = ec2("Subnet");
    integers(
        field(&schema, "ipv4_netmask_length"),
        &[(15, false), (16, true), (17, true), (28, true), (29, false)],
    );
}

#[test]
fn issue_525_ec2_amazon_side_asn_is_the_private_range_union() {
    for resource in ["VpnGateway", "TransitGateway"] {
        let schema = ec2(resource);
        integers(
            field(&schema, "amazon_side_asn"),
            &[
                (64512, true),
                (65534, true),
                (4200000000, true),
                (4294967294, true),
                (64511, false),
                (65535, false),
                (4199999999, false),
                (7224, false),
                (4294967295, false),
            ],
        );
    }
}

#[test]
fn issue_525_ec2_amazon_side_asn_accepts_int_typed_module_arguments() {
    validate_fixtures(&[(
        "ec2_asn_argument.crn",
        r#"
arguments {
  asn: Int
}

awscc.ec2.VpnGateway {
  type = aws.ec2.VpnGateway.Type.ipsec_1
  amazon_side_asn = asn
}

awscc.ec2.TransitGateway {
  amazon_side_asn = asn
}
"#,
    )]);
}

#[test]
fn issue_525_ec2_nat_count_and_accepted_drain_durations() {
    let schema = ec2("NatGateway");
    integers(
        field(&schema, "secondary_private_ip_address_count"),
        &[(0, false), (1, true), (31, true), (32, false)],
    );
    integers(
        field(&schema, "max_drain_duration_seconds"),
        &[(0, true), (4001, true)],
    );
}

#[test]
fn issue_525_ec2_ipam_pool_netmasks_keep_the_family_independent_envelope() {
    let schema = ec2("IpamPool");
    for path in [
        "allocation_min_netmask_length",
        "allocation_max_netmask_length",
        "allocation_default_netmask_length",
    ] {
        integers(
            field(&schema, path),
            &[
                (-1, false),
                (0, true),
                (32, true),
                (33, true),
                (128, true),
                (129, false),
            ],
        );
    }
}

#[test]
fn issue_525_ec2_enums_keep_dsl_names_and_reject_live_invalid_values() {
    for (resource, path, identity, accepted, rejected) in [
        (
            "IpamPool",
            "source_resource.resource_type",
            "aws.ec2.IpamPool.SourceResource.ResourceType",
            vec!["vpc"],
            vec!["subnet"],
        ),
        (
            "Vpc",
            "instance_tenancy",
            "aws.ec2.Vpc.InstanceTenancy",
            vec!["default", "dedicated"],
            vec!["host"],
        ),
    ] {
        let schema = ec2(resource);
        let attribute = field(&schema, path);
        for value in accepted {
            attribute
                .validate(&enum_identifier(&format!("{identity}.{value}")))
                .unwrap();
        }
        for value in rejected {
            assert!(
                attribute
                    .validate(&enum_identifier(&format!("{identity}.{value}")))
                    .is_err()
            );
        }
    }
}
