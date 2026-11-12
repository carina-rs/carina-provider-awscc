---
title: "awscc.iam.Role"
---


CloudFormation Type: `AWS::IAM::Role`

Creates a new role for your AWS-account.
  For more information about roles, see [IAM roles](https://docs.aws.amazon.com/IAM/latest/UserGuide/id_roles.html) in the *IAM User Guide*. For information about quotas for role names and the number of roles you can create, see [IAM and quotas](https://docs.aws.amazon.com/IAM/latest/UserGuide/reference_iam-quotas.html) in the *IAM User Guide*.

## Example

```crn
awscc.iam.Role {
  role_name = 'my-example-role'

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
```

## Read-back

After create and update, read-back waits for `arn` to appear.

The read-back budget is 120 seconds.

## Argument Reference

### `assume_role_policy_document`

- **Type:** [Struct(PolicyDocument)](#policy_document)
- **Required:** Yes

The trust policy that is associated with this role. Trust policies define which entities can assume the role. You can associate only one trust policy with a role. For an example of a policy that can be used to assume a role, see [Template Examples](https://docs.aws.amazon.com/AWSCloudFormation/latest/UserGuide/aws-resource-iam-role.html#aws-resource-iam-role--examples). For more information about the elements that you can use in an IAM policy, see [Policy Elements Reference](https://docs.aws.amazon.com/IAM/latest/UserGuide/reference_policies_elements.html) in the *User Guide*.

### `description`

- **Type:** `String (pattern: /^[\x09\x0A\x0D -~¡-ÿ]*\z/ (Full), len: unbounded..=1000)`
- **Required:** No

A description of the role that you provide.

### `managed_policy_arns`

- **Type:** List\<`aws.iam.Policy.Arn (pattern: /^arn:(?:aws|aws-cn|aws-us-gov|aws-iso|aws-iso-b|aws-iso-e|aws-iso-f|aws-eusc):iam::[^:]*:policy/.+$/ (Full), custom validator)`\> (unique items, unordered)
- **Required:** No

A list of Amazon Resource Names (ARNs) of the IAM managed policies that you want to attach to the role. For more information about ARNs, see [Amazon Resource Names (ARNs) and Service Namespaces](https://docs.aws.amazon.com/general/latest/gr/aws-arns-and-namespaces.html) in the *General Reference*.

### `max_session_duration`

- **Type:** `Int (range: 3600..=43200)`
- **Required:** No

The maximum session duration (in seconds) that you want to set for the specified role. If you do not specify a value for this setting, the default value of one hour is applied. This setting can have a value from 1 hour to 12 hours. Anyone who assumes the role from the CLI or API can use the ``DurationSeconds`` API parameter or the ``duration-seconds``CLI parameter to request a longer session. The ``MaxSessionDuration`` setting determines the maximum duration that can be requested using the ``DurationSeconds`` parameter. If users don't specify a value for the ``DurationSeconds`` parameter, their security credentials are valid for one hour by default. This applies when you use the ``AssumeRole*`` API operations or the ``assume-role*``CLI operations but does not apply when you use those operations to create a console URL. For more information, see [Using IAM roles](https://docs.aws.amazon.com/IAM/latest/UserGuide/id_roles_use.html) in the *IAM User Guide*.

### `path`

- **Type:** `String (pattern: /^(?:/|/[\x21-\x7E]+/)\z/ (Full), len: 1..=512)`
- **Required:** No
- **Create-only:** Yes
- **Default:** `"/"`

The path to the role. For more information about paths, see [IAM Identifiers](https://docs.aws.amazon.com/IAM/latest/UserGuide/Using_Identifiers.html) in the *IAM User Guide*. This parameter is optional. If it is not included, it defaults to a slash (/). This parameter allows (through its [regex pattern](https://docs.aws.amazon.com/http://wikipedia.org/wiki/regex)) a string of characters consisting of either a forward slash (/) by itself or a string that must begin and end with forward slashes. In addition, it can contain any ASCII character from the ! (``\u0021``) through the DEL character (``\u007F``), including most punctuation characters, digits, and upper and lowercased letters.

### `permissions_boundary`

- **Type:** `aws.iam.Policy.Arn (pattern: /^arn:(?:aws|aws-cn|aws-us-gov|aws-iso|aws-iso-b|aws-iso-e|aws-iso-f|aws-eusc):iam::[^:]*:policy/.+$/ (Full), custom validator)`
- **Required:** No

The ARN of the policy used to set the permissions boundary for the role. For more information about permissions boundaries, see [Permissions boundaries for IAM identities](https://docs.aws.amazon.com/IAM/latest/UserGuide/access_policies_boundaries.html) in the *IAM User Guide*.

### `policies`

- **Type:** List\<[Struct(Policy)](#policies-1)\> (unordered)
- **Required:** No

Adds or updates an inline policy document that is embedded in the specified IAM role. When you embed an inline policy in a role, the inline policy is used as part of the role's access (permissions) policy. The role's trust policy is created at the same time as the role. You can update a role's trust policy later. For more information about IAM roles, go to [Using Roles to Delegate Permissions and Federate Identities](https://docs.aws.amazon.com/IAM/latest/UserGuide/roles-toplevel.html). A role can also have an attached managed policy. For information about policies, see [Managed Policies and Inline Policies](https://docs.aws.amazon.com/IAM/latest/UserGuide/policies-managed-vs-inline.html) in the *User Guide*. For information about limits on the number of inline policies that you can embed with a role, see [Limitations on Entities](https://docs.aws.amazon.com/IAM/latest/UserGuide/LimitationsOnEntities.html) in the *User Guide*. If an external policy (such as ``AWS::IAM::Policy`` or ``AWS::IAM::ManagedPolicy``) has a ``Ref`` to a role and if a resource (such as ``AWS::ECS::Service``) also has a ``Ref`` to the same role, add a ``DependsOn`` attribute to the resource to make the resource depend on the external policy. This dependency ensures that the role's policy is available throughout the resource's lifecycle. For example, when you delete a stack with an ``AWS::ECS::Service`` resource, the ``DependsOn`` attribute ensures that CFN deletes the ``AWS::ECS::Service`` resource before deleting its role's policy.

### `role_name`

- **Type:** `aws.iam.Role.Name (identity constraints: pattern: /^[A-Za-z0-9_+=,.@-]+\z/ (Full), len: 1..=64, member constraints: len: 1..=64)`
- **Required:** No
- **Create-only:** Yes

A name for the IAM role, up to 64 characters in length. For valid values, see the ``RoleName`` parameter for the [CreateRole](https://docs.aws.amazon.com/IAM/latest/APIReference/API_CreateRole.html) action in the *User Guide*. This parameter allows (per its [regex pattern](https://docs.aws.amazon.com/http://wikipedia.org/wiki/regex)) a string of characters consisting of upper and lowercase alphanumeric characters with no spaces. You can also include any of the following characters: _+=,.@-. The role name must be unique within the account. Role names are not distinguished by case. For example, you cannot create roles named both "Role1" and "role1". If you don't specify a name, CFN generates a unique physical ID and uses that ID for the role name. If you specify a name, you must specify the ``CAPABILITY_NAMED_IAM`` value to acknowledge your template's capabilities. For more information, see [Acknowledging Resources in Templates](https://docs.aws.amazon.com/AWSCloudFormation/latest/UserGuide/using-iam-template.html#using-iam-capabilities). Naming an IAM resource can cause an unrecoverable error if you reuse the same template in multiple Regions. To prevent this, we recommend using ``Fn::Join`` and ``AWS::Region`` to create a Region-specific name, as in the following example: ``{"Fn::Join": ["", [{"Ref": "AWS::Region"}, {"Ref": "MyResourceName"}]]}``.

### `tags`

- **Type:** Map\<`String`, `String`\>
- **Required:** No

A list of tags that are attached to the role. For more information about tagging, see [Tagging IAM resources](https://docs.aws.amazon.com/IAM/latest/UserGuide/id_tags.html) in the *IAM User Guide*.

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

### policies

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `policy_document` | [Struct(PolicyDocument)](#policy_document) | Yes | The entire contents of the policy that defines permissions. For more information, see [Overview of JSON policies](https://docs.aws.amazon.com/IAM/latest/UserGuide/access_policies.html#access_policies-json). |
| `policy_name` | `aws.iam.Policy.Name (identity constraints: pattern: /^[A-Za-z0-9_+=,.@-]+\z/ (Full), len: 1..=128, member constraints: len: 1..=128)` | Yes | The friendly name (not ARN) identifying the policy. |

### policy_document

Used by: `assume_role_policy_document`, `policies.policy_document`

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

## Attribute Reference

### `arn`

- **Type:** `aws.iam.Role.Arn (pattern: /^arn:(?:aws|aws-cn|aws-us-gov|aws-iso|aws-iso-b|aws-iso-e|aws-iso-f|aws-eusc):iam::[^:]*:role/.+$/ (Full), custom validator)`



### `role_id`

- **Type:** `aws.iam.Role.Id (pattern: /^AROA[A-Z0-9]+$/ (Full))`



## AWS API narrowings

These fields are narrower than or equal to CFN. Input rules combine documented AWS domains, recorded live observations, and Carina canonical spellings for stable state comparison. A canonical DSL spelling does not claim AWS rejects other spellings. Attaching a type name adds no structural constraints. Field names below use CFN spelling.

| CFN field | Relation to CFN | Evidence URL | Verbatim quote / exact observation | Checked / observed |
| --- | --- | --- | --- | --- |
| `/definitions/Policy/properties/PolicyDocument` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/IAM/latest/UserGuide/reference_policies_grammar.html) | ` <version_block> = "Version" : ("2008-10-17" \| "2012-10-17") `<br>` <effect_block> = "Effect" : ("Allow" \| "Deny") ` | 2026-11-07 |
| `/definitions/Policy/properties/PolicyDocument` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/IAM/latest/UserGuide/reference_policies_elements_statement.html) | ` The Statement element can contain a single statement or an array of individual statements. ` | 2026-11-07 |
| `/definitions/Policy/properties/PolicyDocument` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/IAM/latest/UserGuide/reference_policies_elements_condition.html) | ` You can create conditions that compare strings, dates, numbers, and more using typical Boolean comparisons such as equals, greater than, and less than. ` | 2026-11-07 |
| `/definitions/Policy/properties/PolicyName` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/IAM/latest/APIReference/API_PutRolePolicy.html) | ` The name of the policy document. `<br>` This parameter allows (through its regex pattern) a string of characters consisting of upper and lowercase alphanumeric characters with no spaces. You can also include any of the following characters: _+=,.@- `<br>` Length Constraints: Minimum length of 1. Maximum length of 128. `<br>` Pattern: [\w+=,.@-]+ ` | 2026-11-07 |
| `/properties/AssumeRolePolicyDocument` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/IAM/latest/UserGuide/reference_policies_grammar.html) | ` <version_block> = "Version" : ("2008-10-17" \| "2012-10-17") `<br>` <effect_block> = "Effect" : ("Allow" \| "Deny") ` | 2026-11-07 |
| `/properties/AssumeRolePolicyDocument` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/IAM/latest/UserGuide/reference_policies_elements_statement.html) | ` The Statement element can contain a single statement or an array of individual statements. ` | 2026-11-07 |
| `/properties/AssumeRolePolicyDocument` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/IAM/latest/UserGuide/reference_policies_elements_condition.html) | ` You can create conditions that compare strings, dates, numbers, and more using typical Boolean comparisons such as equals, greater than, and less than. ` | 2026-11-07 |
| `/properties/Description` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/IAM/latest/APIReference/API_CreateRole.html) | ` A description of the role. `<br>` Length Constraints: Maximum length of 1000. `<br>` Pattern: [\u0009\u000A\u000D\u0020-\u007E\u00A1-\u00FF]* ` | 2026-11-07 |
| `/properties/MaxSessionDuration` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/AWSCloudFormation/latest/TemplateReference/aws-resource-iam-role.html#cfn-iam-role-maxsessionduration) | ` The maximum session duration (in seconds) that you want to set for the specified role. If you do not specify a value for this setting, the default value of one hour is applied. This setting can have a value from 1 hour to 12 hours. `<br>` Anyone who assumes the role from the AWS CLI or API can use the DurationSeconds API parameter or the duration-secondsAWS CLI parameter to request a longer session. The MaxSessionDuration setting determines the maximum duration that can be requested using the DurationSeconds parameter. If users don't specify a value for the DurationSeconds parameter, their security credentials are valid for one hour by default. This applies when you use the AssumeRole* API operations or the assume-role*AWS CLI operations but does not apply when you use those operations to create a console URL. For more information, see Using IAM roles in the IAM User Guide. `<br>` Minimum: 3600 `<br>` Maximum: 43200 ` | 2026-11-07 |
| `/properties/Path` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/IAM/latest/APIReference/API_CreateRole.html) | ` The path to the role. For more information about paths, see IAM Identifiers in the IAM User Guide. `<br>` This parameter is optional. If it is not included, it defaults to a slash (/). `<br>` This parameter allows (through its regex pattern) a string of characters consisting of either a forward slash (/) by itself or a string that must begin and end with forward slashes. In addition, it can contain any ASCII character from the ! (\u0021) through the DEL character (\u007F), including most punctuation characters, digits, and upper and lowercased letters. `<br>` Length Constraints: Minimum length of 1. Maximum length of 512. `<br>` Pattern: (\u002F)\|(\u002F[\u0021-\u007E]+\u002F) ` | 2026-11-07 |
| `/properties/RoleName` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/IAM/latest/APIReference/API_CreateRole.html) | ` The name of the role to create. `<br>` IAM user, group, role, and policy names must be unique within the account. Names are not distinguished by case. For example, you cannot create resources named both "MyResource" and "myresource". `<br>` This parameter allows (through its regex pattern) a string of characters consisting of upper and lowercase alphanumeric characters with no spaces. You can also include any of the following characters: _+=,.@- `<br>` Length Constraints: Minimum length of 1. Maximum length of 64. `<br>` Pattern: [\w+=,.@-]+ ` | 2026-11-07 |

