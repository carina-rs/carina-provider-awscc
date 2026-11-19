//! Tests ported from carina-aws-types 5ad417e; assertions and vectors retained.
//! See tests/fixtures/ported_schema_tests_5ad417e.tsv for every upstream test
//! and exclusions where AWSCC already had a different implementation at base.
use super::*;
use crate::schemas::type_helpers::condition_operator::{
    ConditionOperator, ConditionOperatorBase, ConditionQualifier,
};
use crate::schemas::type_helpers::kms::{
    kms_key_arn_pattern, kms_key_reference_pattern, validate_kms_key_arn,
    validate_kms_key_reference,
};
use crate::schemas::type_helpers::policy_seed::{
    condition_type, iam_policy_document, iam_policy_effect,
};
use crate::schemas::type_helpers::*;
use carina_core::schema::{
    AttributeSchema, CustomValidator, DslTransform, ResourceSchema, RootlessType,
    ValidatedConfigSchema,
};
use std::collections::BTreeMap;
const KEY_IDS: [&str; 4] = [
    "1234abcd-12ab-34cd-56ef-1234567890ab",
    "1234ABCD-12AB-34CD-56EF-1234567890AB",
    "mrk-0123456789abcdef0123456789abcdef",
    "mrk-0123456789ABCDEF0123456789ABCDEF",
];
pub(crate) struct TestRootlessType(RootlessType);

impl TestRootlessType {
    pub(crate) fn validate(&self, value: &Value) -> Result<(), carina_core::schema::TypeError> {
        self.0.validate(value)
    }

    pub(crate) fn validate_collect(
        &self,
        value: &Value,
    ) -> Vec<(
        carina_core::schema::FieldPath,
        carina_core::schema::TypeError,
    )> {
        self.0.validate_collect(value)
    }
}

pub(crate) fn rootless(attr: AttributeType) -> TestRootlessType {
    const NAME: &str = "test_value";
    TestRootlessType(
        ValidatedConfigSchema::validate(BTreeMap::from([(NAME.to_string(), attr)]))
            .expect("test attribute type must pass config-schema admission")
            .rootless_type(NAME)
            .expect("test attribute type must be present"),
    )
}

fn field_type<'a>(attr: &'a AttributeType, field_name: &str) -> &'a AttributeType {
    let mut budget = carina_core::schema::ShapeWalkBudget::new(8);
    let Some(fields) = attr
        .struct_fields_ref_free_with_budget(&mut budget)
        .expect("test schema is Ref-free")
    else {
        panic!("expected Struct shape");
    };
    fields
        .iter()
        .find(|field| field.name() == field_name)
        .unwrap_or_else(|| panic!("missing field {field_name}"))
        .field_type()
}

fn list_inner(attr: &AttributeType) -> &AttributeType {
    let carina_core::schema::Shape::List { element_type, .. } =
        attr.shape_ref_free().expect("test schema is Ref-free")
    else {
        panic!("expected List shape");
    };
    element_type
}

fn struct_name(attr: &AttributeType) -> &str {
    let carina_core::schema::Shape::Struct { name, .. } =
        attr.shape_ref_free().expect("test schema is Ref-free")
    else {
        panic!("expected Struct shape");
    };
    name
}

fn union_member(attr: &AttributeType, index: usize) -> &AttributeType {
    let mut budget = carina_core::schema::ShapeWalkBudget::new(8);
    let Some(members) = attr
        .union_members_ref_free_with_budget(&mut budget)
        .expect("test schema is Ref-free")
    else {
        panic!("expected Union shape");
    };
    members
        .iter()
        .nth(index)
        .flatten()
        .unwrap_or_else(|| panic!("missing resolved Union member at index {index}"))
        .as_attr()
}

fn assert_union_string_list_is_unordered(attr: &AttributeType, field_path: &str) {
    let mut budget = carina_core::schema::ShapeWalkBudget::new(8);
    let Some(members) = attr
        .union_members_ref_free_with_budget(&mut budget)
        .expect("test schema is Ref-free")
    else {
        panic!("expected Union shape for {field_path}");
    };
    let ordered = members
        .iter()
        .flatten()
        .map(|member| member.as_attr())
        .find_map(
            |member| match member.shape_ref_free().expect("test schema is Ref-free") {
                carina_core::schema::Shape::List { ordered, .. } => Some(ordered),
                _ => None,
            },
        )
        .unwrap_or_else(|| panic!("missing List Union member for {field_path}"));

    assert!(!ordered, "expected {field_path} List to be unordered");
}

fn string_enum_identity(attr: &AttributeType) -> String {
    let carina_core::schema::Shape::Enum { identity, .. } =
        attr.shape_ref_free().expect("test schema is Ref-free")
    else {
        panic!("expected enum shape with identity");
    };
    identity.to_string()
}

fn assert_refined_string_identity(attr: &AttributeType, expected: &str) {
    if let carina_core::schema::Shape::String {
        identity_layer: carina_core::schema::StringIdentityParts { identity, .. },
        ..
    } = attr.shape_ref_free().expect("test schema is Ref-free")
    {
        assert_eq!(identity.map(|id| id.to_string()).as_deref(), Some(expected));
    } else {
        panic!("expected refined String");
    }
}

fn assert_string_attr_accepts_and_rejects(attr: AttributeType, accepted: &str, rejected: &str) {
    let schema = rootless(attr);
    assert!(
        schema
            .validate(&Value::Concrete(ConcreteValue::String(
                accepted.to_string().into()
            )))
            .is_ok(),
        "expected '{accepted}' to validate"
    );
    assert!(
        schema
            .validate(&Value::Concrete(ConcreteValue::String(
                rejected.to_string().into()
            )))
            .is_err(),
        "expected '{rejected}' to be rejected"
    );
}

fn is_valid_condition_operator(key: &str) -> bool {
    ConditionOperator::from_snake(key).is_some()
}

/// Validate condition operators in a parsed IAM policy document.
///
/// Walks the document looking for `condition` maps and validates that
/// all operator keys are valid snake_case condition operators.
// Test helper mirroring base's shared-crate function; not a production validator.
fn validate_condition_operators(value: &Value) -> Result<(), String> {
    let Value::Concrete(ConcreteValue::Map(doc)) = value else {
        return Ok(());
    };
    // Look for "statement" list
    let Some(Value::Concrete(ConcreteValue::List(statements))) = doc.get("statement") else {
        return Ok(());
    };
    for (i, stmt) in statements.iter().enumerate() {
        let Value::Concrete(ConcreteValue::Map(stmt_map)) = stmt else {
            continue;
        };
        let Some(Value::Concrete(ConcreteValue::Map(condition))) = stmt_map.get("condition") else {
            continue;
        };
        for key in condition.keys() {
            if !is_valid_condition_operator(key) {
                let valid_operators: Vec<&'static str> = ConditionOperatorBase::ALL
                    .iter()
                    .map(|b| b.snake())
                    .collect();
                return Err(format!(
                    "statement[{}]: unknown condition operator '{}'. \
                     Valid operators: {} \
                     (prefix with for_all_values_ or for_any_value_ for set operators, \
                     append _if_exists for conditional variants)",
                    i,
                    key,
                    valid_operators.join(", ")
                ));
            }
        }
    }
    Ok(())
}

/// Validate IAM policy document structure and condition operators.
// Test helper mirroring base's shared-crate function; not a production validator.
pub(crate) fn validate_iam_policy_document(value: &Value) -> Result<(), String> {
    // The IAM policy schema is flat (no `AttributeType::Ref`), so an
    // empty `defs` map is sound here (carina#3345).
    let schema = ValidatedConfigSchema::validate(BTreeMap::from([(
        "policy_document".to_string(),
        iam_policy_document(),
    )]))
    .expect("IAM policy document schema must be valid");
    schema
        .rootless_type("policy_document")
        .expect("IAM policy document type must exist")
        .validate(value)
        .map_err(|e| e.to_string())?;
    validate_condition_operators(value)
}

#[test]
fn iam_policy_document_struct_names_are_plain_and_effect_identity_is_structural() {
    let policy_document = iam_policy_document();
    let policy_statement = list_inner(field_type(&policy_document, "statement"));
    let principal = union_member(field_type(policy_statement, "principal"), 0);

    assert_eq!(struct_name(&policy_document), "PolicyDocument");
    assert_eq!(struct_name(policy_statement), "Statement");
    assert_eq!(struct_name(principal), "Principal");
    assert_eq!(
        string_enum_identity(&iam_policy_effect()),
        "awscc.iam.PolicyDocument.Statement.Effect"
    );
}

