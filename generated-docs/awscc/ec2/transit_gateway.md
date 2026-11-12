---
title: "awscc.ec2.TransitGateway"
---


CloudFormation Type: `AWS::EC2::TransitGateway`

Resource Type definition for AWS::EC2::TransitGateway

## Example

```crn
awscc.ec2.TransitGateway {
  description = 'Example Transit Gateway'

  tags = {
    Environment = 'example'
  }
}
```

## Argument Reference

### `amazon_side_asn`

- **Type:** [Union(amazon\_side\_asn)](#amazon_side_asn-1); CFN format: int64
- **Required:** No
- **Create-only:** Yes

### `association_default_route_table_id`

- **Type:** `aws.ec2.TransitGatewayRouteTable.Id (pattern: /^tgw-rtb-[0-9a-f]{8,}$/ (Full))`
- **Required:** No

### `auto_accept_shared_attachments`

- **Type:** [AutoAcceptSharedAttachments](#auto_accept_shared_attachments-autoacceptsharedattachments)
- **Required:** No

### `default_route_table_association`

- **Type:** [DefaultRouteTableAssociation](#default_route_table_association-defaultroutetableassociation)
- **Required:** No

### `default_route_table_propagation`

- **Type:** [DefaultRouteTablePropagation](#default_route_table_propagation-defaultroutetablepropagation)
- **Required:** No

### `description`

- **Type:** `String`
- **Required:** No

### `dns_support`

- **Type:** [DnsSupport](#dns_support-dnssupport)
- **Required:** No

### `encryption_support`

- **Type:** [EncryptionSupport](#encryption_support-encryptionsupport)
- **Required:** No
- **Write-only:** Yes

### `multicast_support`

- **Type:** [MulticastSupport](#multicast_support-multicastsupport)
- **Required:** No
- **Create-only:** Yes

### `propagation_default_route_table_id`

- **Type:** `aws.ec2.TransitGatewayRouteTable.Id (pattern: /^tgw-rtb-[0-9a-f]{8,}$/ (Full))`
- **Required:** No

### `security_group_referencing_support`

- **Type:** [SecurityGroupReferencingSupport](#security_group_referencing_support-securitygroupreferencingsupport)
- **Required:** No

### `tags`

- **Type:** Map\<`String (len: unbounded..=128)`, `String (len: unbounded..=256)`\>
- **Required:** No

### `transit_gateway_cidr_blocks`

- **Type:** List\<`String`\> (ordered)
- **Required:** No

### `vpn_ecmp_support`

- **Type:** [VpnEcmpSupport](#vpn_ecmp_support-vpnecmpsupport)
- **Required:** No

## Enum Values

### auto_accept_shared_attachments (AutoAcceptSharedAttachments)

| Value | DSL Identifier |
|-------|----------------|
| `enable` | `aws.ec2.TransitGateway.AutoAcceptSharedAttachments.enable` |
| `disable` | `aws.ec2.TransitGateway.AutoAcceptSharedAttachments.disable` |

Shorthand formats: `enable` or `AutoAcceptSharedAttachments.enable`

### default_route_table_association (DefaultRouteTableAssociation)

| Value | DSL Identifier |
|-------|----------------|
| `enable` | `aws.ec2.TransitGateway.DefaultRouteTableAssociation.enable` |
| `disable` | `aws.ec2.TransitGateway.DefaultRouteTableAssociation.disable` |

Shorthand formats: `enable` or `DefaultRouteTableAssociation.enable`

### default_route_table_propagation (DefaultRouteTablePropagation)

| Value | DSL Identifier |
|-------|----------------|
| `enable` | `aws.ec2.TransitGateway.DefaultRouteTablePropagation.enable` |
| `disable` | `aws.ec2.TransitGateway.DefaultRouteTablePropagation.disable` |

Shorthand formats: `enable` or `DefaultRouteTablePropagation.enable`

### dns_support (DnsSupport)

| Value | DSL Identifier |
|-------|----------------|
| `enable` | `aws.ec2.TransitGateway.DnsSupport.enable` |
| `disable` | `aws.ec2.TransitGateway.DnsSupport.disable` |

Shorthand formats: `enable` or `DnsSupport.enable`

### encryption_support (EncryptionSupport)

| Value | DSL Identifier |
|-------|----------------|
| `disable` | `aws.ec2.TransitGateway.EncryptionSupport.disable` |
| `enable` | `aws.ec2.TransitGateway.EncryptionSupport.enable` |

Shorthand formats: `disable` or `EncryptionSupport.disable`

### encryption_support_state (EncryptionSupportState)

| Value | DSL Identifier |
|-------|----------------|
| `enabling` | `aws.ec2.TransitGateway.EncryptionSupportState.enabling` |
| `enabled` | `aws.ec2.TransitGateway.EncryptionSupportState.enabled` |
| `disabling` | `aws.ec2.TransitGateway.EncryptionSupportState.disabling` |
| `disabled` | `aws.ec2.TransitGateway.EncryptionSupportState.disabled` |

Shorthand formats: `enabling` or `EncryptionSupportState.enabling`

### multicast_support (MulticastSupport)

| Value | DSL Identifier |
|-------|----------------|
| `enable` | `aws.ec2.TransitGateway.MulticastSupport.enable` |
| `disable` | `aws.ec2.TransitGateway.MulticastSupport.disable` |

Shorthand formats: `enable` or `MulticastSupport.enable`

### security_group_referencing_support (SecurityGroupReferencingSupport)

| Value | DSL Identifier |
|-------|----------------|
| `enable` | `aws.ec2.TransitGateway.SecurityGroupReferencingSupport.enable` |
| `disable` | `aws.ec2.TransitGateway.SecurityGroupReferencingSupport.disable` |

Shorthand formats: `enable` or `SecurityGroupReferencingSupport.enable`

### vpn_ecmp_support (VpnEcmpSupport)

| Value | DSL Identifier |
|-------|----------------|
| `enable` | `aws.ec2.TransitGateway.VpnEcmpSupport.enable` |
| `disable` | `aws.ec2.TransitGateway.VpnEcmpSupport.disable` |

Shorthand formats: `enable` or `VpnEcmpSupport.enable`

## Union Definitions

### amazon_side_asn

| Member | Type |
|--------|------|
| `member_0` | `Int (range: 64512..=65534)` |
| `member_1` | `Int (range: 4200000000..=4294967294)` |

## Attribute Reference

### `encryption_support_state`

- **Type:** [EncryptionSupportState](#encryption_support_state-encryptionsupportstate)

### `id`

- **Type:** `aws.ec2.TransitGateway.Id (pattern: /^tgw-[0-9a-f]{8,}$/ (Full))`

### `transit_gateway_arn`

- **Type:** `aws.Arn (pattern: /^arn:(?:aws|aws-cn|aws-us-gov|aws-iso|aws-iso-b|aws-iso-e|aws-iso-f|aws-eusc):[^:]+:.*$/ (Full), custom validator)`

## AWS API narrowings

These fields are narrower than or equal to CFN. Input rules combine documented AWS domains, recorded live observations, and Carina canonical spellings for stable state comparison. A canonical DSL spelling does not claim AWS rejects other spellings. Attaching a type name adds no structural constraints. Field names below use CFN spelling.

| CFN field | Relation to CFN | Evidence URL | Verbatim quote / exact observation | Checked / observed |
| --- | --- | --- | --- | --- |
| `/definitions/Tag/properties/Key` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/AWSEC2/latest/UserGuide/Using_Tags.html) | ` Maximum key length – 128 Unicode characters in UTF-8 ` | 2026-11-07 |
| `/definitions/Tag/properties/Value` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/AWSEC2/latest/APIReference/API_Tag.html) | ` The value of the tag. `<br>` Constraints: Tag values are case-sensitive and accept a maximum of 256 Unicode characters. ` | 2026-11-07 |
| `/properties/AmazonSideAsn` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/AWSEC2/latest/APIReference/API_ModifyTransitGatewayOptions.html) | ` A private Autonomous System Number (ASN) for the Amazon side of a BGP session. The range is 64512 to 65534 for 16-bit ASNs and 4200000000 to 4294967294 for 32-bit ASNs. `<br>` The modify ASN operation is not allowed on a transit gateway if it has the following attachments: `<br>` Dynamic VPN `<br>` Static VPN `<br>` Direct Connect Gateway `<br>` Connect `<br>` VPN Concentrator `<br>` Client VPN `<br>` You must first delete all transit gateway attachments configured prior to modifying the ASN on the transit gateway. ` | 2026-11-07 |
| `/properties/AutoAcceptSharedAttachments` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/AWSCloudFormation/latest/TemplateReference/aws-resource-ec2-transitgateway.html#cfn-ec2-transitgateway-autoacceptsharedattachments) | ` Enable or disable automatic acceptance of attachment requests. Disabled by default. `<br>` Allowed values: enable \| disable ` | 2026-11-07 |
| `/properties/DefaultRouteTableAssociation` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/AWSCloudFormation/latest/TemplateReference/aws-resource-ec2-transitgateway.html#cfn-ec2-transitgateway-defaultroutetableassociation) | ` Enable or disable automatic association with the default association route table. Enabled by default. If DefaultRouteTableAssociation is set to enable, AWS Transit Gateway will create the default transit gateway route table. `<br>` Allowed values: enable \| disable ` | 2026-11-07 |
| `/properties/DefaultRouteTablePropagation` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/AWSCloudFormation/latest/TemplateReference/aws-resource-ec2-transitgateway.html#cfn-ec2-transitgateway-defaultroutetablepropagation) | ` Enable or disable automatic propagation of routes to the default propagation route table. Enabled by default. If DefaultRouteTablePropagation is set to enable, AWS Transit Gateway will create the default transit gateway route table. `<br>` Allowed values: enable \| disable ` | 2026-11-07 |
| `/properties/DnsSupport` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/AWSCloudFormation/latest/TemplateReference/aws-resource-ec2-transitgateway.html#cfn-ec2-transitgateway-dnssupport) | ` Enable or disable DNS support. Enabled by default. `<br>` Allowed values: enable \| disable ` | 2026-11-07 |
| `/properties/EncryptionSupportState` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/AWSEC2/latest/APIReference/API_EncryptionSupport.html) | ` Valid Values: enabling \| enabled \| disabling \| disabled ` | 2026-11-07 |
| `/properties/MulticastSupport` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/AWSCloudFormation/latest/TemplateReference/aws-resource-ec2-transitgateway.html#cfn-ec2-transitgateway-multicastsupport) | ` Indicates whether multicast is enabled on the transit gateway `<br>` Allowed values: enable \| disable ` | 2026-11-07 |
| `/properties/SecurityGroupReferencingSupport` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/AWSCloudFormation/latest/TemplateReference/aws-resource-ec2-transitgateway.html#cfn-ec2-transitgateway-securitygroupreferencingsupport) | ` Enables you to reference a security group across VPCs attached to a transit gateway (TGW). Use this option to simplify security group management and control of instance-to-instance traffic across VPCs that are connected by transit gateway. You can also use this option to migrate from VPC peering (which was the only option that supported security group referencing) to transit gateways (which now also support security group referencing). This option is disabled by default and there are no additional costs to use this feature. `<br>` For important information about this feature, see Create a transit gateway in the AWS Transit Gateway Guide. `<br>` Allowed values: enable \| disable ` | 2026-11-07 |
| `/properties/VpnEcmpSupport` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/AWSCloudFormation/latest/TemplateReference/aws-resource-ec2-transitgateway.html#cfn-ec2-transitgateway-vpnecmpsupport) | ` Enable or disable Equal Cost Multipath Protocol support. Enabled by default. `<br>` Allowed values: enable \| disable ` | 2026-11-07 |

