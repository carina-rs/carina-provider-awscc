//! S3-specific operations.
//!
//! Handles S3 bucket operations that go beyond the Cloud Control API,
//! such as emptying a bucket before deletion (force_delete).

use aws_sdk_s3::error::SdkError;
use aws_smithy_types::error::metadata::ProvideErrorMetadata;

use carina_core::provider::{ProviderError, ProviderResult};

use super::{AwsccProvider, error_helpers::api_error_with_meta_and_subject};

#[derive(Debug)]
struct S3Bucket(String);

impl std::fmt::Display for S3Bucket {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "bucket {}", self.0)
    }
}

impl AwsccProvider {
    /// Create an S3 client from the stored config
    pub(crate) fn s3_client(&self) -> aws_sdk_s3::Client {
        aws_sdk_s3::Client::new(&self.aws_config)
    }

    /// Empty an S3 bucket by deleting all objects and versions
    pub(crate) async fn empty_s3_bucket(&self, bucket_name: &str) -> ProviderResult<()> {
        let s3 = self.s3_client();

        // Delete all object versions (handles versioned and non-versioned buckets)
        let mut key_marker: Option<String> = None;
        let mut version_id_marker: Option<String> = None;

        loop {
            let mut req = s3.list_object_versions().bucket(bucket_name).max_keys(1000);
            if let Some(ref km) = key_marker {
                req = req.key_marker(km);
            }
            if let Some(ref vim) = version_id_marker {
                req = req.version_id_marker(vim);
            }

            let response = match req.send().await {
                Ok(resp) => resp,
                Err(e) => {
                    // If the bucket no longer exists, there's nothing to empty
                    if is_no_such_bucket(&e) {
                        return Ok(());
                    }
                    return Err(api_error_with_meta_and_subject(
                        "Failed to list object versions",
                        "s3.ListObjectVersions",
                        S3Bucket(bucket_name.to_string()),
                        e,
                    ));
                }
            };

            let mut objects_to_delete = Vec::new();

            // Collect versions
            for version in response.versions() {
                if let Some(key) = version.key() {
                    let mut id = aws_sdk_s3::types::ObjectIdentifier::builder().key(key);
                    if let Some(vid) = version.version_id() {
                        id = id.version_id(vid);
                    }
                    objects_to_delete.push(id.build().map_err(|e| {
                        ProviderError::internal_caused_by("Failed to build ObjectIdentifier", e)
                    })?);
                }
            }

            // Collect delete markers
            for marker in response.delete_markers() {
                if let Some(key) = marker.key() {
                    let mut id = aws_sdk_s3::types::ObjectIdentifier::builder().key(key);
                    if let Some(vid) = marker.version_id() {
                        id = id.version_id(vid);
                    }
                    objects_to_delete.push(id.build().map_err(|e| {
                        ProviderError::internal_caused_by("Failed to build ObjectIdentifier", e)
                    })?);
                }
            }

            // Batch delete (max 1000 per request)
            if !objects_to_delete.is_empty() {
                let delete = aws_sdk_s3::types::Delete::builder()
                    .set_objects(Some(objects_to_delete))
                    .quiet(true)
                    .build()
                    .map_err(|e| {
                        ProviderError::internal_caused_by("Failed to build Delete request", e)
                    })?;

                s3.delete_objects()
                    .bucket(bucket_name)
                    .delete(delete)
                    .send()
                    .await
                    .map_err(|e| {
                        api_error_with_meta_and_subject(
                            "Failed to delete objects",
                            "s3.DeleteObjects",
                            S3Bucket(bucket_name.to_string()),
                            e,
                        )
                    })?;
            }

            if response.is_truncated() == Some(true) {
                key_marker = response.next_key_marker().map(|s| s.to_string());
                version_id_marker = response.next_version_id_marker().map(|s| s.to_string());
            } else {
                break;
            }
        }

        Ok(())
    }
}

/// Check if an S3 SDK error is a NoSuchBucket error.
///
/// When force_delete runs during destroy, the bucket may have already been
/// deleted (e.g., by a concurrent operation or CloudFormation cleanup).
/// In that case, there's nothing to empty and we can proceed.
fn is_no_such_bucket<E: ProvideErrorMetadata>(error: &SdkError<E>) -> bool {
    match error {
        SdkError::ServiceError(service_error) => service_error.err().code() == Some("NoSuchBucket"),
        _ => false,
    }
}