#[test]
fn iam_policy_string_lists_are_unordered_but_statements_remain_ordered() {
    let policy_document = iam_policy_document();
    let statement_type = field_type(&policy_document, "statement");
    let carina_core::schema::Shape::List {
        element_type: policy_statement,
        ordered: statements_ordered,
        ..
    } = statement_type
        .shape_ref_free()
        .expect("test schema is Ref-free")
    else {
        panic!("expected statement List shape");
    };
    assert!(statements_ordered, "statement List must remain ordered");

    for field_name in ["action", "not_action", "resource", "not_resource"] {
        assert_union_string_list_is_unordered(field_type(policy_statement, field_name), field_name);
    }

    for principal_field in ["principal", "not_principal"] {
        let principal = union_member(field_type(policy_statement, principal_field), 0);
        for member_field in ["service", "aws", "federated", "canonical_user"] {
            assert_union_string_list_is_unordered(
                field_type(principal, member_field),
                &format!("{principal_field}.{member_field}"),
            );
        }
    }

    let carina_core::schema::Shape::Map {
        value: condition_keys,
        ..
    } = field_type(policy_statement, "condition")
        .shape_ref_free()
        .expect("test schema is Ref-free")
    else {
        panic!("expected condition Map shape");
    };
    let carina_core::schema::Shape::Map {
        value: condition_value,
        ..
    } = condition_keys
        .shape_ref_free()
        .expect("test schema is Ref-free")
    else {
        panic!("expected condition-key Map shape");
    };
    assert_union_string_list_is_unordered(condition_value, "condition value");
}

#[test]
fn iam_policy_document_resource_comparison_treats_string_lists_as_unordered() {
    fn string_list(values: &[&str]) -> Value {
        Value::Concrete(ConcreteValue::List(
            values
                .iter()
                .map(|value| Value::Concrete(ConcreteValue::String((*value).to_string().into())))
                .collect(),
        ))
    }

    fn policy_document(actions: &[&str], principal_arns: &[&str]) -> Value {
        Value::Concrete(ConcreteValue::Map(
            vec![
                (
                    "version".to_string(),
                    Value::Concrete(ConcreteValue::enum_identifier("2012_10_17")),
                ),
                (
                    "statement".to_string(),
                    Value::Concrete(ConcreteValue::List(vec![Value::Concrete(
                        ConcreteValue::Map(
                            vec![
                                (
                                    "effect".to_string(),
                                    Value::Concrete(ConcreteValue::enum_identifier("allow")),
                                ),
                                ("action".to_string(), string_list(actions)),
                                (
                                    "principal".to_string(),
                                    Value::Concrete(ConcreteValue::Map(
                                        vec![("aws".to_string(), string_list(principal_arns))]
                                            .into_iter()
                                            .collect(),
                                    )),
                                ),
                            ]
                            .into_iter()
                            .collect(),
                        ),
                    )])),
                ),
            ]
            .into_iter()
            .collect(),
        ))
    }

    let schema = ResourceSchema::new("awscc.iam.Role")
        .attribute(
            AttributeSchema::optional("assume_role_policy_document", iam_policy_document()).build(),
        )
        .validate()
        .expect("IAM policy diff fixture schema must pass admission");
    let original = policy_document(
        &["sts:AssumeRole", "sts:TagSession"],
        &[
            "arn:aws:iam::111122223333:root",
            "arn:aws:iam::444455556666:root",
        ],
    );
    let reordered = policy_document(
        &["sts:TagSession", "sts:AssumeRole"],
        &[
            "arn:aws:iam::444455556666:root",
            "arn:aws:iam::111122223333:root",
        ],
    );
    let changed = policy_document(
        &["sts:TagSession", "sts:AssumeRole"],
        &[
            "arn:aws:iam::444455556666:root",
            "arn:aws:iam::777788889999:root",
        ],
    );

    let _schema = schema;
    assert!(
        original.semantically_equal(&reordered),
        "IAM action and principal ARN order must not produce a diff"
    );
    assert!(
        !original.semantically_equal(&changed),
        "a changed principal ARN must produce a diff"
    );
}

#[test]
fn aws_account_id_owns_pattern_and_length_in_its_callback() {
    let t = aws_account_id();
    if let carina_core::schema::Shape::String {
        identity_layer:
            carina_core::schema::StringIdentityParts {
                identity,
                pattern,
                length,
                to_dsl: None,
                ..
            },
        ..
    } = t.shape_ref_free().expect("test schema is Ref-free")
    {
        assert_eq!(
            identity.map(|id| id.to_string()).as_deref(),
            Some("awscc.AccountId")
        );
        assert!(
            pattern.is_some() && length == Some((Some(12), Some(12))),
            "identity layer owns the policy"
        );
    } else {
        panic!("aws_account_id() should be refined String");
    }
}

#[test]
fn vpc_id_carries_identity_and_callback_owned_pattern() {
    let t = vpc_id();
    if let carina_core::schema::Shape::String {
        identity_layer:
            carina_core::schema::StringIdentityParts {
                identity,
                pattern,
                to_dsl: None,
                ..
            },
        ..
    } = t.shape_ref_free().expect("test schema is Ref-free")
    {
        assert_eq!(
            identity.map(|id| id.to_string()).as_deref(),
            Some("awscc.ec2.Vpc.Id")
        );
        assert_eq!(
            pattern.map(carina_core::schema::PatternSpec::as_str),
            Some("^vpc-[0-9a-f]{8,}$")
        );
    } else {
        panic!("vpc_id() should be refined String");
    }
}

#[test]
fn validate_arn_valid() {
    assert!(validate_arn("arn:aws:s3:::my-bucket").is_ok());
    assert!(validate_arn("arn:aws:iam::123456789012:role/MyRole").is_ok());
    assert!(validate_arn("arn:aws-cn:s3:::my-bucket").is_ok());
    assert!(validate_arn("arn:aws:ec2:us-east-1:123456789012:vpc/vpc-1234").is_ok());
}

#[test]
fn validate_arn_invalid() {
    assert!(validate_arn("not-an-arn").is_err());
    assert!(validate_arn("arn:aws:s3").is_err());
    assert!(validate_arn("arn:aws").is_err());
    assert!(validate_arn("").is_err());
}

#[test]
fn validate_arn_rejects_empty_partition() {
    // "arn::::::" has empty partition and service — should be rejected
    assert!(validate_arn("arn::s3:::my-bucket").is_err());
    assert!(validate_arn("arn:::::").is_err());
}

#[test]
fn validate_arn_rejects_empty_service() {
    assert!(validate_arn("arn:aws::::").is_err());
    assert!(validate_arn("arn:aws:::123456789012:resource").is_err());
}

#[test]
fn validate_arn_type_with_value() {
    let t = arn();
    assert!(
        rootless(t.clone())
            .validate(&Value::Concrete(ConcreteValue::String(
                "arn:aws:s3:::my-bucket".to_string().into()
            )))
            .is_ok()
    );
    assert!(
        rootless(t.clone())
            .validate(&Value::Concrete(ConcreteValue::String(
                "not-an-arn".to_string().into()
            )))
            .is_err()
    );
    assert!(
        rootless(t.clone())
            .validate(&Value::Concrete(ConcreteValue::Int(42)))
            .is_err()
    );
    assert!(
        rootless(t.clone())
            .validate(
                &Value::resource_ref("role".to_string(), "arn".to_string(), vec![])
                    .resolve_root_for_test()
            )
            .is_ok()
    );
}

#[test]
fn validate_aws_resource_id_type_with_value() {
    let t = aws_resource_id();
    assert!(
        rootless(t.clone())
            .validate(&Value::Concrete(ConcreteValue::String(
                "vpc-1a2b3c4d".to_string().into()
            )))
            .is_ok()
    );
    assert!(
        rootless(t.clone())
            .validate(&Value::Concrete(ConcreteValue::String(
                "vpc".to_string().into()
            )))
            .is_err()
    );
    assert!(
        rootless(t.clone())
            .validate(&Value::Concrete(ConcreteValue::Int(42)))
            .is_err()
    );
    assert!(
        rootless(t.clone())
            .validate(
                &Value::resource_ref("my_vpc".to_string(), "vpc_id".to_string(), vec![])
                    .resolve_root_for_test()
            )
            .is_ok()
    );
}

#[test]
fn validate_vpc_cidr_block_association_id_valid() {
    let t = vpc_cidr_block_association_id();
    assert!(
        rootless(t.clone())
            .validate(&Value::Concrete(ConcreteValue::String(
                "vpc-cidr-assoc-12345678".to_string().into()
            )))
            .is_ok()
    );
    assert!(
        rootless(t.clone())
            .validate(&Value::Concrete(ConcreteValue::String(
                "vpc-cidr-assoc-0123456789abcdef0".to_string().into()
            )))
            .is_ok()
    );
}

#[test]
fn validate_vpc_cidr_block_association_id_invalid() {
    let t = vpc_cidr_block_association_id();
    assert!(
        rootless(t.clone())
            .validate(&Value::Concrete(ConcreteValue::String(
                "vpc-12345678".to_string().into()
            )))
            .is_err()
    );
    assert!(
        rootless(t.clone())
            .validate(&Value::Concrete(ConcreteValue::String(
                "invalid".to_string().into()
            )))
            .is_err()
    );
}

