//! budget schema definition for AWS Cloud Control
//!
//! Auto-generated from CloudFormation schema: AWS::Budgets::Budget
//!
//! DO NOT EDIT MANUALLY - regenerate with carina-codegen

use crate::schemas::config::AwsccSchemaConfig;
use carina_core::schema::{AttributeSchema, AttributeType, ResourceSchema, StructField};

/// Returns the schema config for budgets_budget (AWS::Budgets::Budget)
#[rustfmt::skip]
pub fn budgets_budget_config() -> AwsccSchemaConfig {
    let mut schema = ResourceSchema::new("budgets.Budget")
        .with_description("Resource Type definition for AWS::Budgets::Budget");
    #[inline(never)]
    fn build_budget_attribute() -> AttributeSchema {
        AttributeSchema::required("budget", AttributeType::struct_("BudgetData".to_string(), vec![StructField::optional("auto_adjust_data", AttributeType::struct_("AutoAdjustData".to_string(), vec![StructField::required("auto_adjust_type", AttributeType::string()).with_provider_name("AutoAdjustType").build(),
                    StructField::optional("historical_options", AttributeType::struct_("HistoricalOptions".to_string(), vec![StructField::required("budget_adjustment_period", AttributeType::int()).with_provider_name("BudgetAdjustmentPeriod").build()])).with_provider_name("HistoricalOptions").build()])).with_provider_name("AutoAdjustData").build(),
                    StructField::optional("billing_view_arn", carina_aws_types::arn()).with_provider_name("BillingViewArn").build(),
                    StructField::optional("budget_limit", AttributeType::struct_("Spend".to_string(), vec![StructField::required("amount", AttributeType::float()).with_provider_name("Amount").build(),
                    StructField::required("unit", AttributeType::string()).with_provider_name("Unit").build()])).with_provider_name("BudgetLimit").build(),
                    StructField::optional("budget_name", AttributeType::string()).with_provider_name("BudgetName").build(),
                    StructField::required("budget_type", AttributeType::string()).with_provider_name("BudgetType").build(),
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
                    StructField::optional("match_options", AttributeType::list(AttributeType::string())).with_provider_name("MatchOptions").build(),
                    StructField::optional("values", AttributeType::list(AttributeType::string())).with_provider_name("Values").build()])).with_provider_name("CostCategories").build(),
                    StructField::optional("dimensions", AttributeType::struct_("ExpressionDimensionValues".to_string(), vec![StructField::optional("key", AttributeType::string()).with_provider_name("Key").build(),
                    StructField::optional("match_options", AttributeType::list(AttributeType::string())).with_provider_name("MatchOptions").build(),
                    StructField::optional("values", AttributeType::list(AttributeType::string())).with_provider_name("Values").build()])).with_provider_name("Dimensions").build(),
                    StructField::optional("not", AttributeType::ref_("Expression".to_string())).with_provider_name("Not").build(),
                    StructField::optional("or", AttributeType::list(AttributeType::ref_("Expression".to_string()))).with_provider_name("Or").with_block_name("or").build(),
                    StructField::optional("tags", AttributeType::struct_("TagValues".to_string(), vec![StructField::optional("key", AttributeType::string()).with_provider_name("Key").build(),
                    StructField::optional("match_options", AttributeType::list(AttributeType::string())).with_provider_name("MatchOptions").build(),
                    StructField::optional("values", AttributeType::list(AttributeType::string())).with_provider_name("Values").build()])).with_provider_name("Tags").build()])).with_provider_name("FilterExpression").build(),
                    StructField::optional("metrics", AttributeType::list(AttributeType::string())).with_provider_name("Metrics").build(),
                    StructField::optional("planned_budget_limits", AttributeType::map(AttributeType::ref_("Spend".to_string()))).with_provider_name("PlannedBudgetLimits").build(),
                    StructField::optional("time_period", AttributeType::struct_("TimePeriod".to_string(), vec![StructField::optional("end", AttributeType::string()).with_provider_name("End").build(),
                    StructField::optional("start", AttributeType::string()).with_provider_name("Start").build()])).with_provider_name("TimePeriod").build(),
                    StructField::required("time_unit", AttributeType::string()).with_provider_name("TimeUnit").build()]))
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
        AttributeSchema::optional("notifications_with_subscribers", AttributeType::list(AttributeType::struct_("NotificationWithSubscribers".to_string(), vec![StructField::required("notification", AttributeType::struct_("Notification".to_string(), vec![StructField::required("comparison_operator", AttributeType::string()).with_provider_name("ComparisonOperator").build(),
                    StructField::required("notification_type", AttributeType::string()).with_provider_name("NotificationType").build(),
                    StructField::required("threshold", AttributeType::float()).with_provider_name("Threshold").build(),
                    StructField::optional("threshold_type", AttributeType::string()).with_provider_name("ThresholdType").build()])).with_provider_name("Notification").build(),
                    StructField::required("subscribers", AttributeType::list(AttributeType::struct_("Subscriber".to_string(), vec![StructField::required("address", AttributeType::string()).with_provider_name("Address").build(),
                    StructField::required("subscription_type", AttributeType::string()).with_provider_name("SubscriptionType").build()]))).with_provider_name("Subscribers").with_block_name("subscriber").build()])))
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
    fn build_schema_def_0() -> AttributeType { AttributeType::struct_("AutoAdjustData".to_string(), vec![StructField::required("auto_adjust_type", AttributeType::string()).with_provider_name("AutoAdjustType").build(),
                    StructField::optional("historical_options", AttributeType::struct_("HistoricalOptions".to_string(), vec![StructField::required("budget_adjustment_period", AttributeType::int()).with_provider_name("BudgetAdjustmentPeriod").build()])).with_provider_name("HistoricalOptions").build()]) }
    schema = schema.with_def("AutoAdjustData", build_schema_def_0());
    #[inline(never)]
    fn build_schema_def_1() -> AttributeType { AttributeType::struct_("BudgetData".to_string(), vec![StructField::optional("auto_adjust_data", AttributeType::struct_("AutoAdjustData".to_string(), vec![StructField::required("auto_adjust_type", AttributeType::string()).with_provider_name("AutoAdjustType").build(),
                    StructField::optional("historical_options", AttributeType::struct_("HistoricalOptions".to_string(), vec![StructField::required("budget_adjustment_period", AttributeType::int()).with_provider_name("BudgetAdjustmentPeriod").build()])).with_provider_name("HistoricalOptions").build()])).with_provider_name("AutoAdjustData").build(),
                    StructField::optional("billing_view_arn", carina_aws_types::arn()).with_provider_name("BillingViewArn").build(),
                    StructField::optional("budget_limit", AttributeType::struct_("Spend".to_string(), vec![StructField::required("amount", AttributeType::float()).with_provider_name("Amount").build(),
                    StructField::required("unit", AttributeType::string()).with_provider_name("Unit").build()])).with_provider_name("BudgetLimit").build(),
                    StructField::optional("budget_name", AttributeType::string()).with_provider_name("BudgetName").build(),
                    StructField::required("budget_type", AttributeType::string()).with_provider_name("BudgetType").build(),
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
                    StructField::optional("match_options", AttributeType::list(AttributeType::string())).with_provider_name("MatchOptions").build(),
                    StructField::optional("values", AttributeType::list(AttributeType::string())).with_provider_name("Values").build()])).with_provider_name("CostCategories").build(),
                    StructField::optional("dimensions", AttributeType::struct_("ExpressionDimensionValues".to_string(), vec![StructField::optional("key", AttributeType::string()).with_provider_name("Key").build(),
                    StructField::optional("match_options", AttributeType::list(AttributeType::string())).with_provider_name("MatchOptions").build(),
                    StructField::optional("values", AttributeType::list(AttributeType::string())).with_provider_name("Values").build()])).with_provider_name("Dimensions").build(),
                    StructField::optional("not", AttributeType::ref_("Expression".to_string())).with_provider_name("Not").build(),
                    StructField::optional("or", AttributeType::list(AttributeType::ref_("Expression".to_string()))).with_provider_name("Or").with_block_name("or").build(),
                    StructField::optional("tags", AttributeType::struct_("TagValues".to_string(), vec![StructField::optional("key", AttributeType::string()).with_provider_name("Key").build(),
                    StructField::optional("match_options", AttributeType::list(AttributeType::string())).with_provider_name("MatchOptions").build(),
                    StructField::optional("values", AttributeType::list(AttributeType::string())).with_provider_name("Values").build()])).with_provider_name("Tags").build()])).with_provider_name("FilterExpression").build(),
                    StructField::optional("metrics", AttributeType::list(AttributeType::string())).with_provider_name("Metrics").build(),
                    StructField::optional("planned_budget_limits", AttributeType::map(AttributeType::ref_("Spend".to_string()))).with_provider_name("PlannedBudgetLimits").build(),
                    StructField::optional("time_period", AttributeType::struct_("TimePeriod".to_string(), vec![StructField::optional("end", AttributeType::string()).with_provider_name("End").build(),
                    StructField::optional("start", AttributeType::string()).with_provider_name("Start").build()])).with_provider_name("TimePeriod").build(),
                    StructField::required("time_unit", AttributeType::string()).with_provider_name("TimeUnit").build()]) }
    schema = schema.with_def("BudgetData", build_schema_def_1());
    #[inline(never)]
    fn build_schema_def_2() -> AttributeType { AttributeType::struct_("CostCategoryValues".to_string(), vec![StructField::optional("key", AttributeType::string()).with_provider_name("Key").build(),
                    StructField::optional("match_options", AttributeType::list(AttributeType::string())).with_provider_name("MatchOptions").build(),
                    StructField::optional("values", AttributeType::list(AttributeType::string())).with_provider_name("Values").build()]) }
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
                    StructField::optional("match_options", AttributeType::list(AttributeType::string())).with_provider_name("MatchOptions").build(),
                    StructField::optional("values", AttributeType::list(AttributeType::string())).with_provider_name("Values").build()])).with_provider_name("CostCategories").build(),
                    StructField::optional("dimensions", AttributeType::struct_("ExpressionDimensionValues".to_string(), vec![StructField::optional("key", AttributeType::string()).with_provider_name("Key").build(),
                    StructField::optional("match_options", AttributeType::list(AttributeType::string())).with_provider_name("MatchOptions").build(),
                    StructField::optional("values", AttributeType::list(AttributeType::string())).with_provider_name("Values").build()])).with_provider_name("Dimensions").build(),
                    StructField::optional("not", AttributeType::ref_("Expression".to_string())).with_provider_name("Not").build(),
                    StructField::optional("or", AttributeType::list(AttributeType::ref_("Expression".to_string()))).with_provider_name("Or").with_block_name("or").build(),
                    StructField::optional("tags", AttributeType::struct_("TagValues".to_string(), vec![StructField::optional("key", AttributeType::string()).with_provider_name("Key").build(),
                    StructField::optional("match_options", AttributeType::list(AttributeType::string())).with_provider_name("MatchOptions").build(),
                    StructField::optional("values", AttributeType::list(AttributeType::string())).with_provider_name("Values").build()])).with_provider_name("Tags").build()]) }
    schema = schema.with_def("Expression", build_schema_def_4());
    #[inline(never)]
    fn build_schema_def_5() -> AttributeType { AttributeType::struct_("ExpressionDimensionValues".to_string(), vec![StructField::optional("key", AttributeType::string()).with_provider_name("Key").build(),
                    StructField::optional("match_options", AttributeType::list(AttributeType::string())).with_provider_name("MatchOptions").build(),
                    StructField::optional("values", AttributeType::list(AttributeType::string())).with_provider_name("Values").build()]) }
    schema = schema.with_def("ExpressionDimensionValues", build_schema_def_5());
    #[inline(never)]
    fn build_schema_def_6() -> AttributeType { AttributeType::struct_("HistoricalOptions".to_string(), vec![StructField::required("budget_adjustment_period", AttributeType::int()).with_provider_name("BudgetAdjustmentPeriod").build()]) }
    schema = schema.with_def("HistoricalOptions", build_schema_def_6());
    #[inline(never)]
    fn build_schema_def_7() -> AttributeType { AttributeType::struct_("Notification".to_string(), vec![StructField::required("comparison_operator", AttributeType::string()).with_provider_name("ComparisonOperator").build(),
                    StructField::required("notification_type", AttributeType::string()).with_provider_name("NotificationType").build(),
                    StructField::required("threshold", AttributeType::float()).with_provider_name("Threshold").build(),
                    StructField::optional("threshold_type", AttributeType::string()).with_provider_name("ThresholdType").build()]) }
    schema = schema.with_def("Notification", build_schema_def_7());
    #[inline(never)]
    fn build_schema_def_8() -> AttributeType { AttributeType::struct_("Spend".to_string(), vec![StructField::required("amount", AttributeType::float()).with_provider_name("Amount").build(),
                    StructField::required("unit", AttributeType::string()).with_provider_name("Unit").build()]) }
    schema = schema.with_def("Spend", build_schema_def_8());
    #[inline(never)]
    fn build_schema_def_9() -> AttributeType { AttributeType::struct_("TagValues".to_string(), vec![StructField::optional("key", AttributeType::string()).with_provider_name("Key").build(),
                    StructField::optional("match_options", AttributeType::list(AttributeType::string())).with_provider_name("MatchOptions").build(),
                    StructField::optional("values", AttributeType::list(AttributeType::string())).with_provider_name("Values").build()]) }
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
        schema,
    }
}

/// Returns the resource type name and all enum valid values for this module
pub fn enum_valid_values() -> (
    &'static str,
    &'static [(&'static str, &'static [&'static str])],
) {
    ("budgets.Budget", &[])
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
