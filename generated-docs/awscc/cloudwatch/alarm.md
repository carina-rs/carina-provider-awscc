---
title: "awscc.cloudwatch.Alarm"
---


CloudFormation Type: `AWS::CloudWatch::Alarm`

The ``AWS::CloudWatch::Alarm`` type specifies an alarm and associates it with the specified metric or metric math expression.
 When this operation creates an alarm, the alarm state is immediately set to ``INSUFFICIENT_DATA``. The alarm is then evaluated and its state is set appropriately. Any actions associated with the new state are then executed.
 When you update an existing alarm, its state is left unchanged, but the update completely overwrites the previous configuration of the alarm.

## Example

```crn
awscc.cloudwatch.Alarm {
  alarm_name        = 'carina-registry-publish-api-errors'
  alarm_description = 'Registry publish API Lambda errors'

  comparison_operator = 'GreaterThanThreshold'
  datapoints_to_alarm = 1
  evaluation_periods  = 2
  metric_name         = 'Errors'
  namespace           = 'AWS/Lambda'
  period              = 60
  statistic           = 'Sum'
  threshold           = 0.5

  dimensions = [{
    name  = 'FunctionName'
    value = 'carina-registry-publish-api'
  }]

  tags = {
    Environment = 'production'
    Project     = 'carina-registry'
  }
}
```

## Argument Reference

### `action_configuration`

