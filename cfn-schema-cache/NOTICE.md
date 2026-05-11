# CloudFormation Resource Type Schemas — third-party content

The `*.json` files in this directory are AWS CloudFormation **resource type
schemas** obtained from the AWS CloudFormation Registry via:

```
aws cloudformation describe-type --type RESOURCE --type-name <Type>
```

They describe the public shape of AWS resources (e.g. `AWS::EC2::VPC`,
`AWS::S3::Bucket`) and are used as input to this repository's codegen
to produce `carina-provider-awscc/src/schemas/generated/`.

## Origin

- **Source:** AWS CloudFormation Registry (`describe-type` public API).
- **Owner:** Amazon Web Services, Inc. and/or its affiliates. The schemas
  describe AWS-provided resource types; the underlying intellectual
  property in the schemas belongs to AWS.
- **Retrieval method:** see
  `carina-provider-awscc/scripts/generate-schemas.sh`.

## Why they are checked in

These schemas are committed to the repository so that:

1. The CI's codegen-drift check can run without AWS credentials.
   CI re-runs the codegen against the committed cache and asserts the
   `src/schemas/generated/` tree is up to date.
2. Builds and tests are reproducible without network access to AWS.

The cache is updated by re-running `generate-schemas.sh` against fresh
AWS API output (see the script header for invocation).

## Precedent

AWS's own [`cdklabs/awscdk-service-spec`](https://github.com/cdklabs/awscdk-service-spec)
repository (Apache-2.0) follows the same pattern: it commits
CloudFormation schemas into a public open-source repo for use by the
AWS CDK.

## License

The schemas themselves carry no embedded license header. This repository
(`carina-rs/carina-provider-awscc`) is published under its own license
(see `LICENSE` at the repo root); that license applies only to the
project's own source code and does **not** purport to license the AWS
schemas themselves. The schemas are reproduced here as factual
descriptions of AWS public APIs.
