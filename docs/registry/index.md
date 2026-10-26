# AWS Cloud Control (AWSCC) Provider

The AWSCC provider manages supported AWS resources through the AWS Cloud Control API. It uses credentials from the standard AWS SDK credential chain. Configure it with a `provider awscc` block, then declare resources with names such as `awscc.s3.Bucket`. Use a `let` binding when another resource needs to reference its attributes.

```crn
provider awscc {
  region = awscc.Region.ap_northeast_1
}

let bucket = awscc.s3.Bucket {
  bucket_name = 'carina-example-bucket'
}
```

## Configuration

| Attribute | Type | Description |
| --- | --- | --- |
| `region` | `awscc.Region` enum | Region used by the AWS service clients. It defaults to `awscc.Region.ap_northeast_1`. Region identifiers use underscores in the DSL, for example `awscc.Region.us_east_1`. |
| `allowed_account_ids` | Unordered list of strings | When non-empty, initialization fails unless the effective AWS caller account is in the list. A determinable account in `assume_role.role_arn` must also be listed. |
| `forbidden_account_ids` | Unordered list of strings | When non-empty, initialization fails if the effective AWS caller account is in the list, even if it is also allowed by `allowed_account_ids`. |
| `assume_role` | `AssumeRole` struct | Uses STS to assume a role on top of the ambient credential chain; all provider operations then use the assumed-role credentials. |

`assume_role` accepts these fields:

| Field | Type | Required | Description |
| --- | --- | --- | --- |
| `role_arn` | String | Yes | IAM role ARN to assume. |
| `session_name` | String | No | STS session name for the assumed-role session. |
| `external_id` | String | No | External ID required by the role's trust policy. |
| `duration` | Duration | No | Session duration, such as `30min` or `1h`. STS accepts 15 minutes up to the role's maximum session duration. |
