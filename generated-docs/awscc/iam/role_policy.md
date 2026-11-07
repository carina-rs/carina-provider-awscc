---
title: "awscc.iam.RolePolicy"
---


CloudFormation Type: `AWS::IAM::RolePolicy`

Adds or updates an inline policy document that is embedded in the specified IAM role.
 When you embed an inline policy in a role, the inline policy is used as part of the role's access (permissions) policy. The role's trust policy is created at the same time as the role, using [CreateRole](https://docs.aws.amazon.com/IAM/latest/APIReference/API_CreateRole.html). You can update a role's trust policy using [UpdateAssumeRolePolicy](https://docs.aws.amazon.com/IAM/latest/APIReference/API_UpdateAssumeRolePolicy.html). For information about roles, see [roles](https://docs.aws.amazon.com/IAM/latest/UserGuide/roles-toplevel.html) in the *IAM User Guide*.
 A role can also have a managed policy attached to it. To attach a managed policy to a role, use [AWS::IAM::Role](https://docs.aws.amazon.com/AWSCloudFormation/latest/UserGuide/aws-resource-iam-role.html). To create a new managed policy, use [AWS::IAM::ManagedPolicy](https://docs.aws.amazon.com/AWSCloudFormation/latest/UserGuide/aws-resource-iam-managedpolicy.html). For information about policies, see [Managed policies and inline policies](https://docs.aws.amazon.com/IAM/latest/UserGuide/policies-managed-vs-inline.html) in the *IAM User Guide*.
 For information about the maximum number of inline policies that you can embed with a role, see [IAM and quotas](https://docs.aws.amazon.com/IAM/latest/UserGuide/reference_iam-quotas.html) in the *IAM User Guide*.

## Example

```crn
let execution_role = awscc.iam.Role {
  role_name = 'carina-example-execution-role'

  assume_role_policy_document = {
    version = aws.iam.PolicyDocument.Version.2012_10_17
    statement {
      effect = aws.iam.PolicyDocument.Statement.Effect.allow

      principal = {
        service = 'lambda.amazonaws.com'
      }

      action = 'sts:AssumeRole'
    }
  }

  tags = {
    Environment = 'example'
  }
}

awscc.iam.RolePolicy {
  policy_name = 'ListBuckets'
  role_name   = execution_role.role_name

  policy_document = {
    version = aws.iam.PolicyDocument.Version.2012_10_17
    statement {
      effect   = aws.iam.PolicyDocument.Statement.Effect.allow
      action   = 's3:ListAllMyBuckets'
      resource = '*'
    }
  }
}
```

## Argument Reference

### `policy_document`

- **Type:** PolicyDocument
- **Required:** No

The policy document. You must provide policies in JSON format in IAM. However, for CFN templates formatted in YAML, you can provide the policy in JSON or YAML format. CFN always converts a YAML policy to JSON format before submitting it to IAM. The [regex pattern](https://docs.aws.amazon.com/http://wikipedia.org/wiki/regex) used to validate this parameter is a string of characters consisting of the following: + Any printable ASCII character ranging from the space character (``\u0020``) through the end of the ASCII character range + The printable characters in the Basic Latin and Latin-1 Supplement character set (through ``\u00FF``) + The special characters tab (``\u0009``), line feed (``\u000A``), and carriage return (``\u000D``)

### `policy_name`

- **Type:** `aws.iam.Policy.Name`
- **Required:** Yes
- **Create-only:** Yes

The name of the policy document. This parameter allows (through its [regex pattern](https://docs.aws.amazon.com/http://wikipedia.org/wiki/regex)) a string of characters consisting of upper and lowercase alphanumeric characters with no spaces. You can also include any of the following characters: _+=,.@-

### `role_name`

- **Type:** `aws.iam.Role.Name`
- **Required:** Yes
- **Create-only:** Yes

The name of the role to associate the policy with. This parameter allows (through its [regex pattern](https://docs.aws.amazon.com/http://wikipedia.org/wiki/regex)) a string of characters consisting of upper and lowercase alphanumeric characters with no spaces. You can also include any of the following characters: _+=,.@-

## AWS API narrowings

These fields are narrower than or equal to CFN. Input rules combine documented AWS domains, recorded live observations, and Carina canonical spellings for stable state comparison. A canonical DSL spelling does not claim AWS rejects other spellings. Attaching a type name adds no structural constraints. Field names below use CFN spelling.

| CFN field | Relation to CFN | Evidence URL | Verbatim quote / exact observation | Checked / observed |
| --- | --- | --- | --- | --- |
| `/properties/PolicyDocument` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/IAM/latest/UserGuide/reference_policies_grammar.html) | ` <version_block> = "Version" : ("2008-10-17" \| "2012-10-17") `<br>` <effect_block> = "Effect" : ("Allow" \| "Deny") ` | 2026-11-07 |
| `/properties/PolicyDocument` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/IAM/latest/UserGuide/reference_policies_elements_statement.html) | ` The Statement element can contain a single statement or an array of individual statements. ` | 2026-11-07 |
| `/properties/PolicyDocument` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/IAM/latest/UserGuide/reference_policies_elements_condition.html) | ` You can create conditions that compare strings, dates, numbers, and more using typical Boolean comparisons such as equals, greater than, and less than. ` | 2026-11-07 |
| `/properties/PolicyName` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/IAM/latest/APIReference/API_PutRolePolicy.html) | ` The name of the policy document. `<br>` This parameter allows (through its regex pattern) a string of characters consisting of upper and lowercase alphanumeric characters with no spaces. You can also include any of the following characters: _+=,.@- `<br>` Length Constraints: Minimum length of 1. Maximum length of 128. `<br>` Pattern: [\w+=,.@-]+ ` | 2026-11-07 |
| `/properties/RoleName` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/IAM/latest/APIReference/API_CreateRole.html) | ` The name of the role to create. `<br>` IAM user, group, role, and policy names must be unique within the account. Names are not distinguished by case. For example, you cannot create resources named both "MyResource" and "myresource". `<br>` This parameter allows (through its regex pattern) a string of characters consisting of upper and lowercase alphanumeric characters with no spaces. You can also include any of the following characters: _+=,.@- `<br>` Length Constraints: Minimum length of 1. Maximum length of 64. `<br>` Pattern: [\w+=,.@-]+ ` | 2026-11-07 |

