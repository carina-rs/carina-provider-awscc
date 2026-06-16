//! load_balancer schema definition for AWS Cloud Control
//!
//! Auto-generated from CloudFormation schema: AWS::ElasticLoadBalancingV2::LoadBalancer
//!
//! DO NOT EDIT MANUALLY - regenerate with carina-codegen

use crate::schemas::config::AwsccSchemaConfig;
use carina_core::resource::{ConcreteValue, Value};
use carina_core::schema::{AttributeSchema, AttributeType, ResourceSchema, StructField};

const VALID_ENABLE_PREFIX_FOR_IPV6_SOURCE_NAT: &[&str] = &["on", "off"];

const VALID_ENFORCE_SECURITY_GROUP_INBOUND_RULES_ON_PRIVATE_LINK_TRAFFIC: &[&str] = &["on", "off"];

const VALID_IP_ADDRESS_TYPE: &[&str] = &["ipv4", "dualstack", "dualstack-without-public-ipv4"];

const VALID_SCHEME: &[&str] = &["internet-facing", "internal"];

const VALID_TYPE: &[&str] = &["application", "network", "gateway"];

/// Returns the schema config for elasticloadbalancingv2_load_balancer (AWS::ElasticLoadBalancingV2::LoadBalancer)
pub fn elasticloadbalancingv2_load_balancer_config() -> AwsccSchemaConfig {
    AwsccSchemaConfig {
        aws_type_name: "AWS::ElasticLoadBalancingV2::LoadBalancer",
        resource_type_name: "elasticloadbalancingv2.LoadBalancer",
        primary_identifier: &["LoadBalancerArn"],
        has_tags: true,
        schema: ResourceSchema::new("elasticloadbalancingv2.LoadBalancer")
        .with_description("Specifies an Application Load Balancer, a Network Load Balancer, or a Gateway Load Balancer.")
        .attribute(
            AttributeSchema::new("canonical_hosted_zone_id", AttributeType::string())
                .read_only()
                .with_description(" (read-only)")
                .with_provider_name("CanonicalHostedZoneID"),
        )
        .attribute(
            AttributeSchema::new("dns_name", AttributeType::string())
                .read_only()
                .with_description(" (read-only)")
                .with_provider_name("DNSName"),
        )
        .attribute(
            AttributeSchema::new("enable_capacity_reservation_provision_stabilize", AttributeType::bool())
                .write_only()
                .with_description("Indicates whether to enable stabilization when creating or updating an LCU reservation. This ensures that the final stack status reflects the status of the LCU reservation. The default is ``false``.")
                .with_provider_name("EnableCapacityReservationProvisionStabilize")
                .with_default(Value::Concrete(ConcreteValue::Bool(false))),
        )
        .attribute(
            AttributeSchema::new("enable_prefix_for_ipv6_source_nat", AttributeType::enum_(carina_core::schema::enum_identity("EnablePrefixForIpv6SourceNat", Some("aws.elasticloadbalancingv2.LoadBalancer")), Some(vec!["on".to_string(), "off".to_string()]), vec![("on".to_string(), "on".to_string()), ("off".to_string(), "off".to_string())], None, None))
                .with_description("[Network Load Balancers with UDP listeners] Indicates whether to use an IPv6 prefix from each subnet for source NAT. The IP address type must be ``dualstack``. The default value is ``off``.")
                .with_provider_name("EnablePrefixForIpv6SourceNat"),
        )
        .attribute(
            AttributeSchema::new("enforce_security_group_inbound_rules_on_private_link_traffic", AttributeType::enum_(carina_core::schema::enum_identity("EnforceSecurityGroupInboundRulesOnPrivateLinkTraffic", Some("aws.elasticloadbalancingv2.LoadBalancer")), Some(vec!["on".to_string(), "off".to_string()]), vec![("on".to_string(), "on".to_string()), ("off".to_string(), "off".to_string())], None, None))
                .with_description("Indicates whether to evaluate inbound security group rules for traffic sent to a Network Load Balancer through privatelink. The default is ``on``. You can't configure this property on a Network Load Balancer unless you associated a security group with the load balancer when you created it.")
                .with_provider_name("EnforceSecurityGroupInboundRulesOnPrivateLinkTraffic"),
        )
        .attribute(
            AttributeSchema::new("ip_address_type", AttributeType::enum_(carina_core::schema::enum_identity("IpAddressType", Some("aws.elasticloadbalancingv2.LoadBalancer")), Some(vec!["ipv4".to_string(), "dualstack".to_string(), "dualstack-without-public-ipv4".to_string()]), vec![("ipv4".to_string(), "ipv4".to_string()), ("dualstack".to_string(), "dualstack".to_string()), ("dualstack-without-public-ipv4".to_string(), "dualstack_without_public_ipv4".to_string())], None, None))
                .with_description("The IP address type. Internal load balancers must use ``ipv4``. [Application Load Balancers] The possible values are ``ipv4`` (IPv4 addresses), ``dualstack`` (IPv4 and IPv6 addresses), and ``dualstack-without-public-ipv4`` (public IPv6 addresses and private IPv4 and IPv6 addresses). Application Load Balancer authentication supports IPv4 addresses only when connecting to an Identity Provider (IdP) or Amazon Cognito endpoint. Without a public IPv4 address the load balancer can't complete the authentication process, resulting in HTTP 500 errors. [Network Load Balancers and Gateway Load Balancers] The possible values are ``ipv4`` (IPv4 addresses) and ``dualstack`` (IPv4 and IPv6 addresses).")
                .with_provider_name("IpAddressType"),
        )
        .attribute(
            AttributeSchema::new("ipv4_ipam_pool_id", carina_aws_types::ipam_pool_id())
                .with_description("The ID of the IPv4 IPAM pool.")
                .with_provider_name("Ipv4IpamPoolId"),
        )
        .attribute(
            AttributeSchema::new("load_balancer_arn", carina_aws_types::arn())
                .read_only()
                .with_description(" (read-only)")
                .with_provider_name("LoadBalancerArn"),
        )
        .attribute(
            AttributeSchema::new("load_balancer_attributes", AttributeType::unordered_list(AttributeType::struct_("LoadBalancerAttribute".to_string(), vec![StructField::new("key", AttributeType::string()).with_description("The name of the attribute. The following attributes are supported by all load balancers: + ``deletion_protection.enabled`` - Indicates whether deletion protection is enabled. The value is ``true`` or ``false``. The default is ``false``. + ``load_balancing.cross_zone.enabled`` - Indicates whether cross-zone load balancing is enabled. The possible values are ``true`` and ``false``. The default for Network Load Balancers and Gateway Load Balancers is ``false``. The default for Application Load Balancers is ``true``, and can't be changed. The following attributes are supported by both Application Load Balancers and Network Load Balancers: + ``access_logs.s3.enabled`` - Indicates whether access logs are enabled. The value is ``true`` or ``false``. The default is ``false``. + ``access_logs.s3.bucket`` - The name of the S3 bucket for the access logs. This attribute is required if access logs are enabled. The bucket must exist in the same region as the load balancer and have a bucket policy that grants Elastic Load Balancing permissions to write to the bucket. + ``access_logs.s3.prefix`` - The prefix for the location in the S3 bucket for the access logs. + ``ipv6.deny_all_igw_traffic`` - Blocks internet gateway (IGW) access to the load balancer. It is set to ``false`` for internet-facing load balancers and ``true`` for internal load balancers, preventing unintended access to your internal load balancer through an internet gateway. + ``zonal_shift.config.enabled`` - Indicates whether zonal shift is enabled. The possible values are ``true`` and ``false``. The default is ``false``. The following attributes are supported by only Application Load Balancers: + ``idle_timeout.timeout_seconds`` - The idle timeout value, in seconds. The valid range is 1-4000 seconds. The default is 60 seconds. + ``client_keep_alive.seconds`` - The client keep alive value, in seconds. The valid range is 60-604800 seconds. The default is 3600 seconds. + ``connection_logs.s3.enabled`` - Indicates whether connection logs are enabled. The value is ``true`` or ``false``. The default is ``false``. + ``connection_logs.s3.bucket`` - The name of the S3 bucket for the connection logs. This attribute is required if connection logs are enabled. The bucket must exist in the same region as the load balancer and have a bucket policy that grants Elastic Load Balancing permissions to write to the bucket. + ``connection_logs.s3.prefix`` - The prefix for the location in the S3 bucket for the connection logs. + ``health_check_logs.s3.enabled`` - Indicates whether health check logs are enabled. The value is ``true`` or ``false``. The default is ``false``. + ``health_check_logs.s3.bucket`` - The name of the S3 bucket for the health check logs. This attribute is required if health check logs are enabled. The bucket must exist in the same region as the load balancer and have a bucket policy that grants Elastic Load Balancing permissions to write to the bucket. + ``health_check_logs.s3.prefix`` - The prefix for the location in the S3 bucket for the health check logs. + ``routing.http.desync_mitigation_mode`` - Determines how the load balancer handles requests that might pose a security risk to your application. The possible values are ``monitor``, ``defensive``, and ``strictest``. The default is ``defensive``. + ``routing.http.drop_invalid_header_fields.enabled`` - Indicates whether HTTP headers with invalid header fields are removed by the load balancer (``true``) or routed to targets (``false``). The default is ``false``. + ``routing.http.preserve_host_header.enabled`` - Indicates whether the Application Load Balancer should preserve the ``Host`` header in the HTTP request and send it to the target without any change. The possible values are ``true`` and ``false``. The default is ``false``. + ``routing.http.x_amzn_tls_version_and_cipher_suite.enabled`` - Indicates whether the two headers (``x-amzn-tls-version`` and ``x-amzn-tls-cipher-suite``), which contain information about the negotiated TLS version and cipher suite, are added to the client request before sending it to the target. The ``x-amzn-tls-version`` header has information about the TLS protocol version negotiated with the client, and the ``x-amzn-tls-cipher-suite`` header has information about the cipher suite negotiated with the client. Both headers are in OpenSSL format. The possible values for the attribute are ``true`` and ``false``. The default is ``false``. + ``routing.http.xff_client_port.enabled`` - Indicates whether the ``X-Forwarded-For`` header should preserve the source port that the client used to connect to the load balancer. The possible values are ``true`` and ``false``. The default is ``false``. + ``routing.http.xff_header_processing.mode`` - Enables you to modify, preserve, or remove the ``X-Forwarded-For`` header in the HTTP request before the Application Load Balancer sends the request to the target. The possible values are ``append``, ``preserve``, and ``remove``. The default is ``append``. + If the value is ``append``, the Application Load Balancer adds the client IP address (of the last hop) to the ``X-Forwarded-For`` header in the HTTP request before it sends it to targets. + If the value is ``preserve`` the Application Load Balancer preserves the ``X-Forwarded-For`` header in the HTTP request, and sends it to targets without any change. + If the value is ``remove``, the Application Load Balancer removes the ``X-Forwarded-For`` header in the HTTP request before it sends it to targets. + ``routing.http2.enabled`` - Indicates whether clients can connect to the load balancer using HTTP/2. If ``true``, clients can connect using HTTP/2 or HTTP/1.1. However, all client requests are subject to the stricter HTTP/2 header validation rules. For example, message header names must contain only alphanumeric characters and hyphens. If ``false``, clients must connect using HTTP/1.1. The default is ``true``. + ``waf.fail_open.enabled`` - Indicates whether to allow a WAF-enabled load balancer to route requests to targets if it is unable to forward the request to AWS WAF. The possible values are ``true`` and ``false``. The default is ``false``. The following attributes are supported by only Network Load Balancers: + ``dns_record.client_routing_policy`` - Indicates how traffic is distributed among the load balancer Availability Zones. The possible values are ``availability_zone_affinity`` with 100 percent zonal affinity, ``partial_availability_zone_affinity`` with 85 percent zonal affinity, and ``any_availability_zone`` with 0 percent zonal affinity. + ``secondary_ips.auto_assigned.per_subnet`` - The number of secondary IP addresses to configure for your load balancer nodes. Use to address port allocation errors if you can't add targets. The valid range is 0 to 7. The default is 0. After you set this value, you can't decrease it.").with_provider_name("Key"),
                    StructField::new("value", AttributeType::string()).with_description("The value of the attribute.").with_provider_name("Value")])))
                .with_description("The load balancer attributes. Attributes that you do not modify retain their current values.")
                .with_provider_name("LoadBalancerAttributes")
                .with_block_name("load_balancer_attribute"),
        )
        .attribute(
            AttributeSchema::new("load_balancer_full_name", AttributeType::string())
                .read_only()
                .with_description(" (read-only)")
                .with_provider_name("LoadBalancerFullName"),
        )
        .attribute(
            AttributeSchema::new("load_balancer_name", AttributeType::string())
                .read_only()
                .with_description(" (read-only)")
                .with_provider_name("LoadBalancerName"),
        )
        .attribute(
            AttributeSchema::new("minimum_load_balancer_capacity", AttributeType::struct_("MinimumLoadBalancerCapacity".to_string(), vec![StructField::new("capacity_units", AttributeType::int()).required().with_description("The number of capacity units.").with_provider_name("CapacityUnits")]))
                .with_description("The minimum capacity for a load balancer.")
                .with_provider_name("MinimumLoadBalancerCapacity"),
        )
        .attribute(
            AttributeSchema::new("name", AttributeType::string())
                .create_only()
                .with_description("The name of the load balancer. This name must be unique per region per account, can have a maximum of 32 characters, must contain only alphanumeric characters or hyphens, must not begin or end with a hyphen, and must not begin with \"internal-\". If you don't specify a name, AWS CloudFormation generates a unique physical ID for the load balancer. If you specify a name, you cannot perform updates that require replacement of this resource, but you can perform other updates. To replace the resource, specify a new name.")
                .with_provider_name("Name"),
        )
        .attribute(
            AttributeSchema::new("scheme", AttributeType::enum_(carina_core::schema::enum_identity("Scheme", Some("aws.elasticloadbalancingv2.LoadBalancer")), Some(vec!["internet-facing".to_string(), "internal".to_string()]), vec![("internet-facing".to_string(), "internet_facing".to_string()), ("internal".to_string(), "internal".to_string())], None, None))
                .create_only()
                .with_description("The nodes of an Internet-facing load balancer have public IP addresses. The DNS name of an Internet-facing load balancer is publicly resolvable to the public IP addresses of the nodes. Therefore, Internet-facing load balancers can route requests from clients over the internet. The nodes of an internal load balancer have only private IP addresses. The DNS name of an internal load balancer is publicly resolvable to the private IP addresses of the nodes. Therefore, internal load balancers can route requests only from clients with access to the VPC for the load balancer. The default is an Internet-facing load balancer. You can't specify a scheme for a Gateway Load Balancer.")
                .with_provider_name("Scheme"),
        )
        .attribute(
            AttributeSchema::new("security_groups", AttributeType::unordered_list(AttributeType::string()))
                .with_description("[Application Load Balancers and Network Load Balancers] The IDs of the security groups for the load balancer.")
                .with_provider_name("SecurityGroups"),
        )
        .attribute(
            AttributeSchema::new("subnet_mappings", AttributeType::unordered_list(AttributeType::struct_("SubnetMapping".to_string(), vec![StructField::new("allocation_id", carina_aws_types::allocation_id()).with_description("[Network Load Balancers] The allocation ID of the Elastic IP address for an internet-facing load balancer.").with_provider_name("AllocationId"),
                    StructField::new("i_pv6_address", AttributeType::string()).with_description("[Network Load Balancers] The IPv6 address.").with_provider_name("IPv6Address"),
                    StructField::new("private_i_pv4_address", AttributeType::string()).with_description("[Network Load Balancers] The private IPv4 address for an internal load balancer.").with_provider_name("PrivateIPv4Address"),
                    StructField::new("source_nat_ipv6_prefix", AttributeType::string()).with_description("[Network Load Balancers with UDP listeners] The IPv6 prefix to use for source NAT. Specify an IPv6 prefix (/80 netmask) from the subnet CIDR block or ``auto_assigned`` to use an IPv6 prefix selected at random from the subnet CIDR block.").with_provider_name("SourceNatIpv6Prefix"),
                    StructField::new("subnet_id", carina_aws_types::subnet_id()).required().with_description("The ID of the subnet.").with_provider_name("SubnetId")])))
                .with_description("The IDs of the subnets. You can specify only one subnet per Availability Zone. You must specify either subnets or subnet mappings, but not both. [Application Load Balancers] You must specify subnets from at least two Availability Zones. You can't specify Elastic IP addresses for your subnets. [Application Load Balancers on Outposts] You must specify one Outpost subnet. [Application Load Balancers on Local Zones] You can specify subnets from one or more Local Zones. [Network Load Balancers] You can specify subnets from one or more Availability Zones. You can specify one Elastic IP address per subnet if you need static IP addresses for your internet-facing load balancer. For internal load balancers, you can specify one private IP address per subnet from the IPv4 range of the subnet. For internet-facing load balancer, you can specify one IPv6 address per subnet. [Gateway Load Balancers] You can specify subnets from one or more Availability Zones. You can't specify Elastic IP addresses for your subnets.")
                .with_provider_name("SubnetMappings")
                .with_block_name("subnet_mapping"),
        )
        .attribute(
            AttributeSchema::new("subnets", AttributeType::unordered_list(AttributeType::string()))
                .with_description("The IDs of the subnets. You can specify only one subnet per Availability Zone. You must specify either subnets or subnet mappings, but not both. To specify an Elastic IP address, specify subnet mappings instead of subnets. [Application Load Balancers] You must specify subnets from at least two Availability Zones. [Application Load Balancers on Outposts] You must specify one Outpost subnet. [Application Load Balancers on Local Zones] You can specify subnets from one or more Local Zones. [Network Load Balancers and Gateway Load Balancers] You can specify subnets from one or more Availability Zones.")
                .with_provider_name("Subnets"),
        )
        .attribute(
            AttributeSchema::new("tags", carina_aws_types::tags_type())
                .with_description("The tags to assign to the load balancer.")
                .with_provider_name("Tags")
                .with_block_name("tag"),
        )
        .attribute(
            AttributeSchema::new("type", AttributeType::enum_(carina_core::schema::enum_identity("Type", Some("aws.elasticloadbalancingv2.LoadBalancer")), Some(vec!["application".to_string(), "network".to_string(), "gateway".to_string()]), vec![("application".to_string(), "application".to_string()), ("network".to_string(), "network".to_string()), ("gateway".to_string(), "gateway".to_string())], None, None))
                .create_only()
                .with_description("The type of load balancer. The default is ``application``.")
                .with_provider_name("Type"),
        )
        .with_validator(|attrs| {
            let mut errors = Vec::new();
            if let Err(mut e) = carina_aws_types::validate_tags_map(attrs) {
                errors.append(&mut e);
            }
            if errors.is_empty() { Ok(()) } else { Err(errors) }
        })
        .with_def("MinimumLoadBalancerCapacity", AttributeType::struct_("MinimumLoadBalancerCapacity".to_string(), vec![StructField::new("capacity_units", AttributeType::int()).required().with_description("The number of capacity units.").with_provider_name("CapacityUnits")]))
    }
}

