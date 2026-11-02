//! budget schema definition for AWS Cloud Control
//!
//! Auto-generated from CloudFormation schema: AWS::Budgets::Budget
//!
//! DO NOT EDIT MANUALLY - regenerate with carina-codegen

use crate::schemas::config::AwsccSchemaConfig;
use carina_core::schema::{AttributeSchema, AttributeType, ResourceSchema, StructField};

const VALID_AUTO_ADJUST_DATA_AUTO_ADJUST_TYPE: &[&str] = &["HISTORICAL", "FORECAST"];

const VALID_BUDGET_DATA_BUDGET_TYPE: &[&str] = &[
    "USAGE",
    "COST",
    "RI_UTILIZATION",
    "RI_COVERAGE",
    "SAVINGS_PLANS_UTILIZATION",
    "SAVINGS_PLANS_COVERAGE",
];

const VALID_BUDGET_DATA_METRICS: &[&str] = &[
    "BlendedCost",
    "UnblendedCost",
    "AmortizedCost",
    "NetUnblendedCost",
    "NetAmortizedCost",
    "UsageQuantity",
    "NormalizedUsageAmount",
    "Hours",
];

const VALID_BUDGET_DATA_TIME_UNIT: &[&str] =
    &["DAILY", "MONTHLY", "QUARTERLY", "ANNUALLY", "CUSTOM"];

const VALID_COST_CATEGORY_VALUES_MATCH_OPTIONS: &[&str] = &[
    "EQUALS",
    "ABSENT",
    "STARTS_WITH",
    "ENDS_WITH",
    "CONTAINS",
    "GREATER_THAN_OR_EQUAL",
    "CASE_SENSITIVE",
    "CASE_INSENSITIVE",
];

const VALID_EXPRESSION_DIMENSION_VALUES_KEY: &[&str] = &[
    "AZ",
    "INSTANCE_TYPE",
    "LINKED_ACCOUNT",
    "LINKED_ACCOUNT_NAME",
    "OPERATION",
    "PURCHASE_TYPE",
    "REGION",
    "SERVICE",
    "SERVICE_CODE",
    "USAGE_TYPE",
    "USAGE_TYPE_GROUP",
    "RECORD_TYPE",
    "OPERATING_SYSTEM",
    "TENANCY",
    "SCOPE",
    "PLATFORM",
    "SUBSCRIPTION_ID",
    "LEGAL_ENTITY_NAME",
    "INVOICING_ENTITY",
    "DEPLOYMENT_OPTION",
    "DATABASE_ENGINE",
    "CACHE_ENGINE",
    "INSTANCE_TYPE_FAMILY",
    "BILLING_ENTITY",
    "RESERVATION_ID",
    "RESOURCE_ID",
    "RIGHTSIZING_TYPE",
    "SAVINGS_PLANS_TYPE",
    "SAVINGS_PLAN_ARN",
    "PAYMENT_OPTION",
    "RESERVATION_MODIFIED",
    "TAG_KEY",
    "COST_CATEGORY_NAME",
];

const VALID_EXPRESSION_DIMENSION_VALUES_MATCH_OPTIONS: &[&str] = &[
    "EQUALS",
    "ABSENT",
    "STARTS_WITH",
    "ENDS_WITH",
    "CONTAINS",
    "GREATER_THAN_OR_EQUAL",
    "CASE_SENSITIVE",
    "CASE_INSENSITIVE",
];

const VALID_NOTIFICATION_COMPARISON_OPERATOR: &[&str] = &["GREATER_THAN", "LESS_THAN", "EQUAL_TO"];

const VALID_NOTIFICATION_NOTIFICATION_TYPE: &[&str] = &["ACTUAL", "FORECASTED"];

const VALID_NOTIFICATION_THRESHOLD_TYPE: &[&str] = &["PERCENTAGE", "ABSOLUTE_VALUE"];

const VALID_SUBSCRIBER_SUBSCRIPTION_TYPE: &[&str] = &["SNS", "EMAIL"];

const VALID_TAG_VALUES_MATCH_OPTIONS: &[&str] = &[
    "EQUALS",
    "ABSENT",
    "STARTS_WITH",
    "ENDS_WITH",
    "CONTAINS",
    "GREATER_THAN_OR_EQUAL",
    "CASE_SENSITIVE",
    "CASE_INSENSITIVE",
];

