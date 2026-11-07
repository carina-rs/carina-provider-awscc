---
title: "awscc.sso.PermissionSet"
---


CloudFormation Type: `AWS::SSO::PermissionSet`

Resource Type definition for SSO PermissionSet

## Example

```crn
awscc.sso.PermissionSet {
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

  tags = {
    Environment = 'example'
  }
}
```

## Argument Reference

### `customer_managed_policy_references`

- **Type:** List\<[Struct(CustomerManagedPolicyReference)](#customer_managed_policy_reference)\> (items: ..=20)
- **Required:** No

### `description`

- **Type:** String(pattern, len: 1..=700)
- **Required:** No

The permission set description.

### `inline_policy`

- **Type:** PolicyDocument
- **Required:** No

The inline policy to put in permission set.

### `instance_arn`

- **Type:** `aws.sso.Instance.Arn`
- **Required:** Yes
- **Create-only:** Yes

The sso instance arn that the permission set is owned.

### `managed_policies`

- **Type:** `List<String(len: 20..=2048)>` (items: ..=20)
- **Required:** No

### `name`

- **Type:** String(pattern, len: 1..=32)
- **Required:** Yes
- **Create-only:** Yes

The name you want to assign to this permission set.

### `permissions_boundary`

- **Type:** [Struct(PermissionsBoundary)](#permissions_boundary-1)
- **Required:** No

### `relay_state_type`

- **Type:** String(pattern, len: 1..=240)
- **Required:** No

The relay state URL that redirect links to any service in the AWS Management Console.

### `session_duration`

- **Type:** `aws.sso.PermissionSet.SessionDuration`
- **Required:** No

The length of time that a user can be signed in to an AWS account.

### `tags`

- **Type:** `Map<String(pattern, len: 1..=128), String(pattern, len: ..=256)>` (entries: ..=50)
- **Required:** No

## Struct Definitions

### customer_managed_policy_reference

Used by: `customer_managed_policy_references`, `permissions_boundary.customer_managed_policy_reference`

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `name` | String(pattern, len: 1..=128) | Yes |  |
| `path` | String(pattern, len: 1..=512) | No |  |

### permissions_boundary

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `customer_managed_policy_reference` | [Struct(CustomerManagedPolicyReference)](#customer_managed_policy_reference) | No |  |
| `managed_policy_arn` | `aws.Arn` | No |  |

## Attribute Reference

### `permission_set_arn`

- **Type:** `aws.sso.PermissionSet.Arn`

The permission set that the policy will be attached to

## AWS API narrowings

These fields are narrower than or equal to CFN. Input rules combine documented AWS domains, recorded live observations, and Carina canonical spellings for stable state comparison. A canonical DSL spelling does not claim AWS rejects other spellings. Attaching a type name adds no structural constraints. Field names below use CFN spelling.

| CFN field | Relation to CFN | Evidence URL | Verbatim quote / exact observation | Checked / observed |
| --- | --- | --- | --- | --- |
| `/properties/InlinePolicy` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/IAM/latest/UserGuide/reference_policies_grammar.html) | ` <version_block> = "Version" : ("2008-10-17" \| "2012-10-17") `<br>` <effect_block> = "Effect" : ("Allow" \| "Deny") ` | 2026-11-07 |
| `/properties/InlinePolicy` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/IAM/latest/UserGuide/reference_policies_elements_statement.html) | ` The Statement element can contain a single statement or an array of individual statements. ` | 2026-11-07 |
| `/properties/InlinePolicy` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/IAM/latest/UserGuide/reference_policies_elements_condition.html) | ` You can create conditions that compare strings, dates, numbers, and more using typical Boolean comparisons such as equals, greater than, and less than. ` | 2026-11-07 |
| `/properties/InstanceArn` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/singlesignon/latest/APIReference/API_CreateAccountAssignment.html#singlesignon-CreateAccountAssignment-request-InstanceArn) | ` The ARN of the IAM Identity Center instance under which the operation will be executed. For more information about ARNs, see Amazon Resource Names (ARNs) and AWS Service Namespaces in the AWS General Reference. `<br>` Length Constraints: Minimum length of 10. Maximum length of 1224. `<br>` Pattern: arn:aws(-[a-z]{1,5}){0,3}:sso:::instance/(sso)?ins-[a-zA-Z0-9-.]{16} ` | 2026-11-07 |
| `/properties/PermissionSetArn` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/singlesignon/latest/APIReference/API_CreateAccountAssignment.html#singlesignon-CreateAccountAssignment-request-PermissionSetArn) | ` The ARN of the permission set that the admin wants to grant the principal access to. `<br>` Length Constraints: Minimum length of 10. Maximum length of 1224. `<br>` Pattern: arn:aws(-[a-z]{1,5}){0,3}:sso:::permissionSet/(sso)?ins-[a-zA-Z0-9-.]{16}/ps-[a-zA-Z0-9-./]{16} ` | 2026-11-07 |

