---
title: "awscc.elasticloadbalancingv2.TargetGroup"
description: "AWSCC Elastic Load Balancing v2 TargetGroup resource reference"
---


CloudFormation Type: `AWS::ElasticLoadBalancingV2::TargetGroup`

Resource Type definition for AWS::ElasticLoadBalancingV2::TargetGroup

## Example

```crn
awscc.elasticloadbalancingv2.TargetGroup {
  name              = 'registry-tg'
  protocol          = 'HTTP'
  port              = 8080
  vpc_id            = 'vpc-dddd4444'
  target_type       = 'ip'
  health_check_path = '/health'

  target {
    id   = '10.0.1.10'
    port = 8080
  }

  tag {
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

- **Type:** Int
- **Required:** No

The approximate amount of time, in seconds, between health checks of an individual target.

### `health_check_path`

- **Type:** String
- **Required:** No

[HTTP/HTTPS health checks] The destination for health checks on the targets. [HTTP1 or HTTP2 protocol version] The ping path. The default is /. [GRPC protocol version] The path of a custom health check method with the format /package.service/method. The default is /AWS.ALB/healthcheck.

### `health_check_port`

- **Type:** String
- **Required:** No

The port the load balancer uses when performing health checks on targets. 

### `health_check_protocol`

- **Type:** String
- **Required:** No

The protocol the load balancer uses when performing health checks on targets. 

### `health_check_timeout_seconds`

- **Type:** Int
- **Required:** No

The amount of time, in seconds, during which no response from a target means a failed health check.

### `healthy_threshold_count`

- **Type:** Int
- **Required:** No

The number of consecutive health checks successes required before considering an unhealthy target healthy. 

### `ip_address_type`

- **Type:** String
- **Required:** No
- **Create-only:** Yes

The type of IP address used for this target group. The possible values are ipv4 and ipv6. 

### `matcher`

- **Type:** [Struct(Matcher)](#matcher)
- **Required:** No

[HTTP/HTTPS health checks] The HTTP or gRPC codes to use when checking for a successful response from a target.

### `name`

- **Type:** String
- **Required:** No
- **Create-only:** Yes

The name of the target group.

### `port`

- **Type:** Int
- **Required:** No
- **Create-only:** Yes

The port on which the targets receive traffic. This port is used unless you specify a port override when registering the target. If the target is a Lambda function, this parameter does not apply. If the protocol is GENEVE, the supported port is 6081.

### `protocol`

- **Type:** String
- **Required:** No
- **Create-only:** Yes

The protocol to use for routing traffic to the targets.

### `protocol_version`

- **Type:** String
- **Required:** No
- **Create-only:** Yes

[HTTP/HTTPS protocol] The protocol version. The possible values are GRPC, HTTP1, and HTTP2.

### `tags`

- **Type:** `Map<String, String>`
- **Required:** No

The tags.

### `target_control_port`

- **Type:** Int
- **Required:** No

The port that the target control agent uses to communicate the available capacity of targets to the load balancer.

### `target_group_attributes`

- **Type:** [List\<TargetGroupAttribute\>](#targetgroupattribute)
- **Required:** No

The attributes.

### `target_type`

- **Type:** String
- **Required:** No
- **Create-only:** Yes

The type of target that you must specify when registering targets with this target group. You can't specify targets for a target group using more than one target type.

### `targets`

- **Type:** [List\<TargetDescription\>](#targetdescription)
- **Required:** No

The targets.

### `unhealthy_threshold_count`

- **Type:** Int
- **Required:** No

The number of consecutive health check failures required before considering a target unhealthy.

### `vpc_id`

- **Type:** VpcId
- **Required:** No
- **Create-only:** Yes

The identifier of the virtual private cloud (VPC). If the target is a Lambda function, this parameter does not apply.

## Struct Definitions

### Matcher

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `grpc_code` | String | No | You can specify values between 0 and 99. You can specify multiple values, or a range of values. The default value is 12. |
| `http_code` | String | No | For Application Load Balancers, you can specify values between 200 and 499, and the default value is 200. You can specify multiple values or a range of values.  |

### TargetDescription

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `availability_zone` | AvailabilityZone | No | An Availability Zone or all. This determines whether the target receives traffic from the load balancer nodes in the specified Availability Zone or from all enabled Availability Zones for the load balancer. |
| `id` | String | Yes | The ID of the target. If the target type of the target group is instance, specify an instance ID. If the target type is ip, specify an IP address. If the target type is lambda, specify the ARN of the Lambda function. If the target type is alb, specify the ARN of the Application Load Balancer target.  |
| `port` | Int | No | The port on which the target is listening. If the target group protocol is GENEVE, the supported port is 6081. If the target type is alb, the targeted Application Load Balancer must have at least one listener whose port matches the target group port. Not used if the target is a Lambda function. |
| `quic_server_id` | String | No | The Server ID used by targets when using QUIC or TCP_QUIC protocols. |

### TargetGroupAttribute

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `key` | String | No | The value of the attribute. |
| `value` | String | No | The name of the attribute. |

## Attribute Reference

### `load_balancer_arns`

- **Type:** `List<Arn>`

The Amazon Resource Names (ARNs) of the load balancers that route traffic to this target group.

### `target_group_arn`

- **Type:** Arn

The ARN of the Target Group

### `target_group_full_name`

- **Type:** String

The full name of the target group.

### `target_group_name`

- **Type:** String

The name of the target group.

