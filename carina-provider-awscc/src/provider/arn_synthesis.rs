//! Synthesize ARNs for resources whose CloudFormation type schema does not
//! expose them as a top-level property.
//!
//! Cloud Control API returns only what the resource type's CFN schema
//! declares; some resources (notably `AWS::CloudFront::Distribution`) omit
//! `Arn` from their schema even though the ARN exists and has a
//! deterministic format. Downstream `.crn` references like
//! `distribution.arn` would otherwise fail at validate time. This module
//! reconstructs those ARNs from the resource id plus the caller's account
//! id.

use std::collections::HashMap;

use super::read_back::ReadFailure;
use crate::partition::Partition;
use carina_core::resource::{ConcreteValue, Value};

use super::AwsccProvider;

/// Build the ARN for `AWS::CloudFront::Distribution`.
///
/// CloudFront is a global service: the region segment is empty.
pub(crate) fn cloudfront_distribution_arn(
    partition: &str,
    account_id: &str,
    distribution_id: &str,
) -> String {
    format!("arn:{partition}:cloudfront::{account_id}:distribution/{distribution_id}")
}

/// Build the ARN for `AWS::IAM::Role`.
///
/// IAM is a global service: the region segment is empty.
pub(crate) fn iam_role_arn(
    partition: &str,
    account_id: &str,
    path: &str,
    role_name: &str,
) -> String {
    format!("arn:{partition}:iam::{account_id}:role{path}{role_name}")
}

pub(crate) fn partition_for_region(region: Option<&str>) -> &'static str {
    region.map_or(Partition::Aws, Partition::from_region).id()
}

/// If `attributes` is missing the `arn` slot for a
/// `cloudfront.Distribution` read-back but `id` is present, return the
/// synthesized ARN value to insert. Returns `None` when nothing is
/// needed (different resource, `arn` already populated, or `id`
/// missing/non-string) — in that case the caller should skip the STS
/// call entirely.
pub(crate) fn pending_cloudfront_distribution_arn(
    resource_type: &str,
    attributes: &HashMap<String, Value>,
    partition: &str,
    account_id: &str,
) -> Option<Value> {
    if resource_type != "cloudfront.Distribution" {
        return None;
    }
    if attributes.contains_key("arn") {
        return None;
    }
    let Some(Value::Concrete(ConcreteValue::String(id))) = attributes.get("id") else {
        return None;
    };
    Some(Value::Concrete(ConcreteValue::String(
        cloudfront_distribution_arn(partition, account_id, id.expose_plaintext()).into(),
    )))
}

/// If `attributes` is missing the `arn` slot for an `iam.Role` read-back
/// but `role_name` is present, return the synthesized ARN value to insert.
/// The read-back declaration controls completeness when synthesis needs more data.
pub(crate) fn pending_iam_role_arn(
    resource_type: &str,
    attributes: &HashMap<String, Value>,
    partition: &str,
    account_id: &str,
) -> Option<Value> {
    if resource_type != "iam.Role" {
        return None;
    }
    if attributes.contains_key("arn") {
        return None;
    }
    let role_name = string_attr(attributes, "role_name")?;
    let path = string_attr(attributes, "path").unwrap_or("/");
    Some(Value::Concrete(ConcreteValue::String(
        iam_role_arn(partition, account_id, path, role_name).into(),
    )))
}

fn string_attr<'a>(attributes: &'a HashMap<String, Value>, name: &str) -> Option<&'a str> {
    match attributes.get(name) {
        Some(Value::Concrete(ConcreteValue::String(value))) => Some(value.expose_plaintext()),
        _ => None,
    }
}

/// Whether `synthesize_read_attributes` would mutate `attributes` for this
/// `(resource_type, attributes)` pair. Extracted as a pure predicate so the
/// "non-distribution reads — and distribution reads with no id — never call
/// STS" invariant can be unit-tested without instantiating a real provider.
pub(crate) fn needs_synthesis(resource_type: &str, attributes: &HashMap<String, Value>) -> bool {
    if attributes.contains_key("arn") {
        return false;
    }
    match resource_type {
        "cloudfront.Distribution" => matches!(
            attributes.get("id"),
            Some(Value::Concrete(ConcreteValue::String(_)))
        ),
        "iam.Role" => string_attr(attributes, "role_name").is_some(),
        _ => false,
    }
}

