---
title: "awscc.wafv2.WebAcl"
description: "AWSCC WAFv2 WebAcl resource reference"
---


CloudFormation Type: `AWS::WAFv2::WebACL`

Contains the Rules that identify the requests that you want to allow, block, or count. In a WebACL, you also specify a default action (ALLOW or BLOCK), and the action for each Rule that you add to a WebACL, for example, block requests from specified IP addresses or block requests from specified referrers. You also associate the WebACL with a CloudFront distribution to identify the requests that you want AWS WAF to filter. If you add more than one Rule to a WebACL, a request needs to match only one of the specifications to be allowed, blocked, or counted.

## Argument Reference

### `application_config`

- **Type:** [Struct(ApplicationConfig)](#applicationconfig)
- **Required:** No

Collection of application attributes.

### `association_config`

- **Type:** [Struct(AssociationConfig)](#associationconfig)
- **Required:** No

### `captcha_config`

- **Type:** [Struct(CaptchaConfig)](#captchaconfig)
- **Required:** No

### `challenge_config`

- **Type:** [Struct(ChallengeConfig)](#challengeconfig)
- **Required:** No

### `custom_response_bodies`

- **Type:** String
- **Required:** No

### `data_protection_config`

- **Type:** [Struct(DataProtectionConfig)](#dataprotectionconfig)
- **Required:** No

Collection of dataProtects.

### `default_action`

- **Type:** [Struct(DefaultAction)](#defaultaction)
- **Required:** Yes

### `description`

- **Type:** String
- **Required:** No

### `name`

- **Type:** String
- **Required:** No
- **Create-only:** Yes

### `on_source_d_do_s_protection_config`

- **Type:** [Struct(OnSourceDDoSProtectionConfig)](#onsourceddosprotectionconfig)
- **Required:** No

### `rules`

- **Type:** [List\<Rule\>](#rule)
- **Required:** No

Collection of Rules.

### `scope`

- **Type:** [Enum (Scope)](#scope-scope)
- **Required:** Yes
- **Create-only:** Yes

### `tags`

- **Type:** `Map<String, String>`
- **Required:** No

### `token_domains`

- **Type:** String
- **Required:** No

### `visibility_config`

- **Type:** [Struct(VisibilityConfig)](#visibilityconfig)
- **Required:** Yes

## Enum Values

### sensitivity_to_block (SensitivityToAct)

| Value | DSL Identifier |
|-------|----------------|
| `LOW` | `aws.wafv2.WebAcl.Rule.Statement.ManagedRuleGroupStatement.ManagedRuleGroupConfig.AwsManagedRulesAntiDDoSRuleSet.SensitivityToAct.low` |
| `MEDIUM` | `aws.wafv2.WebAcl.Rule.Statement.ManagedRuleGroupStatement.ManagedRuleGroupConfig.AwsManagedRulesAntiDDoSRuleSet.SensitivityToAct.medium` |
| `HIGH` | `aws.wafv2.WebAcl.Rule.Statement.ManagedRuleGroupStatement.ManagedRuleGroupConfig.AwsManagedRulesAntiDDoSRuleSet.SensitivityToAct.high` |

Shorthand formats: `low` or `SensitivityToAct.low`

### inspection_level (InspectionLevel)

| Value | DSL Identifier |
|-------|----------------|
| `COMMON` | `aws.wafv2.WebAcl.Rule.Statement.ManagedRuleGroupStatement.ManagedRuleGroupConfig.AwsManagedRulesBotControlRuleSet.InspectionLevel.common` |
| `TARGETED` | `aws.wafv2.WebAcl.Rule.Statement.ManagedRuleGroupStatement.ManagedRuleGroupConfig.AwsManagedRulesBotControlRuleSet.InspectionLevel.targeted` |

Shorthand formats: `common` or `InspectionLevel.common`

### oversize_handling (OversizeHandling)

| Value | DSL Identifier |
|-------|----------------|
| `CONTINUE` | `aws.wafv2.WebAcl.Rule.Statement.ByteMatchStatement.FieldToMatch.Body.OversizeHandling.continue` |
| `MATCH` | `aws.wafv2.WebAcl.Rule.Statement.ByteMatchStatement.FieldToMatch.Body.OversizeHandling.match` |
| `NO_MATCH` | `aws.wafv2.WebAcl.Rule.Statement.ByteMatchStatement.FieldToMatch.Body.OversizeHandling.no_match` |

Shorthand formats: `continue` or `OversizeHandling.continue`

### positional_constraint (PositionalConstraint)

| Value | DSL Identifier |
|-------|----------------|
| `EXACTLY` | `aws.wafv2.WebAcl.Rule.Statement.ByteMatchStatement.PositionalConstraint.exactly` |
| `STARTS_WITH` | `aws.wafv2.WebAcl.Rule.Statement.ByteMatchStatement.PositionalConstraint.starts_with` |
| `ENDS_WITH` | `aws.wafv2.WebAcl.Rule.Statement.ByteMatchStatement.PositionalConstraint.ends_with` |
| `CONTAINS` | `aws.wafv2.WebAcl.Rule.Statement.ByteMatchStatement.PositionalConstraint.contains` |
| `CONTAINS_WORD` | `aws.wafv2.WebAcl.Rule.Statement.ByteMatchStatement.PositionalConstraint.contains_word` |

Shorthand formats: `exactly` or `PositionalConstraint.exactly`

### sensitivity (SensitivityToAct)

| Value | DSL Identifier |
|-------|----------------|
| `LOW` | `aws.wafv2.WebAcl.Rule.Statement.ManagedRuleGroupStatement.ManagedRuleGroupConfig.AwsManagedRulesAntiDDoSRuleSet.ClientSideActionConfig.ClientSideAction.SensitivityToAct.low` |
| `MEDIUM` | `aws.wafv2.WebAcl.Rule.Statement.ManagedRuleGroupStatement.ManagedRuleGroupConfig.AwsManagedRulesAntiDDoSRuleSet.ClientSideActionConfig.ClientSideAction.SensitivityToAct.medium` |
| `HIGH` | `aws.wafv2.WebAcl.Rule.Statement.ManagedRuleGroupStatement.ManagedRuleGroupConfig.AwsManagedRulesAntiDDoSRuleSet.ClientSideActionConfig.ClientSideAction.SensitivityToAct.high` |

Shorthand formats: `low` or `SensitivityToAct.low`

### usage_of_action (UsageOfAction)

| Value | DSL Identifier |
|-------|----------------|
| `ENABLED` | `aws.wafv2.WebAcl.Rule.Statement.ManagedRuleGroupStatement.ManagedRuleGroupConfig.AwsManagedRulesAntiDDoSRuleSet.ClientSideActionConfig.ClientSideAction.UsageOfAction.enabled` |
| `DISABLED` | `aws.wafv2.WebAcl.Rule.Statement.ManagedRuleGroupStatement.ManagedRuleGroupConfig.AwsManagedRulesAntiDDoSRuleSet.ClientSideActionConfig.ClientSideAction.UsageOfAction.disabled` |

Shorthand formats: `enabled` or `UsageOfAction.enabled`

### match_scope (MapMatchScope)

| Value | DSL Identifier |
|-------|----------------|
| `ALL` | `aws.wafv2.WebAcl.Rule.Statement.ByteMatchStatement.FieldToMatch.Cookies.MapMatchScope.all` |
| `KEY` | `aws.wafv2.WebAcl.Rule.Statement.ByteMatchStatement.FieldToMatch.Cookies.MapMatchScope.key` |
| `VALUE` | `aws.wafv2.WebAcl.Rule.Statement.ByteMatchStatement.FieldToMatch.Cookies.MapMatchScope.value` |

Shorthand formats: `all` or `MapMatchScope.all`

### oversize_handling (OversizeHandling)

| Value | DSL Identifier |
|-------|----------------|
| `CONTINUE` | `aws.wafv2.WebAcl.Rule.Statement.ByteMatchStatement.FieldToMatch.Cookies.OversizeHandling.continue` |
| `MATCH` | `aws.wafv2.WebAcl.Rule.Statement.ByteMatchStatement.FieldToMatch.Cookies.OversizeHandling.match` |
| `NO_MATCH` | `aws.wafv2.WebAcl.Rule.Statement.ByteMatchStatement.FieldToMatch.Cookies.OversizeHandling.no_match` |

Shorthand formats: `continue` or `OversizeHandling.continue`

### fallback_behavior (FallbackBehavior)

| Value | DSL Identifier |
|-------|----------------|
| `MATCH` | `aws.wafv2.WebAcl.Rule.Statement.AsnMatchStatement.ForwardedIpConfiguration.FallbackBehavior.match` |
| `NO_MATCH` | `aws.wafv2.WebAcl.Rule.Statement.AsnMatchStatement.ForwardedIpConfiguration.FallbackBehavior.no_match` |

Shorthand formats: `match` or `FallbackBehavior.match`

### oversize_handling (OversizeHandling)

| Value | DSL Identifier |
|-------|----------------|
| `CONTINUE` | `aws.wafv2.WebAcl.Rule.Statement.ByteMatchStatement.FieldToMatch.HeaderOrder.OversizeHandling.continue` |
| `MATCH` | `aws.wafv2.WebAcl.Rule.Statement.ByteMatchStatement.FieldToMatch.HeaderOrder.OversizeHandling.match` |
| `NO_MATCH` | `aws.wafv2.WebAcl.Rule.Statement.ByteMatchStatement.FieldToMatch.HeaderOrder.OversizeHandling.no_match` |

Shorthand formats: `continue` or `OversizeHandling.continue`

### match_scope (MapMatchScope)

| Value | DSL Identifier |
|-------|----------------|
| `ALL` | `aws.wafv2.WebAcl.Rule.Statement.ByteMatchStatement.FieldToMatch.Headers.MapMatchScope.all` |
| `KEY` | `aws.wafv2.WebAcl.Rule.Statement.ByteMatchStatement.FieldToMatch.Headers.MapMatchScope.key` |
| `VALUE` | `aws.wafv2.WebAcl.Rule.Statement.ByteMatchStatement.FieldToMatch.Headers.MapMatchScope.value` |

Shorthand formats: `all` or `MapMatchScope.all`

### oversize_handling (OversizeHandling)

| Value | DSL Identifier |
|-------|----------------|
| `CONTINUE` | `aws.wafv2.WebAcl.Rule.Statement.ByteMatchStatement.FieldToMatch.Headers.OversizeHandling.continue` |
| `MATCH` | `aws.wafv2.WebAcl.Rule.Statement.ByteMatchStatement.FieldToMatch.Headers.OversizeHandling.match` |
| `NO_MATCH` | `aws.wafv2.WebAcl.Rule.Statement.ByteMatchStatement.FieldToMatch.Headers.OversizeHandling.no_match` |

Shorthand formats: `continue` or `OversizeHandling.continue`

### fallback_behavior (FallbackBehavior)

| Value | DSL Identifier |
|-------|----------------|
| `MATCH` | `aws.wafv2.WebAcl.Rule.Statement.IpSetReferenceStatement.IpSetForwardedIpConfiguration.FallbackBehavior.match` |
| `NO_MATCH` | `aws.wafv2.WebAcl.Rule.Statement.IpSetReferenceStatement.IpSetForwardedIpConfiguration.FallbackBehavior.no_match` |

Shorthand formats: `match` or `FallbackBehavior.match`

### position (Position)

| Value | DSL Identifier |
|-------|----------------|
| `FIRST` | `aws.wafv2.WebAcl.Rule.Statement.IpSetReferenceStatement.IpSetForwardedIpConfiguration.Position.first` |
| `LAST` | `aws.wafv2.WebAcl.Rule.Statement.IpSetReferenceStatement.IpSetForwardedIpConfiguration.Position.last` |
| `ANY` | `aws.wafv2.WebAcl.Rule.Statement.IpSetReferenceStatement.IpSetForwardedIpConfiguration.Position.any` |

Shorthand formats: `first` or `Position.first`

### fallback_behavior (FallbackBehavior)

| Value | DSL Identifier |
|-------|----------------|
| `MATCH` | `aws.wafv2.WebAcl.Rule.Statement.ByteMatchStatement.FieldToMatch.Ja3Fingerprint.FallbackBehavior.match` |
| `NO_MATCH` | `aws.wafv2.WebAcl.Rule.Statement.ByteMatchStatement.FieldToMatch.Ja3Fingerprint.FallbackBehavior.no_match` |

Shorthand formats: `match` or `FallbackBehavior.match`

### fallback_behavior (FallbackBehavior)

| Value | DSL Identifier |
|-------|----------------|
| `MATCH` | `aws.wafv2.WebAcl.Rule.Statement.ByteMatchStatement.FieldToMatch.Ja4Fingerprint.FallbackBehavior.match` |
| `NO_MATCH` | `aws.wafv2.WebAcl.Rule.Statement.ByteMatchStatement.FieldToMatch.Ja4Fingerprint.FallbackBehavior.no_match` |

Shorthand formats: `match` or `FallbackBehavior.match`

### invalid_fallback_behavior (BodyParsingFallbackBehavior)

| Value | DSL Identifier |
|-------|----------------|
| `MATCH` | `aws.wafv2.WebAcl.Rule.Statement.ByteMatchStatement.FieldToMatch.JsonBody.BodyParsingFallbackBehavior.match` |
| `NO_MATCH` | `aws.wafv2.WebAcl.Rule.Statement.ByteMatchStatement.FieldToMatch.JsonBody.BodyParsingFallbackBehavior.no_match` |
| `EVALUATE_AS_STRING` | `aws.wafv2.WebAcl.Rule.Statement.ByteMatchStatement.FieldToMatch.JsonBody.BodyParsingFallbackBehavior.evaluate_as_string` |

Shorthand formats: `match` or `BodyParsingFallbackBehavior.match`

### match_scope (JsonMatchScope)

| Value | DSL Identifier |
|-------|----------------|
| `ALL` | `aws.wafv2.WebAcl.Rule.Statement.ByteMatchStatement.FieldToMatch.JsonBody.JsonMatchScope.all` |
| `KEY` | `aws.wafv2.WebAcl.Rule.Statement.ByteMatchStatement.FieldToMatch.JsonBody.JsonMatchScope.key` |
| `VALUE` | `aws.wafv2.WebAcl.Rule.Statement.ByteMatchStatement.FieldToMatch.JsonBody.JsonMatchScope.value` |

Shorthand formats: `all` or `JsonMatchScope.all`

### oversize_handling (OversizeHandling)

| Value | DSL Identifier |
|-------|----------------|
| `CONTINUE` | `aws.wafv2.WebAcl.Rule.Statement.ByteMatchStatement.FieldToMatch.JsonBody.OversizeHandling.continue` |
| `MATCH` | `aws.wafv2.WebAcl.Rule.Statement.ByteMatchStatement.FieldToMatch.JsonBody.OversizeHandling.match` |
| `NO_MATCH` | `aws.wafv2.WebAcl.Rule.Statement.ByteMatchStatement.FieldToMatch.JsonBody.OversizeHandling.no_match` |

Shorthand formats: `continue` or `OversizeHandling.continue`

### scope (LabelMatchScope)

| Value | DSL Identifier |
|-------|----------------|
| `LABEL` | `aws.wafv2.WebAcl.Rule.Statement.LabelMatchStatement.LabelMatchScope.label` |
| `NAMESPACE` | `aws.wafv2.WebAcl.Rule.Statement.LabelMatchStatement.LabelMatchScope.namespace` |

Shorthand formats: `label` or `LabelMatchScope.label`

### payload_type (PayloadType)

| Value | DSL Identifier |
|-------|----------------|
| `JSON` | `aws.wafv2.WebAcl.Rule.Statement.ManagedRuleGroupStatement.ManagedRuleGroupConfig.PayloadType.json` |
| `FORM_ENCODED` | `aws.wafv2.WebAcl.Rule.Statement.ManagedRuleGroupStatement.ManagedRuleGroupConfig.PayloadType.form_encoded` |

Shorthand formats: `json` or `PayloadType.json`

### alb_low_reputation_mode (AlbLowReputationMode)

| Value | DSL Identifier |
|-------|----------------|
| `ACTIVE_UNDER_DDOS` | `aws.wafv2.WebAcl.OnSourceDDoSProtectionConfig.AlbLowReputationMode.active_under_ddos` |
| `ALWAYS_ON` | `aws.wafv2.WebAcl.OnSourceDDoSProtectionConfig.AlbLowReputationMode.always_on` |

Shorthand formats: `active_under_ddos` or `AlbLowReputationMode.active_under_ddos`

### aggregate_key_type (AggregateKeyType)

| Value | DSL Identifier |
|-------|----------------|
| `CONSTANT` | `aws.wafv2.WebAcl.Rule.Statement.RateBasedStatement.AggregateKeyType.constant` |
| `IP` | `aws.wafv2.WebAcl.Rule.Statement.RateBasedStatement.AggregateKeyType.ip` |
| `FORWARDED_IP` | `aws.wafv2.WebAcl.Rule.Statement.RateBasedStatement.AggregateKeyType.forwarded_ip` |
| `CUSTOM_KEYS` | `aws.wafv2.WebAcl.Rule.Statement.RateBasedStatement.AggregateKeyType.custom_keys` |

Shorthand formats: `constant` or `AggregateKeyType.constant`

### evaluation_window_sec (EvaluationWindowSec)

| Value | DSL Identifier |
|-------|----------------|
| `60` | `aws.wafv2.WebAcl.Rule.Statement.RateBasedStatement.EvaluationWindowSec.60` |
| `120` | `aws.wafv2.WebAcl.Rule.Statement.RateBasedStatement.EvaluationWindowSec.120` |
| `300` | `aws.wafv2.WebAcl.Rule.Statement.RateBasedStatement.EvaluationWindowSec.300` |
| `600` | `aws.wafv2.WebAcl.Rule.Statement.RateBasedStatement.EvaluationWindowSec.600` |

Shorthand formats: `60` or `EvaluationWindowSec.60`

### fallback_behavior (FallbackBehavior)

| Value | DSL Identifier |
|-------|----------------|
| `MATCH` | `aws.wafv2.WebAcl.Rule.Statement.RateBasedStatement.RateBasedStatementCustomKey.RateLimitJa3Fingerprint.FallbackBehavior.match` |
| `NO_MATCH` | `aws.wafv2.WebAcl.Rule.Statement.RateBasedStatement.RateBasedStatementCustomKey.RateLimitJa3Fingerprint.FallbackBehavior.no_match` |

Shorthand formats: `match` or `FallbackBehavior.match`

### fallback_behavior (FallbackBehavior)

| Value | DSL Identifier |
|-------|----------------|
| `MATCH` | `aws.wafv2.WebAcl.Rule.Statement.RateBasedStatement.RateBasedStatementCustomKey.RateLimitJa4Fingerprint.FallbackBehavior.match` |
| `NO_MATCH` | `aws.wafv2.WebAcl.Rule.Statement.RateBasedStatement.RateBasedStatementCustomKey.RateLimitJa4Fingerprint.FallbackBehavior.no_match` |

Shorthand formats: `match` or `FallbackBehavior.match`

### payload_type (PayloadType)

| Value | DSL Identifier |
|-------|----------------|
| `JSON` | `aws.wafv2.WebAcl.Rule.Statement.ManagedRuleGroupStatement.ManagedRuleGroupConfig.AwsManagedRulesAtpRuleSet.RequestInspection.PayloadType.json` |
| `FORM_ENCODED` | `aws.wafv2.WebAcl.Rule.Statement.ManagedRuleGroupStatement.ManagedRuleGroupConfig.AwsManagedRulesAtpRuleSet.RequestInspection.PayloadType.form_encoded` |

Shorthand formats: `json` or `PayloadType.json`

### payload_type (PayloadType)

| Value | DSL Identifier |
|-------|----------------|
| `JSON` | `aws.wafv2.WebAcl.Rule.Statement.ManagedRuleGroupStatement.ManagedRuleGroupConfig.AwsManagedRulesAcfpRuleSet.RequestInspectionAcfp.PayloadType.json` |
| `FORM_ENCODED` | `aws.wafv2.WebAcl.Rule.Statement.ManagedRuleGroupStatement.ManagedRuleGroupConfig.AwsManagedRulesAcfpRuleSet.RequestInspectionAcfp.PayloadType.form_encoded` |

Shorthand formats: `json` or `PayloadType.json`

### scope (Scope)

| Value | DSL Identifier |
|-------|----------------|
| `CLOUDFRONT` | `aws.wafv2.WebAcl.Scope.cloudfront` |
| `REGIONAL` | `aws.wafv2.WebAcl.Scope.regional` |

Shorthand formats: `cloudfront` or `Scope.cloudfront`

### comparison_operator (ComparisonOperator)

| Value | DSL Identifier |
|-------|----------------|
| `EQ` | `aws.wafv2.WebAcl.Rule.Statement.SizeConstraintStatement.ComparisonOperator.eq` |
| `NE` | `aws.wafv2.WebAcl.Rule.Statement.SizeConstraintStatement.ComparisonOperator.ne` |
| `LE` | `aws.wafv2.WebAcl.Rule.Statement.SizeConstraintStatement.ComparisonOperator.le` |
| `LT` | `aws.wafv2.WebAcl.Rule.Statement.SizeConstraintStatement.ComparisonOperator.lt` |
| `GE` | `aws.wafv2.WebAcl.Rule.Statement.SizeConstraintStatement.ComparisonOperator.ge` |
| `GT` | `aws.wafv2.WebAcl.Rule.Statement.SizeConstraintStatement.ComparisonOperator.gt` |

Shorthand formats: `eq` or `ComparisonOperator.eq`

### sensitivity_level (SensitivityLevel)

| Value | DSL Identifier |
|-------|----------------|
| `LOW` | `aws.wafv2.WebAcl.Rule.Statement.SqliMatchStatement.SensitivityLevel.low` |
| `HIGH` | `aws.wafv2.WebAcl.Rule.Statement.SqliMatchStatement.SensitivityLevel.high` |

Shorthand formats: `low` or `SensitivityLevel.low`

### type (TextTransformationType)

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

### fallback_behavior (FallbackBehavior)

| Value | DSL Identifier |
|-------|----------------|
| `MATCH` | `aws.wafv2.WebAcl.Rule.Statement.ByteMatchStatement.FieldToMatch.UriFragment.FallbackBehavior.match` |
| `NO_MATCH` | `aws.wafv2.WebAcl.Rule.Statement.ByteMatchStatement.FieldToMatch.UriFragment.FallbackBehavior.no_match` |

Shorthand formats: `match` or `FallbackBehavior.match`

## Struct Definitions

### AWSManagedRulesACFPRuleSet

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `creation_path` | String | Yes |  |
| `enable_regex_in_path` | Bool | No |  |
| `registration_page_path` | String | Yes |  |
| `request_inspection` | [Struct(RequestInspectionACFP)](#requestinspectionacfp) | Yes |  |
| `response_inspection` | [Struct(ResponseInspection)](#responseinspection) | No |  |

### AWSManagedRulesATPRuleSet

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `enable_regex_in_path` | Bool | No |  |
| `login_path` | String | Yes |  |
| `request_inspection` | [Struct(RequestInspection)](#requestinspection) | No |  |
| `response_inspection` | [Struct(ResponseInspection)](#responseinspection) | No |  |

### AWSManagedRulesAntiDDoSRuleSet

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `client_side_action_config` | [Struct(ClientSideActionConfig)](#clientsideactionconfig) | Yes |  |
| `sensitivity_to_block` | [Enum (SensitivityToAct)](#sensitivity_to_block-sensitivitytoact) | No |  |

### AWSManagedRulesBotControlRuleSet

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `enable_machine_learning` | Bool | No |  |
| `inspection_level` | [Enum (InspectionLevel)](#inspection_level-inspectionlevel) | Yes |  |

### AllowAction

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `custom_request_handling` | [Struct(CustomRequestHandling)](#customrequesthandling) | No |  |

### AndStatement

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `statements` | [List\<Statement\>](#statement) | Yes |  |

### ApplicationConfig

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `attributes` | String | Yes |  |

### AsnMatchStatement

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `asn_list` | `List<Int>` | No |  |
| `forwarded_ip_config` | [Struct(ForwardedIPConfiguration)](#forwardedipconfiguration) | No |  |

### AssociationConfig

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `request_body` | String | No |  |

### BlockAction

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `custom_response` | [Struct(CustomResponse)](#customresponse) | No |  |

### Body

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `oversize_handling` | [Enum (OversizeHandling)](#oversize_handling-oversizehandling) | No |  |

### ByteMatchStatement

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `field_to_match` | [Struct(FieldToMatch)](#fieldtomatch) | Yes |  |
| `positional_constraint` | [Enum (PositionalConstraint)](#positional_constraint-positionalconstraint) | Yes |  |
| `search_string` | String | No |  |
| `search_string_base64` | String | No |  |
| `text_transformations` | [List\<TextTransformation\>](#texttransformation) | Yes |  |

### CaptchaAction

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `custom_request_handling` | [Struct(CustomRequestHandling)](#customrequesthandling) | No |  |

### CaptchaConfig

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `immunity_time_property` | [Struct(ImmunityTimeProperty)](#immunitytimeproperty) | No |  |

### ChallengeAction

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `custom_request_handling` | [Struct(CustomRequestHandling)](#customrequesthandling) | No |  |

### ChallengeConfig

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `immunity_time_property` | [Struct(ImmunityTimeProperty)](#immunitytimeproperty) | No |  |

### ClientSideAction

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `exempt_uri_regular_expressions` | String | No |  |
| `sensitivity` | [Enum (SensitivityToAct)](#sensitivity-sensitivitytoact) | No |  |
| `usage_of_action` | [Enum (UsageOfAction)](#usage_of_action-usageofaction) | Yes |  |

### ClientSideActionConfig

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `challenge` | [Struct(ClientSideAction)](#clientsideaction) | Yes |  |

### CookieMatchPattern

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `all` | `Map<String, String>` | No | Inspect all parts of the web request cookies. |
| `excluded_cookies` | `List<String>` (items: 1..=199) | No |  |
| `included_cookies` | `List<String>` (items: 1..=199) | No |  |

### Cookies

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `match_pattern` | [Struct(CookieMatchPattern)](#cookiematchpattern) | Yes |  |
| `match_scope` | [Enum (MapMatchScope)](#match_scope-mapmatchscope) | Yes |  |
| `oversize_handling` | [Enum (OversizeHandling)](#oversize_handling-oversizehandling) | Yes |  |

### CountAction

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `custom_request_handling` | [Struct(CustomRequestHandling)](#customrequesthandling) | No |  |

### CustomHTTPHeader

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `name` | String | Yes |  |
| `value` | String | Yes |  |

### CustomRequestHandling

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `insert_headers` | [List\<CustomHTTPHeader\>](#customhttpheader) (items: 1..) | Yes | Collection of HTTP headers. |

### CustomResponse

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `custom_response_body_key` | String | No | Custom response body key. |
| `response_code` | Int(200..=599) | Yes |  |
| `response_headers` | [List\<CustomHTTPHeader\>](#customhttpheader) (items: 1..) | No | Collection of HTTP headers. |

### DataProtectionConfig

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `data_protections` | String | Yes |  |

### DefaultAction

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `allow` | [Struct(AllowAction)](#allowaction) | No |  |
| `block` | [Struct(BlockAction)](#blockaction) | No |  |

### ExcludedRule

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `name` | String | Yes |  |

### FieldIdentifier

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `identifier` | String(pattern, len: 1..=512) | Yes |  |

### FieldToMatch

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `all_query_arguments` | `Map<String, String>` | No | All query arguments of a web request. |
| `body` | [Struct(Body)](#body) | No |  |
| `cookies` | [Struct(Cookies)](#cookies) | No |  |
| `header_order` | [Struct(HeaderOrder)](#headerorder) | No |  |
| `headers` | [Struct(Headers)](#headers) | No |  |
| `ja3_fingerprint` | [Struct(JA3Fingerprint)](#ja3fingerprint) | No |  |
| `ja4_fingerprint` | [Struct(JA4Fingerprint)](#ja4fingerprint) | No |  |
| `json_body` | [Struct(JsonBody)](#jsonbody) | No |  |
| `method` | `Map<String, String>` | No | The HTTP method of a web request. The method indicates the type of operation that the request is asking the origin to perform. |
| `query_string` | `Map<String, String>` | No | The query string of a web request. This is the part of a URL that appears after a ? character, if any. |
| `single_header` | [Struct(SingleHeader)](#singleheader) | No |  |
| `single_query_argument` | [Struct(SingleQueryArgument)](#singlequeryargument) | No | One query argument in a web request, identified by name, for example UserName or SalesRegion. The name can be up to 30 characters long and isn't case sensitive. |
| `uri_fragment` | [Struct(UriFragment)](#urifragment) | No |  |
| `uri_path` | `Map<String, String>` | No | The path component of the URI of a web request. This is the part of a web request that identifies a resource, for example, /images/daily-ad.jpg. |

### ForwardedIPConfiguration

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `fallback_behavior` | [Enum (FallbackBehavior)](#fallback_behavior-fallbackbehavior) | Yes |  |
| `header_name` | String | Yes |  |

### GeoMatchStatement

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `country_codes` | `List<String>` | No |  |
| `forwarded_ip_config` | [Struct(ForwardedIPConfiguration)](#forwardedipconfiguration) | No |  |

### HeaderMatchPattern

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `all` | `Map<String, String>` | No | Inspect all parts of the web request headers. |
| `excluded_headers` | `List<String>` (items: 1..=199) | No |  |
| `included_headers` | `List<String>` (items: 1..=199) | No |  |

### HeaderOrder

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `oversize_handling` | [Enum (OversizeHandling)](#oversize_handling-oversizehandling) | Yes |  |

### Headers

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `match_pattern` | [Struct(HeaderMatchPattern)](#headermatchpattern) | Yes |  |
| `match_scope` | [Enum (MapMatchScope)](#match_scope-mapmatchscope) | Yes |  |
| `oversize_handling` | [Enum (OversizeHandling)](#oversize_handling-oversizehandling) | Yes |  |

### IPSetForwardedIPConfiguration

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `fallback_behavior` | [Enum (FallbackBehavior)](#fallback_behavior-fallbackbehavior) | Yes |  |
| `header_name` | String | Yes |  |
| `position` | [Enum (Position)](#position-position) | Yes |  |

### IPSetReferenceStatement

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `arn` | Arn | Yes |  |
| `ip_set_forwarded_ip_config` | [Struct(IPSetForwardedIPConfiguration)](#ipsetforwardedipconfiguration) | No |  |

### ImmunityTimeProperty

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `immunity_time` | Int(60..=259200) | Yes |  |

### JA3Fingerprint

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `fallback_behavior` | [Enum (FallbackBehavior)](#fallback_behavior-fallbackbehavior) | Yes |  |

### JA4Fingerprint

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `fallback_behavior` | [Enum (FallbackBehavior)](#fallback_behavior-fallbackbehavior) | Yes |  |

### JsonBody

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `invalid_fallback_behavior` | [Enum (BodyParsingFallbackBehavior)](#invalid_fallback_behavior-bodyparsingfallbackbehavior) | No |  |
| `match_pattern` | [Struct(JsonMatchPattern)](#jsonmatchpattern) | Yes |  |
| `match_scope` | [Enum (JsonMatchScope)](#match_scope-jsonmatchscope) | Yes |  |
| `oversize_handling` | [Enum (OversizeHandling)](#oversize_handling-oversizehandling) | No |  |

### JsonMatchPattern

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `all` | `Map<String, String>` | No | Inspect all parts of the web request's JSON body. |
| `included_paths` | `List<String>` | No |  |

### Label

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `name` | String | Yes |  |

### LabelMatchStatement

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `key` | String | Yes |  |
| `scope` | [Enum (LabelMatchScope)](#scope-labelmatchscope) | Yes |  |

### ManagedRuleGroupConfig

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `aws_managed_rules_acfp_rule_set` | [Struct(AWSManagedRulesACFPRuleSet)](#awsmanagedrulesacfpruleset) | No |  |
| `aws_managed_rules_atp_rule_set` | [Struct(AWSManagedRulesATPRuleSet)](#awsmanagedrulesatpruleset) | No |  |
| `aws_managed_rules_anti_d_do_s_rule_set` | [Struct(AWSManagedRulesAntiDDoSRuleSet)](#awsmanagedrulesantiddosruleset) | No |  |
| `aws_managed_rules_bot_control_rule_set` | [Struct(AWSManagedRulesBotControlRuleSet)](#awsmanagedrulesbotcontrolruleset) | No |  |
| `login_path` | String(pattern, len: 1..=256) | No |  |
| `password_field` | [Struct(FieldIdentifier)](#fieldidentifier) | No |  |
| `payload_type` | [Enum (PayloadType)](#payload_type-payloadtype) | No |  |
| `username_field` | [Struct(FieldIdentifier)](#fieldidentifier) | No |  |

### ManagedRuleGroupStatement

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `excluded_rules` | [List\<ExcludedRule\>](#excludedrule) | No |  |
| `managed_rule_group_configs` | [List\<ManagedRuleGroupConfig\>](#managedrulegroupconfig) | No | Collection of ManagedRuleGroupConfig. |
| `name` | String | Yes |  |
| `rule_action_overrides` | [List\<RuleActionOverride\>](#ruleactionoverride) (items: ..=100) | No | Action overrides for rules in the rule group. |
| `scope_down_statement` | [Struct(Statement)](#statement) | No |  |
| `vendor_name` | String | Yes |  |
| `version` | String(pattern, len: 1..=64) | No |  |

### NotStatement

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `statement` | [Struct(Statement)](#statement) | Yes |  |

### OnSourceDDoSProtectionConfig

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `alb_low_reputation_mode` | [Enum (AlbLowReputationMode)](#alb_low_reputation_mode-alblowreputationmode) | Yes |  |

### OrStatement

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `statements` | [List\<Statement\>](#statement) | Yes |  |

### OverrideAction

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `count` | `Map<String, String>` | No | Count traffic towards application. |
| `none` | `Map<String, String>` | No | Keep the RuleGroup or ManagedRuleGroup behavior as is. |

### RateBasedStatement

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `aggregate_key_type` | [Enum (AggregateKeyType)](#aggregate_key_type-aggregatekeytype) | Yes |  |
| `custom_keys` | [List\<RateBasedStatementCustomKey\>](#ratebasedstatementcustomkey) (items: ..=5) | No | Specifies the aggregate keys to use in a rate-base rule. |
| `evaluation_window_sec` | [Enum (EvaluationWindowSec)](#evaluation_window_sec-evaluationwindowsec) | No |  |
| `forwarded_ip_config` | [Struct(ForwardedIPConfiguration)](#forwardedipconfiguration) | No |  |
| `limit` | Int(10..=2000000000) | Yes |  |
| `scope_down_statement` | [Struct(Statement)](#statement) | No |  |

### RateBasedStatementCustomKey

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `asn` | String | No |  |
| `cookie` | [Struct(RateLimitCookie)](#ratelimitcookie) | No |  |
| `forwarded_ip` | Ipv4Address | No |  |
| `http_method` | String | No |  |
| `header` | [Struct(RateLimitHeader)](#ratelimitheader) | No |  |
| `ip` | Ipv4Address | No |  |
| `ja3_fingerprint` | [Struct(RateLimitJA3Fingerprint)](#ratelimitja3fingerprint) | No |  |
| `ja4_fingerprint` | [Struct(RateLimitJA4Fingerprint)](#ratelimitja4fingerprint) | No |  |
| `label_namespace` | [Struct(RateLimitLabelNamespace)](#ratelimitlabelnamespace) | No |  |
| `query_argument` | [Struct(RateLimitQueryArgument)](#ratelimitqueryargument) | No |  |
| `query_string` | [Struct(RateLimitQueryString)](#ratelimitquerystring) | No |  |
| `uri_path` | [Struct(RateLimitUriPath)](#ratelimituripath) | No |  |

### RateLimitCookie

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `name` | String(pattern, len: 1..=64) | Yes | The name of the cookie to use. |
| `text_transformations` | [List\<TextTransformation\>](#texttransformation) | Yes |  |

### RateLimitHeader

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `name` | String(pattern, len: 1..=64) | Yes | The name of the header to use. |
| `text_transformations` | [List\<TextTransformation\>](#texttransformation) | Yes |  |

### RateLimitJA3Fingerprint

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `fallback_behavior` | [Enum (FallbackBehavior)](#fallback_behavior-fallbackbehavior) | Yes |  |

### RateLimitJA4Fingerprint

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `fallback_behavior` | [Enum (FallbackBehavior)](#fallback_behavior-fallbackbehavior) | Yes |  |

### RateLimitLabelNamespace

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `namespace` | String | Yes | The namespace to use for aggregation. |

### RateLimitQueryArgument

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `name` | String(pattern, len: 1..=64) | Yes | The name of the query argument to use. |
| `text_transformations` | [List\<TextTransformation\>](#texttransformation) | Yes |  |

### RateLimitQueryString

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `text_transformations` | [List\<TextTransformation\>](#texttransformation) | Yes |  |

### RateLimitUriPath

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `text_transformations` | [List\<TextTransformation\>](#texttransformation) | Yes |  |

### RegexMatchStatement

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `field_to_match` | [Struct(FieldToMatch)](#fieldtomatch) | Yes |  |
| `regex_string` | String(len: 1..=512) | Yes |  |
| `text_transformations` | [List\<TextTransformation\>](#texttransformation) | Yes |  |

### RegexPatternSetReferenceStatement

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `arn` | Arn | Yes |  |
| `field_to_match` | [Struct(FieldToMatch)](#fieldtomatch) | Yes |  |
| `text_transformations` | [List\<TextTransformation\>](#texttransformation) | Yes |  |

### RequestInspection

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `password_field` | [Struct(FieldIdentifier)](#fieldidentifier) | Yes |  |
| `payload_type` | [Enum (PayloadType)](#payload_type-payloadtype) | Yes |  |
| `username_field` | [Struct(FieldIdentifier)](#fieldidentifier) | Yes |  |

### RequestInspectionACFP

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `address_fields` | `List<String>` | No |  |
| `email_field` | [Struct(FieldIdentifier)](#fieldidentifier) | No |  |
| `password_field` | [Struct(FieldIdentifier)](#fieldidentifier) | No |  |
| `payload_type` | [Enum (PayloadType)](#payload_type-payloadtype) | Yes |  |
| `phone_number_fields` | `List<String>` | No |  |
| `username_field` | [Struct(FieldIdentifier)](#fieldidentifier) | No |  |

### ResponseInspection

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `body_contains` | [Struct(ResponseInspectionBodyContains)](#responseinspectionbodycontains) | No |  |
| `header` | [Struct(ResponseInspectionHeader)](#responseinspectionheader) | No |  |
| `json` | [Struct(ResponseInspectionJson)](#responseinspectionjson) | No |  |
| `status_code` | [Struct(ResponseInspectionStatusCode)](#responseinspectionstatuscode) | No |  |

### ResponseInspectionBodyContains

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `failure_strings` | `List<String>` (items: 1..=5) | Yes |  |
| `success_strings` | `List<String>` (items: 1..=5) | Yes |  |

### ResponseInspectionHeader

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `failure_values` | `List<String>` (items: 1..=3) | Yes |  |
| `name` | String(pattern, len: 1..=200) | Yes |  |
| `success_values` | `List<String>` (items: 1..=3) | Yes |  |

### ResponseInspectionJson

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `failure_values` | `List<String>` (items: 1..=5) | Yes |  |
| `identifier` | String(pattern, len: 1..=512) | Yes |  |
| `success_values` | `List<String>` (items: 1..=5) | Yes |  |

### ResponseInspectionStatusCode

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `failure_codes` | `List<Int>` (items: 1..=10) | Yes |  |
| `success_codes` | `List<Int>` (items: 1..=10) | Yes |  |

### Rule

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `action` | [Struct(RuleAction)](#ruleaction) | No |  |
| `captcha_config` | [Struct(CaptchaConfig)](#captchaconfig) | No |  |
| `challenge_config` | [Struct(ChallengeConfig)](#challengeconfig) | No |  |
| `name` | String | Yes |  |
| `override_action` | [Struct(OverrideAction)](#overrideaction) | No |  |
| `priority` | Int(0..) | Yes |  |
| `rule_labels` | [List\<Label\>](#label) | No | Collection of Rule Labels. |
| `statement` | [Struct(Statement)](#statement) | Yes |  |
| `visibility_config` | [Struct(VisibilityConfig)](#visibilityconfig) | Yes |  |

### RuleAction

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `allow` | [Struct(AllowAction)](#allowaction) | No |  |
| `block` | [Struct(BlockAction)](#blockaction) | No |  |
| `captcha` | [Struct(CaptchaAction)](#captchaaction) | No |  |
| `challenge` | [Struct(ChallengeAction)](#challengeaction) | No |  |
| `count` | [Struct(CountAction)](#countaction) | No |  |

### RuleActionOverride

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `action_to_use` | [Struct(RuleAction)](#ruleaction) | Yes |  |
| `name` | String | Yes |  |

### RuleGroupReferenceStatement

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `arn` | Arn | Yes |  |
| `excluded_rules` | [List\<ExcludedRule\>](#excludedrule) | No |  |
| `rule_action_overrides` | [List\<RuleActionOverride\>](#ruleactionoverride) (items: ..=100) | No | Action overrides for rules in the rule group. |

### SingleHeader

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `name` | String | Yes |  |

### SingleQueryArgument

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `name` | String | Yes |  |

### SizeConstraintStatement

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `comparison_operator` | [Enum (ComparisonOperator)](#comparison_operator-comparisonoperator) | Yes |  |
| `field_to_match` | [Struct(FieldToMatch)](#fieldtomatch) | Yes |  |
| `size` | Float(0..=21474836480) | Yes |  |
| `text_transformations` | [List\<TextTransformation\>](#texttransformation) | Yes |  |

### SqliMatchStatement

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `field_to_match` | [Struct(FieldToMatch)](#fieldtomatch) | Yes |  |
| `sensitivity_level` | [Enum (SensitivityLevel)](#sensitivity_level-sensitivitylevel) | No |  |
| `text_transformations` | [List\<TextTransformation\>](#texttransformation) | Yes |  |

### Statement

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `and_statement` | [Struct(AndStatement)](#andstatement) | No |  |
| `asn_match_statement` | [Struct(AsnMatchStatement)](#asnmatchstatement) | No |  |
| `byte_match_statement` | [Struct(ByteMatchStatement)](#bytematchstatement) | No |  |
| `geo_match_statement` | [Struct(GeoMatchStatement)](#geomatchstatement) | No |  |
| `ip_set_reference_statement` | [Struct(IPSetReferenceStatement)](#ipsetreferencestatement) | No |  |
| `label_match_statement` | [Struct(LabelMatchStatement)](#labelmatchstatement) | No |  |
| `managed_rule_group_statement` | [Struct(ManagedRuleGroupStatement)](#managedrulegroupstatement) | No |  |
| `not_statement` | [Struct(NotStatement)](#notstatement) | No |  |
| `or_statement` | [Struct(OrStatement)](#orstatement) | No |  |
| `rate_based_statement` | [Struct(RateBasedStatement)](#ratebasedstatement) | No |  |
| `regex_match_statement` | [Struct(RegexMatchStatement)](#regexmatchstatement) | No |  |
| `regex_pattern_set_reference_statement` | [Struct(RegexPatternSetReferenceStatement)](#regexpatternsetreferencestatement) | No |  |
| `rule_group_reference_statement` | [Struct(RuleGroupReferenceStatement)](#rulegroupreferencestatement) | No |  |
| `size_constraint_statement` | [Struct(SizeConstraintStatement)](#sizeconstraintstatement) | No |  |
| `sqli_match_statement` | [Struct(SqliMatchStatement)](#sqlimatchstatement) | No |  |
| `xss_match_statement` | [Struct(XssMatchStatement)](#xssmatchstatement) | No |  |

### TextTransformation

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `priority` | Int(0..) | Yes |  |
| `type` | [Enum (TextTransformationType)](#type-texttransformationtype) | Yes |  |

### UriFragment

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `fallback_behavior` | [Enum (FallbackBehavior)](#fallback_behavior-fallbackbehavior) | No |  |

### VisibilityConfig

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `cloud_watch_metrics_enabled` | Bool | Yes |  |
| `metric_name` | String(len: 1..=128) | Yes |  |
| `sampled_requests_enabled` | Bool | Yes |  |

### XssMatchStatement

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `field_to_match` | [Struct(FieldToMatch)](#fieldtomatch) | Yes |  |
| `text_transformations` | [List\<TextTransformation\>](#texttransformation) | Yes |  |

## Attribute Reference

### `arn`

- **Type:** Arn

### `capacity`

- **Type:** Int(0..)

### `id`

- **Type:** String

### `label_namespace`

- **Type:** String

