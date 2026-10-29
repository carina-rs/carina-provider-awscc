---
title: "awscc.wafv2.WebAcl"
---


CloudFormation Type: `AWS::WAFv2::WebACL`

Contains the Rules that identify the requests that you want to allow, block, or count. In a WebACL, you also specify a default action (ALLOW or BLOCK), and the action for each Rule that you add to a WebACL, for example, block requests from specified IP addresses or block requests from specified referrers. You also associate the WebACL with a CloudFront distribution to identify the requests that you want AWS WAF to filter. If you add more than one Rule to a WebACL, a request needs to match only one of the specifications to be allowed, blocked, or counted.

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

- **Type:** String
- **Required:** No

### `data_protection_config`

- **Type:** [Struct(DataProtectionConfig)](#data_protection_config-1)
- **Required:** No

Collection of dataProtects.

### `default_action`

- **Type:** [Struct(DefaultAction)](#default_action-1)
- **Required:** Yes

### `description`

- **Type:** String
- **Required:** No

### `name`

- **Type:** String
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

- **Type:** `Map<String, String>`
- **Required:** No

### `token_domains`

- **Type:** String
- **Required:** No

### `visibility_config`

- **Type:** [Struct(VisibilityConfig)](#visibility_config-1)
- **Required:** Yes

## Enum Values

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

### statement.rate_based_statement.evaluation_window_sec (EvaluationWindowSec)

| Value | DSL Identifier |
|-------|----------------|
| `60` | `aws.wafv2.WebAcl.Rule.Statement.RateBasedStatement.EvaluationWindowSec.60` |
| `120` | `aws.wafv2.WebAcl.Rule.Statement.RateBasedStatement.EvaluationWindowSec.120` |
| `300` | `aws.wafv2.WebAcl.Rule.Statement.RateBasedStatement.EvaluationWindowSec.300` |
| `600` | `aws.wafv2.WebAcl.Rule.Statement.RateBasedStatement.EvaluationWindowSec.600` |

Shorthand formats: `60` or `EvaluationWindowSec.60`

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
| `attributes` | List\<[Struct(ApplicationAttribute)](#application_configattributes)\> | Yes |  |

### application_config.attributes

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `name` | String | Yes |  |
| `values` | String | Yes |  |

### association_config

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `request_body` | String | No |  |

### block_action

Used by: `default_action.block`, `rules.action.block`

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `custom_response` | [Struct(CustomResponse)](#block_actioncustom_response) | No |  |

### block_action.custom_response

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `custom_response_body_key` | String | No | Custom response body key. |
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
| `name` | String | Yes |  |
| `value` | String | Yes |  |

### custom_request_handling

Used by: `allow_action.custom_request_handling`, `rule_action.captcha.custom_request_handling`, `rule_action.challenge.custom_request_handling`, `rule_action.count.custom_request_handling`

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `insert_headers` | List\<[Struct(CustomHTTPHeader)](#custom_http_header)\> (items: 1..) | Yes | Collection of HTTP headers. |

### data_protection_config

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `data_protections` | List\<[Struct(DataProtect)](#data_protection_configdata_protections)\> | Yes |  |

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
| `field_keys` | `List<String>` | No | List of field keys to protect |
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
| `name` | String | Yes |  |

### field_identifier

Used by: `statement.managed_rule_group_statement.managed_rule_group_configs.aws_managed_rules_acfp_rule_set.request_inspection.email_field`, `statement.managed_rule_group_statement.managed_rule_group_configs.aws_managed_rules_acfp_rule_set.request_inspection.password_field`, `statement.managed_rule_group_statement.managed_rule_group_configs.aws_managed_rules_acfp_rule_set.request_inspection.username_field`, `statement.managed_rule_group_statement.managed_rule_group_configs.aws_managed_rules_atp_rule_set.request_inspection.password_field`, `statement.managed_rule_group_statement.managed_rule_group_configs.aws_managed_rules_atp_rule_set.request_inspection.username_field`, `statement.managed_rule_group_statement.managed_rule_group_configs.password_field`, `statement.managed_rule_group_statement.managed_rule_group_configs.username_field`

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `identifier` | String(pattern, len: 1..=512) | Yes |  |

### field_to_match

Used by: `statement.byte_match_statement.field_to_match`, `statement.regex_match_statement.field_to_match`, `statement.regex_pattern_set_reference_statement.field_to_match`, `statement.size_constraint_statement.field_to_match`, `statement.sqli_match_statement.field_to_match`, `statement.xss_match_statement.field_to_match`

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `all_query_arguments` | `Map<String, String>` | No | All query arguments of a web request. |
| `body` | [Struct(Body)](#field_to_matchbody) | No |  |
| `cookies` | [Struct(Cookies)](#field_to_matchcookies) | No |  |
| `header_order` | [Struct(HeaderOrder)](#field_to_matchheader_order) | No |  |
| `headers` | [Struct(Headers)](#field_to_matchheaders) | No |  |
| `ja3_fingerprint` | [Struct(JA3Fingerprint)](#field_to_matchja3_fingerprint) | No |  |
| `ja4_fingerprint` | [Struct(JA4Fingerprint)](#field_to_matchja4_fingerprint) | No |  |
| `json_body` | [Struct(JsonBody)](#field_to_matchjson_body) | No |  |
| `method` | `Map<String, String>` | No | The HTTP method of a web request. The method indicates the type of operation that the request is asking the origin to perform. |
| `query_string` | `Map<String, String>` | No | The query string of a web request. This is the part of a URL that appears after a ? character, if any. |
| `single_header` | [Struct(SingleHeader)](#field_to_matchsingle_header) | No |  |
| `single_query_argument` | [Struct(SingleQueryArgument)](#field_to_matchsingle_query_argument) | No | One query argument in a web request, identified by name, for example UserName or SalesRegion. The name can be up to 30 characters long and isn't case sensitive. |
| `uri_fragment` | [Struct(UriFragment)](#field_to_matchuri_fragment) | No |  |
| `uri_path` | `Map<String, String>` | No | The path component of the URI of a web request. This is the part of a web request that identifies a resource, for example, /images/daily-ad.jpg. |

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
| `all` | `Map<String, String>` | No | Inspect all parts of the web request cookies. |
| `excluded_cookies` | `List<String>` (items: 1..=199) | No |  |
| `included_cookies` | `List<String>` (items: 1..=199) | No |  |

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
| `all` | `Map<String, String>` | No | Inspect all parts of the web request headers. |
| `excluded_headers` | `List<String>` (items: 1..=199) | No |  |
| `included_headers` | `List<String>` (items: 1..=199) | No |  |

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
| `all` | `Map<String, String>` | No | Inspect all parts of the web request's JSON body. |
| `included_paths` | `List<String>` | No |  |

### field_to_match.single_header

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `name` | String | Yes |  |

### field_to_match.single_query_argument

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `name` | String | Yes |  |

### field_to_match.uri_fragment

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `fallback_behavior` | [Enum (FallbackBehavior)](#field_to_matchuri_fragmentfallback_behavior-fallbackbehavior) | No |  |

### forwarded_ip_configuration

Used by: `statement.asn_match_statement.forwarded_ip_config`, `statement.geo_match_statement.forwarded_ip_config`, `statement.rate_based_statement.forwarded_ip_config`

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `fallback_behavior` | [Enum (FallbackBehavior)](#forwarded_ip_configurationfallback_behavior-fallbackbehavior) | Yes |  |
| `header_name` | String | Yes |  |

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
| `failure_strings` | `List<String>` (items: 1..=5) | Yes |  |
| `success_strings` | `List<String>` (items: 1..=5) | Yes |  |

### response_inspection.header

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `failure_values` | `List<String>` (items: 1..=3) | Yes |  |
| `name` | String(pattern, len: 1..=200) | Yes |  |
| `success_values` | `List<String>` (items: 1..=3) | Yes |  |

### response_inspection.json

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `failure_values` | `List<String>` (items: 1..=5) | Yes |  |
| `identifier` | String(pattern, len: 1..=512) | Yes |  |
| `success_values` | `List<String>` (items: 1..=5) | Yes |  |

### response_inspection.status_code

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `failure_codes` | `List<Int>` (items: 1..=10) | Yes |  |
| `success_codes` | `List<Int>` (items: 1..=10) | Yes |  |

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
| `name` | String | Yes |  |

### rules

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `action` | [Struct(RuleAction)](#rule_action) | No |  |
| `captcha_config` | [Struct(CaptchaConfig)](#captcha_config-1) | No |  |
| `challenge_config` | [Struct(ChallengeConfig)](#challenge_config-1) | No |  |
| `name` | String | Yes |  |
| `override_action` | [Struct(OverrideAction)](#rulesoverride_action) | No |  |
| `priority` | Int(0..) | Yes |  |
| `rule_labels` | List\<[Struct(Label)](#rulesrule_labels)\> | No | Collection of Rule Labels. |
| `statement` | [Struct(Statement)](#statement) | Yes |  |
| `visibility_config` | [Struct(VisibilityConfig)](#visibility_config-1) | Yes |  |

### rules.override_action

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `count` | `Map<String, String>` | No | Count traffic towards application. |
| `none` | `Map<String, String>` | No | Keep the RuleGroup or ManagedRuleGroup behavior as is. |

### rules.rule_labels

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `name` | String | Yes |  |

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
| `statements` | List\<[Struct(Statement)](#statement)\> | Yes |  |

### statement.asn_match_statement

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `asn_list` | `List<Int>` | No |  |
| `forwarded_ip_config` | [Struct(ForwardedIPConfiguration)](#forwarded_ip_configuration) | No |  |

### statement.byte_match_statement

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `field_to_match` | [Struct(FieldToMatch)](#field_to_match) | Yes |  |
| `positional_constraint` | [Enum (PositionalConstraint)](#statementbyte_match_statementpositional_constraint-positionalconstraint) | Yes |  |
| `search_string` | String | No |  |
| `search_string_base64` | String | No |  |
| `text_transformations` | List\<[Struct(TextTransformation)](#text_transformation)\> | Yes |  |

### statement.geo_match_statement

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `country_codes` | `List<String>` | No |  |
| `forwarded_ip_config` | [Struct(ForwardedIPConfiguration)](#forwarded_ip_configuration) | No |  |

### statement.ip_set_reference_statement

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `arn` | Arn | Yes |  |
| `ip_set_forwarded_ip_config` | [Struct(IPSetForwardedIPConfiguration)](#statementip_set_reference_statementip_set_forwarded_ip_config) | No |  |

### statement.ip_set_reference_statement.ip_set_forwarded_ip_config

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `fallback_behavior` | [Enum (FallbackBehavior)](#statementip_set_reference_statementip_set_forwarded_ip_configfallback_behavior-fallbackbehavior) | Yes |  |
| `header_name` | String | Yes |  |
| `position` | [Enum (Position)](#statementip_set_reference_statementip_set_forwarded_ip_configposition-position) | Yes |  |

### statement.label_match_statement

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `key` | String | Yes |  |
| `scope` | [Enum (LabelMatchScope)](#statementlabel_match_statementscope-labelmatchscope) | Yes |  |

### statement.managed_rule_group_statement

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `excluded_rules` | List\<[Struct(ExcludedRule)](#excluded_rule)\> | No |  |
| `managed_rule_group_configs` | List\<[Struct(ManagedRuleGroupConfig)](#statementmanaged_rule_group_statementmanaged_rule_group_configs)\> | No | Collection of ManagedRuleGroupConfig. |
| `name` | String | Yes |  |
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
| `creation_path` | String | Yes |  |
| `enable_regex_in_path` | Bool | No |  |
| `registration_page_path` | String | Yes |  |
| `request_inspection` | [Struct(RequestInspectionACFP)](#statementmanaged_rule_group_statementmanaged_rule_group_configsaws_managed_rules_acfp_rule_setrequest_inspection) | Yes |  |
| `response_inspection` | [Struct(ResponseInspection)](#response_inspection) | No |  |

### statement.managed_rule_group_statement.managed_rule_group_configs.aws_managed_rules_acfp_rule_set.request_inspection

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `address_fields` | `List<String>` | No |  |
| `email_field` | [Struct(FieldIdentifier)](#field_identifier) | No |  |
| `password_field` | [Struct(FieldIdentifier)](#field_identifier) | No |  |
| `payload_type` | [Enum (PayloadType)](#statementmanaged_rule_group_statementmanaged_rule_group_configsaws_managed_rules_acfp_rule_setrequest_inspectionpayload_type-payloadtype) | Yes |  |
| `phone_number_fields` | `List<String>` | No |  |
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
| `regex_string` | String | No |  |

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
| `statements` | List\<[Struct(Statement)](#statement)\> | Yes |  |

### statement.rate_based_statement

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `aggregate_key_type` | [Enum (AggregateKeyType)](#statementrate_based_statementaggregate_key_type-aggregatekeytype) | Yes |  |
| `custom_keys` | List\<[Struct(RateBasedStatementCustomKey)](#statementrate_based_statementcustom_keys)\> (items: ..=5) | No | Specifies the aggregate keys to use in a rate-base rule. |
| `evaluation_window_sec` | [Enum (EvaluationWindowSec)](#statementrate_based_statementevaluation_window_sec-evaluationwindowsec) | No |  |
| `forwarded_ip_config` | [Struct(ForwardedIPConfiguration)](#forwarded_ip_configuration) | No |  |
| `limit` | Int(10..=2000000000) | Yes |  |
| `scope_down_statement` | [Struct(Statement)](#statement) | No |  |

### statement.rate_based_statement.custom_keys

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `asn` | String | No |  |
| `cookie` | [Struct(RateLimitCookie)](#statementrate_based_statementcustom_keyscookie) | No |  |
| `forwarded_ip` | Ipv4Address | No |  |
| `http_method` | String | No |  |
| `header` | [Struct(RateLimitHeader)](#statementrate_based_statementcustom_keysheader) | No |  |
| `ip` | Ipv4Address | No |  |
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
| `text_transformations` | List\<[Struct(TextTransformation)](#text_transformation)\> | Yes |  |

### statement.rate_based_statement.custom_keys.header

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `name` | String(pattern, len: 1..=64) | Yes | The name of the header to use. |
| `text_transformations` | List\<[Struct(TextTransformation)](#text_transformation)\> | Yes |  |

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
| `namespace` | String | Yes | The namespace to use for aggregation. |

### statement.rate_based_statement.custom_keys.query_argument

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `name` | String(pattern, len: 1..=64) | Yes | The name of the query argument to use. |
| `text_transformations` | List\<[Struct(TextTransformation)](#text_transformation)\> | Yes |  |

### statement.rate_based_statement.custom_keys.query_string

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `text_transformations` | List\<[Struct(TextTransformation)](#text_transformation)\> | Yes |  |

### statement.rate_based_statement.custom_keys.uri_path

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `text_transformations` | List\<[Struct(TextTransformation)](#text_transformation)\> | Yes |  |

### statement.regex_match_statement

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `field_to_match` | [Struct(FieldToMatch)](#field_to_match) | Yes |  |
| `regex_string` | String(len: 1..=512) | Yes |  |
| `text_transformations` | List\<[Struct(TextTransformation)](#text_transformation)\> | Yes |  |

### statement.regex_pattern_set_reference_statement

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `arn` | Arn | Yes |  |
| `field_to_match` | [Struct(FieldToMatch)](#field_to_match) | Yes |  |
| `text_transformations` | List\<[Struct(TextTransformation)](#text_transformation)\> | Yes |  |

### statement.rule_group_reference_statement

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `arn` | Arn | Yes |  |
| `excluded_rules` | List\<[Struct(ExcludedRule)](#excluded_rule)\> | No |  |
| `rule_action_overrides` | List\<[Struct(RuleActionOverride)](#rule_action_override)\> (items: ..=100) | No | Action overrides for rules in the rule group. |

### statement.size_constraint_statement

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `comparison_operator` | [Enum (ComparisonOperator)](#statementsize_constraint_statementcomparison_operator-comparisonoperator) | Yes |  |
| `field_to_match` | [Struct(FieldToMatch)](#field_to_match) | Yes |  |
| `size` | Float(0..=21474836480) | Yes |  |
| `text_transformations` | List\<[Struct(TextTransformation)](#text_transformation)\> | Yes |  |

### statement.sqli_match_statement

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `field_to_match` | [Struct(FieldToMatch)](#field_to_match) | Yes |  |
| `sensitivity_level` | [Enum (SensitivityLevel)](#statementsqli_match_statementsensitivity_level-sensitivitylevel) | No |  |
| `text_transformations` | List\<[Struct(TextTransformation)](#text_transformation)\> | Yes |  |

### statement.xss_match_statement

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `field_to_match` | [Struct(FieldToMatch)](#field_to_match) | Yes |  |
| `text_transformations` | List\<[Struct(TextTransformation)](#text_transformation)\> | Yes |  |

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
| `metric_name` | String(len: 1..=128) | Yes |  |
| `sampled_requests_enabled` | Bool | Yes |  |

## Attribute Reference

### `arn`

- **Type:** Arn

### `capacity`

- **Type:** Int(0..)

### `id`

- **Type:** String

### `label_namespace`

- **Type:** String

