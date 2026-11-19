//! AWSCC scalar definitions ported from the pinned catalog at 5ad417e.
//! Each row owns its grammar and sample; identity construction fixes awscc.

use super::catalog_definition::{
    IdentityDefinition, catalog_definition, catalog_values, define_awscc_type_catalog,
};
use super::type_helpers::{validate_availability_zone, validate_aws_region};
use crate::schemas::namespace::AwsccIdentity;
use carina_core::resource::{ConcreteValue, Value};
use carina_core::schema::{DslTransform, PatternMatch, PatternSpec, legacy_validator};

// IAM roleNameType and policyNameType use ASCII letters, digits and _+=,.@-.
const IAM_NAME_PATTERN: &str = r"^[A-Za-z0-9_+=,.@-]+\z";

fn metadata_string(
    identity: AwsccIdentity,
    pattern: PatternSpec,
    length: Option<(Option<u64>, Option<u64>)>,
) -> IdentityDefinition {
    IdentityDefinition::string(identity, Some(pattern), length, None, None)
}

fn validated_int_enum_definition(
    identity: AwsccIdentity,
    valid_values: &'static [i64],
) -> IdentityDefinition {
    IdentityDefinition::int(
        identity,
        None,
        Some(legacy_validator(move |value| {
            if let Value::Concrete(ConcreteValue::Int(value)) = value {
                if valid_values.contains(value) {
                    Ok(())
                } else {
                    Err(
                        format!("Value {value} is not valid; allowed values are: {valid_values:?}")
                            .into(),
                    )
                }
            } else {
                Err("Expected integer".into())
            }
        })),
    )
}

fn prefixed_resource_id(identity: AwsccIdentity, prefix: &'static str) -> IdentityDefinition {
    metadata_string(
        identity,
        PatternSpec::new(format!("^{prefix}-[0-9a-f]{{8,}}$"), PatternMatch::Full),
        None,
    )
}

fn validated_open_enum(
    identity: AwsccIdentity,
    validator: fn(&str) -> Result<(), String>,
) -> IdentityDefinition {
    IdentityDefinition::open_enum(
        identity,
        Some(legacy_validator(move |value| {
            if let Value::Concrete(ConcreteValue::String(value)) = value {
                validator(value.expose_plaintext()).map_err(Into::into)
            } else {
                Err("Expected string".into())
            }
        })),
        Some(DslTransform::HyphenToUnderscore),
    )
}

fn availability_zone_type(identity: AwsccIdentity) -> IdentityDefinition {
    IdentityDefinition::open_enum(
        identity,
        Some(legacy_validator(|value| {
            if let Value::Concrete(ConcreteValue::String(value)) = value {
                let value = value.expose_plaintext();
                validate_availability_zone(value)
                    .map_err(|reason| format!("Invalid availability zone '{value}': {reason}"))
                    .map_err(Into::into)
            } else {
                Err("Expected string".into())
            }
        })),
        Some(DslTransform::HyphenToUnderscore),
    )
}

