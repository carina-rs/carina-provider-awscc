//! Identity bindings carry no field structure or local constraints.
use carina_core::schema::{CustomValidator, RawShape, TypeIdentity};

#[cfg(any(test, feature = "test-utils"))]
pub type StructuralStringMetadata = (Option<String>, Option<(Option<u64>, Option<u64>)>);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum StringIdentity {
    AccountId,
    Arn,
    AvailabilityZoneZoneId,
    AvailabilityZoneZoneName,
    Region,
    CloudfrontDistributionArn,
    DomainName,
    DynamodbTableArn,
    Ec2CarrierGatewayId,
    Ec2EgressOnlyInternetGatewayId,
    Ec2EipAllocationId,
    Ec2FlowLogId,
    Ec2InstanceId,
    Ec2InternetGatewayId,
    Ec2IpamArn,
    Ec2IpamId,
    Ec2IpamPoolArn,
    Ec2IpamPoolId,
    Ec2LocalGatewayId,
    Ec2NatGatewayId,
    Ec2NetworkAclId,
    Ec2NetworkInterfaceId,
    Ec2PrefixListId,
    Ec2RouteTableId,
    Ec2SecurityGroupId,
    Ec2SubnetId,
    Ec2SubnetRouteTableAssociationId,
    Ec2TransitGatewayAttachmentId,
    Ec2TransitGatewayId,
    Ec2TransitGatewayRouteTableId,
    Ec2VpcCidrBlockAssociationId,
    Ec2VpcEndpointId,
    Ec2VpcId,
    Ec2VpcPeeringConnectionId,
    Ec2VpnGatewayId,
    EcsClusterArn,
    Elasticloadbalancingv2TargetGroupGrpcCode,
    Elasticloadbalancingv2TargetGroupHttpCode,
    Email,
    HttpResponseStatusCode,
    HttpsUrl,
    IamOidcProviderArn,
    IamPolicyArn,
    IamPolicyName,
    IamRoleArn,
    IamRoleId,
    IamRoleName,
    IdentitystoreGroupDisplayName,
    IdentitystoreStoreId,
    Ipv4Address,
    Ipv4Cidr,
    Ipv6Address,
    Ipv6Cidr,
    KmsKeyArn,
    KmsKeyId,
    LogsLogGroupArn,
    LogsLogGroupLogGroupName,
    OrganizationsAccountArn,
    OrganizationsAccountEmail,
    OrganizationsAccountRoleName,
    OrganizationsOrganizationArn,
    RedirectHost,
    RedirectPath,
    RedirectPort,
    RedirectProtocol,
    RedirectQuery,
    ResourceId,
    Route53HostedZoneId,
    S3BucketArn,
    S3BucketLifecycleDate,
    SchedulerScheduleGroupName,
    SchedulerScheduleScheduleExpression,
    SsoInstanceArn,
    SsoPermissionSetArn,
    SsoPermissionSetSessionDuration,
    SsoPrincipalId,
    Wafv2WebACLByteMatchSearchString,
    Wafv2WebACLByteMatchSearchStringBase64,
    Wafv2WebAclArn,
}

