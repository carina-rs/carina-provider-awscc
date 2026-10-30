//! schedule_group schema definition for AWS Cloud Control
//!
//! Auto-generated from CloudFormation schema: AWS::Scheduler::ScheduleGroup
//!
//! DO NOT EDIT MANUALLY - regenerate with carina-codegen

use crate::schemas::config::{AwsccSchemaConfig, TagsProperty, TagsWire};
use carina_core::schema::{AttributeSchema, AttributeType, ResourceSchema};

pub fn arn() -> AttributeType {
    carina_aws_types::arn()
}

const VALID_STATE: &[&str] = &["ACTIVE", "DELETING"];

/// Returns the schema config for scheduler_schedule_group (AWS::Scheduler::ScheduleGroup)
#[rustfmt::skip]
pub fn scheduler_schedule_group_config() -> AwsccSchemaConfig {
    let mut schema = ResourceSchema::new("scheduler.ScheduleGroup")
        .with_description("Definition of AWS::Scheduler::ScheduleGroup Resource Type");
    #[inline(never)]
    fn build_arn_attribute() -> AttributeSchema {
        AttributeSchema::provider_populated("arn", self::arn())
                .with_description("The Amazon Resource Name (ARN) of the schedule group. (read-only)")
                .with_provider_name("Arn")
            .build()
    }
    schema = schema.attribute(build_arn_attribute());
    #[inline(never)]
    fn build_creation_date_attribute() -> AttributeSchema {
        AttributeSchema::provider_populated("creation_date", AttributeType::refined_string(None, None, None, None))
                .with_description("The time at which the schedule group was created. (read-only)")
                .with_provider_name("CreationDate")
            .build()
    }
    schema = schema.attribute(build_creation_date_attribute());
    #[inline(never)]
    fn build_last_modification_date_attribute() -> AttributeSchema {
        AttributeSchema::provider_populated("last_modification_date", AttributeType::refined_string(None, None, None, None))
                .with_description("The time at which the schedule group was last modified. (read-only)")
                .with_provider_name("LastModificationDate")
            .build()
    }
    schema = schema.attribute(build_last_modification_date_attribute());
    #[inline(never)]
    fn build_name_attribute() -> AttributeSchema {
        AttributeSchema::optional("name", AttributeType::refined_string(None, Some("^[0-9a-zA-Z-_.]+$".to_string()), Some((Some(1), Some(64))), None))
                .create_only()
                .with_provider_name("Name")
            .build()
    }
    schema = schema.attribute(build_name_attribute());
    #[inline(never)]
    fn build_state_attribute() -> AttributeSchema {
        AttributeSchema::provider_populated("state", AttributeType::enum_(carina_core::schema::enum_identity("ScheduleGroupState", Some("aws.scheduler.ScheduleGroup")), Some(vec!["ACTIVE".to_string(), "DELETING".to_string()]), vec![("ACTIVE".to_string(), "active".to_string()), ("DELETING".to_string(), "deleting".to_string())], None, None))
                .with_provider_name("State")
            .build()
    }
    schema = schema.attribute(build_state_attribute());
    #[inline(never)]
    fn build_tags_attribute() -> AttributeSchema {
        AttributeSchema::optional("tags", carina_aws_types::tags_type())
                .with_description("The list of tags to associate with the schedule group.")
                .with_provider_name("Tags")
                .with_block_name("tag")
            .build()
    }
    schema = schema.attribute(build_tags_attribute());
    schema = schema
        .with_unique_name_attribute("name");
    schema = schema.with_declared_validators(vec![carina_provider_protocol::types::ValidatorType::TagsKeyValueCheck]);
    AwsccSchemaConfig {
        aws_type_name: "AWS::Scheduler::ScheduleGroup",
        resource_type_name: "scheduler.ScheduleGroup",
        primary_identifier: &[crate::schemas::config::PrimaryIdentifierAttribute { provider_name: "Name", dsl_name: "name" }],
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
    ("scheduler.ScheduleGroup", &[("state", VALID_STATE)])
}

/// Returns the IAM permissions declared by the CloudFormation handler for this operation.
pub fn required_permissions(op: carina_core::effect::PlanOp) -> &'static [&'static str] {
    match op {
        carina_core::effect::PlanOp::Create => &[
            "scheduler:TagResource",
            "scheduler:CreateScheduleGroup",
            "scheduler:GetScheduleGroup",
            "scheduler:ListTagsForResource",
        ],
        carina_core::effect::PlanOp::Read => &[
            "scheduler:GetScheduleGroup",
            "scheduler:ListTagsForResource",
        ],
        carina_core::effect::PlanOp::Update => &[
            "scheduler:TagResource",
            "scheduler:UntagResource",
            "scheduler:ListTagsForResource",
            "scheduler:GetScheduleGroup",
        ],
        carina_core::effect::PlanOp::Delete => &[
            "scheduler:DeleteScheduleGroup",
            "scheduler:GetScheduleGroup",
            "scheduler:DeleteSchedule",
        ],
    }
}
