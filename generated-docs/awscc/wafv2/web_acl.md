---
title: "awscc.wafv2.WebAcl"
---


CloudFormation Type: `AWS::WAFv2::WebACL`

Contains the Rules that identify the requests that you want to allow, block, or count. In a WebACL, you also specify a default action (ALLOW or BLOCK), and the action for each Rule that you add to a WebACL, for example, block requests from specified IP addresses or block requests from specified referrers. You also associate the WebACL with a CloudFront distribution to identify the requests that you want AWS WAF to filter. If you add more than one Rule to a WebACL, a request needs to match only one of the specifications to be allowed, blocked, or counted.

## Example

```crn
awscc.wafv2.WebAcl {
  name        = 'carina-example-web-acl'
  description = 'Regional protection for the example workload'
  scope       = aws.wafv2.WebAcl.Scope.regional

  default_action = {
    allow = {}
  }

  visibility_config = {
    cloud_watch_metrics_enabled = true
    metric_name                 = 'carina-example-web-acl'
    sampled_requests_enabled    = true
  }

  tags = {
    Environment = 'example'
  }
}
```

## Argument Reference

### `application_config`

- **Type:** [Struct(ApplicationConfig)](#application_config-1)
- **Required:** No

Collection of application attributes.

### `association_config`

- **Type:** [Struct(AssociationConfig)](#association_config-1)
- **Required:** No

### `captcha_config`

- **Type:** [Struct(CaptchaConfig)](#captcha_config-1)
- **Required:** No

### `challenge_config`

- **Type:** [Struct(ChallengeConfig)](#challenge_config-1)
- **Required:** No

### `custom_response_bodies`

- **Type:** Map\<String(pattern, len: 1..=128), [Struct(CustomResponseBody)](#custom_response_bodies-1)\> (entries: 1..)
- **Required:** No

### `data_protection_config`

- **Type:** [Struct(DataProtectionConfig)](#data_protection_config-1)
- **Required:** No

Collection of dataProtects.

### `default_action`

- **Type:** [Struct(DefaultAction)](#default_action-1)
- **Required:** Yes

### `description`

- **Type:** `String(pattern, len: 1..=256)`
- **Required:** No

### `name`

- **Type:** `String(pattern, len: 1..=128)`
- **Required:** No
- **Create-only:** Yes

### `on_source_d_do_s_protection_config`

- **Type:** [Struct(OnSourceDDoSProtectionConfig)](#on_source_d_do_s_protection_config-1)
- **Required:** No

### `rules`

- **Type:** List\<[Struct(Rule)](#rules-1)\>
- **Required:** No

Collection of Rules.

### `scope`

- **Type:** [Enum (Scope)](#scope-scope)
- **Required:** Yes
- **Create-only:** Yes

### `tags`

- **Type:** `Map<String(pattern, len: 1..=128), String(pattern, len: ..=256)>` (entries: 1..)
- **Required:** No

### `token_domains`

- **Type:** `List<String(pattern, len: 1..=253)>`
- **Required:** No

### `visibility_config`

- **Type:** [Struct(VisibilityConfig)](#visibility_config-1)
- **Required:** Yes

## Enum Values

### association_config.request_body (RequestBody)

| Value | DSL Identifier |
|-------|----------------|
| `CLOUDFRONT` | `aws.wafv2.WebAcl.AssociationConfig.RequestBody.cloudfront` |
| `API_GATEWAY` | `aws.wafv2.WebAcl.AssociationConfig.RequestBody.api_gateway` |
| `COGNITO_USER_POOL` | `aws.wafv2.WebAcl.AssociationConfig.RequestBody.cognito_user_pool` |
| `APP_RUNNER_SERVICE` | `aws.wafv2.WebAcl.AssociationConfig.RequestBody.app_runner_service` |
| `VERIFIED_ACCESS_INSTANCE` | `aws.wafv2.WebAcl.AssociationConfig.RequestBody.verified_access_instance` |

Shorthand formats: `cloudfront` or `RequestBody.cloudfront`

### association_config.request_body.default_size_inspection_limit (SizeInspectionLimit)

| Value | DSL Identifier |
|-------|----------------|
| `KB_16` | `aws.wafv2.WebAcl.AssociationConfig.RequestBodyAssociatedResourceTypeConfig.SizeInspectionLimit.kb_16` |
| `KB_32` | `aws.wafv2.WebAcl.AssociationConfig.RequestBodyAssociatedResourceTypeConfig.SizeInspectionLimit.kb_32` |
| `KB_48` | `aws.wafv2.WebAcl.AssociationConfig.RequestBodyAssociatedResourceTypeConfig.SizeInspectionLimit.kb_48` |
| `KB_64` | `aws.wafv2.WebAcl.AssociationConfig.RequestBodyAssociatedResourceTypeConfig.SizeInspectionLimit.kb_64` |

Shorthand formats: `kb_16` or `SizeInspectionLimit.kb_16`

### custom_response_bodies.content_type (ResponseContentType)

| Value | DSL Identifier |
|-------|----------------|
| `TEXT_PLAIN` | `aws.wafv2.WebAcl.CustomResponseBody.ResponseContentType.text_plain` |
| `TEXT_HTML` | `aws.wafv2.WebAcl.CustomResponseBody.ResponseContentType.text_html` |
| `APPLICATION_JSON` | `aws.wafv2.WebAcl.CustomResponseBody.ResponseContentType.application_json` |

Shorthand formats: `text_plain` or `ResponseContentType.text_plain`

### data_protection_config.data_protections.action (DataProtectionAction)

| Value | DSL Identifier |
|-------|----------------|
| `SUBSTITUTION` | `aws.wafv2.WebAcl.DataProtectionConfig.DataProtect.DataProtectionAction.substitution` |
| `HASH` | `aws.wafv2.WebAcl.DataProtectionConfig.DataProtect.DataProtectionAction.hash` |

Shorthand formats: `substitution` or `DataProtectionAction.substitution`

### data_protection_config.data_protections.field.field_type (FieldType)

| Value | DSL Identifier |
|-------|----------------|
| `SINGLE_HEADER` | `aws.wafv2.WebAcl.DataProtectionConfig.DataProtect.FieldToProtect.FieldType.single_header` |
| `SINGLE_COOKIE` | `aws.wafv2.WebAcl.DataProtectionConfig.DataProtect.FieldToProtect.FieldType.single_cookie` |
| `SINGLE_QUERY_ARGUMENT` | `aws.wafv2.WebAcl.DataProtectionConfig.DataProtect.FieldToProtect.FieldType.single_query_argument` |
| `QUERY_STRING` | `aws.wafv2.WebAcl.DataProtectionConfig.DataProtect.FieldToProtect.FieldType.query_string` |
| `BODY` | `aws.wafv2.WebAcl.DataProtectionConfig.DataProtect.FieldToProtect.FieldType.body` |

Shorthand formats: `single_header` or `FieldType.single_header`

### field_to_match.body.oversize_handling (OversizeHandling)

| Value | DSL Identifier |
|-------|----------------|
| `CONTINUE` | `aws.wafv2.WebAcl.Rule.Statement.ByteMatchStatement.FieldToMatch.Body.OversizeHandling.continue` |
| `MATCH` | `aws.wafv2.WebAcl.Rule.Statement.ByteMatchStatement.FieldToMatch.Body.OversizeHandling.match` |
| `NO_MATCH` | `aws.wafv2.WebAcl.Rule.Statement.ByteMatchStatement.FieldToMatch.Body.OversizeHandling.no_match` |

Shorthand formats: `continue` or `OversizeHandling.continue`

### field_to_match.cookies.match_scope (MapMatchScope)

| Value | DSL Identifier |
|-------|----------------|
| `ALL` | `aws.wafv2.WebAcl.Rule.Statement.ByteMatchStatement.FieldToMatch.Cookies.MapMatchScope.all` |
| `KEY` | `aws.wafv2.WebAcl.Rule.Statement.ByteMatchStatement.FieldToMatch.Cookies.MapMatchScope.key` |
| `VALUE` | `aws.wafv2.WebAcl.Rule.Statement.ByteMatchStatement.FieldToMatch.Cookies.MapMatchScope.value` |

Shorthand formats: `all` or `MapMatchScope.all`

### field_to_match.cookies.oversize_handling (OversizeHandling)

| Value | DSL Identifier |
|-------|----------------|
| `CONTINUE` | `aws.wafv2.WebAcl.Rule.Statement.ByteMatchStatement.FieldToMatch.Cookies.OversizeHandling.continue` |
| `MATCH` | `aws.wafv2.WebAcl.Rule.Statement.ByteMatchStatement.FieldToMatch.Cookies.OversizeHandling.match` |
| `NO_MATCH` | `aws.wafv2.WebAcl.Rule.Statement.ByteMatchStatement.FieldToMatch.Cookies.OversizeHandling.no_match` |

Shorthand formats: `continue` or `OversizeHandling.continue`

### field_to_match.header_order.oversize_handling (OversizeHandling)

| Value | DSL Identifier |
|-------|----------------|
| `CONTINUE` | `aws.wafv2.WebAcl.Rule.Statement.ByteMatchStatement.FieldToMatch.HeaderOrder.OversizeHandling.continue` |
| `MATCH` | `aws.wafv2.WebAcl.Rule.Statement.ByteMatchStatement.FieldToMatch.HeaderOrder.OversizeHandling.match` |
| `NO_MATCH` | `aws.wafv2.WebAcl.Rule.Statement.ByteMatchStatement.FieldToMatch.HeaderOrder.OversizeHandling.no_match` |

Shorthand formats: `continue` or `OversizeHandling.continue`

### field_to_match.headers.match_scope (MapMatchScope)

| Value | DSL Identifier |
|-------|----------------|
| `ALL` | `aws.wafv2.WebAcl.Rule.Statement.ByteMatchStatement.FieldToMatch.Headers.MapMatchScope.all` |
| `KEY` | `aws.wafv2.WebAcl.Rule.Statement.ByteMatchStatement.FieldToMatch.Headers.MapMatchScope.key` |
| `VALUE` | `aws.wafv2.WebAcl.Rule.Statement.ByteMatchStatement.FieldToMatch.Headers.MapMatchScope.value` |

Shorthand formats: `all` or `MapMatchScope.all`

### field_to_match.headers.oversize_handling (OversizeHandling)

| Value | DSL Identifier |
|-------|----------------|
| `CONTINUE` | `aws.wafv2.WebAcl.Rule.Statement.ByteMatchStatement.FieldToMatch.Headers.OversizeHandling.continue` |
| `MATCH` | `aws.wafv2.WebAcl.Rule.Statement.ByteMatchStatement.FieldToMatch.Headers.OversizeHandling.match` |
| `NO_MATCH` | `aws.wafv2.WebAcl.Rule.Statement.ByteMatchStatement.FieldToMatch.Headers.OversizeHandling.no_match` |

Shorthand formats: `continue` or `OversizeHandling.continue`

### field_to_match.ja3_fingerprint.fallback_behavior (FallbackBehavior)

| Value | DSL Identifier |
|-------|----------------|
| `MATCH` | `aws.wafv2.WebAcl.Rule.Statement.ByteMatchStatement.FieldToMatch.Ja3Fingerprint.FallbackBehavior.match` |
| `NO_MATCH` | `aws.wafv2.WebAcl.Rule.Statement.ByteMatchStatement.FieldToMatch.Ja3Fingerprint.FallbackBehavior.no_match` |

Shorthand formats: `match` or `FallbackBehavior.match`

### field_to_match.ja4_fingerprint.fallback_behavior (FallbackBehavior)

| Value | DSL Identifier |
|-------|----------------|
| `MATCH` | `aws.wafv2.WebAcl.Rule.Statement.ByteMatchStatement.FieldToMatch.Ja4Fingerprint.FallbackBehavior.match` |
| `NO_MATCH` | `aws.wafv2.WebAcl.Rule.Statement.ByteMatchStatement.FieldToMatch.Ja4Fingerprint.FallbackBehavior.no_match` |

Shorthand formats: `match` or `FallbackBehavior.match`

### field_to_match.json_body.invalid_fallback_behavior (BodyParsingFallbackBehavior)

| Value | DSL Identifier |
|-------|----------------|
| `MATCH` | `aws.wafv2.WebAcl.Rule.Statement.ByteMatchStatement.FieldToMatch.JsonBody.BodyParsingFallbackBehavior.match` |
| `NO_MATCH` | `aws.wafv2.WebAcl.Rule.Statement.ByteMatchStatement.FieldToMatch.JsonBody.BodyParsingFallbackBehavior.no_match` |
| `EVALUATE_AS_STRING` | `aws.wafv2.WebAcl.Rule.Statement.ByteMatchStatement.FieldToMatch.JsonBody.BodyParsingFallbackBehavior.evaluate_as_string` |

Shorthand formats: `match` or `BodyParsingFallbackBehavior.match`

### field_to_match.json_body.match_scope (JsonMatchScope)

| Value | DSL Identifier |
|-------|----------------|
| `ALL` | `aws.wafv2.WebAcl.Rule.Statement.ByteMatchStatement.FieldToMatch.JsonBody.JsonMatchScope.all` |
| `KEY` | `aws.wafv2.WebAcl.Rule.Statement.ByteMatchStatement.FieldToMatch.JsonBody.JsonMatchScope.key` |
| `VALUE` | `aws.wafv2.WebAcl.Rule.Statement.ByteMatchStatement.FieldToMatch.JsonBody.JsonMatchScope.value` |

Shorthand formats: `all` or `JsonMatchScope.all`

### field_to_match.json_body.oversize_handling (OversizeHandling)

| Value | DSL Identifier |
|-------|----------------|
| `CONTINUE` | `aws.wafv2.WebAcl.Rule.Statement.ByteMatchStatement.FieldToMatch.JsonBody.OversizeHandling.continue` |
| `MATCH` | `aws.wafv2.WebAcl.Rule.Statement.ByteMatchStatement.FieldToMatch.JsonBody.OversizeHandling.match` |
| `NO_MATCH` | `aws.wafv2.WebAcl.Rule.Statement.ByteMatchStatement.FieldToMatch.JsonBody.OversizeHandling.no_match` |

Shorthand formats: `continue` or `OversizeHandling.continue`

### field_to_match.uri_fragment.fallback_behavior (FallbackBehavior)

| Value | DSL Identifier |
|-------|----------------|
| `MATCH` | `aws.wafv2.WebAcl.Rule.Statement.ByteMatchStatement.FieldToMatch.UriFragment.FallbackBehavior.match` |
| `NO_MATCH` | `aws.wafv2.WebAcl.Rule.Statement.ByteMatchStatement.FieldToMatch.UriFragment.FallbackBehavior.no_match` |

Shorthand formats: `match` or `FallbackBehavior.match`

### forwarded_ip_configuration.fallback_behavior (FallbackBehavior)

| Value | DSL Identifier |
|-------|----------------|
| `MATCH` | `aws.wafv2.WebAcl.Rule.Statement.AsnMatchStatement.ForwardedIpConfiguration.FallbackBehavior.match` |
| `NO_MATCH` | `aws.wafv2.WebAcl.Rule.Statement.AsnMatchStatement.ForwardedIpConfiguration.FallbackBehavior.no_match` |

Shorthand formats: `match` or `FallbackBehavior.match`

### on_source_d_do_s_protection_config.alb_low_reputation_mode (AlbLowReputationMode)

| Value | DSL Identifier |
|-------|----------------|
| `ACTIVE_UNDER_DDOS` | `aws.wafv2.WebAcl.OnSourceDDoSProtectionConfig.AlbLowReputationMode.active_under_ddos` |
| `ALWAYS_ON` | `aws.wafv2.WebAcl.OnSourceDDoSProtectionConfig.AlbLowReputationMode.always_on` |

Shorthand formats: `active_under_ddos` or `AlbLowReputationMode.active_under_ddos`

### scope (Scope)

| Value | DSL Identifier |
|-------|----------------|
| `CLOUDFRONT` | `aws.wafv2.WebAcl.Scope.cloudfront` |
| `REGIONAL` | `aws.wafv2.WebAcl.Scope.regional` |

Shorthand formats: `cloudfront` or `Scope.cloudfront`

### statement.byte_match_statement.positional_constraint (PositionalConstraint)

| Value | DSL Identifier |
|-------|----------------|
| `EXACTLY` | `aws.wafv2.WebAcl.Rule.Statement.ByteMatchStatement.PositionalConstraint.exactly` |
| `STARTS_WITH` | `aws.wafv2.WebAcl.Rule.Statement.ByteMatchStatement.PositionalConstraint.starts_with` |
| `ENDS_WITH` | `aws.wafv2.WebAcl.Rule.Statement.ByteMatchStatement.PositionalConstraint.ends_with` |
| `CONTAINS` | `aws.wafv2.WebAcl.Rule.Statement.ByteMatchStatement.PositionalConstraint.contains` |
| `CONTAINS_WORD` | `aws.wafv2.WebAcl.Rule.Statement.ByteMatchStatement.PositionalConstraint.contains_word` |

Shorthand formats: `exactly` or `PositionalConstraint.exactly`

### statement.geo_match_statement.country_codes (CountryCodes)

| Value | DSL Identifier |
|-------|----------------|
| `AF` | `aws.wafv2.WebAcl.Rule.Statement.GeoMatchStatement.CountryCodes.af` |
| `AX` | `aws.wafv2.WebAcl.Rule.Statement.GeoMatchStatement.CountryCodes.ax` |
| `AL` | `aws.wafv2.WebAcl.Rule.Statement.GeoMatchStatement.CountryCodes.al` |
| `DZ` | `aws.wafv2.WebAcl.Rule.Statement.GeoMatchStatement.CountryCodes.dz` |
| `AS` | `aws.wafv2.WebAcl.Rule.Statement.GeoMatchStatement.CountryCodes.as` |
| `AD` | `aws.wafv2.WebAcl.Rule.Statement.GeoMatchStatement.CountryCodes.ad` |
| `AO` | `aws.wafv2.WebAcl.Rule.Statement.GeoMatchStatement.CountryCodes.ao` |
| `AI` | `aws.wafv2.WebAcl.Rule.Statement.GeoMatchStatement.CountryCodes.ai` |
| `AQ` | `aws.wafv2.WebAcl.Rule.Statement.GeoMatchStatement.CountryCodes.aq` |
| `AG` | `aws.wafv2.WebAcl.Rule.Statement.GeoMatchStatement.CountryCodes.ag` |
| `AR` | `aws.wafv2.WebAcl.Rule.Statement.GeoMatchStatement.CountryCodes.ar` |
| `AM` | `aws.wafv2.WebAcl.Rule.Statement.GeoMatchStatement.CountryCodes.am` |
| `AW` | `aws.wafv2.WebAcl.Rule.Statement.GeoMatchStatement.CountryCodes.aw` |
| `AU` | `aws.wafv2.WebAcl.Rule.Statement.GeoMatchStatement.CountryCodes.au` |
| `AT` | `aws.wafv2.WebAcl.Rule.Statement.GeoMatchStatement.CountryCodes.at` |
| `AZ` | `aws.wafv2.WebAcl.Rule.Statement.GeoMatchStatement.CountryCodes.az` |
| `BS` | `aws.wafv2.WebAcl.Rule.Statement.GeoMatchStatement.CountryCodes.bs` |
| `BH` | `aws.wafv2.WebAcl.Rule.Statement.GeoMatchStatement.CountryCodes.bh` |
| `BD` | `aws.wafv2.WebAcl.Rule.Statement.GeoMatchStatement.CountryCodes.bd` |
| `BB` | `aws.wafv2.WebAcl.Rule.Statement.GeoMatchStatement.CountryCodes.bb` |
| `BY` | `aws.wafv2.WebAcl.Rule.Statement.GeoMatchStatement.CountryCodes.by` |
| `BE` | `aws.wafv2.WebAcl.Rule.Statement.GeoMatchStatement.CountryCodes.be` |
| `BZ` | `aws.wafv2.WebAcl.Rule.Statement.GeoMatchStatement.CountryCodes.bz` |
| `BJ` | `aws.wafv2.WebAcl.Rule.Statement.GeoMatchStatement.CountryCodes.bj` |
| `BM` | `aws.wafv2.WebAcl.Rule.Statement.GeoMatchStatement.CountryCodes.bm` |
| `BT` | `aws.wafv2.WebAcl.Rule.Statement.GeoMatchStatement.CountryCodes.bt` |
| `BO` | `aws.wafv2.WebAcl.Rule.Statement.GeoMatchStatement.CountryCodes.bo` |
| `BQ` | `aws.wafv2.WebAcl.Rule.Statement.GeoMatchStatement.CountryCodes.bq` |
| `BA` | `aws.wafv2.WebAcl.Rule.Statement.GeoMatchStatement.CountryCodes.ba` |
| `BW` | `aws.wafv2.WebAcl.Rule.Statement.GeoMatchStatement.CountryCodes.bw` |
| `BV` | `aws.wafv2.WebAcl.Rule.Statement.GeoMatchStatement.CountryCodes.bv` |
| `BR` | `aws.wafv2.WebAcl.Rule.Statement.GeoMatchStatement.CountryCodes.br` |
| `IO` | `aws.wafv2.WebAcl.Rule.Statement.GeoMatchStatement.CountryCodes.io` |
| `BN` | `aws.wafv2.WebAcl.Rule.Statement.GeoMatchStatement.CountryCodes.bn` |
| `BG` | `aws.wafv2.WebAcl.Rule.Statement.GeoMatchStatement.CountryCodes.bg` |
| `BF` | `aws.wafv2.WebAcl.Rule.Statement.GeoMatchStatement.CountryCodes.bf` |
| `BI` | `aws.wafv2.WebAcl.Rule.Statement.GeoMatchStatement.CountryCodes.bi` |
| `KH` | `aws.wafv2.WebAcl.Rule.Statement.GeoMatchStatement.CountryCodes.kh` |
| `CM` | `aws.wafv2.WebAcl.Rule.Statement.GeoMatchStatement.CountryCodes.cm` |
| `CA` | `aws.wafv2.WebAcl.Rule.Statement.GeoMatchStatement.CountryCodes.ca` |
| `CV` | `aws.wafv2.WebAcl.Rule.Statement.GeoMatchStatement.CountryCodes.cv` |
| `KY` | `aws.wafv2.WebAcl.Rule.Statement.GeoMatchStatement.CountryCodes.ky` |
| `CF` | `aws.wafv2.WebAcl.Rule.Statement.GeoMatchStatement.CountryCodes.cf` |
| `TD` | `aws.wafv2.WebAcl.Rule.Statement.GeoMatchStatement.CountryCodes.td` |
| `CL` | `aws.wafv2.WebAcl.Rule.Statement.GeoMatchStatement.CountryCodes.cl` |
| `CN` | `aws.wafv2.WebAcl.Rule.Statement.GeoMatchStatement.CountryCodes.cn` |
| `CX` | `aws.wafv2.WebAcl.Rule.Statement.GeoMatchStatement.CountryCodes.cx` |
| `CC` | `aws.wafv2.WebAcl.Rule.Statement.GeoMatchStatement.CountryCodes.cc` |
| `CO` | `aws.wafv2.WebAcl.Rule.Statement.GeoMatchStatement.CountryCodes.co` |
| `KM` | `aws.wafv2.WebAcl.Rule.Statement.GeoMatchStatement.CountryCodes.km` |
| `CG` | `aws.wafv2.WebAcl.Rule.Statement.GeoMatchStatement.CountryCodes.cg` |
| `CD` | `aws.wafv2.WebAcl.Rule.Statement.GeoMatchStatement.CountryCodes.cd` |
| `CK` | `aws.wafv2.WebAcl.Rule.Statement.GeoMatchStatement.CountryCodes.ck` |
| `CR` | `aws.wafv2.WebAcl.Rule.Statement.GeoMatchStatement.CountryCodes.cr` |
| `CI` | `aws.wafv2.WebAcl.Rule.Statement.GeoMatchStatement.CountryCodes.ci` |
| `HR` | `aws.wafv2.WebAcl.Rule.Statement.GeoMatchStatement.CountryCodes.hr` |
| `CU` | `aws.wafv2.WebAcl.Rule.Statement.GeoMatchStatement.CountryCodes.cu` |
| `CW` | `aws.wafv2.WebAcl.Rule.Statement.GeoMatchStatement.CountryCodes.cw` |
| `CY` | `aws.wafv2.WebAcl.Rule.Statement.GeoMatchStatement.CountryCodes.cy` |
| `CZ` | `aws.wafv2.WebAcl.Rule.Statement.GeoMatchStatement.CountryCodes.cz` |
| `DK` | `aws.wafv2.WebAcl.Rule.Statement.GeoMatchStatement.CountryCodes.dk` |
| `DJ` | `aws.wafv2.WebAcl.Rule.Statement.GeoMatchStatement.CountryCodes.dj` |
| `DM` | `aws.wafv2.WebAcl.Rule.Statement.GeoMatchStatement.CountryCodes.dm` |
| `DO` | `aws.wafv2.WebAcl.Rule.Statement.GeoMatchStatement.CountryCodes.do` |
| `EC` | `aws.wafv2.WebAcl.Rule.Statement.GeoMatchStatement.CountryCodes.ec` |
| `EG` | `aws.wafv2.WebAcl.Rule.Statement.GeoMatchStatement.CountryCodes.eg` |
| `SV` | `aws.wafv2.WebAcl.Rule.Statement.GeoMatchStatement.CountryCodes.sv` |
| `GQ` | `aws.wafv2.WebAcl.Rule.Statement.GeoMatchStatement.CountryCodes.gq` |
| `ER` | `aws.wafv2.WebAcl.Rule.Statement.GeoMatchStatement.CountryCodes.er` |
| `EE` | `aws.wafv2.WebAcl.Rule.Statement.GeoMatchStatement.CountryCodes.ee` |
| `ET` | `aws.wafv2.WebAcl.Rule.Statement.GeoMatchStatement.CountryCodes.et` |
| `FK` | `aws.wafv2.WebAcl.Rule.Statement.GeoMatchStatement.CountryCodes.fk` |
| `FO` | `aws.wafv2.WebAcl.Rule.Statement.GeoMatchStatement.CountryCodes.fo` |
| `FJ` | `aws.wafv2.WebAcl.Rule.Statement.GeoMatchStatement.CountryCodes.fj` |
| `FI` | `aws.wafv2.WebAcl.Rule.Statement.GeoMatchStatement.CountryCodes.fi` |
| `FR` | `aws.wafv2.WebAcl.Rule.Statement.GeoMatchStatement.CountryCodes.fr` |
| `GF` | `aws.wafv2.WebAcl.Rule.Statement.GeoMatchStatement.CountryCodes.gf` |
| `PF` | `aws.wafv2.WebAcl.Rule.Statement.GeoMatchStatement.CountryCodes.pf` |
| `TF` | `aws.wafv2.WebAcl.Rule.Statement.GeoMatchStatement.CountryCodes.tf` |
| `GA` | `aws.wafv2.WebAcl.Rule.Statement.GeoMatchStatement.CountryCodes.ga` |
| `GM` | `aws.wafv2.WebAcl.Rule.Statement.GeoMatchStatement.CountryCodes.gm` |
| `GE` | `aws.wafv2.WebAcl.Rule.Statement.GeoMatchStatement.CountryCodes.ge` |
| `DE` | `aws.wafv2.WebAcl.Rule.Statement.GeoMatchStatement.CountryCodes.de` |
| `GH` | `aws.wafv2.WebAcl.Rule.Statement.GeoMatchStatement.CountryCodes.gh` |
| `GI` | `aws.wafv2.WebAcl.Rule.Statement.GeoMatchStatement.CountryCodes.gi` |
| `GR` | `aws.wafv2.WebAcl.Rule.Statement.GeoMatchStatement.CountryCodes.gr` |
| `GL` | `aws.wafv2.WebAcl.Rule.Statement.GeoMatchStatement.CountryCodes.gl` |
| `GD` | `aws.wafv2.WebAcl.Rule.Statement.GeoMatchStatement.CountryCodes.gd` |
| `GP` | `aws.wafv2.WebAcl.Rule.Statement.GeoMatchStatement.CountryCodes.gp` |
| `GU` | `aws.wafv2.WebAcl.Rule.Statement.GeoMatchStatement.CountryCodes.gu` |
| `GT` | `aws.wafv2.WebAcl.Rule.Statement.GeoMatchStatement.CountryCodes.gt` |
| `GG` | `aws.wafv2.WebAcl.Rule.Statement.GeoMatchStatement.CountryCodes.gg` |
| `GN` | `aws.wafv2.WebAcl.Rule.Statement.GeoMatchStatement.CountryCodes.gn` |
| `GW` | `aws.wafv2.WebAcl.Rule.Statement.GeoMatchStatement.CountryCodes.gw` |
| `GY` | `aws.wafv2.WebAcl.Rule.Statement.GeoMatchStatement.CountryCodes.gy` |
| `HT` | `aws.wafv2.WebAcl.Rule.Statement.GeoMatchStatement.CountryCodes.ht` |
| `HM` | `aws.wafv2.WebAcl.Rule.Statement.GeoMatchStatement.CountryCodes.hm` |
| `VA` | `aws.wafv2.WebAcl.Rule.Statement.GeoMatchStatement.CountryCodes.va` |
| `HN` | `aws.wafv2.WebAcl.Rule.Statement.GeoMatchStatement.CountryCodes.hn` |
| `HK` | `aws.wafv2.WebAcl.Rule.Statement.GeoMatchStatement.CountryCodes.hk` |
| `HU` | `aws.wafv2.WebAcl.Rule.Statement.GeoMatchStatement.CountryCodes.hu` |
| `IS` | `aws.wafv2.WebAcl.Rule.Statement.GeoMatchStatement.CountryCodes.is` |
| `IN` | `aws.wafv2.WebAcl.Rule.Statement.GeoMatchStatement.CountryCodes.in` |
| `ID` | `aws.wafv2.WebAcl.Rule.Statement.GeoMatchStatement.CountryCodes.id` |
| `IR` | `aws.wafv2.WebAcl.Rule.Statement.GeoMatchStatement.CountryCodes.ir` |
| `IQ` | `aws.wafv2.WebAcl.Rule.Statement.GeoMatchStatement.CountryCodes.iq` |
| `IE` | `aws.wafv2.WebAcl.Rule.Statement.GeoMatchStatement.CountryCodes.ie` |
| `IM` | `aws.wafv2.WebAcl.Rule.Statement.GeoMatchStatement.CountryCodes.im` |
| `IL` | `aws.wafv2.WebAcl.Rule.Statement.GeoMatchStatement.CountryCodes.il` |
| `IT` | `aws.wafv2.WebAcl.Rule.Statement.GeoMatchStatement.CountryCodes.it` |
| `JM` | `aws.wafv2.WebAcl.Rule.Statement.GeoMatchStatement.CountryCodes.jm` |
| `JP` | `aws.wafv2.WebAcl.Rule.Statement.GeoMatchStatement.CountryCodes.jp` |
| `JE` | `aws.wafv2.WebAcl.Rule.Statement.GeoMatchStatement.CountryCodes.je` |
| `JO` | `aws.wafv2.WebAcl.Rule.Statement.GeoMatchStatement.CountryCodes.jo` |
| `KZ` | `aws.wafv2.WebAcl.Rule.Statement.GeoMatchStatement.CountryCodes.kz` |
| `KE` | `aws.wafv2.WebAcl.Rule.Statement.GeoMatchStatement.CountryCodes.ke` |
| `KI` | `aws.wafv2.WebAcl.Rule.Statement.GeoMatchStatement.CountryCodes.ki` |
| `KP` | `aws.wafv2.WebAcl.Rule.Statement.GeoMatchStatement.CountryCodes.kp` |
| `KR` | `aws.wafv2.WebAcl.Rule.Statement.GeoMatchStatement.CountryCodes.kr` |
| `KW` | `aws.wafv2.WebAcl.Rule.Statement.GeoMatchStatement.CountryCodes.kw` |
| `KG` | `aws.wafv2.WebAcl.Rule.Statement.GeoMatchStatement.CountryCodes.kg` |
| `LA` | `aws.wafv2.WebAcl.Rule.Statement.GeoMatchStatement.CountryCodes.la` |
| `LV` | `aws.wafv2.WebAcl.Rule.Statement.GeoMatchStatement.CountryCodes.lv` |
| `LB` | `aws.wafv2.WebAcl.Rule.Statement.GeoMatchStatement.CountryCodes.lb` |
| `LS` | `aws.wafv2.WebAcl.Rule.Statement.GeoMatchStatement.CountryCodes.ls` |
| `LR` | `aws.wafv2.WebAcl.Rule.Statement.GeoMatchStatement.CountryCodes.lr` |
| `LY` | `aws.wafv2.WebAcl.Rule.Statement.GeoMatchStatement.CountryCodes.ly` |
| `LI` | `aws.wafv2.WebAcl.Rule.Statement.GeoMatchStatement.CountryCodes.li` |
| `LT` | `aws.wafv2.WebAcl.Rule.Statement.GeoMatchStatement.CountryCodes.lt` |
| `LU` | `aws.wafv2.WebAcl.Rule.Statement.GeoMatchStatement.CountryCodes.lu` |
| `MO` | `aws.wafv2.WebAcl.Rule.Statement.GeoMatchStatement.CountryCodes.mo` |
| `MK` | `aws.wafv2.WebAcl.Rule.Statement.GeoMatchStatement.CountryCodes.mk` |
| `MG` | `aws.wafv2.WebAcl.Rule.Statement.GeoMatchStatement.CountryCodes.mg` |
| `MW` | `aws.wafv2.WebAcl.Rule.Statement.GeoMatchStatement.CountryCodes.mw` |
| `MY` | `aws.wafv2.WebAcl.Rule.Statement.GeoMatchStatement.CountryCodes.my` |
| `MV` | `aws.wafv2.WebAcl.Rule.Statement.GeoMatchStatement.CountryCodes.mv` |
| `ML` | `aws.wafv2.WebAcl.Rule.Statement.GeoMatchStatement.CountryCodes.ml` |
| `MT` | `aws.wafv2.WebAcl.Rule.Statement.GeoMatchStatement.CountryCodes.mt` |
| `MH` | `aws.wafv2.WebAcl.Rule.Statement.GeoMatchStatement.CountryCodes.mh` |
| `MQ` | `aws.wafv2.WebAcl.Rule.Statement.GeoMatchStatement.CountryCodes.mq` |
| `MR` | `aws.wafv2.WebAcl.Rule.Statement.GeoMatchStatement.CountryCodes.mr` |
| `MU` | `aws.wafv2.WebAcl.Rule.Statement.GeoMatchStatement.CountryCodes.mu` |
| `YT` | `aws.wafv2.WebAcl.Rule.Statement.GeoMatchStatement.CountryCodes.yt` |
| `MX` | `aws.wafv2.WebAcl.Rule.Statement.GeoMatchStatement.CountryCodes.mx` |
| `FM` | `aws.wafv2.WebAcl.Rule.Statement.GeoMatchStatement.CountryCodes.fm` |
| `MD` | `aws.wafv2.WebAcl.Rule.Statement.GeoMatchStatement.CountryCodes.md` |
| `MC` | `aws.wafv2.WebAcl.Rule.Statement.GeoMatchStatement.CountryCodes.mc` |
| `MN` | `aws.wafv2.WebAcl.Rule.Statement.GeoMatchStatement.CountryCodes.mn` |
| `ME` | `aws.wafv2.WebAcl.Rule.Statement.GeoMatchStatement.CountryCodes.me` |
| `MS` | `aws.wafv2.WebAcl.Rule.Statement.GeoMatchStatement.CountryCodes.ms` |
| `MA` | `aws.wafv2.WebAcl.Rule.Statement.GeoMatchStatement.CountryCodes.ma` |
| `MZ` | `aws.wafv2.WebAcl.Rule.Statement.GeoMatchStatement.CountryCodes.mz` |
| `MM` | `aws.wafv2.WebAcl.Rule.Statement.GeoMatchStatement.CountryCodes.mm` |
| `NA` | `aws.wafv2.WebAcl.Rule.Statement.GeoMatchStatement.CountryCodes.na` |
| `NR` | `aws.wafv2.WebAcl.Rule.Statement.GeoMatchStatement.CountryCodes.nr` |
| `NP` | `aws.wafv2.WebAcl.Rule.Statement.GeoMatchStatement.CountryCodes.np` |
| `NL` | `aws.wafv2.WebAcl.Rule.Statement.GeoMatchStatement.CountryCodes.nl` |
| `NC` | `aws.wafv2.WebAcl.Rule.Statement.GeoMatchStatement.CountryCodes.nc` |
| `NZ` | `aws.wafv2.WebAcl.Rule.Statement.GeoMatchStatement.CountryCodes.nz` |
| `NI` | `aws.wafv2.WebAcl.Rule.Statement.GeoMatchStatement.CountryCodes.ni` |
| `NE` | `aws.wafv2.WebAcl.Rule.Statement.GeoMatchStatement.CountryCodes.ne` |
| `NG` | `aws.wafv2.WebAcl.Rule.Statement.GeoMatchStatement.CountryCodes.ng` |
| `NU` | `aws.wafv2.WebAcl.Rule.Statement.GeoMatchStatement.CountryCodes.nu` |
| `NF` | `aws.wafv2.WebAcl.Rule.Statement.GeoMatchStatement.CountryCodes.nf` |
| `MP` | `aws.wafv2.WebAcl.Rule.Statement.GeoMatchStatement.CountryCodes.mp` |
| `NO` | `aws.wafv2.WebAcl.Rule.Statement.GeoMatchStatement.CountryCodes.no` |
| `OM` | `aws.wafv2.WebAcl.Rule.Statement.GeoMatchStatement.CountryCodes.om` |
| `PK` | `aws.wafv2.WebAcl.Rule.Statement.GeoMatchStatement.CountryCodes.pk` |
| `PW` | `aws.wafv2.WebAcl.Rule.Statement.GeoMatchStatement.CountryCodes.pw` |
| `PS` | `aws.wafv2.WebAcl.Rule.Statement.GeoMatchStatement.CountryCodes.ps` |
| `PA` | `aws.wafv2.WebAcl.Rule.Statement.GeoMatchStatement.CountryCodes.pa` |
| `PG` | `aws.wafv2.WebAcl.Rule.Statement.GeoMatchStatement.CountryCodes.pg` |
| `PY` | `aws.wafv2.WebAcl.Rule.Statement.GeoMatchStatement.CountryCodes.py` |
| `PE` | `aws.wafv2.WebAcl.Rule.Statement.GeoMatchStatement.CountryCodes.pe` |
| `PH` | `aws.wafv2.WebAcl.Rule.Statement.GeoMatchStatement.CountryCodes.ph` |
| `PN` | `aws.wafv2.WebAcl.Rule.Statement.GeoMatchStatement.CountryCodes.pn` |
| `PL` | `aws.wafv2.WebAcl.Rule.Statement.GeoMatchStatement.CountryCodes.pl` |
| `PT` | `aws.wafv2.WebAcl.Rule.Statement.GeoMatchStatement.CountryCodes.pt` |
| `PR` | `aws.wafv2.WebAcl.Rule.Statement.GeoMatchStatement.CountryCodes.pr` |
| `QA` | `aws.wafv2.WebAcl.Rule.Statement.GeoMatchStatement.CountryCodes.qa` |
| `RE` | `aws.wafv2.WebAcl.Rule.Statement.GeoMatchStatement.CountryCodes.re` |
| `RO` | `aws.wafv2.WebAcl.Rule.Statement.GeoMatchStatement.CountryCodes.ro` |
| `RU` | `aws.wafv2.WebAcl.Rule.Statement.GeoMatchStatement.CountryCodes.ru` |
| `RW` | `aws.wafv2.WebAcl.Rule.Statement.GeoMatchStatement.CountryCodes.rw` |
| `BL` | `aws.wafv2.WebAcl.Rule.Statement.GeoMatchStatement.CountryCodes.bl` |
| `SH` | `aws.wafv2.WebAcl.Rule.Statement.GeoMatchStatement.CountryCodes.sh` |
| `KN` | `aws.wafv2.WebAcl.Rule.Statement.GeoMatchStatement.CountryCodes.kn` |
| `LC` | `aws.wafv2.WebAcl.Rule.Statement.GeoMatchStatement.CountryCodes.lc` |
| `MF` | `aws.wafv2.WebAcl.Rule.Statement.GeoMatchStatement.CountryCodes.mf` |
| `PM` | `aws.wafv2.WebAcl.Rule.Statement.GeoMatchStatement.CountryCodes.pm` |
| `VC` | `aws.wafv2.WebAcl.Rule.Statement.GeoMatchStatement.CountryCodes.vc` |
| `WS` | `aws.wafv2.WebAcl.Rule.Statement.GeoMatchStatement.CountryCodes.ws` |
| `SM` | `aws.wafv2.WebAcl.Rule.Statement.GeoMatchStatement.CountryCodes.sm` |
| `ST` | `aws.wafv2.WebAcl.Rule.Statement.GeoMatchStatement.CountryCodes.st` |
| `SA` | `aws.wafv2.WebAcl.Rule.Statement.GeoMatchStatement.CountryCodes.sa` |
| `SN` | `aws.wafv2.WebAcl.Rule.Statement.GeoMatchStatement.CountryCodes.sn` |
| `RS` | `aws.wafv2.WebAcl.Rule.Statement.GeoMatchStatement.CountryCodes.rs` |
| `SC` | `aws.wafv2.WebAcl.Rule.Statement.GeoMatchStatement.CountryCodes.sc` |
| `SL` | `aws.wafv2.WebAcl.Rule.Statement.GeoMatchStatement.CountryCodes.sl` |
| `SG` | `aws.wafv2.WebAcl.Rule.Statement.GeoMatchStatement.CountryCodes.sg` |
| `SX` | `aws.wafv2.WebAcl.Rule.Statement.GeoMatchStatement.CountryCodes.sx` |
| `SK` | `aws.wafv2.WebAcl.Rule.Statement.GeoMatchStatement.CountryCodes.sk` |
| `SI` | `aws.wafv2.WebAcl.Rule.Statement.GeoMatchStatement.CountryCodes.si` |
| `SB` | `aws.wafv2.WebAcl.Rule.Statement.GeoMatchStatement.CountryCodes.sb` |
| `SO` | `aws.wafv2.WebAcl.Rule.Statement.GeoMatchStatement.CountryCodes.so` |
| `ZA` | `aws.wafv2.WebAcl.Rule.Statement.GeoMatchStatement.CountryCodes.za` |
| `GS` | `aws.wafv2.WebAcl.Rule.Statement.GeoMatchStatement.CountryCodes.gs` |
| `SS` | `aws.wafv2.WebAcl.Rule.Statement.GeoMatchStatement.CountryCodes.ss` |
| `ES` | `aws.wafv2.WebAcl.Rule.Statement.GeoMatchStatement.CountryCodes.es` |
| `LK` | `aws.wafv2.WebAcl.Rule.Statement.GeoMatchStatement.CountryCodes.lk` |
| `SD` | `aws.wafv2.WebAcl.Rule.Statement.GeoMatchStatement.CountryCodes.sd` |
| `SR` | `aws.wafv2.WebAcl.Rule.Statement.GeoMatchStatement.CountryCodes.sr` |
| `SJ` | `aws.wafv2.WebAcl.Rule.Statement.GeoMatchStatement.CountryCodes.sj` |
| `SZ` | `aws.wafv2.WebAcl.Rule.Statement.GeoMatchStatement.CountryCodes.sz` |
| `SE` | `aws.wafv2.WebAcl.Rule.Statement.GeoMatchStatement.CountryCodes.se` |
| `CH` | `aws.wafv2.WebAcl.Rule.Statement.GeoMatchStatement.CountryCodes.ch` |
| `SY` | `aws.wafv2.WebAcl.Rule.Statement.GeoMatchStatement.CountryCodes.sy` |
| `TW` | `aws.wafv2.WebAcl.Rule.Statement.GeoMatchStatement.CountryCodes.tw` |
| `TJ` | `aws.wafv2.WebAcl.Rule.Statement.GeoMatchStatement.CountryCodes.tj` |
| `TZ` | `aws.wafv2.WebAcl.Rule.Statement.GeoMatchStatement.CountryCodes.tz` |
| `TH` | `aws.wafv2.WebAcl.Rule.Statement.GeoMatchStatement.CountryCodes.th` |
| `TL` | `aws.wafv2.WebAcl.Rule.Statement.GeoMatchStatement.CountryCodes.tl` |
| `TG` | `aws.wafv2.WebAcl.Rule.Statement.GeoMatchStatement.CountryCodes.tg` |
| `TK` | `aws.wafv2.WebAcl.Rule.Statement.GeoMatchStatement.CountryCodes.tk` |
| `TO` | `aws.wafv2.WebAcl.Rule.Statement.GeoMatchStatement.CountryCodes.to` |
| `TT` | `aws.wafv2.WebAcl.Rule.Statement.GeoMatchStatement.CountryCodes.tt` |
| `TN` | `aws.wafv2.WebAcl.Rule.Statement.GeoMatchStatement.CountryCodes.tn` |
| `TR` | `aws.wafv2.WebAcl.Rule.Statement.GeoMatchStatement.CountryCodes.tr` |
| `TM` | `aws.wafv2.WebAcl.Rule.Statement.GeoMatchStatement.CountryCodes.tm` |
| `TC` | `aws.wafv2.WebAcl.Rule.Statement.GeoMatchStatement.CountryCodes.tc` |
| `TV` | `aws.wafv2.WebAcl.Rule.Statement.GeoMatchStatement.CountryCodes.tv` |
| `UG` | `aws.wafv2.WebAcl.Rule.Statement.GeoMatchStatement.CountryCodes.ug` |
| `UA` | `aws.wafv2.WebAcl.Rule.Statement.GeoMatchStatement.CountryCodes.ua` |
| `AE` | `aws.wafv2.WebAcl.Rule.Statement.GeoMatchStatement.CountryCodes.ae` |
| `GB` | `aws.wafv2.WebAcl.Rule.Statement.GeoMatchStatement.CountryCodes.gb` |
| `US` | `aws.wafv2.WebAcl.Rule.Statement.GeoMatchStatement.CountryCodes.us` |
| `UM` | `aws.wafv2.WebAcl.Rule.Statement.GeoMatchStatement.CountryCodes.um` |
| `UY` | `aws.wafv2.WebAcl.Rule.Statement.GeoMatchStatement.CountryCodes.uy` |
| `UZ` | `aws.wafv2.WebAcl.Rule.Statement.GeoMatchStatement.CountryCodes.uz` |
| `VU` | `aws.wafv2.WebAcl.Rule.Statement.GeoMatchStatement.CountryCodes.vu` |
| `VE` | `aws.wafv2.WebAcl.Rule.Statement.GeoMatchStatement.CountryCodes.ve` |
| `VN` | `aws.wafv2.WebAcl.Rule.Statement.GeoMatchStatement.CountryCodes.vn` |
| `VG` | `aws.wafv2.WebAcl.Rule.Statement.GeoMatchStatement.CountryCodes.vg` |
| `VI` | `aws.wafv2.WebAcl.Rule.Statement.GeoMatchStatement.CountryCodes.vi` |
| `WF` | `aws.wafv2.WebAcl.Rule.Statement.GeoMatchStatement.CountryCodes.wf` |
| `EH` | `aws.wafv2.WebAcl.Rule.Statement.GeoMatchStatement.CountryCodes.eh` |
| `YE` | `aws.wafv2.WebAcl.Rule.Statement.GeoMatchStatement.CountryCodes.ye` |
| `ZM` | `aws.wafv2.WebAcl.Rule.Statement.GeoMatchStatement.CountryCodes.zm` |
| `ZW` | `aws.wafv2.WebAcl.Rule.Statement.GeoMatchStatement.CountryCodes.zw` |
| `XK` | `aws.wafv2.WebAcl.Rule.Statement.GeoMatchStatement.CountryCodes.xk` |

Shorthand formats: `af` or `CountryCodes.af`

### statement.ip_set_reference_statement.ip_set_forwarded_ip_config.fallback_behavior (FallbackBehavior)

| Value | DSL Identifier |
|-------|----------------|
| `MATCH` | `aws.wafv2.WebAcl.Rule.Statement.IpSetReferenceStatement.IpSetForwardedIpConfiguration.FallbackBehavior.match` |
| `NO_MATCH` | `aws.wafv2.WebAcl.Rule.Statement.IpSetReferenceStatement.IpSetForwardedIpConfiguration.FallbackBehavior.no_match` |

Shorthand formats: `match` or `FallbackBehavior.match`

### statement.ip_set_reference_statement.ip_set_forwarded_ip_config.position (Position)

| Value | DSL Identifier |
|-------|----------------|
| `FIRST` | `aws.wafv2.WebAcl.Rule.Statement.IpSetReferenceStatement.IpSetForwardedIpConfiguration.Position.first` |
| `LAST` | `aws.wafv2.WebAcl.Rule.Statement.IpSetReferenceStatement.IpSetForwardedIpConfiguration.Position.last` |
| `ANY` | `aws.wafv2.WebAcl.Rule.Statement.IpSetReferenceStatement.IpSetForwardedIpConfiguration.Position.any` |

Shorthand formats: `first` or `Position.first`

### statement.label_match_statement.scope (LabelMatchScope)

| Value | DSL Identifier |
|-------|----------------|
| `LABEL` | `aws.wafv2.WebAcl.Rule.Statement.LabelMatchStatement.LabelMatchScope.label` |
| `NAMESPACE` | `aws.wafv2.WebAcl.Rule.Statement.LabelMatchStatement.LabelMatchScope.namespace` |

Shorthand formats: `label` or `LabelMatchScope.label`

### statement.managed_rule_group_statement.managed_rule_group_configs.aws_managed_rules_acfp_rule_set.request_inspection.payload_type (PayloadType)

| Value | DSL Identifier |
|-------|----------------|
| `JSON` | `aws.wafv2.WebAcl.Rule.Statement.ManagedRuleGroupStatement.ManagedRuleGroupConfig.AwsManagedRulesAcfpRuleSet.RequestInspectionAcfp.PayloadType.json` |
| `FORM_ENCODED` | `aws.wafv2.WebAcl.Rule.Statement.ManagedRuleGroupStatement.ManagedRuleGroupConfig.AwsManagedRulesAcfpRuleSet.RequestInspectionAcfp.PayloadType.form_encoded` |

Shorthand formats: `json` or `PayloadType.json`

### statement.managed_rule_group_statement.managed_rule_group_configs.aws_managed_rules_anti_d_do_s_rule_set.client_side_action_config.challenge.sensitivity (SensitivityToAct)

| Value | DSL Identifier |
|-------|----------------|
| `LOW` | `aws.wafv2.WebAcl.Rule.Statement.ManagedRuleGroupStatement.ManagedRuleGroupConfig.AwsManagedRulesAntiDDoSRuleSet.ClientSideActionConfig.ClientSideAction.SensitivityToAct.low` |
| `MEDIUM` | `aws.wafv2.WebAcl.Rule.Statement.ManagedRuleGroupStatement.ManagedRuleGroupConfig.AwsManagedRulesAntiDDoSRuleSet.ClientSideActionConfig.ClientSideAction.SensitivityToAct.medium` |
| `HIGH` | `aws.wafv2.WebAcl.Rule.Statement.ManagedRuleGroupStatement.ManagedRuleGroupConfig.AwsManagedRulesAntiDDoSRuleSet.ClientSideActionConfig.ClientSideAction.SensitivityToAct.high` |

Shorthand formats: `low` or `SensitivityToAct.low`

### statement.managed_rule_group_statement.managed_rule_group_configs.aws_managed_rules_anti_d_do_s_rule_set.client_side_action_config.challenge.usage_of_action (UsageOfAction)

| Value | DSL Identifier |
|-------|----------------|
| `ENABLED` | `aws.wafv2.WebAcl.Rule.Statement.ManagedRuleGroupStatement.ManagedRuleGroupConfig.AwsManagedRulesAntiDDoSRuleSet.ClientSideActionConfig.ClientSideAction.UsageOfAction.enabled` |
| `DISABLED` | `aws.wafv2.WebAcl.Rule.Statement.ManagedRuleGroupStatement.ManagedRuleGroupConfig.AwsManagedRulesAntiDDoSRuleSet.ClientSideActionConfig.ClientSideAction.UsageOfAction.disabled` |

Shorthand formats: `enabled` or `UsageOfAction.enabled`

### statement.managed_rule_group_statement.managed_rule_group_configs.aws_managed_rules_anti_d_do_s_rule_set.sensitivity_to_block (SensitivityToAct)

| Value | DSL Identifier |
|-------|----------------|
| `LOW` | `aws.wafv2.WebAcl.Rule.Statement.ManagedRuleGroupStatement.ManagedRuleGroupConfig.AwsManagedRulesAntiDDoSRuleSet.SensitivityToAct.low` |
| `MEDIUM` | `aws.wafv2.WebAcl.Rule.Statement.ManagedRuleGroupStatement.ManagedRuleGroupConfig.AwsManagedRulesAntiDDoSRuleSet.SensitivityToAct.medium` |
| `HIGH` | `aws.wafv2.WebAcl.Rule.Statement.ManagedRuleGroupStatement.ManagedRuleGroupConfig.AwsManagedRulesAntiDDoSRuleSet.SensitivityToAct.high` |

Shorthand formats: `low` or `SensitivityToAct.low`

### statement.managed_rule_group_statement.managed_rule_group_configs.aws_managed_rules_atp_rule_set.request_inspection.payload_type (PayloadType)

| Value | DSL Identifier |
|-------|----------------|
| `JSON` | `aws.wafv2.WebAcl.Rule.Statement.ManagedRuleGroupStatement.ManagedRuleGroupConfig.AwsManagedRulesAtpRuleSet.RequestInspection.PayloadType.json` |
| `FORM_ENCODED` | `aws.wafv2.WebAcl.Rule.Statement.ManagedRuleGroupStatement.ManagedRuleGroupConfig.AwsManagedRulesAtpRuleSet.RequestInspection.PayloadType.form_encoded` |

Shorthand formats: `json` or `PayloadType.json`

### statement.managed_rule_group_statement.managed_rule_group_configs.aws_managed_rules_bot_control_rule_set.inspection_level (InspectionLevel)

| Value | DSL Identifier |
|-------|----------------|
| `COMMON` | `aws.wafv2.WebAcl.Rule.Statement.ManagedRuleGroupStatement.ManagedRuleGroupConfig.AwsManagedRulesBotControlRuleSet.InspectionLevel.common` |
| `TARGETED` | `aws.wafv2.WebAcl.Rule.Statement.ManagedRuleGroupStatement.ManagedRuleGroupConfig.AwsManagedRulesBotControlRuleSet.InspectionLevel.targeted` |

Shorthand formats: `common` or `InspectionLevel.common`

### statement.managed_rule_group_statement.managed_rule_group_configs.payload_type (PayloadType)

| Value | DSL Identifier |
|-------|----------------|
| `JSON` | `aws.wafv2.WebAcl.Rule.Statement.ManagedRuleGroupStatement.ManagedRuleGroupConfig.PayloadType.json` |
| `FORM_ENCODED` | `aws.wafv2.WebAcl.Rule.Statement.ManagedRuleGroupStatement.ManagedRuleGroupConfig.PayloadType.form_encoded` |

Shorthand formats: `json` or `PayloadType.json`

### statement.rate_based_statement.aggregate_key_type (AggregateKeyType)

| Value | DSL Identifier |
|-------|----------------|
| `CONSTANT` | `aws.wafv2.WebAcl.Rule.Statement.RateBasedStatement.AggregateKeyType.constant` |
| `IP` | `aws.wafv2.WebAcl.Rule.Statement.RateBasedStatement.AggregateKeyType.ip` |
| `FORWARDED_IP` | `aws.wafv2.WebAcl.Rule.Statement.RateBasedStatement.AggregateKeyType.forwarded_ip` |
| `CUSTOM_KEYS` | `aws.wafv2.WebAcl.Rule.Statement.RateBasedStatement.AggregateKeyType.custom_keys` |

Shorthand formats: `constant` or `AggregateKeyType.constant`

### statement.rate_based_statement.custom_keys.ja3_fingerprint.fallback_behavior (FallbackBehavior)

| Value | DSL Identifier |
|-------|----------------|
| `MATCH` | `aws.wafv2.WebAcl.Rule.Statement.RateBasedStatement.RateBasedStatementCustomKey.RateLimitJa3Fingerprint.FallbackBehavior.match` |
| `NO_MATCH` | `aws.wafv2.WebAcl.Rule.Statement.RateBasedStatement.RateBasedStatementCustomKey.RateLimitJa3Fingerprint.FallbackBehavior.no_match` |

Shorthand formats: `match` or `FallbackBehavior.match`

### statement.rate_based_statement.custom_keys.ja4_fingerprint.fallback_behavior (FallbackBehavior)

| Value | DSL Identifier |
|-------|----------------|
| `MATCH` | `aws.wafv2.WebAcl.Rule.Statement.RateBasedStatement.RateBasedStatementCustomKey.RateLimitJa4Fingerprint.FallbackBehavior.match` |
| `NO_MATCH` | `aws.wafv2.WebAcl.Rule.Statement.RateBasedStatement.RateBasedStatementCustomKey.RateLimitJa4Fingerprint.FallbackBehavior.no_match` |

Shorthand formats: `match` or `FallbackBehavior.match`

### statement.size_constraint_statement.comparison_operator (ComparisonOperator)

| Value | DSL Identifier |
|-------|----------------|
| `EQ` | `aws.wafv2.WebAcl.Rule.Statement.SizeConstraintStatement.ComparisonOperator.eq` |
| `NE` | `aws.wafv2.WebAcl.Rule.Statement.SizeConstraintStatement.ComparisonOperator.ne` |
| `LE` | `aws.wafv2.WebAcl.Rule.Statement.SizeConstraintStatement.ComparisonOperator.le` |
| `LT` | `aws.wafv2.WebAcl.Rule.Statement.SizeConstraintStatement.ComparisonOperator.lt` |
| `GE` | `aws.wafv2.WebAcl.Rule.Statement.SizeConstraintStatement.ComparisonOperator.ge` |
| `GT` | `aws.wafv2.WebAcl.Rule.Statement.SizeConstraintStatement.ComparisonOperator.gt` |

Shorthand formats: `eq` or `ComparisonOperator.eq`

### statement.sqli_match_statement.sensitivity_level (SensitivityLevel)

| Value | DSL Identifier |
|-------|----------------|
| `LOW` | `aws.wafv2.WebAcl.Rule.Statement.SqliMatchStatement.SensitivityLevel.low` |
| `HIGH` | `aws.wafv2.WebAcl.Rule.Statement.SqliMatchStatement.SensitivityLevel.high` |

Shorthand formats: `low` or `SensitivityLevel.low`

### text_transformation.type (TextTransformationType)

| Value | DSL Identifier |
|-------|----------------|
| `NONE` | `aws.wafv2.WebAcl.Rule.Statement.ByteMatchStatement.TextTransformation.TextTransformationType.none` |
| `COMPRESS_WHITE_SPACE` | `aws.wafv2.WebAcl.Rule.Statement.ByteMatchStatement.TextTransformation.TextTransformationType.compress_white_space` |
| `HTML_ENTITY_DECODE` | `aws.wafv2.WebAcl.Rule.Statement.ByteMatchStatement.TextTransformation.TextTransformationType.html_entity_decode` |
| `LOWERCASE` | `aws.wafv2.WebAcl.Rule.Statement.ByteMatchStatement.TextTransformation.TextTransformationType.lowercase` |
| `CMD_LINE` | `aws.wafv2.WebAcl.Rule.Statement.ByteMatchStatement.TextTransformation.TextTransformationType.cmd_line` |
| `URL_DECODE` | `aws.wafv2.WebAcl.Rule.Statement.ByteMatchStatement.TextTransformation.TextTransformationType.url_decode` |
| `BASE64_DECODE` | `aws.wafv2.WebAcl.Rule.Statement.ByteMatchStatement.TextTransformation.TextTransformationType.base64_decode` |
| `HEX_DECODE` | `aws.wafv2.WebAcl.Rule.Statement.ByteMatchStatement.TextTransformation.TextTransformationType.hex_decode` |
| `MD5` | `aws.wafv2.WebAcl.Rule.Statement.ByteMatchStatement.TextTransformation.TextTransformationType.md5` |
| `REPLACE_COMMENTS` | `aws.wafv2.WebAcl.Rule.Statement.ByteMatchStatement.TextTransformation.TextTransformationType.replace_comments` |
| `ESCAPE_SEQ_DECODE` | `aws.wafv2.WebAcl.Rule.Statement.ByteMatchStatement.TextTransformation.TextTransformationType.escape_seq_decode` |
| `SQL_HEX_DECODE` | `aws.wafv2.WebAcl.Rule.Statement.ByteMatchStatement.TextTransformation.TextTransformationType.sql_hex_decode` |
| `CSS_DECODE` | `aws.wafv2.WebAcl.Rule.Statement.ByteMatchStatement.TextTransformation.TextTransformationType.css_decode` |
| `JS_DECODE` | `aws.wafv2.WebAcl.Rule.Statement.ByteMatchStatement.TextTransformation.TextTransformationType.js_decode` |
| `NORMALIZE_PATH` | `aws.wafv2.WebAcl.Rule.Statement.ByteMatchStatement.TextTransformation.TextTransformationType.normalize_path` |
| `NORMALIZE_PATH_WIN` | `aws.wafv2.WebAcl.Rule.Statement.ByteMatchStatement.TextTransformation.TextTransformationType.normalize_path_win` |
| `REMOVE_NULLS` | `aws.wafv2.WebAcl.Rule.Statement.ByteMatchStatement.TextTransformation.TextTransformationType.remove_nulls` |
| `REPLACE_NULLS` | `aws.wafv2.WebAcl.Rule.Statement.ByteMatchStatement.TextTransformation.TextTransformationType.replace_nulls` |
| `BASE64_DECODE_EXT` | `aws.wafv2.WebAcl.Rule.Statement.ByteMatchStatement.TextTransformation.TextTransformationType.base64_decode_ext` |
| `URL_DECODE_UNI` | `aws.wafv2.WebAcl.Rule.Statement.ByteMatchStatement.TextTransformation.TextTransformationType.url_decode_uni` |
| `UTF8_TO_UNICODE` | `aws.wafv2.WebAcl.Rule.Statement.ByteMatchStatement.TextTransformation.TextTransformationType.utf8_to_unicode` |

Shorthand formats: `none` or `TextTransformationType.none`

## Struct Definitions

### allow_action

Used by: `default_action.allow`, `rules.action.allow`

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `custom_request_handling` | [Struct(CustomRequestHandling)](#custom_request_handling) | No |  |

### application_config

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `attributes` | List\<[Struct(ApplicationAttribute)](#application_configattributes)\> (items: 1..=10) | Yes |  |

### application_config.attributes

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `name` | String(pattern, len: 1..=64) | Yes |  |
| `values` | `List<String(pattern, len: 1..=64)>` (items: 1..=10) | Yes |  |

### association_config

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `request_body` | Map\<[Enum (RequestBody)](#association_configrequest_body-requestbody), [Struct(RequestBodyAssociatedResourceTypeConfig)](#association_configrequest_body)\> | No |  |

### association_config.request_body

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `default_size_inspection_limit` | [Enum (SizeInspectionLimit)](#association_configrequest_bodydefault_size_inspection_limit-sizeinspectionlimit) | Yes |  |

### block_action

Used by: `default_action.block`, `rules.action.block`

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `custom_response` | [Struct(CustomResponse)](#block_actioncustom_response) | No |  |

### block_action.custom_response

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `custom_response_body_key` | `String(pattern, len: 1..=128)` | No | Custom response body key. |
| `response_code` | Int(200..=599) | Yes |  |
| `response_headers` | List\<[Struct(CustomHTTPHeader)](#custom_http_header)\> (items: 1..) | No | Collection of HTTP headers. |

### captcha_config

Used by: `captcha_config`, `rules.captcha_config`

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `immunity_time_property` | [Struct(ImmunityTimeProperty)](#immunity_time_property) | No |  |

### challenge_config

Used by: `challenge_config`, `rules.challenge_config`

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `immunity_time_property` | [Struct(ImmunityTimeProperty)](#immunity_time_property) | No |  |

### custom_http_header

Used by: `block_action.custom_response.response_headers`, `custom_request_handling.insert_headers`

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `name` | `String(pattern, len: 1..=64)` | Yes |  |
| `value` | String(len: 1..=255) | Yes |  |

### custom_request_handling

Used by: `allow_action.custom_request_handling`, `rule_action.captcha.custom_request_handling`, `rule_action.challenge.custom_request_handling`, `rule_action.count.custom_request_handling`

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `insert_headers` | List\<[Struct(CustomHTTPHeader)](#custom_http_header)\> (items: 1..) | Yes | Collection of HTTP headers. |

### custom_response_bodies

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `content` | String(len: 1..=10240) | Yes |  |
| `content_type` | [Enum (ResponseContentType)](#custom_response_bodiescontent_type-responsecontenttype) | Yes |  |

### data_protection_config

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `data_protections` | List\<[Struct(DataProtect)](#data_protection_configdata_protections)\> (items: 1..=26) | Yes |  |

### data_protection_config.data_protections

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `action` | [Enum (DataProtectionAction)](#data_protection_configdata_protectionsaction-dataprotectionaction) | Yes |  |
| `exclude_rate_based_details` | Bool | No |  |
| `exclude_rule_match_details` | Bool | No |  |
| `field` | [Struct(FieldToProtect)](#data_protection_configdata_protectionsfield) | Yes |  |

### data_protection_config.data_protections.field

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `field_keys` | `List<String(pattern, len: 1..=64)>` (items: ..=100) | No | List of field keys to protect |
| `field_type` | [Enum (FieldType)](#data_protection_configdata_protectionsfieldfield_type-fieldtype) | Yes | Field type to protect |

### default_action

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `allow` | [Struct(AllowAction)](#allow_action) | No |  |
| `block` | [Struct(BlockAction)](#block_action) | No |  |

### excluded_rule

Used by: `statement.managed_rule_group_statement.excluded_rules`, `statement.rule_group_reference_statement.excluded_rules`

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `name` | `String(pattern, len: 1..=128)` | Yes |  |

### field_identifier

Used by: `statement.managed_rule_group_statement.managed_rule_group_configs.aws_managed_rules_acfp_rule_set.request_inspection.address_fields`, `statement.managed_rule_group_statement.managed_rule_group_configs.aws_managed_rules_acfp_rule_set.request_inspection.email_field`, `statement.managed_rule_group_statement.managed_rule_group_configs.aws_managed_rules_acfp_rule_set.request_inspection.password_field`, `statement.managed_rule_group_statement.managed_rule_group_configs.aws_managed_rules_acfp_rule_set.request_inspection.phone_number_fields`, `statement.managed_rule_group_statement.managed_rule_group_configs.aws_managed_rules_acfp_rule_set.request_inspection.username_field`, `statement.managed_rule_group_statement.managed_rule_group_configs.aws_managed_rules_atp_rule_set.request_inspection.password_field`, `statement.managed_rule_group_statement.managed_rule_group_configs.aws_managed_rules_atp_rule_set.request_inspection.username_field`, `statement.managed_rule_group_statement.managed_rule_group_configs.password_field`, `statement.managed_rule_group_statement.managed_rule_group_configs.username_field`

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `identifier` | String(pattern, len: 1..=512) | Yes |  |

### field_to_match

Used by: `statement.byte_match_statement.field_to_match`, `statement.regex_match_statement.field_to_match`, `statement.regex_pattern_set_reference_statement.field_to_match`, `statement.size_constraint_statement.field_to_match`, `statement.sqli_match_statement.field_to_match`, `statement.xss_match_statement.field_to_match`

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `all_query_arguments` | Empty struct (`{}`) | No | All query arguments of a web request. |
| `body` | [Struct(Body)](#field_to_matchbody) | No |  |
| `cookies` | [Struct(Cookies)](#field_to_matchcookies) | No |  |
| `header_order` | [Struct(HeaderOrder)](#field_to_matchheader_order) | No |  |
| `headers` | [Struct(Headers)](#field_to_matchheaders) | No |  |
| `ja3_fingerprint` | [Struct(JA3Fingerprint)](#field_to_matchja3_fingerprint) | No |  |
| `ja4_fingerprint` | [Struct(JA4Fingerprint)](#field_to_matchja4_fingerprint) | No |  |
| `json_body` | [Struct(JsonBody)](#field_to_matchjson_body) | No |  |
| `method` | Empty struct (`{}`) | No | The HTTP method of a web request. The method indicates the type of operation that the request is asking the origin to perform. |
| `query_string` | Empty struct (`{}`) | No | The query string of a web request. This is the part of a URL that appears after a ? character, if any. |
| `single_header` | [Struct(SingleHeader)](#field_to_matchsingle_header) | No |  |
| `single_query_argument` | [Struct(SingleQueryArgument)](#field_to_matchsingle_query_argument) | No | One query argument in a web request, identified by name, for example UserName or SalesRegion. The name can be up to 30 characters long and isn't case sensitive. |
| `uri_fragment` | [Struct(UriFragment)](#field_to_matchuri_fragment) | No |  |
| `uri_path` | Empty struct (`{}`) | No | The path component of the URI of a web request. This is the part of a web request that identifies a resource, for example, /images/daily-ad.jpg. |

### field_to_match.body

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `oversize_handling` | [Enum (OversizeHandling)](#field_to_matchbodyoversize_handling-oversizehandling) | No |  |

### field_to_match.cookies

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `match_pattern` | [Struct(CookieMatchPattern)](#field_to_matchcookiesmatch_pattern) | Yes |  |
| `match_scope` | [Enum (MapMatchScope)](#field_to_matchcookiesmatch_scope-mapmatchscope) | Yes |  |
| `oversize_handling` | [Enum (OversizeHandling)](#field_to_matchcookiesoversize_handling-oversizehandling) | Yes |  |

### field_to_match.cookies.match_pattern

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `all` | Empty struct (`{}`) | No | Inspect all parts of the web request cookies. |
| `excluded_cookies` | `List<String(pattern, len: 1..=60)>` (items: 1..=199) | No |  |
| `included_cookies` | `List<String(pattern, len: 1..=60)>` (items: 1..=199) | No |  |

### field_to_match.header_order

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `oversize_handling` | [Enum (OversizeHandling)](#field_to_matchheader_orderoversize_handling-oversizehandling) | Yes |  |

### field_to_match.headers

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `match_pattern` | [Struct(HeaderMatchPattern)](#field_to_matchheadersmatch_pattern) | Yes |  |
| `match_scope` | [Enum (MapMatchScope)](#field_to_matchheadersmatch_scope-mapmatchscope) | Yes |  |
| `oversize_handling` | [Enum (OversizeHandling)](#field_to_matchheadersoversize_handling-oversizehandling) | Yes |  |

### field_to_match.headers.match_pattern

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `all` | Empty struct (`{}`) | No | Inspect all parts of the web request headers. |
| `excluded_headers` | `List<String(pattern, len: 1..=64)>` (items: 1..=199) | No |  |
| `included_headers` | `List<String(pattern, len: 1..=64)>` (items: 1..=199) | No |  |

### field_to_match.ja3_fingerprint

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `fallback_behavior` | [Enum (FallbackBehavior)](#field_to_matchja3_fingerprintfallback_behavior-fallbackbehavior) | Yes |  |

### field_to_match.ja4_fingerprint

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `fallback_behavior` | [Enum (FallbackBehavior)](#field_to_matchja4_fingerprintfallback_behavior-fallbackbehavior) | Yes |  |

### field_to_match.json_body

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `invalid_fallback_behavior` | [Enum (BodyParsingFallbackBehavior)](#field_to_matchjson_bodyinvalid_fallback_behavior-bodyparsingfallbackbehavior) | No |  |
| `match_pattern` | [Struct(JsonMatchPattern)](#field_to_matchjson_bodymatch_pattern) | Yes |  |
| `match_scope` | [Enum (JsonMatchScope)](#field_to_matchjson_bodymatch_scope-jsonmatchscope) | Yes |  |
| `oversize_handling` | [Enum (OversizeHandling)](#field_to_matchjson_bodyoversize_handling-oversizehandling) | No |  |

### field_to_match.json_body.match_pattern

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `all` | Empty struct (`{}`) | No | Inspect all parts of the web request's JSON body. |
| `included_paths` | `List<String(pattern, len: 1..=512)>` (items: 1..) | No |  |

### field_to_match.single_header

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `name` | `String(pattern, len: 1..=64)` | Yes |  |

### field_to_match.single_query_argument

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `name` | `String(pattern, len: 1..=64)` | Yes |  |

### field_to_match.uri_fragment

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `fallback_behavior` | [Enum (FallbackBehavior)](#field_to_matchuri_fragmentfallback_behavior-fallbackbehavior) | No |  |

### forwarded_ip_configuration

Used by: `statement.asn_match_statement.forwarded_ip_config`, `statement.geo_match_statement.forwarded_ip_config`, `statement.rate_based_statement.forwarded_ip_config`

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `fallback_behavior` | [Enum (FallbackBehavior)](#forwarded_ip_configurationfallback_behavior-fallbackbehavior) | Yes |  |
| `header_name` | `String(pattern, len: 1..=255)` | Yes |  |

### immunity_time_property

Used by: `captcha_config.immunity_time_property`, `challenge_config.immunity_time_property`

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `immunity_time` | Int(60..=259200) | Yes |  |

### on_source_d_do_s_protection_config

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `alb_low_reputation_mode` | [Enum (AlbLowReputationMode)](#on_source_d_do_s_protection_configalb_low_reputation_mode-alblowreputationmode) | Yes |  |

### response_inspection

Used by: `statement.managed_rule_group_statement.managed_rule_group_configs.aws_managed_rules_acfp_rule_set.response_inspection`, `statement.managed_rule_group_statement.managed_rule_group_configs.aws_managed_rules_atp_rule_set.response_inspection`

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `body_contains` | [Struct(ResponseInspectionBodyContains)](#response_inspectionbody_contains) | No |  |
| `header` | [Struct(ResponseInspectionHeader)](#response_inspectionheader) | No |  |
| `json` | [Struct(ResponseInspectionJson)](#response_inspectionjson) | No |  |
| `status_code` | [Struct(ResponseInspectionStatusCode)](#response_inspectionstatus_code) | No |  |

### response_inspection.body_contains

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `failure_strings` | `List<String(pattern, len: 1..=100)>` (items: 1..=5) | Yes |  |
| `success_strings` | `List<String(pattern, len: 1..=100)>` (items: 1..=5) | Yes |  |

### response_inspection.header

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `failure_values` | `List<String(pattern, len: 1..=100)>` (items: 1..=3) | Yes |  |
| `name` | String(pattern, len: 1..=200) | Yes |  |
| `success_values` | `List<String(pattern, len: 1..=100)>` (items: 1..=3) | Yes |  |

### response_inspection.json

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `failure_values` | `List<String(pattern, len: 1..=100)>` (items: 1..=5) | Yes |  |
| `identifier` | String(pattern, len: 1..=512) | Yes |  |
| `success_values` | `List<String(pattern, len: 1..=100)>` (items: 1..=5) | Yes |  |

### response_inspection.status_code

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `failure_codes` | `List<Int(0..=999)> (items: 1..=10)` | Yes |  |
| `success_codes` | `List<Int(0..=999)> (items: 1..=10)` | Yes |  |

### rule_action

Used by: `rules.action`, `rules.statement.managed_rule_group_statement.rule_action_overrides.action_to_use`

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `allow` | [Struct(AllowAction)](#allow_action) | No |  |
| `block` | [Struct(BlockAction)](#block_action) | No |  |
| `captcha` | [Struct(CaptchaAction)](#rule_actioncaptcha) | No |  |
| `challenge` | [Struct(ChallengeAction)](#rule_actionchallenge) | No |  |
| `count` | [Struct(CountAction)](#rule_actioncount) | No |  |

### rule_action.captcha

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `custom_request_handling` | [Struct(CustomRequestHandling)](#custom_request_handling) | No |  |

### rule_action.challenge

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `custom_request_handling` | [Struct(CustomRequestHandling)](#custom_request_handling) | No |  |

### rule_action.count

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `custom_request_handling` | [Struct(CustomRequestHandling)](#custom_request_handling) | No |  |

### rule_action_override

Used by: `statement.managed_rule_group_statement.rule_action_overrides`, `statement.rule_group_reference_statement.rule_action_overrides`

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `action_to_use` | [Struct(RuleAction)](#rule_action) | Yes |  |
| `name` | `String(pattern, len: 1..=128)` | Yes |  |

### rules

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `action` | [Struct(RuleAction)](#rule_action) | No |  |
| `captcha_config` | [Struct(CaptchaConfig)](#captcha_config-1) | No |  |
| `challenge_config` | [Struct(ChallengeConfig)](#challenge_config-1) | No |  |
| `name` | `String(pattern, len: 1..=128)` | Yes |  |
| `override_action` | [Struct(OverrideAction)](#rulesoverride_action) | No |  |
| `priority` | Int(0..) | Yes |  |
| `rule_labels` | List\<[Struct(Label)](#rulesrule_labels)\> | No | Collection of Rule Labels. |
| `statement` | [Struct(Statement)](#statement) | Yes |  |
| `visibility_config` | [Struct(VisibilityConfig)](#visibility_config-1) | Yes |  |

### rules.override_action

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `count` | Empty struct (`{}`) | No | Count traffic towards application. |
| `none` | Empty struct (`{}`) | No | Keep the RuleGroup or ManagedRuleGroup behavior as is. |

### rules.rule_labels

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `name` | `String(pattern, len: 1..=1024)` | Yes |  |

### statement

Used by: `rules.statement`, `rules.statement.and_statement.statements`, `rules.statement.managed_rule_group_statement.scope_down_statement`, `rules.statement.not_statement.statement`, `rules.statement.or_statement.statements`, `rules.statement.rate_based_statement.scope_down_statement`

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `and_statement` | [Struct(AndStatement)](#statementand_statement) | No |  |
| `asn_match_statement` | [Struct(AsnMatchStatement)](#statementasn_match_statement) | No |  |
| `byte_match_statement` | [Struct(ByteMatchStatement)](#statementbyte_match_statement) | No |  |
| `geo_match_statement` | [Struct(GeoMatchStatement)](#statementgeo_match_statement) | No |  |
| `ip_set_reference_statement` | [Struct(IPSetReferenceStatement)](#statementip_set_reference_statement) | No |  |
| `label_match_statement` | [Struct(LabelMatchStatement)](#statementlabel_match_statement) | No |  |
| `managed_rule_group_statement` | [Struct(ManagedRuleGroupStatement)](#statementmanaged_rule_group_statement) | No |  |
| `not_statement` | [Struct(NotStatement)](#statementnot_statement) | No |  |
| `or_statement` | [Struct(OrStatement)](#statementor_statement) | No |  |
| `rate_based_statement` | [Struct(RateBasedStatement)](#statementrate_based_statement) | No |  |
| `regex_match_statement` | [Struct(RegexMatchStatement)](#statementregex_match_statement) | No |  |
| `regex_pattern_set_reference_statement` | [Struct(RegexPatternSetReferenceStatement)](#statementregex_pattern_set_reference_statement) | No |  |
| `rule_group_reference_statement` | [Struct(RuleGroupReferenceStatement)](#statementrule_group_reference_statement) | No |  |
| `size_constraint_statement` | [Struct(SizeConstraintStatement)](#statementsize_constraint_statement) | No |  |
| `sqli_match_statement` | [Struct(SqliMatchStatement)](#statementsqli_match_statement) | No |  |
| `xss_match_statement` | [Struct(XssMatchStatement)](#statementxss_match_statement) | No |  |

### statement.and_statement

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `statements` | List\<[Struct(Statement)](#statement)\> (items: 2..) | Yes |  |

### statement.asn_match_statement

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `asn_list` | `List<Int(0..=4294967295)>` (items: 1..=100) | No |  |
| `forwarded_ip_config` | [Struct(ForwardedIPConfiguration)](#forwarded_ip_configuration) | No |  |

### statement.byte_match_statement

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `field_to_match` | [Struct(FieldToMatch)](#field_to_match) | Yes |  |
| `positional_constraint` | [Enum (PositionalConstraint)](#statementbyte_match_statementpositional_constraint-positionalconstraint) | Yes |  |
| `search_string` | aws.wafv2.WebACL.ByteMatchSearchString — identity: String; member: String | No |  |
| `search_string_base64` | aws.wafv2.WebACL.ByteMatchSearchStringBase64 — identity: String; member: String | No |  |
| `text_transformations` | List\<[Struct(TextTransformation)](#text_transformation)\> (items: 1..) | Yes |  |

### statement.geo_match_statement

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `country_codes` | List\<[Enum (CountryCodes)](#statementgeo_match_statementcountry_codes-countrycodes)\> (items: 1..=50) | No |  |
| `forwarded_ip_config` | [Struct(ForwardedIPConfiguration)](#forwarded_ip_configuration) | No |  |

### statement.ip_set_reference_statement

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `arn` | aws.Arn — identity: String(pattern); member: String(len: 20..=2048) | Yes |  |
| `ip_set_forwarded_ip_config` | [Struct(IPSetForwardedIPConfiguration)](#statementip_set_reference_statementip_set_forwarded_ip_config) | No |  |

### statement.ip_set_reference_statement.ip_set_forwarded_ip_config

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `fallback_behavior` | [Enum (FallbackBehavior)](#statementip_set_reference_statementip_set_forwarded_ip_configfallback_behavior-fallbackbehavior) | Yes |  |
| `header_name` | `String(pattern, len: 1..=255)` | Yes |  |
| `position` | [Enum (Position)](#statementip_set_reference_statementip_set_forwarded_ip_configposition-position) | Yes |  |

### statement.label_match_statement

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `key` | `String(pattern, len: 1..=1024)` | Yes |  |
| `scope` | [Enum (LabelMatchScope)](#statementlabel_match_statementscope-labelmatchscope) | Yes |  |

### statement.managed_rule_group_statement

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `excluded_rules` | List\<[Struct(ExcludedRule)](#excluded_rule)\> (items: ..=100) | No |  |
| `managed_rule_group_configs` | List\<[Struct(ManagedRuleGroupConfig)](#statementmanaged_rule_group_statementmanaged_rule_group_configs)\> | No | Collection of ManagedRuleGroupConfig. |
| `name` | String(pattern) | Yes |  |
| `rule_action_overrides` | List\<[Struct(RuleActionOverride)](#rule_action_override)\> (items: ..=100) | No | Action overrides for rules in the rule group. |
| `scope_down_statement` | [Struct(Statement)](#statement) | No |  |
| `vendor_name` | String | Yes |  |
| `version` | String(pattern, len: 1..=64) | No |  |

### statement.managed_rule_group_statement.managed_rule_group_configs

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `aws_managed_rules_acfp_rule_set` | [Struct(AWSManagedRulesACFPRuleSet)](#statementmanaged_rule_group_statementmanaged_rule_group_configsaws_managed_rules_acfp_rule_set) | No |  |
| `aws_managed_rules_atp_rule_set` | [Struct(AWSManagedRulesATPRuleSet)](#statementmanaged_rule_group_statementmanaged_rule_group_configsaws_managed_rules_atp_rule_set) | No |  |
| `aws_managed_rules_anti_d_do_s_rule_set` | [Struct(AWSManagedRulesAntiDDoSRuleSet)](#statementmanaged_rule_group_statementmanaged_rule_group_configsaws_managed_rules_anti_d_do_s_rule_set) | No |  |
| `aws_managed_rules_bot_control_rule_set` | [Struct(AWSManagedRulesBotControlRuleSet)](#statementmanaged_rule_group_statementmanaged_rule_group_configsaws_managed_rules_bot_control_rule_set) | No |  |
| `login_path` | String(pattern, len: 1..=256) | No |  |
| `password_field` | [Struct(FieldIdentifier)](#field_identifier) | No |  |
| `payload_type` | [Enum (PayloadType)](#statementmanaged_rule_group_statementmanaged_rule_group_configspayload_type-payloadtype) | No |  |
| `username_field` | [Struct(FieldIdentifier)](#field_identifier) | No |  |

### statement.managed_rule_group_statement.managed_rule_group_configs.aws_managed_rules_acfp_rule_set

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `creation_path` | `String(pattern, len: 1..=256)` | Yes |  |
| `enable_regex_in_path` | Bool | No |  |
| `registration_page_path` | `String(pattern, len: 1..=256)` | Yes |  |
| `request_inspection` | [Struct(RequestInspectionACFP)](#statementmanaged_rule_group_statementmanaged_rule_group_configsaws_managed_rules_acfp_rule_setrequest_inspection) | Yes |  |
| `response_inspection` | [Struct(ResponseInspection)](#response_inspection) | No |  |

### statement.managed_rule_group_statement.managed_rule_group_configs.aws_managed_rules_acfp_rule_set.request_inspection

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `address_fields` | List\<[Struct(FieldIdentifier)](#field_identifier)\> | No |  |
| `email_field` | [Struct(FieldIdentifier)](#field_identifier) | No |  |
| `password_field` | [Struct(FieldIdentifier)](#field_identifier) | No |  |
| `payload_type` | [Enum (PayloadType)](#statementmanaged_rule_group_statementmanaged_rule_group_configsaws_managed_rules_acfp_rule_setrequest_inspectionpayload_type-payloadtype) | Yes |  |
| `phone_number_fields` | List\<[Struct(FieldIdentifier)](#field_identifier)\> | No |  |
| `username_field` | [Struct(FieldIdentifier)](#field_identifier) | No |  |

### statement.managed_rule_group_statement.managed_rule_group_configs.aws_managed_rules_anti_d_do_s_rule_set

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `client_side_action_config` | [Struct(ClientSideActionConfig)](#statementmanaged_rule_group_statementmanaged_rule_group_configsaws_managed_rules_anti_d_do_s_rule_setclient_side_action_config) | Yes |  |
| `sensitivity_to_block` | [Enum (SensitivityToAct)](#statementmanaged_rule_group_statementmanaged_rule_group_configsaws_managed_rules_anti_d_do_s_rule_setsensitivity_to_block-sensitivitytoact) | No |  |

### statement.managed_rule_group_statement.managed_rule_group_configs.aws_managed_rules_anti_d_do_s_rule_set.client_side_action_config

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `challenge` | [Struct(ClientSideAction)](#statementmanaged_rule_group_statementmanaged_rule_group_configsaws_managed_rules_anti_d_do_s_rule_setclient_side_action_configchallenge) | Yes |  |

### statement.managed_rule_group_statement.managed_rule_group_configs.aws_managed_rules_anti_d_do_s_rule_set.client_side_action_config.challenge

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `exempt_uri_regular_expressions` | List\<[Struct(Regex)](#statementmanaged_rule_group_statementmanaged_rule_group_configsaws_managed_rules_anti_d_do_s_rule_setclient_side_action_configchallengeexempt_uri_regular_expressions)\> | No |  |
| `sensitivity` | [Enum (SensitivityToAct)](#statementmanaged_rule_group_statementmanaged_rule_group_configsaws_managed_rules_anti_d_do_s_rule_setclient_side_action_configchallengesensitivity-sensitivitytoact) | No |  |
| `usage_of_action` | [Enum (UsageOfAction)](#statementmanaged_rule_group_statementmanaged_rule_group_configsaws_managed_rules_anti_d_do_s_rule_setclient_side_action_configchallengeusage_of_action-usageofaction) | Yes |  |

### statement.managed_rule_group_statement.managed_rule_group_configs.aws_managed_rules_anti_d_do_s_rule_set.client_side_action_config.challenge.exempt_uri_regular_expressions

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `regex_string` | String(len: 1..=512) | No |  |

### statement.managed_rule_group_statement.managed_rule_group_configs.aws_managed_rules_atp_rule_set

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `enable_regex_in_path` | Bool | No |  |
| `login_path` | String | Yes |  |
| `request_inspection` | [Struct(RequestInspection)](#statementmanaged_rule_group_statementmanaged_rule_group_configsaws_managed_rules_atp_rule_setrequest_inspection) | No |  |
| `response_inspection` | [Struct(ResponseInspection)](#response_inspection) | No |  |

### statement.managed_rule_group_statement.managed_rule_group_configs.aws_managed_rules_atp_rule_set.request_inspection

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `password_field` | [Struct(FieldIdentifier)](#field_identifier) | Yes |  |
| `payload_type` | [Enum (PayloadType)](#statementmanaged_rule_group_statementmanaged_rule_group_configsaws_managed_rules_atp_rule_setrequest_inspectionpayload_type-payloadtype) | Yes |  |
| `username_field` | [Struct(FieldIdentifier)](#field_identifier) | Yes |  |

### statement.managed_rule_group_statement.managed_rule_group_configs.aws_managed_rules_bot_control_rule_set

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `enable_machine_learning` | Bool | No |  |
| `inspection_level` | [Enum (InspectionLevel)](#statementmanaged_rule_group_statementmanaged_rule_group_configsaws_managed_rules_bot_control_rule_setinspection_level-inspectionlevel) | Yes |  |

### statement.not_statement

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `statement` | [Struct(Statement)](#statement) | Yes |  |

### statement.or_statement

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `statements` | List\<[Struct(Statement)](#statement)\> (items: 2..) | Yes |  |

### statement.rate_based_statement

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `aggregate_key_type` | [Enum (AggregateKeyType)](#statementrate_based_statementaggregate_key_type-aggregatekeytype) | Yes |  |
| `custom_keys` | List\<[Struct(RateBasedStatementCustomKey)](#statementrate_based_statementcustom_keys)\> (items: ..=5) | No | Specifies the aggregate keys to use in a rate-base rule. |
| `evaluation_window_sec` | IntEnum([60, 120, 300, 600]) | No |  |
| `forwarded_ip_config` | [Struct(ForwardedIPConfiguration)](#forwarded_ip_configuration) | No |  |
| `limit` | Int(10..=2000000000) | Yes |  |
| `scope_down_statement` | [Struct(Statement)](#statement) | No |  |

### statement.rate_based_statement.custom_keys

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `asn` | Empty struct (`{}`) | No |  |
| `cookie` | [Struct(RateLimitCookie)](#statementrate_based_statementcustom_keyscookie) | No |  |
| `forwarded_ip` | Empty struct (`{}`) | No |  |
| `http_method` | Empty struct (`{}`) | No |  |
| `header` | [Struct(RateLimitHeader)](#statementrate_based_statementcustom_keysheader) | No |  |
| `ip` | Empty struct (`{}`) | No |  |
| `ja3_fingerprint` | [Struct(RateLimitJA3Fingerprint)](#statementrate_based_statementcustom_keysja3_fingerprint) | No |  |
| `ja4_fingerprint` | [Struct(RateLimitJA4Fingerprint)](#statementrate_based_statementcustom_keysja4_fingerprint) | No |  |
| `label_namespace` | [Struct(RateLimitLabelNamespace)](#statementrate_based_statementcustom_keyslabel_namespace) | No |  |
| `query_argument` | [Struct(RateLimitQueryArgument)](#statementrate_based_statementcustom_keysquery_argument) | No |  |
| `query_string` | [Struct(RateLimitQueryString)](#statementrate_based_statementcustom_keysquery_string) | No |  |
| `uri_path` | [Struct(RateLimitUriPath)](#statementrate_based_statementcustom_keysuri_path) | No |  |

### statement.rate_based_statement.custom_keys.cookie

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `name` | String(pattern, len: 1..=64) | Yes | The name of the cookie to use. |
| `text_transformations` | List\<[Struct(TextTransformation)](#text_transformation)\> (items: 1..) | Yes |  |

### statement.rate_based_statement.custom_keys.header

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `name` | String(pattern, len: 1..=64) | Yes | The name of the header to use. |
| `text_transformations` | List\<[Struct(TextTransformation)](#text_transformation)\> (items: 1..) | Yes |  |

### statement.rate_based_statement.custom_keys.ja3_fingerprint

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `fallback_behavior` | [Enum (FallbackBehavior)](#statementrate_based_statementcustom_keysja3_fingerprintfallback_behavior-fallbackbehavior) | Yes |  |

### statement.rate_based_statement.custom_keys.ja4_fingerprint

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `fallback_behavior` | [Enum (FallbackBehavior)](#statementrate_based_statementcustom_keysja4_fingerprintfallback_behavior-fallbackbehavior) | Yes |  |

### statement.rate_based_statement.custom_keys.label_namespace

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `namespace` | `String(pattern, len: 1..=1024)` | Yes | The namespace to use for aggregation. |

### statement.rate_based_statement.custom_keys.query_argument

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `name` | String(pattern, len: 1..=64) | Yes | The name of the query argument to use. |
| `text_transformations` | List\<[Struct(TextTransformation)](#text_transformation)\> (items: 1..) | Yes |  |

### statement.rate_based_statement.custom_keys.query_string

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `text_transformations` | List\<[Struct(TextTransformation)](#text_transformation)\> (items: 1..) | Yes |  |

### statement.rate_based_statement.custom_keys.uri_path

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `text_transformations` | List\<[Struct(TextTransformation)](#text_transformation)\> (items: 1..) | Yes |  |

### statement.regex_match_statement

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `field_to_match` | [Struct(FieldToMatch)](#field_to_match) | Yes |  |
| `regex_string` | String(len: 1..=512) | Yes |  |
| `text_transformations` | List\<[Struct(TextTransformation)](#text_transformation)\> (items: 1..) | Yes |  |

### statement.regex_pattern_set_reference_statement

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `arn` | aws.Arn — identity: String(pattern); member: String(len: 20..=2048) | Yes |  |
| `field_to_match` | [Struct(FieldToMatch)](#field_to_match) | Yes |  |
| `text_transformations` | List\<[Struct(TextTransformation)](#text_transformation)\> (items: 1..) | Yes |  |

### statement.rule_group_reference_statement

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `arn` | aws.Arn — identity: String(pattern); member: String(len: 20..=2048) | Yes |  |
| `excluded_rules` | List\<[Struct(ExcludedRule)](#excluded_rule)\> (items: ..=100) | No |  |
| `rule_action_overrides` | List\<[Struct(RuleActionOverride)](#rule_action_override)\> (items: ..=100) | No | Action overrides for rules in the rule group. |

### statement.size_constraint_statement

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `comparison_operator` | [Enum (ComparisonOperator)](#statementsize_constraint_statementcomparison_operator-comparisonoperator) | Yes |  |
| `field_to_match` | [Struct(FieldToMatch)](#field_to_match) | Yes |  |
| `size` | Float(0..=21474836480) | Yes |  |
| `text_transformations` | List\<[Struct(TextTransformation)](#text_transformation)\> (items: 1..) | Yes |  |

### statement.sqli_match_statement

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `field_to_match` | [Struct(FieldToMatch)](#field_to_match) | Yes |  |
| `sensitivity_level` | [Enum (SensitivityLevel)](#statementsqli_match_statementsensitivity_level-sensitivitylevel) | No |  |
| `text_transformations` | List\<[Struct(TextTransformation)](#text_transformation)\> (items: 1..) | Yes |  |

### statement.xss_match_statement

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `field_to_match` | [Struct(FieldToMatch)](#field_to_match) | Yes |  |
| `text_transformations` | List\<[Struct(TextTransformation)](#text_transformation)\> (items: 1..) | Yes |  |

### text_transformation

Used by: `statement.byte_match_statement.text_transformations`, `statement.rate_based_statement.custom_keys.cookie.text_transformations`, `statement.rate_based_statement.custom_keys.header.text_transformations`, `statement.rate_based_statement.custom_keys.query_argument.text_transformations`, `statement.rate_based_statement.custom_keys.query_string.text_transformations`, `statement.rate_based_statement.custom_keys.uri_path.text_transformations`, `statement.regex_match_statement.text_transformations`, `statement.regex_pattern_set_reference_statement.text_transformations`, `statement.size_constraint_statement.text_transformations`, `statement.sqli_match_statement.text_transformations`, `statement.xss_match_statement.text_transformations`

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `priority` | Int(0..) | Yes |  |
| `type` | [Enum (TextTransformationType)](#text_transformationtype-texttransformationtype) | Yes |  |

### visibility_config

Used by: `rules.visibility_config`, `visibility_config`

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `cloud_watch_metrics_enabled` | Bool | Yes |  |
| `metric_name` | `String(pattern, len: 1..=128)` | Yes |  |
| `sampled_requests_enabled` | Bool | Yes |  |

## Attribute Reference

### `arn`

- **Type:** aws.wafv2.WebAcl.Arn — identity: String(pattern); member: String(len: 20..=2048)

### `capacity`

- **Type:** Int(0..)

### `id`

- **Type:** String(pattern)

### `label_namespace`

- **Type:** String(pattern)

## AWS API narrowings

These fields are narrower than or equal to CFN. Input rules combine documented AWS domains, recorded live observations, and Carina canonical spellings for stable state comparison. A canonical DSL spelling does not claim AWS rejects other spellings. Attaching a type name adds no structural constraints. Field names below use CFN spelling.

| CFN field | Relation to CFN | Evidence URL | Verbatim quote / exact observation | Checked / observed |
| --- | --- | --- | --- | --- |
| `/definitions/AWSManagedRulesACFPRuleSet/properties/CreationPath` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/waf/latest/APIReference/API_AWSManagedRulesACFPRuleSet.html#WAF-Type-AWSManagedRulesACFPRuleSet-CreationPath) | ` The path of the account creation endpoint for your application. This is the page on your website that accepts the completed registration form for a new user. This page must accept POST requests. `<br>` For example, for the URL https://example.com/web/newaccount, you would provide the path /web/newaccount. Account creation page paths that start with the path that you provide are considered a match. For example /web/newaccount matches the account creation paths /web/newaccount, /web/newaccount/, /web/newaccountPage, and /web/newaccount/thisPage, but doesn't match the path /home/web/newaccount or /website/newaccount. `<br>` Length Constraints: Minimum length of 1. Maximum length of 256. `<br>` Pattern: .*\S.* ` | 2026-11-07 |
| `/definitions/AWSManagedRulesACFPRuleSet/properties/RegistrationPagePath` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/waf/latest/APIReference/API_AWSManagedRulesACFPRuleSet.html#WAF-Type-AWSManagedRulesACFPRuleSet-RegistrationPagePath) | ` The path of the account registration endpoint for your application. This is the page on your website that presents the registration form to new users. `<br>` This page must accept GET text/html requests. `<br>` For example, for the URL https://example.com/web/registration, you would provide the path /web/registration. Registration page paths that start with the path that you provide are considered a match. For example /web/registration matches the registration paths /web/registration, /web/registration/, /web/registrationPage, and /web/registration/thisPage, but doesn't match the path /home/web/registration or /website/registration. `<br>` Length Constraints: Minimum length of 1. Maximum length of 256. `<br>` Pattern: .*\S.* ` | 2026-11-07 |
| `/definitions/AndStatement/properties/Statements` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/waf/latest/APIReference/API_AndStatement.html) | ` A logical rule statement used to combine other rule statements with AND logic. You provide more than one Statement within the AndStatement. ` | 2026-11-07 |
| `/definitions/AsnMatchStatement/properties/AsnList` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/waf/latest/APIReference/API_AsnMatchStatement.html#WAF-Type-AsnMatchStatement-AsnList) | ` Contains one or more Autonomous System Numbers (ASNs). ASNs are unique identifiers assigned to large internet networks managed by organizations such as internet service providers, enterprises, universities, or government agencies. `<br>` Array Members: Minimum number of 1 item. Maximum number of 100 items. `<br>` Valid Range: Minimum value of 0. Maximum value of 4294967295. ` | 2026-11-07 |
| `/definitions/ByteMatchStatement/properties/TextTransformations` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/waf/latest/APIReference/API_ByteMatchStatement.html#WAF-Type-ByteMatchStatement-TextTransformations) | ` Text transformations eliminate some of the unusual formatting that attackers use in web requests in an effort to bypass detection. Text transformations are used in rule match statements, to transform the FieldToMatch request component before inspecting it, and they're used in rate-based rule statements, to transform request components before using them as custom aggregation keys. If you specify one or more transformations to apply, AWS WAF performs all transformations on the specified content, starting from the lowest priority setting, and then uses the transformed component contents. `<br>` Array Members: Minimum number of 1 item. ` | 2026-11-07 |
| `/definitions/CookieMatchPattern/properties/All` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/waf/latest/APIReference/API_All.html) | ` JSON specification: "All": {} ` | 2026-11-07 |
| `/definitions/CustomHTTPHeader/properties/Name` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/waf/latest/APIReference/API_CustomHTTPHeader.html#WAF-Type-CustomHTTPHeader-Name) | ` The name of the custom header. `<br>` For custom request header insertion, when AWS WAF inserts the header into the request, it prefixes this name x-amzn-waf-, to avoid confusion with the headers that are already in the request. For example, for the header name sample, AWS WAF inserts the header x-amzn-waf-sample. `<br>` Length Constraints: Minimum length of 1. Maximum length of 64. `<br>` Pattern: ^[a-zA-Z0-9._$-]+$ ` | 2026-11-07 |
| `/definitions/CustomResponse/properties/CustomResponseBodyKey` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/waf/latest/APIReference/API_CustomResponse.html#WAF-Type-CustomResponse-CustomResponseBodyKey) | ` References the response body that you want AWS WAF to return to the web request client. You can define a custom response for a rule action or a default web ACL action that is set to block. To do this, you first define the response body key and value in the CustomResponseBodies setting for the WebACL or RuleGroup where you want to use it. Then, in the rule action or web ACL default action BlockAction setting, you reference the response body using this key. `<br>` Length Constraints: Minimum length of 1. Maximum length of 128. `<br>` Pattern: ^[\w\-]+$ ` | 2026-11-07 |
| `/definitions/DataProtectionConfig/properties/DataProtections` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/waf/latest/APIReference/API_DataProtectionConfig.html#WAF-Type-DataProtectionConfig-DataProtections) | ` An array of data protection configurations for specific web request field types. This is defined for each web ACL. AWS WAF applies the specified protection to all web requests that the web ACL inspects. `<br>` Array Members: Minimum number of 1 item. Maximum number of 26 items. ` | 2026-11-07 |
| `/definitions/ExcludedRule/properties/Name` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/waf/latest/APIReference/API_ExcludedRule.html#WAF-Type-ExcludedRule-Name) | ` The name of the rule whose action you want to override to Count. `<br>` Length Constraints: Minimum length of 1. Maximum length of 128. `<br>` Pattern: ^[\w\-]+$ ` | 2026-11-07 |
| `/definitions/FieldToMatch/properties/AllQueryArguments` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/waf/latest/APIReference/API_AllQueryArguments.html) | ` JSON specification: "AllQueryArguments": {} ` | 2026-11-07 |
| `/definitions/FieldToMatch/properties/Method` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/waf/latest/APIReference/API_Method.html) | ` JSON specification: "Method": {} ` | 2026-11-07 |
| `/definitions/FieldToMatch/properties/QueryString` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/waf/latest/APIReference/API_QueryString.html) | ` JSON specification: "QueryString": {} ` | 2026-11-07 |
| `/definitions/FieldToMatch/properties/UriPath` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/waf/latest/APIReference/API_UriPath.html) | ` JSON specification: "UriPath": {} ` | 2026-11-07 |
| `/definitions/FieldToProtect/properties/FieldKeys` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/waf/latest/APIReference/API_FieldToProtect.html#WAF-Type-FieldToProtect-FieldKeys) | ` Specifies the keys to protect for the specified field type. `<br>` Required for SINGLE_HEADER, SINGLE_COOKIE, and SINGLE_QUERY_ARGUMENT: provide a non-empty array naming the specific headers, cookies, or query arguments to protect. There is no option to protect all keys of these field types, so enumerate each key you intend to protect. `<br>` Must be omitted for QUERY_STRING and BODY: the entire component is protected and these field types take no keys. Supplying FieldKeys for them is rejected. `<br>` Array Members: Maximum number of 100 items. `<br>` Length Constraints: Minimum length of 1. Maximum length of 64. `<br>` Pattern: .*\S.* ` | 2026-11-07 |
| `/definitions/ForwardedIPConfiguration/properties/HeaderName` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/waf/latest/APIReference/API_ForwardedIPConfig.html#WAF-Type-ForwardedIPConfig-HeaderName) | ` The name of the HTTP header to use for the IP address. For example, to use the X-Forwarded-For (XFF) header, set this to X-Forwarded-For. `<br>` If the specified header isn't present in the request, AWS WAF doesn't apply the rule to the web request at all. `<br>` Length Constraints: Minimum length of 1. Maximum length of 255. `<br>` Pattern: ^[a-zA-Z0-9-]+$ ` | 2026-11-07 |
| `/definitions/GeoMatchStatement/properties/CountryCodes` | Subset: CFN ∩ reviewed input domain | [Source record](https://github.com/carina-rs/carina-provider-awscc/pull/597) | ` ap-northeast-1; Direct WAFv2 CreateWebACL (waf-cc-chunks.py; the script name does not indicate Cloud Control); Accepted; exact input: {"DefaultAction":{"Allow":{}},"Name":"carina-ccprobe-525-geo","Rules":[{"Action":{"Block":{}},"Name":"r1","Priority":0,"Statement":{"GeoMatchStatement":{"CountryCodes":["AF","AX","AL","DZ","AS","AD","AO","AI","AQ","AG","AR","AM","AW","AU","AT","AZ","BS","BH","BD","BB","BY","BE","BZ","BJ","BM","BT","BO","BQ","BA","BW","BV","BR","IO","BN","BG","BF","BI","KH","CM","CA","CV","KY","CF","TD","CL","CN","CX","CC","CO","KM"]}},"VisibilityConfig":{"CloudWatchMetricsEnabled":false,"MetricName":"probe","SampledRequestsEnabled":false}}],"Scope":"REGIONAL","VisibilityConfig":{"CloudWatchMetricsEnabled":false,"MetricName":"probe","SampledRequestsEnabled":false}}; result: "CreateWebACL succeeded; deleted rc=0" ` | 2026-11-06 |
| `/definitions/GeoMatchStatement/properties/CountryCodes` | Subset: CFN ∩ reviewed input domain | [Source record](https://github.com/carina-rs/carina-provider-awscc/pull/597) | ` ap-northeast-1; Direct WAFv2 CreateWebACL (waf-cc-chunks.py; the script name does not indicate Cloud Control); Accepted; exact input: {"DefaultAction":{"Allow":{}},"Name":"carina-ccprobe-525-geo","Rules":[{"Action":{"Block":{}},"Name":"r1","Priority":0,"Statement":{"GeoMatchStatement":{"CountryCodes":["CG","CD","CK","CR","CI","HR","CU","CW","CY","CZ","DK","DJ","DM","DO","EC","EG","SV","GQ","ER","EE","ET","FK","FO","FJ","FI","FR","GF","PF","TF","GA","GM","GE","DE","GH","GI","GR","GL","GD","GP","GU","GT","GG","GN","GW","GY","HT","HM","VA","HN","HK"]}},"VisibilityConfig":{"CloudWatchMetricsEnabled":false,"MetricName":"probe","SampledRequestsEnabled":false}}],"Scope":"REGIONAL","VisibilityConfig":{"CloudWatchMetricsEnabled":false,"MetricName":"probe","SampledRequestsEnabled":false}}; result: "CreateWebACL succeeded; deleted rc=0" ` | 2026-11-06 |
| `/definitions/GeoMatchStatement/properties/CountryCodes` | Subset: CFN ∩ reviewed input domain | [Source record](https://github.com/carina-rs/carina-provider-awscc/pull/597) | ` ap-northeast-1; Direct WAFv2 CreateWebACL (waf-cc-chunks.py; the script name does not indicate Cloud Control); Accepted; exact input: {"DefaultAction":{"Allow":{}},"Name":"carina-ccprobe-525-geo","Rules":[{"Action":{"Block":{}},"Name":"r1","Priority":0,"Statement":{"GeoMatchStatement":{"CountryCodes":["HU","IS","IN","ID","IR","IQ","IE","IM","IL","IT","JM","JP","JE","JO","KZ","KE","KI","KP","KR","KW","KG","LA","LV","LB","LS","LR","LY","LI","LT","LU","MO","MK","MG","MW","MY","MV","ML","MT","MH","MQ","MR","MU","YT","MX","FM","MD","MC","MN","ME","MS"]}},"VisibilityConfig":{"CloudWatchMetricsEnabled":false,"MetricName":"probe","SampledRequestsEnabled":false}}],"Scope":"REGIONAL","VisibilityConfig":{"CloudWatchMetricsEnabled":false,"MetricName":"probe","SampledRequestsEnabled":false}}; result: "CreateWebACL succeeded; deleted rc=0" ` | 2026-11-06 |
| `/definitions/GeoMatchStatement/properties/CountryCodes` | Subset: CFN ∩ reviewed input domain | [Source record](https://github.com/carina-rs/carina-provider-awscc/pull/597) | ` ap-northeast-1; Direct WAFv2 CreateWebACL (waf-cc-chunks.py; the script name does not indicate Cloud Control); Accepted; exact input: {"DefaultAction":{"Allow":{}},"Name":"carina-ccprobe-525-geo","Rules":[{"Action":{"Block":{}},"Name":"r1","Priority":0,"Statement":{"GeoMatchStatement":{"CountryCodes":["MA","MZ","MM","NA","NR","NP","NL","NC","NZ","NI","NE","NG","NU","NF","MP","NO","OM","PK","PW","PS","PA","PG","PY","PE","PH","PN","PL","PT","PR","QA","RE","RO","RU","RW","BL","SH","KN","LC","MF","PM","VC","WS","SM","ST","SA","SN","RS","SC","SL","SG"]}},"VisibilityConfig":{"CloudWatchMetricsEnabled":false,"MetricName":"probe","SampledRequestsEnabled":false}}],"Scope":"REGIONAL","VisibilityConfig":{"CloudWatchMetricsEnabled":false,"MetricName":"probe","SampledRequestsEnabled":false}}; result: "CreateWebACL succeeded; deleted rc=0" ` | 2026-11-06 |
| `/definitions/GeoMatchStatement/properties/CountryCodes` | Subset: CFN ∩ reviewed input domain | [Source record](https://github.com/carina-rs/carina-provider-awscc/pull/597) | ` ap-northeast-1; Direct WAFv2 CreateWebACL (waf-cc-chunks.py; the script name does not indicate Cloud Control); Accepted; exact input: {"DefaultAction":{"Allow":{}},"Name":"carina-ccprobe-525-geo","Rules":[{"Action":{"Block":{}},"Name":"r1","Priority":0,"Statement":{"GeoMatchStatement":{"CountryCodes":["SX","SK","SI","SB","SO","ZA","GS","SS","ES","LK","SD","SR","SJ","SZ","SE","CH","SY","TW","TJ","TZ","TH","TL","TG","TK","TO","TT","TN","TR","TM","TC","TV","UG","UA","AE","GB","US","UM","UY","UZ","VU","VE","VN","VG","VI","WF","EH","YE","ZM","ZW","XK"]}},"VisibilityConfig":{"CloudWatchMetricsEnabled":false,"MetricName":"probe","SampledRequestsEnabled":false}}],"Scope":"REGIONAL","VisibilityConfig":{"CloudWatchMetricsEnabled":false,"MetricName":"probe","SampledRequestsEnabled":false}}; result: "CreateWebACL succeeded; deleted rc=0" ` | 2026-11-06 |
| `/definitions/GeoMatchStatement/properties/CountryCodes` | Subset: CFN ∩ reviewed input domain | [Source record](https://github.com/carina-rs/carina-provider-awscc/pull/597) | ` ap-northeast-1; Direct WAFv2 CreateWebACL (waf-cc-chunks.py; the script name does not indicate Cloud Control); Rejected; exact input: {"DefaultAction":{"Allow":{}},"Name":"carina-ccprobe-525-geo","Rules":[{"Action":{"Block":{}},"Name":"r1","Priority":0,"Statement":{"GeoMatchStatement":{"CountryCodes":["AF","AX","AL","DZ","AS","AD","AO","AI","AQ","AG","AR","AM","AW","AU","AT","AZ","BS","BH","BD","BB","BY","BE","BZ","BJ","BM","BT","BO","BQ","BA","BW","BV","BR","IO","BN","BG","BF","BI","KH","CM","CA","CV","KY","CF","TD","CL","CN","CX","CC","CO","KM","CG"]}},"VisibilityConfig":{"CloudWatchMetricsEnabled":false,"MetricName":"probe","SampledRequestsEnabled":false}}],"Scope":"REGIONAL","VisibilityConfig":{"CloudWatchMetricsEnabled":false,"MetricName":"probe","SampledRequestsEnabled":false}}; result: "Member must have length less than or equal to 50" ` | 2026-11-06 |
| `/definitions/HeaderMatchPattern/properties/All` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/waf/latest/APIReference/API_All.html) | ` JSON specification: "All": {} ` | 2026-11-07 |
| `/definitions/IPSetForwardedIPConfiguration/properties/HeaderName` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/waf/latest/APIReference/API_IPSetForwardedIPConfig.html#WAF-Type-IPSetForwardedIPConfig-HeaderName) | ` The name of the HTTP header to use for the IP address. For example, to use the X-Forwarded-For (XFF) header, set this to X-Forwarded-For. `<br>` If the specified header isn't present in the request, AWS WAF doesn't apply the rule to the web request at all. `<br>` Length Constraints: Minimum length of 1. Maximum length of 255. `<br>` Pattern: ^[a-zA-Z0-9-]+$ ` | 2026-11-07 |
| `/definitions/JsonMatchPattern/properties/All` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/waf/latest/APIReference/API_All.html) | ` JSON specification: "All": {} ` | 2026-11-07 |
| `/definitions/JsonMatchPattern/properties/IncludedPaths` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/waf/latest/APIReference/API_JsonMatchPattern.html#WAF-Type-JsonMatchPattern-IncludedPaths) | ` Match only the specified include paths. See also MatchScope in JsonBody. `<br>` Provide the include paths using JSON Pointer syntax. For example, "IncludedPaths": ["/dogs/0/name", "/dogs/1/name"]. For information about this syntax, see the Internet Engineering Task Force (IETF) documentation JavaScript Object Notation (JSON) Pointer. `<br>` You must specify either this setting or the All setting, but not both. `<br>` Don't use this option to include all paths. Instead, use the All setting. `<br>` Array Members: Minimum number of 1 item. `<br>` Length Constraints: Minimum length of 1. Maximum length of 512. `<br>` Pattern: ([/])\|([/](([^~])\|(~[01]))+) ` | 2026-11-07 |
| `/definitions/Label/properties/Name` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/waf/latest/APIReference/API_Label.html#WAF-Type-Label-Name) | ` The label string. `<br>` Length Constraints: Minimum length of 1. Maximum length of 1024. `<br>` Pattern: ^[0-9A-Za-z_\-:]+$ ` | 2026-11-07 |
| `/definitions/LabelMatchStatement/properties/Key` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/waf/latest/APIReference/API_LabelMatchStatement.html#WAF-Type-LabelMatchStatement-Key) | ` The string to match against. The setting you provide for this depends on the match statement's Scope setting: `<br>` If the Scope indicates LABEL, then this specification must include the name and can include any number of preceding namespace specifications and prefix up to providing the fully qualified label name. `<br>` If the Scope indicates NAMESPACE, then this specification can include any number of contiguous namespace strings, and can include the entire label namespace prefix from the rule group or web ACL where the label originates. `<br>` Labels are case sensitive and components of a label must be separated by colon, for example NS1:NS2:name. `<br>` Length Constraints: Minimum length of 1. Maximum length of 1024. `<br>` Pattern: ^[0-9A-Za-z_\-:]+$ ` | 2026-11-07 |
| `/definitions/ManagedRuleGroupStatement/properties/ExcludedRules` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/waf/latest/APIReference/API_ManagedRuleGroupStatement.html#WAF-Type-ManagedRuleGroupStatement-ExcludedRules) | ` Rules in the referenced rule group whose actions are set to Count. `<br>` Instead of this option, use RuleActionOverrides. It accepts any valid action setting, including Count. `<br>` Array Members: Maximum number of 100 items. ` | 2026-11-07 |
| `/definitions/OrStatement/properties/Statements` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/waf/latest/APIReference/API_OrStatement.html) | ` A logical rule statement used to combine other rule statements with OR logic. You provide more than one Statement within the OrStatement. ` | 2026-11-07 |
| `/definitions/OverrideAction/properties/Count` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/waf/latest/APIReference/API_CountAction.html) | ` Specifies that AWS WAF should count the request. Optionally defines additional custom handling for the request. ` | 2026-11-07 |
| `/definitions/OverrideAction/properties/None` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/waf/latest/APIReference/API_NoneAction.html) | ` JSON specification: "None": {} ` | 2026-11-07 |
| `/definitions/RateBasedStatementCustomKey/properties/ASN` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/waf/latest/APIReference/API_CreateWebACL.html) | ` "ASN": { `<br>`                          } ` | 2026-11-07 |
| `/definitions/RateBasedStatementCustomKey/properties/ForwardedIP` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/waf/latest/APIReference/API_CreateWebACL.html) | ` "ForwardedIP": { `<br>`                          } ` | 2026-11-07 |
| `/definitions/RateBasedStatementCustomKey/properties/HTTPMethod` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/waf/latest/APIReference/API_CreateWebACL.html) | ` "HTTPMethod": { `<br>`                          } ` | 2026-11-07 |
| `/definitions/RateBasedStatementCustomKey/properties/IP` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/waf/latest/APIReference/API_CreateWebACL.html) | ` "IP": { `<br>`                          } ` | 2026-11-07 |
| `/definitions/RateLimitCookie/properties/TextTransformations` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/waf/latest/APIReference/API_RateLimitCookie.html#WAF-Type-RateLimitCookie-TextTransformations) | ` Text transformations eliminate some of the unusual formatting that attackers use in web requests in an effort to bypass detection. Text transformations are used in rule match statements, to transform the FieldToMatch request component before inspecting it, and they're used in rate-based rule statements, to transform request components before using them as custom aggregation keys. If you specify one or more transformations to apply, AWS WAF performs all transformations on the specified content, starting from the lowest priority setting, and then uses the transformed component contents. `<br>` Array Members: Minimum number of 1 item. ` | 2026-11-07 |
| `/definitions/RateLimitHeader/properties/TextTransformations` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/waf/latest/APIReference/API_RateLimitHeader.html#WAF-Type-RateLimitHeader-TextTransformations) | ` Text transformations eliminate some of the unusual formatting that attackers use in web requests in an effort to bypass detection. Text transformations are used in rule match statements, to transform the FieldToMatch request component before inspecting it, and they're used in rate-based rule statements, to transform request components before using them as custom aggregation keys. If you specify one or more transformations to apply, AWS WAF performs all transformations on the specified content, starting from the lowest priority setting, and then uses the transformed component contents. `<br>` Array Members: Minimum number of 1 item. ` | 2026-11-07 |
| `/definitions/RateLimitLabelNamespace/properties/Namespace` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/waf/latest/APIReference/API_RateLimitLabelNamespace.html#WAF-Type-RateLimitLabelNamespace-Namespace) | ` The namespace to use for aggregation. `<br>` Length Constraints: Minimum length of 1. Maximum length of 1024. `<br>` Pattern: ^[0-9A-Za-z_\-:]+:$ ` | 2026-11-07 |
| `/definitions/RateLimitQueryArgument/properties/TextTransformations` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/waf/latest/APIReference/API_RateLimitQueryArgument.html#WAF-Type-RateLimitQueryArgument-TextTransformations) | ` Text transformations eliminate some of the unusual formatting that attackers use in web requests in an effort to bypass detection. Text transformations are used in rule match statements, to transform the FieldToMatch request component before inspecting it, and they're used in rate-based rule statements, to transform request components before using them as custom aggregation keys. If you specify one or more transformations to apply, AWS WAF performs all transformations on the specified content, starting from the lowest priority setting, and then uses the transformed component contents. `<br>` Array Members: Minimum number of 1 item. ` | 2026-11-07 |
| `/definitions/RateLimitQueryString/properties/TextTransformations` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/waf/latest/APIReference/API_RateLimitQueryString.html#WAF-Type-RateLimitQueryString-TextTransformations) | ` Text transformations eliminate some of the unusual formatting that attackers use in web requests in an effort to bypass detection. Text transformations are used in rule match statements, to transform the FieldToMatch request component before inspecting it, and they're used in rate-based rule statements, to transform request components before using them as custom aggregation keys. If you specify one or more transformations to apply, AWS WAF performs all transformations on the specified content, starting from the lowest priority setting, and then uses the transformed component contents. `<br>` Array Members: Minimum number of 1 item. ` | 2026-11-07 |
| `/definitions/RateLimitUriPath/properties/TextTransformations` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/waf/latest/APIReference/API_RateLimitUriPath.html#WAF-Type-RateLimitUriPath-TextTransformations) | ` Text transformations eliminate some of the unusual formatting that attackers use in web requests in an effort to bypass detection. Text transformations are used in rule match statements, to transform the FieldToMatch request component before inspecting it, and they're used in rate-based rule statements, to transform request components before using them as custom aggregation keys. If you specify one or more transformations to apply, AWS WAF performs all transformations on the specified content, starting from the lowest priority setting, and then uses the transformed component contents. `<br>` Array Members: Minimum number of 1 item. ` | 2026-11-07 |
| `/definitions/RegexMatchStatement/properties/TextTransformations` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/waf/latest/APIReference/API_RegexMatchStatement.html#WAF-Type-RegexMatchStatement-TextTransformations) | ` Text transformations eliminate some of the unusual formatting that attackers use in web requests in an effort to bypass detection. Text transformations are used in rule match statements, to transform the FieldToMatch request component before inspecting it, and they're used in rate-based rule statements, to transform request components before using them as custom aggregation keys. If you specify one or more transformations to apply, AWS WAF performs all transformations on the specified content, starting from the lowest priority setting, and then uses the transformed component contents. `<br>` Array Members: Minimum number of 1 item. ` | 2026-11-07 |
| `/definitions/RegexPatternSetReferenceStatement/properties/TextTransformations` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/waf/latest/APIReference/API_RegexPatternSetReferenceStatement.html#WAF-Type-RegexPatternSetReferenceStatement-TextTransformations) | ` Text transformations eliminate some of the unusual formatting that attackers use in web requests in an effort to bypass detection. Text transformations are used in rule match statements, to transform the FieldToMatch request component before inspecting it, and they're used in rate-based rule statements, to transform request components before using them as custom aggregation keys. If you specify one or more transformations to apply, AWS WAF performs all transformations on the specified content, starting from the lowest priority setting, and then uses the transformed component contents. `<br>` Array Members: Minimum number of 1 item. ` | 2026-11-07 |
| `/definitions/ResponseInspectionStatusCode/properties/FailureCodes` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/waf/latest/APIReference/API_ResponseInspectionStatusCode.html#WAF-Type-ResponseInspectionStatusCode-FailureCodes) | ` Status codes in the response that indicate a failed login or account creation attempt. To be counted as a failure, the response status code must match one of these. Each code must be unique among the success and failure status codes. `<br>` JSON example: "FailureCodes": [ 400, 404 ] `<br>` Array Members: Minimum number of 1 item. Maximum number of 10 items. `<br>` Valid Range: Minimum value of 0. Maximum value of 999. ` | 2026-11-07 |
| `/definitions/ResponseInspectionStatusCode/properties/SuccessCodes` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/waf/latest/APIReference/API_ResponseInspectionStatusCode.html#WAF-Type-ResponseInspectionStatusCode-SuccessCodes) | ` Status codes in the response that indicate a successful login or account creation attempt. To be counted as a success, the response status code must match one of these. Each code must be unique among the success and failure status codes. `<br>` JSON example: "SuccessCodes": [ 200, 201 ] `<br>` Array Members: Minimum number of 1 item. Maximum number of 10 items. `<br>` Valid Range: Minimum value of 0. Maximum value of 999. ` | 2026-11-07 |
| `/definitions/Rule/properties/Name` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/waf/latest/APIReference/API_Rule.html#WAF-Type-Rule-Name) | ` The name of the rule. `<br>` If you change the name of a Rule after you create it and you want the rule's metric name to reflect the change, update the metric name in the rule's VisibilityConfig settings. AWS WAF doesn't automatically update the metric name when you update the rule name. `<br>` Length Constraints: Minimum length of 1. Maximum length of 128. `<br>` Pattern: ^[\w\-]+$ ` | 2026-11-07 |
| `/definitions/RuleActionOverride/properties/Name` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/waf/latest/APIReference/API_RuleActionOverride.html#WAF-Type-RuleActionOverride-Name) | ` The name of the rule to override. `<br>` Verify the rule names in your overrides carefully. With managed rule groups, AWS WAF silently ignores any override that uses an invalid rule name. With customer-owned rule groups, invalid rule names in your overrides will cause web ACL updates to fail. An invalid rule name is any name that doesn't exactly match the case-sensitive name of an existing rule in the rule group. `<br>` Length Constraints: Minimum length of 1. Maximum length of 128. `<br>` Pattern: ^[\w\-]+$ ` | 2026-11-07 |
| `/definitions/RuleGroupReferenceStatement/properties/ExcludedRules` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/waf/latest/APIReference/API_RuleGroupReferenceStatement.html#WAF-Type-RuleGroupReferenceStatement-ExcludedRules) | ` Rules in the referenced rule group whose actions are set to Count. `<br>` Instead of this option, use RuleActionOverrides. It accepts any valid action setting, including Count. `<br>` Array Members: Maximum number of 100 items. ` | 2026-11-07 |
| `/definitions/SingleHeader/properties/Name` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/waf/latest/APIReference/API_SingleHeader.html#WAF-Type-SingleHeader-Name) | ` The name of the query header to inspect. `<br>` Length Constraints: Minimum length of 1. Maximum length of 64. `<br>` Pattern: .*\S.* ` | 2026-11-07 |
| `/definitions/SingleQueryArgument/properties/Name` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/waf/latest/APIReference/API_SingleQueryArgument.html#WAF-Type-SingleQueryArgument-Name) | ` The name of the query argument to inspect. `<br>` Length Constraints: Minimum length of 1. Maximum length of 64. `<br>` Pattern: .*\S.* ` | 2026-11-07 |
| `/definitions/SizeConstraintStatement/properties/TextTransformations` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/waf/latest/APIReference/API_SizeConstraintStatement.html#WAF-Type-SizeConstraintStatement-TextTransformations) | ` Text transformations eliminate some of the unusual formatting that attackers use in web requests in an effort to bypass detection. Text transformations are used in rule match statements, to transform the FieldToMatch request component before inspecting it, and they're used in rate-based rule statements, to transform request components before using them as custom aggregation keys. If you specify one or more transformations to apply, AWS WAF performs all transformations on the specified content, starting from the lowest priority setting, and then uses the transformed component contents. `<br>` Array Members: Minimum number of 1 item. ` | 2026-11-07 |
| `/definitions/SqliMatchStatement/properties/TextTransformations` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/waf/latest/APIReference/API_SqliMatchStatement.html#WAF-Type-SqliMatchStatement-TextTransformations) | ` Text transformations eliminate some of the unusual formatting that attackers use in web requests in an effort to bypass detection. Text transformations are used in rule match statements, to transform the FieldToMatch request component before inspecting it, and they're used in rate-based rule statements, to transform request components before using them as custom aggregation keys. If you specify one or more transformations to apply, AWS WAF performs all transformations on the specified content, starting from the lowest priority setting, and then uses the transformed component contents. `<br>` Array Members: Minimum number of 1 item. ` | 2026-11-07 |
| `/definitions/Tag/properties/Key` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/waf/latest/APIReference/API_Tag.html#WAF-Type-Tag-Key) | ` Part of the key:value pair that defines a tag. You can use a tag key to describe a category of information, such as "customer." Tag keys are case-sensitive. `<br>` Length Constraints: Minimum length of 1. Maximum length of 128. `<br>` Pattern: ^([\p{L}\p{Z}\p{N}_.:/=+\-@]*)$ ` | 2026-11-07 |
| `/definitions/Tag/properties/Value` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/waf/latest/APIReference/API_Tag.html#WAF-Type-Tag-Value) | ` Part of the key:value pair that defines a tag. You can use a tag value to describe a specific value within a category, such as "companyA" or "companyB." Tag values are case-sensitive. `<br>` Length Constraints: Minimum length of 0. Maximum length of 256. `<br>` Pattern: ^([\p{L}\p{Z}\p{N}_.:/=+\-@]*)$ ` | 2026-11-07 |
| `/definitions/VisibilityConfig/properties/MetricName` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/waf/latest/APIReference/API_VisibilityConfig.html#WAF-Type-VisibilityConfig-MetricName) | ` A name of the Amazon CloudWatch metric dimension. The name can contain only the characters: A-Z, a-z, 0-9, - (hyphen), and _ (underscore). The name can be from one to 128 characters long. It can't contain whitespace or metric names that are reserved for AWS WAF, for example All and Default_Action. `<br>` Length Constraints: Minimum length of 1. Maximum length of 255. `<br>` Pattern: ^[\w#:\.\-/]+$ ` | 2026-11-07 |
| `/definitions/XssMatchStatement/properties/TextTransformations` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/waf/latest/APIReference/API_XssMatchStatement.html#WAF-Type-XssMatchStatement-TextTransformations) | ` Text transformations eliminate some of the unusual formatting that attackers use in web requests in an effort to bypass detection. Text transformations are used in rule match statements, to transform the FieldToMatch request component before inspecting it, and they're used in rate-based rule statements, to transform request components before using them as custom aggregation keys. If you specify one or more transformations to apply, AWS WAF performs all transformations on the specified content, starting from the lowest priority setting, and then uses the transformed component contents. `<br>` Array Members: Minimum number of 1 item. ` | 2026-11-07 |
| `/properties/CustomResponseBodies` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/waf/latest/APIReference/API_WebACL.html#WAF-Type-WebACL-CustomResponseBodies) | ` A map of custom response keys and content bodies. When you create a rule with a block action, you can send a custom response to the web request. You define these for the web ACL, and then use them in the rules and default actions that you define in the web ACL. `<br>` For information about customizing web requests and responses, see Customizing web requests and responses in AWS WAF in the AWS WAF Developer Guide. `<br>` For information about the limits on count and size for custom request and response settings, see AWS WAF quotas in the AWS WAF Developer Guide. `<br>` Map Entries: Maximum number of items. `<br>` Key Length Constraints: Minimum length of 1. Maximum length of 128. `<br>` Key Pattern: ^[\w\-]+$ ` | 2026-11-07 |
| `/properties/Description` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/waf/latest/APIReference/API_CreateWebACL.html#WAF-CreateWebACL-request-Description) | ` A description of the web ACL that helps with identification. `<br>` Length Constraints: Minimum length of 1. Maximum length of 256. `<br>` Pattern: ^[\w+=:#@/\-,\.][\w+=:#@/\-,\.\s]+[\w+=:#@/\-,\.]$ ` | 2026-11-07 |
| `/properties/Name` | Subset: CFN ∩ reviewed input domain | [Source record](https://docs.aws.amazon.com/waf/latest/APIReference/API_CreateWebACL.html#WAF-CreateWebACL-request-Name) | ` The name of the web ACL. You cannot change the name of a web ACL after you create it. `<br>` Length Constraints: Minimum length of 1. Maximum length of 128. `<br>` Pattern: ^[\w\-]+$ ` | 2026-11-07 |