#[test]
fn validate_tgw_route_table_id_valid() {
    let t = tgw_route_table_id();
    assert!(
        rootless(t.clone())
            .validate(&Value::Concrete(ConcreteValue::String(
                "tgw-rtb-12345678".to_string().into()
            )))
            .is_ok()
    );
    assert!(
        rootless(t.clone())
            .validate(&Value::Concrete(ConcreteValue::String(
                "tgw-rtb-0123456789abcdef0".to_string().into()
            )))
            .is_ok()
    );
}

#[test]
fn validate_tgw_route_table_id_invalid() {
    let t = tgw_route_table_id();
    // Regular route table ID should fail
    assert!(
        rootless(t.clone())
            .validate(&Value::Concrete(ConcreteValue::String(
                "rtb-12345678".to_string().into()
            )))
            .is_err()
    );
    // Transit gateway ID should fail
    assert!(
        rootless(t.clone())
            .validate(&Value::Concrete(ConcreteValue::String(
                "tgw-12345678".to_string().into()
            )))
            .is_err()
    );
    assert!(
        rootless(t.clone())
            .validate(&Value::Concrete(ConcreteValue::String(
                "invalid".to_string().into()
            )))
            .is_err()
    );
}

#[test]
fn validate_availability_zone_valid() {
    assert!(validate_availability_zone("us-east-1a").is_ok());
    assert!(validate_availability_zone("ap-northeast-1c").is_ok());
    assert!(validate_availability_zone("eu-central-1b").is_ok());
    assert!(validate_availability_zone("me-south-1a").is_ok());
    assert!(validate_availability_zone("us-west-2d").is_ok());
}

#[test]
fn validate_availability_zone_local_zone() {
    // Local Zones: us-east-1-bos-1a, us-west-2-lax-1a
    assert!(validate_availability_zone("us-east-1-bos-1a").is_ok());
    assert!(validate_availability_zone("us-west-2-lax-1a").is_ok());
    assert!(validate_availability_zone("ap-northeast-1-tpe-1a").is_ok());
}

#[test]
fn validate_availability_zone_wavelength_zone() {
    // Wavelength Zones: us-east-1-wl1-bos-wlz-1
    assert!(validate_availability_zone("us-east-1-wl1-bos-wlz-1").is_ok());
    assert!(validate_availability_zone("us-west-2-wl1-las-wlz-1").is_ok());
}

#[test]
fn validate_availability_zone_invalid() {
    assert!(validate_availability_zone("us-east-1").is_err()); // region, not AZ
    assert!(validate_availability_zone("a").is_err()); // too short
    assert!(validate_availability_zone("invalid").is_err()); // no numeric part
    assert!(validate_availability_zone("us-east").is_err()); // no numeric part
}

#[test]
fn az_accepts_aws_format() {
    let az_type = availability_zone();
    assert!(
        rootless(az_type.clone())
            .validate(&Value::Concrete(ConcreteValue::String(
                "us-east-1a".to_string().into()
            )))
            .is_ok()
    );
}

#[test]
fn az_rejects_quoted_dsl_format() {
    let az_type = availability_zone();
    assert!(
        rootless(az_type.clone())
            .validate(&Value::Concrete(ConcreteValue::String(
                "awscc.AvailabilityZone.ZoneName.us_east_1a"
                    .to_string()
                    .into()
            )))
            .is_err()
    );
}

#[test]
fn az_validator_rejects_dsl_spelling() {
    assert!(validate_availability_zone("us_east_1a").is_err());
}

#[test]
fn az_rejects_quoted_kind_shorthand_format() {
    let az_type = availability_zone();
    assert!(
        rootless(az_type.clone())
            .validate(&Value::Concrete(ConcreteValue::String(
                "ZoneName.us_east_1a".to_string().into()
            )))
            .is_err()
    );
}

#[test]
fn az_rejects_invalid_az() {
    let az_type = availability_zone();
    assert!(
        rootless(az_type.clone())
            .validate(&Value::Concrete(ConcreteValue::String(
                "invalid-zone".to_string().into()
            )))
            .is_err()
    );
}

#[test]
fn az_rejects_wrong_namespace() {
    let az_type = availability_zone();
    assert!(
        rootless(az_type.clone())
            .validate(&Value::Concrete(ConcreteValue::String(
                "gcp.AvailabilityZone.ZoneName.us_east_1a"
                    .to_string()
                    .into()
            )))
            .is_err()
    );
}

#[test]
fn az_has_namespace() {
    // Post-#3222: AZ is an enum, not a `Custom`. The legacy
    // `namespace: Some("awscc")` field is now derived from the
    // structured identity via `dotted_prefix()`.
    let az_type = availability_zone();
    if let carina_core::schema::Shape::Enum { identity, .. } =
        az_type.shape_ref_free().expect("test schema is Ref-free")
    {
        assert_eq!(
            identity.dotted_prefix().as_deref(),
            Some("awscc.AvailabilityZone")
        );
    } else {
        panic!("Expected enum type");
    }
}

#[test]
fn az_has_to_dsl() {
    let az_type = availability_zone();
    if let carina_core::schema::Shape::Enum { to_dsl, .. } =
        az_type.shape_ref_free().expect("test schema is Ref-free")
    {
        assert_eq!(to_dsl, Some(&DslTransform::HyphenToUnderscore));
        assert_eq!(
            DslTransform::HyphenToUnderscore.apply("us-east-1a"),
            "us_east_1a"
        );
    } else {
        panic!("Expected enum type");
    }
}

#[test]
fn validate_availability_zone_id_type_with_value() {
    let t = availability_zone_id();
    assert!(
        rootless(t.clone())
            .validate(&Value::Concrete(ConcreteValue::String(
                "use1-az1".to_string().into()
            )))
            .is_ok()
    );
    assert!(
        rootless(t.clone())
            .validate(&Value::Concrete(ConcreteValue::String(
                "us-east-1a".to_string().into()
            )))
            .is_err()
    );
    assert!(
        rootless(t.clone())
            .validate(&Value::Concrete(ConcreteValue::Int(42)))
            .is_err()
    );
    assert!(
        rootless(t.clone())
            .validate(
                &Value::resource_ref(
                    "subnet".to_string(),
                    "availability_zone_id".to_string(),
                    vec![]
                )
                .resolve_root_for_test()
            )
            .is_ok()
    );
}

#[test]
fn canonicalize_enum_value_matches() {
    assert_eq!(
        canonicalize_enum_value("enabled", &["Enabled", "Suspended"]),
        "Enabled"
    );
}

#[test]
fn canonicalize_enum_value_no_match() {
    assert_eq!(
        canonicalize_enum_value("unknown", &["Enabled", "Suspended"]),
        "unknown"
    );
}

#[test]
fn validate_kms_key_id_arn() {
    assert!(
        validate_kms_key_id(
            "arn:aws:kms:us-east-1:123456789012:key/1234abcd-12ab-34cd-56ef-1234567890ab"
        )
        .is_ok()
    );
    assert!(validate_kms_key_id("arn:aws:kms:us-east-1:123456789012:alias/my-key").is_ok());
}

#[test]
fn validate_kms_key_id_alias() {
    assert!(validate_kms_key_id("alias/my-key").is_ok());
    assert!(validate_kms_key_id("alias/").is_err());
}

#[test]
fn validate_kms_key_id_uuid() {
    assert!(validate_kms_key_id("1234abcd-12ab-34cd-56ef-1234567890ab").is_ok());
    assert!(validate_kms_key_id("not-a-uuid").is_err());
}

#[test]
fn kms_key_id_carries_identity_and_validates_values() {
    let attr = kms_key_id();
    assert_refined_string_identity(&attr, "awscc.kms.Key.Id");
    assert_string_attr_accepts_and_rejects(
        attr,
        "arn:aws:kms:us-east-1:123456789012:key/1234abcd-12ab-34cd-56ef-1234567890ab",
        "not-a-kms-key",
    );
}

#[test]
fn validate_service_arn_valid() {
    assert!(
        validate_service_arn(
            "arn:aws:iam::123456789012:role/MyRole",
            "iam",
            Some("role/")
        )
        .is_ok()
    );
}

#[test]
fn validate_service_arn_wrong_service() {
    assert!(validate_service_arn("arn:aws:s3:::bucket", "iam", None).is_err());
}

#[test]
fn validate_service_arn_wrong_prefix() {
    assert!(
        validate_service_arn(
            "arn:aws:iam::123456789012:user/MyUser",
            "iam",
            Some("role/")
        )
        .is_err()
    );
}

#[test]
fn validate_arn_rejects_invalid_partition() {
    assert!(validate_arn("arn:xxx:iam::aws:policy/Foo").is_err());
    assert!(validate_arn("arn:invalid:s3:::bucket").is_err());
}

#[test]
fn validate_arn_accepts_valid_partitions() {
    assert!(validate_arn("arn:aws:s3:::bucket").is_ok());
    assert!(validate_arn("arn:aws-cn:s3:::bucket").is_ok());
    assert!(validate_arn("arn:aws-us-gov:s3:::bucket").is_ok());
}

