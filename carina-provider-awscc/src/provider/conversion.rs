//! Value conversion between DSL and AWS JSON formats.
//!
//! This module handles bidirectional conversion between Carina's DSL `Value` types
//! and AWS CloudControl API JSON representations. It includes type-aware conversion
//! for enums, structs, lists, maps, and unions.

use indexmap::IndexMap;

use carina_core::resource::{ConcreteValue, Value};
use carina_core::schema::{AttributeType, ResourceSchema, Shape, ShapeWalkBudget, StructField};
use serde_json::json;

use carina_aws_types::canonicalize_enum_value;
use carina_core::utils::canonicalize_enum_to_api;

fn struct_fields_for<'a>(
    schema: &'a ResourceSchema,
    attr_type: &'a AttributeType,
) -> Option<&'a [StructField]> {
    schema.struct_fields_with_budget(attr_type, &mut ShapeWalkBudget::new(256))
}

fn union_members_for<'a>(
    schema: &'a ResourceSchema,
    attr_type: &'a AttributeType,
) -> Option<&'a [AttributeType]> {
    schema.union_members_with_budget(attr_type, &mut ShapeWalkBudget::new(256))
}

fn string_or_list_of_strings_from_json(
    schema: &ResourceSchema,
    members: &[AttributeType],
    value: &serde_json::Value,
) -> Option<Value> {
    let has_string = members
        .iter()
        .any(|member| matches!(schema.shape_of(member), Shape::String { .. }));
    let has_list_of_strings = members.iter().any(|member| {
        let Shape::List { element_type, .. } = schema.shape_of(member) else {
            return false;
        };
        matches!(schema.shape_of(element_type), Shape::String { .. })
    });

    if !has_string || !has_list_of_strings {
        return None;
    }

    if let Some(s) = value.as_str() {
        return Some(Value::Concrete(ConcreteValue::StringList(vec![
            s.to_string(),
        ])));
    }

    let arr = value.as_array()?;
    let items: Option<Vec<String>> = arr
        .iter()
        .map(|item| item.as_str().map(ToString::to_string))
        .collect();
    items.map(|items| Value::Concrete(ConcreteValue::StringList(items)))
}

fn json_matches_shape(
    schema: &ResourceSchema,
    attr_type: &AttributeType,
    value: &serde_json::Value,
) -> bool {
    match schema.shape_of(attr_type) {
        Shape::String { .. } | Shape::Enum { .. } => value.is_string(),
        Shape::Int { .. } => value.as_i64().is_some(),
        Shape::Float { .. } => value.is_number(),
        Shape::Bool => value.is_boolean(),
        Shape::Duration => value.is_string() || value.is_number(),
        Shape::List { element_type, .. } => value.as_array().is_some_and(|items| {
            items
                .iter()
                .all(|item| json_matches_shape(schema, element_type, item))
        }),
        Shape::Map { .. } | Shape::Struct { .. } => value.is_object(),
        Shape::Union => union_members_for(schema, attr_type).is_some_and(|members| {
            members
                .iter()
                .any(|member| json_matches_shape(schema, member, value))
        }),
    }
}

/// carina#3340: schema-aware variant that takes the enclosing
/// [`ResourceSchema::defs`] map so `AttributeType::Ref` chains can be
/// resolved during the value-tree walk. Callers that hold a resource
/// schema should prefer this entry point; the 4-arg shim above uses
/// an empty def map and will panic if a `Ref` is reached.
pub(crate) fn aws_value_to_dsl_with_defs(
    dsl_name: &str,
    value: &serde_json::Value,
    attr_type: &AttributeType,
    resource_type: &str,
    defs: &std::collections::BTreeMap<String, AttributeType>,
) -> Option<Value> {
    // Project onto the Shape view, which resolves any top-level `Ref`
    // chain against `defs` so the subsequent dispatch sees the actual
    // Struct / List / Map / ... underneath. `Shape` has no `Ref`
    // variant, so the wildcard fallthrough cannot silently swallow a
    // `Ref` (carina#3349).
    let mut schema = ResourceSchema::new(resource_type);
    schema.defs = defs.clone();
    let shape = schema.shape_of(attr_type);

    // This feeds the read/import path, whose result is written to
    // state. State must hold the provider-side (API) value, NOT a
    // fully-qualified DSL string: carina-core's state-lift reconciles
    // the API value back to an `EnumIdentifier` on the next plan, but a
    // fully-qualified DSL string matches neither `values` nor
    // `dsl_aliases`, so the lift would skip it and every subsequent
    // plan would report a spurious `~ change` (awscc#254).
    if let Shape::Enum {
        identity: _,
        values: Some(values),
        ..
    } = shape
        && let Some(s) = value.as_str()
    {
        let valid_values: Vec<&str> = values.iter().map(String::as_str).collect();
        let canonical = canonicalize_enum_value(s, &valid_values);
        return Some(Value::Concrete(ConcreteValue::String(canonical)));
    }
    if let Shape::Enum { values: None, .. } = shape
        && let Some(s) = value.as_str()
    {
        use crate::schemas::generated::get_enum_valid_values;
        let canonical = if let Some(valid_values) = get_enum_valid_values(resource_type, dsl_name) {
            canonicalize_enum_value(s, valid_values)
        } else {
            s.to_string()
        };
        return Some(Value::Concrete(ConcreteValue::String(canonical)));
    }

    // For List types, recurse into each item with the inner type for type-aware conversion
    if let Shape::List {
        element_type: inner,
        ..
    } = shape
        && let Some(arr) = value.as_array()
    {
        let items: Vec<Value> = arr
            .iter()
            .enumerate()
            .filter_map(|(i, item)| {
                let result = aws_value_to_dsl_with_defs(dsl_name, item, inner, resource_type, defs);
                if result.is_none() {
                    log::warn!(
                        "aws_value_to_dsl: dropping unconvertible array item at index {} for attribute '{}' in resource '{}': {:?}",
                        i, dsl_name, resource_type, item
                    );
                }
                result
            })
            .collect();
        return Some(Value::Concrete(ConcreteValue::List(items)));
    }

    // For Union types, try each member type and use the first that produces a type-aware result
    if let Shape::Union = shape
        && let Some(members) = union_members_for(&schema, attr_type)
    {
        if let Some(result) = string_or_list_of_strings_from_json(&schema, members, value) {
            return Some(result);
        }

        for member in members
            .iter()
            .filter(|member| json_matches_shape(&schema, member, value))
        {
            if let Some(result) =
                aws_value_to_dsl_with_defs(dsl_name, value, member, resource_type, defs)
            {
                return Some(result);
            }
        }
        return json_to_value(value);
    }

    // For bare Struct{fields}, recurse into fields
    if let Shape::Struct { .. } = shape
        && let Some(fields) = struct_fields_for(&schema, attr_type)
        && let Some(obj) = value.as_object()
    {
        let map: IndexMap<String, Value> = fields
            .iter()
            .filter_map(|field| {
                let provider_key = field.provider_name.as_deref().unwrap_or(&field.name);
                let json_val = obj.get(provider_key)?;
                let dsl_val = aws_value_to_dsl_with_defs(
                    &field.name,
                    json_val,
                    &field.field_type,
                    resource_type,
                    defs,
                );
                dsl_val.map(|v| (field.name.clone(), v))
            })
            .collect();
        if !map.is_empty() {
            return Some(Value::Concrete(ConcreteValue::Map(map)));
        }
    }

    // For Map types, recurse into values.
    // For IAM condition maps, convert PascalCase operator keys back to snake_case.
    if let Shape::Map { value: inner, .. } = shape
        && let Some(obj) = value.as_object()
    {
        let is_condition = dsl_name == "condition";
        let map: IndexMap<String, Value> = obj
            .iter()
            .filter_map(|(k, v)| {
                let result = aws_value_to_dsl_with_defs(dsl_name, v, inner, resource_type, defs);
                if result.is_none() {
                    log::warn!(
                        "aws_value_to_dsl: dropping unconvertible map entry '{}' for attribute '{}' in resource '{}': {:?}",
                        k, dsl_name, resource_type, v
                    );
                }
                let key = if is_condition {
                    carina_aws_types::condition_operator_to_snake(k).unwrap_or_else(|| k.clone())
                } else {
                    k.clone()
                };
                result.map(|val| (key, val))
            })
            .collect();
        return Some(Value::Concrete(ConcreteValue::Map(map)));
    }

    // For refined string types carrying a `to_dsl` normalization
    // transform, apply the transformation on read. This handles cases
    // like Route 53 DNS names where the API returns a normalized form
    // (trailing dot) that differs from user input. Post-#3230, the
    // enum-shorthand path lives on `CustomEnum.to_dsl`; `Custom.to_dsl`
    // is restricted to structural state→DSL normalization, so the
    // pre-#3222 `namespace: None` gate is no longer needed.
    if let Shape::String {
        to_dsl: Some(transform),
        ..
    } = shape
        && let Some(s) = value.as_str()
    {
        return Some(Value::Concrete(ConcreteValue::String(
            transform.apply(s).into_owned(),
        )));
    }

    json_to_value(value)
}

