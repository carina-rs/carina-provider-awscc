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

### sensitivity_to_block (AWSManagedRulesAntiDDoSRuleSetSensitivityToAct)

| Value | DSL Identifier |
|-------|----------------|
| `LOW` | `awscc.wafv2.WebAcl.AWSManagedRulesAntiDDoSRuleSetSensitivityToAct.low` |
| `MEDIUM` | `awscc.wafv2.WebAcl.AWSManagedRulesAntiDDoSRuleSetSensitivityToAct.medium` |
| `HIGH` | `awscc.wafv2.WebAcl.AWSManagedRulesAntiDDoSRuleSetSensitivityToAct.high` |

Shorthand formats: `low` or `AWSManagedRulesAntiDDoSRuleSetSensitivityToAct.low`

### inspection_level (AWSManagedRulesBotControlRuleSetInspectionLevel)

| Value | DSL Identifier |
|-------|----------------|
| `COMMON` | `awscc.wafv2.WebAcl.AWSManagedRulesBotControlRuleSetInspectionLevel.common` |
| `TARGETED` | `awscc.wafv2.WebAcl.AWSManagedRulesBotControlRuleSetInspectionLevel.targeted` |

Shorthand formats: `common` or `AWSManagedRulesBotControlRuleSetInspectionLevel.common`

### oversize_handling (BodyOversizeHandling)

| Value | DSL Identifier |
|-------|----------------|
| `CONTINUE` | `awscc.wafv2.WebAcl.BodyOversizeHandling.continue` |
| `MATCH` | `awscc.wafv2.WebAcl.BodyOversizeHandling.match` |
| `NO_MATCH` | `awscc.wafv2.WebAcl.BodyOversizeHandling.no_match` |

Shorthand formats: `continue` or `BodyOversizeHandling.continue`

### positional_constraint (ByteMatchStatementPositionalConstraint)

| Value | DSL Identifier |
|-------|----------------|
| `EXACTLY` | `awscc.wafv2.WebAcl.ByteMatchStatementPositionalConstraint.exactly` |
| `STARTS_WITH` | `awscc.wafv2.WebAcl.ByteMatchStatementPositionalConstraint.starts_with` |
| `ENDS_WITH` | `awscc.wafv2.WebAcl.ByteMatchStatementPositionalConstraint.ends_with` |
| `CONTAINS` | `awscc.wafv2.WebAcl.ByteMatchStatementPositionalConstraint.contains` |
| `CONTAINS_WORD` | `awscc.wafv2.WebAcl.ByteMatchStatementPositionalConstraint.contains_word` |

Shorthand formats: `exactly` or `ByteMatchStatementPositionalConstraint.exactly`

### sensitivity (ClientSideActionSensitivityToAct)

| Value | DSL Identifier |
|-------|----------------|
| `LOW` | `awscc.wafv2.WebAcl.ClientSideActionSensitivityToAct.low` |
| `MEDIUM` | `awscc.wafv2.WebAcl.ClientSideActionSensitivityToAct.medium` |
| `HIGH` | `awscc.wafv2.WebAcl.ClientSideActionSensitivityToAct.high` |

Shorthand formats: `low` or `ClientSideActionSensitivityToAct.low`

### usage_of_action (ClientSideActionUsageOfAction)

| Value | DSL Identifier |
|-------|----------------|
| `ENABLED` | `awscc.wafv2.WebAcl.ClientSideActionUsageOfAction.enabled` |
| `DISABLED` | `awscc.wafv2.WebAcl.ClientSideActionUsageOfAction.disabled` |

Shorthand formats: `enabled` or `ClientSideActionUsageOfAction.enabled`

### match_scope (CookiesMapMatchScope)

| Value | DSL Identifier |
|-------|----------------|
| `ALL` | `awscc.wafv2.WebAcl.CookiesMapMatchScope.all` |
| `KEY` | `awscc.wafv2.WebAcl.CookiesMapMatchScope.key` |
| `VALUE` | `awscc.wafv2.WebAcl.CookiesMapMatchScope.value` |

