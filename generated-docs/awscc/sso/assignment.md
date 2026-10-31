---
title: "awscc.sso.Assignment"
---


CloudFormation Type: `AWS::SSO::Assignment`

Resource Type definition for SSO assignmet

## Example

```crn
let read_only = awscc.sso.PermissionSet {
  instance_arn     = 'arn:aws:sso:::instance/ssoins-1234567890abcdef'
  name             = 'ExampleReadOnly'
  description      = 'Read-only access for the example workload'
  session_duration = 'PT1H'

  inline_policy = {
    version = aws.iam.PolicyDocument.Version.2012_10_17
    statement {
      effect   = aws.iam.PolicyDocument.Statement.Effect.allow
      action   = 's3:ListAllMyBuckets'
      resource = '*'
    }
  }
}

awscc.sso.Assignment {
  instance_arn       = read_only.instance_arn
  permission_set_arn = read_only.permission_set_arn
  principal_id       = '12345678-1234-1234-1234-123456789012'
  principal_type     = aws.sso.Assignment.PrincipalType.group
  target_id          = '123456789012'
  target_type        = aws.sso.Assignment.TargetType.aws_account
}
```

## Argument Reference

### `instance_arn`

- **Type:** String(pattern, len: 10..=1224)
- **Required:** Yes
- **Create-only:** Yes

The sso instance that the permission set is owned.

### `permission_set_arn`

- **Type:** String(pattern, len: 10..=1224)
- **Required:** Yes
- **Create-only:** Yes

The permission set that the assignment will be assigned

### `principal_id`

- **Type:** String(pattern, len: 1..=47)
- **Required:** Yes
- **Create-only:** Yes

The assignee's identifier, user id/group id

### `principal_type`

- **Type:** [Enum (PrincipalType)](#principal_type-principaltype)
- **Required:** Yes
- **Create-only:** Yes

The assignee's type, user/group

### `target_id`

- **Type:** AwsAccountId
- **Required:** Yes
- **Create-only:** Yes

The account id to be provisioned.

### `target_type`

- **Type:** [Enum (TargetType)](#target_type-targettype)
- **Required:** Yes
- **Create-only:** Yes

The type of resource to be provisioned to, only aws account now

## Enum Values

### principal_type (PrincipalType)

| Value | DSL Identifier |
|-------|----------------|
| `USER` | `aws.sso.Assignment.PrincipalType.user` |
| `GROUP` | `aws.sso.Assignment.PrincipalType.group` |

Shorthand formats: `user` or `PrincipalType.user`

### target_type (TargetType)

| Value | DSL Identifier |
|-------|----------------|
| `AWS_ACCOUNT` | `aws.sso.Assignment.TargetType.aws_account` |

Shorthand formats: `aws_account` or `TargetType.aws_account`

