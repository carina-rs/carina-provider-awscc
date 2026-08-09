---
title: "awscc.ecr.Repository"
description: "AWSCC ECR Repository resource reference"
---


CloudFormation Type: `AWS::ECR::Repository`

The ``AWS::ECR::Repository`` resource specifies an Amazon Elastic Container Registry (Amazon ECR) repository, where users can push and pull Docker images, Open Container Initiative (OCI) images, and OCI compatible artifacts. For more information, see [Amazon ECR private repositories](https://docs.aws.amazon.com/AmazonECR/latest/userguide/Repositories.html) in the *Amazon ECR User Guide*.

## Argument Reference

### `empty_on_delete`

- **Type:** Bool
- **Required:** No
- **Write-only:** Yes

If true, deleting the repository force deletes the contents of the repository. If false, the repository must be empty before attempting to delete it.

### `encryption_configuration`

- **Type:** [Struct(EncryptionConfiguration)](#encryptionconfiguration)
- **Required:** No
- **Create-only:** Yes

The encryption configuration for the repository. This determines how the contents of your repository are encrypted at rest.

### `image_scanning_configuration`

- **Type:** [Struct(ImageScanningConfiguration)](#imagescanningconfiguration)
- **Required:** No

The ``imageScanningConfiguration`` parameter is being deprecated, in favor of specifying the image scanning configuration at the registry level. For more information, see ``PutRegistryScanningConfiguration``. The image scanning configuration for the repository. This determines whether images are scanned for known vulnerabilities after being pushed to the repository.

### `image_tag_mutability`

- **Type:** [Enum (ImageTagMutability)](#image_tag_mutability-imagetagmutability)
- **Required:** No

The tag mutability setting for the repository. If this parameter is omitted, the default setting of ``MUTABLE`` will be used which will allow image tags to be overwritten. If ``IMMUTABLE`` is specified, all image tags within the repository will be immutable which will prevent them from being overwritten.

### `image_tag_mutability_exclusion_filters`

- **Type:** [List\<ImageTagMutabilityExclusionFilter\>](#imagetagmutabilityexclusionfilter) (items: 1..=5)
- **Required:** No

A list of filters that specify which image tags are excluded from the repository's image tag mutability setting.

### `lifecycle_policy`

- **Type:** [Struct(LifecyclePolicy)](#lifecyclepolicy)
- **Required:** No

Creates or updates a lifecycle policy. For information about lifecycle policy syntax, see [Lifecycle policy template](https://docs.aws.amazon.com/AmazonECR/latest/userguide/LifecyclePolicies.html).

### `repository_name`

- **Type:** String(pattern, len: 2..=256)
- **Required:** No
- **Create-only:** Yes

The name to use for the repository. The repository name may be specified on its own (such as ``nginx-web-app``) or it can be prepended with a namespace to group the repository into a category (such as ``project-a/nginx-web-app``). If you don't specify a name, CFNlong generates a unique physical ID and uses that ID for the repository name. For more information, see [Name type](https://docs.aws.amazon.com/AWSCloudFormation/latest/UserGuide/aws-properties-name.html). The repository name must start with a letter and can only contain lowercase letters, numbers, hyphens, underscores, and forward slashes. If you specify a name, you cannot perform updates that require replacement of this resource. You can perform updates that require no or some interruption. If you must replace the resource, specify a new name.

### `repository_policy_text`

- **Type:** `Map<String, String>`
- **Required:** No

The JSON repository policy text to apply to the repository. For more information, see [Amazon ECR repository policies](https://docs.aws.amazon.com/AmazonECR/latest/userguide/repository-policy-examples.html) in the *Amazon Elastic Container Registry User Guide*.

### `tags`

- **Type:** `Map<String, String>`
- **Required:** No

An array of key-value pairs to apply to this resource.

## Enum Values

### encryption_type (EncryptionType)

| Value | DSL Identifier |
|-------|----------------|
| `AES256` | `aws.ecr.Repository.EncryptionConfiguration.EncryptionType.aes256` |
| `KMS` | `aws.ecr.Repository.EncryptionConfiguration.EncryptionType.kms` |
| `KMS_DSSE` | `aws.ecr.Repository.EncryptionConfiguration.EncryptionType.kms_dsse` |

Shorthand formats: `aes256` or `EncryptionType.aes256`

### image_tag_mutability (ImageTagMutability)

| Value | DSL Identifier |
|-------|----------------|
| `MUTABLE` | `aws.ecr.Repository.ImageTagMutability.mutable` |
| `IMMUTABLE` | `aws.ecr.Repository.ImageTagMutability.immutable` |
| `MUTABLE_WITH_EXCLUSION` | `aws.ecr.Repository.ImageTagMutability.mutable_with_exclusion` |
| `IMMUTABLE_WITH_EXCLUSION` | `aws.ecr.Repository.ImageTagMutability.immutable_with_exclusion` |

Shorthand formats: `mutable` or `ImageTagMutability.mutable`

### image_tag_mutability_exclusion_filter_type (ImageTagMutabilityExclusionFilterType)

| Value | DSL Identifier |
|-------|----------------|
| `WILDCARD` | `aws.ecr.Repository.ImageTagMutabilityExclusionFilter.ImageTagMutabilityExclusionFilterType.wildcard` |

Shorthand formats: `wildcard` or `ImageTagMutabilityExclusionFilterType.wildcard`

## Struct Definitions

### EncryptionConfiguration

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `encryption_type` | [Enum (EncryptionType)](#encryption_type-encryptiontype) | Yes | The encryption type to use. If you use the ``KMS`` encryption type, the contents of the repository will be encrypted using server-side encryption with KMSlong key stored in KMS. When you use KMS to encrypt your data, you can either use the default AWS managed KMS key for Amazon ECR, or specify your own KMS key, which you already created. If you use the ``KMS_DSSE`` encryption type, the contents of the repository will be encrypted with two layers of encryption using server-side encryption with the KMS Management Service key stored in KMS. Similar to the ``KMS`` encryption type, you can either use the default AWS managed KMS key for Amazon ECR, or specify your own KMS key, which you've already created. If you use the ``AES256`` encryption type, Amazon ECR uses server-side encryption with Amazon S3-managed encryption keys which encrypts the images in the repository using an AES256 encryption algorithm. For more information, see [Amazon ECR encryption at rest](https://docs.aws.amazon.com/AmazonECR/latest/userguide/encryption-at-rest.html) in the *Amazon Elastic Container Registry User Guide*. |
| `kms_key` | String | No | If you use the ``KMS`` encryption type, specify the KMS key to use for encryption. The alias, key ID, or full ARN of the KMS key can be specified. The key must exist in the same Region as the repository. If no key is specified, the default AWS managed KMS key for Amazon ECR will be used. |

### ImageScanningConfiguration

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `scan_on_push` | Bool | No | The setting that determines whether images are scanned after being pushed to a repository. If set to ``true``, images will be scanned after being pushed. If this parameter is not specified, it will default to ``false`` and images will not be scanned unless a scan is manually started. |

### ImageTagMutabilityExclusionFilter

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `image_tag_mutability_exclusion_filter_type` | [Enum (ImageTagMutabilityExclusionFilterType)](#image_tag_mutability_exclusion_filter_type-imagetagmutabilityexclusionfiltertype) | Yes |  |
| `image_tag_mutability_exclusion_filter_value` | String | Yes |  |

### LifecyclePolicy

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `lifecycle_policy_text` | String | No | The JSON repository policy text to apply to the repository. |
| `registry_id` | String | No | The AWS account ID associated with the registry that contains the repository. If you do? not specify a registry, the default registry is assumed. |

## Attribute Reference

### `arn`

- **Type:** Arn



### `repository_uri`

- **Type:** String



