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

- **Type:** [Struct(LogGroupDataProtectionPolicy)](#data_protection_policy-1)
- **Required:** No

Creates a data protection policy and assigns it to the log group. A data protection policy can help safeguard sensitive data that's ingested by the log group by auditing and masking the sensitive log data. When a user who does not have permission to view masked data views a log event that includes masked data, the sensitive data is replaced by asterisks.

### `deletion_protection_enabled`

- **Type:** `Bool`
- **Required:** No
- **Default:** `false`

Indicates whether deletion protection is enabled for this log group. When enabled, deletion protection blocks all deletion operations until it is explicitly disabled.

### `field_index_policies`

- **Type:** List\<[Struct(FieldIndexPolicy)](#field_index_policies-1)\> (items: unbounded..=1, unique items, unordered)
- **Required:** No
- **Default:** `[]`

Creates or updates a *field index policy* for the specified log group. Only log groups in the Standard log class support field index policies. For more information about log classes, see [Log classes](https://docs.aws.amazon.com/AmazonCloudWatch/latest/logs/CloudWatch_Logs_Log_Classes.html). You can use field index policies to create *field indexes* on fields found in log events in the log group. Creating field indexes lowers the costs for CWL Insights queries that reference those field indexes, because these queries attempt to skip the processing of log events that are known to not match the indexed field. Good fields to index are fields that you often need to query for and fields that have high cardinality of values Common examples of indexes include request ID, session ID, userID, and instance IDs. For more information, see [Create field indexes to improve query performance and reduce costs](https://docs.aws.amazon.com/AmazonCloudWatch/latest/logs/CloudWatchLogs-Field-Indexing.html). Currently, this array supports only one field index policy object.

### `kms_key_id`

- **Type:** `aws.kms.Key.Reference (pattern: /^((arn:(?:aws|aws-cn|aws-us-gov|aws-iso|aws-iso-b|aws-iso-e|aws-iso-f|aws-eusc):kms:[a-z]{2,4}(-[a-z0-9]+)+-[0-9]+:[0-9]{12}:(key/([0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12}|mrk-[0-9a-fA-F]{32})|alias/[a-zA-Z0-9:/_-]+))|(alias/[a-zA-Z0-9:/_-]+)|([0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12})|(mrk-[0-9a-fA-F]{32}))$/ (Full), len: 1..=2048)`
- **Required:** No

The Amazon Resource Name (ARN) of the KMS key to use when encrypting log data. To associate an KMS key with the log group, specify the ARN of that KMS key here. If you do so, ingested data is encrypted using this key. This association is stored as long as the data encrypted with the KMS key is still within CWL. This enables CWL to decrypt this data whenever it is requested. If you attempt to associate a KMS key with the log group but the KMS key doesn't exist or is deactivated, you will receive an ``InvalidParameterException`` error. Log group data is always encrypted in CWL. If you omit this key, the encryption does not use KMS. For more information, see [Encrypt log data in using](https://docs.aws.amazon.com/AmazonCloudWatch/latest/logs/encrypt-log-data-kms.html)

### `log_group_class`

- **Type:** [LogGroupClass](#log_group_class-loggroupclass)
- **Required:** No
- **Default:** `"STANDARD"`

Specifies the log group class for this log group. There are two classes: + The ``Standard`` log class supports all CWL features. + The ``Infrequent Access`` log class supports a subset of CWL features and incurs lower costs. For details about the features supported by each class, see [Log classes](https://docs.aws.amazon.com/AmazonCloudWatch/latest/logs/CloudWatch_Logs_Log_Classes.html)

### `log_group_name`

- **Type:** `aws.logs.LogGroup.LogGroupName (identity constraints: pattern: /^[.\-_/#A-Za-z0-9]{1,512}\z/ (Full), len: 1..=512, member constraints: pattern: /^[.\-_/#A-Za-z0-9]{1,512}\z/ (Full), len: 1..=512, custom validator)`
- **Required:** No
- **Create-only:** Yes

The name of the log group. If you don't specify a name, CFNlong generates a unique ID for the log group.

### `resource_policy_document`

- **Type:** [Struct(PolicyDocument)](#resource_policy_document-1)
- **Required:** No

Creates or updates a resource policy for the specified log group that allows other services to put log events to this account. A LogGroup can have 1 resource policy.

### `retention_in_days`

- **Type:** `aws.logs.LogGroup.RetentionInDays (custom validator)`; CFN allowed values: \[1,3,5,7,14,30,60,90,120,150,180,365,400,545,731,1096,1827,2192,2557,2922,3288,3653\]
- **Required:** No

The number of days to retain the log events in the specified log group. Possible values are: 1, 3, 5, 7, 14, 30, 60, 90, 120, 150, 180, 365, 400, 545, 731, 1096, 1827, 2192, 2557, 2922, 3288, and 3653. To set a log group so that its log events do not expire, do not specify this property.

### `tags`

- **Type:** Map\<`String (len: 1..=128)`, `String (len: unbounded..=256)`\>
- **Required:** No

An array of key-value pairs to apply to the log group. For more information, see [Tag](https://docs.aws.amazon.com/AWSCloudFormation/latest/UserGuide/aws-properties-resource-tags.html).

## Enum Values

### data_protection_policy.version (Version)

| Value | DSL Identifier |
|-------|----------------|
| `2021-06-01` | `aws.logs.LogGroup.DataProtectionPolicy.Version.v2021_06_01` |

Shorthand formats: `v2021_06_01` or `Version.v2021_06_01`

### field_index_policies.fields_v2.\<value\>.type (Type)

| Value | DSL Identifier |
|-------|----------------|
| `FIELD_INDEX` | `aws.logs.LogGroup.FieldIndexPolicy.FieldIndexConfiguration.Type.field_index` |
| `FACET` | `aws.logs.LogGroup.FieldIndexPolicy.FieldIndexConfiguration.Type.facet` |

Shorthand formats: `field_index` or `Type.field_index`

### log_group_class (LogGroupClass)

| Value | DSL Identifier |
|-------|----------------|
| `STANDARD` | `aws.logs.LogGroup.LogGroupClass.standard` |
| `INFREQUENT_ACCESS` | `aws.logs.LogGroup.LogGroupClass.infrequent_access` |
| `DELIVERY` | `aws.logs.LogGroup.LogGroupClass.delivery` |

Shorthand formats: `standard` or `LogGroupClass.standard`

### resource_policy_document.version (Version)

| Value | DSL Identifier |
|-------|----------------|
| `2012-10-17` | `aws.iam.PolicyDocument.Version.2012_10_17` |
| `2008-10-17` | `aws.iam.PolicyDocument.Version.2008_10_17` |

Use the fully qualified DSL identifiers above; bare numeric values parse as integers.

### statement.condition.\<key\> (ConditionOperator)

| Value | DSL Identifier |
|-------|----------------|
| `string_equals` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.string_equals` |
| `string_equals_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.string_equals_if_exists` |
| `for_all_values_string_equals` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_all_values_string_equals` |
| `for_all_values_string_equals_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_all_values_string_equals_if_exists` |
| `for_any_value_string_equals` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_any_value_string_equals` |
| `for_any_value_string_equals_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_any_value_string_equals_if_exists` |
| `string_not_equals` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.string_not_equals` |
| `string_not_equals_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.string_not_equals_if_exists` |
| `for_all_values_string_not_equals` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_all_values_string_not_equals` |
| `for_all_values_string_not_equals_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_all_values_string_not_equals_if_exists` |
| `for_any_value_string_not_equals` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_any_value_string_not_equals` |
| `for_any_value_string_not_equals_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_any_value_string_not_equals_if_exists` |
| `string_equals_ignore_case` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.string_equals_ignore_case` |
| `string_equals_ignore_case_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.string_equals_ignore_case_if_exists` |
| `for_all_values_string_equals_ignore_case` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_all_values_string_equals_ignore_case` |
| `for_all_values_string_equals_ignore_case_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_all_values_string_equals_ignore_case_if_exists` |
| `for_any_value_string_equals_ignore_case` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_any_value_string_equals_ignore_case` |
| `for_any_value_string_equals_ignore_case_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_any_value_string_equals_ignore_case_if_exists` |
| `string_not_equals_ignore_case` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.string_not_equals_ignore_case` |
| `string_not_equals_ignore_case_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.string_not_equals_ignore_case_if_exists` |
| `for_all_values_string_not_equals_ignore_case` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_all_values_string_not_equals_ignore_case` |
| `for_all_values_string_not_equals_ignore_case_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_all_values_string_not_equals_ignore_case_if_exists` |
| `for_any_value_string_not_equals_ignore_case` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_any_value_string_not_equals_ignore_case` |
| `for_any_value_string_not_equals_ignore_case_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_any_value_string_not_equals_ignore_case_if_exists` |
| `string_like` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.string_like` |
| `string_like_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.string_like_if_exists` |
| `for_all_values_string_like` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_all_values_string_like` |
| `for_all_values_string_like_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_all_values_string_like_if_exists` |
| `for_any_value_string_like` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_any_value_string_like` |
| `for_any_value_string_like_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_any_value_string_like_if_exists` |
| `string_not_like` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.string_not_like` |
| `string_not_like_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.string_not_like_if_exists` |
| `for_all_values_string_not_like` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_all_values_string_not_like` |
| `for_all_values_string_not_like_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_all_values_string_not_like_if_exists` |
| `for_any_value_string_not_like` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_any_value_string_not_like` |
| `for_any_value_string_not_like_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_any_value_string_not_like_if_exists` |
| `numeric_equals` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.numeric_equals` |
| `numeric_equals_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.numeric_equals_if_exists` |
| `for_all_values_numeric_equals` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_all_values_numeric_equals` |
| `for_all_values_numeric_equals_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_all_values_numeric_equals_if_exists` |
| `for_any_value_numeric_equals` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_any_value_numeric_equals` |
| `for_any_value_numeric_equals_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_any_value_numeric_equals_if_exists` |
| `numeric_not_equals` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.numeric_not_equals` |
| `numeric_not_equals_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.numeric_not_equals_if_exists` |
| `for_all_values_numeric_not_equals` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_all_values_numeric_not_equals` |
| `for_all_values_numeric_not_equals_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_all_values_numeric_not_equals_if_exists` |
| `for_any_value_numeric_not_equals` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_any_value_numeric_not_equals` |
| `for_any_value_numeric_not_equals_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_any_value_numeric_not_equals_if_exists` |
| `numeric_less_than` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.numeric_less_than` |
| `numeric_less_than_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.numeric_less_than_if_exists` |
| `for_all_values_numeric_less_than` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_all_values_numeric_less_than` |
| `for_all_values_numeric_less_than_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_all_values_numeric_less_than_if_exists` |
| `for_any_value_numeric_less_than` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_any_value_numeric_less_than` |
| `for_any_value_numeric_less_than_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_any_value_numeric_less_than_if_exists` |
| `numeric_less_than_equals` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.numeric_less_than_equals` |
| `numeric_less_than_equals_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.numeric_less_than_equals_if_exists` |
| `for_all_values_numeric_less_than_equals` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_all_values_numeric_less_than_equals` |
| `for_all_values_numeric_less_than_equals_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_all_values_numeric_less_than_equals_if_exists` |
| `for_any_value_numeric_less_than_equals` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_any_value_numeric_less_than_equals` |
| `for_any_value_numeric_less_than_equals_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_any_value_numeric_less_than_equals_if_exists` |
| `numeric_greater_than` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.numeric_greater_than` |
| `numeric_greater_than_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.numeric_greater_than_if_exists` |
| `for_all_values_numeric_greater_than` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_all_values_numeric_greater_than` |
| `for_all_values_numeric_greater_than_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_all_values_numeric_greater_than_if_exists` |
| `for_any_value_numeric_greater_than` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_any_value_numeric_greater_than` |
| `for_any_value_numeric_greater_than_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_any_value_numeric_greater_than_if_exists` |
| `numeric_greater_than_equals` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.numeric_greater_than_equals` |
| `numeric_greater_than_equals_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.numeric_greater_than_equals_if_exists` |
| `for_all_values_numeric_greater_than_equals` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_all_values_numeric_greater_than_equals` |
| `for_all_values_numeric_greater_than_equals_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_all_values_numeric_greater_than_equals_if_exists` |
| `for_any_value_numeric_greater_than_equals` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_any_value_numeric_greater_than_equals` |
| `for_any_value_numeric_greater_than_equals_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_any_value_numeric_greater_than_equals_if_exists` |
| `date_equals` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.date_equals` |
| `date_equals_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.date_equals_if_exists` |
| `for_all_values_date_equals` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_all_values_date_equals` |
| `for_all_values_date_equals_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_all_values_date_equals_if_exists` |
| `for_any_value_date_equals` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_any_value_date_equals` |
| `for_any_value_date_equals_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_any_value_date_equals_if_exists` |
| `date_not_equals` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.date_not_equals` |
| `date_not_equals_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.date_not_equals_if_exists` |
| `for_all_values_date_not_equals` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_all_values_date_not_equals` |
| `for_all_values_date_not_equals_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_all_values_date_not_equals_if_exists` |
| `for_any_value_date_not_equals` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_any_value_date_not_equals` |
| `for_any_value_date_not_equals_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_any_value_date_not_equals_if_exists` |
| `date_less_than` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.date_less_than` |
| `date_less_than_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.date_less_than_if_exists` |
| `for_all_values_date_less_than` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_all_values_date_less_than` |
| `for_all_values_date_less_than_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_all_values_date_less_than_if_exists` |
| `for_any_value_date_less_than` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_any_value_date_less_than` |
| `for_any_value_date_less_than_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_any_value_date_less_than_if_exists` |
| `date_less_than_equals` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.date_less_than_equals` |
| `date_less_than_equals_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.date_less_than_equals_if_exists` |
| `for_all_values_date_less_than_equals` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_all_values_date_less_than_equals` |
| `for_all_values_date_less_than_equals_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_all_values_date_less_than_equals_if_exists` |
| `for_any_value_date_less_than_equals` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_any_value_date_less_than_equals` |
| `for_any_value_date_less_than_equals_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_any_value_date_less_than_equals_if_exists` |
| `date_greater_than` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.date_greater_than` |
| `date_greater_than_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.date_greater_than_if_exists` |
| `for_all_values_date_greater_than` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_all_values_date_greater_than` |
| `for_all_values_date_greater_than_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_all_values_date_greater_than_if_exists` |
| `for_any_value_date_greater_than` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_any_value_date_greater_than` |
| `for_any_value_date_greater_than_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_any_value_date_greater_than_if_exists` |
| `date_greater_than_equals` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.date_greater_than_equals` |
| `date_greater_than_equals_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.date_greater_than_equals_if_exists` |
| `for_all_values_date_greater_than_equals` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_all_values_date_greater_than_equals` |
| `for_all_values_date_greater_than_equals_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_all_values_date_greater_than_equals_if_exists` |
| `for_any_value_date_greater_than_equals` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_any_value_date_greater_than_equals` |
| `for_any_value_date_greater_than_equals_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_any_value_date_greater_than_equals_if_exists` |
| `bool` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.bool` |
| `bool_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.bool_if_exists` |
| `for_all_values_bool` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_all_values_bool` |
| `for_all_values_bool_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_all_values_bool_if_exists` |
| `for_any_value_bool` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_any_value_bool` |
| `for_any_value_bool_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_any_value_bool_if_exists` |
| `binary_equals` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.binary_equals` |
| `binary_equals_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.binary_equals_if_exists` |
| `for_all_values_binary_equals` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_all_values_binary_equals` |
| `for_all_values_binary_equals_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_all_values_binary_equals_if_exists` |
| `for_any_value_binary_equals` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_any_value_binary_equals` |
| `for_any_value_binary_equals_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_any_value_binary_equals_if_exists` |
| `ip_address` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.ip_address` |
| `ip_address_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.ip_address_if_exists` |
| `for_all_values_ip_address` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_all_values_ip_address` |
| `for_all_values_ip_address_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_all_values_ip_address_if_exists` |
| `for_any_value_ip_address` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_any_value_ip_address` |
| `for_any_value_ip_address_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_any_value_ip_address_if_exists` |
| `not_ip_address` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.not_ip_address` |
| `not_ip_address_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.not_ip_address_if_exists` |
| `for_all_values_not_ip_address` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_all_values_not_ip_address` |
| `for_all_values_not_ip_address_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_all_values_not_ip_address_if_exists` |
| `for_any_value_not_ip_address` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_any_value_not_ip_address` |
| `for_any_value_not_ip_address_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_any_value_not_ip_address_if_exists` |
| `arn_equals` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.arn_equals` |
| `arn_equals_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.arn_equals_if_exists` |
| `for_all_values_arn_equals` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_all_values_arn_equals` |
| `for_all_values_arn_equals_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_all_values_arn_equals_if_exists` |
| `for_any_value_arn_equals` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_any_value_arn_equals` |
| `for_any_value_arn_equals_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_any_value_arn_equals_if_exists` |
| `arn_not_equals` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.arn_not_equals` |
| `arn_not_equals_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.arn_not_equals_if_exists` |
| `for_all_values_arn_not_equals` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_all_values_arn_not_equals` |
| `for_all_values_arn_not_equals_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_all_values_arn_not_equals_if_exists` |
| `for_any_value_arn_not_equals` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_any_value_arn_not_equals` |
| `for_any_value_arn_not_equals_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_any_value_arn_not_equals_if_exists` |
| `arn_like` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.arn_like` |
| `arn_like_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.arn_like_if_exists` |
| `for_all_values_arn_like` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_all_values_arn_like` |
| `for_all_values_arn_like_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_all_values_arn_like_if_exists` |
| `for_any_value_arn_like` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_any_value_arn_like` |
| `for_any_value_arn_like_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_any_value_arn_like_if_exists` |
| `arn_not_like` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.arn_not_like` |
| `arn_not_like_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.arn_not_like_if_exists` |
| `for_all_values_arn_not_like` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_all_values_arn_not_like` |
| `for_all_values_arn_not_like_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_all_values_arn_not_like_if_exists` |
| `for_any_value_arn_not_like` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_any_value_arn_not_like` |
| `for_any_value_arn_not_like_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_any_value_arn_not_like_if_exists` |
| `null` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.null` |
| `null_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.null_if_exists` |
| `for_all_values_null` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_all_values_null` |
| `for_all_values_null_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_all_values_null_if_exists` |
| `for_any_value_null` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_any_value_null` |
| `for_any_value_null_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_any_value_null_if_exists` |

Shorthand formats: `string_equals` or `ConditionOperator.string_equals`

### statement.effect (Effect)

| Value | DSL Identifier |
|-------|----------------|
| `Allow` | `aws.iam.PolicyDocument.Statement.Effect.allow` |
| `Deny` | `aws.iam.PolicyDocument.Statement.Effect.deny` |

Shorthand formats: `allow` or `Effect.allow`

## Struct Definitions

### data_protection_policy

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `name` | `String` | Yes | Name of the data protection policy. |
| `description` | `String` | No | Description of the data protection policy. |
| `version` | [Version](#data_protection_policyversion-version) | Yes | Data protection policy grammar version. |
| `configuration` | [Struct(DataProtectionConfiguration)](#data_protection_policyconfiguration) | No | Custom data identifiers used by policy statements. |
| `statement` | List\<[Struct(DataProtectionStatement)](#data_protection_policystatement)\> (items: 2..=2, ordered) | Yes | Data protection policy statements. |

### data_protection_policy.configuration

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `custom_data_identifier` | List\<[Struct(CustomDataIdentifier)](#data_protection_policyconfigurationcustom_data_identifier)\> (items: unbounded..=10, ordered) | No | Custom patterns recognized as sensitive data. |

### data_protection_policy.configuration.custom_data_identifier

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `name` | `String` | Yes | Name of the custom data identifier. |
| `regex` | `String (len: unbounded..=200)` | Yes | Regular expression matched by the custom data identifier. |

### data_protection_policy.statement

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `sid` | `String` | No | Optional identifier for the policy statement. |
| `data_identifier` | List\<`String`\> (ordered) | Yes | Sensitive-data identifiers matched by the statement. |
| `operation` | [Struct(DataProtectionOperation)](#data_protection_policystatementoperation) | Yes | Audit and deidentification actions performed by the statement. |

### data_protection_policy.statement.operation

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `audit` | [Struct(AuditOperation)](#data_protection_policystatementoperationaudit) | No | Audits sensitive data found in log events. |
| `deidentify` | [Struct(DeidentifyOperation)](#data_protection_policystatementoperationdeidentify) | No | Masks sensitive data found in log events. |

**Field groups:**

- At least one of: `audit`, `deidentify`.

### data_protection_policy.statement.operation.audit

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `findings_destination` | [Struct(FindingsDestination)](#data_protection_policystatementoperationauditfindings_destination) | Yes | Destinations for sensitive-data audit findings. |

### data_protection_policy.statement.operation.audit.findings_destination

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `cloud_watch_logs` | [Struct(CloudWatchLogsDestination)](#data_protection_policystatementoperationauditfindings_destinationcloud_watch_logs) | No | CloudWatch Logs audit findings destination. |
| `firehose` | [Struct(FirehoseDestination)](#data_protection_policystatementoperationauditfindings_destinationfirehose) | No | Amazon Data Firehose audit findings destination. |
| `s3` | [Struct(S3Destination)](#data_protection_policystatementoperationauditfindings_destinations3) | No | Amazon S3 audit findings destination. |

### data_protection_policy.statement.operation.audit.findings_destination.cloud_watch_logs

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `log_group` | `String` | Yes | CloudWatch Logs log group that receives audit findings. |

### data_protection_policy.statement.operation.audit.findings_destination.firehose

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `delivery_stream` | `String` | Yes | Amazon Data Firehose delivery stream that receives audit findings. |

### data_protection_policy.statement.operation.audit.findings_destination.s3

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `bucket` | `String` | Yes | Amazon S3 bucket that receives audit findings. |

### data_protection_policy.statement.operation.deidentify

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `mask_config` | [Struct(MaskConfig)](#data_protection_policystatementoperationdeidentifymask_config) | Yes | Empty marker selecting the masking operation. |

### data_protection_policy.statement.operation.deidentify.mask_config

| Field | Type | Required | Description |
|-------|------|----------|-------------|

### field_index_policies

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `fields` | List\<`String`\> (ordered) | No | Log-event fields for which CloudWatch Logs creates indexes. |
| `fields_v2` | Map\<`String`, [Struct(FieldIndexConfiguration)](#field_index_policiesfields_v2value)\> | No | Log-event fields and their index or facet types. |

**Field groups:**

- At least one of: `fields`, `fields_v2`.

### field_index_policies.fields_v2.\<value\>

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `type` | [Type](#field_index_policiesfields_v2valuetype-type) | Yes | Whether this field is an index or a facet. |

### principal

Used by: `union_of_principal_or_string.member_0`

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `service` | [Union(union\_of\_string\_or\_list\_of\_string)](#union_of_string_or_list_of_string) | No |  |
| `aws` | [Union(union\_of\_string\_or\_list\_of\_string)](#union_of_string_or_list_of_string) | No |  |
| `federated` | [Union(union\_of\_string\_or\_list\_of\_string)](#union_of_string_or_list_of_string) | No |  |
| `canonical_user` | [Union(union\_of\_string\_or\_list\_of\_string)](#union_of_string_or_list_of_string) | No |  |

**Field groups:**

- At least one of: `service`, `aws`, `federated`, `canonical_user`.

### resource_policy_document

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `version` | [Version](#resource_policy_documentversion-version) | No |  |
| `id` | `String` | No |  |
| `statement` | [Union(resource\_policy\_document.statement)](#resource_policy_documentstatement) | Yes |  |

### statement

Used by: `resource_policy_document.statement.member_0`, `resource_policy_document.statement.member_1`

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `sid` | `String` | No |  |
| `effect` | [Effect](#statementeffect-effect) | Yes |  |
| `action` | [Union(union\_of\_string\_or\_list\_of\_string)](#union_of_string_or_list_of_string) | No |  |
| `not_action` | [Union(union\_of\_string\_or\_list\_of\_string)](#union_of_string_or_list_of_string) | No |  |
| `resource` | [Union(union\_of\_string\_or\_list\_of\_string)](#union_of_string_or_list_of_string) | No |  |
| `not_resource` | [Union(union\_of\_string\_or\_list\_of\_string)](#union_of_string_or_list_of_string) | No |  |
| `principal` | [Union(union\_of\_principal\_or\_string)](#union_of_principal_or_string) | No |  |
| `not_principal` | [Union(union\_of\_principal\_or\_string)](#union_of_principal_or_string) | No |  |
| `condition` | Map\<[ConditionOperator](#statementconditionkey-conditionoperator), Map\<`String`, [Union(statement.condition.\<value\>.\<value\>)](#statementconditionvaluevalue)\>\> | No |  |

**Field groups:**

- Exactly one of: `action`, `not_action`.
- At most one of: `resource`, `not_resource`.
- At most one of: `principal`, `not_principal`.

## Union Definitions

### resource_policy_document.statement

| Member | Type |
|--------|------|
| `member_0` | List\<[Struct(Statement)](#statement)\> (items: 1..unbounded, ordered) |
| `member_1` | [Struct(Statement)](#statement) |

### statement.condition.\<value\>.\<value\>

| Member | Type |
|--------|------|
| `member_0` | [Union(union\_of\_string\_or\_list\_of\_string)](#union_of_string_or_list_of_string) |
| `member_1` | `Int` |
| `member_2` | `Float` |
| `member_3` | `Bool` |
| `member_4` | List\<[Union(union\_of\_string\_or\_int\_or\_float\_or\_bool)](#union_of_string_or_int_or_float_or_bool)\> (items: 1..unbounded, unordered) |

### union_of_principal_or_string

Used by: `statement.not_principal`, `statement.principal`

| Member | Type |
|--------|------|
| `member_0` | [Struct(Principal)](#principal) |
| `member_1` | `String (pattern: /^\*\z/ (Full))` |

### union_of_string_or_int_or_float_or_bool

Used by: `statement.condition.<value>.<value>.member_4`

| Member | Type |
|--------|------|
| `member_0` | `String` |
| `member_1` | `Int` |
| `member_2` | `Float` |
| `member_3` | `Bool` |

### union_of_string_or_list_of_string

Used by: `principal.aws`, `principal.canonical_user`, `principal.federated`, `principal.service`, `statement.action`, `statement.not_action`, `statement.not_resource`, `statement.resource`, `statement.condition.<value>.<value>.member_0`

| Member | Type |
|--------|------|
| `member_0` | `String` |
| `member_1` | List\<`String`\> (items: 1..unbounded, unordered) |

## Additional Input Constraints

| Input rule |
|------------|
| `value at kms_key_id: must match /^arn:[a-z0-9-]+:kms:[a-z0-9-]+:\d{12}:(key\|alias)/.+\z/ (Full) (on every write)` |
| `value at kms_key_id: must have length ..=256 Unicode scalar values (on every write)` |

## Attribute Reference

### `arn`

- **Type:** `aws.logs.LogGroup.Arn (pattern: /^arn:(?:aws|aws-cn|aws-us-gov|aws-iso|aws-iso-b|aws-iso-e|aws-iso-f|aws-eusc):logs:[^:]*:[^:]*:log-group:.+$/ (Full), custom validator)`



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

