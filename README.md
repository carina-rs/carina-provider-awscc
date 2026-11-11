# carina-provider-awscc

The AWS Cloud Control provider for [Carina](https://github.com/carina-rs/carina).

It manages resources through the [AWS Cloud Control API](https://docs.aws.amazon.com/cloudcontrolapi/latest/userguide/what-is-cloudcontrolapi.html), with schemas generated from the CloudFormation resource type definitions. Attribute names, types, enum values, and patterns in your `.crn` files are checked before any API call. The provider is compiled to a WebAssembly component and runs inside the Carina host's sandbox.

> [!CAUTION]
> Carina is experimental. Resource shapes and attributes may change without notice.

## Usage

```crn
provider awscc {
  source  = 'carina-rs/awscc'
  version = '0.11.1'
  region  = awscc.Region.ap_northeast_1
}

awscc.s3.Bucket {
  bucket_name = 'my-example-bucket'
}
```

```console
$ carina init
$ carina plan
```

`carina init` resolves the version through the provider registry, verifies the artifact's signature, and records it in `carina-providers.lock`. See [Installation](https://carina-rs.dev/getting-started/installation/) for the other `source` forms.

### Credentials

The Carina host resolves AWS credentials with the standard AWS credential chain (environment, shared config, SSO, and so on) and hands them to the provider, refreshing them when they expire. The provider routes each credential generation through `carina-aws-types` and host registration before signing, including assumed-role credentials. Provider diagnostics use the typed protocol 25 channel; guest stdout/stderr are discarded. Select a profile with `AWS_PROFILE`. Carina never starts an interactive login: if an SSO session has expired, run `aws sso login` and retry.

Planning KMS key references may require `kms:DescribeKey` for the effective identity,
including any configured assumed role. The provider resolves aliases and key IDs to
key ARNs for comparison, so equivalent values do not cause spurious changes. Authored
configuration and saved values remain unchanged.

## Supported services

Budgets, CloudFront, CloudWatch, DynamoDB, EC2, ECR, ECS, Elastic Load Balancing v2, IAM, Identity Store, KMS, CloudWatch Logs, Organizations, Route 53, S3, EventBridge Scheduler, SNS, SSM, IAM Identity Center (SSO), and WAFv2.

Per-resource reference documentation is in [`generated-docs/`](generated-docs/), and runnable configurations are in [`examples/`](examples/).

## Development

```console
$ cargo test
$ cargo build -p carina-provider-awscc --target wasm32-wasip2 --release
```

Schemas and docs are generated from the cached CloudFormation schemas in `cfn-schema-cache/`; edit the generator, not the output. See [CODEGEN.md](CODEGEN.md).

## Related

- [carina](https://github.com/carina-rs/carina) — CLI, DSL, and plugin host
- [carina-provider-aws](https://github.com/carina-rs/carina-provider-aws) — AWS provider built on the native AWS SDK
