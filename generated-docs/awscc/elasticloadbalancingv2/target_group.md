---
title: "awscc.elasticloadbalancingv2.TargetGroup"
---


CloudFormation Type: `AWS::ElasticLoadBalancingV2::TargetGroup`

Resource Type definition for AWS::ElasticLoadBalancingV2::TargetGroup

## Example

```crn
awscc.elasticloadbalancingv2.TargetGroup {
  name              = 'registry-tg'
  protocol          = aws.elasticloadbalancingv2.TargetGroup.Protocol.http
  port              = 8080
  vpc_id            = 'vpc-dddd4444'
  target_type       = aws.elasticloadbalancingv2.TargetGroup.TargetType.ip
  health_check_path = '/health'

  target {
    id   = '10.0.1.10'
    port = 8080
  }

  tags = {
    Environment = 'example'
    Workload    = 'registry'
  }
}
```

## Argument Reference

### `health_check_enabled`

- **Type:** Bool
- **Required:** No

Indicates whether health checks are enabled. If the target type is lambda, health checks are disabled by default but can be enabled. If the target type is instance, ip, or alb, health checks are always enabled and cannot be disabled.

### `health_check_interval_seconds`

- **Type:** Int(5..=300)
- **Required:** No

The approximate amount of time, in seconds, between health checks of an individual target.

### `health_check_path`

- **Type:** `String(len: 1..=1024)`
- **Required:** No

[HTTP/HTTPS health checks] The destination for health checks on the targets. [HTTP1 or HTTP2 protocol version] The ping path. The default is /. [GRPC protocol version] The path of a custom health check method with the format /package.service/method. The default is /AWS.ALB/healthcheck.

### `health_check_port`

- **Type:** `String(pattern)`
- **Required:** No

The port the load balancer uses when performing health checks on targets. 

### `health_check_protocol`