impl StringIdentity {
    pub const ALL: &[Self] = &[
        Self::AccountId,
        Self::Arn,
        Self::AvailabilityZoneZoneId,
        Self::AvailabilityZoneZoneName,
        Self::Region,
        Self::CloudfrontDistributionArn,
        Self::DomainName,
        Self::DynamodbTableArn,
        Self::Ec2CarrierGatewayId,
        Self::Ec2EgressOnlyInternetGatewayId,
        Self::Ec2EipAllocationId,
        Self::Ec2FlowLogId,
        Self::Ec2InstanceId,
        Self::Ec2InternetGatewayId,
        Self::Ec2IpamArn,
        Self::Ec2IpamId,
        Self::Ec2IpamPoolArn,
        Self::Ec2IpamPoolId,
        Self::Ec2LocalGatewayId,
        Self::Ec2NatGatewayId,
        Self::Ec2NetworkAclId,
        Self::Ec2NetworkInterfaceId,
        Self::Ec2PrefixListId,
        Self::Ec2RouteTableId,
        Self::Ec2SecurityGroupId,
        Self::Ec2SubnetId,
        Self::Ec2SubnetRouteTableAssociationId,
        Self::Ec2TransitGatewayAttachmentId,
        Self::Ec2TransitGatewayId,
        Self::Ec2TransitGatewayRouteTableId,
        Self::Ec2VpcCidrBlockAssociationId,
        Self::Ec2VpcEndpointId,
        Self::Ec2VpcId,
        Self::Ec2VpcPeeringConnectionId,
        Self::Ec2VpnGatewayId,
        Self::EcsClusterArn,
        Self::Elasticloadbalancingv2TargetGroupGrpcCode,
        Self::Elasticloadbalancingv2TargetGroupHttpCode,
        Self::Email,
        Self::HttpResponseStatusCode,
        Self::HttpsUrl,
        Self::IamOidcProviderArn,
        Self::IamPolicyArn,
        Self::IamPolicyName,
        Self::IamRoleArn,
        Self::IamRoleId,
        Self::IamRoleName,
        Self::IdentitystoreGroupDisplayName,
        Self::IdentitystoreStoreId,
        Self::Ipv4Address,
        Self::Ipv4Cidr,
        Self::Ipv6Address,
        Self::Ipv6Cidr,
        Self::KmsKeyArn,
        Self::KmsKeyId,
        Self::LogsLogGroupArn,
        Self::LogsLogGroupLogGroupName,
        Self::OrganizationsAccountArn,
        Self::OrganizationsAccountEmail,
        Self::OrganizationsAccountRoleName,
        Self::OrganizationsOrganizationArn,
        Self::RedirectHost,
        Self::RedirectPath,
        Self::RedirectPort,
        Self::RedirectProtocol,
        Self::RedirectQuery,
        Self::ResourceId,
        Self::Route53HostedZoneId,
        Self::S3BucketArn,
        Self::S3BucketLifecycleDate,
        Self::SchedulerScheduleGroupName,
        Self::SchedulerScheduleScheduleExpression,
        Self::SsoInstanceArn,
        Self::SsoPermissionSetArn,
        Self::SsoPermissionSetSessionDuration,
        Self::SsoPrincipalId,
        Self::Wafv2WebACLByteMatchSearchString,
        Self::Wafv2WebACLByteMatchSearchStringBase64,
        Self::Wafv2WebAclArn,
    ];

