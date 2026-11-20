//! Lower identities plus local CFN constraints to string declarations.
use super::*;

#[cfg(test)]
pub(super) fn from_constructor(source: &str) -> Option<Option<StringIdentity>> {
    Some(match source {
        "AttributeType::string()" => None,
        "crate::schemas::catalog::availability_zone()" => {
            Some(StringIdentity::AvailabilityZoneZoneName)
        }
        "crate::schemas::catalog::aws_region()" => Some(StringIdentity::Region),
        "crate::schemas::catalog::allocation_id()" => Some(StringIdentity::Ec2EipAllocationId),
        "crate::schemas::catalog::availability_zone_id()" => {
            Some(StringIdentity::AvailabilityZoneZoneId)
        }
        "crate::schemas::catalog::aws_account_id()" => Some(StringIdentity::AccountId),
        "crate::schemas::catalog::aws_resource_id()" => Some(StringIdentity::ResourceId),
        "crate::schemas::catalog::carrier_gateway_id()" => {
            Some(StringIdentity::Ec2CarrierGatewayId)
        }
        "crate::schemas::catalog::egress_only_internet_gateway_id()" => {
            Some(StringIdentity::Ec2EgressOnlyInternetGatewayId)
        }
        "crate::schemas::catalog::elbv2_target_group_grpc_code()" => {
            Some(StringIdentity::Elasticloadbalancingv2TargetGroupGrpcCode)
        }
        "crate::schemas::catalog::elbv2_target_group_http_code()" => {
            Some(StringIdentity::Elasticloadbalancingv2TargetGroupHttpCode)
        }
        "crate::schemas::catalog::flow_log_id()" => Some(StringIdentity::Ec2FlowLogId),
        "crate::schemas::catalog::iam_policy_name()" => Some(StringIdentity::IamPolicyName),
        "crate::schemas::catalog::iam_role_id()" => Some(StringIdentity::IamRoleId),
        "crate::schemas::catalog::iam_role_name()" => Some(StringIdentity::IamRoleName),
        "crate::schemas::catalog::identity_store_id()" => {
            Some(StringIdentity::IdentitystoreStoreId)
        }
        "crate::schemas::catalog::instance_id()" => Some(StringIdentity::Ec2InstanceId),
        "crate::schemas::catalog::internet_gateway_id()" => {
            Some(StringIdentity::Ec2InternetGatewayId)
        }
        "crate::schemas::catalog::ipam_id()" => Some(StringIdentity::Ec2IpamId),
        "crate::schemas::catalog::ipam_pool_id()" => Some(StringIdentity::Ec2IpamPoolId),
        "crate::schemas::catalog::local_gateway_id()" => Some(StringIdentity::Ec2LocalGatewayId),
        "crate::schemas::catalog::nat_gateway_id()" => Some(StringIdentity::Ec2NatGatewayId),
        "crate::schemas::catalog::network_acl_id()" => Some(StringIdentity::Ec2NetworkAclId),
        "crate::schemas::catalog::network_interface_id()" => {
            Some(StringIdentity::Ec2NetworkInterfaceId)
        }
        "crate::schemas::catalog::prefix_list_id()" => Some(StringIdentity::Ec2PrefixListId),
        "crate::schemas::catalog::route53_hosted_zone_id()" => {
            Some(StringIdentity::Route53HostedZoneId)
        }
        "crate::schemas::catalog::route_table_id()" => Some(StringIdentity::Ec2RouteTableId),
        "crate::schemas::catalog::security_group_id()" => Some(StringIdentity::Ec2SecurityGroupId),
        "crate::schemas::catalog::sso_principal_id()" => Some(StringIdentity::SsoPrincipalId),
        "crate::schemas::catalog::subnet_id()" => Some(StringIdentity::Ec2SubnetId),
        "crate::schemas::catalog::subnet_route_table_association_id()" => {
            Some(StringIdentity::Ec2SubnetRouteTableAssociationId)
        }
        "crate::schemas::catalog::tgw_route_table_id()" => {
            Some(StringIdentity::Ec2TransitGatewayRouteTableId)
        }
        "crate::schemas::catalog::transit_gateway_attachment_id()" => {
            Some(StringIdentity::Ec2TransitGatewayAttachmentId)
        }
        "crate::schemas::catalog::transit_gateway_id()" => {
            Some(StringIdentity::Ec2TransitGatewayId)
        }
        "crate::schemas::catalog::vpc_cidr_block_association_id()" => {
            Some(StringIdentity::Ec2VpcCidrBlockAssociationId)
        }
        "crate::schemas::catalog::vpc_endpoint_id()" => Some(StringIdentity::Ec2VpcEndpointId),
        "crate::schemas::catalog::vpc_id()" => Some(StringIdentity::Ec2VpcId),
        "crate::schemas::catalog::vpc_peering_connection_id()" => {
            Some(StringIdentity::Ec2VpcPeeringConnectionId)
        }
        "crate::schemas::catalog::vpn_gateway_id()" => Some(StringIdentity::Ec2VpnGatewayId),
        "carina_core::schema::types::domain_name()" => Some(StringIdentity::DomainName),
        "carina_core::schema::types::http_response_status_code()" => {
            Some(StringIdentity::HttpResponseStatusCode)
        }
        "carina_core::schema::types::https_url()" => Some(StringIdentity::HttpsUrl),
        "carina_core::schema::types::ipv4_address()" => Some(StringIdentity::Ipv4Address),
        "carina_core::schema::types::ipv6_address()" => Some(StringIdentity::Ipv6Address),
        "carina_core::schema::types::redirect_host()" => Some(StringIdentity::RedirectHost),
        "carina_core::schema::types::redirect_path()" => Some(StringIdentity::RedirectPath),
        "carina_core::schema::types::redirect_port()" => Some(StringIdentity::RedirectPort),
        "carina_core::schema::types::redirect_protocol()" => Some(StringIdentity::RedirectProtocol),
        "carina_core::schema::types::redirect_query()" => Some(StringIdentity::RedirectQuery),
        "crate::schemas::catalog::arn()" => Some(StringIdentity::Arn),
        "crate::schemas::catalog::cloudfront_distribution_arn()" => {
            Some(StringIdentity::CloudfrontDistributionArn)
        }
        "crate::schemas::catalog::dynamodb_table_arn()" => Some(StringIdentity::DynamodbTableArn),
        "crate::schemas::catalog::ec2_ipam_arn()" => Some(StringIdentity::Ec2IpamArn),
        "crate::schemas::catalog::ec2_ipam_pool_arn()" => Some(StringIdentity::Ec2IpamPoolArn),
        "crate::schemas::catalog::ecs_cluster_arn()" => Some(StringIdentity::EcsClusterArn),
        "crate::schemas::catalog::iam_oidc_provider_arn()" => {
            Some(StringIdentity::IamOidcProviderArn)
        }
        "crate::schemas::catalog::iam_policy_arn()" => Some(StringIdentity::IamPolicyArn),
        "crate::schemas::catalog::iam_role_arn()" => Some(StringIdentity::IamRoleArn),
        "crate::schemas::catalog::identitystore_group_display_name()" => {
            Some(StringIdentity::IdentitystoreGroupDisplayName)
        }
        "crate::schemas::catalog::kms_key_arn()" => Some(StringIdentity::KmsKeyArn),
        "crate::schemas::catalog::kms_key_id()" => Some(StringIdentity::KmsKeyId),
        "crate::schemas::catalog::logs_log_group_arn()" => Some(StringIdentity::LogsLogGroupArn),
        "crate::schemas::catalog::logs_log_group_name()" => {
            Some(StringIdentity::LogsLogGroupLogGroupName)
        }
        "crate::schemas::catalog::organizations_account_arn()" => {
            Some(StringIdentity::OrganizationsAccountArn)
        }
        "crate::schemas::catalog::organizations_account_email()" => {
            Some(StringIdentity::OrganizationsAccountEmail)
        }
        "crate::schemas::catalog::organizations_account_role_name()" => {
            Some(StringIdentity::OrganizationsAccountRoleName)
        }
        "crate::schemas::catalog::organizations_organization_arn()" => {
            Some(StringIdentity::OrganizationsOrganizationArn)
        }
        "crate::schemas::catalog::s3_bucket_arn()" => Some(StringIdentity::S3BucketArn),
        "crate::schemas::catalog::s3_lifecycle_date()" => {
            Some(StringIdentity::S3BucketLifecycleDate)
        }
        "crate::schemas::catalog::scheduler_schedule_expression()" => {
            Some(StringIdentity::SchedulerScheduleScheduleExpression)
        }
        "crate::schemas::catalog::scheduler_schedule_group_name()" => {
            Some(StringIdentity::SchedulerScheduleGroupName)
        }
        "crate::schemas::catalog::sso_instance_arn()" => Some(StringIdentity::SsoInstanceArn),
        "crate::schemas::catalog::sso_permission_set_arn()" => {
            Some(StringIdentity::SsoPermissionSetArn)
        }
        "crate::schemas::catalog::sso_permission_set_session_duration()" => {
            Some(StringIdentity::SsoPermissionSetSessionDuration)
        }
        "crate::schemas::catalog::wafv2_byte_match_search_string()" => {
            Some(StringIdentity::Wafv2WebACLByteMatchSearchString)
        }
        "crate::schemas::catalog::wafv2_byte_match_search_string_base64()" => {
            Some(StringIdentity::Wafv2WebACLByteMatchSearchStringBase64)
        }
        "crate::schemas::catalog::wafv2_web_acl_arn()" => Some(StringIdentity::Wafv2WebAclArn),
        "super::super::iam::oidc_provider::arn()" => Some(StringIdentity::IamOidcProviderArn),
        "super::super::iam::policy::arn()" => Some(StringIdentity::IamPolicyArn),
        "super::super::iam::role::arn()" => Some(StringIdentity::IamRoleArn),
        "super::super::kms::key::arn()" => Some(StringIdentity::KmsKeyArn),
        "types::email()" => Some(StringIdentity::Email),
        "types::ipv4_address()" => Some(StringIdentity::Ipv4Address),
        "types::ipv4_cidr()" => Some(StringIdentity::Ipv4Cidr),
        "types::ipv6_address()" => Some(StringIdentity::Ipv6Address),
        "types::ipv6_cidr()" => Some(StringIdentity::Ipv6Cidr),
        _ => return None,
    })
}

