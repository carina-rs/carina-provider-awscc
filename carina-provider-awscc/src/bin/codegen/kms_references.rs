//! Single classification table for KMS-looking input strings.
//!
//! Keys are DSL resource names and complete paths from the resource root.
//! `$ref` names never replace the use site's path. The final-registry coverage
//! test imports this same module, including its path and candidate rules.

use super::ApprovedPattern;
use anyhow::{Result, bail};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Classification {
    Reference(ReferenceKind),
    NotReference(NotReference),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Fallback grammar for reference leaves without a CFN pattern.
pub enum ReferenceKind {
    Key,
}

impl ReferenceKind {
    pub fn default_length(self) -> Option<(Option<u64>, Option<u64>)> {
        let default_type = carina_aws_types::kms_key_reference();
        let carina_core::schema::RawShape::Reference { length, .. } = default_type.raw_shape()
        else {
            unreachable!("KMS constructors must return reference leaves");
        };
        length
    }

    pub fn constructor(
        self,
        cfn_length: Option<(Option<u64>, Option<u64>)>,
        pattern: &ApprovedPattern,
    ) -> String {
        if cfn_length.is_none() || cfn_length == self.default_length() {
            return "carina_aws_types::kms_key_reference()".to_string();
        }
        let pattern = if pattern.as_str() == carina_aws_types::KMS_KEY_REFERENCE_PATTERN {
            // Retain the shared symbol in generated output after approving
            // its exact value, avoiding unrelated generated-resource diffs.
            "Some(carina_aws_types::KMS_KEY_REFERENCE_PATTERN.to_string())".to_string()
        } else {
            format!("Some({:?}.to_string())", pattern.as_str())
        };
        format!(
            "carina_core::schema::AttributeType::reference(carina_aws_types::AwsType::KmsKeyReference.identity(), {pattern}, {cfn_length:?})"
        )
    }

    #[cfg(test)]
    #[allow(dead_code)] // Used by the library's final-registry test via this shared module.
    pub fn default_pattern(self) -> &'static str {
        carina_aws_types::KMS_KEY_REFERENCE_PATTERN
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NotReference {
    Other(&'static str),
}

pub enum PathSegment<'a> {
    Field(&'a str),
    ListElement,
    MapValue,
}

pub fn property_path(parent: &str, segment: PathSegment<'_>) -> String {
    match segment {
        PathSegment::Field(name) if parent.is_empty() => name.to_string(),
        PathSegment::Field(name) => format!("{parent}.{name}"),
        PathSegment::ListElement => format!("{parent}[]"),
        PathSegment::MapValue => format!("{parent}{{}}"),
    }
}

pub struct Entry {
    pub resource: &'static str,
    pub path: &'static str,
    pub classification: Classification,
    /// Pin the source constraint: a changed CFN pattern needs a fresh decision.
    pub cfn_pattern: Option<&'static str>,
    /// Pin the declared bounds, including absent and one-sided constraints.
    pub cfn_length: Option<(Option<u64>, Option<u64>)>,
}

pub const CLASSIFICATIONS: &[Entry] = &[
    Entry {
        resource: "ecr.Repository",
        path: "encryption_configuration.kms_key",
        classification: Classification::Reference(ReferenceKind::Key),
        // No pattern; description accepts alias, key ID, or full ARN. Length 1..2048.
        cfn_pattern: None,
        cfn_length: Some((Some(1), Some(2048))),
    },
    Entry {
        resource: "dynamodb.Table",
        path: "sse_specification.kms_master_key_id",
        classification: Classification::Reference(ReferenceKind::Key),
        // No pattern; description accepts key ID, ARN, alias name, or alias ARN.
        cfn_pattern: None,
        cfn_length: None,
    },
    Entry {
        resource: "ecs.Cluster",
        path: "configuration.execute_command_configuration.kms_key_id",
        classification: Classification::Reference(ReferenceKind::Key),
        // No pattern; description specifies a KMS key ID for execute-command traffic.
        cfn_pattern: None,
        cfn_length: None,
    },
    Entry {
        resource: "ecs.Cluster",
        path: "configuration.managed_storage_configuration.kms_key_id",
        classification: Classification::Reference(ReferenceKind::Key),
        // No pattern; description specifies a single-Region KMS key ID for managed storage.
        cfn_pattern: None,
        cfn_length: None,
    },
    Entry {
        resource: "ecs.Cluster",
        path: "configuration.managed_storage_configuration.fargate_ephemeral_storage_kms_key_id",
        classification: Classification::Reference(ReferenceKind::Key),
        // No pattern; description specifies a single-Region KMS key ID for ephemeral storage.
        cfn_pattern: None,
        cfn_length: None,
    },
    Entry {
        resource: "logs.LogGroup",
        path: "kms_key_id",
        classification: Classification::Reference(ReferenceKind::Key),
        // CFN accepts (key|alias) ARNs, unlike the AWS provider's key-only Smithy input.
        // Preserve this constraint through the ordinary CFN pattern conversion.
        cfn_pattern: Some(r"^arn:[a-z0-9-]+:kms:[a-z0-9-]+:\d{12}:(key|alias)/.+\Z"),
        cfn_length: Some((None, Some(256))),
    },
    Entry {
        resource: "s3.Bucket",
        path: "bucket_encryption.server_side_encryption_configuration[].server_side_encryption_by_default.kms_master_key_id",
        classification: Classification::Reference(ReferenceKind::Key),
        // No pattern; description accepts key ID, key alias, or key ARN.
        cfn_pattern: None,
        cfn_length: None,
    },
    Entry {
        resource: "s3.Bucket",
        path: "replication_configuration.rules[].destination.encryption_configuration.replica_kms_key_id",
        classification: Classification::Reference(ReferenceKind::Key),
        // No restrictive pattern; description calls this an ID (Key ARN or Alias ARN).
        cfn_pattern: None,
        cfn_length: None,
    },
    Entry {
        resource: "s3.Bucket",
        path: "metadata_configuration.inventory_table_configuration.encryption_configuration.kms_key_arn",
        classification: Classification::Reference(ReferenceKind::Key),
        // No restrictive pattern; description requires a customer-managed key ARN in the bucket Region.
        cfn_pattern: None,
        cfn_length: None,
    },
    Entry {
        resource: "s3.Bucket",
        path: "metadata_configuration.journal_table_configuration.encryption_configuration.kms_key_arn",
        classification: Classification::Reference(ReferenceKind::Key),
        // Same shared definition as inventory encryption: no restrictive pattern; KMS key ARN description.
        cfn_pattern: None,
        cfn_length: None,
    },
    Entry {
        resource: "sns.Topic",
        path: "kms_master_key_id",
        classification: Classification::Reference(ReferenceKind::Key),
        // No pattern; description specifies the AWS managed or custom CMK's ID.
        cfn_pattern: None,
        cfn_length: None,
    },
    Entry {
        resource: "scheduler.Schedule",
        path: "kms_key_arn",
        classification: Classification::Reference(ReferenceKind::Key),
        // CFN explicitly accepts (key|alias)/ ARNs only, length 1..2048.
        cfn_pattern: Some(r"^arn:aws[a-z-]*:kms:[a-z0-9\-]+:\d{12}:(key|alias)\/[0-9a-zA-Z-_]*$"),
        cfn_length: Some((Some(1), Some(2048))),
    },
    Entry {
        resource: "dynamodb.Table",
        path: "sse_specification.sse_type",
        classification: Classification::NotReference(NotReference::Other(
            "Encryption mode selector; it does not identify a KMS key.",
        )),
        cfn_pattern: None,
        cfn_length: None,
    },
    Entry {
        resource: "ecr.Repository",
        path: "encryption_configuration.encryption_type",
        classification: Classification::NotReference(NotReference::Other(
            "Encryption algorithm selector; it does not identify a KMS key.",
        )),
        cfn_pattern: None,
        cfn_length: None,
    },
    Entry {
        resource: "kms.Key",
        path: "description",
        classification: Classification::NotReference(NotReference::Other(
            "Human-readable description of the key being managed.",
        )),
        cfn_pattern: None,
        cfn_length: None,
    },
    Entry {
        resource: "kms.Key",
        path: "key_policy",
        classification: Classification::NotReference(NotReference::Other(
            "IAM policy document for the key being managed, not a key identifier.",
        )),
        cfn_pattern: None,
        cfn_length: None,
    },
    Entry {
        resource: "kms.Key",
        path: "key_usage",
        classification: Classification::NotReference(NotReference::Other(
            "Cryptographic operation selector for the key being managed.",
        )),
        cfn_pattern: None,
        cfn_length: None,
    },
    Entry {
        resource: "kms.Key",
        path: "origin",
        classification: Classification::NotReference(NotReference::Other(
            "Key-material source selector for the key being managed.",
        )),
        cfn_pattern: None,
        cfn_length: None,
    },
    Entry {
        resource: "kms.Key",
        path: "key_spec",
        classification: Classification::NotReference(NotReference::Other(
            "Cryptographic key specification, not an existing key identifier.",
        )),
        cfn_pattern: None,
        cfn_length: None,
    },
    Entry {
        resource: "s3.Bucket",
        path: "bucket_encryption.server_side_encryption_configuration[].server_side_encryption_by_default.sse_algorithm",
        classification: Classification::NotReference(NotReference::Other(
            "Server-side encryption algorithm selector, not a key identifier.",
        )),
        cfn_pattern: None,
        cfn_length: None,
    },
    Entry {
        resource: "s3.Bucket",
        path: "metadata_configuration.inventory_table_configuration.encryption_configuration.sse_algorithm",
        classification: Classification::NotReference(NotReference::Other(
            "Metadata-table encryption algorithm selector, not a key identifier.",
        )),
        cfn_pattern: None,
        cfn_length: None,
    },
    Entry {
        resource: "s3.Bucket",
        path: "metadata_configuration.journal_table_configuration.encryption_configuration.sse_algorithm",
        classification: Classification::NotReference(NotReference::Other(
            "Metadata-table encryption algorithm selector, not a key identifier.",
        )),
        cfn_pattern: None,
        cfn_length: None,
    },
    Entry {
        resource: "s3.Bucket",
        path: "replication_configuration.rules[].source_selection_criteria.sse_kms_encrypted_objects.status",
        classification: Classification::NotReference(NotReference::Other(
            "Replication enable/disable selector for KMS-encrypted objects.",
        )),
        cfn_pattern: None,
        cfn_length: None,
    },
    Entry {
        resource: "dynamodb.Table",
        path: "sse_specification.sse_enabled",
        classification: Classification::NotReference(NotReference::Other(
            "Boolean enabling server-side encryption, not a key identifier.",
        )),
        cfn_pattern: None,
        cfn_length: None,
    },
    Entry {
        resource: "s3.Bucket",
        path: "bucket_encryption.server_side_encryption_configuration[].bucket_key_enabled",
        classification: Classification::NotReference(NotReference::Other(
            "Boolean selecting S3 Bucket Keys, not a key identifier.",
        )),
        cfn_pattern: None,
        cfn_length: None,
    },
    Entry {
        resource: "kms.Key",
        path: "enabled",
        classification: Classification::NotReference(NotReference::Other(
            "Boolean enabling the managed key.",
        )),
        cfn_pattern: None,
        cfn_length: None,
    },
    Entry {
        resource: "kms.Key",
        path: "enable_key_rotation",
        classification: Classification::NotReference(NotReference::Other(
            "Boolean enabling rotation of the managed key.",
        )),
        cfn_pattern: None,
        cfn_length: None,
    },
    Entry {
        resource: "kms.Key",
        path: "multi_region",
        classification: Classification::NotReference(NotReference::Other(
            "Boolean selecting a multi-Region managed key.",
        )),
        cfn_pattern: None,
        cfn_length: None,
    },
    Entry {
        resource: "kms.Key",
        path: "pending_window_in_days",
        classification: Classification::NotReference(NotReference::Other(
            "Numeric deletion waiting period for the managed key.",
        )),
        cfn_pattern: None,
        cfn_length: None,
    },
    Entry {
        resource: "kms.Key",
        path: "bypass_policy_lockout_safety_check",
        classification: Classification::NotReference(NotReference::Other(
            "Boolean controlling the policy lockout safety check.",
        )),
        cfn_pattern: None,
        cfn_length: None,
    },
    Entry {
        resource: "kms.Key",
        path: "rotation_period_in_days",
        classification: Classification::NotReference(NotReference::Other(
            "Numeric rotation period for the managed key.",
        )),
        cfn_pattern: None,
        cfn_length: None,
    },
    Entry {
        resource: "kms.Key",
        path: "tags{}",
        classification: Classification::NotReference(NotReference::Other(
            "User-authored tag values, not KMS key identifiers.",
        )),
        cfn_pattern: None,
        cfn_length: None,
    },
];

pub fn entry(resource: &str, path: &str) -> Option<&'static Entry> {
    CLASSIFICATIONS
        .iter()
        .find(|entry| entry.resource == resource && entry.path == path)
}

fn mentions_kms(text: &str) -> bool {
    let text = text
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_ascii_lowercase();
    text.contains("kms")
        || text.contains("cmk")
        || text.contains("customer master key")
        || text.contains("key management service")
}

pub fn is_candidate<'a>(
    name: &str,
    pattern: Option<&str>,
    description: Option<&str>,
    literals: impl IntoIterator<Item = &'a str>,
) -> bool {
    mentions_kms(name)
        || mentions_kms(&format!(
            "{} {}",
            pattern.unwrap_or_default(),
            description.unwrap_or_default()
        ))
        || literals.into_iter().any(mentions_kms)
}

pub fn classify_string<'a>(
    resource: &str,
    path: &str,
    name: &str,
    pattern: Option<&str>,
    description: Option<&str>,
    literals: impl IntoIterator<Item = &'a str>,
) -> Result<Option<Classification>> {
    let decision = entry(resource, path).map(|entry| entry.classification);
    if is_candidate(name, pattern, description, literals) && decision.is_none() {
        bail!("unclassified KMS string {resource}.{path}");
    }
    Ok(decision)
}