    pub fn identity(self) -> TypeIdentity {
        match self {
            Self::AccountId => {
                TypeIdentity::new(Some("aws"), std::iter::empty::<&str>(), "AccountId")
            }
            Self::Arn => TypeIdentity::new(Some("aws"), std::iter::empty::<&str>(), "Arn"),
            Self::AvailabilityZoneZoneName => {
                TypeIdentity::new(Some("aws"), ["AvailabilityZone"], "ZoneName")
            }
            Self::Region => TypeIdentity::new(Some("aws"), std::iter::empty::<&str>(), "Region"),
            Self::AvailabilityZoneZoneId => {
                TypeIdentity::new(Some("aws"), ["AvailabilityZone"], "ZoneId")
            }
            Self::CloudfrontDistributionArn => {
                TypeIdentity::new(Some("aws"), ["cloudfront", "Distribution"], "Arn")
            }
            Self::DomainName => {
                TypeIdentity::new(None::<&str>, std::iter::empty::<&str>(), "DomainName")
            }
            Self::DynamodbTableArn => TypeIdentity::new(Some("aws"), ["dynamodb", "Table"], "Arn"),
            Self::Ec2CarrierGatewayId => {
                TypeIdentity::new(Some("aws"), ["ec2", "CarrierGateway"], "Id")
            }
            Self::Ec2EgressOnlyInternetGatewayId => {
                TypeIdentity::new(Some("aws"), ["ec2", "EgressOnlyInternetGateway"], "Id")
            }
            Self::Ec2EipAllocationId => {
                TypeIdentity::new(Some("aws"), ["ec2", "Eip"], "AllocationId")
            }
            Self::Ec2FlowLogId => TypeIdentity::new(Some("aws"), ["ec2", "FlowLog"], "Id"),
            Self::Ec2InstanceId => TypeIdentity::new(Some("aws"), ["ec2", "Instance"], "Id"),
            Self::Ec2InternetGatewayId => {
                TypeIdentity::new(Some("aws"), ["ec2", "InternetGateway"], "Id")
            }
            Self::Ec2IpamArn => TypeIdentity::new(Some("aws"), ["ec2", "Ipam"], "Arn"),
            Self::Ec2IpamId => TypeIdentity::new(Some("aws"), ["ec2", "Ipam"], "Id"),
            Self::Ec2IpamPoolArn => TypeIdentity::new(Some("aws"), ["ec2", "IpamPool"], "Arn"),
            Self::Ec2IpamPoolId => TypeIdentity::new(Some("aws"), ["ec2", "IpamPool"], "Id"),
            Self::Ec2LocalGatewayId => {
                TypeIdentity::new(Some("aws"), ["ec2", "LocalGateway"], "Id")
            }
            Self::Ec2NatGatewayId => TypeIdentity::new(Some("aws"), ["ec2", "NatGateway"], "Id"),
            Self::Ec2NetworkAclId => TypeIdentity::new(Some("aws"), ["ec2", "NetworkAcl"], "Id"),
            Self::Ec2NetworkInterfaceId => {
                TypeIdentity::new(Some("aws"), ["ec2", "NetworkInterface"], "Id")
            }
            Self::Ec2PrefixListId => TypeIdentity::new(Some("aws"), ["ec2", "PrefixList"], "Id"),
            Self::Ec2RouteTableId => TypeIdentity::new(Some("aws"), ["ec2", "RouteTable"], "Id"),
            Self::Ec2SecurityGroupId => {
                TypeIdentity::new(Some("aws"), ["ec2", "SecurityGroup"], "Id")
            }
            Self::Ec2SubnetId => TypeIdentity::new(Some("aws"), ["ec2", "Subnet"], "Id"),
            Self::Ec2SubnetRouteTableAssociationId => {
                TypeIdentity::new(Some("aws"), ["ec2", "SubnetRouteTableAssociation"], "Id")
            }
            Self::Ec2TransitGatewayAttachmentId => {
                TypeIdentity::new(Some("aws"), ["ec2", "TransitGatewayAttachment"], "Id")
            }
            Self::Ec2TransitGatewayId => {
                TypeIdentity::new(Some("aws"), ["ec2", "TransitGateway"], "Id")
            }
            Self::Ec2TransitGatewayRouteTableId => {
                TypeIdentity::new(Some("aws"), ["ec2", "TransitGatewayRouteTable"], "Id")
            }
            Self::Ec2VpcCidrBlockAssociationId => {
                TypeIdentity::new(Some("aws"), ["ec2", "VpcCidrBlockAssociation"], "Id")
            }
            Self::Ec2VpcEndpointId => TypeIdentity::new(Some("aws"), ["ec2", "VpcEndpoint"], "Id"),
            Self::Ec2VpcId => TypeIdentity::new(Some("aws"), ["ec2", "Vpc"], "Id"),
            Self::Ec2VpcPeeringConnectionId => {
                TypeIdentity::new(Some("aws"), ["ec2", "VpcPeeringConnection"], "Id")
            }
            Self::Ec2VpnGatewayId => TypeIdentity::new(Some("aws"), ["ec2", "VpnGateway"], "Id"),
            Self::EcsClusterArn => TypeIdentity::new(Some("aws"), ["ecs", "Cluster"], "Arn"),
            Self::Elasticloadbalancingv2TargetGroupGrpcCode => TypeIdentity::new(
                Some("aws"),
                ["elasticloadbalancingv2", "TargetGroup"],
                "GrpcCode",
            ),
            Self::Elasticloadbalancingv2TargetGroupHttpCode => TypeIdentity::new(
                Some("aws"),
                ["elasticloadbalancingv2", "TargetGroup"],
                "HttpCode",
            ),
            Self::Email => TypeIdentity::new(None::<&str>, std::iter::empty::<&str>(), "Email"),
            Self::HttpResponseStatusCode => TypeIdentity::new(
                None::<&str>,
                std::iter::empty::<&str>(),
                "HttpResponseStatusCode",
            ),
            Self::HttpsUrl => {
                TypeIdentity::new(None::<&str>, std::iter::empty::<&str>(), "HttpsUrl")
            }
            Self::IamOidcProviderArn => {
                TypeIdentity::new(Some("aws"), ["iam", "OidcProvider"], "Arn")
            }
            Self::IamPolicyArn => TypeIdentity::new(Some("aws"), ["iam", "Policy"], "Arn"),
            Self::IamPolicyName => TypeIdentity::new(Some("aws"), ["iam", "Policy"], "Name"),
            Self::IamRoleArn => TypeIdentity::new(Some("aws"), ["iam", "Role"], "Arn"),
            Self::IamRoleId => TypeIdentity::new(Some("aws"), ["iam", "Role"], "Id"),
            Self::IamRoleName => TypeIdentity::new(Some("aws"), ["iam", "Role"], "Name"),
            Self::IdentitystoreGroupDisplayName => {
                TypeIdentity::new(Some("aws"), ["identitystore", "Group"], "DisplayName")
            }
            Self::IdentitystoreStoreId => {
                TypeIdentity::new(Some("aws"), ["identitystore", "Store"], "Id")
            }
            Self::Ipv4Address => {
                TypeIdentity::new(None::<&str>, std::iter::empty::<&str>(), "Ipv4Address")
            }
            Self::Ipv4Cidr => {
                TypeIdentity::new(None::<&str>, std::iter::empty::<&str>(), "Ipv4Cidr")
            }
            Self::Ipv6Address => {
                TypeIdentity::new(None::<&str>, std::iter::empty::<&str>(), "Ipv6Address")
            }
            Self::Ipv6Cidr => {
                TypeIdentity::new(None::<&str>, std::iter::empty::<&str>(), "Ipv6Cidr")
            }
            Self::KmsKeyArn => TypeIdentity::new(Some("aws"), ["kms", "Key"], "Arn"),
            Self::KmsKeyId => TypeIdentity::new(Some("aws"), ["kms", "Key"], "Id"),
            Self::LogsLogGroupArn => TypeIdentity::new(Some("aws"), ["logs", "LogGroup"], "Arn"),
            Self::LogsLogGroupLogGroupName => {
                TypeIdentity::new(Some("aws"), ["logs", "LogGroup"], "LogGroupName")
            }
            Self::OrganizationsAccountArn => {
                TypeIdentity::new(Some("aws"), ["organizations", "Account"], "Arn")
            }
            Self::OrganizationsAccountEmail => {
                TypeIdentity::new(Some("aws"), ["organizations", "Account"], "Email")
            }
            Self::OrganizationsAccountRoleName => {
                TypeIdentity::new(Some("aws"), ["organizations", "Account"], "RoleName")
            }
            Self::OrganizationsOrganizationArn => {
                TypeIdentity::new(Some("aws"), ["organizations", "Organization"], "Arn")
            }
            Self::RedirectHost => {
                TypeIdentity::new(None::<&str>, std::iter::empty::<&str>(), "RedirectHost")
            }
            Self::RedirectPath => {
                TypeIdentity::new(None::<&str>, std::iter::empty::<&str>(), "RedirectPath")
            }
            Self::RedirectPort => {
                TypeIdentity::new(None::<&str>, std::iter::empty::<&str>(), "RedirectPort")
            }
            Self::RedirectProtocol => {
                TypeIdentity::new(None::<&str>, std::iter::empty::<&str>(), "RedirectProtocol")
            }
            Self::RedirectQuery => {
                TypeIdentity::new(None::<&str>, std::iter::empty::<&str>(), "RedirectQuery")
            }
            Self::ResourceId => {
                TypeIdentity::new(Some("aws"), std::iter::empty::<&str>(), "ResourceId")
            }
            Self::Route53HostedZoneId => {
                TypeIdentity::new(Some("aws"), ["route53", "HostedZone"], "Id")
            }
            Self::S3BucketArn => TypeIdentity::new(Some("aws"), ["s3", "Bucket"], "Arn"),
            Self::S3BucketLifecycleDate => {
                TypeIdentity::new(Some("aws"), ["s3", "Bucket"], "LifecycleDate")
            }
            Self::SchedulerScheduleGroupName => {
                TypeIdentity::new(Some("aws"), ["scheduler", "Schedule"], "GroupName")
            }
            Self::SchedulerScheduleScheduleExpression => {
                TypeIdentity::new(Some("aws"), ["scheduler", "Schedule"], "ScheduleExpression")
            }
            Self::SsoInstanceArn => TypeIdentity::new(Some("aws"), ["sso", "Instance"], "Arn"),
            Self::SsoPermissionSetArn => {
                TypeIdentity::new(Some("aws"), ["sso", "PermissionSet"], "Arn")
            }
            Self::SsoPermissionSetSessionDuration => {
                TypeIdentity::new(Some("aws"), ["sso", "PermissionSet"], "SessionDuration")
            }
            Self::SsoPrincipalId => TypeIdentity::new(Some("aws"), ["sso", "Principal"], "Id"),
            Self::Wafv2WebACLByteMatchSearchString => {
                TypeIdentity::new(Some("aws"), ["wafv2", "WebACL"], "ByteMatchSearchString")
            }
            Self::Wafv2WebACLByteMatchSearchStringBase64 => TypeIdentity::new(
                Some("aws"),
                ["wafv2", "WebACL"],
                "ByteMatchSearchStringBase64",
            ),
            Self::Wafv2WebAclArn => TypeIdentity::new(Some("aws"), ["wafv2", "WebAcl"], "Arn"),
        }
    }