- **Type:** [Enum (HealthCheckProtocol)](#health_check_protocol-healthcheckprotocol)
- **Required:** No

The protocol the load balancer uses when performing health checks on targets. 

### `health_check_timeout_seconds`

- **Type:** Int(2..=120)
- **Required:** No

The amount of time, in seconds, during which no response from a target means a failed health check.

### `healthy_threshold_count`

- **Type:** Int(2..=10)
- **Required:** No

The number of consecutive health checks successes required before considering an unhealthy target healthy. 

### `ip_address_type`

- **Type:** [Enum (IpAddressType)](#ip_address_type-ipaddresstype)
- **Required:** No
- **Create-only:** Yes

The type of IP address used for this target group. The possible values are ipv4 and ipv6. 

### `matcher`

- **Type:** [Struct(Matcher)](#matcher-1)
- **Required:** No

[HTTP/HTTPS health checks] The HTTP or gRPC codes to use when checking for a successful response from a target.

### `name`

- **Type:** `String(pattern, len: 1..=32)`
- **Required:** No
- **Create-only:** Yes

The name of the target group.

### `port`

- **Type:** Int(1..=65535)
- **Required:** No
- **Create-only:** Yes

The port on which the targets receive traffic. This port is used unless you specify a port override when registering the target. If the target is a Lambda function, this parameter does not apply. If the protocol is GENEVE, the supported port is 6081.

### `protocol`

- **Type:** [Enum (Protocol)](#protocol-protocol)
- **Required:** No
- **Create-only:** Yes

The protocol to use for routing traffic to the targets.

### `protocol_version`

- **Type:** [Enum (ProtocolVersion)](#protocol_version-protocolversion)
- **Required:** No
- **Create-only:** Yes

[HTTP/HTTPS protocol] The protocol version. The possible values are GRPC, HTTP1, and HTTP2.

### `tags`

- **Type:** `Map<String, String>`
- **Required:** No

The tags.

### `target_control_port`

- **Type:** Int(1..=65535)
- **Required:** No

The port that the target control agent uses to communicate the available capacity of targets to the load balancer.

### `target_group_attributes`

- **Type:** List\<[Struct(TargetGroupAttribute)](#target_group_attributes-1)\> (unique items)
- **Required:** No

The attributes.

### `target_type`

- **Type:** [Enum (TargetType)](#target_type-targettype)
- **Required:** No
- **Create-only:** Yes

The type of target that you must specify when registering targets with this target group. You can't specify targets for a target group using more than one target type.

### `targets`

- **Type:** List\<[Struct(TargetDescription)](#targets-1)\> (unique items)
- **Required:** No

The targets.

### `unhealthy_threshold_count`

- **Type:** Int(2..=10)
- **Required:** No

The number of consecutive health check failures required before considering a target unhealthy.

### `vpc_id`

- **Type:** aws.ec2.Vpc.Id — identity: String(pattern); member: String
- **Required:** No
- **Create-only:** Yes

The identifier of the virtual private cloud (VPC). If the target is a Lambda function, this parameter does not apply.

## Enum Values

### health_check_protocol (HealthCheckProtocol)

| Value | DSL Identifier |
|-------|----------------|
| `HTTP` | `aws.elasticloadbalancingv2.TargetGroup.HealthCheckProtocol.http` |
| `HTTPS` | `aws.elasticloadbalancingv2.TargetGroup.HealthCheckProtocol.https` |
| `TCP` | `aws.elasticloadbalancingv2.TargetGroup.HealthCheckProtocol.tcp` |

Shorthand formats: `http` or `HealthCheckProtocol.http`

### ip_address_type (IpAddressType)

| Value | DSL Identifier |
|-------|----------------|
| `ipv4` | `aws.elasticloadbalancingv2.TargetGroup.IpAddressType.ipv4` |
| `ipv6` | `aws.elasticloadbalancingv2.TargetGroup.IpAddressType.ipv6` |

Shorthand formats: `ipv4` or `IpAddressType.ipv4`

### protocol (Protocol)

| Value | DSL Identifier |
|-------|----------------|
| `HTTP` | `aws.elasticloadbalancingv2.TargetGroup.Protocol.http` |
| `HTTPS` | `aws.elasticloadbalancingv2.TargetGroup.Protocol.https` |
| `TCP` | `aws.elasticloadbalancingv2.TargetGroup.Protocol.tcp` |
| `TLS` | `aws.elasticloadbalancingv2.TargetGroup.Protocol.tls` |
| `UDP` | `aws.elasticloadbalancingv2.TargetGroup.Protocol.udp` |
| `TCP_UDP` | `aws.elasticloadbalancingv2.TargetGroup.Protocol.tcp_udp` |
| `GENEVE` | `aws.elasticloadbalancingv2.TargetGroup.Protocol.geneve` |
| `QUIC` | `aws.elasticloadbalancingv2.TargetGroup.Protocol.quic` |
| `TCP_QUIC` | `aws.elasticloadbalancingv2.TargetGroup.Protocol.tcp_quic` |

Shorthand formats: `http` or `Protocol.http`

### protocol_version (ProtocolVersion)

| Value | DSL Identifier |
|-------|----------------|
| `GRPC` | `aws.elasticloadbalancingv2.TargetGroup.ProtocolVersion.grpc` |
| `HTTP1` | `aws.elasticloadbalancingv2.TargetGroup.ProtocolVersion.http1` |
| `HTTP2` | `aws.elasticloadbalancingv2.TargetGroup.ProtocolVersion.http2` |

Shorthand formats: `grpc` or `ProtocolVersion.grpc`

### target_type (TargetType)

| Value | DSL Identifier |
|-------|----------------|
| `instance` | `aws.elasticloadbalancingv2.TargetGroup.TargetType.instance` |
| `ip` | `aws.elasticloadbalancingv2.TargetGroup.TargetType.ip` |
| `lambda` | `aws.elasticloadbalancingv2.TargetGroup.TargetType.lambda` |
| `alb` | `aws.elasticloadbalancingv2.TargetGroup.TargetType.alb` |

Shorthand formats: `instance` or `TargetType.instance`

## Struct Definitions

### matcher

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `grpc_code` | aws.elasticloadbalancingv2.TargetGroup.GrpcCode — identity: String; member: String | No | You can specify values between 0 and 99. You can specify multiple values, or a range of values. The default value is 12. |
| `http_code` | aws.elasticloadbalancingv2.TargetGroup.HttpCode — identity: String; member: String | No | For Application Load Balancers, you can specify values between 200 and 499, and the default value is 200. You can specify multiple values or a range of values.  |

### target_group_attributes

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `key` | `String(pattern, len: ..=256)` | No | The value of the attribute. |
| `value` | String | No | The name of the attribute. |

### targets

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `availability_zone` | `aws.AvailabilityZone.ZoneName \| String (pattern: ^all\z)` | No | An Availability Zone or all. This determines whether the target receives traffic from the load balancer nodes in the specified Availability Zone or from all enabled Availability Zones for the load balancer. |
| `id` | String | Yes | The ID of the target. If the target type of the target group is instance, specify an instance ID. If the target type is ip, specify an IP address. If the target type is lambda, specify the ARN of the Lambda function. If the target type is alb, specify the ARN of the Application Load Balancer target.  |
| `port` | Int(1..=65535) | No | The port on which the target is listening. If the target group protocol is GENEVE, the supported port is 6081. If the target type is alb, the targeted Application Load Balancer must have at least one listener whose port matches the target group port. Not used if the target is a Lambda function. |
| `quic_server_id` | `String(pattern)` | No | The Server ID used by targets when using QUIC or TCP_QUIC protocols. |

## Attribute Reference

### `load_balancer_arns`

- **Type:** `List<aws.Arn — identity: String(pattern); member: String>` (items: ..=1)

The Amazon Resource Names (ARNs) of the load balancers that route traffic to this target group.

### `target_group_arn`

- **Type:** aws.Arn — identity: String(pattern); member: String

The ARN of the Target Group

### `target_group_full_name`

- **Type:** String

The full name of the target group.

### `target_group_name`

- **Type:** `String(pattern, len: 1..=32)`

The name of the target group.

## AWS API narrowings

These fields are narrower than or equal to CFN. Input rules combine documented AWS domains, recorded live observations, and Carina canonical spellings for stable state comparison. A canonical DSL spelling does not claim AWS rejects other spellings. Attaching a type name adds no structural constraints. Field names below use CFN spelling.

| CFN field | Relation to CFN | Evidence URL | Verbatim quote / exact observation | Checked / observed |
| --- | --- | --- | --- | --- |
| `/definitions/TargetDescription/properties/AvailabilityZone` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/elasticloadbalancing/latest/APIReference/API_TargetDescription.html) | ` An Availability Zone or all. This determines whether the target receives traffic from the load balancer nodes in the specified Availability Zone or from all enabled Availability Zones for the load balancer. ` | 2026-11-07 |
| `/definitions/TargetDescription/properties/Port` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/elasticloadbalancing/latest/APIReference/API_TargetDescription.html) | ` The port on which the target is listening. If the target group protocol is GENEVE, the supported port is 6081. If the target type is alb, the targeted Application Load Balancer must have at least one listener whose port matches the target group port. This parameter is not used if the target is a Lambda function. `<br>` Valid Range: Minimum value of 1. Maximum value of 65535. ` | 2026-11-07 |
| `/definitions/TargetDescription/properties/QuicServerId` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/elasticloadbalancing/latest/APIReference/API_TargetDescription.html) | ` The server ID for the targets. This value is required if the protocol is QUIC or TCP_QUIC and can't be used with other protocols. `<br>` The ID consists of the 0x prefix followed by 16 hexadecimal characters. Any letters must be lowercase. The value must be unique at the listener level. You can't modify the server ID for a registered target. You must deregister the target and then provide a new server ID when you register the target again. `<br>` Length Constraints: Minimum length of 1. Maximum length of 256. ` | 2026-11-07 |
| `/definitions/TargetGroupAttribute/properties/Key` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/elasticloadbalancing/latest/APIReference/API_TargetGroupAttribute.html) | ` The name of the attribute. `<br>` Length Constraints: Maximum length of 256. `<br>` Pattern: ^[a-zA-Z0-9._]+$ ` | 2026-11-07 |
| `/properties/HealthCheckIntervalSeconds` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/elasticloadbalancing/latest/APIReference/API_ModifyTargetGroup.html) | ` The approximate amount of time, in seconds, between health checks of an individual target. `<br>` Valid Range: Minimum value of 5. Maximum value of 300. ` | 2026-11-07 |
| `/properties/HealthCheckPath` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/elasticloadbalancing/latest/APIReference/API_ModifyTargetGroup.html) | ` [HTTP/HTTPS health checks] The destination for health checks on the targets. `<br>` [HTTP1 or HTTP2 protocol version] The ping path. The default is /. `<br>` [GRPC protocol version] The path of a custom health check method with the format /package.service/method. The default is /AWS.ALB/healthcheck. `<br>` Length Constraints: Minimum length of 1. Maximum length of 1024. ` | 2026-11-07 |
| `/properties/HealthCheckPort` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/elasticloadbalancing/latest/gateway/health-checks.html) | ` HealthCheckPort The port that Gateway Load Balancer uses when performing health checks on targets. The range is 1 to 65535. The default is 80. ` | 2026-11-07 |
| `/properties/HealthCheckProtocol` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/elasticloadbalancing/latest/APIReference/API_CreateTargetGroup.html) | ` The protocol the load balancer uses when performing health checks on targets. For Application Load Balancers, the default is HTTP. For Network Load Balancers and Gateway Load Balancers, the default is TCP. The TCP protocol is not supported for health checks if the protocol of the target group is HTTP or HTTPS. The GENEVE, TLS, UDP, TCP_UDP, QUIC, and TCP_QUIC protocols are not supported for health checks. `<br>` Valid Values: HTTP \| HTTPS \| TCP \| TLS \| UDP \| TCP_UDP \| GENEVE \| QUIC \| TCP_QUIC ` | 2026-11-07 |
| `/properties/HealthCheckTimeoutSeconds` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/elasticloadbalancing/latest/APIReference/API_ModifyTargetGroup.html) | ` [HTTP/HTTPS health checks] The amount of time, in seconds, during which no response means a failed health check. `<br>` Valid Range: Minimum value of 2. Maximum value of 120. ` | 2026-11-07 |
| `/properties/HealthyThresholdCount` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/elasticloadbalancing/latest/APIReference/API_ModifyTargetGroup.html) | ` The number of consecutive health checks successes required before considering an unhealthy target healthy. `<br>` Valid Range: Minimum value of 2. Maximum value of 10. ` | 2026-11-07 |
| `/properties/LoadBalancerArns` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/elasticloadbalancing/latest/APIReference/API_TargetGroup.html) | ` The Amazon Resource Name (ARN) of the load balancer that routes traffic to this target group. You can use each target group with only one load balancer. ` | 2026-11-07 |
| `/properties/Name` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/elasticloadbalancing/latest/APIReference/API_CreateTargetGroup.html) | ` The name of the target group. `<br>` This name must be unique per region per account, can have a maximum of 32 characters, must contain only alphanumeric characters or hyphens, and must not begin or end with a hyphen. ` | 2026-11-07 |
| `/properties/Port` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/elasticloadbalancing/latest/APIReference/API_CreateTargetGroup.html) | ` The port on which the targets receive traffic. This port is used unless you specify a port override when registering the target. If the target is a Lambda function, this parameter does not apply. If the protocol is GENEVE, the supported port is 6081. `<br>` Valid Range: Minimum value of 1. Maximum value of 65535. ` | 2026-11-07 |
| `/properties/Protocol` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/elasticloadbalancing/latest/APIReference/API_CreateTargetGroup.html) | ` The protocol to use for routing traffic to the targets. For Application Load Balancers, the supported protocols are HTTP and HTTPS. For Network Load Balancers, the supported protocols are TCP, TLS, UDP, TCP_UDP, QUIC, or TCP_QUIC. For Gateway Load Balancers, the supported protocol is GENEVE. A TCP_UDP listener must be associated with a TCP_UDP target group. A TCP_QUIC listener must be associated with a TCP_QUIC target group. If the target is a Lambda function, this parameter does not apply. `<br>` Valid Values: HTTP \| HTTPS \| TCP \| TLS \| UDP \| TCP_UDP \| GENEVE \| QUIC \| TCP_QUIC ` | 2026-11-07 |
| `/properties/TargetControlPort` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/elasticloadbalancing/latest/APIReference/API_CreateTargetGroup.html) | ` The port on which the target control agent and application load balancer exchange management traffic for the target optimizer feature. `<br>` Valid Range: Minimum value of 1. Maximum value of 65535. ` | 2026-11-07 |
| `/properties/TargetGroupName` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/elasticloadbalancing/latest/APIReference/API_CreateTargetGroup.html) | ` The name of the target group. `<br>` This name must be unique per region per account, can have a maximum of 32 characters, must contain only alphanumeric characters or hyphens, and must not begin or end with a hyphen. ` | 2026-11-07 |
| `/properties/TargetType` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/elasticloadbalancing/latest/APIReference/API_CreateTargetGroup.html) | ` The type of target that you must specify when registering targets with this target group. You can't specify targets for a target group using more than one target type. `<br>` instance - Register targets by instance ID. This is the default value. `<br>` ip - Register targets by IP address. You can specify IP addresses from the subnets of the virtual private cloud (VPC) for the target group, the RFC 1918 range (10.0.0.0/8, 172.16.0.0/12, and 192.168.0.0/16), and the RFC 6598 range (100.64.0.0/10). You can't specify publicly routable IP addresses. `<br>` lambda - Register a single Lambda function as a target. `<br>` alb - Register a single Application Load Balancer as a target. `<br>` Valid Values: instance \| ip \| lambda \| alb ` | 2026-11-07 |
| `/properties/UnhealthyThresholdCount` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/elasticloadbalancing/latest/APIReference/API_ModifyTargetGroup.html) | ` The number of consecutive health check failures required before considering the target unhealthy. `<br>` Valid Range: Minimum value of 2. Maximum value of 10. ` | 2026-11-07 |