Shorthand formats: `all` or `CookiesMapMatchScope.all`

### oversize_handling (CookiesOversizeHandling)

| Value | DSL Identifier |
|-------|----------------|
| `CONTINUE` | `awscc.wafv2.WebAcl.CookiesOversizeHandling.continue` |
| `MATCH` | `awscc.wafv2.WebAcl.CookiesOversizeHandling.match` |
| `NO_MATCH` | `awscc.wafv2.WebAcl.CookiesOversizeHandling.no_match` |

Shorthand formats: `continue` or `CookiesOversizeHandling.continue`

### fallback_behavior (ForwardedIPConfigurationFallbackBehavior)

| Value | DSL Identifier |
|-------|----------------|
| `MATCH` | `awscc.wafv2.WebAcl.ForwardedIPConfigurationFallbackBehavior.match` |
| `NO_MATCH` | `awscc.wafv2.WebAcl.ForwardedIPConfigurationFallbackBehavior.no_match` |

Shorthand formats: `match` or `ForwardedIPConfigurationFallbackBehavior.match`

### oversize_handling (HeaderOrderOversizeHandling)

| Value | DSL Identifier |
|-------|----------------|
| `CONTINUE` | `awscc.wafv2.WebAcl.HeaderOrderOversizeHandling.continue` |
| `MATCH` | `awscc.wafv2.WebAcl.HeaderOrderOversizeHandling.match` |
| `NO_MATCH` | `awscc.wafv2.WebAcl.HeaderOrderOversizeHandling.no_match` |

Shorthand formats: `continue` or `HeaderOrderOversizeHandling.continue`

### match_scope (HeadersMapMatchScope)

| Value | DSL Identifier |
|-------|----------------|
| `ALL` | `awscc.wafv2.WebAcl.HeadersMapMatchScope.all` |
| `KEY` | `awscc.wafv2.WebAcl.HeadersMapMatchScope.key` |
| `VALUE` | `awscc.wafv2.WebAcl.HeadersMapMatchScope.value` |

Shorthand formats: `all` or `HeadersMapMatchScope.all`

### oversize_handling (HeadersOversizeHandling)

| Value | DSL Identifier |
|-------|----------------|
| `CONTINUE` | `awscc.wafv2.WebAcl.HeadersOversizeHandling.continue` |
| `MATCH` | `awscc.wafv2.WebAcl.HeadersOversizeHandling.match` |
| `NO_MATCH` | `awscc.wafv2.WebAcl.HeadersOversizeHandling.no_match` |

Shorthand formats: `continue` or `HeadersOversizeHandling.continue`

### fallback_behavior (IPSetForwardedIPConfigurationFallbackBehavior)

| Value | DSL Identifier |
|-------|----------------|
| `MATCH` | `awscc.wafv2.WebAcl.IPSetForwardedIPConfigurationFallbackBehavior.match` |
| `NO_MATCH` | `awscc.wafv2.WebAcl.IPSetForwardedIPConfigurationFallbackBehavior.no_match` |

Shorthand formats: `match` or `IPSetForwardedIPConfigurationFallbackBehavior.match`

### position (IPSetForwardedIPConfigurationPosition)

| Value | DSL Identifier |
|-------|----------------|
| `FIRST` | `awscc.wafv2.WebAcl.IPSetForwardedIPConfigurationPosition.first` |
| `LAST` | `awscc.wafv2.WebAcl.IPSetForwardedIPConfigurationPosition.last` |
| `ANY` | `awscc.wafv2.WebAcl.IPSetForwardedIPConfigurationPosition.any` |

Shorthand formats: `first` or `IPSetForwardedIPConfigurationPosition.first`

### fallback_behavior (JA3FingerprintFallbackBehavior)

| Value | DSL Identifier |
|-------|----------------|
| `MATCH` | `awscc.wafv2.WebAcl.JA3FingerprintFallbackBehavior.match` |
| `NO_MATCH` | `awscc.wafv2.WebAcl.JA3FingerprintFallbackBehavior.no_match` |

