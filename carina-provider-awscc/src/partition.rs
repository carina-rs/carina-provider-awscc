//! AWSCC's partition definition for ARN validation and synthesis.

/// AWS partitions from the SDK partitions metadata embedded in
/// `aws-sdk-cloudcontrol`'s `src/endpoint_lib.rs`.
///
/// The metadata cross-check in this module's tests follows the locked SDK
/// version, so SDK partition and region additions require an explicit update.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Partition {
    Aws,
    AwsCn,
    AwsUsGov,
    AwsIso,
    AwsIsoB,
    AwsIsoE,
    AwsIsoF,
    AwsEusc,
}

impl Partition {
    pub(crate) const ALL: [Self; 8] = [
        Self::Aws,
        Self::AwsCn,
        Self::AwsUsGov,
        Self::AwsIso,
        Self::AwsIsoB,
        Self::AwsIsoE,
        Self::AwsIsoF,
        Self::AwsEusc,
    ];

    pub(crate) const fn id(self) -> &'static str {
        match self {
            Self::Aws => "aws",
            Self::AwsCn => "aws-cn",
            Self::AwsUsGov => "aws-us-gov",
            Self::AwsIso => "aws-iso",
            Self::AwsIsoB => "aws-iso-b",
            Self::AwsIsoE => "aws-iso-e",
            Self::AwsIsoF => "aws-iso-f",
            Self::AwsEusc => "aws-eusc",
        }
    }

    pub(crate) fn from_id(id: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|partition| partition.id() == id)
    }

    pub(crate) fn from_region(region: &str) -> Self {
        if region.starts_with("cn-") {
            Self::AwsCn
        } else if region.starts_with("us-gov-") {
            Self::AwsUsGov
        } else if region.starts_with("us-iso-") {
            Self::AwsIso
        } else if region.starts_with("us-isob-") {
            Self::AwsIsoB
        } else if region.starts_with("eu-isoe-") {
            Self::AwsIsoE
        } else if region.starts_with("us-isof-") {
            Self::AwsIsoF
        } else if region.starts_with("eusc-") {
            Self::AwsEusc
        } else {
            Self::Aws
        }
    }
}

/// Non-capturing partition group shared by every AWSCC catalog ARN pattern.
pub(crate) fn regex_fragment() -> String {
    format!("(?:{})", Partition::ALL.map(Partition::id).join("|"))
}

/// Prefix a row's service/resource grammar, including its end anchor, with ARN
/// syntax and the shared partition group. Preserve the row's grammar verbatim.
pub(crate) fn arn_regex(rest: &str) -> String {
    format!("^arn:{}:{rest}", regex_fragment())
}

#[cfg(test)]
#[path = "partition_tests.rs"]
mod tests;