    pub fn is_core_builtin(self) -> bool {
        self.identity().provider.is_none()
    }

    /// Only the shared callback is returned, never constructor structure.
    fn validation_declaration(self) -> Option<carina_core::schema::AttributeType> {
        Some(match self {
            Self::AccountId => carina_aws_types::aws_account_id(),
            Self::Arn => super::catalog::arn(),
            Self::AvailabilityZoneZoneId => carina_aws_types::availability_zone_id(),
            Self::AvailabilityZoneZoneName => carina_aws_types::availability_zone(),
            Self::Region => carina_aws_types::aws_region(),
            Self::CloudfrontDistributionArn => super::catalog::cloudfront_distribution_arn(),
            Self::DomainName => carina_core::schema::types::domain_name(),
            Self::DynamodbTableArn => super::catalog::dynamodb_table_arn(),
            Self::Ec2CarrierGatewayId => carina_aws_types::carrier_gateway_id(),
            Self::Ec2EgressOnlyInternetGatewayId => {
                carina_aws_types::egress_only_internet_gateway_id()
            }
            Self::Ec2EipAllocationId => carina_aws_types::allocation_id(),
            Self::Ec2FlowLogId => carina_aws_types::flow_log_id(),
            Self::Ec2InstanceId => carina_aws_types::instance_id(),
            Self::Ec2InternetGatewayId => carina_aws_types::internet_gateway_id(),
            Self::Ec2IpamArn => super::catalog::ec2_ipam_arn(),
            Self::Ec2IpamId => carina_aws_types::ipam_id(),
            Self::Ec2IpamPoolArn => super::catalog::ec2_ipam_pool_arn(),
            Self::Ec2IpamPoolId => carina_aws_types::ipam_pool_id(),
            Self::Ec2LocalGatewayId => carina_aws_types::local_gateway_id(),
            Self::Ec2NatGatewayId => carina_aws_types::nat_gateway_id(),
            Self::Ec2NetworkAclId => carina_aws_types::network_acl_id(),
            Self::Ec2NetworkInterfaceId => carina_aws_types::network_interface_id(),
            Self::Ec2PrefixListId => carina_aws_types::prefix_list_id(),
            Self::Ec2RouteTableId => carina_aws_types::route_table_id(),
            Self::Ec2SecurityGroupId => carina_aws_types::security_group_id(),
            Self::Ec2SubnetId => carina_aws_types::subnet_id(),
            Self::Ec2SubnetRouteTableAssociationId => {
                carina_aws_types::subnet_route_table_association_id()
            }
            Self::Ec2TransitGatewayAttachmentId => {
                carina_aws_types::transit_gateway_attachment_id()
            }
            Self::Ec2TransitGatewayId => carina_aws_types::transit_gateway_id(),
            Self::Ec2TransitGatewayRouteTableId => carina_aws_types::tgw_route_table_id(),
            Self::Ec2VpcCidrBlockAssociationId => carina_aws_types::vpc_cidr_block_association_id(),
            Self::Ec2VpcEndpointId => carina_aws_types::vpc_endpoint_id(),
            Self::Ec2VpcId => carina_aws_types::vpc_id(),
            Self::Ec2VpcPeeringConnectionId => carina_aws_types::vpc_peering_connection_id(),
            Self::Ec2VpnGatewayId => carina_aws_types::vpn_gateway_id(),
            Self::EcsClusterArn => super::catalog::ecs_cluster_arn(),
            Self::Elasticloadbalancingv2TargetGroupGrpcCode => {
                carina_aws_types::elbv2_target_group_grpc_code()
            }
            Self::Elasticloadbalancingv2TargetGroupHttpCode => {
                carina_aws_types::elbv2_target_group_http_code()
            }
            Self::Email => carina_core::schema::types::email(),
            Self::HttpResponseStatusCode => carina_core::schema::types::http_response_status_code(),
            Self::HttpsUrl => carina_core::schema::types::https_url(),
            Self::IamOidcProviderArn => super::catalog::iam_oidc_provider_arn(),
            Self::IamPolicyArn => super::catalog::iam_policy_arn(),
            Self::IamPolicyName => carina_aws_types::iam_policy_name(),
            Self::IamRoleArn => super::catalog::iam_role_arn(),
            Self::IamRoleId => carina_aws_types::iam_role_id(),
            Self::IamRoleName => carina_aws_types::iam_role_name(),
            Self::IdentitystoreGroupDisplayName => {
                super::catalog::identitystore_group_display_name()
            }
            Self::IdentitystoreStoreId => return None,
            Self::Ipv4Address => carina_core::schema::types::ipv4_address(),
            Self::Ipv4Cidr => carina_core::schema::types::ipv4_cidr(),
            Self::Ipv6Address => carina_core::schema::types::ipv6_address(),
            Self::Ipv6Cidr => carina_core::schema::types::ipv6_cidr(),
            Self::KmsKeyArn => super::catalog::kms_key_arn(),
            Self::KmsKeyId => super::catalog::kms_key_id(),
            Self::LogsLogGroupArn => super::catalog::logs_log_group_arn(),
            Self::LogsLogGroupLogGroupName => super::catalog::logs_log_group_name(),
            Self::OrganizationsAccountArn => super::catalog::organizations_account_arn(),
            Self::OrganizationsAccountEmail => super::catalog::organizations_account_email(),
            Self::OrganizationsAccountRoleName => super::catalog::organizations_account_role_name(),
            Self::OrganizationsOrganizationArn => super::catalog::organizations_organization_arn(),
            Self::RedirectHost => carina_core::schema::types::redirect_host(),
            Self::RedirectPath => carina_core::schema::types::redirect_path(),
            Self::RedirectPort => carina_core::schema::types::redirect_port(),
            Self::RedirectProtocol => carina_core::schema::types::redirect_protocol(),
            Self::RedirectQuery => carina_core::schema::types::redirect_query(),
            Self::ResourceId => carina_aws_types::aws_resource_id(),
            Self::Route53HostedZoneId => return None,
            Self::S3BucketArn => super::catalog::s3_bucket_arn(),
            Self::S3BucketLifecycleDate => super::catalog::s3_lifecycle_date(),
            Self::SchedulerScheduleGroupName => super::catalog::scheduler_schedule_group_name(),
            Self::SchedulerScheduleScheduleExpression => {
                super::catalog::scheduler_schedule_expression()
            }
            Self::SsoInstanceArn => return None,
            Self::SsoPermissionSetArn => return None,
            Self::SsoPermissionSetSessionDuration => {
                super::catalog::sso_permission_set_session_duration()
            }
            Self::SsoPrincipalId => return None,
            Self::Wafv2WebACLByteMatchSearchString => {
                super::catalog::wafv2_byte_match_search_string()
            }
            Self::Wafv2WebACLByteMatchSearchStringBase64 => {
                super::catalog::wafv2_byte_match_search_string_base64()
            }
            Self::Wafv2WebAclArn => super::catalog::wafv2_web_acl_arn(),
        })
    }

