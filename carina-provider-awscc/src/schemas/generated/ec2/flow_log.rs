//! flow_log schema definition for AWS Cloud Control
//!
//! Auto-generated from CloudFormation schema: AWS::EC2::FlowLog
//!
//! DO NOT EDIT MANUALLY - regenerate with carina-codegen

use super::AwsccSchemaConfig;
use super::tags_type;
use super::validate_tags_map;
use carina_core::resource::{ConcreteValue, Value};
use carina_core::schema::{
    AttributeSchema, AttributeType, ResourceSchema, StructField, legacy_validator,
};

const VALID_LOG_DESTINATION_TYPE: &[&str] = &["cloud-watch-logs", "s3", "kinesis-data-firehose"];

const VALID_RESOURCE_TYPE: &[&str] = &[
    "NetworkInterface",
    "Subnet",
    "VPC",
    "TransitGateway",
    "TransitGatewayAttachment",
    "RegionalNatGateway",
];

const VALID_TRAFFIC_TYPE: &[&str] = &["ACCEPT", "ALL", "REJECT"];

const VALID_MAX_AGGREGATION_INTERVAL_VALUES: &[i64] = &[60, 600];

fn validate_max_aggregation_interval_int_enum(value: &Value) -> Result<(), String> {
    if let Value::Concrete(ConcreteValue::Int(n)) = value {
        if VALID_MAX_AGGREGATION_INTERVAL_VALUES.contains(n) {
            Ok(())
        } else {
            Err(format!("Value {} is not a valid value", n))
        }
    } else {
        Err("Expected integer".to_string())
    }
}

