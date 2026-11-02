//! CloudFormation input model and shared property-path predicates.

use anyhow::Result;
use heck::{ToPascalCase, ToSnakeCase};
use serde::Deserialize;
use std::collections::BTreeMap;

/// CloudFormation Resource Schema
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
pub(super) struct CfnSchema {
    pub(super) type_name: String,
    pub(super) description: Option<String>,
    pub(super) properties: BTreeMap<String, CfnProperty>,
    #[serde(default)]
    pub(super) required: Vec<String>,
    #[serde(default)]
    pub(super) read_only_properties: Vec<String>,
    #[serde(default)]
    pub(super) create_only_properties: Vec<String>,
    #[serde(default)]
    pub(super) write_only_properties: Vec<String>,
    pub(super) primary_identifier: Option<Vec<String>>,
    pub(super) definitions: Option<BTreeMap<String, CfnProperty>>,
    pub(super) tagging: Option<CfnTagging>,
    /// Top-level oneOf variants (mutually exclusive required field groups)
    #[serde(default, rename = "oneOf")]
    pub(super) one_of: Vec<CfnProperty>,
    /// Top-level anyOf variants
    #[serde(default, rename = "anyOf")]
    pub(super) any_of: Vec<CfnProperty>,
    /// Cloud Control handlers (create/read/update/delete/list). Most
    /// NON_PROVISIONABLE types omit this map, but the registry has known
    /// provisionable exceptions; see codegen's `HANDLERLESS_CLOUD_CONTROL_TYPES`.
    #[serde(default)]
    pub(super) handlers: BTreeMap<String, serde_json::Value>,
}

/// Type can be a string or an array of strings in JSON Schema
#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub(super) enum TypeValue {
    Single(String),
    Multiple(Vec<String>),
}

impl TypeValue {
    pub(super) fn contains(&self, expected: &str) -> bool {
        match self {
            Self::Single(value) => value == expected,
            Self::Multiple(values) => values.iter().any(|value| value == expected),
        }
    }

    /// Ignore nullability when choosing the type to generate. For other
    /// unions, retain the first non-null member as the compatibility type.
    pub(super) fn effective_type(&self) -> Option<&str> {
        match self {
            TypeValue::Single(value) => (value != "null").then_some(value.as_str()),
            TypeValue::Multiple(values) => values
                .iter()
                .find(|value| value.as_str() != "null")
                .map(String::as_str),
        }
    }
}

/// Enum value can be a string or an integer in JSON Schema
#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub(super) enum EnumValue {
    Str(String),
    Int(i64),
}

