//! schedule schema definition for AWS Cloud Control
//!
//! Auto-generated from CloudFormation schema: AWS::Scheduler::Schedule
//!
//! DO NOT EDIT MANUALLY - regenerate with carina-codegen

use crate::schemas::config::AwsccSchemaConfig;
use carina_core::schema::{AttributeSchema, AttributeType, ResourceSchema, StructField};

pub fn arn() -> AttributeType {
    carina_aws_types::arn()
}

const VALID_AWS_VPC_CONFIGURATION_ASSIGN_PUBLIC_IP: &[&str] = &["ENABLED", "DISABLED"];

const VALID_ECS_PARAMETERS_LAUNCH_TYPE: &[&str] = &["EC2", "FARGATE", "EXTERNAL"];

const VALID_ECS_PARAMETERS_PROPAGATE_TAGS: &[&str] = &["TASK_DEFINITION"];

const VALID_FLEXIBLE_TIME_WINDOW_MODE: &[&str] = &["OFF", "FLEXIBLE"];

const VALID_PLACEMENT_CONSTRAINT_TYPE: &[&str] = &["distinctInstance", "memberOf"];

const VALID_PLACEMENT_STRATEGY_FIELD: &[&str] = &["instanceId", "cpu", "memory"];

const VALID_PLACEMENT_STRATEGY_TYPE: &[&str] = &["random", "spread", "binpack"];

const VALID_STATE: &[&str] = &["ENABLED", "DISABLED"];

