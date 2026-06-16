//! table schema definition for AWS Cloud Control
//!
//! Auto-generated from CloudFormation schema: AWS::DynamoDB::Table
//!
//! DO NOT EDIT MANUALLY - regenerate with carina-codegen

use crate::schemas::config::AwsccSchemaConfig;
use carina_core::resource::{ConcreteValue, Value};
use carina_core::schema::{
    AttributeSchema, AttributeType, ResourceSchema, StructField, legacy_validator,
};

pub fn arn() -> AttributeType {
    AttributeType::refined_string_with_validator(
        Some(carina_aws_types::provider_type("dynamodb", "Table", "Arn")),
        Some("^arn:(aws|aws-cn|aws-us-gov):dynamodb:[^:]*:[^:]*:table/.+$".to_string()),
        None,
        legacy_validator(|value| {
            if let Value::Concrete(ConcreteValue::String(s)) = value {
                carina_aws_types::validate_service_arn(s, "dynamodb", Some("table/"))
                    .map_err(|reason| format!("Invalid dynamodb ARN '{}': {}", s, reason))
            } else {
                Err("Expected string".to_string())
            }
        }),
        None,
    )
}

const VALID_ATTRIBUTE_DEFINITION_ATTRIBUTE_TYPE: &[&str] = &["S", "N", "B"];

const VALID_BILLING_MODE: &[&str] = &["PAY_PER_REQUEST", "PROVISIONED"];

const VALID_CONTRIBUTOR_INSIGHTS_SPECIFICATION_MODE: &[&str] =
    &["ACCESSED_AND_THROTTLED_KEYS", "THROTTLED_KEYS"];

const VALID_IMPORT_SOURCE_SPECIFICATION_INPUT_FORMAT: &[&str] = &["CSV", "DYNAMODB_JSON", "ION"];

const VALID_KEY_SCHEMA_KEY_TYPE: &[&str] = &["HASH", "RANGE"];

const VALID_KINESIS_STREAM_SPECIFICATION_APPROXIMATE_CREATION_DATE_TIME_PRECISION: &[&str] =
    &["MICROSECOND", "MILLISECOND"];

const VALID_PROJECTION_PROJECTION_TYPE: &[&str] = &["KEYS_ONLY", "INCLUDE", "ALL"];

const VALID_STREAM_SPECIFICATION_STREAM_VIEW_TYPE: &[&str] =
    &["KEYS_ONLY", "NEW_IMAGE", "OLD_IMAGE", "NEW_AND_OLD_IMAGES"];

const VALID_TABLE_CLASS: &[&str] = &["STANDARD", "STANDARD_INFREQUENT_ACCESS"];

#[allow(dead_code)]
fn validate_max_read_request_units_range(value: &Value) -> Result<(), String> {
    if let Value::Concrete(ConcreteValue::Int(n)) = value {
        if *n < 1 {
            Err(format!("Value {} is out of range 1..", n))
        } else {
            Ok(())
        }
    } else {
        Err("Expected integer".to_string())
    }
}

#[allow(dead_code)]
fn validate_max_write_request_units_range(value: &Value) -> Result<(), String> {
    if let Value::Concrete(ConcreteValue::Int(n)) = value {
        if *n < 1 {
            Err(format!("Value {} is out of range 1..", n))
        } else {
            Ok(())
        }
    } else {
        Err("Expected integer".to_string())
    }
}

#[allow(dead_code)]
fn validate_read_units_per_second_range(value: &Value) -> Result<(), String> {
    if let Value::Concrete(ConcreteValue::Int(n)) = value {
        if *n < 1 {
            Err(format!("Value {} is out of range 1..", n))
        } else {
            Ok(())
        }
    } else {
        Err("Expected integer".to_string())
    }
}

#[allow(dead_code)]
fn validate_recovery_period_in_days_range(value: &Value) -> Result<(), String> {
    if let Value::Concrete(ConcreteValue::Int(n)) = value {
        if *n < 1 || *n > 35 {
            Err(format!("Value {} is out of range 1..=35", n))
        } else {
            Ok(())
        }
    } else {
        Err("Expected integer".to_string())
    }
}

#[allow(dead_code)]
fn validate_write_units_per_second_range(value: &Value) -> Result<(), String> {
    if let Value::Concrete(ConcreteValue::Int(n)) = value {
        if *n < 1 {
            Err(format!("Value {} is out of range 1..", n))
        } else {
            Ok(())
        }
    } else {
        Err("Expected integer".to_string())
    }
}

