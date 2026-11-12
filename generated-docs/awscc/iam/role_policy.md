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

- **Type:** [Struct(PolicyDocument)](#policy_document-1)
- **Required:** No

The policy document. You must provide policies in JSON format in IAM. However, for CFN templates formatted in YAML, you can provide the policy in JSON or YAML format. CFN always converts a YAML policy to JSON format before submitting it to IAM. The [regex pattern](https://docs.aws.amazon.com/http://wikipedia.org/wiki/regex) used to validate this parameter is a string of characters consisting of the following: + Any printable ASCII character ranging from the space character (``\u0020``) through the end of the ASCII character range + The printable characters in the Basic Latin and Latin-1 Supplement character set (through ``\u00FF``) + The special characters tab (``\u0009``), line feed (``\u000A``), and carriage return (``\u000D``)

### `policy_name`

- **Type:** `aws.iam.Policy.Name (identity constraints: pattern: /^[A-Za-z0-9_+=,.@-]+\z/ (Full), len: 1..=128, member constraints: len: 1..=128)`
- **Required:** Yes
- **Create-only:** Yes

The name of the policy document. This parameter allows (through its [regex pattern](https://docs.aws.amazon.com/http://wikipedia.org/wiki/regex)) a string of characters consisting of upper and lowercase alphanumeric characters with no spaces. You can also include any of the following characters: _+=,.@-

### `role_name`

- **Type:** `aws.iam.Role.Name (identity constraints: pattern: /^[A-Za-z0-9_+=,.@-]+\z/ (Full), len: 1..=64, member constraints: len: 1..=64)`
- **Required:** Yes
- **Create-only:** Yes

The name of the role to associate the policy with. This parameter allows (through its [regex pattern](https://docs.aws.amazon.com/http://wikipedia.org/wiki/regex)) a string of characters consisting of upper and lowercase alphanumeric characters with no spaces. You can also include any of the following characters: _+=,.@-

## Enum Values

### policy_document.version (Version)

| Value | DSL Identifier |
|-------|----------------|
| `2012-10-17` | `aws.iam.PolicyDocument.Version.2012_10_17` |
| `2008-10-17` | `aws.iam.PolicyDocument.Version.2008_10_17` |

Use the fully qualified DSL identifiers above; bare numeric values parse as integers.

### statement.condition.\<key\> (ConditionOperator)

| Value | DSL Identifier |
|-------|----------------|
| `string_equals` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.string_equals` |
| `string_equals_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.string_equals_if_exists` |
| `for_all_values_string_equals` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_all_values_string_equals` |
| `for_all_values_string_equals_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_all_values_string_equals_if_exists` |
| `for_any_value_string_equals` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_any_value_string_equals` |
| `for_any_value_string_equals_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_any_value_string_equals_if_exists` |
| `string_not_equals` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.string_not_equals` |
| `string_not_equals_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.string_not_equals_if_exists` |
| `for_all_values_string_not_equals` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_all_values_string_not_equals` |
| `for_all_values_string_not_equals_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_all_values_string_not_equals_if_exists` |
| `for_any_value_string_not_equals` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_any_value_string_not_equals` |
| `for_any_value_string_not_equals_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_any_value_string_not_equals_if_exists` |
| `string_equals_ignore_case` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.string_equals_ignore_case` |
| `string_equals_ignore_case_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.string_equals_ignore_case_if_exists` |
| `for_all_values_string_equals_ignore_case` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_all_values_string_equals_ignore_case` |
| `for_all_values_string_equals_ignore_case_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_all_values_string_equals_ignore_case_if_exists` |
| `for_any_value_string_equals_ignore_case` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_any_value_string_equals_ignore_case` |
| `for_any_value_string_equals_ignore_case_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_any_value_string_equals_ignore_case_if_exists` |
| `string_not_equals_ignore_case` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.string_not_equals_ignore_case` |
| `string_not_equals_ignore_case_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.string_not_equals_ignore_case_if_exists` |
| `for_all_values_string_not_equals_ignore_case` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_all_values_string_not_equals_ignore_case` |
| `for_all_values_string_not_equals_ignore_case_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_all_values_string_not_equals_ignore_case_if_exists` |
| `for_any_value_string_not_equals_ignore_case` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_any_value_string_not_equals_ignore_case` |
| `for_any_value_string_not_equals_ignore_case_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_any_value_string_not_equals_ignore_case_if_exists` |
| `string_like` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.string_like` |
| `string_like_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.string_like_if_exists` |
| `for_all_values_string_like` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_all_values_string_like` |
| `for_all_values_string_like_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_all_values_string_like_if_exists` |
| `for_any_value_string_like` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_any_value_string_like` |
| `for_any_value_string_like_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_any_value_string_like_if_exists` |
| `string_not_like` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.string_not_like` |
| `string_not_like_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.string_not_like_if_exists` |
| `for_all_values_string_not_like` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_all_values_string_not_like` |
| `for_all_values_string_not_like_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_all_values_string_not_like_if_exists` |
| `for_any_value_string_not_like` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_any_value_string_not_like` |
| `for_any_value_string_not_like_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_any_value_string_not_like_if_exists` |
| `numeric_equals` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.numeric_equals` |
| `numeric_equals_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.numeric_equals_if_exists` |
| `for_all_values_numeric_equals` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_all_values_numeric_equals` |
| `for_all_values_numeric_equals_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_all_values_numeric_equals_if_exists` |
| `for_any_value_numeric_equals` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_any_value_numeric_equals` |
| `for_any_value_numeric_equals_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_any_value_numeric_equals_if_exists` |
| `numeric_not_equals` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.numeric_not_equals` |
| `numeric_not_equals_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.numeric_not_equals_if_exists` |
| `for_all_values_numeric_not_equals` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_all_values_numeric_not_equals` |
| `for_all_values_numeric_not_equals_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_all_values_numeric_not_equals_if_exists` |
| `for_any_value_numeric_not_equals` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_any_value_numeric_not_equals` |
| `for_any_value_numeric_not_equals_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_any_value_numeric_not_equals_if_exists` |
| `numeric_less_than` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.numeric_less_than` |
| `numeric_less_than_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.numeric_less_than_if_exists` |
| `for_all_values_numeric_less_than` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_all_values_numeric_less_than` |
| `for_all_values_numeric_less_than_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_all_values_numeric_less_than_if_exists` |
| `for_any_value_numeric_less_than` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_any_value_numeric_less_than` |
| `for_any_value_numeric_less_than_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_any_value_numeric_less_than_if_exists` |
| `numeric_less_than_equals` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.numeric_less_than_equals` |
| `numeric_less_than_equals_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.numeric_less_than_equals_if_exists` |
| `for_all_values_numeric_less_than_equals` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_all_values_numeric_less_than_equals` |
| `for_all_values_numeric_less_than_equals_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_all_values_numeric_less_than_equals_if_exists` |
| `for_any_value_numeric_less_than_equals` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_any_value_numeric_less_than_equals` |
| `for_any_value_numeric_less_than_equals_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_any_value_numeric_less_than_equals_if_exists` |
| `numeric_greater_than` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.numeric_greater_than` |
| `numeric_greater_than_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.numeric_greater_than_if_exists` |
| `for_all_values_numeric_greater_than` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_all_values_numeric_greater_than` |
| `for_all_values_numeric_greater_than_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_all_values_numeric_greater_than_if_exists` |
| `for_any_value_numeric_greater_than` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_any_value_numeric_greater_than` |
| `for_any_value_numeric_greater_than_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_any_value_numeric_greater_than_if_exists` |
| `numeric_greater_than_equals` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.numeric_greater_than_equals` |
| `numeric_greater_than_equals_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.numeric_greater_than_equals_if_exists` |
| `for_all_values_numeric_greater_than_equals` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_all_values_numeric_greater_than_equals` |
| `for_all_values_numeric_greater_than_equals_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_all_values_numeric_greater_than_equals_if_exists` |
| `for_any_value_numeric_greater_than_equals` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_any_value_numeric_greater_than_equals` |
| `for_any_value_numeric_greater_than_equals_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_any_value_numeric_greater_than_equals_if_exists` |
| `date_equals` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.date_equals` |
| `date_equals_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.date_equals_if_exists` |
| `for_all_values_date_equals` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_all_values_date_equals` |
| `for_all_values_date_equals_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_all_values_date_equals_if_exists` |
| `for_any_value_date_equals` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_any_value_date_equals` |
| `for_any_value_date_equals_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_any_value_date_equals_if_exists` |
| `date_not_equals` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.date_not_equals` |
| `date_not_equals_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.date_not_equals_if_exists` |
| `for_all_values_date_not_equals` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_all_values_date_not_equals` |
| `for_all_values_date_not_equals_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_all_values_date_not_equals_if_exists` |
| `for_any_value_date_not_equals` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_any_value_date_not_equals` |
| `for_any_value_date_not_equals_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_any_value_date_not_equals_if_exists` |
| `date_less_than` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.date_less_than` |
| `date_less_than_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.date_less_than_if_exists` |
| `for_all_values_date_less_than` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_all_values_date_less_than` |
| `for_all_values_date_less_than_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_all_values_date_less_than_if_exists` |
| `for_any_value_date_less_than` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_any_value_date_less_than` |
| `for_any_value_date_less_than_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_any_value_date_less_than_if_exists` |
| `date_less_than_equals` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.date_less_than_equals` |
| `date_less_than_equals_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.date_less_than_equals_if_exists` |
| `for_all_values_date_less_than_equals` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_all_values_date_less_than_equals` |
| `for_all_values_date_less_than_equals_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_all_values_date_less_than_equals_if_exists` |
| `for_any_value_date_less_than_equals` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_any_value_date_less_than_equals` |
| `for_any_value_date_less_than_equals_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_any_value_date_less_than_equals_if_exists` |
| `date_greater_than` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.date_greater_than` |
| `date_greater_than_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.date_greater_than_if_exists` |
| `for_all_values_date_greater_than` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_all_values_date_greater_than` |
| `for_all_values_date_greater_than_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_all_values_date_greater_than_if_exists` |
| `for_any_value_date_greater_than` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_any_value_date_greater_than` |
| `for_any_value_date_greater_than_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_any_value_date_greater_than_if_exists` |
| `date_greater_than_equals` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.date_greater_than_equals` |
| `date_greater_than_equals_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.date_greater_than_equals_if_exists` |
| `for_all_values_date_greater_than_equals` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_all_values_date_greater_than_equals` |
| `for_all_values_date_greater_than_equals_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_all_values_date_greater_than_equals_if_exists` |
| `for_any_value_date_greater_than_equals` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_any_value_date_greater_than_equals` |
| `for_any_value_date_greater_than_equals_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_any_value_date_greater_than_equals_if_exists` |
| `bool` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.bool` |
| `bool_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.bool_if_exists` |
| `for_all_values_bool` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_all_values_bool` |
| `for_all_values_bool_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_all_values_bool_if_exists` |
| `for_any_value_bool` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_any_value_bool` |
| `for_any_value_bool_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_any_value_bool_if_exists` |
| `binary_equals` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.binary_equals` |
| `binary_equals_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.binary_equals_if_exists` |
| `for_all_values_binary_equals` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_all_values_binary_equals` |
| `for_all_values_binary_equals_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_all_values_binary_equals_if_exists` |
| `for_any_value_binary_equals` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_any_value_binary_equals` |
| `for_any_value_binary_equals_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_any_value_binary_equals_if_exists` |
| `ip_address` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.ip_address` |
| `ip_address_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.ip_address_if_exists` |
| `for_all_values_ip_address` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_all_values_ip_address` |
| `for_all_values_ip_address_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_all_values_ip_address_if_exists` |
| `for_any_value_ip_address` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_any_value_ip_address` |
| `for_any_value_ip_address_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_any_value_ip_address_if_exists` |
| `not_ip_address` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.not_ip_address` |
| `not_ip_address_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.not_ip_address_if_exists` |
| `for_all_values_not_ip_address` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_all_values_not_ip_address` |
| `for_all_values_not_ip_address_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_all_values_not_ip_address_if_exists` |
| `for_any_value_not_ip_address` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_any_value_not_ip_address` |
| `for_any_value_not_ip_address_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_any_value_not_ip_address_if_exists` |
| `arn_equals` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.arn_equals` |
| `arn_equals_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.arn_equals_if_exists` |
| `for_all_values_arn_equals` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_all_values_arn_equals` |
| `for_all_values_arn_equals_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_all_values_arn_equals_if_exists` |
| `for_any_value_arn_equals` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_any_value_arn_equals` |
| `for_any_value_arn_equals_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_any_value_arn_equals_if_exists` |
| `arn_not_equals` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.arn_not_equals` |
| `arn_not_equals_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.arn_not_equals_if_exists` |
| `for_all_values_arn_not_equals` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_all_values_arn_not_equals` |
| `for_all_values_arn_not_equals_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_all_values_arn_not_equals_if_exists` |
| `for_any_value_arn_not_equals` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_any_value_arn_not_equals` |
| `for_any_value_arn_not_equals_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_any_value_arn_not_equals_if_exists` |
| `arn_like` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.arn_like` |
| `arn_like_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.arn_like_if_exists` |
| `for_all_values_arn_like` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_all_values_arn_like` |
| `for_all_values_arn_like_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_all_values_arn_like_if_exists` |
| `for_any_value_arn_like` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_any_value_arn_like` |
| `for_any_value_arn_like_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_any_value_arn_like_if_exists` |
| `arn_not_like` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.arn_not_like` |
| `arn_not_like_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.arn_not_like_if_exists` |
| `for_all_values_arn_not_like` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_all_values_arn_not_like` |
| `for_all_values_arn_not_like_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_all_values_arn_not_like_if_exists` |
| `for_any_value_arn_not_like` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_any_value_arn_not_like` |
| `for_any_value_arn_not_like_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_any_value_arn_not_like_if_exists` |
| `null` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.null` |
| `null_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.null_if_exists` |
| `for_all_values_null` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_all_values_null` |
| `for_all_values_null_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_all_values_null_if_exists` |
| `for_any_value_null` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_any_value_null` |
| `for_any_value_null_if_exists` | `aws.iam.PolicyDocument.Statement.Condition.ConditionOperator.for_any_value_null_if_exists` |

Shorthand formats: `string_equals` or `ConditionOperator.string_equals`

### statement.effect (Effect)

| Value | DSL Identifier |
|-------|----------------|
| `Allow` | `aws.iam.PolicyDocument.Statement.Effect.allow` |
| `Deny` | `aws.iam.PolicyDocument.Statement.Effect.deny` |

Shorthand formats: `allow` or `Effect.allow`

## Struct Definitions

### policy_document

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `version` | [Version](#policy_documentversion-version) | No |  |
| `id` | `String` | No |  |
| `statement` | [Union(policy\_document.statement)](#policy_documentstatement) | Yes |  |

### principal

Used by: `union_of_principal_or_string.member_0`

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `service` | [Union(union\_of\_string\_or\_list\_of\_string)](#union_of_string_or_list_of_string) | No |  |
| `aws` | [Union(union\_of\_string\_or\_list\_of\_string)](#union_of_string_or_list_of_string) | No |  |
| `federated` | [Union(union\_of\_string\_or\_list\_of\_string)](#union_of_string_or_list_of_string) | No |  |
| `canonical_user` | [Union(union\_of\_string\_or\_list\_of\_string)](#union_of_string_or_list_of_string) | No |  |

**Field groups:**

- At least one of: `service`, `aws`, `federated`, `canonical_user`.

### statement

Used by: `policy_document.statement.member_0`, `policy_document.statement.member_1`

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `sid` | `String` | No |  |
| `effect` | [Effect](#statementeffect-effect) | Yes |  |
| `action` | [Union(union\_of\_string\_or\_list\_of\_string)](#union_of_string_or_list_of_string) | No |  |
| `not_action` | [Union(union\_of\_string\_or\_list\_of\_string)](#union_of_string_or_list_of_string) | No |  |
| `resource` | [Union(union\_of\_string\_or\_list\_of\_string)](#union_of_string_or_list_of_string) | No |  |
| `not_resource` | [Union(union\_of\_string\_or\_list\_of\_string)](#union_of_string_or_list_of_string) | No |  |
| `principal` | [Union(union\_of\_principal\_or\_string)](#union_of_principal_or_string) | No |  |
| `not_principal` | [Union(union\_of\_principal\_or\_string)](#union_of_principal_or_string) | No |  |
| `condition` | Map\<[ConditionOperator](#statementconditionkey-conditionoperator), Map\<`String`, [Union(statement.condition.\<value\>.\<value\>)](#statementconditionvaluevalue)\>\> | No |  |

**Field groups:**

- Exactly one of: `action`, `not_action`.
- At most one of: `resource`, `not_resource`.
- At most one of: `principal`, `not_principal`.

## Union Definitions

### policy_document.statement

| Member | Type |
|--------|------|
| `member_0` | List\<[Struct(Statement)](#statement)\> (items: 1..unbounded, ordered) |
| `member_1` | [Struct(Statement)](#statement) |

### statement.condition.\<value\>.\<value\>

| Member | Type |
|--------|------|
| `member_0` | [Union(union\_of\_string\_or\_list\_of\_string)](#union_of_string_or_list_of_string) |
| `member_1` | `Int` |
| `member_2` | `Float` |
| `member_3` | `Bool` |
| `member_4` | List\<[Union(union\_of\_string\_or\_int\_or\_float\_or\_bool)](#union_of_string_or_int_or_float_or_bool)\> (items: 1..unbounded, unordered) |

### union_of_principal_or_string

Used by: `statement.not_principal`, `statement.principal`

| Member | Type |
|--------|------|
| `member_0` | [Struct(Principal)](#principal) |
| `member_1` | `String (pattern: /^\*\z/ (Full))` |

### union_of_string_or_int_or_float_or_bool

Used by: `statement.condition.<value>.<value>.member_4`

| Member | Type |
|--------|------|
| `member_0` | `String` |
| `member_1` | `Int` |
| `member_2` | `Float` |
| `member_3` | `Bool` |

### union_of_string_or_list_of_string

Used by: `principal.aws`, `principal.canonical_user`, `principal.federated`, `principal.service`, `statement.action`, `statement.not_action`, `statement.not_resource`, `statement.resource`, `statement.condition.<value>.<value>.member_0`

| Member | Type |
|--------|------|
| `member_0` | `String` |
| `member_1` | List\<`String`\> (items: 1..unbounded, unordered) |

## AWS API narrowings

These fields are narrower than or equal to CFN. Input rules combine documented AWS domains, recorded live observations, and Carina canonical spellings for stable state comparison. A canonical DSL spelling does not claim AWS rejects other spellings. Attaching a type name adds no structural constraints. Field names below use CFN spelling.

| CFN field | Relation to CFN | Evidence URL | Verbatim quote / exact observation | Checked / observed |
| --- | --- | --- | --- | --- |
| `/properties/PolicyDocument` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/IAM/latest/UserGuide/reference_policies_grammar.html) | ` <version_block> = "Version" : ("2008-10-17" \| "2012-10-17") `<br>` <effect_block> = "Effect" : ("Allow" \| "Deny") ` | 2026-11-07 |
| `/properties/PolicyDocument` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/IAM/latest/UserGuide/reference_policies_elements_statement.html) | ` The Statement element can contain a single statement or an array of individual statements. ` | 2026-11-07 |
| `/properties/PolicyDocument` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/IAM/latest/UserGuide/reference_policies_elements_condition.html) | ` You can create conditions that compare strings, dates, numbers, and more using typical Boolean comparisons such as equals, greater than, and less than. ` | 2026-11-07 |
| `/properties/PolicyName` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/IAM/latest/APIReference/API_PutRolePolicy.html) | ` The name of the policy document. `<br>` This parameter allows (through its regex pattern) a string of characters consisting of upper and lowercase alphanumeric characters with no spaces. You can also include any of the following characters: _+=,.@- `<br>` Length Constraints: Minimum length of 1. Maximum length of 128. `<br>` Pattern: [\w+=,.@-]+ ` | 2026-11-07 |
| `/properties/RoleName` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/IAM/latest/APIReference/API_CreateRole.html) | ` The name of the role to create. `<br>` IAM user, group, role, and policy names must be unique within the account. Names are not distinguished by case. For example, you cannot create resources named both "MyResource" and "myresource". `<br>` This parameter allows (through its regex pattern) a string of characters consisting of upper and lowercase alphanumeric characters with no spaces. You can also include any of the following characters: _+=,.@- `<br>` Length Constraints: Minimum length of 1. Maximum length of 64. `<br>` Pattern: [\w+=,.@-]+ ` | 2026-11-07 |