/// Returns the resource type name and all enum valid values for this module
pub fn enum_valid_values() -> (
    &'static str,
    &'static [(&'static str, &'static [&'static str])],
) {
    (
        "elasticloadbalancingv2.LoadBalancer",
        &[
            (
                "enable_prefix_for_ipv6_source_nat",
                VALID_ENABLE_PREFIX_FOR_IPV6_SOURCE_NAT,
            ),
            (
                "enforce_security_group_inbound_rules_on_private_link_traffic",
                VALID_ENFORCE_SECURITY_GROUP_INBOUND_RULES_ON_PRIVATE_LINK_TRAFFIC,
            ),
            ("ip_address_type", VALID_IP_ADDRESS_TYPE),
            ("scheme", VALID_SCHEME),
            ("type", VALID_TYPE),
        ],
    )
}

/// Returns the IAM permissions declared by the CloudFormation handler for this operation.
pub fn required_permissions(op: carina_core::effect::PlanOp) -> &'static [&'static str] {
    match op {
        carina_core::effect::PlanOp::Create => &[
            "elasticloadbalancing:CreateLoadBalancer",
            "elasticloadbalancing:DescribeLoadBalancers",
            "elasticloadbalancing:ModifyLoadBalancerAttributes",
            "elasticloadbalancing:ModifyCapacityReservation",
            "elasticloadbalancing:AddTags",
            "elasticloadbalancing:SetSecurityGroups",
            "ec2:DescribeIpamPools",
        ],
        carina_core::effect::PlanOp::Read => &[
            "elasticloadbalancing:DescribeLoadBalancers",
            "elasticloadbalancing:DescribeLoadBalancerAttributes",
            "elasticloadbalancing:DescribeCapacityReservation",
            "elasticloadbalancing:DescribeTags",
        ],
        carina_core::effect::PlanOp::Update => &[
            "elasticloadbalancing:ModifyLoadBalancerAttributes",
            "elasticloadbalancing:ModifyCapacityReservation",
            "elasticloadbalancing:SetSubnets",
            "elasticloadbalancing:SetIpAddressType",
            "elasticloadbalancing:ModifyIpPools",
            "elasticloadbalancing:SetSecurityGroups",
            "elasticloadbalancing:AddTags",
            "elasticloadbalancing:RemoveTags",
        ],
        carina_core::effect::PlanOp::Delete => &[
            "elasticloadbalancing:DescribeLoadBalancers",
            "elasticloadbalancing:DeleteLoadBalancer",
        ],
    }
}
