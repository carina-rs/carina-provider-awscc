---
title: "awscc.ec2.SecurityGroup"
---


CloudFormation Type: `AWS::EC2::SecurityGroup`

Resource Type definition for AWS::EC2::SecurityGroup

## Example

```crn
let vpc = awscc.ec2.Vpc {
  cidr_block = '10.0.0.0/16'
}

awscc.ec2.SecurityGroup {
  vpc_id            = vpc.vpc_id
  group_description = 'Example security group'

  security_group_ingress {
    ip_protocol = tcp
    from_port   = 80
    to_port     = 80
    cidr_ip     = '0.0.0.0/0'
  }

  security_group_ingress {
    ip_protocol = tcp
    from_port   = 443
    to_port     = 443
    cidr_ip     = '0.0.0.0/0'
  }

  tags = {
    Environment = 'example'
  }
}
```

## Argument Reference

### `group_description`

- **Type:** `String(pattern, len: ..=255)`
- **Required:** Yes
- **Create-only:** Yes

A description for the security group.

### `group_name`

- **Type:** String
- **Required:** No
- **Create-only:** Yes

The name of the security group.

### `security_group_egress`

- **Type:** List\<[Struct(Egress)](#security_group_egress-1)\>
- **Required:** No

[VPC only] The outbound rules associated with the security group. There is a short interruption during which you cannot connect to the security group.

### `security_group_ingress`

- **Type:** List\<[Struct(Ingress)](#security_group_ingress-1)\>
- **Required:** No

The inbound rules associated with the security group. There is a short interruption during which you cannot connect to the security group.

### `tags`

- **Type:** `Map<String(len: ..=128), String(len: ..=256)>`
- **Required:** No

Any tags assigned to the security group.

### `vpc_id`

- **Type:** `aws.ec2.Vpc.Id`
- **Required:** No
- **Create-only:** Yes

The ID of the VPC for the security group.

## Enum Values

### security_group_egress.ip_protocol (IpProtocol)

| Value | DSL Identifier |
|-------|----------------|
| `tcp` | `aws.ec2.SecurityGroup.Egress.IpProtocol.tcp` |
| `udp` | `aws.ec2.SecurityGroup.Egress.IpProtocol.udp` |
| `icmp` | `aws.ec2.SecurityGroup.Egress.IpProtocol.icmp` |
| `icmpv6` | `aws.ec2.SecurityGroup.Egress.IpProtocol.icmpv6` |
| `-1` | `aws.ec2.SecurityGroup.Egress.IpProtocol.all` |
| `0` | `aws.ec2.SecurityGroup.Egress.IpProtocol.0` |
| `1` | `aws.ec2.SecurityGroup.Egress.IpProtocol.1` |
| `2` | `aws.ec2.SecurityGroup.Egress.IpProtocol.2` |
| `3` | `aws.ec2.SecurityGroup.Egress.IpProtocol.3` |
| `4` | `aws.ec2.SecurityGroup.Egress.IpProtocol.4` |
| `5` | `aws.ec2.SecurityGroup.Egress.IpProtocol.5` |
| `6` | `aws.ec2.SecurityGroup.Egress.IpProtocol.6` |
| `7` | `aws.ec2.SecurityGroup.Egress.IpProtocol.7` |
| `8` | `aws.ec2.SecurityGroup.Egress.IpProtocol.8` |
| `9` | `aws.ec2.SecurityGroup.Egress.IpProtocol.9` |
| `10` | `aws.ec2.SecurityGroup.Egress.IpProtocol.10` |
| `11` | `aws.ec2.SecurityGroup.Egress.IpProtocol.11` |
| `12` | `aws.ec2.SecurityGroup.Egress.IpProtocol.12` |
| `13` | `aws.ec2.SecurityGroup.Egress.IpProtocol.13` |
| `14` | `aws.ec2.SecurityGroup.Egress.IpProtocol.14` |
| `15` | `aws.ec2.SecurityGroup.Egress.IpProtocol.15` |
| `16` | `aws.ec2.SecurityGroup.Egress.IpProtocol.16` |
| `17` | `aws.ec2.SecurityGroup.Egress.IpProtocol.17` |
| `18` | `aws.ec2.SecurityGroup.Egress.IpProtocol.18` |
| `19` | `aws.ec2.SecurityGroup.Egress.IpProtocol.19` |
| `20` | `aws.ec2.SecurityGroup.Egress.IpProtocol.20` |
| `21` | `aws.ec2.SecurityGroup.Egress.IpProtocol.21` |
| `22` | `aws.ec2.SecurityGroup.Egress.IpProtocol.22` |
| `23` | `aws.ec2.SecurityGroup.Egress.IpProtocol.23` |
| `24` | `aws.ec2.SecurityGroup.Egress.IpProtocol.24` |
| `25` | `aws.ec2.SecurityGroup.Egress.IpProtocol.25` |
| `26` | `aws.ec2.SecurityGroup.Egress.IpProtocol.26` |
| `27` | `aws.ec2.SecurityGroup.Egress.IpProtocol.27` |
| `28` | `aws.ec2.SecurityGroup.Egress.IpProtocol.28` |
| `29` | `aws.ec2.SecurityGroup.Egress.IpProtocol.29` |
| `30` | `aws.ec2.SecurityGroup.Egress.IpProtocol.30` |
| `31` | `aws.ec2.SecurityGroup.Egress.IpProtocol.31` |
| `32` | `aws.ec2.SecurityGroup.Egress.IpProtocol.32` |
| `33` | `aws.ec2.SecurityGroup.Egress.IpProtocol.33` |
| `34` | `aws.ec2.SecurityGroup.Egress.IpProtocol.34` |
| `35` | `aws.ec2.SecurityGroup.Egress.IpProtocol.35` |
| `36` | `aws.ec2.SecurityGroup.Egress.IpProtocol.36` |
| `37` | `aws.ec2.SecurityGroup.Egress.IpProtocol.37` |
| `38` | `aws.ec2.SecurityGroup.Egress.IpProtocol.38` |
| `39` | `aws.ec2.SecurityGroup.Egress.IpProtocol.39` |
| `40` | `aws.ec2.SecurityGroup.Egress.IpProtocol.40` |
| `41` | `aws.ec2.SecurityGroup.Egress.IpProtocol.41` |
| `42` | `aws.ec2.SecurityGroup.Egress.IpProtocol.42` |
| `43` | `aws.ec2.SecurityGroup.Egress.IpProtocol.43` |
| `44` | `aws.ec2.SecurityGroup.Egress.IpProtocol.44` |
| `45` | `aws.ec2.SecurityGroup.Egress.IpProtocol.45` |
| `46` | `aws.ec2.SecurityGroup.Egress.IpProtocol.46` |
| `47` | `aws.ec2.SecurityGroup.Egress.IpProtocol.47` |
| `48` | `aws.ec2.SecurityGroup.Egress.IpProtocol.48` |
| `49` | `aws.ec2.SecurityGroup.Egress.IpProtocol.49` |
| `50` | `aws.ec2.SecurityGroup.Egress.IpProtocol.50` |
| `51` | `aws.ec2.SecurityGroup.Egress.IpProtocol.51` |
| `52` | `aws.ec2.SecurityGroup.Egress.IpProtocol.52` |
| `53` | `aws.ec2.SecurityGroup.Egress.IpProtocol.53` |
| `54` | `aws.ec2.SecurityGroup.Egress.IpProtocol.54` |
| `55` | `aws.ec2.SecurityGroup.Egress.IpProtocol.55` |
| `56` | `aws.ec2.SecurityGroup.Egress.IpProtocol.56` |
| `57` | `aws.ec2.SecurityGroup.Egress.IpProtocol.57` |
| `58` | `aws.ec2.SecurityGroup.Egress.IpProtocol.58` |
| `59` | `aws.ec2.SecurityGroup.Egress.IpProtocol.59` |
| `60` | `aws.ec2.SecurityGroup.Egress.IpProtocol.60` |
| `61` | `aws.ec2.SecurityGroup.Egress.IpProtocol.61` |
| `62` | `aws.ec2.SecurityGroup.Egress.IpProtocol.62` |
| `63` | `aws.ec2.SecurityGroup.Egress.IpProtocol.63` |
| `64` | `aws.ec2.SecurityGroup.Egress.IpProtocol.64` |
| `65` | `aws.ec2.SecurityGroup.Egress.IpProtocol.65` |
| `66` | `aws.ec2.SecurityGroup.Egress.IpProtocol.66` |
| `67` | `aws.ec2.SecurityGroup.Egress.IpProtocol.67` |
| `68` | `aws.ec2.SecurityGroup.Egress.IpProtocol.68` |
| `69` | `aws.ec2.SecurityGroup.Egress.IpProtocol.69` |
| `70` | `aws.ec2.SecurityGroup.Egress.IpProtocol.70` |
| `71` | `aws.ec2.SecurityGroup.Egress.IpProtocol.71` |
| `72` | `aws.ec2.SecurityGroup.Egress.IpProtocol.72` |
| `73` | `aws.ec2.SecurityGroup.Egress.IpProtocol.73` |
| `74` | `aws.ec2.SecurityGroup.Egress.IpProtocol.74` |
| `75` | `aws.ec2.SecurityGroup.Egress.IpProtocol.75` |
| `76` | `aws.ec2.SecurityGroup.Egress.IpProtocol.76` |
| `77` | `aws.ec2.SecurityGroup.Egress.IpProtocol.77` |
| `78` | `aws.ec2.SecurityGroup.Egress.IpProtocol.78` |
| `79` | `aws.ec2.SecurityGroup.Egress.IpProtocol.79` |
| `80` | `aws.ec2.SecurityGroup.Egress.IpProtocol.80` |
| `81` | `aws.ec2.SecurityGroup.Egress.IpProtocol.81` |
| `82` | `aws.ec2.SecurityGroup.Egress.IpProtocol.82` |
| `83` | `aws.ec2.SecurityGroup.Egress.IpProtocol.83` |
| `84` | `aws.ec2.SecurityGroup.Egress.IpProtocol.84` |
| `85` | `aws.ec2.SecurityGroup.Egress.IpProtocol.85` |
| `86` | `aws.ec2.SecurityGroup.Egress.IpProtocol.86` |
| `87` | `aws.ec2.SecurityGroup.Egress.IpProtocol.87` |
| `88` | `aws.ec2.SecurityGroup.Egress.IpProtocol.88` |
| `89` | `aws.ec2.SecurityGroup.Egress.IpProtocol.89` |
| `90` | `aws.ec2.SecurityGroup.Egress.IpProtocol.90` |
| `91` | `aws.ec2.SecurityGroup.Egress.IpProtocol.91` |
| `92` | `aws.ec2.SecurityGroup.Egress.IpProtocol.92` |
| `93` | `aws.ec2.SecurityGroup.Egress.IpProtocol.93` |
| `94` | `aws.ec2.SecurityGroup.Egress.IpProtocol.94` |
| `95` | `aws.ec2.SecurityGroup.Egress.IpProtocol.95` |
| `96` | `aws.ec2.SecurityGroup.Egress.IpProtocol.96` |
| `97` | `aws.ec2.SecurityGroup.Egress.IpProtocol.97` |
| `98` | `aws.ec2.SecurityGroup.Egress.IpProtocol.98` |
| `99` | `aws.ec2.SecurityGroup.Egress.IpProtocol.99` |
| `100` | `aws.ec2.SecurityGroup.Egress.IpProtocol.100` |
| `101` | `aws.ec2.SecurityGroup.Egress.IpProtocol.101` |
| `102` | `aws.ec2.SecurityGroup.Egress.IpProtocol.102` |
| `103` | `aws.ec2.SecurityGroup.Egress.IpProtocol.103` |
| `104` | `aws.ec2.SecurityGroup.Egress.IpProtocol.104` |
| `105` | `aws.ec2.SecurityGroup.Egress.IpProtocol.105` |
| `106` | `aws.ec2.SecurityGroup.Egress.IpProtocol.106` |
| `107` | `aws.ec2.SecurityGroup.Egress.IpProtocol.107` |
| `108` | `aws.ec2.SecurityGroup.Egress.IpProtocol.108` |
| `109` | `aws.ec2.SecurityGroup.Egress.IpProtocol.109` |
| `110` | `aws.ec2.SecurityGroup.Egress.IpProtocol.110` |
| `111` | `aws.ec2.SecurityGroup.Egress.IpProtocol.111` |
| `112` | `aws.ec2.SecurityGroup.Egress.IpProtocol.112` |
| `113` | `aws.ec2.SecurityGroup.Egress.IpProtocol.113` |
| `114` | `aws.ec2.SecurityGroup.Egress.IpProtocol.114` |
| `115` | `aws.ec2.SecurityGroup.Egress.IpProtocol.115` |
| `116` | `aws.ec2.SecurityGroup.Egress.IpProtocol.116` |
| `117` | `aws.ec2.SecurityGroup.Egress.IpProtocol.117` |
| `118` | `aws.ec2.SecurityGroup.Egress.IpProtocol.118` |
| `119` | `aws.ec2.SecurityGroup.Egress.IpProtocol.119` |
| `120` | `aws.ec2.SecurityGroup.Egress.IpProtocol.120` |
| `121` | `aws.ec2.SecurityGroup.Egress.IpProtocol.121` |
| `122` | `aws.ec2.SecurityGroup.Egress.IpProtocol.122` |
| `123` | `aws.ec2.SecurityGroup.Egress.IpProtocol.123` |
| `124` | `aws.ec2.SecurityGroup.Egress.IpProtocol.124` |
| `125` | `aws.ec2.SecurityGroup.Egress.IpProtocol.125` |
| `126` | `aws.ec2.SecurityGroup.Egress.IpProtocol.126` |
| `127` | `aws.ec2.SecurityGroup.Egress.IpProtocol.127` |
| `128` | `aws.ec2.SecurityGroup.Egress.IpProtocol.128` |
| `129` | `aws.ec2.SecurityGroup.Egress.IpProtocol.129` |
| `130` | `aws.ec2.SecurityGroup.Egress.IpProtocol.130` |
| `131` | `aws.ec2.SecurityGroup.Egress.IpProtocol.131` |
| `132` | `aws.ec2.SecurityGroup.Egress.IpProtocol.132` |
| `133` | `aws.ec2.SecurityGroup.Egress.IpProtocol.133` |
| `134` | `aws.ec2.SecurityGroup.Egress.IpProtocol.134` |
| `135` | `aws.ec2.SecurityGroup.Egress.IpProtocol.135` |
| `136` | `aws.ec2.SecurityGroup.Egress.IpProtocol.136` |
| `137` | `aws.ec2.SecurityGroup.Egress.IpProtocol.137` |
| `138` | `aws.ec2.SecurityGroup.Egress.IpProtocol.138` |
| `139` | `aws.ec2.SecurityGroup.Egress.IpProtocol.139` |
| `140` | `aws.ec2.SecurityGroup.Egress.IpProtocol.140` |
| `141` | `aws.ec2.SecurityGroup.Egress.IpProtocol.141` |
| `142` | `aws.ec2.SecurityGroup.Egress.IpProtocol.142` |
| `143` | `aws.ec2.SecurityGroup.Egress.IpProtocol.143` |
| `144` | `aws.ec2.SecurityGroup.Egress.IpProtocol.144` |
| `145` | `aws.ec2.SecurityGroup.Egress.IpProtocol.145` |
| `146` | `aws.ec2.SecurityGroup.Egress.IpProtocol.146` |
| `147` | `aws.ec2.SecurityGroup.Egress.IpProtocol.147` |
| `148` | `aws.ec2.SecurityGroup.Egress.IpProtocol.148` |
| `149` | `aws.ec2.SecurityGroup.Egress.IpProtocol.149` |
| `150` | `aws.ec2.SecurityGroup.Egress.IpProtocol.150` |
| `151` | `aws.ec2.SecurityGroup.Egress.IpProtocol.151` |
| `152` | `aws.ec2.SecurityGroup.Egress.IpProtocol.152` |
| `153` | `aws.ec2.SecurityGroup.Egress.IpProtocol.153` |
| `154` | `aws.ec2.SecurityGroup.Egress.IpProtocol.154` |
| `155` | `aws.ec2.SecurityGroup.Egress.IpProtocol.155` |
| `156` | `aws.ec2.SecurityGroup.Egress.IpProtocol.156` |
| `157` | `aws.ec2.SecurityGroup.Egress.IpProtocol.157` |
| `158` | `aws.ec2.SecurityGroup.Egress.IpProtocol.158` |
| `159` | `aws.ec2.SecurityGroup.Egress.IpProtocol.159` |
| `160` | `aws.ec2.SecurityGroup.Egress.IpProtocol.160` |
| `161` | `aws.ec2.SecurityGroup.Egress.IpProtocol.161` |
| `162` | `aws.ec2.SecurityGroup.Egress.IpProtocol.162` |
| `163` | `aws.ec2.SecurityGroup.Egress.IpProtocol.163` |
| `164` | `aws.ec2.SecurityGroup.Egress.IpProtocol.164` |
| `165` | `aws.ec2.SecurityGroup.Egress.IpProtocol.165` |
| `166` | `aws.ec2.SecurityGroup.Egress.IpProtocol.166` |
| `167` | `aws.ec2.SecurityGroup.Egress.IpProtocol.167` |
| `168` | `aws.ec2.SecurityGroup.Egress.IpProtocol.168` |
| `169` | `aws.ec2.SecurityGroup.Egress.IpProtocol.169` |
| `170` | `aws.ec2.SecurityGroup.Egress.IpProtocol.170` |
| `171` | `aws.ec2.SecurityGroup.Egress.IpProtocol.171` |
| `172` | `aws.ec2.SecurityGroup.Egress.IpProtocol.172` |
| `173` | `aws.ec2.SecurityGroup.Egress.IpProtocol.173` |
| `174` | `aws.ec2.SecurityGroup.Egress.IpProtocol.174` |
| `175` | `aws.ec2.SecurityGroup.Egress.IpProtocol.175` |
| `176` | `aws.ec2.SecurityGroup.Egress.IpProtocol.176` |
| `177` | `aws.ec2.SecurityGroup.Egress.IpProtocol.177` |
| `178` | `aws.ec2.SecurityGroup.Egress.IpProtocol.178` |
| `179` | `aws.ec2.SecurityGroup.Egress.IpProtocol.179` |
| `180` | `aws.ec2.SecurityGroup.Egress.IpProtocol.180` |
| `181` | `aws.ec2.SecurityGroup.Egress.IpProtocol.181` |
| `182` | `aws.ec2.SecurityGroup.Egress.IpProtocol.182` |
| `183` | `aws.ec2.SecurityGroup.Egress.IpProtocol.183` |
| `184` | `aws.ec2.SecurityGroup.Egress.IpProtocol.184` |
| `185` | `aws.ec2.SecurityGroup.Egress.IpProtocol.185` |
| `186` | `aws.ec2.SecurityGroup.Egress.IpProtocol.186` |
| `187` | `aws.ec2.SecurityGroup.Egress.IpProtocol.187` |
| `188` | `aws.ec2.SecurityGroup.Egress.IpProtocol.188` |
| `189` | `aws.ec2.SecurityGroup.Egress.IpProtocol.189` |
| `190` | `aws.ec2.SecurityGroup.Egress.IpProtocol.190` |
| `191` | `aws.ec2.SecurityGroup.Egress.IpProtocol.191` |
| `192` | `aws.ec2.SecurityGroup.Egress.IpProtocol.192` |
| `193` | `aws.ec2.SecurityGroup.Egress.IpProtocol.193` |
| `194` | `aws.ec2.SecurityGroup.Egress.IpProtocol.194` |
| `195` | `aws.ec2.SecurityGroup.Egress.IpProtocol.195` |
| `196` | `aws.ec2.SecurityGroup.Egress.IpProtocol.196` |
| `197` | `aws.ec2.SecurityGroup.Egress.IpProtocol.197` |
| `198` | `aws.ec2.SecurityGroup.Egress.IpProtocol.198` |
| `199` | `aws.ec2.SecurityGroup.Egress.IpProtocol.199` |
| `200` | `aws.ec2.SecurityGroup.Egress.IpProtocol.200` |
| `201` | `aws.ec2.SecurityGroup.Egress.IpProtocol.201` |
| `202` | `aws.ec2.SecurityGroup.Egress.IpProtocol.202` |
| `203` | `aws.ec2.SecurityGroup.Egress.IpProtocol.203` |
| `204` | `aws.ec2.SecurityGroup.Egress.IpProtocol.204` |
| `205` | `aws.ec2.SecurityGroup.Egress.IpProtocol.205` |
| `206` | `aws.ec2.SecurityGroup.Egress.IpProtocol.206` |
| `207` | `aws.ec2.SecurityGroup.Egress.IpProtocol.207` |
| `208` | `aws.ec2.SecurityGroup.Egress.IpProtocol.208` |
| `209` | `aws.ec2.SecurityGroup.Egress.IpProtocol.209` |
| `210` | `aws.ec2.SecurityGroup.Egress.IpProtocol.210` |
| `211` | `aws.ec2.SecurityGroup.Egress.IpProtocol.211` |
| `212` | `aws.ec2.SecurityGroup.Egress.IpProtocol.212` |
| `213` | `aws.ec2.SecurityGroup.Egress.IpProtocol.213` |
| `214` | `aws.ec2.SecurityGroup.Egress.IpProtocol.214` |
| `215` | `aws.ec2.SecurityGroup.Egress.IpProtocol.215` |
| `216` | `aws.ec2.SecurityGroup.Egress.IpProtocol.216` |
| `217` | `aws.ec2.SecurityGroup.Egress.IpProtocol.217` |
| `218` | `aws.ec2.SecurityGroup.Egress.IpProtocol.218` |
| `219` | `aws.ec2.SecurityGroup.Egress.IpProtocol.219` |
| `220` | `aws.ec2.SecurityGroup.Egress.IpProtocol.220` |
| `221` | `aws.ec2.SecurityGroup.Egress.IpProtocol.221` |
| `222` | `aws.ec2.SecurityGroup.Egress.IpProtocol.222` |
| `223` | `aws.ec2.SecurityGroup.Egress.IpProtocol.223` |
| `224` | `aws.ec2.SecurityGroup.Egress.IpProtocol.224` |
| `225` | `aws.ec2.SecurityGroup.Egress.IpProtocol.225` |
| `226` | `aws.ec2.SecurityGroup.Egress.IpProtocol.226` |
| `227` | `aws.ec2.SecurityGroup.Egress.IpProtocol.227` |
| `228` | `aws.ec2.SecurityGroup.Egress.IpProtocol.228` |
| `229` | `aws.ec2.SecurityGroup.Egress.IpProtocol.229` |
| `230` | `aws.ec2.SecurityGroup.Egress.IpProtocol.230` |
| `231` | `aws.ec2.SecurityGroup.Egress.IpProtocol.231` |
| `232` | `aws.ec2.SecurityGroup.Egress.IpProtocol.232` |
| `233` | `aws.ec2.SecurityGroup.Egress.IpProtocol.233` |
| `234` | `aws.ec2.SecurityGroup.Egress.IpProtocol.234` |
| `235` | `aws.ec2.SecurityGroup.Egress.IpProtocol.235` |
| `236` | `aws.ec2.SecurityGroup.Egress.IpProtocol.236` |
| `237` | `aws.ec2.SecurityGroup.Egress.IpProtocol.237` |
| `238` | `aws.ec2.SecurityGroup.Egress.IpProtocol.238` |
| `239` | `aws.ec2.SecurityGroup.Egress.IpProtocol.239` |
| `240` | `aws.ec2.SecurityGroup.Egress.IpProtocol.240` |
| `241` | `aws.ec2.SecurityGroup.Egress.IpProtocol.241` |
| `242` | `aws.ec2.SecurityGroup.Egress.IpProtocol.242` |
| `243` | `aws.ec2.SecurityGroup.Egress.IpProtocol.243` |
| `244` | `aws.ec2.SecurityGroup.Egress.IpProtocol.244` |
| `245` | `aws.ec2.SecurityGroup.Egress.IpProtocol.245` |
| `246` | `aws.ec2.SecurityGroup.Egress.IpProtocol.246` |
| `247` | `aws.ec2.SecurityGroup.Egress.IpProtocol.247` |
| `248` | `aws.ec2.SecurityGroup.Egress.IpProtocol.248` |
| `249` | `aws.ec2.SecurityGroup.Egress.IpProtocol.249` |
| `250` | `aws.ec2.SecurityGroup.Egress.IpProtocol.250` |
| `251` | `aws.ec2.SecurityGroup.Egress.IpProtocol.251` |
| `252` | `aws.ec2.SecurityGroup.Egress.IpProtocol.252` |
| `253` | `aws.ec2.SecurityGroup.Egress.IpProtocol.253` |
| `254` | `aws.ec2.SecurityGroup.Egress.IpProtocol.254` |
| `255` | `aws.ec2.SecurityGroup.Egress.IpProtocol.255` |

Shorthand formats: `tcp` or `IpProtocol.tcp`

### security_group_ingress.ip_protocol (IpProtocol)

| Value | DSL Identifier |
|-------|----------------|
| `tcp` | `aws.ec2.SecurityGroup.Ingress.IpProtocol.tcp` |
| `udp` | `aws.ec2.SecurityGroup.Ingress.IpProtocol.udp` |
| `icmp` | `aws.ec2.SecurityGroup.Ingress.IpProtocol.icmp` |
| `icmpv6` | `aws.ec2.SecurityGroup.Ingress.IpProtocol.icmpv6` |
| `-1` | `aws.ec2.SecurityGroup.Ingress.IpProtocol.all` |
| `0` | `aws.ec2.SecurityGroup.Ingress.IpProtocol.0` |
| `1` | `aws.ec2.SecurityGroup.Ingress.IpProtocol.1` |
| `2` | `aws.ec2.SecurityGroup.Ingress.IpProtocol.2` |
| `3` | `aws.ec2.SecurityGroup.Ingress.IpProtocol.3` |
| `4` | `aws.ec2.SecurityGroup.Ingress.IpProtocol.4` |
| `5` | `aws.ec2.SecurityGroup.Ingress.IpProtocol.5` |
| `6` | `aws.ec2.SecurityGroup.Ingress.IpProtocol.6` |
| `7` | `aws.ec2.SecurityGroup.Ingress.IpProtocol.7` |
| `8` | `aws.ec2.SecurityGroup.Ingress.IpProtocol.8` |
| `9` | `aws.ec2.SecurityGroup.Ingress.IpProtocol.9` |
| `10` | `aws.ec2.SecurityGroup.Ingress.IpProtocol.10` |
| `11` | `aws.ec2.SecurityGroup.Ingress.IpProtocol.11` |
| `12` | `aws.ec2.SecurityGroup.Ingress.IpProtocol.12` |
| `13` | `aws.ec2.SecurityGroup.Ingress.IpProtocol.13` |
| `14` | `aws.ec2.SecurityGroup.Ingress.IpProtocol.14` |
| `15` | `aws.ec2.SecurityGroup.Ingress.IpProtocol.15` |
| `16` | `aws.ec2.SecurityGroup.Ingress.IpProtocol.16` |
| `17` | `aws.ec2.SecurityGroup.Ingress.IpProtocol.17` |
| `18` | `aws.ec2.SecurityGroup.Ingress.IpProtocol.18` |
| `19` | `aws.ec2.SecurityGroup.Ingress.IpProtocol.19` |
| `20` | `aws.ec2.SecurityGroup.Ingress.IpProtocol.20` |
| `21` | `aws.ec2.SecurityGroup.Ingress.IpProtocol.21` |
| `22` | `aws.ec2.SecurityGroup.Ingress.IpProtocol.22` |
| `23` | `aws.ec2.SecurityGroup.Ingress.IpProtocol.23` |
| `24` | `aws.ec2.SecurityGroup.Ingress.IpProtocol.24` |
| `25` | `aws.ec2.SecurityGroup.Ingress.IpProtocol.25` |
| `26` | `aws.ec2.SecurityGroup.Ingress.IpProtocol.26` |
| `27` | `aws.ec2.SecurityGroup.Ingress.IpProtocol.27` |
| `28` | `aws.ec2.SecurityGroup.Ingress.IpProtocol.28` |
| `29` | `aws.ec2.SecurityGroup.Ingress.IpProtocol.29` |
| `30` | `aws.ec2.SecurityGroup.Ingress.IpProtocol.30` |
| `31` | `aws.ec2.SecurityGroup.Ingress.IpProtocol.31` |
| `32` | `aws.ec2.SecurityGroup.Ingress.IpProtocol.32` |
| `33` | `aws.ec2.SecurityGroup.Ingress.IpProtocol.33` |
| `34` | `aws.ec2.SecurityGroup.Ingress.IpProtocol.34` |
| `35` | `aws.ec2.SecurityGroup.Ingress.IpProtocol.35` |
| `36` | `aws.ec2.SecurityGroup.Ingress.IpProtocol.36` |
| `37` | `aws.ec2.SecurityGroup.Ingress.IpProtocol.37` |
| `38` | `aws.ec2.SecurityGroup.Ingress.IpProtocol.38` |
| `39` | `aws.ec2.SecurityGroup.Ingress.IpProtocol.39` |
| `40` | `aws.ec2.SecurityGroup.Ingress.IpProtocol.40` |
| `41` | `aws.ec2.SecurityGroup.Ingress.IpProtocol.41` |
| `42` | `aws.ec2.SecurityGroup.Ingress.IpProtocol.42` |
| `43` | `aws.ec2.SecurityGroup.Ingress.IpProtocol.43` |
| `44` | `aws.ec2.SecurityGroup.Ingress.IpProtocol.44` |
| `45` | `aws.ec2.SecurityGroup.Ingress.IpProtocol.45` |
| `46` | `aws.ec2.SecurityGroup.Ingress.IpProtocol.46` |
| `47` | `aws.ec2.SecurityGroup.Ingress.IpProtocol.47` |
| `48` | `aws.ec2.SecurityGroup.Ingress.IpProtocol.48` |
| `49` | `aws.ec2.SecurityGroup.Ingress.IpProtocol.49` |
| `50` | `aws.ec2.SecurityGroup.Ingress.IpProtocol.50` |
| `51` | `aws.ec2.SecurityGroup.Ingress.IpProtocol.51` |
| `52` | `aws.ec2.SecurityGroup.Ingress.IpProtocol.52` |
| `53` | `aws.ec2.SecurityGroup.Ingress.IpProtocol.53` |
| `54` | `aws.ec2.SecurityGroup.Ingress.IpProtocol.54` |
| `55` | `aws.ec2.SecurityGroup.Ingress.IpProtocol.55` |
| `56` | `aws.ec2.SecurityGroup.Ingress.IpProtocol.56` |
| `57` | `aws.ec2.SecurityGroup.Ingress.IpProtocol.57` |
| `58` | `aws.ec2.SecurityGroup.Ingress.IpProtocol.58` |
| `59` | `aws.ec2.SecurityGroup.Ingress.IpProtocol.59` |
| `60` | `aws.ec2.SecurityGroup.Ingress.IpProtocol.60` |
| `61` | `aws.ec2.SecurityGroup.Ingress.IpProtocol.61` |
| `62` | `aws.ec2.SecurityGroup.Ingress.IpProtocol.62` |
| `63` | `aws.ec2.SecurityGroup.Ingress.IpProtocol.63` |
| `64` | `aws.ec2.SecurityGroup.Ingress.IpProtocol.64` |
| `65` | `aws.ec2.SecurityGroup.Ingress.IpProtocol.65` |
| `66` | `aws.ec2.SecurityGroup.Ingress.IpProtocol.66` |
| `67` | `aws.ec2.SecurityGroup.Ingress.IpProtocol.67` |
| `68` | `aws.ec2.SecurityGroup.Ingress.IpProtocol.68` |
| `69` | `aws.ec2.SecurityGroup.Ingress.IpProtocol.69` |
| `70` | `aws.ec2.SecurityGroup.Ingress.IpProtocol.70` |
| `71` | `aws.ec2.SecurityGroup.Ingress.IpProtocol.71` |
| `72` | `aws.ec2.SecurityGroup.Ingress.IpProtocol.72` |
| `73` | `aws.ec2.SecurityGroup.Ingress.IpProtocol.73` |
| `74` | `aws.ec2.SecurityGroup.Ingress.IpProtocol.74` |
| `75` | `aws.ec2.SecurityGroup.Ingress.IpProtocol.75` |
| `76` | `aws.ec2.SecurityGroup.Ingress.IpProtocol.76` |
| `77` | `aws.ec2.SecurityGroup.Ingress.IpProtocol.77` |
| `78` | `aws.ec2.SecurityGroup.Ingress.IpProtocol.78` |
| `79` | `aws.ec2.SecurityGroup.Ingress.IpProtocol.79` |
| `80` | `aws.ec2.SecurityGroup.Ingress.IpProtocol.80` |
| `81` | `aws.ec2.SecurityGroup.Ingress.IpProtocol.81` |
| `82` | `aws.ec2.SecurityGroup.Ingress.IpProtocol.82` |
| `83` | `aws.ec2.SecurityGroup.Ingress.IpProtocol.83` |
| `84` | `aws.ec2.SecurityGroup.Ingress.IpProtocol.84` |
| `85` | `aws.ec2.SecurityGroup.Ingress.IpProtocol.85` |
| `86` | `aws.ec2.SecurityGroup.Ingress.IpProtocol.86` |
| `87` | `aws.ec2.SecurityGroup.Ingress.IpProtocol.87` |
| `88` | `aws.ec2.SecurityGroup.Ingress.IpProtocol.88` |
| `89` | `aws.ec2.SecurityGroup.Ingress.IpProtocol.89` |
| `90` | `aws.ec2.SecurityGroup.Ingress.IpProtocol.90` |
| `91` | `aws.ec2.SecurityGroup.Ingress.IpProtocol.91` |
| `92` | `aws.ec2.SecurityGroup.Ingress.IpProtocol.92` |
| `93` | `aws.ec2.SecurityGroup.Ingress.IpProtocol.93` |
| `94` | `aws.ec2.SecurityGroup.Ingress.IpProtocol.94` |
| `95` | `aws.ec2.SecurityGroup.Ingress.IpProtocol.95` |
| `96` | `aws.ec2.SecurityGroup.Ingress.IpProtocol.96` |
| `97` | `aws.ec2.SecurityGroup.Ingress.IpProtocol.97` |
| `98` | `aws.ec2.SecurityGroup.Ingress.IpProtocol.98` |
| `99` | `aws.ec2.SecurityGroup.Ingress.IpProtocol.99` |
| `100` | `aws.ec2.SecurityGroup.Ingress.IpProtocol.100` |
| `101` | `aws.ec2.SecurityGroup.Ingress.IpProtocol.101` |
| `102` | `aws.ec2.SecurityGroup.Ingress.IpProtocol.102` |
| `103` | `aws.ec2.SecurityGroup.Ingress.IpProtocol.103` |
| `104` | `aws.ec2.SecurityGroup.Ingress.IpProtocol.104` |
| `105` | `aws.ec2.SecurityGroup.Ingress.IpProtocol.105` |
| `106` | `aws.ec2.SecurityGroup.Ingress.IpProtocol.106` |
| `107` | `aws.ec2.SecurityGroup.Ingress.IpProtocol.107` |
| `108` | `aws.ec2.SecurityGroup.Ingress.IpProtocol.108` |
| `109` | `aws.ec2.SecurityGroup.Ingress.IpProtocol.109` |
| `110` | `aws.ec2.SecurityGroup.Ingress.IpProtocol.110` |
| `111` | `aws.ec2.SecurityGroup.Ingress.IpProtocol.111` |
| `112` | `aws.ec2.SecurityGroup.Ingress.IpProtocol.112` |
| `113` | `aws.ec2.SecurityGroup.Ingress.IpProtocol.113` |
| `114` | `aws.ec2.SecurityGroup.Ingress.IpProtocol.114` |
| `115` | `aws.ec2.SecurityGroup.Ingress.IpProtocol.115` |
| `116` | `aws.ec2.SecurityGroup.Ingress.IpProtocol.116` |
| `117` | `aws.ec2.SecurityGroup.Ingress.IpProtocol.117` |
| `118` | `aws.ec2.SecurityGroup.Ingress.IpProtocol.118` |
| `119` | `aws.ec2.SecurityGroup.Ingress.IpProtocol.119` |
| `120` | `aws.ec2.SecurityGroup.Ingress.IpProtocol.120` |
| `121` | `aws.ec2.SecurityGroup.Ingress.IpProtocol.121` |
| `122` | `aws.ec2.SecurityGroup.Ingress.IpProtocol.122` |
| `123` | `aws.ec2.SecurityGroup.Ingress.IpProtocol.123` |
| `124` | `aws.ec2.SecurityGroup.Ingress.IpProtocol.124` |
| `125` | `aws.ec2.SecurityGroup.Ingress.IpProtocol.125` |
| `126` | `aws.ec2.SecurityGroup.Ingress.IpProtocol.126` |
| `127` | `aws.ec2.SecurityGroup.Ingress.IpProtocol.127` |
| `128` | `aws.ec2.SecurityGroup.Ingress.IpProtocol.128` |
| `129` | `aws.ec2.SecurityGroup.Ingress.IpProtocol.129` |
| `130` | `aws.ec2.SecurityGroup.Ingress.IpProtocol.130` |
| `131` | `aws.ec2.SecurityGroup.Ingress.IpProtocol.131` |
| `132` | `aws.ec2.SecurityGroup.Ingress.IpProtocol.132` |
| `133` | `aws.ec2.SecurityGroup.Ingress.IpProtocol.133` |
| `134` | `aws.ec2.SecurityGroup.Ingress.IpProtocol.134` |
| `135` | `aws.ec2.SecurityGroup.Ingress.IpProtocol.135` |
| `136` | `aws.ec2.SecurityGroup.Ingress.IpProtocol.136` |
| `137` | `aws.ec2.SecurityGroup.Ingress.IpProtocol.137` |
| `138` | `aws.ec2.SecurityGroup.Ingress.IpProtocol.138` |
| `139` | `aws.ec2.SecurityGroup.Ingress.IpProtocol.139` |
| `140` | `aws.ec2.SecurityGroup.Ingress.IpProtocol.140` |
| `141` | `aws.ec2.SecurityGroup.Ingress.IpProtocol.141` |
| `142` | `aws.ec2.SecurityGroup.Ingress.IpProtocol.142` |
| `143` | `aws.ec2.SecurityGroup.Ingress.IpProtocol.143` |
| `144` | `aws.ec2.SecurityGroup.Ingress.IpProtocol.144` |
| `145` | `aws.ec2.SecurityGroup.Ingress.IpProtocol.145` |
| `146` | `aws.ec2.SecurityGroup.Ingress.IpProtocol.146` |
| `147` | `aws.ec2.SecurityGroup.Ingress.IpProtocol.147` |
| `148` | `aws.ec2.SecurityGroup.Ingress.IpProtocol.148` |
| `149` | `aws.ec2.SecurityGroup.Ingress.IpProtocol.149` |
| `150` | `aws.ec2.SecurityGroup.Ingress.IpProtocol.150` |
| `151` | `aws.ec2.SecurityGroup.Ingress.IpProtocol.151` |
| `152` | `aws.ec2.SecurityGroup.Ingress.IpProtocol.152` |
| `153` | `aws.ec2.SecurityGroup.Ingress.IpProtocol.153` |
| `154` | `aws.ec2.SecurityGroup.Ingress.IpProtocol.154` |
| `155` | `aws.ec2.SecurityGroup.Ingress.IpProtocol.155` |
| `156` | `aws.ec2.SecurityGroup.Ingress.IpProtocol.156` |
| `157` | `aws.ec2.SecurityGroup.Ingress.IpProtocol.157` |
| `158` | `aws.ec2.SecurityGroup.Ingress.IpProtocol.158` |
| `159` | `aws.ec2.SecurityGroup.Ingress.IpProtocol.159` |
| `160` | `aws.ec2.SecurityGroup.Ingress.IpProtocol.160` |
| `161` | `aws.ec2.SecurityGroup.Ingress.IpProtocol.161` |
| `162` | `aws.ec2.SecurityGroup.Ingress.IpProtocol.162` |
| `163` | `aws.ec2.SecurityGroup.Ingress.IpProtocol.163` |
| `164` | `aws.ec2.SecurityGroup.Ingress.IpProtocol.164` |
| `165` | `aws.ec2.SecurityGroup.Ingress.IpProtocol.165` |
| `166` | `aws.ec2.SecurityGroup.Ingress.IpProtocol.166` |
| `167` | `aws.ec2.SecurityGroup.Ingress.IpProtocol.167` |
| `168` | `aws.ec2.SecurityGroup.Ingress.IpProtocol.168` |
| `169` | `aws.ec2.SecurityGroup.Ingress.IpProtocol.169` |
| `170` | `aws.ec2.SecurityGroup.Ingress.IpProtocol.170` |
| `171` | `aws.ec2.SecurityGroup.Ingress.IpProtocol.171` |
| `172` | `aws.ec2.SecurityGroup.Ingress.IpProtocol.172` |
| `173` | `aws.ec2.SecurityGroup.Ingress.IpProtocol.173` |
| `174` | `aws.ec2.SecurityGroup.Ingress.IpProtocol.174` |
| `175` | `aws.ec2.SecurityGroup.Ingress.IpProtocol.175` |
| `176` | `aws.ec2.SecurityGroup.Ingress.IpProtocol.176` |
| `177` | `aws.ec2.SecurityGroup.Ingress.IpProtocol.177` |
| `178` | `aws.ec2.SecurityGroup.Ingress.IpProtocol.178` |
| `179` | `aws.ec2.SecurityGroup.Ingress.IpProtocol.179` |
| `180` | `aws.ec2.SecurityGroup.Ingress.IpProtocol.180` |
| `181` | `aws.ec2.SecurityGroup.Ingress.IpProtocol.181` |
| `182` | `aws.ec2.SecurityGroup.Ingress.IpProtocol.182` |
| `183` | `aws.ec2.SecurityGroup.Ingress.IpProtocol.183` |
| `184` | `aws.ec2.SecurityGroup.Ingress.IpProtocol.184` |
| `185` | `aws.ec2.SecurityGroup.Ingress.IpProtocol.185` |
| `186` | `aws.ec2.SecurityGroup.Ingress.IpProtocol.186` |
| `187` | `aws.ec2.SecurityGroup.Ingress.IpProtocol.187` |
| `188` | `aws.ec2.SecurityGroup.Ingress.IpProtocol.188` |
| `189` | `aws.ec2.SecurityGroup.Ingress.IpProtocol.189` |
| `190` | `aws.ec2.SecurityGroup.Ingress.IpProtocol.190` |
| `191` | `aws.ec2.SecurityGroup.Ingress.IpProtocol.191` |
| `192` | `aws.ec2.SecurityGroup.Ingress.IpProtocol.192` |
| `193` | `aws.ec2.SecurityGroup.Ingress.IpProtocol.193` |
| `194` | `aws.ec2.SecurityGroup.Ingress.IpProtocol.194` |
| `195` | `aws.ec2.SecurityGroup.Ingress.IpProtocol.195` |
| `196` | `aws.ec2.SecurityGroup.Ingress.IpProtocol.196` |
| `197` | `aws.ec2.SecurityGroup.Ingress.IpProtocol.197` |
| `198` | `aws.ec2.SecurityGroup.Ingress.IpProtocol.198` |
| `199` | `aws.ec2.SecurityGroup.Ingress.IpProtocol.199` |
| `200` | `aws.ec2.SecurityGroup.Ingress.IpProtocol.200` |
| `201` | `aws.ec2.SecurityGroup.Ingress.IpProtocol.201` |
| `202` | `aws.ec2.SecurityGroup.Ingress.IpProtocol.202` |
| `203` | `aws.ec2.SecurityGroup.Ingress.IpProtocol.203` |
| `204` | `aws.ec2.SecurityGroup.Ingress.IpProtocol.204` |
| `205` | `aws.ec2.SecurityGroup.Ingress.IpProtocol.205` |
| `206` | `aws.ec2.SecurityGroup.Ingress.IpProtocol.206` |
| `207` | `aws.ec2.SecurityGroup.Ingress.IpProtocol.207` |
| `208` | `aws.ec2.SecurityGroup.Ingress.IpProtocol.208` |
| `209` | `aws.ec2.SecurityGroup.Ingress.IpProtocol.209` |
| `210` | `aws.ec2.SecurityGroup.Ingress.IpProtocol.210` |
| `211` | `aws.ec2.SecurityGroup.Ingress.IpProtocol.211` |
| `212` | `aws.ec2.SecurityGroup.Ingress.IpProtocol.212` |
| `213` | `aws.ec2.SecurityGroup.Ingress.IpProtocol.213` |
| `214` | `aws.ec2.SecurityGroup.Ingress.IpProtocol.214` |
| `215` | `aws.ec2.SecurityGroup.Ingress.IpProtocol.215` |
| `216` | `aws.ec2.SecurityGroup.Ingress.IpProtocol.216` |
| `217` | `aws.ec2.SecurityGroup.Ingress.IpProtocol.217` |
| `218` | `aws.ec2.SecurityGroup.Ingress.IpProtocol.218` |
| `219` | `aws.ec2.SecurityGroup.Ingress.IpProtocol.219` |
| `220` | `aws.ec2.SecurityGroup.Ingress.IpProtocol.220` |
| `221` | `aws.ec2.SecurityGroup.Ingress.IpProtocol.221` |
| `222` | `aws.ec2.SecurityGroup.Ingress.IpProtocol.222` |
| `223` | `aws.ec2.SecurityGroup.Ingress.IpProtocol.223` |
| `224` | `aws.ec2.SecurityGroup.Ingress.IpProtocol.224` |
| `225` | `aws.ec2.SecurityGroup.Ingress.IpProtocol.225` |
| `226` | `aws.ec2.SecurityGroup.Ingress.IpProtocol.226` |
| `227` | `aws.ec2.SecurityGroup.Ingress.IpProtocol.227` |
| `228` | `aws.ec2.SecurityGroup.Ingress.IpProtocol.228` |
| `229` | `aws.ec2.SecurityGroup.Ingress.IpProtocol.229` |
| `230` | `aws.ec2.SecurityGroup.Ingress.IpProtocol.230` |
| `231` | `aws.ec2.SecurityGroup.Ingress.IpProtocol.231` |
| `232` | `aws.ec2.SecurityGroup.Ingress.IpProtocol.232` |
| `233` | `aws.ec2.SecurityGroup.Ingress.IpProtocol.233` |
| `234` | `aws.ec2.SecurityGroup.Ingress.IpProtocol.234` |
| `235` | `aws.ec2.SecurityGroup.Ingress.IpProtocol.235` |
| `236` | `aws.ec2.SecurityGroup.Ingress.IpProtocol.236` |
| `237` | `aws.ec2.SecurityGroup.Ingress.IpProtocol.237` |
| `238` | `aws.ec2.SecurityGroup.Ingress.IpProtocol.238` |
| `239` | `aws.ec2.SecurityGroup.Ingress.IpProtocol.239` |
| `240` | `aws.ec2.SecurityGroup.Ingress.IpProtocol.240` |
| `241` | `aws.ec2.SecurityGroup.Ingress.IpProtocol.241` |
| `242` | `aws.ec2.SecurityGroup.Ingress.IpProtocol.242` |
| `243` | `aws.ec2.SecurityGroup.Ingress.IpProtocol.243` |
| `244` | `aws.ec2.SecurityGroup.Ingress.IpProtocol.244` |
| `245` | `aws.ec2.SecurityGroup.Ingress.IpProtocol.245` |
| `246` | `aws.ec2.SecurityGroup.Ingress.IpProtocol.246` |
| `247` | `aws.ec2.SecurityGroup.Ingress.IpProtocol.247` |
| `248` | `aws.ec2.SecurityGroup.Ingress.IpProtocol.248` |
| `249` | `aws.ec2.SecurityGroup.Ingress.IpProtocol.249` |
| `250` | `aws.ec2.SecurityGroup.Ingress.IpProtocol.250` |
| `251` | `aws.ec2.SecurityGroup.Ingress.IpProtocol.251` |
| `252` | `aws.ec2.SecurityGroup.Ingress.IpProtocol.252` |
| `253` | `aws.ec2.SecurityGroup.Ingress.IpProtocol.253` |
| `254` | `aws.ec2.SecurityGroup.Ingress.IpProtocol.254` |
| `255` | `aws.ec2.SecurityGroup.Ingress.IpProtocol.255` |

Shorthand formats: `tcp` or `IpProtocol.tcp`

## Struct Definitions

### security_group_egress

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `cidr_ip` | `Ipv4Cidr` | No |  |
| `cidr_ipv6` | `Ipv6Cidr` | No |  |
| `description` | `String(pattern, len: ..=255)` | No |  |
| `destination_prefix_list_id` | `aws.ec2.PrefixList.Id` | No |  |
| `destination_security_group_id` | `aws.ec2.SecurityGroup.Id` | No |  |
| `from_port` | Int(-1..=65535) | No |  |
| `ip_protocol` | [Enum (IpProtocol)](#security_group_egressip_protocol-ipprotocol) | Yes |  |
| `to_port` | Int(-1..=65535) | No |  |

### security_group_ingress

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `cidr_ip` | `Ipv4Cidr` | No |  |
| `cidr_ipv6` | `Ipv6Cidr` | No |  |
| `description` | `String(pattern, len: ..=255)` | No |  |
| `from_port` | Int(-1..=65535) | No |  |
| `ip_protocol` | [Enum (IpProtocol)](#security_group_ingressip_protocol-ipprotocol) | Yes |  |
| `source_prefix_list_id` | `aws.ec2.PrefixList.Id` | No |  |
| `source_security_group_id` | `aws.ec2.SecurityGroup.Id` | No |  |
| `source_security_group_name` | String | No |  |
| `source_security_group_owner_id` | `aws.AccountId` | No |  |
| `to_port` | Int(-1..=65535) | No |  |

## Attribute Reference

### `group_id`

- **Type:** `aws.ec2.SecurityGroup.Id`

The group ID of the specified security group.

### `id`

- **Type:** `aws.ec2.SecurityGroup.Id`

The group name or group ID depending on whether the SG is created in default or specific VPC

## AWS API narrowings

These fields are narrower than or equal to CFN. Input rules combine documented AWS domains, recorded live observations, and Carina canonical spellings for stable state comparison. A canonical DSL spelling does not claim AWS rejects other spellings. Attaching a type name adds no structural constraints. Field names below use CFN spelling.

| CFN field | Relation to CFN | Evidence URL | Verbatim quote / exact observation | Checked / observed |
| --- | --- | --- | --- | --- |
| `/definitions/Egress/properties/Description` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/AWSCloudFormation/latest/TemplateReference/aws-properties-ec2-securitygroup-egress.html#cfn-ec2-securitygroup-egress-description) | ` A description for the security group rule. `<br>` Constraints: Up to 255 characters in length. Allowed characters are a-z, A-Z, 0-9, spaces, and ._-:/()#,@[]+=;{}!$* ` | 2026-11-07 |
| `/definitions/Egress/properties/FromPort` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/managedservices/latest/ctref/schemas.html) | ` The start of allowed port range, from 0 to 65535 for TCP/UDP. For ICMP, use -1. ` | 2026-11-07 |
| `/definitions/Egress/properties/FromPort` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/AWSEC2/latest/APIReference/API_IpPermission.html) | ` If the protocol is ICMP or ICMPv6, this is the ICMP type or -1 (all ICMP types). ` | 2026-11-07 |
| `/definitions/Egress/properties/IpProtocol` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/AWSEC2/latest/APIReference/API_IpPermission.html) | ` The IP protocol name (tcp, udp, icmp, icmpv6) or number (see Protocol Numbers). `<br>` Use -1 to specify all protocols. ` | 2026-11-07 |
| `/definitions/Egress/properties/IpProtocol` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/network-firewall/latest/APIReference/API_MatchAttributes.html#networkfirewall-Type-MatchAttributes-Protocols) | ` The protocols to inspect for, specified using the assigned internet protocol number (IANA) for each protocol. `<br>` Valid Range: Minimum value of 0. Maximum value of 255. ` | 2026-11-07 |
| `/definitions/Egress/properties/ToPort` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/managedservices/latest/ctref/schemas.html) | ` The end of allowed port range, from 0 to 65535 for TCP/UDP. For ICMP, use -1. ` | 2026-11-07 |
| `/definitions/Egress/properties/ToPort` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/AWSEC2/latest/APIReference/API_IpPermission.html) | ` If the protocol is ICMP or ICMPv6, this is the ICMP type or -1 (all ICMP types). ` | 2026-11-07 |
| `/definitions/Ingress/properties/Description` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/AWSCloudFormation/latest/TemplateReference/aws-properties-ec2-securitygroup-ingress.html#cfn-ec2-securitygroup-ingress-description) | ` Updates the description of an ingress (inbound) security group rule. You can replace an existing description, or add a description to a rule that did not have one previously. `<br>` Constraints: Up to 255 characters in length. Allowed characters are a-z, A-Z, 0-9, spaces, and ._-:/()#,@[]+=;{}!$* ` | 2026-11-07 |
| `/definitions/Ingress/properties/FromPort` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/managedservices/latest/ctref/schemas.html) | ` The start of allowed port range, from 0 to 65535 for TCP/UDP. For ICMP, use -1. ` | 2026-11-07 |
| `/definitions/Ingress/properties/FromPort` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/AWSEC2/latest/APIReference/API_IpPermission.html) | ` If the protocol is ICMP or ICMPv6, this is the ICMP type or -1 (all ICMP types). ` | 2026-11-07 |
| `/definitions/Ingress/properties/IpProtocol` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/AWSEC2/latest/APIReference/API_IpPermission.html) | ` The IP protocol name (tcp, udp, icmp, icmpv6) or number (see Protocol Numbers). `<br>` Use -1 to specify all protocols. ` | 2026-11-07 |
| `/definitions/Ingress/properties/IpProtocol` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/network-firewall/latest/APIReference/API_MatchAttributes.html#networkfirewall-Type-MatchAttributes-Protocols) | ` The protocols to inspect for, specified using the assigned internet protocol number (IANA) for each protocol. `<br>` Valid Range: Minimum value of 0. Maximum value of 255. ` | 2026-11-07 |
| `/definitions/Ingress/properties/ToPort` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/managedservices/latest/ctref/schemas.html) | ` The end of allowed port range, from 0 to 65535 for TCP/UDP. For ICMP, use -1. ` | 2026-11-07 |
| `/definitions/Ingress/properties/ToPort` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/AWSEC2/latest/APIReference/API_IpPermission.html) | ` If the protocol is ICMP or ICMPv6, this is the ICMP type or -1 (all ICMP types). ` | 2026-11-07 |
| `/definitions/Tag/properties/Key` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/AWSEC2/latest/UserGuide/Using_Tags.html) | ` Maximum key length – 128 Unicode characters in UTF-8 ` | 2026-11-07 |
| `/definitions/Tag/properties/Value` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/AWSEC2/latest/APIReference/API_Tag.html) | ` The value of the tag. `<br>` Constraints: Tag values are case-sensitive and accept a maximum of 256 Unicode characters. ` | 2026-11-07 |
| `/properties/GroupDescription` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/AWSCloudFormation/latest/TemplateReference/aws-resource-ec2-securitygroup.html#cfn-ec2-securitygroup-groupdescription) | ` A description for the security group. `<br>` Constraints: Up to 255 characters in length `<br>` Valid characters: a-z, A-Z, 0-9, spaces, and ._-:/()#,@[]+=&;{}!$* ` | 2026-11-07 |