#[cfg(test)]
pub(super) fn core_constructor(identity: StringIdentity) -> Option<&'static str> {
    match identity {
        StringIdentity::DomainName => Some("carina_core::schema::types::domain_name()"),
        StringIdentity::Email => Some("carina_core::schema::types::email()"),
        StringIdentity::HttpResponseStatusCode => {
            Some("carina_core::schema::types::http_response_status_code()")
        }
        StringIdentity::HttpsUrl => Some("carina_core::schema::types::https_url()"),
        StringIdentity::Ipv4Address => Some("types::ipv4_address()"),
        StringIdentity::Ipv4Cidr => Some("types::ipv4_cidr()"),
        StringIdentity::Ipv6Address => Some("types::ipv6_address()"),
        StringIdentity::Ipv6Cidr => Some("types::ipv6_cidr()"),
        StringIdentity::RedirectHost => Some("carina_core::schema::types::redirect_host()"),
        StringIdentity::RedirectPath => Some("carina_core::schema::types::redirect_path()"),
        StringIdentity::RedirectPort => Some("carina_core::schema::types::redirect_port()"),
        StringIdentity::RedirectProtocol => Some("carina_core::schema::types::redirect_protocol()"),
        StringIdentity::RedirectQuery => Some("carina_core::schema::types::redirect_query()"),
        _ => None,
    }
}

