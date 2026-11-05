use std::collections::BTreeSet;
use std::path::PathBuf;

use super::*;

#[test]
fn all_is_exhaustive_and_ids_round_trip() {
    // Keep this match exhaustive: a new variant must be added to ALL as well.
    let expected_id = |partition| match partition {
        Partition::Aws => "aws",
        Partition::AwsCn => "aws-cn",
        Partition::AwsUsGov => "aws-us-gov",
        Partition::AwsIso => "aws-iso",
        Partition::AwsIsoB => "aws-iso-b",
        Partition::AwsIsoE => "aws-iso-e",
        Partition::AwsIsoF => "aws-iso-f",
        Partition::AwsEusc => "aws-eusc",
    };
    let ids = Partition::ALL.map(|partition| {
        assert_eq!(partition.id(), expected_id(partition));
        assert_eq!(Partition::from_id(partition.id()), Some(partition));
        partition.id()
    });
    assert_eq!(ids.into_iter().collect::<BTreeSet<_>>().len(), 8);
    for invalid in ["", "aws-xx", "AWS", "aws-iso-c", "aws\n"] {
        assert_eq!(Partition::from_id(invalid), None, "{invalid:?}");
    }
}

#[test]
fn regex_fragment_is_non_capturing_and_uses_all_partitions() {
    let pattern = regex::Regex::new(&format!("^{}$", regex_fragment())).unwrap();
    assert_eq!(pattern.captures_len(), 1);
    for partition in Partition::ALL {
        assert!(pattern.is_match(partition.id()), "{partition:?}");
    }
    for invalid in ["", "aws-xx", "aws-iso-c", "prefix-aws", "aws-eusc-suffix"] {
        assert!(!pattern.is_match(invalid), "{invalid:?}");
    }
    assert_eq!(
        arn_regex("s3:::.+$"),
        format!("^arn:{}:s3:::.+$", regex_fragment())
    );
}

#[test]
fn region_families_include_future_regions_and_default_to_aws() {
    for (region, partition) in [
        ("cn-future-99", Partition::AwsCn),
        ("us-gov-future-99", Partition::AwsUsGov),
        ("us-iso-future-99", Partition::AwsIso),
        ("us-isob-future-99", Partition::AwsIsoB),
        ("eu-isoe-future-99", Partition::AwsIsoE),
        ("us-isof-future-99", Partition::AwsIsoF),
        ("eusc-future-99", Partition::AwsEusc),
        ("", Partition::Aws),
        ("unknown-region-1", Partition::Aws),
        ("ap-northeast-1", Partition::Aws),
        ("us-iso", Partition::Aws),
        ("us-isob", Partition::Aws),
        ("eu-isoe", Partition::Aws),
        ("us-isof", Partition::Aws),
        ("eusc", Partition::Aws),
    ] {
        assert_eq!(Partition::from_region(region), partition, "{region:?}");
    }
}

#[test]
fn partitions_and_regions_match_locked_cloudcontrol_sdk_metadata() {
    // Cargo.toml unconditionally enables carina-aws-types' cloudcontrol feature,
    // so the SDK source is guaranteed to be fetched for this library test target.
    let lock_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../Cargo.lock");
    let lock = std::fs::read_to_string(&lock_path)
        .unwrap_or_else(|error| panic!("Cannot read {}: {error}", lock_path.display()));
    let versions = lock
        .split("[[package]]")
        .filter(|package| {
            package
                .lines()
                .any(|line| line.trim() == "name = \"aws-sdk-cloudcontrol\"")
        })
        .map(|package| {
            package
                .lines()
                .find_map(|line| line.trim().strip_prefix("version = \"")?.strip_suffix('"'))
                .expect("aws-sdk-cloudcontrol lock entry must have a version")
        })
        .collect::<Vec<_>>();
    assert_eq!(
        versions.len(),
        1,
        "Expected exactly one aws-sdk-cloudcontrol version in {}: {versions:?}",
        lock_path.display()
    );
    let cargo_home = std::env::var_os("CARGO_HOME")
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            PathBuf::from(std::env::var_os("HOME").expect("CARGO_HOME or HOME must be set"))
                .join(".cargo")
        });
    let registry_src = cargo_home.join("registry/src");
    let sdk_path = format!("aws-sdk-cloudcontrol-{}/src/endpoint_lib.rs", versions[0]);
    let searched = format!("{}/*/{sdk_path}", registry_src.display());
    let matches = std::fs::read_dir(&registry_src)
        .unwrap_or_else(|error| panic!("Cannot search {searched}: {error}"))
        .map(|entry| {
            entry
                .unwrap_or_else(|error| panic!("Cannot search {searched}: {error}"))
                .path()
                .join(&sdk_path)
        })
        .filter(|path| path.is_file())
        .collect::<Vec<_>>();
    assert_eq!(
        matches.len(),
        1,
        "Expected exactly one SDK source; searched {searched}; found {matches:?}"
    );
    let source = std::fs::read_to_string(&matches[0])
        .unwrap_or_else(|error| panic!("Cannot read {}: {error}", matches[0].display()));
    let byte_literal =
        regex::Regex::new(r#"PartitionResolver::new_from_json\(b("(?:\\.|[^"\\])*")\)"#).unwrap();
    let literals = byte_literal
        .captures_iter(&source)
        .map(|capture| capture.get(1).unwrap().as_str())
        .collect::<Vec<_>>();
    assert_eq!(
        literals.len(),
        1,
        "Expected one PartitionResolver::new_from_json byte-string literal in {}",
        matches[0].display()
    );
    // The generated Rust byte string uses JSON-compatible quote/backslash
    // escapes. Decode the literal first, then parse the embedded JSON document.
    let json: String = serde_json::from_str(literals[0]).unwrap_or_else(|error| {
        panic!("Cannot unescape the SDK partitions byte-string literal: {error}")
    });
    let metadata: serde_json::Value = serde_json::from_str(&json)
        .unwrap_or_else(|error| panic!("Cannot parse SDK partitions metadata: {error}"));
    let partitions = metadata["partitions"]
        .as_array()
        .expect("SDK metadata must contain a partitions array");
    let sdk_ids = partitions
        .iter()
        .map(|partition| {
            partition["id"]
                .as_str()
                .expect("SDK partition must have an id")
        })
        .collect::<BTreeSet<_>>();
    assert_eq!(
        sdk_ids.len(),
        partitions.len(),
        "Duplicate SDK partition ids"
    );
    assert_eq!(
        sdk_ids,
        Partition::ALL.map(Partition::id).into_iter().collect(),
        "AWSCC partitions differ from aws-sdk-cloudcontrol {}",
        versions[0]
    );
    for partition in partitions {
        let id = partition["id"].as_str().unwrap();
        let regions = partition["regions"]
            .as_object()
            .expect("SDK partition must have a regions object");
        assert!(!regions.is_empty(), "SDK partition {id} has no regions");
        for region in regions.keys().filter(|region| !region.ends_with("-global")) {
            assert_eq!(
                Partition::from_region(region).id(),
                id,
                "SDK region {region} belongs to {id}"
            );
        }
    }
}