/// Convert JSON value to DSL Value
pub(crate) fn json_to_value(value: &serde_json::Value) -> Option<Value> {
    match value {
        serde_json::Value::String(s) => Some(Value::Concrete(ConcreteValue::String(s.clone()))),
        serde_json::Value::Bool(b) => Some(Value::Concrete(ConcreteValue::Bool(*b))),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Some(Value::Concrete(ConcreteValue::Int(i)))
            } else {
                n.as_f64().map(|f| Value::Concrete(ConcreteValue::Float(f)))
            }
        }
        serde_json::Value::Array(arr) => {
            let items: Vec<Value> = arr
                .iter()
                .enumerate()
                .filter_map(|(i, item)| {
                    let result = json_to_value(item);
                    if result.is_none() {
                        log::warn!(
                            "json_to_value: dropping unconvertible array item at index {}: {:?}",
                            i,
                            item
                        );
                    }
                    result
                })
                .collect();
            Some(Value::Concrete(ConcreteValue::List(items)))
        }
        serde_json::Value::Object(obj) => {
            let map: IndexMap<String, Value> = obj
                .iter()
                .filter_map(|(k, v)| {
                    let result = json_to_value(v);
                    if result.is_none() {
                        log::warn!(
                            "json_to_value: dropping unconvertible map entry '{}': {:?}",
                            k,
                            v
                        );
                    }
                    result.map(|val| (k.clone(), val))
                })
                .collect();
            Some(Value::Concrete(ConcreteValue::Map(map)))
        }
        _ => None,
    }
}

/// carina#3340: schema-aware variant — see [`aws_value_to_dsl_with_defs`]
/// for the rationale. Same `Ref` chains, same `&defs` parameter.
pub(crate) fn dsl_value_to_aws_with_defs(
    value: &Value,
    attr_type: &AttributeType,
    resource_type: &str,
    attr_name: &str,
    defs: &std::collections::BTreeMap<String, AttributeType>,
) -> Option<serde_json::Value> {
    let mut schema = ResourceSchema::new(resource_type);
    schema.defs = defs.clone();
    let shape = schema.shape_of(attr_type);
    // For schema-level string enums, convert namespaced DSL values back to provider values.
    // The gate is "has a populated namespace identity" on a schema enum.
    let is_namespaced_enum = matches!(shape, Shape::Enum { identity: _, .. });
    if is_namespaced_enum {
        match value {
            // Listed enum `String` values can arrive from read/state JSON
            // conversion or older saved state. Keep schema-aware
            // canonicalization for those closed enums. Validator-only enums
            // have no positional fallback after host canonicalization; pass
            // strings through unchanged so schema desync stays visible.
            Value::Concrete(ConcreteValue::String(s)) => {
                let resolved = if let Shape::Enum {
                    values: Some(values),
                    dsl_aliases,
                    ..
                } = shape
                {
                    let valid: Vec<&str> = values.iter().map(String::as_str).collect();
                    canonicalize_enum_to_api(
                        s,
                        &valid,
                        &carina_core::schema::DslMap::new(dsl_aliases, None),
                    )
                } else {
                    s.clone()
                };
                Some(json!(resolved))
            }
            Value::Concrete(ConcreteValue::EnumIdentifier(s)) => {
                let resolved = if let Shape::Enum {
                    values: Some(values),
                    dsl_aliases,
                    ..
                } = shape
                {
                    let valid: Vec<&str> = values.iter().map(String::as_str).collect();
                    canonicalize_enum_to_api(
                        s.as_str(),
                        &valid,
                        &carina_core::schema::DslMap::new(dsl_aliases, None),
                    )
                } else {
                    // Raw EnumIdentifier at a validator-only enum position
                    // means the host did not canonicalize it; pass it through
                    // so AWS-side rejection is visible instead of silently
                    // mis-splitting.
                    s.as_str().to_string()
                };
                Some(json!(resolved))
            }
            Value::Concrete(ConcreteValue::CanonicalEnum(c)) => Some(json!(c.api_value())),
            _ => value_to_json(value),
        }
    } else if let Shape::List {
        element_type: inner,
        ..
    } = shape
        && let Value::Concrete(ConcreteValue::List(items)) = value
    {
        // Recurse into list items with inner type for type-aware conversion
        let arr: Vec<serde_json::Value> = items
            .iter()
            .enumerate()
            .filter_map(|(i, item)| {
                let result = dsl_value_to_aws_with_defs(item, inner, resource_type, attr_name, defs);
                if result.is_none() {
                    log::warn!(
                        "dsl_value_to_aws: dropping unconvertible list item at index {} for attribute '{}' in resource '{}': {:?}",
                        i, attr_name, resource_type, item
                    );
                }
                result
            })
            .collect();
        Some(serde_json::Value::Array(arr))
    } else if let Shape::Union = shape
        && let Some(members) = union_members_for(&schema, attr_type)
    {
        // Try each member type; use the first that produces a type-aware result
        for member in members {
            if let Some(result) =
                dsl_value_to_aws_with_defs(value, member, resource_type, attr_name, defs)
            {
                return Some(result);
            }
        }
        value_to_json(value)
    } else if let Shape::Struct { .. } = shape
        && let Some(fields) = struct_fields_for(&schema, attr_type)
        && let Value::Concrete(ConcreteValue::Map(map)) = value
    {
        // Recurse into bare struct fields for type-aware conversion (map assignment syntax)
        let obj: serde_json::Map<String, serde_json::Value> = fields
            .iter()
            .filter_map(|field| {
                let dsl_val = map.get(&field.name)?;
                let provider_key = field
                    .provider_name
                    .as_deref()
                    .unwrap_or(&field.name)
                    .to_string();
                let json_val = dsl_value_to_aws_with_defs(
                    dsl_val,
                    &field.field_type,
                    resource_type,
                    &field.name,
                    defs,
                );
                json_val.map(|v| (provider_key, v))
            })
            .collect();
        Some(serde_json::Value::Object(obj))
    } else if let Shape::Struct { .. } = shape
        && let Some(fields) = struct_fields_for(&schema, attr_type)
        && let Value::Concrete(ConcreteValue::List(items)) = value
        && items.len() == 1
        && let Value::Concrete(ConcreteValue::Map(map)) = &items[0]
    {
        // Recurse into bare struct fields for type-aware conversion (block syntax)
        let obj: serde_json::Map<String, serde_json::Value> = fields
            .iter()
            .filter_map(|field| {
                let dsl_val = map.get(&field.name)?;
                let provider_key = field
                    .provider_name
                    .as_deref()
                    .unwrap_or(&field.name)
                    .to_string();
                let json_val = dsl_value_to_aws_with_defs(
                    dsl_val,
                    &field.field_type,
                    resource_type,
                    &field.name,
                    defs,
                );
                json_val.map(|v| (provider_key, v))
            })
            .collect();
        Some(serde_json::Value::Object(obj))
    } else if let Shape::Map { value: inner, .. } = shape
        && let Value::Concrete(ConcreteValue::Map(map)) = value
    {
        // Map type: recurse into values with inner type.
        // For IAM condition maps, convert snake_case operator keys to PascalCase.
        let is_condition = attr_name == "condition";
        let obj: serde_json::Map<String, serde_json::Value> = map
            .iter()
            .filter_map(|(k, v)| {
                let result = dsl_value_to_aws_with_defs(v, inner, resource_type, attr_name, defs);
                if result.is_none() {
                    log::warn!(
                        "dsl_value_to_aws: dropping unconvertible map entry '{}' for attribute '{}' in resource '{}': {:?}",
                        k, attr_name, resource_type, v
                    );
                }
                let key = if is_condition {
                    carina_aws_types::condition_operator_to_aws(k).unwrap_or_else(|| k.clone())
                } else {
                    k.clone()
                };
                result.map(|val| (key, val))
            })
            .collect();
        Some(serde_json::Value::Object(obj))
    } else {
        value_to_json(value)
    }
}

