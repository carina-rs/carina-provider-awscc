//! #613 intentionally intersects each legacy slot language with canonical KMS.
//! Exercise the real type validator and finalized-plan input admission, including
//! the protocol boundary, without resolving references or contacting AWS.
#![cfg(not(target_arch = "wasm32"))]

use std::collections::{BTreeMap, HashMap};

use carina_core::effect::Effect;
use carina_core::input_rules::finalize_plan_for_test;
use carina_core::plan::{Plan, PlanErrorKind};
use carina_core::protocol_convert::{protocol_to_resource_schema, resource_schema_to_protocol};
use carina_core::provider::AppliedIdentifier;
use carina_core::resource::{ConcreteValue, ResolvedResource, Resource, State, Value};
use carina_core::schema::{
    AttributeType, InputRuleScope, PatternMatch, PatternSpec, RootlessType, SchemaRegistry,
    ValidatedConfigSchema, ValidatedResourceSchema,
};
use carina_provider_awscc::schemas::generated::{logs::log_group, scheduler::schedule};

struct Slot {
    resource: &'static str,
    field: &'static str,
    old_pattern: &'static str,
    old_length: (Option<u64>, Option<u64>),
    schema: ValidatedResourceSchema,
}

fn slots() -> [Slot; 2] {
    [
        Slot {
            resource: "scheduler.Schedule",
            field: "kms_key_arn",
            old_pattern: r"^arn:aws[a-z-]*:kms:[a-z0-9\-]+:\d{12}:(key|alias)\/[0-9a-zA-Z-_]*$",
            old_length: (Some(1), Some(2048)),
            schema: schedule::scheduler_schedule_config().schema,
        },
        Slot {
            resource: "logs.LogGroup",
            field: "kms_key_id",
            old_pattern: r"^arn:[a-z0-9-]+:kms:[a-z0-9-]+:\d{12}:(key|alias)/.+\z",
            old_length: (None, Some(256)),
            schema: log_group::logs_log_group_config().schema,
        },
    ]
}

fn string(raw: &str) -> Value {
    Value::Concrete(ConcreteValue::String(raw.into()))
}

fn rootless(ty: AttributeType) -> RootlessType {
    ValidatedConfigSchema::validate(BTreeMap::from([("value".into(), ty)]))
        .unwrap()
        .rootless_type("value")
        .unwrap()
}

/// Return both admissions independently so a type failure cannot hide lost rules.
fn admissions(
    slot: &Slot,
    schema: &ValidatedResourceSchema,
    raw: &str,
    update: bool,
) -> (bool, bool) {
    let value = string(raw);
    let type_ok = schema
        .effective_type_at_attribute_path(&[slot.field.into()])
        .unwrap()
        .validate(&value)
        .is_ok();
    let mut registry = SchemaRegistry::new();
    registry.insert("awscc", schema.clone());
    let to = ResolvedResource::resolve_raw_fixture(
        Resource::with_provider("awscc", slot.resource, "probe", None)
            .with_attribute(slot.field, value),
        &registry,
    )
    .unwrap();
    let mut plan = Plan::new();
    plan.add(if update {
        Effect::Update {
            from: Box::new(State::existing(to.id.clone(), HashMap::new()).with_identifier("probe")),
            identifier: AppliedIdentifier::new("probe").unwrap(),
            to,
            changed_attributes: vec![slot.field.into()],
        }
    } else {
        Effect::Create(to)
    });
    let result = finalize_plan_for_test(plan).check_inputs(&registry, &HashMap::new());
    if let Err(error) = &result {
        assert!(
            error.errors().iter().all(|error| matches!(
                &error.kind, PlanErrorKind::InputRule { path, .. } if path == slot.field
            )),
            "unexpected admission error: {error:?}"
        );
    }
    (type_ok, result.is_ok())
}

struct Probe {
    label: &'static str,
    raw: String,
    old: [bool; 2], // Scheduler, Logs
    canonical: bool,
}

fn arn(partition: &str, region: &str, resource: &str) -> String {
    format!("arn:{partition}:kms:{region}:123456789012:{resource}")
}

fn alias_of_length(length: usize) -> String {
    let mut value = arn("aws", "us-east-1", "alias/");
    value.push_str(&"a".repeat(length - value.len()));
    assert_eq!(value.len(), length);
    value
}