#[test]
fn validate_iam_arn_rejects_non_empty_region() {
    assert!(validate_iam_arn("arn:aws:iam:us-east-1:aws:policy/Foo", "policy/").is_err());
}

#[test]
fn validate_iam_arn_rejects_short_account_id() {
    assert!(validate_iam_arn("arn:aws:iam::1234:policy/Foo", "policy/").is_err());
}

#[test]
fn validate_iam_arn_rejects_non_digit_account() {
    assert!(validate_iam_arn("arn:aws:iam::aw:policy/Foo", "policy/").is_err());
}

#[test]
fn validate_iam_arn_accepts_aws_managed() {
    assert!(validate_iam_arn("arn:aws:iam::aws:policy/AdministratorAccess", "policy/").is_ok());
}

#[test]
fn validate_iam_arn_accepts_customer_managed() {
    assert!(validate_iam_arn("arn:aws:iam::123456789012:policy/MyPolicy", "policy/").is_ok());
}

#[test]
fn validate_iam_arn_rejects_empty_resource_name() {
    assert!(validate_iam_arn("arn:aws:iam::aws:policy/", "policy/").is_err());
}

#[test]
fn validate_iam_arn_rejects_invalid_resource_chars() {
    assert!(validate_iam_arn("arn:aws:iam::aws:policy/My Policy", "policy/").is_err());
    assert!(validate_iam_arn("arn:aws:iam::aws:policy/foo<bar>", "policy/").is_err());
}

#[test]
fn validate_iam_arn_accepts_path_prefix() {
    assert!(
        validate_iam_arn(
            "arn:aws:iam::123456789012:role/service-role/MyRole",
            "role/"
        )
        .is_ok()
    );
}

#[test]
fn validate_iam_arn_error_says_iam_policy_arn() {
    let err = validate_iam_arn("arn:aws:iam:us-east-1:aws:policy/Foo", "policy/").unwrap_err();
    assert!(
        err.contains("IAM Policy ARN"),
        "Error should say 'IAM Policy ARN': {err}"
    );
    assert!(
        err.contains("arn:aws:iam:us-east-1:aws:policy/Foo"),
        "Error should include full ARN: {err}"
    );
}

#[test]
fn validate_iam_arn_error_says_iam_role_arn() {
    let err = validate_iam_arn("arn:aws:iam:us-east-1:aws:role/Foo", "role/").unwrap_err();
    assert!(
        err.contains("IAM Role ARN"),
        "Error should say 'IAM Role ARN': {err}"
    );
}

#[test]
fn iam_role_arn_carries_identity_and_validates_values() {
    let attr = iam_role_arn();
    assert_refined_string_identity(&attr, "awscc.iam.Role.Arn");
    assert_string_attr_accepts_and_rejects(
        attr,
        "arn:aws:iam::123456789012:role/service-role/MyRole",
        "arn:aws:iam::123456789012:policy/MyPolicy",
    );
}

#[test]
fn iam_policy_arn_carries_identity_and_validates_values() {
    let attr = iam_policy_arn();
    assert_refined_string_identity(&attr, "awscc.iam.Policy.Arn");
    assert_string_attr_accepts_and_rejects(
        attr,
        "arn:aws:iam::123456789012:policy/MyPolicy",
        "arn:aws:iam::123456789012:role/MyRole",
    );
}

#[test]
fn iam_oidc_provider_arn_carries_identity_and_validates_values() {
    let attr = iam_oidc_provider_arn();
    assert_refined_string_identity(&attr, "awscc.iam.OidcProvider.Arn");
    assert_string_attr_accepts_and_rejects(
        attr,
        "arn:aws:iam::123456789012:oidc-provider/token.actions.githubusercontent.com",
        "arn:aws:iam::123456789012:role/MyRole",
    );
}

#[test]
fn validate_iam_policy_document_basic() {
    let doc = Value::Concrete(ConcreteValue::Map(
        vec![
            (
                "version".to_string(),
                Value::Concrete(ConcreteValue::enum_identifier("2012_10_17")),
            ),
            (
                "statement".to_string(),
                Value::Concrete(ConcreteValue::List(vec![Value::Concrete(
                    ConcreteValue::Map(
                        vec![
                            (
                                "effect".to_string(),
                                Value::Concrete(ConcreteValue::enum_identifier("allow")),
                            ),
                            (
                                "action".to_string(),
                                Value::Concrete(ConcreteValue::String(
                                    "sts:AssumeRole".to_string().into(),
                                )),
                            ),
                            (
                                "resource".to_string(),
                                Value::Concrete(ConcreteValue::String("*".to_string().into())),
                            ),
                        ]
                        .into_iter()
                        .collect(),
                    ),
                )])),
            ),
        ]
        .into_iter()
        .collect(),
    ));
    assert!(validate_iam_policy_document(&doc).is_ok());
}

#[test]
fn validate_iam_policy_document_invalid_version() {
    let doc = Value::Concrete(ConcreteValue::Map(
        vec![(
            "version".to_string(),
            Value::Concrete(ConcreteValue::enum_identifier("2020_01_01")),
        )]
        .into_iter()
        .collect(),
    ));
    assert!(validate_iam_policy_document(&doc).is_err());
}

#[test]
fn validate_iam_policy_document_invalid_effect() {
    let doc = Value::Concrete(ConcreteValue::Map(
        vec![(
            "statement".to_string(),
            Value::Concrete(ConcreteValue::List(vec![Value::Concrete(
                ConcreteValue::Map(
                    vec![(
                        "effect".to_string(),
                        Value::Concrete(ConcreteValue::enum_identifier("grant")),
                    )]
                    .into_iter()
                    .collect(),
                ),
            )])),
        )]
        .into_iter()
        .collect(),
    ));
    assert!(validate_iam_policy_document(&doc).is_err());
}

#[test]
fn iam_policy_document_type_validates() {
    let t = iam_policy_document();
    let valid_doc = Value::Concrete(ConcreteValue::Map(
        vec![
            (
                "version".to_string(),
                Value::Concrete(ConcreteValue::enum_identifier("2012_10_17")),
            ),
            (
                "statement".to_string(),
                Value::Concrete(ConcreteValue::List(vec![Value::Concrete(
                    ConcreteValue::Map(
                        vec![
                            (
                                "effect".to_string(),
                                Value::Concrete(ConcreteValue::enum_identifier("deny")),
                            ),
                            (
                                "action".to_string(),
                                Value::Concrete(ConcreteValue::String("s3:*".to_string().into())),
                            ),
                            (
                                "resource".to_string(),
                                Value::Concrete(ConcreteValue::String("*".to_string().into())),
                            ),
                        ]
                        .into_iter()
                        .collect(),
                    ),
                )])),
            ),
        ]
        .into_iter()
        .collect(),
    ));
    assert!(rootless(t.clone()).validate(&valid_doc).is_ok());
}

#[test]
fn iam_policy_document_principal_map_validates() {
    let t = iam_policy_document();
    // principal as a map: { service = "ec2.amazonaws.com" }
    let doc_with_principal_map = Value::Concrete(ConcreteValue::Map(
        vec![
            (
                "version".to_string(),
                Value::Concrete(ConcreteValue::enum_identifier("2012_10_17")),
            ),
            (
                "statement".to_string(),
                Value::Concrete(ConcreteValue::List(vec![Value::Concrete(
                    ConcreteValue::Map(
                        vec![
                            (
                                "effect".to_string(),
                                Value::Concrete(ConcreteValue::enum_identifier("allow")),
                            ),
                            (
                                "principal".to_string(),
                                Value::Concrete(ConcreteValue::Map(
                                    vec![(
                                        "service".to_string(),
                                        Value::Concrete(ConcreteValue::String(
                                            "ec2.amazonaws.com".to_string().into(),
                                        )),
                                    )]
                                    .into_iter()
                                    .collect(),
                                )),
                            ),
                            (
                                "action".to_string(),
                                Value::Concrete(ConcreteValue::String(
                                    "sts:AssumeRole".to_string().into(),
                                )),
                            ),
                        ]
                        .into_iter()
                        .collect(),
                    ),
                )])),
            ),
        ]
        .into_iter()
        .collect(),
    ));
    assert!(
        rootless(t.clone())
            .validate(&doc_with_principal_map)
            .is_ok(),
        "principal as map (struct) should be valid: {:?}",
        rootless(t.clone()).validate(&doc_with_principal_map)
    );
}

