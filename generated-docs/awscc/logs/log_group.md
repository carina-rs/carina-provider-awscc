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

### `data_protection_policy`

- **Type:** `Struct(LogGroupDataProtectionPolicy)`
- **Required:** No

Creates a data protection policy and assigns it to the log group. A data protection policy can help safeguard sensitive data that's ingested by the log group by auditing and masking the sensitive log data. When a user who does not have permission to view masked data views a log event that includes masked data, the sensitive data is replaced by asterisks.

### `deletion_protection_enabled`

- **Type:** Bool
- **Required:** No
- **Default:** `false`

Indicates whether deletion protection is enabled for this log group. When enabled, deletion protection blocks all deletion operations until it is explicitly disabled.

### `field_index_policies`

- **Type:** `List<Struct(FieldIndexPolicy)> (items: ..=1) (unique items)`
- **Required:** No
- **Default:** `[]`

Creates or updates a *field index policy* for the specified log group. Only log groups in the Standard log class support field index policies. For more information about log classes, see [Log classes](https://docs.aws.amazon.com/AmazonCloudWatch/latest/logs/CloudWatch_Logs_Log_Classes.html). You can use field index policies to create *field indexes* on fields found in log events in the log group. Creating field indexes lowers the costs for CWL Insights queries that reference those field indexes, because these queries attempt to skip the processing of log events that are known to not match the indexed field. Good fields to index are fields that you often need to query for and fields that have high cardinality of values Common examples of indexes include request ID, session ID, userID, and instance IDs. For more information, see [Create field indexes to improve query performance and reduce costs](https://docs.aws.amazon.com/AmazonCloudWatch/latest/logs/CloudWatchLogs-Field-Indexing.html). Currently, this array supports only one field index policy object.

### `kms_key_id`

- **Type:** aws.kms.Key.Reference
- **Required:** No

The Amazon Resource Name (ARN) of the KMS key to use when encrypting log data. To associate an KMS key with the log group, specify the ARN of that KMS key here. If you do so, ingested data is encrypted using this key. This association is stored as long as the data encrypted with the KMS key is still within CWL. This enables CWL to decrypt this data whenever it is requested. If you attempt to associate a KMS key with the log group but the KMS key doesn't exist or is deactivated, you will receive an ``InvalidParameterException`` error. Log group data is always encrypted in CWL. If you omit this key, the encryption does not use KMS. For more information, see [Encrypt log data in using](https://docs.aws.amazon.com/AmazonCloudWatch/latest/logs/encrypt-log-data-kms.html)

### `log_group_class`

- **Type:** [Enum (LogGroupClass)](#log_group_class-loggroupclass)
- **Required:** No
- **Default:** `"STANDARD"`

Specifies the log group class for this log group. There are two classes: + The ``Standard`` log class supports all CWL features. + The ``Infrequent Access`` log class supports a subset of CWL features and incurs lower costs. For details about the features supported by each class, see [Log classes](https://docs.aws.amazon.com/AmazonCloudWatch/latest/logs/CloudWatch_Logs_Log_Classes.html)

### `log_group_name`

- **Type:** aws.logs.LogGroup.LogGroupName — identity: String(pattern, len: 1..=512); member: String(pattern, len: 1..=512)
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

- **Type:** `Map<String(len: 1..=128), String(len: ..=256)>`
- **Required:** No

An array of key-value pairs to apply to the log group. For more information, see [Tag](https://docs.aws.amazon.com/AWSCloudFormation/latest/UserGuide/aws-properties-resource-tags.html).

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

- **Type:** aws.logs.LogGroup.Arn — identity: String(pattern); member: String



## AWS API narrowings

These fields are narrower than or equal to CFN. Input rules combine documented AWS domains, recorded live observations, and Carina canonical spellings for stable state comparison. A canonical DSL spelling does not claim AWS rejects other spellings. Attaching a type name adds no structural constraints. Field names below use CFN spelling.

| CFN field | Relation to CFN | Evidence URL | Verbatim quote / exact observation | Checked / observed |
| --- | --- | --- | --- | --- |
| `/properties/DataProtectionPolicy` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/AmazonCloudWatch/latest/logs/cloudwatch-logs-data-protection-policies.html) | ` Only one data protection policy can be defined per CloudWatch Logs log group. A data protection policy must include exactly two statements: one that specifies the audit operation, and one that specifies the de-identify operation. ` | 2026-11-07 |
| `/properties/DataProtectionPolicy` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/AmazonCloudWatch/latest/logs/cloudwatch-logs-data-protection-policies.html#data-protection-policy-json-properties) | ` Version – The policy language version. The current version is 2021-06-01. ` | 2026-11-07 |
| `/properties/DataProtectionPolicy` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/AmazonCloudWatch/latest/logs/mask-sensitive-log-data-start.html) | ` A single data protection policy can include up to 10 custom data identifiers. Each regular expression that defines a custom data identifier must be 200 characters or fewer. ` | 2026-11-07 |
| `/properties/FieldIndexPolicies` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/AWSCloudFormation/latest/TemplateReference/aws-resource-logs-loggroup.html#cfn-logs-loggroup-fieldindexpolicies) | ` Creates or updates a field index policy for the specified log group. Only log groups in the Standard log class support field index policies. For more information about log classes, see Log classes. `<br>` You can use field index policies to create field indexes on fields found in log events in the log group. Creating field indexes lowers the costs for CloudWatch Logs Insights queries that reference those field indexes, because these queries attempt to skip the processing of log events that are known to not match the indexed field. Good fields to index are fields that you often need to query for and fields that have high cardinality of values Common examples of indexes include request ID, session ID, userID, and instance IDs. For more information, see Create field indexes to improve query performance and reduce costs. `<br>` Currently, this array supports only one field index policy object. ` | 2026-11-07 |
| `/properties/FieldIndexPolicies` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/AmazonCloudWatchLogs/latest/APIReference/API_PutIndexPolicy.html) | ` You can use FieldsV2 to specify the type for each field. Supported types are FIELD_INDEX and FACET. Field names within Fields and FieldsV2 must be mutually exclusive. ` | 2026-11-07 |
| `/properties/LogGroupName` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/AmazonCloudWatchLogs/latest/APIReference/API_CreateLogGroup.html#CWL-CreateLogGroup-request-logGroupName) | ` A name for the log group. `<br>` Length Constraints: Minimum length of 1. Maximum length of 512. `<br>` Pattern: [\.\-_/#A-Za-z0-9]+ ` | 2026-11-07 |
| `/properties/ResourcePolicyDocument` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/IAM/latest/UserGuide/reference_policies_grammar.html) | ` <version_block> = "Version" : ("2008-10-17" \| "2012-10-17") `<br>` <effect_block> = "Effect" : ("Allow" \| "Deny") ` | 2026-11-07 |
| `/properties/ResourcePolicyDocument` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/IAM/latest/UserGuide/reference_policies_elements_statement.html) | ` The Statement element can contain a single statement or an array of individual statements. ` | 2026-11-07 |
| `/properties/ResourcePolicyDocument` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/IAM/latest/UserGuide/reference_policies_elements_condition.html) | ` You can create conditions that compare strings, dates, numbers, and more using typical Boolean comparisons such as equals, greater than, and less than. ` | 2026-11-07 |
| `/properties/RetentionInDays` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/AWSCloudFormation/latest/TemplateReference/aws-resource-logs-loggroup.html#cfn-logs-loggroup-retentionindays) | ` The number of days to retain the log events in the specified log group. Possible values are: 1, 3, 5, 7, 14, 30, 60, 90, 120, 150, 180, 365, 400, 545, 731, 1096, 1827, 2192, 2557, 2922, 3288, and 3653. `<br>` To set a log group so that its log events do not expire, do not specify this property. `<br>` Allowed values: 1 \| 3 \| 5 \| 7 \| 14 \| 30 \| 60 \| 90 \| 120 \| 150 \| 180 \| 365 \| 400 \| 545 \| 731 \| 1096 \| 1827 \| 2192 \| 2557 \| 2922 \| 3288 \| 3653 ` | 2026-11-07 |