- **Type:** [Struct(ActionConfiguration)](#action_configuration-1)
- **Required:** No



### `actions_enabled`

- **Type:** Bool
- **Required:** No
- **Default:** `true`

Indicates whether actions should be executed during any changes to the alarm state. The default is TRUE.

### `alarm_actions`

- **Type:** `List<String>`
- **Required:** No

The list of actions to execute when this alarm transitions into an ALARM state from any other state. Specify each action as an Amazon Resource Name (ARN). For more information about creating alarms and the actions that you can specify, see [PutMetricAlarm](https://docs.aws.amazon.com/AmazonCloudWatch/latest/APIReference/API_PutMetricAlarm.html) in the *API Reference*.

### `alarm_description`

- **Type:** String
- **Required:** No

The description of the alarm.

### `alarm_name`

- **Type:** String
- **Required:** No
- **Create-only:** Yes

The name of the alarm. If you don't specify a name, CFN generates a unique physical ID and uses that ID for the alarm name. If you specify a name, you cannot perform updates that require replacement of this resource. You can perform updates that require no or some interruption. If you must replace the resource, specify a new name.

### `comparison_operator`

- **Type:** String
- **Required:** No

The arithmetic operation to use when comparing the specified statistic and threshold. The specified statistic value is used as the first operand.

### `datapoints_to_alarm`

- **Type:** Int
- **Required:** No

The number of datapoints that must be breaching to trigger the alarm. This is used only if you are setting an "M out of N" alarm. In that case, this value is the M, and the value that you set for ``EvaluationPeriods`` is the N value. For more information, see [Evaluating an Alarm](https://docs.aws.amazon.com/AmazonCloudWatch/latest/monitoring/AlarmThatSendsEmail.html#alarm-evaluation) in the *User Guide*. If you omit this parameter, CW uses the same value here that you set for ``EvaluationPeriods``, and the alarm goes to alarm state if that many consecutive periods are breaching.

### `dimensions`

- **Type:** List\<[Struct(Dimension)](#dimension)\>
- **Required:** No

The dimensions for the metric associated with the alarm. For an alarm based on a math expression, you can't specify ``Dimensions``. Instead, you use ``Metrics``.

### `evaluate_low_sample_count_percentile`

- **Type:** String
- **Required:** No

Used only for alarms based on percentiles. If ``ignore``, the alarm state does not change during periods with too few data points to be statistically significant. If ``evaluate`` or this parameter is not used, the alarm is always evaluated and possibly changes state no matter how many data points are available.

### `evaluation_criteria`

- **Type:** [Struct(EvaluationCriteria)](#evaluation_criteria-1)
- **Required:** No

The evaluation criteria for the alarm.

### `evaluation_interval`

- **Type:** Int
- **Required:** No

The frequency, in seconds, at which the alarm is evaluated.

### `evaluation_periods`

- **Type:** Int
- **Required:** No

The number of periods over which data is compared to the specified threshold. If you are setting an alarm that requires that a number of consecutive data points be breaching to trigger the alarm, this value specifies that number. If you are setting an "M out of N" alarm, this value is the N, and ``DatapointsToAlarm`` is the M. For more information, see [Evaluating an Alarm](https://docs.aws.amazon.com/AmazonCloudWatch/latest/monitoring/AlarmThatSendsEmail.html#alarm-evaluation) in the *User Guide*.

### `evaluation_window`

- **Type:** [Struct(EvaluationWindow)](#evaluation_window-1)
- **Required:** No

The evaluation window that the alarm uses to select the range of metric data that it evaluates. This is either a sliding window or a wall clock window. For more information, see [Alarm evaluation windows](https://docs.aws.amazon.com/AmazonCloudWatch/latest/monitoring/alarm-evaluation-window.html) in the *CloudWatch User Guide*.

### `extended_statistic`

- **Type:** String
- **Required:** No

The percentile statistic for the metric associated with the alarm. Specify a value between p0.0 and p100. For an alarm based on a metric, you must specify either ``Statistic`` or ``ExtendedStatistic`` but not both. For an alarm based on a math expression, you can't specify ``ExtendedStatistic``. Instead, you use ``Metrics``.

### `insufficient_data_actions`

- **Type:** `List<String>`
- **Required:** No

The actions to execute when this alarm transitions to the ``INSUFFICIENT_DATA`` state from any other state. Each action is specified as an Amazon Resource Name (ARN).

### `metric_name`

- **Type:** String
- **Required:** No

The name of the metric associated with the alarm. This is required for an alarm based on a metric. For an alarm based on a math expression, you use ``Metrics`` instead and you can't specify ``MetricName``.

### `metrics`

- **Type:** List\<[Struct(MetricDataQuery)](#metrics-1)\>
- **Required:** No

An array that enables you to create an alarm based on the result of a metric math expression. Each item in the array either retrieves a metric or performs a math expression. If you specify the ``Metrics`` parameter, you cannot specify ``MetricName``, ``Dimensions``, ``Period``, ``Namespace``, ``Statistic``, ``ExtendedStatistic``, or ``Unit``.

### `namespace`

- **Type:** String
- **Required:** No

The namespace of the metric associated with the alarm. This is required for an alarm based on a metric. For an alarm based on a math expression, you can't specify ``Namespace`` and you use ``Metrics`` instead. For a list of namespaces for metrics from AWS services, see [Services That Publish Metrics.](https://docs.aws.amazon.com/AmazonCloudWatch/latest/monitoring/aws-services-cloudwatch-metrics.html)

### `ok_actions`

- **Type:** `List<String>`
- **Required:** No

The actions to execute when this alarm transitions to the ``OK`` state from any other state. Each action is specified as an Amazon Resource Name (ARN).

### `period`

- **Type:** Int
- **Required:** No

The period, in seconds, over which the statistic is applied. This is required for an alarm based on a metric. Valid values are 10, 20, 30, 60, and any multiple of 60. For an alarm based on a math expression, you can't specify ``Period``, and instead you use the ``Metrics`` parameter. *Minimum:* 10

### `statistic`

- **Type:** String
- **Required:** No

The statistic for the metric associated with the alarm, other than percentile. For percentile statistics, use ``ExtendedStatistic``. For an alarm based on a metric, you must specify either ``Statistic`` or ``ExtendedStatistic`` but not both. For an alarm based on a math expression, you can't specify ``Statistic``. Instead, you use ``Metrics``.

### `tags`

- **Type:** `Map<String, String>`
- **Required:** No

A list of key-value pairs to associate with the alarm. You can associate as many as 50 tags with an alarm. To be able to associate tags with the alarm when you create the alarm, you must have the ``cloudwatch:TagResource`` permission. Tags can help you organize and categorize your resources. You can also use them to scope user permissions by granting a user permission to access or change only resources with certain tag values.

### `threshold`

- **Type:** Float
- **Required:** No

The value to compare with the specified statistic.

### `threshold_metric_id`

- **Type:** String
- **Required:** No

In an alarm based on an anomaly detection model, this is the ID of the ``ANOMALY_DETECTION_BAND`` function used as the threshold for the alarm.

### `treat_missing_data`

- **Type:** [Enum (TreatMissingData)](#treat_missing_data-treatmissingdata)
- **Required:** No

Sets how this alarm is to handle missing data points. Valid values are ``breaching``, ``notBreaching``, ``ignore``, and ``missing``. For more information, see [Configuring How Alarms Treat Missing Data](https://docs.aws.amazon.com/AmazonCloudWatch/latest/monitoring/AlarmThatSendsEmail.html#alarms-and-missing-data) in the *Amazon User Guide*. If you omit this parameter, the default behavior of ``missing`` is used.

### `unit`

- **Type:** String
- **Required:** No

The unit of the metric associated with the alarm. Specify this only if you are creating an alarm based on a single metric. Do not specify this if you are specifying a ``Metrics`` array. You can specify the following values: Seconds, Microseconds, Milliseconds, Bytes, Kilobytes, Megabytes, Gigabytes, Terabytes, Bits, Kilobits, Megabits, Gigabits, Terabits, Percent, Count, Bytes/Second, Kilobytes/Second, Megabytes/Second, Gigabytes/Second, Terabytes/Second, Bits/Second, Kilobits/Second, Megabits/Second, Gigabits/Second, Terabits/Second, Count/Second, or None.

### `warm_up_configuration`

- **Type:** [Struct(WarmUpConfiguration)](#warm_up_configuration-1)
- **Required:** No

The warm-up configuration for the alarm. A warm-up period delays alarm evaluation after you create or update the alarm. During the warm-up period, the alarm stays in ``INSUFFICIENT_DATA`` and does not perform alarm actions. For more information, see [Alarm warm-up periods](https://docs.aws.amazon.com/AmazonCloudWatch/latest/monitoring/alarm-warm-up.html) in the *Amazon CloudWatch User Guide*.

## Enum Values

### treat_missing_data (TreatMissingData)

| Value | DSL Identifier |
|-------|----------------|
| `breaching` | `aws.cloudwatch.Alarm.TreatMissingData.breaching` |
| `notBreaching` | `aws.cloudwatch.Alarm.TreatMissingData.not_breaching` |
| `ignore` | `aws.cloudwatch.Alarm.TreatMissingData.ignore` |
| `missing` | `aws.cloudwatch.Alarm.TreatMissingData.missing` |

Shorthand formats: `breaching` or `TreatMissingData.breaching`

## Struct Definitions

### action_configuration

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `alarm_tags_propagation` | [Struct(AlarmTagsPropagation)](#action_configurationalarm_tags_propagation) | No | Whether the alarm's tags are included in its action notifications and state-change events. |

### action_configuration.alarm_tags_propagation

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `enabled` | Bool | Yes | Whether the alarm's tags are included in its action notifications and state-change events. |
| `excluded_tag_keys` | `List<String>` (items: 1..=50) | No | The tag keys to leave out. Cannot be specified together with IncludedTagKeys. |
| `included_tag_keys` | `List<String>` (items: 1..=50) | No | The only tag keys to include. Cannot be specified together with ExcludedTagKeys. |

### dimension

Used by: `dimensions`, `metrics.metric_stat.metric.dimensions`

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `name` | String | Yes | The name of the dimension, from 1–255 characters in length. This dimension name must have been included when the metric was published. |
| `value` | String | Yes | The value for the dimension, from 1–255 characters in length. |

### evaluation_criteria

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `prom_ql_criteria` | [Struct(AlarmPromQLCriteria)](#evaluation_criteriaprom_ql_criteria) | No | The PromQL criteria for the alarm evaluation. |

### evaluation_criteria.prom_ql_criteria

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `pending_period` | Int | No | The duration, in seconds, that a contributor must be continuously breaching before it transitions to the ``ALARM`` state. |
| `query` | String | No | The PromQL query that the alarm evaluates. The query must return a result of vector type. Each entry in the vector result represents an alarm contributor. |
| `recovery_period` | Int | No | The duration, in seconds, that a contributor must continuously not be breaching before it transitions back to the ``OK`` state. |

### evaluation_window

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `sliding_window` | Empty struct (`{}`) | No | A sliding window, which advances each time the alarm is evaluated, forming a rolling time window. This is the default evaluation window. |
| `wall_clock_window` | [Struct(WallClockWindow)](#evaluation_windowwall_clock_window) | No | A wall clock window, which aligns the evaluated range to fixed clock boundaries that match the alarm's period, such as the top of the hour, midnight, or the start of the calendar week. |

### evaluation_window.wall_clock_window

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `timezone` | String | No | The time zone to use when the alarm aligns the evaluation window to clock boundaries. You can specify an IANA time zone name (for example, ``America/New_York``), a fixed UTC offset (for example, ``+05:30``), or an offset-prefixed identifier (for example, ``UTC+05:30``). The offset must be aligned to a multiple of 5 minutes. If you don't specify a time zone, CloudWatch uses ``UTC``. The time zone affects window alignment for all periods, including periods of one hour or shorter. |

### metrics

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `account_id` | AwsAccountId | No | The ID of the account where the metrics are located, if this is a cross-account alarm. |
| `expression` | String | No | The math expression to be performed on the returned data, if this object is performing a math expression. This expression can use the ``Id`` of the other metrics to refer to those metrics, and can also use the ``Id`` of other expressions to use the result of those expressions. For more information about metric math expressions, see [Metric Math Syntax and Functions](https://docs.aws.amazon.com/AmazonCloudWatch/latest/monitoring/using-metric-math.html#metric-math-syntax) in the *User Guide*. Within each MetricDataQuery object, you must specify either ``Expression`` or ``MetricStat`` but not both. |
| `id` | String | Yes | A short name used to tie this object to the results in the response. This name must be unique within a single call to ``GetMetricData``. If you are performing math expressions on this set of data, this name represents that data and can serve as a variable in the mathematical expression. The valid characters are letters, numbers, and underscore. The first character must be a lowercase letter. |
| `label` | String | No | A human-readable label for this metric or expression. This is especially useful if this is an expression, so that you know what the value represents. If the metric or expression is shown in a CW dashboard widget, the label is shown. If ``Label`` is omitted, CW generates a default. |
| `metric_stat` | [Struct(MetricStat)](#metricsmetric_stat) | No | The metric to be returned, along with statistics, period, and units. Use this parameter only if this object is retrieving a metric and not performing a math expression on returned data. Within one MetricDataQuery object, you must specify either ``Expression`` or ``MetricStat`` but not both. |
| `period` | Int | No | The granularity, in seconds, of the returned data points. For metrics with regular resolution, a period can be as short as one minute (60 seconds) and must be a multiple of 60. For high-resolution metrics that are collected at intervals of less than one minute, the period can be 1, 5, 10, 20, 30, 60, or any multiple of 60. High-resolution metrics are those metrics stored by a ``PutMetricData`` operation that includes a ``StorageResolution of 1 second``. |
| `return_data` | Bool | No | This option indicates whether to return the timestamps and raw data values of this metric. When you create an alarm based on a metric math expression, specify ``True`` for this value for only the one math expression that the alarm is based on. You must specify ``False`` for ``ReturnData`` for all the other metrics and expressions used in the alarm. This field is required. |

### metrics.metric_stat

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `metric` | [Struct(Metric)](#metricsmetric_statmetric) | Yes | The metric to return, including the metric name, namespace, and dimensions. |
| `period` | Int | Yes | The granularity, in seconds, of the returned data points. For metrics with regular resolution, a period can be as short as one minute (60 seconds) and must be a multiple of 60. For high-resolution metrics that are collected at intervals of less than one minute, the period can be 1, 5, 10, 20, 30, 60, or any multiple of 60. High-resolution metrics are those metrics stored by a ``PutMetricData`` call that includes a ``StorageResolution`` of 1 second. If the ``StartTime`` parameter specifies a time stamp that is greater than 3 hours ago, you must specify the period as follows or no data points in that time range is returned: + Start time between 3 hours and 15 days ago - Use a multiple of 60 seconds (1 minute). + Start time between 15 and 63 days ago - Use a multiple of 300 seconds (5 minutes). + Start time greater than 63 days ago - Use a multiple of 3600 seconds (1 hour). |
| `stat` | String | Yes | The statistic to return. It can include any CW statistic or extended statistic. For a list of valid values, see the table in [Statistics](https://docs.aws.amazon.com/AmazonCloudWatch/latest/monitoring/cloudwatch_concepts.html#Statistic) in the *User Guide*. |
| `unit` | String | No | The unit to use for the returned data points. Valid values are: Seconds, Microseconds, Milliseconds, Bytes, Kilobytes, Megabytes, Gigabytes, Terabytes, Bits, Kilobits, Megabits, Gigabits, Terabits, Percent, Count, Bytes/Second, Kilobytes/Second, Megabytes/Second, Gigabytes/Second, Terabytes/Second, Bits/Second, Kilobits/Second, Megabits/Second, Gigabits/Second, Terabits/Second, Count/Second, or None. |

### metrics.metric_stat.metric

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `dimensions` | List\<[Struct(Dimension)](#dimension)\> | No | The metric dimensions that you want to be used for the metric that the alarm will watch. |
| `metric_name` | String | No | The name of the metric that you want the alarm to watch. This is a required field. |
| `namespace` | String | No | The namespace of the metric that the alarm will watch. |

### warm_up_configuration

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `only_start_evaluating_after_warm_up_period_ends` | Bool | No | Specifies whether the alarm waits for the full warm-up period before it starts to evaluate. The default is ``false``. If ``true``, the alarm waits the entire ``WarmUpPeriodDurationInMinutes`` before it starts to evaluate, even if metric data arrives earlier. If ``false``, the alarm ends the warm-up period early. Evaluation begins as soon as the alarm has enough metric data to fill its evaluation window. |
| `warm_up_period_duration_in_minutes` | Int(1..=2880) | No | The length of the warm-up period, in minutes. After you create or update the alarm, the alarm stays in ``INSUFFICIENT_DATA`` for this duration. During this time, the alarm does not perform alarm actions. You can change this value at any time, including after the warm-up period ends. If you change it after the warm-up period ends, the new value does not restart the warm-up period. |

## Attribute Reference

### `arn`

- **Type:** Arn



