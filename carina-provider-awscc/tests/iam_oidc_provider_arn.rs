mod common;

use carina_core::resource::{ConcreteValue, Value};

#[test]
fn arn_identity_is_provider_scoped() {
    common::assert_arn_identity(
        carina_provider_awscc::schemas::generated::iam::oidc_provider::arn(),
        "aws.iam.OidcProvider.Arn",
    );
}

#[test]
fn arn_accepts_eks_multi_segment_oidc_provider() {
    let t = carina_provider_awscc::schemas::generated::iam::oidc_provider::arn();
    let carina_core::schema::RawShape::Custom { validate, .. } = t.raw_shape() else {
        panic!("arn() should be custom");
    };
    let v = Value::Concrete(ConcreteValue::String(
        "arn:aws:iam::123456789012:oidc-provider/oidc.eks.us-east-1.amazonaws.com/id/AAAAAAAA000000"
            .to_string(),
    ));
    assert!(validate(&v).is_ok());
}
