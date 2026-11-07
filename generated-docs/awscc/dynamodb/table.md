---
title: "awscc.dynamodb.Table"
---


CloudFormation Type: `AWS::DynamoDB::Table`

The ``AWS::DynamoDB::Table`` resource creates a DDB table. For more information, see [CreateTable](https://docs.aws.amazon.com/amazondynamodb/latest/APIReference/API_CreateTable.html) in the *API Reference*.
 You should be aware of the following behaviors when working with DDB tables:
  +  CFNlong typically creates DDB tables in parallel. However, if your template includes multiple DDB tables with indexes, you must declare dependencies so that the tables are created sequentially. DDBlong limits the number of tables with secondary indexes that are in the creating state. If you create multiple tables with indexes at the same time, DDB returns an error and the stack operation fails. For an example, see [DynamoDB Table with a DependsOn Attribute](https://docs.aws.amazon.com/AWSCloudFormation/latest/UserGuide/aws-resource-dynamodb-table.html#aws-resource-dynamodb-table--examples--DynamoDB_Table_with_a_DependsOn_Attribute).
  
   Our guidance is to use the latest schema documented for your CFNlong templates. This schema supports the provisioning of all table settings below. When using this schema in your CFNlong templates, please ensure that your Identity and Access Management (IAM) policies are updated with appropriate permissions to allow for the authorization of these setting changes.

## Example

```crn
awscc.dynamodb.Table {
  table_name   = 'jti-store'
  billing_mode = aws.dynamodb.Table.BillingMode.pay_per_request

  attribute_definition {
    attribute_name = 'jti'
    attribute_type = s
  }

  key_schema {
    attribute_name = 'jti'
    key_type       = hash
  }

  time_to_live_specification = {
    enabled        = true
    attribute_name = 'ttl'
  }

  point_in_time_recovery_specification = {
    point_in_time_recovery_enabled = true
  }

  tags = {
    Environment = 'example'
    Workload    = 'registry'
  }
}
```

## Argument Reference

### `attribute_definitions`

- **Type:** List\<[Struct(AttributeDefinition)](#attribute_definitions-1)\> (unique items)
- **Required:** No

A list of attributes that describe the key schema for the table and indexes. This property is required to create a DDB table. Update requires: [Some interruptions](https://docs.aws.amazon.com/AWSCloudFormation/latest/UserGuide/using-cfn-updating-stacks-update-behaviors.html#update-some-interrupt). Replacement if you edit an existing AttributeDefinition.

### `billing_mode`

- **Type:** [Enum (BillingMode)](#billing_mode-billingmode)
- **Required:** No

Specify how you are charged for read and write throughput and how you manage capacity. Valid values include: + ``PAY_PER_REQUEST`` - We recommend using ``PAY_PER_REQUEST`` for most DynamoDB workloads. ``PAY_PER_REQUEST`` sets the billing mode to [On-demand capacity mode](https://docs.aws.amazon.com/amazondynamodb/latest/developerguide/on-demand-capacity-mode.html). + ``PROVISIONED`` - We recommend using ``PROVISIONED`` for steady workloads with predictable growth where capacity requirements can be reliably forecasted. ``PROVISIONED`` sets the billing mode to [Provisioned capacity mode](https://docs.aws.amazon.com/amazondynamodb/latest/developerguide/provisioned-capacity-mode.html). If not specified, the default is ``PROVISIONED``.

### `contributor_insights_specification`

- **Type:** [Struct(ContributorInsightsSpecification)](#contributor_insights_specification-1)
- **Required:** No

The settings used to specify whether to enable CloudWatch Contributor Insights for the table and define which events to monitor.

### `deletion_protection_enabled`

- **Type:** Bool
- **Required:** No

Determines if a table is protected from deletion. When enabled, the table cannot be deleted by any user or process. This setting is disabled by default. For more information, see [Using deletion protection](https://docs.aws.amazon.com/amazondynamodb/latest/developerguide/WorkingWithTables.Basics.html#WorkingWithTables.Basics.DeletionProtection) in the *Developer Guide*.

### `global_secondary_indexes`

- **Type:** List\<[Struct(GlobalSecondaryIndex)](#global_secondary_indexes-1)\>
- **Required:** No

Global secondary indexes to be created on the table. You can create up to 20 global secondary indexes. If you update a table to include a new global secondary index, CFNlong initiates the index creation and then proceeds with the stack update. CFNlong doesn't wait for the index to complete creation because the backfilling phase can take a long time, depending on the size of the table. You can't use the index or update the table until the index's status is ``ACTIVE``. You can track its status by using the DynamoDB [DescribeTable](https://docs.aws.amazon.com/cli/latest/reference/dynamodb/describe-table.html) command. If you add or delete an index during an update, we recommend that you don't update any other resources. If your stack fails to update and is rolled back while adding a new index, you must manually delete the index. Updates are not supported. The following are exceptions: + If you update either the contributor insights specification or the provisioned throughput values of global secondary indexes, you can update the table without interruption. + You can delete or add one global secondary index without interruption. If you do both in the same update (for example, by changing the index's logical ID), the update fails.

### `import_source_specification`

- **Type:** [Struct(ImportSourceSpecification)](#import_source_specification-1)
- **Required:** No
- **Create-only:** Yes
- **Write-only:** Yes

Specifies the properties of data being imported from the S3 bucket source to the" table. If you specify the ``ImportSourceSpecification`` property, and also specify either the ``StreamSpecification``, the ``TableClass`` property, the ``DeletionProtectionEnabled`` property, or the ``WarmThroughput`` property, the IAM entity creating/updating stack must have ``UpdateTable`` permission.

### `key_schema`

- **Type:** List\<[Struct(KeySchema)](#key_schema-1)\> (items: 1..=2) (unique items)
- **Required:** Yes

Specifies the attributes that make up the primary key for the table. The attributes in the ``KeySchema`` property must also be defined in the ``AttributeDefinitions`` property.

### `kinesis_stream_specification`

- **Type:** [Struct(KinesisStreamSpecification)](#kinesis_stream_specification-1)
- **Required:** No

The Kinesis Data Streams configuration for the specified table.

### `local_secondary_indexes`

- **Type:** List\<[Struct(LocalSecondaryIndex)](#local_secondary_indexes-1)\> (items: 1..=5)
- **Required:** No

Local secondary indexes to be created on the table. You can create up to 5 local secondary indexes. Each index is scoped to a given hash key value. The size of each hash key can be up to 10 gigabytes.

### `on_demand_throughput`

- **Type:** [Struct(OnDemandThroughput)](#on_demand_throughput-1)
- **Required:** No

Sets the maximum number of read and write units for the specified on-demand table. If you use this property, you must specify ``MaxReadRequestUnits``, ``MaxWriteRequestUnits``, or both.

### `point_in_time_recovery_specification`

- **Type:** [Struct(PointInTimeRecoverySpecification)](#point_in_time_recovery_specification-1)
- **Required:** No

The settings used to enable point in time recovery.

### `provisioned_throughput`

- **Type:** [Struct(ProvisionedThroughput)](#provisioned_throughput-1)
- **Required:** No

Throughput for the specified table, which consists of values for ``ReadCapacityUnits`` and ``WriteCapacityUnits``. For more information about the contents of a provisioned throughput structure, see [Amazon DynamoDB Table ProvisionedThroughput](https://docs.aws.amazon.com/amazondynamodb/latest/APIReference/API_ProvisionedThroughput.html). If you set ``BillingMode`` as ``PROVISIONED``, you must specify this property. If you set ``BillingMode`` as ``PAY_PER_REQUEST``, you cannot specify this property.

### `resource_policy`

- **Type:** [Struct(ResourcePolicy)](#resource_policy-1)
- **Required:** No

An AWS resource-based policy document in JSON format that will be attached to the table. When you attach a resource-based policy while creating a table, the policy application is *strongly consistent*. The maximum size supported for a resource-based policy document is 20 KB. DynamoDB counts whitespaces when calculating the size of a policy against this limit. For a full list of all considerations that apply for resource-based policies, see [Resource-based policy considerations](https://docs.aws.amazon.com/amazondynamodb/latest/developerguide/rbac-considerations.html). You need to specify the ``CreateTable`` and ``PutResourcePolicy`` IAM actions for authorizing a user to create a table with a resource-based policy.

### `sse_specification`

- **Type:** [Struct(SSESpecification)](#sse_specification-1)
- **Required:** No

Specifies the settings to enable server-side encryption.

### `stream_specification`

- **Type:** [Struct(StreamSpecification)](#stream_specification-1)
- **Required:** No

The settings for the DDB table stream, which captures changes to items stored in the table. Including this property in your CFNlong template automatically enables streaming.

### `table_class`

- **Type:** [Enum (TableClass)](#table_class-tableclass)
- **Required:** No

The table class of the new table. Valid values are ``STANDARD`` and ``STANDARD_INFREQUENT_ACCESS``.

### `table_name`

- **Type:** `String(pattern, len: 3..=255)`
- **Required:** No
- **Create-only:** Yes

A name for the table. If you don't specify a name, CFNlong generates a unique physical ID and uses that ID for the table name. For more information, see [Name Type](https://docs.aws.amazon.com/AWSCloudFormation/latest/UserGuide/aws-properties-name.html). If you specify a name, you cannot perform updates that require replacement of this resource. You can perform updates that require no or some interruption. If you must replace the resource, specify a new name.

### `tags`

- **Type:** `Map<String, String>`
- **Required:** No

An array of key-value pairs to apply to this resource. For more information, see [Tag](https://docs.aws.amazon.com/AWSCloudFormation/latest/UserGuide/aws-properties-resource-tags.html).

### `time_to_live_specification`

- **Type:** [Struct(TimeToLiveSpecification)](#time_to_live_specification-1)
- **Required:** No

Specifies the Time to Live (TTL) settings for the table. For detailed information about the limits in DynamoDB, see [Limits in Amazon DynamoDB](https://docs.aws.amazon.com/amazondynamodb/latest/developerguide/Limits.html) in the Amazon DynamoDB Developer Guide.

### `warm_throughput`

- **Type:** [Struct(WarmThroughput)](#warm_throughput-1)
- **Required:** No

Represents the warm throughput (in read units per second and write units per second) for creating a table.

## Enum Values

### attribute_definitions.attribute_type (AttributeType)

| Value | DSL Identifier |
|-------|----------------|
| `S` | `aws.dynamodb.Table.AttributeDefinition.AttributeType.s` |
| `N` | `aws.dynamodb.Table.AttributeDefinition.AttributeType.n` |
| `B` | `aws.dynamodb.Table.AttributeDefinition.AttributeType.b` |

Shorthand formats: `s` or `AttributeType.s`

### billing_mode (BillingMode)

| Value | DSL Identifier |
|-------|----------------|
| `PAY_PER_REQUEST` | `aws.dynamodb.Table.BillingMode.pay_per_request` |
| `PROVISIONED` | `aws.dynamodb.Table.BillingMode.provisioned` |

Shorthand formats: `pay_per_request` or `BillingMode.pay_per_request`

### contributor_insights_specification.mode (Mode)

| Value | DSL Identifier |
|-------|----------------|
| `ACCESSED_AND_THROTTLED_KEYS` | `aws.dynamodb.Table.ContributorInsightsSpecification.Mode.accessed_and_throttled_keys` |
| `THROTTLED_KEYS` | `aws.dynamodb.Table.ContributorInsightsSpecification.Mode.throttled_keys` |

Shorthand formats: `accessed_and_throttled_keys` or `Mode.accessed_and_throttled_keys`

### import_source_specification.input_compression_type (InputCompressionType)

| Value | DSL Identifier |
|-------|----------------|
| `GZIP` | `aws.dynamodb.Table.ImportSourceSpecification.InputCompressionType.gzip` |
| `ZSTD` | `aws.dynamodb.Table.ImportSourceSpecification.InputCompressionType.zstd` |
| `NONE` | `aws.dynamodb.Table.ImportSourceSpecification.InputCompressionType.none` |

Shorthand formats: `gzip` or `InputCompressionType.gzip`

### import_source_specification.input_format (InputFormat)

| Value | DSL Identifier |
|-------|----------------|
| `CSV` | `aws.dynamodb.Table.ImportSourceSpecification.InputFormat.csv` |
| `DYNAMODB_JSON` | `aws.dynamodb.Table.ImportSourceSpecification.InputFormat.dynamodb_json` |
| `ION` | `aws.dynamodb.Table.ImportSourceSpecification.InputFormat.ion` |

Shorthand formats: `csv` or `InputFormat.csv`

### key_schema.key_type (KeyType)

| Value | DSL Identifier |
|-------|----------------|
| `HASH` | `aws.dynamodb.Table.GlobalSecondaryIndex.KeySchema.KeyType.hash` |
| `RANGE` | `aws.dynamodb.Table.GlobalSecondaryIndex.KeySchema.KeyType.range` |

Shorthand formats: `hash` or `KeyType.hash`

### kinesis_stream_specification.approximate_creation_date_time_precision (ApproximateCreationDateTimePrecision)

| Value | DSL Identifier |
|-------|----------------|
| `MICROSECOND` | `aws.dynamodb.Table.KinesisStreamSpecification.ApproximateCreationDateTimePrecision.microsecond` |
| `MILLISECOND` | `aws.dynamodb.Table.KinesisStreamSpecification.ApproximateCreationDateTimePrecision.millisecond` |

Shorthand formats: `microsecond` or `ApproximateCreationDateTimePrecision.microsecond`

### projection.projection_type (ProjectionType)

| Value | DSL Identifier |
|-------|----------------|
| `KEYS_ONLY` | `aws.dynamodb.Table.GlobalSecondaryIndex.Projection.ProjectionType.keys_only` |
| `INCLUDE` | `aws.dynamodb.Table.GlobalSecondaryIndex.Projection.ProjectionType.include` |
| `ALL` | `aws.dynamodb.Table.GlobalSecondaryIndex.Projection.ProjectionType.all` |

Shorthand formats: `keys_only` or `ProjectionType.keys_only`

### sse_specification.sse_type (SseType)

| Value | DSL Identifier |
|-------|----------------|
| `AES256` | `aws.dynamodb.Table.SseSpecification.SseType.aes256` |
| `KMS` | `aws.dynamodb.Table.SseSpecification.SseType.kms` |

Shorthand formats: `aes256` or `SseType.aes256`

### stream_specification.stream_view_type (StreamViewType)

| Value | DSL Identifier |
|-------|----------------|
| `KEYS_ONLY` | `aws.dynamodb.Table.StreamSpecification.StreamViewType.keys_only` |
| `NEW_IMAGE` | `aws.dynamodb.Table.StreamSpecification.StreamViewType.new_image` |
| `OLD_IMAGE` | `aws.dynamodb.Table.StreamSpecification.StreamViewType.old_image` |
| `NEW_AND_OLD_IMAGES` | `aws.dynamodb.Table.StreamSpecification.StreamViewType.new_and_old_images` |

Shorthand formats: `keys_only` or `StreamViewType.keys_only`

### table_class (TableClass)

| Value | DSL Identifier |
|-------|----------------|
| `STANDARD` | `aws.dynamodb.Table.TableClass.standard` |
| `STANDARD_INFREQUENT_ACCESS` | `aws.dynamodb.Table.TableClass.standard_infrequent_access` |

Shorthand formats: `standard` or `TableClass.standard`

## Struct Definitions

### attribute_definitions

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `attribute_name` | `String(len: 1..=255)` | Yes | A name for the attribute. |
| `attribute_type` | [Enum (AttributeType)](#attribute_definitionsattribute_type-attributetype) | Yes | The data type for the attribute, where: + ``S`` - the attribute is of type String + ``N`` - the attribute is of type Number + ``B`` - the attribute is of type Binary |

### contributor_insights_specification

Used by: `contributor_insights_specification`, `global_secondary_indexes.contributor_insights_specification`

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `enabled` | Bool | Yes | Indicates whether CloudWatch Contributor Insights are to be enabled (true) or disabled (false). |
| `mode` | [Enum (Mode)](#contributor_insights_specificationmode-mode) | No | Specifies the CloudWatch Contributor Insights mode for a table. Valid values are ``ACCESSED_AND_THROTTLED_KEYS`` (tracks all access and throttled events) or ``THROTTLED_KEYS`` (tracks only throttled events). This setting determines what type of contributor insights data is collected for the table. |

### global_secondary_indexes

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `contributor_insights_specification` | [Struct(ContributorInsightsSpecification)](#contributor_insights_specification-1) | No | The settings used to specify whether to enable CloudWatch Contributor Insights for the global table and define which events to monitor. |
| `index_name` | `String(pattern, len: 3..=255)` | Yes | The name of the global secondary index. The name must be unique among all other indexes on this table. |
| `key_schema` | List\<[Struct(KeySchema)](#key_schema-1)\> (items: 1..) (unique items) | Yes | The complete key schema for a global secondary index, which consists of one or more pairs of attribute names and key types: + ``HASH`` - partition key + ``RANGE`` - sort key The partition key of an item is also known as its *hash attribute*. The term "hash attribute" derives from DynamoDB's usage of an internal hash function to evenly distribute data items across partitions, based on their partition key values. The sort key of an item is also known as its *range attribute*. The term "range attribute" derives from the way DynamoDB stores items with the same partition key physically close together, in sorted order by the sort key value. |
| `on_demand_throughput` | [Struct(OnDemandThroughput)](#on_demand_throughput-1) | No | The maximum number of read and write units for the specified global secondary index. If you use this parameter, you must specify ``MaxReadRequestUnits``, ``MaxWriteRequestUnits``, or both. You must use either ``OnDemandThroughput`` or ``ProvisionedThroughput`` based on your table's capacity mode. |
| `projection` | [Struct(Projection)](#projection) | Yes | Represents attributes that are copied (projected) from the table into the global secondary index. These are in addition to the primary key attributes and index key attributes, which are automatically projected. |
| `provisioned_throughput` | [Struct(ProvisionedThroughput)](#provisioned_throughput-1) | No | Represents the provisioned throughput settings for the specified global secondary index. You must use either ``OnDemandThroughput`` or ``ProvisionedThroughput`` based on your table's capacity mode. For current minimum and maximum provisioned throughput values, see [Service, Account, and Table Quotas](https://docs.aws.amazon.com/amazondynamodb/latest/developerguide/Limits.html) in the *Amazon DynamoDB Developer Guide*. |
| `warm_throughput` | [Struct(WarmThroughput)](#warm_throughput-1) | No | Represents the warm throughput value (in read units per second and write units per second) for the specified secondary index. If you use this parameter, you must specify ``ReadUnitsPerSecond``, ``WriteUnitsPerSecond``, or both. |

### import_source_specification

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `input_compression_type` | [Enum (InputCompressionType)](#import_source_specificationinput_compression_type-inputcompressiontype) | No | Type of compression to be used on the input coming from the imported table. |
| `input_format` | [Enum (InputFormat)](#import_source_specificationinput_format-inputformat) | Yes | The format of the source data. Valid values for ``ImportFormat`` are ``CSV``, ``DYNAMODB_JSON`` or ``ION``. |
| `input_format_options` | [Struct(InputFormatOptions)](#import_source_specificationinput_format_options) | No | Additional properties that specify how the input is formatted, |
| `s3_bucket_source` | [Struct(S3BucketSource)](#import_source_specifications3_bucket_source) | Yes | The S3 bucket that provides the source for the import. |

### import_source_specification.input_format_options

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `csv` | [Struct(Csv)](#import_source_specificationinput_format_optionscsv) | No | The options for imported source files in CSV format. The values are Delimiter and HeaderList. |

### import_source_specification.input_format_options.csv

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `delimiter` | `String(pattern, len: 1..=1)` | No | The delimiter used for separating items in the CSV file being imported. |
| `header_list` | `List<String(pattern, len: 1..=65536)>` (items: 1..=255) (unique items) | No | List of the headers used to specify a common header for all source CSV files being imported. If this field is specified then the first line of each CSV file is treated as data instead of the header. If this field is not specified the the first line of each CSV file is treated as the header. |

### import_source_specification.s3_bucket_source

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `s3_bucket` | `String(pattern, len: ..=255)` | Yes | The S3 bucket that is being imported from. |
| `s3_bucket_owner` | `aws.AccountId` | No | The account number of the S3 bucket that is being imported from. If the bucket is owned by the requester this is optional. |
| `s3_key_prefix` | `String(len: ..=1024)` | No | The key prefix shared by all S3 Objects that are being imported. |

### key_schema

Used by: `global_secondary_indexes.key_schema`, `key_schema`, `local_secondary_indexes.key_schema`

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `attribute_name` | `String(len: 1..=255)` | Yes | The name of a key attribute. |
| `key_type` | [Enum (KeyType)](#key_schemakey_type-keytype) | Yes | The role that this key attribute will assume: + ``HASH`` - partition key + ``RANGE`` - sort key The partition key of an item is also known as its *hash attribute*. The term "hash attribute" derives from DynamoDB's usage of an internal hash function to evenly distribute data items across partitions, based on their partition key values. The sort key of an item is also known as its *range attribute*. The term "range attribute" derives from the way DynamoDB stores items with the same partition key physically close together, in sorted order by the sort key value. |

### kinesis_stream_specification

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `approximate_creation_date_time_precision` | [Enum (ApproximateCreationDateTimePrecision)](#kinesis_stream_specificationapproximate_creation_date_time_precision-approximatecreationdatetimeprecision) | No | The precision for the time and date that the stream was created. |
| `stream_arn` | `aws.Arn` | Yes | The ARN for a specific Kinesis data stream. Length Constraints: Minimum length of 37. Maximum length of 1024. |

### local_secondary_indexes

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `index_name` | `String(pattern, len: 3..=255)` | Yes | The name of the local secondary index. The name must be unique among all other indexes on this table. |
| `key_schema` | List\<[Struct(KeySchema)](#key_schema-1)\> (items: 1..) (unique items) | Yes | The complete key schema for the local secondary index, consisting of one or more pairs of attribute names and key types: + ``HASH`` - partition key + ``RANGE`` - sort key The partition key of an item is also known as its *hash attribute*. The term "hash attribute" derives from DynamoDB's usage of an internal hash function to evenly distribute data items across partitions, based on their partition key values. The sort key of an item is also known as its *range attribute*. The term "range attribute" derives from the way DynamoDB stores items with the same partition key physically close together, in sorted order by the sort key value. |
| `projection` | [Struct(Projection)](#projection) | Yes | Represents attributes that are copied (projected) from the table into the local secondary index. These are in addition to the primary key attributes and index key attributes, which are automatically projected. |

### on_demand_throughput

Used by: `global_secondary_indexes.on_demand_throughput`, `on_demand_throughput`

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `max_read_request_units` | Int(1..) | No | Maximum number of read request units for the specified table. To specify a maximum ``OnDemandThroughput`` on your table, set the value of ``MaxReadRequestUnits`` as greater than or equal to 1. To remove the maximum ``OnDemandThroughput`` that is currently set on your table, set the value of ``MaxReadRequestUnits`` to -1. |
| `max_write_request_units` | Int(1..) | No | Maximum number of write request units for the specified table. To specify a maximum ``OnDemandThroughput`` on your table, set the value of ``MaxWriteRequestUnits`` as greater than or equal to 1. To remove the maximum ``OnDemandThroughput`` that is currently set on your table, set the value of ``MaxWriteRequestUnits`` to -1. |

### point_in_time_recovery_specification

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `point_in_time_recovery_enabled` | Bool | No | Indicates whether point in time recovery is enabled (true) or disabled (false) on the table. |
| `recovery_period_in_days` | Int(1..=35) | No | The number of preceding days for which continuous backups are taken and maintained. Your table data is only recoverable to any point-in-time from within the configured recovery period. This parameter is optional. If no value is provided, the value will default to 35. |

`recovery_period_in_days` requires `point_in_time_recovery_enabled`.

### projection

Used by: `global_secondary_indexes.projection`, `local_secondary_indexes.projection`

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `non_key_attributes` | `List<String(len: 1..=255)>` (items: 1..=20) | No | Represents the non-key attribute names which will be projected into the index. For global and local secondary indexes, the total count of ``NonKeyAttributes`` summed across all of the secondary indexes, must not exceed 100. If you project the same attribute into two different indexes, this counts as two distinct attributes when determining the total. This limit only applies when you specify the ProjectionType of ``INCLUDE``. You still can specify the ProjectionType of ``ALL`` to project all attributes from the source table, even if the table has more than 100 attributes. |
| `projection_type` | [Enum (ProjectionType)](#projectionprojection_type-projectiontype) | No | The set of attributes that are projected into the index: + ``KEYS_ONLY`` - Only the index and primary keys are projected into the index. + ``INCLUDE`` - In addition to the attributes described in ``KEYS_ONLY``, the secondary index will include other non-key attributes that you specify. + ``ALL`` - All of the table attributes are projected into the index. When using the DynamoDB console, ``ALL`` is selected by default. |

### provisioned_throughput

Used by: `global_secondary_indexes.provisioned_throughput`, `provisioned_throughput`

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `read_capacity_units` | `Int(1..)` | Yes | The maximum number of strongly consistent reads consumed per second before DynamoDB returns a ``ThrottlingException``. For more information, see [Specifying Read and Write Requirements](https://docs.aws.amazon.com/amazondynamodb/latest/developerguide/ProvisionedThroughput.html) in the *Amazon DynamoDB Developer Guide*. If read/write capacity mode is ``PAY_PER_REQUEST`` the value is set to 0. |
| `write_capacity_units` | `Int(1..)` | Yes | The maximum number of writes consumed per second before DynamoDB returns a ``ThrottlingException``. For more information, see [Specifying Read and Write Requirements](https://docs.aws.amazon.com/amazondynamodb/latest/developerguide/ProvisionedThroughput.html) in the *Amazon DynamoDB Developer Guide*. If read/write capacity mode is ``PAY_PER_REQUEST`` the value is set to 0. |

### resource_policy

Used by: `resource_policy`, `stream_specification.resource_policy`

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `policy_document` | PolicyDocument | Yes | A resource-based policy document that contains permissions to add to the specified DDB table, index, or both. In a CFNshort template, you can provide the policy in JSON or YAML format because CFNshort converts YAML to JSON before submitting it to DDB. For more information about resource-based policies, see [Using resource-based policies for](https://docs.aws.amazon.com/amazondynamodb/latest/developerguide/access-control-resource-based.html) and [Resource-based policy examples](https://docs.aws.amazon.com/amazondynamodb/latest/developerguide/rbac-examples.html). |

### sse_specification

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `kms_master_key_id` | aws.kms.Key.Reference | No | The KMS key that should be used for the KMS encryption. To specify a key, use its key ID, Amazon Resource Name (ARN), alias name, or alias ARN. Note that you should only provide this parameter if the key is different from the default DynamoDB key ``alias/aws/dynamodb``. |
| `sse_enabled` | Bool | Yes | Indicates whether server-side encryption is done using an AWS managed key or an AWS owned key. If enabled (true), server-side encryption type is set to ``KMS`` and an AWS managed key is used (KMS charges apply). If disabled (false) or not specified, server-side encryption is set to AWS owned key. |
| `sse_type` | [Enum (SseType)](#sse_specificationsse_type-ssetype) | No | Server-side encryption type. The only supported value is: + ``KMS`` - Server-side encryption that uses KMSlong. The key is stored in your account and is managed by KMS (KMS charges apply). |

### stream_specification

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `resource_policy` | [Struct(ResourcePolicy)](#resource_policy-1) | No | Creates or updates a resource-based policy document that contains the permissions for DDB resources, such as a table's streams. Resource-based policies let you define access permissions by specifying who has access to each resource, and the actions they are allowed to perform on each resource. When you remove the ``StreamSpecification`` property from the template, DynamoDB disables the stream but retains any attached resource policy until the stream is deleted after 24 hours. When you modify the ``StreamViewType`` property, DynamoDB creates a new stream and retains the old stream's resource policy. The old stream and its resource policy are deleted after the 24-hour retention period. In a CFNshort template, you can provide the policy in JSON or YAML format because CFNshort converts YAML to JSON before submitting it to DDB. For more information about resource-based policies, see [Using resource-based policies for](https://docs.aws.amazon.com/amazondynamodb/latest/developerguide/access-control-resource-based.html) and [Resource-based policy examples](https://docs.aws.amazon.com/amazondynamodb/latest/developerguide/rbac-examples.html). |
| `stream_view_type` | [Enum (StreamViewType)](#stream_specificationstream_view_type-streamviewtype) | Yes | When an item in the table is modified, ``StreamViewType`` determines what information is written to the stream for this table. Valid values for ``StreamViewType`` are: + ``KEYS_ONLY`` - Only the key attributes of the modified item are written to the stream. + ``NEW_IMAGE`` - The entire item, as it appears after it was modified, is written to the stream. + ``OLD_IMAGE`` - The entire item, as it appeared before it was modified, is written to the stream. + ``NEW_AND_OLD_IMAGES`` - Both the new and the old item images of the item are written to the stream. |

### time_to_live_specification

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `attribute_name` | `String(len: 1..=255)` | No | The name of the TTL attribute used to store the expiration time for items in the table. + The ``AttributeName`` property is required when enabling the TTL, or when TTL is already enabled. + To update this property, you must first disable TTL and then enable TTL with the new attribute name. |
| `enabled` | Bool | Yes | Indicates whether TTL is to be enabled (true) or disabled (false) on the table. |

### warm_throughput

Used by: `global_secondary_indexes.warm_throughput`, `warm_throughput`

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `read_units_per_second` | Int(1..) | No | Represents the number of read operations your base table can instantaneously support. |
| `write_units_per_second` | Int(1..) | No | Represents the number of write operations your base table can instantaneously support. |

At least one of `read_units_per_second`, `write_units_per_second` must be specified.

## Attribute Reference

### `arn`

- **Type:** `aws.dynamodb.Table.Arn`



### `stream_arn`

- **Type:** `aws.Arn`



## AWS API narrowings

These fields are narrower than or equal to CFN. Input rules combine documented AWS domains, recorded live observations, and Carina canonical spellings for stable state comparison. A canonical DSL spelling does not claim AWS rejects other spellings. Attaching a type name adds no structural constraints. Field names below use CFN spelling.

| CFN field | Relation to CFN | Evidence URL | Verbatim quote / exact observation | Checked / observed |
| --- | --- | --- | --- | --- |
| `/definitions/AttributeDefinition/properties/AttributeName` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/amazondynamodb/latest/APIReference/API_AttributeDefinition.html#DDB-Type-AttributeDefinition-AttributeName) | ` A name for the attribute. `<br>` Length Constraints: Minimum length of 1. Maximum length of 255. ` | 2026-11-07 |
| `/definitions/Csv/properties/Delimiter` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/amazondynamodb/latest/APIReference/API_CsvOptions.html#DDB-Type-CsvOptions-Delimiter) | ` The delimiter used for separating items in the CSV file being imported. `<br>` Length Constraints: Fixed length of 1. `<br>` Pattern: [,;:\|\t ] ` | 2026-11-07 |
| `/definitions/Csv/properties/HeaderList` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/amazondynamodb/latest/APIReference/API_CsvOptions.html#DDB-Type-CsvOptions-HeaderList) | ` List of the headers used to specify a common header for all source CSV files being imported. If this field is specified then the first line of each CSV file is treated as data instead of the header. If this field is not specified the the first line of each CSV file is treated as the header. `<br>` Array Members: Minimum number of 1 item. Maximum number of 255 items. `<br>` Length Constraints: Minimum length of 1. Maximum length of 65536. `<br>` Pattern: [\x20-\x21\x23-\x2B\x2D-\x7E]* ` | 2026-11-07 |
| `/definitions/GlobalSecondaryIndex/properties/IndexName` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/amazondynamodb/latest/APIReference/API_GlobalSecondaryIndex.html#DDB-Type-GlobalSecondaryIndex-IndexName) | ` The name of the global secondary index. The name must be unique among all other indexes on this table. `<br>` Length Constraints: Minimum length of 3. Maximum length of 255. `<br>` Pattern: [a-zA-Z0-9_.-]+ ` | 2026-11-07 |
| `/definitions/GlobalSecondaryIndex/properties/KeySchema` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/amazondynamodb/latest/APIReference/API_GlobalSecondaryIndex.html#DDB-Type-GlobalSecondaryIndex-KeySchema) | ` The complete key schema for a global secondary index, which consists of one or more pairs of attribute names and key types: `<br>` HASH - partition key `<br>` RANGE - sort key `<br>` The partition key of an item is also known as its hash attribute. The term "hash attribute" derives from DynamoDB's usage of an internal hash function to evenly distribute data items across partitions, based on their partition key values. `<br>` The sort key of an item is also known as its range attribute. The term "range attribute" derives from the way DynamoDB stores items with the same partition key physically close together, in sorted order by the sort key value. `<br>` Array Members: Minimum number of 1 item. ` | 2026-11-07 |
| `/definitions/ImportSourceSpecification/properties/InputCompressionType` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/amazondynamodb/latest/APIReference/API_ImportTable.html#DDB-ImportTable-request-InputCompressionType) | ` Type of compression to be used on the input coming from the imported table. `<br>` Valid Values: GZIP \| ZSTD \| NONE ` | 2026-11-07 |
| `/definitions/KeySchema/properties/AttributeName` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/amazondynamodb/latest/APIReference/API_KeySchemaElement.html#DDB-Type-KeySchemaElement-AttributeName) | ` The name of a key attribute. `<br>` Length Constraints: Minimum length of 1. Maximum length of 255. ` | 2026-11-07 |
| `/definitions/LocalSecondaryIndex/properties/IndexName` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/amazondynamodb/latest/APIReference/API_LocalSecondaryIndex.html#DDB-Type-LocalSecondaryIndex-IndexName) | ` The name of the local secondary index. The name must be unique among all other indexes on this table. `<br>` Length Constraints: Minimum length of 3. Maximum length of 255. `<br>` Pattern: [a-zA-Z0-9_.-]+ ` | 2026-11-07 |
| `/definitions/LocalSecondaryIndex/properties/KeySchema` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/amazondynamodb/latest/APIReference/API_LocalSecondaryIndex.html#DDB-Type-LocalSecondaryIndex-KeySchema) | ` The complete key schema for the local secondary index, consisting of one or more pairs of attribute names and key types: `<br>` HASH - partition key `<br>` RANGE - sort key `<br>` The partition key of an item is also known as its hash attribute. The term "hash attribute" derives from DynamoDB's usage of an internal hash function to evenly distribute data items across partitions, based on their partition key values. `<br>` The sort key of an item is also known as its range attribute. The term "range attribute" derives from the way DynamoDB stores items with the same partition key physically close together, in sorted order by the sort key value. `<br>` Array Members: Minimum number of 1 item. ` | 2026-11-07 |
| `/definitions/Projection/properties/NonKeyAttributes` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/amazondynamodb/latest/APIReference/API_Projection.html#DDB-Type-Projection-NonKeyAttributes) | ` Represents the non-key attribute names which will be projected into the index. `<br>` For global and local secondary indexes, the total count of NonKeyAttributes summed across all of the secondary indexes, must not exceed 100. If you project the same attribute into two different indexes, this counts as two distinct attributes when determining the total. This limit only applies when you specify the ProjectionType of INCLUDE. You still can specify the ProjectionType of ALL to project all attributes from the source table, even if the table has more than 100 attributes. `<br>` Array Members: Minimum number of 1 item. Maximum number of 20 items. `<br>` Length Constraints: Minimum length of 1. Maximum length of 255. ` | 2026-11-07 |
| `/definitions/ProvisionedThroughput/properties/ReadCapacityUnits` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/amazondynamodb/latest/APIReference/API_ProvisionedThroughput.html#DDB-Type-ProvisionedThroughput-ReadCapacityUnits) | ` Valid Range: Minimum value of 1. ` | 2026-11-07 |
| `/definitions/ProvisionedThroughput/properties/ReadCapacityUnits` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/AWSCloudFormation/latest/TemplateReference/aws-resource-dynamodb-table.html#cfn-dynamodb-table-provisionedthroughput) | ` If you set BillingMode as PROVISIONED, you must specify this property. If you set BillingMode as PAY_PER_REQUEST, you cannot specify this property. ` | 2026-11-07 |
| `/definitions/ProvisionedThroughput/properties/WriteCapacityUnits` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/amazondynamodb/latest/APIReference/API_ProvisionedThroughput.html#DDB-Type-ProvisionedThroughput-WriteCapacityUnits) | ` Valid Range: Minimum value of 1. ` | 2026-11-07 |
| `/definitions/ProvisionedThroughput/properties/WriteCapacityUnits` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/AWSCloudFormation/latest/TemplateReference/aws-resource-dynamodb-table.html#cfn-dynamodb-table-provisionedthroughput) | ` If you set BillingMode as PROVISIONED, you must specify this property. If you set BillingMode as PAY_PER_REQUEST, you cannot specify this property. ` | 2026-11-07 |
| `/definitions/ResourcePolicy/properties/PolicyDocument` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/IAM/latest/UserGuide/reference_policies_grammar.html) | ` <version_block> = "Version" : ("2008-10-17" \| "2012-10-17") `<br>` <effect_block> = "Effect" : ("Allow" \| "Deny") ` | 2026-11-07 |
| `/definitions/ResourcePolicy/properties/PolicyDocument` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/IAM/latest/UserGuide/reference_policies_elements_statement.html) | ` The Statement element can contain a single statement or an array of individual statements. ` | 2026-11-07 |
| `/definitions/ResourcePolicy/properties/PolicyDocument` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/IAM/latest/UserGuide/reference_policies_elements_condition.html) | ` You can create conditions that compare strings, dates, numbers, and more using typical Boolean comparisons such as equals, greater than, and less than. ` | 2026-11-07 |
| `/definitions/S3BucketSource/properties/S3Bucket` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/amazondynamodb/latest/APIReference/API_S3BucketSource.html#DDB-Type-S3BucketSource-S3Bucket) | ` The S3 bucket that is being imported from. `<br>` Length Constraints: Maximum length of 255. `<br>` Pattern: ^[a-z0-9A-Z]+[\.\-\w]*[a-z0-9A-Z]+$ ` | 2026-11-07 |
| `/definitions/S3BucketSource/properties/S3KeyPrefix` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/amazondynamodb/latest/APIReference/API_S3BucketSource.html#DDB-Type-S3BucketSource-S3KeyPrefix) | ` The key prefix shared by all S3 Objects that are being imported. `<br>` Length Constraints: Maximum length of 1024. ` | 2026-11-07 |
| `/definitions/SSESpecification/properties/SSEType` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/amazondynamodb/latest/APIReference/API_SSESpecification.html#DDB-Type-SSESpecification-SSEType) | ` Server-side encryption type. The only supported value is: `<br>` KMS - Server-side encryption that uses AWS Key Management Service. The key is stored in your account and is managed by AWS KMS (AWS KMS charges apply). `<br>` Valid Values: AES256 \| KMS ` | 2026-11-07 |
| `/definitions/TimeToLiveSpecification/properties/AttributeName` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/amazondynamodb/latest/APIReference/API_TimeToLiveSpecification.html#DDB-Type-TimeToLiveSpecification-AttributeName) | ` The name of the TTL attribute used to store the expiration time for items in the table. `<br>` Length Constraints: Minimum length of 1. Maximum length of 255. ` | 2026-11-07 |
| `/properties/KeySchema` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/amazondynamodb/latest/APIReference/API_CreateTable.html#DDB-CreateTable-request-KeySchema) | ` Specifies the attributes that make up the primary key for a table or an index. The attributes in KeySchema must also be defined in the AttributeDefinitions array. For more information, see Data Model in the Amazon DynamoDB Developer Guide. `<br>` Each KeySchemaElement in the array is composed of: `<br>` AttributeName - The name of this key attribute. `<br>` KeyType - The role that the key attribute will assume: HASH - partition keyRANGE - sort key `<br>` KeyType - The role that the key attribute will assume: `<br>` HASH - partition key `<br>` RANGE - sort key `<br>` The partition key of an item is also known as its hash attribute. The term "hash attribute" derives from the DynamoDB usage of an internal hash function to evenly distribute data items across partitions, based on their partition key values. `<br>` The sort key of an item is also known as its range attribute. The term "range attribute" derives from the way DynamoDB stores items with the same partition key physically close together, in sorted order by the sort key value. `<br>` For a simple primary key (partition key), you must provide exactly one element with a KeyType of HASH. `<br>` For a composite primary key (partition key and sort key), you must provide exactly two elements, in this order: The first element must have a KeyType of HASH, and the second element must have a KeyType of RANGE. `<br>` For more information, see Working with Tables in the Amazon DynamoDB Developer Guide. `<br>` Array Members: Minimum number of 1 item. ` | 2026-11-07 |
| `/properties/LocalSecondaryIndexes` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/amazondynamodb/latest/APIReference/API_CreateTable.html#DDB-CreateTable-request-LocalSecondaryIndexes) | ` One or more local secondary indexes (the maximum is 5) to be created on the table. Each index is scoped to a given partition key value. There is a 10 GB size limit per partition key value; otherwise, the size of a local secondary index is unconstrained. `<br>` Each local secondary index in the array includes the following: `<br>` IndexName - The name of the local secondary index. Must be unique only for this table. `<br>` KeySchema - Specifies the key schema for the local secondary index. The key schema must begin with the same partition key as the table. `<br>` Projection - Specifies attributes that are copied (projected) from the table into the index. These are in addition to the primary key attributes and index key attributes, which are automatically projected. Each attribute specification is composed of: ProjectionType - One of the following: KEYS_ONLY - Only the index and primary keys are projected into the index.INCLUDE - Only the specified table attributes are projected into the index. The list of projected attributes is in NonKeyAttributes.ALL - All of the table attributes are projected into the index.NonKeyAttributes - A list of one or more non-key attribute names that are projected into the secondary index. The total count of attributes provided in NonKeyAttributes, summed across all of the secondary indexes, must not exceed 100. If you project the same attribute into two different indexes, this counts as two distinct attributes when determining the total. This limit only applies when you specify the ProjectionType of INCLUDE. You still can specify the ProjectionType of ALL to project all attributes from the source table, even if the table has more than 100 attributes. `<br>` Projection - Specifies attributes that are copied (projected) from the table into the index. These are in addition to the primary key attributes and index key attributes, which are automatically projected. Each attribute specification is composed of: `<br>` ProjectionType - One of the following: KEYS_ONLY - Only the index and primary keys are projected into the index.INCLUDE - Only the specified table attributes are projected into the index. The list of projected attributes is in NonKeyAttributes.ALL - All of the table attributes are projected into the index. `<br>` ProjectionType - One of the following: `<br>` KEYS_ONLY - Only the index and primary keys are projected into the index. `<br>` INCLUDE - Only the specified table attributes are projected into the index. The list of projected attributes is in NonKeyAttributes. `<br>` ALL - All of the table attributes are projected into the index. `<br>` NonKeyAttributes - A list of one or more non-key attribute names that are projected into the secondary index. The total count of attributes provided in NonKeyAttributes, summed across all of the secondary indexes, must not exceed 100. If you project the same attribute into two different indexes, this counts as two distinct attributes when determining the total. This limit only applies when you specify the ProjectionType of INCLUDE. You still can specify the ProjectionType of ALL to project all attributes from the source table, even if the table has more than 100 attributes. ` | 2026-11-07 |
| `/properties/TableName` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/amazondynamodb/latest/APIReference/API_TableCreationParameters.html#DDB-Type-TableCreationParameters-TableName) | ` The name of the table created as part of the import operation. `<br>` Length Constraints: Minimum length of 3. Maximum length of 255. `<br>` Pattern: [a-zA-Z0-9_.-]+ ` | 2026-11-07 |

