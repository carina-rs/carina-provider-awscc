---
title: "awscc.sns.Topic"
---


CloudFormation Type: `AWS::SNS::Topic`

The ``AWS::SNS::Topic`` resource creates a topic to which notifications can be published.
  One account can create a maximum of 100,000 standard topics and 1,000 FIFO topics. For more information, see [endpoints and quotas](https://docs.aws.amazon.com/general/latest/gr/sns.html) in the *General Reference*.
   The structure of ``AUTHPARAMS`` depends on the .signature of the API request. For more information, see [Examples of the complete Signature Version 4 signing process](https://docs.aws.amazon.com/general/latest/gr/sigv4-signed-request-examples.html) in the *General Reference*.

## Example

```crn
awscc.sns.Topic {
  topic_name        = 'carina-example-application-alerts'
  display_name      = 'Carina application alerts'
  signature_version = aws.sns.Topic.SignatureVersion.2

  tags = {
    Environment = 'example'
  }
}
```

## Argument Reference

### `archive_policy`

- **Type:** [Struct(TopicArchivePolicy)](#archive_policy-1)
- **Required:** No

The ``ArchivePolicy`` determines the number of days SNS retains messages in FIFO topics. You can set a retention period ranging from 1 to 365 days. This property is only applicable to FIFO topics; attempting to use it with standard topics will result in a creation failure.

### `content_based_deduplication`

- **Type:** `Bool`
- **Required:** No

``ContentBasedDeduplication`` enables deduplication of messages based on their content for FIFO topics. By default, this property is set to false. If you create a FIFO topic with ``ContentBasedDeduplication`` set to false, you must provide a ``MessageDeduplicationId`` for each ``Publish`` action. When set to true, SNS automatically generates a ``MessageDeduplicationId`` using a SHA-256 hash of the message body (excluding message attributes). You can optionally override this generated value by specifying a ``MessageDeduplicationId`` in the ``Publish`` action. Note that this property only applies to FIFO topics; using it with standard topics will cause the creation to fail.

### `delivery_status_logging`

- **Type:** List\<[Struct(LoggingConfig)](#delivery_status_logging-1)\> (unique items, unordered)
- **Required:** No

The ``DeliveryStatusLogging`` configuration enables you to log the delivery status of messages sent from your Amazon SNS topic to subscribed endpoints with the following supported delivery protocols: + HTTP + Amazon Kinesis Data Firehose + AWS Lambda + Platform application endpoint + Amazon Simple Queue Service Once configured, log entries are sent to Amazon CloudWatch Logs.

### `display_name`

- **Type:** `String`
- **Required:** No

The display name to use for an SNS topic with SMS subscriptions. The display name must be maximum 100 characters long, including hyphens (-), underscores (_), spaces, and tabs.

### `fifo_throughput_scope`

- **Type:** [FifoThroughputScope](#fifo_throughput_scope-fifothroughputscope)
- **Required:** No

Specifies the throughput quota and deduplication behavior to apply for the FIFO topic. Valid values are ``Topic`` or ``MessageGroup``.

### `fifo_topic`

- **Type:** `Bool`
- **Required:** No
- **Create-only:** Yes

Set to true to create a FIFO topic.

### `kms_master_key_id`

- **Type:** `aws.kms.Key.Reference (pattern: /^((arn:(?:aws|aws-cn|aws-us-gov|aws-iso|aws-iso-b|aws-iso-e|aws-iso-f|aws-eusc):kms:[a-z]{2,4}(-[a-z0-9]+)+-[0-9]+:[0-9]{12}:(key/([0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12}|mrk-[0-9a-fA-F]{32})|alias/[a-zA-Z0-9:/_-]+))|(alias/[a-zA-Z0-9:/_-]+)|([0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12})|(mrk-[0-9a-fA-F]{32}))$/ (Full), len: 1..=2048)`
- **Required:** No

The ID of an AWS managed customer master key (CMK) for SNS or a custom CMK. For more information, see [Key terms](https://docs.aws.amazon.com/sns/latest/dg/sns-server-side-encryption.html#sse-key-terms). For more examples, see ``KeyId`` in the *API Reference*. This property applies only to [server-side-encryption](https://docs.aws.amazon.com/sns/latest/dg/sns-server-side-encryption.html).

### `maximum_message_size`

- **Type:** `Int (range: 1024..=1048576)`
- **Required:** No

The maximum payload size, in bytes, for messages published to the topic. Valid values are ``1024`` to ``1048576`` (1 MiB). When this property is not specified, the topic uses the default SNS message size of ``262144`` bytes (256 KiB). A topic with ``MaximumMessageSize`` above ``262144`` supports a maximum of 100 subscriptions, all of which must use the SQS, AWS Lambda, or Amazon Data Firehose protocols. For more information, see [Publishing large messages with](https://docs.aws.amazon.com/sns/latest/dg/large-message-payloads.html) in the *Developer Guide*.

### `signature_version`

- **Type:** [SignatureVersion](#signature_version-signatureversion)
- **Required:** No

The signature version corresponds to the hashing algorithm used while creating the signature of the notifications, subscription confirmations, or unsubscribe confirmation messages sent by Amazon SNS. By default, ``SignatureVersion`` is set to ``1``.

### `tags`

- **Type:** Map\<`String`, `String`\>
- **Required:** No

The list of tags to add to a new topic. To be able to tag a topic on creation, you must have the ``sns:CreateTopic`` and ``sns:TagResource`` permissions.

### `topic_name`

- **Type:** `String (pattern: /^[A-Za-z0-9_-]+(?:\.fifo)?$/ (Full), len: 1..=256)`
- **Required:** No
- **Create-only:** Yes

The name of the topic you want to create. Topic names must include only uppercase and lowercase ASCII letters, numbers, underscores, and hyphens, and must be between 1 and 256 characters long. FIFO topic names must end with ``.fifo``. If you don't specify a name, CFN generates a unique physical ID and uses that ID for the topic name. For more information, see [Name type](https://docs.aws.amazon.com/AWSCloudFormation/latest/UserGuide/aws-properties-name.html). If you specify a name, you can't perform updates that require replacement of this resource. You can perform updates that require no or some interruption. If you must replace the resource, specify a new name.

### `tracing_config`

- **Type:** [TracingConfig](#tracing_config-tracingconfig)
- **Required:** No

Tracing mode of an SNS topic. By default ``TracingConfig`` is set to ``PassThrough``, and the topic passes through the tracing header it receives from an SNS publisher to its subscriptions. If set to ``Active``, SNS will vend X-Ray segment data to topic owner account if the sampled flag in the tracing header is true.

## Not yet supported

These CloudFormation properties are deliberately omitted for the provider-specific reasons below:

- `subscription` — Cloud Control aggregates separately managed topic subscriptions; use awscc.sns.Subscription instead.
- `data_protection_policy` — AWS SNS message data protection is no longer available to new customers; Cloud Control rejected it in ap-northeast-1 and us-east-1 on 2026-10-31.

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

### fifo_throughput_scope (FifoThroughputScope)

| Value | DSL Identifier |
|-------|----------------|
| `Topic` | `aws.sns.Topic.FifoThroughputScope.topic` |
| `MessageGroup` | `aws.sns.Topic.FifoThroughputScope.message_group` |

Shorthand formats: `topic` or `FifoThroughputScope.topic`

### signature_version (SignatureVersion)

| Value | DSL Identifier |
|-------|----------------|
| `1` | `aws.sns.Topic.SignatureVersion.1` |
| `2` | `aws.sns.Topic.SignatureVersion.2` |

Use the fully qualified DSL identifiers above; bare numeric values parse as integers.

### tracing_config (TracingConfig)

| Value | DSL Identifier |
|-------|----------------|
| `PassThrough` | `aws.sns.Topic.TracingConfig.pass_through` |
| `Active` | `aws.sns.Topic.TracingConfig.active` |

Shorthand formats: `pass_through` or `TracingConfig.pass_through`

## Struct Definitions

### archive_policy

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `message_retention_period` | `Int (range: 1..=365)` | No | Number of days that the FIFO topic retains messages for replay. |

### delivery_status_logging

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `failure_feedback_role_arn` | `aws.Arn (pattern: /^arn:(?:aws\|aws-cn\|aws-us-gov\|aws-iso\|aws-iso-b\|aws-iso-e\|aws-iso-f\|aws-eusc):[^:]+:.*$/ (Full), custom validator)` | No | The IAM role ARN to be used when logging failed message deliveries in Amazon CloudWatch. |
| `protocol` | [Protocol](#delivery_status_loggingprotocol-protocol) | Yes | Indicates one of the supported protocols for the Amazon SNS topic. At least one of the other three ``LoggingConfig`` properties is recommend along with ``Protocol``. |
| `success_feedback_role_arn` | `aws.Arn (pattern: /^arn:(?:aws\|aws-cn\|aws-us-gov\|aws-iso\|aws-iso-b\|aws-iso-e\|aws-iso-f\|aws-eusc):[^:]+:.*$/ (Full), custom validator)` | No | The IAM role ARN to be used when logging successful message deliveries in Amazon CloudWatch. |
| `success_feedback_sample_rate` | `String (pattern: /^0*(?:100\|[0-9]{1,2})$/ (Full))` | No | The percentage of successful message deliveries to be logged in Amazon CloudWatch. Valid percentage values range from 0 to 100. |

## Attribute Reference

### `topic_arn`

- **Type:** `aws.Arn (pattern: /^arn:(?:aws|aws-cn|aws-us-gov|aws-iso|aws-iso-b|aws-iso-e|aws-iso-f|aws-eusc):[^:]+:.*$/ (Full), custom validator)`



## AWS API narrowings

These fields are narrower than or equal to CFN. Input rules combine documented AWS domains, recorded live observations, and Carina canonical spellings for stable state comparison. A canonical DSL spelling does not claim AWS rejects other spellings. Attaching a type name adds no structural constraints. Field names below use CFN spelling.

| CFN field | Relation to CFN | Evidence URL | Verbatim quote / exact observation | Checked / observed |
| --- | --- | --- | --- | --- |
| `/definitions/LoggingConfig/properties/SuccessFeedbackSampleRate` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/sns/latest/api/API_SetTopicAttributes.html) | ` The <ENDPOINT>SuccessFeedbackRoleArn and <ENDPOINT>FailureFeedbackRoleArn attributes are used to give Amazon SNS write access to use CloudWatch Logs on your behalf. The <ENDPOINT>SuccessFeedbackSampleRate attribute is for specifying the sample rate percentage (0-100) of successfully delivered messages. After you configure the <ENDPOINT>FailureFeedbackRoleArn attribute, then all failed message deliveries generate CloudWatch Logs. ` | 2026-11-07 |
| `/definitions/Subscription/properties/Protocol` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/sns/latest/api/API_Subscribe.html) | ` The protocol that you want to use. Supported protocols include: `<br>` http – delivery of JSON-encoded message via HTTP POST `<br>` https – delivery of JSON-encoded message via HTTPS POST `<br>` email – delivery of message via SMTP `<br>` email-json – delivery of JSON-encoded message via SMTP `<br>` sms – delivery of message via SMS `<br>` sqs – delivery of JSON-encoded message to an Amazon SQS queue `<br>` application – delivery of JSON-encoded message to an EndpointArn for a mobile app and device `<br>` lambda – delivery of JSON-encoded message to an AWS Lambda function `<br>` firehose – delivery of JSON-encoded message to an Amazon Data Firehose delivery stream. ` | 2026-11-07 |
| `/properties/ArchivePolicy` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/sns/latest/dg/message-archiving-and-replay-topic-owner.html) | ` Message archiving lets you archive a single copy of all messages published to your topic. You can store published messages within your topic by enabling the message archive policy on the topic, which enables message archiving for all subscriptions linked to that topic. Messages can be archived for a minimum of one day to a maximum of 365 days. `<br>` On the Topic details page, the Retention policy displays the status of the archive policy, including the number of days for which it is set. Select the Archive policy tab to view the following message archive details: Status – The archive and replay status appears as active when an archive policy is applied. The archive and replay status appears as inactive when the archive policy is set to an empty JSON object.Message retention period – The specified number of days for message retention.Archive start date – The date from which subscribers can replay messages.JSON preview – The JSON preview of the archive policy. `<br>` Status – The archive and replay status appears as active when an archive policy is applied. The archive and replay status appears as inactive when the archive policy is set to an empty JSON object. `<br>` To create a message archive policy using the API, you need to add the attribute ArchivePolicy to your topic. You can set an ArchivePolicy using the API actions CreateTopic and SetTopicAttributes. ArchivePolicy has a single value, MessageRetentionPeriod, which represents the number of days Amazon SNS retains messages. To activate message archiving for your topic, set the MessageRetentionPeriod to an integer value greater than zero. For example, to retain messages in your archive for 30 days, set the ArchivePolicy to: ` | 2026-11-07 |
| `/properties/FifoThroughputScope` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/sns/latest/api/API_SetTopicAttributes.html) | ` FifoThroughputScope – Enables higher throughput for your FIFO topic by adjusting the scope of deduplication. This attribute has two possible values: Topic – The scope of message deduplication is across the entire topic. This is the default value and maintains existing behavior, with a maximum throughput of 3000 messages per second or 20MB per second, whichever comes first.MessageGroup – The scope of deduplication is within each individual message group, which enables higher throughput per topic subject to regional quotas. For more information on quotas or to request an increase, see Amazon SNS service quotas in the Amazon Web Services General Reference. `<br>` FifoThroughputScope – Enables higher throughput for your FIFO topic by adjusting the scope of deduplication. This attribute has two possible values: `<br>` Topic – The scope of message deduplication is across the entire topic. This is the default value and maintains existing behavior, with a maximum throughput of 3000 messages per second or 20MB per second, whichever comes first. `<br>` MessageGroup – The scope of deduplication is within each individual message group, which enables higher throughput per topic subject to regional quotas. For more information on quotas or to request an increase, see Amazon SNS service quotas in the Amazon Web Services General Reference. ` | 2026-11-07 |
| `/properties/SignatureVersion` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/sns/latest/dg/sns-verify-signature-of-message-configure-message-signature.html) | ` Select between SignatureVersion1 (SHA1) and SignatureVersion2 (SHA256) to control the hashing algorithm used for signing messages. Amazon SNS topics default to SignatureVersion1. You can configure this setting using the SetTopicAttributes API action. `<br>` SignatureVersion `<br>` Use the following example to set the topic attribute SignatureVersion using the AWS CLI: `<br>` aws sns set-topic-attributes \ --topic-arn arn:aws:sns:us-east-2:123456789012:MyTopic \ --attribute-name SignatureVersion \ --attribute-value 2 ` | 2026-11-07 |
| `/properties/TopicName` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/sns/latest/api/API_CreateTopic.html) | ` The name of the topic you want to create. `<br>` Constraints: Topic names must be made up of only uppercase and lowercase ASCII letters, numbers, underscores, and hyphens, and must be between 1 and 256 characters long. `<br>` For a FIFO (first-in-first-out) topic, the name must end with the .fifo suffix. ` | 2026-11-07 |
| `/properties/TracingConfig` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/sns/latest/api/API_SetTopicAttributes.html) | ` TracingConfig – Tracing mode of an Amazon SNS topic. By default TracingConfig is set to PassThrough, and the topic passes through the tracing header it receives from an Amazon SNS publisher to its subscriptions. If set to Active, Amazon SNS will vend X-Ray segment data to topic owner account if the sampled flag in the tracing header is true. This is only supported on standard topics. ` | 2026-11-07 |

