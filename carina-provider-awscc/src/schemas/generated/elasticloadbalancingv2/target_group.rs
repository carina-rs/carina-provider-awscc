//! target_group schema definition for AWS Cloud Control
//!
//! Auto-generated from CloudFormation schema: AWS::ElasticLoadBalancingV2::TargetGroup
//!
//! DO NOT EDIT MANUALLY - regenerate with carina-codegen

use crate::schemas::config::{AwsccSchemaConfig, TagsProperty, TagsWire};
use carina_core::schema::{AttributeSchema, AttributeType, ResourceSchema, StructField};

const VALID_IP_ADDRESS_TYPE: &[&str] = &["ipv4", "ipv6"];

const VALID_PROTOCOL: &[&str] = &["HTTP", "HTTPS", "TCP", "TLS", "UDP", "TCP_UDP", "GENEVE"];

const VALID_PROTOCOL_VERSION: &[&str] = &["GRPC", "HTTP1", "HTTP2"];

const VALID_TARGET_TYPE: &[&str] = &["instance", "ip", "lambda", "alb"];

/// Returns the schema config for elasticloadbalancingv2_target_group (AWS::ElasticLoadBalancingV2::TargetGroup)
pub fn elasticloadbalancingv2_target_group_config() -> AwsccSchemaConfig {
    AwsccSchemaConfig {
        aws_type_name: "AWS::ElasticLoadBalancingV2::TargetGroup",
        resource_type_name: "elasticloadbalancingv2.TargetGroup",
        primary_identifier: &[crate::schemas::config::PrimaryIdentifierAttribute { provider_name: "TargetGroupArn", dsl_name: "target_group_arn" }],
        tags: Some(TagsProperty { provider_name: "Tags", wire: TagsWire::KeyValueList }),
        schema: ResourceSchema::new("elasticloadbalancingv2.TargetGroup")
	        .with_description("Resource Type definition for AWS::ElasticLoadBalancingV2::TargetGroup")
        .attribute(
            AttributeSchema::optional("health_check_enabled", AttributeType::bool())
                .with_description("Indicates whether health checks are enabled. If the target type is lambda, health checks are disabled by default but can be enabled. If the target type is instance, ip, or alb, health checks are always enabled and cannot be disabled.")
                .with_provider_name("HealthCheckEnabled")
                .build(),
        )
        .attribute(
            AttributeSchema::optional("health_check_interval_seconds", AttributeType::int())
                .with_description("The approximate amount of time, in seconds, between health checks of an individual target.")
                .with_provider_name("HealthCheckIntervalSeconds")
                .build(),
        )
        .attribute(
            AttributeSchema::optional("health_check_path", AttributeType::string())
                .with_description("[HTTP/HTTPS health checks] The destination for health checks on the targets. [HTTP1 or HTTP2 protocol version] The ping path. The default is /. [GRPC protocol version] The path of a custom health check method with the format /package.service/method. The default is /AWS.ALB/healthcheck.")
                .with_provider_name("HealthCheckPath")
                .build(),
        )
        .attribute(
            AttributeSchema::optional("health_check_port", AttributeType::string())
                .with_description("The port the load balancer uses when performing health checks on targets. ")
                .with_provider_name("HealthCheckPort")
                .build(),
        )
        .attribute(
            AttributeSchema::optional("health_check_protocol", AttributeType::string())
                .with_description("The protocol the load balancer uses when performing health checks on targets. ")
                .with_provider_name("HealthCheckProtocol")
                .build(),
        )
        .attribute(
            AttributeSchema::optional("health_check_timeout_seconds", AttributeType::int())
                .with_description("The amount of time, in seconds, during which no response from a target means a failed health check.")
                .with_provider_name("HealthCheckTimeoutSeconds")
                .build(),
        )
        .attribute(
            AttributeSchema::optional("healthy_threshold_count", AttributeType::int())
                .with_description("The number of consecutive health checks successes required before considering an unhealthy target healthy. ")
                .with_provider_name("HealthyThresholdCount")
                .build(),
        )
        .attribute(
            AttributeSchema::optional("ip_address_type", AttributeType::enum_(carina_core::schema::enum_identity("IpAddressType", Some("aws.elasticloadbalancingv2.TargetGroup")), Some(vec!["ipv4".to_string(), "ipv6".to_string()]), vec![("ipv4".to_string(), "ipv4".to_string()), ("ipv6".to_string(), "ipv6".to_string())], None, None))
                .create_only()
                .with_description("The type of IP address used for this target group. The possible values are ipv4 and ipv6. ")
                .with_provider_name("IpAddressType")
                .build(),
        )
        .attribute(
            AttributeSchema::provider_populated("load_balancer_arns", AttributeType::unordered_list(carina_aws_types::arn()))
                .with_description("The Amazon Resource Names (ARNs) of the load balancers that route traffic to this target group. (read-only)")
                .with_provider_name("LoadBalancerArns")
                .build(),
        )
        .attribute(
            AttributeSchema::optional("matcher", AttributeType::struct_("Matcher".to_string(), vec![StructField::optional("grpc_code", AttributeType::string()).with_description("You can specify values between 0 and 99. You can specify multiple values, or a range of values. The default value is 12.").with_provider_name("GrpcCode").build(),
                    StructField::optional("http_code", AttributeType::string()).with_description("For Application Load Balancers, you can specify values between 200 and 499, and the default value is 200. You can specify multiple values or a range of values. ").with_provider_name("HttpCode").build()]))
                .with_description("[HTTP/HTTPS health checks] The HTTP or gRPC codes to use when checking for a successful response from a target.")
                .with_provider_name("Matcher")
                .build(),
        )
        .attribute(
            AttributeSchema::optional("name", AttributeType::string())
                .create_only()
                .with_description("The name of the target group.")
                .with_provider_name("Name")
                .build(),
        )
        .attribute(
            AttributeSchema::optional("port", AttributeType::int())
                .create_only()
                .with_description("The port on which the targets receive traffic. This port is used unless you specify a port override when registering the target. If the target is a Lambda function, this parameter does not apply. If the protocol is GENEVE, the supported port is 6081.")
                .with_provider_name("Port")
                .build(),
        )
        .attribute(
            AttributeSchema::optional("protocol", AttributeType::enum_(carina_core::schema::enum_identity("Protocol", Some("aws.elasticloadbalancingv2.TargetGroup")), Some(vec!["HTTP".to_string(), "HTTPS".to_string(), "TCP".to_string(), "TLS".to_string(), "UDP".to_string(), "TCP_UDP".to_string(), "GENEVE".to_string()]), vec![("HTTP".to_string(), "http".to_string()), ("HTTPS".to_string(), "https".to_string()), ("TCP".to_string(), "tcp".to_string()), ("TLS".to_string(), "tls".to_string()), ("UDP".to_string(), "udp".to_string()), ("TCP_UDP".to_string(), "tcp_udp".to_string()), ("GENEVE".to_string(), "geneve".to_string())], None, None))
                .create_only()
                .with_description("The protocol to use for routing traffic to the targets.")
                .with_provider_name("Protocol")
                .build(),
        )
        .attribute(
            AttributeSchema::optional("protocol_version", AttributeType::enum_(carina_core::schema::enum_identity("ProtocolVersion", Some("aws.elasticloadbalancingv2.TargetGroup")), Some(vec!["GRPC".to_string(), "HTTP1".to_string(), "HTTP2".to_string()]), vec![("GRPC".to_string(), "grpc".to_string()), ("HTTP1".to_string(), "http1".to_string()), ("HTTP2".to_string(), "http2".to_string())], None, None))
                .create_only()
                .with_description("[HTTP/HTTPS protocol] The protocol version. The possible values are GRPC, HTTP1, and HTTP2.")
                .with_provider_name("ProtocolVersion")
                .build(),
        )
        .attribute(
            AttributeSchema::optional("tags", carina_aws_types::tags_type())
                .with_description("The tags.")
                .with_provider_name("Tags")
                .with_block_name("tag")
                .build(),
        )
        .attribute(
            AttributeSchema::optional("target_control_port", AttributeType::int())
                .with_description("The port that the target control agent uses to communicate the available capacity of targets to the load balancer.")
                .with_provider_name("TargetControlPort")
                .build(),
        )
        .attribute(
            AttributeSchema::provider_populated("target_group_arn", carina_aws_types::arn())
                .with_description("The ARN of the Target Group (read-only)")
                .with_provider_name("TargetGroupArn")
                .build(),
        )
        .attribute(
            AttributeSchema::optional("target_group_attributes", AttributeType::unordered_list(AttributeType::struct_("TargetGroupAttribute".to_string(), vec![StructField::optional("key", AttributeType::string()).with_description("The value of the attribute.").with_provider_name("Key").build(),
                    StructField::optional("value", AttributeType::string()).with_description("The name of the attribute.").with_provider_name("Value").build()])))
                .with_description("The attributes.")
                .with_provider_name("TargetGroupAttributes")
                .with_block_name("target_group_attribute")
                .build(),
        )
        .attribute(
            AttributeSchema::provider_populated("target_group_full_name", AttributeType::string())
                .with_description("The full name of the target group. (read-only)")
                .with_provider_name("TargetGroupFullName")
                .build(),
        )
        .attribute(
            AttributeSchema::provider_populated("target_group_name", AttributeType::string())
                .with_description("The name of the target group. (read-only)")
                .with_provider_name("TargetGroupName")
                .build(),
        )
        .attribute(
            AttributeSchema::optional("target_type", AttributeType::enum_(carina_core::schema::enum_identity("TargetType", Some("aws.elasticloadbalancingv2.TargetGroup")), Some(vec!["instance".to_string(), "ip".to_string(), "lambda".to_string(), "alb".to_string()]), vec![("instance".to_string(), "instance".to_string()), ("ip".to_string(), "ip".to_string()), ("lambda".to_string(), "lambda".to_string()), ("alb".to_string(), "alb".to_string())], None, None))
                .create_only()
                .with_description("The type of target that you must specify when registering targets with this target group. You can't specify targets for a target group using more than one target type.")
                .with_provider_name("TargetType")
                .build(),
        )
        .attribute(
            AttributeSchema::optional("targets", AttributeType::unordered_list(AttributeType::struct_("TargetDescription".to_string(), vec![StructField::optional("availability_zone", carina_aws_types::availability_zone()).with_description("An Availability Zone or all. This determines whether the target receives traffic from the load balancer nodes in the specified Availability Zone or from all enabled Availability Zones for the load balancer.").with_provider_name("AvailabilityZone").build(),
                    StructField::required("id", AttributeType::string()).with_description("The ID of the target. If the target type of the target group is instance, specify an instance ID. If the target type is ip, specify an IP address. If the target type is lambda, specify the ARN of the Lambda function. If the target type is alb, specify the ARN of the Application Load Balancer target. ").with_provider_name("Id").build(),
                    StructField::optional("port", AttributeType::int()).with_description("The port on which the target is listening. If the target group protocol is GENEVE, the supported port is 6081. If the target type is alb, the targeted Application Load Balancer must have at least one listener whose port matches the target group port. Not used if the target is a Lambda function.").with_provider_name("Port").build(),
                    StructField::optional("quic_server_id", AttributeType::string()).with_description("The Server ID used by targets when using QUIC or TCP_QUIC protocols.").with_provider_name("QuicServerId").build()])))
                .with_description("The targets.")
                .with_provider_name("Targets")
                .with_block_name("target")
                .build(),
        )
        .attribute(
            AttributeSchema::optional("unhealthy_threshold_count", AttributeType::int())
                .with_description("The number of consecutive health check failures required before considering a target unhealthy.")
                .with_provider_name("UnhealthyThresholdCount")
                .build(),
        )
        .attribute(
            AttributeSchema::optional("vpc_id", carina_aws_types::vpc_id())
                .create_only()
                .with_description("The identifier of the virtual private cloud (VPC). If the target is a Lambda function, this parameter does not apply.")
                .with_provider_name("VpcId")
                .build(),
        )
        .with_unique_name_attribute("name")
        .with_declared_validators(vec![carina_provider_protocol::types::ValidatorType::TagsKeyValueCheck])
        .with_def("Matcher", AttributeType::struct_("Matcher".to_string(), vec![StructField::optional("grpc_code", AttributeType::string()).with_description("You can specify values between 0 and 99. You can specify multiple values, or a range of values. The default value is 12.").with_provider_name("GrpcCode").build(),
                    StructField::optional("http_code", AttributeType::string()).with_description("For Application Load Balancers, you can specify values between 200 and 499, and the default value is 200. You can specify multiple values or a range of values. ").with_provider_name("HttpCode").build()]))
    }
}

