---
title: "awscc.ecr.Repository"
---


CloudFormation Type: `AWS::ECR::Repository`

The ``AWS::ECR::Repository`` resource specifies an Amazon Elastic Container Registry (Amazon ECR) repository, where users can push and pull Docker images, Open Container Initiative (OCI) images, and OCI compatible artifacts. For more information, see [Amazon ECR private repositories](https://docs.aws.amazon.com/AmazonECR/latest/userguide/Repositories.html) in the *Amazon ECR User Guide*.

## Example

```crn
awscc.ecr.Repository {
  repository_name      = 'registry-publish-api'
  image_tag_mutability = aws.ecr.Repository.ImageTagMutability.immutable

  image_scanning_configuration = {
    scan_on_push = true
  }

  lifecycle_policy = {
    lifecycle_policy_text = '{"rules":[{"rulePriority":1,"description":"keep last 10 images","selection":{"tagStatus":"any","countType":"imageCountMoreThan","countNumber":10},"action":{"type":"expire"}}]}'
  }

  tags = {
    Environment = 'example'
  }
}
```

## Argument Reference

### `empty_on_delete`

- **Type:** `Bool`
- **Required:** No
- **Write-only:** Yes

If true, deleting the repository force deletes the contents of the repository. If false, the repository must be empty before attempting to delete it.

### `encryption_configuration`

- **Type:** [Struct(EncryptionConfiguration)](#encryption_configuration-1)
- **Required:** No
- **Create-only:** Yes

The encryption configuration for the repository. This determines how the contents of your repository are encrypted at rest.

### `image_scanning_configuration`

- **Type:** [Struct(ImageScanningConfiguration)](#image_scanning_configuration-1)
- **Required:** No

The ``imageScanningConfiguration`` parameter is being deprecated, in favor of specifying the image scanning configuration at the registry level. For more information, see ``PutRegistryScanningConfiguration``. The image scanning configuration for the repository. This determines whether images are scanned for known vulnerabilities after being pushed to the repository.

### `image_tag_mutability`

- **Type:** [ImageTagMutability](#image_tag_mutability-imagetagmutability)
- **Required:** No

The tag mutability setting for the repository. If this parameter is omitted, the default setting of ``MUTABLE`` will be used which will allow image tags to be overwritten. If ``IMMUTABLE`` is specified, all image tags within the repository will be immutable which will prevent them from being overwritten.

### `image_tag_mutability_exclusion_filters`

- **Type:** List\<[Struct(ImageTagMutabilityExclusionFilter)](#image_tag_mutability_exclusion_filters-1)\> (items: 1..=5, ordered)
- **Required:** No

A list of filters that specify which image tags are excluded from the repository's image tag mutability setting.

### `lifecycle_policy`

- **Type:** [Struct(LifecyclePolicy)](#lifecycle_policy-1)
- **Required:** No

Creates or updates a lifecycle policy. For information about lifecycle policy syntax, see [Lifecycle policy template](https://docs.aws.amazon.com/AmazonECR/latest/userguide/LifecyclePolicies.html).

### `repository_name`

- **Type:** `String (pattern: /^([a-z0-9]+((\.|_|__|-+)[a-z0-9]+)*(\/[a-z0-9]+((\.|_|__|-+)[a-z0-9]+)*)*)$/ (Full), len: 2..=256)`
- **Required:** No
- **Create-only:** Yes

The name to use for the repository. The repository name may be specified on its own (such as ``nginx-web-app``) or it can be prepended with a namespace to group the repository into a category (such as ``project-a/nginx-web-app``). If you don't specify a name, CFNlong generates a unique physical ID and uses that ID for the repository name. For more information, see [Name type](https://docs.aws.amazon.com/AWSCloudFormation/latest/UserGuide/aws-properties-name.html). The repository name must start with a letter and can only contain lowercase letters, numbers, hyphens, underscores, and forward slashes. If you specify a name, you cannot perform updates that require replacement of this resource. You can perform updates that require no or some interruption. If you must replace the resource, specify a new name.

### `repository_policy_text`

- **Type:** [Struct(PolicyDocument)](#repository_policy_text-1)
- **Required:** No

The JSON repository policy text to apply to the repository. For more information, see [Amazon ECR repository policies](https://docs.aws.amazon.com/AmazonECR/latest/userguide/repository-policy-examples.html) in the *Amazon Elastic Container Registry User Guide*.

### `tags`

- **Type:** Map\<`String (len: 1..=127)`, `String (len: 1..=255)`\> (entries: unbounded..=50)
- **Required:** No

An array of key-value pairs to apply to this resource.

## Enum Values

### encryption_configuration.encryption_type (EncryptionType)

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

### image_tag_mutability_exclusion_filters.image_tag_mutability_exclusion_filter_type (ImageTagMutabilityExclusionFilterType)

| Value | DSL Identifier |
|-------|----------------|
| `WILDCARD` | `aws.ecr.Repository.ImageTagMutabilityExclusionFilter.ImageTagMutabilityExclusionFilterType.wildcard` |

Shorthand formats: `wildcard` or `ImageTagMutabilityExclusionFilterType.wildcard`

### repository_policy_text.version (Version)

| Value | DSL Identifier |
|-------|----------------|
| `2012-10-17` | `aws.iam.PolicyDocument.Version.2012_10_17` |
| `2008-10-17` | `aws.iam.PolicyDocument.Version.2008_10_17` |

Use the fully qualified DSL identifiers above; bare numeric values parse as integers.

### statement.condition.\<key\> (ConditionOperator)

| Value | DSL Identifier |
|-------|----------------|
| `string_equals` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.string_equals` |
| `string_equals_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.string_equals_if_exists` |
| `for_all_values_string_equals` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_all_values_string_equals` |
| `for_all_values_string_equals_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_all_values_string_equals_if_exists` |
| `for_any_value_string_equals` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_any_value_string_equals` |
| `for_any_value_string_equals_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_any_value_string_equals_if_exists` |
| `string_not_equals` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.string_not_equals` |
| `string_not_equals_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.string_not_equals_if_exists` |
| `for_all_values_string_not_equals` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_all_values_string_not_equals` |
| `for_all_values_string_not_equals_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_all_values_string_not_equals_if_exists` |
| `for_any_value_string_not_equals` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_any_value_string_not_equals` |
| `for_any_value_string_not_equals_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_any_value_string_not_equals_if_exists` |
| `string_equals_ignore_case` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.string_equals_ignore_case` |
| `string_equals_ignore_case_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.string_equals_ignore_case_if_exists` |
| `for_all_values_string_equals_ignore_case` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_all_values_string_equals_ignore_case` |
| `for_all_values_string_equals_ignore_case_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_all_values_string_equals_ignore_case_if_exists` |
| `for_any_value_string_equals_ignore_case` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_any_value_string_equals_ignore_case` |
| `for_any_value_string_equals_ignore_case_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_any_value_string_equals_ignore_case_if_exists` |
| `string_not_equals_ignore_case` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.string_not_equals_ignore_case` |
| `string_not_equals_ignore_case_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.string_not_equals_ignore_case_if_exists` |
| `for_all_values_string_not_equals_ignore_case` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_all_values_string_not_equals_ignore_case` |
| `for_all_values_string_not_equals_ignore_case_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_all_values_string_not_equals_ignore_case_if_exists` |
| `for_any_value_string_not_equals_ignore_case` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_any_value_string_not_equals_ignore_case` |
| `for_any_value_string_not_equals_ignore_case_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_any_value_string_not_equals_ignore_case_if_exists` |
| `string_like` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.string_like` |
| `string_like_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.string_like_if_exists` |
| `for_all_values_string_like` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_all_values_string_like` |
| `for_all_values_string_like_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_all_values_string_like_if_exists` |
| `for_any_value_string_like` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_any_value_string_like` |
| `for_any_value_string_like_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_any_value_string_like_if_exists` |
| `string_not_like` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.string_not_like` |
| `string_not_like_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.string_not_like_if_exists` |
| `for_all_values_string_not_like` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_all_values_string_not_like` |
| `for_all_values_string_not_like_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_all_values_string_not_like_if_exists` |
| `for_any_value_string_not_like` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_any_value_string_not_like` |
| `for_any_value_string_not_like_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_any_value_string_not_like_if_exists` |
| `numeric_equals` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.numeric_equals` |
| `numeric_equals_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.numeric_equals_if_exists` |
| `for_all_values_numeric_equals` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_all_values_numeric_equals` |
| `for_all_values_numeric_equals_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_all_values_numeric_equals_if_exists` |
| `for_any_value_numeric_equals` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_any_value_numeric_equals` |
| `for_any_value_numeric_equals_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_any_value_numeric_equals_if_exists` |
| `numeric_not_equals` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.numeric_not_equals` |
| `numeric_not_equals_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.numeric_not_equals_if_exists` |
| `for_all_values_numeric_not_equals` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_all_values_numeric_not_equals` |
| `for_all_values_numeric_not_equals_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_all_values_numeric_not_equals_if_exists` |
| `for_any_value_numeric_not_equals` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_any_value_numeric_not_equals` |
| `for_any_value_numeric_not_equals_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_any_value_numeric_not_equals_if_exists` |
| `numeric_less_than` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.numeric_less_than` |
| `numeric_less_than_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.numeric_less_than_if_exists` |
| `for_all_values_numeric_less_than` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_all_values_numeric_less_than` |
| `for_all_values_numeric_less_than_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_all_values_numeric_less_than_if_exists` |
| `for_any_value_numeric_less_than` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_any_value_numeric_less_than` |
| `for_any_value_numeric_less_than_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_any_value_numeric_less_than_if_exists` |
| `numeric_less_than_equals` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.numeric_less_than_equals` |
| `numeric_less_than_equals_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.numeric_less_than_equals_if_exists` |
| `for_all_values_numeric_less_than_equals` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_all_values_numeric_less_than_equals` |
| `for_all_values_numeric_less_than_equals_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_all_values_numeric_less_than_equals_if_exists` |
| `for_any_value_numeric_less_than_equals` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_any_value_numeric_less_than_equals` |
| `for_any_value_numeric_less_than_equals_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_any_value_numeric_less_than_equals_if_exists` |
| `numeric_greater_than` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.numeric_greater_than` |
| `numeric_greater_than_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.numeric_greater_than_if_exists` |
| `for_all_values_numeric_greater_than` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_all_values_numeric_greater_than` |
| `for_all_values_numeric_greater_than_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_all_values_numeric_greater_than_if_exists` |
| `for_any_value_numeric_greater_than` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_any_value_numeric_greater_than` |
| `for_any_value_numeric_greater_than_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_any_value_numeric_greater_than_if_exists` |
| `numeric_greater_than_equals` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.numeric_greater_than_equals` |
| `numeric_greater_than_equals_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.numeric_greater_than_equals_if_exists` |
| `for_all_values_numeric_greater_than_equals` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_all_values_numeric_greater_than_equals` |
| `for_all_values_numeric_greater_than_equals_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_all_values_numeric_greater_than_equals_if_exists` |
| `for_any_value_numeric_greater_than_equals` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_any_value_numeric_greater_than_equals` |
| `for_any_value_numeric_greater_than_equals_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_any_value_numeric_greater_than_equals_if_exists` |
| `date_equals` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.date_equals` |
| `date_equals_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.date_equals_if_exists` |
| `for_all_values_date_equals` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_all_values_date_equals` |
| `for_all_values_date_equals_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_all_values_date_equals_if_exists` |
| `for_any_value_date_equals` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_any_value_date_equals` |
| `for_any_value_date_equals_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_any_value_date_equals_if_exists` |
| `date_not_equals` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.date_not_equals` |
| `date_not_equals_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.date_not_equals_if_exists` |
| `for_all_values_date_not_equals` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_all_values_date_not_equals` |
| `for_all_values_date_not_equals_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_all_values_date_not_equals_if_exists` |
| `for_any_value_date_not_equals` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_any_value_date_not_equals` |
| `for_any_value_date_not_equals_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_any_value_date_not_equals_if_exists` |
| `date_less_than` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.date_less_than` |
| `date_less_than_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.date_less_than_if_exists` |
| `for_all_values_date_less_than` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_all_values_date_less_than` |
| `for_all_values_date_less_than_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_all_values_date_less_than_if_exists` |
| `for_any_value_date_less_than` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_any_value_date_less_than` |
| `for_any_value_date_less_than_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_any_value_date_less_than_if_exists` |
| `date_less_than_equals` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.date_less_than_equals` |
| `date_less_than_equals_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.date_less_than_equals_if_exists` |
| `for_all_values_date_less_than_equals` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_all_values_date_less_than_equals` |
| `for_all_values_date_less_than_equals_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_all_values_date_less_than_equals_if_exists` |
| `for_any_value_date_less_than_equals` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_any_value_date_less_than_equals` |
| `for_any_value_date_less_than_equals_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_any_value_date_less_than_equals_if_exists` |
| `date_greater_than` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.date_greater_than` |
| `date_greater_than_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.date_greater_than_if_exists` |
| `for_all_values_date_greater_than` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_all_values_date_greater_than` |
| `for_all_values_date_greater_than_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_all_values_date_greater_than_if_exists` |
| `for_any_value_date_greater_than` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_any_value_date_greater_than` |
| `for_any_value_date_greater_than_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_any_value_date_greater_than_if_exists` |
| `date_greater_than_equals` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.date_greater_than_equals` |
| `date_greater_than_equals_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.date_greater_than_equals_if_exists` |
| `for_all_values_date_greater_than_equals` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_all_values_date_greater_than_equals` |
| `for_all_values_date_greater_than_equals_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_all_values_date_greater_than_equals_if_exists` |
| `for_any_value_date_greater_than_equals` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_any_value_date_greater_than_equals` |
| `for_any_value_date_greater_than_equals_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_any_value_date_greater_than_equals_if_exists` |
| `bool` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.bool` |
| `bool_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.bool_if_exists` |
| `for_all_values_bool` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_all_values_bool` |
| `for_all_values_bool_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_all_values_bool_if_exists` |
| `for_any_value_bool` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_any_value_bool` |
| `for_any_value_bool_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_any_value_bool_if_exists` |
| `binary_equals` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.binary_equals` |
| `binary_equals_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.binary_equals_if_exists` |
| `for_all_values_binary_equals` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_all_values_binary_equals` |
| `for_all_values_binary_equals_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_all_values_binary_equals_if_exists` |
| `for_any_value_binary_equals` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_any_value_binary_equals` |
| `for_any_value_binary_equals_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_any_value_binary_equals_if_exists` |
| `ip_address` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.ip_address` |
| `ip_address_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.ip_address_if_exists` |
| `for_all_values_ip_address` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_all_values_ip_address` |
| `for_all_values_ip_address_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_all_values_ip_address_if_exists` |
| `for_any_value_ip_address` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_any_value_ip_address` |
| `for_any_value_ip_address_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_any_value_ip_address_if_exists` |
| `not_ip_address` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.not_ip_address` |
| `not_ip_address_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.not_ip_address_if_exists` |
| `for_all_values_not_ip_address` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_all_values_not_ip_address` |
| `for_all_values_not_ip_address_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_all_values_not_ip_address_if_exists` |
| `for_any_value_not_ip_address` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_any_value_not_ip_address` |
| `for_any_value_not_ip_address_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_any_value_not_ip_address_if_exists` |
| `arn_equals` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.arn_equals` |
| `arn_equals_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.arn_equals_if_exists` |
| `for_all_values_arn_equals` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_all_values_arn_equals` |
| `for_all_values_arn_equals_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_all_values_arn_equals_if_exists` |
| `for_any_value_arn_equals` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_any_value_arn_equals` |
| `for_any_value_arn_equals_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_any_value_arn_equals_if_exists` |
| `arn_not_equals` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.arn_not_equals` |
| `arn_not_equals_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.arn_not_equals_if_exists` |
| `for_all_values_arn_not_equals` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_all_values_arn_not_equals` |
| `for_all_values_arn_not_equals_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_all_values_arn_not_equals_if_exists` |
| `for_any_value_arn_not_equals` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_any_value_arn_not_equals` |
| `for_any_value_arn_not_equals_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_any_value_arn_not_equals_if_exists` |
| `arn_like` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.arn_like` |
| `arn_like_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.arn_like_if_exists` |
| `for_all_values_arn_like` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_all_values_arn_like` |
| `for_all_values_arn_like_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_all_values_arn_like_if_exists` |
| `for_any_value_arn_like` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_any_value_arn_like` |
| `for_any_value_arn_like_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_any_value_arn_like_if_exists` |
| `arn_not_like` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.arn_not_like` |
| `arn_not_like_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.arn_not_like_if_exists` |
| `for_all_values_arn_not_like` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_all_values_arn_not_like` |
| `for_all_values_arn_not_like_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_all_values_arn_not_like_if_exists` |
| `for_any_value_arn_not_like` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_any_value_arn_not_like` |
| `for_any_value_arn_not_like_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_any_value_arn_not_like_if_exists` |
| `null` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.null` |
| `null_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.null_if_exists` |
| `for_all_values_null` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_all_values_null` |
| `for_all_values_null_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_all_values_null_if_exists` |
| `for_any_value_null` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_any_value_null` |
| `for_any_value_null_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_any_value_null_if_exists` |

Shorthand formats: `string_equals` or `ConditionOperator.string_equals`

### statement.effect (Effect)

| Value | DSL Identifier |
|-------|----------------|
| `Allow` | `aws.iam.PolicyDocument.Statement.Effect.allow` |
| `Deny` | `aws.iam.PolicyDocument.Statement.Effect.deny` |

Shorthand formats: `allow` or `Effect.allow`

## Struct Definitions

### encryption_configuration

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `encryption_type` | [EncryptionType](#encryption_configurationencryption_type-encryptiontype) | Yes | The encryption type to use. If you use the ``KMS`` encryption type, the contents of the repository will be encrypted using server-side encryption with KMSlong key stored in KMS. When you use KMS to encrypt your data, you can either use the default AWS managed KMS key for Amazon ECR, or specify your own KMS key, which you already created. If you use the ``KMS_DSSE`` encryption type, the contents of the repository will be encrypted with two layers of encryption using server-side encryption with the KMS Management Service key stored in KMS. Similar to the ``KMS`` encryption type, you can either use the default AWS managed KMS key for Amazon ECR, or specify your own KMS key, which you've already created. If you use the ``AES256`` encryption type, Amazon ECR uses server-side encryption with Amazon S3-managed encryption keys which encrypts the images in the repository using an AES256 encryption algorithm. For more information, see [Amazon ECR encryption at rest](https://docs.aws.amazon.com/AmazonECR/latest/userguide/encryption-at-rest.html) in the *Amazon Elastic Container Registry User Guide*. |
| `kms_key` | `aws.kms.Key.Reference (pattern: /^((arn:(?:aws\|aws-cn\|aws-us-gov\|aws-iso\|aws-iso-b\|aws-iso-e\|aws-iso-f\|aws-eusc):kms:[a-z]{2,4}(-[a-z0-9]+)+-[0-9]+:[0-9]{12}:(key/([0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12}\|mrk-[0-9a-fA-F]{32})\|alias/[a-zA-Z0-9:/_-]+))\|(alias/[a-zA-Z0-9:/_-]+)\|([0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12})\|(mrk-[0-9a-fA-F]{32}))$/ (Full), len: 1..=2048)` | No | If you use the ``KMS`` encryption type, specify the KMS key to use for encryption. The alias, key ID, or full ARN of the KMS key can be specified. The key must exist in the same Region as the repository. If no key is specified, the default AWS managed KMS key for Amazon ECR will be used. |

### image_scanning_configuration

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `scan_on_push` | `Bool` | No | The setting that determines whether images are scanned after being pushed to a repository. If set to ``true``, images will be scanned after being pushed. If this parameter is not specified, it will default to ``false`` and images will not be scanned unless a scan is manually started. |

### image_tag_mutability_exclusion_filters

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `image_tag_mutability_exclusion_filter_type` | [ImageTagMutabilityExclusionFilterType](#image_tag_mutability_exclusion_filtersimage_tag_mutability_exclusion_filter_type-imagetagmutabilityexclusionfiltertype) | Yes |  |
| `image_tag_mutability_exclusion_filter_value` | `String (pattern: /^[0-9a-zA-Z._*-]{1,128}\z/ (Full), len: 1..=128)` | Yes |  |

### lifecycle_policy

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `lifecycle_policy_text` | `String (len: 100..=30720)` | No | The JSON repository policy text to apply to the repository. |
| `registry_id` | `String (pattern: /^[0-9]{12}$/ (Full), len: 12..=12)` | No | The AWS account ID associated with the registry that contains the repository. If you do? not specify a registry, the default registry is assumed. |

### principal

Used by: `union_of_principal_or_string.member_0`

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `service` | [Union(union\_of\_string\_or\_list\_of\_string)](#union_of_string_or_list_of_string) | No |  |
| `aws` | [Union(union\_of\_string\_or\_list\_of\_string)](#union_of_string_or_list_of_string) | No |  |
| `federated` | [Union(union\_of\_string\_or\_list\_of\_string)](#union_of_string_or_list_of_string) | No |  |
| `canonical_user` | [Union(union\_of\_string\_or\_list\_of\_string)](#union_of_string_or_list_of_string) | No |  |

**Field groups:**

- At least one of: `service`, `aws`, `federated`, `canonical_user`.

### repository_policy_text

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `version` | [Version](#repository_policy_textversion-version) | No |  |
| `id` | `String` | No |  |
| `statement` | [Union(repository\_policy\_text.statement)](#repository_policy_textstatement) | Yes |  |

### statement

Used by: `repository_policy_text.statement.member_0`, `repository_policy_text.statement.member_1`

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `sid` | `String` | No |  |
| `effect` | [Effect](#statementeffect-effect) | Yes |  |
| `action` | [Union(union\_of\_string\_or\_list\_of\_string)](#union_of_string_or_list_of_string) | No |  |
| `not_action` | [Union(union\_of\_string\_or\_list\_of\_string)](#union_of_string_or_list_of_string) | No |  |
| `resource` | [Union(union\_of\_string\_or\_list\_of\_string)](#union_of_string_or_list_of_string) | No |  |
| `not_resource` | [Union(union\_of\_string\_or\_list\_of\_string)](#union_of_string_or_list_of_string) | No |  |
| `principal` | [Union(union\_of\_principal\_or\_string)](#union_of_principal_or_string) | No |  |
| `not_principal` | [Union(union\_of\_principal\_or\_string)](#union_of_principal_or_string) | No |  |
| `condition` | Map\<[ConditionOperator](#statementconditionkey-conditionoperator), Map\<`String`, [Union(statement.condition.\<value\>.\<value\>)](#statementconditionvaluevalue)\>\> | No |  |

**Field groups:**

- Exactly one of: `action`, `not_action`.
- At most one of: `resource`, `not_resource`.
- At most one of: `principal`, `not_principal`.

## Union Definitions

### repository_policy_text.statement

| Member | Type |
|--------|------|
| `member_0` | List\<[Struct(Statement)](#statement)\> (items: 1..unbounded, ordered) |
| `member_1` | [Struct(Statement)](#statement) |

### statement.condition.\<value\>.\<value\>

| Member | Type |
|--------|------|
| `member_0` | [Union(union\_of\_string\_or\_list\_of\_string)](#union_of_string_or_list_of_string) |
| `member_1` | `Int` |
| `member_2` | `Float` |
| `member_3` | `Bool` |
| `member_4` | List\<[Union(union\_of\_string\_or\_int\_or\_float\_or\_bool)](#union_of_string_or_int_or_float_or_bool)\> (items: 1..unbounded, unordered) |

### union_of_principal_or_string

Used by: `statement.not_principal`, `statement.principal`

| Member | Type |
|--------|------|
| `member_0` | [Struct(Principal)](#principal) |
| `member_1` | `String (pattern: /^\*\z/ (Full))` |

### union_of_string_or_int_or_float_or_bool

Used by: `statement.condition.<value>.<value>.member_4`

| Member | Type |
|--------|------|
| `member_0` | `String` |
| `member_1` | `Int` |
| `member_2` | `Float` |
| `member_3` | `Bool` |

### union_of_string_or_list_of_string

Used by: `principal.aws`, `principal.canonical_user`, `principal.federated`, `principal.service`, `statement.action`, `statement.not_action`, `statement.not_resource`, `statement.resource`, `statement.condition.<value>.<value>.member_0`

| Member | Type |
|--------|------|
| `member_0` | `String` |
| `member_1` | List\<`String`\> (items: 1..unbounded, unordered) |

## Attribute Reference

### `arn`

- **Type:** `aws.Arn (pattern: /^arn:(?:aws|aws-cn|aws-us-gov|aws-iso|aws-iso-b|aws-iso-e|aws-iso-f|aws-eusc):[^:]+:.*$/ (Full), custom validator)`



### `repository_uri`

- **Type:** `String`



## AWS API narrowings

These fields are narrower than or equal to CFN. Input rules combine documented AWS domains, recorded live observations, and Carina canonical spellings for stable state comparison. A canonical DSL spelling does not claim AWS rejects other spellings. Attaching a type name adds no structural constraints. Field names below use CFN spelling.

| CFN field | Relation to CFN | Evidence URL | Verbatim quote / exact observation | Checked / observed |
| --- | --- | --- | --- | --- |
| `/definitions/ImageTagMutabilityExclusionFilter/properties/ImageTagMutabilityExclusionFilterValue` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/AmazonECR/latest/APIReference/API_ImageTagMutabilityExclusionFilter.html#ECR-Type-ImageTagMutabilityExclusionFilter-filter) | ` The filter value used to match image tags for exclusion from mutability settings. `<br>` Length Constraints: Minimum length of 1. Maximum length of 128. `<br>` Pattern: ^[0-9a-zA-Z._*-]{1,128}$ ` | 2026-11-07 |
| `/properties/RepositoryPolicyText` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/IAM/latest/UserGuide/reference_policies_grammar.html) | ` <version_block> = "Version" : ("2008-10-17" \| "2012-10-17") `<br>` <effect_block> = "Effect" : ("Allow" \| "Deny") ` | 2026-11-07 |
| `/properties/RepositoryPolicyText` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/IAM/latest/UserGuide/reference_policies_elements_statement.html) | ` The Statement element can contain a single statement or an array of individual statements. ` | 2026-11-07 |
| `/properties/RepositoryPolicyText` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/IAM/latest/UserGuide/reference_policies_elements_condition.html) | ` You can create conditions that compare strings, dates, numbers, and more using typical Boolean comparisons such as equals, greater than, and less than. ` | 2026-11-07 |

