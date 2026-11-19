//! The only construction seam for identities owned by AWSCC (#572).

use carina_core::schema::TypeIdentity;

/// Service/resource segments, without a provider. Rendering owns the prefix.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AwsccNamespace(String);

impl AwsccNamespace {
    pub fn new(segments: &str) -> Self {
        assert!(
            segments.is_empty()
                || segments
                    .split('.')
                    .all(|segment| !segment.is_empty() && segment != "aws" && segment != "awscc"),
            "AWSCC namespace takes service/resource segments, never a provider"
        );
        Self(segments.to_string())
    }

    pub fn child(&self, segment: &str) -> Self {
        Self::new(&if self.0.is_empty() {
            segment.to_string()
        } else {
            format!("{}.{segment}", self.0)
        })
    }

    pub fn segments(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for AwsccNamespace {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.0.is_empty() {
            f.write_str("awscc")
        } else {
            write!(f, "awscc.{}", self.0)
        }
    }
}

#[expect(
    clippy::disallowed_methods,
    reason = "the AWSCC identity construction seam owns the provider (#572)"
)]
pub fn awscc_identity(segments: &[&str], kind: &str) -> TypeIdentity {
    TypeIdentity::new(Some("awscc"), segments.iter().copied(), kind)
}

pub fn awscc_enum_identity(kind: &str, namespace: &AwsccNamespace) -> TypeIdentity {
    awscc_identity(
        &namespace
            .0
            .split('.')
            .filter(|segment| !segment.is_empty())
            .collect::<Vec<_>>(),
        kind,
    )
}