/// Convert DSL Value to JSON value
pub(crate) fn value_to_json(value: &Value) -> Option<serde_json::Value> {
    match value {
        Value::Concrete(ConcreteValue::String(s)) => Some(json!(s)),
        Value::Concrete(ConcreteValue::EnumIdentifier(s)) => Some(json!(s.as_str())),
        Value::Concrete(ConcreteValue::CanonicalEnum(c)) => Some(json!(c.api_value())),
        Value::Concrete(ConcreteValue::Bool(b)) => Some(json!(b)),
        Value::Concrete(ConcreteValue::Int(i)) => Some(json!(i)),
        Value::Concrete(ConcreteValue::Float(f)) if f.is_finite() => Some(json!(f)),
        Value::Concrete(ConcreteValue::Float(_)) => None,
        Value::Concrete(ConcreteValue::List(items)) => {
            let arr: Vec<serde_json::Value> = items
                .iter()
                .enumerate()
                .filter_map(|(i, item)| {
                    let result = value_to_json(item);
                    if result.is_none() {
                        log::warn!(
                            "value_to_json: dropping unconvertible list item at index {}: {:?}",
                            i,
                            item
                        );
                    }
                    result
                })
                .collect();
            Some(serde_json::Value::Array(arr))
        }
        Value::Concrete(ConcreteValue::Map(map)) => {
            let obj: serde_json::Map<String, serde_json::Value> = map
                .iter()
                .filter_map(|(k, v)| {
                    let result = value_to_json(v);
                    if result.is_none() {
                        log::warn!(
                            "value_to_json: dropping unconvertible map entry '{}': {:?}",
                            k,
                            v
                        );
                    }
                    result.map(|val| (k.clone(), val))
                })
                .collect();
            Some(serde_json::Value::Object(obj))
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use carina_core::schema::{DslTransform, StructField, legacy_validator};

    fn test_enum(
        name: &str,
        values: Vec<String>,
        identity: Option<carina_core::schema::TypeIdentity>,
        dsl_aliases: Vec<(String, String)>,
    ) -> AttributeType {
        AttributeType::enum_(
            identity.unwrap_or_else(|| carina_core::schema::TypeIdentity::bare(name)),
            Some(values),
            dsl_aliases,
            None,
            None,
        )
    }

    fn assert_canonical_enum(value: &Value, identity: &str, api_value: &str) {
        match value {
            Value::Concrete(ConcreteValue::CanonicalEnum(c)) => {
                assert_eq!(c.identity().to_string(), identity);
                assert_eq!(c.api_value(), api_value);
            }
            other => panic!("expected CanonicalEnum({identity}, {api_value}), got {other:?}"),
        }
    }

    fn assert_canonical_enum_ref(value: Option<&Value>, identity: &str, api_value: &str) {
        let value = value
            .unwrap_or_else(|| panic!("expected CanonicalEnum({identity}, {api_value}), got None"));
        assert_canonical_enum(value, identity, api_value);
    }

    #[test]
    fn test_aws_value_to_dsl_string_enum_returns_api_canonical_value() {
        let attr_type = test_enum(
            "ViewerProtocolPolicy",
            vec![
                "allow-all".to_string(),
                "redirect-to-https".to_string(),
                "https-only".to_string(),
            ],
            Some(carina_core::schema::enum_identity(
                "ViewerProtocolPolicy",
                Some("aws.cloudfront.Distribution"),
            )),
            vec![(
                "redirect-to-https".to_string(),
                "redirect_to_https".to_string(),
            )],
        );

        // CloudControl returns the API-canonical hyphen form.
        let from_api = aws_value_to_dsl_with_defs(
            "viewer_protocol_policy",
            &json!("redirect-to-https"),
            &attr_type,
            "cloudfront.Distribution",
            &std::collections::BTreeMap::new(),
        )
        .expect("read should succeed");
        assert_eq!(
            from_api,
            Value::Concrete(ConcreteValue::String("redirect-to-https".to_string())),
            "read must persist the API-canonical value, not a fully-qualified DSL string"
        );

        // If the API ever echoes the DSL-alias spelling, it must still be
        // canonicalized to the API form so state is stable.
        let from_alias = aws_value_to_dsl_with_defs(
            "viewer_protocol_policy",
            &json!("redirect_to_https"),
            &attr_type,
            "cloudfront.Distribution",
            &std::collections::BTreeMap::new(),
        )
        .expect("read should succeed");
        assert_eq!(
            from_alias,
            Value::Concrete(ConcreteValue::String("redirect-to-https".to_string())),
            "DSL-alias spelling from the API must canonicalize to the API value"
        );
    }

    // awscc#254 literal reproduction: read the real generated
    // `aws.cloudfront.Distribution` `distribution_config` from a
    // CloudControl-shaped response (PascalCase keys, raw API enum
    // values) and assert the nested enum leaves are persisted as the
    // API-canonical value, then lift to the canonical fully-qualified identifier
    // the differ reconciles against.
    #[test]
    fn test_cloudfront_distribution_config_read_is_api_canonical() {
        let config =
            crate::schemas::generated::cloudfront::distribution::cloudfront_distribution_config();
        let attr_schema = config.schema.attributes.get("distribution_config").unwrap();

        let aws_json = json!({
            "DefaultCacheBehavior": {
                "TargetOriginId": "origin-1",
                "ViewerProtocolPolicy": "redirect-to-https"
            },
            "PriceClass": "PriceClass_200"
        });

        let dsl = aws_value_to_dsl_with_defs(
            "distribution_config",
            &aws_json,
            &attr_schema.attr_type,
            "cloudfront.Distribution",
            &config.schema.defs,
        )
        .expect("aws_value_to_dsl should succeed for distribution_config");

        let Value::Concrete(ConcreteValue::Map(top)) = &dsl else {
            panic!("expected distribution_config to be a Map, got {dsl:?}");
        };
        assert_eq!(
            top.get("price_class"),
            Some(&Value::Concrete(ConcreteValue::String(
                "PriceClass_200".to_string()
            ))),
            "price_class must persist the API-canonical value"
        );
        let Some(Value::Concrete(ConcreteValue::Map(dcb))) = top.get("default_cache_behavior")
        else {
            panic!("expected default_cache_behavior map");
        };
        assert_eq!(
            dcb.get("viewer_protocol_policy"),
            Some(&Value::Concrete(ConcreteValue::String(
                "redirect-to-https".to_string()
            ))),
            "nested viewer_protocol_policy must persist the API-canonical value, \
             not a fully-qualified DSL string"
        );

        // carina-core's state-lift reduces the persisted API values to
        // the canonical fully-qualified identifiers the differ reconciles against
        // the parsed-desired side — closing the awscc#254 spurious diff.
        let lifted = carina_core::utils::lift_enum_leaves_with_defs(
            &dsl,
            &attr_schema.attr_type,
            &config.schema.defs,
        )
        .expect("API-canonical distribution_config must lift");
        let Value::Concrete(ConcreteValue::Map(lifted_top)) = &lifted else {
            panic!("expected lifted distribution_config to be a Map, got {lifted:?}");
        };
        assert_canonical_enum_ref(
            lifted_top.get("price_class"),
            "aws.cloudfront.Distribution.DistributionConfig.PriceClass",
            "PriceClass_200",
        );
        let Some(Value::Concrete(ConcreteValue::Map(lifted_dcb))) =
            lifted_top.get("default_cache_behavior")
        else {
            panic!("expected lifted default_cache_behavior map");
        };
        assert_canonical_enum_ref(
            lifted_dcb.get("viewer_protocol_policy"),
            "aws.cloudfront.Distribution.DistributionConfig.DefaultCacheBehavior.ViewerProtocolPolicy",
            "redirect-to-https",
        );
    }

    #[test]
    fn test_aws_value_to_dsl_bare_struct_returns_map() {
        let fields = vec![
            StructField::new("status", AttributeType::string()).with_provider_name("Status"),
            StructField::new("mfa_delete", AttributeType::string()).with_provider_name("MfaDelete"),
        ];
        let attr_type = AttributeType::struct_("VersioningConfiguration".to_string(), fields);
        let json_val = serde_json::json!({
            "Status": "Enabled",
        });

        let result = aws_value_to_dsl_with_defs(
            "versioning_configuration",
            &json_val,
            &attr_type,
            "AWS::S3::Bucket",
            &std::collections::BTreeMap::new(),
        );
        let result = result.expect("Should return Some");

        // Must be Value::Concrete(ConcreteValue::Map(...)) to match parser output for map assignment syntax
        if let Value::Concrete(ConcreteValue::Map(map)) = &result {
            assert_eq!(
                map.get("status"),
                Some(&Value::Concrete(ConcreteValue::String(
                    "Enabled".to_string()
                )))
            );
        } else {
            panic!("Expected Value::Map, got: {:?}", result);
        }
    }

    #[test]
    fn test_dsl_value_to_aws_map_for_bare_struct() {
        let fields = vec![
            StructField::new("status", AttributeType::string()).with_provider_name("Status"),
            StructField::new("mfa_delete", AttributeType::string()).with_provider_name("MfaDelete"),
        ];
        let attr_type = AttributeType::struct_("VersioningConfiguration".to_string(), fields);

        // Parser produces Value::Concrete(ConcreteValue::Map(...)) for map assignment syntax (= { ... })
        let mut map = IndexMap::new();
        map.insert(
            "status".to_string(),
            Value::Concrete(ConcreteValue::String("Enabled".to_string())),
        );
        let dsl_value = Value::Concrete(ConcreteValue::Map(map));

        let result = dsl_value_to_aws_with_defs(
            &dsl_value,
            &attr_type,
            "AWS::S3::Bucket",
            "versioning_configuration",
            &std::collections::BTreeMap::new(),
        );
        let result = result.expect("Should return Some");

        // Must produce a JSON object (not array)
        if let serde_json::Value::Object(obj) = &result {
            assert_eq!(obj.get("Status"), Some(&serde_json::json!("Enabled")));
        } else {
            panic!("Expected JSON Object, got: {:?}", result);
        }
    }

    #[test]
    fn test_dsl_value_to_aws_list_for_bare_struct() {
        let fields = vec![
            StructField::new("status", AttributeType::string()).with_provider_name("Status"),
            StructField::new("mfa_delete", AttributeType::string()).with_provider_name("MfaDelete"),
        ];
        let attr_type = AttributeType::struct_("VersioningConfiguration".to_string(), fields);

        // Parser produces Value::Concrete(ConcreteValue::List(vec![Value::Concrete(ConcreteValue::Map(...))])) for block syntax (name { ... })
        let mut map = IndexMap::new();
        map.insert(
            "status".to_string(),
            Value::Concrete(ConcreteValue::String("Enabled".to_string())),
        );
        let dsl_value = Value::Concrete(ConcreteValue::List(vec![Value::Concrete(
            ConcreteValue::Map(map),
        )]));

        let result = dsl_value_to_aws_with_defs(
            &dsl_value,
            &attr_type,
            "AWS::S3::Bucket",
            "versioning_configuration",
            &std::collections::BTreeMap::new(),
        );
        let result = result.expect("Should return Some");

        // Must produce a JSON object (not array)
        if let serde_json::Value::Object(obj) = &result {
            assert_eq!(obj.get("Status"), Some(&serde_json::json!("Enabled")));
        } else {
            panic!("Expected JSON Object, got: {:?}", result);
        }
    }

    #[test]
    fn test_bare_struct_roundtrip_no_spurious_diff() {
        let fields =
            vec![StructField::new("status", AttributeType::string()).with_provider_name("Status")];
        let attr_type = AttributeType::struct_("VersioningConfiguration".to_string(), fields);

        // Simulate AWS API response (JSON object)
        let aws_json = serde_json::json!({ "Status": "Enabled" });

        // Read path: convert AWS JSON to DSL value
        let dsl_value = aws_value_to_dsl_with_defs(
            "versioning_configuration",
            &aws_json,
            &attr_type,
            "AWS::S3::Bucket",
            &std::collections::BTreeMap::new(),
        )
        .expect("read should succeed");

        // Simulate parser output (what the user wrote in .crn with map assignment syntax)
        let mut parser_map = IndexMap::new();
        parser_map.insert(
            "status".to_string(),
            Value::Concrete(ConcreteValue::String("Enabled".to_string())),
        );
        let parser_value = Value::Concrete(ConcreteValue::Map(parser_map));

        // The read value and parser value must be equal (no spurious diff)
        assert_eq!(
            dsl_value, parser_value,
            "Read value should match parser value — no spurious diff"
        );

        // Write path: convert DSL value back to AWS JSON
        let written_json = dsl_value_to_aws_with_defs(
            &dsl_value,
            &attr_type,
            "AWS::S3::Bucket",
            "versioning_configuration",
            &std::collections::BTreeMap::new(),
        )
        .expect("write should succeed");

        assert_eq!(
            written_json, aws_json,
            "Round-trip should produce original AWS JSON"
        );
    }

    #[test]
    fn test_vpc_endpoint_type_roundtrip_no_false_diff() {
        let config = crate::schemas::generated::ec2::vpc_endpoint::ec2_vpc_endpoint_config();
        let attr_schema = config.schema.attributes.get("vpc_endpoint_type").unwrap();

        // AWS read-back side.
        let aws_json = serde_json::json!("Gateway");
        let aws_dsl = aws_value_to_dsl_with_defs(
            "vpc_endpoint_type",
            &aws_json,
            &attr_schema.attr_type,
            "ec2.VpcEndpoint",
            &std::collections::BTreeMap::new(),
        )
        .expect("aws_value_to_dsl should return Some");

        assert_eq!(
            aws_dsl,
            Value::Concrete(ConcreteValue::String("Gateway".to_string())),
            "AWS read-back must persist the API-canonical value"
        );

        // No false diff: reconciliation happens in carina-core
        // (state-lift + differ), not by the provider emitting identical
        // strings on both sides. Assert the awscc-owned half — the
        // persisted API value lifts to the canonical fully-qualified identifier.
        let lifted = carina_core::utils::lift_enum_leaves(&aws_dsl, &attr_schema.attr_type)
            .expect("API-canonical state value must lift to an EnumIdentifier");
        assert_canonical_enum(&lifted, "aws.ec2.VpcEndpoint.VpcEndpointType", "Gateway");
    }

    #[test]
    fn test_dsl_value_to_aws_preserves_underscores_in_enum_values() {
        let attr_type = test_enum(
            "LogGroupClass",
            vec![
                "STANDARD".to_string(),
                "INFREQUENT_ACCESS".to_string(),
                "DELIVERY".to_string(),
            ],
            Some(carina_core::schema::enum_identity(
                "LogGroupClass",
                Some("aws.logs.LogGroup"),
            )),
            vec![],
        );
        let value = Value::Concrete(ConcreteValue::String(
            "aws.logs.LogGroup.LogGroupClass.INFREQUENT_ACCESS".to_string(),
        ));
        let result = dsl_value_to_aws_with_defs(
            &value,
            &attr_type,
            "logs.LogGroup",
            "log_group_class",
            &std::collections::BTreeMap::new(),
        );

        assert_eq!(result, Some(json!("INFREQUENT_ACCESS")));
    }

    #[test]
    fn test_dsl_value_to_aws_passes_through_validator_only_string() {
        let attr_type = AttributeType::enum_(
            carina_aws_types::provider_bare_type(&[], "Region"),
            None,
            vec![],
            Some(legacy_validator(|_| Ok(()))),
            None,
        );
        let value = Value::Concrete(ConcreteValue::String(
            "aws.Region.ap_northeast_1".to_string(),
        ));
        let result = dsl_value_to_aws_with_defs(
            &value,
            &attr_type,
            "logs.LogGroup",
            "region",
            &std::collections::BTreeMap::new(),
        );

        assert_eq!(result, Some(json!("aws.Region.ap_northeast_1")));
    }

    #[test]
    fn test_dsl_value_to_aws_raw_enum_identifier_passes_through_verbatim() {
        let attr_type = AttributeType::enum_(
            carina_aws_types::provider_bare_type(&[], "Region"),
            None,
            vec![],
            Some(legacy_validator(|_| Ok(()))),
            None,
        );
        let raw = "aws.Region.ap_northeast_1";
        let value = Value::Concrete(ConcreteValue::enum_identifier(raw));
        let result = dsl_value_to_aws_with_defs(
            &value,
            &attr_type,
            "logs.LogGroup",
            "region",
            &std::collections::BTreeMap::new(),
        );

        assert_eq!(result, Some(json!(raw)));
    }

    #[test]
    fn test_dsl_value_to_aws_list_string_enum() {
        let inner = test_enum(
            "AllowedMethod",
            vec!["GET".to_string(), "PUT".to_string(), "DELETE".to_string()],
            Some(carina_core::schema::enum_identity(
                "AllowedMethod",
                Some("aws.s3.Bucket"),
            )),
            vec![],
        );
        let attr_type = AttributeType::list(inner);
        let value = Value::Concrete(ConcreteValue::List(vec![
            Value::Concrete(ConcreteValue::String(
                "aws.s3.Bucket.AllowedMethod.GET".to_string(),
            )),
            Value::Concrete(ConcreteValue::String(
                "aws.s3.Bucket.AllowedMethod.PUT".to_string(),
            )),
        ]));
        let result = dsl_value_to_aws_with_defs(
            &value,
            &attr_type,
            "s3.Bucket",
            "allowed_methods",
            &std::collections::BTreeMap::new(),
        );

        assert_eq!(result, Some(json!(["GET", "PUT"])));
    }

    #[test]
    fn test_aws_value_to_dsl_list_string_enum() {
        let inner = test_enum(
            "AllowedMethod",
            vec!["GET".to_string(), "PUT".to_string(), "DELETE".to_string()],
            Some(carina_core::schema::enum_identity(
                "AllowedMethod",
                Some("aws.s3.Bucket"),
            )),
            vec![],
        );
        let attr_type = AttributeType::list(inner);
        let json_val = json!(["GET", "PUT"]);
        let result = aws_value_to_dsl_with_defs(
            "allowed_methods",
            &json_val,
            &attr_type,
            "s3.Bucket",
            &std::collections::BTreeMap::new(),
        );

        assert_eq!(
            result,
            Some(Value::Concrete(ConcreteValue::List(vec![
                Value::Concrete(ConcreteValue::String("GET".to_string())),
                Value::Concrete(ConcreteValue::String("PUT".to_string())),
            ])))
        );
    }

    #[test]
    fn test_dsl_value_to_aws_list_string_enum_roundtrip() {
        let inner = test_enum(
            "AllowedMethod",
            vec!["GET".to_string(), "PUT".to_string()],
            Some(carina_core::schema::enum_identity(
                "AllowedMethod",
                Some("aws.s3.Bucket"),
            )),
            vec![],
        );
        let attr_type = AttributeType::list(inner);

        let aws_json = json!(["GET", "PUT"]);
        let dsl = aws_value_to_dsl_with_defs(
            "allowed_methods",
            &aws_json,
            &attr_type,
            "s3.Bucket",
            &std::collections::BTreeMap::new(),
        )
        .expect("read should succeed");
        let written = dsl_value_to_aws_with_defs(
            &dsl,
            &attr_type,
            "s3.Bucket",
            "allowed_methods",
            &std::collections::BTreeMap::new(),
        )
        .expect("write should succeed");
        assert_eq!(written, aws_json, "Round-trip should produce original JSON");
    }

    #[test]
    fn test_dsl_value_to_aws_union_with_string_enum() {
        let attr_type = AttributeType::union(vec![
            test_enum(
                "Protocol",
                vec!["tcp".to_string(), "udp".to_string()],
                Some(carina_core::schema::enum_identity(
                    "Protocol",
                    Some("aws.ec2.Sg"),
                )),
                vec![],
            ),
            AttributeType::string(),
        ]);
        let value = Value::Concrete(ConcreteValue::String("aws.ec2.Sg.Protocol.tcp".to_string()));
        let result = dsl_value_to_aws_with_defs(
            &value,
            &attr_type,
            "ec2.Sg",
            "protocol",
            &std::collections::BTreeMap::new(),
        );

        assert_eq!(result, Some(json!("tcp")));
    }

    #[test]
    fn test_dsl_value_to_aws_map_preserves_user_keys() {
        let attr_type = AttributeType::map(AttributeType::string());

        let mut map = IndexMap::new();
        map.insert(
            "my_custom_key".to_string(),
            Value::Concrete(ConcreteValue::String("value1".to_string())),
        );
        map.insert(
            "another-key".to_string(),
            Value::Concrete(ConcreteValue::String("value2".to_string())),
        );
        let dsl_value = Value::Concrete(ConcreteValue::Map(map));

        let result = dsl_value_to_aws_with_defs(
            &dsl_value,
            &attr_type,
            "s3.Bucket",
            "tags",
            &std::collections::BTreeMap::new(),
        );

        let result = result.expect("Should return Some");

        if let serde_json::Value::Object(obj) = &result {
            assert_eq!(obj.get("my_custom_key"), Some(&json!("value1")));
            assert_eq!(obj.get("another-key"), Some(&json!("value2")));
            assert!(obj.get("MyCustomKey").is_none());
            assert!(obj.get("AnotherKey").is_none());
        } else {
            panic!("Expected JSON Object, got: {:?}", result);
        }
    }

    #[test]
    fn test_dsl_value_to_aws_map_recurses_into_values() {
        let inner_type = test_enum(
            "Status",
            vec!["Active".to_string(), "Inactive".to_string()],
            Some(carina_core::schema::enum_identity(
                "Status",
                Some("aws.test.resource"),
            )),
            vec![],
        );
        let attr_type = AttributeType::map(inner_type);

        let mut map = IndexMap::new();
        map.insert(
            "item_one".to_string(),
            Value::Concrete(ConcreteValue::String(
                "aws.test.resource.Status.Active".to_string(),
            )),
        );
        let dsl_value = Value::Concrete(ConcreteValue::Map(map));

        let result = dsl_value_to_aws_with_defs(
            &dsl_value,
            &attr_type,
            "test.resource",
            "status_map",
            &std::collections::BTreeMap::new(),
        );

        let result = result.expect("Should return Some");

        if let serde_json::Value::Object(obj) = &result {
            assert_eq!(obj.get("item_one"), Some(&json!("Active")));
        } else {
            panic!("Expected JSON Object, got: {:?}", result);
        }
    }

    #[test]
    fn test_aws_value_to_dsl_map_preserves_user_keys() {
        let attr_type = AttributeType::map(AttributeType::string());

        let aws_json = json!({
            "MyCustomKey": "value1",
            "another-key": "value2"
        });

        let result = aws_value_to_dsl_with_defs(
            "tags",
            &aws_json,
            &attr_type,
            "s3.Bucket",
            &std::collections::BTreeMap::new(),
        );

        let result = result.expect("Should return Some");

        if let Value::Concrete(ConcreteValue::Map(map)) = &result {
            assert_eq!(
                map.get("MyCustomKey"),
                Some(&Value::Concrete(ConcreteValue::String(
                    "value1".to_string()
                )))
            );
            assert_eq!(
                map.get("another-key"),
                Some(&Value::Concrete(ConcreteValue::String(
                    "value2".to_string()
                )))
            );
            assert!(map.get("my_custom_key").is_none());
        } else {
            panic!("Expected Value::Map, got: {:?}", result);
        }
    }

    #[test]
    fn test_aws_value_to_dsl_union_with_string_enum() {
        let attr_type = AttributeType::union(vec![
            test_enum(
                "Protocol",
                vec!["tcp".to_string(), "udp".to_string()],
                Some(carina_core::schema::enum_identity(
                    "Protocol",
                    Some("aws.ec2.Sg"),
                )),
                vec![],
            ),
            AttributeType::string(),
        ]);
        let json_val = json!("tcp");
        let result = aws_value_to_dsl_with_defs(
            "protocol",
            &json_val,
            &attr_type,
            "ec2.Sg",
            &std::collections::BTreeMap::new(),
        );

        assert_eq!(
            result,
            Some(Value::Concrete(ConcreteValue::String("tcp".to_string())))
        );
    }

    #[test]
    fn test_aws_value_to_dsl_union_fallback() {
        let attr_type = AttributeType::union(vec![
            test_enum(
                "Protocol",
                vec!["tcp".to_string(), "udp".to_string()],
                Some(carina_core::schema::enum_identity(
                    "Protocol",
                    Some("aws.ec2.Sg"),
                )),
                vec![],
            ),
            AttributeType::int(),
        ]);
        let json_val = json!(42);
        let result = aws_value_to_dsl_with_defs(
            "protocol",
            &json_val,
            &attr_type,
            "ec2.Sg",
            &std::collections::BTreeMap::new(),
        );

        assert_eq!(result, Some(Value::Concrete(ConcreteValue::Int(42))));
    }

    #[test]
    fn test_dsl_value_to_aws_iam_policy_document_uses_pascal_case() {
        use carina_aws_types::iam_policy_document;

        let attr_type = iam_policy_document();
        let value = Value::Concrete(ConcreteValue::Map(
            vec![
                (
                    "version".to_string(),
                    Value::Concrete(ConcreteValue::String("2012-10-17".to_string())),
                ),
                (
                    "statement".to_string(),
                    Value::Concrete(ConcreteValue::List(vec![Value::Concrete(
                        ConcreteValue::Map(
                            vec![
                                (
                                    "effect".to_string(),
                                    Value::Concrete(ConcreteValue::String("Allow".to_string())),
                                ),
                                (
                                    "action".to_string(),
                                    Value::Concrete(ConcreteValue::String(
                                        "sts:AssumeRole".to_string(),
                                    )),
                                ),
                                (
                                    "principal".to_string(),
                                    Value::Concrete(ConcreteValue::Map(
                                        vec![(
                                            "service".to_string(),
                                            Value::Concrete(ConcreteValue::String(
                                                "lambda.amazonaws.com".to_string(),
                                            )),
                                        )]
                                        .into_iter()
                                        .collect(),
                                    )),
                                ),
                            ]
                            .into_iter()
                            .collect(),
                        ),
                    )])),
                ),
            ]
            .into_iter()
            .collect(),
        ));

        let result = dsl_value_to_aws_with_defs(
            &value,
            &attr_type,
            "iam.Role",
            "assume_role_policy_document",
            &std::collections::BTreeMap::new(),
        );
        let result = result.expect("Should return Some");

        let obj = result.as_object().expect("Expected JSON Object");
        assert_eq!(obj.get("Version"), Some(&json!("2012-10-17")));
        assert!(
            obj.get("version").is_none(),
            "snake_case 'version' should not exist"
        );

        let statements = obj.get("Statement").expect("Should have Statement");
        assert!(
            obj.get("statement").is_none(),
            "snake_case 'statement' should not exist"
        );
        let stmt = statements.as_array().unwrap().first().unwrap();
        let stmt_obj = stmt.as_object().unwrap();

        assert_eq!(stmt_obj.get("Effect"), Some(&json!("Allow")));
        assert!(stmt_obj.get("effect").is_none());
        assert_eq!(stmt_obj.get("Action"), Some(&json!("sts:AssumeRole")));
        assert!(stmt_obj.get("action").is_none());

        let principal = stmt_obj.get("Principal").expect("Should have Principal");
        assert!(stmt_obj.get("principal").is_none());
        let principal_obj = principal.as_object().unwrap();
        assert_eq!(
            principal_obj.get("Service"),
            Some(&json!("lambda.amazonaws.com"))
        );
        assert!(principal_obj.get("service").is_none());
    }

    /// Regression for aws#315 (cross-checked on the awscc side): the
    /// aws#315 root cause was the aws normalizer's StringEnum leaf guard
    /// matching only `String`, silently skipping the `EnumIdentifier`
    /// shape that nested IAM policy `version` / `effect` arrive in after
    /// the carina#3055 state-lift. This test feeds the awscc serializer
    /// the **same `EnumIdentifier` shape** (namespaced for `version`, bare
    /// alias for `effect`) to prove awscc has no parallel gap:
    /// `dsl_value_to_aws` accepts both `String | EnumIdentifier` at the
    /// StringEnum branch and resolves via `canonicalize_enum_to_api`, so
    /// the Cloud Control `desired_state` payload still gets the AWS wire
    /// form (`"2012-10-17"`, `"Allow"`).
    #[test]
    fn test_dsl_value_to_aws_iam_policy_document_canonicalizes_namespaced_and_alias_enums() {
        use carina_aws_types::iam_policy_document;

        let attr_type = iam_policy_document();
        let value = Value::Concrete(ConcreteValue::Map(
            vec![
                (
                    "version".to_string(),
                    Value::Concrete(ConcreteValue::enum_identifier(
                        "aws.iam.PolicyDocument.Version.2012_10_17",
                    )),
                ),
                (
                    "statement".to_string(),
                    Value::Concrete(ConcreteValue::List(vec![Value::Concrete(
                        ConcreteValue::Map(
                            vec![
                                (
                                    "effect".to_string(),
                                    Value::Concrete(ConcreteValue::enum_identifier("allow")),
                                ),
                                (
                                    "action".to_string(),
                                    Value::Concrete(ConcreteValue::String(
                                        "s3:GetObject".to_string(),
                                    )),
                                ),
                            ]
                            .into_iter()
                            .collect(),
                        ),
                    )])),
                ),
            ]
            .into_iter()
            .collect(),
        ));

        let result = dsl_value_to_aws_with_defs(
            &value,
            &attr_type,
            "iam.RolePolicy",
            "policy_document",
            &std::collections::BTreeMap::new(),
        )
        .expect("Should return Some");
        let obj = result.as_object().expect("Expected JSON Object");
        assert_eq!(
            obj.get("Version"),
            Some(&json!("2012-10-17")),
            "version must be AWS-canonical, got: {result}"
        );
        let stmt = obj
            .get("Statement")
            .and_then(|s| s.as_array())
            .and_then(|a| a.first())
            .and_then(|s| s.as_object())
            .expect("Statement[0] object");
        assert_eq!(
            stmt.get("Effect"),
            Some(&json!("Allow")),
            "effect must be AWS-canonical, got: {result}"
        );
        let serialized = serde_json::to_string(&result).unwrap();
        assert!(
            !serialized.contains("2012_10_17") && !serialized.contains(r#""allow""#),
            "DSL spelling must not reach Cloud Control, got: {serialized}"
        );
    }

    #[test]
    fn test_aws_value_to_dsl_iam_policy_document_uses_snake_case() {
        use carina_aws_types::iam_policy_document;

        let attr_type = iam_policy_document();
        let aws_json = json!({
            "Version": "2012-10-17",
            "Statement": [{
                "Effect": "Allow",
                "Action": "sts:AssumeRole",
                "Principal": {
                    "Service": "lambda.amazonaws.com"
                }
            }]
        });

        let result = aws_value_to_dsl_with_defs(
            "assume_role_policy_document",
            &aws_json,
            &attr_type,
            "iam.Role",
            &std::collections::BTreeMap::new(),
        );
        let result = result.expect("Should return Some");

        if let Value::Concrete(ConcreteValue::Map(map)) = &result {
            // Values are persisted API-canonical; keys stay snake_case.
            assert_eq!(
                map.get("version"),
                Some(&Value::Concrete(ConcreteValue::String(
                    "2012-10-17".to_string()
                )))
            );
            assert!(
                map.get("Version").is_none(),
                "PascalCase 'Version' should not exist"
            );

            if let Some(Value::Concrete(ConcreteValue::List(stmts))) = map.get("statement") {
                if let Some(Value::Concrete(ConcreteValue::Map(stmt))) = stmts.first() {
                    assert_eq!(
                        stmt.get("effect"),
                        Some(&Value::Concrete(ConcreteValue::String("Allow".to_string())))
                    );
                    assert_eq!(
                        stmt.get("action"),
                        Some(&Value::Concrete(ConcreteValue::StringList(vec![
                            "sts:AssumeRole".to_string()
                        ])))
                    );
                    if let Some(Value::Concrete(ConcreteValue::Map(principal))) =
                        stmt.get("principal")
                    {
                        assert_eq!(
                            principal.get("service"),
                            Some(&Value::Concrete(ConcreteValue::StringList(vec![
                                "lambda.amazonaws.com".to_string()
                            ])))
                        );
                    } else {
                        panic!("Expected principal to be a Map");
                    }
                } else {
                    panic!("Expected statement to contain a Map");
                }
            } else {
                panic!("Expected statement to be a List");
            }
        } else {
            panic!("Expected Value::Map, got: {:?}", result);
        }
    }

    #[test]
    fn test_aws_value_to_dsl_iam_policy_document_canonicalizes_scalar_action_to_string_list() {
        use carina_aws_types::iam_policy_document;

        let attr_type = iam_policy_document();
        let aws_json = json!({
            "Version": "2012-10-17",
            "Statement": [{
                "Effect": "Allow",
                "Action": "s3:GetObject",
                "Resource": "arn:aws:s3:::*"
            }]
        });

        let result = aws_value_to_dsl_with_defs(
            "policy_document",
            &aws_json,
            &attr_type,
            "ec2.VpcEndpoint",
            &std::collections::BTreeMap::new(),
        )
        .expect("policy document should convert");
        let Value::Concrete(ConcreteValue::Map(policy_document)) = result else {
            panic!("Expected policy document map");
        };
        let Some(Value::Concrete(ConcreteValue::List(statements))) =
            policy_document.get("statement")
        else {
            panic!("Expected statement list");
        };
        let Some(Value::Concrete(ConcreteValue::Map(statement))) = statements.first() else {
            panic!("Expected statement map");
        };

        assert_eq!(
            statement.get("action"),
            Some(&Value::Concrete(ConcreteValue::StringList(vec![
                "s3:GetObject".to_string()
            ])))
        );
        assert_eq!(
            statement.get("resource"),
            Some(&Value::Concrete(ConcreteValue::StringList(vec![
                "arn:aws:s3:::*".to_string()
            ])))
        );
    }

    #[test]
    fn test_aws_value_to_dsl_iam_policy_document_canonicalizes_list_action_to_string_list() {
        use carina_aws_types::iam_policy_document;

        let attr_type = iam_policy_document();
        let aws_json = json!({
            "Version": "2012-10-17",
            "Statement": [{
                "Effect": "Allow",
                "Action": ["s3:GetObject", "s3:PutObject"],
                "Resource": ["arn:aws:s3:::*", "arn:aws:s3:::example/*"]
            }]
        });

        let result = aws_value_to_dsl_with_defs(
            "policy_document",
            &aws_json,
            &attr_type,
            "ec2.VpcEndpoint",
            &std::collections::BTreeMap::new(),
        )
        .expect("policy document should convert");
        let Value::Concrete(ConcreteValue::Map(policy_document)) = result else {
            panic!("Expected policy document map");
        };
        let Some(Value::Concrete(ConcreteValue::List(statements))) =
            policy_document.get("statement")
        else {
            panic!("Expected statement list");
        };
        let Some(Value::Concrete(ConcreteValue::Map(statement))) = statements.first() else {
            panic!("Expected statement map");
        };

        assert_eq!(
            statement.get("action"),
            Some(&Value::Concrete(ConcreteValue::StringList(vec![
                "s3:GetObject".to_string(),
                "s3:PutObject".to_string()
            ])))
        );
        assert_eq!(
            statement.get("resource"),
            Some(&Value::Concrete(ConcreteValue::StringList(vec![
                "arn:aws:s3:::*".to_string(),
                "arn:aws:s3:::example/*".to_string()
            ])))
        );
    }

    /// Regression for aws#313 + awscc#254. aws#313 made the IAM policy
    /// doc's `version`/`effect` a namespaced `StringEnum`; at the time
    /// the read path emitted the fully-qualified DSL form. awscc#254
    /// flipped that: the read path persists the *API-canonical* value
    /// (`"2012-10-17"`, `"Allow"`) since that is what is written to
    /// state, and carina-core's state-lift reduces it to the canonical
    /// short `EnumIdentifier`. This pins the awscc-owned half of that
    /// contract.
    #[test]
    fn test_aws313_iam_policy_doc_read_is_api_canonical_and_lifts() {
        use carina_aws_types::iam_policy_document;

        let attr_type = iam_policy_document();

        let aws_json = json!({
            "Version": "2012-10-17",
            "Statement": [{
                "Effect": "Allow",
                "Action": "sts:AssumeRole"
            }]
        });
        let read_side = aws_value_to_dsl_with_defs(
            "assume_role_policy_document",
            &aws_json,
            &attr_type,
            "iam.Role",
            &std::collections::BTreeMap::new(),
        )
        .expect("read conversion should return Some");

        let api_canonical = Value::Concrete(ConcreteValue::Map(
            vec![
                (
                    "version".to_string(),
                    Value::Concrete(ConcreteValue::String("2012-10-17".to_string())),
                ),
                (
                    "statement".to_string(),
                    Value::Concrete(ConcreteValue::List(vec![Value::Concrete(
                        ConcreteValue::Map(
                            vec![
                                (
                                    "effect".to_string(),
                                    Value::Concrete(ConcreteValue::String("Allow".to_string())),
                                ),
                                (
                                    "action".to_string(),
                                    Value::Concrete(ConcreteValue::StringList(vec![
                                        "sts:AssumeRole".to_string(),
                                    ])),
                                ),
                            ]
                            .into_iter()
                            .collect(),
                        ),
                    )])),
                ),
            ]
            .into_iter()
            .collect(),
        ));

        assert_eq!(
            read_side, api_canonical,
            "read-side IAM policy doc must persist the API-canonical \
             spelling, not a fully-qualified DSL string; \
             got read={read_side:?}"
        );

        // carina-core's state-lift reduces the persisted API values to
        // the canonical fully-qualified `EnumIdentifier` form — the shape the
        // differ reconciles against the parsed-desired side.
        let lifted = carina_core::utils::lift_enum_leaves(&read_side, &attr_type)
            .expect("API-canonical IAM policy doc must lift");
        let Value::Concrete(ConcreteValue::Map(lifted_map)) = &lifted else {
            panic!("expected lifted policy doc to be a Map, got {lifted:?}");
        };
        assert_canonical_enum_ref(
            lifted_map.get("version"),
            "aws.iam.PolicyDocument.Version",
            "2012-10-17",
        );
        let Some(Value::Concrete(ConcreteValue::List(stmts))) = lifted_map.get("statement") else {
            panic!("expected statement list");
        };
        let Some(Value::Concrete(ConcreteValue::Map(stmt))) = stmts.first() else {
            panic!("expected statement map");
        };
        assert_canonical_enum_ref(
            stmt.get("effect"),
            "aws.iam.PolicyDocument.Statement.Effect",
            "Allow",
        );
    }

    // A namespaced `Custom` (region) also flows through this path and
    // is persisted as the hyphen API form, not a DSL string.
    #[test]
    fn test_aws_value_to_dsl_region_in_struct_is_api_canonical() {
        use carina_aws_types::aws_region;

        let fields = vec![
            StructField::new("region_name", aws_region())
                .required()
                .with_provider_name("RegionName"),
        ];
        let attr_type = AttributeType::list(AttributeType::struct_(
            "IpamOperatingRegion".to_string(),
            fields,
        ));
        let json_val = json!([{"RegionName": "ap-northeast-1"}]);

        let result = aws_value_to_dsl_with_defs(
            "operating_regions",
            &json_val,
            &attr_type,
            "ec2.Ipam",
            &std::collections::BTreeMap::new(),
        );

        let expected = Value::Concrete(ConcreteValue::List(vec![Value::Concrete(
            ConcreteValue::Map(IndexMap::from([(
                "region_name".to_string(),
                Value::Concrete(ConcreteValue::String("ap-northeast-1".to_string())),
            )])),
        )]));
        assert_eq!(result, Some(expected));
    }

    #[test]
    fn test_aws_value_to_dsl_enum_value_with_dot() {
        let attr_type = test_enum(
            "Type",
            vec!["ipsec.1".to_string()],
            Some(carina_core::schema::enum_identity(
                "Type",
                Some("aws.ec2.VpnGateway"),
            )),
            vec![],
        );
        let json_val = json!("ipsec.1");

        let result = aws_value_to_dsl_with_defs(
            "type",
            &json_val,
            &attr_type,
            "ec2.VpnGateway",
            &std::collections::BTreeMap::new(),
        );

        // The dotted tail (`ipsec.1`) is the API value itself, not a
        // namespace separator — it must survive verbatim.
        assert_eq!(
            result,
            Some(Value::Concrete(ConcreteValue::String(
                "ipsec.1".to_string()
            )))
        );
    }

    #[test]
    fn test_dsl_value_to_aws_enum_value_with_dot() {
        let attr_type = test_enum(
            "Type",
            vec!["ipsec.1".to_string()],
            Some(carina_core::schema::enum_identity(
                "Type",
                Some("aws.ec2.VpnGateway"),
            )),
            vec![],
        );
        let value = Value::Concrete(ConcreteValue::String(
            "aws.ec2.VpnGateway.Type.ipsec.1".to_string(),
        ));

        let result = dsl_value_to_aws_with_defs(
            &value,
            &attr_type,
            "ec2.VpnGateway",
            "type",
            &std::collections::BTreeMap::new(),
        );

        assert_eq!(result, Some(json!("ipsec.1")));
    }

    #[test]
    fn test_dsl_value_to_aws_enum_plain_dot_value() {
        let attr_type = test_enum(
            "Type",
            vec!["ipsec.1".to_string()],
            Some(carina_core::schema::enum_identity(
                "Type",
                Some("aws.ec2.VpnGateway"),
            )),
            vec![],
        );
        let value = Value::Concrete(ConcreteValue::String("ipsec.1".to_string()));

        let result = dsl_value_to_aws_with_defs(
            &value,
            &attr_type,
            "ec2.VpnGateway",
            "type",
            &std::collections::BTreeMap::new(),
        );

        assert_eq!(result, Some(json!("ipsec.1")));
    }

    #[test]
    fn test_enum_round_trip_with_dotted_value() {
        let attr_type = test_enum(
            "Type",
            vec!["ipsec.1".to_string()],
            Some(carina_core::schema::enum_identity(
                "Type",
                Some("aws.ec2.VpnGateway"),
            )),
            vec![],
        );

        let aws_val = json!("ipsec.1");
        let dsl_val = aws_value_to_dsl_with_defs(
            "type",
            &aws_val,
            &attr_type,
            "ec2.VpnGateway",
            &std::collections::BTreeMap::new(),
        );

        assert_eq!(
            dsl_val,
            Some(Value::Concrete(ConcreteValue::String(
                "ipsec.1".to_string()
            )))
        );

        let back_to_aws = dsl_value_to_aws_with_defs(
            &dsl_val.unwrap(),
            &attr_type,
            "ec2.VpnGateway",
            "type",
            &std::collections::BTreeMap::new(),
        );

        assert_eq!(back_to_aws, Some(json!("ipsec.1")));
    }

    #[test]
    fn test_value_to_json_nan_returns_none() {
        let value = Value::Concrete(ConcreteValue::Float(f64::NAN));
        assert_eq!(value_to_json(&value), None);
    }

    #[test]
    fn test_value_to_json_infinity_returns_none() {
        let value = Value::Concrete(ConcreteValue::Float(f64::INFINITY));
        assert_eq!(value_to_json(&value), None);
    }

    #[test]
    fn test_value_to_json_neg_infinity_returns_none() {
        let value = Value::Concrete(ConcreteValue::Float(f64::NEG_INFINITY));
        assert_eq!(value_to_json(&value), None);
    }

    #[test]
    fn test_value_to_json_finite_float() {
        let value = Value::Concrete(ConcreteValue::Float(1.5));
        let result = value_to_json(&value);
        assert!(result.is_some());
        assert_eq!(result.unwrap(), serde_json::json!(1.5));
    }

    #[test]
    fn test_value_to_json_canonical_enum_uses_api_value() {
        let attr_type = AttributeType::enum_(
            carina_core::schema::enum_identity("Effect", Some("aws.iam.PolicyDocument")),
            Some(vec!["Allow".to_string(), "Deny".to_string()]),
            vec![
                ("Allow".to_string(), "allow".to_string()),
                ("Deny".to_string(), "deny".to_string()),
            ],
            None,
            None,
        );
        let schema = carina_core::schema::Schema::flat(attr_type);
        let value = schema.canonicalize(Value::Concrete(ConcreteValue::enum_identifier("allow")));

        assert!(matches!(
            value,
            Value::Concrete(ConcreteValue::CanonicalEnum(_))
        ));
        assert_eq!(value_to_json(&value), Some(json!("Allow")));
    }

    #[test]
    fn test_json_to_value_array_with_null_drops_null_items() {
        let json = serde_json::json!(["a", null, "b"]);
        let result = json_to_value(&json);
        let expected = Value::Concrete(ConcreteValue::List(vec![
            Value::Concrete(ConcreteValue::String("a".to_string())),
            Value::Concrete(ConcreteValue::String("b".to_string())),
        ]));
        assert_eq!(result, Some(expected));
    }

    #[test]
    fn test_json_to_value_map_with_null_value_drops_entry() {
        let json = serde_json::json!({"key1": "val1", "key2": null});
        let result = json_to_value(&json);
        match result {
            Some(Value::Concrete(ConcreteValue::Map(map))) => {
                assert_eq!(map.len(), 1);
                assert_eq!(
                    map.get("key1"),
                    Some(&Value::Concrete(ConcreteValue::String("val1".to_string())))
                );
                assert!(!map.contains_key("key2"));
            }
            other => panic!("Expected Some(Value::Map), got {:?}", other),
        }
    }

    #[test]
    fn test_aws_value_to_dsl_list_with_null_drops_null_items() {
        let json = serde_json::json!(["a", null, "b"]);
        let attr_type = AttributeType::list(AttributeType::string());
        let result = aws_value_to_dsl_with_defs(
            "test_attr",
            &json,
            &attr_type,
            "test.resource",
            &std::collections::BTreeMap::new(),
        );

        let expected = Value::Concrete(ConcreteValue::List(vec![
            Value::Concrete(ConcreteValue::String("a".to_string())),
            Value::Concrete(ConcreteValue::String("b".to_string())),
        ]));
        assert_eq!(result, Some(expected));
    }

    #[test]
    fn test_value_to_json_list_with_nan_drops_nan_items() {
        let value = Value::Concrete(ConcreteValue::List(vec![
            Value::Concrete(ConcreteValue::Float(1.0)),
            Value::Concrete(ConcreteValue::Float(f64::NAN)),
            Value::Concrete(ConcreteValue::Float(2.0)),
        ]));
        let result = value_to_json(&value);
        let expected = serde_json::json!([1.0, 2.0]);
        assert_eq!(result, Some(expected));
    }

    #[test]
    fn test_lifecycle_configuration_roundtrip_no_spurious_diff() {
        // Regression test for #1346: lifecycle_configuration round-trip through
        // CloudControl API should produce values matching the DSL parser output.
        let config = crate::schemas::generated::s3::bucket::s3_bucket_config();
        let attr_schema = config
            .schema
            .attributes
            .get("lifecycle_configuration")
            .unwrap();

        // lifecycle_configuration must NOT be write-only — only nested sub-properties
        // (Transition singular, ExpiredObjectDeleteMarker, etc.) are write-only in the
        // CloudFormation schema, not the parent attribute itself.
        assert!(
            !attr_schema.write_only,
            "lifecycle_configuration should NOT be marked write-only; \
             only nested sub-properties are write-only in the CFN schema"
        );

        // Simulate AWS CloudControl API response for lifecycle rules.
        // The API returns PascalCase keys and raw enum values.
        let aws_json = serde_json::json!({
            "Rules": [
                {
                    "ExpirationInDays": 90,
                    "Id": "expire-old-objects",
                    "Status": "Enabled"
                },
                {
                    "Id": "transition-to-glacier",
                    "Status": "Enabled",
                    "Transitions": [
                        {
                            "StorageClass": "GLACIER",
                            "TransitionInDays": 30
                        }
                    ]
                }
            ]
        });

        // Read path: convert AWS JSON to DSL value
        let dsl_value = aws_value_to_dsl_with_defs(
            "lifecycle_configuration",
            &aws_json,
            &attr_schema.attr_type,
            "s3.Bucket",
            &config.schema.defs,
        )
        .expect("aws_value_to_dsl should succeed for lifecycle_configuration");

        // Verify the converted value has the expected structure
        let Value::Concrete(ConcreteValue::Map(top_map)) = &dsl_value else {
            panic!("Expected Value::Map, got: {:?}", dsl_value);
        };
        let Some(Value::Concrete(ConcreteValue::List(rules))) = top_map.get("rules") else {
            panic!("Expected 'rules' key with Value::List, got: {:?}", top_map);
        };
        assert_eq!(rules.len(), 2, "Should have 2 lifecycle rules");

        if let Value::Concrete(ConcreteValue::Map(rule0)) = &rules[0] {
            assert_eq!(
                rule0.get("status"),
                Some(&Value::Concrete(ConcreteValue::String(
                    "Enabled".to_string()
                ))),
                "status should be the API-canonical enum value"
            );
            assert_eq!(
                rule0.get("expiration_in_days"),
                Some(&Value::Concrete(ConcreteValue::Int(90))),
                "expiration_in_days should be Int"
            );
            assert_eq!(
                rule0.get("id"),
                Some(&Value::Concrete(ConcreteValue::String(
                    "expire-old-objects".to_string()
                ))),
            );
        } else {
            panic!("Expected rule[0] to be a Map");
        }

        if let Value::Concrete(ConcreteValue::Map(rule1)) = &rules[1] {
            assert_eq!(
                rule1.get("status"),
                Some(&Value::Concrete(ConcreteValue::String(
                    "Enabled".to_string()
                ))),
            );
            if let Some(Value::Concrete(ConcreteValue::List(transitions))) =
                rule1.get("transitions")
            {
                assert_eq!(transitions.len(), 1);
                if let Value::Concrete(ConcreteValue::Map(t)) = &transitions[0] {
                    assert_eq!(
                        t.get("storage_class"),
                        Some(&Value::Concrete(ConcreteValue::String(
                            "GLACIER".to_string()
                        ))),
                    );
                    assert_eq!(
                        t.get("transition_in_days"),
                        Some(&Value::Concrete(ConcreteValue::Int(30))),
                    );
                } else {
                    panic!("Expected transition to be a Map");
                }
            } else {
                panic!("Expected 'transitions' to be a List in rule[1]");
            }
        } else {
            panic!("Expected rule[1] to be a Map");
        }

        // Write path: convert DSL value back to AWS JSON
        let written_json = dsl_value_to_aws_with_defs(
            &dsl_value,
            &attr_schema.attr_type,
            "s3.Bucket",
            "lifecycle_configuration",
            &config.schema.defs,
        )
        .expect("dsl_value_to_aws should succeed");

        assert_eq!(
            written_json, aws_json,
            "Round-trip should produce original AWS JSON"
        );
    }

    /// Regression for #199: a snake_case DSL enum value (here
    /// `bucket_owner_enforced`) must validate against the regenerated
    /// `aws.s3.Bucket.OwnershipControls.OwnershipControlsRule.ObjectOwnership` schema,
    /// and `dsl_value_to_aws` must round-trip it back to the AWS
    /// spelling (`BucketOwnerEnforced`).
    #[test]
    fn test_object_ownership_snake_case_validates_and_roundtrips() {
        // Walk the schema to find the nested ObjectOwnership StringEnum:
        // s3.Bucket -> ownership_controls -> rules[] -> object_ownership.
        let config = crate::schemas::generated::s3::bucket::s3_bucket_config();
        let ownership_controls = config
            .schema
            .attributes
            .get("ownership_controls")
            .expect("s3.Bucket has ownership_controls");
        // carina#3340: reuse-via-Ref may produce `AttributeType::Ref`
        // at this position; resolve through `schema.defs` so the test
        // still sees the underlying Struct.
        let Shape::Struct { .. } = config.schema.shape_of(&ownership_controls.attr_type) else {
            panic!("ownership_controls is a Struct");
        };
        let fields = config
            .schema
            .struct_fields_with_budget(
                &ownership_controls.attr_type,
                &mut ShapeWalkBudget::new(256),
            )
            .expect("ownership_controls exposes fields");
        let rules = fields.iter().find(|f| f.name == "rules").unwrap();
        let Shape::List {
            element_type: inner,
            ..
        } = config.schema.shape_of(&rules.field_type)
        else {
            panic!("rules is a List");
        };
        let Shape::Struct { .. } = config.schema.shape_of(inner) else {
            panic!("rules.inner is a Struct");
        };
        let rule_fields = config
            .schema
            .struct_fields_with_budget(inner, &mut ShapeWalkBudget::new(256))
            .expect("rules.inner exposes fields");
        let object_ownership = rule_fields
            .iter()
            .find(|f| f.name == "object_ownership")
            .unwrap();

        // Validate the snake_case form (the canonical DSL spelling per D7).
        // Phase 4 of carina#2986: DSL-source enum values arrive as
        // `EnumIdentifier`; a `String` here would route to
        // `StringLiteralExpectedEnum`.
        let snake_case_value = Value::Concrete(ConcreteValue::enum_identifier(
            "aws.s3.Bucket.OwnershipControls.OwnershipControlsRule.ObjectOwnership.bucket_owner_enforced",
        ));
        let ownership_schema =
            carina_core::schema::Schema::flat(object_ownership.field_type.clone());
        ownership_schema
            .validate(&snake_case_value)
            .expect("snake_case DSL spelling must validate");

        // After carina-rs/carina#2980 / awscc#222 the validator is
        // strict on DSL input — the PascalCase API spelling is
        // rejected. State JSON still flows through `aws_value_to_dsl`
        // separately, so this only gates DSL-source values.
        let pascal_value = Value::Concrete(ConcreteValue::enum_identifier(
            "aws.s3.Bucket.OwnershipControls.OwnershipControlsRule.ObjectOwnership.BucketOwnerEnforced",
        ));
        assert!(
            ownership_schema.validate(&pascal_value).is_err(),
            "PascalCase API spelling must be rejected in DSL position",
        );

        // Round-trip: DSL snake_case -> AWS API spelling.
        let aws_json = dsl_value_to_aws_with_defs(
            &snake_case_value,
            &object_ownership.field_type,
            "s3.Bucket",
            "object_ownership",
            &std::collections::BTreeMap::new(),
        )
        .expect("dsl_value_to_aws must succeed");
        assert_eq!(aws_json, json!("BucketOwnerEnforced"));

        // On read the AWS spelling is persisted verbatim, then lifted.
        let dsl = aws_value_to_dsl_with_defs(
            "object_ownership",
            &json!("BucketOwnerEnforced"),
            &object_ownership.field_type,
            "s3.Bucket",
            &std::collections::BTreeMap::new(),
        )
        .expect("aws_value_to_dsl must succeed");
        assert_eq!(
            dsl,
            Value::Concrete(ConcreteValue::String("BucketOwnerEnforced".to_string())),
            "read must persist the API-canonical value"
        );
        let lifted = carina_core::utils::lift_enum_leaves(&dsl, &object_ownership.field_type)
            .expect("API-canonical state value must lift");
        assert_canonical_enum(
            &lifted,
            "aws.s3.Bucket.OwnershipControls.OwnershipControlsRule.ObjectOwnership",
            "BucketOwnerEnforced",
        );
    }

    #[test]
    fn test_dsl_value_to_aws_list_with_nan_drops_nan_items() {
        let value = Value::Concrete(ConcreteValue::List(vec![
            Value::Concrete(ConcreteValue::Float(1.0)),
            Value::Concrete(ConcreteValue::Float(f64::NAN)),
            Value::Concrete(ConcreteValue::Float(2.0)),
        ]));
        let attr_type = AttributeType::list(AttributeType::float());
        let result = dsl_value_to_aws_with_defs(
            &value,
            &attr_type,
            "test.resource",
            "test_attr",
            &std::collections::BTreeMap::new(),
        );

        let expected = serde_json::json!([1.0, 2.0]);
        assert_eq!(result, Some(expected));
    }

    #[test]
    fn test_aws_value_to_dsl_refined_string_to_dsl_strips_trailing_dot() {
        let attr_type = AttributeType::refined_string(
            None,
            None,
            None,
            Some(DslTransform::StripSuffix(".".to_string())),
        );
        let json_val = serde_json::json!("carina-rs.dev.");
        let result = aws_value_to_dsl_with_defs(
            "name",
            &json_val,
            &attr_type,
            "route53.HostedZone",
            &std::collections::BTreeMap::new(),
        );

        assert_eq!(
            result,
            Some(Value::Concrete(ConcreteValue::String(
                "carina-rs.dev".to_string()
            )))
        );
    }

    #[test]
    fn test_aws_value_to_dsl_custom_without_to_dsl_passes_through() {
        let attr_type = AttributeType::refined_string_with_validator(
            Some(carina_core::schema::TypeIdentity::bare("DnsName")),
            None,
            None,
            legacy_validator(|_| Ok(())),
            None,
        );
        let json_val = serde_json::json!("carina-rs.dev.");
        let result = aws_value_to_dsl_with_defs(
            "name",
            &json_val,
            &attr_type,
            "route53.HostedZone",
            &std::collections::BTreeMap::new(),
        );

        assert_eq!(
            result,
            Some(Value::Concrete(ConcreteValue::String(
                "carina-rs.dev.".to_string()
            )))
        );
    }
}