#[test]
fn iam_policy_document_principal_string_validates() {
    let t = iam_policy_document();
    // principal as a string: "*"
    let doc_with_principal_string = Value::Concrete(ConcreteValue::Map(
        vec![
            (
                "version".to_string(),
                Value::Concrete(ConcreteValue::enum_identifier("2012_10_17")),
            ),
            (
                "statement".to_string(),
                Value::Concrete(ConcreteValue::List(vec![Value::Concrete(
                    ConcreteValue::Map(
                        vec![
                            (
                                "effect".to_string(),
                                Value::Concrete(ConcreteValue::enum_identifier("allow")),
                            ),
                            (
                                "principal".to_string(),
                                Value::Concrete(ConcreteValue::String("*".to_string().into())),
                            ),
                            (
                                "action".to_string(),
                                Value::Concrete(ConcreteValue::String(
                                    "sts:AssumeRole".to_string().into(),
                                )),
                            ),
                        ]
                        .into_iter()
                        .collect(),
                    ),
                )])),
            ),
        ]
        .into_iter()
        .collect(),
    ));
    assert!(
        rootless(t.clone())
            .validate(&doc_with_principal_string)
            .is_ok(),
        "principal as string should be valid: {:?}",
        rootless(t.clone()).validate(&doc_with_principal_string)
    );
}

#[test]
fn sso_principal_id_carries_identity_and_validates_values() {
    let attr = sso_principal_id();
    assert_refined_string_identity(&attr, "awscc.sso.Principal.Id");
    assert_string_attr_accepts_and_rejects(
        attr,
        "1234567890-12345678-1234-1234-1234-1234567890ab",
        "",
    );
}

#[test]
fn sso_instance_arn_carries_identity_and_validates_values() {
    let attr = sso_instance_arn();
    assert_refined_string_identity(&attr, "awscc.sso.Instance.Arn");
    assert_string_attr_accepts_and_rejects(
        attr,
        "arn:aws:sso:::instance/ssoins-1234567890abcdef",
        "arn:aws:sso:::permissionSet/ssoins-1234567890abcdef/ps-1234567890abcdef",
    );
}

#[test]
fn identity_store_id_carries_identity_and_validates_values() {
    let attr = identity_store_id();
    assert_refined_string_identity(&attr, "awscc.identitystore.Store.Id");
    assert_string_attr_accepts_and_rejects(attr, "d-1234567890", "store-1234567890");
}

#[test]
fn sso_permission_set_arn_carries_identity_and_validates_values() {
    let attr = sso_permission_set_arn();
    assert_refined_string_identity(&attr, "awscc.sso.PermissionSet.Arn");
    assert_string_attr_accepts_and_rejects(
        attr,
        "arn:aws:sso:::permissionSet/ssoins-1234567890abcdef/ps-1234567890abcdef",
        "arn:aws:sso:::instance/ssoins-1234567890abcdef",
    );
}

#[test]
fn region_accepts_aws_format() {
    let region_type = aws_region();
    assert_eq!(string_enum_identity(&region_type), "awscc.Region");
    assert!(
        rootless(region_type.clone())
            .validate(&Value::Concrete(ConcreteValue::String(
                "ap-northeast-1".to_string().into()
            )))
            .is_ok()
    );
}

#[test]
fn region_rejects_quoted_dsl_format() {
    let region_type = aws_region();
    assert!(
        rootless(region_type.clone())
            .validate(&Value::Concrete(ConcreteValue::String(
                "awscc.Region.ap_northeast_1".to_string().into()
            )))
            .is_err()
    );
}

#[test]
fn region_rejects_quoted_dsl_format_without_aws_prefix() {
    let region_type = aws_region();
    assert!(
        rootless(region_type.clone())
            .validate(&Value::Concrete(ConcreteValue::String(
                "Region.ap_northeast_1".to_string().into()
            )))
            .is_err()
    );
}

#[test]
fn region_validator_rejects_dsl_spelling() {
    assert!(validate_aws_region("ap_northeast_1").is_err());
}

#[test]
fn region_rejects_quoted_canonical_namespace_literal() {
    let region_type = aws_region();
    assert!(
        rootless(region_type)
            .validate(&Value::Concrete(ConcreteValue::String(
                "awscc.Region.us_west_2".to_string().into()
            )))
            .is_err()
    );
}

#[test]
fn region_rejects_invalid_region() {
    let region_type = aws_region();
    let result = rootless(region_type.clone()).validate(&Value::Concrete(ConcreteValue::String(
        "invalid-region".to_string().into(),
    )));
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(err.contains("Invalid region"));
    assert!(err.contains("ap-northeast-1")); // Should suggest valid regions
}

#[test]
fn region_rejects_availability_zone() {
    let region_type = aws_region();
    // ap-northeast-1a is an AZ, not a region
    assert!(
        rootless(region_type.clone())
            .validate(&Value::Concrete(ConcreteValue::String(
                "ap-northeast-1a".to_string().into()
            )))
            .is_err()
    );
}

#[test]
fn region_validates_all_valid_regions() {
    let region_type = aws_region();
    for (region, _) in REGIONS {
        assert!(
            rootless(region_type.clone())
                .validate(&Value::Concrete(ConcreteValue::String(
                    region.to_string().into()
                )))
                .is_ok(),
            "Region {} should be valid",
            region
        );
    }
}

#[test]
fn region_rejects_wrong_namespace() {
    let region_type = aws_region();
    assert!(
        rootless(region_type.clone())
            .validate(&Value::Concrete(ConcreteValue::String(
                "gcp.Region.ap_northeast_1".to_string().into()
            )))
            .is_err()
    );
    assert!(
        rootless(region_type.clone())
            .validate(&Value::Concrete(ConcreteValue::String(
                "awscc.Location.ap_northeast_1".to_string().into()
            )))
            .is_err()
    );
    assert!(
        rootless(region_type.clone())
            .validate(&Value::Concrete(ConcreteValue::String(
                "foo.bar.baz.ap_northeast_1".to_string().into()
            )))
            .is_err()
    );
    assert!(
        rootless(region_type.clone())
            .validate(&Value::Concrete(ConcreteValue::String(
                "Location.ap_northeast_1".to_string().into()
            )))
            .is_err()
    );
}

#[test]
fn region_completions_generates_dsl_format() {
    let completions = region_completions();
    assert_eq!(completions.len(), REGIONS.len());
    // Spot-check a few entries
    assert_eq!(completions[0].value, "awscc.Region.af_south_1");
    assert_eq!(completions[0].description, "Africa (Cape Town)");
    let tokyo = completions
        .iter()
        .find(|c| c.value.contains("ap_northeast_1"))
        .unwrap();
    assert_eq!(tokyo.description, "Asia Pacific (Tokyo)");
}

#[test]
fn route53_hosted_zone_id_carries_identity_and_validates_values() {
    let attr = route53_hosted_zone_id();
    assert_refined_string_identity(&attr, "awscc.route53.HostedZone.Id");
    let carina_core::schema::Shape::String {
        identity_layer:
            carina_core::schema::StringIdentityParts {
                pattern, length, ..
            },
        ..
    } = attr.shape_ref_free().expect("test schema is Ref-free")
    else {
        panic!("expected refined String");
    };
    assert!(
        pattern.is_some() && length == Some((None, Some(32))),
        "identity layer owns both constraints"
    );

    let schema = rootless(attr);
    for hosted_zone_id in ["Z05136711ZXUHBDOA8D5O", "Z2FDTNDATAQYW2", "Z14GRHDCWA56QT"] {
        assert!(
            schema
                .validate(&Value::Concrete(ConcreteValue::String(
                    hosted_zone_id.to_string().into()
                )))
                .is_ok(),
            "expected {hosted_zone_id} to validate"
        );
    }

    let too_long = format!("Z{}", "A".repeat(32));
    assert_eq!(too_long.len(), 33);
    for hosted_zone_id in [
        "",
        "global",
        "awscc.cloudfront.HostedZoneId.global",
        "z14grhdcwa56qt",
        "/hostedzone/Z05136711ZXUHBDOA8D5O",
        too_long.as_str(),
    ] {
        assert!(
            schema
                .validate(&Value::Concrete(ConcreteValue::String(
                    hosted_zone_id.to_string().into()
                )))
                .is_err(),
            "expected {hosted_zone_id:?} to be rejected"
        );
    }
}

#[test]
fn condition_operator_to_aws_basic() {
    assert_eq!(
        condition_operator_to_aws("string_equals"),
        Some("StringEquals".to_string())
    );
    assert_eq!(
        condition_operator_to_aws("arn_like"),
        Some("ArnLike".to_string())
    );
    assert_eq!(condition_operator_to_aws("null"), Some("Null".to_string()));
}

#[test]
fn condition_operator_to_aws_if_exists() {
    assert_eq!(
        condition_operator_to_aws("string_equals_if_exists"),
        Some("StringEqualsIfExists".to_string())
    );
    assert_eq!(
        condition_operator_to_aws("arn_like_if_exists"),
        Some("ArnLikeIfExists".to_string())
    );
}

#[test]
fn condition_operator_to_aws_unknown() {
    assert_eq!(condition_operator_to_aws("unknown_op"), None);
    assert_eq!(condition_operator_to_aws("StringEquals"), None);
}

