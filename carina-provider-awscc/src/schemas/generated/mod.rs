//! Auto-generated AWS Cloud Control resource schemas
//!
//! DO NOT EDIT MANUALLY - regenerate with:
//!   aws-vault exec <profile> -- ./carina-provider-awscc/scripts/generate-schemas.sh

use std::collections::HashMap;
use std::sync::LazyLock;

// Re-export all types and validators from awscc_types so that
// generated schema files can use `super::` to access them.
pub use super::awscc_types::*;

pub mod cloudfront;
pub mod dynamodb;
pub mod ec2;
pub mod ecs;
pub mod iam;
pub mod identitystore;
pub mod kms;
pub mod logs;
pub mod organizations;
pub mod route53;
pub mod s3;
pub mod sso;
pub mod wafv2;

/// Cached schema configs, initialized once on first access.
static SCHEMA_CONFIGS: LazyLock<Vec<AwsccSchemaConfig>> = LazyLock::new(build_configs);

/// Index from resource_type_name (e.g., "ec2.vpc") to position in SCHEMA_CONFIGS.
static SCHEMA_CONFIG_INDEX: LazyLock<HashMap<&'static str, usize>> = LazyLock::new(|| {
    SCHEMA_CONFIGS
        .iter()
        .enumerate()
        .map(|(i, c)| (c.resource_type_name, i))
        .collect()
});

/// Cached enum valid values: resource_type -> (attr_name -> valid values slice).
static ENUM_VALID_VALUES: LazyLock<
    HashMap<&'static str, HashMap<&'static str, &'static [&'static str]>>,
> = LazyLock::new(|| {
    #[allow(clippy::type_complexity)]
    let modules: &[(&str, &[(&str, &[&str])])] = &[
        ec2::vpc::enum_valid_values(),
        ec2::subnet::enum_valid_values(),
        ec2::internet_gateway::enum_valid_values(),
        ec2::route_table::enum_valid_values(),
        ec2::route::enum_valid_values(),
        ec2::subnet_route_table_association::enum_valid_values(),
        ec2::eip::enum_valid_values(),
        ec2::nat_gateway::enum_valid_values(),
        ec2::security_group::enum_valid_values(),
        ec2::security_group_ingress::enum_valid_values(),
        ec2::security_group_egress::enum_valid_values(),
        ec2::vpc_endpoint::enum_valid_values(),
        ec2::vpc_gateway_attachment::enum_valid_values(),
        ec2::flow_log::enum_valid_values(),
        ec2::ipam::enum_valid_values(),
        ec2::ipam_pool::enum_valid_values(),
        ec2::vpn_gateway::enum_valid_values(),
        ec2::transit_gateway::enum_valid_values(),
        ec2::vpc_peering_connection::enum_valid_values(),
        ec2::egress_only_internet_gateway::enum_valid_values(),
        ec2::transit_gateway_attachment::enum_valid_values(),
        s3::bucket::enum_valid_values(),
        s3::bucket_policy::enum_valid_values(),
        iam::role::enum_valid_values(),
        iam::role_policy::enum_valid_values(),
        iam::oidc_provider::enum_valid_values(),
        logs::log_group::enum_valid_values(),
        organizations::organization::enum_valid_values(),
        organizations::account::enum_valid_values(),
        sso::instance::enum_valid_values(),
        sso::permission_set::enum_valid_values(),
        sso::assignment::enum_valid_values(),
        identitystore::group::enum_valid_values(),
        identitystore::group_membership::enum_valid_values(),
        route53::hosted_zone::enum_valid_values(),
        cloudfront::distribution::enum_valid_values(),
        cloudfront::origin_access_control::enum_valid_values(),
        wafv2::web_acl::enum_valid_values(),
        kms::key::enum_valid_values(),
        dynamodb::table::enum_valid_values(),
        ecs::cluster::enum_valid_values(),
    ];
    let mut map: HashMap<&str, HashMap<&str, &[&str]>> = HashMap::new();
    for (rt, attrs) in modules {
        let attr_map = map.entry(rt).or_default();
        for (attr, values) in *attrs {
            attr_map.insert(attr, values);
        }
    }
    map
});

