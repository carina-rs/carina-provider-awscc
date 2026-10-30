//! alarm schema definition for AWS Cloud Control
//!
//! Auto-generated from CloudFormation schema: AWS::CloudWatch::Alarm
//!
//! DO NOT EDIT MANUALLY - regenerate with carina-codegen

use crate::schemas::config::{AwsccSchemaConfig, TagsProperty, TagsWire};
use carina_core::resource::{ConcreteValue, Value};
use carina_core::schema::{AttributeSchema, AttributeType, ResourceSchema, StructField};

pub fn arn() -> AttributeType {
    carina_aws_types::arn()
}

const VALID_TREAT_MISSING_DATA: &[&str] = &["breaching", "notBreaching", "ignore", "missing"];

/// Returns the schema config for cloudwatch_alarm (AWS::CloudWatch::Alarm)
#[rustfmt::skip]
pub fn cloudwatch_alarm_config() -> AwsccSchemaConfig {
    let mut schema = ResourceSchema::new("cloudwatch.Alarm")
        .with_description("The ``AWS::CloudWatch::Alarm`` type specifies an alarm and associates it with the specified metric or metric math expression.  When this operation creates an alarm, the alarm state is immediately set to ``INSUFFICIENT_DATA``. The alarm is then evaluated and its state is set appropriately. Any actions associated with the new state are then executed.  When you update an existing alarm, its state is left unchanged, but the update completely overwrites the previous configuration of the alarm.");
    #[inline(never)]
    fn build_action_configuration_attribute() -> AttributeSchema {
        AttributeSchema::optional("action_configuration", AttributeType::struct_("ActionConfiguration".to_string(), vec![StructField::optional("alarm_tags_propagation", AttributeType::struct_("AlarmTagsPropagation".to_string(), vec![StructField::required("enabled", AttributeType::bool()).with_description("Whether the alarm's tags are included in its action notifications and state-change events.").with_provider_name("Enabled").build(),
                    StructField::optional("excluded_tag_keys", AttributeType::refined_list(AttributeType::refined_string(None, None, Some((Some(1), Some(128))), None), false, Some((Some(1), Some(50))))).with_description("The tag keys to leave out. Cannot be specified together with IncludedTagKeys.").with_provider_name("ExcludedTagKeys").build(),
                    StructField::optional("included_tag_keys", AttributeType::refined_list(AttributeType::refined_string(None, None, Some((Some(1), Some(128))), None), false, Some((Some(1), Some(50))))).with_description("The only tag keys to include. Cannot be specified together with ExcludedTagKeys.").with_provider_name("IncludedTagKeys").build()])).with_description("Whether the alarm's tags are included in its action notifications and state-change events.").with_provider_name("AlarmTagsPropagation").build()]))
                .with_description("")
                .with_provider_name("ActionConfiguration")
            .build()
    }
    schema = schema.attribute(build_action_configuration_attribute());
    #[inline(never)]
    fn build_actions_enabled_attribute() -> AttributeSchema {
        AttributeSchema::optional("actions_enabled", AttributeType::bool())
                .with_description("Indicates whether actions should be executed during any changes to the alarm state. The default is TRUE.")
                .with_provider_name("ActionsEnabled")
                .with_default(Value::Concrete(ConcreteValue::Bool(true)))
            .build()
    }
    schema = schema.attribute(build_actions_enabled_attribute());
    #[inline(never)]
    fn build_alarm_actions_attribute() -> AttributeSchema {
        AttributeSchema::optional("alarm_actions", AttributeType::list(AttributeType::string()))
                .with_description("The list of actions to execute when this alarm transitions into an ALARM state from any other state. Specify each action as an Amazon Resource Name (ARN). For more information about creating alarms and the actions that you can specify, see [PutMetricAlarm](https://docs.aws.amazon.com/AmazonCloudWatch/latest/APIReference/API_PutMetricAlarm.html) in the *API Reference*.")
                .with_provider_name("AlarmActions")
            .build()
    }
    schema = schema.attribute(build_alarm_actions_attribute());
    #[inline(never)]
    fn build_alarm_description_attribute() -> AttributeSchema {
        AttributeSchema::optional("alarm_description", AttributeType::string())
                .with_description("The description of the alarm.")
                .with_provider_name("AlarmDescription")
            .build()
    }
    schema = schema.attribute(build_alarm_description_attribute());
    #[inline(never)]
    fn build_alarm_name_attribute() -> AttributeSchema {
        AttributeSchema::optional("alarm_name", AttributeType::string())
                .create_only()
                .with_description("The name of the alarm. If you don't specify a name, CFN generates a unique physical ID and uses that ID for the alarm name. If you specify a name, you cannot perform updates that require replacement of this resource. You can perform updates that require no or some interruption. If you must replace the resource, specify a new name.")
                .with_provider_name("AlarmName")
            .build()
    }
    schema = schema.attribute(build_alarm_name_attribute());
    #[inline(never)]
    fn build_arn_attribute() -> AttributeSchema {
        AttributeSchema::provider_populated("arn", self::arn())
                .with_description(" (read-only)")
                .with_provider_name("Arn")
            .build()
    }
    schema = schema.attribute(build_arn_attribute());
    #[inline(never)]
    fn build_comparison_operator_attribute() -> AttributeSchema {
        AttributeSchema::optional("comparison_operator", AttributeType::string())
                .with_description("The arithmetic operation to use when comparing the specified statistic and threshold. The specified statistic value is used as the first operand.")
                .with_provider_name("ComparisonOperator")
            .build()
    }
    schema = schema.attribute(build_comparison_operator_attribute());
    #[inline(never)]
    fn build_datapoints_to_alarm_attribute() -> AttributeSchema {
        AttributeSchema::optional("datapoints_to_alarm", AttributeType::int())
                .with_description("The number of datapoints that must be breaching to trigger the alarm. This is used only if you are setting an \"M out of N\" alarm. In that case, this value is the M, and the value that you set for ``EvaluationPeriods`` is the N value. For more information, see [Evaluating an Alarm](https://docs.aws.amazon.com/AmazonCloudWatch/latest/monitoring/AlarmThatSendsEmail.html#alarm-evaluation) in the *User Guide*. If you omit this parameter, CW uses the same value here that you set for ``EvaluationPeriods``, and the alarm goes to alarm state if that many consecutive periods are breaching.")
                .with_provider_name("DatapointsToAlarm")
            .build()
    }
    schema = schema.attribute(build_datapoints_to_alarm_attribute());
    #[inline(never)]
    fn build_dimensions_attribute() -> AttributeSchema {
        AttributeSchema::optional("dimensions", AttributeType::unordered_list(AttributeType::struct_("Dimension".to_string(), vec![StructField::required("name", AttributeType::string()).with_description("The name of the dimension, from 1–255 characters in length. This dimension name must have been included when the metric was published.").with_provider_name("Name").build(),
                    StructField::required("value", AttributeType::string()).with_description("The value for the dimension, from 1–255 characters in length.").with_provider_name("Value").build()])))
                .with_description("The dimensions for the metric associated with the alarm. For an alarm based on a math expression, you can't specify ``Dimensions``. Instead, you use ``Metrics``.")
                .with_provider_name("Dimensions")
                .with_block_name("dimension")
            .build()
    }
    schema = schema.attribute(build_dimensions_attribute());
    #[inline(never)]
    fn build_evaluate_low_sample_count_percentile_attribute() -> AttributeSchema {
        AttributeSchema::optional("evaluate_low_sample_count_percentile", AttributeType::string())
                .with_description("Used only for alarms based on percentiles. If ``ignore``, the alarm state does not change during periods with too few data points to be statistically significant. If ``evaluate`` or this parameter is not used, the alarm is always evaluated and possibly changes state no matter how many data points are available.")
                .with_provider_name("EvaluateLowSampleCountPercentile")
            .build()
    }
    schema = schema.attribute(build_evaluate_low_sample_count_percentile_attribute());
    #[inline(never)]
    fn build_evaluation_criteria_attribute() -> AttributeSchema {
        AttributeSchema::optional("evaluation_criteria", AttributeType::struct_("EvaluationCriteria".to_string(), vec![StructField::optional("prom_ql_criteria", AttributeType::struct_("AlarmPromQLCriteria".to_string(), vec![StructField::optional("pending_period", AttributeType::int()).with_description("The duration, in seconds, that a contributor must be continuously breaching before it transitions to the ``ALARM`` state.").with_provider_name("PendingPeriod").build(),
                    StructField::optional("query", AttributeType::string()).with_description("The PromQL query that the alarm evaluates. The query must return a result of vector type. Each entry in the vector result represents an alarm contributor.").with_provider_name("Query").build(),
                    StructField::optional("recovery_period", AttributeType::int()).with_description("The duration, in seconds, that a contributor must continuously not be breaching before it transitions back to the ``OK`` state.").with_provider_name("RecoveryPeriod").build()])).with_description("The PromQL criteria for the alarm evaluation.").with_provider_name("PromQLCriteria").build()]))
                .with_description("The evaluation criteria for the alarm.")
                .with_provider_name("EvaluationCriteria")
            .build()
    }
    schema = schema.attribute(build_evaluation_criteria_attribute());
    #[inline(never)]
    fn build_evaluation_interval_attribute() -> AttributeSchema {
        AttributeSchema::optional("evaluation_interval", AttributeType::int())
                .with_description("The frequency, in seconds, at which the alarm is evaluated.")
                .with_provider_name("EvaluationInterval")
            .build()
    }
    schema = schema.attribute(build_evaluation_interval_attribute());
    #[inline(never)]
    fn build_evaluation_periods_attribute() -> AttributeSchema {
        AttributeSchema::optional("evaluation_periods", AttributeType::int())
                .with_description("The number of periods over which data is compared to the specified threshold. If you are setting an alarm that requires that a number of consecutive data points be breaching to trigger the alarm, this value specifies that number. If you are setting an \"M out of N\" alarm, this value is the N, and ``DatapointsToAlarm`` is the M. For more information, see [Evaluating an Alarm](https://docs.aws.amazon.com/AmazonCloudWatch/latest/monitoring/AlarmThatSendsEmail.html#alarm-evaluation) in the *User Guide*.")
                .with_provider_name("EvaluationPeriods")
            .build()
    }
    schema = schema.attribute(build_evaluation_periods_attribute());
    #[inline(never)]
    fn build_evaluation_window_attribute() -> AttributeSchema {
        AttributeSchema::optional("evaluation_window", AttributeType::struct_("EvaluationWindow".to_string(), vec![StructField::optional("sliding_window", AttributeType::struct_("SlidingWindow".to_string(), vec![])).with_description("A sliding window, which advances each time the alarm is evaluated, forming a rolling time window. This is the default evaluation window.").with_provider_name("SlidingWindow").build(),
                    StructField::optional("wall_clock_window", AttributeType::struct_("WallClockWindow".to_string(), vec![StructField::optional("timezone", AttributeType::string()).with_description("The time zone to use when the alarm aligns the evaluation window to clock boundaries. You can specify an IANA time zone name (for example, ``America/New_York``), a fixed UTC offset (for example, ``+05:30``), or an offset-prefixed identifier (for example, ``UTC+05:30``). The offset must be aligned to a multiple of 5 minutes. If you don't specify a time zone, CloudWatch uses ``UTC``. The time zone affects window alignment for all periods, including periods of one hour or shorter.").with_provider_name("Timezone").build()])).with_description("A wall clock window, which aligns the evaluated range to fixed clock boundaries that match the alarm's period, such as the top of the hour, midnight, or the start of the calendar week.").with_provider_name("WallClockWindow").build()]))
                .with_description("The evaluation window that the alarm uses to select the range of metric data that it evaluates. This is either a sliding window or a wall clock window. For more information, see [Alarm evaluation windows](https://docs.aws.amazon.com/AmazonCloudWatch/latest/monitoring/alarm-evaluation-window.html) in the *CloudWatch User Guide*.")
                .with_provider_name("EvaluationWindow")
            .build()
    }
    schema = schema.attribute(build_evaluation_window_attribute());
    #[inline(never)]
    fn build_extended_statistic_attribute() -> AttributeSchema {
        AttributeSchema::optional("extended_statistic", AttributeType::string())
                .with_description("The percentile statistic for the metric associated with the alarm. Specify a value between p0.0 and p100. For an alarm based on a metric, you must specify either ``Statistic`` or ``ExtendedStatistic`` but not both. For an alarm based on a math expression, you can't specify ``ExtendedStatistic``. Instead, you use ``Metrics``.")
                .with_provider_name("ExtendedStatistic")
            .build()
    }
    schema = schema.attribute(build_extended_statistic_attribute());
    #[inline(never)]
    fn build_insufficient_data_actions_attribute() -> AttributeSchema {
        AttributeSchema::optional("insufficient_data_actions", AttributeType::list(AttributeType::string()))
                .with_description("The actions to execute when this alarm transitions to the ``INSUFFICIENT_DATA`` state from any other state. Each action is specified as an Amazon Resource Name (ARN).")
                .with_provider_name("InsufficientDataActions")
            .build()
    }
    schema = schema.attribute(build_insufficient_data_actions_attribute());
    #[inline(never)]
    fn build_metric_name_attribute() -> AttributeSchema {
        AttributeSchema::optional("metric_name", AttributeType::string())
                .with_description("The name of the metric associated with the alarm. This is required for an alarm based on a metric. For an alarm based on a math expression, you use ``Metrics`` instead and you can't specify ``MetricName``.")
                .with_provider_name("MetricName")
            .build()
    }
    schema = schema.attribute(build_metric_name_attribute());
    #[inline(never)]
    fn build_metrics_attribute() -> AttributeSchema {
        AttributeSchema::optional("metrics", AttributeType::list(AttributeType::struct_("MetricDataQuery".to_string(), vec![StructField::optional("account_id", carina_aws_types::aws_account_id()).with_description("The ID of the account where the metrics are located, if this is a cross-account alarm.").with_provider_name("AccountId").build(),
                    StructField::optional("expression", AttributeType::string()).with_description("The math expression to be performed on the returned data, if this object is performing a math expression. This expression can use the ``Id`` of the other metrics to refer to those metrics, and can also use the ``Id`` of other expressions to use the result of those expressions. For more information about metric math expressions, see [Metric Math Syntax and Functions](https://docs.aws.amazon.com/AmazonCloudWatch/latest/monitoring/using-metric-math.html#metric-math-syntax) in the *User Guide*. Within each MetricDataQuery object, you must specify either ``Expression`` or ``MetricStat`` but not both.").with_provider_name("Expression").build(),
                    StructField::required("id", AttributeType::string()).with_description("A short name used to tie this object to the results in the response. This name must be unique within a single call to ``GetMetricData``. If you are performing math expressions on this set of data, this name represents that data and can serve as a variable in the mathematical expression. The valid characters are letters, numbers, and underscore. The first character must be a lowercase letter.").with_provider_name("Id").build(),
                    StructField::optional("label", AttributeType::string()).with_description("A human-readable label for this metric or expression. This is especially useful if this is an expression, so that you know what the value represents. If the metric or expression is shown in a CW dashboard widget, the label is shown. If ``Label`` is omitted, CW generates a default.").with_provider_name("Label").build(),
                    StructField::optional("metric_stat", AttributeType::struct_("MetricStat".to_string(), vec![StructField::required("metric", AttributeType::struct_("Metric".to_string(), vec![StructField::optional("dimensions", AttributeType::unordered_list(AttributeType::struct_("Dimension".to_string(), vec![StructField::required("name", AttributeType::string()).with_description("The name of the dimension, from 1–255 characters in length. This dimension name must have been included when the metric was published.").with_provider_name("Name").build(),
                    StructField::required("value", AttributeType::string()).with_description("The value for the dimension, from 1–255 characters in length.").with_provider_name("Value").build()]))).with_description("The metric dimensions that you want to be used for the metric that the alarm will watch.").with_provider_name("Dimensions").with_block_name("dimension").build(),
                    StructField::optional("metric_name", AttributeType::string()).with_description("The name of the metric that you want the alarm to watch. This is a required field.").with_provider_name("MetricName").build(),
                    StructField::optional("namespace", AttributeType::string()).with_description("The namespace of the metric that the alarm will watch.").with_provider_name("Namespace").build()])).with_description("The metric to return, including the metric name, namespace, and dimensions.").with_provider_name("Metric").with_block_name("metric").build(),
                    StructField::required("period", AttributeType::int()).with_description("The granularity, in seconds, of the returned data points. For metrics with regular resolution, a period can be as short as one minute (60 seconds) and must be a multiple of 60. For high-resolution metrics that are collected at intervals of less than one minute, the period can be 1, 5, 10, 20, 30, 60, or any multiple of 60. High-resolution metrics are those metrics stored by a ``PutMetricData`` call that includes a ``StorageResolution`` of 1 second. If the ``StartTime`` parameter specifies a time stamp that is greater than 3 hours ago, you must specify the period as follows or no data points in that time range is returned: + Start time between 3 hours and 15 days ago - Use a multiple of 60 seconds (1 minute). + Start time between 15 and 63 days ago - Use a multiple of 300 seconds (5 minutes). + Start time greater than 63 days ago - Use a multiple of 3600 seconds (1 hour).").with_provider_name("Period").build(),
                    StructField::required("stat", AttributeType::string()).with_description("The statistic to return. It can include any CW statistic or extended statistic. For a list of valid values, see the table in [Statistics](https://docs.aws.amazon.com/AmazonCloudWatch/latest/monitoring/cloudwatch_concepts.html#Statistic) in the *User Guide*.").with_provider_name("Stat").build(),
                    StructField::optional("unit", AttributeType::string()).with_description("The unit to use for the returned data points. Valid values are: Seconds, Microseconds, Milliseconds, Bytes, Kilobytes, Megabytes, Gigabytes, Terabytes, Bits, Kilobits, Megabits, Gigabits, Terabits, Percent, Count, Bytes/Second, Kilobytes/Second, Megabytes/Second, Gigabytes/Second, Terabytes/Second, Bits/Second, Kilobits/Second, Megabits/Second, Gigabits/Second, Terabits/Second, Count/Second, or None.").with_provider_name("Unit").build()])).with_description("The metric to be returned, along with statistics, period, and units. Use this parameter only if this object is retrieving a metric and not performing a math expression on returned data. Within one MetricDataQuery object, you must specify either ``Expression`` or ``MetricStat`` but not both.").with_provider_name("MetricStat").with_block_name("metric_stat").build(),
                    StructField::optional("period", AttributeType::int()).with_description("The granularity, in seconds, of the returned data points. For metrics with regular resolution, a period can be as short as one minute (60 seconds) and must be a multiple of 60. For high-resolution metrics that are collected at intervals of less than one minute, the period can be 1, 5, 10, 20, 30, 60, or any multiple of 60. High-resolution metrics are those metrics stored by a ``PutMetricData`` operation that includes a ``StorageResolution of 1 second``.").with_provider_name("Period").build(),
                    StructField::optional("return_data", AttributeType::bool()).with_description("This option indicates whether to return the timestamps and raw data values of this metric. When you create an alarm based on a metric math expression, specify ``True`` for this value for only the one math expression that the alarm is based on. You must specify ``False`` for ``ReturnData`` for all the other metrics and expressions used in the alarm. This field is required.").with_provider_name("ReturnData").build()])))
                .with_description("An array that enables you to create an alarm based on the result of a metric math expression. Each item in the array either retrieves a metric or performs a math expression. If you specify the ``Metrics`` parameter, you cannot specify ``MetricName``, ``Dimensions``, ``Period``, ``Namespace``, ``Statistic``, ``ExtendedStatistic``, or ``Unit``.")
                .with_provider_name("Metrics")
                .with_block_name("metric")
            .build()
    }
    schema = schema.attribute(build_metrics_attribute());
    #[inline(never)]
    fn build_namespace_attribute() -> AttributeSchema {
        AttributeSchema::optional("namespace", AttributeType::string())
                .with_description("The namespace of the metric associated with the alarm. This is required for an alarm based on a metric. For an alarm based on a math expression, you can't specify ``Namespace`` and you use ``Metrics`` instead. For a list of namespaces for metrics from AWS services, see [Services That Publish Metrics.](https://docs.aws.amazon.com/AmazonCloudWatch/latest/monitoring/aws-services-cloudwatch-metrics.html)")
                .with_provider_name("Namespace")
            .build()
    }
    schema = schema.attribute(build_namespace_attribute());
    #[inline(never)]
    fn build_ok_actions_attribute() -> AttributeSchema {
        AttributeSchema::optional("ok_actions", AttributeType::list(AttributeType::string()))
                .with_description("The actions to execute when this alarm transitions to the ``OK`` state from any other state. Each action is specified as an Amazon Resource Name (ARN).")
                .with_provider_name("OKActions")
            .build()
    }
    schema = schema.attribute(build_ok_actions_attribute());
    #[inline(never)]
    fn build_period_attribute() -> AttributeSchema {
        AttributeSchema::optional("period", AttributeType::int())
                .with_description("The period, in seconds, over which the statistic is applied. This is required for an alarm based on a metric. Valid values are 10, 20, 30, 60, and any multiple of 60. For an alarm based on a math expression, you can't specify ``Period``, and instead you use the ``Metrics`` parameter. *Minimum:* 10")
                .with_provider_name("Period")
            .build()
    }
    schema = schema.attribute(build_period_attribute());
    #[inline(never)]
    fn build_statistic_attribute() -> AttributeSchema {
        AttributeSchema::optional("statistic", AttributeType::string())
                .with_description("The statistic for the metric associated with the alarm, other than percentile. For percentile statistics, use ``ExtendedStatistic``. For an alarm based on a metric, you must specify either ``Statistic`` or ``ExtendedStatistic`` but not both. For an alarm based on a math expression, you can't specify ``Statistic``. Instead, you use ``Metrics``.")
                .with_provider_name("Statistic")
            .build()
    }
    schema = schema.attribute(build_statistic_attribute());
    #[inline(never)]
    fn build_tags_attribute() -> AttributeSchema {
        AttributeSchema::optional("tags", carina_aws_types::tags_type())
                .with_description("A list of key-value pairs to associate with the alarm. You can associate as many as 50 tags with an alarm. To be able to associate tags with the alarm when you create the alarm, you must have the ``cloudwatch:TagResource`` permission. Tags can help you organize and categorize your resources. You can also use them to scope user permissions by granting a user permission to access or change only resources with certain tag values.")
                .with_provider_name("Tags")
                .with_block_name("tag")
            .build()
    }
    schema = schema.attribute(build_tags_attribute());
    #[inline(never)]
    fn build_threshold_attribute() -> AttributeSchema {
        AttributeSchema::optional("threshold", AttributeType::float())
                .with_description("The value to compare with the specified statistic.")
                .with_provider_name("Threshold")
            .build()
    }
    schema = schema.attribute(build_threshold_attribute());
    #[inline(never)]
    fn build_threshold_metric_id_attribute() -> AttributeSchema {
        AttributeSchema::optional("threshold_metric_id", AttributeType::string())
                .with_description("In an alarm based on an anomaly detection model, this is the ID of the ``ANOMALY_DETECTION_BAND`` function used as the threshold for the alarm.")
                .with_provider_name("ThresholdMetricId")
            .build()
    }
    schema = schema.attribute(build_threshold_metric_id_attribute());
    #[inline(never)]
    fn build_treat_missing_data_attribute() -> AttributeSchema {
        AttributeSchema::optional("treat_missing_data", AttributeType::enum_(carina_core::schema::enum_identity("TreatMissingData", Some("aws.cloudwatch.Alarm")), Some(vec!["breaching".to_string(), "notBreaching".to_string(), "ignore".to_string(), "missing".to_string()]), vec![("breaching".to_string(), "breaching".to_string()), ("notBreaching".to_string(), "not_breaching".to_string()), ("ignore".to_string(), "ignore".to_string()), ("missing".to_string(), "missing".to_string())], None, None))
                .with_description("Sets how this alarm is to handle missing data points. Valid values are ``breaching``, ``notBreaching``, ``ignore``, and ``missing``. For more information, see [Configuring How Alarms Treat Missing Data](https://docs.aws.amazon.com/AmazonCloudWatch/latest/monitoring/AlarmThatSendsEmail.html#alarms-and-missing-data) in the *Amazon User Guide*. If you omit this parameter, the default behavior of ``missing`` is used.")
                .with_provider_name("TreatMissingData")
            .build()
    }
    schema = schema.attribute(build_treat_missing_data_attribute());
    #[inline(never)]
    fn build_unit_attribute() -> AttributeSchema {
        AttributeSchema::optional("unit", AttributeType::string())
                .with_description("The unit of the metric associated with the alarm. Specify this only if you are creating an alarm based on a single metric. Do not specify this if you are specifying a ``Metrics`` array. You can specify the following values: Seconds, Microseconds, Milliseconds, Bytes, Kilobytes, Megabytes, Gigabytes, Terabytes, Bits, Kilobits, Megabits, Gigabits, Terabits, Percent, Count, Bytes/Second, Kilobytes/Second, Megabytes/Second, Gigabytes/Second, Terabytes/Second, Bits/Second, Kilobits/Second, Megabits/Second, Gigabits/Second, Terabits/Second, Count/Second, or None.")
                .with_provider_name("Unit")
            .build()
    }
    schema = schema.attribute(build_unit_attribute());
    #[inline(never)]
    fn build_warm_up_configuration_attribute() -> AttributeSchema {
        AttributeSchema::optional("warm_up_configuration", AttributeType::struct_("WarmUpConfiguration".to_string(), vec![StructField::optional("only_start_evaluating_after_warm_up_period_ends", AttributeType::bool()).with_description("Specifies whether the alarm waits for the full warm-up period before it starts to evaluate. The default is ``false``. If ``true``, the alarm waits the entire ``WarmUpPeriodDurationInMinutes`` before it starts to evaluate, even if metric data arrives earlier. If ``false``, the alarm ends the warm-up period early. Evaluation begins as soon as the alarm has enough metric data to fill its evaluation window.").with_provider_name("OnlyStartEvaluatingAfterWarmUpPeriodEnds").build(),
                    StructField::optional("warm_up_period_duration_in_minutes", AttributeType::refined_int(None, Some((Some(1), Some(2880))))).with_description("The length of the warm-up period, in minutes. After you create or update the alarm, the alarm stays in ``INSUFFICIENT_DATA`` for this duration. During this time, the alarm does not perform alarm actions. You can change this value at any time, including after the warm-up period ends. If you change it after the warm-up period ends, the new value does not restart the warm-up period.").with_provider_name("WarmUpPeriodDurationInMinutes").build()]))
                .with_description("The warm-up configuration for the alarm. A warm-up period delays alarm evaluation after you create or update the alarm. During the warm-up period, the alarm stays in ``INSUFFICIENT_DATA`` and does not perform alarm actions. For more information, see [Alarm warm-up periods](https://docs.aws.amazon.com/AmazonCloudWatch/latest/monitoring/alarm-warm-up.html) in the *Amazon CloudWatch User Guide*.")
                .with_provider_name("WarmUpConfiguration")
            .build()
    }
    schema = schema.attribute(build_warm_up_configuration_attribute());
    schema = schema
        .with_unique_name_attribute("alarm_name");
    schema = schema.with_declared_validators(vec![carina_provider_protocol::types::ValidatorType::TagsKeyValueCheck]);
    #[inline(never)]
    fn build_schema_def_0() -> AttributeType { AttributeType::struct_("ActionConfiguration".to_string(), vec![StructField::optional("alarm_tags_propagation", AttributeType::struct_("AlarmTagsPropagation".to_string(), vec![StructField::required("enabled", AttributeType::bool()).with_description("Whether the alarm's tags are included in its action notifications and state-change events.").with_provider_name("Enabled").build(),
                    StructField::optional("excluded_tag_keys", AttributeType::refined_list(AttributeType::refined_string(None, None, Some((Some(1), Some(128))), None), false, Some((Some(1), Some(50))))).with_description("The tag keys to leave out. Cannot be specified together with IncludedTagKeys.").with_provider_name("ExcludedTagKeys").build(),
                    StructField::optional("included_tag_keys", AttributeType::refined_list(AttributeType::refined_string(None, None, Some((Some(1), Some(128))), None), false, Some((Some(1), Some(50))))).with_description("The only tag keys to include. Cannot be specified together with ExcludedTagKeys.").with_provider_name("IncludedTagKeys").build()])).with_description("Whether the alarm's tags are included in its action notifications and state-change events.").with_provider_name("AlarmTagsPropagation").build()]) }
    schema = schema.with_def("ActionConfiguration", build_schema_def_0());
    #[inline(never)]
    fn build_schema_def_1() -> AttributeType { AttributeType::struct_("AlarmPromQLCriteria".to_string(), vec![StructField::optional("pending_period", AttributeType::int()).with_description("The duration, in seconds, that a contributor must be continuously breaching before it transitions to the ``ALARM`` state.").with_provider_name("PendingPeriod").build(),
                    StructField::optional("query", AttributeType::string()).with_description("The PromQL query that the alarm evaluates. The query must return a result of vector type. Each entry in the vector result represents an alarm contributor.").with_provider_name("Query").build(),
                    StructField::optional("recovery_period", AttributeType::int()).with_description("The duration, in seconds, that a contributor must continuously not be breaching before it transitions back to the ``OK`` state.").with_provider_name("RecoveryPeriod").build()]) }
    schema = schema.with_def("AlarmPromQLCriteria", build_schema_def_1());
    #[inline(never)]
    fn build_schema_def_2() -> AttributeType { AttributeType::struct_("AlarmTagsPropagation".to_string(), vec![StructField::required("enabled", AttributeType::bool()).with_description("Whether the alarm's tags are included in its action notifications and state-change events.").with_provider_name("Enabled").build(),
                    StructField::optional("excluded_tag_keys", AttributeType::refined_list(AttributeType::refined_string(None, None, Some((Some(1), Some(128))), None), false, Some((Some(1), Some(50))))).with_description("The tag keys to leave out. Cannot be specified together with IncludedTagKeys.").with_provider_name("ExcludedTagKeys").build(),
                    StructField::optional("included_tag_keys", AttributeType::refined_list(AttributeType::refined_string(None, None, Some((Some(1), Some(128))), None), false, Some((Some(1), Some(50))))).with_description("The only tag keys to include. Cannot be specified together with ExcludedTagKeys.").with_provider_name("IncludedTagKeys").build()]) }
    schema = schema.with_def("AlarmTagsPropagation", build_schema_def_2());
    #[inline(never)]
    fn build_schema_def_3() -> AttributeType { AttributeType::struct_("EvaluationCriteria".to_string(), vec![StructField::optional("prom_ql_criteria", AttributeType::struct_("AlarmPromQLCriteria".to_string(), vec![StructField::optional("pending_period", AttributeType::int()).with_description("The duration, in seconds, that a contributor must be continuously breaching before it transitions to the ``ALARM`` state.").with_provider_name("PendingPeriod").build(),
                    StructField::optional("query", AttributeType::string()).with_description("The PromQL query that the alarm evaluates. The query must return a result of vector type. Each entry in the vector result represents an alarm contributor.").with_provider_name("Query").build(),
                    StructField::optional("recovery_period", AttributeType::int()).with_description("The duration, in seconds, that a contributor must continuously not be breaching before it transitions back to the ``OK`` state.").with_provider_name("RecoveryPeriod").build()])).with_description("The PromQL criteria for the alarm evaluation.").with_provider_name("PromQLCriteria").build()]) }
    schema = schema.with_def("EvaluationCriteria", build_schema_def_3());
    #[inline(never)]
    fn build_schema_def_4() -> AttributeType { AttributeType::struct_("EvaluationWindow".to_string(), vec![StructField::optional("sliding_window", AttributeType::struct_("SlidingWindow".to_string(), vec![])).with_description("A sliding window, which advances each time the alarm is evaluated, forming a rolling time window. This is the default evaluation window.").with_provider_name("SlidingWindow").build(),
                    StructField::optional("wall_clock_window", AttributeType::struct_("WallClockWindow".to_string(), vec![StructField::optional("timezone", AttributeType::string()).with_description("The time zone to use when the alarm aligns the evaluation window to clock boundaries. You can specify an IANA time zone name (for example, ``America/New_York``), a fixed UTC offset (for example, ``+05:30``), or an offset-prefixed identifier (for example, ``UTC+05:30``). The offset must be aligned to a multiple of 5 minutes. If you don't specify a time zone, CloudWatch uses ``UTC``. The time zone affects window alignment for all periods, including periods of one hour or shorter.").with_provider_name("Timezone").build()])).with_description("A wall clock window, which aligns the evaluated range to fixed clock boundaries that match the alarm's period, such as the top of the hour, midnight, or the start of the calendar week.").with_provider_name("WallClockWindow").build()]) }
    schema = schema.with_def("EvaluationWindow", build_schema_def_4());
    #[inline(never)]
    fn build_schema_def_5() -> AttributeType { AttributeType::struct_("Metric".to_string(), vec![StructField::optional("dimensions", AttributeType::unordered_list(AttributeType::struct_("Dimension".to_string(), vec![StructField::required("name", AttributeType::string()).with_description("The name of the dimension, from 1–255 characters in length. This dimension name must have been included when the metric was published.").with_provider_name("Name").build(),
                    StructField::required("value", AttributeType::string()).with_description("The value for the dimension, from 1–255 characters in length.").with_provider_name("Value").build()]))).with_description("The metric dimensions that you want to be used for the metric that the alarm will watch.").with_provider_name("Dimensions").with_block_name("dimension").build(),
                    StructField::optional("metric_name", AttributeType::string()).with_description("The name of the metric that you want the alarm to watch. This is a required field.").with_provider_name("MetricName").build(),
                    StructField::optional("namespace", AttributeType::string()).with_description("The namespace of the metric that the alarm will watch.").with_provider_name("Namespace").build()]) }
    schema = schema.with_def("Metric", build_schema_def_5());
    #[inline(never)]
    fn build_schema_def_6() -> AttributeType { AttributeType::struct_("MetricStat".to_string(), vec![StructField::required("metric", AttributeType::struct_("Metric".to_string(), vec![StructField::optional("dimensions", AttributeType::unordered_list(AttributeType::struct_("Dimension".to_string(), vec![StructField::required("name", AttributeType::string()).with_description("The name of the dimension, from 1–255 characters in length. This dimension name must have been included when the metric was published.").with_provider_name("Name").build(),
                    StructField::required("value", AttributeType::string()).with_description("The value for the dimension, from 1–255 characters in length.").with_provider_name("Value").build()]))).with_description("The metric dimensions that you want to be used for the metric that the alarm will watch.").with_provider_name("Dimensions").with_block_name("dimension").build(),
                    StructField::optional("metric_name", AttributeType::string()).with_description("The name of the metric that you want the alarm to watch. This is a required field.").with_provider_name("MetricName").build(),
                    StructField::optional("namespace", AttributeType::string()).with_description("The namespace of the metric that the alarm will watch.").with_provider_name("Namespace").build()])).with_description("The metric to return, including the metric name, namespace, and dimensions.").with_provider_name("Metric").with_block_name("metric").build(),
                    StructField::required("period", AttributeType::int()).with_description("The granularity, in seconds, of the returned data points. For metrics with regular resolution, a period can be as short as one minute (60 seconds) and must be a multiple of 60. For high-resolution metrics that are collected at intervals of less than one minute, the period can be 1, 5, 10, 20, 30, 60, or any multiple of 60. High-resolution metrics are those metrics stored by a ``PutMetricData`` call that includes a ``StorageResolution`` of 1 second. If the ``StartTime`` parameter specifies a time stamp that is greater than 3 hours ago, you must specify the period as follows or no data points in that time range is returned: + Start time between 3 hours and 15 days ago - Use a multiple of 60 seconds (1 minute). + Start time between 15 and 63 days ago - Use a multiple of 300 seconds (5 minutes). + Start time greater than 63 days ago - Use a multiple of 3600 seconds (1 hour).").with_provider_name("Period").build(),
                    StructField::required("stat", AttributeType::string()).with_description("The statistic to return. It can include any CW statistic or extended statistic. For a list of valid values, see the table in [Statistics](https://docs.aws.amazon.com/AmazonCloudWatch/latest/monitoring/cloudwatch_concepts.html#Statistic) in the *User Guide*.").with_provider_name("Stat").build(),
                    StructField::optional("unit", AttributeType::string()).with_description("The unit to use for the returned data points. Valid values are: Seconds, Microseconds, Milliseconds, Bytes, Kilobytes, Megabytes, Gigabytes, Terabytes, Bits, Kilobits, Megabits, Gigabits, Terabits, Percent, Count, Bytes/Second, Kilobytes/Second, Megabytes/Second, Gigabytes/Second, Terabytes/Second, Bits/Second, Kilobits/Second, Megabits/Second, Gigabits/Second, Terabits/Second, Count/Second, or None.").with_provider_name("Unit").build()]) }
    schema = schema.with_def("MetricStat", build_schema_def_6());
    #[inline(never)]
    fn build_schema_def_7() -> AttributeType { AttributeType::struct_("WallClockWindow".to_string(), vec![StructField::optional("timezone", AttributeType::string()).with_description("The time zone to use when the alarm aligns the evaluation window to clock boundaries. You can specify an IANA time zone name (for example, ``America/New_York``), a fixed UTC offset (for example, ``+05:30``), or an offset-prefixed identifier (for example, ``UTC+05:30``). The offset must be aligned to a multiple of 5 minutes. If you don't specify a time zone, CloudWatch uses ``UTC``. The time zone affects window alignment for all periods, including periods of one hour or shorter.").with_provider_name("Timezone").build()]) }
    schema = schema.with_def("WallClockWindow", build_schema_def_7());
    #[inline(never)]
    fn build_schema_def_8() -> AttributeType { AttributeType::struct_("WarmUpConfiguration".to_string(), vec![StructField::optional("only_start_evaluating_after_warm_up_period_ends", AttributeType::bool()).with_description("Specifies whether the alarm waits for the full warm-up period before it starts to evaluate. The default is ``false``. If ``true``, the alarm waits the entire ``WarmUpPeriodDurationInMinutes`` before it starts to evaluate, even if metric data arrives earlier. If ``false``, the alarm ends the warm-up period early. Evaluation begins as soon as the alarm has enough metric data to fill its evaluation window.").with_provider_name("OnlyStartEvaluatingAfterWarmUpPeriodEnds").build(),
                    StructField::optional("warm_up_period_duration_in_minutes", AttributeType::refined_int(None, Some((Some(1), Some(2880))))).with_description("The length of the warm-up period, in minutes. After you create or update the alarm, the alarm stays in ``INSUFFICIENT_DATA`` for this duration. During this time, the alarm does not perform alarm actions. You can change this value at any time, including after the warm-up period ends. If you change it after the warm-up period ends, the new value does not restart the warm-up period.").with_provider_name("WarmUpPeriodDurationInMinutes").build()]) }
    schema = schema.with_def("WarmUpConfiguration", build_schema_def_8());
    AwsccSchemaConfig {
        aws_type_name: "AWS::CloudWatch::Alarm",
        resource_type_name: "cloudwatch.Alarm",
        primary_identifier: &[crate::schemas::config::PrimaryIdentifierAttribute { provider_name: "AlarmName", dsl_name: "alarm_name" }],
        exclusive_groups: &[],
        tags: Some(TagsProperty { provider_name: "Tags", wire: TagsWire::KeyValueList }),
        schema,
    }
}