Shorthand formats: `match` or `JA3FingerprintFallbackBehavior.match`

### fallback_behavior (JA4FingerprintFallbackBehavior)

| Value | DSL Identifier |
|-------|----------------|
| `MATCH` | `awscc.wafv2.WebAcl.JA4FingerprintFallbackBehavior.match` |
| `NO_MATCH` | `awscc.wafv2.WebAcl.JA4FingerprintFallbackBehavior.no_match` |

Shorthand formats: `match` or `JA4FingerprintFallbackBehavior.match`

### invalid_fallback_behavior (JsonBodyBodyParsingFallbackBehavior)

| Value | DSL Identifier |
|-------|----------------|
| `MATCH` | `awscc.wafv2.WebAcl.JsonBodyBodyParsingFallbackBehavior.match` |
| `NO_MATCH` | `awscc.wafv2.WebAcl.JsonBodyBodyParsingFallbackBehavior.no_match` |
| `EVALUATE_AS_STRING` | `awscc.wafv2.WebAcl.JsonBodyBodyParsingFallbackBehavior.evaluate_as_string` |

Shorthand formats: `match` or `JsonBodyBodyParsingFallbackBehavior.match`

### match_scope (JsonBodyJsonMatchScope)

| Value | DSL Identifier |
|-------|----------------|
| `ALL` | `awscc.wafv2.WebAcl.JsonBodyJsonMatchScope.all` |
| `KEY` | `awscc.wafv2.WebAcl.JsonBodyJsonMatchScope.key` |
| `VALUE` | `awscc.wafv2.WebAcl.JsonBodyJsonMatchScope.value` |

Shorthand formats: `all` or `JsonBodyJsonMatchScope.all`

### oversize_handling (JsonBodyOversizeHandling)

| Value | DSL Identifier |
|-------|----------------|
| `CONTINUE` | `awscc.wafv2.WebAcl.JsonBodyOversizeHandling.continue` |
| `MATCH` | `awscc.wafv2.WebAcl.JsonBodyOversizeHandling.match` |
| `NO_MATCH` | `awscc.wafv2.WebAcl.JsonBodyOversizeHandling.no_match` |

Shorthand formats: `continue` or `JsonBodyOversizeHandling.continue`

### scope (LabelMatchStatementLabelMatchScope)

| Value | DSL Identifier |
|-------|----------------|
| `LABEL` | `awscc.wafv2.WebAcl.LabelMatchStatementLabelMatchScope.label` |
| `NAMESPACE` | `awscc.wafv2.WebAcl.LabelMatchStatementLabelMatchScope.namespace` |

Shorthand formats: `label` or `LabelMatchStatementLabelMatchScope.label`

### payload_type (ManagedRuleGroupConfigPayloadType)

| Value | DSL Identifier |
|-------|----------------|
| `JSON` | `awscc.wafv2.WebAcl.ManagedRuleGroupConfigPayloadType.json` |
| `FORM_ENCODED` | `awscc.wafv2.WebAcl.ManagedRuleGroupConfigPayloadType.form_encoded` |

Shorthand formats: `json` or `ManagedRuleGroupConfigPayloadType.json`

### alb_low_reputation_mode (OnSourceDDoSProtectionConfigAlbLowReputationMode)

| Value | DSL Identifier |
|-------|----------------|
| `ACTIVE_UNDER_DDOS` | `awscc.wafv2.WebAcl.OnSourceDDoSProtectionConfigAlbLowReputationMode.active_under_ddos` |
| `ALWAYS_ON` | `awscc.wafv2.WebAcl.OnSourceDDoSProtectionConfigAlbLowReputationMode.always_on` |

Shorthand formats: `active_under_ddos` or `OnSourceDDoSProtectionConfigAlbLowReputationMode.active_under_ddos`

### aggregate_key_type (RateBasedStatementAggregateKeyType)

