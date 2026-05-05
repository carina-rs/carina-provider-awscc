//! Auto-generated AWS Cloud Control resource schemas
//!
//! DO NOT EDIT MANUALLY - regenerate with:
//!   aws-vault exec <profile> -- ./carina-provider-awscc/scripts/generate-schemas.sh

use std::collections::HashMap;
use std::sync::LazyLock;

// Re-export all types and validators from awscc_types so that
// generated schema files can use `super::` to access them.
pub use super::awscc_types::*;

pub mod ec2;
pub mod iam;
pub mod identitystore;
pub mod logs;
pub mod organizations;
pub mod route53;
pub mod s3;
pub mod sso;

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

/// Function signature for enum alias reverse lookups.
type EnumAliasReverseFn = fn(&str, &str) -> Option<&'static str>;

/// Enum alias reverse dispatch table: resource_type -> dispatch function.
static ENUM_ALIAS_DISPATCH: LazyLock<HashMap<&'static str, EnumAliasReverseFn>> =
    LazyLock::new(|| {
        let entries: Vec<(&str, EnumAliasReverseFn)> = vec![
            ("ec2.Vpc", ec2::vpc::enum_alias_reverse),
            ("ec2.Subnet", ec2::subnet::enum_alias_reverse),
            (
                "ec2.InternetGateway",
                ec2::internet_gateway::enum_alias_reverse,
            ),
            ("ec2.RouteTable", ec2::route_table::enum_alias_reverse),
            ("ec2.Route", ec2::route::enum_alias_reverse),
            (
                "ec2.SubnetRouteTableAssociation",
                ec2::subnet_route_table_association::enum_alias_reverse,
            ),
            ("ec2.Eip", ec2::eip::enum_alias_reverse),
            ("ec2.NatGateway", ec2::nat_gateway::enum_alias_reverse),
            ("ec2.SecurityGroup", ec2::security_group::enum_alias_reverse),
            (
                "ec2.SecurityGroupIngress",
                ec2::security_group_ingress::enum_alias_reverse,
            ),
            (
                "ec2.SecurityGroupEgress",
                ec2::security_group_egress::enum_alias_reverse,
            ),
            ("ec2.VpcEndpoint", ec2::vpc_endpoint::enum_alias_reverse),
            (
                "ec2.VpcGatewayAttachment",
                ec2::vpc_gateway_attachment::enum_alias_reverse,
            ),
            ("ec2.FlowLog", ec2::flow_log::enum_alias_reverse),
            ("ec2.Ipam", ec2::ipam::enum_alias_reverse),
            ("ec2.IpamPool", ec2::ipam_pool::enum_alias_reverse),
            ("ec2.VpnGateway", ec2::vpn_gateway::enum_alias_reverse),
            (
                "ec2.TransitGateway",
                ec2::transit_gateway::enum_alias_reverse,
            ),
            (
                "ec2.VpcPeeringConnection",
                ec2::vpc_peering_connection::enum_alias_reverse,
            ),
            (
                "ec2.EgressOnlyInternetGateway",
                ec2::egress_only_internet_gateway::enum_alias_reverse,
            ),
            (
                "ec2.TransitGatewayAttachment",
                ec2::transit_gateway_attachment::enum_alias_reverse,
            ),
            ("s3.Bucket", s3::bucket::enum_alias_reverse),
            ("s3.BucketPolicy", s3::bucket_policy::enum_alias_reverse),
            ("iam.Role", iam::role::enum_alias_reverse),
            ("iam.RolePolicy", iam::role_policy::enum_alias_reverse),
            ("iam.OidcProvider", iam::oidc_provider::enum_alias_reverse),
            ("logs.LogGroup", logs::log_group::enum_alias_reverse),
            (
                "organizations.Organization",
                organizations::organization::enum_alias_reverse,
            ),
            (
                "organizations.Account",
                organizations::account::enum_alias_reverse,
            ),
            ("sso.Instance", sso::instance::enum_alias_reverse),
            ("sso.PermissionSet", sso::permission_set::enum_alias_reverse),
            ("sso.Assignment", sso::assignment::enum_alias_reverse),
            (
                "identitystore.Group",
                identitystore::group::enum_alias_reverse,
            ),
            (
                "identitystore.GroupMembership",
                identitystore::group_membership::enum_alias_reverse,
            ),
            (
                "route53.HostedZone",
                route53::hosted_zone::enum_alias_reverse,
            ),
        ];
        entries.into_iter().collect()
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

/// Maps DSL alias values back to canonical AWS values. O(1) dispatch.
/// Dispatches to per-module enum_alias_reverse() functions.
pub fn get_enum_alias_reverse(
    resource_type: &str,
    attr_name: &str,
    value: &str,
) -> Option<&'static str> {
    ENUM_ALIAS_DISPATCH
        .get(resource_type)
        .and_then(|f| f(attr_name, value))
}

/// Build a complete enum aliases map for all resource types.
/// Returns: resource_type -> attr_name -> alias -> canonical_value.
/// Used by CarinaProvider::enum_aliases() for the WASM host cache.
pub fn build_enum_aliases_map() -> HashMap<String, HashMap<String, HashMap<String, String>>> {
    let mut map: HashMap<String, HashMap<String, HashMap<String, String>>> = HashMap::new();
    for (attr, alias, canonical) in ec2::vpc::enum_alias_entries() {
        map.entry("ec2.Vpc".to_string())
            .or_default()
            .entry(attr.to_string())
            .or_default()
            .insert(alias.to_string(), canonical.to_string());
    }
    for (attr, alias, canonical) in ec2::subnet::enum_alias_entries() {
        map.entry("ec2.Subnet".to_string())
            .or_default()
            .entry(attr.to_string())
            .or_default()
            .insert(alias.to_string(), canonical.to_string());
    }
    for (attr, alias, canonical) in ec2::internet_gateway::enum_alias_entries() {
        map.entry("ec2.InternetGateway".to_string())
            .or_default()
            .entry(attr.to_string())
            .or_default()
            .insert(alias.to_string(), canonical.to_string());
    }
    for (attr, alias, canonical) in ec2::route_table::enum_alias_entries() {
        map.entry("ec2.RouteTable".to_string())
            .or_default()
            .entry(attr.to_string())
            .or_default()
            .insert(alias.to_string(), canonical.to_string());
    }
    for (attr, alias, canonical) in ec2::route::enum_alias_entries() {
        map.entry("ec2.Route".to_string())
            .or_default()
            .entry(attr.to_string())
            .or_default()
            .insert(alias.to_string(), canonical.to_string());
    }
    for (attr, alias, canonical) in ec2::subnet_route_table_association::enum_alias_entries() {
        map.entry("ec2.SubnetRouteTableAssociation".to_string())
            .or_default()
            .entry(attr.to_string())
            .or_default()
            .insert(alias.to_string(), canonical.to_string());
    }
    for (attr, alias, canonical) in ec2::eip::enum_alias_entries() {
        map.entry("ec2.Eip".to_string())
            .or_default()
            .entry(attr.to_string())
            .or_default()
            .insert(alias.to_string(), canonical.to_string());
    }
    for (attr, alias, canonical) in ec2::nat_gateway::enum_alias_entries() {
        map.entry("ec2.NatGateway".to_string())
            .or_default()
            .entry(attr.to_string())
            .or_default()
            .insert(alias.to_string(), canonical.to_string());
    }
    for (attr, alias, canonical) in ec2::security_group::enum_alias_entries() {
        map.entry("ec2.SecurityGroup".to_string())
            .or_default()
            .entry(attr.to_string())
            .or_default()
            .insert(alias.to_string(), canonical.to_string());
    }
    for (attr, alias, canonical) in ec2::security_group_ingress::enum_alias_entries() {
        map.entry("ec2.SecurityGroupIngress".to_string())
            .or_default()
            .entry(attr.to_string())
            .or_default()
            .insert(alias.to_string(), canonical.to_string());
    }
    for (attr, alias, canonical) in ec2::security_group_egress::enum_alias_entries() {
        map.entry("ec2.SecurityGroupEgress".to_string())
            .or_default()
            .entry(attr.to_string())
            .or_default()
            .insert(alias.to_string(), canonical.to_string());
    }
    for (attr, alias, canonical) in ec2::vpc_endpoint::enum_alias_entries() {
        map.entry("ec2.VpcEndpoint".to_string())
            .or_default()
            .entry(attr.to_string())
            .or_default()
            .insert(alias.to_string(), canonical.to_string());
    }
    for (attr, alias, canonical) in ec2::vpc_gateway_attachment::enum_alias_entries() {
        map.entry("ec2.VpcGatewayAttachment".to_string())
            .or_default()
            .entry(attr.to_string())
            .or_default()
            .insert(alias.to_string(), canonical.to_string());
    }
    for (attr, alias, canonical) in ec2::flow_log::enum_alias_entries() {
        map.entry("ec2.FlowLog".to_string())
            .or_default()
            .entry(attr.to_string())
            .or_default()
            .insert(alias.to_string(), canonical.to_string());
    }
    for (attr, alias, canonical) in ec2::ipam::enum_alias_entries() {
        map.entry("ec2.Ipam".to_string())
            .or_default()
            .entry(attr.to_string())
            .or_default()
            .insert(alias.to_string(), canonical.to_string());
    }
    for (attr, alias, canonical) in ec2::ipam_pool::enum_alias_entries() {
        map.entry("ec2.IpamPool".to_string())
            .or_default()
            .entry(attr.to_string())
            .or_default()
            .insert(alias.to_string(), canonical.to_string());
    }
    for (attr, alias, canonical) in ec2::vpn_gateway::enum_alias_entries() {
        map.entry("ec2.VpnGateway".to_string())
            .or_default()
            .entry(attr.to_string())
            .or_default()
            .insert(alias.to_string(), canonical.to_string());
    }
    for (attr, alias, canonical) in ec2::transit_gateway::enum_alias_entries() {
        map.entry("ec2.TransitGateway".to_string())
            .or_default()
            .entry(attr.to_string())
            .or_default()
            .insert(alias.to_string(), canonical.to_string());
    }
    for (attr, alias, canonical) in ec2::vpc_peering_connection::enum_alias_entries() {
        map.entry("ec2.VpcPeeringConnection".to_string())
            .or_default()
            .entry(attr.to_string())
            .or_default()
            .insert(alias.to_string(), canonical.to_string());
    }
    for (attr, alias, canonical) in ec2::egress_only_internet_gateway::enum_alias_entries() {
        map.entry("ec2.EgressOnlyInternetGateway".to_string())
            .or_default()
            .entry(attr.to_string())
            .or_default()
            .insert(alias.to_string(), canonical.to_string());
    }
    for (attr, alias, canonical) in ec2::transit_gateway_attachment::enum_alias_entries() {
        map.entry("ec2.TransitGatewayAttachment".to_string())
            .or_default()
            .entry(attr.to_string())
            .or_default()
            .insert(alias.to_string(), canonical.to_string());
    }
    for (attr, alias, canonical) in s3::bucket::enum_alias_entries() {
        map.entry("s3.Bucket".to_string())
            .or_default()
            .entry(attr.to_string())
            .or_default()
            .insert(alias.to_string(), canonical.to_string());
    }
    for (attr, alias, canonical) in s3::bucket_policy::enum_alias_entries() {
        map.entry("s3.BucketPolicy".to_string())
            .or_default()
            .entry(attr.to_string())
            .or_default()
            .insert(alias.to_string(), canonical.to_string());
    }
    for (attr, alias, canonical) in iam::role::enum_alias_entries() {
        map.entry("iam.Role".to_string())
            .or_default()
            .entry(attr.to_string())
            .or_default()
            .insert(alias.to_string(), canonical.to_string());
    }
    for (attr, alias, canonical) in iam::role_policy::enum_alias_entries() {
        map.entry("iam.RolePolicy".to_string())
            .or_default()
            .entry(attr.to_string())
            .or_default()
            .insert(alias.to_string(), canonical.to_string());
    }
    for (attr, alias, canonical) in iam::oidc_provider::enum_alias_entries() {
        map.entry("iam.OidcProvider".to_string())
            .or_default()
            .entry(attr.to_string())
            .or_default()
            .insert(alias.to_string(), canonical.to_string());
    }
    for (attr, alias, canonical) in logs::log_group::enum_alias_entries() {
        map.entry("logs.LogGroup".to_string())
            .or_default()
            .entry(attr.to_string())
            .or_default()
            .insert(alias.to_string(), canonical.to_string());
    }
    for (attr, alias, canonical) in organizations::organization::enum_alias_entries() {
        map.entry("organizations.Organization".to_string())
            .or_default()
            .entry(attr.to_string())
            .or_default()
            .insert(alias.to_string(), canonical.to_string());
    }
    for (attr, alias, canonical) in organizations::account::enum_alias_entries() {
        map.entry("organizations.Account".to_string())
            .or_default()
            .entry(attr.to_string())
            .or_default()
            .insert(alias.to_string(), canonical.to_string());
    }
    for (attr, alias, canonical) in sso::instance::enum_alias_entries() {
        map.entry("sso.Instance".to_string())
            .or_default()
            .entry(attr.to_string())
            .or_default()
            .insert(alias.to_string(), canonical.to_string());
    }
    for (attr, alias, canonical) in sso::permission_set::enum_alias_entries() {
        map.entry("sso.PermissionSet".to_string())
            .or_default()
            .entry(attr.to_string())
            .or_default()
            .insert(alias.to_string(), canonical.to_string());
    }
    for (attr, alias, canonical) in sso::assignment::enum_alias_entries() {
        map.entry("sso.Assignment".to_string())
            .or_default()
            .entry(attr.to_string())
            .or_default()
            .insert(alias.to_string(), canonical.to_string());
    }
    for (attr, alias, canonical) in identitystore::group::enum_alias_entries() {
        map.entry("identitystore.Group".to_string())
            .or_default()
            .entry(attr.to_string())
            .or_default()
            .insert(alias.to_string(), canonical.to_string());
    }
    for (attr, alias, canonical) in identitystore::group_membership::enum_alias_entries() {
        map.entry("identitystore.GroupMembership".to_string())
            .or_default()
            .entry(attr.to_string())
            .or_default()
            .insert(alias.to_string(), canonical.to_string());
    }
    for (attr, alias, canonical) in route53::hosted_zone::enum_alias_entries() {
        map.entry("route53.HostedZone".to_string())
            .or_default()
            .entry(attr.to_string())
            .or_default()
            .insert(alias.to_string(), canonical.to_string());
    }
    map
}
