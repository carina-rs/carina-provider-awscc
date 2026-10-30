---
title: "awscc.scheduler.Schedule"
---


CloudFormation Type: `AWS::Scheduler::Schedule`

Definition of AWS::Scheduler::Schedule Resource Type

## Argument Reference

### `description`

- **Type:** String(len: ..=512)
- **Required:** No

The description of the schedule.

### `end_date`

- **Type:** String(date-time)
- **Required:** No

The date, in UTC, before which the schedule can invoke its target. Depending on the schedule's recurrence expression, invocations might stop on, or before, the EndDate you specify.

### `flexible_time_window`

- **Type:** [Struct(FlexibleTimeWindow)](#flexible_time_window-1)
- **Required:** Yes

### `group_name`

- **Type:** SchedulerScheduleGroupName
- **Required:** No

The name of the schedule group to associate with this schedule. If you omit this, the default schedule group is used. Carina requires group_name to be omitted or set to the literal 'default'. Cloud Control identifies schedules by Name only and reads them in the default group, so a schedule in another group can be created but never read or tracked.

### `kms_key_arn`

- **Type:** KmsKeyArn
- **Required:** No

The ARN for a KMS Key that will be used to encrypt customer data.

### `name`

- **Type:** String(pattern, len: 1..=64)
- **Required:** No
- **Create-only:** Yes

### `schedule_expression`

- **Type:** String(len: 1..=256)
- **Required:** Yes

The scheduling expression.

### `schedule_expression_timezone`

- **Type:** String(len: 1..=50)
- **Required:** No

The timezone in which the scheduling expression is evaluated.

### `start_date`

- **Type:** String(date-time)
- **Required:** No

The date, in UTC, after which the schedule can begin invoking its target. Depending on the schedule's recurrence expression, invocations might occur on, or after, the StartDate you specify.

### `state`

- **Type:** [Enum (ScheduleState)](#state-schedulestate)
- **Required:** No

### `target`

- **Type:** [Struct(Target)](#target-1)
- **Required:** Yes

## Enum Values

### flexible_time_window.mode (FlexibleTimeWindowMode)

| Value | DSL Identifier |
|-------|----------------|
| `OFF` | `aws.scheduler.Schedule.FlexibleTimeWindow.FlexibleTimeWindowMode.off` |
| `FLEXIBLE` | `aws.scheduler.Schedule.FlexibleTimeWindow.FlexibleTimeWindowMode.flexible` |

Shorthand formats: `off` or `FlexibleTimeWindowMode.off`

### state (ScheduleState)

| Value | DSL Identifier |
|-------|----------------|
| `ENABLED` | `aws.scheduler.Schedule.ScheduleState.enabled` |
| `DISABLED` | `aws.scheduler.Schedule.ScheduleState.disabled` |

Shorthand formats: `enabled` or `ScheduleState.enabled`

### target.ecs_parameters.launch_type (LaunchType)

| Value | DSL Identifier |
|-------|----------------|
| `EC2` | `aws.scheduler.Schedule.Target.EcsParameters.LaunchType.ec2` |
| `FARGATE` | `aws.scheduler.Schedule.Target.EcsParameters.LaunchType.fargate` |
| `EXTERNAL` | `aws.scheduler.Schedule.Target.EcsParameters.LaunchType.external` |

Shorthand formats: `ec2` or `LaunchType.ec2`

### target.ecs_parameters.network_configuration.awsvpc_configuration.assign_public_ip (AssignPublicIp)

| Value | DSL Identifier |
|-------|----------------|
| `ENABLED` | `aws.scheduler.Schedule.Target.EcsParameters.NetworkConfiguration.AwsVpcConfiguration.AssignPublicIp.enabled` |
| `DISABLED` | `aws.scheduler.Schedule.Target.EcsParameters.NetworkConfiguration.AwsVpcConfiguration.AssignPublicIp.disabled` |

Shorthand formats: `enabled` or `AssignPublicIp.enabled`

### target.ecs_parameters.placement_constraints.type (PlacementConstraintType)

| Value | DSL Identifier |
|-------|----------------|
| `distinctInstance` | `aws.scheduler.Schedule.Target.EcsParameters.PlacementConstraint.PlacementConstraintType.distinct_instance` |
| `memberOf` | `aws.scheduler.Schedule.Target.EcsParameters.PlacementConstraint.PlacementConstraintType.member_of` |

Shorthand formats: `distinct_instance` or `PlacementConstraintType.distinct_instance`

### target.ecs_parameters.placement_strategy.field (Field)

| Value | DSL Identifier |
|-------|----------------|
| `instanceId` | `aws.scheduler.Schedule.Target.EcsParameters.PlacementStrategy.Field.instance_id` |
| `cpu` | `aws.scheduler.Schedule.Target.EcsParameters.PlacementStrategy.Field.cpu` |
| `memory` | `aws.scheduler.Schedule.Target.EcsParameters.PlacementStrategy.Field.memory` |

Shorthand formats: `instance_id` or `Field.instance_id`

### target.ecs_parameters.placement_strategy.type (PlacementStrategyType)

| Value | DSL Identifier |
|-------|----------------|
| `random` | `aws.scheduler.Schedule.Target.EcsParameters.PlacementStrategy.PlacementStrategyType.random` |
| `spread` | `aws.scheduler.Schedule.Target.EcsParameters.PlacementStrategy.PlacementStrategyType.spread` |
| `binpack` | `aws.scheduler.Schedule.Target.EcsParameters.PlacementStrategy.PlacementStrategyType.binpack` |

Shorthand formats: `random` or `PlacementStrategyType.random`

### target.ecs_parameters.propagate_tags (PropagateTags)

| Value | DSL Identifier |
|-------|----------------|
| `TASK_DEFINITION` | `aws.scheduler.Schedule.Target.EcsParameters.PropagateTags.task_definition` |

Shorthand formats: `task_definition` or `PropagateTags.task_definition`

## Struct Definitions

### flexible_time_window

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `maximum_window_in_minutes` | Int(1..=1440) | No | The maximum time window during which a schedule can be invoked. |
| `mode` | [Enum (FlexibleTimeWindowMode)](#flexible_time_windowmode-flexibletimewindowmode) | Yes |  |

### target

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `arn` | Arn | Yes | The Amazon Resource Name (ARN) of the target. |
| `dead_letter_config` | [Struct(DeadLetterConfig)](#targetdead_letter_config) | No |  |
| `ecs_parameters` | [Struct(EcsParameters)](#targetecs_parameters) | No |  |
| `event_bridge_parameters` | [Struct(EventBridgeParameters)](#targetevent_bridge_parameters) | No |  |
| `input` | String(len: 1..) | No | The text, or well-formed JSON, passed to the target. If you are configuring a templated Lambda, AWS Step Functions, or Amazon EventBridge target, the input must be a well-formed JSON. For all other target types, a JSON is not required. If you do not specify anything for this field, EventBridge Scheduler delivers a default notification to the target. |
| `kinesis_parameters` | [Struct(KinesisParameters)](#targetkinesis_parameters) | No |  |
| `retry_policy` | [Struct(RetryPolicy)](#targetretry_policy) | No |  |
| `role_arn` | Arn | Yes | The Amazon Resource Name (ARN) of the IAM role to be used for this target when the schedule is triggered. |
| `sage_maker_pipeline_parameters` | [Struct(SageMakerPipelineParameters)](#targetsage_maker_pipeline_parameters) | No |  |
| `sqs_parameters` | [Struct(SqsParameters)](#targetsqs_parameters) | No |  |

### target.dead_letter_config

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `arn` | Arn | No | The ARN of the SQS queue specified as the target for the dead-letter queue. |

### target.ecs_parameters

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `capacity_provider_strategy` | List\<[Struct(CapacityProviderStrategyItem)](#targetecs_parameterscapacity_provider_strategy)\> (items: ..=6) | No | The capacity provider strategy to use for the task. |
| `enable_ecs_managed_tags` | Bool | No | Specifies whether to enable Amazon ECS managed tags for the task. For more information, see Tagging Your Amazon ECS Resources in the Amazon Elastic Container Service Developer Guide. |
| `enable_execute_command` | Bool | No | Whether or not to enable the execute command functionality for the containers in this task. If true, this enables execute command functionality on all containers in the task. |
| `group` | String(len: 1..=255) | No | Specifies an ECS task group for the task. The maximum length is 255 characters. |
| `launch_type` | [Enum (LaunchType)](#targetecs_parameterslaunch_type-launchtype) | No |  |
| `network_configuration` | [Struct(NetworkConfiguration)](#targetecs_parametersnetwork_configuration) | No |  |
| `placement_constraints` | List\<[Struct(PlacementConstraint)](#targetecs_parametersplacement_constraints)\> (items: ..=10) | No | An array of placement constraint objects to use for the task. You can specify up to 10 constraints per task (including constraints in the task definition and those specified at runtime). |
| `placement_strategy` | List\<[Struct(PlacementStrategy)](#targetecs_parametersplacement_strategy)\> (items: ..=5) | No | The placement strategy objects to use for the task. You can specify a maximum of five strategy rules per task. |
| `platform_version` | String(len: 1..=64) | No | Specifies the platform version for the task. Specify only the numeric portion of the platform version, such as 1.1.0. |
| `propagate_tags` | [Enum (PropagateTags)](#targetecs_parameterspropagate_tags-propagatetags) | No |  |
| `reference_id` | String(len: ..=1024) | No | The reference ID to use for the task. |
| `tags` | `Map<String, String>` | No | The metadata that you apply to the task to help you categorize and organize them. Each tag consists of a key and an optional value, both of which you define. To learn more, see RunTask in the Amazon ECS API Reference. |
| `task_count` | Int(1..=10) | No | The number of tasks to create based on TaskDefinition. The default is 1. |
| `task_definition_arn` | Arn | Yes | The ARN of the task definition to use if the event target is an Amazon ECS task. |

### target.ecs_parameters.capacity_provider_strategy

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `base` | Int(0..=100000) | No | The base value designates how many tasks, at a minimum, to run on the specified capacity provider. Only one capacity provider in a capacity provider strategy can have a base defined. If no value is specified, the default value of 0 is used. |
| `capacity_provider` | String(len: 1..=255) | Yes | The short name of the capacity provider. |
| `weight` | Int(0..=1000) | No | The weight value designates the relative percentage of the total number of tasks launched that should use the specified capacity provider. The weight value is taken into consideration after the base value, if defined, is satisfied. |

### target.ecs_parameters.network_configuration

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `awsvpc_configuration` | [Struct(AwsVpcConfiguration)](#targetecs_parametersnetwork_configurationawsvpc_configuration) | No |  |

### target.ecs_parameters.network_configuration.awsvpc_configuration

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `assign_public_ip` | [Enum (AssignPublicIp)](#targetecs_parametersnetwork_configurationawsvpc_configurationassign_public_ip-assignpublicip) | No |  |
| `security_groups` | `List<String>` (items: 1..=5) | No | Specifies the security groups associated with the task. These security groups must all be in the same VPC. You can specify as many as five security groups. If you do not specify a security group, the default security group for the VPC is used. |
| `subnets` | `List<String>` (items: 1..=16) | Yes | Specifies the subnets associated with the task. These subnets must all be in the same VPC. You can specify as many as 16 subnets. |

### target.ecs_parameters.placement_constraints

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `expression` | String(len: ..=2000) | No | A cluster query language expression to apply to the constraint. You cannot specify an expression if the constraint type is distinctInstance. To learn more, see Cluster Query Language in the Amazon Elastic Container Service Developer Guide. |
| `type` | [Enum (PlacementConstraintType)](#targetecs_parametersplacement_constraintstype-placementconstrainttype) | No |  |

### target.ecs_parameters.placement_strategy

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `field` | [Enum (Field)](#targetecs_parametersplacement_strategyfield-field) | No | The field to apply the placement strategy against. For the spread placement strategy, valid values are instanceId (or host, which has the same effect), or any platform or custom attribute that is applied to a container instance, such as attribute:ecs.availability-zone. For the binpack placement strategy, valid values are cpu and memory. For the random placement strategy, this field is not used. |
| `type` | [Enum (PlacementStrategyType)](#targetecs_parametersplacement_strategytype-placementstrategytype) | No |  |

### target.event_bridge_parameters

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `detail_type` | String(len: 1..=128) | Yes | Free-form string, with a maximum of 128 characters, used to decide what fields to expect in the event detail. |
| `source` | String(pattern, len: 1..=256) | Yes | The source of the event. |

### target.kinesis_parameters

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `partition_key` | String(len: 1..=256) | Yes | The custom parameter used as the Kinesis partition key. For more information, see Amazon Kinesis Streams Key Concepts in the Amazon Kinesis Streams Developer Guide. |

### target.retry_policy

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `maximum_event_age_in_seconds` | Int(60..=86400) | No | The maximum amount of time, in seconds, to continue to make retry attempts. |
| `maximum_retry_attempts` | Int(0..=185) | No | The maximum number of retry attempts to make before the request fails. Retry attempts with exponential backoff continue until either the maximum number of attempts is made or until the duration of the MaximumEventAgeInSeconds is reached. |

### target.sage_maker_pipeline_parameters

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `pipeline_parameter_list` | List\<[Struct(SageMakerPipelineParameter)](#targetsage_maker_pipeline_parameterspipeline_parameter_list)\> (items: 0..=200) | No | List of Parameter names and values for SageMaker Model Building Pipeline execution. |

### target.sage_maker_pipeline_parameters.pipeline_parameter_list

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `name` | String(pattern, len: 1..=256) | Yes | Name of parameter to start execution of a SageMaker Model Building Pipeline. |
| `value` | String(len: 1..=1024) | Yes | Value of parameter to start execution of a SageMaker Model Building Pipeline. |

### target.sqs_parameters

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `message_group_id` | String(len: 1..=128) | No | The FIFO message group ID to use as the target. |

## Attribute Reference

### `arn`

- **Type:** Arn

The Amazon Resource Name (ARN) of the schedule.