| Value | DSL Identifier |
|-------|----------------|
| `CONSTANT` | `awscc.wafv2.WebAcl.RateBasedStatementAggregateKeyType.constant` |
| `IP` | `awscc.wafv2.WebAcl.RateBasedStatementAggregateKeyType.ip` |
| `FORWARDED_IP` | `awscc.wafv2.WebAcl.RateBasedStatementAggregateKeyType.forwarded_ip` |
| `CUSTOM_KEYS` | `awscc.wafv2.WebAcl.RateBasedStatementAggregateKeyType.custom_keys` |

Shorthand formats: `constant` or `RateBasedStatementAggregateKeyType.constant`

### evaluation_window_sec (RateBasedStatementEvaluationWindowSec)

| Value | DSL Identifier |
|-------|----------------|
| `60` | `awscc.wafv2.WebAcl.RateBasedStatementEvaluationWindowSec.60` |
| `120` | `awscc.wafv2.WebAcl.RateBasedStatementEvaluationWindowSec.120` |
| `300` | `awscc.wafv2.WebAcl.RateBasedStatementEvaluationWindowSec.300` |
| `600` | `awscc.wafv2.WebAcl.RateBasedStatementEvaluationWindowSec.600` |

Shorthand formats: `60` or `RateBasedStatementEvaluationWindowSec.60`

### fallback_behavior (RateLimitJA3FingerprintFallbackBehavior)

| Value | DSL Identifier |
|-------|----------------|
| `MATCH` | `awscc.wafv2.WebAcl.RateLimitJA3FingerprintFallbackBehavior.match` |
| `NO_MATCH` | `awscc.wafv2.WebAcl.RateLimitJA3FingerprintFallbackBehavior.no_match` |

Shorthand formats: `match` or `RateLimitJA3FingerprintFallbackBehavior.match`

### fallback_behavior (RateLimitJA4FingerprintFallbackBehavior)

| Value | DSL Identifier |
|-------|----------------|
| `MATCH` | `awscc.wafv2.WebAcl.RateLimitJA4FingerprintFallbackBehavior.match` |
| `NO_MATCH` | `awscc.wafv2.WebAcl.RateLimitJA4FingerprintFallbackBehavior.no_match` |

Shorthand formats: `match` or `RateLimitJA4FingerprintFallbackBehavior.match`

### payload_type (RequestInspectionPayloadType)

| Value | DSL Identifier |
|-------|----------------|
| `JSON` | `awscc.wafv2.WebAcl.RequestInspectionPayloadType.json` |
| `FORM_ENCODED` | `awscc.wafv2.WebAcl.RequestInspectionPayloadType.form_encoded` |

Shorthand formats: `json` or `RequestInspectionPayloadType.json`

### payload_type (RequestInspectionACFPPayloadType)

| Value | DSL Identifier |
|-------|----------------|
| `JSON` | `awscc.wafv2.WebAcl.RequestInspectionACFPPayloadType.json` |
| `FORM_ENCODED` | `awscc.wafv2.WebAcl.RequestInspectionACFPPayloadType.form_encoded` |

Shorthand formats: `json` or `RequestInspectionACFPPayloadType.json`

### scope (Scope)

| Value | DSL Identifier |
|-------|----------------|
| `CLOUDFRONT` | `awscc.wafv2.WebAcl.Scope.cloudfront` |
| `REGIONAL` | `awscc.wafv2.WebAcl.Scope.regional` |

Shorthand formats: `cloudfront` or `Scope.cloudfront`

### comparison_operator (SizeConstraintStatementComparisonOperator)

| Value | DSL Identifier |
|-------|----------------|
| `EQ` | `awscc.wafv2.WebAcl.SizeConstraintStatementComparisonOperator.eq` |
| `NE` | `awscc.wafv2.WebAcl.SizeConstraintStatementComparisonOperator.ne` |
| `LE` | `awscc.wafv2.WebAcl.SizeConstraintStatementComparisonOperator.le` |
| `LT` | `awscc.wafv2.WebAcl.SizeConstraintStatementComparisonOperator.lt` |
| `GE` | `awscc.wafv2.WebAcl.SizeConstraintStatementComparisonOperator.ge` |
| `GT` | `awscc.wafv2.WebAcl.SizeConstraintStatementComparisonOperator.gt` |

