mod common;

#[test]
fn arn_identity_is_provider_scoped() {
    common::assert_arn_identity(
        carina_provider_awscc::schemas::generated::wafv2::web_acl::arn(),
        "awscc.wafv2.WebAcl.Arn",
    );
}
