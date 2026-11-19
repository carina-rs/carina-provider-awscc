//! The only construction seam for identities owned by AWSCC (#572).

use carina_core::schema::TypeIdentity;

pub const AWSCC_PROVIDER_NAME: &str = "awscc";

/// Service/resource segments, without a provider. Rendering owns the prefix.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AwsccNamespace(String);

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error(
    "invalid AWSCC namespace {0:?}: expected nonempty service/resource identifiers without a provider"
)]
pub struct InvalidAwsccNamespace(String);

impl AwsccNamespace {
    pub fn new(segments: &str) -> Result<Self, InvalidAwsccNamespace> {
        if segments.is_empty()
            || !segments.split('.').all(|segment| {
                let mut characters = segment.chars();
                characters
                    .next()
                    .is_some_and(|c| c.is_ascii_alphabetic() || c == '_')
                    && characters.all(|c| c.is_ascii_alphanumeric() || c == '_')
                    && !segment.eq_ignore_ascii_case("aws")
                    && !segment.eq_ignore_ascii_case(AWSCC_PROVIDER_NAME)
            })
        {
            return Err(InvalidAwsccNamespace(segments.to_string()));
        }
        Ok(Self(segments.to_string()))
    }

    /// Explicit provider-level namespace for Arn, Region, AccountId and ResourceId.
    pub fn root() -> Self {
        Self(String::new())
    }

    pub fn child(&self, segments: &str) -> Result<Self, InvalidAwsccNamespace> {
        Self::new(&if self.0.is_empty() {
            segments.to_string()
        } else {
            format!("{}.{segments}", self.0)
        })
    }

    pub fn segments(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for AwsccNamespace {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.0.is_empty() {
            f.write_str(AWSCC_PROVIDER_NAME)
        } else {
            write!(f, "{AWSCC_PROVIDER_NAME}.{}", self.0)
        }
    }
}

#[expect(
    clippy::disallowed_methods,
    reason = "the AWSCC identity construction seam owns the provider (#572)"
)]
pub fn awscc_identity(namespace: &AwsccNamespace, kind: &str) -> TypeIdentity {
    TypeIdentity::new(
        Some(AWSCC_PROVIDER_NAME),
        namespace.0.split('.').filter(|segment| !segment.is_empty()),
        kind,
    )
}

pub fn awscc_enum_identity(kind: &str, namespace: &AwsccNamespace) -> TypeIdentity {
    awscc_identity(namespace, kind)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn namespace_rejects_empty_provider_and_malformed_segments() {
        for invalid in [
            "",
            "aws",
            "awscc",
            "aws.ec2",
            "awscc.ec2",
            "ec2.aws",
            "ec2.awscc",
            "AWS.ec2",
            ".Vpc",
            "ec2.",
            "ec2..Vpc",
            "ec2/Vpc",
            "ec2.Vpc-Tag",
            "ec2.1Vpc",
        ] {
            assert!(AwsccNamespace::new(invalid).is_err(), "{invalid:?}");
        }
    }

    #[test]
    fn namespace_root_and_valid_children_keep_fixed_awscc_ownership() {
        let root = AwsccNamespace::root();
        assert_eq!(root.to_string(), AWSCC_PROVIDER_NAME);
        assert_eq!(awscc_identity(&root, "Region").to_string(), "awscc.Region");
        let namespace = root.child("ec2.Vpc").unwrap().child("Tag").unwrap();
        assert_eq!(namespace, AwsccNamespace::new("ec2.Vpc.Tag").unwrap());
        let identity = awscc_identity(&namespace, "Kind");
        assert_eq!(identity.provider.as_deref(), Some(AWSCC_PROVIDER_NAME));
        assert_eq!(identity.segments, ["ec2", "Vpc", "Tag"]);
        assert_eq!(identity.kind, "Kind");
        assert_eq!(awscc_enum_identity("Kind", &namespace), identity);
    }

    #[test]
    fn namespace_children_cannot_inject_a_provider_or_empty_segment() {
        let namespace = AwsccNamespace::new("ec2.Vpc").unwrap();
        for invalid in ["", "aws", "awscc.Tag", ".Tag", "Tag.", "Tag..Kind"] {
            assert!(namespace.child(invalid).is_err(), "{invalid:?}");
        }
    }
}