Shorthand formats: `eq` or `SizeConstraintStatementComparisonOperator.eq`

### sensitivity_level (SqliMatchStatementSensitivityLevel)

| Value | DSL Identifier |
|-------|----------------|
| `LOW` | `awscc.wafv2.WebAcl.SqliMatchStatementSensitivityLevel.low` |
| `HIGH` | `awscc.wafv2.WebAcl.SqliMatchStatementSensitivityLevel.high` |

Shorthand formats: `low` or `SqliMatchStatementSensitivityLevel.low`

### type (TextTransformationTextTransformationType)

| Value | DSL Identifier |
|-------|----------------|
| `NONE` | `awscc.wafv2.WebAcl.TextTransformationTextTransformationType.none` |
| `COMPRESS_WHITE_SPACE` | `awscc.wafv2.WebAcl.TextTransformationTextTransformationType.compress_white_space` |
| `HTML_ENTITY_DECODE` | `awscc.wafv2.WebAcl.TextTransformationTextTransformationType.html_entity_decode` |
| `LOWERCASE` | `awscc.wafv2.WebAcl.TextTransformationTextTransformationType.lowercase` |
| `CMD_LINE` | `awscc.wafv2.WebAcl.TextTransformationTextTransformationType.cmd_line` |
| `URL_DECODE` | `awscc.wafv2.WebAcl.TextTransformationTextTransformationType.url_decode` |
| `BASE64_DECODE` | `awscc.wafv2.WebAcl.TextTransformationTextTransformationType.base64_decode` |
| `HEX_DECODE` | `awscc.wafv2.WebAcl.TextTransformationTextTransformationType.hex_decode` |
| `MD5` | `awscc.wafv2.WebAcl.TextTransformationTextTransformationType.md5` |
| `REPLACE_COMMENTS` | `awscc.wafv2.WebAcl.TextTransformationTextTransformationType.replace_comments` |
| `ESCAPE_SEQ_DECODE` | `awscc.wafv2.WebAcl.TextTransformationTextTransformationType.escape_seq_decode` |
| `SQL_HEX_DECODE` | `awscc.wafv2.WebAcl.TextTransformationTextTransformationType.sql_hex_decode` |
| `CSS_DECODE` | `awscc.wafv2.WebAcl.TextTransformationTextTransformationType.css_decode` |
| `JS_DECODE` | `awscc.wafv2.WebAcl.TextTransformationTextTransformationType.js_decode` |
| `NORMALIZE_PATH` | `awscc.wafv2.WebAcl.TextTransformationTextTransformationType.normalize_path` |
| `NORMALIZE_PATH_WIN` | `awscc.wafv2.WebAcl.TextTransformationTextTransformationType.normalize_path_win` |
| `REMOVE_NULLS` | `awscc.wafv2.WebAcl.TextTransformationTextTransformationType.remove_nulls` |
| `REPLACE_NULLS` | `awscc.wafv2.WebAcl.TextTransformationTextTransformationType.replace_nulls` |
| `BASE64_DECODE_EXT` | `awscc.wafv2.WebAcl.TextTransformationTextTransformationType.base64_decode_ext` |
| `URL_DECODE_UNI` | `awscc.wafv2.WebAcl.TextTransformationTextTransformationType.url_decode_uni` |
| `UTF8_TO_UNICODE` | `awscc.wafv2.WebAcl.TextTransformationTextTransformationType.utf8_to_unicode` |

Shorthand formats: `none` or `TextTransformationTextTransformationType.none`

### fallback_behavior (UriFragmentFallbackBehavior)

| Value | DSL Identifier |
|-------|----------------|
| `MATCH` | `awscc.wafv2.WebAcl.UriFragmentFallbackBehavior.match` |
| `NO_MATCH` | `awscc.wafv2.WebAcl.UriFragmentFallbackBehavior.no_match` |

