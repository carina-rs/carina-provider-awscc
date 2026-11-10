---
title: "awscc.ec2.EgressOnlyInternetGateway"
---


CloudFormation Type: `AWS::EC2::EgressOnlyInternetGateway`

Resource Type definition for AWS::EC2::EgressOnlyInternetGateway

## Example

```crn
let vpc = awscc.ec2.Vpc {
  cidr_block = '10.0.0.0/16'
}

awscc.ec2.EgressOnlyInternetGateway {
  vpc_id = vpc.vpc_id

  tags = {
    Environment = 'example'
  }
}
```

## Argument Reference

### `tags`

- **Type:** `Map<String(len: 1..=128), String(len: ..=256)>`
- **Required:** No

Any tags assigned to the egress only internet gateway.

### `vpc_id`

- **Type:** aws.ec2.Vpc.Id — identity: String(pattern); member: String
- **Required:** Yes
- **Create-only:** Yes

The ID of the VPC for which to create the egress-only internet gateway.

## Attribute Reference

### `id`

- **Type:** aws.ec2.EgressOnlyInternetGateway.Id — identity: String(pattern); member: String

Service Generated ID of the EgressOnlyInternetGateway

