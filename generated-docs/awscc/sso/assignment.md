---
title: "awscc.sso.Assignment"
---


CloudFormation Type: `AWS::SSO::Assignment`

Resource Type definition for SSO assignmet

## Example

```crn
let assigned_group = awscc.identitystore.Group {
  # Replace this placeholder with your Identity Store ID.
  identity_store_id = 'd-1234567890'
  display_name      = 'ExampleDevelopers'
  description       = 'Developers for the example workload'
}

let read_only = awscc.sso.PermissionSet {
  # Replace this placeholder with your organization's IAM Identity Center instance ARN.
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
  principal_id       = assigned_group.group_id
  principal_type     = aws.sso.Assignment.PrincipalType.group
  # Replace this placeholder with your AWS account ID.
  target_id          = '123456789012'
  target_type        = aws.sso.Assignment.TargetType.aws_account
}
```

## Argument Reference

### `instance_arn`

- **Type:** `aws.sso.Instance.Arn`
- **Required:** Yes
- **Create-only:** Yes

The sso instance that the permission set is owned.

### `permission_set_arn`

- **Type:** `aws.sso.PermissionSet.Arn`
- **Required:** Yes
- **Create-only:** Yes

The permission set that the assignment will be assigned

### `principal_id`

- **Type:** `aws.sso.Principal.Id`
- **Required:** Yes
- **Create-only:** Yes

The assignee's identifier, user id/group id

### `principal_type`

- **Type:** [Enum (PrincipalType)](#principal_type-principaltype)
- **Required:** Yes
- **Create-only:** Yes

The assignee's type, user/group

### `target_id`

- **Type:** `aws.AccountId`
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

## AWS API narrowings

These fields are narrower than or equal to CFN. Input rules combine documented AWS domains, recorded live observations, and Carina canonical spellings for stable state comparison. A canonical DSL spelling does not claim AWS rejects other spellings. Attaching a type name adds no structural constraints. Field names below use CFN spelling.

| CFN field | Relation to CFN | Evidence URL | Verbatim quote / exact observation | Checked / observed |
| --- | --- | --- | --- | --- |
| `/properties/InstanceArn` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/singlesignon/latest/APIReference/API_CreateAccountAssignment.html#singlesignon-CreateAccountAssignment-request-InstanceArn) | ` The ARN of the IAM Identity Center instance under which the operation will be executed. For more information about ARNs, see Amazon Resource Names (ARNs) and AWS Service Namespaces in the AWS General Reference. `<br>` Length Constraints: Minimum length of 10. Maximum length of 1224. `<br>` Pattern: arn:aws(-[a-z]{1,5}){0,3}:sso:::instance/(sso)?ins-[a-zA-Z0-9-.]{16} ` | 2026-11-07 |
| `/properties/PermissionSetArn` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/singlesignon/latest/APIReference/API_CreateAccountAssignment.html#singlesignon-CreateAccountAssignment-request-PermissionSetArn) | ` The ARN of the permission set that the admin wants to grant the principal access to. `<br>` Length Constraints: Minimum length of 10. Maximum length of 1224. `<br>` Pattern: arn:aws(-[a-z]{1,5}){0,3}:sso:::permissionSet/(sso)?ins-[a-zA-Z0-9-.]{16}/ps-[a-zA-Z0-9-./]{16} ` | 2026-11-07 |

