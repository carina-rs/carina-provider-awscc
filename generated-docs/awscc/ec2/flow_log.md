---
title: "awscc.ec2.FlowLog"
---


CloudFormation Type: `AWS::EC2::FlowLog`

Specifies a VPC flow log, which enables you to capture IP traffic for a specific network interface, subnet, or VPC.

## Example

```crn
let vpc = awscc.ec2.Vpc {
  cidr_block = '10.0.0.0/16'
}

awscc.ec2.FlowLog {
  resource_id          = vpc.vpc_id
  resource_type        = aws.ec2.FlowLog.ResourceType.vpc
  traffic_type         = all
  log_destination_type = s3
  log_destination      = 'arn:aws:s3:::example-flow-logs-bucket'

  tags = {
    Environment = 'example'
  }
}
```

## Argument Reference

### `deliver_cross_account_role`

- **Type:** aws.iam.Role.Arn — identity: String(pattern); member: String
- **Required:** No
- **Create-only:** Yes

The ARN of the IAM role that allows Amazon EC2 to publish flow logs across accounts.

### `deliver_logs_permission_arn`

- **Type:** aws.iam.Role.Arn — identity: String(pattern); member: String
- **Required:** No
- **Create-only:** Yes

The ARN for the IAM role that permits Amazon EC2 to publish flow logs to a CloudWatch Logs log group in your account. If you specify LogDestinationType as s3 or kinesis-data-firehose, do not specify DeliverLogsPermissionArn or LogGroupName.

### `destination_options`

