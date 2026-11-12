---
title: "awscc.ec2.NatGateway"
---


CloudFormation Type: `AWS::EC2::NatGateway`

Specifies a network address translation (NAT) gateway in the specified subnet. You can create either a public NAT gateway or a private NAT gateway. The default is a public NAT gateway. If you create a public NAT gateway, you must specify an elastic IP address.
 With a NAT gateway, instances in a private subnet can connect to the internet, other AWS services, or an on-premises network using the IP address of the NAT gateway. For more information, see [NAT gateways](https://docs.aws.amazon.com/vpc/latest/userguide/vpc-nat-gateway.html) in the *Amazon VPC User Guide*.
 If you add a default route (``AWS::EC2::Route`` resource) that points to a NAT gateway, specify the NAT gateway ID for the route's ``NatGatewayId`` property.
  When you associate an Elastic IP address or secondary Elastic IP address with a public NAT gateway, the network border group of the Elastic IP address must match the network border group of the Availability Zone (AZ) that the public NAT gateway is in. Otherwise, the NAT gateway fails to launch. You can see the network border group for the AZ by viewing the details of the subnet. Similarly, you can view the network border group for the Elastic IP address by viewing its details. For more information, see [Allocate an Elastic IP address](https://docs.aws.amazon.com/vpc/latest/userguide/vpc-eips.html#allocate-eip) in the *Amazon VPC User Guide*.

## Example

```crn
let vpc = awscc.ec2.Vpc {
  cidr_block           = '10.0.0.0/16'
  enable_dns_support   = true
  enable_dns_hostnames = true
}

let public_subnet = awscc.ec2.Subnet {
  vpc_id                  = vpc.vpc_id
  cidr_block              = '10.0.1.0/24'
  availability_zone       = 'ap-northeast-1a'
  map_public_ip_on_launch = true
}

let internet_gateway = awscc.ec2.InternetGateway {}

let gateway_attachment = awscc.ec2.VpcGatewayAttachment {
  vpc_id              = vpc.vpc_id
  internet_gateway_id = internet_gateway.internet_gateway_id
}

let eip = awscc.ec2.Eip {
  domain = aws.ec2.Eip.Domain.vpc
}

awscc.ec2.NatGateway {
  allocation_id = eip.allocation_id
  subnet_id     = public_subnet.subnet_id

  directives {
    depends_on = [gateway_attachment]
  }

  tags = {
    Environment = 'example'
  }
}
```

## Argument Reference

### `allocation_id`

- **Type:** `aws.ec2.Eip.AllocationId (pattern: /^eipalloc-[0-9a-f]{8,}$/ (Full))`
- **Required:** No
- **Create-only:** Yes

[Public NAT gateway only] The allocation ID of the Elastic IP address that's associated with the NAT gateway. This property is required for a public NAT gateway and cannot be specified with a private NAT gateway.

### `availability_mode`

- **Type:** [AvailabilityMode](#availability_mode-availabilitymode)
- **Required:** No
- **Create-only:** Yes

Indicates whether this is a zonal (single-AZ) or regional (multi-AZ) NAT gateway. A zonal NAT gateway is a NAT Gateway that provides redundancy and scalability within a single availability zone. A regional NAT gateway is a single NAT Gateway that works across multiple availability zones (AZs) in your VPC, providing redundancy, scalability and availability across all the AZs in a Region. For more information, see [Regional NAT gateways for automatic multi-AZ expansion](https://docs.aws.amazon.com/vpc/latest/userguide/nat-gateways-regional.html) in the *Amazon VPC User Guide*.

### `availability_zone_addresses`

- **Type:** List\<[Struct(AvailabilityZoneAddress)](#availability_zone_addresses-1)\> (unique items, unordered)
- **Required:** No

For regional NAT gateways only: Specifies which Availability Zones you want the NAT gateway to support and the Elastic IP addresses (EIPs) to use in each AZ. The regional NAT gateway uses these EIPs to handle outbound NAT traffic from their respective AZs. If not specified, the NAT gateway will automatically expand to new AZs and associate EIPs upon detection of an elastic network interface. If you specify this parameter, auto-expansion is disabled and you must manually manage AZ coverage. A regional NAT gateway is a single NAT Gateway that works across multiple availability zones (AZs) in your VPC, providing redundancy, scalability and availability across all the AZs in a Region. For more information, see [Regional NAT gateways for automatic multi-AZ expansion](https://docs.aws.amazon.com/vpc/latest/userguide/nat-gateways-regional.html) in the *Amazon VPC User Guide*.

### `connectivity_type`

- **Type:** [ConnectivityType](#connectivity_type-connectivitytype)
- **Required:** No
- **Create-only:** Yes

Indicates whether the NAT gateway supports public or private connectivity. The default is public connectivity.

### `max_drain_duration_seconds`

- **Type:** `Int`
- **Required:** No
- **Write-only:** Yes

The maximum amount of time to wait (in seconds) before forcibly releasing the IP addresses if connections are still in progress. Default value is 350 seconds.

### `private_ip_address`

- **Type:** `Ipv4Address (custom validator)`
- **Required:** No
- **Create-only:** Yes

The private IPv4 address to assign to the NAT gateway. If you don't provide an address, a private IPv4 address will be automatically assigned.

### `secondary_allocation_ids`

- **Type:** List\<`aws.ec2.Eip.AllocationId (pattern: /^eipalloc-[0-9a-f]{8,}$/ (Full))`\> (unique items, ordered)
- **Required:** No

Secondary EIP allocation IDs. For more information, see [Create a NAT gateway](https://docs.aws.amazon.com/vpc/latest/userguide/nat-gateway-working-with.html) in the *Amazon VPC User Guide*.

### `secondary_private_ip_address_count`

- **Type:** `Int (range: 1..=31)`
- **Required:** No

[Private NAT gateway only] The number of secondary private IPv4 addresses you want to assign to the NAT gateway. For more information about secondary addresses, see [Create a NAT gateway](https://docs.aws.amazon.com/vpc/latest/userguide/vpc-nat-gateway.html#nat-gateway-creating) in the *Amazon Virtual Private Cloud User Guide*. ``SecondaryPrivateIpAddressCount`` and ``SecondaryPrivateIpAddresses`` cannot be set at the same time.

### `secondary_private_ip_addresses`

- **Type:** List\<`Ipv4Address (custom validator)`\> (unique items, ordered)
- **Required:** No

Secondary private IPv4 addresses. For more information about secondary addresses, see [Create a NAT gateway](https://docs.aws.amazon.com/vpc/latest/userguide/vpc-nat-gateway.html#nat-gateway-creating) in the *Amazon Virtual Private Cloud User Guide*. ``SecondaryPrivateIpAddressCount`` and ``SecondaryPrivateIpAddresses`` cannot be set at the same time.

### `subnet_id`

- **Type:** `aws.ec2.Subnet.Id (pattern: /^subnet-[0-9a-f]{8,}$/ (Full))`
- **Required:** No
- **Create-only:** Yes

The ID of the subnet in which the NAT gateway is located.

### `tags`

- **Type:** Map\<`String (len: unbounded..=128)`, `String (len: unbounded..=256)`\>
- **Required:** No

The tags for the NAT gateway.

### `vpc_id`

- **Type:** `aws.ec2.Vpc.Id (pattern: /^vpc-[0-9a-f]{8,}$/ (Full))`
- **Required:** No
- **Create-only:** Yes

The ID of the VPC in which the NAT gateway is located.

## Enum Values

### availability_mode (AvailabilityMode)

| Value | DSL Identifier |
|-------|----------------|
| `zonal` | `aws.ec2.NatGateway.AvailabilityMode.zonal` |
| `regional` | `aws.ec2.NatGateway.AvailabilityMode.regional` |

Shorthand formats: `zonal` or `AvailabilityMode.zonal`

### connectivity_type (ConnectivityType)

| Value | DSL Identifier |
|-------|----------------|
| `public` | `aws.ec2.NatGateway.ConnectivityType.public` |
| `private` | `aws.ec2.NatGateway.ConnectivityType.private` |

Shorthand formats: `public` or `ConnectivityType.public`

## Struct Definitions

### availability_zone_addresses

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `allocation_ids` | List\<`aws.ec2.Eip.AllocationId (pattern: /^eipalloc-[0-9a-f]{8,}$/ (Full))`\> (unique items, unordered) | Yes | The allocation IDs of the Elastic IP addresses (EIPs) to be used for handling outbound NAT traffic in this specific Availability Zone. |
| `availability_zone` | `aws.AvailabilityZone.ZoneName (custom validator)` | No | For regional NAT gateways only: The Availability Zone where this specific NAT gateway configuration will be active. Each AZ in a regional NAT gateway has its own configuration to handle outbound NAT traffic from that AZ. A regional NAT gateway is a single NAT Gateway that works across multiple availability zones (AZs) in your VPC, providing redundancy, scalability and availability across all the AZs in a Region. |
| `availability_zone_id` | `aws.AvailabilityZone.ZoneId (pattern: /^[a-z]+[0-9]+-az[0-9]+$/ (Full))` | No | For regional NAT gateways only: The ID of the Availability Zone where this specific NAT gateway configuration will be active. Each AZ in a regional NAT gateway has its own configuration to handle outbound NAT traffic from that AZ. Use this instead of AvailabilityZone for consistent identification of AZs across AWS Regions. A regional NAT gateway is a single NAT Gateway that works across multiple availability zones (AZs) in your VPC, providing redundancy, scalability and availability across all the AZs in a Region. |

## Attribute Reference

### `auto_provision_zones`

- **Type:** `String`



### `auto_scaling_ips`

- **Type:** `String`



### `eni_id`

- **Type:** `aws.ec2.NetworkInterface.Id (pattern: /^eni-[0-9a-f]{8,}$/ (Full))`



### `nat_gateway_id`

- **Type:** `aws.ec2.NatGateway.Id (pattern: /^nat-[0-9a-f]{8,}$/ (Full))`



### `route_table_id`

- **Type:** `aws.ec2.RouteTable.Id (pattern: /^rtb-[0-9a-f]{8,}$/ (Full))`



## AWS API narrowings

These fields are narrower than or equal to CFN. Input rules combine documented AWS domains, recorded live observations, and Carina canonical spellings for stable state comparison. A canonical DSL spelling does not claim AWS rejects other spellings. Attaching a type name adds no structural constraints. Field names below use CFN spelling.

| CFN field | Relation to CFN | Evidence URL | Verbatim quote / exact observation | Checked / observed |
| --- | --- | --- | --- | --- |
| `/definitions/Tag/properties/Key` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/AWSEC2/latest/UserGuide/Using_Tags.html) | ` Maximum key length – 128 Unicode characters in UTF-8 ` | 2026-11-07 |
| `/definitions/Tag/properties/Value` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/AWSEC2/latest/APIReference/API_Tag.html) | ` The value of the tag. `<br>` Constraints: Tag values are case-sensitive and accept a maximum of 256 Unicode characters. ` | 2026-11-07 |
| `/properties/AvailabilityMode` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/AWSCloudFormation/latest/TemplateReference/aws-resource-ec2-natgateway.html#cfn-ec2-natgateway-availabilitymode) | ` Indicates whether this is a zonal (single-AZ) or regional (multi-AZ) NAT gateway. `<br>` A zonal NAT gateway is a NAT Gateway that provides redundancy and scalability within a single availability zone. A regional NAT gateway is a single NAT Gateway that works across multiple availability zones (AZs) in your VPC, providing redundancy, scalability and availability across all the AZs in a Region. `<br>` For more information, see Regional NAT gateways for automatic multi-AZ expansion in the Amazon VPC User Guide. `<br>` Allowed values: zonal \| regional ` | 2026-11-07 |
| `/properties/ConnectivityType` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/AWSCloudFormation/latest/TemplateReference/aws-resource-ec2-natgateway.html#cfn-ec2-natgateway-connectivitytype) | ` Indicates whether the NAT gateway supports public or private connectivity. The default is public connectivity. `<br>` Allowed values: private \| public ` | 2026-11-07 |
| `/properties/SecondaryPrivateIpAddressCount` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/AWSEC2/latest/APIReference/API_CreateNatGateway.html) | ` [Private NAT gateway only] The number of secondary private IPv4 addresses you want to assign to the NAT gateway. For more information about secondary addresses, see Create a NAT gateway in the Amazon VPC User Guide. `<br>` Valid Range: Minimum value of 1. Maximum value of 31. ` | 2026-11-07 |

