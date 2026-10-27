mod common;

use std::collections::HashMap;

use carina_core::resource::{ConcreteValue, Value};
use carina_core::schema::TypeError;
use carina_provider_awscc::schemas::generated::s3::bucket::s3_bucket_config;
use indexmap::IndexMap;

fn string(value: &str) -> Value {
    Value::Concrete(ConcreteValue::String(value.to_string().into()))
}

fn map(entries: impl IntoIterator<Item = (&'static str, Value)>) -> Value {
    Value::Concrete(ConcreteValue::Map(
        entries
            .into_iter()
            .map(|(key, value)| (key.to_string(), value))
            .collect::<IndexMap<_, _>>(),
    ))
}

#[test]
fn arn_identity_is_provider_scoped() {
    common::assert_arn_identity(
        carina_provider_awscc::schemas::generated::s3::bucket::arn(),
        "aws.s3.Bucket.Arn",
    );
}

#[test]
fn nested_metadata_table_arn_is_provider_populated_and_rejects_assignment() {
    let config = s3_bucket_config();
    let attributes = HashMap::from([(
        "metadata_table_configuration".to_string(),
        map([(
            "s3_tables_destination",
            map([
                (
                    "table_bucket_arn",
                    string("arn:aws:s3tables:us-east-1:123456789012:bucket/example"),
                ),
                ("table_name", string("example")),
                (
                    "table_arn",
                    string("arn:aws:s3tables:us-east-1:123456789012:bucket/example/table/example"),
                ),
            ]),
        )]),
    )]);

    let errors = config
        .schema
        .validate(&attributes)
        .expect_err("assigning S3TablesDestination.TableArn must fail validation");
    assert!(
        matches!(
            errors.as_slice(),
            [TypeError::StructFieldError { field, inner }]
                if field == "s3_tables_destination"
                    && matches!(
                        inner.as_ref(),
                        TypeError::StructFieldError { field, inner }
                            if field == "table_arn"
                                && matches!(
                                    inner.as_ref(),
                                    TypeError::ReadOnlyAttribute { name }
                                        if name == "table_arn"
                                )
                    )
        ),
        "expected a nested provider-populated table_arn error, got {errors:?}"
    );
}
