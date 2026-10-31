---
title: "awscc.sns.Topic"
---


CloudFormation Type: `AWS::SNS::Topic`

The ``AWS::SNS::Topic`` resource creates a topic to which notifications can be published.
  One account can create a maximum of 100,000 standard topics and 1,000 FIFO topics. For more information, see [endpoints and quotas](https://docs.aws.amazon.com/general/latest/gr/sns.html) in the *General Reference*.
   The structure of ``AUTHPARAMS`` depends on the .signature of the API request. For more information, see [Examples of the complete Signature Version 4 signing process](https://docs.aws.amazon.com/general/latest/gr/sigv4-signed-request-examples.html) in the *General Reference*.

## Argument Reference

### `content_based_deduplication`

- **Type:** Bool
- **Required:** No

``ContentBasedDeduplication`` enables deduplication of messages based on their content for FIFO topics. By default, this property is set to false. If you create a FIFO topic with ``ContentBasedDeduplication`` set to false, you must provide a ``MessageDeduplicationId`` for each ``Publish`` action. When set to true, SNS automatically generates a ``MessageDeduplicationId`` using a SHA-256 hash of the message body (excluding message attributes). You can optionally override this generated value by specifying a ``MessageDeduplicationId`` in the ``Publish`` action. Note that this property only applies to FIFO topics; using it with standard topics will cause the creation to fail.

### `delivery_status_logging`

- **Type:** List\<[Struct(LoggingConfig)](#delivery_status_logging-1)\>
- **Required:** No

The ``DeliveryStatusLogging`` configuration enables you to log the delivery status of messages sent from your Amazon SNS topic to subscribed endpoints with the following supported delivery protocols: + HTTP + Amazon Kinesis Data Firehose + AWS Lambda + Platform application endpoint + Amazon Simple Queue Service Once configured, log entries are sent to Amazon CloudWatch Logs.

### `display_name`

- **Type:** String
- **Required:** No

The display name to use for an SNS topic with SMS subscriptions. The display name must be maximum 100 characters long, including hyphens (-), underscores (_), spaces, and tabs.

### `fifo_throughput_scope`

- **Type:** String
- **Required:** No

Specifies the throughput quota and deduplication behavior to apply for the FIFO topic. Valid values are ``Topic`` or ``MessageGroup``.

### `fifo_topic`

- **Type:** Bool
- **Required:** No
- **Create-only:** Yes

Set to true to create a FIFO topic.

### `kms_master_key_id`

- **Type:** String
- **Required:** No

The ID of an AWS managed customer master key (CMK) for SNS or a custom CMK. For more information, see [Key terms](https://docs.aws.amazon.com/sns/latest/dg/sns-server-side-encryption.html#sse-key-terms). For more examples, see ``KeyId`` in the *API Reference*. This property applies only to [server-side-encryption](https://docs.aws.amazon.com/sns/latest/dg/sns-server-side-encryption.html).

### `maximum_message_size`

- **Type:** Int(1024..=1048576)
- **Required:** No

The maximum payload size, in bytes, for messages published to the topic. Valid values are ``1024`` to ``1048576`` (1 MiB). When this property is not specified, the topic uses the default SNS message size of ``262144`` bytes (256 KiB). A topic with ``MaximumMessageSize`` above ``262144`` supports a maximum of 100 subscriptions, all of which must use the SQS, AWS Lambda, or Amazon Data Firehose protocols. For more information, see [Publishing large messages with](https://docs.aws.amazon.com/sns/latest/dg/large-message-payloads.html) in the *Developer Guide*.

### `signature_version`

- **Type:** String
- **Required:** No

The signature version corresponds to the hashing algorithm used while creating the signature of the notifications, subscription confirmations, or unsubscribe confirmation messages sent by Amazon SNS. By default, ``SignatureVersion`` is set to ``1``.

### `tags`

- **Type:** `Map<String, String>`
- **Required:** No

The list of tags to add to a new topic. To be able to tag a topic on creation, you must have the ``sns:CreateTopic`` and ``sns:TagResource`` permissions.

### `topic_name`

- **Type:** String
- **Required:** No
- **Create-only:** Yes

The name of the topic you want to create. Topic names must include only uppercase and lowercase ASCII letters, numbers, underscores, and hyphens, and must be between 1 and 256 characters long. FIFO topic names must end with ``.fifo``. If you don't specify a name, CFN generates a unique physical ID and uses that ID for the topic name. For more information, see [Name type](https://docs.aws.amazon.com/AWSCloudFormation/latest/UserGuide/aws-properties-name.html). If you specify a name, you can't perform updates that require replacement of this resource. You can perform updates that require no or some interruption. If you must replace the resource, specify a new name.

### `tracing_config`

- **Type:** String
- **Required:** No

Tracing mode of an SNS topic. By default ``TracingConfig`` is set to ``PassThrough``, and the topic passes through the tracing header it receives from an SNS publisher to its subscriptions. If set to ``Active``, SNS will vend X-Ray segment data to topic owner account if the sampled flag in the tracing header is true.

## Not yet supported

These CloudFormation properties require the free-form JSON document type tracked by [carina-rs/carina#4144](https://github.com/carina-rs/carina/issues/4144):

- `data_protection_policy` — This policy can contain nested, heterogeneous JSON values.
- `archive_policy` — This policy can contain nested, heterogeneous JSON values.

These CloudFormation properties are deliberately omitted because their provider read-back conflicts with Carina's resource model:

- `subscription` — Cloud Control aggregates separately managed topic subscriptions; use awscc.sns.Subscription instead.

## Enum Values

### delivery_status_logging.protocol (Protocol)

| Value | DSL Identifier |
|-------|----------------|
| `http/s` | `aws.sns.Topic.LoggingConfig.Protocol.http_s` |
| `sqs` | `aws.sns.Topic.LoggingConfig.Protocol.sqs` |
| `lambda` | `aws.sns.Topic.LoggingConfig.Protocol.lambda` |
| `firehose` | `aws.sns.Topic.LoggingConfig.Protocol.firehose` |
| `application` | `aws.sns.Topic.LoggingConfig.Protocol.application` |

Shorthand formats: `http_s` or `Protocol.http_s`

## Struct Definitions

### delivery_status_logging

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `failure_feedback_role_arn` | Arn | No | The IAM role ARN to be used when logging failed message deliveries in Amazon CloudWatch. |
| `protocol` | [Enum (Protocol)](#delivery_status_loggingprotocol-protocol) | Yes | Indicates one of the supported protocols for the Amazon SNS topic. At least one of the other three ``LoggingConfig`` properties is recommend along with ``Protocol``. |
| `success_feedback_role_arn` | Arn | No | The IAM role ARN to be used when logging successful message deliveries in Amazon CloudWatch. |
| `success_feedback_sample_rate` | String | No | The percentage of successful message deliveries to be logged in Amazon CloudWatch. Valid percentage values range from 0 to 100. |

## Attribute Reference

### `topic_arn`

- **Type:** Arn



