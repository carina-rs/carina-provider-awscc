---
title: "awscc.ec2.VpcPeeringConnection"
---


CloudFormation Type: `AWS::EC2::VPCPeeringConnection`

Resource Type definition for AWS::EC2::VPCPeeringConnection

## Example

```crn
let vpc1 = awscc.ec2.Vpc {
  cidr_block = '10.0.0.0/16'
}

let vpc2 = awscc.ec2.Vpc {
  cidr_block = '10.1.0.0/16'
}

awscc.ec2.VpcPeeringConnection {
  vpc_id      = vpc1.vpc_id
  peer_vpc_id = vpc2.vpc_id

  tags = {
    Environment = 'example'
  }
}
```

## Argument Reference

### `assume_role_region`

- **Type:** `aws.Region`
- **Required:** No
- **Create-only:** Yes
- **Write-only:** Yes

The Region code to use when calling Security Token Service (STS) to assume the PeerRoleArn, if provided.

### `peer_owner_id`

- **Type:** `aws.AccountId`
- **Required:** No
- **Create-only:** Yes

The AWS account ID of the owner of the accepter VPC.

### `peer_region`

- **Type:** `aws.Region`
- **Required:** No
- **Create-only:** Yes

The Region code for the accepter VPC, if the accepter VPC is located in a Region other than the Region in which you make the request.

### `peer_role_arn`

- **Type:** `aws.iam.Role.Arn`
- **Required:** No
- **Create-only:** Yes
- **Write-only:** Yes

The Amazon Resource Name (ARN) of the VPC peer role for the peering connection in another AWS account.

### `peer_vpc_id`

- **Type:** `aws.ec2.Vpc.Id`
- **Required:** Yes
- **Create-only:** Yes

The ID of the VPC with which you are creating the VPC peering connection. You must specify this parameter in the request.

### `tags`

- **Type:** `Map<String(len: ..=128), String(len: ..=256)>`
- **Required:** No

### `vpc_id`

- **Type:** `aws.ec2.Vpc.Id`
- **Required:** Yes
- **Create-only:** Yes

The ID of the VPC.

## Attribute Reference

### `id`

- **Type:** `aws.ec2.VpcPeeringConnection.Id`

## AWS API narrowings

These fields are narrower than or equal to CFN. Input rules combine documented AWS domains, recorded live observations, and Carina canonical spellings for stable state comparison. A canonical DSL spelling does not claim AWS rejects other spellings. Attaching a type name adds no structural constraints. Field names below use CFN spelling.

| CFN field | Relation to CFN | Evidence URL | Verbatim quote / exact observation | Checked / observed |
| --- | --- | --- | --- | --- |
| `/definitions/Tag/properties/Key` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/AWSEC2/latest/UserGuide/Using_Tags.html) | ` Maximum key length – 128 Unicode characters in UTF-8 ` | 2026-11-07 |
| `/definitions/Tag/properties/Value` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/AWSEC2/latest/APIReference/API_Tag.html) | ` The value of the tag. `<br>` Constraints: Tag values are case-sensitive and accept a maximum of 256 Unicode characters. ` | 2026-11-07 |

