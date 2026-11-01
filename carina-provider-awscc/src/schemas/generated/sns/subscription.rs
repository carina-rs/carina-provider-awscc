//! subscription schema definition for AWS Cloud Control
//!
//! Auto-generated from CloudFormation schema: AWS::SNS::Subscription
//!
//! DO NOT EDIT MANUALLY - regenerate with carina-codegen

use crate::schemas::config::AwsccSchemaConfig;
use carina_core::schema::{AttributeSchema, AttributeType, ResourceSchema};

pub fn arn() -> AttributeType {
    carina_aws_types::arn()
}

/// Returns the schema config for sns_subscription (AWS::SNS::Subscription)
#[rustfmt::skip]
pub fn sns_subscription_config() -> AwsccSchemaConfig {
    let mut schema = ResourceSchema::new("sns.Subscription")
        .with_description("Resource Type definition for AWS::SNS::Subscription");
    #[inline(never)]
    fn build_arn_attribute() -> AttributeSchema {
        AttributeSchema::provider_populated("arn", self::arn())
                .with_description("Arn of the subscription (read-only)")
                .with_provider_name("Arn")
            .build()
    }
    schema = schema.attribute(build_arn_attribute());
    #[inline(never)]
    fn build_delivery_policy_attribute() -> AttributeSchema {
        AttributeSchema::optional("delivery_policy", crate::schemas::awscc_types::sns_subscription_delivery_policy())
                .with_description("The delivery policy JSON assigned to the subscription. Enables the subscriber to define the message delivery retry strategy in the case of an HTTP/S endpoint subscribed to the topic.")
                .with_provider_name("DeliveryPolicy")
            .build()
    }
    schema = schema.attribute(build_delivery_policy_attribute());
    #[inline(never)]
    fn build_endpoint_attribute() -> AttributeSchema {
        AttributeSchema::optional("endpoint", AttributeType::string())
                .create_only()
                .with_description("The subscription's endpoint. The endpoint value depends on the protocol that you specify. ")
                .with_provider_name("Endpoint")
            .build()
    }
    schema = schema.attribute(build_endpoint_attribute());
    #[inline(never)]
    fn build_filter_policy_scope_attribute() -> AttributeSchema {
        AttributeSchema::optional("filter_policy_scope", AttributeType::string())
                .with_description("This attribute lets you choose the filtering scope by using one of the following string value types: MessageAttributes (default) and MessageBody.")
                .with_provider_name("FilterPolicyScope")
            .build()
    }
    schema = schema.attribute(build_filter_policy_scope_attribute());
    #[inline(never)]
    fn build_protocol_attribute() -> AttributeSchema {
        AttributeSchema::required("protocol", AttributeType::string())
                .create_only()
                .with_description("The subscription's protocol.")
                .with_provider_name("Protocol")
            .build()
    }
    schema = schema.attribute(build_protocol_attribute());
    #[inline(never)]
    fn build_raw_message_delivery_attribute() -> AttributeSchema {
        AttributeSchema::optional("raw_message_delivery", AttributeType::bool())
                .with_description("When set to true, enables raw message delivery. Raw messages don't contain any JSON formatting and can be sent to Amazon SQS and HTTP/S endpoints.")
                .with_provider_name("RawMessageDelivery")
            .build()
    }
    schema = schema.attribute(build_raw_message_delivery_attribute());
    #[inline(never)]
    fn build_redrive_policy_attribute() -> AttributeSchema {
        AttributeSchema::optional("redrive_policy", crate::schemas::awscc_types::sns_subscription_redrive_policy())
                .with_description("When specified, sends undeliverable messages to the specified Amazon SQS dead-letter queue. Messages that can't be delivered due to client errors are held in the dead-letter queue for further analysis or reprocessing.")
                .with_provider_name("RedrivePolicy")
            .build()
    }
    schema = schema.attribute(build_redrive_policy_attribute());
    #[inline(never)]
    fn build_region_attribute() -> AttributeSchema {
        AttributeSchema::optional("region", carina_aws_types::aws_region())
                .write_only()
                .with_description("For cross-region subscriptions, the region in which the topic resides.If no region is specified, AWS CloudFormation uses the region of the caller as the default.")
                .with_provider_name("Region")
            .build()
    }
    schema = schema.attribute(build_region_attribute());
    #[inline(never)]
    fn build_replay_policy_attribute() -> AttributeSchema {
        AttributeSchema::optional("replay_policy", crate::schemas::awscc_types::sns_subscription_replay_policy())
                .with_description("Specifies whether Amazon SNS resends the notification to the subscription when a message's attribute changes.")
                .with_provider_name("ReplayPolicy")
            .build()
    }
    schema = schema.attribute(build_replay_policy_attribute());
    #[inline(never)]
    fn build_subscription_role_arn_attribute() -> AttributeSchema {
        AttributeSchema::optional("subscription_role_arn", carina_aws_types::arn())
                .with_description("This property applies only to Amazon Data Firehose delivery stream subscriptions.")
                .with_provider_name("SubscriptionRoleArn")
            .build()
    }
    schema = schema.attribute(build_subscription_role_arn_attribute());
    #[inline(never)]
    fn build_topic_arn_attribute() -> AttributeSchema {
        AttributeSchema::required("topic_arn", carina_aws_types::arn())
                .create_only()
                .with_description("The ARN of the topic to subscribe to.")
                .with_provider_name("TopicArn")
            .build()
    }
    schema = schema.attribute(build_topic_arn_attribute());
    schema = schema
        .with_conflicting_replacement();
    AwsccSchemaConfig {
        aws_type_name: "AWS::SNS::Subscription",
        resource_type_name: "sns.Subscription",
        primary_identifier: &[crate::schemas::config::PrimaryIdentifierAttribute { provider_name: "Arn", dsl_name: "arn" }],
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
    ("sns.Subscription", &[])
}

/// Returns the IAM permissions declared by the CloudFormation handler for this operation.
pub fn required_permissions(op: carina_core::effect::PlanOp) -> &'static [&'static str] {
    match op {
        carina_core::effect::PlanOp::Create => &["iam:GetRole", "iam:PassRole", "sns:Subscribe"],
        carina_core::effect::PlanOp::Read => &["sns:GetSubscriptionAttributes"],
        carina_core::effect::PlanOp::Update => &[
            "iam:GetRole",
            "iam:PassRole",
            "sns:SetSubscriptionAttributes",
        ],
        carina_core::effect::PlanOp::Delete => {
            &["sns:Unsubscribe", "sns:GetSubscriptionAttributes"]
        }
    }
}