/// Returns the schema config for ec2_flow_log (AWS::EC2::FlowLog)
pub fn ec2_flow_log_config() -> AwsccSchemaConfig {
    AwsccSchemaConfig {
        aws_type_name: "AWS::EC2::FlowLog",
        resource_type_name: "ec2.FlowLog",
        has_tags: true,
        schema: ResourceSchema::new("ec2.FlowLog")
        .with_description("Specifies a VPC flow log, which enables you to capture IP traffic for a specific network interface, subnet, or VPC.")
        .attribute(
            AttributeSchema::new("deliver_cross_account_role", super::iam_role_arn())
                .create_only()
                .with_description("The ARN of the IAM role that allows Amazon EC2 to publish flow logs across accounts.")
                .with_provider_name("DeliverCrossAccountRole"),
        )
        .attribute(
            AttributeSchema::new("deliver_logs_permission_arn", super::iam_role_arn())
                .create_only()
                .with_description("The ARN for the IAM role that permits Amazon EC2 to publish flow logs to a CloudWatch Logs log group in your account. If you specify LogDestinationType as s3 or kinesis-data-firehose, do not specify DeliverLogsPermissionArn or LogGroupName.")
                .with_provider_name("DeliverLogsPermissionArn"),
        )
        .attribute(
            AttributeSchema::new("destination_options", AttributeType::struct_("DestinationOptions".to_string(), vec![StructField::new("file_format", AttributeType::string_enum("FileFormat".to_string(), vec!["plain-text".to_string(), "parquet".to_string()], Some(carina_core::schema::string_enum_identity("FileFormat", Some("awscc.ec2.FlowLog.DestinationOptions"))), vec![("plain-text".to_string(), "plain_text".to_string()), ("parquet".to_string(), "parquet".to_string())])).required().with_provider_name("FileFormat"),
                    StructField::new("hive_compatible_partitions", AttributeType::bool()).required().with_provider_name("HiveCompatiblePartitions"),
                    StructField::new("per_hour_partition", AttributeType::bool()).required().with_provider_name("PerHourPartition")]))
                .create_only()
                .with_provider_name("DestinationOptions"),
        )
        .attribute(
            AttributeSchema::new("id", super::flow_log_id())
                .read_only()
                .with_description("The Flow Log ID (read-only)")
                .with_provider_name("Id"),
        )
        .attribute(
            AttributeSchema::new("log_destination", super::arn())
                .create_only()
                .with_description("Specifies the destination to which the flow log data is to be published. Flow log data can be published to a CloudWatch Logs log group, an Amazon S3 bucket, or a Kinesis Firehose stream. The value specified for this parameter depends on the value specified for LogDestinationType.")
                .with_provider_name("LogDestination"),
        )
        .attribute(
            AttributeSchema::new("log_destination_type", AttributeType::string_enum("LogDestinationType".to_string(), vec!["cloud-watch-logs".to_string(), "s3".to_string(), "kinesis-data-firehose".to_string()], Some(carina_core::schema::string_enum_identity("LogDestinationType", Some("awscc.ec2.FlowLog"))), vec![("cloud-watch-logs".to_string(), "cloud_watch_logs".to_string()), ("s3".to_string(), "s3".to_string()), ("kinesis-data-firehose".to_string(), "kinesis_data_firehose".to_string())]))
                .create_only()
                .with_description("Specifies the type of destination to which the flow log data is to be published. Flow log data can be published to CloudWatch Logs or Amazon S3.")
                .with_provider_name("LogDestinationType"),
        )
        .attribute(
            AttributeSchema::new("log_format", AttributeType::string())
                .create_only()
                .with_description("The fields to include in the flow log record, in the order in which they should appear.")
                .with_provider_name("LogFormat"),
        )
        .attribute(
            AttributeSchema::new("log_group_name", AttributeType::string())
                .create_only()
                .with_description("The name of a new or existing CloudWatch Logs log group where Amazon EC2 publishes your flow logs. If you specify LogDestinationType as s3 or kinesis-data-firehose, do not specify DeliverLogsPermissionArn or LogGroupName.")
                .with_provider_name("LogGroupName"),
        )
        .attribute(
            AttributeSchema::new("max_aggregation_interval", AttributeType::custom(None, AttributeType::int(), None, None, legacy_validator(validate_max_aggregation_interval_int_enum), None))
                .create_only()
                .with_description("The maximum interval of time during which a flow of packets is captured and aggregated into a flow log record. You can specify 60 seconds (1 minute) or 600 seconds (10 minutes).")
                .with_provider_name("MaxAggregationInterval"),
        )
        .attribute(
            AttributeSchema::new("resource_id", AttributeType::string())
                .required()
                .create_only()
                .with_description("The ID of the subnet, network interface, or VPC for which you want to create a flow log.")
                .with_provider_name("ResourceId"),
        )
        .attribute(
            AttributeSchema::new("resource_type", AttributeType::string_enum("ResourceType".to_string(), vec!["NetworkInterface".to_string(), "Subnet".to_string(), "VPC".to_string(), "TransitGateway".to_string(), "TransitGatewayAttachment".to_string(), "RegionalNatGateway".to_string()], Some(carina_core::schema::string_enum_identity("ResourceType", Some("awscc.ec2.FlowLog"))), vec![("NetworkInterface".to_string(), "network_interface".to_string()), ("Subnet".to_string(), "subnet".to_string()), ("VPC".to_string(), "vpc".to_string()), ("TransitGateway".to_string(), "transit_gateway".to_string()), ("TransitGatewayAttachment".to_string(), "transit_gateway_attachment".to_string()), ("RegionalNatGateway".to_string(), "regional_nat_gateway".to_string())]))
                .required()
                .create_only()
                .with_description("The type of resource for which to create the flow log. For example, if you specified a VPC ID for the ResourceId property, specify VPC for this property.")
                .with_provider_name("ResourceType"),
        )
        .attribute(
            AttributeSchema::new("tags", tags_type())
                .with_description("The tags to apply to the flow logs.")
                .with_provider_name("Tags")
                .with_block_name("tag"),
        )
        .attribute(
            AttributeSchema::new("traffic_type", AttributeType::string_enum("TrafficType".to_string(), vec!["ACCEPT".to_string(), "ALL".to_string(), "REJECT".to_string()], Some(carina_core::schema::string_enum_identity("TrafficType", Some("awscc.ec2.FlowLog"))), vec![("ACCEPT".to_string(), "accept".to_string()), ("ALL".to_string(), "all".to_string()), ("REJECT".to_string(), "reject".to_string())]))
                .create_only()
                .with_description("The type of traffic to log. You can log traffic that the resource accepts or rejects, or all traffic.")
                .with_provider_name("TrafficType"),
        )
        .with_validator(|attrs| {
            let mut errors = Vec::new();
            if let Err(mut e) = validate_tags_map(attrs) {
                errors.append(&mut e);
            }
            if errors.is_empty() { Ok(()) } else { Err(errors) }
        })
    }
}

/// Returns the resource type name and all enum valid values for this module
pub fn enum_valid_values() -> (
    &'static str,
    &'static [(&'static str, &'static [&'static str])],
) {
    (
        "ec2.FlowLog",
        &[
            ("log_destination_type", VALID_LOG_DESTINATION_TYPE),
            ("resource_type", VALID_RESOURCE_TYPE),
            ("traffic_type", VALID_TRAFFIC_TYPE),
        ],
    )
}
