---
title: "awscc.ec2.Route"
---


CloudFormation Type: `AWS::EC2::Route`

Specifies a route in a route table. For more information, see [Routes](https://docs.aws.amazon.com/vpc/latest/userguide/VPC_Route_Tables.html#route-table-routes) in the *Amazon VPC User Guide*.
 You must specify either a destination CIDR block or prefix list ID. You must also specify exactly one of the resources as the target.
 If you create a route that references a transit gateway in the same template where you create the transit gateway, you must declare a dependency on the transit gateway attachment. The route table cannot use the transit gateway until it has successfully attached to the VPC. Add a [DependsOn Attribute](https://docs.aws.amazon.com/AWSCloudFormation/latest/UserGuide/aws-attribute-dependson.html) in the ``AWS::EC2::Route`` resource to explicitly declare a dependency on the ``AWS::EC2::TransitGatewayAttachment`` resource.

## Example

```crn
let vpc = awscc.ec2.Vpc {
  cidr_block           = '10.0.0.0/16'
  enable_dns_support   = true
  enable_dns_hostnames = true
}

let igw = awscc.ec2.InternetGateway {}

let igw_attachment = awscc.ec2.VpcGatewayAttachment {
  vpc_id              = vpc.vpc_id
  internet_gateway_id = igw.internet_gateway_id
}

let rt = awscc.ec2.RouteTable {
  vpc_id = vpc.vpc_id
}

awscc.ec2.Route {
  route_table_id         = rt.route_table_id
  destination_cidr_block = '0.0.0.0/0'
  gateway_id             = igw_attachment.internet_gateway_id
}
```

## Argument Reference

### `carrier_gateway_id`

- **Type:** `aws.ec2.CarrierGateway.Id (pattern: /^cagw-[0-9a-f]{8,}$/ (Full))`
- **Required:** No

The ID of the carrier gateway. You can only use this option when the VPC contains a subnet which is associated with a Wavelength Zone.

### `core_network_arn`

- **Type:** `aws.Arn (pattern: /^arn:(?:aws|aws-cn|aws-us-gov|aws-iso|aws-iso-b|aws-iso-e|aws-iso-f|aws-eusc):[^:]+:.*$/ (Full), custom validator)`
- **Required:** No

The Amazon Resource Name (ARN) of the core network.

### `destination_cidr_block`

- **Type:** `Ipv4Cidr (custom validator)`
- **Required:** No
- **Create-only:** Yes

The IPv4 CIDR address block used for the destination match. Routing decisions are based on the most specific match. We modify the specified CIDR block to its canonical form; for example, if you specify ``100.68.0.18/18``, we modify it to ``100.68.0.0/18``.

### `destination_ipv6_cidr_block`

- **Type:** `Ipv6Cidr (custom validator)`
- **Required:** No
- **Create-only:** Yes

The IPv6 CIDR block used for the destination match. Routing decisions are based on the most specific match.

### `destination_prefix_list_id`

- **Type:** `aws.ec2.PrefixList.Id (pattern: /^pl-[0-9a-f]{8,}$/ (Full))`
- **Required:** No
- **Create-only:** Yes

The ID of a prefix list used for the destination match.

### `egress_only_internet_gateway_id`

- **Type:** `aws.ec2.EgressOnlyInternetGateway.Id (pattern: /^eigw-[0-9a-f]{8,}$/ (Full))`
- **Required:** No

[IPv6 traffic only] The ID of an egress-only internet gateway.

### `gateway_id`

- **Type:** `String`
- **Required:** No

The ID of an internet gateway or virtual private gateway attached to your VPC.

### `instance_id`

- **Type:** `aws.ec2.Instance.Id (pattern: /^i-[0-9a-f]{8,}$/ (Full))`
- **Required:** No

The ID of a NAT instance in your VPC. The operation fails if you specify an instance ID unless exactly one network interface is attached.

### `local_gateway_id`

- **Type:** `aws.ec2.LocalGateway.Id (pattern: /^lgw-[0-9a-f]{8,}$/ (Full))`
- **Required:** No

The ID of the local gateway.

### `nat_gateway_id`

- **Type:** `aws.ec2.NatGateway.Id (pattern: /^nat-[0-9a-f]{8,}$/ (Full))`
- **Required:** No

[IPv4 traffic only] The ID of a NAT gateway.

### `network_interface_id`

- **Type:** `aws.ec2.NetworkInterface.Id (pattern: /^eni-[0-9a-f]{8,}$/ (Full))`
- **Required:** No

The ID of a network interface.

### `route_table_id`

- **Type:** `aws.ec2.RouteTable.Id (pattern: /^rtb-[0-9a-f]{8,}$/ (Full))`
- **Required:** Yes
- **Create-only:** Yes

The ID of the route table for the route.

### `transit_gateway_id`

- **Type:** `aws.ec2.TransitGateway.Id (pattern: /^tgw-[0-9a-f]{8,}$/ (Full))`
- **Required:** No

The ID of a transit gateway.

### `vpc_endpoint_id`

- **Type:** `aws.ec2.VpcEndpoint.Id (pattern: /^vpce-[0-9a-f]{8,}$/ (Full))`
- **Required:** No

The ID of a VPC endpoint. Supported for Gateway Load Balancer endpoints only.

### `vpc_peering_connection_id`

- **Type:** `aws.ec2.VpcPeeringConnection.Id (pattern: /^pcx-[0-9a-f]{8,}$/ (Full))`
- **Required:** No

The ID of a VPC peering connection.

## Attribute Reference

### `cidr_block`

- **Type:** `Ipv4Cidr (custom validator)`



