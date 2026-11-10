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

- **Type:** Ipv4Cidr — identity: String; member: String
- **Required:** No
- **Create-only:** Yes

The IPv4 ranges

### `cidr_ipv6`

- **Type:** Ipv6Cidr — identity: String; member: String
- **Required:** No
- **Create-only:** Yes

[VPC only] The IPv6 ranges

### `description`

- **Type:** `String(pattern, len: ..=255)`
- **Required:** No

Updates the description of an ingress (inbound) security group rule. You can replace an existing description, or add a description to a rule that did not have one previously

### `from_port`

- **Type:** Int(-1..=65535)
- **Required:** No
- **Create-only:** Yes

The start of port range for the TCP and UDP protocols, or an ICMP/ICMPv6 type number. A value of -1 indicates all ICMP/ICMPv6 types. If you specify all ICMP/ICMPv6 types, you must specify all codes. Use this for ICMP and any protocol that uses ports.

### `group_id`

- **Type:** aws.ec2.SecurityGroup.Id — identity: String(pattern); member: String
- **Required:** No
- **Create-only:** Yes

The ID of the security group. You must specify either the security group ID or the security group name in the request. For security groups in a nondefault VPC, you must specify the security group ID. You must specify the GroupName property or the GroupId property. For security groups that are in a VPC, you must use the GroupId property.

### `group_name`

- **Type:** String
- **Required:** No
- **Create-only:** Yes

The name of the security group.

### `ip_protocol`

