---
title: "awscc.sns.Subscription"
---


CloudFormation Type: `AWS::SNS::Subscription`

Resource Type definition for AWS::SNS::Subscription

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

- **Type:** String
- **Required:** No

This attribute lets you choose the filtering scope by using one of the following string value types: MessageAttributes (default) and MessageBody.

### `protocol`

- **Type:** String
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

- **Type:** Region
- **Required:** No
- **Write-only:** Yes

For cross-region subscriptions, the region in which the topic resides.If no region is specified, AWS CloudFormation uses the region of the caller as the default.

### `replay_policy`

- **Type:** `Struct(SubscriptionReplayPolicy)`
- **Required:** No

Specifies whether Amazon SNS resends the notification to the subscription when a message's attribute changes.

### `subscription_role_arn`

- **Type:** Arn
- **Required:** No

This property applies only to Amazon Data Firehose delivery stream subscriptions.

### `topic_arn`

- **Type:** Arn
- **Required:** Yes
- **Create-only:** Yes

The ARN of the topic to subscribe to.

## Not yet supported

These CloudFormation properties require the free-form JSON document type tracked by [carina-rs/carina#4144](https://github.com/carina-rs/carina/issues/4144):

- `filter_policy` — Filter keys are user-defined message attribute or body-property names, and the operator grammar nests recursively; this remains blocked on carina-rs/carina#4144.

## Attribute Reference

### `arn`

- **Type:** Arn

Arn of the subscription

