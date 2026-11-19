//! Catalog bindings for reusable String definitions, separate from member constraints.
use carina_core::schema::{AttributeType, CustomValidator, StringIdentityLayer, TypeIdentity};
#[cfg(any(test, feature = "test-utils"))]
use carina_core::schema::{RawShape, StringIdentityParts};

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
        let declaration = self.declaration();
        match declaration.raw_shape() {
            carina_core::schema::RawShape::String { identity_layer, .. } => identity_layer
                .identity
                .expect("catalog String must declare its identity")
                .clone(),
            carina_core::schema::RawShape::Enum { identity, .. } => identity.clone(),
            shape => panic!("String catalog declaration must be String or Enum: {shape:?}"),
        }
    }

    pub fn is_core_builtin(self) -> bool {
        self.identity().provider.is_none()
    }

    /// Catalog declarations have no knowledge of the CFN slot using them.
    pub(crate) fn declaration(self) -> AttributeType {
        match self {
            Self::AccountId => crate::schemas::catalog::aws_account_id(),
            Self::Arn => super::catalog::arn(),
            Self::AvailabilityZoneZoneId => crate::schemas::catalog::availability_zone_id(),
            Self::AvailabilityZoneZoneName => crate::schemas::catalog::availability_zone(),
            Self::Region => crate::schemas::catalog::aws_region(),
            Self::CloudfrontDistributionArn => super::catalog::cloudfront_distribution_arn(),
            Self::DomainName => carina_core::schema::types::domain_name(),
            Self::DynamodbTableArn => super::catalog::dynamodb_table_arn(),
            Self::Ec2CarrierGatewayId => crate::schemas::catalog::carrier_gateway_id(),
            Self::Ec2EgressOnlyInternetGatewayId => {
                crate::schemas::catalog::egress_only_internet_gateway_id()
            }
            Self::Ec2EipAllocationId => crate::schemas::catalog::allocation_id(),
            Self::Ec2FlowLogId => crate::schemas::catalog::flow_log_id(),
            Self::Ec2InstanceId => crate::schemas::catalog::instance_id(),
            Self::Ec2InternetGatewayId => crate::schemas::catalog::internet_gateway_id(),
            Self::Ec2IpamArn => super::catalog::ec2_ipam_arn(),
            Self::Ec2IpamId => crate::schemas::catalog::ipam_id(),
            Self::Ec2IpamPoolArn => super::catalog::ec2_ipam_pool_arn(),
            Self::Ec2IpamPoolId => crate::schemas::catalog::ipam_pool_id(),
            Self::Ec2LocalGatewayId => crate::schemas::catalog::local_gateway_id(),
            Self::Ec2NatGatewayId => crate::schemas::catalog::nat_gateway_id(),
            Self::Ec2NetworkAclId => crate::schemas::catalog::network_acl_id(),
            Self::Ec2NetworkInterfaceId => crate::schemas::catalog::network_interface_id(),
            Self::Ec2PrefixListId => crate::schemas::catalog::prefix_list_id(),
            Self::Ec2RouteTableId => crate::schemas::catalog::route_table_id(),
            Self::Ec2SecurityGroupId => crate::schemas::catalog::security_group_id(),
            Self::Ec2SubnetId => crate::schemas::catalog::subnet_id(),
            Self::Ec2SubnetRouteTableAssociationId => {
                crate::schemas::catalog::subnet_route_table_association_id()
            }
            Self::Ec2TransitGatewayAttachmentId => {
                crate::schemas::catalog::transit_gateway_attachment_id()
            }
            Self::Ec2TransitGatewayId => crate::schemas::catalog::transit_gateway_id(),
            Self::Ec2TransitGatewayRouteTableId => crate::schemas::catalog::tgw_route_table_id(),
            Self::Ec2VpcCidrBlockAssociationId => {
                crate::schemas::catalog::vpc_cidr_block_association_id()
            }
            Self::Ec2VpcEndpointId => crate::schemas::catalog::vpc_endpoint_id(),
            Self::Ec2VpcId => crate::schemas::catalog::vpc_id(),
            Self::Ec2VpcPeeringConnectionId => crate::schemas::catalog::vpc_peering_connection_id(),
            Self::Ec2VpnGatewayId => crate::schemas::catalog::vpn_gateway_id(),
            Self::EcsClusterArn => super::catalog::ecs_cluster_arn(),
            Self::Elasticloadbalancingv2TargetGroupGrpcCode => {
                crate::schemas::catalog::elbv2_target_group_grpc_code()
            }
            Self::Elasticloadbalancingv2TargetGroupHttpCode => {
                crate::schemas::catalog::elbv2_target_group_http_code()
            }
            Self::Email => carina_core::schema::types::email(),
            Self::HttpResponseStatusCode => carina_core::schema::types::http_response_status_code(),
            Self::HttpsUrl => carina_core::schema::types::https_url(),
            Self::IamOidcProviderArn => super::catalog::iam_oidc_provider_arn(),
            Self::IamPolicyArn => super::catalog::iam_policy_arn(),
            Self::IamPolicyName => crate::schemas::catalog::iam_policy_name(),
            Self::IamRoleArn => super::catalog::iam_role_arn(),
            Self::IamRoleId => crate::schemas::catalog::iam_role_id(),
            Self::IamRoleName => crate::schemas::catalog::iam_role_name(),
            Self::IdentitystoreGroupDisplayName => {
                super::catalog::identitystore_group_display_name()
            }
            Self::IdentitystoreStoreId => crate::schemas::catalog::identity_store_id(),
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
            Self::ResourceId => crate::schemas::catalog::aws_resource_id(),
            Self::Route53HostedZoneId => crate::schemas::catalog::route53_hosted_zone_id(),
            Self::S3BucketArn => super::catalog::s3_bucket_arn(),
            Self::S3BucketLifecycleDate => super::catalog::s3_lifecycle_date(),
            Self::SchedulerScheduleGroupName => super::catalog::scheduler_schedule_group_name(),
            Self::SchedulerScheduleScheduleExpression => {
                super::catalog::scheduler_schedule_expression()
            }
            Self::SsoInstanceArn => super::catalog::sso_instance_arn(),
            Self::SsoPermissionSetArn => super::catalog::sso_permission_set_arn(),
            Self::SsoPermissionSetSessionDuration => {
                super::catalog::sso_permission_set_session_duration()
            }
            Self::SsoPrincipalId => crate::schemas::catalog::sso_principal_id(),
            Self::Wafv2WebACLByteMatchSearchString => {
                super::catalog::wafv2_byte_match_search_string()
            }
            Self::Wafv2WebACLByteMatchSearchStringBase64 => {
                super::catalog::wafv2_byte_match_search_string_base64()
            }
            Self::Wafv2WebAclArn => super::catalog::wafv2_web_acl_arn(),
        }
    }

    /// The typed catalog layer cannot accept member patterns or bounds.
    pub fn identity_layer(self) -> StringIdentityLayer {
        super::catalog::string_identity_layer(self)
    }

    #[cfg(any(test, feature = "test-utils"))]
    pub fn callback_declaration_metadata(self) -> Option<StructuralStringMetadata> {
        let declaration = self.declaration();
        match declaration.raw_shape() {
            RawShape::String {
                identity_layer:
                    StringIdentityParts {
                        pattern, length, ..
                    },
                ..
            } => Some((pattern.map(|pattern| pattern.as_str().to_string()), length)),
            RawShape::Enum { base, .. } => match base.raw_shape() {
                RawShape::String {
                    identity_layer:
                        StringIdentityParts {
                            pattern, length, ..
                        },
                    ..
                } => Some((pattern.map(|pattern| pattern.as_str().to_string()), length)),
                _ => None,
            },
            _ => None,
        }
    }

    pub fn check(self) -> Option<CustomValidator> {
        self.identity_layer().parts().validate.cloned()
    }
}
