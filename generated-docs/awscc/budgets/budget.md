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

  tags = {
    Project = 'carina-registry'
  }
}
```

## Argument Reference

### `budget`

- **Type:** [Struct(BudgetData)](#budget-1)
- **Required:** Yes

### `notifications_with_subscribers`

- **Type:** List\<[Struct(NotificationWithSubscribers)](#notifications_with_subscribers-1)\> (ordered)
- **Required:** No
- **Create-only:** Yes

### `tags`

- **Type:** Map\<`String`, `String`\>
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
| `billing_view_arn` | `aws.Arn (pattern: /^arn:(?:aws\|aws-cn\|aws-us-gov\|aws-iso\|aws-iso-b\|aws-iso-e\|aws-iso-f\|aws-eusc):[^:]+:.*$/ (Full), custom validator)` | No |  |
| `budget_limit` | [Struct(Spend)](#spend) | No |  |
| `budget_name` | `String (pattern: /^[^:\\]+$/ (Full), len: 1..=100)` | No | Create-only. Anonymous identifiers hash only declared identity values. |
| `budget_type` | [BudgetType](#budgetbudget_type-budgettype) | Yes |  |
| `cost_filters` | Map\<`String`, List\<`String`\> (ordered)\> | No |  |
| `cost_types` | [Struct(CostTypes)](#budgetcost_types) | No |  |
| `filter_expression` | [Struct(Expression)](#expression) | No |  |
| `metrics` | List\<[Metrics](#budgetmetrics-metrics)\> (items: unbounded..=1, ordered) | No |  |
| `planned_budget_limits` | Map\<`String`, [Struct(Spend)](#spend)\> | No |  |
| `time_period` | [Struct(TimePeriod)](#budgettime_period) | No |  |
| `time_unit` | [TimeUnit](#budgettime_unit-timeunit) | Yes |  |

### budget.auto_adjust_data

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `auto_adjust_type` | [AutoAdjustType](#budgetauto_adjust_dataauto_adjust_type-autoadjusttype) | Yes |  |
| `historical_options` | [Struct(HistoricalOptions)](#budgetauto_adjust_datahistorical_options) | No |  |

### budget.auto_adjust_data.historical_options

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `budget_adjustment_period` | `Int (range: 1..=60)` | Yes |  |

### budget.cost_types

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `include_credit` | `Bool` | No |  |
| `include_discount` | `Bool` | No |  |
| `include_other_subscription` | `Bool` | No |  |
| `include_recurring` | `Bool` | No |  |
| `include_refund` | `Bool` | No |  |
| `include_subscription` | `Bool` | No |  |
| `include_support` | `Bool` | No |  |
| `include_tax` | `Bool` | No |  |
| `include_upfront` | `Bool` | No |  |
| `use_amortized` | `Bool` | No |  |
| `use_blended` | `Bool` | No |  |

### budget.time_period

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `end` | `String` | No |  |
| `start` | `String` | No |  |

### expression

Used by: `budget.filter_expression`, `expression.and`, `expression.not`, `expression.or`

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `and` | List\<[Struct(Expression)](#expression)\> (ordered) | No |  |
| `cost_categories` | [Struct(CostCategoryValues)](#expressioncost_categories) | No |  |
| `dimensions` | [Struct(ExpressionDimensionValues)](#expressiondimensions) | No |  |
| `not` | [Struct(Expression)](#expression) | No |  |
| `or` | List\<[Struct(Expression)](#expression)\> (ordered) | No |  |
| `tags` | [Struct(TagValues)](#expressiontags) | No |  |

### expression.cost_categories

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `key` | `String` | No |  |
| `match_options` | List\<[MatchOptions](#expressioncost_categoriesmatch_options-matchoptions)\> (ordered) | No |  |
| `values` | List\<`String (len: unbounded..=1024)`\> (items: 1..unbounded, ordered) | No |  |

### expression.dimensions

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `key` | [Key](#expressiondimensionskey-key) | No |  |
| `match_options` | List\<[MatchOptions](#expressiondimensionsmatch_options-matchoptions)\> (items: unbounded..=1, ordered) | No |  |
| `values` | List\<`String (len: unbounded..=1024)`\> (items: 1..unbounded, ordered) | No |  |

### expression.tags

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `key` | `String (len: unbounded..=1024)` | No |  |
| `match_options` | List\<[MatchOptions](#expressiontagsmatch_options-matchoptions)\> (ordered) | No |  |
| `values` | List\<`String (len: unbounded..=1024)`\> (items: 1..unbounded, ordered) | No |  |

### notifications_with_subscribers

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `notification` | [Struct(Notification)](#notifications_with_subscribersnotification) | Yes |  |
| `subscribers` | List\<[Struct(Subscriber)](#notifications_with_subscriberssubscribers)\> (ordered) | Yes |  |

### notifications_with_subscribers.notification

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `comparison_operator` | [ComparisonOperator](#notifications_with_subscribersnotificationcomparison_operator-comparisonoperator) | Yes |  |
| `notification_type` | [NotificationType](#notifications_with_subscribersnotificationnotification_type-notificationtype) | Yes |  |
| `threshold` | `Float (range: 0..=15000000000000)` | Yes |  |
| `threshold_type` | [ThresholdType](#notifications_with_subscribersnotificationthreshold_type-thresholdtype) | No |  |

### notifications_with_subscribers.subscribers

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `address` | `String` | Yes |  |
| `subscription_type` | [SubscriptionType](#notifications_with_subscriberssubscriberssubscription_type-subscriptiontype) | Yes |  |

### spend

Used by: `budget.budget_limit`, `budget.planned_budget_limits.<value>`

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `amount` | `Float (range: 0..unbounded)` | Yes |  |
| `unit` | `String` | Yes |  |

## Attribute Reference

### `id`

- **Type:** `String`

## AWS API narrowings

These fields are narrower than or equal to CFN. Input rules combine documented AWS domains, recorded live observations, and Carina canonical spellings for stable state comparison. A canonical DSL spelling does not claim AWS rejects other spellings. Attaching a type name adds no structural constraints. Field names below use CFN spelling.

| CFN field | Relation to CFN | Evidence URL | Verbatim quote / exact observation | Checked / observed |
| --- | --- | --- | --- | --- |
| `/definitions/AutoAdjustData/properties/AutoAdjustType` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/aws-cost-management/latest/APIReference/API_budgets_AutoAdjustData.html#awscostmanagement-Type-budgets_AutoAdjustData-AutoAdjustType) | ` The string that defines whether your budget auto-adjusts based on historical or forecasted data. `<br>` Valid Values: HISTORICAL \| FORECAST ` | 2026-11-07 |
| `/definitions/BudgetData/properties/BudgetName` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/aws-cost-management/latest/APIReference/API_budgets_Budget.html#awscostmanagement-Type-budgets_Budget-BudgetName) | ` The name of a budget. The name must be unique within an account. The : and \ characters, and the "/action/" substring, aren't allowed in BudgetName. `<br>` Budget names are validated for content. Names that contain phone numbers, URLs, or email addresses combined with certain terms may be rejected. `<br>` Length Constraints: Minimum length of 1. Maximum length of 100. `<br>` Pattern: ^(?![^:\\]*/action/\|(?i).*<script>.*</script>.*)[^:\\]+$ ` | 2026-11-07 |
| `/definitions/BudgetData/properties/BudgetType` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/aws-cost-management/latest/APIReference/API_budgets_Budget.html#awscostmanagement-Type-budgets_Budget-BudgetType) | ` Specifies whether this budget tracks costs, usage, RI utilization, RI coverage, Savings Plans utilization, or Savings Plans coverage. `<br>` Valid Values: USAGE \| COST \| RI_UTILIZATION \| RI_COVERAGE \| SAVINGS_PLANS_UTILIZATION \| SAVINGS_PLANS_COVERAGE ` | 2026-11-07 |
| `/definitions/BudgetData/properties/CostFilters` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/aws-cost-management/latest/APIReference/API_budgets_Budget.html#awscostmanagement-Type-budgets_Budget-CostFilters) | ` This member has been deprecated. `<br>` The cost filters, such as Region, Service, LinkedAccount, Tag, or CostCategory, that are applied to a budget. `<br>` AWS Budgets supports the following services as a Service filter for RI budgets: `<br>` Amazon EC2 `<br>` Amazon Redshift `<br>` Amazon Relational Database Service `<br>` Amazon ElastiCache `<br>` Amazon OpenSearch Service `<br>` Key Length Constraints: Minimum length of 0. Maximum length of 2147483647. `<br>` Key Pattern: .* `<br>` Length Constraints: Minimum length of 0. Maximum length of 2147483647. `<br>` Pattern: [\S\s]* ` | 2026-11-07 |
| `/definitions/BudgetData/properties/Metrics` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/aws-cost-management/latest/APIReference/API_budgets_Budget.html#awscostmanagement-Type-budgets_Budget-Metrics) | ` The definition for how the budget data is aggregated. `<br>` Array Members: Maximum number of 1 item. `<br>` Valid Values: BlendedCost \| UnblendedCost \| AmortizedCost \| NetUnblendedCost \| NetAmortizedCost \| UsageQuantity \| NormalizedUsageAmount \| Hours ` | 2026-11-07 |
| `/definitions/BudgetData/properties/PlannedBudgetLimits` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/aws-cost-management/latest/APIReference/API_budgets_Budget.html#awscostmanagement-Type-budgets_Budget-PlannedBudgetLimits) | ` A map containing multiple BudgetLimit, including current or future limits. `<br>` PlannedBudgetLimits is available for cost or usage budget and supports both monthly and quarterly TimeUnit. `<br>` For monthly budgets, provide 12 months of PlannedBudgetLimits values. This must start from the current month and include the next 11 months. The key is the start of the month, UTC in epoch seconds. `<br>` For quarterly budgets, provide four quarters of PlannedBudgetLimits value entries in standard calendar quarter increments. This must start from the current quarter and include the next three quarters. The key is the start of the quarter, UTC in epoch seconds. `<br>` If the planned budget expires before 12 months for monthly or four quarters for quarterly, provide the PlannedBudgetLimits values only for the remaining periods. `<br>` If the budget begins at a date in the future, provide PlannedBudgetLimits values from the start date of the budget. `<br>` After all of the BudgetLimit values in PlannedBudgetLimits are used, the budget continues to use the last limit as the BudgetLimit. At that point, the planned budget provides the same experience as a fixed budget. `<br>` DescribeBudget and DescribeBudgets response along with PlannedBudgetLimits also contain BudgetLimit representing the current month or quarter limit present in PlannedBudgetLimits. This only applies to budgets that are created with PlannedBudgetLimits. Budgets that are created without PlannedBudgetLimits only contain BudgetLimit. They don't contain PlannedBudgetLimits. `<br>` Key Length Constraints: Minimum length of 0. Maximum length of 2147483647. `<br>` Key Pattern: .* ` | 2026-11-07 |
| `/definitions/BudgetData/properties/TimeUnit` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/aws-cost-management/latest/APIReference/API_budgets_Budget.html#awscostmanagement-Type-budgets_Budget-TimeUnit) | ` The length of time until a budget resets the actual and forecasted spend. `<br>` Valid Values: DAILY \| MONTHLY \| QUARTERLY \| ANNUALLY \| CUSTOM ` | 2026-11-07 |
| `/definitions/CostCategoryValues/properties/MatchOptions` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/aws-cost-management/latest/APIReference/API_budgets_CostCategoryValues.html#awscostmanagement-Type-budgets_CostCategoryValues-MatchOptions) | ` The match options that you can use to filter your results. `<br>` Valid Values: EQUALS \| ABSENT \| STARTS_WITH \| ENDS_WITH \| CONTAINS \| GREATER_THAN_OR_EQUAL \| CASE_SENSITIVE \| CASE_INSENSITIVE ` | 2026-11-07 |
| `/definitions/CostCategoryValues/properties/Values` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/aws-cost-management/latest/APIReference/API_budgets_CostCategoryValues.html#awscostmanagement-Type-budgets_CostCategoryValues-Values) | ` The specific value of the cost category. `<br>` Array Members: Minimum number of 1 item. `<br>` Length Constraints: Minimum length of 0. Maximum length of 1024. `<br>` Pattern: [\S\s]* ` | 2026-11-07 |
| `/definitions/ExpressionDimensionValues/properties/Key` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/aws-cost-management/latest/APIReference/API_budgets_ExpressionDimensionValues.html#awscostmanagement-Type-budgets_ExpressionDimensionValues-Key) | ` The name of the dimension that you want to filter on. `<br>` Valid Values: AZ \| INSTANCE_TYPE \| LINKED_ACCOUNT \| LINKED_ACCOUNT_NAME \| OPERATION \| PURCHASE_TYPE \| REGION \| SERVICE \| SERVICE_CODE \| USAGE_TYPE \| USAGE_TYPE_GROUP \| RECORD_TYPE \| OPERATING_SYSTEM \| TENANCY \| SCOPE \| PLATFORM \| SUBSCRIPTION_ID \| LEGAL_ENTITY_NAME \| INVOICING_ENTITY \| DEPLOYMENT_OPTION \| DATABASE_ENGINE \| CACHE_ENGINE \| INSTANCE_TYPE_FAMILY \| BILLING_ENTITY \| RESERVATION_ID \| RESOURCE_ID \| RIGHTSIZING_TYPE \| SAVINGS_PLANS_TYPE \| SAVINGS_PLAN_ARN \| PAYMENT_OPTION \| RESERVATION_MODIFIED \| TAG_KEY \| COST_CATEGORY_NAME ` | 2026-11-07 |
| `/definitions/ExpressionDimensionValues/properties/MatchOptions` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/aws-cost-management/latest/APIReference/API_budgets_ExpressionDimensionValues.html#awscostmanagement-Type-budgets_ExpressionDimensionValues-MatchOptions) | ` The match options that you can use to filter your results. You can specify only one of these values in the array. `<br>` Valid Values: EQUALS \| ABSENT \| STARTS_WITH \| ENDS_WITH \| CONTAINS \| GREATER_THAN_OR_EQUAL \| CASE_SENSITIVE \| CASE_INSENSITIVE ` | 2026-11-07 |
| `/definitions/ExpressionDimensionValues/properties/Values` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/aws-cost-management/latest/APIReference/API_budgets_ExpressionDimensionValues.html#awscostmanagement-Type-budgets_ExpressionDimensionValues-Values) | ` The metadata values you can specify to filter upon, so that the results all match at least one of the specified values. `<br>` Array Members: Minimum number of 1 item. `<br>` Length Constraints: Minimum length of 0. Maximum length of 1024. `<br>` Pattern: [\S\s]* ` | 2026-11-07 |
| `/definitions/HistoricalOptions/properties/BudgetAdjustmentPeriod` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/aws-cost-management/latest/APIReference/API_budgets_HistoricalOptions.html#awscostmanagement-Type-budgets_HistoricalOptions-BudgetAdjustmentPeriod) | ` The number of budget periods included in the moving-average calculation that determines your auto-adjusted budget amount. The maximum value depends on the TimeUnit granularity of the budget: `<br>` For the DAILY granularity, the maximum value is 60. `<br>` For the MONTHLY granularity, the maximum value is 12. `<br>` For the QUARTERLY granularity, the maximum value is 4. `<br>` For the ANNUALLY granularity, the maximum value is 1. `<br>` Valid Range: Minimum value of 1. Maximum value of 60. ` | 2026-11-07 |
| `/definitions/Notification/properties/ComparisonOperator` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/aws-cost-management/latest/APIReference/API_budgets_Notification.html#awscostmanagement-Type-budgets_Notification-ComparisonOperator) | ` The comparison that's used for this notification. `<br>` Valid Values: GREATER_THAN \| LESS_THAN \| EQUAL_TO ` | 2026-11-07 |
| `/definitions/Notification/properties/NotificationType` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/aws-cost-management/latest/APIReference/API_budgets_Notification.html#awscostmanagement-Type-budgets_Notification-NotificationType) | ` Specifies whether the notification is for how much you have spent (ACTUAL) or for how much that you're forecasted to spend (FORECASTED). `<br>` Valid Values: ACTUAL \| FORECASTED ` | 2026-11-07 |
| `/definitions/Notification/properties/Threshold` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/aws-cost-management/latest/APIReference/API_budgets_Notification.html#awscostmanagement-Type-budgets_Notification-Threshold) | ` The threshold that's associated with a notification. Thresholds are always a percentage, and many customers find value being alerted between 50% - 200% of the budgeted amount. The maximum limit for your threshold is 1,000,000% above the budgeted amount. `<br>` Valid Range: Minimum value of 0. Maximum value of 15000000000000. ` | 2026-11-07 |
| `/definitions/Notification/properties/ThresholdType` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/aws-cost-management/latest/APIReference/API_budgets_Notification.html#awscostmanagement-Type-budgets_Notification-ThresholdType) | ` The type of threshold for a notification. For ABSOLUTE_VALUE thresholds, AWS notifies you when you go over or are forecasted to go over your total cost threshold. For PERCENTAGE thresholds, AWS notifies you when you go over or are forecasted to go over a certain percentage of your forecasted spend. For example, if you have a budget for 200 dollars and you have a PERCENTAGE threshold of 80%, AWS notifies you when you go over 160 dollars. `<br>` Valid Values: PERCENTAGE \| ABSOLUTE_VALUE ` | 2026-11-07 |
| `/definitions/Spend/properties/Amount` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/aws-cost-management/latest/APIReference/API_budgets_Spend.html#awscostmanagement-Type-budgets_Spend-Amount) | ` The cost or usage amount that's associated with a budget forecast, actual spend, or budget threshold. `<br>` Length Constraints: Minimum length of 1. Maximum length of 2147483647. `<br>` Pattern: ([0-9]*\.)?[0-9]+ ` | 2026-11-07 |
| `/definitions/Subscriber/properties/SubscriptionType` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/aws-cost-management/latest/APIReference/API_budgets_Subscriber.html#awscostmanagement-Type-budgets_Subscriber-SubscriptionType) | ` The type of notification that AWS sends to a subscriber. `<br>` Valid Values: SNS \| EMAIL ` | 2026-11-07 |
| `/definitions/TagValues/properties/Key` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/aws-cost-management/latest/APIReference/API_budgets_TagValues.html#awscostmanagement-Type-budgets_TagValues-Key) | ` The key for the tag. `<br>` Length Constraints: Minimum length of 0. Maximum length of 1024. `<br>` Pattern: [\S\s]* ` | 2026-11-07 |
| `/definitions/TagValues/properties/MatchOptions` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/aws-cost-management/latest/APIReference/API_budgets_TagValues.html#awscostmanagement-Type-budgets_TagValues-MatchOptions) | ` The match options that you can use to filter your results. `<br>` Valid Values: EQUALS \| ABSENT \| STARTS_WITH \| ENDS_WITH \| CONTAINS \| GREATER_THAN_OR_EQUAL \| CASE_SENSITIVE \| CASE_INSENSITIVE ` | 2026-11-07 |
| `/definitions/TagValues/properties/Values` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/aws-cost-management/latest/APIReference/API_budgets_TagValues.html#awscostmanagement-Type-budgets_TagValues-Values) | ` The specific value of the tag. `<br>` Array Members: Minimum number of 1 item. `<br>` Length Constraints: Minimum length of 0. Maximum length of 1024. `<br>` Pattern: [\S\s]* ` | 2026-11-07 |

