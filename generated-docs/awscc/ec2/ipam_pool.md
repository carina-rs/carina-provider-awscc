---
title: "awscc.ec2.IpamPool"
---


CloudFormation Type: `AWS::EC2::IPAMPool`

Resource Schema of AWS::EC2::IPAMPool Type

## Example

```crn
let ipam = awscc.ec2.Ipam {
  description = 'Example IPAM'
  tier        = free

  operating_region {
    region_name = 'ap-northeast-1'
  }
}

awscc.ec2.IpamPool {
  ipam_scope_id  = ipam.private_default_scope_id
  address_family = ipv4
  locale         = 'ap-northeast-1'
  description    = 'Example IPv4 IPAM Pool'

  provisioned_cidr {
    cidr = '10.0.0.0/8'
  }

  tags = {
    Environment = 'example'
  }
}
```

## Argument Reference

### `address_family`

- **Type:** [Enum (AddressFamily)](#address_family-addressfamily)
- **Required:** Yes
- **Create-only:** Yes

The address family of the address space in this pool. Either IPv4 or IPv6.

### `allocation_default_netmask_length`

- **Type:** Int(0..=128)
- **Required:** No

The default netmask length for allocations made from this pool. This value is used when the netmask length of an allocation isn't specified.

### `allocation_max_netmask_length`

- **Type:** Int(0..=128)
- **Required:** No

The maximum allowed netmask length for allocations made from this pool.

### `allocation_min_netmask_length`

- **Type:** Int(0..=128)
- **Required:** No

The minimum allowed netmask length for allocations made from this pool.

### `allocation_resource_tags`

- **Type:** `List<Map<String(len: 1..=128), String(len: ..=256)>>` (unique items)
- **Required:** No

When specified, an allocation will not be allowed unless a resource has a matching set of tags.

### `auto_import`

- **Type:** Bool
- **Required:** No

Determines what to do if IPAM discovers resources that haven't been assigned an allocation. If set to true, an allocation will be made automatically.

### `aws_service`

- **Type:** [Enum (AwsService)](#aws_service-awsservice)
- **Required:** No
- **Create-only:** Yes

Limits which service in Amazon Web Services that the pool can be used in.

### `description`

- **Type:** String
- **Required:** No

### `ipam_scope_id`

- **Type:** String
- **Required:** Yes
- **Create-only:** Yes

The Id of the scope this pool is a part of.

### `locale`

- **Type:** `String`
- **Required:** No
- **Create-only:** Yes

The region of this pool. If not set, this will default to "None" which will disable non-custom allocations. If the locale has been specified for the source pool, this value must match.

### `provisioned_cidrs`

- **Type:** List\<[Struct(ProvisionedCidr)](#provisioned_cidrs-1)\> (unique items)
- **Required:** No

A list of cidrs representing the address space available for allocation in this pool.

### `public_ip_source`

- **Type:** [Enum (PublicIpSource)](#public_ip_source-publicipsource)
- **Required:** No
- **Create-only:** Yes

The IP address source for pools in the public scope. Only used for provisioning IP address CIDRs to pools in the public scope. Default is `byoip`.

### `publicly_advertisable`

- **Type:** Bool
- **Required:** No
- **Create-only:** Yes

Determines whether or not address space from this pool is publicly advertised. Must be set if and only if the pool is IPv6.

### `source_ipam_pool_id`

- **Type:** aws.ec2.IpamPool.Id — identity: String(pattern); member: String
- **Required:** No
- **Create-only:** Yes

The Id of this pool's source. If set, all space provisioned in this pool must be free space provisioned in the parent pool.

### `source_resource`

- **Type:** [Struct(SourceResource)](#source_resource-1)
- **Required:** No
- **Create-only:** Yes

### `tags`

- **Type:** `Map<String(len: 1..=128), String(len: ..=256)>`
- **Required:** No

An array of key-value pairs to apply to this resource.

## Enum Values

### address_family (AddressFamily)

| Value | DSL Identifier |
|-------|----------------|
| `ipv4` | `aws.ec2.IpamPool.AddressFamily.ipv4` |
| `ipv6` | `aws.ec2.IpamPool.AddressFamily.ipv6` |

Shorthand formats: `ipv4` or `AddressFamily.ipv4`

### aws_service (AwsService)

| Value | DSL Identifier |
|-------|----------------|
| `ec2` | `aws.ec2.IpamPool.AwsService.ec2` |
| `global-services` | `aws.ec2.IpamPool.AwsService.global_services` |

Shorthand formats: `ec2` or `AwsService.ec2`

### ipam_scope_type (IpamScopeType)

| Value | DSL Identifier |
|-------|----------------|
| `public` | `aws.ec2.IpamPool.IpamScopeType.public` |
| `private` | `aws.ec2.IpamPool.IpamScopeType.private` |

Shorthand formats: `public` or `IpamScopeType.public`

### public_ip_source (PublicIpSource)

| Value | DSL Identifier |
|-------|----------------|
| `byoip` | `aws.ec2.IpamPool.PublicIpSource.byoip` |
| `amazon` | `aws.ec2.IpamPool.PublicIpSource.amazon` |

Shorthand formats: `byoip` or `PublicIpSource.byoip`

### source_resource.resource_type (ResourceType)

| Value | DSL Identifier |
|-------|----------------|
| `vpc` | `aws.ec2.IpamPool.SourceResource.ResourceType.vpc` |

Shorthand formats: `vpc` or `ResourceType.vpc`

### state (State)

| Value | DSL Identifier |
|-------|----------------|
| `create-in-progress` | `aws.ec2.IpamPool.State.create_in_progress` |
| `create-complete` | `aws.ec2.IpamPool.State.create_complete` |
| `modify-in-progress` | `aws.ec2.IpamPool.State.modify_in_progress` |
| `modify-complete` | `aws.ec2.IpamPool.State.modify_complete` |
| `delete-in-progress` | `aws.ec2.IpamPool.State.delete_in_progress` |
| `delete-complete` | `aws.ec2.IpamPool.State.delete_complete` |

Shorthand formats: `create_in_progress` or `State.create_in_progress`

## Struct Definitions

### provisioned_cidrs

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `cidr` | Ipv4Cidr — identity: String; member: String | Yes |  |

### source_resource

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `resource_id` | String | Yes |  |
| `resource_owner` | String | Yes |  |
| `resource_region` | aws.Region — identity: String; member: String | Yes |  |
| `resource_type` | [Enum (ResourceType)](#source_resourceresource_type-resourcetype) | Yes |  |

## Attribute Reference

### `arn`

- **Type:** aws.ec2.IpamPool.Arn — identity: String(pattern); member: String

The Amazon Resource Name (ARN) of the IPAM Pool.

### `ipam_arn`

- **Type:** aws.Arn — identity: String(pattern); member: String

The Amazon Resource Name (ARN) of the IPAM this pool is a part of.

### `ipam_pool_id`

- **Type:** aws.ec2.IpamPool.Id — identity: String(pattern); member: String

Id of the IPAM Pool.

### `ipam_scope_arn`

- **Type:** aws.Arn — identity: String(pattern); member: String

The Amazon Resource Name (ARN) of the scope this pool is a part of.

### `ipam_scope_type`

- **Type:** [Enum (IpamScopeType)](#ipam_scope_type-ipamscopetype)

Determines whether this scope contains publicly routable space or space for a private network

### `pool_depth`

- **Type:** Int

The depth of this pool in the source pool hierarchy.

### `state`

- **Type:** [Enum (State)](#state-state)

The state of this pool. This can be one of the following values: "create-in-progress", "create-complete", "modify-in-progress", "modify-complete", "delete-in-progress", or "delete-complete"

### `state_message`

- **Type:** String

An explanation of how the pool arrived at it current state.

## AWS API narrowings

These fields are narrower than or equal to CFN. Input rules combine documented AWS domains, recorded live observations, and Carina canonical spellings for stable state comparison. A canonical DSL spelling does not claim AWS rejects other spellings. Attaching a type name adds no structural constraints. Field names below use CFN spelling.

| CFN field | Relation to CFN | Evidence URL | Verbatim quote / exact observation | Checked / observed |
| --- | --- | --- | --- | --- |
| `/definitions/SourceResource/properties/ResourceType` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/AWSCloudFormation/latest/TemplateReference/aws-properties-ec2-ipampool-sourceresource.html#cfn-ec2-ipampool-sourceresource-resourcetype) | ` The source resource type. `<br>` Allowed values: vpc ` | 2026-11-07 |
| `/properties/AddressFamily` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/AWSCloudFormation/latest/TemplateReference/aws-resource-ec2-ipampool.html#cfn-ec2-ipampool-addressfamily) | ` The address family of the pool. `<br>` Allowed values: ipv4 \| ipv6 ` | 2026-11-07 |
| `/properties/AllocationDefaultNetmaskLength` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/AWSCloudFormation/latest/TemplateReference/aws-resource-ec2-ipampool.html#cfn-ec2-ipampool-allocationdefaultnetmasklength) | ` The default netmask length for allocations added to this pool. If, for example, the CIDR assigned to this pool is 10.0.0.0/8 and you enter 16 here, new allocations will default to 10.0.0.0/16. `<br>` Minimum: 0 `<br>` Maximum: 128 ` | 2026-11-07 |
| `/properties/AllocationMaxNetmaskLength` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/AWSCloudFormation/latest/TemplateReference/aws-resource-ec2-ipampool.html#cfn-ec2-ipampool-allocationmaxnetmasklength) | ` The maximum netmask length possible for CIDR allocations in this IPAM pool to be compliant. The maximum netmask length must be greater than the minimum netmask length. Possible netmask lengths for IPv4 addresses are 0 - 32. Possible netmask lengths for IPv6 addresses are 0 - 128. `<br>` Minimum: 0 `<br>` Maximum: 128 ` | 2026-11-07 |
| `/properties/AllocationMinNetmaskLength` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/AWSCloudFormation/latest/TemplateReference/aws-resource-ec2-ipampool.html#cfn-ec2-ipampool-allocationminnetmasklength) | ` The minimum netmask length required for CIDR allocations in this IPAM pool to be compliant. The minimum netmask length must be less than the maximum netmask length. Possible netmask lengths for IPv4 addresses are 0 - 32. Possible netmask lengths for IPv6 addresses are 0 - 128. `<br>` Minimum: 0 `<br>` Maximum: 128 ` | 2026-11-07 |

