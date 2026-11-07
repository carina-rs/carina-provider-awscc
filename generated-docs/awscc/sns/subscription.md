---
title: "awscc.sns.Subscription"
---


CloudFormation Type: `AWS::SNS::Subscription`

Resource Type definition for AWS::SNS::Subscription

## Example

```crn
let alerts = awscc.sns.Topic {
  topic_name   = 'carina-example-subscription-alerts'
  display_name = 'Carina example alerts'

  tags = {
    Environment = 'example'
  }
}

awscc.sns.Subscription {
  topic_arn = alerts.topic_arn
  protocol  = email
  # Replace this placeholder with an email address you control. SNS sends a
  # confirmation email; the subscription delivers nothing until the recipient
  # confirms it.
  endpoint  = 'alerts@example.com'
}
```

## Argument Reference

### `delivery_policy`

- **Type:** `Struct(SubscriptionDeliveryPolicy)`
- **Required:** No

The delivery policy JSON assigned to the subscription. Enables the subscriber to define the message delivery retry strategy in the case of an HTTP/S endpoint subscribed to the topic.

### `endpoint`

- **Type:** String
- **Required:** No
- **Create-only:** Yes

The subscription's endpoint. The endpoint value depends on the protocol that you specify. 

### `filter_policy_scope`

- **Type:** [Enum (FilterPolicyScope)](#filter_policy_scope-filterpolicyscope)
- **Required:** No

This attribute lets you choose the filtering scope by using one of the following string value types: MessageAttributes (default) and MessageBody.

### `protocol`

- **Type:** [Enum (Protocol)](#protocol-protocol)
- **Required:** Yes
- **Create-only:** Yes

The subscription's protocol.

### `raw_message_delivery`

- **Type:** Bool
- **Required:** No

When set to true, enables raw message delivery. Raw messages don't contain any JSON formatting and can be sent to Amazon SQS and HTTP/S endpoints.

### `redrive_policy`

- **Type:** `Struct(SubscriptionRedrivePolicy)`
- **Required:** No

When specified, sends undeliverable messages to the specified Amazon SQS dead-letter queue. Messages that can't be delivered due to client errors are held in the dead-letter queue for further analysis or reprocessing.

### `region`

- **Type:** `aws.Region`
- **Required:** No
- **Write-only:** Yes

For cross-region subscriptions, the region in which the topic resides.If no region is specified, AWS CloudFormation uses the region of the caller as the default.

### `replay_policy`

- **Type:** `Struct(SubscriptionReplayPolicy)`
- **Required:** No

Specifies whether Amazon SNS resends the notification to the subscription when a message's attribute changes.

### `subscription_role_arn`

- **Type:** `aws.Arn`
- **Required:** No

This property applies only to Amazon Data Firehose delivery stream subscriptions.

### `topic_arn`

- **Type:** `aws.Arn`
- **Required:** Yes
- **Create-only:** Yes

The ARN of the topic to subscribe to.

## Not yet supported

These CloudFormation properties require the free-form JSON document type tracked by [carina-rs/carina#4144](https://github.com/carina-rs/carina/issues/4144):

- `filter_policy` — Filter keys are user-defined message attribute or body-property names, and the operator grammar nests recursively; this remains blocked on carina-rs/carina#4144.

## Enum Values

### filter_policy_scope (FilterPolicyScope)

| Value | DSL Identifier |
|-------|----------------|
| `MessageAttributes` | `aws.sns.Subscription.FilterPolicyScope.message_attributes` |
| `MessageBody` | `aws.sns.Subscription.FilterPolicyScope.message_body` |

Shorthand formats: `message_attributes` or `FilterPolicyScope.message_attributes`

### protocol (Protocol)

| Value | DSL Identifier |
|-------|----------------|
| `http` | `aws.sns.Subscription.Protocol.http` |
| `https` | `aws.sns.Subscription.Protocol.https` |
| `email` | `aws.sns.Subscription.Protocol.email` |
| `email-json` | `aws.sns.Subscription.Protocol.email_json` |
| `sms` | `aws.sns.Subscription.Protocol.sms` |
| `sqs` | `aws.sns.Subscription.Protocol.sqs` |
| `application` | `aws.sns.Subscription.Protocol.application` |
| `lambda` | `aws.sns.Subscription.Protocol.lambda` |
| `firehose` | `aws.sns.Subscription.Protocol.firehose` |

Shorthand formats: `http` or `Protocol.http`

## Attribute Reference

### `arn`

- **Type:** `aws.Arn`

Arn of the subscription

## AWS API narrowings

These fields are narrower than or equal to CFN. Input rules combine documented AWS domains, recorded live observations, and Carina canonical spellings for stable state comparison. A canonical DSL spelling does not claim AWS rejects other spellings. Attaching a type name adds no structural constraints. Field names below use CFN spelling.

| CFN field | Relation to CFN | Evidence URL | Verbatim quote / exact observation | Checked / observed |
| --- | --- | --- | --- | --- |
| `/properties/DeliveryPolicy` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/sns/latest/dg/sns-message-delivery-retries.html) | ` numRetries The total number of retries, including immediate, pre-backoff, backoff, and post-backoff retries. 0 to 100Default: 3 ` | 2026-11-07 |
| `/properties/DeliveryPolicy` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/sns/latest/dg/sns-message-delivery-retries.html#creating-delivery-policy) | ` maxReceivesPerSecond The maximum average number of message deliveries per second, per subscription. 1 or greaterDefault: No throttling (no limit on delivery rate) ` | 2026-11-07 |
| `/properties/FilterPolicyScope` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/sns/latest/api/API_Subscribe.html) | ` FilterPolicyScope – This attribute lets you choose the filtering scope by using one of the following string value types: MessageAttributes (default) – The filter is applied on the message attributes.MessageBody – The filter is applied on the message body. `<br>` FilterPolicyScope – This attribute lets you choose the filtering scope by using one of the following string value types: `<br>` MessageAttributes (default) – The filter is applied on the message attributes. `<br>` MessageBody – The filter is applied on the message body. ` | 2026-11-07 |
| `/properties/Protocol` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/sns/latest/api/API_Subscribe.html) | ` The protocol that you want to use. Supported protocols include: `<br>` http – delivery of JSON-encoded message via HTTP POST `<br>` https – delivery of JSON-encoded message via HTTPS POST `<br>` email – delivery of message via SMTP `<br>` email-json – delivery of JSON-encoded message via SMTP `<br>` sms – delivery of message via SMS `<br>` sqs – delivery of JSON-encoded message to an Amazon SQS queue `<br>` application – delivery of JSON-encoded message to an EndpointArn for a mobile app and device `<br>` lambda – delivery of JSON-encoded message to an AWS Lambda function `<br>` firehose – delivery of JSON-encoded message to an Amazon Data Firehose delivery stream. ` | 2026-11-07 |
| `/properties/RedrivePolicy` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/sns/latest/dg/sns-dead-letter-queues.html) | ` Your messages are moved into a dead-letter queue using a redrive policy. A redrive policy is a JSON object that refers to the ARN of the dead-letter queue. The deadLetterTargetArn attribute specifies the ARN. The ARN must point to an Amazon SQS queue in the same AWS account and Region as your Amazon SNS subscription. For more information, see Configuring an Amazon SNS dead-letter queue for a subscription. `<br>` The following JSON object is a sample redrive policy, attached to an SNS subscription. `<br>` { "deadLetterTargetArn": "arn:aws:sqs:us-east-2:123456789012:MyDeadLetterQueue" } ` | 2026-11-07 |
| `/properties/ReplayPolicy` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/sns/latest/dg/message-archiving-and-replay-subscriber.html) | ` StartingPoint (Required) – Signals where to start replaying messages from. `<br>` EndingPoint (Optional) – Signals when to stop replaying messages. If EndingPoint is omitted, then the replay will continue until caught up to the current time. `<br>` PointType (Required) – Sets the type of starting and ending points. Currently, the supported value for PointType is Timestamp. ` | 2026-11-07 |

