//! The only construction seam for identities owned by AWSCC (#572).

use carina_core::schema::TypeIdentity;

pub const AWSCC_PROVIDER_NAME: &str = "awscc";

/// An identity constructed by AWSCC. Its private field cannot be relabelled.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AwsccIdentity(TypeIdentity);

impl AwsccIdentity {
    /// Borrow the identity at a core schema/registry inspection boundary.
    pub fn as_core(&self) -> &TypeIdentity {
        &self.0
    }

    /// Consume the AWSCC identity at a core schema construction boundary.
    pub fn into_core(self) -> TypeIdentity {
        self.0
    }
}

impl std::fmt::Display for AwsccIdentity {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(&self.0, formatter)
    }
}

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
        if segments.is_empty() || !segments.split('.').all(valid_identifier) {
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

/// Namespace segments and identity kinds follow the same identifier policy.
fn valid_identifier(segment: &str) -> bool {
    let mut characters = segment.chars();
    characters
        .next()
        .is_some_and(|c| c.is_ascii_alphabetic() || c == '_')
        && characters.all(|c| c.is_ascii_alphanumeric() || c == '_')
        && !segment.eq_ignore_ascii_case("aws")
        && !segment.eq_ignore_ascii_case(AWSCC_PROVIDER_NAME)
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error(
    "invalid AWSCC identity kind {0:?}: expected a single nonempty identifier without a provider"
)]
pub struct InvalidAwsccKind(String);

/// The sole entrance for AWSCC-owned identities; both axes are validated.
#[expect(
    clippy::disallowed_methods,
    reason = "the AWSCC identity construction seam owns the provider (#572)"
)]
pub fn awscc_identity(
    kind: &str,
    namespace: &AwsccNamespace,
) -> Result<AwsccIdentity, InvalidAwsccKind> {
    if !valid_identifier(kind) {
        return Err(InvalidAwsccKind(kind.to_string()));
    }
    Ok(AwsccIdentity(TypeIdentity::new(
        Some(AWSCC_PROVIDER_NAME),
        namespace.0.split('.').filter(|segment| !segment.is_empty()),
        kind,
    )))
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
        assert_eq!(
            awscc_identity("Region", &root)
                .expect("valid AWSCC identity")
                .to_string(),
            "awscc.Region"
        );
        let namespace = root.child("ec2.Vpc").unwrap().child("Tag").unwrap();
        assert_eq!(namespace, AwsccNamespace::new("ec2.Vpc.Tag").unwrap());
        let identity = awscc_identity("Kind", &namespace).expect("valid AWSCC identity");
        assert_eq!(
            identity.as_core().provider.as_deref(),
            Some(AWSCC_PROVIDER_NAME)
        );
        assert_eq!(identity.as_core().segments, ["ec2", "Vpc", "Tag"]);
        assert_eq!(identity.as_core().kind, "Kind");
        assert_ne!(awscc_identity("OtherKind", &namespace).unwrap(), identity);
    }

    #[test]
    fn namespace_children_cannot_inject_a_provider_or_empty_segment() {
        let namespace = AwsccNamespace::new("ec2.Vpc").unwrap();
        for invalid in ["", "aws", "awscc.Tag", ".Tag", "Tag.", "Tag..Kind"] {
            assert!(namespace.child(invalid).is_err(), "{invalid:?}");
        }
    }
}

#[cfg(test)]
mod kind_tests {
    use super::*;
    #[test]
    fn identity_kinds_reject_provider_and_malformed_identifiers_without_panicking() {
        let namespace = AwsccNamespace::new("ec2.Vpc").unwrap();
        for kind in [
            "",
            "1Kind",
            "Kind.WithDots",
            "bad-name",
            "bad/name",
            "white space",
            "aws",
            "AWS",
            "awscc",
            "AwScC",
        ] {
            assert!(awscc_identity(kind, &namespace).is_err(), "{kind:?}");
        }
        for kind in ["Arn", "Id", "_Test", "IPv6Cidr"] {
            assert_eq!(
                awscc_identity(kind, &namespace).unwrap().as_core().kind,
                kind
            );
        }
    }
}
