---
title: "awscc.organizations.Organization"
---


CloudFormation Type: `AWS::Organizations::Organization`

Resource schema for AWS::Organizations::Organization

## Example

```crn
awscc.organizations.Organization {
  # The account must not already belong to an organization. Destroying this
  # resource deletes the organization, which requires that it have no member
  # accounts.
  feature_set = aws.organizations.Organization.FeatureSet.all
}
```

## Argument Reference

### `feature_set`

- **Type:** [FeatureSet](#feature_set-featureset)
- **Required:** No
- **Default:** `"ALL"`

Specifies the feature set supported by the new organization. Each feature set supports different levels of functionality.

## Enum Values

### feature_set (FeatureSet)

| Value | DSL Identifier |
|-------|----------------|
| `ALL` | `aws.organizations.Organization.FeatureSet.all` |
| `CONSOLIDATED_BILLING` | `aws.organizations.Organization.FeatureSet.consolidated_billing` |

Shorthand formats: `all` or `FeatureSet.all`

## Attribute Reference

### `arn`

- **Type:** `aws.organizations.Organization.Arn (identity constraints: pattern: /^arn:(?:aws|aws-cn|aws-us-gov|aws-iso|aws-iso-b|aws-iso-e|aws-iso-f|aws-eusc):organizations:.*$/ (Full), member constraints: pattern: /^arn:aws.*:organizations::\d{12}:organization\/o-[a-z0-9]{10,32}/ (Full), custom validator)`

The Amazon Resource Name (ARN) of an organization.

### `id`

- **Type:** `String (pattern: /^o-[a-z0-9]{10,32}$/ (Full))`

The unique identifier (ID) of an organization.

### `management_account_arn`

- **Type:** `aws.Arn (identity constraints: pattern: /^arn:(?:aws|aws-cn|aws-us-gov|aws-iso|aws-iso-b|aws-iso-e|aws-iso-f|aws-eusc):[^:]+:.*$/ (Full), member constraints: pattern: /^arn:aws.*:organizations::\d{12}:account\/o-[a-z0-9]{10,32}\/\d{12}/ (Full), custom validator)`

The Amazon Resource Name (ARN) of the account that is designated as the management account for the organization.

### `management_account_email`

- **Type:** `String (pattern: /[^\s@]+@[^\s@]+\.[^\s@]+/ (Full), len: 6..=64)`

The email address that is associated with the AWS account that is designated as the management account for the organization.

### `management_account_id`

- **Type:** `aws.AccountId (identity constraints: pattern: /^[0-9]{12}$/ (Full), len: 12..=12, member constraints: pattern: /^\d{12}$/ (Full))`

The unique identifier (ID) of the management account of an organization.

### `root_id`

- **Type:** `String (pattern: /^r-[0-9a-z]{4,32}$/ (Full), len: unbounded..=64)`

The unique identifier (ID) for the root.

