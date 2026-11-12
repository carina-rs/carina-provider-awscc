---
title: "awscc.identitystore.GroupMembership"
---


CloudFormation Type: `AWS::IdentityStore::GroupMembership`

Resource Type Definition for AWS:IdentityStore::GroupMembership

## Example

```crn
let developers = awscc.identitystore.Group {
  # Replace this placeholder with your Identity Store ID.
  identity_store_id = 'd-1234567890'
  display_name      = 'ExampleDevelopers'
  description       = 'Developers for the example workload'
}

awscc.identitystore.GroupMembership {
  identity_store_id = developers.identity_store_id
  group_id          = developers.group_id

  member_id = {
    # Replace this placeholder with an existing user's ID.
    user_id = '12345678-1234-1234-1234-123456789012'
  }
}
```

## Argument Reference

### `group_id`

- **Type:** `aws.sso.Principal.Id (identity constraints: pattern: /^.{1,64}$/ (Full), member constraints: pattern: /^([0-9a-f]{10}-|)[A-Fa-f0-9]{8}-[A-Fa-f0-9]{4}-[A-Fa-f0-9]{4}-[A-Fa-f0-9]{4}-[A-Fa-f0-9]{12}$/ (Full), len: 1..=47)`
- **Required:** Yes
- **Create-only:** Yes

The unique identifier for a group in the identity store.

### `identity_store_id`

- **Type:** `aws.identitystore.Store.Id (identity constraints: pattern: /^(d-[0-9a-fA-F]{10}|[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12})$/ (Full), member constraints: pattern: /^d-[0-9a-f]{10}$|^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/ (Full), len: 1..=36)`
- **Required:** Yes
- **Create-only:** Yes

The globally unique identifier for the identity store.

### `member_id`

- **Type:** [Struct(MemberId)](#member_id-1)
- **Required:** Yes
- **Create-only:** Yes

An object containing the identifier of a group member.

## Struct Definitions

### member_id

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `user_id` | `String (pattern: /^([0-9a-f]{10}-\|)[A-Fa-f0-9]{8}-[A-Fa-f0-9]{4}-[A-Fa-f0-9]{4}-[A-Fa-f0-9]{4}-[A-Fa-f0-9]{12}$/ (Full), len: 1..=47)` | Yes | The identifier for a user in the identity store. |

## Attribute Reference

### `membership_id`

- **Type:** `String (pattern: /^([0-9a-f]{10}-|)[A-Fa-f0-9]{8}-[A-Fa-f0-9]{4}-[A-Fa-f0-9]{4}-[A-Fa-f0-9]{4}-[A-Fa-f0-9]{12}$/ (Full), len: 1..=47)`

The identifier for a GroupMembership in the identity store.

