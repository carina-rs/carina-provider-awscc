---
title: "awscc.ec2.TransitGatewayAttachment"
description: "AWSCC EC2 TransitGatewayAttachment resource reference"
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

- **Type:** [Struct(Options)](#options)
- **Required:** No

The options for the transit gateway vpc attachment.

### `subnet_ids`

- **Type:** `List<SubnetId>`
- **Required:** Yes

### `tags`

- **Type:** `Map<String, String>`
- **Required:** No

### `transit_gateway_id`

- **Type:** TransitGatewayId
- **Required:** Yes
- **Create-only:** Yes

### `vpc_id`

- **Type:** VpcId
- **Required:** Yes
- **Create-only:** Yes

## Enum Values

### appliance_mode_support (OptionsApplianceModeSupport)

| Value | DSL Identifier |
|-------|----------------|
| `enable` | `awscc.ec2.TransitGatewayAttachment.OptionsApplianceModeSupport.enable` |
| `disable` | `awscc.ec2.TransitGatewayAttachment.OptionsApplianceModeSupport.disable` |

Shorthand formats: `enable` or `OptionsApplianceModeSupport.enable`

### dns_support (OptionsDnsSupport)

| Value | DSL Identifier |
|-------|----------------|
| `enable` | `awscc.ec2.TransitGatewayAttachment.OptionsDnsSupport.enable` |
| `disable` | `awscc.ec2.TransitGatewayAttachment.OptionsDnsSupport.disable` |

Shorthand formats: `enable` or `OptionsDnsSupport.enable`

### ipv6_support (OptionsIpv6Support)

| Value | DSL Identifier |
|-------|----------------|
| `enable` | `awscc.ec2.TransitGatewayAttachment.OptionsIpv6Support.enable` |
| `disable` | `awscc.ec2.TransitGatewayAttachment.OptionsIpv6Support.disable` |

Shorthand formats: `enable` or `OptionsIpv6Support.enable`

### security_group_referencing_support (OptionsSecurityGroupReferencingSupport)

| Value | DSL Identifier |
|-------|----------------|
| `enable` | `awscc.ec2.TransitGatewayAttachment.OptionsSecurityGroupReferencingSupport.enable` |
| `disable` | `awscc.ec2.TransitGatewayAttachment.OptionsSecurityGroupReferencingSupport.disable` |

Shorthand formats: `enable` or `OptionsSecurityGroupReferencingSupport.enable`

## Struct Definitions

### Options

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `appliance_mode_support` | [Enum (OptionsApplianceModeSupport)](#appliance_mode_support-optionsappliancemodesupport) | No | Indicates whether to enable Ipv6 Support for Vpc Attachment. Valid Values: enable | disable |
| `dns_support` | [Enum (OptionsDnsSupport)](#dns_support-optionsdnssupport) | No | Indicates whether to enable DNS Support for Vpc Attachment. Valid Values: enable | disable |
| `ipv6_support` | [Enum (OptionsIpv6Support)](#ipv6_support-optionsipv6support) | No | Indicates whether to enable Ipv6 Support for Vpc Attachment. Valid Values: enable | disable |
| `security_group_referencing_support` | [Enum (OptionsSecurityGroupReferencingSupport)](#security_group_referencing_support-optionssecuritygroupreferencingsupport) | No | Indicates whether to enable Security Group referencing support for Vpc Attachment. Valid Values: enable | disable |

## Attribute Reference

### `id`

- **Type:** TransitGatewayAttachmentId

