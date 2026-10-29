//! Resend safety for AWS Cloud Control operations.

use carina_aws_types::Idempotency;

/// Returns the resend safety for a CloudControl SDK operation.
///
/// This table is derived from the CloudControl 2021-09-30 Smithy model. The
/// model marks `CancelResourceRequest` as `smithy.api#idempotent` and marks the
/// `ClientToken` members of create, delete, and update inputs with
/// `smithy.api#idempotencyToken`. The model omits `smithy.api#readonly` on its
/// read operations, so the get/list operations are classified by their
/// documented read-only semantics to preserve the SDK's existing read retries.
pub(crate) fn cloudcontrol_idempotency(operation: &str) -> Idempotency {
    match operation {
        "CancelResourceRequest" => Idempotency::Idempotent,
        "CreateResource" | "DeleteResource" | "UpdateResource" => Idempotency::IdempotencyToken,
        "GetResource" | "GetResourceRequestStatus" | "ListResourceRequests" | "ListResources" => {
            Idempotency::ReadOnly
        }
        _ => Idempotency::NotIdempotent,
    }
}

#[cfg(test)]
mod tests {
    use carina_aws_types::Idempotency;

    use super::cloudcontrol_idempotency;

    const SDK_VERSION: &str = "1.103.0";
    const OPERATIONS: &[(&str, Idempotency)] = &[
        ("CancelResourceRequest", Idempotency::Idempotent),
        ("CreateResource", Idempotency::IdempotencyToken),
        ("DeleteResource", Idempotency::IdempotencyToken),
        ("GetResource", Idempotency::ReadOnly),
        ("GetResourceRequestStatus", Idempotency::ReadOnly),
        ("ListResourceRequests", Idempotency::ReadOnly),
        ("ListResources", Idempotency::ReadOnly),
        ("UpdateResource", Idempotency::IdempotencyToken),
    ];

    fn locked_cloudcontrol_version() -> Option<&'static str> {
        let lockfile = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../Cargo.lock"));
        lockfile.split("[[package]]").find_map(|package| {
            let mut name = None;
            let mut version = None;
            for line in package.lines().map(str::trim) {
                if let Some(value) = line.strip_prefix("name = \"") {
                    name = value.strip_suffix('"');
                } else if let Some(value) = line.strip_prefix("version = \"") {
                    version = value.strip_suffix('"');
                }
            }
            (name == Some("aws-sdk-cloudcontrol"))
                .then_some(version)
                .flatten()
        })
    }

    #[test]
    fn cloudcontrol_idempotency_covers_every_locked_sdk_operation() {
        assert_eq!(
            locked_cloudcontrol_version(),
            Some(SDK_VERSION),
            "re-enumerate aws-sdk-cloudcontrol operations and Smithy traits before updating the SDK"
        );

        for &(operation, expected) in OPERATIONS {
            assert_eq!(
                cloudcontrol_idempotency(operation),
                expected,
                "wrong idempotency for {operation}"
            );
        }
        assert_eq!(
            cloudcontrol_idempotency("UnknownFutureOperation"),
            Idempotency::NotIdempotent
        );
    }
}
