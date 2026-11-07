---
title: "awscc.organizations.Account"
---


CloudFormation Type: `AWS::Organizations::Account`

You can use AWS::Organizations::Account to manage accounts in organization.

This resource is protected from destruction by default because deleting it closes the AWS account. To opt out, set `directives { prevent_destroy = false }`.

## Example

```crn
awscc.organizations.Account {
  account_name = 'carina-example-workloads'
  # Replace this placeholder with a real email address you control. Apply from
  # the organization's management account; this creates a real member account.
  email        = 'workloads@example.com'
  role_name    = 'OrganizationAccountAccessRole'

  tags = {
    Environment = 'example'
  }
}
```

## Argument Reference

### `account_name`

- **Type:** `String(pattern, len: 1..=50)`
- **Required:** Yes

The friendly name of the member account.

### `email`

- **Type:** aws.organizations.Account.Email — String(pattern, len: 6..=64)
- **Required:** Yes

The email address of the owner to assign to the new member account.

### `parent_ids`

- **Type:** `List<String(pattern, len: ..=100)>` (items: ..=1) (unique items)
- **Required:** No

List of parent nodes for the member account. Currently only one parent at a time is supported. Default is root.

### `role_name`

- **Type:** aws.organizations.Account.RoleName — String(pattern, len: 1..=64)
- **Required:** No
- **Write-only:** Yes
- **Default:** `"OrganizationAccountAccessRole"`

The name of an IAM role that AWS Organizations automatically preconfigures in the new member account. Default name is OrganizationAccountAccessRole if not specified.

### `tags`

- **Type:** `Map<String(pattern, len: 1..=128), String(pattern, len: ..=256)>`
- **Required:** No

A list of tags that you want to attach to the newly created account. For each tag in the list, you must specify both a tag key and a value.

## Enum Values

### joined_method (JoinedMethod)

| Value | DSL Identifier |
|-------|----------------|
| `INVITED` | `aws.organizations.Account.JoinedMethod.invited` |
| `CREATED` | `aws.organizations.Account.JoinedMethod.created` |

Shorthand formats: `invited` or `JoinedMethod.invited`

### state (State)

| Value | DSL Identifier |
|-------|----------------|
| `PENDING_ACTIVATION` | `aws.organizations.Account.State.pending_activation` |
| `ACTIVE` | `aws.organizations.Account.State.active` |
| `SUSPENDED` | `aws.organizations.Account.State.suspended` |
| `PENDING_CLOSURE` | `aws.organizations.Account.State.pending_closure` |
| `CLOSED` | `aws.organizations.Account.State.closed` |

Shorthand formats: `pending_activation` or `State.pending_activation`

### status (Status)

| Value | DSL Identifier |
|-------|----------------|
| `ACTIVE` | `aws.organizations.Account.Status.active` |
| `SUSPENDED` | `aws.organizations.Account.Status.suspended` |
| `PENDING_CLOSURE` | `aws.organizations.Account.Status.pending_closure` |

Shorthand formats: `active` or `Status.active`

## Attribute Reference

### `account_id`

- **Type:** aws.AccountId — String(pattern, len: ..=12)

If the account was created successfully, the unique identifier (ID) of the new account.

### `arn`

- **Type:** aws.organizations.Account.Arn — String(pattern)

The Amazon Resource Name (ARN) of the account.

### `joined_method`

