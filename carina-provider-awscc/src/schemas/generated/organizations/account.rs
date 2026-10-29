//! account schema definition for AWS Cloud Control
//!
//! Auto-generated from CloudFormation schema: AWS::Organizations::Account
//!
//! DO NOT EDIT MANUALLY - regenerate with carina-codegen

use crate::schemas::config::{AwsccSchemaConfig, TagsProperty, TagsWire};
use carina_core::resource::{ConcreteValue, Value};
use carina_core::schema::{AttributeSchema, AttributeType, ResourceSchema, types};

pub fn arn() -> AttributeType {
    crate::schemas::catalog::organizations_account_arn()
}

const VALID_JOINED_METHOD: &[&str] = &["INVITED", "CREATED"];

const VALID_STATE: &[&str] = &[
    "PENDING_ACTIVATION",
    "ACTIVE",
    "SUSPENDED",
    "PENDING_CLOSURE",
    "CLOSED",
];

const VALID_STATUS: &[&str] = &["ACTIVE", "SUSPENDED", "PENDING_CLOSURE"];

/// Returns the schema config for organizations_account (AWS::Organizations::Account)
#[rustfmt::skip]
pub fn organizations_account_config() -> AwsccSchemaConfig {
    let mut schema = ResourceSchema::new("organizations.Account")
        .with_description("You can use AWS::Organizations::Account to manage accounts in organization.");
    #[inline(never)]
    fn build_account_id_attribute() -> AttributeSchema {
        AttributeSchema::provider_populated("account_id", carina_aws_types::aws_account_id())
                .with_description("If the account was created successfully, the unique identifier (ID) of the new account. (read-only)")
                .with_provider_name("AccountId")
            .build()
    }
    schema = schema.attribute(build_account_id_attribute());
    #[inline(never)]
    fn build_account_name_attribute() -> AttributeSchema {
        AttributeSchema::required("account_name", AttributeType::refined_string(None, Some("[\\u0020-\\u007E]+".to_string()), Some((Some(1), Some(50))), None))
                .with_description("The friendly name of the member account.")
                .with_provider_name("AccountName")
            .build()
    }
    schema = schema.attribute(build_account_name_attribute());
    #[inline(never)]
    fn build_arn_attribute() -> AttributeSchema {
        AttributeSchema::provider_populated("arn", self::arn())
                .with_description("The Amazon Resource Name (ARN) of the account. (read-only)")
                .with_provider_name("Arn")
            .build()
    }
    schema = schema.attribute(build_arn_attribute());
    #[inline(never)]
    fn build_email_attribute() -> AttributeSchema {
        AttributeSchema::required("email", types::email())
                .with_description("The email address of the owner to assign to the new member account.")
                .with_provider_name("Email")
            .build()
    }
    schema = schema.attribute(build_email_attribute());
    #[inline(never)]
    fn build_joined_method_attribute() -> AttributeSchema {
        AttributeSchema::provider_populated("joined_method", AttributeType::enum_(carina_core::schema::enum_identity("JoinedMethod", Some("aws.organizations.Account")), Some(vec!["INVITED".to_string(), "CREATED".to_string()]), vec![("INVITED".to_string(), "invited".to_string()), ("CREATED".to_string(), "created".to_string())], None, None))
                .with_description("The method by which the account joined the organization. (read-only)")
                .with_provider_name("JoinedMethod")
            .build()
    }
    schema = schema.attribute(build_joined_method_attribute());
    #[inline(never)]
    fn build_joined_timestamp_attribute() -> AttributeSchema {
        AttributeSchema::provider_populated("joined_timestamp", AttributeType::string())
                .with_description("The date the account became a part of the organization. (read-only)")
                .with_provider_name("JoinedTimestamp")
            .build()
    }
    schema = schema.attribute(build_joined_timestamp_attribute());
    #[inline(never)]
    fn build_parent_ids_attribute() -> AttributeSchema {
        AttributeSchema::optional("parent_ids", AttributeType::unordered_list(AttributeType::refined_string(None, Some("^(r-[0-9a-z]{4,32})|(ou-[0-9a-z]{4,32}-[a-z0-9]{8,32})$".to_string()), None, None)))
                .with_description("List of parent nodes for the member account. Currently only one parent at a time is supported. Default is root.")
                .with_provider_name("ParentIds")
            .build()
    }
    schema = schema.attribute(build_parent_ids_attribute());
    #[inline(never)]
    fn build_paths_attribute() -> AttributeSchema {
        AttributeSchema::provider_populated("paths", AttributeType::list(AttributeType::refined_string(None, Some("^(o-[a-z0-9]{10,32}/r-[0-9a-z]{4,32}(/ou-[0-9a-z]{4,32}-[a-z0-9]{8,32})*(/\\d{12})*)/".to_string()), None, None)))
                .with_description("The paths in the organization where the account exists. (read-only)")
                .with_provider_name("Paths")
            .build()
    }
    schema = schema.attribute(build_paths_attribute());
    #[inline(never)]
    fn build_role_name_attribute() -> AttributeSchema {
        AttributeSchema::optional("role_name", AttributeType::refined_string(None, Some("[\\w+=,.@-]{1,64}".to_string()), Some((Some(1), Some(64))), None))
                .write_only()
                .with_description("The name of an IAM role that AWS Organizations automatically preconfigures in the new member account. Default name is OrganizationAccountAccessRole if not specified.")
                .with_provider_name("RoleName")
                .with_default(Value::Concrete(ConcreteValue::String("OrganizationAccountAccessRole".to_string().into())))
            .build()
    }
    schema = schema.attribute(build_role_name_attribute());
    #[inline(never)]
    fn build_state_attribute() -> AttributeSchema {
        AttributeSchema::provider_populated("state", AttributeType::enum_(carina_core::schema::enum_identity("State", Some("aws.organizations.Account")), Some(vec!["PENDING_ACTIVATION".to_string(), "ACTIVE".to_string(), "SUSPENDED".to_string(), "PENDING_CLOSURE".to_string(), "CLOSED".to_string()]), vec![("PENDING_ACTIVATION".to_string(), "pending_activation".to_string()), ("ACTIVE".to_string(), "active".to_string()), ("SUSPENDED".to_string(), "suspended".to_string()), ("PENDING_CLOSURE".to_string(), "pending_closure".to_string()), ("CLOSED".to_string(), "closed".to_string())], None, None))
                .with_description("The state of the account in the organization. (read-only)")
                .with_provider_name("State")
            .build()
    }
    schema = schema.attribute(build_state_attribute());
    #[inline(never)]
    fn build_status_attribute() -> AttributeSchema {
        AttributeSchema::provider_populated("status", AttributeType::enum_(carina_core::schema::enum_identity("Status", Some("aws.organizations.Account")), Some(vec!["ACTIVE".to_string(), "SUSPENDED".to_string(), "PENDING_CLOSURE".to_string()]), vec![("ACTIVE".to_string(), "active".to_string()), ("SUSPENDED".to_string(), "suspended".to_string()), ("PENDING_CLOSURE".to_string(), "pending_closure".to_string())], None, None))
                .with_description("The status of the account in the organization. (read-only)")
                .with_provider_name("Status")
            .build()
    }
    schema = schema.attribute(build_status_attribute());
    #[inline(never)]
    fn build_tags_attribute() -> AttributeSchema {
        AttributeSchema::optional("tags", carina_aws_types::tags_type())
                .with_description("A list of tags that you want to attach to the newly created account. For each tag in the list, you must specify both a tag key and a value.")
                .with_provider_name("Tags")
                .with_block_name("tag")
            .build()
    }
    schema = schema.attribute(build_tags_attribute());
    schema = schema
        .with_conflicting_replacement();
    schema = schema.with_declared_validators(vec![carina_provider_protocol::types::ValidatorType::TagsKeyValueCheck]);
    AwsccSchemaConfig {
        aws_type_name: "AWS::Organizations::Account",
        resource_type_name: "organizations.Account",
        primary_identifier: &[crate::schemas::config::PrimaryIdentifierAttribute { provider_name: "AccountId", dsl_name: "account_id" }],
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
        "organizations.Account",
        &[
            ("joined_method", VALID_JOINED_METHOD),
            ("state", VALID_STATE),
            ("status", VALID_STATUS),
        ],
    )
}

/// Returns the IAM permissions declared by the CloudFormation handler for this operation.
pub fn required_permissions(op: carina_core::effect::PlanOp) -> &'static [&'static str] {
    match op {
        carina_core::effect::PlanOp::Create => &[
            "organizations:CreateAccount",
            "organizations:DescribeCreateAccountStatus",
            "organizations:MoveAccount",
            "organizations:ListAccounts",
            "organizations:ListParents",
            "organizations:TagResource",
            "organizations:DescribeAccount",
            "organizations:ListTagsForResource",
        ],
        carina_core::effect::PlanOp::Read => &[
            "organizations:DescribeAccount",
            "organizations:ListParents",
            "organizations:ListTagsForResource",
        ],
        carina_core::effect::PlanOp::Update => &[
            "organizations:MoveAccount",
            "organizations:TagResource",
            "organizations:UntagResource",
            "organizations:ListRoots",
            "organizations:DescribeAccount",
            "organizations:ListParents",
            "organizations:ListTagsForResource",
        ],
        carina_core::effect::PlanOp::Delete => &["organizations:CloseAccount"],
    }
}