pub(super) fn emit(identity: Option<StringIdentity>, constraints: &StringConstraints) -> String {
    let pattern = emit_pattern_option(constraints.pattern.as_ref());
    let length = emit_length_option(constraints.min_length, constraints.max_length);
    let Some(identity) = identity else {
        if !constraints.has_local_constraints() {
            return "AttributeType::string()".into();
        }
        return format!("AttributeType::refined_string(None, {pattern}, {length}, None)");
    };
    let name = format!("crate::schemas::string_identity::StringIdentity::{identity:?}");
    format!(
        "AttributeType::string_member({name}.identity_layer(), carina_core::schema::StringMemberConstraints::new({pattern}, {length}))"
    )
}

pub(super) fn emit_type(
    identity: Option<StringIdentity>,
    constraints: &StringConstraints,
) -> EmittedType {
    let pattern = constraints
        .pattern
        .as_ref()
        .map(|pattern| pattern.as_spec().clone());
    let length = type_emission::length(constraints.min_length, constraints.max_length);
    let ty = match identity {
        Some(identity) => carina_core::schema::AttributeType::string_member(
            identity.identity_layer(),
            carina_core::schema::StringMemberConstraints::new(pattern, length),
        ),
        None => carina_core::schema::AttributeType::refined_string(None, pattern, length, None),
    };
    EmittedType::new(emit(identity, constraints), ty)
}

impl StringConstraints {
    pub(super) fn has_local_constraints(&self) -> bool {
        self.pattern.is_some()
            || self.min_length.is_some_and(|n| n > 0)
            || self.max_length.is_some()
    }
}

