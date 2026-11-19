---
title: "awscc.ec2.SecurityGroupIngress"
---


CloudFormation Type: `AWS::EC2::SecurityGroupIngress`

Resource Type definition for AWS::EC2::SecurityGroupIngress

## Example

```crn
let vpc = awscc.ec2.Vpc {
  cidr_block = '10.0.0.0/16'
}

let sg = awscc.ec2.SecurityGroup {
  vpc_id            = vpc.vpc_id
  group_description = 'Example security group'
}

awscc.ec2.SecurityGroupIngress {
  group_id    = sg.group_id
  description = 'Allow HTTPS from VPC'
  ip_protocol = tcp
  from_port   = 443
  to_port     = 443
  cidr_ip     = '10.0.0.0/16'
}
```

## Argument Reference

### `cidr_ip`

- **Type:** `Ipv4Cidr (custom validator)`
- **Required:** No
- **Create-only:** Yes

The IPv4 ranges

### `cidr_ipv6`

- **Type:** `Ipv6Cidr (custom validator)`
- **Required:** No
- **Create-only:** Yes

[VPC only] The IPv6 ranges

### `description`

- **Type:** `String (pattern: /^[a-zA-Z0-9. _:/()#,@\[\]+=&;{}!$*-]*$/ (Full), len: unbounded..=255)`
- **Required:** No

Updates the description of an ingress (inbound) security group rule. You can replace an existing description, or add a description to a rule that did not have one previously

### `from_port`

- **Type:** `Int (range: -1..=65535)`
- **Required:** No
- **Create-only:** Yes

The start of port range for the TCP and UDP protocols, or an ICMP/ICMPv6 type number. A value of -1 indicates all ICMP/ICMPv6 types. If you specify all ICMP/ICMPv6 types, you must specify all codes. Use this for ICMP and any protocol that uses ports.

### `group_id`

- **Type:** `awscc.ec2.SecurityGroup.Id (pattern: /^sg-[0-9a-f]{8,}$/ (Full))`
- **Required:** No
- **Create-only:** Yes

The ID of the security group. You must specify either the security group ID or the security group name in the request. For security groups in a nondefault VPC, you must specify the security group ID. You must specify the GroupName property or the GroupId property. For security groups that are in a VPC, you must use the GroupId property.

### `group_name`

- **Type:** `String`
- **Required:** No
- **Create-only:** Yes

The name of the security group.

### `ip_protocol`

