use carina_core::executor::prepare_provider_ready_resource;
use carina_core::parser::ProviderConfig;
use carina_core::resource::{ConcreteValue, Resource, Value};
use carina_core::schema::{RawShape, TypeError};
use indexmap::IndexMap;

use crate::test_support::{
    field, string, try_validate_fixtures, validate_fixtures, with_preparation_and_configs,
};

#[test]
fn every_registered_tag_map_preserves_its_cfn_string_constraints() {
    for config in super::generated::configs()
        .iter()
        .filter(|config| config.tags.is_some())
    {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(format!(
            "../cfn-schema-cache/{}.json",
            config.aws_type_name.replace("::", "__")
        ));
        let source: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
        let mut tag = &source["properties"]["Tags"];
        if let Some(items) = tag.get("items") {
            tag = items;
        }
        while let Some(reference) = tag.get("$ref").and_then(serde_json::Value::as_str) {
            tag = source
                .pointer(reference.strip_prefix('#').unwrap())
                .unwrap();
        }
        let tags = field(&config.schema, "tags");
        let RawShape::Map {
            key,
            value,
            constraints: _,
        } = tags.as_type().raw_shape()
        else {
            panic!("{} must retain a map", config.aws_type_name);
        };
        let (key_source, value_source) = if let Some(properties) = tag.get("properties") {
            (properties["Key"].clone(), properties["Value"].clone())
        } else {
            let (pattern, value) = tag["patternProperties"]
                .as_object()
                .unwrap()
                .iter()
                .next()
                .unwrap();
            (serde_json::json!({"pattern": pattern}), value.clone())
        };
        for (name, source, generated) in [("Key", key_source, key), ("Value", value_source, value)]
        {
            let RawShape::String {
                identity,
                pattern,
                length,
                validate,
                ..
            } = generated.raw_shape()
            else {
                panic!("{} {name} must remain a string", config.aws_type_name);
            };
            let min = source["minLength"].as_u64().filter(|min| *min > 0);
            let max = if config.aws_type_name.starts_with("AWS::EC2::") {
                // #525: CreateTags, carina-test-008, ap-northeast-1,
                // 2026-11-06: keys 128 and values 256 accepted; 129/257 rejected.
                Some(
                    source["maxLength"]
                        .as_u64()
                        .unwrap_or(if name == "Key" { 128 } else { 256 }),
                )
            } else {
                source["maxLength"].as_u64()
            };
            let expected_length = (min.is_some() || max.is_some()).then_some((min, max));
            let expected_pattern = if config.aws_type_name == "AWS::WAFv2::WebACL" {
                // #525 adds the live-confirmed API pattern to both Tag leaves.
                Some(super::pattern::ApprovedPattern::whole(
                    r"^[\p{L}\p{Z}\p{N}_.:/=+\-@]*$",
                ))
            } else {
                source["pattern"]
                    .as_str()
                    .map(super::pattern::ApprovedPattern::whole)
            };
            assert!(
                identity.is_none() && validate.is_none(),
                "{} {name}",
                config.aws_type_name
            );
            assert_eq!(length, expected_length, "{} {name}", config.aws_type_name);
            assert_eq!(
                pattern,
                expected_pattern
                    .as_ref()
                    .map(super::pattern::ApprovedPattern::as_str),
                "{} {name}",
                config.aws_type_name
            );
        }
    }
}

fn fixture(resource: &str, key: &str, value: &str) -> String {
    let required = match resource {
        "sso.PermissionSet" => {
            "name = 'tag-test'\ninstance_arn = 'arn:aws:sso:::instance/ssoins-0123456789abcdef'"
        }
        "cloudfront.Function" => {
            "name = 'tag-test'\nfunction_code = 'function handler(event) { return event.request; }'\nfunction_config = { comment = 'tags', runtime = cloudfront_js_2_0 }"
        }
        "organizations.Account" => "account_name = 'tag-test'\nemail = 'tags@example.com'",
        "ssm.Parameter" => "type = string\nvalue = 'parameter-value'",
        _ => "",
    };
    format!(
        "provider awscc {{ region = awscc.Region.ap_northeast_1 }}\nawscc.{resource} {{\n{required}\ntags = {{ '{key}' = '{value}' }}\n}}"
    )
}

fn assert_invalid(resource: &str, key: &str, value: &str) {
    let source = fixture(resource, key, value);
    let error = try_validate_fixtures(&[("invalid_tags.crn", &source)])
        .expect_err("the resource's CFN Tag constraints must reject this entry");
    assert!(error.contains("tags"), "{resource}: {error}");
    assert!(error.contains(key), "{resource}: {error}");
}

#[test]
fn ecr_tag_entries_enforce_key_and_value_length_endpoints() {
    for (key, value) in [
        ("k".to_string(), "v".to_string()),
        ("k".repeat(127), "v".repeat(255)),
        ("é".repeat(127), "名".repeat(255)),
    ] {
        validate_fixtures(&[("ecr_tags.crn", &fixture("ecr.Repository", &key, &value))]);
    }
    for (key, value) in [
        ("k".repeat(128), "v".to_string()),
        (String::new(), "v".to_string()),
        ("k".to_string(), "v".repeat(256)),
        ("k".to_string(), String::new()),
    ] {
        assert_invalid("ecr.Repository", &key, &value);
    }
}

