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
    budget_type  = 'COST'
    time_unit    = 'MONTHLY'
  }

  notifications_with_subscribers = [{
    notification = {
      comparison_operator = 'GREATER_THAN'
      notification_type   = 'ACTUAL'
      threshold           = 80.0
      threshold_type      = 'PERCENTAGE'
    }

    subscribers  = [{
      address           = 'finance@example.com'
      subscription_type = 'EMAIL'
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

## Struct Definitions

### budget

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `auto_adjust_data` | [Struct(AutoAdjustData)](#budgetauto_adjust_data) | No |  |
| `billing_view_arn` | Arn | No |  |
| `budget_limit` | [Struct(Spend)](#budgetbudget_limit) | No |  |
| `budget_name` | String | No |  |
| `budget_type` | String | Yes |  |
| `cost_filters` | `Map<String, List<String>>` | No |  |
| `cost_types` | [Struct(CostTypes)](#budgetcost_types) | No |  |
| `filter_expression` | [Struct(Expression)](#expression) | No |  |
| `metrics` | `List<String>` | No |  |
| `planned_budget_limits` | `Map<String, Spend>` | No |  |
| `time_period` | [Struct(TimePeriod)](#budgettime_period) | No |  |
| `time_unit` | String | Yes |  |

### budget.auto_adjust_data

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `auto_adjust_type` | String | Yes |  |
| `historical_options` | [Struct(HistoricalOptions)](#budgetauto_adjust_datahistorical_options) | No |  |

### budget.auto_adjust_data.historical_options

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `budget_adjustment_period` | Int | Yes |  |

### budget.budget_limit

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `amount` | Float | Yes |  |
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
| `match_options` | `List<String>` | No |  |
| `values` | `List<String>` | No |  |

### expression.dimensions

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `key` | String | No |  |
| `match_options` | `List<String>` | No |  |
| `values` | `List<String>` | No |  |

### expression.tags

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `key` | String | No |  |
| `match_options` | `List<String>` | No |  |
| `values` | `List<String>` | No |  |

### notifications_with_subscribers

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `notification` | [Struct(Notification)](#notifications_with_subscribersnotification) | Yes |  |
| `subscribers` | List\<[Struct(Subscriber)](#notifications_with_subscriberssubscribers)\> | Yes |  |

### notifications_with_subscribers.notification

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `comparison_operator` | String | Yes |  |
| `notification_type` | String | Yes |  |
| `threshold` | Float | Yes |  |
| `threshold_type` | String | No |  |

### notifications_with_subscribers.subscribers

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `address` | String | Yes |  |
| `subscription_type` | String | Yes |  |

### resource_tags

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `key` | String | Yes |  |
| `value` | String | No |  |

## Attribute Reference

### `id`

- **Type:** String

