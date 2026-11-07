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

- **Type:** [Enum (FeatureSet)](#feature_set-featureset)
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

- **Type:** `aws.organizations.Organization.Arn`

The Amazon Resource Name (ARN) of an organization.

### `id`

- **Type:** String

The unique identifier (ID) of an organization.

### `management_account_arn`

- **Type:** `aws.Arn`

The Amazon Resource Name (ARN) of the account that is designated as the management account for the organization.

### `management_account_email`

- **Type:** String(pattern, len: 6..=64)

The email address that is associated with the AWS account that is designated as the management account for the organization.

### `management_account_id`

- **Type:** `aws.AccountId`

The unique identifier (ID) of the management account of an organization.

### `root_id`

- **Type:** String(pattern, len: ..=64)

The unique identifier (ID) for the root.