    #[cfg(any(test, feature = "test-utils"))]
    pub fn callback_declaration_metadata(self) -> Option<StructuralStringMetadata> {
        let declaration = self.validation_declaration()?;
        match declaration.raw_shape() {
            RawShape::String {
                pattern, length, ..
            } => Some((pattern.map(str::to_string), length)),
            RawShape::Enum { base, .. } => match base.raw_shape() {
                RawShape::String {
                    pattern, length, ..
                } => Some((pattern.map(str::to_string), length)),
                _ => None,
            },
            _ => None,
        }
    }

    pub fn check(self) -> Option<CustomValidator> {
        use std::collections::BTreeMap;
        use std::sync::LazyLock;
        static CHECKS: LazyLock<BTreeMap<StringIdentity, Option<CustomValidator>>> =
            LazyLock::new(|| {
                StringIdentity::ALL
                    .iter()
                    .map(|identity| {
                        let check = identity.validation_declaration().and_then(|declaration| {
                            match declaration.raw_shape() {
                                RawShape::String { validate, .. }
                                | RawShape::Enum { validate, .. } => validate.cloned(),
                                _ => unreachable!("string identity check"),
                            }
                        });
                        (*identity, check)
                    })
                    .collect()
            });
        CHECKS[&self].clone()
    }
}