fn probes() -> Vec<Probe> {
    let uuid = "1234abcd-12ab-34cd-56ef-1234567890ab";
    let mrk = "mrk-0123456789abcdef0123456789abcdef";
    let mut probes = Vec::new();
    for (partition, region) in [
        ("aws", "us-east-1"),
        ("aws-cn", "cn-north-1"),
        ("aws-us-gov", "us-gov-west-1"),
        ("aws-iso", "us-iso-east-1"),
        ("aws-iso-b", "us-isob-east-1"),
        ("aws-iso-e", "eu-isoe-west-1"),
        ("aws-iso-f", "us-isof-south-1"),
        ("aws-eusc", "eusc-de-east-1"),
    ] {
        for resource in [
            format!("key/{uuid}"),
            format!("key/{mrk}"),
            "alias/customer_1-A".into(),
        ] {
            probes.push(Probe {
                label: "still accepted: partition/key/alias/mrk",
                raw: arn(partition, region, &resource),
                old: [true, true],
                canonical: true,
            });
        }
    }
    for (label, raw, old, canonical) in [
        (
            "uppercase UUID",
            arn("aws", "us-east-1", &format!("key/{}", uuid.to_uppercase())),
            [true, true],
            true,
        ),
        (
            "uppercase mrk hex",
            arn(
                "aws",
                "us-east-1",
                "key/mrk-0123456789ABCDEF0123456789ABCDEF",
            ),
            [true, true],
            true,
        ),
        (
            "newly rejected (intended): key/x",
            arn("aws", "us-east-1", "key/x"),
            [true, true],
            false,
        ),
        (
            "empty key suffix",
            arn("aws", "us-east-1", "key/"),
            [true, false],
            false,
        ),
        (
            "empty alias suffix",
            arn("aws", "us-east-1", "alias/"),
            [true, false],
            false,
        ),
        (
            "canonical nested alias: no new Scheduler acceptance",
            arn("aws", "us-east-1", "alias/team/key"),
            [false, true],
            true,
        ),
        (
            "canonical alias colon: no new Scheduler acceptance",
            arn("aws", "us-east-1", "alias/team:key"),
            [false, true],
            true,
        ),
        (
            "noncanonical alias punctuation",
            arn("aws", "us-east-1", "alias/customer.key"),
            [false, true],
            false,
        ),
        (
            "noncanonical alias Unicode",
            arn("aws", "us-east-1", "alias/鍵"),
            [false, true],
            false,
        ),
        (
            "unknown partition",
            arn("aws-unknown", "us-east-1", "alias/customer"),
            [true, true],
            false,
        ),
        (
            "other partition",
            arn("other", "us-east-1", "alias/customer"),
            [false, true],
            false,
        ),
        (
            "odd but canonical region",
            arn("aws", "zz-odd-999", "alias/customer"),
            [true, true],
            true,
        ),
        (
            "noncanonical region",
            arn("aws", "x", "alias/customer"),
            [true, true],
            false,
        ),
        (
            "uppercase region",
            arn("aws", "US-east-1", "alias/customer"),
            [false, false],
            false,
        ),
        (
            "bare UUID: no new acceptance",
            uuid.into(),
            [false, false],
            true,
        ),
        (
            "bare mrk: no new acceptance",
            mrk.into(),
            [false, false],
            true,
        ),
        (
            "bare alias: no new acceptance",
            "alias/customer".into(),
            [false, false],
            true,
        ),
        ("empty", "".into(), [false, false], false),
        ("one character", "x".into(), [false, false], false),
        (
            "trailing newline: strict Logs \\z",
            format!("{}\n", arn("aws", "us-east-1", "alias/customer")),
            [false, false],
            false,
        ),
        ("Logs max 256", alias_of_length(256), [true, true], true),
        (
            "Logs 257: canonical but slot still rejects",
            alias_of_length(257),
            [true, false],
            true,
        ),
        (
            "Scheduler max 2048",
            alias_of_length(2048),
            [true, false],
            true,
        ),
        (
            "Scheduler 2049",
            alias_of_length(2049),
            [false, false],
            false,
        ),
    ] {
        probes.push(Probe {
            label,
            raw,
            old,
            canonical,
        });
    }
    probes
}