impl EnumValue {
    pub(super) fn to_string_value(&self) -> String {
        match self {
            EnumValue::Str(s) => s.clone(),
            EnumValue::Int(i) => i.to_string(),
        }
    }
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
pub(super) struct CfnProperty {
    #[serde(rename = "type")]
    pub(super) prop_type: Option<TypeValue>,
    pub(super) description: Option<String>,
    #[serde(rename = "enum")]
    pub(super) enum_values: Option<Vec<EnumValue>>,
    pub(super) items: Option<Box<CfnProperty>>,
    #[serde(rename = "$ref")]
    pub(super) ref_path: Option<String>,
    #[serde(default)]
    pub(super) insertion_order: Option<bool>,
    /// Whether array items must be unique.
    #[serde(default)]
    pub(super) unique_items: Option<bool>,
    /// Inline object properties (for nested objects)
    pub(super) properties: Option<BTreeMap<String, CfnProperty>>,
    /// JSON object-map value schemas keyed by key-pattern.
    #[serde(default)]
    pub(super) pattern_properties: Option<BTreeMap<String, CfnProperty>>,
    /// Required fields for inline objects
    #[serde(default)]
    pub(super) required: Vec<String>,
    /// Minimum value constraint (for integer/number types)
    #[serde(default)]
    pub(super) minimum: Option<i64>,
    /// Maximum value constraint (for integer/number types)
    #[serde(default)]
    pub(super) maximum: Option<i64>,
    /// Whether additional properties are allowed (for object types)
    #[serde(default)]
    pub(super) additional_properties: Option<bool>,
    /// Const value (single fixed value for this property)
    #[serde(rename = "const")]
    pub(super) const_value: Option<serde_json::Value>,
    /// Default value for this property
    #[serde(rename = "default")]
    pub(super) default_value: Option<serde_json::Value>,
    /// Regex pattern constraint (for string types)
    #[serde(default)]
    pub(super) pattern: Option<String>,
    /// Minimum number of items (for array types)
    #[serde(default)]
    pub(super) min_items: Option<i64>,
    /// Maximum number of items (for array types)
    #[serde(default)]
    pub(super) max_items: Option<i64>,
    /// Minimum length constraint (for string types)
    #[serde(default, rename = "minLength")]
    pub(super) min_length: Option<u64>,
    /// Maximum length constraint (for string types)
    #[serde(default, rename = "maxLength")]
    pub(super) max_length: Option<u64>,
    /// Format constraint (e.g., "int64", "uri", "double")
    #[serde(default)]
    pub(super) format: Option<String>,
    /// Property-level oneOf variants (union types)
    #[serde(default, rename = "oneOf")]
    pub(super) one_of: Vec<CfnProperty>,
    #[serde(default)]
    #[serde(rename = "anyOf")]
    pub(super) any_of: Vec<CfnProperty>,
}

pub(super) fn decode_json_pointer_segment(segment: &str) -> String {
    segment.replace("~1", "/").replace("~0", "~")
}

pub(super) fn cfn_property_path(pointer: &str) -> Option<Vec<String>> {
    let path = pointer.strip_prefix("/properties/")?;
    if path.is_empty() {
        return None;
    }
    Some(path.split('/').map(decode_json_pointer_segment).collect())
}

pub(super) fn extend_property_path(property_path: &[String], segment: &str) -> Vec<String> {
    let mut extended = property_path.to_vec();
    extended.push(segment.to_string());
    extended
}

pub(super) fn is_read_only_property_path(schema: &CfnSchema, property_path: &[String]) -> bool {
    schema
        .read_only_properties
        .iter()
        .filter_map(|pointer| cfn_property_path(pointer))
        .any(|path| path == property_path)
}

/// Extract the definition name from a $ref path
/// e.g., "#/definitions/Ingress" -> Some("Ingress")
pub(super) fn ref_def_name(ref_path: &str) -> Option<&str> {
    ref_path.strip_prefix("#/definitions/")
}

/// Compute DSL resource name (service.resource) from CloudFormation type name
/// e.g., "AWS::EC2::SecurityGroupEgress" -> "ec2.SecurityGroupEgress"
/// and "AWS::EC2::VPC" -> "ec2.Vpc" (acronyms normalised via snake -> pascal).
/// The service segment is lowercase (a namespace); the resource segment is
/// PascalCase with acronyms treated as single words, matching carina-core's
/// `snake_to_pascal` round-trip (design D2).
pub(super) fn dsl_resource_name_from_type(type_name: &str) -> Result<String> {
    let parts: Vec<&str> = type_name.split("::").collect();
    if parts.len() != 3 {
        anyhow::bail!("Invalid type name format: {}", type_name);
    }
    let service = parts[1].to_lowercase();
    // Route through snake_case first so CloudFormation-style acronyms
    // ("VPC", "IPAM", "EIP") become single Pascal tokens ("Vpc", "Ipam",
    // "Eip") rather than shouty residue.
    let resource = parts[2].to_snake_case().to_pascal_case();
    Ok(format!("{}.{}", service, resource))
}

/// CloudFormation Tagging metadata
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
pub(super) struct CfnTagging {
    #[serde(default)]
    pub(super) taggable: bool,
    pub(super) tag_property: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(super) struct CfnPropertyPath<'a> {
    pub(super) parent: Option<&'a str>,
    pub(super) property: &'a str,
}

impl<'a> CfnPropertyPath<'a> {
    /// Build the definition-local override key for a CFN property.
    /// `parent` is the raw definition name (or raw inline-property name),
    /// never the PascalCased Rust struct namespace. KMS sites use full paths.
    pub(super) const fn new(parent: Option<&'a str>, property: &'a str) -> Self {
        Self { parent, property }
    }
}

impl std::fmt::Display for CfnPropertyPath<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(parent) = self.parent {
            write!(f, "{parent}.{}", self.property)
        } else {
            f.write_str(self.property)
        }
    }
}