- **Type:** [Struct(DestinationOptions)](#destination_options-1)
- **Required:** No
- **Create-only:** Yes

### `log_destination`

- **Type:** aws.Arn — identity: String(pattern); member: String
- **Required:** No
- **Create-only:** Yes

Specifies the destination to which the flow log data is to be published. Flow log data can be published to a CloudWatch Logs log group, an Amazon S3 bucket, or a Kinesis Firehose stream. The value specified for this parameter depends on the value specified for LogDestinationType.

### `log_destination_type`

- **Type:** [Enum (LogDestinationType)](#log_destination_type-logdestinationtype)
- **Required:** No
- **Create-only:** Yes

Specifies the type of destination to which the flow log data is to be published. Flow log data can be published to CloudWatch Logs or Amazon S3.

### `log_format`

- **Type:** String
- **Required:** No
- **Create-only:** Yes

The fields to include in the flow log record, in the order in which they should appear.

### `log_group_name`

- **Type:** String
- **Required:** No
- **Create-only:** Yes

The name of a new or existing CloudWatch Logs log group where Amazon EC2 publishes your flow logs. If you specify LogDestinationType as s3 or kinesis-data-firehose, do not specify DeliverLogsPermissionArn or LogGroupName.

### `max_aggregation_interval`

- **Type:** `aws.ec2.FlowLog.MaxAggregationInterval`
- **Required:** No
- **Create-only:** Yes

The maximum interval of time during which a flow of packets is captured and aggregated into a flow log record. You can specify 60 seconds (1 minute) or 600 seconds (10 minutes).

### `resource_id`

- **Type:** String
- **Required:** Yes
- **Create-only:** Yes

The ID of the subnet, network interface, or VPC for which you want to create a flow log.

### `resource_type`

- **Type:** [Enum (ResourceType)](#resource_type-resourcetype)
- **Required:** Yes
- **Create-only:** Yes

The type of resource for which to create the flow log. For example, if you specified a VPC ID for the ResourceId property, specify VPC for this property.

### `tags`

- **Type:** `Map<String(len: ..=128), String(len: ..=256)>`
- **Required:** No

The tags to apply to the flow logs.

### `traffic_type`

- **Type:** [Enum (TrafficType)](#traffic_type-traffictype)
- **Required:** No
- **Create-only:** Yes

The type of traffic to log. You can log traffic that the resource accepts or rejects, or all traffic.

## Enum Values

### destination_options.file_format (FileFormat)

| Value | DSL Identifier |
|-------|----------------|
| `plain-text` | `aws.ec2.FlowLog.DestinationOptions.FileFormat.plain_text` |
| `parquet` | `aws.ec2.FlowLog.DestinationOptions.FileFormat.parquet` |

Shorthand formats: `plain_text` or `FileFormat.plain_text`

### log_destination_type (LogDestinationType)

| Value | DSL Identifier |
|-------|----------------|
| `cloud-watch-logs` | `aws.ec2.FlowLog.LogDestinationType.cloud_watch_logs` |
| `s3` | `aws.ec2.FlowLog.LogDestinationType.s3` |
| `kinesis-data-firehose` | `aws.ec2.FlowLog.LogDestinationType.kinesis_data_firehose` |

Shorthand formats: `cloud_watch_logs` or `LogDestinationType.cloud_watch_logs`

### resource_type (ResourceType)

| Value | DSL Identifier |
|-------|----------------|
| `NetworkInterface` | `aws.ec2.FlowLog.ResourceType.network_interface` |
| `Subnet` | `aws.ec2.FlowLog.ResourceType.subnet` |
| `VPC` | `aws.ec2.FlowLog.ResourceType.vpc` |
| `TransitGateway` | `aws.ec2.FlowLog.ResourceType.transit_gateway` |
| `TransitGatewayAttachment` | `aws.ec2.FlowLog.ResourceType.transit_gateway_attachment` |
| `RegionalNatGateway` | `aws.ec2.FlowLog.ResourceType.regional_nat_gateway` |

Shorthand formats: `network_interface` or `ResourceType.network_interface`

### traffic_type (TrafficType)

| Value | DSL Identifier |
|-------|----------------|
| `ACCEPT` | `aws.ec2.FlowLog.TrafficType.accept` |
| `ALL` | `aws.ec2.FlowLog.TrafficType.all` |
| `REJECT` | `aws.ec2.FlowLog.TrafficType.reject` |

Shorthand formats: `accept` or `TrafficType.accept`

## Struct Definitions

### destination_options

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `file_format` | [Enum (FileFormat)](#destination_optionsfile_format-fileformat) | Yes |  |
| `hive_compatible_partitions` | Bool | Yes |  |
| `per_hour_partition` | Bool | Yes |  |

## Attribute Reference

### `id`

- **Type:** aws.ec2.FlowLog.Id — identity: String(pattern); member: String

The Flow Log ID

## AWS API narrowings

These fields are narrower than or equal to CFN. Input rules combine documented AWS domains, recorded live observations, and Carina canonical spellings for stable state comparison. A canonical DSL spelling does not claim AWS rejects other spellings. Attaching a type name adds no structural constraints. Field names below use CFN spelling.

| CFN field | Relation to CFN | Evidence URL | Verbatim quote / exact observation | Checked / observed |
| --- | --- | --- | --- | --- |
| `/definitions/Tag/properties/Key` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/AWSEC2/latest/UserGuide/Using_Tags.html) | ` Maximum key length – 128 Unicode characters in UTF-8 ` | 2026-11-07 |
| `/definitions/Tag/properties/Value` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/AWSEC2/latest/APIReference/API_Tag.html) | ` The value of the tag. `<br>` Constraints: Tag values are case-sensitive and accept a maximum of 256 Unicode characters. ` | 2026-11-07 |
| `/properties/MaxAggregationInterval` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/AWSCloudFormation/latest/TemplateReference/aws-resource-ec2-flowlog.html#cfn-ec2-flowlog-maxaggregationinterval) | ` The maximum interval of time during which a flow of packets is captured and aggregated into a flow log record. The possible values are 60 seconds (1 minute) or 600 seconds (10 minutes). This parameter must be 60 seconds for transit gateway resource types. `<br>` When a network interface is attached to a Nitro-based instance, the aggregation interval is always 60 seconds or less, regardless of the value that you specify. `<br>` Default: 600 ` | 2026-11-07 |

