---
title: "awscc.sns.Subscription"
---


CloudFormation Type: `AWS::SNS::Subscription`

Resource Type definition for AWS::SNS::Subscription

## Argument Reference

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

### `region`

- **Type:** Region
- **Required:** No
- **Write-only:** Yes

For cross-region subscriptions, the region in which the topic resides.If no region is specified, AWS CloudFormation uses the region of the caller as the default.

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

- `delivery_policy` — Cloud Control canonicalizes this free-form JSON document to an object.
- `filter_policy` — Cloud Control canonicalizes this free-form JSON document to an object.
- `redrive_policy` — Cloud Control canonicalizes this free-form JSON document to an object.
- `replay_policy` — Cloud Control canonicalizes this free-form JSON document to an object.

## Attribute Reference

### `arn`

- **Type:** Arn

Arn of the subscription

