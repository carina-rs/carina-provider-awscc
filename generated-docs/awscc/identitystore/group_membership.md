---
title: "awscc.identitystore.GroupMembership"
---


CloudFormation Type: `AWS::IdentityStore::GroupMembership`

Resource Type Definition for AWS:IdentityStore::GroupMembership

## Example

```crn
let developers = awscc.identitystore.Group {
  identity_store_id = 'd-1234567890'
  display_name      = 'ExampleDevelopers'
  description       = 'Developers for the example workload'
}

awscc.identitystore.GroupMembership {
  identity_store_id = 'd-1234567890'
  group_id          = developers.group_id

  member_id = {
    user_id = '12345678-1234-1234-1234-123456789012'
  }
}
```

## Argument Reference

### `group_id`

- **Type:** String(pattern, len: 1..=47)
- **Required:** Yes
- **Create-only:** Yes

The unique identifier for a group in the identity store.

### `identity_store_id`

- **Type:** String(pattern, len: 1..=36)
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
| `user_id` | String(pattern, len: 1..=47) | Yes | The identifier for a user in the identity store. |

## Attribute Reference

### `membership_id`

- **Type:** String(pattern, len: 1..=47)

The identifier for a GroupMembership in the identity store.