define_awscc_type_catalog! {
    pub enum ScalarType {
        rows: {
    AllocationId => allocation_id {
        segments: ["ec2", "Eip"],
        kind: "AllocationId",
        valid_sample: "eipalloc-12345678",
        constructor: |identity| prefixed_resource_id(identity, "eipalloc"),
    },
    AvailabilityZone => availability_zone {
        segments: ["AvailabilityZone"],
        kind: "ZoneName",
        valid_sample: "ap-northeast-1a",
        constructor: availability_zone_type,
    },
    AvailabilityZoneId => availability_zone_id {
        segments: ["AvailabilityZone"],
        kind: "ZoneId",
        valid_sample: "use1-az1",
        constructor: |identity| metadata_string(identity, PatternSpec::new("^[a-z]+[0-9]+-az[0-9]+$", PatternMatch::Full), None),
    },
    AccountId => aws_account_id {
        segments: [],
        kind: "AccountId",
        valid_sample: "123456789012",
        constructor: |identity| metadata_string(identity, PatternSpec::new("^[0-9]{12}$", PatternMatch::Full), Some((Some(12), Some(12)))),
    },
    Region => aws_region {
        segments: [],
        kind: "Region",
        valid_sample: "ap-northeast-1",
        constructor: |identity| validated_open_enum(identity, validate_aws_region),
    },
    ResourceId => aws_resource_id {
        segments: [],
        kind: "ResourceId",
        valid_sample: "resource-12345678",
        constructor: |identity| metadata_string(identity, PatternSpec::new("^[a-z0-9]+-[0-9a-f]{8,}$", PatternMatch::Full), None),
    },
    CarrierGatewayId => carrier_gateway_id {
        segments: ["ec2", "CarrierGateway"],
        kind: "Id",
        valid_sample: "cagw-12345678",
        constructor: |identity| prefixed_resource_id(identity, "cagw"),
    },
    EgressOnlyInternetGatewayId => egress_only_internet_gateway_id {
        segments: ["ec2", "EgressOnlyInternetGateway"],
        kind: "Id",
        valid_sample: "eigw-12345678",
        constructor: |identity| prefixed_resource_id(identity, "eigw"),
    },
    Elbv2TargetGroupGrpcCode => elbv2_target_group_grpc_code {
        segments: ["elasticloadbalancingv2", "TargetGroup"],
        kind: "GrpcCode",
        valid_sample: "12",
        constructor: |identity| super::type_helpers::elbv2_matcher_codes(identity, 0, 99),
    },
    Elbv2TargetGroupHttpCode => elbv2_target_group_http_code {
        segments: ["elasticloadbalancingv2", "TargetGroup"],
        kind: "HttpCode",
        valid_sample: "200",
        constructor: |identity| super::type_helpers::elbv2_matcher_codes(identity, 200, 599),
    },
    FlowLogId => flow_log_id {
        segments: ["ec2", "FlowLog"],
        kind: "Id",
        valid_sample: "fl-12345678",
        constructor: |identity| prefixed_resource_id(identity, "fl"),
    },
    IamPolicyName => iam_policy_name {
        segments: ["iam", "Policy"],
        kind: "Name",
        valid_sample: "example-policy",
        constructor: |identity| metadata_string(identity, PatternSpec::new(IAM_NAME_PATTERN, PatternMatch::Full), Some((Some(1), Some(128)))),
    },
    IamRoleId => iam_role_id {
        segments: ["iam", "Role"],
        kind: "Id",
        valid_sample: "AROA1234567890EXAMPLE",
        constructor: |identity| metadata_string(identity, PatternSpec::new("^AROA[A-Z0-9]+$", PatternMatch::Full), None),
    },
    IamRoleName => iam_role_name {
        segments: ["iam", "Role"],
        kind: "Name",
        valid_sample: "example-role",
        constructor: |identity| metadata_string(identity, PatternSpec::new(IAM_NAME_PATTERN, PatternMatch::Full), Some((Some(1), Some(64)))),
    },
    IdentityStoreId => identity_store_id {
        segments: ["identitystore", "Store"],
        kind: "Id",
        valid_sample: "d-1234567890",
        constructor: |identity| metadata_string(
            identity,
            PatternSpec::new("^(d-[0-9a-fA-F]{10}|[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12})$", PatternMatch::Full),
            None,
        ),
    },
    InstanceId => instance_id {
        segments: ["ec2", "Instance"],
        kind: "Id",
        valid_sample: "i-12345678",
        constructor: |identity| prefixed_resource_id(identity, "i"),
    },
    InternetGatewayId => internet_gateway_id {
        segments: ["ec2", "InternetGateway"],
        kind: "Id",
        valid_sample: "igw-12345678",
        constructor: |identity| prefixed_resource_id(identity, "igw"),
    },
    IpamId => ipam_id {
        segments: ["ec2", "Ipam"],
        kind: "Id",
        valid_sample: "ipam-12345678",
        constructor: |identity| prefixed_resource_id(identity, "ipam"),
    },
    IpamPoolId => ipam_pool_id {
        segments: ["ec2", "IpamPool"],
        kind: "Id",
        valid_sample: "ipam-pool-12345678",
        constructor: |identity| metadata_string(identity, PatternSpec::new("^ipam-pool-[0-9a-f]{8,}$", PatternMatch::Full), None),
    },
    KmsKeyReference => kms_key_reference {
        segments: ["kms", "Key"],
        kind: "Reference",
        valid_sample: "alias/aws/sqs",
        constructor: |identity| IdentityDefinition::reference(identity,
Some(PatternSpec::new(super::type_helpers::kms::kms_key_reference_pattern(), PatternMatch::Full)),
Some((Some(1), Some(2048)))),
    },
    LocalGatewayId => local_gateway_id {
        segments: ["ec2", "LocalGateway"],
        kind: "Id",
        valid_sample: "lgw-12345678",
        constructor: |identity| prefixed_resource_id(identity, "lgw"),
    },
    LogsLogGroupRetentionInDays => logs_log_group_retention_in_days {
        segments: ["logs", "LogGroup"],
        kind: "RetentionInDays",
        valid_sample: "30",
        values: [
            1, 3, 5, 7, 14, 30, 60, 90, 120, 150, 180, 365, 400, 545, 731, 1096,
            1827, 2192, 2557, 2922, 3288, 3653,
        ],

        constructor: validated_int_enum_definition,
    },
    NatGatewayId => nat_gateway_id {
        segments: ["ec2", "NatGateway"],
        kind: "Id",
        valid_sample: "nat-12345678",
        constructor: |identity| prefixed_resource_id(identity, "nat"),
    },
    NetworkAclId => network_acl_id {
        segments: ["ec2", "NetworkAcl"],
        kind: "Id",
        valid_sample: "acl-12345678",
        constructor: |identity| prefixed_resource_id(identity, "acl"),
    },
    NetworkInterfaceId => network_interface_id {
        segments: ["ec2", "NetworkInterface"],
        kind: "Id",
        valid_sample: "eni-12345678",
        constructor: |identity| prefixed_resource_id(identity, "eni"),
    },
    PrefixListId => prefix_list_id {
        segments: ["ec2", "PrefixList"],
        kind: "Id",
        valid_sample: "pl-12345678",
        constructor: |identity| prefixed_resource_id(identity, "pl"),
    },
    Route53HostedZoneId => route53_hosted_zone_id {
        segments: ["route53", "HostedZone"],
        kind: "Id",
        valid_sample: "Z2FDTNDATAQYW2",
        constructor: |identity| metadata_string(
            identity,
            PatternSpec::new("^Z[A-Z0-9]+$", PatternMatch::Full),
            Some((None, Some(32))),
        ),
    },
    RouteTableId => route_table_id {
        segments: ["ec2", "RouteTable"],
        kind: "Id",
        valid_sample: "rtb-12345678",
        constructor: |identity| prefixed_resource_id(identity, "rtb"),
    },
    SecurityGroupId => security_group_id {
        segments: ["ec2", "SecurityGroup"],
        kind: "Id",
        valid_sample: "sg-12345678",
        constructor: |identity| prefixed_resource_id(identity, "sg"),
    },
    SsoPrincipalId => sso_principal_id {
        segments: ["sso", "Principal"],
        kind: "Id",
        valid_sample: "12345678-1234-1234-1234-123456789abc",
        constructor: |identity| metadata_string(identity, PatternSpec::new("^.{1,64}$", PatternMatch::Full), None),
    },
    SubnetId => subnet_id {
        segments: ["ec2", "Subnet"],
        kind: "Id",
        valid_sample: "subnet-12345678",
        constructor: |identity| prefixed_resource_id(identity, "subnet"),
    },
    SubnetRouteTableAssociationId => subnet_route_table_association_id {
        segments: ["ec2", "SubnetRouteTableAssociation"],
        kind: "Id",
        valid_sample: "rtbassoc-12345678",
        constructor: |identity| prefixed_resource_id(identity, "rtbassoc"),
    },
    TransitGatewayRouteTableId => tgw_route_table_id {
        segments: ["ec2", "TransitGatewayRouteTable"],
        kind: "Id",
        valid_sample: "tgw-rtb-12345678",
        constructor: |identity| prefixed_resource_id(identity, "tgw-rtb"),
    },
    TransitGatewayAttachmentId => transit_gateway_attachment_id {
        segments: ["ec2", "TransitGatewayAttachment"],
        kind: "Id",
        valid_sample: "tgw-attach-12345678",
        constructor: |identity| prefixed_resource_id(identity, "tgw-attach"),
    },
    TransitGatewayId => transit_gateway_id {
        segments: ["ec2", "TransitGateway"],
        kind: "Id",
        valid_sample: "tgw-12345678",
        constructor: |identity| prefixed_resource_id(identity, "tgw"),
    },
    VpcCidrBlockAssociationId => vpc_cidr_block_association_id {
        segments: ["ec2", "VpcCidrBlockAssociation"],
        kind: "Id",
        valid_sample: "vpc-cidr-assoc-12345678",
        constructor: |identity| prefixed_resource_id(identity, "vpc-cidr-assoc"),
    },
    VpcEndpointId => vpc_endpoint_id {
        segments: ["ec2", "VpcEndpoint"],
        kind: "Id",
        valid_sample: "vpce-12345678",
        constructor: |identity| prefixed_resource_id(identity, "vpce"),
    },
    VpcId => vpc_id {
        segments: ["ec2", "Vpc"],
        kind: "Id",
        valid_sample: "vpc-12345678",
        constructor: |identity| prefixed_resource_id(identity, "vpc"),
    },
    VpcPeeringConnectionId => vpc_peering_connection_id {
        segments: ["ec2", "VpcPeeringConnection"],
        kind: "Id",
        valid_sample: "pcx-12345678",
        constructor: |identity| prefixed_resource_id(identity, "pcx"),
    },
    VpnGatewayId => vpn_gateway_id {
        segments: ["ec2", "VpnGateway"],
        kind: "Id",
        valid_sample: "vgw-12345678",
        constructor: |identity| prefixed_resource_id(identity, "vgw"),
    },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use carina_core::schema::{RawShape, ValidatedConfigSchema};
    use std::collections::BTreeMap;

    #[test]
    fn scalar_catalog_rows_own_their_identity_and_accept_their_samples() {
        for row in ScalarType::ALL.iter().copied() {
            assert_eq!(row.identity().provider.as_deref(), Some("awscc"));
            let ty = row.attribute_type();
            let sample: Value = match ty.raw_shape() {
                RawShape::Int { .. } => {
                    Value::Concrete(ConcreteValue::Int(row.valid_sample().parse().unwrap()))
                }
                RawShape::Enum { dsl_aliases, .. } => {
                    let spelling = dsl_aliases
                        .iter()
                        .find(|(api, _)| api == row.valid_sample())
                        .map_or(row.valid_sample(), |(_, dsl)| dsl.as_str());
                    Value::Concrete(ConcreteValue::enum_identifier(spelling))
                }
                _ => Value::Concrete(ConcreteValue::String(row.valid_sample().into())),
            };
            let schema =
                ValidatedConfigSchema::validate(BTreeMap::from([("sample".to_string(), ty)]))
                    .unwrap();
            schema
                .rootless_type("sample")
                .unwrap()
                .validate(&sample)
                .unwrap_or_else(|error| panic!("{} sample: {error}", row.identity()));
        }
    }
}
