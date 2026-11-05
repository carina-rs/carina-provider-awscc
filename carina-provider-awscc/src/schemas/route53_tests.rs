use crate::test_support::try_validate_fixtures;

#[test]
fn hosted_zone_vpcs_are_optional_but_cannot_be_explicitly_empty() {
    for (label, vpcs, valid) in [
        ("omitted", "", true),
        (
            "single",
            r#"vp_cs = [{
  vpc_id = 'vpc-0123456789abcdef0'
  vpc_region = ap_northeast_1
}]"#,
            true,
        ),
        ("empty", "vp_cs = []", false),
    ] {
        let fixture = format!(
            r#"
provider awscc {{ region = awscc.Region.ap_northeast_1 }}
awscc.route53.HostedZone {{
  name = 'example.com'
  {vpcs}
}}
"#
        );
        let result = try_validate_fixtures(&[("route53_vpcs.crn", &fixture)]);
        if valid {
            result.unwrap_or_else(|error| panic!("{label}: {error}"));
        } else {
            let error = result.expect_err("an explicitly empty VPCs array must fail validation");
            assert!(error.contains("vp_cs"), "{error}");
        }
    }
}
