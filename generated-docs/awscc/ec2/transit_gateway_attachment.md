---
title: "awscc.ec2.TransitGatewayAttachment"
---


CloudFormation Type: `AWS::EC2::TransitGatewayAttachment`

Resource Type definition for AWS::EC2::TransitGatewayAttachment

## Example

```crn
let vpc = awscc.ec2.Vpc {
  cidr_block = '10.0.0.0/16'
}

let subnet = awscc.ec2.Subnet {
  vpc_id            = vpc.vpc_id
  cidr_block        = '10.0.1.0/24'
  availability_zone = 'ap-northeast-1a'
}

let tgw = awscc.ec2.TransitGateway {
  description = 'Example Transit Gateway'
}

awscc.ec2.TransitGatewayAttachment {
  transit_gateway_id = tgw.id
  vpc_id             = vpc.vpc_id
  subnet_ids         = [subnet.subnet_id]

  tags = {
    Environment = 'example'
  }
}
```

## Argument Reference

### `options`

- **Type:** [Struct(Options)](#options-1)
- **Required:** No

The options for the transit gateway vpc attachment.

### `subnet_ids`

- **Type:** `List<aws.ec2.Subnet.Id>`
- **Required:** Yes

### `tags`

- **Type:** `Map<String(len: ..=128), String(len: ..=256)>`
- **Required:** No

### `transit_gateway_id`

- **Type:** `aws.ec2.TransitGateway.Id`
- **Required:** Yes
- **Create-only:** Yes

### `vpc_id`

- **Type:** `aws.ec2.Vpc.Id`
- **Required:** Yes
- **Create-only:** Yes

## Enum Values

### options.appliance_mode_support (ApplianceModeSupport)

| Value | DSL Identifier |
|-------|----------------|
| `enable` | `aws.ec2.TransitGatewayAttachment.Options.ApplianceModeSupport.enable` |
| `disable` | `aws.ec2.TransitGatewayAttachment.Options.ApplianceModeSupport.disable` |

Shorthand formats: `enable` or `ApplianceModeSupport.enable`

### options.dns_support (DnsSupport)

| Value | DSL Identifier |
|-------|----------------|
| `enable` | `aws.ec2.TransitGatewayAttachment.Options.DnsSupport.enable` |
| `disable` | `aws.ec2.TransitGatewayAttachment.Options.DnsSupport.disable` |

Shorthand formats: `enable` or `DnsSupport.enable`

### options.ipv6_support (Ipv6Support)

| Value | DSL Identifier |
|-------|----------------|
| `enable` | `aws.ec2.TransitGatewayAttachment.Options.Ipv6Support.enable` |
| `disable` | `aws.ec2.TransitGatewayAttachment.Options.Ipv6Support.disable` |

Shorthand formats: `enable` or `Ipv6Support.enable`

### options.security_group_referencing_support (SecurityGroupReferencingSupport)

| Value | DSL Identifier |
|-------|----------------|
| `enable` | `aws.ec2.TransitGatewayAttachment.Options.SecurityGroupReferencingSupport.enable` |
| `disable` | `aws.ec2.TransitGatewayAttachment.Options.SecurityGroupReferencingSupport.disable` |

Shorthand formats: `enable` or `SecurityGroupReferencingSupport.enable`

## Struct Definitions

### options

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `appliance_mode_support` | [Enum (ApplianceModeSupport)](#optionsappliance_mode_support-appliancemodesupport) | No | Indicates whether to enable Ipv6 Support for Vpc Attachment. Valid Values: enable \| disable |
| `dns_support` | [Enum (DnsSupport)](#optionsdns_support-dnssupport) | No | Indicates whether to enable DNS Support for Vpc Attachment. Valid Values: enable \| disable |
| `ipv6_support` | [Enum (Ipv6Support)](#optionsipv6_support-ipv6support) | No | Indicates whether to enable Ipv6 Support for Vpc Attachment. Valid Values: enable \| disable |
| `security_group_referencing_support` | [Enum (SecurityGroupReferencingSupport)](#optionssecurity_group_referencing_support-securitygroupreferencingsupport) | No | Indicates whether to enable Security Group referencing support for Vpc Attachment. Valid Values: enable \| disable |

## Attribute Reference

### `id`

- **Type:** `aws.ec2.TransitGatewayAttachment.Id`

## AWS API narrowings

These fields are narrower than or equal to CFN. Input rules combine documented AWS domains, recorded live observations, and Carina canonical spellings for stable state comparison. A canonical DSL spelling does not claim AWS rejects other spellings. Attaching a type name adds no structural constraints. Field names below use CFN spelling.

| CFN field | Relation to CFN | Evidence URL | Verbatim quote / exact observation | Checked / observed |
| --- | --- | --- | --- | --- |
| `/definitions/Tag/properties/Key` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/AWSEC2/latest/UserGuide/Using_Tags.html) | ` Maximum key length – 128 Unicode characters in UTF-8 ` | 2026-11-07 |
| `/definitions/Tag/properties/Value` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/AWSEC2/latest/APIReference/API_Tag.html) | ` The value of the tag. `<br>` Constraints: Tag values are case-sensitive and accept a maximum of 256 Unicode characters. ` | 2026-11-07 |
| `/properties/Options/properties/DnsSupport` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/AWSCloudFormation/latest/TemplateReference/aws-properties-ec2-transitgatewayattachment-options.html#cfn-ec2-transitgatewayattachment-options-dnssupport) | ` Enable or disable DNS support. The default is disable. `<br>` Allowed values: enable \| disable ` | 2026-11-07 |
| `/properties/Options/properties/SecurityGroupReferencingSupport` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/AWSCloudFormation/latest/TemplateReference/aws-properties-ec2-transitgatewayattachment-options.html#cfn-ec2-transitgatewayattachment-options-securitygroupreferencingsupport) | ` Enables you to reference a security group across VPCs attached to a transit gateway (TGW). Use this option to simplify security group management and control of instance-to-instance traffic across VPCs that are connected by transit gateway. You can also use this option to migrate from VPC peering (which was the only option that supported security group referencing) to transit gateways (which now also support security group referencing). This option is disabled by default and there are no additional costs to use this feature. `<br>` For important information about this feature, see Create a transit gateway in the AWS Transit Gateway Guide. `<br>` Allowed values: enable \| disable ` | 2026-11-07 |

