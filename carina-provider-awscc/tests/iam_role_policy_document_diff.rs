use carina_core::diff_helpers::resource_attribute_values_equal;
use carina_core::resource::{ConcreteValue, ResourceId, Value};
use carina_provider_awscc::schemas::generated::iam::role::iam_role_config;
use indexmap::IndexMap;

fn string(value: &str) -> Value {
    Value::Concrete(ConcreteValue::String(value.to_string().into()))
}

fn string_list(values: &[&str]) -> Value {
    Value::Concrete(ConcreteValue::StringList(
        values
            .iter()
            .map(|value| (*value).to_string().into())
            .collect(),
    ))
}

fn assume_role_policy_document(actions: &[&str], principal_arns: &[&str]) -> Value {
    let principal = Value::Concrete(ConcreteValue::Map(IndexMap::from([(
        "aws".to_string(),
        string_list(principal_arns),
    )])));
    let statement = Value::Concrete(ConcreteValue::Map(IndexMap::from([
        ("effect".to_string(), string("Allow")),
        ("action".to_string(), string_list(actions)),
        ("principal".to_string(), principal),
    ])));

    Value::Concrete(ConcreteValue::Map(IndexMap::from([
        ("version".to_string(), string("2012-10-17")),
        (
            "statement".to_string(),
            Value::Concrete(ConcreteValue::List(vec![statement])),
        ),
    ])))
}

#[test]
fn iam_role_policy_document_diff_ignores_unordered_value_list_reordering() {
    let schema = iam_role_config()
        .schema
        .validate()
        .expect("iam.Role schema must be valid");
    let original = assume_role_policy_document(
        &["sts:AssumeRole", "sts:TagSession"],
        &[
            "arn:aws:iam::111122223333:role/alpha",
            "arn:aws:iam::444455556666:role/beta",
        ],
    );
    let reordered = assume_role_policy_document(
        &["sts:TagSession", "sts:AssumeRole"],
        &[
            "arn:aws:iam::444455556666:role/beta",
            "arn:aws:iam::111122223333:role/alpha",
        ],
    );
    let changed = assume_role_policy_document(
        &["sts:TagSession", "sts:AssumeRole"],
        &[
            "arn:aws:iam::444455556666:role/beta",
            "arn:aws:iam::777788889999:role/gamma",
        ],
    );
    let registry = carina_provider_awscc::schemas::schema_registry();
    let router = carina_provider_awscc::test_comparison_router();
    let id = ResourceId::with_provider_identity("awscc", "iam.Role", "policy-diff", None);
    let comparison = router
        .comparison_for(&registry, &id)
        .expect("IAM Role comparison context");

    assert!(
        resource_attribute_values_equal(
            &original,
            &reordered,
            Some(&schema),
            "assume_role_policy_document",
            comparison.context(),
        ),
        "IAM action and principal AWS lists must compare without regard to order"
    );
    assert!(
        !resource_attribute_values_equal(
            &original,
            &changed,
            Some(&schema),
            "assume_role_policy_document",
            comparison.context(),
        ),
        "a changed principal ARN must remain a real policy difference"
    );
}
