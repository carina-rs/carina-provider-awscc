---
title: "awscc.budgets.Budget"
---


CloudFormation Type: `AWS::Budgets::Budget`

Resource Type definition for AWS::Budgets::Budget

## Example

```crn
awscc.budgets.Budget {
  budget = {
    budget_limit = {
      amount = 100.0
      unit   = 'USD'
    }

    budget_name  = 'carina-registry-monthly-cost'
    budget_type  = cost
    time_unit    = monthly
  }

  notifications_with_subscribers = [{
    notification = {
      comparison_operator = greater_than
      notification_type   = actual
      threshold           = 80.0
      threshold_type      = percentage
    }

    subscribers  = [{
      address           = 'finance@example.com'
      subscription_type = email
    }]
  }]

  resource_tags = [{
    key   = 'Project'
    value = 'carina-registry'
  }]
}
```

## Argument Reference

### `budget`

- **Type:** [Struct(BudgetData)](#budget-1)
- **Required:** Yes

### `notifications_with_subscribers`

- **Type:** List\<[Struct(NotificationWithSubscribers)](#notifications_with_subscribers-1)\>
- **Required:** No
- **Create-only:** Yes

### `resource_tags`

- **Type:** List\<[Struct(ResourceTag)](#resource_tags-1)\>
- **Required:** No

## Enum Values

### budget.auto_adjust_data.auto_adjust_type (AutoAdjustType)

| Value | DSL Identifier |
|-------|----------------|
| `HISTORICAL` | `aws.budgets.Budget.BudgetData.AutoAdjustData.AutoAdjustType.historical` |
| `FORECAST` | `aws.budgets.Budget.BudgetData.AutoAdjustData.AutoAdjustType.forecast` |

Shorthand formats: `historical` or `AutoAdjustType.historical`

### budget.budget_type (BudgetType)

| Value | DSL Identifier |
|-------|----------------|
| `USAGE` | `aws.budgets.Budget.BudgetData.BudgetType.usage` |
| `COST` | `aws.budgets.Budget.BudgetData.BudgetType.cost` |
| `RI_UTILIZATION` | `aws.budgets.Budget.BudgetData.BudgetType.ri_utilization` |
| `RI_COVERAGE` | `aws.budgets.Budget.BudgetData.BudgetType.ri_coverage` |
| `SAVINGS_PLANS_UTILIZATION` | `aws.budgets.Budget.BudgetData.BudgetType.savings_plans_utilization` |
| `SAVINGS_PLANS_COVERAGE` | `aws.budgets.Budget.BudgetData.BudgetType.savings_plans_coverage` |

Shorthand formats: `usage` or `BudgetType.usage`

### budget.metrics (Metrics)

| Value | DSL Identifier |
|-------|----------------|
| `BlendedCost` | `aws.budgets.Budget.BudgetData.Metrics.blended_cost` |
| `UnblendedCost` | `aws.budgets.Budget.BudgetData.Metrics.unblended_cost` |
| `AmortizedCost` | `aws.budgets.Budget.BudgetData.Metrics.amortized_cost` |
| `NetUnblendedCost` | `aws.budgets.Budget.BudgetData.Metrics.net_unblended_cost` |
| `NetAmortizedCost` | `aws.budgets.Budget.BudgetData.Metrics.net_amortized_cost` |
| `UsageQuantity` | `aws.budgets.Budget.BudgetData.Metrics.usage_quantity` |
| `NormalizedUsageAmount` | `aws.budgets.Budget.BudgetData.Metrics.normalized_usage_amount` |
| `Hours` | `aws.budgets.Budget.BudgetData.Metrics.hours` |

Shorthand formats: `blended_cost` or `Metrics.blended_cost`

### budget.time_unit (TimeUnit)

| Value | DSL Identifier |
|-------|----------------|
| `DAILY` | `aws.budgets.Budget.BudgetData.TimeUnit.daily` |
| `MONTHLY` | `aws.budgets.Budget.BudgetData.TimeUnit.monthly` |
| `QUARTERLY` | `aws.budgets.Budget.BudgetData.TimeUnit.quarterly` |
| `ANNUALLY` | `aws.budgets.Budget.BudgetData.TimeUnit.annually` |
| `CUSTOM` | `aws.budgets.Budget.BudgetData.TimeUnit.custom` |

Shorthand formats: `daily` or `TimeUnit.daily`

### expression.cost_categories.match_options (MatchOptions)

| Value | DSL Identifier |
|-------|----------------|
| `EQUALS` | `aws.budgets.Budget.BudgetData.Expression.CostCategoryValues.MatchOptions.equals` |
| `ABSENT` | `aws.budgets.Budget.BudgetData.Expression.CostCategoryValues.MatchOptions.absent` |
| `STARTS_WITH` | `aws.budgets.Budget.BudgetData.Expression.CostCategoryValues.MatchOptions.starts_with` |
| `ENDS_WITH` | `aws.budgets.Budget.BudgetData.Expression.CostCategoryValues.MatchOptions.ends_with` |
| `CONTAINS` | `aws.budgets.Budget.BudgetData.Expression.CostCategoryValues.MatchOptions.contains` |
| `GREATER_THAN_OR_EQUAL` | `aws.budgets.Budget.BudgetData.Expression.CostCategoryValues.MatchOptions.greater_than_or_equal` |
| `CASE_SENSITIVE` | `aws.budgets.Budget.BudgetData.Expression.CostCategoryValues.MatchOptions.case_sensitive` |
| `CASE_INSENSITIVE` | `aws.budgets.Budget.BudgetData.Expression.CostCategoryValues.MatchOptions.case_insensitive` |

Shorthand formats: `equals` or `MatchOptions.equals`

### expression.dimensions.key (Key)

| Value | DSL Identifier |
|-------|----------------|
| `AZ` | `aws.budgets.Budget.BudgetData.Expression.ExpressionDimensionValues.Key.az` |
| `INSTANCE_TYPE` | `aws.budgets.Budget.BudgetData.Expression.ExpressionDimensionValues.Key.instance_type` |
| `LINKED_ACCOUNT` | `aws.budgets.Budget.BudgetData.Expression.ExpressionDimensionValues.Key.linked_account` |
| `LINKED_ACCOUNT_NAME` | `aws.budgets.Budget.BudgetData.Expression.ExpressionDimensionValues.Key.linked_account_name` |
| `OPERATION` | `aws.budgets.Budget.BudgetData.Expression.ExpressionDimensionValues.Key.operation` |
| `PURCHASE_TYPE` | `aws.budgets.Budget.BudgetData.Expression.ExpressionDimensionValues.Key.purchase_type` |
| `REGION` | `aws.budgets.Budget.BudgetData.Expression.ExpressionDimensionValues.Key.region` |
| `SERVICE` | `aws.budgets.Budget.BudgetData.Expression.ExpressionDimensionValues.Key.service` |
| `SERVICE_CODE` | `aws.budgets.Budget.BudgetData.Expression.ExpressionDimensionValues.Key.service_code` |
| `USAGE_TYPE` | `aws.budgets.Budget.BudgetData.Expression.ExpressionDimensionValues.Key.usage_type` |
| `USAGE_TYPE_GROUP` | `aws.budgets.Budget.BudgetData.Expression.ExpressionDimensionValues.Key.usage_type_group` |
| `RECORD_TYPE` | `aws.budgets.Budget.BudgetData.Expression.ExpressionDimensionValues.Key.record_type` |
| `OPERATING_SYSTEM` | `aws.budgets.Budget.BudgetData.Expression.ExpressionDimensionValues.Key.operating_system` |
| `TENANCY` | `aws.budgets.Budget.BudgetData.Expression.ExpressionDimensionValues.Key.tenancy` |
| `SCOPE` | `aws.budgets.Budget.BudgetData.Expression.ExpressionDimensionValues.Key.scope` |
| `PLATFORM` | `aws.budgets.Budget.BudgetData.Expression.ExpressionDimensionValues.Key.platform` |
| `SUBSCRIPTION_ID` | `aws.budgets.Budget.BudgetData.Expression.ExpressionDimensionValues.Key.subscription_id` |
| `LEGAL_ENTITY_NAME` | `aws.budgets.Budget.BudgetData.Expression.ExpressionDimensionValues.Key.legal_entity_name` |
| `INVOICING_ENTITY` | `aws.budgets.Budget.BudgetData.Expression.ExpressionDimensionValues.Key.invoicing_entity` |
| `DEPLOYMENT_OPTION` | `aws.budgets.Budget.BudgetData.Expression.ExpressionDimensionValues.Key.deployment_option` |
| `DATABASE_ENGINE` | `aws.budgets.Budget.BudgetData.Expression.ExpressionDimensionValues.Key.database_engine` |
| `CACHE_ENGINE` | `aws.budgets.Budget.BudgetData.Expression.ExpressionDimensionValues.Key.cache_engine` |
| `INSTANCE_TYPE_FAMILY` | `aws.budgets.Budget.BudgetData.Expression.ExpressionDimensionValues.Key.instance_type_family` |
| `BILLING_ENTITY` | `aws.budgets.Budget.BudgetData.Expression.ExpressionDimensionValues.Key.billing_entity` |
| `RESERVATION_ID` | `aws.budgets.Budget.BudgetData.Expression.ExpressionDimensionValues.Key.reservation_id` |
| `RESOURCE_ID` | `aws.budgets.Budget.BudgetData.Expression.ExpressionDimensionValues.Key.resource_id` |
| `RIGHTSIZING_TYPE` | `aws.budgets.Budget.BudgetData.Expression.ExpressionDimensionValues.Key.rightsizing_type` |
| `SAVINGS_PLANS_TYPE` | `aws.budgets.Budget.BudgetData.Expression.ExpressionDimensionValues.Key.savings_plans_type` |
| `SAVINGS_PLAN_ARN` | `aws.budgets.Budget.BudgetData.Expression.ExpressionDimensionValues.Key.savings_plan_arn` |
| `PAYMENT_OPTION` | `aws.budgets.Budget.BudgetData.Expression.ExpressionDimensionValues.Key.payment_option` |
| `RESERVATION_MODIFIED` | `aws.budgets.Budget.BudgetData.Expression.ExpressionDimensionValues.Key.reservation_modified` |
| `TAG_KEY` | `aws.budgets.Budget.BudgetData.Expression.ExpressionDimensionValues.Key.tag_key` |
| `COST_CATEGORY_NAME` | `aws.budgets.Budget.BudgetData.Expression.ExpressionDimensionValues.Key.cost_category_name` |

Shorthand formats: `az` or `Key.az`

### expression.dimensions.match_options (MatchOptions)

| Value | DSL Identifier |
|-------|----------------|
| `EQUALS` | `aws.budgets.Budget.BudgetData.Expression.ExpressionDimensionValues.MatchOptions.equals` |
| `ABSENT` | `aws.budgets.Budget.BudgetData.Expression.ExpressionDimensionValues.MatchOptions.absent` |
| `STARTS_WITH` | `aws.budgets.Budget.BudgetData.Expression.ExpressionDimensionValues.MatchOptions.starts_with` |
| `ENDS_WITH` | `aws.budgets.Budget.BudgetData.Expression.ExpressionDimensionValues.MatchOptions.ends_with` |
| `CONTAINS` | `aws.budgets.Budget.BudgetData.Expression.ExpressionDimensionValues.MatchOptions.contains` |
| `GREATER_THAN_OR_EQUAL` | `aws.budgets.Budget.BudgetData.Expression.ExpressionDimensionValues.MatchOptions.greater_than_or_equal` |
| `CASE_SENSITIVE` | `aws.budgets.Budget.BudgetData.Expression.ExpressionDimensionValues.MatchOptions.case_sensitive` |
| `CASE_INSENSITIVE` | `aws.budgets.Budget.BudgetData.Expression.ExpressionDimensionValues.MatchOptions.case_insensitive` |

Shorthand formats: `equals` or `MatchOptions.equals`

### expression.tags.match_options (MatchOptions)

| Value | DSL Identifier |
|-------|----------------|
| `EQUALS` | `aws.budgets.Budget.BudgetData.Expression.TagValues.MatchOptions.equals` |
| `ABSENT` | `aws.budgets.Budget.BudgetData.Expression.TagValues.MatchOptions.absent` |
| `STARTS_WITH` | `aws.budgets.Budget.BudgetData.Expression.TagValues.MatchOptions.starts_with` |
| `ENDS_WITH` | `aws.budgets.Budget.BudgetData.Expression.TagValues.MatchOptions.ends_with` |
| `CONTAINS` | `aws.budgets.Budget.BudgetData.Expression.TagValues.MatchOptions.contains` |
| `GREATER_THAN_OR_EQUAL` | `aws.budgets.Budget.BudgetData.Expression.TagValues.MatchOptions.greater_than_or_equal` |
| `CASE_SENSITIVE` | `aws.budgets.Budget.BudgetData.Expression.TagValues.MatchOptions.case_sensitive` |
| `CASE_INSENSITIVE` | `aws.budgets.Budget.BudgetData.Expression.TagValues.MatchOptions.case_insensitive` |

Shorthand formats: `equals` or `MatchOptions.equals`

### notifications_with_subscribers.notification.comparison_operator (ComparisonOperator)

| Value | DSL Identifier |
|-------|----------------|
| `GREATER_THAN` | `aws.budgets.Budget.NotificationWithSubscribers.Notification.ComparisonOperator.greater_than` |
| `LESS_THAN` | `aws.budgets.Budget.NotificationWithSubscribers.Notification.ComparisonOperator.less_than` |
| `EQUAL_TO` | `aws.budgets.Budget.NotificationWithSubscribers.Notification.ComparisonOperator.equal_to` |

Shorthand formats: `greater_than` or `ComparisonOperator.greater_than`

### notifications_with_subscribers.notification.notification_type (NotificationType)

| Value | DSL Identifier |
|-------|----------------|
| `ACTUAL` | `aws.budgets.Budget.NotificationWithSubscribers.Notification.NotificationType.actual` |
| `FORECASTED` | `aws.budgets.Budget.NotificationWithSubscribers.Notification.NotificationType.forecasted` |

Shorthand formats: `actual` or `NotificationType.actual`

### notifications_with_subscribers.notification.threshold_type (ThresholdType)

| Value | DSL Identifier |
|-------|----------------|
| `PERCENTAGE` | `aws.budgets.Budget.NotificationWithSubscribers.Notification.ThresholdType.percentage` |
| `ABSOLUTE_VALUE` | `aws.budgets.Budget.NotificationWithSubscribers.Notification.ThresholdType.absolute_value` |

Shorthand formats: `percentage` or `ThresholdType.percentage`

### notifications_with_subscribers.subscribers.subscription_type (SubscriptionType)

| Value | DSL Identifier |
|-------|----------------|
| `SNS` | `aws.budgets.Budget.NotificationWithSubscribers.Subscriber.SubscriptionType.sns` |
| `EMAIL` | `aws.budgets.Budget.NotificationWithSubscribers.Subscriber.SubscriptionType.email` |

Shorthand formats: `sns` or `SubscriptionType.sns`

## Struct Definitions

### budget

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `auto_adjust_data` | [Struct(AutoAdjustData)](#budgetauto_adjust_data) | No |  |
| `billing_view_arn` | Arn | No |  |
| `budget_limit` | [Struct(Spend)](#budgetbudget_limit) | No |  |
| `budget_name` | `String (length: 1..=100, pattern: ^[^:\\]+$)` | No | Create-only. Anonymous identifiers hash only declared identity values. |
| `budget_type` | [Enum (BudgetType)](#budgetbudget_type-budgettype) | Yes |  |
| `cost_filters` | `Map<String, List<String>>` | No |  |
| `cost_types` | [Struct(CostTypes)](#budgetcost_types) | No |  |
| `filter_expression` | [Struct(Expression)](#expression) | No |  |
| `metrics` | List\<[Enum (Metrics)](#budgetmetrics-metrics)\> (items: ..=1) | No |  |
| `planned_budget_limits` | `Map<String, Spend>` | No |  |
| `time_period` | [Struct(TimePeriod)](#budgettime_period) | No |  |
| `time_unit` | [Enum (TimeUnit)](#budgettime_unit-timeunit) | Yes |  |

### budget.auto_adjust_data

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `auto_adjust_type` | [Enum (AutoAdjustType)](#budgetauto_adjust_dataauto_adjust_type-autoadjusttype) | Yes |  |
| `historical_options` | [Struct(HistoricalOptions)](#budgetauto_adjust_datahistorical_options) | No |  |

### budget.auto_adjust_data.historical_options

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `budget_adjustment_period` | `Int(1..=60)` | Yes |  |

### budget.budget_limit

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `amount` | `Float(0..)` | Yes |  |
| `unit` | String | Yes |  |

### budget.cost_types

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `include_credit` | Bool | No |  |
| `include_discount` | Bool | No |  |
| `include_other_subscription` | Bool | No |  |
| `include_recurring` | Bool | No |  |
| `include_refund` | Bool | No |  |
| `include_subscription` | Bool | No |  |
| `include_support` | Bool | No |  |
| `include_tax` | Bool | No |  |
| `include_upfront` | Bool | No |  |
| `use_amortized` | Bool | No |  |
| `use_blended` | Bool | No |  |

### budget.time_period

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `end` | String | No |  |
| `start` | String | No |  |

### expression

Used by: `budget.filter_expression`, `budget.filter_expression.and`, `budget.filter_expression.not`, `budget.filter_expression.or`

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `and` | List\<[Struct(Expression)](#expression)\> | No |  |
| `cost_categories` | [Struct(CostCategoryValues)](#expressioncost_categories) | No |  |
| `dimensions` | [Struct(ExpressionDimensionValues)](#expressiondimensions) | No |  |
| `not` | [Struct(Expression)](#expression) | No |  |
| `or` | List\<[Struct(Expression)](#expression)\> | No |  |
| `tags` | [Struct(TagValues)](#expressiontags) | No |  |

### expression.cost_categories

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `key` | String | No |  |
| `match_options` | List\<[Enum (MatchOptions)](#expressioncost_categoriesmatch_options-matchoptions)\> | No |  |
| `values` | `List<String (length: 0..=1024)> (items: 1..)` | No |  |

### expression.dimensions

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `key` | [Enum (Key)](#expressiondimensionskey-key) | No |  |
| `match_options` | List\<[Enum (MatchOptions)](#expressiondimensionsmatch_options-matchoptions)\> | No |  |
| `values` | `List<String (length: 0..=1024)> (items: 1..)` | No |  |

### expression.tags

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `key` | `String (length: 0..=1024)` | No |  |
| `match_options` | List\<[Enum (MatchOptions)](#expressiontagsmatch_options-matchoptions)\> | No |  |
| `values` | `List<String (length: 0..=1024)> (items: 1..)` | No |  |

### notifications_with_subscribers

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `notification` | [Struct(Notification)](#notifications_with_subscribersnotification) | Yes |  |
| `subscribers` | List\<[Struct(Subscriber)](#notifications_with_subscriberssubscribers)\> | Yes |  |

### notifications_with_subscribers.notification

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `comparison_operator` | [Enum (ComparisonOperator)](#notifications_with_subscribersnotificationcomparison_operator-comparisonoperator) | Yes |  |
| `notification_type` | [Enum (NotificationType)](#notifications_with_subscribersnotificationnotification_type-notificationtype) | Yes |  |
| `threshold` | `Float(0..=15000000000000)` | Yes |  |
| `threshold_type` | [Enum (ThresholdType)](#notifications_with_subscribersnotificationthreshold_type-thresholdtype) | No |  |

### notifications_with_subscribers.subscribers

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `address` | String | Yes |  |
| `subscription_type` | [Enum (SubscriptionType)](#notifications_with_subscriberssubscriberssubscription_type-subscriptiontype) | Yes |  |

### resource_tags

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `key` | String | Yes |  |
| `value` | String | No |  |

## Attribute Reference

### `id`

- **Type:** String