#[test]
fn condition_operator_to_aws_for_all_values() {
    assert_eq!(
        condition_operator_to_aws("for_all_values_string_equals"),
        Some("ForAllValues:StringEquals".to_string())
    );
    assert_eq!(
        condition_operator_to_aws("for_any_value_string_like"),
        Some("ForAnyValue:StringLike".to_string())
    );
    // Any base operator should work with qualifiers
    assert_eq!(
        condition_operator_to_aws("for_all_values_numeric_equals"),
        Some("ForAllValues:NumericEquals".to_string())
    );
    // Combined qualifier + if_exists
    assert_eq!(
        condition_operator_to_aws("for_all_values_string_like_if_exists"),
        Some("ForAllValues:StringLikeIfExists".to_string())
    );
}

#[test]
fn condition_operator_to_snake_roundtrip() {
    assert_eq!(
        condition_operator_to_snake("ForAllValues:NumericEquals"),
        Some("for_all_values_numeric_equals".to_string())
    );
    assert_eq!(
        condition_operator_to_snake("ForAnyValue:ArnLikeIfExists"),
        Some("for_any_value_arn_like_if_exists".to_string())
    );
}

#[test]
fn condition_operator_typed_roundtrip() {
    // Type-level guarantee: every constructible ConditionOperator round-trips
    // through both snake/AWS Display forms. The schema's StringEnum, the
    // validator's "valid operators" list, and the conversion wrappers all
    // flow from `ConditionOperator::all()`, so this assertion forecloses
    // drift between them.
    for op in ConditionOperator::all() {
        let snake = op.to_snake();
        let aws = op.to_aws();
        assert_eq!(
            ConditionOperator::from_snake(&snake),
            Some(op),
            "snake {snake:?} did not round-trip"
        );
        assert_eq!(
            ConditionOperator::from_aws(&aws),
            Some(op),
            "aws {aws:?} did not round-trip"
        );
        // `Display` is the canonical AWS-wire spelling.
        assert_eq!(op.to_string(), aws);
    }
    // Spot-check a representative spelling, including the qualifier+if_exists
    // combination that issue #340 unblocked.
    let op = ConditionOperator {
        qualifier: Some(ConditionQualifier::ForAllValues),
        base: ConditionOperatorBase::StringLike,
        if_exists: true,
    };
    assert_eq!(op.to_snake(), "for_all_values_string_like_if_exists");
    assert_eq!(op.to_aws(), "ForAllValues:StringLikeIfExists");
}

#[test]
fn condition_operator_total_count_matches_cross_product() {
    // 27 base × 3 qualifier options (None, ForAllValues, ForAnyValue) × 2
    // if_exists options = 162. If a new base operator is added the count
    // moves in lockstep, which is the property we want.
    let n_base = ConditionOperatorBase::ALL.len();
    let n_qualifier_options = 1 + ConditionQualifier::ALL.len();
    let expected = n_base * n_qualifier_options * 2;
    assert_eq!(ConditionOperator::all().count(), expected);
}

#[test]
fn condition_type_string_enum_includes_qualifier_and_if_exists_variants() {
    // The schema's enum values must enumerate every snake_case spelling
    // that `condition_operator_to_aws` accepts — base, qualifier-prefixed,
    // `_if_exists` suffixed, and the combination — so that `validate` does
    // not reject inputs that the conversion layer already handles.
    let cond = condition_type();
    let carina_core::schema::Shape::Map { key, .. } =
        cond.shape_ref_free().expect("test schema is Ref-free")
    else {
        panic!("condition_type() should be a Map");
    };
    let carina_core::schema::Shape::Enum {
        values: Some(values),
        ..
    } = key.shape_ref_free().expect("test schema is Ref-free")
    else {
        panic!("condition_type() key should be an enum");
    };
    for expected in [
        "string_equals",
        "for_all_values_string_equals",
        "for_any_value_string_like",
        "string_equals_if_exists",
        "for_all_values_string_like_if_exists",
        "for_any_value_arn_like_if_exists",
        "null",
    ] {
        assert!(
            values.iter().any(|v| v == expected),
            "ConditionOperator StringEnum should include {expected:?}; got {values:?}"
        );
    }
    // Sanity: every value in the schema must round-trip through the
    // conversion layer, otherwise the two stay in drift.
    for v in values {
        assert!(
            condition_operator_to_aws(v).is_some(),
            "schema value {v:?} not accepted by condition_operator_to_aws"
        );
    }
}

#[test]
fn validate_condition_operators_accepts_valid() {
    let doc = Value::Concrete(ConcreteValue::Map(
        vec![(
            "statement".to_string(),
            Value::Concrete(ConcreteValue::List(vec![Value::Concrete(
                ConcreteValue::Map(
                    vec![(
                        "condition".to_string(),
                        Value::Concrete(ConcreteValue::Map(
                            vec![(
                                "string_equals".to_string(),
                                Value::Concrete(ConcreteValue::Map(
                                    vec![(
                                        "aws:RequestedRegion".to_string(),
                                        Value::Concrete(ConcreteValue::String(
                                            "us-east-1".to_string().into(),
                                        )),
                                    )]
                                    .into_iter()
                                    .collect(),
                                )),
                            )]
                            .into_iter()
                            .collect(),
                        )),
                    )]
                    .into_iter()
                    .collect(),
                ),
            )])),
        )]
        .into_iter()
        .collect(),
    ));
    assert!(validate_condition_operators(&doc).is_ok());
}

#[test]
fn validate_condition_operators_rejects_pascal_case() {
    let doc = Value::Concrete(ConcreteValue::Map(
        vec![(
            "statement".to_string(),
            Value::Concrete(ConcreteValue::List(vec![Value::Concrete(
                ConcreteValue::Map(
                    vec![(
                        "condition".to_string(),
                        Value::Concrete(ConcreteValue::Map(
                            vec![(
                                "StringEquals".to_string(),
                                Value::Concrete(ConcreteValue::Map(indexmap::IndexMap::new())),
                            )]
                            .into_iter()
                            .collect(),
                        )),
                    )]
                    .into_iter()
                    .collect(),
                ),
            )])),
        )]
        .into_iter()
        .collect(),
    ));
    let err = validate_condition_operators(&doc).unwrap_err();
    assert!(
        err.contains("StringEquals"),
        "Error should mention the invalid key: {err}"
    );
}

#[test]
fn validate_condition_operators_rejects_unknown() {
    let doc = Value::Concrete(ConcreteValue::Map(
        vec![(
            "statement".to_string(),
            Value::Concrete(ConcreteValue::List(vec![Value::Concrete(
                ConcreteValue::Map(
                    vec![(
                        "condition".to_string(),
                        Value::Concrete(ConcreteValue::Map(
                            vec![(
                                "foo_bar".to_string(),
                                Value::Concrete(ConcreteValue::Map(indexmap::IndexMap::new())),
                            )]
                            .into_iter()
                            .collect(),
                        )),
                    )]
                    .into_iter()
                    .collect(),
                ),
            )])),
        )]
        .into_iter()
        .collect(),
    ));
    assert!(validate_condition_operators(&doc).is_err());
}

#[test]
fn validate_condition_operators_accepts_if_exists() {
    let doc = Value::Concrete(ConcreteValue::Map(
        vec![(
            "statement".to_string(),
            Value::Concrete(ConcreteValue::List(vec![Value::Concrete(
                ConcreteValue::Map(
                    vec![(
                        "condition".to_string(),
                        Value::Concrete(ConcreteValue::Map(
                            vec![(
                                "string_equals_if_exists".to_string(),
                                Value::Concrete(ConcreteValue::Map(indexmap::IndexMap::new())),
                            )]
                            .into_iter()
                            .collect(),
                        )),
                    )]
                    .into_iter()
                    .collect(),
                ),
            )])),
        )]
        .into_iter()
        .collect(),
    ));
    assert!(validate_condition_operators(&doc).is_ok());
}

#[test]
fn validate_iam_policy_document_accepts_for_all_values_string_equals() {
    // Regression for the route53 cross-account delegation writer use case
    // (issue #340): `for_all_values_string_equals` is the only way to narrow
    // `route53:ChangeResourceRecordSets` to a specific record-name / type /
    // action set. The conversion layer accepts it; the schema must too.
    let condition_inner = Value::Concrete(ConcreteValue::Map(
        vec![(
            "route53:ChangeResourceRecordSetsRecordTypes".to_string(),
            Value::Concrete(ConcreteValue::List(vec![Value::Concrete(
                ConcreteValue::String("NS".to_string().into()),
            )])),
        )]
        .into_iter()
        .collect(),
    ));
    let statement = Value::Concrete(ConcreteValue::Map(
        vec![
            (
                "effect".to_string(),
                Value::Concrete(ConcreteValue::enum_identifier("allow")),
            ),
            (
                "action".to_string(),
                Value::Concrete(ConcreteValue::String(
                    "route53:ChangeResourceRecordSets".to_string().into(),
                )),
            ),
            (
                "resource".to_string(),
                Value::Concrete(ConcreteValue::String(
                    "arn:aws:route53:::hostedzone/ABC".to_string().into(),
                )),
            ),
            (
                "condition".to_string(),
                Value::Concrete(ConcreteValue::Map(
                    vec![("for_all_values_string_equals".to_string(), condition_inner)]
                        .into_iter()
                        .collect(),
                )),
            ),
        ]
        .into_iter()
        .collect(),
    ));
    let doc = Value::Concrete(ConcreteValue::Map(
        vec![
            (
                "version".to_string(),
                Value::Concrete(ConcreteValue::enum_identifier("2012_10_17")),
            ),
            (
                "statement".to_string(),
                Value::Concrete(ConcreteValue::List(vec![statement])),
            ),
        ]
        .into_iter()
        .collect(),
    ));
    validate_iam_policy_document(&doc)
        .expect("for_all_values_string_equals must validate (schema must match conversion layer)");
}

