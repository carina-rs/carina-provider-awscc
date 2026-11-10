---
title: "awscc.iam.OidcProvider"
---


CloudFormation Type: `AWS::IAM::OIDCProvider`

Resource Type definition for AWS::IAM::OIDCProvider

## Example

```crn
awscc.iam.OidcProvider {
  url            = 'https://token.actions.githubusercontent.com'
  client_id_list = ['sts.amazonaws.com']

  tags = {
    Environment = 'example'
  }
}
```

## Argument Reference

### `client_id_list`

- **Type:** `List<String(len: 1..=255)>` (items: ..=100)
- **Required:** No

### `tags`

- **Type:** `Map<String(len: 1..=128), String(len: 1..=256)>`
- **Required:** No

### `thumbprint_list`

- **Type:** `List<String(pattern, len: 40..=40)>` (items: ..=5)
- **Required:** No

### `url`

- **Type:** HttpsUrl — identity: String; member: String(len: 1..=255)
- **Required:** No
- **Create-only:** Yes

## Attribute Reference

### `arn`

- **Type:** aws.iam.OidcProvider.Arn — identity: String(pattern); member: String(len: 20..=2048)

Amazon Resource Name (ARN) of the OIDC provider

## AWS API narrowings

These fields are narrower than or equal to CFN. Input rules combine documented AWS domains, recorded live observations, and Carina canonical spellings for stable state comparison. A canonical DSL spelling does not claim AWS rejects other spellings. Attaching a type name adds no structural constraints. Field names below use CFN spelling.

| CFN field | Relation to CFN | Evidence URL | Verbatim quote / exact observation | Checked / observed |
| --- | --- | --- | --- | --- |
| `/properties/ClientIdList` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/IAM/latest/APIReference/API_CreateOpenIDConnectProvider.html) | ` Provides a list of client IDs, also known as audiences. When a mobile or web app registers with an OpenID Connect provider, they establish a value that identifies the application. This is the value that's sent as the client_id parameter on OAuth requests. `<br>` You can register multiple client IDs with the same provider. For example, you might have multiple applications that use the same OIDC provider. You cannot register more than 100 client IDs with a single IAM OIDC provider. `<br>` There is no defined format for a client ID. The CreateOpenIDConnectProviderRequest operation accepts client IDs up to 255 characters long. `<br>` Length Constraints: Minimum length of 1. Maximum length of 255. ` | 2026-11-07 |