- **Type:** [Enum (IpProtocol)](#ip_protocol-ipprotocol)
- **Required:** Yes
- **Create-only:** Yes

The IP protocol name (tcp, udp, icmp, icmpv6) or number (see Protocol Numbers). [VPC only] Use -1 to specify all protocols. When authorizing security group rules, specifying -1 or a protocol number other than tcp, udp, icmp, or icmpv6 allows traffic on all ports, regardless of any port range you specify. For tcp, udp, and icmp, you must specify a port range. For icmpv6, the port range is optional; if you omit the port range, traffic for all types and codes is allowed.

### `source_prefix_list_id`

- **Type:** aws.ec2.PrefixList.Id — identity: String(pattern); member: String
- **Required:** No
- **Create-only:** Yes

[EC2-VPC only] The ID of a prefix list. 

### `source_security_group_id`

- **Type:** aws.ec2.SecurityGroup.Id — identity: String(pattern); member: String
- **Required:** No
- **Create-only:** Yes

The ID of the security group. You must specify either the security group ID or the security group name. For security groups in a nondefault VPC, you must specify the security group ID.

### `source_security_group_name`

- **Type:** String
- **Required:** No
- **Create-only:** Yes

[EC2-Classic, default VPC] The name of the source security group. You must specify the GroupName property or the GroupId property. For security groups that are in a VPC, you must use the GroupId property.

### `source_security_group_owner_id`

- **Type:** aws.AccountId — identity: String(pattern, len: 12..=12); member: String
- **Required:** No
- **Create-only:** Yes

[nondefault VPC] The AWS account ID that owns the source security group. You can't specify this property with an IP address range. If you specify SourceSecurityGroupName or SourceSecurityGroupId and that security group is owned by a different account than the account creating the stack, you must specify the SourceSecurityGroupOwnerId; otherwise, this property is optional.

### `to_port`

- **Type:** Int(-1..=65535)
- **Required:** No
- **Create-only:** Yes

The end of port range for the TCP and UDP protocols, or an ICMP/ICMPv6 code. A value of -1 indicates all ICMP/ICMPv6 codes for the specified ICMP type. If you specify all ICMP/ICMPv6 types, you must specify all codes. Use this for ICMP and any protocol that uses ports.

## Enum Values

### ip_protocol (IpProtocol)

| Value | DSL Identifier |
|-------|----------------|
| `tcp` | `aws.ec2.SecurityGroupIngress.IpProtocol.tcp` |
| `udp` | `aws.ec2.SecurityGroupIngress.IpProtocol.udp` |
| `icmp` | `aws.ec2.SecurityGroupIngress.IpProtocol.icmp` |
| `icmpv6` | `aws.ec2.SecurityGroupIngress.IpProtocol.icmpv6` |
| `-1` | `aws.ec2.SecurityGroupIngress.IpProtocol.all` |
| `0` | `aws.ec2.SecurityGroupIngress.IpProtocol.0` |
| `1` | `aws.ec2.SecurityGroupIngress.IpProtocol.1` |
| `2` | `aws.ec2.SecurityGroupIngress.IpProtocol.2` |
| `3` | `aws.ec2.SecurityGroupIngress.IpProtocol.3` |
| `4` | `aws.ec2.SecurityGroupIngress.IpProtocol.4` |
| `5` | `aws.ec2.SecurityGroupIngress.IpProtocol.5` |
| `6` | `aws.ec2.SecurityGroupIngress.IpProtocol.6` |
| `7` | `aws.ec2.SecurityGroupIngress.IpProtocol.7` |
| `8` | `aws.ec2.SecurityGroupIngress.IpProtocol.8` |
| `9` | `aws.ec2.SecurityGroupIngress.IpProtocol.9` |
| `10` | `aws.ec2.SecurityGroupIngress.IpProtocol.10` |
| `11` | `aws.ec2.SecurityGroupIngress.IpProtocol.11` |
| `12` | `aws.ec2.SecurityGroupIngress.IpProtocol.12` |
| `13` | `aws.ec2.SecurityGroupIngress.IpProtocol.13` |
| `14` | `aws.ec2.SecurityGroupIngress.IpProtocol.14` |
| `15` | `aws.ec2.SecurityGroupIngress.IpProtocol.15` |
| `16` | `aws.ec2.SecurityGroupIngress.IpProtocol.16` |
| `17` | `aws.ec2.SecurityGroupIngress.IpProtocol.17` |
| `18` | `aws.ec2.SecurityGroupIngress.IpProtocol.18` |
| `19` | `aws.ec2.SecurityGroupIngress.IpProtocol.19` |
| `20` | `aws.ec2.SecurityGroupIngress.IpProtocol.20` |
| `21` | `aws.ec2.SecurityGroupIngress.IpProtocol.21` |
| `22` | `aws.ec2.SecurityGroupIngress.IpProtocol.22` |
| `23` | `aws.ec2.SecurityGroupIngress.IpProtocol.23` |
| `24` | `aws.ec2.SecurityGroupIngress.IpProtocol.24` |
| `25` | `aws.ec2.SecurityGroupIngress.IpProtocol.25` |
| `26` | `aws.ec2.SecurityGroupIngress.IpProtocol.26` |
| `27` | `aws.ec2.SecurityGroupIngress.IpProtocol.27` |
| `28` | `aws.ec2.SecurityGroupIngress.IpProtocol.28` |
| `29` | `aws.ec2.SecurityGroupIngress.IpProtocol.29` |
| `30` | `aws.ec2.SecurityGroupIngress.IpProtocol.30` |
| `31` | `aws.ec2.SecurityGroupIngress.IpProtocol.31` |
| `32` | `aws.ec2.SecurityGroupIngress.IpProtocol.32` |
| `33` | `aws.ec2.SecurityGroupIngress.IpProtocol.33` |
| `34` | `aws.ec2.SecurityGroupIngress.IpProtocol.34` |
| `35` | `aws.ec2.SecurityGroupIngress.IpProtocol.35` |
| `36` | `aws.ec2.SecurityGroupIngress.IpProtocol.36` |
| `37` | `aws.ec2.SecurityGroupIngress.IpProtocol.37` |
| `38` | `aws.ec2.SecurityGroupIngress.IpProtocol.38` |
| `39` | `aws.ec2.SecurityGroupIngress.IpProtocol.39` |
| `40` | `aws.ec2.SecurityGroupIngress.IpProtocol.40` |
| `41` | `aws.ec2.SecurityGroupIngress.IpProtocol.41` |
| `42` | `aws.ec2.SecurityGroupIngress.IpProtocol.42` |
| `43` | `aws.ec2.SecurityGroupIngress.IpProtocol.43` |
| `44` | `aws.ec2.SecurityGroupIngress.IpProtocol.44` |
| `45` | `aws.ec2.SecurityGroupIngress.IpProtocol.45` |
| `46` | `aws.ec2.SecurityGroupIngress.IpProtocol.46` |
| `47` | `aws.ec2.SecurityGroupIngress.IpProtocol.47` |
| `48` | `aws.ec2.SecurityGroupIngress.IpProtocol.48` |
| `49` | `aws.ec2.SecurityGroupIngress.IpProtocol.49` |
| `50` | `aws.ec2.SecurityGroupIngress.IpProtocol.50` |
| `51` | `aws.ec2.SecurityGroupIngress.IpProtocol.51` |
| `52` | `aws.ec2.SecurityGroupIngress.IpProtocol.52` |
| `53` | `aws.ec2.SecurityGroupIngress.IpProtocol.53` |
| `54` | `aws.ec2.SecurityGroupIngress.IpProtocol.54` |
| `55` | `aws.ec2.SecurityGroupIngress.IpProtocol.55` |
| `56` | `aws.ec2.SecurityGroupIngress.IpProtocol.56` |
| `57` | `aws.ec2.SecurityGroupIngress.IpProtocol.57` |
| `58` | `aws.ec2.SecurityGroupIngress.IpProtocol.58` |
| `59` | `aws.ec2.SecurityGroupIngress.IpProtocol.59` |
| `60` | `aws.ec2.SecurityGroupIngress.IpProtocol.60` |
| `61` | `aws.ec2.SecurityGroupIngress.IpProtocol.61` |
| `62` | `aws.ec2.SecurityGroupIngress.IpProtocol.62` |
| `63` | `aws.ec2.SecurityGroupIngress.IpProtocol.63` |
| `64` | `aws.ec2.SecurityGroupIngress.IpProtocol.64` |
| `65` | `aws.ec2.SecurityGroupIngress.IpProtocol.65` |
| `66` | `aws.ec2.SecurityGroupIngress.IpProtocol.66` |
| `67` | `aws.ec2.SecurityGroupIngress.IpProtocol.67` |
| `68` | `aws.ec2.SecurityGroupIngress.IpProtocol.68` |
| `69` | `aws.ec2.SecurityGroupIngress.IpProtocol.69` |
| `70` | `aws.ec2.SecurityGroupIngress.IpProtocol.70` |
| `71` | `aws.ec2.SecurityGroupIngress.IpProtocol.71` |
| `72` | `aws.ec2.SecurityGroupIngress.IpProtocol.72` |
| `73` | `aws.ec2.SecurityGroupIngress.IpProtocol.73` |
| `74` | `aws.ec2.SecurityGroupIngress.IpProtocol.74` |
| `75` | `aws.ec2.SecurityGroupIngress.IpProtocol.75` |
| `76` | `aws.ec2.SecurityGroupIngress.IpProtocol.76` |
| `77` | `aws.ec2.SecurityGroupIngress.IpProtocol.77` |
| `78` | `aws.ec2.SecurityGroupIngress.IpProtocol.78` |
| `79` | `aws.ec2.SecurityGroupIngress.IpProtocol.79` |
| `80` | `aws.ec2.SecurityGroupIngress.IpProtocol.80` |
| `81` | `aws.ec2.SecurityGroupIngress.IpProtocol.81` |
| `82` | `aws.ec2.SecurityGroupIngress.IpProtocol.82` |
| `83` | `aws.ec2.SecurityGroupIngress.IpProtocol.83` |
| `84` | `aws.ec2.SecurityGroupIngress.IpProtocol.84` |
| `85` | `aws.ec2.SecurityGroupIngress.IpProtocol.85` |
| `86` | `aws.ec2.SecurityGroupIngress.IpProtocol.86` |
| `87` | `aws.ec2.SecurityGroupIngress.IpProtocol.87` |
| `88` | `aws.ec2.SecurityGroupIngress.IpProtocol.88` |
| `89` | `aws.ec2.SecurityGroupIngress.IpProtocol.89` |
| `90` | `aws.ec2.SecurityGroupIngress.IpProtocol.90` |
| `91` | `aws.ec2.SecurityGroupIngress.IpProtocol.91` |
| `92` | `aws.ec2.SecurityGroupIngress.IpProtocol.92` |
| `93` | `aws.ec2.SecurityGroupIngress.IpProtocol.93` |
| `94` | `aws.ec2.SecurityGroupIngress.IpProtocol.94` |
| `95` | `aws.ec2.SecurityGroupIngress.IpProtocol.95` |
| `96` | `aws.ec2.SecurityGroupIngress.IpProtocol.96` |
| `97` | `aws.ec2.SecurityGroupIngress.IpProtocol.97` |
| `98` | `aws.ec2.SecurityGroupIngress.IpProtocol.98` |
| `99` | `aws.ec2.SecurityGroupIngress.IpProtocol.99` |
| `100` | `aws.ec2.SecurityGroupIngress.IpProtocol.100` |
| `101` | `aws.ec2.SecurityGroupIngress.IpProtocol.101` |
| `102` | `aws.ec2.SecurityGroupIngress.IpProtocol.102` |
| `103` | `aws.ec2.SecurityGroupIngress.IpProtocol.103` |
| `104` | `aws.ec2.SecurityGroupIngress.IpProtocol.104` |
| `105` | `aws.ec2.SecurityGroupIngress.IpProtocol.105` |
| `106` | `aws.ec2.SecurityGroupIngress.IpProtocol.106` |
| `107` | `aws.ec2.SecurityGroupIngress.IpProtocol.107` |
| `108` | `aws.ec2.SecurityGroupIngress.IpProtocol.108` |
| `109` | `aws.ec2.SecurityGroupIngress.IpProtocol.109` |
| `110` | `aws.ec2.SecurityGroupIngress.IpProtocol.110` |
| `111` | `aws.ec2.SecurityGroupIngress.IpProtocol.111` |
| `112` | `aws.ec2.SecurityGroupIngress.IpProtocol.112` |
| `113` | `aws.ec2.SecurityGroupIngress.IpProtocol.113` |
| `114` | `aws.ec2.SecurityGroupIngress.IpProtocol.114` |
| `115` | `aws.ec2.SecurityGroupIngress.IpProtocol.115` |
| `116` | `aws.ec2.SecurityGroupIngress.IpProtocol.116` |
| `117` | `aws.ec2.SecurityGroupIngress.IpProtocol.117` |
| `118` | `aws.ec2.SecurityGroupIngress.IpProtocol.118` |
| `119` | `aws.ec2.SecurityGroupIngress.IpProtocol.119` |
| `120` | `aws.ec2.SecurityGroupIngress.IpProtocol.120` |
| `121` | `aws.ec2.SecurityGroupIngress.IpProtocol.121` |
| `122` | `aws.ec2.SecurityGroupIngress.IpProtocol.122` |
| `123` | `aws.ec2.SecurityGroupIngress.IpProtocol.123` |
| `124` | `aws.ec2.SecurityGroupIngress.IpProtocol.124` |
| `125` | `aws.ec2.SecurityGroupIngress.IpProtocol.125` |
| `126` | `aws.ec2.SecurityGroupIngress.IpProtocol.126` |
| `127` | `aws.ec2.SecurityGroupIngress.IpProtocol.127` |
| `128` | `aws.ec2.SecurityGroupIngress.IpProtocol.128` |
| `129` | `aws.ec2.SecurityGroupIngress.IpProtocol.129` |
| `130` | `aws.ec2.SecurityGroupIngress.IpProtocol.130` |
| `131` | `aws.ec2.SecurityGroupIngress.IpProtocol.131` |
| `132` | `aws.ec2.SecurityGroupIngress.IpProtocol.132` |
| `133` | `aws.ec2.SecurityGroupIngress.IpProtocol.133` |
| `134` | `aws.ec2.SecurityGroupIngress.IpProtocol.134` |
| `135` | `aws.ec2.SecurityGroupIngress.IpProtocol.135` |
| `136` | `aws.ec2.SecurityGroupIngress.IpProtocol.136` |
| `137` | `aws.ec2.SecurityGroupIngress.IpProtocol.137` |
| `138` | `aws.ec2.SecurityGroupIngress.IpProtocol.138` |
| `139` | `aws.ec2.SecurityGroupIngress.IpProtocol.139` |
| `140` | `aws.ec2.SecurityGroupIngress.IpProtocol.140` |
| `141` | `aws.ec2.SecurityGroupIngress.IpProtocol.141` |
| `142` | `aws.ec2.SecurityGroupIngress.IpProtocol.142` |
| `143` | `aws.ec2.SecurityGroupIngress.IpProtocol.143` |
| `144` | `aws.ec2.SecurityGroupIngress.IpProtocol.144` |
| `145` | `aws.ec2.SecurityGroupIngress.IpProtocol.145` |
| `146` | `aws.ec2.SecurityGroupIngress.IpProtocol.146` |
| `147` | `aws.ec2.SecurityGroupIngress.IpProtocol.147` |
| `148` | `aws.ec2.SecurityGroupIngress.IpProtocol.148` |
| `149` | `aws.ec2.SecurityGroupIngress.IpProtocol.149` |
| `150` | `aws.ec2.SecurityGroupIngress.IpProtocol.150` |
| `151` | `aws.ec2.SecurityGroupIngress.IpProtocol.151` |
| `152` | `aws.ec2.SecurityGroupIngress.IpProtocol.152` |
| `153` | `aws.ec2.SecurityGroupIngress.IpProtocol.153` |
| `154` | `aws.ec2.SecurityGroupIngress.IpProtocol.154` |
| `155` | `aws.ec2.SecurityGroupIngress.IpProtocol.155` |
| `156` | `aws.ec2.SecurityGroupIngress.IpProtocol.156` |
| `157` | `aws.ec2.SecurityGroupIngress.IpProtocol.157` |
| `158` | `aws.ec2.SecurityGroupIngress.IpProtocol.158` |
| `159` | `aws.ec2.SecurityGroupIngress.IpProtocol.159` |
| `160` | `aws.ec2.SecurityGroupIngress.IpProtocol.160` |
| `161` | `aws.ec2.SecurityGroupIngress.IpProtocol.161` |
| `162` | `aws.ec2.SecurityGroupIngress.IpProtocol.162` |
| `163` | `aws.ec2.SecurityGroupIngress.IpProtocol.163` |
| `164` | `aws.ec2.SecurityGroupIngress.IpProtocol.164` |
| `165` | `aws.ec2.SecurityGroupIngress.IpProtocol.165` |
| `166` | `aws.ec2.SecurityGroupIngress.IpProtocol.166` |
| `167` | `aws.ec2.SecurityGroupIngress.IpProtocol.167` |
| `168` | `aws.ec2.SecurityGroupIngress.IpProtocol.168` |
| `169` | `aws.ec2.SecurityGroupIngress.IpProtocol.169` |
| `170` | `aws.ec2.SecurityGroupIngress.IpProtocol.170` |
| `171` | `aws.ec2.SecurityGroupIngress.IpProtocol.171` |
| `172` | `aws.ec2.SecurityGroupIngress.IpProtocol.172` |
| `173` | `aws.ec2.SecurityGroupIngress.IpProtocol.173` |
| `174` | `aws.ec2.SecurityGroupIngress.IpProtocol.174` |
| `175` | `aws.ec2.SecurityGroupIngress.IpProtocol.175` |
| `176` | `aws.ec2.SecurityGroupIngress.IpProtocol.176` |
| `177` | `aws.ec2.SecurityGroupIngress.IpProtocol.177` |
| `178` | `aws.ec2.SecurityGroupIngress.IpProtocol.178` |
| `179` | `aws.ec2.SecurityGroupIngress.IpProtocol.179` |
| `180` | `aws.ec2.SecurityGroupIngress.IpProtocol.180` |
| `181` | `aws.ec2.SecurityGroupIngress.IpProtocol.181` |
| `182` | `aws.ec2.SecurityGroupIngress.IpProtocol.182` |
| `183` | `aws.ec2.SecurityGroupIngress.IpProtocol.183` |
| `184` | `aws.ec2.SecurityGroupIngress.IpProtocol.184` |
| `185` | `aws.ec2.SecurityGroupIngress.IpProtocol.185` |
| `186` | `aws.ec2.SecurityGroupIngress.IpProtocol.186` |
| `187` | `aws.ec2.SecurityGroupIngress.IpProtocol.187` |
| `188` | `aws.ec2.SecurityGroupIngress.IpProtocol.188` |
| `189` | `aws.ec2.SecurityGroupIngress.IpProtocol.189` |
| `190` | `aws.ec2.SecurityGroupIngress.IpProtocol.190` |
| `191` | `aws.ec2.SecurityGroupIngress.IpProtocol.191` |
| `192` | `aws.ec2.SecurityGroupIngress.IpProtocol.192` |
| `193` | `aws.ec2.SecurityGroupIngress.IpProtocol.193` |
| `194` | `aws.ec2.SecurityGroupIngress.IpProtocol.194` |
| `195` | `aws.ec2.SecurityGroupIngress.IpProtocol.195` |
| `196` | `aws.ec2.SecurityGroupIngress.IpProtocol.196` |
| `197` | `aws.ec2.SecurityGroupIngress.IpProtocol.197` |
| `198` | `aws.ec2.SecurityGroupIngress.IpProtocol.198` |
| `199` | `aws.ec2.SecurityGroupIngress.IpProtocol.199` |
| `200` | `aws.ec2.SecurityGroupIngress.IpProtocol.200` |
| `201` | `aws.ec2.SecurityGroupIngress.IpProtocol.201` |
| `202` | `aws.ec2.SecurityGroupIngress.IpProtocol.202` |
| `203` | `aws.ec2.SecurityGroupIngress.IpProtocol.203` |
| `204` | `aws.ec2.SecurityGroupIngress.IpProtocol.204` |
| `205` | `aws.ec2.SecurityGroupIngress.IpProtocol.205` |
| `206` | `aws.ec2.SecurityGroupIngress.IpProtocol.206` |
| `207` | `aws.ec2.SecurityGroupIngress.IpProtocol.207` |
| `208` | `aws.ec2.SecurityGroupIngress.IpProtocol.208` |
| `209` | `aws.ec2.SecurityGroupIngress.IpProtocol.209` |
| `210` | `aws.ec2.SecurityGroupIngress.IpProtocol.210` |
| `211` | `aws.ec2.SecurityGroupIngress.IpProtocol.211` |
| `212` | `aws.ec2.SecurityGroupIngress.IpProtocol.212` |
| `213` | `aws.ec2.SecurityGroupIngress.IpProtocol.213` |
| `214` | `aws.ec2.SecurityGroupIngress.IpProtocol.214` |
| `215` | `aws.ec2.SecurityGroupIngress.IpProtocol.215` |
| `216` | `aws.ec2.SecurityGroupIngress.IpProtocol.216` |
| `217` | `aws.ec2.SecurityGroupIngress.IpProtocol.217` |
| `218` | `aws.ec2.SecurityGroupIngress.IpProtocol.218` |
| `219` | `aws.ec2.SecurityGroupIngress.IpProtocol.219` |
| `220` | `aws.ec2.SecurityGroupIngress.IpProtocol.220` |
| `221` | `aws.ec2.SecurityGroupIngress.IpProtocol.221` |
| `222` | `aws.ec2.SecurityGroupIngress.IpProtocol.222` |
| `223` | `aws.ec2.SecurityGroupIngress.IpProtocol.223` |
| `224` | `aws.ec2.SecurityGroupIngress.IpProtocol.224` |
| `225` | `aws.ec2.SecurityGroupIngress.IpProtocol.225` |
| `226` | `aws.ec2.SecurityGroupIngress.IpProtocol.226` |
| `227` | `aws.ec2.SecurityGroupIngress.IpProtocol.227` |
| `228` | `aws.ec2.SecurityGroupIngress.IpProtocol.228` |
| `229` | `aws.ec2.SecurityGroupIngress.IpProtocol.229` |
| `230` | `aws.ec2.SecurityGroupIngress.IpProtocol.230` |
| `231` | `aws.ec2.SecurityGroupIngress.IpProtocol.231` |
| `232` | `aws.ec2.SecurityGroupIngress.IpProtocol.232` |
| `233` | `aws.ec2.SecurityGroupIngress.IpProtocol.233` |
| `234` | `aws.ec2.SecurityGroupIngress.IpProtocol.234` |
| `235` | `aws.ec2.SecurityGroupIngress.IpProtocol.235` |
| `236` | `aws.ec2.SecurityGroupIngress.IpProtocol.236` |
| `237` | `aws.ec2.SecurityGroupIngress.IpProtocol.237` |
| `238` | `aws.ec2.SecurityGroupIngress.IpProtocol.238` |
| `239` | `aws.ec2.SecurityGroupIngress.IpProtocol.239` |
| `240` | `aws.ec2.SecurityGroupIngress.IpProtocol.240` |
| `241` | `aws.ec2.SecurityGroupIngress.IpProtocol.241` |
| `242` | `aws.ec2.SecurityGroupIngress.IpProtocol.242` |
| `243` | `aws.ec2.SecurityGroupIngress.IpProtocol.243` |
| `244` | `aws.ec2.SecurityGroupIngress.IpProtocol.244` |
| `245` | `aws.ec2.SecurityGroupIngress.IpProtocol.245` |
| `246` | `aws.ec2.SecurityGroupIngress.IpProtocol.246` |
| `247` | `aws.ec2.SecurityGroupIngress.IpProtocol.247` |
| `248` | `aws.ec2.SecurityGroupIngress.IpProtocol.248` |
| `249` | `aws.ec2.SecurityGroupIngress.IpProtocol.249` |
| `250` | `aws.ec2.SecurityGroupIngress.IpProtocol.250` |
| `251` | `aws.ec2.SecurityGroupIngress.IpProtocol.251` |
| `252` | `aws.ec2.SecurityGroupIngress.IpProtocol.252` |
| `253` | `aws.ec2.SecurityGroupIngress.IpProtocol.253` |
| `254` | `aws.ec2.SecurityGroupIngress.IpProtocol.254` |
| `255` | `aws.ec2.SecurityGroupIngress.IpProtocol.255` |

Shorthand formats: `tcp` or `IpProtocol.tcp`

## Attribute Reference

### `id`

- **Type:** String

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

