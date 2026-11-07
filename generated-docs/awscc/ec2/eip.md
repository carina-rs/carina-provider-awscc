---
title: "awscc.ec2.Eip"
---


CloudFormation Type: `AWS::EC2::EIP`

Specifies an Elastic IP (EIP) address and can, optionally, associate it with an Amazon EC2 instance.
 You can allocate an Elastic IP address from an address pool owned by AWS or from an address pool created from a public IPv4 address range that you have brought to AWS for use with your AWS resources using bring your own IP addresses (BYOIP). For more information, see [Bring Your Own IP Addresses (BYOIP)](https://docs.aws.amazon.com/AWSEC2/latest/UserGuide/ec2-byoip.html) in the *Amazon EC2 User Guide*.
 For more information, see [Elastic IP Addresses](https://docs.aws.amazon.com/AWSEC2/latest/UserGuide/elastic-ip-addresses-eip.html) in the *Amazon EC2 User Guide*.

## Example

```crn
awscc.ec2.Eip {
  domain = vpc

  tags = {
    Environment = 'example'
  }
}
```

## Argument Reference

### `address`

- **Type:** `Ipv4Address`
- **Required:** No
- **Create-only:** Yes
- **Write-only:** Yes



### `domain`

- **Type:** [Enum (Domain)](#domain-domain)
- **Required:** No

The network (``vpc``). If you define an Elastic IP address and associate it with a VPC that is defined in the same template, you must declare a dependency on the VPC-gateway attachment by using the [DependsOn Attribute](https://docs.aws.amazon.com/AWSCloudFormation/latest/UserGuide/aws-attribute-dependson.html) on this resource.

### `instance_id`

- **Type:** `aws.ec2.Instance.Id`
- **Required:** No

The ID of the instance. Updates to the ``InstanceId`` property may require *some interruptions*. Updates on an EIP reassociates the address on its associated resource.

### `ipam_pool_id`

- **Type:** `aws.ec2.IpamPool.Id`
- **Required:** No
- **Create-only:** Yes
- **Write-only:** Yes



### `network_border_group`

- **Type:** String
- **Required:** No
- **Create-only:** Yes

A unique set of Availability Zones, Local Zones, or Wavelength Zones from which AWS advertises IP addresses. Use this parameter to limit the IP address to this location. IP addresses cannot move between network border groups. Use [DescribeAvailabilityZones](https://docs.aws.amazon.com/AWSEC2/latest/APIReference/API_DescribeAvailabilityZones.html) to view the network border groups.

### `public_ipv4_pool`

- **Type:** String
- **Required:** No

The ID of an address pool that you own. Use this parameter to let Amazon EC2 select an address from the address pool. Updates to the ``PublicIpv4Pool`` property may require *some interruptions*. Updates on an EIP reassociates the address on its associated resource.

### `tags`

- **Type:** `Map<String(len: ..=128), String(len: ..=256)>`
- **Required:** No

Any tags assigned to the Elastic IP address. Updates to the ``Tags`` property may require *some interruptions*. Updates on an EIP reassociates the address on its associated resource.

### `transfer_address`

- **Type:** `Ipv4Address`
- **Required:** No
- **Create-only:** Yes
- **Write-only:** Yes

The Elastic IP address you are accepting for transfer. You can only accept one transferred address. For more information on Elastic IP address transfers, see [Transfer Elastic IP addresses](https://docs.aws.amazon.com/vpc/latest/userguide/vpc-eips.html#transfer-EIPs-intro) in the *Amazon Virtual Private Cloud User Guide*.

## Enum Values

### domain (Domain)

| Value | DSL Identifier |
|-------|----------------|
| `vpc` | `aws.ec2.Eip.Domain.vpc` |
| `standard` | `aws.ec2.Eip.Domain.standard` |

Shorthand formats: `vpc` or `Domain.vpc`

## Attribute Reference

### `allocation_id`

- **Type:** `aws.ec2.Eip.AllocationId`



### `public_ip`

- **Type:** `Ipv4Address`



## AWS API narrowings

These fields are narrower than or equal to CFN. Input rules combine documented AWS domains, recorded live observations, and Carina canonical spellings for stable state comparison. A canonical DSL spelling does not claim AWS rejects other spellings. Attaching a type name adds no structural constraints. Field names below use CFN spelling.

| CFN field | Relation to CFN | Evidence URL | Verbatim quote / exact observation | Checked / observed |
| --- | --- | --- | --- | --- |
| `/definitions/Tag/properties/Key` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/AWSEC2/latest/UserGuide/Using_Tags.html) | ` Maximum key length – 128 Unicode characters in UTF-8 ` | 2026-11-07 |
| `/definitions/Tag/properties/Value` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/AWSEC2/latest/APIReference/API_Tag.html) | ` The value of the tag. `<br>` Constraints: Tag values are case-sensitive and accept a maximum of 256 Unicode characters. ` | 2026-11-07 |
| `/properties/Domain` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/AWSCloudFormation/latest/TemplateReference/aws-resource-ec2-eip.html#cfn-ec2-eip-domain) | ` The network (vpc). `<br>` If you define an Elastic IP address and associate it with a VPC that is defined in the same template, you must declare a dependency on the VPC-gateway attachment by using the DependsOn Attribute on this resource. `<br>` Allowed values: vpc \| standard ` | 2026-11-07 |

