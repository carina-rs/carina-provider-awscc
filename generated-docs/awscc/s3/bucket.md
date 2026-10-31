---
title: "awscc.s3.Bucket"
---


CloudFormation Type: `AWS::S3::Bucket`

The ``AWS::S3::Bucket`` resource creates an Amazon S3 bucket in the same AWS Region where you create the AWS CloudFormation stack.
 To control how AWS CloudFormation handles the bucket when the stack is deleted, you can set a deletion policy for your bucket. You can choose to *retain* the bucket or to *delete* the bucket. For more information, see [DeletionPolicy Attribute](https://docs.aws.amazon.com/AWSCloudFormation/latest/UserGuide/aws-attribute-deletionpolicy.html).
  You can only delete empty buckets. Deletion fails for buckets that have contents.

## Example

```crn
awscc.s3.Bucket {
  bucket_name = 'my-example-bucket'

  versioning_configuration = {
    status = enabled
  }

  tags = {
    Environment = 'example'
  }
}
```

## Argument Reference

### `abac_status`

- **Type:** [Enum (AbacStatus)](#abac_status-abacstatus)
- **Required:** No

The ABAC status of the general purpose bucket. When ABAC is enabled for the general purpose bucket, you can use tags to manage access to the general purpose buckets as well as for cost tracking purposes. When ABAC is disabled for the general purpose buckets, you can only use tags for cost tracking purposes. For more information, see [Using tags with S3 general purpose buckets](https://docs.aws.amazon.com/AmazonS3/latest/userguide/buckets-tagging.html).

### `accelerate_configuration`

- **Type:** [Struct(AccelerateConfiguration)](#accelerate_configuration-1)
- **Required:** No

Configures the transfer acceleration state for an Amazon S3 bucket. For more information, see [Amazon S3 Transfer Acceleration](https://docs.aws.amazon.com/AmazonS3/latest/dev/transfer-acceleration.html) in the *Amazon S3 User Guide*.

### `access_control`

- **Type:** [Enum (AccessControl)](#access_control-accesscontrol)
- **Required:** No
- **Write-only:** Yes

This is a legacy property, and it is not recommended for most use cases. A majority of modern use cases in Amazon S3 no longer require the use of ACLs, and we recommend that you keep ACLs disabled. For more information, see [Controlling object ownership](https://docs.aws.amazon.com//AmazonS3/latest/userguide/about-object-ownership.html) in the *Amazon S3 User Guide*. A canned access control list (ACL) that grants predefined permissions to the bucket. For more information about canned ACLs, see [Canned ACL](https://docs.aws.amazon.com/AmazonS3/latest/dev/acl-overview.html#canned-acl) in the *Amazon S3 User Guide*. S3 buckets are created with ACLs disabled by default. Therefore, unless you explicitly set the [AWS::S3::OwnershipControls](https://docs.aws.amazon.com//AWSCloudFormation/latest/UserGuide/aws-properties-s3-bucket-ownershipcontrols.html) property to enable ACLs, your resource will fail to deploy with any value other than Private. Use cases requiring ACLs are uncommon. The majority of access control configurations can be successfully and more easily achieved with bucket policies. For more information, see [AWS::S3::BucketPolicy](https://docs.aws.amazon.com//AWSCloudFormation/latest/UserGuide/aws-properties-s3-policy.html). For examples of common policy configurations, including S3 Server Access Logs buckets and more, see [Bucket policy examples](https://docs.aws.amazon.com/AmazonS3/latest/userguide/example-bucket-policies.html) in the *Amazon S3 User Guide*.

### `analytics_configurations`

- **Type:** List\<[Struct(AnalyticsConfiguration)](#analytics_configurations-1)\>
- **Required:** No

Specifies the configuration and any analyses for the analytics filter of an Amazon S3 bucket.

### `bucket_encryption`

- **Type:** [Struct(BucketEncryption)](#bucket_encryption-1)
- **Required:** No

Specifies default encryption for a bucket using server-side encryption with Amazon S3-managed keys (SSE-S3), AWS KMS-managed keys (SSE-KMS), or dual-layer server-side encryption with KMS-managed keys (DSSE-KMS). For information about the Amazon S3 default encryption feature, see [Amazon S3 Default Encryption for S3 Buckets](https://docs.aws.amazon.com/AmazonS3/latest/dev/bucket-encryption.html) in the *Amazon S3 User Guide*.

### `bucket_name`

- **Type:** String
- **Required:** No
- **Create-only:** Yes

A name for the bucket. If you don't specify a name, AWS CloudFormation generates a unique ID and uses that ID for the bucket name. The bucket name must contain only lowercase letters, numbers, periods (.), and dashes (-) and must follow [Amazon S3 bucket restrictions and limitations](https://docs.aws.amazon.com/AmazonS3/latest/dev/BucketRestrictions.html). For more information, see [Rules for naming Amazon S3 buckets](https://docs.aws.amazon.com/AmazonS3/latest/userguide/bucketnamingrules.html) in the *Amazon S3 User Guide*. If you specify a name, you can't perform updates that require replacement of this resource. You can perform updates that require no or some interruption. If you need to replace the resource, specify a new name.

### `cors_configuration`

- **Type:** [Struct(CorsConfiguration)](#cors_configuration-1)
- **Required:** No

Describes the cross-origin access configuration for objects in an Amazon S3 bucket. For more information, see [Enabling Cross-Origin Resource Sharing](https://docs.aws.amazon.com/AmazonS3/latest/dev/cors.html) in the *Amazon S3 User Guide*.

### `intelligent_tiering_configurations`

- **Type:** List\<[Struct(IntelligentTieringConfiguration)](#intelligent_tiering_configurations-1)\>
- **Required:** No

Defines how Amazon S3 handles Intelligent-Tiering storage.

### `inventory_configurations`

- **Type:** List\<[Struct(InventoryConfiguration)](#inventory_configurations-1)\>
- **Required:** No

Specifies the S3 Inventory configuration for an Amazon S3 bucket. For more information, see [GET Bucket inventory](https://docs.aws.amazon.com/AmazonS3/latest/API/RESTBucketGETInventoryConfig.html) in the *Amazon S3 API Reference*.

### `lifecycle_configuration`

- **Type:** [Struct(LifecycleConfiguration)](#lifecycle_configuration-1)
- **Required:** No

Specifies the lifecycle configuration for objects in an Amazon S3 bucket. For more information, see [Object Lifecycle Management](https://docs.aws.amazon.com/AmazonS3/latest/dev/object-lifecycle-mgmt.html) in the *Amazon S3 User Guide*.

### `logging_configuration`

- **Type:** [Struct(LoggingConfiguration)](#logging_configuration-1)
- **Required:** No

Settings that define where logs are stored.

### `metadata_configuration`

- **Type:** [Struct(MetadataConfiguration)](#metadata_configuration-1)
- **Required:** No

The S3 Metadata configuration for a general purpose bucket.

### `metadata_table_configuration`

- **Type:** [Struct(MetadataTableConfiguration)](#metadata_table_configuration-1)
- **Required:** No

The metadata table configuration of an S3 general purpose bucket.

### `metrics_configurations`

- **Type:** List\<[Struct(MetricsConfiguration)](#metrics_configurations-1)\>
- **Required:** No

Specifies a metrics configuration for the CloudWatch request metrics (specified by the metrics configuration ID) from an Amazon S3 bucket. If you're updating an existing metrics configuration, note that this is a full replacement of the existing metrics configuration. If you don't include the elements you want to keep, they are erased. For more information, see [PutBucketMetricsConfiguration](https://docs.aws.amazon.com/AmazonS3/latest/API/RESTBucketPUTMetricConfiguration.html).

### `notification_configuration`

- **Type:** [Struct(NotificationConfiguration)](#notification_configuration-1)
- **Required:** No

Configuration that defines how Amazon S3 handles bucket notifications.

### `object_lock_configuration`

- **Type:** [Struct(ObjectLockConfiguration)](#object_lock_configuration-1)
- **Required:** No

This operation is not supported for directory buckets. Places an Object Lock configuration on the specified bucket. The rule specified in the Object Lock configuration will be applied by default to every new object placed in the specified bucket. For more information, see [Locking Objects](https://docs.aws.amazon.com/AmazonS3/latest/dev/object-lock.html). + The ``DefaultRetention`` settings require both a mode and a period. + The ``DefaultRetention`` period can be either ``Days`` or ``Years`` but you must select one. You cannot specify ``Days`` and ``Years`` at the same time. + You can enable Object Lock for new or existing buckets. For more information, see [Configuring Object Lock](https://docs.aws.amazon.com/AmazonS3/latest/userguide/object-lock-configure.html). You must URL encode any signed header values that contain spaces. For example, if your header value is ``my file.txt``, containing two spaces after ``my``, you must URL encode this value to ``my%20%20file.txt``.

### `object_lock_enabled`

- **Type:** Bool
- **Required:** No

Indicates whether this bucket has an Object Lock configuration enabled. Enable ``ObjectLockEnabled`` when you apply ``ObjectLockConfiguration`` to a bucket.

### `ownership_controls`

- **Type:** [Struct(OwnershipControls)](#ownership_controls-1)
- **Required:** No

Configuration that defines how Amazon S3 handles Object Ownership rules.

### `public_access_block_configuration`

- **Type:** [Struct(PublicAccessBlockConfiguration)](#public_access_block_configuration-1)
- **Required:** No

Configuration that defines how Amazon S3 handles public access.

### `replication_configuration`

- **Type:** [Struct(ReplicationConfiguration)](#replication_configuration-1)
- **Required:** No

Configuration for replicating objects in an S3 bucket. To enable replication, you must also enable versioning by using the ``VersioningConfiguration`` property. Amazon S3 can store replicated objects in a single destination bucket or multiple destination buckets. The destination bucket or buckets must already exist.

### `tags`

- **Type:** `Map<String, String>`
- **Required:** No

An arbitrary set of tags (key-value pairs) for this S3 bucket.

### `versioning_configuration`

- **Type:** [Struct(VersioningConfiguration)](#versioning_configuration-1)
- **Required:** No

Enables multiple versions of all objects in this bucket. You might enable versioning to prevent objects from being deleted or overwritten by mistake or to archive objects so that you can retrieve previous versions of them. When you enable versioning on a bucket for the first time, it might take a short amount of time for the change to be fully propagated. We recommend that you wait for 15 minutes after enabling versioning before issuing write operations (``PUT`` or ``DELETE``) on objects in the bucket.

### `website_configuration`

- **Type:** [Struct(WebsiteConfiguration)](#website_configuration-1)
- **Required:** No

Information used to configure the bucket as a static website. For more information, see [Hosting Websites on Amazon S3](https://docs.aws.amazon.com/AmazonS3/latest/dev/WebsiteHosting.html).

## Enum Values

### abac_status (AbacStatus)

| Value | DSL Identifier |
|-------|----------------|
| `Enabled` | `aws.s3.Bucket.AbacStatus.enabled` |
| `Disabled` | `aws.s3.Bucket.AbacStatus.disabled` |

Shorthand formats: `enabled` or `AbacStatus.enabled`

### accelerate_configuration.acceleration_status (AccelerationStatus)

| Value | DSL Identifier |
|-------|----------------|
| `Enabled` | `aws.s3.Bucket.AccelerateConfiguration.AccelerationStatus.enabled` |
| `Suspended` | `aws.s3.Bucket.AccelerateConfiguration.AccelerationStatus.suspended` |

Shorthand formats: `enabled` or `AccelerationStatus.enabled`

### access_control (AccessControl)

| Value | DSL Identifier |
|-------|----------------|
| `AuthenticatedRead` | `aws.s3.Bucket.AccessControl.authenticated_read` |
| `AwsExecRead` | `aws.s3.Bucket.AccessControl.aws_exec_read` |
| `BucketOwnerFullControl` | `aws.s3.Bucket.AccessControl.bucket_owner_full_control` |
| `BucketOwnerRead` | `aws.s3.Bucket.AccessControl.bucket_owner_read` |
| `LogDeliveryWrite` | `aws.s3.Bucket.AccessControl.log_delivery_write` |
| `Private` | `aws.s3.Bucket.AccessControl.private` |
| `PublicRead` | `aws.s3.Bucket.AccessControl.public_read` |
| `PublicReadWrite` | `aws.s3.Bucket.AccessControl.public_read_write` |

Shorthand formats: `authenticated_read` or `AccessControl.authenticated_read`

### analytics_configurations.storage_class_analysis.data_export.output_schema_version (OutputSchemaVersion)

| Value | DSL Identifier |
|-------|----------------|
| `V_1` | `aws.s3.Bucket.AnalyticsConfiguration.StorageClassAnalysis.DataExport.OutputSchemaVersion.v_1` |

Shorthand formats: `v_1` or `OutputSchemaVersion.v_1`

### bucket_encryption.server_side_encryption_configuration.blocked_encryption_types.encryption_type (EncryptionType)

| Value | DSL Identifier |
|-------|----------------|
| `NONE` | `aws.s3.Bucket.BucketEncryption.ServerSideEncryptionRule.BlockedEncryptionTypes.EncryptionType.none` |
| `SSE-C` | `aws.s3.Bucket.BucketEncryption.ServerSideEncryptionRule.BlockedEncryptionTypes.EncryptionType.sse_c` |

Shorthand formats: `none` or `EncryptionType.none`

### bucket_encryption.server_side_encryption_configuration.server_side_encryption_by_default.sse_algorithm (SseAlgorithm)

| Value | DSL Identifier |
|-------|----------------|
| `aws:kms` | `aws.s3.Bucket.BucketEncryption.ServerSideEncryptionRule.ServerSideEncryptionByDefault.SseAlgorithm.aws_kms` |
| `AES256` | `aws.s3.Bucket.BucketEncryption.ServerSideEncryptionRule.ServerSideEncryptionByDefault.SseAlgorithm.aes256` |
| `aws:kms:dsse` | `aws.s3.Bucket.BucketEncryption.ServerSideEncryptionRule.ServerSideEncryptionByDefault.SseAlgorithm.aws_kms_dsse` |

Shorthand formats: `aws_kms` or `SseAlgorithm.aws_kms`

### cors_configuration.cors_rules.allowed_methods (AllowedMethods)

| Value | DSL Identifier |
|-------|----------------|
| `GET` | `aws.s3.Bucket.CorsConfiguration.CorsRule.AllowedMethods.get` |
| `PUT` | `aws.s3.Bucket.CorsConfiguration.CorsRule.AllowedMethods.put` |
| `HEAD` | `aws.s3.Bucket.CorsConfiguration.CorsRule.AllowedMethods.head` |
| `POST` | `aws.s3.Bucket.CorsConfiguration.CorsRule.AllowedMethods.post` |
| `DELETE` | `aws.s3.Bucket.CorsConfiguration.CorsRule.AllowedMethods.delete` |

Shorthand formats: `get` or `AllowedMethods.get`

### destination.format (Format)

| Value | DSL Identifier |
|-------|----------------|
| `CSV` | `aws.s3.Bucket.AnalyticsConfiguration.StorageClassAnalysis.DataExport.Destination.Format.csv` |
| `ORC` | `aws.s3.Bucket.AnalyticsConfiguration.StorageClassAnalysis.DataExport.Destination.Format.orc` |
| `Parquet` | `aws.s3.Bucket.AnalyticsConfiguration.StorageClassAnalysis.DataExport.Destination.Format.parquet` |

Shorthand formats: `csv` or `Format.csv`

### intelligent_tiering_configurations.status (Status)

| Value | DSL Identifier |
|-------|----------------|
| `Disabled` | `aws.s3.Bucket.IntelligentTieringConfiguration.Status.disabled` |
| `Enabled` | `aws.s3.Bucket.IntelligentTieringConfiguration.Status.enabled` |

Shorthand formats: `disabled` or `Status.disabled`

### intelligent_tiering_configurations.tierings.access_tier (AccessTier)

| Value | DSL Identifier |
|-------|----------------|
| `ARCHIVE_ACCESS` | `aws.s3.Bucket.IntelligentTieringConfiguration.Tiering.AccessTier.archive_access` |
| `DEEP_ARCHIVE_ACCESS` | `aws.s3.Bucket.IntelligentTieringConfiguration.Tiering.AccessTier.deep_archive_access` |

Shorthand formats: `archive_access` or `AccessTier.archive_access`

### inventory_configurations.included_object_versions (IncludedObjectVersions)

| Value | DSL Identifier |
|-------|----------------|
| `All` | `aws.s3.Bucket.InventoryConfiguration.IncludedObjectVersions.all` |
| `Current` | `aws.s3.Bucket.InventoryConfiguration.IncludedObjectVersions.current` |

Shorthand formats: `all` or `IncludedObjectVersions.all`

### inventory_configurations.optional_fields (OptionalFields)

| Value | DSL Identifier |
|-------|----------------|
| `Size` | `aws.s3.Bucket.InventoryConfiguration.OptionalFields.size` |
| `LastModifiedDate` | `aws.s3.Bucket.InventoryConfiguration.OptionalFields.last_modified_date` |
| `StorageClass` | `aws.s3.Bucket.InventoryConfiguration.OptionalFields.storage_class` |
| `ETag` | `aws.s3.Bucket.InventoryConfiguration.OptionalFields.etag` |
| `IsMultipartUploaded` | `aws.s3.Bucket.InventoryConfiguration.OptionalFields.is_multipart_uploaded` |
| `ReplicationStatus` | `aws.s3.Bucket.InventoryConfiguration.OptionalFields.replication_status` |
| `EncryptionStatus` | `aws.s3.Bucket.InventoryConfiguration.OptionalFields.encryption_status` |
| `ObjectLockRetainUntilDate` | `aws.s3.Bucket.InventoryConfiguration.OptionalFields.object_lock_retain_until_date` |
| `ObjectLockMode` | `aws.s3.Bucket.InventoryConfiguration.OptionalFields.object_lock_mode` |
| `ObjectLockLegalHoldStatus` | `aws.s3.Bucket.InventoryConfiguration.OptionalFields.object_lock_legal_hold_status` |
| `IntelligentTieringAccessTier` | `aws.s3.Bucket.InventoryConfiguration.OptionalFields.intelligent_tiering_access_tier` |
| `BucketKeyStatus` | `aws.s3.Bucket.InventoryConfiguration.OptionalFields.bucket_key_status` |
| `ChecksumAlgorithm` | `aws.s3.Bucket.InventoryConfiguration.OptionalFields.checksum_algorithm` |
| `ObjectAccessControlList` | `aws.s3.Bucket.InventoryConfiguration.OptionalFields.object_access_control_list` |
| `ObjectOwner` | `aws.s3.Bucket.InventoryConfiguration.OptionalFields.object_owner` |
| `LifecycleExpirationDate` | `aws.s3.Bucket.InventoryConfiguration.OptionalFields.lifecycle_expiration_date` |

Shorthand formats: `size` or `OptionalFields.size`

### inventory_configurations.schedule_frequency (ScheduleFrequency)

| Value | DSL Identifier |
|-------|----------------|
| `Daily` | `aws.s3.Bucket.InventoryConfiguration.ScheduleFrequency.daily` |
| `Weekly` | `aws.s3.Bucket.InventoryConfiguration.ScheduleFrequency.weekly` |

Shorthand formats: `daily` or `ScheduleFrequency.daily`

### lifecycle_configuration.rules.noncurrent_version_transitions.storage_class (StorageClass)

| Value | DSL Identifier |
|-------|----------------|
| `DEEP_ARCHIVE` | `aws.s3.Bucket.LifecycleConfiguration.Rule.NoncurrentVersionTransition.StorageClass.deep_archive` |
| `GLACIER` | `aws.s3.Bucket.LifecycleConfiguration.Rule.NoncurrentVersionTransition.StorageClass.glacier` |
| `GLACIER_IR` | `aws.s3.Bucket.LifecycleConfiguration.Rule.NoncurrentVersionTransition.StorageClass.glacier_ir` |
| `INTELLIGENT_TIERING` | `aws.s3.Bucket.LifecycleConfiguration.Rule.NoncurrentVersionTransition.StorageClass.intelligent_tiering` |
| `ONEZONE_IA` | `aws.s3.Bucket.LifecycleConfiguration.Rule.NoncurrentVersionTransition.StorageClass.onezone_ia` |
| `STANDARD_IA` | `aws.s3.Bucket.LifecycleConfiguration.Rule.NoncurrentVersionTransition.StorageClass.standard_ia` |

Shorthand formats: `deep_archive` or `StorageClass.deep_archive`

### lifecycle_configuration.rules.status (Status)

| Value | DSL Identifier |
|-------|----------------|
| `Enabled` | `aws.s3.Bucket.LifecycleConfiguration.Rule.Status.enabled` |
| `Disabled` | `aws.s3.Bucket.LifecycleConfiguration.Rule.Status.disabled` |

Shorthand formats: `enabled` or `Status.enabled`

### lifecycle_configuration.rules.transitions.storage_class (StorageClass)

| Value | DSL Identifier |
|-------|----------------|
| `DEEP_ARCHIVE` | `aws.s3.Bucket.LifecycleConfiguration.Rule.Transition.StorageClass.deep_archive` |
| `GLACIER` | `aws.s3.Bucket.LifecycleConfiguration.Rule.Transition.StorageClass.glacier` |
| `GLACIER_IR` | `aws.s3.Bucket.LifecycleConfiguration.Rule.Transition.StorageClass.glacier_ir` |
| `INTELLIGENT_TIERING` | `aws.s3.Bucket.LifecycleConfiguration.Rule.Transition.StorageClass.intelligent_tiering` |
| `ONEZONE_IA` | `aws.s3.Bucket.LifecycleConfiguration.Rule.Transition.StorageClass.onezone_ia` |
| `STANDARD_IA` | `aws.s3.Bucket.LifecycleConfiguration.Rule.Transition.StorageClass.standard_ia` |

Shorthand formats: `deep_archive` or `StorageClass.deep_archive`

### lifecycle_configuration.transition_default_minimum_object_size (TransitionDefaultMinimumObjectSize)

| Value | DSL Identifier |
|-------|----------------|
| `varies_by_storage_class` | `aws.s3.Bucket.LifecycleConfiguration.TransitionDefaultMinimumObjectSize.varies_by_storage_class` |
| `all_storage_classes_128K` | `aws.s3.Bucket.LifecycleConfiguration.TransitionDefaultMinimumObjectSize.all_storage_classes_128k` |

Shorthand formats: `varies_by_storage_class` or `TransitionDefaultMinimumObjectSize.varies_by_storage_class`

### logging_configuration.target_object_key_format.partitioned_prefix.partition_date_source (PartitionDateSource)

| Value | DSL Identifier |
|-------|----------------|
| `EventTime` | `aws.s3.Bucket.LoggingConfiguration.TargetObjectKeyFormat.PartitionedPrefix.PartitionDateSource.event_time` |
| `DeliveryTime` | `aws.s3.Bucket.LoggingConfiguration.TargetObjectKeyFormat.PartitionedPrefix.PartitionDateSource.delivery_time` |

Shorthand formats: `event_time` or `PartitionDateSource.event_time`

### metadata_configuration.destination.table_bucket_type (TableBucketType)

| Value | DSL Identifier |
|-------|----------------|
| `aws` | `aws.s3.Bucket.MetadataConfiguration.MetadataDestination.TableBucketType.aws` |
| `customer` | `aws.s3.Bucket.MetadataConfiguration.MetadataDestination.TableBucketType.customer` |

Shorthand formats: `aws` or `TableBucketType.aws`

### metadata_configuration.inventory_table_configuration.configuration_state (ConfigurationState)

| Value | DSL Identifier |
|-------|----------------|
| `ENABLED` | `aws.s3.Bucket.MetadataConfiguration.InventoryTableConfiguration.ConfigurationState.enabled` |
| `DISABLED` | `aws.s3.Bucket.MetadataConfiguration.InventoryTableConfiguration.ConfigurationState.disabled` |

Shorthand formats: `enabled` or `ConfigurationState.enabled`

### metadata_configuration.journal_table_configuration.record_expiration.expiration (Expiration)

| Value | DSL Identifier |
|-------|----------------|
| `ENABLED` | `aws.s3.Bucket.MetadataConfiguration.JournalTableConfiguration.RecordExpiration.Expiration.enabled` |
| `DISABLED` | `aws.s3.Bucket.MetadataConfiguration.JournalTableConfiguration.RecordExpiration.Expiration.disabled` |

Shorthand formats: `enabled` or `Expiration.enabled`

### metadata_table_encryption_configuration.sse_algorithm (SseAlgorithm)

| Value | DSL Identifier |
|-------|----------------|
| `aws:kms` | `aws.s3.Bucket.MetadataConfiguration.InventoryTableConfiguration.MetadataTableEncryptionConfiguration.SseAlgorithm.aws_kms` |
| `AES256` | `aws.s3.Bucket.MetadataConfiguration.InventoryTableConfiguration.MetadataTableEncryptionConfiguration.SseAlgorithm.aes256` |

Shorthand formats: `aws_kms` or `SseAlgorithm.aws_kms`

### object_lock_configuration.object_lock_enabled (ObjectLockEnabled)

| Value | DSL Identifier |
|-------|----------------|
| `Enabled` | `aws.s3.Bucket.ObjectLockConfiguration.ObjectLockEnabled.enabled` |

Shorthand formats: `enabled` or `ObjectLockEnabled.enabled`

### object_lock_configuration.rule.default_retention.mode (Mode)

| Value | DSL Identifier |
|-------|----------------|
| `COMPLIANCE` | `aws.s3.Bucket.ObjectLockConfiguration.ObjectLockRule.DefaultRetention.Mode.compliance` |
| `GOVERNANCE` | `aws.s3.Bucket.ObjectLockConfiguration.ObjectLockRule.DefaultRetention.Mode.governance` |

Shorthand formats: `compliance` or `Mode.compliance`

### ownership_controls.rules.object_ownership (ObjectOwnership)

| Value | DSL Identifier |
|-------|----------------|
| `ObjectWriter` | `aws.s3.Bucket.OwnershipControls.OwnershipControlsRule.ObjectOwnership.object_writer` |
| `BucketOwnerPreferred` | `aws.s3.Bucket.OwnershipControls.OwnershipControlsRule.ObjectOwnership.bucket_owner_preferred` |
| `BucketOwnerEnforced` | `aws.s3.Bucket.OwnershipControls.OwnershipControlsRule.ObjectOwnership.bucket_owner_enforced` |

Shorthand formats: `object_writer` or `ObjectOwnership.object_writer`

### replication_configuration.rules.delete_marker_replication.status (Status)

| Value | DSL Identifier |
|-------|----------------|
| `Disabled` | `aws.s3.Bucket.ReplicationConfiguration.ReplicationRule.DeleteMarkerReplication.Status.disabled` |
| `Enabled` | `aws.s3.Bucket.ReplicationConfiguration.ReplicationRule.DeleteMarkerReplication.Status.enabled` |

Shorthand formats: `disabled` or `Status.disabled`

### replication_configuration.rules.destination.access_control_translation.owner (Owner)

| Value | DSL Identifier |
|-------|----------------|
| `Destination` | `aws.s3.Bucket.ReplicationConfiguration.ReplicationRule.ReplicationDestination.AccessControlTranslation.Owner.destination` |

Shorthand formats: `destination` or `Owner.destination`

### replication_configuration.rules.destination.metrics.status (Status)

| Value | DSL Identifier |
|-------|----------------|
| `Disabled` | `aws.s3.Bucket.ReplicationConfiguration.ReplicationRule.ReplicationDestination.Metrics.Status.disabled` |
| `Enabled` | `aws.s3.Bucket.ReplicationConfiguration.ReplicationRule.ReplicationDestination.Metrics.Status.enabled` |

Shorthand formats: `disabled` or `Status.disabled`

### replication_configuration.rules.destination.replication_time.status (Status)

| Value | DSL Identifier |
|-------|----------------|
| `Disabled` | `aws.s3.Bucket.ReplicationConfiguration.ReplicationRule.ReplicationDestination.ReplicationTime.Status.disabled` |
| `Enabled` | `aws.s3.Bucket.ReplicationConfiguration.ReplicationRule.ReplicationDestination.ReplicationTime.Status.enabled` |

Shorthand formats: `disabled` or `Status.disabled`

### replication_configuration.rules.destination.storage_class (StorageClass)

| Value | DSL Identifier |
|-------|----------------|
| `DEEP_ARCHIVE` | `aws.s3.Bucket.ReplicationConfiguration.ReplicationRule.ReplicationDestination.StorageClass.deep_archive` |
| `GLACIER` | `aws.s3.Bucket.ReplicationConfiguration.ReplicationRule.ReplicationDestination.StorageClass.glacier` |
| `GLACIER_IR` | `aws.s3.Bucket.ReplicationConfiguration.ReplicationRule.ReplicationDestination.StorageClass.glacier_ir` |
| `INTELLIGENT_TIERING` | `aws.s3.Bucket.ReplicationConfiguration.ReplicationRule.ReplicationDestination.StorageClass.intelligent_tiering` |
| `ONEZONE_IA` | `aws.s3.Bucket.ReplicationConfiguration.ReplicationRule.ReplicationDestination.StorageClass.onezone_ia` |
| `REDUCED_REDUNDANCY` | `aws.s3.Bucket.ReplicationConfiguration.ReplicationRule.ReplicationDestination.StorageClass.reduced_redundancy` |
| `STANDARD` | `aws.s3.Bucket.ReplicationConfiguration.ReplicationRule.ReplicationDestination.StorageClass.standard` |
| `STANDARD_IA` | `aws.s3.Bucket.ReplicationConfiguration.ReplicationRule.ReplicationDestination.StorageClass.standard_ia` |

Shorthand formats: `deep_archive` or `StorageClass.deep_archive`

### replication_configuration.rules.source_selection_criteria.replica_modifications.status (Status)

| Value | DSL Identifier |
|-------|----------------|
| `Enabled` | `aws.s3.Bucket.ReplicationConfiguration.ReplicationRule.SourceSelectionCriteria.ReplicaModifications.Status.enabled` |
| `Disabled` | `aws.s3.Bucket.ReplicationConfiguration.ReplicationRule.SourceSelectionCriteria.ReplicaModifications.Status.disabled` |

Shorthand formats: `enabled` or `Status.enabled`

### replication_configuration.rules.source_selection_criteria.sse_kms_encrypted_objects.status (Status)

| Value | DSL Identifier |
|-------|----------------|
| `Disabled` | `aws.s3.Bucket.ReplicationConfiguration.ReplicationRule.SourceSelectionCriteria.SseKmsEncryptedObjects.Status.disabled` |
| `Enabled` | `aws.s3.Bucket.ReplicationConfiguration.ReplicationRule.SourceSelectionCriteria.SseKmsEncryptedObjects.Status.enabled` |

Shorthand formats: `disabled` or `Status.disabled`

### replication_configuration.rules.status (Status)

| Value | DSL Identifier |
|-------|----------------|
| `Disabled` | `aws.s3.Bucket.ReplicationConfiguration.ReplicationRule.Status.disabled` |
| `Enabled` | `aws.s3.Bucket.ReplicationConfiguration.ReplicationRule.Status.enabled` |

Shorthand formats: `disabled` or `Status.disabled`

### versioning_configuration.status (Status)

| Value | DSL Identifier |
|-------|----------------|
| `Enabled` | `aws.s3.Bucket.VersioningConfiguration.Status.enabled` |
| `Suspended` | `aws.s3.Bucket.VersioningConfiguration.Status.suspended` |

Shorthand formats: `enabled` or `Status.enabled`

### website_configuration.redirect_all_requests_to.protocol (Protocol)

| Value | DSL Identifier |
|-------|----------------|
| `http` | `aws.s3.Bucket.WebsiteConfiguration.RedirectAllRequestsTo.Protocol.http` |
| `https` | `aws.s3.Bucket.WebsiteConfiguration.RedirectAllRequestsTo.Protocol.https` |

Shorthand formats: `http` or `Protocol.http`

### website_configuration.routing_rules.redirect_rule.protocol (Protocol)

| Value | DSL Identifier |
|-------|----------------|
| `http` | `aws.s3.Bucket.WebsiteConfiguration.RoutingRule.RedirectRule.Protocol.http` |
| `https` | `aws.s3.Bucket.WebsiteConfiguration.RoutingRule.RedirectRule.Protocol.https` |

Shorthand formats: `http` or `Protocol.http`

## Struct Definitions

### accelerate_configuration

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `acceleration_status` | [Enum (AccelerationStatus)](#accelerate_configurationacceleration_status-accelerationstatus) | Yes | Specifies the transfer acceleration status of the bucket. |

### analytics_configurations

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `id` | String | Yes | The ID that identifies the analytics configuration. |
| `prefix` | String | No | The prefix that an object must have to be included in the analytics results. |
| `storage_class_analysis` | [Struct(StorageClassAnalysis)](#analytics_configurationsstorage_class_analysis) | Yes | Contains data related to access patterns to be collected and made available to analyze the tradeoffs between different storage classes. |
| `tag_filters` | `List<Map<String, String>>` | No | The tags to use when evaluating an analytics filter. The analytics only includes objects that meet the filter's criteria. If no filter is specified, all of the contents of the bucket are included in the analysis. |

### analytics_configurations.storage_class_analysis

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `data_export` | [Struct(DataExport)](#analytics_configurationsstorage_class_analysisdata_export) | No | Specifies how data related to the storage class analysis for an Amazon S3 bucket should be exported. |

### analytics_configurations.storage_class_analysis.data_export

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `destination` | [Struct(Destination)](#destination) | Yes | The place to store the data for an analysis. |
| `output_schema_version` | [Enum (OutputSchemaVersion)](#analytics_configurationsstorage_class_analysisdata_exportoutput_schema_version-outputschemaversion) | Yes | The version of the output schema to use when exporting data. Must be ``V_1``. |

### bucket_encryption

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `server_side_encryption_configuration` | List\<[Struct(ServerSideEncryptionRule)](#bucket_encryptionserver_side_encryption_configuration)\> | Yes | Specifies the default server-side-encryption configuration. |

### bucket_encryption.server_side_encryption_configuration

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `blocked_encryption_types` | [Struct(BlockedEncryptionTypes)](#bucket_encryptionserver_side_encryption_configurationblocked_encryption_types) | No | A bucket-level setting for Amazon S3 general purpose buckets used to prevent the upload of new objects encrypted with the specified server-side encryption type. For example, blocking an encryption type will block ``PutObject``, ``CopyObject``, ``PostObject``, multipart upload, and replication requests to the bucket for objects with the specified encryption type. However, you can continue to read and list any pre-existing objects already encrypted with the specified encryption type. For more information, see [Blocking or unblocking SSE-C for a general purpose bucket](https://docs.aws.amazon.com/AmazonS3/latest/userguide/blocking-unblocking-s3-c-encryption-gpb.html). Currently, this parameter only supports blocking or unblocking server-side encryption with customer-provided keys (SSE-C). For more information about SSE-C, see [Using server-side encryption with customer-provided keys (SSE-C)](https://docs.aws.amazon.com/AmazonS3/latest/userguide/ServerSideEncryptionCustomerKeys.html). |
| `bucket_key_enabled` | Bool | No | Specifies whether Amazon S3 should use an S3 Bucket Key with server-side encryption using KMS (SSE-KMS) for new objects in the bucket. Existing objects are not affected. Setting the ``BucketKeyEnabled`` element to ``true`` causes Amazon S3 to use an S3 Bucket Key. By default, S3 Bucket Key is not enabled. For more information, see [Amazon S3 Bucket Keys](https://docs.aws.amazon.com/AmazonS3/latest/dev/bucket-key.html) in the *Amazon S3 User Guide*. |
| `server_side_encryption_by_default` | [Struct(ServerSideEncryptionByDefault)](#bucket_encryptionserver_side_encryption_configurationserver_side_encryption_by_default) | No | Specifies the default server-side encryption to apply to new objects in the bucket. If a PUT Object request doesn't specify any server-side encryption, this default encryption will be applied. |

### bucket_encryption.server_side_encryption_configuration.blocked_encryption_types

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `encryption_type` | List\<[Enum (EncryptionType)](#bucket_encryptionserver_side_encryption_configurationblocked_encryption_typesencryption_type-encryptiontype)\> | No | The object encryption type that you want to block or unblock for an Amazon S3 general purpose bucket. Currently, this parameter only supports blocking or unblocking server side encryption with customer-provided keys (SSE-C). For more information about SSE-C, see [Using server-side encryption with customer-provided keys (SSE-C)](https://docs.aws.amazon.com/AmazonS3/latest/userguide/ServerSideEncryptionCustomerKeys.html). |

### bucket_encryption.server_side_encryption_configuration.server_side_encryption_by_default

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `kms_master_key_id` | KmsKeyId | No | AWS Key Management Service (KMS) customer managed key ID to use for the default encryption. + *General purpose buckets* - This parameter is allowed if and only if ``SSEAlgorithm`` is set to ``aws:kms`` or ``aws:kms:dsse``. + *Directory buckets* - This parameter is allowed if and only if ``SSEAlgorithm`` is set to ``aws:kms``. You can specify the key ID, key alias, or the Amazon Resource Name (ARN) of the KMS key. + Key ID: ``1234abcd-12ab-34cd-56ef-1234567890ab`` + Key ARN: ``arn:aws:kms:us-east-2:111122223333:key/1234abcd-12ab-34cd-56ef-1234567890ab`` + Key Alias: ``alias/alias-name`` If you are using encryption with cross-account or AWS service operations, you must use a fully qualified KMS key ARN. For more information, see [Using encryption for cross-account operations](https://docs.aws.amazon.com/AmazonS3/latest/dev/bucket-encryption.html#bucket-encryption-update-bucket-policy). + *General purpose buckets* - If you're specifying a customer managed KMS key, we recommend using a fully qualified KMS key ARN. If you use a KMS key alias instead, then KMS resolves the key within the requester?s account. This behavior can result in data that's encrypted with a KMS key that belongs to the requester, and not the bucket owner. Also, if you use a key ID, you can run into a LogDestination undeliverable error when creating a VPC flow log. + *Directory buckets* - When you specify an [customer managed key](https://docs.aws.amazon.com/kms/latest/developerguide/concepts.html#customer-cmk) for encryption in your directory bucket, only use the key ID or key ARN. The key alias format of the KMS key isn't supported. Amazon S3 only supports symmetric encryption KMS keys. For more information, see [Asymmetric keys in KMS](https://docs.aws.amazon.com//kms/latest/developerguide/symmetric-asymmetric.html) in the *Key Management Service Developer Guide*. |
| `sse_algorithm` | [Enum (SseAlgorithm)](#bucket_encryptionserver_side_encryption_configurationserver_side_encryption_by_defaultsse_algorithm-ssealgorithm) | Yes | Server-side encryption algorithm to use for the default encryption. For directory buckets, there are only two supported values for server-side encryption: ``AES256`` and ``aws:kms``. |

### cors_configuration

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `cors_rules` | List\<[Struct(CorsRule)](#cors_configurationcors_rules)\> | Yes | A set of origins and methods (cross-origin access that you want to allow). You can add up to 100 rules to the configuration. |

### cors_configuration.cors_rules

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `allowed_headers` | `List<String>` | No | Headers that are specified in the ``Access-Control-Request-Headers`` header. These headers are allowed in a preflight OPTIONS request. In response to any preflight OPTIONS request, Amazon S3 returns any requested headers that are allowed. |
| `allowed_methods` | List\<[Enum (AllowedMethods)](#cors_configurationcors_rulesallowed_methods-allowedmethods)\> | Yes | An HTTP method that you allow the origin to run. *Allowed values*: ``GET`` | ``PUT`` | ``HEAD`` | ``POST`` | ``DELETE`` |
| `allowed_origins` | `List<String>` | Yes | One or more origins you want customers to be able to access the bucket from. |
| `exposed_headers` | `List<String>` | No | One or more headers in the response that you want customers to be able to access from their applications (for example, from a JavaScript ``XMLHttpRequest`` object). |
| `id` | String(len: ..=255) | No | A unique identifier for this rule. The value must be no more than 255 characters. |
| `max_age` | Int(0..) | No | The time in seconds that your browser is to cache the preflight response for the specified resource. |

### destination

Used by: `analytics_configurations.storage_class_analysis.data_export.destination`, `inventory_configurations.destination`

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `bucket_account_id` | AwsAccountId | No | The account ID that owns the destination S3 bucket. If no account ID is provided, the owner is not validated before exporting data. Although this value is optional, we strongly recommend that you set it to help prevent problems if the destination bucket ownership changes. |
| `bucket_arn` | Arn | Yes | The Amazon Resource Name (ARN) of the bucket to which data is exported. |
| `format` | [Enum (Format)](#destinationformat-format) | Yes | Specifies the file format used when exporting data to Amazon S3. *Allowed values*: ``CSV`` | ``ORC`` | ``Parquet`` |
| `prefix` | String | No | The prefix to use when exporting data. The prefix is prepended to all results. |

### intelligent_tiering_configurations

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `id` | String | Yes | The ID used to identify the S3 Intelligent-Tiering configuration. |
| `prefix` | String | No | An object key name prefix that identifies the subset of objects to which the rule applies. |
| `status` | [Enum (Status)](#intelligent_tiering_configurationsstatus-status) | Yes | Specifies the status of the configuration. |
| `tag_filters` | `List<Map<String, String>>` | No | A container for a key-value pair. |
| `tierings` | List\<[Struct(Tiering)](#intelligent_tiering_configurationstierings)\> | Yes | Specifies a list of S3 Intelligent-Tiering storage class tiers in the configuration. At least one tier must be defined in the list. At most, you can specify two tiers in the list, one for each available AccessTier: ``ARCHIVE_ACCESS`` and ``DEEP_ARCHIVE_ACCESS``. You only need Intelligent Tiering Configuration enabled on a bucket if you want to automatically move objects stored in the Intelligent-Tiering storage class to Archive Access or Deep Archive Access tiers. |

### intelligent_tiering_configurations.tierings

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `access_tier` | [Enum (AccessTier)](#intelligent_tiering_configurationstieringsaccess_tier-accesstier) | Yes | S3 Intelligent-Tiering access tier. See [Storage class for automatically optimizing frequently and infrequently accessed objects](https://docs.aws.amazon.com/AmazonS3/latest/dev/storage-class-intro.html#sc-dynamic-data-access) for a list of access tiers in the S3 Intelligent-Tiering storage class. |
| `days` | Int | Yes | The number of consecutive days of no access after which an object will be eligible to be transitioned to the corresponding tier. The minimum number of days specified for Archive Access tier must be at least 90 days and Deep Archive Access tier must be at least 180 days. The maximum can be up to 2 years (730 days). |

### inventory_configurations

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `destination` | [Struct(Destination)](#destination) | Yes | Contains information about where to publish the inventory results. |
| `enabled` | Bool | Yes | Specifies whether the inventory is enabled or disabled. If set to ``True``, an inventory list is generated. If set to ``False``, no inventory list is generated. |
| `id` | String | Yes | The ID used to identify the inventory configuration. |
| `included_object_versions` | [Enum (IncludedObjectVersions)](#inventory_configurationsincluded_object_versions-includedobjectversions) | Yes | Object versions to include in the inventory list. If set to ``All``, the list includes all the object versions, which adds the version-related fields ``VersionId``, ``IsLatest``, and ``DeleteMarker`` to the list. If set to ``Current``, the list does not contain these version-related fields. |
| `optional_fields` | List\<[Enum (OptionalFields)](#inventory_configurationsoptional_fields-optionalfields)\> | No | Contains the optional fields that are included in the inventory results. |
| `prefix` | String | No | Specifies the inventory filter prefix. |
| `schedule_frequency` | [Enum (ScheduleFrequency)](#inventory_configurationsschedule_frequency-schedulefrequency) | Yes | Specifies the schedule for generating inventory results. |

### lifecycle_configuration

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `rules` | List\<[Struct(Rule)](#lifecycle_configurationrules)\> | Yes | A lifecycle rule for individual objects in an Amazon S3 bucket. |
| `transition_default_minimum_object_size` | [Enum (TransitionDefaultMinimumObjectSize)](#lifecycle_configurationtransition_default_minimum_object_size-transitiondefaultminimumobjectsize) | No | Indicates which default minimum object size behavior is applied to the lifecycle configuration. This parameter applies to general purpose buckets only. It isn't supported for directory bucket lifecycle configurations. + ``all_storage_classes_128K`` - Objects smaller than 128 KB will not transition to any storage class by default. + ``varies_by_storage_class`` - Objects smaller than 128 KB will transition to Glacier Flexible Retrieval or Glacier Deep Archive storage classes. By default, all other storage classes will prevent transitions smaller than 128 KB. To customize the minimum object size for any transition you can add a filter that specifies a custom ``ObjectSizeGreaterThan`` or ``ObjectSizeLessThan`` in the body of your transition rule. Custom filters always take precedence over the default transition behavior. |

### lifecycle_configuration.rules

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `abort_incomplete_multipart_upload` | [Struct(AbortIncompleteMultipartUpload)](#lifecycle_configurationrulesabort_incomplete_multipart_upload) | No | Specifies a lifecycle rule that stops incomplete multipart uploads to an Amazon S3 bucket. |
| `expiration_date` | String | No | Indicates when objects are deleted from Amazon S3 and Amazon S3 Glacier. The date value must be in ISO 8601 format. The time is always midnight UTC. If you specify an expiration and transition time, you must use the same time unit for both properties (either in days or by date). The expiration time must also be later than the transition time. |
| `expiration_in_days` | Int | No | Indicates the number of days after creation when objects are deleted from Amazon S3 and Amazon S3 Glacier. If you specify an expiration and transition time, you must use the same time unit for both properties (either in days or by date). The expiration time must also be later than the transition time. |
| `expired_object_delete_marker` | Bool | No | Indicates whether Amazon S3 will remove a delete marker without any noncurrent versions. If set to true, the delete marker will be removed if there are no noncurrent versions. This cannot be specified with ``ExpirationInDays``, ``ExpirationDate``, or ``TagFilters``. |
| `id` | String(len: ..=255) | No | Unique identifier for the rule. The value can't be longer than 255 characters. |
| `noncurrent_version_expiration` | [Struct(NoncurrentVersionExpiration)](#lifecycle_configurationrulesnoncurrent_version_expiration) | No | Specifies when noncurrent object versions expire. Upon expiration, S3 permanently deletes the noncurrent object versions. You set this lifecycle configuration action on a bucket that has versioning enabled (or suspended) to request that S3 delete noncurrent object versions at a specific period in the object's lifetime. |
| `noncurrent_version_transitions` | List\<[Struct(NoncurrentVersionTransition)](#lifecycle_configurationrulesnoncurrent_version_transitions)\> | No | For buckets with versioning enabled (or suspended), one or more transition rules that specify when non-current objects transition to a specified storage class. If you specify a transition and expiration time, the expiration time must be later than the transition time. If you specify this property, don't specify the ``NoncurrentVersionTransition`` property. |
| `object_size_greater_than` | NumericString(len: ..=20) | No | Specifies the minimum object size in bytes for this rule to apply to. Objects must be larger than this value in bytes. For more information about size based rules, see [Lifecycle configuration using size-based rules](https://docs.aws.amazon.com/AmazonS3/latest/userguide/lifecycle-configuration-examples.html#lc-size-rules) in the *Amazon S3 User Guide*. |
| `object_size_less_than` | NumericString(len: ..=20) | No | Specifies the maximum object size in bytes for this rule to apply to. Objects must be smaller than this value in bytes. For more information about sized based rules, see [Lifecycle configuration using size-based rules](https://docs.aws.amazon.com/AmazonS3/latest/userguide/lifecycle-configuration-examples.html#lc-size-rules) in the *Amazon S3 User Guide*. |
| `prefix` | String | No | Object key prefix that identifies one or more objects to which this rule applies. Replacement must be made for object keys containing special characters (such as carriage returns) when using XML requests. For more information, see [XML related object key constraints](https://docs.aws.amazon.com/AmazonS3/latest/userguide/object-keys.html#object-key-xml-related-constraints). |
| `status` | [Enum (Status)](#lifecycle_configurationrulesstatus-status) | Yes | If ``Enabled``, the rule is currently being applied. If ``Disabled``, the rule is not currently being applied. |
| `tag_filters` | `List<Map<String, String>>` | No | Tags to use to identify a subset of objects to which the lifecycle rule applies. |
| `transitions` | List\<[Struct(Transition)](#lifecycle_configurationrulestransitions)\> | No | One or more transition rules that specify when an object transitions to a specified storage class. If you specify an expiration and transition time, you must use the same time unit for both properties (either in days or by date). The expiration time must also be later than the transition time. If you specify this property, don't specify the ``Transition`` property. |

### lifecycle_configuration.rules.abort_incomplete_multipart_upload

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `days_after_initiation` | Int(0..) | Yes | Specifies the number of days after which Amazon S3 stops an incomplete multipart upload. |

### lifecycle_configuration.rules.noncurrent_version_expiration

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `newer_noncurrent_versions` | Int | No | Specifies how many noncurrent versions S3 will retain. If there are this many more recent noncurrent versions, S3 will take the associated action. For more information about noncurrent versions, see [Lifecycle configuration elements](https://docs.aws.amazon.com/AmazonS3/latest/userguide/intro-lifecycle-rules.html) in the *Amazon S3 User Guide*. |
| `noncurrent_days` | Int | Yes | Specifies the number of days an object is noncurrent before S3 can perform the associated action. For information about the noncurrent days calculations, see [How Amazon S3 Calculates When an Object Became Noncurrent](https://docs.aws.amazon.com/AmazonS3/latest/dev/intro-lifecycle-rules.html#non-current-days-calculations) in the *Amazon S3 User Guide*. |

### lifecycle_configuration.rules.noncurrent_version_transitions

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `newer_noncurrent_versions` | Int | No | Specifies how many noncurrent versions S3 will retain. If there are this many more recent noncurrent versions, S3 will take the associated action. For more information about noncurrent versions, see [Lifecycle configuration elements](https://docs.aws.amazon.com/AmazonS3/latest/userguide/intro-lifecycle-rules.html) in the *Amazon S3 User Guide*. |
| `storage_class` | [Enum (StorageClass)](#lifecycle_configurationrulesnoncurrent_version_transitionsstorage_class-storageclass) | Yes | The class of storage used to store the object. |
| `transition_in_days` | Int | Yes | Specifies the number of days an object is noncurrent before Amazon S3 can perform the associated action. For information about the noncurrent days calculations, see [How Amazon S3 Calculates How Long an Object Has Been Noncurrent](https://docs.aws.amazon.com/AmazonS3/latest/dev/intro-lifecycle-rules.html#non-current-days-calculations) in the *Amazon S3 User Guide*. |

### lifecycle_configuration.rules.transitions

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `storage_class` | [Enum (StorageClass)](#lifecycle_configurationrulestransitionsstorage_class-storageclass) | Yes | The storage class to which you want the object to transition. |
| `transition_date` | String | No | Indicates when objects are transitioned to the specified storage class. The date value must be in ISO 8601 format. The time is always midnight UTC. |
| `transition_in_days` | Int | No | Indicates the number of days after creation when objects are transitioned to the specified storage class. If the specified storage class is ``INTELLIGENT_TIERING``, ``GLACIER_IR``, ``GLACIER``, or ``DEEP_ARCHIVE``, valid values are ``0`` or positive integers. If the specified storage class is ``STANDARD_IA`` or ``ONEZONE_IA``, valid values are positive integers greater than ``30``. Be aware that some storage classes have a minimum storage duration and that you're charged for transitioning objects before their minimum storage duration. For more information, see [Constraints and considerations for transitions](https://docs.aws.amazon.com/AmazonS3/latest/userguide/lifecycle-transition-general-considerations.html#lifecycle-configuration-constraints) in the *Amazon S3 User Guide*. |

### logging_configuration

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `destination_bucket_name` | String | No | The name of the bucket where Amazon S3 should store server access log files. You can store log files in any bucket that you own. By default, logs are stored in the bucket where the ``LoggingConfiguration`` property is defined. |
| `log_file_prefix` | String | No | A prefix for all log object keys. If you store log files from multiple Amazon S3 buckets in a single bucket, you can use a prefix to distinguish which log files came from which bucket. |
| `target_object_key_format` | [Struct(TargetObjectKeyFormat)](#logging_configurationtarget_object_key_format) | No | Amazon S3 key format for log objects. Only one format, either PartitionedPrefix or SimplePrefix, is allowed. |

### logging_configuration.target_object_key_format

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `partitioned_prefix` | [Struct(PartitionedPrefix)](#logging_configurationtarget_object_key_formatpartitioned_prefix) | No |  |
| `simple_prefix` | Empty struct (`{}`) | No | This format defaults the prefix to the given log file prefix for delivering server access log file. |

### logging_configuration.target_object_key_format.partitioned_prefix

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `partition_date_source` | [Enum (PartitionDateSource)](#logging_configurationtarget_object_key_formatpartitioned_prefixpartition_date_source-partitiondatesource) | No | Specifies the partition date source for the partitioned prefix. ``PartitionDateSource`` can be ``EventTime`` or ``DeliveryTime``. For ``DeliveryTime``, the time in the log file names corresponds to the delivery time for the log files. For ``EventTime``, The logs delivered are for a specific day only. The year, month, and day correspond to the day on which the event occurred, and the hour, minutes and seconds are set to 00 in the key. |

### metadata_configuration

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `destination` | [Struct(MetadataDestination)](#metadata_configurationdestination) | No | The destination information for the S3 Metadata configuration. Provider-populated; cannot be set. |
| `inventory_table_configuration` | [Struct(InventoryTableConfiguration)](#metadata_configurationinventory_table_configuration) | No | The inventory table configuration for a metadata configuration. |
| `journal_table_configuration` | [Struct(JournalTableConfiguration)](#metadata_configurationjournal_table_configuration) | Yes | The journal table configuration for a metadata configuration. |

### metadata_configuration.destination

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `table_bucket_arn` | Arn | No | The Amazon Resource Name (ARN) of the table bucket where the metadata configuration is stored. |
| `table_bucket_type` | [Enum (TableBucketType)](#metadata_configurationdestinationtable_bucket_type-tablebuckettype) | Yes | The type of the table bucket where the metadata configuration is stored. The ``aws`` value indicates an AWS managed table bucket, and the ``customer`` value indicates a customer-managed table bucket. V2 metadata configurations are stored in AWS managed table buckets, and V1 metadata configurations are stored in customer-managed table buckets. |
| `table_namespace` | String | No | The namespace in the table bucket where the metadata tables for a metadata configuration are stored. |

### metadata_configuration.inventory_table_configuration

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `configuration_state` | [Enum (ConfigurationState)](#metadata_configurationinventory_table_configurationconfiguration_state-configurationstate) | Yes | The configuration state of the inventory table, indicating whether the inventory table is enabled or disabled. |
| `encryption_configuration` | [Struct(MetadataTableEncryptionConfiguration)](#metadata_table_encryption_configuration) | No | The encryption configuration for the inventory table. |
| `table_arn` | Arn | No | The Amazon Resource Name (ARN) for the inventory table. Provider-populated; cannot be set. |
| `table_name` | String | No | The name of the inventory table. Provider-populated; cannot be set. |

### metadata_configuration.journal_table_configuration

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `encryption_configuration` | [Struct(MetadataTableEncryptionConfiguration)](#metadata_table_encryption_configuration) | No | The encryption configuration for the journal table. |
| `record_expiration` | [Struct(RecordExpiration)](#metadata_configurationjournal_table_configurationrecord_expiration) | Yes | The journal table record expiration settings for the journal table. |
| `table_arn` | Arn | No | The Amazon Resource Name (ARN) for the journal table. Provider-populated; cannot be set. |
| `table_name` | String | No | The name of the journal table. Provider-populated; cannot be set. |

### metadata_configuration.journal_table_configuration.record_expiration

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `days` | Int | No | If you enable journal table record expiration, you can set the number of days to retain your journal table records. Journal table records must be retained for a minimum of 7 days. To set this value, specify any whole number from ``7`` to ``2147483647``. For example, to retain your journal table records for one year, set this value to ``365``. |
| `expiration` | [Enum (Expiration)](#metadata_configurationjournal_table_configurationrecord_expirationexpiration-expiration) | Yes | Specifies whether journal table record expiration is enabled or disabled. |

### metadata_table_configuration

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `s3_tables_destination` | [Struct(S3TablesDestination)](#metadata_table_configurations3_tables_destination) | Yes | The destination information for the metadata table configuration. The destination table bucket must be in the same Region and AWS-account as the general purpose bucket. The specified metadata table name must be unique within the ``aws_s3_metadata`` namespace in the destination table bucket. |

### metadata_table_configuration.s3_tables_destination

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `table_arn` | Arn | No | The Amazon Resource Name (ARN) for the metadata table in the metadata table configuration. The specified metadata table name must be unique within the ``aws_s3_metadata`` namespace in the destination table bucket. Provider-populated; cannot be set. |
| `table_bucket_arn` | Arn | Yes | The Amazon Resource Name (ARN) for the table bucket that's specified as the destination in the metadata table configuration. The destination table bucket must be in the same Region and AWS-account as the general purpose bucket. |
| `table_name` | String | Yes | The name for the metadata table in your metadata table configuration. The specified metadata table name must be unique within the ``aws_s3_metadata`` namespace in the destination table bucket. |
| `table_namespace` | String | No | The table bucket namespace for the metadata table in your metadata table configuration. This value is always ``aws_s3_metadata``. Provider-populated; cannot be set. |

### metadata_table_encryption_configuration

Used by: `metadata_configuration.inventory_table_configuration.encryption_configuration`, `metadata_configuration.journal_table_configuration.encryption_configuration`

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `kms_key_arn` | KmsKeyArn | No | If server-side encryption with KMSlong (KMS) keys (SSE-KMS) is specified, you must also specify the KMS key Amazon Resource Name (ARN). You must specify a customer-managed KMS key that's located in the same Region as the general purpose bucket that corresponds to the metadata table configuration. |
| `sse_algorithm` | [Enum (SseAlgorithm)](#metadata_table_encryption_configurationsse_algorithm-ssealgorithm) | Yes | The encryption type specified for a metadata table. To specify server-side encryption with KMSlong (KMS) keys (SSE-KMS), use the ``aws:kms`` value. To specify server-side encryption with Amazon S3 managed keys (SSE-S3), use the ``AES256`` value. |

### metrics_configurations

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `access_point_arn` | Arn | No | The access point that was used while performing operations on the object. The metrics configuration only includes objects that meet the filter's criteria. |
| `id` | String | Yes | The ID used to identify the metrics configuration. This can be any value you choose that helps you identify your metrics configuration. |
| `prefix` | String | No | The prefix that an object must have to be included in the metrics results. |
| `tag_filters` | `List<Map<String, String>>` | No | Specifies a list of tag filters to use as a metrics configuration filter. The metrics configuration includes only objects that meet the filter's criteria. |

### notification_configuration

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `event_bridge_configuration` | [Struct(EventBridgeConfiguration)](#notification_configurationevent_bridge_configuration) | No | Enables delivery of events to Amazon EventBridge. |
| `lambda_configurations` | List\<[Struct(LambdaConfiguration)](#notification_configurationlambda_configurations)\> | No | Describes the LAMlong functions to invoke and the events for which to invoke them. |
| `queue_configurations` | List\<[Struct(QueueConfiguration)](#notification_configurationqueue_configurations)\> | No | The Amazon Simple Queue Service queues to publish messages to and the events for which to publish messages. |
| `topic_configurations` | List\<[Struct(TopicConfiguration)](#notification_configurationtopic_configurations)\> | No | The topic to which notifications are sent and the events for which notifications are generated. |

### notification_configuration.event_bridge_configuration

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `event_bridge_enabled` | Bool | Yes | Enables delivery of events to Amazon EventBridge. |

### notification_configuration.lambda_configurations

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `event` | String | Yes | The Amazon S3 bucket event for which to invoke the LAMlong function. For more information, see [Supported Event Types](https://docs.aws.amazon.com/AmazonS3/latest/dev/NotificationHowTo.html) in the *Amazon S3 User Guide*. |
| `filter` | [Struct(NotificationFilter)](#notification_filter) | No | The filtering rules that determine which objects invoke the AWS Lambda function. For example, you can create a filter so that only image files with a ``.jpg`` extension invoke the function when they are added to the Amazon S3 bucket. |
| `function` | Arn | Yes | The Amazon Resource Name (ARN) of the LAMlong function that Amazon S3 invokes when the specified event type occurs. |

### notification_configuration.queue_configurations

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `event` | String | Yes | The Amazon S3 bucket event about which you want to publish messages to Amazon SQS. For more information, see [Supported Event Types](https://docs.aws.amazon.com/AmazonS3/latest/dev/NotificationHowTo.html) in the *Amazon S3 User Guide*. |
| `filter` | [Struct(NotificationFilter)](#notification_filter) | No | The filtering rules that determine which objects trigger notifications. For example, you can create a filter so that Amazon S3 sends notifications only when image files with a ``.jpg`` extension are added to the bucket. For more information, see [Configuring event notifications using object key name filtering](https://docs.aws.amazon.com/AmazonS3/latest/user-guide/notification-how-to-filtering.html) in the *Amazon S3 User Guide*. |
| `queue` | Arn | Yes | The Amazon Resource Name (ARN) of the Amazon SQS queue to which Amazon S3 publishes a message when it detects events of the specified type. FIFO queues are not allowed when enabling an SQS queue as the event notification destination. |

### notification_configuration.topic_configurations

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `event` | String | Yes | The Amazon S3 bucket event about which to send notifications. For more information, see [Supported Event Types](https://docs.aws.amazon.com/AmazonS3/latest/dev/NotificationHowTo.html) in the *Amazon S3 User Guide*. |
| `filter` | [Struct(NotificationFilter)](#notification_filter) | No | The filtering rules that determine for which objects to send notifications. For example, you can create a filter so that Amazon S3 sends notifications only when image files with a ``.jpg`` extension are added to the bucket. |
| `topic` | Arn | Yes | The Amazon Resource Name (ARN) of the Amazon SNS topic to which Amazon S3 publishes a message when it detects events of the specified type. |

### notification_filter

Used by: `notification_configuration.lambda_configurations.filter`, `notification_configuration.queue_configurations.filter`, `notification_configuration.topic_configurations.filter`

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `s3_key` | [Struct(S3KeyFilter)](#notification_filters3_key) | Yes | A container for object key name prefix and suffix filtering rules. |

### notification_filter.s3_key

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `rules` | List\<[Struct(FilterRule)](#notification_filters3_keyrules)\> | Yes | A list of containers for the key-value pair that defines the criteria for the filter rule. |

### notification_filter.s3_key.rules

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `name` | String(len: ..=1024) | Yes | The object key name prefix or suffix identifying one or more objects to which the filtering rule applies. The maximum length is 1,024 characters. Overlapping prefixes and suffixes are not supported. For more information, see [Configuring Event Notifications](https://docs.aws.amazon.com/AmazonS3/latest/dev/NotificationHowTo.html) in the *Amazon S3 User Guide*. |
| `value` | String | Yes | The value that the filter searches for in object key names. |

### object_lock_configuration

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `object_lock_enabled` | [Enum (ObjectLockEnabled)](#object_lock_configurationobject_lock_enabled-objectlockenabled) | No | Indicates whether this bucket has an Object Lock configuration enabled. Enable ``ObjectLockEnabled`` when you apply ``ObjectLockConfiguration`` to a bucket. |
| `rule` | [Struct(ObjectLockRule)](#object_lock_configurationrule) | No | Specifies the Object Lock rule for the specified object. Enable this rule when you apply ``ObjectLockConfiguration`` to a bucket. If Object Lock is turned on, bucket settings require both ``Mode`` and a period of either ``Days`` or ``Years``. You cannot specify ``Days`` and ``Years`` at the same time. For more information, see [ObjectLockRule](https://docs.aws.amazon.com/AWSCloudFormation/latest/UserGuide/aws-properties-s3-bucket-objectlockrule.html) and [DefaultRetention](https://docs.aws.amazon.com/AWSCloudFormation/latest/UserGuide/aws-properties-s3-bucket-defaultretention.html). |

### object_lock_configuration.rule

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `default_retention` | [Struct(DefaultRetention)](#object_lock_configurationruledefault_retention) | No | The default Object Lock retention mode and period that you want to apply to new objects placed in the specified bucket. If Object Lock is turned on, bucket settings require both ``Mode`` and a period of either ``Days`` or ``Years``. You cannot specify ``Days`` and ``Years`` at the same time. For more information about allowable values for mode and period, see [DefaultRetention](https://docs.aws.amazon.com/AWSCloudFormation/latest/UserGuide/aws-properties-s3-bucket-defaultretention.html). |

### object_lock_configuration.rule.default_retention

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `days` | Int | No | The number of days that you want to specify for the default retention period. If Object Lock is turned on, you must specify ``Mode`` and specify either ``Days`` or ``Years``. |
| `mode` | [Enum (Mode)](#object_lock_configurationruledefault_retentionmode-mode) | No | The default Object Lock retention mode you want to apply to new objects placed in the specified bucket. If Object Lock is turned on, you must specify ``Mode`` and specify either ``Days`` or ``Years``. |
| `years` | Int | No | The number of years that you want to specify for the default retention period. If Object Lock is turned on, you must specify ``Mode`` and specify either ``Days`` or ``Years``. |

### ownership_controls

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `rules` | List\<[Struct(OwnershipControlsRule)](#ownership_controlsrules)\> | Yes | Specifies the container element for Object Ownership rules. |

### ownership_controls.rules

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `object_ownership` | [Enum (ObjectOwnership)](#ownership_controlsrulesobject_ownership-objectownership) | No | Specifies an object ownership rule. |

### public_access_block_configuration

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `block_public_acls` | Bool | No | Specifies whether Amazon S3 should block public access control lists (ACLs) for this bucket and objects in this bucket. Setting this element to ``TRUE`` causes the following behavior: + PUT Bucket ACL and PUT Object ACL calls fail if the specified ACL is public. + PUT Object calls fail if the request includes a public ACL. + PUT Bucket calls fail if the request includes a public ACL. Enabling this setting doesn't affect existing policies or ACLs. |
| `block_public_policy` | Bool | No | Specifies whether Amazon S3 should block public bucket policies for this bucket. Setting this element to ``TRUE`` causes Amazon S3 to reject calls to PUT Bucket policy if the specified bucket policy allows public access. Enabling this setting doesn't affect existing bucket policies. |
| `ignore_public_acls` | Bool | No | Specifies whether Amazon S3 should ignore public ACLs for this bucket and objects in this bucket. Setting this element to ``TRUE`` causes Amazon S3 to ignore all public ACLs on this bucket and objects in this bucket. Enabling this setting doesn't affect the persistence of any existing ACLs and doesn't prevent new public ACLs from being set. |
| `restrict_public_buckets` | Bool | No | Specifies whether Amazon S3 should restrict public bucket policies for this bucket. Setting this element to ``TRUE`` restricts access to this bucket to only AWS-service principals and authorized users within this account if the bucket has a public policy. Enabling this setting doesn't affect previously stored bucket policies, except that public and cross-account access within any public bucket policy, including non-public delegation to specific accounts, is blocked. |

### replication_configuration

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `role` | IamRoleArn | Yes | The Amazon Resource Name (ARN) of the IAMlong (IAM) role that Amazon S3 assumes when replicating objects. For more information, see [How to Set Up Replication](https://docs.aws.amazon.com/AmazonS3/latest/dev/replication-how-setup.html) in the *Amazon S3 User Guide*. |
| `rules` | List\<[Struct(ReplicationRule)](#replication_configurationrules)\> | Yes | A container for one or more replication rules. A replication configuration must have at least one rule and can contain a maximum of 1,000 rules. |

### replication_configuration.rules

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `delete_marker_replication` | [Struct(DeleteMarkerReplication)](#replication_configurationrulesdelete_marker_replication) | No | Specifies whether Amazon S3 replicates delete markers. If you specify a ``Filter`` in your replication configuration, you must also include a ``DeleteMarkerReplication`` element. If your ``Filter`` includes a ``Tag`` element, the ``DeleteMarkerReplication````Status`` must be set to Disabled, because Amazon S3 does not support replicating delete markers for tag-based rules. For an example configuration, see [Basic Rule Configuration](https://docs.aws.amazon.com/AmazonS3/latest/dev/replication-add-config.html#replication-config-min-rule-config). For more information about delete marker replication, see [Basic Rule Configuration](https://docs.aws.amazon.com/AmazonS3/latest/dev/delete-marker-replication.html). If you are using an earlier version of the replication configuration, Amazon S3 handles replication of delete markers differently. For more information, see [Backward Compatibility](https://docs.aws.amazon.com/AmazonS3/latest/dev/replication-add-config.html#replication-backward-compat-considerations). |
| `destination` | [Struct(ReplicationDestination)](#replication_configurationrulesdestination) | Yes | A container for information about the replication destination and its configurations including enabling the S3 Replication Time Control (S3 RTC). |
| `filter` | [Struct(ReplicationRuleFilter)](#replication_configurationrulesfilter) | No | A filter that identifies the subset of objects to which the replication rule applies. A ``Filter`` must specify exactly one ``Prefix``, ``TagFilter``, or an ``And`` child element. The use of the filter field indicates that this is a V2 replication configuration. This field isn't supported in a V1 replication configuration. V1 replication configuration only supports filtering by key prefix. To filter using a V1 replication configuration, add the ``Prefix`` directly as a child element of the ``Rule`` element. |
| `id` | String(len: ..=255) | No | A unique identifier for the rule. The maximum value is 255 characters. If you don't specify a value, AWS CloudFormation generates a random ID. When using a V2 replication configuration this property is capitalized as "ID". |
| `prefix` | String(len: ..=1024) | No | An object key name prefix that identifies the object or objects to which the rule applies. The maximum prefix length is 1,024 characters. To include all objects in a bucket, specify an empty string. To filter using a V1 replication configuration, add the ``Prefix`` directly as a child element of the ``Rule`` element. Replacement must be made for object keys containing special characters (such as carriage returns) when using XML requests. For more information, see [XML related object key constraints](https://docs.aws.amazon.com/AmazonS3/latest/userguide/object-keys.html#object-key-xml-related-constraints). |
| `priority` | Int | No | The priority indicates which rule has precedence whenever two or more replication rules conflict. Amazon S3 will attempt to replicate objects according to all replication rules. However, if there are two or more rules with the same destination bucket, then objects will be replicated according to the rule with the highest priority. The higher the number, the higher the priority. For more information, see [Replication](https://docs.aws.amazon.com/AmazonS3/latest/dev/replication.html) in the *Amazon S3 User Guide*. |
| `source_selection_criteria` | [Struct(SourceSelectionCriteria)](#replication_configurationrulessource_selection_criteria) | No | A container that describes additional filters for identifying the source objects that you want to replicate. You can choose to enable or disable the replication of these objects. |
| `status` | [Enum (Status)](#replication_configurationrulesstatus-status) | Yes | Specifies whether the rule is enabled. |

### replication_configuration.rules.delete_marker_replication

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `status` | [Enum (Status)](#replication_configurationrulesdelete_marker_replicationstatus-status) | No | Indicates whether to replicate delete markers. |

### replication_configuration.rules.destination

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `access_control_translation` | [Struct(AccessControlTranslation)](#replication_configurationrulesdestinationaccess_control_translation) | No | Specify this only in a cross-account scenario (where source and destination bucket owners are not the same), and you want to change replica ownership to the AWS-account that owns the destination bucket. If this is not specified in the replication configuration, the replicas are owned by same AWS-account that owns the source object. |
| `account` | AwsAccountId | No | Destination bucket owner account ID. In a cross-account scenario, if you direct Amazon S3 to change replica ownership to the AWS-account that owns the destination bucket by specifying the ``AccessControlTranslation`` property, this is the account ID of the destination bucket owner. For more information, see [Cross-Region Replication Additional Configuration: Change Replica Owner](https://docs.aws.amazon.com/AmazonS3/latest/dev/crr-change-owner.html) in the *Amazon S3 User Guide*. If you specify the ``AccessControlTranslation`` property, the ``Account`` property is required. |
| `bucket` | String | Yes | The Amazon Resource Name (ARN) of the bucket where you want Amazon S3 to store the results. |
| `encryption_configuration` | [Struct(EncryptionConfiguration)](#replication_configurationrulesdestinationencryption_configuration) | No | Specifies encryption-related information. |
| `metrics` | [Struct(Metrics)](#replication_configurationrulesdestinationmetrics) | No | A container specifying replication metrics-related settings enabling replication metrics and events. |
| `replication_time` | [Struct(ReplicationTime)](#replication_configurationrulesdestinationreplication_time) | No | A container specifying S3 Replication Time Control (S3 RTC), including whether S3 RTC is enabled and the time when all objects and operations on objects must be replicated. Must be specified together with a ``Metrics`` block. |
| `storage_class` | [Enum (StorageClass)](#replication_configurationrulesdestinationstorage_class-storageclass) | No | The storage class to use when replicating objects, such as S3 Standard or reduced redundancy. By default, Amazon S3 uses the storage class of the source object to create the object replica. For valid values, see the ``StorageClass`` element of the [PUT Bucket replication](https://docs.aws.amazon.com/AmazonS3/latest/API/RESTBucketPUTreplication.html) action in the *Amazon S3 API Reference*. ``FSX_OPENZFS`` is not an accepted value when replicating objects. |

### replication_configuration.rules.destination.access_control_translation

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `owner` | [Enum (Owner)](#replication_configurationrulesdestinationaccess_control_translationowner-owner) | Yes | Specifies the replica ownership. For default and valid values, see [PUT bucket replication](https://docs.aws.amazon.com/AmazonS3/latest/API/RESTBucketPUTreplication.html) in the *Amazon S3 API Reference*. |

### replication_configuration.rules.destination.encryption_configuration

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `replica_kms_key_id` | KmsKeyId | Yes | Specifies the ID (Key ARN or Alias ARN) of the customer managed AWS KMS key stored in AWS Key Management Service (KMS) for the destination bucket. Amazon S3 uses this key to encrypt replica objects. Amazon S3 only supports symmetric encryption KMS keys. For more information, see [Asymmetric keys in KMS](https://docs.aws.amazon.com//kms/latest/developerguide/symmetric-asymmetric.html) in the *Key Management Service Developer Guide*. |

### replication_configuration.rules.destination.metrics

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `event_threshold` | [Struct(ReplicationTimeValue)](#replication_time_value) | No | A container specifying the time threshold for emitting the ``s3:Replication:OperationMissedThreshold`` event. |
| `status` | [Enum (Status)](#replication_configurationrulesdestinationmetricsstatus-status) | Yes | Specifies whether the replication metrics are enabled. |

### replication_configuration.rules.destination.replication_time

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `status` | [Enum (Status)](#replication_configurationrulesdestinationreplication_timestatus-status) | Yes | Specifies whether the replication time is enabled. |
| `time` | [Struct(ReplicationTimeValue)](#replication_time_value) | Yes | A container specifying the time by which replication should be complete for all objects and operations on objects. |

### replication_configuration.rules.filter

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `and` | [Struct(ReplicationRuleAndOperator)](#replication_configurationrulesfilterand) | No | A container for specifying rule filters. The filters determine the subset of objects to which the rule applies. This element is required only if you specify more than one filter. For example: + If you specify both a ``Prefix`` and a ``TagFilter``, wrap these filters in an ``And`` tag. + If you specify a filter based on multiple tags, wrap the ``TagFilter`` elements in an ``And`` tag. |
| `prefix` | String | No | An object key name prefix that identifies the subset of objects to which the rule applies. Replacement must be made for object keys containing special characters (such as carriage returns) when using XML requests. For more information, see [XML related object key constraints](https://docs.aws.amazon.com/AmazonS3/latest/userguide/object-keys.html#object-key-xml-related-constraints). |
| `tag_filter` | `Map<String, String>` | No | A container for specifying a tag key and value. The rule applies only to objects that have the tag in their tag set. |

### replication_configuration.rules.filter.and

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `prefix` | String | No | An object key name prefix that identifies the subset of objects to which the rule applies. |
| `tag_filters` | `List<Map<String, String>>` | No | An array of tags containing key and value pairs. |

### replication_configuration.rules.source_selection_criteria

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `replica_modifications` | [Struct(ReplicaModifications)](#replication_configurationrulessource_selection_criteriareplica_modifications) | No | A filter that you can specify for selection for modifications on replicas. |
| `sse_kms_encrypted_objects` | [Struct(SseKmsEncryptedObjects)](#replication_configurationrulessource_selection_criteriasse_kms_encrypted_objects) | No | A container for filter information for the selection of Amazon S3 objects encrypted with AWS KMS. |

### replication_configuration.rules.source_selection_criteria.replica_modifications

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `status` | [Enum (Status)](#replication_configurationrulessource_selection_criteriareplica_modificationsstatus-status) | Yes | Specifies whether Amazon S3 replicates modifications on replicas. *Allowed values*: ``Enabled`` | ``Disabled`` |

### replication_configuration.rules.source_selection_criteria.sse_kms_encrypted_objects

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `status` | [Enum (Status)](#replication_configurationrulessource_selection_criteriasse_kms_encrypted_objectsstatus-status) | Yes | Specifies whether Amazon S3 replicates objects created with server-side encryption using an AWS KMS key stored in AWS Key Management Service. |

### replication_time_value

Used by: `replication_configuration.rules.destination.metrics.event_threshold`, `replication_configuration.rules.destination.replication_time.time`

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `minutes` | Int | Yes | Contains an integer specifying time in minutes. Valid value: 15 |

### versioning_configuration

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `status` | [Enum (Status)](#versioning_configurationstatus-status) | Yes | The versioning state of the bucket. |

### website_configuration

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `error_document` | String | No | The name of the error document for the website. |
| `index_document` | String | No | The name of the index document for the website. |
| `redirect_all_requests_to` | [Struct(RedirectAllRequestsTo)](#website_configurationredirect_all_requests_to) | No | The redirect behavior for every request to this bucket's website endpoint. If you specify this property, you can't specify any other property. |
| `routing_rules` | List\<[Struct(RoutingRule)](#website_configurationrouting_rules)\> | No | Rules that define when a redirect is applied and the redirect behavior. |

### website_configuration.redirect_all_requests_to

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `host_name` | String | Yes | Name of the host where requests are redirected. |
| `protocol` | [Enum (Protocol)](#website_configurationredirect_all_requests_toprotocol-protocol) | No | Protocol to use when redirecting requests. The default is the protocol that is used in the original request. |

### website_configuration.routing_rules

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `redirect_rule` | [Struct(RedirectRule)](#website_configurationrouting_rulesredirect_rule) | Yes | Container for redirect information. You can redirect requests to another host, to another page, or with another protocol. In the event of an error, you can specify a different error code to return. |
| `routing_rule_condition` | [Struct(RoutingRuleCondition)](#website_configurationrouting_rulesrouting_rule_condition) | No | A container for describing a condition that must be met for the specified redirect to apply. For example, 1. If request is for pages in the ``/docs`` folder, redirect to the ``/documents`` folder. 2. If request results in HTTP error 4xx, redirect request to another host where you might process the error. |

### website_configuration.routing_rules.redirect_rule

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `host_name` | String | No | The host name to use in the redirect request. |
| `http_redirect_code` | String | No | The HTTP redirect code to use on the response. Not required if one of the siblings is present. |
| `protocol` | [Enum (Protocol)](#website_configurationrouting_rulesredirect_ruleprotocol-protocol) | No | Protocol to use when redirecting requests. The default is the protocol that is used in the original request. |
| `replace_key_prefix_with` | String | No | The object key prefix to use in the redirect request. For example, to redirect requests for all pages with prefix ``docs/`` (objects in the ``docs/`` folder) to ``documents/``, you can set a condition block with ``KeyPrefixEquals`` set to ``docs/`` and in the Redirect set ``ReplaceKeyPrefixWith`` to ``/documents``. Not required if one of the siblings is present. Can be present only if ``ReplaceKeyWith`` is not provided. Replacement must be made for object keys containing special characters (such as carriage returns) when using XML requests. For more information, see [XML related object key constraints](https://docs.aws.amazon.com/AmazonS3/latest/userguide/object-keys.html#object-key-xml-related-constraints). |
| `replace_key_with` | String | No | The specific object key to use in the redirect request. For example, redirect request to ``error.html``. Not required if one of the siblings is present. Can be present only if ``ReplaceKeyPrefixWith`` is not provided. Replacement must be made for object keys containing special characters (such as carriage returns) when using XML requests. For more information, see [XML related object key constraints](https://docs.aws.amazon.com/AmazonS3/latest/userguide/object-keys.html#object-key-xml-related-constraints). |

### website_configuration.routing_rules.routing_rule_condition

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `http_error_code_returned_equals` | String | No | The HTTP error code when the redirect is applied. In the event of an error, if the error code equals this value, then the specified redirect is applied. Required when parent element ``Condition`` is specified and sibling ``KeyPrefixEquals`` is not specified. If both are specified, then both must be true for the redirect to be applied. |
| `key_prefix_equals` | String | No | The object key name prefix when the redirect is applied. For example, to redirect requests for ``ExamplePage.html``, the key prefix will be ``ExamplePage.html``. To redirect request for all pages with the prefix ``docs/``, the key prefix will be ``docs/``, which identifies all objects in the docs/ folder. Required when the parent element ``Condition`` is specified and sibling ``HttpErrorCodeReturnedEquals`` is not specified. If both conditions are specified, both must be true for the redirect to be applied. |

## Attribute Reference

### `arn`

- **Type:** Arn



### `domain_name`

- **Type:** String



### `dual_stack_domain_name`

- **Type:** String



### `regional_domain_name`

- **Type:** String



### `website_url`

- **Type:** String(uri)