#[test]
fn iam_policy_effect_is_string_enum() {
    let effect = crate::schemas::type_helpers::policy_seed::iam_policy_effect();
    if let carina_core::schema::Shape::Enum {
        values: Some(values),
        identity,
        dsl_aliases,
        ..
    } = effect.shape_ref_free().expect("test schema is Ref-free")
    {
        assert_eq!(identity.kind, "Effect");
        assert_eq!(values, &["Allow".to_string(), "Deny".to_string()]);
        assert_eq!(
            identity.dotted_prefix(),
            Some("awscc.iam.PolicyDocument.Statement".to_string())
        );
        assert_eq!(
            dsl_aliases,
            &[
                ("Allow".to_string(), "allow".to_string()),
                ("Deny".to_string(), "deny".to_string()),
            ]
        );
    } else {
        panic!("expected enum");
    }
}

#[test]
fn iam_policy_version_is_string_enum() {
    let version = crate::schemas::type_helpers::policy_seed::iam_policy_version();
    if let carina_core::schema::Shape::Enum {
        values: Some(values),
        identity,
        dsl_aliases,
        ..
    } = version.shape_ref_free().expect("test schema is Ref-free")
    {
        assert_eq!(identity.kind, "Version");
        assert_eq!(
            values,
            &["2012-10-17".to_string(), "2008-10-17".to_string()]
        );
        assert_eq!(
            identity.dotted_prefix(),
            Some("awscc.iam.PolicyDocument".to_string())
        );
        assert_eq!(
            dsl_aliases,
            &[
                ("2012-10-17".to_string(), "2012_10_17".to_string()),
                ("2008-10-17".to_string(), "2008_10_17".to_string()),
            ]
        );
    } else {
        panic!("expected enum");
    }
}

fn assert_matcher_codes(row: ScalarType, valid: &[&str], invalid: &[&str]) {
    let admitted = rootless(row.attribute_type());
    for code in valid {
        let value = Value::Concrete(ConcreteValue::String((*code).to_string().into()));
        assert!(
            admitted.validate_collect(&value).is_empty(),
            "{row:?}: {code:?}"
        );
    }
    for code in invalid {
        let value = Value::Concrete(ConcreteValue::String((*code).to_string().into()));
        assert!(
            !admitted.validate_collect(&value).is_empty(),
            "{row:?}: {code:?}"
        );
    }
}

#[test]
fn patterns_accept_current_partitions_and_regions() {
    let key_pattern = regex::Regex::new(kms_key_arn_pattern()).unwrap();
    let reference_pattern = regex::Regex::new(kms_key_reference_pattern()).unwrap();
    for (partition, region) in [
        ("aws", "us-east-1"),
        ("aws", "ap-northeast-1"),
        ("aws-cn", "cn-north-1"),
        ("aws-us-gov", "us-gov-west-1"),
        ("aws-iso", "us-iso-east-1"),
        ("aws-iso-b", "us-isob-east-1"),
        ("aws-iso-e", "eu-isoe-west-1"),
        ("aws-iso-f", "us-isof-south-1"),
        ("aws-eusc", "eusc-de-east-1"),
        ("aws", "il-central-1"),
        ("aws", "ap-southeast-7"),
        ("aws", "mx-central-1"),
    ] {
        for key_id in KEY_IDS {
            let arn = format!("arn:{partition}:kms:{region}:123456789012:key/{key_id}");
            assert!(key_pattern.is_match(&arn), "{arn}");
            assert!(reference_pattern.is_match(&arn), "{arn}");
            assert!(validate_kms_key_arn(&arn).is_ok(), "{arn}");
            assert!(validate_kms_key_reference(&arn).is_ok(), "{arn}");
        }
        let alias = format!("arn:{partition}:kms:{region}:123456789012:alias/customer");
        assert!(!key_pattern.is_match(&alias), "{alias}");
        assert!(reference_pattern.is_match(&alias), "{alias}");
        assert!(validate_kms_key_reference(&alias).is_ok(), "{alias}");
    }
}

#[test]
fn reference_pattern_accepts_uuid_and_mrk_ids_in_both_cases() {
    let pattern = regex::Regex::new(kms_key_reference_pattern()).unwrap();
    for key_id in KEY_IDS {
        assert!(pattern.is_match(key_id), "{key_id}");
        assert!(validate_kms_key_reference(key_id).is_ok(), "{key_id}");
        assert!(validate_kms_key_arn(key_id).is_err(), "{key_id}");
    }
}

#[test]
fn patterns_reject_malformed_arns() {
    let key_pattern = regex::Regex::new(kms_key_arn_pattern()).unwrap();
    let reference_pattern = regex::Regex::new(kms_key_reference_pattern()).unwrap();
    for (partition, region, account, resource) in [
        (
            "aws-xx",
            "us-east-1",
            "123456789012",
            "key/1234abcd-12ab-34cd-56ef-1234567890ab",
        ),
        (
            "AWS",
            "us-east-1",
            "123456789012",
            "key/1234abcd-12ab-34cd-56ef-1234567890ab",
        ),
        (
            "aws",
            "us-east-1",
            "12345678901",
            "key/1234abcd-12ab-34cd-56ef-1234567890ab",
        ),
        ("aws", "us-east-1", "123456789012", "key/not-a-uuid"),
        (
            "aws",
            "us-east-1",
            "123456789012",
            "key/mrk-0123456789abcdef0123456789abcde",
        ),
        (
            "aws",
            "us-east-1",
            "123456789012",
            "key/mrk-0123456789abcdef0123456789abcdeg",
        ),
        ("aws-xx", "us-east-1", "123456789012", "alias/customer"),
        ("AWS", "us-east-1", "123456789012", "alias/customer"),
        ("aws", "us-east-1", "12345678901", "alias/customer"),
        ("aws", "u-east-1", "123456789012", "alias/customer"),
        ("aws", "euscc-de-east-1", "123456789012", "alias/customer"),
    ] {
        let arn = format!("arn:{partition}:kms:{region}:{account}:{resource}");
        assert!(!key_pattern.is_match(&arn), "{arn}");
        assert!(!reference_pattern.is_match(&arn), "{arn}");
        assert!(validate_kms_key_arn(&arn).is_err(), "{arn}");
        assert!(validate_kms_key_reference(&arn).is_err(), "{arn}");
    }
}

#[test]
fn elbv2_http_matcher_codes_validate_boundaries_lists_and_ranges() {
    assert_matcher_codes(
        ScalarType::Elbv2TargetGroupHttpCode,
        &[
            "200",
            "599",
            "200,202,599",
            "200-599",
            "200,201-299,500-599",
            "200-299,302",
            "200-200",
        ],
        &[
            "0200",
            "200-0299",
            "200,0302",
            "0200-0599",
            "0200-200",
            "0200,0599",
            "0000000000000000000000000000000200",
            "199",
            "600",
            "199-200",
            "200-600",
            "600-600",
            "599-200",
            "0201-200",
            "200,199",
            "200,600",
            "",
            ",",
            ",200",
            "200,",
            "200,,201",
            "200, 201",
            "200 ",
            " 200",
            "200 -201",
            "200- 201",
            "200\n",
            "200\r",
            "200\t",
            "2 00",
            "-200",
            "+200",
            "200-",
            "-",
            "200--201",
            "200-201-202",
            "200.0",
            "2e2",
            "0xC8",
            "２００",
            "٢٠٠",
            "abc",
            "200\0",
            "65536",
            "999999999999999999999999",
        ],
    );
}

#[test]
fn elbv2_grpc_matcher_codes_validate_boundaries_lists_and_ranges() {
    assert_matcher_codes(
        ScalarType::Elbv2TargetGroupGrpcCode,
        &[
            "0",
            "99",
            "0,1,99",
            "0-99",
            "0,1-5,80-99",
            "0-5",
            "12",
            "0-0",
            "99-99",
        ],
        &[
            "00-05",
            "0,05",
            "00",
            "080",
            "00-099",
            "080-80",
            "00,080,099",
            "0000000000000000000000000000000080",
            "100",
            "-1",
            "0-100",
            "100-100",
            "99-0",
            "080-79",
            "0,100",
            "",
            ",",
            ",0",
            "0,",
            "0,,1",
            "0, 1",
            "0 ",
            " 0",
            "0 -1",
            "0- 1",
            "0\n",
            "0\r",
            "0\t",
            "+0",
            "0-",
            "-",
            "0--1",
            "0-1-2",
            "0.0",
            "1e1",
            "0x0",
            "０",
            "٠",
            "abc",
            "0\0",
            "65536",
            "999999999999999999999999",
        ],
    );
}