/// Returns the schema config for dynamodb_table (AWS::DynamoDB::Table)
pub fn dynamodb_table_config() -> AwsccSchemaConfig {
    AwsccSchemaConfig {
        aws_type_name: "AWS::DynamoDB::Table",
        resource_type_name: "dynamodb.Table",
        primary_identifier: &["TableName"],
        has_tags: true,
        schema: ResourceSchema::new("dynamodb.Table")
	        .with_description("The ``AWS::DynamoDB::Table`` resource creates a DDB table. For more information, see [CreateTable](https://docs.aws.amazon.com/amazondynamodb/latest/APIReference/API_CreateTable.html) in the *API Reference*.  You should be aware of the following behaviors when working with DDB tables:   +  CFNlong typically creates DDB tables in parallel. However, if your template includes multiple DDB tables with indexes, you must declare dependencies so that the tables are created sequentially. DDBlong limits the number of tables with secondary indexes that are in the creating state. If you create multiple tables with indexes at the same time, DDB returns an error and the stack operation fails. For an example, see [DynamoDB Table with a DependsOn Attribute](https://docs.aws.amazon.com/AWSCloudFormation/latest/UserGuide/aws-resource-dynamodb-table.html#aws-resource-dynamodb-table--examples--DynamoDB_Table_with_a_DependsOn_Attribute).       Our guidance is to use the latest schema documented for your CFNlong templates. This schema supports the provisioning of all table settings below. When using this schema in your CFNlong templates, please ensure that your Identity and Access Management (IAM) policies are updated with appropriate permissions to allow for the authorization of these setting changes.")
        .attribute(
            AttributeSchema::new("arn", self::arn())
                .read_only()
                .with_description(" (read-only)")
                .with_provider_name("Arn"),
        )
        .attribute(
            AttributeSchema::new("attribute_definitions", AttributeType::list(AttributeType::struct_("AttributeDefinition".to_string(), vec![StructField::new("attribute_name", AttributeType::string()).required().with_description("A name for the attribute.").with_provider_name("AttributeName"),
                    StructField::new("attribute_type", AttributeType::enum_(carina_core::schema::enum_identity("AttributeType", Some("aws.dynamodb.Table.AttributeDefinition")), Some(vec!["S".to_string(), "N".to_string(), "B".to_string()]), vec![("S".to_string(), "s".to_string()), ("N".to_string(), "n".to_string()), ("B".to_string(), "b".to_string())], None, None)).required().with_description("The data type for the attribute, where: + ``S`` - the attribute is of type String + ``N`` - the attribute is of type Number + ``B`` - the attribute is of type Binary").with_provider_name("AttributeType")])))
                .with_description("A list of attributes that describe the key schema for the table and indexes. This property is required to create a DDB table. Update requires: [Some interruptions](https://docs.aws.amazon.com/AWSCloudFormation/latest/UserGuide/using-cfn-updating-stacks-update-behaviors.html#update-some-interrupt). Replacement if you edit an existing AttributeDefinition.")
                .with_provider_name("AttributeDefinitions")
                .with_block_name("attribute_definition"),
        )
        .attribute(
            AttributeSchema::new("billing_mode", AttributeType::enum_(carina_core::schema::enum_identity("BillingMode", Some("aws.dynamodb.Table")), Some(vec!["PAY_PER_REQUEST".to_string(), "PROVISIONED".to_string()]), vec![("PAY_PER_REQUEST".to_string(), "pay_per_request".to_string()), ("PROVISIONED".to_string(), "provisioned".to_string())], None, None))
                .with_description("Specify how you are charged for read and write throughput and how you manage capacity. Valid values include: + ``PAY_PER_REQUEST`` - We recommend using ``PAY_PER_REQUEST`` for most DynamoDB workloads. ``PAY_PER_REQUEST`` sets the billing mode to [On-demand capacity mode](https://docs.aws.amazon.com/amazondynamodb/latest/developerguide/on-demand-capacity-mode.html). + ``PROVISIONED`` - We recommend using ``PROVISIONED`` for steady workloads with predictable growth where capacity requirements can be reliably forecasted. ``PROVISIONED`` sets the billing mode to [Provisioned capacity mode](https://docs.aws.amazon.com/amazondynamodb/latest/developerguide/provisioned-capacity-mode.html). If not specified, the default is ``PROVISIONED``.")
                .with_provider_name("BillingMode"),
        )
        .attribute(
            AttributeSchema::new("contributor_insights_specification", AttributeType::struct_("ContributorInsightsSpecification".to_string(), vec![StructField::new("enabled", AttributeType::bool()).required().with_description("Indicates whether CloudWatch Contributor Insights are to be enabled (true) or disabled (false).").with_provider_name("Enabled"),
                    StructField::new("mode", AttributeType::enum_(carina_core::schema::enum_identity("Mode", Some("aws.dynamodb.Table.ContributorInsightsSpecification")), Some(vec!["ACCESSED_AND_THROTTLED_KEYS".to_string(), "THROTTLED_KEYS".to_string()]), vec![("ACCESSED_AND_THROTTLED_KEYS".to_string(), "accessed_and_throttled_keys".to_string()), ("THROTTLED_KEYS".to_string(), "throttled_keys".to_string())], None, None)).with_description("Specifies the CloudWatch Contributor Insights mode for a table. Valid values are ``ACCESSED_AND_THROTTLED_KEYS`` (tracks all access and throttled events) or ``THROTTLED_KEYS`` (tracks only throttled events). This setting determines what type of contributor insights data is collected for the table.").with_provider_name("Mode")]))
                .with_description("The settings used to specify whether to enable CloudWatch Contributor Insights for the table and define which events to monitor.")
                .with_provider_name("ContributorInsightsSpecification"),
        )
        .attribute(
            AttributeSchema::new("deletion_protection_enabled", AttributeType::bool())
                .with_description("Determines if a table is protected from deletion. When enabled, the table cannot be deleted by any user or process. This setting is disabled by default. For more information, see [Using deletion protection](https://docs.aws.amazon.com/amazondynamodb/latest/developerguide/WorkingWithTables.Basics.html#WorkingWithTables.Basics.DeletionProtection) in the *Developer Guide*.")
                .with_provider_name("DeletionProtectionEnabled"),
        )
        .attribute(
            AttributeSchema::new("global_secondary_indexes", AttributeType::list(AttributeType::struct_("GlobalSecondaryIndex".to_string(), vec![StructField::new("contributor_insights_specification", AttributeType::ref_("ContributorInsightsSpecification".to_string())).with_description("The settings used to specify whether to enable CloudWatch Contributor Insights for the global table and define which events to monitor.").with_provider_name("ContributorInsightsSpecification"),
                    StructField::new("index_name", AttributeType::string()).required().with_description("The name of the global secondary index. The name must be unique among all other indexes on this table.").with_provider_name("IndexName"),
                    StructField::new("key_schema", AttributeType::list(AttributeType::struct_("KeySchema".to_string(), vec![StructField::new("attribute_name", AttributeType::string()).required().with_description("The name of a key attribute.").with_provider_name("AttributeName"),
                    StructField::new("key_type", AttributeType::enum_(carina_core::schema::enum_identity("KeyType", Some("aws.dynamodb.Table.GlobalSecondaryIndex.KeySchema")), Some(vec!["HASH".to_string(), "RANGE".to_string()]), vec![("HASH".to_string(), "hash".to_string()), ("RANGE".to_string(), "range".to_string())], None, None)).required().with_description("The role that this key attribute will assume: + ``HASH`` - partition key + ``RANGE`` - sort key The partition key of an item is also known as its *hash attribute*. The term \"hash attribute\" derives from DynamoDB's usage of an internal hash function to evenly distribute data items across partitions, based on their partition key values. The sort key of an item is also known as its *range attribute*. The term \"range attribute\" derives from the way DynamoDB stores items with the same partition key physically close together, in sorted order by the sort key value.").with_provider_name("KeyType")]))).required().with_description("The complete key schema for a global secondary index, which consists of one or more pairs of attribute names and key types: + ``HASH`` - partition key + ``RANGE`` - sort key The partition key of an item is also known as its *hash attribute*. The term \"hash attribute\" derives from DynamoDB's usage of an internal hash function to evenly distribute data items across partitions, based on their partition key values. The sort key of an item is also known as its *range attribute*. The term \"range attribute\" derives from the way DynamoDB stores items with the same partition key physically close together, in sorted order by the sort key value.").with_provider_name("KeySchema").with_block_name("key_schema"),
                    StructField::new("on_demand_throughput", AttributeType::struct_("OnDemandThroughput".to_string(), vec![StructField::new("max_read_request_units", AttributeType::refined_int(None, Some((Some(1), None)))).with_description("Maximum number of read request units for the specified table. To specify a maximum ``OnDemandThroughput`` on your table, set the value of ``MaxReadRequestUnits`` as greater than or equal to 1. To remove the maximum ``OnDemandThroughput`` that is currently set on your table, set the value of ``MaxReadRequestUnits`` to -1.").with_provider_name("MaxReadRequestUnits"),
                    StructField::new("max_write_request_units", AttributeType::refined_int(None, Some((Some(1), None)))).with_description("Maximum number of write request units for the specified table. To specify a maximum ``OnDemandThroughput`` on your table, set the value of ``MaxWriteRequestUnits`` as greater than or equal to 1. To remove the maximum ``OnDemandThroughput`` that is currently set on your table, set the value of ``MaxWriteRequestUnits`` to -1.").with_provider_name("MaxWriteRequestUnits")])).with_description("The maximum number of read and write units for the specified global secondary index. If you use this parameter, you must specify ``MaxReadRequestUnits``, ``MaxWriteRequestUnits``, or both. You must use either ``OnDemandThroughput`` or ``ProvisionedThroughput`` based on your table's capacity mode.").with_provider_name("OnDemandThroughput"),
                    StructField::new("projection", AttributeType::struct_("Projection".to_string(), vec![StructField::new("non_key_attributes", AttributeType::list(AttributeType::string())).with_description("Represents the non-key attribute names which will be projected into the index. For global and local secondary indexes, the total count of ``NonKeyAttributes`` summed across all of the secondary indexes, must not exceed 100. If you project the same attribute into two different indexes, this counts as two distinct attributes when determining the total. This limit only applies when you specify the ProjectionType of ``INCLUDE``. You still can specify the ProjectionType of ``ALL`` to project all attributes from the source table, even if the table has more than 100 attributes.").with_provider_name("NonKeyAttributes"),
                    StructField::new("projection_type", AttributeType::enum_(carina_core::schema::enum_identity("ProjectionType", Some("aws.dynamodb.Table.GlobalSecondaryIndex.Projection")), Some(vec!["KEYS_ONLY".to_string(), "INCLUDE".to_string(), "ALL".to_string()]), vec![("KEYS_ONLY".to_string(), "keys_only".to_string()), ("INCLUDE".to_string(), "include".to_string()), ("ALL".to_string(), "all".to_string())], None, None)).with_description("The set of attributes that are projected into the index: + ``KEYS_ONLY`` - Only the index and primary keys are projected into the index. + ``INCLUDE`` - In addition to the attributes described in ``KEYS_ONLY``, the secondary index will include other non-key attributes that you specify. + ``ALL`` - All of the table attributes are projected into the index. When using the DynamoDB console, ``ALL`` is selected by default.").with_provider_name("ProjectionType")])).required().with_description("Represents attributes that are copied (projected) from the table into the global secondary index. These are in addition to the primary key attributes and index key attributes, which are automatically projected.").with_provider_name("Projection"),
                    StructField::new("provisioned_throughput", AttributeType::struct_("ProvisionedThroughput".to_string(), vec![StructField::new("read_capacity_units", AttributeType::int()).required().with_description("The maximum number of strongly consistent reads consumed per second before DynamoDB returns a ``ThrottlingException``. For more information, see [Specifying Read and Write Requirements](https://docs.aws.amazon.com/amazondynamodb/latest/developerguide/ProvisionedThroughput.html) in the *Amazon DynamoDB Developer Guide*. If read/write capacity mode is ``PAY_PER_REQUEST`` the value is set to 0.").with_provider_name("ReadCapacityUnits"),
                    StructField::new("write_capacity_units", AttributeType::int()).required().with_description("The maximum number of writes consumed per second before DynamoDB returns a ``ThrottlingException``. For more information, see [Specifying Read and Write Requirements](https://docs.aws.amazon.com/amazondynamodb/latest/developerguide/ProvisionedThroughput.html) in the *Amazon DynamoDB Developer Guide*. If read/write capacity mode is ``PAY_PER_REQUEST`` the value is set to 0.").with_provider_name("WriteCapacityUnits")])).with_description("Represents the provisioned throughput settings for the specified global secondary index. You must use either ``OnDemandThroughput`` or ``ProvisionedThroughput`` based on your table's capacity mode. For current minimum and maximum provisioned throughput values, see [Service, Account, and Table Quotas](https://docs.aws.amazon.com/amazondynamodb/latest/developerguide/Limits.html) in the *Amazon DynamoDB Developer Guide*.").with_provider_name("ProvisionedThroughput"),
                    StructField::new("warm_throughput", AttributeType::struct_("WarmThroughput".to_string(), vec![StructField::new("read_units_per_second", AttributeType::refined_int(None, Some((Some(1), None)))).with_description("Represents the number of read operations your base table can instantaneously support.").with_provider_name("ReadUnitsPerSecond"),
                    StructField::new("write_units_per_second", AttributeType::refined_int(None, Some((Some(1), None)))).with_description("Represents the number of write operations your base table can instantaneously support.").with_provider_name("WriteUnitsPerSecond")])).with_description("Represents the warm throughput value (in read units per second and write units per second) for the specified secondary index. If you use this parameter, you must specify ``ReadUnitsPerSecond``, ``WriteUnitsPerSecond``, or both.").with_provider_name("WarmThroughput")])))
                .with_description("Global secondary indexes to be created on the table. You can create up to 20 global secondary indexes. If you update a table to include a new global secondary index, CFNlong initiates the index creation and then proceeds with the stack update. CFNlong doesn't wait for the index to complete creation because the backfilling phase can take a long time, depending on the size of the table. You can't use the index or update the table until the index's status is ``ACTIVE``. You can track its status by using the DynamoDB [DescribeTable](https://docs.aws.amazon.com/cli/latest/reference/dynamodb/describe-table.html) command. If you add or delete an index during an update, we recommend that you don't update any other resources. If your stack fails to update and is rolled back while adding a new index, you must manually delete the index. Updates are not supported. The following are exceptions: + If you update either the contributor insights specification or the provisioned throughput values of global secondary indexes, you can update the table without interruption. + You can delete or add one global secondary index without interruption. If you do both in the same update (for example, by changing the index's logical ID), the update fails.")
                .with_provider_name("GlobalSecondaryIndexes")
                .with_block_name("global_secondary_index"),
        )
        .attribute(
            AttributeSchema::new("import_source_specification", AttributeType::struct_("ImportSourceSpecification".to_string(), vec![StructField::new("input_compression_type", AttributeType::string()).with_description("Type of compression to be used on the input coming from the imported table.").with_provider_name("InputCompressionType"),
                    StructField::new("input_format", AttributeType::enum_(carina_core::schema::enum_identity("InputFormat", Some("aws.dynamodb.Table.ImportSourceSpecification")), Some(vec!["CSV".to_string(), "DYNAMODB_JSON".to_string(), "ION".to_string()]), vec![("CSV".to_string(), "csv".to_string()), ("DYNAMODB_JSON".to_string(), "dynamodb_json".to_string()), ("ION".to_string(), "ion".to_string())], None, None)).required().with_description("The format of the source data. Valid values for ``ImportFormat`` are ``CSV``, ``DYNAMODB_JSON`` or ``ION``.").with_provider_name("InputFormat"),
                    StructField::new("input_format_options", AttributeType::struct_("InputFormatOptions".to_string(), vec![StructField::new("csv", AttributeType::struct_("Csv".to_string(), vec![StructField::new("delimiter", AttributeType::string()).with_description("The delimiter used for separating items in the CSV file being imported.").with_provider_name("Delimiter"),
                    StructField::new("header_list", AttributeType::list(AttributeType::string())).with_description("List of the headers used to specify a common header for all source CSV files being imported. If this field is specified then the first line of each CSV file is treated as data instead of the header. If this field is not specified the the first line of each CSV file is treated as the header.").with_provider_name("HeaderList")])).with_description("The options for imported source files in CSV format. The values are Delimiter and HeaderList.").with_provider_name("Csv")])).with_description("Additional properties that specify how the input is formatted,").with_provider_name("InputFormatOptions"),
                    StructField::new("s3_bucket_source", AttributeType::struct_("S3BucketSource".to_string(), vec![StructField::new("s3_bucket", AttributeType::string()).required().with_description("The S3 bucket that is being imported from.").with_provider_name("S3Bucket"),
                    StructField::new("s3_bucket_owner", AttributeType::string()).with_description("The account number of the S3 bucket that is being imported from. If the bucket is owned by the requester this is optional.").with_provider_name("S3BucketOwner"),
                    StructField::new("s3_key_prefix", AttributeType::string()).with_description("The key prefix shared by all S3 Objects that are being imported.").with_provider_name("S3KeyPrefix")])).required().with_description("The S3 bucket that provides the source for the import.").with_provider_name("S3BucketSource")]))
                .create_only()
                .write_only()
                .with_description("Specifies the properties of data being imported from the S3 bucket source to the\" table. If you specify the ``ImportSourceSpecification`` property, and also specify either the ``StreamSpecification``, the ``TableClass`` property, the ``DeletionProtectionEnabled`` property, or the ``WarmThroughput`` property, the IAM entity creating/updating stack must have ``UpdateTable`` permission.")
                .with_provider_name("ImportSourceSpecification"),
        )
        .attribute(
            AttributeSchema::new("key_schema", AttributeType::list(AttributeType::struct_("KeySchema".to_string(), vec![StructField::new("attribute_name", AttributeType::string()).required().with_description("The name of a key attribute.").with_provider_name("AttributeName"),
                    StructField::new("key_type", AttributeType::enum_(carina_core::schema::enum_identity("KeyType", Some("aws.dynamodb.Table.KeySchema")), Some(vec!["HASH".to_string(), "RANGE".to_string()]), vec![("HASH".to_string(), "hash".to_string()), ("RANGE".to_string(), "range".to_string())], None, None)).required().with_description("The role that this key attribute will assume: + ``HASH`` - partition key + ``RANGE`` - sort key The partition key of an item is also known as its *hash attribute*. The term \"hash attribute\" derives from DynamoDB's usage of an internal hash function to evenly distribute data items across partitions, based on their partition key values. The sort key of an item is also known as its *range attribute*. The term \"range attribute\" derives from the way DynamoDB stores items with the same partition key physically close together, in sorted order by the sort key value.").with_provider_name("KeyType")])))
                .required()
                .with_description("Specifies the attributes that make up the primary key for the table. The attributes in the ``KeySchema`` property must also be defined in the ``AttributeDefinitions`` property.")
                .with_provider_name("KeySchema")
                .with_block_name("key_schema"),
        )
        .attribute(
            AttributeSchema::new("kinesis_stream_specification", AttributeType::struct_("KinesisStreamSpecification".to_string(), vec![StructField::new("approximate_creation_date_time_precision", AttributeType::enum_(carina_core::schema::enum_identity("ApproximateCreationDateTimePrecision", Some("aws.dynamodb.Table.KinesisStreamSpecification")), Some(vec!["MICROSECOND".to_string(), "MILLISECOND".to_string()]), vec![("MICROSECOND".to_string(), "microsecond".to_string()), ("MILLISECOND".to_string(), "millisecond".to_string())], None, None)).with_description("The precision for the time and date that the stream was created.").with_provider_name("ApproximateCreationDateTimePrecision"),
                    StructField::new("stream_arn", carina_aws_types::arn()).required().with_description("The ARN for a specific Kinesis data stream. Length Constraints: Minimum length of 37. Maximum length of 1024.").with_provider_name("StreamArn")]))
                .with_description("The Kinesis Data Streams configuration for the specified table.")
                .with_provider_name("KinesisStreamSpecification"),
        )
        .attribute(
            AttributeSchema::new("local_secondary_indexes", AttributeType::list(AttributeType::struct_("LocalSecondaryIndex".to_string(), vec![StructField::new("index_name", AttributeType::string()).required().with_description("The name of the local secondary index. The name must be unique among all other indexes on this table.").with_provider_name("IndexName"),
                    StructField::new("key_schema", AttributeType::list(AttributeType::struct_("KeySchema".to_string(), vec![StructField::new("attribute_name", AttributeType::string()).required().with_description("The name of a key attribute.").with_provider_name("AttributeName"),
                    StructField::new("key_type", AttributeType::enum_(carina_core::schema::enum_identity("KeyType", Some("aws.dynamodb.Table.LocalSecondaryIndex.KeySchema")), Some(vec!["HASH".to_string(), "RANGE".to_string()]), vec![("HASH".to_string(), "hash".to_string()), ("RANGE".to_string(), "range".to_string())], None, None)).required().with_description("The role that this key attribute will assume: + ``HASH`` - partition key + ``RANGE`` - sort key The partition key of an item is also known as its *hash attribute*. The term \"hash attribute\" derives from DynamoDB's usage of an internal hash function to evenly distribute data items across partitions, based on their partition key values. The sort key of an item is also known as its *range attribute*. The term \"range attribute\" derives from the way DynamoDB stores items with the same partition key physically close together, in sorted order by the sort key value.").with_provider_name("KeyType")]))).required().with_description("The complete key schema for the local secondary index, consisting of one or more pairs of attribute names and key types: + ``HASH`` - partition key + ``RANGE`` - sort key The partition key of an item is also known as its *hash attribute*. The term \"hash attribute\" derives from DynamoDB's usage of an internal hash function to evenly distribute data items across partitions, based on their partition key values. The sort key of an item is also known as its *range attribute*. The term \"range attribute\" derives from the way DynamoDB stores items with the same partition key physically close together, in sorted order by the sort key value.").with_provider_name("KeySchema").with_block_name("key_schema"),
                    StructField::new("projection", AttributeType::ref_("Projection".to_string())).required().with_description("Represents attributes that are copied (projected) from the table into the local secondary index. These are in addition to the primary key attributes and index key attributes, which are automatically projected.").with_provider_name("Projection")])))
                .with_description("Local secondary indexes to be created on the table. You can create up to 5 local secondary indexes. Each index is scoped to a given hash key value. The size of each hash key can be up to 10 gigabytes.")
                .with_provider_name("LocalSecondaryIndexes")
                .with_block_name("local_secondary_index"),
        )
        .attribute(
            AttributeSchema::new("on_demand_throughput", AttributeType::ref_("OnDemandThroughput".to_string()))
                .with_description("Sets the maximum number of read and write units for the specified on-demand table. If you use this property, you must specify ``MaxReadRequestUnits``, ``MaxWriteRequestUnits``, or both.")
                .with_provider_name("OnDemandThroughput"),
        )
        .attribute(
            AttributeSchema::new("point_in_time_recovery_specification", AttributeType::struct_("PointInTimeRecoverySpecification".to_string(), vec![StructField::new("point_in_time_recovery_enabled", AttributeType::bool()).with_description("Indicates whether point in time recovery is enabled (true) or disabled (false) on the table.").with_provider_name("PointInTimeRecoveryEnabled"),
                    StructField::new("recovery_period_in_days", AttributeType::refined_int(None, Some((Some(1), Some(35))))).with_description("The number of preceding days for which continuous backups are taken and maintained. Your table data is only recoverable to any point-in-time from within the configured recovery period. This parameter is optional. If no value is provided, the value will default to 35.").with_provider_name("RecoveryPeriodInDays")]))
                .with_description("The settings used to enable point in time recovery.")
                .with_provider_name("PointInTimeRecoverySpecification"),
        )
        .attribute(
            AttributeSchema::new("provisioned_throughput", AttributeType::ref_("ProvisionedThroughput".to_string()))
                .with_description("Throughput for the specified table, which consists of values for ``ReadCapacityUnits`` and ``WriteCapacityUnits``. For more information about the contents of a provisioned throughput structure, see [Amazon DynamoDB Table ProvisionedThroughput](https://docs.aws.amazon.com/amazondynamodb/latest/APIReference/API_ProvisionedThroughput.html). If you set ``BillingMode`` as ``PROVISIONED``, you must specify this property. If you set ``BillingMode`` as ``PAY_PER_REQUEST``, you cannot specify this property.")
                .with_provider_name("ProvisionedThroughput"),
        )
        .attribute(
            AttributeSchema::new("resource_policy", AttributeType::struct_("ResourcePolicy".to_string(), vec![StructField::new("policy_document", carina_aws_types::iam_policy_document()).required().with_description("A resource-based policy document that contains permissions to add to the specified DDB table, index, or both. In a CFNshort template, you can provide the policy in JSON or YAML format because CFNshort converts YAML to JSON before submitting it to DDB. For more information about resource-based policies, see [Using resource-based policies for](https://docs.aws.amazon.com/amazondynamodb/latest/developerguide/access-control-resource-based.html) and [Resource-based policy examples](https://docs.aws.amazon.com/amazondynamodb/latest/developerguide/rbac-examples.html).").with_provider_name("PolicyDocument")]))
                .with_description("An AWS resource-based policy document in JSON format that will be attached to the table. When you attach a resource-based policy while creating a table, the policy application is *strongly consistent*. The maximum size supported for a resource-based policy document is 20 KB. DynamoDB counts whitespaces when calculating the size of a policy against this limit. For a full list of all considerations that apply for resource-based policies, see [Resource-based policy considerations](https://docs.aws.amazon.com/amazondynamodb/latest/developerguide/rbac-considerations.html). You need to specify the ``CreateTable`` and ``PutResourcePolicy`` IAM actions for authorizing a user to create a table with a resource-based policy.")
                .with_provider_name("ResourcePolicy"),
        )
        .attribute(
            AttributeSchema::new("sse_specification", AttributeType::struct_("SSESpecification".to_string(), vec![StructField::new("kms_master_key_id", AttributeType::string()).with_description("The KMS key that should be used for the KMS encryption. To specify a key, use its key ID, Amazon Resource Name (ARN), alias name, or alias ARN. Note that you should only provide this parameter if the key is different from the default DynamoDB key ``alias/aws/dynamodb``.").with_provider_name("KMSMasterKeyId"),
                    StructField::new("sse_enabled", AttributeType::bool()).required().with_description("Indicates whether server-side encryption is done using an AWS managed key or an AWS owned key. If enabled (true), server-side encryption type is set to ``KMS`` and an AWS managed key is used (KMS charges apply). If disabled (false) or not specified, server-side encryption is set to AWS owned key.").with_provider_name("SSEEnabled"),
                    StructField::new("sse_type", AttributeType::string()).with_description("Server-side encryption type. The only supported value is: + ``KMS`` - Server-side encryption that uses KMSlong. The key is stored in your account and is managed by KMS (KMS charges apply).").with_provider_name("SSEType")]))
                .with_description("Specifies the settings to enable server-side encryption.")
                .with_provider_name("SSESpecification"),
        )
        .attribute(
            AttributeSchema::new("stream_arn", carina_aws_types::arn())
                .read_only()
                .with_description(" (read-only)")
                .with_provider_name("StreamArn"),
        )
        .attribute(
            AttributeSchema::new("stream_specification", AttributeType::struct_("StreamSpecification".to_string(), vec![StructField::new("resource_policy", AttributeType::ref_("ResourcePolicy".to_string())).with_description("Creates or updates a resource-based policy document that contains the permissions for DDB resources, such as a table's streams. Resource-based policies let you define access permissions by specifying who has access to each resource, and the actions they are allowed to perform on each resource. When you remove the ``StreamSpecification`` property from the template, DynamoDB disables the stream but retains any attached resource policy until the stream is deleted after 24 hours. When you modify the ``StreamViewType`` property, DynamoDB creates a new stream and retains the old stream's resource policy. The old stream and its resource policy are deleted after the 24-hour retention period. In a CFNshort template, you can provide the policy in JSON or YAML format because CFNshort converts YAML to JSON before submitting it to DDB. For more information about resource-based policies, see [Using resource-based policies for](https://docs.aws.amazon.com/amazondynamodb/latest/developerguide/access-control-resource-based.html) and [Resource-based policy examples](https://docs.aws.amazon.com/amazondynamodb/latest/developerguide/rbac-examples.html).").with_provider_name("ResourcePolicy"),
                    StructField::new("stream_view_type", AttributeType::enum_(carina_core::schema::enum_identity("StreamViewType", Some("aws.dynamodb.Table.StreamSpecification")), Some(vec!["KEYS_ONLY".to_string(), "NEW_IMAGE".to_string(), "OLD_IMAGE".to_string(), "NEW_AND_OLD_IMAGES".to_string()]), vec![("KEYS_ONLY".to_string(), "keys_only".to_string()), ("NEW_IMAGE".to_string(), "new_image".to_string()), ("OLD_IMAGE".to_string(), "old_image".to_string()), ("NEW_AND_OLD_IMAGES".to_string(), "new_and_old_images".to_string())], None, None)).required().with_description("When an item in the table is modified, ``StreamViewType`` determines what information is written to the stream for this table. Valid values for ``StreamViewType`` are: + ``KEYS_ONLY`` - Only the key attributes of the modified item are written to the stream. + ``NEW_IMAGE`` - The entire item, as it appears after it was modified, is written to the stream. + ``OLD_IMAGE`` - The entire item, as it appeared before it was modified, is written to the stream. + ``NEW_AND_OLD_IMAGES`` - Both the new and the old item images of the item are written to the stream.").with_provider_name("StreamViewType")]))
                .with_description("The settings for the DDB table stream, which captures changes to items stored in the table. Including this property in your CFNlong template automatically enables streaming.")
                .with_provider_name("StreamSpecification"),
        )
        .attribute(
            AttributeSchema::new("table_class", AttributeType::enum_(carina_core::schema::enum_identity("TableClass", Some("aws.dynamodb.Table")), Some(vec!["STANDARD".to_string(), "STANDARD_INFREQUENT_ACCESS".to_string()]), vec![("STANDARD".to_string(), "standard".to_string()), ("STANDARD_INFREQUENT_ACCESS".to_string(), "standard_infrequent_access".to_string())], None, None))
                .with_description("The table class of the new table. Valid values are ``STANDARD`` and ``STANDARD_INFREQUENT_ACCESS``.")
                .with_provider_name("TableClass"),
        )
        .attribute(
            AttributeSchema::new("table_name", AttributeType::string())
                .create_only()
                .with_description("A name for the table. If you don't specify a name, CFNlong generates a unique physical ID and uses that ID for the table name. For more information, see [Name Type](https://docs.aws.amazon.com/AWSCloudFormation/latest/UserGuide/aws-properties-name.html). If you specify a name, you cannot perform updates that require replacement of this resource. You can perform updates that require no or some interruption. If you must replace the resource, specify a new name.")
                .with_provider_name("TableName"),
        )
        .attribute(
            AttributeSchema::new("tags", carina_aws_types::tags_type())
                .with_description("An array of key-value pairs to apply to this resource. For more information, see [Tag](https://docs.aws.amazon.com/AWSCloudFormation/latest/UserGuide/aws-properties-resource-tags.html).")
                .with_provider_name("Tags")
                .with_block_name("tag"),
        )
        .attribute(
            AttributeSchema::new("time_to_live_specification", AttributeType::struct_("TimeToLiveSpecification".to_string(), vec![StructField::new("attribute_name", AttributeType::string()).with_description("The name of the TTL attribute used to store the expiration time for items in the table. + The ``AttributeName`` property is required when enabling the TTL, or when TTL is already enabled. + To update this property, you must first disable TTL and then enable TTL with the new attribute name.").with_provider_name("AttributeName"),
                    StructField::new("enabled", AttributeType::bool()).required().with_description("Indicates whether TTL is to be enabled (true) or disabled (false) on the table.").with_provider_name("Enabled")]))
                .with_description("Specifies the Time to Live (TTL) settings for the table. For detailed information about the limits in DynamoDB, see [Limits in Amazon DynamoDB](https://docs.aws.amazon.com/amazondynamodb/latest/developerguide/Limits.html) in the Amazon DynamoDB Developer Guide.")
                .with_provider_name("TimeToLiveSpecification"),
        )
        .attribute(
            AttributeSchema::new("warm_throughput", AttributeType::ref_("WarmThroughput".to_string()))
                .with_description("Represents the warm throughput (in read units per second and write units per second) for creating a table.")
                .with_provider_name("WarmThroughput"),
        )
        .with_name_attribute("table_name")
        .with_validator(|attrs| {
            let mut errors = Vec::new();
            if let Err(mut e) = carina_aws_types::validate_tags_map(attrs) {
                errors.append(&mut e);
            }
            if errors.is_empty() { Ok(()) } else { Err(errors) }
        })
        .with_def("ContributorInsightsSpecification", AttributeType::struct_("ContributorInsightsSpecification".to_string(), vec![StructField::new("enabled", AttributeType::bool()).required().with_description("Indicates whether CloudWatch Contributor Insights are to be enabled (true) or disabled (false).").with_provider_name("Enabled"),
                    StructField::new("mode", AttributeType::enum_(carina_core::schema::enum_identity("Mode", Some("aws.dynamodb.Table.ContributorInsightsSpecification")), Some(vec!["ACCESSED_AND_THROTTLED_KEYS".to_string(), "THROTTLED_KEYS".to_string()]), vec![("ACCESSED_AND_THROTTLED_KEYS".to_string(), "accessed_and_throttled_keys".to_string()), ("THROTTLED_KEYS".to_string(), "throttled_keys".to_string())], None, None)).with_description("Specifies the CloudWatch Contributor Insights mode for a table. Valid values are ``ACCESSED_AND_THROTTLED_KEYS`` (tracks all access and throttled events) or ``THROTTLED_KEYS`` (tracks only throttled events). This setting determines what type of contributor insights data is collected for the table.").with_provider_name("Mode")]))
        .with_def("Csv", AttributeType::struct_("Csv".to_string(), vec![StructField::new("delimiter", AttributeType::string()).with_description("The delimiter used for separating items in the CSV file being imported.").with_provider_name("Delimiter"),
                    StructField::new("header_list", AttributeType::list(AttributeType::string())).with_description("List of the headers used to specify a common header for all source CSV files being imported. If this field is specified then the first line of each CSV file is treated as data instead of the header. If this field is not specified the the first line of each CSV file is treated as the header.").with_provider_name("HeaderList")]))
        .with_def("ImportSourceSpecification", AttributeType::struct_("ImportSourceSpecification".to_string(), vec![StructField::new("input_compression_type", AttributeType::string()).with_description("Type of compression to be used on the input coming from the imported table.").with_provider_name("InputCompressionType"),
                    StructField::new("input_format", AttributeType::enum_(carina_core::schema::enum_identity("InputFormat", Some("aws.dynamodb.Table.ImportSourceSpecification")), Some(vec!["CSV".to_string(), "DYNAMODB_JSON".to_string(), "ION".to_string()]), vec![("CSV".to_string(), "csv".to_string()), ("DYNAMODB_JSON".to_string(), "dynamodb_json".to_string()), ("ION".to_string(), "ion".to_string())], None, None)).required().with_description("The format of the source data. Valid values for ``ImportFormat`` are ``CSV``, ``DYNAMODB_JSON`` or ``ION``.").with_provider_name("InputFormat"),
                    StructField::new("input_format_options", AttributeType::struct_("InputFormatOptions".to_string(), vec![StructField::new("csv", AttributeType::struct_("Csv".to_string(), vec![StructField::new("delimiter", AttributeType::string()).with_description("The delimiter used for separating items in the CSV file being imported.").with_provider_name("Delimiter"),
                    StructField::new("header_list", AttributeType::list(AttributeType::string())).with_description("List of the headers used to specify a common header for all source CSV files being imported. If this field is specified then the first line of each CSV file is treated as data instead of the header. If this field is not specified the the first line of each CSV file is treated as the header.").with_provider_name("HeaderList")])).with_description("The options for imported source files in CSV format. The values are Delimiter and HeaderList.").with_provider_name("Csv")])).with_description("Additional properties that specify how the input is formatted,").with_provider_name("InputFormatOptions"),
                    StructField::new("s3_bucket_source", AttributeType::struct_("S3BucketSource".to_string(), vec![StructField::new("s3_bucket", AttributeType::string()).required().with_description("The S3 bucket that is being imported from.").with_provider_name("S3Bucket"),
                    StructField::new("s3_bucket_owner", AttributeType::string()).with_description("The account number of the S3 bucket that is being imported from. If the bucket is owned by the requester this is optional.").with_provider_name("S3BucketOwner"),
                    StructField::new("s3_key_prefix", AttributeType::string()).with_description("The key prefix shared by all S3 Objects that are being imported.").with_provider_name("S3KeyPrefix")])).required().with_description("The S3 bucket that provides the source for the import.").with_provider_name("S3BucketSource")]))
        .with_def("InputFormatOptions", AttributeType::struct_("InputFormatOptions".to_string(), vec![StructField::new("csv", AttributeType::struct_("Csv".to_string(), vec![StructField::new("delimiter", AttributeType::string()).with_description("The delimiter used for separating items in the CSV file being imported.").with_provider_name("Delimiter"),
                    StructField::new("header_list", AttributeType::list(AttributeType::string())).with_description("List of the headers used to specify a common header for all source CSV files being imported. If this field is specified then the first line of each CSV file is treated as data instead of the header. If this field is not specified the the first line of each CSV file is treated as the header.").with_provider_name("HeaderList")])).with_description("The options for imported source files in CSV format. The values are Delimiter and HeaderList.").with_provider_name("Csv")]))
        .with_def("KinesisStreamSpecification", AttributeType::struct_("KinesisStreamSpecification".to_string(), vec![StructField::new("approximate_creation_date_time_precision", AttributeType::enum_(carina_core::schema::enum_identity("ApproximateCreationDateTimePrecision", Some("aws.dynamodb.Table.KinesisStreamSpecification")), Some(vec!["MICROSECOND".to_string(), "MILLISECOND".to_string()]), vec![("MICROSECOND".to_string(), "microsecond".to_string()), ("MILLISECOND".to_string(), "millisecond".to_string())], None, None)).with_description("The precision for the time and date that the stream was created.").with_provider_name("ApproximateCreationDateTimePrecision"),
                    StructField::new("stream_arn", carina_aws_types::arn()).required().with_description("The ARN for a specific Kinesis data stream. Length Constraints: Minimum length of 37. Maximum length of 1024.").with_provider_name("StreamArn")]))
        .with_def("OnDemandThroughput", AttributeType::struct_("OnDemandThroughput".to_string(), vec![StructField::new("max_read_request_units", AttributeType::refined_int(None, Some((Some(1), None)))).with_description("Maximum number of read request units for the specified table. To specify a maximum ``OnDemandThroughput`` on your table, set the value of ``MaxReadRequestUnits`` as greater than or equal to 1. To remove the maximum ``OnDemandThroughput`` that is currently set on your table, set the value of ``MaxReadRequestUnits`` to -1.").with_provider_name("MaxReadRequestUnits"),
                    StructField::new("max_write_request_units", AttributeType::refined_int(None, Some((Some(1), None)))).with_description("Maximum number of write request units for the specified table. To specify a maximum ``OnDemandThroughput`` on your table, set the value of ``MaxWriteRequestUnits`` as greater than or equal to 1. To remove the maximum ``OnDemandThroughput`` that is currently set on your table, set the value of ``MaxWriteRequestUnits`` to -1.").with_provider_name("MaxWriteRequestUnits")]))
        .with_def("PointInTimeRecoverySpecification", AttributeType::struct_("PointInTimeRecoverySpecification".to_string(), vec![StructField::new("point_in_time_recovery_enabled", AttributeType::bool()).with_description("Indicates whether point in time recovery is enabled (true) or disabled (false) on the table.").with_provider_name("PointInTimeRecoveryEnabled"),
                    StructField::new("recovery_period_in_days", AttributeType::refined_int(None, Some((Some(1), Some(35))))).with_description("The number of preceding days for which continuous backups are taken and maintained. Your table data is only recoverable to any point-in-time from within the configured recovery period. This parameter is optional. If no value is provided, the value will default to 35.").with_provider_name("RecoveryPeriodInDays")]))
        .with_def("Projection", AttributeType::struct_("Projection".to_string(), vec![StructField::new("non_key_attributes", AttributeType::list(AttributeType::string())).with_description("Represents the non-key attribute names which will be projected into the index. For global and local secondary indexes, the total count of ``NonKeyAttributes`` summed across all of the secondary indexes, must not exceed 100. If you project the same attribute into two different indexes, this counts as two distinct attributes when determining the total. This limit only applies when you specify the ProjectionType of ``INCLUDE``. You still can specify the ProjectionType of ``ALL`` to project all attributes from the source table, even if the table has more than 100 attributes.").with_provider_name("NonKeyAttributes"),
                    StructField::new("projection_type", AttributeType::enum_(carina_core::schema::enum_identity("ProjectionType", Some("aws.dynamodb.Table.GlobalSecondaryIndex.Projection")), Some(vec!["KEYS_ONLY".to_string(), "INCLUDE".to_string(), "ALL".to_string()]), vec![("KEYS_ONLY".to_string(), "keys_only".to_string()), ("INCLUDE".to_string(), "include".to_string()), ("ALL".to_string(), "all".to_string())], None, None)).with_description("The set of attributes that are projected into the index: + ``KEYS_ONLY`` - Only the index and primary keys are projected into the index. + ``INCLUDE`` - In addition to the attributes described in ``KEYS_ONLY``, the secondary index will include other non-key attributes that you specify. + ``ALL`` - All of the table attributes are projected into the index. When using the DynamoDB console, ``ALL`` is selected by default.").with_provider_name("ProjectionType")]))
        .with_def("ProvisionedThroughput", AttributeType::struct_("ProvisionedThroughput".to_string(), vec![StructField::new("read_capacity_units", AttributeType::int()).required().with_description("The maximum number of strongly consistent reads consumed per second before DynamoDB returns a ``ThrottlingException``. For more information, see [Specifying Read and Write Requirements](https://docs.aws.amazon.com/amazondynamodb/latest/developerguide/ProvisionedThroughput.html) in the *Amazon DynamoDB Developer Guide*. If read/write capacity mode is ``PAY_PER_REQUEST`` the value is set to 0.").with_provider_name("ReadCapacityUnits"),
                    StructField::new("write_capacity_units", AttributeType::int()).required().with_description("The maximum number of writes consumed per second before DynamoDB returns a ``ThrottlingException``. For more information, see [Specifying Read and Write Requirements](https://docs.aws.amazon.com/amazondynamodb/latest/developerguide/ProvisionedThroughput.html) in the *Amazon DynamoDB Developer Guide*. If read/write capacity mode is ``PAY_PER_REQUEST`` the value is set to 0.").with_provider_name("WriteCapacityUnits")]))
        .with_def("ResourcePolicy", AttributeType::struct_("ResourcePolicy".to_string(), vec![StructField::new("policy_document", carina_aws_types::iam_policy_document()).required().with_description("A resource-based policy document that contains permissions to add to the specified DDB table, index, or both. In a CFNshort template, you can provide the policy in JSON or YAML format because CFNshort converts YAML to JSON before submitting it to DDB. For more information about resource-based policies, see [Using resource-based policies for](https://docs.aws.amazon.com/amazondynamodb/latest/developerguide/access-control-resource-based.html) and [Resource-based policy examples](https://docs.aws.amazon.com/amazondynamodb/latest/developerguide/rbac-examples.html).").with_provider_name("PolicyDocument")]))
        .with_def("S3BucketSource", AttributeType::struct_("S3BucketSource".to_string(), vec![StructField::new("s3_bucket", AttributeType::string()).required().with_description("The S3 bucket that is being imported from.").with_provider_name("S3Bucket"),
                    StructField::new("s3_bucket_owner", AttributeType::string()).with_description("The account number of the S3 bucket that is being imported from. If the bucket is owned by the requester this is optional.").with_provider_name("S3BucketOwner"),
                    StructField::new("s3_key_prefix", AttributeType::string()).with_description("The key prefix shared by all S3 Objects that are being imported.").with_provider_name("S3KeyPrefix")]))
        .with_def("SSESpecification", AttributeType::struct_("SSESpecification".to_string(), vec![StructField::new("kms_master_key_id", AttributeType::string()).with_description("The KMS key that should be used for the KMS encryption. To specify a key, use its key ID, Amazon Resource Name (ARN), alias name, or alias ARN. Note that you should only provide this parameter if the key is different from the default DynamoDB key ``alias/aws/dynamodb``.").with_provider_name("KMSMasterKeyId"),
                    StructField::new("sse_enabled", AttributeType::bool()).required().with_description("Indicates whether server-side encryption is done using an AWS managed key or an AWS owned key. If enabled (true), server-side encryption type is set to ``KMS`` and an AWS managed key is used (KMS charges apply). If disabled (false) or not specified, server-side encryption is set to AWS owned key.").with_provider_name("SSEEnabled"),
                    StructField::new("sse_type", AttributeType::string()).with_description("Server-side encryption type. The only supported value is: + ``KMS`` - Server-side encryption that uses KMSlong. The key is stored in your account and is managed by KMS (KMS charges apply).").with_provider_name("SSEType")]))
        .with_def("StreamSpecification", AttributeType::struct_("StreamSpecification".to_string(), vec![StructField::new("resource_policy", AttributeType::ref_("ResourcePolicy".to_string())).with_description("Creates or updates a resource-based policy document that contains the permissions for DDB resources, such as a table's streams. Resource-based policies let you define access permissions by specifying who has access to each resource, and the actions they are allowed to perform on each resource. When you remove the ``StreamSpecification`` property from the template, DynamoDB disables the stream but retains any attached resource policy until the stream is deleted after 24 hours. When you modify the ``StreamViewType`` property, DynamoDB creates a new stream and retains the old stream's resource policy. The old stream and its resource policy are deleted after the 24-hour retention period. In a CFNshort template, you can provide the policy in JSON or YAML format because CFNshort converts YAML to JSON before submitting it to DDB. For more information about resource-based policies, see [Using resource-based policies for](https://docs.aws.amazon.com/amazondynamodb/latest/developerguide/access-control-resource-based.html) and [Resource-based policy examples](https://docs.aws.amazon.com/amazondynamodb/latest/developerguide/rbac-examples.html).").with_provider_name("ResourcePolicy"),
                    StructField::new("stream_view_type", AttributeType::enum_(carina_core::schema::enum_identity("StreamViewType", Some("aws.dynamodb.Table.StreamSpecification")), Some(vec!["KEYS_ONLY".to_string(), "NEW_IMAGE".to_string(), "OLD_IMAGE".to_string(), "NEW_AND_OLD_IMAGES".to_string()]), vec![("KEYS_ONLY".to_string(), "keys_only".to_string()), ("NEW_IMAGE".to_string(), "new_image".to_string()), ("OLD_IMAGE".to_string(), "old_image".to_string()), ("NEW_AND_OLD_IMAGES".to_string(), "new_and_old_images".to_string())], None, None)).required().with_description("When an item in the table is modified, ``StreamViewType`` determines what information is written to the stream for this table. Valid values for ``StreamViewType`` are: + ``KEYS_ONLY`` - Only the key attributes of the modified item are written to the stream. + ``NEW_IMAGE`` - The entire item, as it appears after it was modified, is written to the stream. + ``OLD_IMAGE`` - The entire item, as it appeared before it was modified, is written to the stream. + ``NEW_AND_OLD_IMAGES`` - Both the new and the old item images of the item are written to the stream.").with_provider_name("StreamViewType")]))
        .with_def("TimeToLiveSpecification", AttributeType::struct_("TimeToLiveSpecification".to_string(), vec![StructField::new("attribute_name", AttributeType::string()).with_description("The name of the TTL attribute used to store the expiration time for items in the table. + The ``AttributeName`` property is required when enabling the TTL, or when TTL is already enabled. + To update this property, you must first disable TTL and then enable TTL with the new attribute name.").with_provider_name("AttributeName"),
                    StructField::new("enabled", AttributeType::bool()).required().with_description("Indicates whether TTL is to be enabled (true) or disabled (false) on the table.").with_provider_name("Enabled")]))
        .with_def("WarmThroughput", AttributeType::struct_("WarmThroughput".to_string(), vec![StructField::new("read_units_per_second", AttributeType::refined_int(None, Some((Some(1), None)))).with_description("Represents the number of read operations your base table can instantaneously support.").with_provider_name("ReadUnitsPerSecond"),
                    StructField::new("write_units_per_second", AttributeType::refined_int(None, Some((Some(1), None)))).with_description("Represents the number of write operations your base table can instantaneously support.").with_provider_name("WriteUnitsPerSecond")]))
    }
}

