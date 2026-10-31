---
title: "awscc.logs.LogGroup"
---


CloudFormation Type: `AWS::Logs::LogGroup`

The ``AWS::Logs::LogGroup`` resource specifies a log group. A log group defines common properties for log streams, such as their retention and access control rules. Each log stream must belong to one log group.
 You can create up to 1,000,000 log groups per Region per account. You must use the following guidelines when naming a log group:
  +  Log group names must be unique within a Region for an AWS account.
  +  Log group names can be between 1 and 512 characters long.
  +  Log group names consist of the following characters: a-z, A-Z, 0-9, '_' (underscore), '-' (hyphen), '/' (forward slash), and '.' (period).

## Example

```crn
awscc.logs.LogGroup {
  log_group_name    = '/example/my-app'
  retention_in_days = 30

  tags = {
    Environment = 'example'
  }
}
```

## Argument Reference

### `deletion_protection_enabled`

- **Type:** Bool
- **Required:** No
- **Default:** `false`

Indicates whether deletion protection is enabled for this log group. When enabled, deletion protection blocks all deletion operations until it is explicitly disabled.

### `kms_key_id`

- **Type:** KmsKeyArn
- **Required:** No

The Amazon Resource Name (ARN) of the KMS key to use when encrypting log data. To associate an KMS key with the log group, specify the ARN of that KMS key here. If you do so, ingested data is encrypted using this key. This association is stored as long as the data encrypted with the KMS key is still within CWL. This enables CWL to decrypt this data whenever it is requested. If you attempt to associate a KMS key with the log group but the KMS key doesn't exist or is deactivated, you will receive an ``InvalidParameterException`` error. Log group data is always encrypted in CWL. If you omit this key, the encryption does not use KMS. For more information, see [Encrypt log data in using](https://docs.aws.amazon.com/AmazonCloudWatch/latest/logs/encrypt-log-data-kms.html)

### `log_group_class`

- **Type:** [Enum (LogGroupClass)](#log_group_class-loggroupclass)
- **Required:** No
- **Default:** `"STANDARD"`

Specifies the log group class for this log group. There are two classes: + The ``Standard`` log class supports all CWL features. + The ``Infrequent Access`` log class supports a subset of CWL features and incurs lower costs. For details about the features supported by each class, see [Log classes](https://docs.aws.amazon.com/AmazonCloudWatch/latest/logs/CloudWatch_Logs_Log_Classes.html)

### `log_group_name`

- **Type:** String(pattern, len: 1..=512)
- **Required:** No
- **Create-only:** Yes

The name of the log group. If you don't specify a name, CFNlong generates a unique ID for the log group.

### `resource_policy_document`

- **Type:** PolicyDocument
- **Required:** No

Creates or updates a resource policy for the specified log group that allows other services to put log events to this account. A LogGroup can have 1 resource policy.

### `retention_in_days`

- **Type:** IntEnum([1, 3, 5, 7, 14, 30, 60, 90, 120, 150, 180, 365, 400, 545, 731, 1096, 1827, 2192, 2557, 2922, 3288, 3653])
- **Required:** No

The number of days to retain the log events in the specified log group. Possible values are: 1, 3, 5, 7, 14, 30, 60, 90, 120, 150, 180, 365, 400, 545, 731, 1096, 1827, 2192, 2557, 2922, 3288, and 3653. To set a log group so that its log events do not expire, do not specify this property.

### `tags`

- **Type:** `Map<String, String>`
- **Required:** No

An array of key-value pairs to apply to the log group. For more information, see [Tag](https://docs.aws.amazon.com/AWSCloudFormation/latest/UserGuide/aws-properties-resource-tags.html).

## Not yet supported

These CloudFormation properties require the free-form JSON document type tracked by [carina-rs/carina#4144](https://github.com/carina-rs/carina/issues/4144):

- `data_protection_policy` — This policy can contain nested, heterogeneous JSON values.
- `field_index_policies` — Each field-index policy is a free-form JSON document whose Fields value is an array.

## Enum Values

### log_group_class (LogGroupClass)

| Value | DSL Identifier |
|-------|----------------|
| `STANDARD` | `aws.logs.LogGroup.LogGroupClass.standard` |
| `INFREQUENT_ACCESS` | `aws.logs.LogGroup.LogGroupClass.infrequent_access` |
| `DELIVERY` | `aws.logs.LogGroup.LogGroupClass.delivery` |

Shorthand formats: `standard` or `LogGroupClass.standard`

## Attribute Reference

### `arn`

- **Type:** Arn



