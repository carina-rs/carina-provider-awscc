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

- **Type:** `aws.identitystore.Group.DisplayName`
- **Required:** Yes

A string containing the name of the group. This value is commonly displayed when the group is referenced.

### `identity_store_id`

- **Type:** String(pattern, len: 1..=36)
- **Required:** Yes
- **Create-only:** Yes

The globally unique identifier for the identity store.

## Attribute Reference

### `group_id`

- **Type:** String(pattern, len: 1..=47)

The unique identifier for a group in the identity store.