- **Type:** [Enum (JoinedMethod)](#joined_method-joinedmethod)

The method by which the account joined the organization.

### `joined_timestamp`

- **Type:** String

The date the account became a part of the organization.

### `paths`

- **Type:** `List<String(pattern)>`

The paths in the organization where the account exists.

### `state`

- **Type:** [Enum (State)](#state-state)

The state of the account in the organization.

### `status`

- **Type:** [Enum (Status)](#status-status)

The status of the account in the organization.

## AWS API narrowings

These fields are narrower than or equal to CFN. Input rules combine documented AWS domains, recorded live observations, and Carina canonical spellings for stable state comparison. A canonical DSL spelling does not claim AWS rejects other spellings. Attaching a type name adds no structural constraints. Field names below use CFN spelling.

| CFN field | Relation to CFN | Evidence URL | Verbatim quote / exact observation | Checked / observed |
| --- | --- | --- | --- | --- |
| `/properties/AccountName` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/organizations/latest/APIReference/API_CreateAccount.html#organizations-CreateAccount-request-AccountName) | ` The friendly name of the member account. `<br>` Length Constraints: Minimum length of 1. Maximum length of 50. `<br>` Pattern: [\u0020-\u007E]+ ` | 2026-11-07 |
| `/properties/Email` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/organizations/latest/APIReference/API_CreateAccount.html#organizations-CreateAccount-request-Email) | ` The email address of the owner to assign to the new member account. This email address must not already be associated with another AWS account. You must use a valid email address to complete account creation. `<br>` The rules for a valid email address: `<br>` The address must be a minimum of 6 and a maximum of 64 characters long. `<br>` All characters must be 7-bit ASCII characters. `<br>` There must be one and only one @ symbol, which separates the local name from the domain name. `<br>` The local name can't contain any of the following characters: whitespace, " ' ( ) < > [ ] : ; , \ \| % & `<br>` The local name can't contain any of the following characters: `<br>` whitespace, " ' ( ) < > [ ] : ; , \ \| % & `<br>` The local name can't begin with a dot (.) `<br>` The domain name can consist of only the characters [a-z],[A-Z],[0-9], hyphen (-), or dot (.) `<br>` The domain name can't begin or end with a hyphen (-) or dot (.) `<br>` The domain name must contain at least one dot `<br>` You can't access the root user of the account or remove an account that was created with an invalid email address. `<br>` Length Constraints: Minimum length of 6. Maximum length of 64. `<br>` Pattern: See rules in parameter description ` | 2026-11-07 |
| `/properties/ParentIds` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/organizations/latest/APIReference/API_Parent.html#organizations-Type-Parent-Id) | ` The unique identifier (ID) of the parent entity. `<br>` The regex pattern for a parent ID string requires one of the following: `<br>` Root - A string that begins with "r-" followed by from 4 to 32 lowercase letters or digits. `<br>` Organizational unit (OU) - A string that begins with "ou-" followed by from 4 to 32 lowercase letters or digits (the ID of the root that the OU is in). This string is followed by a second "-" dash and from 8 to 32 additional lowercase letters or digits. `<br>` Length Constraints: Maximum length of 100. `<br>` Pattern: ^(r-[0-9a-z]{4,32})\|(ou-[0-9a-z]{4,32}-[a-z0-9]{8,32})$ ` | 2026-11-07 |
| `/properties/ParentIds` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/AWSCloudFormation/latest/TemplateReference/aws-resource-organizations-account.html#cfn-organizations-account-parentids) | ` The unique identifier (ID) of the root or organizational unit (OU) that you want to create the new account in. If you don't specify this parameter, the ParentId defaults to the root ID. `<br>` This parameter only accepts a string array with one string value. `<br>` The regex pattern for a parent ID string requires one of the following: `<br>` Root - A string that begins with "r-" followed by from 4 to 32 lowercase letters or digits. `<br>` Organizational unit (OU) - A string that begins with "ou-" followed by from 4 to 32 lowercase letters or digits (the ID of the root that the OU is in). This string is followed by a second "-" dash and from 8 to 32 additional lowercase letters or digits. `<br>` Pattern: ^(r-[0-9a-z]{4,32})\|(ou-[0-9a-z]{4,32}-[a-z0-9]{8,32})$ `<br>` Maximum: 100 ` | 2026-11-07 |
| `/properties/RoleName` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/organizations/latest/APIReference/API_CreateAccount.html#organizations-CreateAccount-request-RoleName) | ` The name of an IAM role that AWS Organizations automatically preconfigures in the new member account. This role trusts the management account, allowing users in the management account to assume the role, as permitted by the management account administrator. The role has administrator permissions in the new member account. `<br>` If you don't specify this parameter, the role name defaults to OrganizationAccountAccessRole. `<br>` For more information about how to use this role to access the member account, see the following links: `<br>` Creating the OrganizationAccountAccessRole in an invited member account in the AWS Organizations User Guide `<br>` Steps 2 and 3 in IAM Tutorial: Delegate access across AWS accounts using IAM roles in the IAM User Guide `<br>` The regex pattern that is used to validate this parameter. The pattern can include uppercase letters, lowercase letters, digits with no spaces, and any of the following characters: =,.@- `<br>` Length Constraints: Maximum length of 64. `<br>` Pattern: [\w+=,.@-]{1,64} ` | 2026-11-07 |

