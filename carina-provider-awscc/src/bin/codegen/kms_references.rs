//! Single classification table for KMS-looking input strings.
//!
//! Keys are DSL resource names and complete paths from the resource root.
//! `$ref` names never replace the use site's path. The final-registry coverage
//! test imports this same module, including its path and candidate rules.

use anyhow::{Result, bail};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Classification {
    Reference(ReferenceKind),
    NotReference(NotReference),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReferenceKind {
    Key,
    KeyOrAliasArn,
}

impl ReferenceKind {
    pub fn default_length(self) -> Option<(Option<u64>, Option<u64>)> {
        let default_type = match self {
            Self::Key => carina_aws_types::kms_key_reference(),
            Self::KeyOrAliasArn => carina_aws_types::kms_key_or_alias_arn_reference(),
        };
        let carina_core::schema::RawShape::Reference { length, .. } = default_type.raw_shape()
        else {
            unreachable!("KMS constructors must return reference leaves");
        };
        length
    }

    pub fn constructor(self, cfn_length: Option<(Option<u64>, Option<u64>)>) -> String {
        let constructor = match self {
            Self::Key => "carina_aws_types::kms_key_reference()",
            Self::KeyOrAliasArn => "carina_aws_types::kms_key_or_alias_arn_reference()",
        };
        if cfn_length.is_none() || cfn_length == self.default_length() {
            return constructor.to_string();
        }
        let pattern = match self {
            Self::Key => "carina_aws_types::KMS_KEY_REFERENCE_PATTERN",
            Self::KeyOrAliasArn => "carina_aws_types::KMS_KEY_OR_ALIAS_ARN_PATTERN",
        };
        format!(
            "carina_core::schema::AttributeType::reference(carina_aws_types::AwsType::KmsKeyReference.identity(), Some({pattern}.to_string()), {cfn_length:?})"
        )
    }

    #[cfg(test)]
    #[allow(dead_code)] // Used by the library's final-registry test via this shared module.
    pub fn expected_pattern(self) -> &'static str {
        match self {
            Self::Key => carina_aws_types::KMS_KEY_REFERENCE_PATTERN,
            Self::KeyOrAliasArn => carina_aws_types::KMS_KEY_OR_ALIAS_ARN_PATTERN,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)] // Decisions for future candidates must carry an explicit reason.
pub enum NotReference {
    Absent,
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
        classification: Classification::Reference(ReferenceKind::KeyOrAliasArn),
        // CFN accepts (key|alias) ARNs, unlike the AWS provider's key-only Smithy input.
        // Use the portable ARN reference grammar instead of CFN's legacy `\Z` spelling.
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
        classification: Classification::Reference(ReferenceKind::KeyOrAliasArn),
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
        resource: "kms.Key",
        path: "tags",
        classification: Classification::NotReference(NotReference::Other(
            "Tag collection projected to a string map; its KMS description concerns tag permissions.",
        )),
        cfn_pattern: None,
        cfn_length: None,
    },
    Entry {
        resource: "kms.Key",
        path: "tags[]",
        classification: Classification::NotReference(NotReference::Other(
            "Source tag pair in the projected collection, not a KMS key identifier.",
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

pub fn is_candidate(name: &str, pattern: Option<&str>, description: Option<&str>) -> bool {
    let name = name.to_ascii_lowercase();
    let evidence = format!(
        "{} {}",
        pattern.unwrap_or_default(),
        description.unwrap_or_default()
    )
    .to_ascii_lowercase();
    name.contains("kms")
        || name.contains("cmk")
        || evidence.contains("kms")
        || evidence.contains("customer master key")
        || evidence.contains("cmk")
}

pub fn classify_string(
    resource: &str,
    path: &str,
    name: &str,
    pattern: Option<&str>,
    description: Option<&str>,
) -> Result<Option<Classification>> {
    let decision = entry(resource, path).map(|entry| entry.classification);
    if is_candidate(name, pattern, description) && decision.is_none() {
        bail!("unclassified KMS string {resource}.{path}");
    }
    Ok(decision)
}
