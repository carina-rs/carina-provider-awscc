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

- **Type:** `String (pattern: /^[a-zA-Z0-9. _:/()#,@\[\]+=&;{}!$*-]*$/ (Full), len: unbounded..=255)`
- **Required:** Yes
- **Create-only:** Yes

A description for the security group.

### `group_name`

- **Type:** `String`
- **Required:** No
- **Create-only:** Yes

The name of the security group.

### `security_group_egress`

- **Type:** List\<[Struct(Egress)](#security_group_egress-1)\> (unordered)
- **Required:** No

[VPC only] The outbound rules associated with the security group. There is a short interruption during which you cannot connect to the security group.

### `security_group_ingress`

- **Type:** List\<[Struct(Ingress)](#security_group_ingress-1)\> (unordered)
- **Required:** No

The inbound rules associated with the security group. There is a short interruption during which you cannot connect to the security group.

### `tags`

- **Type:** Map\<`String (len: unbounded..=128)`, `String (len: unbounded..=256)`\>
- **Required:** No

Any tags assigned to the security group.

### `vpc_id`

- **Type:** `awscc.ec2.Vpc.Id (pattern: /^vpc-[0-9a-f]{8,}$/ (Full))`
- **Required:** No
- **Create-only:** Yes

The ID of the VPC for the security group.

## Enum Values

### security_group_egress.ip_protocol (IpProtocol)

| Value | DSL Identifier |
|-------|----------------|
| `tcp` | `awscc.ec2.SecurityGroup.Egress.IpProtocol.tcp` |
| `udp` | `awscc.ec2.SecurityGroup.Egress.IpProtocol.udp` |
| `icmp` | `awscc.ec2.SecurityGroup.Egress.IpProtocol.icmp` |
| `icmpv6` | `awscc.ec2.SecurityGroup.Egress.IpProtocol.icmpv6` |
| `-1` | `awscc.ec2.SecurityGroup.Egress.IpProtocol.all` |
| `0` | `awscc.ec2.SecurityGroup.Egress.IpProtocol.0` |
| `1` | `awscc.ec2.SecurityGroup.Egress.IpProtocol.1` |
| `2` | `awscc.ec2.SecurityGroup.Egress.IpProtocol.2` |
| `3` | `awscc.ec2.SecurityGroup.Egress.IpProtocol.3` |
| `4` | `awscc.ec2.SecurityGroup.Egress.IpProtocol.4` |
| `5` | `awscc.ec2.SecurityGroup.Egress.IpProtocol.5` |
| `6` | `awscc.ec2.SecurityGroup.Egress.IpProtocol.6` |
| `7` | `awscc.ec2.SecurityGroup.Egress.IpProtocol.7` |
| `8` | `awscc.ec2.SecurityGroup.Egress.IpProtocol.8` |
| `9` | `awscc.ec2.SecurityGroup.Egress.IpProtocol.9` |
| `10` | `awscc.ec2.SecurityGroup.Egress.IpProtocol.10` |
| `11` | `awscc.ec2.SecurityGroup.Egress.IpProtocol.11` |
| `12` | `awscc.ec2.SecurityGroup.Egress.IpProtocol.12` |
| `13` | `awscc.ec2.SecurityGroup.Egress.IpProtocol.13` |
| `14` | `awscc.ec2.SecurityGroup.Egress.IpProtocol.14` |
| `15` | `awscc.ec2.SecurityGroup.Egress.IpProtocol.15` |
| `16` | `awscc.ec2.SecurityGroup.Egress.IpProtocol.16` |
| `17` | `awscc.ec2.SecurityGroup.Egress.IpProtocol.17` |
| `18` | `awscc.ec2.SecurityGroup.Egress.IpProtocol.18` |
| `19` | `awscc.ec2.SecurityGroup.Egress.IpProtocol.19` |
| `20` | `awscc.ec2.SecurityGroup.Egress.IpProtocol.20` |
| `21` | `awscc.ec2.SecurityGroup.Egress.IpProtocol.21` |
| `22` | `awscc.ec2.SecurityGroup.Egress.IpProtocol.22` |
| `23` | `awscc.ec2.SecurityGroup.Egress.IpProtocol.23` |
| `24` | `awscc.ec2.SecurityGroup.Egress.IpProtocol.24` |
| `25` | `awscc.ec2.SecurityGroup.Egress.IpProtocol.25` |
| `26` | `awscc.ec2.SecurityGroup.Egress.IpProtocol.26` |
| `27` | `awscc.ec2.SecurityGroup.Egress.IpProtocol.27` |
| `28` | `awscc.ec2.SecurityGroup.Egress.IpProtocol.28` |
| `29` | `awscc.ec2.SecurityGroup.Egress.IpProtocol.29` |
| `30` | `awscc.ec2.SecurityGroup.Egress.IpProtocol.30` |
| `31` | `awscc.ec2.SecurityGroup.Egress.IpProtocol.31` |
| `32` | `awscc.ec2.SecurityGroup.Egress.IpProtocol.32` |
| `33` | `awscc.ec2.SecurityGroup.Egress.IpProtocol.33` |
| `34` | `awscc.ec2.SecurityGroup.Egress.IpProtocol.34` |
| `35` | `awscc.ec2.SecurityGroup.Egress.IpProtocol.35` |
| `36` | `awscc.ec2.SecurityGroup.Egress.IpProtocol.36` |
| `37` | `awscc.ec2.SecurityGroup.Egress.IpProtocol.37` |
| `38` | `awscc.ec2.SecurityGroup.Egress.IpProtocol.38` |
| `39` | `awscc.ec2.SecurityGroup.Egress.IpProtocol.39` |
| `40` | `awscc.ec2.SecurityGroup.Egress.IpProtocol.40` |
| `41` | `awscc.ec2.SecurityGroup.Egress.IpProtocol.41` |
| `42` | `awscc.ec2.SecurityGroup.Egress.IpProtocol.42` |
| `43` | `awscc.ec2.SecurityGroup.Egress.IpProtocol.43` |
| `44` | `awscc.ec2.SecurityGroup.Egress.IpProtocol.44` |
| `45` | `awscc.ec2.SecurityGroup.Egress.IpProtocol.45` |
| `46` | `awscc.ec2.SecurityGroup.Egress.IpProtocol.46` |
| `47` | `awscc.ec2.SecurityGroup.Egress.IpProtocol.47` |
| `48` | `awscc.ec2.SecurityGroup.Egress.IpProtocol.48` |
| `49` | `awscc.ec2.SecurityGroup.Egress.IpProtocol.49` |
| `50` | `awscc.ec2.SecurityGroup.Egress.IpProtocol.50` |
| `51` | `awscc.ec2.SecurityGroup.Egress.IpProtocol.51` |
| `52` | `awscc.ec2.SecurityGroup.Egress.IpProtocol.52` |
| `53` | `awscc.ec2.SecurityGroup.Egress.IpProtocol.53` |
| `54` | `awscc.ec2.SecurityGroup.Egress.IpProtocol.54` |
| `55` | `awscc.ec2.SecurityGroup.Egress.IpProtocol.55` |
| `56` | `awscc.ec2.SecurityGroup.Egress.IpProtocol.56` |
| `57` | `awscc.ec2.SecurityGroup.Egress.IpProtocol.57` |
| `58` | `awscc.ec2.SecurityGroup.Egress.IpProtocol.58` |
| `59` | `awscc.ec2.SecurityGroup.Egress.IpProtocol.59` |
| `60` | `awscc.ec2.SecurityGroup.Egress.IpProtocol.60` |
| `61` | `awscc.ec2.SecurityGroup.Egress.IpProtocol.61` |
| `62` | `awscc.ec2.SecurityGroup.Egress.IpProtocol.62` |
| `63` | `awscc.ec2.SecurityGroup.Egress.IpProtocol.63` |
| `64` | `awscc.ec2.SecurityGroup.Egress.IpProtocol.64` |
| `65` | `awscc.ec2.SecurityGroup.Egress.IpProtocol.65` |
| `66` | `awscc.ec2.SecurityGroup.Egress.IpProtocol.66` |
| `67` | `awscc.ec2.SecurityGroup.Egress.IpProtocol.67` |
| `68` | `awscc.ec2.SecurityGroup.Egress.IpProtocol.68` |
| `69` | `awscc.ec2.SecurityGroup.Egress.IpProtocol.69` |
| `70` | `awscc.ec2.SecurityGroup.Egress.IpProtocol.70` |
| `71` | `awscc.ec2.SecurityGroup.Egress.IpProtocol.71` |
| `72` | `awscc.ec2.SecurityGroup.Egress.IpProtocol.72` |
| `73` | `awscc.ec2.SecurityGroup.Egress.IpProtocol.73` |
| `74` | `awscc.ec2.SecurityGroup.Egress.IpProtocol.74` |
| `75` | `awscc.ec2.SecurityGroup.Egress.IpProtocol.75` |
| `76` | `awscc.ec2.SecurityGroup.Egress.IpProtocol.76` |
| `77` | `awscc.ec2.SecurityGroup.Egress.IpProtocol.77` |
| `78` | `awscc.ec2.SecurityGroup.Egress.IpProtocol.78` |
| `79` | `awscc.ec2.SecurityGroup.Egress.IpProtocol.79` |
| `80` | `awscc.ec2.SecurityGroup.Egress.IpProtocol.80` |
| `81` | `awscc.ec2.SecurityGroup.Egress.IpProtocol.81` |
| `82` | `awscc.ec2.SecurityGroup.Egress.IpProtocol.82` |
| `83` | `awscc.ec2.SecurityGroup.Egress.IpProtocol.83` |
| `84` | `awscc.ec2.SecurityGroup.Egress.IpProtocol.84` |
| `85` | `awscc.ec2.SecurityGroup.Egress.IpProtocol.85` |
| `86` | `awscc.ec2.SecurityGroup.Egress.IpProtocol.86` |
| `87` | `awscc.ec2.SecurityGroup.Egress.IpProtocol.87` |
| `88` | `awscc.ec2.SecurityGroup.Egress.IpProtocol.88` |
| `89` | `awscc.ec2.SecurityGroup.Egress.IpProtocol.89` |
| `90` | `awscc.ec2.SecurityGroup.Egress.IpProtocol.90` |
| `91` | `awscc.ec2.SecurityGroup.Egress.IpProtocol.91` |
| `92` | `awscc.ec2.SecurityGroup.Egress.IpProtocol.92` |
| `93` | `awscc.ec2.SecurityGroup.Egress.IpProtocol.93` |
| `94` | `awscc.ec2.SecurityGroup.Egress.IpProtocol.94` |
| `95` | `awscc.ec2.SecurityGroup.Egress.IpProtocol.95` |
| `96` | `awscc.ec2.SecurityGroup.Egress.IpProtocol.96` |
| `97` | `awscc.ec2.SecurityGroup.Egress.IpProtocol.97` |
| `98` | `awscc.ec2.SecurityGroup.Egress.IpProtocol.98` |
| `99` | `awscc.ec2.SecurityGroup.Egress.IpProtocol.99` |
| `100` | `awscc.ec2.SecurityGroup.Egress.IpProtocol.100` |
| `101` | `awscc.ec2.SecurityGroup.Egress.IpProtocol.101` |
| `102` | `awscc.ec2.SecurityGroup.Egress.IpProtocol.102` |
| `103` | `awscc.ec2.SecurityGroup.Egress.IpProtocol.103` |
| `104` | `awscc.ec2.SecurityGroup.Egress.IpProtocol.104` |
| `105` | `awscc.ec2.SecurityGroup.Egress.IpProtocol.105` |
| `106` | `awscc.ec2.SecurityGroup.Egress.IpProtocol.106` |
| `107` | `awscc.ec2.SecurityGroup.Egress.IpProtocol.107` |
| `108` | `awscc.ec2.SecurityGroup.Egress.IpProtocol.108` |
| `109` | `awscc.ec2.SecurityGroup.Egress.IpProtocol.109` |
| `110` | `awscc.ec2.SecurityGroup.Egress.IpProtocol.110` |
| `111` | `awscc.ec2.SecurityGroup.Egress.IpProtocol.111` |
| `112` | `awscc.ec2.SecurityGroup.Egress.IpProtocol.112` |
| `113` | `awscc.ec2.SecurityGroup.Egress.IpProtocol.113` |
| `114` | `awscc.ec2.SecurityGroup.Egress.IpProtocol.114` |
| `115` | `awscc.ec2.SecurityGroup.Egress.IpProtocol.115` |
| `116` | `awscc.ec2.SecurityGroup.Egress.IpProtocol.116` |
| `117` | `awscc.ec2.SecurityGroup.Egress.IpProtocol.117` |
| `118` | `awscc.ec2.SecurityGroup.Egress.IpProtocol.118` |
| `119` | `awscc.ec2.SecurityGroup.Egress.IpProtocol.119` |
| `120` | `awscc.ec2.SecurityGroup.Egress.IpProtocol.120` |
| `121` | `awscc.ec2.SecurityGroup.Egress.IpProtocol.121` |
| `122` | `awscc.ec2.SecurityGroup.Egress.IpProtocol.122` |
| `123` | `awscc.ec2.SecurityGroup.Egress.IpProtocol.123` |
| `124` | `awscc.ec2.SecurityGroup.Egress.IpProtocol.124` |
| `125` | `awscc.ec2.SecurityGroup.Egress.IpProtocol.125` |
| `126` | `awscc.ec2.SecurityGroup.Egress.IpProtocol.126` |
| `127` | `awscc.ec2.SecurityGroup.Egress.IpProtocol.127` |
| `128` | `awscc.ec2.SecurityGroup.Egress.IpProtocol.128` |
| `129` | `awscc.ec2.SecurityGroup.Egress.IpProtocol.129` |
| `130` | `awscc.ec2.SecurityGroup.Egress.IpProtocol.130` |
| `131` | `awscc.ec2.SecurityGroup.Egress.IpProtocol.131` |
| `132` | `awscc.ec2.SecurityGroup.Egress.IpProtocol.132` |
| `133` | `awscc.ec2.SecurityGroup.Egress.IpProtocol.133` |
| `134` | `awscc.ec2.SecurityGroup.Egress.IpProtocol.134` |
| `135` | `awscc.ec2.SecurityGroup.Egress.IpProtocol.135` |
| `136` | `awscc.ec2.SecurityGroup.Egress.IpProtocol.136` |
| `137` | `awscc.ec2.SecurityGroup.Egress.IpProtocol.137` |
| `138` | `awscc.ec2.SecurityGroup.Egress.IpProtocol.138` |
| `139` | `awscc.ec2.SecurityGroup.Egress.IpProtocol.139` |
| `140` | `awscc.ec2.SecurityGroup.Egress.IpProtocol.140` |
| `141` | `awscc.ec2.SecurityGroup.Egress.IpProtocol.141` |
| `142` | `awscc.ec2.SecurityGroup.Egress.IpProtocol.142` |
| `143` | `awscc.ec2.SecurityGroup.Egress.IpProtocol.143` |
| `144` | `awscc.ec2.SecurityGroup.Egress.IpProtocol.144` |
| `145` | `awscc.ec2.SecurityGroup.Egress.IpProtocol.145` |
| `146` | `awscc.ec2.SecurityGroup.Egress.IpProtocol.146` |
| `147` | `awscc.ec2.SecurityGroup.Egress.IpProtocol.147` |
| `148` | `awscc.ec2.SecurityGroup.Egress.IpProtocol.148` |
| `149` | `awscc.ec2.SecurityGroup.Egress.IpProtocol.149` |
| `150` | `awscc.ec2.SecurityGroup.Egress.IpProtocol.150` |
| `151` | `awscc.ec2.SecurityGroup.Egress.IpProtocol.151` |
| `152` | `awscc.ec2.SecurityGroup.Egress.IpProtocol.152` |
| `153` | `awscc.ec2.SecurityGroup.Egress.IpProtocol.153` |
| `154` | `awscc.ec2.SecurityGroup.Egress.IpProtocol.154` |
| `155` | `awscc.ec2.SecurityGroup.Egress.IpProtocol.155` |
| `156` | `awscc.ec2.SecurityGroup.Egress.IpProtocol.156` |
| `157` | `awscc.ec2.SecurityGroup.Egress.IpProtocol.157` |
| `158` | `awscc.ec2.SecurityGroup.Egress.IpProtocol.158` |
| `159` | `awscc.ec2.SecurityGroup.Egress.IpProtocol.159` |
| `160` | `awscc.ec2.SecurityGroup.Egress.IpProtocol.160` |
| `161` | `awscc.ec2.SecurityGroup.Egress.IpProtocol.161` |
| `162` | `awscc.ec2.SecurityGroup.Egress.IpProtocol.162` |
| `163` | `awscc.ec2.SecurityGroup.Egress.IpProtocol.163` |
| `164` | `awscc.ec2.SecurityGroup.Egress.IpProtocol.164` |
| `165` | `awscc.ec2.SecurityGroup.Egress.IpProtocol.165` |
| `166` | `awscc.ec2.SecurityGroup.Egress.IpProtocol.166` |
| `167` | `awscc.ec2.SecurityGroup.Egress.IpProtocol.167` |
| `168` | `awscc.ec2.SecurityGroup.Egress.IpProtocol.168` |
| `169` | `awscc.ec2.SecurityGroup.Egress.IpProtocol.169` |
| `170` | `awscc.ec2.SecurityGroup.Egress.IpProtocol.170` |
| `171` | `awscc.ec2.SecurityGroup.Egress.IpProtocol.171` |
| `172` | `awscc.ec2.SecurityGroup.Egress.IpProtocol.172` |
| `173` | `awscc.ec2.SecurityGroup.Egress.IpProtocol.173` |
| `174` | `awscc.ec2.SecurityGroup.Egress.IpProtocol.174` |
| `175` | `awscc.ec2.SecurityGroup.Egress.IpProtocol.175` |
| `176` | `awscc.ec2.SecurityGroup.Egress.IpProtocol.176` |
| `177` | `awscc.ec2.SecurityGroup.Egress.IpProtocol.177` |
| `178` | `awscc.ec2.SecurityGroup.Egress.IpProtocol.178` |
| `179` | `awscc.ec2.SecurityGroup.Egress.IpProtocol.179` |
| `180` | `awscc.ec2.SecurityGroup.Egress.IpProtocol.180` |
| `181` | `awscc.ec2.SecurityGroup.Egress.IpProtocol.181` |
| `182` | `awscc.ec2.SecurityGroup.Egress.IpProtocol.182` |
| `183` | `awscc.ec2.SecurityGroup.Egress.IpProtocol.183` |
| `184` | `awscc.ec2.SecurityGroup.Egress.IpProtocol.184` |
| `185` | `awscc.ec2.SecurityGroup.Egress.IpProtocol.185` |
| `186` | `awscc.ec2.SecurityGroup.Egress.IpProtocol.186` |
| `187` | `awscc.ec2.SecurityGroup.Egress.IpProtocol.187` |
| `188` | `awscc.ec2.SecurityGroup.Egress.IpProtocol.188` |
| `189` | `awscc.ec2.SecurityGroup.Egress.IpProtocol.189` |
| `190` | `awscc.ec2.SecurityGroup.Egress.IpProtocol.190` |
| `191` | `awscc.ec2.SecurityGroup.Egress.IpProtocol.191` |
| `192` | `awscc.ec2.SecurityGroup.Egress.IpProtocol.192` |
| `193` | `awscc.ec2.SecurityGroup.Egress.IpProtocol.193` |
| `194` | `awscc.ec2.SecurityGroup.Egress.IpProtocol.194` |
| `195` | `awscc.ec2.SecurityGroup.Egress.IpProtocol.195` |
| `196` | `awscc.ec2.SecurityGroup.Egress.IpProtocol.196` |
| `197` | `awscc.ec2.SecurityGroup.Egress.IpProtocol.197` |
| `198` | `awscc.ec2.SecurityGroup.Egress.IpProtocol.198` |
| `199` | `awscc.ec2.SecurityGroup.Egress.IpProtocol.199` |
| `200` | `awscc.ec2.SecurityGroup.Egress.IpProtocol.200` |
| `201` | `awscc.ec2.SecurityGroup.Egress.IpProtocol.201` |
| `202` | `awscc.ec2.SecurityGroup.Egress.IpProtocol.202` |
| `203` | `awscc.ec2.SecurityGroup.Egress.IpProtocol.203` |
| `204` | `awscc.ec2.SecurityGroup.Egress.IpProtocol.204` |
| `205` | `awscc.ec2.SecurityGroup.Egress.IpProtocol.205` |
| `206` | `awscc.ec2.SecurityGroup.Egress.IpProtocol.206` |
| `207` | `awscc.ec2.SecurityGroup.Egress.IpProtocol.207` |
| `208` | `awscc.ec2.SecurityGroup.Egress.IpProtocol.208` |
| `209` | `awscc.ec2.SecurityGroup.Egress.IpProtocol.209` |
| `210` | `awscc.ec2.SecurityGroup.Egress.IpProtocol.210` |
| `211` | `awscc.ec2.SecurityGroup.Egress.IpProtocol.211` |
| `212` | `awscc.ec2.SecurityGroup.Egress.IpProtocol.212` |
| `213` | `awscc.ec2.SecurityGroup.Egress.IpProtocol.213` |
| `214` | `awscc.ec2.SecurityGroup.Egress.IpProtocol.214` |
| `215` | `awscc.ec2.SecurityGroup.Egress.IpProtocol.215` |
| `216` | `awscc.ec2.SecurityGroup.Egress.IpProtocol.216` |
| `217` | `awscc.ec2.SecurityGroup.Egress.IpProtocol.217` |
| `218` | `awscc.ec2.SecurityGroup.Egress.IpProtocol.218` |
| `219` | `awscc.ec2.SecurityGroup.Egress.IpProtocol.219` |
| `220` | `awscc.ec2.SecurityGroup.Egress.IpProtocol.220` |
| `221` | `awscc.ec2.SecurityGroup.Egress.IpProtocol.221` |
| `222` | `awscc.ec2.SecurityGroup.Egress.IpProtocol.222` |
| `223` | `awscc.ec2.SecurityGroup.Egress.IpProtocol.223` |
| `224` | `awscc.ec2.SecurityGroup.Egress.IpProtocol.224` |
| `225` | `awscc.ec2.SecurityGroup.Egress.IpProtocol.225` |
| `226` | `awscc.ec2.SecurityGroup.Egress.IpProtocol.226` |
| `227` | `awscc.ec2.SecurityGroup.Egress.IpProtocol.227` |
| `228` | `awscc.ec2.SecurityGroup.Egress.IpProtocol.228` |
| `229` | `awscc.ec2.SecurityGroup.Egress.IpProtocol.229` |
| `230` | `awscc.ec2.SecurityGroup.Egress.IpProtocol.230` |
| `231` | `awscc.ec2.SecurityGroup.Egress.IpProtocol.231` |
| `232` | `awscc.ec2.SecurityGroup.Egress.IpProtocol.232` |
| `233` | `awscc.ec2.SecurityGroup.Egress.IpProtocol.233` |
| `234` | `awscc.ec2.SecurityGroup.Egress.IpProtocol.234` |
| `235` | `awscc.ec2.SecurityGroup.Egress.IpProtocol.235` |
| `236` | `awscc.ec2.SecurityGroup.Egress.IpProtocol.236` |
| `237` | `awscc.ec2.SecurityGroup.Egress.IpProtocol.237` |
| `238` | `awscc.ec2.SecurityGroup.Egress.IpProtocol.238` |
| `239` | `awscc.ec2.SecurityGroup.Egress.IpProtocol.239` |
| `240` | `awscc.ec2.SecurityGroup.Egress.IpProtocol.240` |
| `241` | `awscc.ec2.SecurityGroup.Egress.IpProtocol.241` |
| `242` | `awscc.ec2.SecurityGroup.Egress.IpProtocol.242` |
| `243` | `awscc.ec2.SecurityGroup.Egress.IpProtocol.243` |
| `244` | `awscc.ec2.SecurityGroup.Egress.IpProtocol.244` |
| `245` | `awscc.ec2.SecurityGroup.Egress.IpProtocol.245` |
| `246` | `awscc.ec2.SecurityGroup.Egress.IpProtocol.246` |
| `247` | `awscc.ec2.SecurityGroup.Egress.IpProtocol.247` |
| `248` | `awscc.ec2.SecurityGroup.Egress.IpProtocol.248` |
| `249` | `awscc.ec2.SecurityGroup.Egress.IpProtocol.249` |
| `250` | `awscc.ec2.SecurityGroup.Egress.IpProtocol.250` |
| `251` | `awscc.ec2.SecurityGroup.Egress.IpProtocol.251` |
| `252` | `awscc.ec2.SecurityGroup.Egress.IpProtocol.252` |
| `253` | `awscc.ec2.SecurityGroup.Egress.IpProtocol.253` |
| `254` | `awscc.ec2.SecurityGroup.Egress.IpProtocol.254` |
| `255` | `awscc.ec2.SecurityGroup.Egress.IpProtocol.255` |

Shorthand formats: `tcp` or `IpProtocol.tcp`

### security_group_ingress.ip_protocol (IpProtocol)

| Value | DSL Identifier |
|-------|----------------|
| `tcp` | `awscc.ec2.SecurityGroup.Ingress.IpProtocol.tcp` |
| `udp` | `awscc.ec2.SecurityGroup.Ingress.IpProtocol.udp` |
| `icmp` | `awscc.ec2.SecurityGroup.Ingress.IpProtocol.icmp` |
| `icmpv6` | `awscc.ec2.SecurityGroup.Ingress.IpProtocol.icmpv6` |
| `-1` | `awscc.ec2.SecurityGroup.Ingress.IpProtocol.all` |
| `0` | `awscc.ec2.SecurityGroup.Ingress.IpProtocol.0` |
| `1` | `awscc.ec2.SecurityGroup.Ingress.IpProtocol.1` |
| `2` | `awscc.ec2.SecurityGroup.Ingress.IpProtocol.2` |
| `3` | `awscc.ec2.SecurityGroup.Ingress.IpProtocol.3` |
| `4` | `awscc.ec2.SecurityGroup.Ingress.IpProtocol.4` |
| `5` | `awscc.ec2.SecurityGroup.Ingress.IpProtocol.5` |
| `6` | `awscc.ec2.SecurityGroup.Ingress.IpProtocol.6` |
| `7` | `awscc.ec2.SecurityGroup.Ingress.IpProtocol.7` |
| `8` | `awscc.ec2.SecurityGroup.Ingress.IpProtocol.8` |
| `9` | `awscc.ec2.SecurityGroup.Ingress.IpProtocol.9` |
| `10` | `awscc.ec2.SecurityGroup.Ingress.IpProtocol.10` |
| `11` | `awscc.ec2.SecurityGroup.Ingress.IpProtocol.11` |
| `12` | `awscc.ec2.SecurityGroup.Ingress.IpProtocol.12` |
| `13` | `awscc.ec2.SecurityGroup.Ingress.IpProtocol.13` |
| `14` | `awscc.ec2.SecurityGroup.Ingress.IpProtocol.14` |
| `15` | `awscc.ec2.SecurityGroup.Ingress.IpProtocol.15` |
| `16` | `awscc.ec2.SecurityGroup.Ingress.IpProtocol.16` |
| `17` | `awscc.ec2.SecurityGroup.Ingress.IpProtocol.17` |
| `18` | `awscc.ec2.SecurityGroup.Ingress.IpProtocol.18` |
| `19` | `awscc.ec2.SecurityGroup.Ingress.IpProtocol.19` |
| `20` | `awscc.ec2.SecurityGroup.Ingress.IpProtocol.20` |
| `21` | `awscc.ec2.SecurityGroup.Ingress.IpProtocol.21` |
| `22` | `awscc.ec2.SecurityGroup.Ingress.IpProtocol.22` |
| `23` | `awscc.ec2.SecurityGroup.Ingress.IpProtocol.23` |
| `24` | `awscc.ec2.SecurityGroup.Ingress.IpProtocol.24` |
| `25` | `awscc.ec2.SecurityGroup.Ingress.IpProtocol.25` |
| `26` | `awscc.ec2.SecurityGroup.Ingress.IpProtocol.26` |
| `27` | `awscc.ec2.SecurityGroup.Ingress.IpProtocol.27` |
| `28` | `awscc.ec2.SecurityGroup.Ingress.IpProtocol.28` |
| `29` | `awscc.ec2.SecurityGroup.Ingress.IpProtocol.29` |
| `30` | `awscc.ec2.SecurityGroup.Ingress.IpProtocol.30` |
| `31` | `awscc.ec2.SecurityGroup.Ingress.IpProtocol.31` |
| `32` | `awscc.ec2.SecurityGroup.Ingress.IpProtocol.32` |
| `33` | `awscc.ec2.SecurityGroup.Ingress.IpProtocol.33` |
| `34` | `awscc.ec2.SecurityGroup.Ingress.IpProtocol.34` |
| `35` | `awscc.ec2.SecurityGroup.Ingress.IpProtocol.35` |
| `36` | `awscc.ec2.SecurityGroup.Ingress.IpProtocol.36` |
| `37` | `awscc.ec2.SecurityGroup.Ingress.IpProtocol.37` |
| `38` | `awscc.ec2.SecurityGroup.Ingress.IpProtocol.38` |
| `39` | `awscc.ec2.SecurityGroup.Ingress.IpProtocol.39` |
| `40` | `awscc.ec2.SecurityGroup.Ingress.IpProtocol.40` |
| `41` | `awscc.ec2.SecurityGroup.Ingress.IpProtocol.41` |
| `42` | `awscc.ec2.SecurityGroup.Ingress.IpProtocol.42` |
| `43` | `awscc.ec2.SecurityGroup.Ingress.IpProtocol.43` |
| `44` | `awscc.ec2.SecurityGroup.Ingress.IpProtocol.44` |
| `45` | `awscc.ec2.SecurityGroup.Ingress.IpProtocol.45` |
| `46` | `awscc.ec2.SecurityGroup.Ingress.IpProtocol.46` |
| `47` | `awscc.ec2.SecurityGroup.Ingress.IpProtocol.47` |
| `48` | `awscc.ec2.SecurityGroup.Ingress.IpProtocol.48` |
| `49` | `awscc.ec2.SecurityGroup.Ingress.IpProtocol.49` |
| `50` | `awscc.ec2.SecurityGroup.Ingress.IpProtocol.50` |
| `51` | `awscc.ec2.SecurityGroup.Ingress.IpProtocol.51` |
| `52` | `awscc.ec2.SecurityGroup.Ingress.IpProtocol.52` |
| `53` | `awscc.ec2.SecurityGroup.Ingress.IpProtocol.53` |
| `54` | `awscc.ec2.SecurityGroup.Ingress.IpProtocol.54` |
| `55` | `awscc.ec2.SecurityGroup.Ingress.IpProtocol.55` |
| `56` | `awscc.ec2.SecurityGroup.Ingress.IpProtocol.56` |
| `57` | `awscc.ec2.SecurityGroup.Ingress.IpProtocol.57` |
| `58` | `awscc.ec2.SecurityGroup.Ingress.IpProtocol.58` |
| `59` | `awscc.ec2.SecurityGroup.Ingress.IpProtocol.59` |
| `60` | `awscc.ec2.SecurityGroup.Ingress.IpProtocol.60` |
| `61` | `awscc.ec2.SecurityGroup.Ingress.IpProtocol.61` |
| `62` | `awscc.ec2.SecurityGroup.Ingress.IpProtocol.62` |
| `63` | `awscc.ec2.SecurityGroup.Ingress.IpProtocol.63` |
| `64` | `awscc.ec2.SecurityGroup.Ingress.IpProtocol.64` |
| `65` | `awscc.ec2.SecurityGroup.Ingress.IpProtocol.65` |
| `66` | `awscc.ec2.SecurityGroup.Ingress.IpProtocol.66` |
| `67` | `awscc.ec2.SecurityGroup.Ingress.IpProtocol.67` |
| `68` | `awscc.ec2.SecurityGroup.Ingress.IpProtocol.68` |
| `69` | `awscc.ec2.SecurityGroup.Ingress.IpProtocol.69` |
| `70` | `awscc.ec2.SecurityGroup.Ingress.IpProtocol.70` |
| `71` | `awscc.ec2.SecurityGroup.Ingress.IpProtocol.71` |
| `72` | `awscc.ec2.SecurityGroup.Ingress.IpProtocol.72` |
| `73` | `awscc.ec2.SecurityGroup.Ingress.IpProtocol.73` |
| `74` | `awscc.ec2.SecurityGroup.Ingress.IpProtocol.74` |
| `75` | `awscc.ec2.SecurityGroup.Ingress.IpProtocol.75` |
| `76` | `awscc.ec2.SecurityGroup.Ingress.IpProtocol.76` |
| `77` | `awscc.ec2.SecurityGroup.Ingress.IpProtocol.77` |
| `78` | `awscc.ec2.SecurityGroup.Ingress.IpProtocol.78` |
| `79` | `awscc.ec2.SecurityGroup.Ingress.IpProtocol.79` |
| `80` | `awscc.ec2.SecurityGroup.Ingress.IpProtocol.80` |
| `81` | `awscc.ec2.SecurityGroup.Ingress.IpProtocol.81` |
| `82` | `awscc.ec2.SecurityGroup.Ingress.IpProtocol.82` |
| `83` | `awscc.ec2.SecurityGroup.Ingress.IpProtocol.83` |
| `84` | `awscc.ec2.SecurityGroup.Ingress.IpProtocol.84` |
| `85` | `awscc.ec2.SecurityGroup.Ingress.IpProtocol.85` |
| `86` | `awscc.ec2.SecurityGroup.Ingress.IpProtocol.86` |
| `87` | `awscc.ec2.SecurityGroup.Ingress.IpProtocol.87` |
| `88` | `awscc.ec2.SecurityGroup.Ingress.IpProtocol.88` |
| `89` | `awscc.ec2.SecurityGroup.Ingress.IpProtocol.89` |
| `90` | `awscc.ec2.SecurityGroup.Ingress.IpProtocol.90` |
| `91` | `awscc.ec2.SecurityGroup.Ingress.IpProtocol.91` |
| `92` | `awscc.ec2.SecurityGroup.Ingress.IpProtocol.92` |
| `93` | `awscc.ec2.SecurityGroup.Ingress.IpProtocol.93` |
| `94` | `awscc.ec2.SecurityGroup.Ingress.IpProtocol.94` |
| `95` | `awscc.ec2.SecurityGroup.Ingress.IpProtocol.95` |
| `96` | `awscc.ec2.SecurityGroup.Ingress.IpProtocol.96` |
| `97` | `awscc.ec2.SecurityGroup.Ingress.IpProtocol.97` |
| `98` | `awscc.ec2.SecurityGroup.Ingress.IpProtocol.98` |
| `99` | `awscc.ec2.SecurityGroup.Ingress.IpProtocol.99` |
| `100` | `awscc.ec2.SecurityGroup.Ingress.IpProtocol.100` |
| `101` | `awscc.ec2.SecurityGroup.Ingress.IpProtocol.101` |
| `102` | `awscc.ec2.SecurityGroup.Ingress.IpProtocol.102` |
| `103` | `awscc.ec2.SecurityGroup.Ingress.IpProtocol.103` |
| `104` | `awscc.ec2.SecurityGroup.Ingress.IpProtocol.104` |
| `105` | `awscc.ec2.SecurityGroup.Ingress.IpProtocol.105` |
| `106` | `awscc.ec2.SecurityGroup.Ingress.IpProtocol.106` |
| `107` | `awscc.ec2.SecurityGroup.Ingress.IpProtocol.107` |
| `108` | `awscc.ec2.SecurityGroup.Ingress.IpProtocol.108` |
| `109` | `awscc.ec2.SecurityGroup.Ingress.IpProtocol.109` |
| `110` | `awscc.ec2.SecurityGroup.Ingress.IpProtocol.110` |
| `111` | `awscc.ec2.SecurityGroup.Ingress.IpProtocol.111` |
| `112` | `awscc.ec2.SecurityGroup.Ingress.IpProtocol.112` |
| `113` | `awscc.ec2.SecurityGroup.Ingress.IpProtocol.113` |
| `114` | `awscc.ec2.SecurityGroup.Ingress.IpProtocol.114` |
| `115` | `awscc.ec2.SecurityGroup.Ingress.IpProtocol.115` |
| `116` | `awscc.ec2.SecurityGroup.Ingress.IpProtocol.116` |
| `117` | `awscc.ec2.SecurityGroup.Ingress.IpProtocol.117` |
| `118` | `awscc.ec2.SecurityGroup.Ingress.IpProtocol.118` |
| `119` | `awscc.ec2.SecurityGroup.Ingress.IpProtocol.119` |
| `120` | `awscc.ec2.SecurityGroup.Ingress.IpProtocol.120` |
| `121` | `awscc.ec2.SecurityGroup.Ingress.IpProtocol.121` |
| `122` | `awscc.ec2.SecurityGroup.Ingress.IpProtocol.122` |
| `123` | `awscc.ec2.SecurityGroup.Ingress.IpProtocol.123` |
| `124` | `awscc.ec2.SecurityGroup.Ingress.IpProtocol.124` |
| `125` | `awscc.ec2.SecurityGroup.Ingress.IpProtocol.125` |
| `126` | `awscc.ec2.SecurityGroup.Ingress.IpProtocol.126` |
| `127` | `awscc.ec2.SecurityGroup.Ingress.IpProtocol.127` |
| `128` | `awscc.ec2.SecurityGroup.Ingress.IpProtocol.128` |
| `129` | `awscc.ec2.SecurityGroup.Ingress.IpProtocol.129` |
| `130` | `awscc.ec2.SecurityGroup.Ingress.IpProtocol.130` |
| `131` | `awscc.ec2.SecurityGroup.Ingress.IpProtocol.131` |
| `132` | `awscc.ec2.SecurityGroup.Ingress.IpProtocol.132` |
| `133` | `awscc.ec2.SecurityGroup.Ingress.IpProtocol.133` |
| `134` | `awscc.ec2.SecurityGroup.Ingress.IpProtocol.134` |
| `135` | `awscc.ec2.SecurityGroup.Ingress.IpProtocol.135` |
| `136` | `awscc.ec2.SecurityGroup.Ingress.IpProtocol.136` |
| `137` | `awscc.ec2.SecurityGroup.Ingress.IpProtocol.137` |
| `138` | `awscc.ec2.SecurityGroup.Ingress.IpProtocol.138` |
| `139` | `awscc.ec2.SecurityGroup.Ingress.IpProtocol.139` |
| `140` | `awscc.ec2.SecurityGroup.Ingress.IpProtocol.140` |
| `141` | `awscc.ec2.SecurityGroup.Ingress.IpProtocol.141` |
| `142` | `awscc.ec2.SecurityGroup.Ingress.IpProtocol.142` |
| `143` | `awscc.ec2.SecurityGroup.Ingress.IpProtocol.143` |
| `144` | `awscc.ec2.SecurityGroup.Ingress.IpProtocol.144` |
| `145` | `awscc.ec2.SecurityGroup.Ingress.IpProtocol.145` |
| `146` | `awscc.ec2.SecurityGroup.Ingress.IpProtocol.146` |
| `147` | `awscc.ec2.SecurityGroup.Ingress.IpProtocol.147` |
| `148` | `awscc.ec2.SecurityGroup.Ingress.IpProtocol.148` |
| `149` | `awscc.ec2.SecurityGroup.Ingress.IpProtocol.149` |
| `150` | `awscc.ec2.SecurityGroup.Ingress.IpProtocol.150` |
| `151` | `awscc.ec2.SecurityGroup.Ingress.IpProtocol.151` |
| `152` | `awscc.ec2.SecurityGroup.Ingress.IpProtocol.152` |
| `153` | `awscc.ec2.SecurityGroup.Ingress.IpProtocol.153` |
| `154` | `awscc.ec2.SecurityGroup.Ingress.IpProtocol.154` |
| `155` | `awscc.ec2.SecurityGroup.Ingress.IpProtocol.155` |
| `156` | `awscc.ec2.SecurityGroup.Ingress.IpProtocol.156` |
| `157` | `awscc.ec2.SecurityGroup.Ingress.IpProtocol.157` |
| `158` | `awscc.ec2.SecurityGroup.Ingress.IpProtocol.158` |
| `159` | `awscc.ec2.SecurityGroup.Ingress.IpProtocol.159` |
| `160` | `awscc.ec2.SecurityGroup.Ingress.IpProtocol.160` |
| `161` | `awscc.ec2.SecurityGroup.Ingress.IpProtocol.161` |
| `162` | `awscc.ec2.SecurityGroup.Ingress.IpProtocol.162` |
| `163` | `awscc.ec2.SecurityGroup.Ingress.IpProtocol.163` |
| `164` | `awscc.ec2.SecurityGroup.Ingress.IpProtocol.164` |
| `165` | `awscc.ec2.SecurityGroup.Ingress.IpProtocol.165` |
| `166` | `awscc.ec2.SecurityGroup.Ingress.IpProtocol.166` |
| `167` | `awscc.ec2.SecurityGroup.Ingress.IpProtocol.167` |
| `168` | `awscc.ec2.SecurityGroup.Ingress.IpProtocol.168` |
| `169` | `awscc.ec2.SecurityGroup.Ingress.IpProtocol.169` |
| `170` | `awscc.ec2.SecurityGroup.Ingress.IpProtocol.170` |
| `171` | `awscc.ec2.SecurityGroup.Ingress.IpProtocol.171` |
| `172` | `awscc.ec2.SecurityGroup.Ingress.IpProtocol.172` |
| `173` | `awscc.ec2.SecurityGroup.Ingress.IpProtocol.173` |
| `174` | `awscc.ec2.SecurityGroup.Ingress.IpProtocol.174` |
| `175` | `awscc.ec2.SecurityGroup.Ingress.IpProtocol.175` |
| `176` | `awscc.ec2.SecurityGroup.Ingress.IpProtocol.176` |
| `177` | `awscc.ec2.SecurityGroup.Ingress.IpProtocol.177` |
| `178` | `awscc.ec2.SecurityGroup.Ingress.IpProtocol.178` |
| `179` | `awscc.ec2.SecurityGroup.Ingress.IpProtocol.179` |
| `180` | `awscc.ec2.SecurityGroup.Ingress.IpProtocol.180` |
| `181` | `awscc.ec2.SecurityGroup.Ingress.IpProtocol.181` |
| `182` | `awscc.ec2.SecurityGroup.Ingress.IpProtocol.182` |
| `183` | `awscc.ec2.SecurityGroup.Ingress.IpProtocol.183` |
| `184` | `awscc.ec2.SecurityGroup.Ingress.IpProtocol.184` |
| `185` | `awscc.ec2.SecurityGroup.Ingress.IpProtocol.185` |
| `186` | `awscc.ec2.SecurityGroup.Ingress.IpProtocol.186` |
| `187` | `awscc.ec2.SecurityGroup.Ingress.IpProtocol.187` |
| `188` | `awscc.ec2.SecurityGroup.Ingress.IpProtocol.188` |
| `189` | `awscc.ec2.SecurityGroup.Ingress.IpProtocol.189` |
| `190` | `awscc.ec2.SecurityGroup.Ingress.IpProtocol.190` |
| `191` | `awscc.ec2.SecurityGroup.Ingress.IpProtocol.191` |
| `192` | `awscc.ec2.SecurityGroup.Ingress.IpProtocol.192` |
| `193` | `awscc.ec2.SecurityGroup.Ingress.IpProtocol.193` |
| `194` | `awscc.ec2.SecurityGroup.Ingress.IpProtocol.194` |
| `195` | `awscc.ec2.SecurityGroup.Ingress.IpProtocol.195` |
| `196` | `awscc.ec2.SecurityGroup.Ingress.IpProtocol.196` |
| `197` | `awscc.ec2.SecurityGroup.Ingress.IpProtocol.197` |
| `198` | `awscc.ec2.SecurityGroup.Ingress.IpProtocol.198` |
| `199` | `awscc.ec2.SecurityGroup.Ingress.IpProtocol.199` |
| `200` | `awscc.ec2.SecurityGroup.Ingress.IpProtocol.200` |
| `201` | `awscc.ec2.SecurityGroup.Ingress.IpProtocol.201` |
| `202` | `awscc.ec2.SecurityGroup.Ingress.IpProtocol.202` |
| `203` | `awscc.ec2.SecurityGroup.Ingress.IpProtocol.203` |
| `204` | `awscc.ec2.SecurityGroup.Ingress.IpProtocol.204` |
| `205` | `awscc.ec2.SecurityGroup.Ingress.IpProtocol.205` |
| `206` | `awscc.ec2.SecurityGroup.Ingress.IpProtocol.206` |
| `207` | `awscc.ec2.SecurityGroup.Ingress.IpProtocol.207` |
| `208` | `awscc.ec2.SecurityGroup.Ingress.IpProtocol.208` |
| `209` | `awscc.ec2.SecurityGroup.Ingress.IpProtocol.209` |
| `210` | `awscc.ec2.SecurityGroup.Ingress.IpProtocol.210` |
| `211` | `awscc.ec2.SecurityGroup.Ingress.IpProtocol.211` |
| `212` | `awscc.ec2.SecurityGroup.Ingress.IpProtocol.212` |
| `213` | `awscc.ec2.SecurityGroup.Ingress.IpProtocol.213` |
| `214` | `awscc.ec2.SecurityGroup.Ingress.IpProtocol.214` |
| `215` | `awscc.ec2.SecurityGroup.Ingress.IpProtocol.215` |
| `216` | `awscc.ec2.SecurityGroup.Ingress.IpProtocol.216` |
| `217` | `awscc.ec2.SecurityGroup.Ingress.IpProtocol.217` |
| `218` | `awscc.ec2.SecurityGroup.Ingress.IpProtocol.218` |
| `219` | `awscc.ec2.SecurityGroup.Ingress.IpProtocol.219` |
| `220` | `awscc.ec2.SecurityGroup.Ingress.IpProtocol.220` |
| `221` | `awscc.ec2.SecurityGroup.Ingress.IpProtocol.221` |
| `222` | `awscc.ec2.SecurityGroup.Ingress.IpProtocol.222` |
| `223` | `awscc.ec2.SecurityGroup.Ingress.IpProtocol.223` |
| `224` | `awscc.ec2.SecurityGroup.Ingress.IpProtocol.224` |
| `225` | `awscc.ec2.SecurityGroup.Ingress.IpProtocol.225` |
| `226` | `awscc.ec2.SecurityGroup.Ingress.IpProtocol.226` |
| `227` | `awscc.ec2.SecurityGroup.Ingress.IpProtocol.227` |
| `228` | `awscc.ec2.SecurityGroup.Ingress.IpProtocol.228` |
| `229` | `awscc.ec2.SecurityGroup.Ingress.IpProtocol.229` |
| `230` | `awscc.ec2.SecurityGroup.Ingress.IpProtocol.230` |
| `231` | `awscc.ec2.SecurityGroup.Ingress.IpProtocol.231` |
| `232` | `awscc.ec2.SecurityGroup.Ingress.IpProtocol.232` |
| `233` | `awscc.ec2.SecurityGroup.Ingress.IpProtocol.233` |
| `234` | `awscc.ec2.SecurityGroup.Ingress.IpProtocol.234` |
| `235` | `awscc.ec2.SecurityGroup.Ingress.IpProtocol.235` |
| `236` | `awscc.ec2.SecurityGroup.Ingress.IpProtocol.236` |
| `237` | `awscc.ec2.SecurityGroup.Ingress.IpProtocol.237` |
| `238` | `awscc.ec2.SecurityGroup.Ingress.IpProtocol.238` |
| `239` | `awscc.ec2.SecurityGroup.Ingress.IpProtocol.239` |
| `240` | `awscc.ec2.SecurityGroup.Ingress.IpProtocol.240` |
| `241` | `awscc.ec2.SecurityGroup.Ingress.IpProtocol.241` |
| `242` | `awscc.ec2.SecurityGroup.Ingress.IpProtocol.242` |
| `243` | `awscc.ec2.SecurityGroup.Ingress.IpProtocol.243` |
| `244` | `awscc.ec2.SecurityGroup.Ingress.IpProtocol.244` |
| `245` | `awscc.ec2.SecurityGroup.Ingress.IpProtocol.245` |
| `246` | `awscc.ec2.SecurityGroup.Ingress.IpProtocol.246` |
| `247` | `awscc.ec2.SecurityGroup.Ingress.IpProtocol.247` |
| `248` | `awscc.ec2.SecurityGroup.Ingress.IpProtocol.248` |
| `249` | `awscc.ec2.SecurityGroup.Ingress.IpProtocol.249` |
| `250` | `awscc.ec2.SecurityGroup.Ingress.IpProtocol.250` |
| `251` | `awscc.ec2.SecurityGroup.Ingress.IpProtocol.251` |
| `252` | `awscc.ec2.SecurityGroup.Ingress.IpProtocol.252` |
| `253` | `awscc.ec2.SecurityGroup.Ingress.IpProtocol.253` |
| `254` | `awscc.ec2.SecurityGroup.Ingress.IpProtocol.254` |
| `255` | `awscc.ec2.SecurityGroup.Ingress.IpProtocol.255` |

Shorthand formats: `tcp` or `IpProtocol.tcp`

## Struct Definitions

### security_group_egress

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `cidr_ip` | `Ipv4Cidr (custom validator)` | No |  |
| `cidr_ipv6` | `Ipv6Cidr (custom validator)` | No |  |
| `description` | `String (pattern: /^[a-zA-Z0-9. _:/()#,@\[\]+=&;{}!$*-]*$/ (Full), len: unbounded..=255)` | No |  |
| `destination_prefix_list_id` | `awscc.ec2.PrefixList.Id (pattern: /^pl-[0-9a-f]{8,}$/ (Full))` | No |  |
| `destination_security_group_id` | `awscc.ec2.SecurityGroup.Id (pattern: /^sg-[0-9a-f]{8,}$/ (Full))` | No |  |
| `from_port` | `Int (range: -1..=65535)` | No |  |
| `ip_protocol` | [IpProtocol](#security_group_egressip_protocol-ipprotocol) | Yes |  |
| `to_port` | `Int (range: -1..=65535)` | No |  |

### security_group_ingress

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `cidr_ip` | `Ipv4Cidr (custom validator)` | No |  |
| `cidr_ipv6` | `Ipv6Cidr (custom validator)` | No |  |
| `description` | `String (pattern: /^[a-zA-Z0-9. _:/()#,@\[\]+=&;{}!$*-]*$/ (Full), len: unbounded..=255)` | No |  |
| `from_port` | `Int (range: -1..=65535)` | No |  |
| `ip_protocol` | [IpProtocol](#security_group_ingressip_protocol-ipprotocol) | Yes |  |
| `source_prefix_list_id` | `awscc.ec2.PrefixList.Id (pattern: /^pl-[0-9a-f]{8,}$/ (Full))` | No |  |
| `source_security_group_id` | `awscc.ec2.SecurityGroup.Id (pattern: /^sg-[0-9a-f]{8,}$/ (Full))` | No |  |
| `source_security_group_name` | `String` | No |  |
| `source_security_group_owner_id` | `awscc.AccountId (pattern: /^[0-9]{12}$/ (Full), len: 12..=12)` | No |  |
| `to_port` | `Int (range: -1..=65535)` | No |  |

## Attribute Reference

### `group_id`

- **Type:** `awscc.ec2.SecurityGroup.Id (pattern: /^sg-[0-9a-f]{8,}$/ (Full))`

The group ID of the specified security group.

### `id`

- **Type:** `awscc.ec2.SecurityGroup.Id (pattern: /^sg-[0-9a-f]{8,}$/ (Full))`

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

