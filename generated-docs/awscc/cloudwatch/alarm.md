---
title: "awscc.cloudwatch.Alarm"
---


CloudFormation Type: `AWS::CloudWatch::Alarm`

The ``AWS::CloudWatch::Alarm`` type specifies an alarm and associates it with the specified metric or metric math expression.
 When this operation creates an alarm, the alarm state is immediately set to ``INSUFFICIENT_DATA``. The alarm is then evaluated and its state is set appropriately. Any actions associated with the new state are then executed.
 When you update an existing alarm, its state is left unchanged, but the update completely overwrites the previous configuration of the alarm.

## Example

```crn
let alarm_topic = awscc.sns.Topic {
  topic_name = 'carina-registry-publish-api-alarms'
}

awscc.cloudwatch.Alarm {
  alarm_name        = 'carina-registry-publish-api-errors'
  alarm_description = 'Registry publish API Lambda errors'

  alarm_actions             = [alarm_topic.topic_arn]
  ok_actions                = [alarm_topic.topic_arn]
  insufficient_data_actions = [alarm_topic.topic_arn]

  comparison_operator = greater_than_threshold
  datapoints_to_alarm = 1
  evaluation_periods  = 2
  metric_name         = 'Errors'
  namespace           = 'AWS/Lambda'
  period              = 60
  statistic           = sum
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

- **Type:** `List<aws.Arn — identity: String(pattern); member: String>` (items: ..=5)
- **Required:** No

The list of actions to execute when this alarm transitions into an ALARM state from any other state. Specify each action as an Amazon Resource Name (ARN). For more information about creating alarms and the actions that you can specify, see [PutMetricAlarm](https://docs.aws.amazon.com/AmazonCloudWatch/latest/APIReference/API_PutMetricAlarm.html) in the *API Reference*.

### `alarm_description`

- **Type:** `String(len: ..=1024)`
- **Required:** No

The description of the alarm.

### `alarm_name`

- **Type:** `String(pattern, len: 1..=255)`
- **Required:** No
- **Create-only:** Yes

The name of the alarm. If you don't specify a name, CFN generates a unique physical ID and uses that ID for the alarm name. If you specify a name, you cannot perform updates that require replacement of this resource. You can perform updates that require no or some interruption. If you must replace the resource, specify a new name.

### `comparison_operator`

- **Type:** [Enum (ComparisonOperator)](#comparison_operator-comparisonoperator)
- **Required:** No

The arithmetic operation to use when comparing the specified statistic and threshold. The specified statistic value is used as the first operand.

### `datapoints_to_alarm`

- **Type:** `Int(1..)`
- **Required:** No

The number of datapoints that must be breaching to trigger the alarm. This is used only if you are setting an "M out of N" alarm. In that case, this value is the M, and the value that you set for ``EvaluationPeriods`` is the N value. For more information, see [Evaluating an Alarm](https://docs.aws.amazon.com/AmazonCloudWatch/latest/monitoring/AlarmThatSendsEmail.html#alarm-evaluation) in the *User Guide*. If you omit this parameter, CW uses the same value here that you set for ``EvaluationPeriods``, and the alarm goes to alarm state if that many consecutive periods are breaching.

### `dimensions`

- **Type:** List\<[Struct(Dimension)](#dimension)\>
- **Required:** No

The dimensions for the metric associated with the alarm. For an alarm based on a math expression, you can't specify ``Dimensions``. Instead, you use ``Metrics``.

### `evaluate_low_sample_count_percentile`

- **Type:** [Enum (EvaluateLowSampleCountPercentile)](#evaluate_low_sample_count_percentile-evaluatelowsamplecountpercentile)
- **Required:** No

Used only for alarms based on percentiles. If ``ignore``, the alarm state does not change during periods with too few data points to be statistically significant. If ``evaluate`` or this parameter is not used, the alarm is always evaluated and possibly changes state no matter how many data points are available.

### `evaluation_criteria`

- **Type:** [Struct(EvaluationCriteria)](#evaluation_criteria-1)
- **Required:** No

The evaluation criteria for the alarm.

### `evaluation_interval`

- **Type:** IntEnum([10, 20, 30, 60, 120, 180, 240, 300, 360, 420, 480, 540, 600, 660, 720, 780, 840, 900, 960, 1020, 1080, 1140, 1200, 1260, 1320, 1380, 1440, 1500, 1560, 1620, 1680, 1740, 1800, 1860, 1920, 1980, 2040, 2100, 2160, 2220, 2280, 2340, 2400, 2460, 2520, 2580, 2640, 2700, 2760, 2820, 2880, 2940, 3000, 3060, 3120, 3180, 3240, 3300, 3360, 3420, 3480, 3540, 3600])
- **Required:** No

The frequency, in seconds, at which the alarm is evaluated.

### `evaluation_periods`

- **Type:** `Int(1..)`
- **Required:** No

The number of periods over which data is compared to the specified threshold. If you are setting an alarm that requires that a number of consecutive data points be breaching to trigger the alarm, this value specifies that number. If you are setting an "M out of N" alarm, this value is the N, and ``DatapointsToAlarm`` is the M. For more information, see [Evaluating an Alarm](https://docs.aws.amazon.com/AmazonCloudWatch/latest/monitoring/AlarmThatSendsEmail.html#alarm-evaluation) in the *User Guide*.

### `evaluation_window`

- **Type:** [Struct(EvaluationWindow)](#evaluation_window-1)
- **Required:** No

The evaluation window that the alarm uses to select the range of metric data that it evaluates. This is either a sliding window or a wall clock window. For more information, see [Alarm evaluation windows](https://docs.aws.amazon.com/AmazonCloudWatch/latest/monitoring/alarm-evaluation-window.html) in the *CloudWatch User Guide*.

### `extended_statistic`

- **Type:** `String(len: 1..)`
- **Required:** No

The percentile statistic for the metric associated with the alarm. Specify a value between p0.0 and p100. For an alarm based on a metric, you must specify either ``Statistic`` or ``ExtendedStatistic`` but not both. For an alarm based on a math expression, you can't specify ``ExtendedStatistic``. Instead, you use ``Metrics``.

### `insufficient_data_actions`

- **Type:** `List<aws.Arn — identity: String(pattern); member: String>` (items: ..=5)
- **Required:** No

The actions to execute when this alarm transitions to the ``INSUFFICIENT_DATA`` state from any other state. Each action is specified as an Amazon Resource Name (ARN).

### `metric_name`

- **Type:** `String(len: 1..=255)`
- **Required:** No

The name of the metric associated with the alarm. This is required for an alarm based on a metric. For an alarm based on a math expression, you use ``Metrics`` instead and you can't specify ``MetricName``.

### `metrics`

- **Type:** List\<[Struct(MetricDataQuery)](#metrics-1)\> (unique items)
- **Required:** No

An array that enables you to create an alarm based on the result of a metric math expression. Each item in the array either retrieves a metric or performs a math expression. If you specify the ``Metrics`` parameter, you cannot specify ``MetricName``, ``Dimensions``, ``Period``, ``Namespace``, ``Statistic``, ``ExtendedStatistic``, or ``Unit``.

### `namespace`

- **Type:** `String(pattern, len: 1..=255)`
- **Required:** No

The namespace of the metric associated with the alarm. This is required for an alarm based on a metric. For an alarm based on a math expression, you can't specify ``Namespace`` and you use ``Metrics`` instead. For a list of namespaces for metrics from AWS services, see [Services That Publish Metrics.](https://docs.aws.amazon.com/AmazonCloudWatch/latest/monitoring/aws-services-cloudwatch-metrics.html)

### `ok_actions`

- **Type:** `List<aws.Arn — identity: String(pattern); member: String>` (items: ..=5)
- **Required:** No

The actions to execute when this alarm transitions to the ``OK`` state from any other state. Each action is specified as an Amazon Resource Name (ARN).

### `period`

- **Type:** `Int(10..)`
- **Required:** No

The period, in seconds, over which the statistic is applied. This is required for an alarm based on a metric. Valid values are 10, 20, 30, 60, and any multiple of 60. For an alarm based on a math expression, you can't specify ``Period``, and instead you use the ``Metrics`` parameter. *Minimum:* 10

### `statistic`

- **Type:** [Enum (Statistic)](#statistic-statistic)
- **Required:** No

The statistic for the metric associated with the alarm, other than percentile. For percentile statistics, use ``ExtendedStatistic``. For an alarm based on a metric, you must specify either ``Statistic`` or ``ExtendedStatistic`` but not both. For an alarm based on a math expression, you can't specify ``Statistic``. Instead, you use ``Metrics``.

### `tags`

- **Type:** `Map<String(len: 1..=128), String(len: 1..=256)>` (entries: ..=50)
- **Required:** No

A list of key-value pairs to associate with the alarm. You can associate as many as 50 tags with an alarm. To be able to associate tags with the alarm when you create the alarm, you must have the ``cloudwatch:TagResource`` permission. Tags can help you organize and categorize your resources. You can also use them to scope user permissions by granting a user permission to access or change only resources with certain tag values.

### `threshold`

- **Type:** Float
- **Required:** No

The value to compare with the specified statistic.

### `threshold_metric_id`

- **Type:** `String(len: 1..=255)`
- **Required:** No

In an alarm based on an anomaly detection model, this is the ID of the ``ANOMALY_DETECTION_BAND`` function used as the threshold for the alarm.

### `treat_missing_data`

- **Type:** [Enum (TreatMissingData)](#treat_missing_data-treatmissingdata)
- **Required:** No

Sets how this alarm is to handle missing data points. Valid values are ``breaching``, ``notBreaching``, ``ignore``, and ``missing``. For more information, see [Configuring How Alarms Treat Missing Data](https://docs.aws.amazon.com/AmazonCloudWatch/latest/monitoring/AlarmThatSendsEmail.html#alarms-and-missing-data) in the *Amazon User Guide*. If you omit this parameter, the default behavior of ``missing`` is used.

### `unit`

- **Type:** [Enum (Unit)](#unit-unit)
- **Required:** No

The unit of the metric associated with the alarm. Specify this only if you are creating an alarm based on a single metric. Do not specify this if you are specifying a ``Metrics`` array. You can specify the following values: Seconds, Microseconds, Milliseconds, Bytes, Kilobytes, Megabytes, Gigabytes, Terabytes, Bits, Kilobits, Megabits, Gigabits, Terabits, Percent, Count, Bytes/Second, Kilobytes/Second, Megabytes/Second, Gigabytes/Second, Terabytes/Second, Bits/Second, Kilobits/Second, Megabits/Second, Gigabits/Second, Terabits/Second, Count/Second, or None.

### `warm_up_configuration`

- **Type:** [Struct(WarmUpConfiguration)](#warm_up_configuration-1)
- **Required:** No

The warm-up configuration for the alarm. A warm-up period delays alarm evaluation after you create or update the alarm. During the warm-up period, the alarm stays in ``INSUFFICIENT_DATA`` and does not perform alarm actions. For more information, see [Alarm warm-up periods](https://docs.aws.amazon.com/AmazonCloudWatch/latest/monitoring/alarm-warm-up.html) in the *Amazon CloudWatch User Guide*.

## Enum Values

### comparison_operator (ComparisonOperator)

| Value | DSL Identifier |
|-------|----------------|
| `GreaterThanOrEqualToThreshold` | `aws.cloudwatch.Alarm.ComparisonOperator.greater_than_or_equal_to_threshold` |
| `GreaterThanThreshold` | `aws.cloudwatch.Alarm.ComparisonOperator.greater_than_threshold` |
| `LessThanThreshold` | `aws.cloudwatch.Alarm.ComparisonOperator.less_than_threshold` |
| `LessThanOrEqualToThreshold` | `aws.cloudwatch.Alarm.ComparisonOperator.less_than_or_equal_to_threshold` |
| `LessThanLowerOrGreaterThanUpperThreshold` | `aws.cloudwatch.Alarm.ComparisonOperator.less_than_lower_or_greater_than_upper_threshold` |
| `LessThanLowerThreshold` | `aws.cloudwatch.Alarm.ComparisonOperator.less_than_lower_threshold` |
| `GreaterThanUpperThreshold` | `aws.cloudwatch.Alarm.ComparisonOperator.greater_than_upper_threshold` |

Shorthand formats: `greater_than_or_equal_to_threshold` or `ComparisonOperator.greater_than_or_equal_to_threshold`

### evaluate_low_sample_count_percentile (EvaluateLowSampleCountPercentile)

| Value | DSL Identifier |
|-------|----------------|
| `evaluate` | `aws.cloudwatch.Alarm.EvaluateLowSampleCountPercentile.evaluate` |
| `ignore` | `aws.cloudwatch.Alarm.EvaluateLowSampleCountPercentile.ignore` |

Shorthand formats: `evaluate` or `EvaluateLowSampleCountPercentile.evaluate`

### metrics.metric_stat.unit (Unit)

| Value | DSL Identifier |
|-------|----------------|
| `Seconds` | `aws.cloudwatch.Alarm.MetricDataQuery.MetricStat.Unit.seconds` |
| `Microseconds` | `aws.cloudwatch.Alarm.MetricDataQuery.MetricStat.Unit.microseconds` |
| `Milliseconds` | `aws.cloudwatch.Alarm.MetricDataQuery.MetricStat.Unit.milliseconds` |
| `Bytes` | `aws.cloudwatch.Alarm.MetricDataQuery.MetricStat.Unit.bytes` |
| `Kilobytes` | `aws.cloudwatch.Alarm.MetricDataQuery.MetricStat.Unit.kilobytes` |
| `Megabytes` | `aws.cloudwatch.Alarm.MetricDataQuery.MetricStat.Unit.megabytes` |
| `Gigabytes` | `aws.cloudwatch.Alarm.MetricDataQuery.MetricStat.Unit.gigabytes` |
| `Terabytes` | `aws.cloudwatch.Alarm.MetricDataQuery.MetricStat.Unit.terabytes` |
| `Bits` | `aws.cloudwatch.Alarm.MetricDataQuery.MetricStat.Unit.bits` |
| `Kilobits` | `aws.cloudwatch.Alarm.MetricDataQuery.MetricStat.Unit.kilobits` |
| `Megabits` | `aws.cloudwatch.Alarm.MetricDataQuery.MetricStat.Unit.megabits` |
| `Gigabits` | `aws.cloudwatch.Alarm.MetricDataQuery.MetricStat.Unit.gigabits` |
| `Terabits` | `aws.cloudwatch.Alarm.MetricDataQuery.MetricStat.Unit.terabits` |
| `Percent` | `aws.cloudwatch.Alarm.MetricDataQuery.MetricStat.Unit.percent` |
| `Count` | `aws.cloudwatch.Alarm.MetricDataQuery.MetricStat.Unit.count` |
| `Bytes/Second` | `aws.cloudwatch.Alarm.MetricDataQuery.MetricStat.Unit.bytes_second` |
| `Kilobytes/Second` | `aws.cloudwatch.Alarm.MetricDataQuery.MetricStat.Unit.kilobytes_second` |
| `Megabytes/Second` | `aws.cloudwatch.Alarm.MetricDataQuery.MetricStat.Unit.megabytes_second` |
| `Gigabytes/Second` | `aws.cloudwatch.Alarm.MetricDataQuery.MetricStat.Unit.gigabytes_second` |
| `Terabytes/Second` | `aws.cloudwatch.Alarm.MetricDataQuery.MetricStat.Unit.terabytes_second` |
| `Bits/Second` | `aws.cloudwatch.Alarm.MetricDataQuery.MetricStat.Unit.bits_second` |
| `Kilobits/Second` | `aws.cloudwatch.Alarm.MetricDataQuery.MetricStat.Unit.kilobits_second` |
| `Megabits/Second` | `aws.cloudwatch.Alarm.MetricDataQuery.MetricStat.Unit.megabits_second` |
| `Gigabits/Second` | `aws.cloudwatch.Alarm.MetricDataQuery.MetricStat.Unit.gigabits_second` |
| `Terabits/Second` | `aws.cloudwatch.Alarm.MetricDataQuery.MetricStat.Unit.terabits_second` |
| `Count/Second` | `aws.cloudwatch.Alarm.MetricDataQuery.MetricStat.Unit.count_second` |
| `None` | `aws.cloudwatch.Alarm.MetricDataQuery.MetricStat.Unit.none` |

Shorthand formats: `seconds` or `Unit.seconds`

### statistic (Statistic)

| Value | DSL Identifier |
|-------|----------------|
| `SampleCount` | `aws.cloudwatch.Alarm.Statistic.sample_count` |
| `Average` | `aws.cloudwatch.Alarm.Statistic.average` |
| `Sum` | `aws.cloudwatch.Alarm.Statistic.sum` |
| `Minimum` | `aws.cloudwatch.Alarm.Statistic.minimum` |
| `Maximum` | `aws.cloudwatch.Alarm.Statistic.maximum` |

Shorthand formats: `sample_count` or `Statistic.sample_count`

### treat_missing_data (TreatMissingData)

| Value | DSL Identifier |
|-------|----------------|
| `breaching` | `aws.cloudwatch.Alarm.TreatMissingData.breaching` |
| `notBreaching` | `aws.cloudwatch.Alarm.TreatMissingData.not_breaching` |
| `ignore` | `aws.cloudwatch.Alarm.TreatMissingData.ignore` |
| `missing` | `aws.cloudwatch.Alarm.TreatMissingData.missing` |

Shorthand formats: `breaching` or `TreatMissingData.breaching`

### unit (Unit)

| Value | DSL Identifier |
|-------|----------------|
| `Seconds` | `aws.cloudwatch.Alarm.Unit.seconds` |
| `Microseconds` | `aws.cloudwatch.Alarm.Unit.microseconds` |
| `Milliseconds` | `aws.cloudwatch.Alarm.Unit.milliseconds` |
| `Bytes` | `aws.cloudwatch.Alarm.Unit.bytes` |
| `Kilobytes` | `aws.cloudwatch.Alarm.Unit.kilobytes` |
| `Megabytes` | `aws.cloudwatch.Alarm.Unit.megabytes` |
| `Gigabytes` | `aws.cloudwatch.Alarm.Unit.gigabytes` |
| `Terabytes` | `aws.cloudwatch.Alarm.Unit.terabytes` |
| `Bits` | `aws.cloudwatch.Alarm.Unit.bits` |
| `Kilobits` | `aws.cloudwatch.Alarm.Unit.kilobits` |
| `Megabits` | `aws.cloudwatch.Alarm.Unit.megabits` |
| `Gigabits` | `aws.cloudwatch.Alarm.Unit.gigabits` |
| `Terabits` | `aws.cloudwatch.Alarm.Unit.terabits` |
| `Percent` | `aws.cloudwatch.Alarm.Unit.percent` |
| `Count` | `aws.cloudwatch.Alarm.Unit.count` |
| `Bytes/Second` | `aws.cloudwatch.Alarm.Unit.bytes_second` |
| `Kilobytes/Second` | `aws.cloudwatch.Alarm.Unit.kilobytes_second` |
| `Megabytes/Second` | `aws.cloudwatch.Alarm.Unit.megabytes_second` |
| `Gigabytes/Second` | `aws.cloudwatch.Alarm.Unit.gigabytes_second` |
| `Terabytes/Second` | `aws.cloudwatch.Alarm.Unit.terabytes_second` |
| `Bits/Second` | `aws.cloudwatch.Alarm.Unit.bits_second` |
| `Kilobits/Second` | `aws.cloudwatch.Alarm.Unit.kilobits_second` |
| `Megabits/Second` | `aws.cloudwatch.Alarm.Unit.megabits_second` |
| `Gigabits/Second` | `aws.cloudwatch.Alarm.Unit.gigabits_second` |
| `Terabits/Second` | `aws.cloudwatch.Alarm.Unit.terabits_second` |
| `Count/Second` | `aws.cloudwatch.Alarm.Unit.count_second` |
| `None` | `aws.cloudwatch.Alarm.Unit.none` |

Shorthand formats: `seconds` or `Unit.seconds`

## Struct Definitions

### action_configuration

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `alarm_tags_propagation` | [Struct(AlarmTagsPropagation)](#action_configurationalarm_tags_propagation) | No | Whether the alarm's tags are included in its action notifications and state-change events. |

### action_configuration.alarm_tags_propagation

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `enabled` | Bool | Yes | Whether the alarm's tags are included in its action notifications and state-change events. |
| `excluded_tag_keys` | `List<String(len: 1..=128)>` (items: 1..=50) (unique items) | No | The tag keys to leave out. Cannot be specified together with IncludedTagKeys. |
| `included_tag_keys` | `List<String(len: 1..=128)>` (items: 1..=50) (unique items) | No | The only tag keys to include. Cannot be specified together with ExcludedTagKeys. |

### dimension

Used by: `dimensions`, `metrics.metric_stat.metric.dimensions`

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `name` | `String(pattern, len: 1..=255)` | Yes | The name of the dimension, from 1–255 characters in length. This dimension name must have been included when the metric was published. |
| `value` | `String(pattern, len: 1..=1024)` | Yes | The value for the dimension, from 1–255 characters in length. |

### evaluation_criteria

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `prom_ql_criteria` | [Struct(AlarmPromQLCriteria)](#evaluation_criteriaprom_ql_criteria) | No | The PromQL criteria for the alarm evaluation. |

### evaluation_criteria.prom_ql_criteria

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `pending_period` | `Int(0..=86400)` | No | The duration, in seconds, that a contributor must be continuously breaching before it transitions to the ``ALARM`` state. |
| `query` | `String(len: 1..=10000)` | No | The PromQL query that the alarm evaluates. The query must return a result of vector type. Each entry in the vector result represents an alarm contributor. |
| `recovery_period` | `Int(0..=86400)` | No | The duration, in seconds, that a contributor must continuously not be breaching before it transitions back to the ``OK`` state. |

### evaluation_window

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `sliding_window` | Empty struct (`{}`) | No | A sliding window, which advances each time the alarm is evaluated, forming a rolling time window. This is the default evaluation window. |
| `wall_clock_window` | [Struct(WallClockWindow)](#evaluation_windowwall_clock_window) | No | A wall clock window, which aligns the evaluated range to fixed clock boundaries that match the alarm's period, such as the top of the hour, midnight, or the start of the calendar week. |

Exactly one of `sliding_window`, `wall_clock_window` must be specified.

### evaluation_window.wall_clock_window

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `timezone` | `String(len: 1..=50)` | No | The time zone to use when the alarm aligns the evaluation window to clock boundaries. You can specify an IANA time zone name (for example, ``America/New_York``), a fixed UTC offset (for example, ``+05:30``), or an offset-prefixed identifier (for example, ``UTC+05:30``). The offset must be aligned to a multiple of 5 minutes. If you don't specify a time zone, CloudWatch uses ``UTC``. The time zone affects window alignment for all periods, including periods of one hour or shorter. |

### metrics

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `account_id` | aws.AccountId — identity: String(pattern, len: 12..=12); member: String | No | The ID of the account where the metrics are located, if this is a cross-account alarm. |
| `expression` | `String(len: 1..=2048)` | No | The math expression to be performed on the returned data, if this object is performing a math expression. This expression can use the ``Id`` of the other metrics to refer to those metrics, and can also use the ``Id`` of other expressions to use the result of those expressions. For more information about metric math expressions, see [Metric Math Syntax and Functions](https://docs.aws.amazon.com/AmazonCloudWatch/latest/monitoring/using-metric-math.html#metric-math-syntax) in the *User Guide*. Within each MetricDataQuery object, you must specify either ``Expression`` or ``MetricStat`` but not both. |
| `id` | `String(pattern, len: 1..=255)` | Yes | A short name used to tie this object to the results in the response. This name must be unique within a single call to ``GetMetricData``. If you are performing math expressions on this set of data, this name represents that data and can serve as a variable in the mathematical expression. The valid characters are letters, numbers, and underscore. The first character must be a lowercase letter. |
| `label` | String | No | A human-readable label for this metric or expression. This is especially useful if this is an expression, so that you know what the value represents. If the metric or expression is shown in a CW dashboard widget, the label is shown. If ``Label`` is omitted, CW generates a default. |
| `metric_stat` | [Struct(MetricStat)](#metricsmetric_stat) | No | The metric to be returned, along with statistics, period, and units. Use this parameter only if this object is retrieving a metric and not performing a math expression on returned data. Within one MetricDataQuery object, you must specify either ``Expression`` or ``MetricStat`` but not both. |
| `period` | `Int(1..)` | No | The granularity, in seconds, of the returned data points. For metrics with regular resolution, a period can be as short as one minute (60 seconds) and must be a multiple of 60. For high-resolution metrics that are collected at intervals of less than one minute, the period can be 1, 5, 10, 20, 30, 60, or any multiple of 60. High-resolution metrics are those metrics stored by a ``PutMetricData`` operation that includes a ``StorageResolution of 1 second``. |
| `return_data` | Bool | No | This option indicates whether to return the timestamps and raw data values of this metric. When you create an alarm based on a metric math expression, specify ``True`` for this value for only the one math expression that the alarm is based on. You must specify ``False`` for ``ReturnData`` for all the other metrics and expressions used in the alarm. This field is required. |

### metrics.metric_stat

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `metric` | [Struct(Metric)](#metricsmetric_statmetric) | Yes | The metric to return, including the metric name, namespace, and dimensions. |
| `period` | `Int(1..)` | Yes | The granularity, in seconds, of the returned data points. For metrics with regular resolution, a period can be as short as one minute (60 seconds) and must be a multiple of 60. For high-resolution metrics that are collected at intervals of less than one minute, the period can be 1, 5, 10, 20, 30, 60, or any multiple of 60. High-resolution metrics are those metrics stored by a ``PutMetricData`` call that includes a ``StorageResolution`` of 1 second. If the ``StartTime`` parameter specifies a time stamp that is greater than 3 hours ago, you must specify the period as follows or no data points in that time range is returned: + Start time between 3 hours and 15 days ago - Use a multiple of 60 seconds (1 minute). + Start time between 15 and 63 days ago - Use a multiple of 300 seconds (5 minutes). + Start time greater than 63 days ago - Use a multiple of 3600 seconds (1 hour). |
| `stat` | `String(len: 1..)` | Yes | The statistic to return. It can include any CW statistic or extended statistic. For a list of valid values, see the table in [Statistics](https://docs.aws.amazon.com/AmazonCloudWatch/latest/monitoring/cloudwatch_concepts.html#Statistic) in the *User Guide*. |
| `unit` | [Enum (Unit)](#metricsmetric_statunit-unit) | No | The unit to use for the returned data points. Valid values are: Seconds, Microseconds, Milliseconds, Bytes, Kilobytes, Megabytes, Gigabytes, Terabytes, Bits, Kilobits, Megabits, Gigabits, Terabits, Percent, Count, Bytes/Second, Kilobytes/Second, Megabytes/Second, Gigabytes/Second, Terabytes/Second, Bits/Second, Kilobits/Second, Megabits/Second, Gigabits/Second, Terabits/Second, Count/Second, or None. |

### metrics.metric_stat.metric

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `dimensions` | List\<[Struct(Dimension)](#dimension)\> | No | The metric dimensions that you want to be used for the metric that the alarm will watch. |
| `metric_name` | `String(len: 1..=255)` | No | The name of the metric that you want the alarm to watch. This is a required field. |
| `namespace` | `String(pattern, len: 1..=255)` | No | The namespace of the metric that the alarm will watch. |

### warm_up_configuration

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `only_start_evaluating_after_warm_up_period_ends` | Bool | No | Specifies whether the alarm waits for the full warm-up period before it starts to evaluate. The default is ``false``. If ``true``, the alarm waits the entire ``WarmUpPeriodDurationInMinutes`` before it starts to evaluate, even if metric data arrives earlier. If ``false``, the alarm ends the warm-up period early. Evaluation begins as soon as the alarm has enough metric data to fill its evaluation window. |
| `warm_up_period_duration_in_minutes` | Int(1..=2880) | No | The length of the warm-up period, in minutes. After you create or update the alarm, the alarm stays in ``INSUFFICIENT_DATA`` for this duration. During this time, the alarm does not perform alarm actions. You can change this value at any time, including after the warm-up period ends. If you change it after the warm-up period ends, the new value does not restart the warm-up period. |

## Attribute Reference

### `arn`

- **Type:** aws.Arn — identity: String(pattern); member: String



## AWS API narrowings

These fields are narrower than or equal to CFN. Input rules combine documented AWS domains, recorded live observations, and Carina canonical spellings for stable state comparison. A canonical DSL spelling does not claim AWS rejects other spellings. Attaching a type name adds no structural constraints. Field names below use CFN spelling.

| CFN field | Relation to CFN | Evidence URL | Verbatim quote / exact observation | Checked / observed |
| --- | --- | --- | --- | --- |
| `/definitions/AlarmPromQLCriteria/properties/PendingPeriod` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/AmazonCloudWatch/latest/APIReference/API_AlarmPromQLCriteria.html#ACW-Type-AlarmPromQLCriteria-PendingPeriod) | ` The duration, in seconds, that a contributor must be continuously breaching before it transitions to the ALARM state. `<br>` Valid Range: Minimum value of 0. Maximum value of 86400. ` | 2026-11-07 |
| `/definitions/AlarmPromQLCriteria/properties/Query` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/AmazonCloudWatch/latest/APIReference/API_AlarmPromQLCriteria.html#ACW-Type-AlarmPromQLCriteria-Query) | ` The PromQL query that the alarm evaluates. The query must return a result of vector type. Each entry in the vector result represents an alarm contributor. `<br>` Length Constraints: Minimum length of 1. Maximum length of 10000. ` | 2026-11-07 |
| `/definitions/AlarmPromQLCriteria/properties/RecoveryPeriod` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/AmazonCloudWatch/latest/APIReference/API_AlarmPromQLCriteria.html#ACW-Type-AlarmPromQLCriteria-RecoveryPeriod) | ` The duration, in seconds, that a contributor must continuously not be breaching before it transitions back to the OK state. `<br>` Valid Range: Minimum value of 0. Maximum value of 86400. ` | 2026-11-07 |
| `/definitions/Dimension/properties/Name` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/AmazonCloudWatch/latest/APIReference/API_Dimension.html#ACW-Type-Dimension-Name) | ` The name of the dimension. Dimension names must contain only ASCII characters, must include at least one non-whitespace character, and cannot start with a colon (:). ASCII control characters are not supported as part of dimension names. `<br>` Length Constraints: Minimum length of 1. Maximum length of 255. ` | 2026-11-07 |
| `/definitions/Dimension/properties/Value` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/AmazonCloudWatch/latest/APIReference/API_Dimension.html#ACW-Type-Dimension-Value) | ` The value of the dimension. Dimension values must contain only ASCII characters and must include at least one non-whitespace character. ASCII control characters are not supported as part of dimension values. `<br>` Length Constraints: Minimum length of 1. Maximum length of 1024. ` | 2026-11-07 |
| `/definitions/Metric/properties/MetricName` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/AmazonCloudWatch/latest/APIReference/API_PutMetricAlarm.html#ACW-PutMetricAlarm-request-MetricName) | ` The name for the metric associated with the alarm. For each PutMetricAlarm operation, you must specify either MetricName, a Metrics array, or an EvaluationCriteria. `<br>` If you are creating an alarm based on a math expression, you cannot specify this parameter, or any of the Namespace, Dimensions, Period, Unit, Statistic, or ExtendedStatistic parameters. Instead, you specify all this information in the Metrics array. `<br>` Length Constraints: Minimum length of 1. Maximum length of 255. ` | 2026-11-07 |
| `/definitions/Metric/properties/Namespace` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/AmazonCloudWatch/latest/APIReference/API_PutMetricAlarm.html#ACW-PutMetricAlarm-request-Namespace) | ` The namespace for the metric associated specified in MetricName. `<br>` Length Constraints: Minimum length of 1. Maximum length of 255. `<br>` Pattern: [^:].* ` | 2026-11-07 |
| `/definitions/MetricDataQuery/properties/Expression` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/AmazonCloudWatch/latest/APIReference/API_MetricDataQuery.html#ACW-Type-MetricDataQuery-Expression) | ` This field can contain either a Metrics Insights query, or a metric math expression to be performed on the returned data. For more information about Metrics Insights queries, see Metrics Insights query components and syntax in the Amazon CloudWatch User Guide. `<br>` A math expression can use the Id of the other metrics or queries to refer to those metrics, and can also use the Id of other expressions to use the result of those expressions. For more information about metric math expressions, see Metric Math Syntax and Functions in the Amazon CloudWatch User Guide. `<br>` Within each MetricDataQuery object, you must specify either Expression or MetricStat but not both. `<br>` Length Constraints: Minimum length of 1. Maximum length of 2048. ` | 2026-11-07 |
| `/definitions/MetricDataQuery/properties/Id` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/AmazonCloudWatch/latest/APIReference/API_MetricDataQuery.html#ACW-Type-MetricDataQuery-Id) | ` A short name used to tie this object to the results in the response. This name must be unique within a single call to GetMetricData. If you are performing math expressions on this set of data, this name represents that data and can serve as a variable in the mathematical expression. The valid characters are letters, numbers, and underscore. The first character must be a lowercase letter. `<br>` Length Constraints: Minimum length of 1. Maximum length of 255. ` | 2026-11-07 |
| `/definitions/MetricDataQuery/properties/Period` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/AmazonCloudWatch/latest/APIReference/API_MetricDataQuery.html#ACW-Type-MetricDataQuery-Period) | ` The granularity, in seconds, of the returned data points. For metrics with regular resolution, a period can be as short as one minute (60 seconds) and must be a multiple of 60. For high-resolution metrics that are collected at intervals of less than one minute, the period can be 1, 5, 10, 20, 30, 60, or any multiple of 60. High-resolution metrics are those metrics stored by a PutMetricData operation that includes a StorageResolution of 1 second. `<br>` Valid Range: Minimum value of 1. ` | 2026-11-07 |
| `/definitions/MetricStat/properties/Period` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/AmazonCloudWatch/latest/APIReference/API_MetricStat.html#ACW-Type-MetricStat-Period) | ` The granularity, in seconds, of the returned data points. For metrics with regular resolution, a period can be as short as one minute (60 seconds) and must be a multiple of 60. For high-resolution metrics that are collected at intervals of less than one minute, the period can be 1, 5, 10, 20, 30, 60, or any multiple of 60. High-resolution metrics are those metrics stored by a PutMetricData call that includes a StorageResolution of 1 second. `<br>` If the StartTime parameter specifies a time stamp that is greater than 3 hours ago, you must specify the period as follows or no data points in that time range is returned: `<br>` Start time between 3 hours and 15 days ago - Use a multiple of 60 seconds (1 minute). `<br>` Start time between 15 and 63 days ago - Use a multiple of 300 seconds (5 minutes). `<br>` Start time greater than 63 days ago - Use a multiple of 3600 seconds (1 hour). `<br>` Valid Range: Minimum value of 1. ` | 2026-11-07 |
| `/definitions/MetricStat/properties/Stat` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/AmazonCloudWatch/latest/APIReference/API_MetricStat.html) | ` The statistic to return. It can include any CloudWatch statistic or extended statistic. ` | 2026-11-07 |
| `/definitions/MetricStat/properties/Stat` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/AmazonCloudWatch/latest/monitoring/Statistics-definitions.html) | ` SampleCount is the number of data points during the period. ` | 2026-11-07 |
| `/definitions/MetricStat/properties/Unit` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/AmazonCloudWatch/latest/APIReference/API_MetricStat.html#ACW-Type-MetricStat-Unit) | ` When you are using a Put operation, this defines what unit you want to use when storing the metric. `<br>` In a Get operation, if you omit Unit then all data that was collected with any unit is returned, along with the corresponding units that were specified when the data was reported to CloudWatch. If you specify a unit, the operation returns only data that was collected with that unit specified. If you specify a unit that does not match the data collected, the results of the operation are null. CloudWatch does not perform unit conversions. `<br>` Valid Values: Seconds \| Microseconds \| Milliseconds \| Bytes \| Kilobytes \| Megabytes \| Gigabytes \| Terabytes \| Bits \| Kilobits \| Megabits \| Gigabits \| Terabits \| Percent \| Count \| Bytes/Second \| Kilobytes/Second \| Megabytes/Second \| Gigabytes/Second \| Terabytes/Second \| Bits/Second \| Kilobits/Second \| Megabits/Second \| Gigabits/Second \| Terabits/Second \| Count/Second \| None ` | 2026-11-07 |
| `/definitions/WallClockWindow/properties/Timezone` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/AmazonCloudWatch/latest/APIReference/API_WallClockWindow.html#ACW-Type-WallClockWindow-Timezone) | ` The time zone to use when the alarm aligns the evaluation window to clock boundaries. You can specify an IANA time zone name (for example, America/New_York), a fixed UTC offset (for example, +05:30), or an offset-prefixed identifier (for example, UTC+05:30). The offset must be aligned to a multiple of 5 minutes. If you don't specify a time zone, CloudWatch uses UTC. `<br>` The time zone affects window alignment for all periods, including periods of one hour or shorter. `<br>` Length Constraints: Minimum length of 1. Maximum length of 50. ` | 2026-11-07 |
| `/properties/AlarmActions` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/AmazonCloudWatch/latest/APIReference/API_PutMetricAlarm.html#ACW-PutMetricAlarm-request-AlarmActions) | ` The actions to execute when this alarm transitions to the ALARM state from any other state. Each action is specified as an Amazon Resource Name (ARN). Valid values: `<br>` EC2 actions: `<br>` arn:aws:automate:region:ec2:stop `<br>` arn:aws:automate:region:ec2:terminate `<br>` arn:aws:automate:region:ec2:reboot `<br>` arn:aws:automate:region:ec2:recover `<br>` arn:aws:swf:region:account-id:action/actions/AWS_EC2.InstanceId.Stop/1.0 `<br>` arn:aws:swf:region:account-id:action/actions/AWS_EC2.InstanceId.Terminate/1.0 `<br>` arn:aws:swf:region:account-id:action/actions/AWS_EC2.InstanceId.Reboot/1.0 `<br>` arn:aws:swf:region:account-id:action/actions/AWS_EC2.InstanceId.Recover/1.0 `<br>` Autoscaling action: `<br>` arn:aws:autoscaling:region:account-id:scalingPolicy:policy-id:autoScalingGroupName/group-friendly-name:policyName/policy-friendly-name `<br>` Lambda actions: `<br>` Invoke the latest version of a Lambda function: arn:aws:lambda:region:account-id:function:function-name `<br>` Invoke a specific version of a Lambda function: arn:aws:lambda:region:account-id:function:function-name:version-number `<br>` Invoke a function by using an alias Lambda function: arn:aws:lambda:region:account-id:function:function-name:alias-name `<br>` SNS notification action: `<br>` arn:aws:sns:region:account-id:sns-topic-name `<br>` SSM integration actions: `<br>` arn:aws:ssm:region:account-id:opsitem:severity#CATEGORY=category-name `<br>` arn:aws:ssm-incidents::account-id:responseplan/response-plan-name `<br>` Start a Amazon Q Developer operational investigation `<br>` arn:aws:aiops:region:account-id:investigation-group:investigation-group-id `<br>` Array Members: Maximum number of 5 items. `<br>` Length Constraints: Minimum length of 1. Maximum length of 1024. ` | 2026-11-07 |
| `/properties/AlarmDescription` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/AmazonCloudWatch/latest/APIReference/API_PutMetricAlarm.html#ACW-PutMetricAlarm-request-AlarmDescription) | ` The description for the alarm. `<br>` Length Constraints: Minimum length of 0. Maximum length of 1024. ` | 2026-11-07 |
| `/properties/AlarmName` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/AmazonCloudWatch/latest/APIReference/API_PutMetricAlarm.html#ACW-PutMetricAlarm-request-AlarmName) | ` The name for the alarm. This name must be unique within the Region. `<br>` The name must contain only UTF-8 characters, and can't contain ASCII control characters `<br>` Length Constraints: Minimum length of 1. Maximum length of 255. ` | 2026-11-07 |
| `/properties/ComparisonOperator` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/AmazonCloudWatch/latest/APIReference/API_PutMetricAlarm.html#ACW-PutMetricAlarm-request-ComparisonOperator) | ` The arithmetic operation to use when comparing the specified statistic and threshold. The specified statistic value is used as the first operand. `<br>` The values LessThanLowerOrGreaterThanUpperThreshold, LessThanLowerThreshold, and GreaterThanUpperThreshold are used only for alarms based on anomaly detection models. `<br>` Valid Values: GreaterThanOrEqualToThreshold \| GreaterThanThreshold \| LessThanThreshold \| LessThanOrEqualToThreshold \| LessThanLowerOrGreaterThanUpperThreshold \| LessThanLowerThreshold \| GreaterThanUpperThreshold ` | 2026-11-07 |
| `/properties/DatapointsToAlarm` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/AmazonCloudWatch/latest/APIReference/API_PutMetricAlarm.html#ACW-PutMetricAlarm-request-DatapointsToAlarm) | ` The number of data points that must be breaching to trigger the alarm. This is used only if you are setting an "M out of N" alarm. In that case, this value is the M. For more information, see Evaluating an Alarm in the Amazon CloudWatch User Guide. `<br>` Valid Range: Minimum value of 1. ` | 2026-11-07 |
| `/properties/EvaluateLowSampleCountPercentile` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/AmazonCloudWatch/latest/APIReference/API_PutMetricAlarm.html#ACW-PutMetricAlarm-request-EvaluateLowSampleCountPercentile) | ` Used only for alarms based on percentiles. If you specify ignore, the alarm state does not change during periods with too few data points to be statistically significant. If you specify evaluate or omit this parameter, the alarm is always evaluated and possibly changes state no matter how many data points are available. For more information, see Percentile-Based CloudWatch Alarms and Low Data Samples. `<br>` Valid Values: evaluate \| ignore `<br>` Length Constraints: Minimum length of 1. Maximum length of 255. ` | 2026-11-07 |
| `/properties/EvaluationInterval` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/AmazonCloudWatch/latest/APIReference/API_PutMetricAlarm.html#ACW-PutMetricAlarm-request-EvaluationInterval) | ` The frequency, in seconds, at which the alarm is evaluated. Valid values are 10, 20, 30, and any multiple of 60. `<br>` This parameter is required for alarms that use EvaluationCriteria, and cannot be specified for alarms configured with MetricName or Metrics. `<br>` Valid Range: Minimum value of 10. Maximum value of 3600. ` | 2026-11-07 |
| `/properties/EvaluationPeriods` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/AmazonCloudWatch/latest/APIReference/API_PutMetricAlarm.html#ACW-PutMetricAlarm-request-EvaluationPeriods) | ` The number of periods over which data is compared to the specified threshold. If you are setting an alarm that requires that a number of consecutive data points be breaching to trigger the alarm, this value specifies that number. If you are setting an "M out of N" alarm, this value is the N. `<br>` Valid Range: Minimum value of 1. ` | 2026-11-07 |
| `/properties/ExtendedStatistic` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/AmazonCloudWatch/latest/APIReference/API_PutMetricAlarm.html#ACW-PutMetricAlarm-request-ExtendedStatistic) | ` The extended statistic for the metric specified in MetricName. When you call PutMetricAlarm and specify a MetricName, you must specify either Statistic or ExtendedStatistic but not both. `<br>` If you specify ExtendedStatistic, the following are valid values: `<br>` p90 `<br>` tm90 `<br>` tc90 `<br>` ts90 `<br>` wm90 `<br>` IQM `<br>` PR(n:m) where n and m are values of the metric `<br>` TC(X%:X%) where X is between 10 and 90 inclusive. `<br>` TM(X%:X%) where X is between 10 and 90 inclusive. `<br>` TS(X%:X%) where X is between 10 and 90 inclusive. `<br>` WM(X%:X%) where X is between 10 and 90 inclusive. `<br>` For more information about these extended statistics, see CloudWatch statistics definitions. ` | 2026-11-07 |
| `/properties/InsufficientDataActions` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/AmazonCloudWatch/latest/APIReference/API_PutMetricAlarm.html#ACW-PutMetricAlarm-request-InsufficientDataActions) | ` The actions to execute when this alarm transitions to the INSUFFICIENT_DATA state from any other state. Each action is specified as an Amazon Resource Name (ARN). Valid values: `<br>` EC2 actions: `<br>` arn:aws:automate:region:ec2:stop `<br>` arn:aws:automate:region:ec2:terminate `<br>` arn:aws:automate:region:ec2:reboot `<br>` arn:aws:automate:region:ec2:recover `<br>` arn:aws:swf:region:account-id:action/actions/AWS_EC2.InstanceId.Stop/1.0 `<br>` arn:aws:swf:region:account-id:action/actions/AWS_EC2.InstanceId.Terminate/1.0 `<br>` arn:aws:swf:region:account-id:action/actions/AWS_EC2.InstanceId.Reboot/1.0 `<br>` arn:aws:swf:region:account-id:action/actions/AWS_EC2.InstanceId.Recover/1.0 `<br>` Autoscaling action: `<br>` arn:aws:autoscaling:region:account-id:scalingPolicy:policy-id:autoScalingGroupName/group-friendly-name:policyName/policy-friendly-name `<br>` Lambda actions: `<br>` Invoke the latest version of a Lambda function: arn:aws:lambda:region:account-id:function:function-name `<br>` Invoke a specific version of a Lambda function: arn:aws:lambda:region:account-id:function:function-name:version-number `<br>` Invoke a function by using an alias Lambda function: arn:aws:lambda:region:account-id:function:function-name:alias-name `<br>` SNS notification action: `<br>` arn:aws:sns:region:account-id:sns-topic-name `<br>` SSM integration actions: `<br>` arn:aws:ssm:region:account-id:opsitem:severity#CATEGORY=category-name `<br>` arn:aws:ssm-incidents::account-id:responseplan/response-plan-name `<br>` Array Members: Maximum number of 5 items. `<br>` Length Constraints: Minimum length of 1. Maximum length of 1024. ` | 2026-11-07 |
| `/properties/MetricName` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/AmazonCloudWatch/latest/APIReference/API_PutMetricAlarm.html#ACW-PutMetricAlarm-request-MetricName) | ` The name for the metric associated with the alarm. For each PutMetricAlarm operation, you must specify either MetricName, a Metrics array, or an EvaluationCriteria. `<br>` If you are creating an alarm based on a math expression, you cannot specify this parameter, or any of the Namespace, Dimensions, Period, Unit, Statistic, or ExtendedStatistic parameters. Instead, you specify all this information in the Metrics array. `<br>` Length Constraints: Minimum length of 1. Maximum length of 255. ` | 2026-11-07 |
| `/properties/Namespace` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/AmazonCloudWatch/latest/APIReference/API_PutMetricAlarm.html#ACW-PutMetricAlarm-request-Namespace) | ` The namespace for the metric associated specified in MetricName. `<br>` Length Constraints: Minimum length of 1. Maximum length of 255. `<br>` Pattern: [^:].* ` | 2026-11-07 |
| `/properties/OKActions` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/AmazonCloudWatch/latest/APIReference/API_PutMetricAlarm.html#ACW-PutMetricAlarm-request-OKActions) | ` The actions to execute when this alarm transitions to an OK state from any other state. Each action is specified as an Amazon Resource Name (ARN). Valid values: `<br>` EC2 actions: `<br>` arn:aws:automate:region:ec2:stop `<br>` arn:aws:automate:region:ec2:terminate `<br>` arn:aws:automate:region:ec2:reboot `<br>` arn:aws:automate:region:ec2:recover `<br>` arn:aws:swf:region:account-id:action/actions/AWS_EC2.InstanceId.Stop/1.0 `<br>` arn:aws:swf:region:account-id:action/actions/AWS_EC2.InstanceId.Terminate/1.0 `<br>` arn:aws:swf:region:account-id:action/actions/AWS_EC2.InstanceId.Reboot/1.0 `<br>` arn:aws:swf:region:account-id:action/actions/AWS_EC2.InstanceId.Recover/1.0 `<br>` Autoscaling action: `<br>` arn:aws:autoscaling:region:account-id:scalingPolicy:policy-id:autoScalingGroupName/group-friendly-name:policyName/policy-friendly-name `<br>` Lambda actions: `<br>` Invoke the latest version of a Lambda function: arn:aws:lambda:region:account-id:function:function-name `<br>` Invoke a specific version of a Lambda function: arn:aws:lambda:region:account-id:function:function-name:version-number `<br>` Invoke a function by using an alias Lambda function: arn:aws:lambda:region:account-id:function:function-name:alias-name `<br>` SNS notification action: `<br>` arn:aws:sns:region:account-id:sns-topic-name `<br>` SSM integration actions: `<br>` arn:aws:ssm:region:account-id:opsitem:severity#CATEGORY=category-name `<br>` arn:aws:ssm-incidents::account-id:responseplan/response-plan-name `<br>` Array Members: Maximum number of 5 items. `<br>` Length Constraints: Minimum length of 1. Maximum length of 1024. ` | 2026-11-07 |
| `/properties/Period` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/AmazonCloudWatch/latest/APIReference/API_PutMetricAlarm.html#ACW-PutMetricAlarm-request-Period) | ` The length, in seconds, used each time the metric specified in MetricName is evaluated. Valid values are 10, 20, 30, and any multiple of 60. `<br>` Period is required for alarms based on static thresholds. If you are creating an alarm based on a metric math expression, you specify the period for each metric within the objects in the Metrics array. `<br>` Be sure to specify 10, 20, or 30 only for metrics that are stored by a PutMetricData call with a StorageResolution of 1. If you specify a period of 10, 20, or 30 for a metric that does not have sub-minute resolution, the alarm still attempts to gather data at the period rate that you specify. In this case, it does not receive data for the attempts that do not correspond to a one-minute data resolution, and the alarm might often lapse into INSUFFICENT_DATA status. Specifying 10, 20, or 30 also sets this alarm as a high-resolution alarm, which has a higher charge than other alarms. For more information about pricing, see Amazon CloudWatch Pricing. `<br>` An alarm's total current evaluation period can be no longer than seven days, so Period multiplied by EvaluationPeriods can't be more than 604,800 seconds. For alarms with a period of less than one hour (3,600 seconds), the total evaluation period can't be longer than one day (86,400 seconds). `<br>` Valid Range: Minimum value of 10. ` | 2026-11-07 |
| `/properties/Statistic` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/AmazonCloudWatch/latest/APIReference/API_PutMetricAlarm.html#ACW-PutMetricAlarm-request-Statistic) | ` The statistic for the metric specified in MetricName, other than percentile. For percentile statistics, use ExtendedStatistic. When you call PutMetricAlarm and specify a MetricName, you must specify either Statistic or ExtendedStatistic, but not both. `<br>` Valid Values: SampleCount \| Average \| Sum \| Minimum \| Maximum ` | 2026-11-07 |
| `/properties/ThresholdMetricId` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/AmazonCloudWatch/latest/APIReference/API_PutMetricAlarm.html#ACW-PutMetricAlarm-request-ThresholdMetricId) | ` If this is an alarm based on an anomaly detection model, make this value match the ID of the ANOMALY_DETECTION_BAND function. `<br>` For an example of how to use this parameter, see the Anomaly Detection Model Alarm example on this page. `<br>` If your alarm uses this parameter, it cannot have Auto Scaling actions. `<br>` Length Constraints: Minimum length of 1. Maximum length of 255. ` | 2026-11-07 |
| `/properties/Unit` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/AmazonCloudWatch/latest/APIReference/API_PutMetricAlarm.html#ACW-PutMetricAlarm-request-Unit) | ` The unit of measure for the statistic. For example, the units for the Amazon EC2 NetworkIn metric are Bytes because NetworkIn tracks the number of bytes that an instance receives on all network interfaces. You can also specify a unit when you create a custom metric. Units help provide conceptual meaning to your data. Metric data points that specify a unit of measure, such as Percent, are aggregated separately. If you are creating an alarm based on a metric math expression, you can specify the unit for each metric (if needed) within the objects in the Metrics array. `<br>` If you don't specify Unit, CloudWatch retrieves all unit types that have been published for the metric and attempts to evaluate the alarm. Usually, metrics are published with only one unit, so the alarm works as intended. `<br>` However, if the metric is published with multiple types of units and you don't specify a unit, the alarm's behavior is not defined and it behaves unpredictably. `<br>` We recommend omitting Unit so that you don't inadvertently specify an incorrect unit that is not published for this metric. Doing so causes the alarm to be stuck in the INSUFFICIENT DATA state. `<br>` Valid Values: Seconds \| Microseconds \| Milliseconds \| Bytes \| Kilobytes \| Megabytes \| Gigabytes \| Terabytes \| Bits \| Kilobits \| Megabits \| Gigabits \| Terabits \| Percent \| Count \| Bytes/Second \| Kilobytes/Second \| Megabytes/Second \| Gigabytes/Second \| Terabytes/Second \| Bits/Second \| Kilobits/Second \| Megabits/Second \| Gigabits/Second \| Terabits/Second \| Count/Second \| None ` | 2026-11-07 |

