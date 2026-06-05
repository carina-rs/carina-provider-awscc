mod common;

#[test]
fn arn_identity_is_provider_scoped() {
    common::assert_arn_identity(
        carina_provider_awscc::schemas::generated::organizations::account::arn(),
        "awscc.organizations.Account.Arn",
    );
}