Shorthand formats: `match` or `UriFragmentFallbackBehavior.match`

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
| `sensitivity_to_block` | [Enum (AWSManagedRulesAntiDDoSRuleSetSensitivityToAct)](#sensitivity_to_block-awsmanagedrulesantiddosrulesetsensitivitytoact) | No |  |

### AWSManagedRulesBotControlRuleSet

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `enable_machine_learning` | Bool | No |  |
| `inspection_level` | [Enum (AWSManagedRulesBotControlRuleSetInspectionLevel)](#inspection_level-awsmanagedrulesbotcontrolrulesetinspectionlevel) | Yes |  |

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
| `oversize_handling` | [Enum (BodyOversizeHandling)](#oversize_handling-bodyoversizehandling) | No |  |

### ByteMatchStatement

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `field_to_match` | [Struct(FieldToMatch)](#fieldtomatch) | Yes |  |
| `positional_constraint` | [Enum (ByteMatchStatementPositionalConstraint)](#positional_constraint-bytematchstatementpositionalconstraint) | Yes |  |
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
| `sensitivity` | [Enum (ClientSideActionSensitivityToAct)](#sensitivity-clientsideactionsensitivitytoact) | No |  |
| `usage_of_action` | [Enum (ClientSideActionUsageOfAction)](#usage_of_action-clientsideactionusageofaction) | Yes |  |

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
| `match_scope` | [Enum (CookiesMapMatchScope)](#match_scope-cookiesmapmatchscope) | Yes |  |
| `oversize_handling` | [Enum (CookiesOversizeHandling)](#oversize_handling-cookiesoversizehandling) | Yes |  |

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
| `fallback_behavior` | [Enum (ForwardedIPConfigurationFallbackBehavior)](#fallback_behavior-forwardedipconfigurationfallbackbehavior) | Yes |  |
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
| `oversize_handling` | [Enum (HeaderOrderOversizeHandling)](#oversize_handling-headerorderoversizehandling) | Yes |  |

### Headers

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `match_pattern` | [Struct(HeaderMatchPattern)](#headermatchpattern) | Yes |  |
| `match_scope` | [Enum (HeadersMapMatchScope)](#match_scope-headersmapmatchscope) | Yes |  |
| `oversize_handling` | [Enum (HeadersOversizeHandling)](#oversize_handling-headersoversizehandling) | Yes |  |

### IPSetForwardedIPConfiguration

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `fallback_behavior` | [Enum (IPSetForwardedIPConfigurationFallbackBehavior)](#fallback_behavior-ipsetforwardedipconfigurationfallbackbehavior) | Yes |  |
| `header_name` | String | Yes |  |
| `position` | [Enum (IPSetForwardedIPConfigurationPosition)](#position-ipsetforwardedipconfigurationposition) | Yes |  |

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
| `fallback_behavior` | [Enum (JA3FingerprintFallbackBehavior)](#fallback_behavior-ja3fingerprintfallbackbehavior) | Yes |  |

### JA4Fingerprint

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `fallback_behavior` | [Enum (JA4FingerprintFallbackBehavior)](#fallback_behavior-ja4fingerprintfallbackbehavior) | Yes |  |

### JsonBody

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `invalid_fallback_behavior` | [Enum (JsonBodyBodyParsingFallbackBehavior)](#invalid_fallback_behavior-jsonbodybodyparsingfallbackbehavior) | No |  |
| `match_pattern` | [Struct(JsonMatchPattern)](#jsonmatchpattern) | Yes |  |
| `match_scope` | [Enum (JsonBodyJsonMatchScope)](#match_scope-jsonbodyjsonmatchscope) | Yes |  |
| `oversize_handling` | [Enum (JsonBodyOversizeHandling)](#oversize_handling-jsonbodyoversizehandling) | No |  |

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
| `scope` | [Enum (LabelMatchStatementLabelMatchScope)](#scope-labelmatchstatementlabelmatchscope) | Yes |  |

### ManagedRuleGroupConfig

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `aws_managed_rules_acfp_rule_set` | [Struct(AWSManagedRulesACFPRuleSet)](#awsmanagedrulesacfpruleset) | No |  |
| `aws_managed_rules_atp_rule_set` | [Struct(AWSManagedRulesATPRuleSet)](#awsmanagedrulesatpruleset) | No |  |
| `aws_managed_rules_anti_d_do_s_rule_set` | [Struct(AWSManagedRulesAntiDDoSRuleSet)](#awsmanagedrulesantiddosruleset) | No |  |
| `aws_managed_rules_bot_control_rule_set` | [Struct(AWSManagedRulesBotControlRuleSet)](#awsmanagedrulesbotcontrolruleset) | No |  |
| `login_path` | String(pattern, len: 1..=256) | No |  |
| `password_field` | [Struct(FieldIdentifier)](#fieldidentifier) | No |  |
| `payload_type` | [Enum (ManagedRuleGroupConfigPayloadType)](#payload_type-managedrulegroupconfigpayloadtype) | No |  |
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
| `alb_low_reputation_mode` | [Enum (OnSourceDDoSProtectionConfigAlbLowReputationMode)](#alb_low_reputation_mode-onsourceddosprotectionconfigalblowreputationmode) | Yes |  |

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
| `aggregate_key_type` | [Enum (RateBasedStatementAggregateKeyType)](#aggregate_key_type-ratebasedstatementaggregatekeytype) | Yes |  |
| `custom_keys` | [List\<RateBasedStatementCustomKey\>](#ratebasedstatementcustomkey) (items: ..=5) | No | Specifies the aggregate keys to use in a rate-base rule. |
| `evaluation_window_sec` | [Enum (RateBasedStatementEvaluationWindowSec)](#evaluation_window_sec-ratebasedstatementevaluationwindowsec) | No |  |
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
| `fallback_behavior` | [Enum (RateLimitJA3FingerprintFallbackBehavior)](#fallback_behavior-ratelimitja3fingerprintfallbackbehavior) | Yes |  |

### RateLimitJA4Fingerprint

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `fallback_behavior` | [Enum (RateLimitJA4FingerprintFallbackBehavior)](#fallback_behavior-ratelimitja4fingerprintfallbackbehavior) | Yes |  |

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
| `payload_type` | [Enum (RequestInspectionPayloadType)](#payload_type-requestinspectionpayloadtype) | Yes |  |
| `username_field` | [Struct(FieldIdentifier)](#fieldidentifier) | Yes |  |

### RequestInspectionACFP

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `address_fields` | `List<String>` | No |  |
| `email_field` | [Struct(FieldIdentifier)](#fieldidentifier) | No |  |
| `password_field` | [Struct(FieldIdentifier)](#fieldidentifier) | No |  |
| `payload_type` | [Enum (RequestInspectionACFPPayloadType)](#payload_type-requestinspectionacfppayloadtype) | Yes |  |
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
| `comparison_operator` | [Enum (SizeConstraintStatementComparisonOperator)](#comparison_operator-sizeconstraintstatementcomparisonoperator) | Yes |  |
| `field_to_match` | [Struct(FieldToMatch)](#fieldtomatch) | Yes |  |
| `size` | Float(0..=21474836480) | Yes |  |
| `text_transformations` | [List\<TextTransformation\>](#texttransformation) | Yes |  |

### SqliMatchStatement

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `field_to_match` | [Struct(FieldToMatch)](#fieldtomatch) | Yes |  |
| `sensitivity_level` | [Enum (SqliMatchStatementSensitivityLevel)](#sensitivity_level-sqlimatchstatementsensitivitylevel) | No |  |
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
| `type` | [Enum (TextTransformationTextTransformationType)](#type-texttransformationtexttransformationtype) | Yes |  |

### UriFragment

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `fallback_behavior` | [Enum (UriFragmentFallbackBehavior)](#fallback_behavior-urifragmentfallbackbehavior) | No |  |

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

