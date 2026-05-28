//! web_acl schema definition for AWS Cloud Control
//!
//! Auto-generated from CloudFormation schema: AWS::WAFv2::WebACL
//!
//! DO NOT EDIT MANUALLY - regenerate with carina-codegen

use super::AwsccSchemaConfig;
use super::tags_type;
use super::validate_tags_map;
use carina_core::resource::{ConcreteValue, Value};
use carina_core::schema::{
    AttributeSchema, AttributeType, ResourceSchema, StructField, legacy_validator, types,
};
use regex::Regex;

const VALID_AWS_MANAGED_RULES_ANTI_D_DO_S_RULE_SET_SENSITIVITY_TO_BLOCK: &[&str] =
    &["LOW", "MEDIUM", "HIGH"];

const VALID_AWS_MANAGED_RULES_BOT_CONTROL_RULE_SET_INSPECTION_LEVEL: &[&str] =
    &["COMMON", "TARGETED"];

const VALID_BODY_OVERSIZE_HANDLING: &[&str] = &["CONTINUE", "MATCH", "NO_MATCH"];

const VALID_BYTE_MATCH_STATEMENT_POSITIONAL_CONSTRAINT: &[&str] = &[
    "EXACTLY",
    "STARTS_WITH",
    "ENDS_WITH",
    "CONTAINS",
    "CONTAINS_WORD",
];

const VALID_CLIENT_SIDE_ACTION_SENSITIVITY: &[&str] = &["LOW", "MEDIUM", "HIGH"];

const VALID_CLIENT_SIDE_ACTION_USAGE_OF_ACTION: &[&str] = &["ENABLED", "DISABLED"];

const VALID_COOKIES_MATCH_SCOPE: &[&str] = &["ALL", "KEY", "VALUE"];

const VALID_COOKIES_OVERSIZE_HANDLING: &[&str] = &["CONTINUE", "MATCH", "NO_MATCH"];

const VALID_CUSTOM_RESPONSE_BODY_CONTENT_TYPE: &[&str] =
    &["TEXT_PLAIN", "TEXT_HTML", "APPLICATION_JSON"];

const VALID_DATA_PROTECT_ACTION: &[&str] = &["SUBSTITUTION", "HASH"];

const VALID_FIELD_TO_PROTECT_FIELD_TYPE: &[&str] = &[
    "SINGLE_HEADER",
    "SINGLE_COOKIE",
    "SINGLE_QUERY_ARGUMENT",
    "QUERY_STRING",
    "BODY",
];

const VALID_FORWARDED_IP_CONFIGURATION_FALLBACK_BEHAVIOR: &[&str] = &["MATCH", "NO_MATCH"];

const VALID_HEADER_ORDER_OVERSIZE_HANDLING: &[&str] = &["CONTINUE", "MATCH", "NO_MATCH"];

const VALID_HEADERS_MATCH_SCOPE: &[&str] = &["ALL", "KEY", "VALUE"];

const VALID_HEADERS_OVERSIZE_HANDLING: &[&str] = &["CONTINUE", "MATCH", "NO_MATCH"];

const VALID_IP_SET_FORWARDED_IP_CONFIGURATION_FALLBACK_BEHAVIOR: &[&str] = &["MATCH", "NO_MATCH"];

const VALID_IP_SET_FORWARDED_IP_CONFIGURATION_POSITION: &[&str] = &["FIRST", "LAST", "ANY"];

const VALID_JA3_FINGERPRINT_FALLBACK_BEHAVIOR: &[&str] = &["MATCH", "NO_MATCH"];

const VALID_JA4_FINGERPRINT_FALLBACK_BEHAVIOR: &[&str] = &["MATCH", "NO_MATCH"];

const VALID_JSON_BODY_INVALID_FALLBACK_BEHAVIOR: &[&str] =
    &["MATCH", "NO_MATCH", "EVALUATE_AS_STRING"];

const VALID_JSON_BODY_MATCH_SCOPE: &[&str] = &["ALL", "KEY", "VALUE"];

const VALID_JSON_BODY_OVERSIZE_HANDLING: &[&str] = &["CONTINUE", "MATCH", "NO_MATCH"];

const VALID_MANAGED_RULE_GROUP_CONFIG_PAYLOAD_TYPE: &[&str] = &["JSON", "FORM_ENCODED"];

const VALID_ON_SOURCE_D_DO_S_PROTECTION_CONFIG_ALB_LOW_REPUTATION_MODE: &[&str] =
    &["ACTIVE_UNDER_DDOS", "ALWAYS_ON"];

const VALID_RATE_BASED_STATEMENT_AGGREGATE_KEY_TYPE: &[&str] =
    &["CONSTANT", "IP", "FORWARDED_IP", "CUSTOM_KEYS"];

const VALID_RATE_BASED_STATEMENT_EVALUATION_WINDOW_SEC: &[&str] = &["60", "120", "300", "600"];

const VALID_RATE_LIMIT_JA3_FINGERPRINT_FALLBACK_BEHAVIOR: &[&str] = &["MATCH", "NO_MATCH"];

const VALID_RATE_LIMIT_JA4_FINGERPRINT_FALLBACK_BEHAVIOR: &[&str] = &["MATCH", "NO_MATCH"];

const VALID_REQUEST_BODY_ASSOCIATED_RESOURCE_TYPE_CONFIG_DEFAULT_SIZE_INSPECTION_LIMIT: &[&str] =
    &["KB_16", "KB_32", "KB_48", "KB_64"];

const VALID_REQUEST_INSPECTION_PAYLOAD_TYPE: &[&str] = &["JSON", "FORM_ENCODED"];

const VALID_REQUEST_INSPECTION_ACFP_PAYLOAD_TYPE: &[&str] = &["JSON", "FORM_ENCODED"];

const VALID_SCOPE: &[&str] = &["CLOUDFRONT", "REGIONAL"];

const VALID_SIZE_CONSTRAINT_STATEMENT_COMPARISON_OPERATOR: &[&str] =
    &["EQ", "NE", "LE", "LT", "GE", "GT"];

const VALID_SQLI_MATCH_STATEMENT_SENSITIVITY_LEVEL: &[&str] = &["LOW", "HIGH"];

const VALID_TEXT_TRANSFORMATION_TYPE: &[&str] = &[
    "NONE",
    "COMPRESS_WHITE_SPACE",
    "HTML_ENTITY_DECODE",
    "LOWERCASE",
    "CMD_LINE",
    "URL_DECODE",
    "BASE64_DECODE",
    "HEX_DECODE",
    "MD5",
    "REPLACE_COMMENTS",
    "ESCAPE_SEQ_DECODE",
    "SQL_HEX_DECODE",
    "CSS_DECODE",
    "JS_DECODE",
    "NORMALIZE_PATH",
    "NORMALIZE_PATH_WIN",
    "REMOVE_NULLS",
    "REPLACE_NULLS",
    "BASE64_DECODE_EXT",
    "URL_DECODE_UNI",
    "UTF8_TO_UNICODE",
];

const VALID_URI_FRAGMENT_FALLBACK_BEHAVIOR: &[&str] = &["MATCH", "NO_MATCH"];

fn validate_capacity_range(value: &Value) -> Result<(), String> {
    if let Value::Concrete(ConcreteValue::Int(n)) = value {
        if *n < 0 {
            Err(format!("Value {} is out of range 0..", n))
        } else {
            Ok(())
        }
    } else {
        Err("Expected integer".to_string())
    }
}

fn validate_immunity_time_range(value: &Value) -> Result<(), String> {
    if let Value::Concrete(ConcreteValue::Int(n)) = value {
        if *n < 60 || *n > 259200 {
            Err(format!("Value {} is out of range 60..=259200", n))
        } else {
            Ok(())
        }
    } else {
        Err("Expected integer".to_string())
    }
}

fn validate_size_range(value: &Value) -> Result<(), String> {
    let n = match value {
        Value::Concrete(ConcreteValue::Int(i)) => *i as f64,
        Value::Concrete(ConcreteValue::Float(f)) => *f,
        _ => return Err("Expected number".to_string()),
    };
    if !(0.0..=21474836480.0).contains(&n) {
        Err(format!("Value {} is out of range 0..=21474836480", n))
    } else {
        Ok(())
    }
}

#[allow(dead_code)]
fn validate_string_pattern_6fd4a12c0ce64c08(value: &Value) -> Result<(), String> {
    if let Value::Concrete(ConcreteValue::String(s)) = value {
        static RE: std::sync::LazyLock<Regex> =
            std::sync::LazyLock::new(|| Regex::new("^[\\w\\-]+$").expect("invalid pattern regex"));
        if RE.is_match(s) {
            Ok(())
        } else {
            Err(format!("Value '{}' does not match pattern ^[\\w\\-]+$", s))
        }
    } else {
        Err("Expected string".to_string())
    }
}

#[allow(dead_code)]
fn validate_string_pattern_eef4eb302f1cdd5a(value: &Value) -> Result<(), String> {
    if let Value::Concrete(ConcreteValue::String(s)) = value {
        static RE: std::sync::LazyLock<Regex> = std::sync::LazyLock::new(|| {
            Regex::new(
                "^[a-zA-Z0-9=:#@/\\-,.][a-zA-Z0-9+=:#@/\\-,.\\s]+[a-zA-Z0-9+=:#@/\\-,.]{1,256}$",
            )
            .expect("invalid pattern regex")
        });
        if RE.is_match(s) {
            Ok(())
        } else {
            Err(format!(
                "Value '{}' does not match pattern ^[a-zA-Z0-9=:#@/\\-,.][a-zA-Z0-9+=:#@/\\-,.\\s]+[a-zA-Z0-9+=:#@/\\-,.]{{1,256}}$",
                s
            ))
        }
    } else {
        Err("Expected string".to_string())
    }
}

#[allow(dead_code)]
fn validate_string_pattern_2d93cc844f6d4014(value: &Value) -> Result<(), String> {
    if let Value::Concrete(ConcreteValue::String(s)) = value {
        static RE: std::sync::LazyLock<Regex> = std::sync::LazyLock::new(|| {
            Regex::new("^[a-zA-Z0-9-]+{1,255}$").expect("invalid pattern regex")
        });
        if RE.is_match(s) {
            Ok(())
        } else {
            Err(format!(
                "Value '{}' does not match pattern ^[a-zA-Z0-9-]+{{1,255}}$",
                s
            ))
        }
    } else {
        Err("Expected string".to_string())
    }
}

#[allow(dead_code)]
fn validate_string_pattern_d04f4c3802439b73(value: &Value) -> Result<(), String> {
    if let Value::Concrete(ConcreteValue::String(s)) = value {
        static RE: std::sync::LazyLock<Regex> = std::sync::LazyLock::new(|| {
            Regex::new("^[0-9a-f]{8}-(?:[0-9a-f]{4}-){3}[0-9a-f]{12}$")
                .expect("invalid pattern regex")
        });
        if RE.is_match(s) {
            Ok(())
        } else {
            Err(format!(
                "Value '{}' does not match pattern ^[0-9a-f]{{8}}-(?:[0-9a-f]{{4}}-){{3}}[0-9a-f]{{12}}$",
                s
            ))
        }
    } else {
        Err("Expected string".to_string())
    }
}

#[allow(dead_code)]
fn validate_string_pattern_b3fc65b549fb77bd(value: &Value) -> Result<(), String> {
    if let Value::Concrete(ConcreteValue::String(s)) = value {
        static RE: std::sync::LazyLock<Regex> = std::sync::LazyLock::new(|| {
            Regex::new("^[0-9A-Za-z_:-]{1,1024}$").expect("invalid pattern regex")
        });
        if RE.is_match(s) {
            Ok(())
        } else {
            Err(format!(
                "Value '{}' does not match pattern ^[0-9A-Za-z_:-]{{1,1024}}$",
                s
            ))
        }
    } else {
        Err("Expected string".to_string())
    }
}

#[allow(dead_code)]
fn validate_string_pattern_42f7eceb887966ad(value: &Value) -> Result<(), String> {
    if let Value::Concrete(ConcreteValue::String(s)) = value {
        static RE: std::sync::LazyLock<Regex> = std::sync::LazyLock::new(|| {
            Regex::new("^[0-9A-Za-z_-]{1,128}$").expect("invalid pattern regex")
        });
        if RE.is_match(s) {
            Ok(())
        } else {
            Err(format!(
                "Value '{}' does not match pattern ^[0-9A-Za-z_-]{{1,128}}$",
                s
            ))
        }
    } else {
        Err("Expected string".to_string())
    }
}

#[allow(dead_code)]
fn validate_list_items_max_5(value: &Value) -> Result<(), String> {
    if let Value::Concrete(ConcreteValue::List(items)) = value {
        let len = items.len();
        if len > 5 {
            Err(format!("List has {} items, expected ..=5", len))
        } else {
            Ok(())
        }
    } else {
        Err("Expected list".to_string())
    }
}

#[allow(dead_code)]
fn validate_list_items_1_199(value: &Value) -> Result<(), String> {
    if let Value::Concrete(ConcreteValue::List(items)) = value {
        let len = items.len();
        if !(1..=199).contains(&len) {
            Err(format!("List has {} items, expected 1..=199", len))
        } else {
            Ok(())
        }
    } else {
        Err("Expected list".to_string())
    }
}

#[allow(dead_code)]
fn validate_list_items_1_10(value: &Value) -> Result<(), String> {
    if let Value::Concrete(ConcreteValue::List(items)) = value {
        let len = items.len();
        if !(1..=10).contains(&len) {
            Err(format!("List has {} items, expected 1..=10", len))
        } else {
            Ok(())
        }
    } else {
        Err("Expected list".to_string())
    }
}

#[allow(dead_code)]
fn validate_list_items_1_5(value: &Value) -> Result<(), String> {
    if let Value::Concrete(ConcreteValue::List(items)) = value {
        let len = items.len();
        if !(1..=5).contains(&len) {
            Err(format!("List has {} items, expected 1..=5", len))
        } else {
            Ok(())
        }
    } else {
        Err("Expected list".to_string())
    }
}

#[allow(dead_code)]
fn validate_list_items_1_3(value: &Value) -> Result<(), String> {
    if let Value::Concrete(ConcreteValue::List(items)) = value {
        let len = items.len();
        if !(1..=3).contains(&len) {
            Err(format!("List has {} items, expected 1..=3", len))
        } else {
            Ok(())
        }
    } else {
        Err("Expected list".to_string())
    }
}

#[allow(dead_code)]
fn validate_list_items_min_1(value: &Value) -> Result<(), String> {
    if let Value::Concrete(ConcreteValue::List(items)) = value {
        let len = items.len();
        if len < 1 {
            Err(format!("List has {} items, expected 1..", len))
        } else {
            Ok(())
        }
    } else {
        Err("Expected list".to_string())
    }
}

#[allow(dead_code)]
fn validate_list_items_max_100(value: &Value) -> Result<(), String> {
    if let Value::Concrete(ConcreteValue::List(items)) = value {
        let len = items.len();
        if len > 100 {
            Err(format!("List has {} items, expected ..=100", len))
        } else {
            Ok(())
        }
    } else {
        Err("Expected list".to_string())
    }
}

fn validate_string_length_1_2(value: &Value) -> Result<(), String> {
    if let Value::Concrete(ConcreteValue::String(s)) = value {
        let len = s.chars().count();
        if !(1..=2).contains(&len) {
            Err(format!("String length {} is out of range 1..=2", len))
        } else {
            Ok(())
        }
    } else {
        Ok(())
    }
}

fn validate_string_length_1_128(value: &Value) -> Result<(), String> {
    if let Value::Concrete(ConcreteValue::String(s)) = value {
        let len = s.chars().count();
        if !(1..=128).contains(&len) {
            Err(format!("String length {} is out of range 1..=128", len))
        } else {
            Ok(())
        }
    } else {
        Ok(())
    }
}

fn validate_string_length_1_512(value: &Value) -> Result<(), String> {
    if let Value::Concrete(ConcreteValue::String(s)) = value {
        let len = s.chars().count();
        if !(1..=512).contains(&len) {
            Err(format!("String length {} is out of range 1..=512", len))
        } else {
            Ok(())
        }
    } else {
        Ok(())
    }
}

#[allow(dead_code)]
fn validate_string_pattern_4e09c821aed9e752_len_1_256(value: &Value) -> Result<(), String> {
    if let Value::Concrete(ConcreteValue::String(s)) = value {
        static RE: std::sync::LazyLock<Regex> =
            std::sync::LazyLock::new(|| Regex::new(".*\\S.*").expect("invalid pattern regex"));
        if !RE.is_match(s) {
            return Err(format!("Value '{}' does not match pattern .*\\S.*", s));
        }
        let len = s.chars().count();
        if !(1..=256).contains(&len) {
            return Err(format!("String length {} is out of range 1..=256", len));
        }
        Ok(())
    } else {
        Err("Expected string".to_string())
    }
}

#[allow(dead_code)]
fn validate_string_pattern_4e09c821aed9e752_len_1_512(value: &Value) -> Result<(), String> {
    if let Value::Concrete(ConcreteValue::String(s)) = value {
        static RE: std::sync::LazyLock<Regex> =
            std::sync::LazyLock::new(|| Regex::new(".*\\S.*").expect("invalid pattern regex"));
        if !RE.is_match(s) {
            return Err(format!("Value '{}' does not match pattern .*\\S.*", s));
        }
        let len = s.chars().count();
        if !(1..=512).contains(&len) {
            return Err(format!("String length {} is out of range 1..=512", len));
        }
        Ok(())
    } else {
        Err("Expected string".to_string())
    }
}

#[allow(dead_code)]
fn validate_string_pattern_82a037d29be8d222_len_1_64(value: &Value) -> Result<(), String> {
    if let Value::Concrete(ConcreteValue::String(s)) = value {
        static RE: std::sync::LazyLock<Regex> = std::sync::LazyLock::new(|| {
            Regex::new("^[\\w#:\\.\\-/]+$").expect("invalid pattern regex")
        });
        if !RE.is_match(s) {
            return Err(format!(
                "Value '{}' does not match pattern ^[\\w#:\\.\\-/]+$",
                s
            ));
        }
        let len = s.chars().count();
        if !(1..=64).contains(&len) {
            return Err(format!("String length {} is out of range 1..=64", len));
        }
        Ok(())
    } else {
        Err("Expected string".to_string())
    }
}

/// Returns the schema config for wafv2_web_acl (AWS::WAFv2::WebACL)
pub fn wafv2_web_acl_config() -> AwsccSchemaConfig {
    AwsccSchemaConfig {
        aws_type_name: "AWS::WAFv2::WebACL",
        resource_type_name: "wafv2.WebAcl",
        has_tags: true,
        schema: ResourceSchema::new("wafv2.WebAcl")
        .with_description("Contains the Rules that identify the requests that you want to allow, block, or count. In a WebACL, you also specify a default action (ALLOW or BLOCK), and the action for each Rule that you add to a WebACL, for example, block requests from specified IP addresses or block requests from specified referrers. You also associate the WebACL with a CloudFront distribution to identify the requests that you want AWS WAF to filter. If you add more than one Rule to a WebACL, a request needs to match only one of the specifications to be allowed, blocked, or counted.")
        .attribute(
            AttributeSchema::new("application_config", AttributeType::Ref("ApplicationConfig".to_string()))
                .with_description("Collection of application attributes.")
                .with_provider_name("ApplicationConfig"),
        )
        .attribute(
            AttributeSchema::new("arn", super::arn())
                .read_only()
                .with_provider_name("Arn"),
        )
        .attribute(
            AttributeSchema::new("association_config", AttributeType::Ref("AssociationConfig".to_string()))
                .with_provider_name("AssociationConfig"),
        )
        .attribute(
            AttributeSchema::new("capacity", AttributeType::Custom {
                identity: None,
                pattern: None,
                length: None,
                base: Box::new(AttributeType::Int),
                validate: legacy_validator(validate_capacity_range),
                to_dsl: None,
            })
                .read_only()
                .with_provider_name("Capacity"),
        )
        .attribute(
            AttributeSchema::new("captcha_config", AttributeType::Ref("CaptchaConfig".to_string()))
                .with_provider_name("CaptchaConfig"),
        )
        .attribute(
            AttributeSchema::new("challenge_config", AttributeType::Ref("ChallengeConfig".to_string()))
                .with_provider_name("ChallengeConfig"),
        )
        .attribute(
            AttributeSchema::new("custom_response_bodies", AttributeType::String)
                .with_provider_name("CustomResponseBodies"),
        )
        .attribute(
            AttributeSchema::new("data_protection_config", AttributeType::Ref("DataProtectionConfig".to_string()))
                .with_description("Collection of dataProtects.")
                .with_provider_name("DataProtectionConfig"),
        )
        .attribute(
            AttributeSchema::new("default_action", AttributeType::Ref("DefaultAction".to_string()))
                .required()
                .with_provider_name("DefaultAction"),
        )
        .attribute(
            AttributeSchema::new("description", AttributeType::Custom {
                identity: None,
                pattern: Some("^[a-zA-Z0-9=:#@/\\-,.][a-zA-Z0-9+=:#@/\\-,.\\s]+[a-zA-Z0-9+=:#@/\\-,.]{1,256}$".to_string()),
                length: None,
                base: Box::new(AttributeType::String),
                validate: legacy_validator(validate_string_pattern_eef4eb302f1cdd5a),
                to_dsl: None,
            })
                .with_provider_name("Description"),
        )
        .attribute(
            AttributeSchema::new("id", AttributeType::Custom {
                identity: None,
                pattern: Some("^[0-9a-f]{8}-(?:[0-9a-f]{4}-){3}[0-9a-f]{12}$".to_string()),
                length: None,
                base: Box::new(AttributeType::String),
                validate: legacy_validator(validate_string_pattern_d04f4c3802439b73),
                to_dsl: None,
            })
                .read_only()
                .with_provider_name("Id"),
        )
        .attribute(
            AttributeSchema::new("label_namespace", AttributeType::Custom {
                identity: None,
                pattern: Some("^[0-9A-Za-z_:-]{1,1024}$".to_string()),
                length: None,
                base: Box::new(AttributeType::String),
                validate: legacy_validator(validate_string_pattern_b3fc65b549fb77bd),
                to_dsl: None,
            })
                .read_only()
                .with_provider_name("LabelNamespace"),
        )
        .attribute(
            AttributeSchema::new("name", AttributeType::Custom {
                identity: None,
                pattern: Some("^[0-9A-Za-z_-]{1,128}$".to_string()),
                length: None,
                base: Box::new(AttributeType::String),
                validate: legacy_validator(validate_string_pattern_42f7eceb887966ad),
                to_dsl: None,
            })
                .create_only()
                .with_provider_name("Name"),
        )
        .attribute(
            AttributeSchema::new("on_source_d_do_s_protection_config", AttributeType::Ref("OnSourceDDoSProtectionConfig".to_string()))
                .with_provider_name("OnSourceDDoSProtectionConfig"),
        )
        .attribute(
            AttributeSchema::new("rules", AttributeType::list(AttributeType::Struct {
                    name: "Rule".to_string(),
                    fields: vec![
                    StructField::new("action", AttributeType::Ref("RuleAction".to_string())).with_provider_name("Action"),
                    StructField::new("captcha_config", AttributeType::Ref("CaptchaConfig".to_string())).with_provider_name("CaptchaConfig"),
                    StructField::new("challenge_config", AttributeType::Ref("ChallengeConfig".to_string())).with_provider_name("ChallengeConfig"),
                    StructField::new("name", AttributeType::Custom {
                identity: None,
                pattern: Some("^[0-9A-Za-z_-]{1,128}$".to_string()),
                length: None,
                base: Box::new(AttributeType::String),
                validate: legacy_validator(validate_string_pattern_42f7eceb887966ad),
                to_dsl: None,
            }).required().with_provider_name("Name"),
                    StructField::new("override_action", AttributeType::Ref("OverrideAction".to_string())).with_provider_name("OverrideAction"),
                    StructField::new("priority", AttributeType::String).required().with_provider_name("Priority"),
                    StructField::new("rule_labels", AttributeType::list(AttributeType::Struct {
                    name: "Label".to_string(),
                    fields: vec![
                    StructField::new("name", AttributeType::Custom {
                identity: None,
                pattern: Some("^[0-9A-Za-z_:-]{1,1024}$".to_string()),
                length: None,
                base: Box::new(AttributeType::String),
                validate: legacy_validator(validate_string_pattern_b3fc65b549fb77bd),
                to_dsl: None,
            }).required().with_provider_name("Name")
                    ],
                })).with_description("Collection of Rule Labels.").with_provider_name("RuleLabels").with_block_name("rule_label"),
                    StructField::new("statement", AttributeType::Ref("Statement".to_string())).required().with_provider_name("Statement"),
                    StructField::new("visibility_config", AttributeType::Ref("VisibilityConfig".to_string())).required().with_provider_name("VisibilityConfig")
                    ],
                }))
                .with_description("Collection of Rules.")
                .with_provider_name("Rules")
                .with_block_name("rule"),
        )
        .attribute(
            AttributeSchema::new("scope", AttributeType::StringEnum {
                name: "Scope".to_string(),
                values: vec!["CLOUDFRONT".to_string(), "REGIONAL".to_string()],
                identity: Some(carina_core::schema::string_enum_identity("Scope", Some("awscc.wafv2.WebAcl"))),
                dsl_aliases: vec![("CLOUDFRONT".to_string(), "cloudfront".to_string()), ("REGIONAL".to_string(), "regional".to_string())],
            })
                .required()
                .create_only()
                .with_provider_name("Scope"),
        )
        .attribute(
            AttributeSchema::new("tags", tags_type())
                .with_provider_name("Tags"),
        )
        .attribute(
            AttributeSchema::new("token_domains", AttributeType::list(AttributeType::Custom {
                identity: None,
                pattern: Some("^[\\w\\.\\-/]+$".to_string()),
                length: Some((Some(1), Some(253))),
                base: Box::new(AttributeType::String),
                validate: legacy_validator(validate_string_pattern_7dc332c889f363a0_len_1_253),
                to_dsl: None,
            }))
                .with_provider_name("TokenDomains"),
        )
        .attribute(
            AttributeSchema::new("visibility_config", AttributeType::Ref("VisibilityConfig".to_string()))
                .required()
                .with_provider_name("VisibilityConfig"),
        )
        .with_validator(|attrs| {
            let mut errors = Vec::new();
            if let Err(mut e) = validate_tags_map(attrs) {
                errors.append(&mut e);
            }
            if errors.is_empty() { Ok(()) } else { Err(errors) }
        })
        .with_def("AWSManagedRulesACFPRuleSet", AttributeType::Struct {
                    name: "AWSManagedRulesACFPRuleSet".to_string(),
                    fields: vec![
                    StructField::new("creation_path", AttributeType::String).required().with_provider_name("CreationPath"),
                    StructField::new("enable_regex_in_path", AttributeType::Bool).with_provider_name("EnableRegexInPath"),
                    StructField::new("registration_page_path", AttributeType::String).required().with_provider_name("RegistrationPagePath"),
                    StructField::new("request_inspection", AttributeType::Struct {
                    name: "RequestInspectionACFP".to_string(),
                    fields: vec![
                    StructField::new("address_fields", AttributeType::list(AttributeType::String)).with_provider_name("AddressFields"),
                    StructField::new("email_field", AttributeType::Struct {
                    name: "FieldIdentifier".to_string(),
                    fields: vec![
                    StructField::new("identifier", AttributeType::Custom {
                identity: None,
                pattern: Some(".*\\S.*".to_string()),
                length: Some((Some(1), Some(512))),
                base: Box::new(AttributeType::String),
                validate: legacy_validator(validate_string_pattern_4e09c821aed9e752_len_1_512),
                to_dsl: None,
            }).required().with_provider_name("Identifier")
                    ],
                }).with_provider_name("EmailField"),
                    StructField::new("password_field", AttributeType::Ref("FieldIdentifier".to_string())).with_provider_name("PasswordField"),
                    StructField::new("payload_type", AttributeType::StringEnum {
                name: "PayloadType".to_string(),
                values: vec!["JSON".to_string(), "FORM_ENCODED".to_string()],
                identity: Some(carina_core::schema::string_enum_identity("PayloadType", Some("awscc.wafv2.WebAcl"))),
                dsl_aliases: vec![("JSON".to_string(), "json".to_string()), ("FORM_ENCODED".to_string(), "form_encoded".to_string())],
            }).required().with_provider_name("PayloadType"),
                    StructField::new("phone_number_fields", AttributeType::list(AttributeType::String)).with_provider_name("PhoneNumberFields"),
                    StructField::new("username_field", AttributeType::Ref("FieldIdentifier".to_string())).with_provider_name("UsernameField")
                    ],
                }).required().with_provider_name("RequestInspection"),
                    StructField::new("response_inspection", AttributeType::Struct {
                    name: "ResponseInspection".to_string(),
                    fields: vec![
                    StructField::new("body_contains", AttributeType::Struct {
                    name: "ResponseInspectionBodyContains".to_string(),
                    fields: vec![
                    StructField::new("failure_strings", AttributeType::Custom {
                identity: None,
                pattern: None,
                length: None,
                base: Box::new(AttributeType::list(AttributeType::Custom {
                identity: None,
                pattern: Some(".*\\S.*".to_string()),
                length: Some((Some(1), Some(100))),
                base: Box::new(AttributeType::String),
                validate: legacy_validator(validate_string_pattern_4e09c821aed9e752_len_1_100),
                to_dsl: None,
            })),
                validate: legacy_validator(validate_list_items_1_5),
                to_dsl: None,
            }).required().with_provider_name("FailureStrings"),
                    StructField::new("success_strings", AttributeType::Custom {
                identity: None,
                pattern: None,
                length: None,
                base: Box::new(AttributeType::list(AttributeType::Custom {
                identity: None,
                pattern: Some(".*\\S.*".to_string()),
                length: Some((Some(1), Some(100))),
                base: Box::new(AttributeType::String),
                validate: legacy_validator(validate_string_pattern_4e09c821aed9e752_len_1_100),
                to_dsl: None,
            })),
                validate: legacy_validator(validate_list_items_1_5),
                to_dsl: None,
            }).required().with_provider_name("SuccessStrings")
                    ],
                }).with_provider_name("BodyContains"),
                    StructField::new("header", AttributeType::Struct {
                    name: "ResponseInspectionHeader".to_string(),
                    fields: vec![
                    StructField::new("failure_values", AttributeType::Custom {
                identity: None,
                pattern: None,
                length: None,
                base: Box::new(AttributeType::list(AttributeType::Custom {
                identity: None,
                pattern: Some(".*\\S.*".to_string()),
                length: Some((Some(1), Some(100))),
                base: Box::new(AttributeType::String),
                validate: legacy_validator(validate_string_pattern_4e09c821aed9e752_len_1_100),
                to_dsl: None,
            })),
                validate: legacy_validator(validate_list_items_1_3),
                to_dsl: None,
            }).required().with_provider_name("FailureValues"),
                    StructField::new("name", AttributeType::Custom {
                identity: None,
                pattern: Some(".*\\S.*".to_string()),
                length: Some((Some(1), Some(200))),
                base: Box::new(AttributeType::String),
                validate: legacy_validator(validate_string_pattern_4e09c821aed9e752_len_1_200),
                to_dsl: None,
            }).required().with_provider_name("Name"),
                    StructField::new("success_values", AttributeType::Custom {
                identity: None,
                pattern: None,
                length: None,
                base: Box::new(AttributeType::list(AttributeType::Custom {
                identity: None,
                pattern: Some(".*\\S.*".to_string()),
                length: Some((Some(1), Some(100))),
                base: Box::new(AttributeType::String),
                validate: legacy_validator(validate_string_pattern_4e09c821aed9e752_len_1_100),
                to_dsl: None,
            })),
                validate: legacy_validator(validate_list_items_1_3),
                to_dsl: None,
            }).required().with_provider_name("SuccessValues")
                    ],
                }).with_provider_name("Header"),
                    StructField::new("json", AttributeType::Struct {
                    name: "ResponseInspectionJson".to_string(),
                    fields: vec![
                    StructField::new("failure_values", AttributeType::Custom {
                identity: None,
                pattern: None,
                length: None,
                base: Box::new(AttributeType::list(AttributeType::Custom {
                identity: None,
                pattern: Some(".*\\S.*".to_string()),
                length: Some((Some(1), Some(100))),
                base: Box::new(AttributeType::String),
                validate: legacy_validator(validate_string_pattern_4e09c821aed9e752_len_1_100),
                to_dsl: None,
            })),
                validate: legacy_validator(validate_list_items_1_5),
                to_dsl: None,
            }).required().with_provider_name("FailureValues"),
                    StructField::new("identifier", AttributeType::Custom {
                identity: None,
                pattern: Some(".*\\S.*".to_string()),
                length: Some((Some(1), Some(512))),
                base: Box::new(AttributeType::String),
                validate: legacy_validator(validate_string_pattern_4e09c821aed9e752_len_1_512),
                to_dsl: None,
            }).required().with_provider_name("Identifier"),
                    StructField::new("success_values", AttributeType::Custom {
                identity: None,
                pattern: None,
                length: None,
                base: Box::new(AttributeType::list(AttributeType::Custom {
                identity: None,
                pattern: Some(".*\\S.*".to_string()),
                length: Some((Some(1), Some(100))),
                base: Box::new(AttributeType::String),
                validate: legacy_validator(validate_string_pattern_4e09c821aed9e752_len_1_100),
                to_dsl: None,
            })),
                validate: legacy_validator(validate_list_items_1_5),
                to_dsl: None,
            }).required().with_provider_name("SuccessValues")
                    ],
                }).with_provider_name("Json"),
                    StructField::new("status_code", AttributeType::Struct {
                    name: "ResponseInspectionStatusCode".to_string(),
                    fields: vec![
                    StructField::new("failure_codes", AttributeType::Custom {
                identity: None,
                pattern: None,
                length: None,
                base: Box::new(AttributeType::list(AttributeType::Int)),
                validate: legacy_validator(validate_list_items_1_10),
                to_dsl: None,
            }).required().with_provider_name("FailureCodes"),
                    StructField::new("success_codes", AttributeType::Custom {
                identity: None,
                pattern: None,
                length: None,
                base: Box::new(AttributeType::list(AttributeType::Int)),
                validate: legacy_validator(validate_list_items_1_10),
                to_dsl: None,
            }).required().with_provider_name("SuccessCodes")
                    ],
                }).with_provider_name("StatusCode")
                    ],
                }).with_provider_name("ResponseInspection")
                    ],
                })
        .with_def("AWSManagedRulesATPRuleSet", AttributeType::Struct {
                    name: "AWSManagedRulesATPRuleSet".to_string(),
                    fields: vec![
                    StructField::new("enable_regex_in_path", AttributeType::Bool).with_provider_name("EnableRegexInPath"),
                    StructField::new("login_path", AttributeType::String).required().with_provider_name("LoginPath"),
                    StructField::new("request_inspection", AttributeType::Struct {
                    name: "RequestInspection".to_string(),
                    fields: vec![
                    StructField::new("password_field", AttributeType::Ref("FieldIdentifier".to_string())).required().with_provider_name("PasswordField"),
                    StructField::new("payload_type", AttributeType::StringEnum {
                name: "PayloadType".to_string(),
                values: vec!["JSON".to_string(), "FORM_ENCODED".to_string()],
                identity: Some(carina_core::schema::string_enum_identity("PayloadType", Some("awscc.wafv2.WebAcl"))),
                dsl_aliases: vec![("JSON".to_string(), "json".to_string()), ("FORM_ENCODED".to_string(), "form_encoded".to_string())],
            }).required().with_provider_name("PayloadType"),
                    StructField::new("username_field", AttributeType::Ref("FieldIdentifier".to_string())).required().with_provider_name("UsernameField")
                    ],
                }).with_provider_name("RequestInspection"),
                    StructField::new("response_inspection", AttributeType::Ref("ResponseInspection".to_string())).with_provider_name("ResponseInspection")
                    ],
                })
        .with_def("AWSManagedRulesAntiDDoSRuleSet", AttributeType::Struct {
                    name: "AWSManagedRulesAntiDDoSRuleSet".to_string(),
                    fields: vec![
                    StructField::new("client_side_action_config", AttributeType::Struct {
                    name: "ClientSideActionConfig".to_string(),
                    fields: vec![
                    StructField::new("challenge", AttributeType::Struct {
                    name: "ClientSideAction".to_string(),
                    fields: vec![
                    StructField::new("exempt_uri_regular_expressions", AttributeType::list(AttributeType::Struct {
                    name: "Regex".to_string(),
                    fields: vec![
                    StructField::new("regex_string", AttributeType::String).with_provider_name("RegexString")
                    ],
                })).with_provider_name("ExemptUriRegularExpressions").with_block_name("exempt_uri_regular_expression"),
                    StructField::new("sensitivity", AttributeType::StringEnum {
                name: "SensitivityToAct".to_string(),
                values: vec!["LOW".to_string(), "MEDIUM".to_string(), "HIGH".to_string()],
                identity: Some(carina_core::schema::string_enum_identity("SensitivityToAct", Some("awscc.wafv2.WebAcl"))),
                dsl_aliases: vec![("LOW".to_string(), "low".to_string()), ("MEDIUM".to_string(), "medium".to_string()), ("HIGH".to_string(), "high".to_string())],
            }).with_provider_name("Sensitivity"),
                    StructField::new("usage_of_action", AttributeType::StringEnum {
                name: "UsageOfAction".to_string(),
                values: vec!["ENABLED".to_string(), "DISABLED".to_string()],
                identity: Some(carina_core::schema::string_enum_identity("UsageOfAction", Some("awscc.wafv2.WebAcl"))),
                dsl_aliases: vec![("ENABLED".to_string(), "enabled".to_string()), ("DISABLED".to_string(), "disabled".to_string())],
            }).required().with_provider_name("UsageOfAction")
                    ],
                }).required().with_provider_name("Challenge").with_block_name("challenge")
                    ],
                }).required().with_provider_name("ClientSideActionConfig").with_block_name("client_side_action_config"),
                    StructField::new("sensitivity_to_block", AttributeType::StringEnum {
                name: "SensitivityToAct".to_string(),
                values: vec!["LOW".to_string(), "MEDIUM".to_string(), "HIGH".to_string()],
                identity: Some(carina_core::schema::string_enum_identity("SensitivityToAct", Some("awscc.wafv2.WebAcl"))),
                dsl_aliases: vec![("LOW".to_string(), "low".to_string()), ("MEDIUM".to_string(), "medium".to_string()), ("HIGH".to_string(), "high".to_string())],
            }).with_provider_name("SensitivityToBlock")
                    ],
                })
        .with_def("AWSManagedRulesBotControlRuleSet", AttributeType::Struct {
                    name: "AWSManagedRulesBotControlRuleSet".to_string(),
                    fields: vec![
                    StructField::new("enable_machine_learning", AttributeType::Bool).with_provider_name("EnableMachineLearning"),
                    StructField::new("inspection_level", AttributeType::StringEnum {
                name: "InspectionLevel".to_string(),
                values: vec!["COMMON".to_string(), "TARGETED".to_string()],
                identity: Some(carina_core::schema::string_enum_identity("InspectionLevel", Some("awscc.wafv2.WebAcl"))),
                dsl_aliases: vec![("COMMON".to_string(), "common".to_string()), ("TARGETED".to_string(), "targeted".to_string())],
            }).required().with_provider_name("InspectionLevel")
                    ],
                })
        .with_def("AllowAction", AttributeType::Struct {
                    name: "AllowAction".to_string(),
                    fields: vec![
                    StructField::new("custom_request_handling", AttributeType::Struct {
                    name: "CustomRequestHandling".to_string(),
                    fields: vec![
                    StructField::new("insert_headers", AttributeType::Custom {
                identity: None,
                pattern: None,
                length: None,
                base: Box::new(AttributeType::list(AttributeType::Struct {
                    name: "CustomHTTPHeader".to_string(),
                    fields: vec![
                    StructField::new("name", AttributeType::String).required().with_provider_name("Name"),
                    StructField::new("value", AttributeType::String).required().with_provider_name("Value")
                    ],
                })),
                validate: legacy_validator(validate_list_items_min_1),
                to_dsl: None,
            }).required().with_description("Collection of HTTP headers.").with_provider_name("InsertHeaders").with_block_name("insert_header")
                    ],
                }).with_provider_name("CustomRequestHandling").with_block_name("custom_request_handling")
                    ],
                })
        .with_def("AndStatement", AttributeType::Struct {
                    name: "AndStatement".to_string(),
                    fields: vec![
                    StructField::new("statements", AttributeType::list(AttributeType::Struct {
                    name: "Statement".to_string(),
                    fields: vec![
                    StructField::new("and_statement", AttributeType::Ref("AndStatement".to_string())).with_provider_name("AndStatement"),
                    StructField::new("asn_match_statement", AttributeType::Struct {
                    name: "AsnMatchStatement".to_string(),
                    fields: vec![
                    StructField::new("asn_list", AttributeType::list(AttributeType::Custom {
                identity: None,
                pattern: None,
                length: None,
                base: Box::new(AttributeType::Int),
                validate: legacy_validator(validate_asn_list_range),
                to_dsl: None,
            })).with_provider_name("AsnList"),
                    StructField::new("forwarded_ip_config", AttributeType::Struct {
                    name: "ForwardedIPConfiguration".to_string(),
                    fields: vec![
                    StructField::new("fallback_behavior", AttributeType::StringEnum {
                name: "FallbackBehavior".to_string(),
                values: vec!["MATCH".to_string(), "NO_MATCH".to_string()],
                identity: Some(carina_core::schema::string_enum_identity("FallbackBehavior", Some("awscc.wafv2.WebAcl"))),
                dsl_aliases: vec![("MATCH".to_string(), "match".to_string()), ("NO_MATCH".to_string(), "no_match".to_string())],
            }).required().with_provider_name("FallbackBehavior"),
                    StructField::new("header_name", AttributeType::Custom {
                identity: None,
                pattern: Some("^[a-zA-Z0-9-]+{1,255}$".to_string()),
                length: None,
                base: Box::new(AttributeType::String),
                validate: legacy_validator(validate_string_pattern_2d93cc844f6d4014),
                to_dsl: None,
            }).required().with_provider_name("HeaderName")
                    ],
                }).with_provider_name("ForwardedIPConfig")
                    ],
                }).with_provider_name("AsnMatchStatement"),
                    StructField::new("byte_match_statement", AttributeType::Struct {
                    name: "ByteMatchStatement".to_string(),
                    fields: vec![
                    StructField::new("field_to_match", AttributeType::Struct {
                    name: "FieldToMatch".to_string(),
                    fields: vec![
                    StructField::new("all_query_arguments", AttributeType::map(AttributeType::String)).with_description("All query arguments of a web request.").with_provider_name("AllQueryArguments"),
                    StructField::new("body", AttributeType::Struct {
                    name: "Body".to_string(),
                    fields: vec![
                    StructField::new("oversize_handling", AttributeType::StringEnum {
                name: "OversizeHandling".to_string(),
                values: vec!["CONTINUE".to_string(), "MATCH".to_string(), "NO_MATCH".to_string()],
                identity: Some(carina_core::schema::string_enum_identity("OversizeHandling", Some("awscc.wafv2.WebAcl"))),
                dsl_aliases: vec![("CONTINUE".to_string(), "continue".to_string()), ("MATCH".to_string(), "match".to_string()), ("NO_MATCH".to_string(), "no_match".to_string())],
            }).with_provider_name("OversizeHandling")
                    ],
                }).with_provider_name("Body"),
                    StructField::new("cookies", AttributeType::Struct {
                    name: "Cookies".to_string(),
                    fields: vec![
                    StructField::new("match_pattern", AttributeType::Struct {
                    name: "CookieMatchPattern".to_string(),
                    fields: vec![
                    StructField::new("all", AttributeType::map(AttributeType::String)).with_description("Inspect all parts of the web request cookies.").with_provider_name("All"),
                    StructField::new("excluded_cookies", AttributeType::Custom {
                identity: None,
                pattern: None,
                length: None,
                base: Box::new(AttributeType::list(AttributeType::Custom {
                identity: None,
                pattern: Some(".*\\S.*".to_string()),
                length: Some((Some(1), Some(60))),
                base: Box::new(AttributeType::String),
                validate: legacy_validator(validate_string_pattern_4e09c821aed9e752_len_1_60),
                to_dsl: None,
            })),
                validate: legacy_validator(validate_list_items_1_199),
                to_dsl: None,
            }).with_provider_name("ExcludedCookies"),
                    StructField::new("included_cookies", AttributeType::Custom {
                identity: None,
                pattern: None,
                length: None,
                base: Box::new(AttributeType::list(AttributeType::Custom {
                identity: None,
                pattern: Some(".*\\S.*".to_string()),
                length: Some((Some(1), Some(60))),
                base: Box::new(AttributeType::String),
                validate: legacy_validator(validate_string_pattern_4e09c821aed9e752_len_1_60),
                to_dsl: None,
            })),
                validate: legacy_validator(validate_list_items_1_199),
                to_dsl: None,
            }).with_provider_name("IncludedCookies")
                    ],
                }).required().with_provider_name("MatchPattern"),
                    StructField::new("match_scope", AttributeType::StringEnum {
                name: "MapMatchScope".to_string(),
                values: vec!["ALL".to_string(), "KEY".to_string(), "VALUE".to_string()],
                identity: Some(carina_core::schema::string_enum_identity("MapMatchScope", Some("awscc.wafv2.WebAcl"))),
                dsl_aliases: vec![("ALL".to_string(), "all".to_string()), ("KEY".to_string(), "key".to_string()), ("VALUE".to_string(), "value".to_string())],
            }).required().with_provider_name("MatchScope"),
                    StructField::new("oversize_handling", AttributeType::StringEnum {
                name: "OversizeHandling".to_string(),
                values: vec!["CONTINUE".to_string(), "MATCH".to_string(), "NO_MATCH".to_string()],
                identity: Some(carina_core::schema::string_enum_identity("OversizeHandling", Some("awscc.wafv2.WebAcl"))),
                dsl_aliases: vec![("CONTINUE".to_string(), "continue".to_string()), ("MATCH".to_string(), "match".to_string()), ("NO_MATCH".to_string(), "no_match".to_string())],
            }).required().with_provider_name("OversizeHandling")
                    ],
                }).with_provider_name("Cookies"),
                    StructField::new("header_order", AttributeType::Struct {
                    name: "HeaderOrder".to_string(),
                    fields: vec![
                    StructField::new("oversize_handling", AttributeType::StringEnum {
                name: "OversizeHandling".to_string(),
                values: vec!["CONTINUE".to_string(), "MATCH".to_string(), "NO_MATCH".to_string()],
                identity: Some(carina_core::schema::string_enum_identity("OversizeHandling", Some("awscc.wafv2.WebAcl"))),
                dsl_aliases: vec![("CONTINUE".to_string(), "continue".to_string()), ("MATCH".to_string(), "match".to_string()), ("NO_MATCH".to_string(), "no_match".to_string())],
            }).required().with_provider_name("OversizeHandling")
                    ],
                }).with_provider_name("HeaderOrder"),
                    StructField::new("headers", AttributeType::Struct {
                    name: "Headers".to_string(),
                    fields: vec![
                    StructField::new("match_pattern", AttributeType::Struct {
                    name: "HeaderMatchPattern".to_string(),
                    fields: vec![
                    StructField::new("all", AttributeType::map(AttributeType::String)).with_description("Inspect all parts of the web request headers.").with_provider_name("All"),
                    StructField::new("excluded_headers", AttributeType::Custom {
                identity: None,
                pattern: None,
                length: None,
                base: Box::new(AttributeType::list(AttributeType::Custom {
                identity: None,
                pattern: Some(".*\\S.*".to_string()),
                length: Some((Some(1), Some(64))),
                base: Box::new(AttributeType::String),
                validate: legacy_validator(validate_string_pattern_4e09c821aed9e752_len_1_64),
                to_dsl: None,
            })),
                validate: legacy_validator(validate_list_items_1_199),
                to_dsl: None,
            }).with_provider_name("ExcludedHeaders"),
                    StructField::new("included_headers", AttributeType::Custom {
                identity: None,
                pattern: None,
                length: None,
                base: Box::new(AttributeType::list(AttributeType::Custom {
                identity: None,
                pattern: Some(".*\\S.*".to_string()),
                length: Some((Some(1), Some(64))),
                base: Box::new(AttributeType::String),
                validate: legacy_validator(validate_string_pattern_4e09c821aed9e752_len_1_64),
                to_dsl: None,
            })),
                validate: legacy_validator(validate_list_items_1_199),
                to_dsl: None,
            }).with_provider_name("IncludedHeaders")
                    ],
                }).required().with_provider_name("MatchPattern"),
                    StructField::new("match_scope", AttributeType::StringEnum {
                name: "MapMatchScope".to_string(),
                values: vec!["ALL".to_string(), "KEY".to_string(), "VALUE".to_string()],
                identity: Some(carina_core::schema::string_enum_identity("MapMatchScope", Some("awscc.wafv2.WebAcl"))),
                dsl_aliases: vec![("ALL".to_string(), "all".to_string()), ("KEY".to_string(), "key".to_string()), ("VALUE".to_string(), "value".to_string())],
            }).required().with_provider_name("MatchScope"),
                    StructField::new("oversize_handling", AttributeType::StringEnum {
                name: "OversizeHandling".to_string(),
                values: vec!["CONTINUE".to_string(), "MATCH".to_string(), "NO_MATCH".to_string()],
                identity: Some(carina_core::schema::string_enum_identity("OversizeHandling", Some("awscc.wafv2.WebAcl"))),
                dsl_aliases: vec![("CONTINUE".to_string(), "continue".to_string()), ("MATCH".to_string(), "match".to_string()), ("NO_MATCH".to_string(), "no_match".to_string())],
            }).required().with_provider_name("OversizeHandling")
                    ],
                }).with_provider_name("Headers"),
                    StructField::new("ja3_fingerprint", AttributeType::Struct {
                    name: "JA3Fingerprint".to_string(),
                    fields: vec![
                    StructField::new("fallback_behavior", AttributeType::StringEnum {
                name: "FallbackBehavior".to_string(),
                values: vec!["MATCH".to_string(), "NO_MATCH".to_string()],
                identity: Some(carina_core::schema::string_enum_identity("FallbackBehavior", Some("awscc.wafv2.WebAcl"))),
                dsl_aliases: vec![("MATCH".to_string(), "match".to_string()), ("NO_MATCH".to_string(), "no_match".to_string())],
            }).required().with_provider_name("FallbackBehavior")
                    ],
                }).with_provider_name("JA3Fingerprint"),
                    StructField::new("ja4_fingerprint", AttributeType::Struct {
                    name: "JA4Fingerprint".to_string(),
                    fields: vec![
                    StructField::new("fallback_behavior", AttributeType::StringEnum {
                name: "FallbackBehavior".to_string(),
                values: vec!["MATCH".to_string(), "NO_MATCH".to_string()],
                identity: Some(carina_core::schema::string_enum_identity("FallbackBehavior", Some("awscc.wafv2.WebAcl"))),
                dsl_aliases: vec![("MATCH".to_string(), "match".to_string()), ("NO_MATCH".to_string(), "no_match".to_string())],
            }).required().with_provider_name("FallbackBehavior")
                    ],
                }).with_provider_name("JA4Fingerprint"),
                    StructField::new("json_body", AttributeType::Struct {
                    name: "JsonBody".to_string(),
                    fields: vec![
                    StructField::new("invalid_fallback_behavior", AttributeType::StringEnum {
                name: "BodyParsingFallbackBehavior".to_string(),
                values: vec!["MATCH".to_string(), "NO_MATCH".to_string(), "EVALUATE_AS_STRING".to_string()],
                identity: Some(carina_core::schema::string_enum_identity("BodyParsingFallbackBehavior", Some("awscc.wafv2.WebAcl"))),
                dsl_aliases: vec![("MATCH".to_string(), "match".to_string()), ("NO_MATCH".to_string(), "no_match".to_string()), ("EVALUATE_AS_STRING".to_string(), "evaluate_as_string".to_string())],
            }).with_provider_name("InvalidFallbackBehavior"),
                    StructField::new("match_pattern", AttributeType::Struct {
                    name: "JsonMatchPattern".to_string(),
                    fields: vec![
                    StructField::new("all", AttributeType::map(AttributeType::String)).with_description("Inspect all parts of the web request's JSON body.").with_provider_name("All"),
                    StructField::new("included_paths", AttributeType::list(AttributeType::Custom {
                identity: None,
                pattern: Some("^[\\/]+([^~]*(~[01])*)*{1,512}$".to_string()),
                length: None,
                base: Box::new(AttributeType::String),
                validate: legacy_validator(validate_string_pattern_c77cf75cf1a75ade),
                to_dsl: None,
            })).with_provider_name("IncludedPaths")
                    ],
                }).required().with_provider_name("MatchPattern"),
                    StructField::new("match_scope", AttributeType::StringEnum {
                name: "JsonMatchScope".to_string(),
                values: vec!["ALL".to_string(), "KEY".to_string(), "VALUE".to_string()],
                identity: Some(carina_core::schema::string_enum_identity("JsonMatchScope", Some("awscc.wafv2.WebAcl"))),
                dsl_aliases: vec![("ALL".to_string(), "all".to_string()), ("KEY".to_string(), "key".to_string()), ("VALUE".to_string(), "value".to_string())],
            }).required().with_provider_name("MatchScope"),
                    StructField::new("oversize_handling", AttributeType::StringEnum {
                name: "OversizeHandling".to_string(),
                values: vec!["CONTINUE".to_string(), "MATCH".to_string(), "NO_MATCH".to_string()],
                identity: Some(carina_core::schema::string_enum_identity("OversizeHandling", Some("awscc.wafv2.WebAcl"))),
                dsl_aliases: vec![("CONTINUE".to_string(), "continue".to_string()), ("MATCH".to_string(), "match".to_string()), ("NO_MATCH".to_string(), "no_match".to_string())],
            }).with_provider_name("OversizeHandling")
                    ],
                }).with_provider_name("JsonBody"),
                    StructField::new("method", AttributeType::map(AttributeType::String)).with_description("The HTTP method of a web request. The method indicates the type of operation that the request is asking the origin to perform.").with_provider_name("Method"),
                    StructField::new("query_string", AttributeType::map(AttributeType::String)).with_description("The query string of a web request. This is the part of a URL that appears after a ? character, if any.").with_provider_name("QueryString"),
                    StructField::new("single_header", AttributeType::Struct {
                    name: "SingleHeader".to_string(),
                    fields: vec![
                    StructField::new("name", AttributeType::String).required().with_provider_name("Name")
                    ],
                }).with_provider_name("SingleHeader"),
                    StructField::new("single_query_argument", AttributeType::Struct {
                    name: "SingleQueryArgument".to_string(),
                    fields: vec![
                    StructField::new("name", AttributeType::String).required().with_provider_name("Name")
                    ],
                }).with_description("One query argument in a web request, identified by name, for example UserName or SalesRegion. The name can be up to 30 characters long and isn't case sensitive.").with_provider_name("SingleQueryArgument"),
                    StructField::new("uri_fragment", AttributeType::Struct {
                    name: "UriFragment".to_string(),
                    fields: vec![
                    StructField::new("fallback_behavior", AttributeType::StringEnum {
                name: "FallbackBehavior".to_string(),
                values: vec!["MATCH".to_string(), "NO_MATCH".to_string()],
                identity: Some(carina_core::schema::string_enum_identity("FallbackBehavior", Some("awscc.wafv2.WebAcl"))),
                dsl_aliases: vec![("MATCH".to_string(), "match".to_string()), ("NO_MATCH".to_string(), "no_match".to_string())],
            }).with_provider_name("FallbackBehavior")
                    ],
                }).with_provider_name("UriFragment"),
                    StructField::new("uri_path", AttributeType::map(AttributeType::String)).with_description("The path component of the URI of a web request. This is the part of a web request that identifies a resource, for example, /images/daily-ad.jpg.").with_provider_name("UriPath")
                    ],
                }).required().with_provider_name("FieldToMatch"),
                    StructField::new("positional_constraint", AttributeType::StringEnum {
                name: "PositionalConstraint".to_string(),
                values: vec!["EXACTLY".to_string(), "STARTS_WITH".to_string(), "ENDS_WITH".to_string(), "CONTAINS".to_string(), "CONTAINS_WORD".to_string()],
                identity: Some(carina_core::schema::string_enum_identity("PositionalConstraint", Some("awscc.wafv2.WebAcl"))),
                dsl_aliases: vec![("EXACTLY".to_string(), "exactly".to_string()), ("STARTS_WITH".to_string(), "starts_with".to_string()), ("ENDS_WITH".to_string(), "ends_with".to_string()), ("CONTAINS".to_string(), "contains".to_string()), ("CONTAINS_WORD".to_string(), "contains_word".to_string())],
            }).required().with_provider_name("PositionalConstraint"),
                    StructField::new("search_string", AttributeType::String).with_provider_name("SearchString"),
                    StructField::new("search_string_base64", AttributeType::String).with_provider_name("SearchStringBase64"),
                    StructField::new("text_transformations", AttributeType::list(AttributeType::Struct {
                    name: "TextTransformation".to_string(),
                    fields: vec![
                    StructField::new("priority", AttributeType::String).required().with_provider_name("Priority"),
                    StructField::new("type", AttributeType::StringEnum {
                name: "TextTransformationType".to_string(),
                values: vec!["NONE".to_string(), "COMPRESS_WHITE_SPACE".to_string(), "HTML_ENTITY_DECODE".to_string(), "LOWERCASE".to_string(), "CMD_LINE".to_string(), "URL_DECODE".to_string(), "BASE64_DECODE".to_string(), "HEX_DECODE".to_string(), "MD5".to_string(), "REPLACE_COMMENTS".to_string(), "ESCAPE_SEQ_DECODE".to_string(), "SQL_HEX_DECODE".to_string(), "CSS_DECODE".to_string(), "JS_DECODE".to_string(), "NORMALIZE_PATH".to_string(), "NORMALIZE_PATH_WIN".to_string(), "REMOVE_NULLS".to_string(), "REPLACE_NULLS".to_string(), "BASE64_DECODE_EXT".to_string(), "URL_DECODE_UNI".to_string(), "UTF8_TO_UNICODE".to_string()],
                identity: Some(carina_core::schema::string_enum_identity("TextTransformationType", Some("awscc.wafv2.WebAcl"))),
                dsl_aliases: vec![("NONE".to_string(), "none".to_string()), ("COMPRESS_WHITE_SPACE".to_string(), "compress_white_space".to_string()), ("HTML_ENTITY_DECODE".to_string(), "html_entity_decode".to_string()), ("LOWERCASE".to_string(), "lowercase".to_string()), ("CMD_LINE".to_string(), "cmd_line".to_string()), ("URL_DECODE".to_string(), "url_decode".to_string()), ("BASE64_DECODE".to_string(), "base64_decode".to_string()), ("HEX_DECODE".to_string(), "hex_decode".to_string()), ("MD5".to_string(), "md5".to_string()), ("REPLACE_COMMENTS".to_string(), "replace_comments".to_string()), ("ESCAPE_SEQ_DECODE".to_string(), "escape_seq_decode".to_string()), ("SQL_HEX_DECODE".to_string(), "sql_hex_decode".to_string()), ("CSS_DECODE".to_string(), "css_decode".to_string()), ("JS_DECODE".to_string(), "js_decode".to_string()), ("NORMALIZE_PATH".to_string(), "normalize_path".to_string()), ("NORMALIZE_PATH_WIN".to_string(), "normalize_path_win".to_string()), ("REMOVE_NULLS".to_string(), "remove_nulls".to_string()), ("REPLACE_NULLS".to_string(), "replace_nulls".to_string()), ("BASE64_DECODE_EXT".to_string(), "base64_decode_ext".to_string()), ("URL_DECODE_UNI".to_string(), "url_decode_uni".to_string()), ("UTF8_TO_UNICODE".to_string(), "utf8_to_unicode".to_string())],
            }).required().with_provider_name("Type")
                    ],
                })).required().with_provider_name("TextTransformations").with_block_name("text_transformation")
                    ],
                }).with_provider_name("ByteMatchStatement").with_block_name("byte_match_statement"),
                    StructField::new("geo_match_statement", AttributeType::Struct {
                    name: "GeoMatchStatement".to_string(),
                    fields: vec![
                    StructField::new("country_codes", AttributeType::list(AttributeType::Custom {
                identity: None,
                pattern: None,
                length: Some((Some(1), Some(2))),
                base: Box::new(AttributeType::String),
                validate: legacy_validator(validate_string_length_1_2),
                to_dsl: None,
            })).with_provider_name("CountryCodes"),
                    StructField::new("forwarded_ip_config", AttributeType::Ref("ForwardedIPConfiguration".to_string())).with_provider_name("ForwardedIPConfig")
                    ],
                }).with_provider_name("GeoMatchStatement"),
                    StructField::new("ip_set_reference_statement", AttributeType::Struct {
                    name: "IPSetReferenceStatement".to_string(),
                    fields: vec![
                    StructField::new("arn", super::arn()).required().with_provider_name("Arn"),
                    StructField::new("ip_set_forwarded_ip_config", AttributeType::Struct {
                    name: "IPSetForwardedIPConfiguration".to_string(),
                    fields: vec![
                    StructField::new("fallback_behavior", AttributeType::StringEnum {
                name: "FallbackBehavior".to_string(),
                values: vec!["MATCH".to_string(), "NO_MATCH".to_string()],
                identity: Some(carina_core::schema::string_enum_identity("FallbackBehavior", Some("awscc.wafv2.WebAcl"))),
                dsl_aliases: vec![("MATCH".to_string(), "match".to_string()), ("NO_MATCH".to_string(), "no_match".to_string())],
            }).required().with_provider_name("FallbackBehavior"),
                    StructField::new("header_name", AttributeType::Custom {
                identity: None,
                pattern: Some("^[a-zA-Z0-9-]+{1,255}$".to_string()),
                length: None,
                base: Box::new(AttributeType::String),
                validate: legacy_validator(validate_string_pattern_2d93cc844f6d4014),
                to_dsl: None,
            }).required().with_provider_name("HeaderName"),
                    StructField::new("position", AttributeType::StringEnum {
                name: "Position".to_string(),
                values: vec!["FIRST".to_string(), "LAST".to_string(), "ANY".to_string()],
                identity: Some(carina_core::schema::string_enum_identity("Position", Some("awscc.wafv2.WebAcl"))),
                dsl_aliases: vec![("FIRST".to_string(), "first".to_string()), ("LAST".to_string(), "last".to_string()), ("ANY".to_string(), "any".to_string())],
            }).required().with_provider_name("Position")
                    ],
                }).with_provider_name("IPSetForwardedIPConfig")
                    ],
                }).with_provider_name("IPSetReferenceStatement"),
                    StructField::new("label_match_statement", AttributeType::Struct {
                    name: "LabelMatchStatement".to_string(),
                    fields: vec![
                    StructField::new("key", AttributeType::Custom {
                identity: None,
                pattern: Some("^[0-9A-Za-z_:-]{1,1024}$".to_string()),
                length: None,
                base: Box::new(AttributeType::String),
                validate: legacy_validator(validate_string_pattern_b3fc65b549fb77bd),
                to_dsl: None,
            }).required().with_provider_name("Key"),
                    StructField::new("scope", AttributeType::StringEnum {
                name: "LabelMatchScope".to_string(),
                values: vec!["LABEL".to_string(), "NAMESPACE".to_string()],
                identity: Some(carina_core::schema::string_enum_identity("LabelMatchScope", Some("awscc.wafv2.WebAcl"))),
                dsl_aliases: vec![("LABEL".to_string(), "label".to_string()), ("NAMESPACE".to_string(), "namespace".to_string())],
            }).required().with_provider_name("Scope")
                    ],
                }).with_provider_name("LabelMatchStatement"),
                    StructField::new("managed_rule_group_statement", AttributeType::Struct {
                    name: "ManagedRuleGroupStatement".to_string(),
                    fields: vec![
                    StructField::new("excluded_rules", AttributeType::list(AttributeType::Struct {
                    name: "ExcludedRule".to_string(),
                    fields: vec![
                    StructField::new("name", AttributeType::Custom {
                identity: None,
                pattern: Some("^[0-9A-Za-z_-]{1,128}$".to_string()),
                length: None,
                base: Box::new(AttributeType::String),
                validate: legacy_validator(validate_string_pattern_42f7eceb887966ad),
                to_dsl: None,
            }).required().with_provider_name("Name")
                    ],
                })).with_provider_name("ExcludedRules").with_block_name("excluded_rule"),
                    StructField::new("managed_rule_group_configs", AttributeType::list(AttributeType::Struct {
                    name: "ManagedRuleGroupConfig".to_string(),
                    fields: vec![
                    StructField::new("aws_managed_rules_acfp_rule_set", AttributeType::Struct {
                    name: "AWSManagedRulesACFPRuleSet".to_string(),
                    fields: vec![
                    StructField::new("creation_path", AttributeType::String).required().with_provider_name("CreationPath"),
                    StructField::new("enable_regex_in_path", AttributeType::Bool).with_provider_name("EnableRegexInPath"),
                    StructField::new("registration_page_path", AttributeType::String).required().with_provider_name("RegistrationPagePath"),
                    StructField::new("request_inspection", AttributeType::Struct {
                    name: "RequestInspectionACFP".to_string(),
                    fields: vec![
                    StructField::new("address_fields", AttributeType::list(AttributeType::String)).with_provider_name("AddressFields"),
                    StructField::new("email_field", AttributeType::Struct {
                    name: "FieldIdentifier".to_string(),
                    fields: vec![
                    StructField::new("identifier", AttributeType::Custom {
                identity: None,
                pattern: Some(".*\\S.*".to_string()),
                length: Some((Some(1), Some(512))),
                base: Box::new(AttributeType::String),
                validate: legacy_validator(validate_string_pattern_4e09c821aed9e752_len_1_512),
                to_dsl: None,
            }).required().with_provider_name("Identifier")
                    ],
                }).with_provider_name("EmailField"),
                    StructField::new("password_field", AttributeType::Ref("FieldIdentifier".to_string())).with_provider_name("PasswordField"),
                    StructField::new("payload_type", AttributeType::StringEnum {
                name: "PayloadType".to_string(),
                values: vec!["JSON".to_string(), "FORM_ENCODED".to_string()],
                identity: Some(carina_core::schema::string_enum_identity("PayloadType", Some("awscc.wafv2.WebAcl"))),
                dsl_aliases: vec![("JSON".to_string(), "json".to_string()), ("FORM_ENCODED".to_string(), "form_encoded".to_string())],
            }).required().with_provider_name("PayloadType"),
                    StructField::new("phone_number_fields", AttributeType::list(AttributeType::String)).with_provider_name("PhoneNumberFields"),
                    StructField::new("username_field", AttributeType::Ref("FieldIdentifier".to_string())).with_provider_name("UsernameField")
                    ],
                }).required().with_provider_name("RequestInspection"),
                    StructField::new("response_inspection", AttributeType::Struct {
                    name: "ResponseInspection".to_string(),
                    fields: vec![
                    StructField::new("body_contains", AttributeType::Struct {
                    name: "ResponseInspectionBodyContains".to_string(),
                    fields: vec![
                    StructField::new("failure_strings", AttributeType::Custom {
                identity: None,
                pattern: None,
                length: None,
                base: Box::new(AttributeType::list(AttributeType::Custom {
                identity: None,
                pattern: Some(".*\\S.*".to_string()),
                length: Some((Some(1), Some(100))),
                base: Box::new(AttributeType::String),
                validate: legacy_validator(validate_string_pattern_4e09c821aed9e752_len_1_100),
                to_dsl: None,
            })),
                validate: legacy_validator(validate_list_items_1_5),
                to_dsl: None,
            }).required().with_provider_name("FailureStrings"),
                    StructField::new("success_strings", AttributeType::Custom {
                identity: None,
                pattern: None,
                length: None,
                base: Box::new(AttributeType::list(AttributeType::Custom {
                identity: None,
                pattern: Some(".*\\S.*".to_string()),
                length: Some((Some(1), Some(100))),
                base: Box::new(AttributeType::String),
                validate: legacy_validator(validate_string_pattern_4e09c821aed9e752_len_1_100),
                to_dsl: None,
            })),
                validate: legacy_validator(validate_list_items_1_5),
                to_dsl: None,
            }).required().with_provider_name("SuccessStrings")
                    ],
                }).with_provider_name("BodyContains"),
                    StructField::new("header", AttributeType::Struct {
                    name: "ResponseInspectionHeader".to_string(),
                    fields: vec![
                    StructField::new("failure_values", AttributeType::Custom {
                identity: None,
                pattern: None,
                length: None,
                base: Box::new(AttributeType::list(AttributeType::Custom {
                identity: None,
                pattern: Some(".*\\S.*".to_string()),
                length: Some((Some(1), Some(100))),
                base: Box::new(AttributeType::String),
                validate: legacy_validator(validate_string_pattern_4e09c821aed9e752_len_1_100),
                to_dsl: None,
            })),
                validate: legacy_validator(validate_list_items_1_3),
                to_dsl: None,
            }).required().with_provider_name("FailureValues"),
                    StructField::new("name", AttributeType::Custom {
                identity: None,
                pattern: Some(".*\\S.*".to_string()),
                length: Some((Some(1), Some(200))),
                base: Box::new(AttributeType::String),
                validate: legacy_validator(validate_string_pattern_4e09c821aed9e752_len_1_200),
                to_dsl: None,
            }).required().with_provider_name("Name"),
                    StructField::new("success_values", AttributeType::Custom {
                identity: None,
                pattern: None,
                length: None,
                base: Box::new(AttributeType::list(AttributeType::Custom {
                identity: None,
                pattern: Some(".*\\S.*".to_string()),
                length: Some((Some(1), Some(100))),
                base: Box::new(AttributeType::String),
                validate: legacy_validator(validate_string_pattern_4e09c821aed9e752_len_1_100),
                to_dsl: None,
            })),
                validate: legacy_validator(validate_list_items_1_3),
                to_dsl: None,
            }).required().with_provider_name("SuccessValues")
                    ],
                }).with_provider_name("Header"),
                    StructField::new("json", AttributeType::Struct {
                    name: "ResponseInspectionJson".to_string(),
                    fields: vec![
                    StructField::new("failure_values", AttributeType::Custom {
                identity: None,
                pattern: None,
                length: None,
                base: Box::new(AttributeType::list(AttributeType::Custom {
                identity: None,
                pattern: Some(".*\\S.*".to_string()),
                length: Some((Some(1), Some(100))),
                base: Box::new(AttributeType::String),
                validate: legacy_validator(validate_string_pattern_4e09c821aed9e752_len_1_100),
                to_dsl: None,
            })),
                validate: legacy_validator(validate_list_items_1_5),
                to_dsl: None,
            }).required().with_provider_name("FailureValues"),
                    StructField::new("identifier", AttributeType::Custom {
                identity: None,
                pattern: Some(".*\\S.*".to_string()),
                length: Some((Some(1), Some(512))),
                base: Box::new(AttributeType::String),
                validate: legacy_validator(validate_string_pattern_4e09c821aed9e752_len_1_512),
                to_dsl: None,
            }).required().with_provider_name("Identifier"),
                    StructField::new("success_values", AttributeType::Custom {
                identity: None,
                pattern: None,
                length: None,
                base: Box::new(AttributeType::list(AttributeType::Custom {
                identity: None,
                pattern: Some(".*\\S.*".to_string()),
                length: Some((Some(1), Some(100))),
                base: Box::new(AttributeType::String),
                validate: legacy_validator(validate_string_pattern_4e09c821aed9e752_len_1_100),
                to_dsl: None,
            })),
                validate: legacy_validator(validate_list_items_1_5),
                to_dsl: None,
            }).required().with_provider_name("SuccessValues")
                    ],
                }).with_provider_name("Json"),
                    StructField::new("status_code", AttributeType::Struct {
                    name: "ResponseInspectionStatusCode".to_string(),
                    fields: vec![
                    StructField::new("failure_codes", AttributeType::Custom {
                identity: None,
                pattern: None,
                length: None,
                base: Box::new(AttributeType::list(AttributeType::Int)),
                validate: legacy_validator(validate_list_items_1_10),
                to_dsl: None,
            }).required().with_provider_name("FailureCodes"),
                    StructField::new("success_codes", AttributeType::Custom {
                identity: None,
                pattern: None,
                length: None,
                base: Box::new(AttributeType::list(AttributeType::Int)),
                validate: legacy_validator(validate_list_items_1_10),
                to_dsl: None,
            }).required().with_provider_name("SuccessCodes")
                    ],
                }).with_provider_name("StatusCode")
                    ],
                }).with_provider_name("ResponseInspection")
                    ],
                }).with_provider_name("AWSManagedRulesACFPRuleSet"),
                    StructField::new("aws_managed_rules_atp_rule_set", AttributeType::Struct {
                    name: "AWSManagedRulesATPRuleSet".to_string(),
                    fields: vec![
                    StructField::new("enable_regex_in_path", AttributeType::Bool).with_provider_name("EnableRegexInPath"),
                    StructField::new("login_path", AttributeType::String).required().with_provider_name("LoginPath"),
                    StructField::new("request_inspection", AttributeType::Struct {
                    name: "RequestInspection".to_string(),
                    fields: vec![
                    StructField::new("password_field", AttributeType::Ref("FieldIdentifier".to_string())).required().with_provider_name("PasswordField"),
                    StructField::new("payload_type", AttributeType::StringEnum {
                name: "PayloadType".to_string(),
                values: vec!["JSON".to_string(), "FORM_ENCODED".to_string()],
                identity: Some(carina_core::schema::string_enum_identity("PayloadType", Some("awscc.wafv2.WebAcl"))),
                dsl_aliases: vec![("JSON".to_string(), "json".to_string()), ("FORM_ENCODED".to_string(), "form_encoded".to_string())],
            }).required().with_provider_name("PayloadType"),
                    StructField::new("username_field", AttributeType::Ref("FieldIdentifier".to_string())).required().with_provider_name("UsernameField")
                    ],
                }).with_provider_name("RequestInspection"),
                    StructField::new("response_inspection", AttributeType::Ref("ResponseInspection".to_string())).with_provider_name("ResponseInspection")
                    ],
                }).with_provider_name("AWSManagedRulesATPRuleSet"),
                    StructField::new("aws_managed_rules_anti_d_do_s_rule_set", AttributeType::Struct {
                    name: "AWSManagedRulesAntiDDoSRuleSet".to_string(),
                    fields: vec![
                    StructField::new("client_side_action_config", AttributeType::Struct {
                    name: "ClientSideActionConfig".to_string(),
                    fields: vec![
                    StructField::new("challenge", AttributeType::Struct {
                    name: "ClientSideAction".to_string(),
                    fields: vec![
                    StructField::new("exempt_uri_regular_expressions", AttributeType::list(AttributeType::Struct {
                    name: "Regex".to_string(),
                    fields: vec![
                    StructField::new("regex_string", AttributeType::String).with_provider_name("RegexString")
                    ],
                })).with_provider_name("ExemptUriRegularExpressions").with_block_name("exempt_uri_regular_expression"),
                    StructField::new("sensitivity", AttributeType::StringEnum {
                name: "SensitivityToAct".to_string(),
                values: vec!["LOW".to_string(), "MEDIUM".to_string(), "HIGH".to_string()],
                identity: Some(carina_core::schema::string_enum_identity("SensitivityToAct", Some("awscc.wafv2.WebAcl"))),
                dsl_aliases: vec![("LOW".to_string(), "low".to_string()), ("MEDIUM".to_string(), "medium".to_string()), ("HIGH".to_string(), "high".to_string())],
            }).with_provider_name("Sensitivity"),
                    StructField::new("usage_of_action", AttributeType::StringEnum {
                name: "UsageOfAction".to_string(),
                values: vec!["ENABLED".to_string(), "DISABLED".to_string()],
                identity: Some(carina_core::schema::string_enum_identity("UsageOfAction", Some("awscc.wafv2.WebAcl"))),
                dsl_aliases: vec![("ENABLED".to_string(), "enabled".to_string()), ("DISABLED".to_string(), "disabled".to_string())],
            }).required().with_provider_name("UsageOfAction")
                    ],
                }).required().with_provider_name("Challenge").with_block_name("challenge")
                    ],
                }).required().with_provider_name("ClientSideActionConfig").with_block_name("client_side_action_config"),
                    StructField::new("sensitivity_to_block", AttributeType::StringEnum {
                name: "SensitivityToAct".to_string(),
                values: vec!["LOW".to_string(), "MEDIUM".to_string(), "HIGH".to_string()],
                identity: Some(carina_core::schema::string_enum_identity("SensitivityToAct", Some("awscc.wafv2.WebAcl"))),
                dsl_aliases: vec![("LOW".to_string(), "low".to_string()), ("MEDIUM".to_string(), "medium".to_string()), ("HIGH".to_string(), "high".to_string())],
            }).with_provider_name("SensitivityToBlock")
                    ],
                }).with_provider_name("AWSManagedRulesAntiDDoSRuleSet").with_block_name("aws_managed_rules_anti_d_do_s_rule_set"),
                    StructField::new("aws_managed_rules_bot_control_rule_set", AttributeType::Struct {
                    name: "AWSManagedRulesBotControlRuleSet".to_string(),
                    fields: vec![
                    StructField::new("enable_machine_learning", AttributeType::Bool).with_provider_name("EnableMachineLearning"),
                    StructField::new("inspection_level", AttributeType::StringEnum {
                name: "InspectionLevel".to_string(),
                values: vec!["COMMON".to_string(), "TARGETED".to_string()],
                identity: Some(carina_core::schema::string_enum_identity("InspectionLevel", Some("awscc.wafv2.WebAcl"))),
                dsl_aliases: vec![("COMMON".to_string(), "common".to_string()), ("TARGETED".to_string(), "targeted".to_string())],
            }).required().with_provider_name("InspectionLevel")
                    ],
                }).with_provider_name("AWSManagedRulesBotControlRuleSet"),
                    StructField::new("login_path", AttributeType::Custom {
                identity: None,
                pattern: Some(".*\\S.*".to_string()),
                length: Some((Some(1), Some(256))),
                base: Box::new(AttributeType::String),
                validate: legacy_validator(validate_string_pattern_4e09c821aed9e752_len_1_256),
                to_dsl: None,
            }).with_provider_name("LoginPath"),
                    StructField::new("password_field", AttributeType::Ref("FieldIdentifier".to_string())).with_provider_name("PasswordField"),
                    StructField::new("payload_type", AttributeType::StringEnum {
                name: "PayloadType".to_string(),
                values: vec!["JSON".to_string(), "FORM_ENCODED".to_string()],
                identity: Some(carina_core::schema::string_enum_identity("PayloadType", Some("awscc.wafv2.WebAcl"))),
                dsl_aliases: vec![("JSON".to_string(), "json".to_string()), ("FORM_ENCODED".to_string(), "form_encoded".to_string())],
            }).with_provider_name("PayloadType"),
                    StructField::new("username_field", AttributeType::Ref("FieldIdentifier".to_string())).with_provider_name("UsernameField")
                    ],
                })).with_description("Collection of ManagedRuleGroupConfig.").with_provider_name("ManagedRuleGroupConfigs").with_block_name("managed_rule_group_config"),
                    StructField::new("name", AttributeType::Custom {
                identity: None,
                pattern: Some("^[0-9A-Za-z_-]{1,128}$".to_string()),
                length: None,
                base: Box::new(AttributeType::String),
                validate: legacy_validator(validate_string_pattern_42f7eceb887966ad),
                to_dsl: None,
            }).required().with_provider_name("Name"),
                    StructField::new("rule_action_overrides", AttributeType::Custom {
                identity: None,
                pattern: None,
                length: None,
                base: Box::new(AttributeType::list(AttributeType::Struct {
                    name: "RuleActionOverride".to_string(),
                    fields: vec![
                    StructField::new("action_to_use", AttributeType::Ref("RuleAction".to_string())).required().with_provider_name("ActionToUse"),
                    StructField::new("name", AttributeType::Custom {
                identity: None,
                pattern: Some("^[0-9A-Za-z_-]{1,128}$".to_string()),
                length: None,
                base: Box::new(AttributeType::String),
                validate: legacy_validator(validate_string_pattern_42f7eceb887966ad),
                to_dsl: None,
            }).required().with_provider_name("Name")
                    ],
                })),
                validate: legacy_validator(validate_list_items_max_100),
                to_dsl: None,
            }).with_description("Action overrides for rules in the rule group.").with_provider_name("RuleActionOverrides").with_block_name("rule_action_override"),
                    StructField::new("scope_down_statement", AttributeType::Ref("Statement".to_string())).with_provider_name("ScopeDownStatement"),
                    StructField::new("vendor_name", AttributeType::String).required().with_provider_name("VendorName"),
                    StructField::new("version", AttributeType::Custom {
                identity: None,
                pattern: Some("^[\\w#:\\.\\-/]+$".to_string()),
                length: Some((Some(1), Some(64))),
                base: Box::new(AttributeType::String),
                validate: legacy_validator(validate_string_pattern_82a037d29be8d222_len_1_64),
                to_dsl: None,
            }).with_provider_name("Version")
                    ],
                }).with_provider_name("ManagedRuleGroupStatement").with_block_name("managed_rule_group_statement"),
                    StructField::new("not_statement", AttributeType::Struct {
                    name: "NotStatement".to_string(),
                    fields: vec![
                    StructField::new("statement", AttributeType::Ref("Statement".to_string())).required().with_provider_name("Statement")
                    ],
                }).with_provider_name("NotStatement"),
                    StructField::new("or_statement", AttributeType::Struct {
                    name: "OrStatement".to_string(),
                    fields: vec![
                    StructField::new("statements", AttributeType::list(AttributeType::Struct {
                    name: "Statement".to_string(),
                    fields: vec![
                    StructField::new("and_statement", AttributeType::Ref("AndStatement".to_string())).with_provider_name("AndStatement"),
                    StructField::new("asn_match_statement", AttributeType::Ref("AsnMatchStatement".to_string())).with_provider_name("AsnMatchStatement"),
                    StructField::new("byte_match_statement", AttributeType::Ref("ByteMatchStatement".to_string())).with_provider_name("ByteMatchStatement"),
                    StructField::new("geo_match_statement", AttributeType::Ref("GeoMatchStatement".to_string())).with_provider_name("GeoMatchStatement"),
                    StructField::new("ip_set_reference_statement", AttributeType::Ref("IPSetReferenceStatement".to_string())).with_provider_name("IPSetReferenceStatement"),
                    StructField::new("label_match_statement", AttributeType::Ref("LabelMatchStatement".to_string())).with_provider_name("LabelMatchStatement"),
                    StructField::new("managed_rule_group_statement", AttributeType::Ref("ManagedRuleGroupStatement".to_string())).with_provider_name("ManagedRuleGroupStatement"),
                    StructField::new("not_statement", AttributeType::Ref("NotStatement".to_string())).with_provider_name("NotStatement"),
                    StructField::new("or_statement", AttributeType::Ref("OrStatement".to_string())).with_provider_name("OrStatement"),
                    StructField::new("rate_based_statement", AttributeType::Struct {
                    name: "RateBasedStatement".to_string(),
                    fields: vec![
                    StructField::new("aggregate_key_type", AttributeType::StringEnum {
                name: "AggregateKeyType".to_string(),
                values: vec!["CONSTANT".to_string(), "IP".to_string(), "FORWARDED_IP".to_string(), "CUSTOM_KEYS".to_string()],
                identity: Some(carina_core::schema::string_enum_identity("AggregateKeyType", Some("awscc.wafv2.WebAcl"))),
                dsl_aliases: vec![("CONSTANT".to_string(), "constant".to_string()), ("IP".to_string(), "ip".to_string()), ("FORWARDED_IP".to_string(), "forwarded_ip".to_string()), ("CUSTOM_KEYS".to_string(), "custom_keys".to_string())],
            }).required().with_provider_name("AggregateKeyType"),
                    StructField::new("custom_keys", AttributeType::Custom {
                identity: None,
                pattern: None,
                length: None,
                base: Box::new(AttributeType::list(AttributeType::Struct {
                    name: "RateBasedStatementCustomKey".to_string(),
                    fields: vec![
                    StructField::new("asn", AttributeType::String).with_provider_name("ASN"),
                    StructField::new("cookie", AttributeType::Struct {
                    name: "RateLimitCookie".to_string(),
                    fields: vec![
                    StructField::new("name", AttributeType::Custom {
                identity: None,
                pattern: Some(".*\\S.*".to_string()),
                length: Some((Some(1), Some(64))),
                base: Box::new(AttributeType::String),
                validate: legacy_validator(validate_string_pattern_4e09c821aed9e752_len_1_64),
                to_dsl: None,
            }).required().with_description("The name of the cookie to use.").with_provider_name("Name"),
                    StructField::new("text_transformations", AttributeType::list(AttributeType::Struct {
                    name: "TextTransformation".to_string(),
                    fields: vec![
                    StructField::new("priority", AttributeType::String).required().with_provider_name("Priority"),
                    StructField::new("type", AttributeType::StringEnum {
                name: "TextTransformationType".to_string(),
                values: vec!["NONE".to_string(), "COMPRESS_WHITE_SPACE".to_string(), "HTML_ENTITY_DECODE".to_string(), "LOWERCASE".to_string(), "CMD_LINE".to_string(), "URL_DECODE".to_string(), "BASE64_DECODE".to_string(), "HEX_DECODE".to_string(), "MD5".to_string(), "REPLACE_COMMENTS".to_string(), "ESCAPE_SEQ_DECODE".to_string(), "SQL_HEX_DECODE".to_string(), "CSS_DECODE".to_string(), "JS_DECODE".to_string(), "NORMALIZE_PATH".to_string(), "NORMALIZE_PATH_WIN".to_string(), "REMOVE_NULLS".to_string(), "REPLACE_NULLS".to_string(), "BASE64_DECODE_EXT".to_string(), "URL_DECODE_UNI".to_string(), "UTF8_TO_UNICODE".to_string()],
                identity: Some(carina_core::schema::string_enum_identity("TextTransformationType", Some("awscc.wafv2.WebAcl"))),
                dsl_aliases: vec![("NONE".to_string(), "none".to_string()), ("COMPRESS_WHITE_SPACE".to_string(), "compress_white_space".to_string()), ("HTML_ENTITY_DECODE".to_string(), "html_entity_decode".to_string()), ("LOWERCASE".to_string(), "lowercase".to_string()), ("CMD_LINE".to_string(), "cmd_line".to_string()), ("URL_DECODE".to_string(), "url_decode".to_string()), ("BASE64_DECODE".to_string(), "base64_decode".to_string()), ("HEX_DECODE".to_string(), "hex_decode".to_string()), ("MD5".to_string(), "md5".to_string()), ("REPLACE_COMMENTS".to_string(), "replace_comments".to_string()), ("ESCAPE_SEQ_DECODE".to_string(), "escape_seq_decode".to_string()), ("SQL_HEX_DECODE".to_string(), "sql_hex_decode".to_string()), ("CSS_DECODE".to_string(), "css_decode".to_string()), ("JS_DECODE".to_string(), "js_decode".to_string()), ("NORMALIZE_PATH".to_string(), "normalize_path".to_string()), ("NORMALIZE_PATH_WIN".to_string(), "normalize_path_win".to_string()), ("REMOVE_NULLS".to_string(), "remove_nulls".to_string()), ("REPLACE_NULLS".to_string(), "replace_nulls".to_string()), ("BASE64_DECODE_EXT".to_string(), "base64_decode_ext".to_string()), ("URL_DECODE_UNI".to_string(), "url_decode_uni".to_string()), ("UTF8_TO_UNICODE".to_string(), "utf8_to_unicode".to_string())],
            }).required().with_provider_name("Type")
                    ],
                })).required().with_provider_name("TextTransformations").with_block_name("text_transformation")
                    ],
                }).with_provider_name("Cookie").with_block_name("cookie"),
                    StructField::new("forwarded_ip", types::ipv4_address()).with_provider_name("ForwardedIP"),
                    StructField::new("http_method", AttributeType::String).with_provider_name("HTTPMethod"),
                    StructField::new("header", AttributeType::Struct {
                    name: "RateLimitHeader".to_string(),
                    fields: vec![
                    StructField::new("name", AttributeType::Custom {
                identity: None,
                pattern: Some(".*\\S.*".to_string()),
                length: Some((Some(1), Some(64))),
                base: Box::new(AttributeType::String),
                validate: legacy_validator(validate_string_pattern_4e09c821aed9e752_len_1_64),
                to_dsl: None,
            }).required().with_description("The name of the header to use.").with_provider_name("Name"),
                    StructField::new("text_transformations", AttributeType::list(AttributeType::Struct {
                    name: "TextTransformation".to_string(),
                    fields: vec![
                    StructField::new("priority", AttributeType::String).required().with_provider_name("Priority"),
                    StructField::new("type", AttributeType::StringEnum {
                name: "TextTransformationType".to_string(),
                values: vec!["NONE".to_string(), "COMPRESS_WHITE_SPACE".to_string(), "HTML_ENTITY_DECODE".to_string(), "LOWERCASE".to_string(), "CMD_LINE".to_string(), "URL_DECODE".to_string(), "BASE64_DECODE".to_string(), "HEX_DECODE".to_string(), "MD5".to_string(), "REPLACE_COMMENTS".to_string(), "ESCAPE_SEQ_DECODE".to_string(), "SQL_HEX_DECODE".to_string(), "CSS_DECODE".to_string(), "JS_DECODE".to_string(), "NORMALIZE_PATH".to_string(), "NORMALIZE_PATH_WIN".to_string(), "REMOVE_NULLS".to_string(), "REPLACE_NULLS".to_string(), "BASE64_DECODE_EXT".to_string(), "URL_DECODE_UNI".to_string(), "UTF8_TO_UNICODE".to_string()],
                identity: Some(carina_core::schema::string_enum_identity("TextTransformationType", Some("awscc.wafv2.WebAcl"))),
                dsl_aliases: vec![("NONE".to_string(), "none".to_string()), ("COMPRESS_WHITE_SPACE".to_string(), "compress_white_space".to_string()), ("HTML_ENTITY_DECODE".to_string(), "html_entity_decode".to_string()), ("LOWERCASE".to_string(), "lowercase".to_string()), ("CMD_LINE".to_string(), "cmd_line".to_string()), ("URL_DECODE".to_string(), "url_decode".to_string()), ("BASE64_DECODE".to_string(), "base64_decode".to_string()), ("HEX_DECODE".to_string(), "hex_decode".to_string()), ("MD5".to_string(), "md5".to_string()), ("REPLACE_COMMENTS".to_string(), "replace_comments".to_string()), ("ESCAPE_SEQ_DECODE".to_string(), "escape_seq_decode".to_string()), ("SQL_HEX_DECODE".to_string(), "sql_hex_decode".to_string()), ("CSS_DECODE".to_string(), "css_decode".to_string()), ("JS_DECODE".to_string(), "js_decode".to_string()), ("NORMALIZE_PATH".to_string(), "normalize_path".to_string()), ("NORMALIZE_PATH_WIN".to_string(), "normalize_path_win".to_string()), ("REMOVE_NULLS".to_string(), "remove_nulls".to_string()), ("REPLACE_NULLS".to_string(), "replace_nulls".to_string()), ("BASE64_DECODE_EXT".to_string(), "base64_decode_ext".to_string()), ("URL_DECODE_UNI".to_string(), "url_decode_uni".to_string()), ("UTF8_TO_UNICODE".to_string(), "utf8_to_unicode".to_string())],
            }).required().with_provider_name("Type")
                    ],
                })).required().with_provider_name("TextTransformations").with_block_name("text_transformation")
                    ],
                }).with_provider_name("Header").with_block_name("header"),
                    StructField::new("ip", types::ipv4_address()).with_provider_name("IP"),
                    StructField::new("ja3_fingerprint", AttributeType::Struct {
                    name: "RateLimitJA3Fingerprint".to_string(),
                    fields: vec![
                    StructField::new("fallback_behavior", AttributeType::StringEnum {
                name: "FallbackBehavior".to_string(),
                values: vec!["MATCH".to_string(), "NO_MATCH".to_string()],
                identity: Some(carina_core::schema::string_enum_identity("FallbackBehavior", Some("awscc.wafv2.WebAcl"))),
                dsl_aliases: vec![("MATCH".to_string(), "match".to_string()), ("NO_MATCH".to_string(), "no_match".to_string())],
            }).required().with_provider_name("FallbackBehavior")
                    ],
                }).with_provider_name("JA3Fingerprint"),
                    StructField::new("ja4_fingerprint", AttributeType::Struct {
                    name: "RateLimitJA4Fingerprint".to_string(),
                    fields: vec![
                    StructField::new("fallback_behavior", AttributeType::StringEnum {
                name: "FallbackBehavior".to_string(),
                values: vec!["MATCH".to_string(), "NO_MATCH".to_string()],
                identity: Some(carina_core::schema::string_enum_identity("FallbackBehavior", Some("awscc.wafv2.WebAcl"))),
                dsl_aliases: vec![("MATCH".to_string(), "match".to_string()), ("NO_MATCH".to_string(), "no_match".to_string())],
            }).required().with_provider_name("FallbackBehavior")
                    ],
                }).with_provider_name("JA4Fingerprint"),
                    StructField::new("label_namespace", AttributeType::Struct {
                    name: "RateLimitLabelNamespace".to_string(),
                    fields: vec![
                    StructField::new("namespace", AttributeType::Custom {
                identity: None,
                pattern: Some("^[0-9A-Za-z_:-]{1,1024}$".to_string()),
                length: None,
                base: Box::new(AttributeType::String),
                validate: legacy_validator(validate_string_pattern_b3fc65b549fb77bd),
                to_dsl: None,
            }).required().with_description("The namespace to use for aggregation.").with_provider_name("Namespace")
                    ],
                }).with_provider_name("LabelNamespace"),
                    StructField::new("query_argument", AttributeType::Struct {
                    name: "RateLimitQueryArgument".to_string(),
                    fields: vec![
                    StructField::new("name", AttributeType::Custom {
                identity: None,
                pattern: Some(".*\\S.*".to_string()),
                length: Some((Some(1), Some(64))),
                base: Box::new(AttributeType::String),
                validate: legacy_validator(validate_string_pattern_4e09c821aed9e752_len_1_64),
                to_dsl: None,
            }).required().with_description("The name of the query argument to use.").with_provider_name("Name"),
                    StructField::new("text_transformations", AttributeType::list(AttributeType::Struct {
                    name: "TextTransformation".to_string(),
                    fields: vec![
                    StructField::new("priority", AttributeType::String).required().with_provider_name("Priority"),
                    StructField::new("type", AttributeType::StringEnum {
                name: "TextTransformationType".to_string(),
                values: vec!["NONE".to_string(), "COMPRESS_WHITE_SPACE".to_string(), "HTML_ENTITY_DECODE".to_string(), "LOWERCASE".to_string(), "CMD_LINE".to_string(), "URL_DECODE".to_string(), "BASE64_DECODE".to_string(), "HEX_DECODE".to_string(), "MD5".to_string(), "REPLACE_COMMENTS".to_string(), "ESCAPE_SEQ_DECODE".to_string(), "SQL_HEX_DECODE".to_string(), "CSS_DECODE".to_string(), "JS_DECODE".to_string(), "NORMALIZE_PATH".to_string(), "NORMALIZE_PATH_WIN".to_string(), "REMOVE_NULLS".to_string(), "REPLACE_NULLS".to_string(), "BASE64_DECODE_EXT".to_string(), "URL_DECODE_UNI".to_string(), "UTF8_TO_UNICODE".to_string()],
                identity: Some(carina_core::schema::string_enum_identity("TextTransformationType", Some("awscc.wafv2.WebAcl"))),
                dsl_aliases: vec![("NONE".to_string(), "none".to_string()), ("COMPRESS_WHITE_SPACE".to_string(), "compress_white_space".to_string()), ("HTML_ENTITY_DECODE".to_string(), "html_entity_decode".to_string()), ("LOWERCASE".to_string(), "lowercase".to_string()), ("CMD_LINE".to_string(), "cmd_line".to_string()), ("URL_DECODE".to_string(), "url_decode".to_string()), ("BASE64_DECODE".to_string(), "base64_decode".to_string()), ("HEX_DECODE".to_string(), "hex_decode".to_string()), ("MD5".to_string(), "md5".to_string()), ("REPLACE_COMMENTS".to_string(), "replace_comments".to_string()), ("ESCAPE_SEQ_DECODE".to_string(), "escape_seq_decode".to_string()), ("SQL_HEX_DECODE".to_string(), "sql_hex_decode".to_string()), ("CSS_DECODE".to_string(), "css_decode".to_string()), ("JS_DECODE".to_string(), "js_decode".to_string()), ("NORMALIZE_PATH".to_string(), "normalize_path".to_string()), ("NORMALIZE_PATH_WIN".to_string(), "normalize_path_win".to_string()), ("REMOVE_NULLS".to_string(), "remove_nulls".to_string()), ("REPLACE_NULLS".to_string(), "replace_nulls".to_string()), ("BASE64_DECODE_EXT".to_string(), "base64_decode_ext".to_string()), ("URL_DECODE_UNI".to_string(), "url_decode_uni".to_string()), ("UTF8_TO_UNICODE".to_string(), "utf8_to_unicode".to_string())],
            }).required().with_provider_name("Type")
                    ],
                })).required().with_provider_name("TextTransformations").with_block_name("text_transformation")
                    ],
                }).with_provider_name("QueryArgument").with_block_name("query_argument"),
                    StructField::new("query_string", AttributeType::Struct {
                    name: "RateLimitQueryString".to_string(),
                    fields: vec![
                    StructField::new("text_transformations", AttributeType::list(AttributeType::Struct {
                    name: "TextTransformation".to_string(),
                    fields: vec![
                    StructField::new("priority", AttributeType::String).required().with_provider_name("Priority"),
                    StructField::new("type", AttributeType::StringEnum {
                name: "TextTransformationType".to_string(),
                values: vec!["NONE".to_string(), "COMPRESS_WHITE_SPACE".to_string(), "HTML_ENTITY_DECODE".to_string(), "LOWERCASE".to_string(), "CMD_LINE".to_string(), "URL_DECODE".to_string(), "BASE64_DECODE".to_string(), "HEX_DECODE".to_string(), "MD5".to_string(), "REPLACE_COMMENTS".to_string(), "ESCAPE_SEQ_DECODE".to_string(), "SQL_HEX_DECODE".to_string(), "CSS_DECODE".to_string(), "JS_DECODE".to_string(), "NORMALIZE_PATH".to_string(), "NORMALIZE_PATH_WIN".to_string(), "REMOVE_NULLS".to_string(), "REPLACE_NULLS".to_string(), "BASE64_DECODE_EXT".to_string(), "URL_DECODE_UNI".to_string(), "UTF8_TO_UNICODE".to_string()],
                identity: Some(carina_core::schema::string_enum_identity("TextTransformationType", Some("awscc.wafv2.WebAcl"))),
                dsl_aliases: vec![("NONE".to_string(), "none".to_string()), ("COMPRESS_WHITE_SPACE".to_string(), "compress_white_space".to_string()), ("HTML_ENTITY_DECODE".to_string(), "html_entity_decode".to_string()), ("LOWERCASE".to_string(), "lowercase".to_string()), ("CMD_LINE".to_string(), "cmd_line".to_string()), ("URL_DECODE".to_string(), "url_decode".to_string()), ("BASE64_DECODE".to_string(), "base64_decode".to_string()), ("HEX_DECODE".to_string(), "hex_decode".to_string()), ("MD5".to_string(), "md5".to_string()), ("REPLACE_COMMENTS".to_string(), "replace_comments".to_string()), ("ESCAPE_SEQ_DECODE".to_string(), "escape_seq_decode".to_string()), ("SQL_HEX_DECODE".to_string(), "sql_hex_decode".to_string()), ("CSS_DECODE".to_string(), "css_decode".to_string()), ("JS_DECODE".to_string(), "js_decode".to_string()), ("NORMALIZE_PATH".to_string(), "normalize_path".to_string()), ("NORMALIZE_PATH_WIN".to_string(), "normalize_path_win".to_string()), ("REMOVE_NULLS".to_string(), "remove_nulls".to_string()), ("REPLACE_NULLS".to_string(), "replace_nulls".to_string()), ("BASE64_DECODE_EXT".to_string(), "base64_decode_ext".to_string()), ("URL_DECODE_UNI".to_string(), "url_decode_uni".to_string()), ("UTF8_TO_UNICODE".to_string(), "utf8_to_unicode".to_string())],
            }).required().with_provider_name("Type")
                    ],
                })).required().with_provider_name("TextTransformations").with_block_name("text_transformation")
                    ],
                }).with_provider_name("QueryString").with_block_name("query_string"),
                    StructField::new("uri_path", AttributeType::Struct {
                    name: "RateLimitUriPath".to_string(),
                    fields: vec![
                    StructField::new("text_transformations", AttributeType::list(AttributeType::Struct {
                    name: "TextTransformation".to_string(),
                    fields: vec![
                    StructField::new("priority", AttributeType::String).required().with_provider_name("Priority"),
                    StructField::new("type", AttributeType::StringEnum {
                name: "TextTransformationType".to_string(),
                values: vec!["NONE".to_string(), "COMPRESS_WHITE_SPACE".to_string(), "HTML_ENTITY_DECODE".to_string(), "LOWERCASE".to_string(), "CMD_LINE".to_string(), "URL_DECODE".to_string(), "BASE64_DECODE".to_string(), "HEX_DECODE".to_string(), "MD5".to_string(), "REPLACE_COMMENTS".to_string(), "ESCAPE_SEQ_DECODE".to_string(), "SQL_HEX_DECODE".to_string(), "CSS_DECODE".to_string(), "JS_DECODE".to_string(), "NORMALIZE_PATH".to_string(), "NORMALIZE_PATH_WIN".to_string(), "REMOVE_NULLS".to_string(), "REPLACE_NULLS".to_string(), "BASE64_DECODE_EXT".to_string(), "URL_DECODE_UNI".to_string(), "UTF8_TO_UNICODE".to_string()],
                identity: Some(carina_core::schema::string_enum_identity("TextTransformationType", Some("awscc.wafv2.WebAcl"))),
                dsl_aliases: vec![("NONE".to_string(), "none".to_string()), ("COMPRESS_WHITE_SPACE".to_string(), "compress_white_space".to_string()), ("HTML_ENTITY_DECODE".to_string(), "html_entity_decode".to_string()), ("LOWERCASE".to_string(), "lowercase".to_string()), ("CMD_LINE".to_string(), "cmd_line".to_string()), ("URL_DECODE".to_string(), "url_decode".to_string()), ("BASE64_DECODE".to_string(), "base64_decode".to_string()), ("HEX_DECODE".to_string(), "hex_decode".to_string()), ("MD5".to_string(), "md5".to_string()), ("REPLACE_COMMENTS".to_string(), "replace_comments".to_string()), ("ESCAPE_SEQ_DECODE".to_string(), "escape_seq_decode".to_string()), ("SQL_HEX_DECODE".to_string(), "sql_hex_decode".to_string()), ("CSS_DECODE".to_string(), "css_decode".to_string()), ("JS_DECODE".to_string(), "js_decode".to_string()), ("NORMALIZE_PATH".to_string(), "normalize_path".to_string()), ("NORMALIZE_PATH_WIN".to_string(), "normalize_path_win".to_string()), ("REMOVE_NULLS".to_string(), "remove_nulls".to_string()), ("REPLACE_NULLS".to_string(), "replace_nulls".to_string()), ("BASE64_DECODE_EXT".to_string(), "base64_decode_ext".to_string()), ("URL_DECODE_UNI".to_string(), "url_decode_uni".to_string()), ("UTF8_TO_UNICODE".to_string(), "utf8_to_unicode".to_string())],
            }).required().with_provider_name("Type")
                    ],
                })).required().with_provider_name("TextTransformations").with_block_name("text_transformation")
                    ],
                }).with_provider_name("UriPath").with_block_name("uri_path")
                    ],
                })),
                validate: legacy_validator(validate_list_items_max_5),
                to_dsl: None,
            }).with_description("Specifies the aggregate keys to use in a rate-base rule.").with_provider_name("CustomKeys").with_block_name("custom_key"),
                    StructField::new("evaluation_window_sec", AttributeType::StringEnum {
                name: "EvaluationWindowSec".to_string(),
                values: vec!["60".to_string(), "120".to_string(), "300".to_string(), "600".to_string()],
                identity: Some(carina_core::schema::string_enum_identity("EvaluationWindowSec", Some("awscc.wafv2.WebAcl"))),
                dsl_aliases: vec![("60".to_string(), "60".to_string()), ("120".to_string(), "120".to_string()), ("300".to_string(), "300".to_string()), ("600".to_string(), "600".to_string())],
            }).with_provider_name("EvaluationWindowSec"),
                    StructField::new("forwarded_ip_config", AttributeType::Ref("ForwardedIPConfiguration".to_string())).with_provider_name("ForwardedIPConfig"),
                    StructField::new("limit", AttributeType::String).required().with_provider_name("Limit"),
                    StructField::new("scope_down_statement", AttributeType::Ref("Statement".to_string())).with_provider_name("ScopeDownStatement")
                    ],
                }).with_provider_name("RateBasedStatement").with_block_name("rate_based_statement"),
                    StructField::new("regex_match_statement", AttributeType::Struct {
                    name: "RegexMatchStatement".to_string(),
                    fields: vec![
                    StructField::new("field_to_match", AttributeType::Ref("FieldToMatch".to_string())).required().with_provider_name("FieldToMatch"),
                    StructField::new("regex_string", AttributeType::Custom {
                identity: None,
                pattern: None,
                length: Some((Some(1), Some(512))),
                base: Box::new(AttributeType::String),
                validate: legacy_validator(validate_string_length_1_512),
                to_dsl: None,
            }).required().with_provider_name("RegexString"),
                    StructField::new("text_transformations", AttributeType::list(AttributeType::Struct {
                    name: "TextTransformation".to_string(),
                    fields: vec![
                    StructField::new("priority", AttributeType::String).required().with_provider_name("Priority"),
                    StructField::new("type", AttributeType::StringEnum {
                name: "TextTransformationType".to_string(),
                values: vec!["NONE".to_string(), "COMPRESS_WHITE_SPACE".to_string(), "HTML_ENTITY_DECODE".to_string(), "LOWERCASE".to_string(), "CMD_LINE".to_string(), "URL_DECODE".to_string(), "BASE64_DECODE".to_string(), "HEX_DECODE".to_string(), "MD5".to_string(), "REPLACE_COMMENTS".to_string(), "ESCAPE_SEQ_DECODE".to_string(), "SQL_HEX_DECODE".to_string(), "CSS_DECODE".to_string(), "JS_DECODE".to_string(), "NORMALIZE_PATH".to_string(), "NORMALIZE_PATH_WIN".to_string(), "REMOVE_NULLS".to_string(), "REPLACE_NULLS".to_string(), "BASE64_DECODE_EXT".to_string(), "URL_DECODE_UNI".to_string(), "UTF8_TO_UNICODE".to_string()],
                identity: Some(carina_core::schema::string_enum_identity("TextTransformationType", Some("awscc.wafv2.WebAcl"))),
                dsl_aliases: vec![("NONE".to_string(), "none".to_string()), ("COMPRESS_WHITE_SPACE".to_string(), "compress_white_space".to_string()), ("HTML_ENTITY_DECODE".to_string(), "html_entity_decode".to_string()), ("LOWERCASE".to_string(), "lowercase".to_string()), ("CMD_LINE".to_string(), "cmd_line".to_string()), ("URL_DECODE".to_string(), "url_decode".to_string()), ("BASE64_DECODE".to_string(), "base64_decode".to_string()), ("HEX_DECODE".to_string(), "hex_decode".to_string()), ("MD5".to_string(), "md5".to_string()), ("REPLACE_COMMENTS".to_string(), "replace_comments".to_string()), ("ESCAPE_SEQ_DECODE".to_string(), "escape_seq_decode".to_string()), ("SQL_HEX_DECODE".to_string(), "sql_hex_decode".to_string()), ("CSS_DECODE".to_string(), "css_decode".to_string()), ("JS_DECODE".to_string(), "js_decode".to_string()), ("NORMALIZE_PATH".to_string(), "normalize_path".to_string()), ("NORMALIZE_PATH_WIN".to_string(), "normalize_path_win".to_string()), ("REMOVE_NULLS".to_string(), "remove_nulls".to_string()), ("REPLACE_NULLS".to_string(), "replace_nulls".to_string()), ("BASE64_DECODE_EXT".to_string(), "base64_decode_ext".to_string()), ("URL_DECODE_UNI".to_string(), "url_decode_uni".to_string()), ("UTF8_TO_UNICODE".to_string(), "utf8_to_unicode".to_string())],
            }).required().with_provider_name("Type")
                    ],
                })).required().with_provider_name("TextTransformations").with_block_name("text_transformation")
                    ],
                }).with_provider_name("RegexMatchStatement").with_block_name("regex_match_statement"),
                    StructField::new("regex_pattern_set_reference_statement", AttributeType::Struct {
                    name: "RegexPatternSetReferenceStatement".to_string(),
                    fields: vec![
                    StructField::new("arn", super::arn()).required().with_provider_name("Arn"),
                    StructField::new("field_to_match", AttributeType::Ref("FieldToMatch".to_string())).required().with_provider_name("FieldToMatch"),
                    StructField::new("text_transformations", AttributeType::list(AttributeType::Struct {
                    name: "TextTransformation".to_string(),
                    fields: vec![
                    StructField::new("priority", AttributeType::String).required().with_provider_name("Priority"),
                    StructField::new("type", AttributeType::StringEnum {
                name: "TextTransformationType".to_string(),
                values: vec!["NONE".to_string(), "COMPRESS_WHITE_SPACE".to_string(), "HTML_ENTITY_DECODE".to_string(), "LOWERCASE".to_string(), "CMD_LINE".to_string(), "URL_DECODE".to_string(), "BASE64_DECODE".to_string(), "HEX_DECODE".to_string(), "MD5".to_string(), "REPLACE_COMMENTS".to_string(), "ESCAPE_SEQ_DECODE".to_string(), "SQL_HEX_DECODE".to_string(), "CSS_DECODE".to_string(), "JS_DECODE".to_string(), "NORMALIZE_PATH".to_string(), "NORMALIZE_PATH_WIN".to_string(), "REMOVE_NULLS".to_string(), "REPLACE_NULLS".to_string(), "BASE64_DECODE_EXT".to_string(), "URL_DECODE_UNI".to_string(), "UTF8_TO_UNICODE".to_string()],
                identity: Some(carina_core::schema::string_enum_identity("TextTransformationType", Some("awscc.wafv2.WebAcl"))),
                dsl_aliases: vec![("NONE".to_string(), "none".to_string()), ("COMPRESS_WHITE_SPACE".to_string(), "compress_white_space".to_string()), ("HTML_ENTITY_DECODE".to_string(), "html_entity_decode".to_string()), ("LOWERCASE".to_string(), "lowercase".to_string()), ("CMD_LINE".to_string(), "cmd_line".to_string()), ("URL_DECODE".to_string(), "url_decode".to_string()), ("BASE64_DECODE".to_string(), "base64_decode".to_string()), ("HEX_DECODE".to_string(), "hex_decode".to_string()), ("MD5".to_string(), "md5".to_string()), ("REPLACE_COMMENTS".to_string(), "replace_comments".to_string()), ("ESCAPE_SEQ_DECODE".to_string(), "escape_seq_decode".to_string()), ("SQL_HEX_DECODE".to_string(), "sql_hex_decode".to_string()), ("CSS_DECODE".to_string(), "css_decode".to_string()), ("JS_DECODE".to_string(), "js_decode".to_string()), ("NORMALIZE_PATH".to_string(), "normalize_path".to_string()), ("NORMALIZE_PATH_WIN".to_string(), "normalize_path_win".to_string()), ("REMOVE_NULLS".to_string(), "remove_nulls".to_string()), ("REPLACE_NULLS".to_string(), "replace_nulls".to_string()), ("BASE64_DECODE_EXT".to_string(), "base64_decode_ext".to_string()), ("URL_DECODE_UNI".to_string(), "url_decode_uni".to_string()), ("UTF8_TO_UNICODE".to_string(), "utf8_to_unicode".to_string())],
            }).required().with_provider_name("Type")
                    ],
                })).required().with_provider_name("TextTransformations").with_block_name("text_transformation")
                    ],
                }).with_provider_name("RegexPatternSetReferenceStatement").with_block_name("regex_pattern_set_reference_statement"),
                    StructField::new("rule_group_reference_statement", AttributeType::Struct {
                    name: "RuleGroupReferenceStatement".to_string(),
                    fields: vec![
                    StructField::new("arn", super::arn()).required().with_provider_name("Arn"),
                    StructField::new("excluded_rules", AttributeType::list(AttributeType::Struct {
                    name: "ExcludedRule".to_string(),
                    fields: vec![
                    StructField::new("name", AttributeType::Custom {
                identity: None,
                pattern: Some("^[0-9A-Za-z_-]{1,128}$".to_string()),
                length: None,
                base: Box::new(AttributeType::String),
                validate: legacy_validator(validate_string_pattern_42f7eceb887966ad),
                to_dsl: None,
            }).required().with_provider_name("Name")
                    ],
                })).with_provider_name("ExcludedRules").with_block_name("excluded_rule"),
                    StructField::new("rule_action_overrides", AttributeType::Custom {
                identity: None,
                pattern: None,
                length: None,
                base: Box::new(AttributeType::list(AttributeType::Struct {
                    name: "RuleActionOverride".to_string(),
                    fields: vec![
                    StructField::new("action_to_use", AttributeType::Ref("RuleAction".to_string())).required().with_provider_name("ActionToUse"),
                    StructField::new("name", AttributeType::Custom {
                identity: None,
                pattern: Some("^[0-9A-Za-z_-]{1,128}$".to_string()),
                length: None,
                base: Box::new(AttributeType::String),
                validate: legacy_validator(validate_string_pattern_42f7eceb887966ad),
                to_dsl: None,
            }).required().with_provider_name("Name")
                    ],
                })),
                validate: legacy_validator(validate_list_items_max_100),
                to_dsl: None,
            }).with_description("Action overrides for rules in the rule group.").with_provider_name("RuleActionOverrides").with_block_name("rule_action_override")
                    ],
                }).with_provider_name("RuleGroupReferenceStatement").with_block_name("rule_group_reference_statement"),
                    StructField::new("size_constraint_statement", AttributeType::Struct {
                    name: "SizeConstraintStatement".to_string(),
                    fields: vec![
                    StructField::new("comparison_operator", AttributeType::StringEnum {
                name: "ComparisonOperator".to_string(),
                values: vec!["EQ".to_string(), "NE".to_string(), "LE".to_string(), "LT".to_string(), "GE".to_string(), "GT".to_string()],
                identity: Some(carina_core::schema::string_enum_identity("ComparisonOperator", Some("awscc.wafv2.WebAcl"))),
                dsl_aliases: vec![("EQ".to_string(), "eq".to_string()), ("NE".to_string(), "ne".to_string()), ("LE".to_string(), "le".to_string()), ("LT".to_string(), "lt".to_string()), ("GE".to_string(), "ge".to_string()), ("GT".to_string(), "gt".to_string())],
            }).required().with_provider_name("ComparisonOperator"),
                    StructField::new("field_to_match", AttributeType::Ref("FieldToMatch".to_string())).required().with_provider_name("FieldToMatch"),
                    StructField::new("size", AttributeType::Custom {
                identity: None,
                pattern: None,
                length: None,
                base: Box::new(AttributeType::Float),
                validate: legacy_validator(validate_size_range),
                to_dsl: None,
            }).required().with_provider_name("Size"),
                    StructField::new("text_transformations", AttributeType::list(AttributeType::Struct {
                    name: "TextTransformation".to_string(),
                    fields: vec![
                    StructField::new("priority", AttributeType::String).required().with_provider_name("Priority"),
                    StructField::new("type", AttributeType::StringEnum {
                name: "TextTransformationType".to_string(),
                values: vec!["NONE".to_string(), "COMPRESS_WHITE_SPACE".to_string(), "HTML_ENTITY_DECODE".to_string(), "LOWERCASE".to_string(), "CMD_LINE".to_string(), "URL_DECODE".to_string(), "BASE64_DECODE".to_string(), "HEX_DECODE".to_string(), "MD5".to_string(), "REPLACE_COMMENTS".to_string(), "ESCAPE_SEQ_DECODE".to_string(), "SQL_HEX_DECODE".to_string(), "CSS_DECODE".to_string(), "JS_DECODE".to_string(), "NORMALIZE_PATH".to_string(), "NORMALIZE_PATH_WIN".to_string(), "REMOVE_NULLS".to_string(), "REPLACE_NULLS".to_string(), "BASE64_DECODE_EXT".to_string(), "URL_DECODE_UNI".to_string(), "UTF8_TO_UNICODE".to_string()],
                identity: Some(carina_core::schema::string_enum_identity("TextTransformationType", Some("awscc.wafv2.WebAcl"))),
                dsl_aliases: vec![("NONE".to_string(), "none".to_string()), ("COMPRESS_WHITE_SPACE".to_string(), "compress_white_space".to_string()), ("HTML_ENTITY_DECODE".to_string(), "html_entity_decode".to_string()), ("LOWERCASE".to_string(), "lowercase".to_string()), ("CMD_LINE".to_string(), "cmd_line".to_string()), ("URL_DECODE".to_string(), "url_decode".to_string()), ("BASE64_DECODE".to_string(), "base64_decode".to_string()), ("HEX_DECODE".to_string(), "hex_decode".to_string()), ("MD5".to_string(), "md5".to_string()), ("REPLACE_COMMENTS".to_string(), "replace_comments".to_string()), ("ESCAPE_SEQ_DECODE".to_string(), "escape_seq_decode".to_string()), ("SQL_HEX_DECODE".to_string(), "sql_hex_decode".to_string()), ("CSS_DECODE".to_string(), "css_decode".to_string()), ("JS_DECODE".to_string(), "js_decode".to_string()), ("NORMALIZE_PATH".to_string(), "normalize_path".to_string()), ("NORMALIZE_PATH_WIN".to_string(), "normalize_path_win".to_string()), ("REMOVE_NULLS".to_string(), "remove_nulls".to_string()), ("REPLACE_NULLS".to_string(), "replace_nulls".to_string()), ("BASE64_DECODE_EXT".to_string(), "base64_decode_ext".to_string()), ("URL_DECODE_UNI".to_string(), "url_decode_uni".to_string()), ("UTF8_TO_UNICODE".to_string(), "utf8_to_unicode".to_string())],
            }).required().with_provider_name("Type")
                    ],
                })).required().with_provider_name("TextTransformations").with_block_name("text_transformation")
                    ],
                }).with_provider_name("SizeConstraintStatement").with_block_name("size_constraint_statement"),
                    StructField::new("sqli_match_statement", AttributeType::Struct {
                    name: "SqliMatchStatement".to_string(),
                    fields: vec![
                    StructField::new("field_to_match", AttributeType::Ref("FieldToMatch".to_string())).required().with_provider_name("FieldToMatch"),
                    StructField::new("sensitivity_level", AttributeType::StringEnum {
                name: "SensitivityLevel".to_string(),
                values: vec!["LOW".to_string(), "HIGH".to_string()],
                identity: Some(carina_core::schema::string_enum_identity("SensitivityLevel", Some("awscc.wafv2.WebAcl"))),
                dsl_aliases: vec![("LOW".to_string(), "low".to_string()), ("HIGH".to_string(), "high".to_string())],
            }).with_provider_name("SensitivityLevel"),
                    StructField::new("text_transformations", AttributeType::list(AttributeType::Struct {
                    name: "TextTransformation".to_string(),
                    fields: vec![
                    StructField::new("priority", AttributeType::String).required().with_provider_name("Priority"),
                    StructField::new("type", AttributeType::StringEnum {
                name: "TextTransformationType".to_string(),
                values: vec!["NONE".to_string(), "COMPRESS_WHITE_SPACE".to_string(), "HTML_ENTITY_DECODE".to_string(), "LOWERCASE".to_string(), "CMD_LINE".to_string(), "URL_DECODE".to_string(), "BASE64_DECODE".to_string(), "HEX_DECODE".to_string(), "MD5".to_string(), "REPLACE_COMMENTS".to_string(), "ESCAPE_SEQ_DECODE".to_string(), "SQL_HEX_DECODE".to_string(), "CSS_DECODE".to_string(), "JS_DECODE".to_string(), "NORMALIZE_PATH".to_string(), "NORMALIZE_PATH_WIN".to_string(), "REMOVE_NULLS".to_string(), "REPLACE_NULLS".to_string(), "BASE64_DECODE_EXT".to_string(), "URL_DECODE_UNI".to_string(), "UTF8_TO_UNICODE".to_string()],
                identity: Some(carina_core::schema::string_enum_identity("TextTransformationType", Some("awscc.wafv2.WebAcl"))),
                dsl_aliases: vec![("NONE".to_string(), "none".to_string()), ("COMPRESS_WHITE_SPACE".to_string(), "compress_white_space".to_string()), ("HTML_ENTITY_DECODE".to_string(), "html_entity_decode".to_string()), ("LOWERCASE".to_string(), "lowercase".to_string()), ("CMD_LINE".to_string(), "cmd_line".to_string()), ("URL_DECODE".to_string(), "url_decode".to_string()), ("BASE64_DECODE".to_string(), "base64_decode".to_string()), ("HEX_DECODE".to_string(), "hex_decode".to_string()), ("MD5".to_string(), "md5".to_string()), ("REPLACE_COMMENTS".to_string(), "replace_comments".to_string()), ("ESCAPE_SEQ_DECODE".to_string(), "escape_seq_decode".to_string()), ("SQL_HEX_DECODE".to_string(), "sql_hex_decode".to_string()), ("CSS_DECODE".to_string(), "css_decode".to_string()), ("JS_DECODE".to_string(), "js_decode".to_string()), ("NORMALIZE_PATH".to_string(), "normalize_path".to_string()), ("NORMALIZE_PATH_WIN".to_string(), "normalize_path_win".to_string()), ("REMOVE_NULLS".to_string(), "remove_nulls".to_string()), ("REPLACE_NULLS".to_string(), "replace_nulls".to_string()), ("BASE64_DECODE_EXT".to_string(), "base64_decode_ext".to_string()), ("URL_DECODE_UNI".to_string(), "url_decode_uni".to_string()), ("UTF8_TO_UNICODE".to_string(), "utf8_to_unicode".to_string())],
            }).required().with_provider_name("Type")
                    ],
                })).required().with_provider_name("TextTransformations").with_block_name("text_transformation")
                    ],
                }).with_provider_name("SqliMatchStatement").with_block_name("sqli_match_statement"),
                    StructField::new("xss_match_statement", AttributeType::Struct {
                    name: "XssMatchStatement".to_string(),
                    fields: vec![
                    StructField::new("field_to_match", AttributeType::Ref("FieldToMatch".to_string())).required().with_provider_name("FieldToMatch"),
                    StructField::new("text_transformations", AttributeType::list(AttributeType::Struct {
                    name: "TextTransformation".to_string(),
                    fields: vec![
                    StructField::new("priority", AttributeType::String).required().with_provider_name("Priority"),
                    StructField::new("type", AttributeType::StringEnum {
                name: "TextTransformationType".to_string(),
                values: vec!["NONE".to_string(), "COMPRESS_WHITE_SPACE".to_string(), "HTML_ENTITY_DECODE".to_string(), "LOWERCASE".to_string(), "CMD_LINE".to_string(), "URL_DECODE".to_string(), "BASE64_DECODE".to_string(), "HEX_DECODE".to_string(), "MD5".to_string(), "REPLACE_COMMENTS".to_string(), "ESCAPE_SEQ_DECODE".to_string(), "SQL_HEX_DECODE".to_string(), "CSS_DECODE".to_string(), "JS_DECODE".to_string(), "NORMALIZE_PATH".to_string(), "NORMALIZE_PATH_WIN".to_string(), "REMOVE_NULLS".to_string(), "REPLACE_NULLS".to_string(), "BASE64_DECODE_EXT".to_string(), "URL_DECODE_UNI".to_string(), "UTF8_TO_UNICODE".to_string()],
                identity: Some(carina_core::schema::string_enum_identity("TextTransformationType", Some("awscc.wafv2.WebAcl"))),
                dsl_aliases: vec![("NONE".to_string(), "none".to_string()), ("COMPRESS_WHITE_SPACE".to_string(), "compress_white_space".to_string()), ("HTML_ENTITY_DECODE".to_string(), "html_entity_decode".to_string()), ("LOWERCASE".to_string(), "lowercase".to_string()), ("CMD_LINE".to_string(), "cmd_line".to_string()), ("URL_DECODE".to_string(), "url_decode".to_string()), ("BASE64_DECODE".to_string(), "base64_decode".to_string()), ("HEX_DECODE".to_string(), "hex_decode".to_string()), ("MD5".to_string(), "md5".to_string()), ("REPLACE_COMMENTS".to_string(), "replace_comments".to_string()), ("ESCAPE_SEQ_DECODE".to_string(), "escape_seq_decode".to_string()), ("SQL_HEX_DECODE".to_string(), "sql_hex_decode".to_string()), ("CSS_DECODE".to_string(), "css_decode".to_string()), ("JS_DECODE".to_string(), "js_decode".to_string()), ("NORMALIZE_PATH".to_string(), "normalize_path".to_string()), ("NORMALIZE_PATH_WIN".to_string(), "normalize_path_win".to_string()), ("REMOVE_NULLS".to_string(), "remove_nulls".to_string()), ("REPLACE_NULLS".to_string(), "replace_nulls".to_string()), ("BASE64_DECODE_EXT".to_string(), "base64_decode_ext".to_string()), ("URL_DECODE_UNI".to_string(), "url_decode_uni".to_string()), ("UTF8_TO_UNICODE".to_string(), "utf8_to_unicode".to_string())],
            }).required().with_provider_name("Type")
                    ],
                })).required().with_provider_name("TextTransformations").with_block_name("text_transformation")
                    ],
                }).with_provider_name("XssMatchStatement").with_block_name("xss_match_statement")
                    ],
                })).required().with_provider_name("Statements").with_block_name("statement")
                    ],
                }).with_provider_name("OrStatement").with_block_name("or_statement"),
                    StructField::new("rate_based_statement", AttributeType::Ref("RateBasedStatement".to_string())).with_provider_name("RateBasedStatement"),
                    StructField::new("regex_match_statement", AttributeType::Ref("RegexMatchStatement".to_string())).with_provider_name("RegexMatchStatement"),
                    StructField::new("regex_pattern_set_reference_statement", AttributeType::Ref("RegexPatternSetReferenceStatement".to_string())).with_provider_name("RegexPatternSetReferenceStatement"),
                    StructField::new("rule_group_reference_statement", AttributeType::Ref("RuleGroupReferenceStatement".to_string())).with_provider_name("RuleGroupReferenceStatement"),
                    StructField::new("size_constraint_statement", AttributeType::Ref("SizeConstraintStatement".to_string())).with_provider_name("SizeConstraintStatement"),
                    StructField::new("sqli_match_statement", AttributeType::Ref("SqliMatchStatement".to_string())).with_provider_name("SqliMatchStatement"),
                    StructField::new("xss_match_statement", AttributeType::Ref("XssMatchStatement".to_string())).with_provider_name("XssMatchStatement")
                    ],
                })).required().with_provider_name("Statements").with_block_name("statement")
                    ],
                })
        .with_def("ApplicationAttribute", AttributeType::Struct {
                    name: "ApplicationAttribute".to_string(),
                    fields: vec![
                    StructField::new("name", AttributeType::Custom {
                identity: None,
                pattern: Some("^[\\w\\-]+$".to_string()),
                length: Some((Some(1), Some(64))),
                base: Box::new(AttributeType::String),
                validate: legacy_validator(validate_string_pattern_6fd4a12c0ce64c08_len_1_64),
                to_dsl: None,
            }).required().with_provider_name("Name"),
                    StructField::new("values", AttributeType::list(AttributeType::Custom {
                identity: None,
                pattern: Some("^[\\w\\-]+$".to_string()),
                length: Some((Some(1), Some(64))),
                base: Box::new(AttributeType::String),
                validate: legacy_validator(validate_string_pattern_6fd4a12c0ce64c08_len_1_64),
                to_dsl: None,
            })).required().with_provider_name("Values")
                    ],
                })
        .with_def("ApplicationConfig", AttributeType::Struct {
                    name: "ApplicationConfig".to_string(),
                    fields: vec![
                    StructField::new("attributes", AttributeType::list(AttributeType::Struct {
                    name: "ApplicationAttribute".to_string(),
                    fields: vec![
                    StructField::new("name", AttributeType::Custom {
                identity: None,
                pattern: Some("^[\\w\\-]+$".to_string()),
                length: Some((Some(1), Some(64))),
                base: Box::new(AttributeType::String),
                validate: legacy_validator(validate_string_pattern_6fd4a12c0ce64c08_len_1_64),
                to_dsl: None,
            }).required().with_provider_name("Name"),
                    StructField::new("values", AttributeType::list(AttributeType::Custom {
                identity: None,
                pattern: Some("^[\\w\\-]+$".to_string()),
                length: Some((Some(1), Some(64))),
                base: Box::new(AttributeType::String),
                validate: legacy_validator(validate_string_pattern_6fd4a12c0ce64c08_len_1_64),
                to_dsl: None,
            })).required().with_provider_name("Values")
                    ],
                })).required().with_provider_name("Attributes").with_block_name("attribute")
                    ],
                })
        .with_def("AsnMatchStatement", AttributeType::Struct {
                    name: "AsnMatchStatement".to_string(),
                    fields: vec![
                    StructField::new("asn_list", AttributeType::list(AttributeType::Custom {
                identity: None,
                pattern: None,
                length: None,
                base: Box::new(AttributeType::Int),
                validate: legacy_validator(validate_asn_list_range),
                to_dsl: None,
            })).with_provider_name("AsnList"),
                    StructField::new("forwarded_ip_config", AttributeType::Struct {
                    name: "ForwardedIPConfiguration".to_string(),
                    fields: vec![
                    StructField::new("fallback_behavior", AttributeType::StringEnum {
                name: "FallbackBehavior".to_string(),
                values: vec!["MATCH".to_string(), "NO_MATCH".to_string()],
                identity: Some(carina_core::schema::string_enum_identity("FallbackBehavior", Some("awscc.wafv2.WebAcl"))),
                dsl_aliases: vec![("MATCH".to_string(), "match".to_string()), ("NO_MATCH".to_string(), "no_match".to_string())],
            }).required().with_provider_name("FallbackBehavior"),
                    StructField::new("header_name", AttributeType::Custom {
                identity: None,
                pattern: Some("^[a-zA-Z0-9-]+{1,255}$".to_string()),
                length: None,
                base: Box::new(AttributeType::String),
                validate: legacy_validator(validate_string_pattern_2d93cc844f6d4014),
                to_dsl: None,
            }).required().with_provider_name("HeaderName")
                    ],
                }).with_provider_name("ForwardedIPConfig")
                    ],
                })
        .with_def("AssociationConfig", AttributeType::Struct {
                    name: "AssociationConfig".to_string(),
                    fields: vec![
                    StructField::new("request_body", AttributeType::String).with_provider_name("RequestBody")
                    ],
                })
        .with_def("BlockAction", AttributeType::Struct {
                    name: "BlockAction".to_string(),
                    fields: vec![
                    StructField::new("custom_response", AttributeType::Struct {
                    name: "CustomResponse".to_string(),
                    fields: vec![
                    StructField::new("custom_response_body_key", AttributeType::Custom {
                identity: None,
                pattern: Some("^[\\w\\-]+$".to_string()),
                length: None,
                base: Box::new(AttributeType::String),
                validate: legacy_validator(validate_string_pattern_6fd4a12c0ce64c08),
                to_dsl: None,
            }).with_description("Custom response body key.").with_provider_name("CustomResponseBodyKey"),
                    StructField::new("response_code", AttributeType::String).required().with_provider_name("ResponseCode"),
                    StructField::new("response_headers", AttributeType::Custom {
                identity: None,
                pattern: None,
                length: None,
                base: Box::new(AttributeType::list(AttributeType::Struct {
                    name: "CustomHTTPHeader".to_string(),
                    fields: vec![
                    StructField::new("name", AttributeType::String).required().with_provider_name("Name"),
                    StructField::new("value", AttributeType::String).required().with_provider_name("Value")
                    ],
                })),
                validate: legacy_validator(validate_list_items_min_1),
                to_dsl: None,
            }).with_description("Collection of HTTP headers.").with_provider_name("ResponseHeaders").with_block_name("response_header")
                    ],
                }).with_provider_name("CustomResponse").with_block_name("custom_response")
                    ],
                })
        .with_def("Body", AttributeType::Struct {
                    name: "Body".to_string(),
                    fields: vec![
                    StructField::new("oversize_handling", AttributeType::StringEnum {
                name: "OversizeHandling".to_string(),
                values: vec!["CONTINUE".to_string(), "MATCH".to_string(), "NO_MATCH".to_string()],
                identity: Some(carina_core::schema::string_enum_identity("OversizeHandling", Some("awscc.wafv2.WebAcl"))),
                dsl_aliases: vec![("CONTINUE".to_string(), "continue".to_string()), ("MATCH".to_string(), "match".to_string()), ("NO_MATCH".to_string(), "no_match".to_string())],
            }).with_provider_name("OversizeHandling")
                    ],
                })
        .with_def("ByteMatchStatement", AttributeType::Struct {
                    name: "ByteMatchStatement".to_string(),
                    fields: vec![
                    StructField::new("field_to_match", AttributeType::Struct {
                    name: "FieldToMatch".to_string(),
                    fields: vec![
                    StructField::new("all_query_arguments", AttributeType::map(AttributeType::String)).with_description("All query arguments of a web request.").with_provider_name("AllQueryArguments"),
                    StructField::new("body", AttributeType::Struct {
                    name: "Body".to_string(),
                    fields: vec![
                    StructField::new("oversize_handling", AttributeType::StringEnum {
                name: "OversizeHandling".to_string(),
                values: vec!["CONTINUE".to_string(), "MATCH".to_string(), "NO_MATCH".to_string()],
                identity: Some(carina_core::schema::string_enum_identity("OversizeHandling", Some("awscc.wafv2.WebAcl"))),
                dsl_aliases: vec![("CONTINUE".to_string(), "continue".to_string()), ("MATCH".to_string(), "match".to_string()), ("NO_MATCH".to_string(), "no_match".to_string())],
            }).with_provider_name("OversizeHandling")
                    ],
                }).with_provider_name("Body"),
                    StructField::new("cookies", AttributeType::Struct {
                    name: "Cookies".to_string(),
                    fields: vec![
                    StructField::new("match_pattern", AttributeType::Struct {
                    name: "CookieMatchPattern".to_string(),
                    fields: vec![
                    StructField::new("all", AttributeType::map(AttributeType::String)).with_description("Inspect all parts of the web request cookies.").with_provider_name("All"),
                    StructField::new("excluded_cookies", AttributeType::Custom {
                identity: None,
                pattern: None,
                length: None,
                base: Box::new(AttributeType::list(AttributeType::Custom {
                identity: None,
                pattern: Some(".*\\S.*".to_string()),
                length: Some((Some(1), Some(60))),
                base: Box::new(AttributeType::String),
                validate: legacy_validator(validate_string_pattern_4e09c821aed9e752_len_1_60),
                to_dsl: None,
            })),
                validate: legacy_validator(validate_list_items_1_199),
                to_dsl: None,
            }).with_provider_name("ExcludedCookies"),
                    StructField::new("included_cookies", AttributeType::Custom {
                identity: None,
                pattern: None,
                length: None,
                base: Box::new(AttributeType::list(AttributeType::Custom {
                identity: None,
                pattern: Some(".*\\S.*".to_string()),
                length: Some((Some(1), Some(60))),
                base: Box::new(AttributeType::String),
                validate: legacy_validator(validate_string_pattern_4e09c821aed9e752_len_1_60),
                to_dsl: None,
            })),
                validate: legacy_validator(validate_list_items_1_199),
                to_dsl: None,
            }).with_provider_name("IncludedCookies")
                    ],
                }).required().with_provider_name("MatchPattern"),
                    StructField::new("match_scope", AttributeType::StringEnum {
                name: "MapMatchScope".to_string(),
                values: vec!["ALL".to_string(), "KEY".to_string(), "VALUE".to_string()],
                identity: Some(carina_core::schema::string_enum_identity("MapMatchScope", Some("awscc.wafv2.WebAcl"))),
                dsl_aliases: vec![("ALL".to_string(), "all".to_string()), ("KEY".to_string(), "key".to_string()), ("VALUE".to_string(), "value".to_string())],
            }).required().with_provider_name("MatchScope"),
                    StructField::new("oversize_handling", AttributeType::StringEnum {
                name: "OversizeHandling".to_string(),
                values: vec!["CONTINUE".to_string(), "MATCH".to_string(), "NO_MATCH".to_string()],
                identity: Some(carina_core::schema::string_enum_identity("OversizeHandling", Some("awscc.wafv2.WebAcl"))),
                dsl_aliases: vec![("CONTINUE".to_string(), "continue".to_string()), ("MATCH".to_string(), "match".to_string()), ("NO_MATCH".to_string(), "no_match".to_string())],
            }).required().with_provider_name("OversizeHandling")
                    ],
                }).with_provider_name("Cookies"),
                    StructField::new("header_order", AttributeType::Struct {
                    name: "HeaderOrder".to_string(),
                    fields: vec![
                    StructField::new("oversize_handling", AttributeType::StringEnum {
                name: "OversizeHandling".to_string(),
                values: vec!["CONTINUE".to_string(), "MATCH".to_string(), "NO_MATCH".to_string()],
                identity: Some(carina_core::schema::string_enum_identity("OversizeHandling", Some("awscc.wafv2.WebAcl"))),
                dsl_aliases: vec![("CONTINUE".to_string(), "continue".to_string()), ("MATCH".to_string(), "match".to_string()), ("NO_MATCH".to_string(), "no_match".to_string())],
            }).required().with_provider_name("OversizeHandling")
                    ],
                }).with_provider_name("HeaderOrder"),
                    StructField::new("headers", AttributeType::Struct {
                    name: "Headers".to_string(),
                    fields: vec![
                    StructField::new("match_pattern", AttributeType::Struct {
                    name: "HeaderMatchPattern".to_string(),
                    fields: vec![
                    StructField::new("all", AttributeType::map(AttributeType::String)).with_description("Inspect all parts of the web request headers.").with_provider_name("All"),
                    StructField::new("excluded_headers", AttributeType::Custom {
                identity: None,
                pattern: None,
                length: None,
                base: Box::new(AttributeType::list(AttributeType::Custom {
                identity: None,
                pattern: Some(".*\\S.*".to_string()),
                length: Some((Some(1), Some(64))),
                base: Box::new(AttributeType::String),
                validate: legacy_validator(validate_string_pattern_4e09c821aed9e752_len_1_64),
                to_dsl: None,
            })),
                validate: legacy_validator(validate_list_items_1_199),
                to_dsl: None,
            }).with_provider_name("ExcludedHeaders"),
                    StructField::new("included_headers", AttributeType::Custom {
                identity: None,
                pattern: None,
                length: None,
                base: Box::new(AttributeType::list(AttributeType::Custom {
                identity: None,
                pattern: Some(".*\\S.*".to_string()),
                length: Some((Some(1), Some(64))),
                base: Box::new(AttributeType::String),
                validate: legacy_validator(validate_string_pattern_4e09c821aed9e752_len_1_64),
                to_dsl: None,
            })),
                validate: legacy_validator(validate_list_items_1_199),
                to_dsl: None,
            }).with_provider_name("IncludedHeaders")
                    ],
                }).required().with_provider_name("MatchPattern"),
                    StructField::new("match_scope", AttributeType::StringEnum {
                name: "MapMatchScope".to_string(),
                values: vec!["ALL".to_string(), "KEY".to_string(), "VALUE".to_string()],
                identity: Some(carina_core::schema::string_enum_identity("MapMatchScope", Some("awscc.wafv2.WebAcl"))),
                dsl_aliases: vec![("ALL".to_string(), "all".to_string()), ("KEY".to_string(), "key".to_string()), ("VALUE".to_string(), "value".to_string())],
            }).required().with_provider_name("MatchScope"),
                    StructField::new("oversize_handling", AttributeType::StringEnum {
                name: "OversizeHandling".to_string(),
                values: vec!["CONTINUE".to_string(), "MATCH".to_string(), "NO_MATCH".to_string()],
                identity: Some(carina_core::schema::string_enum_identity("OversizeHandling", Some("awscc.wafv2.WebAcl"))),
                dsl_aliases: vec![("CONTINUE".to_string(), "continue".to_string()), ("MATCH".to_string(), "match".to_string()), ("NO_MATCH".to_string(), "no_match".to_string())],
            }).required().with_provider_name("OversizeHandling")
                    ],
                }).with_provider_name("Headers"),
                    StructField::new("ja3_fingerprint", AttributeType::Struct {
                    name: "JA3Fingerprint".to_string(),
                    fields: vec![
                    StructField::new("fallback_behavior", AttributeType::StringEnum {
                name: "FallbackBehavior".to_string(),
                values: vec!["MATCH".to_string(), "NO_MATCH".to_string()],
                identity: Some(carina_core::schema::string_enum_identity("FallbackBehavior", Some("awscc.wafv2.WebAcl"))),
                dsl_aliases: vec![("MATCH".to_string(), "match".to_string()), ("NO_MATCH".to_string(), "no_match".to_string())],
            }).required().with_provider_name("FallbackBehavior")
                    ],
                }).with_provider_name("JA3Fingerprint"),
                    StructField::new("ja4_fingerprint", AttributeType::Struct {
                    name: "JA4Fingerprint".to_string(),
                    fields: vec![
                    StructField::new("fallback_behavior", AttributeType::StringEnum {
                name: "FallbackBehavior".to_string(),
                values: vec!["MATCH".to_string(), "NO_MATCH".to_string()],
                identity: Some(carina_core::schema::string_enum_identity("FallbackBehavior", Some("awscc.wafv2.WebAcl"))),
                dsl_aliases: vec![("MATCH".to_string(), "match".to_string()), ("NO_MATCH".to_string(), "no_match".to_string())],
            }).required().with_provider_name("FallbackBehavior")
                    ],
                }).with_provider_name("JA4Fingerprint"),
                    StructField::new("json_body", AttributeType::Struct {
                    name: "JsonBody".to_string(),
                    fields: vec![
                    StructField::new("invalid_fallback_behavior", AttributeType::StringEnum {
                name: "BodyParsingFallbackBehavior".to_string(),
                values: vec!["MATCH".to_string(), "NO_MATCH".to_string(), "EVALUATE_AS_STRING".to_string()],
                identity: Some(carina_core::schema::string_enum_identity("BodyParsingFallbackBehavior", Some("awscc.wafv2.WebAcl"))),
                dsl_aliases: vec![("MATCH".to_string(), "match".to_string()), ("NO_MATCH".to_string(), "no_match".to_string()), ("EVALUATE_AS_STRING".to_string(), "evaluate_as_string".to_string())],
            }).with_provider_name("InvalidFallbackBehavior"),
                    StructField::new("match_pattern", AttributeType::Struct {
                    name: "JsonMatchPattern".to_string(),
                    fields: vec![
                    StructField::new("all", AttributeType::map(AttributeType::String)).with_description("Inspect all parts of the web request's JSON body.").with_provider_name("All"),
                    StructField::new("included_paths", AttributeType::list(AttributeType::Custom {
                identity: None,
                pattern: Some("^[\\/]+([^~]*(~[01])*)*{1,512}$".to_string()),
                length: None,
                base: Box::new(AttributeType::String),
                validate: legacy_validator(validate_string_pattern_c77cf75cf1a75ade),
                to_dsl: None,
            })).with_provider_name("IncludedPaths")
                    ],
                }).required().with_provider_name("MatchPattern"),
                    StructField::new("match_scope", AttributeType::StringEnum {
                name: "JsonMatchScope".to_string(),
                values: vec!["ALL".to_string(), "KEY".to_string(), "VALUE".to_string()],
                identity: Some(carina_core::schema::string_enum_identity("JsonMatchScope", Some("awscc.wafv2.WebAcl"))),
                dsl_aliases: vec![("ALL".to_string(), "all".to_string()), ("KEY".to_string(), "key".to_string()), ("VALUE".to_string(), "value".to_string())],
            }).required().with_provider_name("MatchScope"),
                    StructField::new("oversize_handling", AttributeType::StringEnum {
                name: "OversizeHandling".to_string(),
                values: vec!["CONTINUE".to_string(), "MATCH".to_string(), "NO_MATCH".to_string()],
                identity: Some(carina_core::schema::string_enum_identity("OversizeHandling", Some("awscc.wafv2.WebAcl"))),
                dsl_aliases: vec![("CONTINUE".to_string(), "continue".to_string()), ("MATCH".to_string(), "match".to_string()), ("NO_MATCH".to_string(), "no_match".to_string())],
            }).with_provider_name("OversizeHandling")
                    ],
                }).with_provider_name("JsonBody"),
                    StructField::new("method", AttributeType::map(AttributeType::String)).with_description("The HTTP method of a web request. The method indicates the type of operation that the request is asking the origin to perform.").with_provider_name("Method"),
                    StructField::new("query_string", AttributeType::map(AttributeType::String)).with_description("The query string of a web request. This is the part of a URL that appears after a ? character, if any.").with_provider_name("QueryString"),
                    StructField::new("single_header", AttributeType::Struct {
                    name: "SingleHeader".to_string(),
                    fields: vec![
                    StructField::new("name", AttributeType::String).required().with_provider_name("Name")
                    ],
                }).with_provider_name("SingleHeader"),
                    StructField::new("single_query_argument", AttributeType::Struct {
                    name: "SingleQueryArgument".to_string(),
                    fields: vec![
                    StructField::new("name", AttributeType::String).required().with_provider_name("Name")
                    ],
                }).with_description("One query argument in a web request, identified by name, for example UserName or SalesRegion. The name can be up to 30 characters long and isn't case sensitive.").with_provider_name("SingleQueryArgument"),
                    StructField::new("uri_fragment", AttributeType::Struct {
                    name: "UriFragment".to_string(),
                    fields: vec![
                    StructField::new("fallback_behavior", AttributeType::StringEnum {
                name: "FallbackBehavior".to_string(),
                values: vec!["MATCH".to_string(), "NO_MATCH".to_string()],
                identity: Some(carina_core::schema::string_enum_identity("FallbackBehavior", Some("awscc.wafv2.WebAcl"))),
                dsl_aliases: vec![("MATCH".to_string(), "match".to_string()), ("NO_MATCH".to_string(), "no_match".to_string())],
            }).with_provider_name("FallbackBehavior")
                    ],
                }).with_provider_name("UriFragment"),
                    StructField::new("uri_path", AttributeType::map(AttributeType::String)).with_description("The path component of the URI of a web request. This is the part of a web request that identifies a resource, for example, /images/daily-ad.jpg.").with_provider_name("UriPath")
                    ],
                }).required().with_provider_name("FieldToMatch"),
                    StructField::new("positional_constraint", AttributeType::StringEnum {
                name: "PositionalConstraint".to_string(),
                values: vec!["EXACTLY".to_string(), "STARTS_WITH".to_string(), "ENDS_WITH".to_string(), "CONTAINS".to_string(), "CONTAINS_WORD".to_string()],
                identity: Some(carina_core::schema::string_enum_identity("PositionalConstraint", Some("awscc.wafv2.WebAcl"))),
                dsl_aliases: vec![("EXACTLY".to_string(), "exactly".to_string()), ("STARTS_WITH".to_string(), "starts_with".to_string()), ("ENDS_WITH".to_string(), "ends_with".to_string()), ("CONTAINS".to_string(), "contains".to_string()), ("CONTAINS_WORD".to_string(), "contains_word".to_string())],
            }).required().with_provider_name("PositionalConstraint"),
                    StructField::new("search_string", AttributeType::String).with_provider_name("SearchString"),
                    StructField::new("search_string_base64", AttributeType::String).with_provider_name("SearchStringBase64"),
                    StructField::new("text_transformations", AttributeType::list(AttributeType::Struct {
                    name: "TextTransformation".to_string(),
                    fields: vec![
                    StructField::new("priority", AttributeType::String).required().with_provider_name("Priority"),
                    StructField::new("type", AttributeType::StringEnum {
                name: "TextTransformationType".to_string(),
                values: vec!["NONE".to_string(), "COMPRESS_WHITE_SPACE".to_string(), "HTML_ENTITY_DECODE".to_string(), "LOWERCASE".to_string(), "CMD_LINE".to_string(), "URL_DECODE".to_string(), "BASE64_DECODE".to_string(), "HEX_DECODE".to_string(), "MD5".to_string(), "REPLACE_COMMENTS".to_string(), "ESCAPE_SEQ_DECODE".to_string(), "SQL_HEX_DECODE".to_string(), "CSS_DECODE".to_string(), "JS_DECODE".to_string(), "NORMALIZE_PATH".to_string(), "NORMALIZE_PATH_WIN".to_string(), "REMOVE_NULLS".to_string(), "REPLACE_NULLS".to_string(), "BASE64_DECODE_EXT".to_string(), "URL_DECODE_UNI".to_string(), "UTF8_TO_UNICODE".to_string()],
                identity: Some(carina_core::schema::string_enum_identity("TextTransformationType", Some("awscc.wafv2.WebAcl"))),
                dsl_aliases: vec![("NONE".to_string(), "none".to_string()), ("COMPRESS_WHITE_SPACE".to_string(), "compress_white_space".to_string()), ("HTML_ENTITY_DECODE".to_string(), "html_entity_decode".to_string()), ("LOWERCASE".to_string(), "lowercase".to_string()), ("CMD_LINE".to_string(), "cmd_line".to_string()), ("URL_DECODE".to_string(), "url_decode".to_string()), ("BASE64_DECODE".to_string(), "base64_decode".to_string()), ("HEX_DECODE".to_string(), "hex_decode".to_string()), ("MD5".to_string(), "md5".to_string()), ("REPLACE_COMMENTS".to_string(), "replace_comments".to_string()), ("ESCAPE_SEQ_DECODE".to_string(), "escape_seq_decode".to_string()), ("SQL_HEX_DECODE".to_string(), "sql_hex_decode".to_string()), ("CSS_DECODE".to_string(), "css_decode".to_string()), ("JS_DECODE".to_string(), "js_decode".to_string()), ("NORMALIZE_PATH".to_string(), "normalize_path".to_string()), ("NORMALIZE_PATH_WIN".to_string(), "normalize_path_win".to_string()), ("REMOVE_NULLS".to_string(), "remove_nulls".to_string()), ("REPLACE_NULLS".to_string(), "replace_nulls".to_string()), ("BASE64_DECODE_EXT".to_string(), "base64_decode_ext".to_string()), ("URL_DECODE_UNI".to_string(), "url_decode_uni".to_string()), ("UTF8_TO_UNICODE".to_string(), "utf8_to_unicode".to_string())],
            }).required().with_provider_name("Type")
                    ],
                })).required().with_provider_name("TextTransformations").with_block_name("text_transformation")
                    ],
                })
        .with_def("CaptchaAction", AttributeType::Struct {
                    name: "CaptchaAction".to_string(),
                    fields: vec![
                    StructField::new("custom_request_handling", AttributeType::Ref("CustomRequestHandling".to_string())).with_provider_name("CustomRequestHandling")
                    ],
                })
        .with_def("CaptchaConfig", AttributeType::Struct {
                    name: "CaptchaConfig".to_string(),
                    fields: vec![
                    StructField::new("immunity_time_property", AttributeType::Struct {
                    name: "ImmunityTimeProperty".to_string(),
                    fields: vec![
                    StructField::new("immunity_time", AttributeType::Custom {
                identity: None,
                pattern: None,
                length: None,
                base: Box::new(AttributeType::Int),
                validate: legacy_validator(validate_immunity_time_range),
                to_dsl: None,
            }).required().with_provider_name("ImmunityTime")
                    ],
                }).with_provider_name("ImmunityTimeProperty")
                    ],
                })
        .with_def("ChallengeAction", AttributeType::Struct {
                    name: "ChallengeAction".to_string(),
                    fields: vec![
                    StructField::new("custom_request_handling", AttributeType::Ref("CustomRequestHandling".to_string())).with_provider_name("CustomRequestHandling")
                    ],
                })
        .with_def("ChallengeConfig", AttributeType::Struct {
                    name: "ChallengeConfig".to_string(),
                    fields: vec![
                    StructField::new("immunity_time_property", AttributeType::Ref("ImmunityTimeProperty".to_string())).with_provider_name("ImmunityTimeProperty")
                    ],
                })
        .with_def("ClientSideAction", AttributeType::Struct {
                    name: "ClientSideAction".to_string(),
                    fields: vec![
                    StructField::new("exempt_uri_regular_expressions", AttributeType::list(AttributeType::Struct {
                    name: "Regex".to_string(),
                    fields: vec![
                    StructField::new("regex_string", AttributeType::String).with_provider_name("RegexString")
                    ],
                })).with_provider_name("ExemptUriRegularExpressions").with_block_name("exempt_uri_regular_expression"),
                    StructField::new("sensitivity", AttributeType::StringEnum {
                name: "SensitivityToAct".to_string(),
                values: vec!["LOW".to_string(), "MEDIUM".to_string(), "HIGH".to_string()],
                identity: Some(carina_core::schema::string_enum_identity("SensitivityToAct", Some("awscc.wafv2.WebAcl"))),
                dsl_aliases: vec![("LOW".to_string(), "low".to_string()), ("MEDIUM".to_string(), "medium".to_string()), ("HIGH".to_string(), "high".to_string())],
            }).with_provider_name("Sensitivity"),
                    StructField::new("usage_of_action", AttributeType::StringEnum {
                name: "UsageOfAction".to_string(),
                values: vec!["ENABLED".to_string(), "DISABLED".to_string()],
                identity: Some(carina_core::schema::string_enum_identity("UsageOfAction", Some("awscc.wafv2.WebAcl"))),
                dsl_aliases: vec![("ENABLED".to_string(), "enabled".to_string()), ("DISABLED".to_string(), "disabled".to_string())],
            }).required().with_provider_name("UsageOfAction")
                    ],
                })
        .with_def("ClientSideActionConfig", AttributeType::Struct {
                    name: "ClientSideActionConfig".to_string(),
                    fields: vec![
                    StructField::new("challenge", AttributeType::Struct {
                    name: "ClientSideAction".to_string(),
                    fields: vec![
                    StructField::new("exempt_uri_regular_expressions", AttributeType::list(AttributeType::Struct {
                    name: "Regex".to_string(),
                    fields: vec![
                    StructField::new("regex_string", AttributeType::String).with_provider_name("RegexString")
                    ],
                })).with_provider_name("ExemptUriRegularExpressions").with_block_name("exempt_uri_regular_expression"),
                    StructField::new("sensitivity", AttributeType::StringEnum {
                name: "SensitivityToAct".to_string(),
                values: vec!["LOW".to_string(), "MEDIUM".to_string(), "HIGH".to_string()],
                identity: Some(carina_core::schema::string_enum_identity("SensitivityToAct", Some("awscc.wafv2.WebAcl"))),
                dsl_aliases: vec![("LOW".to_string(), "low".to_string()), ("MEDIUM".to_string(), "medium".to_string()), ("HIGH".to_string(), "high".to_string())],
            }).with_provider_name("Sensitivity"),
                    StructField::new("usage_of_action", AttributeType::StringEnum {
                name: "UsageOfAction".to_string(),
                values: vec!["ENABLED".to_string(), "DISABLED".to_string()],
                identity: Some(carina_core::schema::string_enum_identity("UsageOfAction", Some("awscc.wafv2.WebAcl"))),
                dsl_aliases: vec![("ENABLED".to_string(), "enabled".to_string()), ("DISABLED".to_string(), "disabled".to_string())],
            }).required().with_provider_name("UsageOfAction")
                    ],
                }).required().with_provider_name("Challenge").with_block_name("challenge")
                    ],
                })
        .with_def("CookieMatchPattern", AttributeType::Struct {
                    name: "CookieMatchPattern".to_string(),
                    fields: vec![
                    StructField::new("all", AttributeType::map(AttributeType::String)).with_description("Inspect all parts of the web request cookies.").with_provider_name("All"),
                    StructField::new("excluded_cookies", AttributeType::Custom {
                identity: None,
                pattern: None,
                length: None,
                base: Box::new(AttributeType::list(AttributeType::Custom {
                identity: None,
                pattern: Some(".*\\S.*".to_string()),
                length: Some((Some(1), Some(60))),
                base: Box::new(AttributeType::String),
                validate: legacy_validator(validate_string_pattern_4e09c821aed9e752_len_1_60),
                to_dsl: None,
            })),
                validate: legacy_validator(validate_list_items_1_199),
                to_dsl: None,
            }).with_provider_name("ExcludedCookies"),
                    StructField::new("included_cookies", AttributeType::Custom {
                identity: None,
                pattern: None,
                length: None,
                base: Box::new(AttributeType::list(AttributeType::Custom {
                identity: None,
                pattern: Some(".*\\S.*".to_string()),
                length: Some((Some(1), Some(60))),
                base: Box::new(AttributeType::String),
                validate: legacy_validator(validate_string_pattern_4e09c821aed9e752_len_1_60),
                to_dsl: None,
            })),
                validate: legacy_validator(validate_list_items_1_199),
                to_dsl: None,
            }).with_provider_name("IncludedCookies")
                    ],
                })
        .with_def("Cookies", AttributeType::Struct {
                    name: "Cookies".to_string(),
                    fields: vec![
                    StructField::new("match_pattern", AttributeType::Struct {
                    name: "CookieMatchPattern".to_string(),
                    fields: vec![
                    StructField::new("all", AttributeType::map(AttributeType::String)).with_description("Inspect all parts of the web request cookies.").with_provider_name("All"),
                    StructField::new("excluded_cookies", AttributeType::Custom {
                identity: None,
                pattern: None,
                length: None,
                base: Box::new(AttributeType::list(AttributeType::Custom {
                identity: None,
                pattern: Some(".*\\S.*".to_string()),
                length: Some((Some(1), Some(60))),
                base: Box::new(AttributeType::String),
                validate: legacy_validator(validate_string_pattern_4e09c821aed9e752_len_1_60),
                to_dsl: None,
            })),
                validate: legacy_validator(validate_list_items_1_199),
                to_dsl: None,
            }).with_provider_name("ExcludedCookies"),
                    StructField::new("included_cookies", AttributeType::Custom {
                identity: None,
                pattern: None,
                length: None,
                base: Box::new(AttributeType::list(AttributeType::Custom {
                identity: None,
                pattern: Some(".*\\S.*".to_string()),
                length: Some((Some(1), Some(60))),
                base: Box::new(AttributeType::String),
                validate: legacy_validator(validate_string_pattern_4e09c821aed9e752_len_1_60),
                to_dsl: None,
            })),
                validate: legacy_validator(validate_list_items_1_199),
                to_dsl: None,
            }).with_provider_name("IncludedCookies")
                    ],
                }).required().with_provider_name("MatchPattern"),
                    StructField::new("match_scope", AttributeType::StringEnum {
                name: "MapMatchScope".to_string(),
                values: vec!["ALL".to_string(), "KEY".to_string(), "VALUE".to_string()],
                identity: Some(carina_core::schema::string_enum_identity("MapMatchScope", Some("awscc.wafv2.WebAcl"))),
                dsl_aliases: vec![("ALL".to_string(), "all".to_string()), ("KEY".to_string(), "key".to_string()), ("VALUE".to_string(), "value".to_string())],
            }).required().with_provider_name("MatchScope"),
                    StructField::new("oversize_handling", AttributeType::StringEnum {
                name: "OversizeHandling".to_string(),
                values: vec!["CONTINUE".to_string(), "MATCH".to_string(), "NO_MATCH".to_string()],
                identity: Some(carina_core::schema::string_enum_identity("OversizeHandling", Some("awscc.wafv2.WebAcl"))),
                dsl_aliases: vec![("CONTINUE".to_string(), "continue".to_string()), ("MATCH".to_string(), "match".to_string()), ("NO_MATCH".to_string(), "no_match".to_string())],
            }).required().with_provider_name("OversizeHandling")
                    ],
                })
        .with_def("CountAction", AttributeType::Struct {
                    name: "CountAction".to_string(),
                    fields: vec![
                    StructField::new("custom_request_handling", AttributeType::Ref("CustomRequestHandling".to_string())).with_provider_name("CustomRequestHandling")
                    ],
                })
        .with_def("CustomRequestHandling", AttributeType::Struct {
                    name: "CustomRequestHandling".to_string(),
                    fields: vec![
                    StructField::new("insert_headers", AttributeType::Custom {
                identity: None,
                pattern: None,
                length: None,
                base: Box::new(AttributeType::list(AttributeType::Struct {
                    name: "CustomHTTPHeader".to_string(),
                    fields: vec![
                    StructField::new("name", AttributeType::String).required().with_provider_name("Name"),
                    StructField::new("value", AttributeType::String).required().with_provider_name("Value")
                    ],
                })),
                validate: legacy_validator(validate_list_items_min_1),
                to_dsl: None,
            }).required().with_description("Collection of HTTP headers.").with_provider_name("InsertHeaders").with_block_name("insert_header")
                    ],
                })
        .with_def("CustomResponse", AttributeType::Struct {
                    name: "CustomResponse".to_string(),
                    fields: vec![
                    StructField::new("custom_response_body_key", AttributeType::Custom {
                identity: None,
                pattern: Some("^[\\w\\-]+$".to_string()),
                length: None,
                base: Box::new(AttributeType::String),
                validate: legacy_validator(validate_string_pattern_6fd4a12c0ce64c08),
                to_dsl: None,
            }).with_description("Custom response body key.").with_provider_name("CustomResponseBodyKey"),
                    StructField::new("response_code", AttributeType::String).required().with_provider_name("ResponseCode"),
                    StructField::new("response_headers", AttributeType::Custom {
                identity: None,
                pattern: None,
                length: None,
                base: Box::new(AttributeType::list(AttributeType::Struct {
                    name: "CustomHTTPHeader".to_string(),
                    fields: vec![
                    StructField::new("name", AttributeType::String).required().with_provider_name("Name"),
                    StructField::new("value", AttributeType::String).required().with_provider_name("Value")
                    ],
                })),
                validate: legacy_validator(validate_list_items_min_1),
                to_dsl: None,
            }).with_description("Collection of HTTP headers.").with_provider_name("ResponseHeaders").with_block_name("response_header")
                    ],
                })
        .with_def("DataProtect", AttributeType::Struct {
                    name: "DataProtect".to_string(),
                    fields: vec![
                    StructField::new("action", AttributeType::StringEnum {
                name: "DataProtectionAction".to_string(),
                values: vec!["SUBSTITUTION".to_string(), "HASH".to_string()],
                identity: Some(carina_core::schema::string_enum_identity("DataProtectionAction", Some("awscc.wafv2.WebAcl"))),
                dsl_aliases: vec![("SUBSTITUTION".to_string(), "substitution".to_string()), ("HASH".to_string(), "hash".to_string())],
            }).required().with_provider_name("Action"),
                    StructField::new("exclude_rate_based_details", AttributeType::Bool).with_provider_name("ExcludeRateBasedDetails"),
                    StructField::new("exclude_rule_match_details", AttributeType::Bool).with_provider_name("ExcludeRuleMatchDetails"),
                    StructField::new("field", AttributeType::Struct {
                    name: "FieldToProtect".to_string(),
                    fields: vec![
                    StructField::new("field_keys", AttributeType::list(AttributeType::String)).with_description("List of field keys to protect").with_provider_name("FieldKeys"),
                    StructField::new("field_type", AttributeType::StringEnum {
                name: "FieldType".to_string(),
                values: vec!["SINGLE_HEADER".to_string(), "SINGLE_COOKIE".to_string(), "SINGLE_QUERY_ARGUMENT".to_string(), "QUERY_STRING".to_string(), "BODY".to_string()],
                identity: Some(carina_core::schema::string_enum_identity("FieldType", Some("awscc.wafv2.WebAcl"))),
                dsl_aliases: vec![("SINGLE_HEADER".to_string(), "single_header".to_string()), ("SINGLE_COOKIE".to_string(), "single_cookie".to_string()), ("SINGLE_QUERY_ARGUMENT".to_string(), "single_query_argument".to_string()), ("QUERY_STRING".to_string(), "query_string".to_string()), ("BODY".to_string(), "body".to_string())],
            }).required().with_description("Field type to protect").with_provider_name("FieldType")
                    ],
                }).required().with_provider_name("Field")
                    ],
                })
        .with_def("DataProtectionConfig", AttributeType::Struct {
                    name: "DataProtectionConfig".to_string(),
                    fields: vec![
                    StructField::new("data_protections", AttributeType::list(AttributeType::Struct {
                    name: "DataProtect".to_string(),
                    fields: vec![
                    StructField::new("action", AttributeType::StringEnum {
                name: "DataProtectionAction".to_string(),
                values: vec!["SUBSTITUTION".to_string(), "HASH".to_string()],
                identity: Some(carina_core::schema::string_enum_identity("DataProtectionAction", Some("awscc.wafv2.WebAcl"))),
                dsl_aliases: vec![("SUBSTITUTION".to_string(), "substitution".to_string()), ("HASH".to_string(), "hash".to_string())],
            }).required().with_provider_name("Action"),
                    StructField::new("exclude_rate_based_details", AttributeType::Bool).with_provider_name("ExcludeRateBasedDetails"),
                    StructField::new("exclude_rule_match_details", AttributeType::Bool).with_provider_name("ExcludeRuleMatchDetails"),
                    StructField::new("field", AttributeType::Struct {
                    name: "FieldToProtect".to_string(),
                    fields: vec![
                    StructField::new("field_keys", AttributeType::list(AttributeType::String)).with_description("List of field keys to protect").with_provider_name("FieldKeys"),
                    StructField::new("field_type", AttributeType::StringEnum {
                name: "FieldType".to_string(),
                values: vec!["SINGLE_HEADER".to_string(), "SINGLE_COOKIE".to_string(), "SINGLE_QUERY_ARGUMENT".to_string(), "QUERY_STRING".to_string(), "BODY".to_string()],
                identity: Some(carina_core::schema::string_enum_identity("FieldType", Some("awscc.wafv2.WebAcl"))),
                dsl_aliases: vec![("SINGLE_HEADER".to_string(), "single_header".to_string()), ("SINGLE_COOKIE".to_string(), "single_cookie".to_string()), ("SINGLE_QUERY_ARGUMENT".to_string(), "single_query_argument".to_string()), ("QUERY_STRING".to_string(), "query_string".to_string()), ("BODY".to_string(), "body".to_string())],
            }).required().with_description("Field type to protect").with_provider_name("FieldType")
                    ],
                }).required().with_provider_name("Field")
                    ],
                })).required().with_provider_name("DataProtections").with_block_name("data_protection")
                    ],
                })
        .with_def("DefaultAction", AttributeType::Struct {
                    name: "DefaultAction".to_string(),
                    fields: vec![
                    StructField::new("allow", AttributeType::Struct {
                    name: "AllowAction".to_string(),
                    fields: vec![
                    StructField::new("custom_request_handling", AttributeType::Struct {
                    name: "CustomRequestHandling".to_string(),
                    fields: vec![
                    StructField::new("insert_headers", AttributeType::Custom {
                identity: None,
                pattern: None,
                length: None,
                base: Box::new(AttributeType::list(AttributeType::Struct {
                    name: "CustomHTTPHeader".to_string(),
                    fields: vec![
                    StructField::new("name", AttributeType::String).required().with_provider_name("Name"),
                    StructField::new("value", AttributeType::String).required().with_provider_name("Value")
                    ],
                })),
                validate: legacy_validator(validate_list_items_min_1),
                to_dsl: None,
            }).required().with_description("Collection of HTTP headers.").with_provider_name("InsertHeaders").with_block_name("insert_header")
                    ],
                }).with_provider_name("CustomRequestHandling").with_block_name("custom_request_handling")
                    ],
                }).with_provider_name("Allow").with_block_name("allow"),
                    StructField::new("block", AttributeType::Struct {
                    name: "BlockAction".to_string(),
                    fields: vec![
                    StructField::new("custom_response", AttributeType::Struct {
                    name: "CustomResponse".to_string(),
                    fields: vec![
                    StructField::new("custom_response_body_key", AttributeType::Custom {
                identity: None,
                pattern: Some("^[\\w\\-]+$".to_string()),
                length: None,
                base: Box::new(AttributeType::String),
                validate: legacy_validator(validate_string_pattern_6fd4a12c0ce64c08),
                to_dsl: None,
            }).with_description("Custom response body key.").with_provider_name("CustomResponseBodyKey"),
                    StructField::new("response_code", AttributeType::String).required().with_provider_name("ResponseCode"),
                    StructField::new("response_headers", AttributeType::Custom {
                identity: None,
                pattern: None,
                length: None,
                base: Box::new(AttributeType::list(AttributeType::Struct {
                    name: "CustomHTTPHeader".to_string(),
                    fields: vec![
                    StructField::new("name", AttributeType::String).required().with_provider_name("Name"),
                    StructField::new("value", AttributeType::String).required().with_provider_name("Value")
                    ],
                })),
                validate: legacy_validator(validate_list_items_min_1),
                to_dsl: None,
            }).with_description("Collection of HTTP headers.").with_provider_name("ResponseHeaders").with_block_name("response_header")
                    ],
                }).with_provider_name("CustomResponse").with_block_name("custom_response")
                    ],
                }).with_provider_name("Block").with_block_name("block")
                    ],
                })
        .with_def("FieldIdentifier", AttributeType::Struct {
                    name: "FieldIdentifier".to_string(),
                    fields: vec![
                    StructField::new("identifier", AttributeType::Custom {
                identity: None,
                pattern: Some(".*\\S.*".to_string()),
                length: Some((Some(1), Some(512))),
                base: Box::new(AttributeType::String),
                validate: legacy_validator(validate_string_pattern_4e09c821aed9e752_len_1_512),
                to_dsl: None,
            }).required().with_provider_name("Identifier")
                    ],
                })
        .with_def("FieldToMatch", AttributeType::Struct {
                    name: "FieldToMatch".to_string(),
                    fields: vec![
                    StructField::new("all_query_arguments", AttributeType::map(AttributeType::String)).with_description("All query arguments of a web request.").with_provider_name("AllQueryArguments"),
                    StructField::new("body", AttributeType::Struct {
                    name: "Body".to_string(),
                    fields: vec![
                    StructField::new("oversize_handling", AttributeType::StringEnum {
                name: "OversizeHandling".to_string(),
                values: vec!["CONTINUE".to_string(), "MATCH".to_string(), "NO_MATCH".to_string()],
                identity: Some(carina_core::schema::string_enum_identity("OversizeHandling", Some("awscc.wafv2.WebAcl"))),
                dsl_aliases: vec![("CONTINUE".to_string(), "continue".to_string()), ("MATCH".to_string(), "match".to_string()), ("NO_MATCH".to_string(), "no_match".to_string())],
            }).with_provider_name("OversizeHandling")
                    ],
                }).with_provider_name("Body"),
                    StructField::new("cookies", AttributeType::Struct {
                    name: "Cookies".to_string(),
                    fields: vec![
                    StructField::new("match_pattern", AttributeType::Struct {
                    name: "CookieMatchPattern".to_string(),
                    fields: vec![
                    StructField::new("all", AttributeType::map(AttributeType::String)).with_description("Inspect all parts of the web request cookies.").with_provider_name("All"),
                    StructField::new("excluded_cookies", AttributeType::Custom {
                identity: None,
                pattern: None,
                length: None,
                base: Box::new(AttributeType::list(AttributeType::Custom {
                identity: None,
                pattern: Some(".*\\S.*".to_string()),
                length: Some((Some(1), Some(60))),
                base: Box::new(AttributeType::String),
                validate: legacy_validator(validate_string_pattern_4e09c821aed9e752_len_1_60),
                to_dsl: None,
            })),
                validate: legacy_validator(validate_list_items_1_199),
                to_dsl: None,
            }).with_provider_name("ExcludedCookies"),
                    StructField::new("included_cookies", AttributeType::Custom {
                identity: None,
                pattern: None,
                length: None,
                base: Box::new(AttributeType::list(AttributeType::Custom {
                identity: None,
                pattern: Some(".*\\S.*".to_string()),
                length: Some((Some(1), Some(60))),
                base: Box::new(AttributeType::String),
                validate: legacy_validator(validate_string_pattern_4e09c821aed9e752_len_1_60),
                to_dsl: None,
            })),
                validate: legacy_validator(validate_list_items_1_199),
                to_dsl: None,
            }).with_provider_name("IncludedCookies")
                    ],
                }).required().with_provider_name("MatchPattern"),
                    StructField::new("match_scope", AttributeType::StringEnum {
                name: "MapMatchScope".to_string(),
                values: vec!["ALL".to_string(), "KEY".to_string(), "VALUE".to_string()],
                identity: Some(carina_core::schema::string_enum_identity("MapMatchScope", Some("awscc.wafv2.WebAcl"))),
                dsl_aliases: vec![("ALL".to_string(), "all".to_string()), ("KEY".to_string(), "key".to_string()), ("VALUE".to_string(), "value".to_string())],
            }).required().with_provider_name("MatchScope"),
                    StructField::new("oversize_handling", AttributeType::StringEnum {
                name: "OversizeHandling".to_string(),
                values: vec!["CONTINUE".to_string(), "MATCH".to_string(), "NO_MATCH".to_string()],
                identity: Some(carina_core::schema::string_enum_identity("OversizeHandling", Some("awscc.wafv2.WebAcl"))),
                dsl_aliases: vec![("CONTINUE".to_string(), "continue".to_string()), ("MATCH".to_string(), "match".to_string()), ("NO_MATCH".to_string(), "no_match".to_string())],
            }).required().with_provider_name("OversizeHandling")
                    ],
                }).with_provider_name("Cookies"),
                    StructField::new("header_order", AttributeType::Struct {
                    name: "HeaderOrder".to_string(),
                    fields: vec![
                    StructField::new("oversize_handling", AttributeType::StringEnum {
                name: "OversizeHandling".to_string(),
                values: vec!["CONTINUE".to_string(), "MATCH".to_string(), "NO_MATCH".to_string()],
                identity: Some(carina_core::schema::string_enum_identity("OversizeHandling", Some("awscc.wafv2.WebAcl"))),
                dsl_aliases: vec![("CONTINUE".to_string(), "continue".to_string()), ("MATCH".to_string(), "match".to_string()), ("NO_MATCH".to_string(), "no_match".to_string())],
            }).required().with_provider_name("OversizeHandling")
                    ],
                }).with_provider_name("HeaderOrder"),
                    StructField::new("headers", AttributeType::Struct {
                    name: "Headers".to_string(),
                    fields: vec![
                    StructField::new("match_pattern", AttributeType::Struct {
                    name: "HeaderMatchPattern".to_string(),
                    fields: vec![
                    StructField::new("all", AttributeType::map(AttributeType::String)).with_description("Inspect all parts of the web request headers.").with_provider_name("All"),
                    StructField::new("excluded_headers", AttributeType::Custom {
                identity: None,
                pattern: None,
                length: None,
                base: Box::new(AttributeType::list(AttributeType::Custom {
                identity: None,
                pattern: Some(".*\\S.*".to_string()),
                length: Some((Some(1), Some(64))),
                base: Box::new(AttributeType::String),
                validate: legacy_validator(validate_string_pattern_4e09c821aed9e752_len_1_64),
                to_dsl: None,
            })),
                validate: legacy_validator(validate_list_items_1_199),
                to_dsl: None,
            }).with_provider_name("ExcludedHeaders"),
                    StructField::new("included_headers", AttributeType::Custom {
                identity: None,
                pattern: None,
                length: None,
                base: Box::new(AttributeType::list(AttributeType::Custom {
                identity: None,
                pattern: Some(".*\\S.*".to_string()),
                length: Some((Some(1), Some(64))),
                base: Box::new(AttributeType::String),
                validate: legacy_validator(validate_string_pattern_4e09c821aed9e752_len_1_64),
                to_dsl: None,
            })),
                validate: legacy_validator(validate_list_items_1_199),
                to_dsl: None,
            }).with_provider_name("IncludedHeaders")
                    ],
                }).required().with_provider_name("MatchPattern"),
                    StructField::new("match_scope", AttributeType::StringEnum {
                name: "MapMatchScope".to_string(),
                values: vec!["ALL".to_string(), "KEY".to_string(), "VALUE".to_string()],
                identity: Some(carina_core::schema::string_enum_identity("MapMatchScope", Some("awscc.wafv2.WebAcl"))),
                dsl_aliases: vec![("ALL".to_string(), "all".to_string()), ("KEY".to_string(), "key".to_string()), ("VALUE".to_string(), "value".to_string())],
            }).required().with_provider_name("MatchScope"),
                    StructField::new("oversize_handling", AttributeType::StringEnum {
                name: "OversizeHandling".to_string(),
                values: vec!["CONTINUE".to_string(), "MATCH".to_string(), "NO_MATCH".to_string()],
                identity: Some(carina_core::schema::string_enum_identity("OversizeHandling", Some("awscc.wafv2.WebAcl"))),
                dsl_aliases: vec![("CONTINUE".to_string(), "continue".to_string()), ("MATCH".to_string(), "match".to_string()), ("NO_MATCH".to_string(), "no_match".to_string())],
            }).required().with_provider_name("OversizeHandling")
                    ],
                }).with_provider_name("Headers"),
                    StructField::new("ja3_fingerprint", AttributeType::Struct {
                    name: "JA3Fingerprint".to_string(),
                    fields: vec![
                    StructField::new("fallback_behavior", AttributeType::StringEnum {
                name: "FallbackBehavior".to_string(),
                values: vec!["MATCH".to_string(), "NO_MATCH".to_string()],
                identity: Some(carina_core::schema::string_enum_identity("FallbackBehavior", Some("awscc.wafv2.WebAcl"))),
                dsl_aliases: vec![("MATCH".to_string(), "match".to_string()), ("NO_MATCH".to_string(), "no_match".to_string())],
            }).required().with_provider_name("FallbackBehavior")
                    ],
                }).with_provider_name("JA3Fingerprint"),
                    StructField::new("ja4_fingerprint", AttributeType::Struct {
                    name: "JA4Fingerprint".to_string(),
                    fields: vec![
                    StructField::new("fallback_behavior", AttributeType::StringEnum {
                name: "FallbackBehavior".to_string(),
                values: vec!["MATCH".to_string(), "NO_MATCH".to_string()],
                identity: Some(carina_core::schema::string_enum_identity("FallbackBehavior", Some("awscc.wafv2.WebAcl"))),
                dsl_aliases: vec![("MATCH".to_string(), "match".to_string()), ("NO_MATCH".to_string(), "no_match".to_string())],
            }).required().with_provider_name("FallbackBehavior")
                    ],
                }).with_provider_name("JA4Fingerprint"),
                    StructField::new("json_body", AttributeType::Struct {
                    name: "JsonBody".to_string(),
                    fields: vec![
                    StructField::new("invalid_fallback_behavior", AttributeType::StringEnum {
                name: "BodyParsingFallbackBehavior".to_string(),
                values: vec!["MATCH".to_string(), "NO_MATCH".to_string(), "EVALUATE_AS_STRING".to_string()],
                identity: Some(carina_core::schema::string_enum_identity("BodyParsingFallbackBehavior", Some("awscc.wafv2.WebAcl"))),
                dsl_aliases: vec![("MATCH".to_string(), "match".to_string()), ("NO_MATCH".to_string(), "no_match".to_string()), ("EVALUATE_AS_STRING".to_string(), "evaluate_as_string".to_string())],
            }).with_provider_name("InvalidFallbackBehavior"),
                    StructField::new("match_pattern", AttributeType::Struct {
                    name: "JsonMatchPattern".to_string(),
                    fields: vec![
                    StructField::new("all", AttributeType::map(AttributeType::String)).with_description("Inspect all parts of the web request's JSON body.").with_provider_name("All"),
                    StructField::new("included_paths", AttributeType::list(AttributeType::Custom {
                identity: None,
                pattern: Some("^[\\/]+([^~]*(~[01])*)*{1,512}$".to_string()),
                length: None,
                base: Box::new(AttributeType::String),
                validate: legacy_validator(validate_string_pattern_c77cf75cf1a75ade),
                to_dsl: None,
            })).with_provider_name("IncludedPaths")
                    ],
                }).required().with_provider_name("MatchPattern"),
                    StructField::new("match_scope", AttributeType::StringEnum {
                name: "JsonMatchScope".to_string(),
                values: vec!["ALL".to_string(), "KEY".to_string(), "VALUE".to_string()],
                identity: Some(carina_core::schema::string_enum_identity("JsonMatchScope", Some("awscc.wafv2.WebAcl"))),
                dsl_aliases: vec![("ALL".to_string(), "all".to_string()), ("KEY".to_string(), "key".to_string()), ("VALUE".to_string(), "value".to_string())],
            }).required().with_provider_name("MatchScope"),
                    StructField::new("oversize_handling", AttributeType::StringEnum {
                name: "OversizeHandling".to_string(),
                values: vec!["CONTINUE".to_string(), "MATCH".to_string(), "NO_MATCH".to_string()],
                identity: Some(carina_core::schema::string_enum_identity("OversizeHandling", Some("awscc.wafv2.WebAcl"))),
                dsl_aliases: vec![("CONTINUE".to_string(), "continue".to_string()), ("MATCH".to_string(), "match".to_string()), ("NO_MATCH".to_string(), "no_match".to_string())],
            }).with_provider_name("OversizeHandling")
                    ],
                }).with_provider_name("JsonBody"),
                    StructField::new("method", AttributeType::map(AttributeType::String)).with_description("The HTTP method of a web request. The method indicates the type of operation that the request is asking the origin to perform.").with_provider_name("Method"),
                    StructField::new("query_string", AttributeType::map(AttributeType::String)).with_description("The query string of a web request. This is the part of a URL that appears after a ? character, if any.").with_provider_name("QueryString"),
                    StructField::new("single_header", AttributeType::Struct {
                    name: "SingleHeader".to_string(),
                    fields: vec![
                    StructField::new("name", AttributeType::String).required().with_provider_name("Name")
                    ],
                }).with_provider_name("SingleHeader"),
                    StructField::new("single_query_argument", AttributeType::Struct {
                    name: "SingleQueryArgument".to_string(),
                    fields: vec![
                    StructField::new("name", AttributeType::String).required().with_provider_name("Name")
                    ],
                }).with_description("One query argument in a web request, identified by name, for example UserName or SalesRegion. The name can be up to 30 characters long and isn't case sensitive.").with_provider_name("SingleQueryArgument"),
                    StructField::new("uri_fragment", AttributeType::Struct {
                    name: "UriFragment".to_string(),
                    fields: vec![
                    StructField::new("fallback_behavior", AttributeType::StringEnum {
                name: "FallbackBehavior".to_string(),
                values: vec!["MATCH".to_string(), "NO_MATCH".to_string()],
                identity: Some(carina_core::schema::string_enum_identity("FallbackBehavior", Some("awscc.wafv2.WebAcl"))),
                dsl_aliases: vec![("MATCH".to_string(), "match".to_string()), ("NO_MATCH".to_string(), "no_match".to_string())],
            }).with_provider_name("FallbackBehavior")
                    ],
                }).with_provider_name("UriFragment"),
                    StructField::new("uri_path", AttributeType::map(AttributeType::String)).with_description("The path component of the URI of a web request. This is the part of a web request that identifies a resource, for example, /images/daily-ad.jpg.").with_provider_name("UriPath")
                    ],
                })
        .with_def("FieldToProtect", AttributeType::Struct {
                    name: "FieldToProtect".to_string(),
                    fields: vec![
                    StructField::new("field_keys", AttributeType::list(AttributeType::String)).with_description("List of field keys to protect").with_provider_name("FieldKeys"),
                    StructField::new("field_type", AttributeType::StringEnum {
                name: "FieldType".to_string(),
                values: vec!["SINGLE_HEADER".to_string(), "SINGLE_COOKIE".to_string(), "SINGLE_QUERY_ARGUMENT".to_string(), "QUERY_STRING".to_string(), "BODY".to_string()],
                identity: Some(carina_core::schema::string_enum_identity("FieldType", Some("awscc.wafv2.WebAcl"))),
                dsl_aliases: vec![("SINGLE_HEADER".to_string(), "single_header".to_string()), ("SINGLE_COOKIE".to_string(), "single_cookie".to_string()), ("SINGLE_QUERY_ARGUMENT".to_string(), "single_query_argument".to_string()), ("QUERY_STRING".to_string(), "query_string".to_string()), ("BODY".to_string(), "body".to_string())],
            }).required().with_description("Field type to protect").with_provider_name("FieldType")
                    ],
                })
        .with_def("ForwardedIPConfiguration", AttributeType::Struct {
                    name: "ForwardedIPConfiguration".to_string(),
                    fields: vec![
                    StructField::new("fallback_behavior", AttributeType::StringEnum {
                name: "FallbackBehavior".to_string(),
                values: vec!["MATCH".to_string(), "NO_MATCH".to_string()],
                identity: Some(carina_core::schema::string_enum_identity("FallbackBehavior", Some("awscc.wafv2.WebAcl"))),
                dsl_aliases: vec![("MATCH".to_string(), "match".to_string()), ("NO_MATCH".to_string(), "no_match".to_string())],
            }).required().with_provider_name("FallbackBehavior"),
                    StructField::new("header_name", AttributeType::Custom {
                identity: None,
                pattern: Some("^[a-zA-Z0-9-]+{1,255}$".to_string()),
                length: None,
                base: Box::new(AttributeType::String),
                validate: legacy_validator(validate_string_pattern_2d93cc844f6d4014),
                to_dsl: None,
            }).required().with_provider_name("HeaderName")
                    ],
                })
        .with_def("GeoMatchStatement", AttributeType::Struct {
                    name: "GeoMatchStatement".to_string(),
                    fields: vec![
                    StructField::new("country_codes", AttributeType::list(AttributeType::Custom {
                identity: None,
                pattern: None,
                length: Some((Some(1), Some(2))),
                base: Box::new(AttributeType::String),
                validate: legacy_validator(validate_string_length_1_2),
                to_dsl: None,
            })).with_provider_name("CountryCodes"),
                    StructField::new("forwarded_ip_config", AttributeType::Ref("ForwardedIPConfiguration".to_string())).with_provider_name("ForwardedIPConfig")
                    ],
                })
        .with_def("HeaderMatchPattern", AttributeType::Struct {
                    name: "HeaderMatchPattern".to_string(),
                    fields: vec![
                    StructField::new("all", AttributeType::map(AttributeType::String)).with_description("Inspect all parts of the web request headers.").with_provider_name("All"),
                    StructField::new("excluded_headers", AttributeType::Custom {
                identity: None,
                pattern: None,
                length: None,
                base: Box::new(AttributeType::list(AttributeType::Custom {
                identity: None,
                pattern: Some(".*\\S.*".to_string()),
                length: Some((Some(1), Some(64))),
                base: Box::new(AttributeType::String),
                validate: legacy_validator(validate_string_pattern_4e09c821aed9e752_len_1_64),
                to_dsl: None,
            })),
                validate: legacy_validator(validate_list_items_1_199),
                to_dsl: None,
            }).with_provider_name("ExcludedHeaders"),
                    StructField::new("included_headers", AttributeType::Custom {
                identity: None,
                pattern: None,
                length: None,
                base: Box::new(AttributeType::list(AttributeType::Custom {
                identity: None,
                pattern: Some(".*\\S.*".to_string()),
                length: Some((Some(1), Some(64))),
                base: Box::new(AttributeType::String),
                validate: legacy_validator(validate_string_pattern_4e09c821aed9e752_len_1_64),
                to_dsl: None,
            })),
                validate: legacy_validator(validate_list_items_1_199),
                to_dsl: None,
            }).with_provider_name("IncludedHeaders")
                    ],
                })
        .with_def("HeaderOrder", AttributeType::Struct {
                    name: "HeaderOrder".to_string(),
                    fields: vec![
                    StructField::new("oversize_handling", AttributeType::StringEnum {
                name: "OversizeHandling".to_string(),
                values: vec!["CONTINUE".to_string(), "MATCH".to_string(), "NO_MATCH".to_string()],
                identity: Some(carina_core::schema::string_enum_identity("OversizeHandling", Some("awscc.wafv2.WebAcl"))),
                dsl_aliases: vec![("CONTINUE".to_string(), "continue".to_string()), ("MATCH".to_string(), "match".to_string()), ("NO_MATCH".to_string(), "no_match".to_string())],
            }).required().with_provider_name("OversizeHandling")
                    ],
                })
        .with_def("Headers", AttributeType::Struct {
                    name: "Headers".to_string(),
                    fields: vec![
                    StructField::new("match_pattern", AttributeType::Struct {
                    name: "HeaderMatchPattern".to_string(),
                    fields: vec![
                    StructField::new("all", AttributeType::map(AttributeType::String)).with_description("Inspect all parts of the web request headers.").with_provider_name("All"),
                    StructField::new("excluded_headers", AttributeType::Custom {
                identity: None,
                pattern: None,
                length: None,
                base: Box::new(AttributeType::list(AttributeType::Custom {
                identity: None,
                pattern: Some(".*\\S.*".to_string()),
                length: Some((Some(1), Some(64))),
                base: Box::new(AttributeType::String),
                validate: legacy_validator(validate_string_pattern_4e09c821aed9e752_len_1_64),
                to_dsl: None,
            })),
                validate: legacy_validator(validate_list_items_1_199),
                to_dsl: None,
            }).with_provider_name("ExcludedHeaders"),
                    StructField::new("included_headers", AttributeType::Custom {
                identity: None,
                pattern: None,
                length: None,
                base: Box::new(AttributeType::list(AttributeType::Custom {
                identity: None,
                pattern: Some(".*\\S.*".to_string()),
                length: Some((Some(1), Some(64))),
                base: Box::new(AttributeType::String),
                validate: legacy_validator(validate_string_pattern_4e09c821aed9e752_len_1_64),
                to_dsl: None,
            })),
                validate: legacy_validator(validate_list_items_1_199),
                to_dsl: None,
            }).with_provider_name("IncludedHeaders")
                    ],
                }).required().with_provider_name("MatchPattern"),
                    StructField::new("match_scope", AttributeType::StringEnum {
                name: "MapMatchScope".to_string(),
                values: vec!["ALL".to_string(), "KEY".to_string(), "VALUE".to_string()],
                identity: Some(carina_core::schema::string_enum_identity("MapMatchScope", Some("awscc.wafv2.WebAcl"))),
                dsl_aliases: vec![("ALL".to_string(), "all".to_string()), ("KEY".to_string(), "key".to_string()), ("VALUE".to_string(), "value".to_string())],
            }).required().with_provider_name("MatchScope"),
                    StructField::new("oversize_handling", AttributeType::StringEnum {
                name: "OversizeHandling".to_string(),
                values: vec!["CONTINUE".to_string(), "MATCH".to_string(), "NO_MATCH".to_string()],
                identity: Some(carina_core::schema::string_enum_identity("OversizeHandling", Some("awscc.wafv2.WebAcl"))),
                dsl_aliases: vec![("CONTINUE".to_string(), "continue".to_string()), ("MATCH".to_string(), "match".to_string()), ("NO_MATCH".to_string(), "no_match".to_string())],
            }).required().with_provider_name("OversizeHandling")
                    ],
                })
        .with_def("IPSetForwardedIPConfiguration", AttributeType::Struct {
                    name: "IPSetForwardedIPConfiguration".to_string(),
                    fields: vec![
                    StructField::new("fallback_behavior", AttributeType::StringEnum {
                name: "FallbackBehavior".to_string(),
                values: vec!["MATCH".to_string(), "NO_MATCH".to_string()],
                identity: Some(carina_core::schema::string_enum_identity("FallbackBehavior", Some("awscc.wafv2.WebAcl"))),
                dsl_aliases: vec![("MATCH".to_string(), "match".to_string()), ("NO_MATCH".to_string(), "no_match".to_string())],
            }).required().with_provider_name("FallbackBehavior"),
                    StructField::new("header_name", AttributeType::Custom {
                identity: None,
                pattern: Some("^[a-zA-Z0-9-]+{1,255}$".to_string()),
                length: None,
                base: Box::new(AttributeType::String),
                validate: legacy_validator(validate_string_pattern_2d93cc844f6d4014),
                to_dsl: None,
            }).required().with_provider_name("HeaderName"),
                    StructField::new("position", AttributeType::StringEnum {
                name: "Position".to_string(),
                values: vec!["FIRST".to_string(), "LAST".to_string(), "ANY".to_string()],
                identity: Some(carina_core::schema::string_enum_identity("Position", Some("awscc.wafv2.WebAcl"))),
                dsl_aliases: vec![("FIRST".to_string(), "first".to_string()), ("LAST".to_string(), "last".to_string()), ("ANY".to_string(), "any".to_string())],
            }).required().with_provider_name("Position")
                    ],
                })
        .with_def("IPSetReferenceStatement", AttributeType::Struct {
                    name: "IPSetReferenceStatement".to_string(),
                    fields: vec![
                    StructField::new("arn", super::arn()).required().with_provider_name("Arn"),
                    StructField::new("ip_set_forwarded_ip_config", AttributeType::Struct {
                    name: "IPSetForwardedIPConfiguration".to_string(),
                    fields: vec![
                    StructField::new("fallback_behavior", AttributeType::StringEnum {
                name: "FallbackBehavior".to_string(),
                values: vec!["MATCH".to_string(), "NO_MATCH".to_string()],
                identity: Some(carina_core::schema::string_enum_identity("FallbackBehavior", Some("awscc.wafv2.WebAcl"))),
                dsl_aliases: vec![("MATCH".to_string(), "match".to_string()), ("NO_MATCH".to_string(), "no_match".to_string())],
            }).required().with_provider_name("FallbackBehavior"),
                    StructField::new("header_name", AttributeType::Custom {
                identity: None,
                pattern: Some("^[a-zA-Z0-9-]+{1,255}$".to_string()),
                length: None,
                base: Box::new(AttributeType::String),
                validate: legacy_validator(validate_string_pattern_2d93cc844f6d4014),
                to_dsl: None,
            }).required().with_provider_name("HeaderName"),
                    StructField::new("position", AttributeType::StringEnum {
                name: "Position".to_string(),
                values: vec!["FIRST".to_string(), "LAST".to_string(), "ANY".to_string()],
                identity: Some(carina_core::schema::string_enum_identity("Position", Some("awscc.wafv2.WebAcl"))),
                dsl_aliases: vec![("FIRST".to_string(), "first".to_string()), ("LAST".to_string(), "last".to_string()), ("ANY".to_string(), "any".to_string())],
            }).required().with_provider_name("Position")
                    ],
                }).with_provider_name("IPSetForwardedIPConfig")
                    ],
                })
        .with_def("ImmunityTimeProperty", AttributeType::Struct {
                    name: "ImmunityTimeProperty".to_string(),
                    fields: vec![
                    StructField::new("immunity_time", AttributeType::Custom {
                identity: None,
                pattern: None,
                length: None,
                base: Box::new(AttributeType::Int),
                validate: legacy_validator(validate_immunity_time_range),
                to_dsl: None,
            }).required().with_provider_name("ImmunityTime")
                    ],
                })
        .with_def("JA3Fingerprint", AttributeType::Struct {
                    name: "JA3Fingerprint".to_string(),
                    fields: vec![
                    StructField::new("fallback_behavior", AttributeType::StringEnum {
                name: "FallbackBehavior".to_string(),
                values: vec!["MATCH".to_string(), "NO_MATCH".to_string()],
                identity: Some(carina_core::schema::string_enum_identity("FallbackBehavior", Some("awscc.wafv2.WebAcl"))),
                dsl_aliases: vec![("MATCH".to_string(), "match".to_string()), ("NO_MATCH".to_string(), "no_match".to_string())],
            }).required().with_provider_name("FallbackBehavior")
                    ],
                })
        .with_def("JA4Fingerprint", AttributeType::Struct {
                    name: "JA4Fingerprint".to_string(),
                    fields: vec![
                    StructField::new("fallback_behavior", AttributeType::StringEnum {
                name: "FallbackBehavior".to_string(),
                values: vec!["MATCH".to_string(), "NO_MATCH".to_string()],
                identity: Some(carina_core::schema::string_enum_identity("FallbackBehavior", Some("awscc.wafv2.WebAcl"))),
                dsl_aliases: vec![("MATCH".to_string(), "match".to_string()), ("NO_MATCH".to_string(), "no_match".to_string())],
            }).required().with_provider_name("FallbackBehavior")
                    ],
                })
        .with_def("JsonBody", AttributeType::Struct {
                    name: "JsonBody".to_string(),
                    fields: vec![
                    StructField::new("invalid_fallback_behavior", AttributeType::StringEnum {
                name: "BodyParsingFallbackBehavior".to_string(),
                values: vec!["MATCH".to_string(), "NO_MATCH".to_string(), "EVALUATE_AS_STRING".to_string()],
                identity: Some(carina_core::schema::string_enum_identity("BodyParsingFallbackBehavior", Some("awscc.wafv2.WebAcl"))),
                dsl_aliases: vec![("MATCH".to_string(), "match".to_string()), ("NO_MATCH".to_string(), "no_match".to_string()), ("EVALUATE_AS_STRING".to_string(), "evaluate_as_string".to_string())],
            }).with_provider_name("InvalidFallbackBehavior"),
                    StructField::new("match_pattern", AttributeType::Struct {
                    name: "JsonMatchPattern".to_string(),
                    fields: vec![
                    StructField::new("all", AttributeType::map(AttributeType::String)).with_description("Inspect all parts of the web request's JSON body.").with_provider_name("All"),
                    StructField::new("included_paths", AttributeType::list(AttributeType::Custom {
                identity: None,
                pattern: Some("^[\\/]+([^~]*(~[01])*)*{1,512}$".to_string()),
                length: None,
                base: Box::new(AttributeType::String),
                validate: legacy_validator(validate_string_pattern_c77cf75cf1a75ade),
                to_dsl: None,
            })).with_provider_name("IncludedPaths")
                    ],
                }).required().with_provider_name("MatchPattern"),
                    StructField::new("match_scope", AttributeType::StringEnum {
                name: "JsonMatchScope".to_string(),
                values: vec!["ALL".to_string(), "KEY".to_string(), "VALUE".to_string()],
                identity: Some(carina_core::schema::string_enum_identity("JsonMatchScope", Some("awscc.wafv2.WebAcl"))),
                dsl_aliases: vec![("ALL".to_string(), "all".to_string()), ("KEY".to_string(), "key".to_string()), ("VALUE".to_string(), "value".to_string())],
            }).required().with_provider_name("MatchScope"),
                    StructField::new("oversize_handling", AttributeType::StringEnum {
                name: "OversizeHandling".to_string(),
                values: vec!["CONTINUE".to_string(), "MATCH".to_string(), "NO_MATCH".to_string()],
                identity: Some(carina_core::schema::string_enum_identity("OversizeHandling", Some("awscc.wafv2.WebAcl"))),
                dsl_aliases: vec![("CONTINUE".to_string(), "continue".to_string()), ("MATCH".to_string(), "match".to_string()), ("NO_MATCH".to_string(), "no_match".to_string())],
            }).with_provider_name("OversizeHandling")
                    ],
                })
        .with_def("JsonMatchPattern", AttributeType::Struct {
                    name: "JsonMatchPattern".to_string(),
                    fields: vec![
                    StructField::new("all", AttributeType::map(AttributeType::String)).with_description("Inspect all parts of the web request's JSON body.").with_provider_name("All"),
                    StructField::new("included_paths", AttributeType::list(AttributeType::Custom {
                identity: None,
                pattern: Some("^[\\/]+([^~]*(~[01])*)*{1,512}$".to_string()),
                length: None,
                base: Box::new(AttributeType::String),
                validate: legacy_validator(validate_string_pattern_c77cf75cf1a75ade),
                to_dsl: None,
            })).with_provider_name("IncludedPaths")
                    ],
                })
        .with_def("LabelMatchStatement", AttributeType::Struct {
                    name: "LabelMatchStatement".to_string(),
                    fields: vec![
                    StructField::new("key", AttributeType::Custom {
                identity: None,
                pattern: Some("^[0-9A-Za-z_:-]{1,1024}$".to_string()),
                length: None,
                base: Box::new(AttributeType::String),
                validate: legacy_validator(validate_string_pattern_b3fc65b549fb77bd),
                to_dsl: None,
            }).required().with_provider_name("Key"),
                    StructField::new("scope", AttributeType::StringEnum {
                name: "LabelMatchScope".to_string(),
                values: vec!["LABEL".to_string(), "NAMESPACE".to_string()],
                identity: Some(carina_core::schema::string_enum_identity("LabelMatchScope", Some("awscc.wafv2.WebAcl"))),
                dsl_aliases: vec![("LABEL".to_string(), "label".to_string()), ("NAMESPACE".to_string(), "namespace".to_string())],
            }).required().with_provider_name("Scope")
                    ],
                })
        .with_def("ManagedRuleGroupStatement", AttributeType::Struct {
                    name: "ManagedRuleGroupStatement".to_string(),
                    fields: vec![
                    StructField::new("excluded_rules", AttributeType::list(AttributeType::Struct {
                    name: "ExcludedRule".to_string(),
                    fields: vec![
                    StructField::new("name", AttributeType::Custom {
                identity: None,
                pattern: Some("^[0-9A-Za-z_-]{1,128}$".to_string()),
                length: None,
                base: Box::new(AttributeType::String),
                validate: legacy_validator(validate_string_pattern_42f7eceb887966ad),
                to_dsl: None,
            }).required().with_provider_name("Name")
                    ],
                })).with_provider_name("ExcludedRules").with_block_name("excluded_rule"),
                    StructField::new("managed_rule_group_configs", AttributeType::list(AttributeType::Struct {
                    name: "ManagedRuleGroupConfig".to_string(),
                    fields: vec![
                    StructField::new("aws_managed_rules_acfp_rule_set", AttributeType::Struct {
                    name: "AWSManagedRulesACFPRuleSet".to_string(),
                    fields: vec![
                    StructField::new("creation_path", AttributeType::String).required().with_provider_name("CreationPath"),
                    StructField::new("enable_regex_in_path", AttributeType::Bool).with_provider_name("EnableRegexInPath"),
                    StructField::new("registration_page_path", AttributeType::String).required().with_provider_name("RegistrationPagePath"),
                    StructField::new("request_inspection", AttributeType::Struct {
                    name: "RequestInspectionACFP".to_string(),
                    fields: vec![
                    StructField::new("address_fields", AttributeType::list(AttributeType::String)).with_provider_name("AddressFields"),
                    StructField::new("email_field", AttributeType::Struct {
                    name: "FieldIdentifier".to_string(),
                    fields: vec![
                    StructField::new("identifier", AttributeType::Custom {
                identity: None,
                pattern: Some(".*\\S.*".to_string()),
                length: Some((Some(1), Some(512))),
                base: Box::new(AttributeType::String),
                validate: legacy_validator(validate_string_pattern_4e09c821aed9e752_len_1_512),
                to_dsl: None,
            }).required().with_provider_name("Identifier")
                    ],
                }).with_provider_name("EmailField"),
                    StructField::new("password_field", AttributeType::Ref("FieldIdentifier".to_string())).with_provider_name("PasswordField"),
                    StructField::new("payload_type", AttributeType::StringEnum {
                name: "PayloadType".to_string(),
                values: vec!["JSON".to_string(), "FORM_ENCODED".to_string()],
                identity: Some(carina_core::schema::string_enum_identity("PayloadType", Some("awscc.wafv2.WebAcl"))),
                dsl_aliases: vec![("JSON".to_string(), "json".to_string()), ("FORM_ENCODED".to_string(), "form_encoded".to_string())],
            }).required().with_provider_name("PayloadType"),
                    StructField::new("phone_number_fields", AttributeType::list(AttributeType::String)).with_provider_name("PhoneNumberFields"),
                    StructField::new("username_field", AttributeType::Ref("FieldIdentifier".to_string())).with_provider_name("UsernameField")
                    ],
                }).required().with_provider_name("RequestInspection"),
                    StructField::new("response_inspection", AttributeType::Struct {
                    name: "ResponseInspection".to_string(),
                    fields: vec![
                    StructField::new("body_contains", AttributeType::Struct {
                    name: "ResponseInspectionBodyContains".to_string(),
                    fields: vec![
                    StructField::new("failure_strings", AttributeType::Custom {
                identity: None,
                pattern: None,
                length: None,
                base: Box::new(AttributeType::list(AttributeType::Custom {
                identity: None,
                pattern: Some(".*\\S.*".to_string()),
                length: Some((Some(1), Some(100))),
                base: Box::new(AttributeType::String),
                validate: legacy_validator(validate_string_pattern_4e09c821aed9e752_len_1_100),
                to_dsl: None,
            })),
                validate: legacy_validator(validate_list_items_1_5),
                to_dsl: None,
            }).required().with_provider_name("FailureStrings"),
                    StructField::new("success_strings", AttributeType::Custom {
                identity: None,
                pattern: None,
                length: None,
                base: Box::new(AttributeType::list(AttributeType::Custom {
                identity: None,
                pattern: Some(".*\\S.*".to_string()),
                length: Some((Some(1), Some(100))),
                base: Box::new(AttributeType::String),
                validate: legacy_validator(validate_string_pattern_4e09c821aed9e752_len_1_100),
                to_dsl: None,
            })),
                validate: legacy_validator(validate_list_items_1_5),
                to_dsl: None,
            }).required().with_provider_name("SuccessStrings")
                    ],
                }).with_provider_name("BodyContains"),
                    StructField::new("header", AttributeType::Struct {
                    name: "ResponseInspectionHeader".to_string(),
                    fields: vec![
                    StructField::new("failure_values", AttributeType::Custom {
                identity: None,
                pattern: None,
                length: None,
                base: Box::new(AttributeType::list(AttributeType::Custom {
                identity: None,
                pattern: Some(".*\\S.*".to_string()),
                length: Some((Some(1), Some(100))),
                base: Box::new(AttributeType::String),
                validate: legacy_validator(validate_string_pattern_4e09c821aed9e752_len_1_100),
                to_dsl: None,
            })),
                validate: legacy_validator(validate_list_items_1_3),
                to_dsl: None,
            }).required().with_provider_name("FailureValues"),
                    StructField::new("name", AttributeType::Custom {
                identity: None,
                pattern: Some(".*\\S.*".to_string()),
                length: Some((Some(1), Some(200))),
                base: Box::new(AttributeType::String),
                validate: legacy_validator(validate_string_pattern_4e09c821aed9e752_len_1_200),
                to_dsl: None,
            }).required().with_provider_name("Name"),
                    StructField::new("success_values", AttributeType::Custom {
                identity: None,
                pattern: None,
                length: None,
                base: Box::new(AttributeType::list(AttributeType::Custom {
                identity: None,
                pattern: Some(".*\\S.*".to_string()),
                length: Some((Some(1), Some(100))),
                base: Box::new(AttributeType::String),
                validate: legacy_validator(validate_string_pattern_4e09c821aed9e752_len_1_100),
                to_dsl: None,
            })),
                validate: legacy_validator(validate_list_items_1_3),
                to_dsl: None,
            }).required().with_provider_name("SuccessValues")
                    ],
                }).with_provider_name("Header"),
                    StructField::new("json", AttributeType::Struct {
                    name: "ResponseInspectionJson".to_string(),
                    fields: vec![
                    StructField::new("failure_values", AttributeType::Custom {
                identity: None,
                pattern: None,
                length: None,
                base: Box::new(AttributeType::list(AttributeType::Custom {
                identity: None,
                pattern: Some(".*\\S.*".to_string()),
                length: Some((Some(1), Some(100))),
                base: Box::new(AttributeType::String),
                validate: legacy_validator(validate_string_pattern_4e09c821aed9e752_len_1_100),
                to_dsl: None,
            })),
                validate: legacy_validator(validate_list_items_1_5),
                to_dsl: None,
            }).required().with_provider_name("FailureValues"),
                    StructField::new("identifier", AttributeType::Custom {
                identity: None,
                pattern: Some(".*\\S.*".to_string()),
                length: Some((Some(1), Some(512))),
                base: Box::new(AttributeType::String),
                validate: legacy_validator(validate_string_pattern_4e09c821aed9e752_len_1_512),
                to_dsl: None,
            }).required().with_provider_name("Identifier"),
                    StructField::new("success_values", AttributeType::Custom {
                identity: None,
                pattern: None,
                length: None,
                base: Box::new(AttributeType::list(AttributeType::Custom {
                identity: None,
                pattern: Some(".*\\S.*".to_string()),
                length: Some((Some(1), Some(100))),
                base: Box::new(AttributeType::String),
                validate: legacy_validator(validate_string_pattern_4e09c821aed9e752_len_1_100),
                to_dsl: None,
            })),
                validate: legacy_validator(validate_list_items_1_5),
                to_dsl: None,
            }).required().with_provider_name("SuccessValues")
                    ],
                }).with_provider_name("Json"),
                    StructField::new("status_code", AttributeType::Struct {
                    name: "ResponseInspectionStatusCode".to_string(),
                    fields: vec![
                    StructField::new("failure_codes", AttributeType::Custom {
                identity: None,
                pattern: None,
                length: None,
                base: Box::new(AttributeType::list(AttributeType::Int)),
                validate: legacy_validator(validate_list_items_1_10),
                to_dsl: None,
            }).required().with_provider_name("FailureCodes"),
                    StructField::new("success_codes", AttributeType::Custom {
                identity: None,
                pattern: None,
                length: None,
                base: Box::new(AttributeType::list(AttributeType::Int)),
                validate: legacy_validator(validate_list_items_1_10),
                to_dsl: None,
            }).required().with_provider_name("SuccessCodes")
                    ],
                }).with_provider_name("StatusCode")
                    ],
                }).with_provider_name("ResponseInspection")
                    ],
                }).with_provider_name("AWSManagedRulesACFPRuleSet"),
                    StructField::new("aws_managed_rules_atp_rule_set", AttributeType::Struct {
                    name: "AWSManagedRulesATPRuleSet".to_string(),
                    fields: vec![
                    StructField::new("enable_regex_in_path", AttributeType::Bool).with_provider_name("EnableRegexInPath"),
                    StructField::new("login_path", AttributeType::String).required().with_provider_name("LoginPath"),
                    StructField::new("request_inspection", AttributeType::Struct {
                    name: "RequestInspection".to_string(),
                    fields: vec![
                    StructField::new("password_field", AttributeType::Ref("FieldIdentifier".to_string())).required().with_provider_name("PasswordField"),
                    StructField::new("payload_type", AttributeType::StringEnum {
                name: "PayloadType".to_string(),
                values: vec!["JSON".to_string(), "FORM_ENCODED".to_string()],
                identity: Some(carina_core::schema::string_enum_identity("PayloadType", Some("awscc.wafv2.WebAcl"))),
                dsl_aliases: vec![("JSON".to_string(), "json".to_string()), ("FORM_ENCODED".to_string(), "form_encoded".to_string())],
            }).required().with_provider_name("PayloadType"),
                    StructField::new("username_field", AttributeType::Ref("FieldIdentifier".to_string())).required().with_provider_name("UsernameField")
                    ],
                }).with_provider_name("RequestInspection"),
                    StructField::new("response_inspection", AttributeType::Ref("ResponseInspection".to_string())).with_provider_name("ResponseInspection")
                    ],
                }).with_provider_name("AWSManagedRulesATPRuleSet"),
                    StructField::new("aws_managed_rules_anti_d_do_s_rule_set", AttributeType::Struct {
                    name: "AWSManagedRulesAntiDDoSRuleSet".to_string(),
                    fields: vec![
                    StructField::new("client_side_action_config", AttributeType::Struct {
                    name: "ClientSideActionConfig".to_string(),
                    fields: vec![
                    StructField::new("challenge", AttributeType::Struct {
                    name: "ClientSideAction".to_string(),
                    fields: vec![
                    StructField::new("exempt_uri_regular_expressions", AttributeType::list(AttributeType::Struct {
                    name: "Regex".to_string(),
                    fields: vec![
                    StructField::new("regex_string", AttributeType::String).with_provider_name("RegexString")
                    ],
                })).with_provider_name("ExemptUriRegularExpressions").with_block_name("exempt_uri_regular_expression"),
                    StructField::new("sensitivity", AttributeType::StringEnum {
                name: "SensitivityToAct".to_string(),
                values: vec!["LOW".to_string(), "MEDIUM".to_string(), "HIGH".to_string()],
                identity: Some(carina_core::schema::string_enum_identity("SensitivityToAct", Some("awscc.wafv2.WebAcl"))),
                dsl_aliases: vec![("LOW".to_string(), "low".to_string()), ("MEDIUM".to_string(), "medium".to_string()), ("HIGH".to_string(), "high".to_string())],
            }).with_provider_name("Sensitivity"),
                    StructField::new("usage_of_action", AttributeType::StringEnum {
                name: "UsageOfAction".to_string(),
                values: vec!["ENABLED".to_string(), "DISABLED".to_string()],
                identity: Some(carina_core::schema::string_enum_identity("UsageOfAction", Some("awscc.wafv2.WebAcl"))),
                dsl_aliases: vec![("ENABLED".to_string(), "enabled".to_string()), ("DISABLED".to_string(), "disabled".to_string())],
            }).required().with_provider_name("UsageOfAction")
                    ],
                }).required().with_provider_name("Challenge").with_block_name("challenge")
                    ],
                }).required().with_provider_name("ClientSideActionConfig").with_block_name("client_side_action_config"),
                    StructField::new("sensitivity_to_block", AttributeType::StringEnum {
                name: "SensitivityToAct".to_string(),
                values: vec!["LOW".to_string(), "MEDIUM".to_string(), "HIGH".to_string()],
                identity: Some(carina_core::schema::string_enum_identity("SensitivityToAct", Some("awscc.wafv2.WebAcl"))),
                dsl_aliases: vec![("LOW".to_string(), "low".to_string()), ("MEDIUM".to_string(), "medium".to_string()), ("HIGH".to_string(), "high".to_string())],
            }).with_provider_name("SensitivityToBlock")
                    ],
                }).with_provider_name("AWSManagedRulesAntiDDoSRuleSet").with_block_name("aws_managed_rules_anti_d_do_s_rule_set"),
                    StructField::new("aws_managed_rules_bot_control_rule_set", AttributeType::Struct {
                    name: "AWSManagedRulesBotControlRuleSet".to_string(),
                    fields: vec![
                    StructField::new("enable_machine_learning", AttributeType::Bool).with_provider_name("EnableMachineLearning"),
                    StructField::new("inspection_level", AttributeType::StringEnum {
                name: "InspectionLevel".to_string(),
                values: vec!["COMMON".to_string(), "TARGETED".to_string()],
                identity: Some(carina_core::schema::string_enum_identity("InspectionLevel", Some("awscc.wafv2.WebAcl"))),
                dsl_aliases: vec![("COMMON".to_string(), "common".to_string()), ("TARGETED".to_string(), "targeted".to_string())],
            }).required().with_provider_name("InspectionLevel")
                    ],
                }).with_provider_name("AWSManagedRulesBotControlRuleSet"),
                    StructField::new("login_path", AttributeType::Custom {
                identity: None,
                pattern: Some(".*\\S.*".to_string()),
                length: Some((Some(1), Some(256))),
                base: Box::new(AttributeType::String),
                validate: legacy_validator(validate_string_pattern_4e09c821aed9e752_len_1_256),
                to_dsl: None,
            }).with_provider_name("LoginPath"),
                    StructField::new("password_field", AttributeType::Ref("FieldIdentifier".to_string())).with_provider_name("PasswordField"),
                    StructField::new("payload_type", AttributeType::StringEnum {
                name: "PayloadType".to_string(),
                values: vec!["JSON".to_string(), "FORM_ENCODED".to_string()],
                identity: Some(carina_core::schema::string_enum_identity("PayloadType", Some("awscc.wafv2.WebAcl"))),
                dsl_aliases: vec![("JSON".to_string(), "json".to_string()), ("FORM_ENCODED".to_string(), "form_encoded".to_string())],
            }).with_provider_name("PayloadType"),
                    StructField::new("username_field", AttributeType::Ref("FieldIdentifier".to_string())).with_provider_name("UsernameField")
                    ],
                })).with_description("Collection of ManagedRuleGroupConfig.").with_provider_name("ManagedRuleGroupConfigs").with_block_name("managed_rule_group_config"),
                    StructField::new("name", AttributeType::Custom {
                identity: None,
                pattern: Some("^[0-9A-Za-z_-]{1,128}$".to_string()),
                length: None,
                base: Box::new(AttributeType::String),
                validate: legacy_validator(validate_string_pattern_42f7eceb887966ad),
                to_dsl: None,
            }).required().with_provider_name("Name"),
                    StructField::new("rule_action_overrides", AttributeType::Custom {
                identity: None,
                pattern: None,
                length: None,
                base: Box::new(AttributeType::list(AttributeType::Struct {
                    name: "RuleActionOverride".to_string(),
                    fields: vec![
                    StructField::new("action_to_use", AttributeType::Ref("RuleAction".to_string())).required().with_provider_name("ActionToUse"),
                    StructField::new("name", AttributeType::Custom {
                identity: None,
                pattern: Some("^[0-9A-Za-z_-]{1,128}$".to_string()),
                length: None,
                base: Box::new(AttributeType::String),
                validate: legacy_validator(validate_string_pattern_42f7eceb887966ad),
                to_dsl: None,
            }).required().with_provider_name("Name")
                    ],
                })),
                validate: legacy_validator(validate_list_items_max_100),
                to_dsl: None,
            }).with_description("Action overrides for rules in the rule group.").with_provider_name("RuleActionOverrides").with_block_name("rule_action_override"),
                    StructField::new("scope_down_statement", AttributeType::Ref("Statement".to_string())).with_provider_name("ScopeDownStatement"),
                    StructField::new("vendor_name", AttributeType::String).required().with_provider_name("VendorName"),
                    StructField::new("version", AttributeType::Custom {
                identity: None,
                pattern: Some("^[\\w#:\\.\\-/]+$".to_string()),
                length: Some((Some(1), Some(64))),
                base: Box::new(AttributeType::String),
                validate: legacy_validator(validate_string_pattern_82a037d29be8d222_len_1_64),
                to_dsl: None,
            }).with_provider_name("Version")
                    ],
                })
        .with_def("NotStatement", AttributeType::Struct {
                    name: "NotStatement".to_string(),
                    fields: vec![
                    StructField::new("statement", AttributeType::Ref("Statement".to_string())).required().with_provider_name("Statement")
                    ],
                })
        .with_def("OnSourceDDoSProtectionConfig", AttributeType::Struct {
                    name: "OnSourceDDoSProtectionConfig".to_string(),
                    fields: vec![
                    StructField::new("alb_low_reputation_mode", AttributeType::StringEnum {
                name: "AlbLowReputationMode".to_string(),
                values: vec!["ACTIVE_UNDER_DDOS".to_string(), "ALWAYS_ON".to_string()],
                identity: Some(carina_core::schema::string_enum_identity("AlbLowReputationMode", Some("awscc.wafv2.WebAcl"))),
                dsl_aliases: vec![("ACTIVE_UNDER_DDOS".to_string(), "active_under_ddos".to_string()), ("ALWAYS_ON".to_string(), "always_on".to_string())],
            }).required().with_provider_name("ALBLowReputationMode")
                    ],
                })
        .with_def("OrStatement", AttributeType::Struct {
                    name: "OrStatement".to_string(),
                    fields: vec![
                    StructField::new("statements", AttributeType::list(AttributeType::Struct {
                    name: "Statement".to_string(),
                    fields: vec![
                    StructField::new("and_statement", AttributeType::Ref("AndStatement".to_string())).with_provider_name("AndStatement"),
                    StructField::new("asn_match_statement", AttributeType::Ref("AsnMatchStatement".to_string())).with_provider_name("AsnMatchStatement"),
                    StructField::new("byte_match_statement", AttributeType::Ref("ByteMatchStatement".to_string())).with_provider_name("ByteMatchStatement"),
                    StructField::new("geo_match_statement", AttributeType::Ref("GeoMatchStatement".to_string())).with_provider_name("GeoMatchStatement"),
                    StructField::new("ip_set_reference_statement", AttributeType::Ref("IPSetReferenceStatement".to_string())).with_provider_name("IPSetReferenceStatement"),
                    StructField::new("label_match_statement", AttributeType::Ref("LabelMatchStatement".to_string())).with_provider_name("LabelMatchStatement"),
                    StructField::new("managed_rule_group_statement", AttributeType::Ref("ManagedRuleGroupStatement".to_string())).with_provider_name("ManagedRuleGroupStatement"),
                    StructField::new("not_statement", AttributeType::Ref("NotStatement".to_string())).with_provider_name("NotStatement"),
                    StructField::new("or_statement", AttributeType::Ref("OrStatement".to_string())).with_provider_name("OrStatement"),
                    StructField::new("rate_based_statement", AttributeType::Struct {
                    name: "RateBasedStatement".to_string(),
                    fields: vec![
                    StructField::new("aggregate_key_type", AttributeType::StringEnum {
                name: "AggregateKeyType".to_string(),
                values: vec!["CONSTANT".to_string(), "IP".to_string(), "FORWARDED_IP".to_string(), "CUSTOM_KEYS".to_string()],
                identity: Some(carina_core::schema::string_enum_identity("AggregateKeyType", Some("awscc.wafv2.WebAcl"))),
                dsl_aliases: vec![("CONSTANT".to_string(), "constant".to_string()), ("IP".to_string(), "ip".to_string()), ("FORWARDED_IP".to_string(), "forwarded_ip".to_string()), ("CUSTOM_KEYS".to_string(), "custom_keys".to_string())],
            }).required().with_provider_name("AggregateKeyType"),
                    StructField::new("custom_keys", AttributeType::Custom {
                identity: None,
                pattern: None,
                length: None,
                base: Box::new(AttributeType::list(AttributeType::Struct {
                    name: "RateBasedStatementCustomKey".to_string(),
                    fields: vec![
                    StructField::new("asn", AttributeType::String).with_provider_name("ASN"),
                    StructField::new("cookie", AttributeType::Struct {
                    name: "RateLimitCookie".to_string(),
                    fields: vec![
                    StructField::new("name", AttributeType::Custom {
                identity: None,
                pattern: Some(".*\\S.*".to_string()),
                length: Some((Some(1), Some(64))),
                base: Box::new(AttributeType::String),
                validate: legacy_validator(validate_string_pattern_4e09c821aed9e752_len_1_64),
                to_dsl: None,
            }).required().with_description("The name of the cookie to use.").with_provider_name("Name"),
                    StructField::new("text_transformations", AttributeType::list(AttributeType::Struct {
                    name: "TextTransformation".to_string(),
                    fields: vec![
                    StructField::new("priority", AttributeType::String).required().with_provider_name("Priority"),
                    StructField::new("type", AttributeType::StringEnum {
                name: "TextTransformationType".to_string(),
                values: vec!["NONE".to_string(), "COMPRESS_WHITE_SPACE".to_string(), "HTML_ENTITY_DECODE".to_string(), "LOWERCASE".to_string(), "CMD_LINE".to_string(), "URL_DECODE".to_string(), "BASE64_DECODE".to_string(), "HEX_DECODE".to_string(), "MD5".to_string(), "REPLACE_COMMENTS".to_string(), "ESCAPE_SEQ_DECODE".to_string(), "SQL_HEX_DECODE".to_string(), "CSS_DECODE".to_string(), "JS_DECODE".to_string(), "NORMALIZE_PATH".to_string(), "NORMALIZE_PATH_WIN".to_string(), "REMOVE_NULLS".to_string(), "REPLACE_NULLS".to_string(), "BASE64_DECODE_EXT".to_string(), "URL_DECODE_UNI".to_string(), "UTF8_TO_UNICODE".to_string()],
                identity: Some(carina_core::schema::string_enum_identity("TextTransformationType", Some("awscc.wafv2.WebAcl"))),
                dsl_aliases: vec![("NONE".to_string(), "none".to_string()), ("COMPRESS_WHITE_SPACE".to_string(), "compress_white_space".to_string()), ("HTML_ENTITY_DECODE".to_string(), "html_entity_decode".to_string()), ("LOWERCASE".to_string(), "lowercase".to_string()), ("CMD_LINE".to_string(), "cmd_line".to_string()), ("URL_DECODE".to_string(), "url_decode".to_string()), ("BASE64_DECODE".to_string(), "base64_decode".to_string()), ("HEX_DECODE".to_string(), "hex_decode".to_string()), ("MD5".to_string(), "md5".to_string()), ("REPLACE_COMMENTS".to_string(), "replace_comments".to_string()), ("ESCAPE_SEQ_DECODE".to_string(), "escape_seq_decode".to_string()), ("SQL_HEX_DECODE".to_string(), "sql_hex_decode".to_string()), ("CSS_DECODE".to_string(), "css_decode".to_string()), ("JS_DECODE".to_string(), "js_decode".to_string()), ("NORMALIZE_PATH".to_string(), "normalize_path".to_string()), ("NORMALIZE_PATH_WIN".to_string(), "normalize_path_win".to_string()), ("REMOVE_NULLS".to_string(), "remove_nulls".to_string()), ("REPLACE_NULLS".to_string(), "replace_nulls".to_string()), ("BASE64_DECODE_EXT".to_string(), "base64_decode_ext".to_string()), ("URL_DECODE_UNI".to_string(), "url_decode_uni".to_string()), ("UTF8_TO_UNICODE".to_string(), "utf8_to_unicode".to_string())],
            }).required().with_provider_name("Type")
                    ],
                })).required().with_provider_name("TextTransformations").with_block_name("text_transformation")
                    ],
                }).with_provider_name("Cookie").with_block_name("cookie"),
                    StructField::new("forwarded_ip", types::ipv4_address()).with_provider_name("ForwardedIP"),
                    StructField::new("http_method", AttributeType::String).with_provider_name("HTTPMethod"),
                    StructField::new("header", AttributeType::Struct {
                    name: "RateLimitHeader".to_string(),
                    fields: vec![
                    StructField::new("name", AttributeType::Custom {
                identity: None,
                pattern: Some(".*\\S.*".to_string()),
                length: Some((Some(1), Some(64))),
                base: Box::new(AttributeType::String),
                validate: legacy_validator(validate_string_pattern_4e09c821aed9e752_len_1_64),
                to_dsl: None,
            }).required().with_description("The name of the header to use.").with_provider_name("Name"),
                    StructField::new("text_transformations", AttributeType::list(AttributeType::Struct {
                    name: "TextTransformation".to_string(),
                    fields: vec![
                    StructField::new("priority", AttributeType::String).required().with_provider_name("Priority"),
                    StructField::new("type", AttributeType::StringEnum {
                name: "TextTransformationType".to_string(),
                values: vec!["NONE".to_string(), "COMPRESS_WHITE_SPACE".to_string(), "HTML_ENTITY_DECODE".to_string(), "LOWERCASE".to_string(), "CMD_LINE".to_string(), "URL_DECODE".to_string(), "BASE64_DECODE".to_string(), "HEX_DECODE".to_string(), "MD5".to_string(), "REPLACE_COMMENTS".to_string(), "ESCAPE_SEQ_DECODE".to_string(), "SQL_HEX_DECODE".to_string(), "CSS_DECODE".to_string(), "JS_DECODE".to_string(), "NORMALIZE_PATH".to_string(), "NORMALIZE_PATH_WIN".to_string(), "REMOVE_NULLS".to_string(), "REPLACE_NULLS".to_string(), "BASE64_DECODE_EXT".to_string(), "URL_DECODE_UNI".to_string(), "UTF8_TO_UNICODE".to_string()],
                identity: Some(carina_core::schema::string_enum_identity("TextTransformationType", Some("awscc.wafv2.WebAcl"))),
                dsl_aliases: vec![("NONE".to_string(), "none".to_string()), ("COMPRESS_WHITE_SPACE".to_string(), "compress_white_space".to_string()), ("HTML_ENTITY_DECODE".to_string(), "html_entity_decode".to_string()), ("LOWERCASE".to_string(), "lowercase".to_string()), ("CMD_LINE".to_string(), "cmd_line".to_string()), ("URL_DECODE".to_string(), "url_decode".to_string()), ("BASE64_DECODE".to_string(), "base64_decode".to_string()), ("HEX_DECODE".to_string(), "hex_decode".to_string()), ("MD5".to_string(), "md5".to_string()), ("REPLACE_COMMENTS".to_string(), "replace_comments".to_string()), ("ESCAPE_SEQ_DECODE".to_string(), "escape_seq_decode".to_string()), ("SQL_HEX_DECODE".to_string(), "sql_hex_decode".to_string()), ("CSS_DECODE".to_string(), "css_decode".to_string()), ("JS_DECODE".to_string(), "js_decode".to_string()), ("NORMALIZE_PATH".to_string(), "normalize_path".to_string()), ("NORMALIZE_PATH_WIN".to_string(), "normalize_path_win".to_string()), ("REMOVE_NULLS".to_string(), "remove_nulls".to_string()), ("REPLACE_NULLS".to_string(), "replace_nulls".to_string()), ("BASE64_DECODE_EXT".to_string(), "base64_decode_ext".to_string()), ("URL_DECODE_UNI".to_string(), "url_decode_uni".to_string()), ("UTF8_TO_UNICODE".to_string(), "utf8_to_unicode".to_string())],
            }).required().with_provider_name("Type")
                    ],
                })).required().with_provider_name("TextTransformations").with_block_name("text_transformation")
                    ],
                }).with_provider_name("Header").with_block_name("header"),
                    StructField::new("ip", types::ipv4_address()).with_provider_name("IP"),
                    StructField::new("ja3_fingerprint", AttributeType::Struct {
                    name: "RateLimitJA3Fingerprint".to_string(),
                    fields: vec![
                    StructField::new("fallback_behavior", AttributeType::StringEnum {
                name: "FallbackBehavior".to_string(),
                values: vec!["MATCH".to_string(), "NO_MATCH".to_string()],
                identity: Some(carina_core::schema::string_enum_identity("FallbackBehavior", Some("awscc.wafv2.WebAcl"))),
                dsl_aliases: vec![("MATCH".to_string(), "match".to_string()), ("NO_MATCH".to_string(), "no_match".to_string())],
            }).required().with_provider_name("FallbackBehavior")
                    ],
                }).with_provider_name("JA3Fingerprint"),
                    StructField::new("ja4_fingerprint", AttributeType::Struct {
                    name: "RateLimitJA4Fingerprint".to_string(),
                    fields: vec![
                    StructField::new("fallback_behavior", AttributeType::StringEnum {
                name: "FallbackBehavior".to_string(),
                values: vec!["MATCH".to_string(), "NO_MATCH".to_string()],
                identity: Some(carina_core::schema::string_enum_identity("FallbackBehavior", Some("awscc.wafv2.WebAcl"))),
                dsl_aliases: vec![("MATCH".to_string(), "match".to_string()), ("NO_MATCH".to_string(), "no_match".to_string())],
            }).required().with_provider_name("FallbackBehavior")
                    ],
                }).with_provider_name("JA4Fingerprint"),
                    StructField::new("label_namespace", AttributeType::Struct {
                    name: "RateLimitLabelNamespace".to_string(),
                    fields: vec![
                    StructField::new("namespace", AttributeType::Custom {
                identity: None,
                pattern: Some("^[0-9A-Za-z_:-]{1,1024}$".to_string()),
                length: None,
                base: Box::new(AttributeType::String),
                validate: legacy_validator(validate_string_pattern_b3fc65b549fb77bd),
                to_dsl: None,
            }).required().with_description("The namespace to use for aggregation.").with_provider_name("Namespace")
                    ],
                }).with_provider_name("LabelNamespace"),
                    StructField::new("query_argument", AttributeType::Struct {
                    name: "RateLimitQueryArgument".to_string(),
                    fields: vec![
                    StructField::new("name", AttributeType::Custom {
                identity: None,
                pattern: Some(".*\\S.*".to_string()),
                length: Some((Some(1), Some(64))),
                base: Box::new(AttributeType::String),
                validate: legacy_validator(validate_string_pattern_4e09c821aed9e752_len_1_64),
                to_dsl: None,
            }).required().with_description("The name of the query argument to use.").with_provider_name("Name"),
                    StructField::new("text_transformations", AttributeType::list(AttributeType::Struct {
                    name: "TextTransformation".to_string(),
                    fields: vec![
                    StructField::new("priority", AttributeType::String).required().with_provider_name("Priority"),
                    StructField::new("type", AttributeType::StringEnum {
                name: "TextTransformationType".to_string(),
                values: vec!["NONE".to_string(), "COMPRESS_WHITE_SPACE".to_string(), "HTML_ENTITY_DECODE".to_string(), "LOWERCASE".to_string(), "CMD_LINE".to_string(), "URL_DECODE".to_string(), "BASE64_DECODE".to_string(), "HEX_DECODE".to_string(), "MD5".to_string(), "REPLACE_COMMENTS".to_string(), "ESCAPE_SEQ_DECODE".to_string(), "SQL_HEX_DECODE".to_string(), "CSS_DECODE".to_string(), "JS_DECODE".to_string(), "NORMALIZE_PATH".to_string(), "NORMALIZE_PATH_WIN".to_string(), "REMOVE_NULLS".to_string(), "REPLACE_NULLS".to_string(), "BASE64_DECODE_EXT".to_string(), "URL_DECODE_UNI".to_string(), "UTF8_TO_UNICODE".to_string()],
                identity: Some(carina_core::schema::string_enum_identity("TextTransformationType", Some("awscc.wafv2.WebAcl"))),
                dsl_aliases: vec![("NONE".to_string(), "none".to_string()), ("COMPRESS_WHITE_SPACE".to_string(), "compress_white_space".to_string()), ("HTML_ENTITY_DECODE".to_string(), "html_entity_decode".to_string()), ("LOWERCASE".to_string(), "lowercase".to_string()), ("CMD_LINE".to_string(), "cmd_line".to_string()), ("URL_DECODE".to_string(), "url_decode".to_string()), ("BASE64_DECODE".to_string(), "base64_decode".to_string()), ("HEX_DECODE".to_string(), "hex_decode".to_string()), ("MD5".to_string(), "md5".to_string()), ("REPLACE_COMMENTS".to_string(), "replace_comments".to_string()), ("ESCAPE_SEQ_DECODE".to_string(), "escape_seq_decode".to_string()), ("SQL_HEX_DECODE".to_string(), "sql_hex_decode".to_string()), ("CSS_DECODE".to_string(), "css_decode".to_string()), ("JS_DECODE".to_string(), "js_decode".to_string()), ("NORMALIZE_PATH".to_string(), "normalize_path".to_string()), ("NORMALIZE_PATH_WIN".to_string(), "normalize_path_win".to_string()), ("REMOVE_NULLS".to_string(), "remove_nulls".to_string()), ("REPLACE_NULLS".to_string(), "replace_nulls".to_string()), ("BASE64_DECODE_EXT".to_string(), "base64_decode_ext".to_string()), ("URL_DECODE_UNI".to_string(), "url_decode_uni".to_string()), ("UTF8_TO_UNICODE".to_string(), "utf8_to_unicode".to_string())],
            }).required().with_provider_name("Type")
                    ],
                })).required().with_provider_name("TextTransformations").with_block_name("text_transformation")
                    ],
                }).with_provider_name("QueryArgument").with_block_name("query_argument"),
                    StructField::new("query_string", AttributeType::Struct {
                    name: "RateLimitQueryString".to_string(),
                    fields: vec![
                    StructField::new("text_transformations", AttributeType::list(AttributeType::Struct {
                    name: "TextTransformation".to_string(),
                    fields: vec![
                    StructField::new("priority", AttributeType::String).required().with_provider_name("Priority"),
                    StructField::new("type", AttributeType::StringEnum {
                name: "TextTransformationType".to_string(),
                values: vec!["NONE".to_string(), "COMPRESS_WHITE_SPACE".to_string(), "HTML_ENTITY_DECODE".to_string(), "LOWERCASE".to_string(), "CMD_LINE".to_string(), "URL_DECODE".to_string(), "BASE64_DECODE".to_string(), "HEX_DECODE".to_string(), "MD5".to_string(), "REPLACE_COMMENTS".to_string(), "ESCAPE_SEQ_DECODE".to_string(), "SQL_HEX_DECODE".to_string(), "CSS_DECODE".to_string(), "JS_DECODE".to_string(), "NORMALIZE_PATH".to_string(), "NORMALIZE_PATH_WIN".to_string(), "REMOVE_NULLS".to_string(), "REPLACE_NULLS".to_string(), "BASE64_DECODE_EXT".to_string(), "URL_DECODE_UNI".to_string(), "UTF8_TO_UNICODE".to_string()],
                identity: Some(carina_core::schema::string_enum_identity("TextTransformationType", Some("awscc.wafv2.WebAcl"))),
                dsl_aliases: vec![("NONE".to_string(), "none".to_string()), ("COMPRESS_WHITE_SPACE".to_string(), "compress_white_space".to_string()), ("HTML_ENTITY_DECODE".to_string(), "html_entity_decode".to_string()), ("LOWERCASE".to_string(), "lowercase".to_string()), ("CMD_LINE".to_string(), "cmd_line".to_string()), ("URL_DECODE".to_string(), "url_decode".to_string()), ("BASE64_DECODE".to_string(), "base64_decode".to_string()), ("HEX_DECODE".to_string(), "hex_decode".to_string()), ("MD5".to_string(), "md5".to_string()), ("REPLACE_COMMENTS".to_string(), "replace_comments".to_string()), ("ESCAPE_SEQ_DECODE".to_string(), "escape_seq_decode".to_string()), ("SQL_HEX_DECODE".to_string(), "sql_hex_decode".to_string()), ("CSS_DECODE".to_string(), "css_decode".to_string()), ("JS_DECODE".to_string(), "js_decode".to_string()), ("NORMALIZE_PATH".to_string(), "normalize_path".to_string()), ("NORMALIZE_PATH_WIN".to_string(), "normalize_path_win".to_string()), ("REMOVE_NULLS".to_string(), "remove_nulls".to_string()), ("REPLACE_NULLS".to_string(), "replace_nulls".to_string()), ("BASE64_DECODE_EXT".to_string(), "base64_decode_ext".to_string()), ("URL_DECODE_UNI".to_string(), "url_decode_uni".to_string()), ("UTF8_TO_UNICODE".to_string(), "utf8_to_unicode".to_string())],
            }).required().with_provider_name("Type")
                    ],
                })).required().with_provider_name("TextTransformations").with_block_name("text_transformation")
                    ],
                }).with_provider_name("QueryString").with_block_name("query_string"),
                    StructField::new("uri_path", AttributeType::Struct {
                    name: "RateLimitUriPath".to_string(),
                    fields: vec![
                    StructField::new("text_transformations", AttributeType::list(AttributeType::Struct {
                    name: "TextTransformation".to_string(),
                    fields: vec![
                    StructField::new("priority", AttributeType::String).required().with_provider_name("Priority"),
                    StructField::new("type", AttributeType::StringEnum {
                name: "TextTransformationType".to_string(),
                values: vec!["NONE".to_string(), "COMPRESS_WHITE_SPACE".to_string(), "HTML_ENTITY_DECODE".to_string(), "LOWERCASE".to_string(), "CMD_LINE".to_string(), "URL_DECODE".to_string(), "BASE64_DECODE".to_string(), "HEX_DECODE".to_string(), "MD5".to_string(), "REPLACE_COMMENTS".to_string(), "ESCAPE_SEQ_DECODE".to_string(), "SQL_HEX_DECODE".to_string(), "CSS_DECODE".to_string(), "JS_DECODE".to_string(), "NORMALIZE_PATH".to_string(), "NORMALIZE_PATH_WIN".to_string(), "REMOVE_NULLS".to_string(), "REPLACE_NULLS".to_string(), "BASE64_DECODE_EXT".to_string(), "URL_DECODE_UNI".to_string(), "UTF8_TO_UNICODE".to_string()],
                identity: Some(carina_core::schema::string_enum_identity("TextTransformationType", Some("awscc.wafv2.WebAcl"))),
                dsl_aliases: vec![("NONE".to_string(), "none".to_string()), ("COMPRESS_WHITE_SPACE".to_string(), "compress_white_space".to_string()), ("HTML_ENTITY_DECODE".to_string(), "html_entity_decode".to_string()), ("LOWERCASE".to_string(), "lowercase".to_string()), ("CMD_LINE".to_string(), "cmd_line".to_string()), ("URL_DECODE".to_string(), "url_decode".to_string()), ("BASE64_DECODE".to_string(), "base64_decode".to_string()), ("HEX_DECODE".to_string(), "hex_decode".to_string()), ("MD5".to_string(), "md5".to_string()), ("REPLACE_COMMENTS".to_string(), "replace_comments".to_string()), ("ESCAPE_SEQ_DECODE".to_string(), "escape_seq_decode".to_string()), ("SQL_HEX_DECODE".to_string(), "sql_hex_decode".to_string()), ("CSS_DECODE".to_string(), "css_decode".to_string()), ("JS_DECODE".to_string(), "js_decode".to_string()), ("NORMALIZE_PATH".to_string(), "normalize_path".to_string()), ("NORMALIZE_PATH_WIN".to_string(), "normalize_path_win".to_string()), ("REMOVE_NULLS".to_string(), "remove_nulls".to_string()), ("REPLACE_NULLS".to_string(), "replace_nulls".to_string()), ("BASE64_DECODE_EXT".to_string(), "base64_decode_ext".to_string()), ("URL_DECODE_UNI".to_string(), "url_decode_uni".to_string()), ("UTF8_TO_UNICODE".to_string(), "utf8_to_unicode".to_string())],
            }).required().with_provider_name("Type")
                    ],
                })).required().with_provider_name("TextTransformations").with_block_name("text_transformation")
                    ],
                }).with_provider_name("UriPath").with_block_name("uri_path")
                    ],
                })),
                validate: legacy_validator(validate_list_items_max_5),
                to_dsl: None,
            }).with_description("Specifies the aggregate keys to use in a rate-base rule.").with_provider_name("CustomKeys").with_block_name("custom_key"),
                    StructField::new("evaluation_window_sec", AttributeType::StringEnum {
                name: "EvaluationWindowSec".to_string(),
                values: vec!["60".to_string(), "120".to_string(), "300".to_string(), "600".to_string()],
                identity: Some(carina_core::schema::string_enum_identity("EvaluationWindowSec", Some("awscc.wafv2.WebAcl"))),
                dsl_aliases: vec![("60".to_string(), "60".to_string()), ("120".to_string(), "120".to_string()), ("300".to_string(), "300".to_string()), ("600".to_string(), "600".to_string())],
            }).with_provider_name("EvaluationWindowSec"),
                    StructField::new("forwarded_ip_config", AttributeType::Ref("ForwardedIPConfiguration".to_string())).with_provider_name("ForwardedIPConfig"),
                    StructField::new("limit", AttributeType::String).required().with_provider_name("Limit"),
                    StructField::new("scope_down_statement", AttributeType::Ref("Statement".to_string())).with_provider_name("ScopeDownStatement")
                    ],
                }).with_provider_name("RateBasedStatement").with_block_name("rate_based_statement"),
                    StructField::new("regex_match_statement", AttributeType::Struct {
                    name: "RegexMatchStatement".to_string(),
                    fields: vec![
                    StructField::new("field_to_match", AttributeType::Ref("FieldToMatch".to_string())).required().with_provider_name("FieldToMatch"),
                    StructField::new("regex_string", AttributeType::Custom {
                identity: None,
                pattern: None,
                length: Some((Some(1), Some(512))),
                base: Box::new(AttributeType::String),
                validate: legacy_validator(validate_string_length_1_512),
                to_dsl: None,
            }).required().with_provider_name("RegexString"),
                    StructField::new("text_transformations", AttributeType::list(AttributeType::Struct {
                    name: "TextTransformation".to_string(),
                    fields: vec![
                    StructField::new("priority", AttributeType::String).required().with_provider_name("Priority"),
                    StructField::new("type", AttributeType::StringEnum {
                name: "TextTransformationType".to_string(),
                values: vec!["NONE".to_string(), "COMPRESS_WHITE_SPACE".to_string(), "HTML_ENTITY_DECODE".to_string(), "LOWERCASE".to_string(), "CMD_LINE".to_string(), "URL_DECODE".to_string(), "BASE64_DECODE".to_string(), "HEX_DECODE".to_string(), "MD5".to_string(), "REPLACE_COMMENTS".to_string(), "ESCAPE_SEQ_DECODE".to_string(), "SQL_HEX_DECODE".to_string(), "CSS_DECODE".to_string(), "JS_DECODE".to_string(), "NORMALIZE_PATH".to_string(), "NORMALIZE_PATH_WIN".to_string(), "REMOVE_NULLS".to_string(), "REPLACE_NULLS".to_string(), "BASE64_DECODE_EXT".to_string(), "URL_DECODE_UNI".to_string(), "UTF8_TO_UNICODE".to_string()],
                identity: Some(carina_core::schema::string_enum_identity("TextTransformationType", Some("awscc.wafv2.WebAcl"))),
                dsl_aliases: vec![("NONE".to_string(), "none".to_string()), ("COMPRESS_WHITE_SPACE".to_string(), "compress_white_space".to_string()), ("HTML_ENTITY_DECODE".to_string(), "html_entity_decode".to_string()), ("LOWERCASE".to_string(), "lowercase".to_string()), ("CMD_LINE".to_string(), "cmd_line".to_string()), ("URL_DECODE".to_string(), "url_decode".to_string()), ("BASE64_DECODE".to_string(), "base64_decode".to_string()), ("HEX_DECODE".to_string(), "hex_decode".to_string()), ("MD5".to_string(), "md5".to_string()), ("REPLACE_COMMENTS".to_string(), "replace_comments".to_string()), ("ESCAPE_SEQ_DECODE".to_string(), "escape_seq_decode".to_string()), ("SQL_HEX_DECODE".to_string(), "sql_hex_decode".to_string()), ("CSS_DECODE".to_string(), "css_decode".to_string()), ("JS_DECODE".to_string(), "js_decode".to_string()), ("NORMALIZE_PATH".to_string(), "normalize_path".to_string()), ("NORMALIZE_PATH_WIN".to_string(), "normalize_path_win".to_string()), ("REMOVE_NULLS".to_string(), "remove_nulls".to_string()), ("REPLACE_NULLS".to_string(), "replace_nulls".to_string()), ("BASE64_DECODE_EXT".to_string(), "base64_decode_ext".to_string()), ("URL_DECODE_UNI".to_string(), "url_decode_uni".to_string()), ("UTF8_TO_UNICODE".to_string(), "utf8_to_unicode".to_string())],
            }).required().with_provider_name("Type")
                    ],
                })).required().with_provider_name("TextTransformations").with_block_name("text_transformation")
                    ],
                }).with_provider_name("RegexMatchStatement").with_block_name("regex_match_statement"),
                    StructField::new("regex_pattern_set_reference_statement", AttributeType::Struct {
                    name: "RegexPatternSetReferenceStatement".to_string(),
                    fields: vec![
                    StructField::new("arn", super::arn()).required().with_provider_name("Arn"),
                    StructField::new("field_to_match", AttributeType::Ref("FieldToMatch".to_string())).required().with_provider_name("FieldToMatch"),
                    StructField::new("text_transformations", AttributeType::list(AttributeType::Struct {
                    name: "TextTransformation".to_string(),
                    fields: vec![
                    StructField::new("priority", AttributeType::String).required().with_provider_name("Priority"),
                    StructField::new("type", AttributeType::StringEnum {
                name: "TextTransformationType".to_string(),
                values: vec!["NONE".to_string(), "COMPRESS_WHITE_SPACE".to_string(), "HTML_ENTITY_DECODE".to_string(), "LOWERCASE".to_string(), "CMD_LINE".to_string(), "URL_DECODE".to_string(), "BASE64_DECODE".to_string(), "HEX_DECODE".to_string(), "MD5".to_string(), "REPLACE_COMMENTS".to_string(), "ESCAPE_SEQ_DECODE".to_string(), "SQL_HEX_DECODE".to_string(), "CSS_DECODE".to_string(), "JS_DECODE".to_string(), "NORMALIZE_PATH".to_string(), "NORMALIZE_PATH_WIN".to_string(), "REMOVE_NULLS".to_string(), "REPLACE_NULLS".to_string(), "BASE64_DECODE_EXT".to_string(), "URL_DECODE_UNI".to_string(), "UTF8_TO_UNICODE".to_string()],
                identity: Some(carina_core::schema::string_enum_identity("TextTransformationType", Some("awscc.wafv2.WebAcl"))),
                dsl_aliases: vec![("NONE".to_string(), "none".to_string()), ("COMPRESS_WHITE_SPACE".to_string(), "compress_white_space".to_string()), ("HTML_ENTITY_DECODE".to_string(), "html_entity_decode".to_string()), ("LOWERCASE".to_string(), "lowercase".to_string()), ("CMD_LINE".to_string(), "cmd_line".to_string()), ("URL_DECODE".to_string(), "url_decode".to_string()), ("BASE64_DECODE".to_string(), "base64_decode".to_string()), ("HEX_DECODE".to_string(), "hex_decode".to_string()), ("MD5".to_string(), "md5".to_string()), ("REPLACE_COMMENTS".to_string(), "replace_comments".to_string()), ("ESCAPE_SEQ_DECODE".to_string(), "escape_seq_decode".to_string()), ("SQL_HEX_DECODE".to_string(), "sql_hex_decode".to_string()), ("CSS_DECODE".to_string(), "css_decode".to_string()), ("JS_DECODE".to_string(), "js_decode".to_string()), ("NORMALIZE_PATH".to_string(), "normalize_path".to_string()), ("NORMALIZE_PATH_WIN".to_string(), "normalize_path_win".to_string()), ("REMOVE_NULLS".to_string(), "remove_nulls".to_string()), ("REPLACE_NULLS".to_string(), "replace_nulls".to_string()), ("BASE64_DECODE_EXT".to_string(), "base64_decode_ext".to_string()), ("URL_DECODE_UNI".to_string(), "url_decode_uni".to_string()), ("UTF8_TO_UNICODE".to_string(), "utf8_to_unicode".to_string())],
            }).required().with_provider_name("Type")
                    ],
                })).required().with_provider_name("TextTransformations").with_block_name("text_transformation")
                    ],
                }).with_provider_name("RegexPatternSetReferenceStatement").with_block_name("regex_pattern_set_reference_statement"),
                    StructField::new("rule_group_reference_statement", AttributeType::Struct {
                    name: "RuleGroupReferenceStatement".to_string(),
                    fields: vec![
                    StructField::new("arn", super::arn()).required().with_provider_name("Arn"),
                    StructField::new("excluded_rules", AttributeType::list(AttributeType::Struct {
                    name: "ExcludedRule".to_string(),
                    fields: vec![
                    StructField::new("name", AttributeType::Custom {
                identity: None,
                pattern: Some("^[0-9A-Za-z_-]{1,128}$".to_string()),
                length: None,
                base: Box::new(AttributeType::String),
                validate: legacy_validator(validate_string_pattern_42f7eceb887966ad),
                to_dsl: None,
            }).required().with_provider_name("Name")
                    ],
                })).with_provider_name("ExcludedRules").with_block_name("excluded_rule"),
                    StructField::new("rule_action_overrides", AttributeType::Custom {
                identity: None,
                pattern: None,
                length: None,
                base: Box::new(AttributeType::list(AttributeType::Struct {
                    name: "RuleActionOverride".to_string(),
                    fields: vec![
                    StructField::new("action_to_use", AttributeType::Ref("RuleAction".to_string())).required().with_provider_name("ActionToUse"),
                    StructField::new("name", AttributeType::Custom {
                identity: None,
                pattern: Some("^[0-9A-Za-z_-]{1,128}$".to_string()),
                length: None,
                base: Box::new(AttributeType::String),
                validate: legacy_validator(validate_string_pattern_42f7eceb887966ad),
                to_dsl: None,
            }).required().with_provider_name("Name")
                    ],
                })),
                validate: legacy_validator(validate_list_items_max_100),
                to_dsl: None,
            }).with_description("Action overrides for rules in the rule group.").with_provider_name("RuleActionOverrides").with_block_name("rule_action_override")
                    ],
                }).with_provider_name("RuleGroupReferenceStatement").with_block_name("rule_group_reference_statement"),
                    StructField::new("size_constraint_statement", AttributeType::Struct {
                    name: "SizeConstraintStatement".to_string(),
                    fields: vec![
                    StructField::new("comparison_operator", AttributeType::StringEnum {
                name: "ComparisonOperator".to_string(),
                values: vec!["EQ".to_string(), "NE".to_string(), "LE".to_string(), "LT".to_string(), "GE".to_string(), "GT".to_string()],
                identity: Some(carina_core::schema::string_enum_identity("ComparisonOperator", Some("awscc.wafv2.WebAcl"))),
                dsl_aliases: vec![("EQ".to_string(), "eq".to_string()), ("NE".to_string(), "ne".to_string()), ("LE".to_string(), "le".to_string()), ("LT".to_string(), "lt".to_string()), ("GE".to_string(), "ge".to_string()), ("GT".to_string(), "gt".to_string())],
            }).required().with_provider_name("ComparisonOperator"),
                    StructField::new("field_to_match", AttributeType::Ref("FieldToMatch".to_string())).required().with_provider_name("FieldToMatch"),
                    StructField::new("size", AttributeType::Custom {
                identity: None,
                pattern: None,
                length: None,
                base: Box::new(AttributeType::Float),
                validate: legacy_validator(validate_size_range),
                to_dsl: None,
            }).required().with_provider_name("Size"),
                    StructField::new("text_transformations", AttributeType::list(AttributeType::Struct {
                    name: "TextTransformation".to_string(),
                    fields: vec![
                    StructField::new("priority", AttributeType::String).required().with_provider_name("Priority"),
                    StructField::new("type", AttributeType::StringEnum {
                name: "TextTransformationType".to_string(),
                values: vec!["NONE".to_string(), "COMPRESS_WHITE_SPACE".to_string(), "HTML_ENTITY_DECODE".to_string(), "LOWERCASE".to_string(), "CMD_LINE".to_string(), "URL_DECODE".to_string(), "BASE64_DECODE".to_string(), "HEX_DECODE".to_string(), "MD5".to_string(), "REPLACE_COMMENTS".to_string(), "ESCAPE_SEQ_DECODE".to_string(), "SQL_HEX_DECODE".to_string(), "CSS_DECODE".to_string(), "JS_DECODE".to_string(), "NORMALIZE_PATH".to_string(), "NORMALIZE_PATH_WIN".to_string(), "REMOVE_NULLS".to_string(), "REPLACE_NULLS".to_string(), "BASE64_DECODE_EXT".to_string(), "URL_DECODE_UNI".to_string(), "UTF8_TO_UNICODE".to_string()],
                identity: Some(carina_core::schema::string_enum_identity("TextTransformationType", Some("awscc.wafv2.WebAcl"))),
                dsl_aliases: vec![("NONE".to_string(), "none".to_string()), ("COMPRESS_WHITE_SPACE".to_string(), "compress_white_space".to_string()), ("HTML_ENTITY_DECODE".to_string(), "html_entity_decode".to_string()), ("LOWERCASE".to_string(), "lowercase".to_string()), ("CMD_LINE".to_string(), "cmd_line".to_string()), ("URL_DECODE".to_string(), "url_decode".to_string()), ("BASE64_DECODE".to_string(), "base64_decode".to_string()), ("HEX_DECODE".to_string(), "hex_decode".to_string()), ("MD5".to_string(), "md5".to_string()), ("REPLACE_COMMENTS".to_string(), "replace_comments".to_string()), ("ESCAPE_SEQ_DECODE".to_string(), "escape_seq_decode".to_string()), ("SQL_HEX_DECODE".to_string(), "sql_hex_decode".to_string()), ("CSS_DECODE".to_string(), "css_decode".to_string()), ("JS_DECODE".to_string(), "js_decode".to_string()), ("NORMALIZE_PATH".to_string(), "normalize_path".to_string()), ("NORMALIZE_PATH_WIN".to_string(), "normalize_path_win".to_string()), ("REMOVE_NULLS".to_string(), "remove_nulls".to_string()), ("REPLACE_NULLS".to_string(), "replace_nulls".to_string()), ("BASE64_DECODE_EXT".to_string(), "base64_decode_ext".to_string()), ("URL_DECODE_UNI".to_string(), "url_decode_uni".to_string()), ("UTF8_TO_UNICODE".to_string(), "utf8_to_unicode".to_string())],
            }).required().with_provider_name("Type")
                    ],
                })).required().with_provider_name("TextTransformations").with_block_name("text_transformation")
                    ],
                }).with_provider_name("SizeConstraintStatement").with_block_name("size_constraint_statement"),
                    StructField::new("sqli_match_statement", AttributeType::Struct {
                    name: "SqliMatchStatement".to_string(),
                    fields: vec![
                    StructField::new("field_to_match", AttributeType::Ref("FieldToMatch".to_string())).required().with_provider_name("FieldToMatch"),
                    StructField::new("sensitivity_level", AttributeType::StringEnum {
                name: "SensitivityLevel".to_string(),
                values: vec!["LOW".to_string(), "HIGH".to_string()],
                identity: Some(carina_core::schema::string_enum_identity("SensitivityLevel", Some("awscc.wafv2.WebAcl"))),
                dsl_aliases: vec![("LOW".to_string(), "low".to_string()), ("HIGH".to_string(), "high".to_string())],
            }).with_provider_name("SensitivityLevel"),
                    StructField::new("text_transformations", AttributeType::list(AttributeType::Struct {
                    name: "TextTransformation".to_string(),
                    fields: vec![
                    StructField::new("priority", AttributeType::String).required().with_provider_name("Priority"),
                    StructField::new("type", AttributeType::StringEnum {
                name: "TextTransformationType".to_string(),
                values: vec!["NONE".to_string(), "COMPRESS_WHITE_SPACE".to_string(), "HTML_ENTITY_DECODE".to_string(), "LOWERCASE".to_string(), "CMD_LINE".to_string(), "URL_DECODE".to_string(), "BASE64_DECODE".to_string(), "HEX_DECODE".to_string(), "MD5".to_string(), "REPLACE_COMMENTS".to_string(), "ESCAPE_SEQ_DECODE".to_string(), "SQL_HEX_DECODE".to_string(), "CSS_DECODE".to_string(), "JS_DECODE".to_string(), "NORMALIZE_PATH".to_string(), "NORMALIZE_PATH_WIN".to_string(), "REMOVE_NULLS".to_string(), "REPLACE_NULLS".to_string(), "BASE64_DECODE_EXT".to_string(), "URL_DECODE_UNI".to_string(), "UTF8_TO_UNICODE".to_string()],
                identity: Some(carina_core::schema::string_enum_identity("TextTransformationType", Some("awscc.wafv2.WebAcl"))),
                dsl_aliases: vec![("NONE".to_string(), "none".to_string()), ("COMPRESS_WHITE_SPACE".to_string(), "compress_white_space".to_string()), ("HTML_ENTITY_DECODE".to_string(), "html_entity_decode".to_string()), ("LOWERCASE".to_string(), "lowercase".to_string()), ("CMD_LINE".to_string(), "cmd_line".to_string()), ("URL_DECODE".to_string(), "url_decode".to_string()), ("BASE64_DECODE".to_string(), "base64_decode".to_string()), ("HEX_DECODE".to_string(), "hex_decode".to_string()), ("MD5".to_string(), "md5".to_string()), ("REPLACE_COMMENTS".to_string(), "replace_comments".to_string()), ("ESCAPE_SEQ_DECODE".to_string(), "escape_seq_decode".to_string()), ("SQL_HEX_DECODE".to_string(), "sql_hex_decode".to_string()), ("CSS_DECODE".to_string(), "css_decode".to_string()), ("JS_DECODE".to_string(), "js_decode".to_string()), ("NORMALIZE_PATH".to_string(), "normalize_path".to_string()), ("NORMALIZE_PATH_WIN".to_string(), "normalize_path_win".to_string()), ("REMOVE_NULLS".to_string(), "remove_nulls".to_string()), ("REPLACE_NULLS".to_string(), "replace_nulls".to_string()), ("BASE64_DECODE_EXT".to_string(), "base64_decode_ext".to_string()), ("URL_DECODE_UNI".to_string(), "url_decode_uni".to_string()), ("UTF8_TO_UNICODE".to_string(), "utf8_to_unicode".to_string())],
            }).required().with_provider_name("Type")
                    ],
                })).required().with_provider_name("TextTransformations").with_block_name("text_transformation")
                    ],
                }).with_provider_name("SqliMatchStatement").with_block_name("sqli_match_statement"),
                    StructField::new("xss_match_statement", AttributeType::Struct {
                    name: "XssMatchStatement".to_string(),
                    fields: vec![
                    StructField::new("field_to_match", AttributeType::Ref("FieldToMatch".to_string())).required().with_provider_name("FieldToMatch"),
                    StructField::new("text_transformations", AttributeType::list(AttributeType::Struct {
                    name: "TextTransformation".to_string(),
                    fields: vec![
                    StructField::new("priority", AttributeType::String).required().with_provider_name("Priority"),
                    StructField::new("type", AttributeType::StringEnum {
                name: "TextTransformationType".to_string(),
                values: vec!["NONE".to_string(), "COMPRESS_WHITE_SPACE".to_string(), "HTML_ENTITY_DECODE".to_string(), "LOWERCASE".to_string(), "CMD_LINE".to_string(), "URL_DECODE".to_string(), "BASE64_DECODE".to_string(), "HEX_DECODE".to_string(), "MD5".to_string(), "REPLACE_COMMENTS".to_string(), "ESCAPE_SEQ_DECODE".to_string(), "SQL_HEX_DECODE".to_string(), "CSS_DECODE".to_string(), "JS_DECODE".to_string(), "NORMALIZE_PATH".to_string(), "NORMALIZE_PATH_WIN".to_string(), "REMOVE_NULLS".to_string(), "REPLACE_NULLS".to_string(), "BASE64_DECODE_EXT".to_string(), "URL_DECODE_UNI".to_string(), "UTF8_TO_UNICODE".to_string()],
                identity: Some(carina_core::schema::string_enum_identity("TextTransformationType", Some("awscc.wafv2.WebAcl"))),
                dsl_aliases: vec![("NONE".to_string(), "none".to_string()), ("COMPRESS_WHITE_SPACE".to_string(), "compress_white_space".to_string()), ("HTML_ENTITY_DECODE".to_string(), "html_entity_decode".to_string()), ("LOWERCASE".to_string(), "lowercase".to_string()), ("CMD_LINE".to_string(), "cmd_line".to_string()), ("URL_DECODE".to_string(), "url_decode".to_string()), ("BASE64_DECODE".to_string(), "base64_decode".to_string()), ("HEX_DECODE".to_string(), "hex_decode".to_string()), ("MD5".to_string(), "md5".to_string()), ("REPLACE_COMMENTS".to_string(), "replace_comments".to_string()), ("ESCAPE_SEQ_DECODE".to_string(), "escape_seq_decode".to_string()), ("SQL_HEX_DECODE".to_string(), "sql_hex_decode".to_string()), ("CSS_DECODE".to_string(), "css_decode".to_string()), ("JS_DECODE".to_string(), "js_decode".to_string()), ("NORMALIZE_PATH".to_string(), "normalize_path".to_string()), ("NORMALIZE_PATH_WIN".to_string(), "normalize_path_win".to_string()), ("REMOVE_NULLS".to_string(), "remove_nulls".to_string()), ("REPLACE_NULLS".to_string(), "replace_nulls".to_string()), ("BASE64_DECODE_EXT".to_string(), "base64_decode_ext".to_string()), ("URL_DECODE_UNI".to_string(), "url_decode_uni".to_string()), ("UTF8_TO_UNICODE".to_string(), "utf8_to_unicode".to_string())],
            }).required().with_provider_name("Type")
                    ],
                })).required().with_provider_name("TextTransformations").with_block_name("text_transformation")
                    ],
                }).with_provider_name("XssMatchStatement").with_block_name("xss_match_statement")
                    ],
                })).required().with_provider_name("Statements").with_block_name("statement")
                    ],
                })
        .with_def("OverrideAction", AttributeType::Struct {
                    name: "OverrideAction".to_string(),
                    fields: vec![
                    StructField::new("count", AttributeType::map(AttributeType::String)).with_description("Count traffic towards application.").with_provider_name("Count"),
                    StructField::new("none", AttributeType::map(AttributeType::String)).with_description("Keep the RuleGroup or ManagedRuleGroup behavior as is.").with_provider_name("None")
                    ],
                })
        .with_def("RateBasedStatement", AttributeType::Struct {
                    name: "RateBasedStatement".to_string(),
                    fields: vec![
                    StructField::new("aggregate_key_type", AttributeType::StringEnum {
                name: "AggregateKeyType".to_string(),
                values: vec!["CONSTANT".to_string(), "IP".to_string(), "FORWARDED_IP".to_string(), "CUSTOM_KEYS".to_string()],
                identity: Some(carina_core::schema::string_enum_identity("AggregateKeyType", Some("awscc.wafv2.WebAcl"))),
                dsl_aliases: vec![("CONSTANT".to_string(), "constant".to_string()), ("IP".to_string(), "ip".to_string()), ("FORWARDED_IP".to_string(), "forwarded_ip".to_string()), ("CUSTOM_KEYS".to_string(), "custom_keys".to_string())],
            }).required().with_provider_name("AggregateKeyType"),
                    StructField::new("custom_keys", AttributeType::Custom {
                identity: None,
                pattern: None,
                length: None,
                base: Box::new(AttributeType::list(AttributeType::Struct {
                    name: "RateBasedStatementCustomKey".to_string(),
                    fields: vec![
                    StructField::new("asn", AttributeType::String).with_provider_name("ASN"),
                    StructField::new("cookie", AttributeType::Struct {
                    name: "RateLimitCookie".to_string(),
                    fields: vec![
                    StructField::new("name", AttributeType::Custom {
                identity: None,
                pattern: Some(".*\\S.*".to_string()),
                length: Some((Some(1), Some(64))),
                base: Box::new(AttributeType::String),
                validate: legacy_validator(validate_string_pattern_4e09c821aed9e752_len_1_64),
                to_dsl: None,
            }).required().with_description("The name of the cookie to use.").with_provider_name("Name"),
                    StructField::new("text_transformations", AttributeType::list(AttributeType::Struct {
                    name: "TextTransformation".to_string(),
                    fields: vec![
                    StructField::new("priority", AttributeType::String).required().with_provider_name("Priority"),
                    StructField::new("type", AttributeType::StringEnum {
                name: "TextTransformationType".to_string(),
                values: vec!["NONE".to_string(), "COMPRESS_WHITE_SPACE".to_string(), "HTML_ENTITY_DECODE".to_string(), "LOWERCASE".to_string(), "CMD_LINE".to_string(), "URL_DECODE".to_string(), "BASE64_DECODE".to_string(), "HEX_DECODE".to_string(), "MD5".to_string(), "REPLACE_COMMENTS".to_string(), "ESCAPE_SEQ_DECODE".to_string(), "SQL_HEX_DECODE".to_string(), "CSS_DECODE".to_string(), "JS_DECODE".to_string(), "NORMALIZE_PATH".to_string(), "NORMALIZE_PATH_WIN".to_string(), "REMOVE_NULLS".to_string(), "REPLACE_NULLS".to_string(), "BASE64_DECODE_EXT".to_string(), "URL_DECODE_UNI".to_string(), "UTF8_TO_UNICODE".to_string()],
                identity: Some(carina_core::schema::string_enum_identity("TextTransformationType", Some("awscc.wafv2.WebAcl"))),
                dsl_aliases: vec![("NONE".to_string(), "none".to_string()), ("COMPRESS_WHITE_SPACE".to_string(), "compress_white_space".to_string()), ("HTML_ENTITY_DECODE".to_string(), "html_entity_decode".to_string()), ("LOWERCASE".to_string(), "lowercase".to_string()), ("CMD_LINE".to_string(), "cmd_line".to_string()), ("URL_DECODE".to_string(), "url_decode".to_string()), ("BASE64_DECODE".to_string(), "base64_decode".to_string()), ("HEX_DECODE".to_string(), "hex_decode".to_string()), ("MD5".to_string(), "md5".to_string()), ("REPLACE_COMMENTS".to_string(), "replace_comments".to_string()), ("ESCAPE_SEQ_DECODE".to_string(), "escape_seq_decode".to_string()), ("SQL_HEX_DECODE".to_string(), "sql_hex_decode".to_string()), ("CSS_DECODE".to_string(), "css_decode".to_string()), ("JS_DECODE".to_string(), "js_decode".to_string()), ("NORMALIZE_PATH".to_string(), "normalize_path".to_string()), ("NORMALIZE_PATH_WIN".to_string(), "normalize_path_win".to_string()), ("REMOVE_NULLS".to_string(), "remove_nulls".to_string()), ("REPLACE_NULLS".to_string(), "replace_nulls".to_string()), ("BASE64_DECODE_EXT".to_string(), "base64_decode_ext".to_string()), ("URL_DECODE_UNI".to_string(), "url_decode_uni".to_string()), ("UTF8_TO_UNICODE".to_string(), "utf8_to_unicode".to_string())],
            }).required().with_provider_name("Type")
                    ],
                })).required().with_provider_name("TextTransformations").with_block_name("text_transformation")
                    ],
                }).with_provider_name("Cookie").with_block_name("cookie"),
                    StructField::new("forwarded_ip", types::ipv4_address()).with_provider_name("ForwardedIP"),
                    StructField::new("http_method", AttributeType::String).with_provider_name("HTTPMethod"),
                    StructField::new("header", AttributeType::Struct {
                    name: "RateLimitHeader".to_string(),
                    fields: vec![
                    StructField::new("name", AttributeType::Custom {
                identity: None,
                pattern: Some(".*\\S.*".to_string()),
                length: Some((Some(1), Some(64))),
                base: Box::new(AttributeType::String),
                validate: legacy_validator(validate_string_pattern_4e09c821aed9e752_len_1_64),
                to_dsl: None,
            }).required().with_description("The name of the header to use.").with_provider_name("Name"),
                    StructField::new("text_transformations", AttributeType::list(AttributeType::Struct {
                    name: "TextTransformation".to_string(),
                    fields: vec![
                    StructField::new("priority", AttributeType::String).required().with_provider_name("Priority"),
                    StructField::new("type", AttributeType::StringEnum {
                name: "TextTransformationType".to_string(),
                values: vec!["NONE".to_string(), "COMPRESS_WHITE_SPACE".to_string(), "HTML_ENTITY_DECODE".to_string(), "LOWERCASE".to_string(), "CMD_LINE".to_string(), "URL_DECODE".to_string(), "BASE64_DECODE".to_string(), "HEX_DECODE".to_string(), "MD5".to_string(), "REPLACE_COMMENTS".to_string(), "ESCAPE_SEQ_DECODE".to_string(), "SQL_HEX_DECODE".to_string(), "CSS_DECODE".to_string(), "JS_DECODE".to_string(), "NORMALIZE_PATH".to_string(), "NORMALIZE_PATH_WIN".to_string(), "REMOVE_NULLS".to_string(), "REPLACE_NULLS".to_string(), "BASE64_DECODE_EXT".to_string(), "URL_DECODE_UNI".to_string(), "UTF8_TO_UNICODE".to_string()],
                identity: Some(carina_core::schema::string_enum_identity("TextTransformationType", Some("awscc.wafv2.WebAcl"))),
                dsl_aliases: vec![("NONE".to_string(), "none".to_string()), ("COMPRESS_WHITE_SPACE".to_string(), "compress_white_space".to_string()), ("HTML_ENTITY_DECODE".to_string(), "html_entity_decode".to_string()), ("LOWERCASE".to_string(), "lowercase".to_string()), ("CMD_LINE".to_string(), "cmd_line".to_string()), ("URL_DECODE".to_string(), "url_decode".to_string()), ("BASE64_DECODE".to_string(), "base64_decode".to_string()), ("HEX_DECODE".to_string(), "hex_decode".to_string()), ("MD5".to_string(), "md5".to_string()), ("REPLACE_COMMENTS".to_string(), "replace_comments".to_string()), ("ESCAPE_SEQ_DECODE".to_string(), "escape_seq_decode".to_string()), ("SQL_HEX_DECODE".to_string(), "sql_hex_decode".to_string()), ("CSS_DECODE".to_string(), "css_decode".to_string()), ("JS_DECODE".to_string(), "js_decode".to_string()), ("NORMALIZE_PATH".to_string(), "normalize_path".to_string()), ("NORMALIZE_PATH_WIN".to_string(), "normalize_path_win".to_string()), ("REMOVE_NULLS".to_string(), "remove_nulls".to_string()), ("REPLACE_NULLS".to_string(), "replace_nulls".to_string()), ("BASE64_DECODE_EXT".to_string(), "base64_decode_ext".to_string()), ("URL_DECODE_UNI".to_string(), "url_decode_uni".to_string()), ("UTF8_TO_UNICODE".to_string(), "utf8_to_unicode".to_string())],
            }).required().with_provider_name("Type")
                    ],
                })).required().with_provider_name("TextTransformations").with_block_name("text_transformation")
                    ],
                }).with_provider_name("Header").with_block_name("header"),
                    StructField::new("ip", types::ipv4_address()).with_provider_name("IP"),
                    StructField::new("ja3_fingerprint", AttributeType::Struct {
                    name: "RateLimitJA3Fingerprint".to_string(),
                    fields: vec![
                    StructField::new("fallback_behavior", AttributeType::StringEnum {
                name: "FallbackBehavior".to_string(),
                values: vec!["MATCH".to_string(), "NO_MATCH".to_string()],
                identity: Some(carina_core::schema::string_enum_identity("FallbackBehavior", Some("awscc.wafv2.WebAcl"))),
                dsl_aliases: vec![("MATCH".to_string(), "match".to_string()), ("NO_MATCH".to_string(), "no_match".to_string())],
            }).required().with_provider_name("FallbackBehavior")
                    ],
                }).with_provider_name("JA3Fingerprint"),
                    StructField::new("ja4_fingerprint", AttributeType::Struct {
                    name: "RateLimitJA4Fingerprint".to_string(),
                    fields: vec![
                    StructField::new("fallback_behavior", AttributeType::StringEnum {
                name: "FallbackBehavior".to_string(),
                values: vec!["MATCH".to_string(), "NO_MATCH".to_string()],
                identity: Some(carina_core::schema::string_enum_identity("FallbackBehavior", Some("awscc.wafv2.WebAcl"))),
                dsl_aliases: vec![("MATCH".to_string(), "match".to_string()), ("NO_MATCH".to_string(), "no_match".to_string())],
            }).required().with_provider_name("FallbackBehavior")
                    ],
                }).with_provider_name("JA4Fingerprint"),
                    StructField::new("label_namespace", AttributeType::Struct {
                    name: "RateLimitLabelNamespace".to_string(),
                    fields: vec![
                    StructField::new("namespace", AttributeType::Custom {
                identity: None,
                pattern: Some("^[0-9A-Za-z_:-]{1,1024}$".to_string()),
                length: None,
                base: Box::new(AttributeType::String),
                validate: legacy_validator(validate_string_pattern_b3fc65b549fb77bd),
                to_dsl: None,
            }).required().with_description("The namespace to use for aggregation.").with_provider_name("Namespace")
                    ],
                }).with_provider_name("LabelNamespace"),
                    StructField::new("query_argument", AttributeType::Struct {
                    name: "RateLimitQueryArgument".to_string(),
                    fields: vec![
                    StructField::new("name", AttributeType::Custom {
                identity: None,
                pattern: Some(".*\\S.*".to_string()),
                length: Some((Some(1), Some(64))),
                base: Box::new(AttributeType::String),
                validate: legacy_validator(validate_string_pattern_4e09c821aed9e752_len_1_64),
                to_dsl: None,
            }).required().with_description("The name of the query argument to use.").with_provider_name("Name"),
                    StructField::new("text_transformations", AttributeType::list(AttributeType::Struct {
                    name: "TextTransformation".to_string(),
                    fields: vec![
                    StructField::new("priority", AttributeType::String).required().with_provider_name("Priority"),
                    StructField::new("type", AttributeType::StringEnum {
                name: "TextTransformationType".to_string(),
                values: vec!["NONE".to_string(), "COMPRESS_WHITE_SPACE".to_string(), "HTML_ENTITY_DECODE".to_string(), "LOWERCASE".to_string(), "CMD_LINE".to_string(), "URL_DECODE".to_string(), "BASE64_DECODE".to_string(), "HEX_DECODE".to_string(), "MD5".to_string(), "REPLACE_COMMENTS".to_string(), "ESCAPE_SEQ_DECODE".to_string(), "SQL_HEX_DECODE".to_string(), "CSS_DECODE".to_string(), "JS_DECODE".to_string(), "NORMALIZE_PATH".to_string(), "NORMALIZE_PATH_WIN".to_string(), "REMOVE_NULLS".to_string(), "REPLACE_NULLS".to_string(), "BASE64_DECODE_EXT".to_string(), "URL_DECODE_UNI".to_string(), "UTF8_TO_UNICODE".to_string()],
                identity: Some(carina_core::schema::string_enum_identity("TextTransformationType", Some("awscc.wafv2.WebAcl"))),
                dsl_aliases: vec![("NONE".to_string(), "none".to_string()), ("COMPRESS_WHITE_SPACE".to_string(), "compress_white_space".to_string()), ("HTML_ENTITY_DECODE".to_string(), "html_entity_decode".to_string()), ("LOWERCASE".to_string(), "lowercase".to_string()), ("CMD_LINE".to_string(), "cmd_line".to_string()), ("URL_DECODE".to_string(), "url_decode".to_string()), ("BASE64_DECODE".to_string(), "base64_decode".to_string()), ("HEX_DECODE".to_string(), "hex_decode".to_string()), ("MD5".to_string(), "md5".to_string()), ("REPLACE_COMMENTS".to_string(), "replace_comments".to_string()), ("ESCAPE_SEQ_DECODE".to_string(), "escape_seq_decode".to_string()), ("SQL_HEX_DECODE".to_string(), "sql_hex_decode".to_string()), ("CSS_DECODE".to_string(), "css_decode".to_string()), ("JS_DECODE".to_string(), "js_decode".to_string()), ("NORMALIZE_PATH".to_string(), "normalize_path".to_string()), ("NORMALIZE_PATH_WIN".to_string(), "normalize_path_win".to_string()), ("REMOVE_NULLS".to_string(), "remove_nulls".to_string()), ("REPLACE_NULLS".to_string(), "replace_nulls".to_string()), ("BASE64_DECODE_EXT".to_string(), "base64_decode_ext".to_string()), ("URL_DECODE_UNI".to_string(), "url_decode_uni".to_string()), ("UTF8_TO_UNICODE".to_string(), "utf8_to_unicode".to_string())],
            }).required().with_provider_name("Type")
                    ],
                })).required().with_provider_name("TextTransformations").with_block_name("text_transformation")
                    ],
                }).with_provider_name("QueryArgument").with_block_name("query_argument"),
                    StructField::new("query_string", AttributeType::Struct {
                    name: "RateLimitQueryString".to_string(),
                    fields: vec![
                    StructField::new("text_transformations", AttributeType::list(AttributeType::Struct {
                    name: "TextTransformation".to_string(),
                    fields: vec![
                    StructField::new("priority", AttributeType::String).required().with_provider_name("Priority"),
                    StructField::new("type", AttributeType::StringEnum {
                name: "TextTransformationType".to_string(),
                values: vec!["NONE".to_string(), "COMPRESS_WHITE_SPACE".to_string(), "HTML_ENTITY_DECODE".to_string(), "LOWERCASE".to_string(), "CMD_LINE".to_string(), "URL_DECODE".to_string(), "BASE64_DECODE".to_string(), "HEX_DECODE".to_string(), "MD5".to_string(), "REPLACE_COMMENTS".to_string(), "ESCAPE_SEQ_DECODE".to_string(), "SQL_HEX_DECODE".to_string(), "CSS_DECODE".to_string(), "JS_DECODE".to_string(), "NORMALIZE_PATH".to_string(), "NORMALIZE_PATH_WIN".to_string(), "REMOVE_NULLS".to_string(), "REPLACE_NULLS".to_string(), "BASE64_DECODE_EXT".to_string(), "URL_DECODE_UNI".to_string(), "UTF8_TO_UNICODE".to_string()],
                identity: Some(carina_core::schema::string_enum_identity("TextTransformationType", Some("awscc.wafv2.WebAcl"))),
                dsl_aliases: vec![("NONE".to_string(), "none".to_string()), ("COMPRESS_WHITE_SPACE".to_string(), "compress_white_space".to_string()), ("HTML_ENTITY_DECODE".to_string(), "html_entity_decode".to_string()), ("LOWERCASE".to_string(), "lowercase".to_string()), ("CMD_LINE".to_string(), "cmd_line".to_string()), ("URL_DECODE".to_string(), "url_decode".to_string()), ("BASE64_DECODE".to_string(), "base64_decode".to_string()), ("HEX_DECODE".to_string(), "hex_decode".to_string()), ("MD5".to_string(), "md5".to_string()), ("REPLACE_COMMENTS".to_string(), "replace_comments".to_string()), ("ESCAPE_SEQ_DECODE".to_string(), "escape_seq_decode".to_string()), ("SQL_HEX_DECODE".to_string(), "sql_hex_decode".to_string()), ("CSS_DECODE".to_string(), "css_decode".to_string()), ("JS_DECODE".to_string(), "js_decode".to_string()), ("NORMALIZE_PATH".to_string(), "normalize_path".to_string()), ("NORMALIZE_PATH_WIN".to_string(), "normalize_path_win".to_string()), ("REMOVE_NULLS".to_string(), "remove_nulls".to_string()), ("REPLACE_NULLS".to_string(), "replace_nulls".to_string()), ("BASE64_DECODE_EXT".to_string(), "base64_decode_ext".to_string()), ("URL_DECODE_UNI".to_string(), "url_decode_uni".to_string()), ("UTF8_TO_UNICODE".to_string(), "utf8_to_unicode".to_string())],
            }).required().with_provider_name("Type")
                    ],
                })).required().with_provider_name("TextTransformations").with_block_name("text_transformation")
                    ],
                }).with_provider_name("QueryString").with_block_name("query_string"),
                    StructField::new("uri_path", AttributeType::Struct {
                    name: "RateLimitUriPath".to_string(),
                    fields: vec![
                    StructField::new("text_transformations", AttributeType::list(AttributeType::Struct {
                    name: "TextTransformation".to_string(),
                    fields: vec![
                    StructField::new("priority", AttributeType::String).required().with_provider_name("Priority"),
                    StructField::new("type", AttributeType::StringEnum {
                name: "TextTransformationType".to_string(),
                values: vec!["NONE".to_string(), "COMPRESS_WHITE_SPACE".to_string(), "HTML_ENTITY_DECODE".to_string(), "LOWERCASE".to_string(), "CMD_LINE".to_string(), "URL_DECODE".to_string(), "BASE64_DECODE".to_string(), "HEX_DECODE".to_string(), "MD5".to_string(), "REPLACE_COMMENTS".to_string(), "ESCAPE_SEQ_DECODE".to_string(), "SQL_HEX_DECODE".to_string(), "CSS_DECODE".to_string(), "JS_DECODE".to_string(), "NORMALIZE_PATH".to_string(), "NORMALIZE_PATH_WIN".to_string(), "REMOVE_NULLS".to_string(), "REPLACE_NULLS".to_string(), "BASE64_DECODE_EXT".to_string(), "URL_DECODE_UNI".to_string(), "UTF8_TO_UNICODE".to_string()],
                identity: Some(carina_core::schema::string_enum_identity("TextTransformationType", Some("awscc.wafv2.WebAcl"))),
                dsl_aliases: vec![("NONE".to_string(), "none".to_string()), ("COMPRESS_WHITE_SPACE".to_string(), "compress_white_space".to_string()), ("HTML_ENTITY_DECODE".to_string(), "html_entity_decode".to_string()), ("LOWERCASE".to_string(), "lowercase".to_string()), ("CMD_LINE".to_string(), "cmd_line".to_string()), ("URL_DECODE".to_string(), "url_decode".to_string()), ("BASE64_DECODE".to_string(), "base64_decode".to_string()), ("HEX_DECODE".to_string(), "hex_decode".to_string()), ("MD5".to_string(), "md5".to_string()), ("REPLACE_COMMENTS".to_string(), "replace_comments".to_string()), ("ESCAPE_SEQ_DECODE".to_string(), "escape_seq_decode".to_string()), ("SQL_HEX_DECODE".to_string(), "sql_hex_decode".to_string()), ("CSS_DECODE".to_string(), "css_decode".to_string()), ("JS_DECODE".to_string(), "js_decode".to_string()), ("NORMALIZE_PATH".to_string(), "normalize_path".to_string()), ("NORMALIZE_PATH_WIN".to_string(), "normalize_path_win".to_string()), ("REMOVE_NULLS".to_string(), "remove_nulls".to_string()), ("REPLACE_NULLS".to_string(), "replace_nulls".to_string()), ("BASE64_DECODE_EXT".to_string(), "base64_decode_ext".to_string()), ("URL_DECODE_UNI".to_string(), "url_decode_uni".to_string()), ("UTF8_TO_UNICODE".to_string(), "utf8_to_unicode".to_string())],
            }).required().with_provider_name("Type")
                    ],
                })).required().with_provider_name("TextTransformations").with_block_name("text_transformation")
                    ],
                }).with_provider_name("UriPath").with_block_name("uri_path")
                    ],
                })),
                validate: legacy_validator(validate_list_items_max_5),
                to_dsl: None,
            }).with_description("Specifies the aggregate keys to use in a rate-base rule.").with_provider_name("CustomKeys").with_block_name("custom_key"),
                    StructField::new("evaluation_window_sec", AttributeType::StringEnum {
                name: "EvaluationWindowSec".to_string(),
                values: vec!["60".to_string(), "120".to_string(), "300".to_string(), "600".to_string()],
                identity: Some(carina_core::schema::string_enum_identity("EvaluationWindowSec", Some("awscc.wafv2.WebAcl"))),
                dsl_aliases: vec![("60".to_string(), "60".to_string()), ("120".to_string(), "120".to_string()), ("300".to_string(), "300".to_string()), ("600".to_string(), "600".to_string())],
            }).with_provider_name("EvaluationWindowSec"),
                    StructField::new("forwarded_ip_config", AttributeType::Ref("ForwardedIPConfiguration".to_string())).with_provider_name("ForwardedIPConfig"),
                    StructField::new("limit", AttributeType::String).required().with_provider_name("Limit"),
                    StructField::new("scope_down_statement", AttributeType::Ref("Statement".to_string())).with_provider_name("ScopeDownStatement")
                    ],
                })
        .with_def("RateLimitCookie", AttributeType::Struct {
                    name: "RateLimitCookie".to_string(),
                    fields: vec![
                    StructField::new("name", AttributeType::Custom {
                identity: None,
                pattern: Some(".*\\S.*".to_string()),
                length: Some((Some(1), Some(64))),
                base: Box::new(AttributeType::String),
                validate: legacy_validator(validate_string_pattern_4e09c821aed9e752_len_1_64),
                to_dsl: None,
            }).required().with_description("The name of the cookie to use.").with_provider_name("Name"),
                    StructField::new("text_transformations", AttributeType::list(AttributeType::Struct {
                    name: "TextTransformation".to_string(),
                    fields: vec![
                    StructField::new("priority", AttributeType::String).required().with_provider_name("Priority"),
                    StructField::new("type", AttributeType::StringEnum {
                name: "TextTransformationType".to_string(),
                values: vec!["NONE".to_string(), "COMPRESS_WHITE_SPACE".to_string(), "HTML_ENTITY_DECODE".to_string(), "LOWERCASE".to_string(), "CMD_LINE".to_string(), "URL_DECODE".to_string(), "BASE64_DECODE".to_string(), "HEX_DECODE".to_string(), "MD5".to_string(), "REPLACE_COMMENTS".to_string(), "ESCAPE_SEQ_DECODE".to_string(), "SQL_HEX_DECODE".to_string(), "CSS_DECODE".to_string(), "JS_DECODE".to_string(), "NORMALIZE_PATH".to_string(), "NORMALIZE_PATH_WIN".to_string(), "REMOVE_NULLS".to_string(), "REPLACE_NULLS".to_string(), "BASE64_DECODE_EXT".to_string(), "URL_DECODE_UNI".to_string(), "UTF8_TO_UNICODE".to_string()],
                identity: Some(carina_core::schema::string_enum_identity("TextTransformationType", Some("awscc.wafv2.WebAcl"))),
                dsl_aliases: vec![("NONE".to_string(), "none".to_string()), ("COMPRESS_WHITE_SPACE".to_string(), "compress_white_space".to_string()), ("HTML_ENTITY_DECODE".to_string(), "html_entity_decode".to_string()), ("LOWERCASE".to_string(), "lowercase".to_string()), ("CMD_LINE".to_string(), "cmd_line".to_string()), ("URL_DECODE".to_string(), "url_decode".to_string()), ("BASE64_DECODE".to_string(), "base64_decode".to_string()), ("HEX_DECODE".to_string(), "hex_decode".to_string()), ("MD5".to_string(), "md5".to_string()), ("REPLACE_COMMENTS".to_string(), "replace_comments".to_string()), ("ESCAPE_SEQ_DECODE".to_string(), "escape_seq_decode".to_string()), ("SQL_HEX_DECODE".to_string(), "sql_hex_decode".to_string()), ("CSS_DECODE".to_string(), "css_decode".to_string()), ("JS_DECODE".to_string(), "js_decode".to_string()), ("NORMALIZE_PATH".to_string(), "normalize_path".to_string()), ("NORMALIZE_PATH_WIN".to_string(), "normalize_path_win".to_string()), ("REMOVE_NULLS".to_string(), "remove_nulls".to_string()), ("REPLACE_NULLS".to_string(), "replace_nulls".to_string()), ("BASE64_DECODE_EXT".to_string(), "base64_decode_ext".to_string()), ("URL_DECODE_UNI".to_string(), "url_decode_uni".to_string()), ("UTF8_TO_UNICODE".to_string(), "utf8_to_unicode".to_string())],
            }).required().with_provider_name("Type")
                    ],
                })).required().with_provider_name("TextTransformations").with_block_name("text_transformation")
                    ],
                })
        .with_def("RateLimitHeader", AttributeType::Struct {
                    name: "RateLimitHeader".to_string(),
                    fields: vec![
                    StructField::new("name", AttributeType::Custom {
                identity: None,
                pattern: Some(".*\\S.*".to_string()),
                length: Some((Some(1), Some(64))),
                base: Box::new(AttributeType::String),
                validate: legacy_validator(validate_string_pattern_4e09c821aed9e752_len_1_64),
                to_dsl: None,
            }).required().with_description("The name of the header to use.").with_provider_name("Name"),
                    StructField::new("text_transformations", AttributeType::list(AttributeType::Struct {
                    name: "TextTransformation".to_string(),
                    fields: vec![
                    StructField::new("priority", AttributeType::String).required().with_provider_name("Priority"),
                    StructField::new("type", AttributeType::StringEnum {
                name: "TextTransformationType".to_string(),
                values: vec!["NONE".to_string(), "COMPRESS_WHITE_SPACE".to_string(), "HTML_ENTITY_DECODE".to_string(), "LOWERCASE".to_string(), "CMD_LINE".to_string(), "URL_DECODE".to_string(), "BASE64_DECODE".to_string(), "HEX_DECODE".to_string(), "MD5".to_string(), "REPLACE_COMMENTS".to_string(), "ESCAPE_SEQ_DECODE".to_string(), "SQL_HEX_DECODE".to_string(), "CSS_DECODE".to_string(), "JS_DECODE".to_string(), "NORMALIZE_PATH".to_string(), "NORMALIZE_PATH_WIN".to_string(), "REMOVE_NULLS".to_string(), "REPLACE_NULLS".to_string(), "BASE64_DECODE_EXT".to_string(), "URL_DECODE_UNI".to_string(), "UTF8_TO_UNICODE".to_string()],
                identity: Some(carina_core::schema::string_enum_identity("TextTransformationType", Some("awscc.wafv2.WebAcl"))),
                dsl_aliases: vec![("NONE".to_string(), "none".to_string()), ("COMPRESS_WHITE_SPACE".to_string(), "compress_white_space".to_string()), ("HTML_ENTITY_DECODE".to_string(), "html_entity_decode".to_string()), ("LOWERCASE".to_string(), "lowercase".to_string()), ("CMD_LINE".to_string(), "cmd_line".to_string()), ("URL_DECODE".to_string(), "url_decode".to_string()), ("BASE64_DECODE".to_string(), "base64_decode".to_string()), ("HEX_DECODE".to_string(), "hex_decode".to_string()), ("MD5".to_string(), "md5".to_string()), ("REPLACE_COMMENTS".to_string(), "replace_comments".to_string()), ("ESCAPE_SEQ_DECODE".to_string(), "escape_seq_decode".to_string()), ("SQL_HEX_DECODE".to_string(), "sql_hex_decode".to_string()), ("CSS_DECODE".to_string(), "css_decode".to_string()), ("JS_DECODE".to_string(), "js_decode".to_string()), ("NORMALIZE_PATH".to_string(), "normalize_path".to_string()), ("NORMALIZE_PATH_WIN".to_string(), "normalize_path_win".to_string()), ("REMOVE_NULLS".to_string(), "remove_nulls".to_string()), ("REPLACE_NULLS".to_string(), "replace_nulls".to_string()), ("BASE64_DECODE_EXT".to_string(), "base64_decode_ext".to_string()), ("URL_DECODE_UNI".to_string(), "url_decode_uni".to_string()), ("UTF8_TO_UNICODE".to_string(), "utf8_to_unicode".to_string())],
            }).required().with_provider_name("Type")
                    ],
                })).required().with_provider_name("TextTransformations").with_block_name("text_transformation")
                    ],
                })
        .with_def("RateLimitJA3Fingerprint", AttributeType::Struct {
                    name: "RateLimitJA3Fingerprint".to_string(),
                    fields: vec![
                    StructField::new("fallback_behavior", AttributeType::StringEnum {
                name: "FallbackBehavior".to_string(),
                values: vec!["MATCH".to_string(), "NO_MATCH".to_string()],
                identity: Some(carina_core::schema::string_enum_identity("FallbackBehavior", Some("awscc.wafv2.WebAcl"))),
                dsl_aliases: vec![("MATCH".to_string(), "match".to_string()), ("NO_MATCH".to_string(), "no_match".to_string())],
            }).required().with_provider_name("FallbackBehavior")
                    ],
                })
        .with_def("RateLimitJA4Fingerprint", AttributeType::Struct {
                    name: "RateLimitJA4Fingerprint".to_string(),
                    fields: vec![
                    StructField::new("fallback_behavior", AttributeType::StringEnum {
                name: "FallbackBehavior".to_string(),
                values: vec!["MATCH".to_string(), "NO_MATCH".to_string()],
                identity: Some(carina_core::schema::string_enum_identity("FallbackBehavior", Some("awscc.wafv2.WebAcl"))),
                dsl_aliases: vec![("MATCH".to_string(), "match".to_string()), ("NO_MATCH".to_string(), "no_match".to_string())],
            }).required().with_provider_name("FallbackBehavior")
                    ],
                })
        .with_def("RateLimitLabelNamespace", AttributeType::Struct {
                    name: "RateLimitLabelNamespace".to_string(),
                    fields: vec![
                    StructField::new("namespace", AttributeType::Custom {
                identity: None,
                pattern: Some("^[0-9A-Za-z_:-]{1,1024}$".to_string()),
                length: None,
                base: Box::new(AttributeType::String),
                validate: legacy_validator(validate_string_pattern_b3fc65b549fb77bd),
                to_dsl: None,
            }).required().with_description("The namespace to use for aggregation.").with_provider_name("Namespace")
                    ],
                })
        .with_def("RateLimitQueryArgument", AttributeType::Struct {
                    name: "RateLimitQueryArgument".to_string(),
                    fields: vec![
                    StructField::new("name", AttributeType::Custom {
                identity: None,
                pattern: Some(".*\\S.*".to_string()),
                length: Some((Some(1), Some(64))),
                base: Box::new(AttributeType::String),
                validate: legacy_validator(validate_string_pattern_4e09c821aed9e752_len_1_64),
                to_dsl: None,
            }).required().with_description("The name of the query argument to use.").with_provider_name("Name"),
                    StructField::new("text_transformations", AttributeType::list(AttributeType::Struct {
                    name: "TextTransformation".to_string(),
                    fields: vec![
                    StructField::new("priority", AttributeType::String).required().with_provider_name("Priority"),
                    StructField::new("type", AttributeType::StringEnum {
                name: "TextTransformationType".to_string(),
                values: vec!["NONE".to_string(), "COMPRESS_WHITE_SPACE".to_string(), "HTML_ENTITY_DECODE".to_string(), "LOWERCASE".to_string(), "CMD_LINE".to_string(), "URL_DECODE".to_string(), "BASE64_DECODE".to_string(), "HEX_DECODE".to_string(), "MD5".to_string(), "REPLACE_COMMENTS".to_string(), "ESCAPE_SEQ_DECODE".to_string(), "SQL_HEX_DECODE".to_string(), "CSS_DECODE".to_string(), "JS_DECODE".to_string(), "NORMALIZE_PATH".to_string(), "NORMALIZE_PATH_WIN".to_string(), "REMOVE_NULLS".to_string(), "REPLACE_NULLS".to_string(), "BASE64_DECODE_EXT".to_string(), "URL_DECODE_UNI".to_string(), "UTF8_TO_UNICODE".to_string()],
                identity: Some(carina_core::schema::string_enum_identity("TextTransformationType", Some("awscc.wafv2.WebAcl"))),
                dsl_aliases: vec![("NONE".to_string(), "none".to_string()), ("COMPRESS_WHITE_SPACE".to_string(), "compress_white_space".to_string()), ("HTML_ENTITY_DECODE".to_string(), "html_entity_decode".to_string()), ("LOWERCASE".to_string(), "lowercase".to_string()), ("CMD_LINE".to_string(), "cmd_line".to_string()), ("URL_DECODE".to_string(), "url_decode".to_string()), ("BASE64_DECODE".to_string(), "base64_decode".to_string()), ("HEX_DECODE".to_string(), "hex_decode".to_string()), ("MD5".to_string(), "md5".to_string()), ("REPLACE_COMMENTS".to_string(), "replace_comments".to_string()), ("ESCAPE_SEQ_DECODE".to_string(), "escape_seq_decode".to_string()), ("SQL_HEX_DECODE".to_string(), "sql_hex_decode".to_string()), ("CSS_DECODE".to_string(), "css_decode".to_string()), ("JS_DECODE".to_string(), "js_decode".to_string()), ("NORMALIZE_PATH".to_string(), "normalize_path".to_string()), ("NORMALIZE_PATH_WIN".to_string(), "normalize_path_win".to_string()), ("REMOVE_NULLS".to_string(), "remove_nulls".to_string()), ("REPLACE_NULLS".to_string(), "replace_nulls".to_string()), ("BASE64_DECODE_EXT".to_string(), "base64_decode_ext".to_string()), ("URL_DECODE_UNI".to_string(), "url_decode_uni".to_string()), ("UTF8_TO_UNICODE".to_string(), "utf8_to_unicode".to_string())],
            }).required().with_provider_name("Type")
                    ],
                })).required().with_provider_name("TextTransformations").with_block_name("text_transformation")
                    ],
                })
        .with_def("RateLimitQueryString", AttributeType::Struct {
                    name: "RateLimitQueryString".to_string(),
                    fields: vec![
                    StructField::new("text_transformations", AttributeType::list(AttributeType::Struct {
                    name: "TextTransformation".to_string(),
                    fields: vec![
                    StructField::new("priority", AttributeType::String).required().with_provider_name("Priority"),
                    StructField::new("type", AttributeType::StringEnum {
                name: "TextTransformationType".to_string(),
                values: vec!["NONE".to_string(), "COMPRESS_WHITE_SPACE".to_string(), "HTML_ENTITY_DECODE".to_string(), "LOWERCASE".to_string(), "CMD_LINE".to_string(), "URL_DECODE".to_string(), "BASE64_DECODE".to_string(), "HEX_DECODE".to_string(), "MD5".to_string(), "REPLACE_COMMENTS".to_string(), "ESCAPE_SEQ_DECODE".to_string(), "SQL_HEX_DECODE".to_string(), "CSS_DECODE".to_string(), "JS_DECODE".to_string(), "NORMALIZE_PATH".to_string(), "NORMALIZE_PATH_WIN".to_string(), "REMOVE_NULLS".to_string(), "REPLACE_NULLS".to_string(), "BASE64_DECODE_EXT".to_string(), "URL_DECODE_UNI".to_string(), "UTF8_TO_UNICODE".to_string()],
                identity: Some(carina_core::schema::string_enum_identity("TextTransformationType", Some("awscc.wafv2.WebAcl"))),
                dsl_aliases: vec![("NONE".to_string(), "none".to_string()), ("COMPRESS_WHITE_SPACE".to_string(), "compress_white_space".to_string()), ("HTML_ENTITY_DECODE".to_string(), "html_entity_decode".to_string()), ("LOWERCASE".to_string(), "lowercase".to_string()), ("CMD_LINE".to_string(), "cmd_line".to_string()), ("URL_DECODE".to_string(), "url_decode".to_string()), ("BASE64_DECODE".to_string(), "base64_decode".to_string()), ("HEX_DECODE".to_string(), "hex_decode".to_string()), ("MD5".to_string(), "md5".to_string()), ("REPLACE_COMMENTS".to_string(), "replace_comments".to_string()), ("ESCAPE_SEQ_DECODE".to_string(), "escape_seq_decode".to_string()), ("SQL_HEX_DECODE".to_string(), "sql_hex_decode".to_string()), ("CSS_DECODE".to_string(), "css_decode".to_string()), ("JS_DECODE".to_string(), "js_decode".to_string()), ("NORMALIZE_PATH".to_string(), "normalize_path".to_string()), ("NORMALIZE_PATH_WIN".to_string(), "normalize_path_win".to_string()), ("REMOVE_NULLS".to_string(), "remove_nulls".to_string()), ("REPLACE_NULLS".to_string(), "replace_nulls".to_string()), ("BASE64_DECODE_EXT".to_string(), "base64_decode_ext".to_string()), ("URL_DECODE_UNI".to_string(), "url_decode_uni".to_string()), ("UTF8_TO_UNICODE".to_string(), "utf8_to_unicode".to_string())],
            }).required().with_provider_name("Type")
                    ],
                })).required().with_provider_name("TextTransformations").with_block_name("text_transformation")
                    ],
                })
        .with_def("RateLimitUriPath", AttributeType::Struct {
                    name: "RateLimitUriPath".to_string(),
                    fields: vec![
                    StructField::new("text_transformations", AttributeType::list(AttributeType::Struct {
                    name: "TextTransformation".to_string(),
                    fields: vec![
                    StructField::new("priority", AttributeType::String).required().with_provider_name("Priority"),
                    StructField::new("type", AttributeType::StringEnum {
                name: "TextTransformationType".to_string(),
                values: vec!["NONE".to_string(), "COMPRESS_WHITE_SPACE".to_string(), "HTML_ENTITY_DECODE".to_string(), "LOWERCASE".to_string(), "CMD_LINE".to_string(), "URL_DECODE".to_string(), "BASE64_DECODE".to_string(), "HEX_DECODE".to_string(), "MD5".to_string(), "REPLACE_COMMENTS".to_string(), "ESCAPE_SEQ_DECODE".to_string(), "SQL_HEX_DECODE".to_string(), "CSS_DECODE".to_string(), "JS_DECODE".to_string(), "NORMALIZE_PATH".to_string(), "NORMALIZE_PATH_WIN".to_string(), "REMOVE_NULLS".to_string(), "REPLACE_NULLS".to_string(), "BASE64_DECODE_EXT".to_string(), "URL_DECODE_UNI".to_string(), "UTF8_TO_UNICODE".to_string()],
                identity: Some(carina_core::schema::string_enum_identity("TextTransformationType", Some("awscc.wafv2.WebAcl"))),
                dsl_aliases: vec![("NONE".to_string(), "none".to_string()), ("COMPRESS_WHITE_SPACE".to_string(), "compress_white_space".to_string()), ("HTML_ENTITY_DECODE".to_string(), "html_entity_decode".to_string()), ("LOWERCASE".to_string(), "lowercase".to_string()), ("CMD_LINE".to_string(), "cmd_line".to_string()), ("URL_DECODE".to_string(), "url_decode".to_string()), ("BASE64_DECODE".to_string(), "base64_decode".to_string()), ("HEX_DECODE".to_string(), "hex_decode".to_string()), ("MD5".to_string(), "md5".to_string()), ("REPLACE_COMMENTS".to_string(), "replace_comments".to_string()), ("ESCAPE_SEQ_DECODE".to_string(), "escape_seq_decode".to_string()), ("SQL_HEX_DECODE".to_string(), "sql_hex_decode".to_string()), ("CSS_DECODE".to_string(), "css_decode".to_string()), ("JS_DECODE".to_string(), "js_decode".to_string()), ("NORMALIZE_PATH".to_string(), "normalize_path".to_string()), ("NORMALIZE_PATH_WIN".to_string(), "normalize_path_win".to_string()), ("REMOVE_NULLS".to_string(), "remove_nulls".to_string()), ("REPLACE_NULLS".to_string(), "replace_nulls".to_string()), ("BASE64_DECODE_EXT".to_string(), "base64_decode_ext".to_string()), ("URL_DECODE_UNI".to_string(), "url_decode_uni".to_string()), ("UTF8_TO_UNICODE".to_string(), "utf8_to_unicode".to_string())],
            }).required().with_provider_name("Type")
                    ],
                })).required().with_provider_name("TextTransformations").with_block_name("text_transformation")
                    ],
                })
        .with_def("Regex", AttributeType::Struct {
                    name: "Regex".to_string(),
                    fields: vec![
                    StructField::new("regex_string", AttributeType::String).with_provider_name("RegexString")
                    ],
                })
        .with_def("RegexMatchStatement", AttributeType::Struct {
                    name: "RegexMatchStatement".to_string(),
                    fields: vec![
                    StructField::new("field_to_match", AttributeType::Ref("FieldToMatch".to_string())).required().with_provider_name("FieldToMatch"),
                    StructField::new("regex_string", AttributeType::Custom {
                identity: None,
                pattern: None,
                length: Some((Some(1), Some(512))),
                base: Box::new(AttributeType::String),
                validate: legacy_validator(validate_string_length_1_512),
                to_dsl: None,
            }).required().with_provider_name("RegexString"),
                    StructField::new("text_transformations", AttributeType::list(AttributeType::Struct {
                    name: "TextTransformation".to_string(),
                    fields: vec![
                    StructField::new("priority", AttributeType::String).required().with_provider_name("Priority"),
                    StructField::new("type", AttributeType::StringEnum {
                name: "TextTransformationType".to_string(),
                values: vec!["NONE".to_string(), "COMPRESS_WHITE_SPACE".to_string(), "HTML_ENTITY_DECODE".to_string(), "LOWERCASE".to_string(), "CMD_LINE".to_string(), "URL_DECODE".to_string(), "BASE64_DECODE".to_string(), "HEX_DECODE".to_string(), "MD5".to_string(), "REPLACE_COMMENTS".to_string(), "ESCAPE_SEQ_DECODE".to_string(), "SQL_HEX_DECODE".to_string(), "CSS_DECODE".to_string(), "JS_DECODE".to_string(), "NORMALIZE_PATH".to_string(), "NORMALIZE_PATH_WIN".to_string(), "REMOVE_NULLS".to_string(), "REPLACE_NULLS".to_string(), "BASE64_DECODE_EXT".to_string(), "URL_DECODE_UNI".to_string(), "UTF8_TO_UNICODE".to_string()],
                identity: Some(carina_core::schema::string_enum_identity("TextTransformationType", Some("awscc.wafv2.WebAcl"))),
                dsl_aliases: vec![("NONE".to_string(), "none".to_string()), ("COMPRESS_WHITE_SPACE".to_string(), "compress_white_space".to_string()), ("HTML_ENTITY_DECODE".to_string(), "html_entity_decode".to_string()), ("LOWERCASE".to_string(), "lowercase".to_string()), ("CMD_LINE".to_string(), "cmd_line".to_string()), ("URL_DECODE".to_string(), "url_decode".to_string()), ("BASE64_DECODE".to_string(), "base64_decode".to_string()), ("HEX_DECODE".to_string(), "hex_decode".to_string()), ("MD5".to_string(), "md5".to_string()), ("REPLACE_COMMENTS".to_string(), "replace_comments".to_string()), ("ESCAPE_SEQ_DECODE".to_string(), "escape_seq_decode".to_string()), ("SQL_HEX_DECODE".to_string(), "sql_hex_decode".to_string()), ("CSS_DECODE".to_string(), "css_decode".to_string()), ("JS_DECODE".to_string(), "js_decode".to_string()), ("NORMALIZE_PATH".to_string(), "normalize_path".to_string()), ("NORMALIZE_PATH_WIN".to_string(), "normalize_path_win".to_string()), ("REMOVE_NULLS".to_string(), "remove_nulls".to_string()), ("REPLACE_NULLS".to_string(), "replace_nulls".to_string()), ("BASE64_DECODE_EXT".to_string(), "base64_decode_ext".to_string()), ("URL_DECODE_UNI".to_string(), "url_decode_uni".to_string()), ("UTF8_TO_UNICODE".to_string(), "utf8_to_unicode".to_string())],
            }).required().with_provider_name("Type")
                    ],
                })).required().with_provider_name("TextTransformations").with_block_name("text_transformation")
                    ],
                })
        .with_def("RegexPatternSetReferenceStatement", AttributeType::Struct {
                    name: "RegexPatternSetReferenceStatement".to_string(),
                    fields: vec![
                    StructField::new("arn", super::arn()).required().with_provider_name("Arn"),
                    StructField::new("field_to_match", AttributeType::Ref("FieldToMatch".to_string())).required().with_provider_name("FieldToMatch"),
                    StructField::new("text_transformations", AttributeType::list(AttributeType::Struct {
                    name: "TextTransformation".to_string(),
                    fields: vec![
                    StructField::new("priority", AttributeType::String).required().with_provider_name("Priority"),
                    StructField::new("type", AttributeType::StringEnum {
                name: "TextTransformationType".to_string(),
                values: vec!["NONE".to_string(), "COMPRESS_WHITE_SPACE".to_string(), "HTML_ENTITY_DECODE".to_string(), "LOWERCASE".to_string(), "CMD_LINE".to_string(), "URL_DECODE".to_string(), "BASE64_DECODE".to_string(), "HEX_DECODE".to_string(), "MD5".to_string(), "REPLACE_COMMENTS".to_string(), "ESCAPE_SEQ_DECODE".to_string(), "SQL_HEX_DECODE".to_string(), "CSS_DECODE".to_string(), "JS_DECODE".to_string(), "NORMALIZE_PATH".to_string(), "NORMALIZE_PATH_WIN".to_string(), "REMOVE_NULLS".to_string(), "REPLACE_NULLS".to_string(), "BASE64_DECODE_EXT".to_string(), "URL_DECODE_UNI".to_string(), "UTF8_TO_UNICODE".to_string()],
                identity: Some(carina_core::schema::string_enum_identity("TextTransformationType", Some("awscc.wafv2.WebAcl"))),
                dsl_aliases: vec![("NONE".to_string(), "none".to_string()), ("COMPRESS_WHITE_SPACE".to_string(), "compress_white_space".to_string()), ("HTML_ENTITY_DECODE".to_string(), "html_entity_decode".to_string()), ("LOWERCASE".to_string(), "lowercase".to_string()), ("CMD_LINE".to_string(), "cmd_line".to_string()), ("URL_DECODE".to_string(), "url_decode".to_string()), ("BASE64_DECODE".to_string(), "base64_decode".to_string()), ("HEX_DECODE".to_string(), "hex_decode".to_string()), ("MD5".to_string(), "md5".to_string()), ("REPLACE_COMMENTS".to_string(), "replace_comments".to_string()), ("ESCAPE_SEQ_DECODE".to_string(), "escape_seq_decode".to_string()), ("SQL_HEX_DECODE".to_string(), "sql_hex_decode".to_string()), ("CSS_DECODE".to_string(), "css_decode".to_string()), ("JS_DECODE".to_string(), "js_decode".to_string()), ("NORMALIZE_PATH".to_string(), "normalize_path".to_string()), ("NORMALIZE_PATH_WIN".to_string(), "normalize_path_win".to_string()), ("REMOVE_NULLS".to_string(), "remove_nulls".to_string()), ("REPLACE_NULLS".to_string(), "replace_nulls".to_string()), ("BASE64_DECODE_EXT".to_string(), "base64_decode_ext".to_string()), ("URL_DECODE_UNI".to_string(), "url_decode_uni".to_string()), ("UTF8_TO_UNICODE".to_string(), "utf8_to_unicode".to_string())],
            }).required().with_provider_name("Type")
                    ],
                })).required().with_provider_name("TextTransformations").with_block_name("text_transformation")
                    ],
                })
        .with_def("RequestInspection", AttributeType::Struct {
                    name: "RequestInspection".to_string(),
                    fields: vec![
                    StructField::new("password_field", AttributeType::Ref("FieldIdentifier".to_string())).required().with_provider_name("PasswordField"),
                    StructField::new("payload_type", AttributeType::StringEnum {
                name: "PayloadType".to_string(),
                values: vec!["JSON".to_string(), "FORM_ENCODED".to_string()],
                identity: Some(carina_core::schema::string_enum_identity("PayloadType", Some("awscc.wafv2.WebAcl"))),
                dsl_aliases: vec![("JSON".to_string(), "json".to_string()), ("FORM_ENCODED".to_string(), "form_encoded".to_string())],
            }).required().with_provider_name("PayloadType"),
                    StructField::new("username_field", AttributeType::Ref("FieldIdentifier".to_string())).required().with_provider_name("UsernameField")
                    ],
                })
        .with_def("RequestInspectionACFP", AttributeType::Struct {
                    name: "RequestInspectionACFP".to_string(),
                    fields: vec![
                    StructField::new("address_fields", AttributeType::list(AttributeType::String)).with_provider_name("AddressFields"),
                    StructField::new("email_field", AttributeType::Struct {
                    name: "FieldIdentifier".to_string(),
                    fields: vec![
                    StructField::new("identifier", AttributeType::Custom {
                identity: None,
                pattern: Some(".*\\S.*".to_string()),
                length: Some((Some(1), Some(512))),
                base: Box::new(AttributeType::String),
                validate: legacy_validator(validate_string_pattern_4e09c821aed9e752_len_1_512),
                to_dsl: None,
            }).required().with_provider_name("Identifier")
                    ],
                }).with_provider_name("EmailField"),
                    StructField::new("password_field", AttributeType::Ref("FieldIdentifier".to_string())).with_provider_name("PasswordField"),
                    StructField::new("payload_type", AttributeType::StringEnum {
                name: "PayloadType".to_string(),
                values: vec!["JSON".to_string(), "FORM_ENCODED".to_string()],
                identity: Some(carina_core::schema::string_enum_identity("PayloadType", Some("awscc.wafv2.WebAcl"))),
                dsl_aliases: vec![("JSON".to_string(), "json".to_string()), ("FORM_ENCODED".to_string(), "form_encoded".to_string())],
            }).required().with_provider_name("PayloadType"),
                    StructField::new("phone_number_fields", AttributeType::list(AttributeType::String)).with_provider_name("PhoneNumberFields"),
                    StructField::new("username_field", AttributeType::Ref("FieldIdentifier".to_string())).with_provider_name("UsernameField")
                    ],
                })
        .with_def("ResponseInspection", AttributeType::Struct {
                    name: "ResponseInspection".to_string(),
                    fields: vec![
                    StructField::new("body_contains", AttributeType::Struct {
                    name: "ResponseInspectionBodyContains".to_string(),
                    fields: vec![
                    StructField::new("failure_strings", AttributeType::Custom {
                identity: None,
                pattern: None,
                length: None,
                base: Box::new(AttributeType::list(AttributeType::Custom {
                identity: None,
                pattern: Some(".*\\S.*".to_string()),
                length: Some((Some(1), Some(100))),
                base: Box::new(AttributeType::String),
                validate: legacy_validator(validate_string_pattern_4e09c821aed9e752_len_1_100),
                to_dsl: None,
            })),
                validate: legacy_validator(validate_list_items_1_5),
                to_dsl: None,
            }).required().with_provider_name("FailureStrings"),
                    StructField::new("success_strings", AttributeType::Custom {
                identity: None,
                pattern: None,
                length: None,
                base: Box::new(AttributeType::list(AttributeType::Custom {
                identity: None,
                pattern: Some(".*\\S.*".to_string()),
                length: Some((Some(1), Some(100))),
                base: Box::new(AttributeType::String),
                validate: legacy_validator(validate_string_pattern_4e09c821aed9e752_len_1_100),
                to_dsl: None,
            })),
                validate: legacy_validator(validate_list_items_1_5),
                to_dsl: None,
            }).required().with_provider_name("SuccessStrings")
                    ],
                }).with_provider_name("BodyContains"),
                    StructField::new("header", AttributeType::Struct {
                    name: "ResponseInspectionHeader".to_string(),
                    fields: vec![
                    StructField::new("failure_values", AttributeType::Custom {
                identity: None,
                pattern: None,
                length: None,
                base: Box::new(AttributeType::list(AttributeType::Custom {
                identity: None,
                pattern: Some(".*\\S.*".to_string()),
                length: Some((Some(1), Some(100))),
                base: Box::new(AttributeType::String),
                validate: legacy_validator(validate_string_pattern_4e09c821aed9e752_len_1_100),
                to_dsl: None,
            })),
                validate: legacy_validator(validate_list_items_1_3),
                to_dsl: None,
            }).required().with_provider_name("FailureValues"),
                    StructField::new("name", AttributeType::Custom {
                identity: None,
                pattern: Some(".*\\S.*".to_string()),
                length: Some((Some(1), Some(200))),
                base: Box::new(AttributeType::String),
                validate: legacy_validator(validate_string_pattern_4e09c821aed9e752_len_1_200),
                to_dsl: None,
            }).required().with_provider_name("Name"),
                    StructField::new("success_values", AttributeType::Custom {
                identity: None,
                pattern: None,
                length: None,
                base: Box::new(AttributeType::list(AttributeType::Custom {
                identity: None,
                pattern: Some(".*\\S.*".to_string()),
                length: Some((Some(1), Some(100))),
                base: Box::new(AttributeType::String),
                validate: legacy_validator(validate_string_pattern_4e09c821aed9e752_len_1_100),
                to_dsl: None,
            })),
                validate: legacy_validator(validate_list_items_1_3),
                to_dsl: None,
            }).required().with_provider_name("SuccessValues")
                    ],
                }).with_provider_name("Header"),
                    StructField::new("json", AttributeType::Struct {
                    name: "ResponseInspectionJson".to_string(),
                    fields: vec![
                    StructField::new("failure_values", AttributeType::Custom {
                identity: None,
                pattern: None,
                length: None,
                base: Box::new(AttributeType::list(AttributeType::Custom {
                identity: None,
                pattern: Some(".*\\S.*".to_string()),
                length: Some((Some(1), Some(100))),
                base: Box::new(AttributeType::String),
                validate: legacy_validator(validate_string_pattern_4e09c821aed9e752_len_1_100),
                to_dsl: None,
            })),
                validate: legacy_validator(validate_list_items_1_5),
                to_dsl: None,
            }).required().with_provider_name("FailureValues"),
                    StructField::new("identifier", AttributeType::Custom {
                identity: None,
                pattern: Some(".*\\S.*".to_string()),
                length: Some((Some(1), Some(512))),
                base: Box::new(AttributeType::String),
                validate: legacy_validator(validate_string_pattern_4e09c821aed9e752_len_1_512),
                to_dsl: None,
            }).required().with_provider_name("Identifier"),
                    StructField::new("success_values", AttributeType::Custom {
                identity: None,
                pattern: None,
                length: None,
                base: Box::new(AttributeType::list(AttributeType::Custom {
                identity: None,
                pattern: Some(".*\\S.*".to_string()),
                length: Some((Some(1), Some(100))),
                base: Box::new(AttributeType::String),
                validate: legacy_validator(validate_string_pattern_4e09c821aed9e752_len_1_100),
                to_dsl: None,
            })),
                validate: legacy_validator(validate_list_items_1_5),
                to_dsl: None,
            }).required().with_provider_name("SuccessValues")
                    ],
                }).with_provider_name("Json"),
                    StructField::new("status_code", AttributeType::Struct {
                    name: "ResponseInspectionStatusCode".to_string(),
                    fields: vec![
                    StructField::new("failure_codes", AttributeType::Custom {
                identity: None,
                pattern: None,
                length: None,
                base: Box::new(AttributeType::list(AttributeType::Int)),
                validate: legacy_validator(validate_list_items_1_10),
                to_dsl: None,
            }).required().with_provider_name("FailureCodes"),
                    StructField::new("success_codes", AttributeType::Custom {
                identity: None,
                pattern: None,
                length: None,
                base: Box::new(AttributeType::list(AttributeType::Int)),
                validate: legacy_validator(validate_list_items_1_10),
                to_dsl: None,
            }).required().with_provider_name("SuccessCodes")
                    ],
                }).with_provider_name("StatusCode")
                    ],
                })
        .with_def("ResponseInspectionBodyContains", AttributeType::Struct {
                    name: "ResponseInspectionBodyContains".to_string(),
                    fields: vec![
                    StructField::new("failure_strings", AttributeType::Custom {
                identity: None,
                pattern: None,
                length: None,
                base: Box::new(AttributeType::list(AttributeType::Custom {
                identity: None,
                pattern: Some(".*\\S.*".to_string()),
                length: Some((Some(1), Some(100))),
                base: Box::new(AttributeType::String),
                validate: legacy_validator(validate_string_pattern_4e09c821aed9e752_len_1_100),
                to_dsl: None,
            })),
                validate: legacy_validator(validate_list_items_1_5),
                to_dsl: None,
            }).required().with_provider_name("FailureStrings"),
                    StructField::new("success_strings", AttributeType::Custom {
                identity: None,
                pattern: None,
                length: None,
                base: Box::new(AttributeType::list(AttributeType::Custom {
                identity: None,
                pattern: Some(".*\\S.*".to_string()),
                length: Some((Some(1), Some(100))),
                base: Box::new(AttributeType::String),
                validate: legacy_validator(validate_string_pattern_4e09c821aed9e752_len_1_100),
                to_dsl: None,
            })),
                validate: legacy_validator(validate_list_items_1_5),
                to_dsl: None,
            }).required().with_provider_name("SuccessStrings")
                    ],
                })
        .with_def("ResponseInspectionHeader", AttributeType::Struct {
                    name: "ResponseInspectionHeader".to_string(),
                    fields: vec![
                    StructField::new("failure_values", AttributeType::Custom {
                identity: None,
                pattern: None,
                length: None,
                base: Box::new(AttributeType::list(AttributeType::Custom {
                identity: None,
                pattern: Some(".*\\S.*".to_string()),
                length: Some((Some(1), Some(100))),
                base: Box::new(AttributeType::String),
                validate: legacy_validator(validate_string_pattern_4e09c821aed9e752_len_1_100),
                to_dsl: None,
            })),
                validate: legacy_validator(validate_list_items_1_3),
                to_dsl: None,
            }).required().with_provider_name("FailureValues"),
                    StructField::new("name", AttributeType::Custom {
                identity: None,
                pattern: Some(".*\\S.*".to_string()),
                length: Some((Some(1), Some(200))),
                base: Box::new(AttributeType::String),
                validate: legacy_validator(validate_string_pattern_4e09c821aed9e752_len_1_200),
                to_dsl: None,
            }).required().with_provider_name("Name"),
                    StructField::new("success_values", AttributeType::Custom {
                identity: None,
                pattern: None,
                length: None,
                base: Box::new(AttributeType::list(AttributeType::Custom {
                identity: None,
                pattern: Some(".*\\S.*".to_string()),
                length: Some((Some(1), Some(100))),
                base: Box::new(AttributeType::String),
                validate: legacy_validator(validate_string_pattern_4e09c821aed9e752_len_1_100),
                to_dsl: None,
            })),
                validate: legacy_validator(validate_list_items_1_3),
                to_dsl: None,
            }).required().with_provider_name("SuccessValues")
                    ],
                })
        .with_def("ResponseInspectionJson", AttributeType::Struct {
                    name: "ResponseInspectionJson".to_string(),
                    fields: vec![
                    StructField::new("failure_values", AttributeType::Custom {
                identity: None,
                pattern: None,
                length: None,
                base: Box::new(AttributeType::list(AttributeType::Custom {
                identity: None,
                pattern: Some(".*\\S.*".to_string()),
                length: Some((Some(1), Some(100))),
                base: Box::new(AttributeType::String),
                validate: legacy_validator(validate_string_pattern_4e09c821aed9e752_len_1_100),
                to_dsl: None,
            })),
                validate: legacy_validator(validate_list_items_1_5),
                to_dsl: None,
            }).required().with_provider_name("FailureValues"),
                    StructField::new("identifier", AttributeType::Custom {
                identity: None,
                pattern: Some(".*\\S.*".to_string()),
                length: Some((Some(1), Some(512))),
                base: Box::new(AttributeType::String),
                validate: legacy_validator(validate_string_pattern_4e09c821aed9e752_len_1_512),
                to_dsl: None,
            }).required().with_provider_name("Identifier"),
                    StructField::new("success_values", AttributeType::Custom {
                identity: None,
                pattern: None,
                length: None,
                base: Box::new(AttributeType::list(AttributeType::Custom {
                identity: None,
                pattern: Some(".*\\S.*".to_string()),
                length: Some((Some(1), Some(100))),
                base: Box::new(AttributeType::String),
                validate: legacy_validator(validate_string_pattern_4e09c821aed9e752_len_1_100),
                to_dsl: None,
            })),
                validate: legacy_validator(validate_list_items_1_5),
                to_dsl: None,
            }).required().with_provider_name("SuccessValues")
                    ],
                })
        .with_def("ResponseInspectionStatusCode", AttributeType::Struct {
                    name: "ResponseInspectionStatusCode".to_string(),
                    fields: vec![
                    StructField::new("failure_codes", AttributeType::Custom {
                identity: None,
                pattern: None,
                length: None,
                base: Box::new(AttributeType::list(AttributeType::Int)),
                validate: legacy_validator(validate_list_items_1_10),
                to_dsl: None,
            }).required().with_provider_name("FailureCodes"),
                    StructField::new("success_codes", AttributeType::Custom {
                identity: None,
                pattern: None,
                length: None,
                base: Box::new(AttributeType::list(AttributeType::Int)),
                validate: legacy_validator(validate_list_items_1_10),
                to_dsl: None,
            }).required().with_provider_name("SuccessCodes")
                    ],
                })
        .with_def("RuleAction", AttributeType::Struct {
                    name: "RuleAction".to_string(),
                    fields: vec![
                    StructField::new("allow", AttributeType::Ref("AllowAction".to_string())).with_provider_name("Allow"),
                    StructField::new("block", AttributeType::Ref("BlockAction".to_string())).with_provider_name("Block"),
                    StructField::new("captcha", AttributeType::Struct {
                    name: "CaptchaAction".to_string(),
                    fields: vec![
                    StructField::new("custom_request_handling", AttributeType::Ref("CustomRequestHandling".to_string())).with_provider_name("CustomRequestHandling")
                    ],
                }).with_provider_name("Captcha"),
                    StructField::new("challenge", AttributeType::Struct {
                    name: "ChallengeAction".to_string(),
                    fields: vec![
                    StructField::new("custom_request_handling", AttributeType::Ref("CustomRequestHandling".to_string())).with_provider_name("CustomRequestHandling")
                    ],
                }).with_provider_name("Challenge"),
                    StructField::new("count", AttributeType::Struct {
                    name: "CountAction".to_string(),
                    fields: vec![
                    StructField::new("custom_request_handling", AttributeType::Ref("CustomRequestHandling".to_string())).with_provider_name("CustomRequestHandling")
                    ],
                }).with_provider_name("Count")
                    ],
                })
        .with_def("RuleGroupReferenceStatement", AttributeType::Struct {
                    name: "RuleGroupReferenceStatement".to_string(),
                    fields: vec![
                    StructField::new("arn", super::arn()).required().with_provider_name("Arn"),
                    StructField::new("excluded_rules", AttributeType::list(AttributeType::Struct {
                    name: "ExcludedRule".to_string(),
                    fields: vec![
                    StructField::new("name", AttributeType::Custom {
                identity: None,
                pattern: Some("^[0-9A-Za-z_-]{1,128}$".to_string()),
                length: None,
                base: Box::new(AttributeType::String),
                validate: legacy_validator(validate_string_pattern_42f7eceb887966ad),
                to_dsl: None,
            }).required().with_provider_name("Name")
                    ],
                })).with_provider_name("ExcludedRules").with_block_name("excluded_rule"),
                    StructField::new("rule_action_overrides", AttributeType::Custom {
                identity: None,
                pattern: None,
                length: None,
                base: Box::new(AttributeType::list(AttributeType::Struct {
                    name: "RuleActionOverride".to_string(),
                    fields: vec![
                    StructField::new("action_to_use", AttributeType::Ref("RuleAction".to_string())).required().with_provider_name("ActionToUse"),
                    StructField::new("name", AttributeType::Custom {
                identity: None,
                pattern: Some("^[0-9A-Za-z_-]{1,128}$".to_string()),
                length: None,
                base: Box::new(AttributeType::String),
                validate: legacy_validator(validate_string_pattern_42f7eceb887966ad),
                to_dsl: None,
            }).required().with_provider_name("Name")
                    ],
                })),
                validate: legacy_validator(validate_list_items_max_100),
                to_dsl: None,
            }).with_description("Action overrides for rules in the rule group.").with_provider_name("RuleActionOverrides").with_block_name("rule_action_override")
                    ],
                })
        .with_def("SizeConstraintStatement", AttributeType::Struct {
                    name: "SizeConstraintStatement".to_string(),
                    fields: vec![
                    StructField::new("comparison_operator", AttributeType::StringEnum {
                name: "ComparisonOperator".to_string(),
                values: vec!["EQ".to_string(), "NE".to_string(), "LE".to_string(), "LT".to_string(), "GE".to_string(), "GT".to_string()],
                identity: Some(carina_core::schema::string_enum_identity("ComparisonOperator", Some("awscc.wafv2.WebAcl"))),
                dsl_aliases: vec![("EQ".to_string(), "eq".to_string()), ("NE".to_string(), "ne".to_string()), ("LE".to_string(), "le".to_string()), ("LT".to_string(), "lt".to_string()), ("GE".to_string(), "ge".to_string()), ("GT".to_string(), "gt".to_string())],
            }).required().with_provider_name("ComparisonOperator"),
                    StructField::new("field_to_match", AttributeType::Ref("FieldToMatch".to_string())).required().with_provider_name("FieldToMatch"),
                    StructField::new("size", AttributeType::Custom {
                identity: None,
                pattern: None,
                length: None,
                base: Box::new(AttributeType::Float),
                validate: legacy_validator(validate_size_range),
                to_dsl: None,
            }).required().with_provider_name("Size"),
                    StructField::new("text_transformations", AttributeType::list(AttributeType::Struct {
                    name: "TextTransformation".to_string(),
                    fields: vec![
                    StructField::new("priority", AttributeType::String).required().with_provider_name("Priority"),
                    StructField::new("type", AttributeType::StringEnum {
                name: "TextTransformationType".to_string(),
                values: vec!["NONE".to_string(), "COMPRESS_WHITE_SPACE".to_string(), "HTML_ENTITY_DECODE".to_string(), "LOWERCASE".to_string(), "CMD_LINE".to_string(), "URL_DECODE".to_string(), "BASE64_DECODE".to_string(), "HEX_DECODE".to_string(), "MD5".to_string(), "REPLACE_COMMENTS".to_string(), "ESCAPE_SEQ_DECODE".to_string(), "SQL_HEX_DECODE".to_string(), "CSS_DECODE".to_string(), "JS_DECODE".to_string(), "NORMALIZE_PATH".to_string(), "NORMALIZE_PATH_WIN".to_string(), "REMOVE_NULLS".to_string(), "REPLACE_NULLS".to_string(), "BASE64_DECODE_EXT".to_string(), "URL_DECODE_UNI".to_string(), "UTF8_TO_UNICODE".to_string()],
                identity: Some(carina_core::schema::string_enum_identity("TextTransformationType", Some("awscc.wafv2.WebAcl"))),
                dsl_aliases: vec![("NONE".to_string(), "none".to_string()), ("COMPRESS_WHITE_SPACE".to_string(), "compress_white_space".to_string()), ("HTML_ENTITY_DECODE".to_string(), "html_entity_decode".to_string()), ("LOWERCASE".to_string(), "lowercase".to_string()), ("CMD_LINE".to_string(), "cmd_line".to_string()), ("URL_DECODE".to_string(), "url_decode".to_string()), ("BASE64_DECODE".to_string(), "base64_decode".to_string()), ("HEX_DECODE".to_string(), "hex_decode".to_string()), ("MD5".to_string(), "md5".to_string()), ("REPLACE_COMMENTS".to_string(), "replace_comments".to_string()), ("ESCAPE_SEQ_DECODE".to_string(), "escape_seq_decode".to_string()), ("SQL_HEX_DECODE".to_string(), "sql_hex_decode".to_string()), ("CSS_DECODE".to_string(), "css_decode".to_string()), ("JS_DECODE".to_string(), "js_decode".to_string()), ("NORMALIZE_PATH".to_string(), "normalize_path".to_string()), ("NORMALIZE_PATH_WIN".to_string(), "normalize_path_win".to_string()), ("REMOVE_NULLS".to_string(), "remove_nulls".to_string()), ("REPLACE_NULLS".to_string(), "replace_nulls".to_string()), ("BASE64_DECODE_EXT".to_string(), "base64_decode_ext".to_string()), ("URL_DECODE_UNI".to_string(), "url_decode_uni".to_string()), ("UTF8_TO_UNICODE".to_string(), "utf8_to_unicode".to_string())],
            }).required().with_provider_name("Type")
                    ],
                })).required().with_provider_name("TextTransformations").with_block_name("text_transformation")
                    ],
                })
        .with_def("SqliMatchStatement", AttributeType::Struct {
                    name: "SqliMatchStatement".to_string(),
                    fields: vec![
                    StructField::new("field_to_match", AttributeType::Ref("FieldToMatch".to_string())).required().with_provider_name("FieldToMatch"),
                    StructField::new("sensitivity_level", AttributeType::StringEnum {
                name: "SensitivityLevel".to_string(),
                values: vec!["LOW".to_string(), "HIGH".to_string()],
                identity: Some(carina_core::schema::string_enum_identity("SensitivityLevel", Some("awscc.wafv2.WebAcl"))),
                dsl_aliases: vec![("LOW".to_string(), "low".to_string()), ("HIGH".to_string(), "high".to_string())],
            }).with_provider_name("SensitivityLevel"),
                    StructField::new("text_transformations", AttributeType::list(AttributeType::Struct {
                    name: "TextTransformation".to_string(),
                    fields: vec![
                    StructField::new("priority", AttributeType::String).required().with_provider_name("Priority"),
                    StructField::new("type", AttributeType::StringEnum {
                name: "TextTransformationType".to_string(),
                values: vec!["NONE".to_string(), "COMPRESS_WHITE_SPACE".to_string(), "HTML_ENTITY_DECODE".to_string(), "LOWERCASE".to_string(), "CMD_LINE".to_string(), "URL_DECODE".to_string(), "BASE64_DECODE".to_string(), "HEX_DECODE".to_string(), "MD5".to_string(), "REPLACE_COMMENTS".to_string(), "ESCAPE_SEQ_DECODE".to_string(), "SQL_HEX_DECODE".to_string(), "CSS_DECODE".to_string(), "JS_DECODE".to_string(), "NORMALIZE_PATH".to_string(), "NORMALIZE_PATH_WIN".to_string(), "REMOVE_NULLS".to_string(), "REPLACE_NULLS".to_string(), "BASE64_DECODE_EXT".to_string(), "URL_DECODE_UNI".to_string(), "UTF8_TO_UNICODE".to_string()],
                identity: Some(carina_core::schema::string_enum_identity("TextTransformationType", Some("awscc.wafv2.WebAcl"))),
                dsl_aliases: vec![("NONE".to_string(), "none".to_string()), ("COMPRESS_WHITE_SPACE".to_string(), "compress_white_space".to_string()), ("HTML_ENTITY_DECODE".to_string(), "html_entity_decode".to_string()), ("LOWERCASE".to_string(), "lowercase".to_string()), ("CMD_LINE".to_string(), "cmd_line".to_string()), ("URL_DECODE".to_string(), "url_decode".to_string()), ("BASE64_DECODE".to_string(), "base64_decode".to_string()), ("HEX_DECODE".to_string(), "hex_decode".to_string()), ("MD5".to_string(), "md5".to_string()), ("REPLACE_COMMENTS".to_string(), "replace_comments".to_string()), ("ESCAPE_SEQ_DECODE".to_string(), "escape_seq_decode".to_string()), ("SQL_HEX_DECODE".to_string(), "sql_hex_decode".to_string()), ("CSS_DECODE".to_string(), "css_decode".to_string()), ("JS_DECODE".to_string(), "js_decode".to_string()), ("NORMALIZE_PATH".to_string(), "normalize_path".to_string()), ("NORMALIZE_PATH_WIN".to_string(), "normalize_path_win".to_string()), ("REMOVE_NULLS".to_string(), "remove_nulls".to_string()), ("REPLACE_NULLS".to_string(), "replace_nulls".to_string()), ("BASE64_DECODE_EXT".to_string(), "base64_decode_ext".to_string()), ("URL_DECODE_UNI".to_string(), "url_decode_uni".to_string()), ("UTF8_TO_UNICODE".to_string(), "utf8_to_unicode".to_string())],
            }).required().with_provider_name("Type")
                    ],
                })).required().with_provider_name("TextTransformations").with_block_name("text_transformation")
                    ],
                })
        .with_def("Statement", AttributeType::Struct {
                    name: "Statement".to_string(),
                    fields: vec![
                    StructField::new("and_statement", AttributeType::Struct {
                    name: "AndStatement".to_string(),
                    fields: vec![
                    StructField::new("statements", AttributeType::list(AttributeType::Struct {
                    name: "Statement".to_string(),
                    fields: vec![
                    StructField::new("and_statement", AttributeType::Ref("AndStatement".to_string())).with_provider_name("AndStatement"),
                    StructField::new("asn_match_statement", AttributeType::Struct {
                    name: "AsnMatchStatement".to_string(),
                    fields: vec![
                    StructField::new("asn_list", AttributeType::list(AttributeType::Custom {
                identity: None,
                pattern: None,
                length: None,
                base: Box::new(AttributeType::Int),
                validate: legacy_validator(validate_asn_list_range),
                to_dsl: None,
            })).with_provider_name("AsnList"),
                    StructField::new("forwarded_ip_config", AttributeType::Struct {
                    name: "ForwardedIPConfiguration".to_string(),
                    fields: vec![
                    StructField::new("fallback_behavior", AttributeType::StringEnum {
                name: "FallbackBehavior".to_string(),
                values: vec!["MATCH".to_string(), "NO_MATCH".to_string()],
                identity: Some(carina_core::schema::string_enum_identity("FallbackBehavior", Some("awscc.wafv2.WebAcl"))),
                dsl_aliases: vec![("MATCH".to_string(), "match".to_string()), ("NO_MATCH".to_string(), "no_match".to_string())],
            }).required().with_provider_name("FallbackBehavior"),
                    StructField::new("header_name", AttributeType::Custom {
                identity: None,
                pattern: Some("^[a-zA-Z0-9-]+{1,255}$".to_string()),
                length: None,
                base: Box::new(AttributeType::String),
                validate: legacy_validator(validate_string_pattern_2d93cc844f6d4014),
                to_dsl: None,
            }).required().with_provider_name("HeaderName")
                    ],
                }).with_provider_name("ForwardedIPConfig")
                    ],
                }).with_provider_name("AsnMatchStatement"),
                    StructField::new("byte_match_statement", AttributeType::Struct {
                    name: "ByteMatchStatement".to_string(),
                    fields: vec![
                    StructField::new("field_to_match", AttributeType::Struct {
                    name: "FieldToMatch".to_string(),
                    fields: vec![
                    StructField::new("all_query_arguments", AttributeType::map(AttributeType::String)).with_description("All query arguments of a web request.").with_provider_name("AllQueryArguments"),
                    StructField::new("body", AttributeType::Struct {
                    name: "Body".to_string(),
                    fields: vec![
                    StructField::new("oversize_handling", AttributeType::StringEnum {
                name: "OversizeHandling".to_string(),
                values: vec!["CONTINUE".to_string(), "MATCH".to_string(), "NO_MATCH".to_string()],
                identity: Some(carina_core::schema::string_enum_identity("OversizeHandling", Some("awscc.wafv2.WebAcl"))),
                dsl_aliases: vec![("CONTINUE".to_string(), "continue".to_string()), ("MATCH".to_string(), "match".to_string()), ("NO_MATCH".to_string(), "no_match".to_string())],
            }).with_provider_name("OversizeHandling")
                    ],
                }).with_provider_name("Body"),
                    StructField::new("cookies", AttributeType::Struct {
                    name: "Cookies".to_string(),
                    fields: vec![
                    StructField::new("match_pattern", AttributeType::Struct {
                    name: "CookieMatchPattern".to_string(),
                    fields: vec![
                    StructField::new("all", AttributeType::map(AttributeType::String)).with_description("Inspect all parts of the web request cookies.").with_provider_name("All"),
                    StructField::new("excluded_cookies", AttributeType::Custom {
                identity: None,
                pattern: None,
                length: None,
                base: Box::new(AttributeType::list(AttributeType::Custom {
                identity: None,
                pattern: Some(".*\\S.*".to_string()),
                length: Some((Some(1), Some(60))),
                base: Box::new(AttributeType::String),
                validate: legacy_validator(validate_string_pattern_4e09c821aed9e752_len_1_60),
                to_dsl: None,
            })),
                validate: legacy_validator(validate_list_items_1_199),
                to_dsl: None,
            }).with_provider_name("ExcludedCookies"),
                    StructField::new("included_cookies", AttributeType::Custom {
                identity: None,
                pattern: None,
                length: None,
                base: Box::new(AttributeType::list(AttributeType::Custom {
                identity: None,
                pattern: Some(".*\\S.*".to_string()),
                length: Some((Some(1), Some(60))),
                base: Box::new(AttributeType::String),
                validate: legacy_validator(validate_string_pattern_4e09c821aed9e752_len_1_60),
                to_dsl: None,
            })),
                validate: legacy_validator(validate_list_items_1_199),
                to_dsl: None,
            }).with_provider_name("IncludedCookies")
                    ],
                }).required().with_provider_name("MatchPattern"),
                    StructField::new("match_scope", AttributeType::StringEnum {
                name: "MapMatchScope".to_string(),
                values: vec!["ALL".to_string(), "KEY".to_string(), "VALUE".to_string()],
                identity: Some(carina_core::schema::string_enum_identity("MapMatchScope", Some("awscc.wafv2.WebAcl"))),
                dsl_aliases: vec![("ALL".to_string(), "all".to_string()), ("KEY".to_string(), "key".to_string()), ("VALUE".to_string(), "value".to_string())],
            }).required().with_provider_name("MatchScope"),
                    StructField::new("oversize_handling", AttributeType::StringEnum {
                name: "OversizeHandling".to_string(),
                values: vec!["CONTINUE".to_string(), "MATCH".to_string(), "NO_MATCH".to_string()],
                identity: Some(carina_core::schema::string_enum_identity("OversizeHandling", Some("awscc.wafv2.WebAcl"))),
                dsl_aliases: vec![("CONTINUE".to_string(), "continue".to_string()), ("MATCH".to_string(), "match".to_string()), ("NO_MATCH".to_string(), "no_match".to_string())],
            }).required().with_provider_name("OversizeHandling")
                    ],
                }).with_provider_name("Cookies"),
                    StructField::new("header_order", AttributeType::Struct {
                    name: "HeaderOrder".to_string(),
                    fields: vec![
                    StructField::new("oversize_handling", AttributeType::StringEnum {
                name: "OversizeHandling".to_string(),
                values: vec!["CONTINUE".to_string(), "MATCH".to_string(), "NO_MATCH".to_string()],
                identity: Some(carina_core::schema::string_enum_identity("OversizeHandling", Some("awscc.wafv2.WebAcl"))),
                dsl_aliases: vec![("CONTINUE".to_string(), "continue".to_string()), ("MATCH".to_string(), "match".to_string()), ("NO_MATCH".to_string(), "no_match".to_string())],
            }).required().with_provider_name("OversizeHandling")
                    ],
                }).with_provider_name("HeaderOrder"),
                    StructField::new("headers", AttributeType::Struct {
                    name: "Headers".to_string(),
                    fields: vec![
                    StructField::new("match_pattern", AttributeType::Struct {
                    name: "HeaderMatchPattern".to_string(),
                    fields: vec![
                    StructField::new("all", AttributeType::map(AttributeType::String)).with_description("Inspect all parts of the web request headers.").with_provider_name("All"),
                    StructField::new("excluded_headers", AttributeType::Custom {
                identity: None,
                pattern: None,
                length: None,
                base: Box::new(AttributeType::list(AttributeType::Custom {
                identity: None,
                pattern: Some(".*\\S.*".to_string()),
                length: Some((Some(1), Some(64))),
                base: Box::new(AttributeType::String),
                validate: legacy_validator(validate_string_pattern_4e09c821aed9e752_len_1_64),
                to_dsl: None,
            })),
                validate: legacy_validator(validate_list_items_1_199),
                to_dsl: None,
            }).with_provider_name("ExcludedHeaders"),
                    StructField::new("included_headers", AttributeType::Custom {
                identity: None,
                pattern: None,
                length: None,
                base: Box::new(AttributeType::list(AttributeType::Custom {
                identity: None,
                pattern: Some(".*\\S.*".to_string()),
                length: Some((Some(1), Some(64))),
                base: Box::new(AttributeType::String),
                validate: legacy_validator(validate_string_pattern_4e09c821aed9e752_len_1_64),
                to_dsl: None,
            })),
                validate: legacy_validator(validate_list_items_1_199),
                to_dsl: None,
            }).with_provider_name("IncludedHeaders")
                    ],
                }).required().with_provider_name("MatchPattern"),
                    StructField::new("match_scope", AttributeType::StringEnum {
                name: "MapMatchScope".to_string(),
                values: vec!["ALL".to_string(), "KEY".to_string(), "VALUE".to_string()],
                identity: Some(carina_core::schema::string_enum_identity("MapMatchScope", Some("awscc.wafv2.WebAcl"))),
                dsl_aliases: vec![("ALL".to_string(), "all".to_string()), ("KEY".to_string(), "key".to_string()), ("VALUE".to_string(), "value".to_string())],
            }).required().with_provider_name("MatchScope"),
                    StructField::new("oversize_handling", AttributeType::StringEnum {
                name: "OversizeHandling".to_string(),
                values: vec!["CONTINUE".to_string(), "MATCH".to_string(), "NO_MATCH".to_string()],
                identity: Some(carina_core::schema::string_enum_identity("OversizeHandling", Some("awscc.wafv2.WebAcl"))),
                dsl_aliases: vec![("CONTINUE".to_string(), "continue".to_string()), ("MATCH".to_string(), "match".to_string()), ("NO_MATCH".to_string(), "no_match".to_string())],
            }).required().with_provider_name("OversizeHandling")
                    ],
                }).with_provider_name("Headers"),
                    StructField::new("ja3_fingerprint", AttributeType::Struct {
                    name: "JA3Fingerprint".to_string(),
                    fields: vec![
                    StructField::new("fallback_behavior", AttributeType::StringEnum {
                name: "FallbackBehavior".to_string(),
                values: vec!["MATCH".to_string(), "NO_MATCH".to_string()],
                identity: Some(carina_core::schema::string_enum_identity("FallbackBehavior", Some("awscc.wafv2.WebAcl"))),
                dsl_aliases: vec![("MATCH".to_string(), "match".to_string()), ("NO_MATCH".to_string(), "no_match".to_string())],
            }).required().with_provider_name("FallbackBehavior")
                    ],
                }).with_provider_name("JA3Fingerprint"),
                    StructField::new("ja4_fingerprint", AttributeType::Struct {
                    name: "JA4Fingerprint".to_string(),
                    fields: vec![
                    StructField::new("fallback_behavior", AttributeType::StringEnum {
                name: "FallbackBehavior".to_string(),
                values: vec!["MATCH".to_string(), "NO_MATCH".to_string()],
                identity: Some(carina_core::schema::string_enum_identity("FallbackBehavior", Some("awscc.wafv2.WebAcl"))),
                dsl_aliases: vec![("MATCH".to_string(), "match".to_string()), ("NO_MATCH".to_string(), "no_match".to_string())],
            }).required().with_provider_name("FallbackBehavior")
                    ],
                }).with_provider_name("JA4Fingerprint"),
                    StructField::new("json_body", AttributeType::Struct {
                    name: "JsonBody".to_string(),
                    fields: vec![
                    StructField::new("invalid_fallback_behavior", AttributeType::StringEnum {
                name: "BodyParsingFallbackBehavior".to_string(),
                values: vec!["MATCH".to_string(), "NO_MATCH".to_string(), "EVALUATE_AS_STRING".to_string()],
                identity: Some(carina_core::schema::string_enum_identity("BodyParsingFallbackBehavior", Some("awscc.wafv2.WebAcl"))),
                dsl_aliases: vec![("MATCH".to_string(), "match".to_string()), ("NO_MATCH".to_string(), "no_match".to_string()), ("EVALUATE_AS_STRING".to_string(), "evaluate_as_string".to_string())],
            }).with_provider_name("InvalidFallbackBehavior"),
                    StructField::new("match_pattern", AttributeType::Struct {
                    name: "JsonMatchPattern".to_string(),
                    fields: vec![
                    StructField::new("all", AttributeType::map(AttributeType::String)).with_description("Inspect all parts of the web request's JSON body.").with_provider_name("All"),
                    StructField::new("included_paths", AttributeType::list(AttributeType::Custom {
                identity: None,
                pattern: Some("^[\\/]+([^~]*(~[01])*)*{1,512}$".to_string()),
                length: None,
                base: Box::new(AttributeType::String),
                validate: legacy_validator(validate_string_pattern_c77cf75cf1a75ade),
                to_dsl: None,
            })).with_provider_name("IncludedPaths")
                    ],
                }).required().with_provider_name("MatchPattern"),
                    StructField::new("match_scope", AttributeType::StringEnum {
                name: "JsonMatchScope".to_string(),
                values: vec!["ALL".to_string(), "KEY".to_string(), "VALUE".to_string()],
                identity: Some(carina_core::schema::string_enum_identity("JsonMatchScope", Some("awscc.wafv2.WebAcl"))),
                dsl_aliases: vec![("ALL".to_string(), "all".to_string()), ("KEY".to_string(), "key".to_string()), ("VALUE".to_string(), "value".to_string())],
            }).required().with_provider_name("MatchScope"),
                    StructField::new("oversize_handling", AttributeType::StringEnum {
                name: "OversizeHandling".to_string(),
                values: vec!["CONTINUE".to_string(), "MATCH".to_string(), "NO_MATCH".to_string()],
                identity: Some(carina_core::schema::string_enum_identity("OversizeHandling", Some("awscc.wafv2.WebAcl"))),
                dsl_aliases: vec![("CONTINUE".to_string(), "continue".to_string()), ("MATCH".to_string(), "match".to_string()), ("NO_MATCH".to_string(), "no_match".to_string())],
            }).with_provider_name("OversizeHandling")
                    ],
                }).with_provider_name("JsonBody"),
                    StructField::new("method", AttributeType::map(AttributeType::String)).with_description("The HTTP method of a web request. The method indicates the type of operation that the request is asking the origin to perform.").with_provider_name("Method"),
                    StructField::new("query_string", AttributeType::map(AttributeType::String)).with_description("The query string of a web request. This is the part of a URL that appears after a ? character, if any.").with_provider_name("QueryString"),
                    StructField::new("single_header", AttributeType::Struct {
                    name: "SingleHeader".to_string(),
                    fields: vec![
                    StructField::new("name", AttributeType::String).required().with_provider_name("Name")
                    ],
                }).with_provider_name("SingleHeader"),
                    StructField::new("single_query_argument", AttributeType::Struct {
                    name: "SingleQueryArgument".to_string(),
                    fields: vec![
                    StructField::new("name", AttributeType::String).required().with_provider_name("Name")
                    ],
                }).with_description("One query argument in a web request, identified by name, for example UserName or SalesRegion. The name can be up to 30 characters long and isn't case sensitive.").with_provider_name("SingleQueryArgument"),
                    StructField::new("uri_fragment", AttributeType::Struct {
                    name: "UriFragment".to_string(),
                    fields: vec![
                    StructField::new("fallback_behavior", AttributeType::StringEnum {
                name: "FallbackBehavior".to_string(),
                values: vec!["MATCH".to_string(), "NO_MATCH".to_string()],
                identity: Some(carina_core::schema::string_enum_identity("FallbackBehavior", Some("awscc.wafv2.WebAcl"))),
                dsl_aliases: vec![("MATCH".to_string(), "match".to_string()), ("NO_MATCH".to_string(), "no_match".to_string())],
            }).with_provider_name("FallbackBehavior")
                    ],
                }).with_provider_name("UriFragment"),
                    StructField::new("uri_path", AttributeType::map(AttributeType::String)).with_description("The path component of the URI of a web request. This is the part of a web request that identifies a resource, for example, /images/daily-ad.jpg.").with_provider_name("UriPath")
                    ],
                }).required().with_provider_name("FieldToMatch"),
                    StructField::new("positional_constraint", AttributeType::StringEnum {
                name: "PositionalConstraint".to_string(),
                values: vec!["EXACTLY".to_string(), "STARTS_WITH".to_string(), "ENDS_WITH".to_string(), "CONTAINS".to_string(), "CONTAINS_WORD".to_string()],
                identity: Some(carina_core::schema::string_enum_identity("PositionalConstraint", Some("awscc.wafv2.WebAcl"))),
                dsl_aliases: vec![("EXACTLY".to_string(), "exactly".to_string()), ("STARTS_WITH".to_string(), "starts_with".to_string()), ("ENDS_WITH".to_string(), "ends_with".to_string()), ("CONTAINS".to_string(), "contains".to_string()), ("CONTAINS_WORD".to_string(), "contains_word".to_string())],
            }).required().with_provider_name("PositionalConstraint"),
                    StructField::new("search_string", AttributeType::String).with_provider_name("SearchString"),
                    StructField::new("search_string_base64", AttributeType::String).with_provider_name("SearchStringBase64"),
                    StructField::new("text_transformations", AttributeType::list(AttributeType::Struct {
                    name: "TextTransformation".to_string(),
                    fields: vec![
                    StructField::new("priority", AttributeType::String).required().with_provider_name("Priority"),
                    StructField::new("type", AttributeType::StringEnum {
                name: "TextTransformationType".to_string(),
                values: vec!["NONE".to_string(), "COMPRESS_WHITE_SPACE".to_string(), "HTML_ENTITY_DECODE".to_string(), "LOWERCASE".to_string(), "CMD_LINE".to_string(), "URL_DECODE".to_string(), "BASE64_DECODE".to_string(), "HEX_DECODE".to_string(), "MD5".to_string(), "REPLACE_COMMENTS".to_string(), "ESCAPE_SEQ_DECODE".to_string(), "SQL_HEX_DECODE".to_string(), "CSS_DECODE".to_string(), "JS_DECODE".to_string(), "NORMALIZE_PATH".to_string(), "NORMALIZE_PATH_WIN".to_string(), "REMOVE_NULLS".to_string(), "REPLACE_NULLS".to_string(), "BASE64_DECODE_EXT".to_string(), "URL_DECODE_UNI".to_string(), "UTF8_TO_UNICODE".to_string()],
                identity: Some(carina_core::schema::string_enum_identity("TextTransformationType", Some("awscc.wafv2.WebAcl"))),
                dsl_aliases: vec![("NONE".to_string(), "none".to_string()), ("COMPRESS_WHITE_SPACE".to_string(), "compress_white_space".to_string()), ("HTML_ENTITY_DECODE".to_string(), "html_entity_decode".to_string()), ("LOWERCASE".to_string(), "lowercase".to_string()), ("CMD_LINE".to_string(), "cmd_line".to_string()), ("URL_DECODE".to_string(), "url_decode".to_string()), ("BASE64_DECODE".to_string(), "base64_decode".to_string()), ("HEX_DECODE".to_string(), "hex_decode".to_string()), ("MD5".to_string(), "md5".to_string()), ("REPLACE_COMMENTS".to_string(), "replace_comments".to_string()), ("ESCAPE_SEQ_DECODE".to_string(), "escape_seq_decode".to_string()), ("SQL_HEX_DECODE".to_string(), "sql_hex_decode".to_string()), ("CSS_DECODE".to_string(), "css_decode".to_string()), ("JS_DECODE".to_string(), "js_decode".to_string()), ("NORMALIZE_PATH".to_string(), "normalize_path".to_string()), ("NORMALIZE_PATH_WIN".to_string(), "normalize_path_win".to_string()), ("REMOVE_NULLS".to_string(), "remove_nulls".to_string()), ("REPLACE_NULLS".to_string(), "replace_nulls".to_string()), ("BASE64_DECODE_EXT".to_string(), "base64_decode_ext".to_string()), ("URL_DECODE_UNI".to_string(), "url_decode_uni".to_string()), ("UTF8_TO_UNICODE".to_string(), "utf8_to_unicode".to_string())],
            }).required().with_provider_name("Type")
                    ],
                })).required().with_provider_name("TextTransformations").with_block_name("text_transformation")
                    ],
                }).with_provider_name("ByteMatchStatement").with_block_name("byte_match_statement"),
                    StructField::new("geo_match_statement", AttributeType::Struct {
                    name: "GeoMatchStatement".to_string(),
                    fields: vec![
                    StructField::new("country_codes", AttributeType::list(AttributeType::Custom {
                identity: None,
                pattern: None,
                length: Some((Some(1), Some(2))),
                base: Box::new(AttributeType::String),
                validate: legacy_validator(validate_string_length_1_2),
                to_dsl: None,
            })).with_provider_name("CountryCodes"),
                    StructField::new("forwarded_ip_config", AttributeType::Ref("ForwardedIPConfiguration".to_string())).with_provider_name("ForwardedIPConfig")
                    ],
                }).with_provider_name("GeoMatchStatement"),
                    StructField::new("ip_set_reference_statement", AttributeType::Struct {
                    name: "IPSetReferenceStatement".to_string(),
                    fields: vec![
                    StructField::new("arn", super::arn()).required().with_provider_name("Arn"),
                    StructField::new("ip_set_forwarded_ip_config", AttributeType::Struct {
                    name: "IPSetForwardedIPConfiguration".to_string(),
                    fields: vec![
                    StructField::new("fallback_behavior", AttributeType::StringEnum {
                name: "FallbackBehavior".to_string(),
                values: vec!["MATCH".to_string(), "NO_MATCH".to_string()],
                identity: Some(carina_core::schema::string_enum_identity("FallbackBehavior", Some("awscc.wafv2.WebAcl"))),
                dsl_aliases: vec![("MATCH".to_string(), "match".to_string()), ("NO_MATCH".to_string(), "no_match".to_string())],
            }).required().with_provider_name("FallbackBehavior"),
                    StructField::new("header_name", AttributeType::Custom {
                identity: None,
                pattern: Some("^[a-zA-Z0-9-]+{1,255}$".to_string()),
                length: None,
                base: Box::new(AttributeType::String),
                validate: legacy_validator(validate_string_pattern_2d93cc844f6d4014),
                to_dsl: None,
            }).required().with_provider_name("HeaderName"),
                    StructField::new("position", AttributeType::StringEnum {
                name: "Position".to_string(),
                values: vec!["FIRST".to_string(), "LAST".to_string(), "ANY".to_string()],
                identity: Some(carina_core::schema::string_enum_identity("Position", Some("awscc.wafv2.WebAcl"))),
                dsl_aliases: vec![("FIRST".to_string(), "first".to_string()), ("LAST".to_string(), "last".to_string()), ("ANY".to_string(), "any".to_string())],
            }).required().with_provider_name("Position")
                    ],
                }).with_provider_name("IPSetForwardedIPConfig")
                    ],
                }).with_provider_name("IPSetReferenceStatement"),
                    StructField::new("label_match_statement", AttributeType::Struct {
                    name: "LabelMatchStatement".to_string(),
                    fields: vec![
                    StructField::new("key", AttributeType::Custom {
                identity: None,
                pattern: Some("^[0-9A-Za-z_:-]{1,1024}$".to_string()),
                length: None,
                base: Box::new(AttributeType::String),
                validate: legacy_validator(validate_string_pattern_b3fc65b549fb77bd),
                to_dsl: None,
            }).required().with_provider_name("Key"),
                    StructField::new("scope", AttributeType::StringEnum {
                name: "LabelMatchScope".to_string(),
                values: vec!["LABEL".to_string(), "NAMESPACE".to_string()],
                identity: Some(carina_core::schema::string_enum_identity("LabelMatchScope", Some("awscc.wafv2.WebAcl"))),
                dsl_aliases: vec![("LABEL".to_string(), "label".to_string()), ("NAMESPACE".to_string(), "namespace".to_string())],
            }).required().with_provider_name("Scope")
                    ],
                }).with_provider_name("LabelMatchStatement"),
                    StructField::new("managed_rule_group_statement", AttributeType::Struct {
                    name: "ManagedRuleGroupStatement".to_string(),
                    fields: vec![
                    StructField::new("excluded_rules", AttributeType::list(AttributeType::Struct {
                    name: "ExcludedRule".to_string(),
                    fields: vec![
                    StructField::new("name", AttributeType::Custom {
                identity: None,
                pattern: Some("^[0-9A-Za-z_-]{1,128}$".to_string()),
                length: None,
                base: Box::new(AttributeType::String),
                validate: legacy_validator(validate_string_pattern_42f7eceb887966ad),
                to_dsl: None,
            }).required().with_provider_name("Name")
                    ],
                })).with_provider_name("ExcludedRules").with_block_name("excluded_rule"),
                    StructField::new("managed_rule_group_configs", AttributeType::list(AttributeType::Struct {
                    name: "ManagedRuleGroupConfig".to_string(),
                    fields: vec![
                    StructField::new("aws_managed_rules_acfp_rule_set", AttributeType::Struct {
                    name: "AWSManagedRulesACFPRuleSet".to_string(),
                    fields: vec![
                    StructField::new("creation_path", AttributeType::String).required().with_provider_name("CreationPath"),
                    StructField::new("enable_regex_in_path", AttributeType::Bool).with_provider_name("EnableRegexInPath"),
                    StructField::new("registration_page_path", AttributeType::String).required().with_provider_name("RegistrationPagePath"),
                    StructField::new("request_inspection", AttributeType::Struct {
                    name: "RequestInspectionACFP".to_string(),
                    fields: vec![
                    StructField::new("address_fields", AttributeType::list(AttributeType::String)).with_provider_name("AddressFields"),
                    StructField::new("email_field", AttributeType::Struct {
                    name: "FieldIdentifier".to_string(),
                    fields: vec![
                    StructField::new("identifier", AttributeType::Custom {
                identity: None,
                pattern: Some(".*\\S.*".to_string()),
                length: Some((Some(1), Some(512))),
                base: Box::new(AttributeType::String),
                validate: legacy_validator(validate_string_pattern_4e09c821aed9e752_len_1_512),
                to_dsl: None,
            }).required().with_provider_name("Identifier")
                    ],
                }).with_provider_name("EmailField"),
                    StructField::new("password_field", AttributeType::Ref("FieldIdentifier".to_string())).with_provider_name("PasswordField"),
                    StructField::new("payload_type", AttributeType::StringEnum {
                name: "PayloadType".to_string(),
                values: vec!["JSON".to_string(), "FORM_ENCODED".to_string()],
                identity: Some(carina_core::schema::string_enum_identity("PayloadType", Some("awscc.wafv2.WebAcl"))),
                dsl_aliases: vec![("JSON".to_string(), "json".to_string()), ("FORM_ENCODED".to_string(), "form_encoded".to_string())],
            }).required().with_provider_name("PayloadType"),
                    StructField::new("phone_number_fields", AttributeType::list(AttributeType::String)).with_provider_name("PhoneNumberFields"),
                    StructField::new("username_field", AttributeType::Ref("FieldIdentifier".to_string())).with_provider_name("UsernameField")
                    ],
                }).required().with_provider_name("RequestInspection"),
                    StructField::new("response_inspection", AttributeType::Struct {
                    name: "ResponseInspection".to_string(),
                    fields: vec![
                    StructField::new("body_contains", AttributeType::Struct {
                    name: "ResponseInspectionBodyContains".to_string(),
                    fields: vec![
                    StructField::new("failure_strings", AttributeType::Custom {
                identity: None,
                pattern: None,
                length: None,
                base: Box::new(AttributeType::list(AttributeType::Custom {
                identity: None,
                pattern: Some(".*\\S.*".to_string()),
                length: Some((Some(1), Some(100))),
                base: Box::new(AttributeType::String),
                validate: legacy_validator(validate_string_pattern_4e09c821aed9e752_len_1_100),
                to_dsl: None,
            })),
                validate: legacy_validator(validate_list_items_1_5),
                to_dsl: None,
            }).required().with_provider_name("FailureStrings"),
                    StructField::new("success_strings", AttributeType::Custom {
                identity: None,
                pattern: None,
                length: None,
                base: Box::new(AttributeType::list(AttributeType::Custom {
                identity: None,
                pattern: Some(".*\\S.*".to_string()),
                length: Some((Some(1), Some(100))),
                base: Box::new(AttributeType::String),
                validate: legacy_validator(validate_string_pattern_4e09c821aed9e752_len_1_100),
                to_dsl: None,
            })),
                validate: legacy_validator(validate_list_items_1_5),
                to_dsl: None,
            }).required().with_provider_name("SuccessStrings")
                    ],
                }).with_provider_name("BodyContains"),
                    StructField::new("header", AttributeType::Struct {
                    name: "ResponseInspectionHeader".to_string(),
                    fields: vec![
                    StructField::new("failure_values", AttributeType::Custom {
                identity: None,
                pattern: None,
                length: None,
                base: Box::new(AttributeType::list(AttributeType::Custom {
                identity: None,
                pattern: Some(".*\\S.*".to_string()),
                length: Some((Some(1), Some(100))),
                base: Box::new(AttributeType::String),
                validate: legacy_validator(validate_string_pattern_4e09c821aed9e752_len_1_100),
                to_dsl: None,
            })),
                validate: legacy_validator(validate_list_items_1_3),
                to_dsl: None,
            }).required().with_provider_name("FailureValues"),
                    StructField::new("name", AttributeType::Custom {
                identity: None,
                pattern: Some(".*\\S.*".to_string()),
                length: Some((Some(1), Some(200))),
                base: Box::new(AttributeType::String),
                validate: legacy_validator(validate_string_pattern_4e09c821aed9e752_len_1_200),
                to_dsl: None,
            }).required().with_provider_name("Name"),
                    StructField::new("success_values", AttributeType::Custom {
                identity: None,
                pattern: None,
                length: None,
                base: Box::new(AttributeType::list(AttributeType::Custom {
                identity: None,
                pattern: Some(".*\\S.*".to_string()),
                length: Some((Some(1), Some(100))),
                base: Box::new(AttributeType::String),
                validate: legacy_validator(validate_string_pattern_4e09c821aed9e752_len_1_100),
                to_dsl: None,
            })),
                validate: legacy_validator(validate_list_items_1_3),
                to_dsl: None,
            }).required().with_provider_name("SuccessValues")
                    ],
                }).with_provider_name("Header"),
                    StructField::new("json", AttributeType::Struct {
                    name: "ResponseInspectionJson".to_string(),
                    fields: vec![
                    StructField::new("failure_values", AttributeType::Custom {
                identity: None,
                pattern: None,
                length: None,
                base: Box::new(AttributeType::list(AttributeType::Custom {
                identity: None,
                pattern: Some(".*\\S.*".to_string()),
                length: Some((Some(1), Some(100))),
                base: Box::new(AttributeType::String),
                validate: legacy_validator(validate_string_pattern_4e09c821aed9e752_len_1_100),
                to_dsl: None,
            })),
                validate: legacy_validator(validate_list_items_1_5),
                to_dsl: None,
            }).required().with_provider_name("FailureValues"),
                    StructField::new("identifier", AttributeType::Custom {
                identity: None,
                pattern: Some(".*\\S.*".to_string()),
                length: Some((Some(1), Some(512))),
                base: Box::new(AttributeType::String),
                validate: legacy_validator(validate_string_pattern_4e09c821aed9e752_len_1_512),
                to_dsl: None,
            }).required().with_provider_name("Identifier"),
                    StructField::new("success_values", AttributeType::Custom {
                identity: None,
                pattern: None,
                length: None,
                base: Box::new(AttributeType::list(AttributeType::Custom {
                identity: None,
                pattern: Some(".*\\S.*".to_string()),
                length: Some((Some(1), Some(100))),
                base: Box::new(AttributeType::String),
                validate: legacy_validator(validate_string_pattern_4e09c821aed9e752_len_1_100),
                to_dsl: None,
            })),
                validate: legacy_validator(validate_list_items_1_5),
                to_dsl: None,
            }).required().with_provider_name("SuccessValues")
                    ],
                }).with_provider_name("Json"),
                    StructField::new("status_code", AttributeType::Struct {
                    name: "ResponseInspectionStatusCode".to_string(),
                    fields: vec![
                    StructField::new("failure_codes", AttributeType::Custom {
                identity: None,
                pattern: None,
                length: None,
                base: Box::new(AttributeType::list(AttributeType::Int)),
                validate: legacy_validator(validate_list_items_1_10),
                to_dsl: None,
            }).required().with_provider_name("FailureCodes"),
                    StructField::new("success_codes", AttributeType::Custom {
                identity: None,
                pattern: None,
                length: None,
                base: Box::new(AttributeType::list(AttributeType::Int)),
                validate: legacy_validator(validate_list_items_1_10),
                to_dsl: None,
            }).required().with_provider_name("SuccessCodes")
                    ],
                }).with_provider_name("StatusCode")
                    ],
                }).with_provider_name("ResponseInspection")
                    ],
                }).with_provider_name("AWSManagedRulesACFPRuleSet"),
                    StructField::new("aws_managed_rules_atp_rule_set", AttributeType::Struct {
                    name: "AWSManagedRulesATPRuleSet".to_string(),
                    fields: vec![
                    StructField::new("enable_regex_in_path", AttributeType::Bool).with_provider_name("EnableRegexInPath"),
                    StructField::new("login_path", AttributeType::String).required().with_provider_name("LoginPath"),
                    StructField::new("request_inspection", AttributeType::Struct {
                    name: "RequestInspection".to_string(),
                    fields: vec![
                    StructField::new("password_field", AttributeType::Ref("FieldIdentifier".to_string())).required().with_provider_name("PasswordField"),
                    StructField::new("payload_type", AttributeType::StringEnum {
                name: "PayloadType".to_string(),
                values: vec!["JSON".to_string(), "FORM_ENCODED".to_string()],
                identity: Some(carina_core::schema::string_enum_identity("PayloadType", Some("awscc.wafv2.WebAcl"))),
                dsl_aliases: vec![("JSON".to_string(), "json".to_string()), ("FORM_ENCODED".to_string(), "form_encoded".to_string())],
            }).required().with_provider_name("PayloadType"),
                    StructField::new("username_field", AttributeType::Ref("FieldIdentifier".to_string())).required().with_provider_name("UsernameField")
                    ],
                }).with_provider_name("RequestInspection"),
                    StructField::new("response_inspection", AttributeType::Ref("ResponseInspection".to_string())).with_provider_name("ResponseInspection")
                    ],
                }).with_provider_name("AWSManagedRulesATPRuleSet"),
                    StructField::new("aws_managed_rules_anti_d_do_s_rule_set", AttributeType::Struct {
                    name: "AWSManagedRulesAntiDDoSRuleSet".to_string(),
                    fields: vec![
                    StructField::new("client_side_action_config", AttributeType::Struct {
                    name: "ClientSideActionConfig".to_string(),
                    fields: vec![
                    StructField::new("challenge", AttributeType::Struct {
                    name: "ClientSideAction".to_string(),
                    fields: vec![
                    StructField::new("exempt_uri_regular_expressions", AttributeType::list(AttributeType::Struct {
                    name: "Regex".to_string(),
                    fields: vec![
                    StructField::new("regex_string", AttributeType::String).with_provider_name("RegexString")
                    ],
                })).with_provider_name("ExemptUriRegularExpressions").with_block_name("exempt_uri_regular_expression"),
                    StructField::new("sensitivity", AttributeType::StringEnum {
                name: "SensitivityToAct".to_string(),
                values: vec!["LOW".to_string(), "MEDIUM".to_string(), "HIGH".to_string()],
                identity: Some(carina_core::schema::string_enum_identity("SensitivityToAct", Some("awscc.wafv2.WebAcl"))),
                dsl_aliases: vec![("LOW".to_string(), "low".to_string()), ("MEDIUM".to_string(), "medium".to_string()), ("HIGH".to_string(), "high".to_string())],
            }).with_provider_name("Sensitivity"),
                    StructField::new("usage_of_action", AttributeType::StringEnum {
                name: "UsageOfAction".to_string(),
                values: vec!["ENABLED".to_string(), "DISABLED".to_string()],
                identity: Some(carina_core::schema::string_enum_identity("UsageOfAction", Some("awscc.wafv2.WebAcl"))),
                dsl_aliases: vec![("ENABLED".to_string(), "enabled".to_string()), ("DISABLED".to_string(), "disabled".to_string())],
            }).required().with_provider_name("UsageOfAction")
                    ],
                }).required().with_provider_name("Challenge").with_block_name("challenge")
                    ],
                }).required().with_provider_name("ClientSideActionConfig").with_block_name("client_side_action_config"),
                    StructField::new("sensitivity_to_block", AttributeType::StringEnum {
                name: "SensitivityToAct".to_string(),
                values: vec!["LOW".to_string(), "MEDIUM".to_string(), "HIGH".to_string()],
                identity: Some(carina_core::schema::string_enum_identity("SensitivityToAct", Some("awscc.wafv2.WebAcl"))),
                dsl_aliases: vec![("LOW".to_string(), "low".to_string()), ("MEDIUM".to_string(), "medium".to_string()), ("HIGH".to_string(), "high".to_string())],
            }).with_provider_name("SensitivityToBlock")
                    ],
                }).with_provider_name("AWSManagedRulesAntiDDoSRuleSet").with_block_name("aws_managed_rules_anti_d_do_s_rule_set"),
                    StructField::new("aws_managed_rules_bot_control_rule_set", AttributeType::Struct {
                    name: "AWSManagedRulesBotControlRuleSet".to_string(),
                    fields: vec![
                    StructField::new("enable_machine_learning", AttributeType::Bool).with_provider_name("EnableMachineLearning"),
                    StructField::new("inspection_level", AttributeType::StringEnum {
                name: "InspectionLevel".to_string(),
                values: vec!["COMMON".to_string(), "TARGETED".to_string()],
                identity: Some(carina_core::schema::string_enum_identity("InspectionLevel", Some("awscc.wafv2.WebAcl"))),
                dsl_aliases: vec![("COMMON".to_string(), "common".to_string()), ("TARGETED".to_string(), "targeted".to_string())],
            }).required().with_provider_name("InspectionLevel")
                    ],
                }).with_provider_name("AWSManagedRulesBotControlRuleSet"),
                    StructField::new("login_path", AttributeType::Custom {
                identity: None,
                pattern: Some(".*\\S.*".to_string()),
                length: Some((Some(1), Some(256))),
                base: Box::new(AttributeType::String),
                validate: legacy_validator(validate_string_pattern_4e09c821aed9e752_len_1_256),
                to_dsl: None,
            }).with_provider_name("LoginPath"),
                    StructField::new("password_field", AttributeType::Ref("FieldIdentifier".to_string())).with_provider_name("PasswordField"),
                    StructField::new("payload_type", AttributeType::StringEnum {
                name: "PayloadType".to_string(),
                values: vec!["JSON".to_string(), "FORM_ENCODED".to_string()],
                identity: Some(carina_core::schema::string_enum_identity("PayloadType", Some("awscc.wafv2.WebAcl"))),
                dsl_aliases: vec![("JSON".to_string(), "json".to_string()), ("FORM_ENCODED".to_string(), "form_encoded".to_string())],
            }).with_provider_name("PayloadType"),
                    StructField::new("username_field", AttributeType::Ref("FieldIdentifier".to_string())).with_provider_name("UsernameField")
                    ],
                })).with_description("Collection of ManagedRuleGroupConfig.").with_provider_name("ManagedRuleGroupConfigs").with_block_name("managed_rule_group_config"),
                    StructField::new("name", AttributeType::Custom {
                identity: None,
                pattern: Some("^[0-9A-Za-z_-]{1,128}$".to_string()),
                length: None,
                base: Box::new(AttributeType::String),
                validate: legacy_validator(validate_string_pattern_42f7eceb887966ad),
                to_dsl: None,
            }).required().with_provider_name("Name"),
                    StructField::new("rule_action_overrides", AttributeType::Custom {
                identity: None,
                pattern: None,
                length: None,
                base: Box::new(AttributeType::list(AttributeType::Struct {
                    name: "RuleActionOverride".to_string(),
                    fields: vec![
                    StructField::new("action_to_use", AttributeType::Ref("RuleAction".to_string())).required().with_provider_name("ActionToUse"),
                    StructField::new("name", AttributeType::Custom {
                identity: None,
                pattern: Some("^[0-9A-Za-z_-]{1,128}$".to_string()),
                length: None,
                base: Box::new(AttributeType::String),
                validate: legacy_validator(validate_string_pattern_42f7eceb887966ad),
                to_dsl: None,
            }).required().with_provider_name("Name")
                    ],
                })),
                validate: legacy_validator(validate_list_items_max_100),
                to_dsl: None,
            }).with_description("Action overrides for rules in the rule group.").with_provider_name("RuleActionOverrides").with_block_name("rule_action_override"),
                    StructField::new("scope_down_statement", AttributeType::Ref("Statement".to_string())).with_provider_name("ScopeDownStatement"),
                    StructField::new("vendor_name", AttributeType::String).required().with_provider_name("VendorName"),
                    StructField::new("version", AttributeType::Custom {
                identity: None,
                pattern: Some("^[\\w#:\\.\\-/]+$".to_string()),
                length: Some((Some(1), Some(64))),
                base: Box::new(AttributeType::String),
                validate: legacy_validator(validate_string_pattern_82a037d29be8d222_len_1_64),
                to_dsl: None,
            }).with_provider_name("Version")
                    ],
                }).with_provider_name("ManagedRuleGroupStatement").with_block_name("managed_rule_group_statement"),
                    StructField::new("not_statement", AttributeType::Struct {
                    name: "NotStatement".to_string(),
                    fields: vec![
                    StructField::new("statement", AttributeType::Ref("Statement".to_string())).required().with_provider_name("Statement")
                    ],
                }).with_provider_name("NotStatement"),
                    StructField::new("or_statement", AttributeType::Struct {
                    name: "OrStatement".to_string(),
                    fields: vec![
                    StructField::new("statements", AttributeType::list(AttributeType::Struct {
                    name: "Statement".to_string(),
                    fields: vec![
                    StructField::new("and_statement", AttributeType::Ref("AndStatement".to_string())).with_provider_name("AndStatement"),
                    StructField::new("asn_match_statement", AttributeType::Ref("AsnMatchStatement".to_string())).with_provider_name("AsnMatchStatement"),
                    StructField::new("byte_match_statement", AttributeType::Ref("ByteMatchStatement".to_string())).with_provider_name("ByteMatchStatement"),
                    StructField::new("geo_match_statement", AttributeType::Ref("GeoMatchStatement".to_string())).with_provider_name("GeoMatchStatement"),
                    StructField::new("ip_set_reference_statement", AttributeType::Ref("IPSetReferenceStatement".to_string())).with_provider_name("IPSetReferenceStatement"),
                    StructField::new("label_match_statement", AttributeType::Ref("LabelMatchStatement".to_string())).with_provider_name("LabelMatchStatement"),
                    StructField::new("managed_rule_group_statement", AttributeType::Ref("ManagedRuleGroupStatement".to_string())).with_provider_name("ManagedRuleGroupStatement"),
                    StructField::new("not_statement", AttributeType::Ref("NotStatement".to_string())).with_provider_name("NotStatement"),
                    StructField::new("or_statement", AttributeType::Ref("OrStatement".to_string())).with_provider_name("OrStatement"),
                    StructField::new("rate_based_statement", AttributeType::Struct {
                    name: "RateBasedStatement".to_string(),
                    fields: vec![
                    StructField::new("aggregate_key_type", AttributeType::StringEnum {
                name: "AggregateKeyType".to_string(),
                values: vec!["CONSTANT".to_string(), "IP".to_string(), "FORWARDED_IP".to_string(), "CUSTOM_KEYS".to_string()],
                identity: Some(carina_core::schema::string_enum_identity("AggregateKeyType", Some("awscc.wafv2.WebAcl"))),
                dsl_aliases: vec![("CONSTANT".to_string(), "constant".to_string()), ("IP".to_string(), "ip".to_string()), ("FORWARDED_IP".to_string(), "forwarded_ip".to_string()), ("CUSTOM_KEYS".to_string(), "custom_keys".to_string())],
            }).required().with_provider_name("AggregateKeyType"),
                    StructField::new("custom_keys", AttributeType::Custom {
                identity: None,
                pattern: None,
                length: None,
                base: Box::new(AttributeType::list(AttributeType::Struct {
                    name: "RateBasedStatementCustomKey".to_string(),
                    fields: vec![
                    StructField::new("asn", AttributeType::String).with_provider_name("ASN"),
                    StructField::new("cookie", AttributeType::Struct {
                    name: "RateLimitCookie".to_string(),
                    fields: vec![
                    StructField::new("name", AttributeType::Custom {
                identity: None,
                pattern: Some(".*\\S.*".to_string()),
                length: Some((Some(1), Some(64))),
                base: Box::new(AttributeType::String),
                validate: legacy_validator(validate_string_pattern_4e09c821aed9e752_len_1_64),
                to_dsl: None,
            }).required().with_description("The name of the cookie to use.").with_provider_name("Name"),
                    StructField::new("text_transformations", AttributeType::list(AttributeType::Struct {
                    name: "TextTransformation".to_string(),
                    fields: vec![
                    StructField::new("priority", AttributeType::String).required().with_provider_name("Priority"),
                    StructField::new("type", AttributeType::StringEnum {
                name: "TextTransformationType".to_string(),
                values: vec!["NONE".to_string(), "COMPRESS_WHITE_SPACE".to_string(), "HTML_ENTITY_DECODE".to_string(), "LOWERCASE".to_string(), "CMD_LINE".to_string(), "URL_DECODE".to_string(), "BASE64_DECODE".to_string(), "HEX_DECODE".to_string(), "MD5".to_string(), "REPLACE_COMMENTS".to_string(), "ESCAPE_SEQ_DECODE".to_string(), "SQL_HEX_DECODE".to_string(), "CSS_DECODE".to_string(), "JS_DECODE".to_string(), "NORMALIZE_PATH".to_string(), "NORMALIZE_PATH_WIN".to_string(), "REMOVE_NULLS".to_string(), "REPLACE_NULLS".to_string(), "BASE64_DECODE_EXT".to_string(), "URL_DECODE_UNI".to_string(), "UTF8_TO_UNICODE".to_string()],
                identity: Some(carina_core::schema::string_enum_identity("TextTransformationType", Some("awscc.wafv2.WebAcl"))),
                dsl_aliases: vec![("NONE".to_string(), "none".to_string()), ("COMPRESS_WHITE_SPACE".to_string(), "compress_white_space".to_string()), ("HTML_ENTITY_DECODE".to_string(), "html_entity_decode".to_string()), ("LOWERCASE".to_string(), "lowercase".to_string()), ("CMD_LINE".to_string(), "cmd_line".to_string()), ("URL_DECODE".to_string(), "url_decode".to_string()), ("BASE64_DECODE".to_string(), "base64_decode".to_string()), ("HEX_DECODE".to_string(), "hex_decode".to_string()), ("MD5".to_string(), "md5".to_string()), ("REPLACE_COMMENTS".to_string(), "replace_comments".to_string()), ("ESCAPE_SEQ_DECODE".to_string(), "escape_seq_decode".to_string()), ("SQL_HEX_DECODE".to_string(), "sql_hex_decode".to_string()), ("CSS_DECODE".to_string(), "css_decode".to_string()), ("JS_DECODE".to_string(), "js_decode".to_string()), ("NORMALIZE_PATH".to_string(), "normalize_path".to_string()), ("NORMALIZE_PATH_WIN".to_string(), "normalize_path_win".to_string()), ("REMOVE_NULLS".to_string(), "remove_nulls".to_string()), ("REPLACE_NULLS".to_string(), "replace_nulls".to_string()), ("BASE64_DECODE_EXT".to_string(), "base64_decode_ext".to_string()), ("URL_DECODE_UNI".to_string(), "url_decode_uni".to_string()), ("UTF8_TO_UNICODE".to_string(), "utf8_to_unicode".to_string())],
            }).required().with_provider_name("Type")
                    ],
                })).required().with_provider_name("TextTransformations").with_block_name("text_transformation")
                    ],
                }).with_provider_name("Cookie").with_block_name("cookie"),
                    StructField::new("forwarded_ip", types::ipv4_address()).with_provider_name("ForwardedIP"),
                    StructField::new("http_method", AttributeType::String).with_provider_name("HTTPMethod"),
                    StructField::new("header", AttributeType::Struct {
                    name: "RateLimitHeader".to_string(),
                    fields: vec![
                    StructField::new("name", AttributeType::Custom {
                identity: None,
                pattern: Some(".*\\S.*".to_string()),
                length: Some((Some(1), Some(64))),
                base: Box::new(AttributeType::String),
                validate: legacy_validator(validate_string_pattern_4e09c821aed9e752_len_1_64),
                to_dsl: None,
            }).required().with_description("The name of the header to use.").with_provider_name("Name"),
                    StructField::new("text_transformations", AttributeType::list(AttributeType::Struct {
                    name: "TextTransformation".to_string(),
                    fields: vec![
                    StructField::new("priority", AttributeType::String).required().with_provider_name("Priority"),
                    StructField::new("type", AttributeType::StringEnum {
                name: "TextTransformationType".to_string(),
                values: vec!["NONE".to_string(), "COMPRESS_WHITE_SPACE".to_string(), "HTML_ENTITY_DECODE".to_string(), "LOWERCASE".to_string(), "CMD_LINE".to_string(), "URL_DECODE".to_string(), "BASE64_DECODE".to_string(), "HEX_DECODE".to_string(), "MD5".to_string(), "REPLACE_COMMENTS".to_string(), "ESCAPE_SEQ_DECODE".to_string(), "SQL_HEX_DECODE".to_string(), "CSS_DECODE".to_string(), "JS_DECODE".to_string(), "NORMALIZE_PATH".to_string(), "NORMALIZE_PATH_WIN".to_string(), "REMOVE_NULLS".to_string(), "REPLACE_NULLS".to_string(), "BASE64_DECODE_EXT".to_string(), "URL_DECODE_UNI".to_string(), "UTF8_TO_UNICODE".to_string()],
                identity: Some(carina_core::schema::string_enum_identity("TextTransformationType", Some("awscc.wafv2.WebAcl"))),
                dsl_aliases: vec![("NONE".to_string(), "none".to_string()), ("COMPRESS_WHITE_SPACE".to_string(), "compress_white_space".to_string()), ("HTML_ENTITY_DECODE".to_string(), "html_entity_decode".to_string()), ("LOWERCASE".to_string(), "lowercase".to_string()), ("CMD_LINE".to_string(), "cmd_line".to_string()), ("URL_DECODE".to_string(), "url_decode".to_string()), ("BASE64_DECODE".to_string(), "base64_decode".to_string()), ("HEX_DECODE".to_string(), "hex_decode".to_string()), ("MD5".to_string(), "md5".to_string()), ("REPLACE_COMMENTS".to_string(), "replace_comments".to_string()), ("ESCAPE_SEQ_DECODE".to_string(), "escape_seq_decode".to_string()), ("SQL_HEX_DECODE".to_string(), "sql_hex_decode".to_string()), ("CSS_DECODE".to_string(), "css_decode".to_string()), ("JS_DECODE".to_string(), "js_decode".to_string()), ("NORMALIZE_PATH".to_string(), "normalize_path".to_string()), ("NORMALIZE_PATH_WIN".to_string(), "normalize_path_win".to_string()), ("REMOVE_NULLS".to_string(), "remove_nulls".to_string()), ("REPLACE_NULLS".to_string(), "replace_nulls".to_string()), ("BASE64_DECODE_EXT".to_string(), "base64_decode_ext".to_string()), ("URL_DECODE_UNI".to_string(), "url_decode_uni".to_string()), ("UTF8_TO_UNICODE".to_string(), "utf8_to_unicode".to_string())],
            }).required().with_provider_name("Type")
                    ],
                })).required().with_provider_name("TextTransformations").with_block_name("text_transformation")
                    ],
                }).with_provider_name("Header").with_block_name("header"),
                    StructField::new("ip", types::ipv4_address()).with_provider_name("IP"),
                    StructField::new("ja3_fingerprint", AttributeType::Struct {
                    name: "RateLimitJA3Fingerprint".to_string(),
                    fields: vec![
                    StructField::new("fallback_behavior", AttributeType::StringEnum {
                name: "FallbackBehavior".to_string(),
                values: vec!["MATCH".to_string(), "NO_MATCH".to_string()],
                identity: Some(carina_core::schema::string_enum_identity("FallbackBehavior", Some("awscc.wafv2.WebAcl"))),
                dsl_aliases: vec![("MATCH".to_string(), "match".to_string()), ("NO_MATCH".to_string(), "no_match".to_string())],
            }).required().with_provider_name("FallbackBehavior")
                    ],
                }).with_provider_name("JA3Fingerprint"),
                    StructField::new("ja4_fingerprint", AttributeType::Struct {
                    name: "RateLimitJA4Fingerprint".to_string(),
                    fields: vec![
                    StructField::new("fallback_behavior", AttributeType::StringEnum {
                name: "FallbackBehavior".to_string(),
                values: vec!["MATCH".to_string(), "NO_MATCH".to_string()],
                identity: Some(carina_core::schema::string_enum_identity("FallbackBehavior", Some("awscc.wafv2.WebAcl"))),
                dsl_aliases: vec![("MATCH".to_string(), "match".to_string()), ("NO_MATCH".to_string(), "no_match".to_string())],
            }).required().with_provider_name("FallbackBehavior")
                    ],
                }).with_provider_name("JA4Fingerprint"),
                    StructField::new("label_namespace", AttributeType::Struct {
                    name: "RateLimitLabelNamespace".to_string(),
                    fields: vec![
                    StructField::new("namespace", AttributeType::Custom {
                identity: None,
                pattern: Some("^[0-9A-Za-z_:-]{1,1024}$".to_string()),
                length: None,
                base: Box::new(AttributeType::String),
                validate: legacy_validator(validate_string_pattern_b3fc65b549fb77bd),
                to_dsl: None,
            }).required().with_description("The namespace to use for aggregation.").with_provider_name("Namespace")
                    ],
                }).with_provider_name("LabelNamespace"),
                    StructField::new("query_argument", AttributeType::Struct {
                    name: "RateLimitQueryArgument".to_string(),
                    fields: vec![
                    StructField::new("name", AttributeType::Custom {
                identity: None,
                pattern: Some(".*\\S.*".to_string()),
                length: Some((Some(1), Some(64))),
                base: Box::new(AttributeType::String),
                validate: legacy_validator(validate_string_pattern_4e09c821aed9e752_len_1_64),
                to_dsl: None,
            }).required().with_description("The name of the query argument to use.").with_provider_name("Name"),
                    StructField::new("text_transformations", AttributeType::list(AttributeType::Struct {
                    name: "TextTransformation".to_string(),
                    fields: vec![
                    StructField::new("priority", AttributeType::String).required().with_provider_name("Priority"),
                    StructField::new("type", AttributeType::StringEnum {
                name: "TextTransformationType".to_string(),
                values: vec!["NONE".to_string(), "COMPRESS_WHITE_SPACE".to_string(), "HTML_ENTITY_DECODE".to_string(), "LOWERCASE".to_string(), "CMD_LINE".to_string(), "URL_DECODE".to_string(), "BASE64_DECODE".to_string(), "HEX_DECODE".to_string(), "MD5".to_string(), "REPLACE_COMMENTS".to_string(), "ESCAPE_SEQ_DECODE".to_string(), "SQL_HEX_DECODE".to_string(), "CSS_DECODE".to_string(), "JS_DECODE".to_string(), "NORMALIZE_PATH".to_string(), "NORMALIZE_PATH_WIN".to_string(), "REMOVE_NULLS".to_string(), "REPLACE_NULLS".to_string(), "BASE64_DECODE_EXT".to_string(), "URL_DECODE_UNI".to_string(), "UTF8_TO_UNICODE".to_string()],
                identity: Some(carina_core::schema::string_enum_identity("TextTransformationType", Some("awscc.wafv2.WebAcl"))),
                dsl_aliases: vec![("NONE".to_string(), "none".to_string()), ("COMPRESS_WHITE_SPACE".to_string(), "compress_white_space".to_string()), ("HTML_ENTITY_DECODE".to_string(), "html_entity_decode".to_string()), ("LOWERCASE".to_string(), "lowercase".to_string()), ("CMD_LINE".to_string(), "cmd_line".to_string()), ("URL_DECODE".to_string(), "url_decode".to_string()), ("BASE64_DECODE".to_string(), "base64_decode".to_string()), ("HEX_DECODE".to_string(), "hex_decode".to_string()), ("MD5".to_string(), "md5".to_string()), ("REPLACE_COMMENTS".to_string(), "replace_comments".to_string()), ("ESCAPE_SEQ_DECODE".to_string(), "escape_seq_decode".to_string()), ("SQL_HEX_DECODE".to_string(), "sql_hex_decode".to_string()), ("CSS_DECODE".to_string(), "css_decode".to_string()), ("JS_DECODE".to_string(), "js_decode".to_string()), ("NORMALIZE_PATH".to_string(), "normalize_path".to_string()), ("NORMALIZE_PATH_WIN".to_string(), "normalize_path_win".to_string()), ("REMOVE_NULLS".to_string(), "remove_nulls".to_string()), ("REPLACE_NULLS".to_string(), "replace_nulls".to_string()), ("BASE64_DECODE_EXT".to_string(), "base64_decode_ext".to_string()), ("URL_DECODE_UNI".to_string(), "url_decode_uni".to_string()), ("UTF8_TO_UNICODE".to_string(), "utf8_to_unicode".to_string())],
            }).required().with_provider_name("Type")
                    ],
                })).required().with_provider_name("TextTransformations").with_block_name("text_transformation")
                    ],
                }).with_provider_name("QueryArgument").with_block_name("query_argument"),
                    StructField::new("query_string", AttributeType::Struct {
                    name: "RateLimitQueryString".to_string(),
                    fields: vec![
                    StructField::new("text_transformations", AttributeType::list(AttributeType::Struct {
                    name: "TextTransformation".to_string(),
                    fields: vec![
                    StructField::new("priority", AttributeType::String).required().with_provider_name("Priority"),
                    StructField::new("type", AttributeType::StringEnum {
                name: "TextTransformationType".to_string(),
                values: vec!["NONE".to_string(), "COMPRESS_WHITE_SPACE".to_string(), "HTML_ENTITY_DECODE".to_string(), "LOWERCASE".to_string(), "CMD_LINE".to_string(), "URL_DECODE".to_string(), "BASE64_DECODE".to_string(), "HEX_DECODE".to_string(), "MD5".to_string(), "REPLACE_COMMENTS".to_string(), "ESCAPE_SEQ_DECODE".to_string(), "SQL_HEX_DECODE".to_string(), "CSS_DECODE".to_string(), "JS_DECODE".to_string(), "NORMALIZE_PATH".to_string(), "NORMALIZE_PATH_WIN".to_string(), "REMOVE_NULLS".to_string(), "REPLACE_NULLS".to_string(), "BASE64_DECODE_EXT".to_string(), "URL_DECODE_UNI".to_string(), "UTF8_TO_UNICODE".to_string()],
                identity: Some(carina_core::schema::string_enum_identity("TextTransformationType", Some("awscc.wafv2.WebAcl"))),
                dsl_aliases: vec![("NONE".to_string(), "none".to_string()), ("COMPRESS_WHITE_SPACE".to_string(), "compress_white_space".to_string()), ("HTML_ENTITY_DECODE".to_string(), "html_entity_decode".to_string()), ("LOWERCASE".to_string(), "lowercase".to_string()), ("CMD_LINE".to_string(), "cmd_line".to_string()), ("URL_DECODE".to_string(), "url_decode".to_string()), ("BASE64_DECODE".to_string(), "base64_decode".to_string()), ("HEX_DECODE".to_string(), "hex_decode".to_string()), ("MD5".to_string(), "md5".to_string()), ("REPLACE_COMMENTS".to_string(), "replace_comments".to_string()), ("ESCAPE_SEQ_DECODE".to_string(), "escape_seq_decode".to_string()), ("SQL_HEX_DECODE".to_string(), "sql_hex_decode".to_string()), ("CSS_DECODE".to_string(), "css_decode".to_string()), ("JS_DECODE".to_string(), "js_decode".to_string()), ("NORMALIZE_PATH".to_string(), "normalize_path".to_string()), ("NORMALIZE_PATH_WIN".to_string(), "normalize_path_win".to_string()), ("REMOVE_NULLS".to_string(), "remove_nulls".to_string()), ("REPLACE_NULLS".to_string(), "replace_nulls".to_string()), ("BASE64_DECODE_EXT".to_string(), "base64_decode_ext".to_string()), ("URL_DECODE_UNI".to_string(), "url_decode_uni".to_string()), ("UTF8_TO_UNICODE".to_string(), "utf8_to_unicode".to_string())],
            }).required().with_provider_name("Type")
                    ],
                })).required().with_provider_name("TextTransformations").with_block_name("text_transformation")
                    ],
                }).with_provider_name("QueryString").with_block_name("query_string"),
                    StructField::new("uri_path", AttributeType::Struct {
                    name: "RateLimitUriPath".to_string(),
                    fields: vec![
                    StructField::new("text_transformations", AttributeType::list(AttributeType::Struct {
                    name: "TextTransformation".to_string(),
                    fields: vec![
                    StructField::new("priority", AttributeType::String).required().with_provider_name("Priority"),
                    StructField::new("type", AttributeType::StringEnum {
                name: "TextTransformationType".to_string(),
                values: vec!["NONE".to_string(), "COMPRESS_WHITE_SPACE".to_string(), "HTML_ENTITY_DECODE".to_string(), "LOWERCASE".to_string(), "CMD_LINE".to_string(), "URL_DECODE".to_string(), "BASE64_DECODE".to_string(), "HEX_DECODE".to_string(), "MD5".to_string(), "REPLACE_COMMENTS".to_string(), "ESCAPE_SEQ_DECODE".to_string(), "SQL_HEX_DECODE".to_string(), "CSS_DECODE".to_string(), "JS_DECODE".to_string(), "NORMALIZE_PATH".to_string(), "NORMALIZE_PATH_WIN".to_string(), "REMOVE_NULLS".to_string(), "REPLACE_NULLS".to_string(), "BASE64_DECODE_EXT".to_string(), "URL_DECODE_UNI".to_string(), "UTF8_TO_UNICODE".to_string()],
                identity: Some(carina_core::schema::string_enum_identity("TextTransformationType", Some("awscc.wafv2.WebAcl"))),
                dsl_aliases: vec![("NONE".to_string(), "none".to_string()), ("COMPRESS_WHITE_SPACE".to_string(), "compress_white_space".to_string()), ("HTML_ENTITY_DECODE".to_string(), "html_entity_decode".to_string()), ("LOWERCASE".to_string(), "lowercase".to_string()), ("CMD_LINE".to_string(), "cmd_line".to_string()), ("URL_DECODE".to_string(), "url_decode".to_string()), ("BASE64_DECODE".to_string(), "base64_decode".to_string()), ("HEX_DECODE".to_string(), "hex_decode".to_string()), ("MD5".to_string(), "md5".to_string()), ("REPLACE_COMMENTS".to_string(), "replace_comments".to_string()), ("ESCAPE_SEQ_DECODE".to_string(), "escape_seq_decode".to_string()), ("SQL_HEX_DECODE".to_string(), "sql_hex_decode".to_string()), ("CSS_DECODE".to_string(), "css_decode".to_string()), ("JS_DECODE".to_string(), "js_decode".to_string()), ("NORMALIZE_PATH".to_string(), "normalize_path".to_string()), ("NORMALIZE_PATH_WIN".to_string(), "normalize_path_win".to_string()), ("REMOVE_NULLS".to_string(), "remove_nulls".to_string()), ("REPLACE_NULLS".to_string(), "replace_nulls".to_string()), ("BASE64_DECODE_EXT".to_string(), "base64_decode_ext".to_string()), ("URL_DECODE_UNI".to_string(), "url_decode_uni".to_string()), ("UTF8_TO_UNICODE".to_string(), "utf8_to_unicode".to_string())],
            }).required().with_provider_name("Type")
                    ],
                })).required().with_provider_name("TextTransformations").with_block_name("text_transformation")
                    ],
                }).with_provider_name("UriPath").with_block_name("uri_path")
                    ],
                })),
                validate: legacy_validator(validate_list_items_max_5),
                to_dsl: None,
            }).with_description("Specifies the aggregate keys to use in a rate-base rule.").with_provider_name("CustomKeys").with_block_name("custom_key"),
                    StructField::new("evaluation_window_sec", AttributeType::StringEnum {
                name: "EvaluationWindowSec".to_string(),
                values: vec!["60".to_string(), "120".to_string(), "300".to_string(), "600".to_string()],
                identity: Some(carina_core::schema::string_enum_identity("EvaluationWindowSec", Some("awscc.wafv2.WebAcl"))),
                dsl_aliases: vec![("60".to_string(), "60".to_string()), ("120".to_string(), "120".to_string()), ("300".to_string(), "300".to_string()), ("600".to_string(), "600".to_string())],
            }).with_provider_name("EvaluationWindowSec"),
                    StructField::new("forwarded_ip_config", AttributeType::Ref("ForwardedIPConfiguration".to_string())).with_provider_name("ForwardedIPConfig"),
                    StructField::new("limit", AttributeType::String).required().with_provider_name("Limit"),
                    StructField::new("scope_down_statement", AttributeType::Ref("Statement".to_string())).with_provider_name("ScopeDownStatement")
                    ],
                }).with_provider_name("RateBasedStatement").with_block_name("rate_based_statement"),
                    StructField::new("regex_match_statement", AttributeType::Struct {
                    name: "RegexMatchStatement".to_string(),
                    fields: vec![
                    StructField::new("field_to_match", AttributeType::Ref("FieldToMatch".to_string())).required().with_provider_name("FieldToMatch"),
                    StructField::new("regex_string", AttributeType::Custom {
                identity: None,
                pattern: None,
                length: Some((Some(1), Some(512))),
                base: Box::new(AttributeType::String),
                validate: legacy_validator(validate_string_length_1_512),
                to_dsl: None,
            }).required().with_provider_name("RegexString"),
                    StructField::new("text_transformations", AttributeType::list(AttributeType::Struct {
                    name: "TextTransformation".to_string(),
                    fields: vec![
                    StructField::new("priority", AttributeType::String).required().with_provider_name("Priority"),
                    StructField::new("type", AttributeType::StringEnum {
                name: "TextTransformationType".to_string(),
                values: vec!["NONE".to_string(), "COMPRESS_WHITE_SPACE".to_string(), "HTML_ENTITY_DECODE".to_string(), "LOWERCASE".to_string(), "CMD_LINE".to_string(), "URL_DECODE".to_string(), "BASE64_DECODE".to_string(), "HEX_DECODE".to_string(), "MD5".to_string(), "REPLACE_COMMENTS".to_string(), "ESCAPE_SEQ_DECODE".to_string(), "SQL_HEX_DECODE".to_string(), "CSS_DECODE".to_string(), "JS_DECODE".to_string(), "NORMALIZE_PATH".to_string(), "NORMALIZE_PATH_WIN".to_string(), "REMOVE_NULLS".to_string(), "REPLACE_NULLS".to_string(), "BASE64_DECODE_EXT".to_string(), "URL_DECODE_UNI".to_string(), "UTF8_TO_UNICODE".to_string()],
                identity: Some(carina_core::schema::string_enum_identity("TextTransformationType", Some("awscc.wafv2.WebAcl"))),
                dsl_aliases: vec![("NONE".to_string(), "none".to_string()), ("COMPRESS_WHITE_SPACE".to_string(), "compress_white_space".to_string()), ("HTML_ENTITY_DECODE".to_string(), "html_entity_decode".to_string()), ("LOWERCASE".to_string(), "lowercase".to_string()), ("CMD_LINE".to_string(), "cmd_line".to_string()), ("URL_DECODE".to_string(), "url_decode".to_string()), ("BASE64_DECODE".to_string(), "base64_decode".to_string()), ("HEX_DECODE".to_string(), "hex_decode".to_string()), ("MD5".to_string(), "md5".to_string()), ("REPLACE_COMMENTS".to_string(), "replace_comments".to_string()), ("ESCAPE_SEQ_DECODE".to_string(), "escape_seq_decode".to_string()), ("SQL_HEX_DECODE".to_string(), "sql_hex_decode".to_string()), ("CSS_DECODE".to_string(), "css_decode".to_string()), ("JS_DECODE".to_string(), "js_decode".to_string()), ("NORMALIZE_PATH".to_string(), "normalize_path".to_string()), ("NORMALIZE_PATH_WIN".to_string(), "normalize_path_win".to_string()), ("REMOVE_NULLS".to_string(), "remove_nulls".to_string()), ("REPLACE_NULLS".to_string(), "replace_nulls".to_string()), ("BASE64_DECODE_EXT".to_string(), "base64_decode_ext".to_string()), ("URL_DECODE_UNI".to_string(), "url_decode_uni".to_string()), ("UTF8_TO_UNICODE".to_string(), "utf8_to_unicode".to_string())],
            }).required().with_provider_name("Type")
                    ],
                })).required().with_provider_name("TextTransformations").with_block_name("text_transformation")
                    ],
                }).with_provider_name("RegexMatchStatement").with_block_name("regex_match_statement"),
                    StructField::new("regex_pattern_set_reference_statement", AttributeType::Struct {
                    name: "RegexPatternSetReferenceStatement".to_string(),
                    fields: vec![
                    StructField::new("arn", super::arn()).required().with_provider_name("Arn"),
                    StructField::new("field_to_match", AttributeType::Ref("FieldToMatch".to_string())).required().with_provider_name("FieldToMatch"),
                    StructField::new("text_transformations", AttributeType::list(AttributeType::Struct {
                    name: "TextTransformation".to_string(),
                    fields: vec![
                    StructField::new("priority", AttributeType::String).required().with_provider_name("Priority"),
                    StructField::new("type", AttributeType::StringEnum {
                name: "TextTransformationType".to_string(),
                values: vec!["NONE".to_string(), "COMPRESS_WHITE_SPACE".to_string(), "HTML_ENTITY_DECODE".to_string(), "LOWERCASE".to_string(), "CMD_LINE".to_string(), "URL_DECODE".to_string(), "BASE64_DECODE".to_string(), "HEX_DECODE".to_string(), "MD5".to_string(), "REPLACE_COMMENTS".to_string(), "ESCAPE_SEQ_DECODE".to_string(), "SQL_HEX_DECODE".to_string(), "CSS_DECODE".to_string(), "JS_DECODE".to_string(), "NORMALIZE_PATH".to_string(), "NORMALIZE_PATH_WIN".to_string(), "REMOVE_NULLS".to_string(), "REPLACE_NULLS".to_string(), "BASE64_DECODE_EXT".to_string(), "URL_DECODE_UNI".to_string(), "UTF8_TO_UNICODE".to_string()],
                identity: Some(carina_core::schema::string_enum_identity("TextTransformationType", Some("awscc.wafv2.WebAcl"))),
                dsl_aliases: vec![("NONE".to_string(), "none".to_string()), ("COMPRESS_WHITE_SPACE".to_string(), "compress_white_space".to_string()), ("HTML_ENTITY_DECODE".to_string(), "html_entity_decode".to_string()), ("LOWERCASE".to_string(), "lowercase".to_string()), ("CMD_LINE".to_string(), "cmd_line".to_string()), ("URL_DECODE".to_string(), "url_decode".to_string()), ("BASE64_DECODE".to_string(), "base64_decode".to_string()), ("HEX_DECODE".to_string(), "hex_decode".to_string()), ("MD5".to_string(), "md5".to_string()), ("REPLACE_COMMENTS".to_string(), "replace_comments".to_string()), ("ESCAPE_SEQ_DECODE".to_string(), "escape_seq_decode".to_string()), ("SQL_HEX_DECODE".to_string(), "sql_hex_decode".to_string()), ("CSS_DECODE".to_string(), "css_decode".to_string()), ("JS_DECODE".to_string(), "js_decode".to_string()), ("NORMALIZE_PATH".to_string(), "normalize_path".to_string()), ("NORMALIZE_PATH_WIN".to_string(), "normalize_path_win".to_string()), ("REMOVE_NULLS".to_string(), "remove_nulls".to_string()), ("REPLACE_NULLS".to_string(), "replace_nulls".to_string()), ("BASE64_DECODE_EXT".to_string(), "base64_decode_ext".to_string()), ("URL_DECODE_UNI".to_string(), "url_decode_uni".to_string()), ("UTF8_TO_UNICODE".to_string(), "utf8_to_unicode".to_string())],
            }).required().with_provider_name("Type")
                    ],
                })).required().with_provider_name("TextTransformations").with_block_name("text_transformation")
                    ],
                }).with_provider_name("RegexPatternSetReferenceStatement").with_block_name("regex_pattern_set_reference_statement"),
                    StructField::new("rule_group_reference_statement", AttributeType::Struct {
                    name: "RuleGroupReferenceStatement".to_string(),
                    fields: vec![
                    StructField::new("arn", super::arn()).required().with_provider_name("Arn"),
                    StructField::new("excluded_rules", AttributeType::list(AttributeType::Struct {
                    name: "ExcludedRule".to_string(),
                    fields: vec![
                    StructField::new("name", AttributeType::Custom {
                identity: None,
                pattern: Some("^[0-9A-Za-z_-]{1,128}$".to_string()),
                length: None,
                base: Box::new(AttributeType::String),
                validate: legacy_validator(validate_string_pattern_42f7eceb887966ad),
                to_dsl: None,
            }).required().with_provider_name("Name")
                    ],
                })).with_provider_name("ExcludedRules").with_block_name("excluded_rule"),
                    StructField::new("rule_action_overrides", AttributeType::Custom {
                identity: None,
                pattern: None,
                length: None,
                base: Box::new(AttributeType::list(AttributeType::Struct {
                    name: "RuleActionOverride".to_string(),
                    fields: vec![
                    StructField::new("action_to_use", AttributeType::Ref("RuleAction".to_string())).required().with_provider_name("ActionToUse"),
                    StructField::new("name", AttributeType::Custom {
                identity: None,
                pattern: Some("^[0-9A-Za-z_-]{1,128}$".to_string()),
                length: None,
                base: Box::new(AttributeType::String),
                validate: legacy_validator(validate_string_pattern_42f7eceb887966ad),
                to_dsl: None,
            }).required().with_provider_name("Name")
                    ],
                })),
                validate: legacy_validator(validate_list_items_max_100),
                to_dsl: None,
            }).with_description("Action overrides for rules in the rule group.").with_provider_name("RuleActionOverrides").with_block_name("rule_action_override")
                    ],
                }).with_provider_name("RuleGroupReferenceStatement").with_block_name("rule_group_reference_statement"),
                    StructField::new("size_constraint_statement", AttributeType::Struct {
                    name: "SizeConstraintStatement".to_string(),
                    fields: vec![
                    StructField::new("comparison_operator", AttributeType::StringEnum {
                name: "ComparisonOperator".to_string(),
                values: vec!["EQ".to_string(), "NE".to_string(), "LE".to_string(), "LT".to_string(), "GE".to_string(), "GT".to_string()],
                identity: Some(carina_core::schema::string_enum_identity("ComparisonOperator", Some("awscc.wafv2.WebAcl"))),
                dsl_aliases: vec![("EQ".to_string(), "eq".to_string()), ("NE".to_string(), "ne".to_string()), ("LE".to_string(), "le".to_string()), ("LT".to_string(), "lt".to_string()), ("GE".to_string(), "ge".to_string()), ("GT".to_string(), "gt".to_string())],
            }).required().with_provider_name("ComparisonOperator"),
                    StructField::new("field_to_match", AttributeType::Ref("FieldToMatch".to_string())).required().with_provider_name("FieldToMatch"),
                    StructField::new("size", AttributeType::Custom {
                identity: None,
                pattern: None,
                length: None,
                base: Box::new(AttributeType::Float),
                validate: legacy_validator(validate_size_range),
                to_dsl: None,
            }).required().with_provider_name("Size"),
                    StructField::new("text_transformations", AttributeType::list(AttributeType::Struct {
                    name: "TextTransformation".to_string(),
                    fields: vec![
                    StructField::new("priority", AttributeType::String).required().with_provider_name("Priority"),
                    StructField::new("type", AttributeType::StringEnum {
                name: "TextTransformationType".to_string(),
                values: vec!["NONE".to_string(), "COMPRESS_WHITE_SPACE".to_string(), "HTML_ENTITY_DECODE".to_string(), "LOWERCASE".to_string(), "CMD_LINE".to_string(), "URL_DECODE".to_string(), "BASE64_DECODE".to_string(), "HEX_DECODE".to_string(), "MD5".to_string(), "REPLACE_COMMENTS".to_string(), "ESCAPE_SEQ_DECODE".to_string(), "SQL_HEX_DECODE".to_string(), "CSS_DECODE".to_string(), "JS_DECODE".to_string(), "NORMALIZE_PATH".to_string(), "NORMALIZE_PATH_WIN".to_string(), "REMOVE_NULLS".to_string(), "REPLACE_NULLS".to_string(), "BASE64_DECODE_EXT".to_string(), "URL_DECODE_UNI".to_string(), "UTF8_TO_UNICODE".to_string()],
                identity: Some(carina_core::schema::string_enum_identity("TextTransformationType", Some("awscc.wafv2.WebAcl"))),
                dsl_aliases: vec![("NONE".to_string(), "none".to_string()), ("COMPRESS_WHITE_SPACE".to_string(), "compress_white_space".to_string()), ("HTML_ENTITY_DECODE".to_string(), "html_entity_decode".to_string()), ("LOWERCASE".to_string(), "lowercase".to_string()), ("CMD_LINE".to_string(), "cmd_line".to_string()), ("URL_DECODE".to_string(), "url_decode".to_string()), ("BASE64_DECODE".to_string(), "base64_decode".to_string()), ("HEX_DECODE".to_string(), "hex_decode".to_string()), ("MD5".to_string(), "md5".to_string()), ("REPLACE_COMMENTS".to_string(), "replace_comments".to_string()), ("ESCAPE_SEQ_DECODE".to_string(), "escape_seq_decode".to_string()), ("SQL_HEX_DECODE".to_string(), "sql_hex_decode".to_string()), ("CSS_DECODE".to_string(), "css_decode".to_string()), ("JS_DECODE".to_string(), "js_decode".to_string()), ("NORMALIZE_PATH".to_string(), "normalize_path".to_string()), ("NORMALIZE_PATH_WIN".to_string(), "normalize_path_win".to_string()), ("REMOVE_NULLS".to_string(), "remove_nulls".to_string()), ("REPLACE_NULLS".to_string(), "replace_nulls".to_string()), ("BASE64_DECODE_EXT".to_string(), "base64_decode_ext".to_string()), ("URL_DECODE_UNI".to_string(), "url_decode_uni".to_string()), ("UTF8_TO_UNICODE".to_string(), "utf8_to_unicode".to_string())],
            }).required().with_provider_name("Type")
                    ],
                })).required().with_provider_name("TextTransformations").with_block_name("text_transformation")
                    ],
                }).with_provider_name("SizeConstraintStatement").with_block_name("size_constraint_statement"),
                    StructField::new("sqli_match_statement", AttributeType::Struct {
                    name: "SqliMatchStatement".to_string(),
                    fields: vec![
                    StructField::new("field_to_match", AttributeType::Ref("FieldToMatch".to_string())).required().with_provider_name("FieldToMatch"),
                    StructField::new("sensitivity_level", AttributeType::StringEnum {
                name: "SensitivityLevel".to_string(),
                values: vec!["LOW".to_string(), "HIGH".to_string()],
                identity: Some(carina_core::schema::string_enum_identity("SensitivityLevel", Some("awscc.wafv2.WebAcl"))),
                dsl_aliases: vec![("LOW".to_string(), "low".to_string()), ("HIGH".to_string(), "high".to_string())],
            }).with_provider_name("SensitivityLevel"),
                    StructField::new("text_transformations", AttributeType::list(AttributeType::Struct {
                    name: "TextTransformation".to_string(),
                    fields: vec![
                    StructField::new("priority", AttributeType::String).required().with_provider_name("Priority"),
                    StructField::new("type", AttributeType::StringEnum {
                name: "TextTransformationType".to_string(),
                values: vec!["NONE".to_string(), "COMPRESS_WHITE_SPACE".to_string(), "HTML_ENTITY_DECODE".to_string(), "LOWERCASE".to_string(), "CMD_LINE".to_string(), "URL_DECODE".to_string(), "BASE64_DECODE".to_string(), "HEX_DECODE".to_string(), "MD5".to_string(), "REPLACE_COMMENTS".to_string(), "ESCAPE_SEQ_DECODE".to_string(), "SQL_HEX_DECODE".to_string(), "CSS_DECODE".to_string(), "JS_DECODE".to_string(), "NORMALIZE_PATH".to_string(), "NORMALIZE_PATH_WIN".to_string(), "REMOVE_NULLS".to_string(), "REPLACE_NULLS".to_string(), "BASE64_DECODE_EXT".to_string(), "URL_DECODE_UNI".to_string(), "UTF8_TO_UNICODE".to_string()],
                identity: Some(carina_core::schema::string_enum_identity("TextTransformationType", Some("awscc.wafv2.WebAcl"))),
                dsl_aliases: vec![("NONE".to_string(), "none".to_string()), ("COMPRESS_WHITE_SPACE".to_string(), "compress_white_space".to_string()), ("HTML_ENTITY_DECODE".to_string(), "html_entity_decode".to_string()), ("LOWERCASE".to_string(), "lowercase".to_string()), ("CMD_LINE".to_string(), "cmd_line".to_string()), ("URL_DECODE".to_string(), "url_decode".to_string()), ("BASE64_DECODE".to_string(), "base64_decode".to_string()), ("HEX_DECODE".to_string(), "hex_decode".to_string()), ("MD5".to_string(), "md5".to_string()), ("REPLACE_COMMENTS".to_string(), "replace_comments".to_string()), ("ESCAPE_SEQ_DECODE".to_string(), "escape_seq_decode".to_string()), ("SQL_HEX_DECODE".to_string(), "sql_hex_decode".to_string()), ("CSS_DECODE".to_string(), "css_decode".to_string()), ("JS_DECODE".to_string(), "js_decode".to_string()), ("NORMALIZE_PATH".to_string(), "normalize_path".to_string()), ("NORMALIZE_PATH_WIN".to_string(), "normalize_path_win".to_string()), ("REMOVE_NULLS".to_string(), "remove_nulls".to_string()), ("REPLACE_NULLS".to_string(), "replace_nulls".to_string()), ("BASE64_DECODE_EXT".to_string(), "base64_decode_ext".to_string()), ("URL_DECODE_UNI".to_string(), "url_decode_uni".to_string()), ("UTF8_TO_UNICODE".to_string(), "utf8_to_unicode".to_string())],
            }).required().with_provider_name("Type")
                    ],
                })).required().with_provider_name("TextTransformations").with_block_name("text_transformation")
                    ],
                }).with_provider_name("SqliMatchStatement").with_block_name("sqli_match_statement"),
                    StructField::new("xss_match_statement", AttributeType::Struct {
                    name: "XssMatchStatement".to_string(),
                    fields: vec![
                    StructField::new("field_to_match", AttributeType::Ref("FieldToMatch".to_string())).required().with_provider_name("FieldToMatch"),
                    StructField::new("text_transformations", AttributeType::list(AttributeType::Struct {
                    name: "TextTransformation".to_string(),
                    fields: vec![
                    StructField::new("priority", AttributeType::String).required().with_provider_name("Priority"),
                    StructField::new("type", AttributeType::StringEnum {
                name: "TextTransformationType".to_string(),
                values: vec!["NONE".to_string(), "COMPRESS_WHITE_SPACE".to_string(), "HTML_ENTITY_DECODE".to_string(), "LOWERCASE".to_string(), "CMD_LINE".to_string(), "URL_DECODE".to_string(), "BASE64_DECODE".to_string(), "HEX_DECODE".to_string(), "MD5".to_string(), "REPLACE_COMMENTS".to_string(), "ESCAPE_SEQ_DECODE".to_string(), "SQL_HEX_DECODE".to_string(), "CSS_DECODE".to_string(), "JS_DECODE".to_string(), "NORMALIZE_PATH".to_string(), "NORMALIZE_PATH_WIN".to_string(), "REMOVE_NULLS".to_string(), "REPLACE_NULLS".to_string(), "BASE64_DECODE_EXT".to_string(), "URL_DECODE_UNI".to_string(), "UTF8_TO_UNICODE".to_string()],
                identity: Some(carina_core::schema::string_enum_identity("TextTransformationType", Some("awscc.wafv2.WebAcl"))),
                dsl_aliases: vec![("NONE".to_string(), "none".to_string()), ("COMPRESS_WHITE_SPACE".to_string(), "compress_white_space".to_string()), ("HTML_ENTITY_DECODE".to_string(), "html_entity_decode".to_string()), ("LOWERCASE".to_string(), "lowercase".to_string()), ("CMD_LINE".to_string(), "cmd_line".to_string()), ("URL_DECODE".to_string(), "url_decode".to_string()), ("BASE64_DECODE".to_string(), "base64_decode".to_string()), ("HEX_DECODE".to_string(), "hex_decode".to_string()), ("MD5".to_string(), "md5".to_string()), ("REPLACE_COMMENTS".to_string(), "replace_comments".to_string()), ("ESCAPE_SEQ_DECODE".to_string(), "escape_seq_decode".to_string()), ("SQL_HEX_DECODE".to_string(), "sql_hex_decode".to_string()), ("CSS_DECODE".to_string(), "css_decode".to_string()), ("JS_DECODE".to_string(), "js_decode".to_string()), ("NORMALIZE_PATH".to_string(), "normalize_path".to_string()), ("NORMALIZE_PATH_WIN".to_string(), "normalize_path_win".to_string()), ("REMOVE_NULLS".to_string(), "remove_nulls".to_string()), ("REPLACE_NULLS".to_string(), "replace_nulls".to_string()), ("BASE64_DECODE_EXT".to_string(), "base64_decode_ext".to_string()), ("URL_DECODE_UNI".to_string(), "url_decode_uni".to_string()), ("UTF8_TO_UNICODE".to_string(), "utf8_to_unicode".to_string())],
            }).required().with_provider_name("Type")
                    ],
                })).required().with_provider_name("TextTransformations").with_block_name("text_transformation")
                    ],
                }).with_provider_name("XssMatchStatement").with_block_name("xss_match_statement")
                    ],
                })).required().with_provider_name("Statements").with_block_name("statement")
                    ],
                }).with_provider_name("OrStatement").with_block_name("or_statement"),
                    StructField::new("rate_based_statement", AttributeType::Ref("RateBasedStatement".to_string())).with_provider_name("RateBasedStatement"),
                    StructField::new("regex_match_statement", AttributeType::Ref("RegexMatchStatement".to_string())).with_provider_name("RegexMatchStatement"),
                    StructField::new("regex_pattern_set_reference_statement", AttributeType::Ref("RegexPatternSetReferenceStatement".to_string())).with_provider_name("RegexPatternSetReferenceStatement"),
                    StructField::new("rule_group_reference_statement", AttributeType::Ref("RuleGroupReferenceStatement".to_string())).with_provider_name("RuleGroupReferenceStatement"),
                    StructField::new("size_constraint_statement", AttributeType::Ref("SizeConstraintStatement".to_string())).with_provider_name("SizeConstraintStatement"),
                    StructField::new("sqli_match_statement", AttributeType::Ref("SqliMatchStatement".to_string())).with_provider_name("SqliMatchStatement"),
                    StructField::new("xss_match_statement", AttributeType::Ref("XssMatchStatement".to_string())).with_provider_name("XssMatchStatement")
                    ],
                })).required().with_provider_name("Statements").with_block_name("statement")
                    ],
                }).with_provider_name("AndStatement").with_block_name("and_statement"),
                    StructField::new("asn_match_statement", AttributeType::Ref("AsnMatchStatement".to_string())).with_provider_name("AsnMatchStatement"),
                    StructField::new("byte_match_statement", AttributeType::Ref("ByteMatchStatement".to_string())).with_provider_name("ByteMatchStatement"),
                    StructField::new("geo_match_statement", AttributeType::Ref("GeoMatchStatement".to_string())).with_provider_name("GeoMatchStatement"),
                    StructField::new("ip_set_reference_statement", AttributeType::Ref("IPSetReferenceStatement".to_string())).with_provider_name("IPSetReferenceStatement"),
                    StructField::new("label_match_statement", AttributeType::Ref("LabelMatchStatement".to_string())).with_provider_name("LabelMatchStatement"),
                    StructField::new("managed_rule_group_statement", AttributeType::Ref("ManagedRuleGroupStatement".to_string())).with_provider_name("ManagedRuleGroupStatement"),
                    StructField::new("not_statement", AttributeType::Ref("NotStatement".to_string())).with_provider_name("NotStatement"),
                    StructField::new("or_statement", AttributeType::Ref("OrStatement".to_string())).with_provider_name("OrStatement"),
                    StructField::new("rate_based_statement", AttributeType::Ref("RateBasedStatement".to_string())).with_provider_name("RateBasedStatement"),
                    StructField::new("regex_match_statement", AttributeType::Ref("RegexMatchStatement".to_string())).with_provider_name("RegexMatchStatement"),
                    StructField::new("regex_pattern_set_reference_statement", AttributeType::Ref("RegexPatternSetReferenceStatement".to_string())).with_provider_name("RegexPatternSetReferenceStatement"),
                    StructField::new("rule_group_reference_statement", AttributeType::Ref("RuleGroupReferenceStatement".to_string())).with_provider_name("RuleGroupReferenceStatement"),
                    StructField::new("size_constraint_statement", AttributeType::Ref("SizeConstraintStatement".to_string())).with_provider_name("SizeConstraintStatement"),
                    StructField::new("sqli_match_statement", AttributeType::Ref("SqliMatchStatement".to_string())).with_provider_name("SqliMatchStatement"),
                    StructField::new("xss_match_statement", AttributeType::Ref("XssMatchStatement".to_string())).with_provider_name("XssMatchStatement")
                    ],
                })
        .with_def("UriFragment", AttributeType::Struct {
                    name: "UriFragment".to_string(),
                    fields: vec![
                    StructField::new("fallback_behavior", AttributeType::StringEnum {
                name: "FallbackBehavior".to_string(),
                values: vec!["MATCH".to_string(), "NO_MATCH".to_string()],
                identity: Some(carina_core::schema::string_enum_identity("FallbackBehavior", Some("awscc.wafv2.WebAcl"))),
                dsl_aliases: vec![("MATCH".to_string(), "match".to_string()), ("NO_MATCH".to_string(), "no_match".to_string())],
            }).with_provider_name("FallbackBehavior")
                    ],
                })
        .with_def("VisibilityConfig", AttributeType::Struct {
                    name: "VisibilityConfig".to_string(),
                    fields: vec![
                    StructField::new("cloud_watch_metrics_enabled", AttributeType::Bool).required().with_provider_name("CloudWatchMetricsEnabled"),
                    StructField::new("metric_name", AttributeType::Custom {
                identity: None,
                pattern: None,
                length: Some((Some(1), Some(128))),
                base: Box::new(AttributeType::String),
                validate: legacy_validator(validate_string_length_1_128),
                to_dsl: None,
            }).required().with_provider_name("MetricName"),
                    StructField::new("sampled_requests_enabled", AttributeType::Bool).required().with_provider_name("SampledRequestsEnabled")
                    ],
                })
        .with_def("XssMatchStatement", AttributeType::Struct {
                    name: "XssMatchStatement".to_string(),
                    fields: vec![
                    StructField::new("field_to_match", AttributeType::Ref("FieldToMatch".to_string())).required().with_provider_name("FieldToMatch"),
                    StructField::new("text_transformations", AttributeType::list(AttributeType::Struct {
                    name: "TextTransformation".to_string(),
                    fields: vec![
                    StructField::new("priority", AttributeType::String).required().with_provider_name("Priority"),
                    StructField::new("type", AttributeType::StringEnum {
                name: "TextTransformationType".to_string(),
                values: vec!["NONE".to_string(), "COMPRESS_WHITE_SPACE".to_string(), "HTML_ENTITY_DECODE".to_string(), "LOWERCASE".to_string(), "CMD_LINE".to_string(), "URL_DECODE".to_string(), "BASE64_DECODE".to_string(), "HEX_DECODE".to_string(), "MD5".to_string(), "REPLACE_COMMENTS".to_string(), "ESCAPE_SEQ_DECODE".to_string(), "SQL_HEX_DECODE".to_string(), "CSS_DECODE".to_string(), "JS_DECODE".to_string(), "NORMALIZE_PATH".to_string(), "NORMALIZE_PATH_WIN".to_string(), "REMOVE_NULLS".to_string(), "REPLACE_NULLS".to_string(), "BASE64_DECODE_EXT".to_string(), "URL_DECODE_UNI".to_string(), "UTF8_TO_UNICODE".to_string()],
                identity: Some(carina_core::schema::string_enum_identity("TextTransformationType", Some("awscc.wafv2.WebAcl"))),
                dsl_aliases: vec![("NONE".to_string(), "none".to_string()), ("COMPRESS_WHITE_SPACE".to_string(), "compress_white_space".to_string()), ("HTML_ENTITY_DECODE".to_string(), "html_entity_decode".to_string()), ("LOWERCASE".to_string(), "lowercase".to_string()), ("CMD_LINE".to_string(), "cmd_line".to_string()), ("URL_DECODE".to_string(), "url_decode".to_string()), ("BASE64_DECODE".to_string(), "base64_decode".to_string()), ("HEX_DECODE".to_string(), "hex_decode".to_string()), ("MD5".to_string(), "md5".to_string()), ("REPLACE_COMMENTS".to_string(), "replace_comments".to_string()), ("ESCAPE_SEQ_DECODE".to_string(), "escape_seq_decode".to_string()), ("SQL_HEX_DECODE".to_string(), "sql_hex_decode".to_string()), ("CSS_DECODE".to_string(), "css_decode".to_string()), ("JS_DECODE".to_string(), "js_decode".to_string()), ("NORMALIZE_PATH".to_string(), "normalize_path".to_string()), ("NORMALIZE_PATH_WIN".to_string(), "normalize_path_win".to_string()), ("REMOVE_NULLS".to_string(), "remove_nulls".to_string()), ("REPLACE_NULLS".to_string(), "replace_nulls".to_string()), ("BASE64_DECODE_EXT".to_string(), "base64_decode_ext".to_string()), ("URL_DECODE_UNI".to_string(), "url_decode_uni".to_string()), ("UTF8_TO_UNICODE".to_string(), "utf8_to_unicode".to_string())],
            }).required().with_provider_name("Type")
                    ],
                })).required().with_provider_name("TextTransformations").with_block_name("text_transformation")
                    ],
                })
    }
}

#[allow(dead_code)]
fn validate_string_pattern_4e09c821aed9e752_len_1_60(value: &Value) -> Result<(), String> {
    if let Value::Concrete(ConcreteValue::String(s)) = value {
        static RE: std::sync::LazyLock<Regex> =
            std::sync::LazyLock::new(|| Regex::new(".*\\S.*").expect("invalid pattern regex"));
        if !RE.is_match(s) {
            return Err(format!("Value '{}' does not match pattern .*\\S.*", s));
        }
        let len = s.chars().count();
        if !(1..=60).contains(&len) {
            return Err(format!("String length {} is out of range 1..=60", len));
        }
        Ok(())
    } else {
        Err("Expected string".to_string())
    }
}

#[allow(dead_code)]
fn validate_string_pattern_4e09c821aed9e752_len_1_64(value: &Value) -> Result<(), String> {
    if let Value::Concrete(ConcreteValue::String(s)) = value {
        static RE: std::sync::LazyLock<Regex> =
            std::sync::LazyLock::new(|| Regex::new(".*\\S.*").expect("invalid pattern regex"));
        if !RE.is_match(s) {
            return Err(format!("Value '{}' does not match pattern .*\\S.*", s));
        }
        let len = s.chars().count();
        if !(1..=64).contains(&len) {
            return Err(format!("String length {} is out of range 1..=64", len));
        }
        Ok(())
    } else {
        Err("Expected string".to_string())
    }
}

#[allow(dead_code)]
fn validate_string_pattern_4e09c821aed9e752_len_1_100(value: &Value) -> Result<(), String> {
    if let Value::Concrete(ConcreteValue::String(s)) = value {
        static RE: std::sync::LazyLock<Regex> =
            std::sync::LazyLock::new(|| Regex::new(".*\\S.*").expect("invalid pattern regex"));
        if !RE.is_match(s) {
            return Err(format!("Value '{}' does not match pattern .*\\S.*", s));
        }
        let len = s.chars().count();
        if !(1..=100).contains(&len) {
            return Err(format!("String length {} is out of range 1..=100", len));
        }
        Ok(())
    } else {
        Err("Expected string".to_string())
    }
}

#[allow(dead_code)]
fn validate_string_pattern_4e09c821aed9e752_len_1_200(value: &Value) -> Result<(), String> {
    if let Value::Concrete(ConcreteValue::String(s)) = value {
        static RE: std::sync::LazyLock<Regex> =
            std::sync::LazyLock::new(|| Regex::new(".*\\S.*").expect("invalid pattern regex"));
        if !RE.is_match(s) {
            return Err(format!("Value '{}' does not match pattern .*\\S.*", s));
        }
        let len = s.chars().count();
        if !(1..=200).contains(&len) {
            return Err(format!("String length {} is out of range 1..=200", len));
        }
        Ok(())
    } else {
        Err("Expected string".to_string())
    }
}

#[allow(dead_code)]
fn validate_string_pattern_6fd4a12c0ce64c08_len_1_64(value: &Value) -> Result<(), String> {
    if let Value::Concrete(ConcreteValue::String(s)) = value {
        static RE: std::sync::LazyLock<Regex> =
            std::sync::LazyLock::new(|| Regex::new("^[\\w\\-]+$").expect("invalid pattern regex"));
        if !RE.is_match(s) {
            return Err(format!("Value '{}' does not match pattern ^[\\w\\-]+$", s));
        }
        let len = s.chars().count();
        if !(1..=64).contains(&len) {
            return Err(format!("String length {} is out of range 1..=64", len));
        }
        Ok(())
    } else {
        Err("Expected string".to_string())
    }
}

#[allow(dead_code)]
fn validate_string_pattern_7dc332c889f363a0_len_1_253(value: &Value) -> Result<(), String> {
    if let Value::Concrete(ConcreteValue::String(s)) = value {
        static RE: std::sync::LazyLock<Regex> = std::sync::LazyLock::new(|| {
            Regex::new("^[\\w\\.\\-/]+$").expect("invalid pattern regex")
        });
        if !RE.is_match(s) {
            return Err(format!(
                "Value '{}' does not match pattern ^[\\w\\.\\-/]+$",
                s
            ));
        }
        let len = s.chars().count();
        if !(1..=253).contains(&len) {
            return Err(format!("String length {} is out of range 1..=253", len));
        }
        Ok(())
    } else {
        Err("Expected string".to_string())
    }
}

#[allow(dead_code)]
fn validate_asn_list_range(value: &Value) -> Result<(), String> {
    if let Value::Concrete(ConcreteValue::Int(n)) = value {
        if *n < 0 || *n > 4294967295 {
            Err(format!("Value {} is out of range 0..=4294967295", n))
        } else {
            Ok(())
        }
    } else {
        Err("Expected integer".to_string())
    }
}

#[allow(dead_code)]
fn validate_string_pattern_c77cf75cf1a75ade(value: &Value) -> Result<(), String> {
    if let Value::Concrete(ConcreteValue::String(s)) = value {
        static RE: std::sync::LazyLock<Regex> = std::sync::LazyLock::new(|| {
            Regex::new("^[\\/]+([^~]*(~[01])*)*{1,512}$").expect("invalid pattern regex")
        });
        if RE.is_match(s) {
            Ok(())
        } else {
            Err(format!(
                "Value '{}' does not match pattern ^[\\/]+([^~]*(~[01])*)*{{1,512}}$",
                s
            ))
        }
    } else {
        Err("Expected string".to_string())
    }
}

/// Returns the resource type name and all enum valid values for this module
pub fn enum_valid_values() -> (
    &'static str,
    &'static [(&'static str, &'static [&'static str])],
) {
    (
        "wafv2.WebAcl",
        &[
            (
                "sensitivity_to_block",
                VALID_AWS_MANAGED_RULES_ANTI_D_DO_S_RULE_SET_SENSITIVITY_TO_BLOCK,
            ),
            (
                "inspection_level",
                VALID_AWS_MANAGED_RULES_BOT_CONTROL_RULE_SET_INSPECTION_LEVEL,
            ),
            ("oversize_handling", VALID_BODY_OVERSIZE_HANDLING),
            (
                "positional_constraint",
                VALID_BYTE_MATCH_STATEMENT_POSITIONAL_CONSTRAINT,
            ),
            ("sensitivity", VALID_CLIENT_SIDE_ACTION_SENSITIVITY),
            ("usage_of_action", VALID_CLIENT_SIDE_ACTION_USAGE_OF_ACTION),
            ("match_scope", VALID_COOKIES_MATCH_SCOPE),
            ("oversize_handling", VALID_COOKIES_OVERSIZE_HANDLING),
            ("content_type", VALID_CUSTOM_RESPONSE_BODY_CONTENT_TYPE),
            ("action", VALID_DATA_PROTECT_ACTION),
            ("field_type", VALID_FIELD_TO_PROTECT_FIELD_TYPE),
            (
                "fallback_behavior",
                VALID_FORWARDED_IP_CONFIGURATION_FALLBACK_BEHAVIOR,
            ),
            ("oversize_handling", VALID_HEADER_ORDER_OVERSIZE_HANDLING),
            ("match_scope", VALID_HEADERS_MATCH_SCOPE),
            ("oversize_handling", VALID_HEADERS_OVERSIZE_HANDLING),
            (
                "fallback_behavior",
                VALID_IP_SET_FORWARDED_IP_CONFIGURATION_FALLBACK_BEHAVIOR,
            ),
            ("position", VALID_IP_SET_FORWARDED_IP_CONFIGURATION_POSITION),
            ("fallback_behavior", VALID_JA3_FINGERPRINT_FALLBACK_BEHAVIOR),
            ("fallback_behavior", VALID_JA4_FINGERPRINT_FALLBACK_BEHAVIOR),
            (
                "invalid_fallback_behavior",
                VALID_JSON_BODY_INVALID_FALLBACK_BEHAVIOR,
            ),
            ("match_scope", VALID_JSON_BODY_MATCH_SCOPE),
            ("oversize_handling", VALID_JSON_BODY_OVERSIZE_HANDLING),
            ("payload_type", VALID_MANAGED_RULE_GROUP_CONFIG_PAYLOAD_TYPE),
            (
                "alb_low_reputation_mode",
                VALID_ON_SOURCE_D_DO_S_PROTECTION_CONFIG_ALB_LOW_REPUTATION_MODE,
            ),
            (
                "aggregate_key_type",
                VALID_RATE_BASED_STATEMENT_AGGREGATE_KEY_TYPE,
            ),
            (
                "evaluation_window_sec",
                VALID_RATE_BASED_STATEMENT_EVALUATION_WINDOW_SEC,
            ),
            (
                "fallback_behavior",
                VALID_RATE_LIMIT_JA3_FINGERPRINT_FALLBACK_BEHAVIOR,
            ),
            (
                "fallback_behavior",
                VALID_RATE_LIMIT_JA4_FINGERPRINT_FALLBACK_BEHAVIOR,
            ),
            (
                "default_size_inspection_limit",
                VALID_REQUEST_BODY_ASSOCIATED_RESOURCE_TYPE_CONFIG_DEFAULT_SIZE_INSPECTION_LIMIT,
            ),
            ("payload_type", VALID_REQUEST_INSPECTION_PAYLOAD_TYPE),
            ("payload_type", VALID_REQUEST_INSPECTION_ACFP_PAYLOAD_TYPE),
            ("scope", VALID_SCOPE),
            (
                "comparison_operator",
                VALID_SIZE_CONSTRAINT_STATEMENT_COMPARISON_OPERATOR,
            ),
            (
                "sensitivity_level",
                VALID_SQLI_MATCH_STATEMENT_SENSITIVITY_LEVEL,
            ),
            ("type", VALID_TEXT_TRANSFORMATION_TYPE),
            ("fallback_behavior", VALID_URI_FRAGMENT_FALLBACK_BEHAVIOR),
        ],
    )
}