/// Build all schema configs (called once by LazyLock).
fn build_configs() -> Vec<AwsccSchemaConfig> {
    vec![
        ec2::vpc::ec2_vpc_config(),
        ec2::subnet::ec2_subnet_config(),
        ec2::internet_gateway::ec2_internet_gateway_config(),
        ec2::route_table::ec2_route_table_config(),
        ec2::route::ec2_route_config(),
        ec2::subnet_route_table_association::ec2_subnet_route_table_association_config(),
        ec2::eip::ec2_eip_config(),
        ec2::nat_gateway::ec2_nat_gateway_config(),
        ec2::security_group::ec2_security_group_config(),
        ec2::security_group_ingress::ec2_security_group_ingress_config(),
        ec2::security_group_egress::ec2_security_group_egress_config(),
        ec2::vpc_endpoint::ec2_vpc_endpoint_config(),
        ec2::vpc_gateway_attachment::ec2_vpc_gateway_attachment_config(),
        ec2::flow_log::ec2_flow_log_config(),
        ec2::ipam::ec2_ipam_config(),
        ec2::ipam_pool::ec2_ipam_pool_config(),
        ec2::vpn_gateway::ec2_vpn_gateway_config(),
        ec2::transit_gateway::ec2_transit_gateway_config(),
        ec2::vpc_peering_connection::ec2_vpc_peering_connection_config(),
        ec2::egress_only_internet_gateway::ec2_egress_only_internet_gateway_config(),
        ec2::transit_gateway_attachment::ec2_transit_gateway_attachment_config(),
        s3::bucket::s3_bucket_config(),
        s3::bucket_policy::s3_bucket_policy_config(),
        iam::role::iam_role_config(),
        iam::role_policy::iam_role_policy_config(),
        iam::oidc_provider::iam_oidc_provider_config(),
        logs::log_group::logs_log_group_config(),
        organizations::organization::organizations_organization_config(),
        organizations::account::organizations_account_config(),
        sso::instance::sso_instance_config(),
        sso::permission_set::sso_permission_set_config(),
        sso::assignment::sso_assignment_config(),
        identitystore::group::identitystore_group_config(),
        identitystore::group_membership::identitystore_group_membership_config(),
        route53::hosted_zone::route53_hosted_zone_config(),
        cloudfront::distribution::cloudfront_distribution_config(),
        cloudfront::origin_access_control::cloudfront_origin_access_control_config(),
        wafv2::web_acl::wafv2_web_acl_config(),
        kms::key::kms_key_config(),
        dynamodb::table::dynamodb_table_config(),
        ecs::cluster::ecs_cluster_config(),
    ]
}

/// Returns a reference to the cached schema configs slice.
pub fn configs() -> &'static [AwsccSchemaConfig] {
    &SCHEMA_CONFIGS
}

/// Look up a schema config by resource_type_name (e.g., "ec2.vpc"). O(1).
pub fn get_config_by_type(resource_type: &str) -> Option<&'static AwsccSchemaConfig> {
    SCHEMA_CONFIG_INDEX
        .get(resource_type)
        .map(|&i| &SCHEMA_CONFIGS[i])
}

/// Get valid enum values for a given resource type and attribute name. O(1).
/// Used during read-back to normalize AWS-returned values to canonical DSL form.
///
/// Auto-generated from schema enum constants.
pub fn get_enum_valid_values(
    resource_type: &str,
    attr_name: &str,
) -> Option<&'static [&'static str]> {
    ENUM_VALID_VALUES
        .get(resource_type)
        .and_then(|attrs| attrs.get(attr_name))
        .copied()
}

// `get_enum_alias_reverse` and `build_enum_aliases_map` are no
// longer emitted — DSL → API canonical conversion now goes through
// `DslMap::api_for` against the exhaustive `dsl_aliases` table on
// each `StringEnum` (awscc#220). The single source of truth lives
// inline in `schemas/generated/<service>/<resource>.rs`.
