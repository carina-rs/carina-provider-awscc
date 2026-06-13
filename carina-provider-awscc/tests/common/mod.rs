use carina_core::executor::normalized::{NormalizedResource, apply_desired_normalization};
use carina_core::resource::Resource;
use carina_core::schema::AttributeType;
use carina_core::schema::SchemaRegistry;
use carina_provider_awscc::AwsccNormalizer;

#[allow(dead_code)]
pub fn assert_arn_identity(t: AttributeType, expected: &str) {
    let carina_core::schema::RawShape::String { identity, .. } = t.raw_shape() else {
        panic!("arn() should be a refined string");
    };
    assert_eq!(identity.map(|id| id.to_string()).as_deref(), Some(expected));
}

#[allow(dead_code)]
pub async fn normalize_resource(resource: Resource) -> NormalizedResource {
    apply_desired_normalization(resource, &[], &AwsccNormalizer, &[], &SchemaRegistry::new()).await
}
