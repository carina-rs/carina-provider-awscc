---
title: "awscc.sso.Instance"
---


CloudFormation Type: `AWS::SSO::Instance`

Resource Type definition for Identity Center (SSO) Instance

## Example

```crn
awscc.sso.Instance {
  # Creates an account instance. Apply from a standalone or member account
  # that has no IAM Identity Center instance. AWS rejects this in the
  # organization's management account and where organization policy disallows
  # account instances.
  name = 'carina-example-identity-center'

  tags = {
    Environment = 'example'
  }
}
```

## Argument Reference

### `name`

- **Type:** `String (pattern: /^[\w+=,.@-]+$/ (Full), len: 1..=32)`
- **Required:** No

The name you want to assign to this Identity Center (SSO) Instance

### `tags`

- **Type:** Map\<`String (pattern: /[\w+=,.@-]+/ (Full), len: 1..=128)`, `String (pattern: /[\w+=,.@-]+/ (Full), len: unbounded..=256)`\> (entries: unbounded..=75)
- **Required:** No

## Enum Values

### status (Status)

| Value | DSL Identifier |
|-------|----------------|
| `CREATE_IN_PROGRESS` | `aws.sso.Instance.Status.create_in_progress` |
| `DELETE_IN_PROGRESS` | `aws.sso.Instance.Status.delete_in_progress` |
| `ACTIVE` | `aws.sso.Instance.Status.active` |

Shorthand formats: `create_in_progress` or `Status.create_in_progress`

## Attribute Reference

### `identity_store_id`

- **Type:** `aws.identitystore.Store.Id (identity constraints: pattern: /^(d-[0-9a-fA-F]{10}|[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12})$/ (Full), member constraints: pattern: /^(d-[0-9a-f]{10}|[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12})\z/ (Full), len: 1..=64)`

The ID of the identity store associated with the created Identity Center (SSO) Instance

### `instance_arn`

- **Type:** `aws.sso.Instance.Arn (identity constraints: pattern: /^arn:(?:aws|aws-cn|aws-us-gov|aws-iso|aws-iso-b|aws-iso-e|aws-iso-f|aws-eusc):sso:::instance/.+$/ (Full), member constraints: pattern: /^arn:aws(-[a-z]{1,5}){0,3}:sso:::instance/(sso)?ins-[a-zA-Z0-9-.]{16}\z/ (Full), len: 10..=1224)`

The SSO Instance ARN that is returned upon creation of the Identity Center (SSO) Instance

### `owner_account_id`

- **Type:** `aws.AccountId (identity constraints: pattern: /^[0-9]{12}$/ (Full), len: 12..=12, member constraints: pattern: /^\d{12}?$/ (Full), len: 12..=12)`

The AWS accountId of the owner of the Identity Center (SSO) Instance

### `status`

- **Type:** [Status](#status-status)

The status of the Identity Center (SSO) Instance, create_in_progress/delete_in_progress/active

## AWS API narrowings

These fields are narrower than or equal to CFN. Input rules combine documented AWS domains, recorded live observations, and Carina canonical spellings for stable state comparison. A canonical DSL spelling does not claim AWS rejects other spellings. Attaching a type name adds no structural constraints. Field names below use CFN spelling.

| CFN field | Relation to CFN | Evidence URL | Verbatim quote / exact observation | Checked / observed |
| --- | --- | --- | --- | --- |
| `/properties/IdentityStoreId` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/singlesignon/latest/IdentityStoreAPIReference/API_CreateGroup.html#singlesignon-CreateGroup-response-IdentityStoreId) | ` The globally unique identifier for the identity store. `<br>` Length Constraints: Minimum length of 1. Maximum length of 93. `<br>` Pattern: (arn:aws[a-z-]*:identitystore::\d{12}:identitystore/)?(d-[0-9a-f]{10}\|[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}) ` | 2026-11-07 |
| `/properties/InstanceArn` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/singlesignon/latest/APIReference/API_CreateAccountAssignment.html#singlesignon-CreateAccountAssignment-request-InstanceArn) | ` The ARN of the IAM Identity Center instance under which the operation will be executed. For more information about ARNs, see Amazon Resource Names (ARNs) and AWS Service Namespaces in the AWS General Reference. `<br>` Length Constraints: Minimum length of 10. Maximum length of 1224. `<br>` Pattern: arn:aws(-[a-z]{1,5}){0,3}:sso:::instance/(sso)?ins-[a-zA-Z0-9-.]{16} ` | 2026-11-07 |