/// Returns the schema config for budgets_budget (AWS::Budgets::Budget)
#[rustfmt::skip]
pub fn budgets_budget_config() -> AwsccSchemaConfig {
    let mut schema = ResourceSchema::new("budgets.Budget")
        .with_description("Resource Type definition for AWS::Budgets::Budget");
    #[inline(never)]
    fn build_budget_attribute() -> AttributeSchema {
        AttributeSchema::required("budget", AttributeType::struct_("BudgetData".to_string(), vec![StructField::optional("auto_adjust_data", AttributeType::struct_("AutoAdjustData".to_string(), vec![StructField::required("auto_adjust_type", AttributeType::enum_(carina_core::schema::enum_identity("AutoAdjustType", Some("aws.budgets.Budget.BudgetData.AutoAdjustData")), Some(vec!["HISTORICAL".to_string(), "FORECAST".to_string()]), vec![("HISTORICAL".to_string(), "historical".to_string()), ("FORECAST".to_string(), "forecast".to_string())], None, None)).with_provider_name("AutoAdjustType").build(),
                    StructField::optional("historical_options", AttributeType::struct_("HistoricalOptions".to_string(), vec![StructField::required("budget_adjustment_period", AttributeType::refined_int(None, Some((Some(1), Some(60))))).with_provider_name("BudgetAdjustmentPeriod").build()])).with_provider_name("HistoricalOptions").build()])).with_provider_name("AutoAdjustData").build(),
                    StructField::optional("billing_view_arn", carina_aws_types::arn()).with_provider_name("BillingViewArn").build(),
                    StructField::optional("budget_limit", AttributeType::struct_("Spend".to_string(), vec![StructField::required("amount", AttributeType::refined_float(None, Some((Some(0.0), None)))).with_provider_name("Amount").build(),
                    StructField::required("unit", AttributeType::string()).with_provider_name("Unit").build()])).with_provider_name("BudgetLimit").build(),
                    StructField::optional("budget_name", AttributeType::string()).create_only().identity().with_provider_name("BudgetName").build(),
                    StructField::required("budget_type", AttributeType::enum_(carina_core::schema::enum_identity("BudgetType", Some("aws.budgets.Budget.BudgetData")), Some(vec!["USAGE".to_string(), "COST".to_string(), "RI_UTILIZATION".to_string(), "RI_COVERAGE".to_string(), "SAVINGS_PLANS_UTILIZATION".to_string(), "SAVINGS_PLANS_COVERAGE".to_string()]), vec![("USAGE".to_string(), "usage".to_string()), ("COST".to_string(), "cost".to_string()), ("RI_UTILIZATION".to_string(), "ri_utilization".to_string()), ("RI_COVERAGE".to_string(), "ri_coverage".to_string()), ("SAVINGS_PLANS_UTILIZATION".to_string(), "savings_plans_utilization".to_string()), ("SAVINGS_PLANS_COVERAGE".to_string(), "savings_plans_coverage".to_string())], None, None)).with_provider_name("BudgetType").build(),
                    StructField::optional("cost_filters", AttributeType::map(AttributeType::list(AttributeType::string()))).with_provider_name("CostFilters").build(),
                    StructField::optional("cost_types", AttributeType::struct_("CostTypes".to_string(), vec![StructField::optional("include_credit", AttributeType::bool()).with_provider_name("IncludeCredit").build(),
                    StructField::optional("include_discount", AttributeType::bool()).with_provider_name("IncludeDiscount").build(),
                    StructField::optional("include_other_subscription", AttributeType::bool()).with_provider_name("IncludeOtherSubscription").build(),
                    StructField::optional("include_recurring", AttributeType::bool()).with_provider_name("IncludeRecurring").build(),
                    StructField::optional("include_refund", AttributeType::bool()).with_provider_name("IncludeRefund").build(),
                    StructField::optional("include_subscription", AttributeType::bool()).with_provider_name("IncludeSubscription").build(),
                    StructField::optional("include_support", AttributeType::bool()).with_provider_name("IncludeSupport").build(),
                    StructField::optional("include_tax", AttributeType::bool()).with_provider_name("IncludeTax").build(),
                    StructField::optional("include_upfront", AttributeType::bool()).with_provider_name("IncludeUpfront").build(),
                    StructField::optional("use_amortized", AttributeType::bool()).with_provider_name("UseAmortized").build(),
                    StructField::optional("use_blended", AttributeType::bool()).with_provider_name("UseBlended").build()])).with_provider_name("CostTypes").build(),
                    StructField::optional("filter_expression", AttributeType::struct_("Expression".to_string(), vec![StructField::optional("and", AttributeType::list(AttributeType::ref_("Expression".to_string()))).with_provider_name("And").with_block_name("and").build(),
                    StructField::optional("cost_categories", AttributeType::struct_("CostCategoryValues".to_string(), vec![StructField::optional("key", AttributeType::string()).with_provider_name("Key").build(),
                    StructField::optional("match_options", AttributeType::list(AttributeType::enum_(carina_core::schema::enum_identity("MatchOptions", Some("aws.budgets.Budget.BudgetData.Expression.CostCategoryValues")), Some(vec!["EQUALS".to_string(), "ABSENT".to_string(), "STARTS_WITH".to_string(), "ENDS_WITH".to_string(), "CONTAINS".to_string(), "GREATER_THAN_OR_EQUAL".to_string(), "CASE_SENSITIVE".to_string(), "CASE_INSENSITIVE".to_string()]), vec![("EQUALS".to_string(), "equals".to_string()), ("ABSENT".to_string(), "absent".to_string()), ("STARTS_WITH".to_string(), "starts_with".to_string()), ("ENDS_WITH".to_string(), "ends_with".to_string()), ("CONTAINS".to_string(), "contains".to_string()), ("GREATER_THAN_OR_EQUAL".to_string(), "greater_than_or_equal".to_string()), ("CASE_SENSITIVE".to_string(), "case_sensitive".to_string()), ("CASE_INSENSITIVE".to_string(), "case_insensitive".to_string())], None, None))).with_provider_name("MatchOptions").build(),
                    StructField::optional("values", AttributeType::refined_list(AttributeType::string(), true, Some((Some(1), None)), carina_core::schema::ListElements::AllowDuplicates)).with_provider_name("Values").build()])).with_provider_name("CostCategories").build(),
                    StructField::optional("dimensions", AttributeType::struct_("ExpressionDimensionValues".to_string(), vec![StructField::optional("key", AttributeType::enum_(carina_core::schema::enum_identity("Key", Some("aws.budgets.Budget.BudgetData.Expression.ExpressionDimensionValues")), Some(vec!["AZ".to_string(), "INSTANCE_TYPE".to_string(), "LINKED_ACCOUNT".to_string(), "LINKED_ACCOUNT_NAME".to_string(), "OPERATION".to_string(), "PURCHASE_TYPE".to_string(), "REGION".to_string(), "SERVICE".to_string(), "SERVICE_CODE".to_string(), "USAGE_TYPE".to_string(), "USAGE_TYPE_GROUP".to_string(), "RECORD_TYPE".to_string(), "OPERATING_SYSTEM".to_string(), "TENANCY".to_string(), "SCOPE".to_string(), "PLATFORM".to_string(), "SUBSCRIPTION_ID".to_string(), "LEGAL_ENTITY_NAME".to_string(), "INVOICING_ENTITY".to_string(), "DEPLOYMENT_OPTION".to_string(), "DATABASE_ENGINE".to_string(), "CACHE_ENGINE".to_string(), "INSTANCE_TYPE_FAMILY".to_string(), "BILLING_ENTITY".to_string(), "RESERVATION_ID".to_string(), "RESOURCE_ID".to_string(), "RIGHTSIZING_TYPE".to_string(), "SAVINGS_PLANS_TYPE".to_string(), "SAVINGS_PLAN_ARN".to_string(), "PAYMENT_OPTION".to_string(), "RESERVATION_MODIFIED".to_string(), "TAG_KEY".to_string(), "COST_CATEGORY_NAME".to_string()]), vec![("AZ".to_string(), "az".to_string()), ("INSTANCE_TYPE".to_string(), "instance_type".to_string()), ("LINKED_ACCOUNT".to_string(), "linked_account".to_string()), ("LINKED_ACCOUNT_NAME".to_string(), "linked_account_name".to_string()), ("OPERATION".to_string(), "operation".to_string()), ("PURCHASE_TYPE".to_string(), "purchase_type".to_string()), ("REGION".to_string(), "region".to_string()), ("SERVICE".to_string(), "service".to_string()), ("SERVICE_CODE".to_string(), "service_code".to_string()), ("USAGE_TYPE".to_string(), "usage_type".to_string()), ("USAGE_TYPE_GROUP".to_string(), "usage_type_group".to_string()), ("RECORD_TYPE".to_string(), "record_type".to_string()), ("OPERATING_SYSTEM".to_string(), "operating_system".to_string()), ("TENANCY".to_string(), "tenancy".to_string()), ("SCOPE".to_string(), "scope".to_string()), ("PLATFORM".to_string(), "platform".to_string()), ("SUBSCRIPTION_ID".to_string(), "subscription_id".to_string()), ("LEGAL_ENTITY_NAME".to_string(), "legal_entity_name".to_string()), ("INVOICING_ENTITY".to_string(), "invoicing_entity".to_string()), ("DEPLOYMENT_OPTION".to_string(), "deployment_option".to_string()), ("DATABASE_ENGINE".to_string(), "database_engine".to_string()), ("CACHE_ENGINE".to_string(), "cache_engine".to_string()), ("INSTANCE_TYPE_FAMILY".to_string(), "instance_type_family".to_string()), ("BILLING_ENTITY".to_string(), "billing_entity".to_string()), ("RESERVATION_ID".to_string(), "reservation_id".to_string()), ("RESOURCE_ID".to_string(), "resource_id".to_string()), ("RIGHTSIZING_TYPE".to_string(), "rightsizing_type".to_string()), ("SAVINGS_PLANS_TYPE".to_string(), "savings_plans_type".to_string()), ("SAVINGS_PLAN_ARN".to_string(), "savings_plan_arn".to_string()), ("PAYMENT_OPTION".to_string(), "payment_option".to_string()), ("RESERVATION_MODIFIED".to_string(), "reservation_modified".to_string()), ("TAG_KEY".to_string(), "tag_key".to_string()), ("COST_CATEGORY_NAME".to_string(), "cost_category_name".to_string())], None, None)).with_provider_name("Key").build(),
                    StructField::optional("match_options", AttributeType::refined_list(AttributeType::enum_(carina_core::schema::enum_identity("MatchOptions", Some("aws.budgets.Budget.BudgetData.Expression.ExpressionDimensionValues")), Some(vec!["EQUALS".to_string(), "ABSENT".to_string(), "STARTS_WITH".to_string(), "ENDS_WITH".to_string(), "CONTAINS".to_string(), "GREATER_THAN_OR_EQUAL".to_string(), "CASE_SENSITIVE".to_string(), "CASE_INSENSITIVE".to_string()]), vec![("EQUALS".to_string(), "equals".to_string()), ("ABSENT".to_string(), "absent".to_string()), ("STARTS_WITH".to_string(), "starts_with".to_string()), ("ENDS_WITH".to_string(), "ends_with".to_string()), ("CONTAINS".to_string(), "contains".to_string()), ("GREATER_THAN_OR_EQUAL".to_string(), "greater_than_or_equal".to_string()), ("CASE_SENSITIVE".to_string(), "case_sensitive".to_string()), ("CASE_INSENSITIVE".to_string(), "case_insensitive".to_string())], None, None), true, Some((None, Some(1))), carina_core::schema::ListElements::AllowDuplicates)).with_provider_name("MatchOptions").build(),
                    StructField::optional("values", AttributeType::refined_list(AttributeType::string(), true, Some((Some(1), None)), carina_core::schema::ListElements::AllowDuplicates)).with_provider_name("Values").build()])).with_provider_name("Dimensions").build(),
                    StructField::optional("not", AttributeType::ref_("Expression".to_string())).with_provider_name("Not").build(),
                    StructField::optional("or", AttributeType::list(AttributeType::ref_("Expression".to_string()))).with_provider_name("Or").with_block_name("or").build(),
                    StructField::optional("tags", AttributeType::struct_("TagValues".to_string(), vec![StructField::optional("key", AttributeType::string()).with_provider_name("Key").build(),
                    StructField::optional("match_options", AttributeType::list(AttributeType::enum_(carina_core::schema::enum_identity("MatchOptions", Some("aws.budgets.Budget.BudgetData.Expression.TagValues")), Some(vec!["EQUALS".to_string(), "ABSENT".to_string(), "STARTS_WITH".to_string(), "ENDS_WITH".to_string(), "CONTAINS".to_string(), "GREATER_THAN_OR_EQUAL".to_string(), "CASE_SENSITIVE".to_string(), "CASE_INSENSITIVE".to_string()]), vec![("EQUALS".to_string(), "equals".to_string()), ("ABSENT".to_string(), "absent".to_string()), ("STARTS_WITH".to_string(), "starts_with".to_string()), ("ENDS_WITH".to_string(), "ends_with".to_string()), ("CONTAINS".to_string(), "contains".to_string()), ("GREATER_THAN_OR_EQUAL".to_string(), "greater_than_or_equal".to_string()), ("CASE_SENSITIVE".to_string(), "case_sensitive".to_string()), ("CASE_INSENSITIVE".to_string(), "case_insensitive".to_string())], None, None))).with_provider_name("MatchOptions").build(),
                    StructField::optional("values", AttributeType::refined_list(AttributeType::string(), true, Some((Some(1), None)), carina_core::schema::ListElements::AllowDuplicates)).with_provider_name("Values").build()])).with_provider_name("Tags").build()])).with_provider_name("FilterExpression").build(),
                    StructField::optional("metrics", AttributeType::refined_list(AttributeType::enum_(carina_core::schema::enum_identity("Metrics", Some("aws.budgets.Budget.BudgetData")), Some(vec!["BlendedCost".to_string(), "UnblendedCost".to_string(), "AmortizedCost".to_string(), "NetUnblendedCost".to_string(), "NetAmortizedCost".to_string(), "UsageQuantity".to_string(), "NormalizedUsageAmount".to_string(), "Hours".to_string()]), vec![("BlendedCost".to_string(), "blended_cost".to_string()), ("UnblendedCost".to_string(), "unblended_cost".to_string()), ("AmortizedCost".to_string(), "amortized_cost".to_string()), ("NetUnblendedCost".to_string(), "net_unblended_cost".to_string()), ("NetAmortizedCost".to_string(), "net_amortized_cost".to_string()), ("UsageQuantity".to_string(), "usage_quantity".to_string()), ("NormalizedUsageAmount".to_string(), "normalized_usage_amount".to_string()), ("Hours".to_string(), "hours".to_string())], None, None), true, Some((None, Some(1))), carina_core::schema::ListElements::AllowDuplicates)).with_provider_name("Metrics").build(),
                    StructField::optional("planned_budget_limits", AttributeType::map(AttributeType::ref_("Spend".to_string()))).with_provider_name("PlannedBudgetLimits").build(),
                    StructField::optional("time_period", AttributeType::struct_("TimePeriod".to_string(), vec![StructField::optional("end", AttributeType::string()).with_provider_name("End").build(),
                    StructField::optional("start", AttributeType::string()).with_provider_name("Start").build()])).with_provider_name("TimePeriod").build(),
                    StructField::required("time_unit", AttributeType::enum_(carina_core::schema::enum_identity("TimeUnit", Some("aws.budgets.Budget.BudgetData")), Some(vec!["DAILY".to_string(), "MONTHLY".to_string(), "QUARTERLY".to_string(), "ANNUALLY".to_string(), "CUSTOM".to_string()]), vec![("DAILY".to_string(), "daily".to_string()), ("MONTHLY".to_string(), "monthly".to_string()), ("QUARTERLY".to_string(), "quarterly".to_string()), ("ANNUALLY".to_string(), "annually".to_string()), ("CUSTOM".to_string(), "custom".to_string())], None, None)).with_provider_name("TimeUnit").build()]))
                .with_provider_name("Budget")
            .build()
    }
    schema = schema.attribute(build_budget_attribute());
    #[inline(never)]
    fn build_id_attribute() -> AttributeSchema {
        AttributeSchema::provider_populated("id", AttributeType::string())
                .with_provider_name("Id")
            .build()
    }
    schema = schema.attribute(build_id_attribute());
    #[inline(never)]
    fn build_notifications_with_subscribers_attribute() -> AttributeSchema {
        AttributeSchema::optional("notifications_with_subscribers", AttributeType::list(AttributeType::struct_("NotificationWithSubscribers".to_string(), vec![StructField::required("notification", AttributeType::struct_("Notification".to_string(), vec![StructField::required("comparison_operator", AttributeType::enum_(carina_core::schema::enum_identity("ComparisonOperator", Some("aws.budgets.Budget.NotificationWithSubscribers.Notification")), Some(vec!["GREATER_THAN".to_string(), "LESS_THAN".to_string(), "EQUAL_TO".to_string()]), vec![("GREATER_THAN".to_string(), "greater_than".to_string()), ("LESS_THAN".to_string(), "less_than".to_string()), ("EQUAL_TO".to_string(), "equal_to".to_string())], None, None)).with_provider_name("ComparisonOperator").build(),
                    StructField::required("notification_type", AttributeType::enum_(carina_core::schema::enum_identity("NotificationType", Some("aws.budgets.Budget.NotificationWithSubscribers.Notification")), Some(vec!["ACTUAL".to_string(), "FORECASTED".to_string()]), vec![("ACTUAL".to_string(), "actual".to_string()), ("FORECASTED".to_string(), "forecasted".to_string())], None, None)).with_provider_name("NotificationType").build(),
                    StructField::required("threshold", AttributeType::refined_float(None, Some((Some(0.0), Some(15000000000000.0))))).with_provider_name("Threshold").build(),
                    StructField::optional("threshold_type", AttributeType::enum_(carina_core::schema::enum_identity("ThresholdType", Some("aws.budgets.Budget.NotificationWithSubscribers.Notification")), Some(vec!["PERCENTAGE".to_string(), "ABSOLUTE_VALUE".to_string()]), vec![("PERCENTAGE".to_string(), "percentage".to_string()), ("ABSOLUTE_VALUE".to_string(), "absolute_value".to_string())], None, None)).with_provider_name("ThresholdType").build()])).with_provider_name("Notification").build(),
                    StructField::required("subscribers", AttributeType::list(AttributeType::struct_("Subscriber".to_string(), vec![StructField::required("address", AttributeType::string()).with_provider_name("Address").build(),
                    StructField::required("subscription_type", AttributeType::enum_(carina_core::schema::enum_identity("SubscriptionType", Some("aws.budgets.Budget.NotificationWithSubscribers.Subscriber")), Some(vec!["SNS".to_string(), "EMAIL".to_string()]), vec![("SNS".to_string(), "sns".to_string()), ("EMAIL".to_string(), "email".to_string())], None, None)).with_provider_name("SubscriptionType").build()]))).with_provider_name("Subscribers").with_block_name("subscriber").build()])))
                .create_only()
                .with_provider_name("NotificationsWithSubscribers")
                .with_block_name("notifications_with_subscriber")
            .build()
    }
    schema = schema.attribute(build_notifications_with_subscribers_attribute());
    #[inline(never)]
    fn build_resource_tags_attribute() -> AttributeSchema {
        AttributeSchema::optional("resource_tags", AttributeType::list(AttributeType::struct_("ResourceTag".to_string(), vec![StructField::required("key", AttributeType::string()).with_provider_name("Key").build(),
                    StructField::optional("value", AttributeType::string()).with_provider_name("Value").build()])))
                .with_provider_name("ResourceTags")
                .with_block_name("resource_tag")
            .build()
    }
    schema = schema.attribute(build_resource_tags_attribute());
    schema = schema
        .with_conflicting_replacement();
    #[inline(never)]
    fn build_schema_def_0() -> AttributeType { AttributeType::struct_("AutoAdjustData".to_string(), vec![StructField::required("auto_adjust_type", AttributeType::enum_(carina_core::schema::enum_identity("AutoAdjustType", Some("aws.budgets.Budget.BudgetData.AutoAdjustData")), Some(vec!["HISTORICAL".to_string(), "FORECAST".to_string()]), vec![("HISTORICAL".to_string(), "historical".to_string()), ("FORECAST".to_string(), "forecast".to_string())], None, None)).with_provider_name("AutoAdjustType").build(),
                    StructField::optional("historical_options", AttributeType::struct_("HistoricalOptions".to_string(), vec![StructField::required("budget_adjustment_period", AttributeType::refined_int(None, Some((Some(1), Some(60))))).with_provider_name("BudgetAdjustmentPeriod").build()])).with_provider_name("HistoricalOptions").build()]) }
    schema = schema.with_def("AutoAdjustData", build_schema_def_0());
    #[inline(never)]
    fn build_schema_def_1() -> AttributeType { AttributeType::struct_("BudgetData".to_string(), vec![StructField::optional("auto_adjust_data", AttributeType::struct_("AutoAdjustData".to_string(), vec![StructField::required("auto_adjust_type", AttributeType::enum_(carina_core::schema::enum_identity("AutoAdjustType", Some("aws.budgets.Budget.BudgetData.AutoAdjustData")), Some(vec!["HISTORICAL".to_string(), "FORECAST".to_string()]), vec![("HISTORICAL".to_string(), "historical".to_string()), ("FORECAST".to_string(), "forecast".to_string())], None, None)).with_provider_name("AutoAdjustType").build(),
                    StructField::optional("historical_options", AttributeType::struct_("HistoricalOptions".to_string(), vec![StructField::required("budget_adjustment_period", AttributeType::refined_int(None, Some((Some(1), Some(60))))).with_provider_name("BudgetAdjustmentPeriod").build()])).with_provider_name("HistoricalOptions").build()])).with_provider_name("AutoAdjustData").build(),
                    StructField::optional("billing_view_arn", carina_aws_types::arn()).with_provider_name("BillingViewArn").build(),
                    StructField::optional("budget_limit", AttributeType::struct_("Spend".to_string(), vec![StructField::required("amount", AttributeType::refined_float(None, Some((Some(0.0), None)))).with_provider_name("Amount").build(),
                    StructField::required("unit", AttributeType::string()).with_provider_name("Unit").build()])).with_provider_name("BudgetLimit").build(),
                    StructField::optional("budget_name", AttributeType::string()).create_only().identity().with_provider_name("BudgetName").build(),
                    StructField::required("budget_type", AttributeType::enum_(carina_core::schema::enum_identity("BudgetType", Some("aws.budgets.Budget.BudgetData")), Some(vec!["USAGE".to_string(), "COST".to_string(), "RI_UTILIZATION".to_string(), "RI_COVERAGE".to_string(), "SAVINGS_PLANS_UTILIZATION".to_string(), "SAVINGS_PLANS_COVERAGE".to_string()]), vec![("USAGE".to_string(), "usage".to_string()), ("COST".to_string(), "cost".to_string()), ("RI_UTILIZATION".to_string(), "ri_utilization".to_string()), ("RI_COVERAGE".to_string(), "ri_coverage".to_string()), ("SAVINGS_PLANS_UTILIZATION".to_string(), "savings_plans_utilization".to_string()), ("SAVINGS_PLANS_COVERAGE".to_string(), "savings_plans_coverage".to_string())], None, None)).with_provider_name("BudgetType").build(),
                    StructField::optional("cost_filters", AttributeType::map(AttributeType::list(AttributeType::string()))).with_provider_name("CostFilters").build(),
                    StructField::optional("cost_types", AttributeType::struct_("CostTypes".to_string(), vec![StructField::optional("include_credit", AttributeType::bool()).with_provider_name("IncludeCredit").build(),
                    StructField::optional("include_discount", AttributeType::bool()).with_provider_name("IncludeDiscount").build(),
                    StructField::optional("include_other_subscription", AttributeType::bool()).with_provider_name("IncludeOtherSubscription").build(),
                    StructField::optional("include_recurring", AttributeType::bool()).with_provider_name("IncludeRecurring").build(),
                    StructField::optional("include_refund", AttributeType::bool()).with_provider_name("IncludeRefund").build(),
                    StructField::optional("include_subscription", AttributeType::bool()).with_provider_name("IncludeSubscription").build(),
                    StructField::optional("include_support", AttributeType::bool()).with_provider_name("IncludeSupport").build(),
                    StructField::optional("include_tax", AttributeType::bool()).with_provider_name("IncludeTax").build(),
                    StructField::optional("include_upfront", AttributeType::bool()).with_provider_name("IncludeUpfront").build(),
                    StructField::optional("use_amortized", AttributeType::bool()).with_provider_name("UseAmortized").build(),
                    StructField::optional("use_blended", AttributeType::bool()).with_provider_name("UseBlended").build()])).with_provider_name("CostTypes").build(),
                    StructField::optional("filter_expression", AttributeType::struct_("Expression".to_string(), vec![StructField::optional("and", AttributeType::list(AttributeType::ref_("Expression".to_string()))).with_provider_name("And").with_block_name("and").build(),
                    StructField::optional("cost_categories", AttributeType::struct_("CostCategoryValues".to_string(), vec![StructField::optional("key", AttributeType::string()).with_provider_name("Key").build(),
                    StructField::optional("match_options", AttributeType::list(AttributeType::enum_(carina_core::schema::enum_identity("MatchOptions", Some("aws.budgets.Budget.BudgetData.Expression.CostCategoryValues")), Some(vec!["EQUALS".to_string(), "ABSENT".to_string(), "STARTS_WITH".to_string(), "ENDS_WITH".to_string(), "CONTAINS".to_string(), "GREATER_THAN_OR_EQUAL".to_string(), "CASE_SENSITIVE".to_string(), "CASE_INSENSITIVE".to_string()]), vec![("EQUALS".to_string(), "equals".to_string()), ("ABSENT".to_string(), "absent".to_string()), ("STARTS_WITH".to_string(), "starts_with".to_string()), ("ENDS_WITH".to_string(), "ends_with".to_string()), ("CONTAINS".to_string(), "contains".to_string()), ("GREATER_THAN_OR_EQUAL".to_string(), "greater_than_or_equal".to_string()), ("CASE_SENSITIVE".to_string(), "case_sensitive".to_string()), ("CASE_INSENSITIVE".to_string(), "case_insensitive".to_string())], None, None))).with_provider_name("MatchOptions").build(),
                    StructField::optional("values", AttributeType::refined_list(AttributeType::string(), true, Some((Some(1), None)), carina_core::schema::ListElements::AllowDuplicates)).with_provider_name("Values").build()])).with_provider_name("CostCategories").build(),
                    StructField::optional("dimensions", AttributeType::struct_("ExpressionDimensionValues".to_string(), vec![StructField::optional("key", AttributeType::enum_(carina_core::schema::enum_identity("Key", Some("aws.budgets.Budget.BudgetData.Expression.ExpressionDimensionValues")), Some(vec!["AZ".to_string(), "INSTANCE_TYPE".to_string(), "LINKED_ACCOUNT".to_string(), "LINKED_ACCOUNT_NAME".to_string(), "OPERATION".to_string(), "PURCHASE_TYPE".to_string(), "REGION".to_string(), "SERVICE".to_string(), "SERVICE_CODE".to_string(), "USAGE_TYPE".to_string(), "USAGE_TYPE_GROUP".to_string(), "RECORD_TYPE".to_string(), "OPERATING_SYSTEM".to_string(), "TENANCY".to_string(), "SCOPE".to_string(), "PLATFORM".to_string(), "SUBSCRIPTION_ID".to_string(), "LEGAL_ENTITY_NAME".to_string(), "INVOICING_ENTITY".to_string(), "DEPLOYMENT_OPTION".to_string(), "DATABASE_ENGINE".to_string(), "CACHE_ENGINE".to_string(), "INSTANCE_TYPE_FAMILY".to_string(), "BILLING_ENTITY".to_string(), "RESERVATION_ID".to_string(), "RESOURCE_ID".to_string(), "RIGHTSIZING_TYPE".to_string(), "SAVINGS_PLANS_TYPE".to_string(), "SAVINGS_PLAN_ARN".to_string(), "PAYMENT_OPTION".to_string(), "RESERVATION_MODIFIED".to_string(), "TAG_KEY".to_string(), "COST_CATEGORY_NAME".to_string()]), vec![("AZ".to_string(), "az".to_string()), ("INSTANCE_TYPE".to_string(), "instance_type".to_string()), ("LINKED_ACCOUNT".to_string(), "linked_account".to_string()), ("LINKED_ACCOUNT_NAME".to_string(), "linked_account_name".to_string()), ("OPERATION".to_string(), "operation".to_string()), ("PURCHASE_TYPE".to_string(), "purchase_type".to_string()), ("REGION".to_string(), "region".to_string()), ("SERVICE".to_string(), "service".to_string()), ("SERVICE_CODE".to_string(), "service_code".to_string()), ("USAGE_TYPE".to_string(), "usage_type".to_string()), ("USAGE_TYPE_GROUP".to_string(), "usage_type_group".to_string()), ("RECORD_TYPE".to_string(), "record_type".to_string()), ("OPERATING_SYSTEM".to_string(), "operating_system".to_string()), ("TENANCY".to_string(), "tenancy".to_string()), ("SCOPE".to_string(), "scope".to_string()), ("PLATFORM".to_string(), "platform".to_string()), ("SUBSCRIPTION_ID".to_string(), "subscription_id".to_string()), ("LEGAL_ENTITY_NAME".to_string(), "legal_entity_name".to_string()), ("INVOICING_ENTITY".to_string(), "invoicing_entity".to_string()), ("DEPLOYMENT_OPTION".to_string(), "deployment_option".to_string()), ("DATABASE_ENGINE".to_string(), "database_engine".to_string()), ("CACHE_ENGINE".to_string(), "cache_engine".to_string()), ("INSTANCE_TYPE_FAMILY".to_string(), "instance_type_family".to_string()), ("BILLING_ENTITY".to_string(), "billing_entity".to_string()), ("RESERVATION_ID".to_string(), "reservation_id".to_string()), ("RESOURCE_ID".to_string(), "resource_id".to_string()), ("RIGHTSIZING_TYPE".to_string(), "rightsizing_type".to_string()), ("SAVINGS_PLANS_TYPE".to_string(), "savings_plans_type".to_string()), ("SAVINGS_PLAN_ARN".to_string(), "savings_plan_arn".to_string()), ("PAYMENT_OPTION".to_string(), "payment_option".to_string()), ("RESERVATION_MODIFIED".to_string(), "reservation_modified".to_string()), ("TAG_KEY".to_string(), "tag_key".to_string()), ("COST_CATEGORY_NAME".to_string(), "cost_category_name".to_string())], None, None)).with_provider_name("Key").build(),
                    StructField::optional("match_options", AttributeType::refined_list(AttributeType::enum_(carina_core::schema::enum_identity("MatchOptions", Some("aws.budgets.Budget.BudgetData.Expression.ExpressionDimensionValues")), Some(vec!["EQUALS".to_string(), "ABSENT".to_string(), "STARTS_WITH".to_string(), "ENDS_WITH".to_string(), "CONTAINS".to_string(), "GREATER_THAN_OR_EQUAL".to_string(), "CASE_SENSITIVE".to_string(), "CASE_INSENSITIVE".to_string()]), vec![("EQUALS".to_string(), "equals".to_string()), ("ABSENT".to_string(), "absent".to_string()), ("STARTS_WITH".to_string(), "starts_with".to_string()), ("ENDS_WITH".to_string(), "ends_with".to_string()), ("CONTAINS".to_string(), "contains".to_string()), ("GREATER_THAN_OR_EQUAL".to_string(), "greater_than_or_equal".to_string()), ("CASE_SENSITIVE".to_string(), "case_sensitive".to_string()), ("CASE_INSENSITIVE".to_string(), "case_insensitive".to_string())], None, None), true, Some((None, Some(1))), carina_core::schema::ListElements::AllowDuplicates)).with_provider_name("MatchOptions").build(),
                    StructField::optional("values", AttributeType::refined_list(AttributeType::string(), true, Some((Some(1), None)), carina_core::schema::ListElements::AllowDuplicates)).with_provider_name("Values").build()])).with_provider_name("Dimensions").build(),
                    StructField::optional("not", AttributeType::ref_("Expression".to_string())).with_provider_name("Not").build(),
                    StructField::optional("or", AttributeType::list(AttributeType::ref_("Expression".to_string()))).with_provider_name("Or").with_block_name("or").build(),
                    StructField::optional("tags", AttributeType::struct_("TagValues".to_string(), vec![StructField::optional("key", AttributeType::string()).with_provider_name("Key").build(),
                    StructField::optional("match_options", AttributeType::list(AttributeType::enum_(carina_core::schema::enum_identity("MatchOptions", Some("aws.budgets.Budget.BudgetData.Expression.TagValues")), Some(vec!["EQUALS".to_string(), "ABSENT".to_string(), "STARTS_WITH".to_string(), "ENDS_WITH".to_string(), "CONTAINS".to_string(), "GREATER_THAN_OR_EQUAL".to_string(), "CASE_SENSITIVE".to_string(), "CASE_INSENSITIVE".to_string()]), vec![("EQUALS".to_string(), "equals".to_string()), ("ABSENT".to_string(), "absent".to_string()), ("STARTS_WITH".to_string(), "starts_with".to_string()), ("ENDS_WITH".to_string(), "ends_with".to_string()), ("CONTAINS".to_string(), "contains".to_string()), ("GREATER_THAN_OR_EQUAL".to_string(), "greater_than_or_equal".to_string()), ("CASE_SENSITIVE".to_string(), "case_sensitive".to_string()), ("CASE_INSENSITIVE".to_string(), "case_insensitive".to_string())], None, None))).with_provider_name("MatchOptions").build(),
                    StructField::optional("values", AttributeType::refined_list(AttributeType::string(), true, Some((Some(1), None)), carina_core::schema::ListElements::AllowDuplicates)).with_provider_name("Values").build()])).with_provider_name("Tags").build()])).with_provider_name("FilterExpression").build(),
                    StructField::optional("metrics", AttributeType::refined_list(AttributeType::enum_(carina_core::schema::enum_identity("Metrics", Some("aws.budgets.Budget.BudgetData")), Some(vec!["BlendedCost".to_string(), "UnblendedCost".to_string(), "AmortizedCost".to_string(), "NetUnblendedCost".to_string(), "NetAmortizedCost".to_string(), "UsageQuantity".to_string(), "NormalizedUsageAmount".to_string(), "Hours".to_string()]), vec![("BlendedCost".to_string(), "blended_cost".to_string()), ("UnblendedCost".to_string(), "unblended_cost".to_string()), ("AmortizedCost".to_string(), "amortized_cost".to_string()), ("NetUnblendedCost".to_string(), "net_unblended_cost".to_string()), ("NetAmortizedCost".to_string(), "net_amortized_cost".to_string()), ("UsageQuantity".to_string(), "usage_quantity".to_string()), ("NormalizedUsageAmount".to_string(), "normalized_usage_amount".to_string()), ("Hours".to_string(), "hours".to_string())], None, None), true, Some((None, Some(1))), carina_core::schema::ListElements::AllowDuplicates)).with_provider_name("Metrics").build(),
                    StructField::optional("planned_budget_limits", AttributeType::map(AttributeType::ref_("Spend".to_string()))).with_provider_name("PlannedBudgetLimits").build(),
                    StructField::optional("time_period", AttributeType::struct_("TimePeriod".to_string(), vec![StructField::optional("end", AttributeType::string()).with_provider_name("End").build(),
                    StructField::optional("start", AttributeType::string()).with_provider_name("Start").build()])).with_provider_name("TimePeriod").build(),
                    StructField::required("time_unit", AttributeType::enum_(carina_core::schema::enum_identity("TimeUnit", Some("aws.budgets.Budget.BudgetData")), Some(vec!["DAILY".to_string(), "MONTHLY".to_string(), "QUARTERLY".to_string(), "ANNUALLY".to_string(), "CUSTOM".to_string()]), vec![("DAILY".to_string(), "daily".to_string()), ("MONTHLY".to_string(), "monthly".to_string()), ("QUARTERLY".to_string(), "quarterly".to_string()), ("ANNUALLY".to_string(), "annually".to_string()), ("CUSTOM".to_string(), "custom".to_string())], None, None)).with_provider_name("TimeUnit").build()]) }
    schema = schema.with_def("BudgetData", build_schema_def_1());
    #[inline(never)]
    fn build_schema_def_2() -> AttributeType { AttributeType::struct_("CostCategoryValues".to_string(), vec![StructField::optional("key", AttributeType::string()).with_provider_name("Key").build(),
                    StructField::optional("match_options", AttributeType::list(AttributeType::enum_(carina_core::schema::enum_identity("MatchOptions", Some("aws.budgets.Budget.BudgetData.Expression.CostCategoryValues")), Some(vec!["EQUALS".to_string(), "ABSENT".to_string(), "STARTS_WITH".to_string(), "ENDS_WITH".to_string(), "CONTAINS".to_string(), "GREATER_THAN_OR_EQUAL".to_string(), "CASE_SENSITIVE".to_string(), "CASE_INSENSITIVE".to_string()]), vec![("EQUALS".to_string(), "equals".to_string()), ("ABSENT".to_string(), "absent".to_string()), ("STARTS_WITH".to_string(), "starts_with".to_string()), ("ENDS_WITH".to_string(), "ends_with".to_string()), ("CONTAINS".to_string(), "contains".to_string()), ("GREATER_THAN_OR_EQUAL".to_string(), "greater_than_or_equal".to_string()), ("CASE_SENSITIVE".to_string(), "case_sensitive".to_string()), ("CASE_INSENSITIVE".to_string(), "case_insensitive".to_string())], None, None))).with_provider_name("MatchOptions").build(),
                    StructField::optional("values", AttributeType::refined_list(AttributeType::string(), true, Some((Some(1), None)), carina_core::schema::ListElements::AllowDuplicates)).with_provider_name("Values").build()]) }
    schema = schema.with_def("CostCategoryValues", build_schema_def_2());
    #[inline(never)]
    fn build_schema_def_3() -> AttributeType { AttributeType::struct_("CostTypes".to_string(), vec![StructField::optional("include_credit", AttributeType::bool()).with_provider_name("IncludeCredit").build(),
                    StructField::optional("include_discount", AttributeType::bool()).with_provider_name("IncludeDiscount").build(),
                    StructField::optional("include_other_subscription", AttributeType::bool()).with_provider_name("IncludeOtherSubscription").build(),
                    StructField::optional("include_recurring", AttributeType::bool()).with_provider_name("IncludeRecurring").build(),
                    StructField::optional("include_refund", AttributeType::bool()).with_provider_name("IncludeRefund").build(),
                    StructField::optional("include_subscription", AttributeType::bool()).with_provider_name("IncludeSubscription").build(),
                    StructField::optional("include_support", AttributeType::bool()).with_provider_name("IncludeSupport").build(),
                    StructField::optional("include_tax", AttributeType::bool()).with_provider_name("IncludeTax").build(),
                    StructField::optional("include_upfront", AttributeType::bool()).with_provider_name("IncludeUpfront").build(),
                    StructField::optional("use_amortized", AttributeType::bool()).with_provider_name("UseAmortized").build(),
                    StructField::optional("use_blended", AttributeType::bool()).with_provider_name("UseBlended").build()]) }
    schema = schema.with_def("CostTypes", build_schema_def_3());
    #[inline(never)]
    fn build_schema_def_4() -> AttributeType { AttributeType::struct_("Expression".to_string(), vec![StructField::optional("and", AttributeType::list(AttributeType::ref_("Expression".to_string()))).with_provider_name("And").with_block_name("and").build(),
                    StructField::optional("cost_categories", AttributeType::struct_("CostCategoryValues".to_string(), vec![StructField::optional("key", AttributeType::string()).with_provider_name("Key").build(),
                    StructField::optional("match_options", AttributeType::list(AttributeType::enum_(carina_core::schema::enum_identity("MatchOptions", Some("aws.budgets.Budget.BudgetData.Expression.CostCategoryValues")), Some(vec!["EQUALS".to_string(), "ABSENT".to_string(), "STARTS_WITH".to_string(), "ENDS_WITH".to_string(), "CONTAINS".to_string(), "GREATER_THAN_OR_EQUAL".to_string(), "CASE_SENSITIVE".to_string(), "CASE_INSENSITIVE".to_string()]), vec![("EQUALS".to_string(), "equals".to_string()), ("ABSENT".to_string(), "absent".to_string()), ("STARTS_WITH".to_string(), "starts_with".to_string()), ("ENDS_WITH".to_string(), "ends_with".to_string()), ("CONTAINS".to_string(), "contains".to_string()), ("GREATER_THAN_OR_EQUAL".to_string(), "greater_than_or_equal".to_string()), ("CASE_SENSITIVE".to_string(), "case_sensitive".to_string()), ("CASE_INSENSITIVE".to_string(), "case_insensitive".to_string())], None, None))).with_provider_name("MatchOptions").build(),
                    StructField::optional("values", AttributeType::refined_list(AttributeType::string(), true, Some((Some(1), None)), carina_core::schema::ListElements::AllowDuplicates)).with_provider_name("Values").build()])).with_provider_name("CostCategories").build(),
                    StructField::optional("dimensions", AttributeType::struct_("ExpressionDimensionValues".to_string(), vec![StructField::optional("key", AttributeType::enum_(carina_core::schema::enum_identity("Key", Some("aws.budgets.Budget.BudgetData.Expression.ExpressionDimensionValues")), Some(vec!["AZ".to_string(), "INSTANCE_TYPE".to_string(), "LINKED_ACCOUNT".to_string(), "LINKED_ACCOUNT_NAME".to_string(), "OPERATION".to_string(), "PURCHASE_TYPE".to_string(), "REGION".to_string(), "SERVICE".to_string(), "SERVICE_CODE".to_string(), "USAGE_TYPE".to_string(), "USAGE_TYPE_GROUP".to_string(), "RECORD_TYPE".to_string(), "OPERATING_SYSTEM".to_string(), "TENANCY".to_string(), "SCOPE".to_string(), "PLATFORM".to_string(), "SUBSCRIPTION_ID".to_string(), "LEGAL_ENTITY_NAME".to_string(), "INVOICING_ENTITY".to_string(), "DEPLOYMENT_OPTION".to_string(), "DATABASE_ENGINE".to_string(), "CACHE_ENGINE".to_string(), "INSTANCE_TYPE_FAMILY".to_string(), "BILLING_ENTITY".to_string(), "RESERVATION_ID".to_string(), "RESOURCE_ID".to_string(), "RIGHTSIZING_TYPE".to_string(), "SAVINGS_PLANS_TYPE".to_string(), "SAVINGS_PLAN_ARN".to_string(), "PAYMENT_OPTION".to_string(), "RESERVATION_MODIFIED".to_string(), "TAG_KEY".to_string(), "COST_CATEGORY_NAME".to_string()]), vec![("AZ".to_string(), "az".to_string()), ("INSTANCE_TYPE".to_string(), "instance_type".to_string()), ("LINKED_ACCOUNT".to_string(), "linked_account".to_string()), ("LINKED_ACCOUNT_NAME".to_string(), "linked_account_name".to_string()), ("OPERATION".to_string(), "operation".to_string()), ("PURCHASE_TYPE".to_string(), "purchase_type".to_string()), ("REGION".to_string(), "region".to_string()), ("SERVICE".to_string(), "service".to_string()), ("SERVICE_CODE".to_string(), "service_code".to_string()), ("USAGE_TYPE".to_string(), "usage_type".to_string()), ("USAGE_TYPE_GROUP".to_string(), "usage_type_group".to_string()), ("RECORD_TYPE".to_string(), "record_type".to_string()), ("OPERATING_SYSTEM".to_string(), "operating_system".to_string()), ("TENANCY".to_string(), "tenancy".to_string()), ("SCOPE".to_string(), "scope".to_string()), ("PLATFORM".to_string(), "platform".to_string()), ("SUBSCRIPTION_ID".to_string(), "subscription_id".to_string()), ("LEGAL_ENTITY_NAME".to_string(), "legal_entity_name".to_string()), ("INVOICING_ENTITY".to_string(), "invoicing_entity".to_string()), ("DEPLOYMENT_OPTION".to_string(), "deployment_option".to_string()), ("DATABASE_ENGINE".to_string(), "database_engine".to_string()), ("CACHE_ENGINE".to_string(), "cache_engine".to_string()), ("INSTANCE_TYPE_FAMILY".to_string(), "instance_type_family".to_string()), ("BILLING_ENTITY".to_string(), "billing_entity".to_string()), ("RESERVATION_ID".to_string(), "reservation_id".to_string()), ("RESOURCE_ID".to_string(), "resource_id".to_string()), ("RIGHTSIZING_TYPE".to_string(), "rightsizing_type".to_string()), ("SAVINGS_PLANS_TYPE".to_string(), "savings_plans_type".to_string()), ("SAVINGS_PLAN_ARN".to_string(), "savings_plan_arn".to_string()), ("PAYMENT_OPTION".to_string(), "payment_option".to_string()), ("RESERVATION_MODIFIED".to_string(), "reservation_modified".to_string()), ("TAG_KEY".to_string(), "tag_key".to_string()), ("COST_CATEGORY_NAME".to_string(), "cost_category_name".to_string())], None, None)).with_provider_name("Key").build(),
                    StructField::optional("match_options", AttributeType::refined_list(AttributeType::enum_(carina_core::schema::enum_identity("MatchOptions", Some("aws.budgets.Budget.BudgetData.Expression.ExpressionDimensionValues")), Some(vec!["EQUALS".to_string(), "ABSENT".to_string(), "STARTS_WITH".to_string(), "ENDS_WITH".to_string(), "CONTAINS".to_string(), "GREATER_THAN_OR_EQUAL".to_string(), "CASE_SENSITIVE".to_string(), "CASE_INSENSITIVE".to_string()]), vec![("EQUALS".to_string(), "equals".to_string()), ("ABSENT".to_string(), "absent".to_string()), ("STARTS_WITH".to_string(), "starts_with".to_string()), ("ENDS_WITH".to_string(), "ends_with".to_string()), ("CONTAINS".to_string(), "contains".to_string()), ("GREATER_THAN_OR_EQUAL".to_string(), "greater_than_or_equal".to_string()), ("CASE_SENSITIVE".to_string(), "case_sensitive".to_string()), ("CASE_INSENSITIVE".to_string(), "case_insensitive".to_string())], None, None), true, Some((None, Some(1))), carina_core::schema::ListElements::AllowDuplicates)).with_provider_name("MatchOptions").build(),
                    StructField::optional("values", AttributeType::refined_list(AttributeType::string(), true, Some((Some(1), None)), carina_core::schema::ListElements::AllowDuplicates)).with_provider_name("Values").build()])).with_provider_name("Dimensions").build(),
                    StructField::optional("not", AttributeType::ref_("Expression".to_string())).with_provider_name("Not").build(),
                    StructField::optional("or", AttributeType::list(AttributeType::ref_("Expression".to_string()))).with_provider_name("Or").with_block_name("or").build(),
                    StructField::optional("tags", AttributeType::struct_("TagValues".to_string(), vec![StructField::optional("key", AttributeType::string()).with_provider_name("Key").build(),
                    StructField::optional("match_options", AttributeType::list(AttributeType::enum_(carina_core::schema::enum_identity("MatchOptions", Some("aws.budgets.Budget.BudgetData.Expression.TagValues")), Some(vec!["EQUALS".to_string(), "ABSENT".to_string(), "STARTS_WITH".to_string(), "ENDS_WITH".to_string(), "CONTAINS".to_string(), "GREATER_THAN_OR_EQUAL".to_string(), "CASE_SENSITIVE".to_string(), "CASE_INSENSITIVE".to_string()]), vec![("EQUALS".to_string(), "equals".to_string()), ("ABSENT".to_string(), "absent".to_string()), ("STARTS_WITH".to_string(), "starts_with".to_string()), ("ENDS_WITH".to_string(), "ends_with".to_string()), ("CONTAINS".to_string(), "contains".to_string()), ("GREATER_THAN_OR_EQUAL".to_string(), "greater_than_or_equal".to_string()), ("CASE_SENSITIVE".to_string(), "case_sensitive".to_string()), ("CASE_INSENSITIVE".to_string(), "case_insensitive".to_string())], None, None))).with_provider_name("MatchOptions").build(),
                    StructField::optional("values", AttributeType::refined_list(AttributeType::string(), true, Some((Some(1), None)), carina_core::schema::ListElements::AllowDuplicates)).with_provider_name("Values").build()])).with_provider_name("Tags").build()]) }
    schema = schema.with_def("Expression", build_schema_def_4());
    #[inline(never)]
    fn build_schema_def_5() -> AttributeType { AttributeType::struct_("ExpressionDimensionValues".to_string(), vec![StructField::optional("key", AttributeType::enum_(carina_core::schema::enum_identity("Key", Some("aws.budgets.Budget.BudgetData.Expression.ExpressionDimensionValues")), Some(vec!["AZ".to_string(), "INSTANCE_TYPE".to_string(), "LINKED_ACCOUNT".to_string(), "LINKED_ACCOUNT_NAME".to_string(), "OPERATION".to_string(), "PURCHASE_TYPE".to_string(), "REGION".to_string(), "SERVICE".to_string(), "SERVICE_CODE".to_string(), "USAGE_TYPE".to_string(), "USAGE_TYPE_GROUP".to_string(), "RECORD_TYPE".to_string(), "OPERATING_SYSTEM".to_string(), "TENANCY".to_string(), "SCOPE".to_string(), "PLATFORM".to_string(), "SUBSCRIPTION_ID".to_string(), "LEGAL_ENTITY_NAME".to_string(), "INVOICING_ENTITY".to_string(), "DEPLOYMENT_OPTION".to_string(), "DATABASE_ENGINE".to_string(), "CACHE_ENGINE".to_string(), "INSTANCE_TYPE_FAMILY".to_string(), "BILLING_ENTITY".to_string(), "RESERVATION_ID".to_string(), "RESOURCE_ID".to_string(), "RIGHTSIZING_TYPE".to_string(), "SAVINGS_PLANS_TYPE".to_string(), "SAVINGS_PLAN_ARN".to_string(), "PAYMENT_OPTION".to_string(), "RESERVATION_MODIFIED".to_string(), "TAG_KEY".to_string(), "COST_CATEGORY_NAME".to_string()]), vec![("AZ".to_string(), "az".to_string()), ("INSTANCE_TYPE".to_string(), "instance_type".to_string()), ("LINKED_ACCOUNT".to_string(), "linked_account".to_string()), ("LINKED_ACCOUNT_NAME".to_string(), "linked_account_name".to_string()), ("OPERATION".to_string(), "operation".to_string()), ("PURCHASE_TYPE".to_string(), "purchase_type".to_string()), ("REGION".to_string(), "region".to_string()), ("SERVICE".to_string(), "service".to_string()), ("SERVICE_CODE".to_string(), "service_code".to_string()), ("USAGE_TYPE".to_string(), "usage_type".to_string()), ("USAGE_TYPE_GROUP".to_string(), "usage_type_group".to_string()), ("RECORD_TYPE".to_string(), "record_type".to_string()), ("OPERATING_SYSTEM".to_string(), "operating_system".to_string()), ("TENANCY".to_string(), "tenancy".to_string()), ("SCOPE".to_string(), "scope".to_string()), ("PLATFORM".to_string(), "platform".to_string()), ("SUBSCRIPTION_ID".to_string(), "subscription_id".to_string()), ("LEGAL_ENTITY_NAME".to_string(), "legal_entity_name".to_string()), ("INVOICING_ENTITY".to_string(), "invoicing_entity".to_string()), ("DEPLOYMENT_OPTION".to_string(), "deployment_option".to_string()), ("DATABASE_ENGINE".to_string(), "database_engine".to_string()), ("CACHE_ENGINE".to_string(), "cache_engine".to_string()), ("INSTANCE_TYPE_FAMILY".to_string(), "instance_type_family".to_string()), ("BILLING_ENTITY".to_string(), "billing_entity".to_string()), ("RESERVATION_ID".to_string(), "reservation_id".to_string()), ("RESOURCE_ID".to_string(), "resource_id".to_string()), ("RIGHTSIZING_TYPE".to_string(), "rightsizing_type".to_string()), ("SAVINGS_PLANS_TYPE".to_string(), "savings_plans_type".to_string()), ("SAVINGS_PLAN_ARN".to_string(), "savings_plan_arn".to_string()), ("PAYMENT_OPTION".to_string(), "payment_option".to_string()), ("RESERVATION_MODIFIED".to_string(), "reservation_modified".to_string()), ("TAG_KEY".to_string(), "tag_key".to_string()), ("COST_CATEGORY_NAME".to_string(), "cost_category_name".to_string())], None, None)).with_provider_name("Key").build(),
                    StructField::optional("match_options", AttributeType::refined_list(AttributeType::enum_(carina_core::schema::enum_identity("MatchOptions", Some("aws.budgets.Budget.BudgetData.Expression.ExpressionDimensionValues")), Some(vec!["EQUALS".to_string(), "ABSENT".to_string(), "STARTS_WITH".to_string(), "ENDS_WITH".to_string(), "CONTAINS".to_string(), "GREATER_THAN_OR_EQUAL".to_string(), "CASE_SENSITIVE".to_string(), "CASE_INSENSITIVE".to_string()]), vec![("EQUALS".to_string(), "equals".to_string()), ("ABSENT".to_string(), "absent".to_string()), ("STARTS_WITH".to_string(), "starts_with".to_string()), ("ENDS_WITH".to_string(), "ends_with".to_string()), ("CONTAINS".to_string(), "contains".to_string()), ("GREATER_THAN_OR_EQUAL".to_string(), "greater_than_or_equal".to_string()), ("CASE_SENSITIVE".to_string(), "case_sensitive".to_string()), ("CASE_INSENSITIVE".to_string(), "case_insensitive".to_string())], None, None), true, Some((None, Some(1))), carina_core::schema::ListElements::AllowDuplicates)).with_provider_name("MatchOptions").build(),
                    StructField::optional("values", AttributeType::refined_list(AttributeType::string(), true, Some((Some(1), None)), carina_core::schema::ListElements::AllowDuplicates)).with_provider_name("Values").build()]) }
    schema = schema.with_def("ExpressionDimensionValues", build_schema_def_5());
    #[inline(never)]
    fn build_schema_def_6() -> AttributeType { AttributeType::struct_("HistoricalOptions".to_string(), vec![StructField::required("budget_adjustment_period", AttributeType::refined_int(None, Some((Some(1), Some(60))))).with_provider_name("BudgetAdjustmentPeriod").build()]) }
    schema = schema.with_def("HistoricalOptions", build_schema_def_6());
    #[inline(never)]
    fn build_schema_def_7() -> AttributeType { AttributeType::struct_("Notification".to_string(), vec![StructField::required("comparison_operator", AttributeType::enum_(carina_core::schema::enum_identity("ComparisonOperator", Some("aws.budgets.Budget.NotificationWithSubscribers.Notification")), Some(vec!["GREATER_THAN".to_string(), "LESS_THAN".to_string(), "EQUAL_TO".to_string()]), vec![("GREATER_THAN".to_string(), "greater_than".to_string()), ("LESS_THAN".to_string(), "less_than".to_string()), ("EQUAL_TO".to_string(), "equal_to".to_string())], None, None)).with_provider_name("ComparisonOperator").build(),
                    StructField::required("notification_type", AttributeType::enum_(carina_core::schema::enum_identity("NotificationType", Some("aws.budgets.Budget.NotificationWithSubscribers.Notification")), Some(vec!["ACTUAL".to_string(), "FORECASTED".to_string()]), vec![("ACTUAL".to_string(), "actual".to_string()), ("FORECASTED".to_string(), "forecasted".to_string())], None, None)).with_provider_name("NotificationType").build(),
                    StructField::required("threshold", AttributeType::refined_float(None, Some((Some(0.0), Some(15000000000000.0))))).with_provider_name("Threshold").build(),
                    StructField::optional("threshold_type", AttributeType::enum_(carina_core::schema::enum_identity("ThresholdType", Some("aws.budgets.Budget.NotificationWithSubscribers.Notification")), Some(vec!["PERCENTAGE".to_string(), "ABSOLUTE_VALUE".to_string()]), vec![("PERCENTAGE".to_string(), "percentage".to_string()), ("ABSOLUTE_VALUE".to_string(), "absolute_value".to_string())], None, None)).with_provider_name("ThresholdType").build()]) }
    schema = schema.with_def("Notification", build_schema_def_7());
    #[inline(never)]
    fn build_schema_def_8() -> AttributeType { AttributeType::struct_("Spend".to_string(), vec![StructField::required("amount", AttributeType::refined_float(None, Some((Some(0.0), None)))).with_provider_name("Amount").build(),
                    StructField::required("unit", AttributeType::string()).with_provider_name("Unit").build()]) }
    schema = schema.with_def("Spend", build_schema_def_8());
    #[inline(never)]
    fn build_schema_def_9() -> AttributeType { AttributeType::struct_("TagValues".to_string(), vec![StructField::optional("key", AttributeType::string()).with_provider_name("Key").build(),
                    StructField::optional("match_options", AttributeType::list(AttributeType::enum_(carina_core::schema::enum_identity("MatchOptions", Some("aws.budgets.Budget.BudgetData.Expression.TagValues")), Some(vec!["EQUALS".to_string(), "ABSENT".to_string(), "STARTS_WITH".to_string(), "ENDS_WITH".to_string(), "CONTAINS".to_string(), "GREATER_THAN_OR_EQUAL".to_string(), "CASE_SENSITIVE".to_string(), "CASE_INSENSITIVE".to_string()]), vec![("EQUALS".to_string(), "equals".to_string()), ("ABSENT".to_string(), "absent".to_string()), ("STARTS_WITH".to_string(), "starts_with".to_string()), ("ENDS_WITH".to_string(), "ends_with".to_string()), ("CONTAINS".to_string(), "contains".to_string()), ("GREATER_THAN_OR_EQUAL".to_string(), "greater_than_or_equal".to_string()), ("CASE_SENSITIVE".to_string(), "case_sensitive".to_string()), ("CASE_INSENSITIVE".to_string(), "case_insensitive".to_string())], None, None))).with_provider_name("MatchOptions").build(),
                    StructField::optional("values", AttributeType::refined_list(AttributeType::string(), true, Some((Some(1), None)), carina_core::schema::ListElements::AllowDuplicates)).with_provider_name("Values").build()]) }
    schema = schema.with_def("TagValues", build_schema_def_9());
    #[inline(never)]
    fn build_schema_def_10() -> AttributeType { AttributeType::struct_("TimePeriod".to_string(), vec![StructField::optional("end", AttributeType::string()).with_provider_name("End").build(),
                    StructField::optional("start", AttributeType::string()).with_provider_name("Start").build()]) }
    schema = schema.with_def("TimePeriod", build_schema_def_10());
    AwsccSchemaConfig {
        aws_type_name: "AWS::Budgets::Budget",
        resource_type_name: "budgets.Budget",
        primary_identifier: &[crate::schemas::config::PrimaryIdentifierAttribute { provider_name: "Id", dsl_name: "id" }],
        exclusive_groups: &[],
        tags: None,
        await_on_read_back: vec![],
        schema: schema.validate().expect("generated schema must pass admission"),
    }
}

