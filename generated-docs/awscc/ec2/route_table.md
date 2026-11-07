---
title: "awscc.ec2.RouteTable"
---


CloudFormation Type: `AWS::EC2::RouteTable`

Specifies a route table for the specified VPC. After you create a route table, you can add routes and associate the table with a subnet.
 For more information, see [Route tables](https://docs.aws.amazon.com/vpc/latest/userguide/VPC_Route_Tables.html) in the *Amazon VPC User Guide*.

## Example

```crn
let vpc = awscc.ec2.Vpc {
  cidr_block           = '10.0.0.0/16'
  enable_dns_support   = true
  enable_dns_hostnames = true
}

awscc.ec2.RouteTable {
  vpc_id = vpc.vpc_id

  tags = {
    Environment = 'example'
  }
}
```

## Argument Reference

### `tags`

- **Type:** `Map<String(len: ..=128), String(len: ..=256)>`
- **Required:** No

Any tags assigned to the route table.

### `vpc_id`

- **Type:** `aws.ec2.Vpc.Id`
- **Required:** Yes
- **Create-only:** Yes

The ID of the VPC.

## Attribute Reference

### `route_table_id`

- **Type:** `aws.ec2.RouteTable.Id`



## AWS API narrowings

These fields are narrower than or equal to CFN. Input rules combine documented AWS domains, recorded live observations, and Carina canonical spellings for stable state comparison. A canonical DSL spelling does not claim AWS rejects other spellings. Attaching a type name adds no structural constraints. Field names below use CFN spelling.

| CFN field | Relation to CFN | Evidence URL | Verbatim quote / exact observation | Checked / observed |
| --- | --- | --- | --- | --- |
| `/definitions/Tag/properties/Key` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/AWSEC2/latest/UserGuide/Using_Tags.html) | ` Maximum key length – 128 Unicode characters in UTF-8 ` | 2026-11-07 |
| `/definitions/Tag/properties/Value` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/AWSEC2/latest/APIReference/API_Tag.html) | ` The value of the tag. `<br>` Constraints: Tag values are case-sensitive and accept a maximum of 256 Unicode characters. ` | 2026-11-07 |