- **Type:** [IpProtocol](#ip_protocol-ipprotocol)
- **Required:** Yes
- **Create-only:** Yes

The IP protocol name (tcp, udp, icmp, icmpv6) or number (see Protocol Numbers). [VPC only] Use -1 to specify all protocols. When authorizing security group rules, specifying -1 or a protocol number other than tcp, udp, icmp, or icmpv6 allows traffic on all ports, regardless of any port range you specify. For tcp, udp, and icmp, you must specify a port range. For icmpv6, the port range is optional; if you omit the port range, traffic for all types and codes is allowed.

### `source_prefix_list_id`

- **Type:** `awscc.ec2.PrefixList.Id (pattern: /^pl-[0-9a-f]{8,}$/ (Full))`
- **Required:** No
- **Create-only:** Yes

[EC2-VPC only] The ID of a prefix list. 

### `source_security_group_id`

- **Type:** `awscc.ec2.SecurityGroup.Id (pattern: /^sg-[0-9a-f]{8,}$/ (Full))`
- **Required:** No
- **Create-only:** Yes

The ID of the security group. You must specify either the security group ID or the security group name. For security groups in a nondefault VPC, you must specify the security group ID.

### `source_security_group_name`

- **Type:** `String`
- **Required:** No
- **Create-only:** Yes

[EC2-Classic, default VPC] The name of the source security group. You must specify the GroupName property or the GroupId property. For security groups that are in a VPC, you must use the GroupId property.

### `source_security_group_owner_id`

- **Type:** `awscc.AccountId (pattern: /^[0-9]{12}$/ (Full), len: 12..=12)`
- **Required:** No
- **Create-only:** Yes

[nondefault VPC] The AWS account ID that owns the source security group. You can't specify this property with an IP address range. If you specify SourceSecurityGroupName or SourceSecurityGroupId and that security group is owned by a different account than the account creating the stack, you must specify the SourceSecurityGroupOwnerId; otherwise, this property is optional.

### `to_port`

- **Type:** `Int (range: -1..=65535)`
- **Required:** No
- **Create-only:** Yes

The end of port range for the TCP and UDP protocols, or an ICMP/ICMPv6 code. A value of -1 indicates all ICMP/ICMPv6 codes for the specified ICMP type. If you specify all ICMP/ICMPv6 types, you must specify all codes. Use this for ICMP and any protocol that uses ports.

## Enum Values

### ip_protocol (IpProtocol)

| Value | DSL Identifier |
|-------|----------------|
| `tcp` | `awscc.ec2.SecurityGroupIngress.IpProtocol.tcp` |
| `udp` | `awscc.ec2.SecurityGroupIngress.IpProtocol.udp` |
| `icmp` | `awscc.ec2.SecurityGroupIngress.IpProtocol.icmp` |
| `icmpv6` | `awscc.ec2.SecurityGroupIngress.IpProtocol.icmpv6` |
| `-1` | `awscc.ec2.SecurityGroupIngress.IpProtocol.all` |
| `0` | `awscc.ec2.SecurityGroupIngress.IpProtocol.0` |
| `1` | `awscc.ec2.SecurityGroupIngress.IpProtocol.1` |
| `2` | `awscc.ec2.SecurityGroupIngress.IpProtocol.2` |
| `3` | `awscc.ec2.SecurityGroupIngress.IpProtocol.3` |
| `4` | `awscc.ec2.SecurityGroupIngress.IpProtocol.4` |
| `5` | `awscc.ec2.SecurityGroupIngress.IpProtocol.5` |
| `6` | `awscc.ec2.SecurityGroupIngress.IpProtocol.6` |
| `7` | `awscc.ec2.SecurityGroupIngress.IpProtocol.7` |
| `8` | `awscc.ec2.SecurityGroupIngress.IpProtocol.8` |
| `9` | `awscc.ec2.SecurityGroupIngress.IpProtocol.9` |
| `10` | `awscc.ec2.SecurityGroupIngress.IpProtocol.10` |
| `11` | `awscc.ec2.SecurityGroupIngress.IpProtocol.11` |
| `12` | `awscc.ec2.SecurityGroupIngress.IpProtocol.12` |
| `13` | `awscc.ec2.SecurityGroupIngress.IpProtocol.13` |
| `14` | `awscc.ec2.SecurityGroupIngress.IpProtocol.14` |
| `15` | `awscc.ec2.SecurityGroupIngress.IpProtocol.15` |
| `16` | `awscc.ec2.SecurityGroupIngress.IpProtocol.16` |
| `17` | `awscc.ec2.SecurityGroupIngress.IpProtocol.17` |
| `18` | `awscc.ec2.SecurityGroupIngress.IpProtocol.18` |
| `19` | `awscc.ec2.SecurityGroupIngress.IpProtocol.19` |
| `20` | `awscc.ec2.SecurityGroupIngress.IpProtocol.20` |
| `21` | `awscc.ec2.SecurityGroupIngress.IpProtocol.21` |
| `22` | `awscc.ec2.SecurityGroupIngress.IpProtocol.22` |
| `23` | `awscc.ec2.SecurityGroupIngress.IpProtocol.23` |
| `24` | `awscc.ec2.SecurityGroupIngress.IpProtocol.24` |
| `25` | `awscc.ec2.SecurityGroupIngress.IpProtocol.25` |
| `26` | `awscc.ec2.SecurityGroupIngress.IpProtocol.26` |
| `27` | `awscc.ec2.SecurityGroupIngress.IpProtocol.27` |
| `28` | `awscc.ec2.SecurityGroupIngress.IpProtocol.28` |
| `29` | `awscc.ec2.SecurityGroupIngress.IpProtocol.29` |
| `30` | `awscc.ec2.SecurityGroupIngress.IpProtocol.30` |
| `31` | `awscc.ec2.SecurityGroupIngress.IpProtocol.31` |
| `32` | `awscc.ec2.SecurityGroupIngress.IpProtocol.32` |
| `33` | `awscc.ec2.SecurityGroupIngress.IpProtocol.33` |
| `34` | `awscc.ec2.SecurityGroupIngress.IpProtocol.34` |
| `35` | `awscc.ec2.SecurityGroupIngress.IpProtocol.35` |
| `36` | `awscc.ec2.SecurityGroupIngress.IpProtocol.36` |
| `37` | `awscc.ec2.SecurityGroupIngress.IpProtocol.37` |
| `38` | `awscc.ec2.SecurityGroupIngress.IpProtocol.38` |
| `39` | `awscc.ec2.SecurityGroupIngress.IpProtocol.39` |
| `40` | `awscc.ec2.SecurityGroupIngress.IpProtocol.40` |
| `41` | `awscc.ec2.SecurityGroupIngress.IpProtocol.41` |
| `42` | `awscc.ec2.SecurityGroupIngress.IpProtocol.42` |
| `43` | `awscc.ec2.SecurityGroupIngress.IpProtocol.43` |
| `44` | `awscc.ec2.SecurityGroupIngress.IpProtocol.44` |
| `45` | `awscc.ec2.SecurityGroupIngress.IpProtocol.45` |
| `46` | `awscc.ec2.SecurityGroupIngress.IpProtocol.46` |
| `47` | `awscc.ec2.SecurityGroupIngress.IpProtocol.47` |
| `48` | `awscc.ec2.SecurityGroupIngress.IpProtocol.48` |
| `49` | `awscc.ec2.SecurityGroupIngress.IpProtocol.49` |
| `50` | `awscc.ec2.SecurityGroupIngress.IpProtocol.50` |
| `51` | `awscc.ec2.SecurityGroupIngress.IpProtocol.51` |
| `52` | `awscc.ec2.SecurityGroupIngress.IpProtocol.52` |
| `53` | `awscc.ec2.SecurityGroupIngress.IpProtocol.53` |
| `54` | `awscc.ec2.SecurityGroupIngress.IpProtocol.54` |
| `55` | `awscc.ec2.SecurityGroupIngress.IpProtocol.55` |
| `56` | `awscc.ec2.SecurityGroupIngress.IpProtocol.56` |
| `57` | `awscc.ec2.SecurityGroupIngress.IpProtocol.57` |
| `58` | `awscc.ec2.SecurityGroupIngress.IpProtocol.58` |
| `59` | `awscc.ec2.SecurityGroupIngress.IpProtocol.59` |
| `60` | `awscc.ec2.SecurityGroupIngress.IpProtocol.60` |
| `61` | `awscc.ec2.SecurityGroupIngress.IpProtocol.61` |
| `62` | `awscc.ec2.SecurityGroupIngress.IpProtocol.62` |
| `63` | `awscc.ec2.SecurityGroupIngress.IpProtocol.63` |
| `64` | `awscc.ec2.SecurityGroupIngress.IpProtocol.64` |
| `65` | `awscc.ec2.SecurityGroupIngress.IpProtocol.65` |
| `66` | `awscc.ec2.SecurityGroupIngress.IpProtocol.66` |
| `67` | `awscc.ec2.SecurityGroupIngress.IpProtocol.67` |
| `68` | `awscc.ec2.SecurityGroupIngress.IpProtocol.68` |
| `69` | `awscc.ec2.SecurityGroupIngress.IpProtocol.69` |
| `70` | `awscc.ec2.SecurityGroupIngress.IpProtocol.70` |
| `71` | `awscc.ec2.SecurityGroupIngress.IpProtocol.71` |
| `72` | `awscc.ec2.SecurityGroupIngress.IpProtocol.72` |
| `73` | `awscc.ec2.SecurityGroupIngress.IpProtocol.73` |
| `74` | `awscc.ec2.SecurityGroupIngress.IpProtocol.74` |
| `75` | `awscc.ec2.SecurityGroupIngress.IpProtocol.75` |
| `76` | `awscc.ec2.SecurityGroupIngress.IpProtocol.76` |
| `77` | `awscc.ec2.SecurityGroupIngress.IpProtocol.77` |
| `78` | `awscc.ec2.SecurityGroupIngress.IpProtocol.78` |
| `79` | `awscc.ec2.SecurityGroupIngress.IpProtocol.79` |
| `80` | `awscc.ec2.SecurityGroupIngress.IpProtocol.80` |
| `81` | `awscc.ec2.SecurityGroupIngress.IpProtocol.81` |
| `82` | `awscc.ec2.SecurityGroupIngress.IpProtocol.82` |
| `83` | `awscc.ec2.SecurityGroupIngress.IpProtocol.83` |
| `84` | `awscc.ec2.SecurityGroupIngress.IpProtocol.84` |
| `85` | `awscc.ec2.SecurityGroupIngress.IpProtocol.85` |
| `86` | `awscc.ec2.SecurityGroupIngress.IpProtocol.86` |
| `87` | `awscc.ec2.SecurityGroupIngress.IpProtocol.87` |
| `88` | `awscc.ec2.SecurityGroupIngress.IpProtocol.88` |
| `89` | `awscc.ec2.SecurityGroupIngress.IpProtocol.89` |
| `90` | `awscc.ec2.SecurityGroupIngress.IpProtocol.90` |
| `91` | `awscc.ec2.SecurityGroupIngress.IpProtocol.91` |
| `92` | `awscc.ec2.SecurityGroupIngress.IpProtocol.92` |
| `93` | `awscc.ec2.SecurityGroupIngress.IpProtocol.93` |
| `94` | `awscc.ec2.SecurityGroupIngress.IpProtocol.94` |
| `95` | `awscc.ec2.SecurityGroupIngress.IpProtocol.95` |
| `96` | `awscc.ec2.SecurityGroupIngress.IpProtocol.96` |
| `97` | `awscc.ec2.SecurityGroupIngress.IpProtocol.97` |
| `98` | `awscc.ec2.SecurityGroupIngress.IpProtocol.98` |
| `99` | `awscc.ec2.SecurityGroupIngress.IpProtocol.99` |
| `100` | `awscc.ec2.SecurityGroupIngress.IpProtocol.100` |
| `101` | `awscc.ec2.SecurityGroupIngress.IpProtocol.101` |
| `102` | `awscc.ec2.SecurityGroupIngress.IpProtocol.102` |
| `103` | `awscc.ec2.SecurityGroupIngress.IpProtocol.103` |
| `104` | `awscc.ec2.SecurityGroupIngress.IpProtocol.104` |
| `105` | `awscc.ec2.SecurityGroupIngress.IpProtocol.105` |
| `106` | `awscc.ec2.SecurityGroupIngress.IpProtocol.106` |
| `107` | `awscc.ec2.SecurityGroupIngress.IpProtocol.107` |
| `108` | `awscc.ec2.SecurityGroupIngress.IpProtocol.108` |
| `109` | `awscc.ec2.SecurityGroupIngress.IpProtocol.109` |
| `110` | `awscc.ec2.SecurityGroupIngress.IpProtocol.110` |
| `111` | `awscc.ec2.SecurityGroupIngress.IpProtocol.111` |
| `112` | `awscc.ec2.SecurityGroupIngress.IpProtocol.112` |
| `113` | `awscc.ec2.SecurityGroupIngress.IpProtocol.113` |
| `114` | `awscc.ec2.SecurityGroupIngress.IpProtocol.114` |
| `115` | `awscc.ec2.SecurityGroupIngress.IpProtocol.115` |
| `116` | `awscc.ec2.SecurityGroupIngress.IpProtocol.116` |
| `117` | `awscc.ec2.SecurityGroupIngress.IpProtocol.117` |
| `118` | `awscc.ec2.SecurityGroupIngress.IpProtocol.118` |
| `119` | `awscc.ec2.SecurityGroupIngress.IpProtocol.119` |
| `120` | `awscc.ec2.SecurityGroupIngress.IpProtocol.120` |
| `121` | `awscc.ec2.SecurityGroupIngress.IpProtocol.121` |
| `122` | `awscc.ec2.SecurityGroupIngress.IpProtocol.122` |
| `123` | `awscc.ec2.SecurityGroupIngress.IpProtocol.123` |
| `124` | `awscc.ec2.SecurityGroupIngress.IpProtocol.124` |
| `125` | `awscc.ec2.SecurityGroupIngress.IpProtocol.125` |
| `126` | `awscc.ec2.SecurityGroupIngress.IpProtocol.126` |
| `127` | `awscc.ec2.SecurityGroupIngress.IpProtocol.127` |
| `128` | `awscc.ec2.SecurityGroupIngress.IpProtocol.128` |
| `129` | `awscc.ec2.SecurityGroupIngress.IpProtocol.129` |
| `130` | `awscc.ec2.SecurityGroupIngress.IpProtocol.130` |
| `131` | `awscc.ec2.SecurityGroupIngress.IpProtocol.131` |
| `132` | `awscc.ec2.SecurityGroupIngress.IpProtocol.132` |
| `133` | `awscc.ec2.SecurityGroupIngress.IpProtocol.133` |
| `134` | `awscc.ec2.SecurityGroupIngress.IpProtocol.134` |
| `135` | `awscc.ec2.SecurityGroupIngress.IpProtocol.135` |
| `136` | `awscc.ec2.SecurityGroupIngress.IpProtocol.136` |
| `137` | `awscc.ec2.SecurityGroupIngress.IpProtocol.137` |
| `138` | `awscc.ec2.SecurityGroupIngress.IpProtocol.138` |
| `139` | `awscc.ec2.SecurityGroupIngress.IpProtocol.139` |
| `140` | `awscc.ec2.SecurityGroupIngress.IpProtocol.140` |
| `141` | `awscc.ec2.SecurityGroupIngress.IpProtocol.141` |
| `142` | `awscc.ec2.SecurityGroupIngress.IpProtocol.142` |
| `143` | `awscc.ec2.SecurityGroupIngress.IpProtocol.143` |
| `144` | `awscc.ec2.SecurityGroupIngress.IpProtocol.144` |
| `145` | `awscc.ec2.SecurityGroupIngress.IpProtocol.145` |
| `146` | `awscc.ec2.SecurityGroupIngress.IpProtocol.146` |
| `147` | `awscc.ec2.SecurityGroupIngress.IpProtocol.147` |
| `148` | `awscc.ec2.SecurityGroupIngress.IpProtocol.148` |
| `149` | `awscc.ec2.SecurityGroupIngress.IpProtocol.149` |
| `150` | `awscc.ec2.SecurityGroupIngress.IpProtocol.150` |
| `151` | `awscc.ec2.SecurityGroupIngress.IpProtocol.151` |
| `152` | `awscc.ec2.SecurityGroupIngress.IpProtocol.152` |
| `153` | `awscc.ec2.SecurityGroupIngress.IpProtocol.153` |
| `154` | `awscc.ec2.SecurityGroupIngress.IpProtocol.154` |
| `155` | `awscc.ec2.SecurityGroupIngress.IpProtocol.155` |
| `156` | `awscc.ec2.SecurityGroupIngress.IpProtocol.156` |
| `157` | `awscc.ec2.SecurityGroupIngress.IpProtocol.157` |
| `158` | `awscc.ec2.SecurityGroupIngress.IpProtocol.158` |
| `159` | `awscc.ec2.SecurityGroupIngress.IpProtocol.159` |
| `160` | `awscc.ec2.SecurityGroupIngress.IpProtocol.160` |
| `161` | `awscc.ec2.SecurityGroupIngress.IpProtocol.161` |
| `162` | `awscc.ec2.SecurityGroupIngress.IpProtocol.162` |
| `163` | `awscc.ec2.SecurityGroupIngress.IpProtocol.163` |
| `164` | `awscc.ec2.SecurityGroupIngress.IpProtocol.164` |
| `165` | `awscc.ec2.SecurityGroupIngress.IpProtocol.165` |
| `166` | `awscc.ec2.SecurityGroupIngress.IpProtocol.166` |
| `167` | `awscc.ec2.SecurityGroupIngress.IpProtocol.167` |
| `168` | `awscc.ec2.SecurityGroupIngress.IpProtocol.168` |
| `169` | `awscc.ec2.SecurityGroupIngress.IpProtocol.169` |
| `170` | `awscc.ec2.SecurityGroupIngress.IpProtocol.170` |
| `171` | `awscc.ec2.SecurityGroupIngress.IpProtocol.171` |
| `172` | `awscc.ec2.SecurityGroupIngress.IpProtocol.172` |
| `173` | `awscc.ec2.SecurityGroupIngress.IpProtocol.173` |
| `174` | `awscc.ec2.SecurityGroupIngress.IpProtocol.174` |
| `175` | `awscc.ec2.SecurityGroupIngress.IpProtocol.175` |
| `176` | `awscc.ec2.SecurityGroupIngress.IpProtocol.176` |
| `177` | `awscc.ec2.SecurityGroupIngress.IpProtocol.177` |
| `178` | `awscc.ec2.SecurityGroupIngress.IpProtocol.178` |
| `179` | `awscc.ec2.SecurityGroupIngress.IpProtocol.179` |
| `180` | `awscc.ec2.SecurityGroupIngress.IpProtocol.180` |
| `181` | `awscc.ec2.SecurityGroupIngress.IpProtocol.181` |
| `182` | `awscc.ec2.SecurityGroupIngress.IpProtocol.182` |
| `183` | `awscc.ec2.SecurityGroupIngress.IpProtocol.183` |
| `184` | `awscc.ec2.SecurityGroupIngress.IpProtocol.184` |
| `185` | `awscc.ec2.SecurityGroupIngress.IpProtocol.185` |
| `186` | `awscc.ec2.SecurityGroupIngress.IpProtocol.186` |
| `187` | `awscc.ec2.SecurityGroupIngress.IpProtocol.187` |
| `188` | `awscc.ec2.SecurityGroupIngress.IpProtocol.188` |
| `189` | `awscc.ec2.SecurityGroupIngress.IpProtocol.189` |
| `190` | `awscc.ec2.SecurityGroupIngress.IpProtocol.190` |
| `191` | `awscc.ec2.SecurityGroupIngress.IpProtocol.191` |
| `192` | `awscc.ec2.SecurityGroupIngress.IpProtocol.192` |
| `193` | `awscc.ec2.SecurityGroupIngress.IpProtocol.193` |
| `194` | `awscc.ec2.SecurityGroupIngress.IpProtocol.194` |
| `195` | `awscc.ec2.SecurityGroupIngress.IpProtocol.195` |
| `196` | `awscc.ec2.SecurityGroupIngress.IpProtocol.196` |
| `197` | `awscc.ec2.SecurityGroupIngress.IpProtocol.197` |
| `198` | `awscc.ec2.SecurityGroupIngress.IpProtocol.198` |
| `199` | `awscc.ec2.SecurityGroupIngress.IpProtocol.199` |
| `200` | `awscc.ec2.SecurityGroupIngress.IpProtocol.200` |
| `201` | `awscc.ec2.SecurityGroupIngress.IpProtocol.201` |
| `202` | `awscc.ec2.SecurityGroupIngress.IpProtocol.202` |
| `203` | `awscc.ec2.SecurityGroupIngress.IpProtocol.203` |
| `204` | `awscc.ec2.SecurityGroupIngress.IpProtocol.204` |
| `205` | `awscc.ec2.SecurityGroupIngress.IpProtocol.205` |
| `206` | `awscc.ec2.SecurityGroupIngress.IpProtocol.206` |
| `207` | `awscc.ec2.SecurityGroupIngress.IpProtocol.207` |
| `208` | `awscc.ec2.SecurityGroupIngress.IpProtocol.208` |
| `209` | `awscc.ec2.SecurityGroupIngress.IpProtocol.209` |
| `210` | `awscc.ec2.SecurityGroupIngress.IpProtocol.210` |
| `211` | `awscc.ec2.SecurityGroupIngress.IpProtocol.211` |
| `212` | `awscc.ec2.SecurityGroupIngress.IpProtocol.212` |
| `213` | `awscc.ec2.SecurityGroupIngress.IpProtocol.213` |
| `214` | `awscc.ec2.SecurityGroupIngress.IpProtocol.214` |
| `215` | `awscc.ec2.SecurityGroupIngress.IpProtocol.215` |
| `216` | `awscc.ec2.SecurityGroupIngress.IpProtocol.216` |
| `217` | `awscc.ec2.SecurityGroupIngress.IpProtocol.217` |
| `218` | `awscc.ec2.SecurityGroupIngress.IpProtocol.218` |
| `219` | `awscc.ec2.SecurityGroupIngress.IpProtocol.219` |
| `220` | `awscc.ec2.SecurityGroupIngress.IpProtocol.220` |
| `221` | `awscc.ec2.SecurityGroupIngress.IpProtocol.221` |
| `222` | `awscc.ec2.SecurityGroupIngress.IpProtocol.222` |
| `223` | `awscc.ec2.SecurityGroupIngress.IpProtocol.223` |
| `224` | `awscc.ec2.SecurityGroupIngress.IpProtocol.224` |
| `225` | `awscc.ec2.SecurityGroupIngress.IpProtocol.225` |
| `226` | `awscc.ec2.SecurityGroupIngress.IpProtocol.226` |
| `227` | `awscc.ec2.SecurityGroupIngress.IpProtocol.227` |
| `228` | `awscc.ec2.SecurityGroupIngress.IpProtocol.228` |
| `229` | `awscc.ec2.SecurityGroupIngress.IpProtocol.229` |
| `230` | `awscc.ec2.SecurityGroupIngress.IpProtocol.230` |
| `231` | `awscc.ec2.SecurityGroupIngress.IpProtocol.231` |
| `232` | `awscc.ec2.SecurityGroupIngress.IpProtocol.232` |
| `233` | `awscc.ec2.SecurityGroupIngress.IpProtocol.233` |
| `234` | `awscc.ec2.SecurityGroupIngress.IpProtocol.234` |
| `235` | `awscc.ec2.SecurityGroupIngress.IpProtocol.235` |
| `236` | `awscc.ec2.SecurityGroupIngress.IpProtocol.236` |
| `237` | `awscc.ec2.SecurityGroupIngress.IpProtocol.237` |
| `238` | `awscc.ec2.SecurityGroupIngress.IpProtocol.238` |
| `239` | `awscc.ec2.SecurityGroupIngress.IpProtocol.239` |
| `240` | `awscc.ec2.SecurityGroupIngress.IpProtocol.240` |
| `241` | `awscc.ec2.SecurityGroupIngress.IpProtocol.241` |
| `242` | `awscc.ec2.SecurityGroupIngress.IpProtocol.242` |
| `243` | `awscc.ec2.SecurityGroupIngress.IpProtocol.243` |
| `244` | `awscc.ec2.SecurityGroupIngress.IpProtocol.244` |
| `245` | `awscc.ec2.SecurityGroupIngress.IpProtocol.245` |
| `246` | `awscc.ec2.SecurityGroupIngress.IpProtocol.246` |
| `247` | `awscc.ec2.SecurityGroupIngress.IpProtocol.247` |
| `248` | `awscc.ec2.SecurityGroupIngress.IpProtocol.248` |
| `249` | `awscc.ec2.SecurityGroupIngress.IpProtocol.249` |
| `250` | `awscc.ec2.SecurityGroupIngress.IpProtocol.250` |
| `251` | `awscc.ec2.SecurityGroupIngress.IpProtocol.251` |
| `252` | `awscc.ec2.SecurityGroupIngress.IpProtocol.252` |
| `253` | `awscc.ec2.SecurityGroupIngress.IpProtocol.253` |
| `254` | `awscc.ec2.SecurityGroupIngress.IpProtocol.254` |
| `255` | `awscc.ec2.SecurityGroupIngress.IpProtocol.255` |

Shorthand formats: `tcp` or `IpProtocol.tcp`

## Attribute Reference

### `id`

- **Type:** `String`

The Security Group Rule Id

## AWS API narrowings

These fields are narrower than or equal to CFN. Input rules combine documented AWS domains, recorded live observations, and Carina canonical spellings for stable state comparison. A canonical DSL spelling does not claim AWS rejects other spellings. Attaching a type name adds no structural constraints. Field names below use CFN spelling.

| CFN field | Relation to CFN | Evidence URL | Verbatim quote / exact observation | Checked / observed |
| --- | --- | --- | --- | --- |
| `/properties/Description` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/AWSCloudFormation/latest/TemplateReference/aws-resource-ec2-securitygroupingress.html#cfn-ec2-securitygroupingress-description) | ` Updates the description of an ingress (inbound) security group rule. You can replace an existing description, or add a description to a rule that did not have one previously. `<br>` Constraints: Up to 255 characters in length. Allowed characters are a-z, A-Z, 0-9, spaces, and ._-:/()#,@[]+=;{}!$* ` | 2026-11-07 |
| `/properties/FromPort` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/managedservices/latest/ctref/schemas.html) | ` The start of allowed port range, from 0 to 65535 for TCP/UDP. For ICMP, use -1. ` | 2026-11-07 |
| `/properties/FromPort` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/AWSEC2/latest/APIReference/API_IpPermission.html) | ` If the protocol is ICMP or ICMPv6, this is the ICMP type or -1 (all ICMP types). ` | 2026-11-07 |
| `/properties/IpProtocol` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/AWSEC2/latest/APIReference/API_IpPermission.html) | ` The IP protocol name (tcp, udp, icmp, icmpv6) or number (see Protocol Numbers). `<br>` Use -1 to specify all protocols. ` | 2026-11-07 |
| `/properties/IpProtocol` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/network-firewall/latest/APIReference/API_MatchAttributes.html#networkfirewall-Type-MatchAttributes-Protocols) | ` The protocols to inspect for, specified using the assigned internet protocol number (IANA) for each protocol. `<br>` Valid Range: Minimum value of 0. Maximum value of 255. ` | 2026-11-07 |
| `/properties/ToPort` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/managedservices/latest/ctref/schemas.html) | ` The end of allowed port range, from 0 to 65535 for TCP/UDP. For ICMP, use -1. ` | 2026-11-07 |
| `/properties/ToPort` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/AWSEC2/latest/APIReference/API_IpPermission.html) | ` If the protocol is ICMP or ICMPv6, this is the ICMP type or -1 (all ICMP types). ` | 2026-11-07 |