#[test]
fn both_slots_accept_exactly_canonical_intersected_with_the_old_slot() {
    let canonical = rootless(carina_aws_types::kms_key_reference());
    for (index, slot) in slots().iter().enumerate() {
        let old = rootless(AttributeType::reference(
            carina_aws_types::AwsType::KmsKeyReference.identity(),
            Some(PatternSpec::new(slot.old_pattern, PatternMatch::Full)),
            Some(slot.old_length),
        ));
        let wire = protocol_to_resource_schema(&resource_schema_to_protocol(&slot.schema).unwrap())
            .unwrap();
        for schema in [&slot.schema, &wire] {
            assert!(!schema.as_schema().input_rules().is_empty());
            assert!(
                schema
                    .as_schema()
                    .input_rules()
                    .iter()
                    .all(|rule| rule.scope() == InputRuleScope::EveryWrite)
            );
            for probe in probes() {
                let value = string(&probe.raw);
                let before = old.validate(&value).is_ok();
                let canonical = canonical.validate(&value).is_ok();
                let diagnostic = format!(
                    "{}: {} (length {})",
                    slot.resource,
                    probe.label,
                    probe.raw.len()
                );
                assert_eq!(before, probe.old[index], "old acceptance: {diagnostic}");
                assert_eq!(
                    canonical, probe.canonical,
                    "canonical acceptance: {diagnostic}"
                );
                for update in [false, true] {
                    let (type_ok, rules_ok) = admissions(slot, schema, &probe.raw, update);
                    assert_eq!(type_ok, canonical, "schema type: {diagnostic}");
                    let after = type_ok && rules_ok;
                    assert_eq!(
                        after,
                        before && canonical,
                        "intersection, update={update}: {diagnostic}"
                    );
                    // Canonical probes exercise the slot rules without a type
                    // failure hiding loss of the old pattern or length bound.
                    if canonical {
                        assert_eq!(rules_ok, before, "slot rule: {diagnostic}");
                    }
                }
            }
        }
    }
}

#[tokio::test]
async fn provider_reads_preserve_noncanonical_kms_values() {
    use carina_core::provider::{Provider, ReadOutcome, ReadRequest};
    use carina_provider_awscc::{AwsccProvider, BoundedSdkConfig, provider::AwsccProviderConfig};
    use std::{future::Future, pin::Pin, sync::Arc};
    use winterbaume_core::{MockAws, MockRequest, MockResponse, MockService};

    struct ReadService(String);
    impl MockService for ReadService {
        fn service_name(&self) -> &str {
            "cloudcontrolapi"
        }
        fn url_patterns(&self) -> Vec<&str> {
            vec![r"^https://cloudcontrolapi\.[^/]+\.amazonaws\.com"]
        }
        fn handle(
            &self,
            request: MockRequest,
        ) -> Pin<Box<dyn Future<Output = MockResponse> + Send + '_>> {
            assert!(
                request.headers["x-amz-target"]
                    .to_str()
                    .unwrap()
                    .ends_with(".GetResource")
            );
            let props = serde_json::json!({
                (self.0.as_str()): "arn:aws:kms:us-east-1:123456789012:key/x"
            });
            Box::pin(std::future::ready(MockResponse::json(200, serde_json::json!({
                "ResourceDescription": { "Identifier": "probe", "Properties": props.to_string() }
            }).to_string())))
        }
    }
    for slot in slots() {
        let property = slot.schema.as_schema().attributes()[slot.field]
            .provider_name()
            .unwrap();
        let mock = MockAws::builder()
            .with_service(Arc::new(ReadService(property.into())))
            .build();
        let provider = AwsccProvider::from_sdk_config(
            BoundedSdkConfig::for_test(mock.sdk_config("us-east-1").await),
            &AwsccProviderConfig::default(),
        )
        .await;
        let resource: Resource = Resource::with_provider("awscc", slot.resource, "probe", None);
        let outcome = Provider::read(
            &provider,
            &resource.id,
            &AppliedIdentifier::new("probe").unwrap(),
            ReadRequest,
        )
        .await
        .expect("READ must not validate desired-input constraints");
        let ReadOutcome::Found { state } = outcome else {
            panic!("mock must return found state")
        };
        assert_eq!(
            state.attributes()[slot.field],
            string("arn:aws:kms:us-east-1:123456789012:key/x").to_snapshot()
        );
        // The provider normalizer uses the same state-only canonicalizer.
        let canonical: HashMap<_, _> =
            carina_core::resource::CanonicalComparisonAttributes::canonicalize(
                state.attributes().clone(),
                slot.schema.as_schema(),
            )
            .into_iter()
            .collect();
        assert_eq!(
            canonical[slot.field],
            string("arn:aws:kms:us-east-1:123456789012:key/x").to_snapshot(),
        );
    }
}
