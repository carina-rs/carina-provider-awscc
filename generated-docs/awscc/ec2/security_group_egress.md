---
title: "awscc.ec2.SecurityGroupEgress"
---


CloudFormation Type: `AWS::EC2::SecurityGroupEgress`

Adds the specified outbound (egress) rule to a security group.
 An outbound rule permits instances to send traffic to the specified IPv4 or IPv6 address range, the IP addresses that are specified by a prefix list, or the instances that are associated with a destination security group. For more information, see [Security group rules](https://docs.aws.amazon.com/vpc/latest/userguide/security-group-rules.html).
 You must specify exactly one of the following destinations: an IPv4 address range, an IPv6 address range, a prefix list, or a security group.
 You must specify a protocol for each rule (for example, TCP). If the protocol is TCP or UDP, you must also specify a port or port range. If the protocol is ICMP or ICMPv6, you must also specify the ICMP/ICMPv6 type and code. To specify all types or all codes, use -1.
 Rule changes are propagated to instances associated with the security group as quickly as possible. However, a small delay might occur.

## Example

```crn
let vpc = awscc.ec2.Vpc {
  cidr_block = '10.0.0.0/16'
}

let sg = awscc.ec2.SecurityGroup {
  vpc_id            = vpc.vpc_id
  group_description = 'Example security group'
}

awscc.ec2.SecurityGroupEgress {
  group_id    = sg.group_id
  description = 'Allow all outbound traffic'
  ip_protocol = all
  cidr_ip     = '0.0.0.0/0'
}
```

## Argument Reference

### `cidr_ip`

- **Type:** `Ipv4Cidr`
- **Required:** No
- **Create-only:** Yes

The IPv4 address range, in CIDR format. You must specify exactly one of the following: ``CidrIp``, ``CidrIpv6``, ``DestinationPrefixListId``, or ``DestinationSecurityGroupId``. For examples of rules that you can add to security groups for specific access scenarios, see [Security group rules for different use cases](https://docs.aws.amazon.com/AWSEC2/latest/UserGuide/security-group-rules-reference.html) in the *User Guide*.

### `cidr_ipv6`

- **Type:** `Ipv6Cidr`
- **Required:** No
- **Create-only:** Yes

The IPv6 address range, in CIDR format. You must specify exactly one of the following: ``CidrIp``, ``CidrIpv6``, ``DestinationPrefixListId``, or ``DestinationSecurityGroupId``. For examples of rules that you can add to security groups for specific access scenarios, see [Security group rules for different use cases](https://docs.aws.amazon.com/AWSEC2/latest/UserGuide/security-group-rules-reference.html) in the *User Guide*.

### `description`

- **Type:** `String(pattern, len: ..=255)`
- **Required:** No

The description of an egress (outbound) security group rule. Constraints: Up to 255 characters in length. Allowed characters are a-z, A-Z, 0-9, spaces, and ._-:/()#,@[]+=;{}!$*

### `destination_prefix_list_id`

- **Type:** `aws.ec2.PrefixList.Id`
- **Required:** No
- **Create-only:** Yes

The prefix list IDs for an AWS service. This is the AWS service to access through a VPC endpoint from instances associated with the security group. You must specify exactly one of the following: ``CidrIp``, ``CidrIpv6``, ``DestinationPrefixListId``, or ``DestinationSecurityGroupId``.

### `destination_security_group_id`

- **Type:** `aws.ec2.SecurityGroup.Id`
- **Required:** No
- **Create-only:** Yes

The ID of the security group. You must specify exactly one of the following: ``CidrIp``, ``CidrIpv6``, ``DestinationPrefixListId``, or ``DestinationSecurityGroupId``.

### `from_port`

- **Type:** Int(-1..=65535)
- **Required:** No
- **Create-only:** Yes

If the protocol is TCP or UDP, this is the start of the port range. If the protocol is ICMP or ICMPv6, this is the ICMP type or -1 (all ICMP types).

### `group_id`

- **Type:** `aws.ec2.SecurityGroup.Id`
- **Required:** Yes
- **Create-only:** Yes

The ID of the security group. You must specify either the security group ID or the security group name in the request. For security groups in a nondefault VPC, you must specify the security group ID.

### `ip_protocol`

- **Type:** [Enum (IpProtocol)](#ip_protocol-ipprotocol)
- **Required:** Yes
- **Create-only:** Yes

The IP protocol name (``tcp``, ``udp``, ``icmp``, ``icmpv6``) or number (see [Protocol Numbers](https://docs.aws.amazon.com/http://www.iana.org/assignments/protocol-numbers/protocol-numbers.xhtml)). Use ``-1`` to specify all protocols. When authorizing security group rules, specifying ``-1`` or a protocol number other than ``tcp``, ``udp``, ``icmp``, or ``icmpv6`` allows traffic on all ports, regardless of any port range you specify. For ``tcp``, ``udp``, and ``icmp``, you must specify a port range. For ``icmpv6``, the port range is optional; if you omit the port range, traffic for all types and codes is allowed.

### `to_port`

- **Type:** Int(-1..=65535)
- **Required:** No
- **Create-only:** Yes

If the protocol is TCP or UDP, this is the end of the port range. If the protocol is ICMP or ICMPv6, this is the ICMP code or -1 (all ICMP codes). If the start port is -1 (all ICMP types), then the end port must be -1 (all ICMP codes).

## Enum Values

### ip_protocol (IpProtocol)

| Value | DSL Identifier |
|-------|----------------|
| `tcp` | `aws.ec2.SecurityGroupEgress.IpProtocol.tcp` |
| `udp` | `aws.ec2.SecurityGroupEgress.IpProtocol.udp` |
| `icmp` | `aws.ec2.SecurityGroupEgress.IpProtocol.icmp` |
| `icmpv6` | `aws.ec2.SecurityGroupEgress.IpProtocol.icmpv6` |
| `-1` | `aws.ec2.SecurityGroupEgress.IpProtocol.all` |
| `0` | `aws.ec2.SecurityGroupEgress.IpProtocol.0` |
| `1` | `aws.ec2.SecurityGroupEgress.IpProtocol.1` |
| `2` | `aws.ec2.SecurityGroupEgress.IpProtocol.2` |
| `3` | `aws.ec2.SecurityGroupEgress.IpProtocol.3` |
| `4` | `aws.ec2.SecurityGroupEgress.IpProtocol.4` |
| `5` | `aws.ec2.SecurityGroupEgress.IpProtocol.5` |
| `6` | `aws.ec2.SecurityGroupEgress.IpProtocol.6` |
| `7` | `aws.ec2.SecurityGroupEgress.IpProtocol.7` |
| `8` | `aws.ec2.SecurityGroupEgress.IpProtocol.8` |
| `9` | `aws.ec2.SecurityGroupEgress.IpProtocol.9` |
| `10` | `aws.ec2.SecurityGroupEgress.IpProtocol.10` |
| `11` | `aws.ec2.SecurityGroupEgress.IpProtocol.11` |
| `12` | `aws.ec2.SecurityGroupEgress.IpProtocol.12` |
| `13` | `aws.ec2.SecurityGroupEgress.IpProtocol.13` |
| `14` | `aws.ec2.SecurityGroupEgress.IpProtocol.14` |
| `15` | `aws.ec2.SecurityGroupEgress.IpProtocol.15` |
| `16` | `aws.ec2.SecurityGroupEgress.IpProtocol.16` |
| `17` | `aws.ec2.SecurityGroupEgress.IpProtocol.17` |
| `18` | `aws.ec2.SecurityGroupEgress.IpProtocol.18` |
| `19` | `aws.ec2.SecurityGroupEgress.IpProtocol.19` |
| `20` | `aws.ec2.SecurityGroupEgress.IpProtocol.20` |
| `21` | `aws.ec2.SecurityGroupEgress.IpProtocol.21` |
| `22` | `aws.ec2.SecurityGroupEgress.IpProtocol.22` |
| `23` | `aws.ec2.SecurityGroupEgress.IpProtocol.23` |
| `24` | `aws.ec2.SecurityGroupEgress.IpProtocol.24` |
| `25` | `aws.ec2.SecurityGroupEgress.IpProtocol.25` |
| `26` | `aws.ec2.SecurityGroupEgress.IpProtocol.26` |
| `27` | `aws.ec2.SecurityGroupEgress.IpProtocol.27` |
| `28` | `aws.ec2.SecurityGroupEgress.IpProtocol.28` |
| `29` | `aws.ec2.SecurityGroupEgress.IpProtocol.29` |
| `30` | `aws.ec2.SecurityGroupEgress.IpProtocol.30` |
| `31` | `aws.ec2.SecurityGroupEgress.IpProtocol.31` |
| `32` | `aws.ec2.SecurityGroupEgress.IpProtocol.32` |
| `33` | `aws.ec2.SecurityGroupEgress.IpProtocol.33` |
| `34` | `aws.ec2.SecurityGroupEgress.IpProtocol.34` |
| `35` | `aws.ec2.SecurityGroupEgress.IpProtocol.35` |
| `36` | `aws.ec2.SecurityGroupEgress.IpProtocol.36` |
| `37` | `aws.ec2.SecurityGroupEgress.IpProtocol.37` |
| `38` | `aws.ec2.SecurityGroupEgress.IpProtocol.38` |
| `39` | `aws.ec2.SecurityGroupEgress.IpProtocol.39` |
| `40` | `aws.ec2.SecurityGroupEgress.IpProtocol.40` |
| `41` | `aws.ec2.SecurityGroupEgress.IpProtocol.41` |
| `42` | `aws.ec2.SecurityGroupEgress.IpProtocol.42` |
| `43` | `aws.ec2.SecurityGroupEgress.IpProtocol.43` |
| `44` | `aws.ec2.SecurityGroupEgress.IpProtocol.44` |
| `45` | `aws.ec2.SecurityGroupEgress.IpProtocol.45` |
| `46` | `aws.ec2.SecurityGroupEgress.IpProtocol.46` |
| `47` | `aws.ec2.SecurityGroupEgress.IpProtocol.47` |
| `48` | `aws.ec2.SecurityGroupEgress.IpProtocol.48` |
| `49` | `aws.ec2.SecurityGroupEgress.IpProtocol.49` |
| `50` | `aws.ec2.SecurityGroupEgress.IpProtocol.50` |
| `51` | `aws.ec2.SecurityGroupEgress.IpProtocol.51` |
| `52` | `aws.ec2.SecurityGroupEgress.IpProtocol.52` |
| `53` | `aws.ec2.SecurityGroupEgress.IpProtocol.53` |
| `54` | `aws.ec2.SecurityGroupEgress.IpProtocol.54` |
| `55` | `aws.ec2.SecurityGroupEgress.IpProtocol.55` |
| `56` | `aws.ec2.SecurityGroupEgress.IpProtocol.56` |
| `57` | `aws.ec2.SecurityGroupEgress.IpProtocol.57` |
| `58` | `aws.ec2.SecurityGroupEgress.IpProtocol.58` |
| `59` | `aws.ec2.SecurityGroupEgress.IpProtocol.59` |
| `60` | `aws.ec2.SecurityGroupEgress.IpProtocol.60` |
| `61` | `aws.ec2.SecurityGroupEgress.IpProtocol.61` |
| `62` | `aws.ec2.SecurityGroupEgress.IpProtocol.62` |
| `63` | `aws.ec2.SecurityGroupEgress.IpProtocol.63` |
| `64` | `aws.ec2.SecurityGroupEgress.IpProtocol.64` |
| `65` | `aws.ec2.SecurityGroupEgress.IpProtocol.65` |
| `66` | `aws.ec2.SecurityGroupEgress.IpProtocol.66` |
| `67` | `aws.ec2.SecurityGroupEgress.IpProtocol.67` |
| `68` | `aws.ec2.SecurityGroupEgress.IpProtocol.68` |
| `69` | `aws.ec2.SecurityGroupEgress.IpProtocol.69` |
| `70` | `aws.ec2.SecurityGroupEgress.IpProtocol.70` |
| `71` | `aws.ec2.SecurityGroupEgress.IpProtocol.71` |
| `72` | `aws.ec2.SecurityGroupEgress.IpProtocol.72` |
| `73` | `aws.ec2.SecurityGroupEgress.IpProtocol.73` |
| `74` | `aws.ec2.SecurityGroupEgress.IpProtocol.74` |
| `75` | `aws.ec2.SecurityGroupEgress.IpProtocol.75` |
| `76` | `aws.ec2.SecurityGroupEgress.IpProtocol.76` |
| `77` | `aws.ec2.SecurityGroupEgress.IpProtocol.77` |
| `78` | `aws.ec2.SecurityGroupEgress.IpProtocol.78` |
| `79` | `aws.ec2.SecurityGroupEgress.IpProtocol.79` |
| `80` | `aws.ec2.SecurityGroupEgress.IpProtocol.80` |
| `81` | `aws.ec2.SecurityGroupEgress.IpProtocol.81` |
| `82` | `aws.ec2.SecurityGroupEgress.IpProtocol.82` |
| `83` | `aws.ec2.SecurityGroupEgress.IpProtocol.83` |
| `84` | `aws.ec2.SecurityGroupEgress.IpProtocol.84` |
| `85` | `aws.ec2.SecurityGroupEgress.IpProtocol.85` |
| `86` | `aws.ec2.SecurityGroupEgress.IpProtocol.86` |
| `87` | `aws.ec2.SecurityGroupEgress.IpProtocol.87` |
| `88` | `aws.ec2.SecurityGroupEgress.IpProtocol.88` |
| `89` | `aws.ec2.SecurityGroupEgress.IpProtocol.89` |
| `90` | `aws.ec2.SecurityGroupEgress.IpProtocol.90` |
| `91` | `aws.ec2.SecurityGroupEgress.IpProtocol.91` |
| `92` | `aws.ec2.SecurityGroupEgress.IpProtocol.92` |
| `93` | `aws.ec2.SecurityGroupEgress.IpProtocol.93` |
| `94` | `aws.ec2.SecurityGroupEgress.IpProtocol.94` |
| `95` | `aws.ec2.SecurityGroupEgress.IpProtocol.95` |
| `96` | `aws.ec2.SecurityGroupEgress.IpProtocol.96` |
| `97` | `aws.ec2.SecurityGroupEgress.IpProtocol.97` |
| `98` | `aws.ec2.SecurityGroupEgress.IpProtocol.98` |
| `99` | `aws.ec2.SecurityGroupEgress.IpProtocol.99` |
| `100` | `aws.ec2.SecurityGroupEgress.IpProtocol.100` |
| `101` | `aws.ec2.SecurityGroupEgress.IpProtocol.101` |
| `102` | `aws.ec2.SecurityGroupEgress.IpProtocol.102` |
| `103` | `aws.ec2.SecurityGroupEgress.IpProtocol.103` |
| `104` | `aws.ec2.SecurityGroupEgress.IpProtocol.104` |
| `105` | `aws.ec2.SecurityGroupEgress.IpProtocol.105` |
| `106` | `aws.ec2.SecurityGroupEgress.IpProtocol.106` |
| `107` | `aws.ec2.SecurityGroupEgress.IpProtocol.107` |
| `108` | `aws.ec2.SecurityGroupEgress.IpProtocol.108` |
| `109` | `aws.ec2.SecurityGroupEgress.IpProtocol.109` |
| `110` | `aws.ec2.SecurityGroupEgress.IpProtocol.110` |
| `111` | `aws.ec2.SecurityGroupEgress.IpProtocol.111` |
| `112` | `aws.ec2.SecurityGroupEgress.IpProtocol.112` |
| `113` | `aws.ec2.SecurityGroupEgress.IpProtocol.113` |
| `114` | `aws.ec2.SecurityGroupEgress.IpProtocol.114` |
| `115` | `aws.ec2.SecurityGroupEgress.IpProtocol.115` |
| `116` | `aws.ec2.SecurityGroupEgress.IpProtocol.116` |
| `117` | `aws.ec2.SecurityGroupEgress.IpProtocol.117` |
| `118` | `aws.ec2.SecurityGroupEgress.IpProtocol.118` |
| `119` | `aws.ec2.SecurityGroupEgress.IpProtocol.119` |
| `120` | `aws.ec2.SecurityGroupEgress.IpProtocol.120` |
| `121` | `aws.ec2.SecurityGroupEgress.IpProtocol.121` |
| `122` | `aws.ec2.SecurityGroupEgress.IpProtocol.122` |
| `123` | `aws.ec2.SecurityGroupEgress.IpProtocol.123` |
| `124` | `aws.ec2.SecurityGroupEgress.IpProtocol.124` |
| `125` | `aws.ec2.SecurityGroupEgress.IpProtocol.125` |
| `126` | `aws.ec2.SecurityGroupEgress.IpProtocol.126` |
| `127` | `aws.ec2.SecurityGroupEgress.IpProtocol.127` |
| `128` | `aws.ec2.SecurityGroupEgress.IpProtocol.128` |
| `129` | `aws.ec2.SecurityGroupEgress.IpProtocol.129` |
| `130` | `aws.ec2.SecurityGroupEgress.IpProtocol.130` |
| `131` | `aws.ec2.SecurityGroupEgress.IpProtocol.131` |
| `132` | `aws.ec2.SecurityGroupEgress.IpProtocol.132` |
| `133` | `aws.ec2.SecurityGroupEgress.IpProtocol.133` |
| `134` | `aws.ec2.SecurityGroupEgress.IpProtocol.134` |
| `135` | `aws.ec2.SecurityGroupEgress.IpProtocol.135` |
| `136` | `aws.ec2.SecurityGroupEgress.IpProtocol.136` |
| `137` | `aws.ec2.SecurityGroupEgress.IpProtocol.137` |
| `138` | `aws.ec2.SecurityGroupEgress.IpProtocol.138` |
| `139` | `aws.ec2.SecurityGroupEgress.IpProtocol.139` |
| `140` | `aws.ec2.SecurityGroupEgress.IpProtocol.140` |
| `141` | `aws.ec2.SecurityGroupEgress.IpProtocol.141` |
| `142` | `aws.ec2.SecurityGroupEgress.IpProtocol.142` |
| `143` | `aws.ec2.SecurityGroupEgress.IpProtocol.143` |
| `144` | `aws.ec2.SecurityGroupEgress.IpProtocol.144` |
| `145` | `aws.ec2.SecurityGroupEgress.IpProtocol.145` |
| `146` | `aws.ec2.SecurityGroupEgress.IpProtocol.146` |
| `147` | `aws.ec2.SecurityGroupEgress.IpProtocol.147` |
| `148` | `aws.ec2.SecurityGroupEgress.IpProtocol.148` |
| `149` | `aws.ec2.SecurityGroupEgress.IpProtocol.149` |
| `150` | `aws.ec2.SecurityGroupEgress.IpProtocol.150` |
| `151` | `aws.ec2.SecurityGroupEgress.IpProtocol.151` |
| `152` | `aws.ec2.SecurityGroupEgress.IpProtocol.152` |
| `153` | `aws.ec2.SecurityGroupEgress.IpProtocol.153` |
| `154` | `aws.ec2.SecurityGroupEgress.IpProtocol.154` |
| `155` | `aws.ec2.SecurityGroupEgress.IpProtocol.155` |
| `156` | `aws.ec2.SecurityGroupEgress.IpProtocol.156` |
| `157` | `aws.ec2.SecurityGroupEgress.IpProtocol.157` |
| `158` | `aws.ec2.SecurityGroupEgress.IpProtocol.158` |
| `159` | `aws.ec2.SecurityGroupEgress.IpProtocol.159` |
| `160` | `aws.ec2.SecurityGroupEgress.IpProtocol.160` |
| `161` | `aws.ec2.SecurityGroupEgress.IpProtocol.161` |
| `162` | `aws.ec2.SecurityGroupEgress.IpProtocol.162` |
| `163` | `aws.ec2.SecurityGroupEgress.IpProtocol.163` |
| `164` | `aws.ec2.SecurityGroupEgress.IpProtocol.164` |
| `165` | `aws.ec2.SecurityGroupEgress.IpProtocol.165` |
| `166` | `aws.ec2.SecurityGroupEgress.IpProtocol.166` |
| `167` | `aws.ec2.SecurityGroupEgress.IpProtocol.167` |
| `168` | `aws.ec2.SecurityGroupEgress.IpProtocol.168` |
| `169` | `aws.ec2.SecurityGroupEgress.IpProtocol.169` |
| `170` | `aws.ec2.SecurityGroupEgress.IpProtocol.170` |
| `171` | `aws.ec2.SecurityGroupEgress.IpProtocol.171` |
| `172` | `aws.ec2.SecurityGroupEgress.IpProtocol.172` |
| `173` | `aws.ec2.SecurityGroupEgress.IpProtocol.173` |
| `174` | `aws.ec2.SecurityGroupEgress.IpProtocol.174` |
| `175` | `aws.ec2.SecurityGroupEgress.IpProtocol.175` |
| `176` | `aws.ec2.SecurityGroupEgress.IpProtocol.176` |
| `177` | `aws.ec2.SecurityGroupEgress.IpProtocol.177` |
| `178` | `aws.ec2.SecurityGroupEgress.IpProtocol.178` |
| `179` | `aws.ec2.SecurityGroupEgress.IpProtocol.179` |
| `180` | `aws.ec2.SecurityGroupEgress.IpProtocol.180` |
| `181` | `aws.ec2.SecurityGroupEgress.IpProtocol.181` |
| `182` | `aws.ec2.SecurityGroupEgress.IpProtocol.182` |
| `183` | `aws.ec2.SecurityGroupEgress.IpProtocol.183` |
| `184` | `aws.ec2.SecurityGroupEgress.IpProtocol.184` |
| `185` | `aws.ec2.SecurityGroupEgress.IpProtocol.185` |
| `186` | `aws.ec2.SecurityGroupEgress.IpProtocol.186` |
| `187` | `aws.ec2.SecurityGroupEgress.IpProtocol.187` |
| `188` | `aws.ec2.SecurityGroupEgress.IpProtocol.188` |
| `189` | `aws.ec2.SecurityGroupEgress.IpProtocol.189` |
| `190` | `aws.ec2.SecurityGroupEgress.IpProtocol.190` |
| `191` | `aws.ec2.SecurityGroupEgress.IpProtocol.191` |
| `192` | `aws.ec2.SecurityGroupEgress.IpProtocol.192` |
| `193` | `aws.ec2.SecurityGroupEgress.IpProtocol.193` |
| `194` | `aws.ec2.SecurityGroupEgress.IpProtocol.194` |
| `195` | `aws.ec2.SecurityGroupEgress.IpProtocol.195` |
| `196` | `aws.ec2.SecurityGroupEgress.IpProtocol.196` |
| `197` | `aws.ec2.SecurityGroupEgress.IpProtocol.197` |
| `198` | `aws.ec2.SecurityGroupEgress.IpProtocol.198` |
| `199` | `aws.ec2.SecurityGroupEgress.IpProtocol.199` |
| `200` | `aws.ec2.SecurityGroupEgress.IpProtocol.200` |
| `201` | `aws.ec2.SecurityGroupEgress.IpProtocol.201` |
| `202` | `aws.ec2.SecurityGroupEgress.IpProtocol.202` |
| `203` | `aws.ec2.SecurityGroupEgress.IpProtocol.203` |
| `204` | `aws.ec2.SecurityGroupEgress.IpProtocol.204` |
| `205` | `aws.ec2.SecurityGroupEgress.IpProtocol.205` |
| `206` | `aws.ec2.SecurityGroupEgress.IpProtocol.206` |
| `207` | `aws.ec2.SecurityGroupEgress.IpProtocol.207` |
| `208` | `aws.ec2.SecurityGroupEgress.IpProtocol.208` |
| `209` | `aws.ec2.SecurityGroupEgress.IpProtocol.209` |
| `210` | `aws.ec2.SecurityGroupEgress.IpProtocol.210` |
| `211` | `aws.ec2.SecurityGroupEgress.IpProtocol.211` |
| `212` | `aws.ec2.SecurityGroupEgress.IpProtocol.212` |
| `213` | `aws.ec2.SecurityGroupEgress.IpProtocol.213` |
| `214` | `aws.ec2.SecurityGroupEgress.IpProtocol.214` |
| `215` | `aws.ec2.SecurityGroupEgress.IpProtocol.215` |
| `216` | `aws.ec2.SecurityGroupEgress.IpProtocol.216` |
| `217` | `aws.ec2.SecurityGroupEgress.IpProtocol.217` |
| `218` | `aws.ec2.SecurityGroupEgress.IpProtocol.218` |
| `219` | `aws.ec2.SecurityGroupEgress.IpProtocol.219` |
| `220` | `aws.ec2.SecurityGroupEgress.IpProtocol.220` |
| `221` | `aws.ec2.SecurityGroupEgress.IpProtocol.221` |
| `222` | `aws.ec2.SecurityGroupEgress.IpProtocol.222` |
| `223` | `aws.ec2.SecurityGroupEgress.IpProtocol.223` |
| `224` | `aws.ec2.SecurityGroupEgress.IpProtocol.224` |
| `225` | `aws.ec2.SecurityGroupEgress.IpProtocol.225` |
| `226` | `aws.ec2.SecurityGroupEgress.IpProtocol.226` |
| `227` | `aws.ec2.SecurityGroupEgress.IpProtocol.227` |
| `228` | `aws.ec2.SecurityGroupEgress.IpProtocol.228` |
| `229` | `aws.ec2.SecurityGroupEgress.IpProtocol.229` |
| `230` | `aws.ec2.SecurityGroupEgress.IpProtocol.230` |
| `231` | `aws.ec2.SecurityGroupEgress.IpProtocol.231` |
| `232` | `aws.ec2.SecurityGroupEgress.IpProtocol.232` |
| `233` | `aws.ec2.SecurityGroupEgress.IpProtocol.233` |
| `234` | `aws.ec2.SecurityGroupEgress.IpProtocol.234` |
| `235` | `aws.ec2.SecurityGroupEgress.IpProtocol.235` |
| `236` | `aws.ec2.SecurityGroupEgress.IpProtocol.236` |
| `237` | `aws.ec2.SecurityGroupEgress.IpProtocol.237` |
| `238` | `aws.ec2.SecurityGroupEgress.IpProtocol.238` |
| `239` | `aws.ec2.SecurityGroupEgress.IpProtocol.239` |
| `240` | `aws.ec2.SecurityGroupEgress.IpProtocol.240` |
| `241` | `aws.ec2.SecurityGroupEgress.IpProtocol.241` |
| `242` | `aws.ec2.SecurityGroupEgress.IpProtocol.242` |
| `243` | `aws.ec2.SecurityGroupEgress.IpProtocol.243` |
| `244` | `aws.ec2.SecurityGroupEgress.IpProtocol.244` |
| `245` | `aws.ec2.SecurityGroupEgress.IpProtocol.245` |
| `246` | `aws.ec2.SecurityGroupEgress.IpProtocol.246` |
| `247` | `aws.ec2.SecurityGroupEgress.IpProtocol.247` |
| `248` | `aws.ec2.SecurityGroupEgress.IpProtocol.248` |
| `249` | `aws.ec2.SecurityGroupEgress.IpProtocol.249` |
| `250` | `aws.ec2.SecurityGroupEgress.IpProtocol.250` |
| `251` | `aws.ec2.SecurityGroupEgress.IpProtocol.251` |
| `252` | `aws.ec2.SecurityGroupEgress.IpProtocol.252` |
| `253` | `aws.ec2.SecurityGroupEgress.IpProtocol.253` |
| `254` | `aws.ec2.SecurityGroupEgress.IpProtocol.254` |
| `255` | `aws.ec2.SecurityGroupEgress.IpProtocol.255` |

Shorthand formats: `tcp` or `IpProtocol.tcp`

## Attribute Reference

### `id`

- **Type:** String



## AWS API narrowings

These fields are narrower than or equal to CFN. Input rules combine documented AWS domains, recorded live observations, and Carina canonical spellings for stable state comparison. A canonical DSL spelling does not claim AWS rejects other spellings. Attaching a type name adds no structural constraints. Field names below use CFN spelling.

| CFN field | Relation to CFN | Evidence URL | Verbatim quote / exact observation | Checked / observed |
| --- | --- | --- | --- | --- |
| `/properties/Description` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/AWSCloudFormation/latest/TemplateReference/aws-resource-ec2-securitygroupegress.html#cfn-ec2-securitygroupegress-description) | ` The description of an egress (outbound) security group rule. `<br>` Constraints: Up to 255 characters in length. Allowed characters are a-z, A-Z, 0-9, spaces, and ._-:/()#,@[]+=;{}!$* ` | 2026-11-07 |
| `/properties/FromPort` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/managedservices/latest/ctref/schemas.html) | ` The start of allowed port range, from 0 to 65535 for TCP/UDP. For ICMP, use -1. ` | 2026-11-07 |
| `/properties/FromPort` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/AWSEC2/latest/APIReference/API_IpPermission.html) | ` If the protocol is ICMP or ICMPv6, this is the ICMP type or -1 (all ICMP types). ` | 2026-11-07 |
| `/properties/IpProtocol` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/AWSEC2/latest/APIReference/API_IpPermission.html) | ` The IP protocol name (tcp, udp, icmp, icmpv6) or number (see Protocol Numbers). `<br>` Use -1 to specify all protocols. ` | 2026-11-07 |
| `/properties/IpProtocol` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/network-firewall/latest/APIReference/API_MatchAttributes.html#networkfirewall-Type-MatchAttributes-Protocols) | ` The protocols to inspect for, specified using the assigned internet protocol number (IANA) for each protocol. `<br>` Valid Range: Minimum value of 0. Maximum value of 255. ` | 2026-11-07 |
| `/properties/ToPort` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/managedservices/latest/ctref/schemas.html) | ` The end of allowed port range, from 0 to 65535 for TCP/UDP. For ICMP, use -1. ` | 2026-11-07 |
| `/properties/ToPort` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/AWSEC2/latest/APIReference/API_IpPermission.html) | ` If the protocol is ICMP or ICMPv6, this is the ICMP type or -1 (all ICMP types). ` | 2026-11-07 |