/// Returns the resource type name and all enum valid values for this module
pub fn enum_valid_values() -> (
    &'static str,
    &'static [(&'static str, &'static [&'static str])],
) {
    (
        "budgets.Budget",
        &[
            ("auto_adjust_type", VALID_AUTO_ADJUST_DATA_AUTO_ADJUST_TYPE),
            ("budget_type", VALID_BUDGET_DATA_BUDGET_TYPE),
            ("metrics", VALID_BUDGET_DATA_METRICS),
            ("time_unit", VALID_BUDGET_DATA_TIME_UNIT),
            ("match_options", VALID_COST_CATEGORY_VALUES_MATCH_OPTIONS),
            ("key", VALID_EXPRESSION_DIMENSION_VALUES_KEY),
            (
                "match_options",
                VALID_EXPRESSION_DIMENSION_VALUES_MATCH_OPTIONS,
            ),
            (
                "comparison_operator",
                VALID_NOTIFICATION_COMPARISON_OPERATOR,
            ),
            ("notification_type", VALID_NOTIFICATION_NOTIFICATION_TYPE),
            ("threshold_type", VALID_NOTIFICATION_THRESHOLD_TYPE),
            ("subscription_type", VALID_SUBSCRIBER_SUBSCRIPTION_TYPE),
            ("match_options", VALID_TAG_VALUES_MATCH_OPTIONS),
        ],
    )
}

/// Returns the IAM permissions declared by the CloudFormation handler for this operation.
pub fn required_permissions(op: carina_core::effect::PlanOp) -> &'static [&'static str] {
    match op {
        carina_core::effect::PlanOp::Create => &[
            "aws-portal:ModifyBilling",
            "budgets:ModifyBudget",
            "budgets:TagResource",
        ],
        carina_core::effect::PlanOp::Read => &[
            "aws-portal:ViewBilling",
            "budgets:ListTagsForResource",
            "budgets:ViewBudget",
        ],
        carina_core::effect::PlanOp::Update => &[
            "aws-portal:ModifyBilling",
            "aws-portal:ViewBilling",
            "budgets:ListTagsForResource",
            "budgets:ModifyBudget",
            "budgets:TagResource",
            "budgets:UntagResource",
        ],
        carina_core::effect::PlanOp::Delete => {
            &["aws-portal:ModifyBilling", "budgets:ModifyBudget"]
        }
    }
}