/// Returns the resource type name and all enum valid values for this module
pub fn enum_valid_values() -> (
    &'static str,
    &'static [(&'static str, &'static [&'static str])],
) {
    (
        "cloudwatch.Alarm",
        &[("treat_missing_data", VALID_TREAT_MISSING_DATA)],
    )
}

/// Returns the IAM permissions declared by the CloudFormation handler for this operation.
pub fn required_permissions(op: carina_core::effect::PlanOp) -> &'static [&'static str] {
    match op {
        carina_core::effect::PlanOp::Create => &[
            "cloudwatch:PutMetricAlarm",
            "cloudwatch:DescribeAlarms",
            "cloudwatch:TagResource",
            "cloudwatch:ListTagsForResource",
        ],
        carina_core::effect::PlanOp::Read => &[
            "cloudwatch:DescribeAlarms",
            "cloudwatch:ListTagsForResource",
        ],
        carina_core::effect::PlanOp::Update => &[
            "cloudwatch:PutMetricAlarm",
            "cloudwatch:DescribeAlarms",
            "cloudwatch:TagResource",
            "cloudwatch:UntagResource",
            "cloudwatch:ListTagsForResource",
        ],
        carina_core::effect::PlanOp::Delete => {
            &["cloudwatch:DeleteAlarms", "cloudwatch:DescribeAlarms"]
        }
    }
}