#[cfg(test)]
pub(super) fn constructor(identity: StringIdentity) -> &'static str {
    match identity {
        StringIdentity::AccountId => "crate::schemas::catalog::aws_account_id()",
        StringIdentity::AvailabilityZoneZoneName => "crate::schemas::catalog::availability_zone()",
        StringIdentity::Region => "crate::schemas::catalog::aws_region()",
        StringIdentity::Arn => "crate::schemas::catalog::arn()",
        StringIdentity::AvailabilityZoneZoneId => "crate::schemas::catalog::availability_zone_id()",
        StringIdentity::CloudfrontDistributionArn => {
            "crate::schemas::catalog::cloudfront_distribution_arn()"
        }
        StringIdentity::DomainName => "carina_core::schema::types::domain_name()",
        StringIdentity::DynamodbTableArn => "crate::schemas::catalog::dynamodb_table_arn()",
        StringIdentity::Ec2CarrierGatewayId => "crate::schemas::catalog::carrier_gateway_id()",
        StringIdentity::Ec2EgressOnlyInternetGatewayId => {
            "crate::schemas::catalog::egress_only_internet_gateway_id()"
        }
        StringIdentity::Ec2EipAllocationId => "crate::schemas::catalog::allocation_id()",
        StringIdentity::Ec2FlowLogId => "crate::schemas::catalog::flow_log_id()",
        StringIdentity::Ec2InstanceId => "crate::schemas::catalog::instance_id()",
        StringIdentity::Ec2InternetGatewayId => "crate::schemas::catalog::internet_gateway_id()",
        StringIdentity::Ec2IpamArn => "crate::schemas::catalog::ec2_ipam_arn()",
        StringIdentity::Ec2IpamId => "crate::schemas::catalog::ipam_id()",
        StringIdentity::Ec2IpamPoolArn => "crate::schemas::catalog::ec2_ipam_pool_arn()",
        StringIdentity::Ec2IpamPoolId => "crate::schemas::catalog::ipam_pool_id()",
        StringIdentity::Ec2LocalGatewayId => "crate::schemas::catalog::local_gateway_id()",
        StringIdentity::Ec2NatGatewayId => "crate::schemas::catalog::nat_gateway_id()",
        StringIdentity::Ec2NetworkAclId => "crate::schemas::catalog::network_acl_id()",
        StringIdentity::Ec2NetworkInterfaceId => "crate::schemas::catalog::network_interface_id()",
        StringIdentity::Ec2PrefixListId => "crate::schemas::catalog::prefix_list_id()",
        StringIdentity::Ec2RouteTableId => "crate::schemas::catalog::route_table_id()",
        StringIdentity::Ec2SecurityGroupId => "crate::schemas::catalog::security_group_id()",
        StringIdentity::Ec2SubnetId => "crate::schemas::catalog::subnet_id()",
        StringIdentity::Ec2SubnetRouteTableAssociationId => {
            "crate::schemas::catalog::subnet_route_table_association_id()"
        }
        StringIdentity::Ec2TransitGatewayAttachmentId => {
            "crate::schemas::catalog::transit_gateway_attachment_id()"
        }
        StringIdentity::Ec2TransitGatewayId => "crate::schemas::catalog::transit_gateway_id()",
        StringIdentity::Ec2TransitGatewayRouteTableId => {
            "crate::schemas::catalog::tgw_route_table_id()"
        }
        StringIdentity::Ec2VpcCidrBlockAssociationId => {
            "crate::schemas::catalog::vpc_cidr_block_association_id()"
        }
        StringIdentity::Ec2VpcEndpointId => "crate::schemas::catalog::vpc_endpoint_id()",
        StringIdentity::Ec2VpcId => "crate::schemas::catalog::vpc_id()",
        StringIdentity::Ec2VpcPeeringConnectionId => {
            "crate::schemas::catalog::vpc_peering_connection_id()"
        }
        StringIdentity::Ec2VpnGatewayId => "crate::schemas::catalog::vpn_gateway_id()",
        StringIdentity::EcsClusterArn => "crate::schemas::catalog::ecs_cluster_arn()",
        StringIdentity::Elasticloadbalancingv2TargetGroupGrpcCode => {
            "crate::schemas::catalog::elbv2_target_group_grpc_code()"
        }
        StringIdentity::Elasticloadbalancingv2TargetGroupHttpCode => {
            "crate::schemas::catalog::elbv2_target_group_http_code()"
        }
        StringIdentity::Email => "types::email()",
        StringIdentity::HttpResponseStatusCode => {
            "carina_core::schema::types::http_response_status_code()"
        }
        StringIdentity::HttpsUrl => "carina_core::schema::types::https_url()",
        StringIdentity::IamOidcProviderArn => "super::super::iam::oidc_provider::arn()",
        StringIdentity::IamPolicyArn => "crate::schemas::catalog::iam_policy_arn()",
        StringIdentity::IamPolicyName => "crate::schemas::catalog::iam_policy_name()",
        StringIdentity::IamRoleArn => "super::super::iam::role::arn()",
        StringIdentity::IamRoleId => "crate::schemas::catalog::iam_role_id()",
        StringIdentity::IamRoleName => "crate::schemas::catalog::iam_role_name()",
        StringIdentity::IdentitystoreGroupDisplayName => {
            "crate::schemas::catalog::identitystore_group_display_name()"
        }
        StringIdentity::IdentitystoreStoreId => "crate::schemas::catalog::identity_store_id()",
        StringIdentity::Ipv4Address => "carina_core::schema::types::ipv4_address()",
        StringIdentity::Ipv4Cidr => "types::ipv4_cidr()",
        StringIdentity::Ipv6Address => "carina_core::schema::types::ipv6_address()",
        StringIdentity::Ipv6Cidr => "types::ipv6_cidr()",
        StringIdentity::KmsKeyArn => "crate::schemas::catalog::kms_key_arn()",
        StringIdentity::KmsKeyId => "crate::schemas::catalog::kms_key_id()",
        StringIdentity::LogsLogGroupArn => "crate::schemas::catalog::logs_log_group_arn()",
        StringIdentity::LogsLogGroupLogGroupName => {
            "crate::schemas::catalog::logs_log_group_name()"
        }
        StringIdentity::OrganizationsAccountArn => {
            "crate::schemas::catalog::organizations_account_arn()"
        }
        StringIdentity::OrganizationsAccountEmail => {
            "crate::schemas::catalog::organizations_account_email()"
        }
        StringIdentity::OrganizationsAccountRoleName => {
            "crate::schemas::catalog::organizations_account_role_name()"
        }
        StringIdentity::OrganizationsOrganizationArn => {
            "crate::schemas::catalog::organizations_organization_arn()"
        }
        StringIdentity::RedirectHost => "carina_core::schema::types::redirect_host()",
        StringIdentity::RedirectPath => "carina_core::schema::types::redirect_path()",
        StringIdentity::RedirectPort => "carina_core::schema::types::redirect_port()",
        StringIdentity::RedirectProtocol => "carina_core::schema::types::redirect_protocol()",
        StringIdentity::RedirectQuery => "carina_core::schema::types::redirect_query()",
        StringIdentity::ResourceId => "crate::schemas::catalog::aws_resource_id()",
        StringIdentity::Route53HostedZoneId => "crate::schemas::catalog::route53_hosted_zone_id()",
        StringIdentity::S3BucketArn => "crate::schemas::catalog::s3_bucket_arn()",
        StringIdentity::S3BucketLifecycleDate => "crate::schemas::catalog::s3_lifecycle_date()",
        StringIdentity::SchedulerScheduleGroupName => {
            "crate::schemas::catalog::scheduler_schedule_group_name()"
        }
        StringIdentity::SchedulerScheduleScheduleExpression => {
            "crate::schemas::catalog::scheduler_schedule_expression()"
        }
        StringIdentity::SsoInstanceArn => "crate::schemas::catalog::sso_instance_arn()",
        StringIdentity::SsoPermissionSetArn => "crate::schemas::catalog::sso_permission_set_arn()",
        StringIdentity::SsoPermissionSetSessionDuration => {
            "crate::schemas::catalog::sso_permission_set_session_duration()"
        }
        StringIdentity::SsoPrincipalId => "crate::schemas::catalog::sso_principal_id()",
        StringIdentity::Wafv2WebACLByteMatchSearchString => {
            "crate::schemas::catalog::wafv2_byte_match_search_string()"
        }
        StringIdentity::Wafv2WebACLByteMatchSearchStringBase64 => {
            "crate::schemas::catalog::wafv2_byte_match_search_string_base64()"
        }
        StringIdentity::Wafv2WebAclArn => "crate::schemas::catalog::wafv2_web_acl_arn()",
    }
}