#[test]
fn invalid_tag_key_is_a_map_key_error() {
    let schema = super::generated::get_config_by_type("ecr.Repository").unwrap();
    let tags = field(&schema.schema, "tags");
    let key = "k".repeat(128);
    let entry = Value::Concrete(ConcreteValue::Map(IndexMap::from([(
        key.clone(),
        string("valid"),
    )])));
    let error = tags
        .validate(&entry)
        .expect_err("128 exceeds CFN maxLength 127");
    assert!(matches!(error, TypeError::MapKeyError { key: actual, .. } if actual == key));
}

#[test]
fn tag_value_minimum_is_resource_specific() {
    validate_fixtures(&[
        ("kms_empty_tag.crn", &fixture("kms.Key", "team", "")),
        ("s3_empty_tag.crn", &fixture("s3.Bucket", "team", "")),
    ]);
    for resource in [
        "cloudwatch.Alarm",
        "scheduler.ScheduleGroup",
        "iam.OidcProvider",
    ] {
        assert_invalid(resource, "team", "");
    }
    assert_invalid("kms.Key", "team", &"v".repeat(257));
    assert_invalid("s3.Bucket", "team", &"v".repeat(257));
}

#[test]
fn sso_tag_patterns_are_whole_value_and_keep_ecmascript_ascii_word_classes() {
    for resource in ["sso.PermissionSet", "sso.Instance"] {
        validate_fixtures(&[(
            "sso_tags.crn",
            &fixture(resource, "Team_1+=,.@-", "value_1"),
        )]);
        for key in [
            "team space",
            " team",
            "team ",
            "café",
            "チーム",
            "team١",
            "",
        ] {
            assert_invalid(resource, key, "valid");
        }
        for value in ["has space", "café", "", "valid!"] {
            assert_invalid(resource, "team", value);
        }
    }
}

#[test]
fn unicode_tag_patterns_and_unrestricted_patterns_keep_their_source_meaning() {
    for resource in ["cloudfront.Function", "ssm.Parameter"] {
        validate_fixtures(&[("unicode_tags.crn", &fixture(resource, "チーム é١", "valid"))]);
        assert_invalid(resource, "team!", "valid");
    }
    validate_fixtures(&[(
        "organizations_tags.crn",
        &fixture("organizations.Account", "team!", "anything!"),
    )]);
}

#[test]
fn unconstrained_tags_keep_plain_strings_and_no_entry_count_limit_is_added() {
    let config = super::generated::get_config_by_type("ecs.Cluster").unwrap();
    let tags = field(&config.schema, "tags");
    let RawShape::Map {
        key,
        value,
        constraints,
    } = tags.as_type().raw_shape()
    else {
        panic!("Tags must remain a map");
    };
    assert!(constraints.length().is_none());
    for entry in [key, value] {
        assert!(matches!(
            entry.raw_shape(),
            RawShape::String {
                identity: None,
                pattern: None,
                length: None,
                validate: None,
                ..
            }
        ));
    }
    validate_fixtures(&[(
        "unconstrained_tags.crn",
        &fixture("ecs.Cluster", "", &"v".repeat(300)),
    )]);
    let entries = (0..51)
        .map(|i| format!("'key{i}' = 'value'"))
        .collect::<Vec<_>>()
        .join("\n");
    validate_fixtures(&[(
        "many_tags.crn",
        &format!(
            "provider awscc {{ region = awscc.Region.ap_northeast_1 }}\nawscc.ecr.Repository {{ tags = {{ {entries} }} }}"
        ),
    )]);
}

#[tokio::test]
async fn default_tags_merge_into_typed_maps_but_core_does_not_revalidate_new_entries() {
    for (resource_type, key, value, valid) in [
        ("ecr.Repository", "team".to_string(), "v".repeat(256), false),
        (
            "ecr.Repository",
            "k".repeat(128),
            "valid".to_string(),
            false,
        ),
        ("cloudwatch.Alarm", "team".to_string(), String::new(), false),
        ("kms.Key", "team".to_string(), String::new(), true),
        ("ec2.Eip", "team".to_string(), "v".repeat(300), false),
    ] {
        let config = ProviderConfig {
            name: "awscc".into(),
            attributes: IndexMap::new(),
            default_tags: IndexMap::from([(key.clone(), string(&value))]),
            source: None,
            version: None,
            revision: None,
            unresolved_attributes: IndexMap::new(),
            binding: None,
        };
        with_preparation_and_configs(resource_type, &[config], async |preparation| {
            let resource = Resource::with_provider("awscc", resource_type, "default_tags", None);
            // Known core gap: https://github.com/carina-rs/carina/issues/4279.
            // Validation runs before normalization merges default_tags. The first
            // assertion below is expected to flip for invalid entries once #4279
            // lands: this is a tripwire for the gap, not desired behavior.
            let ready = prepare_provider_ready_resource(resource.clone(), preparation)
                .await
                .unwrap_or_else(|error| panic!("{resource_type}: {error}"));
            let expected = Value::Concrete(ConcreteValue::Map(IndexMap::from([(
                key.clone(),
                string(&value),
            )])));
            assert_eq!(ready.as_resource().attributes.get("tags"), Some(&expected));

            // Entries already present at that gate use the same Tag type,
            // regardless of whether they were authored or merged earlier.
            let result = prepare_provider_ready_resource(
                resource.with_attribute("tags", expected),
                preparation,
            )
            .await;
            if valid {
                result.unwrap_or_else(|error| panic!("{resource_type}: {error}"));
            } else {
                let error = result
                    .expect_err("entries present before validation must satisfy the Tag schema")
                    .to_string();
                assert!(error.contains("tags"), "{resource_type}: {error}");
                assert!(error.contains(&key), "{resource_type}: {error}");
            }
        })
        .await;
    }
}