fn attribute_type_validator(attribute_type: &AttributeType) -> Option<&CustomValidator> {
    match attribute_type.raw_shape() {
        RawShape::Reference { .. } => None,
        RawShape::String {
            identity_layer: carina_core::schema::StringIdentityParts { validate, .. },
            ..
        }
        | RawShape::Int {
            identity_layer: carina_core::schema::IntIdentityParts { validate, .. },
            ..
        }
        | RawShape::Float { validate, .. }
        | RawShape::Enum { validate, .. } => validate,
        shape => panic!("catalog constructor must carry a scalar identity, got {shape:?}"),
    }
}

#[test]
fn iam_name_catalog_rows_match_smithy_constraints_and_callbacks() {
    for (resource, maximum, constructor, sample) in [
        (
            "Role",
            64,
            "iam_role_name",
            "AWSReservedSSO_AdministratorAccess_0123456789abcdef",
        ),
        ("Policy", 128, "iam_policy_name", "carina-example-policy"),
    ] {
        let expected = format!("awscc.iam.{resource}.Name");
        let row = ScalarType::ALL
            .iter()
            .copied()
            .find(|row| row.identity().to_string() == expected)
            .unwrap_or_else(|| panic!("missing shared catalog row {expected}"));
        let identity = row.identity();
        assert_eq!(identity.provider.as_deref(), Some("awscc"));
        assert_eq!(identity.segments, ["iam", resource]);
        assert_eq!(identity.kind, "Name");
        assert_eq!(
            row.constructor_path(),
            format!("crate::schemas::catalog::{constructor}()")
        );
        assert!(!row.has_validator());
        let attr = row.attribute_type();
        let RawShape::String {
            identity_layer:
                carina_core::schema::StringIdentityParts {
                    identity: Some(actual),
                    pattern,
                    length,
                    validate: None,
                    ..
                },
            ..
        } = attr.raw_shape()
        else {
            panic!("{expected} must be a validated, identity-bearing String");
        };
        assert_eq!(actual, &identity);
        assert!(pattern.is_some());
        assert_eq!(length, Some((Some(1), Some(maximum))));
        let admitted = rootless(attr.clone());
        let validate = |value: &Value| admitted.validate(value);
        for valid in [
            sample.to_string(),
            "AZaz09_+=,.@-".to_string(),
            "a".repeat(maximum as usize),
        ] {
            let value = Value::Concrete(ConcreteValue::String(valid.clone().into()));
            assert!(validate(&value).is_ok(), "{expected}: {valid:?}");
        }
        for invalid in [
            "".to_string(),
            "bad name".to_string(),
            "a".repeat(maximum as usize + 1),
            "valid\n".to_string(),
            "rôle".to_string(),
            "invalid/".to_string(),
        ] {
            let value = Value::Concrete(ConcreteValue::String(invalid.clone().into()));
            assert!(validate(&value).is_err(), "{expected}: {invalid:?}");
        }
        assert!(validate(&Value::Concrete(ConcreteValue::Int(1))).is_err());
    }
}

#[test]
fn elbv2_matcher_catalog_rows_report_errors_and_reject_wrong_types() {
    for (row, range) in [
        (ScalarType::Elbv2TargetGroupHttpCode, "200..=599"),
        (ScalarType::Elbv2TargetGroupGrpcCode, "0..=99"),
    ] {
        let attr = row.attribute_type();
        let validate = attribute_type_validator(&attr).expect("matcher validator");
        let error = validate(&Value::Concrete(ConcreteValue::String("".into())))
            .expect_err("empty matcher codes must fail")
            .to_string();
        assert!(error.contains(&format!("ELBv2 matcher codes must be comma-separated items N or N-N (canonical decimal digits only, no leading zeros), with every N in {range} and range start <= end; no spaces or empty items")),"{error}");
        // The semantic callback reports its grammar; core adds the type identity.
        assert!(validate(&Value::Concrete(ConcreteValue::Int(200))).is_err());
    }
}

#[test]
fn logs_log_group_retention_integer_enum_matches_awscc() {
    let row = ScalarType::ALL
        .iter()
        .copied()
        .find(|row| row.identity().to_string() == "awscc.logs.LogGroup.RetentionInDays")
        .expect("shared Logs retention catalog entry");
    let values: &[i64] = &[
        1, 3, 5, 7, 14, 30, 60, 90, 120, 150, 180, 365, 400, 545, 731, 1096, 1827, 2192, 2557,
        2922, 3288, 3653,
    ];
    assert_eq!(row.identity().segments, ["logs", "LogGroup"]);
    assert_eq!(row.identity().kind, "RetentionInDays");
    assert_eq!(row.allowed_int_values(), Some(values));
    assert_eq!(
        row.constructor_path(),
        "crate::schemas::catalog::logs_log_group_retention_in_days()"
    );
    let attr = row.attribute_type();
    let RawShape::Int {
        identity_layer:
            carina_core::schema::IntIdentityParts {
                identity: Some(identity),
                range: None,
                validate: Some(validate),
                ..
            },
        ..
    } = attr.raw_shape()
    else {
        panic!("Logs retention must match AWSCC's validated Int shape");
    };
    assert_eq!(identity, &row.identity());
    assert!(row.has_validator());
    assert!(validate(&Value::Concrete(ConcreteValue::Int(30))).is_ok());
    let error = validate(&Value::Concrete(ConcreteValue::Int(2)))
        .expect_err("non-member must fail")
        .to_string();
    assert!(error.contains("allowed values"), "{error}");
    assert!(error.contains(&format!("{values:?}")), "{error}");
}

#[test]
fn standalone_constraints_are_identity_metadata() {
    let ty = ScalarType::VpcId.attribute_type();
    let RawShape::String {
        identity_layer,
        member_layer,
    } = ty.raw_shape()
    else {
        panic!("String")
    };
    assert_eq!(
        identity_layer.pattern.unwrap().as_str(),
        "^vpc-[0-9a-f]{8,}$"
    );
    assert!(member_layer.pattern.is_none() && member_layer.length.is_none());
    let zone = ScalarType::Route53HostedZoneId.attribute_type();
    let RawShape::String { identity_layer, .. } = zone.raw_shape() else {
        panic!("String")
    };
    assert_eq!(identity_layer.length, Some((None, Some(32))));
    assert!(
        identity_layer.validate.is_none(),
        "structural checks need no callback"
    );
}

#[test]
fn find_matching_enum_value_exact() {
    let values = &["Enabled", "Suspended"];
    assert_eq!(find_matching_enum_value("Enabled", values), Some("Enabled"));
    assert_eq!(find_matching_enum_value("Missing", values), None);
}

#[test]
fn find_matching_enum_value_case_insensitive() {
    let values = &["Enabled", "Suspended"];
    assert_eq!(find_matching_enum_value("enabled", values), Some("Enabled"));
}

#[test]
fn find_matching_enum_value_underscore_to_hyphen() {
    let values = &["us-east-1", "eu-west-1"];
    assert_eq!(
        find_matching_enum_value("us_east_1", values),
        Some("us-east-1")
    );
}

#[test]
fn catalog_definitions_never_contain_member_constraints() {
    let rows = AwsccType::ALL
        .iter()
        .map(|row| (format!("{row:?}"), row.attribute_type()))
        .chain(
            ScalarType::ALL
                .iter()
                .map(|row| (format!("{row:?}"), row.attribute_type())),
        )
        .chain(
            StringIdentity::ALL
                .iter()
                .map(|row| (format!("{row:?}"), row.declaration())),
        );
    let mut ints = 0;
    let mut references = 0;
    let mut examined = 0;
    for (row, ty) in rows {
        examined += 1;
        let scalar = if let RawShape::Enum { base, .. } = ty.raw_shape() {
            base
        } else {
            &ty
        };
        match scalar.raw_shape() {
            RawShape::String { member_layer, .. } => assert!(
                member_layer.pattern.is_none() && member_layer.length.is_none(),
                "{row:?}"
            ),
            RawShape::Int { member_layer, .. } => {
                ints += 1;
                assert!(member_layer.range.is_none(), "{row:?}");
            }
            RawShape::Reference { .. } => {
                references += 1;
            }
            _ => panic!("unexpected catalog carrier"),
        }
    }
    assert_eq!(
        examined,
        AwsccType::ALL.len() + ScalarType::ALL.len() + StringIdentity::ALL.len()
    );
    assert!(ints > 0 && references > 0);
}
