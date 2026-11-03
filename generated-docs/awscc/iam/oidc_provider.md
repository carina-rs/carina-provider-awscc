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

- **Type:** `List<String (length: 1..=255)> (items: ..=100)`
- **Required:** No

### `tags`

- **Type:** `Map<String, String>`
- **Required:** No

### `thumbprint_list`

- **Type:** `List<String(pattern, len: 40..=40)>` (items: ..=5)
- **Required:** No

### `url`

- **Type:** HttpsUrl
- **Required:** No
- **Create-only:** Yes

## Attribute Reference

### `arn`

- **Type:** IamOidcProviderArn

Amazon Resource Name (ARN) of the OIDC provider

