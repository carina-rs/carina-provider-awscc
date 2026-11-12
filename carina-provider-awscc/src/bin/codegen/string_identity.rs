//! Lower identities plus local CFN constraints to string declarations.
use super::*;

#[cfg(test)]
pub(super) fn from_constructor(source: &str) -> Option<Option<StringIdentity>> {
    Some(match source {
        "AttributeType::string()" => None,
        "carina_aws_types::availability_zone()" => Some(StringIdentity::AvailabilityZoneZoneName),
        "carina_aws_types::aws_region()" => Some(StringIdentity::Region),
        "carina_aws_types::allocation_id()" => Some(StringIdentity::Ec2EipAllocationId),
        "carina_aws_types::availability_zone_id()" => Some(StringIdentity::AvailabilityZoneZoneId),
        "carina_aws_types::aws_account_id()" => Some(StringIdentity::AccountId),
        "carina_aws_types::aws_resource_id()" => Some(StringIdentity::ResourceId),
        "carina_aws_types::carrier_gateway_id()" => Some(StringIdentity::Ec2CarrierGatewayId),
        "carina_aws_types::egress_only_internet_gateway_id()" => {
            Some(StringIdentity::Ec2EgressOnlyInternetGatewayId)
        }
        "carina_aws_types::elbv2_health_check_port()" => None,
        "carina_aws_types::elbv2_target_group_grpc_code()" => {
            Some(StringIdentity::Elasticloadbalancingv2TargetGroupGrpcCode)
        }
        "carina_aws_types::elbv2_target_group_http_code()" => {
            Some(StringIdentity::Elasticloadbalancingv2TargetGroupHttpCode)
        }
        "carina_aws_types::flow_log_id()" => Some(StringIdentity::Ec2FlowLogId),
        "carina_aws_types::iam_path()" => None,
        "carina_aws_types::iam_policy_name()" => Some(StringIdentity::IamPolicyName),
        "carina_aws_types::iam_role_id()" => Some(StringIdentity::IamRoleId),
        "carina_aws_types::iam_role_name()" => Some(StringIdentity::IamRoleName),
        "carina_aws_types::identity_store_id()" => Some(StringIdentity::IdentitystoreStoreId),
        "carina_aws_types::instance_id()" => Some(StringIdentity::Ec2InstanceId),
        "carina_aws_types::internet_gateway_id()" => Some(StringIdentity::Ec2InternetGatewayId),
        "carina_aws_types::ipam_id()" => Some(StringIdentity::Ec2IpamId),
        "carina_aws_types::ipam_pool_id()" => Some(StringIdentity::Ec2IpamPoolId),
        "carina_aws_types::local_gateway_id()" => Some(StringIdentity::Ec2LocalGatewayId),
        "carina_aws_types::nat_gateway_id()" => Some(StringIdentity::Ec2NatGatewayId),
        "carina_aws_types::network_acl_id()" => Some(StringIdentity::Ec2NetworkAclId),
        "carina_aws_types::network_interface_id()" => Some(StringIdentity::Ec2NetworkInterfaceId),
        "carina_aws_types::prefix_list_id()" => Some(StringIdentity::Ec2PrefixListId),
        "carina_aws_types::route53_hosted_zone_id()" => Some(StringIdentity::Route53HostedZoneId),
        "carina_aws_types::route_table_id()" => Some(StringIdentity::Ec2RouteTableId),
        "carina_aws_types::security_group_id()" => Some(StringIdentity::Ec2SecurityGroupId),
        "carina_aws_types::sso_principal_id()" => Some(StringIdentity::SsoPrincipalId),
        "carina_aws_types::subnet_id()" => Some(StringIdentity::Ec2SubnetId),
        "carina_aws_types::subnet_route_table_association_id()" => {
            Some(StringIdentity::Ec2SubnetRouteTableAssociationId)
        }
        "carina_aws_types::tgw_route_table_id()" => {
            Some(StringIdentity::Ec2TransitGatewayRouteTableId)
        }
        "carina_aws_types::transit_gateway_attachment_id()" => {
            Some(StringIdentity::Ec2TransitGatewayAttachmentId)
        }
        "carina_aws_types::transit_gateway_id()" => Some(StringIdentity::Ec2TransitGatewayId),
        "carina_aws_types::vpc_cidr_block_association_id()" => {
            Some(StringIdentity::Ec2VpcCidrBlockAssociationId)
        }
        "carina_aws_types::vpc_endpoint_id()" => Some(StringIdentity::Ec2VpcEndpointId),
        "carina_aws_types::vpc_id()" => Some(StringIdentity::Ec2VpcId),
        "carina_aws_types::vpc_peering_connection_id()" => {
            Some(StringIdentity::Ec2VpcPeeringConnectionId)
        }
        "carina_aws_types::vpn_gateway_id()" => Some(StringIdentity::Ec2VpnGatewayId),
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
        StringIdentity::AccountId => "carina_aws_types::aws_account_id()",
        StringIdentity::AvailabilityZoneZoneName => "carina_aws_types::availability_zone()",
        StringIdentity::Region => "carina_aws_types::aws_region()",
        StringIdentity::Arn => "crate::schemas::catalog::arn()",
        StringIdentity::AvailabilityZoneZoneId => "carina_aws_types::availability_zone_id()",
        StringIdentity::CloudfrontDistributionArn => {
            "crate::schemas::catalog::cloudfront_distribution_arn()"
        }
        StringIdentity::DomainName => "carina_core::schema::types::domain_name()",
        StringIdentity::DynamodbTableArn => "crate::schemas::catalog::dynamodb_table_arn()",
        StringIdentity::Ec2CarrierGatewayId => "carina_aws_types::carrier_gateway_id()",
        StringIdentity::Ec2EgressOnlyInternetGatewayId => {
            "carina_aws_types::egress_only_internet_gateway_id()"
        }
        StringIdentity::Ec2EipAllocationId => "carina_aws_types::allocation_id()",
        StringIdentity::Ec2FlowLogId => "carina_aws_types::flow_log_id()",
        StringIdentity::Ec2InstanceId => "carina_aws_types::instance_id()",
        StringIdentity::Ec2InternetGatewayId => "carina_aws_types::internet_gateway_id()",
        StringIdentity::Ec2IpamArn => "crate::schemas::catalog::ec2_ipam_arn()",
        StringIdentity::Ec2IpamId => "carina_aws_types::ipam_id()",
        StringIdentity::Ec2IpamPoolArn => "crate::schemas::catalog::ec2_ipam_pool_arn()",
        StringIdentity::Ec2IpamPoolId => "carina_aws_types::ipam_pool_id()",
        StringIdentity::Ec2LocalGatewayId => "carina_aws_types::local_gateway_id()",
        StringIdentity::Ec2NatGatewayId => "carina_aws_types::nat_gateway_id()",
        StringIdentity::Ec2NetworkAclId => "carina_aws_types::network_acl_id()",
        StringIdentity::Ec2NetworkInterfaceId => "carina_aws_types::network_interface_id()",
        StringIdentity::Ec2PrefixListId => "carina_aws_types::prefix_list_id()",
        StringIdentity::Ec2RouteTableId => "carina_aws_types::route_table_id()",
        StringIdentity::Ec2SecurityGroupId => "carina_aws_types::security_group_id()",
        StringIdentity::Ec2SubnetId => "carina_aws_types::subnet_id()",
        StringIdentity::Ec2SubnetRouteTableAssociationId => {
            "carina_aws_types::subnet_route_table_association_id()"
        }
        StringIdentity::Ec2TransitGatewayAttachmentId => {
            "carina_aws_types::transit_gateway_attachment_id()"
        }
        StringIdentity::Ec2TransitGatewayId => "carina_aws_types::transit_gateway_id()",
        StringIdentity::Ec2TransitGatewayRouteTableId => "carina_aws_types::tgw_route_table_id()",
        StringIdentity::Ec2VpcCidrBlockAssociationId => {
            "carina_aws_types::vpc_cidr_block_association_id()"
        }
        StringIdentity::Ec2VpcEndpointId => "carina_aws_types::vpc_endpoint_id()",
        StringIdentity::Ec2VpcId => "carina_aws_types::vpc_id()",
        StringIdentity::Ec2VpcPeeringConnectionId => {
            "carina_aws_types::vpc_peering_connection_id()"
        }
        StringIdentity::Ec2VpnGatewayId => "carina_aws_types::vpn_gateway_id()",
        StringIdentity::EcsClusterArn => "crate::schemas::catalog::ecs_cluster_arn()",
        StringIdentity::Elasticloadbalancingv2TargetGroupGrpcCode => {
            "carina_aws_types::elbv2_target_group_grpc_code()"
        }
        StringIdentity::Elasticloadbalancingv2TargetGroupHttpCode => {
            "carina_aws_types::elbv2_target_group_http_code()"
        }
        StringIdentity::Email => "types::email()",
        StringIdentity::HttpResponseStatusCode => {
            "carina_core::schema::types::http_response_status_code()"
        }
        StringIdentity::HttpsUrl => "carina_core::schema::types::https_url()",
        StringIdentity::IamOidcProviderArn => "super::super::iam::oidc_provider::arn()",
        StringIdentity::IamPolicyArn => "crate::schemas::catalog::iam_policy_arn()",
        StringIdentity::IamPolicyName => "carina_aws_types::iam_policy_name()",
        StringIdentity::IamRoleArn => "super::super::iam::role::arn()",
        StringIdentity::IamRoleId => "carina_aws_types::iam_role_id()",
        StringIdentity::IamRoleName => "carina_aws_types::iam_role_name()",
        StringIdentity::IdentitystoreGroupDisplayName => {
            "crate::schemas::catalog::identitystore_group_display_name()"
        }
        StringIdentity::IdentitystoreStoreId => "carina_aws_types::identity_store_id()",
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
        StringIdentity::ResourceId => "carina_aws_types::aws_resource_id()",
        StringIdentity::Route53HostedZoneId => "carina_aws_types::route53_hosted_zone_id()",
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
        StringIdentity::SsoPrincipalId => "carina_aws_types::sso_principal_id()",
        StringIdentity::Wafv2WebACLByteMatchSearchString => {
            "crate::schemas::catalog::wafv2_byte_match_search_string()"
        }
        StringIdentity::Wafv2WebACLByteMatchSearchStringBase64 => {
            "crate::schemas::catalog::wafv2_byte_match_search_string_base64()"
        }
        StringIdentity::Wafv2WebAclArn => "crate::schemas::catalog::wafv2_web_acl_arn()",
    }
}