/// Returns the schema config for scheduler_schedule (AWS::Scheduler::Schedule)
#[rustfmt::skip]
pub fn scheduler_schedule_config() -> AwsccSchemaConfig {
    let mut schema = ResourceSchema::new("scheduler.Schedule")
        .with_description("Definition of AWS::Scheduler::Schedule Resource Type");
    #[inline(never)]
    fn build_arn_attribute() -> AttributeSchema {
        AttributeSchema::provider_populated("arn", self::arn())
                .with_description("The Amazon Resource Name (ARN) of the schedule. (read-only)")
                .with_provider_name("Arn")
            .build()
    }
    schema = schema.attribute(build_arn_attribute());
    #[inline(never)]
    fn build_description_attribute() -> AttributeSchema {
        AttributeSchema::optional("description", AttributeType::refined_string(None, None, Some((None, Some(512))), None))
                .with_description("The description of the schedule.")
                .with_provider_name("Description")
            .build()
    }
    schema = schema.attribute(build_description_attribute());
    #[inline(never)]
    fn build_end_date_attribute() -> AttributeSchema {
        AttributeSchema::optional("end_date", AttributeType::refined_string(None, None, None, None))
                .with_description("The date, in UTC, before which the schedule can invoke its target. Depending on the schedule's recurrence expression, invocations might stop on, or before, the EndDate you specify.")
                .with_provider_name("EndDate")
            .build()
    }
    schema = schema.attribute(build_end_date_attribute());
    #[inline(never)]
    fn build_flexible_time_window_attribute() -> AttributeSchema {
        AttributeSchema::required("flexible_time_window", AttributeType::struct_("FlexibleTimeWindow".to_string(), vec![StructField::optional("maximum_window_in_minutes", AttributeType::refined_int(None, Some((Some(1), Some(1440))))).with_description("The maximum time window during which a schedule can be invoked.").with_provider_name("MaximumWindowInMinutes").build(),
                    StructField::required("mode", AttributeType::enum_(carina_core::schema::enum_identity("FlexibleTimeWindowMode", Some("aws.scheduler.Schedule.FlexibleTimeWindow")), Some(vec!["OFF".to_string(), "FLEXIBLE".to_string()]), vec![("OFF".to_string(), "off".to_string()), ("FLEXIBLE".to_string(), "flexible".to_string())], None, None)).with_provider_name("Mode").build()]))
                .with_provider_name("FlexibleTimeWindow")
            .build()
    }
    schema = schema.attribute(build_flexible_time_window_attribute());
    #[inline(never)]
    fn build_group_name_attribute() -> AttributeSchema {
        AttributeSchema::optional("group_name", crate::schemas::catalog::scheduler_schedule_group_name())
                .with_description("The name of the schedule group to associate with this schedule. If you omit this, the default schedule group is used. Carina requires group_name to be omitted or set to the literal 'default'. Cloud Control identifies schedules by Name only and reads them in the default group, so a schedule in another group can be created but never read or tracked.")
                .with_provider_name("GroupName")
            .build()
    }
    schema = schema.attribute(build_group_name_attribute());
    #[inline(never)]
    fn build_kms_key_arn_attribute() -> AttributeSchema {
        AttributeSchema::optional("kms_key_arn", super::super::kms::key::arn())
                .with_description("The ARN for a KMS Key that will be used to encrypt customer data.")
                .with_provider_name("KmsKeyArn")
            .build()
    }
    schema = schema.attribute(build_kms_key_arn_attribute());
    #[inline(never)]
    fn build_name_attribute() -> AttributeSchema {
        AttributeSchema::optional("name", AttributeType::refined_string(None, Some("^[0-9a-zA-Z-_.]+$".to_string()), Some((Some(1), Some(64))), None))
                .create_only()
                .with_provider_name("Name")
            .build()
    }
    schema = schema.attribute(build_name_attribute());
    #[inline(never)]
    fn build_schedule_expression_attribute() -> AttributeSchema {
        AttributeSchema::required("schedule_expression", AttributeType::refined_string(None, None, Some((Some(1), Some(256))), None))
                .with_description("The scheduling expression.")
                .with_provider_name("ScheduleExpression")
            .build()
    }
    schema = schema.attribute(build_schedule_expression_attribute());
    #[inline(never)]
    fn build_schedule_expression_timezone_attribute() -> AttributeSchema {
        AttributeSchema::optional("schedule_expression_timezone", AttributeType::refined_string(None, None, Some((Some(1), Some(50))), None))
                .with_description("The timezone in which the scheduling expression is evaluated.")
                .with_provider_name("ScheduleExpressionTimezone")
            .build()
    }
    schema = schema.attribute(build_schedule_expression_timezone_attribute());
    #[inline(never)]
    fn build_start_date_attribute() -> AttributeSchema {
        AttributeSchema::optional("start_date", AttributeType::refined_string(None, None, None, None))
                .with_description("The date, in UTC, after which the schedule can begin invoking its target. Depending on the schedule's recurrence expression, invocations might occur on, or after, the StartDate you specify.")
                .with_provider_name("StartDate")
            .build()
    }
    schema = schema.attribute(build_start_date_attribute());
    #[inline(never)]
    fn build_state_attribute() -> AttributeSchema {
        AttributeSchema::optional("state", AttributeType::enum_(carina_core::schema::enum_identity("ScheduleState", Some("aws.scheduler.Schedule")), Some(vec!["ENABLED".to_string(), "DISABLED".to_string()]), vec![("ENABLED".to_string(), "enabled".to_string()), ("DISABLED".to_string(), "disabled".to_string())], None, None))
                .with_provider_name("State")
            .build()
    }
    schema = schema.attribute(build_state_attribute());
    #[inline(never)]
    fn build_target_attribute() -> AttributeSchema {
        AttributeSchema::required("target", AttributeType::struct_("Target".to_string(), vec![StructField::required("arn", carina_aws_types::arn()).with_description("The Amazon Resource Name (ARN) of the target.").with_provider_name("Arn").build(),
                    StructField::optional("dead_letter_config", AttributeType::struct_("DeadLetterConfig".to_string(), vec![StructField::optional("arn", carina_aws_types::arn()).with_description("The ARN of the SQS queue specified as the target for the dead-letter queue.").with_provider_name("Arn").build()])).with_provider_name("DeadLetterConfig").build(),
                    StructField::optional("ecs_parameters", AttributeType::struct_("EcsParameters".to_string(), vec![StructField::optional("capacity_provider_strategy", AttributeType::refined_list(AttributeType::struct_("CapacityProviderStrategyItem".to_string(), vec![StructField::optional("base", AttributeType::refined_int(None, Some((Some(0), Some(100000))))).with_description("The base value designates how many tasks, at a minimum, to run on the specified capacity provider. Only one capacity provider in a capacity provider strategy can have a base defined. If no value is specified, the default value of 0 is used.").with_provider_name("Base").build(),
                    StructField::required("capacity_provider", AttributeType::refined_string(None, None, Some((Some(1), Some(255))), None)).with_description("The short name of the capacity provider.").with_provider_name("CapacityProvider").build(),
                    StructField::optional("weight", AttributeType::refined_int(None, Some((Some(0), Some(1000))))).with_description("The weight value designates the relative percentage of the total number of tasks launched that should use the specified capacity provider. The weight value is taken into consideration after the base value, if defined, is satisfied.").with_provider_name("Weight").build()]), false, Some((None, Some(6))))).with_description("The capacity provider strategy to use for the task.").with_provider_name("CapacityProviderStrategy").with_block_name("capacity_provider_strategy").build(),
                    StructField::optional("enable_ecs_managed_tags", AttributeType::bool()).with_description("Specifies whether to enable Amazon ECS managed tags for the task. For more information, see Tagging Your Amazon ECS Resources in the Amazon Elastic Container Service Developer Guide.").with_provider_name("EnableECSManagedTags").build(),
                    StructField::optional("enable_execute_command", AttributeType::bool()).with_description("Whether or not to enable the execute command functionality for the containers in this task. If true, this enables execute command functionality on all containers in the task.").with_provider_name("EnableExecuteCommand").build(),
                    StructField::optional("group", AttributeType::refined_string(None, None, Some((Some(1), Some(255))), None)).with_description("Specifies an ECS task group for the task. The maximum length is 255 characters.").with_provider_name("Group").build(),
                    StructField::optional("launch_type", AttributeType::enum_(carina_core::schema::enum_identity("LaunchType", Some("aws.scheduler.Schedule.Target.EcsParameters")), Some(vec!["EC2".to_string(), "FARGATE".to_string(), "EXTERNAL".to_string()]), vec![("EC2".to_string(), "ec2".to_string()), ("FARGATE".to_string(), "fargate".to_string()), ("EXTERNAL".to_string(), "external".to_string())], None, None)).with_provider_name("LaunchType").build(),
                    StructField::optional("network_configuration", AttributeType::struct_("NetworkConfiguration".to_string(), vec![StructField::optional("awsvpc_configuration", AttributeType::struct_("AwsVpcConfiguration".to_string(), vec![StructField::optional("assign_public_ip", AttributeType::enum_(carina_core::schema::enum_identity("AssignPublicIp", Some("aws.scheduler.Schedule.Target.EcsParameters.NetworkConfiguration.AwsVpcConfiguration")), Some(vec!["ENABLED".to_string(), "DISABLED".to_string()]), vec![("ENABLED".to_string(), "enabled".to_string()), ("DISABLED".to_string(), "disabled".to_string())], None, None)).with_provider_name("AssignPublicIp").build(),
                    StructField::optional("security_groups", AttributeType::refined_list(AttributeType::refined_string(None, None, Some((Some(1), Some(1000))), None), false, Some((Some(1), Some(5))))).with_description("Specifies the security groups associated with the task. These security groups must all be in the same VPC. You can specify as many as five security groups. If you do not specify a security group, the default security group for the VPC is used.").with_provider_name("SecurityGroups").build(),
                    StructField::required("subnets", AttributeType::refined_list(AttributeType::refined_string(None, None, Some((Some(1), Some(1000))), None), false, Some((Some(1), Some(16))))).with_description("Specifies the subnets associated with the task. These subnets must all be in the same VPC. You can specify as many as 16 subnets.").with_provider_name("Subnets").build()])).with_provider_name("AwsvpcConfiguration").build()])).with_provider_name("NetworkConfiguration").build(),
                    StructField::optional("placement_constraints", AttributeType::refined_list(AttributeType::struct_("PlacementConstraint".to_string(), vec![StructField::optional("expression", AttributeType::refined_string(None, None, Some((None, Some(2000))), None)).with_description("A cluster query language expression to apply to the constraint. You cannot specify an expression if the constraint type is distinctInstance. To learn more, see Cluster Query Language in the Amazon Elastic Container Service Developer Guide.").with_provider_name("Expression").build(),
                    StructField::optional("type", AttributeType::enum_(carina_core::schema::enum_identity("PlacementConstraintType", Some("aws.scheduler.Schedule.Target.EcsParameters.PlacementConstraint")), Some(vec!["distinctInstance".to_string(), "memberOf".to_string()]), vec![("distinctInstance".to_string(), "distinct_instance".to_string()), ("memberOf".to_string(), "member_of".to_string())], None, None)).with_provider_name("Type").build()]), false, Some((None, Some(10))))).with_description("An array of placement constraint objects to use for the task. You can specify up to 10 constraints per task (including constraints in the task definition and those specified at runtime).").with_provider_name("PlacementConstraints").with_block_name("placement_constraint").build(),
                    StructField::optional("placement_strategy", AttributeType::refined_list(AttributeType::struct_("PlacementStrategy".to_string(), vec![StructField::optional("field", AttributeType::enum_(carina_core::schema::enum_identity("Field", Some("aws.scheduler.Schedule.Target.EcsParameters.PlacementStrategy")), Some(vec!["instanceId".to_string(), "cpu".to_string(), "memory".to_string()]), vec![("instanceId".to_string(), "instance_id".to_string()), ("cpu".to_string(), "cpu".to_string()), ("memory".to_string(), "memory".to_string())], None, None)).with_description("The field to apply the placement strategy against. For the spread placement strategy, valid values are instanceId (or host, which has the same effect), or any platform or custom attribute that is applied to a container instance, such as attribute:ecs.availability-zone. For the binpack placement strategy, valid values are cpu and memory. For the random placement strategy, this field is not used.").with_provider_name("Field").build(),
                    StructField::optional("type", AttributeType::enum_(carina_core::schema::enum_identity("PlacementStrategyType", Some("aws.scheduler.Schedule.Target.EcsParameters.PlacementStrategy")), Some(vec!["random".to_string(), "spread".to_string(), "binpack".to_string()]), vec![("random".to_string(), "random".to_string()), ("spread".to_string(), "spread".to_string()), ("binpack".to_string(), "binpack".to_string())], None, None)).with_provider_name("Type").build()]), false, Some((None, Some(5))))).with_description("The placement strategy objects to use for the task. You can specify a maximum of five strategy rules per task.").with_provider_name("PlacementStrategy").with_block_name("placement_strategy").build(),
                    StructField::optional("platform_version", AttributeType::refined_string(None, None, Some((Some(1), Some(64))), None)).with_description("Specifies the platform version for the task. Specify only the numeric portion of the platform version, such as 1.1.0.").with_provider_name("PlatformVersion").build(),
                    StructField::optional("propagate_tags", AttributeType::enum_(carina_core::schema::enum_identity("PropagateTags", Some("aws.scheduler.Schedule.Target.EcsParameters")), Some(vec!["TASK_DEFINITION".to_string()]), vec![("TASK_DEFINITION".to_string(), "task_definition".to_string())], None, None)).with_provider_name("PropagateTags").build(),
                    StructField::optional("reference_id", AttributeType::refined_string(None, None, Some((None, Some(1024))), None)).with_description("The reference ID to use for the task.").with_provider_name("ReferenceId").build(),
                    StructField::optional("tags", AttributeType::refined_list(carina_aws_types::tags_type(), false, Some((None, Some(50))))).with_description("The metadata that you apply to the task to help you categorize and organize them. Each tag consists of a key and an optional value, both of which you define. To learn more, see RunTask in the Amazon ECS API Reference.").with_provider_name("Tags").build(),
                    StructField::optional("task_count", AttributeType::refined_int(None, Some((Some(1), Some(10))))).with_description("The number of tasks to create based on TaskDefinition. The default is 1.").with_provider_name("TaskCount").build(),
                    StructField::required("task_definition_arn", carina_aws_types::arn()).with_description("The ARN of the task definition to use if the event target is an Amazon ECS task.").with_provider_name("TaskDefinitionArn").build()])).with_provider_name("EcsParameters").with_block_name("ecs_parameter").build(),
                    StructField::optional("event_bridge_parameters", AttributeType::struct_("EventBridgeParameters".to_string(), vec![StructField::required("detail_type", AttributeType::refined_string(None, None, Some((Some(1), Some(128))), None)).with_description("Free-form string, with a maximum of 128 characters, used to decide what fields to expect in the event detail.").with_provider_name("DetailType").build(),
                    StructField::required("source", AttributeType::refined_string(None, Some("^(?=[/\\.\\-_A-Za-z0-9]+)((?!aws\\.).*)|(\\$(\\.[\\w_-]+(\\[(\\d+|\\*)\\])*)*)$".to_string()), Some((Some(1), Some(256))), None)).with_description("The source of the event.").with_provider_name("Source").build()])).with_provider_name("EventBridgeParameters").build(),
                    StructField::optional("input", AttributeType::refined_string(None, None, Some((Some(1), None)), None)).with_description("The text, or well-formed JSON, passed to the target. If you are configuring a templated Lambda, AWS Step Functions, or Amazon EventBridge target, the input must be a well-formed JSON. For all other target types, a JSON is not required. If you do not specify anything for this field, EventBridge Scheduler delivers a default notification to the target.").with_provider_name("Input").build(),
                    StructField::optional("kinesis_parameters", AttributeType::struct_("KinesisParameters".to_string(), vec![StructField::required("partition_key", AttributeType::refined_string(None, None, Some((Some(1), Some(256))), None)).with_description("The custom parameter used as the Kinesis partition key. For more information, see Amazon Kinesis Streams Key Concepts in the Amazon Kinesis Streams Developer Guide.").with_provider_name("PartitionKey").build()])).with_provider_name("KinesisParameters").build(),
                    StructField::optional("retry_policy", AttributeType::struct_("RetryPolicy".to_string(), vec![StructField::optional("maximum_event_age_in_seconds", AttributeType::refined_int(None, Some((Some(60), Some(86400))))).with_description("The maximum amount of time, in seconds, to continue to make retry attempts.").with_provider_name("MaximumEventAgeInSeconds").build(),
                    StructField::optional("maximum_retry_attempts", AttributeType::refined_int(None, Some((Some(0), Some(185))))).with_description("The maximum number of retry attempts to make before the request fails. Retry attempts with exponential backoff continue until either the maximum number of attempts is made or until the duration of the MaximumEventAgeInSeconds is reached.").with_provider_name("MaximumRetryAttempts").build()])).with_provider_name("RetryPolicy").build(),
                    StructField::required("role_arn", carina_aws_types::arn()).with_description("The Amazon Resource Name (ARN) of the IAM role to be used for this target when the schedule is triggered.").with_provider_name("RoleArn").build(),
                    StructField::optional("sage_maker_pipeline_parameters", AttributeType::struct_("SageMakerPipelineParameters".to_string(), vec![StructField::optional("pipeline_parameter_list", AttributeType::refined_list(AttributeType::struct_("SageMakerPipelineParameter".to_string(), vec![StructField::required("name", AttributeType::refined_string(None, Some("^[A-Za-z0-9\\-_]*$".to_string()), Some((Some(1), Some(256))), None)).with_description("Name of parameter to start execution of a SageMaker Model Building Pipeline.").with_provider_name("Name").build(),
                    StructField::required("value", AttributeType::refined_string(None, None, Some((Some(1), Some(1024))), None)).with_description("Value of parameter to start execution of a SageMaker Model Building Pipeline.").with_provider_name("Value").build()]), false, Some((None, Some(200))))).with_description("List of Parameter names and values for SageMaker Model Building Pipeline execution.").with_provider_name("PipelineParameterList").with_block_name("pipeline_parameter_list").build()])).with_provider_name("SageMakerPipelineParameters").with_block_name("sage_maker_pipeline_parameter").build(),
                    StructField::optional("sqs_parameters", AttributeType::struct_("SqsParameters".to_string(), vec![StructField::optional("message_group_id", AttributeType::refined_string(None, None, Some((Some(1), Some(128))), None)).with_description("The FIFO message group ID to use as the target.").with_provider_name("MessageGroupId").build()])).with_provider_name("SqsParameters").build()]))
                .with_provider_name("Target")
                .with_block_name("target")
            .build()
    }
    schema = schema.attribute(build_target_attribute());
    schema = schema
        .with_unique_name_attribute("name");
    #[inline(never)]
    fn build_schema_def_0() -> AttributeType { AttributeType::struct_("AwsVpcConfiguration".to_string(), vec![StructField::optional("assign_public_ip", AttributeType::enum_(carina_core::schema::enum_identity("AssignPublicIp", Some("aws.scheduler.Schedule.Target.EcsParameters.NetworkConfiguration.AwsVpcConfiguration")), Some(vec!["ENABLED".to_string(), "DISABLED".to_string()]), vec![("ENABLED".to_string(), "enabled".to_string()), ("DISABLED".to_string(), "disabled".to_string())], None, None)).with_provider_name("AssignPublicIp").build(),
                    StructField::optional("security_groups", AttributeType::refined_list(AttributeType::refined_string(None, None, Some((Some(1), Some(1000))), None), false, Some((Some(1), Some(5))))).with_description("Specifies the security groups associated with the task. These security groups must all be in the same VPC. You can specify as many as five security groups. If you do not specify a security group, the default security group for the VPC is used.").with_provider_name("SecurityGroups").build(),
                    StructField::required("subnets", AttributeType::refined_list(AttributeType::refined_string(None, None, Some((Some(1), Some(1000))), None), false, Some((Some(1), Some(16))))).with_description("Specifies the subnets associated with the task. These subnets must all be in the same VPC. You can specify as many as 16 subnets.").with_provider_name("Subnets").build()]) }
    schema = schema.with_def("AwsVpcConfiguration", build_schema_def_0());
    #[inline(never)]
    fn build_schema_def_1() -> AttributeType { AttributeType::struct_("DeadLetterConfig".to_string(), vec![StructField::optional("arn", carina_aws_types::arn()).with_description("The ARN of the SQS queue specified as the target for the dead-letter queue.").with_provider_name("Arn").build()]) }
    schema = schema.with_def("DeadLetterConfig", build_schema_def_1());
    #[inline(never)]
    fn build_schema_def_2() -> AttributeType { AttributeType::struct_("EcsParameters".to_string(), vec![StructField::optional("capacity_provider_strategy", AttributeType::refined_list(AttributeType::struct_("CapacityProviderStrategyItem".to_string(), vec![StructField::optional("base", AttributeType::refined_int(None, Some((Some(0), Some(100000))))).with_description("The base value designates how many tasks, at a minimum, to run on the specified capacity provider. Only one capacity provider in a capacity provider strategy can have a base defined. If no value is specified, the default value of 0 is used.").with_provider_name("Base").build(),
                    StructField::required("capacity_provider", AttributeType::refined_string(None, None, Some((Some(1), Some(255))), None)).with_description("The short name of the capacity provider.").with_provider_name("CapacityProvider").build(),
                    StructField::optional("weight", AttributeType::refined_int(None, Some((Some(0), Some(1000))))).with_description("The weight value designates the relative percentage of the total number of tasks launched that should use the specified capacity provider. The weight value is taken into consideration after the base value, if defined, is satisfied.").with_provider_name("Weight").build()]), false, Some((None, Some(6))))).with_description("The capacity provider strategy to use for the task.").with_provider_name("CapacityProviderStrategy").with_block_name("capacity_provider_strategy").build(),
                    StructField::optional("enable_ecs_managed_tags", AttributeType::bool()).with_description("Specifies whether to enable Amazon ECS managed tags for the task. For more information, see Tagging Your Amazon ECS Resources in the Amazon Elastic Container Service Developer Guide.").with_provider_name("EnableECSManagedTags").build(),
                    StructField::optional("enable_execute_command", AttributeType::bool()).with_description("Whether or not to enable the execute command functionality for the containers in this task. If true, this enables execute command functionality on all containers in the task.").with_provider_name("EnableExecuteCommand").build(),
                    StructField::optional("group", AttributeType::refined_string(None, None, Some((Some(1), Some(255))), None)).with_description("Specifies an ECS task group for the task. The maximum length is 255 characters.").with_provider_name("Group").build(),
                    StructField::optional("launch_type", AttributeType::enum_(carina_core::schema::enum_identity("LaunchType", Some("aws.scheduler.Schedule.Target.EcsParameters")), Some(vec!["EC2".to_string(), "FARGATE".to_string(), "EXTERNAL".to_string()]), vec![("EC2".to_string(), "ec2".to_string()), ("FARGATE".to_string(), "fargate".to_string()), ("EXTERNAL".to_string(), "external".to_string())], None, None)).with_provider_name("LaunchType").build(),
                    StructField::optional("network_configuration", AttributeType::struct_("NetworkConfiguration".to_string(), vec![StructField::optional("awsvpc_configuration", AttributeType::struct_("AwsVpcConfiguration".to_string(), vec![StructField::optional("assign_public_ip", AttributeType::enum_(carina_core::schema::enum_identity("AssignPublicIp", Some("aws.scheduler.Schedule.Target.EcsParameters.NetworkConfiguration.AwsVpcConfiguration")), Some(vec!["ENABLED".to_string(), "DISABLED".to_string()]), vec![("ENABLED".to_string(), "enabled".to_string()), ("DISABLED".to_string(), "disabled".to_string())], None, None)).with_provider_name("AssignPublicIp").build(),
                    StructField::optional("security_groups", AttributeType::refined_list(AttributeType::refined_string(None, None, Some((Some(1), Some(1000))), None), false, Some((Some(1), Some(5))))).with_description("Specifies the security groups associated with the task. These security groups must all be in the same VPC. You can specify as many as five security groups. If you do not specify a security group, the default security group for the VPC is used.").with_provider_name("SecurityGroups").build(),
                    StructField::required("subnets", AttributeType::refined_list(AttributeType::refined_string(None, None, Some((Some(1), Some(1000))), None), false, Some((Some(1), Some(16))))).with_description("Specifies the subnets associated with the task. These subnets must all be in the same VPC. You can specify as many as 16 subnets.").with_provider_name("Subnets").build()])).with_provider_name("AwsvpcConfiguration").build()])).with_provider_name("NetworkConfiguration").build(),
                    StructField::optional("placement_constraints", AttributeType::refined_list(AttributeType::struct_("PlacementConstraint".to_string(), vec![StructField::optional("expression", AttributeType::refined_string(None, None, Some((None, Some(2000))), None)).with_description("A cluster query language expression to apply to the constraint. You cannot specify an expression if the constraint type is distinctInstance. To learn more, see Cluster Query Language in the Amazon Elastic Container Service Developer Guide.").with_provider_name("Expression").build(),
                    StructField::optional("type", AttributeType::enum_(carina_core::schema::enum_identity("PlacementConstraintType", Some("aws.scheduler.Schedule.Target.EcsParameters.PlacementConstraint")), Some(vec!["distinctInstance".to_string(), "memberOf".to_string()]), vec![("distinctInstance".to_string(), "distinct_instance".to_string()), ("memberOf".to_string(), "member_of".to_string())], None, None)).with_provider_name("Type").build()]), false, Some((None, Some(10))))).with_description("An array of placement constraint objects to use for the task. You can specify up to 10 constraints per task (including constraints in the task definition and those specified at runtime).").with_provider_name("PlacementConstraints").with_block_name("placement_constraint").build(),
                    StructField::optional("placement_strategy", AttributeType::refined_list(AttributeType::struct_("PlacementStrategy".to_string(), vec![StructField::optional("field", AttributeType::enum_(carina_core::schema::enum_identity("Field", Some("aws.scheduler.Schedule.Target.EcsParameters.PlacementStrategy")), Some(vec!["instanceId".to_string(), "cpu".to_string(), "memory".to_string()]), vec![("instanceId".to_string(), "instance_id".to_string()), ("cpu".to_string(), "cpu".to_string()), ("memory".to_string(), "memory".to_string())], None, None)).with_description("The field to apply the placement strategy against. For the spread placement strategy, valid values are instanceId (or host, which has the same effect), or any platform or custom attribute that is applied to a container instance, such as attribute:ecs.availability-zone. For the binpack placement strategy, valid values are cpu and memory. For the random placement strategy, this field is not used.").with_provider_name("Field").build(),
                    StructField::optional("type", AttributeType::enum_(carina_core::schema::enum_identity("PlacementStrategyType", Some("aws.scheduler.Schedule.Target.EcsParameters.PlacementStrategy")), Some(vec!["random".to_string(), "spread".to_string(), "binpack".to_string()]), vec![("random".to_string(), "random".to_string()), ("spread".to_string(), "spread".to_string()), ("binpack".to_string(), "binpack".to_string())], None, None)).with_provider_name("Type").build()]), false, Some((None, Some(5))))).with_description("The placement strategy objects to use for the task. You can specify a maximum of five strategy rules per task.").with_provider_name("PlacementStrategy").with_block_name("placement_strategy").build(),
                    StructField::optional("platform_version", AttributeType::refined_string(None, None, Some((Some(1), Some(64))), None)).with_description("Specifies the platform version for the task. Specify only the numeric portion of the platform version, such as 1.1.0.").with_provider_name("PlatformVersion").build(),
                    StructField::optional("propagate_tags", AttributeType::enum_(carina_core::schema::enum_identity("PropagateTags", Some("aws.scheduler.Schedule.Target.EcsParameters")), Some(vec!["TASK_DEFINITION".to_string()]), vec![("TASK_DEFINITION".to_string(), "task_definition".to_string())], None, None)).with_provider_name("PropagateTags").build(),
                    StructField::optional("reference_id", AttributeType::refined_string(None, None, Some((None, Some(1024))), None)).with_description("The reference ID to use for the task.").with_provider_name("ReferenceId").build(),
                    StructField::optional("tags", AttributeType::refined_list(carina_aws_types::tags_type(), false, Some((None, Some(50))))).with_description("The metadata that you apply to the task to help you categorize and organize them. Each tag consists of a key and an optional value, both of which you define. To learn more, see RunTask in the Amazon ECS API Reference.").with_provider_name("Tags").build(),
                    StructField::optional("task_count", AttributeType::refined_int(None, Some((Some(1), Some(10))))).with_description("The number of tasks to create based on TaskDefinition. The default is 1.").with_provider_name("TaskCount").build(),
                    StructField::required("task_definition_arn", carina_aws_types::arn()).with_description("The ARN of the task definition to use if the event target is an Amazon ECS task.").with_provider_name("TaskDefinitionArn").build()]) }
    schema = schema.with_def("EcsParameters", build_schema_def_2());
    #[inline(never)]
    fn build_schema_def_3() -> AttributeType { AttributeType::struct_("EventBridgeParameters".to_string(), vec![StructField::required("detail_type", AttributeType::refined_string(None, None, Some((Some(1), Some(128))), None)).with_description("Free-form string, with a maximum of 128 characters, used to decide what fields to expect in the event detail.").with_provider_name("DetailType").build(),
                    StructField::required("source", AttributeType::refined_string(None, Some("^(?=[/\\.\\-_A-Za-z0-9]+)((?!aws\\.).*)|(\\$(\\.[\\w_-]+(\\[(\\d+|\\*)\\])*)*)$".to_string()), Some((Some(1), Some(256))), None)).with_description("The source of the event.").with_provider_name("Source").build()]) }
    schema = schema.with_def("EventBridgeParameters", build_schema_def_3());
    #[inline(never)]
    fn build_schema_def_4() -> AttributeType { AttributeType::struct_("FlexibleTimeWindow".to_string(), vec![StructField::optional("maximum_window_in_minutes", AttributeType::refined_int(None, Some((Some(1), Some(1440))))).with_description("The maximum time window during which a schedule can be invoked.").with_provider_name("MaximumWindowInMinutes").build(),
                    StructField::required("mode", AttributeType::enum_(carina_core::schema::enum_identity("FlexibleTimeWindowMode", Some("aws.scheduler.Schedule.FlexibleTimeWindow")), Some(vec!["OFF".to_string(), "FLEXIBLE".to_string()]), vec![("OFF".to_string(), "off".to_string()), ("FLEXIBLE".to_string(), "flexible".to_string())], None, None)).with_provider_name("Mode").build()]) }
    schema = schema.with_def("FlexibleTimeWindow", build_schema_def_4());
    #[inline(never)]
    fn build_schema_def_5() -> AttributeType { AttributeType::struct_("KinesisParameters".to_string(), vec![StructField::required("partition_key", AttributeType::refined_string(None, None, Some((Some(1), Some(256))), None)).with_description("The custom parameter used as the Kinesis partition key. For more information, see Amazon Kinesis Streams Key Concepts in the Amazon Kinesis Streams Developer Guide.").with_provider_name("PartitionKey").build()]) }
    schema = schema.with_def("KinesisParameters", build_schema_def_5());
    #[inline(never)]
    fn build_schema_def_6() -> AttributeType { AttributeType::struct_("NetworkConfiguration".to_string(), vec![StructField::optional("awsvpc_configuration", AttributeType::struct_("AwsVpcConfiguration".to_string(), vec![StructField::optional("assign_public_ip", AttributeType::enum_(carina_core::schema::enum_identity("AssignPublicIp", Some("aws.scheduler.Schedule.Target.EcsParameters.NetworkConfiguration.AwsVpcConfiguration")), Some(vec!["ENABLED".to_string(), "DISABLED".to_string()]), vec![("ENABLED".to_string(), "enabled".to_string()), ("DISABLED".to_string(), "disabled".to_string())], None, None)).with_provider_name("AssignPublicIp").build(),
                    StructField::optional("security_groups", AttributeType::refined_list(AttributeType::refined_string(None, None, Some((Some(1), Some(1000))), None), false, Some((Some(1), Some(5))))).with_description("Specifies the security groups associated with the task. These security groups must all be in the same VPC. You can specify as many as five security groups. If you do not specify a security group, the default security group for the VPC is used.").with_provider_name("SecurityGroups").build(),
                    StructField::required("subnets", AttributeType::refined_list(AttributeType::refined_string(None, None, Some((Some(1), Some(1000))), None), false, Some((Some(1), Some(16))))).with_description("Specifies the subnets associated with the task. These subnets must all be in the same VPC. You can specify as many as 16 subnets.").with_provider_name("Subnets").build()])).with_provider_name("AwsvpcConfiguration").build()]) }
    schema = schema.with_def("NetworkConfiguration", build_schema_def_6());
    #[inline(never)]
    fn build_schema_def_7() -> AttributeType { AttributeType::struct_("RetryPolicy".to_string(), vec![StructField::optional("maximum_event_age_in_seconds", AttributeType::refined_int(None, Some((Some(60), Some(86400))))).with_description("The maximum amount of time, in seconds, to continue to make retry attempts.").with_provider_name("MaximumEventAgeInSeconds").build(),
                    StructField::optional("maximum_retry_attempts", AttributeType::refined_int(None, Some((Some(0), Some(185))))).with_description("The maximum number of retry attempts to make before the request fails. Retry attempts with exponential backoff continue until either the maximum number of attempts is made or until the duration of the MaximumEventAgeInSeconds is reached.").with_provider_name("MaximumRetryAttempts").build()]) }
    schema = schema.with_def("RetryPolicy", build_schema_def_7());
    #[inline(never)]
    fn build_schema_def_8() -> AttributeType { AttributeType::struct_("SageMakerPipelineParameters".to_string(), vec![StructField::optional("pipeline_parameter_list", AttributeType::refined_list(AttributeType::struct_("SageMakerPipelineParameter".to_string(), vec![StructField::required("name", AttributeType::refined_string(None, Some("^[A-Za-z0-9\\-_]*$".to_string()), Some((Some(1), Some(256))), None)).with_description("Name of parameter to start execution of a SageMaker Model Building Pipeline.").with_provider_name("Name").build(),
                    StructField::required("value", AttributeType::refined_string(None, None, Some((Some(1), Some(1024))), None)).with_description("Value of parameter to start execution of a SageMaker Model Building Pipeline.").with_provider_name("Value").build()]), false, Some((None, Some(200))))).with_description("List of Parameter names and values for SageMaker Model Building Pipeline execution.").with_provider_name("PipelineParameterList").with_block_name("pipeline_parameter_list").build()]) }
    schema = schema.with_def("SageMakerPipelineParameters", build_schema_def_8());
    #[inline(never)]
    fn build_schema_def_9() -> AttributeType { AttributeType::struct_("SqsParameters".to_string(), vec![StructField::optional("message_group_id", AttributeType::refined_string(None, None, Some((Some(1), Some(128))), None)).with_description("The FIFO message group ID to use as the target.").with_provider_name("MessageGroupId").build()]) }
    schema = schema.with_def("SqsParameters", build_schema_def_9());
    #[inline(never)]
    fn build_schema_def_10() -> AttributeType { AttributeType::struct_("Target".to_string(), vec![StructField::required("arn", carina_aws_types::arn()).with_description("The Amazon Resource Name (ARN) of the target.").with_provider_name("Arn").build(),
                    StructField::optional("dead_letter_config", AttributeType::struct_("DeadLetterConfig".to_string(), vec![StructField::optional("arn", carina_aws_types::arn()).with_description("The ARN of the SQS queue specified as the target for the dead-letter queue.").with_provider_name("Arn").build()])).with_provider_name("DeadLetterConfig").build(),
                    StructField::optional("ecs_parameters", AttributeType::struct_("EcsParameters".to_string(), vec![StructField::optional("capacity_provider_strategy", AttributeType::refined_list(AttributeType::struct_("CapacityProviderStrategyItem".to_string(), vec![StructField::optional("base", AttributeType::refined_int(None, Some((Some(0), Some(100000))))).with_description("The base value designates how many tasks, at a minimum, to run on the specified capacity provider. Only one capacity provider in a capacity provider strategy can have a base defined. If no value is specified, the default value of 0 is used.").with_provider_name("Base").build(),
                    StructField::required("capacity_provider", AttributeType::refined_string(None, None, Some((Some(1), Some(255))), None)).with_description("The short name of the capacity provider.").with_provider_name("CapacityProvider").build(),
                    StructField::optional("weight", AttributeType::refined_int(None, Some((Some(0), Some(1000))))).with_description("The weight value designates the relative percentage of the total number of tasks launched that should use the specified capacity provider. The weight value is taken into consideration after the base value, if defined, is satisfied.").with_provider_name("Weight").build()]), false, Some((None, Some(6))))).with_description("The capacity provider strategy to use for the task.").with_provider_name("CapacityProviderStrategy").with_block_name("capacity_provider_strategy").build(),
                    StructField::optional("enable_ecs_managed_tags", AttributeType::bool()).with_description("Specifies whether to enable Amazon ECS managed tags for the task. For more information, see Tagging Your Amazon ECS Resources in the Amazon Elastic Container Service Developer Guide.").with_provider_name("EnableECSManagedTags").build(),
                    StructField::optional("enable_execute_command", AttributeType::bool()).with_description("Whether or not to enable the execute command functionality for the containers in this task. If true, this enables execute command functionality on all containers in the task.").with_provider_name("EnableExecuteCommand").build(),
                    StructField::optional("group", AttributeType::refined_string(None, None, Some((Some(1), Some(255))), None)).with_description("Specifies an ECS task group for the task. The maximum length is 255 characters.").with_provider_name("Group").build(),
                    StructField::optional("launch_type", AttributeType::enum_(carina_core::schema::enum_identity("LaunchType", Some("aws.scheduler.Schedule.Target.EcsParameters")), Some(vec!["EC2".to_string(), "FARGATE".to_string(), "EXTERNAL".to_string()]), vec![("EC2".to_string(), "ec2".to_string()), ("FARGATE".to_string(), "fargate".to_string()), ("EXTERNAL".to_string(), "external".to_string())], None, None)).with_provider_name("LaunchType").build(),
                    StructField::optional("network_configuration", AttributeType::struct_("NetworkConfiguration".to_string(), vec![StructField::optional("awsvpc_configuration", AttributeType::struct_("AwsVpcConfiguration".to_string(), vec![StructField::optional("assign_public_ip", AttributeType::enum_(carina_core::schema::enum_identity("AssignPublicIp", Some("aws.scheduler.Schedule.Target.EcsParameters.NetworkConfiguration.AwsVpcConfiguration")), Some(vec!["ENABLED".to_string(), "DISABLED".to_string()]), vec![("ENABLED".to_string(), "enabled".to_string()), ("DISABLED".to_string(), "disabled".to_string())], None, None)).with_provider_name("AssignPublicIp").build(),
                    StructField::optional("security_groups", AttributeType::refined_list(AttributeType::refined_string(None, None, Some((Some(1), Some(1000))), None), false, Some((Some(1), Some(5))))).with_description("Specifies the security groups associated with the task. These security groups must all be in the same VPC. You can specify as many as five security groups. If you do not specify a security group, the default security group for the VPC is used.").with_provider_name("SecurityGroups").build(),
                    StructField::required("subnets", AttributeType::refined_list(AttributeType::refined_string(None, None, Some((Some(1), Some(1000))), None), false, Some((Some(1), Some(16))))).with_description("Specifies the subnets associated with the task. These subnets must all be in the same VPC. You can specify as many as 16 subnets.").with_provider_name("Subnets").build()])).with_provider_name("AwsvpcConfiguration").build()])).with_provider_name("NetworkConfiguration").build(),
                    StructField::optional("placement_constraints", AttributeType::refined_list(AttributeType::struct_("PlacementConstraint".to_string(), vec![StructField::optional("expression", AttributeType::refined_string(None, None, Some((None, Some(2000))), None)).with_description("A cluster query language expression to apply to the constraint. You cannot specify an expression if the constraint type is distinctInstance. To learn more, see Cluster Query Language in the Amazon Elastic Container Service Developer Guide.").with_provider_name("Expression").build(),
                    StructField::optional("type", AttributeType::enum_(carina_core::schema::enum_identity("PlacementConstraintType", Some("aws.scheduler.Schedule.Target.EcsParameters.PlacementConstraint")), Some(vec!["distinctInstance".to_string(), "memberOf".to_string()]), vec![("distinctInstance".to_string(), "distinct_instance".to_string()), ("memberOf".to_string(), "member_of".to_string())], None, None)).with_provider_name("Type").build()]), false, Some((None, Some(10))))).with_description("An array of placement constraint objects to use for the task. You can specify up to 10 constraints per task (including constraints in the task definition and those specified at runtime).").with_provider_name("PlacementConstraints").with_block_name("placement_constraint").build(),
                    StructField::optional("placement_strategy", AttributeType::refined_list(AttributeType::struct_("PlacementStrategy".to_string(), vec![StructField::optional("field", AttributeType::enum_(carina_core::schema::enum_identity("Field", Some("aws.scheduler.Schedule.Target.EcsParameters.PlacementStrategy")), Some(vec!["instanceId".to_string(), "cpu".to_string(), "memory".to_string()]), vec![("instanceId".to_string(), "instance_id".to_string()), ("cpu".to_string(), "cpu".to_string()), ("memory".to_string(), "memory".to_string())], None, None)).with_description("The field to apply the placement strategy against. For the spread placement strategy, valid values are instanceId (or host, which has the same effect), or any platform or custom attribute that is applied to a container instance, such as attribute:ecs.availability-zone. For the binpack placement strategy, valid values are cpu and memory. For the random placement strategy, this field is not used.").with_provider_name("Field").build(),
                    StructField::optional("type", AttributeType::enum_(carina_core::schema::enum_identity("PlacementStrategyType", Some("aws.scheduler.Schedule.Target.EcsParameters.PlacementStrategy")), Some(vec!["random".to_string(), "spread".to_string(), "binpack".to_string()]), vec![("random".to_string(), "random".to_string()), ("spread".to_string(), "spread".to_string()), ("binpack".to_string(), "binpack".to_string())], None, None)).with_provider_name("Type").build()]), false, Some((None, Some(5))))).with_description("The placement strategy objects to use for the task. You can specify a maximum of five strategy rules per task.").with_provider_name("PlacementStrategy").with_block_name("placement_strategy").build(),
                    StructField::optional("platform_version", AttributeType::refined_string(None, None, Some((Some(1), Some(64))), None)).with_description("Specifies the platform version for the task. Specify only the numeric portion of the platform version, such as 1.1.0.").with_provider_name("PlatformVersion").build(),
                    StructField::optional("propagate_tags", AttributeType::enum_(carina_core::schema::enum_identity("PropagateTags", Some("aws.scheduler.Schedule.Target.EcsParameters")), Some(vec!["TASK_DEFINITION".to_string()]), vec![("TASK_DEFINITION".to_string(), "task_definition".to_string())], None, None)).with_provider_name("PropagateTags").build(),
                    StructField::optional("reference_id", AttributeType::refined_string(None, None, Some((None, Some(1024))), None)).with_description("The reference ID to use for the task.").with_provider_name("ReferenceId").build(),
                    StructField::optional("tags", AttributeType::refined_list(carina_aws_types::tags_type(), false, Some((None, Some(50))))).with_description("The metadata that you apply to the task to help you categorize and organize them. Each tag consists of a key and an optional value, both of which you define. To learn more, see RunTask in the Amazon ECS API Reference.").with_provider_name("Tags").build(),
                    StructField::optional("task_count", AttributeType::refined_int(None, Some((Some(1), Some(10))))).with_description("The number of tasks to create based on TaskDefinition. The default is 1.").with_provider_name("TaskCount").build(),
                    StructField::required("task_definition_arn", carina_aws_types::arn()).with_description("The ARN of the task definition to use if the event target is an Amazon ECS task.").with_provider_name("TaskDefinitionArn").build()])).with_provider_name("EcsParameters").with_block_name("ecs_parameter").build(),
                    StructField::optional("event_bridge_parameters", AttributeType::struct_("EventBridgeParameters".to_string(), vec![StructField::required("detail_type", AttributeType::refined_string(None, None, Some((Some(1), Some(128))), None)).with_description("Free-form string, with a maximum of 128 characters, used to decide what fields to expect in the event detail.").with_provider_name("DetailType").build(),
                    StructField::required("source", AttributeType::refined_string(None, Some("^(?=[/\\.\\-_A-Za-z0-9]+)((?!aws\\.).*)|(\\$(\\.[\\w_-]+(\\[(\\d+|\\*)\\])*)*)$".to_string()), Some((Some(1), Some(256))), None)).with_description("The source of the event.").with_provider_name("Source").build()])).with_provider_name("EventBridgeParameters").build(),
                    StructField::optional("input", AttributeType::refined_string(None, None, Some((Some(1), None)), None)).with_description("The text, or well-formed JSON, passed to the target. If you are configuring a templated Lambda, AWS Step Functions, or Amazon EventBridge target, the input must be a well-formed JSON. For all other target types, a JSON is not required. If you do not specify anything for this field, EventBridge Scheduler delivers a default notification to the target.").with_provider_name("Input").build(),
                    StructField::optional("kinesis_parameters", AttributeType::struct_("KinesisParameters".to_string(), vec![StructField::required("partition_key", AttributeType::refined_string(None, None, Some((Some(1), Some(256))), None)).with_description("The custom parameter used as the Kinesis partition key. For more information, see Amazon Kinesis Streams Key Concepts in the Amazon Kinesis Streams Developer Guide.").with_provider_name("PartitionKey").build()])).with_provider_name("KinesisParameters").build(),
                    StructField::optional("retry_policy", AttributeType::struct_("RetryPolicy".to_string(), vec![StructField::optional("maximum_event_age_in_seconds", AttributeType::refined_int(None, Some((Some(60), Some(86400))))).with_description("The maximum amount of time, in seconds, to continue to make retry attempts.").with_provider_name("MaximumEventAgeInSeconds").build(),
                    StructField::optional("maximum_retry_attempts", AttributeType::refined_int(None, Some((Some(0), Some(185))))).with_description("The maximum number of retry attempts to make before the request fails. Retry attempts with exponential backoff continue until either the maximum number of attempts is made or until the duration of the MaximumEventAgeInSeconds is reached.").with_provider_name("MaximumRetryAttempts").build()])).with_provider_name("RetryPolicy").build(),
                    StructField::required("role_arn", carina_aws_types::arn()).with_description("The Amazon Resource Name (ARN) of the IAM role to be used for this target when the schedule is triggered.").with_provider_name("RoleArn").build(),
                    StructField::optional("sage_maker_pipeline_parameters", AttributeType::struct_("SageMakerPipelineParameters".to_string(), vec![StructField::optional("pipeline_parameter_list", AttributeType::refined_list(AttributeType::struct_("SageMakerPipelineParameter".to_string(), vec![StructField::required("name", AttributeType::refined_string(None, Some("^[A-Za-z0-9\\-_]*$".to_string()), Some((Some(1), Some(256))), None)).with_description("Name of parameter to start execution of a SageMaker Model Building Pipeline.").with_provider_name("Name").build(),
                    StructField::required("value", AttributeType::refined_string(None, None, Some((Some(1), Some(1024))), None)).with_description("Value of parameter to start execution of a SageMaker Model Building Pipeline.").with_provider_name("Value").build()]), false, Some((None, Some(200))))).with_description("List of Parameter names and values for SageMaker Model Building Pipeline execution.").with_provider_name("PipelineParameterList").with_block_name("pipeline_parameter_list").build()])).with_provider_name("SageMakerPipelineParameters").with_block_name("sage_maker_pipeline_parameter").build(),
                    StructField::optional("sqs_parameters", AttributeType::struct_("SqsParameters".to_string(), vec![StructField::optional("message_group_id", AttributeType::refined_string(None, None, Some((Some(1), Some(128))), None)).with_description("The FIFO message group ID to use as the target.").with_provider_name("MessageGroupId").build()])).with_provider_name("SqsParameters").build()]) }
    schema = schema.with_def("Target", build_schema_def_10());
    AwsccSchemaConfig {
        aws_type_name: "AWS::Scheduler::Schedule",
        resource_type_name: "scheduler.Schedule",
        primary_identifier: &[crate::schemas::config::PrimaryIdentifierAttribute { provider_name: "Name", dsl_name: "name" }],
        exclusive_groups: &[],
        tags: None,
        schema,
    }
}

