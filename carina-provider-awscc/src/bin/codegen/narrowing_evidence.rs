//! Documentation and recorded live observations are distinct evidence kinds.
//! Quotes are verbatim; observations retain exact inputs and source records.
use super::*;
use overrides::{AwsEvidence, CheckedOn, Narrowing, ObservedInput, VerifiedDirection};

static REVIEWS: LazyLock<Vec<serde_json::Value>> = LazyLock::new(|| {
    serde_json::from_str(include_str!("narrowing_evidence.json"))
        .expect("reviewed narrowing evidence JSON")
});

fn review(resource: &str, path: &str) -> Option<&'static serde_json::Value> {
    REVIEWS
        .iter()
        .find(|row| row["resource"] == resource && (row["path"] == path || row["field"] == path))
}

fn evidence(row: &'static serde_json::Value) -> AwsEvidence {
    let observation = &row["evidence"];
    let date = if observation["type"] == "observation" {
        &observation["date"]
    } else {
        &row["checked_on"]
    }
    .as_str()
    .expect("required evidence checked date");
    let parts = date
        .split('-')
        .map(str::parse::<u16>)
        .collect::<std::result::Result<Vec<_>, _>>()
        .expect("ISO evidence date");
    assert_eq!(parts.len(), 3, "ISO evidence date");
    let date = CheckedOn {
        year: parts[0],
        month: u8::try_from(parts[1]).expect("month"),
        day: u8::try_from(parts[2]).expect("day"),
    };
    if observation["type"] == "observation" {
        let cases = observation["cases"]
            .as_array()
            .expect("recorded exact inputs")
            .iter()
            .map(|case| ObservedInput {
                exact_input: &case["exact_input"],
                direction: match case["direction"].as_str().expect("verified direction") {
                    "accepted" => VerifiedDirection::Accepted,
                    "rejected" => VerifiedDirection::Rejected,
                    "read-back form" => VerifiedDirection::ReadBackForm,
                    _ => panic!("unknown observation direction"),
                },
                result: &case["result"],
            })
            .collect();
        return AwsEvidence::observed(
            cases,
            observation["region"].as_str().expect("observation region"),
            date,
            observation["source_record"]
                .as_str()
                .expect("observation source record"),
            observation["method"].as_str().expect("observation method"),
        )
        .expect("complete recorded observation");
    }
    let mut evidence = AwsEvidence::new(
        row["url"].as_str().expect("required evidence URL"),
        row["quote"].as_str().expect("required evidence quote"),
        date,
    )
    .expect("complete AWS evidence");
    if let Some(sources) = row["additional_sources"].as_array() {
        for source in sources {
            evidence
                .add_source(
                    source["url"].as_str().expect("additional URL"),
                    source["quote"].as_str().expect("additional quote"),
                )
                .expect("complete additional source");
        }
    }
    evidence
}

pub(super) fn for_field(resource: &str, path: &str) -> Option<AwsEvidence> {
    // Synthetic generator tests exercise inference without making an AWS claim.
    #[cfg(test)]
    if resource.starts_with("AWS::Test::") {
        return Some(AwsEvidence::fixture());
    }
    let row = review(resource, path)?;
    (row["status"] != "dropped").then(|| evidence(row))
}

pub(super) fn json_fallback(resource: &str, path: CfnPropertyPath<'_>) -> bool {
    review(resource, &path.to_string()).is_some_and(|r| r["json_fallback"] == true)
}

pub(super) fn reviewed(resource: &str, path: &str, mut kind: NarrowingKind) -> Option<Narrowing> {
    #[cfg(test)]
    if resource.starts_with("AWS::Test::") {
        return Some(Narrowing {
            kind,
            evidence: AwsEvidence::fixture(),
        });
    }
    let row = review(resource, path).unwrap_or_else(|| {
        panic!("{resource}.{path}: narrowing candidate has no documentation review")
    });
    if row["status"] == "dropped" {
        return None;
    }
    let adjustment = &row["adjustment"];
    if adjustment["availability_zone_or_all"] == true {
        let NarrowingKind::Primitive { expression, .. } = &mut kind else {
            panic!("Availability Zone amendment")
        };
        *expression = ConstraintSpec::availability_zone_or_all();
    }
    if let Some(pattern) = adjustment["string_pattern"].as_str() {
        let NarrowingKind::Primitive { expression, .. } = &mut kind else {
            panic!("string pattern amendment")
        };
        *expression = ConstraintSpec::string(
            Some(ApprovedPattern::whole(pattern)),
            expression.string_length(),
        );
    }
    if let Some(bounds) = adjustment["integer_range"].as_array() {
        kind = NarrowingKind::IntRange(bounds[0].as_i64().unwrap(), bounds[1].as_i64().unwrap());
    }
    if let Some(values) = adjustment["integer_values"].as_array() {
        kind = NarrowingKind::IntEnum(values.iter().map(|v| v.as_i64().unwrap()).collect());
    }
    if adjustment["string_length_only"] == true {
        let NarrowingKind::Primitive { expression, .. } = &mut kind else {
            panic!("string amendment")
        };
        *expression = expression.without_string_pattern();
    }
    if let Some(min) = adjustment["min_number"].as_i64() {
        let NarrowingKind::Primitive { expression, .. } = &mut kind else {
            panic!("numeric amendment")
        };
        *expression = expression.with_minimum(min);
    }
    if let Some(values) = adjustment["enum_values"].as_array() {
        kind = NarrowingKind::Enum(
            values
                .iter()
                .map(|v| v.as_str().expect("documented enum value"))
                .collect(),
        );
    }
    if adjustment["array_bounds_only"] == true {
        kind = match kind {
            NarrowingKind::Array {
                min_items,
                max_items,
                ..
            }
            | NarrowingKind::ArrayItemType {
                min_items,
                max_items,
                ..
            } => NarrowingKind::Array {
                items: None,
                min_items,
                max_items,
            },
            _ => panic!("array amendment"),
        };
    }
    if adjustment["remove_max_items"] == true {
        let NarrowingKind::Array { max_items, .. } = &mut kind else {
            panic!("array maximum amendment")
        };
        *max_items = None;
    }
    if let Some(maximum) = adjustment["max_items"].as_i64() {
        let NarrowingKind::Array { max_items, .. } = &mut kind else {
            panic!("array maximum amendment")
        };
        *max_items = Some(maximum);
    }
    Some(Narrowing {
        kind,
        evidence: evidence(row),
    })
}

pub(super) fn restored() -> impl Iterator<Item = (&'static str, &'static str, Narrowing)> {
    REVIEWS
        .iter()
        .filter(|row| row["status"] == "restored")
        .map(|row| {
            let c = &row["adjustment"]["restore_string"];
            let kind = NarrowingKind::Primitive {
                expression: ConstraintSpec::string(
                    c["pattern"].as_str().map(ApprovedPattern::whole),
                    c["max_length"].as_u64().map(|n| (None, Some(n))),
                ),
                block_name: None,
                default: None,
            };
            (
                row["resource"].as_str().expect("resource"),
                row["path"].as_str().expect("path"),
                Narrowing {
                    kind,
                    evidence: evidence(row),
                },
            )
        })
}

#[cfg(test)]
pub(super) fn rows() -> &'static [serde_json::Value] {
    &REVIEWS
}
