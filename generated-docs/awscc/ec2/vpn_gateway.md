---
title: "awscc.ec2.VpnGateway"
---


CloudFormation Type: `AWS::EC2::VPNGateway`

Specifies a virtual private gateway. A virtual private gateway is the endpoint on the VPC side of your VPN connection. You can create a virtual private gateway before creating the VPC itself.
 For more information, see [](https://docs.aws.amazon.com/vpn/latest/s2svpn/VPC_VPN.html) in the *User Guide*.

## Example

```crn
awscc.ec2.VpnGateway {
  type = aws.ec2.VpnGateway.Type.ipsec_1

  tags = {
    Environment = 'example'
  }
}
```

## Argument Reference

### `amazon_side_asn`

- **Type:** `Int(64512..=65534) | Int(4200000000..=4294967294)`
- **Required:** No
- **Create-only:** Yes

The private Autonomous System Number (ASN) for the Amazon side of a BGP session.

### `tags`

- **Type:** `Map<String(len: ..=128), String(len: ..=256)>`
- **Required:** No

Any tags assigned to the virtual private gateway.

### `type`

- **Type:** [Enum (Type)](#type-type)
- **Required:** Yes
- **Create-only:** Yes

The type of VPN connection the virtual private gateway supports.

## Enum Values

### type (Type)

| Value | DSL Identifier |
|-------|----------------|
| `ipsec.1` | `aws.ec2.VpnGateway.Type.ipsec_1` |

Shorthand formats: `ipsec_1` or `Type.ipsec_1`

## Attribute Reference

### `vpn_gateway_id`

- **Type:** `aws.ec2.VpnGateway.Id`



## AWS API narrowings

These fields are narrower than or equal to CFN. Input rules combine documented AWS domains, recorded live observations, and Carina canonical spellings for stable state comparison. A canonical DSL spelling does not claim AWS rejects other spellings. Attaching a type name adds no structural constraints. Field names below use CFN spelling.

| CFN field | Relation to CFN | Evidence URL | Verbatim quote / exact observation | Checked / observed |
| --- | --- | --- | --- | --- |
| `/definitions/Tag/properties/Key` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/AWSEC2/latest/UserGuide/Using_Tags.html) | ` Maximum key length – 128 Unicode characters in UTF-8 ` | 2026-11-07 |
| `/definitions/Tag/properties/Value` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/AWSEC2/latest/APIReference/API_Tag.html) | ` The value of the tag. `<br>` Constraints: Tag values are case-sensitive and accept a maximum of 256 Unicode characters. ` | 2026-11-07 |
| `/properties/AmazonSideAsn` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/AWSEC2/latest/APIReference/API_CreateVpnGateway.html) | ` A private Autonomous System Number (ASN) for the Amazon side of a BGP session. If you're using a 16-bit ASN, it must be in the 64512 to 65534 range. If you're using a 32-bit ASN, it must be in the 4200000000 to 4294967294 range. `<br>` Default: 64512 ` | 2026-11-07 |
| `/properties/Type` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/AWSCloudFormation/latest/TemplateReference/aws-resource-ec2-vpngateway.html#cfn-ec2-vpngateway-type) | ` The type of VPN connection the virtual private gateway supports. `<br>` Allowed values: ipsec.1 ` | 2026-11-07 |