/// Returns the resource type name and all enum valid values for this module
pub fn enum_valid_values() -> (
    &'static str,
    &'static [(&'static str, &'static [&'static str])],
) {
    (
        "scheduler.Schedule",
        &[
            (
                "assign_public_ip",
                VALID_AWS_VPC_CONFIGURATION_ASSIGN_PUBLIC_IP,
            ),
            ("launch_type", VALID_ECS_PARAMETERS_LAUNCH_TYPE),
            ("propagate_tags", VALID_ECS_PARAMETERS_PROPAGATE_TAGS),
            ("mode", VALID_FLEXIBLE_TIME_WINDOW_MODE),
            ("type", VALID_PLACEMENT_CONSTRAINT_TYPE),
            ("field", VALID_PLACEMENT_STRATEGY_FIELD),
            ("type", VALID_PLACEMENT_STRATEGY_TYPE),
            ("state", VALID_STATE),
        ],
    )
}

/// Returns the IAM permissions declared by the CloudFormation handler for this operation.
pub fn required_permissions(op: carina_core::effect::PlanOp) -> &'static [&'static str] {
    match op {
        carina_core::effect::PlanOp::Create => &[
            "scheduler:CreateSchedule",
            "scheduler:GetSchedule",
            "iam:PassRole",
        ],
        carina_core::effect::PlanOp::Read => &["scheduler:GetSchedule"],
        carina_core::effect::PlanOp::Update => &[
            "scheduler:UpdateSchedule",
            "scheduler:GetSchedule",
            "iam:PassRole",
        ],
        carina_core::effect::PlanOp::Delete => {
            &["scheduler:DeleteSchedule", "scheduler:GetSchedule"]
        }
    }
}
