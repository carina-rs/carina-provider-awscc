---
title: "awscc.identitystore.Group"
---


CloudFormation Type: `AWS::IdentityStore::Group`

Resource Type definition for AWS::IdentityStore::Group

## Example

```crn
awscc.identitystore.Group {
  # Replace this placeholder with your Identity Store ID.
  identity_store_id = 'd-1234567890'
  display_name      = 'ExampleDevelopers'
  description       = 'Developers for the example workload'
}
```

## Argument Reference

### `description`

- **Type:** String(pattern, len: 1..=1024)
- **Required:** No

A string containing the description of the group.

### `display_name`

- **Type:** aws.identitystore.Group.DisplayName — identity: String(pattern, len: 1..=1024); member: String(pattern, len: 1..=1024)
- **Required:** Yes

A string containing the name of the group. This value is commonly displayed when the group is referenced.

### `identity_store_id`

- **Type:** aws.identitystore.Store.Id — identity: String(pattern); member: String(pattern, len: 1..=36)
- **Required:** Yes
- **Create-only:** Yes

The globally unique identifier for the identity store.

## Attribute Reference

### `group_id`

- **Type:** aws.sso.Principal.Id — identity: String(pattern); member: String(pattern, len: 1..=47)

The unique identifier for a group in the identity store.

## AWS API narrowings

These fields are narrower than or equal to CFN. Input rules combine documented AWS domains, recorded live observations, and Carina canonical spellings for stable state comparison. A canonical DSL spelling does not claim AWS rejects other spellings. Attaching a type name adds no structural constraints. Field names below use CFN spelling.

| CFN field | Relation to CFN | Evidence URL | Verbatim quote / exact observation | Checked / observed |
| --- | --- | --- | --- | --- |
| `/properties/DisplayName` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/singlesignon/latest/IdentityStoreAPIReference/API_CreateGroup.html#singlesignon-CreateGroup-request-DisplayName) | ` A string containing the name of the group. This value is commonly displayed when the group is referenced. Administrator and AWSAdministrators are reserved names and can't be used for users or groups. `<br>` Length Constraints: Minimum length of 1. Maximum length of 1024. `<br>` Pattern: [\p{L}\p{M}\p{S}\p{N}\p{P}\t\n\r ]+ ` | 2026-11-07 |