/// Returns the resource type name and all enum valid values for this module
pub fn enum_valid_values() -> (
    &'static str,
    &'static [(&'static str, &'static [&'static str])],
) {
    (
        "dynamodb.Table",
        &[
            ("attribute_type", VALID_ATTRIBUTE_DEFINITION_ATTRIBUTE_TYPE),
            ("billing_mode", VALID_BILLING_MODE),
            ("mode", VALID_CONTRIBUTOR_INSIGHTS_SPECIFICATION_MODE),
            (
                "input_format",
                VALID_IMPORT_SOURCE_SPECIFICATION_INPUT_FORMAT,
            ),
            ("key_type", VALID_KEY_SCHEMA_KEY_TYPE),
            (
                "approximate_creation_date_time_precision",
                VALID_KINESIS_STREAM_SPECIFICATION_APPROXIMATE_CREATION_DATE_TIME_PRECISION,
            ),
            ("projection_type", VALID_PROJECTION_PROJECTION_TYPE),
            (
                "stream_view_type",
                VALID_STREAM_SPECIFICATION_STREAM_VIEW_TYPE,
            ),
            ("table_class", VALID_TABLE_CLASS),
        ],
    )
}

/// Returns the IAM permissions declared by the CloudFormation handler for this operation.
pub fn required_permissions(op: carina_core::effect::PlanOp) -> &'static [&'static str] {
    match op {
        carina_core::effect::PlanOp::Create => &[
            "dynamodb:CreateTable",
            "dynamodb:DescribeImport",
            "dynamodb:DescribeTable",
            "dynamodb:DescribeTimeToLive",
            "dynamodb:UpdateTimeToLive",
            "dynamodb:UpdateContributorInsights",
            "dynamodb:UpdateContinuousBackups",
            "dynamodb:DescribeContinuousBackups",
            "dynamodb:DescribeContributorInsights",
            "dynamodb:EnableKinesisStreamingDestination",
            "dynamodb:DisableKinesisStreamingDestination",
            "dynamodb:DescribeKinesisStreamingDestination",
            "dynamodb:ImportTable",
            "dynamodb:ListTagsOfResource",
            "dynamodb:TagResource",
            "dynamodb:UpdateTable",
            "dynamodb:GetResourcePolicy",
            "dynamodb:PutResourcePolicy",
            "dynamodb:CreateTableReplica",
            "dynamodb:Scan",
            "dynamodb:Query",
            "dynamodb:UpdateItem",
            "dynamodb:PutItem",
            "dynamodb:GetItem",
            "dynamodb:DeleteItem",
            "dynamodb:BatchWriteItem",
            "dynamodb:AssociateTableReplica",
            "kinesis:DescribeStream",
            "kinesis:PutRecords",
            "iam:CreateServiceLinkedRole",
            "kms:CreateGrant",
            "kms:Decrypt",
            "kms:DescribeKey",
            "kms:ListAliases",
            "kms:Encrypt",
            "kms:RevokeGrant",
            "logs:CreateLogGroup",
            "logs:CreateLogStream",
            "logs:DescribeLogGroups",
            "logs:DescribeLogStreams",
            "logs:PutLogEvents",
            "logs:PutRetentionPolicy",
            "s3:GetObject",
            "s3:GetObjectMetadata",
            "s3:ListBucket",
        ],
        carina_core::effect::PlanOp::Read => &[
            "dynamodb:DescribeTable",
            "dynamodb:DescribeContinuousBackups",
            "dynamodb:DescribeContributorInsights",
            "dynamodb:DescribeKinesisStreamingDestination",
            "dynamodb:ListTagsOfResource",
            "dynamodb:GetResourcePolicy",
            "dynamodb:DescribeTimeToLive",
        ],
        carina_core::effect::PlanOp::Update => &[
            "dynamodb:UpdateTable",
            "dynamodb:DescribeTable",
            "dynamodb:DescribeTimeToLive",
            "dynamodb:UpdateTimeToLive",
            "dynamodb:UpdateContinuousBackups",
            "dynamodb:UpdateContributorInsights",
            "dynamodb:UpdateKinesisStreamingDestination",
            "dynamodb:DescribeContinuousBackups",
            "dynamodb:DescribeKinesisStreamingDestination",
            "dynamodb:ListTagsOfResource",
            "dynamodb:TagResource",
            "dynamodb:UntagResource",
            "dynamodb:DescribeContributorInsights",
            "dynamodb:EnableKinesisStreamingDestination",
            "dynamodb:DisableKinesisStreamingDestination",
            "dynamodb:GetResourcePolicy",
            "dynamodb:PutResourcePolicy",
            "dynamodb:DeleteResourcePolicy",
            "dynamodb:CreateTable",
            "dynamodb:CreateTableReplica",
            "dynamodb:Scan",
            "dynamodb:Query",
            "dynamodb:UpdateItem",
            "dynamodb:PutItem",
            "dynamodb:GetItem",
            "dynamodb:DeleteItem",
            "dynamodb:BatchWriteItem",
            "dynamodb:AssociateTableReplica",
            "kinesis:DescribeStream",
            "kinesis:PutRecords",
            "iam:CreateServiceLinkedRole",
            "kms:CreateGrant",
            "kms:DescribeKey",
            "kms:ListAliases",
            "kms:RevokeGrant",
        ],
        carina_core::effect::PlanOp::Delete => &["dynamodb:DeleteTable", "dynamodb:DescribeTable"],
    }
}
