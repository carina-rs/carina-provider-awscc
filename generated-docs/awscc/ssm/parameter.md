---
title: "awscc.ssm.Parameter"
---


CloudFormation Type: `AWS::SSM::Parameter`

The ``AWS::SSM::Parameter`` resource creates an SSM parameter in SYSlong Parameter Store.
  To create an SSM parameter, you must have the IAMlong (IAM) permissions ``ssm:PutParameter`` and ``ssm:AddTagsToResource``. On stack creation, CFNlong adds the following three tags to the parameter: ``aws:cloudformation:stack-name``, ``aws:cloudformation:logical-id``, and ``aws:cloudformation:stack-id``, in addition to any custom tags you specify.
 To add, update, or remove tags during stack update, you must have IAM permissions for both ``ssm:AddTagsToResource`` and ``ssm:RemoveTagsFromResource``. For more information, see [Managing access using policies](https://docs.aws.amazon.com/systems-manager/latest/userguide/security-iam.html#security_iam_access-manage) in the *User Guide*.
  For information about valid values for parameters, see [About requirements and constraints for parameter names](https://docs.aws.amazon.com/systems-manager/latest/userguide/sysman-paramstore-su-create.html#sysman-parameter-name-constraints) in the *User Guide* and [PutParameter](https://docs.aws.amazon.com/systems-manager/latest/APIReference/API_PutParameter.html) in the *API Reference*.

## Example

```crn
awscc.ssm.Parameter {
  name  = '/carina-registry/dev/publish/task_role_arn'
  type  = string
  value = 'arn:aws:iam::123456789012:role/carina-registry-publish-task'

  tags = {
    Environment = 'dev'
    Project     = 'carina-registry'
  }
}
```

## Argument Reference

### `allowed_pattern`

- **Type:** `String (len: unbounded..=1024)`
- **Required:** No
- **Write-only:** Yes

A regular expression used to validate the parameter value. For example, for ``String`` types with values restricted to numbers, you can specify the following: ``AllowedPattern=^\d+$``

### `data_type`

- **Type:** [DataType](#data_type-datatype)
- **Required:** No

The data type of the parameter, such as ``text`` or ``aws:ec2:image``. The default is ``text``.

### `description`

- **Type:** `String`
- **Required:** No
- **Write-only:** Yes

Information about the parameter.

### `name`

- **Type:** `String`
- **Required:** No
- **Create-only:** Yes

The name of the parameter. The reported maximum length of 2048 characters for a parameter name includes 1037 characters that are reserved for internal use by SYS. The maximum length for a parameter name that you specify is 1011 characters. This count of 1011 characters includes the characters in the ARN that precede the name you specify. This ARN length will vary depending on your partition and Region. For example, the following 45 characters count toward the 1011 character maximum for a parameter created in the US East (Ohio) Region: ``arn:aws:ssm:us-east-2:111122223333:parameter/``.

### `policies`

- **Type:** `String (len: 1..=4096)`
- **Required:** No
- **Write-only:** Yes

Information about the policies assigned to a parameter. [Assigning parameter policies](https://docs.aws.amazon.com/systems-manager/latest/userguide/parameter-store-policies.html) in the *User Guide*.

### `tags`

- **Type:** Map\<`String (pattern: /^([\p{L}\p{Z}\p{N}_.:/=+\-@]*)$/ (Full))`, `String`\>
- **Required:** No

Optional metadata that you assign to a resource in the form of an arbitrary set of tags (key-value pairs). Tags enable you to categorize a resource in different ways, such as by purpose, owner, or environment. For example, you might want to tag a SYS parameter to identify the type of resource to which it applies, the environment, or the type of configuration data referenced by the parameter.

### `tier`

- **Type:** [Tier](#tier-tier)
- **Required:** No
- **Write-only:** Yes

The parameter tier.

### `type`

- **Type:** [Type](#type-type)
- **Required:** Yes

The type of parameter. Parameters of type ``SecureString`` are not supported by CFNlong.

### `value`

- **Type:** `String`
- **Required:** Yes

The parameter value. If type is ``StringList``, the system returns a comma-separated string with no spaces between commas in the ``Value`` field.

## Enum Values

### data_type (DataType)

| Value | DSL Identifier |
|-------|----------------|
| `text` | `aws.ssm.Parameter.DataType.text` |
| `aws:ec2:image` | `aws.ssm.Parameter.DataType.aws_ec2_image` |

Shorthand formats: `text` or `DataType.text`

### tier (Tier)

| Value | DSL Identifier |
|-------|----------------|
| `Standard` | `aws.ssm.Parameter.Tier.standard` |
| `Advanced` | `aws.ssm.Parameter.Tier.advanced` |
| `Intelligent-Tiering` | `aws.ssm.Parameter.Tier.intelligent_tiering` |

Shorthand formats: `standard` or `Tier.standard`

### type (Type)

| Value | DSL Identifier |
|-------|----------------|
| `String` | `aws.ssm.Parameter.Type.string` |
| `StringList` | `aws.ssm.Parameter.Type.string_list` |

Shorthand formats: `string` or `Type.string`

## Attribute Reference

### `arn`

- **Type:** `aws.Arn (identity constraints: pattern: /^arn:(?:aws|aws-cn|aws-us-gov|aws-iso|aws-iso-b|aws-iso-e|aws-iso-f|aws-eusc):[^:]+:.*$/ (Full), member constraints: pattern: /arn:aws(-[a-z0-9-]+)*:ssm:[a-z0-9-]+:[0-9]{12}:parameter/.+/ (Full), custom validator)`



## AWS API narrowings

These fields are narrower than or equal to CFN. Input rules combine documented AWS domains, recorded live observations, and Carina canonical spellings for stable state comparison. A canonical DSL spelling does not claim AWS rejects other spellings. Attaching a type name adds no structural constraints. Field names below use CFN spelling.

| CFN field | Relation to CFN | Evidence URL | Verbatim quote / exact observation | Checked / observed |
| --- | --- | --- | --- | --- |
| `/properties/AllowedPattern` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/AWSCloudFormation/latest/TemplateReference/aws-resource-ssm-parameter.html#cfn-ssm-parameter-allowedpattern) | ` A regular expression used to validate the parameter value. For example, for String types with values restricted to numbers, you can specify the following: AllowedPattern=^\d+$ `<br>` Minimum: 0 `<br>` Maximum: 1024 ` | 2026-11-07 |
| `/properties/Policies` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/systems-manager/latest/APIReference/API_PutParameter.html#systemsmanager-PutParameter-request-Policies) | ` One or more policies to apply to a parameter. This operation takes a JSON array. Parameter Store, a tool in AWS Systems Manager supports the following policy types: `<br>` Expiration: This policy deletes the parameter after it expires. When you create the policy, you specify the expiration date. You can update the expiration date and time by updating the policy. Updating the parameter doesn't affect the expiration date and time. When the expiration time is reached, Parameter Store deletes the parameter. `<br>` ExpirationNotification: This policy initiates an event in Amazon CloudWatch Events that notifies you about the expiration. By using this policy, you can receive notification before or after the expiration time is reached, in units of days or hours. `<br>` NoChangeNotification: This policy initiates a CloudWatch Events event if a parameter hasn't been modified for a specified period of time. This policy type is useful when, for example, a secret needs to be changed within a period of time, but it hasn't been changed. `<br>` All existing policies are preserved until you send new policies or an empty policy. For more information about parameter policies, see Assigning parameter policies. `<br>` Length Constraints: Minimum length of 1. Maximum length of 4096. ` | 2026-11-07 |

