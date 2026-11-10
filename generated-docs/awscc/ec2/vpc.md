---
title: "awscc.ec2.Vpc"
---


CloudFormation Type: `AWS::EC2::VPC`

Specifies a virtual private cloud (VPC).
 To add an IPv6 CIDR block to the VPC, see [AWS::EC2::VPCCidrBlock](https://docs.aws.amazon.com/AWSCloudFormation/latest/UserGuide/aws-resource-ec2-vpccidrblock.html).
 For more information, see [Virtual private clouds (VPC)](https://docs.aws.amazon.com/vpc/latest/userguide/configure-your-vpc.html) in the *Amazon VPC User Guide*.

## Example

```crn
awscc.ec2.Vpc {
  cidr_block           = '10.0.0.0/16'
  enable_dns_support   = true
  enable_dns_hostnames = true
  instance_tenancy     = default

  tags = {
    Environment = 'example'
  }
}
```

## Argument Reference

### `cidr_block`

- **Type:** Ipv4Cidr — identity: String; member: String
- **Required:** No
- **Create-only:** Yes

The IPv4 network range for the VPC, in CIDR notation. For example, ``10.0.0.0/16``. We modify the specified CIDR block to its canonical form; for example, if you specify ``100.68.0.18/18``, we modify it to ``100.68.0.0/18``. You must specify either``CidrBlock`` or ``Ipv4IpamPoolId``.

### `enable_dns_hostnames`

- **Type:** Bool
- **Required:** No

Indicates whether the instances launched in the VPC get DNS hostnames. If enabled, instances in the VPC get DNS hostnames; otherwise, they do not. Disabled by default for nondefault VPCs. For more information, see [DNS attributes in your VPC](https://docs.aws.amazon.com/vpc/latest/userguide/vpc-dns.html#vpc-dns-support). You can only enable DNS hostnames if you've enabled DNS support.

### `enable_dns_support`

- **Type:** Bool
- **Required:** No

Indicates whether the DNS resolution is supported for the VPC. If enabled, queries to the Amazon provided DNS server at the 169.254.169.253 IP address, or the reserved IP address at the base of the VPC network range "plus two" succeed. If disabled, the Amazon provided DNS service in the VPC that resolves public DNS hostnames to IP addresses is not enabled. Enabled by default. For more information, see [DNS attributes in your VPC](https://docs.aws.amazon.com/vpc/latest/userguide/vpc-dns.html#vpc-dns-support).

### `instance_tenancy`

- **Type:** [Enum (InstanceTenancy)](#instance_tenancy-instancetenancy)
- **Required:** No

The allowed tenancy of instances launched into the VPC. + ``default``: An instance launched into the VPC runs on shared hardware by default, unless you explicitly specify a different tenancy during instance launch. + ``dedicated``: An instance launched into the VPC runs on dedicated hardware by default, unless you explicitly specify a tenancy of ``host`` during instance launch. You cannot specify a tenancy of ``default`` during instance launch. Updating ``InstanceTenancy`` requires no replacement only if you are updating its value from ``dedicated`` to ``default``. Updating ``InstanceTenancy`` from ``default`` to ``dedicated`` requires replacement.

### `ipv4_ipam_pool_id`

- **Type:** aws.ec2.IpamPool.Id — identity: String(pattern); member: String
- **Required:** No
- **Create-only:** Yes
- **Write-only:** Yes

The ID of an IPv4 IPAM pool you want to use for allocating this VPC's CIDR. For more information, see [What is IPAM?](https://docs.aws.amazon.com//vpc/latest/ipam/what-is-it-ipam.html) in the *Amazon VPC IPAM User Guide*. You must specify either``CidrBlock`` or ``Ipv4IpamPoolId``.

### `ipv4_netmask_length`

- **Type:** Int(16..=28)
- **Required:** No
- **Create-only:** Yes
- **Write-only:** Yes

The netmask length of the IPv4 CIDR you want to allocate to this VPC from an Amazon VPC IP Address Manager (IPAM) pool. For more information about IPAM, see [What is IPAM?](https://docs.aws.amazon.com//vpc/latest/ipam/what-is-it-ipam.html) in the *Amazon VPC IPAM User Guide*.

### `tags`

- **Type:** `Map<String(len: ..=128), String(len: ..=256)>`
- **Required:** No

The tags for the VPC.

## Enum Values

### instance_tenancy (InstanceTenancy)

| Value | DSL Identifier |
|-------|----------------|
| `default` | `aws.ec2.Vpc.InstanceTenancy.default` |
| `dedicated` | `aws.ec2.Vpc.InstanceTenancy.dedicated` |

Shorthand formats: `default` or `InstanceTenancy.default`

## Attribute Reference

### `cidr_block_associations`

- **Type:** `List<aws.ec2.VpcCidrBlockAssociation.Id — identity: String(pattern); member: String>`



### `default_network_acl`

- **Type:** aws.ec2.NetworkAcl.Id — identity: String(pattern); member: String



### `default_security_group`

- **Type:** aws.ec2.SecurityGroup.Id — identity: String(pattern); member: String



### `ipv6_cidr_blocks`

- **Type:** `List<Ipv6Cidr — identity: String; member: String>`



### `vpc_id`

- **Type:** aws.ec2.Vpc.Id — identity: String(pattern); member: String



## AWS API narrowings

These fields are narrower than or equal to CFN. Input rules combine documented AWS domains, recorded live observations, and Carina canonical spellings for stable state comparison. A canonical DSL spelling does not claim AWS rejects other spellings. Attaching a type name adds no structural constraints. Field names below use CFN spelling.

| CFN field | Relation to CFN | Evidence URL | Verbatim quote / exact observation | Checked / observed |
| --- | --- | --- | --- | --- |
| `/definitions/Tag/properties/Key` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/AWSEC2/latest/UserGuide/Using_Tags.html) | ` Maximum key length – 128 Unicode characters in UTF-8 ` | 2026-11-07 |
| `/definitions/Tag/properties/Value` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/AWSEC2/latest/APIReference/API_Tag.html) | ` The value of the tag. `<br>` Constraints: Tag values are case-sensitive and accept a maximum of 256 Unicode characters. ` | 2026-11-07 |
| `/properties/InstanceTenancy` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/AWSEC2/latest/APIReference/API_CreateVpc.html) | ` The tenancy options for instances launched into the VPC. For default, instances are launched with shared tenancy by default. You can launch instances with any tenancy into a shared tenancy VPC. For dedicated, instances are launched as dedicated tenancy instances by default. You can only launch instances with a tenancy of dedicated or host into a dedicated tenancy VPC. `<br>` Important: The host value cannot be used with this parameter. Use the default or dedicated values only. `<br>` Default: default `<br>` Valid Values: default \| dedicated \| host ` | 2026-11-07 |
| `/properties/Ipv4NetmaskLength` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/vpc/latest/userguide/vpc-cidr-blocks.html) | ` When you create a VPC, you must specify an IPv4 CIDR block for the VPC. ` | 2026-11-07 |

