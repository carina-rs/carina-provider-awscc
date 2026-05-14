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

use carina_core::provider::ProviderResult;
use carina_core::resource::{ConcreteValue, Value};

use super::AwsccProvider;

/// Build the ARN for `AWS::CloudFront::Distribution`.
///
/// CloudFront is a global service: the region segment is empty.
pub(crate) fn cloudfront_distribution_arn(account_id: &str, distribution_id: &str) -> String {
    format!("arn:aws:cloudfront::{account_id}:distribution/{distribution_id}")
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
        cloudfront_distribution_arn(account_id, id),
    )))
}

/// Whether `synthesize_read_attributes` would mutate `attributes` for this
/// `(resource_type, attributes)` pair. Extracted as a pure predicate so the
/// "non-distribution reads — and distribution reads with no id — never call
/// STS" invariant can be unit-tested without instantiating a real provider.
pub(crate) fn needs_synthesis(resource_type: &str, attributes: &HashMap<String, Value>) -> bool {
    resource_type == "cloudfront.Distribution"
        && !attributes.contains_key("arn")
        && matches!(
            attributes.get("id"),
            Some(Value::Concrete(ConcreteValue::String(_)))
        )
}

impl AwsccProvider {
    /// Populate read-side attributes whose value is not returned by Cloud
    /// Control but is derivable from data we already have. Today this is
    /// just `cloudfront.Distribution.arn`; the function is a switch on
    /// resource type so adding the next case stays a few lines.
    pub(crate) async fn synthesize_read_attributes(
        &self,
        resource_type: &str,
        attributes: &mut HashMap<String, Value>,
    ) -> ProviderResult<()> {
        if !needs_synthesis(resource_type, attributes) {
            return Ok(());
        }
        let account_id = self.account_id().await?;
        if let Some(value) =
            pending_cloudfront_distribution_arn(resource_type, attributes, account_id)
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
        let arn = cloudfront_distribution_arn("123456789012", "E1U5RQF7T870K0");
        assert_eq!(
            arn,
            "arn:aws:cloudfront::123456789012:distribution/E1U5RQF7T870K0"
        );
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
            .attributes
            .get("arn")
            .expect("cloudfront.Distribution must expose an `arn` attribute");
        assert!(arn.read_only, "synthesized `arn` must be read_only");
        assert!(
            arn.provider_name.is_none(),
            "synthesized `arn` must not declare a CFN provider_name; \
             the value is built on the provider side from id + account id"
        );
    }

    fn attrs_with_id(id: &str) -> HashMap<String, Value> {
        let mut a = HashMap::new();
        a.insert(
            "id".to_string(),
            Value::Concrete(ConcreteValue::String(id.to_string())),
        );
        a
    }

    #[test]
    fn pending_arn_builds_value_when_id_present_and_arn_missing() {
        let attrs = attrs_with_id("E1U5RQF7T870K0");
        let v =
            pending_cloudfront_distribution_arn("cloudfront.Distribution", &attrs, "123456789012")
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
            Value::Concrete(ConcreteValue::String("preexisting".to_string())),
        );
        assert!(
            pending_cloudfront_distribution_arn("cloudfront.Distribution", &attrs, "123456789012",)
                .is_none()
        );
    }

    #[test]
    fn pending_arn_returns_none_when_id_missing() {
        let attrs = HashMap::new();
        assert!(
            pending_cloudfront_distribution_arn("cloudfront.Distribution", &attrs, "123456789012",)
                .is_none()
        );
    }

    #[test]
    fn pending_arn_returns_none_for_unrelated_resource_type() {
        let attrs = attrs_with_id("E1U5RQF7T870K0");
        assert!(
            pending_cloudfront_distribution_arn("cloudfront.OriginAccessControl", &attrs, "123",)
                .is_none()
        );
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
            Value::Concrete(ConcreteValue::String("preexisting".to_string())),
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