/// Returns the resource type name and all enum valid values for this module
pub fn enum_valid_values() -> (
    &'static str,
    &'static [(&'static str, &'static [&'static str])],
) {
    (
        "elasticloadbalancingv2.TargetGroup",
        &[
            ("ip_address_type", VALID_IP_ADDRESS_TYPE),
            ("protocol", VALID_PROTOCOL),
            ("protocol_version", VALID_PROTOCOL_VERSION),
            ("target_type", VALID_TARGET_TYPE),
        ],
    )
}

/// Returns the IAM permissions declared by the CloudFormation handler for this operation.
pub fn required_permissions(op: carina_core::effect::PlanOp) -> &'static [&'static str] {
    match op {
        carina_core::effect::PlanOp::Create => &[
            "elasticloadbalancing:CreateTargetGroup",
            "elasticloadbalancing:DescribeTargetGroups",
            "elasticloadbalancing:RegisterTargets",
            "elasticloadbalancing:ModifyTargetGroupAttributes",
            "elasticloadbalancing:DescribeTargetHealth",
            "elasticloadbalancing:AddTags",
        ],
        carina_core::effect::PlanOp::Read => &[
            "elasticloadbalancing:DescribeTargetGroups",
            "elasticloadbalancing:DescribeTargetGroupAttributes",
            "elasticloadbalancing:DescribeTargetHealth",
            "elasticloadbalancing:DescribeTags",
        ],
        carina_core::effect::PlanOp::Update => &[
            "elasticloadbalancing:DescribeTargetGroups",
            "elasticloadbalancing:ModifyTargetGroup",
            "elasticloadbalancing:ModifyTargetGroupAttributes",
            "elasticloadbalancing:RegisterTargets",
            "elasticloadbalancing:DescribeTargetHealth",
            "elasticloadbalancing:DeregisterTargets",
            "elasticloadbalancing:AddTags",
            "elasticloadbalancing:RemoveTags",
        ],
        carina_core::effect::PlanOp::Delete => &[
            "elasticloadbalancing:DeleteTargetGroup",
            "elasticloadbalancing:DescribeTargetGroups",
        ],
    }
}