impl AwsccProvider {
    /// Populate read-side attributes whose value is not returned by Cloud
    /// Control but is derivable from data we already have.
    pub(crate) async fn synthesize_read_attributes(
        &self,
        resource_type: &str,
        attributes: &mut HashMap<String, Value>,
    ) -> Result<(), ReadFailure> {
        if !needs_synthesis(resource_type, attributes) {
            return Ok(());
        }
        let account_id = self.account_id().await?;
        let partition = partition_for_region(
            self.cloudcontrol_client
                .inner()
                .config()
                .region()
                .map(|region| region.as_ref()),
        );
        if let Some(value) =
            pending_cloudfront_distribution_arn(resource_type, attributes, partition, account_id)
        {
            attributes.insert("arn".to_string(), value);
        }
        if let Some(value) = pending_iam_role_arn(resource_type, attributes, partition, account_id)
        {
            attributes.insert("arn".to_string(), value);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schemas::generated::get_config_by_type;

    #[test]
    fn cloudfront_distribution_arn_uses_empty_region_segment() {
        let arn = cloudfront_distribution_arn("aws", "123456789012", "E1U5RQF7T870K0");
        assert_eq!(
            arn,
            "arn:aws:cloudfront::123456789012:distribution/E1U5RQF7T870K0"
        );
    }

    #[test]
    fn iam_role_arn_uses_empty_region_segment_and_path() {
        let arn = iam_role_arn("aws-us-gov", "123456789012", "/service-role/", "flow-log");
        assert_eq!(
            arn,
            "arn:aws-us-gov:iam::123456789012:role/service-role/flow-log"
        );
    }

    #[test]
    fn partition_for_region_uses_aws_partition_by_region_family() {
        assert_eq!(partition_for_region(Some("us-east-1")), "aws");
        assert_eq!(partition_for_region(Some("ap-northeast-1")), "aws");
        assert_eq!(partition_for_region(Some("us-gov-west-1")), "aws-us-gov");
        assert_eq!(partition_for_region(Some("cn-north-1")), "aws-cn");
        assert_eq!(partition_for_region(None), "aws");
    }

    #[test]
    fn synthesized_arns_use_every_region_partition() {
        let mut failures = Vec::new();
        for (region, expected) in [
            ("us-east-1", "aws"),
            ("ap-northeast-1", "aws"),
            ("cn-north-1", "aws-cn"),
            ("us-gov-west-1", "aws-us-gov"),
            ("us-iso-east-1", "aws-iso"),
            ("us-isob-east-1", "aws-iso-b"),
            ("eu-isoe-west-1", "aws-iso-e"),
            ("us-isof-south-1", "aws-iso-f"),
            ("eusc-de-east-1", "aws-eusc"),
        ] {
            let partition = partition_for_region(Some(region));
            if partition != expected {
                failures.push(format!("{region}: expected {expected}, got {partition}"));
            }
            for (actual, expected_arn) in [
                (
                    cloudfront_distribution_arn(partition, "123456789012", "E123"),
                    format!("arn:{expected}:cloudfront::123456789012:distribution/E123"),
                ),
                (
                    iam_role_arn(partition, "123456789012", "/service-role/", "example"),
                    format!("arn:{expected}:iam::123456789012:role/service-role/example"),
                ),
            ] {
                if actual != expected_arn {
                    failures.push(format!("{region}: expected {expected_arn}, got {actual}"));
                }
            }
        }
        assert!(failures.is_empty(), "{}", failures.join("\n"));
    }

    #[test]
    fn synthesized_iam_roles_validate_in_every_partition() {
        use carina_core::schema::{CompiledPattern, RawShape};
        let ty = crate::schemas::generated::iam::role::arn();
        let RawShape::String {
            pattern: Some(pattern),
            validate: Some(validate),
            ..
        } = ty.raw_shape()
        else {
            panic!("IAM role ARN must have a pattern and callback");
        };
        let pattern = CompiledPattern::try_new(pattern).unwrap();
        let mut seen = std::collections::BTreeSet::new();
        let mut failures = Vec::new();
        for region in [
            "us-east-1",
            "cn-north-1",
            "us-gov-west-1",
            "us-iso-east-1",
            "us-isob-east-1",
            "eu-isoe-west-1",
            "us-isof-south-1",
            "eusc-de-east-1",
        ] {
            let partition = partition_for_region(Some(region));
            seen.insert(partition);
            let arn = iam_role_arn(partition, "123456789012", "/service-role/", "example");
            if !pattern.is_match(&arn) {
                failures.push(format!("{region}: pattern rejected {arn}"));
            }
            if let Err(error) = validate(&Value::Concrete(ConcreteValue::String(arn.into()))) {
                failures.push(format!("{region}: {error}"));
            }
        }
        assert_eq!(
            seen,
            Partition::ALL.map(Partition::id).into_iter().collect()
        );
        assert!(failures.is_empty(), "{}", failures.join("\n"));
    }

    /// `AWS::CloudFront::Distribution`'s CFN schema does not expose `Arn`,
    /// but downstream `.crn` files reference `distribution.arn` (e.g. for
    /// CloudFront/S3 OAC bucket policies). The schema must therefore
    /// declare `arn` as a read-only attribute the provider synthesizes.
    /// Closes carina-rs/carina-provider-awscc#240.
    #[test]
    fn cloudfront_distribution_schema_exposes_arn_attribute() {
        let config = get_config_by_type("cloudfront.Distribution")
            .expect("cloudfront.Distribution schema must be registered");
        let arn = config
            .schema
            .as_schema()
            .attributes()
            .get("arn")
            .expect("cloudfront.Distribution must expose an `arn` attribute");
        assert!(arn.is_read_only(), "synthesized `arn` must be read_only");
        assert!(
            arn.provider_name().is_none(),
            "synthesized `arn` must not declare a CFN provider_name; \
             the value is built on the provider side from id + account id"
        );
    }

    fn attrs_with_id(id: &str) -> HashMap<String, Value> {
        let mut a = HashMap::new();
        a.insert(
            "id".to_string(),
            Value::Concrete(ConcreteValue::String(id.to_string().into())),
        );
        a
    }

    #[test]
    fn pending_arn_builds_value_when_id_present_and_arn_missing() {
        let attrs = attrs_with_id("E1U5RQF7T870K0");
        let v = pending_cloudfront_distribution_arn(
            "cloudfront.Distribution",
            &attrs,
            "aws",
            "123456789012",
        )
        .expect("ARN must be synthesizable when id is present");
        match v {
            Value::Concrete(ConcreteValue::String(s)) => assert_eq!(
                s,
                "arn:aws:cloudfront::123456789012:distribution/E1U5RQF7T870K0"
            ),
            other => panic!("expected concrete string ARN, got {other:?}"),
        }
    }

    #[test]
    fn pending_arn_returns_none_when_arn_already_populated() {
        let mut attrs = attrs_with_id("E1U5RQF7T870K0");
        attrs.insert(
            "arn".to_string(),
            Value::Concrete(ConcreteValue::String("preexisting".to_string().into())),
        );
        assert!(
            pending_cloudfront_distribution_arn(
                "cloudfront.Distribution",
                &attrs,
                "aws",
                "123456789012",
            )
            .is_none()
        );
    }

    #[test]
    fn pending_arn_returns_none_when_id_missing() {
        let attrs = HashMap::new();
        assert!(
            pending_cloudfront_distribution_arn(
                "cloudfront.Distribution",
                &attrs,
                "aws",
                "123456789012",
            )
            .is_none()
        );
    }

    #[test]
    fn pending_arn_returns_none_for_unrelated_resource_type() {
        let attrs = attrs_with_id("E1U5RQF7T870K0");
        assert!(
            pending_cloudfront_distribution_arn(
                "cloudfront.OriginAccessControl",
                &attrs,
                "aws",
                "123",
            )
            .is_none()
        );
    }

    fn attrs_with_role(path: Option<&str>, role_name: Option<&str>) -> HashMap<String, Value> {
        let mut attrs = HashMap::new();
        if let Some(path) = path {
            attrs.insert(
                "path".to_string(),
                Value::Concrete(ConcreteValue::String(path.to_string().into())),
            );
        }
        if let Some(role_name) = role_name {
            attrs.insert(
                "role_name".to_string(),
                Value::Concrete(ConcreteValue::String(role_name.to_string().into())),
            );
        }
        attrs
    }

    #[test]
    fn pending_iam_role_arn_builds_value_when_role_name_present() {
        let attrs = attrs_with_role(Some("/service-role/"), Some("flow-log-role"));
        let v = pending_iam_role_arn("iam.Role", &attrs, "aws-cn", "123456789012")
            .expect("ARN must be synthesizable when role_name is present");
        match v {
            Value::Concrete(ConcreteValue::String(s)) => assert_eq!(
                s,
                "arn:aws-cn:iam::123456789012:role/service-role/flow-log-role"
            ),
            other => panic!("expected concrete string ARN, got {other:?}"),
        }
    }

    #[test]
    fn pending_iam_role_arn_defaults_path_to_slash() {
        let attrs = attrs_with_role(None, Some("flow-log-role"));
        let v = pending_iam_role_arn("iam.Role", &attrs, "aws", "123456789012")
            .expect("ARN must be synthesizable when role_name is present");
        match v {
            Value::Concrete(ConcreteValue::String(s)) => {
                assert_eq!(s, "arn:aws:iam::123456789012:role/flow-log-role");
            }
            other => panic!("expected concrete string ARN, got {other:?}"),
        }
    }

    /// `needs_synthesis` is the gate `synthesize_read_attributes` checks
    /// before touching STS. These cases must short-circuit to `false` so
    /// reads of unrelated resource types — or distribution reads that
    /// arrive with no `id` — never trigger a `sts:GetCallerIdentity` call.
    #[test]
    fn needs_synthesis_false_for_unrelated_resource_type() {
        let attrs = attrs_with_id("E1U5RQF7T870K0");
        assert!(!needs_synthesis("s3.Bucket", &attrs));
        assert!(!needs_synthesis("cloudfront.OriginAccessControl", &attrs));
    }

    #[test]
    fn needs_synthesis_false_when_arn_already_present() {
        let mut attrs = attrs_with_id("E1U5RQF7T870K0");
        attrs.insert(
            "arn".to_string(),
            Value::Concrete(ConcreteValue::String("preexisting".to_string().into())),
        );
        assert!(!needs_synthesis("cloudfront.Distribution", &attrs));
    }

    #[test]
    fn needs_synthesis_false_when_id_missing() {
        let attrs = HashMap::new();
        assert!(!needs_synthesis("cloudfront.Distribution", &attrs));
    }

    #[test]
    fn needs_synthesis_true_for_distribution_with_id_and_no_arn() {
        let attrs = attrs_with_id("E1U5RQF7T870K0");
        assert!(needs_synthesis("cloudfront.Distribution", &attrs));
    }
}
